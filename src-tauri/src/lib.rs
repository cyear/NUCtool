mod acpi;
mod config;
mod fan_control;
mod osd;
mod win;
use acpi::{uniwillfnkeyhook, UniwillAcpiEc, UniwillWcfEc, UniwillWmiEc, get_model, get_gpu_driver, get_gsc_driver};
use config::FanData;
use fan_control::{calculate_speed, FanControlState};
use osd::{create_osd, osd_ready, show_osd_command, show_osd_i18n, show_osd_i18n_value};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{
    include_image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State,
};
use win::{create_startup_task, is_startup_task_exists, privilege_escalation, remove_startup_task};
use windows::Win32::{
    Foundation::HINSTANCE,
    UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, MSG, WH_KEYBOARD_LL,
    },
};

use crate::acpi::uniwillwcf::NativeLightbarProfile;

#[derive(Clone, Serialize)]
struct SensorData {
    cpu_temp: u8,
    gpu_temp: u8,
    fan1_rpm: u16,
    fan2_rpm: u16,
    system_power: u8,
    bat_mah_percent: u8,
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
    pub battery_charglimit: u8,
    pub psys_pl1: u8,
}

#[derive(Debug, Serialize)]
pub struct GpuDriverInfo {
    pub name: String,
    pub version: String,
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
    let wmi = match UniwillWmiEc::new() {
        Ok(wmi) => wmi,
        Err(e) => {
            eprintln!("打开 WMI 失败: {}", e);
            return;
        }
    };
    let bat_mah_percent = (100 * wmi.get_set(0x0000010000000404).unwrap_or(0)
        / wmi.get_set(0x0000010000000402).unwrap_or(1)) as u8;
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
                system_power: ec.system_read_power().unwrap_or(0),
                bat_mah_percent: bat_mah_percent,
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
fn get_fan_control_status(state: tauri::State<'_, FanControlState>) -> bool {
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

                let _ = app_handle.emit("fan-control-status", false);

                return;
            }
        };

        // WMI init
        let wmi = match UniwillWmiEc::new() {
            Ok(wmi) => wmi,
            Err(e) => {
                eprintln!("打开 WMI 失败: {}", e);
                running.store(false, Ordering::SeqCst);

                let _ = app_handle.emit("fan-control-status", false);

                return;
            }
        };

        let get_set = |data: u64| match wmi.get_set(data) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("GetSetULong failed: {e}");
            }
        };

        if let Err(e) = ec.fan_write_mode(acpi::uniwillacpi::FanModeByte::FanBoostMode) {
            eprintln!("切换FAN手动模式失败: {}", e);
        }

        // 通知前端：已经启动
        let _ = app_handle.emit("fan-control-status", true);
        let mut fan_data_set = fan_data;
        // 0 = 独立
        // 1 = 主风扇优先
        // 2 = 分风扇优先
        match config::load_fan_mode() {
            Ok(1) => {
                fan_data_set.right_fan = fan_data_set.left_fan.clone();
                println!("主风扇优先");
                let _ = show_osd_i18n(&app_handle, "fanControl", "startMainPriority");
            }
            Ok(2) => {
                fan_data_set.left_fan = fan_data_set.right_fan.clone();
                println!("分风扇优先");
                let _ = show_osd_i18n(&app_handle, "fanControl", "startSplitPriority");
            }
            Ok(_) => {
                println!("独立");
                let _ = show_osd_i18n(&app_handle, "fanControl", "startIndependent");
            }
            Err(e) => {
                println!("读取风扇模式配置失败: {}", e);
            }
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
            let right_speed = calculate_speed(&fan_data_set.left_fan, cpu_temp);

            // GPU 风扇 分 => 左
            let left_speed = calculate_speed(&fan_data_set.right_fan, gpu_temp);

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

                if let Err(e) = ec.fan_write_mode(acpi::uniwillacpi::FanModeByte::FanBoostMode) {
                    eprintln!("切换FAN手动模式失败: {}", e);
                    let _ = show_osd_i18n_value(&app_handle, "fanControl", "fanModeError", e);
                } else {
                    println!("切换FAN手动模式成功");
                    let _ = show_osd_i18n_value(&app_handle, "fanControl", "fanModeSuccess", fanm);
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
                "CPU {}°C → Fan1 {}%  GPU {}°C → Fan2 {}%  M: {}",
                cpu_temp, right_speed, gpu_temp, left_speed, fanm
            );

            // ========================================
            // 6. 控制周期
            // ========================================

            thread::sleep(Duration::from_millis(1500));
        }
        println!("风扇自动控制线程退出");

        let _ = app_handle.emit("fan-control-status", false);
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
    start_fan_control_internal(&app, fan_data, &state)
}

fn stop_fan_control_inner(app: &tauri::AppHandle, state: &FanControlState) -> Result<(), String> {
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

    if let Some(handle) = state.thread.lock().unwrap().take() {
        let _ = handle.join();
    }

    // ========================================
    // 3. 打开 EC
    // ========================================

    let ec = match UniwillAcpiEc::open() {
        Ok(ec) => ec,

        Err(e) => {
            eprintln!("打开 ACPIDriver 失败: {}", e);

            let _ = app.emit("fan-control-status", false);

            return Err(format!("打开 ACPIDriver 失败: {}", e));
        }
    };

    // ========================================
    // 4. 恢复自动风扇模式
    // ========================================

    if let Err(e) = ec.fan_write_mode(acpi::uniwillacpi::FanModeByte::AutoMode) {
        eprintln!("恢复自动风扇模式失败: {}", e);

        let _ = app.emit("fan-control-status", false);

        return Err(format!("恢复自动风扇模式失败: {}", e));
    }

    println!("风扇控制已停止");

    // ========================================
    // 5. 通知前端
    // ========================================

    let _ = app.emit("fan-control-status", false);
    let _ = show_osd_i18n(app, "fanControl", "stopFanControl");

    Ok(())
}

#[tauri::command]
async fn stop_fan_control(
    app: tauri::AppHandle,
    state: tauri::State<'_, FanControlState>,
) -> Result<(), String> {
    stop_fan_control_inner(&app, &state)
}

#[tauri::command]
async fn set_fan_mode(mode: i32) {
    let _ = config::save_fan_mode(mode);
}

#[tauri::command]
async fn get_fan_mode() -> i32 {
    if let Ok(mode) = config::load_fan_mode() {
        mode
    } else {
        println!("读取风扇模式配置失败");
        1
    }
}

#[tauri::command]
async fn set_performance_mode(app: tauri::AppHandle, mode: String) {
    let wcf = match UniwillWcfEc::new() {
        Ok(wcf) => wcf,
        Err(e) => {
            eprintln!("加载 NUCtool DLL 失败: {}", e);
            None
        }
        .expect("加载 NUCtool DLL 失败"),
    };
    let ret = wcf.connect();
    println!("connect: {}", ret);
    match mode.as_str() {
        "power-saving" => {
            println!("省电模式");
            println!(
                "benchmark_off apply_profile: {}",
                wcf.apply_benchmark_mode(0)
            );
            let _ = show_osd_i18n(&app, "powerSavingMode", "empty");
            println!("quiet apply_profile: {}", wcf.apply_profile(3));
        }
        "balanced" => {
            println!("平衡模式");
            println!(
                "benchmark_off apply_profile: {}",
                wcf.apply_benchmark_mode(0)
            );
            let _ = show_osd_i18n(&app, "balancedMode", "empty");
            println!("balanced apply_profile: {}", wcf.apply_profile(2));
        }
        "performance" => {
            println!("性能模式");
            println!(
                "benchmark_off apply_profile: {}",
                wcf.apply_benchmark_mode(0)
            );
            let _ = show_osd_i18n(&app, "performanceMode", "empty");
            println!("performance apply_profile: {}", wcf.apply_profile(1));
        }
        "benchmark-on" => {
            println!("基准模式");
            println!(
                "benchmark_on apply_profile: {}",
                wcf.apply_benchmark_mode(1)
            );
            let _ = show_osd_i18n(&app, "benchmarkMode", "empty");
        }
        _ => {
            eprintln!("未知性能模式: {}", mode);
            return;
        }
    }
    wcf.disconnect();
}

#[tauri::command]
async fn get_performance_mode() -> i32 {
    let wcf = match UniwillWcfEc::new() {
        Ok(wcf) => wcf,
        Err(e) => {
            eprintln!("加载 NUCtool DLL 失败: {}", e);
            None
        }
        .expect("加载 NUCtool DLL 失败"),
    };
    let ret = wcf.connect();
    let state = wcf.get_current_state();
    println!("connect: {} get_performance_mode {:?}", ret, &state);
    if state.benchmark_mode == 1 {
        5
    } else {
        state.selected_profile_index
    }
}

#[tauri::command]
async fn set_power_plan(app: tauri::AppHandle, mode: i32) -> Result<(), String> {
    let wcf = match UniwillWcfEc::new() {
        Ok(wcf) => wcf,
        Err(e) => {
            eprintln!("加载 NUCtool DLL 失败: {}", e);
            None
        }
        .expect("加载 NUCtool DLL 失败"),
    };
    let ret = wcf.connect();
    println!("connect: {}", ret);
    match mode {
        0 => {
            wcf.set_power_plan(mode);
            println!("电源计划切换: 关闭 {}", mode);
            let _ = show_osd_i18n(&app, "powerPlan", "powerPlanOff");
        }
        1 => {
            wcf.set_power_plan(mode);
            println!("电源计划切换: 高性能 {}", mode);
            let _ = show_osd_i18n(&app, "powerPlan", "powerPlanHighPerformance");
        }
        2 => {
            wcf.set_power_plan(mode);
            println!("电源计划切换: 平衡 {}", mode);
            let _ = show_osd_i18n(&app, "powerPlan", "powerPlanBalanced");
        }
        3 => {
            wcf.set_power_plan(mode);
            println!("电源计划切换: 节能 {}", mode);
            let _ = show_osd_i18n(&app, "powerPlan", "powerPlanPowerSaving");
        }
        4 => {
            wcf.set_power_plan(mode);
            println!("电源计划切换: 基准高性能 {}", mode);
            let _ = show_osd_i18n(&app, "powerPlan", "powerPlanBenchmark");
        }
        _ => {
            eprintln!("错误的电源计划: {}", mode);
        }
    }
    wcf.disconnect();
    Ok(())
}

#[tauri::command]
async fn set_display_mode(app: tauri::AppHandle, mode: i32) {
    let wcf = match UniwillWcfEc::new() {
        Ok(wcf) => wcf,
        Err(e) => {
            eprintln!("加载 NUCtool DLL 失败: {}", e);
            None
        }
        .expect("加载 NUCtool DLL 失败"),
    };
    let ret = wcf.connect();
    println!("connect: {}", ret);
    println!("显示设置: {}", mode);
    if mode != 5 {
        wcf.display_enable_mode_mgmt(1);
        let _ = show_osd_i18n(&app, "displaySettings", "displaySettingsOn");
    }
    match mode {
        0 => {
            wcf.display_set_mode(mode);
            let _ = show_osd_i18n(&app, "displaySettings", "displaySettingsStandard");
        }
        1 => {
            wcf.display_set_mode(mode);
            let _ = show_osd_i18n(&app, "displaySettings", "displaySettingsGaming");
        }
        2 => {
            wcf.display_set_mode(mode);
            let _ = show_osd_i18n(&app, "displaySettings", "displaySettingsVideo");
        }
        3 => {
            wcf.display_set_mode(mode);
            let _ = show_osd_i18n(&app, "displaySettings", "displaySettingsReading");
        }
        4 => {
            wcf.display_set_mode(mode);
            let _ = show_osd_i18n(&app, "displaySettings", "displaySettingsCustom");
        }
        5 => {
            wcf.display_enable_mode_mgmt(0);
            let _ = show_osd_i18n(&app, "displaySettings", "displaySettingsOff");
        }
        _ => {
            eprintln!("错误的显示模式: {}", mode);
        }
    }
    wcf.disconnect();
}

#[tauri::command]
fn get_keyboard_led() -> bool {
    let wcf = match UniwillWcfEc::new() {
        Ok(wcf) => wcf,
        Err(e) => {
            eprintln!("加载 NUCtool DLL 失败: {}", e);
            None
        }
        .expect("加载 NUCtool DLL 失败"),
    };
    let ret = wcf.connect();
    let g = wcf.keyboard_get_leds_power();
    println!("connect: {} wcf_get_keyboard_leds_power: {}", ret, g);
    wcf.disconnect();
    if g == 1 {
        true
    } else {
        false
    }
}

#[tauri::command]
fn set_keyboard_led(app: tauri::AppHandle, enabled: bool) {
    let wcf = match UniwillWcfEc::new() {
        Ok(wcf) => wcf,
        Err(e) => {
            eprintln!("加载 NUCtool DLL 失败: {}", e);
            None
        }
        .expect("加载 NUCtool DLL 失败"),
    };
    let ret = wcf.connect();
    println!("connect: {} set_keyboard_led: {}", ret, enabled);
    if enabled {
        wcf.keyboard_set_leds_power(1);
        let _ = show_osd_i18n(&app, "keyboardLed", "keyboardLedOn");
    } else {
        wcf.keyboard_set_leds_power(0);
        let _ = show_osd_i18n(&app, "keyboardLed", "keyboardLedOff");
    }
    wcf.disconnect();
}

#[tauri::command]
async fn get_lightbar_profile() -> NativeLightbarProfile {
    let wcf = match UniwillWcfEc::new() {
        Ok(wcf) => wcf,
        Err(e) => {
            eprintln!("加载 NUCtool DLL 失败: {}", e);
            None
        }
        .expect("加载 NUCtool DLL 失败"),
    };
    let ret = wcf.connect();
    let profile = wcf.lightbar_get_profile();
    println!("connect: {} get_lightbar_profile: {:?}", ret, &profile);
    profile
}

#[tauri::command]
async fn set_lightbar_profile(app: tauri::AppHandle, profile: NativeLightbarProfile) {
    let wcf = match UniwillWcfEc::new() {
        Ok(wcf) => wcf,
        Err(e) => {
            eprintln!("加载 NUCtool DLL 失败: {}", e);
            None
        }
        .expect("加载 NUCtool DLL 失败"),
    };
    let ret = wcf.connect();
    let profile = wcf.lightbar_set_profile(profile);
    println!("connect: {} set_lightbar_profile: {:?}", ret, &profile);
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
        }
        .expect("EC init Error"),
    };

    // =================================================
    // CPU
    // =================================================

    let cpu_pl1 = ec.cpu_read_pl1().expect("Error");

    let cpu_pl2 = ec.cpu_read_pl2().expect("Error");

    let cpu_pl4 = ec.cpu_read_pl4().expect("Error");

    // =================================================
    // GPU
    // =================================================

    let gpu_pl1 = ec.gpu_read_pl1().expect("Error");

    let gpu_pl2 = ec.gpu_read_pl2().expect("Error");

    // =================================================
    // Battery
    // =================================================
    let wcf = match UniwillWcfEc::new() {
        Ok(wcf) => wcf,
        Err(e) => {
            eprintln!("加载 NUCtool DLL 失败: {}", e);
            None
        }
        .expect("加载 NUCtool DLL 失败"),
    };
    let ret = wcf.connect();
    println!("connect: {}", ret);

    let battery_charglimit = wcf.battery_get_charging_level() as u8;
    wcf.disconnect();

    // =================================================
    // PSYS PL1
    // =================================================

    let psys_pl1 = ec.psys_read_pl1().expect("Error");

    println!(
        "TDP 读取: CPU PL1={}W PL2={}W PL4={}W, GPU PL1={}W PL2={}W, Battery_Charging_limit={}%, PSYS_PL1={}W",
        cpu_pl1, cpu_pl2, cpu_pl4, gpu_pl1, gpu_pl2, battery_charglimit, psys_pl1
    );

    Ok(TdpConfig {
        cpu_pl1,
        cpu_pl2,
        cpu_pl4,
        gpu_pl1,
        gpu_pl2,
        battery_charglimit,
        psys_pl1,
    })
}

// =====================================================
// 写入 TDP
// =====================================================

#[tauri::command]
async fn set_tdp(app: tauri::AppHandle, tdp_type: String, value: u8) -> Result<(), String> {
    // EC init
    let ec = match UniwillAcpiEc::open() {
        Ok(ec) => ec,
        Err(e) => {
            eprintln!("打开 ACPIDriver 失败: {}", e);
            None
        }
        .expect("EC init Error"),
    };

    match tdp_type.as_str() {
        // =============================================
        // CPU
        // =============================================
        "cpu-pl1" => {
            ec.cpu_write_pl1(value).expect("Error");
            println!("写入 CPU PL1: {} W", value);
            let _ = show_osd_i18n_value(&app, "tdpSettings", "cpuPl1", value);
        }

        "cpu-pl2" => {
            ec.cpu_write_pl2(value).expect("Error");
            println!("写入 CPU PL2: {} W", value);
            let _ = show_osd_i18n_value(&app, "tdpSettings", "cpuPl2", value);
        }

        "cpu-pl4" => {
            ec.cpu_write_pl4(value).expect("Error");
            println!("写入 CPU PL4: {} W", value);
            let _ = show_osd_i18n_value(&app, "tdpSettings", "cpuPl4", value);
        }

        // =============================================
        // GPU
        // =============================================
        "gpu-pl1" => {
            ec.gpu_write_pl1(value).expect("Error");
            println!("写入 GPU PL1: {} W", value);
            let _ = show_osd_i18n_value(&app, "tdpSettings", "gpuPl1", value);
        }

        "gpu-pl2" => {
            ec.gpu_write_pl2(value).expect("Error");
            println!("写入 GPU PL2: {} W", value);
            let _ = show_osd_i18n_value(&app, "tdpSettings", "gpuPl2", value);
        }

        // =============================================
        // Battery
        // =============================================
        "battery_charglimit" => {
            let wcf = match UniwillWcfEc::new() {
                Ok(wcf) => wcf,
                Err(e) => {
                    eprintln!("加载 NUCtool DLL 失败: {}", e);
                    None
                }
                .expect("加载 NUCtool DLL 失败"),
            };
            let ret = wcf.connect();
            println!("connect: {}", ret);
            wcf.battery_set_charging_level(value as i32);
            println!("写入Battery Charging limit： {}%", value);
            let _ = show_osd_i18n_value(&app, "batterySettings", "batteryChargingLimit", value);
            wcf.disconnect();
        }

        // =============================================
        // PSYS PL1
        // =============================================
        "psys_pl1" => {
            ec.psys_write_pl1(value).expect("Error");
            println!("写入PSYS PL1： {} W", value);
            let _ = show_osd_i18n_value(&app, "tdpSettings", "psysPl1", value);
        }

        // =============================================
        // 未知类型
        // =============================================
        _ => {
            return Err(format!("未知的 TDP 类型: {}", tdp_type));
        }
    }

    Ok(())
}

#[tauri::command]
async fn get_autostart() -> Result<bool, String> {
    match is_startup_task_exists() {
        Ok(b) => {
            println!("自启动状态: {}", b);
            Ok(b)
        }
        Err(e) => {
            eprintln!("获取自启动失败: {}", e);
            Ok(false)
        }
    }
}

#[tauri::command]
async fn set_autostart(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    if enabled {
        if let Err(e) = create_startup_task() {
            println!("创建开机自启动任务计划失败: {}", e);
            let _ = show_osd_i18n_value(&app, "autostartCreateFailed", "error", e);
        } else {
            println!("添加开机自启动任务计划成功");
            let _ = show_osd_i18n(&app, "autostartCreateSuccess", "empty");
        }
    } else {
        if let Err(e) = remove_startup_task() {
            println!("删除开机自启动任务计划失败: {}", e);
            let _ = show_osd_i18n_value(&app, "autostartRemoveFailed", "error", e);
        } else {
            println!("删除开机自启动任务计划成功");
            let _ = show_osd_i18n(&app, "autostartRemoveSuccess", "empty");
        }
    }
    Ok(())
}

#[tauri::command]
async fn get_sys_model() -> String {
    let model = get_model().expect("get_model error");
    println!("get_model Model: {}", model);
    model
}

#[tauri::command]
async fn get_sys_arc_gpu_driver() -> Result<Option<GpuDriverInfo>, String> {
    get_gpu_driver()
        .map_err(|e| e.to_string())
        .map(|items| {
            items
                .into_iter()
                .find(|(name, _)| name.contains("Arc"))
                .map(|(name, version)| GpuDriverInfo {
                    name,
                    version,
                })
        })
}

#[tauri::command]
async fn get_sys_gsc_driver() -> Result<Option<String>, String> {
    get_gsc_driver()
        .map_err(|e| e.to_string())
}

fn setup(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    // OSD
    let osd = args.iter().any(|arg| arg == "--no-osd");
    if !osd {
        if let Err(e) = create_osd(&app.handle()) {
            eprintln!("OSD ERROR: {}", e);
        }
    } else {
        println!("检测到 --no-osd: {}/跳过创建OSD界面", osd);
    }

    // =========================
    // 启动最小化
    // =========================

    let hide = args.iter().any(|arg| arg == "--hide");

    let window = app.get_webview_window("main").unwrap();

    if hide {
        println!("检测到 --hide: {}/开机自启，只保留托盘", hide);
        window.hide()?;
    } else {
        println!("未检测到 --hide，默认显示窗口");
        window.show()?;
        window.set_focus()?;
    }

    // =========================
    // 托盘菜单
    // =========================

    let show = MenuItemBuilder::with_id("show", "显示窗口").build(app)?;

    let fancontrol_on = MenuItemBuilder::with_id("fancontrol_on", "开启风扇控制").build(app)?;

    let fancontrol_off = MenuItemBuilder::with_id("fancontrol_off", "关闭风扇控制").build(app)?;

    let benchmark_on = MenuItemBuilder::with_id("benchmark_on", "开启基准模式").build(app)?;

    let benchmark_off = MenuItemBuilder::with_id("benchmark_off", "关闭基准模式").build(app)?;

    let performance = MenuItemBuilder::with_id("performance", "性能模式").build(app)?;

    let balanced = MenuItemBuilder::with_id("balanced", "平衡模式").build(app)?;

    let quiet = MenuItemBuilder::with_id("quiet", "省电模式").build(app)?;

    let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;

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
                }
                .expect("加载 NUCtool DLL 失败"),
            };
            let ret = wcf.connect();
            println!("connect: {}", ret);

            match event.id().as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "fancontrol_on" => {
                    let state = app.state::<FanControlState>();
                    if !state.running.load(Ordering::SeqCst) {
                        println!("托盘启动风扇控制");
                        let fan_data = match config::load() {
                            Ok(data) => data,
                            Err(e) => {
                                eprintln!("auto_fan load config error: {}", e);
                                return;
                            }
                        };
                        if let Err(e) =
                            start_fan_control_internal(&app.app_handle(), fan_data, &state)
                        {
                            eprintln!("托盘启动风扇控制失败: {}", e);
                        }
                    } else {
                        println!("托盘启动风扇控制：已经在运行 跳过");
                    }
                }
                "fancontrol_off" => {
                    let state = app.state::<FanControlState>();
                    if state.running.load(Ordering::SeqCst) {
                        println!("托盘关闭风扇控制");
                        if let Err(e) = stop_fan_control_inner(&app.app_handle(), state.inner()) {
                            eprintln!("托盘关闭风扇控制失败: {}", e);
                        }
                    } else {
                        println!("托盘关闭风扇控制：已经关闭了 跳过")
                    }
                }
                "benchmark_on" => {
                    println!(
                        "benchmark_on apply_profile: {}",
                        wcf.apply_benchmark_mode(1)
                    );
                }
                "benchmark_off" => {
                    println!(
                        "benchmark_off apply_profile: {}",
                        wcf.apply_benchmark_mode(0)
                    );
                }
                "performance" => {
                    println!("performance apply_profile: {}", wcf.apply_profile(1));
                }
                "balanced" => {
                    println!("balanced apply_profile: {}", wcf.apply_profile(2));
                }
                "quiet" => {
                    println!("quiet apply_profile: {}", wcf.apply_profile(3));
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
                if button == MouseButton::Left && button_state == MouseButtonState::Up {
                    if let Some(window) = tray.app_handle().get_webview_window("main") {
                        let visible = window.is_visible().unwrap_or(false);

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

    let auto_fan_control = args.iter().any(|arg| arg == "--fan-control");
    if auto_fan_control {
        println!("检测到 --fan-control，自动启动风扇控制");
        let fan_data = match config::load() {
            Ok(data) => data,
            Err(e) => {
                eprintln!("auto_fan load config error: {}", e);
                // 配置读取失败，不启动风扇控制
                return Ok(());
            }
        };
        let state = app.state::<FanControlState>();
        if let Err(e) = start_fan_control_internal(&app.handle(), fan_data, &state) {
            eprintln!("自动启动风扇控制失败: {}", e);
        };
    } else {
        println!("未检测到 --fan-control，风扇控制默认关闭");
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
            Some(uniwillfnkeyhook),
            Some(HINSTANCE::default()),
            0,
        )
        .expect("Fn Key Hook Error");

        println!("======================================");
        println!("       NUCtool Fn 快捷键监听");
        println!("======================================");

        // ==================================
        // 消息循环
        // ==================================

        let mut msg = MSG::default();

        while GetMessageW(&mut msg, None, 0, 0).into() {
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
    thread::spawn(|| loop {
        fnhook();
    });
    let app = tauri::Builder::default()
        // .plugin(tauri_plugin_autostart::Builder::new().build())
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
            get_performance_mode,
            get_tdp,
            set_tdp,
            get_fan_control_status,
            get_autostart,
            set_autostart,
            set_power_plan,
            set_display_mode,
            get_keyboard_led,
            set_keyboard_led,
            osd_ready,
            show_osd_command,
            set_fan_mode,
            get_fan_mode,
            get_lightbar_profile,
            set_lightbar_profile,
            get_sys_model,
            get_sys_arc_gpu_driver,
            get_sys_gsc_driver,
        ])
        .setup(setup)
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle: &tauri::AppHandle, event: tauri::RunEvent| {
        if let tauri::RunEvent::ExitRequested { .. } = event {
            println!("程序正在退出...");
            if let Some(state) = app_handle.try_state::<FanControlState>() {
                if let Err(e) = stop_fan_control_inner(app_handle, state.inner()) {
                    eprintln!("退出时停止风扇控制失败: {}", e);
                }
            }
        }
    })
}
