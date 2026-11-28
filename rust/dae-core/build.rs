use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let port_rs = manifest_dir.join("../../webui/rust/dae-rs/src/port.rs");
    println!("cargo:rerun-if-changed={}", port_rs.display());
    let content = fs::read_to_string(&port_rs).expect("read dae-rs/src/port.rs");
    let filtered: Vec<&str> = content
        .lines()
        .filter(|line| {
            let t = line.trim();
            !(t.starts_with("mod ") && t.ends_with(';'))
                && t != "#![forbid(unsafe_code)]"
                && !t.starts_with("//!")
        })
        .collect();
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("port_body.rs");
    fs::write(out, filtered.join("\n")).expect("write port_body.rs");
}
