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
