use crate::mem;
use crate::disasm::riscv::disasm;
pub use crate::error::HookError;
pub use crate::arch::riscv::subhook::shared::hookflags::HookFlags;
use crate::arch::riscv::subhook::shared::jmp;

use std::ptr;

const MAX_INSTRUCTION_LEN: usize = 4;

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
		
		let saved_bytes = unsafe { std::slice::from_raw_parts(src, jmp_size) }.to_vec();

		let tail_jmp_size = jmp::tail_jmp_size();
		let trampoline_size = jmp_size + MAX_INSTRUCTION_LEN + tail_jmp_size;
		let trampoline = unsafe { build_trampoline(src, jmp_size, trampoline_size) }.ok();

		Ok(Self { src, dst, flags, saved_bytes, jmp_size, trampoline, installed: false })
	}

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

	pub fn remove(&mut self) -> Result<(), HookError> {
		if !self.installed {
			return Err(HookError::NotInstalled);
		}
		unsafe { mem::patch_bytes(self.src, self.saved_bytes.as_ptr(), self.jmp_size) }?;
		self.installed = false;
		Ok(())
	}

	pub fn is_installed(&self) -> bool {
		self.installed
	}

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
				MAX_INSTRUCTION_LEN,
			)
		};

		let (instruction_len, relocation_offset) = disasm(remaining).inspect_err(|_| {
			unsafe { mem::free_code(buffer, trampoline_size) };
		})?;

		unsafe {
			ptr::copy_nonoverlapping(
				(src_addr + orig_size) as *const u8, (buffer_addr + orig_size) as *mut u8, instruction_len
			)
		};

		if let Some(_offset) = relocation_offset {
			let pc_offset = buffer_addr as i64 - src_addr as i64;

			let insn_ptr = (buffer_addr + orig_size) as *mut u32;
			let mut insn = unsafe { insn_ptr.read_unaligned() };
			
			let opcode = insn & 0x7F;
			
			match opcode {
				0b0010111 => {
					let old_imm = (insn >> 12) as i32;
					let new_imm = old_imm.wrapping_sub(pc_offset as i32 >> 12);
					insn = (insn & 0xFFF) | ((new_imm as u32) << 12);
					unsafe { insn_ptr.write_unaligned(insn) };
				}
				0b1101111 => {
					let imm_20 = ((insn >> 31) & 1) as i32;
					let imm_10_1 = ((insn >> 21) & 0x3FF) as i32;
					let imm_11 = ((insn >> 20) & 1) as i32;
					let imm_19_12 = ((insn >> 12) & 0xFF) as i32;
					
					let old_offset = (imm_20 << 20) | (imm_19_12 << 12) | (imm_11 << 11) | (imm_10_1 << 1);
					let old_offset = if old_offset & (1 << 20) != 0 { old_offset | !0xFFFFF } else { old_offset };
					
					let new_offset = old_offset.wrapping_sub(pc_offset as i32);
					
					let imm_20 = ((new_offset >> 20) & 1) as u32;
					let imm_10_1 = ((new_offset >> 1) & 0x3FF) as u32;
					let imm_11 = ((new_offset >> 11) & 1) as u32;
					let imm_19_12 = ((new_offset >> 12) & 0xFF) as u32;
					
					insn = 0b1101111
						| (imm_20 << 31)
						| (imm_10_1 << 21)
						| (imm_11 << 20)
						| (imm_19_12 << 12);
					
					unsafe { insn_ptr.write_unaligned(insn) };
				}
				0b1100011 => {
					let imm_12 = ((insn >> 31) & 1) as i32;
					let imm_10_5 = ((insn >> 25) & 0x3F) as i32;
					let imm_4_1 = ((insn >> 8) & 0xF) as i32;
					let imm_11 = ((insn >> 7) & 1) as i32;
					
					let old_offset = (imm_12 << 12) | (imm_11 << 11) | (imm_10_5 << 5) | (imm_4_1 << 1);
					let old_offset = if old_offset & (1 << 12) != 0 { old_offset | !0x1FFF } else { old_offset };
					
					let new_offset = old_offset.wrapping_sub(pc_offset as i32);
					
					let imm_12 = ((new_offset >> 12) & 1) as u32;
					let imm_10_5 = ((new_offset >> 5) & 0x3F) as u32;
					let imm_4_1 = ((new_offset >> 1) & 0xF) as u32;
					let imm_11 = ((new_offset >> 11) & 1) as u32;
					
					insn = (insn & 0x1FFF07F)
						| (imm_12 << 31)
						| (imm_10_5 << 25)
						| (imm_4_1 << 8)
						| (imm_11 << 7);
					
					unsafe { insn_ptr.write_unaligned(insn) };
				}
				_ => {}
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
