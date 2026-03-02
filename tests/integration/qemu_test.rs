//! QEMU integration entrypoint notes.
//!
//! Workspace-level integration tests run under `drishti-daemon/tests`.
//! Cross-arch runtime validation is driven by:
//! - `cargo run -p xtask -- qemu smoke --arch x86_64`
//! - `cargo run -p xtask -- qemu smoke --arch aarch64`
//!
//! The QEMU lane emits deterministic artifacts under `target/qemu/<arch>/`:
//! `serial.log`, `smoke.log`, `metrics.prom`, and `summary.json`.
