pub struct JumpTask {
	offset: i32,
	kind: JumpKind,
}

impl JumpTask {}

#[derive(Copy, Clone, Debug)]
pub enum JumpKind {
	IF_ACMPEQ,
	IF_ACMPNE,
	IF_ICMPEQ,
	IF_ICMPNE,
	IF_ICMPLT,
	IF_ICMPGE,
	IF_ICMPGT,
	IF_ICMPLE,
	IFEQ,
	IFNE,
	IFLT,
	IFGE,
	IFGT,
	IFLE,
	IFNONNULL,
	IFNULL,
	GOTO,
}
