use std::str::{Chars};
use std::collections::HashSet;
use std::iter::{FromIterator, Peekable};
use std::fmt::Formatter;

use lazy_static::lazy_static;
use crate::parsing::{NumericConstant, NumericConstantChar};
use crate::parsing::operator::Operator;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Number(NumericConstant),
    Operator(Operator),
    Identifier(String),
    LeftParentheses,
    RightParentheses,
    Comma,
    End
}

#[derive(Debug, PartialEq, Clone)]
pub struct TokenLocation {
    pub line: usize,
    pub column: usize,
}

impl TokenLocation {
    pub fn new(line: usize, column: usize) -> TokenLocation {
        TokenLocation {
            line,
            column
        }
    }

    pub fn extract_near(&self, text: &str) -> String {
        if let Some(line) = text.lines().nth(self.line) {
            let mut words = Vec::new();
            let mut word_start: usize = 0;
            let mut word_length: usize = 0;
            let mut index = 0;

            let line_chars = line.chars().collect::<Vec<_>>();
            for current in &line_chars {
                if current.is_whitespace() {
                    words.push((word_start, word_length));
                    word_length = 0;
                    word_start = index + 1;
                } else {
                    word_length += 1;
                }

                index += 1;
            }

            if word_length > 0 {
                words.push((word_start, word_length));
            }

            let get_substr = |(start, length): &(usize, usize)| String::from_iter(&line_chars[*start..(*start + *length)]);

            let mut near_text = String::new();
            for (word_index, &(word_start, word_length)) in words.iter().enumerate() {
                if word_start >= self.column || word_start + word_length >= self.column {
                    near_text = format!(
                        "{}{}{}",
                        words.get(word_index - 1).map(|w| get_substr(w) + " ").unwrap_or(String::new()),
                        get_substr(&(word_start, word_length)),
                        words.get(word_index + 1).map(|w| " ".to_owned() + &get_substr(w)).unwrap_or(String::new()),
                    );
                    break;
                }
            }

            near_text
        } else {
            String::new()
        }
    }
}

impl Default for TokenLocation {
    fn default() -> Self {
        TokenLocation::new(0, 0)
    }
}

impl std::fmt::Display for TokenLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TokenLocation({}:{})", self.line, self.column)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParserToken {
    pub location: TokenLocation,
    pub token: Token
}

impl ParserToken {
    pub fn new(line: usize, column: usize, token: Token) -> ParserToken {
        ParserToken {
            location: TokenLocation { line, column },
            token
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParserError {
    pub location: TokenLocation,
    pub error: ParserErrorType
}

impl ParserError {
    pub fn new(location: TokenLocation, error: ParserErrorType) -> ParserError {
        ParserError {
            location,
            error
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ParserErrorType {
    Unknown,
    ReachedEndOfTokens,
    TooManyTokens,
    IntConvertError,
    FloatConvertError,
    AlreadyHasDot,
    ExpectedLeftParentheses,
    ExpectedRightParentheses,
    ExpectedExpression,
    ExpectedArgumentListContinuation,
    ExpectedProjectionContinuation,
    ExpectedColumnDefinitionStart,
    ExpectedColumnDefinitionContinuation,
    ExpectedJsonColumnPartStart,
    ExpectedIdentifier,
    ExpectedString,
    ExpectedInt,
    ExpectedOperator,
    ExpectedSpecificOperator(Operator),
    ExpectedTuple,
    ExpectedColon,
    ExpectedDoubleColon,
    ExpectedRightArrow,
    ExpectedSemiColon,
    ExpectedNull,
    ExpectedColumnAccess,
    NotDefinedBinaryOperator(Operator),
    NotDefinedUnaryOperator(Operator),
    NotDefinedType(String),
    TrimOnlyForString,
    ExpectedValueForDefaultValue,
    AlreadyHaveWhere,
    AlreadyHaveJoin,
    AlreadyHaveGroupBy,
    AlreadyHaveHaving,
    AlreadyHaveLimit
}

impl std::fmt::Display for ParserErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserErrorType::Unknown => { write!(f, "Unknown error") }
            ParserErrorType::ReachedEndOfTokens => { write!(f, "Reached end of tokens") }
            ParserErrorType::TooManyTokens => { write!(f, "Too many tokens") }
            ParserErrorType::IntConvertError => { write!(f, "Failed to parse integer") }
            ParserErrorType::FloatConvertError => { write!(f, "Failed to parse float") }
            ParserErrorType::AlreadyHasDot => { write!(f, "The number already has a dot") }
            ParserErrorType::ExpectedLeftParentheses => { write!(f, "Expected '('") }
            ParserErrorType::ExpectedRightParentheses => { write!(f, "Expected ')'") }
            ParserErrorType::ExpectedExpression => { write!(f, "Expected an expression") }
            ParserErrorType::ExpectedArgumentListContinuation => { write!(f, "Expected ',' or ')'") }
            ParserErrorType::ExpectedProjectionContinuation => { write!(f, "Expected ','") }
            ParserErrorType::ExpectedColumnDefinitionStart => { write!(f, "Expected start of column definition")  }
            ParserErrorType::ExpectedJsonColumnPartStart => { write!(f, "Expected '.', '[' or '}}'")  }
            ParserErrorType::ExpectedColumnDefinitionContinuation => { write!(f, "Expected ',' or ')'")  }
            ParserErrorType::ExpectedIdentifier => { write!(f, "Expected an identifier") }
            ParserErrorType::ExpectedString => { write!(f, "Expected a string") }
            ParserErrorType::ExpectedInt => { write!(f, "Expected an integer") }
            ParserErrorType::ExpectedOperator => { write!(f, "Expected an operator") }
            ParserErrorType::ExpectedSpecificOperator(operator) => { write!(f, "Expected '{}' operator", operator) }
            ParserErrorType::ExpectedTuple => { write!(f, "Expected tuple expression") }
            ParserErrorType::ExpectedColon => { write!(f, "Expected ':'") }
            ParserErrorType::ExpectedDoubleColon => { write!(f, "Expected '::'") }
            ParserErrorType::ExpectedRightArrow => { write!(f, "Expected '=>'") }
            ParserErrorType::ExpectedSemiColon => { write!(f, "Expected ';'") }
            ParserErrorType::ExpectedNull => { write!(f, "Expected NULL") }
            ParserErrorType::ExpectedColumnAccess => { write!(f, "Expected column access") }
            ParserErrorType::NotDefinedBinaryOperator(operator) => { write!(f, "'{}' is not a valid binary operator", operator) }
            ParserErrorType::NotDefinedUnaryOperator(operator) => { write!(f, "'{}' is not a valid unary operator", operator) }
            ParserErrorType::NotDefinedType(value_type) => { write!(f, "'{}' is not a valid type", value_type) }
            ParserErrorType::TrimOnlyForString => { write!(f, "The 'TRIM' modifier is only available for 'TEXT' columns") }
            ParserErrorType::ExpectedValueForDefaultValue => { write!(f, "Expected a value expression for the default value") }
            ParserErrorType::AlreadyHaveWhere => { write!(f, "This statement already have a 'WHERE' clause") }
            ParserErrorType::AlreadyHaveJoin => { write!(f, "This statement already have a 'JOIN' clause") }
            ParserErrorType::AlreadyHaveGroupBy => { write!(f, "This statement already have a 'GROUP BY' clause") }
            ParserErrorType::AlreadyHaveHaving => { write!(f, "This statement already have a 'HAVING' clause") }
            ParserErrorType::AlreadyHaveLimit => { write!(f, "This statement already have a 'LIMIT' clause") }
        }
    }
}

pub fn tokenize_simple(text: &str) -> Result<Vec<Token>, ParserError> {
    Ok(tokenize(text)?.into_iter().map(|token| token.token).collect())
}

lazy_static! {
    static ref TWO_CHAR_OPERATORS: HashSet<char> = HashSet::from_iter(vec!['<', '>'].into_iter());
}

pub fn tokenize(text: &str) -> Result<Vec<ParserToken>, ParserError> {
    struct TokenizerState<'a> {
        char_iterator: Peekable<Chars<'a>>,
        tokens: Vec<ParserToken>,
        line: usize,
        column: usize,
        token_column_start: usize
    }

    impl<'a> TokenizerState<'a> {
        fn next_char(&mut self) -> Option<char> {
            let result = self.char_iterator.next();

            if result.is_some() {
                self.column += 1;
            }

            result
        }

        fn peek_char(&mut self) -> Option<&char> {
            self.char_iterator.peek()
        }

        fn location(&self) -> TokenLocation {
            TokenLocation { line: self.line, column: self.column }
        }

        fn add(&mut self, token: Token) {
            self.tokens.push(ParserToken::new(self.line, self.token_column_start, token));
            self.token_column_start = self.column;
        }
    }

    let mut state = TokenizerState {
        char_iterator: text.chars().peekable(),
        tokens: Vec::new(),
        line: 0,
        column: 0,
        token_column_start: 0
    };

    let mut current_str: Option<String> = None;
    while let Some(current) = state.next_char() {
        if current == '\n' {
            state.line += 1;
            state.column = 0;
            state.token_column_start = 0;
        }

        if let Some(current_str) = current_str.as_mut() {
            current_str.push(current);
            continue;
        }

        if current.is_alphabetic() {
            let mut identifier = String::new();
            identifier.push(current);

            loop {
                match state.char_iterator.peek() {
                    Some(next) if next.is_alphanumeric() || next == &'_' => {
                        identifier.push(state.next_char().unwrap());
                    }
                    _ => {
                        break
                    }
                };
            }

            state.add(Token::Identifier(identifier));
        } else if current.is_numeric() {
            let mut chars = Vec::new();
            let base = if current == '0' {
                match state.peek_char().cloned() {
                    Some('b') => {
                        state.next_char();
                        2
                    }
                    Some('x') => {
                        state.next_char();
                        16
                    }
                    _ => {
                        chars.push(NumericConstantChar::from_char(current).unwrap());
                        10
                    }
                }
            } else {
                chars.push(NumericConstantChar::from_char(current).unwrap());
                10
            };

            let mut has_dot = false;
            let mut inside_exponent = true;
            let mut has_minus_sign = false;

            loop {
                match state.peek_char() {
                    Some(next) if base == 10 && next.is_numeric() => {
                        chars.push(NumericConstantChar::from_char(state.next_char().unwrap()).unwrap());
                    }
                    Some(next) if base == 2 && (next == &'0' || next == &'1') => {
                        chars.push(NumericConstantChar::from_char(state.next_char().unwrap()).unwrap());
                    }
                    Some(next) if base == 16 && is_hex_digit(*next) => {
                        chars.push(NumericConstantChar::from_char(state.next_char().unwrap().to_ascii_lowercase()).unwrap());
                    }
                    Some('E') if has_dot => {
                        chars.push(NumericConstantChar::from_char(state.next_char().unwrap()).unwrap());
                        inside_exponent = true;
                    }
                    Some('-') if inside_exponent && !has_minus_sign => {
                        chars.push(NumericConstantChar::from_char(state.next_char().unwrap()).unwrap());
                        has_minus_sign = true;
                    }
                    Some(next) if next == &'.' => {
                        if has_dot {
                            return Err(ParserError::new(state.location(), ParserErrorType::AlreadyHasDot));
                        }

                        has_dot = true;
                        chars.push(NumericConstantChar::from_char(state.next_char().unwrap()).unwrap());
                    }
                    _ => {
                        break
                    }
                };
            }

            state.add(Token::Number(NumericConstant::new(chars, base)))
        } else if current == '(' {
            state.add(Token::LeftParentheses);
        } else if current == ')' {
            state.add(Token::RightParentheses);
        } else if current == ',' {
            state.add(Token::Comma);
        } else if current.is_whitespace() {
            // Skip
        } else {
            //If the previous token is an operator and the current one also is, upgrade to a two-op char
            let mut is_dual = false;
            if let Some(last) = state.tokens.last().map(|t| &t.token) {
                match last {
                    Token::Operator(Operator::Single(operator)) if TWO_CHAR_OPERATORS.contains(operator) => {
                        state.tokens.last_mut().unwrap().token = Token::Operator(Operator::Dual(*operator, current));
                        is_dual = true;
                    }
                    _ => {}
                }
            }

            if !is_dual {
                state.add(Token::Operator(Operator::Single(current)));
            }
        }
    }

    state.add(Token::End);
    Ok(state.tokens)
}

fn is_hex_digit(current: char) -> bool {
    match current.to_ascii_lowercase() {
        current if current.is_numeric() => true,
        'a' | 'b' | 'c'| 'd' | 'e' | 'f' => true,
        _ => false
    }
}

#[test]
fn test_tokenize1() {
    let tokens = tokenize_simple("a1 + b + caba + 134 + 12 + 0");
    assert_eq!(
        vec![
            Token::Identifier("a1".to_string()),
            Token::Operator(Operator::Single('+')),
            Token::Identifier("b".to_string()),
            Token::Operator(Operator::Single('+')),
            Token::Identifier("caba".to_string()),
            Token::Operator(Operator::Single('+')),
            Token::Number(NumericConstant::from_i64(134)),
            Token::Operator(Operator::Single('+')),
            Token::Number(NumericConstant::from_i64(12)),
            Token::Operator(Operator::Single('+')),
            Token::Number(NumericConstant::from_i64(0)),
            Token::End
        ],
        tokens.unwrap()
    );
}

#[test]
fn test_tokenize2() {
    let tokens = tokenize_simple("f(a, b, 4)");
    assert_eq!(
        vec![
            Token::Identifier("f".to_string()),
            Token::LeftParentheses,
            Token::Identifier("a".to_string()),
            Token::Comma,
            Token::Identifier("b".to_string()),
            Token::Comma,
            Token::Number(NumericConstant::from_i64(4)),
            Token::RightParentheses,
            Token::End
        ],
        tokens.unwrap()
    );
}

#[test]
fn test_tokenize3() {
    let tokens = tokenize_simple("a + 4");
    assert_eq!(
        vec![
            Token::Identifier("a".to_string()),
            Token::Operator(Operator::Single('+')),
            Token::Number(NumericConstant::from_i64(4)),
            Token::End
        ],
        tokens.unwrap()
    );
}

#[test]
fn test_tokenize4() {
    let tokens = tokenize_simple("a <= 4");
    assert_eq!(
        vec![
            Token::Identifier("a".to_string()),
            Token::Operator(Operator::Dual('<', '=')),
            Token::Number(NumericConstant::from_i64(4)),
            Token::End
        ],
        tokens.unwrap()
    );
}

#[test]
fn test_tokenize6() {
    let tokens = tokenize_simple("4.0");
    assert_eq!(
        vec![
            Token::Number(NumericConstant::from_f64(4.0)),
            Token::End
        ],
        tokens.unwrap()
    );
}

#[test]
fn test_tokenize7() {
    let tokens = tokenize_simple("13.37E6");
    assert_eq!(
        vec![
            Token::Number(NumericConstant::new(
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
            )),
            Token::End
        ],
        tokens.unwrap()
    );
}

#[test]
fn test_tokenize8() {
    let tokens = tokenize_simple("1.0E-3");
    assert_eq!(
        vec![
            Token::Number(NumericConstant::new(
                vec![
                    NumericConstantChar::One,
                    NumericConstantChar::DecimalPoint,
                    NumericConstantChar::Zero,
                    NumericConstantChar::Exponent,
                    NumericConstantChar::MinusSign,
                    NumericConstantChar::Three
                ],
                10
            )),
            Token::End
        ],
        tokens.unwrap()
    );
}

#[test]
fn test_tokenize10() {
    let tokens = tokenize_simple("0xFF");
    assert_eq!(
        vec![
            Token::Number(NumericConstant::new(vec![NumericConstantChar::F, NumericConstantChar::F], 16)),
            Token::End
        ],
        tokens.unwrap()
    );
}

#[test]
fn test_tokenize11() {
    let tokens = tokenize_simple("0b101");
    assert_eq!(
        vec![
            Token::Number(NumericConstant::new(vec![NumericConstantChar::One, NumericConstantChar::Zero, NumericConstantChar::One], 2)),
            Token::End
        ],
        tokens.unwrap()
    );
}