use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{self, AsyncReadExt, AsyncSeekExt};
use crate::{model::AlignedRead, common::ANNError, common::ANNResult};

pub struct LinuxAlignedFileReader {
    pub file: Arc<File>,
}

impl LinuxAlignedFileReader {
    pub async fn new(fname: &str) -> ANNResult<Self> {
        // Open the file asynchronously and wrap it in an Arc.
        let file = Arc::new(
            File::open(fname)
                .await
                .map_err(ANNError::log_io_error)?,
        );
        Ok(Self { file })
    }

    /// Reads concurrently into each provided read request.
    ///
    /// This API takes ownership of the read requests (each of which owns its buffer)
    /// and returns a vector of the updated read requests after the reads complete.
    ///
    /// # Safety
    ///
    /// The conversion from `&mut [T]` to `&mut [u8]` is unsafe. It is assumed that the type
    /// `T` has a memory layout compatible with raw bytes (for example, if `T` is `u8` or a plain-old-data type).
    ///
    /// # Type Bounds
    ///
    /// `T` must be `Send` and `'static` so that the future spawned by `tokio::spawn` is valid.
    pub async fn read<T>(
        &self,
        read_requests: Vec<AlignedRead<T>>,
    ) -> ANNResult<Vec<AlignedRead<T>>>
    where
        T: Send + 'static,
    {
        let mut handles = Vec::new();

        for req in read_requests.into_iter() {
            let file = self.file.clone();
            let offset = req.offset;
            // Move the entire `req` (which owns its buffer) into the async task.
            let handle = tokio::spawn(async move {
                // Clone the file handle so we can obtain a mutable one.
                let mut file = file
                    .try_clone()
                    .await.map_err(ANNError::log_io_error)?;
                let mut req = req;
                // Convert the buffer from a slice of T to a slice of u8.
                // This conversion is unsafe because it reinterprets the underlying bytes.
                let buf = unsafe {
                    std::slice::from_raw_parts_mut(
                        req.aligned_buf.as_mut_ptr() as *mut u8,
                        req.aligned_buf.len() * std::mem::size_of::<T>(),
                    )
                };
                file.seek(std::io::SeekFrom::Start(offset))
                    .await
                    .map_err(ANNError::log_io_error)?;
                file.read_exact(buf)
                    .await
                    .map_err(ANNError::log_io_error)?;
                Ok::<AlignedRead<T>, ANNError>(req)
            });
            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            // Convert any JoinError to ANNError and then propagate any error from the async task.
            let req = handle.await.map_err(ANNError::from)??;
            results.push(req);
        }

        Ok(results)
    }
}
