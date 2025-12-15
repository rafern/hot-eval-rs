use std::fmt::Debug;

use crate::{ast::ast_node::{BinaryOperator, UnaryOperator}, common::{untyped_value::UntypedValue, value::Value, value_type::ValueType}};

#[derive(Debug)]
pub enum FunctionArgument {
    Parameter { idx: usize, expected_type: ValueType },
    ConstArgument { value: Value },
    HiddenStateArgument { hidden_state_idx: usize, slab_value_type: ValueType, cast_to_type: Option<ValueType> },
}

#[derive(Debug)]
pub enum PackedAnalysisNodeData {
    TypedValue { value: Value },
    UntypedValue { value: UntypedValue },
    FunctionCall { args: Vec<FunctionArgument>, fn_ptr: *const () },
    UnaryOperation { operator: UnaryOperator, right_idx: usize },
    BinaryOperation { operator: BinaryOperator, left_idx: usize, right_idx: usize },
    Variable { name: String },
    Ternary { cond_idx: usize, left_idx: usize, right_idx: usize },
}

#[derive(Debug)]
pub struct PackedAnalysisNode {
    /// None means that the type is unknown, not that it hasn't been resolved
    /// yet. This means that it's an untyped value (or an operation that depends
    /// on untyped values), and that there needs to be a second phase to finish
    /// type resolution. If a type is still unknown after this second phase,
    /// then it means that there isn't enough information to resolve the type,
    /// and therefore the code doesn't compile.
    pub resolved_type: Option<ValueType>,
    pub data: PackedAnalysisNodeData,
    pub parent_idx: Option<usize>,
}