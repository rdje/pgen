//! SystemVerilog preprocessor execution stage (baseline) — a COMPONENT of the SystemVerilog
//! family (`ENGINE-UNIVERSAL-SERVICES.113`): it lives beside its family, may name its own
//! family and nothing else, and the engine reaches it only through the neutral
//! [`crate::preprocessing_stage::PreprocessingStage`] the family declares in `super::HOOKS`.
//!
//! This module provides a deterministic, file-based preprocessing pass:
//! - macro define/undef and expansion (object-like + function-like),
//! - include resolution with bounded depth and cycle detection,
//! - conditional-compilation flow (`ifdef/`ifndef/`elsif/`else/`endif),
//! - source mapping metadata from expanded output back to originating file/line.

use super::keyword_versions::{
    KeywordVersion, profile_admits_specifier, region_within_profile, version_for_profile,
    words_to_escape,
};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SvPreprocessorConfig {
    pub include_dirs: Vec<PathBuf>,
    pub max_include_depth: usize,
    pub include_path_policy: IncludePathPolicy,
    pub macro_redefinition_policy: MacroRedefinitionPolicy,
    pub conditional_symbol_policy: ConditionalSymbolPolicy,
    pub conditional_expr_policy: ConditionalExprPolicy,
    /// `SVPP-EXPANSION.5`(k): a formal inside a plain `"…"` string in macro text (default: the standard's opaque string)
    pub plain_string_substitution: PlainStringSubstitutionPolicy,
    pub strict_warning_codes: HashSet<String>,
    /// `SVPP-EXPANSION.6f`: what a backtick that SURVIVES expansion outside a comment or string means —
    /// `Error` (the default): an undefined macro, a function-like macro without its argument list, a
    /// stray macro-string token or an unknown directive is refused with its clause, because the
    /// expanded text is otherwise handed to a parser whose raw-lane residue would ACCEPT a top-level
    /// backtick line; `Keep`: the old passthrough, for a consumer that resolves the rest itself.
    pub undefined_macro_policy: UndefinedMacroPolicy,
    /// `SVPP-EXPANSION.2`(a) — the compilation context's PREDEFINED macros (the `+define+` of every
    /// commercial front end). Each one is seeded into the macro table before the first source line
    /// is read, exactly as if `` `define NAME TEXT `` stood at the top of the compilation unit — so
    /// the redefinition policy, `` `undef `` and `` `ifdef `` all see it as an ordinary definition.
    pub defines: Vec<PredefinedMacro>,
    /// `SVPP-EXPANSION.8` — the `--profile` spelling the EXPANDED text will be parsed under, so
    /// this stage knows which identifiers the parser reserves and can render a
    /// `` `begin_keywords `` region's non-keywords as escaped identifiers (IEEE 1800-2017 §22.14).
    /// `None` is the parser's unprofiled reading, which reserves IEEE 1800-2017 Table B.1; see
    /// [`super::keyword_versions::version_for_profile`]. Since `SVPP-EXPANSION.5`(p) the EDITION behind
    /// the profile also selects the three readings where IEEE 1800-2023 says more than 1800-2017 — see
    /// [`reads_1800_2023`]: a block comment in macro text, a block comment that continues past the
    /// newline of a `` `define ``, and a macro usage as the argument of a conditional directive.
    pub parse_profile: Option<String>,
}

impl Default for SvPreprocessorConfig {
    fn default() -> Self {
        Self {
            include_dirs: Vec::new(),
            max_include_depth: 64,
            include_path_policy: IncludePathPolicy::AllowAbsolute,
            macro_redefinition_policy: MacroRedefinitionPolicy::Allow,
            conditional_symbol_policy: ConditionalSymbolPolicy::AssumeFalseSilent,
            conditional_expr_policy: ConditionalExprPolicy::IdentifierOrDefined,
            plain_string_substitution: PlainStringSubstitutionPolicy::Standard,
            strict_warning_codes: HashSet::new(),
            undefined_macro_policy: UndefinedMacroPolicy::Error,
            defines: Vec::new(),
            parse_profile: None,
        }
    }
}

/// `SVPP-EXPANSION.6f`: see [`SvPreprocessorConfig::undefined_macro_policy`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndefinedMacroPolicy {
    Error,
    Keep,
}

impl UndefinedMacroPolicy {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "error" | "refuse" => Some(Self::Error),
            "keep" | "passthrough" => Some(Self::Keep),
            _ => None,
        }
    }
}

/// A macro supplied by the compilation context rather than by source text (`SVPP-EXPANSION.2`(a)).
///
/// Object-like only: a predefine carries no formal-argument list, matching what `+define+NAME=TEXT`
/// means in every tool that accepts it. `NAME` alone defines the macro with EMPTY text — the same
/// meaning a bare `` `define NAME `` line has in the source (IEEE 1800-2017 §22.5.1: the macro text
/// may be empty), so `` `ifdef NAME `` is true and `` `NAME `` expands to nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PredefinedMacro {
    pub name: String,
    pub text: String,
}

impl PredefinedMacro {
    /// Parse the `NAME` / `NAME=TEXT` spelling of a command-line define. The text keeps its bytes
    /// verbatim after the first `=` (a second `=` is part of the text). The name must be a legal
    /// text-macro identifier; a parenthesised formal list is refused rather than silently read as a
    /// name, because a function-like predefine has no command-line meaning anywhere.
    pub fn parse(raw: &str) -> Result<Self> {
        let raw = raw.trim();
        let (name, text) = match raw.split_once('=') {
            Some((name, text)) => (name.trim(), text),
            None => (raw, ""),
        };
        if name.is_empty() {
            bail!("a predefined macro needs a name: got '{}'", raw);
        }
        if !is_valid_macro_symbol(name) {
            bail!(
                "'{}' is not a legal text-macro name for a predefined macro (expected NAME or NAME=TEXT; \
                 a formal-argument list is not accepted on the command line)",
                name
            );
        }
        Ok(Self {
            name: name.to_string(),
            text: text.to_string(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncludePathPolicy {
    AllowAbsolute,
    RelativeOnly,
}

impl IncludePathPolicy {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "allow_absolute" | "allow-absolute" | "allowabsolute" => Some(Self::AllowAbsolute),
            "relative_only" | "relative-only" | "relativeonly" => Some(Self::RelativeOnly),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroRedefinitionPolicy {
    Allow,
    Warn,
    Error,
}

impl MacroRedefinitionPolicy {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "allow" => Some(Self::Allow),
            "warn" | "warning" => Some(Self::Warn),
            "error" | "strict" => Some(Self::Error),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionalSymbolPolicy {
    AssumeFalseSilent,
    AssumeFalseWarn,
    Error,
}

impl ConditionalSymbolPolicy {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "assume_false_silent" | "assume-false-silent" | "silent" => {
                Some(Self::AssumeFalseSilent)
            }
            "assume_false_warn" | "assume-false-warn" | "warn" | "warning" => {
                Some(Self::AssumeFalseWarn)
            }
            "error" | "strict" => Some(Self::Error),
            _ => None,
        }
    }
}

/// `SVPP-EXPANSION.5`(k): what a FORMAL inside a plain `"…"` string literal in macro text means.
/// IEEE 1800-2017 §22.5.1 (`:370-372`): *"the use of " always starts a string literal … Any
/// characters embedded inside this string literal, including `", become part of the string in the
/// replaced macro text"* — and its own example `` `define H(x) "Hello, x" `` prints `Hello, x`.
/// The `` `" `` form exists because a plain string does NOT substitute. Verilator 5.050 reads it so;
/// Icarus 13.0 substitutes every formal, and so did this engine, which MANGLES a message whose prose
/// happens to contain a formal's name (`"  expected %0h, got %0h"` with a formal `expected`,
/// `"in %0d fast clock cycles"` with `cycles` — 66 of the corpus's 72 such sites). Design code
/// written for commercial simulators relies on the other 6: OpenTitan's paste-delimited idiom
/// `"tb.dut.gen_classes[``i``].u_accu…"`, where the pastes MARK the formal. Three readings, each a
/// measured population; the standard's is the default, the other two are named tool extensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlainStringSubstitutionPolicy {
    /// the standard: a plain string in macro text is opaque — no formal substituted, no paste joined
    #[default]
    Standard,
    /// OpenTitan's idiom: only a formal DELIMITED by paste tokens on both sides (`` ``i`` ``) is
    /// substituted, the pastes removed; a bare word in prose stays prose
    PasteDelimited,
    /// Icarus's reading (this engine's until `.5`(k)): every formal substituted, every paste joined
    All,
}

impl PlainStringSubstitutionPolicy {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "standard" | "lrm" | "opaque" => Some(Self::Standard),
            "paste_delimited" | "paste-delimited" | "delimited" => Some(Self::PasteDelimited),
            "all" | "substitute" | "icarus" => Some(Self::All),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionalExprPolicy {
    IdentifierOnly,
    IdentifierOrDefined,
}

impl ConditionalExprPolicy {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "identifier_only" | "identifier-only" | "identifieronly" => Some(Self::IdentifierOnly),
            "identifier_or_defined" | "identifier-or-defined" | "identifierordefined" => {
                Some(Self::IdentifierOrDefined)
            }
            _ => None,
        }
    }
}

// The four output shapes are the ENGINE's neutral stage types (`crate::preprocessing_stage`),
// used here under the names this module always had: the JSON this stage writes is byte-identical
// to what it wrote before the stage interface existed (`.113`).
pub use crate::preprocessing_stage::{
    SourceLocation, SourceMapEntry, StageDiagnostic as PreprocessorDiagnostic,
    StageDiagnosticSeverity as PreprocessorDiagnosticSeverity,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PreprocessorEventKind {
    Define,
    Undef,
    Include,
    Ifdef,
    Ifndef,
    Elsif,
    Else,
    Endif,
    PassthroughLine,
    SkippedLine,
    MacroExpand,
    /// `SVPP-EXPANSION.6b.2`(ii): a well-formed `` `line `` directive consumed (§22.12); the detail
    /// carries the number and the file name it sets for the following lines.
    LineDirective,
    /// `SVPP-EXPANSION.6b.5`(d): a §34 protected envelope opened (`begin_protected`) or closed
    /// (`end_protected`); every line between the two was consumed as the envelope's, not source text.
    ProtectedEnvelope,
    /// `SVPP-EXPANSION.8`: a §22.14 `` `begin_keywords `` region opened, closed, or left open at the
    /// end of the compilation unit; the detail carries the version specifier and, on the closing
    /// event, how many tokens the region rendered as escaped identifiers.
    KeywordsRegion,
}

impl PreprocessorEventKind {
    /// The kind's serde spelling — what [`crate::preprocessing_stage::StageEvent::kind`] carries,
    /// so a converted event log serializes byte-identically (pinned by a test over every variant).
    pub fn as_str(&self) -> &'static str {
        match self {
            PreprocessorEventKind::Define => "define",
            PreprocessorEventKind::Undef => "undef",
            PreprocessorEventKind::Include => "include",
            PreprocessorEventKind::Ifdef => "ifdef",
            PreprocessorEventKind::Ifndef => "ifndef",
            PreprocessorEventKind::Elsif => "elsif",
            PreprocessorEventKind::Else => "else",
            PreprocessorEventKind::Endif => "endif",
            PreprocessorEventKind::PassthroughLine => "passthrough_line",
            PreprocessorEventKind::SkippedLine => "skipped_line",
            PreprocessorEventKind::MacroExpand => "macro_expand",
            PreprocessorEventKind::LineDirective => "line_directive",
            PreprocessorEventKind::ProtectedEnvelope => "protected_envelope",
            PreprocessorEventKind::KeywordsRegion => "keywords_region",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreprocessorEvent {
    pub kind: PreprocessorEventKind,
    pub file: String,
    pub line: usize,
    pub active: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessedOutput {
    pub text: String,
    pub source_map: Vec<SourceMapEntry>,
    pub events: Vec<PreprocessorEvent>,
    pub diagnostics: Vec<PreprocessorDiagnostic>,
    pub included_files: Vec<String>,
}

/// `.113`: this family's output as the engine's neutral [`crate::preprocessing_stage::StageOutput`]
/// — text, source map, diagnostics and the included-file list move; only the event kind is
/// re-spelled (its serde name), so every JSON artifact is byte-identical.
impl From<PreprocessedOutput> for crate::preprocessing_stage::StageOutput {
    fn from(output: PreprocessedOutput) -> Self {
        crate::preprocessing_stage::StageOutput {
            text: output.text,
            source_map: output.source_map,
            events: output
                .events
                .into_iter()
                .map(|event| crate::preprocessing_stage::StageEvent {
                    kind: event.kind.as_str().to_string(),
                    file: event.file,
                    line: event.line,
                    active: event.active,
                    detail: event.detail,
                })
                .collect(),
            diagnostics: output.diagnostics,
            included_files: output.included_files,
        }
    }
}

#[derive(Debug, Clone)]
struct MacroParameter {
    name: String,
    default: Option<String>,
}

#[derive(Debug, Clone)]
struct MacroDefinition {
    params: Option<Vec<MacroParameter>>,
    body: String,
}

#[derive(Debug, Clone)]
struct ConditionalFrame {
    parent_active: bool,
    current_active: bool,
    branch_taken: bool,
}

#[derive(Debug)]
struct PreprocessorState {
    config: SvPreprocessorConfig,
    macros: HashMap<String, MacroDefinition>,
    output: String,
    source_map: Vec<SourceMapEntry>,
    events: Vec<PreprocessorEvent>,
    diagnostics: Vec<PreprocessorDiagnostic>,
    include_stack: Vec<PathBuf>,
    included_files: Vec<String>,
    included_seen: HashSet<String>,
    /// `SVPP-EXPANSION.8`: the open `` `begin_keywords `` regions, innermost last. §22.14: the
    /// directive *"affects all source code that follows the directive, even across source code file
    /// boundaries, until the matching `` `end_keywords `` directive or the end of the compilation
    /// unit"*, and the pairs *"can be nested … Each nested pair is stacked"* — so this is unit
    /// state, carried between the files of a unit exactly as the macro table is.
    keyword_regions: Vec<KeywordVersion>,
}

/// IEEE 1800-2017 §40.3.1's predefined coverage-control macros and their values (the clause's
/// text at `section-40-code-coverage-control-and-api.md:96-140`).
const SV_COV_PREDEFINED_MACROS: &[(&str, &str)] = &[
    ("SV_COV_START", "0"),
    ("SV_COV_STOP", "1"),
    ("SV_COV_RESET", "2"),
    ("SV_COV_CHECK", "3"),
    ("SV_COV_MODULE", "10"),
    ("SV_COV_HIER", "11"),
    ("SV_COV_ASSERTION", "20"),
    ("SV_COV_FSM_STATE", "21"),
    ("SV_COV_STATEMENT", "22"),
    ("SV_COV_TOGGLE", "23"),
    ("SV_COV_OVERFLOW", "-2"),
    ("SV_COV_ERROR", "-1"),
    ("SV_COV_NOCOV", "0"),
    ("SV_COV_OK", "1"),
    ("SV_COV_PARTIAL", "2"),
];

impl PreprocessorState {
    fn new(config: SvPreprocessorConfig) -> Self {
        // `SVPP-EXPANSION.2`(a): the context's predefines ARE the macro table's initial contents.
        // Seeded in declaration order, so a later `NAME=…` on the command line supersedes an
        // earlier one the way a later `` `define `` supersedes an earlier one under the default
        // redefinition policy — the policy itself is not consulted here, because these lines have
        // no source position to report a redefinition at.
        let mut macros = HashMap::new();
        // `SVPP-EXPANSION.6b.5`(c): IEEE 1800-2017 §40.3.1 — "The following predefined `define macros
        // represent basic real-time coverage capabilities accessible directly from SystemVerilog"
        // (section-40-code-coverage-control-and-api.md:96-140): the TOOL defines them, no include
        // does; a context predefine or a source `` `define `` of the same name supersedes (below).
        for (name, value) in SV_COV_PREDEFINED_MACROS {
            macros.insert(
                name.to_string(),
                MacroDefinition {
                    params: None,
                    body: value.to_string(),
                },
            );
        }
        for predefined in &config.defines {
            macros.insert(
                predefined.name.clone(),
                MacroDefinition {
                    params: None,
                    body: predefined.text.clone(),
                },
            );
        }
        Self {
            config,
            macros,
            output: String::new(),
            source_map: Vec::new(),
            events: Vec::new(),
            diagnostics: Vec::new(),
            include_stack: Vec::new(),
            included_files: Vec::new(),
            included_seen: HashSet::new(),
            keyword_regions: Vec::new(),
        }
    }

    /// Hand this state's macro table to the next file of the same compilation unit
    /// (`SVPP-EXPANSION.2`(b)). Everything else — output text, source map, events, diagnostics,
    /// the include stack and the included-file list — is per FILE and starts fresh.
    fn into_carried_macros(self) -> HashMap<String, MacroDefinition> {
        self.macros
    }

    fn take_output(&mut self) -> PreprocessedOutput {
        PreprocessedOutput {
            text: std::mem::take(&mut self.output),
            source_map: std::mem::take(&mut self.source_map),
            events: std::mem::take(&mut self.events),
            diagnostics: std::mem::take(&mut self.diagnostics),
            included_files: std::mem::take(&mut self.included_files),
        }
    }

    fn push_event(
        &mut self,
        kind: PreprocessorEventKind,
        file: &Path,
        line: usize,
        active: bool,
        detail: impl Into<String>,
    ) {
        self.events.push(PreprocessorEvent {
            kind,
            file: file.display().to_string(),
            line,
            active,
            detail: detail.into(),
        });
    }

    fn append_output_chunk(
        &mut self,
        chunk: &str,
        file: &Path,
        line: usize,
        column_start: usize,
        column_end: usize,
    ) {
        if chunk.is_empty() {
            return;
        }
        let start = self.output.len();
        self.output.push_str(chunk);
        let end = self.output.len();
        self.source_map.push(SourceMapEntry {
            output_start: start,
            output_end: end,
            source: SourceLocation {
                file: file.display().to_string(),
                line,
                column_start,
                column_end,
            },
        });
    }

    fn record_include_file(&mut self, path: &Path) {
        let key = path.display().to_string();
        if self.included_seen.insert(key.clone()) {
            self.included_files.push(key);
        }
    }

    fn warning_is_promoted(&self, code: &str) -> bool {
        self.config.strict_warning_codes.contains("*")
            || self
                .config
                .strict_warning_codes
                .contains(&code.to_ascii_uppercase())
    }

    fn push_warning(
        &mut self,
        code: &str,
        file: &Path,
        line: usize,
        message: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<()> {
        let promoted = self.warning_is_promoted(code);
        let severity = if promoted {
            PreprocessorDiagnosticSeverity::Error
        } else {
            PreprocessorDiagnosticSeverity::Warning
        };
        let diagnostic = PreprocessorDiagnostic {
            code: code.to_string(),
            severity: severity.clone(),
            file: file.display().to_string(),
            line,
            message: message.into(),
            detail: detail.into(),
        };
        self.diagnostics.push(diagnostic.clone());
        if promoted {
            bail!(
                "{} at {}:{}: {} ({})",
                diagnostic.code,
                diagnostic.file,
                diagnostic.line,
                diagnostic.message,
                diagnostic.detail
            );
        }
        Ok(())
    }

    /// `SVPP-EXPANSION.8` — IEEE 1800-2017 §22.14 applied to the expanded text.
    ///
    /// §22.14 selects, per REGION, which identifiers are reserved. The generated parser reserves one
    /// set per PROFILE and cannot switch mid-file, so the difference is rendered here as TEXT:
    /// inside a region, every token the parse profile reserves and the region's version does not is
    /// re-emitted as an escaped identifier — `logic` becomes `\logic `, which §5.6.2 makes "not
    /// interpreted as a keyword" and §5.6.1 makes the same identifier as the unescaped spelling. The
    /// reverse difference cannot be rendered — text cannot create a keyword — so a region naming a
    /// version RICHER than the profile is reported rather than half-applied.
    ///
    /// The pass reads the EXPANDED text, so a directive produced by a macro counts and one inside an
    /// inactive `` `ifdef `` branch does not: it never reached the output. Comments, string
    /// literals, escaped identifiers, numeric literals and `$`-names are skipped, so a keyword named
    /// inside any of them is left alone. The source map is re-indexed by the exact byte delta of
    /// each rendering, so an origin lookup still lands on the line the reader wrote.
    fn render_keyword_regions(&mut self) -> Result<()> {
        let text = std::mem::take(&mut self.output);
        // The whole corpus pays only these two substring searches: a file with no §22.14 directive
        // and no region carried in from an earlier file of the unit cannot render anything, and an
        // unmatched `` `end_keywords `` must still be seen, so both spellings are looked for.
        if self.keyword_regions.is_empty()
            && !text.contains("`begin_keywords")
            && !text.contains("`end_keywords")
        {
            self.output = text;
            return Ok(());
        }
        let profile_name = self.config.parse_profile.clone();
        let profile = version_for_profile(profile_name.as_deref());

        let bytes = text.as_bytes();
        let mut idx = 0usize;
        let mut replacements: Vec<(usize, usize, String)> = Vec::new();
        let mut warnings: Vec<(&'static str, PathBuf, usize, String, String)> = Vec::new();
        let mut events: Vec<(PathBuf, usize, String)> = Vec::new();
        // (the region's version, how many tokens it has rendered in THIS file)
        let mut unknown_open: usize = 0;
        let mut open: Vec<(KeywordVersion, usize)> = self
            .keyword_regions
            .iter()
            .map(|version| (*version, 0usize))
            .collect();
        let mut unreserved: Option<BTreeSet<&'static str>> = open
            .last()
            .map(|(version, _)| words_to_escape(profile, *version));

        while idx < bytes.len() {
            match bytes[idx] {
                b'/' if bytes.get(idx + 1) == Some(&b'/') => {
                    idx = text[idx..].find('\n').map_or(bytes.len(), |at| idx + at);
                }
                b'/' if bytes.get(idx + 1) == Some(&b'*') => {
                    idx = text[idx + 2..]
                        .find("*/")
                        .map_or(bytes.len(), |at| idx + 2 + at + 2);
                }
                b'"' => {
                    idx += 1;
                    while idx < bytes.len() {
                        match bytes[idx] {
                            b'\\' => idx += 2,
                            b'"' => {
                                idx += 1;
                                break;
                            }
                            // an unterminated literal ends at its line, never swallows the file
                            b'\n' => break,
                            _ => idx += 1,
                        }
                    }
                }
                // §5.6.1: an escaped identifier already in the text runs to the next white space
                b'\\' => {
                    idx += 1;
                    while idx < bytes.len() && (0x21..=0x7e).contains(&bytes[idx]) {
                        idx += 1;
                    }
                }
                // a based or unsized literal — `4'b1010`, `'sd5`, `'0` — is one token, not two
                b'\'' => {
                    idx += 1;
                    if matches!(bytes.get(idx), Some(b's') | Some(b'S')) {
                        idx += 1;
                    }
                    if matches!(
                        bytes.get(idx).map(u8::to_ascii_lowercase),
                        Some(b'b') | Some(b'o') | Some(b'd') | Some(b'h')
                    ) {
                        idx += 1;
                    }
                    while idx < bytes.len()
                        && (bytes[idx].is_ascii_alphanumeric()
                            || bytes[idx] == b'_'
                            || bytes[idx] == b'?')
                    {
                        idx += 1;
                    }
                }
                // §5.6.3: a name after `$` is a system task or function, never an identifier
                b'$' => {
                    idx += 1;
                    while idx < bytes.len() && is_ident_continue(bytes[idx]) {
                        idx += 1;
                    }
                }
                b'`' => {
                    idx += 1;
                    let name_start = idx;
                    while idx < bytes.len() && is_ident_continue(bytes[idx]) {
                        idx += 1;
                    }
                    let directive = &text[name_start..idx];
                    if directive != "begin_keywords" && directive != "end_keywords" {
                        continue;
                    }
                    let (file, line) = self
                        .origin_of_output_offset(name_start)
                        .unwrap_or_else(|| (PathBuf::from("<expanded>"), 0));
                    if directive == "end_keywords" {
                        match open.pop() {
                            Some((region, escaped)) => events.push((
                                file,
                                line,
                                format!("end_keywords \"{}\" escaped={escaped}", region.specifier()),
                            )),
                            None if unknown_open > 0 => {
                                // closes a region whose specifier the stage did not recognise: the
                                // parser refuses the `begin_keywords line, this directive pairs with it
                                unknown_open -= 1;
                            }
                            // `SVPP-EXPANSION.6f`: refused, not left for the parser (which reads the line as
                            // a directive and ACCEPTED verilator t_preproc_kwd_bad.v)
                            None => bail!(
                                "`end_keywords closes no open `begin_keywords region at {}:{} — IEEE 1364-2005 \
                                 §19.11: \"Each `begin_keywords directive must be paired with an `end_keywords \
                                 directive\"; IEEE 1800-2017 §22.14 speaks of \"the matching `end_keywords directive\"",
                                file.display(),
                                line
                            ),
                        }
                        unreserved = open
                            .last()
                            .map(|(version, _)| words_to_escape(profile, *version));
                        continue;
                    }
                    let (specifier, after) = read_version_specifier(&text, idx);
                    idx = after;
                    let Some(specifier) = specifier else {
                        warnings.push((
                            "W_SVPP_MALFORMED_KEYWORDS_DIRECTIVE",
                            file,
                            line,
                            "`begin_keywords is not followed by a quoted version_specifier"
                                .to_string(),
                            "IEEE 1800-2017 §22.14 Syntax 22-10: keywords_directive ::= \
                             `begin_keywords \"version_specifier\"; no region is opened and the \
                             parser refuses the directive itself"
                                .to_string(),
                        ));
                        continue;
                    };
                    // ⛔ An UNRECOGNIZED specifier is REPORTED here and REFUSED by the PARSER, not
                    // refused twice. §22.14 says "It shall be an error if an implementation does not
                    // recognize the version_specifier", and the implementation that errors is the
                    // grammar: `keywords_version_specifier_<profile>` admits exactly the specifiers
                    // its own edition lists, per edition, so every other spelling is already a parse
                    // rejection. Refusing here as well would only PREEMPT that precise refusal and
                    // turn a rejection into a preprocessor refusal — measured on the corpus, eight
                    // occurrences (`"VAMS-2.3"` ×7, `"1800+VAMS"` ×1) whose rows are keyed on the
                    // parser's verdict. An operator who wants the clause fatal at this stage has the
                    // existing knob: `--sv-strict-warning-codes W_SVPP_UNKNOWN_KEYWORDS_VERSION`.
                    // No region is opened either way: an unknown version has no known keyword set,
                    // so there is nothing this stage could correctly render.
                    let Some(region) = KeywordVersion::from_specifier(specifier) else {
                        // `SVPP-EXPANSION.6f`: the region's PAIRING is a textual fact independent of the
                        // version being known — its `end_keywords closes it (counted here), so the refusal
                        // of an unmatched `end_keywords below does not preempt the parser's own refusal of
                        // the unknown specifier
                        unknown_open += 1;
                        warnings.push((
                            "W_SVPP_UNKNOWN_KEYWORDS_VERSION",
                            file,
                            line,
                            format!("unrecognized `begin_keywords version_specifier \"{specifier}\""),
                            format!(
                                "IEEE 1800-2017 §22.14: \"It shall be an error if an implementation \
                                 does not recognize the version_specifier used with the \
                                 `begin_keywords directive\"; no region is opened and the parser \
                                 refuses the directive. The specifiers this standard defines are {}",
                                KeywordVersion::ALL
                                    .iter()
                                    .map(|version| version.specifier())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                        ));
                        continue;
                    };
                    if !profile_admits_specifier(profile_name.as_deref(), specifier) {
                        warnings.push((
                            "W_SVPP_KEYWORDS_VERSION_NOT_IN_EDITION",
                            file.clone(),
                            line,
                            format!(
                                "version_specifier \"{specifier}\" is not one the parse profile's \
                                 own edition defines"
                            ),
                            format!(
                                "IEEE 1800-2017 §22.14 Syntax 22-10 enumerates the specifiers per \
                                 edition; the region is applied here and the parser refuses the \
                                 directive under profile {}",
                                profile_name.as_deref().unwrap_or("<none>")
                            ),
                        ));
                    }
                    if !region_within_profile(profile, region) {
                        warnings.push((
                            "W_SVPP_KEYWORDS_VERSION_ABOVE_PROFILE",
                            file.clone(),
                            line,
                            format!(
                                "version_specifier \"{specifier}\" reserves identifiers the parse \
                                 profile does not"
                            ),
                            "this stage can unreserve a keyword (§5.6.1: an escaped identifier is \
                             not interpreted as a keyword) but cannot reserve a non-keyword, so the \
                             region is applied only in the direction text can express"
                                .to_string(),
                        ));
                    }
                    open.push((region, 0));
                    let words = words_to_escape(profile, region);
                    events.push((
                        file,
                        line,
                        format!("begin_keywords \"{specifier}\" unreserves={}", words.len()),
                    ));
                    unreserved = Some(words);
                }
                digit if digit.is_ascii_digit() => {
                    idx += 1;
                    while idx < bytes.len()
                        && (bytes[idx].is_ascii_alphanumeric()
                            || bytes[idx] == b'_'
                            || bytes[idx] == b'$')
                    {
                        idx += 1;
                    }
                }
                start if is_ident_start(start) => {
                    let word_start = idx;
                    while idx < bytes.len() && is_ident_continue(bytes[idx]) {
                        idx += 1;
                    }
                    if let Some(words) = unreserved.as_ref() {
                        let word = &text[word_start..idx];
                        if words.contains(word) {
                            replacements.push((word_start, idx, format!("\\{word} ")));
                            if let Some(region) = open.last_mut() {
                                region.1 += 1;
                            }
                        }
                    }
                }
                _ => idx += 1,
            }
        }

        // §22.14: a region runs "until the matching `end_keywords directive or the end of the
        // compilation unit", so an unclosed one is legal — it carries into the unit's next file.
        for (region, escaped) in &open {
            events.push((
                PathBuf::from("<compilation unit>"),
                0,
                format!(
                    "still open at end of file: \"{}\" escaped={escaped}",
                    region.specifier()
                ),
            ));
        }
        self.keyword_regions = open.iter().map(|(version, _)| *version).collect();

        self.output = apply_renderings(&text, &replacements, &mut self.source_map);
        for (file, line, detail) in events {
            self.push_event(
                PreprocessorEventKind::KeywordsRegion,
                &file,
                line,
                true,
                detail,
            );
        }
        for (code, file, line, message, detail) in warnings {
            self.push_warning(code, &file, line, message, detail)?;
        }
        Ok(())
    }

    /// The `(file, line)` the byte at `offset` of the expanded text came from, or `None` when no
    /// chunk covers it. The map is appended in output order, so the covering chunk is found by
    /// bisection rather than by scanning (`SVPP-EXPANSION.8`).
    /// `SVPP-EXPANSION.6f`: a backtick that SURVIVES expansion outside a comment or a string literal is not
    /// text the parser may judge. IEEE 1800-2017 §22.5.1 gives a text-macro usage its meaning by substitution
    /// ("Once a text macro name has been defined, it can be used anywhere …", docs/systemverilog/2017/md/
    /// section-22-compiler-directives.md:272-273) — an undefined name has no derivation; "For a macro with
    /// arguments, the parentheses are always required in the macro call" (:224-225) — a function-like macro
    /// used bare, or with an argument list the file never closes (Syntax 22-3), is an incomplete usage kept as
    /// written by the expander; `` `" `` / `` `\`" `` / `` `` `` have meaning only inside macro text. Before
    /// this the leftover reached the parser, whose raw-lane residue (`compiler_directive` → `opaque`) ACCEPTS
    /// a top-level backtick line: 38 pipeline rows read `pass` over text the stage never resolved. The LRM
    /// directives the parser reads (`` `timescale ``, `` `default_nettype ``, …) are not unresolved usages.
    /// `SVPP-EXPANSION.6h`: an escaped identifier (§5.6.1, backslash to white space) is ONE token the scan steps
    /// over, a backtick inside it included; an escaped usage `` `\name `` is looked up by its name like a simple
    /// one; and a backtick before white space, a line end, the end of the text, a digit or punctuation begins
    /// neither a directive (§22.2) nor a usage (Syntax 22-3) and is refused — through 1.0.251 it reached the
    /// parser, which accepted it as the same opaque line.
    fn refuse_unresolved_backticks(&self) -> Result<()> {
        if self.config.undefined_macro_policy == UndefinedMacroPolicy::Keep {
            return Ok(());
        }
        let version = version_for_profile(self.config.parse_profile.as_deref());
        let text = &self.output;
        let bytes = text.as_bytes();
        let mut i = 0usize;
        let (mut in_string, mut escaped) = (false, false);
        while i < bytes.len() {
            let b = bytes[i];
            if in_string {
                if escaped {
                    escaped = false;
                } else if b == b'\\' {
                    escaped = true;
                } else if b == b'"' || b == b'\n' {
                    in_string = false;
                }
                i += 1;
                continue;
            }
            if b == b'"' {
                in_string = true;
                i += 1;
                continue;
            }
            if text[i..].starts_with("//") {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            if text[i..].starts_with("/*") {
                match text[i + 2..].find("*/") {
                    Some(close) => i += 2 + close + 2,
                    None => i = bytes.len(),
                }
                continue;
            }
            if b == b'\\' {
                // `SVPP-EXPANSION.6h`: an escaped identifier "shall start with the backslash character (\) and
                // end with white space" and may carry "any of the printable ASCII characters" (IEEE 1800-2017
                // §5.6.1; the grammar's `escaped_identifier := /\\[!-~]+/`) — a backtick inside it is part of ONE
                // token of the language, not a usage (iverilog mangle.v `\`~!-_=+\|[]{};:'",./<>?`). A keyword
                // the `` `begin_keywords `` renderer unreserved arrives here in the same spelling (`\logic `).
                let mut e = i + 1;
                while e < bytes.len() && bytes[e] > b' ' && bytes[e] < 0x7f {
                    e += 1;
                }
                i = if e > i + 1 { e } else { i + 1 };
                continue;
            }
            if b == b'`' {
                let (file, line) = self
                    .origin_of_output_offset(i)
                    .unwrap_or_else(|| (PathBuf::from("<expanded>"), 0));
                let next = bytes.get(i + 1).copied();
                let macro_string_token = matches!(next, Some(b'"') | Some(b'`'))
                    || (next == Some(b'\\') && text[i + 1..].starts_with("\\`\""));
                if macro_string_token {
                    bail!(
                        "a macro-string or paste token (`\", `\\`\", ``) outside macro text at {}:{} — IEEE \
                         1800-2017 §22.5.1 gives these tokens meaning only within the text of a `define; \
                         here they are no token of the language",
                        file.display(),
                        line
                    );
                }
                // A usage names its macro by a simple identifier or an escaped one (§22.5.1 Syntax 22-2
                // `text_macro_identifier ::= identifier`, A.9.3; `SVPP-EXPANSION.5`(q)(ii)) — the same reader the
                // expander uses, so `` `\esc `` and `` `esc `` are looked up under ONE key.
                if let Some((name, rest)) = parse_macro_name_token(&text[i + 1..]) {
                    let e = text.len() - rest.len();
                    if next != Some(b'\\') && is_compiler_directive_name(name, version) {
                        i = e;
                        continue;
                    }
                    match self.macros.get(name) {
                        Some(def) if def.params.is_some() => bail!(
                            "the function-like macro `{} is used without a complete argument list at {}:{} — IEEE \
                             1800-2017 §22.5.1: \"For a macro with arguments, the parentheses are always required in \
                             the macro call, even if all the arguments have defaults\" (Syntax 22-3: the closing \
                             parenthesis is part of the usage)",
                            name,
                            file.display(),
                            line
                        ),
                        Some(_) => bail!(
                            "the macro `{} survived expansion at {}:{} — an internal expansion gap, please report it",
                            name,
                            file.display(),
                            line
                        ),
                        None => bail!(
                            "`{} is neither a defined macro nor a compiler directive at {}:{} — IEEE 1800-2017 \
                             §22.5.1 gives a text-macro usage its meaning by substitution (\"Once a text macro name \
                             has been defined, it can be used anywhere in the compilation unit where it is defined\"); \
                             no definition in the compilation unit or its context, no substitution (a vendor's own \
                             directive is refused here too: clause 22 lists the standard's)",
                            name,
                            file.display(),
                            line
                        ),
                    }
                }
                // `SVPP-EXPANSION.6h`: a backtick before white space, a line end, the end of the text, a digit or
                // a punctuation character begins neither a compiler directive (§22.2: "All compiler directives are
                // preceded by the (`) character" — the character introduces a directive NAME) nor a text-macro
                // usage (§22.5.1 Syntax 22-3 `text_macro_usage ::= `text_macro_identifier [ ( … ) ]`). Through
                // 1.0.251 the scan tested only an identifier or a macro-string successor and let `` ` ``, `` `+ ``
                // and `` `9 `` through to the parser, whose raw-lane residue accepted them as an opaque line.
                let what = match next {
                    None => "the end of the text".to_string(),
                    Some(b'\n') | Some(b'\r') => "a line end".to_string(),
                    Some(b' ') | Some(b'\t') => "white space".to_string(),
                    Some(b'\\') => "a backslash that begins no escaped identifier".to_string(),
                    Some(c) => format!("`{}`", c as char),
                };
                bail!(
                    "a backtick that begins no compiler directive and no text-macro usage at {}:{} — IEEE 1800-2017 \
                     §22.2 (\"All compiler directives are preceded by the (`) character\") and §22.5.1 Syntax 22-3 \
                     (`text_macro_usage ::= `text_macro_identifier …`) give the character meaning only before a \
                     name; before {} it is no token of the language",
                    file.display(),
                    line,
                    what
                );
            }
            i += 1;
        }
        Ok(())
    }

    fn origin_of_output_offset(&self, offset: usize) -> Option<(PathBuf, usize)> {
        let at = self
            .source_map
            .partition_point(|entry| entry.output_start <= offset);
        let entry = self.source_map.get(at.checked_sub(1)?)?;
        if offset < entry.output_end {
            Some((PathBuf::from(&entry.source.file), entry.source.line))
        } else {
            None
        }
    }

    fn push_error(
        &mut self,
        code: &str,
        file: &Path,
        line: usize,
        message: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<()> {
        let diagnostic = PreprocessorDiagnostic {
            code: code.to_string(),
            severity: PreprocessorDiagnosticSeverity::Error,
            file: file.display().to_string(),
            line,
            message: message.into(),
            detail: detail.into(),
        };
        self.diagnostics.push(diagnostic.clone());
        bail!(
            "{} at {}:{}: {} ({})",
            diagnostic.code,
            diagnostic.file,
            diagnostic.line,
            diagnostic.message,
            diagnostic.detail
        );
    }
}

/// `SVPP-EXPANSION.8` — the quoted `version_specifier` that follows `` `begin_keywords ``, and the
/// byte index just past it. §22.14 Syntax 22-10 writes the specifier as a string literal; only white
/// space may separate it from the directive, so a directive with anything else after it reads as
/// malformed rather than silently opening a region.
fn read_version_specifier(text: &str, after_directive: usize) -> (Option<&str>, usize) {
    let bytes = text.as_bytes();
    let mut idx = after_directive;
    while matches!(bytes.get(idx), Some(b' ') | Some(b'\t')) {
        idx += 1;
    }
    if bytes.get(idx) != Some(&b'"') {
        return (None, idx);
    }
    let start = idx + 1;
    let mut end = start;
    while end < bytes.len() && bytes[end] != b'"' && bytes[end] != b'\n' {
        end += 1;
    }
    if bytes.get(end) != Some(&b'"') {
        return (None, end);
    }
    (Some(&text[start..end]), end + 1)
}

/// `SVPP-EXPANSION.8` — splice the renderings into the expanded text and RE-INDEX the source map by
/// the exact byte delta each one contributes.
///
/// The renderings arrive in ascending, non-overlapping output order (they are produced by one
/// left-to-right scan), so an output offset moves by the total delta of every rendering that ENDS at
/// or before it: a chunk's start is unaffected by a rendering that begins at that same offset, while
/// its end absorbs every rendering inside it. A chunk is one emitted line and an identifier never
/// spans two, so no rendering straddles a chunk boundary.
fn apply_renderings(
    text: &str,
    replacements: &[(usize, usize, String)],
    source_map: &mut [SourceMapEntry],
) -> String {
    if replacements.is_empty() {
        return text.to_string();
    }
    let mut rendered = String::with_capacity(text.len() + replacements.len() * 2);
    let mut cursor = 0usize;
    let mut ends: Vec<usize> = Vec::with_capacity(replacements.len());
    let mut deltas: Vec<isize> = Vec::with_capacity(replacements.len());
    let mut running = 0isize;
    for (start, end, replacement) in replacements {
        rendered.push_str(&text[cursor..*start]);
        rendered.push_str(replacement);
        cursor = *end;
        running += replacement.len() as isize - (*end - *start) as isize;
        ends.push(*end);
        deltas.push(running);
    }
    rendered.push_str(&text[cursor..]);

    let shift = |offset: usize| -> usize {
        let at = ends.partition_point(|end| *end <= offset);
        let delta = if at == 0 { 0 } else { deltas[at - 1] };
        (offset as isize + delta).max(0) as usize
    };
    for entry in source_map.iter_mut() {
        entry.output_start = shift(entry.output_start);
        entry.output_end = shift(entry.output_end);
    }
    rendered
}

pub fn preprocess_systemverilog_file(
    input_path: &Path,
    config: &SvPreprocessorConfig,
) -> Result<PreprocessedOutput> {
    let mut state = PreprocessorState::new(config.clone());
    preprocess_file_internal(input_path, &mut state, 0)?;
    state.render_keyword_regions()?;
    state.refuse_unresolved_backticks()?;
    Ok(state.take_output())
}

/// Preprocess an ordered COMPILATION UNIT (`SVPP-EXPANSION.2`(b)): the files are processed in
/// the order given under ONE macro table, so a macro defined (or undefined) in file *n* is in
/// force for file *n+1* and every file after it.
///
/// This is IEEE 1800-2017 §22.5.1 (`docs/systemverilog/2017/md/section-22-compiler-directives.md:272-273`):
/// *"Once a text macro name has been defined, it can be used anywhere in the compilation unit where
/// it is defined. There are no other scope restrictions once inside the compilation unit"* — and
/// §3.12.1 (`:406`) makes *"all files on a given compilation command line"* one compilation unit.
/// `preprocess_systemverilog_file` is the one-file unit; nothing else differs between the two.
///
/// What is per FILE, deliberately: the expanded text, its source map, the event log, the
/// diagnostics and the included-file list. The returned vector is index-aligned with `files`.
/// A failure in any file aborts the unit with that file named in the error, because a later file
/// may depend on definitions the failed one never reached.
///
/// What `` `resetall `` does here: nothing to the table. IEEE 1800-2017 §22.3 (`:2-3` of the
/// clause): *"Not all compiler directives have a default value (e.g., `` `define `` and
/// `` `include ``). Directives that do not have a default are not affected by `` `resetall ``."*
pub fn preprocess_systemverilog_unit(
    files: &[PathBuf],
    config: &SvPreprocessorConfig,
) -> Result<Vec<PreprocessedOutput>> {
    let mut outputs = Vec::with_capacity(files.len());
    let mut carried: Option<HashMap<String, MacroDefinition>> = None;
    // `SVPP-EXPANSION.8`: §22.14's region stack is UNIT state — "even across source code file
    // boundaries" — so it is carried between the unit's files exactly as the macro table is.
    let mut keyword_regions: Vec<KeywordVersion> = Vec::new();
    for (index, file) in files.iter().enumerate() {
        let mut state = PreprocessorState::new(config.clone());
        if let Some(macros) = carried.take() {
            state.macros = macros;
        }
        state.keyword_regions = std::mem::take(&mut keyword_regions);
        preprocess_file_internal(file, &mut state, 0).with_context(|| {
            format!(
                "compilation unit file {} of {} ('{}') failed to preprocess",
                index + 1,
                files.len(),
                file.display()
            )
        })?;
        state.render_keyword_regions().with_context(|| {
            format!(
                "compilation unit file {} of {} ('{}') failed to preprocess",
                index + 1,
                files.len(),
                file.display()
            )
        })?;
        state.refuse_unresolved_backticks().with_context(|| {
            format!(
                "compilation unit file {} of {} ('{}') failed to preprocess",
                index + 1,
                files.len(),
                file.display()
            )
        })?;
        keyword_regions = state.keyword_regions.clone();
        outputs.push(state.take_output());
        carried = Some(state.into_carried_macros());
    }
    Ok(outputs)
}

pub fn parse_strict_warning_codes(raw: &str) -> HashSet<String> {
    let mut codes = HashSet::new();
    for token in raw.split(',') {
        let normalized = token.trim().to_ascii_uppercase();
        if normalized.is_empty() || normalized == "NONE" {
            continue;
        }
        if normalized == "ALL" {
            codes.insert("*".to_string());
            continue;
        }
        codes.insert(normalized);
    }
    codes
}

fn preprocess_file_internal(
    path: &Path,
    state: &mut PreprocessorState,
    depth: usize,
) -> Result<()> {
    if depth > state.config.max_include_depth {
        // `SVPP-EXPANSION.5`(q)(i): IEEE 1800-2017 §22.4 — "The number of nesting levels for include files
        // shall be finite. Implementations may limit the maximum number of levels to which include files can
        // be nested, but the limit shall be at least 15." The bound is NESTING: a file that includes itself
        // under a guard nests once and is finite (sv2v `file_line.sv`, `include_self.sv`); one that does so
        // unguarded reaches this limit, and the refusal shows the chain's last steps.
        let chain = state
            .include_stack
            .iter()
            .rev()
            .take(5)
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>();
        let chain = chain.into_iter().rev().collect::<Vec<_>>().join(" -> ");
        bail!(
            "include nesting deeper than {} levels while entering '{}' (IEEE 1800-2017 §22.4: \"The number \
             of nesting levels for include files shall be finite. Implementations may limit the maximum \
             number of levels to which include files can be nested, but the limit shall be at least 15\"); \
             the chain's last steps: {} -> {}",
            state.config.max_include_depth,
            path.display(),
            chain,
            path.display()
        );
    }

    let canonical_path = path
        .canonicalize()
        .with_context(|| format!("failed to canonicalize '{}'", path.display()))?;

    let content_bytes = fs::read(&canonical_path)
        .with_context(|| format!("failed to read '{}'", canonical_path.display()))?;
    // ⭐ SV-CORPUS-GRAD.12c.1 — the shared source-text decoder, not a local lossy fallback.
    //
    // Until this leaf the preprocessor substituted U+FFFD for every non-UTF-8 byte while
    // `parseability_probe` refused the same file outright: two USER-SOURCE readers, two
    // contradictory answers, and nothing in the repo saying so. The decoder ends that — a
    // Latin-1 `©` in a header comment now survives as `©` on BOTH paths. The warning stays
    // (the encoding is still worth telling the operator about) but it now names the encoding
    // instead of announcing a loss that no longer happens.
    let decoded = crate::source_text::decode_source_bytes(content_bytes)
        .with_context(|| format!("failed to decode '{}'", canonical_path.display()))?;
    if decoded.encoding != crate::source_text::SourceEncoding::Utf8 {
        state.push_warning(
            "W_SVPP_NON_UTF8_SOURCE",
            &canonical_path,
            1,
            format!(
                "source file is not plain UTF-8; decoded as {}",
                decoded.encoding
            ),
            match decoded.first_non_utf8_byte_offset {
                Some(at) => format!(
                    "first non-UTF-8 byte at file offset {}; decoded losslessly, so byte offsets \
                     into the preprocessed text no longer match the file",
                    at
                ),
                None => "decoded losslessly; byte offsets into the preprocessed text no longer \
                         match the file"
                    .to_string(),
            },
        )?;
    }
    let content = decoded.text;
    state.include_stack.push(canonical_path.clone());
    state.record_include_file(&canonical_path);
    preprocess_text_internal(&content, &canonical_path, state, depth, None)?;
    state.include_stack.pop();
    Ok(())
}

/// `SVPP-EXPANSION.5`(b): `line_base` is `Some(use_line)` when `content` is the EXPANSION of a
/// macro used on that line — an expansion has no lines of its own, so every event, diagnostic and
/// source-map chunk it produces names the line of the use (the same rule `.3`'s origin mapping
/// documents). Before this, a re-entered expansion numbered its lines from 1, so a diagnostic
/// named `tlul_assert.sv:4` — a comment line — for a `` `else `` that sat inside a macro body.
fn preprocess_text_internal(
    content: &str,
    file_path: &Path,
    state: &mut PreprocessorState,
    depth: usize,
    line_base: Option<usize>,
) -> Result<()> {
    let mut conditionals: Vec<ConditionalFrame> = Vec::new();
    let lines = split_lines_preserve_terminator(content);
    let mut line_index = 0usize;
    // `SVPP-EXPANSION.5`(b): the text of a line that FOLLOWS a conditional directive on the same
    // line (`` `ifdef B b `else c `endif ; `` — IEEE 1800-2017 §22.6: "these directives may appear
    // anywhere in the source description"), or that PRECEDES one. It is processed as its own logical
    // line, at the same line number, before `line_index` advances.
    let mut pending_tail: Option<String> = None;
    // `SVPP-EXPANSION.6b.2`(ii): the file name and line number a `` `line `` directive assigned to
    // the FOLLOWING physical lines — (named file, number for the first of them, its physical index).
    let mut line_override: Option<(PathBuf, usize, usize)> = None;
    // `SVPP-EXPANSION.6b.5`(b): a block comment opened on an earlier line is still open — its lines
    // are comment text (IEEE 1800-2017 §5.4), emitted verbatim and never scanned for a directive.
    let mut in_block_comment = false;
    // `SVPP-EXPANSION.6b.5`(d): the nesting depth of §34 protected envelopes being consumed.
    let mut protected_depth = 0usize;
    while line_index < lines.len() || pending_tail.is_some() {
        let line_no = line_base.unwrap_or(line_index + 1);
        // where an expansion on this line says it is: `__FILE__` / `__LINE__` and the messages read
        // the `` `line ``-assigned location (IEEE 1800-2017 §22.12: "the compiler shall maintain the
        // current line number and file name … the `line directive shall set [them] for the following
        // line"); the events and the source map keep the physical one.
        let (exp_file, exp_line): (PathBuf, usize) = match (&line_override, line_base) {
            (Some((named_file, number, start)), None) if line_index >= *start => {
                (named_file.clone(), number + (line_index - start))
            }
            _ => (file_path.to_path_buf(), line_no),
        };
        let active = conditionals
            .last()
            .map(|c| c.current_active)
            .unwrap_or(true);
        let from_pending = pending_tail.is_some();
        let lrm_2023 = reads_1800_2023(&state.config);
        let mut raw_line = match pending_tail.take() {
            Some(tail) => tail,
            None => lines[line_index].clone(),
        };
        // `SVPP-EXPANSION.6b.5`(d): inside a §34 protected envelope every line is the envelope's,
        // consumed until the `` `pragma protect `` … `end_protected` that closes it (§34.3: "the end of
        // the decryption envelope occurs at the point where an end_protected pragma expression is
        // encountered"); the payload is not source text until decrypted, and this stage does not
        // decrypt. Before this the base64 payload reached the parser as source.
        if protected_depth > 0 {
            if let Some(("pragma", rest)) =
                parse_directive_line(raw_line.trim_start_matches([' ', '\t']))
            {
                if protect_pragma_has_keyword(rest, "end_protected") {
                    protected_depth -= 1;
                    if protected_depth == 0 {
                        state.push_event(
                            PreprocessorEventKind::ProtectedEnvelope,
                            file_path,
                            line_no,
                            active,
                            "end_protected".to_string(),
                        );
                    }
                } else if protect_pragma_has_keyword(rest, "begin_protected") {
                    protected_depth += 1;
                }
            }
            if pending_tail.is_none() {
                line_index += 1;
            }
            continue;
        }
        // `SVPP-EXPANSION.6b.5`(b): the continuation of a block comment opened on an earlier line —
        // Surelog PPComment opens `/*` after `assign c= d;` and the `` `else `` / `` `endif `` inside
        // the comment were honoured as directives.
        if in_block_comment {
            match raw_line.find("*/") {
                None => {
                    if active {
                        state.append_output_chunk(
                            &raw_line,
                            file_path,
                            line_no,
                            1,
                            raw_line.chars().count().max(1),
                        );
                    }
                    state.push_event(
                        PreprocessorEventKind::PassthroughLine,
                        file_path,
                        line_no,
                        active,
                        "block comment".to_string(),
                    );
                    if pending_tail.is_none() {
                        line_index += 1;
                    }
                    continue;
                }
                Some(close) => {
                    in_block_comment = false;
                    let tail = &raw_line[close + 2..];
                    if tail.trim().is_empty() {
                        // the close ends the line: emit it whole, terminator included (the first cut
                        // dropped the newline here and `*/` fused with the next line's text)
                        if active {
                            state.append_output_chunk(
                                &raw_line,
                                file_path,
                                line_no,
                                1,
                                raw_line.chars().count().max(1),
                            );
                        }
                        if pending_tail.is_none() {
                            line_index += 1;
                        }
                        continue;
                    }
                    let head = raw_line[..close + 2].to_string();
                    let tail = tail.to_string();
                    if active {
                        state.append_output_chunk(
                            &head,
                            file_path,
                            line_no,
                            1,
                            head.chars().count().max(1),
                        );
                    }
                    raw_line = tail; // the rest of the line is its own logical line
                }
            }
        }
        let initial_trimmed = raw_line.trim_start_matches([' ', '\t']).to_string();

        if let Some((directive, _)) = parse_directive_line(&initial_trimmed) {
            if directive == "define" && !from_pending {
                let mut continuation_index = line_index;
                loop {
                    // `SVPP-EXPANSION.5`(a): a formal-argument list may span lines with no backslash —
                    // IEEE 1800-2017 §22.5.1: "The formal argument names shall be simple_identifiers,
                    // separated by commas and optionally white space", and §5.3 white space includes
                    // newlines. The list closes at its `)`, wherever that is; only then does the
                    // macro TEXT's own backslash-newline rule apply.
                    if define_formal_list_unclosed(&raw_line)
                        && continuation_index + 1 < lines.len()
                    {
                        raw_line = format!(
                            "{}{}",
                            raw_line.trim_end_matches(['\r', '\n']).to_string() + "\n",
                            lines[continuation_index + 1]
                        );
                        continuation_index += 1;
                        continue;
                    }
                    if line_has_continuation(&raw_line) && continuation_index + 1 < lines.len() {
                        // `SVPP-EXPANSION.5`(u): IEEE 1800-2023 §22.5.1 — "if the backslash-newline character
                        // sequence occurs in the middle of a string literal, both the backslash and the newline
                        // characters are omitted in the expanded macro" (docs/systemverilog/2023/md/
                        // section-22-compiler-directives.md:182-184). 1800-2017 has no such sentence: the
                        // continuation is replaced by a newline there, inside a string as anywhere else.
                        let stripped = strip_line_continuation(&raw_line);
                        let joiner = if lrm_2023 && macro_text_leaves_string_open(&stripped) { "" } else { "\n" };
                        raw_line = format!("{}{}{}", stripped, joiner, lines[continuation_index + 1]);
                        continuation_index += 1;
                        continue;
                    }
                    // `SVPP-EXPANSION.5`(p)(i): IEEE 1800-2023 §22.5.1 — "A newline character that is not
                    // contained in a triple-quoted string literal or in a block comment and is not
                    // preceded by a backslash shall end the macro text": under the 2023 reading a block
                    // comment the text opened carries the `` `define `` across its newlines.
                    if lrm_2023
                        && define_leaves_block_comment_open(&raw_line)
                        && continuation_index + 1 < lines.len()
                    {
                        raw_line = format!(
                            "{}{}",
                            raw_line.trim_end_matches(['\r', '\n']).to_string() + "\n",
                            lines[continuation_index + 1]
                        );
                        continuation_index += 1;
                        continue;
                    }
                    break;
                }
                line_index = continuation_index;
                // …and under the 2017 reading the same text is refused: the newline ended the macro
                // text INSIDE a comment, which §22.5.1 forbids ("shall not be split across … Comments").
                // Before this the comment's tail was emitted as source text with its opener consumed.
                if active && !lrm_2023 && define_leaves_block_comment_open(&raw_line) {
                    bail!(
                        "invalid `define directive at {}:{}: a block comment opened in the macro text is \
                         not closed before the text ends (IEEE 1800-2017 §22.5.1: the first newline not \
                         preceded by a backslash ends the macro text, and the text \"shall not be split \
                         across\" a comment; IEEE 1800-2023 §22.5.1 lets a block comment carry the text \
                         across its newlines — that reading is selected by --profile sv_2023)",
                        file_path.display(),
                        line_no
                    );
                }
            }
        }

        // `SVPP-EXPANSION.5`(b): a conditional directive AFTER text on the same line splits the line:
        // the text before it is this logical line, the directive and what follows it is the next
        // one (same line number). A `` `define `` line is exempt — its conditionals belong to the
        // macro text and are honoured when the macro is expanded.
        let line_is_define = matches!(parse_directive_line(&initial_trimmed), Some(("define", _)));
        if !line_is_define && let Some(split_at) = first_midline_conditional(&raw_line) {
            let head = raw_line[..split_at].to_string();
            let tail = raw_line[split_at..].to_string();
            if active {
                let expanded = expand_macros_in_text(
                    &head,
                    &state.macros,
                    0,
                    &mut state.events,
                    exp_file.as_path(),
                    exp_line,
                    &mut Vec::new(),
                    None,
                    state.config.plain_string_substitution,
                )?;
                state.push_event(
                    PreprocessorEventKind::PassthroughLine,
                    file_path,
                    line_no,
                    true,
                    String::new(),
                );
                if expanded != head && contains_line_start_backtick(&expanded) {
                    preprocess_text_internal(&expanded, file_path, state, depth, Some(line_no))?;
                } else {
                    state.append_output_chunk(
                        &expanded,
                        file_path,
                        line_no,
                        1,
                        head.chars().count().max(1),
                    );
                }
            } else {
                state.push_event(
                    PreprocessorEventKind::SkippedLine,
                    file_path,
                    line_no,
                    false,
                    "inactive branch".to_string(),
                );
            }
            pending_tail = Some(tail);
            continue;
        }

        let trimmed = raw_line.trim_start_matches([' ', '\t']).to_string();
        let directive_parse = parse_directive_line(&trimmed).filter(|(directive, _)| {
            !state.macros.contains_key(*directive)
                && expand_builtin_macro(directive, file_path, line_no).is_none()
        });

        if let Some((directive, rest)) = directive_parse {
            // `SVPP-EXPANSION.5`(n): a `` `line `` directive whose PARAMETERS are macro usages (sv2v
            // `test/lex/line.sv:9` `` `line `new_line_num `foo(line) 1 ``). IEEE 1800-2023 §22.2: "Macro
            // expansion shall occur within a compiler directive"; 1800-2017 §22.5.1: a defined text macro
            // "can be used anywhere in the compilation unit where it is defined". The parameters are
            // expanded FIRST and what they yield is read as §22.12's `number "filename" level`. The
            // expansion's events are kept only when the directive is then consumed: a result that is not a
            // well-formed directive passes through below, where the line is expanded (and recorded) once.
            let mut line_probe_events = Vec::new();
            let line_rest_expanded: Option<String> =
                if directive == "line" && active && rest.contains('`') {
                    Some(expand_macros_in_text(
                        rest,
                        &state.macros,
                        0,
                        &mut line_probe_events,
                        exp_file.as_path(),
                        exp_line,
                        &mut Vec::new(),
                        None,
                        state.config.plain_string_substitution,
                    )?)
                } else {
                    None
                };
            let line_rest: &str = line_rest_expanded.as_deref().unwrap_or(rest);
            match directive {
                "define" => {
                    state.push_event(
                        PreprocessorEventKind::Define,
                        file_path,
                        line_no,
                        active,
                        rest.to_string(),
                    );
                    if active {
                        let (name, definition) =
                            parse_define_directive(
                                rest,
                                lrm_2023,
                                version_for_profile(state.config.parse_profile.as_deref()),
                            )
                            .with_context(|| {
                                format!(
                                    "invalid `define directive at {}:{}",
                                    file_path.display(),
                                    line_no
                                )
                            })?;
                        if state.macros.contains_key(&name) {
                            match state.config.macro_redefinition_policy {
                                MacroRedefinitionPolicy::Allow => {}
                                MacroRedefinitionPolicy::Warn => state.push_warning(
                                    "W_SVPP_MACRO_REDEFINED",
                                    file_path,
                                    line_no,
                                    "macro redefinition detected",
                                    format!("macro '{}'", name),
                                )?,
                                MacroRedefinitionPolicy::Error => state.push_error(
                                    "E_SVPP_MACRO_REDEFINED",
                                    file_path,
                                    line_no,
                                    "macro redefinition is disallowed by policy",
                                    format!("macro '{}'", name),
                                )?,
                            }
                        }
                        state.macros.insert(name, definition);
                    }
                }
                "undef" => {
                    state.push_event(
                        PreprocessorEventKind::Undef,
                        file_path,
                        line_no,
                        active,
                        rest.to_string(),
                    );
                    if active {
                        // `SVPP-EXPANSION.5`(p)(iii): 1800-2023 §22.2 — a macro usage as the argument
                        let argument = if lrm_2023 && rest.trim_start().starts_with('`') {
                            Some(expand_directive_argument(
                                rest,
                                state,
                                exp_file.as_path(),
                                exp_line,
                            )?)
                        } else {
                            None
                        };
                        let rest: &str = argument.as_deref().unwrap_or(rest);
                        if let Some((name, _)) = parse_macro_name_token(rest.trim_start()) {
                            state.macros.remove(name);
                        }
                    }
                }
                "include" => {
                    state.push_event(
                        PreprocessorEventKind::Include,
                        file_path,
                        line_no,
                        active,
                        rest.to_string(),
                    );
                    if active {
                        // `SVPP-EXPANSION.5`(m): IEEE 1800-2017 §22.5.1 — "The `include directive can be
                        // followed by a macro, instead of a string literal" (its example: `` `define
                        // home(filename) `"/home/mydir/filename`" `` then `` `include `home(myfile) ``). The
                        // macro is expanded first; what it yields is then read as Syntax 22-1's filename.
                        let expanded_rest: Option<String> = if rest.trim_start().starts_with('`') {
                            Some(expand_macros_in_text(
                                rest,
                                &state.macros,
                                0,
                                &mut state.events,
                                exp_file.as_path(),
                                exp_line,
                                &mut Vec::new(),
                                None,
                                state.config.plain_string_substitution,
                            )?)
                        } else {
                            None
                        };
                        let rest: &str = expanded_rest.as_deref().unwrap_or(rest);
                        let (include_token, is_angle) = parse_include_directive(rest)
                            .with_context(|| {
                                format!(
                                    "invalid `include directive at {}:{}",
                                    file_path.display(),
                                    line_no
                                )
                            })?;
                        if Path::new(include_token).is_absolute()
                            && state.config.include_path_policy == IncludePathPolicy::RelativeOnly
                        {
                            state.push_warning(
                                "W_SVPP_ABSOLUTE_INCLUDE_PATH",
                                file_path,
                                line_no,
                                "absolute include path violates include_path_policy=relative_only",
                                include_token.to_string(),
                            )?;
                        }
                        let include_path =
                            resolve_include_path(include_token, is_angle, file_path, state)
                                .with_context(|| {
                                    format!(
                                        "failed to resolve include '{}' at {}:{}",
                                        include_token,
                                        file_path.display(),
                                        line_no
                                    )
                                })?;
                        preprocess_file_internal(&include_path, state, depth + 1)?;
                        // `SVPP-EXPANSION.6b.5`(b): text after the filename is the next logical
                        // line's (iverilog br_gh782a: `` `include "x.vi"  /* multi-line `` opened a
                        // block comment the directive line then swallowed, orphaning its close) —
                        // the same rule as `.5`(b)'s remainder for a conditional.
                        let remainder = include_directive_remainder(rest);
                        if !remainder.trim().is_empty() {
                            pending_tail = Some(remainder.to_string());
                        }
                    }
                }
                "ifdef" => {
                    // `SVPP-EXPANSION.5`(p)(iii): IEEE 1800-2023 §22.2 — "Macro expansion shall occur
                    // within a compiler directive": an argument written as a macro USAGE is expanded
                    // first (verilator t_preproc.v:423, :426). 1800-2017 has no such sentence and its
                    // Syntax 22-6 takes a text_macro_identifier: there the argument is read as written.
                    let argument = if lrm_2023 && active && rest.trim_start().starts_with('`') {
                        Some(expand_directive_argument(
                            rest,
                            state,
                            exp_file.as_path(),
                            exp_line,
                        )?)
                    } else {
                        None
                    };
                    let rest: &str = argument.as_deref().unwrap_or(rest);
                    let (name, remainder) = split_conditional_argument(rest);
                    pending_tail = remainder_as_pending(remainder);
                    if !is_valid_macro_symbol(name) {
                        state.push_warning(
                            "W_SVPP_INVALID_CONDITIONAL_SYMBOL",
                            file_path,
                            line_no,
                            "`ifdef expects an identifier-like macro symbol",
                            name.to_string(),
                        )?;
                    }
                    let cond = eval_symbol_condition(name, state, file_path, line_no, "ifdef")?;
                    let parent_active = active;
                    let current_active = parent_active && cond;
                    conditionals.push(ConditionalFrame {
                        parent_active,
                        current_active,
                        branch_taken: cond,
                    });
                    state.push_event(
                        PreprocessorEventKind::Ifdef,
                        file_path,
                        line_no,
                        current_active,
                        name.to_string(),
                    );
                }
                "ifndef" => {
                    // `SVPP-EXPANSION.5`(p)(iii): IEEE 1800-2023 §22.2 — "Macro expansion shall occur
                    // within a compiler directive": an argument written as a macro USAGE is expanded
                    // first (verilator t_preproc.v:423, :426). 1800-2017 has no such sentence and its
                    // Syntax 22-6 takes a text_macro_identifier: there the argument is read as written.
                    let argument = if lrm_2023 && active && rest.trim_start().starts_with('`') {
                        Some(expand_directive_argument(
                            rest,
                            state,
                            exp_file.as_path(),
                            exp_line,
                        )?)
                    } else {
                        None
                    };
                    let rest: &str = argument.as_deref().unwrap_or(rest);
                    let (name, remainder) = split_conditional_argument(rest);
                    pending_tail = remainder_as_pending(remainder);
                    if !is_valid_macro_symbol(name) {
                        state.push_warning(
                            "W_SVPP_INVALID_CONDITIONAL_SYMBOL",
                            file_path,
                            line_no,
                            "`ifndef expects an identifier-like macro symbol",
                            name.to_string(),
                        )?;
                    }
                    // `ifndef is defined in terms of symbol presence, and should not
                    // trigger undefined-symbol diagnostics for the "missing" branch. `SVPP-EXPANSION.5`(t):
                    // a compiler directive's name is a PREDEFINED macro name (IEEE 1800-2017 §22.5.1), so
                    // `` `ifndef __FILE__ `` is false (ispras 22.13_01.sv guards its redefinition with it).
                    let cond = !(state.macros.contains_key(name)
                        || is_compiler_directive_name(
                            name,
                            version_for_profile(state.config.parse_profile.as_deref()),
                        ));
                    let parent_active = active;
                    let current_active = parent_active && cond;
                    conditionals.push(ConditionalFrame {
                        parent_active,
                        current_active,
                        branch_taken: cond,
                    });
                    state.push_event(
                        PreprocessorEventKind::Ifndef,
                        file_path,
                        line_no,
                        current_active,
                        name.to_string(),
                    );
                }
                "elsif" => {
                    // `SVPP-EXPANSION.5`(p)(iii), the `elsif arm: expanded only where the directive is
                    // actually evaluated (the enclosing region live and no earlier branch taken).
                    let evaluated = conditionals
                        .last()
                        .map(|frame| frame.parent_active && !frame.branch_taken)
                        .unwrap_or(false);
                    let argument = if lrm_2023 && evaluated && rest.trim_start().starts_with('`') {
                        Some(expand_directive_argument(
                            rest,
                            state,
                            exp_file.as_path(),
                            exp_line,
                        )?)
                    } else {
                        None
                    };
                    let rest: &str = argument.as_deref().unwrap_or(rest);
                    let (expr, remainder) = split_conditional_expression(rest);
                    pending_tail = remainder_as_pending(remainder);
                    let frame = conditionals.last_mut().with_context(|| {
                        format!(
                            "`elsif without matching `ifdef/`ifndef at {}:{}",
                            file_path.display(),
                            line_no
                        )
                    })?;
                    let cond = evaluate_elsif_expression(expr, state, file_path, line_no)?;
                    if frame.branch_taken {
                        frame.current_active = false;
                    } else {
                        frame.current_active = frame.parent_active && cond;
                        if cond {
                            frame.branch_taken = true;
                        }
                    }
                    state.push_event(
                        PreprocessorEventKind::Elsif,
                        file_path,
                        line_no,
                        frame.current_active,
                        expr.to_string(),
                    );
                }
                "else" => {
                    pending_tail = remainder_as_pending(rest);
                    let frame = conditionals.last_mut().with_context(|| {
                        format!(
                            "`else without matching `ifdef/`ifndef at {}:{}",
                            file_path.display(),
                            line_no
                        )
                    })?;
                    if frame.branch_taken {
                        frame.current_active = false;
                    } else {
                        frame.current_active = frame.parent_active;
                        frame.branch_taken = true;
                    }
                    state.push_event(
                        PreprocessorEventKind::Else,
                        file_path,
                        line_no,
                        frame.current_active,
                        String::new(),
                    );
                }
                "endif" => {
                    pending_tail = remainder_as_pending(rest);
                    conditionals.pop().with_context(|| {
                        format!(
                            "`endif without matching `ifdef/`ifndef at {}:{}",
                            file_path.display(),
                            line_no
                        )
                    })?;
                    state.push_event(
                        PreprocessorEventKind::Endif,
                        file_path,
                        line_no,
                        active,
                        String::new(),
                    );
                }
                // Keep common non-control directives in output if active.
                "timescale" | "default_nettype" | "celldefine" | "endcelldefine" => {
                    if active {
                        let expanded = expand_macros_in_text(
                            &raw_line,
                            &state.macros,
                            0,
                            &mut state.events,
                            exp_file.as_path(),
                            exp_line,
                            &mut Vec::new(),
                            None,
                            state.config.plain_string_substitution,
                        )?;
                        state.push_event(
                            PreprocessorEventKind::PassthroughLine,
                            file_path,
                            line_no,
                            true,
                            directive.to_string(),
                        );
                        state.append_output_chunk(
                            &expanded,
                            file_path,
                            line_no,
                            1,
                            raw_line.chars().count().max(1),
                        );
                    } else {
                        state.push_event(
                            PreprocessorEventKind::SkippedLine,
                            file_path,
                            line_no,
                            false,
                            format!("inactive directive `{}`", directive),
                        );
                    }
                }
                // `SVPP-EXPANSION.6b.2`(ii): IEEE 1800-2017 §22.12 — "The `line directive shall set the
                // line number and file name of the following line to those specified in the directive
                // … only white space may appear on the same line as the `line directive … All
                // parameters in the `line directive are required." A well-formed directive is CONSUMED
                // here — the preprocessor is the tool that "shall maintain the current line number and
                // file name", and the source map already carries the physical location — while a
                // malformed one falls through to the pass-through below, where the parser's own
                // §22.12 production refuses it (SV-CORPUS-GRAD.3.39's must_reject rows keep their
                // verdict under the pipeline). Before this the directive reached the parser inside a
                // statement block and iverilog `line_directive.v` rejected at the line after it.
                // `SVPP-EXPANSION.6b.5`(d): a §34 protected envelope opens on a `` `pragma protect ``
                // line carrying `begin_protected`; the lines up to the matching `end_protected` are
                // consumed above. Any other `` `pragma `` line keeps its pass-through.
                "pragma" if protect_pragma_has_keyword(rest, "begin_protected") => {
                    if active {
                        protected_depth = 1;
                    }
                    state.push_event(
                        PreprocessorEventKind::ProtectedEnvelope,
                        file_path,
                        line_no,
                        active,
                        "begin_protected".to_string(),
                    );
                }
                "line" if parse_line_directive(line_rest).is_some() => {
                    let (number, named_file) =
                        parse_line_directive(line_rest).expect("guarded above");
                    state.events.append(&mut line_probe_events);
                    if active {
                        line_override = Some((PathBuf::from(&named_file), number, line_index + 1));
                    }
                    state.push_event(
                        PreprocessorEventKind::LineDirective,
                        file_path,
                        line_no,
                        active,
                        format!("{} \"{}\"", number, named_file),
                    );
                }
                _ => {
                    // Unknown directive: pass through only when active to preserve tool behavior.
                    if active {
                        let expanded = expand_macros_in_text(
                            &raw_line,
                            &state.macros,
                            0,
                            &mut state.events,
                            exp_file.as_path(),
                            exp_line,
                            &mut Vec::new(),
                            None,
                            state.config.plain_string_substitution,
                        )?;
                        state.push_event(
                            PreprocessorEventKind::PassthroughLine,
                            file_path,
                            line_no,
                            true,
                            format!("unknown directive `{}`", directive),
                        );
                        state.append_output_chunk(
                            &expanded,
                            file_path,
                            line_no,
                            1,
                            raw_line.chars().count().max(1),
                        );
                    } else {
                        state.push_event(
                            PreprocessorEventKind::SkippedLine,
                            file_path,
                            line_no,
                            false,
                            format!("inactive unknown directive `{}`", directive),
                        );
                    }
                }
            }
            if pending_tail.is_none() {
                line_index += 1;
            }
            continue;
        }

        let opens_block_comment = leaves_block_comment_open(&raw_line);
        if active {
            let (logical_line, last_line_index) = collect_multiline_macro_invocation(
                lines.as_slice(),
                line_index,
                raw_line,
                &state.macros,
            );
            let expanded = expand_macros_in_text(
                &logical_line,
                &state.macros,
                0,
                &mut state.events,
                exp_file.as_path(),
                exp_line,
                &mut Vec::new(),
                None,
                state.config.plain_string_substitution,
            )?;
            state.push_event(
                PreprocessorEventKind::PassthroughLine,
                file_path,
                line_no,
                true,
                String::new(),
            );
            if expanded != logical_line && contains_line_start_backtick(&expanded) {
                preprocess_text_internal(&expanded, file_path, state, depth, Some(line_no))?;
            } else {
                state.append_output_chunk(
                    &expanded,
                    file_path,
                    line_no,
                    1,
                    logical_line.chars().count().max(1),
                );
            }
            line_index = last_line_index;
        } else {
            state.push_event(
                PreprocessorEventKind::SkippedLine,
                file_path,
                line_no,
                false,
                "inactive branch".to_string(),
            );
        }
        in_block_comment = opens_block_comment;
        if pending_tail.is_none() {
            line_index += 1;
        }
    }
    if protected_depth > 0 {
        bail!(
            "E_SVPP_UNTERMINATED_PROTECTED_ENVELOPE in {}: a `pragma protect begin_protected has no \
             matching end_protected — IEEE 1800-2017 §34.3: \"The end_protected pragma expression … \
             shall be associated with the most recent begin_protected that has not already been closed\"",
            file_path.display()
        );
    }

    if !conditionals.is_empty() {
        bail!(
            "unterminated conditional block in '{}'",
            file_path.display()
        );
    }
    Ok(())
}

const CONDITIONAL_DIRECTIVES: [&str; 5] = ["ifdef", "ifndef", "elsif", "else", "endif"];

/// `SVPP-EXPANSION.5`(b): the byte index of the first conditional directive that sits AFTER
/// non-blank text on this line (outside strings and comments), or `None`. A directive at the
/// line's start is not "mid-line" — the line-oriented path handles it.
fn first_midline_conditional(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0usize;
    let mut seen_text = false;
    let mut in_string = false;
    let mut escaped = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            return None; // the rest of the line is a comment
        }
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            // skip a block comment on this line; an unterminated one ends the scan
            match line[i + 2..].find("*/") {
                Some(end) => {
                    i += 2 + end + 2;
                    continue;
                }
                None => return None,
            }
        }
        if b == b'"' {
            in_string = true;
            seen_text = true;
            i += 1;
            continue;
        }
        if b == b'`' {
            let start = i + 1;
            let mut end = start;
            while end < bytes.len() && is_ident_continue(bytes[end]) {
                end += 1;
            }
            let name = &line[start..end];
            if CONDITIONAL_DIRECTIVES.contains(&name) {
                if seen_text {
                    return Some(i);
                }
                // a line-start directive: the rest of the line belongs to it
                return None;
            }
            i = end.max(i + 1);
            seen_text = true;
            continue;
        }
        if !b.is_ascii_whitespace() {
            seen_text = true;
        }
        i += 1;
    }
    None
}

/// `SVPP-EXPANSION.5`(b): a conditional directive's SYMBOL is the identifier that follows it — not
/// the rest of the line. Measured before this: `` `ifdef X // a trailing comment `` read the symbol
/// as `X // a trailing comment`, found it undefined, and took the `` `else `` branch with `X`
/// defined — silently, under the default policy. Returns `(symbol, remainder)`.
fn split_conditional_argument(rest: &str) -> (&str, &str) {
    let trimmed = rest.trim_start();
    match parse_macro_name_token(trimmed) {
        Some((name, remainder)) => (name, remainder),
        None => (trimmed.trim_end_matches(['\r', '\n']), ""),
    }
}

/// `SVPP-EXPANSION.5`(b): an `` `elsif `` expression is a symbol, `defined(SYM)` or `!defined(SYM)`;
/// what follows it on the line is text (or a comment). Returns `(expression, remainder)`.
fn split_conditional_expression(rest: &str) -> (&str, &str) {
    let trimmed = rest.trim_start();
    for prefix in ["!defined(", "defined("] {
        if let Some(after) = trimmed.strip_prefix(prefix)
            && let Some(close) = after.find(')')
        {
            let end = prefix.len() + close + 1;
            return (&trimmed[..end], &trimmed[end..]);
        }
    }
    split_conditional_argument(rest)
}

/// What follows a conditional directive on its line: nothing (consumed), a comment (consumed — the
/// `` `endif // NAME `` idiom), or TEXT that becomes the next logical line.
fn remainder_as_pending(remainder: &str) -> Option<String> {
    let body = remainder.trim();
    if body.is_empty() || body.starts_with("//") {
        return None;
    }
    if let Some(after) = body.strip_prefix("/*")
        && let Some(end) = after.find("*/")
    {
        let rest = after[end + 2..].trim();
        if rest.is_empty() || rest.starts_with("//") {
            return None;
        }
    }
    Some(remainder.to_string())
}

/// `SVPP-EXPANSION.5`(a): does this `` `define `` line open a formal-argument list it has not closed?
fn define_formal_list_unclosed(line: &str) -> bool {
    let trimmed = line.trim_start_matches([' ', '\t']);
    let Some(rest) = trimmed.strip_prefix("`define") else {
        return false;
    };
    let Some((_, after_name)) = parse_macro_name_token(rest.trim_start()) else {
        return false;
    };
    if !after_name.starts_with('(') {
        return false;
    }
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for ch in after_name.chars() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth > 0
}

fn collect_multiline_macro_invocation(
    lines: &[String],
    start_index: usize,
    initial_line: String,
    macros: &HashMap<String, MacroDefinition>,
) -> (String, usize) {
    let mut combined = initial_line;
    let mut current_index = start_index;
    while has_unclosed_function_macro_invocation(&combined, macros)
        && current_index + 1 < lines.len()
    {
        combined.push_str(&lines[current_index + 1]);
        current_index += 1;
    }
    (combined, current_index)
}

/// `SVPP-EXPANSION.6b.5`(b): does this source line leave a block comment OPEN at its end (a `/*`
/// outside a string literal and outside a `//` comment, with no `*/` after it)?
fn leaves_block_comment_open(line: &str) -> bool {
    let bytes = line.as_bytes();
    let mut i = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut open = false;
    while i < bytes.len() {
        let b = bytes[i];
        if open {
            if b == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                open = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if b == b'"' {
            in_string = true;
            i += 1;
            continue;
        }
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            return false;
        }
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            open = true;
            i += 2;
            continue;
        }
        i += 1;
    }
    open
}

/// `SVPP-EXPANSION.6b.5`(b): the text of an `` `include `` line after its `"filename"` / `<filename>`.
fn include_directive_remainder(rest: &str) -> &str {
    let trimmed = rest.trim_start();
    let close = if trimmed.starts_with('"') {
        '"'
    } else if trimmed.starts_with('<') {
        '>'
    } else {
        return "";
    };
    let body = &trimmed[1..];
    match body.find(close) {
        Some(end) => &body[end + 1..],
        None => "",
    }
}

/// `SVPP-EXPANSION.6b.5`(d): does a `` `pragma protect `` line carry the envelope keyword (§34.3.1: a
/// pragma expression's items are comma-separated; `begin_protected` / `end_protected` stand alone)?
fn protect_pragma_has_keyword(rest: &str, keyword: &str) -> bool {
    let text = rest.trim();
    let Some(after) = text.strip_prefix("protect") else {
        return false;
    };
    if !after.is_empty() && !after.starts_with(char::is_whitespace) {
        return false;
    }
    after.split(',').any(|item| item.trim() == keyword)
}

/// `SVPP-EXPANSION.6b.2`(ii): the three required parameters of a `` `line `` directive — IEEE
/// 1800-2017 §22.12 Syntax 22-9 `` `line number "filename" level ``, the level one of 0 | 1 | 2 and
/// only white space after — or None when the line is malformed (then it is not this stage's to
/// consume: it passes through and the parser refuses it).
fn parse_line_directive(rest: &str) -> Option<(usize, String)> {
    let text = rest.trim();
    let digits_end = text
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(text.len());
    if digits_end == 0 {
        return None;
    }
    let number: usize = text[..digits_end].parse().ok()?;
    let quoted = text[digits_end..].trim_start().strip_prefix('"')?;
    let close = quoted.find('"')?;
    let filename = &quoted[..close];
    if !matches!(quoted[close + 1..].trim(), "0" | "1" | "2") {
        return None;
    }
    Some((number, filename.to_string()))
}

fn parse_directive_line(input: &str) -> Option<(&str, &str)> {
    if !input.starts_with('`') {
        return None;
    }
    let mut end = 1;
    for ch in input[1..].chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            end += ch.len_utf8();
        } else {
            break;
        }
    }
    if end <= 1 {
        return None;
    }
    let name = &input[1..end];
    let rest = input[end..].trim_start();
    Some((name, rest))
}

#[derive(Debug, Clone)]
enum ParsedConditionalExpr {
    Symbol(String),
    Defined(String),
    NotDefined(String),
}

fn is_valid_macro_symbol(symbol: &str) -> bool {
    let bytes = symbol.as_bytes();
    if bytes.is_empty() || !is_ident_start(bytes[0]) {
        return false;
    }
    bytes[1..].iter().all(|b| is_ident_continue(*b))
}

fn eval_symbol_condition(
    symbol: &str,
    state: &mut PreprocessorState,
    file: &Path,
    line: usize,
    directive: &str,
) -> Result<bool> {
    // `SVPP-EXPANSION.5`(t): "All compiler directives shall be considered predefined macro names" (IEEE
    // 1800-2017 §22.5.1) — a directive name is DEFINED to `ifdef / `ifndef / `elsif, so a guarded
    // `` `ifndef __FILE__ `` / `` `define __FILE__ … `` (ispras 22.13_01.sv) skips the illegal redefinition.
    let defined = state.macros.contains_key(symbol)
        || is_compiler_directive_name(symbol, version_for_profile(state.config.parse_profile.as_deref()));
    if !defined {
        match state.config.conditional_symbol_policy {
            ConditionalSymbolPolicy::AssumeFalseSilent => {}
            ConditionalSymbolPolicy::AssumeFalseWarn => state.push_warning(
                "W_SVPP_UNDEFINED_CONDITIONAL_SYMBOL",
                file,
                line,
                "conditional symbol is undefined; policy assumes false",
                format!("`{} {}", directive, symbol),
            )?,
            ConditionalSymbolPolicy::Error => state.push_error(
                "E_SVPP_UNDEFINED_CONDITIONAL_SYMBOL",
                file,
                line,
                "conditional symbol is undefined; policy requires defined symbol",
                format!("`{} {}", directive, symbol),
            )?,
        }
    }
    Ok(defined)
}

fn parse_elsif_expression(
    expr: &str,
    policy: ConditionalExprPolicy,
) -> Result<Option<ParsedConditionalExpr>> {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    if is_valid_macro_symbol(trimmed) {
        return Ok(Some(ParsedConditionalExpr::Symbol(trimmed.to_string())));
    }

    match policy {
        ConditionalExprPolicy::IdentifierOnly => Ok(None),
        ConditionalExprPolicy::IdentifierOrDefined => {
            if let Some(inner) = parse_defined_call(trimmed, false) {
                return Ok(Some(ParsedConditionalExpr::Defined(inner.to_string())));
            }
            if let Some(inner) = parse_defined_call(trimmed, true) {
                return Ok(Some(ParsedConditionalExpr::NotDefined(inner.to_string())));
            }
            Ok(None)
        }
    }
}

fn parse_defined_call(expr: &str, negated: bool) -> Option<&str> {
    let trimmed = expr.trim();
    let prefix = if negated { "!defined(" } else { "defined(" };
    if !trimmed.starts_with(prefix) || !trimmed.ends_with(')') {
        return None;
    }
    let inner = &trimmed[prefix.len()..trimmed.len() - 1];
    let symbol = inner.trim();
    if is_valid_macro_symbol(symbol) {
        Some(symbol)
    } else {
        None
    }
}

fn evaluate_elsif_expression(
    expr: &str,
    state: &mut PreprocessorState,
    file: &Path,
    line: usize,
) -> Result<bool> {
    let parsed = parse_elsif_expression(expr, state.config.conditional_expr_policy)?;
    let Some(parsed_expr) = parsed else {
        state.push_warning(
            "W_SVPP_UNSUPPORTED_CONDITIONAL_EXPR",
            file,
            line,
            "unsupported `elsif conditional expression for configured policy",
            expr.to_string(),
        )?;
        return Ok(false);
    };
    match parsed_expr {
        ParsedConditionalExpr::Symbol(symbol) => {
            eval_symbol_condition(&symbol, state, file, line, "elsif")
        }
        ParsedConditionalExpr::Defined(symbol) => {
            eval_symbol_condition(&symbol, state, file, line, "elsif-defined")
        }
        ParsedConditionalExpr::NotDefined(symbol) => {
            let cond = eval_symbol_condition(&symbol, state, file, line, "elsif-not-defined")?;
            Ok(!cond)
        }
    }
}

/// `SVPP-EXPANSION.5`(q)(ii): a MACRO NAME — IEEE 1800-2017 §22.5.1 Syntax 22-2 `text_macro_identifier ::=
/// identifier`, and A.9.3's identifier is a simple OR an escaped one; 1800-2023 §22.5.1 says it outright
/// ("The macro name may be either a simple_identifier or an escaped_identifier"). An escaped identifier is a
/// backslash and the printable characters up to white space (§5.6.1), and "is treated the same as a
/// nonescaped identifier": the name returned is WITHOUT its backslash, so `\esc` and `esc` are one key.
/// For an escaped name exactly ONE terminating white-space character is consumed, so a `(` right after it
/// opens the formal list (1800-2023: "only the single white space character terminating the identifier
/// name … shall separate the opening parenthesis from the macro name"). The caller trims nothing before.
fn parse_macro_name_token(input: &str) -> Option<(&str, &str)> {
    if let Some(after_backslash) = input.strip_prefix('\\') {
        let end = after_backslash
            .find(|c: char| c.is_ascii_whitespace())
            .unwrap_or(after_backslash.len());
        if end == 0 {
            return None;
        }
        let name = &after_backslash[..end];
        let rest = &after_backslash[end..];
        let rest = rest
            .strip_prefix(|c: char| c.is_ascii_whitespace() && c != '\n' && c != '\r')
            .unwrap_or(rest);
        return Some((name, rest));
    }
    parse_identifier_token(input)
}

fn parse_identifier_token(input: &str) -> Option<(&str, &str)> {
    let mut chars = input.char_indices();
    let (_, first) = chars.next()?;
    if !(first.is_ascii_alphabetic() || first == '_') {
        return None;
    }
    let mut end = first.len_utf8();
    for (idx, ch) in chars {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '$' {
            end = idx + ch.len_utf8();
        } else {
            break;
        }
    }
    Some((&input[..end], &input[end..]))
}

/// `SVPP-EXPANSION.5`(p): does the parse profile name the IEEE 1800-2023 edition (`sv_2023`, and the
/// opt-in profile built on it)? Three sentences of 1800-2023 have no counterpart in 1800-2017:
/// §22.2 "Macro expansion shall occur within a compiler directive"; §22.5.1 "If a one-line comment or
/// block comment (see 5.4) is included in the text, then the comment shall not become part of the
/// substituted text"; and §22.5.1 "A newline character that is not contained in … a block comment and
/// is not preceded by a backslash shall end the macro text". The unprofiled reading is 1800-2017's,
/// as it is for keywords.
fn reads_1800_2023(config: &SvPreprocessorConfig) -> bool {
    version_for_profile(config.parse_profile.as_deref()) == KeywordVersion::Sv2023
}

/// `SVPP-EXPANSION.5`(p)(i): does this LOGICAL `` `define `` line — physical lines already joined by
/// `\n` — end inside a block comment? Unlike [`leaves_block_comment_open`], a `//` comment hides only
/// the rest of ITS physical line.
fn define_leaves_block_comment_open(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut i = 0usize;
    let (mut in_string, mut escaped, mut open) = (false, false, false);
    while i < bytes.len() {
        let b = bytes[i];
        if open {
            if b == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                open = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if b == b'"' {
            in_string = true;
        } else if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        } else if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            open = true;
            i += 2;
            continue;
        }
        i += 1;
    }
    open
}

/// `SVPP-EXPANSION.5`(p)(ii): IEEE 1800-2023 §22.5.1 — a block comment in macro text "shall not become
/// part of the substituted text". Each `/* … */` outside a string literal, a `` `"…`" `` macro string
/// and a `//` comment is replaced by ONE space, so the tokens on its two sides stay two tokens (the
/// paste operator is `` `` ``, never a comment). A `//` comment is copied through for
/// [`strip_line_comments_from_macro_text`] to cut; a comment built by pasting (`` /``* … *``/ ``) is
/// not a comment until the macro is used and is left alone.
fn drop_block_comments_from_macro_text(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    let (mut in_string, mut escaped, mut in_macro_string) = (false, false, false);
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_string = false;
            }
            out.push(b);
            i += 1;
            continue;
        }
        if text[i..].starts_with("`\\`\"") {
            out.extend_from_slice(&bytes[i..i + 4]);
            i += 4;
            continue;
        }
        if text[i..].starts_with("`\"") {
            in_macro_string = !in_macro_string;
            out.extend_from_slice(&bytes[i..i + 2]);
            i += 2;
            continue;
        }
        if in_macro_string {
            out.push(b);
            i += 1;
            continue;
        }
        if b == b'"' {
            in_string = true;
            out.push(b);
            i += 1;
            continue;
        }
        if text[i..].starts_with("//") {
            while i < bytes.len() && bytes[i] != b'\n' {
                out.push(bytes[i]);
                i += 1;
            }
            continue;
        }
        if text[i..].starts_with("/*") {
            match text[i + 2..].find("*/") {
                Some(close) => {
                    out.push(b' ');
                    i += 2 + close + 2;
                }
                None => {
                    // not closed inside the text: the caller's business (the logical line was joined
                    // through the comment, so this is reached only at end of file)
                    out.extend_from_slice(&bytes[i..]);
                    i = bytes.len();
                }
            }
            continue;
        }
        out.push(b);
        i += 1;
    }
    // every removed region begins and ends on an ASCII delimiter, so what is left is valid UTF-8
    String::from_utf8(out).unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned())
}

/// `SVPP-EXPANSION.5`(p)(iii): IEEE 1800-2023 §22.2 — "Macro expansion shall occur within a compiler
/// directive." The argument of a conditional directive (or of `` `undef ``) that is written as a macro
/// USAGE is expanded, and what it yields is the text the directive reads.
fn expand_directive_argument(
    rest: &str,
    state: &mut PreprocessorState,
    exp_file: &Path,
    exp_line: usize,
) -> Result<String> {
    expand_macros_in_text(
        rest,
        &state.macros,
        0,
        &mut state.events,
        exp_file,
        exp_line,
        &mut Vec::new(),
        None,
        state.config.plain_string_substitution,
    )
}

/// The macro text as the edition reads it BEFORE the one-line-comment cut: 1800-2023 drops block
/// comments (`SVPP-EXPANSION.5`(p)(ii)); 1800-2017 keeps them.
/// `SVPP-EXPANSION.5`(t): the compiler directives of the EDITION behind the profile — IEEE 1800-2017 §22.5.1
/// "All compiler directives shall be considered predefined macro names; it shall be illegal to redefine a
/// compiler directive as a macro name" (docs/systemverilog/2017/md/section-22-compiler-directives.md:139-140;
/// 1800-2023 :152-153). IEEE 1364-2005 §19.3.1 says the same of ITS directives ("Text macro names may not be the
/// same as compiler directive keywords", docs/verilog/2005/md/section-19-compiler-directives.md:100-101, :229),
/// and its §19 lists neither `undefineall` (1800 §22.5.3) nor `__FILE__` / `__LINE__` (1800 §22.13), so under a
/// Verilog profile those three are ordinary macro names. The set is matched as a whole name.
const DIRECTIVES_1364_2005: &[&str] = &[
    "begin_keywords", "celldefine", "default_nettype", "define", "else", "elsif", "end_keywords",
    "endcelldefine", "endif", "ifdef", "ifndef", "include", "line", "nounconnected_drive", "pragma",
    "resetall", "timescale", "unconnected_drive", "undef",
];
const DIRECTIVES_1800_ONLY: &[&str] = &["undefineall", "__FILE__", "__LINE__"];

fn is_compiler_directive_name(name: &str, version: KeywordVersion) -> bool {
    let sv = !matches!(
        version,
        KeywordVersion::Verilog1995
            | KeywordVersion::Verilog2001NoConfig
            | KeywordVersion::Verilog2001
            | KeywordVersion::Verilog2005
    );
    DIRECTIVES_1364_2005.contains(&name) || (sv && DIRECTIVES_1800_ONLY.contains(&name))
}

/// `SVPP-EXPANSION.5`(s): does the macro text leave a plain string literal OPEN at its end? IEEE 1800-2017 §22.5.1
/// "The text specified for macro text shall not be split across the following lexical tokens: … String literals"
/// and its own illegal example `` `define first_half "start of string `` (docs/systemverilog/2017/md/
/// section-22-compiler-directives.md:275-281, :295-297; 1800-2023 :307, :320-322; 1364-2005 §19.3.1 :172, :192,
/// :214-215). The scan is the stage's own: a `"` inside a `//` or `/* … */` comment or inside a `` `"…`" `` macro
/// string is not a delimiter, `\"` inside a string is an escape, and a backslash-newline inside a string continues
/// it (1800-2023 :176-177 names that exception). Macro strings (`` `" ``) have their own rules (`.5`(i)) and are
/// not judged here.
fn macro_text_leaves_string_open(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut i = 0usize;
    let (mut in_string, mut escaped, mut in_macro_string) = (false, false, false);
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if text[i..].starts_with("`\\`\"") {
            i += 4;
            continue;
        }
        if text[i..].starts_with("`\"") {
            in_macro_string = !in_macro_string;
            i += 2;
            continue;
        }
        if in_macro_string {
            i += 1;
            continue;
        }
        if b == b'"' {
            in_string = true;
            i += 1;
            continue;
        }
        if text[i..].starts_with("//") {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if text[i..].starts_with("/*") {
            match text[i + 2..].find("*/") {
                Some(close) => i += 2 + close + 2,
                None => i = bytes.len(),
            }
            continue;
        }
        i += 1;
    }
    in_string
}

fn macro_text_for_edition(text: &str, lrm_2023: bool) -> String {
    if lrm_2023 {
        drop_block_comments_from_macro_text(text)
    } else {
        text.to_string()
    }
}

fn parse_define_directive(
    input: &str,
    lrm_2023: bool,
    version: KeywordVersion,
) -> Result<(String, MacroDefinition)> {
    let trimmed = input.trim_end_matches(['\r', '\n']);
    let (name, rest_after_name) = parse_macro_name_token(trimmed.trim_start())
        .with_context(|| "missing macro name in `define")?;
    if is_compiler_directive_name(name, version) {
        bail!(
            "the macro name '{}' is a compiler directive — IEEE 1800-2017 §22.5.1: \"All compiler directives shall \
             be considered predefined macro names; it shall be illegal to redefine a compiler directive as a macro \
             name\" (IEEE 1364-2005 §19.3.1: \"Text macro names may not be the same as compiler directive keywords\")",
            name
        );
    }
    if rest_after_name.starts_with('(') {
        let (params, after) = parse_macro_parameter_list(rest_after_name)
            .with_context(|| "invalid macro parameter list")?;
        if macro_text_leaves_string_open(after) {
            bail!(
                "the macro text is split across a string literal — IEEE 1800-2017 §22.5.1: \"The text specified for \
                 macro text shall not be split across the following lexical tokens: … String literals\" (the \
                 standard's own illegal example is `define first_half \"start of string)"
            );
        }
        let text = macro_text_for_edition(after.trim_start(), lrm_2023);
        let body = normalize_macro_body_for_directives(&strip_line_comments_from_macro_text(&text));
        Ok((
            name.to_string(),
            MacroDefinition {
                params: Some(params),
                body,
            },
        ))
    } else {
        if macro_text_leaves_string_open(rest_after_name) {
            bail!(
                "the macro text is split across a string literal — IEEE 1800-2017 §22.5.1: \"The text specified for \
                 macro text shall not be split across the following lexical tokens: … String literals\" (the \
                 standard's own illegal example is `define first_half \"start of string)"
            );
        }
        Ok((
            name.to_string(),
            MacroDefinition {
                params: None,
                body: normalize_macro_body_for_directives(&strip_line_comments_from_macro_text(
                    &macro_text_for_edition(rest_after_name.trim_start(), lrm_2023),
                )),
            },
        ))
    }
}

/// `SVPP-EXPANSION.6b.2`(iii): IEEE 1800-2017 §22.5.1 — "If a one-line comment (that is, a comment
/// specified with the characters //) is included in the text, then the comment shall not become part
/// of the substituted text." Each line of the macro text is cut at a `//` that sits outside a string
/// literal, a `` `"…`" `` macro string and a block comment; a backslash that continued the line
/// survives the cut (the comment sat on the continued line, the continuation is the text's). Block
/// comments stay: the standard excludes one-line comments only, and `/* … */` is white space to the
/// parser. Before this, sv2v `macro.sv`'s `` `define NUMBER_12 21 */* foo */2//foo `` expanded with
/// its comment and the `)` of every use was commented out.
fn strip_line_comments_from_macro_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_block = false;
    for raw in split_lines_preserve_terminator(text) {
        let line = raw.trim_end_matches(['\r', '\n']);
        let terminator = &raw[line.len()..];
        let bytes = line.as_bytes();
        let mut in_string = false;
        let mut in_macro_string = false;
        let mut escaped = false;
        let mut cut: Option<usize> = None;
        let mut i = 0usize;
        while i < bytes.len() {
            let b = bytes[i];
            if in_block {
                if b == b'*' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    in_block = false;
                    i += 2;
                } else {
                    i += 1;
                }
                continue;
            }
            if in_string {
                if escaped {
                    escaped = false;
                } else if b == b'\\' {
                    escaped = true;
                } else if b == b'"' {
                    in_string = false;
                }
                i += 1;
                continue;
            }
            // `\`" — an escaped quote inside a macro string (§22.5.3.2): four bytes, no toggle
            if b == b'`' && line[i..].starts_with("`\\`\"") {
                i += 4;
                continue;
            }
            if b == b'`' && i + 1 < bytes.len() && bytes[i + 1] == b'"' {
                in_macro_string = !in_macro_string;
                i += 2;
                continue;
            }
            if in_macro_string {
                i += 1;
                continue;
            }
            if b == b'"' {
                in_string = true;
                i += 1;
                continue;
            }
            if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
                in_block = true;
                i += 2;
                continue;
            }
            if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                cut = Some(i);
                break;
            }
            i += 1;
        }
        match cut {
            Some(at) => {
                let kept = line[..at].trim_end();
                out.push_str(kept);
                if line.trim_end().ends_with('\\') && !kept.ends_with('\\') {
                    // the comment swallowed the continuation marker: the continuation is the text's
                    out.push_str(" \\");
                }
            }
            None => out.push_str(line),
        }
        out.push_str(terminator);
    }
    out
}

fn normalize_macro_body_for_directives(body: &str) -> String {
    let mut normalized = Vec::new();

    for raw_line in split_lines_preserve_terminator(body) {
        let line = raw_line.trim_end_matches(['\r', '\n']);
        let trimmed = line.trim_start_matches([' ', '\t']);
        let indent = &line[..line.len().saturating_sub(trimmed.len())];

        if let Some((directive, rest)) = parse_directive_line(trimmed) {
            match directive {
                "ifdef" | "ifndef" => {
                    if let Some((symbol, trailing)) = parse_identifier_token(rest.trim_start()) {
                        normalized.push(format!("{indent}`{directive} {symbol}"));
                        append_inline_conditional_payload(
                            indent,
                            trailing.trim_start(),
                            &mut normalized,
                        );
                        continue;
                    }
                }
                "else" => {
                    normalized.push(format!("{indent}`else"));
                    append_inline_conditional_payload(indent, rest.trim_start(), &mut normalized);
                    continue;
                }
                "endif" => {
                    normalized.push(format!("{indent}`endif"));
                    let trailing = rest.trim_start();
                    if !trailing.is_empty() {
                        normalized.push(format!("{indent}{trailing}"));
                    }
                    continue;
                }
                _ => {}
            }
        }

        normalized.push(line.to_string());
    }

    normalized.join("\n")
}

fn append_inline_conditional_payload(indent: &str, payload: &str, out: &mut Vec<String>) {
    let trimmed = payload.trim();
    if trimmed.is_empty() {
        return;
    }

    if let Some(prefix) = trimmed.strip_suffix("`endif") {
        let payload_prefix = prefix.trim_end();
        if !payload_prefix.is_empty() {
            out.push(format!("{indent}{payload_prefix}"));
        }
        out.push(format!("{indent}`endif"));
        return;
    }

    out.push(format!("{indent}{trimmed}"));
}

fn parse_macro_parameter_list(input: &str) -> Result<(Vec<MacroParameter>, &str)> {
    if !input.starts_with('(') {
        bail!("macro parameter list must start with '('");
    }
    // `SVPP-EXPANSION.5`(v): the list closes at the `)` matching its `(` OUTSIDE any string literal — a default
    // argument may be a string holding `)` or `(` (verilator t_preproc_def09.v:56 `(a="==)",b="((((",c=() )`);
    // the two sibling scanners were string-aware already, this one closed inside "==)" and handed the rest of
    // the list to the macro text.
    let mut depth = 0usize;
    let mut end_idx = None;
    let (mut in_string, mut escaped) = (false, false);
    for (idx, ch) in input.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '(' => depth += 1,
            ')' => {
                if depth == 0 {
                    bail!("unexpected ')' while parsing macro parameter list");
                }
                depth -= 1;
                if depth == 0 {
                    end_idx = Some(idx);
                    break;
                }
            }
            _ => {}
        }
    }
    let close_idx = end_idx.with_context(|| "unterminated macro parameter list")?;
    let param_segment = &input[1..close_idx];
    let mut params = Vec::new();
    for token in split_macro_parameter_tokens(param_segment) {
        let t = token.trim();
        if t.is_empty() {
            continue;
        }
        // `SVPP-EXPANSION.5`(o): a COMMENT between formals (verilator `t_preproc.v:277`, iverilog
        // `macro_with_args.v:17`) is refused BY THE CLAUSE, and the refusal says so: IEEE 1800-2017 /
        // 1800-2023 §22.5.1 — "The formal argument names shall be simple_identifiers, separated by commas
        // and optionally white space" — and §5.3's white space is spaces, tabs, newlines and formfeeds; the
        // standard writes "white space or a comment" where it admits one (§22.4). The verdict is unchanged
        // (these two shapes were refused before, as an "invalid token" / "invalid trailing text").
        if t.starts_with("//") || t.starts_with("/*") {
            bail!(formal_list_comment_refusal(t));
        }
        let (name, trailing) = parse_identifier_token(t)
            .with_context(|| format!("invalid macro parameter token '{}'", t))?;
        let trailing = trailing.trim();
        let default = if trailing.is_empty() {
            None
        } else if let Some(rest) = trailing.strip_prefix('=') {
            Some(rest.trim().to_string())
        } else if trailing.starts_with("//") || trailing.starts_with("/*") {
            bail!(formal_list_comment_refusal(trailing));
        } else {
            bail!("invalid trailing text in macro parameter '{}'", t);
        };
        params.push(MacroParameter {
            name: name.to_string(),
            default,
        });
    }
    Ok((params, &input[close_idx + 1..]))
}

/// `SVPP-EXPANSION.5`(o): the refusal of a comment inside a macro formal argument list, naming the clause.
fn formal_list_comment_refusal(at: &str) -> String {
    format!(
        "a comment inside a macro formal argument list is not admitted (IEEE 1800-2017 / 1800-2023 \
         §22.5.1: the formal argument names are \"separated by commas and optionally white space\"; §5.3: \
         white space is spaces, tabs, newlines and formfeeds, and the standard writes \"white space or a \
         comment\" where it admits one, §22.4); got '{}'",
        at.lines().next().unwrap_or(at).trim_end()
    )
}

fn split_macro_parameter_tokens(input: &str) -> Vec<&str> {
    let bytes = input.as_bytes();
    let mut tokens = Vec::new();
    let mut start = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    // `SVPP-EXPANSION.5`(g′): NO single-quote state. SystemVerilog has no single-quoted string
    // (§5.9: string literals are double-quoted); an apostrophe in a formal's default is a based
    // literal's (`COND_ = 1'b1`) or an assignment pattern's (`'{…}`), and treating it as a quote
    // swallowed every comma after it — OpenTitan's `DV_FCOV_INSTANTIATE_CG(NAME_, COND_ = 1'b1,
    // ARGS_ = ())` was read as TWO formals with a garbage default, and the third actual was silently
    // dropped since this scanner's first commit, 2026-03-17 (surfaced the moment extra actuals were refused).
    let mut in_double_quote = false;
    let mut escaped = false;
    let mut i = 0usize;

    while i < bytes.len() {
        // SV-CORPUS-GRAD.12c.2 — `bytes[i] as char` is a Latin-1 promotion, and it stays here
        // deliberately: this scanner never EMITS a character, it only compares against ASCII
        // literals and slices `&input[..]` at the ASCII delimiters it finds. A promoted
        // U+0080..=U+00FF matches none of those literals, so a multi-byte character is skipped a
        // byte at a time and both slice endpoints stay on character boundaries. Examined, correct
        // — unlike the six emitting sites this leaf fixed.
        let ch = bytes[i] as char;
        if escaped {
            escaped = false;
            i += 1;
            continue;
        }
        if ch == '\\' && in_double_quote {
            escaped = true;
            i += 1;
            continue;
        }
        if ch == '"' {
            in_double_quote = !in_double_quote;
            i += 1;
            continue;
        }
        if in_double_quote {
            i += 1;
            continue;
        }
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            ',' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                tokens.push(&input[start..i]);
                i += 1;
                start = i;
                continue;
            }
            _ => {}
        }
        i += 1;
    }
    tokens.push(&input[start..]);
    tokens
}

fn parse_include_directive(input: &str) -> Result<(&str, bool)> {
    let trimmed = input.trim();
    if let Some(rest) = trimmed.strip_prefix('"') {
        let quote_end = rest
            .find('"')
            .with_context(|| "unterminated quoted include path")?;
        let path = &rest[..quote_end];
        if path.is_empty() {
            bail!("empty quoted include path");
        }
        Ok((path, false))
    } else if let Some(rest) = trimmed.strip_prefix('<') {
        let angle_end = rest
            .find('>')
            .with_context(|| "unterminated angle include path")?;
        let path = &rest[..angle_end];
        if path.is_empty() {
            bail!("empty angle include path");
        }
        Ok((path, true))
    } else {
        // IEEE 1800-2017 §22.4 Syntax 22-1 admits `` `include "filename" `` and `` `include <filename> ``;
        // §22.5.1 adds that the directive "can be followed by a macro, instead of a string literal" — the
        // caller expands such a macro first (`SVPP-EXPANSION.5`(m)), so text reaching this arm is neither
        // form: an unquoted name, an undefined macro, or a macro whose text is not a filename.
        bail!(
            "include path must be quoted or angle-bracketed (IEEE 1800-2017 §22.4 Syntax 22-1: \
             `include \"filename\" | `include <filename>; §22.5.1: a macro after `include must expand \
             to one of the two); got '{}'",
            trimmed.trim_end_matches(['\r', '\n'])
        );
    }
}

fn resolve_include_path(
    include_token: &str,
    is_angle: bool,
    current_file: &Path,
    state: &PreprocessorState,
) -> Result<PathBuf> {
    let include_path = PathBuf::from(include_token);
    let mut candidates: Vec<PathBuf> = Vec::new();

    let current_parent = current_file
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    if include_path.is_absolute() {
        candidates.push(include_path);
    } else if is_angle {
        for dir in &state.config.include_dirs {
            candidates.push(dir.join(&include_path));
        }
        candidates.push(current_parent.join(&include_path));
    } else {
        candidates.push(current_parent.join(&include_path));
        for dir in &state.config.include_dirs {
            candidates.push(dir.join(&include_path));
        }
    }

    for candidate in candidates {
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    bail!("include '{}' was not found in search paths", include_token)
}

fn split_lines_preserve_terminator(content: &str) -> Vec<String> {
    if content.is_empty() {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let mut start = 0usize;
    for (idx, ch) in content.char_indices() {
        if ch == '\n' {
            lines.push(content[start..idx + 1].to_string());
            start = idx + 1;
        }
    }
    if start < content.len() {
        lines.push(content[start..].to_string());
    }
    lines
}

fn line_has_continuation(raw_line: &str) -> bool {
    raw_line.trim_end_matches(['\r', '\n']).ends_with('\\')
}

fn strip_line_continuation(raw_line: &str) -> &str {
    raw_line
        .trim_end_matches(['\r', '\n'])
        .strip_suffix('\\')
        .unwrap_or(raw_line)
}

fn contains_line_start_backtick(text: &str) -> bool {
    split_lines_preserve_terminator(text)
        .iter()
        .map(|line| line.trim_start_matches([' ', '\t']))
        .any(|trimmed| trimmed.starts_with('`'))
}

/// `SVPP-EXPANSION.5`(h): `expanding` is the stack of macro names whose BODY is being rescanned;
/// a usage of a name on that stack is the recursion IEEE 1800-2017 §22.5.1 forbids — *"It shall be
/// an error for a macro to expand directly or indirectly to text containing another usage of
/// itself"* — UNLESS the caller wrote the usage: *"an actual argument to a macro or a default may
/// contain a usage of itself"* (the standard's own `` `TOP( `TOP(b,1), `TOP(42,a) ) `` example).
/// `user_mask` is that PROVENANCE, one flag per byte of `text`: `None` = the whole text is what
/// the caller wrote (a source line); `Some(mask)` = an expansion's text, `true` on the bytes that
/// came in through an actual or a default (transitively — an actual's bytes keep the flags they had
/// where they were written) and `false` on the bytes the macro's body produced. A user-written
/// usage starts its OWN expansion context (a fresh stack): the caller wrote finitely many of them,
/// so they cannot loop; a body-produced usage is checked against the stack. Byte ranges alone were
/// not enough — OpenTitan nests `` `DV_SPINWAIT( … `DV_SPINWAIT(…) … ) ``, and the inner usage's
/// body re-enters `` `DV_SPINWAIT_EXIT `` while the OUTER one is still on the stack: legal, since
/// the inner usage was written by the caller; and `` `define A(x) `B(`A(x)) `` must still refuse,
/// since the `` `A `` handed to `B` is A's own body text. Before this leaf a recursive macro was
/// expanded until the depth guard, and one naming itself twice expanded exponentially (verilator
/// `t_pp_circdef_bad.v` timed out at 60 s).
fn expand_macros_in_text(
    text: &str,
    macros: &HashMap<String, MacroDefinition>,
    depth: usize,
    events: &mut Vec<PreprocessorEvent>,
    file: &Path,
    line: usize,
    expanding: &mut Vec<String>,
    user_mask: Option<&[bool]>,
    plain_strings: PlainStringSubstitutionPolicy,
) -> Result<String> {
    if depth > 16 {
        return Ok(text.to_string());
    }

    let bytes = text.as_bytes();
    let mut i = 0usize;
    let mut out = String::new();
    let mut expanded_any = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut in_double_quote = false;
    let mut escaped_in_string = false;

    while i < bytes.len() {
        if in_line_comment {
            // SV-CORPUS-GRAD.12c.2 — copy the whole UTF-8 sequence; see `source_char_at`.
            // Comments are where the LRM permits non-ASCII, so this is the site that mattered.
            let ch = source_char_at(text, i);
            out.push(ch);
            i += ch.len_utf8();
            if ch == '\n' {
                in_line_comment = false;
            }
            continue;
        }

        if in_block_comment {
            if i + 1 < bytes.len() && bytes[i] == b'*' && bytes[i + 1] == b'/' {
                out.push('*');
                out.push('/');
                i += 2;
                in_block_comment = false;
            } else {
                let ch = source_char_at(text, i);
                out.push(ch);
                i += ch.len_utf8();
            }
            continue;
        }

        if in_double_quote {
            let ch = source_char_at(text, i);
            out.push(ch);
            i += ch.len_utf8();
            if escaped_in_string {
                escaped_in_string = false;
            } else if ch == '\\' {
                escaped_in_string = true;
            } else if ch == '"' {
                in_double_quote = false;
            }
            continue;
        }

        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            out.push('/');
            out.push('/');
            i += 2;
            in_line_comment = true;
            continue;
        }

        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            out.push('/');
            out.push('*');
            i += 2;
            in_block_comment = true;
            continue;
        }

        if bytes[i] == b'"' {
            out.push('"');
            i += 1;
            in_double_quote = true;
            continue;
        }

        if bytes[i] != b'`' {
            let ch = source_char_at(text, i);
            out.push(ch);
            i += ch.len_utf8();
            continue;
        }

        if i + 1 < bytes.len() && bytes[i + 1] == b'`' {
            // `SVPP-EXPANSION.6b.2`(i): in MACRO-PRODUCED text the ``` `` ``` token has done its work
            // — "A `` delimits lexical tokens without introducing white space" (IEEE 1800-2017
            // §22.5.1) — so an object macro's `/``* … *``/` joins to `/* … */` exactly as a function
            // macro's sides join in substitution (sv2v `macro_vendor_comment.sv` reached the parser
            // with the token still in it); in a source line it stays, for the parser to judge.
            if user_mask.is_some() {
                i += 2;
                continue;
            }
            out.push_str("``");
            i += 2;
            continue;
        }

        if i + 1 < bytes.len() && bytes[i + 1] == b'"' {
            // `SVPP-EXPANSION.5`(j): in EXPANSION text a `` `"…`" `` region is rendered here, ONCE,
            // for the object-macro path (whose body never passed through a substitution step and
            // so reached the parser with the markers still on — OpenTitan `otbn_ssctrl_if.sv`'s
            // `` `define SSCTRL_PATH `"`SSCTRL_HPATH`" `` was refused at the `{` that held it) and
            // for the function-macro path (whose substitution now emits the region with its markers
            // and the actuals in place). A region in a SOURCE line stays as written: §22.5.1 defines
            // the form for macro text, and sv2v `string_macro.sv:9` is pinned must-reject on it.
            if user_mask.is_some() {
                if let Some(close) = find_macro_string_close(text, i + 2) {
                    let (rendered, any_expanded) = render_macro_string_region(
                        text,
                        i + 2,
                        close,
                        macros,
                        depth,
                        events,
                        file,
                        line,
                        expanding,
                        user_mask,
                        plain_strings,
                    )?;
                    out.push_str(&rendered);
                    if any_expanded {
                        expanded_any = true;
                    }
                    i = close + 2;
                    continue;
                }
            }
            out.push_str("`\"");
            i += 2;
            continue;
        }

        let mut ident_start = i + 1;
        // `SVPP-EXPANSION.5`(q)(ii): a usage spelled with an ESCAPED name — a backtick, a backslash and
        // the characters up to white space (`` `\esc ``, `` `\M@CRO3 ( 1 ) ``, iverilog's `` `\`name ``).
        // Recognised only where the run IS a defined macro name, so the escaped-quote token `` `\`" ``
        // of a macro string keeps its meaning wherever that run is not a name someone defined.
        let mut escaped_run_end: Option<usize> = None;
        if ident_start < bytes.len() && bytes[ident_start] == b'\\' {
            let mut e = ident_start + 1;
            while e < bytes.len() && !bytes[e].is_ascii_whitespace() {
                e += 1;
            }
            if e > ident_start + 1 && macros.contains_key(&text[ident_start + 1..e]) {
                escaped_run_end = Some(e);
                ident_start += 1;
            }
        }
        if escaped_run_end.is_none()
            && (ident_start >= bytes.len() || !is_ident_start(bytes[ident_start]))
        {
            out.push('`');
            i += 1;
            continue;
        }
        let mut ident_end = match escaped_run_end {
            Some(e) => e,
            None => {
                let mut e = ident_start + 1;
                while e < bytes.len() && is_ident_continue(bytes[e]) {
                    e += 1;
                }
                e
            }
        };
        let usage_start = i;
        // `SVPP-EXPANSION.5`(i): an UNDEFINED usage immediately followed by a ``` `` ``` paste is the
        // LEFT END of a CONSTRUCTED usage — `` `CHOICE_``A```SUFFIX `` (sv2v `macro_macro.sv`),
        // `` `uvm_pack_``TYPE``(ARG, p) `` (UVM), `` ```rname``_ADR `` (Surelog). §22.5.1: a paste
        // "delimits lexical tokens without introducing white space, allowing identifiers to be
        // constructed"; the identifier being constructed is the MACRO NAME, so the chain is joined
        // HERE, with every operand that is itself a defined usage expanded first (Verilator's
        // `` `A``b```C `` → `abc`, `` XXS_```SOME `` → `XXS_some`; sv2v's `` `CHOICE_``FOO```SUFFIX ``
        // → `` `CHOICE_FOO_MAGIC ``), and the WHOLE constructed name is then looked up below, in
        // place, so a function-like result collects its arguments still UNEXPANDED (the order
        // §22.5.1 gives every actual). Before this, `substitute_function_macro_body` deleted the
        // paste tokens while it substituted, so `` `CHOICE_``FOO```SUFFIX `` reached this scan as
        // `` `CHOICE_FOO`SUFFIX `` and the FIRST defined prefix won: `1_MAGIC` where sv2v and
        // Verilator both read `3`. A DEFINED left operand never gets here: it is expanded by the
        // ordinary path and the paste after it is dropped by the loop, which is the same
        // expand-first reading (`` `A``_b `` with `A` and `A_b` both defined is `a_b` in Verilator,
        // Icarus and here). A source line (`user_mask` = `None`) keeps its paste for the parser.
        let mut joined: std::borrow::Cow<'_, str> =
            std::borrow::Cow::Borrowed(&text[ident_start..ident_end]);
        if escaped_run_end.is_none()
            && user_mask.is_some()
            && !macros.contains_key(joined.as_ref())
            && text[ident_end..].starts_with("``")
        {
            let (suffix, chain_end, operand_expanded) = join_paste_chain(
                text,
                ident_end,
                macros,
                depth,
                events,
                file,
                line,
                expanding,
                user_mask,
                plain_strings,
            )?;
            if operand_expanded {
                expanded_any = true;
            }
            joined = std::borrow::Cow::Owned(format!("{joined}{suffix}"));
            ident_end = chain_end;
        }
        let name: &str = joined.as_ref();
        let Some(def) = macros.get(name) else {
            if let Some(expanded) = expand_builtin_macro(name, file, line) {
                out.push_str(&expanded);
                i = ident_end;
                expanded_any = true;
                continue;
            }
            out.push('`');
            out.push_str(name);
            i = ident_end;
            continue;
        };

        // (h) provenance decides: a usage the CALLER wrote — a source line, or bytes that came in
        // through an actual or a default — starts its own expansion context below; a usage a macro
        // BODY produced is checked against the stack of bodies being rescanned.
        let user_written = user_mask.is_none_or(|m| m.get(usage_start).copied().unwrap_or(false));
        if !user_written && expanding.iter().any(|open| open == name) {
            let chain = expanding
                .iter()
                .skip_while(|open| open.as_str() != name)
                .map(|open| format!("`{}", open))
                .collect::<Vec<_>>()
                .join(" -> ");
            bail!(
                "E_SVPP_RECURSIVE_MACRO at {}:{}: macro '{}' expands to a usage of itself ({} -> `{}) \
                 — IEEE 1800-2017 §22.5.1: \"It shall be an error for a macro to expand directly or \
                 indirectly to text containing another usage of itself\"",
                file.display(),
                line,
                name,
                chain,
                name
            );
        }

        if let Some(params) = &def.params {
            let mut j = ident_end;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            let Some((arg_ranges, next_index)) = parse_macro_invocation_arg_ranges(text, j) else {
                out.push('`');
                out.push_str(name);
                i = ident_end;
                continue;
            };
            let args: Vec<String> = arg_ranges
                .iter()
                .map(|&(s, e)| text[s..e].to_string())
                .collect();
            // each actual keeps the provenance of the bytes it was written in
            let arg_masks: Vec<Vec<bool>> = arg_ranges
                .iter()
                .map(|&(s, e)| match user_mask {
                    None => vec![true; e - s],
                    Some(m) => (s..e).map(|k| m.get(k).copied().unwrap_or(false)).collect(),
                })
                .collect();
            // `SVPP-EXPANSION.5`(l): a one-line comment written inside an actual is not part of
            // the substituted text (the mask stays aligned byte for byte)
            let (args, arg_masks): (Vec<String>, Vec<Vec<bool>>) = args
                .into_iter()
                .zip(arg_masks)
                .map(|(arg, mask)| strip_line_comments_from_actual(&arg, &mask))
                .unzip();
            let mut fresh: Vec<String> = Vec::new();
            let stack: &mut Vec<String> = if user_written {
                &mut fresh
            } else {
                &mut *expanding
            };
            stack.push(name.to_string());
            let expanded = expand_function_macro(
                def,
                params,
                &args,
                &arg_masks,
                macros,
                depth + 1,
                file,
                line,
                name,
                stack,
                plain_strings,
            );
            stack.pop();
            let expanded = expanded?;
            events.push(PreprocessorEvent {
                kind: PreprocessorEventKind::MacroExpand,
                file: file.display().to_string(),
                line,
                active: true,
                detail: format!("`{}({})", name, args.join(", ")),
            });
            out.push_str(&expanded);
            i = next_index;
            expanded_any = true;
        } else {
            let mut fresh: Vec<String> = Vec::new();
            let stack: &mut Vec<String> = if user_written {
                &mut fresh
            } else {
                &mut *expanding
            };
            stack.push(name.to_string());
            let expanded =
                expand_object_macro(def, macros, depth + 1, file, line, stack, plain_strings);
            stack.pop();
            let expanded = expanded?;
            events.push(PreprocessorEvent {
                kind: PreprocessorEventKind::MacroExpand,
                file: file.display().to_string(),
                line,
                active: true,
                detail: format!("`{}", name),
            });
            out.push_str(&expanded);
            i = ident_end;
            expanded_any = true;
        }
    }

    if expanded_any && out != text {
        // a second pass over EXPANSION text carries no provenance: what an expansion produced is
        // body text. A second pass over a SOURCE line stays a source line (`None`): every region
        // and paste an expansion carried was consumed inside that expansion's own rescan, so the
        // ones still on the line are the author's, and §22.5.1 defines both forms for macro text —
        // rendering them here turned sv2v `string_macro.sv:9`, a pinned must-reject, into an accept
        // the moment a usage shared its line (`SVPP-EXPANSION.5`(j)).
        let second_mask: Option<&[bool]> = if user_mask.is_none() { None } else { Some(&[]) };
        expand_macros_in_text(
            &out,
            macros,
            depth + 1,
            events,
            file,
            line,
            expanding,
            second_mask,
            plain_strings,
        )
    } else {
        Ok(out)
    }
}

/// Expand the macro usage whose backtick is at `at`. Returns the expansion and the index just past
/// the usage — its argument list included for a function-like macro — or `None` when the name is
/// undefined (and no builtin), or function-like with no parseable argument list, or not a name at
/// all; the caller then keeps the bytes as written. The expansion runs through
/// [`expand_macros_in_text`] on exactly the usage's own bytes and its own slice of `user_mask`, so
/// provenance, the `.5`(h) recursion check and the depth guard are the ordinary ones. Shared by the
/// paste-chain walker (`.5`(i)) and the string-region renderer (`.5`(j)).
#[allow(clippy::too_many_arguments)] // two call sites; the arguments ARE the expansion's context
fn expand_usage_at(
    text: &str,
    at: usize,
    macros: &HashMap<String, MacroDefinition>,
    depth: usize,
    events: &mut Vec<PreprocessorEvent>,
    file: &Path,
    line: usize,
    expanding: &mut Vec<String>,
    user_mask: Option<&[bool]>,
    plain_strings: PlainStringSubstitutionPolicy,
) -> Result<Option<(String, usize)>> {
    let bytes = text.as_bytes();
    let n_start = at + 1;
    if at >= bytes.len()
        || bytes[at] != b'`'
        || n_start >= bytes.len()
        || !is_ident_start(bytes[n_start])
    {
        return Ok(None);
    }
    let mut n_end = n_start + 1;
    while n_end < bytes.len() && is_ident_continue(bytes[n_end]) {
        n_end += 1;
    }
    let name = &text[n_start..n_end];
    let end = match macros.get(name) {
        Some(def) if def.params.is_some() => {
            // a function-like usage takes its argument list along
            let mut k = n_end;
            while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                k += 1;
            }
            match parse_macro_invocation_arg_ranges(text, k) {
                Some((_, next)) => next,
                None => return Ok(None),
            }
        }
        Some(_) => n_end,
        None => {
            return Ok(expand_builtin_macro(name, file, line).map(|expanded| (expanded, n_end)));
        }
    };
    // the usage's own bytes, with the provenance they carry (a second pass carries none:
    // `Some(&[])` stays `Some(&[])`)
    let sub_mask: Option<&[bool]> =
        user_mask.map(|m| if m.len() >= end { &m[at..end] } else { &[][..] });
    let expansion = expand_macros_in_text(
        &text[at..end],
        macros,
        depth + 1,
        events,
        file,
        line,
        expanding,
        sub_mask,
        plain_strings,
    )?;
    Ok(Some((expansion, end)))
}

/// `SVPP-EXPANSION.5`(j): the index of the ``` `" ``` that CLOSES a macro-string region whose
/// content starts at `from`, or `None`. An escaped quote ``` `\`" ``` (four bytes) is stepped over
/// whole — its last two bytes spell the closer, which is how the previous renderer ended the
/// standard's own `msg` example at the first escaped quote.
fn find_macro_string_close(text: &str, from: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut j = from;
    while j + 1 < bytes.len() {
        if text[j..].starts_with("`\\`\"") {
            j += 4;
            continue;
        }
        if bytes[j] == b'`' && bytes[j + 1] == b'"' {
            return Some(j);
        }
        j += 1;
    }
    None
}

/// `SVPP-EXPANSION.5`(j): render the macro-string region `text[start..close]` as ONE string
/// literal — IEEE 1800-2017 §22.5.1: *"An `" overrides the usual lexical meaning of " and indicates
/// that the expansion shall include the quotation mark, substitution of actual arguments, and
/// expansions of embedded macros."* The actuals were substituted before this text reached the rescan;
/// here an embedded usage (`` `SSCTRL_HPATH ``, `` `FOO ``, a builtin) is expanded through
/// [`expand_usage_at`], an escaped quote ``` `\`" ``` becomes `\"` (the clause's own
/// `"left side: \"right side\""`), a paste joins its sides, and every other byte — a plain `"`
/// included — is the string's. An undefined usage, or a function-like one with no argument list, is
/// kept as written (Surelog's `PreprocString` golden keeps its bare `` `message ``). Returns the
/// literal and whether any embedded usage was expanded.
#[allow(clippy::too_many_arguments)]
fn render_macro_string_region(
    text: &str,
    start: usize,
    close: usize,
    macros: &HashMap<String, MacroDefinition>,
    depth: usize,
    events: &mut Vec<PreprocessorEvent>,
    file: &Path,
    line: usize,
    expanding: &mut Vec<String>,
    user_mask: Option<&[bool]>,
    plain_strings: PlainStringSubstitutionPolicy,
) -> Result<(String, bool)> {
    let bytes = text.as_bytes();
    let mut out = String::from("\"");
    let mut any_expanded = false;
    let mut k = start;
    while k < close {
        if text[k..close].starts_with("`\\`\"") {
            out.push_str("\\\"");
            k += 4;
            continue;
        }
        if text[k..close].starts_with("``") {
            k += 2;
            continue;
        }
        if bytes[k] == b'`' && k + 1 < close && is_ident_start(bytes[k + 1]) {
            match expand_usage_at(
                text,
                k,
                macros,
                depth,
                events,
                file,
                line,
                expanding,
                user_mask,
                plain_strings,
            )? {
                Some((expansion, end)) if end <= close => {
                    out.push_str(&expansion);
                    any_expanded = true;
                    k = end;
                    continue;
                }
                _ => {
                    out.push('`');
                    k += 1;
                    continue;
                }
            }
        }
        let ch = source_char_at(text, k);
        out.push(ch);
        k += ch.len_utf8();
    }
    out.push('"');
    // A string literal is ONE line (§5.9), and a macro body's backslash-newline "shall be replaced in
    // the expanded macro with a newline character" (§22.5.1) — so an embedded usage whose body spans
    // lines (OpenTitan's `` `DV_LC_TX_DIST `` inside `` `DV_CHECK_STD_RANDOMIZE_WITH_FATAL ``'s
    // stringified condition) would put a raw newline INSIDE the literal, which the parser refuses.
    // Verilator renders the newline as a space (its own `twoline` golden: `"first        second"`);
    // Icarus leaves it raw. The one-line reading is the only one the parser can read, so every
    // newline in the rendered literal becomes a space, the indentation it carried kept.
    let out = if out.contains('\n') {
        out.replace("\r\n", " ").replace('\n', " ")
    } else {
        out
    };
    Ok((out, any_expanded))
}

/// `SVPP-EXPANSION.5`(i): join the ``` `` ``` paste chain that follows the left operand of a
/// CONSTRUCTED macro usage (see the call site in [`expand_macros_in_text`]). `start` is the index of
/// the first paste token. Returns the joined suffix, the index just past the chain, and whether any
/// operand was a defined macro usage that was expanded on the way.
///
/// Operands, left to right, each preceded by its paste: an identifier fragment (`_ADR`, `3`) is
/// appended as written; a DEFINED macro usage (`` `SUFFIX ``, `` `SOMEP(foo) ``) is expanded — through
/// [`expand_macros_in_text`] on exactly its own bytes, so provenance, the recursion check and the
/// depth guard are the ordinary ones — and its expansion appended; an UNDEFINED usage is kept as
/// written, backtick and all, so the constructed name fails its lookup and the parser reports the
/// byte rather than this stage guessing. A paste that is followed by neither — `` `X``( `` — joins
/// nothing and ends the chain (`` `uvm_pack_``int``(ARG, p) ``: the parentheses belong to the
/// constructed usage, which the caller looks up at the returned index).
#[allow(clippy::too_many_arguments)] // one call site; the arguments ARE the expansion's context
fn join_paste_chain(
    text: &str,
    start: usize,
    macros: &HashMap<String, MacroDefinition>,
    depth: usize,
    events: &mut Vec<PreprocessorEvent>,
    file: &Path,
    line: usize,
    expanding: &mut Vec<String>,
    user_mask: Option<&[bool]>,
    plain_strings: PlainStringSubstitutionPolicy,
) -> Result<(String, usize, bool)> {
    let bytes = text.as_bytes();
    let mut j = start;
    let mut suffix = String::new();
    let mut operand_expanded = false;
    while text[j..].starts_with("``") {
        let after = j + 2;
        if after < bytes.len() && bytes[after] == b'`' {
            let n_start = after + 1;
            if n_start < bytes.len() && is_ident_start(bytes[n_start]) {
                let mut n_end = n_start + 1;
                while n_end < bytes.len() && is_ident_continue(bytes[n_end]) {
                    n_end += 1;
                }
                match expand_usage_at(
                    text,
                    after,
                    macros,
                    depth,
                    events,
                    file,
                    line,
                    expanding,
                    user_mask,
                    plain_strings,
                )? {
                    Some((expansion, op_end)) => {
                        suffix.push_str(&expansion);
                        operand_expanded = true;
                        j = op_end;
                    }
                    None => {
                        suffix.push('`');
                        suffix.push_str(&text[n_start..n_end]);
                        j = n_end;
                    }
                }
                continue;
            }
            // "````" — a paste followed by another: this one joins nothing
            j = after;
            continue;
        }
        if after < bytes.len() && is_ident_continue(bytes[after]) {
            let mut k = after;
            while k < bytes.len() && is_ident_continue(bytes[k]) {
                k += 1;
            }
            suffix.push_str(&text[after..k]);
            j = k;
            continue;
        }
        // `` `X``( `` — the token after the paste is not a name: the paste joins nothing, the chain ends
        j = after;
        break;
    }
    Ok((suffix, j, operand_expanded))
}

fn has_unclosed_function_macro_invocation(
    text: &str,
    macros: &HashMap<String, MacroDefinition>,
) -> bool {
    let bytes = text.as_bytes();
    let mut i = 0usize;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut in_double_quote = false;
    let mut escaped_in_string = false;

    while i < bytes.len() {
        if in_line_comment {
            if bytes[i] == b'\n' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }

        if in_block_comment {
            if i + 1 < bytes.len() && bytes[i] == b'*' && bytes[i + 1] == b'/' {
                in_block_comment = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }

        if in_double_quote {
            if escaped_in_string {
                escaped_in_string = false;
            } else if bytes[i] == b'\\' {
                escaped_in_string = true;
            } else if bytes[i] == b'"' {
                in_double_quote = false;
            }
            i += 1;
            continue;
        }

        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            in_line_comment = true;
            i += 2;
            continue;
        }

        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            in_block_comment = true;
            i += 2;
            continue;
        }

        if bytes[i] == b'"' {
            in_double_quote = true;
            i += 1;
            continue;
        }

        if bytes[i] != b'`' {
            i += 1;
            continue;
        }

        if i + 1 < bytes.len() && (bytes[i + 1] == b'`' || bytes[i + 1] == b'"') {
            i += 2;
            continue;
        }

        let ident_start = i + 1;
        if ident_start >= bytes.len() || !is_ident_start(bytes[ident_start]) {
            i += 1;
            continue;
        }

        let mut ident_end = ident_start + 1;
        while ident_end < bytes.len() && is_ident_continue(bytes[ident_end]) {
            ident_end += 1;
        }

        let name = &text[ident_start..ident_end];
        let Some(def) = macros.get(name) else {
            i = ident_end;
            continue;
        };

        if def.params.is_some() {
            let mut open_paren_idx = ident_end;
            while open_paren_idx < bytes.len() && bytes[open_paren_idx].is_ascii_whitespace() {
                open_paren_idx += 1;
            }
            if open_paren_idx >= bytes.len() {
                // `SVPP-EXPANSION.6b.2`(iv): the name closes the line, so its `(` may open the next
                // one — §22.5.1 requires the parentheses, §5.3 lets white space, newlines included,
                // separate the tokens (sv2v `macro_whitespace.sv`): read on before deciding.
                return true;
            }
            if open_paren_idx < bytes.len() && bytes[open_paren_idx] == b'(' {
                if let Some((_, next_index)) = parse_macro_invocation_args(text, open_paren_idx) {
                    i = next_index;
                    continue;
                }
                return true;
            }
        }

        i = ident_end;
    }

    false
}

/// `SVPP-EXPANSION.5`(l): an actual with its one-line comments removed, its provenance mask kept
/// aligned byte for byte. IEEE 1800-2017 §22.5.1 says of the macro text that a `//` comment "shall
/// not become part of the substituted text"; an actual argument is text that gets substituted, and
/// `parse_macro_invocation_arg_ranges` already treats the comment as one — a comma or a `)` inside
/// it does not delimit — while leaving its bytes in the range. Substituted onto the expansion's
/// single line, the comment then swallowed the rest of the call: OpenTitan's
/// `` `DV_CHECK_RANDOMIZE_WITH_FATAL(req, … // TODO(#15721), dummy CSB … ) `` expanded to
/// `if (!(… // TODO…)) …` and the parser met an unbalanced line (`spi_agent_pkg`,
/// `alert_esc_agent_pkg`, `core_ibex_test_pkg`). The state machine is the scanner's own, so the two
/// agree on what a comment is: a `//` outside a string literal and outside a block comment runs to
/// the line feed, which stays (it separates what the caller wrote after it). Block comments stay —
/// they are white space to the parser — and a `//` inside a string literal is the string's.
fn strip_line_comments_from_actual(actual: &str, mask: &[bool]) -> (String, Vec<bool>) {
    if !actual.contains("//") {
        return (actual.to_string(), mask.to_vec());
    }
    let bytes = actual.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut out_mask: Vec<bool> = Vec::with_capacity(bytes.len());
    let keep = |out: &mut Vec<u8>, out_mask: &mut Vec<bool>, k: usize| {
        out.push(bytes[k]);
        out_mask.push(mask.get(k).copied().unwrap_or(true));
    };
    let mut in_double_quote = false;
    let mut in_block_comment = false;
    let mut escaped = false;
    let mut i = 0usize;
    while i < bytes.len() {
        if in_block_comment {
            if i + 1 < bytes.len() && bytes[i] == b'*' && bytes[i + 1] == b'/' {
                keep(&mut out, &mut out_mask, i);
                keep(&mut out, &mut out_mask, i + 1);
                in_block_comment = false;
                i += 2;
            } else {
                keep(&mut out, &mut out_mask, i);
                i += 1;
            }
            continue;
        }
        if escaped {
            escaped = false;
            keep(&mut out, &mut out_mask, i);
            i += 1;
            continue;
        }
        if bytes[i] == b'\\' && in_double_quote {
            escaped = true;
            keep(&mut out, &mut out_mask, i);
            i += 1;
            continue;
        }
        if bytes[i] == b'"' {
            in_double_quote = !in_double_quote;
            keep(&mut out, &mut out_mask, i);
            i += 1;
            continue;
        }
        if in_double_quote {
            keep(&mut out, &mut out_mask, i);
            i += 1;
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            // the comment runs to the line feed; the line feed itself is the caller's white space
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            keep(&mut out, &mut out_mask, i);
            keep(&mut out, &mut out_mask, i + 1);
            in_block_comment = true;
            i += 2;
            continue;
        }
        keep(&mut out, &mut out_mask, i);
        i += 1;
    }
    (
        String::from_utf8(out)
            .expect("removing a whole `//…` run up to a line feed keeps UTF-8 valid"),
        out_mask,
    )
}

fn parse_macro_invocation_args(input: &str, open_paren_idx: usize) -> Option<(Vec<String>, usize)> {
    let (ranges, next) = parse_macro_invocation_arg_ranges(input, open_paren_idx)?;
    Some((
        ranges
            .iter()
            .map(|&(s, e)| input[s..e].to_string())
            .collect(),
        next,
    ))
}

/// The byte range of `input[start..end]` with its surrounding white space trimmed off (empty when
/// the segment is blank), so an actual's bytes can be located in the text they were written in.
fn trimmed_range(input: &str, start: usize, end: usize) -> (usize, usize) {
    let segment = &input[start..end];
    let lead = segment.len() - segment.trim_start().len();
    let trail = segment.len() - segment.trim_end().len();
    let s = start + lead;
    (s, (end - trail).max(s))
}

/// `SVPP-EXPANSION.5`(h): the actuals of a function-like macro usage as BYTE RANGES of `input`
/// (trimmed), so the rescan can carry each byte's provenance — whether the caller wrote it or a
/// macro body produced it — into the expansion.
fn parse_macro_invocation_arg_ranges(
    input: &str,
    open_paren_idx: usize,
) -> Option<(Vec<(usize, usize)>, usize)> {
    let bytes = input.as_bytes();
    if open_paren_idx >= bytes.len() || bytes[open_paren_idx] != b'(' {
        return None;
    }
    let mut paren_depth = 1usize;
    let mut brace_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut i = open_paren_idx + 1;
    let mut start = open_paren_idx + 1;
    let mut args = Vec::new();
    let mut in_double_quote = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut escaped = false;

    while i < bytes.len() {
        // SV-CORPUS-GRAD.12c.2 — compare-only, like `split_macro_parameter_tokens`: this scanner
        // emits nothing and slices at ASCII delimiters, so the Latin-1 promotion is harmless here.
        let ch = bytes[i] as char;
        if in_line_comment {
            if ch == '\n' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }
        if in_block_comment {
            if i + 1 < bytes.len() && bytes[i] == b'*' && bytes[i + 1] == b'/' {
                in_block_comment = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        if escaped {
            escaped = false;
            i += 1;
            continue;
        }
        if ch == '\\' && in_double_quote {
            escaped = true;
            i += 1;
            continue;
        }
        if ch == '"' {
            in_double_quote = !in_double_quote;
            i += 1;
            continue;
        }
        if in_double_quote {
            i += 1;
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            in_line_comment = true;
            i += 2;
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            in_block_comment = true;
            i += 2;
            continue;
        }
        match ch {
            '(' => {
                paren_depth += 1;
                i += 1;
            }
            ')' => {
                if paren_depth == 0 {
                    return None;
                }
                paren_depth -= 1;
                if paren_depth == 0 {
                    let (s, e) = trimmed_range(input, start, i);
                    if s < e || !args.is_empty() {
                        args.push((s, e));
                    }
                    return Some((args, i + 1));
                }
                i += 1;
            }
            '{' => {
                brace_depth += 1;
                i += 1;
            }
            '}' => {
                brace_depth = brace_depth.saturating_sub(1);
                i += 1;
            }
            '[' => {
                bracket_depth += 1;
                i += 1;
            }
            ']' => {
                bracket_depth = bracket_depth.saturating_sub(1);
                i += 1;
            }
            ',' if paren_depth == 1 && brace_depth == 0 && bracket_depth == 0 => {
                args.push(trimmed_range(input, start, i));
                i += 1;
                start = i;
            }
            _ => i += 1,
        }
    }
    None
}

fn expand_object_macro(
    definition: &MacroDefinition,
    macros: &HashMap<String, MacroDefinition>,
    depth: usize,
    file: &Path,
    line: usize,
    expanding: &mut Vec<String>,
    plain_strings: PlainStringSubstitutionPolicy,
) -> Result<String> {
    expand_macros_in_text(
        &definition.body,
        macros,
        depth,
        &mut Vec::new(),
        file,
        line,
        expanding,
        Some(&[]),
        plain_strings,
    )
}

#[allow(clippy::too_many_arguments)] // one call site; the arguments ARE the expansion's context
fn expand_function_macro(
    definition: &MacroDefinition,
    params: &[MacroParameter],
    args: &[String],
    arg_masks: &[Vec<bool>],
    macros: &HashMap<String, MacroDefinition>,
    depth: usize,
    file: &Path,
    line: usize,
    name: &str,
    expanding: &mut Vec<String>,
    plain_strings: PlainStringSubstitutionPolicy,
) -> Result<String> {
    // `SVPP-EXPANSION.5`(g): a MISSING actual (the list is shorter than the formals) for a formal
    // WITHOUT a default is an error — IEEE 1800-2017 §22.5.1's own example:
    // `` `MACRO1 ( 1 ) // ILLEGAL: b and c omitted, no default for c ``. An EMPTY actual is a
    // different thing (below, (f)): present and blank, it takes the default or "replaced by nothing".
    // Before this the formal bound to "" and the parser rejected the text the standard calls illegal
    // — the right verdict at the wrong stage (sv-tests 22.5.1--define-expansion_7 / _8).
    // and the mirror image: MORE actuals than formals — the standard's `` `D(,,) // illegal, more
    // actual than formal arguments `` (sv-tests 22.5.1--define-expansion_8 pins it).
    // `SVPP-EXPANSION.5`(j′): `` `thru() `` is ONE EMPTY actual when the macro declares formals —
    // the argument list `()` holds one (blank) argument, exactly as `(,)` holds two — and the empty
    // list of a formal-less macro (`` `define foo8() `` … `` `foo8() ``). The range parser returns no
    // range for `()`, so the two were both read as ZERO actuals and `` `thru() `` refused as MISSING
    // (`.5`(g)) where the clause's own `` `MACRO3 ( ) // all arguments replaced by defaults `` and
    // both reference tools read an empty actual (`$display(": \"\"")` for `` `msg(`thru(),) ``).
    // Found by `.5`(j): rendering a `` `" `` region expands the usages inside it, and verilator
    // `t_pp_display.v:35` carries `` `thru() `` inside one.
    let empty_actual: Vec<String>;
    let empty_mask: Vec<Vec<bool>>;
    let (args, arg_masks): (&[String], &[Vec<bool>]) = if args.is_empty() && !params.is_empty() {
        empty_actual = vec![String::new()];
        empty_mask = vec![Vec::new()];
        (&empty_actual, &empty_mask)
    } else {
        (args, arg_masks)
    };
    if args.len() > params.len() {
        bail!(
            "E_SVPP_EXTRA_MACRO_ARGUMENT at {}:{}: `{} invoked with {} arguments for {} formals — IEEE \
             1800-2017 §22.5.1: it shall be an error to specify more actual arguments than the number of \
             formal arguments",
            file.display(),
            line,
            name,
            args.len(),
            params.len()
        );
    }
    if args.len() < params.len() {
        for param in &params[args.len()..] {
            if param.default.is_none() {
                bail!(
                    "E_SVPP_MISSING_MACRO_ARGUMENT at {}:{}: `{} invoked with {} of {} arguments and formal \
                     '{}' has no default — IEEE 1800-2017 §22.5.1: \"ILLEGAL: … omitted, no default\"",
                    file.display(),
                    line,
                    name,
                    args.len(),
                    params.len(),
                    param.name
                );
            }
        }
    }
    let mut bindings: HashMap<&str, &str> = HashMap::new();
    // (h) each binding's provenance: an actual keeps the flags of the bytes it was written in; a
    // default is the caller's text too ("an actual argument to a macro or a default may contain a
    // usage of itself" — the depth guard bounds a default that names its own macro without end)
    let mut binding_masks: HashMap<&str, Vec<bool>> = HashMap::new();
    for (idx, param) in params.iter().enumerate() {
        // `SVPP-EXPANSION.5`(f): an EMPTY actual is an OMITTED one and takes the formal's default —
        // IEEE 1800-2017 §22.5.1's own examples: `` `MACRO1 ( , 2, 3 ) // argument a omitted,
        // replaced by default ``. Before this, an empty actual bound the formal to nothing, so
        // `` `MK(t1, 0, , sqr) `` lost `seq_t_`'s default. A formal with NO default binds to "".
        let actual = args.get(idx).map(|s| s.as_str());
        let (arg, mask) = match actual {
            Some(text) if !text.trim().is_empty() => (
                text,
                arg_masks
                    .get(idx)
                    .cloned()
                    .unwrap_or_else(|| vec![true; text.len()]),
            ),
            _ => {
                let default = param.default.as_deref().unwrap_or("");
                (default, vec![true; default.len()])
            }
        };
        bindings.insert(param.name.as_str(), arg);
        binding_masks.insert(param.name.as_str(), mask);
    }
    let (substituted, mask) =
        substitute_function_macro_body(&definition.body, &bindings, &binding_masks, plain_strings);
    expand_macros_in_text(
        &substituted,
        macros,
        depth,
        &mut Vec::new(),
        file,
        line,
        expanding,
        Some(&mask),
        plain_strings,
    )
}

fn expand_builtin_macro(name: &str, file: &Path, line: usize) -> Option<String> {
    match name {
        "__FILE__" => Some(encode_sv_string_literal(&file.display().to_string())),
        "__LINE__" => Some(line.to_string()),
        _ => None,
    }
}

fn encode_sv_string_literal(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{}\"", escaped)
}

/// Substitute the formals with their actuals and remove the ``` `` ``` paste tokens — returning,
/// with the text, its PROVENANCE mask (one flag per byte): `true` on the bytes an actual or a
/// default supplied (each keeping the flags it was written with), `false` on the bytes the body
/// produced — so the rescan can tell a usage the caller wrote (legal, §22.5.1) from one the macro's
/// own body contains (recursion, `.5`(h)).
fn substitute_function_macro_body(
    body: &str,
    bindings: &HashMap<&str, &str>,
    binding_masks: &HashMap<&str, Vec<bool>>,
    plain_strings: PlainStringSubstitutionPolicy,
) -> (String, Vec<bool>) {
    let bytes = body.as_bytes();
    let mut i = 0usize;
    let mut out = String::new();
    let mut mask: Vec<bool> = Vec::new();
    fn push_body(out: &mut String, mask: &mut Vec<bool>, text: &str) {
        out.push_str(text);
        mask.extend(std::iter::repeat_n(false, text.len()));
    }
    fn push_body_char(out: &mut String, mask: &mut Vec<bool>, ch: char) {
        out.push(ch);
        mask.extend(std::iter::repeat_n(false, ch.len_utf8()));
    }
    // `SVPP-EXPANSION.5`(i): a paste token is a TOKEN of the expansion text — the rescan
    // (`expand_macros_in_text`) is the one place that knows what its operands are, so it is kept
    // here and consumed there. Inside a string literal — a `` `" `` region, or a plain `"…"` in the
    // body — the rescan never looks, so a paste is removed DURING the substitution as before
    // (`strip_paste`), the mask kept aligned byte for byte.
    fn push_actual(
        out: &mut String,
        mask: &mut Vec<bool>,
        actual: &str,
        actual_mask: &[bool],
        strip_paste: bool,
    ) {
        let lead = actual.len() - actual.trim_start().len();
        let trimmed = actual.trim().as_bytes();
        let mut buf: Vec<u8> = Vec::with_capacity(trimmed.len());
        let mut k = 0usize;
        while k < trimmed.len() {
            if strip_paste && k + 1 < trimmed.len() && trimmed[k] == b'`' && trimmed[k + 1] == b'`'
            {
                k += 2;
                continue;
            }
            buf.push(trimmed[k]);
            mask.push(actual_mask.get(lead + k).copied().unwrap_or(true));
            k += 1;
        }
        out.push_str(
            std::str::from_utf8(&buf).expect("removing ASCII paste tokens keeps UTF-8 valid"),
        );
    }
    // a plain `"…"` string literal in the body: the paste tokens inside it are stripped here (the
    // rescan does not enter strings); what ELSE happens inside it — a formal is substituted there
    // today, which IEEE 1800-2017 §22.5.1's "the use of \" always starts a string literal" reads
    // against — is `SVPP-EXPANSION.5`(k)'s question and is deliberately left as it was
    let mut in_plain_string = false;
    let mut escaped_in_plain_string = false;
    let binding_mask =
        |token: &str| -> &[bool] { binding_masks.get(token).map(Vec::as_slice).unwrap_or(&[]) };

    while i < bytes.len() {
        if i + 2 <= bytes.len() && bytes[i] == b'`' && bytes[i + 1] == b'`' {
            // the ``` `` ``` token-paste operator: the two sides join with nothing between — kept
            // for the rescan outside a string literal (`.5`(i)), removed inside one
            // `.5`(k): inside a plain string the paste is the string's under the standard's reading
            // (the literal is opaque) and removed under the two substituting readings
            if !in_plain_string || plain_strings == PlainStringSubstitutionPolicy::Standard {
                push_body(&mut out, &mut mask, "``");
            }
            i += 2;
            continue;
        }
        if i + 2 <= bytes.len() && bytes[i] == b'`' && bytes[i + 1] == b'"' {
            // SV-EXH-PROOF.3.3.4.b.6.2.PP.1 — per IEEE 1800 §22.5.3.2:
            // `" opens a "macro string with substitution" region; macro
            // arguments inside the region are substituted, and the whole
            // region (terminated by a matching `") becomes a SINGLE
            // string literal "...". Pre-fix the algorithm assumed the
            // region contained ONLY a macro-arg identifier with no other
            // literal text — so `"ARG[+]`" (UVM's `m_uvm_field_qda_int`
            // pattern) was mis-expanded: the `[+]` was emitted bare and
            // the closing `" became a literal backtick + ". Fix: scan
            // forward to the matching closing `", then substitute macro
            // arguments inside the content, emitting a SINGLE "..." pair.
            // Surfaced by uvm_pkg furthest reaching byte 3,006,234 in
            // `__local_field_names__.push_back('{"abstractions", "abstractions"[+]`")`
            // where the second arg should have been the string
            // "abstractions[+]" but came out as `"abstractions"[+]`"`.
            let region_start = i + 2;
            // `SVPP-EXPANSION.5`(j): the region is no longer RENDERED here — the actuals are
            // substituted into it and it is emitted with its `` `" `` markers on, for the rescan's
            // renderer (`render_macro_string_region`), which is the one place that can also expand
            // an EMBEDDED macro usage (the clause's "expansions of embedded macros") and which the
            // object-macro path reaches too. The closer is found by `find_macro_string_close`, which
            // steps over an escaped quote ``` `\`" ``` whole: the scan that used to live here
            // matched the escaped quote's last two bytes and ended the standard's own `msg`
            // example at its first `\"`.
            let found = find_macro_string_close(body, region_start);
            if let Some(j) = found {
                let content = &body[region_start..j];
                push_body(&mut out, &mut mask, "`\"");
                let content_bytes = content.as_bytes();
                let mut k = 0usize;
                while k < content_bytes.len() {
                    // an escaped quote and a paste are the renderer's to read: copied as written
                    if content[k..].starts_with("`\\`\"") {
                        push_body(&mut out, &mut mask, "`\\`\"");
                        k += 4;
                        continue;
                    }
                    if k + 2 <= content_bytes.len()
                        && content_bytes[k] == b'`'
                        && content_bytes[k + 1] == b'`'
                    {
                        push_body(&mut out, &mut mask, "``");
                        k += 2;
                        continue;
                    }
                    if is_ident_start(content_bytes[k]) {
                        let tok_start = k;
                        k += 1;
                        while k < content_bytes.len() && is_ident_continue(content_bytes[k]) {
                            k += 1;
                        }
                        let token = &content[tok_start..k];
                        let system_name = tok_start > 0 && content_bytes[tok_start - 1] == b'$';
                        if let Some(arg) = bindings.get(token).filter(|_| !system_name) {
                            push_actual(&mut out, &mut mask, arg, binding_mask(token), false);
                        } else {
                            push_body(&mut out, &mut mask, token);
                        }
                    } else {
                        let ch = source_char_at(content, k);
                        push_body_char(&mut out, &mut mask, ch);
                        k += ch.len_utf8();
                    }
                }
                push_body(&mut out, &mut mask, "`\"");
                i = j + 2;
                continue;
            }
            // No matching closing `" — fall back to the legacy single-ident
            // form (`"<ident> with no close, often seen in informal macros
            // that rely on macro-body-end as an implicit close). This
            // preserves backwards-compat with the original algorithm.
            let mut k = region_start;
            if k < bytes.len() && is_ident_start(bytes[k]) {
                k += 1;
                while k < bytes.len() && is_ident_continue(bytes[k]) {
                    k += 1;
                }
                let token = &body[region_start..k];
                if let Some(arg) = bindings.get(token) {
                    push_body(&mut out, &mut mask, "\"");
                    push_actual(&mut out, &mut mask, arg, binding_mask(token), true);
                    push_body(&mut out, &mut mask, "\"");
                    i = k;
                    continue;
                }
            }
            // Truly unmatched — emit literally.
            push_body(&mut out, &mut mask, "`\"");
            i += 2;
            continue;
        }

        if !is_ident_start(bytes[i]) {
            let ch = source_char_at(body, i);
            if in_plain_string {
                if escaped_in_plain_string {
                    escaped_in_plain_string = false;
                } else if ch == '\\' {
                    escaped_in_plain_string = true;
                } else if ch == '"' {
                    in_plain_string = false;
                }
            } else if ch == '"' {
                in_plain_string = true;
            }
            push_body_char(&mut out, &mut mask, ch);
            i += ch.len_utf8();
            continue;
        }
        // `.5`(k): an escape inside a plain string escapes ONE character, and that character may
        // start an identifier (`\n`, `\t`, `\"`) — it is consumed here, so the quote that follows
        // it closes the string. Before this, `"%%Error: … got=%0x\n"` never closed and every formal
        // after it on the line (Verilator's `` `checkh(gotv, expv) ``) read as inside the string.
        if in_plain_string && escaped_in_plain_string {
            escaped_in_plain_string = false;
        }
        let start = i;
        i += 1;
        while i < bytes.len() && is_ident_continue(bytes[i]) {
            i += 1;
        }
        let token = &body[start..i];
        // `SVPP-EXPANSION.6b.5`(a): `$display` is ONE token (IEEE 1800-2017 A.9.3 system_tf_identifier
        // ::= $[a-zA-Z0-9_$]{…}), so a formal named `display` is not inside it — sv2v `macro.sv`'s
        // `` `define MACRO_D(display) $display(display); `` expanded to `$"display"("display")`.
        let system_name = start > 0 && bytes[start - 1] == b'$';
        // `SVPP-EXPANSION.5`(k): inside a plain string a formal is substituted only as the policy
        // reads it — never (the standard), only when paste tokens delimit it on both sides
        // (`` "[``i``]" ``, OpenTitan's idiom), or always (Icarus's reading)
        let substitutable = match plain_strings {
            _ if !in_plain_string => true,
            PlainStringSubstitutionPolicy::Standard => false,
            PlainStringSubstitutionPolicy::PasteDelimited => {
                start >= 2 && &body[start - 2..start] == "``" && body[i..].starts_with("``")
            }
            PlainStringSubstitutionPolicy::All => true,
        };
        if let Some(arg) = bindings
            .get(token)
            .filter(|_| !system_name && substitutable)
        {
            push_actual(
                &mut out,
                &mut mask,
                arg,
                binding_mask(token),
                in_plain_string,
            );
        } else {
            push_body(&mut out, &mut mask, token);
        }
    }

    debug_assert_eq!(
        out.len(),
        mask.len(),
        "the provenance mask is one flag per byte"
    );
    (out, mask)
}

/// The character whose UTF-8 sequence starts at `index`.
///
/// ⭐ `SV-CORPUS-GRAD.12c.2`. The line scanners in this file walk their input as `as_bytes()`, and
/// that is the right design — every delimiter they look for (`` ` ``, `"`, `/`, `\n`, the
/// identifier alphabet) is ASCII, and the source map they feed is byte-indexed. What was wrong was
/// how they re-emitted: `out.push(bytes[i] as char)` is a **Latin-1 promotion**, not a UTF-8
/// decode, so byte `0xC2` became U+00C2 and re-encoded as two bytes. Every multi-byte character
/// came back out doubled — `©` as `Â©` — including in files that are perfectly valid UTF-8.
///
/// Emitting the whole sequence and advancing by its length keeps the output byte-identical to the
/// input *and* keeps the cursor on a character boundary.
///
/// ⛔ The boundary invariant this relies on, stated because a future edit could break it silently:
/// every cursor advance in these scanners is either this function's `len_utf8()`, or a `+1`/`+2`
/// over bytes the scanner has already compared against an ASCII literal, or a jump to an index
/// produced by [`is_ident_start`]/[`is_ident_continue`] — and both of those are **ASCII-only**, so
/// identifier scanning always stops before a continuation byte.
fn source_char_at(text: &str, index: usize) -> char {
    text[index..]
        .chars()
        .next()
        .expect("caller loops while index < text.len(), and index is on a char boundary")
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_ident_continue(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

#[cfg(test)]
mod tests {
    /// `.113`: the neutral event log carries the kind's serde name — every variant's `as_str`
    /// equals what serde writes for it, so the converted `--event-log-json` is byte-identical.
    #[test]
    fn every_event_kind_spells_itself_as_serde_does() {
        use super::PreprocessorEventKind as K;
        let all = [
            K::Define, K::Undef, K::Include, K::Ifdef, K::Ifndef, K::Elsif, K::Else, K::Endif,
            K::PassthroughLine, K::SkippedLine, K::MacroExpand, K::LineDirective,
            K::ProtectedEnvelope, K::KeywordsRegion,
        ];
        for kind in &all {
            let via_serde = serde_json::to_string(kind).expect("serializes");
            assert_eq!(via_serde, format!("{:?}", kind.as_str()), "{:?}", kind);
        }
        // exhaustiveness: a variant added without a row above fails to compile in `as_str`,
        // and one added without a row HERE is caught by the count.
        let variants_in_as_str = 14;
        assert_eq!(all.len(), variants_in_as_str);
    }

    use super::{
        ConditionalSymbolPolicy, IncludePathPolicy, MacroDefinition, MacroParameter,
        MacroRedefinitionPolicy, PlainStringSubstitutionPolicy, PredefinedMacro,
        PreprocessedOutput, PreprocessorDiagnosticSeverity, PreprocessorEventKind,
        SvPreprocessorConfig, UndefinedMacroPolicy, expand_function_macro, normalize_macro_body_for_directives,
        parse_strict_warning_codes, preprocess_systemverilog_file, preprocess_systemverilog_unit,
    };
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    /// `SVPP-EXPANSION.5`(e): every test's scratch directory lives under the crate's own `target/`,
    /// on the repository volume (the data-locality rule). This older helper used the OS temp dir;
    /// it now delegates so the 26 tests written against it move without being rewritten.
    fn create_temp_dir(prefix: &str) -> PathBuf {
        create_repo_scratch_dir(prefix)
    }

    #[test]
    fn expands_object_macro_define() {
        let dir = create_temp_dir("svpp_define");
        let input = dir.join("top.sv");
        fs::write(&input, "`define WIDTH 16\nlogic [`WIDTH-1:0] data;\n").expect("write input");

        let output = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("preprocess");
        assert!(output.text.contains("logic [16-1:0] data;"));
        assert!(!output.text.contains("`define"));
    }

    #[test]
    fn resolves_include_and_tracks_source_map() {
        let dir = create_temp_dir("svpp_include");
        let inc = dir.join("inc.svh");
        let top = dir.join("top.sv");
        fs::write(&inc, "logic from_inc;\n").expect("write inc");
        fs::write(&top, "`include \"inc.svh\"\nlogic from_top;\n").expect("write top");

        let output = preprocess_systemverilog_file(&top, &SvPreprocessorConfig::default())
            .expect("preprocess include");
        assert!(output.text.contains("logic from_inc;"));
        assert!(output.text.contains("logic from_top;"));
        assert!(output.included_files.iter().any(|p| p.ends_with("inc.svh")));
        assert!(
            output
                .source_map
                .iter()
                .any(|m| m.source.file.ends_with("inc.svh"))
        );
    }

    #[test]
    fn honors_ifdef_else_endif() {
        let dir = create_temp_dir("svpp_ifdef");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define FLAG 1\n`ifdef FLAG\nlogic yes;\n`else\nlogic no;\n`endif\n",
        )
        .expect("write top");

        let output = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("preprocess ifdef");
        assert!(output.text.contains("logic yes;"));
        assert!(!output.text.contains("logic no;"));
    }

    #[test]
    fn expands_function_macro_with_token_paste_and_stringize() {
        let dir = create_temp_dir("svpp_func");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define CAT(a,b) a``b\n`define STR(x) `\"x\nwire `CAT(sig,_id);\nstring s = `STR(hello);\n",
        )
        .expect("write top");

        let output = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("preprocess function-like macros");
        assert!(output.text.contains("wire sig_id;"));
        assert!(output.text.contains("string s = \"hello\";"));
    }

    #[test]
    fn expands_function_macro_with_default_parameter_expression() {
        let dir = create_temp_dir("svpp_func_default");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define REPORT_BEGIN(SEVERITY, ID, VERBOSITY, RO=uvm_get_report_object()) RO\nlogic [0:0] a = `REPORT_BEGIN(UVM_INFO, ID, 100);\n",
        )
        .expect("write top");

        let output = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("preprocess function-like macro with default");
        assert!(
            output
                .text
                .contains("logic [0:0] a = uvm_get_report_object();")
        );
    }

    #[test]
    fn expands_function_macro_with_empty_default_parameter() {
        let dir = create_temp_dir("svpp_func_empty_default");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define DECL(TYPE, NAME, SFX=) TYPE NAME``SFX;\n`DECL(logic, signal)\n",
        )
        .expect("write top");

        let output = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("preprocess function-like macro with empty default");
        assert!(output.text.contains("logic signal;"));
    }

    #[test]
    fn expands_builtin_file_and_line_macros_at_invocation_site() {
        let dir = create_temp_dir("svpp_builtin_file_line");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define UVM_FILE `__FILE__\n`define UVM_LINE `__LINE__\nstring f = `UVM_FILE;\nint l = `UVM_LINE;\n",
        )
        .expect("write top");

        let output = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("preprocess builtin file/line macros");
        let canonical_input = fs::canonicalize(&input).expect("canonicalize input");
        assert!(
            output
                .text
                .contains(&format!("string f = \"{}\";", canonical_input.display())),
            "unexpected preprocessor output: {}",
            output.text
        );
        assert!(
            output.text.contains("int l = 4;"),
            "unexpected preprocessor output: {}",
            output.text
        );
    }

    #[test]
    fn expands_multiline_function_macro_invocation() {
        let dir = create_temp_dir("svpp_multiline_call");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define CHECKER(condition, msg) if (condition) begin $display(\"%s\", msg); end\ninitial begin\n  `CHECKER((A!=0),\n    \"bad\");\nend\n",
        )
        .expect("write top");

        let output = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("preprocess multiline macro invocation");
        assert!(
            output
                .text
                .contains("if ((A!=0)) begin $display(\"%s\", \"bad\"); end"),
            "unexpected preprocessor output: {}",
            output.text
        );
        assert!(
            !output.text.contains("`CHECKER"),
            "unexpected preprocessor output: {}",
            output.text
        );
    }

    #[test]
    fn expands_nested_macro_with_stringized_parameter() {
        let dir = create_temp_dir("svpp_nested_stringized_param");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define TYPE_NAME_DECL(TNAME_STRING) function string type_name(); return TNAME_STRING; endfunction\n`define OBJECT_UTILS(T) `TYPE_NAME_DECL(`\"T`\")\npackage p;\nclass c;\n  `OBJECT_UTILS(c)\nendclass\nendpackage\n",
        )
        .expect("write top");

        let output = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("preprocess nested stringized macro");
        assert!(
            output
                .text
                .contains("function string type_name(); return \"c\"; endfunction"),
            "unexpected preprocessor output: {}",
            output.text
        );
        assert!(
            !output.text.contains("`TYPE_NAME_DECL"),
            "unexpected preprocessor output: {}",
            output.text
        );
        assert!(
            !output.text.contains("`\""),
            "unexpected preprocessor output: {}",
            output.text
        );
    }

    #[test]
    fn keeps_parenthesized_object_macro_body() {
        let dir = create_temp_dir("svpp_object_paren_body");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define MASK          (1<<0)\nlocalparam int M = `MASK;\n",
        )
        .expect("write top");

        let output = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("preprocess object-like macro with parenthesized body");
        assert!(output.text.contains("localparam int M = (1<<0);"));
    }

    #[test]
    fn accepts_multiline_define_with_directive_tokens_in_body() {
        let dir = create_temp_dir("svpp_multiline_define");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define MAYBE_FIELD \\\n`ifdef ENABLE_FIELD \\\nlogic enabled; \\\n`endif\n`MAYBE_FIELD\n",
        )
        .expect("write top");

        let output = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("preprocess multiline define");
        assert!(!output.text.contains("`ifdef"));
        assert!(!output.text.contains("`endif"));
        assert!(!output.text.contains("logic enabled;"));
        assert!(
            output
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.severity != PreprocessorDiagnosticSeverity::Error)
        );
    }

    #[test]
    fn expands_multiline_define_with_directive_tokens_when_enabled() {
        let dir = create_temp_dir("svpp_multiline_define_enabled");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define ENABLE_FIELD 1\n`define MAYBE_FIELD \\\n`ifdef ENABLE_FIELD \\\nlogic enabled; \\\n`else \\\nlogic disabled; \\\n`endif\nmodule m;\n  `MAYBE_FIELD\nendmodule\n",
        )
        .expect("write top");

        let output = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("preprocess multiline define with enabled branch");
        assert!(output.text.contains("logic enabled;"));
        assert!(!output.text.contains("logic disabled;"));
        assert!(!output.text.contains("`ifdef"));
        assert!(!output.text.contains("`else"));
        assert!(!output.text.contains("`endif"));
    }

    #[test]
    fn expands_inline_ifndef_endif_payload_in_macro_body() {
        let dir = create_temp_dir("svpp_inline_ifndef_endif");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define MAYBE_FIELD \\\n`ifndef DISABLE_FIELD logic enabled; `endif\nmodule m;\n  `MAYBE_FIELD\nendmodule\n",
        )
        .expect("write top");

        let output = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("preprocess inline ifndef/endif payload");
        assert!(output.text.contains("logic enabled;"));
        assert!(!output.text.contains("`ifndef"));
        assert!(!output.text.contains("`endif"));
    }

    #[test]
    fn does_not_expand_macros_inside_comments() {
        let dir = create_temp_dir("svpp_comment_inert_macros");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define DECL(NAME) logic NAME;\nmodule m;\n// `DECL(from_line_comment)\n/* `DECL(from_block_comment) */\n  `DECL(from_code)\nendmodule\n",
        )
        .expect("write top");

        let output = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("preprocess comment-contained macro text");
        assert!(output.text.contains("logic from_code;"));
        assert!(!output.text.contains("logic from_line_comment;"));
        assert!(!output.text.contains("logic from_block_comment;"));
        assert!(output.text.contains("// `DECL(from_line_comment)"));
        assert!(output.text.contains("/* `DECL(from_block_comment) */"));
    }

    #[test]
    fn ignores_unclosed_macro_examples_inside_comments_when_collecting_lines() {
        let dir = create_temp_dir("svpp_comment_multiline_collect");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define DECL(ID, MSG) logic seen;\nmodule m;\n//| `DECL(\"ID\",\n`ifndef DISABLE_FIELD\nlogic enabled;\n`endif\nendmodule\n",
        )
        .expect("write top");

        let output = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("preprocess with comment-contained unclosed macro example");
        assert!(output.text.contains("logic enabled;"));
        assert!(output.text.contains("//| `DECL(\"ID\","));
    }

    #[test]
    fn preserves_brace_concat_function_macro_argument() {
        let dir = create_temp_dir("svpp_brace_concat_arg");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define WARN(ID, MSG) uvm_report_warning(ID, MSG, UVM_NONE, `__FILE__, `__LINE__, \"\", 1)\nmodule m;\n  initial begin\n    `WARN(\"find_type-no match\", {\"Instance of type '\", TYPE::type_name, \" not found in component hierarchy beginning at \", start.get_full_name()})\n  end\nendmodule\n",
        )
        .expect("write top");

        let output = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("preprocess brace-concat macro arg");
        assert!(output.text.contains("uvm_report_warning(\"find_type-no match\", {\"Instance of type '\", TYPE::type_name, \" not found in component hierarchy beginning at \", start.get_full_name()}, UVM_NONE,"));
    }

    #[test]
    fn expands_nested_uvm_style_field_macros_without_leaking_conditionals() {
        let dir = create_temp_dir("svpp_nested_uvm_field_macros");
        let input = dir.join("top.sv");
        let warn_body = normalize_macro_body_for_directives(
            "begin \n    string behavior; \n    `ifdef UVM_LEGACY_FIELD_MACRO_SEMANTICS behavior = \"legacy\"; \n    `else behavior = \"modern\"; \n    `endif \n  end",
        );
        let field_begin_body = normalize_macro_body_for_directives(
            "`WARN(ARG,FLAG) \n  begin \n    if ( \n       `ifndef UVM_LEGACY_FIELD_MACRO_SEMANTICS (((FLAG)&UVM_SET)) && `endif \n       (!((FLAG)&UVM_NOSET)) \n       ) begin \n      logic enabled; \n    end \n  end",
        );
        let mut macros = HashMap::new();
        macros.insert(
            "WARN".to_string(),
            MacroDefinition {
                params: Some(vec![
                    MacroParameter {
                        name: "ARG".to_string(),
                        default: None,
                    },
                    MacroParameter {
                        name: "FLAG".to_string(),
                        default: None,
                    },
                ]),
                body: warn_body,
            },
        );
        let field_begin = MacroDefinition {
            params: Some(vec![
                MacroParameter {
                    name: "ARG".to_string(),
                    default: None,
                },
                MacroParameter {
                    name: "FLAG".to_string(),
                    default: None,
                },
            ]),
            body: field_begin_body,
        };
        let expanded = expand_function_macro(
            &field_begin,
            field_begin.params.as_ref().expect("params"),
            &["ID".to_string(), "UVM_ALL_ON".to_string()],
            &[],
            &macros,
            0,
            &input,
            1,
            "uvm_field_begin",
            &mut Vec::new(),
            PlainStringSubstitutionPolicy::Standard,
        )
        .expect("expand");
        assert!(
            expanded.contains("string behavior;"),
            "expanded nested macro body missing expected content:\n{expanded}"
        );
        assert!(
            expanded.contains("`ifndef UVM_LEGACY_FIELD_MACRO_SEMANTICS"),
            "expanded nested macro body missing ifndef split:\n{expanded}"
        );
        assert!(
            expanded.contains("\n       `endif"),
            "expanded nested macro body missing endif split:\n{expanded}"
        );
        assert!(
            !expanded.contains(
                "`ifndef UVM_LEGACY_FIELD_MACRO_SEMANTICS (((UVM_ALL_ON)&UVM_SET)) && `endif"
            ),
            "expanded nested macro body still contains unsplit inline conditional:\n{expanded}"
        );

        fs::write(
            &input,
            "`define WARN(ARG,FLAG) \\\n  begin \\\n    string behavior; \\\n    `ifdef UVM_LEGACY_FIELD_MACRO_SEMANTICS behavior = \"legacy\"; \\\n    `else behavior = \"modern\"; \\\n    `endif \\\n  end\n\
`define FIELD_BEGIN(ARG, FLAG) \\\n  `WARN(ARG,FLAG) \\\n  begin \\\n    if ( \\\n       `ifndef UVM_LEGACY_FIELD_MACRO_SEMANTICS (((FLAG)&UVM_SET)) && `endif \\\n       (!((FLAG)&UVM_NOSET)) \\\n       ) begin \\\n      logic enabled; \\\n    end \\\n  end\n\
module m;\n  `FIELD_BEGIN(ID,UVM_ALL_ON)\nendmodule\n",
        )
        .expect("write top");

        let output = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("preprocess nested UVM-style field macros");
        assert!(output.text.contains("string behavior;"));
        assert!(output.text.contains("behavior = \"modern\";"));
        assert!(output.text.contains("logic enabled;"));
        assert!(!output.text.contains("`ifdef"));
        assert!(!output.text.contains("`else"));
        assert!(!output.text.contains("`ifndef"));
        assert!(!output.text.contains("`endif"));
    }

    #[test]
    fn warns_on_macro_redefine_when_policy_warn() {
        let dir = create_temp_dir("svpp_redefine_warn");
        let input = dir.join("top.sv");
        fs::write(&input, "`define A 1\n`define A 2\nlogic [`A-1:0] data;\n").expect("write top");

        let config = SvPreprocessorConfig {
            macro_redefinition_policy: MacroRedefinitionPolicy::Warn,
            ..SvPreprocessorConfig::default()
        };
        let output =
            preprocess_systemverilog_file(&input, &config).expect("warn policy should not fail");
        assert!(output.text.contains("logic [2-1:0] data;"));
        assert!(output.diagnostics.iter().any(|d| {
            d.code == "W_SVPP_MACRO_REDEFINED"
                && d.severity == PreprocessorDiagnosticSeverity::Warning
        }));
    }

    #[test]
    fn errors_on_macro_redefine_when_policy_error() {
        let dir = create_temp_dir("svpp_redefine_err");
        let input = dir.join("top.sv");
        fs::write(&input, "`define A 1\n`define A 2\n").expect("write top");

        let config = SvPreprocessorConfig {
            macro_redefinition_policy: MacroRedefinitionPolicy::Error,
            ..SvPreprocessorConfig::default()
        };
        let err = preprocess_systemverilog_file(&input, &config)
            .expect_err("error policy should fail on redefinition");
        let err_text = format!("{err:#}");
        assert!(err_text.contains("E_SVPP_MACRO_REDEFINED"));
    }

    #[test]
    fn ifndef_missing_symbol_does_not_warn_when_ifdef_policy_warns() {
        let dir = create_temp_dir("svpp_ifndef_policy");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`ifndef MISSING\nlogic via_ifndef;\n`endif\n`ifdef MISSING\nlogic via_ifdef;\n`endif\n",
        )
        .expect("write top");

        let config = SvPreprocessorConfig {
            conditional_symbol_policy: ConditionalSymbolPolicy::AssumeFalseWarn,
            ..SvPreprocessorConfig::default()
        };
        let output =
            preprocess_systemverilog_file(&input, &config).expect("policy warn should still pass");
        assert!(output.text.contains("logic via_ifndef;"));
        assert!(!output.text.contains("logic via_ifdef;"));
        let undefined_warnings: Vec<_> = output
            .diagnostics
            .iter()
            .filter(|d| d.code == "W_SVPP_UNDEFINED_CONDITIONAL_SYMBOL")
            .collect();
        assert_eq!(
            undefined_warnings.len(),
            1,
            "expected warning only for `ifdef"
        );
    }

    #[test]
    fn strict_warning_promotion_turns_warning_into_error() {
        let dir = create_temp_dir("svpp_strict_warn");
        let inc = dir.join("inc.svh");
        let top = dir.join("top.sv");
        fs::write(&inc, "logic from_inc;\n").expect("write inc");
        fs::write(&top, format!("`include \"{}\"\n", inc.display())).expect("write top");

        let config = SvPreprocessorConfig {
            include_path_policy: IncludePathPolicy::RelativeOnly,
            strict_warning_codes: parse_strict_warning_codes("W_SVPP_ABSOLUTE_INCLUDE_PATH"),
            ..SvPreprocessorConfig::default()
        };
        let err = preprocess_systemverilog_file(&top, &config)
            .expect_err("strict warning promotion should fail");
        let err_text = format!("{err:#}");
        assert!(err_text.contains("W_SVPP_ABSOLUTE_INCLUDE_PATH"));
    }

    #[test]
    fn tolerates_non_utf8_include_with_warning() {
        let dir = create_temp_dir("svpp_non_utf8");
        let inc = dir.join("inc.svh");
        let top = dir.join("top.sv");
        let mut inc_bytes = b"// Copyright \xa9\nlogic from_inc;\n".to_vec();
        fs::write(&inc, std::mem::take(&mut inc_bytes)).expect("write inc bytes");
        fs::write(&top, "`include \"inc.svh\"\nlogic from_top;\n").expect("write top");

        let output = preprocess_systemverilog_file(&top, &SvPreprocessorConfig::default())
            .expect("preprocess non-utf8 include");
        assert!(output.text.contains("logic from_inc;"));
        assert!(output.text.contains("logic from_top;"));
        // ⭐ SV-CORPUS-GRAD.12c.1 — NO INFORMATION IS LOST AT THE READER ANY MORE. Until this
        // leaf the preprocessor substituted U+FFFD for the `0xA9`, which is unrecoverable; the
        // warning is kept because the encoding is still worth reporting, but the loss is gone.
        assert!(!output.text.contains('\u{FFFD}'), "{}", output.text);
        // ⭐ END TO END, BOTH HALVES. `.12c.1` made the READER lossless — the `0xA9` decodes to
        // U+00A9 instead of U+FFFD — and `.12c.2` made the EXPANDER lossless, so the character
        // now survives all the way to the preprocessed text. This assertion was pinned to the
        // then-current defect (`Â©`) for exactly one commit, so that the fix could not land
        // silently; it now asserts the correct output.
        assert!(output.text.contains("// Copyright ©"), "{}", output.text);
        let diagnostic = output
            .diagnostics
            .iter()
            .find(|d| d.code == "W_SVPP_NON_UTF8_SOURCE")
            .expect("the non-UTF-8 encoding is still reported");
        assert_eq!(diagnostic.severity, PreprocessorDiagnosticSeverity::Warning);
        assert!(
            diagnostic.message.contains("iso-8859-1"),
            "the warning must name the encoding it decoded from: {}",
            diagnostic.message
        );
    }

    #[test]
    fn a_plain_utf8_source_raises_no_encoding_warning() {
        // The other direction: the warning must stay rare. A UTF-8 file with multi-byte
        // characters is not an encoding event and must not be reported as one.
        let dir = create_temp_dir("svpp_utf8");
        let top = dir.join("top.sv");
        fs::write(&top, "// Copyright © 2016 — µ\nlogic from_top;\n").expect("write top");

        let output = preprocess_systemverilog_file(&top, &SvPreprocessorConfig::default())
            .expect("preprocess utf-8 source");
        assert!(output.text.contains("logic from_top;"));
        assert!(
            !output
                .diagnostics
                .iter()
                .any(|d| d.code == "W_SVPP_NON_UTF8_SOURCE"),
            "plain UTF-8 must not raise the encoding warning"
        );
    }

    #[test]
    fn a_plain_utf8_source_passes_through_byte_identical() {
        // ⭐ `SV-CORPUS-GRAD.12c.2` — the inverse of the test this replaced.
        //
        // Until `.12c.2`, `expand_macros_in_text` and its siblings re-emitted with
        // `out.push(bytes[i] as char)` — a Latin-1 promotion, not a UTF-8 decode — so a two-byte
        // sequence came back out as two characters and four bytes. This same input measured
        // 44 B in, **51 B out**: exactly one extra byte per non-ASCII byte. That was MOJIBAKE for
        // every non-ASCII input, and it had nothing to do with the file being Latin-1 — this
        // input is valid UTF-8.
        //
        // ⛔ The byte-LENGTH assertion is the load-bearing one, not the `contains`: a scanner that
        // re-encodes wrongly can still contain the right substring somewhere. Length equality over
        // a line carrying a 2-, a 3- and a 2-byte character catches any per-byte promotion.
        let dir = create_temp_dir("svpp_utf8_passthrough");
        let top = dir.join("top.sv");
        let source = "// Copyright © 2016 — µ\nlogic from_top;\n";
        fs::write(&top, source).expect("write top");

        let output = preprocess_systemverilog_file(&top, &SvPreprocessorConfig::default())
            .expect("preprocess utf-8 source");
        assert_eq!(
            output.text.len(),
            source.len(),
            "the preprocessed text must be byte-identical in length; was source.len()+7 before \
             .12c.2 (© = 2 B, — = 3 B, µ = 2 B)"
        );
        assert_eq!(output.text, source);
    }

    #[test]
    fn non_ascii_survives_in_strings_and_block_comments_and_macro_bodies() {
        // The other three emitting scanners `.12c.2` fixed, each on the path that reaches it:
        // the double-quote branch, the block-comment branch, and the function-macro body
        // substitution. A fix applied only to the line-comment branch — the one the corpus
        // exercises — would pass the test above and still corrupt these.
        let dir = create_temp_dir("svpp_utf8_branches");
        let top = dir.join("top.sv");
        let source = concat!(
            "`define GREET(x) $display(\"hé: %s — µ\", x)\n",
            "/* blöck çomment ©\n   still in it ± */\n",
            "initial `GREET(\"wörld\");\n",
            "localparam string S = \"ünïcode\";\n"
        );
        fs::write(&top, source).expect("write top");

        let output = preprocess_systemverilog_file(&top, &SvPreprocessorConfig::default())
            .expect("preprocess utf-8 source");
        for expected in [
            "blöck çomment ©",
            "still in it ±",
            "hé: %s — µ",
            "\"wörld\"",
            "\"ünïcode\"",
        ] {
            assert!(
                output.text.contains(expected),
                "lost {expected:?} in:\n{}",
                output.text
            );
        }
        assert!(!output.text.contains('\u{FFFD}'), "{}", output.text);
        assert!(
            !output.text.contains("Ã"),
            "double-encoded: {}",
            output.text
        );
    }

    #[test]
    fn the_source_map_byte_ranges_index_the_output_after_a_non_ascii_line() {
        // ⛔ The consequence that outlived the visible mojibake. `SourceMapEntry` carries
        // `output_start`/`output_end` byte offsets into `output.text`; while the expander inflated
        // every non-ASCII character, every range after such a character named the wrong bytes —
        // silently, because nothing ever sliced the text with them.
        let dir = create_temp_dir("svpp_utf8_source_map");
        let top = dir.join("top.sv");
        fs::write(&top, "// © — µ\nlogic a;\nlogic b;\n").expect("write top");

        let output = preprocess_systemverilog_file(&top, &SvPreprocessorConfig::default())
            .expect("preprocess utf-8 source");
        assert!(!output.source_map.is_empty());
        for entry in &output.source_map {
            let slice = output
                .text
                .get(entry.output_start..entry.output_end)
                .unwrap_or_else(|| {
                    panic!(
                        "source-map range {}..{} is not a valid slice of {} bytes of output",
                        entry.output_start,
                        entry.output_end,
                        output.text.len()
                    )
                });
            assert!(!slice.is_empty());
        }
        let last = output.source_map.last().expect("at least one entry");
        assert_eq!(
            last.output_end,
            output.text.len(),
            "the map must cover the output exactly"
        );
    }

    // ── SVPP-EXPANSION.2 — the compilation context: predefines and the compilation unit ──────
    //
    // ⛔ These scratch directories live under the crate's own `target/`, on the repository volume
    // (`docs/decisions/` data-locality rule). `create_temp_dir` above delegates here since `.5`(e).
    fn create_repo_scratch_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("svpp_unit_tests")
            .join(format!("{}_{}", prefix, nanos));
        fs::create_dir_all(&dir).expect("mkdir repo scratch");
        dir
    }

    // ── SVPP-EXPANSION.8 — IEEE 1800-2017 §22.14 `begin_keywords regions ──────────────────────

    fn preprocess_with_profile(source: &str, profile: Option<&str>) -> PreprocessedOutput {
        // ⛔ A COUNTER, not just the clock: a dozen tests share this helper and `cargo test` runs
        // them in parallel, so two calls inside one clock tick would share a scratch directory and
        // overwrite each other's `top.sv`. Observed once before this line existed.
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let seq = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = create_repo_scratch_dir(&format!("svpp_keywords_{seq}"));
        let input = dir.join("top.sv");
        fs::write(&input, source).expect("write input");
        let config = SvPreprocessorConfig {
            parse_profile: profile.map(str::to_string),
            ..SvPreprocessorConfig::default()
        };
        preprocess_systemverilog_file(&input, &config).expect("preprocess")
    }

    /// §22.14.3's own example: `logic` is not a keyword in `"1364-2001"`, so a parser holding the
    /// 1800-2017 set must be handed it as an escaped identifier (§5.6.2).
    #[test]
    fn a_region_unreserves_the_versions_non_keywords() {
        let output = preprocess_with_profile(
            "`begin_keywords \"1364-2001\"\nmodule top;\n  reg [63:0] logic;\nendmodule\n`end_keywords\n",
            Some("sv_2017"),
        );
        assert!(
            output.text.contains("reg [63:0] \\logic ;"),
            "{}",
            output.text
        );
        assert!(output.diagnostics.is_empty());
    }

    /// A keyword of the region's OWN version is untouched, and so is one outside the region.
    #[test]
    fn a_regions_own_keywords_and_the_text_outside_it_are_untouched() {
        let output = preprocess_with_profile(
            "reg logic;\n`begin_keywords \"1364-2005\"\nwire uwire_ok;\nreg logic;\n`end_keywords\nreg logic;\n",
            Some("sv_2017"),
        );
        let lines: Vec<&str> = output.text.lines().collect();
        assert_eq!(lines[0], "reg logic;", "before the region");
        assert_eq!(lines[3], "reg \\logic ;", "inside the region");
        assert_eq!(lines[5], "reg logic;", "after the region");
    }

    /// §22.14: *"The `begin_keywords...`end_keywords directive pair can be nested. Each nested pair
    /// is stacked"* — sv2v's `test/lex/begin_keywords.sv` turns on exactly this: `automatic` is a
    /// keyword of the INNER `"1364-2005"` region and not of the outer `"1364-1995"` one.
    #[test]
    fn nested_regions_stack_and_the_inner_version_wins() {
        let output = preprocess_with_profile(
            "`begin_keywords \"1364-1995\"\ntask foo;\n  integer automatic = 2;\nendtask\n\
             `begin_keywords \"1364-2005\"\ntask automatic bar;\n  integer logic = 3;\nendtask\n\
             `end_keywords\nreg automatic;\n`end_keywords\n",
            Some("sv_2017"),
        );
        assert!(
            output.text.contains("integer \\automatic  = 2;"),
            "{}",
            output.text
        );
        assert!(
            output.text.contains("task automatic bar;"),
            "{}",
            output.text
        );
        assert!(
            output.text.contains("integer \\logic  = 3;"),
            "{}",
            output.text
        );
        assert!(
            output.text.contains("reg \\automatic ;"),
            "outer region resumes: {}",
            output.text
        );
    }

    /// §22.14.4: `"1364-2001-noconfig"` is `"1364-2001"` minus the ten configuration keywords, so
    /// `cell` is an identifier there and `automatic` is still a keyword.
    #[test]
    fn noconfig_excludes_the_configuration_keywords_and_nothing_else() {
        let output = preprocess_with_profile(
            "`begin_keywords \"1364-2001-noconfig\"\ntask automatic baz;\n  integer cell = 3;\nendtask\n`end_keywords\n",
            Some("sv_2017"),
        );
        assert!(
            output.text.contains("task automatic baz;"),
            "{}",
            output.text
        );
        assert!(
            output.text.contains("integer \\cell  = 3;"),
            "{}",
            output.text
        );
    }

    /// The rendering is LEXICAL: a keyword named inside a comment, a string or an existing escaped
    /// identifier is not a token of the design and is left exactly as written.
    #[test]
    fn comments_strings_literals_and_escaped_identifiers_are_left_alone() {
        let output = preprocess_with_profile(
            "`begin_keywords \"1364-1995\"\n// automatic in a line comment\n\
             /* automatic in a block comment */\n\
             module top;\n  initial $display(\"automatic in a string\");\n\
             wire \\automatic_already = 1'b1;\n  integer automatic;\nendmodule\n`end_keywords\n",
            Some("sv_2017"),
        );
        assert!(
            output.text.contains("// automatic in a line comment"),
            "{}",
            output.text
        );
        assert!(
            output.text.contains("/* automatic in a block comment */"),
            "{}",
            output.text
        );
        assert!(
            output.text.contains("$display(\"automatic in a string\")"),
            "{}",
            output.text
        );
        assert!(
            output.text.contains("wire \\automatic_already = 1'b1;"),
            "{}",
            output.text
        );
        assert!(
            output.text.contains("integer \\automatic ;"),
            "{}",
            output.text
        );
        assert_eq!(
            output.text.matches("\\automatic ").count(),
            1,
            "{}",
            output.text
        );
    }

    /// The parse profile decides what has to be unreserved: under `verilog_2005` the parser never
    /// reserved `logic`, so a `"1364-2001"` region renders nothing at all.
    #[test]
    fn the_parse_profile_decides_which_words_need_unreserving() {
        let source = "`begin_keywords \"1364-2001\"\nreg logic;\nreg automatic;\n`end_keywords\n";
        let sv = preprocess_with_profile(source, Some("sv_2017"));
        assert!(sv.text.contains("reg \\logic ;"));
        let v2005 = preprocess_with_profile(source, Some("verilog_2005"));
        assert!(v2005.text.contains("reg logic;"), "{}", v2005.text);
        assert!(v2005.text.contains("reg automatic;"), "{}", v2005.text);
    }

    /// The source map is re-indexed by the exact delta of every rendering, so a byte AFTER one
    /// still names the line its author wrote.
    #[test]
    fn the_source_map_survives_the_rendering() {
        let output = preprocess_with_profile(
            "`begin_keywords \"1364-2001\"\nmodule top;\n  reg [63:0] logic;\n  wire w;\nendmodule\n`end_keywords\n",
            Some("sv_2017"),
        );
        let at = output
            .text
            .find("wire w;")
            .expect("the line after the rendering");
        let entry = output
            .source_map
            .iter()
            .find(|entry| at >= entry.output_start && at < entry.output_end)
            .expect("a source-map chunk covers it");
        assert_eq!(entry.source.line, 4);
        // the map still TILES the rendered text: ascending, contiguous, and ending at its length
        let mut previous_end = 0usize;
        for entry in &output.source_map {
            assert_eq!(
                entry.output_start, previous_end,
                "chunks tile without a gap"
            );
            assert!(
                entry.output_end <= output.text.len(),
                "chunk past the rendered text"
            );
            previous_end = entry.output_end;
        }
        assert_eq!(
            previous_end,
            output.text.len(),
            "the last chunk ends at the text"
        );
        // and the rendered token lives inside the chunk of the line that wrote it
        let rendered = output.text.find("\\logic ").expect("the rendering");
        let owner = output
            .source_map
            .iter()
            .find(|entry| rendered >= entry.output_start && rendered < entry.output_end)
            .expect("a chunk covers the rendering");
        assert_eq!(owner.source.line, 3);
    }

    /// §22.14: *"It shall be an error if an implementation does not recognize the version_specifier
    /// used with the `begin_keywords directive."* The implementation that errors is the GRAMMAR —
    /// `keywords_version_specifier_<profile>` admits exactly its own edition's list — so this stage
    /// REPORTS it, opens no region, and leaves the text for the parser to refuse. Promoting the
    /// warning makes the clause fatal here for an operator who wants that.
    #[test]
    fn an_unrecognized_version_specifier_is_reported_and_opens_no_region() {
        let output = preprocess_with_profile(
            "`begin_keywords \"1364-2010\"\nreg logic;\n`end_keywords\n",
            Some("sv_2017"),
        );
        let diagnostic = output
            .diagnostics
            .iter()
            .find(|d| d.code == "W_SVPP_UNKNOWN_KEYWORDS_VERSION")
            .expect("the unknown specifier is reported");
        assert!(
            diagnostic.message.contains("1364-2010"),
            "{}",
            diagnostic.message
        );
        assert!(
            output.text.contains("reg logic;"),
            "no region opened: {}",
            output.text
        );

        let dir = create_repo_scratch_dir("svpp_keywords_unknown_strict");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`begin_keywords \"1364-2010\"\nreg logic;\n`end_keywords\n",
        )
        .expect("write input");
        let config = SvPreprocessorConfig {
            parse_profile: Some("sv_2017".to_string()),
            strict_warning_codes: parse_strict_warning_codes("W_SVPP_UNKNOWN_KEYWORDS_VERSION"),
            ..SvPreprocessorConfig::default()
        };
        let error = preprocess_systemverilog_file(&input, &config)
            .expect_err("the promoted warning is fatal");
        assert!(format!("{error:#}").contains("W_SVPP_UNKNOWN_KEYWORDS_VERSION"));
    }

    /// A region naming a version the profile's own edition does not define, and one reserving MORE
    /// than the profile, are each reported — the second because text cannot create a keyword.
    #[test]
    fn a_region_the_profile_cannot_fully_express_is_reported() {
        let output = preprocess_with_profile(
            "`begin_keywords \"1800-2005\"\nmodule m; endmodule\n`end_keywords\n",
            Some("verilog_2005"),
        );
        let codes: Vec<&str> = output.diagnostics.iter().map(|d| d.code.as_str()).collect();
        assert!(
            codes.contains(&"W_SVPP_KEYWORDS_VERSION_NOT_IN_EDITION"),
            "{codes:?}"
        );
        assert!(
            codes.contains(&"W_SVPP_KEYWORDS_VERSION_ABOVE_PROFILE"),
            "{codes:?}"
        );
        assert!(
            output.text.contains("module m; endmodule"),
            "{}",
            output.text
        );
    }

    /// IEEE 1364-2005 §19.11: *"Each `begin_keywords directive must be paired with an
    /// `end_keywords directive."* An unpaired close is reported and renders nothing.
    #[test]
    fn an_unmatched_end_keywords_is_refused() {
        // `SVPP-EXPANSION.6f`: through `1.0.250` this was a warning (W_SVPP_UNMATCHED_END_KEYWORDS) and the
        // directive was left for the parser, which reads the line as a directive and accepted it.
        let dir = create_repo_scratch_dir("svpp_unmatched_end_keywords_profile");
        let input = dir.join("top.sv");
        fs::write(&input, "module m; endmodule\n`end_keywords\n").expect("write");
        let config = SvPreprocessorConfig { parse_profile: Some("sv_2017".to_string()), ..SvPreprocessorConfig::default() };
        let err = preprocess_systemverilog_file(&input, &config).expect_err("closes no open region");
        assert!(format!("{err:#}").contains("`end_keywords closes no open `begin_keywords region"), "{err:#}");
    }

    /// §22.14: the region *"affects all source code that follows the directive, even across source
    /// code file boundaries, until the matching `end_keywords directive or the end of the
    /// compilation unit"* — so an unclosed region carries into the unit's next file.
    #[test]
    fn an_open_region_carries_across_the_compilation_units_files() {
        let dir = create_repo_scratch_dir("svpp_keywords_unit");
        let first = dir.join("first.sv");
        let second = dir.join("second.sv");
        fs::write(&first, "`begin_keywords \"1364-1995\"\nreg automatic;\n").expect("write first");
        fs::write(&second, "reg automatic;\n").expect("write second");
        let config = SvPreprocessorConfig {
            parse_profile: Some("sv_2017".to_string()),
            ..SvPreprocessorConfig::default()
        };
        let outputs = preprocess_systemverilog_unit(&[first, second], &config).expect("preprocess");
        assert!(
            outputs[0].text.contains("reg \\automatic ;"),
            "{}",
            outputs[0].text
        );
        assert!(
            outputs[1].text.contains("reg \\automatic ;"),
            "{}",
            outputs[1].text
        );
    }

    /// A file with no §22.14 directive is byte-identical to what it was before this pass existed —
    /// the whole corpus takes that path.
    #[test]
    fn a_file_with_no_region_is_untouched() {
        let source =
            "module top;\n  logic [1:0] automatic_like;\n  wire interface_name;\nendmodule\n";
        let output = preprocess_with_profile(source, Some("sv_2017"));
        assert_eq!(output.text, source);
        assert!(output.diagnostics.is_empty());
        assert!(
            !output
                .events
                .iter()
                .any(|event| event.kind == PreprocessorEventKind::KeywordsRegion)
        );
    }

    /// The event log names each directive's own line and what the region cost.
    #[test]
    fn the_event_log_names_each_region_and_its_renderings() {
        let output = preprocess_with_profile(
            "`begin_keywords \"1364-1995\"\nreg automatic;\nreg logic;\n`end_keywords\n",
            Some("sv_2017"),
        );
        let regions: Vec<(usize, String)> = output
            .events
            .iter()
            .filter(|event| event.kind == PreprocessorEventKind::KeywordsRegion)
            .map(|event| (event.line, event.detail.clone()))
            .collect();
        assert_eq!(
            regions,
            vec![
                (1, "begin_keywords \"1364-1995\" unreserves=146".to_string()),
                (4, "end_keywords \"1364-1995\" escaped=2".to_string()),
            ]
        );
    }

    #[test]
    fn predefined_macro_parse_accepts_name_and_name_equals_text() {
        let bare = PredefinedMacro::parse("UVM").expect("bare name");
        assert_eq!(bare.name, "UVM");
        assert_eq!(bare.text, "");
        let valued = PredefinedMacro::parse(" WIDTH=16 ").expect("name=text");
        assert_eq!(valued.name, "WIDTH");
        assert_eq!(valued.text, "16");
        // everything after the FIRST `=` is text, verbatim
        let nested = PredefinedMacro::parse("EXPR=a==b").expect("text with =");
        assert_eq!(nested.text, "a==b");
        let empty_text = PredefinedMacro::parse("FLAG=").expect("explicit empty text");
        assert_eq!(empty_text.text, "");
    }

    #[test]
    fn predefined_macro_parse_refuses_illegal_names() {
        assert!(
            PredefinedMacro::parse("").is_err(),
            "an empty define has no name"
        );
        assert!(
            PredefinedMacro::parse("=1").is_err(),
            "a text without a name"
        );
        assert!(
            PredefinedMacro::parse("1ABC").is_err(),
            "a name must start like an identifier"
        );
        assert!(
            PredefinedMacro::parse("F(x)=x+1").is_err(),
            "a function-like predefine has no command-line meaning and is refused, not misread"
        );
    }

    #[test]
    fn predefined_macro_is_visible_before_the_first_source_line() {
        let dir = create_repo_scratch_dir("svpp_predefine");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`ifdef UVM\nlogic uvm_branch;\n`else\nlogic plain_branch;\n`endif\nlogic [`WIDTH-1:0] d;\n",
        )
        .expect("write top");
        let config = SvPreprocessorConfig {
            defines: vec![
                PredefinedMacro::parse("UVM").expect("UVM"),
                PredefinedMacro::parse("WIDTH=8").expect("WIDTH"),
            ],
            ..SvPreprocessorConfig::default()
        };
        let output = preprocess_systemverilog_file(&input, &config).expect("preprocess");
        assert!(output.text.contains("logic uvm_branch;"), "{}", output.text);
        assert!(
            !output.text.contains("logic plain_branch;"),
            "{}",
            output.text
        );
        assert!(output.text.contains("logic [8-1:0] d;"), "{}", output.text);

        // the same file with NO context takes the other branch and leaves `WIDTH unresolved — which
        // `SVPP-EXPANSION.6f` REFUSES, naming the macro (a context gap, read at the stage); under `keep`
        // the usage stays as written, as before
        let err = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect_err("without the context `WIDTH is undefined");
        assert!(format!("{err:#}").contains("`WIDTH is neither a defined macro"), "{err:#}");
        let keep = SvPreprocessorConfig {
            undefined_macro_policy: UndefinedMacroPolicy::Keep,
            ..SvPreprocessorConfig::default()
        };
        let bare = preprocess_systemverilog_file(&input, &keep).expect("preprocess without context (keep)");
        assert!(bare.text.contains("logic plain_branch;"), "{}", bare.text);
        assert!(bare.text.contains("`WIDTH"), "{}", bare.text);
    }

    #[test]
    fn predefined_macro_without_text_expands_to_nothing_and_is_an_ordinary_definition() {
        let dir = create_repo_scratch_dir("svpp_predefine_empty");
        let input = dir.join("top.sv");
        // `NAME` alone == a bare `define NAME: defined, empty text; `undef removes it like any other
        fs::write(
            &input,
            "wire a`EMPTY;\n`undef EMPTY\n`ifdef EMPTY\nlogic still_defined;\n`endif\n",
        )
        .expect("write top");
        let config = SvPreprocessorConfig {
            defines: vec![PredefinedMacro::parse("EMPTY").expect("EMPTY")],
            ..SvPreprocessorConfig::default()
        };
        let output = preprocess_systemverilog_file(&input, &config).expect("preprocess");
        assert!(output.text.contains("wire a;"), "{}", output.text);
        assert!(!output.text.contains("still_defined"), "{}", output.text);
    }

    #[test]
    fn a_source_define_supersedes_a_predefine_under_the_default_policy() {
        let dir = create_repo_scratch_dir("svpp_predefine_redefine");
        let input = dir.join("top.sv");
        fs::write(&input, "`define WIDTH 32\nlogic [`WIDTH-1:0] d;\n").expect("write top");
        let mut config = SvPreprocessorConfig {
            defines: vec![PredefinedMacro::parse("WIDTH=8").expect("WIDTH")],
            ..SvPreprocessorConfig::default()
        };
        let output = preprocess_systemverilog_file(&input, &config).expect("preprocess");
        assert!(output.text.contains("logic [32-1:0] d;"), "{}", output.text);

        // and under the `error` policy the source `define IS a redefinition of the predefine
        config.macro_redefinition_policy = MacroRedefinitionPolicy::Error;
        let refused = preprocess_systemverilog_file(&input, &config);
        assert!(
            refused.is_err(),
            "a predefine is an ordinary definition to the policy"
        );
    }

    #[test]
    fn compilation_unit_carries_the_macro_table_across_files_in_order() {
        let dir = create_repo_scratch_dir("svpp_unit");
        let defs = dir.join("defs.svh");
        let first = dir.join("a.sv");
        let second = dir.join("b.sv");
        fs::write(&defs, "`define FROM_DEFS 1\n").expect("write defs");
        fs::write(&first, "`define FROM_A 2\nlogic [`FROM_DEFS:0] a;\n").expect("write a");
        fs::write(
            &second,
            "logic [`FROM_A:`FROM_DEFS] b;\n`undef FROM_A\n`ifdef FROM_A\nlogic still_a;\n`endif\n",
        )
        .expect("write b");
        let outputs = preprocess_systemverilog_unit(
            &[defs.clone(), first.clone(), second.clone()],
            &SvPreprocessorConfig::default(),
        )
        .expect("unit");
        assert_eq!(outputs.len(), 3);
        // file 2 sees file 1's macro; file 3 sees both; every output is that FILE's text only
        assert!(
            outputs[1].text.contains("logic [1:0] a;"),
            "{}",
            outputs[1].text
        );
        assert!(
            outputs[2].text.contains("logic [2:1] b;"),
            "{}",
            outputs[2].text
        );
        assert!(!outputs[2].text.contains("still_a"), "{}", outputs[2].text);
        assert!(
            !outputs[2].text.contains("logic [1:0] a;"),
            "per-file text, not a concatenation"
        );
        // and the per-file surfaces are per FILE
        assert!(
            outputs[0]
                .source_map
                .iter()
                .all(|m| m.source.file.ends_with("defs.svh"))
        );
        assert!(
            outputs[2]
                .source_map
                .iter()
                .all(|m| m.source.file.ends_with("b.sv"))
        );
        assert!(
            outputs[1]
                .included_files
                .iter()
                .all(|f| f.ends_with("a.sv"))
        );
    }

    #[test]
    fn a_fresh_unit_does_not_see_a_previous_units_macros() {
        let dir = create_repo_scratch_dir("svpp_unit_isolation");
        let first = dir.join("a.sv");
        let second = dir.join("b.sv");
        fs::write(&first, "`define ONLY_IN_A 1\n").expect("write a");
        fs::write(&second, "logic [`ONLY_IN_A:0] b;\n").expect("write b");
        let config = SvPreprocessorConfig::default();
        let _ =
            preprocess_systemverilog_unit(std::slice::from_ref(&first), &config).expect("unit 1");
        // `SVPP-EXPANSION.6f`: the second unit does not see the first's macro, so its usage is an
        // UNDEFINED one — refused, naming the name (the isolation claim, read through the refusal)
        let err = preprocess_systemverilog_unit(std::slice::from_ref(&second), &config)
            .expect_err("unit 2 does not see ONLY_IN_A");
        assert!(format!("{err:#}").contains("`ONLY_IN_A is neither a defined macro"), "{err:#}");
        // and the one-file entry is the one-file unit — byte-identical text, read under `keep`
        let keep = SvPreprocessorConfig {
            undefined_macro_policy: UndefinedMacroPolicy::Keep,
            ..SvPreprocessorConfig::default()
        };
        let alone = preprocess_systemverilog_unit(std::slice::from_ref(&second), &keep).expect("unit 2 (keep)");
        assert!(alone[0].text.contains("`ONLY_IN_A"), "{}", alone[0].text);
        let single = preprocess_systemverilog_file(&second, &keep).expect("single");
        assert_eq!(single.text, alone[0].text);
    }

    #[test]
    fn a_unit_failure_names_the_file_that_failed() {
        let dir = create_repo_scratch_dir("svpp_unit_failure");
        let first = dir.join("a.sv");
        let second = dir.join("b.sv");
        fs::write(&first, "`define OK 1\n").expect("write a");
        fs::write(&second, "`ifdef OK\nlogic unterminated;\n").expect("write b");
        let err = preprocess_systemverilog_unit(&[first, second], &SvPreprocessorConfig::default())
            .expect_err("the second file is unterminated");
        let text = format!("{:#}", err);
        assert!(text.contains("compilation unit file 2 of 2"), "{}", text);
        assert!(text.contains("b.sv"), "{}", text);
    }

    #[test]
    fn resetall_does_not_undefine_macros() {
        // IEEE 1800-2017 §22.3: directives with no default value (`define among them) are not
        // affected by `resetall — pinned so a future "reset the table" edit cannot land silently.
        let dir = create_repo_scratch_dir("svpp_resetall");
        let input = dir.join("top.sv");
        fs::write(&input, "`define KEEP 7\n`resetall\nlogic [`KEEP:0] k;\n").expect("write top");
        let output = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("preprocess");
        assert!(output.text.contains("logic [7:0] k;"), "{}", output.text);
        assert!(
            output.text.contains("`resetall"),
            "the directive itself passes through"
        );
    }

    // ── SVPP-EXPANSION.5 — the three census defects (a) (b) (c) and the expansion line numbers ──

    #[test]
    fn a_define_formal_list_may_span_lines_without_a_backslash() {
        // IEEE 1800-2017 §22.5.1: formals are "separated by commas and optionally white space" (§5.3:
        // newlines are white space). OpenTitan `cip_base_vseq__tl_errors.svh:8`, the census's 4 rows.
        let dir = create_repo_scratch_dir("svpp_formals_span");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define MK(task_name_, with_c_,\n           seq_t_ = host_seq #(item_t),\n           seqr_t)   \\\n  begin seq_t_ s; run(task_name_, seqr_t); end\n`MK(t1, 0, , sqr)\n`MK(t2, 1, my_seq, sqr2)\n",
        )
        .expect("write");
        let output = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("a formal list spanning three lines preprocesses");
        assert!(
            output
                .text
                .contains("begin host_seq #(item_t) s; run(t1, sqr); end"),
            "{}",
            output.text
        );
        assert!(
            output.text.contains("begin my_seq s; run(t2, sqr2); end"),
            "{}",
            output.text
        );
        assert!(!output.text.contains("`MK"), "{}", output.text);
    }

    #[test]
    fn a_conditional_directive_may_appear_mid_line() {
        // IEEE 1800-2017 §22.6: "These directives may appear anywhere in the source description."
        let dir = create_repo_scratch_dir("svpp_midline_cond");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "module m;\n  assign a = `ifdef B b `else c `endif ;\nendmodule\n",
        )
        .expect("write");
        let plain =
            preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default()).expect("plain");
        assert!(
            plain.text.contains("assign a =")
                && plain.text.contains(" c ")
                && !plain.text.contains(" b "),
            "{}",
            plain.text
        );
        assert!(
            !plain.text.contains('`'),
            "no directive survives: {}",
            plain.text
        );
        let config = SvPreprocessorConfig {
            defines: vec![PredefinedMacro::parse("B").expect("B")],
            ..SvPreprocessorConfig::default()
        };
        let with_b = preprocess_systemverilog_file(&input, &config).expect("with B");
        assert!(
            with_b.text.contains(" b ") && !with_b.text.contains(" c "),
            "{}",
            with_b.text
        );
        // the same line number for every piece of the split line
        assert!(
            with_b
                .source_map
                .iter()
                .filter(|e| e.source.line == 2)
                .count()
                >= 2,
            "{:?}",
            with_b.source_map
        );
    }

    #[test]
    fn a_macro_body_starting_with_a_conditional_expands_when_used_mid_line() {
        // the OpenTitan shape: `ASSERT_ERROR`'s body opens with `ifdef UVM; `ASSERT` uses it mid-line
        let dir = create_repo_scratch_dir("svpp_nested_cond");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define ERR(n) \\\n`ifdef UVM \\\n  uvm_err(n); \\\n`else \\\n  plain_err(n); \\\n`endif\n`define CHECK(p) if (!(p)) begin `ERR(p) end\nmodule m;\n  initial begin\n    `CHECK(a)\n  end\nendmodule\n",
        )
        .expect("write");
        let plain = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("a directive that opens an expansion mid-line is honoured");
        assert!(
            plain.text.contains("plain_err(a);") && !plain.text.contains("uvm_err"),
            "{}",
            plain.text
        );
        assert!(
            !plain.text.contains("`ifdef") && !plain.text.contains("`else"),
            "{}",
            plain.text
        );
        let config = SvPreprocessorConfig {
            defines: vec![PredefinedMacro::parse("UVM").expect("UVM")],
            ..SvPreprocessorConfig::default()
        };
        let uvm = preprocess_systemverilog_file(&input, &config).expect("with UVM");
        assert!(
            uvm.text.contains("uvm_err(a);") && !uvm.text.contains("plain_err"),
            "{}",
            uvm.text
        );
        // every chunk of the expansion names the USE line (10), never a line of its own
        assert!(
            plain.source_map.iter().all(|e| e.source.line != 1
                || !e.source.file.ends_with("top.sv")
                || e.output_start < plain.text.find("initial").unwrap_or(usize::MAX)),
            "{:?}",
            plain.source_map
        );
        assert!(
            plain
                .events
                .iter()
                .filter(|e| matches!(e.kind, super::PreprocessorEventKind::Ifdef))
                .all(|e| e.line == 10),
            "{:?}",
            plain.events
        );
    }

    #[test]
    fn a_trailing_comment_after_a_conditional_symbol_is_not_part_of_the_symbol() {
        // measured before .5: `ifdef X // c read the symbol as "X // c" and took the else branch
        let dir = create_repo_scratch_dir("svpp_trailing_comment");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define X 1\n`ifdef X // a trailing comment\nlogic yes;\n`else\nlogic no;\n`endif // X\n`ifndef Y /* block */\nlogic y_undefined;\n`elsif X // also\nlogic unreachable;\n`endif\n",
        )
        .expect("write");
        let config = SvPreprocessorConfig {
            conditional_symbol_policy: ConditionalSymbolPolicy::Error, // any misread symbol would now ERROR
            ..SvPreprocessorConfig::default()
        };
        let output =
            preprocess_systemverilog_file(&input, &config).expect("comments are not symbols");
        assert!(
            output.text.contains("logic yes;") && !output.text.contains("logic no;"),
            "{}",
            output.text
        );
        assert!(
            output.text.contains("logic y_undefined;") && !output.text.contains("unreachable"),
            "{}",
            output.text
        );
        assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
        assert!(
            !output.text.contains("// X") && !output.text.contains("/* block */"),
            "a directive's trailing comment is consumed with it: {}",
            output.text
        );
    }

    #[test]
    fn a_re_entered_expansions_diagnostic_names_the_use_line() {
        // a macro whose body carries an `else with no `ifdef: the error names the USE line, not line 3 of the expansion
        let dir = create_repo_scratch_dir("svpp_reentry_line");
        let input = dir.join("top.sv");
        fs::write(&input, "`define BAD(n) \\\n  x(n); \\\n`else \\\n  y(n);\nmodule m;\n\n\n  initial `BAD(1)\nendmodule\n").expect("write");
        let err = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect_err("an unmatched `else");
        let text = format!("{:#}", err);
        assert!(
            text.contains("top.sv:8"),
            "the use line, not the expansion's own: {}",
            text
        );
    }

    #[test]
    fn include_followed_by_a_macro_expands_the_macro_first() {
        // `SVPP-EXPANSION.5`(m): IEEE 1800-2017 §22.5.1 — "The `include directive can be followed by a
        // macro, instead of a string literal" (docs/systemverilog/2017/txt/section-22-compiler-directives.txt:341-343;
        // 1800-2023 :409-411). `.5`(c) had ruled the form a tool extension from Syntax 22-1 alone.
        let dir = create_repo_scratch_dir("svpp_include_macro");
        fs::write(dir.join("inc.svh"), "wire from_inc;\n").expect("write inc");
        let input = dir.join("top.sv");
        // an object macro holding the quoted filename (black-parrot's `BP_CUSTOM_DEFINES_PATH shape)
        fs::write(&input, "`define F \"inc.svh\"\n`include `F\nwire after;\n").expect("write");
        let output = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("an object macro naming the file is included");
        assert!(output.text.contains("wire from_inc;"), "{}", output.text);
        assert!(output.text.contains("wire after;"), "{}", output.text);
        // the standard's own shape: a function macro whose text stringizes its argument
        fs::write(
            &input,
            "`define home(filename) `\"filename`\"\n`include `home(inc.svh) // the LRM's example\nwire after;\n",
        )
        .expect("write");
        let output = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("the LRM's own example shape is included");
        assert!(output.text.contains("wire from_inc;"), "{}", output.text);
        assert!(output.text.contains("wire after;"), "{}", output.text);
        // …and it reads EXACTLY as the literal form does: what follows the filename on the directive
        // line is the next logical line's for both (`.6b.5`(b)), so the two outputs are one text.
        let via_macro = output.text.clone();
        fs::write(
            &input,
            "`define home(filename) `\"filename`\"\n`include \"inc.svh\" // the LRM's example\nwire after;\n",
        )
        .expect("write");
        let literal = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("the literal form");
        assert_eq!(
            via_macro, literal.text,
            "a macro-named include reads as the literal one"
        );
    }

    #[test]
    fn include_followed_by_a_macro_that_is_not_a_filename_is_refused_naming_the_clause() {
        // what the macro yields must still be Syntax 22-1's "filename" or <filename>
        let dir = create_repo_scratch_dir("svpp_include_macro_bad");
        let input = dir.join("top.sv");
        fs::write(&input, "`define F inc.svh\n`include `F\n").expect("write");
        let err = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect_err("an unquoted expansion is not a filename form");
        let text = format!("{:#}", err);
        assert!(
            text.contains("22.5.1") && text.contains("top.sv:2") && !text.contains("tool extension"),
            "{}",
            text
        );
        // an undefined macro expands to nothing usable
        fs::write(&input, "`include `NOT_DEFINED\n").expect("write");
        let err = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect_err("an undefined macro names no file");
        let text = format!("{:#}", err);
        assert!(text.contains("22.5.1") && text.contains("top.sv:1"), "{}", text);
    }

    #[test]
    fn an_empty_actual_takes_the_formals_default() {
        // IEEE 1800-2017 §22.5.1's own examples (section-22-compiler-directives.md, the MACRO1 block):
        // `MACRO1 ( , 2, 3 ) — "argument a omitted, replaced by default"; `MACRO3 ( ) — all defaults.
        let dir = create_repo_scratch_dir("svpp_empty_actual_default");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define MACRO1(a=5,b=\"B\",c) $display(a,,b,,c);\n`define MACRO3(a=5, b=0, c=\"C\") $display(a,,b,,c);\n`MACRO1 ( , 2, 3 )\n`MACRO1 ( 1 , , 3 )\n`MACRO3 ( 1 )\n`MACRO3 ( )\n",
        )
        .expect("write");
        let output = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("preprocess");
        assert!(
            output.text.contains("$display(5,,2,,3);"),
            "{}",
            output.text
        );
        assert!(
            output.text.contains("$display(1,,\"B\",,3);"),
            "{}",
            output.text
        );
        assert!(
            output.text.contains("$display(1,,0,,\"C\");"),
            "{}",
            output.text
        );
        assert!(
            output.text.contains("$display(5,,0,,\"C\");"),
            "{}",
            output.text
        );
    }

    // ── SVPP-EXPANSION.5 (g) (h) — the two items the whole lane found ─────────────────────────

    #[test]
    fn a_missing_actual_for_a_formal_without_a_default_is_refused() {
        // IEEE 1800-2017 §22.5.1's examples (section-22-compiler-directives.md:251-266):
        //   `MACRO1 ( , 2, )   // argument c omitted, replaced by nothing   ← EMPTY, no default: legal
        //   `MACRO1 ( 1 )      // ILLEGAL: b and c omitted, no default for c ← MISSING, no default: error
        //   `MACRO2 (, 2)      // a and c omitted, replaced by defaults      ← MISSING, with defaults: legal
        let dir = create_repo_scratch_dir("svpp_missing_actual");
        let legal = dir.join("legal.sv");
        fs::write(
            &legal,
            "`define MACRO1(a=5,b=\"B\",c) $display(a,,b,,c);\n`define MACRO2(a=5, b, c=\"C\") $display(a,,b,,c);\n`MACRO1 ( , 2, )\n`MACRO2 (1, , 3)\n`MACRO2 (, 2)\n",
        )
        .expect("write");
        let out = preprocess_systemverilog_file(&legal, &SvPreprocessorConfig::default())
            .expect("legal forms");
        assert!(out.text.contains("$display(5,,2,,);"), "{}", out.text);
        assert!(out.text.contains("$display(1,,,,3);"), "{}", out.text);
        assert!(out.text.contains("$display(5,,2,,\"C\");"), "{}", out.text);

        let illegal = dir.join("illegal.sv");
        fs::write(
            &illegal,
            "`define MACRO1(a=5,b=\"B\",c) $display(a,,b,,c);\n`MACRO1 ( 1 )\n",
        )
        .expect("write");
        let err = preprocess_systemverilog_file(&illegal, &SvPreprocessorConfig::default())
            .expect_err("MACRO1 ( 1 )");
        let text = format!("{:#}", err);
        assert!(
            text.contains("E_SVPP_MISSING_MACRO_ARGUMENT")
                && text.contains("'c'")
                && text.contains("illegal.sv:2"),
            "{}",
            text
        );

        // the sv-tests shape: `D() on a two-formal macro with no defaults
        let d = dir.join("d.sv");
        fs::write(
            &d,
            "`define D(x,y) initial $display(\"start\", x , y, \"end\");\n`D()\n",
        )
        .expect("write");
        let err =
            preprocess_systemverilog_file(&d, &SvPreprocessorConfig::default()).expect_err("D()");
        assert!(format!("{:#}", err).contains("E_SVPP_MISSING_MACRO_ARGUMENT"));

        // and MORE actuals than formals — `D(,,) — the standard's "illegal, more actual than formal
        // arguments" (section-22-compiler-directives.md:226; sv-tests 22.5.1--define-expansion_8)
        let extra = dir.join("extra.sv");
        fs::write(
            &extra,
            "`define D(x,y) initial $display(\"start\", x , y, \"end\");\n`D(,,)\n",
        )
        .expect("write");
        let err = preprocess_systemverilog_file(&extra, &SvPreprocessorConfig::default())
            .expect_err("D(,,)");
        let text = format!("{:#}", err);
        assert!(
            text.contains("E_SVPP_EXTRA_MACRO_ARGUMENT")
                && text.contains("3 arguments for 2 formals"),
            "{}",
            text
        );
        // while `D(,) — two EMPTY actuals for two formals without defaults — is "replaced by nothing"
        let two_empty = dir.join("two_empty.sv");
        fs::write(
            &two_empty,
            "`define D(x,y) initial $display(\"start\", x , y, \"end\");\n`D(,)\n",
        )
        .expect("write");
        let ok = preprocess_systemverilog_file(&two_empty, &SvPreprocessorConfig::default())
            .expect("D(,) is legal");
        assert!(
            ok.text
                .contains("initial $display(\"start\",  , , \"end\");"),
            "{}",
            ok.text
        );
    }

    #[test]
    fn a_recursive_macro_is_refused_not_expanded() {
        // §22.5.1: "It shall be an error for a macro to expand directly or indirectly to text
        // containing another usage of itself" — direct, indirect, and the exponential shape that
        // timed out the corpus lane (verilator t_pp_circdef_bad.v), which must be refused FAST.
        let dir = create_repo_scratch_dir("svpp_recursive");
        let direct = dir.join("direct.sv");
        fs::write(&direct, "`define e fun `e\n`e\n").expect("write");
        let err = preprocess_systemverilog_file(&direct, &SvPreprocessorConfig::default())
            .expect_err("direct");
        let text = format!("{:#}", err);
        assert!(
            text.contains("E_SVPP_RECURSIVE_MACRO")
                && text.contains("`e -> `e")
                && text.contains("direct.sv:2"),
            "{}",
            text
        );

        let indirect = dir.join("indirect.sv");
        fs::write(
            &indirect,
            "`define A x `B y\n`define B z `A w\nlogic q = `A;\n",
        )
        .expect("write");
        let err = preprocess_systemverilog_file(&indirect, &SvPreprocessorConfig::default())
            .expect_err("indirect");
        let text = format!("{:#}", err);
        assert!(
            text.contains("E_SVPP_RECURSIVE_MACRO") && text.contains("`A -> `B -> `A"),
            "{}",
            text
        );

        let exponential = dir.join("exponential.sv");
        fs::write(
            &exponential,
            "`define WIDTH  12\n`define SEL_NUM_BITS `WIDTH-`SEL_NUM_BITS +: `SEL_NUM_BITS\ntypedef logic [`SEL_NUM_BITS-1:0] d_t;\n",
        )
        .expect("write");
        let started = std::time::Instant::now();
        let err = preprocess_systemverilog_file(&exponential, &SvPreprocessorConfig::default())
            .expect_err("exponential");
        assert!(format!("{:#}", err).contains("E_SVPP_RECURSIVE_MACRO"));
        assert!(
            started.elapsed().as_secs() < 2,
            "refused in {:?}, not expanded first",
            started.elapsed()
        );
    }

    #[test]
    fn an_actual_may_contain_a_usage_of_the_macro_itself() {
        // §22.5.1's own example, verbatim, with the two forbidden readings pinned:
        //   `define TOP(a,b) a + b
        //   `TOP( `TOP(b,1), `TOP(42,a) )   expands to   b + 1 + 42 + a
        //   not into  42 + a + 1 + 42 + a   nor into  b + 1 + 42 + b + 1
        let dir = create_repo_scratch_dir("svpp_actual_self_use");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define TOP(a,b) a + b\nx = `TOP( `TOP(b,1), `TOP(42,a) );\n",
        )
        .expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("an actual may name the macro it is passed to");
        assert!(out.text.contains("x = b + 1 + 42 + a;"), "{}", out.text);
        assert!(
            !out.text.contains("42 + a + 1 + 42 + a") && !out.text.contains("b + 1 + 42 + b + 1"),
            "{}",
            out.text
        );
    }

    #[test]
    fn an_apostrophe_in_a_formals_default_is_not_a_quote() {
        // OpenTitan dv_fcov_macros.svh: `DV_FCOV_INSTANTIATE_CG(NAME_, COND_ = 1'b1, ARGS_ = ()) — three
        // formals; the based literal's apostrophe was read as an opening quote, so the list was two
        // formals and the third actual was dropped silently
        let dir = create_repo_scratch_dir("svpp_apostrophe_default");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define CG(NAME_, COND_ = 1'b1, ARGS_ = ()) inst NAME_ ARGS_ if (COND_);\n`CG(a)\n`CG(b, en)\n`CG(c, 1'b1, (x, y))\n`define PAT(v = '{1, 2}, w = \"s,t\") v w\n`PAT()\n`PAT('{3}, \"u\")\n",
        )
        .expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("three formals, three actuals");
        assert!(out.text.contains("inst a () if (1'b1);"), "{}", out.text);
        assert!(out.text.contains("inst b () if (en);"), "{}", out.text);
        assert!(
            out.text.contains("inst c (x, y) if (1'b1);"),
            "{}",
            out.text
        );
        assert!(out.text.contains("'{1, 2} \"s,t\""), "{}", out.text);
        assert!(out.text.contains("'{3} \"u\""), "{}", out.text);
    }

    #[test]
    fn a_usage_written_inside_an_actual_starts_its_own_expansion_context() {
        // OpenTitan `chip_sw_lc_raw_unlock_vseq.sv:57`: `DV_SPINWAIT( … `DV_SPINWAIT(…) … ) — the
        // inner usage was written by the caller, so its body may re-enter `DV_SPINWAIT_EXIT while the
        // outer expansion still holds that name on its stack (IEEE 1800-2017 §22.5.1: "an actual
        // argument to a macro or a default may contain a usage of itself")
        let dir = create_repo_scratch_dir("svpp_nested_actual_usage");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define EXIT(W_, E_) begin W_ E_ end\n`define SPIN(W_, M_ = \"t\") `EXIT(W_, wait_(M_);)\n`SPIN(while (a) begin `SPIN(x;, \"in\") y; end, \"out\")\n",
        )
        .expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("a caller-written nested usage is not recursion");
        assert!(
            out.text.contains(
                "begin while (a) begin begin x; wait_(\"in\"); end y; end wait_(\"out\"); end"
            ),
            "{}",
            out.text
        );
    }

    #[test]
    fn an_actual_a_body_wrote_does_not_launder_recursion() {
        // `define A(x) `B(`A(x)) — the `A handed to B is A's OWN body text, so B's rescan of it is
        // A expanding indirectly to a usage of itself: the standard's error, however deep
        let dir = create_repo_scratch_dir("svpp_body_written_actual");
        let input = dir.join("top.sv");
        fs::write(&input, "`define B(y) y\n`define A(x) `B(`A(x))\n`A(1)\n").expect("write");
        let err = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect_err("recursion through a body-written actual is refused");
        let text = format!("{:#}", err);
        assert!(
            text.contains("E_SVPP_RECURSIVE_MACRO") && text.contains("`A -> `B -> `A"),
            "{text}"
        );
    }

    // ── SVPP-EXPANSION.6b.2 — the four engine items the refuted rows named ─────────────────────

    #[test]
    fn a_paste_token_in_an_object_macro_joins_its_sides() {
        // sv2v lex/macro_vendor_comment.sv: `define YUCK /``* some awful garbage *``/ — §22.5.1, the
        // paste "delimits lexical tokens without introducing white space"
        let dir = create_repo_scratch_dir("svpp_object_paste");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define YUCK /``* some awful garbage *``/\nmodule top;\n    `YUCK\nendmodule\n",
        )
        .expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("expand");
        assert!(
            out.text.contains("/* some awful garbage */"),
            "{}",
            out.text
        );
        assert!(!out.text.contains("``"), "{}", out.text);
    }

    // ── SVPP-EXPANSION.5(i) — the ORDER around a ``` `` ``` paste ───────────────────────────────

    /// Expand one source text through the whole stage and return the expanded text.
    fn expand_text(prefix: &str, source: &str) -> String {
        let dir = create_repo_scratch_dir(prefix);
        let input = dir.join("top.sv");
        fs::write(&input, source).expect("write");
        preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("expand")
            .text
    }

    /// `SVPP-EXPANSION.6f`: the same, under the `keep` policy — for a test whose SUBJECT is the expansion
    /// machinery (a paste that constructs an undefined name, a region that keeps a bare usage) and whose
    /// text deliberately leaves a usage unresolved; under the default policy that leftover is refused.
    fn expand_text_keep(prefix: &str, source: &str) -> String {
        let dir = create_repo_scratch_dir(prefix);
        let input = dir.join("top.sv");
        fs::write(&input, source).expect("write");
        let keep = SvPreprocessorConfig {
            undefined_macro_policy: UndefinedMacroPolicy::Keep,
            ..SvPreprocessorConfig::default()
        };
        preprocess_systemverilog_file(&input, &keep).expect("expand").text
    }

    #[test]
    fn a_usage_on_the_right_of_a_paste_is_expanded_before_the_join() {
        // sv2v test/lex/macro_macro.sv verbatim — its reference macro_macro.v reads 1 2 / 3 4 / 5 5
        // and Verilator 5.050 (-E -P) reads the same; before this slice the engine read
        // `1_MAGIC , 2_MAGIC` for line 16, because substitution had deleted the pastes and the
        // first defined prefix, `CHOICE_FOO`, won the lookup (Icarus 13.0 reads it that way too;
        // its line 17 is `_MAGIC , _MAGIC`, which nobody else reads)
        let text = expand_text(
            "svpp_paste_order_sv2v",
            "`define SUFFIX _MAGIC\n`define CHOICE_FOO 1\n`define CHOICE_BAR 2\n`define CHOICE_FOO_MAGIC 3\n`define CHOICE_BAR_MAGIC 4\n`define CHOICE__MAGIC 5\n`define MACRO1(A, B) \\\n    `CHOICE_``A , `CHOICE_``B\n`define MACRO2(A, B) \\\n    `CHOICE_``A```SUFFIX , `CHOICE_``B```SUFFIX\n`define MACRO3 \\\n    `CHOICE_```SUFFIX , `CHOICE_```SUFFIX\nmodule top;\n    initial begin\n        $display(`MACRO1(FOO, BAR));\n        $display(`MACRO2(FOO, BAR));\n        $display(`MACRO3);\n    end\nendmodule\n",
        );
        assert!(text.contains("$display(1 , 2);"), "{text}");
        assert!(text.contains("$display(3 , 4);"), "{text}");
        assert!(text.contains("$display(5 , 5);"), "{text}");
        assert!(!text.contains("``"), "{text}");
    }

    #[test]
    fn a_defined_usage_as_a_paste_operand_is_expanded_before_the_join() {
        // Verilator t_preproc.v bug283 (`C5`, EXP: abc) and XACTION3 (EXP: sonet_frame — the joined
        // identifier is NOT looked up again although `sonet_frame` is a macro); and the shape both
        // Verilator 5.050 and Icarus 13.0 read `x_y` / `a_b` for, where this engine read the WHOLE
        // name's definition (`from_whole_name` / `whole`) because the paste was gone before the scan
        let text = expand_text(
            "svpp_paste_order_defined_left",
            "`define A a\n`define B b\n`define C c\n`define C5 `A``b```C\n`define XTYPE sonet\n`define sonet_frame other_frame\n`define XACTION3 `XTYPE``_frame\n`define X x\n`define X_y from_whole_name\n`define M(arg) arg``_y\n`define A_b whole\n`define S5 `A``_b\nwire `C5;\nwire `XACTION3;\nwire `M(`X);\nwire `S5;\n",
        );
        assert!(text.contains("wire abc;"), "{text}");
        assert!(text.contains("wire sonet_frame;"), "{text}");
        assert!(text.contains("wire x_y;"), "{text}");
        assert!(text.contains("wire a_b;"), "{text}");
    }

    #[test]
    fn an_undefined_usage_joined_by_a_paste_is_looked_up_as_the_whole_name() {
        // Verilator t_preproc.v `Q1` (EXP: module zzz ; endmodule — "the existence or non-existence
        // of a base define can make a difference"), t_preproc_str_undef.v `name1` / `name3`
        // (`PREFIX` undefined while `PREFIX_` and `PREFIX_SUFFIX` are defined: `PREFIX_my_suffix
        // stays a usage of an undefined name — no partial lookup of `PREFIX_` — and `name3(PREFIX)`
        // is `my_prefix_suffix`), and the bug1225 families (`XXE_```EMPTY` → XXE_, `XXS_```SOME` →
        // XXS_some, `XYS_```SOMEP(foo)` → XYS_foo), each the line of t_preproc.out
        let text = expand_text_keep(
            "svpp_paste_order_whole_name",
            "`define QA_b zzz\n`define Q1 `QA``_b\n`define PREFIX_ my_prefix_\n`define SUFFIX my_suffix\n`define PREFIX_SUFFIX my_prefix_suffix\n`define name1 `PREFIX``_```SUFFIX\n`define name3(p) ```p``_SUFFIX\n`define EMPTY\n`define SOME some\n`define SOMEP(foo) foo\n`define XXE_FAMILY XXE_```EMPTY\n`define XXS_FAMILY XXS_```SOME\n`define XYS_FAMILY XYS_```SOMEP(foo)\nmodule `Q1 ; endmodule\nA `name1\nB `name3(PREFIX)\nC `XXE_FAMILY\nD `XXS_FAMILY\nE `XYS_FAMILY\n",
        );
        assert!(text.contains("module zzz ; endmodule"), "{text}");
        assert!(text.contains("A `PREFIX_my_suffix\n"), "{text}");
        assert!(text.contains("B my_prefix_suffix\n"), "{text}");
        assert!(text.contains("C XXE_\n"), "{text}");
        assert!(text.contains("D XXS_some\n"), "{text}");
        assert!(text.contains("E XYS_foo\n"), "{text}");
    }

    #[test]
    fn a_constructed_function_usage_takes_its_arguments_unexpanded() {
        // UVM uvm_object_defines.svh: `uvm_pack_``TYPE``(ARG, __local_packer__) — the constructed
        // name is looked up IN PLACE and its argument list is collected as written, which is the
        // order §22.5.1 gives every actual; Surelog PpLppdr/dut.sv: ```rname``_ADR; sv2v
        // macro_arg_invoke.sv: `inv1``inv2(str) and the empty-inv1 form ```MODULE(example6)
        let text = expand_text(
            "svpp_paste_order_constructed_fn",
            "`define pack_int(A) got(A)\n`define P(T, A) `pack_``T``(A)\n`define FOO_ADR 32'h8\n`define REG_ADR(rname) ```rname``_ADR\n`define MODULE(str) module str; endmodule\n`define MACRO_C(inv1, inv2, str) `inv1``inv2(str)\nX `P(int, q)\nY `REG_ADR(FOO)\n`MACRO_C(MOD, ULE, example4)\n`MACRO_C(, MODULE, example6)\n",
        );
        assert!(text.contains("X got(q)\n"), "{text}");
        assert!(text.contains("Y 32'h8\n"), "{text}");
        assert!(text.contains("module example4; endmodule"), "{text}");
        assert!(text.contains("module example6; endmodule"), "{text}");
    }

    #[test]
    fn a_paste_inside_a_plain_string_is_the_strings_by_default() {
        // verible lint/testdata/macro_string_concatenation.sv: `define FOO(arg) "foo``arg``bar" —
        // pinned by .5(i) as "fooxbar", the bytes of the day, with the ruling deferred to .5(k); .5(k)
        // ruled with IEEE 1800-2017 §22.5.1 :370-372 (a plain string in macro text is opaque) and
        // Verilator 5.050, so the default reading keeps the literal as written, pastes included.
        // Icarus 13.0's "fooxbar" is `--sv-plain-string-substitution all`, and OpenTitan's
        // paste-delimited idiom is `paste_delimited` (the_verible_paste_string_shape_under_each_reading).
        let text = expand_text(
            "svpp_paste_order_plain_string",
            "`define FOO(arg) \"foo``arg``bar\"\nmodule t; initial $display(`FOO(x)); endmodule\n",
        );
        assert!(text.contains("$display(\"foo``arg``bar\");"), "{text}");
    }

    // ── SVPP-EXPANSION.5(j) — the `"…`" string region: "the expansion shall include the quotation
    //    mark, substitution of actual arguments, and expansions of embedded macros" (§22.5.1) ──────

    #[test]
    fn a_macro_usage_embedded_in_a_string_region_is_expanded() {
        // through an ACTUAL: iverilog ivltests/sv_macro3a.v (its tracked gold/sv_macro3.gold reads
        // my_prefix_my_suffix twice; Verilator 5.050 and Icarus 13.0 read the same); in the BODY:
        // the Surelog PreprocString shape with its -DMEM_ROOT=test supplied — Surelog's own golden
        // log holds the StringLiteral "test/rom.mem`message", the bare function-like `message kept
        let text = expand_text(
            "svpp_region_embedded_usage",
            "`define PREFIX my_prefix\n`define SUFFIX my_suffix\n`define name1 `PREFIX``_```SUFFIX\n`define name2(p,s) p``_``s\n`define stringify(text) `\"text`\"\n`define MEM_ROOT test\n`define message(TOTO) TATA\n`define MAKE_MEM_PATH(filename) `\"`MEM_ROOT/filename`message`\"\nA `stringify(`name1)\nB `stringify(`name2(`PREFIX, `SUFFIX))\nC `MAKE_MEM_PATH(rom.mem)\n",
        );
        assert!(text.contains("A \"my_prefix_my_suffix\"\n"), "{text}");
        assert!(text.contains("B \"my_prefix_my_suffix\"\n"), "{text}");
        assert!(text.contains("C \"test/rom.mem`message\"\n"), "{text}");
    }

    #[test]
    fn an_object_macros_string_region_is_rendered() {
        // verilator t_preproc_strify_join.v, its .out verbatim: "foo-bar-qux" / ""foo-bar-qux"" / "";
        // OpenTitan otbn_ssctrl_if.sv: `define SSCTRL_PATH `"`SSCTRL_HPATH`" — an object macro whose
        // region embeds a usage, refused by the parser inside `{`SSCTRL_PATH, ".", signal}` until now
        let text = expand_text(
            "svpp_region_object_macro",
            "`define FOO foo\n`define BAR bar\n`define QUX qux\n`define STRIFY `\"`FOO``-```BAR``-```QUX```\"\n`define NESTED_STRIFY `\"`STRIFY```\"\n`define EMPTY\n`define EMPTY_STRIFY `\"`EMPTY```\"\n`define SSCTRL_HPATH tb.dut.u_otbn_core.u_otbn_start_stop_control\n`define SSCTRL_PATH  `\"`SSCTRL_HPATH`\"\n`STRIFY\n`NESTED_STRIFY\n`EMPTY_STRIFY\nreturn {`SSCTRL_PATH, \".\", signal};\n",
        );
        assert!(
            text.contains("\"foo-bar-qux\"\n\"\"foo-bar-qux\"\"\n\"\"\n"),
            "{text}"
        );
        assert!(
            text.contains(
                "return {\"tb.dut.u_otbn_core.u_otbn_start_stop_control\", \".\", signal};"
            ),
            "{text}"
        );
    }

    #[test]
    fn an_escaped_quote_in_a_string_region_renders_as_a_quote() {
        // IEEE 1800-2017 §22.5.1's own example (section-22-compiler-directives.md:376-379):
        // `define msg(x,y) `"x: `\`"y`\`"`" ... expands to $display("left side: \"right side\"");
        // before this slice the close finder matched the escaped quote's last two bytes and the
        // engine emitted "left side: `\"right side`\""; an object macro's region the same
        let text = expand_text(
            "svpp_region_escaped_quote",
            "`define msg(x,y) `\"x: `\\`\"y`\\`\"`\"\n`define obj `\"obj `\\`\"q`\\`\" end`\"\n$display(`msg(left side,right side));\n$display(`obj);\n",
        );
        assert!(
            text.contains("$display(\"left side: \\\"right side\\\"\");"),
            "{text}"
        );
        assert!(text.contains("$display(\"obj \\\"q\\\" end\");"), "{text}");
    }

    #[test]
    fn a_string_region_in_a_source_line_stays_verbatim() {
        // §22.5.1 defines `" for MACRO text; sv2v test/lex/string_macro.sv:9 ($display(`"`FOO`");) is
        // pinned must-reject on exactly that (SVPP-EXPANSION.6c.2). Verilator 5.050 renders a
        // source-line region ("standalone") and Icarus 13.0 leaves it — recorded, not followed
        // `SVPP-EXPANSION.6f`: under the default policy the stage REFUSES the stray `" (the pin's verdict,
        // now at the expansion stage); the region's bytes are read under `keep` to hold the rendering rule
        let dir = create_repo_scratch_dir("svpp_region_source_line_refused");
        let input = dir.join("top.sv");
        fs::write(&input, "`define FOO BAR\nmodule t; initial $display(`\"standalone `FOO`\"); endmodule\n").expect("write");
        let err = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect_err("a macro-string token outside macro text is refused");
        assert!(format!("{err:#}").contains("outside macro text"), "{err:#}");
        let text = expand_text_keep(
            "svpp_region_source_line",
            "`define FOO BAR\nmodule t; initial $display(`\"standalone `FOO`\"); endmodule\n",
        );
        assert!(text.contains("$display(`\"standalone BAR`\");"), "{text}");
    }

    #[test]
    fn an_undefined_or_argumentless_usage_in_a_region_stays_verbatim() {
        // Surelog tests/PreprocString/dut.sv as the lane sees it (no MEM_ROOT): the bytes it produced
        // before this slice, and Surelog's golden shape for the bare `message — an undefined name and
        // a function-like name with no argument list are the string's, backtick and all
        let text = expand_text(
            "svpp_region_undefined_usage",
            "`define message(TOTO) TATA\n`define MAKE_MEM_PATH(filename) `\"`MEM_ROOT/filename`message`\"\n$display(`MAKE_MEM_PATH(rom.mem));\n",
        );
        assert!(
            text.contains("$display(\"`MEM_ROOT/rom.mem`message\");"),
            "{text}"
        );
    }

    #[test]
    fn pastes_inside_a_string_region_still_join() {
        // sv2v test/lex/macro_string.sv (all three tools agree today, the bytes must not move) and the
        // UVM shape the region renderer was first written for: `"ARG[+]`" → "abstractions[+]"
        let text = expand_text(
            "svpp_region_pastes",
            "`define PRINT(str, num) $display(`\"StrIs``str NumIs``num`\");\n`define Q(ARG) push(`\"ARG[+]`\")\n`PRINT(FOO, 0)\n`Q(abstractions)\n",
        );
        assert!(text.contains("$display(\"StrIsFOO NumIs0\");"), "{text}");
        assert!(text.contains("push(\"abstractions[+]\")"), "{text}");
    }

    #[test]
    fn a_region_arriving_through_an_actual_is_rendered_where_it_lands() {
        // verilator t_preproc.v "Stringify in nested macro", its .out: `quux(`bar(`a,`a)) →
        // string boo = "test x,y x,y"; — the region `"x`" is quux's, its actual reaches qux, and the
        // usages inside it are expanded where the region is rendered
        let text = expand_text(
            "svpp_region_nested_actual",
            "`define foo test\n`define a x,y\n`define bar(a, b) test a b\n`define qux(x) string boo = x;\n`define quux(x) `qux(`\"x`\")\n`quux(`foo)\n`quux(`bar(`a,`a))\n",
        );
        assert!(text.contains("string boo = \"test\";"), "{text}");
        assert!(text.contains("string boo = \"test x,y x,y\";"), "{text}");
    }

    #[test]
    fn an_empty_argument_list_is_one_empty_actual_when_the_macro_has_formals() {
        // verilator t_pp_display.v:35 `msg(`thru(),) — Verilator 5.050 and Icarus 13.0 both read
        // `thru() as ONE empty actual (`$display(": \"\"")`); before this the engine read `()` as
        // ZERO actuals and refused it as MISSING. A formal-less macro's `()` is still its empty list
        // (`foo8`), and a second formal without a default is still MISSING (`D`, .5(g)).
        let text = expand_text(
            "svpp_empty_actual",
            "`define thru(x) x\n`define msg(x,y) `\"x: `\\`\"y`\\`\"`\"\n`define foo8() eight\nA [`thru()]\nB `msg(`thru(),)\nC `foo8()\n",
        );
        assert!(text.contains("A []\n"), "{text}");
        assert!(text.contains("B \": \\\"\\\"\"\n"), "{text}");
        assert!(text.contains("C eight\n"), "{text}");
        let dir = create_repo_scratch_dir("svpp_empty_actual_missing");
        let input = dir.join("top.sv");
        fs::write(&input, "`define D(x, y) x y\n`D()\n").expect("write");
        let err = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default();
        assert!(err.contains("E_SVPP_MISSING_MACRO_ARGUMENT"), "{err}");
        assert!(err.contains("formal 'y'"), "{err}");
    }

    #[test]
    fn a_multi_line_expansion_inside_a_string_region_is_one_line() {
        // verilator t_preproc.v: `define twoline first \ / second, then `msg(twoline, `twoline) —
        // its .out reads $display("twoline: \"first        second\""): the continuation's newline is
        // a space inside the literal (Icarus 13.0 leaves a raw newline, which the parser refuses —
        // rust/target-free proof: `$display("a\nb")` is a rejection, `"a \\\n b"` an accept)
        let text = expand_text(
            "svpp_region_multiline",
            "`define msg(x,y) `\"x: `\\`\"y`\\`\"`\"\n`define twoline first \\\n      second\n$display(`msg(twoline, `twoline));\n",
        );
        assert!(
            text.contains("$display(\"twoline: \\\"first        second\\\"\");"),
            "{text}"
        );
        assert!(!text.contains("first \n"), "{text}");
    }

    // ── SVPP-EXPANSION.5(k) — a formal inside a plain "…" string in macro text ─────────────────

    /// Expand one source text under a plain-string policy and return the expanded text.
    fn expand_text_with(
        prefix: &str,
        source: &str,
        policy: PlainStringSubstitutionPolicy,
    ) -> String {
        let dir = create_repo_scratch_dir(prefix);
        let input = dir.join("top.sv");
        fs::write(&input, source).expect("write");
        let config = SvPreprocessorConfig {
            plain_string_substitution: policy,
            ..SvPreprocessorConfig::default()
        };
        preprocess_systemverilog_file(&input, &config)
            .expect("expand")
            .text
    }

    #[test]
    fn a_formal_inside_a_plain_string_is_the_strings_by_default() {
        // IEEE 1800-2017 §22.5.1's own example (section-22-compiler-directives.md:340-352):
        // `define H(x) "Hello, x" … $display(`H(world)); … will print: Hello, x — and the two lines
        // before it: a `HI inside a plain string is not a usage. Verilator 5.050 reads all three so;
        // Icarus 13.0 and this engine (until .5(k)) printed Hello, world. sv-tests
        // 22.5.1--define-expansion_24.sv and ispras 22.05.01_07.sv carry the example verbatim.
        let text = expand_text(
            "svpp_plain_string_default",
            "`define HI Hello\n`define LO \"`HI, world\"\n`define H(x) \"Hello, x\"\ninitial begin\n$display(\"`HI, world\");\n$display(`LO);\n$display(`H(world));\nend\n",
        );
        assert!(
            text.contains(
                "$display(\"`HI, world\");\n$display(\"`HI, world\");\n$display(\"Hello, x\");"
            ),
            "{text}"
        );
    }

    #[test]
    fn prose_that_names_a_formal_is_no_longer_mangled() {
        // iverilog ivltests/sv_type_identifier_*.v: `define check(value, expected, error) …
        // $display("  expected %0h, got %0h", expected, value) — the word inside the string is prose,
        // the actual belongs after it; verilator t_*: "got='h%x exp='h%x" with formals exp/got;
        // OpenTitan pwrmgr_base_vseq.sv: "in %0d fast clock cycles" with a formal `cycles`. 66 of the
        // corpus's 72 in-string formal sites are this shape, every one substituted into its actual.
        let text = expand_text(
            "svpp_plain_string_prose",
            "`define check(value, expected) $display(\"  expected %0h, got %0h\", expected, value);\n`check(a, 8'h5)\n",
        );
        assert!(
            text.contains("$display(\"  expected %0h, got %0h\", 8'h5, a);"),
            "{text}"
        );
    }

    #[test]
    fn a_paste_delimited_formal_inside_a_plain_string_is_the_idiom_the_knob_admits() {
        // OpenTitan alert_handler_alert_accum_saturation_vseq.sv: `define CLASS_CNT_PATH(i)
        // string class_``i``_path_0 = "tb.dut.gen_classes[``i``].u_accu.u_prim_count.cnt_q[0]"; —
        // design code written for commercial simulators. Under the standard the literal is opaque
        // (Verilator 5.050: the pastes stay inside the string); under paste_delimited the marked
        // formal is substituted and the pastes removed, a bare word elsewhere in the same string
        // stays prose; under all (Icarus's reading) every formal is substituted.
        let src = "`define P(i) string class_``i``_path = \"tb.gen[``i``].cnt i\";\n`P(3)\n";
        let standard = expand_text_with(
            "svpp_plain_string_standard",
            src,
            PlainStringSubstitutionPolicy::Standard,
        );
        assert!(
            standard.contains("string class_3_path = \"tb.gen[``i``].cnt i\";"),
            "{standard}"
        );
        let delimited = expand_text_with(
            "svpp_plain_string_delimited",
            src,
            PlainStringSubstitutionPolicy::PasteDelimited,
        );
        assert!(
            delimited.contains("string class_3_path = \"tb.gen[3].cnt i\";"),
            "{delimited}"
        );
        let all = expand_text_with(
            "svpp_plain_string_all",
            src,
            PlainStringSubstitutionPolicy::All,
        );
        assert!(
            all.contains("string class_3_path = \"tb.gen[3].cnt 3\";"),
            "{all}"
        );
    }

    #[test]
    fn the_verible_paste_string_shape_under_each_reading() {
        // verible lint/testdata/macro_string_concatenation.sv: `define FOO(arg) "foo``arg``bar" —
        // Verilator 5.050 keeps "foo``arg``bar" (the standard), Icarus 13.0 reads "fooxbar" (all)
        let src = "`define FOO(arg) \"foo``arg``bar\"\n$display(`FOO(x));\n";
        assert!(
            expand_text_with(
                "svpp_verible_standard",
                src,
                PlainStringSubstitutionPolicy::Standard
            )
            .contains("$display(\"foo``arg``bar\");")
        );
        assert!(
            expand_text_with(
                "svpp_verible_delimited",
                src,
                PlainStringSubstitutionPolicy::PasteDelimited
            )
            .contains("$display(\"fooxbar\");")
        );
        assert!(
            expand_text_with("svpp_verible_all", src, PlainStringSubstitutionPolicy::All)
                .contains("$display(\"fooxbar\");")
        );
    }

    #[test]
    fn the_policy_parses_its_three_spellings_and_refuses_others() {
        assert_eq!(
            PlainStringSubstitutionPolicy::parse("standard"),
            Some(PlainStringSubstitutionPolicy::Standard)
        );
        assert_eq!(
            PlainStringSubstitutionPolicy::parse("paste_delimited"),
            Some(PlainStringSubstitutionPolicy::PasteDelimited)
        );
        assert_eq!(
            PlainStringSubstitutionPolicy::parse("all"),
            Some(PlainStringSubstitutionPolicy::All)
        );
        assert_eq!(PlainStringSubstitutionPolicy::parse("sometimes"), None);
        assert_eq!(
            PlainStringSubstitutionPolicy::default(),
            PlainStringSubstitutionPolicy::Standard
        );
    }

    #[test]
    fn an_escaped_character_inside_a_plain_string_closes_it_where_the_author_did() {
        // verilator t_alias_unsup.v's `checkh(gotv, expv): `$write("… got=%0x exp=%0x\n", `__FILE__,
        // `__LINE__, (gotv), (expv), "gotv", "expv")` — the `\n` escapes the `n`, the `"` after it
        // closes the string, and the formals AFTER it are substituted; the words INSIDE the two
        // trailing strings are not. Found by the whole-corpus diff of .5(k): an escape whose escaped
        // character started an identifier was never cleared, so the string never closed.
        let text = expand_text(
            "svpp_plain_string_escape",
            "`define checkh(gotv, expv) do if ((gotv) !== (expv)) $write(\"got=%0x exp=%0x\\n\", (gotv), (expv), \"gotv\", \"expv\"); while (0)\n`checkh(crc, 64'h1);\n",
        );
        assert!(
            text.contains("do if ((crc) !== (64'h1)) $write(\"got=%0x exp=%0x\\n\", (crc), (64'h1), \"gotv\", \"expv\"); while (0);"),
            "{text}"
        );
    }

    #[test]
    fn edition_2023_block_comment_in_macro_text_is_not_substituted() {
        // `SVPP-EXPANSION.5`(p)(ii): IEEE 1800-2023 §22.5.1 — "If a one-line comment or block comment (see 5.4)
        // is included in the text, then the comment shall not become part of the substituted text"
        // (docs/systemverilog/2023/txt/section-22-compiler-directives.txt:175-176). IEEE 1800-2017 says it of
        // one-line comments only (docs/systemverilog/2017/txt/section-22-compiler-directives.txt:151-152), so
        // the block comment stays in the 2017 reading (`.6b.2`(iii)).
        let dir = create_repo_scratch_dir("svpp_edition_block_comment");
        let input = dir.join("top.sv");
        fs::write(&input, "`define Y(a) a /* gone? */ + 1\nwire [`Y(3):0] w;\n").expect("write");
        let lrm_2017 = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("the 2017 reading");
        assert!(
            lrm_2017.text.contains("/* gone? */"),
            "the 2017 reading keeps a block comment in the substituted text:\n{}",
            lrm_2017.text
        );
        let config_2023 = SvPreprocessorConfig {
            parse_profile: Some("sv_2023".to_string()),
            ..SvPreprocessorConfig::default()
        };
        let lrm_2023 =
            preprocess_systemverilog_file(&input, &config_2023).expect("the 2023 reading");
        assert!(
            !lrm_2023.text.contains("gone?"),
            "the 2023 reading does not substitute a block comment:\n{}",
            lrm_2023.text
        );
        assert!(
            lrm_2023.text.contains("wire [3") && lrm_2023.text.contains("+ 1:0] w;"),
            "{}",
            lrm_2023.text
        );
    }

    #[test]
    fn edition_2023_multi_line_block_comment_continues_the_macro_text() {
        // `SVPP-EXPANSION.5`(p)(i): IEEE 1800-2023 §22.5.1 — "A newline character that is not contained in a
        // triple-quoted string literal or in a block comment and is not preceded by a backslash shall end the
        // macro text" (:150-151); "If the macro text contains a multi-line block comment, the newline characters
        // in the block comment are not required to be preceded by a backslash" (:177-179).
        let dir = create_repo_scratch_dir("svpp_edition_multiline_comment");
        let input = dir.join("top.sv");
        fs::write(&input, "`define X 1 /* multi\nline */ + 2\nwire [`X:0] w;\n").expect("write");
        let config_2023 = SvPreprocessorConfig {
            parse_profile: Some("sv_2023".to_string()),
            ..SvPreprocessorConfig::default()
        };
        let out = preprocess_systemverilog_file(&input, &config_2023)
            .expect("the 2023 reading takes the define through its comment");
        assert!(
            !out.text.contains("line */"),
            "the comment's tail is not source text:\n{}",
            out.text
        );
        assert!(
            out.text.contains("wire [1") && out.text.contains("+ 2:0] w;"),
            "{}",
            out.text
        );
    }

    #[test]
    fn edition_2017_block_comment_left_open_by_a_define_is_refused_naming_the_clause() {
        // `SVPP-EXPANSION.5`(p)(i), the other edition: IEEE 1800-2017 §22.5.1 — "The first newline character not
        // preceded by a backslash shall end the macro text" (:135-136), and "The text specified for macro text
        // shall not be split across the following lexical tokens: — Comments …" (:230-232). Before this the tail
        // of the comment was emitted as source text with its opener consumed into the macro.
        let dir = create_repo_scratch_dir("svpp_edition_comment_left_open");
        let input = dir.join("top.sv");
        fs::write(&input, "`define X 1 /* multi\nline */ + 2\nwire [`X:0] w;\n").expect("write");
        let err = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect_err("a comment split across the end of the macro text is refused under 2017");
        let text = format!("{:#}", err);
        assert!(
            text.contains("22.5.1") && text.contains("split") && text.contains("sv_2023"),
            "{}",
            text
        );
    }

    #[test]
    fn edition_2023_ifdef_argument_macro_usage_is_expanded() {
        // `SVPP-EXPANSION.5`(p)(iii): IEEE 1800-2023 §22.2 — "Macro expansion shall occur within a compiler
        // directive" (:68). verilator t_preproc.v:423, :426 (its own comment: "It's unclear if the spec allows
        // this" — 1800-2023 settles it). 1800-2017 has no such sentence and Syntax 22-6 takes a
        // text_macro_identifier (:379-380): the argument is read as written.
        let dir = create_repo_scratch_dir("svpp_edition_ifdef_usage");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define NODS_INDIRECT(x) x\n`define NODS_DEFINED\n`ifndef `NODS_INDIRECT(NODS_DEFINED)\nwire indirect_failed;\n`endif\n`ifdef `NODS_INDIRECT(NODS_UNDEFINED)\nwire indirect2_failed;\n`endif\nwire ok;\n",
        )
        .expect("write");
        let config_2023 = SvPreprocessorConfig {
            parse_profile: Some("sv_2023".to_string()),
            ..SvPreprocessorConfig::default()
        };
        let out =
            preprocess_systemverilog_file(&input, &config_2023).expect("the 2023 reading");
        assert!(
            !out.text.contains("indirect_failed") && !out.text.contains("indirect2_failed"),
            "{}",
            out.text
        );
        assert!(out.text.contains("wire ok;"), "{}", out.text);
        let lrm_2017 = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("the 2017 reading");
        assert!(
            lrm_2017.text.contains("wire indirect_failed;"),
            "the 2017 reading takes the argument as written:\n{}",
            lrm_2017.text
        );
    }

    #[test]
    fn edition_2023_elsif_argument_macro_usage_is_expanded() {
        // `SVPP-EXPANSION.5`(p)(iii), the `elsif arm of the same sentence (1800-2023 §22.2, :68).
        let dir = create_repo_scratch_dir("svpp_edition_elsif_usage");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define ID(x) x\n`define B\n`ifdef A\nwire a;\n`elsif `ID(B)\nwire b;\n`endif\nwire ok;\n",
        )
        .expect("write");
        let config_2023 = SvPreprocessorConfig {
            parse_profile: Some("sv_2023".to_string()),
            ..SvPreprocessorConfig::default()
        };
        let out =
            preprocess_systemverilog_file(&input, &config_2023).expect("the 2023 reading");
        assert!(out.text.contains("wire b;"), "{}", out.text);
        assert!(!out.text.contains("wire a;"), "{}", out.text);
    }

    #[test]
    fn edition_2023_undef_argument_macro_usage_is_expanded() {
        // `SVPP-EXPANSION.5`(p)(iii), the `undef arm of the same sentence (1800-2023 §22.2, :68).
        let dir = create_repo_scratch_dir("svpp_edition_undef_usage");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define NAME FOO\n`define FOO 1\n`undef `NAME\n`ifdef FOO\nwire still;\n`endif\nwire ok;\n",
        )
        .expect("write");
        let config_2023 = SvPreprocessorConfig {
            parse_profile: Some("sv_2023".to_string()),
            ..SvPreprocessorConfig::default()
        };
        let out =
            preprocess_systemverilog_file(&input, &config_2023).expect("the 2023 reading");
        assert!(!out.text.contains("wire still;"), "{}", out.text);
        assert!(out.text.contains("wire ok;"), "{}", out.text);
    }

    #[test]
    fn a_define_whose_text_is_split_across_a_string_literal_is_refused() {
        // `SVPP-EXPANSION.5`(s): IEEE 1800-2017 §22.5.1 — "The text specified for macro text shall not be split
        // across the following lexical tokens: Comments, Numbers, String literals, Identifiers, Keywords, Operators"
        // and "The following is illegal syntax because it is split across a string: `define first_half "start of
        // string / $display(`first_half end of string");" (docs/systemverilog/2017/md/section-22-compiler-directives.md
        // :275-281, :295-297; 1800-2023 :307, :320-322; 1364-2005 §19.3.1 :172, :192, :214-215). Before this the
        // definition was accepted and the use site glued one string: $display("start of string end of string");
        // (sv-tests 22.5.1--define-expansion_21.sv, keyed should_fail, ACCEPTED under the pipeline).
        for profile in [None, Some("sv_2017"), Some("sv_2023"), Some("verilog_2005")] {
            let dir = create_repo_scratch_dir("svpp_split_string");
            let input = dir.join("top.sv");
            fs::write(
                &input,
                "`define first_half \"start of string\nmodule top ();\ninitial $display(`first_half end of string\");\nendmodule\n",
            )
            .expect("write");
            let config = SvPreprocessorConfig { parse_profile: profile.map(str::to_string), ..SvPreprocessorConfig::default() };
            let err = preprocess_systemverilog_file(&input, &config)
                .expect_err("macro text split across a string literal must be refused");
            let text = format!("{err:#}");
            assert!(text.contains("invalid `define directive"), "{text}");
            assert!(text.contains("split across a string literal") && text.contains("22.5.1"), "{text}");
        }
    }

    #[test]
    fn a_string_literal_spanning_a_continuation_is_not_split() {
        // `SVPP-EXPANSION.5`(s) control: the backslash-newline continues the macro text INSIDE the string, so the
        // literal is complete by the end of the text (1800-2023 §22.5.1 :176-177 names the exception outright; the
        // 2017 text's continuation rule says the same of the joined text). A `"` inside a comment in the macro text
        // is not a string delimiter either.
        let dir = create_repo_scratch_dir("svpp_string_continuation");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define MSG \"two \\\nlines\"\n`define Q 1 // a \" in a comment\n`define R 2 /* a \" in a block comment */\nmodule top ();\ninitial $display(`MSG, `Q, `R);\nendmodule\n",
        )
        .expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("a string spanning a continuation and quotes inside comments are not splits");
        assert!(out.text.contains("$display("), "{}", out.text);
        assert!(!out.text.contains('`'), "every usage expanded:\n{}", out.text);
    }

    #[test]
    fn a_compiler_directive_cannot_be_redefined_as_a_macro_name() {
        // `SVPP-EXPANSION.5`(t): IEEE 1800-2017 §22.5.1 — "All compiler directives shall be considered predefined
        // macro names; it shall be illegal to redefine a compiler directive as a macro name"
        // (docs/systemverilog/2017/md/section-22-compiler-directives.md:139-140; 1800-2023 :152-153; 1364-2005
        // §19.3.1 :100-101 and :229 "Text macro names may not be the same as compiler directive keywords"). Before
        // this `define define "illegal" and `define include 1 were accepted (sv-tests 22.5.1--define-expansion_23.sv,
        // keyed should_fail, ACCEPTED under the pipeline).
        for (name, profile) in [("define", None), ("include", Some("sv_2017")), ("celldefine", Some("sv_2023")),
                                ("timescale", Some("verilog_2005")), ("__FILE__", Some("sv_2017"))] {
            let dir = create_repo_scratch_dir("svpp_redefine_directive");
            let input = dir.join("top.sv");
            fs::write(&input, format!("`define {name} 1\nmodule top (); endmodule\n")).expect("write");
            let config = SvPreprocessorConfig { parse_profile: profile.map(str::to_string), ..SvPreprocessorConfig::default() };
            let err = preprocess_systemverilog_file(&input, &config)
                .expect_err("a compiler directive redefined as a macro name must be refused");
            let text = format!("{err:#}");
            assert!(text.contains("invalid `define directive"), "{name}: {text}");
            assert!(text.contains("compiler directive") && text.contains("22.5.1"), "{name}: {text}");
        }
    }

    #[test]
    fn the_directive_set_is_the_editions_own() {
        // `SVPP-EXPANSION.5`(t): `__FILE__`, `__LINE__` and `undefineall` are IEEE 1800 directives (§22.13, §22.5.3)
        // and not IEEE 1364-2005's (§19 lists neither), so under the verilog_2005 profile they are ordinary macro
        // names — the set is the edition's own list, not a union.
        let dir = create_repo_scratch_dir("svpp_directive_set_edition");
        let input = dir.join("top.sv");
        fs::write(&input, "`define __LINE__ 0\n`define undefineall 2\nmodule top (); initial $display(`__LINE__, `undefineall); endmodule\n")
            .expect("write");
        let v2005 = SvPreprocessorConfig { parse_profile: Some("verilog_2005".to_string()), ..SvPreprocessorConfig::default() };
        let out = preprocess_systemverilog_file(&input, &v2005).expect("not directives of 1364-2005: ordinary macro names");
        assert!(out.text.contains("$display(0, 2)"), "{}", out.text);
        let sv = SvPreprocessorConfig { parse_profile: Some("sv_2017".to_string()), ..SvPreprocessorConfig::default() };
        let err = preprocess_systemverilog_file(&input, &sv).expect_err("`__LINE__ is an IEEE 1800 directive");
        assert!(format!("{err:#}").contains("compiler directive"), "{err:#}");
    }

    #[test]
    fn a_macro_named_like_a_directive_with_a_suffix_is_an_ordinary_name() {
        // `SVPP-EXPANSION.5`(t) control: the set is matched as a whole name — `define_me`, `includes`, `line1` are
        // not directives (the corpus writes `` `define DEFINE_MACROS `` and `` `define LINE_WIDTH `` shapes freely).
        let dir = create_repo_scratch_dir("svpp_directive_like_names");
        let input = dir.join("top.sv");
        fs::write(&input, "`define define_me 1\n`define includes 2\n`define line1 3\n`define DEFINE 4\nmodule top (); initial $display(`define_me, `includes, `line1, `DEFINE); endmodule\n")
            .expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default()).expect("ordinary names");
        assert!(out.text.contains("$display(1, 2, 3, 4)"), "{}", out.text);
    }

    #[test]
    fn the_lrm_split_string_example_is_refused_and_a_closed_string_in_text_is_not() {
        // `SVPP-EXPANSION.5`(s) control: a complete string literal in macro text is fine; the refusal is for one the
        // text leaves OPEN at its end (the standard's own example), under every profile.
        let dir = create_repo_scratch_dir("svpp_closed_string");
        let input = dir.join("top.sv");
        fs::write(&input, "`define WHOLE \"a whole string\"\n`define TWO(a) $display(\"%s\", a)\nmodule top ();\ninitial `TWO(`WHOLE);\nendmodule\n")
            .expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default()).expect("closed strings");
        assert!(out.text.contains("$display(\"%s\", \"a whole string\")"), "{}", out.text);
    }

    #[test]
    fn a_compiler_directive_name_is_predefined_to_the_conditionals() {
        // `SVPP-EXPANSION.5`(t): the SAME sentence — "All compiler directives shall be considered predefined macro
        // names" — makes `` `ifdef __FILE__ `` true and `` `ifndef __FILE__ `` false, so a guarded redefinition is
        // skipped rather than refused (ispras ieee-1800-2012/22/22.13_01.sv, keyed must_accept: `ifndef __FILE__ /
        // `define __FILE__ "unknown" / `endif). Before this a directive name was "not defined" to the conditionals.
        let dir = create_repo_scratch_dir("svpp_directive_predefined");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`ifndef __FILE__\n`define __FILE__ \"unknown\"\n`endif\n`ifdef define\nmodule define_is_predefined; endmodule\n`endif\n`ifndef include\nmodule include_not_defined; endmodule\n`endif\n",
        )
        .expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("the guarded redefinition is skipped: the directive is predefined");
        assert!(out.text.contains("define_is_predefined"), "{}", out.text);
        assert!(!out.text.contains("include_not_defined"), "{}", out.text);
        // under a Verilog profile `__FILE__` is not a directive: the guard is TRUE and the (legal) definition runs
        let v2005 = SvPreprocessorConfig { parse_profile: Some("verilog_2005".to_string()), ..SvPreprocessorConfig::default() };
        fs::write(&input, "`ifndef __FILE__\n`define __FILE__ \"unknown\"\n`endif\nmodule t; initial $display(`__FILE__); endmodule\n").expect("write");
        let out = preprocess_systemverilog_file(&input, &v2005).expect("an ordinary macro under 1364-2005");
        assert!(out.text.contains("$display(\"unknown\")"), "{}", out.text);
    }

    #[test]
    fn a_continuation_inside_a_string_literal_omits_both_characters_under_2023() {
        // `SVPP-EXPANSION.5`(u): IEEE 1800-2023 §22.5.1 — "An exception to the previous sentence is that if the
        // backslash-newline character sequence occurs in the middle of a string literal, both the backslash and the
        // newline characters are omitted in the expanded macro (see 5.9)" (docs/systemverilog/2023/md/
        // section-22-compiler-directives.md:182-184). 1800-2017 has no such sentence: its continuation "shall be
        // replaced in the expanded macro with a newline character" (docs/systemverilog/2017/md/
        // section-22-compiler-directives.md:171-175), so there the literal spans two lines and the parser rejects it
        // (§5.9: a string literal is a single line). The edition twin pins hold both readings.
        let dir = create_repo_scratch_dir("svpp_string_continuation_2023");
        let input = dir.join("top.sv");
        fs::write(&input, "`define MSG \"two \\\nlines\"\nmodule top ();\ninitial $display(`MSG);\nendmodule\n").expect("write");
        let sv2023 = SvPreprocessorConfig { parse_profile: Some("sv_2023".to_string()), ..SvPreprocessorConfig::default() };
        let out = preprocess_systemverilog_file(&input, &sv2023).expect("2023: the string continues");
        assert!(out.text.contains("$display(\"two lines\")"), "2023 omits both characters:\n{}", out.text);
        let sv2017 = SvPreprocessorConfig { parse_profile: Some("sv_2017".to_string()), ..SvPreprocessorConfig::default() };
        let out = preprocess_systemverilog_file(&input, &sv2017).expect("2017: the continuation is a newline");
        assert!(out.text.contains("$display(\"two \nlines\")"), "2017 keeps the newline:\n{}", out.text);
    }

    #[test]
    fn a_default_argument_may_hold_parentheses_and_commas_inside_a_string_literal() {
        // `SVPP-EXPANSION.5`(v): the formal list closes at the `)` that matches its `(` OUTSIDE any string literal
        // (IEEE 1800-2017 §22.5.1 Syntax 22-2: `default_text`; §5.9: a string literal is one token). The closing
        // scan read `(a="==)",b="((((",c=() )` as closing inside "==)" and handed the rest to the macro text — a
        // mis-split since the scanner's first commit, surfaced by `.5`(s)'s check on the text it left behind
        // (verilator t_preproc_def09.v:56). The sibling scanners (`define_formal_list_unclosed`,
        // `split_macro_parameter_tokens`) were already string-aware.
        let dir = create_repo_scratch_dir("svpp_default_arg_string_parens");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define MACROQUOTE(a=\"==)\",b=\"((((\",c=() ) [a b c]\nmodule top ();\ninitial $display(\"%s\", `MACROQUOTE());\nendmodule\n",
        )
        .expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("the list closes outside the string literals");
        assert!(out.text.contains("[\"==)\" \"((((\" ()]"), "the three defaults substituted:\n{}", out.text);
    }

    #[test]
    fn an_unresolved_macro_usage_in_the_expanded_text_is_refused() {
        // `SVPP-EXPANSION.6f`: a backtick that SURVIVES expansion outside a comment or string is not text the parser
        // may judge — an undefined macro has no derivation in IEEE 1800-2017 §22.5.1 ("Once a text macro name has
        // been defined, it can be used anywhere …": no definition, no substitution), and the SV grammar's raw-lane
        // residue (`compiler_directive` → `opaque`) would otherwise ACCEPT a top-level `` `not_defined `` line, which
        // is how 38 pipeline rows read `pass` over text the stage never resolved (verilator t_preproc_nodef_bad.v,
        // sv2v error/undefined_macro.sv, 25 riscv-dv instruction files whose macros live in a package that fails).
        let dir = create_repo_scratch_dir("svpp_unresolved_usage");
        let input = dir.join("top.sv");
        fs::write(&input, "`not_defined\nmodule t;\nendmodule\n").expect("write");
        let err = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect_err("an undefined usage left in the output is refused");
        let text = format!("{err:#}");
        assert!(text.contains("`not_defined") && text.contains("neither a defined macro nor a compiler directive"), "{text}");
        assert!(text.contains("top.sv:1"), "the refusal names the source origin:\n{text}");
        // the `keep` policy is the old passthrough, for a consumer that resolves the rest itself
        let keep = SvPreprocessorConfig { undefined_macro_policy: UndefinedMacroPolicy::Keep, ..SvPreprocessorConfig::default() };
        let out = preprocess_systemverilog_file(&input, &keep).expect("keep: passthrough");
        assert!(out.text.contains("`not_defined"), "{}", out.text);
    }

    #[test]
    fn a_backtick_inside_a_string_or_comment_survives_expansion_legitimately() {
        // `SVPP-EXPANSION.6f` control: the scan skips string literals and both comment forms, and an LRM directive
        // the parser reads (`` `timescale ``, `` `default_nettype ``) is not an unresolved usage.
        let dir = create_repo_scratch_dir("svpp_backtick_in_string");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`timescale 1ns/1ps\n`default_nettype none\nmodule t;\n  // a `comment_word here\n  /* and `another */\n  initial $display(\"a `string_word\");\nendmodule\n`default_nettype wire\n",
        )
        .expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default()).expect("legitimate backticks");
        assert!(out.text.contains("`timescale 1ns/1ps") && out.text.contains("\"a `string_word\""), "{}", out.text);
    }

    #[test]
    fn a_function_like_macro_used_without_its_argument_list_is_refused() {
        // `SVPP-EXPANSION.6f`: IEEE 1800-2017 §22.5.1 — "For a macro with arguments, the parentheses are always required
        // in the macro call, even if all the arguments have defaults" (docs/systemverilog/2017/md/
        // section-22-compiler-directives.md:224-225). sv2v error/macro_unapplied.sv (`MACRO asdf) and
        // macro_unapplied_eof.sv (`MACRO at end of input) read `pass` before this: the usage was kept as written and
        // the grammar's opaque residue accepted the line.
        for body in ["`define MACRO(a) a\n`MACRO asdf\nmodule t; endmodule\n", "`define MACRO(a) a\n`MACRO\n"] {
            let dir = create_repo_scratch_dir("svpp_bare_function_like");
            let input = dir.join("top.sv");
            fs::write(&input, body).expect("write");
            let err = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
                .expect_err("a function-like macro without its parentheses is refused");
            let text = format!("{err:#}");
            assert!(text.contains("`MACRO") && text.contains("parentheses are always required"), "{text}");
        }
    }

    #[test]
    fn an_unterminated_macro_argument_list_is_refused() {
        // `SVPP-EXPANSION.6f`: Syntax 22-3 `text_macro_usage ::= `text_macro_identifier [ ( list_of_actual_arguments ) ]`
        // — the closing parenthesis is part of the usage; verilator t_preproc_eof3_bad.v (`FOO(1, at end of file) read
        // `pass` before this, the incomplete usage kept as written and accepted as an opaque directive line.
        let dir = create_repo_scratch_dir("svpp_unterminated_actuals");
        let input = dir.join("top.sv");
        fs::write(&input, "`define FOO(a,b)\n`FOO(1,\n").expect("write");
        let err = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect_err("an unterminated argument list is refused");
        let text = format!("{err:#}");
        assert!(text.contains("`FOO") && text.contains("argument list"), "{text}");
    }

    #[test]
    fn a_macro_string_token_outside_macro_text_is_refused() {
        // `SVPP-EXPANSION.6f`: `` `" ``, `` `\`" `` and `` `` `` have meaning only INSIDE macro text (§22.5.1); at the
        // top level they are no token of the language (sv2v error/string_directive.sv `"asdf `line`",
        // string_literal_backtick_eof.sv `"; friscv's testbenches write ``TCNAME in plain text).
        let dir = create_repo_scratch_dir("svpp_stray_macro_string");
        let input = dir.join("top.sv");
        fs::write(&input, "module t;\ninitial $display(`\"asdf`\");\nendmodule\n").expect("write");
        let err = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect_err("a macro-string token outside macro text is refused");
        let text = format!("{err:#}");
        assert!(text.contains("outside macro text"), "{text}");
    }

    #[test]
    fn a_backtick_that_begins_no_directive_and_no_usage_is_refused() {
        // `SVPP-EXPANSION.6h`: IEEE 1800-2017 §22.2 "All compiler directives are preceded by the (`) character" and
        // §22.5.1 Syntax 22-3 `text_macro_usage ::= `text_macro_identifier …` — the character has meaning only before
        // a NAME. Through 1.0.251 the scan tested only an identifier or a macro-string successor, so `` ` `` before a
        // line end, `` `+ `` and `` `9 `` reached the parser and the raw lane's opaque residue accepted them
        // (measured on the closed loop's generated batches: the initial batch carried `` `9\W `` and a bare `` ` ``).
        for (body, what) in [
            ("module t; endmodule\n`\n", "a line end"),
            ("module t; endmodule\n`+\n", "`+`"),
            ("module t; endmodule\n`9\n", "`9`"),
            ("module t; endmodule\n` timescale 1ns/1ps\n", "white space"),
            ("module t; endmodule\n`", "the end of the text"),
            ("module t; endmodule\n`\\\n", "a backslash that begins no escaped identifier"),
        ] {
            let dir = create_repo_scratch_dir("svpp_backtick_no_usage");
            let input = dir.join("top.sv");
            fs::write(&input, body).expect("write");
            let err = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
                .expect_err("a backtick that begins no directive and no usage is refused");
            let text = format!("{err:#}");
            assert!(
                text.contains("begins no compiler directive and no text-macro usage") && text.contains(what),
                "{body:?}: {text}"
            );
            assert!(text.contains("top.sv:2"), "the refusal names the source origin:\n{text}");
            let keep = SvPreprocessorConfig { undefined_macro_policy: UndefinedMacroPolicy::Keep, ..SvPreprocessorConfig::default() };
            let out = preprocess_systemverilog_file(&input, &keep).expect("keep: passthrough");
            assert!(out.text.contains('`'), "{}", out.text);
        }
    }

    #[test]
    fn an_escaped_identifier_carrying_a_backtick_is_one_token() {
        // `SVPP-EXPANSION.6h`: IEEE 1800-2017 §5.6.1 "Escaped identifiers shall start with the backslash character (\)
        // and end with white space … any of the printable ASCII characters" (the grammar: `/\\[!-~]+/`). Through
        // 1.0.251 the scan read the backtick INSIDE `\a`b` as a usage of `b` and refused a legal identifier
        // ("`b is neither a defined macro nor a compiler directive"); iverilog mangle.v writes
        // `\`~!-_=+\|[]{};:'",./<>?` and read `pass` only because `~` is not an identifier start.
        let dir = create_repo_scratch_dir("svpp_escaped_identifier_backtick");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "module m;\n  wire \\a`b ;\n  wire \\`~!-_=+\\|[]{};:'\",./<>? ;\n  assign \\a`b = \\`~!-_=+\\|[]{};:'\",./<>? ;\nendmodule\n",
        )
        .expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("an escaped identifier carrying a backtick is one token");
        assert!(out.text.contains("\\a`b ") && out.text.contains("\\`~!-_=+\\|[]{};:'\",./<>? "), "{}", out.text);
    }

    #[test]
    fn an_escaped_macro_usage_is_looked_up_by_its_name() {
        // `SVPP-EXPANSION.6h`: `` `\name `` is a text-macro usage whose name is an escaped identifier
        // (`SVPP-EXPANSION.5`(q)(ii): §22.5.1 Syntax 22-2 + A.9.3, `\esc` and `esc` one key). Through 1.0.251 an
        // UNDEFINED one was refused with the macro-string clause (`` `\ `` read as the `` `\`" `` prefix); it is the
        // undefined-name clause, and `` `\`" `` itself keeps its own. A defined one expands (iverilog
        // escaped_macro_name.v) and is never seen by the scan.
        let dir = create_repo_scratch_dir("svpp_escaped_macro_usage");
        let input = dir.join("top.sv");
        fs::write(&input, "module t; endmodule\n`\\not_defined\n").expect("write");
        let err = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect_err("an undefined escaped usage is refused");
        let text = format!("{err:#}");
        assert!(
            text.contains("`not_defined is neither a defined macro nor a compiler directive") && !text.contains("outside macro text"),
            "{text}"
        );
        fs::write(&input, "module t; endmodule\n`\\`\"\n").expect("write");
        let err = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect_err("the macro-string token keeps its clause");
        assert!(format!("{err:#}").contains("outside macro text"), "{err:#}");
        fs::write(&input, "`define \\esc 1\nmodule t; localparam X = `\\esc ; endmodule\n").expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default()).expect("a defined escaped usage expands");
        assert!(out.text.contains("localparam X = 1 ;"), "{}", out.text);
    }

    #[test]
    fn an_end_keywords_with_no_open_region_is_refused() {
        // `SVPP-EXPANSION.6f`: IEEE 1364-2005 §19.11 "Each `begin_keywords directive must be paired with an `end_keywords
        // directive"; IEEE 1800-2017 §22.14 speaks of "the matching `end_keywords directive" and a stacked pair. An
        // `end_keywords that closes nothing was a WARNING left for the parser, which reads the line as a directive
        // (verilator t_preproc_kwd_bad.v read `pass`). It is refused, naming the clause.
        let dir = create_repo_scratch_dir("svpp_unmatched_end_keywords");
        let input = dir.join("top.sv");
        fs::write(&input, "`end_keywords\nmodule t;\nendmodule\n").expect("write");
        let err = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect_err("an unmatched `end_keywords is refused");
        let text = format!("{err:#}");
        assert!(text.contains("`end_keywords") && text.contains("begin_keywords"), "{text}");
    }

    #[test]
    fn an_escaped_identifier_names_a_macro_in_both_spellings() {
        // `SVPP-EXPANSION.5`(q)(ii): IEEE 1800-2017 §22.5.1 Syntax 22-2 `text_macro_identifier ::= identifier`
        // (docs/systemverilog/2017/txt/section-22-compiler-directives.txt:131-132) and A.9.3 `identifier ::=
        // simple_identifier | escaped_identifier`; 1800-2023 §22.5.1 says it in a sentence — "The macro name may be
        // either a simple_identifier or an escaped_identifier (see 5.6)" (docs/systemverilog/2023/txt/
        // section-22-compiler-directives.txt:130). §5.6.1: "an escaped identifier \cpu3 is treated the same as a
        // nonescaped identifier cpu3" — so `\escaped` and `escaped` name ONE macro (iverilog
        // ivltests/escaped_macro_name.v:1-2, :11-14). Before this: `invalid `define directive … missing macro name`.
        let dir = create_repo_scratch_dir("svpp_escaped_macro_name");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define simple      \"simple name\"\n`define \\escaped    \"escaped name\"\nmodule t;\ninitial begin\n  $display(`simple);\n  $display(`\\simple );\n  $display(`escaped);\n  $display(`\\escaped );\nend\nendmodule\n",
        )
        .expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("an escaped identifier is a macro name");
        assert_eq!(
            out.text.matches("\"simple name\"").count(),
            2,
            "`simple and `\\simple name one macro:\n{}",
            out.text
        );
        assert_eq!(
            out.text.matches("\"escaped name\"").count(),
            2,
            "`escaped and `\\escaped name one macro:\n{}",
            out.text
        );
        assert!(!out.text.contains('`'), "every usage expanded:\n{}", out.text);
    }

    #[test]
    fn an_escaped_macro_name_takes_its_formals_after_one_space() {
        // `SVPP-EXPANSION.5`(q)(ii): IEEE 1800-2023 §22.5.1 — "If the macro name is an escaped_identifier, only the
        // single white space character terminating the identifier name (see 5.6.1) shall separate the opening
        // parenthesis from the macro name" (:166-167), with the standard's own example `\M@CRO3` (:268-275); in a
        // usage "If the text macro name is an escaped identifier, then white space shall be required" (:223).
        let dir = create_repo_scratch_dir("svpp_escaped_macro_formals");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define \\M@CRO3 (a=5, b=0, c=\"C\") $display(a,,b,,c);\nmodule t;\ninitial begin\n`\\M@CRO3 ( 1 )\n`\\M@CRO3 ( )\nend\nendmodule\n",
        )
        .expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("the standard's own example");
        assert!(out.text.contains("$display(1,,0,,\"C\");"), "{}", out.text);
        assert!(out.text.contains("$display(5,,0,,\"C\");"), "{}", out.text);
    }

    #[test]
    fn an_escaped_macro_name_may_contain_a_backtick() {
        // `SVPP-EXPANSION.5`(q)(ii): §5.6.1 — an escaped identifier is a backslash and "any printable ASCII
        // characters except white space", so the NAME may hold a backtick or a quote (iverilog
        // ivltests/escaped_macro_name.v:3-6, :15-19). The usage `` `\`\`" `` is the macro named `` `\`" `` where that
        // name is defined, while the escaped-quote token `` `\`" `` of a macro STRING keeps its meaning.
        let dir = create_repo_scratch_dir("svpp_escaped_macro_backtick");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define \\`name      \"backtick name\"\n`define \\`          \"backtick\"\n`define \\quote (x)  `\"`\\`\"x`\\`\"`\"\n`define \\`\\`\"       \"escaped quote\"\nmodule t;\ninitial begin\n  $display(`\\`name );\n  $display(`\\` );\n  $display(`quote(text));\n  $display(`\\quote (text));\n  $display(`\\`\\`\" );\nend\nendmodule\n",
        )
        .expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("escaped names holding a backtick or a quote");
        assert!(out.text.contains("$display(\"backtick name\" );"), "{}", out.text);
        assert!(out.text.contains("$display(\"backtick\" );"), "{}", out.text);
        assert_eq!(
            out.text.matches("$display(\"\\\"text\\\"\");").count(),
            2,
            "`quote(text) and `\\quote (text) both render the escaped quotes:\n{}",
            out.text
        );
        assert!(out.text.contains("$display(\"escaped quote\" );"), "{}", out.text);
    }

    #[test]
    fn an_escaped_identifier_as_a_conditional_argument() {
        // `SVPP-EXPANSION.5`(q)(ii): `ifdef / `undef take a text_macro_identifier, which may be escaped, and
        // `\esc` is `esc` (§5.6.1).
        let dir = create_repo_scratch_dir("svpp_escaped_conditional");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define \\esc 1\n`ifdef \\esc\nwire a;\n`endif\n`undef \\esc\n`ifdef esc\nwire b;\n`endif\nwire ok;\n",
        )
        .expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("expand");
        assert!(out.text.contains("wire a;"), "{}", out.text);
        assert!(!out.text.contains("wire b;"), "{}", out.text);
        assert!(out.text.contains("wire ok;"), "{}", out.text);
    }

    #[test]
    fn a_guarded_self_include_is_finite_and_accepted() {
        // `SVPP-EXPANSION.5`(q)(i): IEEE 1800-2017 §22.4 — "A file included in the source using the `include
        // compiler directive may contain other `include compiler directives. The number of nesting levels for
        // include files shall be finite. Implementations may limit the maximum number of levels to which include
        // files can be nested, but the limit shall be at least 15." (docs/systemverilog/2017/txt/
        // section-22-compiler-directives.txt:96-98; 1800-2023 :107-109). A file that includes ITSELF under a
        // guard nests once and is finite (sv2v `test/lex/file_line.sv`, `include_self.sv`); before this the
        // stage refused any file already on the include stack as an "include cycle".
        let dir = create_repo_scratch_dir("svpp_guarded_self_include");
        let input = dir.join("self.sv");
        fs::write(
            &input,
            "`ifndef INCLUDED\n`define INCLUDED\n`include \"self.sv\"\n`else\nwire from_second_pass;\n`endif\nwire after;\n",
        )
        .expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("a guarded self-include is finite");
        assert_eq!(
            out.text.matches("wire from_second_pass;").count(),
            1,
            "{}",
            out.text
        );
        assert_eq!(out.text.matches("wire after;").count(), 2, "{}", out.text);
    }

    #[test]
    fn an_unguarded_self_include_is_refused_by_depth_naming_the_clause() {
        // `SVPP-EXPANSION.5`(q)(i), the other side: the bound the standard names is NESTING, so an unguarded
        // self-include is refused when it passes the depth limit, and the refusal cites §22.4 and shows the chain.
        let dir = create_repo_scratch_dir("svpp_unguarded_self_include");
        let input = dir.join("loop.sv");
        fs::write(&input, "wire w;\n`include \"loop.sv\"\n").expect("write");
        let err = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect_err("an infinite include nesting is refused");
        let text = format!("{:#}", err);
        assert!(
            text.contains("22.4") && text.contains("finite") && text.contains("loop.sv -> "),
            "{}",
            text
        );
    }

    #[test]
    fn a_line_directive_whose_parameters_are_macros_is_expanded_then_consumed() {
        // `SVPP-EXPANSION.5`(n): sv2v `test/lex/line.sv:9` `` `line `new_line_num `foo(line) 1 ``. IEEE
        // 1800-2023 §22.2: "Macro expansion shall occur within a compiler directive"
        // (docs/systemverilog/2023/txt/section-22-compiler-directives.txt:68); 1800-2017 §22.5.1: a defined
        // text macro "can be used anywhere in the compilation unit where it is defined"
        // (docs/systemverilog/2017/txt/section-22-compiler-directives.txt:227). The parameters are expanded
        // first; what they yield is §22.12's `number "filename" level`, and the directive is CONSUMED.
        let dir = create_repo_scratch_dir("svpp_line_macro_parameters");
        let input = dir.join("top.sv");
        let body = |directive: &str| {
            format!(
                "`define new_line_num 200\n`define foo(filename) `\"filename.vh`\"\nmodule t;\ninitial begin\n  $display(`__LINE__);\n{directive}\n  $display(`__FILE__, `__LINE__);\n  $display(`__LINE__);\nend\nendmodule\n"
            )
        };
        fs::write(&input, body("`line `new_line_num `foo(line) 1")).expect("write");
        let via_macro = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("expand");
        assert!(
            !via_macro.text.contains("`line"),
            "the directive is consumed:\n{}",
            via_macro.text
        );
        assert!(
            via_macro.text.contains("$display(\"line.vh\", 200);"),
            "{}",
            via_macro.text
        );
        assert!(via_macro.text.contains("$display(201);"), "{}", via_macro.text);
        assert!(
            via_macro
                .events
                .iter()
                .any(|e| e.kind == super::PreprocessorEventKind::LineDirective
                    && e.detail == "200 \"line.vh\""),
            "{:?}",
            via_macro.events
        );
        // …and it reads EXACTLY as the literal form does.
        fs::write(&input, body("`line 200 \"line.vh\" 1")).expect("write");
        let literal = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("the literal form");
        assert_eq!(
            via_macro.text, literal.text,
            "a macro-spelled `line reads as the literal one"
        );
    }

    #[test]
    fn a_line_directive_whose_macros_do_not_form_it_still_passes_through() {
        // `SVPP-EXPANSION.5`(n), the other side: the expansion is not a well-formed §22.12 directive
        // (level 3), so it is not this stage's to consume — it passes through, expanded ONCE, and the
        // parser's own production refuses it. Green before and after the fix: a regression guard.
        let dir = create_repo_scratch_dir("svpp_line_macro_malformed");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define LVL 3\nmodule t;\n`line 1 \"f\" `LVL\nendmodule\n",
        )
        .expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("expand");
        assert!(
            out.text.contains("`line 1 \"f\" 3"),
            "the malformed one passes through:\n{}",
            out.text
        );
        assert!(
            !out.events
                .iter()
                .any(|e| e.kind == super::PreprocessorEventKind::LineDirective),
            "{:?}",
            out.events
        );
        assert_eq!(
            out.events
                .iter()
                .filter(|e| e.kind == super::PreprocessorEventKind::MacroExpand)
                .count(),
            1,
            "the parameter is expanded (and recorded) once:\n{:?}",
            out.events
        );
    }

    #[test]
    fn a_comment_inside_a_macro_formal_list_is_refused_naming_the_clause() {
        // `SVPP-EXPANSION.5`(o): verilator `t_preproc.v:277` (its own words: "comments MIGHT not be
        // legal"), iverilog `ivltests/macro_with_args.v:17`. IEEE 1800-2017 §22.5.1: "The formal argument
        // names shall be simple_identifiers, separated by commas and optionally white space"
        // (docs/systemverilog/2017/txt/section-22-compiler-directives.txt:143-144; 1800-2023 :168-169);
        // §5.3: white space is spaces, tabs, newlines and formfeeds — and the clause writes "white space
        // or a comment" where it admits one (§22.4, :82). No sentence of either edition admits a comment
        // between formals, so the refusal stands and now names the clause.
        let dir = create_repo_scratch_dir("svpp_formal_list_comment");
        let input = dir.join("top.sv");
        for source in [
            "`define ARGPAR(a, /*more,,)cmts*/ b) (a+b)\nwire x = `ARGPAR(p,q);\n",
            "`define ARGPAR(a,  // Hello\n  b) (a+b)\nwire x = `ARGPAR(p,q);\n",
            "`define sum( a /* comment */ , b /* comment */ ) (a + b)\nwire x = `sum(p,q);\n",
        ] {
            fs::write(&input, source).expect("write");
            let err = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
                .expect_err("a comment in the formal list is refused");
            let text = format!("{:#}", err);
            assert!(
                text.contains("22.5.1") && text.contains("white space") && text.contains("comment"),
                "{}\n— for —\n{}",
                text,
                source
            );
        }
        // the control: a NEWLINE is white space (§5.3) — the same list without the comments
        fs::write(
            &input,
            "`define ARGPAR(a,\n  b) (a+b)\nwire x = `ARGPAR(p,q);\n",
        )
        .expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("newlines separate formals");
        assert!(out.text.contains("wire x = (p+q);"), "{}", out.text);
    }

    #[test]
    fn a_line_directive_is_consumed_and_renames_the_following_lines() {
        // iverilog ivltests/line_directive.v — §22.12: the directive sets the line number and file
        // name of the FOLLOWING line; a malformed one (level 3) is not this stage's and passes through
        let dir = create_repo_scratch_dir("svpp_line_directive");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "module t;\ninitial begin\n  $display(`__LINE__);\n`line 10 \"real_source.v\" 0\n  $display(`__FILE__, `__LINE__);\n  $display(`__LINE__);\n`line 1 \"f\" 3\nend\nendmodule\n",
        )
        .expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("expand");
        assert!(out.text.contains("$display(3);"), "{}", out.text);
        assert!(
            out.text.contains("$display(\"real_source.v\", 10);"),
            "{}",
            out.text
        );
        assert!(out.text.contains("$display(11);"), "{}", out.text);
        assert!(
            !out.text.contains("`line 10"),
            "the well-formed directive is consumed:\n{}",
            out.text
        );
        assert!(
            out.text.contains("`line 1 \"f\" 3"),
            "the malformed one passes through:\n{}",
            out.text
        );
        assert!(
            out.events
                .iter()
                .any(|e| e.kind == super::PreprocessorEventKind::LineDirective
                    && e.detail == "10 \"real_source.v\""),
            "{:?}",
            out.events
        );
    }

    #[test]
    fn a_one_line_comment_in_a_macro_actual_is_not_substituted() {
        // SVPP-EXPANSION.5(l) — OpenTitan `spi_host_dummy_seq.sv:14-24`:
        // `` `DV_CHECK_RANDOMIZE_WITH_FATAL(req, … // TODO(#15721), dummy CSB … ) ``. The argument
        // scanner already ignores a comma or a `)` inside a `//` comment, but the comment's TEXT
        // stayed in the actual and was substituted onto the expansion's single line, where it
        // commented out the rest of the call. §22.5.1: a one-line comment "shall not become part
        // of the substituted text" — an actual is substituted text. A block comment stays, and a
        // `//` inside a string literal is the string's.
        let dir = create_repo_scratch_dir("svpp_actual_line_comment");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define M(T_, MSG_=\"\") if (!(T_)) $display(MSG_);\n`M(a == b // a comment, with a comma\n)\n`M(a == b, // why\n   \"msg\")\n`M(a /* kept */ == c)\n`M(\"a//b\" == s)\n",
        )
        .expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("expand");
        assert!(
            out.text.contains("if (!(a == b)) $display(\"\");"),
            "a line comment inside an actual must not reach the expansion: {}",
            out.text
        );
        assert!(
            out.text.contains("if (!(a == b)) $display(\"msg\");"),
            "an actual that begins with a line comment keeps what follows it: {}",
            out.text
        );
        assert!(!out.text.contains("a comment"), "{}", out.text);
        assert!(!out.text.contains("why"), "{}", out.text);
        assert!(
            out.text.contains("if (!(a /* kept */ == c)) $display(\"\");"),
            "a block comment is white space to the parser and stays: {}",
            out.text
        );
        assert!(
            out.text.contains("if (!(\"a//b\" == s)) $display(\"\");"),
            "a `//` inside a string literal is the string's: {}",
            out.text
        );
    }

    #[test]
    fn a_one_line_comment_in_macro_text_is_not_substituted() {
        // sv2v lex/macro.sv NUMBER_11 / NUMBER_12 — §22.5.1: "the comment shall not become part of
        // the substituted text"; a block comment stays; a string keeps its slashes; a continued
        // line keeps its continuation
        let dir = create_repo_scratch_dir("svpp_macro_text_comment");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define N11 42 //\n`define N12 21 */* foo */2//foo\n`define S \"a//b\" // c\n`define M first // x \\\n  second\n$display(`N11);\n$display(`N12);\n$display(`S);\n$display(`M);\n",
        )
        .expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("expand");
        assert!(out.text.contains("$display(42);"), "{}", out.text);
        assert!(
            out.text.contains("$display(21 */* foo */2);"),
            "{}",
            out.text
        );
        assert!(out.text.contains("$display(\"a//b\");"), "{}", out.text);
        assert!(
            out.text.contains("$display(first") && out.text.contains("second);"),
            "{}",
            out.text
        );
    }

    #[test]
    fn a_function_like_macro_may_open_its_parentheses_on_a_later_line() {
        // sv2v lex/macro_whitespace.sv — §22.5.1 requires the parentheses; §5.3 lets a newline
        // separate the name from them
        let dir = create_repo_scratch_dir("svpp_paren_next_line");
        let input = dir.join("top.sv");
        fs::write(&input, "`define FOO(a, b) ((a)+(b))\nmodule top;\n  initial begin\n    $display(`FOO\n    (\n      1\n      ,\n      2\n    ));\n  end\nendmodule\n").expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("expand");
        assert!(out.text.contains("$display(((1)+(2)));"), "{}", out.text);
    }

    // ── SVPP-EXPANSION.6b.5 — the four engine rules the dispositioned rows named ────────────────

    #[test]
    fn a_formal_named_like_a_system_task_is_not_substituted_inside_it() {
        // sv2v lex/macro.sv: `define MACRO_D(display) $display(display); — A.9.3: `$display` is one token
        let dir = create_repo_scratch_dir("svpp_system_name_formal");
        let input = dir.join("top.sv");
        fs::write(&input, "`define MACRO_D(display) $display(display);\n`define S(x) `\"$x x`\"\n`MACRO_D(\"hi\")\n$display(`S(1));\n").expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("expand");
        assert!(out.text.contains("$display(\"hi\");"), "{}", out.text);
        assert!(out.text.contains("$display(\"$x 1\");"), "{}", out.text);
    }

    #[test]
    fn a_block_comment_hides_directive_lines_and_survives_an_include_line() {
        // Surelog PPComment: `/*` opened on a source line, the `else / `endif inside the comment are
        // comment text (§5.4); iverilog br_gh782a: `include "x" /* multi-line` keeps its opener
        let dir = create_repo_scratch_dir("svpp_block_comment_lines");
        fs::write(dir.join("inc.svh"), "int from_inc;\n").expect("write inc");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "module top();\n`define USE\n`ifdef USE\n  assign a = b;\n  assign c = d;/*\n`else\n  assign a = 1'b0;\n  assign c = 1'b0;*/\n`endif\n`include \"inc.svh\"  /* multi-line\n   comment */\n  int after;\nendmodule\n",
        )
        .expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("expand");
        assert!(
            out.text
                .contains("assign c = d;/*\n`else\n  assign a = 1'b0;\n  assign c = 1'b0;*/"),
            "{}",
            out.text
        );
        assert!(
            out.text.contains("int from_inc;") && out.text.contains("/* multi-line\n   comment */"),
            "{}",
            out.text
        );
        assert!(out.text.contains("int after;"), "{}", out.text);
        assert!(!out.text.contains("`endif"), "{}", out.text);
        // a close that ends its line keeps the line's terminator (the first cut fused `*/endfunction`)
        fs::write(&input, "function f;\n  /* a\n     b\n   */\nendfunction\n").expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("expand");
        assert_eq!(
            out.text,
            "function f;\n  /* a\n     b\n   */\nendfunction\n"
        );
    }

    #[test]
    fn the_coverage_control_macros_are_predefined() {
        // IEEE 1800-2017 §40.3.1 (section-40-code-coverage-control-and-api.md:96-140): the tool defines them
        let dir = create_repo_scratch_dir("svpp_sv_cov_predefined");
        let input = dir.join("top.sv");
        fs::write(&input, "int i = $coverage_control(`SV_COV_CHECK, `SV_COV_TOGGLE, `SV_COV_HIER, $root);\n`define SV_COV_ERROR -9\nint e = `SV_COV_ERROR;\n").expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("expand");
        assert!(
            out.text.contains("$coverage_control(3, 23, 11, $root);"),
            "{}",
            out.text
        );
        assert!(
            out.text.contains("int e = -9;"),
            "a source define supersedes the predefinition:\n{}",
            out.text
        );
    }

    #[test]
    fn a_protected_envelope_is_consumed_as_opaque() {
        // IEEE 1800-2017 §34.3: begin_protected … end_protected delimit the envelope; ispras 34.03.01_02.sv
        let dir = create_repo_scratch_dir("svpp_protected_envelope");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "module top(a, b);\n  input a;\n  output b;\n`pragma protect encoding=(enctype=\"raw\")\n`pragma protect data_method=\"x-caesar\", data_keyname=\"rot13\", begin_protected\n`pragma protect encoding=(enctype=\"raw\"), data_block\n`centzn cebgrpg ehagvzr_yvprafr=(yvoenel=\"yvp.fb\")\n  ert o;\n`pragma protect end_protected\n`pragma reset protect\nendmodule\n",
        )
        .expect("write");
        let out = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("expand");
        assert!(
            !out.text.contains("ert o;") && !out.text.contains("centzn"),
            "{}",
            out.text
        );
        assert!(
            out.text
                .contains("`pragma protect encoding=(enctype=\"raw\")\n")
                && out.text.contains("`pragma reset protect"),
            "{}",
            out.text
        );
        assert!(out.text.contains("endmodule"), "{}", out.text);
        let n = out
            .events
            .iter()
            .filter(|e| e.kind == super::PreprocessorEventKind::ProtectedEnvelope)
            .count();
        assert_eq!(n, 2, "{:?}", out.events);
        fs::write(&input, "`pragma protect begin_protected\nxx\n").expect("write");
        let err = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect_err("unterminated");
        assert!(format!("{:#}", err).contains("E_SVPP_UNTERMINATED_PROTECTED_ENVELOPE"));
    }
}
