//! Glue from the front door to `config-rs` and `loader-rs` (full port of `boxproxy/boxbpf/main.c`).

use std::ffi::c_int;
use std::io;
use std::path::Path;
use std::process::ExitCode;

use clap::ArgMatches;
use config_rs::port::{
    Config, Lpm4Key, Lpm6Key, config_defaults, load_cidr_file, load_uid_file, read_config_json,
    uid_file_has_entries,
};
use loader_rs::port::{
    BPF_MAP_TYPE_HASH, BPF_MAP_TYPE_LPM_TRIE, COMPAT_MAP_NO_PREALLOC, PIN_DIR, PROBE_MAP_APP_UID,
    PROBE_MAP_CIDR4, PROBE_MAP_CIDR6, PROBE_MAP_FORCE_UID, PROBE_PIN4, PROBE_PIN6, clear_map,
    close_fd, create_map, get_pinned, load_program, pin_replace, raise_memlock, remove_known_pins,
};

const MAX_CIDRS: u32 = 65536;
const MAX_UIDS: u32 = 8192;

const STATUS_OK: u8 = 0;
const STATUS_UNSUPPORTED: u8 = 1;
const STATUS_CONFIG_ERROR: u8 = 2;
const STATUS_UPDATE_ERROR: u8 = 3;

struct Fds {
    cidr4: c_int,
    cidr6: c_int,
    force_uid: c_int,
    app_uid: c_int,
}

impl Fds {
    const fn new() -> Self {
        Self {
            cidr4: -1,
            cidr6: -1,
            force_uid: -1,
            app_uid: -1,
        }
    }

    fn close_all(&mut self) {
        close_fd(self.cidr4);
        close_fd(self.cidr6);
        close_fd(self.force_uid);
        close_fd(self.app_uid);
        *self = Self::new();
    }
}

fn os_errno_and_msg() -> (i32, String) {
    let err = io::Error::last_os_error();
    let code = err.raw_os_error().unwrap_or(0);
    let full = err.to_string();
    let suffix = format!(" (os error {code})");
    let msg = full.strip_suffix(&suffix).unwrap_or(&full).to_string();
    (code, msg)
}

fn has_arg(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

fn arg_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    for i in 0..args.len().saturating_sub(1) {
        if args[i] == flag {
            return Some(args[i + 1].as_str());
        }
    }
    None
}

fn print_usage(argv0: &str) {
    eprintln!(
        "Usage: {argv0} --probe [--ipv6 1]\n       \
         {argv0} --init --config FILE\n       \
         {argv0} --apply --config FILE\n       \
         {argv0} --update --config FILE\n       \
         {argv0} --clear"
    );
}

fn create_cidr_map(source: &str, pin_path: &str, ipv6: bool, fd_out: &mut c_int) -> u8 {
    let key_size = if ipv6 {
        std::mem::size_of::<Lpm6Key>()
    } else {
        std::mem::size_of::<Lpm4Key>()
    };
    *fd_out = create_map(
        BPF_MAP_TYPE_LPM_TRIE,
        key_size,
        std::mem::size_of::<u8>(),
        MAX_CIDRS,
        COMPAT_MAP_NO_PREALLOC,
    );
    if *fd_out < 0 {
        let (code, msg) = os_errno_and_msg();
        eprintln!(
            "create IPv{} CIDR map failed: errno={code} ({msg})",
            if ipv6 { 6 } else { 4 }
        );
        return STATUS_CONFIG_ERROR;
    }
    let _ = load_cidr_file(*fd_out, Some(source.as_bytes()), ipv6);
    if pin_replace(*fd_out, Some(pin_path.as_bytes())) == 0 {
        STATUS_OK
    } else {
        STATUS_CONFIG_ERROR
    }
}

fn create_uid_map(source: &str, pin_path: &str, required: bool, fd_out: &mut c_int) -> u8 {
    *fd_out = create_map(
        BPF_MAP_TYPE_HASH,
        std::mem::size_of::<u32>(),
        std::mem::size_of::<u8>(),
        MAX_UIDS,
        0,
    );
    if *fd_out < 0 {
        let (code, msg) = os_errno_and_msg();
        eprintln!("create UID map failed: errno={code} ({msg})");
        return STATUS_CONFIG_ERROR;
    }

    let loaded = load_uid_file(*fd_out, Some(source.as_bytes()));
    if required && loaded <= 0 {
        let label = if source.is_empty() { "-" } else { source };
        eprintln!("required UID map is empty: {label}");
        return STATUS_CONFIG_ERROR;
    }

    if pin_replace(*fd_out, Some(pin_path.as_bytes())) == 0 {
        STATUS_OK
    } else {
        STATUS_CONFIG_ERROR
    }
}

fn pin_program(section: &str, name: &str, pin_path: &str, fds: &Fds) -> u8 {
    let fd = load_program(
        Some(section.as_bytes()),
        Some(name.as_bytes()),
        fds.cidr4,
        fds.cidr6,
        fds.force_uid,
        fds.app_uid,
    );
    if fd < 0 {
        return STATUS_CONFIG_ERROR;
    }
    let rc = pin_replace(fd, Some(pin_path.as_bytes()));
    close_fd(fd);
    if rc == 0 {
        STATUS_OK
    } else {
        STATUS_CONFIG_ERROR
    }
}

fn apply_config(config: &Config) -> u8 {
    remove_known_pins();

    let mut fds = Fds::new();
    let app_uid_required = uid_file_has_entries(Some(config.app_uid_file.as_bytes()));

    let status = (|| {
        if create_cidr_map(&config.cidr4_file, &config.map_cidr4, false, &mut fds.cidr4)
            != STATUS_OK
        {
            return STATUS_CONFIG_ERROR;
        }
        if create_cidr_map(&config.cidr6_file, &config.map_cidr6, true, &mut fds.cidr6) != STATUS_OK
        {
            return STATUS_CONFIG_ERROR;
        }
        if create_uid_map(
            &config.force_uid_file,
            &config.map_force_uid,
            false,
            &mut fds.force_uid,
        ) != STATUS_OK
        {
            return STATUS_CONFIG_ERROR;
        }
        if create_uid_map(
            &config.app_uid_file,
            &config.map_app_uid,
            app_uid_required,
            &mut fds.app_uid,
        ) != STATUS_OK
        {
            return STATUS_CONFIG_ERROR;
        }

        if pin_program("socket/cidr4", "cidr4", &config.pin_cidr_out4, &fds) != STATUS_OK {
            return STATUS_CONFIG_ERROR;
        }
        if pin_program("socket/cidr4", "pre4", &config.pin_cidr_pre4, &fds) != STATUS_OK {
            return STATUS_CONFIG_ERROR;
        }
        if pin_program("socket/force4", "force4", &config.pin_force_out4, &fds) != STATUS_OK {
            return STATUS_CONFIG_ERROR;
        }
        if pin_program("socket/appuid", "app4", &config.pin_app_out4, &fds) != STATUS_OK {
            return STATUS_CONFIG_ERROR;
        }

        if config.ipv6 {
            if pin_program("socket/cidr6", "cidr6", &config.pin_cidr_out6, &fds) != STATUS_OK {
                return STATUS_CONFIG_ERROR;
            }
            if pin_program("socket/cidr6", "pre6", &config.pin_cidr_pre6, &fds) != STATUS_OK {
                return STATUS_CONFIG_ERROR;
            }
            if pin_program("socket/force6", "force6", &config.pin_force_out6, &fds) != STATUS_OK {
                return STATUS_CONFIG_ERROR;
            }
            if pin_program("socket/appuid", "app6", &config.pin_app_out6, &fds) != STATUS_OK {
                return STATUS_CONFIG_ERROR;
            }
        }

        STATUS_OK
    })();

    if status != STATUS_OK {
        remove_known_pins();
    }
    fds.close_all();
    status
}

fn update_config(config: &Config) -> u8 {
    let cidr4 = get_pinned(Some(config.map_cidr4.as_bytes()));
    let cidr6 = get_pinned(Some(config.map_cidr6.as_bytes()));
    let force_uid = get_pinned(Some(config.map_force_uid.as_bytes()));
    let app_uid = get_pinned(Some(config.map_app_uid.as_bytes()));

    if cidr4 < 0 || (config.ipv6 && cidr6 < 0) || force_uid < 0 || app_uid < 0 {
        close_fd(cidr4);
        close_fd(cidr6);
        close_fd(force_uid);
        close_fd(app_uid);
        return apply_config(config);
    }

    let mut status = STATUS_OK;
    if clear_map(cidr4, std::mem::size_of::<Lpm4Key>()) < 0 {
        status = STATUS_UPDATE_ERROR;
    }
    let _ = load_cidr_file(cidr4, Some(config.cidr4_file.as_bytes()), false);
    if config.ipv6 {
        if clear_map(cidr6, std::mem::size_of::<Lpm6Key>()) < 0 {
            status = STATUS_UPDATE_ERROR;
        }
        let _ = load_cidr_file(cidr6, Some(config.cidr6_file.as_bytes()), true);
    }
    if clear_map(force_uid, std::mem::size_of::<u32>()) < 0 {
        status = STATUS_UPDATE_ERROR;
    }
    let _ = load_uid_file(force_uid, Some(config.force_uid_file.as_bytes()));
    if clear_map(app_uid, std::mem::size_of::<u32>()) < 0 {
        status = STATUS_UPDATE_ERROR;
    }
    let _ = load_uid_file(app_uid, Some(config.app_uid_file.as_bytes()));

    close_fd(cidr4);
    close_fd(cidr6);
    close_fd(force_uid);
    close_fd(app_uid);
    status
}

fn run_with_config(config_path: &str, update_only: bool) -> u8 {
    let Some(config) = read_config_json(Path::new(config_path)) else {
        return STATUS_CONFIG_ERROR;
    };
    if update_only {
        update_config(&config)
    } else {
        apply_config(&config)
    }
}

fn probe_runtime(ipv6: bool) -> u8 {
    let mut config = config_defaults();
    config.map_cidr4 = PROBE_MAP_CIDR4.to_string();
    config.map_cidr6 = PROBE_MAP_CIDR6.to_string();
    config.map_force_uid = PROBE_MAP_FORCE_UID.to_string();
    config.map_app_uid = PROBE_MAP_APP_UID.to_string();

    let mut fds = Fds::new();
    let ok = (|| {
        if create_cidr_map("/dev/null", &config.map_cidr4, false, &mut fds.cidr4) != STATUS_OK {
            return false;
        }
        if create_cidr_map("/dev/null", &config.map_cidr6, true, &mut fds.cidr6) != STATUS_OK {
            return false;
        }
        if create_uid_map("", &config.map_force_uid, false, &mut fds.force_uid) != STATUS_OK {
            return false;
        }
        if create_uid_map("", &config.map_app_uid, false, &mut fds.app_uid) != STATUS_OK {
            return false;
        }

        let mut ok = pin_program("socket/cidr4", "probe4", PROBE_PIN4, &fds) == STATUS_OK;
        if ipv6 {
            ok = pin_program("socket/cidr6", "probe6", PROBE_PIN6, &fds) == STATUS_OK && ok;
        }
        ok
    })();

    fds.close_all();
    let _ = std::fs::remove_file(PROBE_PIN4);
    let _ = std::fs::remove_file(PROBE_PIN6);
    let _ = std::fs::remove_file(&config.map_cidr4);
    let _ = std::fs::remove_file(&config.map_cidr6);
    let _ = std::fs::remove_file(&config.map_force_uid);
    let _ = std::fs::remove_file(&config.map_app_uid);
    let _ = std::fs::remove_dir(PIN_DIR);

    let flag = i32::from(ok);
    println!("supported={flag}");
    println!(
        "message={}",
        if ok { "ok" } else { "eBPF xt_bpf unavailable" }
    );
    println!("lpm_ipv4={flag}");
    println!("program_ipv4={flag}");
    println!("pin_ipv4={flag}");
    if ipv6 {
        println!("lpm_ipv6={flag}");
        println!("program_ipv6={flag}");
        println!("pin_ipv6={flag}");
    }
    if ok { STATUS_OK } else { STATUS_UNSUPPORTED }
}

pub fn run(_command: &str, _matches: &ArgMatches) -> ExitCode {
    raise_memlock();

    let raw_argv: Vec<String> = std::env::args_os()
        .map(|s| s.to_string_lossy().into_owned())
        .collect();
    let argv0 = raw_argv.first().map(String::as_str).unwrap_or("boxbpf");
    let args = raw_argv.get(1..).unwrap_or(&[]);

    let code = if has_arg(args, "--clear") {
        remove_known_pins();
        STATUS_OK
    } else if has_arg(args, "--probe") {
        let ipv6 = arg_value(args, "--ipv6") == Some("1");
        probe_runtime(ipv6)
    } else if has_arg(args, "--init") || has_arg(args, "--apply") {
        match arg_value(args, "--config") {
            Some(cfg) => run_with_config(cfg, false),
            None => {
                eprintln!("--config is required");
                STATUS_CONFIG_ERROR
            }
        }
    } else if has_arg(args, "--update") {
        match arg_value(args, "--config") {
            Some(cfg) => run_with_config(cfg, true),
            None => {
                eprintln!("--config is required");
                STATUS_CONFIG_ERROR
            }
        }
    } else {
        print_usage(argv0);
        STATUS_CONFIG_ERROR
    };

    ExitCode::from(code)
}
