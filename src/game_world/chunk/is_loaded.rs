#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IsLoaded {
	is_simple_loaded: bool,
	is_visible: bool,
}

impl IsLoaded {
	pub const SIMPLE_LOADED: Self = Self {
		is_simple_loaded: true,
		is_visible: false,
	};
	pub const NOT_LOADED: Self = Self {
		is_simple_loaded: false,
		is_visible: false,
	};
	#[allow(dead_code)]
	pub const VISIBLE: Self = Self {
		is_simple_loaded: true,
		is_visible: true,
	};

	// having these methods feels kind of reduntant

	pub fn is_simple_loaded(&self) -> bool {
		self.is_simple_loaded
	}

	pub fn set_simple_loaded(&mut self, value: bool) {
		self.is_simple_loaded = value;
		// if a chunk isn't loaded at all it should also be invisible
		if !value {
			self.is_visible = false;
		}
	}

	pub fn is_visible(&self) -> bool {
		self.is_visible
	}

	pub fn set_visible(&mut self, value: bool) {
		self.is_visible = value;
	}
}

impl Default for IsLoaded {
	fn default() -> Self {
		Self::NOT_LOADED
	}
}
