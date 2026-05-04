use super::*;

#[test]
fn simple_hook() {
	#[crate::hook]
	fn add(a: i32, b: i32) -> i32 {
		a + b
	}

	#[crate::hook]
	fn add_hook(a: i32, b: i32) -> i32 {
		a + b + 1
	}

	unsafe {
		let mut hook = Hook::new(
			add as *mut u8,
			add_hook as *const u8,
			HookFlags::NONE,
		).unwrap();

		assert_eq!(add(2, 3), 5);
		hook.install().unwrap();
		assert_eq!(add(2, 3), 6);
		hook.remove().unwrap();
		assert_eq!(add(2, 3), 5);
	}
}

#[test]
fn hook_with_trampoline() {
	static mut ORIGINAL: Option<unsafe extern "C" fn(i32, i32) -> i32> = None;

	#[crate::hook]
	fn add(a: i32, b: i32) -> i32 {
		a + b
	}

	#[crate::hook]
	unsafe fn add_hook(a: i32, b: i32) -> i32 {
		let orig = unsafe { ORIGINAL.unwrap() };
		(unsafe { orig(a, b) }) + 10
	}

	unsafe {
		let hook = Hook::new(
			add as *mut u8,
			add_hook as *const u8,
			HookFlags::NONE,
		).unwrap();

		if let Some(trampoline) = hook.trampoline() {
			ORIGINAL = Some(std::mem::transmute(trampoline));
		}

		assert_eq!(add(5, 7), 12);
	}
}

#[test]
fn double_install_errors() {
	#[crate::hook]
	fn add(a: i32, b: i32) -> i32 {
		a + b
	}

	#[crate::hook]
	fn hook(_a: i32, _b: i32) -> i32 {
		999
	}

	unsafe {
		let mut hook = Hook::new(
			add as *mut u8,
			hook as *const u8,
			HookFlags::NONE,
		).unwrap();

		hook.install().unwrap();
		assert!(hook.install().is_err());
	}
}

#[test]
fn remove_before_install_errors() {
	#[crate::hook]
	fn add(a: i32, b: i32) -> i32 {
		a + b
	}

	#[crate::hook]
	fn hook(_a: i32, _b: i32) -> i32 {
		999
	}

	unsafe {
		let mut hook = Hook::new(
			add as *mut u8,
			hook as *const u8,
			HookFlags::NONE,
		).unwrap();

		assert!(hook.remove().is_err());
	}
}

#[test]
fn scoped_remove_restores_and_reinstalls() {
	use crate::common::ScopedRemove;

	#[crate::hook]
	fn add(a: i32, b: i32) -> i32 {
		a + b
	}

	#[crate::hook]
	fn hook(_a: i32, _b: i32) -> i32 {
		999
	}

	unsafe {
		let mut hook = Hook::new(
			add as *mut u8,
			hook as *const u8,
			HookFlags::NONE,
		).unwrap();

		hook.install().unwrap();
		assert_eq!(add(1, 2), 999);

		{
			let _guard = ScopedRemove::new(&mut hook);
			assert_eq!(add(1, 2), 3);
		}

		assert_eq!(add(1, 2), 999);
	}
}

#[test]
fn test_hook_macro() {
	#[crate::hook]
	fn test_fn(x: i32) -> i32 {
		x * 2
	}

	assert_eq!(test_fn(5), 10);
}
