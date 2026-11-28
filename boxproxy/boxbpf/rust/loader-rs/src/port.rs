//! Pure Rust port of `loader.c` (`boxproxy/boxbpf/loader.c`).

use std::ffi::{CStr, CString, c_int};
use std::io;

pub const PIN_DIR: &str = "/sys/fs/bpf/box";

pub const PIN_CIDR_OUT4: &str = "/sys/fs/bpf/box/box_cidr_out4";
pub const PIN_CIDR_OUT6: &str = "/sys/fs/bpf/box/box_cidr_out6";
pub const PIN_CIDR_PRE4: &str = "/sys/fs/bpf/box/box_cidr_pre4";
pub const PIN_CIDR_PRE6: &str = "/sys/fs/bpf/box/box_cidr_pre6";
pub const PIN_FORCE_OUT4: &str = "/sys/fs/bpf/box/box_force_out4";
pub const PIN_FORCE_OUT6: &str = "/sys/fs/bpf/box/box_force_out6";
pub const PIN_APP_OUT4: &str = "/sys/fs/bpf/box/box_uid_out4";
pub const PIN_APP_OUT6: &str = "/sys/fs/bpf/box/box_uid_out6";

pub const MAP_RUNTIME: &str = "/sys/fs/bpf/box/box_runtime_cfg";
pub const MAP_CIDR4: &str = "/sys/fs/bpf/box/box_cidr4_lpm";
pub const MAP_CIDR6: &str = "/sys/fs/bpf/box/box_cidr6_lpm";
pub const MAP_FORCE_UID: &str = "/sys/fs/bpf/box/box_force_uid_set";
pub const MAP_APP_UID: &str = "/sys/fs/bpf/box/box_app_uid_set";

pub const PROBE_PIN4: &str = "/sys/fs/bpf/box/box_probe4";
pub const PROBE_PIN6: &str = "/sys/fs/bpf/box/box_probe6";
pub const PROBE_MAP_RUNTIME: &str = "/sys/fs/bpf/box/box_probe_runtime";
pub const PROBE_MAP_CIDR4: &str = "/sys/fs/bpf/box/box_probe_cidr4";
pub const PROBE_MAP_CIDR6: &str = "/sys/fs/bpf/box/box_probe_cidr6";
pub const PROBE_MAP_FORCE_UID: &str = "/sys/fs/bpf/box/box_probe_force_uid";
pub const PROBE_MAP_APP_UID: &str = "/sys/fs/bpf/box/box_probe_app_uid";

pub const BPF_MAP_TYPE_HASH: c_int = 1;
pub const BPF_MAP_TYPE_LPM_TRIE: c_int = 11;
pub const COMPAT_MAP_NO_PREALLOC: u32 = 1;

const BPF_MAP_CREATE: c_int = 0;
const BPF_MAP_DELETE_ELEM: c_int = 3;
const BPF_MAP_GET_NEXT_KEY: c_int = 4;
const BPF_PROG_LOAD: c_int = 5;
const BPF_OBJ_PIN: c_int = 6;
const BPF_OBJ_GET: c_int = 7;

const BPF_PROG_TYPE_SOCKET_FILTER: u32 = 1;
const BPF_PSEUDO_MAP_FD: u8 = 1;
const BPF_OBJ_NAME_LEN: usize = 16;
const VERIFY_LOG_SIZE: usize = 65536;
const MAX_PATH_LEN: usize = 512;
const MAX_PROGRAM_INSNS: usize = 160;

const BPF_FUNC_MAP_LOOKUP_ELEM: i32 = 1;
const COMPAT_SKB_LOAD_BYTES: i32 = 26;
const COMPAT_GET_SOCKET_UID: i32 = 47;

const BPF_REG_0: u8 = 0;
const BPF_REG_1: u8 = 1;
const BPF_REG_2: u8 = 2;
const BPF_REG_3: u8 = 3;
const BPF_REG_4: u8 = 4;
const BPF_REG_6: u8 = 6;
const BPF_REG_10: u8 = 10;

const BPF_LD: u8 = 0x00;
const BPF_LDX: u8 = 0x01;
const BPF_ST: u8 = 0x02;
const BPF_STX: u8 = 0x03;
const BPF_JMP: u8 = 0x05;
const BPF_ALU64: u8 = 0x07;

const BPF_W: u8 = 0x00;
const BPF_B: u8 = 0x10;
const BPF_DW: u8 = 0x18;

const BPF_IMM: u8 = 0x00;
const BPF_ABS: u8 = 0x20;
const BPF_MEM: u8 = 0x60;

const BPF_ADD: u8 = 0x00;
const BPF_MOV: u8 = 0xb0;
const BPF_K: u8 = 0x00;
const BPF_X: u8 = 0x08;

const BPF_JEQ: u8 = 0x10;
const BPF_JGT: u8 = 0x20;
const BPF_JNE: u8 = 0x50;
const BPF_CALL: u8 = 0x80;
const BPF_EXIT: u8 = 0x90;

const SK_BUFF_DATA_OFF: i16 = 76;
const SK_BUFF_DATA_END_OFF: i16 = 80;

const XT_BPF_MISS: i32 = 0;
const XT_BPF_MATCH: i32 = 0xffff;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BpfInsn {
    pub code: u8,
    pub regs: u8,
    pub off: i16,
    pub imm: i32,
}

impl BpfInsn {
    const fn new(code: u8, dst_reg: u8, src_reg: u8, off: i16, imm: i32) -> Self {
        Self {
            code,
            regs: ((src_reg & 0x0f) << 4) | (dst_reg & 0x0f),
            off,
            imm,
        }
    }
}

#[repr(C)]
struct BpfAttrRaw {
    bytes: [u8; 168],
}

impl BpfAttrRaw {
    const fn zeroed() -> Self {
        Self { bytes: [0u8; 168] }
    }

    fn write_u32(&mut self, offset: usize, val: u32) {
        self.bytes[offset..offset + 4].copy_from_slice(&val.to_ne_bytes());
    }

    fn write_u64(&mut self, offset: usize, val: u64) {
        self.bytes[offset..offset + 8].copy_from_slice(&val.to_ne_bytes());
    }
}

fn bpf_syscall(cmd: c_int, attr: &BpfAttrRaw) -> c_int {
    // SAFETY: `attr` is a valid 168-byte zero-initialized `union bpf_attr` matching the Linux UAPI.
    let rc = unsafe {
        libc::syscall(
            libc::SYS_bpf,
            cmd,
            std::ptr::from_ref(attr),
            std::mem::size_of::<BpfAttrRaw>(),
        )
    };
    rc as c_int
}

fn last_errno_and_msg() -> (i32, String) {
    let err = io::Error::last_os_error();
    let code = err.raw_os_error().unwrap_or(0);
    // SAFETY: `libc::strerror` returns a valid NUL-terminated C string for any `errno` code.
    let msg = unsafe {
        let ptr = libc::strerror(code);
        if ptr.is_null() {
            String::new()
        } else {
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        }
    };
    (code, msg)
}

pub fn raise_memlock() {
    let unlimited = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    // SAFETY: `unlimited` is a valid `libc::rlimit` struct passed to `setrlimit(RLIMIT_MEMLOCK)`.
    unsafe {
        libc::setrlimit(libc::RLIMIT_MEMLOCK, &unlimited);
    }
}

/// Port of `int create_map(...)` (`loader.c:19`).
pub fn create_map(
    r#type: c_int,
    key_size: usize,
    value_size: usize,
    max_entries: u32,
    flags: u32,
) -> c_int {
    let mut attr = BpfAttrRaw::zeroed();
    attr.write_u32(0, r#type as u32);
    attr.write_u32(4, key_size as u32);
    attr.write_u32(8, value_size as u32);
    attr.write_u32(12, max_entries);
    attr.write_u32(16, flags);
    bpf_syscall(BPF_MAP_CREATE, &attr)
}

fn delete_elem_raw(fd: c_int, key: *const libc::c_void) -> c_int {
    let mut attr = BpfAttrRaw::zeroed();
    attr.write_u32(0, fd as u32);
    attr.write_u64(8, key as usize as u64);
    bpf_syscall(BPF_MAP_DELETE_ELEM, &attr)
}

fn next_key_raw(fd: c_int, key: *const libc::c_void, next: *mut libc::c_void) -> c_int {
    let mut attr = BpfAttrRaw::zeroed();
    attr.write_u32(0, fd as u32);
    attr.write_u64(8, key as usize as u64);
    attr.write_u64(16, next as usize as u64);
    bpf_syscall(BPF_MAP_GET_NEXT_KEY, &attr)
}

/// Port of `int clear_map(int fd, size_t key_size)` (`loader.c:57`).
pub fn clear_map(fd: c_int, key_size: usize) -> c_int {
    // SAFETY: Mirrors C `calloc(key_size, 1)` so unallocatable sizes return NULL (-> -1) without aborting.
    let key = unsafe { libc::calloc(key_size, 1) };
    if key.is_null() {
        return -1;
    }

    let mut count: c_int = 0;
    while next_key_raw(fd, std::ptr::null(), key) == 0 {
        if delete_elem_raw(fd, key) != 0 {
            let (code, msg) = last_errno_and_msg();
            eprintln!("delete BPF map item failed: errno={code} ({msg})");
            // SAFETY: `key` was allocated by `libc::calloc` above and has not been freed.
            unsafe { libc::free(key) };
            return -1;
        }
        count += 1;
    }

    // SAFETY: `key` was allocated by `libc::calloc` above and has not been freed.
    unsafe { libc::free(key) };
    count
}

fn path_to_cstring(path: Option<&[u8]>) -> Option<CString> {
    let bytes = path?;
    let trimmed = bytes.split(|&b| b == 0).next().unwrap_or(bytes);
    CString::new(trimmed).ok()
}

/// Port of `int get_pinned(const char *path)` (`loader.c:75`).
pub fn get_pinned(path: Option<&[u8]>) -> c_int {
    let Some(cpath) = path_to_cstring(path) else {
        return -1;
    };
    let mut attr = BpfAttrRaw::zeroed();
    attr.write_u64(0, cpath.as_ptr() as usize as u64);
    bpf_syscall(BPF_OBJ_GET, &attr)
}

fn ensure_parent_dir(path: &[u8]) {
    let limit = path.len().min(MAX_PATH_LEN - 1);
    let truncated = &path[..limit];
    let Some(slash) = truncated.iter().rposition(|&b| b == b'/') else {
        return;
    };
    if let Ok(cdir) = CString::new(&truncated[..slash]) {
        // SAFETY: `cdir` is a valid NUL-terminated path passed to POSIX `mkdir`.
        unsafe {
            libc::mkdir(cdir.as_ptr(), 0o700);
        }
    }
}

/// Port of `int pin_replace(int fd, const char *path)` (`loader.c:91`).
pub fn pin_replace(fd: c_int, path: Option<&[u8]>) -> c_int {
    let Some(bytes) = path else {
        return -1;
    };
    ensure_parent_dir(bytes);
    let Some(cpath) = path_to_cstring(Some(bytes)) else {
        return -1;
    };
    // SAFETY: `cpath` is a valid NUL-terminated path passed to POSIX `unlink`.
    unsafe {
        libc::unlink(cpath.as_ptr());
    }

    let mut attr = BpfAttrRaw::zeroed();
    attr.write_u64(0, cpath.as_ptr() as usize as u64);
    attr.write_u32(8, fd as u32);
    let rc = bpf_syscall(BPF_OBJ_PIN, &attr);
    if rc != 0 {
        let (code, msg) = last_errno_and_msg();
        eprintln!(
            "pin BPF object failed: {} errno={code} ({msg})",
            String::from_utf8_lossy(bytes)
        );
    }
    rc
}

/// Port of `void close_fd(int fd)` (`loader.c:106`).
pub fn close_fd(fd: c_int) {
    if fd >= 0 {
        // SAFETY: Matches C `if (fd >= 0) close(fd)` semantics directly.
        unsafe {
            libc::close(fd);
        }
    }
}

/// Port of `void remove_known_pins(void)` (`loader.c:110`).
pub fn remove_known_pins() {
    const PINS: [&str; 20] = [
        PIN_CIDR_OUT4,
        PIN_CIDR_OUT6,
        PIN_CIDR_PRE4,
        PIN_CIDR_PRE6,
        PIN_FORCE_OUT4,
        PIN_FORCE_OUT6,
        PIN_APP_OUT4,
        PIN_APP_OUT6,
        MAP_RUNTIME,
        MAP_CIDR4,
        MAP_CIDR6,
        MAP_FORCE_UID,
        MAP_APP_UID,
        PROBE_PIN4,
        PROBE_PIN6,
        PROBE_MAP_RUNTIME,
        PROBE_MAP_CIDR4,
        PROBE_MAP_CIDR6,
        PROBE_MAP_FORCE_UID,
        PROBE_MAP_APP_UID,
    ];
    for p in PINS {
        let _ = std::fs::remove_file(p);
    }
    let _ = std::fs::remove_dir(PIN_DIR);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ReadMode {
    Direct,
    Helper,
    LdAbs,
}

#[derive(Clone, Copy)]
struct PacketLayout {
    key_stack_off: i16,
    addr_stack_off: i16,
    scratch_stack_off: i16,
    packet_addr_off: i32,
    addr_len: i32,
    full_prefix: i32,
}

const LAYOUT_V4: PacketLayout = PacketLayout {
    key_stack_off: -8,
    addr_stack_off: -4,
    scratch_stack_off: -12,
    packet_addr_off: 16,
    addr_len: 4,
    full_prefix: 32,
};

const LAYOUT_V6: PacketLayout = PacketLayout {
    key_stack_off: -24,
    addr_stack_off: -20,
    scratch_stack_off: -28,
    packet_addr_off: 24,
    addr_len: 16,
    full_prefix: 128,
};

struct ProgBuilder {
    insns: Vec<BpfInsn>,
    overflow: bool,
}

impl ProgBuilder {
    fn new() -> Self {
        Self {
            insns: Vec::with_capacity(64),
            overflow: false,
        }
    }

    fn emit(&mut self, insn: BpfInsn) {
        if self.insns.len() >= MAX_PROGRAM_INSNS {
            self.overflow = true;
            return;
        }
        self.insns.push(insn);
    }

    fn emit_jump(&mut self, insn: BpfInsn) -> usize {
        let idx = self.insns.len();
        self.emit(insn);
        idx
    }

    fn patch_jump_here(&mut self, jump_idx: usize) {
        if jump_idx < self.insns.len() {
            self.insns[jump_idx].off = (self.insns.len() - jump_idx - 1) as i16;
        }
    }

    fn emit_map_fd(&mut self, reg: u8, map_fd: c_int) {
        self.emit(BpfInsn::new(
            BPF_LD | BPF_DW | BPF_IMM,
            reg,
            BPF_PSEUDO_MAP_FD,
            0,
            map_fd,
        ));
        self.emit(BpfInsn::default());
    }

    fn emit_result(&mut self, value: i32) {
        self.emit(BpfInsn::new(
            BPF_ALU64 | BPF_MOV | BPF_K,
            BPF_REG_0,
            0,
            0,
            value,
        ));
        self.emit(BpfInsn::new(BPF_JMP | BPF_EXIT, 0, 0, 0, 0));
    }

    fn emit_load_destination(&mut self, l: &PacketLayout, mode: ReadMode) -> Option<usize> {
        self.emit(BpfInsn::new(
            BPF_ST | BPF_W | BPF_MEM,
            BPF_REG_10,
            0,
            l.key_stack_off,
            l.full_prefix,
        ));

        match mode {
            ReadMode::LdAbs => {
                for i in 0..l.addr_len {
                    self.emit(BpfInsn::new(
                        BPF_LD | BPF_B | BPF_ABS,
                        0,
                        0,
                        0,
                        l.packet_addr_off + i,
                    ));
                    self.emit(BpfInsn::new(
                        BPF_STX | BPF_B | BPF_MEM,
                        BPF_REG_10,
                        BPF_REG_0,
                        l.addr_stack_off + i as i16,
                        0,
                    ));
                }
                None
            }
            ReadMode::Helper => {
                self.emit(BpfInsn::new(
                    BPF_ALU64 | BPF_MOV | BPF_X,
                    BPF_REG_1,
                    BPF_REG_6,
                    0,
                    0,
                ));
                self.emit(BpfInsn::new(
                    BPF_ALU64 | BPF_MOV | BPF_K,
                    BPF_REG_2,
                    0,
                    0,
                    l.packet_addr_off,
                ));
                self.emit(BpfInsn::new(
                    BPF_ALU64 | BPF_MOV | BPF_X,
                    BPF_REG_3,
                    BPF_REG_10,
                    0,
                    0,
                ));
                self.emit(BpfInsn::new(
                    BPF_ALU64 | BPF_ADD | BPF_K,
                    BPF_REG_3,
                    0,
                    0,
                    i32::from(l.addr_stack_off),
                ));
                self.emit(BpfInsn::new(
                    BPF_ALU64 | BPF_MOV | BPF_K,
                    BPF_REG_4,
                    0,
                    0,
                    l.addr_len,
                ));
                self.emit(BpfInsn::new(
                    BPF_JMP | BPF_CALL,
                    0,
                    0,
                    0,
                    COMPAT_SKB_LOAD_BYTES,
                ));
                Some(self.emit_jump(BpfInsn::new(BPF_JMP | BPF_JNE | BPF_K, BPF_REG_0, 0, 0, 0)))
            }
            ReadMode::Direct => {
                self.emit(BpfInsn::new(
                    BPF_LDX | BPF_W | BPF_MEM,
                    BPF_REG_1,
                    BPF_REG_6,
                    SK_BUFF_DATA_OFF,
                    0,
                ));
                self.emit(BpfInsn::new(
                    BPF_LDX | BPF_W | BPF_MEM,
                    BPF_REG_2,
                    BPF_REG_6,
                    SK_BUFF_DATA_END_OFF,
                    0,
                ));
                self.emit(BpfInsn::new(
                    BPF_ALU64 | BPF_MOV | BPF_X,
                    BPF_REG_3,
                    BPF_REG_1,
                    0,
                    0,
                ));
                self.emit(BpfInsn::new(
                    BPF_ALU64 | BPF_ADD | BPF_K,
                    BPF_REG_3,
                    0,
                    0,
                    l.packet_addr_off + l.addr_len,
                ));
                let out_of_bounds = self.emit_jump(BpfInsn::new(
                    BPF_JMP | BPF_JGT | BPF_X,
                    BPF_REG_3,
                    BPF_REG_2,
                    0,
                    0,
                ));
                self.emit(BpfInsn::new(
                    BPF_ALU64 | BPF_ADD | BPF_K,
                    BPF_REG_1,
                    0,
                    0,
                    l.packet_addr_off,
                ));
                for i in 0..l.addr_len {
                    self.emit(BpfInsn::new(
                        BPF_LDX | BPF_B | BPF_MEM,
                        BPF_REG_3,
                        BPF_REG_1,
                        i as i16,
                        0,
                    ));
                    self.emit(BpfInsn::new(
                        BPF_STX | BPF_B | BPF_MEM,
                        BPF_REG_10,
                        BPF_REG_3,
                        l.addr_stack_off + i as i16,
                        0,
                    ));
                }
                Some(out_of_bounds)
            }
        }
    }

    fn emit_cidr_lookup(&mut self, cidr_map_fd: c_int, l: &PacketLayout) {
        self.emit_map_fd(BPF_REG_1, cidr_map_fd);
        self.emit(BpfInsn::new(
            BPF_ALU64 | BPF_MOV | BPF_X,
            BPF_REG_2,
            BPF_REG_10,
            0,
            0,
        ));
        self.emit(BpfInsn::new(
            BPF_ALU64 | BPF_ADD | BPF_K,
            BPF_REG_2,
            0,
            0,
            i32::from(l.key_stack_off),
        ));
        self.emit(BpfInsn::new(
            BPF_JMP | BPF_CALL,
            0,
            0,
            0,
            BPF_FUNC_MAP_LOOKUP_ELEM,
        ));
    }
}

fn build_cidr_insns(cidr_map_fd: c_int, l: &PacketLayout, mode: ReadMode) -> ProgBuilder {
    let mut b = ProgBuilder::new();
    b.emit(BpfInsn::new(
        BPF_ALU64 | BPF_MOV | BPF_X,
        BPF_REG_6,
        BPF_REG_1,
        0,
        0,
    ));
    let load_fail = b.emit_load_destination(l, mode);
    b.emit_cidr_lookup(cidr_map_fd, l);
    let cidr_miss = b.emit_jump(BpfInsn::new(BPF_JMP | BPF_JEQ | BPF_K, BPF_REG_0, 0, 0, 0));
    b.emit_result(XT_BPF_MATCH);
    b.patch_jump_here(cidr_miss);
    if let Some(idx) = load_fail {
        b.patch_jump_here(idx);
    }
    b.emit_result(XT_BPF_MISS);
    b
}

fn build_force_insns(
    cidr_map_fd: c_int,
    uid_map_fd: c_int,
    l: &PacketLayout,
    mode: ReadMode,
) -> ProgBuilder {
    let mut b = ProgBuilder::new();
    b.emit(BpfInsn::new(
        BPF_ALU64 | BPF_MOV | BPF_X,
        BPF_REG_6,
        BPF_REG_1,
        0,
        0,
    ));
    b.emit(BpfInsn::new(
        BPF_ALU64 | BPF_MOV | BPF_X,
        BPF_REG_1,
        BPF_REG_6,
        0,
        0,
    ));
    b.emit(BpfInsn::new(
        BPF_JMP | BPF_CALL,
        0,
        0,
        0,
        COMPAT_GET_SOCKET_UID,
    ));
    b.emit(BpfInsn::new(
        BPF_STX | BPF_W | BPF_MEM,
        BPF_REG_10,
        BPF_REG_0,
        l.scratch_stack_off,
        0,
    ));
    b.emit_map_fd(BPF_REG_1, uid_map_fd);
    b.emit(BpfInsn::new(
        BPF_ALU64 | BPF_MOV | BPF_X,
        BPF_REG_2,
        BPF_REG_10,
        0,
        0,
    ));
    b.emit(BpfInsn::new(
        BPF_ALU64 | BPF_ADD | BPF_K,
        BPF_REG_2,
        0,
        0,
        i32::from(l.scratch_stack_off),
    ));
    b.emit(BpfInsn::new(
        BPF_JMP | BPF_CALL,
        0,
        0,
        0,
        BPF_FUNC_MAP_LOOKUP_ELEM,
    ));
    let uid_miss = b.emit_jump(BpfInsn::new(BPF_JMP | BPF_JEQ | BPF_K, BPF_REG_0, 0, 0, 0));

    let load_fail = b.emit_load_destination(l, mode);
    b.emit_cidr_lookup(cidr_map_fd, l);
    let cidr_miss = b.emit_jump(BpfInsn::new(BPF_JMP | BPF_JEQ | BPF_K, BPF_REG_0, 0, 0, 0));

    b.emit_result(XT_BPF_MATCH);
    b.patch_jump_here(uid_miss);
    b.patch_jump_here(cidr_miss);
    if let Some(idx) = load_fail {
        b.patch_jump_here(idx);
    }
    b.emit_result(XT_BPF_MISS);
    b
}

fn build_uid_insns(uid_map_fd: c_int) -> ProgBuilder {
    let mut b = ProgBuilder::new();
    b.emit(BpfInsn::new(
        BPF_JMP | BPF_CALL,
        0,
        0,
        0,
        COMPAT_GET_SOCKET_UID,
    ));
    b.emit(BpfInsn::new(
        BPF_STX | BPF_W | BPF_MEM,
        BPF_REG_10,
        BPF_REG_0,
        -4,
        0,
    ));
    b.emit_map_fd(BPF_REG_1, uid_map_fd);
    b.emit(BpfInsn::new(
        BPF_ALU64 | BPF_MOV | BPF_X,
        BPF_REG_2,
        BPF_REG_10,
        0,
        0,
    ));
    b.emit(BpfInsn::new(
        BPF_ALU64 | BPF_ADD | BPF_K,
        BPF_REG_2,
        0,
        0,
        -4,
    ));
    b.emit(BpfInsn::new(
        BPF_JMP | BPF_CALL,
        0,
        0,
        0,
        BPF_FUNC_MAP_LOOKUP_ELEM,
    ));
    let uid_miss = b.emit_jump(BpfInsn::new(BPF_JMP | BPF_JEQ | BPF_K, BPF_REG_0, 0, 0, 0));
    b.emit_result(XT_BPF_MATCH);
    b.patch_jump_here(uid_miss);
    b.emit_result(XT_BPF_MISS);
    b
}

fn load_socket_filter(b: &ProgBuilder, name: &[u8], log_error: bool) -> c_int {
    let name_str = String::from_utf8_lossy(name);
    if b.overflow {
        if log_error {
            eprintln!("BPF program {name_str} is too large");
        }
        return -1;
    }

    let mut verifier_log = vec![0u8; VERIFY_LOG_SIZE];
    let license = b"GPL\0";
    let mut attr = BpfAttrRaw::zeroed();
    attr.write_u32(0, BPF_PROG_TYPE_SOCKET_FILTER);
    attr.write_u32(4, b.insns.len() as u32);
    attr.write_u64(8, b.insns.as_ptr() as usize as u64);
    attr.write_u64(16, license.as_ptr() as usize as u64);
    attr.write_u32(24, 1);
    attr.write_u32(28, VERIFY_LOG_SIZE as u32);
    attr.write_u64(32, verifier_log.as_mut_ptr() as usize as u64);

    let copy_len = name.len().min(BPF_OBJ_NAME_LEN - 1);
    attr.bytes[48..48 + copy_len].copy_from_slice(&name[..copy_len]);

    let fd = bpf_syscall(BPF_PROG_LOAD, &attr);
    if fd < 0 && log_error {
        let (code, msg) = last_errno_and_msg();
        eprintln!(
            "eBPF program load rejected: program={name_str} errno={code} ({msg}). \
             The device kernel or SELinux policy does not support this socket-filter eBPF matcher. \
             Use standard/ipset rule matching on this device."
        );
        if verifier_log[0] != 0 {
            let end = verifier_log
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(verifier_log.len());
            eprintln!(
                "verifier log:\n{}",
                String::from_utf8_lossy(&verifier_log[..end])
            );
        }
    }
    fd
}

fn load_with_read_fallback(build: impl Fn(ReadMode) -> ProgBuilder, name: &[u8]) -> c_int {
    const ORDER: [ReadMode; 3] = [ReadMode::Direct, ReadMode::Helper, ReadMode::LdAbs];
    for (idx, mode) in ORDER.iter().copied().enumerate() {
        let b = build(mode);
        let fd = load_socket_filter(&b, name, idx + 1 == ORDER.len());
        if fd >= 0 {
            return fd;
        }
    }
    -1
}

/// Port of `int load_program(...)` (`loader.c:404`).
pub fn load_program(
    section_name: Option<&[u8]>,
    program_name: Option<&[u8]>,
    cidr4_fd: c_int,
    cidr6_fd: c_int,
    force_uid_fd: c_int,
    app_uid_fd: c_int,
) -> c_int {
    let Some(section) = section_name else {
        return -1;
    };
    let name = program_name.unwrap_or(b"");
    match section {
        b"socket/cidr4" => {
            load_with_read_fallback(|m| build_cidr_insns(cidr4_fd, &LAYOUT_V4, m), name)
        }
        b"socket/cidr6" => {
            load_with_read_fallback(|m| build_cidr_insns(cidr6_fd, &LAYOUT_V6, m), name)
        }
        b"socket/force4" => load_with_read_fallback(
            |m| build_force_insns(cidr4_fd, force_uid_fd, &LAYOUT_V4, m),
            name,
        ),
        b"socket/force6" => load_with_read_fallback(
            |m| build_force_insns(cidr6_fd, force_uid_fd, &LAYOUT_V6, m),
            name,
        ),
        b"socket/appuid" => {
            let b = build_uid_insns(app_uid_fd);
            load_socket_filter(&b, name, true)
        }
        _ => {
            eprintln!(
                "unknown BPF program section: {}",
                String::from_utf8_lossy(section)
            );
            -1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bpf_insn_matches_kernel_abi_size_and_register_packing() {
        assert_eq!(std::mem::size_of::<BpfInsn>(), 8);
        assert_eq!(std::mem::size_of::<BpfAttrRaw>(), 168);
        let mov = BpfInsn::new(BPF_ALU64 | BPF_MOV | BPF_X, BPF_REG_6, BPF_REG_1, 0, 0);
        assert_eq!(mov.code, 0xbf);
        assert_eq!(mov.regs, 0x16);
    }

    #[test]
    fn generated_cidr4_direct_program_has_valid_jumps_and_no_overflow() {
        let b = build_cidr_insns(7, &LAYOUT_V4, ReadMode::Direct);
        assert!(!b.overflow);
        assert!(b.insns.len() < MAX_PROGRAM_INSNS);
        // First instruction saves skb ctx in r6; last two return XT_BPF_MISS + EXIT.
        assert_eq!(b.insns[0], BpfInsn::new(0xbf, BPF_REG_6, BPF_REG_1, 0, 0));
        assert_eq!(b.insns[b.insns.len() - 1].code, BPF_JMP | BPF_EXIT);
        assert_eq!(b.insns[b.insns.len() - 2].imm, XT_BPF_MISS);
    }

    #[test]
    fn generated_force6_and_uid_programs_fit_in_budget() {
        for mode in [ReadMode::Direct, ReadMode::Helper, ReadMode::LdAbs] {
            let b = build_force_insns(3, 4, &LAYOUT_V6, mode);
            assert!(!b.overflow);
            assert!(b.insns.len() < MAX_PROGRAM_INSNS);
        }
        let u = build_uid_insns(5);
        assert!(!u.overflow);
        assert_eq!(u.insns.len(), 12);
    }
}
