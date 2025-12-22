use std::fmt::{Debug, Formatter};

use crate::{ast::ast_node::{BinaryOperator, UnaryOperator}, common::{binding::FnSpec, untyped_value::UntypedValue, value::Value, value_type::ValueType}};

#[derive(Debug)]
pub enum FunctionArgument {
    Parameter { idx: usize, expected_type: ValueType },
    ConstArgument { value: Value },
    HiddenStateArgument { hidden_state_idx: usize, slab_value_type: ValueType, cast_to_type: Option<ValueType> },
}

pub enum PackedAnalysisNodeData<'table> {
    TypedValue { value: Value },
    UntypedValue { value: UntypedValue },
    FunctionCall { args: Vec<FunctionArgument>, fn_spec: &'table FnSpec<'table> },
    UnaryOperation { operator: UnaryOperator, right_idx: usize },
    BinaryOperation { operator: BinaryOperator, left_idx: usize, right_idx: usize },
    Variable { name: String },
    Ternary { cond_idx: usize, left_idx: usize, right_idx: usize },
}

#[derive(Debug)]
pub struct PackedAnalysisNode<'table> {
    /// None means that the type is unknown, not that it hasn't been resolved
    /// yet. This means that it's an untyped value (or an operation that depends
    /// on untyped values), and that there needs to be a second phase to finish
    /// type resolution. If a type is still unknown after this second phase,
    /// then it means that there isn't enough information to resolve the type,
    /// and therefore the code doesn't compile.
    pub resolved_type: Option<ValueType>,
    pub data: PackedAnalysisNodeData<'table>,
    pub parent_idx: Option<usize>,
}

// XXX: can't derive debug due to fn_spec
impl Debug for PackedAnalysisNodeData<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            PackedAnalysisNodeData::TypedValue { value } => {
                f.debug_struct("PackedAnalysisNodeData::TypedValue")
                 .field("value", value)
                 .finish()
            },
            PackedAnalysisNodeData::UntypedValue { value } => {
                f.debug_struct("PackedAnalysisNodeData::UntypedValue")
                 .field("value", value)
                 .finish()
            },
            PackedAnalysisNodeData::FunctionCall { args, fn_spec: _ } => {
                f.debug_struct("PackedAnalysisNodeData::FunctionCall")
                 .field("args", args)
                 .finish_non_exhaustive()
            },
            PackedAnalysisNodeData::UnaryOperation { operator, right_idx } => {
                f.debug_struct("PackedAnalysisNodeData::UnaryOperation")
                 .field("operator", operator)
                 .field("right_idx", right_idx)
                 .finish()
            },
            PackedAnalysisNodeData::BinaryOperation { operator, left_idx, right_idx } => {
                f.debug_struct("PackedAnalysisNodeData::BinaryOperation")
                 .field("operator", operator)
                 .field("left_idx", left_idx)
                 .field("right_idx", right_idx)
                 .finish()
            },
            PackedAnalysisNodeData::Variable { name } => {
                f.debug_struct("PackedAnalysisNodeData::Variable")
                 .field("name", name)
                 .finish()
            },
            PackedAnalysisNodeData::Ternary { cond_idx, left_idx, right_idx } => {
                f.debug_struct("PackedAnalysisNodeData::Ternary")
                 .field("cond_idx", cond_idx)
                 .field("left_idx", left_idx)
                 .field("right_idx", right_idx)
                 .finish()
            },
        }
    }
}