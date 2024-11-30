#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, FALSE};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::CreateMutexW;

#[cfg(target_os = "windows")]
fn get_mutex_full_name(name: &str) -> String {
    format!("Global\\{}\0", name)
}

#[cfg(target_os = "windows")]
pub fn ensure_single_instance(name: &str) -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    use log::error;

    unsafe {
        let wide: Vec<u16> = OsStr::new(get_mutex_full_name(name).as_str())
            .encode_wide()
            .collect();

        let mutex_handle = CreateMutexW(std::ptr::null_mut(), FALSE, wide.as_ptr());

        if mutex_handle.is_null() {
            error!("Failed to create mutex");
            return false;
        }

        if GetLastError() == ERROR_ALREADY_EXISTS {
            error!("Another instance is already running");
            CloseHandle(mutex_handle);
            return false;
        }

        true
    }
}
