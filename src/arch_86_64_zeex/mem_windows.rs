use crate::error::HookError;
use windows_sys::Win32::System::Memory::{
    VirtualAlloc, VirtualFree, VirtualProtect,
    MEM_COMMIT, MEM_RELEASE, MEM_RESERVE,
    PAGE_EXECUTE_READWRITE,
};

/// Try to make `size` bytes starting at `addr` readable, writable, and executable.
///
/// Returns an empty `Result` or a hook error of type `HookError::UnprotectFailed` with the OS
/// error code.
pub(crate) unsafe fn unprotect(addr: *mut u8, size: usize) -> Result<(), HookError> {
    let mut old_flags: u32 = 0;
    let result = VirtualProtect(
        addr as *const _,
        size,
        PAGE_EXECUTE_READWRITE,
        &mut old_flags,
    );
    if result == 0 {
        // GetLastError() would be more precise but i32 cast is fine for our error type.
        Err(HookError::UnprotectFailed(windows_sys::Win32::Foundation::GetLastError() as i32))
    } else {
        Ok(())
    }
}

/// Allocate `size` bytes of RWX memory.
///
/// Returns an empty `Result` or a hook error of type `HookError::AllocationFailed` with the OS
/// error code.
pub(crate) unsafe fn alloc_code(size: usize) -> Result<*mut u8, HookError> {
    let ptr = VirtualAlloc(
        std::ptr::null(),
        size,
        MEM_COMMIT | MEM_RESERVE,
        PAGE_EXECUTE_READWRITE,
    );
    if ptr.is_null() {
        Err(HookError::AllocationFailed(unsafe { windows_sys::Win32::Foundation::GetLastError() } as i32))
    } else {
        Ok(ptr as *mut u8)
    }
}

/// Release memory previously allocated via `alloc_code`.
pub(crate) unsafe fn free_code(addr: *mut u8, _size: usize) {
    if !addr.is_null() {
        VirtualFree(addr as *mut _, 0, MEM_RELEASE);
    }
}
