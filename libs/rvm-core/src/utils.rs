pub const fn align_size(bytes: usize, byte_alignment: usize) -> usize {
	let remainder = bytes % byte_alignment;
	if remainder == 0 {
		bytes // Already aligned
	} else {
		bytes + byte_alignment - remainder
	}
}

pub trait ResultUnwrapOrErr {
	type Output;
	fn unwrap_or_err(self) -> Self::Output;
}

impl<V> ResultUnwrapOrErr for Result<V, V> {
	type Output = V;

	fn unwrap_or_err(self) -> Self::Output {
		self.unwrap_or_else(|v| v)
	}
}

pub trait VecExt<T> {
	fn find_and_remove(&mut self, predicate: impl Fn(&T) -> bool) -> Option<T>;
	fn first_where<O>(&self, predicate: impl Fn(&T) -> Option<O>) -> Option<O>;
}

impl<T> VecExt<T> for Vec<T> {
	fn find_and_remove(&mut self, predicate: impl Fn(&T) -> bool) -> Option<T> {
		let index = self.iter().position(predicate);

		if let Some(index) = index {
			Some(self.remove(index))
		} else {
			None
		}
	}

	fn first_where<O>(&self, predicate: impl Fn(&T) -> Option<O>) -> Option<O> {
		for value in self.iter() {
			if let Some(value) = predicate(value) {
				return Some(value);
			}
		}

		None
	}
}
