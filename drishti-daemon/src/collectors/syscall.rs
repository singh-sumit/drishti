use drishti_common::events::SyscallEvent;

use crate::aggregator::AppMetrics;

pub fn handle_syscall(metrics: &AppMetrics, event: &SyscallEvent) {
    metrics.record_syscall(
        event.syscall_nr,
        event.ret,
        event.latency_usec,
        event.pid,
        &fixed_to_string(&event.comm),
    );
}

fn fixed_to_string(value: &[u8]) -> String {
    let len = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    String::from_utf8_lossy(&value[..len]).into_owned()
}
