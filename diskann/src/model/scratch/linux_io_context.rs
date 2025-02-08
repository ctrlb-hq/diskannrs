use std::sync::Arc;
use tokio::fs::File;
use tokio::sync::Mutex;

use crate::common::ANNError;

/// LinuxIOContext holds a shared file handle (an Arc<File>)
/// guarded by a mutex so that seek/read operations can be serialized.
pub struct LinuxIOContext {
    pub status: Status,
    pub file: Mutex<Arc<File>>,
}

impl Default for LinuxIOContext {
    fn default() -> Self {
        // Because File::open is async, we create a temporary Tokio runtime to open "/dev/null".
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        let default_file = rt
            .block_on(File::open("/dev/null"))
            .expect("Failed to open /dev/null");
        LinuxIOContext {
            status: Status::ReadWait,
            // Wrap the file (now a File) in an Arc and then in a Mutex.
            file: Mutex::new(Arc::new(default_file)),
        }
    }
}

impl LinuxIOContext {
    /// Accepts an Arc<File> (as produced by LinuxAlignedFileReader) and stores it in a mutex.
    pub fn new(file: Arc<File>) -> Self {
        Self {
            status: Status::ReadWait,
            file: Mutex::new(file),
        }
    }
}

/// The various statuses for the IO context.
pub enum Status {
    ReadWait,
    ReadSuccess,
    ReadFailed(ANNError),
    ProcessComplete,
}
