use std::{env, fs, path::Path};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub daemon: DaemonConfig,
    #[serde(default)]
    pub collectors: CollectorsConfig,
    #[serde(default)]
    pub filters: FiltersConfig,
    #[serde(default)]
    pub export: ExportConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    #[serde(default = "default_pid_file")]
    pub pid_file: String,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_metrics_addr")]
    pub metrics_addr: String,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            pid_file: default_pid_file(),
            log_level: default_log_level(),
            metrics_addr: default_metrics_addr(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorsConfig {
    #[serde(default = "default_true")]
    pub cpu: bool,
    #[serde(default)]
    pub memory: MemoryCollectorConfig,
    #[serde(default)]
    pub process: ProcessCollectorConfig,
    #[serde(default, deserialize_with = "deserialize_network_collector")]
    pub network: NetworkCollectorConfig,
    #[serde(default, deserialize_with = "deserialize_disk_collector")]
    pub disk: DiskCollectorConfig,
    #[serde(default, deserialize_with = "deserialize_syscall_collector")]
    pub syscall: SyscallCollectorConfig,
}

impl Default for CollectorsConfig {
    fn default() -> Self {
        Self {
            cpu: true,
            memory: MemoryCollectorConfig::default(),
            process: ProcessCollectorConfig::default(),
            network: NetworkCollectorConfig::default(),
            disk: DiskCollectorConfig::default(),
            syscall: SyscallCollectorConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessCollectorConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub track_threads: bool,
}

impl Default for ProcessCollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            track_threads: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryCollectorConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_memory_poll_ms")]
    pub poll_interval_ms: u64,
    #[serde(default = "default_true")]
    pub track_oom: bool,
}

impl Default for MemoryCollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            poll_interval_ms: default_memory_poll_ms(),
            track_oom: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkCollectorConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub interfaces: Vec<String>,
    #[serde(default = "default_true")]
    pub tcp_rtt: bool,
    #[serde(default = "default_true")]
    pub tcp_retransmits: bool,
}

impl Default for NetworkCollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interfaces: Vec::new(),
            tcp_rtt: true,
            tcp_retransmits: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskCollectorConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub devices: Vec<String>,
    #[serde(default = "default_latency_buckets_usec")]
    pub latency_buckets_usec: Vec<u64>,
}

impl Default for DiskCollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            devices: Vec::new(),
            latency_buckets_usec: default_latency_buckets_usec(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyscallCollectorConfig {
    #[serde(default = "default_false")]
    pub enabled: bool,
    #[serde(default = "default_syscall_top_n")]
    pub top_n: usize,
    #[serde(default = "default_syscall_latency_buckets_usec")]
    pub latency_buckets_usec: Vec<u64>,
}

impl Default for SyscallCollectorConfig {
    fn default() -> Self {
        Self {
            enabled: default_false(),
            top_n: default_syscall_top_n(),
            latency_buckets_usec: default_syscall_latency_buckets_usec(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FiltersConfig {
    #[serde(default)]
    pub exclude_pids: Vec<u32>,
    #[serde(default)]
    pub exclude_comms: Vec<String>,
    #[serde(default)]
    pub include_comms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    #[serde(default = "default_scrape_interval_ms")]
    pub scrape_interval_ms: u64,
    #[serde(default = "default_max_series")]
    pub max_series: usize,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            scrape_interval_ms: default_scrape_interval_ms(),
            max_series: default_max_series(),
        }
    }
}

impl Config {
    pub fn from_path(path: &Path) -> Result<Self> {
        let mut cfg = if path.exists() {
            let raw = fs::read_to_string(path)
                .with_context(|| format!("unable to read config file {}", path.display()))?;
            toml::from_str::<Self>(&raw)
                .with_context(|| format!("unable to parse TOML in {}", path.display()))?
        } else {
            Self::default()
        };

        apply_env_overrides(&mut cfg, env::vars());

        Ok(cfg)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum BoolOrNetworkConfig {
    Enabled(bool),
    Config(NetworkCollectorConfig),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum BoolOrDiskConfig {
    Enabled(bool),
    Config(DiskCollectorConfig),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum BoolOrSyscallConfig {
    Enabled(bool),
    Config(SyscallCollectorConfig),
}

fn deserialize_network_collector<'de, D>(
    deserializer: D,
) -> std::result::Result<NetworkCollectorConfig, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match Option::<BoolOrNetworkConfig>::deserialize(deserializer)? {
        None => Ok(NetworkCollectorConfig::default()),
        Some(BoolOrNetworkConfig::Enabled(enabled)) => Ok(NetworkCollectorConfig {
            enabled,
            ..NetworkCollectorConfig::default()
        }),
        Some(BoolOrNetworkConfig::Config(config)) => Ok(config),
    }
}

fn deserialize_disk_collector<'de, D>(
    deserializer: D,
) -> std::result::Result<DiskCollectorConfig, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match Option::<BoolOrDiskConfig>::deserialize(deserializer)? {
        None => Ok(DiskCollectorConfig::default()),
        Some(BoolOrDiskConfig::Enabled(enabled)) => Ok(DiskCollectorConfig {
            enabled,
            ..DiskCollectorConfig::default()
        }),
        Some(BoolOrDiskConfig::Config(config)) => Ok(config),
    }
}

fn deserialize_syscall_collector<'de, D>(
    deserializer: D,
) -> std::result::Result<SyscallCollectorConfig, D::Error>
where
    D: serde::Deserializer<'de>,
{
    match Option::<BoolOrSyscallConfig>::deserialize(deserializer)? {
        None => Ok(SyscallCollectorConfig::default()),
        Some(BoolOrSyscallConfig::Enabled(enabled)) => Ok(SyscallCollectorConfig {
            enabled,
            ..SyscallCollectorConfig::default()
        }),
        Some(BoolOrSyscallConfig::Config(config)) => Ok(config),
    }
}

fn apply_env_overrides<I>(cfg: &mut Config, vars: I)
where
    I: IntoIterator<Item = (String, String)>,
{
    for (key, value) in vars {
        if !key.starts_with("DRISHTI_") {
            continue;
        }

        let normalized = key.trim_start_matches("DRISHTI_").to_ascii_uppercase();

        match normalized.as_str() {
            "DAEMON__PID_FILE" => cfg.daemon.pid_file = value,
            "DAEMON__LOG_LEVEL" => cfg.daemon.log_level = value,
            "DAEMON__METRICS_ADDR" => cfg.daemon.metrics_addr = value,
            "COLLECTORS__CPU" => cfg.collectors.cpu = parse_bool(&value, cfg.collectors.cpu),
            "COLLECTORS__MEMORY" => {
                cfg.collectors.memory.enabled = parse_bool(&value, cfg.collectors.memory.enabled)
            }
            "COLLECTORS__PROCESS" => {
                cfg.collectors.process.enabled = parse_bool(&value, cfg.collectors.process.enabled)
            }
            "COLLECTORS__NETWORK" => {
                cfg.collectors.network.enabled = parse_bool(&value, cfg.collectors.network.enabled)
            }
            "COLLECTORS__DISK" => {
                cfg.collectors.disk.enabled = parse_bool(&value, cfg.collectors.disk.enabled)
            }
            "COLLECTORS__SYSCALL" => {
                cfg.collectors.syscall.enabled = parse_bool(&value, cfg.collectors.syscall.enabled)
            }
            "COLLECTORS__PROCESS__TRACK_THREADS" => {
                cfg.collectors.process.track_threads =
                    parse_bool(&value, cfg.collectors.process.track_threads)
            }
            "COLLECTORS__MEMORY__POLL_INTERVAL_MS" => {
                cfg.collectors.memory.poll_interval_ms = value
                    .parse()
                    .unwrap_or(cfg.collectors.memory.poll_interval_ms)
            }
            "COLLECTORS__MEMORY__TRACK_OOM" => {
                cfg.collectors.memory.track_oom =
                    parse_bool(&value, cfg.collectors.memory.track_oom)
            }
            "COLLECTORS__NETWORK__INTERFACES" => {
                cfg.collectors.network.interfaces = parse_string_list(&value);
            }
            "COLLECTORS__NETWORK__TCP_RTT" => {
                cfg.collectors.network.tcp_rtt = parse_bool(&value, cfg.collectors.network.tcp_rtt)
            }
            "COLLECTORS__NETWORK__TCP_RETRANSMITS" => {
                cfg.collectors.network.tcp_retransmits =
                    parse_bool(&value, cfg.collectors.network.tcp_retransmits)
            }
            "COLLECTORS__DISK__DEVICES" => {
                cfg.collectors.disk.devices = parse_string_list(&value);
            }
            "COLLECTORS__DISK__LATENCY_BUCKETS_USEC" => {
                cfg.collectors.disk.latency_buckets_usec = parse_u64_list(&value);
            }
            "COLLECTORS__SYSCALL__TOP_N" => {
                cfg.collectors.syscall.top_n = value.parse().unwrap_or(cfg.collectors.syscall.top_n)
            }
            "COLLECTORS__SYSCALL__LATENCY_BUCKETS_USEC" => {
                cfg.collectors.syscall.latency_buckets_usec = parse_u64_list(&value);
            }
            "FILTERS__EXCLUDE_PIDS" => {
                cfg.filters.exclude_pids = parse_u32_list(&value);
            }
            "FILTERS__EXCLUDE_COMMS" => {
                cfg.filters.exclude_comms = parse_string_list(&value);
            }
            "FILTERS__INCLUDE_COMMS" => {
                cfg.filters.include_comms = parse_string_list(&value);
            }
            "EXPORT__SCRAPE_INTERVAL_MS" => {
                cfg.export.scrape_interval_ms =
                    value.parse().unwrap_or(cfg.export.scrape_interval_ms)
            }
            "EXPORT__MAX_SERIES" => {
                cfg.export.max_series = value.parse().unwrap_or(cfg.export.max_series)
            }
            _ => {}
        }
    }
}

fn parse_bool(value: &str, default: bool) -> bool {
    match value.to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => true,
        "false" | "0" | "no" | "off" => false,
        _ => default,
    }
}

fn parse_string_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn parse_u32_list(value: &str) -> Vec<u32> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .filter_map(|item| item.parse::<u32>().ok())
        .collect()
}

fn parse_u64_list(value: &str) -> Vec<u64> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .filter_map(|item| item.parse::<u64>().ok())
        .collect()
}

fn default_pid_file() -> String {
    "/var/run/drishti.pid".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_metrics_addr() -> String {
    "0.0.0.0:9090".to_string()
}

const fn default_true() -> bool {
    true
}

const fn default_false() -> bool {
    false
}

const fn default_memory_poll_ms() -> u64 {
    1000
}

const fn default_scrape_interval_ms() -> u64 {
    1000
}

const fn default_max_series() -> usize {
    10_000
}

fn default_latency_buckets_usec() -> Vec<u64> {
    vec![10, 50, 100, 500, 1000, 5000, 10_000]
}

const fn default_syscall_top_n() -> usize {
    20
}

fn default_syscall_latency_buckets_usec() -> Vec<u64> {
    vec![1, 10, 50, 100, 500, 1000, 5000]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_defaults_from_empty_config() {
        let cfg: Config = toml::from_str("").expect("empty config should parse");
        assert!(cfg.collectors.cpu);
        assert!(cfg.collectors.memory.enabled);
        assert!(cfg.collectors.network.enabled);
        assert!(cfg.collectors.disk.enabled);
        assert!(!cfg.collectors.syscall.enabled);
        assert!(cfg.collectors.network.interfaces.is_empty());
        assert!(cfg.collectors.disk.devices.is_empty());
        assert_eq!(cfg.collectors.syscall.top_n, 20);
        assert_eq!(
            cfg.collectors.syscall.latency_buckets_usec,
            vec![1, 10, 50, 100, 500, 1000, 5000]
        );
        assert_eq!(cfg.daemon.metrics_addr, "0.0.0.0:9090");
        assert_eq!(cfg.export.max_series, 10_000);
    }

    #[test]
    fn env_overrides_apply() {
        let mut cfg = Config::default();

        apply_env_overrides(
            &mut cfg,
            vec![
                ("DRISHTI_DAEMON__LOG_LEVEL".to_string(), "debug".to_string()),
                (
                    "DRISHTI_COLLECTORS__MEMORY__POLL_INTERVAL_MS".to_string(),
                    "250".to_string(),
                ),
                (
                    "DRISHTI_FILTERS__EXCLUDE_COMMS".to_string(),
                    "kworker,init".to_string(),
                ),
                ("DRISHTI_EXPORT__MAX_SERIES".to_string(), "42".to_string()),
                (
                    "DRISHTI_COLLECTORS__NETWORK".to_string(),
                    "false".to_string(),
                ),
                (
                    "DRISHTI_COLLECTORS__NETWORK__INTERFACES".to_string(),
                    "eth0,wlan0".to_string(),
                ),
                (
                    "DRISHTI_COLLECTORS__DISK__DEVICES".to_string(),
                    "sda,nvme0n1".to_string(),
                ),
                (
                    "DRISHTI_COLLECTORS__DISK__LATENCY_BUCKETS_USEC".to_string(),
                    "5,25,100".to_string(),
                ),
                (
                    "DRISHTI_COLLECTORS__SYSCALL".to_string(),
                    "true".to_string(),
                ),
                (
                    "DRISHTI_COLLECTORS__SYSCALL__TOP_N".to_string(),
                    "10".to_string(),
                ),
                (
                    "DRISHTI_COLLECTORS__SYSCALL__LATENCY_BUCKETS_USEC".to_string(),
                    "2,20,200".to_string(),
                ),
            ],
        );

        assert_eq!(cfg.daemon.log_level, "debug");
        assert_eq!(cfg.collectors.memory.poll_interval_ms, 250);
        assert_eq!(cfg.filters.exclude_comms, vec!["kworker", "init"]);
        assert_eq!(cfg.export.max_series, 42);
        assert!(!cfg.collectors.network.enabled);
        assert_eq!(cfg.collectors.network.interfaces, vec!["eth0", "wlan0"]);
        assert_eq!(cfg.collectors.disk.devices, vec!["sda", "nvme0n1"]);
        assert_eq!(cfg.collectors.disk.latency_buckets_usec, vec![5, 25, 100]);
        assert!(cfg.collectors.syscall.enabled);
        assert_eq!(cfg.collectors.syscall.top_n, 10);
        assert_eq!(
            cfg.collectors.syscall.latency_buckets_usec,
            vec![2, 20, 200]
        );
    }

    #[test]
    fn parse_collector_bool_shorthand() {
        let cfg: Config = toml::from_str(
            r#"
            [collectors]
            network = false
            disk = true
            syscall = true
            "#,
        )
        .expect("collector shorthand should parse");

        assert!(!cfg.collectors.network.enabled);
        assert!(cfg.collectors.disk.enabled);
        assert!(cfg.collectors.syscall.enabled);
    }
}
