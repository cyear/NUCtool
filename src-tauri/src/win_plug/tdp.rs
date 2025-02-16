use crate::win_plug::{
    struct_set::{Tdp, R_TDP_CPU1, R_TDP_CPU2, R_TDP_GPU1, R_TDP_GPU2, R_TDP_TCC},
    wmi::{wmi_init, wmi_set},
};
use notify_rust::Notification;

/**
* @Author: cyear
* @Create time: 2025-01-30
* @Description:
* @Version: 0.3.5
**/

#[tauri::command]
pub async fn get_tdp() -> (i64, i64, i64, i64, i64) {
    let (in_cls, svc, obj_path, method_name) = wmi_init();
    let gpu1 = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_TDP_GPU1) & 0xFF;
    let gpu2 = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_TDP_GPU2) & 0xFF;
    let cpu1 = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_TDP_CPU1) & 0xFF;
    let cpu2 = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_TDP_CPU2);
    let tcc = wmi_set(&in_cls, &svc, &obj_path, &method_name, R_TDP_TCC);
    (cpu1, cpu2, gpu1, gpu2, tcc)
}

#[tauri::command]
pub async fn set_tdp(t: Tdp) {
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
