use inkwell::{context::Context, types::{BasicMetadataTypeEnum, FunctionType, IntType}};

use crate::{codegen::ir_value_type::IRValueType, common::value_type::ValueType};

pub fn get_usize_llvm_type<'ctx>(llvm_ctx: &'ctx Context) -> IntType<'ctx> {
    match size_of::<usize>() {
        1 => llvm_ctx.i8_type(),
        2 => llvm_ctx.i16_type(),
        4 => llvm_ctx.i32_type(),
        8 => llvm_ctx.i64_type(),
        16 => llvm_ctx.i128_type(),
        // XXX i don't think this will ever be needed, but you never know...
        bytes => llvm_ctx.custom_width_int_type((bytes * 8) as u32),
    }
}

pub fn get_fn_llvm_type<'ctx>(llvm_ctx: &'ctx Context, ret_type: ValueType, arg_types: Vec<ValueType>) -> FunctionType<'ctx> {
    let mut param_types = Vec::<BasicMetadataTypeEnum<'ctx>>::new();
    for arg_type in arg_types {
        param_types.push(match arg_type {
            ValueType::U8 |
            ValueType::I8 => llvm_ctx.i8_type().into(),
            ValueType::U16 |
            ValueType::I16 => llvm_ctx.i16_type().into(),
            ValueType::U32 |
            ValueType::I32 => llvm_ctx.i32_type().into(),
            ValueType::U64 |
            ValueType::I64 => llvm_ctx.i64_type().into(),
            ValueType::USize => get_usize_llvm_type(llvm_ctx).into(),
            ValueType::F32 => llvm_ctx.f32_type().into(),
            ValueType::F64 => llvm_ctx.f64_type().into(),
            ValueType::Bool => llvm_ctx.bool_type().into(),
        })
    }

    match IRValueType::from_value_type(&ret_type, llvm_ctx) {
        IRValueType::Int { llvm, is_signed: _ } => {
            llvm.fn_type(param_types.iter().as_slice(), false)
        },
        IRValueType::Float { llvm } => {
            llvm.fn_type(param_types.iter().as_slice(), false)
        },
    }
}