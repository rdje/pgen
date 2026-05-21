// `SV-EXH-PROOF.3.3.4.b.5.1.5.a` — the predicate-expression sub-language.
//
// A `@predicate_def:` block (Stage 3 of the lifecycle protocol — composed
// predicates, see `CONTEXT_AWARE_PARSING_DESIGN.md` §4.4 and
// `PGEN_SEMANTIC_STORE_SCHEMA_LANGUAGE_SPEC.md` §7) carries a `body:` field
// whose value is a small boolean expression over the four built-in
// predicate primitives:
//
//   @predicate_def: {
//     name: receiver_is_array,
//     args: [receiver_path],
//     body: "resolve_path($receiver_path).attribute('type_kind') in ['array','queue','dynamic_array','assoc_array']"
//   }
//
// This module owns the AST for that expression plus the lexer + recursive-
// descent parser. The EVALUATOR (which needs `SemanticRuntimeState`) and the
// `@predicate_def:` annotation integration land in the companion sub-leaf
// `.b.5.1.5.b`; this module is deliberately self-contained and depends only
// on `std` so the parser can be unit-tested in isolation.
//
// Body expression language (deliberately small — predicates are *decisions*,
// not computations; no recursion, no arithmetic):
//
//   expr        := or_expr
//   or_expr     := and_expr ( "||" and_expr )*
//   and_expr    := not_expr ( "&&" not_expr )*
//   not_expr    := "!" not_expr | cmp_expr
//   cmp_expr    := primary
//                | value cmp_op value
//                | value "in" "[" value ( "," value )* "]"
//   primary     := "(" expr ")" | bool_call
//   bool_call   := primitive_call                  # used as a boolean
//   value       := primitive_call "." "attribute" "(" string ")"   # value-bearing
//                | "$" ident                       # arg reference
//                | string | integer | ident        # literals
//   primitive_call := ident "(" ( value ( "," value )* )? ")"
//   cmp_op      := "==" | "!=" | "<" | "<=" | ">" | ">="
//
// String literals accept BOTH single and double quotes — so a `@predicate_def`
// `body:` field can be wrapped in double quotes at the annotation surface
// while its inner string literals use single quotes, sidestepping the
// bootstrap annotation language's lack of `\"` escaping
// (see memory `feedback_annotation_no_dquote_escape`).

use std::fmt;

// =============================================================================
// AST
// =============================================================================

/// A boolean-valued predicate expression — the type of a `@predicate_def`
/// `body:` and of every logical sub-term.
#[derive(Debug, Clone, PartialEq)]
pub enum PredicateExpr {
    /// A bare primitive call used as a boolean (`has_fact(...)`,
    /// `fact_count_at_least(...)`, `resolve_path(...)` interpreted as
    /// "did it resolve?").
    Call(PrimitiveCall),
    /// `! <expr>`
    Not(Box<PredicateExpr>),
    /// `<lhs> && <rhs>`
    And(Box<PredicateExpr>, Box<PredicateExpr>),
    /// `<lhs> || <rhs>`
    Or(Box<PredicateExpr>, Box<PredicateExpr>),
    /// `<lhs> <op> <rhs>` — a value comparison.
    Compare {
        lhs: PredicateValue,
        op: CompareOp,
        rhs: PredicateValue,
    },
    /// `<lhs> in [ <set...> ]` — set membership.
    In {
        lhs: PredicateValue,
        set: Vec<PredicateValue>,
    },
}

/// A value-valued sub-expression — appears on either side of a comparison
/// or inside an `in [...]` set.
#[derive(Debug, Clone, PartialEq)]
pub enum PredicateValue {
    /// `<primitive_call>.attribute("key")` — drills into the fact a
    /// `resolve_path` call resolved, reading one attribute.
    AttributeOf { call: PrimitiveCall, key: String },
    /// `$name` — an argument reference, substituted at call time.
    ArgRef(String),
    /// A quoted string literal (`'text'` or `"text"`).
    StringLit(String),
    /// An integer literal.
    IntLit(i64),
    /// A bare identifier literal (e.g. `array`, `class`).
    IdentLit(String),
}

/// A call to one of the four built-in predicate primitives.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimitiveCall {
    pub name: String,
    pub args: Vec<PredicateValue>,
}

/// Comparison operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl fmt::Display for CompareOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            CompareOp::Eq => "==",
            CompareOp::Ne => "!=",
            CompareOp::Lt => "<",
            CompareOp::Le => "<=",
            CompareOp::Gt => ">",
            CompareOp::Ge => ">=",
        })
    }
}

// =============================================================================
// Lexer
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Ident(String),
    Str(String),
    Int(i64),
    Dollar,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    Dot,
    Bang,
    AmpAmp,
    PipePipe,
    EqEq,
    BangEq,
    Lt,
    Le,
    Gt,
    Ge,
    In,
}

fn lex(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            ' ' | '\t' | '\r' | '\n' => {
                i += 1;
            }
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            '[' => {
                tokens.push(Token::LBracket);
                i += 1;
            }
            ']' => {
                tokens.push(Token::RBracket);
                i += 1;
            }
            ',' => {
                tokens.push(Token::Comma);
                i += 1;
            }
            '.' => {
                tokens.push(Token::Dot);
                i += 1;
            }
            '$' => {
                tokens.push(Token::Dollar);
                i += 1;
            }
            '!' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::BangEq);
                    i += 2;
                } else {
                    tokens.push(Token::Bang);
                    i += 1;
                }
            }
            '&' => {
                if i + 1 < chars.len() && chars[i + 1] == '&' {
                    tokens.push(Token::AmpAmp);
                    i += 2;
                } else {
                    return Err("predicate body: single '&' is not an operator (use '&&')".to_string());
                }
            }
            '|' => {
                if i + 1 < chars.len() && chars[i + 1] == '|' {
                    tokens.push(Token::PipePipe);
                    i += 2;
                } else {
                    return Err("predicate body: single '|' is not an operator (use '||')".to_string());
                }
            }
            '=' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::EqEq);
                    i += 2;
                } else {
                    return Err("predicate body: single '=' is not an operator (use '==')".to_string());
                }
            }
            '<' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::Le);
                    i += 2;
                } else {
                    tokens.push(Token::Lt);
                    i += 1;
                }
            }
            '>' => {
                if i + 1 < chars.len() && chars[i + 1] == '=' {
                    tokens.push(Token::Ge);
                    i += 2;
                } else {
                    tokens.push(Token::Gt);
                    i += 1;
                }
            }
            '\'' | '"' => {
                let quote = c;
                i += 1;
                let mut s = String::new();
                let mut closed = false;
                while i < chars.len() {
                    if chars[i] == quote {
                        closed = true;
                        i += 1;
                        break;
                    }
                    s.push(chars[i]);
                    i += 1;
                }
                if !closed {
                    return Err(format!(
                        "predicate body: unterminated string literal (opened with {})",
                        quote
                    ));
                }
                tokens.push(Token::Str(s));
            }
            '0'..='9' => {
                let mut s = String::new();
                while i < chars.len() && chars[i].is_ascii_digit() {
                    s.push(chars[i]);
                    i += 1;
                }
                let value = s
                    .parse::<i64>()
                    .map_err(|e| format!("predicate body: bad integer literal '{}': {}", s, e))?;
                tokens.push(Token::Int(value));
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let mut s = String::new();
                while i < chars.len()
                    && (chars[i].is_ascii_alphanumeric() || chars[i] == '_')
                {
                    s.push(chars[i]);
                    i += 1;
                }
                if s == "in" {
                    tokens.push(Token::In);
                } else {
                    tokens.push(Token::Ident(s));
                }
            }
            other => {
                return Err(format!(
                    "predicate body: unexpected character '{}'",
                    other
                ));
            }
        }
    }
    Ok(tokens)
}

// =============================================================================
// Parser (recursive descent)
// =============================================================================

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        match self.advance() {
            Some(ref t) if t == expected => Ok(()),
            Some(t) => Err(format!(
                "predicate body: expected {:?}, found {:?}",
                expected, t
            )),
            None => Err(format!(
                "predicate body: expected {:?}, found end of input",
                expected
            )),
        }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    // cmp_expr := value cmp_op value
    //           | value "in" "[" value ( "," value )* "]"
    //
    // `parse_cmp` is only ever entered with a value-leading term — the
    // boolean cases (parenthesised groups, bare primitive calls) are
    // intercepted upstream by `parse_primary_with_bool_calls`. So the term
    // here always begins with a `value`, legal only as the LHS of a
    // comparison or an `in [...]` membership test.
    fn parse_cmp(&mut self) -> Result<PredicateExpr, String> {
        let value = self.parse_value()?;

        match self.peek() {
            Some(Token::EqEq | Token::BangEq | Token::Lt | Token::Le | Token::Gt | Token::Ge) => {
                let op = match self.advance().unwrap() {
                    Token::EqEq => CompareOp::Eq,
                    Token::BangEq => CompareOp::Ne,
                    Token::Lt => CompareOp::Lt,
                    Token::Le => CompareOp::Le,
                    Token::Gt => CompareOp::Gt,
                    Token::Ge => CompareOp::Ge,
                    _ => unreachable!(),
                };
                let rhs = self.parse_value()?;
                Ok(PredicateExpr::Compare { lhs: value, op, rhs })
            }
            Some(Token::In) => {
                self.advance();
                self.expect(&Token::LBracket)?;
                let mut set = Vec::new();
                if !matches!(self.peek(), Some(Token::RBracket)) {
                    loop {
                        set.push(self.parse_value()?);
                        match self.peek() {
                            Some(Token::Comma) => {
                                self.advance();
                            }
                            _ => break,
                        }
                    }
                }
                self.expect(&Token::RBracket)?;
                Ok(PredicateExpr::In { lhs: value, set })
            }
            _ => {
                // No comparison follows — the value must itself be a bare
                // boolean primitive call. `AttributeOf` / literals / arg
                // refs are not booleans, so they are a type error here.
                match value {
                    PredicateValue::AttributeOf { .. } => Err(
                        "predicate body: an `.attribute(...)` value cannot stand alone as a boolean — use it in a comparison or `in [...]`".to_string(),
                    ),
                    PredicateValue::ArgRef(name) => Err(format!(
                        "predicate body: `${}` is a value, not a boolean — use it in a comparison or `in [...]`",
                        name
                    )),
                    PredicateValue::StringLit(_) | PredicateValue::IntLit(_) => Err(
                        "predicate body: a literal cannot stand alone as a boolean".to_string(),
                    ),
                    PredicateValue::IdentLit(_) => {
                        // A bare identifier that wasn't a call is a dangling
                        // identifier — only `ident(...)` is a boolean term.
                        Err(
                            "predicate body: a bare identifier is not a boolean — did you mean a primitive call `name(...)`?".to_string(),
                        )
                    }
                }
            }
        }
    }

    // value := primitive_call "." "attribute" "(" string ")"
    //        | "$" ident
    //        | string | integer | ident
    //
    // Returns a `PredicateValue`. NOTE: a bare `ident(...)` primitive call
    // that is NOT followed by `.attribute(...)` is returned as part of a
    // synthetic `IdentLit`-tagged path so `parse_cmp` can promote it back
    // to a boolean `Call`. To keep the AST clean we instead special-case:
    // when a primitive call appears with no `.attribute`, this function
    // signals it via the `bool_call` out-param mechanism below.
    fn parse_value(&mut self) -> Result<PredicateValue, String> {
        match self.peek().cloned() {
            Some(Token::Dollar) => {
                self.advance();
                match self.advance() {
                    // Named arg ref: `$receiver_path`.
                    Some(Token::Ident(name)) => Ok(PredicateValue::ArgRef(name)),
                    // Positional arg ref: `$1`, `$2` — the digit sequence
                    // is the arg name (mirrors the existing rule-reference
                    // surface where `$1` is a positional capture).
                    Some(Token::Int(n)) => Ok(PredicateValue::ArgRef(n.to_string())),
                    other => Err(format!(
                        "predicate body: expected identifier or integer after '$', found {:?}",
                        other
                    )),
                }
            }
            Some(Token::Str(s)) => {
                self.advance();
                Ok(PredicateValue::StringLit(s))
            }
            Some(Token::Int(n)) => {
                self.advance();
                Ok(PredicateValue::IntLit(n))
            }
            Some(Token::Ident(name)) => {
                self.advance();
                if matches!(self.peek(), Some(Token::LParen)) {
                    // A primitive call. It may be followed by
                    // `.attribute("key")` (→ AttributeOf, a value) or stand
                    // alone (→ a boolean Call, which `parse_cmp` reconstructs
                    // from the `IdentLit`-free path). To keep `parse_value`
                    // pure-value, a bare call is wrapped as an AttributeOf
                    // ONLY when `.attribute` follows; otherwise we must hand
                    // the call back. We do that by parsing the call and, if
                    // no `.attribute` follows, returning a sentinel the
                    // caller turns into a boolean. Implemented via the
                    // dedicated `parse_primitive_call` + the
                    // `value_or_bool_call` split — see `parse_cmp`'s primary
                    // handling. Here, a bare call with no `.attribute` is a
                    // parse error in *value position*; `parse_cmp` routes
                    // bare calls through `parse_bool_or_value` instead.
                    let call = self.parse_primitive_call_tail(name)?;
                    if matches!(self.peek(), Some(Token::Dot)) {
                        self.advance();
                        // expect `attribute`
                        match self.advance() {
                            Some(Token::Ident(ref m)) if m == "attribute" => {}
                            other => {
                                return Err(format!(
                                    "predicate body: expected `.attribute(...)` after a primitive call, found {:?}",
                                    other
                                ));
                            }
                        }
                        self.expect(&Token::LParen)?;
                        let key = match self.advance() {
                            Some(Token::Str(k)) => k,
                            other => {
                                return Err(format!(
                                    "predicate body: `.attribute(...)` requires a string key, found {:?}",
                                    other
                                ));
                            }
                        };
                        self.expect(&Token::RParen)?;
                        Ok(PredicateValue::AttributeOf { call, key })
                    } else {
                        // Bare call in value position is not a value.
                        Err(format!(
                            "predicate body: primitive call `{}(...)` produces a boolean, not a value — to read a value use `.attribute(\"key\")`",
                            call.name
                        ))
                    }
                } else {
                    // Bare identifier literal.
                    Ok(PredicateValue::IdentLit(name))
                }
            }
            other => Err(format!(
                "predicate body: expected a value, found {:?}",
                other
            )),
        }
    }

    /// Parse the `( args )` tail of a primitive call whose name has already
    /// been consumed.
    fn parse_primitive_call_tail(&mut self, name: String) -> Result<PrimitiveCall, String> {
        self.expect(&Token::LParen)?;
        let mut args = Vec::new();
        if !matches!(self.peek(), Some(Token::RParen)) {
            loop {
                args.push(self.parse_value()?);
                match self.peek() {
                    Some(Token::Comma) => {
                        self.advance();
                    }
                    _ => break,
                }
            }
        }
        self.expect(&Token::RParen)?;
        Ok(PrimitiveCall { name, args })
    }
}

/// Parse a `@predicate_def` `body:` expression string into a `PredicateExpr`.
///
/// Returns a precise `Err(String)` on any lexical or syntactic problem; the
/// message is suitable for surfacing as a grammar-compile-time diagnostic.
pub fn parse_predicate_expression(input: &str) -> Result<PredicateExpr, String> {
    let tokens = lex(input)?;
    if tokens.is_empty() {
        return Err("predicate body: empty expression".to_string());
    }
    let mut parser = Parser { tokens, pos: 0 };
    let expr = parse_top_level(&mut parser)?;
    if !parser.at_end() {
        return Err(format!(
            "predicate body: trailing tokens after a complete expression (next: {:?})",
            parser.peek()
        ));
    }
    Ok(expr)
}

/// Top-level: the body is a boolean expression. A bare primitive call at
/// top level (e.g. `has_fact(variable_binding, $1)`) is a boolean term;
/// `parse_cmp` rejects bare calls in *value* position, so we special-case
/// the "primitive call used as boolean" path here and inside `parse_cmp`'s
/// primary handling by re-routing.
fn parse_top_level(parser: &mut Parser) -> Result<PredicateExpr, String> {
    parse_or_with_bool_calls(parser)
}

// The boolean grammar layered so that a bare `ident(...)` is a boolean
// `Call`. We re-implement the or/and/not/primary chain here to thread the
// "bare call is boolean" rule, which `parse_value` cannot express alone.

fn parse_or_with_bool_calls(parser: &mut Parser) -> Result<PredicateExpr, String> {
    let mut left = parse_and_with_bool_calls(parser)?;
    while matches!(parser.peek(), Some(Token::PipePipe)) {
        parser.advance();
        let right = parse_and_with_bool_calls(parser)?;
        left = PredicateExpr::Or(Box::new(left), Box::new(right));
    }
    Ok(left)
}

fn parse_and_with_bool_calls(parser: &mut Parser) -> Result<PredicateExpr, String> {
    let mut left = parse_not_with_bool_calls(parser)?;
    while matches!(parser.peek(), Some(Token::AmpAmp)) {
        parser.advance();
        let right = parse_not_with_bool_calls(parser)?;
        left = PredicateExpr::And(Box::new(left), Box::new(right));
    }
    Ok(left)
}

fn parse_not_with_bool_calls(parser: &mut Parser) -> Result<PredicateExpr, String> {
    if matches!(parser.peek(), Some(Token::Bang)) {
        parser.advance();
        let inner = parse_not_with_bool_calls(parser)?;
        return Ok(PredicateExpr::Not(Box::new(inner)));
    }
    parse_primary_with_bool_calls(parser)
}

fn parse_primary_with_bool_calls(parser: &mut Parser) -> Result<PredicateExpr, String> {
    // Parenthesised boolean.
    if matches!(parser.peek(), Some(Token::LParen)) {
        parser.advance();
        let inner = parse_or_with_bool_calls(parser)?;
        parser.expect(&Token::RParen)?;
        return Ok(inner);
    }
    // A bare `ident(...)` with no `.attribute` is a boolean primitive call.
    // Detect it by lookahead: Ident followed by LParen, and after the call
    // there is NO Dot.
    if let Some(Token::Ident(name)) = parser.peek().cloned() {
        // Peek for `(`
        if parser.tokens.get(parser.pos + 1) == Some(&Token::LParen) {
            // Tentatively consume the call.
            let saved = parser.pos;
            parser.advance(); // ident
            let call = parser.parse_primitive_call_tail(name)?;
            if matches!(parser.peek(), Some(Token::Dot)) {
                // It is actually a value (AttributeOf) used as the LHS of a
                // comparison/in. Rewind and let `parse_cmp` handle it.
                parser.pos = saved;
                return parser.parse_cmp();
            }
            // After a bare boolean call, a comparison operator would be a
            // type error (a boolean isn't comparable); reject early for a
            // clearer message.
            if matches!(
                parser.peek(),
                Some(
                    Token::EqEq
                        | Token::BangEq
                        | Token::Lt
                        | Token::Le
                        | Token::Gt
                        | Token::Ge
                        | Token::In
                )
            ) {
                return Err(format!(
                    "predicate body: primitive call `{}(...)` produces a boolean and cannot be compared — to compare a value, use `.attribute(\"key\")`",
                    call.name
                ));
            }
            return Ok(PredicateExpr::Call(call));
        }
    }
    // Otherwise it is a comparison / membership term beginning with a value.
    parser.parse_cmp()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arg(name: &str) -> PredicateValue {
        PredicateValue::ArgRef(name.to_string())
    }

    #[test]
    fn parses_bare_primitive_call() {
        let expr = parse_predicate_expression("has_fact(variable_binding, $1)").expect("parse");
        match expr {
            PredicateExpr::Call(call) => {
                assert_eq!(call.name, "has_fact");
                assert_eq!(call.args.len(), 2);
                assert_eq!(call.args[0], PredicateValue::IdentLit("variable_binding".to_string()));
                assert_eq!(call.args[1], arg("1"));
            }
            other => panic!("expected Call, got {:?}", other),
        }
    }

    #[test]
    fn parses_not() {
        let expr = parse_predicate_expression("!has_fact(k, $n)").expect("parse");
        assert!(matches!(expr, PredicateExpr::Not(_)));
    }

    #[test]
    fn parses_and_or_precedence() {
        // && binds tighter than ||: a || b && c  ==  a || (b && c)
        let expr = parse_predicate_expression(
            "has_fact(a, $x) || has_fact(b, $x) && has_fact(c, $x)",
        )
        .expect("parse");
        match expr {
            PredicateExpr::Or(lhs, rhs) => {
                assert!(matches!(*lhs, PredicateExpr::Call(_)));
                assert!(matches!(*rhs, PredicateExpr::And(_, _)));
            }
            other => panic!("expected Or at top, got {:?}", other),
        }
    }

    #[test]
    fn parses_attribute_in_set() {
        // The motivating receiver_is_array body.
        let expr = parse_predicate_expression(
            "resolve_path($p).attribute('type_kind') in ['array', 'queue', 'dynamic_array']",
        )
        .expect("parse");
        match expr {
            PredicateExpr::In { lhs, set } => {
                match lhs {
                    PredicateValue::AttributeOf { call, key } => {
                        assert_eq!(call.name, "resolve_path");
                        assert_eq!(call.args, vec![arg("p")]);
                        assert_eq!(key, "type_kind");
                    }
                    other => panic!("expected AttributeOf lhs, got {:?}", other),
                }
                assert_eq!(set.len(), 3);
                assert_eq!(set[0], PredicateValue::StringLit("array".to_string()));
            }
            other => panic!("expected In, got {:?}", other),
        }
    }

    #[test]
    fn parses_comparison() {
        let expr = parse_predicate_expression(
            "resolve_path($p).attribute('type_kind') == 'class'",
        )
        .expect("parse");
        match expr {
            PredicateExpr::Compare { op, rhs, .. } => {
                assert_eq!(op, CompareOp::Eq);
                assert_eq!(rhs, PredicateValue::StringLit("class".to_string()));
            }
            other => panic!("expected Compare, got {:?}", other),
        }
    }

    #[test]
    fn parses_all_comparison_operators() {
        for (text, op) in [
            ("== 'x'", CompareOp::Eq),
            ("!= 'x'", CompareOp::Ne),
            ("< 5", CompareOp::Lt),
            ("<= 5", CompareOp::Le),
            ("> 5", CompareOp::Gt),
            (">= 5", CompareOp::Ge),
        ] {
            let body = format!("resolve_path($p).attribute('n') {}", text);
            let expr = parse_predicate_expression(&body).unwrap_or_else(|e| panic!("{}: {}", body, e));
            match expr {
                PredicateExpr::Compare { op: parsed, .. } => assert_eq!(parsed, op),
                other => panic!("{}: expected Compare, got {:?}", body, other),
            }
        }
    }

    #[test]
    fn parses_parenthesised_grouping() {
        // (a || b) && c
        let expr = parse_predicate_expression(
            "(has_fact(a, $x) || has_fact(b, $x)) && has_fact(c, $x)",
        )
        .expect("parse");
        match expr {
            PredicateExpr::And(lhs, _) => assert!(matches!(*lhs, PredicateExpr::Or(_, _))),
            other => panic!("expected And at top, got {:?}", other),
        }
    }

    #[test]
    fn accepts_single_and_double_quoted_strings() {
        let a = parse_predicate_expression("resolve_path($p).attribute('k') == 'v'").expect("single");
        let b = parse_predicate_expression("resolve_path($p).attribute(\"k\") == \"v\"").expect("double");
        assert_eq!(a, b);
    }

    #[test]
    fn rejects_empty_expression() {
        assert!(parse_predicate_expression("").is_err());
        assert!(parse_predicate_expression("   ").is_err());
    }

    #[test]
    fn rejects_trailing_tokens() {
        let err = parse_predicate_expression("has_fact(k, $n) garbage").expect_err("trailing");
        assert!(err.contains("trailing"), "got: {}", err);
    }

    #[test]
    fn rejects_unterminated_string() {
        let err = parse_predicate_expression("resolve_path($p).attribute('k").expect_err("unterminated");
        assert!(err.contains("unterminated"), "got: {}", err);
    }

    #[test]
    fn rejects_single_ampersand() {
        let err = parse_predicate_expression("has_fact(k,$n) & has_fact(k,$m)").expect_err("&");
        assert!(err.contains("&&"), "got: {}", err);
    }

    #[test]
    fn rejects_bare_value_as_boolean() {
        // A bare arg ref or literal cannot be a boolean term.
        assert!(parse_predicate_expression("$x").is_err());
        assert!(parse_predicate_expression("'literal'").is_err());
        assert!(parse_predicate_expression("42").is_err());
    }

    #[test]
    fn rejects_comparing_a_boolean_call() {
        // has_fact(...) is a boolean; comparing it is a type error.
        let err = parse_predicate_expression("has_fact(k, $n) == 'x'").expect_err("bool compare");
        assert!(err.contains("boolean"), "got: {}", err);
    }

    #[test]
    fn rejects_bare_call_in_value_position() {
        // resolve_path used as a value without .attribute is an error.
        let err = parse_predicate_expression(
            "resolve_path($p) == 'x'",
        )
        .expect_err("bare call value");
        assert!(err.contains("boolean"), "got: {}", err);
    }
}
