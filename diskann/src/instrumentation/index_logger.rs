use std::sync::atomic::{AtomicUsize, Ordering};
use log::{info, error};
use crate::utils::Timer;
use crate::common::ANNResult;

pub struct IndexLogger {
    items_processed: AtomicUsize,
    timer: Timer,
    range: usize,
}

impl IndexLogger {
    pub fn new(range: usize) -> Self {
        Self {
            items_processed: AtomicUsize::new(0),
            timer: Timer::new(),
            range,
        }
    }

    pub fn vertex_processed(&self) -> ANNResult<()> {
        let count = self.items_processed.fetch_add(1, Ordering::Relaxed);
        if count % 100_000 == 0 {
            let percentage_complete = (100_f32 * count as f32) / (self.range as f32);
            let elapsed_time = self.timer.elapsed().as_secs_f32();
            info!(
                "Index Construction: {}% complete, Time Spent: {:.2} seconds",
                percentage_complete, elapsed_time
            );
        }

        Ok(())
    }
}