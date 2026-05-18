// Copyright (c) Kiran Ayyagari. All rights reserved.
// Copyright (c) Diridium Technologies Inc. All rights reserved.
// Licensed under the MPL-2.0 License. See LICENSE file in the project root.

use anyhow::Error;
use home::env::Env;
use home::env::OS_ENV;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::ops::Deref;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionEntry {
    pub address: String,
    #[serde(rename = "heapSize")]
    pub heap_size: String,
    pub icon: String,
    pub id: String,
    #[serde(rename = "javaHome")]
    pub java_home: String,
    #[serde(rename = "javaArgs")]
    pub java_args: Option<String>,
    pub name: String,
    pub username: Option<String>,
    pub password: Option<String>,
    #[serde(default = "get_default_group")]
    pub group: String,
    #[serde(default = "get_default_notes")]
    pub notes: String,
    #[serde(default = "get_default_donotcache")]
    pub donotcache: bool,
    #[serde(default, rename = "lastConnected")]
    pub last_connected: Option<i64>,
    #[serde(default, rename = "showConsole")]
    pub show_console: bool,
    #[serde(default, rename = "peerCertificate")]
    pub peer_certificate: Option<String>,
}

pub struct ConnectionStore {
    con_cache: Mutex<HashMap<String, Arc<ConnectionEntry>>>,
    con_location: PathBuf,
    pub cache_dir: PathBuf,
}

impl Default for ConnectionEntry {
    fn default() -> Self {
        let empty_str = String::from("");
        ConnectionEntry {
            address: empty_str.clone(),
            heap_size: String::from("512m"),
            icon: empty_str.clone(),
            id: Uuid::new_v4().to_string(),
            java_home: find_java_home(),
            java_args: Option::from(empty_str.clone()),
            name: empty_str.clone(),
            username: None,
            password: None,
            group: get_default_group(),
            notes: get_default_notes(),
            donotcache: get_default_donotcache(),
            last_connected: None,
            show_console: false,
            peer_certificate: None,
        }
    }
}

impl ConnectionStore {
    pub fn init(data_dir_path: PathBuf) -> Result<Self, Error> {
        let con_location = data_dir_path.join("ballista-data.json");
        let mut con_location_file = File::open(&con_location);
        if let Err(_e) = con_location_file {
            con_location_file = File::create(&con_location);
        }
        let con_location_file = con_location_file?;

        let mut cache = HashMap::new();
        let data: serde_json::Result<HashMap<String, ConnectionEntry>> =
            serde_json::from_reader(con_location_file);
        match data {
            Ok(data) => {
                for (id, ce) in data {
                    cache.insert(id, Arc::new(ce));
                }
            }
            Err(e) => {
                println!("{}", e);
            }
        }

        let cache_dir = data_dir_path.join("cache");
        if !cache_dir.exists() {
            fs::create_dir(&cache_dir)?;
        }

        Ok(ConnectionStore {
            con_location,
            con_cache: Mutex::new(cache),
            cache_dir
        })
    }

    pub fn to_json_array_string(&self) -> String {
        let cache = self.con_cache.lock().expect("connection cache lock poisoned");
        let mut sb = String::with_capacity(1024);
        let len = cache.len();
        sb.push('[');
        for (pos, ce) in cache.values().enumerate() {
            let c = serde_json::to_string(ce).unwrap_or_default();
            sb.push_str(c.as_str());
            if pos + 1 < len {
                sb.push(',');
            }
        }
        sb.push(']');

        sb
    }

    pub fn get(&self, id: &str) -> Option<Arc<ConnectionEntry>> {
        let cs = self.con_cache.lock().expect("connection cache lock poisoned");
        let val = cs.get(id);
        if let Some(val) = val {
            return Some(Arc::clone(val));
        }
        None
    }

    pub fn save(&self, mut ce: ConnectionEntry) -> Result<String, Error> {
        if ce.id.is_empty() {
            ce.id = uuid::Uuid::new_v4().to_string();
        }

        let mut jh = ce.java_home.trim().to_string();
        if jh.is_empty() {
            jh = find_java_home();
        }
        ce.java_home = jh;

        if let Some(ref username) = ce.username {
            let username = username.trim();
            if username.is_empty() {
                ce.username = None;
            }
        }

        if let Some(ref password) = ce.password {
            let password = password.trim();
            if password.is_empty() {
                ce.password = None;
            }
        }

        let data = serde_json::to_string(&ce)?;
        self.con_cache
            .lock()
            .expect("connection cache lock poisoned")
            .insert(ce.id.clone(), Arc::new(ce));
        self.write_connections_to_disk()?;
        Ok(data)
    }

    pub fn delete(&self, id: &str) -> Result<(), Error> {
        self.con_cache.lock().expect("connection cache lock poisoned").remove(id);
        self.write_connections_to_disk()?;
        Ok(())
    }

    pub fn import(&self, file_path: &str, overwrite: bool) -> Result<String, Error> {
        let f = File::open(file_path)?;
        let data: Vec<ConnectionEntry> = serde_json::from_reader(f)?;

        // Check for collisions with existing connections
        let cache = self.con_cache.lock().expect("connection cache lock poisoned");
        let duplicates: Vec<String> = data
            .iter()
            .filter(|ce| cache.contains_key(&ce.id))
            .map(|ce| ce.name.clone())
            .collect();
        drop(cache);

        if !duplicates.is_empty() && !overwrite {
            let result = serde_json::json!({
                "status": "duplicates",
                "names": duplicates,
                "total": data.len(),
            });
            return Ok(result.to_string());
        }

        let mut count = 0;
        let java_home = find_java_home();
        for mut ce in data {
            ce.java_home = java_home.clone();
            self.con_cache
                .lock()
                .expect("connection cache lock poisoned")
                .insert(ce.id.clone(), Arc::new(ce));
            count += 1;
        }

        self.write_connections_to_disk()?;
        let result = serde_json::json!({
            "status": "ok",
            "total": count,
        });
        Ok(result.to_string())
    }

    pub fn add_trusted_cert(&self, connection_id: &str, peer_certificate: &str) -> Result<(), Error> {
        let mut cache = self.con_cache.lock().expect("connection cache lock poisoned");
        let Some(entry) = cache.get(connection_id) else {
            return Err(Error::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("connection not found: {}", connection_id),
            )));
        };

        let mut updated = (**entry).clone();
        updated.peer_certificate = Some(peer_certificate.to_string());
        cache.insert(connection_id.to_string(), Arc::new(updated));
        drop(cache);

        self.write_connections_to_disk()
    }

    fn write_connections_to_disk(&self) -> Result<(), Error> {
        let c = self.con_cache.lock().expect("connection cache lock poisoned");
        let val = serde_json::to_string_pretty(c.deref())?;
        let mut f = OpenOptions::new()
            .append(false)
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.con_location)
            .map_err(|e| {
                println!("unable to open file for writing: {}", e);
                Error::new(e)
            })?;
        f.write_all(val.as_bytes())?;
        Ok(())
    }

    pub fn update_last_connected(&self, id: &str) -> Result<(), Error> {
        let mut cache = self.con_cache.lock().expect("connection cache lock poisoned");
        if let Some(entry) = cache.get(id) {
            let mut updated = (**entry).clone();
            updated.last_connected = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("system clock is before UNIX epoch")
                    .as_millis() as i64,
            );
            cache.insert(id.to_string(), Arc::new(updated));
        }
        drop(cache);
        self.write_connections_to_disk()?;
        Ok(())
    }

    pub fn get_all_groups(&self) -> Result<HashSet<String>, Error> {
        let connections = self.con_cache
            .lock()
            .expect("connection cache lock poisoned");

        let mut groups: HashSet<String> = HashSet::new();

        // Ensure default group
        groups.insert(get_default_group());

        let collected_groups: HashSet<String> = connections
            .values()
            .map(|connection_entry| connection_entry.group.clone())  // extract the property
            .collect();

        groups.extend(collected_groups);

        Ok(groups)
    }
}

pub fn find_java_home() -> String {
    let mut java_home = String::from("");
    if let Some(jh) = OS_ENV.var_os("JAVA_HOME") {
        if let Some(jh_str) = jh.to_str() {
            java_home = String::from(jh_str);
            println!("JAVA_HOME is set to {}", java_home);
        } else {
            println!("JAVA_HOME contains non-UTF-8 characters, ignoring");
        }
    }

    if java_home.is_empty() {
        let out = Command::new("/usr/libexec/java_home")
            .args(["-v", "1.8"])
            .output();
        if let Ok(out) = out {
            if out.status.success() {
                match String::from_utf8(out.stdout) {
                    Ok(jh) => {
                        println!("/usr/libexec/java_home -v 1.8 returned {}", jh);
                        java_home = jh;
                    }
                    Err(e) => {
                        println!("java_home output was not valid UTF-8: {}", e);
                    }
                }
            }
        }
    }
    java_home
}

fn get_default_group() -> String {
    String::from("Default")
}

fn get_default_notes() -> String {
    String::from("")
}

fn get_default_donotcache() -> bool {
    false
}
