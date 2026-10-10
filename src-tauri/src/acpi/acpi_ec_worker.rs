use crate::acpi::UniwillAcpiEc;

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::{self, Sender};
use std::sync::OnceLock;
use std::thread;

// EC 操作任务。所有任务均在专用线程中执行。
type EcJob =
    Box<dyn FnOnce(&mut Option<UniwillAcpiEc>) + Send + 'static>;

#[derive(Clone)]
pub struct AcpiEcWorker {
    tx: Sender<EcJob>,
}

impl AcpiEcWorker {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel::<EcJob>();

        thread::spawn(move || {
            // EC 实例只在这个线程中创建和使用。
            let mut ec: Option<UniwillAcpiEc> = None;

            while let Ok(job) = rx.recv() {
                job(&mut ec);
            }
        });

        Self { tx }
    }

    /// 提交一个 EC 操作，并等待执行结果。
    ///
    /// EC 首次打开成功后会持续复用。
    /// 打开失败时保留未初始化状态，下次调用可以重试。
    pub fn call<R, F>(&self, f: F) -> Result<R, String>
    where
        R: Send + 'static,
        F: FnOnce(&UniwillAcpiEc) -> R + Send + 'static,
    {
        let (result_tx, result_rx) =
            mpsc::sync_channel::<Result<R, String>>(1);

        self.tx
            .send(Box::new(move |slot| {
                // 延迟初始化：只在尚无有效实例时尝试打开。
                if slot.is_none() {
                    match UniwillAcpiEc::open() {
                        Ok(instance) => {
                            println!("ACPI EC 初始化成功");
                            *slot = Some(instance);
                        }
                        Err(e) => {
                            let _ = result_tx.send(Err(format!(
                                "打开 ACPIDriver 失败: {}",
                                e
                            )));
                            return;
                        }
                    }
                }

                // 捕获操作过程中发生的 panic，避免工作线程直接退出。
                let outcome = {
                    let ec = slot.as_ref().expect("EC 应已初始化");

                    catch_unwind(AssertUnwindSafe(|| f(ec)))
                };

                match outcome {
                    Ok(value) => {
                        let _ = result_tx.send(Ok(value));
                    }
                    Err(_) => {
                        // 操作发生 panic，释放当前实例。
                        // 后续请求将重新尝试初始化。
                        *slot = None;

                        let _ = result_tx.send(Err(
                            "EC 操作发生 panic，实例已释放".to_string()
                        ));
                    }
                }
            }))
            .map_err(|_| "EC 工作线程已停止".to_string())?;

        result_rx
            .recv()
            .map_err(|_| "EC 工作线程未返回结果".to_string())?
    }
}

// 全局唯一的 EC 工作线程。
static ACPI_EC_WORKER: OnceLock<AcpiEcWorker> = OnceLock::new();

/// 获取全局 EC 工作线程。
pub fn acpi_ec_worker() -> &'static AcpiEcWorker {
    ACPI_EC_WORKER.get_or_init(AcpiEcWorker::new)
}