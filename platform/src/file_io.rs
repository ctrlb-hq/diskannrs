use std::io;
use std::ptr;

#[cfg(target_os = "windows")]
use winapi::{
    ctypes::c_void,
    shared::{
        basetsd::ULONG_PTR,
        minwindef::{DWORD, FALSE},
        winerror::{ERROR_IO_PENDING, WAIT_TIMEOUT},
    },
    um::{
        errhandlingapi::GetLastError,
        fileapi::ReadFile,
        ioapiset::GetQueuedCompletionStatus,
        minwinbase::OVERLAPPED,
        winnt::HANDLE,
    },
};

#[cfg(target_os = "linux")]
use tokio::fs::File;
#[cfg(target_os = "linux")]
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::FileHandle;


#[cfg(target_os = "windows")]
/// Asynchronously queue a read request from a file into a buffer slice.
///
/// Wraps the unsafe Windows API function `ReadFile`, making it safe to call only when the overlapped buffer
/// remains valid and unchanged anywhere else during the entire async operation.
///
/// Returns a boolean indicating whether the read operation completed synchronously or is pending.
///
/// # Safety
///
/// This function is marked as `unsafe` because it uses raw pointers and requires the caller to ensure
/// that the buffer slice and the overlapped buffer stay valid during the whole async operation.
pub unsafe fn read_file_to_slice(
    file_handle: &FileHandle,
    buffer_slice: &mut [u8], // Changed to &[u8] for consistency and safety.
    overlapped: *mut OVERLAPPED,
    offset: u64,
) -> io::Result<bool> {
    let num_bytes = buffer_slice.len(); // Use slice's len() for size
    ptr::write_volatile(overlapped, std::mem::zeroed()); // Use write_volatile
    (*overlapped).u.s_mut().Offset = offset as u32;
    (*overlapped).u.s_mut().OffsetHigh = (offset >> 32) as u32;

    let result = ReadFile(
        file_handle.raw_handle(),
        buffer_slice.as_mut_ptr() as *mut c_void,
        num_bytes as DWORD,
        ptr::null_mut(),
        overlapped,
    );

    match result {
        FALSE => {
            let error = GetLastError();
            if error != ERROR_IO_PENDING {
                Err(io::Error::from_raw_os_error(error as i32))
            } else {
                Ok(false)
            }
        }
        _ => Ok(true),
    }
}

#[cfg(target_os = "linux")]
pub async fn read_file_to_slice(
    file_handle: &FileHandle,
    buffer_slice: &mut [u8], // Changed to &[u8]
) -> io::Result<()> {
    let mut file = file_handle.file.try_clone().await?;
    file.read_exact(buffer_slice).await?; // Directly use the buffer slice.
    Ok(())
}

#[cfg(target_os = "windows")]
/// Retrieves the results of an asynchronous I/O operation on an I/O completion port.
///
/// Wraps the unsafe Windows API function `GetQueuedCompletionStatus`, making it safe to call only when the overlapped buffer
/// remains valid and unchanged anywhere else during the entire async operation.
///
/// Returns a boolean indicating whether an I/O operation completed synchronously or is still pending.
///
/// # Safety
///
/// This function is marked as `unsafe` because it uses raw pointers and requires the caller to ensure
/// that the overlapped buffer stays valid during the whole async operation.
pub unsafe fn get_queued_completion_status(
    completion_port: &IOCompletionPort, // Assuming IOCompletionPort exists and is correctly defined
    lp_number_of_bytes: &mut DWORD,
    lp_completion_key: &mut ULONG_PTR,
    lp_overlapped: *mut *mut OVERLAPPED,
    dw_milliseconds: DWORD,
) -> io::Result<bool> {
    let result = GetQueuedCompletionStatus(
        completion_port.raw_handle(),
        lp_number_of_bytes,
        lp_completion_key,
        lp_overlapped,
        dw_milliseconds,
    );

    match result {
        0 => {
            let error = GetLastError();
            if error == WAIT_TIMEOUT {
                Ok(false)
            } else {
                Err(io::Error::from_raw_os_error(error as i32))
            }
        }
        _ => Ok(true),
    }
}

use crate::IOCompletionPort;

#[cfg(target_os = "linux")]
pub async fn get_queued_completion_status(
    _completion_port: &IOCompletionPort, // The parameter is kept for API consistency.
    _lp_number_of_bytes: &mut usize,       // Use usize for Linux.
    _lp_completion_key: &mut usize,        // Use usize for Linux.
    // _lp_overlapped: *mut *mut OVERLAPPED,    // Keep OVERLAPPED for API consistency.
    _dw_milliseconds: u32,                  // Use u32 for Linux.
) -> io::Result<bool> {
    Ok(true)
}

#[cfg(test)]
mod tests {
    use crate::file_handle::{AccessMode, ShareMode};

    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use tokio::test;

    #[cfg(target_os = "windows")]
    #[test]
    fn test_read_file_to_slice_windows_sync() {  // Windows-specific test
        let path = Path::new("temp.txt");
        {
            let mut file = File::create(path).unwrap();
            file.write_all(b"Hello, world!").unwrap();
        }

        let mut buffer: [u8; 512] = [0; 512];
        let mut overlapped = unsafe { std::mem::zeroed::<OVERLAPPED>() }; // OVERLAPPED is used here
        {
            let file_handle = unsafe {
                FileHandle::new(path.to_str().unwrap(), AccessMode::Read, ShareMode::Read)
            }
            .unwrap();

            let result = unsafe { read_file_to_slice(&file_handle, &mut buffer, &mut overlapped, 0) };

            assert!(result.is_ok());
            let result_str = std::str::from_utf8(&buffer[.."Hello, world!".len()]).unwrap();
            assert_eq!(result_str, "Hello, world!");
        }

        std::fs::remove_file("temp.txt").unwrap();
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn test_read_file_to_slice_linux_async() { // Linux-specific test
        let path = Path::new("temp_async.txt");
        {
            let mut file = tokio::fs::File::create(path).await.unwrap();
            file.write_all(b"Hello, world!").await.unwrap();
        }

        let mut buffer: [u8; 512] = [0; 512];
        {
            let file_handle = FileHandle::new(path.to_str().unwrap(), AccessMode::Read, ShareMode::Read).await.unwrap();

            read_file_to_slice(&file_handle, &mut buffer).await.unwrap();

            let result_str = std::str::from_utf8(&buffer[.."Hello, world!".len()]).unwrap();
            assert_eq!(result_str, "Hello, world!");
        }

        tokio::fs::remove_file("temp_async.txt").await.unwrap();
    }

    #[tokio::test]
    async fn test_read_file_to_slice_async() {
        let path = Path::new("temp_async.txt");
        {
            let mut file = tokio::fs::File::create(path).await.unwrap();
            file.write_all(b"Hello, world!").await.unwrap();
        }

        let mut buffer: [u8; 512] = [0; 512];
        {
            let file_handle = FileHandle::new(path.to_str().unwrap(), AccessMode::Read, ShareMode::Read).await.unwrap();

            read_file_to_slice(&file_handle, &mut buffer).await.unwrap();

            let result_str = std::str::from_utf8(&buffer[.."Hello, world!".len()]).unwrap();
            assert_eq!(result_str, "Hello, world!");
        }

        tokio::fs::remove_file("temp_async.txt").await.unwrap();
    }
}