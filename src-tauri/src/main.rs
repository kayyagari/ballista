// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::{Command, exit};
use anyhow::Error;
use home::env::Env;
use tauri::State;

use crate::con::{ConnectionEntry, ConnectionStore};
use crate::webstart::WebstartFile;

mod webstart;
mod con;

#[tauri::command(rename_all = "snake_case")]
fn launch(id: &str, cs: State<ConnectionStore>) -> String {
    let ce = cs.get(id);
    if let Some(ce) = ce {
        let ws = WebstartFile::load(&ce.address);
        if let Err(e) = ws {
            return e.to_string();
        }

        let r = ws.unwrap().run(ce);
        if let Err(e) = r {
            return  e.to_string();
        }
    }

    String::from("success")
}

#[tauri::command]
fn load_connections(cs: State<ConnectionStore>) -> String {
    cs.to_json_array_string()
}

#[tauri::command]
fn save(ce: &str, cs: State<ConnectionStore>) -> String {
    let ce : serde_json::Result<ConnectionEntry> = serde_json::from_str(ce);
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

fn main() {
    fix_path_env::fix();
    let hd = home::home_dir().expect("unable to find the path to home directory");
    let hd = hd.join("catapult-data.json");

    let cs = ConnectionStore::init(hd);
    if let Err(e) = cs {
        println!("failed to initialize ConnectionStore: {}", e.to_string());
        exit(1);
    }

    tauri::Builder::default()
        .manage(cs.unwrap())
        .invoke_handler(tauri::generate_handler![launch, import, delete, save, load_connections])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
