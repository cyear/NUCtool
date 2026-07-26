//! 硬件访问入口: 所有 WMI/EC 操作经由**单一工作线程**串行执行
//!
//! 旧版在多个线程各自反复 `ApiFan::init()` 创建 WMI 连接并发调用,
//! 是"数据偶发巨大负数 / 风扇异常反复恢复"等问题的主要嫌疑。
//! 现在整个进程只建立一条 COM/WMI 连接, 全部请求排队执行:
//!
//! - 彻底消除并发 `ExecMethod` 交错
//! - COM 初始化时序确定(在工作线程内完成, 不再与其它线程竞争)
//! - 连接失败时进入降级模式(返回哨兵值), UI 仍可打开而不是直接闪退

mod device;
mod registers;
mod wmi;

use std::sync::mpsc::{self, Sender};
use std::sync::OnceLock;
use std::thread;

use colored::Colorize;
use serde::{Deserialize, Serialize};

use device::Device;

/// 风扇转速与温度(推送给前端图表)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FanSpeeds {
    pub left_fan_speed: i64,
    pub right_fan_speed: i64,
    pub left_temp: i64,
    pub right_temp: i64,
}

/// TDP 配置(与前端 `set_tdp` 参数结构一致)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tdp {
    pub cpu1: i64,
    pub cpu2: i64,
    pub gpu1: i64,
    pub gpu2: i64,
    pub tcc: i64,
}

enum Request {
    FanSpeeds(Sender<FanSpeeds>),
    Temps(Sender<(i64, i64)>),
    FanMode(Sender<i64>),
    SetFan {
        left: i64,
        right: i64,
        done: Sender<bool>,
    },
    FanAuto(Option<Sender<bool>>),
    FanManual(Sender<bool>),
    GetTdp(Sender<Tdp>),
    SetTdp(Tdp, Sender<bool>),
    SetLedAc {
        on: bool,
        done: Sender<bool>,
    },
    LedAcState(Sender<i64>),
}

/// 硬件句柄(可跨线程共享, 内部经通道与工作线程通信)
pub struct Hw {
    tx: Sender<Request>,
}

/// 全局硬件句柄, 首次访问时启动工作线程
pub fn hw() -> &'static Hw {
    static HW: OnceLock<Hw> = OnceLock::new();
    HW.get_or_init(Hw::spawn)
}

impl Hw {
    fn spawn() -> Self {
        let (tx, rx) = mpsc::channel::<Request>();
        thread::Builder::new()
            .name("hw-worker".into())
            .spawn(move || {
                if let Err(e) = wmi::init_com() {
                    println!("{} {e:#}", "COM 初始化失败:".red());
                }
                let model = wmi::query_model().unwrap_or_else(|e| {
                    println!("{} {e:#}", "机型查询失败:".red());
                    String::new()
                });
                let device = match wmi::WmiSession::connect() {
                    Ok(s) => Some(Device::new(s, &model)),
                    Err(e) => {
                        println!("{} {e:#}", "WMI 会话建立失败(硬件控制不可用):".red());
                        None
                    }
                };
                for req in rx {
                    match &device {
                        Some(d) => Self::handle(d, req),
                        None => Self::degraded(req),
                    }
                }
            })
            .expect("硬件工作线程启动失败");
        Hw { tx }
    }

    fn handle(d: &Device, req: Request) {
        match req {
            Request::FanSpeeds(tx) => {
                let _ = tx.send(d.fan_speeds());
            }
            Request::Temps(tx) => {
                let _ = tx.send(d.temps());
            }
            Request::FanMode(tx) => {
                let _ = tx.send(d.fan_mode());
            }
            Request::SetFan { left, right, done } => {
                let _ = done.send(d.set_fan_raw(left, right));
            }
            Request::FanAuto(done) => {
                let ok = d.set_fan_auto();
                println!("{}", "风扇已恢复自动".red());
                if let Some(tx) = done {
                    let _ = tx.send(ok);
                }
            }
            Request::FanManual(done) => {
                let _ = done.send(d.set_fan_manual());
            }
            Request::GetTdp(tx) => {
                let _ = tx.send(d.tdp());
            }
            Request::SetTdp(t, tx) => {
                let _ = tx.send(d.set_tdp(&t));
            }
            Request::SetLedAc { on, done } => {
                let _ = done.send(d.set_led_ac(on));
            }
            Request::LedAcState(tx) => {
                let _ = tx.send(d.led_ac_state());
            }
        }
    }

    /// 无硬件连接时的兜底应答, 保证 UI 可用
    fn degraded(req: Request) {
        match req {
            Request::FanSpeeds(tx) => {
                let _ = tx.send(FanSpeeds {
                    left_fan_speed: -1,
                    right_fan_speed: -1,
                    left_temp: -1,
                    right_temp: -1,
                });
            }
            Request::Temps(tx) => {
                let _ = tx.send((-1, -1));
            }
            Request::FanMode(tx) => {
                let _ = tx.send(1);
            }
            Request::SetFan { done, .. } => {
                let _ = done.send(false);
            }
            Request::FanAuto(done) => {
                if let Some(tx) = done {
                    let _ = tx.send(false);
                }
            }
            Request::FanManual(done) => {
                let _ = done.send(false);
            }
            Request::GetTdp(tx) => {
                let _ = tx.send(Tdp {
                    cpu1: -1,
                    cpu2: -1,
                    gpu1: -1,
                    gpu2: -1,
                    tcc: -1,
                });
            }
            Request::SetTdp(_, tx) => {
                let _ = tx.send(false);
            }
            Request::SetLedAc { done, .. } => {
                let _ = done.send(false);
            }
            Request::LedAcState(tx) => {
                let _ = tx.send(0);
            }
        }
    }

    /// 发送请求并等待应答; 工作线程不可用时返回兜底值
    fn request<T>(&self, make: impl FnOnce(Sender<T>) -> Request, fallback: T) -> T {
        let (tx, rx) = mpsc::channel();
        if self.tx.send(make(tx)).is_err() {
            return fallback;
        }
        rx.recv().unwrap_or(fallback)
    }

    pub fn fan_speeds(&self) -> FanSpeeds {
        self.request(
            Request::FanSpeeds,
            FanSpeeds {
                left_fan_speed: -1,
                right_fan_speed: -1,
                left_temp: -1,
                right_temp: -1,
            },
        )
    }

    /// (CPU 温度, GPU 温度)
    pub fn temps(&self) -> (i64, i64) {
        self.request(Request::Temps, (-1, -1))
    }

    /// 1 - 受控模式, 2 - 自动模式
    pub fn fan_mode(&self) -> i64 {
        self.request(Request::FanMode, 1)
    }

    /// 设置风扇转速(原始值 0-200)
    pub fn set_fan(&self, left: i64, right: i64) -> bool {
        self.request(|done| Request::SetFan { left, right, done }, false)
    }

    /// 风扇恢复自动(不等待完成 — 启动时用)
    pub fn fan_auto(&self) {
        let _ = self.tx.send(Request::FanAuto(None));
    }

    /// 风扇恢复自动(阻塞等待完成 — 退出/停止控制时用)
    pub fn fan_auto_blocking(&self) -> bool {
        self.request(|done| Request::FanAuto(Some(done)), false)
    }

    /// 接管风扇
    pub fn fan_manual(&self) -> bool {
        self.request(Request::FanManual, false)
    }

    pub fn get_tdp(&self) -> Tdp {
        self.request(
            Request::GetTdp,
            Tdp {
                cpu1: -1,
                cpu2: -1,
                gpu1: -1,
                gpu2: -1,
                tcc: -1,
            },
        )
    }

    pub fn set_tdp(&self, t: Tdp) -> bool {
        self.request(|done| Request::SetTdp(t, done), false)
    }

    pub fn set_led_ac(&self, on: bool) -> bool {
        self.request(|done| Request::SetLedAc { on, done }, false)
    }

    /// 0 - 异常, 1 - 关, 2 - 开
    pub fn led_ac_state(&self) -> i64 {
        self.request(Request::LedAcState, 0)
    }
}
