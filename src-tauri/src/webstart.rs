use std::{env, io};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use anyhow::Error;
use hex::encode;
use openssl::x509::store::X509StoreRef;
use reqwest::blocking::{Client, ClientBuilder};
use reqwest::Url;
use roxmltree::Node;
use rustc_hash::FxHashMap;
use sha2::{Digest, Sha256};

use crate::connection::ConnectionEntry;
use crate::errors::VerificationError;
use crate::verify::verify_jar;

#[derive(Debug)]
pub struct WebstartFile {
    url: String,
    main_class: String,
    args: Vec<String>,
    j2ses: Option<Vec<J2se>>,
    //jars: Vec<Jar>,
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

    pub fn put(&mut self, wf: Arc<WebstartFile>) {
        self.cache.lock().unwrap().insert(wf.url.clone(), wf);
    }

    pub fn get(&self, url: &str) -> Option<Arc<WebstartFile>> {
        let cache = self.cache.lock().unwrap();
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
    pub fn load(base_url: &str, cache_dir: &PathBuf, donotcache: bool) -> Result<WebstartFile, Error> {
        let (base_url, host) = normalize_url(base_url)?;
        let webstart = format!("{}/webstart.jnlp", base_url); // base_url will never contain a / at the end after normalization
        let cb = ClientBuilder::default()
            // in certain network environments client is failing with error message "connection closed before message completed"
            // disabling the pooling resolved the issue
            .pool_max_idle_per_host(0)
            // accept any cert presented by the MC server
            .danger_accept_invalid_certs(true);
        let client = cb.build()?;

        let r = client.get(&webstart).send()?;
        let data = r.text()?;
        //TODO VERY NOISY, is there a log level lower than debug?
        //println!("Got response from MC as: {:?}", data);
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

        let mut version = "default";
        if let Some(jnlp_node) = get_node(&root, "jnlp") {
            if let Some(v) = jnlp_node.attribute("version") {
                version = v;
            }
        }

        let jar_dir = cache_dir.join(host).join(version);
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
            download_jars(&resources_node, &client, dir_path, &base_url)?;
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

    pub fn run(&self, ce: Arc<ConnectionEntry>) -> Result<(), Error> {
        let itr = self.jar_dir.read_dir()?;
        let mut classpath = String::with_capacity(1152);
        let mut classpath_suffix = String::with_capacity(1024);
        for e in itr {
            let e = e?;
            if e.metadata().unwrap().is_dir() {
                continue;
            }
            let file_path = e.path();
            let file_name = file_path.file_name().unwrap();
            let file_path = file_path.as_os_str();
            let file_path = file_path.to_str().unwrap();

            //In Windows the CP separator is ';' and literally every other OS is ':'
            let classpath_separator = if cfg!(windows) { ';' } else { ':' };

            //println!("{}", file_path);
            // MirthConnect's own jars contain some overridden classes
            // of the dependent libraries and hence must be loaded first
            // https://forums.mirthproject.io/forum/mirth-connect/support/15524-using-com-mirth-connect-client-core-client
            //TODO this should probably build the classpath objects as an ordered set, then do a .join(classpath_separator)
            if file_name.to_str().unwrap().starts_with("mirth") {
                classpath.push_str(file_path);
                classpath.push(classpath_separator);
            } else {
                classpath_suffix.push_str(file_path);
                classpath_suffix.push(classpath_separator);
            }
        }

        classpath.push_str(&classpath_suffix);

        //println!("class path: {}", classpath);
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
                        println!("setting JDK_JAVA_OPTIONS environment variable with the java-vm-args given for version {} in JNLP file", va.version);
                        cmd.env("JDK_JAVA_OPTIONS", java_vm_args);
                    }
                }
            }
        }

        let heap = ce.heap_size.trim();
        if !heap.is_empty() {
            cmd.arg(format!("-Xmx{}", heap));
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

        let log_file_path = env::temp_dir().join("ballista.log");
        println!("log_file_path {:?}", log_file_path);
        let f = File::create(log_file_path)?;
        cmd.stdout(Stdio::from(f));
        //TODO noisy, should be a debug logger
        //println!("Executing with: {:?}", cmd);
        cmd.spawn()?;
        Ok(())
    }

    pub fn verify(&self, cert_store: &X509StoreRef) -> Result<(), VerificationError> {
        let mut jar_files = Vec::with_capacity(128);
        let itr = self
            .jar_dir
            .read_dir()
            .expect("failed to read the jar files directory");
        for e in itr {
            let e = e.expect("failed to list a directory entry");
            let file_path = e.path();
            jar_files.push(file_path);
        }

        jar_files.sort_unstable();
        println!("{:?}", jar_files);

        for jf in jar_files {
            let file_path = jf.as_os_str();
            let file_path = file_path.to_str().unwrap();
            verify_jar(file_path, cert_store)?;
        }
        Ok(())
    }
}

fn download_jars(
    resources_node: &Node,
    client: &Client,
    dir_path: &Path,
    base_url: &str,
) -> Result<(), Error> {
    for n in resources_node.children() {
        let jar = n.has_tag_name("jar");
        let extension = n.has_tag_name("extension");

        if !jar && !extension {
            continue;
        }

        let href = n.attribute("href").unwrap();
        let hash_in_jnlp = n.attribute("sha256");
        let url = format!("{}/{}", base_url, href);

        if jar {
            let file_name = get_file_name_from_path(href);
            let jar_file_path = dir_path.join(file_name);
            if has_file_changed(&jar_file_path, hash_in_jnlp)? {
                //println!("downloading file {}", file_name);
                let mut resp = client.get(url).send()?;
                let mut f = File::create(&jar_file_path)?;
                resp.copy_to(&mut f)?;
            }
            else {
                //println!("file {} is cached", file_name);
            }
        } else if extension {
            let r = client.get(url).send()?;
            let data = r.text()?;
            let doc = roxmltree::Document::parse(&data)?;
            let root = doc.root();
            let resources_node = get_node(&root, "resources");
            let ext_base_url = format!("{}/webstart/extensions", base_url);
            if let Some(resources_node) = resources_node {
                download_jars(&resources_node, client, dir_path, &ext_base_url)?;
            }
        }
    }

    Ok(())
}

fn get_file_name_from_path(p: &str) -> &str {
    let mut itr = p.rsplit_terminator("/");
    itr.next().unwrap()
}

fn get_client_args(root: &Node) -> Vec<String> {
    let mut args = Vec::new();
    for n in root.descendants() {
        if n.has_tag_name("argument") {
            args.push(n.text().unwrap().to_string());
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
    let mut path_parts = parsed_url.path().split_terminator("/");
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
            let actual = normalize_url(src)?;
            assert_eq!(expected, &actual);
        }
        Ok(())
    }
}
