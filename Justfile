set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default:
    @just --list

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

lint:
    cargo clippy --workspace --all-targets -- -D warnings

test:
    cargo test --workspace

validate-skills:
    python3 scripts/validate_codex_skills.py

build:
    cargo build --workspace

build-ebpf:
    cargo run -p xtask -- build-ebpf

qemu-prepare arch="x86_64":
    cargo run -p xtask -- qemu prepare --arch {{arch}}

qemu-run arch="x86_64" kvm="auto" timeout="180":
    cargo run -p xtask -- qemu run --arch {{arch}} --kvm {{kvm}} --timeout-secs {{timeout}}

qemu-smoke arch="x86_64" kvm="auto" timeout="180":
    cargo run -p xtask -- qemu smoke --arch {{arch}} --kvm {{kvm}} --timeout-secs {{timeout}}

qemu-smoke-x86 timeout="180":
    cargo run -p xtask -- qemu smoke --arch x86_64 --kvm auto --timeout-secs {{timeout}}

qemu-smoke-arm64 timeout="300":
    cargo run -p xtask -- qemu smoke --arch aarch64 --kvm off --timeout-secs {{timeout}}

qemu-ci arch="x86_64" kvm="auto" timeout="180":
    cargo run -p xtask -- qemu ci --arch {{arch}} --kvm {{kvm}} --timeout-secs {{timeout}}

run:
    cargo run -p drishti-daemon -- --config config/drishti.toml

docs-install:
    cd drishti-docs && npm ci

docs-dev:
    cd drishti-docs && npm run start

docs-build:
    cd drishti-docs && npm run build

docs-verify:
    cd drishti-docs && npm ci && npm run check:mermaid && npm run build
