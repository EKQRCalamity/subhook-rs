## Subhook-rs

This is an extended port of the old, now removed, Zeex subhook C/C++ library. Long term I plan to support other hooking implementations as well. This crate supports x86 32bit and 64bit hooking for Unix and Windows systems. 

Linux was already tested, Windows and Macos were not. I extended the old library a bit, added a MacOS fallback with `vm_protect` instead of `mprotect` and did some tiny additions to the disassambler for 2 byte opcodes.


### Usage / Examples
Coming soon

I tried to use rust doc comments where useful, so hopefully the documentation is somewhat useable even without examples for the time being.

### License
BSD-2-Clause - see [License](https://github.com/EKQRCalamity/subhook-rs/tree/master/LICENSE).

The original project was licensed under the BSD 2-Clause License. See [Original License](https://github.com/EKQRCalamity/subhook-rs/tree/master/ORIGINAL_LICENSE) for the original copyright notice.
