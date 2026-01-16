use crate::config;
use std::thread;
use tracing;

pub struct BackgroundTask {
    pub cfg: config::TasksConfig,
}

impl BackgroundTask {
    pub fn new(cfg: config::TasksConfig) -> Self {
        Self { cfg }
    }

    // Runs a background task at specified intervals.
    pub fn run(&self) {
        let interval_minutes = self.cfg.interval_minutes.clone();

        thread::spawn(move || {
            let interval = std::time::Duration::from_secs((interval_minutes.clone() as u64) * 60);
            // Simulate a background task
            loop {
                tracing::info!("Background task is running...");

                thread::sleep(interval);
            }
        });
    }
}
