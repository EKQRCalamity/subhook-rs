## Subhook-rs

This is an extended port of the old, now removed, Zeex subhook C/C++ library. Long term I plan to support other hooking implementations as well. This crate supports x86 (32 and 64 bit) and RISC-V (32 and 64 bit) hooking for Unix and Windows systems. 

x86 on Linux was tested, Windows and MacOS were not. RISC-V was tested on Linux with QEMU emulation. I extended the old x86 library a bit, added a MacOS fallback with `vm_protect` instead of `mprotect` and did some tiny additions to the disassembler for 2 byte opcodes. The RISC-V implementation follows the same structure as the x86 one but handles compressed instructions and RISC-V specific PC-relative relocations.


### Features

By default, both x86 and RISC-V subhook implementations are enabled. You can control this with cargo features for fine-grained control over what gets compiled in:

- `x86_fam` - Enables x86/x86_64 architecture support (pulls in OS dependencies)
- `x86_fam_subhook` - Enables the subhook implementation for x86 (requires `x86_fam`)
- `riscv` - Enables RISC-V (32 and 64 bit) architecture support (pulls in OS dependencies)
- `riscv_subhook` - Enables the subhook implementation for RISC-V (requires `riscv`)
- `x86_64_Zeex` - Legacy compatibility feature, will be removed later (same as `x86_fam_subhook`)

The split between base architecture features (`x86_fam`, `riscv`) and implementation features (`x86_fam_subhook`, `riscv_subhook`) lets you pick exactly which hooking implementation you want. When I add other implementations later (like VTable, PLT, IAT, ...), you'll be able to choose between them without pulling in code you don't need.

If you only need x86 support, disable default features and enable just what you want:
```toml
[dependencies]
subhook-rs = { version = "0.1.0", default-features = false, features = ["x86_fam_subhook"] }
```

### Usage / Examples
Coming soon

I tried to use rust doc comments where useful, so hopefully the documentation is somewhat useable even without examples for the time being.

### License
BSD-2-Clause - see [License](https://github.com/EKQRCalamity/subhook-rs/tree/master/LICENSE).

The original project was licensed under the BSD 2-Clause License. See [Original License](https://github.com/EKQRCalamity/subhook-rs/tree/master/ORIGINAL_LICENSE) for the original copyright notice.
