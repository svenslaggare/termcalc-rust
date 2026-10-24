use crate::model::ExpressionTree;
use crate::model::numeric_constant::NumericConstant;
use crate::model::operator::{BinaryOperators, Operator, UnaryOperators};
use crate::parsing::{parse_str, BinaryOperator, UnaryOperator};
use crate::parsing::parser::{Parser, ParsingContext};
use crate::parsing::tokenizer::{ParserErrorType, Token};

#[test]
fn test_advance_parser() {
    let binary_operators = test_binary_operators();
    let unary_operators = test_unary_operators();

    let mut parser = Parser::from_plain_tokens(
        &binary_operators,
        &unary_operators,
        vec![
            Token::Identifier("a".to_string()),
            Token::Operator(Operator::Single('+')),
            Token::Number(NumericConstant::from_i64(4))
        ]
    );

    assert_eq!(&Token::Identifier("a".to_string()), parser.next().unwrap());
    assert_eq!(&Token::Operator(Operator::Single('+')), parser.next().unwrap());
    assert_eq!(&Token::Number(NumericConstant::from_i64(4)), parser.next().unwrap());
    assert_eq!(Err(ParserErrorType::ReachedEndOfTokens), parser.next().map_err(|err| err.error));
    assert_eq!(Err(ParserErrorType::ReachedEndOfTokens), parser.next().map_err(|err| err.error));
}

#[test]
fn test_parse_values() {
    let binary_operators = test_binary_operators();
    let unary_operators = test_unary_operators();

    let tree = Parser::from_plain_tokens(
        &binary_operators,
        &unary_operators,
        vec![
            Token::Number(NumericConstant::from_i64(4711)),
            Token::End
        ]
    ).parse().unwrap();

    assert_eq!(
        ExpressionTree::Number(NumericConstant::from_i64(4711)),
        tree
    );

    let tree = Parser::from_plain_tokens(
        &binary_operators,
        &unary_operators,
        vec![
            Token::Number(NumericConstant::from_f64(13.37)),
            Token::End
        ]
    ).parse().unwrap();

    assert_eq!(
        ExpressionTree::Number(NumericConstant::from_f64(13.37)),
        tree
    );
}

#[test]
fn test_parse_expression1() {
    let binary_operators = test_binary_operators();
    let unary_operators = test_unary_operators();

    let context = ParsingContext {
        binary_operators: &binary_operators,
        unary_operators: &unary_operators
    };

    let tree = parse_str(context, "a + 4").unwrap();
    assert_eq!("(a + 4)", tree.to_string());
}

#[test]
fn test_parse_expression2() {
    let binary_operators = test_binary_operators();
    let unary_operators = test_unary_operators();

    let context = ParsingContext {
        binary_operators: &binary_operators,
        unary_operators: &unary_operators
    };

    let tree = parse_str(context, "(a + 4) * b").unwrap();
    assert_eq!("((a + 4) * b)", tree.to_string());
}

#[test]
fn test_parse_expression3() {
    let binary_operators = test_binary_operators();
    let unary_operators = test_unary_operators();

    let context = ParsingContext {
        binary_operators: &binary_operators,
        unary_operators: &unary_operators
    };

    let tree = parse_str(context, "-4").unwrap();
    assert_eq!("-4", tree.to_string());
}

#[test]
fn test_parse_expression4() {
    let binary_operators = test_binary_operators();
    let unary_operators = test_unary_operators();

    let context = ParsingContext {
        binary_operators: &binary_operators,
        unary_operators: &unary_operators
    };

    let tree = parse_str(context, "2^8").unwrap();
    assert_eq!("(2 ^ 8)", tree.to_string());
}

#[test]
fn test_parse_expression5() {
    let binary_operators = test_binary_operators();
    let unary_operators = test_unary_operators();

    let context = ParsingContext {
        binary_operators: &binary_operators,
        unary_operators: &unary_operators
    };

    let tree = parse_str(context, "2^(a + 8)").unwrap();
    assert_eq!("(2 ^ (a + 8))", tree.to_string());
}

#[test]
fn test_parse_call1() {
    let binary_operators = test_binary_operators();
    let unary_operators = test_unary_operators();

    let context = ParsingContext {
        binary_operators: &binary_operators,
        unary_operators: &unary_operators
    };

    let tree = parse_str(context, "f(4, a)").unwrap();
    assert_eq!("f(4, a)", tree.to_string());
}

#[test]
fn test_parse_call2() {
    let binary_operators = test_binary_operators();
    let unary_operators = test_unary_operators();

    let context = ParsingContext {
        binary_operators: &binary_operators,
        unary_operators: &unary_operators
    };

    let tree = parse_str(context, "f(g(b, c), a)").unwrap();
    assert_eq!("f(g(b, c), a)", tree.to_string());
}

fn test_binary_operators() -> BinaryOperators {
    BinaryOperators::new(
        vec![
            (Operator::Single('^'), BinaryOperator::new(6, Box::new(|_, _| None))),
            (Operator::Single('*'), BinaryOperator::new(5, Box::new(|_, _| None))),
            (Operator::Single('/'), BinaryOperator::new(5, Box::new(|_, _| None))),
            (Operator::Single('+'), BinaryOperator::new(4, Box::new(|_, _| None))),
            (Operator::Single('-'), BinaryOperator::new(4, Box::new(|_, _| None)))
        ]
    )
}

fn test_unary_operators() -> UnaryOperators {
    UnaryOperators::new(
        vec![(Operator::Single('-'), UnaryOperator::new(Box::new(|_| None)))]
    )
}