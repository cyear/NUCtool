//! 设备操作层(旧版 `struct_set.rs` 中 `ApiFan` 的替代)
//!
//! 只在硬件工作线程内部使用; 所有读取失败返回 -1,
//! 上层(风扇控制/前端图表)已把负值当作异常处理。

use std::{thread, time::Duration};

use colored::Colorize;

use super::registers as reg;
use super::wmi::WmiSession;
use super::{FanSpeeds, Tdp};

pub struct Device {
    wmi: WmiSession,
    /// LAPAC71H 为 true; LAPKC71F 及其它机型为 false(左右风扇寄存器相反)
    model_ac71h: bool,
}

impl Device {
    pub fn new(wmi: WmiSession, model: &str) -> Self {
        let model = model.trim();
        let model_ac71h = model == "LAPAC71H";
        println!(
            "机型: {} ({})",
            model.blue(),
            if model_ac71h {
                "LAPAC71H 布局"
            } else {
                "LAPKC71F/通用布局"
            }
        );
        Device { wmi, model_ac71h }
    }

    /// 读寄存器, 失败返回 -1
    fn read(&self, r: u64) -> i64 {
        match self.wmi.exec(reg::read(r)) {
            Ok(v) => v,
            Err(e) => {
                println!("{} {e:#}", "WMI 读取失败:".red());
                -1
            }
        }
    }

    /// 写寄存器
    fn write(&self, r: u64, v: u64) -> bool {
        match self.wmi.exec(reg::write(r, v)) {
            Ok(_) => true,
            Err(e) => {
                println!("{} {e:#}", "WMI 写入失败:".red());
                false
            }
        }
    }

    pub fn cpu_temp(&self) -> i64 {
        self.read(reg::REG_TEMP_CPU)
    }

    pub fn gpu_temp(&self) -> i64 {
        let v = self.read(reg::REG_TEMP_GPU);
        if v < 0 {
            v
        } else {
            v & 0xFF
        }
    }

    /// (左1, 左2, 右1, 右2) 转速寄存器 — KC71F 机型左右相反
    fn fan_regs(&self) -> (u64, u64, u64, u64) {
        if self.model_ac71h {
            (
                reg::REG_FAN_L1,
                reg::REG_FAN_L2,
                reg::REG_FAN_R1,
                reg::REG_FAN_R2,
            )
        } else {
            (
                reg::REG_FAN_R1,
                reg::REG_FAN_R2,
                reg::REG_FAN_L1,
                reg::REG_FAN_L2,
            )
        }
    }

    /// 高低字节拼接转速; 任一字节读取失败返回 -1
    fn read_rpm(&self, hi: u64, lo: u64) -> i64 {
        let (h, l) = (self.read(hi), self.read(lo));
        if h < 0 || l < 0 {
            return -1;
        }
        ((h & 0xFF) << 8) | (l & 0xFF)
    }

    pub fn fan_speeds(&self) -> FanSpeeds {
        let (l1, l2, r1, r2) = self.fan_regs();
        FanSpeeds {
            left_fan_speed: self.read_rpm(l1, l2),
            right_fan_speed: self.read_rpm(r1, r2),
            left_temp: self.cpu_temp(),
            right_temp: self.gpu_temp(),
        }
    }

    pub fn temps(&self) -> (i64, i64) {
        (self.cpu_temp(), self.gpu_temp())
    }

    /// 设置风扇转速, 原始值 0-200(前端百分比的 2 倍)
    pub fn set_fan_raw(&self, left: i64, right: i64) -> bool {
        let (l, r) = if self.model_ac71h {
            (left, right)
        } else {
            (right, left)
        };
        // 注意用 `&` 而不是 `&&`: 即使左风扇写入失败也要尝试写右风扇
        self.write(reg::REG_FAN_SET_L, l.clamp(0, 200) as u64)
            & self.write(reg::REG_FAN_SET_R, r.clamp(0, 200) as u64)
    }

    /// 风扇恢复自动模式
    pub fn set_fan_auto(&self) -> bool {
        self.write(reg::REG_FAN_MODE, reg::FAN_AUTO)
    }

    /// 接管风扇(手动控制模式)
    pub fn set_fan_manual(&self) -> bool {
        let v = if self.model_ac71h {
            reg::FAN_TAKEOVER_AC71H
        } else {
            reg::FAN_TAKEOVER_KC71F
        };
        self.write(reg::REG_FAN_MODE, v)
    }

    /// 1 - 受控模式, 2 - 自动模式(读取异常按受控处理, 与旧版一致)
    pub fn fan_mode(&self) -> i64 {
        let out = self.read(reg::REG_FAN_MODE);
        println!("MODE: {out}");
        if out <= 0 {
            return 1;
        }
        // 0x6C10 / 0x6C00 表示自动模式
        if out == 0x6C10 || out == 0x6C00 {
            2
        } else {
            1
        }
    }

    /// 读取 TDP 配置
    pub fn tdp(&self) -> Tdp {
        let low = |v: i64| if v < 0 { v } else { v & 0xFF };
        Tdp {
            cpu1: low(self.read(reg::REG_TDP_CPU1)),
            cpu2: low(self.read(reg::REG_TDP_CPU2)),
            gpu1: low(self.read(reg::REG_TDP_GPU1)),
            gpu2: low(self.read(reg::REG_TDP_GPU2)),
            tcc: self.read(reg::REG_TDP_TCC),
        }
    }

    /// 写入 TDP 配置; 每次写入间隔 0.5s(EC 需要时间消化, 与旧版一致)
    pub fn set_tdp(&self, t: &Tdp) -> bool {
        let ops = [
            (reg::REG_TDP_CPU1, t.cpu1),
            (reg::REG_TDP_CPU2, t.cpu2),
            (reg::REG_TDP_GPU1, t.gpu1),
            (reg::REG_TDP_GPU2, t.gpu2),
            (reg::REG_TDP_TCC, t.tcc),
        ];
        let mut ok = true;
        for (i, (r, v)) in ops.iter().enumerate() {
            if i > 0 {
                thread::sleep(Duration::from_millis(500));
            }
            ok &= self.write(*r, (*v).clamp(0, 255) as u64);
        }
        ok
    }

    /// AC 供电键盘 LED 彩色模式开关
    pub fn set_led_ac(&self, on: bool) -> bool {
        self.write(reg::REG_LED_AC, if on { reg::LED_ON } else { reg::LED_OFF })
    }

    /// 0 - 异常, 1 - 关, 2 - 开
    pub fn led_ac_state(&self) -> i64 {
        let out = self.read(reg::REG_LED_AC);
        if out < 0 {
            return 0;
        }
        match out & 0xFF {
            2 | 4 => 1,
            34 | 36 => 2,
            _ => {
                println!("LED AC 状态异常: {out}");
                0
            }
        }
    }
}
