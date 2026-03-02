use drishti_common::events::{CpuRuntimeEvent, CpuWaitEvent};

use crate::aggregator::AppMetrics;

pub fn handle_runtime(metrics: &AppMetrics, event: &CpuRuntimeEvent) {
    metrics.record_cpu_runtime(event.pid, &comm_to_string(&event.comm), event.run_time_ns);
}

pub fn handle_wait(metrics: &AppMetrics, event: &CpuWaitEvent) {
    metrics.record_cpu_wait(event.pid, &comm_to_string(&event.comm), event.wait_time_ns);
}

fn comm_to_string(comm: &[u8]) -> String {
    let len = comm
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(comm.len());
    String::from_utf8_lossy(&comm[..len]).into_owned()
}
