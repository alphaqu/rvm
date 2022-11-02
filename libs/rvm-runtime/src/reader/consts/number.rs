#[derive(Copy, Clone, Debug)]
pub struct IntegerConst {
	pub bytes: i32,
}

#[derive(Copy, Clone, Debug)]
pub struct FloatConst {
	pub bytes: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct LongConst {
	pub bytes: i64,
}

#[derive(Copy, Clone, Debug)]
pub struct DoubleConst {
	pub bytes: f64,
}
