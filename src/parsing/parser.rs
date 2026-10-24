use crate::model::ExpressionTree;
use crate::parsing::Operator;
use crate::parsing::tokenizer::{ParserErrorType, Token, ParserToken, tokenize, ParserError, TokenLocation};
use crate::parsing::operator::{BinaryOperators, UnaryOperators};
use crate::types::NumberType;

pub struct ParsingContext<'a> {
    pub binary_operators: &'a BinaryOperators,
    pub unary_operators: &'a UnaryOperators
}

impl<'a> ParsingContext<'a> {
    pub fn from_number_type(number_type: &'a dyn NumberType) -> ParsingContext<'a> {
        ParsingContext {
            binary_operators: number_type.binary_operators(),
            unary_operators: number_type.unary_operators()
        }
    }
}

pub fn parse_str(context: ParsingContext, text: &str) -> ParserResult<ExpressionTree> {
    let tokens = tokenize(text)?;
    Parser::new(context.binary_operators, context.unary_operators, tokens).parse()
}



pub type ParserResult<T> = Result<T, ParserError>;

pub struct Parser<'a> {
    tokens: Vec<ParserToken>,
    index: isize,
    binary_operators: &'a BinaryOperators,
    unary_operators: &'a UnaryOperators
}

impl<'a> Parser<'a> {
    pub fn new(
        binary_operators: &'a BinaryOperators,
        unary_operators: &'a UnaryOperators,
        tokens: Vec<ParserToken>
    ) -> Parser<'a> {
        Parser {
            tokens,
            index: -1,
            binary_operators,
            unary_operators
        }
    }

    pub fn from_plain_tokens(
        binary_operators: &'a BinaryOperators,
        unary_operators: &'a UnaryOperators,
        tokens: Vec<Token>
    ) -> Parser<'a> {
        Parser {
            tokens: tokens.into_iter().map(|token| ParserToken::new(0, 0, token)).collect(),
            index: -1,
            binary_operators,
            unary_operators
        }
    }

    pub fn parse(&mut self) -> ParserResult<ExpressionTree>{
        self.next()?;
        self.parse_expression_internal()
    }

    fn parse_expression_internal(&mut self) -> ParserResult<ExpressionTree>{
        let lhs = self.parse_unary_operator()?;
        self.parse_binary_operator_rhs(0, lhs)
    }

    fn parse_binary_operator_rhs(&mut self, precedence: i32, lhs: ExpressionTree) -> ParserResult<ExpressionTree>{
        let mut lhs = lhs;
        loop {
            let token_precedence = self.get_token_precedence()?;

            if token_precedence < precedence {
                return Ok(lhs);
            }

            let op_location = self.current_location();
            let op = self.current().clone();
            self.next()?;

            let mut rhs = self.parse_unary_operator()?;
            if token_precedence < self.get_token_precedence()? {
                rhs = self.parse_binary_operator_rhs(token_precedence + 1, rhs)?;
            }

            match op {
                Token::Operator(Operator::Single('=')) => {
                    lhs = ExpressionTree::Declaration {
                        left: Box::new(lhs),
                        right: Box::new(rhs)
                    }
                }
                Token::Operator(op) => {
                    lhs = ExpressionTree::BinaryOperator {
                        operator: op,
                        left: Box::new(lhs),
                        right: Box::new(rhs)
                    }
                }
                _ => { return Err(ParserError::new(op_location, ParserErrorType::ExpectedOperator)); }
            }
        }
    }

    fn get_token_precedence(&self) -> ParserResult<i32> {
        match self.current() {
            Token::Operator(op) => {
                if op == &Operator::Single('=') {
                    return Ok(1);
                }

                match self.binary_operators.get(op) {
                    Some(bin_op) => Ok(bin_op.precedence),
                    None => Err(self.create_error(ParserErrorType::NotDefinedBinaryOperator(op.clone())))
                }
            }
            _ => Ok(-1)
        }
    }

    fn parse_primary_expression(&mut self) -> ParserResult<ExpressionTree>{
        match self.current().clone() {
            Token::Number(value) => {
                self.next()?;
                Ok(ExpressionTree::Number(value))
            }
            Token::Identifier(identifier) => self.parse_identifier_expression(identifier.clone()),
            Token::LeftParentheses => {
                self.next()?;
                let expression = self.parse_expression_internal();

                self.expect_and_consume_token(
                    Token::RightParentheses,
                    ParserErrorType::ExpectedRightParentheses
                )?;

                expression
            }
            _ => Err(self.create_error(ParserErrorType::ExpectedExpression))
        }
    }

    fn parse_identifier_expression(&mut self, identifier: String) -> ParserResult<ExpressionTree>{
        self.next()?;

        match self.current() {
            Token::LeftParentheses => (),
            _ => return Ok(ExpressionTree::Variable(identifier))
        }

        self.next()?;

        let arguments = self.parse_arguments(Token::RightParentheses)?;
        Ok(ExpressionTree::Call { name: identifier, arguments })
    }

    fn parse_unary_operator(&mut self) -> ParserResult<ExpressionTree>{
        match self.current() {
            Token::Operator(_) => {},
            _ => return self.parse_primary_expression()
        }

        let op_location = self.current_location();
        let op_token = self.current().clone();
        self.next()?;

        let operand = self.parse_unary_operator()?;
        match op_token {
            Token::Operator(op) => {
                if !self.unary_operators.exists(&op) {
                    return Err(ParserError::new(op_location, ParserErrorType::NotDefinedUnaryOperator(op)));
                }

                Ok(ExpressionTree::UnaryOperator { operator: op, operand: Box::new(operand) })
            }
            _ => Err(ParserError::new(op_location, ParserErrorType::Unknown))
        }
    }

    fn parse_arguments(&mut self, right_parentheses_token: Token) -> ParserResult<Vec<ExpressionTree>> {
        let mut arguments = Vec::<ExpressionTree>::new();

        match self.current() {
            token if token == &right_parentheses_token => {
                self.next()?;
            }
            _ => {
                self.parse_list(right_parentheses_token, &mut arguments)?;
            }
        }

        Ok(arguments)
    }

    fn parse_list(
        &mut self,
        right_parentheses_token: Token,
        list: &mut Vec<ExpressionTree>
    ) -> ParserResult<()> {
        loop {
            list.push(self.parse_expression_internal()?);
            match self.current() {
                token if token == &right_parentheses_token => {
                    self.next()?;
                    break;
                }
                Token::Comma => {}
                _ => {
                    return Err(self.create_error(ParserErrorType::ExpectedArgumentListContinuation))
                }
            }

            self.next()?;
        }

        Ok(())
    }

    fn expect_and_consume_token(&mut self, token: Token, error: ParserErrorType) -> ParserResult<()> {
        self.expect_token(token, error)?;
        self.next()?;
        Ok(())
    }

    fn expect_token(&self, token: Token, error: ParserErrorType) -> ParserResult<()> {
        if self.current() != &token {
            return Err(self.create_error(error));
        }

        Ok(())
    }

    fn current(&self) -> &Token {
        &self.tokens[self.index as usize].token
    }

    fn current_location(&self) -> TokenLocation {
        self.tokens[self.index as usize].location.clone()
    }

    fn create_error(&self, error: ParserErrorType) -> ParserError {
        ParserError::new(self.current_location(), error)
    }

    pub fn next(&mut self) -> ParserResult<&Token> {
        let next_index = self.index + 1;
        if next_index >= self.tokens.len() as isize {
            return Err(self.create_error(ParserErrorType::ReachedEndOfTokens));
        }

        self.index = next_index;
        Ok(&self.tokens[self.index as usize].token)
    }
}

impl std::fmt::Display for ExpressionTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        ParserExpressionTreeDataVisualizer {}.fmt(self, f)
    }
}

struct ParserExpressionTreeDataVisualizer {}

impl ParserExpressionTreeDataVisualizer {
    fn fmt(&self, value: &ExpressionTree, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match value {
            ExpressionTree::Number(value) => {
                write!(f, "{}", value)
            }
            ExpressionTree::Variable(name) => {
                write!(f, "{}", name)
            }
            ExpressionTree::BinaryOperator { operator, left, right } => {
                write!(f, "(")?;
                self.fmt(&left, f)?;
                write!(f, " {} ", operator)?;
                self.fmt(&right, f)?;
                write!(f, ")")?;
                Ok(())
            }
            ExpressionTree::UnaryOperator { operator, operand } => {
                write!(f, "{}", operator)?;
                self.fmt(&operand, f)?;
                Ok(())
            }
            ExpressionTree::Declaration { left, right } => {
                write!(f, "(")?;
                self.fmt(&left, f)?;
                write!(f, " = ")?;
                self.fmt(&right, f)?;
                write!(f, ")")?;
                Ok(())
            }
            ExpressionTree::Call { name, arguments } => {
                write!(f, "{}", name)?;
                write!(f, "(")?;
                let mut is_first = true;
                for argument in arguments {
                    if !is_first {
                        write!(f, ", ")?;
                    } else {
                        is_first = false;
                    }

                    self.fmt(&argument, f)?;
                }

                write!(f, ")")?;
                Ok(())
            }
        }
    }
}