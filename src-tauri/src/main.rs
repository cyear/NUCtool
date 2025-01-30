#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use std::{
    env,
    sync::{mpsc, Arc, Mutex},
    thread,
};
mod modules;
use modules::{
    config::{get_config_dir, load_fan_config, save_fan_config},
    fan::{fan_reset, get_fan_speeds, start_fan_control, stop_fan_control},
    permissions::privilege_escalation,
    setup,
    struct_set::{ChannelControlState, FanControlState},
    tdp::{get_tdp, set_tdp},
    wmi::{wmi_init, wmi_security, wmi_set},
};

fn main() {
    privilege_escalation();
    let (tx, rx) = mpsc::channel::<String>();
    let (tx1, _rx1) = mpsc::channel::<i64>();
    thread::spawn(move || {
        wmi_security();
        let (in_cls, svc, obj_path, method_name) = wmi_init();
        while let Ok(data) = rx.recv() {
            let out = wmi_set(&in_cls, &svc, &obj_path, &method_name, data.as_str());
            println!("{:?}", out);
        }
    });
    let channel_control_state = ChannelControlState {
        tx: Arc::new(Mutex::new(tx)),
    };
    let fan_control_state = FanControlState {
        is_running: Arc::new(Mutex::new(false)),
    };
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| setup::init(app))
        .manage(fan_control_state)
        .manage(channel_control_state)
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
