//! Candidate skb metadata becomes evidence only at successful outer recvmsg exit.

use aya_ebpf::{
    EbpfContext, Global,
    helpers::{bpf_get_current_pid_tgid, bpf_probe_read_kernel},
    macros::{fentry, fexit, kprobe, kretprobe, map},
    maps::HashMap,
    programs::{FEntryContext, FExitContext, ProbeContext, RetProbeContext},
};
use honk_ebpf_common::receive_trace::{
    RECEIVE_TRACE_BATCH_SIZE, RECEIVE_TRACE_SOCKETS, RECEIVE_TRACE_VALID, ReceiveTraceBatch,
};

#[unsafe(no_mangle)]
pub static RECEIVE_SOCK_COOKIE_OFFSET: Global<u32> = Global::new(0);
#[unsafe(no_mangle)]
pub static RECEIVE_SKB_PRIORITY_OFFSET: Global<u32> = Global::new(0);
#[unsafe(no_mangle)]
pub static RECEIVE_SKB_MARK_OFFSET: Global<u32> = Global::new(0);
#[unsafe(no_mangle)]
pub static RECEIVE_OWNER_TGID: Global<u32> = Global::new(0);

#[map]
pub static RECEIVE_TRACE: HashMap<u64, ReceiveTraceBatch> =
    HashMap::with_max_entries(RECEIVE_TRACE_SOCKETS, 0);

/// Maps `pid_tgid` -> active `sock_cookie` during a `udp_recvmsg`/`udpv6_recvmsg`
/// call when using the ARM64 kprobe/kretprobe fallback (where kretprobe lacks entry args).
#[map]
pub static RECEIVE_ACTIVE_TASK: HashMap<u64, u64> = HashMap::with_max_entries(1024, 0);

#[repr(C)]
struct Arm64PtRegs {
    regs: [u64; 31],
}

#[inline(always)]
fn state_cookie(sk: *const u8) -> Option<(u64, *mut ReceiveTraceBatch)> {
    // The hooks run for every UDP receive on the host.
    if sk.is_null() || (bpf_get_current_pid_tgid() >> 32) as u32 != RECEIVE_OWNER_TGID.load() {
        return None;
    }
    let cookie = unsafe {
        bpf_probe_read_kernel::<u64>(sk.add(RECEIVE_SOCK_COOKIE_OFFSET.load() as usize).cast())
    }
    .ok()?;
    if cookie == 0 {
        return None;
    }
    let state = RECEIVE_TRACE.get_ptr_mut(&cookie)?;
    (unsafe { (*state).active } != 0).then_some((cookie, state))
}

#[inline(always)]
fn state(sk: *const u8) -> Option<*mut ReceiveTraceBatch> {
    state_cookie(sk).map(|(_, state)| state)
}

#[inline(always)]
fn enter_batch(batch: &mut ReceiveTraceBatch, owner: u64, flags: i32) {
    if batch.depth != 0 {
        batch.lost = 1;
        batch.depth = batch.depth.saturating_add(1);
        return;
    }
    batch.owner = owner;
    batch.depth = 1;
    batch.candidate.valid = 0;
    // PEEK and ERRQUEUE are never data-consumption witnesses.
    if flags & (0x2 | 0x2000) != 0 {
        batch.lost = 1;
    }
}

#[inline(always)]
fn enter(sk: *const u8, flags: i32) {
    let Some(ptr) = state(sk) else { return };
    enter_batch(unsafe { &mut *ptr }, bpf_get_current_pid_tgid(), flags);
}

#[inline(always)]
fn exit_batch(batch: &mut ReceiveTraceBatch, owner: u64, result: Result<i32, i32>) -> bool {
    if batch.depth == 0 || batch.owner != owner {
        batch.lost = 1;
        return batch.depth == 0;
    }
    batch.depth -= 1;
    if batch.depth != 0 {
        return false;
    }
    let Ok(length) = result else {
        batch.lost = 1;
        return true;
    };
    if length < 0 {
        batch.candidate.valid = 0;
        return true;
    }
    let index = batch.count as usize;
    if index >= RECEIVE_TRACE_BATCH_SIZE {
        batch.lost = 1;
        return true;
    }
    // Zero bytes is a successfully consumed datagram, not an empty batch.
    batch.candidate.length = length as u32;
    batch.packets[index] = batch.candidate;
    batch.count += 1;
    batch.candidate.valid = 0;
    true
}

#[inline(always)]
fn exit(sk: *const u8, result: Result<i32, i32>) {
    let Some(ptr) = state(sk) else { return };
    let _ = exit_batch(unsafe { &mut *ptr }, bpf_get_current_pid_tgid(), result);
}

#[inline(always)]
fn record_candidate(batch: &mut ReceiveTraceBatch, owner: u64, skb: *const u8) {
    if batch.depth != 1 || batch.owner != owner {
        batch.lost = 1;
        return;
    }
    // A checksum-discarded candidate is overwritten by the next helper return.
    batch.candidate.valid = 0;
    if skb.is_null() || skb as usize >= usize::MAX - 4095 {
        return;
    }
    let priority = unsafe {
        bpf_probe_read_kernel::<u32>(skb.add(RECEIVE_SKB_PRIORITY_OFFSET.load() as usize).cast())
    };
    let mark = unsafe {
        bpf_probe_read_kernel::<u32>(skb.add(RECEIVE_SKB_MARK_OFFSET.load() as usize).cast())
    };
    if let (Ok(priority), Ok(mark)) = (priority, mark) {
        batch.candidate.priority = priority;
        batch.candidate.mark = mark;
        batch.candidate.valid = RECEIVE_TRACE_VALID;
    }
}

#[fentry]
pub fn honk_udp_receive_enter(ctx: FEntryContext) -> u32 {
    enter(ctx.arg(0), ctx.arg(3));
    0
}

#[fentry]
pub fn honk_udp6_receive_enter(ctx: FEntryContext) -> u32 {
    enter(ctx.arg(0), ctx.arg(3));
    0
}

#[fexit]
pub fn honk_udp_receive_exit(ctx: FExitContext) -> u32 {
    exit(ctx.arg(0), ctx.ret());
    0
}

#[fexit]
pub fn honk_udp6_receive_exit(ctx: FExitContext) -> u32 {
    exit(ctx.arg(0), ctx.ret());
    0
}

#[fexit]
pub fn honk_udp_receive_candidate(ctx: FExitContext) -> u32 {
    let Some(ptr) = state(ctx.arg(0)) else {
        return 0;
    };
    let Ok(skb) = ctx.ret::<*const u8>() else {
        return 0;
    };
    record_candidate(unsafe { &mut *ptr }, bpf_get_current_pid_tgid(), skb);
    0
}

#[inline(always)]
fn enter_kprobe_arm64(ctx: &ProbeContext) {
    let regs = ctx.as_ptr() as *const Arm64PtRegs;
    let sk = unsafe { (*regs).regs[0] } as *const u8;
    let flags = unsafe { (*regs).regs[3] } as i32;
    let Some((cookie, ptr)) = state_cookie(sk) else {
        return;
    };
    let owner = bpf_get_current_pid_tgid();
    let _ = RECEIVE_ACTIVE_TASK.insert(&owner, &cookie, 0);
    enter_batch(unsafe { &mut *ptr }, owner, flags);
}

#[inline(always)]
fn exit_kretprobe_arm64(ctx: &RetProbeContext) {
    let owner = bpf_get_current_pid_tgid();
    if (owner >> 32) as u32 != RECEIVE_OWNER_TGID.load() {
        return;
    }
    let Some(cookie) = (unsafe { RECEIVE_ACTIVE_TASK.get(&owner) }).copied() else {
        return;
    };
    let Some(ptr) = RECEIVE_TRACE.get_ptr_mut(&cookie) else {
        let _ = RECEIVE_ACTIVE_TASK.remove(&owner);
        return;
    };
    if unsafe { (*ptr).active } == 0 {
        let _ = RECEIVE_ACTIVE_TASK.remove(&owner);
        return;
    }
    let regs = ctx.as_ptr() as *const Arm64PtRegs;
    let ret_val = unsafe { (*regs).regs[0] } as i64 as i32;
    if exit_batch(unsafe { &mut *ptr }, owner, Ok(ret_val)) {
        let _ = RECEIVE_ACTIVE_TASK.remove(&owner);
    }
}

#[kprobe]
pub fn honk_udp_receive_enter_kp(ctx: ProbeContext) -> u32 {
    enter_kprobe_arm64(&ctx);
    0
}

#[kprobe]
pub fn honk_udp6_receive_enter_kp(ctx: ProbeContext) -> u32 {
    enter_kprobe_arm64(&ctx);
    0
}

#[kretprobe]
pub fn honk_udp_receive_exit_krp(ctx: RetProbeContext) -> u32 {
    exit_kretprobe_arm64(&ctx);
    0
}

#[kretprobe]
pub fn honk_udp6_receive_exit_krp(ctx: RetProbeContext) -> u32 {
    exit_kretprobe_arm64(&ctx);
    0
}

#[kretprobe]
pub fn honk_udp_receive_candidate_krp(ctx: RetProbeContext) -> u32 {
    let owner = bpf_get_current_pid_tgid();
    if (owner >> 32) as u32 != RECEIVE_OWNER_TGID.load() {
        return 0;
    }
    let Some(cookie) = (unsafe { RECEIVE_ACTIVE_TASK.get(&owner) }).copied() else {
        return 0;
    };
    let Some(ptr) = RECEIVE_TRACE.get_ptr_mut(&cookie) else {
        return 0;
    };
    if unsafe { (*ptr).active } == 0 {
        return 0;
    }
    let regs = ctx.as_ptr() as *const Arm64PtRegs;
    let skb = unsafe { (*regs).regs[0] } as *const u8;
    record_candidate(unsafe { &mut *ptr }, owner, skb);
    0
}
