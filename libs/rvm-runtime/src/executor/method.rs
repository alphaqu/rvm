use crate::reader::{ConstPtr, MethodConst};
//use crate::{Executor, JError, JResult, MethodIdentifier, Runtime, TraceEntry};

// pub struct MethodInsn;
//
// impl MethodInsn {
// 	pub fn invoke<H: MethodInsnHandler>(exec: &mut Executor, method: ConstPtr<MethodConst>, runtime: &Runtime) -> JResult<()> {
// 		let method_const = method.get(exec.cp);
// 		let name_and_type_const = method_const.name_and_type.get(exec.cp);
// 		let class_const = method_const.class.get(exec.cp);
// 		let class_name = class_const.name.get(exec.cp).as_str();
//
// 		let method_ident = MethodIdentifier::new(name_and_type_const, exec.cp);
// 		let class_id = runtime.get_or_load_class(class_name);
// 		let class = runtime.classes.get(class_id);
//
// 		let mut invoke = runtime.create_scope(class, &method_ident);
//
// 		H::insert_locals(exec, &mut invoke, runtime)?;
// 		// let mut index = 0u16;
// 		// 		if pop_self {
// 		// 			if let StackValue::Object(object) = exec.s.pop() {
// 		// 				object.assert_matching_class(class_id, runtime)?;
// 		// 				invoke.l.set(index, Value::Object(object));
// 		// 			} else {
// 		// 				panic!("Expected object");
// 		// 			}
// 		// 			index += 1;
// 		// 		}
// 		// 		for _ in invoke.m.desc.parameters.iter() {
// 		// 			// TODO check if parameters match
// 		// 			invoke.l.set(index, Value::from(self.s.pop()));
// 		// 			index += 1;
// 		// 		}
//
// 		match invoke.run(runtime) {
// 			Err(mut err) => {
// 				err.stacktrace.push(TraceEntry {
// 					class: exec.this_class,
// 					method: exec.this_method,
// 					line: 69
// 				});
// 			}
// 			Ok(Some(ret)) => {
// 				if invoke.m.desc.ret.is_void() {
// 					return Err(JError::new("Function with void descriptor returned a value."));
// 				} else {
// 					exec.s.push(ret);
// 				}
// 			}
// 			Ok(None) => {
//
// 			}
// 		}
//
// 		Ok(())
// 	}
// }
//
// pub trait MethodInsnHandler {
// 	fn additional_locals() -> usize;
// 	fn insert_locals(exec: &mut Executor, invoke: &mut Executor, runtime: &Runtime) -> JResult<()>;
// }
//
// pub struct StaticInsn;
//
// impl MethodInsnHandler for StaticInsn {
// 	fn additional_locals() -> usize {
// 		0
// 	}
//
// 	fn insert_locals(exec: &mut Executor, invoke: &mut Executor, runtime: &Runtime) -> JResult<()> {
//
//
//
// 		Ok(())
// 	}
// }