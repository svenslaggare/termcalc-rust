use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::fmt::Write;

use crate::execution::engine::{ExecutionEngine, ExecutionError};
use crate::model::{Function, ResultValue};
use crate::number_helpers;
use crate::parsing::{parse_str, ParserError, ParsingContext};
use crate::plot::GnuPlotter;
use crate::types::NumberTypeId;

pub struct Calculator {
    execution: ExecutionEngine,
    output_base: u32
}

impl Calculator {
    pub fn new() -> Calculator {
        Calculator {
            execution: ExecutionEngine::new(NumberTypeId::Float),
            output_base: 10
        }
    }

    pub fn execute(&mut self, text: &str) -> CalculationResult<String> {
        match Command::from_str(text) {
            Ok(command) => {
                return self.handle_command(command);
            }
            Err(CommandError::NoCommand) => {}
            Err(err) => {
                return Ok(err.to_string())
            }
        }

        let result = self.evaluate(text)?;
        Ok(self.value_to_string(&result))
    }

    fn evaluate(&mut self, text: &str) -> CalculationResult<ResultValue> {
        let tree = parse_str(ParsingContext::from_number_type(self.execution.number_type()), text)?;
        let result = self.execution.execute(&tree)?;
        Ok(result)
    }

    fn handle_command(&mut self, command: Command) -> CalculationResult<String> {
        match command {
            Command::Exit => {
                Err(CalculationError::Exit)
            }
            Command::Help => {
                Ok(
                r#"Commands:
:exit|:q|:quit                              Exits the program.
:mode mode                                  Sets the evaluation mode: float (default), int.
:display base                               Sets to display the result in the given base.
:vars                                       Prints the defined variables.
:funcs                                      Prints the defined functions.
:plot func start end step (optional)        Plots a function using gnuplot."#.to_owned()
                )
            }
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
            Command::DisplayBase(base) => {
                if base >= 2 && base <= 36 {
                    self.output_base = base;
                    Ok(String::new())
                } else {
                    Err(CalculationError::InvalidOutputBase)
                }
            }
            Command::Plot { function, start, end, step } => {
                let start = self.evaluate(&start)?;
                let end = self.evaluate(&end)?;
                let step = match step {
                    Some(step) => Some(self.evaluate(&step)?),
                    None => None
                };

                GnuPlotter::new(&function, start, end, step).plot(&self.execution)
            }
        }
    }

    fn value_to_string(&self, value: &ResultValue) -> String {
        match value {
            ResultValue::Empty => String::new(),
            ResultValue::Int(value) => {
                match self.output_base {
                    10 => value.to_string(),
                    2 => number_helpers::to_base(*value, self.output_base, "0b"),
                    16 => number_helpers::to_base(*value, self.output_base, "0x"),
                    _ => number_helpers::to_base(*value, self.output_base, "")
                }
            },
            ResultValue::Float(value) => value.to_string()
        }
    }
}

pub type CalculationResult<T> = Result<T, CalculationError>;

#[derive(Debug, PartialEq)]
pub enum CalculationError {
    Exit,
    Parser(ParserError),
    Execution(ExecutionError),
    InvalidOutputBase,
    GnuPlotNotFound,
    InternalError
}

impl Display for CalculationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CalculationError::Exit => write!(f, "Exiting..."),
            CalculationError::Parser(error) => write!(f, "{}", error.error),
            CalculationError::Execution(error) => write!(f, "{}", error),
            CalculationError::InvalidOutputBase => write!(f, "The base must be >= 2 and <= 36"),
            CalculationError::GnuPlotNotFound => write!(f, "Please install 'gnuplot' to enable plotting"),
            CalculationError::InternalError => write!(f, "Internal error")
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
    Exit,
    Help,
    Mode(NumberTypeId),
    ShowVariables,
    ShowFunctions,
    DisplayBase(u32),
    Plot { function: String, start: String, end: String, step: Option<String> }
}

enum CommandError {
    NoCommand,
    ExpectedArgument,
    ExpectedInt,
    InvalidNumberType
}

impl Display for CommandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::NoCommand => write!(f, ""),
            CommandError::ExpectedArgument => write!(f, "Expected an argument"),
            CommandError::ExpectedInt => write!(f, "Expected an integer"),
            CommandError::InvalidNumberType => write!(f, "Invalid number type")
        }
    }
}

impl Command {
    pub fn from_str(text: &str) -> Result<Command, CommandError> {
        if text.chars().next() == Some(':') {
            let (_, text) = text.split_at(1);
            let mut parts = text.split_whitespace();

            let mut next_part = || {
                parts.next().ok_or_else(|| CommandError::ExpectedArgument)
            };

            let command = next_part()?;
            match command {
                "exit" | "quit" | "q" => {
                    return Ok(Command::Exit);
                }
                "help" => {
                    return Ok(Command::Help);
                }
                "mode" => {
                    let number_type = NumberTypeId::from_str(next_part()?).ok_or_else(|| CommandError::InvalidNumberType)?;
                    return Ok(Command::Mode(number_type));
                }
                "vars" => {
                    return Ok(Command::ShowVariables);
                }
                "funcs" => {
                    return Ok(Command::ShowFunctions);
                },
                "display" => {
                    let base = u32::from_str(next_part()?).map_err(|_| CommandError::ExpectedInt)?;
                    return Ok(Command::DisplayBase(base));
                }
                "plot" => {
                    let function = next_part()?.to_owned();
                    let start = next_part()?.to_owned();
                    let end = next_part()?.to_owned();
                    let step = next_part().ok().map(|x| x.to_owned());
                    return Ok(Command::Plot { function, start, end, step })
                }
                _ => {}
            }
        }

        Err(CommandError::NoCommand)
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
fn test_display_base() {
    let mut calculator = Calculator::new();
    calculator.execution.set_number_type(NumberTypeId::Int);
    calculator.output_base = 16;
    assert_eq!(Ok("0xff".to_string()), calculator.execute("255"));
    calculator.output_base = 2;
    assert_eq!(Ok("0b101".to_string()), calculator.execute("5"));
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
        Ok("Built-in:\n    sqrt(x): Computes the square root of x.\n    ln(x): Computes the natural logarithm of x.\n    logb(x, y): Computes the y-logarithm of x.\n    acos(x): Computes the inverse cosine of x\n    atan(x): Computes the inverse tangent of x\n    ceil(x): Ceils x.\n    floor(x): Floors x.\n    sin(x): Computes the sine of x\n    tan(x): Computes the tangent of x\n    cos(x): Computes the cosine of x\n    asin(x): Computes the inverse sine of x\n    round(x): Rounds x.\n    log10(x): Computes the 10-logarithm of x.\n\nUser-defined:\n    f(x) = (x ^ 2)\n".to_string()),
        calculator.execute(":funcs")
    );
}