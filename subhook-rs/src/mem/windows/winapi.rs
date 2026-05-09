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

#[cfg(feature = "thread_suspend")]
mod thread {
    use windows_sys::Win32::Foundation::{HANDLE, GetLastError};
    use windows_sys::Win32::System::Threading::{SuspendThread, ResumeThread};
    use windows_sys::Win32::System::Diagnostics::Debug::{
        GetThreadContext, SetThreadContext, CONTEXT,
        CONTEXT_CONTROL_AMD64, CONTEXT_CONTROL_X86,
    };
    use crate::error::HookError;

    pub(crate) unsafe fn suspend_thread(handle: HANDLE) -> Result<(), HookError> {
        if unsafe { SuspendThread(handle) } == u32::MAX {
            Err(HookError::ThreadOperationFailed(unsafe { GetLastError() }))
        } else {
            Ok(())
        }
    }

    pub(crate) unsafe fn resume_thread(handle: HANDLE) -> Result<(), HookError> {
        if unsafe { ResumeThread(handle) } == u32::MAX {
            Err(HookError::ThreadOperationFailed(unsafe { GetLastError() }))
        } else {
            Ok(())
        }
    }

    pub(crate) unsafe fn get_thread_ip(handle: HANDLE) -> Result<usize, HookError> {
        let mut ctx = unsafe { std::mem::zeroed::<CONTEXT>() };
        #[cfg(target_arch = "x86_64")]
        { ctx.ContextFlags = CONTEXT_CONTROL_AMD64; }
        #[cfg(target_arch = "x86")]
        { ctx.ContextFlags = CONTEXT_CONTROL_X86; }

        if unsafe { GetThreadContext(handle, &mut ctx) } == 0 {
            return Err(HookError::ThreadOperationFailed(unsafe { GetLastError() }));
        }

        #[cfg(target_arch = "x86_64")]
        { Ok(ctx.Rip as usize) }
        #[cfg(target_arch = "x86")]
        { Ok(ctx.Eip as usize) }
    }

    pub(crate) unsafe fn set_thread_ip(handle: HANDLE, ip: usize) -> Result<(), HookError> {
        let mut ctx = unsafe { std::mem::zeroed::<CONTEXT>() };
        #[cfg(target_arch = "x86_64")]
        { ctx.ContextFlags = CONTEXT_CONTROL_AMD64; }
        #[cfg(target_arch = "x86")]
        { ctx.ContextFlags = CONTEXT_CONTROL_X86; }

        if unsafe { GetThreadContext(handle, &mut ctx) } == 0 {
            return Err(HookError::ThreadOperationFailed(unsafe { GetLastError() }));
        }

        #[cfg(target_arch = "x86_64")]
        { ctx.Rip = ip as u64; }
        #[cfg(target_arch = "x86")]
        { ctx.Eip = ip as u32; }

        if unsafe { SetThreadContext(handle, &ctx) } == 0 {
            Err(HookError::ThreadOperationFailed(unsafe { GetLastError() }))
        } else {
            Ok(())
        }
    }
}

#[cfg(feature = "thread_suspend")]
pub(crate) use thread::{suspend_thread, resume_thread, get_thread_ip, set_thread_ip};
