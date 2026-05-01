#[cfg(unix)]
pub(crate) use super::mem_unix::{alloc_code, free_code, unprotect};

#[cfg(windows)]
pub(crate) use super::mem_windows::{alloc_code, free_code, unprotect};
