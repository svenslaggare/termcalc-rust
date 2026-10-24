use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use crate::model::{EnvironmentScope, Function, ResultValue};
use crate::parsing::{Operator, ParserExpressionTree, ParserExpressionTreeData};
use crate::types::{NumberTypeId, NumberType, NumberTypeError};
use crate::types::float_type::FloatType;
use crate::types::int_type::IntType;

pub struct ExecutionEngine {
    number_types: HashMap<NumberTypeId, Box<dyn NumberType>>,
    top_level_context: ExecutionContext<'static>
}

impl ExecutionEngine {
    pub fn new(active: NumberTypeId) -> ExecutionEngine {
        fn cast_number_type(data: Box<dyn NumberType>) -> Box<dyn NumberType> {
            data
        }

        let number_types = HashMap::from_iter(vec![
            (NumberTypeId::Int, cast_number_type(Box::new(IntType::new()))),
            (NumberTypeId::Float, cast_number_type(Box::new(FloatType::new())))
        ]);

        let top_level_context = ExecutionContext::new(
            Environment::new(
                number_types.iter().map(|(id, number_type)| (id.clone(), number_type.environment().clone())),
                active
            )
        );

        ExecutionEngine {
            number_types,
            top_level_context
        }
    }
    
    pub fn number_type(&self) -> &dyn NumberType {
        self.number_types[&self.top_level_context.environment.active].as_ref()
    }

    pub fn number_type_id(&self) -> NumberTypeId {
        self.top_level_context.environment.active
    }

    pub fn set_number_type(&mut self, id: NumberTypeId) {
        self.top_level_context.environment.active = id;
    }

    pub fn environment(&self) -> &Environment {
        &self.top_level_context.environment
    }
    
    pub fn get_variable(&self, name: &str) -> Option<ResultValue> {
        self.top_level_context.get_value(name)
    }

    pub fn define_variable(&mut self, name: &str, value: ResultValue) {
        self.top_level_context.environment.define_variable(name, value);
    }

    pub fn define_function(&mut self, name: &str, function: Function) {
        self.top_level_context.environment.define_function(name, function);
    }

    pub fn execute(&mut self, tree: &ParserExpressionTree) -> Result<ResultValue, ExecutionError> {
        let mut context = self.top_level_context.child();
        let result = self.execute_internal(&mut context, tree)?;

        let defined_variables = context.defined_variables;
        let defined_functions = context.defined_functions;

        self.top_level_context.environment.define_variables(defined_variables.into_iter());
        self.top_level_context.environment.define_functions(defined_functions.into_iter());

        Ok(result)
    }

    pub fn execute_function<const N: usize>(
        &self,
        function: &Function,
        invocations_arguments: impl Iterator<Item=[ResultValue; N]>
    ) -> Result<Vec<([ResultValue; N], ResultValue)>, ExecutionError> {
        match function {
            Function::Builtin { callback, .. } => {
                let mut results = Vec::new();

                for arguments in invocations_arguments {
                    let result = callback(&arguments).ok_or_else(|| ExecutionError::InvalidOperation)?;
                    results.push((arguments, result));
                }

                Ok(results)
            }
            Function::User { parameters, body } => {
                let mut results = Vec::new();

                let mut child_context = ExecutionContext::new(self.environment().clone());

                for arguments in invocations_arguments {
                    for (parameter, argument) in parameters.iter().zip(arguments.iter()) {
                        child_context.environment.define_variable(parameter, argument.clone());
                    }

                    results.push((arguments, self.execute_internal(&mut child_context, &body)?));
                }

                Ok(results)
            }
        }
    }

    fn execute_internal(&self, context: &mut ExecutionContext, tree: &ParserExpressionTree) -> Result<ResultValue, ExecutionError> {
        match &tree.tree {
            ParserExpressionTreeData::Number(value) => {
                self.number_type().to_value(value).map_err(|err| ExecutionError::Number(err))
            }
            ParserExpressionTreeData::Variable(name) => {
                context.get_value(name).ok_or_else(|| ExecutionError::VariableNotFound(name.clone()))
            }
            ParserExpressionTreeData::BinaryOperator { operator, left, right } => {
                let operator = self.number_type().binary_operators().get(operator)
                    .ok_or_else(|| ExecutionError::UndefinedOperator(operator.clone()))?;

                let left = self.execute_internal(context, left)?;
                let right = self.execute_internal(context, right)?;

                (operator.callback)(left, right).ok_or_else(|| ExecutionError::InvalidOperation)
            }
            ParserExpressionTreeData::UnaryOperator { operator, operand } => {
                let operator = self.number_type().unary_operators().get(operator)
                    .ok_or_else(|| ExecutionError::UndefinedOperator(operator.clone()))?;

                let operand = self.execute_internal(context, operand)?;

                (operator.callback)(operand).ok_or_else(|| ExecutionError::InvalidOperation)
            }
            ParserExpressionTreeData::Declaration { left, right } => {
                match &left.tree {
                    ParserExpressionTreeData::Variable(name) => {
                        let value = self.execute_internal(context, right)?;
                        context.defined_variables.insert(name.clone(), value.clone());
                        Ok(ResultValue::Empty)
                    }
                    ParserExpressionTreeData::Call { name, arguments } => {
                        let mut parameters = Vec::new();
                        for argument in arguments {
                            if let ParserExpressionTreeData::Variable(parameter) = &argument.tree {
                                parameters.push(parameter.clone());
                            } else {
                                return Err(ExecutionError::ExpectedVariable);
                            }
                        }

                        context.defined_functions.insert(
                            name.clone(),
                            Function::User {
                                parameters,
                                body: *right.clone()
                            }
                        );

                        Ok(ResultValue::Empty)
                    }
                    _ => {
                        Err(ExecutionError::InvalidDeclaration)
                    }
                }
            }
            ParserExpressionTreeData::Call { name, arguments } => {
                let function = self.top_level_context.environment.get_function(name)
                    .ok_or_else(|| ExecutionError::FunctionNotFound(name.clone()))?;

                let mut argument_values = Vec::new();
                for argument in arguments {
                    argument_values.push(self.execute_internal(context, argument)?);
                }

                let parameters = function.parameters();
                if parameters.len() != argument_values.len() {
                    return Err(ExecutionError::InvalidCallArguments { expected: parameters.len(), actual: arguments.len() });
                }

                match function {
                    Function::Builtin { callback, .. } => {
                        callback(&argument_values).ok_or_else(|| ExecutionError::InvalidOperation)
                    }
                    Function::User { parameters, body } => {
                        let mut child_context = context.child();

                        for (parameter, argument) in parameters.iter().zip(argument_values.iter()) {
                            child_context.environment.define_variable(parameter, argument.clone());
                        }

                        self.execute_internal(&mut child_context, &body)
                    }
                }
            }
        }
    }
}

struct ExecutionContext<'a> {
    parent: Option<&'a ExecutionContext<'a>>,
    environment: Environment,

    defined_variables: HashMap<String, ResultValue>,
    defined_functions: HashMap<String, Function>
}

impl<'a> ExecutionContext<'a> {
    pub fn new(initial_environment: Environment) -> ExecutionContext<'a> {
        ExecutionContext {
            parent: None,
            environment: initial_environment,

            defined_variables: HashMap::new(),
            defined_functions: HashMap::new()
        }
    }

    pub fn child(&'a self) -> ExecutionContext<'a> {
        ExecutionContext {
            parent: Some(self),
            environment: Environment::empty(self.environment.scopes.keys().cloned(), self.environment.active.clone()),

            defined_variables: HashMap::new(),
            defined_functions: HashMap::new()
        }
    }

    pub fn get_value(&self, name: &str) -> Option<ResultValue> {
        if let Some(value) = self.environment.get_variable(name) {
            return Some(value);
        }

        if let Some(parent) = self.parent {
            parent.get_value(name)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionError {
    Number(NumberTypeError),
    VariableNotFound(String),
    FunctionNotFound(String),
    UndefinedOperator(Operator),
    InvalidOperation,
    InvalidDeclaration,
    ExpectedVariable,
    InvalidCallArguments { expected: usize, actual: usize }
}

impl Display for ExecutionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionError::Number(number) => write!(f, "{}", number),
            ExecutionError::VariableNotFound(name) => write!(f, "The variable '{}' was not found", name),
            ExecutionError::FunctionNotFound(name) => write!(f, "The function '{}' was not found", name),
            ExecutionError::UndefinedOperator(operator) => write!(f, "The operator '{}' is not defined", operator),
            ExecutionError::InvalidOperation => write!(f, "Invalid operation"),
            ExecutionError::InvalidDeclaration => write!(f, "Invalid declaration"),
            ExecutionError::ExpectedVariable => write!(f, "Expected a variable"),
            ExecutionError::InvalidCallArguments { expected, actual } => write!(f, "Expected {} arguments but got {}", expected, actual)
        }
    }
}

#[derive(Clone)]
pub struct Environment {
    scopes: HashMap<NumberTypeId, EnvironmentScope>,
    active: NumberTypeId
}

impl Environment {
    pub fn new(
        scopes: impl Iterator<Item=(NumberTypeId, EnvironmentScope)>,
        active: NumberTypeId
    ) -> Environment {
        Environment {
            scopes: HashMap::from_iter(scopes),
            active
        }
    }

    pub fn empty(number_types: impl Iterator<Item=NumberTypeId>, active: NumberTypeId) -> Environment {
        Environment {
            scopes: HashMap::from_iter(number_types.map(|number_type| (number_type, EnvironmentScope::new(Vec::new(), Vec::new())))),
            active
        }
    }

    pub fn scope(&self) -> &EnvironmentScope {
        &self.scopes[&self.active]
    }

    pub fn scope_mut(&mut self) -> &mut EnvironmentScope {
        self.scopes.get_mut(&self.active).unwrap()
    }

    pub fn get_variable(&self, name: &str) -> Option<ResultValue> {
        self.scope().get_variable(name)
    }

    pub fn variables(&self) -> impl Iterator<Item=(&String, &ResultValue)> {
        self.scope().variables()
    }

    pub fn define_variable(&mut self, name: &str, value: ResultValue) {
        for scope in self.scopes.values_mut() {
            scope.define_variable(name, value.clone());
        }
    }

    pub fn define_variables(&mut self, iterator: impl Iterator<Item=(String, ResultValue)>) {
        let variables = iterator.collect::<Vec<_>>();
        for scope in self.scopes.values_mut() {
            scope.define_variables(variables.clone().into_iter());
        }
    }

    pub fn get_function(&self, name: &str) -> Option<&Function> {
        self.scope().get_function(name)
    }

    pub fn functions(&self) -> impl Iterator<Item=(&String, &Function)> {
        self.scope().functions()
    }

    pub fn define_function(&mut self, name: &str, function: Function) {
        for scope in self.scopes.values_mut() {
            scope.define_function(name, function.clone());
        }
    }

    pub fn define_functions(&mut self, iterator: impl Iterator<Item=(String, Function)>) {
        let functions = iterator.collect::<Vec<_>>();
        for scope in self.scopes.values_mut() {
            scope.define_functions(functions.clone().into_iter());
        }
    }
}

#[test]
fn test_int1() {
    use std::rc::Rc;
    use crate::parsing::{parse_str, ParsingContext};

    let mut engine = ExecutionEngine::new(NumberTypeId::Int);
    assert_eq!(
        Ok(ResultValue::Int(255)),
        engine.execute(&parse_str(ParsingContext::from_number_type(engine.number_type()), "0xFF").unwrap())
    );

    assert_eq!(
        Ok(ResultValue::Int(10)),
        engine.execute(&parse_str(ParsingContext::from_number_type(engine.number_type()), "3 + 7").unwrap())
    );

    assert_eq!(
        Ok(ResultValue::Int(-10)),
        engine.execute(&parse_str(ParsingContext::from_number_type(engine.number_type()), "-(3 + 7)").unwrap())
    );

    engine.define_variable("a", ResultValue::Int(3));
    assert_eq!(
        Ok(ResultValue::Int(10)),
        engine.execute(&parse_str(ParsingContext::from_number_type(engine.number_type()), "a + 7").unwrap())
    );

    engine.define_function(
        "test",
        Function::Builtin {
            parameters: vec!["x".to_owned(), "y".to_owned()],
            description: "test".to_owned(),
            callback: Rc::new(|arguments| {
                match (arguments[0].as_i64(), arguments[1].as_i64()) {
                    (Some(x), Some(y)) => {
                        let mut result = x % y;
                        if result < 0 {
                            result += y;
                        }

                        Some(ResultValue::Int(result))
                    }
                    _ => None
                }
            })
        }
    );

    assert_eq!(
        Ok(ResultValue::Int(3)),
        engine.execute(&parse_str(ParsingContext::from_number_type(engine.number_type()), "test(7, 2) * 3").unwrap())
    );
    engine.define_variable("a", ResultValue::Int(3));

    assert_eq!(
        Ok(ResultValue::Int(8)),
        engine.execute(&parse_str(ParsingContext::from_number_type(engine.number_type()), "1 << 3").unwrap())
    );

    assert_eq!(
        Ok(ResultValue::Int(32)),
        engine.execute(&parse_str(ParsingContext::from_number_type(engine.number_type()), "2^5").unwrap())
    );
}

#[test]
fn test_float1() {
    use std::rc::Rc;
    use crate::parsing::{parse_str, ParsingContext};

    let mut engine = ExecutionEngine::new(NumberTypeId::Float);
    assert_eq!(
        Ok(ResultValue::Float(13.37E6)),
        engine.execute(&parse_str(ParsingContext::from_number_type(engine.number_type()), "13.37E6").unwrap())
    );

    assert_eq!(
        Ok(ResultValue::Float(10.0)),
        engine.execute(&parse_str(ParsingContext::from_number_type(engine.number_type()), "3 + 7").unwrap())
    );

    assert_eq!(
        Ok(ResultValue::Float(-10.0)),
        engine.execute(&parse_str(ParsingContext::from_number_type(engine.number_type()), "-(3 + 7)").unwrap())
    );

    engine.define_variable("a", ResultValue::Float(3.0));
    assert_eq!(
        Ok(ResultValue::Float(10.0)),
        engine.execute(&parse_str(ParsingContext::from_number_type(engine.number_type()), "a + 7").unwrap())
    );

    engine.define_function(
        "test",
        Function::Builtin {
            parameters: vec!["x".to_owned(), "y".to_owned()],
            description: "test".to_owned(),
            callback: Rc::new(|arguments| {
                match (arguments[0].as_f64(), arguments[1].as_f64()) {
                    (Some(x), Some(y)) => {
                        Some(ResultValue::Float(x.powf(y)))
                    }
                    _ => None
                }
            })
        }
    );

    assert_eq!(
        Ok(ResultValue::Float(8.0)),
        engine.execute(&parse_str(ParsingContext::from_number_type(engine.number_type()), "test(2, 3)").unwrap())
    );
}

#[test]
fn test_user_function1() {
    use crate::parsing::{parse_str, ParsingContext};

    let mut engine = ExecutionEngine::new(NumberTypeId::Int);
    engine.define_function(
        "add",
        Function::User {
            parameters: vec!["x".to_owned(), "y".to_owned()],
            body: parse_str(ParsingContext::from_number_type(engine.number_type()), "x + y").unwrap()
        }
    );

    assert_eq!(
        Ok(ResultValue::Int(11)),
        engine.execute(&parse_str(ParsingContext::from_number_type(engine.number_type()), "add(7, 4)").unwrap())
    );

    assert_eq!(
        Ok(ResultValue::Int(11)),
        engine.execute(&parse_str(ParsingContext::from_number_type(engine.number_type()), "add(add(3, 1), 7)").unwrap())
    );
}

#[test]
fn test_define_variable() {
    use crate::parsing::{parse_str, ParsingContext};

    let mut engine = ExecutionEngine::new(NumberTypeId::Int);
    assert_eq!(
        Ok(ResultValue::Empty),
        engine.execute(&parse_str(ParsingContext::from_number_type(engine.number_type()), "x = 3 + 7").unwrap())
    );

    assert_eq!(Some(ResultValue::Int(10)), engine.get_variable("x"));
}

#[test]
fn test_define_function() {
    use crate::parsing::{parse_str, ParsingContext};

    let mut engine = ExecutionEngine::new(NumberTypeId::Int);
    assert_eq!(
        Ok(ResultValue::Empty),
        engine.execute(&parse_str(ParsingContext::from_number_type(engine.number_type()), "f(x) = x*x + 4").unwrap())
    );

    assert_eq!(
        Ok(ResultValue::Int(13)),
        engine.execute(&parse_str(ParsingContext::from_number_type(engine.number_type()), "f(3)").unwrap())
    );
}

#[test]
fn test_predefined1() {
    use crate::parsing::{parse_str, ParsingContext};

    let mut engine = ExecutionEngine::new(NumberTypeId::Float);
    assert_eq!(
        Ok(ResultValue::Float(8.539734222673566)),
        engine.execute(&parse_str(ParsingContext::from_number_type(engine.number_type()), "e * pi").unwrap())
    );

    assert_eq!(
        Ok(ResultValue::Float(0.8414709848078965)),
        engine.execute(&parse_str(ParsingContext::from_number_type(engine.number_type()), "sin(1.0)").unwrap())
    );
}