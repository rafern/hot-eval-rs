use super::error::CommonError;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ValueType {
    U8,
    U16,
    U32,
    U64,
    USize,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Bool,
}

impl ValueType {
    pub const fn get_implicit_cast_priority(&self) -> u32 {
        match self {
            Self::Bool => 0,
            Self::U8 => 1,
            Self::I8 => 2,
            Self::U16 => 3,
            Self::I16 => 4,
            Self::U32 => 5,
            Self::I32 => 6,
            Self::U64 => 7,
            Self::I64 => 8,
            Self::USize => 9,
            Self::F32 => 10,
            Self::F64 => 11,
        }
    }

    pub const fn is_signed(&self) -> bool {
        match self {
            Self::Bool | Self::U8 | Self::U16 | Self::U32 | Self::U64 | Self::USize => false,
            Self::I8 | Self::I16 | Self::I32 | Self::I64 | Self::F32 | Self::F64 => true,
        }
    }

    pub const fn is_float(&self) -> bool {
        match self {
            Self::Bool | Self::U8 | Self::U16 | Self::U32 | Self::U64 | Self::USize | Self::I8 | Self::I16 | Self::I32 | Self::I64 => false,
            Self::F32 | Self::F64 => true,
        }
    }

    pub const fn can_implicit_cast_to(&self, to: &Self) -> bool {
        // can implicitly cast:
        // - low-priority unsigned/signed -> high-priority signed
        // - low-priority unsigned -> high-priority unsigned
        // all these should be safe except for floats which will inherently
        // lose precision for high values, but is expected behaviour in most
        // languages (e.g. i32 -> f32, or u64 -> f32)
        (self.get_implicit_cast_priority() < to.get_implicit_cast_priority()) &&
            (to.is_signed() || !self.is_signed())
    }

    pub fn widen(mut a: Self, mut b: Self) -> Result<Self, CommonError> {
        if a == b { return Ok(a) }

        if a.get_implicit_cast_priority() > b.get_implicit_cast_priority() {
            (a, b) = (b, a);
        }

        if a.can_implicit_cast_to(&b) {
            Ok(b)
        } else {
            Err(CommonError::CannotImplicitCast { from: a, to: b }.into())
        }
    }

    pub fn widen_optional_greedy(a: Option<Self>, b: Option<Self>) -> Result<Option<Self>, CommonError> {
        Ok(if a.is_some() {
            if let Some(b) = b {
                Some(Self::widen(unsafe { a.unwrap_unchecked() }, b)?)
            } else {
                a
            }
        } else {
            b
        })
    }

    pub fn widen_optional_non_greedy(a: Option<Self>, b: Option<Self>) -> Result<Option<Self>, CommonError> {
        Ok(if let Some(a) = a {
            if let Some(b) = b {
                Some(Self::widen(a, b)?)
            } else {
                None
            }
        } else {
            None
        })
    }

    pub fn to_signed(&self) -> Result<Self, CommonError> {
        Ok(match self {
            Self::Bool |
            Self::USize => return Err(CommonError::CannotMakeSigned { from: self.clone() }),
            Self::U8 => Self::I8,
            Self::U16 => Self::I16,
            Self::U32 => Self::I32,
            Self::U64 => Self::I64,
            _ => self.clone(),
        })
    }

    pub fn to_signed_optional(t: Option<Self>) -> Result<Option<Self>, CommonError> {
        Ok(if let Some(t) = t {
            Some(t.to_signed()?)
        } else {
            None
        })
    }
}