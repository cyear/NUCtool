use crate::Result;
use std::{marker::PhantomData, rc::Rc};
use windows::Win32::{
    Foundation::RPC_E_CHANGED_MODE,
    System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize},
};

pub(crate) struct Com {
    uninitialize: bool,
    thread_local: PhantomData<Rc<()>>,
}

impl Com {
    pub(crate) fn initialize() -> Result<Self> {
        let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        // An SDK host may have already initialized this thread as STA.
        if hr != RPC_E_CHANGED_MODE {
            hr.ok()?;
        }
        Ok(Self {
            uninitialize: hr != RPC_E_CHANGED_MODE,
            thread_local: PhantomData,
        })
    }
}

impl Drop for Com {
    fn drop(&mut self) {
        if self.uninitialize {
            unsafe { CoUninitialize() };
        }
    }
}
