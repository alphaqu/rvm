#![feature(hash_drain_filter)]

mod root;

use crate::class::Class;
use crate::object::{Ref, ObjectData, HEADER_SIZE, Value, ValueType};
use crate::{ClassKind, ClassLoader, JResult, Runtime};
use ahash::HashSet;
use mimalloc::MiMalloc;
use rvm_core::{Id};
use std::alloc::dealloc;
use tracing::{info, trace};
use crate::executor::StackValue;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub struct GarbageCollector {
    root_objects: HashSet<Ref>,
    objects: HashSet<Ref>,
    mark: bool,
}

impl GarbageCollector {
    pub fn new() -> GarbageCollector {
        GarbageCollector {
            root_objects: Default::default(),
            objects: Default::default(),
            mark: false,
        }
    }

    pub unsafe fn alloc(&mut self, class_id: Id<Class>, size: usize) -> Ref {
        let object = unsafe { Ref::new(self.mark, class_id.idx(), size) };
        self.root_objects.insert(object);
        self.objects.insert(object);
        object
    }

    pub fn add_root(&mut self, object: Ref) {
        self.root_objects.insert(object);
    }

    pub fn remove_root(&mut self, object: Ref) {
        self.root_objects.remove(&object);
    }

    pub fn gc(&mut self, runtime: &Runtime)  -> JResult<()> {
	    info!("Garbage collecting");
        self.mark = !self.mark;

        // mark
        for object in &self.root_objects {
            Self::mark_object(*object, self.mark, runtime)?;
        }

        // sweep
        self.objects.retain(|v| {
            if v.get_mark() == self.mark {
                true
            } else {
                unsafe {
                    info!("Deallocated {v:?}");
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

    fn mark_object(object: Ref, mark: bool, runtime: &Runtime) -> JResult<()> {
        if object.get_mark() != mark {
            object.set_mark(mark);
	        trace!("Marked {}", object);
            let class = runtime.cl.get(object.get_class());

	        match &class.kind {
		        ClassKind::Object(_) => {
			        let object = runtime.get_object(object.get_class(), object)?;
			        for id in object.class.fields.object_fields(false) {
				        if let StackValue::Reference(object) = object.get_field(*id) {
					        if !object.is_null() {
						        Self::mark_object(object, mark, runtime)?;
					        }
				        }
			        }
		        }
		        ClassKind::Array(class) => {
			        if let ValueType::Reference = class.component() {

				        // TODO result type that tells its the wrong type so we dont need to double request?
				        let array = runtime.get_array::<Ref>(object)?;
				        let length =  array.get_length();
				        for i in 0..length {
					        let reference = array.load(i);
					        if !reference.is_null() {
						        Self::mark_object(reference, mark, runtime)?;
					        }
				        }
			        }
		        }
		        _ => {}
	        }
        }
	    Ok(())
    }
}
