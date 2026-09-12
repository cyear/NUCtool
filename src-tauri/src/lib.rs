mod acpi;
use acpi::UniwillEc;

use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Clone, Serialize)]
struct SensorData {
    cpu_temp: u8,
    gpu_temp: u8,
    fan1_rpm: u16,
    fan2_rpm: u16,
}

struct AppState {
    running: Arc<AtomicBool>,
}

#[tauri::command]
fn start_sensor_loop(app: AppHandle, state: State<AppState>) {
    // 防止重复启动
    if state.running.swap(true, Ordering::SeqCst) {
        return;
    }

    let running = state.running.clone();
    let handle = app.clone();

    thread::spawn(move || {
        let ec = match UniwillEc::open() {
            Ok(ec) => ec,
            Err(e) => {
                eprintln!("打开 ACPIDriver 失败: {}", e);
                running.store(false, Ordering::SeqCst);
                return;
            }
        };

        while running.load(Ordering::SeqCst) {
            let data = SensorData {
                cpu_temp: ec.cpu_temperature().unwrap_or(0),
                gpu_temp: ec.gpu_temperature().unwrap_or(0),
                fan1_rpm: ec.fan1_rpm().unwrap_or(0),
                fan2_rpm: ec.fan2_rpm().unwrap_or(0),
            };

            // 推送给前端
            let _ = handle.emit("sensor-update", data);

            thread::sleep(Duration::from_millis(3000));
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            running: Arc::new(AtomicBool::new(false)),
        })
        .invoke_handler(tauri::generate_handler![
            start_sensor_loop,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// #[cfg_attr(mobile, tauri::mobile_entry_point)]
// pub fn run() {
//     tauri::Builder::default()
//         .plugin(tauri_plugin_opener::init())
//         .invoke_handler(tauri::generate_handler![greet])
//         .run(tauri::generate_context!())
//         .expect("error while running tauri application");
// }
