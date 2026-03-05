use drishti_common::events::SyscallEvent;

use crate::aggregator::AppMetrics;

pub fn handle_syscall(metrics: &AppMetrics, event: &SyscallEvent) {
    // Use process id (tgid) to bound label cardinality across threads.
    metrics.record_syscall(
        event.syscall_nr,
        event.ret,
        event.latency_usec,
        event.tgid,
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

#[cfg(test)]
mod tests {
    use drishti_common::{COMM_LEN, events::EventKind};

    use super::*;

    #[test]
    fn syscall_labels_use_tgid_to_reduce_thread_cardinality() {
        let metrics = AppMetrics::new(10_000, 20, &[1, 10, 50]);
        let mut comm = [0u8; COMM_LEN];
        comm[..4].copy_from_slice(b"proc");

        let event = SyscallEvent {
            kind: EventKind::Syscall as u8,
            _pad0: [0; 3],
            pid: 7777,
            tgid: 1234,
            syscall_nr: 0,
            ret: 0,
            latency_usec: 5,
            comm,
        };

        handle_syscall(&metrics, &event);
        let rendered = metrics.render().expect("metrics should render");

        assert!(rendered.contains(
            "drishti_syscall_count_total{syscall=\"read\",pid=\"1234\",comm=\"proc\"} 1"
        ));
        assert!(!rendered.contains("pid=\"7777\""));
    }
}
