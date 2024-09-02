use crate::launch;
use rvm_runtime::bind;
use std::sync::Arc;
pub struct SwitchTest;
bind!("tests/switch_statement" {
	SwitchTest {
		testSwitch(i: i32) -> i32
	}
});

#[test]
fn basic_switch() {
	let runtime = launch(1024, vec!["tests/switch_statement/SwitchTest.class"]);
	let test_switch = SwitchTest::testSwitch(&runtime);

	assert_eq!(test_switch(0), 420);
	assert_eq!(test_switch(1), 3);
	assert_eq!(test_switch(2), 2);
	assert_eq!(test_switch(3), 420);
	assert_eq!(test_switch(4), 4);
	assert_eq!(test_switch(5), 420);
	assert_eq!(test_switch(10), 10);
}
