pub mod error;
pub mod arch {
	#[cfg(all(feature = "x86_fam", any(target_arch = "x86_64", target_arch = "x86")))]
	pub mod x86_fam {
		#[cfg(feature = "x86_64_Zeex")]
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
