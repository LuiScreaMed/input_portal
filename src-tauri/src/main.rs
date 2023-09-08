// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    ffi::OsString,
    fs::{self, DirEntry},
    hash,
    io::Error,
    path::{Path, PathBuf},
    sync::Mutex,
    thread,
    time::{Duration, SystemTime},
};

use config::Config;
use file::{get_dir_entries, FileError};
use inputs::{start, InputInfo, InputMessage, InputSource};
use message::{Message, MessageData, MessageType};
use once_cell::sync::{Lazy, OnceCell};
use rdev::{listen, Event};
use serde_json::Value;
use tauri::{
    api::notification::Notification, App, AppHandle, CustomMenuItem, Manager, State, SystemTray,
    SystemTrayMenu, SystemTrayMenuItem, WindowEvent,
};
use tokio::{
    runtime::Runtime,
    sync::{
        broadcast::{self, Sender},
        mpsc::{self, UnboundedReceiver},
    },
};
use window_shadows::set_shadow;

mod config;
mod constants;
mod file;
mod inputs;
mod keys;
mod message;
mod server;

/// 设置
// static CONFIG: Lazy<Mutex<Config>> = Lazy::new(|| {
//     Mutex::new(Config {
//         preset: None,
//         enable: true,
//         key_down_transition_duration: 0,
//         key_up_transition_duration: 100,
//         mouse_move_enable: true,
//         mouse_move_radius_px: 50,
//         mouse_move_transition_duration: 100,
//     })
// });

static mut CONFIG: Lazy<Mutex<Config>> = Lazy::new(|| {
    Mutex::from(Config {
        preset: None,
        enable: true,
        key_down_transition_duration: 0,
        key_up_transition_duration: 100,
        mouse_move_enable: true,
        mouse_move_radius_px: 50,
        mouse_move_transition_duration: 100,
    })
});

static mut NOTIFIED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::from(false));

#[derive(Debug, Clone)]
struct Version(String);

#[tokio::main]
async fn main() {
    let version = Version(env!("CARGO_PKG_VERSION").to_string());
    // 初始化配置
    initialize_config();
    // 广播通道
    let (message_sender, _) = broadcast::channel(256);
    // 设置发送器
    let message_sender_config = message_sender.clone();
    // 输入发送器
    let message_sender_input = message_sender.clone();

    // 初始化端口
    let mut port = 61477;
    loop {
        if port_check::is_local_port_free(port) {
            break;
        } else {
            port = port.wrapping_add(1);
        }
    }

    // 服务器task
    let _server_task = tokio::task::spawn(server::run(message_sender, port));

    // 按键监听task
    let _input = tokio::task::spawn_blocking(move || {
        start(message_sender_input);
    });

    // 系统托盘图标
    let system_tray = initialize_system_tray();

    // tauri，启动!
    tauri::Builder::default()
        // 发送器，用于在设置改变后进行广播
        .manage(message_sender_config)
        .manage(port)
        .manage(version)
        .invoke_handler(tauri::generate_handler![
            get_presets,
            set_config,
            get_config,
            get_port,
            get_version,
            close_window,
            open_credit
        ])
        // 单一APP实例
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            app.get_window("main").unwrap().show().unwrap();
        }))
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            tauri::SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                "setting" => {
                    let window = app.get_window("main").unwrap();
                    window.show().unwrap();
                }
                _ => {}
            },
            tauri::SystemTrayEvent::LeftClick { .. } => {
                let window = app.get_window("main").unwrap();
                window.show().unwrap();
            }
            tauri::SystemTrayEvent::DoubleClick { .. } => {}
            tauri::SystemTrayEvent::RightClick { .. } => {}
            _ => todo!(),
        })
        .setup(move |app| {
            let window = app.get_window("main").unwrap();
            set_shadow(&window, true).expect("window shadow error: Unsupported platform!");

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
    // tokio::join!(server::start());
}

/// 初始化系统托盘图标
fn initialize_system_tray() -> SystemTray {
    let quit = CustomMenuItem::new("quit".to_string(), "退出");
    let setting = CustomMenuItem::new("setting".to_string(), "选项");
    let menu = SystemTrayMenu::new()
        .add_item(setting)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);
    SystemTray::new().with_menu(menu)
}

/// 初始化设置
fn initialize_config() -> () {
    let config = get_config_from_file();
    // CONFIG.lock().unwrap().set(config);
    unsafe { CONFIG.lock().unwrap().set(config) };
}

// 从设置文件读取设置
fn get_config_from_file() -> Config {
    // 初始化config
    let mut config = Config {
        preset: None,
        enable: true,
        key_down_transition_duration: 0,
        key_up_transition_duration: 100,
        mouse_move_enable: true,
        mouse_move_radius_px: 50,
        mouse_move_transition_duration: 100,
    };
    // 获取设置文件
    let path = Path::new("./config.json");
    if !path.exists() {
        let _ = fs::write(path, serde_json::to_string(&config).unwrap());
    } else {
        let file_string = fs::read_to_string(path);
        if let Err(_) = file_string {
            return config;
        }
        let file_string = file_string.unwrap();
        let config_from_file: Config = serde_json::from_str(&file_string).unwrap();
        config = config_from_file;
    }
    config
}

/// 前端关闭窗口
#[tauri::command]
fn close_window(handle: tauri::AppHandle) -> () {
    handle.get_window("main").unwrap().hide().unwrap();
    // 显示notification
    unsafe {
        if !*NOTIFIED.lock().unwrap() {
            *NOTIFIED.lock().unwrap() = true;
            Notification::new(&handle.config().tauri.bundle.identifier)
                .body("Input Portal 已最小化到系统托盘")
                .show()
                .unwrap();
        }
    }
}

/// 打开个人页面
#[tauri::command]
fn open_credit(site: String) -> () {
    let link = match site.as_str() {
        "bilibili" => Some("https://space.bilibili.com/3837681"),
        "github" => Some("https://github.com/LuiScreaMed"),
        _ => None,
    };
    match link {
        Some(url) => open::that(url).unwrap_or(()),
        None => (),
    };
}

/// 前端获取版本
#[tauri::command]
fn get_version(version: State<Version>) -> String {
    version.0.clone()
}

/// 前端获取端口
#[tauri::command]
fn get_port(port: State<u16>) -> u16 {
    *port
}

/// 前端获取设置
#[tauri::command]
fn get_config() -> Value {
    unsafe { serde_json::to_value(CONFIG.lock().unwrap().clone()).unwrap() }
    // serde_json::to_string(CONFIG.get().unwrap()).unwrap()
}

/// 前端修改设置
#[tauri::command]
fn set_config(new_config: Value, state: State<Sender<Message>>) -> bool {
    let config_res: Result<_, serde_json::Error> = serde_json::from_value(new_config);
    if let Err(error) = config_res {
        eprintln!("{:?}", error);
        return false;
    }
    let config = config_res.unwrap();
    if save_config(&config) {
        unsafe {
            CONFIG.lock().unwrap().set(config);
            let _ = state
                .send(Message {
                    r#type: MessageType::Config,
                    data: MessageData::ConfigMessage(CONFIG.lock().unwrap().clone()),
                })
                .unwrap_or(0);
        }
        return true;
    } else {
        return false;
    }
}

/// 保存设置
fn save_config(config: &Config) -> bool {
    let path = Path::new("./config.json");
    if let Err(error) = fs::write(path, serde_json::to_string(config).unwrap()) {
        eprintln!("{:?}", error);
        return false;
    };
    true
}

#[derive(Debug)]
enum PresetError {
    ReadFilesError,
    FileTypeError,
    NotDirError,
    NotPresetError,
}

/// 读取预设列表
#[tauri::command]
fn get_presets(handle: tauri::AppHandle) -> Vec<String> {
    // 初始化预设数组
    let mut preset_list: Vec<String> = Vec::new();
    // 获取路径
    let path_buf = handle.path_resolver().resolve_resource("presets");
    // 获取文件夹文件
    let read_dir = get_dir_entries(path_buf);
    if let Err(_) = read_dir {
        return preset_list;
    }
    // 遍历文件入口
    for entry_res in read_dir.unwrap() {
        let preset_res = get_single_preset(entry_res);
        if let Err(_) = preset_res {
            continue;
        }
        preset_list.push(preset_res.unwrap());
    }
    preset_list
}

/// 读取并判断文件夹是否为单个预设文件夹
fn get_single_preset(entry_res: Result<DirEntry, Error>) -> Result<String, PresetError> {
    if let Err(_) = entry_res {
        return Err(PresetError::ReadFilesError);
    }
    let entry = entry_res.unwrap();
    // 读取文件类型
    let file_type_res = entry.file_type();
    if let Err(_) = file_type_res {
        return Err(PresetError::FileTypeError);
    }
    // 是否为文件夹
    if !file_type_res.unwrap().is_dir() {
        return Err(PresetError::NotDirError);
    }
    // 读取文件夹
    let file_path = entry.path();
    let read_dir_res = get_dir_entries(Some(file_path));
    if let Err(_) = read_dir_res {
        return Err(PresetError::ReadFilesError);
    }
    // 判断文件
    let mut has_image = false;
    let mut has_config = false;
    for entry_res in read_dir_res.unwrap() {
        if let Err(_) = entry_res {
            continue;
        }
        // 判断文件名
        let file_name = get_file_name(entry_res.unwrap()).unwrap_or("".to_string());
        if file_name.eq("") {
            continue;
        } else if file_name.eq(constants::PRESET_IMAGE_FILE_NAME) {
            has_image = true;
        } else if file_name.eq(constants::PRESET_CONFIG_FILE_NAME) {
            has_config = true;
        };
        // 图片和配置文件都存在则
        if has_image && has_config {
            return if let Some(str) = entry.file_name().to_str() {
                Ok(str.to_string())
            } else {
                Err(PresetError::ReadFilesError)
            };
        }
    }
    Err(PresetError::NotPresetError)
}

/// 获取文件名，如果不是文件或者其他则返回错误
fn get_file_name(entry: DirEntry) -> Result<String, FileError> {
    let file_type_res = entry.file_type();
    if let Err(_) = file_type_res {
        return Err(FileError::FileTypeError);
    }
    if file_type_res.unwrap().is_file() {
        match entry.file_name().to_str() {
            Some(str) => Ok(str.to_string()),
            None => Err(FileError::ReadDirError),
        }
    } else {
        return Err(FileError::NotFileError);
    }
}
