use std::{error::Error, fmt};

use super::{untyped_value::UntypedValue, value_type::ValueType};

#[derive(Debug)]
pub enum CommonError {
    CannotImplicitCast { from: ValueType, to: ValueType },
    CannotResolve { from: UntypedValue, to: ValueType },
    CannotMakeSigned { from: ValueType },
    BindingAlreadyExists { name: String },
    BindingFuncParamBadType { expected: ValueType, got: ValueType },
}

impl fmt::Display for CommonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CannotImplicitCast { from, to } => write!(f, "Cannot implicitly cast from {from:?} to {to:?}"),
            Self::CannotResolve { from, to } => write!(f, "Cannot resolve untyped value ({from:?}) to {to:?}"),
            Self::CannotMakeSigned { from } => write!(f, "Cannot convert {from:?} to another signed type"),
            Self::BindingAlreadyExists { name } => write!(f, "Binding \"{name}\" already exists"),
            Self::BindingFuncParamBadType { expected, got } => write!(f, "Expected binding function parameter with type {expected:?}, got {got:?}"),
        }
    }
}

impl Error for CommonError { }