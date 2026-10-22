use std::fmt::Write;

use crate::model::{EnvironmentScope, ResultValue};
use crate::parsing::{BinaryOperator, BinaryOperators, NumericConstant, NumericConstantChar, Operator, UnaryOperator, UnaryOperators};
use crate::types::{NumberType, NumberTypeError};

pub struct IntType {
    binary_operators: BinaryOperators,
    unary_operators: UnaryOperators,
    environment: EnvironmentScope
}

impl IntType {
    pub fn new() -> IntType {
        IntType {
            binary_operators: BinaryOperators::new(
                vec![
                    (Operator::Single('^'), BinaryOperator::new(6, Box::new(|x, y| x.apply_binary(|x, y| Some(ResultValue::Int(x.pow(y as u32))), |_, _| None, &y)))),
                    (Operator::Single('*'), BinaryOperator::new(5, Box::new(|x, y| x.apply_binary(|x, y| Some(ResultValue::Int(x * y)), |_, _| None, &y)))),
                    (Operator::Single('/'), BinaryOperator::new(5, Box::new(|x, y| x.apply_binary(|x, y| Some(ResultValue::Int(x / y)), |_, _| None, &y)))),
                    (Operator::Single('&'), BinaryOperator::new(5, Box::new(|x, y| x.apply_binary(|x, y| Some(ResultValue::Int(x % y)), |_, _| None, &y)))),
                    (Operator::Single('+'), BinaryOperator::new(4, Box::new(|x, y| x.apply_binary(|x, y| Some(ResultValue::Int(x + y)), |_, _| None, &y)))),
                    (Operator::Single('-'), BinaryOperator::new(4, Box::new(|x, y| x.apply_binary(|x, y| Some(ResultValue::Int(x - y)), |_, _| None, &y)))),
                    (Operator::Dual('<', '<'), BinaryOperator::new(3, Box::new(|x, y| x.apply_binary(|x, y| Some(ResultValue::Int(x << y)), |_, _| None, &y)))),
                    (Operator::Dual('>', '>'), BinaryOperator::new(3, Box::new(|x, y| x.apply_binary(|x, y| Some(ResultValue::Int(x >> y)), |_, _| None, &y)))),
                    (Operator::Single('|'), BinaryOperator::new(2, Box::new(|x, y| x.apply_binary(|x, y| Some(ResultValue::Int(x | y)), |_, _| None, &y)))),
                    (Operator::Single('&'), BinaryOperator::new(2, Box::new(|x, y| x.apply_binary(|x, y| Some(ResultValue::Int(x & y)), |_, _| None, &y)))),
                ]
            ),
            unary_operators: UnaryOperators::new(
                vec![
                    (Operator::Single('-'), UnaryOperator::new(Box::new(|x| x.apply(|x| Some(ResultValue::Int(-x)), |_| None)))),
                    (Operator::Single('!'), UnaryOperator::new(Box::new(|x| x.apply(|x| Some(ResultValue::Int(!x)), |_| None))))
                ]
            ),
            environment: EnvironmentScope::new(
                vec![],
                vec![]
            )
        }
    }
}

impl NumberType for IntType {
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
                | NumericConstantChar::A | NumericConstantChar::B | NumericConstantChar::C | NumericConstantChar::D | NumericConstantChar::E | NumericConstantChar::F
                => {
                    write!(value, "{}", current).map_err(|_| NumberTypeError::ConstructionError)?;
                }
                NumericConstantChar::MinusSign | NumericConstantChar::PlusSign => {
                    if index > 0 {
                        break;
                    } else {
                        write!(value, "{}", current).map_err(|_| NumberTypeError::ConstructionError)?;
                    }
                }
                NumericConstantChar::DecimalPoint | NumericConstantChar::ImaginaryUnit | NumericConstantChar::Exponent => {
                    break;
                }
            }
        }

        i64::from_str_radix(&value, constant.base()).map_err(|_| NumberTypeError::TooLargeValue).map(|x| ResultValue::Int(x))
    }
}

#[test]
fn test_to_value1() {
    assert_eq!(
        Ok(ResultValue::Int(4711)),
        IntType::new().to_value(&NumericConstant::from_i64(4711))
    );
}

#[test]
fn test_to_value2() {
    assert_eq!(
        Ok(ResultValue::Int(0xFF)),
        IntType::new().to_value(&NumericConstant::new(vec![NumericConstantChar::F, NumericConstantChar::F], 16))
    );
}