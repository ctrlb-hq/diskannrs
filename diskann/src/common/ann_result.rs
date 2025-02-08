use std::alloc::LayoutError;
use std::array::TryFromSliceError;
use std::io;
use std::num::TryFromIntError;
use tokio::task::JoinError; // Changed from std::thread::JoinError

use log::error;

/// Result type alias
pub type ANNResult<T> = Result<T, ANNError>;

/// DiskANN Error
/// ANNError is `Send` (i.e., safe to send across threads)
#[derive(thiserror::Error, Debug)]
pub enum ANNError {
    /// Index construction and search error
    #[error("IndexError: {err}")]
    IndexError { err: String },

    /// Index configuration error
    #[error("IndexConfigError: {parameter} is invalid, err={err}")]
    IndexConfigError { parameter: String, err: String },

    /// Integer conversion error
    #[error("TryFromIntError: {err}")]
    TryFromIntError {
        #[from]
        err: TryFromIntError,
    },

    /// IO error
    #[error("IOError: {err}")]
    IOError {
        #[from]
        err: io::Error,
    },

    /// Layout error in memory allocation
    #[error("MemoryAllocLayoutError: {err}")]
    MemoryAllocLayoutError {
        #[from]
        err: LayoutError,
    },

    /// PoisonError which can be returned whenever a lock is acquired.
    /// Both Mutexes and RwLocks are poisoned whenever a thread fails while the lock is held.
    #[error("LockPoisonError: {err}")]
    LockPoisonError { err: String },

    /// DiskIOAlignmentError returned when a disk index file fails to align correctly.
    #[error("DiskIOAlignmentError: {err}")]
    DiskIOAlignmentError { err: String },

    /// Logging error.
    /// Note: We **do not** use #[from] here to avoid conflicting with IOError,
    /// because LogError is just another alias for std::io::Error.
    #[error("LogError: {err}")]
    LogError {
        err: io::Error,
    },

    /// PQ construction error.
    /// Error occurred when constructing the PQ pivot or compressed table.
    #[error("PQError: {err}")]
    PQError { err: String },

    /// Array conversion error.
    #[error("Error try creating array from slice: {err}")]
    TryFromSliceError {
        #[from]
        err: TryFromSliceError,
    },

    /// JoinError from task joining failures.
    #[error("JoinError: {0}")]
    JoinError(#[from] JoinError),
}

impl ANNError {
    /// Create, log, and return IndexError
    #[inline]
    pub fn log_index_error(err: String) -> Self {
        error!("IndexError: {}", err);
        ANNError::IndexError { err }
    }

    /// Create, log, and return IndexConfigError
    #[inline]
    pub fn log_index_config_error(parameter: String, err: String) -> Self {
        error!("IndexConfigError: {} is invalid, err={}", parameter, err);
        ANNError::IndexConfigError { parameter, err }
    }

    /// Create, log, and return TryFromIntError
    #[inline]
    pub fn log_try_from_int_error(err: TryFromIntError) -> Self {
        error!("TryFromIntError: {}", err);
        ANNError::TryFromIntError { err }
    }

    /// Create, log, and return IOError
    #[inline]
    pub fn log_io_error(err: io::Error) -> Self {
        error!("IOError: {}", err);
        ANNError::IOError { err }
    }

    /// Create, log, and return DiskIOAlignmentError
    #[inline]
    pub fn log_disk_io_request_alignment_error(err: String) -> Self {
        error!("DiskIOAlignmentError: {}", err);
        ANNError::DiskIOAlignmentError { err }
    }

    /// Create, log, and return MemoryAllocLayoutError
    #[inline]
    pub fn log_mem_alloc_layout_error(err: LayoutError) -> Self {
        error!("MemoryAllocLayoutError: {}", err);
        ANNError::MemoryAllocLayoutError { err }
    }

    /// Create, log, and return LockPoisonError
    #[inline]
    pub fn log_lock_poison_error(err: String) -> Self {
        error!("LockPoisonError: {}", err);
        ANNError::LockPoisonError { err }
    }

    /// Create, log, and return PQError
    #[inline]
    pub fn log_pq_error(err: String) -> Self {
        error!("PQError: {}", err);
        ANNError::PQError { err }
    }

    /// Create, log, and return TryFromSliceError
    #[inline]
    pub fn log_try_from_slice_error(err: TryFromSliceError) -> Self {
        error!("TryFromSliceError: {}", err);
        ANNError::TryFromSliceError { err }
    }
}

#[cfg(test)]
mod ann_result_test {
    use super::*;

    #[test]
    fn ann_err_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<ANNError>();
    }
}
