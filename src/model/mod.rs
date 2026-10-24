use std::fmt::{Display, Formatter};

pub use crate::model::function::Function;
use crate::parsing::{NumericConstant, Operator};

pub mod operator;
pub mod numeric_constant;
pub mod function;

#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub enum ResultValue {
    Empty,
    Int(i64),
    Float(f64)
}

impl ResultValue {
    pub fn as_i64(&self) -> Option<i64>{
        if let ResultValue::Int(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn as_f64(&self) -> Option<f64>{
        if let ResultValue::Float(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn apply<
        F1: Fn(i64) -> Option<ResultValue>,
        F2: Fn(f64) -> Option<ResultValue>
    >(&self, f1: F1, f2: F2) -> Option<ResultValue> {
        match self {
            ResultValue::Empty => {
                None
            }
            ResultValue::Int(x) => {
                f1(*x)
            }
            ResultValue::Float(x) => {
                f2(*x)
            }
        }
    }

    pub fn apply_binary<
        F1: Fn(i64, i64) -> Option<ResultValue>,
        F2: Fn(f64, f64) -> Option<ResultValue>
    >(&self, f1: F1, f2: F2, other: &ResultValue) -> Option<ResultValue> {
        match (self, other) {
            (ResultValue::Int(x), ResultValue::Int(y)) => {
                f1(*x, *y)
            }
            (ResultValue::Float(x), ResultValue::Float(y)) => {
                f2(*x, *y)
            }
            _ => None
        }
    }
}

impl Display for ResultValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ResultValue::Empty => write!(f, ""),
            ResultValue::Int(value) => write!(f, "{}", value),
            ResultValue::Float(value) => write!(f, "{}", value),
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum ExpressionTree {
    Number(NumericConstant),
    Variable(String),
    BinaryOperator { operator: Operator, left: Box<ExpressionTree>, right: Box<ExpressionTree> },
    UnaryOperator { operator: Operator, operand: Box<ExpressionTree> },
    Declaration { left: Box<ExpressionTree>, right: Box<ExpressionTree> },
    Call { name: String, arguments: Vec<ExpressionTree> }
}