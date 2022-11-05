use either::Either;
use std::cmp::Ordering;
use std::mem::transmute;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Rem, Sub};
use std::sync::Arc;

use tracing::trace;

use rvm_consts::MethodAccessFlags;
use rvm_core::Id;

use crate::class::{Array, ObjectClass};
use crate::executor::instruction::Inst;
use crate::executor::locals::{LocalCast, LocalVariables};
use crate::executor::stack::{Stack, StackCast, StackValue};
use crate::object::{MethodCode, Type};
use crate::reader::{BinaryName, Code, ConstantInfo};
use crate::{Class, ClassKind, JResult, Method, Ref, Runtime, ValueDesc};

pub struct Frame<'a> {
	pub invoker: Option<&'a Frame<'a>>,
	pub locals: LocalVariables,
	pub stack: Stack,
	class: Id<Class>,
}

impl<'a> Frame<'a> {
	pub fn raw_frame(class_id: Id<Class>, stack: Stack) -> Frame<'a> {
		Frame {
			invoker: None,
			locals: LocalVariables::new(0),
			stack,
			class: class_id,
		}
	}

	pub fn new(
		class_id: Id<Class>,
		method_id: Id<Method>,
		runtime: &Runtime,
		invoker: &'a mut Frame,
	) -> JResult<Frame<'a>> {
		let class = runtime.cl.get_obj_class(class_id);
		let method = class.methods.get(method_id);

		let mut locals = LocalVariables::new(method.max_locals);
		let instance_method = !method.flags.contains(MethodAccessFlags::STATIC);
		for (i, _parameters) in method.desc.parameters.iter().enumerate().rev() {
			// Todo check parameters
			let value = invoker.stack.pop_raw()?;
			locals.set_stack((i + (instance_method as u8 as usize)) as u16, value)?;
		}
		if instance_method {
			locals.set(0, invoker.stack.pop::<Ref>()?)?;
		}
		Ok(Frame {
			locals,
			stack: Stack::new(method.max_stack),
			class: class_id,
			invoker: Some(invoker),
		})
	}

	pub fn execute(
		mut self,
		runtime: &Runtime,
		method_id: Id<Method>,
	) -> JResult<Option<StackValue>> {
		let class = runtime.cl.get_obj_class(self.class);
		let method = class.methods.get(method_id);
		let code = method.code.clone();
		drop(class);
		match code {
			Some(MethodCode::LLVM(code, _)) | Some(MethodCode::JVM(code)) => {
				self.execute_jvm(code, runtime)
			}
			Some(MethodCode::Native(either)) => {
				let code = match &either {
					Either::Left(source) => {
						let code = runtime.cl.native_methods().get(source).unwrap();
						// todo: save in method.code as Some(MethodCode::Native(Either::Right(*code)))
						code
					}
					Either::Right(code) => code,
				};

				(code.func)(&mut self.locals, runtime)
			}
			None => {
				panic!("cringe")
			}
		}
	}

	fn execute_jvm(&mut self, code: Arc<Code>, runtime: &Runtime) -> JResult<Option<StackValue>> {
		//let mut inv = self.invoker;
		//let mut out = 0;
		//while let Some(invoker) = inv {
		//    out += 1;
		//    inv = invoker.invoker;
		//}
		//info!("hi {out}");
		//trace!(target: "exec", "{} About to execute: ", self.name);
		//for (i, inst) in instructions.iter().enumerate() {
		//   // trace!(target: "exec", "{i} | {inst:?}");
		//}
		// trace!(target: "exec", "{} Executing: ", self.name);
		let mut op_idx = 0;
		while let Some(inst) = code.instructions.get(op_idx) {
			trace!(target: "exec", "{} | {inst:?}\t[{}]{}", op_idx, self.locals, &self.stack);
			// runtime.gc.write().unwrap().gc(runtime, self)?;
			match inst {
				Inst::NOP => {
					// pray
				}
				// Const
				Inst::ACONST_NULL
				| Inst::DCONST_0
				| Inst::DCONST_1
				| Inst::FCONST_0
				| Inst::FCONST_1
				| Inst::FCONST_2
				| Inst::ICONST_M1
				| Inst::ICONST_0
				| Inst::ICONST_1
				| Inst::ICONST_2
				| Inst::ICONST_3
				| Inst::ICONST_4
				| Inst::ICONST_5
				| Inst::LCONST_0
				| Inst::LCONST_1 => self.handle_const_op(inst),
				// Stack
				Inst::DUP
				| Inst::DUP_X1
				| Inst::DUP_X2
				| Inst::DUP2
				| Inst::DUP2_X1
				| Inst::DUP2_X2
				| Inst::POP
				| Inst::POP2
				| Inst::SWAP => self.handle_stack_op(inst)?,
				// Array
				Inst::NEWARRAY(_)
				| Inst::AALOAD
				| Inst::AASTORE
				| Inst::BALOAD
				| Inst::BASTORE
				| Inst::CALOAD
				| Inst::CASTORE
				| Inst::DALOAD
				| Inst::DASTORE
				| Inst::FALOAD
				| Inst::FASTORE
				| Inst::IALOAD
				| Inst::IASTORE
				| Inst::LALOAD
				| Inst::LASTORE
				| Inst::SALOAD
				| Inst::SASTORE
				| Inst::ARRAYLENGTH
				| Inst::ANEWARRAY(_)
				| Inst::MULTIANEWARRAY { .. } => self.handle_array_op(inst, runtime)?,
				// Math
				Inst::DADD
				| Inst::DDIV
				| Inst::DMUL
				| Inst::DNEG
				| Inst::DREM
				| Inst::DSUB
				| Inst::FADD
				| Inst::FDIV
				| Inst::FMUL
				| Inst::FNEG
				| Inst::FREM
				| Inst::FSUB
				| Inst::IADD
				| Inst::IDIV
				| Inst::IMUL
				| Inst::INEG
				| Inst::IREM
				| Inst::ISUB
				| Inst::IAND
				| Inst::IOR
				| Inst::ISHL
				| Inst::ISHR
				| Inst::IUSHR
				| Inst::IXOR
				| Inst::LADD
				| Inst::LDIV
				| Inst::LMUL
				| Inst::LNEG
				| Inst::LREM
				| Inst::LSUB
				| Inst::LAND
				| Inst::LOR
				| Inst::LSHL
				| Inst::LSHR
				| Inst::LUSHR
				| Inst::LXOR => self.handle_math_op(inst)?,
				// Comparison
				Inst::D2F
				| Inst::D2I
				| Inst::D2L
				| Inst::F2D
				| Inst::F2I
				| Inst::F2L
				| Inst::I2B
				| Inst::I2C
				| Inst::I2D
				| Inst::I2F
				| Inst::I2L
				| Inst::I2S
				| Inst::L2D
				| Inst::L2F
				| Inst::L2I => self.handle_conversion_op(inst)?,
				Inst::DCMPG | Inst::DCMPL | Inst::FCMPG | Inst::FCMPL | Inst::LCMP => {
					self.handle_comparison_op(inst)?
				}
				// Jump
				Inst::IF_ACMPEQ(_)
				| Inst::IF_ACMPNE(_)
				| Inst::IF_ICMPEQ(_)
				| Inst::IF_ICMPNE(_)
				| Inst::IF_ICMPLT(_)
				| Inst::IF_ICMPGE(_)
				| Inst::IF_ICMPGT(_)
				| Inst::IF_ICMPLE(_)
				| Inst::IFEQ(_)
				| Inst::IFNE(_)
				| Inst::IFLT(_)
				| Inst::IFGE(_)
				| Inst::IFGT(_)
				| Inst::IFLE(_)
				| Inst::IFNONNULL(_)
				| Inst::IFNULL(_)
				| Inst::GOTO(_)
				| Inst::GOTO_W(_) => {
					let offset = self.handle_jump_op(inst)?;
					if offset != 0 {
						op_idx = ((op_idx as isize) + (offset as isize)) as usize;
						continue;
					}
				}
				// Locals
				Inst::ALOAD(_)
				| Inst::ALOAD_W(_)
				| Inst::ALOAD0
				| Inst::ALOAD1
				| Inst::ALOAD2
				| Inst::ALOAD3
				| Inst::DLOAD(_)
				| Inst::DLOAD_W(_)
				| Inst::DLOAD0
				| Inst::DLOAD1
				| Inst::DLOAD2
				| Inst::DLOAD3
				| Inst::FLOAD(_)
				| Inst::FLOAD_W(_)
				| Inst::FLOAD0
				| Inst::FLOAD1
				| Inst::FLOAD2
				| Inst::FLOAD3
				| Inst::ILOAD(_)
				| Inst::ILOAD_W(_)
				| Inst::ILOAD0
				| Inst::ILOAD1
				| Inst::ILOAD2
				| Inst::ILOAD3
				| Inst::LLOAD(_)
				| Inst::LLOAD_W(_)
				| Inst::LLOAD0
				| Inst::LLOAD1
				| Inst::LLOAD2
				| Inst::LLOAD3
				| Inst::ASTORE(_)
				| Inst::ASTORE_W(_)
				| Inst::ASTORE0
				| Inst::ASTORE1
				| Inst::ASTORE2
				| Inst::ASTORE3
				| Inst::DSTORE(_)
				| Inst::DSTORE_W(_)
				| Inst::DSTORE0
				| Inst::DSTORE1
				| Inst::DSTORE2
				| Inst::DSTORE3
				| Inst::FSTORE(_)
				| Inst::FSTORE_W(_)
				| Inst::FSTORE0
				| Inst::FSTORE1
				| Inst::FSTORE2
				| Inst::FSTORE3
				| Inst::ISTORE(_)
				| Inst::ISTORE_W(_)
				| Inst::ISTORE0
				| Inst::ISTORE1
				| Inst::ISTORE2
				| Inst::ISTORE3
				| Inst::LSTORE(_)
				| Inst::LSTORE_W(_)
				| Inst::LSTORE0
				| Inst::LSTORE1
				| Inst::LSTORE2
				| Inst::LSTORE3
				| Inst::IINC(_, _)
				| Inst::IINC_W(_, _) => self.handle_local_op(inst)?,
				// Misc
				Inst::NEW(object) => {
					let class_id = runtime.resolve_class(self.class, *object)?;
					let object = runtime.new_object(class_id)?;
					self.stack.push(*object);
				}
				Inst::ATHROW => {
					todo!("throw")
				}
				Inst::BIPUSH(v) => {
					self.stack.push((*v) as i32);
				}
				Inst::SIPUSH(v) => {
					self.stack.push((*v) as i32);
				}
				Inst::CHECKCAST(class) => {
					todo!("checkcast")
				}
				Inst::INSTANCEOF(_) => {
					todo!("instanceof")
				}
				Inst::GETFIELD(field) => {
					let (class_id, field_id) = runtime.resolve_field(self.class, *field)?;
					let reference = self.stack.pop()?;
					let object = runtime.get_object(class_id, reference)?;

					self.stack.push_raw(object.get_field(field_id));
				}
				Inst::PUTFIELD(field) => {
					let (class_id, field_id) = runtime.resolve_field(self.class, *field)?;

					let stack_value: StackValue = self.stack.pop_raw()?;
					let reference = self.stack.pop()?;

					let object = runtime.get_object(class_id, reference)?;
					object.set_field(field_id, stack_value);
				}
				Inst::GETSTATIC(field) => {
					let (class_id, field_id) = runtime.resolve_field(self.class, *field)?;

					runtime.cl.scope_class(class_id, |class| {
						self.stack.push_raw(class.get_static(field_id));
					});
				}

				Inst::PUTSTATIC(field) => {
					let (class_id, field_id) = runtime.resolve_field(self.class, *field)?;

					let stack_value: StackValue = self.stack.pop_raw()?;
					runtime.cl.scope_class(class_id, |class| {
						class.set_static(field_id, stack_value);
					});
				}
				Inst::INVOKEDYNAMIC(_) => todo!("INVOKEDYNAMIC"),
				Inst::INVOKEINTERFACE(_, _) => todo!("INVOKEINTERFACE"),
				Inst::INVOKEVIRTUAL(value)
				| Inst::INVOKESTATIC(value)
				| Inst::INVOKESPECIAL(value) => {
					let (class_id, method_id) = runtime.resolve_method(self.class, *value)?;
					let frame = Frame::new(class_id, method_id, runtime, self)?;

					if let Some(value) = frame.execute(runtime, method_id)? {
						self.stack.push_raw(value);
					}
				}
				// grandpa shit
				Inst::JSR(_) => todo!("grandpa shit"),
				Inst::JSR_W(_) => todo!("grandpa shit"),
				Inst::RET(_) => todo!("grandpa shit"),
				// ConstantPool Loading
				Inst::LDC(_) | Inst::LDC_W(_) | Inst::LDC2_W(_) => {
					self.handle_constant_pool_op(inst, runtime)?
				}
				// alpha reading challange any%
				Inst::LOOKUPSWITCH => todo!("read"),
				Inst::TABLESWITCH => todo!("read"),
				Inst::MONITORENTER => todo!("read"),
				Inst::MONITOREXIT => todo!("read"),
				// Return
				Inst::RETURN
				| Inst::ARETURN
				| Inst::DRETURN
				| Inst::FRETURN
				| Inst::IRETURN
				| Inst::LRETURN => {
					trace!(target: "exec", "Method end");

					return self.handle_return_op(inst);
				}
			}

			op_idx += 1;
		}

		panic!("method unexpectedly stopped")
	}

	fn handle_const_op(&mut self, inst: &Inst) {
		self.stack.push_raw(match inst {
			Inst::ACONST_NULL => StackCast::push(Ref::null()),
			Inst::DCONST_0 => StackCast::push(0.0f64),
			Inst::DCONST_1 => StackCast::push(1.0f64),
			Inst::FCONST_0 => StackCast::push(0.0f32),
			Inst::FCONST_1 => StackCast::push(1.0f32),
			Inst::FCONST_2 => StackCast::push(2.0f32),
			Inst::ICONST_M1 => StackCast::push(-1i32),
			Inst::ICONST_0 => StackCast::push(0i32),
			Inst::ICONST_1 => StackCast::push(1i32),
			Inst::ICONST_2 => StackCast::push(2i32),
			Inst::ICONST_3 => StackCast::push(3i32),
			Inst::ICONST_4 => StackCast::push(4i32),
			Inst::ICONST_5 => StackCast::push(5i32),
			Inst::LCONST_0 => StackCast::push(0i64),
			Inst::LCONST_1 => StackCast::push(1i64),
			_ => {
				panic!("Invalid instruction");
			}
		});
	}

	fn handle_stack_op(&mut self, inst: &Inst) -> JResult<()> {
		match inst {
			Inst::DUP => {
				let value = self.stack.pop_raw()?;
				self.stack.push_raw(value.clone());
				self.stack.push_raw(value);
			}
			Inst::DUP_X1 => {
				let value1 = self.stack.pop_raw()?;
				let value2 = self.stack.pop_raw()?;
				self.stack.push_raw(value1.clone());
				self.stack.push_raw(value2);
				self.stack.push_raw(value1);
			}
			Inst::DUP_X2 => {
				let value1 = self.stack.pop_raw()?;
				let value2 = self.stack.pop_raw()?;
				if !value2.is_category_2() {
					// Form 1
					let value3 = self.stack.pop_raw()?;
					self.stack.push_raw(value1.clone());
					self.stack.push_raw(value3);
					self.stack.push_raw(value2);
					self.stack.push_raw(value1);
				} else {
					// Form 2
					self.stack.push_raw(value1.clone());
					self.stack.push_raw(value2);
					self.stack.push_raw(value1);
				}
			}
			Inst::DUP2 => {
				let value1 = self.stack.pop_raw()?;
				if !value1.is_category_2() {
					// Form 1
					let value2 = self.stack.pop_raw()?;
					self.stack.push_raw(value2.clone());
					self.stack.push_raw(value1.clone());
					self.stack.push_raw(value2);
					self.stack.push_raw(value1);
				} else {
					// Form 2
					self.stack.push_raw(value1.clone());
					self.stack.push_raw(value1);
				}
			}
			Inst::DUP2_X1 => {
				let value1 = self.stack.pop_raw()?;
				let value2 = self.stack.pop_raw()?;
				if !value1.is_category_2() {
					// Form 1
					let value3 = self.stack.pop_raw()?;
					self.stack.push_raw(value1.clone());
					self.stack.push_raw(value2.clone());
					self.stack.push_raw(value3);
					self.stack.push_raw(value1);
					self.stack.push_raw(value2);
				} else {
					// Form 2
					self.stack.push_raw(value1.clone());
					self.stack.push_raw(value2);
					self.stack.push_raw(value1);
				}
			}
			// this is hell
			Inst::DUP2_X2 => {
				let value1 = self.stack.pop_raw()?;
				let value2 = self.stack.pop_raw()?;

				if value1.is_category_2() && value2.is_category_2() {
					// Form 4
					self.stack.push_raw(value1.clone());
					self.stack.push_raw(value2);
					self.stack.push_raw(value1);
					return Ok(());
				}

				let value3 = self.stack.pop_raw()?;
				if value3.is_category_2() {
					// Form 3
					self.stack.push_raw(value2.clone());
					self.stack.push_raw(value1.clone());
					self.stack.push_raw(value3);
					self.stack.push_raw(value2);
					self.stack.push_raw(value1);
					return Ok(());
				}

				if value1.is_category_2() {
					// Form 2
					self.stack.push_raw(value1.clone());
					self.stack.push_raw(value3);
					self.stack.push_raw(value2);
					self.stack.push_raw(value1);
					return Ok(());
				}

				let value4 = self.stack.pop_raw()?;
				{
					// Form 1
					self.stack.push_raw(value2.clone());
					self.stack.push_raw(value1.clone());
					self.stack.push_raw(value4);
					self.stack.push_raw(value3);
					self.stack.push_raw(value2);
					self.stack.push_raw(value1);
				}
			}
			Inst::POP => {
				if self.stack.pop_raw()?.is_category_2() {
					panic!("category 2 not allowed")
				}
			}
			Inst::POP2 => {
				let value1 = self.stack.pop_raw()?;
				if !value1.is_category_2() {
					let value2 = self.stack.pop_raw()?;
					if value2.is_category_2() {
						panic!("category 2 not allowed")
					}
				}
			}
			Inst::SWAP => {
				// funny jvm no form business this time
				let value1 = self.stack.pop_raw()?;
				let value2 = self.stack.pop_raw()?;
				self.stack.push_raw(value1);
				self.stack.push_raw(value2);
			}
			_ => {
				panic!("Invalid instruction");
			}
		}
		Ok(())
	}

	fn handle_array_op(&mut self, inst: &Inst, runtime: &Runtime) -> JResult<()> {
		fn load<T: Type + StackCast>(frame: &mut Frame, runtime: &Runtime) -> JResult<()> {
			let index: i32 = frame.stack.pop()?;
			let reference: Ref = frame.stack.pop()?;
			let array: Array<T> = runtime.get_array(reference)?;
			frame.stack.push(array.load(index));
			Ok(())
		}

		fn store<T: Type + StackCast>(frame: &mut Frame, runtime: &Runtime) -> JResult<()> {
			let value: T = frame.stack.pop()?;
			let index: i32 = frame.stack.pop()?;
			let reference: Ref = frame.stack.pop()?;
			let array: Array<T> = runtime.get_array(reference)?;
			array.store(index, value);
			Ok(())
		}

		match inst {
			Inst::NEWARRAY(desc) => {
				let desc = BinaryName::Array(ValueDesc::Base(*desc));
				let class_id = runtime.cl.get_class_id(&desc);
				let class = runtime.cl.get(class_id);
				match &class.kind {
					ClassKind::Array(array) => {
						let length: i32 = self.stack.pop()?;
						let object = array.new_array(class_id, length, runtime);
						self.stack.push(object);
					}
					_ => {
						panic!("id should be array class")
					}
				}
			}
			Inst::ANEWARRAY(desc) => {
				let component_id = runtime.resolve_class(self.class, *desc)?;
				let component = runtime.cl.get(component_id);
				let name = BinaryName::parse(&component.binary_name).to_component();
				drop(component);

				let class_id = runtime.cl.get_class_id(&name);
				let class = runtime.cl.get(class_id);

				match &class.kind {
					ClassKind::Array(array) => {
						let length: i32 = self.stack.pop()?;
						let object = array.new_array(class_id, length, runtime);
						self.stack.push(object);
					}
					_ => {
						panic!("id should be array class")
					}
				}
			}
			Inst::MULTIANEWARRAY { .. } => todo!("MULTIANEWARRAY"),
			Inst::AALOAD => load::<Ref>(self, runtime)?,
			Inst::BALOAD => load::<i8>(self, runtime)?,
			Inst::CALOAD => load::<u16>(self, runtime)?,
			Inst::DALOAD => load::<f64>(self, runtime)?,
			Inst::FALOAD => load::<f32>(self, runtime)?,
			Inst::IALOAD => load::<i32>(self, runtime)?,
			Inst::LALOAD => load::<i64>(self, runtime)?,
			Inst::SALOAD => load::<i16>(self, runtime)?,
			Inst::AASTORE => store::<Ref>(self, runtime)?,
			Inst::BASTORE => store::<i8>(self, runtime)?,
			Inst::CASTORE => store::<u16>(self, runtime)?,
			Inst::DASTORE => store::<f64>(self, runtime)?,
			Inst::FASTORE => store::<f32>(self, runtime)?,
			Inst::IASTORE => store::<i32>(self, runtime)?,
			Inst::LASTORE => store::<i64>(self, runtime)?,
			Inst::SASTORE => store::<i16>(self, runtime)?,
			Inst::ARRAYLENGTH => {
				let object: Ref = self.stack.pop()?;
				let array = runtime.get_untyped_array(object)?;
				self.stack.push(array.get_length());
			}
			_ => {
				panic!("invalid")
			}
		};

		Ok(())
	}

	fn handle_math_op(&mut self, inst: &Inst) -> JResult<()> {
		fn merge<V: StackCast>(stack: &mut Stack, operation: fn(V, V) -> V) -> JResult<()> {
			let v2 = stack.pop::<V>()?;
			let v = stack.pop::<V>()?;
			stack.push(operation(v, v2));
			Ok(())
		}

		fn apply<V: StackCast>(stack: &mut Stack, operation: fn(V) -> V) -> JResult<()> {
			let v = stack.pop::<V>()?;
			stack.push(operation(v));
			Ok(())
		}

		match inst {
			Inst::DADD => merge(&mut self.stack, f64::add)?,
			Inst::DDIV => merge(&mut self.stack, f64::div)?,
			Inst::DMUL => merge(&mut self.stack, f64::mul)?,
			Inst::DNEG => apply(&mut self.stack, f64::neg)?,
			Inst::DREM => merge(&mut self.stack, f64::rem)?,
			Inst::DSUB => merge(&mut self.stack, f64::sub)?,

			Inst::FADD => merge(&mut self.stack, f32::add)?,
			Inst::FDIV => merge(&mut self.stack, f32::div)?,
			Inst::FMUL => merge(&mut self.stack, f32::mul)?,
			Inst::FNEG => apply(&mut self.stack, f32::neg)?,
			Inst::FREM => merge(&mut self.stack, f32::rem)?,
			Inst::FSUB => merge(&mut self.stack, f32::sub)?,

			Inst::IADD => merge(&mut self.stack, i32::wrapping_add)?,
			Inst::IDIV => merge(&mut self.stack, i32::wrapping_div)?,
			Inst::IMUL => merge(&mut self.stack, i32::wrapping_mul)?,
			Inst::INEG => apply(&mut self.stack, i32::wrapping_neg)?,
			Inst::IREM => merge(&mut self.stack, i32::wrapping_rem)?,
			Inst::ISUB => merge(&mut self.stack, i32::wrapping_sub)?,
			Inst::IAND => merge(&mut self.stack, i32::bitand)?,
			Inst::IOR => merge(&mut self.stack, i32::bitor)?,
			Inst::ISHL => merge(&mut self.stack, |v0, v1| {
				if v1 > 0 {
					i32::wrapping_shl(v0, v1 as u32)
				} else {
					i32::wrapping_shl(v0, -v1 as u32)
				}
			})?,
			Inst::ISHR => merge(&mut self.stack, |v0, v1| {
				if v1 > 0 {
					i32::wrapping_shr(v0, v1 as u32)
				} else {
					i32::wrapping_shr(v0, -v1 as u32)
				}
			})?,
			Inst::IUSHR => merge(&mut self.stack, |v0, v1| {
				// local shift kinda cringe here
				unsafe {
					let v0 = transmute::<i32, u32>(v0);
					transmute::<u32, i32>(if v1 > 0 {
						u32::wrapping_shr(v0, v1 as u32)
					} else {
						u32::wrapping_shr(v0, -v1 as u32)
					})
				}
			})?,
			Inst::IXOR => merge(&mut self.stack, i32::bitxor)?,

			Inst::LADD => merge(&mut self.stack, i64::wrapping_add)?,
			Inst::LDIV => merge(&mut self.stack, i64::wrapping_div)?,
			Inst::LMUL => merge(&mut self.stack, i64::wrapping_mul)?,
			Inst::LNEG => apply(&mut self.stack, i64::wrapping_neg)?,
			Inst::LREM => merge(&mut self.stack, i64::wrapping_rem)?,
			Inst::LSUB => merge(&mut self.stack, i64::wrapping_sub)?,
			Inst::LAND => merge(&mut self.stack, i64::bitand)?,
			Inst::LOR => merge(&mut self.stack, i64::bitor)?,
			Inst::LSHL => merge(&mut self.stack, |v0, v1| {
				if v1 > 0 {
					i64::wrapping_shl(v0, v1 as u32)
				} else {
					i64::wrapping_shl(v0, -v1 as u32)
				}
			})?,
			Inst::LSHR => merge(&mut self.stack, |v0, v1| {
				if v1 > 0 {
					i64::wrapping_shr(v0, v1 as u32)
				} else {
					i64::wrapping_shr(v0, -v1 as u32)
				}
			})?,
			Inst::LUSHR => merge(&mut self.stack, |v0, v1| {
				// local shift kinda cringe here
				unsafe {
					let v0 = transmute::<i64, u64>(v0);
					transmute::<u64, i64>(if v1 > 0 {
						u64::wrapping_shr(v0, v1 as u32)
					} else {
						u64::wrapping_shr(v0, -v1 as u32)
					})
				}
			})?,
			Inst::LXOR => merge(&mut self.stack, i64::bitxor)?,
			_ => {
				panic!("invalid")
			}
		}
		Ok(())
	}

	fn handle_conversion_op(&mut self, inst: &Inst) -> JResult<()> {
		match inst {
			Inst::D2F => {
				let value = self.stack.pop::<f64>()?;
				self.stack.push(value as f32)
			}
			Inst::D2I => {
				let value = self.stack.pop::<f64>()?;
				self.stack.push(value as i32)
			}
			Inst::D2L => {
				let x3 = self.stack.pop::<f64>()?;
				self.stack.push(x3 as i64)
			}
			Inst::F2D => {
				let x2 = self.stack.pop::<f32>()?;
				self.stack.push(x2 as f64)
			}
			Inst::F2I => {
				let x1 = self.stack.pop::<f32>()?;
				self.stack.push(x1 as i32)
			}
			Inst::F2L => {
				let x = self.stack.pop::<f32>()?;
				self.stack.push(x as i64)
			}
			Inst::I2B => {
				let i8 = self.stack.pop::<i32>()?;
				self.stack.push(i8 as f32)
			}
			Inst::I2C => {
				let i7 = self.stack.pop::<i32>()?;
				self.stack.push(i7 as u16 as i32)
			}
			Inst::I2D => {
				let i6 = self.stack.pop::<i32>()?;
				self.stack.push(i6 as f64)
			}
			Inst::I2F => {
				let i5 = self.stack.pop::<i32>()?;
				self.stack.push(i5 as f32)
			}
			Inst::I2L => {
				let i4 = self.stack.pop::<i32>()?;
				self.stack.push(i4 as i64)
			}
			Inst::I2S => {
				let i3 = self.stack.pop::<i32>()?;
				self.stack.push(i3 as i16 as i32)
			}
			Inst::L2D => {
				let i2 = self.stack.pop::<i64>()?;
				self.stack.push(i2 as f64)
			}
			Inst::L2F => {
				let i1 = self.stack.pop::<i64>()?;
				self.stack.push(i1 as f32)
			}
			Inst::L2I => {
				let i = self.stack.pop::<i64>()?;
				self.stack.push(i as i32)
			}
			_ => {
				panic!("invalid")
			}
		}
		Ok(())
	}

	fn handle_comparison_op(&mut self, inst: &Inst) -> JResult<()> {
		match inst {
			Inst::DCMPL | Inst::DCMPG => {
				let value2: f64 = self.stack.pop()?;
				let value1: f64 = self.stack.pop()?;
				if value1 > value2 {
					self.stack.push(1);
				} else if value1 == value2 {
					self.stack.push(0);
				} else if value1 < value2 {
					self.stack.push(-1);
				} else {
					// i hope rustc inlines the shit out of this
					if matches!(inst, Inst::DCMPG) {
						self.stack.push(1);
					} else {
						self.stack.push(-1);
					}
				}
			}
			Inst::FCMPL | Inst::FCMPG => {
				let value2: f32 = self.stack.pop()?;
				let value1: f32 = self.stack.pop()?;
				if value1 > value2 {
					self.stack.push(1);
				} else if value1 == value2 {
					self.stack.push(0);
				} else if value1 < value2 {
					self.stack.push(-1);
				} else {
					// i hope rustc inlines the shit out of this
					if matches!(inst, Inst::FCMPG) {
						self.stack.push(1);
					} else {
						self.stack.push(-1);
					}
				}
			}
			Inst::LCMP => {
				let value2: i64 = self.stack.pop()?;
				let value1: i64 = self.stack.pop()?;
				self.stack.push(match value1.cmp(&value2) {
					Ordering::Less => -1,
					Ordering::Equal => 0,
					Ordering::Greater => 1,
				});
			}
			_ => {
				panic!("invalid")
			}
		}
		Ok(())
	}

	fn handle_jump_op(&mut self, inst: &Inst) -> JResult<i32> {
		match inst {
			Inst::IF_ACMPEQ(target) | Inst::IF_ACMPNE(target) => {
				let value2: Ref = self.stack.pop()?;
				let value1: Ref = self.stack.pop()?;
				if match inst {
					Inst::IF_ACMPEQ(_) => value1 == value2,
					Inst::IF_ACMPNE(_) => value1 != value2,
					_ => {
						panic!("invalid")
					}
				} {
					return Ok(target.0 as i32);
				}
			}
			Inst::IF_ICMPEQ(target)
			| Inst::IF_ICMPNE(target)
			| Inst::IF_ICMPLT(target)
			| Inst::IF_ICMPGE(target)
			| Inst::IF_ICMPGT(target)
			| Inst::IF_ICMPLE(target) => {
				let value2: i32 = self.stack.pop()?;
				let value1: i32 = self.stack.pop()?;
				if match inst {
					Inst::IF_ICMPEQ(_) => value1 == value2,
					Inst::IF_ICMPNE(_) => value1 != value2,
					Inst::IF_ICMPLT(_) => value1 < value2,
					Inst::IF_ICMPLE(_) => value1 <= value2,
					Inst::IF_ICMPGT(_) => value1 > value2,
					Inst::IF_ICMPGE(_) => value1 >= value2,
					_ => {
						panic!("invalid")
					}
				} {
					return Ok(target.0 as i32);
				}
			}
			Inst::IFEQ(target)
			| Inst::IFNE(target)
			| Inst::IFLT(target)
			| Inst::IFGE(target)
			| Inst::IFGT(target)
			| Inst::IFLE(target) => {
				let value: i32 = self.stack.pop()?;
				if match inst {
					Inst::IFEQ(_) => value == 0,
					Inst::IFNE(_) => value != 0,
					Inst::IFLT(_) => value < 0,
					Inst::IFLE(_) => value <= 0,
					Inst::IFGT(_) => value > 0,
					Inst::IFGE(_) => value >= 0,
					_ => {
						panic!("invalid")
					}
				} {
					return Ok(target.0 as i32);
				}
			}
			Inst::IFNONNULL(target) => {
				let value: Ref = self.stack.pop()?;
				if !value.is_null() {
					return Ok(target.0 as i32);
				}
			}
			Inst::IFNULL(target) => {
				let value: Ref = self.stack.pop()?;
				if value.is_null() {
					return Ok(target.0 as i32);
				}
			}
			Inst::GOTO(target) => {
				return Ok(target.0 as i32);
			}
			_ => {
				panic!("invalid")
			}
		}

		Ok(0)
	}

	fn handle_local_op(&mut self, inst: &Inst) -> JResult<()> {
		fn load<V: StackCast + LocalCast>(frame: &mut Frame, local: u16) -> JResult<()>
		where
			[(); V::L]:,
		{
			frame.stack.push(frame.locals.get::<V>(local)?);
			Ok(())
		}

		fn store<V: StackCast + LocalCast>(frame: &mut Frame, local: u16) -> JResult<()>
		where
			[(); V::L]:,
		{
			let value = frame.stack.pop::<V>()?;
			frame.locals.set(local, value)?;
			Ok(())
		}

		// cringetastic:tm:
		// it is also your responsibility to ensure this works
		match inst {
			Inst::ALOAD(v) => load::<Ref>(self, *v as u16),
			Inst::ALOAD_W(v) => load::<Ref>(self, *v),
			Inst::ALOAD0 => load::<Ref>(self, 0),
			Inst::ALOAD1 => load::<Ref>(self, 1),
			Inst::ALOAD2 => load::<Ref>(self, 2),
			Inst::ALOAD3 => load::<Ref>(self, 3),
			Inst::FLOAD(v) => load::<f32>(self, *v as u16),
			Inst::FLOAD_W(v) => load::<f32>(self, *v),
			Inst::FLOAD0 => load::<f32>(self, 0),
			Inst::FLOAD1 => load::<f32>(self, 1),
			Inst::FLOAD2 => load::<f32>(self, 2),
			Inst::FLOAD3 => load::<f32>(self, 3),
			Inst::ILOAD(v) => load::<i32>(self, *v as u16),
			Inst::ILOAD_W(v) => load::<i32>(self, *v),
			Inst::ILOAD0 => load::<i32>(self, 0),
			Inst::ILOAD1 => load::<i32>(self, 1),
			Inst::ILOAD2 => load::<i32>(self, 2),
			Inst::ILOAD3 => load::<i32>(self, 3),
			Inst::DLOAD(v) => load::<f64>(self, *v as u16),
			Inst::DLOAD_W(v) => load::<f64>(self, *v),
			Inst::DLOAD0 => load::<f64>(self, 0),
			Inst::DLOAD1 => load::<f64>(self, 1),
			Inst::DLOAD2 => load::<f64>(self, 2),
			Inst::DLOAD3 => load::<f64>(self, 3),
			Inst::LLOAD(v) => load::<i64>(self, *v as u16),
			Inst::LLOAD_W(v) => load::<i64>(self, *v),
			Inst::LLOAD0 => load::<i64>(self, 0),
			Inst::LLOAD1 => load::<i64>(self, 1),
			Inst::LLOAD2 => load::<i64>(self, 2),
			Inst::LLOAD3 => load::<i64>(self, 3),
			Inst::ASTORE(v) => store::<Ref>(self, *v as u16),
			Inst::ASTORE_W(v) => store::<Ref>(self, *v),
			Inst::ASTORE0 => store::<Ref>(self, 0),
			Inst::ASTORE1 => store::<Ref>(self, 1),
			Inst::ASTORE2 => store::<Ref>(self, 2),
			Inst::ASTORE3 => store::<Ref>(self, 3),
			Inst::FSTORE(v) => store::<f32>(self, *v as u16),
			Inst::FSTORE_W(v) => store::<f32>(self, *v),
			Inst::FSTORE0 => store::<f32>(self, 0),
			Inst::FSTORE1 => store::<f32>(self, 1),
			Inst::FSTORE2 => store::<f32>(self, 2),
			Inst::FSTORE3 => store::<f32>(self, 3),
			Inst::ISTORE(v) => store::<i32>(self, *v as u16),
			Inst::ISTORE_W(v) => store::<i32>(self, *v),
			Inst::ISTORE0 => store::<i32>(self, 0),
			Inst::ISTORE1 => store::<i32>(self, 1),
			Inst::ISTORE2 => store::<i32>(self, 2),
			Inst::ISTORE3 => store::<i32>(self, 3),
			Inst::DSTORE(v) => store::<f64>(self, *v as u16),
			Inst::DSTORE_W(v) => store::<f64>(self, *v),
			Inst::DSTORE0 => store::<f64>(self, 0),
			Inst::DSTORE1 => store::<f64>(self, 1),
			Inst::DSTORE2 => store::<f64>(self, 2),
			Inst::DSTORE3 => store::<f64>(self, 3),
			Inst::LSTORE(v) => store::<i64>(self, *v as u16),
			Inst::LSTORE_W(v) => store::<i64>(self, *v),
			Inst::LSTORE0 => store::<i64>(self, 0),
			Inst::LSTORE1 => store::<i64>(self, 1),
			Inst::LSTORE2 => store::<i64>(self, 2),
			Inst::LSTORE3 => store::<i64>(self, 3),
			Inst::IINC(index, amount) => {
				let mut value = self.locals.get::<i32>(*index as u16)?;
				value = value.wrapping_add((*amount) as i32);
				self.locals.set::<i32>(*index as u16, value)?;
				Ok(())
			}
			Inst::IINC_W(index, amount) => {
				let mut value = self.locals.get::<i32>(*index)?;
				value = value.wrapping_add((*amount) as i32);
				self.locals.set::<i32>(*index, value)?;
				Ok(())
			}
			_ => {
				panic!("invalid")
			}
		}
	}

	fn handle_return_op(&mut self, inst: &Inst) -> JResult<Option<StackValue>> {
		match inst {
			Inst::RETURN => Ok(None),
			Inst::ARETURN => {
				let value: Ref = self.stack.pop()?;
				Ok(Some(StackCast::push(value)))
			}
			Inst::DRETURN => {
				let value: f64 = self.stack.pop()?;
				Ok(Some(StackCast::push(value)))
			}
			Inst::FRETURN => {
				let value: f32 = self.stack.pop()?;
				Ok(Some(StackCast::push(value)))
			}
			Inst::IRETURN => {
				let value: i32 = self.stack.pop()?;
				Ok(Some(StackCast::push(value)))
			}
			Inst::LRETURN => {
				let value: i64 = self.stack.pop()?;
				Ok(Some(StackCast::push(value)))
			}
			_ => {
				panic!("Invalid")
			}
		}
	}

	fn handle_constant_pool_op(&mut self, inst: &Inst, runtime: &Runtime) -> JResult<()> {
		let class = runtime.cl.get_obj_class(self.class);

		fn ldc(class: &ObjectClass, frame: &mut Frame, index: u16, runtime: &Runtime) {
			let constant = class.cp.get_raw(index).unwrap();
			match constant {
				ConstantInfo::Integer(int) => {
					frame.stack.push(int.bytes);
				}
				ConstantInfo::Float(float) => {
					frame.stack.push(float.bytes);
				}
				ConstantInfo::String(string) => {
					let string = string.string.get(&class.cp);
					let id = runtime
						.cl
						.get_class_id(&BinaryName::Object("java.lang.String".to_string()));
					todo!("string constants")
				}
				ConstantInfo::Class(class) => {
					todo!("Class objects")
				}
				ConstantInfo::MethodHandle(_) => {
					todo!("method handle")
				}
				ConstantInfo::MethodType(_) => {
					todo!("method type")
				}
				_ => {
					todo!("maybe unsupported")
				}
			}
		}
		match inst {
			Inst::LDC(loc) => ldc(&class, self, *loc as u16, runtime),
			Inst::LDC_W(loc) => ldc(&class, self, *loc, runtime),
			Inst::LDC2_W(loc) => {
				let constant = class.cp.get_raw(*loc).unwrap();
				match constant {
					ConstantInfo::Float(int) => {
						self.stack.push(int.bytes);
					}
					ConstantInfo::Double(float) => {
						self.stack.push(float.bytes);
					}
					_ => {
						todo!("maybe unsupported")
					}
				}
			}
			_ => {
				panic!("invalid")
			}
		}
		Ok(())
	}
}
