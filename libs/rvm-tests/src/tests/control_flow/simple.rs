use crate::bindings::tests::control_flow::Java;
use crate::launch;

#[test]
fn test() -> Result<(), std::io::Error> {
	let mut runtime = launch(128);

	for i in 0..8i32 {
		for j in 0..8i32 {
			assert_eq!(Java::pow(&mut runtime, i, j).unwrap(), i.pow(j as u32));
		}
	}

	Ok(())
}
