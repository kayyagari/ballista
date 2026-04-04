// Copyright (c) Kiran Ayyagari. All rights reserved.
// Copyright (c) Diridium Technologies Inc. All rights reserved.
// Licensed under the MPL-2.0 License. See LICENSE file in the project root.

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::PathBuf;
use std::process::exit;
use std::sync::Arc;

use serde_json::Number;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};

use crate::connection::{ConnectionEntry, ConnectionStore};
use crate::errors::{LaunchError, PeerDetails};
use crate::pinned_downloader::PinnedDownloadError;
use crate::webstart::{WebStartCache, WebstartFile};

mod connection;
mod errors;
mod pinned_downloader;
mod webstart;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tauri::command]
async fn get_launcher_info() -> String {
    let mut obj = serde_json::Map::new();
    obj.insert(
        "launcher_version".to_string(),
        serde_json::Value::String(String::from(APP_VERSION)),
    );
    return serde_json::to_string(&obj).unwrap_or_default();
}

#[tauri::command(rename_all = "snake_case")]
async fn launch(id: String, on_progress: Channel<serde_json::Value>, app: AppHandle, cs: State<'_, ConnectionStore>, wc: State<'_, WebStartCache>) -> Result<String, String> {
    let ce = cs.get(&id)
        .ok_or_else(|| format!("connection not found: {}", id))?;
    let cache_dir = cs.cache_dir.clone();
    let address = ce.address.clone();
    let conn_id = ce.id.clone();
    let conn_name = ce.name.clone();
    let donotcache = ce.donotcache;
    let expected_certificate = ce
        .peer_certificate
        .as_deref()
        .and_then(|peer_certificate| openssl::base64::decode_block(peer_certificate).ok());

    let mut ws = wc.get(&address);
    if ws.is_none() {
        let tmp = tauri::async_runtime::spawn_blocking({
            let on_progress = on_progress.clone();
            let address = address.clone();
            let cache_dir = cache_dir.clone();
            let expected_certificate = expected_certificate.clone();
            move || WebstartFile::load(&address, &cache_dir, donotcache, &conn_id, &conn_name, expected_certificate.as_deref(), &on_progress)
        }).await.map_err(|e| e.to_string())?;

        match tmp {
            Err(e) => {
                if let Some(download_error) = e.downcast_ref::<PinnedDownloadError>() {
                    let response = create_pinned_download_response(download_error, expected_certificate.as_deref());
                    println!("{}", response);
                    return Ok(response);
                }

                let msg = e.to_string();
                println!("{}", msg);
                return Ok(create_json_resp(-1, &msg));
            }
            Ok(wf) => {
                ws = Some(Arc::new(wf));
            }
        }
    }
    let ws = ws.expect("WebstartFile should be loaded at this point");
    let _ = on_progress.send(serde_json::json!({"message": "Launching administrator..."}));
    let console_jar = if ce.show_console {
        Some(app.path().resource_dir()
            .map_err(|e| e.to_string())?
            .join("lib")
            .join("java-console.jar"))
    } else {
        None
    };
    let r = ws.run(ce, console_jar);
    if let Err(e) = r {
        let msg = e.to_string();
        println!("{}", msg);
        return Ok(create_json_resp(-1, &msg));
    }

    let _ = cs.update_last_connected(&id);
    Ok(String::from("{\"code\": 0}"))
}

fn create_pinned_download_response(error: &PinnedDownloadError, expected_certificate: Option<&[u8]>) -> String {
    match error {
        PinnedDownloadError::PeerCertificate { peer_cert_der } => match PeerDetails::from_der(peer_cert_der) {
            Ok(peer) => {
                if let Some(expected_certificate) = expected_certificate {
                    let expected_fingerprint = PeerDetails::from_der(expected_certificate)
                        .map(|expected_peer| expected_peer.sha256sum)
                        .unwrap_or_else(|_| "unknown".to_string());
                    LaunchError::FingerprintMismatch {
                        peer,
                        expected_fingerprint,
                    }
                    .to_json()
                } else {
                    LaunchError::UntrustedPeer { peer }.to_json()
                }
            }
            Err(parse_error) => create_json_resp(-1, &format!("failed to parse peer certificate: {}", parse_error)),
        },
        PinnedDownloadError::Other(message) => create_json_resp(-1, message),
    }
}

#[tauri::command]
fn get_default_connectionentry(_cs: State<ConnectionStore>) -> Result<serde_json::Value, String> {
    let connection_entry = ConnectionEntry::default();
    Ok(serde_json::json!(connection_entry))
}

#[tauri::command]
fn get_all_groups(cs: State<ConnectionStore>) -> Result<serde_json::Value, String> {
    let groups = cs.get_all_groups().map_err(|e| e.to_string())?;
    Ok(serde_json::json!(groups))
}

#[tauri::command]
fn load_connections(cs: State<ConnectionStore>) -> String {
    cs.to_json_array_string()
}

#[tauri::command]
fn load_single_connection(cs: State<ConnectionStore>, connection_id: String) -> Result<serde_json::Value, String> {
    let connection_entry = cs.get(connection_id.as_str())
        .ok_or_else(|| format!("connection not found: {}", connection_id))?;
    Ok(serde_json::json!(connection_entry))
}

#[tauri::command]
fn save(ce: &str, cs: State<ConnectionStore>) -> Result<String, String> {
    let ce: ConnectionEntry = serde_json::from_str(ce)
        .map_err(|e| format!("failed to deserialize ConnectionEntry: {}", e))?;
    cs.save(ce).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete(id: &str, cs: State<ConnectionStore>) -> Result<String, String> {
    cs.delete(id).map_err(|e| e.to_string())?;
    Ok(String::from("success"))
}

#[tauri::command(rename_all = "snake_case")]
fn import(file_path: &str, overwrite: bool, cs: State<ConnectionStore>) -> Result<String, String> {
    cs.import(file_path, overwrite).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
fn trust_cert(connection_id: &str, peer_certificate: &str, cs: State<ConnectionStore>) -> Result<String, String> {
    cs.add_trusted_cert(connection_id, peer_certificate).map_err(|e| e.to_string())?;
    Ok(String::from("success"))
}

fn main() {
    let env_fix = fix_path_env::fix_vars(&["JAVA_HOME", "PATH"]);
    if let Err(_e) = env_fix {
        println!("failed to read JAVA_HOME and PATH environment variables");
    }

    let home_directory = home::home_dir().expect("unable to find the path to home directory");
    // <= 0.2.0 migrate from loose files to .ballista directory
    let legacy_ballista_dir = home_directory.join(".ballista");
    let r = fs::create_dir(&legacy_ballista_dir);
    if let Ok(_) = r {
        move_file(home_directory.join("catapult-data.json"), legacy_ballista_dir.join("ballista-data.json"));
    }

    // >= 2.1.0 migrate from .ballista to .launcher
    let launcher_directory = home_directory.join(".ballista");
    if let Err(e) = fs::create_dir(&launcher_directory) {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            println!("failed to create .ballista directory: {}", e);
            exit(1);
        }
    }

    let connection_store = ConnectionStore::init(launcher_directory);
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
        .manage(connection_store.expect("ConnectionStore init was checked above"))
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
            get_launcher_info
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
    serde_json::to_string(&obj).unwrap_or_default()
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
