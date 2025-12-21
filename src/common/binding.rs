use super::{error::CommonError, value::Value, value_type::ValueType};

pub enum BindingFunctionParameter {
    Parameter { value_type: ValueType },
    ConstArgument { value: Value },
    HiddenStateArgument { hidden_state_idx: usize, cast_to_type: Option<ValueType> },
}

pub enum Binding {
    Const { value: Value },
    Variable { value_type: ValueType },
    Function { ret_type: ValueType, params: Vec<BindingFunctionParameter>, fn_ptr: *const () },
}

impl BindingFunctionParameter {
    pub fn from_hidden_state(hidden_state_idx: usize) -> Self {
        Self::HiddenStateArgument { hidden_state_idx, cast_to_type: None }
    }

    pub fn from_hidden_state_cast(hidden_state_idx: usize, value_type: ValueType) -> Self {
        Self::HiddenStateArgument { hidden_state_idx, cast_to_type: Some(value_type) }
    }

    pub fn guard<T: ToBFPValueType>(self) -> Result<Self, CommonError> {
        let got = match self {
            BindingFunctionParameter::Parameter { value_type } => {
                value_type
            },
            BindingFunctionParameter::ConstArgument { ref value } => {
                value.get_value_type()
            },
            BindingFunctionParameter::HiddenStateArgument { hidden_state_idx: _, cast_to_type } => {
                if let Some(value_type) = cast_to_type {
                    value_type
                } else {
                    return Ok(self)
                }
            },
        };

        let expected = T::to_bfp_value_type();
        if got == expected {
            Ok(self)
        } else {
            Err(CommonError::BindingFuncParamBadType { expected, got })
        }
    }
}

impl From<ValueType> for BindingFunctionParameter { fn from(value_type: ValueType) -> Self { BindingFunctionParameter::Parameter { value_type } } }
impl From<Value> for BindingFunctionParameter { fn from(value: Value) -> Self { BindingFunctionParameter::ConstArgument { value } } }
impl From<u8> for BindingFunctionParameter { fn from(x: u8) -> Self { BindingFunctionParameter::ConstArgument { value: x.into() } } }
impl From<u16> for BindingFunctionParameter { fn from(x: u16) -> Self { BindingFunctionParameter::ConstArgument { value: x.into() } } }
impl From<u32> for BindingFunctionParameter { fn from(x: u32) -> Self { BindingFunctionParameter::ConstArgument { value: x.into() } } }
impl From<u64> for BindingFunctionParameter { fn from(x: u64) -> Self { BindingFunctionParameter::ConstArgument { value: x.into() } } }
impl From<usize> for BindingFunctionParameter { fn from(x: usize) -> Self { BindingFunctionParameter::ConstArgument { value: x.into() } } }
impl From<i8> for BindingFunctionParameter { fn from(x: i8) -> Self { BindingFunctionParameter::ConstArgument { value: x.into() } } }
impl From<i16> for BindingFunctionParameter { fn from(x: i16) -> Self { BindingFunctionParameter::ConstArgument { value: x.into() } } }
impl From<i32> for BindingFunctionParameter { fn from(x: i32) -> Self { BindingFunctionParameter::ConstArgument { value: x.into() } } }
impl From<i64> for BindingFunctionParameter { fn from(x: i64) -> Self { BindingFunctionParameter::ConstArgument { value: x.into() } } }
impl From<f32> for BindingFunctionParameter { fn from(x: f32) -> Self { BindingFunctionParameter::ConstArgument { value: x.into() } } }
impl From<f64> for BindingFunctionParameter { fn from(x: f64) -> Self { BindingFunctionParameter::ConstArgument { value: x.into() } } }
impl From<bool> for BindingFunctionParameter { fn from(x: bool) -> Self { BindingFunctionParameter::ConstArgument { value: x.into() } } }

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