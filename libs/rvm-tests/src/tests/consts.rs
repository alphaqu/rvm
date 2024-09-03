pub trait Samples: Sized {
	fn samples() -> Vec<Self>;
}
impl Samples for i8 {
	fn samples() -> Vec<Self> {
		vec![i8::MIN, -1, 0, -0, 1, i8::MAX]
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
		vec![
			1.0,
			0.0,
			-1.0,
			f32::EPSILON,
			f32::MIN,
			f32::MIN_POSITIVE,
			f32::MAX,
			f32::NAN,
			f32::INFINITY,
			f32::NEG_INFINITY,
		]
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
