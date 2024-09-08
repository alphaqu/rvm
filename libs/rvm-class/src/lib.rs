#![feature(type_alias_impl_trait)]
mod instance;
mod loader;
mod source;

pub use instance::*;
pub use loader::*;
use rvm_core::{Id, ObjectType};
pub use source::*;
type ClassResolver<'a> = dyn FnMut(&ObjectType) -> eyre::Result<Id<Class>> + 'a;

//impl<V: > ClassResolver for fn(&ObjectType) -> eyre::Result<Id<Class>> {
//
//	fn resolve(&mut self, ty: &ObjectType) -> eyre::Result<Id<Class>> {
//		self(ty)
//	}
//}
#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn compile() {}
}
