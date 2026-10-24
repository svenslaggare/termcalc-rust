use std::collections::{HashMap};
use std::fmt::Formatter;

use crate::model::ResultValue;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Operator {
    Single(char),
    Dual(char, char)
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::Single(x) => write!(f, "{}", x),
            Operator::Dual(x, y) => write!(f, "{}{}", x, y)
        }
    }
}

pub struct BinaryOperator {
    pub precedence: i32,
    pub callback: Box<dyn Fn(ResultValue, ResultValue) -> Option<ResultValue>>
}

impl BinaryOperator {
    pub fn new(
        precedence: i32,
        callback: Box<dyn Fn(ResultValue, ResultValue) -> Option<ResultValue>>
    )-> BinaryOperator {
        BinaryOperator {
            precedence,
            callback
        }
    }
}

pub struct BinaryOperators {
    operators: HashMap<Operator, BinaryOperator>
}

impl BinaryOperators {
    pub fn new(operators: Vec<(Operator, BinaryOperator)>) -> BinaryOperators {
        BinaryOperators {
            operators: HashMap::from_iter(operators.into_iter())
        }
    }

    pub fn get(&self, op: &Operator) -> Option<&BinaryOperator> {
        self.operators.get(op)
    }
}

pub struct UnaryOperator {
    pub callback: Box<dyn Fn(ResultValue) -> Option<ResultValue>>
}

impl UnaryOperator {
    pub fn new(
        callback: Box<dyn Fn(ResultValue) -> Option<ResultValue>>
    )-> UnaryOperator {
        UnaryOperator {
            callback
        }
    }
}

pub struct UnaryOperators {
    operators: HashMap<Operator, UnaryOperator>
}

impl UnaryOperators {
    pub fn new(operators: Vec<(Operator, UnaryOperator)>) -> UnaryOperators {
        UnaryOperators {
            operators: HashMap::from_iter(operators.into_iter())
        }
    }

    pub fn exists(&self, op: &Operator) -> bool {
        self.operators.contains_key(op)
    }

    pub fn get(&self, op: &Operator) -> Option<&UnaryOperator> {
        self.operators.get(op)
    }
}