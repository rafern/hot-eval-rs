use std::error::Error;

use super::{error::CommonError, value::Value, value_type::ValueType};

#[derive(Debug, Clone)]
pub enum UntypedValue {
    Float { inner: f64 },
    Integer { inner: u64 },
}

impl UntypedValue {
    pub fn get_resolved_value(&self, resolved_type: ValueType) -> Result<Value, Box<dyn Error>> {
        Ok(match self {
            UntypedValue::Float { inner } => {
                match resolved_type {
                    ValueType::F32 => Value::F32 { inner: *inner as f32 },
                    ValueType::F64 => Value::F64 { inner: *inner },
                    _ => return Err(Box::new(CommonError::CannotResolve { from: self.clone(), to: resolved_type })),
                }
            },
            UntypedValue::Integer { inner } => {
                match resolved_type {
                    ValueType::U8 => Value::U8 { inner: (*inner).try_into()? },
                    ValueType::U16 => Value::U16 { inner: (*inner).try_into()? },
                    ValueType::U32 => Value::U32 { inner: (*inner).try_into()? },
                    ValueType::U64 => Value::U64 { inner: *inner },
                    ValueType::USize => Value::USize { inner: (*inner).try_into()? },
                    ValueType::I8 => Value::I8 { inner: (*inner).try_into()? },
                    ValueType::I16 => Value::I16 { inner: (*inner).try_into()? },
                    ValueType::I32 => Value::I32 { inner: (*inner).try_into()? },
                    ValueType::I64 => Value::I64 { inner: (*inner).try_into()? },
                    ValueType::F32 => Value::F32 { inner: *inner as f32 },
                    ValueType::F64 => Value::F64 { inner: *inner as f64 },
                    _ => return Err(Box::new(CommonError::CannotResolve { from: self.clone(), to: resolved_type })),
                }
            },
        })
    }
}