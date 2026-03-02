use crate::{COMM_LEN, IFACE_LEN};

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EventKind {
    CpuRuntime = 1,
    CpuWait = 2,
    ProcLifecycle = 3,
    OomKill = 4,
    NetTraffic = 5,
    TcpRtt = 6,
    TcpRetransmit = 7,
    DiskIo = 8,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ProcLifecycleKind {
    Fork = 1,
    Exec = 2,
    Exit = 3,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum NetDirection {
    Tx = 1,
    Rx = 2,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DiskOp {
    Read = 1,
    Write = 2,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CpuRuntimeEvent {
    pub kind: u8,
    pub _pad0: [u8; 3],
    pub pid: u32,
    pub tgid: u32,
    pub cpu: u32,
    pub run_time_ns: u64,
    pub comm: [u8; COMM_LEN],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct CpuWaitEvent {
    pub kind: u8,
    pub _pad0: [u8; 3],
    pub pid: u32,
    pub tgid: u32,
    pub cpu: u32,
    pub wait_time_ns: u64,
    pub comm: [u8; COMM_LEN],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ProcLifecycleEvent {
    pub kind: u8,
    pub lifecycle: u8,
    pub _pad0: [u8; 2],
    pub pid: u32,
    pub tgid: u32,
    pub ppid: u32,
    pub exit_code: i32,
    pub comm: [u8; COMM_LEN],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct OomKillEvent {
    pub kind: u8,
    pub _pad0: [u8; 3],
    pub pid: u32,
    pub tgid: u32,
    pub pages: u64,
    pub comm: [u8; COMM_LEN],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct NetTrafficEvent {
    pub kind: u8,
    pub direction: u8,
    pub _pad0: [u8; 2],
    pub pid: u32,
    pub tgid: u32,
    pub ifindex: u32,
    pub bytes: u64,
    pub packets: u64,
    pub comm: [u8; COMM_LEN],
    pub iface: [u8; IFACE_LEN],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TcpRttEvent {
    pub kind: u8,
    pub _pad0: [u8; 3],
    pub pid: u32,
    pub tgid: u32,
    pub ifindex: u32,
    pub rtt_usec: u32,
    pub comm: [u8; COMM_LEN],
    pub iface: [u8; IFACE_LEN],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TcpRetransmitEvent {
    pub kind: u8,
    pub _pad0: [u8; 3],
    pub pid: u32,
    pub tgid: u32,
    pub ifindex: u32,
    pub comm: [u8; COMM_LEN],
    pub iface: [u8; IFACE_LEN],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct DiskIoEvent {
    pub kind: u8,
    pub op: u8,
    pub _pad0: [u8; 2],
    pub pid: u32,
    pub tgid: u32,
    pub dev_major: u32,
    pub dev_minor: u32,
    pub bytes: u64,
    pub latency_usec: u64,
    pub queue_depth: u32,
    pub comm: [u8; COMM_LEN],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct EventHeader {
    pub kind: u8,
    pub _pad0: [u8; 3],
}
