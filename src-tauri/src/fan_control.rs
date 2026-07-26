//! 风扇曲线控制循环
//!
//! 相对旧版 `plug/fan.rs` 的修复:
//!
//! - 曲线按温度排序并去重 — 旧版温度重复的两个点会触发**除零 panic**
//! - 温度超出曲线末端时保持末点转速 — 旧版会回落为 0%(高温时风扇停转!)
//! - 温度读取异常时增加休眠 — 旧版会空转打满一个 CPU 核心
//! - 停止后由控制线程自身复位风扇 — 旧版用一个分离的 2 秒定时线程,
//!   与"立即重新启动控制"存在竞态
//! - 配置改为强类型 `FanData` — 旧版对 `serde_json::Value` 的字段直接
//!   `unwrap()`, 异常配置会导致崩溃

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use colored::Colorize;

use crate::config::{FanData, FanPoint};
use crate::hw::hw;

/// 风扇控制运行状态(由 tauri 管理)
#[derive(Default)]
pub struct FanControlState {
    pub running: Arc<AtomicBool>,
}

/// 已排序、去重的风扇曲线: (温度, 速度%)
struct Curve(Vec<(i64, i64)>);

impl Curve {
    fn new(points: &[FanPoint]) -> Option<Self> {
        let mut v: Vec<(i64, i64)> = points
            .iter()
            .map(|p| {
                (
                    p.temperature.round() as i64,
                    (p.speed.round() as i64).clamp(0, 100),
                )
            })
            .collect();
        v.sort_by_key(|p| p.0);
        v.dedup_by_key(|p| p.0); // 温度重复会导致插值除零
        if v.is_empty() {
            None
        } else {
            Some(Curve(v))
        }
    }

    /// 分段线性插值:
    /// 低于首点时从 (0, 0) 过渡; 高于末点时保持末点速度
    fn speed_at(&self, temp: i64) -> i64 {
        let (mut t0, mut s0) = (0i64, 0i64);
        for &(t, s) in &self.0 {
            if t >= temp {
                if t == t0 {
                    return s;
                }
                return s0 + (s - s0) * (temp - t0) / (t - t0);
            }
            (t0, s0) = (t, s);
        }
        s0
    }
}

/// 启动风扇控制线程
pub fn start(data: &FanData, running: Arc<AtomicBool>) -> Result<(), String> {
    let left = Curve::new(&data.left_fan).ok_or("左风扇曲线为空")?;
    let right = Curve::new(&data.right_fan).ok_or("右风扇曲线为空")?;
    if running.swap(true, Ordering::SeqCst) {
        println!("风扇控制已在运行");
        return Ok(());
    }
    thread::Builder::new()
        .name("fan-control".into())
        .spawn(move || {
            println!("{}", "风扇控制启动".green());
            let mut cache = [-1i64; 2];
            while running.load(Ordering::SeqCst) {
                println!("---------------------------------------------------------------");
                tick(&left, &right, &mut cache);
            }
            // 停止后由控制线程自身复位, 与重新启动之间无竞态
            hw().fan_auto_blocking();
            println!("{}", "风扇控制停止, 已恢复自动".green());
        })
        .map_err(|e| format!("控制线程启动失败: {e}"))?;
    Ok(())
}

/// 请求停止风扇控制(控制线程将在当前周期结束后复位风扇并退出)
pub fn stop(running: &Arc<AtomicBool>) {
    if !running.swap(false, Ordering::SeqCst) {
        // 本就未运行: 仍确保风扇回到自动(与旧版行为一致)
        hw().fan_auto();
    }
    println!("{}", "风扇控制停止请求已发送".green());
}

/// 单个控制周期
fn tick(left: &Curve, right: &Curve, cache: &mut [i64; 2]) {
    let (cpu, gpu) = hw().temps();
    println!("CPU: {cpu}°C, GPU: {gpu}°C, 缓存: {cache:?}");

    // 高温保护: 满速
    if cpu > 95 || gpu > 95 {
        if *cache == [100, 100] {
            println!("{}", "已满速, 跳过重复写入".red());
        } else {
            set_percent(100, 100);
            *cache = [100, 100];
        }
        thread::sleep(Duration::from_secs(4));
        return;
    }

    // 读取异常(负值): 跳过本轮(旧版此处不休眠, 会空转打满 CPU)
    if cpu < 0 || gpu < 0 {
        println!("温度读取异常: cpu={cpu}, gpu={gpu}");
        thread::sleep(Duration::from_secs(2));
        return;
    }

    // 风扇被 EC 切回自动(异常): 按旧版时序重新接管
    if hw().fan_mode() == 2 {
        println!("{}", "风扇异常, 自动恢复中...".red());
        thread::sleep(Duration::from_secs_f64(1.5));
        hw().fan_auto_blocking();
        thread::sleep(Duration::from_secs_f64(2.5));
        println!("重新接管: {}", hw().fan_manual());
        return;
    }

    let (l, r) = (left.speed_at(cpu), right.speed_at(gpu));
    if *cache == [l, r] {
        println!("{}", "风扇速度未变化".green());
    } else {
        set_percent(l, r);
        *cache = [l, r];
    }
    thread::sleep(Duration::from_secs(3));
}

/// 按百分比写入风扇转速(EC 原始值为 0-200)
fn set_percent(l: i64, r: i64) {
    let ok = hw().set_fan(l * 2, r * 2);
    println!("FAN_L: {l}% / FAN_R: {r}% 写入: {ok}");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn curve(points: &[(f64, f64)]) -> Curve {
        let pts: Vec<FanPoint> = points
            .iter()
            .map(|&(temperature, speed)| FanPoint { temperature, speed })
            .collect();
        Curve::new(&pts).unwrap()
    }

    #[test]
    fn interpolation_basic() {
        let c = curve(&[(30.0, 20.0), (50.0, 40.0), (70.0, 80.0)]);
        assert_eq!(c.speed_at(30), 20);
        assert_eq!(c.speed_at(40), 30); // 30-50 中点
        assert_eq!(c.speed_at(50), 40);
        assert_eq!(c.speed_at(60), 60); // 50-70 中点
        assert_eq!(c.speed_at(70), 80);
    }

    #[test]
    fn below_first_point_ramps_from_zero() {
        let c = curve(&[(30.0, 40.0)]);
        assert_eq!(c.speed_at(15), 20); // (0,0) 到 (30,40) 的中点
    }

    #[test]
    fn above_last_point_keeps_last_speed() {
        // 旧版会回落为 0, 高温风扇停转
        let c = curve(&[(30.0, 20.0), (60.0, 70.0)]);
        assert_eq!(c.speed_at(90), 70);
    }

    #[test]
    fn duplicate_temperature_no_panic() {
        // 旧版在此场景除零 panic
        let c = curve(&[(50.0, 30.0), (50.0, 60.0), (70.0, 80.0)]);
        let _ = c.speed_at(50);
        let _ = c.speed_at(55);
    }

    #[test]
    fn unsorted_input_is_sorted() {
        let c = curve(&[(70.0, 80.0), (30.0, 20.0), (50.0, 40.0)]);
        assert_eq!(c.speed_at(40), 30);
    }

    #[test]
    fn speed_clamped_to_100() {
        let c = curve(&[(30.0, 250.0)]);
        assert_eq!(c.speed_at(30), 100);
    }
}
