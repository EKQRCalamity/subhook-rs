#[derive(Debug, PartialEq, Eq, Default, Clone, Copy)]
pub struct HookFlags(u32);

impl HookFlags {
	/// Indicates no special behaviour
	pub const NONE: Self = Self(0);
	/// Should use a 14 byte push/mov/ret instead of a 5 byte rel32 jmp, this is required on x86-64
	/// when src and dst are too far apart.
	pub const USE_64BIT_OFFSET: Self = Self(1 << 0);

	/// Returns `true` if all bits in `other` are in `self`
	#[inline]
	pub fn contains(self, other: Self) -> bool {
		self.0 & other.0 == other.0
	}
}

impl std::ops::BitOr for HookFlags {
	type Output = Self;

	fn bitor(self, rhs: Self) -> Self::Output {
		Self(self.0 | rhs.0)
	}
}

impl std::ops::BitAnd for HookFlags {
	type Output = Self;

	fn bitand(self, rhs: Self) -> Self::Output {
	  Self(self.0 & rhs.0)
	}
}
