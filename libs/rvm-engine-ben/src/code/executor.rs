use either::Either;
use eyre::{bail, Context, ContextCompat};
use std::panic::UnwindSafe;
use std::sync::Arc;
use tracing::{debug, info, trace};

use crate::code::{CallTask, Task};
use crate::thread::{BenCallStack, BenFrame, BenFrameMut, BenFrameTicket, FrameHeader};
use crate::value::StackValue;
use crate::{BenEngine, BenMethod};
use rvm_core::{Id, MethodAccessFlags, MethodDescriptor, ObjectType, Type};
use rvm_runtime::engine::Thread;
use rvm_runtime::gc::{GcMarker, GcRef, GcSweeper, JavaUser, RootProvider};
use rvm_runtime::native::{JNIFunction, JNIFunctionSignature};
use rvm_runtime::{AnyValue, CallType, MethodIdentifier, Reference, Runtime, ThreadContext, Vm};

/// The executor is where the java code actually executes.
pub struct Executor {
	pub call_stack: BenCallStack,
	pub sweeper: GcSweeper,
	pub thread: Thread,
	pub engine: Arc<BenEngine>,

	pub vm: Vm,
	pub java_scopes: Vec<JavaScope>,
	pub frozen_references: Vec<Reference>,
}

impl Executor {
	pub fn runtime(&mut self) -> Runtime<'_> {
		Runtime {
			vm: self.vm.clone(),
			thread: Some(self),
		}
	}

	pub fn current_frame(&mut self) -> BenFrameMut {
		let scope = self.java_scopes.last().unwrap();
		let ticket = &scope.frame_ticket;
		self.call_stack.get_mut(ticket)
	}

	pub fn continue_execution(&mut self) -> eyre::Result<Option<AnyValue>> {
		let to_scope = self.java_scopes.len();
		let mut returned = None;
		while let Some(ticket) = self.java_scopes.last()
			&& self.java_scopes.len() >= to_scope
		{
			let method = ticket.method.clone();
			let Some(method) = method.as_java() else {
				panic!("Method is not java.");
			};

			{
				let frame = *self.current_frame().header();
				let runtime = self.runtime();
				let class = runtime.classes.get(frame.class_id);
				let class = class.to_instance();
				let method = class.methods.get(frame.method_id);

				debug!(target: "exe", "class: {}", class.ty);
				debug!(target: "exe", "method: {}{}",method.name, method.desc.to_java() );
			}

			let mut first = true;
			loop {
				if first {
					// we do this because cursor++ is at different places.
					first = false;
				} else {
					//if self.runner.last_return.is_some() {
					// 			panic!("Return was never consumed by the caller.");
					// 		}
				}
				self.prepare_task();

				// not do this every instruction
				let mut frame = self.current_frame();

				let task = &method.tasks[frame.cursor];
				trace!(target: "exe", "s[{}] l[{}] {task}", frame.stack_values_debug(), frame.local_values_debug());

				match task {
					Task::New(object) => {
						let mut ctx = self.runtime();
						let id = ctx.resolve_class(&Type::Object(object.class_name.clone()))?;

						let class = ctx.vm.classes.get(id);
						let class = class.to_instance();

						let instance = ctx.alloc_object(class)?;

						frame = self.current_frame();
						frame.push(StackValue::Reference(*instance.raw()));
					}
					Task::Call(task) => {
						let returned = match returned.take() {
							Some(returned) => returned,
							None => {
								let frame_id = self.current_frame().method_id;
								let scope = self.push_scope(
									&task.object,
									&task.method,
									&task.method_descriptor,
									task.ty,
									None,
								)?;

								match scope {
									ScopeResult::ContinueJava => {
										break;
									}
									ScopeResult::Return(returned) => {
										frame = self.current_frame();
										assert_eq!(frame_id, frame.method_id);
										returned
									}
								}
							}
						};

						if let Some(value) = returned {
							frame.push(StackValue::from_any(value));
						}
						//match self.call_method(task)? {
						//	Either::Left(returned) => {
						//		frame = self.current_frame();
						//		if let Some(value) = returned {
						//			frame.push(StackValue::from_any(value));
						//		}
						//	}
						//	Either::Right(scope) => {
						//		// Go into a new scope
						//		self.scopes.push(scope);
						//		break;
						//	}
						//}
					}
					Task::Return(_return) => {
						let output = method.returns.map(|kind| {
							let value = frame.pop();
							value.convert(kind).unwrap()
						});

						returned = Some(output);

						// We pop our scope
						let scope = self.java_scopes.pop().unwrap();
						self.call_stack.pop(scope.frame_ticket);
						break;
					}
					Task::Nop => {}
					Task::Const(v) => {
						v.exec(self)?;
						frame = self.current_frame();
					}
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
					Task::Field(task) => {
						task.exec(self)?;
						frame = self.current_frame();
					}
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
					Task::ArrayCreate(v) => {
						v.exec(self)?;
						frame = self.current_frame();
					}
					Task::ArrayCreateRef(v) => {
						v.exec(self)?;
						frame = self.current_frame();
					}
					Task::Unsupported(v) => todo!("{v:?}"),
				};
				frame.cursor += 1;
			}
		}

		Ok(returned.expect("Last method never had a return op"))
	}
}

pub struct JavaScope {
	pub(crate) frame_ticket: BenFrameTicket,
	method: Arc<BenMethod>,
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
enum ScopeResult {
	ContinueJava,
	Return(Option<AnyValue>),
}

impl Executor {
	pub fn prepare_task(&mut self) {
		self.yield_gc();
	}

	//pub fn call_method(
	//	&mut self,
	//	task: &CallTask,
	//) -> eyre::Result<Either<Option<AnyValue>, JavaScope>> {
	//	// When we first call, the output will be None, it will push a frame onto the stack and start running that method.
	//	// When that method returns, it will set the output to Some(Option<Value>) and pop itself out of the stack.
	//	// We will come back here (because we never incremented the pointer) and see that our output is now Some.
	//	// We push that return value (if it exists) and continue running.
	//
	//	// TODO caller validation??
	//	Ok(match self.runner.last_return.take() {
	//		None => {
	//			let scope =
	//				self.push_scope(&task.object, &task.method, &task.method_descriptor, task.ty)?;
	//			match scope {
	//				ScopeResult::ContinueJava(java) => Either::Right(java),
	//				ScopeResult::Return(value) => Either::Left(value),
	//			}
	//		}
	//		Some(value) => Either::Left(value),
	//	})
	//}

	fn push_scope(
		&mut self,
		ty: &ObjectType,
		method_ident: &MethodIdentifier,
		method_descriptor: &MethodDescriptor,
		call_ty: CallType,
		ticket: Option<&BenFrameTicket>,
	) -> eyre::Result<ScopeResult> {
		trace!(target: "exe",  "Creating frame for {ty:?} {method_ident:?}");

		let ticket = ticket.unwrap_or_else(|| &self.java_scopes.last().unwrap().frame_ticket);
		//let desc = MethodDescriptor::parse(&method_ident.descriptor).wrap_err_with(|| {
		//	format!("Parsing method descriptor \"{}\"", method_ident.descriptor)
		//})?;
		let mut frame = self.call_stack.get_mut(ticket);
		let inputs = MethodInputs::flush_from(call_ty, method_descriptor, || frame.pop())
			.wrap_err_with(|| format!("Method inputs for {method_descriptor}"))?;

		let class_id = if call_ty.is_static() || call_ty.is_special() {
			self.runtime().resolve_class(&Type::Object(ty.clone()))?
		} else {
			let reference = inputs.instance.unwrap();
			let class_object = reference.to_instance()?;
			class_object.class()
		};

		let (method_class, method_id) = self
			.engine
			.resolve_method(&self.vm, class_id, method_ident)
			.wrap_err_with(|| {
				format!(
					"Could not resolve method \"{}{method_descriptor:?}\" error",
					method_ident.name
				)
			})?;

		let method = self
			.engine
			.compile_method(&self.vm, method_class, method_id);

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

				let scope = JavaScope {
					frame_ticket: scope.to_ticket(),
					method,
				};

				self.java_scopes.push(scope);
				ScopeResult::ContinueJava
			}
			BenMethod::Binding(binding) => ScopeResult::Return(
				binding
					.call(&self.vm, inputs.parameters)
					.wrap_err("Failed externally")?,
			),
			BenMethod::Native(native, desc) => {
				let mut linker = self.vm.linker.lock();
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
							&self.vm,
							&inputs.parameters,
							method_descriptor.returns.as_ref().map(|v| v.kind()),
						)
					})
					.wrap_err_with(|| {
						format!("Could not find native function link for {native}{desc:?}")
					})?;
				ScopeResult::Return(option)
			}
		})
	}
}

impl ThreadContext for Executor {
	fn yield_gc(&mut self) {
		GcSweeper::yield_gc(self);
	}

	fn wait_until_gc(&mut self) {
		GcSweeper::wait_until_gc(self);
	}

	fn run(
		&mut self,
		call_type: CallType,
		ty: &ObjectType,
		method: &MethodIdentifier,
		parameters: Vec<AnyValue>,
	) -> eyre::Result<Option<AnyValue>> {
		// Bootstrap
		info!("Starting execution with {parameters:?}");
		let mut guard = self
			.call_stack
			.push(
				parameters.len() as u16,
				0,
				FrameHeader {
					// TODO native tracing
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
			.push_scope(
				ty,
				method,
				&MethodDescriptor::parse(&method.descriptor).unwrap(),
				call_type,
				Some(&ticket),
			)
			.wrap_err("Creating bootstrapping scope")?;

		let return_value = match scope {
			ScopeResult::ContinueJava => self.continue_execution()?,
			ScopeResult::Return(value) => value,
		};

		self.call_stack.pop(ticket);
		Ok(return_value)

		//let mut scopes = match scope {
		// 			Scope::Java(frame) => {
		// 				vec![frame]
		// 			}
		// 			Scope::Return(value) => {
		// 				return Ok(value);
		// 			}
		// 		};
		//
		// 		while let Some(scope) = scopes.last_mut() {
		// 			match scope.run(&mut self) {
		// 				Ok(MethodScopeResult::MoveInto(new_scope)) => {
		// 					scopes.push(new_scope);
		// 				}
		// 				Ok(MethodScopeResult::Finish(value)) => {
		// 					let scope = scopes.pop().unwrap();
		// 					self.call_stack.pop(scope.frame_ticket);
		//
		// 					assert!(self.runner.last_return.is_none());
		// 					self.runner.last_return = Some(value);
		// 				}
		// 				Err(mut error) => {
		// 					// Unravel
		// 					for scope in scopes {
		// 						let frame = self.call_stack.get_mut(&scope.frame_ticket);
		// 						let class = self.runtime.classes.get(frame.class_id);
		// 						let method = class.as_instance().unwrap().methods.get(frame.method_id);
		//
		// 						error =
		// 							error.wrap_err(format!("In method: {}{:?}", method.name, method.desc));
		// 						self.call_stack.pop(scope.frame_ticket);
		// 					}
		// 					return Err(error);
		// 				}
		// 			}
		// 		}

		//Ok(self.runner.last_return.unwrap())
	}
}
impl RootProvider<JavaUser> for Executor {
	fn mark_roots(&mut self, visitor: GcMarker) {
		for reference in &self.frozen_references {
			visitor.mark(**reference);
		}
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
		for reference in &mut self.frozen_references {
			*reference = Reference::new(mapper(**reference));
		}
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
		&mut self.sweeper
	}
}
