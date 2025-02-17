use std::fs;
use std::path::{Path, PathBuf};
use std::process::{exit, Command, Output};
use colored::Colorize;
use crate::{
    plug::struct_set::{
        Tdp, KERNEL_ID
    }
};

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

pub fn get_model_id() -> i64 {
        let output = Command::new("dmidecode")
        .arg("-t")
        .arg("system")
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        let dmidecode_output = String::from_utf8_lossy(&output.stdout);
        // 在输出中查找包含 "Product Name" 的行
        for line in dmidecode_output.lines() {
            if line.contains("Product Name"){
                if let Some(line) =  line.to_string().split(" ").last() {
                    println!("Product Name: {}", line);
                    if line == "LAPAC71H" {
                        return 1
                    } else {
                        return 0
                    }
                }
            }
        }
        eprintln!("Product Name not found.")
    } else {
        eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr))
    }
    0
}

pub fn get_kernel_version() -> i64 {
    let output = Command::new("uname")
        .arg("-r")
        .output()
        .expect("Failed to execute command");
    if output.status.success() {
        let kernel_version = String::from_utf8_lossy(&output.stdout);
        let kernel_version = kernel_version.rsplit("-").last().unwrap().split(".").collect::<Vec<&str>>();
        println!("Linux Kernel Version: {:?}", kernel_version);
        return if kernel_version[0].parse::<i64>().unwrap() == 6 {
            match kernel_version[1] {
                "13" => 2,
                "12" => 3,
                "11" => 2,
                "10" => 1,
                _ => 0
            }
        } else {
            0
        }
    }
    0
}

pub fn sys_init() {
    if *KERNEL_ID == 0 {
        println!("{}", "内核版本不支持".red());
        exit(0)
    }
    let output = Command::new("lsmod")
        .output()
        .expect("Failed to execute command");
    if output.status.success() {
        let lsmod_output = String::from_utf8_lossy(&output.stdout);
        if lsmod_output.contains("uniwill_laptop") {
            println!("{}", "模块已加载".green());
            return;
        } else {
            println!("{}", "模块未加载".red());
        }
    } 
    let model_path = "/root/.config/nuc_model";
    if Path::new(model_path).is_dir() {
        println!("{}", "模块存在跳过".blue());
    } else {
        println!("{}", "获取模块...".green());
        let out: Output;
        if *KERNEL_ID >= 2 {
            out = Command::new("git")
                .args(["clone", "https://github.com/cyear/uniwill-laptop"])
                .args(["--branch", "kernel-6.13"])
                .arg(model_path)
                .output().unwrap();
        } else {
            out = Command::new("git")
                .args(["clone", "https://github.com/cyear/uniwill-laptop"])
                .arg(model_path)
                .output().unwrap();
        }
        if out.status.success() {
            println!("{}", "获取模块成功".green())
        } else {
            println!("{}", "获取模块失败".red());
            println!("{:?}", String::from_utf8_lossy(&out.stderr));
            exit(0)
        }
    }
    // sudo insmod /home/nian/data/uniwill-laptop/uniwill-laptop/uniwill-wmi.ko
    // sudo insmod /home/nian/data/uniwill-laptop/uniwill-laptop/uniwill-laptop.ko;
    let m1 = "/root/.config/nuc_model/uniwill-laptop.ko";
    let m2 = "/root/.config/nuc_model/uniwill-wmi.ko";
    if Path::new(m1).exists() || Path::new(m2).exists() {
        println!("{}", "模块存在".green());
    } else {
        println!("{}", "模块不存在".red());
        let out = Command::new("make")
            .args(["--directory", model_path])
            .output().unwrap();
        if out.status.success() {
            println!("{}", "生成模块成功".green());
        } else {
            println!("{}: {}", "生成模块错误".red(), String::from_utf8_lossy(&out.stderr));
            exit(0);
        }
    }
    let out2 = Command::new("insmod")
        .arg(m2).output().unwrap();
    let out1 = Command::new("insmod")
        .arg(m1).output().unwrap();
    if out1.status.success() && out2.status.success() {
        println!("{}", "加载模块成功".green());
    } else { 
        println!("{}: {} {}", "加载模块失败".red(), String::from_utf8_lossy(&out1.stderr), String::from_utf8_lossy(&out2.stderr));
        exit(0);
    }
}