#[cfg(unix)]
pub (crate) mod unix {
	pub (crate) mod libc;
}

#[cfg(windows)]
pub (crate) mod windows {
	pub (crate) mod winapi;
}

#[cfg(unix)]
pub (crate) use unix::libc::{alloc_code, alloc_code_near, flush_icache, free_code, reprotect, unprotect_with_old};

#[cfg(windows)]
#[allow(dead_code, unused_imports)]
pub (crate) use windows::winapi::{alloc_code, alloc_code_near, flush_icache, free_code, reprotect, unprotect_with_old};

#[cfg(all(windows, feature = "thread_suspend"))]
pub(crate) use windows::winapi::{suspend_thread, resume_thread, get_thread_ip, set_thread_ip};

pub(crate) unsafe fn patch_bytes(dst: *mut u8, src: *const u8, size: usize) -> Result<(), crate::error::HookError> {
	let old = unsafe { unprotect_with_old(dst, size) }?;
	unsafe { std::ptr::copy_nonoverlapping(src, dst, size) };
	unsafe { flush_icache(dst, size) }?;
	unsafe { reprotect(dst, size, old) }
}
