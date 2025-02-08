use std::sync::Arc;
use crate::{model::AlignedRead, common::ANNResult};

#[cfg(target_os = "windows")]
use crate::model::{WindowsAlignedFileReader, IOContext};

#[cfg(target_os = "linux")]
use crate::model::{LinuxAlignedFileReader, LinuxIOContext};

pub struct DiskGraphStorage {
    #[cfg(target_os = "windows")]
    disk_graph_reader: Arc<WindowsAlignedFileReader>,

    #[cfg(target_os = "linux")]
    disk_graph_reader: Arc<LinuxAlignedFileReader>,

    #[cfg(target_os = "windows")]
    ctx: Arc<IOContext>,

    #[cfg(target_os = "linux")]
    ctx: Arc<LinuxIOContext>,
}

impl DiskGraphStorage {
    #[cfg(target_os = "windows")]
    pub fn new(disk_graph_reader: Arc<WindowsAlignedFileReader>) -> ANNResult<Self> {
        let ctx = disk_graph_reader.get_ctx()?;
        Ok(Self {
            disk_graph_reader,
            ctx,
        })
    }

    #[cfg(target_os = "linux")]
    pub async fn new(disk_graph_reader: Arc<LinuxAlignedFileReader>) -> ANNResult<Self> {
        // LinuxAlignedFileReader holds an Arc<File> already.
        let file = disk_graph_reader.file.clone();
        // LinuxIOContext::new now accepts an Arc<File>.
        let ctx = Arc::new(LinuxIOContext::new(file));
        Ok(Self {
            disk_graph_reader,
            ctx,
        })
    }

    // Windows branch: expects a mutable slice.
    #[cfg(target_os = "windows")]
    pub async fn read<T>(&self, read_requests: &mut [AlignedRead<T>]) -> ANNResult<()> {
        self.disk_graph_reader.read(read_requests, &self.ctx)
    }

    // Linux branch: expects a Vec (i.e. ownership is transferred).
    // Here we add the trait bounds to T.
    #[cfg(target_os = "linux")]
    pub async fn read<T: Send + 'static>(&self, read_requests: Vec<AlignedRead<T>>) -> ANNResult<()> {
        self.disk_graph_reader.read(read_requests).await?;
        Ok(())
    }
}
