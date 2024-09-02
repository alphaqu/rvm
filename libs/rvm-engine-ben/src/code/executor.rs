use either::Either;
use eyre::{bail, Context, ContextCompat, Report};
use std::borrow::Cow;
use std::sync::Arc;
use tracing::{debug, info, trace, warn};

use rvm_core::{Id, Kind, MethodAccessFlags, MethodDescriptor, ObjectType, Type};
use rvm_reader::JumpKind;
use rvm_runtime::engine::Thread;
use rvm_runtime::gc::{AllocationError, GcMarker, GcSweeper, RootProvider};
use rvm_runtime::{
	AnyValue, Class, MethodBinding, MethodCode, MethodIdentifier, Reference, Runtime,
};

use crate::code::{CallTask, CallType, Task};
use crate::thread::{ThreadFrame, ThreadStack};
use crate::value::StackValue;
use crate::{BenEngine, BenMethod};

/// The executor is where the java code actually executes.
pub struct Executor<'a> {
	pub thread: Thread,
	pub stack: &'a mut ThreadStack,
	pub engine: Arc<BenEngine>,
	pub runtime: Arc<Runtime>,

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

impl<'a> Executor<'a> {
	pub fn prepare_task(&mut self) {
		GcSweeper::yield_gc(self);
	}

	pub fn finalize_task(&mut self) {
		if self.runner.last_return.is_some() {
			panic!("Return was never consumed by the caller.");
		}
	}

	pub fn call_method<'f>(
		&mut self,
		task: &CallTask,
		parameter_getter: impl FnMut() -> StackValue,
	) -> eyre::Result<Either<Option<AnyValue>, JavaScope<'f>>> {
		// When we first call, the output will be None, it will push a frame onto the stack and start running that method.
		// When that method returns, it will set the output to Some(Option<Value>) and pop itself out of the stack.
		// We will come back here (because we never incremented the pointer) and see that our output is now Some.
		// We push that return value (if it exists) and continue running.

		// TODO caller validation??
		Ok(match self.runner.last_return.take() {
			None => {
				let scope =
					self.create_scope(&task.object, &task.method, task.ty, parameter_getter)?;
				match scope {
					Scope::Java(java) => Either::Right(java),
					Scope::Return(value) => Either::Left(value),
				}
			}
			Some(value) => Either::Left(value),
		})
	}
	pub fn new_object(&mut self, id: Id<Class>) -> eyre::Result<Option<Reference>> {
		let class = self.runtime.cl.get(id);
		let object = class.as_instance().unwrap();
		let result = self.runtime.gc.lock().allocate_instance(id, object);

		match result {
			Ok(object) => {
				self.runner.gc_attempts = 0;
				return Ok(Some(*object));
			}
			Err(AllocationError::OutOfHeap) => {
				self.runner.gc_attempts += 1;
				if self.runner.gc_attempts > 5 {
					bail!(AllocationError::OutOfHeap);
				}
				Runtime::gc(self.runtime.clone());
				GcSweeper::wait_until_gc(self);
				trace!("Forcing gc, and trying again.");
				// try this instruction again, if we fail 5 time, we blow up.
				return Ok(None);
			}
			Err(error) => {
				bail!(error);
			}
		}
	}
}
pub enum MethodScopeResult<'f> {
	MoveInto(JavaScope<'f>),
	Finish(Option<AnyValue>),
}
pub struct JavaScope<'f> {
	pub(crate) frame: ThreadFrame<'f>,
	method: Arc<BenMethod>,
	name: String,
	desc: MethodDescriptor,
	pub(crate) cursor: usize,
}

impl<'f> JavaScope<'f> {
	pub fn run(&mut self, executor: &mut Executor) -> eyre::Result<MethodScopeResult<'f>> {
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

			let frame = &mut self.frame;

			let task = &method.tasks[self.cursor];
			trace!(target: "exe", "s[{}] l[{}] {task}", frame.stack_values_debug(), frame.local_values_debug());

			match task {
				Task::New(object) => {
					let id = executor
						.runtime
						.cl
						.resolve_class(&Type::Object(object.class_name.clone()));

					match executor.new_object(id).unwrap() {
						Some(object) => {
							frame.push(StackValue::Reference(object));
						}
						None => {
							// Retry
							continue;
						}
					}
				}
				Task::Call(task) => {
					// Details about how this works is in [executor.call_method]!!!
					match executor.call_method(task, || frame.pop())? {
						Either::Left(returned) => {
							if let Some(value) = returned {
								frame.push(StackValue::from_any(value));
							}
						}
						Either::Right(scope) => {
							return Ok(MethodScopeResult::MoveInto(scope));
						}
					}
				}
				Task::Return(v) => {
					let output = method.returns.map(|kind| {
						let value = frame.pop();
						value.convert(kind).unwrap()
					});

					return Ok(MethodScopeResult::Finish(output));
				}
				Task::Nop => {}
				Task::Const(v) => v.exec(frame),
				Task::Combine(v) => v.exec(frame),
				Task::Local(v) => v.exec(frame),
				Task::Jump(task) => {
					task.exec(self);
					continue;
				}
				Task::SwitchTable(v) => {
					let offset = v.exec(frame);
					self.cursor = self.cursor.checked_add_signed(offset as isize).unwrap();
					continue;
				}
				Task::Stack(task) => task.exec(frame),
				Task::Field(task) => task.exec(&executor.runtime, frame),
				Task::Increment(task) => {
					let value = frame.load(task.local);
					frame.store(
						task.local,
						StackValue::Int(value.to_int().unwrap() + task.increment as i32),
					);
				}
				Task::ArrayLength(v) => v.exec(frame),
				Task::ArrayLoad(v) => v.exec(frame),
				Task::ArrayStore(v) => v.exec(frame),
				Task::ArrayCreate(v) => v.exec(&executor.runtime, frame),
				Task::ArrayCreateRef(v) => v.exec(&executor.runtime, frame),
			};
			self.cursor += 1;
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
		let mut parameters = Vec::new();
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
enum Scope<'f> {
	Java(JavaScope<'f>),
	Return(Option<AnyValue>),
}

impl<'a> Executor<'a> {
	fn create_scope<'f>(
		&mut self,
		ty: &ObjectType,
		method_ident: &MethodIdentifier,
		call_ty: CallType,
		parameter_getter: impl FnMut() -> StackValue,
	) -> eyre::Result<Scope<'f>> {
		debug!("Creating frame for {ty:?} {method_ident:?}");

		let desc = MethodDescriptor::parse(&method_ident.descriptor).wrap_err_with(|| {
			format!("Parsing method descriptor \"{}\"", method_ident.descriptor)
		})?;
		let inputs = MethodInputs::flush_from(call_ty, &desc, parameter_getter)
			.wrap_err_with(|| format!("Method inputs for {desc}"))?;

		let class_id = if call_ty.is_static() || call_ty.is_special() {
			self.runtime.cl.resolve_class(&Type::Object(ty.clone()))
		} else {
			let reference = inputs.instance.unwrap();
			let class_object = reference.to_class().unwrap();
			class_object.class()
		};

		let (method_class, method_id) = self
			.engine
			.resolve_method(&self.runtime, class_id, method_ident)
			.wrap_err_with(|| {
				format!(
					"Could not resolve method \"{}{desc:?}\" error",
					method_ident.name
				)
			})?;

		let method = self
			.engine
			.compile_method(&self.runtime, method_class, method_id);

		let returns = desc.returns.as_ref().map(|v| v.kind());
		Ok(match &*method {
			BenMethod::Java(java) => {
				let is_method_static = java.flags.contains(MethodAccessFlags::STATIC);
				if call_ty.is_static() != is_method_static {
					bail!(
						"Method invocation ({call_ty:?}) is not compatible with {} method \"{}{desc:?}\"",
						if is_method_static {
							"static"
						} else {
							"non-static"
						},
						method_ident.name
					);
				}
				assert!(java.max_locals as usize >= inputs.parameters.len());
				let mut frame = self.stack.create(java.max_stack, java.max_locals);

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
					frame,
					method,
					name: method_ident.name.clone(),
					desc,
					cursor: 0,
				})
			}
			BenMethod::Binding(binding) => {
				Scope::Return(binding.call(&self.runtime, &inputs.parameters, returns))
			}
			BenMethod::Native(native, desc) => {
				let mut linker = self.runtime.linker.lock();
				Scope::Return(linker.get(native, |function| unsafe {
					trace!("Calling native function");

					let binding = match function {
						Either::Left(left) => Cow::Owned(MethodBinding::new(left, desc.clone())),
						Either::Right(right) => Cow::Borrowed(right),
					};

					binding.call(&self.runtime, &inputs.parameters, returns)
				}))
			}
		})
	}
	pub fn execute(
		mut self,
		ty: &ObjectType,
		method: &MethodIdentifier,
		mut parameters: Vec<AnyValue>,
	) -> eyre::Result<Option<AnyValue>> {
		info!("Starting execution with {parameters:?}");
		let scope = self
			.create_scope(ty, method, CallType::Static, || {
				StackValue::from_any(parameters.pop().expect("Not enough parameters"))
			})
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
					self.stack.finish(scope.frame);

					assert!(self.runner.last_return.is_none());
					self.runner.last_return = Some(value);
				}
				Err(mut error) => {
					// Unravel
					for scope in scopes {
						error =
							error.wrap_err(format!("In method: {}{:?}", scope.name, scope.desc));
						self.stack.finish(scope.frame);
					}
					return Err(error);
				}
			}
		}

		Ok(self.runner.last_return.unwrap())
	}
}

impl<'a> RootProvider for Executor<'a> {
	fn mark_roots(&mut self, visitor: GcMarker) {
		self.stack.visit_frames_mut(|frame| {
			for i in 0..frame.stack_pos {
				if let StackValue::Reference(reference) = frame.get_stack_value(i) {
					visitor.mark(reference);
				}
			}

			for i in 0..frame.local_size {
				if let StackValue::Reference(reference) = frame.load(i) {
					visitor.mark(reference);
				}
			}
		});
	}

	fn remap_roots(&mut self, mut mapper: impl FnMut(Reference) -> Reference) {
		self.stack.visit_frames_mut(|frame| {
			for i in 0..frame.stack_pos {
				if let StackValue::Reference(reference) = frame.get_stack_value(i) {
					frame.set_stack_value(i, StackValue::Reference(mapper(reference)));
				}
			}

			for i in 0..frame.local_size {
				if let StackValue::Reference(reference) = frame.load(i) {
					frame.store(i, StackValue::Reference(reference));
				}
			}
		});
	}

	fn sweeper(&mut self) -> &mut GcSweeper {
		&mut self.thread.gc
	}
}
