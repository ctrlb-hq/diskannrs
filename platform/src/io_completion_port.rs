use std::io;

#[cfg(target_os = "windows")]
use winapi::{
    ctypes::c_void,
    shared::{basetsd::ULONG_PTR, minwindef::DWORD},
    um::{
        errhandlingapi::GetLastError,
        handleapi::{CloseHandle, INVALID_HANDLE_VALUE},
        ioapiset::CreateIoCompletionPort,
        winnt::HANDLE,
    },
};

#[cfg(target_os = "linux")]
use tokio::sync::mpsc;

use crate::FileHandle;

#[cfg(target_os = "windows")]
pub struct IOCompletionPort {
    io_completion_port: HANDLE,
}

#[cfg(target_os = "linux")]
pub struct IOCompletionPort {
    sender: mpsc::Sender<()>,
    receiver: mpsc::Receiver<()>,
}

#[cfg(target_os = "windows")]
impl IOCompletionPort {
    pub fn new(
        file_handle: &FileHandle,
        existing_completion_port: Option<&IOCompletionPort>,
        completion_key: ULONG_PTR,
        number_of_concurrent_threads: DWORD,
    ) -> io::Result<Self> {
        let io_completion_port = unsafe {
            CreateIoCompletionPort(
                file_handle.raw_handle(),
                existing_completion_port
                    .map_or(std::ptr::null_mut::<c_void>(), |io_completion_port| {
                        io_completion_port.raw_handle()
                    }),
                completion_key,
                number_of_concurrent_threads,
            )
        };

        if io_completion_port == INVALID_HANDLE_VALUE {
            let error_code = unsafe { GetLastError() };
            return Err(io::Error::from_raw_os_error(error_code as i32));
        }

        Ok(IOCompletionPort { io_completion_port })
    }

    pub fn raw_handle(&self) -> HANDLE {
        self.io_completion_port
    }
}

#[cfg(target_os = "linux")]
impl IOCompletionPort {
    pub fn new(
        _file_handle: &FileHandle,
        _existing_completion_port: Option<&IOCompletionPort>,
        _completion_key: u64,
        _number_of_concurrent_threads: u32,
    ) -> io::Result<Self> {
        let (sender, receiver) = mpsc::channel(1);
        Ok(IOCompletionPort { sender, receiver })
    }

    pub fn raw_handle(&self) -> *mut std::ffi::c_void {
        std::ptr::null_mut()
    }
}

#[cfg(target_os = "windows")]
impl Drop for IOCompletionPort {
    fn drop(&mut self) {
        let result = unsafe { CloseHandle(self.io_completion_port) };
        if result == 0 {
            let error_code = unsafe { GetLastError() };
            let error = io::Error::from_raw_os_error(error_code as i32);
            log::warn!("Error when dropping IOCompletionPort: {:?}", error);
        }
    }
}

#[cfg(target_os = "linux")]
impl Drop for IOCompletionPort {
    fn drop(&mut self) {
        // No specific cleanup needed for the mpsc channel
    }
}

#[cfg(target_os = "windows")]
impl Default for IOCompletionPort {
    fn default() -> Self {
        Self {
            io_completion_port: INVALID_HANDLE_VALUE,
        }
    }
}

#[cfg(target_os = "linux")]
impl Default for IOCompletionPort {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel(1);
        Self { sender, receiver }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_handle::{AccessMode, ShareMode};

    #[tokio::test]
    async fn create_io_completion_port() {
        let file_name = "../diskann/tests/data/delete_set_50pts.bin";
        let file_handle = unsafe { FileHandle::new(file_name, AccessMode::Read, ShareMode::Read) }
            .await.expect("Failed to create file handle.");

        let io_completion_port = IOCompletionPort::new(&file_handle, None, 0, 0);

        assert!(
            io_completion_port.is_ok(),
            "Failed to create IOCompletionPort."
        );
    }

    #[tokio::test]
    async fn drop_io_completion_port() {
        let file_name = "../diskann/tests/data/delete_set_50pts.bin";
        let file_handle = unsafe { FileHandle::new(file_name, AccessMode::Read, ShareMode::Read) }
            .await.expect("Failed to create file handle.");

        let io_completion_port = IOCompletionPort::new(&file_handle, None, 0, 0)
            .expect("Failed to create IOCompletionPort.");

        // After this line, io_completion_port goes out of scope and its Drop trait will be called.
        let _ = io_completion_port;
        // We have no easy way to test that the Drop trait works correctly, but if it doesn't,
        // a resource leak or other problem may become apparent in later tests or in real use of the code.
    }

    #[test]
    fn default_io_completion_port() {
        let io_completion_port = IOCompletionPort::default();
        #[cfg(target_os = "windows")]
        assert_eq!(
            io_completion_port.raw_handle(),
            INVALID_HANDLE_VALUE,
            "Default IOCompletionPort did not have INVALID_HANDLE_VALUE."
        );
        #[cfg(target_os = "linux")]
        assert!(io_completion_port.raw_handle().is_null(), "Default IOCompletionPort did not have a null handle.");
    }
}