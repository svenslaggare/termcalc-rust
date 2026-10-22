use std::fmt::Write;
use std::rc::Rc;
use std::str::FromStr;

use crate::model::{EnvironmentScope, Function, ResultValue};
use crate::parsing::{BinaryOperator, BinaryOperators, NumericConstant, NumericConstantChar, Operator, UnaryOperator, UnaryOperators};
use crate::types::{NumberType, NumberTypeError};

pub struct FloatType {
    binary_operators: BinaryOperators,
    unary_operators: UnaryOperators,
    environment: EnvironmentScope
}

impl FloatType {
    pub fn new() -> FloatType {
        FloatType {
            binary_operators: BinaryOperators::new(
                vec![
                    (Operator::Single('^'), BinaryOperator::new(6, Box::new(|x, y| x.apply_binary(|_, _| None, |x, y| Some(ResultValue::Float(x.powf(y))), &y)))),
                    (Operator::Single('*'), BinaryOperator::new(5, Box::new(|x, y| x.apply_binary(|_, _| None, |x, y| Some(ResultValue::Float(x * y)), &y)))),
                    (Operator::Single('/'), BinaryOperator::new(5, Box::new(|x, y| x.apply_binary(|_, _| None, |x, y| Some(ResultValue::Float(x / y)), &y)))),
                    (Operator::Single('+'), BinaryOperator::new(4, Box::new(|x, y| x.apply_binary(|_, _| None, |x, y| Some(ResultValue::Float(x + y)), &y)))),
                    (Operator::Single('-'), BinaryOperator::new(4, Box::new(|x, y| x.apply_binary(|_, _| None, |x, y| Some(ResultValue::Float(x - y)), &y))))
                ]
            ),
            unary_operators: UnaryOperators::new(
                vec![
                    (Operator::Single('-'), UnaryOperator::new(Box::new(|x| x.apply(|_| None, |x| Some(ResultValue::Float(-x))))))
                ]
            ),
            environment: EnvironmentScope::new(
                vec![
                    ("e".to_owned(), ResultValue::Float(std::f64::consts::E)),
                    ("pi".to_owned(), ResultValue::Float(std::f64::consts::PI))
                ],
                vec![
                    (
                        "sqrt".to_owned(),
                        Function::Builtin {
                            parameters: vec!["x".to_owned()],
                            description: "Computes the square root of x.".to_owned(),
                            callback: create_single_argument_function(|x| x.sqrt()),
                        }
                    ),
                    (
                        "ln".to_owned(),
                        Function::Builtin {
                            parameters: vec!["x".to_owned()],
                            description: "Computes the natural logarithm of x.".to_owned(),
                            callback: create_single_argument_function(|x| x.ln()),
                        }
                    ),
                    (
                        "log10".to_owned(),
                        Function::Builtin {
                            parameters: vec!["x".to_owned()],
                            description: "Computes the 10-logarithm of x.".to_owned(),
                            callback: create_single_argument_function(|x| x.log10()),
                        }
                    ),
                    (
                        "logb".to_owned(),
                        Function::Builtin {
                            parameters: vec!["x".to_owned()],
                            description: "Computes the y-logarithm of x..".to_owned(),
                            callback: create_two_argument_function(|x, y| x.log(y)),
                        }
                    ),
                    (
                        "ceil".to_owned(),
                        Function::Builtin {
                            parameters: vec!["x".to_owned()],
                            description: "Ceils x.".to_owned(),
                            callback: create_single_argument_function(|x| x.ceil()),
                        }
                    ),
                    (
                        "floor".to_owned(),
                        Function::Builtin {
                            parameters: vec!["x".to_owned()],
                            description: "Floors x.".to_owned(),
                            callback: create_single_argument_function(|x| x.floor()),
                        }
                    ),
                    (
                        "round".to_owned(),
                        Function::Builtin {
                            parameters: vec!["x".to_owned()],
                            description: "Rounds x.".to_owned(),
                            callback: create_single_argument_function(|x| x.round()),
                        }
                    ),
                    (
                        "sin".to_owned(),
                        Function::Builtin {
                            parameters: vec!["x".to_owned()],
                            description: "Computes the sine of x".to_owned(),
                            callback: create_single_argument_function(|x| x.sin()),
                        }
                    ),
                    (
                        "cos".to_owned(),
                        Function::Builtin {
                            parameters: vec!["x".to_owned()],
                            description: "Computes the cosine of x".to_owned(),
                            callback: create_single_argument_function(|x| x.cos()),
                        }
                    ),
                    (
                        "tan".to_owned(),
                        Function::Builtin {
                            parameters: vec!["x".to_owned()],
                            description: "Computes the tangent of x".to_owned(),
                            callback: create_single_argument_function(|x| x.tan()),
                        }
                    ),
                    (
                        "asin".to_owned(),
                        Function::Builtin {
                            parameters: vec!["x".to_owned()],
                            description: "Computes the inverse sine of x".to_owned(),
                            callback: create_single_argument_function(|x| x.asin()),
                        }
                    ),
                    (
                        "acos".to_owned(),
                        Function::Builtin {
                            parameters: vec!["x".to_owned()],
                            description: "Computes the inverse cosine of x".to_owned(),
                            callback: create_single_argument_function(|x| x.acos()),
                        }
                    ),
                    (
                        "atan".to_owned(),
                        Function::Builtin {
                            parameters: vec!["x".to_owned()],
                            description: "Computes the inverse tangent of x".to_owned(),
                            callback: create_single_argument_function(|x| x.atan()),
                        }
                    )
                ]
            )
        }
    }
}

fn create_single_argument_function(callback: fn(f64) -> f64) -> Rc<dyn Fn(&Vec<ResultValue>) -> Option<ResultValue>> {
    Rc::new(move |arguments| {
        match arguments[0].as_f64() {
            Some(x) => {
                Some(ResultValue::Float(callback(x)))
            }
            _ => None
        }
    })
}

fn create_two_argument_function(callback: fn(f64, f64) -> f64) -> Rc<dyn Fn(&Vec<ResultValue>) -> Option<ResultValue>> {
    Rc::new(move |arguments| {
        match (arguments[0].as_f64(), arguments[1].as_f64()) {
            (Some(x), Some(y)) => {
                Some(ResultValue::Float(callback(x, y)))
            }
            _ => None
        }
    })
}

impl NumberType for FloatType {
    fn binary_operators(&self) -> &BinaryOperators {
        &self.binary_operators
    }

    fn unary_operators(&self) -> &UnaryOperators {
        &self.unary_operators
    }

    fn environment(&self) -> &EnvironmentScope {
        &self.environment
    }

    fn to_value(&self, constant: &NumericConstant) -> Result<ResultValue, NumberTypeError> {
        let mut value = String::new();
        for (index, current) in constant.iter().enumerate() {
            match current {
                NumericConstantChar::Zero | NumericConstantChar::One | NumericConstantChar::Two | NumericConstantChar::Three | NumericConstantChar::Four
                | NumericConstantChar::Five | NumericConstantChar::Six | NumericConstantChar::Seven | NumericConstantChar::Eight | NumericConstantChar::Nine
                | NumericConstantChar::DecimalPoint | NumericConstantChar::Exponent
                => {
                    write!(value, "{}", current).map_err(|_| NumberTypeError::ConstructionError)?;
                }
                | NumericConstantChar::A | NumericConstantChar::B | NumericConstantChar::C | NumericConstantChar::D | NumericConstantChar::E | NumericConstantChar::F => {}
                NumericConstantChar::MinusSign | NumericConstantChar::PlusSign => {
                    if index > 0 {
                        break;
                    } else {
                        write!(value, "{}", current).map_err(|_| NumberTypeError::ConstructionError)?;
                    }
                }
                NumericConstantChar::ImaginaryUnit => {
                    break;
                }
            }
        }

        f64::from_str(&value).map_err(|_| NumberTypeError::TooLargeValue).map(|x| ResultValue::Float(x))
    }
}

#[test]
fn test_to_value() {
    let float_type = FloatType::new();
    assert_eq!(Ok(ResultValue::Float(13.37)), float_type.to_value(&NumericConstant::from_f64(13.37)));
    assert_eq!(
        Ok(ResultValue::Float(13.37E6)),
        float_type.to_value(&NumericConstant::new(
            vec![
                NumericConstantChar::One,
                NumericConstantChar::Three,
                NumericConstantChar::DecimalPoint,
                NumericConstantChar::Three,
                NumericConstantChar::Seven,
                NumericConstantChar::Exponent,
                NumericConstantChar::Six
            ],
            10
        ))
    );
}