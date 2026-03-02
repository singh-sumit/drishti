use std::{
    ffi::CString,
    io::{Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use clap::Parser;
use drishti_daemon::config::Config;
use serde::Serialize;

#[cfg(feature = "ebpf-runtime")]
use drishti_daemon::loader;
#[cfg(feature = "ebpf-runtime")]
use tokio::sync::{mpsc, watch};

#[derive(Debug, Parser)]
#[command(
    name = "qemu_smoke",
    about = "QEMU guest smoke harness for drishti-daemon"
)]
struct Cli {
    #[arg(long, default_value = "/etc/drishti.toml")]
    config: PathBuf,
    #[arg(long, default_value = "/bin/drishti-daemon")]
    daemon_bin: PathBuf,
    #[arg(long, default_value = "127.0.0.1:9090")]
    metrics_addr: String,
    #[arg(long, default_value_t = 60)]
    timeout_secs: u64,
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    require_ebpf: bool,
}

#[derive(Debug, Serialize)]
struct SmokeSummary {
    result: &'static str,
    error_category: Option<&'static str>,
    message: String,
    loader_check: bool,
    healthz: bool,
    metrics_ok: bool,
    metrics_addr: String,
    duration_ms: u128,
}

#[tokio::main]
async fn main() {
    let started = Instant::now();
    let cli = Cli::parse();

    if let Err(err) = bootstrap_guest_mounts() {
        emit_result(
            SmokeSummary {
                result: "FAIL",
                error_category: Some("guest_bootstrap_failure"),
                message: format!("failed to initialize guest mounts: {err:#}"),
                loader_check: false,
                healthz: false,
                metrics_ok: false,
                metrics_addr: cli.metrics_addr.clone(),
                duration_ms: started.elapsed().as_millis(),
            },
            None,
        );
        std::process::exit(1);
    }

    let result = run_smoke(&cli, started).await;
    match result {
        Ok((summary, metrics)) => {
            emit_result(summary, Some(&metrics));
            std::process::exit(0);
        }
        Err((summary, metrics)) => {
            emit_result(summary, metrics.as_deref());
            std::process::exit(1);
        }
    }
}

async fn run_smoke(
    cli: &Cli,
    started: Instant,
) -> std::result::Result<(SmokeSummary, String), (SmokeSummary, Option<String>)> {
    let config = match Config::from_path(&cli.config) {
        Ok(config) => config,
        Err(err) => {
            return Err((
                SmokeSummary {
                    result: "FAIL",
                    error_category: Some("config_error"),
                    message: format!("failed to load config {}: {err:#}", cli.config.display()),
                    loader_check: false,
                    healthz: false,
                    metrics_ok: false,
                    metrics_addr: cli.metrics_addr.clone(),
                    duration_ms: started.elapsed().as_millis(),
                },
                None,
            ));
        }
    };

    if cli.require_ebpf {
        if let Err(err) = loader_check(config.clone()).await {
            return Err((
                SmokeSummary {
                    result: "FAIL",
                    error_category: Some("attach_load_failure"),
                    message: format!("loader check failed: {err:#}"),
                    loader_check: false,
                    healthz: false,
                    metrics_ok: false,
                    metrics_addr: cli.metrics_addr.clone(),
                    duration_ms: started.elapsed().as_millis(),
                },
                None,
            ));
        }
    }

    let mut daemon = match spawn_daemon(cli) {
        Ok(child) => child,
        Err(err) => {
            return Err((
                SmokeSummary {
                    result: "FAIL",
                    error_category: Some("daemon_startup_failure"),
                    message: format!(
                        "failed to spawn daemon {}: {err:#}",
                        cli.daemon_bin.display()
                    ),
                    loader_check: cli.require_ebpf,
                    healthz: false,
                    metrics_ok: false,
                    metrics_addr: cli.metrics_addr.clone(),
                    duration_ms: started.elapsed().as_millis(),
                },
                None,
            ));
        }
    };

    let wait_duration = Duration::from_secs(cli.timeout_secs);
    if let Err(err) = wait_for_healthz(&cli.metrics_addr, wait_duration) {
        stop_daemon(&mut daemon);
        return Err((
            SmokeSummary {
                result: "FAIL",
                error_category: Some("exporter_readiness_failure"),
                message: format!("healthz probe failed: {err:#}"),
                loader_check: cli.require_ebpf,
                healthz: false,
                metrics_ok: false,
                metrics_addr: cli.metrics_addr.clone(),
                duration_ms: started.elapsed().as_millis(),
            },
            None,
        ));
    }

    let metrics = match wait_for_metrics(&cli.metrics_addr, wait_duration) {
        Ok(metrics) => metrics,
        Err(err) => {
            stop_daemon(&mut daemon);
            return Err((
                SmokeSummary {
                    result: "FAIL",
                    error_category: Some("missing_metrics_family"),
                    message: format!("metrics validation failed: {err:#}"),
                    loader_check: cli.require_ebpf,
                    healthz: true,
                    metrics_ok: false,
                    metrics_addr: cli.metrics_addr.clone(),
                    duration_ms: started.elapsed().as_millis(),
                },
                None,
            ));
        }
    };

    stop_daemon(&mut daemon);

    let summary = SmokeSummary {
        result: "PASS",
        error_category: None,
        message: "QEMU smoke checks passed".to_string(),
        loader_check: cli.require_ebpf,
        healthz: true,
        metrics_ok: true,
        metrics_addr: cli.metrics_addr.clone(),
        duration_ms: started.elapsed().as_millis(),
    };

    Ok((summary, metrics))
}

fn emit_result(summary: SmokeSummary, metrics: Option<&str>) {
    println!("DRISHTI_QEMU_RESULT={}", summary.result);
    let summary_json = serde_json::to_string(&summary).unwrap_or_else(|_| {
        "{\"result\":\"FAIL\",\"error_category\":\"summary_encode_failure\"}".to_string()
    });
    println!("DRISHTI_QEMU_SUMMARY={summary_json}");

    if let Some(metrics_body) = metrics {
        println!("DRISHTI_METRICS_BEGIN");
        print!("{metrics_body}");
        if !metrics_body.ends_with('\n') {
            println!();
        }
        println!("DRISHTI_METRICS_END");
    }
}

#[cfg(feature = "ebpf-runtime")]
async fn loader_check(config: Config) -> Result<()> {
    let (event_tx, _event_rx) = mpsc::channel(128);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let handles = loader::start(config, event_tx, shutdown_rx, false)
        .await
        .context("loader::start returned an error")?;

    let _ = shutdown_tx.send(true);

    for handle in handles {
        match tokio::time::timeout(Duration::from_secs(5), handle).await {
            Ok(join_result) => {
                let task_result = join_result.context("loader task join failed")?;
                task_result.context("loader task returned failure")?;
            }
            Err(_) => bail!("loader task did not shut down within timeout"),
        }
    }

    Ok(())
}

#[cfg(not(feature = "ebpf-runtime"))]
async fn loader_check(_config: Config) -> Result<()> {
    bail!("qemu_smoke was built without ebpf-runtime feature")
}

fn spawn_daemon(cli: &Cli) -> Result<Child> {
    Command::new(&cli.daemon_bin)
        .arg("--config")
        .arg(&cli.config)
        .env("DRISHTI_SYNTHETIC_EVENTS", "1")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| format!("failed to launch {}", cli.daemon_bin.display()))
}

fn stop_daemon(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn wait_for_healthz(metrics_addr: &str, timeout: Duration) -> Result<()> {
    let deadline = Instant::now() + timeout;

    while Instant::now() < deadline {
        if let Ok(response) = http_get(metrics_addr, "/healthz") {
            if response.status_code == 200 && response.body.contains("ok") {
                return Ok(());
            }
        }
        std::thread::sleep(Duration::from_millis(250));
    }

    bail!("timed out waiting for /healthz at {metrics_addr}")
}

fn wait_for_metrics(metrics_addr: &str, timeout: Duration) -> Result<String> {
    let required_series = [
        "drishti_cpu_run_time_ns_total{",
        "drishti_proc_lifecycle_total{",
        "drishti_mem_rss_bytes{",
        "drishti_net_tx_bytes_total{",
        "drishti_disk_read_bytes_total{",
        "drishti_syscall_count_total{",
        "drishti_loader_failures_total",
    ];

    let deadline = Instant::now() + timeout;
    let mut last_body = String::new();

    while Instant::now() < deadline {
        if let Ok(response) = http_get(metrics_addr, "/metrics") {
            if response.status_code == 200 {
                if required_series
                    .iter()
                    .all(|series| response.body.contains(series))
                {
                    return Ok(response.body);
                }
                last_body = response.body;
            }
        }
        std::thread::sleep(Duration::from_millis(250));
    }

    bail!("timed out waiting for required metrics at {metrics_addr}; last payload:\n{last_body}")
}

struct HttpResponse {
    status_code: u16,
    body: String,
}

fn http_get(metrics_addr: &str, path: &str) -> Result<HttpResponse> {
    let mut stream = TcpStream::connect(metrics_addr)
        .with_context(|| format!("failed to connect to {metrics_addr}"))?;
    let request = format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .context("failed to write HTTP request")?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .context("failed to read HTTP response")?;

    let mut sections = response.splitn(2, "\r\n\r\n");
    let headers = sections.next().unwrap_or_default();
    let body = sections.next().unwrap_or_default().to_string();

    let status_line = headers.lines().next().context("missing HTTP status line")?;
    let status_code = status_line
        .split_whitespace()
        .nth(1)
        .context("missing HTTP status code")?
        .parse::<u16>()
        .context("invalid HTTP status code")?;

    Ok(HttpResponse { status_code, body })
}

fn bootstrap_guest_mounts() -> Result<()> {
    ensure_dir("/proc")?;
    ensure_dir("/sys")?;
    ensure_dir("/sys/kernel")?;
    ensure_dir("/sys/kernel/debug")?;
    ensure_dir("/sys/kernel/tracing")?;
    ensure_dir("/sys/fs")?;
    ensure_dir("/sys/fs/bpf")?;
    ensure_dir("/tmp")?;

    mount_if_needed("proc", "/proc", "proc", 0)?;
    mount_if_needed("sysfs", "/sys", "sysfs", 0)?;
    mount_if_needed("debugfs", "/sys/kernel/debug", "debugfs", 0)?;
    mount_if_needed("tracefs", "/sys/kernel/tracing", "tracefs", 0)?;
    mount_if_needed("bpf", "/sys/fs/bpf", "bpf", 0)?;
    bring_up_loopback()?;

    Ok(())
}

fn ensure_dir(path: &str) -> Result<()> {
    std::fs::create_dir_all(path).with_context(|| format!("failed to create directory {path}"))
}

fn mount_if_needed(source: &str, target: &str, fstype: &str, flags: libc::c_ulong) -> Result<()> {
    let source_c = CString::new(source).context("invalid mount source")?;
    let target_c = CString::new(target).context("invalid mount target")?;
    let fstype_c = CString::new(fstype).context("invalid mount fstype")?;

    // SAFETY: pointers are valid C strings and data pointer is null.
    let rc = unsafe {
        libc::mount(
            source_c.as_ptr(),
            target_c.as_ptr(),
            fstype_c.as_ptr(),
            flags,
            std::ptr::null(),
        )
    };

    if rc == 0 {
        return Ok(());
    }

    let err = std::io::Error::last_os_error();
    if err.raw_os_error() == Some(libc::EBUSY) {
        return Ok(());
    }

    Err(err).with_context(|| format!("failed to mount {fstype} at {target}"))
}

fn bring_up_loopback() -> Result<()> {
    const LOOPBACK_FLAGS: &str = "/sys/class/net/lo/flags";
    const LOOPBACK_TOOLS: &[(&str, &[&str])] = &[
        ("/bin/busybox", &["ip", "link", "set", "lo", "up"]),
        ("/bin/ip", &["link", "set", "lo", "up"]),
        ("/sbin/ip", &["link", "set", "lo", "up"]),
        ("/bin/busybox", &["ifconfig", "lo", "up"]),
        ("/sbin/ifconfig", &["lo", "up"]),
    ];

    if Path::new(LOOPBACK_FLAGS).exists() {
        if let Ok(raw_flags) = std::fs::read_to_string(LOOPBACK_FLAGS) {
            let parsed = raw_flags.trim().trim_start_matches("0x");
            if let Ok(flags) = u64::from_str_radix(parsed, 16) {
                if flags & (libc::IFF_UP as u64) != 0 {
                    return Ok(());
                }

                let updated = flags | (libc::IFF_UP as u64);
                if std::fs::write(LOOPBACK_FLAGS, format!("0x{updated:x}\n")).is_ok() {
                    return Ok(());
                }
            }
        }
    }

    for (tool, args) in LOOPBACK_TOOLS {
        if !Path::new(tool).exists() {
            continue;
        }

        if Command::new(tool)
            .args(*args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
        {
            return Ok(());
        }
    }

    bail!("failed to bring up loopback interface with ip/ifconfig helpers")
}
