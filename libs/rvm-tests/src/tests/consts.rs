pub trait Samples: Sized {
	fn samples() -> Vec<Self>;
}
impl Samples for i8 {
	fn samples() -> Vec<Self> {
		vec![i8::MIN, -5, -4, -3, -2 - 1, 0, -0, 1, 2, 3, 4, 5, i8::MAX]
	}
}
impl Samples for i16 {
	fn samples() -> Vec<Self> {
		let mut output: Vec<i16> = i8::samples().into_iter().map(|v| v as i16).collect();
		output.push(i16::MIN);
		output.push(i16::MAX);
		output
	}
}
impl Samples for i32 {
	fn samples() -> Vec<Self> {
		let mut output: Vec<i32> = i16::samples().into_iter().map(|v| v as i32).collect();
		output.push(i32::MIN);
		output.push(i32::MAX);
		output
	}
}
impl Samples for i64 {
	fn samples() -> Vec<Self> {
		let mut output: Vec<i64> = i32::samples().into_iter().map(|v| v as i64).collect();
		output.push(i64::MIN);
		output.push(i64::MAX);
		output
	}
}
impl Samples for f32 {
	fn samples() -> Vec<Self> {
		let mut output: Vec<f32> = i64::samples().into_iter().map(|v| v as f32).collect();
		output.push(f32::EPSILON);
		output.push(f32::MIN);
		output.push(f32::MIN_POSITIVE);
		output.push(f32::MAX);
		output.push(f32::NAN);
		output.push(f32::INFINITY);
		output.push(f32::NEG_INFINITY);
		output
	}
}

impl Samples for f64 {
	fn samples() -> Vec<Self> {
		let mut output: Vec<f64> = i64::samples().into_iter().map(|v| v as f64).collect();
		output.push(f64::EPSILON);
		output.push(f64::MIN);
		output.push(f64::MIN_POSITIVE);
		output.push(f64::MAX);
		output.push(f64::NAN);
		output.push(f64::INFINITY);
		output.push(f64::NEG_INFINITY);
		output
	}
}
