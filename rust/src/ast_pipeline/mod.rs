use anyhow::{Result, anyhow};
use serde;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum TraceVerbosity {
    None = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Debug = 4,
}

impl TraceVerbosity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Debug => "debug",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "none" | "off" | "0" => Some(Self::None),
            "low" | "1" => Some(Self::Low),
            "medium" | "med" | "2" => Some(Self::Medium),
            "high" | "3" => Some(Self::High),
            "debug" | "trace" | "4" => Some(Self::Debug),
            _ => None,
        }
    }

    pub fn from_flags(debug: bool, trace: bool) -> Self {
        if trace {
            Self::Debug
        } else if debug {
            Self::High
        } else {
            Self::None
        }
    }

    pub fn allows(self, level: TraceLevel) -> bool {
        self as u8 >= level.min_verbosity() as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TraceLevel {
    Low = 1,
    Medium = 2,
    High = 3,
    Debug = 4,
}

impl TraceLevel {
    fn as_str(self) -> &'static str {
        match self {
            Self::Low => "LOW",
            Self::Medium => "MED",
            Self::High => "HIGH",
            Self::Debug => "DBG",
        }
    }

    fn emoji(self) -> &'static str {
        match self {
            Self::Low => "🧭",
            Self::Medium => "🧩",
            Self::High => "🔎",
            Self::Debug => "🧠",
        }
    }

    fn min_verbosity(self) -> TraceVerbosity {
        match self {
            Self::Low => TraceVerbosity::Low,
            Self::Medium => TraceVerbosity::Medium,
            Self::High => TraceVerbosity::High,
            Self::Debug => TraceVerbosity::Debug,
        }
    }
}

static GLOBAL_TRACE_VERBOSITY: AtomicU8 = AtomicU8::new(TraceVerbosity::None as u8);
static TRACE_OUTPUT_SINK: OnceLock<Mutex<Option<File>>> = OnceLock::new();
static TRACE_FUNCTION_CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

pub fn parse_trace_verbosity(raw: &str) -> Option<TraceVerbosity> {
    TraceVerbosity::parse(raw)
}

pub fn trace_verbosity_from_env() -> Option<TraceVerbosity> {
    std::env::var("PGEN_TRACE_VERBOSITY")
        .ok()
        .or_else(|| std::env::var("PGEN_VERBOSITY").ok())
        .as_deref()
        .and_then(TraceVerbosity::parse)
}

fn trace_sink() -> &'static Mutex<Option<File>> {
    TRACE_OUTPUT_SINK.get_or_init(|| Mutex::new(None))
}

fn trace_function_cache() -> &'static Mutex<HashMap<String, String>> {
    TRACE_FUNCTION_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn is_internal_trace_symbol(symbol: &str) -> bool {
    symbol.contains("ast_pipeline::trace_log")
        || symbol.contains("ast_pipeline::resolve_trace_function_name")
        || symbol.contains("ast_pipeline::trace_function_name_from_backtrace")
        || symbol.contains("ast_pipeline::is_internal_trace_symbol")
        || symbol.contains("pgen_trace")
        || symbol.contains("VerbosityLogger::emit")
        || symbol.contains("std::backtrace_rs::backtrace")
        || symbol.contains("std::backtrace::Backtrace::")
        || symbol.starts_with("std::")
        || symbol.starts_with("core::")
        || symbol.starts_with("alloc::")
}

fn normalize_trace_symbol(symbol: &str) -> String {
    if let Some((base, hash)) = symbol.rsplit_once("::h") {
        if hash.len() >= 8 && hash.chars().all(|ch| ch.is_ascii_hexdigit()) {
            return base.to_string();
        }
    }
    symbol.to_string()
}

fn trace_function_name_from_backtrace(module_path: &str) -> String {
    let backtrace = format!("{:?}", std::backtrace::Backtrace::force_capture());
    let marker = "fn: \"";
    let mut cursor = backtrace.as_str();

    while let Some(start_idx) = cursor.find(marker) {
        let remaining = &cursor[start_idx + marker.len()..];
        let Some(end_idx) = remaining.find('"') else {
            break;
        };
        let symbol = &remaining[..end_idx];
        if !is_internal_trace_symbol(symbol) {
            return normalize_trace_symbol(symbol);
        }
        cursor = &remaining[end_idx + 1..];
    }

    module_path.to_string()
}

fn resolve_trace_function_name(file: &str, line: u32, module_path: &str) -> String {
    let key = format!("{}:{}:{}", file, line, module_path);

    if let Ok(cache) = trace_function_cache().lock() {
        if let Some(cached) = cache.get(&key) {
            return cached.clone();
        }
    }

    let resolved = trace_function_name_from_backtrace(module_path);
    if let Ok(mut cache) = trace_function_cache().lock() {
        cache.insert(key, resolved.clone());
    }
    resolved
}

pub fn configure_trace_output(path: Option<&str>) -> Result<()> {
    let mut guard = trace_sink()
        .lock()
        .map_err(|_| anyhow!("trace output sink lock poisoned"))?;

    if let Some(path) = path.map(str::trim).filter(|path| !path.is_empty()) {
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)
            .map_err(|err| anyhow!("failed to open trace output file '{}': {}", path, err))?;
        *guard = Some(file);
    } else {
        *guard = None;
    }

    Ok(())
}

pub fn resolve_trace_verbosity(
    cli_value: Option<&str>,
    debug_flag: bool,
    trace_flag: bool,
) -> Result<TraceVerbosity> {
    if let Some(raw) = cli_value {
        return TraceVerbosity::parse(raw).ok_or_else(|| {
            anyhow!(
                "Invalid trace verbosity '{}'. Expected one of: none, low, medium, high, debug",
                raw
            )
        });
    }
    if let Some(from_env) = trace_verbosity_from_env() {
        return Ok(from_env);
    }
    Ok(TraceVerbosity::from_flags(debug_flag, trace_flag))
}

pub fn set_global_trace_verbosity(verbosity: TraceVerbosity) {
    GLOBAL_TRACE_VERBOSITY.store(verbosity as u8, Ordering::Relaxed);
}

pub fn global_trace_verbosity() -> TraceVerbosity {
    match GLOBAL_TRACE_VERBOSITY.load(Ordering::Relaxed) {
        0 => TraceVerbosity::None,
        1 => TraceVerbosity::Low,
        2 => TraceVerbosity::Medium,
        3 => TraceVerbosity::High,
        _ => TraceVerbosity::Debug,
    }
}

pub fn trace_enabled(level: TraceLevel) -> bool {
    global_trace_verbosity().allows(level)
}

pub fn trace_log(
    level: TraceLevel,
    file: &str,
    line: u32,
    module_path: &str,
    args: fmt::Arguments<'_>,
) {
    if !trace_enabled(level) {
        return;
    }

    let function_name = resolve_trace_function_name(file, line, module_path);
    let rendered = format!("{}", args);
    let output = if rendered.is_empty() {
        format!(
            "[PGEN][{}] {} [{}:{}] [{}]",
            level.as_str(),
            level.emoji(),
            file,
            line,
            function_name
        )
    } else {
        format!(
            "[PGEN][{}] {} [{}:{}] [{}] {}",
            level.as_str(),
            level.emoji(),
            file,
            line,
            function_name,
            rendered
        )
    };

    if let Ok(mut guard) = trace_sink().lock() {
        if let Some(file) = guard.as_mut() {
            let _ = writeln!(file, "{}", output);
            let _ = file.flush();
            return;
        }
    }
    println!("{}", output);
}

#[macro_export]
macro_rules! pgen_trace {
    ($level:expr) => {
        $crate::ast_pipeline::trace_log(
            $level,
            file!(),
            line!(),
            module_path!(),
            format_args!(""),
        )
    };
    ($level:expr, $($arg:tt)*) => {
        $crate::ast_pipeline::trace_log(
            $level,
            file!(),
            line!(),
            module_path!(),
            format_args!($($arg)*),
        )
    };
}

#[macro_export]
macro_rules! pgen_trace_low {
    () => {
        $crate::pgen_trace!($crate::ast_pipeline::TraceLevel::Low)
    };
    ($($arg:tt)*) => {
        $crate::pgen_trace!($crate::ast_pipeline::TraceLevel::Low, $($arg)*)
    };
}

#[macro_export]
macro_rules! pgen_trace_medium {
    () => {
        $crate::pgen_trace!($crate::ast_pipeline::TraceLevel::Medium)
    };
    ($($arg:tt)*) => {
        $crate::pgen_trace!($crate::ast_pipeline::TraceLevel::Medium, $($arg)*)
    };
}

#[macro_export]
macro_rules! pgen_trace_high {
    () => {
        $crate::pgen_trace!($crate::ast_pipeline::TraceLevel::High)
    };
    ($($arg:tt)*) => {
        $crate::pgen_trace!($crate::ast_pipeline::TraceLevel::High, $($arg)*)
    };
}

#[macro_export]
macro_rules! pgen_trace_debug {
    () => {
        $crate::pgen_trace!($crate::ast_pipeline::TraceLevel::Debug)
    };
    ($($arg:tt)*) => {
        $crate::pgen_trace!($crate::ast_pipeline::TraceLevel::Debug, $($arg)*)
    };
}

macro_rules! eprintln {
    ($($arg:tt)*) => {
        crate::pgen_trace_debug!($($arg)*)
    };
}

// Shared Logger trait that both binaries can access
pub trait Logger: std::fmt::Debug {
    fn is_enabled(&self) -> bool;
    fn log_info(&self, file: &str, line: u32, message: &str);
    fn log_debug(&self, file: &str, line: u32, message: &str);
    fn log_success(&self, file: &str, line: u32, message: &str);
    fn log_warning(&self, file: &str, line: u32, message: &str);
    fn log_error(&self, file: &str, line: u32, message: &str);

    // Clone method for logger instances
    fn clone_box(&self) -> Box<dyn Logger>;
}

#[derive(Debug, Clone)]
pub struct VerbosityLogger {
    component: String,
    verbosity: TraceVerbosity,
}

impl VerbosityLogger {
    pub fn new(component: impl Into<String>, verbosity: TraceVerbosity) -> Self {
        Self {
            component: component.into(),
            verbosity,
        }
    }

    fn emit(&self, level: TraceLevel, file: &str, line: u32, message: &str) {
        if !self.verbosity.allows(level) {
            return;
        }

        let title = format!(
            "[TRACE][{}][{}] {}",
            self.component,
            level.as_str(),
            level.emoji()
        );
        trace_log(
            level,
            file,
            line,
            self.component.as_str(),
            format_args!("{}", title),
        );
        trace_log(
            level,
            file,
            line,
            self.component.as_str(),
            format_args!("  📍 {}:{}", file, line),
        );
        trace_log(
            level,
            file,
            line,
            self.component.as_str(),
            format_args!("  {}", message),
        );
        trace_log(level, file, line, self.component.as_str(), format_args!(""));
    }
}

impl Logger for VerbosityLogger {
    fn is_enabled(&self) -> bool {
        self.verbosity != TraceVerbosity::None
    }

    fn log_info(&self, file: &str, line: u32, message: &str) {
        self.emit(TraceLevel::High, file, line, message);
    }

    fn log_debug(&self, file: &str, line: u32, message: &str) {
        self.emit(TraceLevel::Debug, file, line, message);
    }

    fn log_success(&self, file: &str, line: u32, message: &str) {
        self.emit(TraceLevel::Medium, file, line, message);
    }

    fn log_warning(&self, file: &str, line: u32, message: &str) {
        self.emit(TraceLevel::Low, file, line, message);
    }

    fn log_error(&self, file: &str, line: u32, message: &str) {
        self.emit(TraceLevel::Low, file, line, message);
    }

    fn clone_box(&self) -> Box<dyn Logger> {
        Box::new(self.clone())
    }
}

pub fn runtime_logger(component: impl Into<String>) -> VerbosityLogger {
    VerbosityLogger::new(component, global_trace_verbosity())
}

pub fn runtime_logger_box(component: impl Into<String>) -> Box<dyn Logger> {
    let verbosity = global_trace_verbosity();
    if verbosity == TraceVerbosity::None {
        Box::new(NoOpLogger)
    } else {
        Box::new(VerbosityLogger::new(component, verbosity))
    }
}

// No-op logger implementation
#[derive(Debug, Clone)]
pub struct NoOpLogger;

impl Logger for NoOpLogger {
    fn is_enabled(&self) -> bool {
        false
    }
    fn log_info(&self, _file: &str, _line: u32, _message: &str) {}
    fn log_debug(&self, _file: &str, _line: u32, _message: &str) {}
    fn log_success(&self, _file: &str, _line: u32, _message: &str) {}
    fn log_warning(&self, _file: &str, _line: u32, _message: &str) {}
    fn log_error(&self, _file: &str, _line: u32, _message: &str) {}

    fn clone_box(&self) -> Box<dyn Logger> {
        Box::new(self.clone())
    }
}

#[cfg(feature = "generated_parsers")]
use crate::generated_parsers::return_annotation::Return_annotationParser;
#[cfg(feature = "generated_parsers")]
use crate::generated_parsers::semantic_annotation::Semantic_annotationParser;

// Shared parser types used by generated parsers
/// Parse result type
pub type ParseResult<T> = Result<T, ParseError>;

/// Parse errors
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedEof {
        position: usize,
    },
    UnexpectedToken {
        expected: &'static str,
        found: char,
        position: usize,
    },
    InvalidSyntax {
        message: &'static str,
        position: usize,
    },
    Backtrack {
        position: usize,
    },
    RecursionDepthExceeded {
        position: usize,
        depth: usize,
    },
    ContextualError {
        message: String,
        position: usize,
        rule_stack: Vec<&'static str>,
        input_context: String,
    },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedEof { position } => {
                write!(f, "Unexpected EOF at position {}", position)
            }
            ParseError::UnexpectedToken {
                expected,
                found,
                position,
            } => {
                write!(
                    f,
                    "Expected '{}', found '{}' at position {}",
                    expected, found, position
                )
            }
            ParseError::InvalidSyntax { message, position } => {
                write!(f, "{} at position {}", message, position)
            }
            ParseError::Backtrack { position } => {
                write!(f, "Backtrack at position {}", position)
            }
            ParseError::RecursionDepthExceeded { position, depth } => {
                write!(
                    f,
                    "Recursion depth exceeded ({} levels) at position {}",
                    depth, position
                )
            }
            ParseError::ContextualError {
                message,
                position,
                rule_stack,
                input_context,
            } => {
                writeln!(f, "Parse Error: {}\n", message)?;
                writeln!(f, "Position: {}\n", position)?;
                writeln!(f, "Context: {}\n", input_context)?;
                writeln!(f, "Rule Stack:")?;
                for (i, rule) in rule_stack.iter().enumerate() {
                    writeln!(f, "  {}: {}", i, rule)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// Parse content types
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum ParseContent<'input> {
    Terminal(&'input str),
    TransformedTerminal(String),
    /// Typed structured carrier for return-annotation object/array literals and
    /// property/array access results. Avoids the runtime serialise/parse/serialise
    /// roundtrip the older `TransformedTerminal(stringified-json)` path used.
    Json(serde_json::Value),
    Sequence(Vec<ParseNode<'input>>),
    Alternative(Box<ParseNode<'input>>),
    Quantified(Vec<ParseNode<'input>>, &'static str),
}

impl<'input> ParseContent<'input> {
    /// Convert any `ParseContent` shape to a `serde_json::Value` without going
    /// through string-encoded intermediates. Used by return-annotation object/array
    /// transforms and property/array access at runtime.
    pub fn to_json_value(&self) -> serde_json::Value {
        match self {
            ParseContent::Terminal(s) => serde_json::Value::String((*s).to_string()),
            ParseContent::TransformedTerminal(s) => {
                // Best-effort: parse if it already encodes a JSON value, otherwise
                // wrap the raw string. Existing scalar `@transform` coercion paths
                // produce TransformedTerminal(numeric-or-bool-text) and rely on
                // wrap-as-string when JSON parsing fails.
                serde_json::from_str::<serde_json::Value>(s)
                    .unwrap_or_else(|_| serde_json::Value::String(s.clone()))
            }
            ParseContent::Json(value) => value.clone(),
            ParseContent::Alternative(node) => node.content.to_json_value(),
            ParseContent::Sequence(nodes) | ParseContent::Quantified(nodes, _) => {
                serde_json::Value::Array(
                    nodes.iter().map(|n| n.content.to_json_value()).collect(),
                )
            }
        }
    }
}

/// Parse node
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ParseNode<'input> {
    pub rule_name: &'static str,
    pub content: ParseContent<'input>,
    pub span: std::ops::Range<usize>,
}

/// Memoization entry
#[derive(Debug, Clone)]
pub struct MemoEntry<'input> {
    pub result: Option<ParseNode<'input>>,
    pub raw_semantic_content: Option<ParseContent<'input>>,
    pub end_pos: usize,
}

/// Rule ID type for memoization
pub type RuleId = u16;

/// Recursion cycle types
#[derive(Debug, Clone, PartialEq)]
pub enum CycleType {
    None,
    Infinite,
    LeftRecursive,
    MutualRecursive { depth: usize, rules: Vec<&'static str> },
}

/// Recursion guard
#[derive(Debug, Clone)]
pub struct RecursionGuard {
    pub parse_stack: Vec<(&'static str, usize)>,
    pub max_depth: usize,
    pub cycle_cache: HashMap<(String, usize), CycleType>,
}

impl RecursionGuard {
    pub fn new(max_depth: usize) -> Self {
        Self {
            parse_stack: Vec::new(),
            max_depth,
            cycle_cache: HashMap::new(),
        }
    }

    pub fn check_cycle(&mut self, rule_name: &'static str, position: usize) -> CycleType {
        for (r, p) in self.parse_stack.iter() {
            if *r == rule_name && *p == position {
                return CycleType::Infinite;
            }
            if *r == rule_name && *p > position {
                return CycleType::LeftRecursive;
            }
        }
        if self.parse_stack.len() >= self.max_depth {
            let rules: Vec<&'static str> = self.parse_stack.iter().map(|(r, _)| *r).collect();
            return CycleType::MutualRecursive {
                depth: self.parse_stack.len(),
                rules,
            };
        }
        CycleType::None
    }

    pub fn enter(&mut self, rule_name: &'static str, position: usize) {
        self.parse_stack.push((rule_name, position));
    }

    pub fn exit(&mut self) {
        self.parse_stack.pop();
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum ASTValue {
    Token(Vec<TokenValue>),
    Node(Box<ASTNode>),
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum TokenValue {
    String(String),
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum ASTNode {
    Or {
        alternatives: Vec<ASTNode>,
    },
    Sequence {
        elements: Vec<ASTNode>,
    },
    Atom {
        value: ASTValue,
    },
    Quantified {
        element: Box<ASTNode>,
        quantifier: String,
    },
    Lookahead {
        element: Box<ASTNode>,
        positive: bool,
    },
}

/// SV-EXH-PROOF.3.3.4.b.3 (Layer 0, PGEN-SV-EXH-PROOF-0029, 2026-05-21):
/// Map a quantifier string carried in `ASTNode::Quantified::quantifier` to
/// its `(min, max)` bounds:
///
/// * `"?"`        → `(0, Some(1))`
/// * `"*"`        → `(0, None)`
/// * `"+"`        → `(1, None)`
/// * `"{N}"`      → `(N, Some(N))`         (exact count)
/// * `"{N,M}"`    → `(N, Some(M))`         (range, requires `M >= N`)
/// * `"{N,}"`     → `(N, None)`            (at least N)
/// * `"{,M}"`     → `(0, Some(M))`         (at most M)
///
/// Returns `None` for any other input (invalid quantifier string).
///
/// This is the canonical surface-form → bounds mapping used by both the
/// `ast_based_generator` and `ast_code_generator` quantifier codegen, so a
/// single helper carries every repetition operator the engine supports.
pub fn parse_quantifier_bounds(quantifier: &str) -> Option<(usize, Option<usize>)> {
    let q = quantifier.trim();
    match q {
        "?" => Some((0, Some(1))),
        "*" => Some((0, None)),
        "+" => Some((1, None)),
        _ if q.starts_with('{') && q.ends_with('}') && q.len() >= 3 => {
            let inner = &q[1..q.len() - 1];
            if let Some(comma_pos) = inner.find(',') {
                let min_str = inner[..comma_pos].trim();
                let max_str = inner[comma_pos + 1..].trim();
                let min: usize = if min_str.is_empty() {
                    0
                } else {
                    min_str.parse().ok()?
                };
                let max: Option<usize> = if max_str.is_empty() {
                    None
                } else {
                    Some(max_str.parse().ok()?)
                };
                if let Some(m) = max {
                    if m < min {
                        return None;
                    }
                }
                Some((min, max))
            } else {
                // "{N}" — exact count
                let n: usize = inner.trim().parse().ok()?;
                Some((n, Some(n)))
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod parse_quantifier_bounds_tests {
    use super::parse_quantifier_bounds;

    #[test]
    fn simple_quantifiers() {
        assert_eq!(parse_quantifier_bounds("?"), Some((0, Some(1))));
        assert_eq!(parse_quantifier_bounds("*"), Some((0, None)));
        assert_eq!(parse_quantifier_bounds("+"), Some((1, None)));
    }

    #[test]
    fn bounded_quantifiers() {
        assert_eq!(parse_quantifier_bounds("{3}"), Some((3, Some(3))));
        assert_eq!(parse_quantifier_bounds("{2,5}"), Some((2, Some(5))));
        assert_eq!(parse_quantifier_bounds("{2,}"), Some((2, None)));
        assert_eq!(parse_quantifier_bounds("{,5}"), Some((0, Some(5))));
        assert_eq!(parse_quantifier_bounds("{0,0}"), Some((0, Some(0))));
    }

    #[test]
    fn whitespace_tolerated() {
        assert_eq!(parse_quantifier_bounds(" * "), Some((0, None)));
        assert_eq!(parse_quantifier_bounds("{ 2 , 5 }"), Some((2, Some(5))));
    }

    #[test]
    fn invalid_quantifiers_return_none() {
        assert_eq!(parse_quantifier_bounds(""), None);
        assert_eq!(parse_quantifier_bounds("foo"), None);
        assert_eq!(parse_quantifier_bounds("{}"), None);
        assert_eq!(parse_quantifier_bounds("{a}"), None);
        // M < N is invalid
        assert_eq!(parse_quantifier_bounds("{5,2}"), None);
        // Negative numbers reject via usize parse
        assert_eq!(parse_quantifier_bounds("{-1}"), None);
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct BranchAnnotation {
    pub annotation_type: String,
    pub annotation_content: String,
    pub parsed_ast: Option<UnifiedReturnAST>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(untagged)]
pub enum SemanticAnnotation {
    Legacy(UnifiedSemanticAST),
    Named {
        name: String,
        ast: UnifiedSemanticAST,
    },
}

impl SemanticAnnotation {
    pub fn ast(&self) -> &UnifiedSemanticAST {
        match self {
            SemanticAnnotation::Legacy(ast) => ast,
            SemanticAnnotation::Named { ast, .. } => ast,
        }
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            SemanticAnnotation::Legacy(_) => None,
            SemanticAnnotation::Named { name, .. } => Some(name.as_str()),
        }
    }
}

impl From<UnifiedSemanticAST> for SemanticAnnotation {
    fn from(value: UnifiedSemanticAST) -> Self {
        SemanticAnnotation::Legacy(value)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct MidSequenceSemanticAnnotation {
    pub syntax_position: usize,
    #[serde(default)]
    pub group_depth: usize,
    pub annotation: SemanticAnnotation,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Default)]
pub struct Annotations {
    #[serde(default)]
    pub branch_return_annotations: std::collections::HashMap<String, Vec<Option<BranchAnnotation>>>,
    #[serde(default)]
    pub branch_semantic_annotations:
        std::collections::HashMap<String, Vec<Vec<SemanticAnnotation>>>,
    #[serde(default)]
    pub branch_mid_sequence_semantic_annotations:
        std::collections::HashMap<String, Vec<Vec<MidSequenceSemanticAnnotation>>>,
    #[serde(default)]
    pub semantic_annotations: std::collections::HashMap<String, Vec<SemanticAnnotation>>,
    /// Pre-LR-elim snapshot of `branch_return_annotations`. Populated by
    /// the LR-elim pass before it rewrites annotations into the
    /// `_pgen_lr_chain` shape (Strategy 3a). The inventory builder uses
    /// this snapshot when present so the emitted artifact reflects the
    /// grammar-author-written annotations rather than the post-LR-elim
    /// migration. `None` when no LR-elim transformation has run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pre_lr_elim_branch_return_annotations:
        Option<std::collections::HashMap<String, Vec<Option<BranchAnnotation>>>>,
}

/// Normalize a return-annotation payload string for stable comparison. Trim
/// outer whitespace; collapse runs of whitespace inside the payload to a
/// single space; preserve characters inside string literals (quoted with
/// `"` or `'`) verbatim. Same algorithm used by the AST-shape contract gate
/// so the emitted inventory artifact and the gate's tracked manifest agree
/// byte-for-byte after normalization.
pub fn normalize_return_annotation_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_ws = false;
    let mut in_str = false;
    let mut quote: Option<char> = None;
    for ch in s.trim().chars() {
        if in_str {
            out.push(ch);
            if Some(ch) == quote {
                in_str = false;
                quote = None;
            }
            prev_ws = false;
        } else if ch == '"' || ch == '\'' {
            in_str = true;
            quote = Some(ch);
            out.push(ch);
            prev_ws = false;
        } else if ch.is_whitespace() {
            if !prev_ws {
                out.push(' ');
                prev_ws = true;
            }
        } else {
            out.push(ch);
            prev_ws = false;
        }
    }
    out.trim_end().to_string()
}

/// One entry in the emitted return-annotation inventory artifact.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmittedReturnAnnotationEntry {
    pub rule: String,
    pub branch_index: usize,
    pub annotation_type: String,
    pub raw_text: String,
    pub normalized_text: String,
}

/// Top-level shape of `<grammar>_return_annotations.json`. This is the
/// pipeline-emitted inventory the AST-shape contract gate compares against.
/// Single source of truth: the pipeline's own annotation extraction step
/// produces this artifact, the gate reads it directly, no re-derivation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EmittedReturnAnnotationInventory {
    pub version: u32,
    pub grammar: String,
    pub annotation_count: usize,
    pub annotations: Vec<EmittedReturnAnnotationEntry>,
}

impl EmittedReturnAnnotationInventory {
    /// Build the inventory from the in-memory `Annotations` struct that the
    /// pipeline carries. Annotations are emitted sorted by (rule,
    /// branch_index) for stable diffing across regenerations.
    pub fn from_annotations(grammar: &str, annotations: Option<&Annotations>) -> Self {
        let mut entries: Vec<EmittedReturnAnnotationEntry> = Vec::new();
        if let Some(annotations) = annotations {
            // Prefer the pre-LR-elim snapshot when present so the emitted
            // inventory artifact reflects the grammar-author-written
            // annotations rather than any pipeline-internal migrations
            // (Strategy 3a moves per-branch annotations from base rules to
            // their `*_lr_base` helpers). When no snapshot exists (e.g. no
            // LR-elim ran), the live `branch_return_annotations` is the
            // authoritative source.
            let source = annotations
                .pre_lr_elim_branch_return_annotations
                .as_ref()
                .unwrap_or(&annotations.branch_return_annotations);
            for (rule, branches) in source {
                for (branch_index, opt_annotation) in branches.iter().enumerate() {
                    if let Some(annotation) = opt_annotation {
                        // LR-elim's Strategy 3a synthetic entries are book-keeping
                        // for the runtime fold; they are not grammar-author-
                        // written and must not surface in the public inventory
                        // contract. Skip them so the inventory continues to
                        // reflect only declared annotations. (The pre-LR-elim
                        // snapshot won't contain these by construction, but
                        // the fallback path may.)
                        if annotation.annotation_type == "_pgen_lr_chain_synthetic" {
                            continue;
                        }
                        entries.push(EmittedReturnAnnotationEntry {
                            rule: rule.clone(),
                            branch_index,
                            annotation_type: annotation.annotation_type.clone(),
                            raw_text: annotation.annotation_content.clone(),
                            normalized_text: normalize_return_annotation_text(
                                &annotation.annotation_content,
                            ),
                        });
                    }
                }
            }
        }
        entries.sort_by(|a, b| {
            a.rule
                .cmp(&b.rule)
                .then_with(|| a.branch_index.cmp(&b.branch_index))
        });
        Self {
            version: 1,
            grammar: grammar.to_string(),
            annotation_count: entries.len(),
            annotations: entries,
        }
    }

    /// Write the inventory artifact to `path`. Creates the parent directory
    /// if needed. Returns the same path so callers can log/print it.
    pub fn write_to_file<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> std::io::Result<std::path::PathBuf> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let pretty = serde_json::to_string_pretty(self).map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("inventory serialise failed: {}", err),
            )
        })?;
        std::fs::write(&path, pretty + "\n")?;
        Ok(path.as_ref().to_path_buf())
    }
}

/// Default sibling path for the emitted inventory: same directory as the
/// parser output, with `<grammar>_return_annotations.json` filename.
pub fn default_return_annotation_inventory_path<P: AsRef<std::path::Path>>(
    grammar: &str,
    parser_output_path: P,
) -> std::path::PathBuf {
    let parser_path = parser_output_path.as_ref();
    let parent = parser_path.parent().unwrap_or_else(|| std::path::Path::new("."));
    parent.join(format!("{}_return_annotations.json", grammar))
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TransformMetadata {
    pub format: String,
    pub source_format: String,
    pub transformed_at: String,
    pub transformer: String,
    pub pipeline_stage: String,
    pub annotations: Option<Annotations>,
    #[serde(default)]
    pub stats: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TransformedASTJson {
    pub grammar_name: String,
    pub grammar_tree: std::collections::HashMap<String, ASTNode>,
    pub rule_order: Vec<String>,
    pub metadata: TransformMetadata,
}

// Type aliases for compatibility
// pub type ParseNode<'input> = ASTNode;  // Removed - now using full ParseNode struct

pub struct PipelineConfig {
    pub debug: bool,
    pub trace: bool,
    pub trace_verbosity: TraceVerbosity,
    pub bootstrap_mode: bool,
    pub preserve_annotations: bool,
    pub validate_input: bool,
    pub validate_output: bool,
    pub max_recursion_depth: usize,
    pub eliminate_left_recursion: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        PipelineConfig {
            debug: false,
            trace: false,
            trace_verbosity: trace_verbosity_from_env().unwrap_or(TraceVerbosity::None),
            bootstrap_mode: false,
            preserve_annotations: true,
            validate_input: true,
            validate_output: true,
            max_recursion_depth: 100,
            eliminate_left_recursion: true,
        }
    }
}

pub struct RustASTPipeline {
    config: PipelineConfig,
}

#[derive(Debug, Clone)]
struct LeftRecursiveChainPlan {
    base_rule: String,
    helper_base_rule: String,
    /// Synthetic helper rule that owns the suffix Or, allocated on demand
    /// when the LR-elim pass also rewrites annotations into a
    /// `_pgen_lr_chain` shape (Strategy 3a). Empty when annotations are
    /// not in scope (e.g. tools that disable annotation preservation).
    helper_suffix_rule: String,
    /// Surviving (non-LR) alternatives from the base rule's original Or.
    /// Each entry is `(original_or_branch_index, alternative_node)` so
    /// that per-branch annotations attached to the original base rule's
    /// Or can migrate to `helper_base_rule` in their proper positions.
    base_alternatives: Vec<(usize, ASTNode)>,
    /// LR alternatives from the base rule's original Or. Each entry is
    /// `(original_or_branch_index, wrapper_rule_name, wrapper_suffix_node)`.
    /// The `wrapper_suffix_node` is the wrapper rule's body with the
    /// leading LR self-reference stripped.
    wrapper_rules: Vec<(usize, String, ASTNode)>,
    suffix_alternative: ASTNode,
}

#[derive(Debug, Clone)]
enum RawRuleElement {
    Atom(ASTNode),
    OrOperator,
    GroupOpen,
    GroupClose,
    Quantifier(String),
    Lookahead(bool),
}

#[derive(Debug, Clone)]
struct ParsedRuleContent {
    ast_node: ASTNode,
    branch_return_annotations: Vec<Option<BranchAnnotation>>,
    branch_semantic_annotations: Vec<Vec<SemanticAnnotation>>,
    branch_mid_sequence_semantic_annotations: Vec<Vec<MidSequenceSemanticAnnotation>>,
    semantic_annotations: Vec<SemanticAnnotation>,
}

#[derive(Debug, Clone)]
struct ExtractedRuleAnnotations {
    syntax_elements: Vec<serde_json::Value>,
    branch_return_annotations: Vec<Option<BranchAnnotation>>,
    branch_semantic_annotations: Vec<Vec<SemanticAnnotation>>,
    branch_mid_sequence_semantic_annotations: Vec<Vec<MidSequenceSemanticAnnotation>>,
    semantic_annotations: Vec<SemanticAnnotation>,
}

impl RustASTPipeline {
    pub fn new(config: PipelineConfig) -> Self {
        set_global_trace_verbosity(config.trace_verbosity);
        RustASTPipeline { config }
    }

    /// Transform raw AST JSON into processed AST format
    pub fn transform_from_raw_ast(
        &self,
        raw_ast_data: &[serde_json::Value],
    ) -> Result<(HashMap<String, ASTNode>, Vec<String>, Option<Annotations>)> {
        eprintln!("\n{}", "=".repeat(80));
        eprintln!("🔄  AST PIPELINE TRANSFORMATION STARTED");
        eprintln!("{}", "=".repeat(80));
        eprintln!(
            "📊  Processing {} raw AST elements into structured grammar",
            raw_ast_data.len()
        );
        eprintln!("📂  File: {}:{}", file!(), line!());
        eprintln!();

        let mut grammar_tree = HashMap::new();
        let mut rule_order = Vec::new();
        let mut annotations = Annotations::default();

        for (rule_idx, rule_data) in raw_ast_data.iter().enumerate() {
            eprintln!("   📋  Rule {}/{}", rule_idx + 1, raw_ast_data.len());
            eprintln!(
                "       Raw JSON: {}",
                rule_data.to_string().chars().take(80).collect::<String>()
                    + if rule_data.to_string().len() > 80 {
                        "..."
                    } else {
                        ""
                    }
            );
            eprintln!("       File: {}:{}", file!(), line!());

            if let Some(rule_array) = rule_data.as_array() {
                if rule_array.is_empty() {
                    eprintln!("       ⚠️   WARNING: Skipping empty rule array");
                    eprintln!("       File: {}:{}", file!(), line!());
                    eprintln!();
                    continue;
                }

                // First element should be ["rule", "rule_name"]
                if let Some(first_elem) = rule_array.first() {
                    if let Some(rule_name) = self.extract_rule_name(first_elem) {
                        eprintln!("       ✅  Rule declaration found: '{}' ", rule_name);
                        eprintln!("       File: {}:{}", file!(), line!());
                        if !rule_order.contains(&rule_name) {
                            rule_order.push(rule_name.clone());
                        }

                        // Parse the rule content (everything after the rule declaration)
                        let rule_content = &rule_array[1..];
                        eprintln!(
                            "       🔍  Parsing {} content elements for rule '{}'",
                            rule_content.len(),
                            rule_name
                        );
                        eprintln!("       File: {}:{}", file!(), line!());

                        let parsed_rule = self.parse_rule_content(rule_content)?;

                        eprintln!(
                            "       🎯  Rule '{}' successfully transformed to AST",
                            rule_name
                        );
                        eprintln!("       Result: {:?}", parsed_rule.ast_node);
                        eprintln!("       File: {}:{}", file!(), line!());
                        if self.config.preserve_annotations {
                            if parsed_rule
                                .branch_return_annotations
                                .iter()
                                .any(|entry| entry.is_some())
                            {
                                annotations
                                    .branch_return_annotations
                                    .entry(rule_name.clone())
                                    .or_default()
                                    .extend(parsed_rule.branch_return_annotations.clone());
                            }
                            if parsed_rule.branch_semantic_annotations.len() > 1
                                || parsed_rule
                                    .branch_semantic_annotations
                                    .iter()
                                    .any(|entry| !entry.is_empty())
                            {
                                annotations
                                    .branch_semantic_annotations
                                    .entry(rule_name.clone())
                                    .or_default()
                                    .extend(parsed_rule.branch_semantic_annotations.clone());
                            }
                            if parsed_rule
                                .branch_mid_sequence_semantic_annotations
                                .iter()
                                .any(|entry| !entry.is_empty())
                            {
                                annotations
                                    .branch_mid_sequence_semantic_annotations
                                    .entry(rule_name.clone())
                                    .or_default()
                                    .extend(
                                        parsed_rule
                                            .branch_mid_sequence_semantic_annotations
                                            .clone(),
                                    );
                            }
                            if !parsed_rule.semantic_annotations.is_empty() {
                                annotations
                                    .semantic_annotations
                                    .entry(rule_name.clone())
                                    .or_default()
                                    .extend(parsed_rule.semantic_annotations.clone());
                            }
                        }
                        if let Some(existing_rule) = grammar_tree.get(&rule_name).cloned() {
                            let mut merged_alternatives = Self::as_alternatives(&existing_rule);
                            merged_alternatives
                                .extend(Self::as_alternatives(&parsed_rule.ast_node));
                            grammar_tree
                                .insert(rule_name, Self::build_or_node(merged_alternatives));
                        } else {
                            grammar_tree.insert(rule_name, parsed_rule.ast_node);
                        }
                        eprintln!();
                    } else {
                        eprintln!("       ❌  ERROR: Failed to extract rule name from element");
                        eprintln!("       Element: {:?}", first_elem);
                        eprintln!("       File: {}:{}", file!(), line!());
                        eprintln!();
                    }
                } else {
                    eprintln!("       ❌  ERROR: Rule array has no first element");
                    eprintln!("       File: {}:{}", file!(), line!());
                    eprintln!();
                }
            } else {
                eprintln!("       ❌  ERROR: Rule data is not an array");
                eprintln!(
                    "       Data type: {}",
                    std::any::type_name::<serde_json::Value>()
                );
                eprintln!("       File: {}:{}", file!(), line!());
                eprintln!();
            }
        }

        if self.config.eliminate_left_recursion {
            self.eliminate_left_recursive_patterns(
                &mut grammar_tree,
                &mut rule_order,
                Some(&mut annotations),
            );
        } else {
            eprintln!(
                "[mod.rs][transform_from_raw_ast()] ⏭️  Left-recursion elimination disabled by configuration"
            );
        }

        eprintln!("🎉  TRANSFORMATION COMPLETE");
        eprintln!("📊  Generated grammar with {} rules", grammar_tree.len());
        eprintln!("📋  Rule execution order: {:?}", rule_order);
        eprintln!("📂  File: {}:{}", file!(), line!());
        eprintln!("{}", "=".repeat(80));
        eprintln!();

        let annotations = if self.config.preserve_annotations
            && (!annotations.branch_return_annotations.is_empty()
                || !annotations.branch_semantic_annotations.is_empty()
                || !annotations
                    .branch_mid_sequence_semantic_annotations
                    .is_empty()
                || !annotations.semantic_annotations.is_empty())
        {
            Some(annotations)
        } else {
            None
        };

        Ok((grammar_tree, rule_order, annotations))
    }

    fn eliminate_left_recursive_patterns(
        &self,
        grammar_tree: &mut HashMap<String, ASTNode>,
        rule_order: &mut Vec<String>,
        mut annotations: Option<&mut Annotations>,
    ) {
        eprintln!(
            "[mod.rs][eliminate_left_recursive_patterns()] 🔧 Starting left-recursion elimination pass"
        );
        // Take a snapshot of the user's grammar-author-written branch return
        // annotations BEFORE any LR-elim migration. Strategy 3a's annotation
        // rewrite migrates per-branch annotations from base rules to
        // `*_lr_base` helpers; the inventory contract should still surface
        // the original layout so grammar authors recognise their own
        // annotations and the pre-LR-elim crosscheck against the frontend
        // raw_ast JSON stays valid.
        if let Some(annotations_ref) = annotations.as_deref_mut() {
            if annotations_ref
                .pre_lr_elim_branch_return_annotations
                .is_none()
            {
                annotations_ref.pre_lr_elim_branch_return_annotations =
                    Some(annotations_ref.branch_return_annotations.clone());
            }
        }
        let original_order = rule_order.clone();
        let mut transformed_rules = HashSet::new();
        let mut transformation_count = 0usize;

        for rule_name in original_order {
            if transformed_rules.contains(&rule_name) {
                continue;
            }

            let Some(plan) = self.detect_left_recursive_chain_plan(&rule_name, grammar_tree) else {
                continue;
            };

            eprintln!(
                "[mod.rs][eliminate_left_recursive_patterns()] ✅ Rewriting left-recursive chain for rule '{}' via helper '{}' ({} wrapper rules)",
                plan.base_rule,
                plan.helper_base_rule,
                plan.wrapper_rules.len()
            );

            self.apply_left_recursive_chain_plan(
                &plan,
                grammar_tree,
                rule_order,
                annotations.as_deref_mut(),
            );
            transformation_count += 1;
            transformed_rules.insert(plan.base_rule.clone());
            for (_orig_idx, wrapper_rule, _suffix) in &plan.wrapper_rules {
                transformed_rules.insert(wrapper_rule.clone());
            }
            transformed_rules.insert(plan.helper_base_rule.clone());
        }

        eprintln!(
            "[mod.rs][eliminate_left_recursive_patterns()] 🏁 Completed left-recursion elimination pass ({} transformations)",
            transformation_count
        );
    }

    fn detect_left_recursive_chain_plan(
        &self,
        rule_name: &str,
        grammar_tree: &HashMap<String, ASTNode>,
    ) -> Option<LeftRecursiveChainPlan> {
        let rule_node = grammar_tree.get(rule_name)?;
        let rule_alternatives = Self::as_alternatives(rule_node);
        if rule_alternatives.is_empty() {
            return None;
        }

        let mut base_alternatives: Vec<(usize, ASTNode)> = Vec::new();
        let mut wrapper_rules: Vec<(usize, String, ASTNode)> = Vec::new();

        for (orig_idx, alternative) in rule_alternatives.iter().enumerate() {
            if let Some(wrapper_rule) = Self::extract_rule_reference_name(alternative) {
                if let Some(wrapper_suffix) =
                    Self::extract_wrapper_suffix(rule_name, &wrapper_rule, grammar_tree)
                {
                    wrapper_rules.push((orig_idx, wrapper_rule, wrapper_suffix));
                    continue;
                }
            }
            base_alternatives.push((orig_idx, alternative.clone()));
        }

        if wrapper_rules.is_empty() || base_alternatives.is_empty() {
            return None;
        }

        let suffix_alternative = Self::build_or_node(
            wrapper_rules
                .iter()
                .map(|(_, _, suffix)| suffix.clone())
                .collect(),
        );

        let helper_base_rule =
            Self::allocate_synthetic_rule_name(format!("{}_lr_base", rule_name), grammar_tree);
        let helper_suffix_rule =
            Self::allocate_synthetic_rule_name(format!("{}_lr_suffix", rule_name), grammar_tree);

        Some(LeftRecursiveChainPlan {
            base_rule: rule_name.to_string(),
            helper_base_rule,
            helper_suffix_rule,
            base_alternatives,
            wrapper_rules,
            suffix_alternative,
        })
    }

    fn apply_left_recursive_chain_plan(
        &self,
        plan: &LeftRecursiveChainPlan,
        grammar_tree: &mut HashMap<String, ASTNode>,
        rule_order: &mut Vec<String>,
        annotations: Option<&mut Annotations>,
    ) {
        let helper_base_ref = Self::make_rule_reference_node(&plan.helper_base_rule);

        // Strategy 3a only kicks in when annotations are in scope. When they
        // are, the suffix Or is hoisted into a `<base>_lr_suffix` helper rule
        // so per-branch annotations carrying `alt_index`/`captures` can ride
        // each suffix iteration. When annotations are not preserved, keep the
        // legacy inline Quantified(suffix_alternative) shape.
        let want_chain_annotations = annotations.is_some();
        let suffix_element_node = if want_chain_annotations {
            Self::make_rule_reference_node(&plan.helper_suffix_rule)
        } else {
            plan.suffix_alternative.clone()
        };
        let suffix_repetition = ASTNode::Quantified {
            element: Box::new(suffix_element_node),
            quantifier: "*".to_string(),
        };

        let rewritten_base_rule =
            Self::build_sequence_node(vec![helper_base_ref.clone(), suffix_repetition.clone()]);
        grammar_tree.insert(plan.base_rule.clone(), rewritten_base_rule);

        // Track each wrapper rule's ORIGINAL body length so we can compute
        // where the appended `suffix_repetition` lives ($N where N is one
        // past the original body length) — this becomes the `suffixes:` $-ref
        // in the wrapper rule's synthetic `_pgen_lr_chain` annotation.
        let mut wrapper_original_body_lengths: Vec<usize> =
            Vec::with_capacity(plan.wrapper_rules.len());
        for (_orig_idx, wrapper_rule, wrapper_suffix) in &plan.wrapper_rules {
            // Flatten wrapper_suffix's elements into the outer Sequence rather
            // than nesting it. This preserves the original 1-based `$N`
            // element-index convention in any inline return annotation
            // attached to this wrapper rule. Without flattening, the rewritten
            // rule body becomes `Sequence([helper_base_ref, wrapper_suffix,
            // suffix_repetition])` — three top-level elements regardless of
            // the original body length — and the grammar author's `$3`
            // reference (intended for, say, `identifier` in
            // `accessor_base '.' identifier`) ends up pointing at the
            // synthetic `suffix_repetition` instead. Flattening preserves the
            // original element positions; the only NEW element is the
            // tail-position `suffix_repetition` whose index is past the
            // original body's range.
            let mut flat_elements: Vec<ASTNode> = Vec::new();
            flat_elements.push(helper_base_ref.clone());
            let suffix_element_count = match wrapper_suffix {
                ASTNode::Sequence { elements } => {
                    flat_elements.extend(elements.iter().cloned());
                    elements.len()
                }
                other => {
                    flat_elements.push(other.clone());
                    1
                }
            };
            // Original body length = LR-self-ref (1) + suffix elements.
            wrapper_original_body_lengths.push(1 + suffix_element_count);
            flat_elements.push(suffix_repetition.clone());
            let rewritten_wrapper = Self::build_sequence_node(flat_elements);
            grammar_tree.insert(wrapper_rule.clone(), rewritten_wrapper);
        }

        let helper_base_ast = Self::build_or_node(
            plan.base_alternatives
                .iter()
                .map(|(_, alt)| alt.clone())
                .collect(),
        );
        grammar_tree.insert(plan.helper_base_rule.clone(), helper_base_ast);

        if !rule_order.contains(&plan.helper_base_rule) {
            if let Some(base_pos) = rule_order.iter().position(|name| name == &plan.base_rule) {
                rule_order.insert(base_pos, plan.helper_base_rule.clone());
            } else {
                rule_order.push(plan.helper_base_rule.clone());
            }
        }

        if want_chain_annotations {
            grammar_tree
                .insert(plan.helper_suffix_rule.clone(), plan.suffix_alternative.clone());
            if !rule_order.contains(&plan.helper_suffix_rule) {
                if let Some(base_pos) = rule_order.iter().position(|name| name == &plan.base_rule)
                {
                    rule_order.insert(base_pos, plan.helper_suffix_rule.clone());
                } else {
                    rule_order.push(plan.helper_suffix_rule.clone());
                }
            }
        }

        if let Some(annotations) = annotations {
            self.rewrite_lr_chain_annotations(
                plan,
                &wrapper_original_body_lengths,
                annotations,
            );
        }
    }

    /// Strategy 3a annotation rewrite. Mutates `annotations` to:
    ///
    /// - Migrate per-branch annotations from `plan.base_rule` (whose original
    ///   Or has been replaced by a Sequence) onto `helper_base_rule` at the
    ///   surviving alternatives' new branch indices.
    /// - Replace `plan.base_rule`'s branch annotations with a single synthetic
    ///   `_pgen_lr_chain` entry whose `initial: $1`, `suffixes: $2`, and
    ///   `wrapper_specs` carry the chain-fold metadata.
    /// - Replace each wrapper rule's branch[0] annotation with a synthetic
    ///   `_pgen_lr_chain` entry whose `initial` embeds the wrapper's original
    ///   parsed AST verbatim (with `$1..$N` still meaning the wrapper's
    ///   original body positions — the flatten path preserved them) and
    ///   whose `suffixes` reads from the trailing `suffix_repetition`.
    /// - Add per-branch `_pgen_lr_chain_alt` annotations to `helper_suffix_rule`,
    ///   one per branch, that emit `{alt_index, captures: [$1, .., $M]}`.
    ///
    /// Synthetic entries (those with `annotation_type == "_pgen_lr_chain_synthetic"`)
    /// are skipped by the inventory builder so the grammar's tracked
    /// declared-annotation contract stays stable across this rewrite.
    fn rewrite_lr_chain_annotations(
        &self,
        plan: &LeftRecursiveChainPlan,
        wrapper_original_body_lengths: &[usize],
        annotations: &mut Annotations,
    ) {
        // ---- 1. Migrate base_rule's per-branch annotations to helper_base_rule.
        // The base rule's branch_return_annotations Vec was indexed against
        // the original Or's branch positions. After LR-elim, helper_base_rule
        // owns the surviving non-LR alternatives; its branch indices are the
        // positions in plan.base_alternatives, in order.
        let original_base_branches = annotations
            .branch_return_annotations
            .remove(&plan.base_rule)
            .unwrap_or_default();
        let migrated_helper_base_branches: Vec<Option<BranchAnnotation>> = plan
            .base_alternatives
            .iter()
            .map(|(orig_idx, _)| {
                original_base_branches
                    .get(*orig_idx)
                    .cloned()
                    .unwrap_or(None)
            })
            .collect();
        if migrated_helper_base_branches.iter().any(|e| e.is_some()) {
            annotations
                .branch_return_annotations
                .insert(plan.helper_base_rule.clone(), migrated_helper_base_branches);
        }

        // ---- 2. Compute wrapper_specs (one entry per branch of the suffix Or).
        // For each wrapper rule, its wrapper_suffix may be a Sequence (single
        // suffix branch) or another shape; we expand each into one or more
        // alts in suffix order. alt_index runs sequentially across wrappers.
        let mut wrapper_specs: Vec<unified_return_ast::LrChainWrapperSpec> = Vec::new();
        let mut alt_index_cursor = 0usize;
        // Track per-helper-suffix-branch the matching wrapper's original body
        // length and elements count for capture annotation generation.
        let mut suffix_branch_metadata: Vec<usize> = Vec::new(); // captures count per branch
        for ((_orig_idx, wrapper_rule, wrapper_suffix), wrapper_body_len) in plan
            .wrapper_rules
            .iter()
            .zip(wrapper_original_body_lengths.iter())
        {
            let wrapper_branch_annotation: UnifiedReturnAST = annotations
                .branch_return_annotations
                .get(wrapper_rule)
                .and_then(|branches| branches.get(0).cloned().flatten())
                .and_then(|ann| ann.parsed_ast)
                .unwrap_or(UnifiedReturnAST::PositionalRef { index: 1 });

            let branch_element_counts: Vec<usize> = match wrapper_suffix {
                ASTNode::Or { alternatives } => alternatives
                    .iter()
                    .map(|alt| match alt {
                        ASTNode::Sequence { elements } => elements.len(),
                        _ => 1,
                    })
                    .collect(),
                ASTNode::Sequence { elements } => vec![elements.len()],
                _ => vec![1],
            };
            for elem_count in &branch_element_counts {
                wrapper_specs.push(unified_return_ast::LrChainWrapperSpec {
                    alt_index: alt_index_cursor,
                    original_body_length: *wrapper_body_len,
                    annotation_template: wrapper_branch_annotation.clone(),
                });
                suffix_branch_metadata.push(*elem_count);
                alt_index_cursor += 1;
            }
        }

        let wrapper_specs_serialized = serde_json::to_string(&wrapper_specs)
            .expect("LrChainWrapperSpec Serialize must not fail");

        // ---- 3. Build the synthetic _pgen_lr_chain Object literal that's
        // attached as the rewritten annotation. `wrapper_specs_str` is shared
        // across base_rule and all wrapper rules.
        let wrapper_specs_node = UnifiedReturnAST::StringLiteral {
            value: wrapper_specs_serialized,
        };
        let make_chain_annotation = |initial: UnifiedReturnAST, suffix_position: usize| {
            let mut props: std::collections::HashMap<String, Box<UnifiedReturnAST>> =
                std::collections::HashMap::new();
            props.insert(
                "type".to_string(),
                Box::new(UnifiedReturnAST::StringLiteral {
                    value: "_pgen_lr_chain".to_string(),
                }),
            );
            props.insert("initial".to_string(), Box::new(initial));
            props.insert(
                "suffixes".to_string(),
                Box::new(UnifiedReturnAST::PositionalRef {
                    index: suffix_position,
                }),
            );
            props.insert(
                "wrapper_specs".to_string(),
                Box::new(wrapper_specs_node.clone()),
            );
            UnifiedReturnAST::Object { properties: props }
        };

        // ---- 4. Replace base_rule's annotation with a single synthetic
        // _pgen_lr_chain entry. The base rule's rewritten body is
        // `[helper_base_ref, suffix_repetition]`, so initial = $1 and
        // suffixes = $2.
        let base_chain_ast =
            make_chain_annotation(UnifiedReturnAST::PositionalRef { index: 1 }, 2);
        annotations.branch_return_annotations.insert(
            plan.base_rule.clone(),
            vec![Some(BranchAnnotation {
                annotation_type: "_pgen_lr_chain_synthetic".to_string(),
                annotation_content: String::new(),
                parsed_ast: Some(base_chain_ast),
            })],
        );

        // ---- 5. Replace each wrapper rule's branch[0] annotation with a
        // synthetic _pgen_lr_chain entry that wraps the original. We KEEP
        // the original `annotation_content` and `annotation_type` so the
        // emitted return-annotation inventory continues to surface the
        // grammar-author-written text and its declared type — the rewrite
        // is invisible to the contract gate. Only `parsed_ast` flips to the
        // chain shape (which is what codegen consumes).
        for ((_orig_idx, wrapper_rule, _suffix), wrapper_body_len) in plan
            .wrapper_rules
            .iter()
            .zip(wrapper_original_body_lengths.iter())
        {
            let suffix_position = wrapper_body_len + 1;
            let wrapper_branches = annotations
                .branch_return_annotations
                .entry(wrapper_rule.clone())
                .or_insert_with(|| vec![None]);
            if wrapper_branches.is_empty() {
                wrapper_branches.push(None);
            }
            let original_initial = wrapper_branches[0]
                .as_ref()
                .and_then(|ann| ann.parsed_ast.clone())
                .unwrap_or(UnifiedReturnAST::PositionalRef { index: 1 });
            let chain_ast = make_chain_annotation(original_initial, suffix_position);
            let (preserved_type, preserved_content) = match &wrapper_branches[0] {
                Some(existing) => (
                    existing.annotation_type.clone(),
                    existing.annotation_content.clone(),
                ),
                None => (
                    "_pgen_lr_chain_synthetic".to_string(),
                    String::new(),
                ),
            };
            wrapper_branches[0] = Some(BranchAnnotation {
                annotation_type: preserved_type,
                annotation_content: preserved_content,
                parsed_ast: Some(chain_ast),
            });
        }

        // ---- 6. Add per-branch annotations to helper_suffix_rule so each
        // suffix iteration emits `{type: "_pgen_lr_chain_alt", alt_index,
        // captures: [$1, .., $M]}` at runtime — the shape the walker's
        // chain fold step consumes.
        let suffix_branch_annotations: Vec<Option<BranchAnnotation>> = suffix_branch_metadata
            .iter()
            .enumerate()
            .map(|(alt_index, captures_count)| {
                let mut props: std::collections::HashMap<String, Box<UnifiedReturnAST>> =
                    std::collections::HashMap::new();
                props.insert(
                    "type".to_string(),
                    Box::new(UnifiedReturnAST::StringLiteral {
                        value: "_pgen_lr_chain_alt".to_string(),
                    }),
                );
                props.insert(
                    "alt_index".to_string(),
                    Box::new(UnifiedReturnAST::NumberLiteral {
                        value: alt_index as f64,
                    }),
                );
                let captures_array_elements: Vec<UnifiedReturnAST> = (1..=*captures_count)
                    .map(|i| UnifiedReturnAST::PositionalRef { index: i })
                    .collect();
                props.insert(
                    "captures".to_string(),
                    Box::new(UnifiedReturnAST::Array {
                        elements: captures_array_elements,
                    }),
                );
                Some(BranchAnnotation {
                    annotation_type: "_pgen_lr_chain_synthetic".to_string(),
                    annotation_content: String::new(),
                    parsed_ast: Some(UnifiedReturnAST::Object { properties: props }),
                })
            })
            .collect();
        if !suffix_branch_annotations.is_empty() {
            annotations
                .branch_return_annotations
                .insert(plan.helper_suffix_rule.clone(), suffix_branch_annotations);
        }
    }

    fn as_alternatives(node: &ASTNode) -> Vec<ASTNode> {
        match node {
            ASTNode::Or { alternatives } => alternatives.clone(),
            _ => vec![node.clone()],
        }
    }

    fn build_or_node(mut alternatives: Vec<ASTNode>) -> ASTNode {
        if alternatives.len() == 1 {
            alternatives.remove(0)
        } else {
            ASTNode::Or { alternatives }
        }
    }

    fn build_sequence_node(mut elements: Vec<ASTNode>) -> ASTNode {
        if elements.len() == 1 {
            elements.remove(0)
        } else {
            ASTNode::Sequence { elements }
        }
    }

    fn make_rule_reference_node(rule_name: &str) -> ASTNode {
        ASTNode::Atom {
            value: ASTValue::Token(vec![
                TokenValue::String("rule_reference".to_string()),
                TokenValue::String(rule_name.to_string()),
            ]),
        }
    }

    fn allocate_synthetic_rule_name(
        base_name: String,
        grammar_tree: &HashMap<String, ASTNode>,
    ) -> String {
        if !grammar_tree.contains_key(&base_name) {
            return base_name;
        }

        let mut index = 1usize;
        loop {
            let candidate = format!("{}_{}", base_name, index);
            if !grammar_tree.contains_key(&candidate) {
                return candidate;
            }
            index += 1;
        }
    }

    fn extract_rule_reference_name(node: &ASTNode) -> Option<String> {
        match node {
            ASTNode::Atom {
                value: ASTValue::Token(parts),
            } => {
                if parts.len() < 2 {
                    return None;
                }
                let TokenValue::String(token_type) = &parts[0] else {
                    return None;
                };
                let TokenValue::String(token_value) = &parts[1] else {
                    return None;
                };
                if token_type == "rule_reference" {
                    Some(token_value.clone())
                } else {
                    None
                }
            }
            ASTNode::Sequence { elements } if elements.len() == 1 => {
                Self::extract_rule_reference_name(&elements[0])
            }
            _ => None,
        }
    }

    fn sequence_suffix_if_prefixed_with_rule(
        elements: &[ASTNode],
        base_rule: &str,
    ) -> Option<ASTNode> {
        if elements.is_empty() {
            return None;
        }
        if Self::extract_rule_reference_name(&elements[0]).as_deref() != Some(base_rule) {
            return None;
        }
        if elements.len() < 2 {
            return None;
        }
        Some(Self::build_sequence_node(elements[1..].to_vec()))
    }

    fn extract_wrapper_suffix(
        base_rule: &str,
        wrapper_rule: &str,
        grammar_tree: &HashMap<String, ASTNode>,
    ) -> Option<ASTNode> {
        let wrapper_node = grammar_tree.get(wrapper_rule)?;
        match wrapper_node {
            ASTNode::Sequence { elements } => {
                Self::sequence_suffix_if_prefixed_with_rule(elements, base_rule)
            }
            ASTNode::Or { alternatives } => {
                let mut suffixes = Vec::new();
                for alternative in alternatives {
                    let ASTNode::Sequence { elements } = alternative else {
                        return None;
                    };
                    let Some(suffix) =
                        Self::sequence_suffix_if_prefixed_with_rule(elements, base_rule)
                    else {
                        return None;
                    };
                    suffixes.push(suffix);
                }
                if suffixes.is_empty() {
                    None
                } else {
                    Some(Self::build_or_node(suffixes))
                }
            }
            ASTNode::Lookahead { .. } => None,
            _ => None,
        }
    }

    fn extract_rule_name(&self, rule_decl: &serde_json::Value) -> Option<String> {
        if let Some(arr) = rule_decl.as_array() {
            if arr.len() >= 2 {
                if let (Some(type_str), Some(name_str)) = (arr[0].as_str(), arr[1].as_str()) {
                    if type_str == "rule" {
                        return Some(name_str.to_string());
                    }
                }
            }
        }
        None
    }

    fn parse_rule_content(&self, content: &[serde_json::Value]) -> Result<ParsedRuleContent> {
        if content.is_empty() {
            eprintln!(
                "[mod.rs][parse_rule_content()] 📝 Rule content is empty - creating empty sequence node"
            );
            eprintln!("   File: {}:{}", file!(), line!());
            return Ok(ParsedRuleContent {
                ast_node: ASTNode::Sequence { elements: vec![] },
                branch_return_annotations: vec![None],
                branch_semantic_annotations: vec![Vec::new()],
                branch_mid_sequence_semantic_annotations: vec![Vec::new()],
                semantic_annotations: Vec::new(),
            });
        }

        eprintln!("   🏗️   RULE CONTENT PARSING (STAGED PIPELINE)");
        eprintln!("        Elements to process: {}", content.len());
        eprintln!("        File: {}:{}", file!(), line!());

        let extracted = self.extract_rule_annotations(content)?;
        eprintln!(
            "        Annotation extraction: {} branch return slot(s), {} branch semantic slot(s), {} branch mid-sequence slot(s), {} rule semantic annotation(s)",
            extracted.branch_return_annotations.len(),
            extracted.branch_semantic_annotations.len(),
            extracted.branch_mid_sequence_semantic_annotations.len(),
            extracted.semantic_annotations.len()
        );

        if extracted.syntax_elements.is_empty() {
            let mut branch_return_annotations = extracted.branch_return_annotations;
            let mut branch_semantic_annotations = extracted.branch_semantic_annotations;
            let mut branch_mid_sequence_semantic_annotations =
                extracted.branch_mid_sequence_semantic_annotations;
            if branch_return_annotations.is_empty() {
                branch_return_annotations.push(None);
            }
            if branch_semantic_annotations.is_empty() {
                branch_semantic_annotations.push(Vec::new());
            }
            if branch_mid_sequence_semantic_annotations.is_empty() {
                branch_mid_sequence_semantic_annotations.push(Vec::new());
            }
            return Ok(ParsedRuleContent {
                ast_node: ASTNode::Sequence { elements: vec![] },
                branch_return_annotations,
                branch_semantic_annotations,
                branch_mid_sequence_semantic_annotations,
                semantic_annotations: extracted.semantic_annotations,
            });
        }

        eprintln!("        Stage-1: normalize raw elements");
        let normalized = self.step1_normalize_raw_elements(&extracted.syntax_elements)?;
        eprintln!(
            "        Stage-1 result: {} normalized elements",
            normalized.len()
        );
        eprintln!("        Stage-2: group top-level alternatives (|)");
        let branches = self.step2_group_by_or(&normalized);
        eprintln!(
            "        Stage-2 result: {} top-level branches",
            branches.len()
        );
        eprintln!("        Stage-2.5: handle parentheses/groups per branch");
        let mut branch_asts = Vec::with_capacity(branches.len());
        for (branch_idx, branch) in branches.iter().enumerate() {
            eprintln!(
                "          🔀 Branch {}/{} has {} elements",
                branch_idx + 1,
                branches.len(),
                branch.len()
            );
            let branch_elements = self.step2_5_handle_parentheses(branch)?;
            eprintln!(
                "          ✅ Branch {} grouped into {} sequence elements",
                branch_idx + 1,
                branch_elements.len()
            );
            eprintln!("          Stage-3: build sequence nodes");
            let branch_ast = self.step3_parse_sequences(branch_elements);
            branch_asts.push(branch_ast);
        }
        eprintln!("        Stage-5: build final tree structure");
        let result = self.step5_build_tree_structure(branch_asts);

        eprintln!("   🏆  Rule content parsing complete (staged pipeline)");
        eprintln!("       Final AST: {:?}", result);
        eprintln!("       File: {}:{}", file!(), line!());

        let mut branch_return_annotations = extracted.branch_return_annotations;
        let mut branch_semantic_annotations = extracted.branch_semantic_annotations;
        let mut branch_mid_sequence_semantic_annotations =
            extracted.branch_mid_sequence_semantic_annotations;
        let branch_count = match &result {
            ASTNode::Or { alternatives } => alternatives.len(),
            _ => 1,
        };
        if branch_return_annotations.len() < branch_count {
            branch_return_annotations.resize(branch_count, None);
        } else if branch_return_annotations.len() > branch_count {
            branch_return_annotations.truncate(branch_count);
        }
        if branch_semantic_annotations.len() < branch_count {
            branch_semantic_annotations.resize_with(branch_count, Vec::new);
        } else if branch_semantic_annotations.len() > branch_count {
            branch_semantic_annotations.truncate(branch_count);
        }
        if branch_mid_sequence_semantic_annotations.len() < branch_count {
            branch_mid_sequence_semantic_annotations.resize_with(branch_count, Vec::new);
        } else if branch_mid_sequence_semantic_annotations.len() > branch_count {
            branch_mid_sequence_semantic_annotations.truncate(branch_count);
        }

        Ok(ParsedRuleContent {
            ast_node: result,
            branch_return_annotations,
            branch_semantic_annotations,
            branch_mid_sequence_semantic_annotations,
            semantic_annotations: extracted.semantic_annotations,
        })
    }

    fn extract_rule_annotations(
        &self,
        content: &[serde_json::Value],
    ) -> Result<ExtractedRuleAnnotations> {
        let mut syntax_elements = Vec::with_capacity(content.len());
        let mut branch_return_annotations: Vec<Option<BranchAnnotation>> = vec![None];
        let mut branch_semantic_annotations: Vec<Vec<SemanticAnnotation>> = vec![Vec::new()];
        let mut branch_mid_sequence_semantic_annotations: Vec<Vec<MidSequenceSemanticAnnotation>> =
            vec![Vec::new()];
        let mut semantic_annotations = Vec::new();
        let mut branch_syntax_positions: Vec<usize> = vec![0];

        let mut group_depth = 0usize;
        let mut branch_idx = 0usize;

        // Track the branch_idx active at each `group_open`. When the
        // matching `group_close` is encountered, pop and remember the
        // (open_branch_idx ..= close_branch_idx) range so a return
        // annotation IMMEDIATELY following the close can be broadcast
        // to every branch that was inside the just-closed group.
        // This fixes task #38 — `RULE = (A | B | C) -> ann` previously
        // landed the annotation on branch 0 only, leaving branches 1+
        // with raw passthrough.
        let mut group_open_branch_stack: Vec<usize> = Vec::new();
        let mut last_closed_group_range: Option<(usize, usize)> = None;

        // Map inner branch_idx → outer (top-level) branch_idx. Inner branches
        // are created by `|` at any group_depth (needed for broadcast); outer
        // branches are created only by `|` at group_depth == 0. The AST after
        // step2_group_by_or only carries outer branches, so `branch_return_annotations`
        // must be remapped from inner to outer indices before the truncation
        // at parse_rule_content (line ~1922) would lop off inner-indexed
        // entries. See codegen-drop fix for patterns:
        //   (A) `id ( a | b )* -> ann` (binary_value/hex_value/etc.)
        //   (B) `( a | b | c )? id -> ann` (ps_type_identifier_sv_2017/2023)
        //   (C) `RULE = X | Y | ( a )? token id lparen ( e )? rparen -> ann`
        //                                                              (^ ansi_port_declaration branch 3)
        //   (D) per-branch annotation on `( a | b )? id` in multi-branch rule
        //                                              (method_call_receiver_*)
        let mut outer_branch_idx = 0usize;
        let mut branch_to_outer: Vec<usize> = vec![0];

        for item in content {
            let Some(arr) = item.as_array() else {
                syntax_elements.push(item.clone());
                last_closed_group_range = None;
                continue;
            };
            let Some(elem_type) = arr.first().and_then(|v| v.as_str()) else {
                syntax_elements.push(item.clone());
                last_closed_group_range = None;
                continue;
            };

            match elem_type {
                "group_open" => {
                    group_open_branch_stack.push(branch_idx);
                    group_depth = group_depth.saturating_add(1);
                    syntax_elements.push(item.clone());
                    last_closed_group_range = None;
                }
                "group_close" => {
                    group_depth = group_depth.saturating_sub(1);
                    if let Some(open_branch_idx) = group_open_branch_stack.pop() {
                        last_closed_group_range = Some((open_branch_idx, branch_idx));
                    } else {
                        last_closed_group_range = None;
                    }
                    syntax_elements.push(item.clone());
                }
                "operator" => {
                    let is_pipe = arr.get(1).and_then(|v| v.as_str()) == Some("|");
                    if is_pipe {
                        // Increment branch_idx for EVERY `|`, regardless of
                        // group depth. The `last_closed_group_range` mechanism
                        // (above) handles broadcasting trailing annotations
                        // back across grouped branches; tracking inner
                        // branches here is what makes that broadcast possible.
                        branch_idx = branch_idx.saturating_add(1);
                        if group_depth == 0 {
                            outer_branch_idx = outer_branch_idx.saturating_add(1);
                        }
                        if branch_to_outer.len() <= branch_idx {
                            branch_to_outer.push(outer_branch_idx);
                        }
                        if branch_return_annotations.len() <= branch_idx {
                            branch_return_annotations.push(None);
                        }
                        if branch_semantic_annotations.len() <= branch_idx {
                            branch_semantic_annotations.push(Vec::new());
                        }
                        if branch_mid_sequence_semantic_annotations.len() <= branch_idx {
                            branch_mid_sequence_semantic_annotations.push(Vec::new());
                        }
                        if branch_syntax_positions.len() <= branch_idx {
                            branch_syntax_positions.push(0);
                        }
                        last_closed_group_range = None;
                    }
                    syntax_elements.push(item.clone());
                    if !is_pipe {
                        if branch_syntax_positions.len() <= branch_idx {
                            branch_syntax_positions.resize(branch_idx + 1, 0);
                        }
                        branch_syntax_positions[branch_idx] =
                            branch_syntax_positions[branch_idx].saturating_add(1);
                        last_closed_group_range = None;
                    }
                }
                "return_scalar" | "return_array" | "return_object" => {
                    let Some(annotation_content) = arr.get(1).and_then(|v| v.as_str()) else {
                        eprintln!(
                            "[mod.rs][extract_rule_annotations()] ⚠️ malformed return annotation payload: {:?}",
                            item
                        );
                        continue;
                    };
                    let parsed_ast = self.parse_return_annotation_ast(annotation_content);

                    // Determine target branch range. If the annotation
                    // immediately follows a group_close at the rule top
                    // level, broadcast to every branch that was inside
                    // the just-closed group. Otherwise, the annotation
                    // lands on the current branch (the existing per-branch
                    // semantics). See task #38.
                    let (range_start, range_end) = match last_closed_group_range {
                        Some((s, e)) => (s, e),
                        None => (branch_idx, branch_idx),
                    };
                    if branch_return_annotations.len() <= range_end {
                        branch_return_annotations.resize(range_end + 1, None);
                    }
                    for tgt_idx in range_start..=range_end {
                        if branch_return_annotations[tgt_idx].is_some() {
                            eprintln!(
                                "[mod.rs][extract_rule_annotations()] ⚠️ multiple return annotations in branch {} - keeping last",
                                tgt_idx + 1
                            );
                        }
                        branch_return_annotations[tgt_idx] = Some(BranchAnnotation {
                            annotation_type: elem_type.to_string(),
                            annotation_content: annotation_content.to_string(),
                            parsed_ast: parsed_ast.clone(),
                        });
                    }
                    last_closed_group_range = None;
                }
                "semantic_annotation" => {
                    if let Some(payload) = arr.get(1) {
                        if let Some(annotation) =
                            self.parse_semantic_annotation_entry(payload, item)?
                        {
                            semantic_annotations.push(annotation);
                        }
                    } else {
                        eprintln!(
                            "[mod.rs][extract_rule_annotations()] ⚠️ semantic annotation missing payload: {:?}",
                            item
                        );
                    }
                }
                "semantic_annotation_inline" => {
                    if let Some(payload) = arr.get(1) {
                        if let Some(annotation) =
                            self.parse_semantic_annotation_entry(payload, item)?
                        {
                            if branch_semantic_annotations.len() <= branch_idx {
                                branch_semantic_annotations.resize_with(branch_idx + 1, Vec::new);
                            }
                            branch_semantic_annotations[branch_idx].push(annotation);
                        }
                    } else {
                        eprintln!(
                            "[mod.rs][extract_rule_annotations()] ⚠️ inline semantic annotation missing payload: {:?}",
                            item
                        );
                    }
                }
                "semantic_annotation_mid_sequence" => {
                    if let Some(payload) = arr.get(1) {
                        if let Some(annotation) =
                            self.parse_semantic_annotation_entry(payload, item)?
                        {
                            if branch_mid_sequence_semantic_annotations.len() <= branch_idx {
                                branch_mid_sequence_semantic_annotations
                                    .resize_with(branch_idx + 1, Vec::new);
                            }
                            if branch_syntax_positions.len() <= branch_idx {
                                branch_syntax_positions.resize(branch_idx + 1, 0);
                            }
                            branch_mid_sequence_semantic_annotations[branch_idx].push(
                                MidSequenceSemanticAnnotation {
                                    syntax_position: branch_syntax_positions[branch_idx],
                                    group_depth,
                                    annotation,
                                },
                            );
                        }
                    } else {
                        eprintln!(
                            "[mod.rs][extract_rule_annotations()] ⚠️ mid-sequence semantic annotation missing payload: {:?}",
                            item
                        );
                    }
                }
                _ => {
                    syntax_elements.push(item.clone());
                    if branch_syntax_positions.len() <= branch_idx {
                        branch_syntax_positions.resize(branch_idx + 1, 0);
                    }
                    branch_syntax_positions[branch_idx] =
                        branch_syntax_positions[branch_idx].saturating_add(1);
                    last_closed_group_range = None;
                }
            }
        }

        // Remap inner-indexed annotations to outer-indexed (top-level)
        // branches. parse_rule_content truncates these vectors to the
        // AST's top-level branch count; without this remap, inner-counted
        // entries get lopped off. The last annotation per outer branch wins
        // (matches existing "multiple return annotations in branch — keeping
        // last" warning semantics).
        let outer_count = outer_branch_idx + 1;
        let remap_returns = |inner: Vec<Option<BranchAnnotation>>| -> Vec<Option<BranchAnnotation>> {
            let mut out: Vec<Option<BranchAnnotation>> = vec![None; outer_count];
            for (i, slot) in inner.into_iter().enumerate() {
                if let Some(ann) = slot {
                    let outer_idx = branch_to_outer.get(i).copied().unwrap_or(0);
                    if outer_idx < out.len() {
                        out[outer_idx] = Some(ann);
                    }
                }
            }
            out
        };
        let remap_vec_vec = |inner: Vec<Vec<SemanticAnnotation>>| -> Vec<Vec<SemanticAnnotation>> {
            let mut out: Vec<Vec<SemanticAnnotation>> = vec![Vec::new(); outer_count];
            for (i, vec_anns) in inner.into_iter().enumerate() {
                if !vec_anns.is_empty() {
                    let outer_idx = branch_to_outer.get(i).copied().unwrap_or(0);
                    if outer_idx < out.len() {
                        out[outer_idx].extend(vec_anns);
                    }
                }
            }
            out
        };
        let remap_mid_seq = |inner: Vec<Vec<MidSequenceSemanticAnnotation>>| -> Vec<Vec<MidSequenceSemanticAnnotation>> {
            let mut out: Vec<Vec<MidSequenceSemanticAnnotation>> = vec![Vec::new(); outer_count];
            for (i, vec_anns) in inner.into_iter().enumerate() {
                if !vec_anns.is_empty() {
                    let outer_idx = branch_to_outer.get(i).copied().unwrap_or(0);
                    if outer_idx < out.len() {
                        out[outer_idx].extend(vec_anns);
                    }
                }
            }
            out
        };
        let branch_return_annotations = remap_returns(branch_return_annotations);
        let branch_semantic_annotations = remap_vec_vec(branch_semantic_annotations);
        let branch_mid_sequence_semantic_annotations =
            remap_mid_seq(branch_mid_sequence_semantic_annotations);

        Ok(ExtractedRuleAnnotations {
            syntax_elements,
            branch_return_annotations,
            branch_semantic_annotations,
            branch_mid_sequence_semantic_annotations,
            semantic_annotations,
        })
    }

    fn parse_return_annotation_ast(&self, annotation_content: &str) -> Option<UnifiedReturnAST> {
        let content = annotation_content.trim();
        if content.is_empty() {
            return None;
        }

        let logger = runtime_logger("pipeline.return_annotation.bootstrap");
        if !self.config.bootstrap_mode {
            if !self.validate_return_annotation_backend(content) {
                eprintln!(
                    "[mod.rs][parse_return_annotation_ast()] ⚠️ selected backend could not validate return annotation '{}'",
                    content
                );
                return None;
            }

            #[cfg(feature = "generated_parsers")]
            {
                let mut parser = Return_annotationParser::new(
                    content,
                    runtime_logger_box("pipeline.return_annotation.generated"),
                );
                match parser.parse_full_return_annotation() {
                    Ok(parse_tree) => {
                        return match UnifiedReturnAST::parse_generated_return_annotation(
                            content,
                            &parse_tree,
                            &logger,
                        ) {
                            Ok(ast) => Some(ast),
                            Err(err) => {
                                eprintln!(
                                    "[mod.rs][parse_return_annotation_ast()] ⚠️ generated return tree -> typed AST failed for '{}' ({})",
                                    content, err
                                );
                                None
                            }
                        };
                    }
                    Err(err) => {
                        eprintln!(
                            "[mod.rs][parse_return_annotation_ast()] ⚠️ generated parser failed for '{}' ({})",
                            content, err
                        );
                        return None;
                    }
                }
            }
        }

        match UnifiedReturnAST::parse_bootstrap(content, &logger) {
            Ok(ast) => Some(ast),
            Err(err) => {
                eprintln!(
                    "[mod.rs][parse_return_annotation_ast()] ⚠️ failed to build typed return AST for '{}' ({})",
                    content, err
                );
                None
            }
        }
    }

    fn parse_semantic_annotation_entry(
        &self,
        payload: &serde_json::Value,
        original_element: &serde_json::Value,
    ) -> Result<Option<SemanticAnnotation>> {
        match payload {
            serde_json::Value::Array(parts) if parts.len() >= 2 => {
                let name = self.semantic_value_to_string(&parts[0]);
                let annotation_name = name.trim().to_ascii_lowercase();
                if annotation_name.is_empty() {
                    eprintln!(
                        "[mod.rs][parse_semantic_annotation_entry()] ⚠️ empty semantic annotation name in {:?}",
                        original_element
                    );
                    return Ok(None);
                }

                let payload_text = self.semantic_value_to_string(&parts[1]);
                let canonical = format!("@{}: {}", annotation_name, payload_text);
                let backend_valid = self.validate_semantic_annotation_backend(&canonical);
                if !backend_valid {
                    eprintln!(
                        "[mod.rs][parse_semantic_annotation_entry()] ⚠️ selected backend could not validate semantic annotation '{}'",
                        canonical
                    );
                }

                Ok(Some(SemanticAnnotation::Named {
                    name: annotation_name.clone(),
                    ast: self.parse_semantic_annotation_ast(
                        &annotation_name,
                        &payload_text,
                        backend_valid,
                    )?,
                }))
            }
            serde_json::Value::String(text) => {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    return Ok(None);
                }

                if let Some((name, payload)) =
                    self::semantic_directive_registry::extract_semantic_directive(trimmed)
                {
                    let backend_valid = self.validate_semantic_annotation_backend(trimmed);
                    if !backend_valid {
                        eprintln!(
                            "[mod.rs][parse_semantic_annotation_entry()] ⚠️ selected backend could not validate semantic annotation '{}'",
                            trimmed
                        );
                    }
                    return Ok(Some(SemanticAnnotation::Named {
                        name: name.clone(),
                        ast: self.parse_semantic_annotation_ast(&name, &payload, backend_valid)?,
                    }));
                }

                let ast = if self.config.bootstrap_mode {
                    let logger = runtime_logger("pipeline.semantic_annotation.bootstrap");
                    UnifiedSemanticAST::parse_bootstrap(trimmed, &logger).unwrap_or_else(|_| {
                        UnifiedSemanticAST::Raw {
                            content: trimmed.to_string(),
                        }
                    })
                } else {
                    // Only directive-shaped strings should go through generated semantic
                    // full-parse conversion in non-bootstrap mode.
                    if trimmed.starts_with('@') {
                        if let Some((name, ast)) =
                            self.parse_semantic_annotation_with_generated_parser(trimmed)?
                        {
                            return Ok(Some(SemanticAnnotation::Named { name, ast }));
                        }
                    }
                    // In non-bootstrap mode, do not apply bootstrap marker heuristics.
                    // Non-directive payload is intentionally preserved as raw content.
                    UnifiedSemanticAST::Raw {
                        content: trimmed.to_string(),
                    }
                };
                Ok(Some(SemanticAnnotation::Legacy(ast)))
            }
            _ => {
                let raw = self.semantic_value_to_string(payload);
                if raw.trim().is_empty() {
                    return Ok(None);
                }
                Ok(Some(SemanticAnnotation::Legacy(UnifiedSemanticAST::Raw {
                    content: raw,
                })))
            }
        }
    }

    fn parse_semantic_annotation_ast(
        &self,
        annotation_name: &str,
        payload: &str,
        backend_valid: bool,
    ) -> Result<UnifiedSemanticAST> {
        let normalized_name = annotation_name.trim().to_ascii_lowercase();
        let canonical = format!("@{}: {}", normalized_name, payload.trim());

        if backend_valid {
            if let Some((parsed_name, ast)) =
                self.parse_semantic_annotation_with_generated_parser(&canonical)?
            {
                if parsed_name != normalized_name {
                    return Err(anyhow::anyhow!(
                        "named semantic annotation parse produced mismatched name '{}' (expected '{}') for canonical '{}'",
                        parsed_name,
                        normalized_name,
                        canonical
                    ));
                }
                return Ok(ast);
            }
        }

        Ok(self.semantic_named_ast(&normalized_name, payload))
    }

    fn parse_semantic_annotation_with_generated_parser(
        &self,
        annotation_text: &str,
    ) -> Result<Option<(String, UnifiedSemanticAST)>> {
        if self.config.bootstrap_mode {
            return Ok(None);
        }

        #[cfg(feature = "generated_parsers")]
        {
            let logger = runtime_logger("pipeline.semantic_annotation.generated");
            let mut parser = Semantic_annotationParser::new(
                annotation_text,
                runtime_logger_box("pipeline.semantic_annotation.generated"),
            );
            let parse_tree = parser.parse_full_semantic_annotation().map_err(|err| {
                anyhow::anyhow!(
                    "generated semantic parser failed for '{}': {}",
                    annotation_text,
                    err
                )
            })?;
            let entry = UnifiedSemanticAST::parse_generated_semantic_annotation_entry(
                annotation_text,
                &parse_tree,
                &logger,
            )
            .map_err(|err| {
                anyhow::anyhow!(
                    "generated semantic tree -> typed AST failed for '{}': {}",
                    annotation_text,
                    err
                )
            })?;
            return Ok(Some(entry));
        }

        #[cfg(not(feature = "generated_parsers"))]
        {
            let _ = annotation_text;
            Ok(None)
        }
    }

    fn semantic_named_ast(&self, name: &str, payload: &str) -> UnifiedSemanticAST {
        UnifiedSemanticAST::from_named_payload(name, payload)
    }

    fn semantic_value_to_string(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(text) => text.clone(),
            _ => value.to_string(),
        }
    }

    fn validate_return_annotation_backend(&self, annotation_content: &str) -> bool {
        if self.config.bootstrap_mode {
            return true;
        }

        #[cfg(feature = "generated_parsers")]
        {
            let mut parser = Return_annotationParser::new(
                annotation_content,
                runtime_logger_box("pipeline.return_annotation.backend_validate"),
            );
            return parser.parse_full_return_annotation().is_ok();
        }

        #[cfg(not(feature = "generated_parsers"))]
        {
            let _ = annotation_content;
            eprintln!(
                "[mod.rs][validate_return_annotation_backend()] ⚠️ generated parser backend unavailable (build with --features generated_parsers)"
            );
            true
        }
    }

    fn validate_semantic_annotation_backend(&self, annotation_text: &str) -> bool {
        if self.config.bootstrap_mode {
            return true;
        }

        #[cfg(feature = "generated_parsers")]
        {
            let mut parser = Semantic_annotationParser::new(
                annotation_text,
                runtime_logger_box("pipeline.semantic_annotation.backend_validate"),
            );
            return parser.parse_full_semantic_annotation().is_ok();
        }

        #[cfg(not(feature = "generated_parsers"))]
        {
            let _ = annotation_text;
            eprintln!(
                "[mod.rs][validate_semantic_annotation_backend()] ⚠️ generated parser backend unavailable (build with --features generated_parsers)"
            );
            true
        }
    }

    fn step1_normalize_raw_elements(
        &self,
        content: &[serde_json::Value],
    ) -> Result<Vec<RawRuleElement>> {
        eprintln!("[mod.rs][step1_normalize_raw_elements()] 🔎 Start normalization");
        let mut normalized = Vec::new();

        for (elem_idx, item) in content.iter().enumerate() {
            eprintln!("        🔧  Element {}/{}", elem_idx + 1, content.len());
            eprintln!("            Raw data: {:?}", item);
            eprintln!("            File: {}:{}", file!(), line!());
            if let Some(parsed) = self.parse_raw_element(item)? {
                eprintln!(
                    "            ✅  Normalized element kind: {}",
                    self.raw_element_kind(&parsed)
                );
                normalized.push(parsed);
            } else {
                eprintln!("            ⚠️   Element skipped (return annotation or unknown type)");
            }
            eprintln!();
        }

        Ok(normalized)
    }

    fn parse_raw_element(&self, element: &serde_json::Value) -> Result<Option<RawRuleElement>> {
        let Some(arr) = element.as_array() else {
            eprintln!("            ❌  [mod.rs][parse_raw_element()] Element is not array");
            return Ok(None);
        };

        if arr.len() < 2 {
            eprintln!(
                "            ❌  [mod.rs][parse_raw_element()] Element array too short: {}",
                arr.len()
            );
            return Ok(None);
        }

        let (Some(elem_type), Some(elem_value)) = (arr[0].as_str(), arr[1].as_str()) else {
            eprintln!(
                "            ❌  [mod.rs][parse_raw_element()] Invalid element structure: {:?}",
                arr
            );
            return Ok(None);
        };

        eprintln!("            🔍  \x1b[34mELEMENT ANALYSIS\x1b[0m");
        eprintln!(
            "                Type: '{}' | Value: '{}'",
            elem_type, elem_value
        );
        eprintln!("                File: {}:{}", file!(), line!());

        let atom_from = |token_type: &str, token_value: &str| -> RawRuleElement {
            RawRuleElement::Atom(ASTNode::Atom {
                value: ASTValue::Token(vec![
                    TokenValue::String(token_type.to_string()),
                    TokenValue::String(token_value.to_string()),
                ]),
            })
        };

        let parsed = match elem_type {
            "rule_reference" => {
                eprintln!(
                    "                📋  RULE REFERENCE - Creating call to rule '{}'",
                    elem_value
                );
                Some(atom_from("rule_reference", elem_value))
            }
            "quoted_string" => {
                eprintln!(
                    "                💬  \x1b[32mSTRING TERMINAL\x1b[0m - Creating matcher for '{}'",
                    elem_value
                );
                Some(atom_from("quoted_string", elem_value))
            }
            "regex" => {
                eprintln!(
                    "                🔤  \x1b[32mREGEX PATTERN\x1b[0m - Creating regex matcher for '{}'",
                    elem_value
                );
                Some(atom_from("regex", elem_value))
            }
            "group_open" => {
                eprintln!(
                    "                🔓  \x1b[32mGROUP OPEN\x1b[0m - Start grouped expression"
                );
                Some(RawRuleElement::GroupOpen)
            }
            "group_close" => {
                eprintln!(
                    "                🔒  \x1b[32mGROUP CLOSE\x1b[0m - End grouped expression"
                );
                Some(RawRuleElement::GroupClose)
            }
            "quantifier" => {
                eprintln!(
                    "                🔢  \x1b[32mQUANTIFIER\x1b[0m - Binding quantifier '{}'",
                    elem_value
                );
                Some(RawRuleElement::Quantifier(elem_value.to_string()))
            }
            "operator" => match elem_value {
                "|" => {
                    eprintln!(
                        "                🔀  \x1b[32mALTERNATIVE OPERATOR\x1b[0m (|) - Split branches"
                    );
                    Some(RawRuleElement::OrOperator)
                }
                "?" | "*" | "+" => {
                    eprintln!(
                        "                🔁  \x1b[32mQUANTIFIER OPERATOR\x1b[0m '{}' - Bind to previous primary",
                        elem_value
                    );
                    Some(RawRuleElement::Quantifier(elem_value.to_string()))
                }
                "&" => {
                    eprintln!(
                        "                👀  \x1b[32mPOSITIVE LOOKAHEAD\x1b[0m (&) - Assert next primary without consuming"
                    );
                    Some(RawRuleElement::Lookahead(true))
                }
                "!" => {
                    eprintln!(
                        "                🚫  \x1b[32mNEGATIVE LOOKAHEAD\x1b[0m (!) - Reject matching next primary without consuming"
                    );
                    Some(RawRuleElement::Lookahead(false))
                }
                _ => {
                    eprintln!(
                        "                ⚙️   \x1b[33mNON-STRUCTURAL OPERATOR\x1b[0m '{}' - treat as terminal",
                        elem_value
                    );
                    Some(atom_from("quoted_string", elem_value))
                }
            },
            "number" => {
                eprintln!(
                    "                🔢  \x1b[32mNUMBER\x1b[0m - treat as terminal '{}'",
                    elem_value
                );
                Some(atom_from("number", elem_value))
            }
            "probability" => {
                eprintln!(
                    "                🎲  \x1b[32mPROBABILITY\x1b[0m - treat as terminal '{}'",
                    elem_value
                );
                Some(atom_from("probability", elem_value))
            }
            "include_dir" => {
                eprintln!(
                    "                📁  \x1b[32mINCLUDE DIR\x1b[0m - preserve '{}' token",
                    elem_value
                );
                Some(atom_from("include_dir", elem_value))
            }
            "include_file" => {
                eprintln!(
                    "                📄  \x1b[32mINCLUDE FILE\x1b[0m - preserve '{}' token",
                    elem_value
                );
                Some(atom_from("include_file", elem_value))
            }
            "rule" => {
                eprintln!(
                    "                📝  \x1b[33mRULE TOKEN\x1b[0m - preserve '{}' token",
                    elem_value
                );
                Some(atom_from("rule", elem_value))
            }
            "return_scalar" | "return_array" | "return_object" => {
                eprintln!(
                    "                🔙  \x1b[33mRETURN ANNOTATION\x1b[0m '{}' - skipped in syntax tree stage",
                    elem_type
                );
                None
            }
            _ => {
                eprintln!(
                    "                ❓  \x1b[33mUNKNOWN ELEMENT TYPE\x1b[0m '{}' - skipping",
                    elem_type
                );
                None
            }
        };

        Ok(parsed)
    }

    fn step2_group_by_or(&self, elements: &[RawRuleElement]) -> Vec<Vec<RawRuleElement>> {
        eprintln!("[mod.rs][step2_group_by_or()] 🔀 Splitting top-level alternatives");
        let mut branches: Vec<Vec<RawRuleElement>> = Vec::new();
        let mut current: Vec<RawRuleElement> = Vec::new();
        let mut group_depth = 0usize;

        for elem in elements {
            match elem {
                RawRuleElement::GroupOpen => {
                    group_depth += 1;
                    current.push(elem.clone());
                }
                RawRuleElement::GroupClose => {
                    if group_depth > 0 {
                        group_depth -= 1;
                    } else {
                        eprintln!(
                            "  ⚠️ [mod.rs][step2_group_by_or()] unmatched group_close at top-level"
                        );
                    }
                    current.push(elem.clone());
                }
                RawRuleElement::OrOperator if group_depth == 0 => {
                    branches.push(current);
                    current = Vec::new();
                }
                _ => current.push(elem.clone()),
            }
        }

        branches.push(current);

        if group_depth != 0 {
            eprintln!(
                "  ⚠️ [mod.rs][step2_group_by_or()] unbalanced parentheses depth={}",
                group_depth
            );
        }

        branches
    }

    fn step2_5_handle_parentheses(&self, branch: &[RawRuleElement]) -> Result<Vec<ASTNode>> {
        eprintln!(
            "[mod.rs][step2_5_handle_parentheses()] 🧩 Parsing grouped branch of {} elements",
            branch.len()
        );
        let mut result = Vec::new();
        let mut idx = 0usize;

        while idx < branch.len() {
            if let Some(primary) = self.parse_branch_primary(branch, &mut idx)? {
                result.push(primary);
            }
        }

        Ok(result)
    }

    fn parse_branch_primary(
        &self,
        branch: &[RawRuleElement],
        idx: &mut usize,
    ) -> Result<Option<ASTNode>> {
        if *idx >= branch.len() {
            return Ok(None);
        }

        let mut lookahead_polarity = Vec::new();
        while *idx < branch.len() {
            match &branch[*idx] {
                RawRuleElement::Lookahead(positive) => {
                    lookahead_polarity.push(*positive);
                    *idx += 1;
                }
                _ => break,
            }
        }

        if *idx >= branch.len() {
            return Ok(None);
        }

        let mut primary = match &branch[*idx] {
            RawRuleElement::Atom(node) => {
                *idx += 1;
                node.clone()
            }
            RawRuleElement::GroupOpen => {
                let (inner, next_idx) = self.extract_group_contents(branch, *idx)?;
                *idx = next_idx;
                self.build_ast_from_elements(&inner)?
            }
            RawRuleElement::GroupClose => {
                eprintln!(
                    "  ⚠️ [mod.rs][parse_branch_primary()] unexpected group_close at idx={}",
                    *idx
                );
                *idx += 1;
                return Ok(None);
            }
            RawRuleElement::OrOperator => {
                eprintln!(
                    "  ⚠️ [mod.rs][parse_branch_primary()] unexpected top-level OR token inside branch at idx={}",
                    *idx
                );
                *idx += 1;
                return Ok(None);
            }
            RawRuleElement::Quantifier(q) => {
                eprintln!(
                    "  ⚠️ [mod.rs][parse_branch_primary()] dangling quantifier '{}' at idx={} (ignored)",
                    q, *idx
                );
                *idx += 1;
                return Ok(None);
            }
            RawRuleElement::Lookahead(_) => unreachable!("lookahead prefixes already consumed"),
        };

        primary = self.step4_handle_quantifiers(primary, branch, idx);
        for positive in lookahead_polarity.into_iter().rev() {
            primary = ASTNode::Lookahead {
                element: Box::new(primary),
                positive,
            };
        }

        Ok(Some(primary))
    }

    fn extract_group_contents(
        &self,
        branch: &[RawRuleElement],
        open_idx: usize,
    ) -> Result<(Vec<RawRuleElement>, usize)> {
        let mut depth = 1usize;
        let mut idx = open_idx + 1;
        let mut inner = Vec::new();

        while idx < branch.len() {
            match &branch[idx] {
                RawRuleElement::GroupOpen => {
                    depth += 1;
                    inner.push(branch[idx].clone());
                }
                RawRuleElement::GroupClose => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok((inner, idx + 1));
                    }
                    inner.push(branch[idx].clone());
                }
                _ => inner.push(branch[idx].clone()),
            }
            idx += 1;
        }

        Err(anyhow::anyhow!(
            "[mod.rs][extract_group_contents()] Unclosed group starting at index {}",
            open_idx
        ))
    }

    fn step3_parse_sequences(&self, elements: Vec<ASTNode>) -> ASTNode {
        eprintln!(
            "[mod.rs][step3_parse_sequences()] 🧱 Building sequence from {} elements",
            elements.len()
        );
        match elements.len() {
            0 => ASTNode::Sequence { elements: vec![] },
            1 => elements.into_iter().next().unwrap(),
            _ => ASTNode::Sequence { elements },
        }
    }

    fn step4_handle_quantifiers(
        &self,
        mut node: ASTNode,
        branch: &[RawRuleElement],
        idx: &mut usize,
    ) -> ASTNode {
        while *idx < branch.len() {
            match &branch[*idx] {
                RawRuleElement::Quantifier(q) => {
                    eprintln!(
                        "[mod.rs][step4_handle_quantifiers()] 🔁 Apply quantifier '{}' at idx={}",
                        q, *idx
                    );
                    node = ASTNode::Quantified {
                        element: Box::new(node),
                        quantifier: q.clone(),
                    };
                    *idx += 1;
                }
                _ => break,
            }
        }

        node
    }

    fn step5_build_tree_structure(&self, branches: Vec<ASTNode>) -> ASTNode {
        eprintln!(
            "[mod.rs][step5_build_tree_structure()] 🌳 Final tree from {} branches",
            branches.len()
        );
        if branches.len() <= 1 {
            branches
                .into_iter()
                .next()
                .unwrap_or(ASTNode::Sequence { elements: vec![] })
        } else {
            ASTNode::Or {
                alternatives: branches,
            }
        }
    }

    fn build_ast_from_elements(&self, elements: &[RawRuleElement]) -> Result<ASTNode> {
        let branches = self.step2_group_by_or(elements);
        let mut branch_asts = Vec::with_capacity(branches.len());
        for branch in branches {
            let seq_elements = self.step2_5_handle_parentheses(&branch)?;
            branch_asts.push(self.step3_parse_sequences(seq_elements));
        }
        Ok(self.step5_build_tree_structure(branch_asts))
    }

    fn raw_element_kind(&self, elem: &RawRuleElement) -> &'static str {
        match elem {
            RawRuleElement::Atom(_) => "atom",
            RawRuleElement::OrOperator => "or_operator",
            RawRuleElement::GroupOpen => "group_open",
            RawRuleElement::GroupClose => "group_close",
            RawRuleElement::Quantifier(_) => "quantifier",
            RawRuleElement::Lookahead(true) => "positive_lookahead",
            RawRuleElement::Lookahead(false) => "negative_lookahead",
        }
    }

    fn parse_single_element(&self, element: &serde_json::Value) -> Result<Option<ASTNode>> {
        if let Some(arr) = element.as_array() {
            if arr.len() >= 2 {
                if let (Some(elem_type), Some(elem_value)) = (arr[0].as_str(), arr[1].as_str()) {
                    eprintln!("            🔍  \x1b[34mELEMENT ANALYSIS\x1b[0m");
                    eprintln!(
                        "                Type: '{}' | Value: '{}'",
                        elem_type, elem_value
                    );
                    eprintln!("                File: {}:{}", file!(), line!());

                    match elem_type {
                        "rule" => {
                            eprintln!(
                                "                📝  \x1b[32mRULE DECLARATION\x1b[0m - Defining rule '{}'",
                                elem_value
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            Ok(Some(ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("rule".to_string()),
                                    TokenValue::String(elem_value.to_string()),
                                ]),
                            }))
                        }
                        "rule_reference" => {
                            eprintln!(
                                "                📋  RULE REFERENCE - Creating call to rule '{}'",
                                elem_value
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            Ok(Some(ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("rule_reference".to_string()),
                                    TokenValue::String(elem_value.to_string()),
                                ]),
                            }))
                        }
                        "quoted_string" => {
                            eprintln!(
                                "                💬  \x1b[32mSTRING TERMINAL\x1b[0m - Creating matcher for '{}'",
                                elem_value
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            Ok(Some(ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("quoted_string".to_string()),
                                    TokenValue::String(elem_value.to_string()),
                                ]),
                            }))
                        }
                        "regex" => {
                            eprintln!(
                                "                🔤  \x1b[32mREGEX PATTERN\x1b[0m - Creating regex matcher for '{}'",
                                elem_value
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            Ok(Some(ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("regex".to_string()),
                                    TokenValue::String(elem_value.to_string()),
                                ]),
                            }))
                        }
                        "group_open" => {
                            eprintln!(
                                "                🔓  \x1b[32mGROUP OPEN\x1b[0m - Starting group '{}'",
                                elem_value
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            Ok(Some(ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("group_open".to_string()),
                                    TokenValue::String(elem_value.to_string()),
                                ]),
                            }))
                        }
                        "group_close" => {
                            eprintln!(
                                "                🔒  \x1b[32mGROUP CLOSE\x1b[0m - Ending group '{}'",
                                elem_value
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            Ok(Some(ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("group_close".to_string()),
                                    TokenValue::String(elem_value.to_string()),
                                ]),
                            }))
                        }
                        "quantifier" => {
                            eprintln!(
                                "                🔢  \x1b[32mEXPLICIT QUANTIFIER\x1b[0m - Processing '{}'",
                                elem_value
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            Ok(Some(ASTNode::Quantified {
                                element: Box::new(ASTNode::Sequence { elements: vec![] }), // Placeholder
                                quantifier: elem_value.to_string(),
                            }))
                        }
                        "number" => {
                            eprintln!(
                                "                🔢  \x1b[32mNUMERIC LITERAL\x1b[0m - Processing '{}'",
                                elem_value
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            Ok(Some(ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("number".to_string()),
                                    TokenValue::String(elem_value.to_string()),
                                ]),
                            }))
                        }
                        "probability" => {
                            eprintln!(
                                "                🎲  \x1b[32mPROBABILITY\x1b[0m - Processing '{}'",
                                elem_value
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            Ok(Some(ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("probability".to_string()),
                                    TokenValue::String(elem_value.to_string()),
                                ]),
                            }))
                        }
                        "include_dir" => {
                            eprintln!(
                                "                📁  \x1b[32mINCLUDE DIRECTORY\x1b[0m - Processing '{}'",
                                elem_value
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            Ok(Some(ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("include_dir".to_string()),
                                    TokenValue::String(elem_value.to_string()),
                                ]),
                            }))
                        }
                        "include_file" => {
                            eprintln!(
                                "                📄  \x1b[32mINCLUDE FILE\x1b[0m - Processing '{}'",
                                elem_value
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            Ok(Some(ASTNode::Atom {
                                value: ASTValue::Token(vec![
                                    TokenValue::String("include_file".to_string()),
                                    TokenValue::String(elem_value.to_string()),
                                ]),
                            }))
                        }
                        "operator" => {
                            eprintln!(
                                "                🔄  \x1b[33mQUANTIFIER OPERATOR\x1b[0m - Processing '{}'",
                                elem_value
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            // Handle quantifiers
                            match elem_value {
                                "?" => {
                                    eprintln!(
                                        "                    ❓  \x1b[32mOPTIONAL QUANTIFIER\x1b[0m (?) - Zero or one occurrence"
                                    );
                                    eprintln!("                    File: {}:{}", file!(), line!());
                                    Ok(Some(ASTNode::Quantified {
                                        element: Box::new(ASTNode::Sequence { elements: vec![] }), // Placeholder
                                        quantifier: "?".to_string(),
                                    }))
                                }
                                "*" => {
                                    eprintln!(
                                        "                    🔁  \x1b[32mZERO-OR-MORE QUANTIFIER\x1b[0m (*) - Zero or more occurrences"
                                    );
                                    eprintln!("                    File: {}:{}", file!(), line!());
                                    Ok(Some(ASTNode::Quantified {
                                        element: Box::new(ASTNode::Sequence { elements: vec![] }), // Placeholder
                                        quantifier: "*".to_string(),
                                    }))
                                }
                                "+" => {
                                    eprintln!(
                                        "                    ➕  \x1b[32mONE-OR-MORE QUANTIFIER\x1b[0m (+) - One or more occurrences"
                                    );
                                    eprintln!("                    File: {}:{}", file!(), line!());
                                    Ok(Some(ASTNode::Quantified {
                                        element: Box::new(ASTNode::Sequence { elements: vec![] }), // Placeholder
                                        quantifier: "+".to_string(),
                                    }))
                                }
                                "&" => {
                                    eprintln!(
                                        "                    👀  \x1b[32mPOSITIVE LOOKAHEAD\x1b[0m (&) - Placeholder lookahead node"
                                    );
                                    eprintln!("                    File: {}:{}", file!(), line!());
                                    Ok(Some(ASTNode::Lookahead {
                                        element: Box::new(ASTNode::Sequence { elements: vec![] }),
                                        positive: true,
                                    }))
                                }
                                "!" => {
                                    eprintln!(
                                        "                    🚫  \x1b[32mNEGATIVE LOOKAHEAD\x1b[0m (!) - Placeholder lookahead node"
                                    );
                                    eprintln!("                    File: {}:{}", file!(), line!());
                                    Ok(Some(ASTNode::Lookahead {
                                        element: Box::new(ASTNode::Sequence { elements: vec![] }),
                                        positive: false,
                                    }))
                                }
                                "|" => {
                                    eprintln!(
                                        "                    🔀  \x1b[32mALTERNATIVE OPERATOR\x1b[0m (|) - Creating choice between alternatives"
                                    );
                                    eprintln!("                    File: {}:{}", file!(), line!());
                                    Ok(Some(ASTNode::Atom {
                                        value: ASTValue::Token(vec![
                                            TokenValue::String("operator".to_string()),
                                            TokenValue::String("|".to_string()),
                                        ]),
                                    }))
                                }
                                _ => {
                                    eprintln!(
                                        "                    ⚠️   \x1b[33mUNKNOWN OPERATOR\x1b[0m '{}' - Skipping",
                                        elem_value
                                    );
                                    eprintln!("                    File: {}:{}", file!(), line!());
                                    Ok(None) // Skip unknown operators
                                }
                            }
                        }
                        "return_scalar" | "return_array" | "return_object" => {
                            eprintln!(
                                "                🔙  \x1b[33mRETURN ANNOTATION\x1b[0m '{}' - Skipping (semantic annotation)",
                                elem_type
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            // Skip return annotations for now
                            Ok(None)
                        }
                        _ => {
                            eprintln!(
                                "                ❓  \x1b[33mUNKNOWN ELEMENT TYPE\x1b[0m '{}' - Skipping",
                                elem_type
                            );
                            eprintln!("                File: {}:{}", file!(), line!());
                            Ok(None) // Skip unknown element types
                        }
                    }
                } else {
                    eprintln!("            ❌  \x1b[31mERROR: Invalid element structure\x1b[0m");
                    eprintln!(
                        "                Expected [string, string] but got: [{:?}, {:?}]",
                        arr[0], arr[1]
                    );
                    eprintln!("                File: {}:{}", file!(), line!());
                    Ok(None)
                }
            } else {
                eprintln!("            ❌  \x1b[31mERROR: Element array too short\x1b[0m");
                eprintln!(
                    "                Need at least 2 elements, got {}",
                    arr.len()
                );
                eprintln!("                File: {}:{}", file!(), line!());
                Ok(None)
            }
        } else {
            eprintln!("            ❌  \x1b[31mERROR: Element is not an array\x1b[0m");
            eprintln!(
                "                Type: {} | Value: {:?}",
                std::any::type_name::<serde_json::Value>(),
                element
            );
            eprintln!("                File: {}:{}", file!(), line!());
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn transform_from_raw_ast_preserves_return_and_semantic_annotations() {
        let pipeline = RustASTPipeline::new(PipelineConfig::default());
        let raw_ast_data = vec![json!([
            ["rule", "expr"],
            ["semantic_annotation", ["priority", "[9, 1]"]],
            ["rule_reference", "lhs"],
            ["operator", "|"],
            ["rule_reference", "rhs"],
            ["return_scalar", "$1"]
        ])];

        let (_grammar_tree, _rule_order, annotations) = pipeline
            .transform_from_raw_ast(&raw_ast_data)
            .expect("raw_ast transformation should succeed");
        let annotations = annotations.expect("annotations should be preserved");

        let branch_annotations = annotations
            .branch_return_annotations
            .get("expr")
            .expect("rule return annotations should exist");
        assert_eq!(branch_annotations.len(), 2);
        assert!(branch_annotations[0].is_none());
        let return_annotation = branch_annotations[1]
            .as_ref()
            .expect("second branch should carry return annotation");
        assert_eq!(return_annotation.annotation_type, "return_scalar");
        assert_eq!(return_annotation.annotation_content, "$1");
        assert!(return_annotation.parsed_ast.is_some());

        let semantic_annotations = annotations
            .semantic_annotations
            .get("expr")
            .expect("rule semantic annotations should exist");
        assert_eq!(semantic_annotations.len(), 1);
        match &semantic_annotations[0] {
            SemanticAnnotation::Named { name, ast } => {
                assert_eq!(name, "priority");
                assert!(matches!(
                    ast,
                    UnifiedSemanticAST::Structured { canonical, .. } if canonical == "[9, 1]"
                ));
            }
            _ => panic!("semantic annotation should be captured as named directive"),
        }

        let branch_semantic_annotations = annotations
            .branch_semantic_annotations
            .get("expr")
            .expect("rule branch semantic annotations should exist");
        assert_eq!(branch_semantic_annotations.len(), 2);
        assert!(branch_semantic_annotations[0].is_empty());
        assert!(branch_semantic_annotations[1].is_empty());
    }

    #[test]
    fn transform_from_raw_ast_preserves_branch_semantic_annotations() {
        let pipeline = RustASTPipeline::new(PipelineConfig::default());
        let raw_ast_data = vec![json!([
            ["rule", "expr"],
            ["rule_reference", "lhs"],
            ["operator", "|"],
            [
                "semantic_annotation_inline",
                [
                    "predicate",
                    "{ name: has_fact, args: [type_name, $rhs], phase: branch, view: raw }"
                ]
            ],
            ["rule_reference", "rhs"]
        ])];

        let (_grammar_tree, _rule_order, annotations) = pipeline
            .transform_from_raw_ast(&raw_ast_data)
            .expect("raw_ast transformation should succeed");
        let annotations = annotations.expect("annotations should be preserved");

        let branch_semantic_annotations = annotations
            .branch_semantic_annotations
            .get("expr")
            .expect("branch semantic annotations should exist");
        assert_eq!(branch_semantic_annotations.len(), 2);
        assert!(branch_semantic_annotations[0].is_empty());
        assert_eq!(branch_semantic_annotations[1].len(), 1);
        match &branch_semantic_annotations[1][0] {
            SemanticAnnotation::Named { name, ast } => {
                assert_eq!(name, "predicate");
                assert!(matches!(
                    ast,
                    UnifiedSemanticAST::Structured { canonical, .. }
                        if canonical
                            == "{ name: has_fact, args: [type_name, $rhs], phase: branch, view: raw }"
                ));
            }
            other => panic!(
                "branch semantic annotation should be captured as named directive, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn transform_from_raw_ast_preserves_mid_sequence_semantic_annotations() {
        let pipeline = RustASTPipeline::new(PipelineConfig::default());
        let raw_ast_data = vec![json!([
            ["rule", "expr"],
            ["rule_reference", "alpha"],
            [
                "semantic_annotation_mid_sequence",
                [
                    "predicate",
                    "{ name: has_fact, args: [type_name, $beta], phase: branch, view: raw }"
                ]
            ],
            ["rule_reference", "beta"]
        ])];

        let (_grammar_tree, _rule_order, annotations) = pipeline
            .transform_from_raw_ast(&raw_ast_data)
            .expect("raw_ast transformation should succeed");
        let annotations = annotations.expect("annotations should be preserved");

        assert!(
            annotations
                .branch_semantic_annotations
                .get("expr")
                .is_none(),
            "mid-sequence-only annotations should not create a separate branch-local annotation entry"
        );

        let mid_sequence_annotations = annotations
            .branch_mid_sequence_semantic_annotations
            .get("expr")
            .expect("mid-sequence semantic annotations should exist");
        assert_eq!(mid_sequence_annotations.len(), 1);
        assert_eq!(mid_sequence_annotations[0].len(), 1);
        let entry = &mid_sequence_annotations[0][0];
        assert_eq!(entry.syntax_position, 1);
        assert_eq!(entry.group_depth, 0);
        match &entry.annotation {
            SemanticAnnotation::Named { name, ast } => {
                assert_eq!(name, "predicate");
                assert!(matches!(
                    ast,
                    UnifiedSemanticAST::Structured { canonical, .. }
                        if canonical
                            == "{ name: has_fact, args: [type_name, $beta], phase: branch, view: raw }"
                ));
            }
            other => panic!(
                "mid-sequence semantic annotation should be captured as named directive, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn transform_from_raw_ast_promotes_transform_semantic_payload() {
        let pipeline = RustASTPipeline::new(PipelineConfig::default());
        let raw_ast_data = vec![json!([
            ["rule", "int_rule"],
            [
                "semantic_annotation",
                ["transform", "str::parse::<i64>().unwrap_or(0)"]
            ],
            ["regex", "[-+]?[0-9]+"]
        ])];

        let (_grammar_tree, _rule_order, annotations) = pipeline
            .transform_from_raw_ast(&raw_ast_data)
            .expect("raw_ast transformation should succeed");
        let annotations = annotations.expect("annotations should be preserved");

        let semantic_annotations = annotations
            .semantic_annotations
            .get("int_rule")
            .expect("semantic annotation should be present for int_rule");
        assert_eq!(semantic_annotations.len(), 1);
        match &semantic_annotations[0] {
            SemanticAnnotation::Named { name, ast } => {
                assert_eq!(name, "transform");
                assert!(matches!(
                    ast,
                    UnifiedSemanticAST::TransformExpr { expression }
                        if expression == "str::parse::<i64>().unwrap_or(0)"
                ));
            }
            _ => panic!("transform semantic annotation should be named"),
        }
    }

    #[test]
    fn transform_from_raw_ast_nonbootstrap_legacy_semantic_does_not_use_marker_transform_fallback()
    {
        let pipeline = RustASTPipeline::new(PipelineConfig::default());
        let raw_ast_data = vec![json!([
            ["rule", "legacy_sem_rule"],
            ["semantic_annotation", "str::parse::<i64>().unwrap_or(0)"],
            ["regex", "[-+]?[0-9]+"]
        ])];

        let (_grammar_tree, _rule_order, annotations) = pipeline
            .transform_from_raw_ast(&raw_ast_data)
            .expect("raw_ast transformation should succeed");
        let annotations = annotations.expect("annotations should be preserved");

        let semantic_annotations = annotations
            .semantic_annotations
            .get("legacy_sem_rule")
            .expect("legacy semantic annotation should be present");
        assert_eq!(semantic_annotations.len(), 1);
        match &semantic_annotations[0] {
            SemanticAnnotation::Legacy(UnifiedSemanticAST::Raw { content }) => {
                assert_eq!(content, "str::parse::<i64>().unwrap_or(0)")
            }
            other => panic!(
                "non-bootstrap legacy semantic should stay raw and not transform fallback: {:?}",
                other
            ),
        }
    }

    #[cfg(feature = "generated_parsers")]
    #[test]
    fn transform_from_raw_ast_nonbootstrap_named_semantic_preserves_payload_when_backend_rejects() {
        let pipeline = RustASTPipeline::new(PipelineConfig::default());
        let raw_ast_data = vec![json!([
            ["rule", "bad_sem_rule"],
            ["semantic_annotation", ["priority", "\"unterminated"]],
            ["regex", "[0-9]+"]
        ])];

        let (_grammar_tree, _rule_order, annotations) = pipeline
            .transform_from_raw_ast(&raw_ast_data)
            .expect("invalid named semantic payload should remain preserved when backend rejects");
        let annotations = annotations.expect("annotations should be preserved");
        let semantic_annotations = annotations
            .semantic_annotations
            .get("bad_sem_rule")
            .expect("semantic annotation should be present");
        assert_eq!(semantic_annotations.len(), 1);
        match &semantic_annotations[0] {
            SemanticAnnotation::Named { name, ast } => {
                assert_eq!(name, "priority");
                assert!(matches!(
                    ast,
                    UnifiedSemanticAST::Raw { content } if content == "\"unterminated"
                ));
            }
            other => panic!(
                "expected named semantic annotation with preserved raw payload, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn transform_from_raw_ast_merges_duplicate_rule_heads_into_one_rule() {
        let pipeline = RustASTPipeline::new(PipelineConfig::default());
        let raw_ast_data = vec![
            json!([["rule", "value"], ["quoted_string", "a"]]),
            json!([["rule", "value"], ["quoted_string", "b"]]),
        ];

        let (grammar_tree, rule_order, _annotations) = pipeline
            .transform_from_raw_ast(&raw_ast_data)
            .expect("raw_ast transformation should succeed");

        assert_eq!(rule_order, vec!["value".to_string()]);
        let value_rule = grammar_tree.get("value").expect("merged value rule");
        match value_rule {
            ASTNode::Or { alternatives } => assert_eq!(alternatives.len(), 2),
            other => panic!("expected merged alternation, got {:?}", other),
        }
    }

    #[test]
    fn transform_from_raw_ast_preserves_lookahead_prefixes() {
        let pipeline = RustASTPipeline::new(PipelineConfig::default());
        let raw_ast_data = vec![json!([
            ["rule", "ident_like"],
            ["operator", "!"],
            ["rule_reference", "kw_parameter"],
            ["rule_reference", "identifier"]
        ])];

        let (grammar_tree, _rule_order, _annotations) = pipeline
            .transform_from_raw_ast(&raw_ast_data)
            .expect("raw_ast transformation should succeed");

        let ident_like = grammar_tree.get("ident_like").expect("ident_like rule");
        match ident_like {
            ASTNode::Sequence { elements } => {
                assert_eq!(elements.len(), 2);
                assert!(matches!(
                    &elements[0],
                    ASTNode::Lookahead {
                        positive: false,
                        ..
                    }
                ));
                assert!(matches!(&elements[1], ASTNode::Atom { .. }));
            }
            other => panic!("expected lookahead sequence, got {:?}", other),
        }
    }
}

pub mod annotation_validator;
pub mod ast_based_generator;
pub mod ast_code_generator;
pub mod ast_generator_direct;
pub mod ast_return_transform;
pub mod grouped_quantifier_parser;
pub mod library;
pub mod mutual_recursion_handler;
pub mod parser_hooks;
pub mod return_annotation_handler;
pub mod semantic_directive_registry;
pub mod semantic_runtime;
pub mod semantic_transform;
pub mod stimuli_generator;
pub mod unified_return_ast;
pub mod unified_semantic_ast;

// Re-export key types
pub use annotation_validator::{
    AnnotationDiagnostic, AnnotationKind, AnnotationSeverity, AnnotationValidationReport,
    AnnotationValidator, AnnotationValidatorConfig,
};
pub use semantic_directive_registry::{
    SemanticAssociativity, SemanticBranchPolicy, SemanticDeterministicGroupHint,
    SemanticDirectiveCapability, SemanticDirectiveSpec, SemanticTokenClass,
    SemanticValueConstraints, UnknownSemanticDirectivePolicy, extract_semantic_directive,
    extract_semantic_directive_name, normalize_semantic_scalar, parse_semantic_bool,
    parse_semantic_branch_priorities, parse_semantic_charset, parse_semantic_constraint_expression,
    parse_semantic_coverage_target_weight, parse_semantic_deterministic_group,
    parse_semantic_float_list, parse_semantic_group_label, parse_semantic_implication,
    parse_semantic_len_bounds, parse_semantic_nonnegative_usize, parse_semantic_numeric_bounds,
    parse_semantic_numeric_list, parse_semantic_pattern, parse_semantic_reference_list,
    parse_semantic_string_list, parse_semantic_token_class, semantic_directive_spec,
};
pub use semantic_runtime::{
    CompiledSemanticRuntimeAnnotations, FactKindDecl, SemanticCloseScopeSpec, SemanticFactRecord,
    SemanticFactSpec, SemanticLibraryExportSpec, SemanticLibraryImportSpec,
    SemanticPredicateContentView, SemanticPredicatePhase, SemanticPredicateSpec,
    SemanticRuntimeCheckpoint, SemanticRuntimeDirective, SemanticRuntimeState,
    SemanticRuntimeTransaction, SemanticRuntimeValue, SemanticScopeFrame, SemanticScopeKind,
    SemanticScopeSpec, compile_rule_semantic_runtime_directives,
    compile_semantic_runtime_annotations, parse_semantic_runtime_directive,
    parse_semantic_runtime_directives,
};
pub use semantic_transform::{
    CanonicalSemanticTransform, parse_canonical_transform_expression, stimuli_hint_for_target_type,
};
pub use parser_hooks::{ParserHookRegistry, ParserHooks, ParserImplContext};
pub use unified_return_ast::{ExtractionTarget, UnifiedReturnAST};
pub use unified_semantic_ast::{UnifiedSemanticAST, UnifiedSemanticProperty, UnifiedSemanticValue};
