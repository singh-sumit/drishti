use drishti_common::events::{DiskIoEvent, DiskOp};

use crate::aggregator::AppMetrics;

pub fn handle_disk_io(metrics: &AppMetrics, event: &DiskIoEvent) {
    let comm = fixed_to_string(&event.comm);
    let operation = match event.op {
        x if x == DiskOp::Read as u8 => "read",
        x if x == DiskOp::Write as u8 => "write",
        _ => "unknown",
    };

    metrics.record_disk_io(
        event.pid,
        &comm,
        event.dev_major,
        event.dev_minor,
        operation,
        event.bytes,
        event.latency_usec,
        event.queue_depth,
    );
}

fn fixed_to_string(value: &[u8]) -> String {
    let len = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    String::from_utf8_lossy(&value[..len]).into_owned()
}
