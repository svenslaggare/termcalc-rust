use std::fmt::{Display, Formatter};

use crate::model::{EnvironmentScope, ResultValue};
use crate::parsing::{BinaryOperators, NumericConstant, UnaryOperators};

pub trait NumberType {
    fn binary_operators(&self) -> &BinaryOperators;
    fn unary_operators(&self) -> &UnaryOperators;

    fn environment(&self) -> &EnvironmentScope;

    fn to_value(&self, constant: &NumericConstant) -> Result<ResultValue, NumberTypeError>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumberTypeError {
    ConstructionError,
    TooLargeValue,
    NonDecimalBase
}

impl Display for NumberTypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NumberTypeError::ConstructionError => write!(f, "Failed to construct a number"),
            NumberTypeError::TooLargeValue => write!(f, "This number is too large"),
            NumberTypeError::NonDecimalBase => write!(f, "Non decimal base")
        }
    }
}

pub mod int_type;
pub mod float_type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NumberTypeId {
    Int,
    Float
}

impl NumberTypeId {
    pub fn from_str(text: &str) -> Option<NumberTypeId> {
        match text {
            "int" => Some(NumberTypeId::Int),
            "float" => Some(NumberTypeId::Float),
            _ => None
        }
    }
}