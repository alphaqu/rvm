use rvm_macro::java_desc;

#[test]
fn hi() {
	assert_eq!(java_desc!(i32), "I");
	assert_eq!(java_desc!(fn(i32) -> i32), "(I)I");
	assert_eq!(java_desc!(fn(i32, i32) -> i32), "(II)I");
	assert_eq!(java_desc!(fn(i32)), "(I)V");
}
