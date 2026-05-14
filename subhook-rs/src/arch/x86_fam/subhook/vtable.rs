use crate::error::HookError;

/// Patch one vtable slot and return the previous function pointer.
///
/// This mirrors the manual vtable slot write currently used by rubles-rs DX11 hooks,
/// but keeps the low-level patching logic centralized in subhook-rs.
pub unsafe fn patch_vtable_slot(slot: *const *mut (), new_fn: *mut ()) -> Result<*mut (), HookError> {
	if slot.is_null() {
		return Err(HookError::NullPointer(crate::error::NullPointerSource::Src));
	}

	let slot_addr = slot as *mut u8;
	let size = core::mem::size_of::<usize>();
	let old = unsafe { crate::mem::unprotect_with_old(slot_addr, size) }?;

	let prev = unsafe { core::ptr::read_volatile(slot) };
	unsafe { core::ptr::write_volatile(slot as *mut *mut (), new_fn) };

	unsafe { crate::mem::reprotect(slot_addr, size, old) }?;
	Ok(prev)
}
