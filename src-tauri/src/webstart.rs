use std::fs::File;
use std::os;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::Error;
use reqwest::blocking::{ClientBuilder, Client};
use reqwest::Url;
use roxmltree::{Document, Node};
use sha2::{Sha256, Digest};

#[derive(Debug)]
pub struct WebstartFile {
    main_class: String,
    args: Vec<String>,
    //jars: Vec<Jar>,
    tmp_dir: PathBuf
}

pub struct Jar {
    url: String,
    hash: String
}

impl WebstartFile {
    pub fn load(base_url: &str) -> Result<WebstartFile, Error> {
        let webstart = format!("{}/webstart.jnlp", base_url);
        let cb = ClientBuilder::default().danger_accept_invalid_certs(true);
        let client = cb.build()?;

        let r = client.get(&webstart).send()?;
        let data = r.text()?;
        let doc = roxmltree::Document::parse(&data)?;

        let root = doc.root();
        let main_class_node = get_node(&root, "application-desc").expect("application-desc node");
        let main_class = main_class_node.attribute("main-class").expect("missing main-class attribute").to_string();
        let args = get_client_args(&main_class_node);

        let resources_node = get_node(&root, "resources");

        let mut hasher = Sha256::new();
        hasher.update(&webstart);
        let hash = hasher.finalize();
        let hash = hex::encode(&hash);
        let tmp_dir = PathBuf::from(format!("/tmp/catapult/{}", hash));
        if tmp_dir.exists() {
            std::fs::remove_dir_all(&tmp_dir)?;
        }
        let dir_path = tmp_dir.as_path();
        std::fs::create_dir_all(dir_path)?;

        if let Some(resources_node) = resources_node {
            download_jars(&resources_node, &client, dir_path, base_url)?;
        }

        let ws = WebstartFile{main_class, tmp_dir, args};

        Ok(ws)
    }

    pub fn run(&self, java_home: &str, username: &str, password: &str) -> Result<(), Error> {
        let itr = self.tmp_dir.read_dir()?;
        let mut classpath = String::with_capacity(1024);
        let mut rhino_classpath = String::with_capacity(100);
        for e in itr {
            let e = e?;
            let file_path = e.path();
            let file_path = file_path.as_os_str();
            let file_path = file_path.to_str().unwrap();
            //println!("{}", file_path);
            // https://forums.mirthproject.io/forum/mirth-connect/support/15524-using-com-mirth-connect-client-core-client
            if file_path.to_lowercase().contains("rhino") {
                rhino_classpath.push_str(file_path);
                rhino_classpath.push(':');
            }
            else {
                classpath.push_str(file_path);
                classpath.push(':');
            }
        }

        classpath.push_str(&rhino_classpath);
        //println!("class path: {}", classpath);
        let mut cmd;
        let java_home = java_home.trim();
        if java_home.is_empty() {
            cmd = Command::new("java")
        }
        else {
            cmd = Command::new(format!("{}/bin/java", java_home));
        }

        println!("using java from: {:?}", cmd.get_program().to_str());

        cmd.arg("-Xms1g")
        .arg("-cp")
        .arg(classpath)
        .arg(&self.main_class)
        .args(&self.args);

        let username = username.trim();
        if !username.is_empty() {
            cmd.arg(username);
            if !password.is_empty() {
                cmd.arg(password);
            }
        }

        cmd.spawn()?;
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

        ws.run("", "admin", "admin").unwrap();
    }
}