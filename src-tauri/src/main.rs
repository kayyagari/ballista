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
fn launch(id: &str, cs: State<ConnectionStore>, wc: State<WebStartCache>, ) -> String {
    let ce = cs.get(id);
    if let Some(ce) = ce {
        let mut ws = wc.get(&ce.address);
        if let None = ws {
            let tmp = WebstartFile::load(&ce.address, &cs.cache_dir, ce.donotcache, &cs.log_dir);
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
fn load_connections(cs: State<ConnectionStore>) -> String {
    cs.to_json_array_string()
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

    let hd = home::home_dir().expect("unable to find the path to home directory");
    // <= 0.2.0 migrate to a new app specific location
    let bd = hd.join(".ballista");
    let r = fs::create_dir(&bd);
    if let Ok(_) = r {
        move_file(hd.join("catapult-data.json"), bd.join("ballista-data.json"));
        move_file(
            hd.join("catapult-trusted-certs.json"),
            bd.join("ballista-trusted-certs.json"),
        );
    }

    let cs = ConnectionStore::init(bd);
    if let Err(e) = cs {
        println!("failed to initialize ConnectionStore: {}", e.to_string());
        exit(1);
    }

    let wc = WebStartCache::init();
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
        .manage(cs.unwrap())
        .manage(wc)
        .invoke_handler(tauri::generate_handler![
            launch,
            import,
            delete,
            save,
            load_connections,
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
