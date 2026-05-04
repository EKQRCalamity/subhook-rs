
use crate::error::HookError;

/// Trait for types that can be installed and removed as hooks.
///
/// This trait provides a minimal interface for hook management,
/// allowing `ScopedRemove` to work with different hook types
/// (inline hooks, IAT hooks, vtable hooks, etc.).
pub trait Hookable {
	/// Install the hook.
	fn install(&mut self) -> Result<(), HookError>;
	
	/// Remove the hook.
	fn remove(&mut self) -> Result<(), HookError>;
}

/// Removes a hook on construction and reinstalls it on drop.
/// Useful for calling the original function from within a hook handler.
///
/// Generic over any type implementing `Hookable`, allowing it to work
/// with inline hooks, IAT hooks, and future hook types.
pub struct ScopedRemove<'a, T: Hookable> {
	hook:    &'a mut T,
	removed: bool,
}

impl<'a, T: Hookable> ScopedRemove<'a, T> {
	pub fn new(hook: &'a mut T) -> Self {
		let removed = hook.remove().is_ok();
		Self { hook, removed }
	}

	/// Returns a shared reference to the inner hook.
	pub fn inner(&self) -> &T {
		self.hook
	}

	/// Returns `true` if the hook was successfully removed when this guard was created.
	///
	/// Returns `false` if the hook was not installed at construction time, in which
	/// case the guard is a no-op and no reinstall will occur on drop.
	pub fn removed(&self) -> bool {
		self.removed
	}
}

impl<T: Hookable> Drop for ScopedRemove<'_, T> {
	fn drop(&mut self) {
		if self.removed {
			let _ = self.hook.install();
		}
	}
}
