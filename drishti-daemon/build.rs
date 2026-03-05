use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    println!("cargo:rerun-if-env-changed=DRISHTI_EMBEDDED_BPF_PATH");
    println!("cargo:rerun-if-env-changed=DRISHTI_EMBEDDED_BPF_META");
    println!("cargo:rerun-if-changed=../drishti-ebpf/src");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR missing"));
    let out_file = out_dir.join("drishti.bpf.o");

    if let Ok(path) = env::var("DRISHTI_EMBEDDED_BPF_PATH") {
        let from = resolve_embedded_bpf_path(&path).unwrap_or_else(|err| {
            panic!(
                "failed to resolve DRISHTI_EMBEDDED_BPF_PATH `{path}`: {err}. \
use an absolute path like `$(pwd)/target/bpfel-unknown-none/release/drishti-ebpf`"
            )
        });
        let from = fs::canonicalize(&from).unwrap_or(from);
        println!("cargo:rerun-if-changed={}", from.display());

        let bytes = fs::read(&from).unwrap_or_else(|err| {
            panic!(
                "failed to read DRISHTI_EMBEDDED_BPF_PATH {}: {err}",
                from.display()
            )
        });

        if bytes.is_empty() {
            panic!("embedded eBPF source object is empty: {}", from.display());
        }

        if !looks_like_bpf_elf(&bytes) {
            panic!(
                "embedded eBPF source object is not an ELF64 eBPF object: {}",
                from.display()
            );
        }

        fs::write(&out_file, bytes)
            .unwrap_or_else(|err| panic!("failed to write {}: {err}", out_file.display()));
        return;
    }

    fs::write(&out_file, [])
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", out_file.display()));
}

fn looks_like_bpf_elf(bytes: &[u8]) -> bool {
    const ELF_MAGIC: &[u8; 4] = b"\x7fELF";
    const ELF_CLASS_64: u8 = 2;
    const ELFDATA2LSB: u8 = 1;
    const ELFDATA2MSB: u8 = 2;
    const EM_BPF: u16 = 247;

    if bytes.len() < 20 {
        return false;
    }
    if &bytes[0..4] != ELF_MAGIC {
        return false;
    }
    if bytes[4] != ELF_CLASS_64 {
        return false;
    }

    let machine = match bytes[5] {
        ELFDATA2LSB => u16::from_le_bytes([bytes[18], bytes[19]]),
        ELFDATA2MSB => u16::from_be_bytes([bytes[18], bytes[19]]),
        _ => return false,
    };

    machine == EM_BPF
}

fn resolve_embedded_bpf_path(raw: &str) -> Result<PathBuf, String> {
    let direct = PathBuf::from(raw);
    if direct.is_absolute() {
        return Ok(direct);
    }

    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR")
            .map_err(|err| format!("CARGO_MANIFEST_DIR missing: {err}"))?,
    );
    let workspace_root = manifest_dir
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| manifest_dir.clone());

    let candidates = [
        direct.clone(),
        manifest_dir.join(raw),
        workspace_root.join(raw),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return Ok(candidate.clone());
        }
    }

    let attempted = candidates
        .iter()
        .map(|value| value.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    Err(format!("path not found; attempted: {attempted}"))
}
