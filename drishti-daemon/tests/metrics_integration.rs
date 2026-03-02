use std::{
    env, fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use tempfile::tempdir;

struct HttpResponse {
    status_code: u16,
    body: String,
}

fn reserve_port() -> Option<u16> {
    match TcpListener::bind("127.0.0.1:0") {
        Ok(listener) => {
            let port = listener.local_addr().expect("addr must exist").port();
            drop(listener);
            Some(port)
        }
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!("skipping integration test: TCP bind not permitted in this environment");
            None
        }
        Err(err) => panic!("failed to reserve integration test port: {err}"),
    }
}

fn wait_for_health(port: u16) {
    let deadline = Instant::now() + Duration::from_secs(15);
    while Instant::now() < deadline {
        if let Ok(response) = http_get(port, "/healthz") {
            if response.status_code == 200 && response.body.contains("ok") {
                return;
            }
        }
        thread::sleep(Duration::from_millis(200));
    }
    panic!("daemon did not become healthy in time");
}

fn http_get(port: u16, path: &str) -> Result<HttpResponse, String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).map_err(|err| err.to_string())?;
    let request = format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .map_err(|err| err.to_string())?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|err| err.to_string())?;

    let mut sections = response.splitn(2, "\r\n\r\n");
    let headers = sections
        .next()
        .ok_or_else(|| "invalid HTTP response".to_string())?;
    let body = sections.next().unwrap_or_default().to_string();

    let status_line = headers
        .lines()
        .next()
        .ok_or_else(|| "missing status line".to_string())?;
    let status_code = status_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| "missing status code".to_string())?
        .parse::<u16>()
        .map_err(|err| err.to_string())?;

    Ok(HttpResponse { status_code, body })
}

fn write_config(
    path: &PathBuf,
    port: u16,
    cpu_enabled: bool,
    process_enabled: bool,
    network_enabled: bool,
    disk_enabled: bool,
    syscall_enabled: bool,
) {
    let config = format!(
        r#"
[daemon]
metrics_addr = "127.0.0.1:{port}"

[collectors]
cpu = {cpu_enabled}

[collectors.process]
enabled = {process_enabled}
track_threads = false

[collectors.memory]
enabled = true
poll_interval_ms = 250
track_oom = true

[collectors.network]
enabled = {network_enabled}
interfaces = []
tcp_rtt = true
tcp_retransmits = true

[collectors.disk]
enabled = {disk_enabled}
devices = []
latency_buckets_usec = [10,50,100,500,1000,5000,10000]

[collectors.syscall]
enabled = {syscall_enabled}
top_n = 20
latency_buckets_usec = [1,10,50,100,500,1000,5000]

[filters]
exclude_pids = []
exclude_comms = []
include_comms = []

[export]
max_series = 10000
"#
    );
    fs::write(path, config).expect("config write should succeed");
}

fn resolve_daemon_bin() -> PathBuf {
    for key in [
        "CARGO_BIN_EXE_drishti-daemon",
        "CARGO_BIN_EXE_drishti_daemon",
    ] {
        if let Ok(bin) = env::var(key) {
            let path = PathBuf::from(bin);
            if path.exists() {
                return path;
            }
        }
    }

    let current_exe = env::current_exe().expect("current_exe should be available");
    let fallback = current_exe
        .parent()
        .and_then(Path::parent)
        .map(|dir| {
            if cfg!(windows) {
                dir.join("drishti-daemon.exe")
            } else {
                dir.join("drishti-daemon")
            }
        })
        .expect("failed to derive fallback daemon binary path");

    if fallback.exists() {
        return fallback;
    }

    panic!(
        "failed to locate drishti-daemon binary; checked CARGO_BIN_EXE_* and {}",
        fallback.display()
    );
}

fn spawn_daemon(config_path: &PathBuf) -> Child {
    let bin = resolve_daemon_bin();

    Command::new(bin)
        .arg("--config")
        .arg(config_path)
        .env("DRISHTI_SYNTHETIC_EVENTS", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("daemon process should start")
}

fn stop_daemon(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn metrics_endpoint_exposes_core_series() {
    let temp = tempdir().expect("temp dir should be created");
    let config_path = temp.path().join("drishti.toml");
    let Some(port) = reserve_port() else {
        return;
    };
    write_config(&config_path, port, true, true, true, true, true);

    let mut daemon = spawn_daemon(&config_path);
    wait_for_health(port);

    let health = http_get(port, "/healthz").expect("health response should succeed");
    assert_eq!(health.status_code, 200);
    assert!(health.body.contains("ok"));

    let metrics = http_get(port, "/metrics").expect("metrics response should succeed");
    assert_eq!(metrics.status_code, 200);
    let metrics = metrics.body;

    assert!(metrics.contains("drishti_cpu_run_time_ns_total{"));
    assert!(metrics.contains("drishti_cpu_wait_time_ns_total{"));
    assert!(metrics.contains("drishti_proc_lifecycle_total{"));
    assert!(metrics.contains("drishti_mem_rss_bytes{"));
    assert!(metrics.contains("drishti_net_tx_bytes_total{"));
    assert!(metrics.contains("drishti_net_rx_bytes_total{"));
    assert!(metrics.contains("drishti_net_tcp_rtt_usec_sum{"));
    assert!(metrics.contains("drishti_disk_read_bytes_total{"));
    assert!(metrics.contains("drishti_disk_write_bytes_total{"));
    assert!(metrics.contains("drishti_disk_iops_total{"));
    assert!(metrics.contains("drishti_disk_io_latency_usec_sum{"));
    assert!(metrics.contains("drishti_disk_queue_depth{"));
    assert!(metrics.contains("drishti_syscall_count_total{"));
    assert!(metrics.contains("drishti_syscall_error_total{"));
    assert!(metrics.contains("drishti_syscall_latency_usec_sum{"));

    stop_daemon(&mut daemon);
}

#[test]
fn disabled_collectors_do_not_emit_cpu_process_network_disk_syscall_series() {
    let temp = tempdir().expect("temp dir should be created");
    let config_path = temp.path().join("drishti-disabled.toml");
    let Some(port) = reserve_port() else {
        return;
    };
    write_config(&config_path, port, false, false, false, false, false);

    let mut daemon = spawn_daemon(&config_path);
    wait_for_health(port);

    let health = http_get(port, "/healthz").expect("health response should succeed");
    assert_eq!(health.status_code, 200);
    assert!(health.body.contains("ok"));

    let metrics = http_get(port, "/metrics").expect("metrics response should succeed");
    assert_eq!(metrics.status_code, 200);
    let metrics = metrics.body;

    assert!(!metrics.contains("drishti_cpu_run_time_ns_total{"));
    assert!(!metrics.contains("drishti_proc_lifecycle_total{"));
    assert!(!metrics.contains("drishti_net_tx_bytes_total{"));
    assert!(!metrics.contains("drishti_net_rx_bytes_total{"));
    assert!(!metrics.contains("drishti_disk_read_bytes_total{"));
    assert!(!metrics.contains("drishti_disk_iops_total{"));
    assert!(!metrics.contains("drishti_syscall_count_total{"));
    assert!(!metrics.contains("drishti_syscall_error_total{"));
    assert!(!metrics.contains("drishti_syscall_latency_usec_sum{"));
    assert!(metrics.contains("drishti_mem_rss_bytes{"));

    stop_daemon(&mut daemon);
}
