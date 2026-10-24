use std::fmt::{Display, Formatter};
use std::rc::Rc;
use fnv::FnvHashMap;

use crate::parsing::{NumericConstant, Operator};

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

#[derive(Clone)]
pub enum Function {
    Builtin {
        parameters: Vec<String>,
        description: String,
        callback: Rc<dyn Fn(&[ResultValue]) -> Option<ResultValue>>
    },
    User {
        parameters: Vec<String>,
        body: ExpressionTree
    }
}

impl Function {
    pub fn parameters(&self) -> &Vec<String> {
        match self {
            Function::Builtin { parameters, .. } => parameters,
            Function::User { parameters, .. } => parameters
        }
    }
}

#[derive(Clone)]
pub struct EnvironmentScope {
    variables: FnvHashMap<String, ResultValue>,
    functions: FnvHashMap<String, Function>
}

impl EnvironmentScope {
    pub fn new(variables: Vec<(String, ResultValue)>, functions: Vec<(String, Function)>) -> EnvironmentScope {
        EnvironmentScope {
            variables: FnvHashMap::from_iter(variables),
            functions: FnvHashMap::from_iter(functions)
        }
    }

    pub fn get_variable(&self, name: &str) -> Option<ResultValue> {
        self.variables.get(name).cloned()
    }

    pub fn variables(&self) -> impl Iterator<Item=(&String, &ResultValue)> {
        self.variables.iter()
    }

    pub fn define_variable(&mut self, name: &str, value: ResultValue) {
        self.variables.insert(name.to_owned(), value);
    }

    pub fn define_variables(&mut self, iterator: impl Iterator<Item=(String, ResultValue)>) {
        self.variables.extend(iterator);
    }

    pub fn get_function(&self, name: &str) -> Option<&Function> {
        self.functions.get(name)
    }

    pub fn functions(&self) -> impl Iterator<Item=(&String, &Function)> {
        self.functions.iter()
    }

    pub fn define_function(&mut self, name: &str, function: Function) {
        self.functions.insert(name.to_owned(), function);
    }

    pub fn define_functions(&mut self, iterator: impl Iterator<Item=(String, Function)>) {
        self.functions.extend(iterator);
    }
}