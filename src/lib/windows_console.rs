use std::io;

use log::info;
use windows_sys::Win32::Foundation::{FALSE, INVALID_HANDLE_VALUE, TRUE};
use windows_sys::Win32::System::Console::{
    AllocConsole, AttachConsole, FreeConsole, GetStdHandle, ATTACH_PARENT_PROCESS,
    STD_OUTPUT_HANDLE,
};

// 分配控制台
pub fn alloc_console() -> io::Result<()> {
    unsafe {
        if AllocConsole() == FALSE {
            return Err(io::Error::last_os_error());
        }
        let stdout_handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if stdout_handle == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

// 销毁当前控制台 (AllocConsole 后销毁, 或是控制台应用隐藏窗口)
pub fn free_console() -> bool {
    unsafe { FreeConsole() == TRUE }
}

// 尝试附加到父进程的控制台 (让UI进程可以输出日志)
pub fn try_attach_parent_console() -> Result<(), std::io::Error> {
    unsafe {
        let result = AttachConsole(ATTACH_PARENT_PROCESS);
        if result != 0 {
            let stdout_handle = GetStdHandle(STD_OUTPUT_HANDLE);
            if stdout_handle != INVALID_HANDLE_VALUE {
                info!("Successfully attached to parent console");
                return Ok(());
            }
        }
        Ok(())
    }
}
