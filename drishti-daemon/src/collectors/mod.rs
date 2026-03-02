pub mod cpu;
pub mod memory;
pub mod process;

use std::sync::Arc;

use anyhow::Result;
use drishti_common::events::{CpuRuntimeEvent, CpuWaitEvent, OomKillEvent, ProcLifecycleEvent};
use tokio::sync::{mpsc, watch};

use crate::aggregator::AppMetrics;

#[derive(Debug, Clone)]
pub enum ObservabilityEvent {
    CpuRuntime(CpuRuntimeEvent),
    CpuWait(CpuWaitEvent),
    ProcLifecycle(ProcLifecycleEvent),
    OomKill(OomKillEvent),
}

pub async fn run_event_consumer(
    mut event_rx: mpsc::Receiver<ObservabilityEvent>,
    metrics: Arc<AppMetrics>,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<()> {
    loop {
        tokio::select! {
            changed = shutdown_rx.changed() => {
                if changed.is_ok() && *shutdown_rx.borrow() {
                    break;
                }
            }
            event = event_rx.recv() => {
                let Some(event) = event else {
                    break;
                };
                handle_event(&metrics, event);
            }
        }
    }

    Ok(())
}

pub async fn drain_events_once(
    mut event_rx: mpsc::Receiver<ObservabilityEvent>,
    metrics: Arc<AppMetrics>,
) {
    while let Some(event) = event_rx.recv().await {
        handle_event(&metrics, event);
    }
}

fn handle_event(metrics: &AppMetrics, event: ObservabilityEvent) {
    match event {
        ObservabilityEvent::CpuRuntime(event) => cpu::handle_runtime(metrics, &event),
        ObservabilityEvent::CpuWait(event) => cpu::handle_wait(metrics, &event),
        ObservabilityEvent::ProcLifecycle(event) => process::handle_lifecycle(metrics, &event),
        ObservabilityEvent::OomKill(event) => process::handle_oom(metrics, &event),
    }
}
