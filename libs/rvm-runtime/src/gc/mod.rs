#![feature(hash_drain_filter)]

mod root;
mod chunk;

use crate::class::Class;
use crate::executor::{Frame, LocalVar, StackValue};
use crate::object::{ObjectData, Ref, ValueType, HEADER_SIZE};
use crate::{ClassKind, JResult, Runtime};
use ahash::{HashMap, HashSet};
use mimalloc::MiMalloc;
pub use root::RootHandle;
use rvm_core::Id;
use std::alloc::dealloc;
use std::sync::{Arc, Weak};
use std::time::Instant;
use tracing::{debug, info, trace};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub struct GarbageCollector {
    objects: HashSet<Ref>,
    mark: bool,
}

impl GarbageCollector {
    pub fn new() -> GarbageCollector {
        GarbageCollector {
            objects: Default::default(),
            mark: false,
        }
    }

    pub unsafe fn alloc(&mut self, class_id: Id<Class>, size: usize) -> Ref {
        let object = unsafe { Ref::new(self.mark, class_id.idx(), size) };
        self.objects.insert(object);
        object
    }

    pub fn gc(&mut self, runtime: &Runtime, frame: &Frame) -> JResult<()> {
        debug!(target: "gc", "Starting garbage collection");
        self.mark = !self.mark;

        let start = Instant::now();
        let mut count = 0usize;
        // mark statics
        for class in runtime.cl.classes().iter() {
            match &class.kind {
                ClassKind::Object(object) => {
                    for field in object.fields.object_fields(true) {
                        if let StackValue::Reference(object) = object.get_static(*field) {
                            Self::mark_object(object, self.mark, runtime, &mut count)?;
                        }
                    }
                }
                ClassKind::Array(_) => {}
                ClassKind::Primitive(_) => {}
            }
        }

        // mark frames
        Self::mark_frame(frame, self.mark, runtime, &mut count)?;
        trace!(target: "gc", "Marked {} objects in {}ms", count, start.elapsed().as_millis());
        // sweep
        self.objects.retain(|v| {
            if v.get_mark() == self.mark {
                true
            } else {
                unsafe {
                    debug!(target: "gc", "Deallocating {}", v);
                    // dealloc
                    let class = runtime.cl.get(v.get_class());
                    let size = class.kind.obj_size(*v);
                    let ptr = v.ptr();
                    dealloc(ptr.sub(HEADER_SIZE), ObjectData::layout(size + HEADER_SIZE));
                }
                false
            }
        });

        Ok(())
    }

    fn mark_frame(frame: &Frame, mark: bool, runtime: &Runtime, count: &mut usize) -> JResult<()> {
       // trace!(target: "gc", "Marking frame {}", frame.name);

        for value in frame.stack.iter() {
            if let StackValue::Reference(value) = value {
                Self::mark_object(*value, mark, runtime, count)?;
            }
        }
        for value in frame.locals.iter() {
            if let LocalVar::Reference(value) = value {
                Self::mark_object(*value, mark, runtime, count)?;
            }
        }

        if let Some(parent) = frame.invoker {
            Self::mark_frame(parent, mark, runtime, count)?;
        }

        Ok(())
    }

    fn mark_object(object: Ref, mark: bool, runtime: &Runtime, count: &mut usize) -> JResult<()> {
        if !object.is_null() && object.get_mark() != mark {
            trace!(target: "gc", "Marking object {object}");
            *count += 1;
            object.set_mark(mark);

            let class = runtime.cl.get(object.get_class());

            match &class.kind {
                ClassKind::Object(_) => {
                    let object = runtime.get_object(object.get_class(), object)?;
                    for id in object.class.fields.object_fields(false) {
                        if let StackValue::Reference(object) = object.get_field(*id) {
                            Self::mark_object(object, mark, runtime, count)?;
                        }
                    }
                }
                ClassKind::Array(class) => {
                    if let ValueType::Reference = class.component() {
                        // TODO result type that tells its the wrong type so we dont need to double request?
                        let array = runtime.get_array::<Ref>(object)?;
                        let length = array.get_length();
                        for i in 0..length {
                            let reference = array.load(i);
                            Self::mark_object(reference, mark, runtime, count)?;
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
