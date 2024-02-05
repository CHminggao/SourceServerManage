// Prevents additional console window on Windows in release, DO NOT REMOVE!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::process::Command;

const FILE_PATH: &str = ".config/serverinfo.json";
const CONFIG_PATH: &str = ".config/config.json";
fn main() {
    hasfile();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            server_info,
            server_all,
            start_game,
            write_config_file,
            read_from_config_file,
            read_server,
            write_server
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Serialize, Deserialize, Debug)]
struct ServerInfo {
    name: String,
    map: String,
    game: String,
    players: u8,
    max_players: u8,
    ip: String,
    port: String,
    incount: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ServerConfig {
    ip: String,
    port: u32,
    incount: u32,
}

#[tauri::command]
fn start_game(server: String) {
    match Command::new("cmd")
        .arg("/C")
        .arg("start")
        .arg(server)
        .spawn()
    {
        Ok(_) => println!("Command executed successfully."),
        Err(err) => eprintln!("Error executing command: {}", err),
    }
}

#[tauri::command]
fn server_all() -> String {
    let server_config: Vec<ServerConfig> = read_from_server_file().unwrap();
    let mut server_info: Vec<ServerInfo> = Vec::new();
    for value in server_config.iter() {
        let some_value: ServerConfig = value.clone();

        let client: a2s::A2SClient = a2s::A2SClient::new().unwrap();
        let result: Result<a2s::info::Info, a2s::errors::Error> =
            client.info(format!("{}:{}", &some_value.ip, &some_value.port));
        match result {
            Ok(info) => {
                server_info.push(ServerInfo {
                    name: info.name,
                    map: info.map,
                    game: info.game,
                    players: info.players,
                    max_players: info.max_players,
                    ip: some_value.ip.clone(),
                    port: some_value.port.to_string(),
                    incount: some_value.incount.to_string(),
                });
            }
            Err(_) => {}
        }
    }
    return serde_json::to_string(&server_info).unwrap();
}

#[tauri::command]
fn server_info(ip: String, port: String) -> String {
    let client: a2s::A2SClient = a2s::A2SClient::new().unwrap();
    let result: Result<a2s::info::Info, a2s::errors::Error> = client.info(ip.clone() + ":" + &port);
    match result {
        Ok(info) => {
            let server: ServerInfo = ServerInfo {
                name: info.name,
                map: info.map,
                game: info.game,
                players: info.players,
                max_players: info.max_players,
                ip: ip,
                port: port,
                incount: "0".to_string(),
            };
            return serde_json::to_string(&server).unwrap();
        }
        Err(_) => {
            return "".to_string();
        }
    }
}

#[tauri::command]
fn read_from_config_file() -> String {
    let mut content: String = String::new();
    // 尝试以只读方式打开文件
    match File::open(CONFIG_PATH) {
        Ok(mut f) => {
            f.read_to_string(&mut content).unwrap();
        }
        Err(_) => {
            // 文件不存在，创建文件
            let created_file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(CONFIG_PATH);

            match created_file {
                Ok(mut a) => a.write_all(b"[]").unwrap(),
                Err(_) => {}
            }
        }
    };
    return content;
}

#[tauri::command]
fn write_config_file(json_string: String) -> String {
    let mut content: String = "".to_string();
    match File::create(CONFIG_PATH) {
        Ok(mut f) => match f.write_all(json_string.as_bytes()) {
            Ok(_) => {}
            Err(_) => {
                content ="写入失败".to_string();
            }
        },
        Err(_) => {
            content ="写入失败".to_string();
        }
    }
    return content;
}

#[tauri::command]
fn read_server()-> String{
    // 尝试以只读方式打开文件
    let mut file = match File::open(FILE_PATH) {
        Ok(f) => f,
        Err(_) => {
            // 文件不存在，创建文件
            let created_file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(FILE_PATH).unwrap();
            created_file
        }
    };

    // 读取文件内容
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    return content;
}

#[tauri::command]
fn write_server(json_string: String) -> String {
    let mut content: String = "".to_string();
    match File::create(FILE_PATH) {
        Ok(mut f) => match f.write_all(json_string.as_bytes()) {
            Ok(_) => {}
            Err(_) => {
                content ="写入失败".to_string();
            }
        },
        Err(_) => {
            content ="写入失败".to_string();
        }
    }
    return content;
}

fn read_from_server_file() -> io::Result<Vec<ServerConfig>> {
    // 尝试以只读方式打开文件
    let mut file = match File::open(FILE_PATH) {
        Ok(f) => f,
        Err(_) => {
            // 文件不存在，创建文件
            let created_file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(FILE_PATH).unwrap();
            created_file
        }
    };

    // 读取文件内容
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    if content.is_empty() {
        return Ok(Vec::new());
    }
    let person: Vec<ServerConfig> = serde_json::from_str(&content)?;
    Ok(person)
}

fn hasfile(){
    // 确保路径存在
    if let Some(parent_dir) = std::path::Path::new(FILE_PATH).parent() {
        fs::create_dir_all(parent_dir).unwrap();
    }
    // 使用 Path 类型来表示文件路径
    let file_path = std::path::Path::new(FILE_PATH);

    // 使用 exists 方法检查文件是否存在
    if !file_path.exists() {
        // 不存在则创建文件
        match File::create(file_path) {
            Ok(mut _file) => {
                // 在这里可以写入初始内容到文件，例如:
                _file.write_all(b"[]").expect("Failed to write to file.");
                println!("File created successfully.");
            }
            Err(err) => eprintln!("Error creating file: {}", err),
        }
    } else {
        println!("File already exists.");
    }
}