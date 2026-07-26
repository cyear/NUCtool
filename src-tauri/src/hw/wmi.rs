//! WMI/COM 底层封装
//!
//! 调用序列与旧版 `win_plug/wmi.rs` 保持一致, 区别在于:
//!
//! - 所有错误以 `Result` 返回, 不再 `unwrap()` 直接崩溃
//!   (release 配置为 `panic = "abort"`, 旧版任何一次 WMI 抖动都会闪退)
//! - 机型查询改用原生 COM (`ROOT\CIMV2`), 去掉了重量级的 `wmi` crate
//! - 只在硬件工作线程中使用, 整个进程仅建立一次连接

use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use windows::{
    core::{w, BSTR},
    Win32::System::{
        Com::{
            CoCreateInstance, CoInitializeEx, CoInitializeSecurity, CLSCTX_INPROC_SERVER,
            COINIT_MULTITHREADED, EOAC_NONE, RPC_C_AUTHN_LEVEL_DEFAULT,
            RPC_C_IMP_LEVEL_IMPERSONATE,
        },
        Variant::VARIANT,
        Wmi::{
            IWbemClassObject, IWbemLocator, IWbemServices, WbemLocator, WBEM_FLAG_FORWARD_ONLY,
            WBEM_FLAG_RETURN_ERROR_OBJECT, WBEM_FLAG_RETURN_IMMEDIATELY,
            WBEM_FLAG_RETURN_WBEM_COMPLETE, WBEM_INFINITE,
        },
    },
};

/// 在当前线程初始化 COM 与进程级安全设置。
/// 每个使用 COM 的线程调用一次; 本项目中仅硬件工作线程调用。
pub fn init_com() -> Result<()> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED)
            .ok()
            .context("CoInitializeEx 失败")?;
        // CoInitializeSecurity 进程级只能设置一次,
        // 若已被其它组件设置会返回 RPC_E_TOO_LATE, 忽略即可
        if let Err(e) = CoInitializeSecurity(
            None,
            -1,
            None,
            None,
            RPC_C_AUTHN_LEVEL_DEFAULT,
            RPC_C_IMP_LEVEL_IMPERSONATE,
            None,
            EOAC_NONE,
            None,
        ) {
            println!("CoInitializeSecurity: {e}");
        }
    }
    Ok(())
}

fn create_locator() -> Result<IWbemLocator> {
    unsafe {
        CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER).context("创建 WbemLocator 失败")
    }
}

fn connect(loc: &IWbemLocator, namespace: &str) -> Result<IWbemServices> {
    unsafe {
        loc.ConnectServer(
            &BSTR::from(namespace),
            &BSTR::new(),
            &BSTR::new(),
            &BSTR::new(),
            0,
            &BSTR::new(),
            None,
        )
        .with_context(|| format!("连接 {namespace} 失败"))
    }
}

/// 与 `ROOT\WMI` 下 `AcpiTest_MULong.GetSetULong` 方法的会话
pub struct WmiSession {
    svc: IWbemServices,
    in_cls: IWbemClassObject,
    obj_path: BSTR,
    method: BSTR,
}

impl WmiSession {
    /// 建立会话(定位实例、缓存方法输入参数模板)
    pub fn connect() -> Result<Self> {
        let loc = create_locator()?;
        let svc = connect(&loc, r"ROOT\WMI")?;
        let cls_name = BSTR::from("AcpiTest_MULong");
        let method = BSTR::from("GetSetULong");

        // 定位该类的第一个实例, 取其 __RELPATH 作为调用路径
        let enumerator = unsafe {
            svc.CreateInstanceEnum(
                &cls_name,
                WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_ERROR_OBJECT,
                None,
            )
            .context("枚举 AcpiTest_MULong 实例失败")?
        };
        let mut objects = [None; 1];
        let mut count = 0u32;
        unsafe {
            enumerator
                .Next(WBEM_INFINITE, &mut objects, &mut count)
                .ok()
                .context("获取 AcpiTest_MULong 实例失败")?;
        }
        let obj = objects[0]
            .take()
            .ok_or_else(|| anyhow!("未找到 AcpiTest_MULong 实例(本机可能不受支持)"))?;
        let mut rel_path = VARIANT::default();
        unsafe {
            obj.Get(w!("__RELPATH"), 0, &mut rel_path, None, None)
                .context("读取实例路径失败")?;
        }
        let obj_path = BSTR::try_from(&rel_path).context("实例路径转换失败")?;

        // 获取方法输入参数模板
        let mut cls = None;
        unsafe {
            svc.GetObject(
                &cls_name,
                WBEM_FLAG_RETURN_WBEM_COMPLETE,
                None,
                Some(&mut cls),
                None,
            )
            .context("获取 AcpiTest_MULong 类定义失败")?;
        }
        let cls = cls.ok_or_else(|| anyhow!("类定义为空"))?;
        let (mut in_cls, mut out_cls) = (None, None);
        unsafe {
            cls.GetMethod(w!("GetSetULong"), 0, &mut in_cls, &mut out_cls)
                .context("获取 GetSetULong 方法失败")?;
        }
        let in_cls = in_cls.ok_or_else(|| anyhow!("方法输入参数模板为空"))?;
        println!("{} {}", "WMI 会话已建立:".green(), obj_path);
        Ok(WmiSession {
            svc,
            in_cls,
            obj_path,
            method,
        })
    }

    /// 执行一次 `GetSetULong`。`data` 为完整命令字(见 registers.rs), 返回 EC 应答。
    pub fn exec(&self, data: u64) -> Result<i64> {
        let in_params =
            unsafe { self.in_cls.SpawnInstance(0).context("创建输入参数失败")? };
        unsafe {
            // 与旧版一致: 以十进制字符串传入, 由 WMI 按 CIM 类型转换
            in_params
                .Put(w!("Data"), 0, &VARIANT::from(data.to_string().as_str()), 0)
                .context("写入 Data 参数失败")?;
        }
        let mut out_params = None;
        unsafe {
            self.svc
                .ExecMethod(
                    &self.obj_path,
                    &self.method,
                    WBEM_FLAG_RETURN_WBEM_COMPLETE,
                    None,
                    &in_params,
                    Some(&mut out_params),
                    None,
                )
                .context("ExecMethod 调用失败")?;
        }
        let out_params = out_params.ok_or_else(|| anyhow!("无输出参数"))?;
        let mut ret = VARIANT::default();
        unsafe {
            out_params
                .Get(w!("Return"), 0, &mut ret, None, None)
                .context("读取返回值失败")?;
        }
        ret.to_string().parse::<i64>().context("返回值解析失败")
    }
}

/// 查询本机型号(`Win32_ComputerSystem.Model`)
pub fn query_model() -> Result<String> {
    let loc = create_locator()?;
    let svc = connect(&loc, r"ROOT\CIMV2")?;
    let enumerator = unsafe {
        svc.ExecQuery(
            &BSTR::from("WQL"),
            &BSTR::from("SELECT Model FROM Win32_ComputerSystem"),
            WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY,
            None,
        )
        .context("查询机型失败")?
    };
    let mut row = [None; 1];
    let mut count = 0u32;
    unsafe {
        enumerator
            .Next(WBEM_INFINITE, &mut row, &mut count)
            .ok()
            .context("读取机型结果失败")?;
    }
    let row = row[0].take().ok_or_else(|| anyhow!("机型查询无结果"))?;
    let mut model = VARIANT::default();
    unsafe {
        row.Get(w!("Model"), 0, &mut model, None, None)
            .context("读取 Model 字段失败")?;
    }
    Ok(model.to_string())
}
