#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use std::{
    env,
    thread,
    sync::{Arc, Mutex}
};
use tauri_plugin_autostart::MacosLauncher;

#[cfg(windows)]
mod win_plug;
#[cfg(unix)]
mod linux_plug;
mod plug;
mod tests;

use plug::{
    setup,
    struct_set::FanControlState,
    config::{save_fan_config, load_fan_config},
    fan::{fan_reset, get_fan_speeds, start_fan_control, stop_fan_control},
    tdp::{get_tdp, set_tdp, set_rgb, get_rgb, set_rgb_color_y, set_rgb_color_n, get_rgb_color},
};

#[cfg(windows)]
use win_plug::{
    permissions::privilege_escalation,
    wmi::wmi_security,
};
#[cfg(unix)]
use linux_plug::{
    sysfs::sys_init,
};

fn main() {
    #[cfg(debug_assertions)]
    let devtools = tauri_plugin_devtools::init();
    let mut builder = tauri::Builder::default();
    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(devtools);
    }
    #[cfg(windows)]
    privilege_escalation();
    #[cfg(windows)]
    thread::spawn(move || {
        wmi_security();
        fan_reset();
    });
    #[cfg(unix)]
    sys_init();
    let fan_control_state = FanControlState {
        is_running: Arc::new(Mutex::new(false)),
    };
    builder
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
            set_rgb,
            get_rgb,
            set_rgb_color_y,
            set_rgb_color_n,
            get_rgb_color
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
