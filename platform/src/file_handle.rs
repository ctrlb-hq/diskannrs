// use std::fs::File;
use std::io::{self, ErrorKind};
use std::os::unix::io::AsRawFd; // For Linux file descriptors
use std::ptr;

#[cfg(target_os = "windows")]
use winapi::{
    shared::minwindef::DWORD,
    um::{
        errhandlingapi::GetLastError,
        fileapi::{CreateFileA, OPEN_EXISTING},
        handleapi::{CloseHandle, INVALID_HANDLE_VALUE},
        winbase::{FILE_FLAG_NO_BUFFERING, FILE_FLAG_OVERLAPPED, FILE_FLAG_RANDOM_ACCESS},
        winnt::{FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE, HANDLE},
    },
};

pub enum AccessMode {
    Read,
    Write,
    ReadWrite,
}

pub enum ShareMode {
    None,
    Read,
    Write,
    Delete,
}

#[cfg(target_os = "windows")]
pub struct FileHandle {
    handle: HANDLE,
}

#[cfg(target_os = "linux")]
use tokio::fs::File;
pub struct FileHandle {
    pub file: File,
}

#[cfg(target_os = "windows")]
impl FileHandle {
    pub unsafe fn new(file_name: &str, access_mode: AccessMode, share_mode: ShareMode) -> io::Result<Self> {
        let file_name_c = CString::new(file_name).map_err(|_| {
            io::Error::new(
                ErrorKind::InvalidData,
                format!("Invalid file name: {}", file_name),
            )
        })?;

        let dw_desired_access = match access_mode {
            AccessMode::Read => GENERIC_READ,
            AccessMode::Write => GENERIC_WRITE,
            AccessMode::ReadWrite => GENERIC_READ | GENERIC_WRITE,
        };

        let dw_share_mode = match share_mode {
            ShareMode::None => 0,
            ShareMode::Read => FILE_SHARE_READ,
            ShareMode::Write => FILE_SHARE_WRITE,
            ShareMode::Delete => FILE_SHARE_DELETE,
        };

        let dw_flags_and_attributes = FILE_ATTRIBUTE_READONLY
            | FILE_FLAG_NO_BUFFERING
            | FILE_FLAG_OVERLAPPED
            | FILE_FLAG_RANDOM_ACCESS;

        let handle = CreateFileA(
            file_name_c.as_ptr(),
            dw_desired_access,
            dw_share_mode,
            ptr::null_mut(),
            OPEN_EXISTING,
            dw_flags_and_attributes,
            ptr::null_mut(),
        );

        if handle == INVALID_HANDLE_VALUE {
            let error_code = GetLastError();
            Err(io::Error::from_raw_os_error(error_code as i32))
        } else {
            Ok(Self { handle })
        }
    }

    pub fn raw_handle(&self) -> HANDLE {
        self.handle
    }
}

#[cfg(target_os = "linux")]
impl FileHandle {
    pub async fn new(file_name: &str, access_mode: AccessMode, _share_mode: ShareMode) -> io::Result<Self> {
        let file = match access_mode {
            AccessMode::Read => File::open(file_name).await?,
            AccessMode::Write => File::create(file_name).await?,
            AccessMode::ReadWrite => {
                let file = File::options()
                    .read(true)
                    .write(true)
                    .open(file_name).await?;
                file
            }
        };

        Ok(Self { file })
    }

    pub fn raw_handle(&self) -> i32 {
        self.file.as_raw_fd()
    }
}

#[cfg(target_os = "windows")]
impl Drop for FileHandle {
    fn drop(&mut self) {
        let result = unsafe { CloseHandle(self.handle) };
        if result == 0 {
            let error_code = unsafe { GetLastError() };
            let error = io::Error::from_raw_os_error(error_code as i32);
            log::warn!("Error when dropping FileHandle: {:?}", error);
        }
    }
}

#[cfg(target_os = "linux")]
impl Drop for FileHandle {
    fn drop(&mut self) {
        // File will be automatically closed when it goes out of scope
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::path::Path;
    use tokio::test; // Import tokio::test

    #[test]
    async fn test_create_file_sync() { // Renamed to indicate it's synchronous
        let dummy_file_path = "dummy_file.txt";
        {
            let _file = File::create(dummy_file_path).expect("Failed to create dummy file.");
        }

        let path = Path::new(dummy_file_path);
        {
            let file_handle = FileHandle::new(path.to_str().unwrap(), AccessMode::Read, ShareMode::Read).await.expect("Failed to create FileHandle");

            // Check that the file handle is valid
            #[cfg(target_os = "windows")]
            assert_ne!(file_handle.raw_handle(), INVALID_HANDLE_VALUE);

            #[cfg(target_os = "linux")]
            assert!(file_handle.raw_handle() >= 0);
        }

        // Try to delete the file. If the handle was correctly dropped, this should succeed.
        match std::fs::remove_file(dummy_file_path) {
            Ok(()) => (), // File was deleted successfully, which means the handle was closed.
            Err(e) => panic!("Failed to delete file: {}", e), // Failed to delete the file, likely because the handle is still open.
        }
    }


    #[tokio::test] // Use #[tokio::test] for async tests
    async fn test_create_file_async() {
        let dummy_file_path = "dummy_file_async.txt"; // Different name for async test
        {
            let _file = tokio::fs::File::create(dummy_file_path).await.expect("Failed to create dummy file.");
        }

        let path = Path::new(dummy_file_path);
        {
            let file_handle = FileHandle::new(path.to_str().unwrap(), AccessMode::Read, ShareMode::Read).await.expect("Failed to create FileHandle");

            // Check that the file handle is valid
            #[cfg(target_os = "windows")]
            assert_ne!(file_handle.raw_handle(), INVALID_HANDLE_VALUE);

            #[cfg(target_os = "linux")]
            assert!(file_handle.raw_handle() >= 0);
        }

        // Try to delete the file. If the handle was correctly dropped, this should succeed.
        match tokio::fs::remove_file(dummy_file_path).await {
            Ok(()) => (), // File was deleted successfully, which means the handle was closed.
            Err(e) => panic!("Failed to delete file: {}", e), // Failed to delete the file, likely because the handle is still open.
        }
    }

    #[tokio::test]
    async fn test_file_not_found() {
        let path = Path::new("non_existent_file.txt");
        let file_handle = FileHandle::new(path.to_str().unwrap(), AccessMode::Read, ShareMode::Read).await; // Await here

        assert!(file_handle.is_err());
    }
}