use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum HotEvalParserError {
    BadLiteral { type_str: &'static str },
}

impl Display for HotEvalParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", match self {
            Self::BadLiteral { type_str } => format!("Invalid literal for type {}", type_str),
        })
    }
}

pub struct UnevaluatedNumberLiteral {
    pub string: String,
    pub radix: u32,
}

/*pub fn slice_before_end<'a>(value: &'a str, amount: usize) -> &'a str {
    &value[0..value.len()-amount]
}*/

pub fn slice_after_begin<'a>(value: &'a str, amount: usize) -> &'a str {
    &value[amount..value.len()]
}

pub fn filter_str_chars(s: &str, c: char) -> String {
    s.chars().filter(|&o| o != c).collect()
}

pub fn make_bad_literal_error<'a>(type_str: &'static str) -> HotEvalParserError {
    HotEvalParserError::BadLiteral { type_str }
}