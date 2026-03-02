use std::{
    collections::{HashMap, HashSet},
    sync::{Mutex, MutexGuard},
};

use anyhow::Result;
use prometheus_client::{
    encoding::{EncodeLabelSet, text::encode},
    metrics::{
        counter::Counter,
        family::{Family, MetricConstructor},
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

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct NetLabels {
    pub pid: u32,
    pub comm: String,
    pub ifindex: u32,
    pub iface: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct DiskProcLabels {
    pub pid: u32,
    pub comm: String,
    pub device: String,
    pub op: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct DiskDeviceLabels {
    pub device: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct SyscallLabels {
    pub syscall: String,
    pub pid: u32,
    pub comm: String,
}

#[derive(Clone)]
struct SyscallHistogramConstructor {
    buckets: Vec<f64>,
}

impl MetricConstructor<Histogram> for SyscallHistogramConstructor {
    fn new_metric(&self) -> Histogram {
        Histogram::new(self.buckets.clone())
    }
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
    net_tx_bytes_total: Family<NetLabels, Counter>,
    net_rx_bytes_total: Family<NetLabels, Counter>,
    net_tx_packets_total: Family<NetLabels, Counter>,
    net_rx_packets_total: Family<NetLabels, Counter>,
    net_tcp_rtt_usec: Family<NetLabels, Histogram>,
    net_tcp_retransmits_total: Family<NetLabels, Counter>,
    disk_read_bytes_total: Family<DiskProcLabels, Counter>,
    disk_write_bytes_total: Family<DiskProcLabels, Counter>,
    disk_iops_total: Family<DiskDeviceLabels, Counter>,
    disk_io_latency_usec: Family<DiskDeviceLabels, Histogram>,
    disk_queue_depth: Family<DiskDeviceLabels, Gauge>,
    syscall_count_total: Family<SyscallLabels, Counter>,
    syscall_error_total: Family<SyscallLabels, Counter>,
    syscall_latency_usec: Family<SyscallLabels, Histogram, SyscallHistogramConstructor>,
    syscall_top_n: usize,
    syscall_popularity: Mutex<HashMap<String, u64>>,
    scrape_duration_ms: Histogram,
    series_dropped_total: Counter,
    loader_failures_total: Counter,
}

impl AppMetrics {
    #[must_use]
    pub fn new(
        max_series: usize,
        syscall_top_n: usize,
        syscall_latency_buckets_usec: &[u64],
    ) -> Self {
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
        let net_tx_bytes_total = Family::default();
        let net_rx_bytes_total = Family::default();
        let net_tx_packets_total = Family::default();
        let net_rx_packets_total = Family::default();
        let net_tcp_rtt_usec: Family<NetLabels, Histogram> =
            Family::new_with_constructor(network_rtt_histogram as fn() -> Histogram);
        let net_tcp_retransmits_total = Family::default();
        let disk_read_bytes_total = Family::default();
        let disk_write_bytes_total = Family::default();
        let disk_iops_total = Family::default();
        let disk_io_latency_usec: Family<DiskDeviceLabels, Histogram> =
            Family::new_with_constructor(disk_latency_histogram as fn() -> Histogram);
        let disk_queue_depth = Family::default();
        let syscall_count_total = Family::default();
        let syscall_error_total = Family::default();
        let syscall_latency_usec = Family::new_with_constructor(SyscallHistogramConstructor {
            buckets: syscall_histogram_buckets(syscall_latency_buckets_usec),
        });
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
            "drishti_net_tx_bytes",
            "Transmitted bytes per process and interface",
            net_tx_bytes_total.clone(),
        );
        registry.register(
            "drishti_net_rx_bytes",
            "Received bytes per process and interface",
            net_rx_bytes_total.clone(),
        );
        registry.register(
            "drishti_net_tx_packets",
            "Transmitted packets per process and interface",
            net_tx_packets_total.clone(),
        );
        registry.register(
            "drishti_net_rx_packets",
            "Received packets per process and interface",
            net_rx_packets_total.clone(),
        );
        registry.register(
            "drishti_net_tcp_rtt_usec",
            "TCP round-trip-time observations in microseconds",
            net_tcp_rtt_usec.clone(),
        );
        registry.register(
            "drishti_net_tcp_retransmits",
            "TCP retransmission events",
            net_tcp_retransmits_total.clone(),
        );
        registry.register(
            "drishti_disk_read_bytes",
            "Disk read bytes per process and device",
            disk_read_bytes_total.clone(),
        );
        registry.register(
            "drishti_disk_write_bytes",
            "Disk write bytes per process and device",
            disk_write_bytes_total.clone(),
        );
        registry.register(
            "drishti_disk_iops",
            "Disk I/O operations per device",
            disk_iops_total.clone(),
        );
        registry.register(
            "drishti_disk_io_latency_usec",
            "Disk I/O completion latency per device in microseconds",
            disk_io_latency_usec.clone(),
        );
        registry.register(
            "drishti_disk_queue_depth",
            "In-flight disk requests per device",
            disk_queue_depth.clone(),
        );
        registry.register(
            "drishti_syscall_count",
            "Syscall invocation count grouped by syscall, pid, and comm",
            syscall_count_total.clone(),
        );
        registry.register(
            "drishti_syscall_error",
            "Syscall error count (ret < 0) grouped by syscall, pid, and comm",
            syscall_error_total.clone(),
        );
        registry.register(
            "drishti_syscall_latency_usec",
            "Syscall latency observations in microseconds",
            syscall_latency_usec.clone(),
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
            net_tx_bytes_total,
            net_rx_bytes_total,
            net_tx_packets_total,
            net_rx_packets_total,
            net_tcp_rtt_usec,
            net_tcp_retransmits_total,
            disk_read_bytes_total,
            disk_write_bytes_total,
            disk_iops_total,
            disk_io_latency_usec,
            disk_queue_depth,
            syscall_count_total,
            syscall_error_total,
            syscall_latency_usec,
            syscall_top_n,
            syscall_popularity: Mutex::new(HashMap::new()),
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

    #[allow(clippy::too_many_arguments)]
    pub fn record_net_tx(
        &self,
        pid: u32,
        comm: &str,
        ifindex: u32,
        iface: &str,
        bytes: u64,
        packets: u64,
    ) {
        let labels = NetLabels {
            pid,
            comm: normalize_comm(comm),
            ifindex,
            iface: normalize_interface(iface, ifindex),
        };

        if self.allow_series("drishti_net_tx_bytes", &labels) {
            self.net_tx_bytes_total.get_or_create(&labels).inc_by(bytes);
            self.net_tx_packets_total
                .get_or_create(&labels)
                .inc_by(packets);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_net_rx(
        &self,
        pid: u32,
        comm: &str,
        ifindex: u32,
        iface: &str,
        bytes: u64,
        packets: u64,
    ) {
        let labels = NetLabels {
            pid,
            comm: normalize_comm(comm),
            ifindex,
            iface: normalize_interface(iface, ifindex),
        };

        if self.allow_series("drishti_net_rx_bytes", &labels) {
            self.net_rx_bytes_total.get_or_create(&labels).inc_by(bytes);
            self.net_rx_packets_total
                .get_or_create(&labels)
                .inc_by(packets);
        }
    }

    pub fn record_tcp_rtt(&self, pid: u32, comm: &str, ifindex: u32, iface: &str, rtt_usec: u64) {
        let labels = NetLabels {
            pid,
            comm: normalize_comm(comm),
            ifindex,
            iface: normalize_interface(iface, ifindex),
        };

        if self.allow_series("drishti_net_tcp_rtt_usec", &labels) {
            self.net_tcp_rtt_usec
                .get_or_create(&labels)
                .observe(rtt_usec as f64);
        }
    }

    pub fn record_tcp_retransmit(&self, pid: u32, comm: &str, ifindex: u32, iface: &str) {
        let labels = NetLabels {
            pid,
            comm: normalize_comm(comm),
            ifindex,
            iface: normalize_interface(iface, ifindex),
        };

        if self.allow_series("drishti_net_tcp_retransmits", &labels) {
            self.net_tcp_retransmits_total.get_or_create(&labels).inc();
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_disk_io(
        &self,
        pid: u32,
        comm: &str,
        dev_major: u32,
        dev_minor: u32,
        operation: &str,
        bytes: u64,
        latency_usec: u64,
        queue_depth: u32,
    ) {
        let device = normalize_device(&format!("{dev_major}:{dev_minor}"));
        let op = normalize_op(operation);

        let proc_labels = DiskProcLabels {
            pid,
            comm: normalize_comm(comm),
            device: device.clone(),
            op: op.to_string(),
        };
        let device_labels = DiskDeviceLabels { device };

        if op == "read" {
            if self.allow_series("drishti_disk_read_bytes", &proc_labels) {
                self.disk_read_bytes_total
                    .get_or_create(&proc_labels)
                    .inc_by(bytes);
            }
        } else if op == "write" && self.allow_series("drishti_disk_write_bytes", &proc_labels) {
            self.disk_write_bytes_total
                .get_or_create(&proc_labels)
                .inc_by(bytes);
        }

        if self.allow_series("drishti_disk_iops", &device_labels) {
            self.disk_iops_total.get_or_create(&device_labels).inc();
            self.disk_io_latency_usec
                .get_or_create(&device_labels)
                .observe(latency_usec as f64);
            self.disk_queue_depth
                .get_or_create(&device_labels)
                .set(i64::from(queue_depth));
        }
    }

    pub fn record_syscall(
        &self,
        syscall_nr: i64,
        ret: i64,
        latency_usec: u64,
        pid: u32,
        comm: &str,
    ) {
        let syscall_name = resolve_syscall_name(syscall_nr);
        let syscall_label = self.project_syscall_label(&syscall_name);
        let labels = SyscallLabels {
            syscall: syscall_label,
            pid,
            comm: normalize_comm(comm),
        };

        if self.allow_series("drishti_syscall_count", &labels) {
            self.syscall_count_total.get_or_create(&labels).inc();
            if ret < 0 {
                self.syscall_error_total.get_or_create(&labels).inc();
            }
            self.syscall_latency_usec
                .get_or_create(&labels)
                .observe(latency_usec as f64);
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

    fn project_syscall_label(&self, syscall_name: &str) -> String {
        if self.syscall_top_n == 0 {
            return "other".to_string();
        }

        let mut popularity = self
            .syscall_popularity
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let entry = popularity.entry(syscall_name.to_string()).or_insert(0);
        *entry = entry.saturating_add(1);

        if popularity.len() <= self.syscall_top_n {
            return syscall_name.to_string();
        }

        let mut ranked: Vec<(&String, u64)> = popularity
            .iter()
            .map(|(name, count)| (name, *count))
            .collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));

        if ranked
            .iter()
            .take(self.syscall_top_n)
            .any(|(name, _)| name.as_str() == syscall_name)
        {
            syscall_name.to_string()
        } else {
            "other".to_string()
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

fn network_rtt_histogram() -> Histogram {
    Histogram::new(exponential_buckets(10.0, 2.0, 8))
}

fn disk_latency_histogram() -> Histogram {
    Histogram::new(exponential_buckets(10.0, 2.0, 10))
}

fn syscall_histogram_buckets(raw_buckets: &[u64]) -> Vec<f64> {
    let mut buckets: Vec<f64> = raw_buckets
        .iter()
        .copied()
        .filter(|value| *value > 0)
        .map(|value| value as f64)
        .collect();

    if buckets.is_empty() {
        buckets = vec![1.0, 10.0, 50.0, 100.0, 500.0, 1000.0, 5000.0];
    }

    buckets.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    buckets.dedup_by(|a, b| (*a - *b).abs() < f64::EPSILON);
    buckets
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

fn normalize_interface(iface: &str, ifindex: u32) -> String {
    let mut normalized = String::with_capacity(iface.len());
    for ch in iface.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.' {
            normalized.push(ch);
        }
    }

    if normalized.is_empty() {
        format!("if{ifindex}")
    } else {
        normalized
    }
}

fn normalize_device(device: &str) -> String {
    let mut normalized = String::with_capacity(device.len());
    for ch in device.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.' || ch == ':' {
            normalized.push(ch);
        }
    }

    if normalized.is_empty() {
        "unknown".to_string()
    } else {
        normalized
    }
}

fn normalize_op(operation: &str) -> &'static str {
    match operation {
        "read" => "read",
        "write" => "write",
        _ => "unknown",
    }
}

fn resolve_syscall_name(syscall_nr: i64) -> String {
    let name = match syscall_nr {
        0 => "read",
        1 => "write",
        2 => "open",
        3 => "close",
        9 => "mmap",
        10 => "mprotect",
        11 => "munmap",
        12 => "brk",
        16 => "ioctl",
        21 => "access",
        32 => "dup",
        39 => "getpid",
        41 => "socket",
        42 => "connect",
        44 => "sendto",
        45 => "recvfrom",
        49 => "bind",
        50 => "listen",
        51 => "getsockname",
        52 => "getpeername",
        53 => "socketpair",
        54 => "setsockopt",
        55 => "getsockopt",
        56 => "clone",
        57 => "fork",
        58 => "vfork",
        59 => "execve",
        60 => "exit",
        61 => "wait4",
        62 => "kill",
        63 => "uname",
        72 => "fcntl",
        78 => "getdents",
        79 => "getcwd",
        80 => "chdir",
        87 => "unlink",
        89 => "readlink",
        97 => "getrlimit",
        158 => "arch_prctl",
        202 => "futex",
        217 => "getdents64",
        231 => "exit_group",
        257 => "openat",
        262 => "newfstatat",
        318 => "getrandom",
        332 => "statx",
        _ => return format!("nr_{syscall_nr}"),
    };

    name.to_string()
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
        let metrics = AppMetrics::new(1, 20, &[1, 10, 50]);
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

    #[test]
    fn record_network_and_disk_metrics() {
        let metrics = AppMetrics::new(10_000, 20, &[1, 10, 50]);
        metrics.record_net_tx(42, "proc", 2, "eth0", 1000, 10);
        metrics.record_net_rx(42, "proc", 2, "eth0", 2000, 20);
        metrics.record_tcp_rtt(42, "proc", 2, "eth0", 55);
        metrics.record_tcp_retransmit(42, "proc", 2, "eth0");
        metrics.record_disk_io(42, "proc", 8, 0, "read", 4096, 120, 3);
        metrics.record_disk_io(42, "proc", 8, 0, "write", 2048, 90, 2);

        let rendered = metrics.render().expect("render should succeed");
        assert!(rendered.contains("drishti_net_tx_bytes_total{"));
        assert!(rendered.contains("drishti_net_rx_bytes_total{"));
        assert!(rendered.contains("drishti_net_tcp_rtt_usec_sum"));
        assert!(rendered.contains("drishti_net_tcp_retransmits_total{"));
        assert!(rendered.contains("drishti_disk_read_bytes_total{"));
        assert!(rendered.contains("drishti_disk_write_bytes_total{"));
        assert!(rendered.contains("drishti_disk_iops_total{"));
        assert!(rendered.contains("drishti_disk_io_latency_usec_sum{"));
        assert!(rendered.contains("drishti_disk_queue_depth{"));
    }

    #[test]
    fn record_syscall_metrics_and_top_n_collapse() {
        let metrics = AppMetrics::new(10_000, 1, &[1, 10, 50, 100]);

        for _ in 0..4 {
            metrics.record_syscall(0, 0, 10, 7, "proc-a");
        }
        metrics.record_syscall(9999, -1, 77, 7, "proc-a");

        let rendered = metrics.render().expect("render should succeed");
        assert!(rendered.contains("drishti_syscall_count_total{syscall=\"read\""));
        assert!(rendered.contains("drishti_syscall_count_total{syscall=\"other\""));
        assert!(rendered.contains("drishti_syscall_error_total{syscall=\"other\""));
        assert!(rendered.contains("drishti_syscall_latency_usec_sum{syscall=\"other\""));
    }

    #[test]
    fn syscall_name_fallback_is_deterministic() {
        assert_eq!(resolve_syscall_name(9999), "nr_9999");
        assert_eq!(resolve_syscall_name(0), "read");
    }
}
