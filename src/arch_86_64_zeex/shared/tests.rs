use crate::arch_86_64_zeex::shared::disasm::disasm;

#[test]
fn decode_nop() {
	let (len, reloc) = disasm(&[0x90]).unwrap();
	assert_eq!(len, 1);
	assert_eq!(reloc, None);
}

#[test]
fn decode_push_rbp() {
	// 55  push rbp
	let (len, reloc) = disasm(&[0x55]).unwrap();
	assert_eq!(len, 1);
	assert_eq!(reloc, None);
}

#[test]
fn decode_mov_rbp_rsp() {
	// 48 89 E5  mov rbp, rsp
	let (len, reloc) = disasm(&[0x48, 0x89, 0xE5]).unwrap();
	assert_eq!(len, 3);
	assert_eq!(reloc, None);
}

#[test]
fn decode_jmp_rel32() {
	// E9 xx xx xx xx  jmp rel32
	let (len, reloc) = disasm(&[0xE9, 0x00, 0x00, 0x00, 0x00]).unwrap();
	assert_eq!(len, 5);
	assert!(reloc.is_some());
}

#[test]
fn decode_call_rel32() {
	let (len, reloc) = disasm(&[0xE8, 0x01, 0x00, 0x00, 0x00]).unwrap();
	assert_eq!(len, 5);
	assert!(reloc.is_some());
}

#[test]
fn unknown_opcode_errors() {
	assert!(disasm(&[0x0F, 0xFF]).is_err()); // two-byte opcode not in table
}

#[test]
fn decode_endbr64() {
	// F3 0F 1E FA  endbr64
	let (len, reloc) = disasm(&[0xF3, 0x0F, 0x1E, 0xFA]).unwrap();
	assert_eq!(len, 4);
	assert_eq!(reloc, None);
}

#[test]
fn decode_endbr32() {
	// F3 0F 1E FB  endbr32
	let (len, reloc) = disasm(&[0xF3, 0x0F, 0x1E, 0xFB]).unwrap();
	assert_eq!(len, 4);
	assert_eq!(reloc, None);
}

#[test]
fn decode_movzx() {
	// 0F B6 C0  movzx eax, al
	let (len, reloc) = disasm(&[0x0F, 0xB6, 0xC0]).unwrap();
	assert_eq!(len, 3);
	assert_eq!(reloc, None);
}

#[test]
fn decode_cmovne() {
	// 48 0F 45 C8  cmovne rcx, rax  (REX.W + 0F 45 /r)
	let (len, reloc) = disasm(&[0x48, 0x0F, 0x45, 0xC8]).unwrap();
	assert_eq!(len, 4);
	assert_eq!(reloc, None);
}

#[test]
fn decode_multibyte_nop() {
	// 0F 1F 44 00 00  nop dword [rax+rax*1+0]
	let (len, reloc) = disasm(&[0x0F, 0x1F, 0x44, 0x00, 0x00]).unwrap();
	assert_eq!(len, 5);
	assert_eq!(reloc, None);
}
