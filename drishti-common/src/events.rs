use crate::COMM_LEN;

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EventKind {
    CpuRuntime = 1,
    CpuWait = 2,
    ProcLifecycle = 3,
    OomKill = 4,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ProcLifecycleKind {
    Fork = 1,
    Exec = 2,
    Exit = 3,
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
pub struct EventHeader {
    pub kind: u8,
    pub _pad0: [u8; 3],
}
