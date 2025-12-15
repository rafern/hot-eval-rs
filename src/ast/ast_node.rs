use std::error::Error;

use crate::common::{untyped_value::UntypedValue, value::Value};

use super::parser;

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Negate,
    LogicalNot,
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Mul,
    Div,
    Mod,
    Add,
    Sub,
    Equals,
    NotEquals,
    LesserThanEquals,
    GreaterThanEquals,
    LesserThan,
    GreaterThan,
    LogicalAnd,
    LogicalOr,
}

#[derive(Debug)]
pub enum Expression {
    TypedValue { value: Value },
    UntypedValue { value: UntypedValue },
    FunctionCall { name: String, arguments: Vec<Expression> },
    UnaryOperation { operator: UnaryOperator, right: Box<Expression> },
    BinaryOperation { operator: BinaryOperator, left: Box<Expression>, right: Box<Expression> },
    Binding { name: String },
    Ternary { cond: Box<Expression>, left: Box<Expression>, right: Box<Expression> },
}

impl Expression {
    pub fn from_src<'src>(source: &'src str) -> Result<Expression, Box<dyn Error + 'src>> {
        Ok(parser::ExpressionParser::new().parse(source)?)
    }
}