pub mod aggregator;
pub mod collectors;
pub mod config;
pub mod exporter;
pub mod loader;
pub mod procfs;

use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result};
use clap::ValueEnum;
use collectors::memory::MemoryCollector;
use config::Config;
use tokio::{
    signal,
    sync::{mpsc, watch},
    task::JoinHandle,
};
use tracing::{error, info, warn};

use crate::{
    aggregator::AppMetrics,
    collectors::{ObservabilityEvent, run_event_consumer},
};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LogFormat {
    Text,
    Json,
}

#[derive(Debug, Clone)]
pub struct RunOptions {
    pub config_path: PathBuf,
    pub validate_config: bool,
    pub once: bool,
}

impl RunOptions {
    #[must_use]
    pub fn synthetic_events_enabled(&self) -> bool {
        std::env::var("DRISHTI_SYNTHETIC_EVENTS")
            .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    }
}

pub async fn run(options: RunOptions) -> Result<()> {
    let config = Config::from_path(&options.config_path)?;

    if options.validate_config {
        info!(path = %options.config_path.display(), "configuration validated successfully");
        return Ok(());
    }

    let _pid_file_guard = create_pid_file(&config.daemon.pid_file)?;

    let metrics = Arc::new(AppMetrics::new(
        config.export.max_series,
        config.collectors.syscall.top_n,
        &config.collectors.syscall.latency_buckets_usec,
    ));
    let (event_tx, event_rx) = mpsc::channel::<ObservabilityEvent>(8192);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    if options.once {
        loader::emit_synthetic_once(&event_tx, &config).await?;
        drop(event_tx);
        collectors::drain_events_once(event_rx, metrics.clone()).await;

        if config.collectors.memory.enabled {
            let mut memory_collector = MemoryCollector::new(config.clone(), metrics.clone());
            memory_collector.collect_once().await?;
        }

        println!("{}", metrics.render()?);
        return Ok(());
    }

    let (listen_addr, exporter_handle) = exporter::spawn(
        metrics.clone(),
        &config.daemon.metrics_addr,
        shutdown_rx.clone(),
    )
    .await
    .context("failed to start metrics exporter")?;
    info!(%listen_addr, "metrics endpoint ready");

    let event_handle = tokio::spawn(run_event_consumer(
        event_rx,
        metrics.clone(),
        shutdown_rx.clone(),
    ));

    let memory_handle = if config.collectors.memory.enabled {
        let memory_collector = MemoryCollector::new(config.clone(), metrics.clone());
        tokio::spawn(memory_collector.run(shutdown_rx.clone()))
    } else {
        tokio::spawn(async { Ok(()) })
    };

    let mut loader_handles = loader::start(
        config.clone(),
        event_tx,
        shutdown_rx.clone(),
        options.synthetic_events_enabled(),
    )
    .await
    .context("failed to initialize eBPF loader")?;

    wait_for_shutdown().await;
    let _ = shutdown_tx.send(true);

    join_and_log("exporter", exporter_handle).await;
    join_and_log("events", event_handle).await;
    join_and_log("memory", memory_handle).await;

    for (idx, handle) in loader_handles.drain(..).enumerate() {
        join_and_log(&format!("loader-{idx}"), handle).await;
    }

    Ok(())
}

async fn wait_for_shutdown() {
    if let Err(err) = signal::ctrl_c().await {
        error!(error = %err, "failed to install ctrl-c handler; shutting down immediately");
    }
}

async fn join_and_log(name: &str, handle: JoinHandle<Result<()>>) {
    match handle.await {
        Ok(Ok(())) => {}
        Ok(Err(err)) => error!(component = name, error = %err, "task failed"),
        Err(err) => error!(component = name, error = %err, "task panicked"),
    }
}

fn create_pid_file(path: &str) -> Result<PidFileGuard> {
    if path.is_empty() {
        return Ok(PidFileGuard::disabled());
    }

    let path = PathBuf::from(path);
    if let Some(parent) = path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            if is_permission_denied(&err) {
                warn!(
                    path = %path.display(),
                    error = %err,
                    "permission denied creating pid file directory; continuing without pid file"
                );
                return Ok(PidFileGuard::disabled());
            }
            return Err(err.into());
        }
    }
    if let Err(err) = fs::write(&path, std::process::id().to_string()) {
        if is_permission_denied(&err) {
            warn!(
                path = %path.display(),
                error = %err,
                "permission denied writing pid file; continuing without pid file"
            );
            return Ok(PidFileGuard::disabled());
        }
        return Err(err.into());
    }

    Ok(PidFileGuard::enabled(path))
}

struct PidFileGuard {
    path: Option<PathBuf>,
}

impl PidFileGuard {
    fn disabled() -> Self {
        Self { path: None }
    }

    fn enabled(path: PathBuf) -> Self {
        Self { path: Some(path) }
    }
}

impl Drop for PidFileGuard {
    fn drop(&mut self) {
        if let Some(path) = self.path.as_ref() {
            remove_if_exists(path);
        }
    }
}

fn remove_if_exists(path: &Path) {
    if path.exists() {
        let _ = fs::remove_file(path);
    }
}

fn is_permission_denied(err: &io::Error) -> bool {
    err.kind() == io::ErrorKind::PermissionDenied
}
