use std::{error::Error, fmt};

use crate::common::value_type::ValueType;

#[derive(Debug)]
pub enum CodegenError {
    UnexpectedBaseType,
    UnexpectedBasicValueEnum,
    UnexpectedFunctionReturnValue,
    UnknownBinding { name: String },
    BadBindingType { name: String, actual_type: ValueType, expected_type: ValueType },
    BadBindingKind { name: String, is_var: bool },
    UnknownHiddenState { idx: usize },
    SpecFailed { msg: String },
    BadSpecConst { actual_type: ValueType, expected_type: ValueType },
}

impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnexpectedBaseType => write!(f, "Unexpected base type. This is probably a bug"),
            Self::UnexpectedBasicValueEnum => write!(f, "Unexpected BasicValueEnum. This is probably a bug"),
            Self::UnexpectedFunctionReturnValue => write!(f, "Unexpected function return value. This is probably a bug"),
            Self::UnknownBinding { name } => write!(f, "Unknown binding \"{name}\""),
            Self::BadBindingType { name, actual_type, expected_type } => write!(f, "Binding \"{name}\" has an unexpected type; expected {expected_type:?}, got {actual_type:?}"),
            Self::BadBindingKind { name, is_var } => {
                if *is_var {
                    write!(f, "Binding \"{name}\" is of an unexpected kind; expected function, got variable")
                } else {
                    write!(f, "Binding \"{name}\" is of an unexpected kind; expected variable, got function")
                }
            },
            Self::UnknownHiddenState { idx } => write!(f, "Unknown hidden state {idx}"),
            Self::SpecFailed { msg } => write!(f, "Function specialization failed: {msg}"),
            Self::BadSpecConst { actual_type, expected_type } => write!(f, "Const specialization has an unexpected type; expected {expected_type:?}, got {actual_type:?}"),
        }
    }
}

impl Error for CodegenError { }