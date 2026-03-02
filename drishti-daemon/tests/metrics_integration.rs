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
        if let Ok(body) = http_get(port, "/healthz") {
            if body.contains("ok") {
                return;
            }
        }
        thread::sleep(Duration::from_millis(200));
    }
    panic!("daemon did not become healthy in time");
}

fn http_get(port: u16, path: &str) -> Result<String, String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).map_err(|err| err.to_string())?;
    let request = format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .map_err(|err| err.to_string())?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|err| err.to_string())?;

    response
        .split("\r\n\r\n")
        .nth(1)
        .map(ToOwned::to_owned)
        .ok_or_else(|| "invalid HTTP response".to_string())
}

fn write_config(path: &PathBuf, port: u16, cpu_enabled: bool, process_enabled: bool) {
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
    write_config(&config_path, port, true, true);

    let mut daemon = spawn_daemon(&config_path);
    wait_for_health(port);

    let metrics = http_get(port, "/metrics").expect("metrics response should succeed");

    assert!(metrics.contains("drishti_cpu_run_time_ns_total{"));
    assert!(metrics.contains("drishti_cpu_wait_time_ns_total{"));
    assert!(metrics.contains("drishti_proc_lifecycle_total{"));
    assert!(metrics.contains("drishti_mem_rss_bytes{"));

    stop_daemon(&mut daemon);
}

#[test]
fn disabled_collectors_do_not_emit_cpu_process_series() {
    let temp = tempdir().expect("temp dir should be created");
    let config_path = temp.path().join("drishti-disabled.toml");
    let Some(port) = reserve_port() else {
        return;
    };
    write_config(&config_path, port, false, false);

    let mut daemon = spawn_daemon(&config_path);
    wait_for_health(port);

    let metrics = http_get(port, "/metrics").expect("metrics response should succeed");

    assert!(!metrics.contains("drishti_cpu_run_time_ns_total{"));
    assert!(!metrics.contains("drishti_proc_lifecycle_total{"));
    assert!(metrics.contains("drishti_mem_rss_bytes{"));

    stop_daemon(&mut daemon);
}
