use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=DRISHTI_EMBEDDED_BPF_PATH");
    println!("cargo:rerun-if-changed=../drishti-ebpf/src");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR missing"));
    let out_file = out_dir.join("drishti.bpf.o");

    if let Ok(path) = env::var("DRISHTI_EMBEDDED_BPF_PATH") {
        let from = PathBuf::from(path);
        if from.exists() {
            let _ = fs::copy(from, &out_file);
            return;
        }
    }

    let _ = fs::write(out_file, []);
}
