use std::{error::Error, fmt};

use crate::common::value_type::ValueType;

#[derive(Debug)]
pub enum AnalysisError {
    BadAnalysis,
    EmptyAST,
    InvalidTypeForOp { value_type: ValueType },
    UnknownBinding { name: String },
    BadBindingKind { name: String, is_var: bool },
    BadArguments { name: String, expected_argc: usize, actual_argc: usize },
    UnknownHiddenState { idx: usize },
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BadAnalysis => write!(f, "Invalid AAST; maybe it was manually changed?"),
            Self::EmptyAST => write!(f, "AST is empty"),
            Self::InvalidTypeForOp { value_type } => write!(f, "Type {:?} is invalid for operation", value_type),
            Self::UnknownBinding { name } => write!(f, "Unknown binding \"{name}\""),
            Self::BadBindingKind { name, is_var } => {
                if *is_var {
                    write!(f, "Binding \"{name}\" is of an unexpected kind; expected function, got variable")
                } else {
                    write!(f, "Binding \"{name}\" is of an unexpected kind; expected variable, got function")
                }
            },
            Self::BadArguments { name, expected_argc, actual_argc } => write!(f, "Function \"{name}\" expects {expected_argc} arguments, got {actual_argc} instead"),
            Self::UnknownHiddenState { idx } => write!(f, "Unknown hidden state {idx}"),
        }
    }
}

impl Error for AnalysisError { }