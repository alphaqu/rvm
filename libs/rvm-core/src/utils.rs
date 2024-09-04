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
