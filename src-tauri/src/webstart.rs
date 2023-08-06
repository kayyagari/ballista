use std::fs::File;
use std::{env, os};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime};

use anyhow::Error;
use openssl::x509::store::X509StoreRef;
use reqwest::blocking::{Client, ClientBuilder};
use roxmltree::Node;
use rustc_hash::FxHashMap;
use sha2::{Digest, Sha256};

use crate::connection::{ConnectionEntry, ConnectionStore};
use crate::errors::VerificationError;
use crate::verify::verify_jar;

#[derive(Debug)]
pub struct WebstartFile {
    url: String,
    main_class: String,
    args: Vec<String>,
    j2ses: Option<Vec<J2se>>,
    //jars: Vec<Jar>,
    tmp_dir: PathBuf,
    loaded_at: SystemTime
}

/// from jnlp -> resources -> j2se
#[derive(Debug)]
pub struct J2se {
    java_vm_args: Option<String>,
    version: String
}

pub struct WebStartCache {
    cache: Mutex<FxHashMap<String, Arc<WebstartFile>>>
}

impl WebStartCache {
    pub fn init() -> Self {
        let cache = Mutex::new(FxHashMap::default());
        WebStartCache{cache}
    }

    pub fn put(&mut self, wf: Arc<WebstartFile>) {
        self.cache.lock().unwrap().insert(wf.url.clone(), wf);
    }

    pub fn get(&self, url: &str) -> Option<Arc<WebstartFile>> {
        let cache = self.cache.lock().unwrap();
        let wf = cache.get(url);
        if let Some(wf) = wf {
            let now = SystemTime::now();
            let elapsed = now.duration_since(wf.loaded_at).expect("failed to calculate the duration");
            if elapsed.as_secs() < 120 {
                return Some(Arc::clone(wf));
            }
        }
        None
    }
}

impl WebstartFile {
    pub fn load(base_url: &str) -> Result<WebstartFile, Error> {
        let webstart = format!("{}/webstart.jnlp", base_url);
        let cb = ClientBuilder::default().danger_accept_invalid_certs(true);
        let client = cb.build()?;

        let r = client.get(&webstart).send()?;
        let data = r.text()?;
        //TODO VERY NOISY, is there a log level lower than debug?
        //println!("Got response from MC as: {:?}", data);
        let doc = roxmltree::Document::parse(&data)?;

        let root = doc.root();
        let main_class_node = get_node(&root, "application-desc").expect("Got something from MC that was not an application-desc node in a JNLP XML");
        let main_class = main_class_node.attribute("main-class").expect("missing main-class attribute").to_string();
        let args = get_client_args(&main_class_node);

        let resources_node = get_node(&root, "resources");

        let mut hasher = Sha256::new();
        hasher.update(&webstart);
        let hash = hasher.finalize();
        let hash = hex::encode(&hash);
        let tmp_dir = env::temp_dir().join(format!("ballista/{}", hash));
        println!("creating directory {:?}", tmp_dir);
        if tmp_dir.exists() {
            std::fs::remove_dir_all(&tmp_dir)?;
        }
        let dir_path = tmp_dir.as_path();
        std::fs::create_dir_all(dir_path)?;

        let mut j2ses = None;
        if let Some(resources_node) = resources_node {
            j2ses = get_j2ses(&resources_node);
            download_jars(&resources_node, &client, dir_path, base_url)?;
        }

        let loaded_at = SystemTime::now();
        let ws = WebstartFile{url: base_url.to_string(), main_class, tmp_dir, args, loaded_at, j2ses};

        Ok(ws)
    }

    pub fn run(&self, ce: Arc<ConnectionEntry>) -> Result<(), Error> {
        let itr = self.tmp_dir.read_dir()?;
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
            let classpath_separator = if cfg!(windows) {';'} else  {':' };

            //println!("{}", file_path);
            // MirthConnect's own jars contain some overridden classes
            // of the dependent libraries and hence must be loaded first
            // https://forums.mirthproject.io/forum/mirth-connect/support/15524-using-com-mirth-connect-client-core-client
            //TODO this should probably build the classpath objects as an ordered set, then do a .join(classpath_separator)
            if file_name.to_str().unwrap().starts_with("mirth") {
                classpath.push_str(file_path);
                classpath.push(classpath_separator);
            }
            else {
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
        }
        else {
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
        let itr = self.tmp_dir.read_dir().expect("failed to read the jar files directory");
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

fn download_jars(resources_node: &Node, client: &Client, dir_path: &Path, base_url: &str) -> Result<(), Error> {
    for n in resources_node.children() {
        let jar = n.has_tag_name("jar");
        let extension = n.has_tag_name("extension");

        if !jar && !extension {
            continue;
        }

        let href = n.attribute("href").unwrap();
        let url = format!("{}/{}", base_url, href);

        if jar {
            let file_name = get_file_name_from_path(href);
            let mut resp = client.get(url).send()?;
            let mut f = File::create(dir_path.join(file_name))?;
            resp.copy_to(&mut f)?;
        }
        else if extension {
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
                    let j2se = J2se{ java_vm_args, version: version.to_string()};
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

#[cfg(test)]
mod tests {
    use crate::webstart::WebstartFile;

    #[test]
    pub fn test_load() {
        let ws = WebstartFile::load("https://localhost:8443").unwrap();
        println!("{:?}", ws);
    }
}
