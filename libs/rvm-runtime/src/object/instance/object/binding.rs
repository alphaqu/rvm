use crate::{AnyArray, AnyInstance, AnyValue, Class, Returnable, Runtime};
use rvm_core::{CastKindError, CastTypeError, Id, ObjectType, Type, Typed};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

pub trait InstanceBinding {
	fn ty() -> ObjectType;
	fn bind(instance: &AnyInstance) -> Self;
}

impl<B: InstanceBinding> Returnable for Instance<B> {
	fn from_value(runtime: &Arc<Runtime>, value: Option<AnyValue>) -> Self {
		let instance = AnyInstance::from_value(runtime, value);
		Instance::try_new(instance).unwrap()
	}
}

impl<B: InstanceBinding> From<Instance<B>> for AnyValue {
	fn from(value: Instance<B>) -> Self {
		value.instance.into()
	}
}

#[derive(Clone)]
pub struct Instance<B: InstanceBinding> {
	instance: AnyInstance,
	binding: B,
}

impl<B: InstanceBinding> Instance<B> {
	pub fn try_new(instance: AnyInstance) -> Result<Self, CastTypeError> {
		let target_class = instance
			.runtime
			.cl
			.get_named(&B::ty().into())
			.expect("Class is not loaded");

		if !instance.instance_of(target_class) {
			return Err(CastTypeError {
				expected: B::ty().into(),
				found: instance.class.cloned_ty(),
			});
		}

		Ok(Instance {
			binding: B::bind(&instance),
			instance,
		})
	}

	pub fn cast_to<T: InstanceBinding>(&self) -> Instance<T> {
		self.untyped().clone().typed::<T>()
	}

	pub fn class_id(&self) -> Id<Class> {
		self.instance.class_id()
	}

	pub fn untyped(&self) -> &AnyInstance {
		&self.instance
	}
}

impl<B: InstanceBinding> Deref for Instance<B> {
	type Target = B;

	fn deref(&self) -> &Self::Target {
		&self.binding
	}
}
impl<B: InstanceBinding> DerefMut for Instance<B> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.binding
	}
}

impl<B: InstanceBinding> Typed for Instance<B> {
	fn ty() -> Type {
		Type::Object(B::ty())
	}
}
