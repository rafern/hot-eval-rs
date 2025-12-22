use std::ffi::c_void;

use super::{error::CommonError, ir_const::IRConst, value::Value, value_type::ValueType};

pub type FnPointer = *const c_void;

#[derive(Clone, Copy)]
pub enum FnSpecCallArg {
    MappedArgument { param_idx: usize },
    ConstArgument { value: Value },
    HiddenStateArgument { hidden_state_idx: usize, cast_to_type: Option<ValueType> },
}

pub enum FnSpecChoice {
    Call { fn_ptr: FnPointer, args: Box<[FnSpecCallArg]> },
    Const { value: Value },
}

pub struct FnSpecHints {
    pub consts: Box<[Option<IRConst>]>,
}

pub type BindingFuncParams = Box<[ValueType]>;
pub type FnSpec<'table> = Box<dyn Fn(FnSpecHints) -> Result<FnSpecChoice, String> + 'table>;

pub enum Binding<'table> {
    Const { value: Value },
    Variable { value_type: ValueType },
    Function { ret_type: ValueType, params: BindingFuncParams, fn_spec: FnSpec<'table> },
}

impl FnSpecCallArg {
    pub fn from_hidden_state(hidden_state_idx: usize) -> Self {
        Self::HiddenStateArgument { hidden_state_idx, cast_to_type: None }
    }

    pub fn from_hidden_state_cast(hidden_state_idx: usize, value_type: ValueType) -> Self {
        Self::HiddenStateArgument { hidden_state_idx, cast_to_type: Some(value_type) }
    }

    pub fn guard<T: ToBFPValueType>(&self, params: &BindingFuncParams) -> Result<(), CommonError> {
        let got = match *self {
            Self::MappedArgument { param_idx } => {
                let count = params.len();
                if param_idx >= count {
                    return Err(CommonError::FuncSpecArgBadParamIndex { idx: param_idx, count })
                }

                params[param_idx]
            },
            Self::ConstArgument { ref value } => {
                value.get_value_type()
            },
            Self::HiddenStateArgument { hidden_state_idx: _, cast_to_type } => {
                if let Some(value_type) = cast_to_type {
                    value_type
                } else {
                    return Ok(())
                }
            },
        };

        let expected = T::to_bfp_value_type();
        if got == expected {
            Ok(())
        } else {
            Err(CommonError::FuncSpecArgBadType { expected, got })
        }
    }
}

impl From<Value> for FnSpecCallArg { fn from(value: Value) -> Self { Self::ConstArgument { value } } }
impl From<u8> for FnSpecCallArg { fn from(x: u8) -> Self { Self::ConstArgument { value: x.into() } } }
impl From<u16> for FnSpecCallArg { fn from(x: u16) -> Self { Self::ConstArgument { value: x.into() } } }
impl From<u32> for FnSpecCallArg { fn from(x: u32) -> Self { Self::ConstArgument { value: x.into() } } }
impl From<u64> for FnSpecCallArg { fn from(x: u64) -> Self { Self::ConstArgument { value: x.into() } } }
impl From<usize> for FnSpecCallArg { fn from(x: usize) -> Self { Self::ConstArgument { value: x.into() } } }
impl From<i8> for FnSpecCallArg { fn from(x: i8) -> Self { Self::ConstArgument { value: x.into() } } }
impl From<i16> for FnSpecCallArg { fn from(x: i16) -> Self { Self::ConstArgument { value: x.into() } } }
impl From<i32> for FnSpecCallArg { fn from(x: i32) -> Self { Self::ConstArgument { value: x.into() } } }
impl From<i64> for FnSpecCallArg { fn from(x: i64) -> Self { Self::ConstArgument { value: x.into() } } }
impl From<f32> for FnSpecCallArg { fn from(x: f32) -> Self { Self::ConstArgument { value: x.into() } } }
impl From<f64> for FnSpecCallArg { fn from(x: f64) -> Self { Self::ConstArgument { value: x.into() } } }
impl From<bool> for FnSpecCallArg { fn from(x: bool) -> Self { Self::ConstArgument { value: x.into() } } }

pub trait ToBFPValueType {
    fn to_bfp_value_type() -> ValueType;
}

impl ToBFPValueType for u8 { fn to_bfp_value_type() -> ValueType { ValueType::U8 } }
impl ToBFPValueType for u16 { fn to_bfp_value_type() -> ValueType { ValueType::U16 } }
impl ToBFPValueType for u32 { fn to_bfp_value_type() -> ValueType { ValueType::U32 } }
impl ToBFPValueType for u64 { fn to_bfp_value_type() -> ValueType { ValueType::U64 } }
impl ToBFPValueType for usize { fn to_bfp_value_type() -> ValueType { ValueType::USize } }
impl ToBFPValueType for i8 { fn to_bfp_value_type() -> ValueType { ValueType::I8 } }
impl ToBFPValueType for i16 { fn to_bfp_value_type() -> ValueType { ValueType::I16 } }
impl ToBFPValueType for i32 { fn to_bfp_value_type() -> ValueType { ValueType::I32 } }
impl ToBFPValueType for i64 { fn to_bfp_value_type() -> ValueType { ValueType::I64 } }
impl ToBFPValueType for f32 { fn to_bfp_value_type() -> ValueType { ValueType::F32 } }
impl ToBFPValueType for f64 { fn to_bfp_value_type() -> ValueType { ValueType::F64 } }
impl ToBFPValueType for bool { fn to_bfp_value_type() -> ValueType { ValueType::Bool } }
impl<T> ToBFPValueType for *const T { fn to_bfp_value_type() -> ValueType { ValueType::USize } }
impl<T> ToBFPValueType for *mut T { fn to_bfp_value_type() -> ValueType { ValueType::USize } }
impl<T> ToBFPValueType for &T { fn to_bfp_value_type() -> ValueType { ValueType::USize } }
impl<T> ToBFPValueType for &mut T { fn to_bfp_value_type() -> ValueType { ValueType::USize } }