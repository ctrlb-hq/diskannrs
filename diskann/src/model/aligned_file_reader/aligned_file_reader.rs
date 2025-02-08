use crate::common::{ANNError, ANNResult};
use crate::model::IOContext;
use std::sync::Arc;
use std::time::Duration;
use std::{ptr, thread};

pub const DISK_IO_ALIGNMENT: usize = 512;

/// Aligned read struct for disk IO.
/// This version takes ownership of the aligned buffer (as a Vec<T>),
/// so that the buffer can be moved into concurrent tasks safely.
pub struct AlignedRead<T> {
    /// Where to read from.
    /// The offset must be aligned to DISK_IO_ALIGNMENT.
    pub offset: u64,

    /// The buffer into which data is read.
    /// The size (in bytes) of the buffer must be a multiple of DISK_IO_ALIGNMENT.
    pub aligned_buf: Vec<T>,
}

impl<T> AlignedRead<T> {
    /// Create a new AlignedRead.
    ///
    /// # Parameters
    /// - `offset`: The file offset from which to read. Must be a multiple of DISK_IO_ALIGNMENT.
    /// - `aligned_buf`: The owned buffer to read data into. Its total byte size (i.e. length * size_of::<T>()) must be aligned.
    ///
    /// # Errors
    /// Returns an error if either the offset or the buffer size is not properly aligned.
    pub fn new(offset: u64, aligned_buf: Vec<T>) -> ANNResult<Self> {
        Self::assert_is_aligned(offset as usize)?;
        let buffer_size = aligned_buf.len() * std::mem::size_of::<T>();
        Self::assert_is_aligned(buffer_size)?;
        Ok(Self { offset, aligned_buf })
    }

    /// Check that a given value is a multiple of DISK_IO_ALIGNMENT.
    fn assert_is_aligned(val: usize) -> ANNResult<()> {
        if val % DISK_IO_ALIGNMENT == 0 {
            Ok(())
        } else {
            Err(ANNError::log_disk_io_request_alignment_error(format!(
                "The offset or length (in bytes: {}) of AlignedRead request is not {} bytes aligned",
                val, DISK_IO_ALIGNMENT
            )))
        }
    }

    /// Returns an immutable slice of the aligned buffer.
    pub fn aligned_buf(&self) -> &[T] {
        &self.aligned_buf
    }
}
