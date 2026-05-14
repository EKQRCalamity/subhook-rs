use crate::mem;
use crate::disasm::x86_fam::{disasm, RelocHint};
pub use crate::error::HookError;
pub use crate::arch::x86_fam::subhook::shared::hookflags::HookFlags;
use crate::arch::x86_fam::subhook::shared::jmp;

use std::ptr;

const MAX_INSTRUCTIONN_LEN: usize = 15;
/// Extra bytes reserved in the trampoline buffer to absorb short-jump expansions
const EXPAND_BUDGET: usize = 64;

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
	trampoline_err: Option<HookError>,
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

		let tail_jmp_size = jmp::tail_jmp_size();
		let trampoline_size = jmp_size + MAX_INSTRUCTIONN_LEN + tail_jmp_size + EXPAND_BUDGET;
		let (trampoline, trampoline_err) = match unsafe { build_trampoline(src, jmp_size, trampoline_size) } {
			Ok(t) => (Some(t), None),
			Err(e) => (None, Some(e)),
		};

		Ok(Self { src, dst, flags, saved_bytes, jmp_size, trampoline, trampoline_err, installed: false })
	}

	/// Install the hook, overwriting the prologue of `src` with a jmp to `dst`.
	pub fn install(&mut self) -> Result<(), HookError> {
		if self.installed {
			return Err(HookError::AlreadyInstalled);
		}
		let mut patch = vec![0u8; self.jmp_size];
		unsafe {
			jmp::write_jmp(
				patch.as_mut_ptr(),
				self.src as usize,
				self.dst as usize,
				self.flags,
			)
		}?;
		unsafe { mem::patch_bytes(self.src, patch.as_ptr(), self.jmp_size) }?;
		self.installed = true;
		Ok(())
	}


	/// Remove the hook, restoring the original prologue bytes.
	pub fn remove(&mut self) -> Result<(), HookError> {
		if !self.installed {
			return Err(HookError::NotInstalled);
		}
		unsafe { mem::patch_bytes(self.src, self.saved_bytes.as_ptr(), self.jmp_size) }?;
		self.installed = false;
		Ok(())
	}

	/// Returns `true` if the hook is currently installed.
	pub fn is_installed(&self) -> bool {
		self.installed
	}

	/// Install the hook while suspending a set of threads for the duration, remapping any thread
	/// whose instruction pointer falls inside the overwritten prologue.
	///
	/// Requires the `thread_suspend` feature and a Windows target.
	/// Returns `HookError::NoTrampoline` if no trampoline was built during `Hook::new`.
	///
	/// # Safety
	/// All handles in `threads` must be valid, open thread handles with
	/// `THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT | THREAD_SET_CONTEXT` access rights.
	/// Do not pass the handle of the calling thread.
	#[cfg(all(windows, feature = "thread_suspend"))]
	pub unsafe fn install_with_threads(
		&mut self,
		threads: &[windows_sys::Win32::Foundation::HANDLE],
	) -> Result<(), HookError> {
		if self.installed {
			return Err(HookError::AlreadyInstalled);
		}
		if self.trampoline.is_none() {
			return Err(HookError::NoTrampoline);
		}

		for &h in threads {
			unsafe { crate::mem::suspend_thread(h) }?;
		}

		let mut patch = vec![0u8; self.jmp_size];
		unsafe {
			jmp::write_jmp(
				patch.as_mut_ptr(),
				self.src as usize,
				self.dst as usize,
				self.flags,
			)
		}?;

		let result = unsafe { mem::patch_bytes(self.src, patch.as_ptr(), self.jmp_size) };

		if result.is_ok() {
			self.installed = true;

			if let Some(ref tramp) = self.trampoline {
				let src = self.src as usize;
				let tramp_base = tramp.ptr as usize;
				let patch_end = src + self.jmp_size;

				for &h in threads {
					if let Ok(ip) = unsafe { crate::mem::get_thread_ip(h) } {
						if ip >= src && ip < patch_end {
							let _ = unsafe { crate::mem::set_thread_ip(h, tramp_base + (ip - src)) };
						}
					}
				}
			}
		}

		for &h in threads {
			let _ = unsafe { crate::mem::resume_thread(h) };
		}

		result
	}

	/// Remove the hook while suspending a set of threads for the duration, remapping any thread
	/// whose instruction pointer falls inside the trampoline's copied bytes.
	///
	/// Requires the `thread_suspend` feature and a Windows target.
	///
	/// # Safety
	/// All handles in `threads` must be valid, open thread handles with
	/// `THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT | THREAD_SET_CONTEXT` access rights.
	/// Do not pass the handle of the calling thread.
	#[cfg(all(windows, feature = "thread_suspend"))]
	pub unsafe fn remove_with_threads(
		&mut self,
		threads: &[windows_sys::Win32::Foundation::HANDLE],
	) -> Result<(), HookError> {
		if !self.installed {
			return Err(HookError::NotInstalled);
		}

		for &h in threads {
			unsafe { crate::mem::suspend_thread(h) }?;
		}

		unsafe { mem::patch_bytes(self.src, self.saved_bytes.as_ptr(), self.jmp_size) }?;
		self.installed = false;

		if let Some(ref tramp) = self.trampoline {
			let src = self.src as usize;
			let tramp_base = tramp.ptr as usize;
			let patch_end = tramp_base + self.jmp_size;

			for &h in threads {
				if let Ok(ip) = unsafe { crate::mem::get_thread_ip(h) } {
					if ip >= tramp_base && ip < patch_end {
						let _ = unsafe { crate::mem::set_thread_ip(h, src + (ip - tramp_base)) };
					}
				}
			}
		}

		for &h in threads {
			let _ = unsafe { crate::mem::resume_thread(h) };
		}

		Ok(())
	}

	/// Returns a pointer to the trampoline, or `None` if one could not be built.
	///
	/// Call through the trampoline to invoke the original function while the
	/// hook is installed.
	pub fn trampoline(&self) -> Option<*const u8> {
		self.trampoline.as_ref().map(|t| t.ptr as *const u8)
	}

	/// Returns the error from trampoline building, if it failed.
	/// Use this to diagnose `trampoline()` returning `None`.
	pub fn trampoline_error(&self) -> Option<&HookError> {
		self.trampoline_err.as_ref()
	}

	/// Returns the first `n` bytes of the hooked function's original prologue.
	/// Use this to provide context in case you want to open an issue for the disasm.
	pub fn prologue_bytes(&self, n: usize) -> Vec<u8> {
		unsafe { std::slice::from_raw_parts(self.src, n) }.to_vec()
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
	let buffer = unsafe { mem::alloc_code_near(src, trampoline_size) }?;

	let src_addr = src as usize;
	let buffer_addr = buffer as usize;
	let mut orig_size: usize = 0;
	let mut buf_offset: usize = 0;

	while orig_size < jmp_size {
		let remaining = unsafe {
			std::slice::from_raw_parts(
				(src_addr + orig_size) as *const u8,
				MAX_INSTRUCTIONN_LEN,
			)
		};

		let (instruction_len, reloc) = disasm(remaining).inspect_err(|_| {
			unsafe { mem::free_code(buffer, trampoline_size) };
		})?;

		match reloc {
			None => {
				unsafe {
					ptr::copy_nonoverlapping(
						(src_addr + orig_size) as *const u8,
						(buffer_addr + buf_offset) as *mut u8,
						instruction_len,
					)
				};
				buf_offset += instruction_len;
			}

			Some(RelocHint::I32(op_offset)) => {
				unsafe {
					ptr::copy_nonoverlapping(
						(src_addr + orig_size) as *const u8,
						(buffer_addr + buf_offset) as *mut u8,
						instruction_len,
					)
				};
				if op_offset > 0 {
					let pos_shift =
						(buffer_addr + buf_offset) as i64 - (src_addr + orig_size) as i64;
					if pos_shift < i32::MIN as i64 || pos_shift > i32::MAX as i64 {
						unsafe { mem::free_code(buffer, trampoline_size) };
						return Err(HookError::TrampolineRelocateOverflow(pos_shift));
					}
					let op_ptr = (buffer_addr + buf_offset + op_offset) as *mut i32;
					unsafe {
						let old_value = op_ptr.read_unaligned();
						op_ptr.write_unaligned(old_value - pos_shift as i32);
					}
				}
				buf_offset += instruction_len;
			}

			Some(RelocHint::ShortJcc(opcode)) => {
				let disp8 = remaining[1] as i8 as i32;
				let abs_target =
					(src_addr as isize + orig_size as isize + 2 + disp8 as isize) as usize;
				let new_disp = abs_target as i64 - (buffer_addr + buf_offset + 6) as i64;
				if new_disp < i32::MIN as i64 || new_disp > i32::MAX as i64 {
					unsafe { mem::free_code(buffer, trampoline_size) };
					return Err(HookError::TrampolineRelocateOverflow(new_disp));
				}
				unsafe {
					let out = (buffer_addr + buf_offset) as *mut u8;
					out.write(0x0F);
					out.add(1).write(0x80 | (opcode & 0x0F));
					(out.add(2) as *mut i32).write_unaligned(new_disp as i32);
				}
				buf_offset += 6;
			}

			Some(RelocHint::ShortJmp) => {
				let disp8 = remaining[1] as i8 as i32;
				let abs_target =
					(src_addr as isize + orig_size as isize + 2 + disp8 as isize) as usize;
				let new_disp = abs_target as i64 - (buffer_addr + buf_offset + 5) as i64;
				if new_disp < i32::MIN as i64 || new_disp > i32::MAX as i64 {
					unsafe { mem::free_code(buffer, trampoline_size) };
					return Err(HookError::TrampolineRelocateOverflow(new_disp));
				}
				unsafe {
					let out = (buffer_addr + buf_offset) as *mut u8;
					out.write(0xE9);
					(out.add(1) as *mut i32).write_unaligned(new_disp as i32);
				}
				buf_offset += 5;
			}
		}

		orig_size += instruction_len;
	}

	let tail_src = buffer_addr + buf_offset;
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
