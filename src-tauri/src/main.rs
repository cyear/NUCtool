#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use std::{
    env,
    thread,
    sync::{
        Arc,
        Mutex,
    }
};
use tauri_plugin_autostart::MacosLauncher;

mod modules;
use modules::{
    config::{get_config_dir, load_fan_config, save_fan_config},
    fan::{fan_reset, get_fan_speeds, start_fan_control, stop_fan_control},
    permissions::privilege_escalation,
    setup,
    tdp::{get_tdp, set_tdp},
    wmi::wmi_security,
    struct_set::FanControlState,
};

fn main() {
    privilege_escalation();
    thread::spawn(move || {
        wmi_security();
        fan_reset();
    });
    let fan_control_state = FanControlState {
        is_running: Arc::new(Mutex::new(false)),
    };
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![]), ))
        .manage(fan_control_state)
        .setup(|app| setup::init(app))
        .invoke_handler(tauri::generate_handler![
            start_fan_control,
            stop_fan_control,
            save_fan_config,
            load_fan_config,
            get_fan_speeds,
            get_tdp,
            set_tdp,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .unwrap();
}
