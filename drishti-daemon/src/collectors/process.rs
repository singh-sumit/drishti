use drishti_common::events::{OomKillEvent, ProcLifecycleEvent, ProcLifecycleKind};

use crate::aggregator::AppMetrics;

pub fn handle_lifecycle(metrics: &AppMetrics, event: &ProcLifecycleEvent) {
    let event_name = match event.lifecycle {
        x if x == ProcLifecycleKind::Fork as u8 => "fork",
        x if x == ProcLifecycleKind::Exec as u8 => "exec",
        x if x == ProcLifecycleKind::Exit as u8 => "exit",
        _ => "unknown",
    };

    metrics.record_proc_lifecycle(event_name, event.pid, &comm_to_string(&event.comm));
}

pub fn handle_oom(metrics: &AppMetrics, event: &OomKillEvent) {
    metrics.record_oom(event.pid, &comm_to_string(&event.comm));
}

fn comm_to_string(comm: &[u8]) -> String {
    let len = comm
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(comm.len());
    String::from_utf8_lossy(&comm[..len]).into_owned()
}
