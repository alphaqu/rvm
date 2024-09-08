use either::Either;
use eyre::{bail, Context, ContextCompat};
use std::sync::Arc;
use tracing::{info, trace};

use crate::code::{CallTask, CallType, Task};
use crate::thread::{BenCallStack, BenFrameTicket, FrameHeader};
use crate::value::StackValue;
use crate::{BenEngine, BenMethod};
use rvm_core::{Id, MethodAccessFlags, MethodDescriptor, ObjectType, Type};
use rvm_runtime::engine::Thread;
use rvm_runtime::gc::{AllocationError, GcMarker, GcRef, GcSweeper, JavaUser, RootProvider};
use rvm_runtime::native::{JNIFunction, JNIFunctionSignature};
use rvm_runtime::{AnyValue, Class, MethodIdentifier, Reference, Runtime};

/// The executor is where the java code actually executes.
pub struct Executor<'d> {
	pub call_stack: BenCallStack<'d>,
	pub thread: Thread,
	pub engine: Arc<BenEngine>,

	pub runtime: Runtime,
	pub runner: ExecutorRunner,
}

pub struct ExecutorRunner {
	pub gc_attempts: u32,
	// this is Some when a scope has just finished and returned a value.
	// This will hard crash if the next instruction does not pop take this value.
	pub last_return: Option<Option<AnyValue>>,
}

impl ExecutorRunner {
	pub fn new() -> ExecutorRunner {
		ExecutorRunner {
			gc_attempts: 0,
			last_return: None,
		}
	}
}

pub enum MethodScopeResult {
	MoveInto(JavaScope),
	Finish(Option<AnyValue>),
}
pub struct JavaScope {
	pub(crate) frame_ticket: BenFrameTicket,
	method: Arc<BenMethod>,
}

impl JavaScope {
	pub fn run(&mut self, executor: &mut Executor) -> eyre::Result<MethodScopeResult> {
		let method = self.method.clone();
		let Some(method) = method.as_java() else {
			panic!("Method is not java.");
		};

		let mut first = true;
		loop {
			if first {
				// we do this because cursor++ is at different places.
				first = false;
			} else {
				executor.finalize_task();
			}
			executor.prepare_task();

			let mut frame = executor.call_stack.get_mut(&self.frame_ticket);
			let task = &method.tasks[frame.cursor];
			trace!(target: "exe", "s[{}] l[{}] {task}", frame.stack_values_debug(), frame.local_values_debug());

			match task {
				Task::New(object) => {
					let id = executor
						.runtime
						.resolve_class(&Type::Object(object.class_name.clone()))?;

					let instance = executor.runtime.alloc_object(id)?;
					frame.push(StackValue::Reference(*instance.raw()));
				}
				Task::Call(task) => {
					// Details about how this works is in [executor.call_method]!!!
					match executor.call_method(task, &self.frame_ticket)? {
						Either::Left(returned) => {
							frame = executor.call_stack.get_mut(&self.frame_ticket);
							if let Some(value) = returned {
								frame.push(StackValue::from_any(value));
							}
						}
						Either::Right(scope) => {
							return Ok(MethodScopeResult::MoveInto(scope));
						}
					}
				}
				Task::Return(_return) => {
					let output = method.returns.map(|kind| {
						let value = frame.pop();
						value.convert(kind).unwrap()
					});

					return Ok(MethodScopeResult::Finish(output));
				}
				Task::Nop => {}
				Task::Const(v) => v.exec(&mut frame),
				Task::Combine(v) => v
					.exec(&mut frame)
					.wrap_err_with(|| format!("Combine {}", v))?,
				Task::Local(v) => v.exec(&mut frame),
				Task::Jump(task) => {
					task.exec(&mut frame);
					continue;
				}
				Task::SwitchTable(v) => {
					let offset = v.exec(&mut frame);
					frame.cursor = frame.cursor.checked_add_signed(offset as isize).unwrap();
					continue;
				}
				Task::Stack(task) => task.exec(&mut frame),
				Task::Field(task) => task.exec(&executor.runtime, &mut frame)?,
				Task::Increment(task) => {
					let value = frame.load(task.local);
					frame.store(
						task.local,
						StackValue::Int(value.to_int()? + task.increment as i32),
					);
				}
				Task::ArrayLength(v) => v.exec(&mut frame),
				Task::ArrayLoad(v) => v.exec(&mut frame),
				Task::ArrayStore(v) => v.exec(&mut frame)?,
				Task::ArrayCreate(v) => v.exec(&executor.runtime, &mut frame)?,
				Task::ArrayCreateRef(v) => v.exec(&executor.runtime, &mut frame)?,
			};
			frame.cursor += 1;
		}
	}
}
pub struct MethodInputs {
	instance: Option<Reference>,
	parameters: Vec<AnyValue>,
}

impl MethodInputs {
	pub fn flush_from(
		ty: CallType,
		desc: &MethodDescriptor,
		mut parameter_getter: impl FnMut() -> StackValue,
	) -> eyre::Result<MethodInputs> {
		let mut parameters = Vec::with_capacity(desc.parameters.len());
		for (i, v) in desc.parameters.iter().enumerate().rev() {
			let stack_value = parameter_getter();
			let kind = v.kind();
			let value = stack_value
				.convert(kind)
				.wrap_err_with(|| format!("Parameter {i} {v}"))?;
			parameters.push(value);
		}
		parameters.reverse();

		let mut instance: Option<Reference> = None;
		if !ty.is_static() {
			let stack_value = parameter_getter();
			instance = Some(stack_value.to_ref().wrap_err("Instance")?);
		}

		Ok(MethodInputs {
			instance,
			parameters,
		})
	}
}
enum Scope {
	Java(JavaScope),
	Return(Option<AnyValue>),
}

impl<'d> Executor<'d> {
	pub fn prepare_task(&mut self) {
		GcSweeper::yield_gc(self);
	}

	pub fn finalize_task(&mut self) {
		if self.runner.last_return.is_some() {
			panic!("Return was never consumed by the caller.");
		}
	}

	pub fn call_method(
		&mut self,
		task: &CallTask,
		frame: &BenFrameTicket,
	) -> eyre::Result<Either<Option<AnyValue>, JavaScope>> {
		// When we first call, the output will be None, it will push a frame onto the stack and start running that method.
		// When that method returns, it will set the output to Some(Option<Value>) and pop itself out of the stack.
		// We will come back here (because we never incremented the pointer) and see that our output is now Some.
		// We push that return value (if it exists) and continue running.

		// TODO caller validation??
		Ok(match self.runner.last_return.take() {
			None => {
				let scope = self.create_scope(
					&task.object,
					&task.method,
					&task.method_descriptor,
					task.ty,
					frame,
				)?;
				match scope {
					Scope::Java(java) => Either::Right(java),
					Scope::Return(value) => Either::Left(value),
				}
			}
			Some(value) => Either::Left(value),
		})
	}
	pub fn new_object(&mut self, id: Id<Class>) -> eyre::Result<Option<Reference>> {
		let class = self.runtime.classes.get(id);
		let object = class.as_instance().unwrap();
		let result = self.runtime.gc.alloc_instance(object);

		match result {
			Ok(object) => {
				self.runner.gc_attempts = 0;
				Ok(Some(*object))
			}
			Err(AllocationError::OutOfHeap) => {
				self.runner.gc_attempts += 1;
				if self.runner.gc_attempts > 5 {
					bail!(AllocationError::OutOfHeap);
				}
				self.runtime.gc();
				GcSweeper::wait_until_gc(self);
				trace!("Forcing gc, and trying again.");
				// try this instruction again, if we fail 5 time, we blow up.
				Ok(None)
			}
			Err(error) => {
				bail!(error);
			}
		}
	}
	fn create_scope(
		&mut self,
		ty: &ObjectType,
		method_ident: &MethodIdentifier,
		method_descriptor: &MethodDescriptor,
		call_ty: CallType,
		frame: &BenFrameTicket,
	) -> eyre::Result<Scope> {
		trace!(target: "exe",  "Creating frame for {ty:?} {method_ident:?}");

		//let desc = MethodDescriptor::parse(&method_ident.descriptor).wrap_err_with(|| {
		//	format!("Parsing method descriptor \"{}\"", method_ident.descriptor)
		//})?;
		let mut frame = self.call_stack.get_mut(frame);
		let inputs = MethodInputs::flush_from(call_ty, method_descriptor, || frame.pop())
			.wrap_err_with(|| format!("Method inputs for {method_descriptor}"))?;

		let class_id = if call_ty.is_static() || call_ty.is_special() {
			self.runtime.resolve_class(&Type::Object(ty.clone()))?
		} else {
			let reference = inputs.instance.unwrap();
			let class_object = reference.to_instance().unwrap();
			class_object.class()
		};

		let (method_class, method_id) = self
			.engine
			.resolve_method(&self.runtime, class_id, method_ident)
			.wrap_err_with(|| {
				format!(
					"Could not resolve method \"{}{method_descriptor:?}\" error",
					method_ident.name
				)
			})?;

		let method = self
			.engine
			.compile_method(&self.runtime, method_class, method_id);

		Ok(match &*method {
			BenMethod::Java(java) => {
				let is_method_static = java.flags.contains(MethodAccessFlags::STATIC);
				if call_ty.is_static() != is_method_static {
					bail!(
						"Method invocation ({call_ty:?}) is not compatible with {} method \"{}{method_descriptor:?}\"",
						if is_method_static {
							"statics"
						} else {
							"non-statics"
						},
						method_ident.name
					);
				}
				assert!(java.max_locals as usize >= inputs.parameters.len());

				let mut scope = self
					.call_stack
					.push(
						java.max_stack,
						java.max_locals,
						FrameHeader {
							class_id,
							method_id,
							cursor: 0,
						},
					)
					.unwrap();
				let mut frame = scope.frame_mut();
				let mut i = inputs.instance.is_some() as u8 as u16;
				for value in inputs.parameters.into_iter() {
					let local_size = value.kind().local_size();
					frame.store(i, StackValue::from_any(value));
					i += local_size as u16;
				}

				if let Some(value) = inputs.instance {
					frame.store(0, value.into());
				}

				Scope::Java(JavaScope {
					frame_ticket: scope.to_ticket(),
					method,
				})
			}
			BenMethod::Binding(binding) => Scope::Return(
				binding
					.call(&self.runtime, inputs.parameters)
					.wrap_err("Failed externally")?,
			),
			BenMethod::Native(native, desc) => {
				let mut linker = self.runtime.linker.lock();
				let option = linker
					.get(native, |function| unsafe {
						trace!("Calling native function");

						let desc = desc.clone();

						let Either::Left(value) = function else {
							panic!("Cringe");
						};
						let jni_function = JNIFunction::new(
							value,
							JNIFunctionSignature {
								parameters: desc.parameters.iter().map(|v| v.kind()).collect(),
								returns: desc.returns.map(|v| v.kind()),
							},
						);

						jni_function.call(
							&self.runtime,
							&inputs.parameters,
							method_descriptor.returns.as_ref().map(|v| v.kind()),
						)
					})
					.wrap_err_with(|| {
						format!("Could not find native function link for {native}{desc:?}")
					})?;
				Scope::Return(option)
			}
		})
	}
	pub fn execute(
		mut self,
		ty: &ObjectType,
		method: &MethodIdentifier,
		mut parameters: Vec<AnyValue>,
	) -> eyre::Result<Option<AnyValue>> {
		// Bootstrap
		info!("Starting execution with {parameters:?}");
		let mut guard = self
			.call_stack
			.push(
				parameters.len() as u16,
				0,
				FrameHeader {
					class_id: Id::null(),
					method_id: Id::null(),
					cursor: usize::MAX,
				},
			)
			.unwrap();

		let mut frame = guard.frame_mut();

		for value in parameters.into_iter() {
			frame.push(StackValue::from_any(value));
		}

		let ticket = guard.to_ticket();

		let scope = self
			.create_scope(
				ty,
				method,
				&MethodDescriptor::parse(&method.descriptor).unwrap(),
				CallType::Static,
				&ticket,
			)
			.wrap_err("Creating bootstrapping scope")?;

		let mut scopes = match scope {
			Scope::Java(frame) => {
				vec![frame]
			}
			Scope::Return(value) => {
				return Ok(value);
			}
		};

		while let Some(scope) = scopes.last_mut() {
			match scope.run(&mut self) {
				Ok(MethodScopeResult::MoveInto(new_scope)) => {
					scopes.push(new_scope);
				}
				Ok(MethodScopeResult::Finish(value)) => {
					let scope = scopes.pop().unwrap();
					self.call_stack.pop(scope.frame_ticket);

					assert!(self.runner.last_return.is_none());
					self.runner.last_return = Some(value);
				}
				Err(mut error) => {
					// Unravel
					for scope in scopes {
						let frame = self.call_stack.get_mut(&scope.frame_ticket);
						let class = self.runtime.classes.get(frame.class_id);
						let method = class.as_instance().unwrap().methods.get(frame.method_id);

						error =
							error.wrap_err(format!("In method: {}{:?}", method.name, method.desc));
						self.call_stack.pop(scope.frame_ticket);
					}
					return Err(error);
				}
			}
		}

		Ok(self.runner.last_return.unwrap())
	}
}

impl<'d> RootProvider<JavaUser> for Executor<'d> {
	fn mark_roots(&mut self, visitor: GcMarker) {
		for frame in self.call_stack.iter() {
			for value in frame.stack_slice() {
				if let StackValue::Reference(reference) = value {
					visitor.mark(**reference);
				}
			}

			for value in frame.local_slice() {
				if let StackValue::Reference(reference) = value {
					visitor.mark(**reference);
				}
			}
		}
	}

	fn remap_roots(&mut self, mut mapper: impl FnMut(GcRef) -> GcRef) {
		for mut frame in self.call_stack.iter_mut() {
			for value in frame.stack_slice_mut() {
				if let StackValue::Reference(reference) = value {
					*reference = Reference::new(mapper(**reference));
				}
			}

			for value in frame.local_slice_mut() {
				if let StackValue::Reference(reference) = value {
					*reference = Reference::new(mapper(**reference));
				}
			}
		}
	}

	fn sweeper(&mut self) -> &mut GcSweeper {
		&mut self.thread.gc
	}
}
