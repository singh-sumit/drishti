use std::time::Duration;

use anyhow::Result;
use drishti_common::{
    COMM_LEN, IFACE_LEN,
    events::{
        CpuRuntimeEvent, CpuWaitEvent, DiskIoEvent, DiskOp, EventKind, NetDirection,
        NetTrafficEvent, OomKillEvent, ProcLifecycleEvent, ProcLifecycleKind, SyscallEvent,
        TcpRetransmitEvent, TcpRttEvent,
    },
};
use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
};
use tracing::info;

use crate::{collectors::ObservabilityEvent, config::Config};

pub async fn start(
    config: Config,
    event_tx: mpsc::Sender<ObservabilityEvent>,
    shutdown_rx: watch::Receiver<bool>,
    synthetic_events: bool,
) -> Result<Vec<JoinHandle<Result<()>>>> {
    if synthetic_events {
        info!("starting synthetic eBPF event stream");
        return Ok(vec![tokio::spawn(run_synthetic_stream(
            event_tx,
            shutdown_rx,
            config.collectors.cpu,
            config.collectors.process.enabled,
            config.collectors.memory.track_oom,
            config.collectors.network.enabled,
            config.collectors.network.tcp_rtt,
            config.collectors.network.tcp_retransmits,
            config.collectors.disk.enabled,
            config.collectors.syscall.enabled,
        ))]);
    }

    #[cfg(feature = "ebpf-runtime")]
    {
        return ebpf_runtime::start_real_ebpf(config, event_tx, shutdown_rx).await;
    }

    #[cfg(not(feature = "ebpf-runtime"))]
    {
        let _ = config;
        let _ = event_tx;
        let _ = shutdown_rx;
        tracing::warn!(
            "compiled without ebpf-runtime feature; running without kernel event collection"
        );
        Ok(Vec::new())
    }
}

pub async fn emit_synthetic_once(
    event_tx: &mpsc::Sender<ObservabilityEvent>,
    config: &Config,
) -> Result<()> {
    if config.collectors.cpu {
        event_tx
            .send(ObservabilityEvent::CpuRuntime(CpuRuntimeEvent {
                kind: EventKind::CpuRuntime as u8,
                _pad0: [0; 3],
                pid: 4242,
                tgid: 4242,
                cpu: 0,
                run_time_ns: 120_000,
                comm: fixed_from_str::<COMM_LEN>("synthetic-cpu"),
            }))
            .await?;

        event_tx
            .send(ObservabilityEvent::CpuWait(CpuWaitEvent {
                kind: EventKind::CpuWait as u8,
                _pad0: [0; 3],
                pid: 4242,
                tgid: 4242,
                cpu: 0,
                wait_time_ns: 64_000,
                comm: fixed_from_str::<COMM_LEN>("synthetic-cpu"),
            }))
            .await?;
    }

    if config.collectors.process.enabled {
        event_tx
            .send(ObservabilityEvent::ProcLifecycle(ProcLifecycleEvent {
                kind: EventKind::ProcLifecycle as u8,
                lifecycle: ProcLifecycleKind::Exec as u8,
                _pad0: [0; 2],
                pid: 4242,
                tgid: 4242,
                ppid: 1,
                exit_code: 0,
                comm: fixed_from_str::<COMM_LEN>("synthetic-proc"),
            }))
            .await?;
    }

    if config.collectors.memory.track_oom {
        event_tx
            .send(ObservabilityEvent::OomKill(OomKillEvent {
                kind: EventKind::OomKill as u8,
                _pad0: [0; 3],
                pid: 4242,
                tgid: 4242,
                pages: 1,
                comm: fixed_from_str::<COMM_LEN>("synthetic-oom"),
            }))
            .await?;
    }

    if config.collectors.network.enabled {
        event_tx
            .send(ObservabilityEvent::NetTraffic(NetTrafficEvent {
                kind: EventKind::NetTraffic as u8,
                direction: NetDirection::Tx as u8,
                _pad0: [0; 2],
                pid: 4242,
                tgid: 4242,
                ifindex: 2,
                bytes: 2048,
                packets: 8,
                comm: fixed_from_str::<COMM_LEN>("synthetic-net"),
                iface: fixed_from_str::<IFACE_LEN>("eth0"),
            }))
            .await?;

        event_tx
            .send(ObservabilityEvent::NetTraffic(NetTrafficEvent {
                kind: EventKind::NetTraffic as u8,
                direction: NetDirection::Rx as u8,
                _pad0: [0; 2],
                pid: 4242,
                tgid: 4242,
                ifindex: 2,
                bytes: 1024,
                packets: 5,
                comm: fixed_from_str::<COMM_LEN>("synthetic-net"),
                iface: fixed_from_str::<IFACE_LEN>("eth0"),
            }))
            .await?;

        if config.collectors.network.tcp_rtt {
            event_tx
                .send(ObservabilityEvent::TcpRtt(TcpRttEvent {
                    kind: EventKind::TcpRtt as u8,
                    _pad0: [0; 3],
                    pid: 4242,
                    tgid: 4242,
                    ifindex: 2,
                    rtt_usec: 120,
                    comm: fixed_from_str::<COMM_LEN>("synthetic-net"),
                    iface: fixed_from_str::<IFACE_LEN>("eth0"),
                }))
                .await?;
        }

        if config.collectors.network.tcp_retransmits {
            event_tx
                .send(ObservabilityEvent::TcpRetransmit(TcpRetransmitEvent {
                    kind: EventKind::TcpRetransmit as u8,
                    _pad0: [0; 3],
                    pid: 4242,
                    tgid: 4242,
                    ifindex: 2,
                    comm: fixed_from_str::<COMM_LEN>("synthetic-net"),
                    iface: fixed_from_str::<IFACE_LEN>("eth0"),
                }))
                .await?;
        }
    }

    if config.collectors.disk.enabled {
        event_tx
            .send(ObservabilityEvent::DiskIo(DiskIoEvent {
                kind: EventKind::DiskIo as u8,
                op: DiskOp::Read as u8,
                _pad0: [0; 2],
                pid: 4242,
                tgid: 4242,
                dev_major: 8,
                dev_minor: 0,
                bytes: 4096,
                latency_usec: 350,
                queue_depth: 2,
                comm: fixed_from_str::<COMM_LEN>("synthetic-disk"),
            }))
            .await?;

        event_tx
            .send(ObservabilityEvent::DiskIo(DiskIoEvent {
                kind: EventKind::DiskIo as u8,
                op: DiskOp::Write as u8,
                _pad0: [0; 2],
                pid: 4242,
                tgid: 4242,
                dev_major: 8,
                dev_minor: 0,
                bytes: 2048,
                latency_usec: 210,
                queue_depth: 1,
                comm: fixed_from_str::<COMM_LEN>("synthetic-disk"),
            }))
            .await?;
    }

    if config.collectors.syscall.enabled {
        event_tx
            .send(ObservabilityEvent::Syscall(SyscallEvent {
                kind: EventKind::Syscall as u8,
                _pad0: [0; 3],
                pid: 4242,
                tgid: 4242,
                syscall_nr: 0,
                ret: 64,
                latency_usec: 22,
                comm: fixed_from_str::<COMM_LEN>("synthetic-syscall"),
            }))
            .await?;

        event_tx
            .send(ObservabilityEvent::Syscall(SyscallEvent {
                kind: EventKind::Syscall as u8,
                _pad0: [0; 3],
                pid: 4242,
                tgid: 4242,
                syscall_nr: 257,
                ret: -2,
                latency_usec: 31,
                comm: fixed_from_str::<COMM_LEN>("synthetic-syscall"),
            }))
            .await?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn run_synthetic_stream(
    event_tx: mpsc::Sender<ObservabilityEvent>,
    mut shutdown_rx: watch::Receiver<bool>,
    cpu_enabled: bool,
    process_enabled: bool,
    oom_enabled: bool,
    network_enabled: bool,
    network_rtt_enabled: bool,
    network_retransmits_enabled: bool,
    disk_enabled: bool,
    syscall_enabled: bool,
) -> Result<()> {
    let mut interval = tokio::time::interval(Duration::from_millis(250));
    let mut tick: u64 = 0;

    loop {
        tokio::select! {
            changed = shutdown_rx.changed() => {
                if changed.is_ok() && *shutdown_rx.borrow() {
                    break;
                }
            }
            _ = interval.tick() => {
                tick = tick.saturating_add(1);
                if cpu_enabled {
                    event_tx.send(ObservabilityEvent::CpuRuntime(CpuRuntimeEvent {
                        kind: EventKind::CpuRuntime as u8,
                        _pad0: [0; 3],
                        pid: 1200,
                        tgid: 1200,
                        cpu: (tick % 2) as u32,
                        run_time_ns: 100_000 + tick,
                        comm: fixed_from_str::<COMM_LEN>("synthetic-cpu"),
                    })).await?;

                    event_tx.send(ObservabilityEvent::CpuWait(CpuWaitEvent {
                        kind: EventKind::CpuWait as u8,
                        _pad0: [0; 3],
                        pid: 1200,
                        tgid: 1200,
                        cpu: (tick % 2) as u32,
                        wait_time_ns: 50_000 + tick,
                        comm: fixed_from_str::<COMM_LEN>("synthetic-cpu"),
                    })).await?;
                }

                if process_enabled {
                    event_tx.send(ObservabilityEvent::ProcLifecycle(ProcLifecycleEvent {
                        kind: EventKind::ProcLifecycle as u8,
                        lifecycle: ProcLifecycleKind::Fork as u8,
                        _pad0: [0; 2],
                        pid: 1200,
                        tgid: 1200,
                        ppid: 1,
                        exit_code: 0,
                        comm: fixed_from_str::<COMM_LEN>("synthetic-proc"),
                    })).await?;
                }

                if oom_enabled && tick % 8 == 0 {
                    event_tx.send(ObservabilityEvent::OomKill(OomKillEvent {
                        kind: EventKind::OomKill as u8,
                        _pad0: [0; 3],
                        pid: 1200,
                        tgid: 1200,
                        pages: 32,
                        comm: fixed_from_str::<COMM_LEN>("synthetic-oom"),
                    })).await?;
                }

                if network_enabled {
                    let ifindex = if tick % 2 == 0 { 2 } else { 3 };
                    let iface = if ifindex == 2 { "eth0" } else { "wlan0" };

                    event_tx.send(ObservabilityEvent::NetTraffic(NetTrafficEvent {
                        kind: EventKind::NetTraffic as u8,
                        direction: NetDirection::Tx as u8,
                        _pad0: [0; 2],
                        pid: 1200,
                        tgid: 1200,
                        ifindex,
                        bytes: 2_000 + tick,
                        packets: 4 + (tick % 3),
                        comm: fixed_from_str::<COMM_LEN>("synthetic-net"),
                        iface: fixed_from_str::<IFACE_LEN>(iface),
                    })).await?;

                    event_tx.send(ObservabilityEvent::NetTraffic(NetTrafficEvent {
                        kind: EventKind::NetTraffic as u8,
                        direction: NetDirection::Rx as u8,
                        _pad0: [0; 2],
                        pid: 1200,
                        tgid: 1200,
                        ifindex,
                        bytes: 1_000 + tick,
                        packets: 3 + (tick % 2),
                        comm: fixed_from_str::<COMM_LEN>("synthetic-net"),
                        iface: fixed_from_str::<IFACE_LEN>(iface),
                    })).await?;

                    if network_rtt_enabled {
                        event_tx.send(ObservabilityEvent::TcpRtt(TcpRttEvent {
                            kind: EventKind::TcpRtt as u8,
                            _pad0: [0; 3],
                            pid: 1200,
                            tgid: 1200,
                            ifindex,
                            rtt_usec: 80 + (tick % 20) as u32,
                            comm: fixed_from_str::<COMM_LEN>("synthetic-net"),
                            iface: fixed_from_str::<IFACE_LEN>(iface),
                        })).await?;
                    }

                    if network_retransmits_enabled && tick % 4 == 0 {
                        event_tx.send(ObservabilityEvent::TcpRetransmit(TcpRetransmitEvent {
                            kind: EventKind::TcpRetransmit as u8,
                            _pad0: [0; 3],
                            pid: 1200,
                            tgid: 1200,
                            ifindex,
                            comm: fixed_from_str::<COMM_LEN>("synthetic-net"),
                            iface: fixed_from_str::<IFACE_LEN>(iface),
                        })).await?;
                    }
                }

                if disk_enabled {
                    event_tx
                        .send(ObservabilityEvent::DiskIo(DiskIoEvent {
                            kind: EventKind::DiskIo as u8,
                            op: DiskOp::Read as u8,
                            _pad0: [0; 2],
                            pid: 1200,
                            tgid: 1200,
                            dev_major: 8,
                            dev_minor: 0,
                            bytes: 4096 + tick,
                            latency_usec: 120 + tick,
                            queue_depth: (tick % 8) as u32,
                            comm: fixed_from_str::<COMM_LEN>("synthetic-disk"),
                        }))
                        .await?;

                    event_tx
                        .send(ObservabilityEvent::DiskIo(DiskIoEvent {
                            kind: EventKind::DiskIo as u8,
                            op: DiskOp::Write as u8,
                            _pad0: [0; 2],
                            pid: 1200,
                            tgid: 1200,
                            dev_major: 8,
                            dev_minor: 0,
                            bytes: 2048 + tick,
                            latency_usec: 140 + tick,
                            queue_depth: ((tick + 1) % 8) as u32,
                            comm: fixed_from_str::<COMM_LEN>("synthetic-disk"),
                        }))
                        .await?;
                }

                if syscall_enabled {
                    event_tx.send(ObservabilityEvent::Syscall(SyscallEvent {
                        kind: EventKind::Syscall as u8,
                        _pad0: [0; 3],
                        pid: 1200,
                        tgid: 1200,
                        syscall_nr: if tick % 2 == 0 { 0 } else { 257 },
                        ret: if tick % 3 == 0 { -1 } else { 0 },
                        latency_usec: 10 + tick,
                        comm: fixed_from_str::<COMM_LEN>("synthetic-syscall"),
                    })).await?;
                }
            }
        }
    }

    Ok(())
}

fn fixed_from_str<const N: usize>(value: &str) -> [u8; N] {
    let mut field = [0u8; N];
    let bytes = value.as_bytes();
    let copy_len = bytes.len().min(N);
    field[..copy_len].copy_from_slice(&bytes[..copy_len]);
    field
}

#[cfg(feature = "ebpf-runtime")]
mod ebpf_runtime {
    use std::{mem, time::Duration};

    use anyhow::{Context, Result};
    use aya::{Ebpf, maps::ring_buf::RingBuf, programs::TracePoint};
    use drishti_common::events::{
        CpuRuntimeEvent, CpuWaitEvent, DiskIoEvent, EventKind, NetTrafficEvent, OomKillEvent,
        ProcLifecycleEvent, SyscallEvent, TcpRetransmitEvent, TcpRttEvent,
    };
    use tokio::{
        sync::{mpsc, watch},
        task::JoinHandle,
    };
    use tracing::{error, warn};

    use crate::{collectors::ObservabilityEvent, config::Config};

    pub(super) async fn start_real_ebpf(
        config: Config,
        event_tx: mpsc::Sender<ObservabilityEvent>,
        shutdown_rx: watch::Receiver<bool>,
    ) -> Result<Vec<JoinHandle<Result<()>>>> {
        let mut bpf = load_bpf_object()?;

        if config.collectors.cpu {
            attach_tracepoint(&mut bpf, "sched_switch", "sched", "sched_switch")?;
            attach_tracepoint(&mut bpf, "sched_wakeup", "sched", "sched_wakeup")?;
        }

        if config.collectors.process.enabled {
            attach_tracepoint(
                &mut bpf,
                "sched_process_fork",
                "sched",
                "sched_process_fork",
            )?;
            attach_tracepoint(
                &mut bpf,
                "sched_process_exec",
                "sched",
                "sched_process_exec",
            )?;
            attach_tracepoint(
                &mut bpf,
                "sched_process_exit",
                "sched",
                "sched_process_exit",
            )?;
        }

        if config.collectors.memory.track_oom {
            if let Err(err) =
                attach_tracepoint(&mut bpf, "oom_kill_process", "oom", "oom_kill_process")
            {
                warn!(error = %err, "OOM tracepoint not available on this kernel");
            }
        }

        if config.collectors.network.enabled {
            attach_optional_tracepoint(&mut bpf, "net_dev_xmit", "net", "net_dev_xmit");
            attach_optional_tracepoint(&mut bpf, "netif_receive_skb", "net", "netif_receive_skb");

            if config.collectors.network.tcp_rtt {
                attach_optional_tracepoint(&mut bpf, "tcp_probe", "tcp", "tcp_probe");
            }
            if config.collectors.network.tcp_retransmits {
                attach_optional_tracepoint(
                    &mut bpf,
                    "tcp_retransmit_skb",
                    "tcp",
                    "tcp_retransmit_skb",
                );
            }
        }

        if config.collectors.disk.enabled {
            attach_optional_tracepoint(&mut bpf, "block_rq_issue", "block", "block_rq_issue");
            attach_optional_tracepoint(&mut bpf, "block_rq_complete", "block", "block_rq_complete");
        }

        if config.collectors.syscall.enabled {
            attach_optional_tracepoint(&mut bpf, "sys_enter", "raw_syscalls", "sys_enter");
            attach_optional_tracepoint(&mut bpf, "sys_exit", "raw_syscalls", "sys_exit");
        }

        let handle = tokio::task::spawn_blocking(move || {
            let mut bpf = bpf;
            let mut ring = RingBuf::try_from(
                bpf.map_mut("EVENTS")
                    .context("EVENTS map missing in eBPF object")?,
            )
            .context("failed to initialize ring buffer")?;
            let mut shutdown_rx_for_thread = shutdown_rx;

            loop {
                while let Some(item) = ring.next() {
                    if let Some(event) = parse_ring_event(item.as_ref()) {
                        if event_tx.blocking_send(event).is_err() {
                            return Ok(());
                        }
                    }
                }

                if shutdown_rx_for_thread
                    .has_changed()
                    .map(|changed| changed && *shutdown_rx_for_thread.borrow_and_update())
                    .unwrap_or(true)
                {
                    break;
                }

                std::thread::sleep(Duration::from_millis(20));
            }

            drop(bpf);
            Ok(())
        });

        Ok(vec![handle])
    }

    fn load_bpf_object() -> Result<Ebpf> {
        const BPF_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/drishti.bpf.o"));

        if BPF_BYTES.is_empty() {
            anyhow::bail!(
                "embedded eBPF object is empty. Build with DRISHTI_EMBEDDED_BPF_PATH set to a compiled .bpf.o"
            );
        }

        // Keep an aligned view for parsers that assume natural alignment on strict-arch guests.
        let align = std::mem::align_of::<u64>();
        let mut storage = vec![0_u8; BPF_BYTES.len() + align];
        let base = storage.as_ptr() as usize;
        let offset = (align - (base % align)) % align;
        let aligned = &mut storage[offset..offset + BPF_BYTES.len()];
        aligned.copy_from_slice(BPF_BYTES);

        Ebpf::load(aligned).context("failed to load embedded eBPF object")
    }

    fn attach_tracepoint(
        bpf: &mut Ebpf,
        program_name: &str,
        category: &str,
        name: &str,
    ) -> Result<()> {
        let program: &mut TracePoint = bpf
            .program_mut(program_name)
            .with_context(|| format!("missing eBPF program {program_name}"))?
            .try_into()
            .with_context(|| format!("program {program_name} is not a tracepoint"))?;

        program
            .load()
            .with_context(|| format!("failed to load program {program_name}"))?;
        program
            .attach(category, name)
            .with_context(|| format!("failed to attach {program_name} to {category}:{name}"))?;

        Ok(())
    }

    fn attach_optional_tracepoint(bpf: &mut Ebpf, program_name: &str, category: &str, name: &str) {
        if let Err(err) = attach_tracepoint(bpf, program_name, category, name) {
            warn!(
                program = program_name,
                category,
                name,
                error = %err,
                "optional tracepoint attach failed; collector feature will be partially disabled"
            );
        }
    }

    fn parse_ring_event(payload: &[u8]) -> Option<ObservabilityEvent> {
        let kind = *payload.first()?;
        match kind {
            x if x == EventKind::CpuRuntime as u8 => {
                read_event::<CpuRuntimeEvent>(payload).map(ObservabilityEvent::CpuRuntime)
            }
            x if x == EventKind::CpuWait as u8 => {
                read_event::<CpuWaitEvent>(payload).map(ObservabilityEvent::CpuWait)
            }
            x if x == EventKind::ProcLifecycle as u8 => {
                read_event::<ProcLifecycleEvent>(payload).map(ObservabilityEvent::ProcLifecycle)
            }
            x if x == EventKind::OomKill as u8 => {
                read_event::<OomKillEvent>(payload).map(ObservabilityEvent::OomKill)
            }
            x if x == EventKind::NetTraffic as u8 => {
                read_event::<NetTrafficEvent>(payload).map(ObservabilityEvent::NetTraffic)
            }
            x if x == EventKind::TcpRtt as u8 => {
                read_event::<TcpRttEvent>(payload).map(ObservabilityEvent::TcpRtt)
            }
            x if x == EventKind::TcpRetransmit as u8 => {
                read_event::<TcpRetransmitEvent>(payload).map(ObservabilityEvent::TcpRetransmit)
            }
            x if x == EventKind::DiskIo as u8 => {
                read_event::<DiskIoEvent>(payload).map(ObservabilityEvent::DiskIo)
            }
            x if x == EventKind::Syscall as u8 => {
                read_event::<SyscallEvent>(payload).map(ObservabilityEvent::Syscall)
            }
            _ => {
                warn!(kind, "unknown event kind from ring buffer");
                None
            }
        }
    }

    fn read_event<T: Copy>(payload: &[u8]) -> Option<T> {
        if payload.len() < mem::size_of::<T>() {
            error!(
                expected = mem::size_of::<T>(),
                got = payload.len(),
                "ring buffer payload too small"
            );
            return None;
        }

        // Ring buffer events are plain #[repr(C)] structs from kernel memory.
        let value = unsafe { std::ptr::read_unaligned(payload.as_ptr().cast::<T>()) };
        Some(value)
    }
}
