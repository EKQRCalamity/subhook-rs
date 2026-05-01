pub mod subhook;
pub mod shared;
pub (crate) mod mem;
#[cfg(test)]
pub mod tests;

#[cfg(unix)]
pub (crate) mod mem_unix;
#[cfg(windows)]
pub (crate) mod mem_windows;
