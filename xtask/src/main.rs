use std::{env, path::PathBuf, process::Command};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};

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
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        CommandKind::BuildEbpf { toolchain, target } => build_ebpf(&toolchain, &target),
        CommandKind::ValidateSkills { root } => validate_skills(&root),
    }
}

fn build_ebpf(toolchain: &str, target: &str) -> Result<()> {
    let status = Command::new("cargo")
        .args([
            &format!("+{toolchain}"),
            "build",
            "-p",
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
        .status()
        .context("failed to execute cargo build for eBPF target")?;

    if !status.success() {
        bail!("eBPF build failed; ensure nightly, llvm/clang, and bpf-linker are installed");
    }

    Ok(())
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
