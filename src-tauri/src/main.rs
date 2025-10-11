// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::PathBuf;
use std::process::exit;
use std::sync::Arc;

use serde_json::Number;
use tauri::State;

use crate::connection::{ConnectionEntry, ConnectionStore};
use crate::webstart::{WebStartCache, WebstartFile};

mod connection;
mod errors;
mod verify;
mod webstart;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tauri::command]
async fn get_ballista_info() -> String {
    let mut obj = serde_json::Map::new();
    obj.insert(
        "ballista_version".to_string(),
        serde_json::Value::String(String::from(APP_VERSION)),
    );
    return serde_json::to_string(&obj).unwrap();
}

#[tauri::command(rename_all = "snake_case")]
fn launch(id: &str, cs: State<ConnectionStore>, wc: State<WebStartCache>) -> String {
    let ce = cs.get(id);
    if let Some(ce) = ce {
        let mut ws = wc.get(&ce.address);
        if let None = ws {
            let tmp = WebstartFile::load(&ce.address, &cs.cache_dir, ce.donotcache);
            if let Err(e) = tmp {
                let msg = e.to_string();
                println!("{}", msg);
                return create_json_resp(-1, &msg);
            }

            ws = Some(Arc::new(tmp.unwrap()));
        }
        let ws = ws.unwrap();
        if ce.verify {
            let verification_status = ws.verify(cs.get_cert_store().as_ref());
            if let Err(e) = verification_status {
                let resp = e.to_json();
                println!("{}", resp);
                return resp;
            }
        }
        let r = ws.run(ce);
        if let Err(e) = r {
            let msg = e.to_string();
            println!("{}", msg);
            return create_json_resp(-1, &msg);
        }
    }

    String::from("{\"code\": 0}")
}

#[tauri::command]
fn get_default_connectionentry(cs: State<ConnectionStore>) -> Result<serde_json::Value, String> {
    let connection_entry = ConnectionEntry::default();
    Ok(serde_json::json!(connection_entry))
}

#[tauri::command]
fn get_all_groups(cs: State<ConnectionStore>) -> Result<serde_json::Value, String> {
    let groups = cs.get_all_groups().unwrap();
    Ok(serde_json::json!(groups))
}

#[tauri::command]
fn load_connections(cs: State<ConnectionStore>) -> String {
    cs.to_json_array_string()
}

#[tauri::command]
fn load_single_connection(cs: State<ConnectionStore>, connection_id: String) -> Result<serde_json::Value, String> {
    let connection_entry = cs.get(connection_id.as_str()).unwrap();
    Ok(serde_json::json!(connection_entry))
}

#[tauri::command]
fn save(ce: &str, cs: State<ConnectionStore>) -> String {
    let ce: serde_json::Result<ConnectionEntry> = serde_json::from_str(ce);
    //println!("received connection data {:?}", ce);
    let r = cs.save(ce.expect("failed to deserialize the given ConnectionEntry"));
    if let Err(e) = r {
        return e.to_string();
    }

    r.unwrap()
}

#[tauri::command]
fn delete(id: &str, cs: State<ConnectionStore>) -> String {
    let r = cs.delete(id);
    if let Err(e) = r {
        return e.to_string();
    }
    String::from("success")
}

#[tauri::command(rename_all = "snake_case")]
fn import(file_path: &str, cs: State<ConnectionStore>) -> String {
    let r = cs.import(file_path);
    if let Err(e) = r {
        let msg = e.to_string();
        println!("{}", msg);
        return msg;
    }

    r.unwrap()
}

#[tauri::command(rename_all = "snake_case")]
fn trust_cert(cert: &str, cs: State<ConnectionStore>) -> String {
    let r = cs.add_trusted_cert(cert);
    if let Err(e) = r {
        return e.to_string();
    }
    String::from("success")
}

fn main() {
    let env_fix = fix_path_env::fix_vars(&["JAVA_HOME", "PATH"]);
    if let Err(_e) = env_fix {
        println!("failed to read JAVA_HOME and PATH environment variables");
    }

    let home_directory = home::home_dir().expect("unable to find the path to home directory");
    // <= 0.2.0 migrate to a new app specific location
    let ballista_directory = home_directory.join(".ballista");
    let r = fs::create_dir(&ballista_directory);
    if let Ok(_) = r {
        move_file(home_directory.join("catapult-data.json"), ballista_directory.join("ballista-data.json"));
        move_file(
            home_directory.join("catapult-trusted-certs.json"),
            ballista_directory.join("ballista-trusted-certs.json"),
        );
    }

    let connection_store = ConnectionStore::init(ballista_directory);
    if let Err(e) = connection_store {
        println!("failed to initialize ConnectionStore: {}", e.to_string());
        exit(1);
    }

    let webcache = WebStartCache::init();
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .manage(connection_store.unwrap())
        .manage(webcache)
        .invoke_handler(tauri::generate_handler![
            launch,
            import,
            delete,
            save,
            get_default_connectionentry,
            get_all_groups,
            load_connections,
            load_single_connection,
            trust_cert,
            get_ballista_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn create_json_resp(code: i32, msg: &str) -> String {
    let mut obj = serde_json::Map::new();
    obj.insert(
        "code".to_string(),
        serde_json::Value::Number(Number::from(code)),
    );
    obj.insert(
        "msg".to_string(),
        serde_json::Value::String(String::from(msg)),
    );
    serde_json::to_string(&obj).unwrap()
}

fn move_file(old: PathBuf, new: PathBuf) {
    if old.exists() && !new.exists() {
        let r = fs::rename(&old, &new);
        if let Err(e) = r {
            println!(
                "failed to move the file from {:?} to {:?} : {}",
                old,
                new,
                e.to_string()
            );
        }
    }
}
