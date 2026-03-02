use std::{
    collections::HashSet,
    sync::{Mutex, MutexGuard},
};

use anyhow::Result;
use prometheus_client::{
    encoding::{EncodeLabelSet, text::encode},
    metrics::{
        counter::Counter,
        family::Family,
        gauge::Gauge,
        histogram::{Histogram, exponential_buckets},
    },
    registry::Registry,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct ProcLabels {
    pub pid: u32,
    pub comm: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct LifecycleLabels {
    pub event: String,
    pub pid: u32,
    pub comm: String,
}

pub struct AppMetrics {
    registry: Mutex<Registry>,
    series_limiter: SeriesLimiter,
    cpu_run_time_ns_total: Family<ProcLabels, Counter>,
    cpu_wait_time_ns_total: Family<ProcLabels, Counter>,
    proc_lifecycle_total: Family<LifecycleLabels, Counter>,
    mem_rss_bytes: Family<ProcLabels, Gauge>,
    mem_vss_bytes: Family<ProcLabels, Gauge>,
    mem_page_faults_minor_total: Family<ProcLabels, Counter>,
    mem_page_faults_major_total: Family<ProcLabels, Counter>,
    mem_oom_kills_total: Family<ProcLabels, Counter>,
    mem_available_bytes: Gauge,
    mem_cache_bytes: Gauge,
    mem_total_bytes: Gauge,
    scrape_duration_ms: Histogram,
    series_dropped_total: Counter,
    loader_failures_total: Counter,
}

impl AppMetrics {
    #[must_use]
    pub fn new(max_series: usize) -> Self {
        let mut registry = Registry::default();

        let cpu_run_time_ns_total = Family::default();
        let cpu_wait_time_ns_total = Family::default();
        let proc_lifecycle_total = Family::default();
        let mem_rss_bytes = Family::default();
        let mem_vss_bytes = Family::default();
        let mem_page_faults_minor_total = Family::default();
        let mem_page_faults_major_total = Family::default();
        let mem_oom_kills_total = Family::default();
        let mem_available_bytes = Gauge::default();
        let mem_cache_bytes = Gauge::default();
        let mem_total_bytes = Gauge::default();
        let scrape_duration_ms = Histogram::new(exponential_buckets(1.0, 2.0, 12));
        let series_dropped_total = Counter::default();
        let loader_failures_total = Counter::default();

        registry.register(
            "drishti_cpu_run_time_ns",
            "Cumulative CPU runtime in nanoseconds per process",
            cpu_run_time_ns_total.clone(),
        );
        registry.register(
            "drishti_cpu_wait_time_ns",
            "Cumulative scheduler wait time in nanoseconds per process",
            cpu_wait_time_ns_total.clone(),
        );
        registry.register(
            "drishti_proc_lifecycle",
            "Process lifecycle events grouped by event type",
            proc_lifecycle_total.clone(),
        );
        registry.register(
            "drishti_mem_rss_bytes",
            "Resident set size in bytes per process",
            mem_rss_bytes.clone(),
        );
        registry.register(
            "drishti_mem_vss_bytes",
            "Virtual memory size in bytes per process",
            mem_vss_bytes.clone(),
        );
        registry.register(
            "drishti_mem_page_faults_minor",
            "Minor page faults by process",
            mem_page_faults_minor_total.clone(),
        );
        registry.register(
            "drishti_mem_page_faults_major",
            "Major page faults by process",
            mem_page_faults_major_total.clone(),
        );
        registry.register(
            "drishti_mem_oom_kills",
            "OOM kill events",
            mem_oom_kills_total.clone(),
        );
        registry.register(
            "drishti_mem_available_bytes",
            "System available memory in bytes",
            mem_available_bytes.clone(),
        );
        registry.register(
            "drishti_mem_cache_bytes",
            "System page cache memory in bytes",
            mem_cache_bytes.clone(),
        );
        registry.register(
            "drishti_mem_total_bytes",
            "System total memory in bytes",
            mem_total_bytes.clone(),
        );
        registry.register(
            "drishti_collect_scrape_duration_ms",
            "Time spent collecting memory snapshots",
            scrape_duration_ms.clone(),
        );
        registry.register(
            "drishti_series_dropped",
            "Dropped metric series due to cardinality limits",
            series_dropped_total.clone(),
        );
        registry.register(
            "drishti_loader_failures",
            "Number of eBPF loader errors",
            loader_failures_total.clone(),
        );

        Self {
            registry: Mutex::new(registry),
            series_limiter: SeriesLimiter::new(max_series),
            cpu_run_time_ns_total,
            cpu_wait_time_ns_total,
            proc_lifecycle_total,
            mem_rss_bytes,
            mem_vss_bytes,
            mem_page_faults_minor_total,
            mem_page_faults_major_total,
            mem_oom_kills_total,
            mem_available_bytes,
            mem_cache_bytes,
            mem_total_bytes,
            scrape_duration_ms,
            series_dropped_total,
            loader_failures_total,
        }
    }

    pub fn record_cpu_runtime(&self, pid: u32, comm: &str, runtime_ns: u64) {
        let labels = ProcLabels {
            pid,
            comm: normalize_comm(comm),
        };
        if self.allow_series("drishti_cpu_run_time_ns", &labels) {
            self.cpu_run_time_ns_total
                .get_or_create(&labels)
                .inc_by(runtime_ns);
        }
    }

    pub fn record_cpu_wait(&self, pid: u32, comm: &str, wait_ns: u64) {
        let labels = ProcLabels {
            pid,
            comm: normalize_comm(comm),
        };
        if self.allow_series("drishti_cpu_wait_time_ns", &labels) {
            self.cpu_wait_time_ns_total
                .get_or_create(&labels)
                .inc_by(wait_ns);
        }
    }

    pub fn record_proc_lifecycle(&self, event: &str, pid: u32, comm: &str) {
        let labels = LifecycleLabels {
            event: event.to_string(),
            pid,
            comm: normalize_comm(comm),
        };

        if self.allow_series("drishti_proc_lifecycle", &labels) {
            self.proc_lifecycle_total.get_or_create(&labels).inc();
        }
    }

    pub fn record_oom(&self, pid: u32, comm: &str) {
        let labels = ProcLabels {
            pid,
            comm: normalize_comm(comm),
        };

        if self.allow_series("drishti_mem_oom_kills", &labels) {
            self.mem_oom_kills_total.get_or_create(&labels).inc();
        }
    }

    pub fn update_process_memory(
        &self,
        pid: u32,
        comm: &str,
        rss_bytes: u64,
        vss_bytes: u64,
        minor_faults: u64,
        major_faults: u64,
    ) {
        let labels = ProcLabels {
            pid,
            comm: normalize_comm(comm),
        };

        if !self.allow_series("drishti_mem_rss_bytes", &labels) {
            return;
        }

        self.mem_rss_bytes
            .get_or_create(&labels)
            .set(cast_i64(rss_bytes));
        self.mem_vss_bytes
            .get_or_create(&labels)
            .set(cast_i64(vss_bytes));
        self.mem_page_faults_minor_total
            .get_or_create(&labels)
            .inc_by(minor_faults);
        self.mem_page_faults_major_total
            .get_or_create(&labels)
            .inc_by(major_faults);
    }

    pub fn update_system_memory(&self, available_bytes: u64, cache_bytes: u64, total_bytes: u64) {
        self.mem_available_bytes.set(cast_i64(available_bytes));
        self.mem_cache_bytes.set(cast_i64(cache_bytes));
        self.mem_total_bytes.set(cast_i64(total_bytes));
    }

    pub fn observe_scrape_duration_ms(&self, duration_ms: f64) {
        self.scrape_duration_ms.observe(duration_ms);
    }

    pub fn record_loader_failure(&self) {
        self.loader_failures_total.inc();
    }

    pub fn render(&self) -> Result<String> {
        let mut output = String::new();
        let registry = self.lock_registry();
        encode(&mut output, &registry)?;
        Ok(output)
    }

    fn allow_series<T: core::fmt::Debug + ?Sized>(&self, metric_name: &str, labels: &T) -> bool {
        let key = format!("{metric_name}:{labels:?}");
        if self.series_limiter.allow(key) {
            true
        } else {
            self.series_dropped_total.inc();
            false
        }
    }

    fn lock_registry(&self) -> MutexGuard<'_, Registry> {
        self.registry
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

struct SeriesLimiter {
    max_series: usize,
    seen: Mutex<HashSet<String>>,
}

impl SeriesLimiter {
    fn new(max_series: usize) -> Self {
        Self {
            max_series,
            seen: Mutex::new(HashSet::new()),
        }
    }

    fn allow(&self, series_key: String) -> bool {
        let mut seen = self
            .seen
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if seen.contains(&series_key) {
            return true;
        }
        if seen.len() >= self.max_series {
            return false;
        }
        seen.insert(series_key);
        true
    }
}

fn normalize_comm(comm: &str) -> String {
    let mut normalized = String::with_capacity(comm.len());
    for ch in comm.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.' {
            normalized.push(ch);
        }
    }

    if normalized.is_empty() {
        "unknown".to_string()
    } else {
        normalized
    }
}

fn cast_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_comm_strips_invalid_characters() {
        assert_eq!(normalize_comm("nginx\0worker"), "nginxworker");
        assert_eq!(normalize_comm("***"), "unknown");
    }

    #[test]
    fn series_limit_drops_extra_series() {
        let metrics = AppMetrics::new(1);
        metrics.record_cpu_runtime(1, "proc-a", 10);
        metrics.record_cpu_runtime(2, "proc-b", 20);

        let rendered = metrics.render().expect("render should succeed");
        assert!(rendered.contains("drishti_series_dropped_total"));
        assert!(
            rendered.lines().any(
                |line| line.starts_with("drishti_series_dropped_total") && line.ends_with(" 1")
            )
        );
    }
}
