use log::{info, error};
use crate::utils::Timer;
use crate::common::ANNResult;

pub struct DiskIndexBuildLogger {
    timer: Timer,
}

impl DiskIndexBuildLogger {
    pub fn new() -> Self {
        Self { timer: Timer::new() }
    }

    pub fn log_checkpoint(&mut self, message: &str) -> ANNResult<()> {
        let elapsed_time = self.timer.elapsed().as_secs_f32();
        info!("Checkpoint: {}, Time Spent: {:.2} seconds", message, elapsed_time);
        self.timer.reset();
        Ok(())
    }
}

#[cfg(test)]
mod dataset_test {
    use super::*;

    #[test]
    fn test_log() {
        let mut logger = DiskIndexBuildLogger::new();
        logger.log_checkpoint("PQ Construction").unwrap();
        logger.log_checkpoint("Inmem Index Build").unwrap();
        logger.log_checkpoint("Disk Layout").unwrap();
    }
}