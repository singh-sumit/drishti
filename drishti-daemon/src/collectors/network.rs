use drishti_common::events::{NetDirection, NetTrafficEvent, TcpRetransmitEvent, TcpRttEvent};

use crate::aggregator::AppMetrics;

pub fn handle_traffic(metrics: &AppMetrics, event: &NetTrafficEvent) {
    let comm = fixed_to_string(&event.comm);
    let iface = fixed_to_string(&event.iface);

    match event.direction {
        x if x == NetDirection::Tx as u8 => metrics.record_net_tx(
            event.pid,
            &comm,
            event.ifindex,
            &iface,
            event.bytes,
            event.packets,
        ),
        x if x == NetDirection::Rx as u8 => metrics.record_net_rx(
            event.pid,
            &comm,
            event.ifindex,
            &iface,
            event.bytes,
            event.packets,
        ),
        _ => {}
    }
}

pub fn handle_tcp_rtt(metrics: &AppMetrics, event: &TcpRttEvent) {
    let comm = fixed_to_string(&event.comm);
    let iface = fixed_to_string(&event.iface);
    metrics.record_tcp_rtt(
        event.pid,
        &comm,
        event.ifindex,
        &iface,
        u64::from(event.rtt_usec),
    );
}

pub fn handle_tcp_retransmit(metrics: &AppMetrics, event: &TcpRetransmitEvent) {
    let comm = fixed_to_string(&event.comm);
    let iface = fixed_to_string(&event.iface);
    metrics.record_tcp_retransmit(event.pid, &comm, event.ifindex, &iface);
}

fn fixed_to_string(value: &[u8]) -> String {
    let len = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    String::from_utf8_lossy(&value[..len]).into_owned()
}
