use crate::error::HookError;

pub(crate) type ProtectToken = ();

/// Try to make `size` bytes starting at `addr` readable, writable, and executable.
///
/// Returns an empty `Result` or a hook error of type `HookError::UnprotectFailed` with the OS
/// error code.
pub (crate) unsafe fn unprotect(addr: *mut u8, size: usize) -> Result<(), HookError> {
	let page = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as usize;
	let aligned = (addr as usize) & !(page - 1);
	let new_size = (addr as usize + size) - aligned;

	let ret = unsafe {
		libc::mprotect(
			aligned as *mut libc::c_void,
			new_size,
			libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
		)
	};

	if ret == 0 {
		return Ok(());
	}

	#[cfg(target_os = "macos")]
	{
		let kernel_return = unsafe {
			libc::vm_protect(
				lib::mach_task_self(),
				aligned as libc::vm_address_t,
				new_size,
				0,
				libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC | 0x10 /* VM PROT COPY */,
			);
			if kernel_return == 0 {
				return Ok(());
			}
			return Err(HookError::UnprotectFailed(kernel_return));
		};
	}

	#[cfg(not(target_os = "macos"))]
	{
		// _errno_location is, if an error occured, always safe to call
		let code = unsafe { *libc::__errno_location() };
		Err(HookError::UnprotectFailed(code))
	}
}

pub(crate) unsafe fn unprotect_with_old(addr: *mut u8, size: usize) -> Result<ProtectToken, HookError> {
	unsafe { unprotect(addr, size) }?;
	Ok(())
}

pub(crate) unsafe fn reprotect(_addr: *mut u8, _size: usize, _old: ProtectToken) -> Result<(), HookError> {
	Ok(())
}

pub(crate) unsafe fn flush_icache(_addr: *mut u8, _size: usize) -> Result<(), HookError> {
	Ok(())
}

/// Allocate `size` bytes of RWX memory.
///
/// Returns an empty `Result` or a hook error of type `HookError::AllocationFailed` with the OS
/// error code.
pub(crate) unsafe fn alloc_code(size: usize) -> Result<*mut u8, HookError> {
	let flags = libc::MAP_PRIVATE | libc::MAP_ANONYMOUS;

	let ptr = unsafe {
		libc::mmap(
			std::ptr::null_mut(),
			size,
			libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
			flags,
			-1,
			0,
		)
	};

	if ptr == libc::MAP_FAILED {
		let code = unsafe { *libc::__errno_location() };
		Err(HookError::AllocationFailed(code))
	} else {
		Ok(ptr as *mut u8)
	}
}

/// Allocate `size` bytes of RWX memory near `_near`. On Unix falls back to unconstrained mmap.
pub(crate) unsafe fn alloc_code_near(_near: *const u8, size: usize) -> Result<*mut u8, HookError> {
	unsafe { alloc_code(size) }
}

/// Release memory previously allocated via `alloc_code`.
pub(crate) unsafe fn free_code(addr: *mut u8, size: usize) {
	if !addr.is_null() {
		unsafe { libc::munmap(addr as *mut libc::c_void, size) };
	}
}
