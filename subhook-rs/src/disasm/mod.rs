#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub (crate) mod x86_fam;

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
pub (crate) mod riscv;

#[cfg(target_arch = "arm")]
pub (crate) mod arm32;

#[cfg(target_arch = "aarch64")]
pub (crate) mod aarch64;
