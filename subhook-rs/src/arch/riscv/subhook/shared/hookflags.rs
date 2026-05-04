#[derive(Debug, PartialEq, Eq, Default, Clone, Copy)]
pub struct HookFlags(u32);

impl HookFlags {
	pub const NONE: Self = Self(0);
}
