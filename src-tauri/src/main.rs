use std::env;
use std::sync::{Arc, Mutex};
use tauri_plugin_autostart::MacosLauncher;

#[cfg(windows)]
mod win_plug;
#[cfg(unix)]
mod linux_plug;
mod plug;
use plug::{
    setup,
    config::{save_fan_config, load_fan_config},
    struct_set::FanControlState
};

#[cfg(windows)]
use win_plug::{
    fan::{fan_reset, get_fan_speeds, start_fan_control, stop_fan_control},
    permissions::privilege_escalation,
    tdp::{get_tdp, set_tdp},
    wmi::wmi_security,
};
#[cfg(unix)]
use linux_plug::{
    sysfs::{get_tdp, set_tdp},
    fan::{start_fan_control, stop_fan_control, get_fan_speeds}
};

fn main() {
    // This should be called as early in the execution of the app as possible
    #[cfg(debug_assertions)] // only enable instrumentation in development builds
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
