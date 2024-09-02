use rvm_runtime::{ClassLoader, Reference, Value};

use crate::args::Args;
use crate::ret::ReturnValue;

mod args;
mod ret;

pub unsafe trait JavaBinder {
	fn load_class(binder: &Binder, cl: &ClassLoader);
}

pub struct JavaField<V: Value> {}
#[cfg(test)]
mod tests {
	use rvm_core::{FieldAccessFlags, Kind, MethodDescriptor, ObjectType, Type};
	use rvm_runtime::{
		Array, Class, ClassMethodManager, FieldData, InstanceClass, Method, MethodCode,
		MethodIdentifier, ObjectFieldLayout,
	};

	use super::*;

	pub struct JavaString {
		hi: JavaField<Array<i8>>,
	}

	impl JavaString {
		pub fn hello(&mut self, things: i32) {}
	}

	unsafe impl JavaBinder for JavaString {
		fn load_class(binder: &Binder, cl: &ClassLoader) {
			let layout = ObjectFieldLayout::new(
				&[FieldData {
					name: "hi".to_string(),
					ty: Type::parse("[I").unwrap(),
					flags: FieldAccessFlags::PRIVATE,
				}],
				None,
				false,
			);
			static mut HI_FIELD: Option<(u32, Kind)> = None;
			unsafe {
				let field = layout.get_keyed("hi").unwrap();
				HI_FIELD = Some((field.offset, field.ty.kind()));
			}
			pub extern "C" fn hello(reference: Reference, things: i32) {
				let instance = reference.to_class().unwrap();
				unsafe {
					JavaString {
						hi: instance.get_any(HI_FIELD.unwrap().0, HI_FIELD.unwrap().1),
					}
					.hello(things);
				}
			}
			cl.define(Class::Object(InstanceClass {
				ty: ObjectType("java/lang/String".to_string()),
				super_class: None,
				super_id: None,
				cp: Default::default(),
				fields: ObjectFieldLayout::new(
					&[FieldData {
						name: "hi".to_string(),
						ty: Type::parse("[I").unwrap(),
						flags: FieldAccessFlags::PRIVATE,
					}],
					None,
					false,
				),
				static_fields: ObjectFieldLayout::new(&[], None, true),
				methods: ClassMethodManager::new(vec![Method {
					name: "hello".to_string(),
					desc: MethodDescriptor::parse("(Ljava/lang/String;)"),
					flags: (),
					code: Some(MethodCode::Binding(MethodIdentifier {
						name: "".to_string(),
						descriptor: "".to_string(),
					})),
				}]),
			}));
		}
	}

	#[test]
	fn test() {
		Binder {}.add_method("hi", |runtime, this, hi: i32| {});
	}
}
