use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType};
use rvm_core::{Kind, MethodDesc};

pub fn kind_ty(kind: Kind, ctx: &Context) -> BasicTypeEnum {
    match kind {
        Kind::Boolean => ctx.bool_type().as_basic_type_enum(),
        Kind::Byte => ctx.i8_type().as_basic_type_enum(),
        Kind::Short => ctx.i16_type().as_basic_type_enum(),
        Kind::Int => ctx.i32_type().as_basic_type_enum(),
        Kind::Long => ctx.i64_type().as_basic_type_enum(),
        Kind::Char => ctx.i16_type().as_basic_type_enum(),
        Kind::Float => ctx.f32_type().as_basic_type_enum(),
        Kind::Double => ctx.f64_type().as_basic_type_enum(),
        Kind::Reference => ctx.i64_type().as_basic_type_enum(),
    }
}

pub fn desc_ty<'ctx>(desc: &MethodDesc, ctx: &'ctx Context) -> FunctionType<'ctx> {
    let param_types: Vec<BasicMetadataTypeEnum> = desc
        .parameters
        .iter()
        .map(|v| BasicMetadataTypeEnum::from(kind_ty(v.kind(), ctx)))
        .collect();

    match &desc.ret {
        None => {
            ctx.void_type().fn_type(&param_types, false)
        }
        Some(ty) => {
            kind_ty(ty.kind(), ctx).fn_type(&param_types, false)
        }
    }
}