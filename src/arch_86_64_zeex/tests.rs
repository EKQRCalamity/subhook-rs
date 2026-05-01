use crate::{arch_86_64_zeex::subhook::{Hook, HookFlags, ScopedRemove}, error};
use std::sync::atomic::{AtomicU32, Ordering};

#[inline(never)]
extern "C" fn add(a: u32, b: u32) -> u32 {
	a + b
}

static CALL_COUNT: AtomicU32 = AtomicU32::new(0);

static mut ORIGINAL: Option<unsafe extern "C" fn(u32, u32) -> u32> = None;

#[inline(never)]
extern "C" fn add_hook(a: u32, b: u32) -> u32 {
	CALL_COUNT.fetch_add(1, Ordering::SeqCst);

	unsafe {
		if let Some(orig) = ORIGINAL {
			orig(a, b) + 1337
		} else {
			1337
		}
	}
}

#[test]
fn hook_add_call_trampoline() {
	CALL_COUNT.store(0, Ordering::SeqCst);
	unsafe { ORIGINAL = None; }
	assert_eq!(add(1, 2), 3);

	let mut hook = unsafe {
		Hook::new(
			add as *mut u8,
			add_hook as *const u8,
			HookFlags::NONE,
		)
		.expect("Failed to create hook!")
	};

	assert!(hook.trampoline().is_some(), "Trampoline should have been built...");

	unsafe {
		ORIGINAL = hook.trampoline().map(|p| std::mem::transmute(p));
	}

	hook.install().expect("Intall failed.");
	assert!(hook.is_installed(), "Hook should have been installed...");

	let result = add(1, 2);
	assert_eq!(result, 1340, "Hook did not fire or trampoline returned wrong value!");
	assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 1);

	let result2 = add(10, 20);
	assert_eq!(result2, 1367, "Second hook call failure!");
	assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 2);

	hook.remove().expect("Hook removal failed!");
	assert!(!hook.is_installed());
	assert_eq!(add(1,2), 3);
}

#[test]
fn double_install_errors() {
    let mut hook = unsafe {
        Hook::new(add as *mut u8, add_hook as *const u8, HookFlags::NONE)
            .expect("Failed to create hook!")
    };
    hook.install().expect("First install failed");
    let err = hook.install().unwrap_err();
    assert_eq!(err, error::HookError::AlreadyInstalled);
    hook.remove().ok();
}

#[test]
fn remove_before_install_errors() {
    let mut hook = unsafe {
        Hook::new(add as *mut u8, add_hook as *const u8, HookFlags::NONE)
            .expect("Failed to create hook!")
    };
    let err = hook.remove().unwrap_err();
    assert_eq!(err, error::HookError::NotInstalled);
}

#[test]
fn scoped_remove_restores_and_reinstalls() {
    CALL_COUNT.store(0, Ordering::SeqCst);
    unsafe { ORIGINAL = None; }

    let mut hook = unsafe {
        Hook::new(add as *mut u8, add_hook as *const u8, HookFlags::NONE)
            .expect("Hook::new failed")
    };
    unsafe {
        ORIGINAL = hook.trampoline().map(|p| std::mem::transmute(p));
    }
    hook.install().expect("install failed");

    {
        let guard = ScopedRemove::new(&mut hook);
        assert!(!guard.inner().is_installed());
        // add() should behave normally (no hook).
        assert_eq!(add(1, 2), 3);
        assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 0);
    }

    assert!(hook.is_installed());
    // add() should go through the hook again.
    let result = add(1, 2);
    assert_eq!(result, 1340);
    assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 1);

    hook.remove().ok();
}

