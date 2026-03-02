// Copyright (c) Kiran Ayyagari. All rights reserved.
// Copyright (c) Diridium Technologies Inc. All rights reserved.
// Licensed under the MPL-2.0 License. See LICENSE file in the project root.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use anyhow::Error;
use openssl::x509::store::X509StoreRef;
use openssl::x509::X509;
use reqwest::blocking::{Client, ClientBuilder};
use reqwest::Url;
use roxmltree::Node;
use rustc_hash::FxHashMap;
use sha2::{Digest, Sha256};
use tauri::ipc::Channel;

use crate::connection::ConnectionEntry;
use crate::errors::VerificationError;
use crate::verify::verify_jar;

#[derive(Debug)]
#[allow(dead_code)]
pub struct WebstartFile {
    url: String,
    main_class: String,
    args: Vec<String>,
    j2ses: Option<Vec<J2se>>,
    jar_dir: PathBuf,
    loaded_at: SystemTime,
}

/// from jnlp -> resources -> j2se
#[derive(Debug)]
pub struct J2se {
    java_vm_args: Option<String>,
    version: String,
}

pub struct WebStartCache {
    cache: Mutex<FxHashMap<String, Arc<WebstartFile>>>,
}

impl WebStartCache {
    pub fn init() -> Self {
        let cache = Mutex::new(FxHashMap::default());
        WebStartCache { cache }
    }

    pub fn get(&self, url: &str) -> Option<Arc<WebstartFile>> {
        let cache = self.cache.lock().expect("webstart cache lock poisoned");
        let wf = cache.get(url);
        if let Some(wf) = wf {
            let now = SystemTime::now();
            let elapsed = now
                .duration_since(wf.loaded_at)
                .expect("failed to calculate the duration");
            if elapsed.as_secs() < 120 {
                return Some(Arc::clone(wf));
            }
        }
        None
    }
}

impl WebstartFile {
    pub fn load(base_url: &str, cache_dir: &PathBuf, donotcache: bool, conn_id: &str, conn_name: &str, on_progress: &Channel<serde_json::Value>) -> Result<WebstartFile, Error> {
        let (base_url, _host) = normalize_url(base_url)?;
        let webstart = format!("{}/webstart.jnlp", base_url); // base_url will never contain a / at the end after normalization
        let _ = on_progress.send(serde_json::json!({"message": "Fetching server configuration..."}));
        let cb = ClientBuilder::default()
            // in certain network environments client is failing with error message "connection closed before message completed"
            // disabling the pooling resolved the issue
            .pool_max_idle_per_host(0)
            // accept any cert presented by the MC server
            .danger_accept_invalid_certs(true);
        let client = cb.build()?;

        let r = client.get(&webstart).send()?;
        let data = r.text()?;
        let doc = roxmltree::Document::parse(&data)?;

        let root = doc.root();
        let main_class_node = get_node(&root, "application-desc").ok_or(Error::msg(
            "Got something from MC that was not an application-desc node in a JNLP XML",
        ))?;
        let main_class = main_class_node
            .attribute("main-class")
            .ok_or(Error::msg("missing main-class attribute"))?
            .to_string();
        let args = get_client_args(&main_class_node);

        let resources_node = get_node(&root, "resources");

        let mut version = "default".to_string();
        if let Some(jnlp_node) = get_node(&root, "jnlp") {
            if let Some(v) = jnlp_node.attribute("version") {
                // Sanitize to prevent path traversal (e.g. "../../.ssh")
                version = v.replace(['/', '\\', '.'], "_");
            }
        }

        let sanitized_name = conn_name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>();
        let id_prefix = &conn_id[..conn_id.len().min(8)];
        let cache_folder = format!("{}_{}", sanitized_name, id_prefix);
        let jar_dir = cache_dir.join(cache_folder).join(&version);
        if donotcache && jar_dir.exists() {
            println!("removing directory {:?}", jar_dir);
            std::fs::remove_dir_all(&jar_dir)?;
        }

        let dir_path = jar_dir.as_path();
        if !jar_dir.exists() {
            println!("creating directory {:?}", jar_dir);
            std::fs::create_dir_all(dir_path)?;
        }

        let mut j2ses = None;
        if let Some(resources_node) = resources_node {
            j2ses = get_j2ses(&resources_node);
            download_jars(&resources_node, &client, dir_path, &base_url, on_progress)?;
        }

        let loaded_at = SystemTime::now();
        let ws = WebstartFile {
            url: base_url.to_string(),
            main_class,
            jar_dir,
            args,
            loaded_at,
            j2ses,
        };

        Ok(ws)
    }

    pub fn run(&self, ce: Arc<ConnectionEntry>, console_jar: Option<PathBuf>) -> Result<(), Error> {
        let itr = self.jar_dir.read_dir()?;
        let mut classpath = String::with_capacity(1152);
        let mut classpath_suffix = String::with_capacity(1024);
        for e in itr {
            let e = e?;
            if e.metadata()?.is_dir() {
                continue;
            }
            let file_path = e.path();
            let file_name = match file_path.file_name().and_then(|f| f.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };
            let file_path_str = match file_path.to_str() {
                Some(p) => p,
                None => continue,
            };

            //In Windows the CP separator is ';' and literally every other OS is ':'
            let classpath_separator = if cfg!(windows) { ';' } else { ':' };

            // MirthConnect's own jars contain some overridden classes
            // of the dependent libraries and hence must be loaded first
            // https://forums.mirthproject.io/forum/mirth-connect/support/15524-using-com-mirth-connect-client-core-client
            //TODO this should probably build the classpath objects as an ordered set, then do a .join(classpath_separator)
            if file_name.starts_with("mirth") {
                classpath.push_str(file_path_str);
                classpath.push(classpath_separator);
            } else {
                classpath_suffix.push_str(file_path_str);
                classpath_suffix.push(classpath_separator);
            }
        }

        classpath.push_str(&classpath_suffix);

        let mut cmd;
        let java_home = ce.java_home.trim();
        if java_home.is_empty() {
            cmd = Command::new("java")
        } else {
            cmd = Command::new(format!("{}/bin/java", java_home));
        }

        println!("using java from: {:?}", cmd.get_program().to_str());

        if let Some(ref vm_args) = self.j2ses {
            for va in vm_args {
                // if there are VM args for java version >= 1.9
                // then set the JDK_JAVA_OPTIONS environment variable
                // this will be ignored by java version <= 1.8
                if va.version.contains("1.9") {
                    if let Some(java_vm_args) = &va.java_vm_args {
                        let filtered = sanitize_vm_args(java_vm_args);
                        if !filtered.is_empty() {
                            println!("setting JDK_JAVA_OPTIONS environment variable with the java-vm-args given for version {} in JNLP file", va.version);
                            cmd.env("JDK_JAVA_OPTIONS", &filtered);
                        }
                    }
                }
            }
        }

        let heap = ce.heap_size.trim();
        if !heap.is_empty() {
            cmd.arg(format!("-Xmx{}", heap));
        }

        if let Some(args) = ce.java_args.as_deref() {
            // Should probably do some sanitization here...
            cmd.args(args.trim().lines());
        }

        cmd.arg("-cp")
            .arg(classpath)
            .arg(&self.main_class)
            .args(&self.args);

        if let Some(ref username) = ce.username {
            cmd.arg(username);
            if let Some(ref password) = ce.password {
                cmd.arg(password);
            }
        }

        if ce.show_console {
            let console_jar = console_jar
                .ok_or(Error::msg("Java console jar path not provided"))?;

            // Launch the Java Console as a separate Java Swing process
            let java_bin = if java_home.is_empty() {
                "java".to_string()
            } else {
                format!("{}/bin/java", java_home)
            };

            let mut console_cmd = Command::new(&java_bin);
            console_cmd
                .arg("-Xmx256m")
                .arg("-cp")
                .arg(console_jar.to_str().ok_or_else(|| Error::msg("console jar path is not valid UTF-8"))?)
                .arg("com.innovarhealthcare.launcher.JavaConsoleDialog")
                .stdin(Stdio::piped());
            #[cfg(windows)]
            console_cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
            let mut console_proc = console_cmd.spawn()?;

            // Launch the target process with stdout piped to the console
            // stderr inherits (default) so it doesn't block the process
            cmd.stdout(Stdio::piped());
            #[cfg(windows)]
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
            let mut target_proc = cmd.spawn()?;

            // Pipe target stdout → console stdin in a background thread
            let target_stdout = target_proc.stdout.take();
            let console_stdin = console_proc.stdin.take();
            if let (Some(stdout), Some(stdin)) = (target_stdout, console_stdin) {
                std::thread::spawn(move || {
                    use std::io::{Read, Write};
                    let mut stdout = stdout;
                    let mut stdin = stdin;
                    let mut buf = [0u8; 1024];
                    loop {
                        match stdout.read(&mut buf) {
                            Ok(0) => break,
                            Ok(n) => {
                                let _ = stdin.write_all(&buf[..n]);
                                let _ = stdin.flush();
                            }
                            Err(_) => break,
                        }
                    }
                    // Target process exited — kill the console window
                    let _ = console_proc.kill();
                });
            }
        } else {
            cmd.stdout(Stdio::inherit());
            cmd.stderr(Stdio::inherit());
            #[cfg(windows)]
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
            cmd.spawn()?;
        }

        Ok(())
    }

    pub fn verify(&self, cert_store: &X509StoreRef, trusted_certs: &[X509]) -> Result<(), VerificationError> {
        let mut jar_files = Vec::with_capacity(128);
        let itr = self
            .jar_dir
            .read_dir()
            .map_err(|e| VerificationError {
                cert: None,
                msg: format!("failed to read jar files directory: {}", e),
            })?;
        for e in itr {
            let e = e.map_err(|e| VerificationError {
                cert: None,
                msg: format!("failed to list directory entry: {}", e),
            })?;
            let file_path = e.path();
            jar_files.push(file_path);
        }

        jar_files.sort_unstable();
        println!("{:?}", jar_files);

        for jf in jar_files {
            let file_path = jf.to_str().ok_or_else(|| VerificationError {
                cert: None,
                msg: format!("jar file path is not valid UTF-8: {:?}", jf),
            })?;
            verify_jar(file_path, cert_store, trusted_certs)?;
        }
        Ok(())
    }
}

fn download_jars(
    resources_node: &Node,
    client: &Client,
    dir_path: &Path,
    base_url: &str,
    on_progress: &Channel<serde_json::Value>,
) -> Result<(), Error> {
    let mut counter = 0usize;
    for n in resources_node.children() {
        let jar = n.has_tag_name("jar");
        let extension = n.has_tag_name("extension");

        if !jar && !extension {
            continue;
        }

        let href = match n.attribute("href") {
            Some(h) => h,
            None => continue,
        };
        let hash_in_jnlp = n.attribute("sha256");
        let url = format!("{}/{}", base_url, href);

        if jar {
            let file_name = get_file_name_from_path(href);
            counter += 1;
            let jar_file_path = dir_path.join(file_name);
            let _ = on_progress.send(serde_json::json!({
                "message": format!("Verifying cache file {}", file_name),
            }));
            if has_file_changed(&jar_file_path, hash_in_jnlp)? {
                let _ = on_progress.send(serde_json::json!({
                    "message": format!("Downloading {} ({})", file_name, counter),
                }));
                let mut resp = client.get(url).send()?;
                let mut f = File::create(&jar_file_path)?;
                resp.copy_to(&mut f)?;
            }
        } else if extension {
            let r = client.get(url).send()?;
            let data = r.text()?;
            let doc = roxmltree::Document::parse(&data)?;
            let root = doc.root();
            let resources_node = get_node(&root, "resources");
            let ext_base_url = format!("{}/webstart/extensions", base_url);
            if let Some(resources_node) = resources_node {
                download_jars(&resources_node, client, dir_path, &ext_base_url, on_progress)?;
            }
        }
    }

    Ok(())
}

/// Filter JNLP java-vm-args to block flags that could execute arbitrary code.
fn sanitize_vm_args(args: &str) -> String {
    let dangerous_prefixes: &[&str] = &[
        "-javaagent:",
        "-agentpath:",
        "-agentlib:",
        "-xbootclasspath",
        "-xx:onoutofmemoryerror",
        "-xx:onoutofmemoryerror=",
        "-xx:onerror",
        "-xx:onerror=",
    ];

    args.split_whitespace()
        .filter(|arg| {
            let lower = arg.to_lowercase();
            let dominated = dangerous_prefixes.iter().any(|p| lower.starts_with(p));
            if dominated {
                println!("sanitize_vm_args: dropping dangerous flag: {}", arg);
            }
            !dominated
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn get_file_name_from_path(p: &str) -> &str {
    p.rsplit('/').next().unwrap_or(p)
}

fn get_client_args(root: &Node) -> Vec<String> {
    let mut args = Vec::new();
    for n in root.descendants() {
        if n.has_tag_name("argument") {
            if let Some(text) = n.text() {
                args.push(text.to_string());
            }
        }
    }
    args
}

fn get_j2ses(resources: &Node) -> Option<Vec<J2se>> {
    let mut j2ses = Vec::new();
    for n in resources.descendants() {
        if n.has_tag_name("j2se") {
            // only consider those that have java-vm-args and version
            if let Some(java_vm_args) = n.attribute("java-vm-args") {
                if let Some(version) = n.attribute("version") {
                    let java_vm_args = Some(java_vm_args.to_string());
                    let j2se = J2se {
                        java_vm_args,
                        version: version.to_string(),
                    };
                    j2ses.push(j2se);
                }
            }
        }
    }
    if !j2ses.is_empty() {
        return Some(j2ses);
    }
    None
}

fn get_node<'a>(root: &'a Node, tag_name: &str) -> Option<Node<'a, 'a>> {
    root.descendants().find(|n| {
        if n.has_tag_name(tag_name) {
            return true;
        }
        return false;
    })
}

fn normalize_url(u: &str) -> Result<(String, String), Error> {
    let parsed_url = Url::parse(u)?;
    let mut reconstructed_url = String::with_capacity(u.len());
    reconstructed_url.push_str(parsed_url.scheme());
    reconstructed_url.push_str("://");
    let host = parsed_url.host_str().map_or("", |h| h);
    reconstructed_url.push_str(host);
    let port = parsed_url
        .port()
        .map_or("".to_string(), |p| format!(":{}", p));
    reconstructed_url.push_str(&port);
    reconstructed_url.push('/');
    let path_parts = parsed_url.path().split_terminator("/");
    for pp in path_parts {
        if !pp.is_empty() {
            reconstructed_url.push_str(pp);
            reconstructed_url.push('/');
        }
    }

    reconstructed_url.pop(); // remove the trailing /
    let host = format!("{}{}", host, port).replace(":", "_");
    Ok((reconstructed_url, host))
}

fn has_file_changed(jar_file_path: &Path, hash_in_jnlp: Option<&str>) -> Result<bool, Error> {
    if let Some(hash_in_jnlp) = hash_in_jnlp {
        let mut hasher = Sha256::new();
        if jar_file_path.exists() {
            let jar_file = File::open(&jar_file_path)?;
            let mut reader = BufReader::new(&jar_file);
            let mut buf = [0; 2048];
            while let Ok(count) = reader.read(&mut buf) {
                if count <= 0 {
                    break;
                }
                hasher.update(&buf[..count]);
            }
            let val = hasher.finalize();
            let val = openssl::base64::encode_block(val.as_slice());
            return Ok(hash_in_jnlp != &val);
        }
    }

    Ok(true)
}
#[cfg(test)]
mod tests {
    use crate::webstart::normalize_url;
    use anyhow::Error;

    #[test]
    pub fn test_normalize_url() -> Result<(), Error> {
        let candidates = [
            ("https://localhost:8443", "https://localhost:8443"),
            ("https://localhost:8443/", "https://localhost:8443"),
            ("https://localhost:8443//", "https://localhost:8443"),
            (
                "https://localhost:8443//a///bv",
                "https://localhost:8443/a/bv",
            ),
        ];

        for (src, expected) in candidates {
            let (reconstructed_url, _host) = normalize_url(src)?;
            assert_eq!(expected, &reconstructed_url);
        }
        Ok(())
    }
}
