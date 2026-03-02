#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ProcKey {
    pub pid: u32,
    pub tgid: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct CpuStats {
    pub run_time_ns: u64,
    pub wait_time_ns: u64,
    pub voluntary_ctx: u64,
    pub involuntary_ctx: u64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct WakeKey {
    pub pid: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct WakeTimestamp {
    pub wake_ts_ns: u64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct LastSeenRun {
    pub ts_ns: u64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct SyscallKey {
    pub pid: u32,
    pub syscall_nr: i64,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct SyscallStartTs {
    pub ts_ns: u64,
}
