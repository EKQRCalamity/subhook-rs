use crate::mem;
use crate::disasm::x86_fam::disasm;
pub use crate::error::HookError;
pub use crate::arch::x86_fam::subhook::shared::hookflags::HookFlags;
use crate::arch::x86_fam::subhook::shared::jmp;

use std::ptr;

const MAX_INSTRUCTIONN_LEN: usize = 15;

struct Trampoline {
	ptr: *mut u8,
	size: usize,
}

pub struct Hook {
	src: *mut u8,
	dst: *const u8,
	flags: HookFlags,
	saved_bytes: Vec<u8>,
	jmp_size: usize,
	trampoline: Option<Trampoline>,
	installed: bool,
}

unsafe impl Send for Hook {}

impl Hook {
	pub unsafe fn new(
		src: *mut u8,
		dst: *const u8,
		flags: HookFlags,
	) -> Result<Self, HookError> {
		if src.is_null() {
			return Err(HookError::NullPointer(crate::error::NullPointerSource::Src));
		}
		if dst.is_null() {
			return Err(HookError::NullPointer(crate::error::NullPointerSource::Dst));
		}

		let jmp_size = jmp::jmp_size(flags);
		
		// UNSAFE: This needs to be enforced by the caller!
		let saved_bytes = unsafe { std::slice::from_raw_parts(src, jmp_size) }.to_vec();

		unsafe { mem::unprotect(src, jmp_size ) }?;

		let tail_jmp_size = jmp::tail_jmp_size();
		let trampoline_size = jmp_size + MAX_INSTRUCTIONN_LEN + tail_jmp_size;
		let trampoline = unsafe { build_trampoline(src, jmp_size, trampoline_size) }.ok();

		Ok(Self { src, dst, flags, saved_bytes, jmp_size, trampoline, installed: false })
	}

	/// Install the hook, overwriting the prologue of `src` with a jmp to `dst`.
	pub fn install(&mut self) -> Result<(), HookError> {
		if self.installed {
			return Err(HookError::AlreadyInstalled);
		}
		unsafe {
			jmp::write_jmp(self.src, self.src as usize, self.dst as usize, self.flags)
		}?;
		self.installed = true;
		Ok(())
	}


	/// Remove the hook, restoring the original prologue bytes.
	pub fn remove(&mut self) -> Result<(), HookError> {
		if !self.installed {
			return Err(HookError::NotInstalled);
		}
		unsafe {
			ptr::copy_nonoverlapping(
				self.saved_bytes.as_ptr(),
				self.src,
				self.jmp_size,
			)
		};
		self.installed = false;
		Ok(())
	}

	/// Returns `true` if the hook is currently installed.
	pub fn is_installed(&self) -> bool {
		self.installed
	}

	/// Returns a pointer to the trampoline, or `None` if one could not be built.
	///
	/// Call through the trampoline to invoke the original function while the
	/// hook is installed.
	pub fn trampoline(&self) -> Option<*const u8> {
		self.trampoline.as_ref().map(|t| t.ptr as *const u8)
	}

	pub fn src(&self) -> *mut u8 { self.src }
	pub fn dst(&self) -> *const u8 { self.dst }
}

impl Drop for Hook {
	fn drop(&mut self) {
	  if self.installed {
			let _ = self.remove();
		}
		if let Some(ref trampoline) = self.trampoline {
			unsafe { mem::free_code(trampoline.ptr, trampoline.size) };
		}
	}
}

unsafe fn build_trampoline(src: *const u8, jmp_size: usize, trampoline_size: usize) -> Result<Trampoline, HookError> {
	let buffer = unsafe { mem::alloc_code(trampoline_size) }?;

	let src_addr = src as usize;
	let buffer_addr = buffer as usize;
	let mut orig_size: usize = 0;

	while orig_size < jmp_size {
		let remaining = unsafe {
			std::slice::from_raw_parts(
				(src_addr + orig_size) as *const u8,
				MAX_INSTRUCTIONN_LEN,
			)
		};

		let (instruction_len, relocation_opcode_offset) = disasm(remaining).inspect_err(|_| {
			unsafe { mem::free_code(buffer, trampoline_size) };
		})?;

		unsafe {
			ptr::copy_nonoverlapping(
				(src_addr + orig_size) as *const u8, (buffer_addr + orig_size) as *mut u8, instruction_len
			)
		};

		if let Some(op_offset) = relocation_opcode_offset {
			if op_offset > 0 {
				let offset = buffer_addr as i64 - src_addr as i64;

				if offset < i32::MIN as i64 || offset > i32::MAX as i64 {
					unsafe { mem::free_code(buffer, trampoline_size) };
					return Err(HookError::TrampolineRelocateOverflow(offset));
				}

				let op_ptr = (buffer_addr + orig_size + op_offset) as *mut i32;
				unsafe {
					let old_value = op_ptr.read_unaligned();
					op_ptr.write_unaligned(old_value - offset as i32);
				}
			}
		}
		orig_size += instruction_len;
	}

	let tail_src = buffer_addr + orig_size;
	let tail_dst = src_addr + orig_size;
	let tail_flags = jmp::tail_flags();
	if let Err(e) = unsafe { jmp::write_jmp(tail_src as *mut u8, tail_src, tail_dst, tail_flags) } {
		unsafe { mem::free_code(buffer, trampoline_size) };
		return Err(e);
	}

	Ok(Trampoline { ptr: buffer, size: trampoline_size })
}

impl crate::common::scoped_remove::Hookable for Hook {
	fn install(&mut self) -> Result<(), HookError> {
		self.install()
	}

	fn remove(&mut self) -> Result<(), HookError> {
		self.remove()
	}
}
