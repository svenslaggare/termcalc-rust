use std::rc::Rc;

use crate::model::{ExpressionTree, ResultValue};

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
