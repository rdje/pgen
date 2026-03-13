//! Initial synthesizable-RTL frontend baseline for planned RTLSyn work.
//!
//! The current scope is intentionally narrow:
//! - module headers with optional parameter and ANSI port lists
//! - parameter/localparam declarations
//! - packed ranges backed by `rtl_const_expr`
//! - net declarations, continuous assigns, and basic procedural blocks
//! - explicit `generate` regions with `if` / `for` constructs
//!
//! This is not a full SystemVerilog frontend. The goal is to establish the
//! frontend/elaboration boundary and wire constant-expression parsing into it.

use rtl_const_expr::{EvalError, Expr, parse_expression};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendError {
    pub message: String,
    pub position: usize,
}

impl FrontendError {
    fn new(message: impl Into<String>, position: usize) -> Self {
        Self {
            message: message.into(),
            position,
        }
    }
}

impl fmt::Display for FrontendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at byte {}", self.message, self.position)
    }
}

impl Error for FrontendError {}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Design {
    pub modules: Vec<Module>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub name: String,
    pub parameters: Vec<ParameterDecl>,
    pub ports: Vec<PortDecl>,
    pub items: Vec<ModuleItem>,
}

impl Module {
    pub fn evaluate_constant_env(
        &self,
        overrides: &HashMap<String, i64>,
    ) -> Result<HashMap<String, i64>, FrontendError> {
        let mut symbols = HashMap::new();
        for (name, value) in overrides {
            symbols.insert(name.clone(), *value);
        }

        for parameter in &self.parameters {
            apply_parameter_decl(&mut symbols, overrides, parameter)?;
        }
        for item in &self.items {
            if let ModuleItem::ParameterDecl(parameter) = item {
                apply_parameter_decl(&mut symbols, overrides, parameter)?;
            }
        }

        Ok(symbols)
    }
}

fn apply_parameter_decl(
    symbols: &mut HashMap<String, i64>,
    overrides: &HashMap<String, i64>,
    decl: &ParameterDecl,
) -> Result<(), FrontendError> {
    if decl.flavor == ParameterFlavor::Parameter {
        if let Some(value) = overrides.get(&decl.name) {
            symbols.insert(decl.name.clone(), *value);
            return Ok(());
        }
    }

    let Some(expr) = decl.value.as_ref() else {
        return Err(FrontendError::new(
            format!(
                "missing default value for constant declaration '{}'",
                decl.name
            ),
            0,
        ));
    };

    let value = expr.eval(symbols).map_err(|err| {
        FrontendError::new(
            format!(
                "failed to evaluate constant declaration '{}': {}",
                decl.name, err.message
            ),
            err.position,
        )
    })?;
    symbols.insert(decl.name.clone(), value);
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataType {
    pub keyword: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedRange {
    pub msb: Expr,
    pub lsb: Expr,
}

impl PackedRange {
    pub fn width(&self, symbols: &HashMap<String, i64>) -> Result<i64, FrontendError> {
        let msb = self.msb.eval(symbols).map_err(map_eval_error)?;
        let lsb = self.lsb.eval(symbols).map_err(map_eval_error)?;
        Ok((msb - lsb).abs() + 1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterFlavor {
    Parameter,
    Localparam,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterDecl {
    pub flavor: ParameterFlavor,
    pub data_type: Option<DataType>,
    pub name: String,
    pub value: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortDirection {
    Input,
    Output,
    Inout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortDecl {
    pub direction: PortDirection,
    pub data_type: Option<DataType>,
    pub packed_range: Option<PackedRange>,
    pub name: String,
}

impl PortDecl {
    pub fn width(&self, symbols: &HashMap<String, i64>) -> Result<Option<i64>, FrontendError> {
        self.packed_range
            .as_ref()
            .map(|range| range.width(symbols))
            .transpose()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetKind {
    Logic,
    Wire,
    Reg,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetDecl {
    pub kind: NetKind,
    pub packed_range: Option<PackedRange>,
    pub names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenvarDecl {
    pub names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuousAssign {
    pub target: String,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProceduralKind {
    AlwaysComb,
    AlwaysStar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProceduralBlock {
    pub kind: ProceduralKind,
    pub statement: Statement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssignmentKind {
    Blocking,
    NonBlocking,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Block {
        label: Option<String>,
        statements: Vec<Statement>,
    },
    If {
        condition: Expr,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    Assignment {
        target: String,
        kind: AssignmentKind,
        value: Expr,
    },
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateRegion {
    pub items: Vec<ModuleItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateIf {
    pub condition: Expr,
    pub then_label: Option<String>,
    pub then_items: Vec<ModuleItem>,
    pub else_label: Option<String>,
    pub else_items: Vec<ModuleItem>,
}

impl GenerateIf {
    pub fn is_enabled(&self, symbols: &HashMap<String, i64>) -> Result<bool, FrontendError> {
        Ok(self.condition.eval(symbols).map_err(map_eval_error)? != 0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateFor {
    pub index_name: String,
    pub declares_genvar: bool,
    pub init: Expr,
    pub condition: Expr,
    pub step: Expr,
    pub label: Option<String>,
    pub body_items: Vec<ModuleItem>,
}

impl GenerateFor {
    pub fn iteration_values(
        &self,
        symbols: &HashMap<String, i64>,
        max_iterations: usize,
    ) -> Result<Vec<i64>, FrontendError> {
        let mut local = symbols.clone();
        let initial = self.init.eval(symbols).map_err(map_eval_error)?;
        local.insert(self.index_name.clone(), initial);

        let mut values = Vec::new();
        for _ in 0..max_iterations {
            let current = *local
                .get(&self.index_name)
                .ok_or_else(|| FrontendError::new("missing loop index value", 0))?;
            if self.condition.eval(&local).map_err(map_eval_error)? == 0 {
                return Ok(values);
            }
            values.push(current);
            let next = self.step.eval(&local).map_err(map_eval_error)?;
            local.insert(self.index_name.clone(), next);
        }

        Err(FrontendError::new(
            format!(
                "generate-for loop exceeded max_iterations={} while unrolling '{}'",
                max_iterations, self.index_name
            ),
            0,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleItem {
    ParameterDecl(ParameterDecl),
    GenvarDecl(GenvarDecl),
    NetDecl(NetDecl),
    ContinuousAssign(ContinuousAssign),
    ProceduralBlock(ProceduralBlock),
    GenerateRegion(GenerateRegion),
    GenerateIf(GenerateIf),
    GenerateFor(GenerateFor),
}

pub fn parse_design(input: &str) -> Result<Design, FrontendError> {
    let tokens = Lexer::new(input).tokenize()?;
    Parser::new(input, tokens).parse_design()
}

pub fn parse_module(input: &str) -> Result<Module, FrontendError> {
    let design = parse_design(input)?;
    match design.modules.as_slice() {
        [module] => Ok(module.clone()),
        [] => Err(FrontendError::new(
            "expected exactly one module, found none",
            0,
        )),
        _ => Err(FrontendError::new(
            format!(
                "expected exactly one module, found {}",
                design.modules.len()
            ),
            0,
        )),
    }
}

fn map_eval_error(err: EvalError) -> FrontendError {
    FrontendError::new(err.message, err.position)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenKind {
    Ident(String),
    Number(String),
    Symbol(char),
    Operator(&'static str),
    End,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token {
    kind: TokenKind,
    start: usize,
    end: usize,
}

struct Lexer<'a> {
    input: &'a str,
    index: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, index: 0 }
    }

    fn tokenize(mut self) -> Result<Vec<Token>, FrontendError> {
        let mut tokens = Vec::new();
        loop {
            self.skip_ws_and_comments()?;
            let start = self.index;
            let Some(ch) = self.peek_char() else {
                tokens.push(Token {
                    kind: TokenKind::End,
                    start,
                    end: start,
                });
                break;
            };

            let token = if is_ident_start(ch) {
                self.lex_identifier()
            } else if ch.is_ascii_digit() {
                self.lex_number()
            } else {
                self.lex_punctuation()?
            };
            tokens.push(token);
        }

        Ok(tokens)
    }

    fn skip_ws_and_comments(&mut self) -> Result<(), FrontendError> {
        loop {
            while matches!(self.peek_char(), Some(ch) if ch.is_ascii_whitespace()) {
                self.bump_char();
            }

            if self.peek_char() == Some('/') && self.peek_next_char() == Some('/') {
                while let Some(ch) = self.peek_char() {
                    self.bump_char();
                    if ch == '\n' {
                        break;
                    }
                }
                continue;
            }

            if self.peek_char() == Some('/') && self.peek_next_char() == Some('*') {
                let start = self.index;
                self.bump_char();
                self.bump_char();
                loop {
                    match (self.peek_char(), self.peek_next_char()) {
                        (Some('*'), Some('/')) => {
                            self.bump_char();
                            self.bump_char();
                            break;
                        }
                        (Some(_), _) => {
                            self.bump_char();
                        }
                        (None, _) => {
                            return Err(FrontendError::new("unterminated block comment", start));
                        }
                    }
                }
                continue;
            }

            break;
        }

        Ok(())
    }

    fn lex_identifier(&mut self) -> Token {
        let start = self.index;
        let mut text = String::new();
        while let Some(ch) = self.peek_char() {
            if !is_ident_continue(ch) {
                break;
            }
            text.push(ch);
            self.bump_char();
        }
        Token {
            kind: TokenKind::Ident(text),
            start,
            end: self.index,
        }
    }

    fn lex_number(&mut self) -> Token {
        let start = self.index;
        let mut text = String::new();
        while let Some(ch) = self.peek_char() {
            if !(ch.is_ascii_digit() || ch == '_') {
                break;
            }
            text.push(ch);
            self.bump_char();
        }

        if self.peek_char() == Some('\'') {
            text.push('\'');
            self.bump_char();
            if matches!(self.peek_char(), Some('s' | 'S')) {
                let ch = self.peek_char().unwrap_or('s');
                text.push(ch);
                self.bump_char();
            }
            if let Some(ch) = self.peek_char() {
                text.push(ch);
                self.bump_char();
            }
            while let Some(ch) = self.peek_char() {
                if !(ch.is_ascii_alphanumeric() || ch == '_') {
                    break;
                }
                text.push(ch);
                self.bump_char();
            }
        }

        Token {
            kind: TokenKind::Number(text),
            start,
            end: self.index,
        }
    }

    fn lex_punctuation(&mut self) -> Result<Token, FrontendError> {
        let start = self.index;
        let Some(ch) = self.peek_char() else {
            return Err(FrontendError::new("unexpected end of input", start));
        };

        let token = match ch {
            '<' => {
                self.bump_char();
                if self.peek_char() == Some('=') {
                    self.bump_char();
                    TokenKind::Operator("<=")
                } else if self.peek_char() == Some('<') {
                    self.bump_char();
                    TokenKind::Operator("<<")
                } else {
                    TokenKind::Symbol('<')
                }
            }
            '>' => {
                self.bump_char();
                if self.peek_char() == Some('=') {
                    self.bump_char();
                    TokenKind::Operator(">=")
                } else if self.peek_char() == Some('>') {
                    self.bump_char();
                    TokenKind::Operator(">>")
                } else {
                    TokenKind::Symbol('>')
                }
            }
            '=' => {
                self.bump_char();
                if self.peek_char() == Some('=') {
                    self.bump_char();
                    TokenKind::Operator("==")
                } else {
                    TokenKind::Symbol('=')
                }
            }
            '!' => {
                self.bump_char();
                if self.peek_char() == Some('=') {
                    self.bump_char();
                    TokenKind::Operator("!=")
                } else {
                    TokenKind::Symbol('!')
                }
            }
            '&' => {
                self.bump_char();
                if self.peek_char() == Some('&') {
                    self.bump_char();
                    TokenKind::Operator("&&")
                } else {
                    TokenKind::Symbol('&')
                }
            }
            '|' => {
                self.bump_char();
                if self.peek_char() == Some('|') {
                    self.bump_char();
                    TokenKind::Operator("||")
                } else {
                    TokenKind::Symbol('|')
                }
            }
            _ => {
                self.bump_char();
                TokenKind::Symbol(ch)
            }
        };

        Ok(Token {
            kind: token,
            start,
            end: self.index,
        })
    }

    fn peek_char(&self) -> Option<char> {
        self.input.get(self.index..)?.chars().next()
    }

    fn peek_next_char(&self) -> Option<char> {
        let mut chars = self.input.get(self.index..)?.chars();
        chars.next()?;
        chars.next()
    }

    fn bump_char(&mut self) {
        if let Some(ch) = self.peek_char() {
            self.index += ch.len_utf8();
        }
    }
}

struct Parser<'a> {
    input: &'a str,
    tokens: Vec<Token>,
    index: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str, tokens: Vec<Token>) -> Self {
        Self {
            input,
            tokens,
            index: 0,
        }
    }

    fn parse_design(mut self) -> Result<Design, FrontendError> {
        let mut modules = Vec::new();
        while !self.is_end() {
            modules.push(self.parse_module()?);
        }
        Ok(Design { modules })
    }

    fn parse_module(&mut self) -> Result<Module, FrontendError> {
        self.expect_keyword("module")?;
        let name = self.expect_identifier()?;
        let parameters = if self.consume_symbol('#') {
            self.expect_symbol('(')?;
            self.parse_parameter_list(')')?
        } else {
            Vec::new()
        };

        let ports = if self.consume_symbol('(') {
            self.parse_port_list()?
        } else {
            Vec::new()
        };

        self.expect_symbol(';')?;
        let mut items = Vec::new();
        while !self.peek_keyword("endmodule") {
            items.extend(self.parse_item_group(false)?);
        }
        self.expect_keyword("endmodule")?;

        Ok(Module {
            name,
            parameters,
            ports,
            items,
        })
    }

    fn parse_parameter_list(
        &mut self,
        terminator: char,
    ) -> Result<Vec<ParameterDecl>, FrontendError> {
        let mut params = Vec::new();
        if self.consume_symbol(terminator) {
            return Ok(params);
        }

        let mut current_flavor: Option<ParameterFlavor> = None;
        loop {
            if self.consume_keyword("parameter") {
                current_flavor = Some(ParameterFlavor::Parameter);
            } else if self.consume_keyword("localparam") {
                current_flavor = Some(ParameterFlavor::Localparam);
            }

            let Some(flavor) = current_flavor.clone() else {
                return Err(FrontendError::new(
                    "expected parameter/localparam declaration",
                    self.current().start,
                ));
            };

            params.push(self.parse_parameter_decl(flavor, &[',', terminator])?);
            if !self.consume_symbol(',') {
                break;
            }
        }

        self.expect_symbol(terminator)?;
        Ok(params)
    }

    fn parse_parameter_decl(
        &mut self,
        flavor: ParameterFlavor,
        expr_terminators: &[char],
    ) -> Result<ParameterDecl, FrontendError> {
        let data_type = self.parse_optional_data_type();
        let name = self.expect_identifier()?;
        let value = if self.consume_symbol('=') {
            Some(self.parse_expression_until(expr_terminators)?)
        } else {
            None
        };

        Ok(ParameterDecl {
            flavor,
            data_type,
            name,
            value,
        })
    }

    fn parse_port_list(&mut self) -> Result<Vec<PortDecl>, FrontendError> {
        let mut ports = Vec::new();
        if self.consume_symbol(')') {
            return Ok(ports);
        }

        loop {
            let direction = self.parse_port_direction()?;
            let data_type = self.parse_optional_data_type();
            let packed_range = self.parse_optional_packed_range()?;

            loop {
                let name = self.expect_identifier()?;
                ports.push(PortDecl {
                    direction: direction.clone(),
                    data_type: data_type.clone(),
                    packed_range: packed_range.clone(),
                    name,
                });

                if !self.consume_symbol(',') {
                    break;
                }
                if self.peek_symbol(')') || self.peek_direction_keyword() {
                    break;
                }
            }

            if self.peek_symbol(')') {
                break;
            }
        }

        self.expect_symbol(')')?;
        Ok(ports)
    }

    fn parse_item_group(
        &mut self,
        allow_generate_constructs: bool,
    ) -> Result<Vec<ModuleItem>, FrontendError> {
        if self.peek_keyword("parameter") || self.peek_keyword("localparam") {
            return self.parse_parameter_items();
        }
        if self.peek_keyword("genvar") {
            return Ok(vec![ModuleItem::GenvarDecl(self.parse_genvar_decl()?)]);
        }
        if self.peek_keyword("logic") || self.peek_keyword("wire") || self.peek_keyword("reg") {
            return Ok(vec![ModuleItem::NetDecl(self.parse_net_decl()?)]);
        }
        if self.peek_keyword("assign") {
            return Ok(vec![ModuleItem::ContinuousAssign(
                self.parse_continuous_assign()?,
            )]);
        }
        if self.peek_keyword("always_comb") || self.peek_keyword("always") {
            return Ok(vec![ModuleItem::ProceduralBlock(
                self.parse_procedural_block()?,
            )]);
        }
        if self.peek_keyword("generate") {
            return Ok(vec![ModuleItem::GenerateRegion(
                self.parse_generate_region()?,
            )]);
        }
        if allow_generate_constructs && self.peek_keyword("if") {
            return Ok(vec![ModuleItem::GenerateIf(self.parse_generate_if()?)]);
        }
        if allow_generate_constructs && self.peek_keyword("for") {
            return Ok(vec![ModuleItem::GenerateFor(self.parse_generate_for()?)]);
        }

        Err(FrontendError::new(
            format!(
                "unsupported module item starting with {}",
                self.describe_current()
            ),
            self.current().start,
        ))
    }

    fn parse_parameter_items(&mut self) -> Result<Vec<ModuleItem>, FrontendError> {
        let mut items = Vec::new();
        let mut current_flavor: Option<ParameterFlavor> = None;

        loop {
            if self.consume_keyword("parameter") {
                current_flavor = Some(ParameterFlavor::Parameter);
            } else if self.consume_keyword("localparam") {
                current_flavor = Some(ParameterFlavor::Localparam);
            }

            let Some(flavor) = current_flavor.clone() else {
                return Err(FrontendError::new(
                    "expected parameter/localparam declaration",
                    self.current().start,
                ));
            };

            let decl = self.parse_parameter_decl(flavor, &[',', ';'])?;
            items.push(ModuleItem::ParameterDecl(decl));
            if !self.consume_symbol(',') {
                break;
            }
        }

        self.expect_symbol(';')?;
        Ok(items)
    }

    fn parse_genvar_decl(&mut self) -> Result<GenvarDecl, FrontendError> {
        self.expect_keyword("genvar")?;
        let mut names = vec![self.expect_identifier()?];
        while self.consume_symbol(',') {
            names.push(self.expect_identifier()?);
        }
        self.expect_symbol(';')?;
        Ok(GenvarDecl { names })
    }

    fn parse_net_decl(&mut self) -> Result<NetDecl, FrontendError> {
        let kind = if self.consume_keyword("logic") {
            NetKind::Logic
        } else if self.consume_keyword("wire") {
            NetKind::Wire
        } else if self.consume_keyword("reg") {
            NetKind::Reg
        } else {
            return Err(FrontendError::new(
                "expected net declaration",
                self.current().start,
            ));
        };

        let packed_range = self.parse_optional_packed_range()?;
        let mut names = vec![self.expect_identifier()?];
        while self.consume_symbol(',') {
            names.push(self.expect_identifier()?);
        }
        self.expect_symbol(';')?;

        Ok(NetDecl {
            kind,
            packed_range,
            names,
        })
    }

    fn parse_continuous_assign(&mut self) -> Result<ContinuousAssign, FrontendError> {
        self.expect_keyword("assign")?;
        let target = self.expect_identifier()?;
        self.expect_symbol('=')?;
        let value = self.parse_expression_until(&[';'])?;
        self.expect_symbol(';')?;
        Ok(ContinuousAssign { target, value })
    }

    fn parse_procedural_block(&mut self) -> Result<ProceduralBlock, FrontendError> {
        let kind = if self.consume_keyword("always_comb") {
            ProceduralKind::AlwaysComb
        } else {
            self.expect_keyword("always")?;
            self.expect_symbol('@')?;
            if self.consume_symbol('(') {
                self.expect_symbol('*')?;
                self.expect_symbol(')')?;
            } else {
                self.expect_symbol('*')?;
            }
            ProceduralKind::AlwaysStar
        };

        let statement = self.parse_statement()?;
        Ok(ProceduralBlock { kind, statement })
    }

    fn parse_statement(&mut self) -> Result<Statement, FrontendError> {
        if self.consume_symbol(';') {
            return Ok(Statement::Empty);
        }
        if self.consume_keyword("begin") {
            return self.parse_statement_block();
        }
        if self.consume_keyword("if") {
            return self.parse_if_statement();
        }

        let target = self.expect_identifier()?;
        let kind = if self.consume_operator("<=") {
            AssignmentKind::NonBlocking
        } else {
            self.expect_symbol('=')?;
            AssignmentKind::Blocking
        };
        let value = self.parse_expression_until(&[';'])?;
        self.expect_symbol(';')?;
        Ok(Statement::Assignment {
            target,
            kind,
            value,
        })
    }

    fn parse_statement_block(&mut self) -> Result<Statement, FrontendError> {
        let label = self.parse_optional_label()?;
        let mut statements = Vec::new();
        while !self.peek_keyword("end") {
            statements.push(self.parse_statement()?);
        }
        self.expect_keyword("end")?;
        Ok(Statement::Block { label, statements })
    }

    fn parse_if_statement(&mut self) -> Result<Statement, FrontendError> {
        self.expect_symbol('(')?;
        let condition = self.parse_expression_until(&[')'])?;
        self.expect_symbol(')')?;
        let then_branch = self.parse_statement()?;
        let else_branch = if self.consume_keyword("else") {
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };
        Ok(Statement::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn parse_generate_region(&mut self) -> Result<GenerateRegion, FrontendError> {
        self.expect_keyword("generate")?;
        let mut items = Vec::new();
        while !self.peek_keyword("endgenerate") {
            items.extend(self.parse_item_group(true)?);
        }
        self.expect_keyword("endgenerate")?;
        Ok(GenerateRegion { items })
    }

    fn parse_generate_if(&mut self) -> Result<GenerateIf, FrontendError> {
        self.expect_keyword("if")?;
        self.expect_symbol('(')?;
        let condition = self.parse_expression_until(&[')'])?;
        self.expect_symbol(')')?;
        let (then_label, then_items) = self.parse_generate_body()?;
        let (else_label, else_items) = if self.consume_keyword("else") {
            let (label, items) = self.parse_generate_body()?;
            (label, items)
        } else {
            (None, Vec::new())
        };

        Ok(GenerateIf {
            condition,
            then_label,
            then_items,
            else_label,
            else_items,
        })
    }

    fn parse_generate_for(&mut self) -> Result<GenerateFor, FrontendError> {
        self.expect_keyword("for")?;
        self.expect_symbol('(')?;
        let declares_genvar = self.consume_keyword("genvar");
        let index_name = self.expect_identifier()?;
        self.expect_symbol('=')?;
        let init = self.parse_expression_until(&[';'])?;
        self.expect_symbol(';')?;
        let condition = self.parse_expression_until(&[';'])?;
        self.expect_symbol(';')?;
        let step_name = self.expect_identifier()?;
        if step_name != index_name {
            return Err(FrontendError::new(
                format!(
                    "generate-for step target '{}' does not match loop variable '{}'",
                    step_name, index_name
                ),
                self.current().start,
            ));
        }
        self.expect_symbol('=')?;
        let step = self.parse_expression_until(&[')'])?;
        self.expect_symbol(')')?;
        let (label, body_items) = self.parse_generate_body()?;

        Ok(GenerateFor {
            index_name,
            declares_genvar,
            init,
            condition,
            step,
            label,
            body_items,
        })
    }

    fn parse_generate_body(&mut self) -> Result<(Option<String>, Vec<ModuleItem>), FrontendError> {
        if self.consume_keyword("begin") {
            let label = self.parse_optional_label()?;
            let mut items = Vec::new();
            while !self.peek_keyword("end") {
                items.extend(self.parse_item_group(true)?);
            }
            self.expect_keyword("end")?;
            Ok((label, items))
        } else {
            Ok((None, self.parse_item_group(true)?))
        }
    }

    fn parse_optional_data_type(&mut self) -> Option<DataType> {
        let keyword = self.current_ident()?.to_string();
        if is_data_type_keyword(&keyword) {
            self.advance();
            Some(DataType { keyword })
        } else {
            None
        }
    }

    fn parse_optional_packed_range(&mut self) -> Result<Option<PackedRange>, FrontendError> {
        if !self.consume_symbol('[') {
            return Ok(None);
        }
        let msb = self.parse_expression_until(&[':'])?;
        self.expect_symbol(':')?;
        let lsb = self.parse_expression_until(&[']'])?;
        self.expect_symbol(']')?;
        Ok(Some(PackedRange { msb, lsb }))
    }

    fn parse_expression_until(&mut self, terminators: &[char]) -> Result<Expr, FrontendError> {
        if self.is_end() {
            return Err(FrontendError::new(
                "expected expression, found end of input",
                0,
            ));
        }

        let start = self.current().start;
        let mut end = start;
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut brace_depth = 0usize;
        let mut ternary_depth = 0usize;

        while !self.is_end() {
            let token = self.current();
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;

            if top_level {
                if let TokenKind::Symbol(':') = token.kind {
                    if ternary_depth > 0 {
                        ternary_depth -= 1;
                    } else if terminators.contains(&':') {
                        break;
                    }
                } else if let TokenKind::Symbol(ch) = token.kind {
                    if terminators.contains(&ch) {
                        break;
                    }
                }
            }

            match token.kind {
                TokenKind::Symbol('(') => paren_depth += 1,
                TokenKind::Symbol(')') => paren_depth = paren_depth.saturating_sub(1),
                TokenKind::Symbol('[') => bracket_depth += 1,
                TokenKind::Symbol(']') => bracket_depth = bracket_depth.saturating_sub(1),
                TokenKind::Symbol('{') => brace_depth += 1,
                TokenKind::Symbol('}') => brace_depth = brace_depth.saturating_sub(1),
                TokenKind::Symbol('?') if top_level => ternary_depth += 1,
                _ => {}
            }

            end = token.end;
            self.advance();
        }

        let text = self
            .input
            .get(start..end)
            .unwrap_or_default()
            .trim()
            .to_string();
        if text.is_empty() {
            return Err(FrontendError::new("expected expression", start));
        }

        parse_expression(&text).map_err(|err| {
            FrontendError::new(
                format!("failed to parse expression '{}': {}", text, err.message),
                start + err.position,
            )
        })
    }

    fn parse_port_direction(&mut self) -> Result<PortDirection, FrontendError> {
        if self.consume_keyword("input") {
            Ok(PortDirection::Input)
        } else if self.consume_keyword("output") {
            Ok(PortDirection::Output)
        } else if self.consume_keyword("inout") {
            Ok(PortDirection::Inout)
        } else {
            Err(FrontendError::new(
                "expected port direction",
                self.current().start,
            ))
        }
    }

    fn parse_optional_label(&mut self) -> Result<Option<String>, FrontendError> {
        if self.consume_symbol(':') {
            Ok(Some(self.expect_identifier()?))
        } else {
            Ok(None)
        }
    }

    fn current(&self) -> &Token {
        &self.tokens[self.index]
    }

    fn current_ident(&self) -> Option<&str> {
        match &self.current().kind {
            TokenKind::Ident(value) => Some(value.as_str()),
            _ => None,
        }
    }

    fn is_end(&self) -> bool {
        matches!(self.current().kind, TokenKind::End)
    }

    fn advance(&mut self) {
        if !self.is_end() {
            self.index += 1;
        }
    }

    fn peek_keyword(&self, keyword: &str) -> bool {
        matches!(self.current_ident(), Some(value) if value == keyword)
    }

    fn consume_keyword(&mut self, keyword: &str) -> bool {
        if self.peek_keyword(keyword) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_keyword(&mut self, keyword: &str) -> Result<(), FrontendError> {
        if self.consume_keyword(keyword) {
            Ok(())
        } else {
            Err(FrontendError::new(
                format!("expected keyword '{}'", keyword),
                self.current().start,
            ))
        }
    }

    fn peek_symbol(&self, symbol: char) -> bool {
        matches!(self.current().kind, TokenKind::Symbol(ch) if ch == symbol)
    }

    fn consume_symbol(&mut self, symbol: char) -> bool {
        if self.peek_symbol(symbol) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_symbol(&mut self, symbol: char) -> Result<(), FrontendError> {
        if self.consume_symbol(symbol) {
            Ok(())
        } else {
            Err(FrontendError::new(
                format!("expected symbol '{}'", symbol),
                self.current().start,
            ))
        }
    }

    fn consume_operator(&mut self, operator: &str) -> bool {
        if matches!(self.current().kind, TokenKind::Operator(value) if value == operator) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_identifier(&mut self) -> Result<String, FrontendError> {
        match &self.current().kind {
            TokenKind::Ident(value) if !is_keyword(value) => {
                let value = value.clone();
                self.advance();
                Ok(value)
            }
            _ => Err(FrontendError::new(
                "expected identifier",
                self.current().start,
            )),
        }
    }

    fn peek_direction_keyword(&self) -> bool {
        self.peek_keyword("input") || self.peek_keyword("output") || self.peek_keyword("inout")
    }

    fn describe_current(&self) -> String {
        match &self.current().kind {
            TokenKind::Ident(value) => format!("identifier '{}'", value),
            TokenKind::Number(value) => format!("number '{}'", value),
            TokenKind::Symbol(value) => format!("symbol '{}'", value),
            TokenKind::Operator(value) => format!("operator '{}'", value),
            TokenKind::End => "end of input".to_string(),
        }
    }
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    is_ident_start(ch) || ch.is_ascii_digit() || ch == '$'
}

fn is_keyword(value: &str) -> bool {
    matches!(
        value,
        "always"
            | "always_comb"
            | "assign"
            | "begin"
            | "else"
            | "end"
            | "endgenerate"
            | "endmodule"
            | "for"
            | "generate"
            | "genvar"
            | "if"
            | "inout"
            | "input"
            | "localparam"
            | "logic"
            | "module"
            | "output"
            | "parameter"
            | "reg"
            | "wire"
    )
}

fn is_data_type_keyword(value: &str) -> bool {
    matches!(value, "bit" | "int" | "integer" | "logic" | "reg" | "wire")
}

#[cfg(test)]
mod tests {
    use super::{
        GenerateFor, GenerateIf, ModuleItem, ProceduralKind, Statement, parse_design, parse_module,
    };
    use std::collections::HashMap;

    #[test]
    fn parses_module_shape_and_evaluates_constants() {
        let module = parse_module(
            r#"
            module arithmetic #(
                parameter WIDTH = 8,
                parameter DEPTH = WIDTH + 4
            ) (
                input logic [WIDTH-1:0] a,
                input logic [WIDTH-1:0] b,
                output logic [DEPTH-1:0] y
            );
            parameter EXTRA = DEPTH + 1;
            localparam TOTAL = WIDTH * 2;
            logic [WIDTH-1:0] data, scratch;
            assign y = DEPTH > WIDTH ? DEPTH : WIDTH;
            always_comb begin : comb_blk
                data = WIDTH + TOTAL;
                if (EXTRA > 0)
                    scratch = TOTAL;
                else
                    scratch = 0;
            end
            generate
                if (WIDTH > 4) begin : has_extra
                    logic [TOTAL-1:0] extra;
                end else begin : no_extra
                    logic dummy;
                end
                for (genvar i = 0; i < 3; i = i + 1) begin : lanes
                    logic tap;
                end
            endgenerate
            endmodule
            "#,
        )
        .expect("module should parse");

        assert_eq!(module.name, "arithmetic");
        assert_eq!(module.parameters.len(), 2);
        assert_eq!(module.ports.len(), 3);

        let env = module
            .evaluate_constant_env(&HashMap::new())
            .expect("constant env should resolve");
        assert_eq!(env.get("WIDTH"), Some(&8));
        assert_eq!(env.get("DEPTH"), Some(&12));
        assert_eq!(env.get("EXTRA"), Some(&13));
        assert_eq!(env.get("TOTAL"), Some(&16));
        assert_eq!(module.ports[2].width(&env).unwrap(), Some(12));

        let generate_region = module
            .items
            .iter()
            .find_map(|item| match item {
                ModuleItem::GenerateRegion(region) => Some(region),
                _ => None,
            })
            .expect("generate region should exist");

        match &generate_region.items[0] {
            ModuleItem::GenerateIf(generate_if) => {
                assert!(generate_if.is_enabled(&env).unwrap());
                assert_eq!(generate_if.then_label.as_deref(), Some("has_extra"));
                assert_eq!(generate_if.else_label.as_deref(), Some("no_extra"));
            }
            other => panic!("expected generate-if, got {other:?}"),
        }

        match &generate_region.items[1] {
            ModuleItem::GenerateFor(generate_for) => {
                assert_eq!(generate_for.label.as_deref(), Some("lanes"));
                assert_eq!(
                    generate_for.iteration_values(&env, 8).unwrap(),
                    vec![0, 1, 2]
                );
            }
            other => panic!("expected generate-for, got {other:?}"),
        }
    }

    #[test]
    fn parses_always_star_blocks() {
        let module = parse_module(
            r#"
            module star (
                input logic a,
                output logic y
            );
            always @(*) begin
                if (a)
                    y = 1;
                else
                    y = 0;
            end
            endmodule
            "#,
        )
        .expect("module should parse");

        let block = module
            .items
            .iter()
            .find_map(|item| match item {
                ModuleItem::ProceduralBlock(block) => Some(block),
                _ => None,
            })
            .expect("procedural block should exist");

        assert_eq!(block.kind, ProceduralKind::AlwaysStar);
        match &block.statement {
            Statement::Block { statements, .. } => assert_eq!(statements.len(), 1),
            other => panic!("expected block statement, got {other:?}"),
        }
    }

    #[test]
    fn parse_design_supports_multiple_modules() {
        let design = parse_design(
            r#"
            module first;
            endmodule

            module second;
            endmodule
            "#,
        )
        .expect("design should parse");

        assert_eq!(design.modules.len(), 2);
        assert_eq!(design.modules[0].name, "first");
        assert_eq!(design.modules[1].name, "second");
    }

    #[test]
    fn generate_if_condition_evaluates_from_symbols() {
        let generate_if = GenerateIf {
            condition: rtl_const_expr::parse_expression("WIDTH > 1").unwrap(),
            then_label: None,
            then_items: Vec::new(),
            else_label: None,
            else_items: Vec::new(),
        };

        let symbols = HashMap::from([("WIDTH".to_string(), 4)]);
        assert!(generate_if.is_enabled(&symbols).unwrap());
    }

    #[test]
    fn generate_for_unrolls_with_bounded_iteration() {
        let generate_for = GenerateFor {
            index_name: "i".to_string(),
            declares_genvar: true,
            init: rtl_const_expr::parse_expression("0").unwrap(),
            condition: rtl_const_expr::parse_expression("i < LIMIT").unwrap(),
            step: rtl_const_expr::parse_expression("i + 2").unwrap(),
            label: Some("step2".to_string()),
            body_items: Vec::new(),
        };

        let symbols = HashMap::from([("LIMIT".to_string(), 5)]);
        assert_eq!(
            generate_for.iteration_values(&symbols, 8).unwrap(),
            vec![0, 2, 4]
        );
    }
}
