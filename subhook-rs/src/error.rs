use std::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub enum NullPointerSource {
	Src,
	Dst
}

#[derive(Debug, PartialEq, Eq)]
pub enum HookError {
	/// The src or dst was null.
	NullPointer(NullPointerSource),
	/// Tried to install a hook that was already installed previously.
	AlreadyInstalled,
	/// Tried to remove a hook that was not installed yet.
	NotInstalled,
	/// Executeable memory allocation failed.
	/// Carries the OS error code.
	AllocationFailed(i32),
	/// Tried to remove a hook that is not installed.
	/// Carries the OS error code.
	UnprotectFailed(i32),
	/// Usually `src` and `dst` being too far apart and 'USE_64BIT_OFFSET' is not set.
	/// Carries the distance in bytes.
	Overflow(i64),
	/// Dissassambler caught an unrecognized instruction.
	/// Carries the unrecognized opcode byte.
	UnknownInstruction(u8),
	/// Trampoline is too far from the original code and could not be relocated.
	/// Carries the distance in bytes.
	TrampolineRelocateOverflow(i64),
	/// The input slice was truncated and full decoding of instruction was not possible.
	TruncatedInstruction,
	/// Trampoline construction failed, but is required
	NoTrampoline,
	/// A Windows thread operation failed.
	/// Carries the OS error code.
	#[cfg(all(windows, feature = "thread_suspend"))]
	ThreadOperationFailed(u32),
}

impl Display for NullPointerSource {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	  match self {
			Self::Src => write!(f, "src"),
			Self::Dst => write!(f, "dst")
		}
	}
}

impl Display for HookError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::NullPointer(source) => write!(f, "The '{}' pointer is null", source),
			Self::AlreadyInstalled => write!(f, "Hook is already installed"),
			Self::NotInstalled => write!(f, "Tried to remove a non installed hook"),
			Self::AllocationFailed(code) => write!(f, "Failed to allocate executable memory (OS Error: {})", code),
			Self::UnprotectFailed(code) => write!(f, "Memory unprotect failed (OS Error: {})", code),
			Self::Overflow(dist) => write!(f, "Src/Dst distance ({} bytes) exceeds 32 bit integer range, use USE_64BIT_OFFSET", dist),
			Self::UnknownInstruction(opcode) => write!(f, "Dissassambler encountered unrecognized opcode 0x{opcode:02X}"),
			Self::TrampolineRelocateOverflow(dist) => write!(f, "Trampoline distance ({} bytes) too large to relocate rel32 operand.", dist),
			Self::TruncatedInstruction => write!(f, "Instruction data ended unexpectedly, could not fully decode instruction."),
			Self::NoTrampoline => write!(f, "No trampoline available; thread-safe install/remove requires a trampoline to remap thread IPs"),
			#[cfg(all(windows, feature = "thread_suspend"))]
			Self::ThreadOperationFailed(code) => write!(f, "Thread operation failed (OS Error: {})", code),
		}
	}
}

impl std::error::Error for HookError {}
