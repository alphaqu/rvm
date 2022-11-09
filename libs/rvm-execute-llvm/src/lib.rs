use crate::executor::Executor;
use inkwell::context::Context;
use once_cell::sync::Lazy;
use rvm_execute::{Bindings, ExecutionEngine, Method};
use rvm_reader::ConstantPool;
use self_cell::self_cell;
use std::ffi::c_void;
use std::pin::Pin;
use std::sync::Mutex;

mod block;
mod compiler;
mod executor;
mod ir_gen;
mod op;
mod resolver;
pub(crate) mod util;

#[repr(transparent)]
struct LLVMEngine {
	unsafe_self_cell: ::self_cell::unsafe_self_cell::UnsafeSelfCell<
		LLVMEngine,
		Context,
		Executor<'static>
	>,

}
impl LLVMEngine {
	fn new(
		owner: Context,
		dependent_builder: impl for<'_q> FnOnce(&'_q Context) -> Executor<'_q>,
	) -> Self {
		use core::ptr::NonNull;

		unsafe {
			type JoinedCell<'_q> =
			::self_cell::unsafe_self_cell::JoinedCell<Context, Executor<'_q>>;

			let layout = ::self_cell::alloc::alloc::Layout::new::<JoinedCell>();
			assert!(layout.size() != 0);

			let joined_void_ptr = NonNull::new(::self_cell::alloc::alloc::alloc(layout)).unwrap();

			let mut joined_ptr = core::mem::transmute::<NonNull<u8>, NonNull<JoinedCell>>(
				joined_void_ptr
			);

			let (owner_ptr, dependent_ptr) = JoinedCell::_field_pointers(joined_ptr.as_ptr());


			owner_ptr.write(owner);


			let drop_guard =
				::self_cell::unsafe_self_cell::OwnerAndCellDropGuard::new(joined_ptr);


			dependent_ptr.write(dependent_builder(&*owner_ptr));
			core::mem::forget(drop_guard);

			Self {
				unsafe_self_cell: ::self_cell::unsafe_self_cell::UnsafeSelfCell::new(
					joined_void_ptr,
				),

			}
		}
	}

	fn try_new<Err>(
		owner: Context,
		dependent_builder:
		impl for<'_q> FnOnce(&'_q Context) -> core::result::Result<Executor<'_q>, Err>,
	) -> core::result::Result<Self, Err> {
		use core::ptr::NonNull;

		unsafe {
			type JoinedCell<'_q> =
			::self_cell::unsafe_self_cell::JoinedCell<Context, Executor<'_q>>;

			let layout = ::self_cell::alloc::alloc::Layout::new::<JoinedCell>();
			assert!(layout.size() != 0);

			let joined_void_ptr = NonNull::new(::self_cell::alloc::alloc::alloc(layout)).unwrap();

			let mut joined_ptr = core::mem::transmute::<NonNull<u8>, NonNull<JoinedCell>>(
				joined_void_ptr
			);

			let (owner_ptr, dependent_ptr) = JoinedCell::_field_pointers(joined_ptr.as_ptr());


			owner_ptr.write(owner);


			let mut drop_guard =
				::self_cell::unsafe_self_cell::OwnerAndCellDropGuard::new(joined_ptr);

			match dependent_builder(&*owner_ptr) {
				Ok(dependent) => {
					dependent_ptr.write(dependent);
					core::mem::forget(drop_guard);

					Ok(Self {
						unsafe_self_cell: ::self_cell::unsafe_self_cell::UnsafeSelfCell::new(
							joined_void_ptr,
						),

					})
				}
				Err(err) => Err(err)
			}
		}
	}

	fn try_new_or_recover<Err>(
		owner: Context,
		dependent_builder:
		impl for<'_q> FnOnce(&'_q Context) -> core::result::Result<Executor<'_q>, Err>,
	) -> core::result::Result<Self, (Context, Err)> {
		use core::ptr::NonNull;

		unsafe {
			type JoinedCell<'_q> =
			::self_cell::unsafe_self_cell::JoinedCell<Context, Executor<'_q>>;

			let layout = ::self_cell::alloc::alloc::Layout::new::<JoinedCell>();
			assert!(layout.size() != 0);

			let joined_void_ptr = NonNull::new(::self_cell::alloc::alloc::alloc(layout)).unwrap();

			let mut joined_ptr = core::mem::transmute::<NonNull<u8>, NonNull<JoinedCell>>(
				joined_void_ptr
			);

			let (owner_ptr, dependent_ptr) = JoinedCell::_field_pointers(joined_ptr.as_ptr());


			owner_ptr.write(owner);


			let mut drop_guard =
				::self_cell::unsafe_self_cell::OwnerAndCellDropGuard::new(joined_ptr);

			match dependent_builder(&*owner_ptr) {
				Ok(dependent) => {
					dependent_ptr.write(dependent);
					core::mem::forget(drop_guard);

					Ok(Self {
						unsafe_self_cell: ::self_cell::unsafe_self_cell::UnsafeSelfCell::new(
							joined_void_ptr,
						),

					})
				}
				Err(err) => {
					let owner_on_err = core::ptr::read(owner_ptr);


					core::mem::forget(drop_guard);
					::self_cell::alloc::alloc::dealloc(joined_void_ptr.as_ptr(), layout);

					Err((owner_on_err, err))
				}
			}
		}
	}

	fn borrow_owner<'_q>(&'_q self) -> &'_q Context {
		unsafe { self.unsafe_self_cell.borrow_owner::<Executor<'_q>>() }
	}

	fn with_dependent<'outer_fn, Ret>(
		&'outer_fn self,
		func: impl for<'_q> FnOnce(&'_q Context, &'outer_fn Executor<'_q>,
		) -> Ret) -> Ret {
		unsafe {
			func(
				self.unsafe_self_cell.borrow_owner::<Executor>(),
				self.unsafe_self_cell.borrow_dependent(),
			)
		}
	}

	fn with_dependent_mut<'outer_fn, Ret>(
		&'outer_fn mut self,
		func: impl for<'_q> FnOnce(&'_q Context, &'outer_fn mut Executor<'_q>) -> Ret,
	) -> Ret {
		let (owner, dependent) = unsafe {
			self.unsafe_self_cell.borrow_mut()
		};

		func(owner, dependent)
	}

	fn borrow_dependent<'_q>(&'_q self) -> &'_q Executor<'_q> {

		unsafe { self.unsafe_self_cell.borrow_dependent() }
	}

	fn into_owner(self) -> Context {
		let unsafe_self_cell = unsafe {
			core::mem::transmute::<
				Self,
				::self_cell::unsafe_self_cell::UnsafeSelfCell<
					LLVMEngine,
					Context,
					Executor<'static>
				>
			>(self)
		};

		let owner = unsafe { unsafe_self_cell.into_owner::<Executor>() };

		owner
	}
}
impl Drop for LLVMEngine {
	fn drop(&mut self) {
		unsafe {
			self.unsafe_self_cell.drop_joined::<Executor>();
		}
	}
}
impl core::fmt::Debug for LLVMEngine {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		self.with_dependent(|owner, dependent| {
			fmt.debug_struct(stringify!( LLVMEngine ))
				.field("owner", owner)
				.field("dependent", dependent)
				.finish()
		})
	}
}

pub struct LLVMExecutionEngine {
	engine: LLVMEngine,
}

impl LLVMExecutionEngine {
	pub fn new() -> LLVMExecutionEngine {
		LLVMExecutionEngine {
			engine: LLVMEngine::new(Context::create(), |ctx| {
				Executor::new(ctx)
			}),
		}
	}
}

impl ExecutionEngine for LLVMExecutionEngine {
	fn compile_method(
		&self,
		bindings: &Bindings,
		method: &Method,
		cp: &ConstantPool,
	) -> *const c_void {
		self.engine
			.borrow_dependent()
			.compile_method(bindings, method, cp) as *const c_void
	}
}