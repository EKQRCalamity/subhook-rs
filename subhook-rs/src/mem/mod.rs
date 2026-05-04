#[cfg(unix)]
pub (crate) mod unix {
	pub (crate) mod libc;
}

#[cfg(windows)]
pub (crate) mod windows {
	pub (crate) mod winapi;
}

#[cfg(unix)]
pub (crate) use unix::libc::{alloc_code, free_code, unprotect};

#[cfg(windows)]
pub (crate) use windows::winapi::{alloc_code, free_code, unprotect};
