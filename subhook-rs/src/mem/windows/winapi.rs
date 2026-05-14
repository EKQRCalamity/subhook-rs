use crate::error::HookError;
use windows_sys::Win32::System::Memory::{
    VirtualAlloc, VirtualFree, VirtualProtect,
    MEM_COMMIT, MEM_RELEASE, MEM_RESERVE,
    PAGE_EXECUTE_READWRITE,
};
use windows_sys::Win32::System::Diagnostics::Debug::FlushInstructionCache;
use windows_sys::Win32::System::Threading::GetCurrentProcess;

/// Temporarily switch a code region to RWX and return the previous page flags.
pub(crate) unsafe fn unprotect_with_old(addr: *mut u8, size: usize) -> Result<u32, HookError> {
    let mut old_flags: u32 = 0;
    let result = unsafe {
        VirtualProtect(
            addr as *const _,
            size,
            PAGE_EXECUTE_READWRITE,
            &mut old_flags,
        )
    };
    if result == 0 {
        Err(HookError::UnprotectFailed(unsafe {
            windows_sys::Win32::Foundation::GetLastError()
        } as i32))
    } else {
        Ok(old_flags)
    }
}

/// Restore a previously saved page protection value.
pub(crate) unsafe fn reprotect(addr: *mut u8, size: usize, old_flags: u32) -> Result<(), HookError> {
    let mut prev: u32 = 0;
    let result = unsafe {
        VirtualProtect(
            addr as *const _,
            size,
            old_flags,
            &mut prev,
        )
    };
    if result == 0 {
        Err(HookError::UnprotectFailed(unsafe {
            windows_sys::Win32::Foundation::GetLastError()
        } as i32))
    } else {
        Ok(())
    }
}

/// Flush CPU instruction cache for freshly patched code bytes.
pub(crate) unsafe fn flush_icache(addr: *mut u8, size: usize) -> Result<(), HookError> {
    let result = unsafe { FlushInstructionCache(GetCurrentProcess(), addr as *const _, size) };
    if result == 0 {
        Err(HookError::UnprotectFailed(unsafe {
            windows_sys::Win32::Foundation::GetLastError()
        } as i32))
    } else {
        Ok(())
    }
}

/// Allocate `size` bytes of RWX memory.
///
/// Returns an empty `Result` or a hook error of type `HookError::AllocationFailed` with the OS
/// error code.
pub(crate) unsafe fn alloc_code(size: usize) -> Result<*mut u8, HookError> {
    let ptr = unsafe { VirtualAlloc(
        std::ptr::null(),
        size,
        MEM_COMMIT | MEM_RESERVE,
        PAGE_EXECUTE_READWRITE,
    ) };
    if ptr.is_null() {
        Err(HookError::AllocationFailed(unsafe { windows_sys::Win32::Foundation::GetLastError() } as i32))
    } else {
        Ok(ptr as *mut u8)
    }
}

/// Allocate `size` bytes of RWX memory within 2 GB of `near`.
///
/// Scans aligned pages from `near` outward so that rel32 relocations in copied
/// prologues remain representable.  Falls back to unconstrained allocation if
/// no near page is available.
pub(crate) unsafe fn alloc_code_near(near: *const u8, size: usize) -> Result<*mut u8, HookError> {
    use windows_sys::Win32::System::Memory::{VirtualQuery, MEMORY_BASIC_INFORMATION};

    const GRANULARITY: usize = 0x10000;
    const MAX_RANGE: usize = 0x7FFF0000;

    let origin = near as usize;

    let mut lo = origin.saturating_sub(GRANULARITY) & !(GRANULARITY - 1);
    let mut hi = (origin + GRANULARITY) & !(GRANULARITY - 1);

    for _ in 0..(MAX_RANGE / GRANULARITY) {
        // Forward candidate
        if hi < origin.saturating_add(MAX_RANGE) {
            let mut mbi = unsafe { std::mem::zeroed::<MEMORY_BASIC_INFORMATION>() };
            let ok = unsafe { VirtualQuery(hi as *const _, &mut mbi, std::mem::size_of::<MEMORY_BASIC_INFORMATION>()) };
            if ok != 0 && mbi.State == windows_sys::Win32::System::Memory::MEM_FREE {
                let ptr = unsafe { VirtualAlloc(hi as *const _, size, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE) };
                if !ptr.is_null() {
                    return Ok(ptr as *mut u8);
                }
            }
            hi = hi.saturating_add(GRANULARITY);
        }

        // Backward candidate
        if lo > origin.saturating_sub(MAX_RANGE) {
            let mut mbi = unsafe { std::mem::zeroed::<MEMORY_BASIC_INFORMATION>() };
            let ok = unsafe { VirtualQuery(lo as *const _, &mut mbi, std::mem::size_of::<MEMORY_BASIC_INFORMATION>()) };
            if ok != 0 && mbi.State == windows_sys::Win32::System::Memory::MEM_FREE {
                let ptr = unsafe { VirtualAlloc(lo as *const _, size, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE) };
                if !ptr.is_null() {
                    return Ok(ptr as *mut u8);
                }
            }
            lo = lo.saturating_sub(GRANULARITY);
        }
    }

    unsafe { alloc_code(size) }
}

/// Release memory previously allocated via `alloc_code`.
pub(crate) unsafe fn free_code(addr: *mut u8, _size: usize) {
    if !addr.is_null() {
        unsafe { VirtualFree(addr as *mut _, 0, MEM_RELEASE) };
    }
}

#[cfg(feature = "thread_suspend")]
mod thread {
    use windows_sys::Win32::Foundation::{HANDLE, GetLastError};
    use windows_sys::Win32::System::Threading::{SuspendThread, ResumeThread};
    use windows_sys::Win32::System::Diagnostics::Debug::{
        GetThreadContext, SetThreadContext, CONTEXT,
        CONTEXT_CONTROL_AMD64,
    };
    #[cfg(target_arch = "x86")]
    use windows_sys::Win32::System::Diagnostics::Debug::CONTEXT_CONTROL_X86;
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
