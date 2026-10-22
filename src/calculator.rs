use std::fmt::{Display, Formatter};
use std::fmt::Write;

use crate::execution::engine::{ExecutionEngine, ExecutionError};
use crate::model::{Function, ResultValue};
use crate::parsing::{parse_str, ParserError, ParsingContext};
use crate::types::NumberTypeId;

pub struct Calculator {
    execution: ExecutionEngine
}

impl Calculator {
    pub fn new() -> Calculator {
        Calculator {
            execution: ExecutionEngine::new(NumberTypeId::Float)
        }
    }

    pub fn execute(&mut self, text: &str) -> CalculationResult<String> {
        if let Some(command) = Command::from_str(text) {
            return self.handle_command(command);
        }

        let tree = parse_str(ParsingContext::from_number_type(self.execution.number_type()), text)?;
        let result = self.execution.execute(&tree)?;
        Ok(self.value_to_string(&result))
    }

    fn handle_command(&mut self, command: Command) -> CalculationResult<String> {
        match command {
            Command::Mode(number_type) => {
                self.execution.set_number_type(number_type);
                Ok(String::new())
            }
            Command::ShowVariables => {
                let mut result = String::new();
                for (name, value) in self.execution.environment().variables() {
                    write!(result, "{} = {}\n", name, self.value_to_string(value)).unwrap();
                }

                Ok(result)
            }
            Command::ShowFunctions => {
                let mut result = String::new();
                let mut built_in_functions = Vec::new();
                let mut user_defined_functions = Vec::new();
                let leading_whitespace = "    ";

                for (name, function) in self.execution.environment().functions() {
                    match function {
                        Function::Builtin { .. } => {
                            built_in_functions.push((name, function));
                        }
                        Function::User { .. } => {
                            user_defined_functions.push((name, function));
                        }
                    }
                }

                if !built_in_functions.is_empty() {
                    write!(result, "Built-in:\n").unwrap();
                    for (name, function) in &built_in_functions {
                        write!(
                            result,
                            "{}{}({}): {}\n",
                            leading_whitespace,
                            name,
                            function.parameters().join(", "),
                            match function {
                                Function::Builtin { description, .. } => description,
                                Function::User { .. } => ""
                            }
                        ).unwrap();
                    }
                }

                if !user_defined_functions.is_empty() {
                    if !built_in_functions.is_empty() {
                        write!(result, "\n").unwrap();
                    }

                    write!(result, "User-defined:\n").unwrap();
                    for (name, function) in &user_defined_functions {
                        write!(
                            result,
                            "{}{}({}) = {}\n",
                            leading_whitespace,
                            name,
                            function.parameters().join(", "),
                            match function {
                                Function::Builtin { .. } => String::new(),
                                Function::User { body, .. } => body.tree.to_string()
                            }
                        ).unwrap();
                    }
                }

                Ok(result)
            }
        }
    }

    fn value_to_string(&self, value: &ResultValue) -> String {
        match value {
            ResultValue::Empty => String::new(),
            ResultValue::Int(value) => value.to_string(),
            ResultValue::Float(value) => value.to_string()
        }
    }
}

pub type CalculationResult<T> = Result<T, CalculationError>;

#[derive(Debug, PartialEq)]
pub enum CalculationError {
    Parser(ParserError),
    Execution(ExecutionError)
}

impl Display for CalculationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CalculationError::Parser(error) => write!(f, "{}", error.error),
            CalculationError::Execution(error) => write!(f, "{}", error)
        }
    }
}

impl From<ParserError> for CalculationError {
    fn from(value: ParserError) -> Self {
        CalculationError::Parser(value)
    }
}

impl From<ExecutionError> for CalculationError {
    fn from(value: ExecutionError) -> Self {
        CalculationError::Execution(value)
    }
}

enum Command {
    Mode(NumberTypeId),
    ShowVariables,
    ShowFunctions,
}

impl Command {
    pub fn from_str(text: &str) -> Option<Command> {
        if text.chars().next() == Some(':') {
            let (_, text) = text.split_at(1);
            let mut parts = text.split_whitespace();
            let command = parts.next()?;
            match command {
                "mode" => {
                    let number_type = NumberTypeId::from_str(parts.next()?)?;
                    return Some(Command::Mode(number_type));
                }
                "vars" => {
                    return Some(Command::ShowVariables);
                }
                "funcs" => {
                    return Some(Command::ShowFunctions);
                }
                _ => {}
            }
        }

        None
    }
}

#[test]
fn test_calc1() {
    let mut calculator = Calculator::new();
    assert_eq!(Ok("8.539734222673566".to_string()), calculator.execute("e * pi"));
}

#[test]
fn test_calc2() {
    let mut calculator = Calculator::new();
    assert_eq!(Ok("".to_string()), calculator.execute("x = 4"));
    assert_eq!(Ok("4".to_string()), calculator.execute("x"));
}

#[test]
fn test_set_mode() {
    let mut calculator = Calculator::new();
    assert_eq!(Ok("".to_string()), calculator.execute(":mode int"));
    assert_eq!(NumberTypeId::Int, calculator.execution.number_type_id());
}

#[test]
fn test_display_vars() {
    let mut calculator = Calculator::new();
    assert_eq!(Ok("e = 2.718281828459045\npi = 3.141592653589793\n".to_string()), calculator.execute(":vars"));
}

#[test]
fn test_display_funcs() {
    let mut calculator = Calculator::new();
    assert_eq!(Ok("".to_string()), calculator.execute("f(x) = x^2"));
    assert_eq!(
        Ok("Built-in:\n    sqrt(x): Computes the square root of x.\n    ln(x): Computes the natural logarithm of x.\n    logb(x): Computes the y-logarithm of x..\n    acos(x): Computes the inverse cosine of x\n    atan(x): Computes the inverse tangent of x\n    ceil(x): Ceils x.\n    floor(x): Floors x.\n    sin(x): Computes the sine of x\n    tan(x): Computes the tangent of x\n    cos(x): Computes the cosine of x\n    asin(x): Computes the inverse sine of x\n    round(x): Rounds x.\n    log10(x): Computes the 10-logarithm of x.\n\nUser-defined:\n    f(x) = (x ^ 2)\n".to_string()),
        calculator.execute(":funcs")
    );
}