use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::Read;

use time::macros::format_description;
use tracing::{debug, event, info, Level, span};
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::class_reader::{AttributeInfo, ClassInfo, ConstantInfo, IResult};
use crate::stack::{Stack, StackValue};
use crate::variable::Locals;

mod class_reader;
mod consts;
mod code;
mod stack;
mod variable;
mod interop;
mod reader;


pub type MethodId = u32;
pub type ClassId = u32;

fn main() {
    let mut result = File::open("./Main.class").unwrap();
    let mut data = Vec::new();
    result.read_to_end(&mut data);


    let timer = UtcTime::new(format_description!(
        "[hour]:[minute]:[second].[subsecond digits:3]"
    ));
    let format = tracing_subscriber::fmt::format()
        .with_timer(timer)
        .compact();
    let fmt_layer = tracing_subscriber::fmt::layer().event_format(format);
    tracing_subscriber::registry()
        .with(fmt_layer)
        .init();



    if let Ok((x, info)) = ClassInfo::parse(&data) {

        for method in info.methods {
            for x in method.attribute_info {
                match x {
                    AttributeInfo::CodeAttribute { code } => {
                        let mut stack = Stack::new(code.max_stack);
                        let mut locals = Locals::new(code.max_locals);
                        if let Some(ConstantInfo::UTF8 { text }) = info.constant_pool.get( method.name_index) {
                            info!("Running {}", text);
                        }
                        for x in code.code {
                            x.run(&mut stack, &mut locals, &info.constant_pool);
                        }
                    }
                    _ => {}
                }
            }
        }
    };
}

pub struct RVM {
    methods: HashMap<String, Method>,
    classes: HashMap<String, Class>,
}

impl RVM {
    pub fn load(&mut self, info: ClassInfo) {}
}

pub struct Method {}

pub struct Class {}
