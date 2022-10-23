mod frame;
mod instruction;
mod locals;
mod stack;

pub use frame::Frame;
pub use instruction::BranchOffset;
pub use instruction::Inst;
pub use instruction::WideBranchOffset;
pub use locals::LocalCast;
pub use locals::LocalVar;
pub use locals::LocalVariables;
pub use stack::Stack;
pub use stack::StackCast;
pub use stack::StackValue;
pub use stack::StackValueType;
//
// use anyways::ext::AuditExt;
// use tracing::trace;
// use rvm_core::Id;
// use crate::reader::{ConstantPool, ConstPtr, Instruction, MethodConst};
// use crate::{Class, JResult, LocalTable, Method, MethodCode, Object, Runtime, Stack, StackValue, TraceEntry};
// use std::ops::{Add, Div, Mul, Rem, Shl, Shr, Sub};
//
// pub struct Executor {
// 	pub this_class: Id<Class>,
// 	pub this_method: Id<Method>,
//
// 	pub l: LocalTable,
// }
//
// impl Executor {
// 	pub fn resolve(method: ConstPtr<MethodConst>, cp: &ConstantPool, runtime: &Runtime) -> JResult<Executor> {
// 		let (this_class, this_method) = runtime.resolve_method(method, cp)?;
// 		Ok(Executor::new(this_class, this_method, runtime))
// 	}
//
// 	pub fn new(this_class: Id<Class>, this_method: Id<Method>, runtime: &Runtime) -> Executor {
// 		let class = runtime.class_loader.get(this_class);
// 		let method = class.methods.get(this_method);
//
// 		Executor {
// 			this_class,
// 			this_method,
// 			l: LocalTable::new(method.max_locals),
// 		}
// 	}
//
// 	pub fn run(&mut self, runtime: &Runtime) -> JResult<Option<StackValue>> {
// 		macro_rules! manip2 {
//             ($S:ident, $TY:path, $METHOD:ident) => {{
//                 let v0 = $S.pop();
//                 let v1 = $S.pop();
//                 if let ($TY(v0), $TY(v1)) = (v0, v1) {
//                     $S.push($TY(v0.$METHOD(v1)));
//                 } else {
//                     //  panic!("Cannot do {} on {v0:?} and {v1:?}", stringify!($TY));
//                     panic!("your mom blew up")
//                 }
//             }};
//         }
//
// 		let c = runtime.class_loader.get(self.this_class);
// 		let m = c.methods.get(self.this_method);
//
// 		match &m.code {
// 			Some(MethodCode::Native(func)) => {
// 				return (func.func)(&mut self.l, runtime);
// 			}
// 			Some(MethodCode::JVM(code)) => {
// 				let mut s = Stack::new(code.max_stack);
// 				for insn in &code.code {
// 					trace!(target: "exec", "{:?} => {:?} | {:?}", &insn.inst, s, self.l);
// 					match &insn.inst {
// 						Instruction::NOP => {
// 							// pray
// 						}
// 						Instruction::ACONST_NULL => s.push(StackValue::Object(Object::null())),
// 						Instruction::ICONST_M1 => s.push(StackValue::Int(-1)),
// 						Instruction::ICONST_0 => s.push(StackValue::Int(0)),
// 						Instruction::ICONST_1 => s.push(StackValue::Int(1)),
// 						Instruction::ICONST_2 => s.push(StackValue::Int(2)),
// 						Instruction::ICONST_3 => s.push(StackValue::Int(3)),
// 						Instruction::ICONST_4 => s.push(StackValue::Int(4)),
// 						Instruction::ICONST_5 => s.push(StackValue::Int(5)),
// 						Instruction::LCONST_0 => s.push(StackValue::Long(0)),
// 						Instruction::LCONST_1 => s.push(StackValue::Long(1)),
// 						Instruction::FCONST_0 => s.push(StackValue::Float(0.0)),
// 						Instruction::FCONST_1 => s.push(StackValue::Float(1.0)),
// 						Instruction::FCONST_2 => s.push(StackValue::Float(2.0)),
// 						Instruction::DCONST_0 => s.push(StackValue::Double(0.0)),
// 						Instruction::DCONST_1 => s.push(StackValue::Double(1.0)),
// 						Instruction::POP => {
// 							s.pop();
// 						}
// 						Instruction::POP2 => {
// 							s.pop();
// 							s.pop();
// 						}
// 						Instruction::DUP => {
// 							let value = s.pop();
// 							s.push(value.clone());
// 							s.push(value);
// 						}
//
// 						Instruction::IADD => manip2!(s, StackValue::Int, add),
// 						Instruction::LADD => manip2!(s, StackValue::Long, add),
// 						Instruction::FADD => manip2!(s, StackValue::Float, add),
// 						Instruction::DADD => manip2!(s, StackValue::Double, add),
// 						Instruction::ISUB => manip2!(s, StackValue::Int, sub),
// 						Instruction::LSUB => manip2!(s, StackValue::Long, sub),
// 						Instruction::FSUB => manip2!(s, StackValue::Float, sub),
// 						Instruction::DSUB => manip2!(s, StackValue::Double, sub),
// 						Instruction::IMUL => manip2!(s, StackValue::Int, mul),
// 						Instruction::LMUL => manip2!(s, StackValue::Long, mul),
// 						Instruction::FMUL => manip2!(s, StackValue::Float, mul),
// 						Instruction::DMUL => manip2!(s, StackValue::Double, mul),
// 						Instruction::IDIV => manip2!(s, StackValue::Int, div),
// 						Instruction::LDIV => manip2!(s, StackValue::Long, div),
// 						Instruction::FDIV => manip2!(s, StackValue::Float, div),
// 						Instruction::DDIV => manip2!(s, StackValue::Double, div),
// 						Instruction::IREM => manip2!(s, StackValue::Int, rem),
// 						Instruction::LREM => manip2!(s, StackValue::Long, rem),
// 						Instruction::FREM => manip2!(s, StackValue::Float, rem),
// 						Instruction::DREM => manip2!(s, StackValue::Double, rem),
// 						Instruction::ISHL => manip2!(s, StackValue::Int, shl),
// 						Instruction::LSHL => manip2!(s, StackValue::Long, shl),
// 						Instruction::ISHR => manip2!(s, StackValue::Int, shr),
// 						Instruction::LSHR => manip2!(s, StackValue::Long, shr),
// 						Instruction::PushByte { value } => {
// 							s.push(StackValue::Int(*value as i32))
// 						}
// 						Instruction::PushShort { value } => {
// 							s.push(StackValue::Int(*value as i32))
// 						}
// 						Instruction::Increment { var, amount } => {
// 							if let Value::Int(i) = self.l.get(*var) {
// 								self.l.set(*var, Value::Int(i + *amount as i32))
// 							} else {
// 								panic!("oh no");
// 							}
// 						}
// 						Instruction::Load { var } => {
// 							s.push(StackValue::from(self.l.get(*var)))
// 						}
// 						Instruction::Store { var } => {
// 							self.l.set(*var, Value::from(s.pop()))
// 						}
// 						Instruction::InvokeStatic { method } => {
// 							let mut executor = Executor::resolve(*method, &c.cp, runtime)?;
//
// 							let class = runtime.class_loader.get(executor.this_class);
// 							let method = class.methods.get(executor.this_method);
//
// 							for (i, _) in method.desc.parameters.iter().enumerate() {
// 								// TODO check if parameters match
// 								executor.l.set(i as u16, Value::from(s.pop()))
// 							}
// 							if let Some(value) = self.handle_err(executor.run(runtime))? {
// 								s.push(value);
// 							}
// 						}
// 						Instruction::InvokeSpecial { method } => {
// 							let mut executor = Executor::resolve(*method, &c.cp, runtime)?;
// 							let class = runtime.class_loader.get(executor.this_class);
// 							let method = class.methods.get(executor.this_method);
//
// 							executor.l.set(0, Value::from(s.pop()));
// 							for (i, _) in method.desc.parameters.iter().enumerate() {
// 								// TODO check if parameters match
// 								executor.l.set((i + 1) as u16, Value::from(s.pop()))
// 							}
//
// 							if let Some(value) = self.handle_err(executor.run(runtime))? {
// 								s.push(value);
// 							}
// 						}
// 						Instruction::New { class } => {
// 							let class = runtime.resolve_class(*class, &c.cp)?;
// 							let mut guard = runtime.gc.write().unwrap();
//
// 							let object = guard.alloc(class, &runtime.class_loader);
// 							s.push(StackValue::Object(object));
// 						}
// 						Instruction::IRETURN
// 						| Instruction::LRETURN
// 						| Instruction::FRETURN
// 						| Instruction::DRETURN
// 						| Instruction::ARETURN => {
// 							return Ok(Some(s.pop()));
// 						}
// 						Instruction::RETURN => {
// 							return Ok(None);
// 						}
// 						Instruction::PutStaticField { field } => {
// 							let (class_id, field) = runtime.resolve_field(*field, &c.cp)?;
//
// 							let class = runtime.class_loader.get(class_id);
// 							let field = class.static_fields.get(field);
// 							class.static_object.set_field(field, Value::from(s.pop()));
// 						}
// 						Instruction::GetStaticField { field } => {
// 							let (class_id, field) = runtime.resolve_field(*field, &c.cp)?;
//
// 							let class = runtime.class_loader.get(class_id);
// 							let field = class.static_fields.get(field);
// 							s.push(StackValue::from(class.static_object.get_field(field)));
// 						}
// 						Instruction::PutField { field } => {
// 							if let StackValue::Object(object) = s.pop() {
// 								let (class_id, field) = runtime.resolve_field(*field, &c.cp)?;
//
// 								let class = runtime.class_loader.get(class_id);
// 								let field = class.fields.get(field);
//
// 								object.assert_matching_class(class_id, runtime)?;
// 								object.set_field(field, Value::from(s.pop()));
// 							}
// 						}
// 						Instruction::GetField { field } => {
// 							if let StackValue::Object(object) = s.pop() {
// 								let (class_id, field) = runtime.resolve_field(*field, &c.cp)?;
//
// 								let class = runtime.class_loader.get(class_id);
// 								let field = class.static_fields.get(field);
//
// 								object.assert_matching_class(class_id, runtime)?;
// 								s.push(StackValue::from(object.get_field(field)));
// 							}
// 						}
//
// 						_ => todo!("{:?}", insn.inst),
// 					}
// 				}
// 			}
//
// 			_ => {
//
// 			}
// 		}
//
// 		Ok(None)
// 	}
//
// 	fn handle_err<O>(&mut self, res: JResult<O>) -> JResult<O> {
// 		match res {
// 			Ok(_) => {
// 				res
// 			}
// 			Err(mut err) => {
// 				err.stacktrace.push(TraceEntry {
// 					class: self.this_class,
// 					method: self.this_method,
// 					line: 0
// 				});
// 				Err(err)
// 			}
// 		}
// 	}
//
// 	// fn invoke_method(&mut self, method: ConstPtr<MethodConst>, runtime: &Runtime, pop_self: bool) -> JResult<()> {
// 	// 		let method_const = method.get(self.cp);
// 	// 		let name_and_type_const = method_const.name_and_type.get(self.cp);
// 	// 		let class_const = method_const.class.get(self.cp);
// 	// 		let class_name = class_const.name.get(self.cp).as_str();
// 	//
// 	// 		let method_ident = MethodIdentifier::new(name_and_type_const, self.cp);
// 	// 		let class_id = runtime.get_or_load_class(class_name);
// 	// 		let class = runtime.classes.get(class_id);
// 	//
// 	// 		let mut exec = runtime.create_scope(class, &method_ident);
// 	// 		let mut index = 0u16;
// 	// 		if pop_self {
// 	// 			if let StackValue::Object(object) = s.pop() {
// 	// 				object.assert_matching_class(class_id, runtime)?;
// 	// 				exec.l.set(index, Value::Object(object));
// 	// 			} else {
// 	// 				panic!("Expected object");
// 	// 			}
// 	// 			index += 1;
// 	// 		}
// 	// 		for _ in exec.m.desc.parameters.iter() {
// 	// 			// TODO check if parameters match
// 	// 			exec.l.set(index, Value::from(s.pop()));
// 	// 			index += 1;
// 	// 		}
// 	//
// 	// 		match exec.run(runtime) {
// 	// 			Err(mut err) => {
// 	// 				err.stacktrace.push(TraceEntry {
// 	// 					class: self.this_class,
// 	// 					method: self.this_method,
// 	// 					line: 69
// 	// 				});
// 	// 			}
// 	// 			Ok(Some(ret)) => {
// 	// 				if exec.m.desc.ret.is_void() {
// 	// 					return Err(JError::new("Function with void descriptor returned a value."));
// 	// 				} else {
// 	// 					s.push(ret);
// 	// 				}
// 	// 			}
// 	// 			Ok(None) => {
// 	//
// 	// 			}
// 	// 		}
// 	// 		Ok(())
// 	// 	}
// }
