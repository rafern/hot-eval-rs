use super::value_type::ValueType;

#[derive(Debug, Clone)]
pub enum Value {
    U8 { inner: u8 },
    U16 { inner: u16 },
    U32 { inner: u32 },
    U64 { inner: u64 },
    USize { inner: usize },
    I8 { inner: i8 },
    I16 { inner: i16 },
    I32 { inner: i32 },
    I64 { inner: i64 },
    F32 { inner: f32 },
    F64 { inner: f64 },
    Bool { inner: bool },
}

impl Value {
    pub const fn get_value_type(&self) -> ValueType {
        match self {
            Value::U8 { .. } => ValueType::U8,
            Value::U16 { .. } => ValueType::U16,
            Value::U32 { .. } => ValueType::U32,
            Value::U64 { .. } => ValueType::U64,
            Value::USize { .. } => ValueType::USize,
            Value::I8 { .. } => ValueType::I8,
            Value::I16 { .. } => ValueType::I16,
            Value::I32 { .. } => ValueType::I32,
            Value::I64 { .. } => ValueType::I64,
            Value::F32 { .. } => ValueType::F32,
            Value::F64 { .. } => ValueType::F64,
            Value::Bool { .. } => ValueType::Bool,
        }
    }
}

impl From<u8> for Value { fn from(inner: u8) -> Self { Self::U8 { inner } } }
impl From<u16> for Value { fn from(inner: u16) -> Self { Self::U16 { inner } } }
impl From<u32> for Value { fn from(inner: u32) -> Self { Self::U32 { inner } } }
impl From<u64> for Value { fn from(inner: u64) -> Self { Self::U64 { inner } } }
impl From<usize> for Value { fn from(inner: usize) -> Self { Self::USize { inner } } }
impl From<i8> for Value { fn from(inner: i8) -> Self { Self::I8 { inner } } }
impl From<i16> for Value { fn from(inner: i16) -> Self { Self::I16 { inner } } }
impl From<i32> for Value { fn from(inner: i32) -> Self { Self::I32 { inner } } }
impl From<i64> for Value { fn from(inner: i64) -> Self { Self::I64 { inner } } }
impl From<f32> for Value { fn from(inner: f32) -> Self { Self::F32 { inner } } }
impl From<f64> for Value { fn from(inner: f64) -> Self { Self::F64 { inner } } }
impl From<bool> for Value { fn from(inner: bool) -> Self { Self::Bool { inner } } }