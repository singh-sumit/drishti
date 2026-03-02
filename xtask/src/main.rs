use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, bail, ensure};
use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(name = "xtask", about = "drishti developer automation tasks")]
struct Cli {
    #[command(subcommand)]
    command: CommandKind,
}

#[derive(Debug, Subcommand)]
enum CommandKind {
    BuildEbpf {
        #[arg(long, default_value = "nightly")]
        toolchain: String,
        #[arg(long, default_value = "bpfel-unknown-none")]
        target: String,
    },
    ValidateSkills {
        #[arg(long, default_value = ".codex/skills")]
        root: PathBuf,
    },
    Qemu {
        #[command(subcommand)]
        command: QemuCommand,
    },
}

#[derive(Debug, Subcommand)]
enum QemuCommand {
    Prepare {
        #[arg(long, value_enum, default_value_t = QemuArch::X86_64)]
        arch: QemuArch,
        #[arg(long)]
        skip_build: bool,
    },
    Run {
        #[arg(long, value_enum, default_value_t = QemuArch::X86_64)]
        arch: QemuArch,
        #[arg(long, value_enum, default_value_t = KvmMode::Auto)]
        kvm: KvmMode,
        #[arg(long, default_value_t = 180)]
        timeout_secs: u64,
    },
    Smoke {
        #[arg(long, value_enum, default_value_t = QemuArch::X86_64)]
        arch: QemuArch,
        #[arg(long, value_enum, default_value_t = KvmMode::Auto)]
        kvm: KvmMode,
        #[arg(long, default_value_t = 180)]
        timeout_secs: u64,
        #[arg(long)]
        skip_build: bool,
    },
    Ci {
        #[arg(long, value_enum, default_value_t = QemuArch::X86_64)]
        arch: QemuArch,
        #[arg(long, value_enum, default_value_t = KvmMode::Auto)]
        kvm: KvmMode,
        #[arg(long, default_value_t = 180)]
        timeout_secs: u64,
        #[arg(long)]
        skip_build: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum QemuArch {
    #[value(name = "x86_64", alias = "x86-64")]
    X86_64,
    #[value(name = "aarch64")]
    Aarch64,
}

impl QemuArch {
    fn as_str(self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64",
            Self::Aarch64 => "aarch64",
        }
    }

    fn default_target(self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64-unknown-linux-gnu",
            Self::Aarch64 => "aarch64-unknown-linux-musl",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum KvmMode {
    Auto,
    On,
    Off,
}

impl KvmMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::On => "on",
            Self::Off => "off",
        }
    }
}

#[derive(Debug, Serialize)]
struct ArtifactFile {
    path: String,
    exists: bool,
    bytes: u64,
}

#[derive(Debug, Serialize)]
struct ArtifactIndex {
    arch: String,
    generated_at_unix_secs: u64,
    files: Vec<ArtifactFile>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        CommandKind::BuildEbpf { toolchain, target } => build_ebpf(&toolchain, &target),
        CommandKind::ValidateSkills { root } => validate_skills(&root),
        CommandKind::Qemu { command } => run_qemu_command(command),
    }
}

fn run_qemu_command(command: QemuCommand) -> Result<()> {
    match command {
        QemuCommand::Prepare { arch, skip_build } => qemu_prepare(arch, skip_build),
        QemuCommand::Run {
            arch,
            kvm,
            timeout_secs,
        } => qemu_run(arch, kvm, timeout_secs),
        QemuCommand::Smoke {
            arch,
            kvm,
            timeout_secs,
            skip_build,
        } => qemu_smoke(arch, kvm, timeout_secs, skip_build),
        QemuCommand::Ci {
            arch,
            kvm,
            timeout_secs,
            skip_build,
        } => qemu_ci(arch, kvm, timeout_secs, skip_build),
    }
}

fn qemu_prepare(arch: QemuArch, skip_build: bool) -> Result<()> {
    if !skip_build {
        build_qemu_binaries(arch)?;
    }

    let daemon_bin = daemon_binary_path(arch);
    let smoke_bin = smoke_binary_path(arch);

    ensure!(
        daemon_bin.exists(),
        "missing daemon binary at {}; run without --skip-build to compile it",
        daemon_bin.display()
    );
    ensure!(
        smoke_bin.exists(),
        "missing qemu smoke binary at {}; run without --skip-build to compile it",
        smoke_bin.display()
    );

    let envs = vec![
        (
            "DRISHTI_QEMU_DAEMON_BIN".to_string(),
            daemon_bin.display().to_string(),
        ),
        (
            "DRISHTI_QEMU_SMOKE_BIN".to_string(),
            smoke_bin.display().to_string(),
        ),
    ];

    run_qemu_script(
        "prepare.sh",
        &["--arch", arch.as_str()],
        &envs,
        "failed to prepare QEMU initramfs artifacts",
    )
}

fn qemu_run(arch: QemuArch, kvm: KvmMode, timeout_secs: u64) -> Result<()> {
    let timeout = timeout_secs.to_string();
    run_qemu_script(
        "run.sh",
        &[
            "--arch",
            arch.as_str(),
            "--kvm",
            kvm.as_str(),
            "--timeout",
            &timeout,
        ],
        &[],
        "QEMU run command failed",
    )
}

fn qemu_smoke(arch: QemuArch, kvm: KvmMode, timeout_secs: u64, skip_build: bool) -> Result<()> {
    qemu_prepare(arch, skip_build)?;

    let timeout = timeout_secs.to_string();
    run_qemu_script(
        "run_tests.sh",
        &[
            "--arch",
            arch.as_str(),
            "--kvm",
            kvm.as_str(),
            "--timeout",
            &timeout,
            "--skip-prepare",
        ],
        &[],
        "QEMU smoke test command failed",
    )
}

fn qemu_ci(arch: QemuArch, kvm: KvmMode, timeout_secs: u64, skip_build: bool) -> Result<()> {
    qemu_smoke(arch, kvm, timeout_secs, skip_build)?;
    write_artifact_index(arch)
}

fn build_qemu_binaries(arch: QemuArch) -> Result<()> {
    build_ebpf("nightly", "bpfel-unknown-none")?;
    let ebpf_object = find_ebpf_object()?;
    let ebpf_fingerprint = ebpf_object_fingerprint(&ebpf_object)?;

    let target = arch.default_target();
    let build_tool = select_build_tool(arch);
    if build_tool == "cargo" {
        ensure_rust_target_installed(target)?;
    }
    let embedded_bpf_path = if build_tool == "cross" {
        to_cross_container_path(&ebpf_object)?
    } else {
        ebpf_object
    };

    let mut command = Command::new(&build_tool);
    command.args([
        "build",
        "-p",
        "drishti-daemon",
        "--features",
        "ebpf-runtime",
        "--release",
        "--target",
        target,
        "--bin",
        "drishti-daemon",
        "--bin",
        "qemu_smoke",
    ]);
    command.env("DRISHTI_EMBEDDED_BPF_PATH", &embedded_bpf_path);
    command.env("DRISHTI_EMBEDDED_BPF_META", &ebpf_fingerprint);

    let status = command
        .status()
        .with_context(|| format!("failed to invoke {build_tool} for target {target}"))?;

    if !status.success() {
        bail!("failed to build drishti-daemon + qemu_smoke for {target} using {build_tool}");
    }

    Ok(())
}

fn to_cross_container_path(host_path: &Path) -> Result<PathBuf> {
    let root = workspace_root();
    let relative = host_path.strip_prefix(&root).with_context(|| {
        format!(
            "failed to map {} into cross container path; expected it under workspace {}",
            host_path.display(),
            root.display()
        )
    })?;

    Ok(Path::new("/project").join(relative))
}

fn ensure_rust_target_installed(target: &str) -> Result<()> {
    let output = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .context("failed to query installed rust targets via rustup")?;

    ensure!(
        output.status.success(),
        "rustup target listing failed; install rustup and run `rustup target add {target}`"
    );

    let installed =
        String::from_utf8(output.stdout).context("rustup target output was not valid UTF-8")?;
    if installed.lines().any(|line| line.trim() == target) {
        return Ok(());
    }

    bail!("rust target `{target}` is not installed. Install it with: `rustup target add {target}`");
}

fn select_build_tool(arch: QemuArch) -> String {
    if let Ok(tool) = env::var("DRISHTI_QEMU_BUILD_TOOL") {
        return tool;
    }

    if matches!(arch, QemuArch::Aarch64) && command_exists("cross") {
        return "cross".to_string();
    }

    "cargo".to_string()
}

fn find_ebpf_object() -> Result<PathBuf> {
    let root = workspace_root();
    let candidates = [
        root.join("target/bpfel-unknown-none/release/drishti-ebpf"),
        root.join("target/bpfel-unknown-none/release/drishti-ebpf.o"),
        root.join("target/bpfel-unknown-none/release/drishti-ebpf.bin"),
    ];

    for candidate in candidates {
        if candidate.exists() && is_ebpf_elf(&candidate)? {
            return Ok(candidate);
        }
    }

    let deps_dir = root.join("target/bpfel-unknown-none/release/deps");
    if deps_dir.exists() {
        for entry in fs::read_dir(&deps_dir)
            .with_context(|| format!("failed to read {}", deps_dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let file_name = path
                .file_name()
                .map(|value| value.to_string_lossy())
                .unwrap_or_default();
            if file_name.starts_with("drishti_ebpf-")
                && !file_name.ends_with(".d")
                && is_ebpf_elf(&path)?
            {
                return Ok(path);
            }
        }
    }

    bail!(
        "unable to find compiled eBPF object under target/bpfel-unknown-none/release; run `cargo run -p xtask -- build-ebpf` and ensure drishti-ebpf binary target builds"
    )
}

fn ebpf_object_fingerprint(path: &Path) -> Result<String> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("failed to stat eBPF object at {}", path.display()))?;
    let modified_nanos = metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map_or(0_u128, |value| value.as_nanos());
    Ok(format!("{}:{modified_nanos}", metadata.len()))
}

fn is_ebpf_elf(path: &Path) -> Result<bool> {
    const ELF_MAGIC: &[u8; 4] = b"\x7fELF";
    const ELF_CLASS_64: u8 = 2;
    const ELFDATA2LSB: u8 = 1;
    const ELFDATA2MSB: u8 = 2;
    const EM_BPF: u16 = 247;

    let bytes = fs::read(path)
        .with_context(|| format!("failed to read potential eBPF object {}", path.display()))?;
    if bytes.len() < 20 {
        return Ok(false);
    }
    if &bytes[0..4] != ELF_MAGIC {
        return Ok(false);
    }
    if bytes[4] != ELF_CLASS_64 {
        return Ok(false);
    }

    let machine = match bytes[5] {
        ELFDATA2LSB => u16::from_le_bytes([bytes[18], bytes[19]]),
        ELFDATA2MSB => u16::from_be_bytes([bytes[18], bytes[19]]),
        _ => return Ok(false),
    };

    Ok(machine == EM_BPF)
}

fn daemon_binary_path(arch: QemuArch) -> PathBuf {
    binary_path(arch, "drishti-daemon")
}

fn smoke_binary_path(arch: QemuArch) -> PathBuf {
    binary_path(arch, "qemu_smoke")
}

fn binary_path(arch: QemuArch, name: &str) -> PathBuf {
    let root = workspace_root();
    root.join("target")
        .join(arch.default_target())
        .join("release")
        .join(name)
}

fn run_qemu_script(
    script_name: &str,
    args: &[&str],
    envs: &[(String, String)],
    failure_message: &str,
) -> Result<()> {
    let script_path = workspace_root().join("deploy/qemu").join(script_name);
    ensure!(
        script_path.exists(),
        "missing script {}",
        script_path.display()
    );

    let mut command = Command::new(&script_path);
    command.args(args);
    for (key, value) in envs {
        command.env(key, value);
    }

    let status = command
        .status()
        .with_context(|| format!("failed to execute {}", script_path.display()))?;

    if !status.success() {
        bail!("{failure_message}: {}", script_path.display());
    }

    Ok(())
}

fn write_artifact_index(arch: QemuArch) -> Result<()> {
    let artifact_root = workspace_root().join("target/qemu").join(arch.as_str());
    fs::create_dir_all(&artifact_root)
        .with_context(|| format!("failed to create {}", artifact_root.display()))?;

    let files = ["serial.log", "smoke.log", "metrics.prom", "summary.json"]
        .into_iter()
        .map(|name| {
            let path = artifact_root.join(name);
            let metadata = fs::metadata(&path).ok();
            ArtifactFile {
                path: format!("target/qemu/{}/{}", arch.as_str(), name),
                exists: path.exists(),
                bytes: metadata.map_or(0, |meta| meta.len()),
            }
        })
        .collect::<Vec<_>>();

    let generated_at_unix_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());

    let index = ArtifactIndex {
        arch: arch.as_str().to_string(),
        generated_at_unix_secs,
        files,
    };

    let index_path = artifact_root.join("index.json");
    let payload = serde_json::to_string_pretty(&index).context("failed to serialize index")?;
    fs::write(&index_path, payload)
        .with_context(|| format!("failed to write {}", index_path.display()))?;

    println!("wrote QEMU artifact index to {}", index_path.display());
    Ok(())
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root should exist")
        .to_path_buf()
}

fn command_exists(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn build_ebpf(toolchain: &str, target: &str) -> Result<()> {
    let rustup_llvm_dir = rustup_llvm_dir(toolchain)?;
    let llvm_proxy_dir = workspace_root().join("target/llvm-proxy").join(toolchain);
    fs::create_dir_all(&llvm_proxy_dir)
        .with_context(|| format!("failed to create {}", llvm_proxy_dir.display()))?;

    let llvm_real = find_real_llvm_dylib(&rustup_llvm_dir)?;
    let proxy_lib = llvm_proxy_dir.join(
        llvm_real
            .file_name()
            .context("LLVM library path missing filename")?,
    );

    if proxy_lib.exists() {
        fs::remove_file(&proxy_lib)
            .with_context(|| format!("failed to remove {}", proxy_lib.display()))?;
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(&llvm_real, &proxy_lib).with_context(|| {
        format!(
            "failed to symlink LLVM dylib {} -> {}",
            proxy_lib.display(),
            llvm_real.display()
        )
    })?;
    #[cfg(not(unix))]
    fs::copy(&llvm_real, &proxy_lib).with_context(|| {
        format!(
            "failed to copy LLVM dylib {} -> {}",
            llvm_real.display(),
            proxy_lib.display()
        )
    })?;

    let mut ld_library_path = llvm_proxy_dir.display().to_string();
    if let Some(existing) = env::var_os("LD_LIBRARY_PATH") {
        let existing = existing.to_string_lossy();
        if !existing.is_empty() {
            ld_library_path.push(':');
            ld_library_path.push_str(&existing);
        }
    }

    let status = Command::new("cargo")
        .args([
            &format!("+{toolchain}"),
            "build",
            "-p",
            "drishti-ebpf",
            "--bin",
            "drishti-ebpf",
            "--target",
            target,
            "-Z",
            "build-std=core",
            "--release",
        ])
        .env(
            "RUSTFLAGS",
            env::var("RUSTFLAGS").unwrap_or_else(|_| {
                "-C debuginfo=2 -C linker=bpf-linker -C link-arg=--llvm-args=--opaque-pointers"
                    .to_string()
            }),
        )
        .env("LD_LIBRARY_PATH", ld_library_path)
        .status()
        .context("failed to execute cargo build for eBPF target")?;

    if !status.success() {
        bail!(
            "eBPF build failed; ensure nightly+rust-src, llvm/clang, and bpf-linker are installed"
        );
    }

    Ok(())
}

fn rustup_llvm_dir(toolchain: &str) -> Result<PathBuf> {
    let output = Command::new("rustup")
        .args(["which", "--toolchain", toolchain, "rustc"])
        .output()
        .with_context(|| format!("failed to resolve rustc for toolchain {toolchain}"))?;
    if !output.status.success() {
        bail!(
            "rustup could not resolve rustc for toolchain {toolchain}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let rustc_path = String::from_utf8(output.stdout)
        .context("rustup output for rustc path was not valid UTF-8")?
        .trim()
        .to_string();
    let rustc_path = PathBuf::from(rustc_path);

    let toolchain_root = rustc_path
        .parent()
        .and_then(Path::parent)
        .context("unable to derive rustup toolchain root")?;
    Ok(toolchain_root.join("lib"))
}

fn find_real_llvm_dylib(lib_dir: &Path) -> Result<PathBuf> {
    let mut candidates = Vec::new();
    for entry in
        fs::read_dir(lib_dir).with_context(|| format!("failed to read {}", lib_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().map(|value| value.to_string_lossy()) else {
            continue;
        };
        if !name.starts_with("libLLVM") {
            continue;
        }
        if !name.contains(".so") {
            continue;
        }
        let size = fs::metadata(&path)
            .with_context(|| format!("failed to stat {}", path.display()))?
            .len();
        if size > 1024 * 1024 {
            candidates.push(path);
        }
    }

    candidates.sort();
    candidates
        .into_iter()
        .next()
        .context("unable to locate a real libLLVM shared library in rustup toolchain")
}

fn validate_skills(root: &PathBuf) -> Result<()> {
    if !root.exists() {
        bail!("skill directory {} does not exist", root.display());
    }

    for entry in
        std::fs::read_dir(root).with_context(|| format!("failed reading {}", root.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let skill = path.join("SKILL.md");
        if !skill.exists() {
            bail!("missing SKILL.md in {}", path.display());
        }

        let raw = std::fs::read_to_string(&skill)
            .with_context(|| format!("failed reading {}", skill.display()))?;
        if !raw.starts_with("---\n") {
            bail!("{} is missing YAML frontmatter", skill.display());
        }

        let name_exists = raw
            .lines()
            .any(|line| line.trim_start().starts_with("name:"));
        let description_exists = raw
            .lines()
            .any(|line| line.trim_start().starts_with("description:"));

        if !(name_exists && description_exists) {
            bail!(
                "{} frontmatter must include name and description",
                skill.display()
            );
        }
    }

    Ok(())
}
