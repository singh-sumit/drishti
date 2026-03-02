#![cfg_attr(target_arch = "bpf", no_std)]
#![cfg_attr(target_arch = "bpf", no_main)]

#[cfg(not(target_arch = "bpf"))]
pub mod host_stub {
    pub const PROGRAMS: &[&str] = &[
        "sched_switch",
        "sched_wakeup",
        "sched_process_fork",
        "sched_process_exec",
        "sched_process_exit",
        "oom_kill_process",
        "net_dev_xmit",
        "netif_receive_skb",
        "tcp_probe",
        "tcp_retransmit_skb",
        "block_rq_issue",
        "block_rq_complete",
        "sys_enter",
        "sys_exit",
    ];
}

#[cfg(target_arch = "bpf")]
mod programs;

#[cfg(target_arch = "bpf")]
pub use programs::*;
