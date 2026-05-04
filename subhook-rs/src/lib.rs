pub mod error;
pub mod arch {
	#[cfg(all(feature = "x86_fam", any(target_arch = "x86_64", target_arch = "x86")))]
	pub mod x86_fam {
		#[cfg(feature = "x86_fam_subhook")]
		pub mod subhook;
	}
	
	#[cfg(all(feature="riscv", any(target_arch = "riscv32", target_arch = "riscv64")))]
	pub mod riscv {
		#[cfg(feature = "riscv_subhook")]
		pub mod subhook;
	}
}

mod mem;
mod disasm;

pub mod common {
	pub mod scoped_remove;
	pub use scoped_remove::{Hookable, ScopedRemove};
}

// Reexport hook macro
pub use subhook_rs_macros::hook;
