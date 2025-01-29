#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use powershell_script;
use powershell_script::PsScriptBuilder;
use std::{
    env, fs, process,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};
use lazy_static::lazy_static;
use notify_rust::Notification;
use tauri::{self, State};
use windows::Win32::System::Wmi::IWbemClassObject;

mod modules;
use modules::{
    config::{get_config_dir, get_config_file_path},
    setup,
    struct_set::{ChannelControlState, FanControlState, FanData, FanSpeeds, Tdp},
    wmi::{get_model, wmi_init, wmi_security, wmi_set},
};

static R_TDP_GPU1: &str = "0x000001000000073d";
static R_TDP_GPU2: &str = "0x0000010000000733";
static R_TDP_CPU1: &str = "0x0000010000000783";
static R_TDP_CPU2: &str = "0x0000010000000784";
static R_TDP_TCC: &str = "0x0000010000000786";

static R_FAN_L1: &str = "0x000001000000046C";
static R_FAN_L2: &str = "0x000001000000046D";
static R_FAN_R1: &str = "0x0000010000000464";
static R_FAN_R2: &str = "0x0000010000000465";
static R_TEMP_L: &str = "0x000001000000043E";
static R_TEMP_R: &str = "0x000001000000044F";

lazy_static! {
    static ref MODEL_ID: i64 = get_model();
}

struct WmiState {
    pub wmi: Arc<Mutex<IWbemClassObject>>,
}

#[tauri::command]
async fn save_fan_config(fan_data: FanData) -> Result<(), String> {
    // 获取配置文件路径
    let config_path = get_config_file_path()?;

    // 将 fan_data 序列化为 JSON
    let json_data = serde_json::to_string_pretty(&fan_data).map_err(|e| e.to_string())?;

    // 写入配置文件
    fs::write(config_path, json_data).map_err(|e| e.to_string())?;

    println!("风扇配置已保存");
    Ok(())
}
#[tauri::command]
async fn load_fan_config() -> FanData {
    // 获取配置文件路径
    let config_path = get_config_file_path().unwrap();
    // 检查配置文件是否存在
    if !config_path.exists() {
        println!("配置文件不存在");
    }
    // 读取配置文件
    let json_data = fs::read_to_string(config_path)
        .map_err(|e| e.to_string())
        .unwrap();
    // 反序列化为 FanData
    let fan_data: FanData = serde_json::from_str(&json_data)
        .map_err(|e| e.to_string())
        .unwrap();
    println!("风扇配置已加载");
    fan_data
}

#[tauri::command]
async fn get_tdp() -> (i64, i64, i64, i64, i64) {
    let (in_cls, svc, obj_path, method_name) = wmi_init();
    let gpu1 = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_TDP_GPU1) & 0xFF;
    let gpu2 = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_TDP_GPU2) & 0xFF;
    let cpu1 = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_TDP_CPU1) & 0xFF;
    let cpu2 = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_TDP_CPU2);
    let tcc = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_TDP_TCC);
    (cpu1, cpu2, gpu1, gpu2, tcc)
}

#[tauri::command]
async fn set_tdp(t: Tdp) {
    let (in_cls, svc, obj_path, method_name) = wmi_init();
    let _gpu1 = wmi_set(
        &in_cls,
        &svc,
        &obj_path,
        &method_name,
        format!("0x000000000{:02x}073d", t.gpu1).as_str(),
    );
    let _gpu2 = wmi_set(
        &in_cls,
        &svc,
        &obj_path,
        &method_name,
        format!("0x000000000{:02x}0733", t.gpu2).as_str(),
    );
    let _cpu1 = wmi_set(
        &in_cls,
        &svc,
        &obj_path,
        &method_name,
        format!("0x000000000{:02x}0783", t.cpu1).as_str(),
    );
    let _cpu2 = wmi_set(
        &in_cls,
        &svc,
        &obj_path,
        &method_name,
        format!("0x000000000{:02x}0784", t.cpu2).as_str(),
    );
    let _tcc = wmi_set(
        &in_cls,
        &svc,
        &obj_path,
        &method_name,
        format!("0x000000000{:02x}0786", t.tcc).as_str(),
    );
    Notification::new()
        .summary("NUC X15 Fan Control")
        .body("TDP设置成功")
        .icon("firefox")
        .show()
        .unwrap();
}

#[tauri::command]
async fn get_fan_speeds() -> FanSpeeds {
    let (in_cls, svc, obj_path, method_name) = wmi_init();
    let l_fan_1 = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_FAN_L1);
    let l_fan_2 = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_FAN_L2);
    let r_fan_1 = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_FAN_R1);
    let r_fan_2 = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_FAN_R2);
    let l_temp = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_TEMP_L);
    let r_temp = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_TEMP_R);
    if *MODEL_ID == 1 {
        FanSpeeds {
            left_fan_speed: (l_fan_1 & 0xFF) << 8 | l_fan_2,
            right_fan_speed: (r_fan_1 & 0xFF) << 8 | r_fan_2,
            left_temp: l_temp,
            right_temp: r_temp & 0xFF,
        }
    } else {
        FanSpeeds {
            left_fan_speed: (r_fan_1 & 0xFF) << 8 | r_fan_2,
            right_fan_speed: (l_fan_1 & 0xFF) << 8 | l_fan_2,
            left_temp: l_temp,
            right_temp: r_temp & 0xFF,
        }
    }
}
#[tauri::command]
fn start_fan_control(
    fan_data: serde_json::Value,
    state: State<FanControlState>,
    tx: State<ChannelControlState>,
) {
    Arc::clone(&tx.tx)
        .lock()
        .unwrap()
        .send("0x000001000000044F".to_string())
        .unwrap();
    let is_running = Arc::clone(&state.is_running);
    Notification::new()
        .summary("NUC X15 Fan Control")
        .body("正在运行")
        .icon("firefox")
        .show()
        .unwrap();
    // 打印接收到的风扇数据
    // println!("left fan data: {:?}", fan_data.get("left_fan"));
    // println!("right fan data: {:?}", fan_data.get("right_fan"));
    println!("接受风扇配置信息");
    fan_init();
    // 如果已经在运行，跳过启动
    if *is_running.lock().unwrap() {
        println!("Fan control is already running.");
        return;
    }
    // 启动新的控制线程
    *is_running.lock().unwrap() = true;
    thread::spawn(move || {
        while *is_running.lock().unwrap() {
            // 模拟执行时间
            // println!("Fan control loop running...");
            cpu_temp(&fan_data.get("left_fan"), &fan_data.get("right_fan"));
            thread::sleep(Duration::from_secs(1));
            // println!("TEMP: {}", cpu_temp());
        }
        println!("Fan control stopped.");
    });
}

#[tauri::command]
fn stop_fan_control(state: State<FanControlState>) {
    let mut is_running = state.is_running.lock().unwrap();
    // fan_init();
    *is_running = false; // 停止风扇控制
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(1));
        fan_reset();
        Notification::new()
            .summary("NUC X15 Fan Control")
            .body("停止运行")
            .icon("firefox")
            .show()
            .unwrap();
    });
}

fn speed_c(speed_n: i64, speed_l: i64, temp_n: i64, temp_l: i64, temp: i64) -> i64 {
    println!("{} {} {} {} {}", speed_n, speed_l, temp_n, temp_l, temp);
    speed_l
        + (((speed_n - speed_l) as f64 / ((temp_n - temp_l) as f64 + 0.001))
            * (temp - temp_l) as f64) as i64
}

fn cpu_temp(left: &Option<&serde_json::Value>, right: &Option<&serde_json::Value>) {
    let (in_cls, svc, obj_path, method_name) = wmi_init();
    let cpu_out = wmi_set(
        &in_cls,
        &svc,
        &obj_path,
        &method_name,
        R_TEMP_L.to_string().as_str(),
    );
    let gpu_out = wmi_set(
        &in_cls,
        &svc,
        &obj_path,
        &method_name,
        R_TEMP_R.to_string().as_str(),
    ) & 0xFF;
    println!("CPU Temp: {:?}, GPU Temp: {:?}", &cpu_out, &gpu_out);
    if cpu_out > 95 || gpu_out > 95 {
        fan_set(100, 100);
        return;
    }
    let (mut l_c, mut s_c, mut r_c, mut s_c_) = (0i64, 0i64, 0i64, 0i64);
    if let (Some(left), Some(right)) = (left.expect("l").as_array(), right.expect("r").as_array()) {
        for l_ in left {
            if let (Some(l), Some(s)) = (
                l_.get("temperature").expect("转换错误").as_i64(),
                l_.get("speed").expect("转换错误").as_i64(),
            ) {
                if l >= cpu_out {
                    for r_ in right {
                        if let (Some(r), Some(s_)) = (
                            r_.get("temperature").expect("转换错误").as_i64(),
                            r_.get("speed").expect("转换错误").as_i64(),
                        ) {
                            if r >= gpu_out {
                                let s = speed_c(s, s_c, l, l_c, cpu_out);
                                let s_ = speed_c(s_, s_c_, r, r_c, gpu_out);
                                fan_set(s as i16, s_ as i16);
                                println!(
                                    "cpu_t: {:?} l_fan: {:?} gpu_t: {:?} r_fan: {:?}",
                                    cpu_out, s, gpu_out, s_
                                );
                                return;
                            } else {
                                (l_c, s_c, r_c, s_c_) = (l, s, r, s_);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn fan_init() {
    let (in_cls, svc, obj_path, method_name) = wmi_init();
    let out = wmi_set(
        &in_cls,
        &svc,
        &obj_path,
        &method_name,
        "0x0000010000000751".to_string().as_str(),
    );
    if out - 27728 == -64 {
        let _ = wmi_set(
            &in_cls,
            &svc,
            &obj_path,
            &method_name,
            "0x0000000000400751".to_string().as_str(),
        );
    }
}

fn fan_reset() {
    let (in_cls, svc, obj_path, method_name) = wmi_init();
    let _ = wmi_set(
        &in_cls,
        &svc,
        &obj_path,
        &method_name,
        "0x0000000000A00751".to_string().as_str(),
    );
}

fn fan_set(left: i16, right: i16) {
    let (in_cls, svc, obj_path, method_name) = wmi_init();
    let out = wmi_set(
        &in_cls,
        &svc,
        &obj_path,
        &method_name,
        "0x0000010000000751".to_string().as_str(),
    );
    if out - 27728 == -64 && out - 27728 == -80 {
        fan_init();
        // println!("风扇状态异常已尝试恢复");
        Notification::new()
            .summary("NUC X15 Fan Control")
            .body("风扇状态异常已尝试恢复")
            .icon("firefox")
            .show()
            .unwrap();
    }
    let left = left * 2;
    let right = right * 2;
    println!("{} {}", left, right);
    if *MODEL_ID == 1 {
        let _ = wmi_set(
            &in_cls,
            &svc,
            &obj_path,
            &method_name,
            format!("0x000000000{:02x}1809", left).as_str(),
        );
        let _ = wmi_set(
            &in_cls,
            &svc,
            &obj_path,
            &method_name,
            format!("0x000000000{:02x}1804", right).as_str(),
        );
    } else {
        let _ = wmi_set(
            &in_cls,
            &svc,
            &obj_path,
            &method_name,
            format!("0x000000000{:02x}1809", right).as_str(),
        );
        let _ = wmi_set(
            &in_cls,
            &svc,
            &obj_path,
            &method_name,
            format!("0x000000000{:02x}1804", left).as_str(),
        );
    }
}

fn main() {
    let ps = PsScriptBuilder::new()
        .no_profile(true)
        .non_interactive(true)
        .hidden(true)
        .print_commands(false)
        .build();
    let s = r#"if (([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { Write-Output "1" } else { Write-Output "0" }"#;
    let out = ps.run(s).unwrap();
    println!("{:?}", &out);
    if out.to_string().trim().parse::<i64>().unwrap() == 0 {
        if let Ok(path) = env::current_exe() {
            let path_str = path.to_string_lossy();
            println!("当前程序的路径是: {}", &path_str);
            ps.run(format!(r#"Start-Process "{}" -Verb RunAs"#, path_str).as_str())
                .unwrap();
            process::exit(0);
        }
    };
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
