#![allow(unsafe_code)]

use aya_bpf::{
    helpers::{bpf_get_current_comm, bpf_get_current_pid_tgid, bpf_ktime_get_ns},
    macros::{map, tracepoint},
    maps::{HashMap, RingBuf},
    programs::TracePointContext,
};
use core::mem;
use drishti_common::{
    COMM_LEN,
    events::{
        CpuRuntimeEvent, CpuWaitEvent, EventKind, OomKillEvent, ProcLifecycleEvent,
        ProcLifecycleKind,
    },
};

#[map(name = "EVENTS")]
static EVENTS: RingBuf = RingBuf::with_byte_size(1 << 20, 0);

#[map(name = "LAST_RUN_TS")]
static LAST_RUN_TS: HashMap<u32, u64> = HashMap::with_max_entries(16384, 0);

#[map(name = "WAKE_TS")]
static WAKE_TS: HashMap<u32, u64> = HashMap::with_max_entries(16384, 0);

#[tracepoint(name = "sched_switch")]
pub fn sched_switch(ctx: TracePointContext) -> u32 {
    match try_sched_switch(ctx) {
        Ok(v) => v,
        Err(_) => 1,
    }
}

fn try_sched_switch(_ctx: TracePointContext) -> Result<u32, i64> {
    let now = unsafe { bpf_ktime_get_ns() };
    let pid = current_pid();
    let tgid = current_tgid();

    let run_delta = LAST_RUN_TS
        .get(&pid)
        .map_or(0, |last| now.saturating_sub(*last));
    let _ = LAST_RUN_TS.insert(&pid, &now, 0);

    emit_runtime(pid, tgid, run_delta)?;

    if let Some(wake_ts) = WAKE_TS.get(&pid) {
        let wait_delta = now.saturating_sub(*wake_ts);
        emit_wait(pid, tgid, wait_delta)?;
        let _ = WAKE_TS.remove(&pid);
    }

    Ok(0)
}

#[tracepoint(name = "sched_wakeup")]
pub fn sched_wakeup(_ctx: TracePointContext) -> u32 {
    let now = unsafe { bpf_ktime_get_ns() };
    let pid = current_pid();
    let _ = WAKE_TS.insert(&pid, &now, 0);
    0
}

#[tracepoint(name = "sched_process_fork")]
pub fn sched_process_fork(_ctx: TracePointContext) -> u32 {
    emit_proc(ProcLifecycleKind::Fork, 0)
}

#[tracepoint(name = "sched_process_exec")]
pub fn sched_process_exec(_ctx: TracePointContext) -> u32 {
    emit_proc(ProcLifecycleKind::Exec, 0)
}

#[tracepoint(name = "sched_process_exit")]
pub fn sched_process_exit(_ctx: TracePointContext) -> u32 {
    emit_proc(ProcLifecycleKind::Exit, 0)
}

#[tracepoint(name = "oom_kill_process")]
pub fn oom_kill_process(_ctx: TracePointContext) -> u32 {
    let event = OomKillEvent {
        kind: EventKind::OomKill as u8,
        _pad0: [0; 3],
        pid: current_pid(),
        tgid: current_tgid(),
        pages: 0,
        comm: current_comm(),
    };
    submit_event(&event)
}

fn emit_proc(kind: ProcLifecycleKind, exit_code: i32) -> u32 {
    let event = ProcLifecycleEvent {
        kind: EventKind::ProcLifecycle as u8,
        lifecycle: kind as u8,
        _pad0: [0; 2],
        pid: current_pid(),
        tgid: current_tgid(),
        ppid: 0,
        exit_code,
        comm: current_comm(),
    };
    submit_event(&event)
}

fn emit_runtime(pid: u32, tgid: u32, run_time_ns: u64) -> Result<(), i64> {
    let event = CpuRuntimeEvent {
        kind: EventKind::CpuRuntime as u8,
        _pad0: [0; 3],
        pid,
        tgid,
        cpu: 0,
        run_time_ns,
        comm: current_comm(),
    };
    if submit_event(&event) == 0 {
        Ok(())
    } else {
        Err(-1)
    }
}

fn emit_wait(pid: u32, tgid: u32, wait_time_ns: u64) -> Result<(), i64> {
    let event = CpuWaitEvent {
        kind: EventKind::CpuWait as u8,
        _pad0: [0; 3],
        pid,
        tgid,
        cpu: 0,
        wait_time_ns,
        comm: current_comm(),
    };
    if submit_event(&event) == 0 {
        Ok(())
    } else {
        Err(-1)
    }
}

fn submit_event<T>(value: &T) -> u32 {
    let size = mem::size_of::<T>() as u32;
    match EVENTS.reserve(size, 0) {
        Some(mut entry) => {
            entry.copy_from_slice(unsafe {
                core::slice::from_raw_parts((value as *const T).cast::<u8>(), size as usize)
            });
            entry.submit(0);
            0
        }
        None => 1,
    }
}

#[inline(always)]
fn current_pid() -> u32 {
    (unsafe { bpf_get_current_pid_tgid() } & 0xffff_ffff) as u32
}

#[inline(always)]
fn current_tgid() -> u32 {
    (unsafe { bpf_get_current_pid_tgid() } >> 32) as u32
}

#[inline(always)]
fn current_comm() -> [u8; COMM_LEN] {
    let mut comm = [0u8; COMM_LEN];
    let _ = unsafe { bpf_get_current_comm(comm.as_mut_ptr().cast(), COMM_LEN as u32) };
    comm
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
static LICENSE: &[u8] = b"GPL\0";
