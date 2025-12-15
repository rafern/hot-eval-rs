use inkwell::{context::Context, types::{FloatType, IntType}};

use crate::common::value_type::ValueType;

use super::utils::get_usize_llvm_type;

pub enum IRValueType<'ctx> {
    Int { llvm: IntType<'ctx>, is_signed: bool },
    Float { llvm: FloatType<'ctx> },
}

impl<'ctx> IRValueType<'ctx> {
    pub fn from_value_type(value_type: &ValueType, llvm_ctx: &'ctx Context) -> Self {
        match value_type {
            ValueType::U8 => Self::Int { llvm: llvm_ctx.i8_type(), is_signed: false },
            ValueType::U16 => Self::Int { llvm: llvm_ctx.i16_type(), is_signed: false },
            ValueType::U32 => Self::Int { llvm: llvm_ctx.i32_type(), is_signed: false },
            ValueType::U64 => Self::Int { llvm: llvm_ctx.i64_type(), is_signed: false },
            ValueType::USize => Self::Int { llvm: get_usize_llvm_type(llvm_ctx), is_signed: false },
            ValueType::I8 => Self::Int { llvm: llvm_ctx.i8_type(), is_signed: true },
            ValueType::I16 => Self::Int { llvm: llvm_ctx.i16_type(), is_signed: true },
            ValueType::I32 => Self::Int { llvm: llvm_ctx.i32_type(), is_signed: true },
            ValueType::I64 => Self::Int { llvm: llvm_ctx.i64_type(), is_signed: true },
            ValueType::F32 => Self::Float { llvm: llvm_ctx.f32_type() },
            ValueType::F64 => Self::Float { llvm: llvm_ctx.f64_type() },
            ValueType::Bool => Self::Int { llvm: llvm_ctx.bool_type(), is_signed: false },
        }
    }
}