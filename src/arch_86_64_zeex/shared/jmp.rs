use crate::{error::HookError, arch_86_64_zeex::shared::hookflags::HookFlags};

const JMP_OPCODE:  u8 = 0xE9;
const PUSH_OPCODE: u8 = 0x68;
const MOV_OPCODE:  u8 = 0xC7;
const RET_OPCODE:  u8 = 0xC3;

const MOV_MODRM:   u8 = 0x44;
const MOV_SIB:     u8 = 0x24;
const MOV_OFFSET:  u8 = 0x04;

/// Returns the number of bytes the jmp thunk for `flags` occupies.
pub(crate) fn jmp_size(flags: HookFlags) -> usize {
    #[cfg(target_arch = "x86_64")]
    if flags.contains(HookFlags::USE_64BIT_OFFSET) {
        return JMP64_SIZE;
    }
    let _ = flags;
    JMP32_SIZE
}

pub(crate) const JMP32_SIZE: usize = 5;
pub(crate) const JMP64_SIZE: usize = 14;

/// Size of the tail jmp used in trampolines.
/// Always the absolute 64-bit thunk on x86-64 so the trampoline can live anywhere in the address
/// space.
pub(crate) fn tail_jmp_size() -> usize {
    #[cfg(target_arch = "x86_64")]
    return JMP64_SIZE;
    #[cfg(not(target_arch = "x86_64"))]
    JMP32_SIZE
}

/// Flags to use when writing the tail jmp in a trampoline.
pub(crate) fn tail_flags() -> HookFlags {
    #[cfg(target_arch = "x86_64")]
    return HookFlags::USE_64BIT_OFFSET;
    #[cfg(not(target_arch = "x86_64"))]
    HookFlags::NONE
}

/// Write a jmp thunk at `buf` redirecting execution to `dst_addr`.
///
/// # Safety
/// `buf` must be writable for at least `jmp_size(flags)` bytes.
pub(crate) unsafe fn write_jmp(
    buf: *mut u8,
    src_addr: usize,
    dst_addr: usize,
    flags: HookFlags,
) -> Result<(), HookError> {
    #[cfg(target_arch = "x86_64")]
    if flags.contains(HookFlags::USE_64BIT_OFFSET) {
        return unsafe { write_jmp64(buf, dst_addr) };
    }
    let _ = flags;
    unsafe { write_jmp32(buf, src_addr, dst_addr) }
}

unsafe fn write_jmp32(buf: *mut u8, src: usize, dst: usize) -> Result<(), HookError> {
    let offset = (dst as i64) - (src as i64 + JMP32_SIZE as i64);
    if offset < i32::MIN as i64 || offset > i32::MAX as i64 {
        return Err(HookError::Overflow(offset));
    }
    unsafe {
        buf.write(JMP_OPCODE);
        buf.add(1).cast::<i32>().write_unaligned(offset as i32);
    }
    Ok(())
}

#[cfg(target_arch = "x86_64")]
unsafe fn write_jmp64(buf: *mut u8, dst: usize) -> Result<(), HookError> {
    let lo = dst as u32;
    let hi = (dst >> 32) as u32;
    unsafe {
        buf.add(0).write(PUSH_OPCODE);
        buf.add(1).cast::<u32>().write_unaligned(lo);
        buf.add(5).write(MOV_OPCODE);
        buf.add(6).write(MOV_MODRM);
        buf.add(7).write(MOV_SIB);
        buf.add(8).write(MOV_OFFSET);
        buf.add(9).cast::<u32>().write_unaligned(hi);
        buf.add(13).write(RET_OPCODE);
    }
    Ok(())
}
