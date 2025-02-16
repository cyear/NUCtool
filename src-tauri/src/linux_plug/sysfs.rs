use std::fs;
use std::path::PathBuf;
use crate::plug::struct_set::Tdp;

#[tauri::command]
pub async fn get_tdp() -> (i64, i64, i64, i64, i64) {
    (0, 0 ,0, 0, 0)
}

#[tauri::command]
pub async fn set_tdp(t: Tdp) {}

pub fn get_sys(driver: &PathBuf) -> i64 {
    match fs::read_to_string(driver) {
        Ok(content) => content.trim().parse::<i64>().unwrap_or(0),
        Err(_) => 0
    }
}

pub fn set_sys(driver: &PathBuf, n: i64) -> bool {
    let content = n.to_string();
    match fs::write(driver, content) {
        Ok(_) => true,
        Err(_) => false,
    }
}