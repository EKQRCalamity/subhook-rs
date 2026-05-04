use crate::error::HookError;

/// The instruction has a ModRM byte.
const FLAG_MODRM: u8 = 1 << 0;
/// Low 3 bits encode a register.
const FLAG_PLUS_REGISTER: u8 = 1 << 1;
/// Register field of ModRM is an opcode extension
const FLAG_REGISTER_OPCODE: u8 = 1 << 2;
/// 1-Byte Immediate is following
const FLAG_IMMEDIATE_8BIT: u8 = 1 << 3;
/// 2-Byte immediate is following
const FLAG_IMMEDIATE_16BIT: u8 = 1 << 4;
/// 4-Byte immediate (or REX.W) is following
const FLAG_IMMEDIATE_32BIT: u8 = 1 << 5;
/// Immediate is a relative address and needs relocation
const FLAG_RELOCATION: u8 = 1 << 6;

struct OpCodeInformation {
	code: u8,
	reg_opcode: u8,
	flags: u8,
}

#[rustfmt::skip]
/// Ordering is in SDM/mnemonic style
static OPCODE: &[OpCodeInformation] = &[
	// ADD
	OpCodeInformation { code: 0x04, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x05, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT },
	OpCodeInformation { code: 0x80, reg_opcode: 0, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x81, reg_opcode: 0, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_32BIT },
	OpCodeInformation { code: 0x83, reg_opcode: 0, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x00, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x01, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x02, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x03, reg_opcode: 0, flags: FLAG_MODRM },
	// AND
	OpCodeInformation { code: 0x24, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x25, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT },
	OpCodeInformation { code: 0x80, reg_opcode: 4, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x81, reg_opcode: 4, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_32BIT },
	OpCodeInformation { code: 0x83, reg_opcode: 4, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x20, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x21, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x22, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x23, reg_opcode: 0, flags: FLAG_MODRM },
	// CALL
	OpCodeInformation { code: 0xE8, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT | FLAG_RELOCATION },
	OpCodeInformation { code: 0xFF, reg_opcode: 2, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE },
	// CMP
	OpCodeInformation { code: 0x83, reg_opcode: 7, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x39, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x3D, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT },
	// DEC
	OpCodeInformation { code: 0xFF, reg_opcode: 1, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE },
	OpCodeInformation { code: 0x48, reg_opcode: 0, flags: FLAG_PLUS_REGISTER },
	// ENTER
	OpCodeInformation { code: 0xC8, reg_opcode: 0, flags: FLAG_IMMEDIATE_16BIT | FLAG_IMMEDIATE_8BIT },
	// FLD
	OpCodeInformation { code: 0xD9, reg_opcode: 0, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE },
	OpCodeInformation { code: 0xDD, reg_opcode: 0, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE },
	OpCodeInformation { code: 0xDB, reg_opcode: 5, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE },
	// INT3
	OpCodeInformation { code: 0xCC, reg_opcode: 0, flags: 0 },
	// JMP
	OpCodeInformation { code: 0xE9, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT | FLAG_RELOCATION },
	OpCodeInformation { code: 0xFF, reg_opcode: 4, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE },
	// LEA
	OpCodeInformation { code: 0x8D, reg_opcode: 0, flags: FLAG_MODRM },
	// LEAVE
	OpCodeInformation { code: 0xC9, reg_opcode: 0, flags: 0 },
	// MOV
	OpCodeInformation { code: 0x88, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x89, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x8A, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x8B, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x8C, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x8E, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0xA0, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0xA1, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT },
	OpCodeInformation { code: 0xA2, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0xA3, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT },
	OpCodeInformation { code: 0xB0, reg_opcode: 0, flags: FLAG_PLUS_REGISTER | FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0xB8, reg_opcode: 0, flags: FLAG_PLUS_REGISTER | FLAG_IMMEDIATE_32BIT },
	OpCodeInformation { code: 0xC6, reg_opcode: 0, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0xC7, reg_opcode: 0, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_32BIT },
	// NOP
	OpCodeInformation { code: 0x90, reg_opcode: 0, flags: 0 },
	// OR
	OpCodeInformation { code: 0x0C, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x0D, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT },
	OpCodeInformation { code: 0x80, reg_opcode: 1, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x81, reg_opcode: 1, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_32BIT },
	OpCodeInformation { code: 0x83, reg_opcode: 1, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x08, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x09, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x0A, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x0B, reg_opcode: 0, flags: FLAG_MODRM },
	// POP
	OpCodeInformation { code: 0x8F, reg_opcode: 0, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE },
	OpCodeInformation { code: 0x58, reg_opcode: 0, flags: FLAG_PLUS_REGISTER },
	// PUSH
	OpCodeInformation { code: 0xFF, reg_opcode: 6, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE },
	OpCodeInformation { code: 0x50, reg_opcode: 0, flags: FLAG_PLUS_REGISTER },
	OpCodeInformation { code: 0x6A, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x68, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT },
	// RET
	OpCodeInformation { code: 0xC3, reg_opcode: 0, flags: 0 },
	OpCodeInformation { code: 0xC2, reg_opcode: 0, flags: FLAG_IMMEDIATE_16BIT },
	// SUB
	OpCodeInformation { code: 0x2C, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x2D, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT },
	OpCodeInformation { code: 0x80, reg_opcode: 5, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x81, reg_opcode: 5, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_32BIT },
	OpCodeInformation { code: 0x83, reg_opcode: 5, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x28, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x29, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x2A, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x2B, reg_opcode: 0, flags: FLAG_MODRM },
	// TEST
	OpCodeInformation { code: 0xA8, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0xA9, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT },
	OpCodeInformation { code: 0xF6, reg_opcode: 0, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0xF7, reg_opcode: 0, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_32BIT },
	OpCodeInformation { code: 0x84, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x85, reg_opcode: 0, flags: FLAG_MODRM },
	// XOR
	OpCodeInformation { code: 0x34, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x35, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT },
	OpCodeInformation { code: 0x80, reg_opcode: 6, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x81, reg_opcode: 6, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_32BIT },
	OpCodeInformation { code: 0x83, reg_opcode: 6, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x30, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x31, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x32, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x33, reg_opcode: 0, flags: FLAG_MODRM },
];

#[rustfmt::skip]
static PREFIXES: &[u8] = &[
	// LOCK, REPNE and REP
	0xF0, 0xF2, 0xF3,
  // Segment overrides
	0x2E, 0x36, 0x3E, 0x26, 0x64, 0x65,
	// Operand sized override
	0x66,
	// address sized override
	0x67
];

#[rustfmt::skip]
static OPCODES_TWO_BYTE: &[OpCodeInformation] = &[
    // MOVUPS xmm, xmm/mem  /  MOVUPS xmm/mem, xmm
    OpCodeInformation { code: 0x10, reg_opcode: 0, flags: FLAG_MODRM },
    OpCodeInformation { code: 0x11, reg_opcode: 0, flags: FLAG_MODRM },
    // Multi-byte NOP  (0F 1F /0)
    OpCodeInformation { code: 0x1F, reg_opcode: 0, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE },
    // MOVAPS xmm, xmm/mem  /  MOVAPS xmm/mem, xmm
    OpCodeInformation { code: 0x28, reg_opcode: 0, flags: FLAG_MODRM },
    OpCodeInformation { code: 0x29, reg_opcode: 0, flags: FLAG_MODRM },
    // CMOVcc r, r/m  (40–4F)
    OpCodeInformation { code: 0x40, reg_opcode: 0, flags: FLAG_MODRM },
    OpCodeInformation { code: 0x41, reg_opcode: 0, flags: FLAG_MODRM },
    OpCodeInformation { code: 0x42, reg_opcode: 0, flags: FLAG_MODRM },
    OpCodeInformation { code: 0x43, reg_opcode: 0, flags: FLAG_MODRM },
    OpCodeInformation { code: 0x44, reg_opcode: 0, flags: FLAG_MODRM },
    OpCodeInformation { code: 0x45, reg_opcode: 0, flags: FLAG_MODRM },
    OpCodeInformation { code: 0x46, reg_opcode: 0, flags: FLAG_MODRM },
    OpCodeInformation { code: 0x47, reg_opcode: 0, flags: FLAG_MODRM },
    OpCodeInformation { code: 0x48, reg_opcode: 0, flags: FLAG_MODRM },
    OpCodeInformation { code: 0x49, reg_opcode: 0, flags: FLAG_MODRM },
    OpCodeInformation { code: 0x4A, reg_opcode: 0, flags: FLAG_MODRM },
    OpCodeInformation { code: 0x4B, reg_opcode: 0, flags: FLAG_MODRM },
    OpCodeInformation { code: 0x4C, reg_opcode: 0, flags: FLAG_MODRM },
    OpCodeInformation { code: 0x4D, reg_opcode: 0, flags: FLAG_MODRM },
    OpCodeInformation { code: 0x4E, reg_opcode: 0, flags: FLAG_MODRM },
    OpCodeInformation { code: 0x4F, reg_opcode: 0, flags: FLAG_MODRM },
    // MOVZX r, r/m8  /  MOVZX r, r/m16
    OpCodeInformation { code: 0xB6, reg_opcode: 0, flags: FLAG_MODRM },
    OpCodeInformation { code: 0xB7, reg_opcode: 0, flags: FLAG_MODRM },
    // MOVSX r, r/m8  /  MOVSX r, r/m16
    OpCodeInformation { code: 0xBE, reg_opcode: 0, flags: FLAG_MODRM },
    OpCodeInformation { code: 0xBF, reg_opcode: 0, flags: FLAG_MODRM },
];

/// Decode one instruction from `code`
///
/// Returns `(len, relocation_opcode_offset)` where relocation_opcode_offset is `Some(usize)` when
/// byte `n` of the instruction holds the start of a rel32 operand where relocation is necessary.
pub fn disasm(code: &[u8]) -> Result<(usize, Option<usize>), HookError> {
	let mut pos: usize = 0;
	let mut operand_size: usize = 4;

	// Handle endbr64 and endbr32 (F3 0F 1E FA and F3 0F 1E FB respectively) 
	if code.starts_with(&[0xF3, 0x0F, 0x1E, 0xFA]) || code.starts_with(&[0xF3, 0x0F, 0x1E, 0xFB]) {
		return Ok((4, None));
	}

	for &prefix in PREFIXES {
		if code.get(pos).copied() == Some(prefix) {
			if prefix == 0x66 {
				operand_size = 2;
			}
			pos += 1;
		}
	}

	#[cfg(target_arch = "x86_64")]
	if let Some(&rex) = code.get(pos) {
		if (rex & 0xF0) == 0x40 {
			pos += 1;
			// REX.W has a 64bit operand
			if rex & 0x08 != 0 {
				operand_size = 8;
			}
		}
	}

	let raw_opcode = *code.get(pos).ok_or(HookError::TruncatedInstruction)?;

	let mut relocation_opcode_offset: Option<usize> = None;

	// Handle two byte escape
	if raw_opcode == 0x0F {
		pos += 1;
		let opcode2 = *code.get(pos).ok_or(HookError::TruncatedInstruction)?;
		let entry = OPCODES_TWO_BYTE
			.iter()
			.find(|e| e.code == opcode2)
			.ok_or(HookError::UnknownInstruction(opcode2))?;

		pos += 1;

		if entry.flags & FLAG_MODRM != 0 {
			let modrm = *code.get(pos).ok_or(HookError::TruncatedInstruction)?;

			pos += 1;

			let addr_mode = modrm >> 6;
			let rm_field = modrm & 0x07;

			if addr_mode != 3 && rm_field == 4 {
				let scale_index_base = *code.get(pos).ok_or(HookError::TruncatedInstruction)?;
				pos += 1;
				if (scale_index_base & 0x07) == 5 { 
					pos += match addr_mode {
						1 => 1,
						_ => 4,
					};
				}
			}

			#[cfg(target_arch = "x86_64")]
			if addr_mode == 0 && rm_field == 5 { relocation_opcode_offset = Some(pos); }
			if addr_mode == 1 {
				pos += 1; 
			}
			if addr_mode == 2 || (addr_mode == 0 && rm_field == 5) {
				pos += 4;
			}
		}
		return Ok((pos, relocation_opcode_offset));
	}

	let mut matched: Option<&OpCodeInformation> = None;

	for entry in OPCODE {
		let found = if entry.flags & FLAG_PLUS_REGISTER != 0 {
			(raw_opcode & 0xF8) == entry.code
		} else if entry.flags & FLAG_REGISTER_OPCODE != 0 {
			raw_opcode == entry.code
				&& code.get(pos + 1)
						.map(|&x| (x >> 3) & 7)
						== Some(entry.reg_opcode)
		} else {
			raw_opcode == entry.code
		};

		if found {
			matched = Some(entry);
			break;
		}
	}

	let entry = matched.ok_or(HookError::UnknownInstruction(raw_opcode))?;

	pos += 1;

	if entry.flags & FLAG_RELOCATION != 0 {
		relocation_opcode_offset = Some(pos);
	}

	if entry.flags & FLAG_MODRM != 0 {
		let modrm = *code.get(pos).ok_or(HookError::TruncatedInstruction)?;
		pos += 1;

		let addr_mode = modrm >> 6;
		let rm_field = modrm & 0x07;

		if addr_mode != 3 && rm_field == 4 {
			let scale_index_base = *code.get(pos).ok_or(HookError::TruncatedInstruction)?;
			pos += 1;
			let sib_base = scale_index_base & 0x07;
			if sib_base == 5 {
				pos += match addr_mode {
					1 => 1,
					_ => 4,
				};
			}
		}

		#[cfg(target_arch = "x86_64")]
		if addr_mode == 0 && rm_field == 5 {
			relocation_opcode_offset = Some(pos);
		}

		if addr_mode == 1 {
			pos += 1;
		}
		if addr_mode == 2 || (addr_mode == 0 && rm_field == 5) {
			pos += 4;
		}
	}

	if entry.flags & FLAG_IMMEDIATE_8BIT != 0 {
		pos += 1;
	}
	if entry.flags & FLAG_IMMEDIATE_16BIT != 0 {
		pos += 2;
	}
	if entry.flags & FLAG_IMMEDIATE_32BIT != 0 {
		pos += operand_size;
	}

	Ok((pos, relocation_opcode_offset))
}
