use crate::bindings::tests::switch_statement::SwitchTest;
use crate::launch;

#[test]
fn basic_switch() {
	let runtime = launch(1024);
	let test_switch = |v| SwitchTest::testSwitch(&runtime, v).unwrap();

	assert_eq!(test_switch(0), 420);
	assert_eq!(test_switch(1), 3);
	assert_eq!(test_switch(2), 2);
	assert_eq!(test_switch(3), 420);
	assert_eq!(test_switch(4), 4);
	assert_eq!(test_switch(5), 420);
	assert_eq!(test_switch(10), 10);
}
