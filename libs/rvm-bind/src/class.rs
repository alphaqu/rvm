use rvm_class::Class;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Arc;

pub struct ClassFile {
	pub data: Arc<Class>,
	pub full_binding: bool,
	//pub source: PathBuf,
}

impl Deref for ClassFile {
	type Target = Arc<Class>;

	fn deref(&self) -> &Self::Target {
		&self.data
	}
}
