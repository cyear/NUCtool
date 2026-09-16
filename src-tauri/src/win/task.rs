use std::env;
use std::thread;

use windows::{
    core::{BSTR, Interface, Result},
    Win32::{
        System::{
            Com::{
                CoCreateInstance,
                CoInitializeEx,
                CoUninitialize,
                CLSCTX_INPROC_SERVER,
                COINIT_APARTMENTTHREADED,
            },
            TaskScheduler::{
                IExecAction,
                ILogonTrigger,
                ITaskService,
                TaskScheduler,
                TASK_ACTION_EXEC,
                TASK_CREATE_OR_UPDATE,
                TASK_LOGON_INTERACTIVE_TOKEN,
                TASK_RUNLEVEL_HIGHEST,
                TASK_TRIGGER_LOGON,
            },
            Variant::VARIANT,
        },
    },
};

const TASK_NAME: &str = "NUCtool";

/// 在独立 COM 线程执行任务计划操作
fn run_on_com_thread<F, T>(f: F) -> Result<T>
where
    F: FnOnce() -> Result<T> + Send + 'static,
    T: Send + 'static,
{
    thread::spawn(move || {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
        }

        let result = f();

        unsafe {
            CoUninitialize();
        }

        result
    })
    .join()
    .map_err(|_| {
        windows::core::Error::new(
            windows::core::HRESULT(0x80004005u32 as i32),
            "COM 线程执行失败",
        )
    })?
}

fn connect_task_service() -> Result<ITaskService> {
    unsafe {
        let service: ITaskService = CoCreateInstance(
            &TaskScheduler,
            None,
            CLSCTX_INPROC_SERVER,
        )?;

        service.Connect(
            &VARIANT::default(),
            &VARIANT::default(),
            &VARIANT::default(),
            &VARIANT::default(),
        )?;

        Ok(service)
    }
}

/// 检查 NUCtool 开机自启任务是否存在
pub fn is_startup_task_exists() -> Result<bool> {
    run_on_com_thread(|| {
        let service = connect_task_service()?;

        let root = unsafe {
            service.GetFolder(&BSTR::from("\\"))
        }?;

        match unsafe {
            root.GetTask(&BSTR::from(TASK_NAME))
        } {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    })
}

/// 创建 NUCtool 开机自启任务
pub fn create_startup_task() -> Result<()> {
    run_on_com_thread(|| {
        let service = connect_task_service()?;

        let root = unsafe {
            service.GetFolder(&BSTR::from("\\"))
        }?;

        let definition = unsafe {
            service.NewTask(0)
        }?;

        // =========================
        // Principal
        // =========================

        let principal = unsafe {
            definition.Principal()
        }?;

        unsafe {
            principal.SetLogonType(TASK_LOGON_INTERACTIVE_TOKEN)?;
            principal.SetRunLevel(TASK_RUNLEVEL_HIGHEST)?;
        }

        // =========================
        // Action
        // =========================

        let actions = unsafe {
            definition.Actions()
        }?;

        let action = unsafe {
            actions.Create(TASK_ACTION_EXEC)?
        };

        let exec_action: IExecAction = action.cast()?;

        let exe = env::current_exe()
            .map_err(|e| {
                windows::core::Error::new(
                    windows::core::HRESULT(0x80004005u32 as i32),
                    format!("获取当前 EXE 路径失败: {e}"),
                )
            })?;

        let exe = exe
            .to_str()
            .ok_or_else(|| {
                windows::core::Error::new(
                    windows::core::HRESULT(0x80004005u32 as i32),
                    "EXE 路径无法转换",
                )
            })?
            .to_string();

        unsafe {
            exec_action.SetPath(&BSTR::from(exe))?;

            exec_action.SetArguments(
                &BSTR::from("--hide --fan-control"),
            )?;
        }

        // =========================
        // Trigger
        // =========================

        let triggers = unsafe {
            definition.Triggers()
        }?;

        let trigger = unsafe {
            triggers.Create(TASK_TRIGGER_LOGON)?
        };

        let logon_trigger: ILogonTrigger = trigger.cast()?;

        let username = env::var("USERNAME")
            .map_err(|e| {
                windows::core::Error::new(
                    windows::core::HRESULT(0x80004005u32 as i32),
                    format!("获取用户名失败: {e}"),
                )
            })?;

        let domain = env::var("USERDOMAIN")
            .unwrap_or_default();

        let user_id = if domain.is_empty() {
            username
        } else {
            format!("{domain}\\{username}")
        };

        unsafe {
            logon_trigger.SetUserId(
                &BSTR::from(user_id)
            )?;
        }

        // =========================
        // 注册任务
        // =========================

        unsafe {
            root.RegisterTaskDefinition(
                &BSTR::from(TASK_NAME),
                &definition,
                TASK_CREATE_OR_UPDATE.0,
                &VARIANT::default(),
                &VARIANT::default(),
                TASK_LOGON_INTERACTIVE_TOKEN,
                &VARIANT::default(),
            )?;
        }

        Ok(())
    })
}

/// 删除 NUCtool 开机自启任务
pub fn remove_startup_task() -> Result<()> {
    run_on_com_thread(|| {
        let service = connect_task_service()?;

        let root = unsafe {
            service.GetFolder(&BSTR::from("\\"))
        }?;

        unsafe {
            root.DeleteTask(
                &BSTR::from(TASK_NAME),
                0,
            )?;
        }

        Ok(())
    })
}