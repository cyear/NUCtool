#[macro_use]
mod logging;
mod acpi;
mod config;
mod fan_control;
mod osd;
mod win;
mod window_push;
use acpi::{
    uniwillfnkeyhook,
    UniwillWcfEc, UniwillWmiEc,
    get_model, get_gpu_driver, get_gsc_driver,
    KeyboardBacklight,NativeLightbarProfile, acpi_ec_worker
};
use config::FanData;
use fan_control::{calculate_speed, FanControlState};
use osd::{
    create_osd,
    osd_ready, show_osd_command,
    show_osd_i18n, show_osd_i18n_value
};
use win::{
    create_startup_task,
    is_startup_task_exists, privilege_escalation, remove_startup_task, keyboard_registry
};
use window_push::WindowPushState;
use serde::Serialize;
use std::{
    fs,
    thread,
    time::Duration,
    process::Command,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering}
    }
};
use tauri::{
    include_image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State,
};
use windows::Win32::{
    Foundation::HINSTANCE,
    UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, MSG, WH_KEYBOARD_LL,
    },
};

#[derive(Clone, Serialize)]
struct SensorData {
    cpu_temp: u8,
    gpu_temp: u8,
    fan1_rpm: u16,
    fan2_rpm: u16,
    system_power: u8,
    bat_mah_percent: u8,
}

#[derive(Debug, Serialize)]
pub struct TdpConfig {
    pub cpu_pl1: u8,
    pub cpu_pl2: u8,
    pub cpu_pl4: u8,
    pub gpu_pl1: u8,
    pub gpu_pl2: u8,
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

#[derive(Clone, Default)]
struct NewFanMonitorState {
    running: Arc<AtomicBool>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct NewFanStatus {
    pub mode: bool,
    pub independent: bool,
    pub fan: bool,
    pub duty: bool,
}

#[tauri::command]
fn start_sensor_loop(app: AppHandle, state: State<AppState>, push_state: tauri::State<'_, WindowPushState>) {
    // 防止重复启动
    if state.running.swap(true, Ordering::SeqCst) {
        return;
    }
    let running = state.running.clone();
    let push_state = push_state.inner().clone();
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
        while running.load(Ordering::SeqCst) {
            if !push_state.is_enabled() {
                thread::sleep(Duration::from_millis(3000));
                continue;
            }
            let data = match acpi_ec_worker().call(move |ec| {
                SensorData {
                    cpu_temp: ec.cpu_temperature().unwrap_or(0),
                    gpu_temp: ec.gpu_temperature().unwrap_or(0),
                    fan1_rpm: ec.fan1_rpm().unwrap_or(0),
                    fan2_rpm: ec.fan2_rpm().unwrap_or(0),
                    system_power: ec.system_read_power().unwrap_or(0),
                    bat_mah_percent,
                }
            }) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("读取 EC 传感器失败: {}", e);
                    thread::sleep(Duration::from_millis(3000));
                    continue;
                }
            };

            let _ = handle.emit("sensor-update", data);

            thread::sleep(Duration::from_millis(2500));
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

        if let Err(e) = acpi_ec_worker().call(|ec| { ec.fan_write_manual(false) })
        {
            eprintln!("切换FAN手动模式失败: {}", e);
        };

        // 通知前端：已经启动
        let _ = app_handle.emit("fan-control-status", true);
        // 0 = 独立 1 = 主风扇优先  2 = 分风扇优先
        let fan_mode: i32;
        match config::load_fan_mode() {
            Ok(1) => {
                fan_mode = 1;
                println!("主风扇优先");
                let _ = show_osd_i18n(&app_handle, "fanControl", "startMainPriority");
            }
            Ok(2) => {
                fan_mode = 2;
                println!("分风扇优先");
                let _ = show_osd_i18n(&app_handle, "fanControl", "startSplitPriority");
            }
            Ok(_) => {
                fan_mode = 0;
                println!("独立");
                let _ = show_osd_i18n(&app_handle, "fanControl", "startIndependent");
            }
            Err(e) => {
                fan_mode = 1;
                println!("读取风扇模式配置失败: {}", e);
                println!("主风扇优先 默认");
                let _ = show_osd_i18n(&app_handle, "fanControl", "startMainPriority");
            }
        }
        while running.load(Ordering::SeqCst) {
            let ec_result = acpi_ec_worker().call(|ec| {
                let cpu_temp = ec.cpu_temperature().unwrap_or(0);
                let gpu_temp = ec.gpu_temperature().unwrap_or(0);
                if !ec.fan_read_manual() {
                    ec.fan_write_manual(false);
                }
            (cpu_temp, gpu_temp)
            });
            let (cpu_temp, gpu_temp) = match ec_result {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("访问 EC 失败: {}", e);
                    thread::sleep(Duration::from_millis(1500));
                    continue;
                }
            };
            
            // &fan_data.left_fan M
            // &fan_data.right_fan S

            // CPU 风扇 主 => 右
            let mut right_speed = calculate_speed(&fan_data.left_fan, cpu_temp);
            // GPU 风扇 分 => 左
            let mut left_speed = calculate_speed(&fan_data.right_fan, gpu_temp);

            if fan_mode == 1 {
                left_speed = right_speed;
            } else if fan_mode == 2 {
                right_speed = left_speed;
            }

            // 左风扇 分
            get_set(((left_speed as u64 * 2) << 16) | 0x0000000000001809);

            // 右风扇 主
            get_set(((right_speed as u64 * 2) << 16) | 0x0000000000001804);

            println!(
                "CPU {}°C → Fan1 {}%  GPU {}°C → Fan2 {}%",
                cpu_temp, right_speed, gpu_temp, left_speed
            );

            thread::sleep(Duration::from_millis(2500));
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
    if !state.running.swap(false, Ordering::SeqCst) {
        return Ok(());
    }
    println!("正在停止风扇控制...");
    if let Some(handle) = state.thread.lock().unwrap().take() {
        let _ = handle.join();
    }
    if let Err(e) = acpi_ec_worker().call(|ec| { ec.fan_write_manual(true) })
    {   
        eprintln!("恢复自动风扇模式失败: {}", e);
    } else {
        let _ = app.emit("fan-control-status", false);
        let _ = show_osd_i18n(app, "fanControl", "stopFanControl");
    };
    println!("风扇控制已停止");
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
async fn set_fan_mode_file(mode: i32) {
    let _ = config::save_fan_mode(mode);
}

#[tauri::command]
async fn get_fan_mode_file() -> i32 {
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
async fn get_keyboard_led(ac: bool) -> KeyboardBacklight {
    let result = acpi_ec_worker().call(move |ec| { ec.keyboard_read(ac) });
    return match result {
        Ok(ret) => ret,
        Err(e) => {
            eprintln!("get_keyboard_led 失败: {}", e);
            KeyboardBacklight {
                enabled: false,
                brightness: 0,
                rainbow: false,
                red: 0,
                green: 0,
                blue: 0,
            }
        }
    };
}

#[tauri::command]
async fn set_keyboard_enabled(enable: bool, ac: bool) {
    if let Err(e) = acpi_ec_worker().call(move |ec| { ec.keyboard_write_enable(enable, ac) })
    {
        eprintln!("set_keyboard_enabled 失败: {}", e);
    }
    let _ = keyboard_registry::set_power(enable);
}

#[tauri::command]
async fn set_keyboard_brightness(brightness: u8, ac: bool) {
    if let Err(e) = acpi_ec_worker().call(move |ec| { ec.keyboard_write_brightness(brightness, ac) })
    {
        eprintln!("set_keyboard_brightness 失败: {}", e);
    }
    let _ = keyboard_registry::set_brightness(brightness, ac);
}

#[tauri::command]
async fn set_keyboard_rainbow(rainbow: bool, ac: bool) {
    if let Err(e) = acpi_ec_worker().call(move |ec| { ec.keyboard_write_rainbow(rainbow, ac) })
    {
        eprintln!("set_keyboard_rainbow 失败: {}", e);
    }
    let _ = keyboard_registry::set_effect(rainbow);
}

#[tauri::command]
async fn set_keyboard_red(red: u8, ac: bool) {
    if let Err(e) = acpi_ec_worker().call(move |ec| { ec.keyboard_write_red(red, ac) })
    {
        eprintln!("set_keyboard_red 失败: {}", e);
    }
    let _ = keyboard_registry::set_red(red, ac);
}

#[tauri::command]
async fn set_keyboard_green(green: u8, ac: bool) {
    if let Err(e) = acpi_ec_worker().call(move |ec| { ec.keyboard_write_green(green, ac) })
    {
        eprintln!("set_keyboard_green 失败: {}", e);
    }
    let _ = keyboard_registry::set_green(green, ac);
}

#[tauri::command]
async fn set_keyboard_blue(blue: u8, ac: bool) {
    if let Err(e) = acpi_ec_worker().call(move |ec| { ec.keyboard_write_blue(blue, ac) })
    {
        eprintln!("set_keyboard_blue 失败: {}", e);
    }
    let _ = keyboard_registry::set_blue(blue, ac);
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
async fn set_lightbar_profile(profile: NativeLightbarProfile) {
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

#[tauri::command]
async fn get_tdp() -> Result<TdpConfig, String> {
    let (cpu_pl1, cpu_pl2, cpu_pl4, gpu_pl1, gpu_pl2, psys_pl1) = acpi_ec_worker().call(|ec| {
        let cpu_pl1 = ec.cpu_read_pl1();
        let cpu_pl2 = ec.cpu_read_pl2();
        let cpu_pl4 = ec.cpu_read_pl4();
        let gpu_pl1 = ec.gpu_read_pl1().expect("Error");
        let gpu_pl2 = ec.gpu_read_pl2().expect("Error");
        let psys_pl1 = ec.psys_read_pl1().expect("Error");
        println!(
            "TDP 读取: CPU PL1={}W PL2={}W PL4={}W, GPU PL1={}W PL2={}W, PSYS_PL1={}W",
            cpu_pl1, cpu_pl2, cpu_pl4, gpu_pl1, gpu_pl2, psys_pl1
        );
        (cpu_pl1, cpu_pl2, cpu_pl4, gpu_pl1, gpu_pl2, psys_pl1)
    }).expect("get_tdp Error");
    Ok(TdpConfig {
        cpu_pl1,
        cpu_pl2,
        cpu_pl4,
        gpu_pl1,
        gpu_pl2,
        psys_pl1,
    })
}

#[tauri::command]
async fn set_tdp(app: tauri::AppHandle, tdp_type: String, value: u8) -> Result<(), String> {
    if let Err(e) = acpi_ec_worker().call(move |ec| { 
        match tdp_type.as_str() {
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
            "psys_pl1" => {
                ec.psys_write_pl1(value).expect("Error");
                println!("写入PSYS PL1： {} W", value);
                let _ = show_osd_i18n_value(&app, "tdpSettings", "psysPl1", value);
            }
            _ => {
                eprintln!("未知的 TDP 类型: {}", tdp_type)
            }
        };
    }){ eprintln!("set_tdp 失败: {}", e); }
    Ok(())
}


// =================================================
// Battery
// =================================================    
#[tauri::command]
async fn get_battery_charging_level() -> i32 {
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

    let battery_charglimit = wcf.battery_get_charging_level();
    wcf.disconnect();
    battery_charglimit
}

#[tauri::command]
async fn set_battery_charging_level(app: tauri::AppHandle, level: i32) {
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
    wcf.battery_set_charging_level(level);
    println!("写入Battery Charging limit： {}%", level);
    let _ = show_osd_i18n_value(&app, "batterySettings", "batteryChargingLimit", level);
    wcf.disconnect();
}

#[tauri::command]
async fn get_battery_mode() -> i32 {
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

    let mode = wcf.battery_get_mode();
    wcf.disconnect();
    mode
}

#[tauri::command]
async fn set_battery_mode(app: tauri::AppHandle, mode: i32) {
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
            let _ = show_osd_i18n_value(&app, "batterySettings", "batteryHealthFull", mode);
        }
        1 => {
            let _ = show_osd_i18n_value(&app, "batterySettings", "batteryHealthCustom", mode);
        }
        2 => {
            let _ = show_osd_i18n_value(&app, "batterySettings", "batteryHealthBest", mode);
        }
        _ => {
            let _ = show_osd_i18n_value(&app, "batterySettings", "error", mode);
        }
    }
    let _ = wcf.battery_set_mode(mode);
    wcf.disconnect();
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

#[tauri::command]
fn start_newfan_monitor(
    app: tauri::AppHandle,
    push_state: tauri::State<'_, WindowPushState>,
    monitor_state: tauri::State<'_, NewFanMonitorState>,
) -> Result<(), String> {
    let push_state = push_state.inner().clone();
    let running = monitor_state.running.clone();

    // 原子操作：只有第一个调用者能够启动线程。
    if running.swap(true, Ordering::AcqRel) {
        eprintln!("[NewFan] 监控线程已运行，忽略重复启动");
        return Ok(());
    }

    let running_for_thread = running.clone();

    let spawn_result = std::thread::Builder::new()
        .name("newfan-monitor".to_string())
        .spawn(move || {
            eprintln!("[NewFan] 监控线程启动");
            loop {
                if !running_for_thread.load(Ordering::Acquire) {
                    break;
                }
                std::thread::sleep(Duration::from_secs(3));
                if !running_for_thread.load(Ordering::Acquire) {
                    break;
                }
                if !push_state.is_enabled() {
                    continue;
                }
                let status = match acpi_ec_worker().call(|ec| {
                    NewFanStatus {
                        mode: ec.fan_read_manual(),
                        independent: ec.fan_read_custom_table_1(),
                        fan: ec.fan_read_custom_table_2(),
                        duty: ec.fan_read_duty(),
                    }
                }) {
                    Ok(status) => status,
                    Err(e) => {
                        eprintln!("[NewFan] 读取 EC 状态失败: {}", e);
                        continue;
                    }
                };
                if !running_for_thread.load(Ordering::Acquire) {
                    break;
                }
                if let Err(e) = app.emit("newfan-status", status) {
                    eprintln!("[NewFan] 发送状态失败: {}", e);
                    break;
                }
            }
            // 线程退出后允许重新启动。
            running_for_thread.store(false, Ordering::Release);
            eprintln!("[NewFan] 监控线程退出");
        });
    if let Err(e) = spawn_result {
        // 线程创建失败，恢复启动标志。
        running.store(false, Ordering::Release);
        return Err(format!("创建 NewFan 监控线程失败: {}", e));
    }

    Ok(())
}

#[tauri::command]
async fn start_newfan_write(app: tauri::AppHandle, fandata: FanData) {
    start_newfan(&app, fandata);
}

fn start_newfan(app: &tauri::AppHandle, fandata: FanData) {
    let app = app.clone();
    if let Err(e) = acpi_ec_worker().call(move |ec| { 
        ec.fan_write_init();
        ec.fan_write_set(fandata);
        println!("start_newfan: Ok");
        let _ = show_osd_i18n(&app, "fanControl", "startEcFan");
     }) {
        eprintln!("start_newfan 失败: {}", e);
    }
}

#[tauri::command]
async fn set_fan_max(app: tauri::AppHandle) {
    set_fan_max_auto(&app);
}

fn set_fan_max_auto(app: &tauri::AppHandle) {
    let app = app.clone();
    if let Err(e) = acpi_ec_worker().call(move |ec| {
        if ec.fan_read_manual() {
            ec.fan_write_manual(false);
            let _ = show_osd_i18n(&app, "fanControl", "fanMax");
        } else {
            ec.fan_write_manual(true);
            let _ = show_osd_i18n(&app, "fanControl", "fanAuto");
        }
    }) {
        eprintln!("set_fan_max: {}", e);
    };
}
#[tauri::command]
async fn stop_newfan_write(app: tauri::AppHandle) {
    let app = app.clone();
    if let Err(e) = acpi_ec_worker().call(move |ec| {
        ec.fan_write_close(); 
        let _ = show_osd_i18n(&app, "fanControl", "stopFanControl");
    }) {
        eprintln!("stop_newfan_write: {}", e);
    };
}

#[tauri::command]
async fn read_bios_nvram() -> Result<String, String> {
    let dir = config::dll_dir()?;
    let config_dir = config::config_dir();

    let exe = dir.join("SCEWIN_64.exe");
    let driver1 = dir.join("amifldrv64.sys");
    let driver2 = dir.join("amigendrv64.sys");

    // 读取 BIOS 后保存的备份文件。
    let output_file = config_dir.join("nvram_bak.txt");

    // 检查工具及驱动文件。
    for path in [&exe, &driver1, &driver2] {
        if !path.is_file() {
            let msg = format!("缺少 BIOS NVRAM 工具文件：{}", path.display());
            eprintln!("{msg}");
            return Err(msg);
        }
    }

    // 确保配置目录存在。
    fs::create_dir_all(&config_dir).map_err(|e| {
        format!(
            "创建配置目录失败：{}，错误：{e}",
            config_dir.display()
        )
    })?;

    println!("开始读取 BIOS NVRAM...");
    println!("工具路径：{}", exe.display());
    println!("备份路径：{}", output_file.display());

    // 删除旧备份，避免读取到上一次的结果。
    if output_file.exists() {
        fs::remove_file(&output_file).map_err(|e| {
            format!(
                "删除旧 NVRAM 备份失败：{}，错误：{e}",
                output_file.display()
            )
        })?;
    }
    // 文件先生成在工具目录中，随后移动到配置目录。
    let result = Command::new(&exe)
        .args(["/o", "/s", "nvram_bak.txt"])
        .current_dir(&dir)
        .output()
        .map_err(|e| format!("启动 BIOS NVRAM 工具失败：{e}"))?;

    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);

    if !stdout.trim().is_empty() {
        println!("SCEWIN 输出：\n{stdout}");
    }

    if !stderr.trim().is_empty() {
        eprintln!("SCEWIN 错误输出：\n{stderr}");
    }

    if !result.status.success() {
        let msg = format!(
            "BIOS NVRAM 导出失败，退出码：{:?}",
            result.status.code()
        );
        eprintln!("{msg}");
        return Err(msg);
    }

    // SCEWIN 实际生成文件的位置。
    let generated_file = dir.join("nvram_bak.txt");

    if !generated_file.is_file() {
        let msg = format!(
            "工具未生成 NVRAM 文件：{}",
            generated_file.display()
        );
        eprintln!("{msg}");
        return Err(msg);
    }

    // 读取生成的文件。
    let bytes = fs::read(&generated_file)
        .map_err(|e| format!("读取 NVRAM 文件失败：{e}"))?;

    // 移除 UTF-8 BOM。
    let bytes = bytes
        .strip_prefix(&[0xEF, 0xBB, 0xBF])
        .unwrap_or(&bytes);

    let text = String::from_utf8_lossy(bytes).into_owned();

    // 验证导出内容。
    if !text.contains("Setup Question") {
        let msg = "导出文件中未找到有效的 BIOS 配置区块".to_string();
        eprintln!("{msg}");
        return Err(msg);
    }

    // 将原始文件复制到配置目录，保留工具目录中的生成文件。
    fs::write(&output_file, text.as_bytes()).map_err(|e| {
        format!(
            "保存 NVRAM 备份失败：{}，错误：{e}",
            output_file.display()
        )
    })?;

    println!(
        "BIOS NVRAM 读取成功，共 {} 字节，备份路径：{}",
        text.len(),
        output_file.display()
    );

    Ok(text)
}

#[tauri::command]
fn export_nvram(text: String) -> Result<String, String> {
    if text.trim().is_empty() {
        return Err("NVRAM 数据为空，无法导出。".to_string());
    }

    let config_dir = config::config_dir();

    fs::create_dir_all(&config_dir).map_err(|e| {
        format!(
            "创建配置目录失败：{}，错误：{e}",
            config_dir.display()
        )
    })?;

    let output_file = config_dir.join("nvram.txt");

    fs::write(&output_file, text.as_bytes()).map_err(|e| {
        format!(
            "导出 NVRAM 失败：{}，错误：{e}",
            output_file.display()
        )
    })?;

    println!("NVRAM 已导出至：{}", output_file.display());

    Ok(output_file.display().to_string())
}

#[tauri::command]
async fn write_bios_nvram(password: String) -> Result<(), String> {
    use std::fs;
    use std::process::Command;

    if password.trim().is_empty() {
        return Err("BIOS 管理员密码不能为空。".to_string());
    }

    // SCEWIN 工具和驱动位于 DLL 目录。
    let dir = config::dll_dir()?;

    // NVRAM 文件位于配置目录。
    let config_dir = config::config_dir();

    let exe = dir.join("SCEWIN_64.exe");
    let driver1 = dir.join("amifldrv64.sys");
    let driver2 = dir.join("amigendrv64.sys");
    let nvram_file = config_dir.join("nvram.txt");

    // 检查工具和驱动文件。
    for path in [&exe, &driver1, &driver2] {
        if !path.is_file() {
            let msg = format!("缺少 BIOS NVRAM 工具文件：{}", path.display());
            eprintln!("{msg}");
            return Err(msg);
        }
    }

    // 检查 NVRAM 文件。
    if !nvram_file.is_file() {
        let msg = format!(
            "未找到 NVRAM 文件：{}\n请先导出或保存修改后的 NVRAM。",
            nvram_file.display()
        );
        eprintln!("{msg}");
        return Err(msg);
    }

    // 确保配置目录存在。
    fs::create_dir_all(&config_dir).map_err(|e| {
        format!(
            "创建配置目录失败：{}，错误：{e}",
            config_dir.display()
        )
    })?;

    println!("开始写入 BIOS NVRAM...");
    println!("工具路径：{}", exe.display());
    println!("NVRAM 路径：{}", nvram_file.display());

    // 执行 BIOS NVRAM 写入命令。
    let result = Command::new(&exe)
        .arg("/cpwd")
        .arg(&password)
        .args(["/i", "/s"])
        .arg(&nvram_file)
        .current_dir(&dir)
        .output()
        .map_err(|e| {
            let msg = format!("启动 BIOS NVRAM 工具失败：{e}");
            eprintln!("{msg}");
            msg
        })?;

    // 输出标准输出。
    let stdout = String::from_utf8_lossy(&result.stdout);
    if !stdout.trim().is_empty() {
        println!("SCEWIN 输出：\n{stdout}");
    }

    // 输出标准错误。
    let stderr = String::from_utf8_lossy(&result.stderr);
    if !stderr.trim().is_empty() {
        eprintln!("SCEWIN 错误输出：\n{stderr}");
    }

    // 检查进程退出状态。
    if !result.status.success() {
        let msg = format!(
            "BIOS NVRAM 写入失败，退出码：{:?}",
            result.status.code()
        );
        eprintln!("{msg}");
        return Err(msg);
    }

    println!("BIOS NVRAM 写入命令执行完成。");

    Ok(())
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

    // 启动最小化
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

    // 托盘菜单
    let show = MenuItemBuilder::with_id("show", "显示窗口").build(app)?;
    let benchmark_on = MenuItemBuilder::with_id("benchmark_on", "开启基准模式").build(app)?;
    let benchmark_off = MenuItemBuilder::with_id("benchmark_off", "关闭基准模式").build(app)?;
    let performance = MenuItemBuilder::with_id("performance", "性能模式").build(app)?;
    let balanced = MenuItemBuilder::with_id("balanced", "平衡模式").build(app)?;
    let quiet = MenuItemBuilder::with_id("quiet", "省电模式").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;
    let menu = MenuBuilder::new(app)
        .item(&show)
        .item(&benchmark_on)
        .item(&benchmark_off)
        .item(&performance)
        .item(&balanced)
        .item(&quiet)
        .separator()
        .item(&quit)
        .build()?;

    const TRAY_ICON: tauri::image::Image<'_> = include_image!("icons/32x32.png");
    // 创建托盘
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

    // 窗口关闭 → 隐藏到托盘
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

    // 风扇自启
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
        let app_handle = app.handle().clone();
        if let Err(e) = acpi_ec_worker().call(move |ec| {
            if ec.fan_read_custom_table_1() && ec.fan_read_custom_table_2() {
                println!("启用新EC风扇");
                thread::sleep(Duration::from_millis(3000));
                start_newfan(&app_handle, fan_data);
            } else {
                let state = app_handle.state::<FanControlState>();
                if let Err(e) = start_fan_control_internal(&app_handle, fan_data, &state) {
                    eprintln!("自动启动风扇控制失败: {}", e);
                };
            }
        }) {
            eprintln!("风扇自启 失败: {}", e);
        };
    } else {
        println!("未检测到 --fan-control，风扇控制默认关闭");
    }

    window_push::start_window_push_monitor(
        app.handle().clone(),
    );
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
    logging::init_log().expect("初始化日志系统失败");
    logging::install_panic_hook();
    println!("======================================");
    println!("       NUCtool PANIC HOOK");
    println!("======================================");
    // Fn Hook
    thread::spawn(|| loop {
        fnhook();
    });
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init()) // opener plugin
        // .plugin(tauri_plugin_autostart::Builder::new().build())
        .manage(AppState {
            running: Arc::new(AtomicBool::new(false)),
        })
        .manage(NewFanMonitorState::default())
        .manage(FanControlState::new())
        .manage(WindowPushState::new())
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
            osd_ready,
            show_osd_command,
            set_fan_mode_file,
            get_fan_mode_file,
            get_lightbar_profile,
            set_lightbar_profile,
            get_sys_model,
            get_sys_arc_gpu_driver,
            get_sys_gsc_driver,
            get_battery_charging_level,
            set_battery_charging_level,
            get_battery_mode,
            set_battery_mode,
            set_keyboard_enabled,
            set_keyboard_brightness,
            set_keyboard_rainbow,
            set_keyboard_red,
            set_keyboard_green,
            set_keyboard_blue,
            start_newfan_monitor,
            start_newfan_write,
            stop_newfan_write,
            set_fan_max,
            read_bios_nvram,
            export_nvram,
            write_bios_nvram,
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
