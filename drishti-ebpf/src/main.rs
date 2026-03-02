#![cfg_attr(target_arch = "bpf", no_std)]
#![cfg_attr(target_arch = "bpf", no_main)]

#[cfg(target_arch = "bpf")]
mod programs;

#[cfg(target_arch = "bpf")]
pub use programs::*;

#[cfg(not(target_arch = "bpf"))]
fn main() {}
