#![feature(generic_const_exprs)]
#![feature(drain_filter)]
#![feature(array_try_from_fn)]
#![feature(box_syntax)]
use crate::class::{Class, ClassKind, Object};
use crate::gc::GarbageCollector;
use crate::object::Field;
use crate::object::Ref;
use crate::object::{Method, MethodIdentifier, NativeCode};
use crate::reader::{ClassConst, ClassInfo, ConstPtr, ConstantPool, FieldConst, MethodConst, StrParse, ValueDesc, BinaryName};
use ahash::HashMap;

use std::sync::RwLock;
use tracing::{debug, info};

use crate::class_loader::ClassLoader;
use crate::error::{JError, JResult};
use rvm_core::Id;

pub mod class;
mod class_loader;
pub mod convert;
pub mod error;
pub mod executor;
pub mod gc;
pub mod object;
pub mod reader;

#[cfg(feature = "native")]
pub mod native;

pub struct Runtime {
    pub cl: ClassLoader,
    pub gc: RwLock<GarbageCollector>,
    pub native_methods: HashMap<(String, MethodIdentifier), NativeCode>,
}

impl Runtime {
    pub fn new() -> Runtime {
        Runtime {
            cl: ClassLoader::new(),
            gc: RwLock::new(GarbageCollector::new()),
            native_methods: Default::default(),
        }
    }


    pub fn resolve_class(
        &self,
        from: Id<Class>,
        class_ptr: ConstPtr<ClassConst>,
    ) -> JResult<Id<Class>> {
        let desc = {
            // very important to free the class lock if its gonna get resolved
            let class = self.cl.get_obj_class(from);
            let class_const = class_ptr.get(&class.cp);
            if let Some(value) = class_const.link.get() {
                // symbolic link fast af
                return Ok(value);
            }

            let desc1 = class_const.name.get(&class.cp).as_str().replace('/', ".");
            info!("{desc1}");
            BinaryName::parse(&desc1)
        };

        debug!(target: "resolve", "Resolving class \"{:?}\"", desc);
        let id = self.cl.get_class_id(&desc);

        // Link the value
        let class = self.cl.get_obj_class(from);
        class_ptr.get(&class.cp).link.replace(Some(id));
        Ok(id)
    }

    pub fn resolve_field(
        &self,
        from: Id<Class>,
        field_ptr: ConstPtr<FieldConst>,
    ) -> JResult<(Id<Class>, Id<Field>)> {
        let from_class = self.cl.get_obj_class(from);
        let field_const = field_ptr.get(&from_class.cp);
        if let Some(value) = field_const.link.get() {
            let class_id = field_const
                .class
                .get(&from_class.cp)
                .link
                .get()
                .expect("Field linked to a non linked class");
            return Ok((class_id, value));
        }

        let name_and_type = field_const.name_and_type.get(&from_class.cp);
        let name = name_and_type.name.get(&from_class.cp).to_string();
        let class_ptr = field_const.class;
        //let descriptor = name_and_type.descriptor.get(&class.cp).as_str();

        debug!(target: "resolve", "Resolving field \"{}\"", name);
        // to allow for loading incase it gets defined
        drop(from_class);
        let class_id = self.resolve_class(from, class_ptr)?;

        let class = self.cl.get_obj_class(class_id);
        if let Some(id) = class.fields.get_id(&name) {
            let from_class = self.cl.get_obj_class(from);
            field_ptr.get(&from_class.cp).link.replace(Some(id));
            return Ok((class_id, id));
        }
        //let class_id = self.class.get(cp).get_id(cp, runtime)?;
        //         let name_and_type = self.name_and_type.get(cp);
        //         let name = name_and_type.name.get(cp).as_str();
        //
        //         let id = runtime.get_field(class_id, name)?;
        //         self.link.replace(Some(id));
        //         Ok(id)
        //
        //         debug!(target: "resolve", "Resolving field \"{}\"", field);
        //         let class = self.class_loader.get(from);
        //         match &class.kind {
        //             ClassKind::Object(object) => {
        //                 if let Some(value) = object.fields.get_id(field) {
        //                     return Ok(value)
        //                 }
        //             }
        //             _ => {
        //                 panic!("Expected object but found other")
        //             }
        //         }
        panic!("Failed to resolve field. SUPER not yet supported")
    }

    pub fn resolve_method(
        &self,
        from: Id<Class>,
        method_ptr: ConstPtr<MethodConst>,
    ) -> JResult<(Id<Class>, Id<Method>)> {
        let from_class = self.cl.get_obj_class(from);
        let method_const = method_ptr.get(&from_class.cp);
        if let Some(value) = method_const.link.get() {
            let class_id = method_const
                .class
                .get(&from_class.cp)
                .link
                .get()
                .expect("Method linked to a non linked class");
            return Ok((class_id, value));
        }

        let name_and_type = method_const.name_and_type.get(&from_class.cp);
        let name = MethodIdentifier::new(name_and_type, &from_class.cp);
        let class_ptr = method_const.class;
        //let descriptor = name_and_type.descriptor.get(&class.cp).as_str();

        debug!(target: "resolve", "Resolving method \"{:?}\"", name);

        // to allow for loading incase it gets defined
        drop(from_class);
        let class_id = self.resolve_class(from, class_ptr)?;

        let class = self.cl.get_obj_class(class_id);
        if let Some(id) = class.methods.get_id(&name) {
            let from_class = self.cl.get_obj_class(from);
            method_ptr.get(&from_class.cp).link.replace(Some(id));
            return Ok((class_id, id));
        }
        //      if let Some(value) = self.link.get() {
        // 			return Ok(value);
        // 		}
        //
        // 		let class_id = self.class.get(cp).get_id(cp, runtime)?;
        // 		let name_and_type = self.name_and_type.get(cp);
        // 		debug!(target: "resolve", "Resolving method \"{}\"", name_and_type.name.get(cp).as_str());
        // 		let identifier = MethodIdentifier::new(name_and_type, cp);
        //
        // 		let id = runtime.get_method(class_id, &identifier)?;
        // 		self.link.replace(Some(id));
        // 		Ok(id)
        // debug!(target: "resolve", "Resolving method \"{method:?}\"");
        //
        //         let class = self.class_loader.get(class_id);
        //         match &class.kind {
        //             ClassKind::Object(object) => {
        //                 if let Some(value) = object.methods.get_id(method) {
        //                     return Ok(value);
        //                 }
        //             }
        //             _ => {
        //                 panic!("Expected object but found other")
        //             }
        //         }

        panic!("Failed to resolve method. SUPER not yet supported")
    }
}
