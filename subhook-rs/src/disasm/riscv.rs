use crate::error::HookError;

const OPCODE_AUIPC: u8 = 0b0010111;
const OPCODE_JAL: u8 = 0b1101111;
const OPCODE_BRANCH: u8 = 0b1100011;

const QUADRANT_C1: u8 = 0b01;
const QUADRANT_C2: u8 = 0b10;

/// Decode one instruction from `code`
///
/// Returns `(len, relocation_opcode_offset)` where relocation_opcode_offset is `Some(0)` when
/// the instruction is PC-relative and needs relocation.
pub fn disasm(code: &[u8]) -> Result<(usize, Option<usize>), HookError> {
	if code.len() < 2 {
		return Err(HookError::TruncatedInstruction);
	}

	let low16 = u16::from_le_bytes([code[0], code[1]]);
	let quadrant = (low16 & 0b11) as u8;

	// Compressed instructions have bits [1:0] != 0b11
	if quadrant != 0b11 {
		return disasm_compressed(low16);
	}

	if code.len() < 4 {
		return Err(HookError::TruncatedInstruction);
	}

	let insn = u32::from_le_bytes([code[0], code[1], code[2], code[3]]);
	disasm_standard(insn)
}

fn disasm_compressed(insn: u16) -> Result<(usize, Option<usize>), HookError> {
	let quadrant = (insn & 0b11) as u8;
	let funct3 = ((insn >> 13) & 0b111) as u8;

	let needs_relocation = match quadrant {
		QUADRANT_C1 => {
			// C.J, C.JAL, C.BEQZ, C.BNEZ
			matches!(funct3, 0b001 | 0b101 | 0b110 | 0b111)
		}
		QUADRANT_C2 => {
			// C.JR and C.JALR are indirect, not PC-relative
			false
		}
		_ => false,
	};

	let relocation_offset = if needs_relocation { Some(0) } else { None };
	Ok((2, relocation_offset))
}

fn disasm_standard(insn: u32) -> Result<(usize, Option<usize>), HookError> {
	let opcode = (insn & 0x7F) as u8;

	let needs_relocation = match opcode {
		OPCODE_AUIPC | OPCODE_JAL | OPCODE_BRANCH => true,
		_ => false,
	};

	let relocation_offset = if needs_relocation { Some(0) } else { None };
	Ok((4, relocation_offset))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn decode_compressed_nop() {
		let (len, reloc) = disasm(&[0x01, 0x00]).unwrap();
		assert_eq!(len, 2);
		assert_eq!(reloc, None);
	}

	#[test]
	fn decode_compressed_jump() {
		let insn: u16 = 0b101_00000000000_01;
		let bytes = insn.to_le_bytes();
		let (len, reloc) = disasm(&bytes).unwrap();
		assert_eq!(len, 2);
		assert_eq!(reloc, Some(0));
	}

	#[test]
	fn decode_compressed_beqz() {
		let insn: u16 = 0b110_000_000_00000_01;
		let bytes = insn.to_le_bytes();
		let (len, reloc) = disasm(&bytes).unwrap();
		assert_eq!(len, 2);
		assert_eq!(reloc, Some(0));
	}

	#[test]
	fn decode_addi() {
		let (len, reloc) = disasm(&[0x93, 0x00, 0x10, 0x00]).unwrap();
		assert_eq!(len, 4);
		assert_eq!(reloc, None);
	}

	#[test]
	fn decode_auipc() {
		let (len, reloc) = disasm(&[0x97, 0x50, 0x34, 0x12]).unwrap();
		assert_eq!(len, 4);
		assert_eq!(reloc, Some(0));
	}

	#[test]
	fn decode_jal() {
		let insn: u32 = 0b0_0000000000_0_00000000_00001_1101111;
		let bytes = insn.to_le_bytes();
		let (len, reloc) = disasm(&bytes).unwrap();
		assert_eq!(len, 4);
		assert_eq!(reloc, Some(0));
	}

	#[test]
	fn decode_beq() {
		let insn: u32 = 0b0000000_00010_00001_000_00000_1100011;
		let bytes = insn.to_le_bytes();
		let (len, reloc) = disasm(&bytes).unwrap();
		assert_eq!(len, 4);
		assert_eq!(reloc, Some(0));
	}

	#[test]
	fn decode_load() {
		let insn: u32 = 0b000000000000_00010_010_00001_0000011;
		let bytes = insn.to_le_bytes();
		let (len, reloc) = disasm(&bytes).unwrap();
		assert_eq!(len, 4);
		assert_eq!(reloc, None);
	}

	#[test]
	fn truncated_instruction() {
		assert!(disasm(&[0x93]).is_err());
	}

	#[test]
	fn truncated_standard_instruction() {
		assert!(disasm(&[0x93, 0x00, 0x10]).is_err());
	}
}
