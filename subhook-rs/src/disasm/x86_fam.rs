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
	// JMP short rel8
	OpCodeInformation { code: 0xEB, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	// Jcc short rel8 (0x70-0x7F): JO JNO JB JAE JE JNE JBE JA JS JNS JP JNP JL JGE JLE JG
	OpCodeInformation { code: 0x70, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x71, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x72, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x73, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x74, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x75, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x76, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x77, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x78, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x79, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x7A, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x7B, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x7C, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x7D, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x7E, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x7F, reg_opcode: 0, flags: FLAG_IMMEDIATE_8BIT },
	// MOVSXD r64, r/m32
	OpCodeInformation { code: 0x63, reg_opcode: 0, flags: FLAG_MODRM },
	// IMUL r, r/m, imm32
	OpCodeInformation { code: 0x69, reg_opcode: 0, flags: FLAG_MODRM | FLAG_IMMEDIATE_32BIT },
	// IMUL r, r/m, imm8
	OpCodeInformation { code: 0x6B, reg_opcode: 0, flags: FLAG_MODRM | FLAG_IMMEDIATE_8BIT },
	// Shift/Rotate group 2: C0=r/m8,imm8  C1=r/m,imm8  D0=r/m8,1  D1=r/m,1  D2=r/m8,CL  D3=r/m,CL
	OpCodeInformation { code: 0xC0, reg_opcode: 0, flags: FLAG_MODRM | FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0xC1, reg_opcode: 0, flags: FLAG_MODRM | FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0xD0, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0xD1, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0xD2, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0xD3, reg_opcode: 0, flags: FLAG_MODRM },
	// NOT/NEG/MUL/IMUL/DIV/IDIV r/m8 (F6) and r/m (F7)
	OpCodeInformation { code: 0xF6, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0xF7, reg_opcode: 0, flags: FLAG_MODRM },
	// INC/DEC r/m
	OpCodeInformation { code: 0xFE, reg_opcode: 0, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE },
	OpCodeInformation { code: 0xFE, reg_opcode: 1, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE },
	OpCodeInformation { code: 0xFF, reg_opcode: 0, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE },
	// XCHG r/m, r
	OpCodeInformation { code: 0x86, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x87, reg_opcode: 0, flags: FLAG_MODRM },
	// XCHG rAX, r (91-97)
	OpCodeInformation { code: 0x90, reg_opcode: 0, flags: FLAG_PLUS_REGISTER },
	// CBW/CWDE/CDQE
	OpCodeInformation { code: 0x98, reg_opcode: 0, flags: 0 },
	// CDQ/CQO
	OpCodeInformation { code: 0x99, reg_opcode: 0, flags: 0 },
	// PUSHFQ / POPFQ
	OpCodeInformation { code: 0x9C, reg_opcode: 0, flags: 0 },
	OpCodeInformation { code: 0x9D, reg_opcode: 0, flags: 0 },
	// SAHF / LAHF
	OpCodeInformation { code: 0x9E, reg_opcode: 0, flags: 0 },
	OpCodeInformation { code: 0x9F, reg_opcode: 0, flags: 0 },
	// ADC
	OpCodeInformation { code: 0x10, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x11, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x12, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x13, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x80, reg_opcode: 2, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x81, reg_opcode: 2, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_32BIT },
	OpCodeInformation { code: 0x83, reg_opcode: 2, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_8BIT },
	// SBB
	OpCodeInformation { code: 0x18, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x19, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x1A, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x1B, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x80, reg_opcode: 3, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0x81, reg_opcode: 3, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_32BIT },
	OpCodeInformation { code: 0x83, reg_opcode: 3, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_8BIT },
	// CMP (extra forms)
	OpCodeInformation { code: 0x38, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x3A, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x3B, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x81, reg_opcode: 7, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_32BIT },
	OpCodeInformation { code: 0x80, reg_opcode: 7, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_8BIT },
	// MOVS/CMPS/SCAS/LODS/STOS
	OpCodeInformation { code: 0xA4, reg_opcode: 0, flags: 0 },
	OpCodeInformation { code: 0xA5, reg_opcode: 0, flags: 0 },
	OpCodeInformation { code: 0xA6, reg_opcode: 0, flags: 0 },
	OpCodeInformation { code: 0xA7, reg_opcode: 0, flags: 0 },
	OpCodeInformation { code: 0xAA, reg_opcode: 0, flags: 0 },
	OpCodeInformation { code: 0xAB, reg_opcode: 0, flags: 0 },
	OpCodeInformation { code: 0xAC, reg_opcode: 0, flags: 0 },
	OpCodeInformation { code: 0xAD, reg_opcode: 0, flags: 0 },
	OpCodeInformation { code: 0xAE, reg_opcode: 0, flags: 0 },
	OpCodeInformation { code: 0xAF, reg_opcode: 0, flags: 0 },
	// CLC/STC/CLD/STD
	OpCodeInformation { code: 0xF8, reg_opcode: 0, flags: 0 },
	OpCodeInformation { code: 0xF9, reg_opcode: 0, flags: 0 },
	OpCodeInformation { code: 0xFC, reg_opcode: 0, flags: 0 },
	OpCodeInformation { code: 0xFD, reg_opcode: 0, flags: 0 },
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
	// Jcc near rel32 (0F 80-0F 8F)
	OpCodeInformation { code: 0x80, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT | FLAG_RELOCATION },
	OpCodeInformation { code: 0x81, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT | FLAG_RELOCATION },
	OpCodeInformation { code: 0x82, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT | FLAG_RELOCATION },
	OpCodeInformation { code: 0x83, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT | FLAG_RELOCATION },
	OpCodeInformation { code: 0x84, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT | FLAG_RELOCATION },
	OpCodeInformation { code: 0x85, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT | FLAG_RELOCATION },
	OpCodeInformation { code: 0x86, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT | FLAG_RELOCATION },
	OpCodeInformation { code: 0x87, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT | FLAG_RELOCATION },
	OpCodeInformation { code: 0x88, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT | FLAG_RELOCATION },
	OpCodeInformation { code: 0x89, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT | FLAG_RELOCATION },
	OpCodeInformation { code: 0x8A, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT | FLAG_RELOCATION },
	OpCodeInformation { code: 0x8B, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT | FLAG_RELOCATION },
	OpCodeInformation { code: 0x8C, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT | FLAG_RELOCATION },
	OpCodeInformation { code: 0x8D, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT | FLAG_RELOCATION },
	OpCodeInformation { code: 0x8E, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT | FLAG_RELOCATION },
	OpCodeInformation { code: 0x8F, reg_opcode: 0, flags: FLAG_IMMEDIATE_32BIT | FLAG_RELOCATION },
	// SETcc (0F 90-0F 9F)
	OpCodeInformation { code: 0x90, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x91, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x92, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x93, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x94, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x95, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x96, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x97, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x98, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x99, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x9A, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x9B, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x9C, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x9D, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x9E, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x9F, reg_opcode: 0, flags: FLAG_MODRM },
	// BT/BTS/BTR/BTC r/m, r
	OpCodeInformation { code: 0xA3, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0xAB, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0xB3, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0xBB, reg_opcode: 0, flags: FLAG_MODRM },
	// BT/BTS/BTR/BTC r/m, imm8 (0F BA /4-/7)
	OpCodeInformation { code: 0xBA, reg_opcode: 4, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0xBA, reg_opcode: 5, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0xBA, reg_opcode: 6, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0xBA, reg_opcode: 7, flags: FLAG_MODRM | FLAG_REGISTER_OPCODE | FLAG_IMMEDIATE_8BIT },
	// IMUL r, r/m
	OpCodeInformation { code: 0xAF, reg_opcode: 0, flags: FLAG_MODRM },
	// BSF / TZCNT (with F3 prefix)
	OpCodeInformation { code: 0xBC, reg_opcode: 0, flags: FLAG_MODRM },
	// BSR / LZCNT (with F3 prefix)
	OpCodeInformation { code: 0xBD, reg_opcode: 0, flags: FLAG_MODRM },
	// POPCNT (with F3 prefix)
	OpCodeInformation { code: 0xB8, reg_opcode: 0, flags: FLAG_MODRM },
	// XADD r/m, r
	OpCodeInformation { code: 0xC0, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0xC1, reg_opcode: 0, flags: FLAG_MODRM },
	// CMPPS/CMPSS/CMPPD/CMPSD xmm, xmm/m, imm8
	OpCodeInformation { code: 0xC2, reg_opcode: 0, flags: FLAG_MODRM | FLAG_IMMEDIATE_8BIT },
	// BSWAP r32/r64 (C8-CF, encoded as +rd in opcode)
	OpCodeInformation { code: 0xC8, reg_opcode: 0, flags: 0 },
	OpCodeInformation { code: 0xC9, reg_opcode: 0, flags: 0 },
	OpCodeInformation { code: 0xCA, reg_opcode: 0, flags: 0 },
	OpCodeInformation { code: 0xCB, reg_opcode: 0, flags: 0 },
	OpCodeInformation { code: 0xCC, reg_opcode: 0, flags: 0 },
	OpCodeInformation { code: 0xCD, reg_opcode: 0, flags: 0 },
	OpCodeInformation { code: 0xCE, reg_opcode: 0, flags: 0 },
	OpCodeInformation { code: 0xCF, reg_opcode: 0, flags: 0 },
	// SSE scalar/packed arithmetic (ANDPS/ANDNPS/ORPS/XORPS/ADDPS/MULPS/SUBPS/MINPS/DIVPS/MAXPS)
	OpCodeInformation { code: 0x54, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x55, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x56, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x57, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x58, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x59, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x5A, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x5B, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x5C, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x5D, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x5E, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0x5F, reg_opcode: 0, flags: FLAG_MODRM },
	// CVTPI2PS/CVTSI2SS/CVTPI2PD/CVTSI2SD
	OpCodeInformation { code: 0x2A, reg_opcode: 0, flags: FLAG_MODRM },
	// CVTTPS2PI/CVTTSS2SI/CVTTPD2PI/CVTTSD2SI
	OpCodeInformation { code: 0x2C, reg_opcode: 0, flags: FLAG_MODRM },
	// CVTPS2PI/CVTSS2SI/CVTPD2PI/CVTSD2SI
	OpCodeInformation { code: 0x2D, reg_opcode: 0, flags: FLAG_MODRM },
	// UCOMISS/UCOMISD
	OpCodeInformation { code: 0x2E, reg_opcode: 0, flags: FLAG_MODRM },
	// COMISS/COMISD
	OpCodeInformation { code: 0x2F, reg_opcode: 0, flags: FLAG_MODRM },
	// SQRTPS/SQRTSS
	OpCodeInformation { code: 0x51, reg_opcode: 0, flags: FLAG_MODRM },
	// RSQRTPS/RSQRTSS
	OpCodeInformation { code: 0x52, reg_opcode: 0, flags: FLAG_MODRM },
	// RCPPS/RCPSS
	OpCodeInformation { code: 0x53, reg_opcode: 0, flags: FLAG_MODRM },
	// MOVD/MOVQ xmm, r/m
	OpCodeInformation { code: 0x6E, reg_opcode: 0, flags: FLAG_MODRM },
	// MOVDQA/MOVDQU xmm, xmm/m
	OpCodeInformation { code: 0x6F, reg_opcode: 0, flags: FLAG_MODRM },
	// PSHUFD/PSHUFHW/PSHUFLW xmm, xmm/m, imm8
	OpCodeInformation { code: 0x70, reg_opcode: 0, flags: FLAG_MODRM | FLAG_IMMEDIATE_8BIT },
	// MOVD/MOVQ r/m, xmm
	OpCodeInformation { code: 0x7E, reg_opcode: 0, flags: FLAG_MODRM },
	// MOVDQA/MOVDQU xmm/m, xmm
	OpCodeInformation { code: 0x7F, reg_opcode: 0, flags: FLAG_MODRM },
	// MOVQ xmm, xmm/m64
	OpCodeInformation { code: 0xD6, reg_opcode: 0, flags: FLAG_MODRM },
	// Integer SSE: PADDQ PMULLW PAND PANDN POR PXOR PMULUDQ
	OpCodeInformation { code: 0xD4, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0xD5, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0xDB, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0xDF, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0xEB, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0xEF, reg_opcode: 0, flags: FLAG_MODRM },
	OpCodeInformation { code: 0xF4, reg_opcode: 0, flags: FLAG_MODRM },
	// MOVNTQ/MOVNTDQ
	OpCodeInformation { code: 0xE7, reg_opcode: 0, flags: FLAG_MODRM },
	// SHLD r/m, r, imm8 / CL
	OpCodeInformation { code: 0xA4, reg_opcode: 0, flags: FLAG_MODRM | FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0xA5, reg_opcode: 0, flags: FLAG_MODRM },
	// SHRD r/m, r, imm8 / CL
	OpCodeInformation { code: 0xAC, reg_opcode: 0, flags: FLAG_MODRM | FLAG_IMMEDIATE_8BIT },
	OpCodeInformation { code: 0xAD, reg_opcode: 0, flags: FLAG_MODRM },
	// RDTSC
	OpCodeInformation { code: 0x31, reg_opcode: 0, flags: 0 },
	// PUSH FS/GS / POP FS/GS
	OpCodeInformation { code: 0xA0, reg_opcode: 0, flags: 0 },
	OpCodeInformation { code: 0xA1, reg_opcode: 0, flags: 0 },
	OpCodeInformation { code: 0xA8, reg_opcode: 0, flags: 0 },
	OpCodeInformation { code: 0xA9, reg_opcode: 0, flags: 0 },
];

/// How an instruction's relative operand must be fixed up when copied into a trampoline.
#[derive(Debug, PartialEq)]
pub enum RelocHint {
    /// Instruction contains a 4-byte relative immediate at byte offset `op_offset` within the
    /// (already-copied) instruction. Subtract the per-instruction position delta to fix it up.
    I32(usize),
    /// Short conditional jump (`7x disp8`). Must be expanded to `0F 8x disp32` (6 bytes).
    ShortJcc(u8),
    /// Short unconditional jump (`EB disp8`). Must be expanded to `E9 disp32` (5 bytes).
    ShortJmp,
}

/// Decode one instruction from `code`
///
/// Returns `(len, reloc)` where `reloc` describes any necessary relocation.
pub fn disasm(code: &[u8]) -> Result<(usize, Option<RelocHint>), HookError> {
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

    let mut relocation_opcode_offset: Option<RelocHint> = None;

    if raw_opcode == 0x0F {
        pos += 1;
        let opcode2 = *code.get(pos).ok_or(HookError::TruncatedInstruction)?;

        if opcode2 == 0x38 {
            pos += 2; // skip 0x38 + opcode3
            let modrm = *code.get(pos).ok_or(HookError::TruncatedInstruction)?;
            pos += 1;
            let addr_mode = modrm >> 6;
            let rm_field = modrm & 0x07;
            if addr_mode != 3 && rm_field == 4 {
                let sib = *code.get(pos).ok_or(HookError::TruncatedInstruction)?;
                pos += 1;
                if (sib & 0x07) == 5 {
                    pos += match addr_mode {
                        1 => 1,
                        _ => 4,
                    };
                }
            }
            if addr_mode == 1 {
                pos += 1;
            }
            if addr_mode == 2 || (addr_mode == 0 && rm_field == 5) {
                pos += 4;
            }
            return Ok((pos, None));
        }

        if opcode2 == 0x3A {
            pos += 2; // skip 0x3A + opcode3
            let modrm = *code.get(pos).ok_or(HookError::TruncatedInstruction)?;
            pos += 1;
            let addr_mode = modrm >> 6;
            let rm_field = modrm & 0x07;
            if addr_mode != 3 && rm_field == 4 {
                let sib = *code.get(pos).ok_or(HookError::TruncatedInstruction)?;
                pos += 1;
                if (sib & 0x07) == 5 {
                    pos += match addr_mode {
                        1 => 1,
                        _ => 4,
                    };
                }
            }
            if addr_mode == 1 {
                pos += 1;
            }
            if addr_mode == 2 || (addr_mode == 0 && rm_field == 5) {
                pos += 4;
            }
            pos += 1; // imm8
            return Ok((pos, None));
        }

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
            if addr_mode == 0 && rm_field == 5 {
                relocation_opcode_offset = Some(RelocHint::I32(pos));
            }
            if addr_mode == 1 {
                pos += 1;
            }
            if addr_mode == 2 || (addr_mode == 0 && rm_field == 5) {
                pos += 4;
            }
        }

        // Handle relocation and immediates for two-byte instructions (Jcc near, CMPPS, etc.)
        if entry.flags & FLAG_RELOCATION != 0 {
            relocation_opcode_offset = Some(RelocHint::I32(pos));
        }
        if entry.flags & FLAG_IMMEDIATE_8BIT != 0 {
            pos += 1;
        }
        if entry.flags & FLAG_IMMEDIATE_32BIT != 0 {
            pos += 4;
        }

        return Ok((pos, relocation_opcode_offset));
    }

    let mut matched: Option<&OpCodeInformation> = None;

    for entry in OPCODE {
        let found = if entry.flags & FLAG_PLUS_REGISTER != 0 {
            (raw_opcode & 0xF8) == entry.code
        } else if entry.flags & FLAG_REGISTER_OPCODE != 0 {
            raw_opcode == entry.code
                && code.get(pos + 1).map(|&x| (x >> 3) & 7) == Some(entry.reg_opcode)
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
        relocation_opcode_offset = Some(RelocHint::I32(pos));
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
            relocation_opcode_offset = Some(RelocHint::I32(pos));
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

    let reloc_hint = match raw_opcode {
        0x70..=0x7F => Some(RelocHint::ShortJcc(raw_opcode)),
        0xEB => Some(RelocHint::ShortJmp),
        _ => relocation_opcode_offset,
    };
    Ok((pos, reloc_hint))
}
