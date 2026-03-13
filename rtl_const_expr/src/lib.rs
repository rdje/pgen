//! Minimal constant-expression parser/evaluator for planned RTL frontend work.
//!
//! This crate is intentionally small and self-contained. It provides a
//! synthesis-oriented baseline for elaboration-time integer expressions:
//! - decimal and sized based integer literals (`8'hff`, `4'b1010`, `12`)
//! - identifiers resolved from a caller-provided symbol table
//! - unary `+ - ! ~`
//! - binary arithmetic, shifts, comparisons, equality, bitwise ops, logical ops
//! - ternary `?:`
//!
//! The current evaluator returns `i64` values and treats logical/comparison
//! results as `0` or `1`.

use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalError {
    pub message: String,
    pub position: usize,
}

impl EvalError {
    fn new(message: impl Into<String>, position: usize) -> Self {
        Self {
            message: message.into(),
            position,
        }
    }
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at byte {}", self.message, self.position)
    }
}

impl Error for EvalError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Plus,
    Minus,
    LogicalNot,
    BitNot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Mul,
    Div,
    Mod,
    Add,
    Sub,
    Shl,
    Shr,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    BitAnd,
    BitXor,
    BitOr,
    LogicalAnd,
    LogicalOr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Literal(i64),
    Ident(String),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Ternary {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
}

impl Expr {
    pub fn eval(&self, symbols: &HashMap<String, i64>) -> Result<i64, EvalError> {
        self.eval_inner(symbols)
    }

    fn eval_inner(&self, symbols: &HashMap<String, i64>) -> Result<i64, EvalError> {
        match self {
            Expr::Literal(value) => Ok(*value),
            Expr::Ident(name) => symbols
                .get(name)
                .copied()
                .ok_or_else(|| EvalError::new(format!("unknown identifier '{}'", name), 0)),
            Expr::Unary { op, expr } => {
                let value = expr.eval_inner(symbols)?;
                Ok(match op {
                    UnaryOp::Plus => value,
                    UnaryOp::Minus => -value,
                    UnaryOp::LogicalNot => i64::from(value == 0),
                    UnaryOp::BitNot => !value,
                })
            }
            Expr::Binary { op, left, right } => match op {
                BinaryOp::LogicalAnd => {
                    let left_value = left.eval_inner(symbols)?;
                    if left_value == 0 {
                        Ok(0)
                    } else {
                        let right_value = right.eval_inner(symbols)?;
                        Ok(i64::from(right_value != 0))
                    }
                }
                BinaryOp::LogicalOr => {
                    let left_value = left.eval_inner(symbols)?;
                    if left_value != 0 {
                        Ok(1)
                    } else {
                        let right_value = right.eval_inner(symbols)?;
                        Ok(i64::from(right_value != 0))
                    }
                }
                _ => {
                    let left_value = left.eval_inner(symbols)?;
                    let right_value = right.eval_inner(symbols)?;
                    Self::eval_binary_op(op, left_value, right_value)
                }
            },
            Expr::Ternary {
                condition,
                then_expr,
                else_expr,
            } => {
                let condition_value = condition.eval_inner(symbols)?;
                if condition_value != 0 {
                    then_expr.eval_inner(symbols)
                } else {
                    else_expr.eval_inner(symbols)
                }
            }
        }
    }

    fn eval_binary_op(op: &BinaryOp, left: i64, right: i64) -> Result<i64, EvalError> {
        match op {
            BinaryOp::Mul => Ok(left * right),
            BinaryOp::Div => {
                if right == 0 {
                    Err(EvalError::new("division by zero", 0))
                } else {
                    Ok(left / right)
                }
            }
            BinaryOp::Mod => {
                if right == 0 {
                    Err(EvalError::new("modulo by zero", 0))
                } else {
                    Ok(left % right)
                }
            }
            BinaryOp::Add => Ok(left + right),
            BinaryOp::Sub => Ok(left - right),
            BinaryOp::Shl => {
                let shift = Self::validate_shift(right)?;
                Ok(left
                    .checked_shl(shift)
                    .ok_or_else(|| EvalError::new("left shift overflow", 0))?)
            }
            BinaryOp::Shr => {
                let shift = Self::validate_shift(right)?;
                Ok(left >> shift)
            }
            BinaryOp::Lt => Ok(i64::from(left < right)),
            BinaryOp::Le => Ok(i64::from(left <= right)),
            BinaryOp::Gt => Ok(i64::from(left > right)),
            BinaryOp::Ge => Ok(i64::from(left >= right)),
            BinaryOp::Eq => Ok(i64::from(left == right)),
            BinaryOp::Ne => Ok(i64::from(left != right)),
            BinaryOp::BitAnd => Ok(left & right),
            BinaryOp::BitXor => Ok(left ^ right),
            BinaryOp::BitOr => Ok(left | right),
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr => unreachable!(),
        }
    }

    fn validate_shift(shift: i64) -> Result<u32, EvalError> {
        if !(0..=63).contains(&shift) {
            return Err(EvalError::new(
                format!("shift count {} is outside supported range 0..63", shift),
                0,
            ));
        }
        Ok(shift as u32)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenKind {
    Integer(i64),
    Ident(String),
    LParen,
    RParen,
    Question,
    Colon,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    Tilde,
    Amp,
    AmpAmp,
    Pipe,
    PipePipe,
    Caret,
    Shl,
    Shr,
    Lt,
    Le,
    Gt,
    Ge,
    EqEq,
    Ne,
    End,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token {
    kind: TokenKind,
    position: usize,
}

struct Lexer<'a> {
    input: &'a str,
    index: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, index: 0 }
    }

    fn tokenize(mut self) -> Result<Vec<Token>, EvalError> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            let end = matches!(token.kind, TokenKind::End);
            tokens.push(token);
            if end {
                break;
            }
        }
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, EvalError> {
        self.skip_whitespace();
        let position = self.index;
        let Some(ch) = self.peek_char() else {
            return Ok(Token {
                kind: TokenKind::End,
                position,
            });
        };

        if ch.is_ascii_digit() {
            return self.lex_number();
        }
        if is_ident_start(ch) {
            return Ok(self.lex_ident());
        }

        let token = match ch {
            '(' => self.single_char(TokenKind::LParen),
            ')' => self.single_char(TokenKind::RParen),
            '?' => self.single_char(TokenKind::Question),
            ':' => self.single_char(TokenKind::Colon),
            '+' => self.single_char(TokenKind::Plus),
            '-' => self.single_char(TokenKind::Minus),
            '*' => self.single_char(TokenKind::Star),
            '/' => self.single_char(TokenKind::Slash),
            '%' => self.single_char(TokenKind::Percent),
            '!' => {
                if self.consume_if('=') {
                    Token {
                        kind: TokenKind::Ne,
                        position,
                    }
                } else {
                    Token {
                        kind: TokenKind::Bang,
                        position,
                    }
                }
            }
            '~' => self.single_char(TokenKind::Tilde),
            '&' => {
                if self.consume_if('&') {
                    Token {
                        kind: TokenKind::AmpAmp,
                        position,
                    }
                } else {
                    Token {
                        kind: TokenKind::Amp,
                        position,
                    }
                }
            }
            '|' => {
                if self.consume_if('|') {
                    Token {
                        kind: TokenKind::PipePipe,
                        position,
                    }
                } else {
                    Token {
                        kind: TokenKind::Pipe,
                        position,
                    }
                }
            }
            '^' => self.single_char(TokenKind::Caret),
            '<' => {
                if self.consume_if('<') {
                    Token {
                        kind: TokenKind::Shl,
                        position,
                    }
                } else if self.consume_if('=') {
                    Token {
                        kind: TokenKind::Le,
                        position,
                    }
                } else {
                    Token {
                        kind: TokenKind::Lt,
                        position,
                    }
                }
            }
            '>' => {
                if self.consume_if('>') {
                    Token {
                        kind: TokenKind::Shr,
                        position,
                    }
                } else if self.consume_if('=') {
                    Token {
                        kind: TokenKind::Ge,
                        position,
                    }
                } else {
                    Token {
                        kind: TokenKind::Gt,
                        position,
                    }
                }
            }
            '=' => {
                if self.consume_if('=') {
                    Token {
                        kind: TokenKind::EqEq,
                        position,
                    }
                } else {
                    return Err(EvalError::new("unexpected '='; expected '=='", position));
                }
            }
            _ => {
                return Err(EvalError::new(
                    format!("unexpected character '{}'", ch),
                    position,
                ));
            }
        };

        Ok(token)
    }

    fn lex_number(&mut self) -> Result<Token, EvalError> {
        let start = self.index;
        let size_text = self.take_while(|ch| ch.is_ascii_digit() || ch == '_');

        if self.peek_char() == Some('\'') {
            self.index += 1;
            let signed = matches!(self.peek_char(), Some('s' | 'S'));
            if signed {
                self.index += 1;
            }

            let Some(base_char) = self.peek_char() else {
                return Err(EvalError::new("expected base after apostrophe", self.index));
            };
            self.index += 1;
            let digits = self.take_while(is_based_digit_char);
            if digits.is_empty() {
                return Err(EvalError::new("expected digits after base specifier", self.index));
            }

            let base = match base_char.to_ascii_lowercase() {
                'b' => 2,
                'o' => 8,
                'd' => 10,
                'h' => 16,
                _ => {
                    return Err(EvalError::new(
                        format!("unsupported literal base '{}'", base_char),
                        self.index.saturating_sub(1),
                    ));
                }
            };
            let value = parse_based_integer(&digits, base, start)?;

            let _ = size_text;
            let _ = signed;
            return Ok(Token {
                kind: TokenKind::Integer(value),
                position: start,
            });
        }

        let cleaned = strip_underscores(&size_text);
        let value = cleaned.parse::<i64>().map_err(|_| {
            EvalError::new(format!("invalid decimal integer '{}'", cleaned), start)
        })?;
        Ok(Token {
            kind: TokenKind::Integer(value),
            position: start,
        })
    }

    fn lex_ident(&mut self) -> Token {
        let start = self.index;
        let text = self.take_while(is_ident_continue);
        Token {
            kind: TokenKind::Ident(text),
            position: start,
        }
    }

    fn single_char(&mut self, kind: TokenKind) -> Token {
        let position = self.index;
        self.index += 1;
        Token { kind, position }
    }

    fn consume_if(&mut self, expected: char) -> bool {
        self.index += 1;
        if self.peek_char() == Some(expected) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek_char(), Some(ch) if ch.is_ascii_whitespace()) {
            self.index += 1;
        }
    }

    fn take_while<F>(&mut self, mut predicate: F) -> String
    where
        F: FnMut(char) -> bool,
    {
        let mut output = String::new();
        while let Some(ch) = self.peek_char() {
            if !predicate(ch) {
                break;
            }
            output.push(ch);
            self.index += ch.len_utf8();
        }
        output
    }

    fn peek_char(&self) -> Option<char> {
        self.input.get(self.index..)?.chars().next()
    }
}

struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, index: 0 }
    }

    fn parse(mut self) -> Result<Expr, EvalError> {
        let expr = self.parse_conditional()?;
        if !matches!(self.current().kind, TokenKind::End) {
            return Err(EvalError::new(
                "unexpected trailing tokens",
                self.current().position,
            ));
        }
        Ok(expr)
    }

    fn parse_conditional(&mut self) -> Result<Expr, EvalError> {
        let condition = self.parse_binary_expression(1)?;
        if matches!(self.current().kind, TokenKind::Question) {
            self.bump();
            let then_expr = self.parse_conditional()?;
            self.expect(TokenKind::Colon)?;
            let else_expr = self.parse_conditional()?;
            return Ok(Expr::Ternary {
                condition: Box::new(condition),
                then_expr: Box::new(then_expr),
                else_expr: Box::new(else_expr),
            });
        }
        Ok(condition)
    }

    fn parse_binary_expression(&mut self, min_precedence: u8) -> Result<Expr, EvalError> {
        let mut left = self.parse_prefix()?;

        loop {
            let Some((op, precedence)) = self.current_binary_op() else {
                break;
            };
            if precedence < min_precedence {
                break;
            }

            self.bump();
            let right = self.parse_binary_expression(precedence + 1)?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expr, EvalError> {
        let position = self.current().position;
        match &self.current().kind {
            TokenKind::Plus => {
                self.bump();
                Ok(Expr::Unary {
                    op: UnaryOp::Plus,
                    expr: Box::new(self.parse_prefix()?),
                })
            }
            TokenKind::Minus => {
                self.bump();
                Ok(Expr::Unary {
                    op: UnaryOp::Minus,
                    expr: Box::new(self.parse_prefix()?),
                })
            }
            TokenKind::Bang => {
                self.bump();
                Ok(Expr::Unary {
                    op: UnaryOp::LogicalNot,
                    expr: Box::new(self.parse_prefix()?),
                })
            }
            TokenKind::Tilde => {
                self.bump();
                Ok(Expr::Unary {
                    op: UnaryOp::BitNot,
                    expr: Box::new(self.parse_prefix()?),
                })
            }
            TokenKind::Integer(value) => {
                let value = *value;
                self.bump();
                Ok(Expr::Literal(value))
            }
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.bump();
                Ok(Expr::Ident(name))
            }
            TokenKind::LParen => {
                self.bump();
                let expr = self.parse_conditional()?;
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            }
            _ => Err(EvalError::new("expected expression", position)),
        }
    }

    fn current_binary_op(&self) -> Option<(BinaryOp, u8)> {
        let op = match self.current().kind {
            TokenKind::Star => BinaryOp::Mul,
            TokenKind::Slash => BinaryOp::Div,
            TokenKind::Percent => BinaryOp::Mod,
            TokenKind::Plus => BinaryOp::Add,
            TokenKind::Minus => BinaryOp::Sub,
            TokenKind::Shl => BinaryOp::Shl,
            TokenKind::Shr => BinaryOp::Shr,
            TokenKind::Lt => BinaryOp::Lt,
            TokenKind::Le => BinaryOp::Le,
            TokenKind::Gt => BinaryOp::Gt,
            TokenKind::Ge => BinaryOp::Ge,
            TokenKind::EqEq => BinaryOp::Eq,
            TokenKind::Ne => BinaryOp::Ne,
            TokenKind::Amp => BinaryOp::BitAnd,
            TokenKind::Caret => BinaryOp::BitXor,
            TokenKind::Pipe => BinaryOp::BitOr,
            TokenKind::AmpAmp => BinaryOp::LogicalAnd,
            TokenKind::PipePipe => BinaryOp::LogicalOr,
            _ => return None,
        };
        Some((op.clone(), precedence_of(&op)))
    }

    fn current(&self) -> &Token {
        &self.tokens[self.index]
    }

    fn bump(&mut self) {
        if self.index + 1 < self.tokens.len() {
            self.index += 1;
        }
    }

    fn expect(&mut self, expected: TokenKind) -> Result<(), EvalError> {
        if self.current().kind == expected {
            self.bump();
            Ok(())
        } else {
            Err(EvalError::new(
                format!("expected {:?}", expected),
                self.current().position,
            ))
        }
    }
}

pub fn parse_expression(input: &str) -> Result<Expr, EvalError> {
    let tokens = Lexer::new(input).tokenize()?;
    Parser::new(tokens).parse()
}

pub fn evaluate_expression(
    input: &str,
    symbols: &HashMap<String, i64>,
) -> Result<i64, EvalError> {
    parse_expression(input)?.eval(symbols)
}

fn precedence_of(op: &BinaryOp) -> u8 {
    match op {
        BinaryOp::LogicalOr => 1,
        BinaryOp::LogicalAnd => 2,
        BinaryOp::BitOr => 3,
        BinaryOp::BitXor => 4,
        BinaryOp::BitAnd => 5,
        BinaryOp::Eq | BinaryOp::Ne => 6,
        BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => 7,
        BinaryOp::Shl | BinaryOp::Shr => 8,
        BinaryOp::Add | BinaryOp::Sub => 9,
        BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => 10,
    }
}

fn parse_based_integer(digits: &str, base: u32, position: usize) -> Result<i64, EvalError> {
    let mut value = 0i64;
    for ch in digits.chars() {
        if ch == '_' {
            continue;
        }
        let digit = match ch.to_digit(base) {
            Some(digit) => digit as i64,
            None => {
                return Err(EvalError::new(
                    format!(
                        "unsupported digit '{}' in base-{} integer literal",
                        ch, base
                    ),
                    position,
                ));
            }
        };
        value = value
            .checked_mul(base as i64)
            .and_then(|partial| partial.checked_add(digit))
            .ok_or_else(|| EvalError::new("integer literal overflow", position))?;
    }
    Ok(value)
}

fn strip_underscores(input: &str) -> String {
    input.chars().filter(|ch| *ch != '_').collect()
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    is_ident_start(ch) || ch.is_ascii_digit() || ch == '$'
}

fn is_based_digit_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

#[cfg(test)]
mod tests {
    use super::{Expr, parse_expression};
    use crate::evaluate_expression;
    use std::collections::HashMap;

    #[test]
    fn arithmetic_precedence_works() {
        assert_eq!(evaluate_expression("2 + 3 * 4", &HashMap::new()).unwrap(), 14);
    }

    #[test]
    fn parentheses_override_precedence() {
        assert_eq!(evaluate_expression("(2 + 3) * 4", &HashMap::new()).unwrap(), 20);
    }

    #[test]
    fn identifiers_resolve_from_symbol_table() {
        let symbols = HashMap::from([
            ("WIDTH".to_string(), 8),
            ("LANES".to_string(), 2),
        ]);
        assert_eq!(evaluate_expression("WIDTH * LANES", &symbols).unwrap(), 16);
    }

    #[test]
    fn based_literals_are_supported() {
        assert_eq!(evaluate_expression("8'hff + 4'b0001", &HashMap::new()).unwrap(), 256);
    }

    #[test]
    fn shift_and_bitwise_precedence_works() {
        assert_eq!(evaluate_expression("1 << 3 | 2", &HashMap::new()).unwrap(), 10);
    }

    #[test]
    fn comparisons_and_logical_ops_return_zero_or_one() {
        let symbols = HashMap::from([
            ("WIDTH".to_string(), 8),
            ("ENABLE".to_string(), 1),
        ]);
        assert_eq!(
            evaluate_expression("(WIDTH > 4) && ENABLE", &symbols).unwrap(),
            1
        );
    }

    #[test]
    fn ternary_expression_selects_branch() {
        let symbols = HashMap::from([
            ("ENABLE".to_string(), 0),
            ("WIDTH".to_string(), 8),
        ]);
        assert_eq!(
            evaluate_expression("ENABLE ? WIDTH : 1", &symbols).unwrap(),
            1
        );
    }

    #[test]
    fn division_by_zero_is_reported() {
        let err = evaluate_expression("4 / 0", &HashMap::new()).unwrap_err();
        assert!(err.message.contains("division by zero"));
    }

    #[test]
    fn unknown_identifier_is_reported() {
        let err = evaluate_expression("WIDTH + 1", &HashMap::new()).unwrap_err();
        assert!(err.message.contains("unknown identifier"));
    }

    #[test]
    fn parser_builds_structured_ast() {
        let expr = parse_expression("A ? B + 1 : C").unwrap();
        assert!(matches!(expr, Expr::Ternary { .. }));
    }
}
