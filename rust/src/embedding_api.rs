use crate::ast_pipeline::{ParseError, UnifiedReturnAST, UnifiedSemanticAST, runtime_logger};
use crate::regex_compile_validation::{
    RegexCompileValidationError, validate_regex_compile_contract,
};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::fmt;
use std::str::FromStr;
#[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
const GENERATED_REGEX_WORKER_STACK_BYTES: usize = 8 * 1024 * 1024;

/// Stable embedding API contract version.
///
/// Compatibility policy:
/// - major version changes signal breaking API/behavioral contract changes
/// - minor/patch changes are backward compatible for existing callers
pub const EMBEDDING_API_VERSION: &str = "1.2.0";

/// Stable schema version for serialized embedding API metadata.
pub const EMBEDDING_API_SCHEMA_VERSION: u32 = 2;

/// Stable downstream contract version for the published regex parser handoff.
pub const REGEX_PARSER_INTEGRATION_CONTRACT_VERSION: &str = "1.1.20";

/// Stable release version for the published regex parser.
pub const REGEX_PARSER_RELEASE_VERSION: &str = "1.1.19";

/// Stable schema version for regex AST-dump JSON payloads.
pub const REGEX_AST_DUMP_SCHEMA_VERSION: u32 = 1;

/// Default hard limit for embedding API input payload size (in bytes).
///
/// This bound is intentionally conservative enough for typical annotation payloads
/// while protecting embedders from accidental unbounded input growth.
pub const EMBEDDING_API_DEFAULT_MAX_INPUT_BYTES: usize = 1_048_576; // 1 MiB

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationFamily {
    Return,
    Semantic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParserBackend {
    Bootstrap,
    Generated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParseStatus {
    Success,
    Failure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputOwnershipModel {
    BorrowedStr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParseSessionModel {
    StatelessPerCall,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseDiagnosticLocation {
    pub byte_offset: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseDiagnostic {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<ParseDiagnosticLocation>,
}

impl ParseDiagnostic {
    fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            location: None,
        }
    }

    fn with_location(
        code: impl Into<String>,
        message: impl Into<String>,
        location: ParseDiagnosticLocation,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            location: Some(location),
        }
    }
}

impl fmt::Display for ParseDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ParseDiagnostic {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseLimits {
    pub max_input_bytes: usize,
}

impl Default for ParseLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: EMBEDDING_API_DEFAULT_MAX_INPUT_BYTES,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AstDumpOptions {
    pub pretty: bool,
    pub max_ast_bytes: Option<usize>,
}

impl Default for AstDumpOptions {
    fn default() -> Self {
        Self {
            pretty: false,
            max_ast_bytes: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AstDumpPayload {
    pub dump_json: String,
    pub truncated: bool,
    pub full_bytes: usize,
    pub emitted_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseOutcome {
    pub api_version: String,
    pub family: AnnotationFamily,
    pub backend: ParserBackend,
    pub status: ParseStatus,
    pub diagnostic: Option<ParseDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrammarFamily {
    #[serde(rename = "systemverilog")]
    SystemVerilog,
    #[serde(rename = "vhdl")]
    Vhdl,
    #[serde(rename = "regex")]
    Regex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrammarProfile {
    #[serde(rename = "sv_2017")]
    Sv2017,
    #[serde(rename = "sv_2023")]
    Sv2023,
    #[serde(rename = "vhdl_1076_2019")]
    Vhdl1076_2019,
    #[serde(rename = "regex_default")]
    RegexDefault,
}

impl AnnotationFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            AnnotationFamily::Return => "return",
            AnnotationFamily::Semantic => "semantic",
        }
    }
}

impl ParserBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            ParserBackend::Bootstrap => "bootstrap",
            ParserBackend::Generated => "generated",
        }
    }
}

impl GrammarFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            GrammarFamily::SystemVerilog => "systemverilog",
            GrammarFamily::Vhdl => "vhdl",
            GrammarFamily::Regex => "regex",
        }
    }
}

impl GrammarProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            GrammarProfile::Sv2017 => "sv_2017",
            GrammarProfile::Sv2023 => "sv_2023",
            GrammarProfile::Vhdl1076_2019 => "vhdl_1076_2019",
            GrammarProfile::RegexDefault => "regex_default",
        }
    }
}

impl FromStr for AnnotationFamily {
    type Err = ParseDiagnostic;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_ascii_lowercase().as_str() {
            "return" | "return_annotation" => Ok(Self::Return),
            "semantic" | "semantic_annotation" => Ok(Self::Semantic),
            other => Err(invalid_argument_diagnostic(format!(
                "unsupported annotation family '{}'",
                other
            ))),
        }
    }
}

impl FromStr for ParserBackend {
    type Err = ParseDiagnostic;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_ascii_lowercase().as_str() {
            "bootstrap" => Ok(Self::Bootstrap),
            "generated" => Ok(Self::Generated),
            other => Err(invalid_argument_diagnostic(format!(
                "unsupported parser backend '{}'",
                other
            ))),
        }
    }
}

impl FromStr for GrammarFamily {
    type Err = ParseDiagnostic;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_ascii_lowercase().as_str() {
            "systemverilog" | "sv" => Ok(Self::SystemVerilog),
            "vhdl" => Ok(Self::Vhdl),
            "regex" => Ok(Self::Regex),
            other => Err(invalid_argument_diagnostic(format!(
                "unsupported grammar family '{}'",
                other
            ))),
        }
    }
}

impl FromStr for GrammarProfile {
    type Err = ParseDiagnostic;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim().to_ascii_lowercase().as_str() {
            "sv_2017" | "2017" | "ieee1800-2017" | "ieee_1800_2017" => Ok(Self::Sv2017),
            "sv_2023" | "2023" | "ieee1800-2023" | "ieee_1800_2023" => Ok(Self::Sv2023),
            "vhdl_1076_2019" | "1076-2019" | "ieee1076-2019" | "ieee_1076_2019" => {
                Ok(Self::Vhdl1076_2019)
            }
            "regex_default" | "regex" => Ok(Self::RegexDefault),
            other => Err(invalid_argument_diagnostic(format!(
                "unsupported grammar profile '{}'",
                other
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrammarParseOutcome {
    pub api_version: String,
    pub grammar: GrammarFamily,
    pub profile: GrammarProfile,
    pub status: ParseStatus,
    pub diagnostic: Option<ParseDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrammarAstDumpOutcome {
    pub api_version: String,
    pub grammar: GrammarFamily,
    pub profile: GrammarProfile,
    pub status: ParseStatus,
    pub diagnostic: Option<ParseDiagnostic>,
    pub ast_dump: Option<AstDumpPayload>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedAnnotationParseOutcome {
    pub api_version: String,
    pub family: String,
    pub backend: String,
    pub status: ParseStatus,
    pub diagnostic: Option<ParseDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedGrammarParseOutcome {
    pub api_version: String,
    pub grammar: String,
    pub profile: String,
    pub status: ParseStatus,
    pub diagnostic: Option<ParseDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedGrammarAstDumpOutcome {
    pub api_version: String,
    pub grammar: String,
    pub profile: String,
    pub status: ParseStatus,
    pub diagnostic: Option<ParseDiagnostic>,
    pub ast_dump: Option<AstDumpPayload>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingApiContract {
    pub api_version: String,
    pub schema_version: u32,
    pub supports_generated_backend: bool,
    pub deterministic_by_default: bool,
    pub supported_families: Vec<AnnotationFamily>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParserEmbeddingApiContract {
    pub api_version: String,
    pub schema_version: u32,
    pub deterministic_by_default: bool,
    pub input_ownership_model: InputOwnershipModel,
    pub parse_session_model: ParseSessionModel,
    pub zero_copy_input_boundary: bool,
    pub stable_diagnostic_codes: Vec<String>,
    pub stable_diagnostic_location_fields: Vec<String>,
    pub supported_grammars: Vec<GrammarFamily>,
    pub supported_profiles: Vec<GrammarProfile>,
    pub profile_matrix: Vec<GrammarProfileBinding>,
    pub supports_systemverilog_generated_backend: bool,
    pub supports_vhdl_generated_backend: bool,
    pub supports_regex_generated_backend: bool,
    pub regex_integration_contract_version: String,
    pub regex_parser_release_version: String,
    pub regex_ast_dump_schema_version: u32,
    pub regex_generated_backend_required_feature: String,
    pub regex_generated_backend_required_artifact: String,
    pub regex_generated_backend_env_override: String,
    pub regex_frontend_json_artifact: String,
    pub regex_frontend_json_role: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrammarProfileBinding {
    pub grammar: GrammarFamily,
    pub profiles: Vec<GrammarProfile>,
}

/// Returns stable contract metadata for embedding users.
pub fn embedding_api_contract() -> EmbeddingApiContract {
    EmbeddingApiContract {
        api_version: EMBEDDING_API_VERSION.to_string(),
        schema_version: EMBEDDING_API_SCHEMA_VERSION,
        supports_generated_backend: generated_backend_enabled(),
        deterministic_by_default: true,
        supported_families: vec![AnnotationFamily::Return, AnnotationFamily::Semantic],
    }
}

/// Returns stable grammar-parser contract metadata for embedding users.
///
/// This contract covers profile-aware parser entry points used by host applications
/// (for example Nexsim) and intentionally keeps diagnostics deterministic.
pub fn parser_embedding_api_contract() -> ParserEmbeddingApiContract {
    let systemverilog_profiles = vec![GrammarProfile::Sv2017, GrammarProfile::Sv2023];
    let vhdl_profiles = vec![GrammarProfile::Vhdl1076_2019];
    let regex_profiles = vec![GrammarProfile::RegexDefault];
    ParserEmbeddingApiContract {
        api_version: EMBEDDING_API_VERSION.to_string(),
        schema_version: EMBEDDING_API_SCHEMA_VERSION,
        deterministic_by_default: true,
        input_ownership_model: InputOwnershipModel::BorrowedStr,
        parse_session_model: ParseSessionModel::StatelessPerCall,
        zero_copy_input_boundary: true,
        stable_diagnostic_codes: vec![
            "E_BACKEND_UNAVAILABLE".to_string(),
            "E_INPUT_TOO_LARGE".to_string(),
            "E_INVALID_ARGUMENT".to_string(),
            "E_INVALID_LIMITS".to_string(),
            "E_PARSE_FAILURE".to_string(),
            "E_UNSUPPORTED_PROFILE".to_string(),
        ],
        stable_diagnostic_location_fields: vec![
            "byte_offset".to_string(),
            "line".to_string(),
            "column".to_string(),
        ],
        supported_grammars: vec![
            GrammarFamily::SystemVerilog,
            GrammarFamily::Vhdl,
            GrammarFamily::Regex,
        ],
        supported_profiles: vec![
            systemverilog_profiles[0],
            systemverilog_profiles[1],
            vhdl_profiles[0],
            regex_profiles[0],
        ],
        profile_matrix: vec![
            GrammarProfileBinding {
                grammar: GrammarFamily::SystemVerilog,
                profiles: systemverilog_profiles,
            },
            GrammarProfileBinding {
                grammar: GrammarFamily::Vhdl,
                profiles: vhdl_profiles,
            },
            GrammarProfileBinding {
                grammar: GrammarFamily::Regex,
                profiles: regex_profiles,
            },
        ],
        supports_systemverilog_generated_backend: systemverilog_generated_backend_enabled(),
        supports_vhdl_generated_backend: vhdl_generated_backend_enabled(),
        supports_regex_generated_backend: regex_generated_backend_enabled(),
        regex_integration_contract_version: REGEX_PARSER_INTEGRATION_CONTRACT_VERSION.to_string(),
        regex_parser_release_version: REGEX_PARSER_RELEASE_VERSION.to_string(),
        regex_ast_dump_schema_version: REGEX_AST_DUMP_SCHEMA_VERSION,
        regex_generated_backend_required_feature: "generated_parsers".to_string(),
        regex_generated_backend_required_artifact: "generated/regex_parser.rs".to_string(),
        regex_generated_backend_env_override: "PGEN_REGEX_PARSER_PATH".to_string(),
        regex_frontend_json_artifact: "generated/regex.json".to_string(),
        regex_frontend_json_role:
            "PGEN-generated frontend normalization and provenance artifact; not a stable downstream runtime input or AST contract"
                .to_string(),
    }
}

fn invalid_argument_diagnostic(message: String) -> ParseDiagnostic {
    ParseDiagnostic::new("E_INVALID_ARGUMENT", message)
}

fn invalid_limits_diagnostic(message: impl Into<String>) -> ParseDiagnostic {
    ParseDiagnostic::new("E_INVALID_LIMITS", message)
}

fn input_too_large_diagnostic(message: impl Into<String>) -> ParseDiagnostic {
    ParseDiagnostic::new("E_INPUT_TOO_LARGE", message)
}

fn backend_unavailable_diagnostic(message: impl Into<String>) -> ParseDiagnostic {
    ParseDiagnostic::new("E_BACKEND_UNAVAILABLE", message)
}

fn parse_failure_diagnostic(message: impl Into<String>) -> ParseDiagnostic {
    ParseDiagnostic::new("E_PARSE_FAILURE", message)
}

fn unsupported_profile_diagnostic(
    grammar: GrammarFamily,
    profile: GrammarProfile,
) -> ParseDiagnostic {
    ParseDiagnostic::new(
        "E_UNSUPPORTED_PROFILE",
        format!(
            "profile {:?} is not supported for grammar {:?}",
            profile, grammar
        ),
    )
}

fn parse_error_position(error: &ParseError) -> usize {
    match error {
        ParseError::UnexpectedEof { position }
        | ParseError::UnexpectedToken { position, .. }
        | ParseError::InvalidSyntax { position, .. }
        | ParseError::Backtrack { position }
        | ParseError::RecursionDepthExceeded { position, .. }
        | ParseError::ContextualError { position, .. } => *position,
    }
}

fn clamp_to_char_boundary(input: &str, byte_offset: usize) -> usize {
    let mut clamped = byte_offset.min(input.len());
    while clamped > 0 && !input.is_char_boundary(clamped) {
        clamped -= 1;
    }
    clamped
}

fn parse_diagnostic_location(input: &str, byte_offset: usize) -> ParseDiagnosticLocation {
    let byte_offset = clamp_to_char_boundary(input, byte_offset);
    let mut line = 1usize;
    let mut column = 1usize;
    for ch in input[..byte_offset].chars() {
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    ParseDiagnosticLocation {
        byte_offset,
        line,
        column,
    }
}

fn generated_parse_failure_diagnostic(
    parser_label: &str,
    input: &str,
    error: ParseError,
) -> ParseDiagnostic {
    ParseDiagnostic::with_location(
        "E_PARSE_FAILURE",
        format!("generated {} parse failed: {}", parser_label, error),
        parse_diagnostic_location(input, parse_error_position(&error)),
    )
}

fn regex_compile_contract_diagnostic(
    input: &str,
    error: RegexCompileValidationError,
) -> ParseDiagnostic {
    ParseDiagnostic::with_location(
        "E_PARSE_FAILURE",
        error.message,
        parse_diagnostic_location(input, error.byte_offset),
    )
}

/// Parses a single annotation payload using the selected parser family/backend.
///
/// This API intentionally returns a structured outcome instead of exposing
/// internal AST/parser types, so embedders are insulated from internal refactors.
pub fn parse_annotation(
    family: AnnotationFamily,
    backend: ParserBackend,
    input: &str,
) -> ParseOutcome {
    parse_annotation_with_limits(family, backend, input, &ParseLimits::default())
}

/// Parses a single annotation payload using explicit parse limits.
///
/// This allows embedders to tighten bounds per call site while preserving
/// deterministic and structured outcomes.
pub fn parse_annotation_with_limits(
    family: AnnotationFamily,
    backend: ParserBackend,
    input: &str,
    limits: &ParseLimits,
) -> ParseOutcome {
    match run_parse(family, backend, input, limits) {
        Ok(()) => ParseOutcome {
            api_version: EMBEDDING_API_VERSION.to_string(),
            family,
            backend,
            status: ParseStatus::Success,
            diagnostic: None,
        },
        Err(diagnostic) => ParseOutcome {
            api_version: EMBEDDING_API_VERSION.to_string(),
            family,
            backend,
            status: ParseStatus::Failure,
            diagnostic: Some(diagnostic),
        },
    }
}

/// Idiomatic Rust Result-based annotation parse API.
///
/// This wrapper is intended for Rust projects that prefer `Result` error flow.
pub fn parse_annotation_result(
    family: AnnotationFamily,
    backend: ParserBackend,
    input: &str,
) -> Result<(), ParseDiagnostic> {
    parse_annotation_with_limits_result(family, backend, input, &ParseLimits::default())
}

/// Result-based annotation parse API with explicit input limits.
pub fn parse_annotation_with_limits_result(
    family: AnnotationFamily,
    backend: ParserBackend,
    input: &str,
    limits: &ParseLimits,
) -> Result<(), ParseDiagnostic> {
    run_parse(family, backend, input, limits)
}

/// Language-neutral annotation parse API using string names.
///
/// This entry point is useful for FFI/binding layers that pass string identifiers.
pub fn parse_annotation_named(
    family: &str,
    backend: &str,
    input: &str,
) -> NamedAnnotationParseOutcome {
    parse_annotation_named_with_limits(family, backend, input, &ParseLimits::default())
}

/// Language-neutral annotation parse API using string names and explicit limits.
pub fn parse_annotation_named_with_limits(
    family: &str,
    backend: &str,
    input: &str,
    limits: &ParseLimits,
) -> NamedAnnotationParseOutcome {
    match parse_annotation_named_with_limits_result(family, backend, input, limits) {
        Ok(()) => NamedAnnotationParseOutcome {
            api_version: EMBEDDING_API_VERSION.to_string(),
            family: family.to_string(),
            backend: backend.to_string(),
            status: ParseStatus::Success,
            diagnostic: None,
        },
        Err(diagnostic) => NamedAnnotationParseOutcome {
            api_version: EMBEDDING_API_VERSION.to_string(),
            family: family.to_string(),
            backend: backend.to_string(),
            status: ParseStatus::Failure,
            diagnostic: Some(diagnostic),
        },
    }
}

/// Result-based language-neutral annotation parse API using string names.
pub fn parse_annotation_named_with_limits_result(
    family: &str,
    backend: &str,
    input: &str,
    limits: &ParseLimits,
) -> Result<(), ParseDiagnostic> {
    let family = AnnotationFamily::from_str(family)?;
    let backend = ParserBackend::from_str(backend)?;
    run_parse(family, backend, input, limits)
}

/// Parses a full grammar input using a stable profile-aware parser entry point.
pub fn parse_grammar_profile(
    grammar: GrammarFamily,
    profile: GrammarProfile,
    input: &str,
) -> GrammarParseOutcome {
    parse_grammar_profile_with_limits(grammar, profile, input, &ParseLimits::default())
}

/// Convenience entry point for Nexsim SV integration (`IEEE 1800-2017` profile).
pub fn parse_systemverilog_2017(input: &str) -> GrammarParseOutcome {
    parse_grammar_profile(GrammarFamily::SystemVerilog, GrammarProfile::Sv2017, input)
}

/// Convenience entry point for Nexsim SV integration (`IEEE 1800-2023` profile).
pub fn parse_systemverilog_2023(input: &str) -> GrammarParseOutcome {
    parse_grammar_profile(GrammarFamily::SystemVerilog, GrammarProfile::Sv2023, input)
}

/// Convenience entry point for Nexsim VHDL integration (`IEEE 1076-2019` profile).
pub fn parse_vhdl_1076_2019(input: &str) -> GrammarParseOutcome {
    parse_grammar_profile(GrammarFamily::Vhdl, GrammarProfile::Vhdl1076_2019, input)
}

/// Convenience entry point for regex host integrations (`regex_default` profile).
pub fn parse_regex_default(input: &str) -> GrammarParseOutcome {
    parse_grammar_profile(GrammarFamily::Regex, GrammarProfile::RegexDefault, input)
}

/// Parses a full grammar input using explicit input limits.
pub fn parse_grammar_profile_with_limits(
    grammar: GrammarFamily,
    profile: GrammarProfile,
    input: &str,
    limits: &ParseLimits,
) -> GrammarParseOutcome {
    match run_grammar_parse(grammar, profile, input, limits) {
        Ok(()) => GrammarParseOutcome {
            api_version: EMBEDDING_API_VERSION.to_string(),
            grammar,
            profile,
            status: ParseStatus::Success,
            diagnostic: None,
        },
        Err(diagnostic) => GrammarParseOutcome {
            api_version: EMBEDDING_API_VERSION.to_string(),
            grammar,
            profile,
            status: ParseStatus::Failure,
            diagnostic: Some(diagnostic),
        },
    }
}

/// Convenience entry point with explicit limits for `IEEE 1800-2017`.
pub fn parse_systemverilog_2017_with_limits(
    input: &str,
    limits: &ParseLimits,
) -> GrammarParseOutcome {
    parse_grammar_profile_with_limits(
        GrammarFamily::SystemVerilog,
        GrammarProfile::Sv2017,
        input,
        limits,
    )
}

/// Convenience entry point with explicit limits for `IEEE 1800-2023`.
pub fn parse_systemverilog_2023_with_limits(
    input: &str,
    limits: &ParseLimits,
) -> GrammarParseOutcome {
    parse_grammar_profile_with_limits(
        GrammarFamily::SystemVerilog,
        GrammarProfile::Sv2023,
        input,
        limits,
    )
}

/// Convenience entry point with explicit limits for `IEEE 1076-2019`.
pub fn parse_vhdl_1076_2019_with_limits(input: &str, limits: &ParseLimits) -> GrammarParseOutcome {
    parse_grammar_profile_with_limits(
        GrammarFamily::Vhdl,
        GrammarProfile::Vhdl1076_2019,
        input,
        limits,
    )
}

/// Convenience entry point with explicit limits for `regex_default`.
pub fn parse_regex_default_with_limits(input: &str, limits: &ParseLimits) -> GrammarParseOutcome {
    parse_grammar_profile_with_limits(
        GrammarFamily::Regex,
        GrammarProfile::RegexDefault,
        input,
        limits,
    )
}

/// Idiomatic Rust Result-based grammar parse API.
pub fn parse_grammar_profile_result(
    grammar: GrammarFamily,
    profile: GrammarProfile,
    input: &str,
) -> Result<(), ParseDiagnostic> {
    parse_grammar_profile_with_limits_result(grammar, profile, input, &ParseLimits::default())
}

/// Convenience `Result` API for `IEEE 1800-2017`.
pub fn parse_systemverilog_2017_result(input: &str) -> Result<(), ParseDiagnostic> {
    parse_grammar_profile_result(GrammarFamily::SystemVerilog, GrammarProfile::Sv2017, input)
}

/// Convenience `Result` API for `IEEE 1800-2023`.
pub fn parse_systemverilog_2023_result(input: &str) -> Result<(), ParseDiagnostic> {
    parse_grammar_profile_result(GrammarFamily::SystemVerilog, GrammarProfile::Sv2023, input)
}

/// Convenience `Result` API for `IEEE 1076-2019`.
pub fn parse_vhdl_1076_2019_result(input: &str) -> Result<(), ParseDiagnostic> {
    parse_grammar_profile_result(GrammarFamily::Vhdl, GrammarProfile::Vhdl1076_2019, input)
}

/// Convenience `Result` API for `regex_default`.
pub fn parse_regex_default_result(input: &str) -> Result<(), ParseDiagnostic> {
    parse_grammar_profile_result(GrammarFamily::Regex, GrammarProfile::RegexDefault, input)
}

/// Result-based grammar parse API with explicit input limits.
pub fn parse_grammar_profile_with_limits_result(
    grammar: GrammarFamily,
    profile: GrammarProfile,
    input: &str,
    limits: &ParseLimits,
) -> Result<(), ParseDiagnostic> {
    run_grammar_parse(grammar, profile, input, limits)
}

/// Convenience `Result` API with explicit limits for `IEEE 1800-2017`.
pub fn parse_systemverilog_2017_with_limits_result(
    input: &str,
    limits: &ParseLimits,
) -> Result<(), ParseDiagnostic> {
    parse_grammar_profile_with_limits_result(
        GrammarFamily::SystemVerilog,
        GrammarProfile::Sv2017,
        input,
        limits,
    )
}

/// Convenience `Result` API with explicit limits for `IEEE 1800-2023`.
pub fn parse_systemverilog_2023_with_limits_result(
    input: &str,
    limits: &ParseLimits,
) -> Result<(), ParseDiagnostic> {
    parse_grammar_profile_with_limits_result(
        GrammarFamily::SystemVerilog,
        GrammarProfile::Sv2023,
        input,
        limits,
    )
}

/// Convenience `Result` API with explicit limits for `IEEE 1076-2019`.
pub fn parse_vhdl_1076_2019_with_limits_result(
    input: &str,
    limits: &ParseLimits,
) -> Result<(), ParseDiagnostic> {
    parse_grammar_profile_with_limits_result(
        GrammarFamily::Vhdl,
        GrammarProfile::Vhdl1076_2019,
        input,
        limits,
    )
}

/// Convenience `Result` API with explicit limits for `regex_default`.
pub fn parse_regex_default_with_limits_result(
    input: &str,
    limits: &ParseLimits,
) -> Result<(), ParseDiagnostic> {
    parse_grammar_profile_with_limits_result(
        GrammarFamily::Regex,
        GrammarProfile::RegexDefault,
        input,
        limits,
    )
}

/// Language-neutral grammar parse API using string names.
pub fn parse_grammar_profile_named(
    grammar: &str,
    profile: &str,
    input: &str,
) -> NamedGrammarParseOutcome {
    parse_grammar_profile_named_with_limits(grammar, profile, input, &ParseLimits::default())
}

/// Language-neutral grammar parse API using string names with explicit limits.
pub fn parse_grammar_profile_named_with_limits(
    grammar: &str,
    profile: &str,
    input: &str,
    limits: &ParseLimits,
) -> NamedGrammarParseOutcome {
    match parse_grammar_profile_named_with_limits_result(grammar, profile, input, limits) {
        Ok(()) => NamedGrammarParseOutcome {
            api_version: EMBEDDING_API_VERSION.to_string(),
            grammar: grammar.to_string(),
            profile: profile.to_string(),
            status: ParseStatus::Success,
            diagnostic: None,
        },
        Err(diagnostic) => NamedGrammarParseOutcome {
            api_version: EMBEDDING_API_VERSION.to_string(),
            grammar: grammar.to_string(),
            profile: profile.to_string(),
            status: ParseStatus::Failure,
            diagnostic: Some(diagnostic),
        },
    }
}

/// Result-based language-neutral grammar parse API using string names.
pub fn parse_grammar_profile_named_with_limits_result(
    grammar: &str,
    profile: &str,
    input: &str,
    limits: &ParseLimits,
) -> Result<(), ParseDiagnostic> {
    let grammar = GrammarFamily::from_str(grammar)?;
    let profile = GrammarProfile::from_str(profile)?;
    run_grammar_parse(grammar, profile, input, limits)
}

/// Parses a full grammar input and returns deterministic parser-returned AST dump payload.
pub fn parse_grammar_profile_ast_dump(
    grammar: GrammarFamily,
    profile: GrammarProfile,
    input: &str,
    options: &AstDumpOptions,
) -> GrammarAstDumpOutcome {
    parse_grammar_profile_ast_dump_with_limits(
        grammar,
        profile,
        input,
        &ParseLimits::default(),
        options,
    )
}

/// Parses a full grammar input with explicit limits and returns parser-returned AST dump payload.
pub fn parse_grammar_profile_ast_dump_with_limits(
    grammar: GrammarFamily,
    profile: GrammarProfile,
    input: &str,
    limits: &ParseLimits,
    options: &AstDumpOptions,
) -> GrammarAstDumpOutcome {
    match parse_grammar_profile_ast_dump_with_limits_result(
        grammar, profile, input, limits, options,
    ) {
        Ok(ast_dump) => GrammarAstDumpOutcome {
            api_version: EMBEDDING_API_VERSION.to_string(),
            grammar,
            profile,
            status: ParseStatus::Success,
            diagnostic: None,
            ast_dump: Some(ast_dump),
        },
        Err(diagnostic) => GrammarAstDumpOutcome {
            api_version: EMBEDDING_API_VERSION.to_string(),
            grammar,
            profile,
            status: ParseStatus::Failure,
            diagnostic: Some(diagnostic),
            ast_dump: None,
        },
    }
}

/// Idiomatic Rust Result-based grammar AST dump API.
pub fn parse_grammar_profile_ast_dump_result(
    grammar: GrammarFamily,
    profile: GrammarProfile,
    input: &str,
    options: &AstDumpOptions,
) -> Result<AstDumpPayload, ParseDiagnostic> {
    parse_grammar_profile_ast_dump_with_limits_result(
        grammar,
        profile,
        input,
        &ParseLimits::default(),
        options,
    )
}

/// Result-based grammar AST dump API with explicit input limits.
pub fn parse_grammar_profile_ast_dump_with_limits_result(
    grammar: GrammarFamily,
    profile: GrammarProfile,
    input: &str,
    limits: &ParseLimits,
    options: &AstDumpOptions,
) -> Result<AstDumpPayload, ParseDiagnostic> {
    run_grammar_parse_ast_dump(grammar, profile, input, limits, options)
}

/// Language-neutral grammar AST dump API using string names.
pub fn parse_grammar_profile_ast_dump_named(
    grammar: &str,
    profile: &str,
    input: &str,
    options: &AstDumpOptions,
) -> NamedGrammarAstDumpOutcome {
    parse_grammar_profile_ast_dump_named_with_limits(
        grammar,
        profile,
        input,
        &ParseLimits::default(),
        options,
    )
}

/// Language-neutral grammar AST dump API with explicit limits.
pub fn parse_grammar_profile_ast_dump_named_with_limits(
    grammar: &str,
    profile: &str,
    input: &str,
    limits: &ParseLimits,
    options: &AstDumpOptions,
) -> NamedGrammarAstDumpOutcome {
    match parse_grammar_profile_ast_dump_named_with_limits_result(
        grammar, profile, input, limits, options,
    ) {
        Ok(ast_dump) => NamedGrammarAstDumpOutcome {
            api_version: EMBEDDING_API_VERSION.to_string(),
            grammar: grammar.to_string(),
            profile: profile.to_string(),
            status: ParseStatus::Success,
            diagnostic: None,
            ast_dump: Some(ast_dump),
        },
        Err(diagnostic) => NamedGrammarAstDumpOutcome {
            api_version: EMBEDDING_API_VERSION.to_string(),
            grammar: grammar.to_string(),
            profile: profile.to_string(),
            status: ParseStatus::Failure,
            diagnostic: Some(diagnostic),
            ast_dump: None,
        },
    }
}

/// Result-based language-neutral grammar AST dump API using string names.
pub fn parse_grammar_profile_ast_dump_named_with_limits_result(
    grammar: &str,
    profile: &str,
    input: &str,
    limits: &ParseLimits,
    options: &AstDumpOptions,
) -> Result<AstDumpPayload, ParseDiagnostic> {
    let grammar = GrammarFamily::from_str(grammar)?;
    let profile = GrammarProfile::from_str(profile)?;
    run_grammar_parse_ast_dump(grammar, profile, input, limits, options)
}

/// Convenience AST-dump entry point for Nexsim SV integration (`IEEE 1800-2017` profile).
pub fn parse_systemverilog_2017_ast_dump(
    input: &str,
    options: &AstDumpOptions,
) -> GrammarAstDumpOutcome {
    parse_grammar_profile_ast_dump(
        GrammarFamily::SystemVerilog,
        GrammarProfile::Sv2017,
        input,
        options,
    )
}

/// Convenience AST-dump entry point for Nexsim SV integration (`IEEE 1800-2023` profile).
pub fn parse_systemverilog_2023_ast_dump(
    input: &str,
    options: &AstDumpOptions,
) -> GrammarAstDumpOutcome {
    parse_grammar_profile_ast_dump(
        GrammarFamily::SystemVerilog,
        GrammarProfile::Sv2023,
        input,
        options,
    )
}

/// Convenience AST-dump entry point for Nexsim VHDL integration (`IEEE 1076-2019` profile).
pub fn parse_vhdl_1076_2019_ast_dump(
    input: &str,
    options: &AstDumpOptions,
) -> GrammarAstDumpOutcome {
    parse_grammar_profile_ast_dump(
        GrammarFamily::Vhdl,
        GrammarProfile::Vhdl1076_2019,
        input,
        options,
    )
}

/// Convenience AST-dump entry point for regex host integrations (`regex_default` profile).
pub fn parse_regex_default_ast_dump(
    input: &str,
    options: &AstDumpOptions,
) -> GrammarAstDumpOutcome {
    parse_grammar_profile_ast_dump(
        GrammarFamily::Regex,
        GrammarProfile::RegexDefault,
        input,
        options,
    )
}

/// Convenience AST-dump entry point with explicit limits for `IEEE 1800-2017`.
pub fn parse_systemverilog_2017_ast_dump_with_limits(
    input: &str,
    limits: &ParseLimits,
    options: &AstDumpOptions,
) -> GrammarAstDumpOutcome {
    parse_grammar_profile_ast_dump_with_limits(
        GrammarFamily::SystemVerilog,
        GrammarProfile::Sv2017,
        input,
        limits,
        options,
    )
}

/// Convenience AST-dump entry point with explicit limits for `IEEE 1800-2023`.
pub fn parse_systemverilog_2023_ast_dump_with_limits(
    input: &str,
    limits: &ParseLimits,
    options: &AstDumpOptions,
) -> GrammarAstDumpOutcome {
    parse_grammar_profile_ast_dump_with_limits(
        GrammarFamily::SystemVerilog,
        GrammarProfile::Sv2023,
        input,
        limits,
        options,
    )
}

/// Convenience AST-dump entry point with explicit limits for `IEEE 1076-2019`.
pub fn parse_vhdl_1076_2019_ast_dump_with_limits(
    input: &str,
    limits: &ParseLimits,
    options: &AstDumpOptions,
) -> GrammarAstDumpOutcome {
    parse_grammar_profile_ast_dump_with_limits(
        GrammarFamily::Vhdl,
        GrammarProfile::Vhdl1076_2019,
        input,
        limits,
        options,
    )
}

/// Convenience AST-dump entry point with explicit limits for `regex_default`.
pub fn parse_regex_default_ast_dump_with_limits(
    input: &str,
    limits: &ParseLimits,
    options: &AstDumpOptions,
) -> GrammarAstDumpOutcome {
    parse_grammar_profile_ast_dump_with_limits(
        GrammarFamily::Regex,
        GrammarProfile::RegexDefault,
        input,
        limits,
        options,
    )
}

fn run_parse(
    family: AnnotationFamily,
    backend: ParserBackend,
    input: &str,
    limits: &ParseLimits,
) -> Result<(), ParseDiagnostic> {
    validate_input_limits(input, limits)?;
    match (family, backend) {
        (AnnotationFamily::Return, ParserBackend::Bootstrap) => parse_bootstrap_return(input),
        (AnnotationFamily::Return, ParserBackend::Generated) => parse_generated_return(input),
        (AnnotationFamily::Semantic, ParserBackend::Bootstrap) => parse_bootstrap_semantic(input),
        (AnnotationFamily::Semantic, ParserBackend::Generated) => parse_generated_semantic(input),
    }
}

fn run_grammar_parse(
    grammar: GrammarFamily,
    profile: GrammarProfile,
    input: &str,
    limits: &ParseLimits,
) -> Result<(), ParseDiagnostic> {
    validate_input_limits(input, limits)?;
    validate_profile_match(grammar, profile)?;
    match grammar {
        GrammarFamily::SystemVerilog => {
            parse_generated_systemverilog(input, Some(profile.as_str()))
        }
        GrammarFamily::Vhdl => parse_generated_vhdl(input),
        GrammarFamily::Regex => parse_generated_regex(input),
    }
}

#[derive(Debug, Serialize)]
struct AstDumpTruncationDiagnostic {
    pgen_dump_contract_version: u32,
    kind: &'static str,
    truncated: bool,
    dump_kind: &'static str,
    max_bytes: usize,
    full_bytes: usize,
    reason: &'static str,
}

fn run_grammar_parse_ast_dump(
    grammar: GrammarFamily,
    profile: GrammarProfile,
    input: &str,
    limits: &ParseLimits,
    options: &AstDumpOptions,
) -> Result<AstDumpPayload, ParseDiagnostic> {
    validate_input_limits(input, limits)?;
    validate_ast_dump_options(options)?;
    validate_profile_match(grammar, profile)?;
    let ast_json = match grammar {
        GrammarFamily::SystemVerilog => {
            parse_generated_systemverilog_ast_json(input, Some(profile.as_str()))?
        }
        GrammarFamily::Vhdl => parse_generated_vhdl_ast_json(input)?,
        GrammarFamily::Regex => parse_generated_regex_ast_json(input)?,
    };
    encode_ast_dump_payload(&ast_json, options)
}

fn validate_ast_dump_options(options: &AstDumpOptions) -> Result<(), ParseDiagnostic> {
    if matches!(options.max_ast_bytes, Some(0)) {
        return Err(invalid_limits_diagnostic(
            "max_ast_bytes must be greater than 0",
        ));
    }
    Ok(())
}

fn canonicalize_json_value(value: JsonValue) -> JsonValue {
    match value {
        JsonValue::Array(values) => {
            JsonValue::Array(values.into_iter().map(canonicalize_json_value).collect())
        }
        JsonValue::Object(map) => {
            let mut entries = map.into_iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            let mut normalized = serde_json::Map::new();
            for (key, value) in entries {
                normalized.insert(key, canonicalize_json_value(value));
            }
            JsonValue::Object(normalized)
        }
        other => other,
    }
}

fn serialize_canonical_json<T: Serialize>(
    value: &T,
    pretty: bool,
) -> Result<String, ParseDiagnostic> {
    let normalized = canonicalize_json_value(serde_json::to_value(value).map_err(|err| {
        parse_failure_diagnostic(format!("failed to serialize parser AST payload: {}", err))
    })?);
    if pretty {
        serde_json::to_string_pretty(&normalized).map_err(|err| {
            parse_failure_diagnostic(format!("failed to encode parser AST payload: {}", err))
        })
    } else {
        serde_json::to_string(&normalized).map_err(|err| {
            parse_failure_diagnostic(format!("failed to encode parser AST payload: {}", err))
        })
    }
}

fn encode_ast_dump_payload(
    ast_json: &JsonValue,
    options: &AstDumpOptions,
) -> Result<AstDumpPayload, ParseDiagnostic> {
    let encoded = serialize_canonical_json(ast_json, options.pretty)?;
    let full_bytes = encoded.len();
    let max_ast_bytes = options.max_ast_bytes;
    if let Some(max_bytes) = max_ast_bytes {
        if full_bytes > max_bytes {
            let diagnostic = AstDumpTruncationDiagnostic {
                pgen_dump_contract_version: 1,
                kind: "pgen_ast_dump_truncation",
                truncated: true,
                dump_kind: "parser_return_ast",
                max_bytes,
                full_bytes,
                reason: "encoded parser AST JSON exceeded configured max bytes; payload omitted",
            };
            let encoded_diagnostic = serialize_canonical_json(&diagnostic, options.pretty)?;
            let emitted_bytes = encoded_diagnostic.len();
            if emitted_bytes > max_bytes {
                return Err(invalid_limits_diagnostic(format!(
                    "max_ast_bytes {} is too small to fit truncation diagnostics (requires at least {} bytes)",
                    max_bytes, emitted_bytes
                )));
            }
            return Ok(AstDumpPayload {
                dump_json: encoded_diagnostic,
                truncated: true,
                full_bytes,
                emitted_bytes,
            });
        }
    }

    Ok(AstDumpPayload {
        dump_json: encoded,
        truncated: false,
        full_bytes,
        emitted_bytes: full_bytes,
    })
}

fn validate_profile_match(
    grammar: GrammarFamily,
    profile: GrammarProfile,
) -> Result<(), ParseDiagnostic> {
    let valid = match grammar {
        GrammarFamily::SystemVerilog => {
            matches!(profile, GrammarProfile::Sv2017 | GrammarProfile::Sv2023)
        }
        GrammarFamily::Vhdl => matches!(profile, GrammarProfile::Vhdl1076_2019),
        GrammarFamily::Regex => matches!(profile, GrammarProfile::RegexDefault),
    };
    if valid {
        return Ok(());
    }

    Err(unsupported_profile_diagnostic(grammar, profile))
}

fn validate_input_limits(input: &str, limits: &ParseLimits) -> Result<(), ParseDiagnostic> {
    if limits.max_input_bytes == 0 {
        return Err(invalid_limits_diagnostic(
            "max_input_bytes must be greater than 0",
        ));
    }

    let input_len = input.len();
    if input_len > limits.max_input_bytes {
        return Err(input_too_large_diagnostic(format!(
            "input size {} bytes exceeds max_input_bytes {}",
            input_len, limits.max_input_bytes
        )));
    }

    Ok(())
}

fn parse_bootstrap_return(input: &str) -> Result<(), ParseDiagnostic> {
    let logger = runtime_logger("embedding.bootstrap.return_annotation");
    UnifiedReturnAST::parse_bootstrap(input, &logger)
        .map(|_| ())
        .map_err(|err| parse_failure_diagnostic(format!("bootstrap return parse failed: {}", err)))
}

fn parse_bootstrap_semantic(input: &str) -> Result<(), ParseDiagnostic> {
    let logger = runtime_logger("embedding.bootstrap.semantic_annotation");
    UnifiedSemanticAST::parse_bootstrap(input, &logger)
        .map(|_| ())
        .map_err(|err| {
            parse_failure_diagnostic(format!("bootstrap semantic parse failed: {}", err))
        })
}

fn parse_generated_return(input: &str) -> Result<(), ParseDiagnostic> {
    #[cfg(feature = "generated_parsers")]
    {
        use crate::generated_parsers::return_annotation::Return_annotationParser;
        let mut parser = Return_annotationParser::new(
            input,
            crate::ast_pipeline::runtime_logger_box("embedding.generated.return_annotation"),
        );
        return parser
            .parse_full_return_annotation()
            .map(|_| ())
            .map_err(|err| generated_parse_failure_diagnostic("return", input, err));
    }
    #[cfg(not(feature = "generated_parsers"))]
    {
        let _ = input;
        Err(backend_unavailable_diagnostic(
            "generated parser backend requires feature `generated_parsers`",
        ))
    }
}

fn parse_generated_semantic(input: &str) -> Result<(), ParseDiagnostic> {
    #[cfg(feature = "generated_parsers")]
    {
        use crate::generated_parsers::semantic_annotation::Semantic_annotationParser;
        let mut parser = Semantic_annotationParser::new(
            input,
            crate::ast_pipeline::runtime_logger_box("embedding.generated.semantic_annotation"),
        );
        return parser
            .parse_full_semantic_annotation()
            .map(|_| ())
            .map_err(|err| generated_parse_failure_diagnostic("semantic", input, err));
    }
    #[cfg(not(feature = "generated_parsers"))]
    {
        let _ = input;
        Err(backend_unavailable_diagnostic(
            "generated parser backend requires feature `generated_parsers`",
        ))
    }
}

fn parse_generated_systemverilog(
    input: &str,
    grammar_profile: Option<&str>,
) -> Result<(), ParseDiagnostic> {
    #[cfg(all(feature = "generated_parsers", has_generated_systemverilog_parser))]
    {
        use crate::generated_parsers::systemverilog::SystemverilogParser;
        let mut parser = SystemverilogParser::new(
            input,
            crate::ast_pipeline::runtime_logger_box("embedding.generated.systemverilog"),
        );
        parser.set_grammar_profile(grammar_profile);
        return parser
            .parse_full_systemverilog_file()
            .map(|_| ())
            .map_err(|err| generated_parse_failure_diagnostic("systemverilog", input, err));
    }
    #[cfg(not(all(feature = "generated_parsers", has_generated_systemverilog_parser)))]
    {
        let _ = (input, grammar_profile);
        Err(backend_unavailable_diagnostic(
            "systemverilog parser backend requires `generated_parsers` and generated/systemverilog_parser.rs",
        ))
    }
}

fn parse_generated_systemverilog_ast_json(
    input: &str,
    grammar_profile: Option<&str>,
) -> Result<JsonValue, ParseDiagnostic> {
    #[cfg(all(feature = "generated_parsers", has_generated_systemverilog_parser))]
    {
        use crate::generated_parsers::systemverilog::SystemverilogParser;
        let mut parser = SystemverilogParser::new(
            input,
            crate::ast_pipeline::runtime_logger_box("embedding.generated.systemverilog"),
        );
        parser.set_grammar_profile(grammar_profile);
        let parsed = parser
            .parse_full_systemverilog_file()
            .map_err(|err| generated_parse_failure_diagnostic("systemverilog", input, err))?;
        return serde_json::to_value(parsed).map_err(|err| {
            parse_failure_diagnostic(format!(
                "generated systemverilog AST serialization failed: {}",
                err
            ))
        });
    }
    #[cfg(not(all(feature = "generated_parsers", has_generated_systemverilog_parser)))]
    {
        let _ = (input, grammar_profile);
        Err(backend_unavailable_diagnostic(
            "systemverilog parser backend requires `generated_parsers` and generated/systemverilog_parser.rs",
        ))
    }
}

fn parse_generated_vhdl(input: &str) -> Result<(), ParseDiagnostic> {
    #[cfg(all(feature = "generated_parsers", has_generated_vhdl_parser))]
    {
        use crate::generated_parsers::vhdl::VhdlParser;
        let mut parser = VhdlParser::new(
            input,
            crate::ast_pipeline::runtime_logger_box("embedding.generated.vhdl"),
        );
        return parser
            .parse_full_vhdl_file()
            .map(|_| ())
            .map_err(|err| generated_parse_failure_diagnostic("vhdl", input, err));
    }
    #[cfg(not(all(feature = "generated_parsers", has_generated_vhdl_parser)))]
    {
        let _ = input;
        Err(backend_unavailable_diagnostic(
            "vhdl parser backend requires `generated_parsers` and generated/vhdl_parser.rs",
        ))
    }
}

fn parse_generated_vhdl_ast_json(input: &str) -> Result<JsonValue, ParseDiagnostic> {
    #[cfg(all(feature = "generated_parsers", has_generated_vhdl_parser))]
    {
        use crate::generated_parsers::vhdl::VhdlParser;
        let mut parser = VhdlParser::new(
            input,
            crate::ast_pipeline::runtime_logger_box("embedding.generated.vhdl"),
        );
        let parsed = parser
            .parse_full_vhdl_file()
            .map_err(|err| generated_parse_failure_diagnostic("vhdl", input, err))?;
        return serde_json::to_value(parsed).map_err(|err| {
            parse_failure_diagnostic(format!("generated vhdl AST serialization failed: {}", err))
        });
    }
    #[cfg(not(all(feature = "generated_parsers", has_generated_vhdl_parser)))]
    {
        let _ = input;
        Err(backend_unavailable_diagnostic(
            "vhdl parser backend requires `generated_parsers` and generated/vhdl_parser.rs",
        ))
    }
}

fn parse_generated_regex(input: &str) -> Result<(), ParseDiagnostic> {
    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    {
        return run_generated_regex_on_dedicated_stack(input, |owned_input| {
            use crate::generated_parsers::regex::RegexParser;
            let mut parser = RegexParser::new(
                &owned_input,
                crate::ast_pipeline::runtime_logger_box("embedding.generated.regex"),
            );
            parser
                .parse_full_regex()
                .map_err(|err| generated_parse_failure_diagnostic("regex", &owned_input, err))?;
            validate_regex_compile_contract(&owned_input)
                .map_err(|err| regex_compile_contract_diagnostic(&owned_input, err))
        });
    }
    #[cfg(not(all(feature = "generated_parsers", has_generated_regex_parser)))]
    {
        let _ = input;
        Err(backend_unavailable_diagnostic(
            "regex parser backend requires `generated_parsers` and generated/regex_parser.rs",
        ))
    }
}

fn parse_generated_regex_ast_json(input: &str) -> Result<JsonValue, ParseDiagnostic> {
    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    {
        return run_generated_regex_on_dedicated_stack(input, |owned_input| {
            use crate::generated_parsers::regex::RegexParser;
            let mut parser = RegexParser::new(
                &owned_input,
                crate::ast_pipeline::runtime_logger_box("embedding.generated.regex"),
            );
            let parsed = parser
                .parse_full_regex()
                .map_err(|err| generated_parse_failure_diagnostic("regex", &owned_input, err))?;
            validate_regex_compile_contract(&owned_input)
                .map_err(|err| regex_compile_contract_diagnostic(&owned_input, err))?;
            serde_json::to_value(parsed).map_err(|err| {
                parse_failure_diagnostic(format!(
                    "generated regex AST serialization failed: {}",
                    err
                ))
            })
        });
    }
    #[cfg(not(all(feature = "generated_parsers", has_generated_regex_parser)))]
    {
        let _ = input;
        Err(backend_unavailable_diagnostic(
            "regex parser backend requires `generated_parsers` and generated/regex_parser.rs",
        ))
    }
}

#[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
fn run_generated_regex_on_dedicated_stack<T, F>(
    input: &str,
    f: F,
) -> Result<T, ParseDiagnostic>
where
    T: Send + 'static,
    F: FnOnce(String) -> Result<T, ParseDiagnostic> + Send + 'static,
{
    let owned_input = input.to_string();
    let handle = std::thread::Builder::new()
        .name("pgen-generated-regex".to_string())
        .stack_size(GENERATED_REGEX_WORKER_STACK_BYTES)
        .spawn(move || f(owned_input))
        .map_err(|err| {
            parse_failure_diagnostic(format!(
                "failed to spawn generated regex worker thread: {}",
                err
            ))
        })?;
    handle
        .join()
        .map_err(|_| parse_failure_diagnostic("generated regex worker thread panicked"))?
}

#[cfg(feature = "generated_parsers")]
fn generated_backend_enabled() -> bool {
    true
}

#[cfg(not(feature = "generated_parsers"))]
fn generated_backend_enabled() -> bool {
    false
}

#[cfg(all(feature = "generated_parsers", has_generated_systemverilog_parser))]
fn systemverilog_generated_backend_enabled() -> bool {
    true
}

#[cfg(not(all(feature = "generated_parsers", has_generated_systemverilog_parser)))]
fn systemverilog_generated_backend_enabled() -> bool {
    false
}

#[cfg(all(feature = "generated_parsers", has_generated_vhdl_parser))]
fn vhdl_generated_backend_enabled() -> bool {
    true
}

#[cfg(not(all(feature = "generated_parsers", has_generated_vhdl_parser)))]
fn vhdl_generated_backend_enabled() -> bool {
    false
}

#[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
fn regex_generated_backend_enabled() -> bool {
    true
}

#[cfg(not(all(feature = "generated_parsers", has_generated_regex_parser)))]
fn regex_generated_backend_enabled() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Deserialize)]
    struct RegexParserIntegrationContractCase {
        name: String,
        input: String,
        #[serde(default)]
        required_rule_names: Vec<String>,
        #[serde(default)]
        forbidden_rule_names: Vec<String>,
        #[serde(default)]
        expected_rule_texts: std::collections::BTreeMap<String, Vec<String>>,
    }

    #[derive(Debug, Deserialize)]
    struct RegexParserIntegrationContractManifest {
        version: u32,
        integration_contract_version: String,
        parser_release_version: String,
        grammar: String,
        profile: String,
        required_generated_backend_flag: String,
        required_generated_backend_feature: String,
        required_generated_backend_artifact: String,
        generated_backend_env_override: String,
        ast_dump_schema_version: u32,
        frontend_json_artifact: String,
        frontend_json_role: String,
        stable_diagnostic_location_fields: Vec<String>,
        success_samples: Vec<RegexParserIntegrationContractCase>,
        failure_samples: Vec<RegexParserIntegrationContractCase>,
    }

    fn regex_parser_integration_contract_manifest() -> RegexParserIntegrationContractManifest {
        serde_json::from_str(include_str!(
            "../test_data/grammar_quality/regex_parser_integration_contract_v1.json"
        ))
        .expect("valid regex parser integration contract manifest")
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    fn regex_ast_dump_json(input: &str) -> serde_json::Value {
        parse_generated_regex_ast_json(input).expect("generated regex ast json")
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    fn collect_rule_spans(
        node: &serde_json::Value,
        rule_name: &str,
        spans: &mut Vec<(u64, u64)>,
    ) {
        match node {
            serde_json::Value::Array(values) => {
                for value in values {
                    collect_rule_spans(value, rule_name, spans);
                }
            }
            serde_json::Value::Object(object) => {
                if object
                    .get("rule_name")
                    .and_then(serde_json::Value::as_str)
                    == Some(rule_name)
                {
                    let span = object
                        .get("span")
                        .and_then(serde_json::Value::as_object)
                        .expect("regex AST dump node must have span");
                    let start = span
                        .get("start")
                        .and_then(serde_json::Value::as_u64)
                        .expect("regex AST dump node span.start must be numeric");
                    let end = span
                        .get("end")
                        .and_then(serde_json::Value::as_u64)
                        .expect("regex AST dump node span.end must be numeric");
                    spans.push((start, end));
                }

                if let Some(content) = object
                    .get("content")
                    .and_then(serde_json::Value::as_object)
                {
                    for payload in content.values() {
                        collect_rule_spans(payload, rule_name, spans);
                    }
                }
            }
            _ => {}
        }
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    fn regex_rule_spans(node: &serde_json::Value, rule_name: &str) -> Vec<(u64, u64)> {
        let mut spans = Vec::new();
        collect_rule_spans(node, rule_name, &mut spans);
        spans
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    fn regex_rule_texts(input: &str, node: &serde_json::Value, rule_name: &str) -> Vec<String> {
        regex_rule_spans(node, rule_name)
            .into_iter()
            .map(|(start, end)| {
                input.get(start as usize..end as usize)
                    .expect("regex AST dump span must map back to original input")
                    .to_string()
            })
            .collect()
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    fn assert_regex_ast_dump_node_schema(node: &serde_json::Value) {
        let object = node
            .as_object()
            .expect("regex AST dump node must be a JSON object");
        let rule_name = object
            .get("rule_name")
            .and_then(serde_json::Value::as_str)
            .expect("regex AST dump node must have rule_name");
        assert!(
            !rule_name.is_empty(),
            "regex AST dump node rule_name must be non-empty"
        );
        let span = object
            .get("span")
            .and_then(serde_json::Value::as_object)
            .expect("regex AST dump node must have span");
        let start = span
            .get("start")
            .and_then(serde_json::Value::as_u64)
            .expect("regex AST dump node span.start must be numeric");
        let end = span
            .get("end")
            .and_then(serde_json::Value::as_u64)
            .expect("regex AST dump node span.end must be numeric");
        assert!(end >= start, "regex AST dump node span must be ordered");

        let content = object
            .get("content")
            .and_then(serde_json::Value::as_object)
            .expect("regex AST dump node must have externally tagged content");
        assert_eq!(
            content.len(),
            1,
            "regex AST dump node content must have exactly one active variant"
        );
        let (variant, payload) = content.iter().next().expect("content variant");
        match variant.as_str() {
            "Terminal" | "TransformedTerminal" => {
                payload
                    .as_str()
                    .expect("terminal payload must be encoded as a string");
            }
            "Sequence" => {
                for child in payload
                    .as_array()
                    .expect("sequence payload must be an array")
                {
                    assert_regex_ast_dump_node_schema(child);
                }
            }
            "Alternative" => assert_regex_ast_dump_node_schema(payload),
            "Quantified" => {
                let quantified = payload
                    .as_array()
                    .expect("quantified payload must be an array");
                assert_eq!(
                    quantified.len(),
                    2,
                    "quantified payload must contain node array and quantifier marker"
                );
                for child in quantified[0]
                    .as_array()
                    .expect("quantified payload first element must be an array")
                {
                    assert_regex_ast_dump_node_schema(child);
                }
                quantified[1]
                    .as_str()
                    .expect("quantified payload second element must be a quantifier string");
            }
            other => panic!("unexpected regex AST dump content variant {}", other),
        }
    }

    #[test]
    fn embedding_api_contract_versions_are_stable() {
        let contract = embedding_api_contract();
        assert_eq!(contract.api_version, EMBEDDING_API_VERSION);
        assert_eq!(contract.schema_version, EMBEDDING_API_SCHEMA_VERSION);
        assert!(
            contract
                .supported_families
                .contains(&AnnotationFamily::Return)
        );
        assert!(
            contract
                .supported_families
                .contains(&AnnotationFamily::Semantic)
        );
    }

    #[test]
    fn bootstrap_return_smoke_parse_succeeds() {
        let outcome = parse_annotation(
            AnnotationFamily::Return,
            ParserBackend::Bootstrap,
            "$1.property",
        );
        assert_eq!(outcome.status, ParseStatus::Success);
        assert!(outcome.diagnostic.is_none());
    }

    #[test]
    fn bootstrap_semantic_smoke_parse_succeeds() {
        let outcome = parse_annotation(
            AnnotationFamily::Semantic,
            ParserBackend::Bootstrap,
            "@type: \"Expression\"",
        );
        assert_eq!(outcome.status, ParseStatus::Success);
        assert!(outcome.diagnostic.is_none());
    }

    #[test]
    fn parse_with_limits_rejects_input_that_exceeds_bound() {
        let limits = ParseLimits { max_input_bytes: 4 };
        let outcome = parse_annotation_with_limits(
            AnnotationFamily::Return,
            ParserBackend::Bootstrap,
            "$12345",
            &limits,
        );
        assert_eq!(outcome.status, ParseStatus::Failure);
        let diagnostic = outcome
            .diagnostic
            .as_ref()
            .expect("expected size limit diagnostic");
        assert_eq!(diagnostic.code, "E_INPUT_TOO_LARGE");
    }

    #[test]
    fn parse_with_limits_rejects_invalid_zero_max_input_bytes() {
        let limits = ParseLimits { max_input_bytes: 0 };
        let outcome = parse_annotation_with_limits(
            AnnotationFamily::Semantic,
            ParserBackend::Bootstrap,
            "@type: \"Expression\"",
            &limits,
        );
        assert_eq!(outcome.status, ParseStatus::Failure);
        let diagnostic = outcome
            .diagnostic
            .as_ref()
            .expect("expected invalid limits diagnostic");
        assert_eq!(diagnostic.code, "E_INVALID_LIMITS");
    }

    #[test]
    fn parse_diagnostic_location_is_one_based_and_clamped_to_utf8_boundaries() {
        let location = parse_diagnostic_location("a\néz", 4);
        assert_eq!(location.byte_offset, 4);
        assert_eq!(location.line, 2);
        assert_eq!(location.column, 2);

        let clamped = parse_diagnostic_location("a\néz", 3);
        assert_eq!(clamped.byte_offset, 2);
        assert_eq!(clamped.line, 2);
        assert_eq!(clamped.column, 1);
    }

    #[test]
    fn generated_parse_failure_diagnostic_builds_machine_localizable_payload() {
        let diagnostic = generated_parse_failure_diagnostic(
            "regex",
            "a\n(",
            ParseError::Backtrack { position: 2 },
        );
        assert_eq!(diagnostic.code, "E_PARSE_FAILURE");
        let location = diagnostic
            .location
            .as_ref()
            .expect("generated parse failure should carry location");
        assert_eq!(location.byte_offset, 2);
        assert_eq!(location.line, 2);
        assert_eq!(location.column, 1);
    }

    #[test]
    fn parse_annotation_default_limits_allow_normal_input() {
        let outcome = parse_annotation(
            AnnotationFamily::Semantic,
            ParserBackend::Bootstrap,
            "@type: \"Expression\"",
        );
        assert_eq!(outcome.status, ParseStatus::Success);
        assert!(outcome.diagnostic.is_none());
    }

    #[test]
    fn result_wrapper_succeeds_for_bootstrap_return() {
        let result = parse_annotation_result(
            AnnotationFamily::Return,
            ParserBackend::Bootstrap,
            "$1.property",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn named_annotation_entry_rejects_unknown_family() {
        let outcome = parse_annotation_named("unknown_family", "bootstrap", "$1");
        assert_eq!(outcome.status, ParseStatus::Failure);
        let diagnostic = outcome
            .diagnostic
            .as_ref()
            .expect("expected invalid-argument diagnostic");
        assert_eq!(diagnostic.code, "E_INVALID_ARGUMENT");
    }

    #[test]
    fn named_grammar_entry_rejects_unknown_profile() {
        let outcome =
            parse_grammar_profile_named("systemverilog", "unknown_profile", "module m; endmodule");
        assert_eq!(outcome.status, ParseStatus::Failure);
        let diagnostic = outcome
            .diagnostic
            .as_ref()
            .expect("expected invalid-argument diagnostic");
        assert_eq!(diagnostic.code, "E_INVALID_ARGUMENT");
    }

    #[test]
    fn from_str_and_as_str_aliases_are_stable() {
        assert_eq!(
            AnnotationFamily::from_str("return_annotation")
                .expect("family alias")
                .as_str(),
            "return"
        );
        assert_eq!(
            ParserBackend::from_str("generated")
                .expect("backend alias")
                .as_str(),
            "generated"
        );
        assert_eq!(
            GrammarFamily::from_str("sv")
                .expect("grammar alias")
                .as_str(),
            "systemverilog"
        );
        assert_eq!(
            GrammarFamily::from_str("regex")
                .expect("grammar alias")
                .as_str(),
            "regex"
        );
        assert_eq!(
            GrammarProfile::from_str("ieee1800-2023")
                .expect("profile alias")
                .as_str(),
            "sv_2023"
        );
        assert_eq!(
            GrammarProfile::from_str("regex")
                .expect("profile alias")
                .as_str(),
            "regex_default"
        );
    }

    #[test]
    fn parser_embedding_contract_exposes_profile_matrix() {
        let contract = parser_embedding_api_contract();
        assert_eq!(contract.api_version, EMBEDDING_API_VERSION);
        assert_eq!(contract.schema_version, EMBEDDING_API_SCHEMA_VERSION);
        assert_eq!(
            contract.input_ownership_model,
            InputOwnershipModel::BorrowedStr
        );
        assert_eq!(
            contract.parse_session_model,
            ParseSessionModel::StatelessPerCall
        );
        assert!(contract.zero_copy_input_boundary);
        assert_eq!(
            contract.stable_diagnostic_codes,
            vec![
                "E_BACKEND_UNAVAILABLE".to_string(),
                "E_INPUT_TOO_LARGE".to_string(),
                "E_INVALID_ARGUMENT".to_string(),
                "E_INVALID_LIMITS".to_string(),
                "E_PARSE_FAILURE".to_string(),
                "E_UNSUPPORTED_PROFILE".to_string(),
            ]
        );
        assert_eq!(
            contract.stable_diagnostic_location_fields,
            vec![
                "byte_offset".to_string(),
                "line".to_string(),
                "column".to_string(),
            ]
        );
        assert!(
            contract
                .supported_grammars
                .contains(&GrammarFamily::SystemVerilog)
        );
        assert!(contract.supported_grammars.contains(&GrammarFamily::Vhdl));
        assert!(
            contract
                .supported_profiles
                .contains(&GrammarProfile::Sv2017)
        );
        assert!(
            contract
                .supported_profiles
                .contains(&GrammarProfile::Sv2023)
        );
        assert!(
            contract
                .supported_profiles
                .contains(&GrammarProfile::Vhdl1076_2019)
        );
        assert!(contract.supported_grammars.contains(&GrammarFamily::Regex));
        assert!(
            contract
                .supported_profiles
                .contains(&GrammarProfile::RegexDefault)
        );
        assert_eq!(
            contract.regex_integration_contract_version,
            REGEX_PARSER_INTEGRATION_CONTRACT_VERSION
        );
        assert_eq!(
            contract.regex_parser_release_version,
            REGEX_PARSER_RELEASE_VERSION
        );
        assert_eq!(
            contract.regex_ast_dump_schema_version,
            REGEX_AST_DUMP_SCHEMA_VERSION
        );
        assert_eq!(
            contract.regex_generated_backend_required_feature,
            "generated_parsers"
        );
        assert_eq!(
            contract.regex_generated_backend_required_artifact,
            "generated/regex_parser.rs"
        );
        assert_eq!(
            contract.regex_generated_backend_env_override,
            "PGEN_REGEX_PARSER_PATH"
        );
        assert_eq!(
            contract.regex_frontend_json_artifact,
            "generated/regex.json"
        );
    }

    #[test]
    fn regex_parser_integration_contract_metadata_is_stable() {
        let manifest = regex_parser_integration_contract_manifest();
        let contract = parser_embedding_api_contract();

        assert_eq!(manifest.version, 1);
        assert_eq!(
            manifest.integration_contract_version,
            REGEX_PARSER_INTEGRATION_CONTRACT_VERSION
        );
        assert_eq!(
            manifest.parser_release_version,
            REGEX_PARSER_RELEASE_VERSION
        );
        assert_eq!(manifest.grammar, "regex");
        assert_eq!(manifest.profile, "regex_default");
        assert_eq!(
            manifest.required_generated_backend_flag,
            "supports_regex_generated_backend"
        );
        assert_eq!(
            manifest.required_generated_backend_feature,
            "generated_parsers"
        );
        assert_eq!(
            manifest.required_generated_backend_artifact,
            "generated/regex_parser.rs"
        );
        assert_eq!(
            manifest.generated_backend_env_override,
            "PGEN_REGEX_PARSER_PATH"
        );
        assert_eq!(
            manifest.ast_dump_schema_version,
            REGEX_AST_DUMP_SCHEMA_VERSION
        );
        assert_eq!(manifest.frontend_json_artifact, "generated/regex.json");
        assert!(
            manifest
                .frontend_json_role
                .contains("not a stable downstream runtime input"),
            "frontend JSON role should explain non-contractual downstream status"
        );
        assert_eq!(
            manifest.stable_diagnostic_location_fields,
            vec![
                "byte_offset".to_string(),
                "line".to_string(),
                "column".to_string(),
            ]
        );
        assert_eq!(manifest.success_samples.len(), 52);
        assert_eq!(manifest.failure_samples.len(), 8);
        assert_eq!(manifest.success_samples[0].name, "empty_regex");
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "named_recursion_conditional")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "bare_recursion_conditional")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "numeric_recursion_conditional")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "version_conditional")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "version_conditional_whitespace_and_missing_minor")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "returned_capture_numeric_subroutine")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "returned_capture_named_subroutine")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "braced_octal_escape")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "numeric_angle_subroutine_ref")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "unicode_emoji_literal")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "malformed_posix_open_bracket_literal_fallback")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "posix_space_class_item_ast")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "posix_blank_class_item_ast")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "mixed_literal_and_posix_digit_class_item_ast")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "mixed_literal_posix_digit_literal_class_item_ast")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "posix_digit_then_literal_dash_class_item_ast")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "quoted_literal_metacharacters")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "mark_shorthand_payload_with_open_paren")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "prune_directive_payload_with_open_paren")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "directive_payload_general_mark_prune_skip")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "then_directive_payload_with_open_paren")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "alpha_lookahead_condition_short")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "unicode_accented_literal")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "nested_capturing_groups_50")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "lua_embedded_code_block")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "rhai_embedded_code_block")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "native_embedded_code_block")
        );
        assert!(
            manifest
                .success_samples
                .iter()
                .any(|sample| sample.name == "wasm_embedded_code_block")
        );
        assert_eq!(manifest.failure_samples[0].name, "unbalanced_group");
        assert!(contract.supported_grammars.contains(&GrammarFamily::Regex));
        assert!(
            contract
                .supported_profiles
                .contains(&GrammarProfile::RegexDefault)
        );
        assert_eq!(
            contract.regex_integration_contract_version,
            manifest.integration_contract_version
        );
        assert_eq!(
            contract.regex_parser_release_version,
            manifest.parser_release_version
        );
        assert_eq!(
            contract.regex_generated_backend_required_feature,
            manifest.required_generated_backend_feature
        );
        assert_eq!(
            contract.regex_generated_backend_required_artifact,
            manifest.required_generated_backend_artifact
        );
        assert_eq!(
            contract.regex_generated_backend_env_override,
            manifest.generated_backend_env_override
        );
        assert_eq!(
            contract.regex_ast_dump_schema_version,
            manifest.ast_dump_schema_version
        );
        assert_eq!(
            contract.regex_frontend_json_artifact,
            manifest.frontend_json_artifact
        );
        assert_eq!(
            contract.regex_frontend_json_role,
            manifest.frontend_json_role
        );
        assert_eq!(
            contract.stable_diagnostic_location_fields,
            manifest.stable_diagnostic_location_fields
        );
    }

    #[test]
    fn regex_parser_integration_contract_named_surface_accepts_regex_aliases() {
        let outcome = parse_grammar_profile_named("regex", "regex", "a|b");
        let maybe_code = outcome.diagnostic.as_ref().map(|diag| diag.code.as_str());

        assert_eq!(outcome.grammar, "regex");
        assert_eq!(outcome.profile, "regex");
        assert_ne!(maybe_code, Some("E_INVALID_ARGUMENT"));
        assert_ne!(maybe_code, Some("E_UNSUPPORTED_PROFILE"));
    }

    #[test]
    fn regex_parser_integration_contract_enforces_limits_before_backend_availability() {
        let result =
            parse_regex_default_with_limits_result("a|b", &ParseLimits { max_input_bytes: 1 });
        let diagnostic = result.expect_err("expected input limit failure");
        assert_eq!(diagnostic.code, "E_INPUT_TOO_LARGE");
    }

    #[test]
    fn parser_embedding_convenience_sv2017_matches_profile_entry_point() {
        let input = "module m; endmodule";
        let convenience = parse_systemverilog_2017(input);
        let profile =
            parse_grammar_profile(GrammarFamily::SystemVerilog, GrammarProfile::Sv2017, input);

        assert_eq!(convenience.status, profile.status);
        assert_eq!(
            convenience
                .diagnostic
                .as_ref()
                .map(|diag| diag.code.as_str()),
            profile.diagnostic.as_ref().map(|diag| diag.code.as_str())
        );
    }

    #[test]
    fn parser_embedding_convenience_sv2023_matches_profile_entry_point() {
        let input = "module m; endmodule";
        let convenience = parse_systemverilog_2023(input);
        let profile =
            parse_grammar_profile(GrammarFamily::SystemVerilog, GrammarProfile::Sv2023, input);

        assert_eq!(convenience.status, profile.status);
        assert_eq!(
            convenience
                .diagnostic
                .as_ref()
                .map(|diag| diag.code.as_str()),
            profile.diagnostic.as_ref().map(|diag| diag.code.as_str())
        );
    }

    #[test]
    fn parser_embedding_convenience_vhdl_matches_profile_entry_point() {
        let input = "entity e is end entity;";
        let convenience = parse_vhdl_1076_2019(input);
        let profile =
            parse_grammar_profile(GrammarFamily::Vhdl, GrammarProfile::Vhdl1076_2019, input);

        assert_eq!(convenience.status, profile.status);
        assert_eq!(
            convenience
                .diagnostic
                .as_ref()
                .map(|diag| diag.code.as_str()),
            profile.diagnostic.as_ref().map(|diag| diag.code.as_str())
        );
    }

    #[test]
    fn parser_embedding_convenience_regex_matches_profile_entry_point() {
        let input = "a|b";
        let convenience = parse_regex_default(input);
        let profile =
            parse_grammar_profile(GrammarFamily::Regex, GrammarProfile::RegexDefault, input);

        assert_eq!(convenience.status, profile.status);
        assert_eq!(
            convenience
                .diagnostic
                .as_ref()
                .map(|diag| diag.code.as_str()),
            profile.diagnostic.as_ref().map(|diag| diag.code.as_str())
        );
    }

    #[test]
    fn parser_embedding_rejects_profile_grammar_mismatch() {
        let outcome = parse_grammar_profile(
            GrammarFamily::SystemVerilog,
            GrammarProfile::Vhdl1076_2019,
            "module m; endmodule",
        );
        assert_eq!(outcome.status, ParseStatus::Failure);
        let diagnostic = outcome
            .diagnostic
            .as_ref()
            .expect("expected unsupported-profile diagnostic");
        assert_eq!(diagnostic.code, "E_UNSUPPORTED_PROFILE");
    }

    #[test]
    fn parser_embedding_rejects_regex_profile_grammar_mismatch() {
        let outcome = parse_grammar_profile(GrammarFamily::Regex, GrammarProfile::Sv2017, "a|b");
        assert_eq!(outcome.status, ParseStatus::Failure);
        let diagnostic = outcome
            .diagnostic
            .as_ref()
            .expect("expected unsupported-profile diagnostic");
        assert_eq!(diagnostic.code, "E_UNSUPPORTED_PROFILE");
    }

    #[test]
    fn parser_embedding_enforces_input_limits() {
        let limits = ParseLimits { max_input_bytes: 4 };
        let outcome = parse_grammar_profile_with_limits(
            GrammarFamily::SystemVerilog,
            GrammarProfile::Sv2017,
            "module m; endmodule",
            &limits,
        );
        assert_eq!(outcome.status, ParseStatus::Failure);
        let diagnostic = outcome
            .diagnostic
            .as_ref()
            .expect("expected size-limit diagnostic");
        assert_eq!(diagnostic.code, "E_INPUT_TOO_LARGE");
    }

    #[test]
    fn ast_dump_options_default_is_compact_unbounded() {
        let options = AstDumpOptions::default();
        assert!(!options.pretty);
        assert!(options.max_ast_bytes.is_none());
    }

    #[test]
    fn parser_embedding_ast_dump_rejects_zero_max_ast_bytes() {
        let outcome = parse_grammar_profile_ast_dump(
            GrammarFamily::SystemVerilog,
            GrammarProfile::Sv2017,
            "module m; endmodule",
            &AstDumpOptions {
                pretty: false,
                max_ast_bytes: Some(0),
            },
        );
        assert_eq!(outcome.status, ParseStatus::Failure);
        let diagnostic = outcome
            .diagnostic
            .as_ref()
            .expect("expected invalid limits diagnostic");
        assert_eq!(diagnostic.code, "E_INVALID_LIMITS");
    }

    #[test]
    fn parser_embedding_ast_dump_named_rejects_unknown_profile() {
        let outcome = parse_grammar_profile_ast_dump_named(
            "systemverilog",
            "unknown_profile",
            "module m; endmodule",
            &AstDumpOptions::default(),
        );
        assert_eq!(outcome.status, ParseStatus::Failure);
        let diagnostic = outcome
            .diagnostic
            .as_ref()
            .expect("expected invalid argument diagnostic");
        assert_eq!(diagnostic.code, "E_INVALID_ARGUMENT");
    }

    #[test]
    fn ast_dump_payload_truncates_with_diagnostics_envelope() {
        let payload = serde_json::json!({
            "payload": "x".repeat(4096),
        });
        let ast_dump = encode_ast_dump_payload(
            &payload,
            &AstDumpOptions {
                pretty: false,
                max_ast_bytes: Some(256),
            },
        )
        .expect("ast dump encoding");
        assert!(ast_dump.truncated);
        assert!(ast_dump.full_bytes > 256);
        let envelope: serde_json::Value =
            serde_json::from_str(&ast_dump.dump_json).expect("truncation envelope json");
        assert_eq!(envelope["kind"], "pgen_ast_dump_truncation");
        assert_eq!(envelope["dump_kind"], "parser_return_ast");
        assert_eq!(envelope["max_bytes"], 256);
    }

    #[cfg(not(all(feature = "generated_parsers", has_generated_systemverilog_parser)))]
    #[test]
    fn parser_embedding_reports_missing_systemverilog_backend() {
        let outcome = parse_grammar_profile(
            GrammarFamily::SystemVerilog,
            GrammarProfile::Sv2017,
            "module m; endmodule",
        );
        assert_eq!(outcome.status, ParseStatus::Failure);
        let diagnostic = outcome
            .diagnostic
            .as_ref()
            .expect("expected backend-unavailable diagnostic");
        assert_eq!(diagnostic.code, "E_BACKEND_UNAVAILABLE");
    }

    #[cfg(not(all(feature = "generated_parsers", has_generated_vhdl_parser)))]
    #[test]
    fn parser_embedding_reports_missing_vhdl_backend() {
        let outcome = parse_grammar_profile(
            GrammarFamily::Vhdl,
            GrammarProfile::Vhdl1076_2019,
            "entity e is end entity;",
        );
        assert_eq!(outcome.status, ParseStatus::Failure);
        let diagnostic = outcome
            .diagnostic
            .as_ref()
            .expect("expected backend-unavailable diagnostic");
        assert_eq!(diagnostic.code, "E_BACKEND_UNAVAILABLE");
    }

    #[cfg(not(all(feature = "generated_parsers", has_generated_regex_parser)))]
    #[test]
    fn parser_embedding_reports_missing_regex_backend() {
        let outcome =
            parse_grammar_profile(GrammarFamily::Regex, GrammarProfile::RegexDefault, "a|b");
        assert_eq!(outcome.status, ParseStatus::Failure);
        let diagnostic = outcome
            .diagnostic
            .as_ref()
            .expect("expected backend-unavailable diagnostic");
        assert_eq!(diagnostic.code, "E_BACKEND_UNAVAILABLE");
    }

    #[cfg(not(all(feature = "generated_parsers", has_generated_regex_parser)))]
    #[test]
    fn regex_parser_integration_contract_reports_backend_unavailable_without_generated_backend() {
        let manifest = regex_parser_integration_contract_manifest();
        let diagnostic = parse_regex_default_result(&manifest.success_samples[0].input)
            .expect_err("expected backend-unavailable diagnostic");
        assert_eq!(diagnostic.code, "E_BACKEND_UNAVAILABLE");
    }

    #[cfg(all(feature = "generated_parsers", has_generated_systemverilog_parser))]
    #[test]
    fn parser_embedding_uses_systemverilog_generated_backend_when_available() {
        let outcome = parse_grammar_profile(
            GrammarFamily::SystemVerilog,
            GrammarProfile::Sv2017,
            "module m; endmodule",
        );
        let maybe_code = outcome.diagnostic.as_ref().map(|diag| diag.code.as_str());
        assert_ne!(maybe_code, Some("E_BACKEND_UNAVAILABLE"));
    }

    #[cfg(all(feature = "generated_parsers", has_generated_vhdl_parser))]
    #[test]
    fn parser_embedding_uses_vhdl_generated_backend_when_available() {
        let outcome = parse_grammar_profile(
            GrammarFamily::Vhdl,
            GrammarProfile::Vhdl1076_2019,
            "entity e is end entity;",
        );
        let maybe_code = outcome.diagnostic.as_ref().map(|diag| diag.code.as_str());
        assert_ne!(maybe_code, Some("E_BACKEND_UNAVAILABLE"));
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    #[test]
    fn parser_embedding_uses_regex_generated_backend_when_available() {
        let outcome =
            parse_grammar_profile(GrammarFamily::Regex, GrammarProfile::RegexDefault, "a|b");
        let maybe_code = outcome.diagnostic.as_ref().map(|diag| diag.code.as_str());
        assert_ne!(maybe_code, Some("E_BACKEND_UNAVAILABLE"));
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    #[test]
    fn regex_parser_integration_contract_accepts_declared_success_samples() {
        let manifest = regex_parser_integration_contract_manifest();

        for sample in &manifest.success_samples {
            if let Err(diagnostic) = parse_regex_default_result(&sample.input) {
                panic!(
                    "expected regex success sample '{}' to parse, got {}: {}",
                    sample.name, diagnostic.code, diagnostic.message
                );
            }
        }
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    #[test]
    fn regex_parser_integration_contract_enforces_declared_ast_shape_for_success_samples() {
        let manifest = regex_parser_integration_contract_manifest();

        for sample in &manifest.success_samples {
            if sample.required_rule_names.is_empty()
                && sample.forbidden_rule_names.is_empty()
                && sample.expected_rule_texts.is_empty()
            {
                continue;
            }

            let parsed = regex_ast_dump_json(&sample.input);
            for rule_name in &sample.required_rule_names {
                assert!(
                    !regex_rule_spans(&parsed, rule_name).is_empty(),
                    "expected regex success sample '{}' to contain AST rule '{}'",
                    sample.name,
                    rule_name
                );
            }
            for rule_name in &sample.forbidden_rule_names {
                assert!(
                    regex_rule_spans(&parsed, rule_name).is_empty(),
                    "expected regex success sample '{}' to forbid AST rule '{}'",
                    sample.name,
                    rule_name
                );
            }
            for (rule_name, expected_texts) in &sample.expected_rule_texts {
                assert_eq!(
                    regex_rule_texts(&sample.input, &parsed, rule_name),
                    *expected_texts,
                    "expected regex success sample '{}' to preserve exact text for AST rule '{}'",
                    sample.name,
                    rule_name
                );
            }
        }
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    #[test]
    fn regex_parser_integration_contract_rejects_declared_failure_samples() {
        let manifest = regex_parser_integration_contract_manifest();

        for sample in &manifest.failure_samples {
            let diagnostic = parse_regex_default_result(&sample.input)
                .expect_err("expected regex failure sample to fail");
            assert_eq!(
                diagnostic.code, "E_PARSE_FAILURE",
                "expected failure sample '{}' to produce parse failure",
                sample.name
            );
        }
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    #[test]
    fn regex_parser_integration_contract_failures_are_machine_localizable() {
        let manifest = regex_parser_integration_contract_manifest();

        for sample in &manifest.failure_samples {
            let diagnostic = parse_regex_default_result(&sample.input)
                .expect_err("expected regex failure sample to fail");
            let location = diagnostic
                .location
                .as_ref()
                .expect("generated regex failures should expose structured location");
            assert!(location.line >= 1, "line must be one-based");
            assert!(location.column >= 1, "column must be one-based");
            assert!(
                location.byte_offset <= sample.input.len(),
                "byte_offset must not point past the original input"
            );
        }
    }

    #[cfg(not(all(feature = "generated_parsers", has_generated_systemverilog_parser)))]
    #[test]
    fn parser_embedding_ast_dump_reports_missing_systemverilog_backend() {
        let outcome =
            parse_systemverilog_2017_ast_dump("module m; endmodule", &AstDumpOptions::default());
        assert_eq!(outcome.status, ParseStatus::Failure);
        let diagnostic = outcome
            .diagnostic
            .as_ref()
            .expect("expected backend-unavailable diagnostic");
        assert_eq!(diagnostic.code, "E_BACKEND_UNAVAILABLE");
    }

    #[cfg(all(feature = "generated_parsers", has_generated_systemverilog_parser))]
    #[test]
    fn parser_embedding_ast_dump_uses_systemverilog_generated_backend_when_available() {
        let outcome =
            parse_systemverilog_2017_ast_dump("module m; endmodule", &AstDumpOptions::default());
        let maybe_code = outcome.diagnostic.as_ref().map(|diag| diag.code.as_str());
        assert_ne!(maybe_code, Some("E_BACKEND_UNAVAILABLE"));
        if outcome.status == ParseStatus::Success {
            let ast_dump = outcome.ast_dump.expect("ast dump payload");
            let parsed: serde_json::Value =
                serde_json::from_str(&ast_dump.dump_json).expect("dump json");
            assert!(parsed.is_object());
        }
    }

    #[cfg(not(all(feature = "generated_parsers", has_generated_vhdl_parser)))]
    #[test]
    fn parser_embedding_ast_dump_reports_missing_vhdl_backend() {
        let outcome =
            parse_vhdl_1076_2019_ast_dump("entity e is end entity;", &AstDumpOptions::default());
        assert_eq!(outcome.status, ParseStatus::Failure);
        let diagnostic = outcome
            .diagnostic
            .as_ref()
            .expect("expected backend-unavailable diagnostic");
        assert_eq!(diagnostic.code, "E_BACKEND_UNAVAILABLE");
    }

    #[cfg(not(all(feature = "generated_parsers", has_generated_regex_parser)))]
    #[test]
    fn parser_embedding_ast_dump_reports_missing_regex_backend() {
        let outcome = parse_regex_default_ast_dump("a|b", &AstDumpOptions::default());
        assert_eq!(outcome.status, ParseStatus::Failure);
        let diagnostic = outcome
            .diagnostic
            .as_ref()
            .expect("expected backend-unavailable diagnostic");
        assert_eq!(diagnostic.code, "E_BACKEND_UNAVAILABLE");
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    #[test]
    fn regex_parser_integration_contract_ast_dump_succeeds_for_contract_sample() {
        let manifest = regex_parser_integration_contract_manifest();
        let outcome = parse_regex_default_ast_dump(
            &manifest.success_samples[1].input,
            &AstDumpOptions::default(),
        );

        assert_eq!(outcome.status, ParseStatus::Success);
        let ast_dump = outcome.ast_dump.expect("ast dump payload");
        let parsed: serde_json::Value =
            serde_json::from_str(&ast_dump.dump_json).expect("dump json");
        assert!(parsed.is_object());
        assert_regex_ast_dump_node_schema(&parsed);
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    #[test]
    fn regex_parser_integration_contract_classifies_whole_pattern_recursion_as_subroutine_call() {
        let parsed = regex_ast_dump_json("(?R)");

        assert_eq!(regex_rule_spans(&parsed, "subroutine_call"), vec![(0, 4)]);
        assert_eq!(regex_rule_spans(&parsed, "subroutine_target"), vec![(2, 3)]);
        assert!(
            regex_rule_spans(&parsed, "inline_modifiers").is_empty(),
            "whole-pattern recursion must not be misclassified as inline modifiers"
        );
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    #[test]
    fn regex_parser_integration_contract_classifies_returned_capture_subroutine() {
        let input = "(?1(1))";
        let parsed = regex_ast_dump_json(input);

        assert_eq!(regex_rule_spans(&parsed, "subroutine_call"), vec![(0, 7)]);
        assert_eq!(
            regex_rule_spans(&parsed, "returned_capture_subroutine"),
            vec![(2, 6)]
        );
        assert_eq!(regex_rule_spans(&parsed, "subroutine_target"), vec![(2, 3)]);
        assert_eq!(
            regex_rule_spans(&parsed, "returned_capture_group_list"),
            vec![(3, 6)]
        );
        assert_eq!(
            regex_rule_texts(input, &parsed, "returned_capture_group"),
            vec!["1"]
        );
        assert_eq!(
            regex_rule_texts(input, &parsed, "signed_digits"),
            vec!["1", "1"]
        );
        assert!(
            regex_rule_spans(&parsed, "inline_modifiers").is_empty(),
            "returned-capture subroutine must not be misclassified as inline modifiers"
        );
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    #[test]
    fn regex_parser_integration_contract_classifies_numeric_backreferences() {
        let parsed = regex_ast_dump_json("(a)\\1");

        assert_eq!(regex_rule_spans(&parsed, "backreference"), vec![(3, 5)]);
        assert!(
            regex_rule_spans(&parsed, "escape").is_empty(),
            "numeric backreference must not be reported as a generic escape"
        );
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    #[test]
    fn regex_parser_integration_contract_classifies_braced_octal_escape() {
        let parsed = regex_ast_dump_json("\\o{101}");

        assert_eq!(regex_rule_spans(&parsed, "escape"), vec![(0, 7)]);
        assert_eq!(regex_rule_spans(&parsed, "octal_escape"), vec![(1, 7)]);
        assert_eq!(regex_rule_spans(&parsed, "octal_digits"), vec![(3, 6)]);
        assert!(
            regex_rule_spans(&parsed, "simple_escape").is_empty(),
            "braced octal escape must not degrade into a simple escape"
        );
        assert!(
            regex_rule_spans(&parsed, "quantifier").is_empty(),
            "braced octal escape must not spill into a counted quantifier"
        );
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    #[test]
    fn regex_parser_integration_contract_classifies_numeric_angle_subroutine_ref() {
        let parsed = regex_ast_dump_json("\\g<1>");

        assert_eq!(regex_rule_spans(&parsed, "backreference"), vec![(0, 5)]);
        assert_eq!(regex_rule_spans(&parsed, "subroutine_ref"), vec![(2, 5)]);
        assert_eq!(regex_rule_spans(&parsed, "signed_digits"), vec![(3, 4)]);
        assert!(
            regex_rule_spans(&parsed, "escape").is_empty(),
            "numeric angle subroutine ref must not degrade into a generic escape"
        );
        assert!(
            regex_rule_spans(&parsed, "literal").is_empty(),
            "numeric angle subroutine ref must not spill into literal atoms"
        );
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    #[test]
    fn regex_parser_integration_contract_classifies_language_tagged_code_blocks() {
        for (tag, body, input) in [
            ("lua", "return true", "(?{lua:return true})"),
            ("js", "return true;", "(?{js:return true;})"),
            ("javascript", "return true;", "(?{javascript:return true;})"),
            ("rhai", "let x = 1;", "(?{rhai:let x = 1;})"),
            ("native", "callback_name", "(?{native:callback_name})"),
            ("wasm", "module:function", "(?{wasm:module:function})"),
        ] {
            let parsed = regex_ast_dump_json(input);

            assert_eq!(
                regex_rule_spans(&parsed, "code_block_lang"),
                vec![(0, input.len() as u64)],
                "tagged code block '{}' must classify as code_block_lang",
                input
            );
            assert_eq!(
                regex_rule_spans(&parsed, "code_lang"),
                vec![(3, (3 + tag.len()) as u64)],
                "tagged code block '{}' must preserve the language tag span",
                input
            );
            assert_eq!(
                regex_rule_texts(input, &parsed, "code_lang"),
                vec![tag.to_string()],
                "tagged code block '{}' must preserve the language tag text",
                input
            );
            assert_eq!(
                regex_rule_texts(input, &parsed, "code_content"),
                vec![body.to_string()],
                "tagged code block '{}' must preserve the full code body text",
                input
            );
            assert!(
                regex_rule_spans(&parsed, "code_block_plain").is_empty(),
                "tagged code block '{}' must not degrade into code_block_plain",
                input
            );
        }
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    #[test]
    fn regex_parser_integration_contract_preserves_plain_code_blocks_as_plain() {
        let input = "(?{payload})";
        let parsed = regex_ast_dump_json(input);

        assert_eq!(
            regex_rule_spans(&parsed, "code_block_plain"),
            vec![(0, input.len() as u64)]
        );
        assert!(
            regex_rule_spans(&parsed, "code_block_lang").is_empty(),
            "plain code blocks must not be misclassified as code_block_lang"
        );
        assert!(
            regex_rule_spans(&parsed, "code_lang").is_empty(),
            "plain code blocks must not synthesize a language tag"
        );
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    #[test]
    fn regex_parser_integration_contract_preserves_conditional_false_branch() {
        let parsed = regex_ast_dump_json("(?(1)a|b)");

        assert_eq!(regex_rule_spans(&parsed, "conditional"), vec![(0, 9)]);
        assert_eq!(regex_rule_spans(&parsed, "yes_branch"), vec![(5, 6)]);
        assert_eq!(regex_rule_spans(&parsed, "no_branch"), vec![(7, 8)]);
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    #[test]
    fn regex_parser_integration_contract_accepts_named_recursion_conditionals() {
        let input = "(?(R&word)a|b)";
        let parsed = regex_ast_dump_json(input);

        assert_eq!(regex_rule_spans(&parsed, "conditional"), vec![(0, 14)]);
        assert_eq!(regex_rule_spans(&parsed, "recursion_condition"), vec![(3, 9)]);
        assert_eq!(regex_rule_texts(input, &parsed, "recursion_condition"), vec!["R&word"]);
        assert_eq!(regex_rule_texts(input, &parsed, "name"), vec!["word"]);
        assert_eq!(regex_rule_spans(&parsed, "yes_branch"), vec![(10, 11)]);
        assert_eq!(regex_rule_spans(&parsed, "no_branch"), vec![(12, 13)]);
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    #[test]
    fn regex_parser_integration_contract_accepts_bare_recursion_conditionals() {
        let input = "(?(R)a|b)";
        let parsed = regex_ast_dump_json(input);

        assert_eq!(regex_rule_spans(&parsed, "conditional"), vec![(0, 9)]);
        assert_eq!(regex_rule_spans(&parsed, "recursion_condition"), vec![(3, 4)]);
        assert_eq!(regex_rule_texts(input, &parsed, "recursion_condition"), vec!["R"]);
        assert!(
            regex_rule_spans(&parsed, "name").is_empty(),
            "bare recursion conditionals must not fall back to name"
        );
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    #[test]
    fn regex_parser_integration_contract_accepts_numeric_recursion_conditionals() {
        let input = "(a)(?(R1)b|c)";
        let parsed = regex_ast_dump_json(input);

        assert_eq!(regex_rule_spans(&parsed, "conditional"), vec![(3, 13)]);
        assert_eq!(regex_rule_spans(&parsed, "recursion_condition"), vec![(6, 8)]);
        assert_eq!(regex_rule_texts(input, &parsed, "recursion_condition"), vec!["R1"]);
        assert_eq!(regex_rule_texts(input, &parsed, "digits"), vec!["1"]);
        assert!(
            regex_rule_spans(&parsed, "name").is_empty(),
            "numeric recursion conditionals must not fall back to name"
        );
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    #[test]
    fn regex_parser_integration_contract_accepts_version_conditionals() {
        for (input, condition_text, operator, version) in [
            (
                "(?(VERSION>=10.0)cat|dog)",
                "VERSION>=10.0",
                ">=",
                "10.0",
            ),
            (
                "(?(VERSION >= 10)cat|dog)",
                "VERSION >= 10",
                ">=",
                "10",
            ),
        ] {
            let parsed = regex_ast_dump_json(input);

            assert_eq!(
                regex_rule_spans(&parsed, "conditional"),
                vec![(0, input.len() as u64)],
                "VERSION conditional must classify as a conditional for '{}'",
                input
            );
            assert_eq!(
                regex_rule_texts(input, &parsed, "version_condition"),
                vec![condition_text],
                "VERSION conditional must preserve the full condition body for '{}'",
                input
            );
            assert_eq!(
                regex_rule_texts(input, &parsed, "version_operator"),
                vec![operator],
                "VERSION conditional must preserve the comparison operator for '{}'",
                input
            );
            assert_eq!(
                regex_rule_texts(input, &parsed, "version_number"),
                vec![version],
                "VERSION conditional must preserve the target version for '{}'",
                input
            );
            assert!(
                regex_rule_spans(&parsed, "name").is_empty(),
                "VERSION conditionals must not fall back to bare name parsing for '{}'",
                input
            );
        }
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    #[test]
    fn regex_parser_integration_contract_binds_quantifier_to_final_literal_atom() {
        let parsed = regex_ast_dump_json("ab+");

        assert_eq!(regex_rule_spans(&parsed, "piece"), vec![(0, 1), (1, 3)]);
        assert_eq!(regex_rule_spans(&parsed, "literal"), vec![(0, 1), (1, 2)]);
        assert!(
            !regex_rule_spans(&parsed, "literal").contains(&(0, 2)),
            "multi-character literal runs must not absorb the quantified suffix"
        );
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    #[test]
    fn regex_parser_integration_contract_accepts_unicode_emoji_literal() {
        let input = "🎉";
        let parsed = regex_ast_dump_json(input);

        assert_eq!(regex_rule_spans(&parsed, "literal"), vec![(0, input.len() as u64)]);
        assert_eq!(regex_rule_texts(input, &parsed, "literal"), vec!["🎉"]);
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    #[test]
    fn regex_parser_integration_contract_accepts_mixed_ascii_unicode_literal_run() {
        let input = "café";
        let parsed = regex_ast_dump_json(input);

        assert_eq!(regex_rule_spans(&parsed, "literal"), vec![(0, 1), (1, 2), (2, 3), (3, 5)]);
        assert_eq!(regex_rule_texts(input, &parsed, "literal"), vec!["c", "a", "f", "é"]);
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    #[test]
    fn regex_parser_integration_contract_accepts_50_nested_capturing_groups() {
        let input = format!("{}a{}", "(".repeat(50), ")".repeat(50));
        let parsed = regex_ast_dump_json(&input);

        assert_eq!(
            regex_rule_spans(&parsed, "capturing_group").len(),
            50,
            "expected 50 nested capturing_group nodes"
        );
        assert_eq!(regex_rule_texts(&input, &parsed, "literal"), vec!["a"]);
        assert_eq!(
            regex_rule_spans(&parsed, "capturing_group")[0],
            (0, input.len() as u64),
            "outermost capturing group should span the full input"
        );
    }

    #[cfg(all(feature = "generated_parsers", has_generated_vhdl_parser))]
    #[test]
    fn parser_embedding_ast_dump_uses_vhdl_generated_backend_when_available() {
        let outcome =
            parse_vhdl_1076_2019_ast_dump("entity e is end entity;", &AstDumpOptions::default());
        let maybe_code = outcome.diagnostic.as_ref().map(|diag| diag.code.as_str());
        assert_ne!(maybe_code, Some("E_BACKEND_UNAVAILABLE"));
        if outcome.status == ParseStatus::Success {
            let ast_dump = outcome.ast_dump.expect("ast dump payload");
            let parsed: serde_json::Value =
                serde_json::from_str(&ast_dump.dump_json).expect("dump json");
            assert!(parsed.is_object());
        }
    }

    #[cfg(all(feature = "generated_parsers", has_generated_regex_parser))]
    #[test]
    fn parser_embedding_ast_dump_uses_regex_generated_backend_when_available() {
        let outcome = parse_regex_default_ast_dump("a|b", &AstDumpOptions::default());
        let maybe_code = outcome.diagnostic.as_ref().map(|diag| diag.code.as_str());
        assert_ne!(maybe_code, Some("E_BACKEND_UNAVAILABLE"));
        if outcome.status == ParseStatus::Success {
            let ast_dump = outcome.ast_dump.expect("ast dump payload");
            let parsed: serde_json::Value =
                serde_json::from_str(&ast_dump.dump_json).expect("dump json");
            assert!(parsed.is_object());
        }
    }

    #[cfg(not(feature = "generated_parsers"))]
    #[test]
    fn generated_backend_reports_feature_requirement_without_generated_feature() {
        let outcome = parse_annotation(AnnotationFamily::Return, ParserBackend::Generated, "$1");
        assert_eq!(outcome.status, ParseStatus::Failure);
        let diagnostic = outcome
            .diagnostic
            .as_ref()
            .expect("expected backend-availability diagnostic");
        assert_eq!(diagnostic.code, "E_BACKEND_UNAVAILABLE");
    }

    #[cfg(feature = "generated_parsers")]
    #[test]
    fn generated_return_smoke_parse_succeeds() {
        let outcome = parse_annotation(AnnotationFamily::Return, ParserBackend::Generated, "$1");
        assert_eq!(outcome.status, ParseStatus::Success);
        assert!(outcome.diagnostic.is_none());
    }

    #[cfg(feature = "generated_parsers")]
    #[test]
    fn generated_semantic_smoke_parse_succeeds() {
        let outcome = parse_annotation(
            AnnotationFamily::Semantic,
            ParserBackend::Generated,
            "@deprecated: true",
        );
        assert_eq!(outcome.status, ParseStatus::Success);
        assert!(outcome.diagnostic.is_none());
    }
}
