mod win;
mod acpi;
mod config;
mod fan_control;
use win::privilege_escalation;
use acpi::{UniwillAcpiEc, UniwillWmiEc, UniwillWcfEc, FnKeyhook};
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
use windows::{
    Win32::{
        Foundation::{
            HINSTANCE,
        },
        UI::WindowsAndMessaging::{
            DispatchMessageW,
            GetMessageW,
            SetWindowsHookExW,
            UnhookWindowsHookEx,
            MSG,
            WH_KEYBOARD_LL,
        },
    },
};

#[derive(Clone, Serialize)]
struct SensorData {
    cpu_temp: u8,
    gpu_temp: u8,
    fan1_rpm: u16,
    fan2_rpm: u16,
}

// =====================================================
// TDP 数据
// =====================================================

#[derive(Debug, Serialize)]
pub struct TdpConfig {
    pub cpu_pl1: u8,
    pub cpu_pl2: u8,
    pub cpu_pl4: u8,
    pub gpu_pl1: u8,
    pub gpu_pl2: u8,
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

/// 控制状态

#[tauri::command]
fn get_fan_control_status(
    state: tauri::State<'_, FanControlState>,
) -> bool {
    state.running.load(Ordering::SeqCst)
}

fn start_fan_control_internal(
    app: &tauri::AppHandle,
    fan_data: FanData,
    state: &FanControlState,
) -> Result<(), String> {
    // 防止重复启动
    if state.running.swap(true, Ordering::SeqCst) {
        return Err("风扇控制已经在运行".into());
    }

    let running = Arc::clone(&state.running);
    let app_handle = app.clone();
    // println!("Fan Data: {:#?}", fan_data);
    let handle = thread::spawn(move || {
        println!("================================");
        println!("风扇控制线程启动");
        println!("================================");

        // EC init
        let ec = match UniwillAcpiEc::open() {
            Ok(ec) => ec,
            Err(e) => {
                eprintln!("打开 ACPIDriver 失败: {}", e);
                running.store(false, Ordering::SeqCst);

                let _ = app_handle.emit(
                    "fan-control-status",
                    false,
                );

                return;
            }
        };

        // WMI init
        let wmi = match UniwillWmiEc::new() {
            Ok(wmi) => wmi,
            Err(e) => {
                eprintln!("打开 WMI 失败: {}", e);
                running.store(false, Ordering::SeqCst);

                let _ = app_handle.emit(
                    "fan-control-status",
                    false,
                );

                return;
            }
        };

        let get_set = |data: u64| {
            match wmi.get_set(data) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("GetSetULong failed: {e}");
                }
            }
        };

        if let Err(e) =
            ec.fan_write_mode(
                acpi::uniwillacpi::FanModeByte::FanBoostMode
            )
        {
            eprintln!("切换FAN手动模式失败: {}", e);
        }

        // 通知前端：已经启动
        let _ = app_handle.emit(
            "fan-control-status",
            true,
        );

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

            let fanm = match ec.fan_read_mode() {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("读取MODE失败: {}", e);
                    0
                }
            };

            if fanm == acpi::uniwillacpi::FanModeByte::AutoMode as u8 {
                println!("当前是 Auto Mode");

                if let Err(e) =
                    ec.fan_write_mode(
                        acpi::uniwillacpi::FanModeByte::FanBoostMode
                    )
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
            get_set(
                ((left_speed as u64 * 2) << 16)
                    | 0x0000000000001809
            );

            // 右风扇 主
            get_set(
                ((right_speed as u64 * 2) << 16)
                    | 0x0000000000001804
            );

            // ========================================
            // 5. 输出调试信息
            // ========================================

            println!(
                "CPU {}°C → Fan1 {}%  GPU {}°C → Fan2 {}%  M: {}",
                cpu_temp,
                right_speed,
                gpu_temp,
                left_speed,
                fanm
            );
            
            // ========================================
            // 6. 控制周期
            // ========================================

            thread::sleep(Duration::from_millis(1500));
        }

        println!("风扇自动控制线程退出");

        let _ = app_handle.emit(
            "fan-control-status",
            false,
        );
    });

    *state.thread.lock().unwrap() = Some(handle);

    Ok(())
}

#[tauri::command]
async fn start_fan_control(
    app: tauri::AppHandle,
    fan_data: FanData,
    state: tauri::State<'_, FanControlState>,
) -> Result<(), String> {
    start_fan_control_internal(
        &app,
        fan_data,
        &state,
    )
}

fn stop_fan_control_inner(
    app: &tauri::AppHandle,
    state: &FanControlState,
) -> Result<(), String> {

    // ========================================
    // 1. 检查是否正在运行
    // ========================================

    if !state.running.swap(false, Ordering::SeqCst) {
        return Ok(());
    }

    println!("正在停止风扇控制...");

    // ========================================
    // 2. 等待控制线程退出
    // ========================================

    if let Some(handle) =
        state.thread.lock().unwrap().take()
    {
        let _ = handle.join();
    }

    // ========================================
    // 3. 打开 EC
    // ========================================

    let ec = match UniwillAcpiEc::open() {
        Ok(ec) => ec,

        Err(e) => {
            eprintln!(
                "打开 ACPIDriver 失败: {}",
                e
            );

            let _ = app.emit(
                "fan-control-status",
                false,
            );

            return Err(
                format!(
                    "打开 ACPIDriver 失败: {}",
                    e
                )
            );
        }
    };

    // ========================================
    // 4. 恢复自动风扇模式
    // ========================================

    if let Err(e) =
        ec.fan_write_mode(
            acpi::uniwillacpi::FanModeByte::AutoMode,
        )
    {
        eprintln!(
            "恢复自动风扇模式失败: {}",
            e
        );

        let _ = app.emit(
            "fan-control-status",
            false,
        );

        return Err(
            format!(
                "恢复自动风扇模式失败: {}",
                e
            )
        );
    }

    println!("风扇控制已停止");

    // ========================================
    // 5. 通知前端
    // ========================================

    let _ = app.emit(
        "fan-control-status",
        false,
    );

    Ok(())
}

#[tauri::command]
async fn stop_fan_control(
    app: tauri::AppHandle,
    state: tauri::State<'_, FanControlState>,
) -> Result<(), String> {
    stop_fan_control_inner(
        &app,
        &state,
    )
}

#[tauri::command]
async fn set_performance_mode(mode: String) {
    let wcf = match UniwillWcfEc::new() {
        Ok(wcf) => wcf,
        Err(e) => {
            eprintln!("加载 NUCtool DLL 失败: {}", e);
                None
            }.expect("加载 NUCtool DLL 失败")
    };
    let ret = wcf.connect();
    println!("connect: {}", ret);
    match mode.as_str() {
        "power-saving" => {
            println!("省电模式");
            println!("benchmark_off apply_profile: {}", wcf.apply_benchmark_mode(0));
            println!("quiet apply_profile: {}", wcf.apply_profile(3));
        }

        "balanced" => {
            println!("平衡模式");
            println!("benchmark_off apply_profile: {}", wcf.apply_benchmark_mode(0));
            println!("balanced apply_profile: {}", wcf.apply_profile(2));
        }

        "performance" => {
            println!("性能模式");
            println!("benchmark_off apply_profile: {}", wcf.apply_benchmark_mode(0));
            println!("performance apply_profile: {}", wcf.apply_profile(1));
        }

        "benchmark-on" => {
            println!("基准模式");
            println!("benchmark_on apply_profile: {}", wcf.apply_benchmark_mode(1));
        }

        // "benchmark-off" => {
        //     println!("基准模式 OFF");
        //     println!("benchmark_off apply_profile: {}", wcf.apply_benchmark_mode(0));
        // }

        _ => {
            eprintln!("未知性能模式: {}", mode);
            return;
        }
    }
}

// =====================================================
// 读取 TDP
// =====================================================

#[tauri::command]
async fn get_tdp() -> Result<TdpConfig, String> {

    // EC init
    let ec = match UniwillAcpiEc::open() {
        Ok(ec) => ec,
        Err(e) => {
            eprintln!("打开 ACPIDriver 失败: {}", e);
            None
        }.expect("EC init Error")
    };


    // =================================================
    // CPU
    // =================================================

    let cpu_pl1 = {
        ec.cpu_read_pl1().expect("Error")
    };

    let cpu_pl2 = {
        ec.cpu_read_pl2().expect("Error")
    };

    let cpu_pl4 = {
        ec.cpu_read_pl4().expect("Error")
    };


    // =================================================
    // GPU
    // =================================================

    let gpu_pl1 = {
        ec.gpu_read_pl1().expect("Error")
    };

    let gpu_pl2 = {
        ec.gpu_read_pl2().expect("Error")
    };


    println!(
        "TDP 读取: CPU PL1={}W PL2={}W PL4={}W, GPU PL1={}W PL2={}W",
        cpu_pl1,
        cpu_pl2,
        cpu_pl4,
        gpu_pl1,
        gpu_pl2
    );


    Ok(TdpConfig {
        cpu_pl1,
        cpu_pl2,
        cpu_pl4,
        gpu_pl1,
        gpu_pl2,
    })
}


// =====================================================
// 写入 TDP
// =====================================================

#[tauri::command]
async fn set_tdp(
    tdp_type: String,
    value: u8,
) -> Result<(), String> {

    // EC init
    let ec = match UniwillAcpiEc::open() {
        Ok(ec) => ec,
        Err(e) => {
            eprintln!("打开 ACPIDriver 失败: {}", e);
            None
        }.expect("EC init Error")
    };

    match tdp_type.as_str() {

        // =============================================
        // CPU
        // =============================================

        "cpu-pl1" => {
            ec.cpu_write_pl1(value).expect("Error");
            println!("写入 CPU PL1: {} W", value);
        }


        "cpu-pl2" => {
            ec.cpu_write_pl2(value).expect("Error");
            println!("写入 CPU PL2: {} W", value);
        }


        "cpu-pl4" => {
            ec.cpu_write_pl4(value).expect("Error");
            println!("写入 CPU PL4: {} W", value);
        }


        // =============================================
        // GPU
        // =============================================

        "gpu-pl1" => {
            ec.gpu_write_pl1(value).expect("Error");
            println!("写入 GPU PL1: {} W", value);
        }


        "gpu-pl2" => {
            ec.gpu_write_pl2(value).expect("Error");
            println!("写入 GPU PL2: {} W", value);
        }

        // =============================================
        // 未知类型
        // =============================================

        _ => {

            return Err(
                format!("未知的 TDP 类型: {}", tdp_type)
            );
        }
    }


    Ok(())
}

pub fn setup(
    app: &mut tauri::App,
) -> Result<(), Box<dyn std::error::Error>> {

    // =========================
    // 启动最小化
    // =========================

    let args: Vec<String> =
        std::env::args().collect();

    let hide =
        args.iter().any(|arg| arg == "--hide");

    let window =
        app.get_webview_window("main").unwrap();

    if hide {
        println!("检测到 --hide，只保留托盘");
        window.hide()?;
    } else {
        println!("未检测到 --hide，默认显示窗口");
        window.show()?;
        window.set_focus()?;
    }

    // =========================
    // 托盘菜单
    // =========================

    let show = MenuItemBuilder::with_id("show", "显示窗口")
        .build(app)?;

    let fancontrol_on = MenuItemBuilder::with_id("fancontrol_on", "开启风扇控制")
        .build(app)?;

    let fancontrol_off = MenuItemBuilder::with_id("fancontrol_off", "关闭风扇控制")
        .build(app)?;

    let benchmark_on = MenuItemBuilder::with_id("benchmark_on", "开启基准模式")
        .build(app)?;

    let benchmark_off = MenuItemBuilder::with_id("benchmark_off", "关闭基准模式")
        .build(app)?;

    let performance = MenuItemBuilder::with_id("performance", "性能模式")
        .build(app)?;

    let balanced = MenuItemBuilder::with_id("balanced", "平衡模式")
        .build(app)?;

    let quiet = MenuItemBuilder::with_id("quiet", "省电模式")
        .build(app)?;

    let quit = MenuItemBuilder::with_id("quit", "退出")
        .build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&show)
        .item(&fancontrol_on)
        .item(&fancontrol_off)
        .item(&benchmark_on)
        .item(&benchmark_off)
        .item(&performance)
        .item(&balanced)
        .item(&quiet)
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
        .on_menu_event(move |app, event| {
            let wcf = match UniwillWcfEc::new() {
                Ok(wcf) => wcf,
                Err(e) => {
                    eprintln!("加载 NUCtool DLL 失败: {}", e);
                    None
                }.expect("加载 NUCtool DLL 失败")
            };
            let ret = wcf.connect();
            println!("connect: {}", ret);

            match event.id().as_ref() {
                "show" => {
                    if let Some(window) =
                        app.get_webview_window("main")
                    {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                },
                "fancontrol_on" => {
                    let state = app.state::<FanControlState>();
                    if !state.running.load(Ordering::SeqCst) {
                        println!("托盘启动风扇控制");
                        let fan_data = match config::load() {
                            Ok(data) => data,
                            Err(e) => {
                                eprintln!(
                                    "auto_fan load config error: {}",
                                    e
                                );
                                return;
                            }
                        };
                        if let Err(e) =
                            start_fan_control_internal(
                                &app.app_handle(),
                                fan_data,
                                &state,
                            )
                        {
                            eprintln!(
                                "托盘启动风扇控制失败: {}",
                                e
                            );
                        }
                    } else {
                        println!("托盘启动风扇控制：已经在运行 跳过");    
                    }
                    
                },
                "fancontrol_off" => {
                    let state = app.state::<FanControlState>();
                    if state.running.load(Ordering::SeqCst) {
                        println!("托盘关闭风扇控制");
                        if let Err(e) =
                            stop_fan_control_inner(
                                &app.app_handle(),
                                state.inner()
                            )
                        {
                            eprintln!(
                                "托盘关闭风扇控制失败: {}",
                                e
                            );
                        }
                    } else {
                        println!("托盘关闭风扇控制：已经关闭了 跳过")
                    }
                },
                "benchmark_on" => {
                    println!("benchmark_on apply_profile: {}", wcf.apply_benchmark_mode(1));
                },
                "benchmark_off" => {
                    println!("benchmark_off apply_profile: {}", wcf.apply_benchmark_mode(0));
                },
                "performance" => {
                    println!("performance apply_profile: {}", wcf.apply_profile(1));
                },
                "balanced" => {
                    println!("balanced apply_profile: {}", wcf.apply_profile(2));
                },
                "quiet" => {
                    println!("quiet apply_profile: {}", wcf.apply_profile(3));
                },
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

    // =========================
    // 风扇自启
    // =========================

    let auto_fan_control =
        args.iter().any(|arg| {
            arg == "--fan-control"
        });

    if auto_fan_control {
        println!(
            "检测到 --fan-control，自动启动风扇控制"
        );

        let fan_data = match config::load() {
            Ok(data) => data,

            Err(e) => {
                eprintln!(
                    "auto_fan load config error: {}",
                    e
                );
                // 配置读取失败，不启动风扇控制
                return Ok(());
            }
        };

        let state =
            app.state::<FanControlState>();

        if let Err(e) =
            start_fan_control_internal(
                &app.handle(),
                fan_data,
                &state,
            )
        {
            eprintln!(
                "自动启动风扇控制失败: {}",
                e
            );
        }
    } else {
        println!(
            "未检测到 --fan-control，风扇控制默认关闭"
        );
    }
    Ok(())
}

fn fnhook() {

    unsafe {

        // ==================================
        // 安装低级键盘 Hook
        // ==================================

        let hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(FnKeyhook),
            Some(HINSTANCE::default()),
            0,
        ).expect("Fn Key Hook Error");

        println!("======================================");
        println!("       NUCtool Fn 快捷键监听");
        println!("======================================");
        println!();
        println!("监听：");
        println!("  Fn + 1");
        println!("  Fn + 2");
        println!("  Fn + 3");
        println!("  Fn + 4");
        println!("  Fn + 5");
        println!("  Fn + 6");
        println!("  Fn + 7");
        println!("  Fn + 8");
        println!("  Fn + 9");
        println!();
        println!("等待按键...");
        println!("======================================");

        // ==================================
        // 消息循环
        // ==================================

        let mut msg = MSG::default();

        while GetMessageW(
            &mut msg,
            None,
            0,
            0,
        )
        .into()
        {
            DispatchMessageW(&msg);
        }

        // ==================================
        // 卸载 Hook
        // ==================================

        UnhookWindowsHookEx(hook).expect("卸载失败");
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    
    // 管理员权限！！！
    privilege_escalation();
    // Fn Hook
    thread::spawn(|| {
        loop {
            fnhook();
        }
    });
    
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
            stop_fan_control,
            set_performance_mode,
            get_tdp,
            set_tdp,
            get_fan_control_status
        ])
        .setup(setup)     
        .build(tauri::generate_context!())
        .expect("error while building tauri application");
    
    app.run(|app_handle: &tauri::AppHandle, event: tauri::RunEvent| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                println!("程序正在退出...");
                if let Some(state) =
                    app_handle.try_state::<FanControlState>()
                {
                    if let Err(e) =
                        stop_fan_control_inner(app_handle, state.inner())
                    {
                        eprintln!("退出时停止风扇控制失败: {}", e);
                    }
                }
            }
        })
}