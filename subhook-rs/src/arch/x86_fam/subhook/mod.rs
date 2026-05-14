mod subhook;
pub use subhook::{Hook, HookFlags};
pub mod vtable;
pub mod shared;
#[cfg(test)]
pub mod tests;
