use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Result;
use tokio::sync::watch;

use crate::{
    aggregator::AppMetrics,
    config::Config,
    procfs::{ProcReader, ProcessSnapshot},
};

pub struct MemoryCollector {
    config: Config,
    metrics: Arc<AppMetrics>,
    proc_reader: ProcReader,
    fault_cache: HashMap<u32, (u64, u64)>,
}

impl MemoryCollector {
    #[must_use]
    pub fn new(config: Config, metrics: Arc<AppMetrics>) -> Self {
        Self {
            config,
            metrics,
            proc_reader: ProcReader::new("/proc"),
            fault_cache: HashMap::new(),
        }
    }

    pub async fn run(mut self, mut shutdown_rx: watch::Receiver<bool>) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_millis(
            self.config.collectors.memory.poll_interval_ms,
        ));

        loop {
            tokio::select! {
                changed = shutdown_rx.changed() => {
                    if changed.is_ok() && *shutdown_rx.borrow() {
                        break;
                    }
                }
                _ = interval.tick() => {
                    self.collect_once().await?;
                }
            }
        }

        Ok(())
    }

    pub async fn collect_once(&mut self) -> Result<()> {
        let started_at = Instant::now();
        let snapshot = self.proc_reader.collect()?;

        self.metrics.update_system_memory(
            snapshot.system_memory.available_bytes,
            snapshot.system_memory.cache_bytes,
            snapshot.system_memory.total_bytes,
        );

        for process in &snapshot.processes {
            if !self.should_include(process) {
                continue;
            }

            let (delta_minor, delta_major) = self.consume_fault_deltas(process);

            self.metrics.update_process_memory(
                process.pid,
                &process.comm,
                process.rss_bytes,
                process.vss_bytes,
                delta_minor,
                delta_major,
            );
        }

        let elapsed = started_at.elapsed();
        self.metrics
            .observe_scrape_duration_ms(elapsed.as_secs_f64() * 1000.0);

        Ok(())
    }

    fn consume_fault_deltas(&mut self, process: &ProcessSnapshot) -> (u64, u64) {
        let previous = self
            .fault_cache
            .insert(process.pid, (process.minor_faults, process.major_faults));

        if let Some((last_minor, last_major)) = previous {
            (
                process.minor_faults.saturating_sub(last_minor),
                process.major_faults.saturating_sub(last_major),
            )
        } else {
            (0, 0)
        }
    }

    fn should_include(&self, process: &ProcessSnapshot) -> bool {
        if self.config.filters.exclude_pids.contains(&process.pid) {
            return false;
        }

        if self
            .config
            .filters
            .exclude_comms
            .iter()
            .any(|prefix| process.comm.starts_with(prefix))
        {
            return false;
        }

        if self.config.filters.include_comms.is_empty() {
            return true;
        }

        self.config
            .filters
            .include_comms
            .iter()
            .any(|prefix| process.comm.starts_with(prefix))
    }
}
