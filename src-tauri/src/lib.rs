mod win;
mod acpi;
mod config;
mod fan_control;
use win::privilege_escalation;
use acpi::{UniwillAcpiEc, UniwillWmiEc};
use config::FanData;
use fan_control::{FanControlState, calculate_speed};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{
    AppHandle, Emitter, Manager, State, include_image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{
        MouseButton,
        MouseButtonState,
        TrayIconBuilder,
        TrayIconEvent,
    },
};

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
        let ec = match UniwillAcpiEc::open() {
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

#[tauri::command]
async fn load_fan_config() -> Result<FanData, String> {
    config::load()
}

#[tauri::command]
async fn save_fan_config(fan_data: FanData) -> Result<(), String> {
    config::save(&fan_data)
}

#[tauri::command]
async fn start_fan_control(
    fan_data: FanData,
    state: tauri::State<'_, FanControlState>,
) -> Result<(), String> {

    // 防止重复启动
    if state.running.swap(true, Ordering::SeqCst) {
        return Err("风扇控制已经在运行".into());
    }

    let running = Arc::clone(&state.running);

    let handle = thread::spawn(move || {

        println!("================================");
        println!("风扇自动控制线程启动");
        println!("================================");
        
        // EC init
        let ec = match UniwillAcpiEc::open() {
            Ok(ec) => ec,
            Err(e) => {
                eprintln!("打开 ACPIDriver 失败: {}", e);
                running.store(false, Ordering::SeqCst);
                return;
            }
        };

        // WMI init
        let wmi = match UniwillWmiEc::new() {
            Ok(wmi) => wmi,
            Err(e) => {
                eprintln!("打开 WMI 失败: {}", e);
                running.store(false, Ordering::SeqCst);
                return;
            }
        };
        // WMI
        let get_set = |data: u64| {
            match wmi.get_set(data) {
                Ok(ret) => {
                    // println!("Return = 0x{:08X}", ret);
                    // println!("EC Data = 0x{:02X}", ret & 0xFF);
                    // Debug
                }
                Err(e) => {
                    eprintln!("GetSetULong failed: {e}");
                }
            }
        };
        if let Err(e) =
            ec.fan_write_mode(acpi::uniwillacpi::FanModeByte::FanBoostMode)
        {
            eprintln!("切换FAN手动模式失败: {}", e);
        }
        while running.load(Ordering::SeqCst) {

            // ========================================
            // 1. 读取温度
            // ========================================
            let cpu_temp = ec.cpu_temperature().unwrap_or(0);
            let gpu_temp = ec.gpu_temperature().unwrap_or(0);

            // ========================================
            // 2. 根据曲线计算风扇转速
            // ========================================
            // &fan_data.left_fan M
            // &fan_data.right_fan S

            // CPU 风扇 主 => 右
            let right_speed =
                calculate_speed(&fan_data.left_fan, cpu_temp);
            
            // GPU 风扇 分 => 左
            let left_speed =
                calculate_speed(&fan_data.right_fan, gpu_temp);
            
            // ========================================
            // 3. 模式检查
            // ========================================

            let fanm = ec.fan_read_mode().expect("读取MODE失败");    
            if fanm == acpi::uniwillacpi::FanModeByte::AutoMode as u8 {
                println!("当前是 Auto Mode");
                if let Err(e) =
                    ec.fan_write_mode(acpi::uniwillacpi::FanModeByte::FanBoostMode)
                {
                    eprintln!("切换FAN手动模式失败: {}", e);
                } else {
                    println!("切换FAN手动模式成功");
                }
            }

            // ========================================
            // 4. 写入 WMI
            // ========================================

            // 左风扇 分
            get_set(((left_speed as u64 * 2) << 16) | 0x0000000000001809);

            // 右风扇 主
            get_set(((right_speed as u64 * 2) << 16) | 0x0000000000001804);
            
            // ========================================
            // 5. 输出调试信息
            // ========================================

            println!(
                "CPU {:.1}°C → Fan1 {}%  GPU {:.1}°C → Fan2 {}%  M: {}",
                cpu_temp,
                left_speed,
                gpu_temp,
                right_speed,
                fanm
            );

            // ========================================
            // 6. 控制周期
            // ========================================

            thread::sleep(Duration::from_millis(1500));
        }

        println!("风扇自动控制线程退出");
    });


    *state.thread.lock().unwrap() = Some(handle);

    Ok(())
}

#[tauri::command]
async fn stop_fan_control(
    state: tauri::State<'_, FanControlState>,
) -> Result<(), String> {
    stop_fan_control_inner(&state)
}

fn stop_fan_control_inner(
    state: &FanControlState,
) -> Result<(), String> {
    // 先停止控制线程
    if !state.running.swap(false, Ordering::SeqCst) {
        return Ok(());
    }

    println!("正在停止风扇控制...");

    // 等待控制线程退出
    if let Some(handle) = state.thread.lock().unwrap().take() {
        let _ = handle.join();
    }

    // EC init
    let ec = match UniwillAcpiEc::open() {
        Ok(ec) => ec,
        Err(e) => {
            eprintln!("打开 ACPIDriver 失败: {}", e);
            return Err(format!("打开 ACPIDriver 失败: {}", e));
        }
    };

    // 恢复自动风扇模式
    if let Err(e) =
        ec.fan_write_mode(acpi::uniwillacpi::FanModeByte::AutoMode)
    {
        eprintln!("恢复自动风扇模式失败: {}", e);
        return Err(format!("恢复自动风扇模式失败: {}", e));
    }

    println!("风扇控制已停止");

    Ok(())
}

pub fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // =========================
    // 托盘菜单
    // =========================

    let show = MenuItemBuilder::with_id("show", "显示窗口")
        .build(app)?;

    let quit = MenuItemBuilder::with_id("quit", "退出")
        .build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&show)
        .separator()
        .item(&quit)
        .build()?;

    const TRAY_ICON: tauri::image::Image<'_> = include_image!("icons/32x32.png");

    // =========================
    // 创建托盘
    // =========================

    TrayIconBuilder::new()
        .icon(TRAY_ICON)
        .menu(&menu)
        .tooltip("NUCtool")

        // 托盘菜单
        .on_menu_event(|app, event| {
            match event.id().as_ref() {
                "show" => {
                    if let Some(window) =
                        app.get_webview_window("main")
                    {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }

                "quit" => {
                    app.exit(0);
                }

                _ => {}
            }
        })

        // 托盘鼠标事件
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button,
                button_state,
                ..
            } = event
            {
                if button == MouseButton::Left
                    && button_state == MouseButtonState::Up
                {
                    if let Some(window) =
                        tray.app_handle().get_webview_window("main")
                    {
                        let visible =
                            window.is_visible().unwrap_or(false);

                        if visible {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            }
        })
        .build(app)?;

    // =========================
    // 窗口关闭 → 隐藏到托盘
    // =========================

    if let Some(window) = app.get_webview_window("main") {
        let window_for_event = window.clone();

        window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // 阻止真正关闭
                api.prevent_close();

                // 隐藏窗口
                let _ = window_for_event.hide();
            }
        });
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    
    // 管理员权限！！！
    privilege_escalation();
    
    let app = tauri::Builder::default()
        .manage(AppState {
            running: Arc::new(AtomicBool::new(false)),
        })
        .manage(FanControlState::new())
        .invoke_handler(tauri::generate_handler![
            start_sensor_loop,
            load_fan_config,
            save_fan_config,
            start_fan_control,
            stop_fan_control
        ])
        .setup(setup)
        // .setup(|app| {
        //     setup(app)?;

        //     #[cfg(target_os = "windows")]
        //     {
        //         let window = app
        //             .get_webview_window("main")
        //             .ok_or("找不到 main 窗口")?;

        //         let state = app
        //             .try_state::<FanControlState>()
        //             .ok_or("找不到 FanControlState")?;

        //         windows_shutdown::install(
        //             &window,
        //             state.inner(),
        //         )?;
        //     }

        //     Ok(())
        // })        
        .build(tauri::generate_context!())
        .expect("error while building tauri application");
    
    app.run(|app_handle: &tauri::AppHandle, event: tauri::RunEvent| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                println!("程序正在退出...");
                if let Some(state) =
                    app_handle.try_state::<FanControlState>()
                {
                    if let Err(e) =
                        stop_fan_control_inner(state.inner())
                    {
                        eprintln!("退出时停止风扇控制失败: {}", e);
                    }
                }
            }
        })
}

// #[cfg_attr(mobile, tauri::mobile_entry_point)]
// pub fn run() {
//     tauri::Builder::default()
//         .plugin(tauri_plugin_opener::init())
//         .invoke_handler(tauri::generate_handler![greet])
//         .run(tauri::generate_context!())
//         .expect("error while running tauri application");
// }
