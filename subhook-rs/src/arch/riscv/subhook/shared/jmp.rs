use crate::{error::HookError, arch::riscv::subhook::shared::hookflags::HookFlags};

pub(crate) fn jmp_size(_flags: HookFlags) -> usize {
    JMP_SIZE
}

pub(crate) const JMP_SIZE: usize = 8;

pub(crate) fn tail_jmp_size() -> usize {
    JMP_SIZE
}

pub(crate) fn tail_flags() -> HookFlags {
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
    _flags: HookFlags,
) -> Result<(), HookError> {
    let offset = (dst_addr as i64) - (src_addr as i64);
    
    if offset >= -(1 << 20) && offset < (1 << 20) {
        return unsafe { write_jal(buf, offset as i32) };
    }
    
    unsafe { write_auipc_jalr(buf, dst_addr) }
}

unsafe fn write_jal(buf: *mut u8, offset: i32) -> Result<(), HookError> {
    let imm_20 = ((offset >> 20) & 1) as u32;
    let imm_10_1 = ((offset >> 1) & 0x3FF) as u32;
    let imm_11 = ((offset >> 11) & 1) as u32;
    let imm_19_12 = ((offset >> 12) & 0xFF) as u32;
    
    let insn: u32 = 0b1101111
        | (imm_20 << 31)
        | (imm_10_1 << 21)
        | (imm_11 << 20)
        | (imm_19_12 << 12);
    
    unsafe {
        buf.cast::<u32>().write_unaligned(insn);
        buf.add(4).cast::<u32>().write_unaligned(0x00000013);
    }
    Ok(())
}

unsafe fn write_auipc_jalr(buf: *mut u8, dst: usize) -> Result<(), HookError> {
    let hi = ((dst + 0x800) >> 12) & 0xFFFFF;
    let lo = dst & 0xFFF;
    
    let auipc: u32 = 0b0010111 | ((hi as u32) << 12);
    let jalr: u32 = 0b1100111 | (1 << 7) | ((lo as u32) << 20);
    
    unsafe {
        buf.cast::<u32>().write_unaligned(auipc);
        buf.add(4).cast::<u32>().write_unaligned(jalr);
    }
    Ok(())
}
