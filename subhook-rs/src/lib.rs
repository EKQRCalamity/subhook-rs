pub mod error;
pub mod arch {
	/// x86/x86_64 inline hooking with PC-relative instruction relocation.
	/// Handles jumps, calls, and RIP-relative memory operands in trampolines.
	#[cfg_attr(docsrs, doc(cfg(all(feature = "x86_fam", any(target_arch = "x86_64", target_arch = "x86")))))]
	#[cfg(all(feature = "x86_fam", any(target_arch = "x86_64", target_arch = "x86")))]
	pub mod x86_fam {
		/// Subhook-style implementation for x86 architectures.
		#[cfg_attr(docsrs, doc(cfg(feature = "x86_fam_subhook")))]
		#[cfg(feature = "x86_fam_subhook")]
		pub mod subhook;
	}
	
	/// RISC-V (32 and 64 bit) inline hooking with support for compressed instructions.
	/// Relocates AUIPC, JAL, and branch instructions in trampolines.
	#[cfg_attr(docsrs, doc(cfg(all(feature = "riscv", any(target_arch = "riscv32", target_arch = "riscv64")))))]
	#[cfg(all(feature="riscv", any(target_arch = "riscv32", target_arch = "riscv64")))]
	pub mod riscv {
		/// Subhook-style implementation for RISC-V architectures.
		#[cfg_attr(docsrs, doc(cfg(feature = "riscv_subhook")))]
		#[cfg(feature = "riscv_subhook")]
		pub mod subhook;
	}
}

mod mem;
mod disasm;

/// Common utilities and traits for all hooking implementations.
pub mod common {
	pub mod scoped_remove;
	pub use scoped_remove::{Hookable, ScopedRemove};
}

/// Procedural macro for marking functions as hookable.
/// Adds `#[inline(never)]` and converts to `extern "C"` ABI.
#[cfg_attr(docsrs, doc(cfg(feature = "subhook-rs-macros")))]
pub use subhook_rs_macros::hook;
