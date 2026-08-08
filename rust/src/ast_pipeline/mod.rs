use anyhow::{Result, anyhow};
use serde;

pub use self::pgen_value::PgenValue;
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

/// DIAG-SEVERITY.2 (PGEN-DIAG-SEVERITY-0002): a SEVERITY dimension ORTHOGONAL to the
/// `TraceLevel` verbosity scale. A diagnostic of severity ≥ Warning is emitted
/// UNCONDITIONALLY — it is NEVER suppressed by `trace_verbosity`, because masking a
/// warning/error/fatal behind a verbosity level is a silent failure (this is exactly
/// what hid the SV depth-exceeded error for the whole `.7.2` campaign). Trace verbosity
/// (`TraceLevel`) governs INFORMATIONAL output only. See docs/tasks/DIAG-SEVERITY.md and
/// the standing principle [[feedback_severity_never_gated_by_verbosity]].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Severity {
    Warning = 1,
    Error = 2,
    Fatal = 3,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Warning => "WARN",
            Self::Error => "ERROR",
            Self::Fatal => "FATAL",
        }
    }

    fn emoji(self) -> &'static str {
        match self {
            Self::Warning => "⚠️",
            Self::Error => "⛔",
            Self::Fatal => "💀",
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

/// Process-once cache of the `PGEN_REPORT_MEMO_STATS` diagnostic switch.
///
/// RGX-0078.5.i.7 `P-env`: every generated `parse()` consulted this flag via
/// `std::env::var("PGEN_REPORT_MEMO_STATS")` on the hot path — twice per parse
/// (the `bare_parse` observability-twin routing compute + the post-parse memo
/// report gate) — each call a `getenv`/`__findenv_locked` locked linear
/// `environ` scan. RE-PROFILE #12 (`-0098`) priced it at ≈3% of the regex
/// parse, and because it is codegen-emitted every generated parser paid it
/// every parse. The flag is a process-launch diagnostic switch (no code path
/// sets it via `set_var` — verified by grep), so reading it ONCE per process is
/// behavior-identical and removes the per-parse syscall. Correctness-neutral:
/// the fused-vs-protocol routing decision is a process-level constant, so no
/// parse output changes (like `trace_verbosity`, already atomic-hoisted).
static REPORT_MEMO_STATS_ENABLED: OnceLock<bool> = OnceLock::new();

pub fn report_memo_stats_enabled() -> bool {
    *REPORT_MEMO_STATS_ENABLED.get_or_init(|| std::env::var("PGEN_REPORT_MEMO_STATS").is_ok())
}

fn trace_sink() -> &'static Mutex<Option<File>> {
    TRACE_OUTPUT_SINK.get_or_init(|| Mutex::new(None))
}

fn trace_function_cache() -> &'static Mutex<HashMap<String, String>> {
    TRACE_FUNCTION_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn is_internal_trace_symbol(symbol: &str) -> bool {
    // SV-EXH-PROOF.3.3.4.b.6.2.20: also match the angle-bracketed
    // fully-qualified form `<std::backtrace::Backtrace>::create` that the
    // backtrace formatter emits — the prior plain-prefix arms below missed
    // it, so the resolver would stop at Backtrace::create and attribute
    // every trace line to it instead of the real generated-parser frame.
    let strip_angle = |s: &str| -> String { s.replace('<', "").replace('>', "") };
    let bare = strip_angle(symbol);
    symbol.contains("ast_pipeline::trace_log")
        || symbol.contains("ast_pipeline::resolve_trace_function_name")
        || symbol.contains("ast_pipeline::trace_function_name_from_backtrace")
        || symbol.contains("ast_pipeline::is_internal_trace_symbol")
        || symbol.contains("pgen_trace")
        || symbol.contains("VerbosityLogger::emit")
        || symbol.contains("Logger::log_")
        || bare.contains("std::backtrace_rs::backtrace")
        || bare.contains("std::backtrace::Backtrace")
        || bare.starts_with("std::")
        || bare.starts_with("core::")
        || bare.starts_with("alloc::")
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

/// DIAG-SEVERITY.2: format a severity-tagged diagnostic line. Pure (no I/O, no gate)
/// so it is unit-testable. Mirrors `trace_log`'s layout but tagged with the SEVERITY,
/// not a verbosity level.
fn format_diagnostic(
    severity: Severity,
    file: &str,
    line: u32,
    function_name: &str,
    rendered: &str,
) -> String {
    if rendered.is_empty() {
        format!(
            "[PGEN][{}] {} [{}:{}] [{}]",
            severity.as_str(),
            severity.emoji(),
            file,
            line,
            function_name
        )
    } else {
        format!(
            "[PGEN][{}] {} [{}:{}] [{}] {}",
            severity.as_str(),
            severity.emoji(),
            file,
            line,
            function_name,
            rendered
        )
    }
}

/// DIAG-SEVERITY.2: testable core — write a severity diagnostic to `w` with NO
/// verbosity gate (severity ≥ Warning always writes). `emit_diagnostic` is the
/// production wrapper; tests drive this with an in-memory buffer.
pub fn write_diagnostic<W: Write>(
    w: &mut W,
    severity: Severity,
    file: &str,
    line: u32,
    function_name: &str,
    args: fmt::Arguments<'_>,
) -> std::io::Result<()> {
    let rendered = format!("{}", args);
    writeln!(w, "{}", format_diagnostic(severity, file, line, function_name, &rendered))
}

/// DIAG-SEVERITY.2: emit a severity-bearing diagnostic. **NEVER gated by trace
/// verbosity** — there is deliberately NO `trace_enabled` check here. Always written to
/// **stderr** (the conventional diagnostic channel, unaffected by stdout redirection or
/// the verbosity setting) and mirrored to the trace file if one is configured, so a
/// warning/error/fatal can never be silently masked.
#[track_caller]
pub fn emit_diagnostic(
    severity: Severity,
    file: &str,
    line: u32,
    module_path: &str,
    args: fmt::Arguments<'_>,
) {
    let function_name = resolve_trace_function_name(file, line, module_path);
    let rendered = format!("{}", args);
    let output = format_diagnostic(severity, file, line, &function_name, &rendered);
    // Always-on channel: stderr, unconditionally (the whole point of this mechanism).
    eprintln!("{}", output);
    // Also mirror into the trace file if configured, so trace captures include severities.
    if let Ok(mut guard) = trace_sink().lock() {
        if let Some(file) = guard.as_mut() {
            let _ = writeln!(file, "{}", output);
            let _ = file.flush();
        }
    }
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

/// DIAG-SEVERITY.2: emit a severity-bearing diagnostic that is NEVER gated by trace
/// verbosity. Use `pgen_warn!` / `pgen_error!` / `pgen_fatal!` for the common cases.
#[macro_export]
macro_rules! pgen_diag {
    ($severity:expr, $($arg:tt)*) => {
        $crate::ast_pipeline::emit_diagnostic(
            $severity,
            file!(),
            line!(),
            module_path!(),
            format_args!($($arg)*),
        )
    };
}

#[macro_export]
macro_rules! pgen_warn {
    ($($arg:tt)*) => {
        $crate::pgen_diag!($crate::ast_pipeline::Severity::Warning, $($arg)*)
    };
}

#[macro_export]
macro_rules! pgen_error {
    ($($arg:tt)*) => {
        $crate::pgen_diag!($crate::ast_pipeline::Severity::Error, $($arg)*)
    };
}

#[macro_export]
macro_rules! pgen_fatal {
    ($($arg:tt)*) => {
        $crate::pgen_diag!($crate::ast_pipeline::Severity::Fatal, $($arg)*)
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

/// RGX-0078.5.j.4 (`PGEN-RGX-0078-0202`) — the drop-free INTERNAL error
/// carrier for the fused cascade graph. `ParseError` is 80 bytes with drop
/// glue (its cold `ContextualError` variant owns heap payloads), so every
/// `Result<_, ParseError>` discarded by fused speculation pays a
/// `drop_in_place` call. This enum mirrors exactly the variants fused bodies
/// construct, is `Copy` (`needs_drop = false` — pinned by test), and is
/// converted to the rich public `ParseError` only at the region boundary
/// (the sub-root orchestrators), so public error payloads are byte-identical
/// by bijection and the public `ParseError` ABI is untouched.
///
/// Totality contract: any public error variant WITHOUT a mirror here
/// (`ContextualError`, plus the legacy `UnexpectedEof`/`UnexpectedToken`,
/// constructed by zero AST-based-generator artifacts) crosses the boundary by
/// being PARKED in the generated parser's `cascade_parked_error` slot and
/// carried as [`CascadeControlError::Parked`]. A live `Parked` marker always
/// corresponds to the latest park (error propagation is synchronous and
/// single-threaded; a marker discarded by speculation leaves only a stale,
/// unread slot value that the next park overwrites), so the boundary
/// `take()` always observes its own park.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CascadeControlError {
    /// Mirror of [`ParseError::InvalidSyntax`].
    InvalidSyntax {
        message: &'static str,
        position: usize,
    },
    /// Mirror of [`ParseError::Backtrack`].
    Backtrack { position: usize },
    /// Mirror of [`ParseError::RecursionDepthExceeded`].
    RecursionDepthExceeded { position: usize, depth: usize },
    /// A rich/legacy boundary error parked in the generated parser's
    /// `cascade_parked_error` slot (see the totality contract above).
    Parked,
}

/// Result alias for the fused cascade graph's internal error channel
/// (RGX-0078.5.j.4 `-0202`). `CascadeResult<()>` is `Copy`, so a discarded
/// fused speculation result compiles to no drop code at all.
pub type CascadeResult<T> = Result<T, CascadeControlError>;

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

/// The per-parse node arena (RGX-0078.5.d.4.i, candidate B).
///
/// Every `ParseNode` a parse produces lives in one of these; children are held
/// as `&'input` borrows INTO the arena rather than as owned `Box`/`Vec`. The
/// arena is created at the boundary, passed into the parser by reference, and
/// dropped once the boundary has walked the tree to owned output — a single
/// mass free that ALSO runs each leaf's `String`/`serde_json::Value` destructor
/// (`typed_arena::Arena` is drop-correct, unlike a bump allocator). The lifetime
/// `'input` is unified to the ARENA's scope: input terminals are reborrowed from
/// the true input (which outlives the arena) down to `'input`, so a SINGLE
/// lifetime parameter expresses both "borrows input" and "borrows arena" — no
/// viral second lifetime. `#[derive(Serialize)]`/`PartialEq` stay byte-identical
/// because serde/Eq see through the `&'input` borrow exactly as through the old
/// `Box`/`Vec`.
/// RGX-0078.5.i.7 REPRESENTATION (`PGEN-RGX-0078-0104`): `NodeArena` grew from
/// a bare `typed_arena::Arena<ParseNode>` alias into a struct that ALSO owns
/// the arenas backing [`PgenValue`]'s composite slices and rendered strings.
/// The swap is source-compatible by the `-0103` verified fact that every
/// consumer (lib + all generated artifacts, 22,029 sites) touches the arena
/// ONLY through `NodeArena::new()` and `.alloc(node)` — both preserved with
/// identical signatures — so on-disk artifacts keep compiling unchanged.
pub struct NodeArena<'input> {
    nodes: typed_arena::Arena<ParseNode<'input>>,
    shaped_values: typed_arena::Arena<PgenValue<'input>>,
    shaped_pairs: typed_arena::Arena<(&'input str, PgenValue<'input>)>,
    /// Owned rendered strings (transform outputs and other non-input-slice
    /// text) interned for the parse's lifetime; `typed_arena` is drop-correct,
    /// so they are freed with the arena exactly like node payloads.
    rendered_strings: typed_arena::Arena<String>,
}

impl<'input> NodeArena<'input> {
    pub fn new() -> Self {
        NodeArena {
            nodes: typed_arena::Arena::new(),
            shaped_values: typed_arena::Arena::new(),
            shaped_pairs: typed_arena::Arena::new(),
            rendered_strings: typed_arena::Arena::new(),
        }
    }

    /// The historical node allocation — signature identical to the
    /// `typed_arena::Arena::alloc` every generated artifact already calls.
    #[inline]
    pub fn alloc(&self, node: ParseNode<'input>) -> &mut ParseNode<'input> {
        self.nodes.alloc(node)
    }

    /// Allocate a contiguous shaped-value slice (a `PgenValue::Array` body).
    #[inline]
    pub fn alloc_shaped_values<I>(&self, values: I) -> &mut [PgenValue<'input>]
    where
        I: IntoIterator<Item = PgenValue<'input>>,
    {
        #[cfg(debug_assertions)]
        shaped_conversion_census::record(|c| c.arena_value_slices += 1);
        self.shaped_values.alloc_extend(values)
    }

    /// Allocate a contiguous, ALREADY key-sorted/deduped pair slice (a
    /// `PgenValue::Object` body — build it with
    /// [`pgen_value::insert_object_pair`]).
    #[inline]
    pub fn alloc_shaped_pairs<I>(
        &self,
        pairs: I,
    ) -> &mut [(&'input str, PgenValue<'input>)]
    where
        I: IntoIterator<Item = (&'input str, PgenValue<'input>)>,
    {
        #[cfg(debug_assertions)]
        shaped_conversion_census::record(|c| c.arena_pair_slices += 1);
        self.shaped_pairs.alloc_extend(pairs)
    }

    /// Intern an owned rendered string for the parse's lifetime and hand back
    /// the borrow `PgenValue::Str` needs.
    #[inline]
    pub fn alloc_rendered_string(&self, text: String) -> &str {
        #[cfg(debug_assertions)]
        shaped_conversion_census::record(|c| c.arena_rendered_strings += 1);
        self.rendered_strings.alloc(text).as_str()
    }

    /// RGX-0078.5.j.1 REPRESENTATION-ROAD STEP-0 — the read-only arena
    /// population census for the construction/allocation scout: how many
    /// items each of the four arenas allocated over the parse's lifetime.
    /// Purely an accessor over `typed_arena::Arena::len()` (which counts
    /// ITEMS, so an `alloc_extend` slice of N values contributes N) — no
    /// hot-path change, no behavior change.
    pub fn census(&self) -> NodeArenaCensus {
        NodeArenaCensus {
            nodes: self.nodes.len(),
            shaped_values: self.shaped_values.len(),
            shaped_pairs: self.shaped_pairs.len(),
            rendered_strings: self.rendered_strings.len(),
        }
    }
}

/// Arena population counts reported by [`NodeArena::census`] (RGX-0078.5.j.1
/// construction/allocation census instrument).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeArenaCensus {
    /// `ParseNode` allocations (committed AST nodes + build scaffolding +
    /// doomed speculative boundary builds).
    pub nodes: usize,
    /// `PgenValue` array-slice ITEMS allocated via `alloc_shaped_values`.
    pub shaped_values: usize,
    /// `(&str, PgenValue)` object-pair ITEMS allocated via `alloc_shaped_pairs`.
    pub shaped_pairs: usize,
    /// Owned rendered strings interned via `alloc_rendered_string`.
    pub rendered_strings: usize,
}

impl Default for NodeArena<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// RGX-0078.5.j.4 V1 STEP-0 — the BUILD-VALUE census counters (debug builds
/// ONLY: `#[cfg(debug_assertions)]` strips every counter from release
/// binaries, so the shipped floor probes and the release hot path are
/// byte-untouched by this instrument). Thread-local exact counts of:
///
/// - every [`ParseContent::to_shaped_value`] conversion by INPUT variant,
///   plus the element population of `Sequence`/`Quantified` conversions
///   (each such conversion materializes a transient `Vec` and an arena
///   `alloc_extend` of that many items — the `-0151` build-value-pass
///   traffic);
/// - every [`ParseContent::clone`] by variant, plus the cloned
///   `Sequence`/`Quantified` element totals (each is a real Vec
///   malloc+memcpy; `Terminal`/`Shaped`/`Alternative` clones are
///   pointer-copies) — the `$N`-property-access clone-artifact population
///   the `-0151` re-steer named;
/// - every arena shaped-slice / rendered-string allocation CALL (the item
///   counts are already visible read-only via [`NodeArena::census`]).
///
/// Read by `regex_construction_census_probe` (`--case-file` corpus-cell
/// mode); reset per censused parse.
#[cfg(debug_assertions)]
pub mod shaped_conversion_census {
    use std::cell::Cell;

    /// One exact-count snapshot of the debug-build conversion/clone/alloc
    /// counters (thread-local; zeroed via [`reset`]).
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct ShapedConversionCensus {
        /// `to_shaped_value` calls whose receiver was `Terminal` (zero-copy `Str`).
        pub to_shaped_terminal: u64,
        /// … `TransformedTerminal` whose text parsed as JSON (`from_serde` build).
        pub to_shaped_transformed_parsed: u64,
        /// … `TransformedTerminal` wrapped as an arena-interned string.
        pub to_shaped_transformed_wrapped: u64,
        /// … `Shaped` (a plain `Copy` — the cheap arm).
        pub to_shaped_shaped: u64,
        /// … `Alternative` (a recursion hop into the child's content).
        pub to_shaped_alternative: u64,
        /// … `Sequence` (transient `Vec` collect + `alloc_extend`).
        pub to_shaped_sequence: u64,
        /// … `Quantified` (same mechanics as `Sequence`).
        pub to_shaped_quantified: u64,
        /// Total elements converted inside `Sequence`/`Quantified` arms.
        pub to_shaped_sequence_items: u64,
        /// `ParseContent::clone` calls by receiver variant. `Sequence`/
        /// `Quantified` clones are real Vec malloc+memcpy; the others are
        /// pointer/`Copy`-cheap (`TransformedTerminal` clones its `String`).
        pub clone_terminal: u64,
        pub clone_transformed: u64,
        pub clone_shaped: u64,
        pub clone_sequence: u64,
        pub clone_alternative: u64,
        pub clone_quantified: u64,
        /// Total elements across cloned `Sequence`/`Quantified` Vecs.
        pub clone_sequence_items: u64,
        /// `alloc_shaped_values` / `alloc_shaped_pairs` / `alloc_rendered_string`
        /// CALLS (slice granularity; items are in `NodeArena::census`).
        pub arena_value_slices: u64,
        pub arena_pair_slices: u64,
        pub arena_rendered_strings: u64,
    }

    thread_local! {
        static CENSUS: Cell<ShapedConversionCensus> =
            const { Cell::new(ShapedConversionCensus::new()) };
    }

    impl ShapedConversionCensus {
        const fn new() -> Self {
            // `const` twin of `Default::default()` for the const thread_local.
            ShapedConversionCensus {
                to_shaped_terminal: 0,
                to_shaped_transformed_parsed: 0,
                to_shaped_transformed_wrapped: 0,
                to_shaped_shaped: 0,
                to_shaped_alternative: 0,
                to_shaped_sequence: 0,
                to_shaped_quantified: 0,
                to_shaped_sequence_items: 0,
                clone_terminal: 0,
                clone_transformed: 0,
                clone_shaped: 0,
                clone_sequence: 0,
                clone_alternative: 0,
                clone_quantified: 0,
                clone_sequence_items: 0,
                arena_value_slices: 0,
                arena_pair_slices: 0,
                arena_rendered_strings: 0,
            }
        }
    }

    /// Zero every counter on this thread.
    pub fn reset() {
        CENSUS.with(|c| c.set(ShapedConversionCensus::new()));
    }

    /// Read the current counters on this thread.
    pub fn snapshot() -> ShapedConversionCensus {
        CENSUS.with(|c| c.get())
    }

    pub(super) fn record(update: impl FnOnce(&mut ShapedConversionCensus)) {
        CENSUS.with(|c| {
            let mut census = c.get();
            update(&mut census);
            c.set(census);
        });
    }
}

/// Parse content types
///
/// `Clone` is hand-written (below) IDENTICALLY to the former
/// `#[derive(Clone)]` expansion so debug builds can count the clone
/// population per variant (RGX-0078.5.j.4 V1 STEP-0 census); release builds
/// compile to the exact derived body with the counter stripped.
#[derive(Debug, PartialEq, serde::Serialize)]
pub enum ParseContent<'input> {
    Terminal(&'input str),
    TransformedTerminal(String),
    /// The typed structured carrier for return-annotation object/array
    /// literals and property/array access results (RGX-0078.5.i.7
    /// REPRESENTATION, `PGEN-RGX-0078-0104/-0105/-0106`) — an arena-`Copy`
    /// shaped value built directly inside the parse, with no runtime
    /// serialise/parse/serialise roundtrip and no owned `serde_json::Value`
    /// construction. Serializes as `"Json"`, the wire tag of the retired
    /// owned-`Value` variant it replaced, keeping the released typed-AST JSON
    /// carrier byte-identical: same variant tag, same value bytes (the
    /// [`PgenValue`] `Serialize` mirror), no schema bump.
    #[serde(rename = "Json")]
    Shaped(PgenValue<'input>),
    Sequence(Vec<&'input ParseNode<'input>>),
    Alternative(&'input ParseNode<'input>),
    /// The quantifier-kind label is a THIN `&'static &'static str`
    /// (RGX-0078.5.j.4 `-0212` RESULT-CARRIER SLIMMING): the double reference
    /// halves the variant's label footprint (16 → 8 B), which is what shrinks
    /// `ParseContent` 40 → 32 B (this variant is the size-dominant one). A
    /// `u8`/enum kind was refuted — the kind set is OPEN (bounded forms such
    /// as `"0,127"` are emitted as grammar-static literals). serde's blanket
    /// `&T` impl delegates, so the serialized wire bytes are unchanged; the
    /// emitted spelling is the static-promoted literal `&"*"`.
    Quantified(Vec<&'input ParseNode<'input>>, &'static &'static str),
}

impl<'input> Clone for ParseContent<'input> {
    /// Byte-for-byte the `#[derive(Clone)]` semantics: reference variants
    /// copy the reference, `TransformedTerminal` clones its `String`,
    /// `Sequence`/`Quantified` clone their `Vec` of arena references. The
    /// only addition is the debug-build census counter (stripped in release).
    fn clone(&self) -> Self {
        #[cfg(debug_assertions)]
        shaped_conversion_census::record(|c| match self {
            ParseContent::Terminal(_) => c.clone_terminal += 1,
            ParseContent::TransformedTerminal(_) => c.clone_transformed += 1,
            ParseContent::Shaped(_) => c.clone_shaped += 1,
            ParseContent::Sequence(nodes) => {
                c.clone_sequence += 1;
                c.clone_sequence_items += nodes.len() as u64;
            }
            ParseContent::Alternative(_) => c.clone_alternative += 1,
            ParseContent::Quantified(nodes, _) => {
                c.clone_quantified += 1;
                c.clone_sequence_items += nodes.len() as u64;
            }
        });
        match self {
            ParseContent::Terminal(text) => ParseContent::Terminal(text),
            ParseContent::TransformedTerminal(text) => {
                ParseContent::TransformedTerminal(text.clone())
            }
            ParseContent::Shaped(value) => ParseContent::Shaped(*value),
            ParseContent::Sequence(nodes) => ParseContent::Sequence(nodes.clone()),
            ParseContent::Alternative(node) => ParseContent::Alternative(node),
            ParseContent::Quantified(nodes, kind) => {
                ParseContent::Quantified(nodes.clone(), kind)
            }
        }
    }
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
            ParseContent::Shaped(value) => value.to_serde_value(),
            ParseContent::Alternative(node) => node.content.to_json_value(),
            ParseContent::Sequence(nodes) | ParseContent::Quantified(nodes, _) => {
                serde_json::Value::Array(
                    nodes.iter().map(|n| n.content.to_json_value()).collect(),
                )
            }
        }
    }

    /// Convert any `ParseContent` shape to the arena-`Copy` [`PgenValue`]
    /// representation — the [`Self::to_json_value`] twin for the committed-value
    /// migration (RGX-0078.5.i.7 REPRESENTATION, `PGEN-RGX-0078-0105`), mirrored
    /// variant-for-variant so the two carriers serialize byte-identically:
    /// `Terminal` → `Str` (zero-copy, where `to_json_value` allocated a `String`),
    /// `TransformedTerminal` best-effort-parses JSON text with the same
    /// `from_str`-or-wrap-as-string rule, `Sequence`/`Quantified` → `Array`,
    /// `Alternative` recurses, `Shaped` is a plain copy.
    pub fn to_shaped_value(&self, arena: &'input NodeArena<'input>) -> PgenValue<'input> {
        match self {
            ParseContent::Terminal(text) => {
                #[cfg(debug_assertions)]
                shaped_conversion_census::record(|c| c.to_shaped_terminal += 1);
                PgenValue::Str(text)
            }
            ParseContent::TransformedTerminal(text) => {
                match serde_json::from_str::<serde_json::Value>(text) {
                    Ok(value) => {
                        #[cfg(debug_assertions)]
                        shaped_conversion_census::record(|c| {
                            c.to_shaped_transformed_parsed += 1;
                        });
                        PgenValue::from_serde(&value, arena)
                    }
                    Err(_) => {
                        #[cfg(debug_assertions)]
                        shaped_conversion_census::record(|c| {
                            c.to_shaped_transformed_wrapped += 1;
                        });
                        PgenValue::Str(arena.alloc_rendered_string(text.clone()))
                    }
                }
            }
            ParseContent::Shaped(value) => {
                #[cfg(debug_assertions)]
                shaped_conversion_census::record(|c| c.to_shaped_shaped += 1);
                *value
            }
            ParseContent::Alternative(node) => {
                #[cfg(debug_assertions)]
                shaped_conversion_census::record(|c| c.to_shaped_alternative += 1);
                node.content.to_shaped_value(arena)
            }
            ParseContent::Sequence(nodes) | ParseContent::Quantified(nodes, _) => {
                #[cfg(debug_assertions)]
                shaped_conversion_census::record(|c| {
                    if matches!(self, ParseContent::Sequence(_)) {
                        c.to_shaped_sequence += 1;
                    } else {
                        c.to_shaped_quantified += 1;
                    }
                    c.to_shaped_sequence_items += nodes.len() as u64;
                });
                // Materialize BEFORE the arena call: `alloc_extend` drains its
                // iterator while holding the arena's internal borrow, and the
                // per-child conversion may itself allocate from this arena.
                let converted: Vec<PgenValue<'input>> = nodes
                    .iter()
                    .map(|node| node.content.to_shaped_value(arena))
                    .collect();
                PgenValue::Array(arena.alloc_shaped_values(converted))
            }
        }
    }
}

/// The committed-result span — a `Copy` 8-byte `{start, end}` byte-offset
/// pair (RGX-0078.5.j.4 `-0212` RESULT-CARRIER SLIMMING; was `Range<usize>`,
/// 16 B). Serializes exactly like serde's `Range` impl (a `{"start","end"}`
/// struct — declaration order IS the wire order) and `Debug`-prints as
/// `start..end`, so neither the typed-AST JSON carrier nor trace output can
/// drift. Offsets are `u32`: the generated parse entry refuses inputs longer
/// than `u32::MAX` bytes up front (one branch per PARSE), so every
/// construction-site cast is provably lossless.
#[derive(Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    #[inline(always)]
    pub fn new(start: usize, end: usize) -> Self {
        debug_assert!(
            start <= u32::MAX as usize && end <= u32::MAX as usize,
            "Span offsets exceed u32 — the parse-entry input-length guard was bypassed"
        );
        Span {
            start: start as u32,
            end: end as u32,
        }
    }

    /// The span as a `Range<usize>` — the slicing/consumer view.
    #[inline(always)]
    pub fn range(&self) -> std::ops::Range<usize> {
        self.start as usize..self.end as usize
    }
}

impl std::fmt::Debug for Span {
    /// `Range`'s `Debug` shape (`start..end`) so trace/debug output is
    /// byte-identical to the pre-`-0212` carrier.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

/// Parse node — the by-value committed-result carrier at every rule boundary.
///
/// RGX-0078.5.j.4 `-0212` RESULT-CARRIER SLIMMING: 72 → 48 B. `rule_name` is a
/// THIN `&'static &'static str` (8 B; the emitted spelling is the
/// static-promoted literal `&"name"`, the interpreter interns). Chosen over a
/// `u16` id + table: both land at exactly 48 B by alignment, and the thin ref
/// needs no table, no id→name resolution in shared serde/consumer code, and no
/// new state — serde's blanket `&T` impl and `&&str`'s `PartialEq`/`Debug`
/// delegation keep every derived behavior observably identical.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ParseNode<'input> {
    pub rule_name: &'static &'static str,
    pub content: ParseContent<'input>,
    pub span: Span,
}

// RGX-0078.5.j.4 `-0212` — the slimmed carrier layout is load-bearing for the
// measured floor; a regression to a fatter layout must fail the BUILD, not a
// bench run. (Sizes are pointer-width-dependent; pinned on 64-bit targets.)
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(std::mem::size_of::<Span>() == 8);
    assert!(std::mem::size_of::<ParseContent<'static>>() == 32);
    assert!(std::mem::size_of::<ParseNode<'static>>() == 48);
};

/// Memoization entry for a SUCCESSFUL parse.
///
/// PARSE-TERMINATION.6 — the packrat memo is SPLIT by outcome. On real grammars
/// ~81% of `(rule, position)` probes are cached FAILURES (uvm: 21 M of 25.9 M),
/// and a failure carries no information beyond "this rule failed here → backtrack
/// to this position" — which IS the key. So failures live in a lean
/// `FxHashSet<(RuleId, usize)>` (one contiguous table, no value, no allocation),
/// and only SUCCESSES carry a `MemoEntry` (kept inline/unboxed — there are few of
/// them, and a prior experiment confirmed that boxing the fields instead just
/// trades the compact table for millions of tiny heap allocations whose allocator
/// overhead on macOS negates the saving). Memoization behaviour is unchanged —
/// pure storage reshape, zero linearity impact.
#[derive(Debug, Clone)]
pub struct MemoEntry<'input> {
    /// RGX-0078.5.d.4.i — the memoized subtree root stays OWNED, but its
    /// `.5.d.3` deep-clone cost is already gone: the node's CHILDREN are now
    /// arena `&'input` borrows, so a cache-HIT `node.clone()` is a cheap shallow
    /// clone (a `Vec` of `Copy` references), not a recursive deep copy. Keeping
    /// the field owned means the generated `memoized_call` (`Some(node.clone())`
    /// on insert, `node.clone()` on hit) is unchanged.
    pub result: Option<ParseNode<'input>>,
    pub raw_semantic_content: Option<ParseContent<'input>>,
    pub end_pos: usize,
    /// SV-EXH-PROOF.3.3.4.b.6.2.36.4 — the semantic-runtime delta the rule's
    /// body produced (relative to the checkpoint captured at memoization-call
    /// entry). When the cache hit reuses the parse result instead of
    /// re-executing the body, this delta is REPLAYED so the rule's
    /// `@emit_fact` / `@open_scope` / `@close_scope` side effects appear in
    /// the current state — closing the memoization × semantic-store
    /// composition gap pinned by `.b.6.2.36.3`.
    ///
    /// `None` when the parse failed (no delta to replay); `Some(delta)` when
    /// the parse succeeded — even if the delta is empty (no side effects),
    /// the field is `Some(empty_delta)` to keep the type uniform.
    pub semantic_delta: Option<SemanticRuntimeDelta>,
    /// MEMO-STORE-SOUNDNESS.2 — `Some(write_epoch_at_insert)` when the rule
    /// body was STORE-TAINTED (it transitively evaluated ≥1 predicate), else
    /// `None` (pure-structural — valid forever). A tainted entry is replayable
    /// only while the store's write epoch is unchanged: predicates are pure
    /// functions of (position-determined args, store), so an unchanged epoch
    /// means every predicate the body evaluated would answer identically
    /// today. A stale tainted entry is evicted on hit and the caller
    /// re-parses fresh (the `sem_memo_success_*` sound pins).
    pub tainted_at_epoch: Option<u64>,
    /// GRAMMAR-WELLFORMED.H.10.2.2 — the transactional parse-COVERAGE entries
    /// the rule's body pushed (relative to the coverage-stack length captured
    /// at memoization-call entry). The same memoization × transactional-record
    /// composition gap `semantic_delta` closes for the semantic store exists
    /// for the coverage record: a memo HIT reuses the cached parse result
    /// without re-entering the rule body, so the per-rule-entry coverage push
    /// never fires — a subtree first parsed inside a rolled-back speculation
    /// (coverage truncated) and then memo-hit on the committed path ends up
    /// absent from `exercised_rule_names()`, breaking the witness record's
    /// completeness. On a cache hit this delta is replayed (extended onto the
    /// live coverage stack, inside the current speculation, so a later
    /// rollback still truncates it — transactionality preserved).
    ///
    /// `None` when coverage recording was disabled at memoization time (the
    /// ordinary-parsing default — no allocation, zero cost) or the parse
    /// failed; `Some(entries)` when coverage was enabled and the body
    /// succeeded.
    pub coverage_delta: Option<Vec<u32>>,
}

/// RGX-0078.5.i.7 (MTB-A) — one committed-derivation TAPE event of the fused
/// graph's match-then-build split (`docs/tasks/RGX-0078.md`, the `-0093`
/// design). `cascade_match_*` functions run today's fused control flow minus
/// ALL value construction, appending the decisions a later `cascade_build_*`
/// pass cannot re-derive from cursor replay alone; the build pass walks the
/// committed tape ONCE, constructing the exact `ParseContent`/`ParseNode`
/// values with a replayed position cursor. Events are POD `Copy` so
/// tournament winner-segment compaction is a `copy_within` + `truncate`
/// (zero allocation), and they never encode absolute tape indices (input
/// positions / counts / branch indices only), so compaction moves are safe.
///
/// Statically elided wherever the build cursor can re-derive the fact:
/// - `OrWinner` only at multi-branch non-byte-switch `Or` sites (a degenerate
///   byte-switch site re-dispatches on the input byte at the replayed cursor,
///   deterministic by construction);
/// - `TokStart`/`TokEnd` only where a terminal's span is dynamic (a layout
///   skip may precede it / its length is regex-matched) — a layout-sensitive
///   grammar's static literals need NO terminal events at all;
/// - `OptPresent` is mandatory at `?` fast-path sites (a static-literal inner
///   produces zero events yet advances the cursor);
/// - boundary call-out VALUES ride the unified tape as tag-0 [`TapeWord`]
///   records in append order (RGX-0078.5.j.4 `-0203`; this enum is the
///   DECODE-result vocabulary the build pass matches on — the tape itself
///   stores packed words).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivEvent {
    /// The committed winner's branch index at a multi-branch tournament site
    /// (placeholder pushed before the attempts, patched once the tournament
    /// concludes).
    OrWinner(usize),
    /// The committed iteration count of a quantified site (placeholder pushed
    /// at loop entry, patched at loop exit; enclosing-failure truncation keeps
    /// it consistent).
    QuantCount(usize),
    /// Presence of a `?` fast-path optional element (placeholder `false`
    /// pushed before the attempt, patched to `true` on success).
    OptPresent(bool),
    /// A dynamic terminal's start position (emitted only where a layout skip
    /// can precede the terminal, so the build cursor cannot derive it).
    TokStart(usize),
    /// A dynamic terminal's end position (emitted where the terminal's length
    /// is not a static literal length, or where a layout skip made the start
    /// dynamic — the literal start is then `end − literal_len`).
    TokEnd(usize),
}

/// RGX-0078.5.j.4 (`-0203` carrier core) — one word of the UNIFIED PACKED
/// derivation tape: the single 8-byte tagged-address carrier that replaces
/// the two-lane `Vec<DerivEvent>` (16 B/event) + `Vec<&ParseNode>` tape.
/// The `-0196` feasibility proof established the losslessness license: both
/// lanes are append-only preorder logs produced and consumed at the same AST
/// sites, so their merge preserves patching, speculation/lookahead suffix
/// truncation, tournament winner compaction, thin-memo segments, and nested
/// orchestrator stack discipline — each as ONE word-range operation instead
/// of two.
///
/// Encoding (the three low bits of the word are the tag; `ParseNode`'s
/// alignment is ≥ 8, so a live node pointer always has them zero):
/// - tag `0b000` — a BOUNDARY record: the aligned `&'input ParseNode`
///   pointer stored unchanged (provenance intact; the only word class that
///   is ever dereferenced, exclusively through [`TapeWord::boundary_node`]'s
///   hard tag check).
/// - tags `1..=6` — the [`DerivEvent`] variants: `OrWinner` / `QuantCount` /
///   `TokStart` / `TokEnd` carry their payload as `value << 3 | tag`
///   (narrow limit `usize::MAX >> 3`); `OptPresent` uses two payload-free
///   tags. Event words are built with `ptr::without_provenance` and are
///   never dereferenced.
/// - tag `0b111` — the WIDE escape: the header word's payload names the
///   variant and ONE following raw `usize` word carries the full payload,
///   so arbitrary `usize` values (including `usize::MAX`) round-trip and
///   the carrier imposes no input/count/index cap.
///
/// PATCHED placeholders (`OrWinner`/`QuantCount`/`OptPresent`) are narrow by
/// construction — a branch index is bounded by its site's branch count, an
/// iteration count by the emitted `SAFETY_LIMIT` (10,000), and `OptPresent`
/// is payload-free — so in-place patching (`tape[mark] = word`) is total and
/// never needs the wide escape; only appended `TokStart`/`TokEnd` positions
/// can be wide, and an append chooses 1 or 2 words at push time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TapeWord<'input> {
    word: *const ParseNode<'input>,
    _lifetime: std::marker::PhantomData<&'input ParseNode<'input>>,
}

// SAFETY: a `TapeWord` is semantically either an integer event word (built
// without provenance and never dereferenced) or exactly the
// `&'input ParseNode<'input>` the bound names; its thread-safety is
// therefore exactly that reference's, and these conditional impls preserve
// the parser struct's pre-`-0203` auto-trait surface bit for bit.
unsafe impl<'input> Send for TapeWord<'input> where &'input ParseNode<'input>: Send {}
unsafe impl<'input> Sync for TapeWord<'input> where &'input ParseNode<'input>: Sync {}

impl<'input> TapeWord<'input> {
    const TAG_MASK: usize = 0b111;
    const TAG_BOUNDARY: usize = 0b000;
    const TAG_OR_WINNER: usize = 1;
    const TAG_QUANT_COUNT: usize = 2;
    const TAG_TOK_START: usize = 3;
    const TAG_TOK_END: usize = 4;
    const TAG_OPT_ABSENT: usize = 5;
    const TAG_OPT_PRESENT: usize = 6;
    const TAG_WIDE: usize = 0b111;
    /// The largest payload a single narrow word carries.
    pub const NARROW_LIMIT: usize = usize::MAX >> 3;
    const WIDE_OR_WINNER: usize = 0;
    const WIDE_QUANT_COUNT: usize = 1;
    const WIDE_TOK_START: usize = 2;
    const WIDE_TOK_END: usize = 3;

    #[inline]
    fn from_bits(bits: usize) -> Self {
        TapeWord {
            word: std::ptr::without_provenance(bits),
            _lifetime: std::marker::PhantomData,
        }
    }

    #[inline]
    fn bits(self) -> usize {
        self.word.addr()
    }

    /// A BOUNDARY word: the aligned arena reference stored unchanged.
    #[inline]
    pub fn boundary(node: &'input ParseNode<'input>) -> Self {
        let word: *const ParseNode<'input> = node;
        debug_assert_eq!(
            word.addr() & Self::TAG_MASK,
            Self::TAG_BOUNDARY,
            "ParseNode alignment no longer clears the tape tag bits"
        );
        TapeWord {
            word,
            _lifetime: std::marker::PhantomData,
        }
    }

    /// The boundary accessor — the carrier's SOLE dereference, hard-gated on
    /// the boundary tag so an event word can never be minted into a
    /// reference (an out-of-shape read is codegen drift, never an input
    /// error).
    #[inline]
    pub fn boundary_node(self) -> &'input ParseNode<'input> {
        assert_eq!(
            self.bits() & Self::TAG_MASK,
            Self::TAG_BOUNDARY,
            "derivation-tape drift: event word read as a boundary record"
        );
        // SAFETY: tag-0 words are constructed exclusively by
        // `TapeWord::boundary` from a live `&'input ParseNode<'input>` whose
        // provenance the stored pointer preserves (event words carry no
        // provenance and are excluded by the tag check above), and the node
        // arena outlives `'input`.
        unsafe { &*self.word }
    }

    #[inline]
    fn narrow_payload(tag: usize, value: usize) -> Self {
        debug_assert!(
            value <= Self::NARROW_LIMIT,
            "derivation-tape drift: patched placeholder payload {value} exceeds the narrow limit"
        );
        Self::from_bits((value << 3) | tag)
    }

    /// Encode a STATICALLY-NARROW event as one word — the placeholder
    /// push/patch form (branch indices, iteration counts, and `OptPresent`
    /// are narrow by construction; see the type doc).
    #[inline]
    pub fn narrow_event(event: DerivEvent) -> Self {
        match event {
            DerivEvent::OrWinner(value) => Self::narrow_payload(Self::TAG_OR_WINNER, value),
            DerivEvent::QuantCount(value) => Self::narrow_payload(Self::TAG_QUANT_COUNT, value),
            DerivEvent::TokStart(value) => Self::narrow_payload(Self::TAG_TOK_START, value),
            DerivEvent::TokEnd(value) => Self::narrow_payload(Self::TAG_TOK_END, value),
            DerivEvent::OptPresent(false) => Self::from_bits(Self::TAG_OPT_ABSENT),
            DerivEvent::OptPresent(true) => Self::from_bits(Self::TAG_OPT_PRESENT),
        }
    }

    #[inline]
    fn push_payload(tape: &mut Vec<TapeWord<'input>>, tag: usize, wide_variant: usize, value: usize) {
        if value <= Self::NARROW_LIMIT {
            tape.push(Self::from_bits((value << 3) | tag));
        } else {
            tape.push(Self::from_bits((wide_variant << 3) | Self::TAG_WIDE));
            tape.push(Self::from_bits(value));
        }
    }

    /// Append an event of ARBITRARY payload — one narrow word, or the wide
    /// header + raw payload word (the lossless escape).
    #[inline]
    pub fn push_event(tape: &mut Vec<TapeWord<'input>>, event: DerivEvent) {
        match event {
            DerivEvent::OrWinner(value) => {
                Self::push_payload(tape, Self::TAG_OR_WINNER, Self::WIDE_OR_WINNER, value)
            }
            DerivEvent::QuantCount(value) => {
                Self::push_payload(tape, Self::TAG_QUANT_COUNT, Self::WIDE_QUANT_COUNT, value)
            }
            DerivEvent::TokStart(value) => {
                Self::push_payload(tape, Self::TAG_TOK_START, Self::WIDE_TOK_START, value)
            }
            DerivEvent::TokEnd(value) => {
                Self::push_payload(tape, Self::TAG_TOK_END, Self::WIDE_TOK_END, value)
            }
            DerivEvent::OptPresent(present) => tape.push(Self::from_bits(if present {
                Self::TAG_OPT_PRESENT
            } else {
                Self::TAG_OPT_ABSENT
            })),
        }
    }

    /// Decode the event starting at `tape[index]`, returning it with the
    /// number of words consumed (1, or 2 for a wide escape). Reading a
    /// boundary word here is codegen drift — loud, never silent.
    #[inline]
    pub fn decode_event(tape: &[TapeWord<'input>], index: usize) -> (DerivEvent, usize) {
        let bits = tape[index].bits();
        let payload = bits >> 3;
        match bits & Self::TAG_MASK {
            Self::TAG_OR_WINNER => (DerivEvent::OrWinner(payload), 1),
            Self::TAG_QUANT_COUNT => (DerivEvent::QuantCount(payload), 1),
            Self::TAG_TOK_START => (DerivEvent::TokStart(payload), 1),
            Self::TAG_TOK_END => (DerivEvent::TokEnd(payload), 1),
            Self::TAG_OPT_ABSENT => (DerivEvent::OptPresent(false), 1),
            Self::TAG_OPT_PRESENT => (DerivEvent::OptPresent(true), 1),
            Self::TAG_WIDE => {
                let value = tape[index + 1].bits();
                let event = match payload {
                    Self::WIDE_OR_WINNER => DerivEvent::OrWinner(value),
                    Self::WIDE_QUANT_COUNT => DerivEvent::QuantCount(value),
                    Self::WIDE_TOK_START => DerivEvent::TokStart(value),
                    Self::WIDE_TOK_END => DerivEvent::TokEnd(value),
                    other => unreachable!(
                        "derivation-tape drift: unknown wide event variant {other}"
                    ),
                };
                (event, 2)
            }
            _ => unreachable!("derivation-tape drift: boundary word read as an event"),
        }
    }
}

/// RGX-0078.5.i.7 (D2-B + MTB-B) / RGX-0078.5.i.14 (C3) / RGX-0078.5.j.4
/// (`-0203` carrier core) — one entry of the fused cascade graph's THIN memo:
/// the ⛔ session-#49 bound's carrier for CYCLE-PARTICIPATING fused rules
/// (`CascadeEmissionPlan::thin_memo`), which must never lose memo protection.
/// The payload is a committed derivation SEGMENT over the UNIFIED packed tape
/// — ONE contiguous [`TapeWord`] slice (events and boundary records
/// interleaved in append order). Lineage (each step an additive transient
/// migration completed once no regenerated artifact referenced the old type):
/// the eager value-carrying `ThinMemoEntry` → the `Vec`-backed
/// `ThinDerivMemoEntry` (retired by C3) → the two-segment inline-small
/// `ThinDerivSegMemoEntry` (retired by `-0203` when the two tape lanes merged).
///
/// A valid HIT splices the cached `(end, tape-segment)` onto the live tape
/// with one `extend_from_slice` and jumps the position; the build pass
/// constructs the value ONCE from the spliced words — so a memoized
/// sub-derivation on a DOOMED path is truncated un-built. Tape records are
/// tape-index-free (input positions / counts / branch indices / arena
/// pointers only), so a segment is position-independent within the tape; the
/// memo key pins the input position, so the absolute input positions inside
/// the segment replay exactly. Boundary words carry arena refs (`Copy`),
/// alive for the whole parse — the eager entry's shallow-replay economics.
///
/// Unlike [`MemoEntry`], a thin entry carries NO semantic/coverage delta —
/// replay is `position = end` plus the tape splice (or the cached failure)
/// and nothing else, so an entry is cached ONLY when splice-replay is
/// provably equivalent to re-execution. The entry's `stamp` carries the
/// protocol memo's own taint classes, measured across the body with three
/// engine counters (the store write epoch — monotone, bumped by EVERY
/// delta-visible mutation — the deferred-obligation count — the one
/// deliberately epoch-blind mutation — and the predicate-evaluation counter,
/// the store's only read path into parsing):
///
/// - **PURE** (`stamp: None`) — the body neither read a predicate nor mutated
///   the store: its outcome is a function of (input, position) alone, and a
///   replay skips nothing, so it is valid at ANY later store state — exactly
///   the protocol's untainted-entry license. (The first thin-memo emission
///   validated every entry against the global "store unchanged since insert"
///   pair instead; measured on the 8-pattern bench that evicted the whole
///   spine's entries on every capture fact write and REGRESSED the two
///   fact-writing patterns +3.4/+15.4% — the per-entry class is the fix.)
/// - **STORE-READ** (`stamp: Some((write_epoch, deferred_len))`) — the body
///   evaluated ≥1 predicate but mutated nothing: replayable only while both
///   stamps are unchanged (predicates are pure functions of position + store,
///   the MEMO-STORE-SOUNDNESS.2 license); evicted and re-executed otherwise.
/// - **STORE-MUTATING** — the body changed the epoch or enqueued an
///   obligation: NOT cached at all. A splice-only replay would skip the
///   mutation the protocol memo re-applies from its stored delta, so every
///   re-probe honestly re-executes (deterministic ⇒ same outcome + same
///   effects).
///
/// INLINE-SMALL SEGMENT (the C3 economics carried over): the inline capacity
/// of 6 words (48 B) covers the census common segment (one placeholder +
/// 0–2 events + 0–1 boundary); a longer segment spills to the heap exactly
/// as before — same worst case, cheaper common case — and the per-success
/// copy is ONE `SmallVec::from_slice` memcpy instead of the two-lane form's
/// two ([`TapeWord`] is `Copy` POD, so the copy is a plain memcpy and a hit
/// still splices via `extend_from_slice`).
#[derive(Debug, Clone)]
pub struct ThinTapeMemoEntry<'input> {
    /// `None` = PURE (valid forever); `Some((write_epoch, deferred_len))` =
    /// STORE-READ, both captured at body entry and validated at replay.
    pub stamp: Option<(u64, usize)>,
    /// `Some((end_pos, tape_segment))` for a successful match; `None` for a
    /// cached failure, replayed as a backtrack at the probe key's position.
    pub outcome: Option<(usize, smallvec::SmallVec<[TapeWord<'input>; 6]>)>,
}

/// (`-0205` direct-index carrier) — the thin memo's ROW TABLE: a
/// generation-stamped `(thin_rule_row × position)` slot array replacing the
/// `FxHashMap<(RuleId, usize), ThinTapeMemoEntry>` CONTAINER (the entry type
/// and its stamp/taint doctrine are untouched; entries live in a per-parser
/// dense `Vec` and a slot holds the entry's index).
///
/// Slot word encoding: `(gen as u64) << 32 | (idx + 1)` — `0` = never
/// written. A slot is live only when its high word equals the CURRENT
/// generation, so [`Self::begin`] invalidates every previous parse's slots
/// with ONE counter bump instead of an `O(rules × positions)` clear — the
/// design fork's naive per-parse zeroed array was refused precisely because
/// its alloc+memset is an ADDED fixed cost on the sub-1 µs corpus band.
/// On u32 generation wrap (once per 2³² parses per thread) the table is
/// hard-cleared and the generation restarts at 1.
///
/// The table itself is POSITION-KEYED PER PARSE but its STORAGE is recycled
/// across parses through a thread-local (see [`ThinMemoScratchLease`]);
/// `begin` grows it to the current parse's `rows × (input_len + 1)` need
/// (`resize` zeroes only the new tail — old slots die by generation
/// mismatch).
#[derive(Debug, Default)]
pub struct ThinMemoScratch {
    slots: Vec<u64>,
    generation: u32,
}

impl ThinMemoScratch {
    /// Open a new parse: invalidate every prior slot (generation bump) and
    /// ensure capacity for `need = thin_rule_count * (input_len + 1)` slots.
    pub fn begin(&mut self, need: usize) {
        self.generation = self.generation.wrapping_add(1);
        if self.generation == 0 {
            // u32 generation wrap: hard-clear once, restart at 1 so the
            // never-written encoding (0) stays unambiguous.
            self.slots.iter_mut().for_each(|slot| *slot = 0);
            self.generation = 1;
        }
        if self.slots.len() < need {
            self.slots.resize(need, 0);
        }
    }

    /// Current-generation entry index at `slot`, if one was stored this parse.
    #[inline]
    pub fn lookup(&self, slot: usize) -> Option<u32> {
        let word = self.slots[slot];
        if (word >> 32) as u32 == self.generation {
            Some((word as u32) - 1)
        } else {
            None
        }
    }

    /// Store `idx` at `slot` for the current generation.
    #[inline]
    pub fn store(&mut self, slot: usize, idx: u32) {
        self.slots[slot] = (u64::from(self.generation) << 32) | u64::from(idx + 1);
    }

    /// Evict a stale entry's slot (the `remove` analogue; the dense entry
    /// itself stays as bounded garbage, superseded on re-insert).
    #[inline]
    pub fn clear(&mut self, slot: usize) {
        self.slots[slot] = 0;
    }
}

std::thread_local! {
    /// The per-thread recycled thin-memo row table (see
    /// [`ThinMemoScratchLease`]). `Cell<Option<…>>` so take/put are plain
    /// moves with no borrow-state.
    static THIN_MEMO_SCRATCH: std::cell::Cell<Option<ThinMemoScratch>> =
        const { std::cell::Cell::new(None) };
}

/// RAII lease on the thread's recycled [`ThinMemoScratch`]. Generated
/// parsers hold one as a FIELD: taking the lease at construction and
/// returning the scratch when the parser drops (any exit path — success,
/// error, unwind) without an `impl Drop` on the parser struct itself. If two
/// parsers are alive on one thread the second simply takes a fresh scratch
/// (correct, merely unshared); on return, last-drop wins the thread slot —
/// the steady state is one warm table per thread either way.
#[derive(Debug, Default)]
pub struct ThinMemoScratchLease {
    inner: ThinMemoScratch,
}

impl ThinMemoScratchLease {
    /// Take the thread's scratch (or a fresh default on the cold path).
    pub fn take() -> Self {
        Self {
            inner: THIN_MEMO_SCRATCH.with(|cell| cell.take()).unwrap_or_default(),
        }
    }
}

impl Drop for ThinMemoScratchLease {
    fn drop(&mut self) {
        THIN_MEMO_SCRATCH.with(|cell| {
            cell.set(Some(std::mem::take(&mut self.inner)));
        });
    }
}

impl std::ops::Deref for ThinMemoScratchLease {
    type Target = ThinMemoScratch;
    fn deref(&self) -> &ThinMemoScratch {
        &self.inner
    }
}

impl std::ops::DerefMut for ThinMemoScratchLease {
    fn deref_mut(&mut self) -> &mut ThinMemoScratch {
        &mut self.inner
    }
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
    /// The active parse stack, oldest frame first: `(rule_name, position)`.
    /// The legacy name-scan cycle check keys on this, and the emitted
    /// protocol call sites that read a frame's rule name (the protocol
    /// `try_parse` rollback label and the debug stack-path log) read
    /// `entry.0` here — so its 2-tuple shape is kept byte-compatible with
    /// every on-disk artifact (RGX-0078.5.i.14/C1: the id lives in the
    /// parallel stack below, not by widening this frame). RGX-0078.5.j.4
    /// (`-0200`): generated BARE fused frames no longer push here — their
    /// name readers (`create_contextual_error`, the bare rollback label)
    /// reconstruct names from `rule_id_stack` through the parser's own
    /// `RULE_NAMES` bijection instead.
    pub parse_stack: Vec<(&'static str, usize)>,
    /// RGX-0078.5.i.14 (C1) — a dense parallel `(RuleId, position)` stack.
    /// [`Self::check_cycle_id`] scans THIS instead of the name stack,
    /// replacing the per-frame `&'static str` content (length + `memcmp`)
    /// compare with a single `RuleId` integer compare, and it is
    /// self-contained (position travels with the id) so the hot scan never
    /// touches the wider name frames. A legacy name-scan parser fills the id
    /// slot with a `RuleId::MAX` placeholder it never reads.
    ///
    /// RGX-0078.5.j.4 (`-0200`) — this is the COMPLETE representation for a
    /// generated BARE fused frame: [`Self::enter_id_bare`]/[`Self::exit_bare`]
    /// maintain ONLY this stack (the parser reconstructs a name through its
    /// own `RULE_NAMES` bijection when an error/diagnostic path needs one),
    /// so during a bare parse this stack can be DEEPER than `parse_stack`.
    /// The paired methods (`enter`, `enter_id`, `exit`, `truncate_stack`)
    /// keep the two stacks in lockstep wherever bare frames are not used —
    /// i.e., in every protocol/legacy parse the lockstep invariant holds
    /// unchanged.
    pub rule_id_stack: Vec<(RuleId, usize)>,
    pub max_depth: usize,
    pub cycle_cache: HashMap<(String, usize), CycleType>,
    /// SV-CORPUS-GRAD.3.12 — the STACK INDEX of the frame that caused the most recent blocking
    /// cycle verdict (`Infinite` / `LeftRecursive`); `usize::MAX` when the last check did not
    /// block. Written only on the already-failing paths of [`Self::check_cycle`] /
    /// [`Self::check_cycle_id`], so the hot `CycleType::None` return is untouched.
    ///
    /// Why an index rather than a flag: a guard rejection is a fact about the live parse stack, so
    /// an outcome computed under one is not a function of the memo key `(rule, position)`. But it
    /// is only the frames OUTSIDE the memoized rule's own subtree that make it so — a block whose
    /// blocking frame is the memoized rule itself, or one of its own descendants, is re-created
    /// identically by every replay of that body and is therefore harmless. Both scans return on
    /// the FIRST (oldest, shallowest) match, so this index is exactly the floor a caller needs to
    /// compare against its own frame depth. See `memoized_call`'s taint gate.
    pub last_block_frame: usize,
}

impl RecursionGuard {
    pub fn new(max_depth: usize) -> Self {
        Self {
            parse_stack: Vec::new(),
            rule_id_stack: Vec::new(),
            max_depth,
            cycle_cache: HashMap::new(),
            last_block_frame: usize::MAX,
        }
    }

    /// Legacy NAME-scan cycle check — byte-for-byte the pre-C1 behavior, used
    /// by the bootstrap (`ast_code_generator`) emitter and any caller without a
    /// `RuleId`.
    pub fn check_cycle(&mut self, rule_name: &'static str, position: usize) -> CycleType {
        for (index, (r, p)) in self.parse_stack.iter().enumerate() {
            if *r == rule_name && *p == position {
                self.last_block_frame = index;
                return CycleType::Infinite;
            }
            if *r == rule_name && *p > position {
                self.last_block_frame = index;
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

    /// RGX-0078.5.i.14 (C1) — id-aware cycle check. Scans the dense
    /// `rule_id_stack` (`RuleId` integer compare) instead of the name stack.
    /// Because a parser's `RuleId`↔rule-name mapping is a bijection,
    /// `*rid == rule_id` holds for exactly the frames `*r == rule_name`
    /// would — so the Infinite/LeftRecursive/MutualRecursive verdict, its
    /// `rules` payload (still read from `parse_stack`), and the oldest-first
    /// total-depth walk (including the `max_depth` ceiling) are
    /// byte-identical to [`Self::check_cycle`] whenever the stacks are in
    /// lockstep (every protocol/legacy parse).
    ///
    /// RGX-0078.5.j.4 (`-0200`) — the maximum-depth arm counts
    /// `rule_id_stack.len()`: the ID stack holds EVERY live frame (paired
    /// pushes fill both stacks; bare pushes fill only this one), so the
    /// recursion ceiling keeps firing at the true total depth even while
    /// generated bare frames leave `parse_stack` shallow. Byte-identical to
    /// the previous `parse_stack.len()` count wherever bare frames do not
    /// exist. During a bare parse the `rules` payload may name only the
    /// paired (protocol) frames — the generated recursion-error arms bind
    /// `depth` alone, and generated bare paths reconstruct complete names
    /// from `rule_id_stack` through their own `RULE_NAMES` table instead.
    pub fn check_cycle_id(&mut self, rule_id: RuleId, position: usize) -> CycleType {
        for (index, (rid, p)) in self.rule_id_stack.iter().enumerate() {
            if *rid == rule_id && *p == position {
                // SV-CORPUS-GRAD.3.12 — record the BLOCKING FRAME's index (see the field). One
                // store on a path that is already returning an error; the `None` fall-through
                // below — the hot case — writes nothing.
                self.last_block_frame = index;
                return CycleType::Infinite;
            }
            if *rid == rule_id && *p > position {
                self.last_block_frame = index;
                return CycleType::LeftRecursive;
            }
        }
        if self.rule_id_stack.len() >= self.max_depth {
            let rules: Vec<&'static str> = self.parse_stack.iter().map(|(r, _)| *r).collect();
            return CycleType::MutualRecursive {
                depth: self.rule_id_stack.len(),
                rules,
            };
        }
        CycleType::None
    }

    /// Legacy name-only push. Mirrors the frame into `rule_id_stack` with a
    /// `RuleId::MAX` placeholder so the two stacks stay length-synced even for a
    /// bootstrap parser (which only ever reads the name via
    /// [`Self::check_cycle`]).
    pub fn enter(&mut self, rule_name: &'static str, position: usize) {
        self.parse_stack.push((rule_name, position));
        self.rule_id_stack.push((RuleId::MAX, position));
    }

    /// RGX-0078.5.i.14 (C1) — id-carrying push used by the protocol + cascade
    /// emitters so [`Self::check_cycle_id`] can scan by integer. The name is
    /// still pushed to `parse_stack` for the `MutualRecursive` payload, the
    /// error/trace text, and the three name-reading call sites.
    pub fn enter_id(&mut self, rule_id: RuleId, rule_name: &'static str, position: usize) {
        self.parse_stack.push((rule_name, position));
        self.rule_id_stack.push((rule_id, position));
    }

    pub fn exit(&mut self) {
        self.parse_stack.pop();
        self.rule_id_stack.pop();
    }

    /// RGX-0078.5.j.4 (`-0200`) — BARE id-only push for a generated fused
    /// frame: maintains ONLY `rule_id_stack`, the complete modern recursion
    /// representation ([`Self::check_cycle_id`] scans it and counts its
    /// depth). The 24-byte name frame is not written; a generated error/
    /// diagnostic path that needs a name maps the retained `RuleId` through
    /// that parser's static `RULE_NAMES` table (rule id = table index, a
    /// total bijection for a modern generated parser). Pair with
    /// [`Self::exit_bare`]; protocol/legacy callers keep the paired
    /// [`Self::enter_id`]/[`Self::exit`] path unchanged.
    #[inline]
    pub fn enter_id_bare(&mut self, rule_id: RuleId, position: usize) {
        self.rule_id_stack.push((rule_id, position));
    }

    /// RGX-0078.5.j.4 (`-0200`) — BARE id-only pop, the [`Self::enter_id_bare`]
    /// counterpart.
    #[inline]
    pub fn exit_bare(&mut self) {
        self.rule_id_stack.pop();
    }

    /// RGX-0078.5.i.14 (C1) — truncate BOTH stacks to `len`, the legacy
    /// `try_parse` rollback restore for an all-paired parse, where a prior
    /// `parse_stack.len()` equals `rule_id_stack.len()` by the lockstep
    /// invariant. A caller whose speculation may span BARE id-only frames
    /// (RGX-0078.5.j.4) must use [`Self::truncate_stacks`] instead — this
    /// single-length form would cut live deeper ID frames down to the name
    /// depth.
    pub fn truncate_stack(&mut self, len: usize) {
        self.parse_stack.truncate(len);
        self.rule_id_stack.truncate(len);
    }

    /// RGX-0078.5.j.4 (`-0200`) — per-stack rollback restore: each stack is
    /// truncated to its OWN saved length, which is correct both for an
    /// all-paired parse (the two saved lengths are equal, degenerating to
    /// [`Self::truncate_stack`]) and for a mixed-depth parse where bare
    /// id-only frames made `rule_id_stack` deeper than `parse_stack`.
    #[inline]
    pub fn truncate_stacks(&mut self, name_len: usize, id_len: usize) {
        self.parse_stack.truncate(name_len);
        self.rule_id_stack.truncate(id_len);
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
/// The bounded forms are also accepted in their brace-STRIPPED spelling
/// (`"N"` / `"N,M"` / `"N,"` / `",M"`): the Rust EBNF frontend's
/// `parse_braced_quantifier` normalizes `item{2,3}` to the raw-AST token
/// `["quantifier","2,3"]` (braces removed), and that is the string
/// `ASTNode::Quantified::quantifier` actually carries at codegen /
/// interpret / lint time. Decoding both spellings here closed the
/// bounded-quantifier half-wire in which stimuli generation accepted a
/// grammar whose parser could not be compiled (`Unknown quantifier: 2,3`)
/// — BOUNDED-QUANT.1.
///
/// Returns `None` for any other input (invalid quantifier string).
///
/// This is the canonical surface-form → bounds mapping used by the
/// `ast_based_generator` / `ast_code_generator` quantifier codegen, the
/// parse-harness interpreter, the grammar-wellformedness linter, and (by
/// delegation) the stimuli generator, so a single helper carries every
/// repetition operator the engine supports.
pub fn parse_quantifier_bounds(quantifier: &str) -> Option<(usize, Option<usize>)> {
    let q = quantifier.trim();
    match q {
        "?" => return Some((0, Some(1))),
        "*" => return Some((0, None)),
        "+" => return Some((1, None)),
        _ => {}
    }
    // Bounded forms: strip the braces when present so the documented braced
    // spelling and the frontend's brace-stripped raw-AST spelling decode
    // through the same inner parser.
    let inner = if q.starts_with('{') && q.ends_with('}') && q.len() >= 3 {
        &q[1..q.len() - 1]
    } else {
        q
    };
    if inner.trim().is_empty() {
        return None;
    }
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
        // "{N}" / "N" — exact count
        let n: usize = inner.trim().parse().ok()?;
        Some((n, Some(n)))
    }
}

#[cfg(test)]
mod diag_severity_tests {
    use super::*;

    // DIAG-SEVERITY.2: the core invariant — a severity-bearing diagnostic must emit
    // even at the very verbosity setting (`None`) that masks every trace level. This
    // is the regression test for the defect that hid the SV depth-exceeded error.
    #[test]
    fn severity_diagnostics_emit_regardless_of_verbosity() {
        // The exact masking setting the closed-loop gate runs at:
        set_global_trace_verbosity(TraceVerbosity::None);

        // CONTRAST (the bug): a verbosity-gated trace is suppressed at None — even High.
        assert!(
            !trace_enabled(TraceLevel::High),
            "a High *trace* (verbosity) must be suppressed at verbosity None"
        );

        // THE FIX: the severity path still writes, with NO verbosity gate. Drive the
        // testable core into an in-memory buffer (no stderr capture needed).
        let mut buf: Vec<u8> = Vec::new();
        write_diagnostic(
            &mut buf,
            Severity::Error,
            "stimuli_generator.rs",
            4377,
            "generate_rule",
            format_args!("Stimuli generation depth exceeded max_depth={}", 24),
        )
        .expect("write_diagnostic must not fail on an in-memory buffer");
        let rendered = String::from_utf8(buf).expect("utf8");
        assert!(
            rendered.contains("ERROR"),
            "diagnostic must carry the ERROR severity tag, got: {rendered}"
        );
        assert!(
            rendered.contains("Stimuli generation depth exceeded max_depth=24"),
            "diagnostic must carry the message verbatim, got: {rendered}"
        );
    }

    // Severity is an orderable scale so callers can threshold (e.g. error-and-above).
    #[test]
    fn severity_is_ordered_warning_lt_error_lt_fatal() {
        assert!(Severity::Warning < Severity::Error);
        assert!(Severity::Error < Severity::Fatal);
        assert_eq!(Severity::Warning.as_str(), "WARN");
        assert_eq!(Severity::Error.as_str(), "ERROR");
        assert_eq!(Severity::Fatal.as_str(), "FATAL");
    }
}

#[cfg(test)]
mod cascade_control_error_tests {
    use super::{CascadeControlError, CascadeResult, ParseResult};

    // RGX-0078.5.j.4 (-0202): the in-tree layout pin the -0173 design record
    // owed — the internal carrier must be Copy and drop-free (the whole point
    // of the lever: `drop_in_place::<Result<(), ParseError>>` measured 4.422 ns
    // on the fused-path profiles), and strictly narrower than the 80-byte
    // public carrier.
    #[test]
    fn cascade_control_error_is_copy_dropfree_and_narrower_than_parse_error() {
        fn assert_copy<T: Copy>() {}
        assert_copy::<CascadeControlError>();
        assert_copy::<CascadeResult<()>>();
        assert!(
            !std::mem::needs_drop::<CascadeControlError>(),
            "the internal cascade carrier must be drop-free"
        );
        assert!(
            !std::mem::needs_drop::<CascadeResult<()>>(),
            "a discarded fused speculation result must compile to no drop code"
        );
        assert!(
            std::mem::size_of::<CascadeControlError>() <= 32,
            "the internal carrier must stay at/below the 32-byte -0173 shape-C width, got {}",
            std::mem::size_of::<CascadeControlError>()
        );
        assert!(
            std::mem::size_of::<CascadeResult<()>>() < std::mem::size_of::<ParseResult<()>>(),
            "the internal result must be narrower than the public one ({} vs {})",
            std::mem::size_of::<CascadeResult<()>>(),
            std::mem::size_of::<ParseResult<()>>()
        );
    }
}

#[cfg(test)]
mod tape_word_tests {
    use super::{DerivEvent, NodeArena, ParseContent, ParseNode, TapeWord};

    // RGX-0078.5.j.4 (-0203): the carrier-core layout pin — the unified tape
    // word must be exactly one 8-byte Copy drop-free word (the whole point of
    // the representation: halve the 16-byte event element and let boundary
    // records share the lane), and every event payload must round-trip
    // losslessly through the narrow form and the wide escape.
    #[test]
    fn tape_word_is_one_copy_dropfree_word() {
        fn assert_copy<T: Copy>() {}
        assert_copy::<TapeWord<'static>>();
        assert_eq!(
            std::mem::size_of::<TapeWord<'static>>(),
            8,
            "the unified tape word must be exactly one 8-byte word"
        );
        assert!(
            !std::mem::needs_drop::<TapeWord<'static>>(),
            "the unified tape word must be drop-free"
        );
        assert!(
            std::mem::size_of::<TapeWord<'static>>() < std::mem::size_of::<DerivEvent>(),
            "the packed word must be narrower than the two-lane DerivEvent element ({} vs {})",
            std::mem::size_of::<TapeWord<'static>>(),
            std::mem::size_of::<DerivEvent>()
        );
    }

    #[test]
    fn every_event_payload_round_trips_narrow_and_wide() {
        let payload_events: &[fn(usize) -> DerivEvent] = &[
            DerivEvent::OrWinner,
            DerivEvent::QuantCount,
            DerivEvent::TokStart,
            DerivEvent::TokEnd,
        ];
        let boundary_payloads = [
            0usize,
            1,
            4096,
            TapeWord::NARROW_LIMIT,
            TapeWord::NARROW_LIMIT + 1,
            usize::MAX,
        ];
        for make in payload_events {
            for &value in &boundary_payloads {
                let mut tape: Vec<TapeWord<'static>> = Vec::new();
                TapeWord::push_event(&mut tape, make(value));
                let expected_words = if value <= TapeWord::NARROW_LIMIT { 1 } else { 2 };
                assert_eq!(tape.len(), expected_words, "wrong word count for {value}");
                let (event, consumed) = TapeWord::decode_event(&tape, 0);
                assert_eq!(event, make(value));
                assert_eq!(consumed, expected_words);
            }
        }
        for present in [false, true] {
            let mut tape: Vec<TapeWord<'static>> = Vec::new();
            TapeWord::push_event(&mut tape, DerivEvent::OptPresent(present));
            assert_eq!(tape.len(), 1);
            assert_eq!(
                TapeWord::decode_event(&tape, 0),
                (DerivEvent::OptPresent(present), 1)
            );
        }
    }

    #[test]
    fn narrow_event_patch_form_round_trips_placeholder_values() {
        // The patch population: branch indices (bounded by branch counts),
        // iteration counts (bounded by the emitted SAFETY_LIMIT), OptPresent.
        for event in [
            DerivEvent::OrWinner(0),
            DerivEvent::OrWinner(23),
            DerivEvent::QuantCount(0),
            DerivEvent::QuantCount(10_000),
            DerivEvent::OptPresent(false),
            DerivEvent::OptPresent(true),
        ] {
            let tape = [TapeWord::narrow_event(event)];
            assert_eq!(TapeWord::decode_event(&tape, 0), (event, 1));
        }
    }

    #[test]
    fn boundary_pointer_round_trips_through_the_tagged_word() {
        let arena = NodeArena::new();
        let node: &ParseNode<'_> = arena.alloc(ParseNode {
            rule_name: &"tape_word_pin",
            content: ParseContent::Terminal("x"),
            span: crate::ast_pipeline::Span::new(0, 1),
        });
        let word = TapeWord::boundary(node);
        let restored = word.boundary_node();
        assert!(std::ptr::eq(node, restored), "boundary identity must survive the tag");
        assert_eq!(*restored.rule_name, "tape_word_pin");
        assert_eq!(restored.span, crate::ast_pipeline::Span::new(0, 1));
    }

    #[test]
    #[should_panic(expected = "derivation-tape drift")]
    fn an_event_word_can_never_be_minted_into_a_reference() {
        let word = TapeWord::narrow_event(DerivEvent::OrWinner(3));
        let _ = word.boundary_node();
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

    // BOUNDED-QUANT.1: the brace-STRIPPED spelling the Rust EBNF frontend
    // actually emits into the raw AST (`["quantifier","2,3"]`) must decode
    // identically to the braced source spelling.
    #[test]
    fn brace_stripped_bounded_quantifiers() {
        assert_eq!(parse_quantifier_bounds("3"), Some((3, Some(3))));
        assert_eq!(parse_quantifier_bounds("2,5"), Some((2, Some(5))));
        assert_eq!(parse_quantifier_bounds("2,"), Some((2, None)));
        assert_eq!(parse_quantifier_bounds(",5"), Some((0, Some(5))));
        assert_eq!(parse_quantifier_bounds("0,0"), Some((0, Some(0))));
        // inner whitespace tolerated, same as the braced spelling
        assert_eq!(parse_quantifier_bounds(" 2 , 5 "), Some((2, Some(5))));
    }

    #[test]
    fn invalid_quantifiers_return_none() {
        assert_eq!(parse_quantifier_bounds(""), None);
        assert_eq!(parse_quantifier_bounds("foo"), None);
        assert_eq!(parse_quantifier_bounds("{}"), None);
        assert_eq!(parse_quantifier_bounds("{a}"), None);
        // M < N is invalid — both spellings
        assert_eq!(parse_quantifier_bounds("{5,2}"), None);
        assert_eq!(parse_quantifier_bounds("5,2"), None);
        // Negative numbers reject via usize parse — both spellings
        assert_eq!(parse_quantifier_bounds("{-1}"), None);
        assert_eq!(parse_quantifier_bounds("-1"), None);
        // A second comma is not a valid bounds shape
        assert_eq!(parse_quantifier_bounds("1,2,3"), None);
        // A lone comma decodes as the degenerate open range `(0, None)` in
        // BOTH spellings (`{,}` always did; the stripped form must match).
        assert_eq!(parse_quantifier_bounds(","), Some((0, None)));
        assert_eq!(parse_quantifier_bounds("{,}"), Some((0, None)));
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

/// LEXICAL-ANNOTATIONS.3c — one item in a follow-restriction list. The list is a
/// union: a `Regex` item matches if its pattern matches the follow text; a
/// `Literal` item matches the exact string. Items are derived from the EBNF
/// `[> /regex/ "string" … ]` / `[>! … ]` before-rule directive.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum FollowItem {
    Regex(String),
    Literal(String),
}

/// LEXICAL-ANNOTATIONS.3c — a per-rule lexical follow-restriction (the declarative
/// "Obligation C" of the faithful-rendering invariant). A rule annotated with
/// `[>! LIST ]` must NOT be immediately followed by any item in `LIST`
/// (`forbid == true`); `[> LIST ]` must be followed by one of them
/// (`forbid == false`). FORBID is consumed by the stimuli generator as a minimal
/// self-terminating separator; REQUIRE is carried for parse-side disambiguation
/// (the bidirectional pillar) and is a documented generation no-op today.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct FollowRestriction {
    pub forbid: bool,
    pub items: Vec<FollowItem>,
}

impl FollowRestriction {
    /// Parse the structured token payload emitted by the EBNF frontend
    /// (`{ "polarity": "forbid"|"require", "items": [{ "kind": …, "value": … }] }`).
    /// Returns `None` for any malformed payload so the caller can fail loudly
    /// (mirrors the frontend's strict parse — never silently accept a half-token).
    pub fn from_token_payload(payload: &serde_json::Value) -> Option<Self> {
        let forbid = match payload.get("polarity").and_then(|v| v.as_str())? {
            "forbid" => true,
            "require" => false,
            _ => return None,
        };
        let raw_items = payload.get("items").and_then(|v| v.as_array())?;
        let mut items = Vec::with_capacity(raw_items.len());
        for raw in raw_items {
            let value = raw.get("value").and_then(|v| v.as_str())?.to_string();
            match raw.get("kind").and_then(|v| v.as_str())? {
                "regex" => items.push(FollowItem::Regex(value)),
                "literal" => items.push(FollowItem::Literal(value)),
                _ => return None,
            }
        }
        if items.is_empty() {
            return None;
        }
        Some(FollowRestriction { forbid, items })
    }
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
    /// LEXICAL-ANNOTATIONS.3c — per-rule lexical follow-restrictions declared via
    /// the before-rule `[> … ]` / `[>! … ]` directive. Keyed by rule name. Additive
    /// and `#[serde(default)]` so deserialization of pre-existing artifacts (which
    /// lack this key) stays backward-compatible.
    #[serde(default)]
    pub lexical_follow_restrictions: std::collections::HashMap<String, FollowRestriction>,
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
    /// LEXICAL-ANNOTATIONS.3c — the rule's before-rule follow-restriction, if any.
    lexical_follow_restriction: Option<FollowRestriction>,
}

#[derive(Debug, Clone)]
struct ExtractedRuleAnnotations {
    syntax_elements: Vec<serde_json::Value>,
    branch_return_annotations: Vec<Option<BranchAnnotation>>,
    branch_semantic_annotations: Vec<Vec<SemanticAnnotation>>,
    branch_mid_sequence_semantic_annotations: Vec<Vec<MidSequenceSemanticAnnotation>>,
    semantic_annotations: Vec<SemanticAnnotation>,
    /// LEXICAL-ANNOTATIONS.3c — the rule's before-rule follow-restriction, if any.
    lexical_follow_restriction: Option<FollowRestriction>,
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
                            // LEXICAL-ANNOTATIONS.3c — record the rule's before-rule
                            // follow-restriction. A rule may be split across multiple raw
                            // arrays (merged into one OR node above); the directive is
                            // declared once, so first-writer-wins keeps it stable.
                            if let Some(restriction) = &parsed_rule.lexical_follow_restriction {
                                annotations
                                    .lexical_follow_restrictions
                                    .entry(rule_name.clone())
                                    .or_insert_with(|| restriction.clone());
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
                || !annotations.semantic_annotations.is_empty()
                // LEXICAL-ANNOTATIONS.3c — a grammar that declares only lexical
                // follow-restrictions (no other annotations) must still surface its
                // `Annotations` so the generator can consume the restriction.
                || !annotations.lexical_follow_restrictions.is_empty())
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
                lexical_follow_restriction: None,
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
                lexical_follow_restriction: extracted.lexical_follow_restriction,
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
            lexical_follow_restriction: extracted.lexical_follow_restriction,
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
        // LEXICAL-ANNOTATIONS.3c — the rule's before-rule follow-restriction (per-rule,
        // so the stimuli generator can consume it; see the annotation consumption matrix).
        let mut lexical_follow_restriction: Option<FollowRestriction> = None;
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

        // BRANCH-BROADCAST-FIX.2 — second mapping for the WHOLE-BODY-GROUP
        // case. When the rule body is exactly one top-level parens group
        // (`RULE = ( A | B ) -> ann`), step2_group_by_or sees no top-level
        // `|`, the group's Or node is unwrapped to the rule root, and the
        // group's alternatives ARE the runtime branches. The runtime branch
        // index is then created by `|` at group_depth == 1 (inside the body
        // group), not at depth 0 — collapsing such rules to `branch_to_outer`
        // (which is constant 0 for them) is the regression that re-broke
        // task #38's parens-group trailing-annotation broadcast. Track the
        // depth<=1 mapping alongside and select per rule after the walk via
        // `syntax_is_single_whole_body_group`.
        let mut body_branch_idx = 0usize;
        let mut branch_to_body: Vec<usize> = vec![0];

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
                        if group_depth <= 1 {
                            body_branch_idx = body_branch_idx.saturating_add(1);
                        }
                        if branch_to_outer.len() <= branch_idx {
                            branch_to_outer.push(outer_branch_idx);
                        }
                        if branch_to_body.len() <= branch_idx {
                            branch_to_body.push(body_branch_idx);
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
                    let parsed_ast = self.parse_return_annotation_ast(annotation_content)?;

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
                // LEXICAL-ANNOTATIONS.3c — the before-rule follow-restriction directive
                // (`[> … ]` / `[>! … ]`). It MUST have an explicit arm here: the catch-all
                // `_ =>` below would otherwise push the token into `syntax_elements` and
                // corrupt the IR (the exact defect that got the `-0011` tokenizer-only
                // attempt reverted). The directive is per-rule, so it never participates in
                // branch/position bookkeeping.
                "lexical_annotation" => {
                    let payload = arr.get(1).and_then(FollowRestriction::from_token_payload);
                    match payload {
                        Some(restriction) => lexical_follow_restriction = Some(restriction),
                        None => {
                            return Err(anyhow::anyhow!(
                                "malformed lexical follow-restriction token in grammar IR: {:?}",
                                item
                            ));
                        }
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

        // Remap inner-indexed annotations to runtime branch indices.
        // parse_rule_content truncates these vectors to the AST's top-level
        // branch count; without this remap, inner-counted entries get lopped
        // off. The last annotation per mapped branch wins (matches existing
        // "multiple return annotations in branch — keeping last" warning
        // semantics).
        //
        // BRANCH-BROADCAST-FIX.2 — the runtime branch structure differs by
        // rule shape: for a whole-body group the runtime branches are the
        // group's alternatives (`|` at depth 1 — `branch_to_body`); for every
        // other shape they are the top-level alternatives (`|` at depth 0 —
        // `branch_to_outer`, the 2026-05-14 codegen-drop fix for patterns
        // (A)–(D) above).
        let whole_body_group = syntax_is_single_whole_body_group(&syntax_elements);
        let (branch_map, mapped_count) = if whole_body_group {
            (&branch_to_body, body_branch_idx + 1)
        } else {
            (&branch_to_outer, outer_branch_idx + 1)
        };
        let remap_returns = |inner: Vec<Option<BranchAnnotation>>| -> Vec<Option<BranchAnnotation>> {
            let mut out: Vec<Option<BranchAnnotation>> = vec![None; mapped_count];
            for (i, slot) in inner.into_iter().enumerate() {
                if let Some(ann) = slot {
                    let mapped_idx = branch_map.get(i).copied().unwrap_or(0);
                    if mapped_idx < out.len() {
                        out[mapped_idx] = Some(ann);
                    }
                }
            }
            out
        };
        let remap_vec_vec = |inner: Vec<Vec<SemanticAnnotation>>| -> Vec<Vec<SemanticAnnotation>> {
            let mut out: Vec<Vec<SemanticAnnotation>> = vec![Vec::new(); mapped_count];
            for (i, vec_anns) in inner.into_iter().enumerate() {
                if !vec_anns.is_empty() {
                    let mapped_idx = branch_map.get(i).copied().unwrap_or(0);
                    if mapped_idx < out.len() {
                        out[mapped_idx].extend(vec_anns);
                    }
                }
            }
            out
        };
        let remap_mid_seq = |inner: Vec<Vec<MidSequenceSemanticAnnotation>>| -> Vec<Vec<MidSequenceSemanticAnnotation>> {
            let mut out: Vec<Vec<MidSequenceSemanticAnnotation>> = vec![Vec::new(); mapped_count];
            for (i, vec_anns) in inner.into_iter().enumerate() {
                if !vec_anns.is_empty() {
                    let mapped_idx = branch_map.get(i).copied().unwrap_or(0);
                    if mapped_idx < out.len() {
                        out[mapped_idx].extend(vec_anns);
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
            lexical_follow_restriction,
        })
    }

    /// RGX-0078.5.i.1.t2 loud-refusal enforcement — the pure opt-in decision.
    /// A non-bootstrap pipeline built WITHOUT `--features generated_parsers` may
    /// route annotation parsing through the hand-rolled bootstrap surface ONLY
    /// under the explicit `PGEN_ALLOW_BOOTSTRAP_ANNOTATION_FALLBACK=1` opt-in
    /// (the legitimate chicken-and-egg recovery flow). Anything else refuses:
    /// the bootstrap surface silently re-interprets constructs beyond its
    /// subset (the `.5.i.1.t1` `null` → `"null"` regen-drift incident).
    /// (Compiled only where used: feature-absent builds + the unit test.)
    #[cfg(any(not(feature = "generated_parsers"), test))]
    pub(crate) fn bootstrap_annotation_fallback_allowed(opt_in: Option<&str>) -> bool {
        matches!(opt_in.map(str::trim), Some("1"))
    }

    /// RGX-0078.5.i.1.t2 — refuse the silent non-bootstrap → bootstrap
    /// annotation fallback, or (under the explicit opt-in) license it while
    /// stamping the run NON-CANONICAL with a once-per-process banner.
    #[cfg(not(feature = "generated_parsers"))]
    fn require_bootstrap_annotation_fallback_license(
        annotation_kind: &str,
        payload: &str,
    ) -> Result<()> {
        static NON_CANONICAL_BANNER: std::sync::Once = std::sync::Once::new();
        let opted_in = Self::bootstrap_annotation_fallback_allowed(
            std::env::var("PGEN_ALLOW_BOOTSTRAP_ANNOTATION_FALLBACK")
                .ok()
                .as_deref(),
        );
        if !opted_in {
            return Err(anyhow!(
                "REFUSED: {} annotation '{}' needs the generated annotation backend, but this \
                 binary was built WITHOUT `--features generated_parsers` and is not running in \
                 --bootstrap-mode. Parsing it through the hand-rolled bootstrap surface can \
                 silently re-interpret constructs beyond its subset (the RGX-0078.5.i.1.t1 \
                 `null` -> \"null\" drift incident). Either regenerate through the canonical \
                 path (`make -C rust focus_<grammar>`, whose ast_pipeline is built with \
                 `--features generated_parsers`), or set \
                 PGEN_ALLOW_BOOTSTRAP_ANNOTATION_FALLBACK=1 to accept NON-CANONICAL artifacts \
                 that MUST be re-derived canonically and pass \
                 `make -C rust parse_harness_equivalence_gate` before being trusted.",
                annotation_kind,
                payload
            ));
        }
        NON_CANONICAL_BANNER.call_once(|| {
            // Deliberately `std::eprintln!`: the module-local `eprintln!` shadow
            // routes to debug-gated tracing, and this banner must be
            // unconditional (severity is never gated by verbosity).
            std::eprintln!(
                "⚠️ PGEN NON-CANONICAL REGEN: bootstrap annotation fallback ACTIVE \
                 (PGEN_ALLOW_BOOTSTRAP_ANNOTATION_FALLBACK=1; no generated_parsers backend in \
                 this binary). Emitted artifacts are NOT canonical until re-derived via \
                 `make -C rust focus_<grammar>` and verified by \
                 `make -C rust parse_harness_equivalence_gate`."
            );
        });
        Ok(())
    }

    fn parse_return_annotation_ast(
        &self,
        annotation_content: &str,
    ) -> Result<Option<UnifiedReturnAST>> {
        let content = annotation_content.trim();
        if content.is_empty() {
            return Ok(None);
        }

        let logger = runtime_logger("pipeline.return_annotation.bootstrap");
        if !self.config.bootstrap_mode {
            if !self.validate_return_annotation_backend(content) {
                eprintln!(
                    "[mod.rs][parse_return_annotation_ast()] ⚠️ selected backend could not validate return annotation '{}'",
                    content
                );
                return Ok(None);
            }

            #[cfg(feature = "generated_parsers")]
            {
                let node_arena = NodeArena::new();
                let mut parser = Return_annotationParser::new(
                    content,
                    &node_arena,
                    runtime_logger_box("pipeline.return_annotation.generated"),
                );
                match parser.parse_full_return_annotation() {
                    Ok(parse_tree) => {
                        return match UnifiedReturnAST::parse_generated_return_annotation(
                            content,
                            &parse_tree,
                            &logger,
                        ) {
                            Ok(ast) => Ok(Some(ast)),
                            Err(err) => {
                                eprintln!(
                                    "[mod.rs][parse_return_annotation_ast()] ⚠️ generated return tree -> typed AST failed for '{}' ({})",
                                    content, err
                                );
                                Ok(None)
                            }
                        };
                    }
                    Err(err) => {
                        eprintln!(
                            "[mod.rs][parse_return_annotation_ast()] ⚠️ generated parser failed for '{}' ({})",
                            content, err
                        );
                        return Ok(None);
                    }
                }
            }

            // Reached ONLY when the generated backend is not compiled in: the
            // cfg block above returns on every path when the feature exists.
            #[cfg(not(feature = "generated_parsers"))]
            Self::require_bootstrap_annotation_fallback_license("return", content)?;
        }

        match UnifiedReturnAST::parse_bootstrap(content, &logger) {
            Ok(ast) => Ok(Some(ast)),
            Err(err) => {
                eprintln!(
                    "[mod.rs][parse_return_annotation_ast()] ⚠️ failed to build typed return AST for '{}' ({})",
                    content, err
                );
                Ok(None)
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
            let node_arena = NodeArena::new();
            let mut parser = Semantic_annotationParser::new(
                annotation_text,
                &node_arena,
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
            // RGX-0078.5.i.1.t2: same silent-fallback class as the return lane —
            // without the generated backend a named semantic annotation would
            // silently degrade to the hand-rolled `semantic_named_ast` path.
            Self::require_bootstrap_annotation_fallback_license("semantic", annotation_text)?;
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
            let node_arena = NodeArena::new();
            let mut parser = Return_annotationParser::new(
                annotation_content,
                &node_arena,
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
            let node_arena = NodeArena::new();
            let mut parser = Semantic_annotationParser::new(
                annotation_text,
                &node_arena,
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

/// BRANCH-BROADCAST-FIX.2 — true when a rule's syntax elements form exactly
/// one top-level parens group: the first syntax element is a `group_open`
/// whose matching `group_close` is the last syntax element. In that shape
/// `step2_group_by_or` sees no top-level `|`, the group's `Or` node is
/// unwrapped to the rule root, and the group's alternatives become the rule's
/// own runtime branches — so annotation branch indices must stay group-local
/// (`|` at depth 1) instead of collapsing to the single outer branch.
///
/// The input is the annotation-free `syntax_elements` token list (raw-IR
/// items like `["group_open"]`, `["operator", "|"]`, `["rule_reference", …]`)
/// that `extract_rule_annotations` accumulates; `ast_shape_contract`'s
/// cross-extractor builds the same list to share this discriminator.
pub(crate) fn syntax_is_single_whole_body_group(syntax_elements: &[serde_json::Value]) -> bool {
    fn kind_of(v: &serde_json::Value) -> Option<&str> {
        v.as_array()?.first()?.as_str()
    }
    if syntax_elements.len() < 2 {
        return false;
    }
    if kind_of(&syntax_elements[0]) != Some("group_open") {
        return false;
    }
    let mut depth = 0usize;
    for (idx, elem) in syntax_elements.iter().enumerate() {
        match kind_of(elem) {
            Some("group_open") => depth = depth.saturating_add(1),
            Some("group_close") => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    // This close matches the leading `group_open`; the body
                    // is a single whole group iff nothing follows it.
                    return idx == syntax_elements.len() - 1;
                }
            }
            _ => {
                // Any syntax at depth 0 outside the leading group (a
                // quantifier on the group, a token before/after it, a
                // top-level `|`) disqualifies the whole-body shape.
                if depth == 0 {
                    return false;
                }
            }
        }
    }
    // Unbalanced groups: stay conservative (outer mapping, the status quo).
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// RGX-0078.5.j.4 (`-0200`) — the BARE id-only guard path: bare frames
    /// maintain only `rule_id_stack`; the cycle scan sees them, the depth
    /// ceiling counts them, per-stack truncation restores each stack to its
    /// own snapshot, and with zero bare frames every surface degenerates to
    /// the paired lockstep behavior.
    #[test]
    fn recursion_guard_bare_id_only_path_semantics() {
        // 1. Paired degeneracy: no bare frames ⇒ lockstep intact and the
        //    id-aware check matches the legacy name-scan verdicts.
        let mut paired = RecursionGuard::new(4);
        paired.enter_id(1, "alpha", 0);
        paired.enter_id(2, "beta", 3);
        assert_eq!(paired.parse_stack.len(), paired.rule_id_stack.len());
        assert_eq!(paired.check_cycle_id(2, 3), CycleType::Infinite);
        assert_eq!(paired.check_cycle_id(2, 1), CycleType::LeftRecursive);
        assert_eq!(paired.check_cycle_id(3, 5), CycleType::None);

        // 2. Bare frames participate in the cycle scan exactly like paired
        //    frames (same-id same-position ⇒ Infinite; regressing ⇒ Left).
        let mut guard = RecursionGuard::new(4);
        guard.enter_id(1, "alpha", 0);
        guard.enter_id_bare(7, 2);
        assert_eq!(guard.parse_stack.len(), 1);
        assert_eq!(guard.rule_id_stack.len(), 2);
        assert_eq!(guard.check_cycle_id(7, 2), CycleType::Infinite);
        assert_eq!(guard.check_cycle_id(7, 1), CycleType::LeftRecursive);

        // 3. The depth ceiling counts the TOTAL live-frame depth (the ID
        //    stack), not the shallow name stack.
        guard.enter_id_bare(8, 4);
        guard.enter_id_bare(9, 6);
        match guard.check_cycle_id(10, 8) {
            CycleType::MutualRecursive { depth, .. } => assert_eq!(depth, 4),
            other => panic!("expected the depth ceiling at 4 total frames, got {other:?}"),
        }

        // 4. Per-stack truncation restores each stack to its OWN snapshot.
        let (name_len, id_len) = (guard.parse_stack.len(), guard.rule_id_stack.len());
        guard.enter_id(12, "extra", 9);
        guard.enter_id_bare(11, 10);
        guard.truncate_stacks(name_len, id_len);
        assert_eq!(guard.parse_stack.len(), name_len);
        assert_eq!(guard.rule_id_stack.len(), id_len);

        // 5. exit_bare pops only the ID stack, restoring the pre-bare state.
        guard.exit_bare();
        guard.exit_bare();
        guard.exit_bare();
        assert_eq!(guard.parse_stack.len(), 1);
        assert_eq!(guard.rule_id_stack.len(), 1);
        assert_eq!(guard.check_cycle_id(1, 0), CycleType::Infinite);
    }

    /// RGX-0078.5.i.1.t2 — the loud-refusal opt-in accepts EXACTLY "1"
    /// (whitespace-trimmed); everything else refuses the silent
    /// non-bootstrap → bootstrap annotation fallback.
    #[test]
    fn bootstrap_annotation_fallback_opt_in_accepts_exactly_one() {
        for allowed in [Some("1"), Some(" 1 "), Some("1\n")] {
            assert!(
                RustASTPipeline::bootstrap_annotation_fallback_allowed(allowed),
                "{allowed:?} should license the fallback"
            );
        }
        for refused in [
            None,
            Some(""),
            Some("0"),
            Some("true"),
            Some("yes"),
            Some("11"),
        ] {
            assert!(
                !RustASTPipeline::bootstrap_annotation_fallback_allowed(refused),
                "{refused:?} must refuse the fallback"
            );
        }
    }

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

    // LEXICAL-ANNOTATIONS.3c — a `lexical_annotation` token in the raw_ast is parsed
    // into a per-rule `FollowRestriction`, carried into `Annotations`, and NEVER leaks
    // into `syntax_elements` (which would corrupt the IR — the `-0011` revert cause).
    #[test]
    fn transform_from_raw_ast_carries_lexical_follow_restriction() {
        let pipeline = RustASTPipeline::new(PipelineConfig::default());
        let raw_ast_data = vec![json!([
            ["rule", "lt"],
            [
                "lexical_annotation",
                {
                    "polarity": "forbid",
                    "items": [
                        { "kind": "regex", "value": "\\w" },
                        { "kind": "literal", "value": "<" }
                    ]
                }
            ],
            ["quoted_string", "<"]
        ])];

        let (grammar_tree, _rule_order, annotations) = pipeline
            .transform_from_raw_ast(&raw_ast_data)
            .expect("raw_ast transformation should succeed");
        let annotations = annotations.expect("annotations should be preserved");

        let restriction = annotations
            .lexical_follow_restrictions
            .get("lt")
            .expect("rule `lt` should carry a follow-restriction");
        assert!(restriction.forbid, "[>! …] is a FORBID restriction");
        assert_eq!(restriction.items.len(), 2);
        assert_eq!(restriction.items[0], FollowItem::Regex("\\w".to_string()));
        assert_eq!(restriction.items[1], FollowItem::Literal("<".to_string()));

        // The directive token must NOT have leaked into the rule's syntax: `lt`'s body
        // is exactly the single `"<"` terminal (an Atom), not a sequence padded with the
        // annotation token.
        let lt_node = grammar_tree.get("lt").expect("rule `lt` should exist");
        assert!(
            matches!(lt_node, ASTNode::Atom { .. }),
            "lt body must be a single terminal Atom, got {:?}",
            lt_node
        );
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

    // BRANCH-BROADCAST-FIX.2 — whole-body parens group with a trailing
    // annotation: `item = ( "D" | "S" ) -> $text`. step2_group_by_or unwraps
    // the group's alternatives into the rule's own runtime branches, so the
    // broadcast annotation must land on EVERY runtime branch (the 2026-05-14
    // outer remap collapsed it to branch 0 only — the regression that re-broke
    // task #38's `string_literal` exemplar).
    #[test]
    fn whole_body_group_trailing_annotation_broadcasts_to_every_runtime_branch() {
        let pipeline = RustASTPipeline::new(PipelineConfig::default());
        let raw_ast_data = vec![
            json!([
                ["rule", "item"],
                ["group_open", "("],
                ["quoted_string", "D"],
                ["operator", "|"],
                ["quoted_string", "S"],
                ["group_close", ")"],
                ["return_scalar", "$text"]
            ]),
            // The defect is shape-level, not `$text`-specific: lock the
            // object form too.
            json!([
                ["rule", "other"],
                ["group_open", "("],
                ["quoted_string", "X"],
                ["operator", "|"],
                ["quoted_string", "Y"],
                ["group_close", ")"],
                ["return_object", "{kind: $1}"]
            ]),
        ];

        let (grammar_tree, _rule_order, annotations) = pipeline
            .transform_from_raw_ast(&raw_ast_data)
            .expect("raw_ast transformation should succeed");
        let annotations = annotations.expect("annotations should be preserved");

        for (rule, expected_type, expected_content) in [
            ("item", "return_scalar", "$text"),
            ("other", "return_object", "{kind: $1}"),
        ] {
            // The runtime branch structure: the group's Or is the rule root.
            let node = grammar_tree.get(rule).expect("rule should exist");
            match node {
                ASTNode::Or { alternatives } => assert_eq!(
                    alternatives.len(),
                    2,
                    "{} should unwrap to a 2-branch Or root",
                    rule
                ),
                other => panic!("{} root should be Or, got {:?}", rule, other),
            }
            let branches = annotations
                .branch_return_annotations
                .get(rule)
                .unwrap_or_else(|| panic!("{} return annotations should exist", rule));
            assert_eq!(
                branches.len(),
                2,
                "{} should carry one annotation slot per runtime branch",
                rule
            );
            for (idx, slot) in branches.iter().enumerate() {
                let ann = slot.as_ref().unwrap_or_else(|| {
                    panic!("{} branch {} should carry the broadcast annotation", rule, idx)
                });
                assert_eq!(ann.annotation_type, expected_type);
                assert_eq!(ann.annotation_content, expected_content);
            }
        }
    }

    // BRANCH-BROADCAST-FIX.2 — whole-body group with PER-BRANCH annotations
    // keeps each annotation on its own runtime branch (previously both
    // collapsed into branch 0, last one winning).
    #[test]
    fn whole_body_group_per_branch_annotations_keep_their_branches() {
        let pipeline = RustASTPipeline::new(PipelineConfig::default());
        let raw_ast_data = vec![json!([
            ["rule", "lit"],
            ["group_open", "("],
            ["quoted_string", "a"],
            ["return_scalar", "$1"],
            ["operator", "|"],
            ["quoted_string", "b"],
            ["return_scalar", "$2"],
            ["group_close", ")"]
        ])];

        let (_grammar_tree, _rule_order, annotations) = pipeline
            .transform_from_raw_ast(&raw_ast_data)
            .expect("raw_ast transformation should succeed");
        let annotations = annotations.expect("annotations should be preserved");

        let branches = annotations
            .branch_return_annotations
            .get("lit")
            .expect("lit return annotations should exist");
        assert_eq!(branches.len(), 2);
        assert_eq!(
            branches[0].as_ref().map(|a| a.annotation_content.as_str()),
            Some("$1")
        );
        assert_eq!(
            branches[1].as_ref().map(|a| a.annotation_content.as_str()),
            Some("$2")
        );
    }

    // BRANCH-BROADCAST-FIX.2 — the documented disambiguations stay intact:
    // `(A|B) | C -> ann` binds ann to C only (the annotation does not follow
    // a group_close), and `A | (B|C) -> ann` binds ann to the outer branch
    // holding the group (broadcast within that branch's nested Or).
    #[test]
    fn mixed_and_trailing_group_annotation_disambiguation_is_unchanged() {
        let pipeline = RustASTPipeline::new(PipelineConfig::default());
        let raw_ast_data = vec![
            // mixed = ( a | b ) | c -> ann
            json!([
                ["rule", "mixed"],
                ["group_open", "("],
                ["rule_reference", "a"],
                ["operator", "|"],
                ["rule_reference", "b"],
                ["group_close", ")"],
                ["operator", "|"],
                ["rule_reference", "c"],
                ["return_scalar", "$1"]
            ]),
            // trailing = a | ( b | c ) -> ann
            json!([
                ["rule", "trailing"],
                ["rule_reference", "a"],
                ["operator", "|"],
                ["group_open", "("],
                ["rule_reference", "b"],
                ["operator", "|"],
                ["rule_reference", "c"],
                ["group_close", ")"],
                ["return_scalar", "$1"]
            ]),
        ];

        let (_grammar_tree, _rule_order, annotations) = pipeline
            .transform_from_raw_ast(&raw_ast_data)
            .expect("raw_ast transformation should succeed");
        let annotations = annotations.expect("annotations should be preserved");

        for rule in ["mixed", "trailing"] {
            let branches = annotations
                .branch_return_annotations
                .get(rule)
                .unwrap_or_else(|| panic!("{} return annotations should exist", rule));
            assert_eq!(branches.len(), 2, "{} has 2 top-level branches", rule);
            assert!(
                branches[0].is_none(),
                "{} branch 0 must not carry the annotation",
                rule
            );
            assert_eq!(
                branches[1].as_ref().map(|a| a.annotation_content.as_str()),
                Some("$1"),
                "{} branch 1 must carry the annotation",
                rule
            );
        }
    }

    // BRANCH-BROADCAST-FIX.2 — the 2026-05-14 codegen-drop patterns (A)–(D)
    // stay green: groups that are SUB-PARTS of a sequence (not the whole
    // body) keep the outer remap so inner-indexed annotations are not lopped
    // off by the runtime-branch truncation.
    #[test]
    fn inner_group_remap_patterns_a_through_d_stay_green() {
        let pipeline = RustASTPipeline::new(PipelineConfig::default());
        let raw_ast_data = vec![
            // (A) id ( a | b )* -> ann   — single-branch rule, trailing group
            //     quantifier; annotation is rule-level (outer branch 0).
            json!([
                ["rule", "pat_a"],
                ["rule_reference", "id"],
                ["group_open", "("],
                ["rule_reference", "a"],
                ["operator", "|"],
                ["rule_reference", "b"],
                ["group_close", ")"],
                ["operator", "*"],
                ["return_object", "{kind: $1}"]
            ]),
            // (B) ( a | b | c )? id -> ann — leading optional group.
            json!([
                ["rule", "pat_b"],
                ["group_open", "("],
                ["rule_reference", "a"],
                ["operator", "|"],
                ["rule_reference", "b"],
                ["operator", "|"],
                ["rule_reference", "c"],
                ["group_close", ")"],
                ["operator", "?"],
                ["rule_reference", "id"],
                ["return_object", "{kind: $2}"]
            ]),
            // (C) x | y | ( a )? id -> ann — annotation on the LAST top-level
            //     branch, which contains an inner optional group.
            json!([
                ["rule", "pat_c"],
                ["rule_reference", "x"],
                ["operator", "|"],
                ["rule_reference", "y"],
                ["operator", "|"],
                ["group_open", "("],
                ["rule_reference", "a"],
                ["group_close", ")"],
                ["operator", "?"],
                ["rule_reference", "id"],
                ["return_object", "{kind: $2}"]
            ]),
            // (D) ( a | b )? id -> ann0 | z -> ann1 — per-branch annotations
            //     in a multi-branch rule whose first branch holds a group.
            json!([
                ["rule", "pat_d"],
                ["group_open", "("],
                ["rule_reference", "a"],
                ["operator", "|"],
                ["rule_reference", "b"],
                ["group_close", ")"],
                ["operator", "?"],
                ["rule_reference", "id"],
                ["return_object", "{kind: $2}"],
                ["operator", "|"],
                ["rule_reference", "z"],
                ["return_object", "{kind: $1}"]
            ]),
        ];

        let (_grammar_tree, _rule_order, annotations) = pipeline
            .transform_from_raw_ast(&raw_ast_data)
            .expect("raw_ast transformation should succeed");
        let annotations = annotations.expect("annotations should be preserved");

        // (A): one runtime branch, annotation present at branch 0.
        let pat_a = annotations
            .branch_return_annotations
            .get("pat_a")
            .expect("pat_a annotations");
        assert_eq!(pat_a.len(), 1);
        assert_eq!(
            pat_a[0].as_ref().map(|a| a.annotation_content.as_str()),
            Some("{kind: $1}")
        );

        // (B): one runtime branch, annotation present at branch 0.
        let pat_b = annotations
            .branch_return_annotations
            .get("pat_b")
            .expect("pat_b annotations");
        assert_eq!(pat_b.len(), 1);
        assert_eq!(
            pat_b[0].as_ref().map(|a| a.annotation_content.as_str()),
            Some("{kind: $2}")
        );

        // (C): three top-level branches, annotation on the last only.
        let pat_c = annotations
            .branch_return_annotations
            .get("pat_c")
            .expect("pat_c annotations");
        assert_eq!(pat_c.len(), 3);
        assert!(pat_c[0].is_none());
        assert!(pat_c[1].is_none());
        assert_eq!(
            pat_c[2].as_ref().map(|a| a.annotation_content.as_str()),
            Some("{kind: $2}")
        );

        // (D): two top-level branches, each keeping its own annotation.
        let pat_d = annotations
            .branch_return_annotations
            .get("pat_d")
            .expect("pat_d annotations");
        assert_eq!(pat_d.len(), 2);
        assert_eq!(
            pat_d[0].as_ref().map(|a| a.annotation_content.as_str()),
            Some("{kind: $2}")
        );
        assert_eq!(
            pat_d[1].as_ref().map(|a| a.annotation_content.as_str()),
            Some("{kind: $1}")
        );
    }

    // BRANCH-BROADCAST-FIX.2 — the whole-body-group discriminator itself.
    #[test]
    fn whole_body_group_discriminator_classifies_token_shapes() {
        let tok = |kind: &str| json!([kind]);
        let tok1 = |kind: &str, val: &str| json!([kind, val]);

        // ( A | B )  → whole body.
        assert!(syntax_is_single_whole_body_group(&[
            tok("group_open"),
            tok1("rule_reference", "a"),
            tok1("operator", "|"),
            tok1("rule_reference", "b"),
            tok("group_close"),
        ]));
        // ( A | B ) ? → quantified group, NOT whole body.
        assert!(!syntax_is_single_whole_body_group(&[
            tok("group_open"),
            tok1("rule_reference", "a"),
            tok1("operator", "|"),
            tok1("rule_reference", "b"),
            tok("group_close"),
            tok1("operator", "?"),
        ]));
        // ( A | B ) | C → top-level alternation, NOT whole body.
        assert!(!syntax_is_single_whole_body_group(&[
            tok("group_open"),
            tok1("rule_reference", "a"),
            tok1("operator", "|"),
            tok1("rule_reference", "b"),
            tok("group_close"),
            tok1("operator", "|"),
            tok1("rule_reference", "c"),
        ]));
        // id ( A | B ) → leading token, NOT whole body.
        assert!(!syntax_is_single_whole_body_group(&[
            tok1("rule_reference", "id"),
            tok("group_open"),
            tok1("rule_reference", "a"),
            tok1("operator", "|"),
            tok1("rule_reference", "b"),
            tok("group_close"),
        ]));
        // ( ( A | B ) | C ) → nested groups, whole body (outermost spans all).
        assert!(syntax_is_single_whole_body_group(&[
            tok("group_open"),
            tok("group_open"),
            tok1("rule_reference", "a"),
            tok1("operator", "|"),
            tok1("rule_reference", "b"),
            tok("group_close"),
            tok1("operator", "|"),
            tok1("rule_reference", "c"),
            tok("group_close"),
        ]));
        // ( A | B )( C | D ) → two adjacent groups, NOT whole body.
        assert!(!syntax_is_single_whole_body_group(&[
            tok("group_open"),
            tok1("rule_reference", "a"),
            tok1("operator", "|"),
            tok1("rule_reference", "b"),
            tok("group_close"),
            tok("group_open"),
            tok1("rule_reference", "c"),
            tok1("operator", "|"),
            tok1("rule_reference", "d"),
            tok("group_close"),
        ]));
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
pub mod pgen_value;
// SV-EXH-PROOF.3.3.4.b.6.2.22 — live per-rule call-counter dashboard.
pub mod call_count_dashboard;
pub mod ast_code_generator;
pub mod ast_generator_direct;
pub mod ast_return_transform;
pub mod first_set;
// RGX-0078.5.h.1 — STEP-0 fusibility census (read-only capability-gate classifier).
pub mod fusibility_census;
pub mod grammar_wellformedness;
pub mod grouped_quantifier_parser;
pub mod library;
pub mod mutual_recursion_handler;
pub mod predicate_expr;
pub use predicate_expr::{
    CompareOp, PredicateDef, PredicateExpr, PredicateValue, PrimitiveCall,
    parse_predicate_expression,
};
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
    CompiledSemanticRuntimeAnnotations, FactFilter, FactKindDecl, LayoutSensitivity, QueryExplain,
    ResolveResult, RollbackLabel, ScopeId, ScopeNode, SemanticCloseScopeSpec, SemanticFactRecord,
    SemanticFactSpec, SemanticLibraryExportSpec, SemanticLibraryImportSpec,
    SemanticPredicateContentView, SemanticPredicatePhase, SemanticPredicateSpec,
    SemanticRuntimeCheckpoint, SemanticRuntimeDelta, SemanticRuntimeDirective,
    SemanticRuntimeState, SemanticRuntimeTransaction, SemanticRuntimeValue, SemanticScopeFrame,
    SemanticScopeKind, SemanticScopeSpec, SemanticStoreCounters, compile_default_profile,
    compile_layout_sensitivity, compile_profile_aliases,
    compile_rule_semantic_runtime_directives, compile_semantic_runtime_annotations,
    parse_semantic_runtime_directive, parse_semantic_runtime_directives,
};
pub use semantic_transform::{
    CanonicalSemanticTransform, parse_canonical_transform_expression, stimuli_hint_for_target_type,
};
pub use unified_return_ast::{ExtractionTarget, UnifiedReturnAST};
pub use unified_semantic_ast::{UnifiedSemanticAST, UnifiedSemanticProperty, UnifiedSemanticValue};
