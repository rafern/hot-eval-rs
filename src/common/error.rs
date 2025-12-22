use std::{error::Error, fmt};

use super::{untyped_value::UntypedValue, value_type::ValueType};

#[derive(Debug)]
pub enum CommonError {
    CannotImplicitCast { from: ValueType, to: ValueType },
    CannotResolve { from: UntypedValue, to: ValueType },
    CannotMakeSigned { from: ValueType },
    BindingAlreadyExists { name: String },
    FuncSpecArgBadType { expected: ValueType, got: ValueType },
    FuncSpecArgBadParamIndex { idx: usize, count: usize },
    FuncSpecArgParamIndexConflict { idx: usize, new_type: ValueType, existing_type: ValueType },
    FuncSpecArgDiscontinuousParamMap { max_idx: usize, missing_idx: usize },
}

impl fmt::Display for CommonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CannotImplicitCast { from, to } => write!(f, "Cannot implicitly cast from {from:?} to {to:?}"),
            Self::CannotResolve { from, to } => write!(f, "Cannot resolve untyped value ({from:?}) to {to:?}"),
            Self::CannotMakeSigned { from } => write!(f, "Cannot convert {from:?} to another signed type"),
            Self::BindingAlreadyExists { name } => write!(f, "Binding \"{name}\" already exists"),
            Self::FuncSpecArgBadType { expected, got } => write!(f, "Expected function specialisation argument with type {expected:?}, got {got:?}"),
            Self::FuncSpecArgBadParamIndex { idx, count } => write!(f, "Function specialisation argument is mapped to parameter index {idx}, but there are only {count} parameters"),
            Self::FuncSpecArgParamIndexConflict { idx, new_type, existing_type } => write!(f, "Function specialisation argument is mapped to parameter index {idx} with type {new_type:?}, which is already mapped to a different type {existing_type:?}"),
            Self::FuncSpecArgDiscontinuousParamMap { max_idx, missing_idx } => write!(f, "Function specialisation arguments are mapped to a discontinuous parameter index range; expected range 0..={max_idx}, but missing index {missing_idx}"),
        }
    }
}

impl Error for CommonError { }