use crate::ast_pipeline::{UnifiedReturnAST, UnifiedSemanticAST, runtime_logger};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Stable embedding API contract version.
///
/// Compatibility policy:
/// - major version changes signal breaking API/behavioral contract changes
/// - minor/patch changes are backward compatible for existing callers
pub const EMBEDDING_API_VERSION: &str = "1.0.0";

/// Stable schema version for serialized embedding API metadata.
pub const EMBEDDING_API_SCHEMA_VERSION: u32 = 1;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParseDiagnostic {
    pub code: String,
    pub message: String,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrammarProfile {
    #[serde(rename = "sv_2017")]
    Sv2017,
    #[serde(rename = "sv_2023")]
    Sv2023,
    #[serde(rename = "vhdl_1076_2019")]
    Vhdl1076_2019,
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
        }
    }
}

impl GrammarProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            GrammarProfile::Sv2017 => "sv_2017",
            GrammarProfile::Sv2023 => "sv_2023",
            GrammarProfile::Vhdl1076_2019 => "vhdl_1076_2019",
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
    pub supported_grammars: Vec<GrammarFamily>,
    pub supported_profiles: Vec<GrammarProfile>,
    pub profile_matrix: Vec<GrammarProfileBinding>,
    pub supports_systemverilog_generated_backend: bool,
    pub supports_vhdl_generated_backend: bool,
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
    ParserEmbeddingApiContract {
        api_version: EMBEDDING_API_VERSION.to_string(),
        schema_version: EMBEDDING_API_SCHEMA_VERSION,
        deterministic_by_default: true,
        supported_grammars: vec![GrammarFamily::SystemVerilog, GrammarFamily::Vhdl],
        supported_profiles: vec![
            systemverilog_profiles[0],
            systemverilog_profiles[1],
            vhdl_profiles[0],
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
        ],
        supports_systemverilog_generated_backend: systemverilog_generated_backend_enabled(),
        supports_vhdl_generated_backend: vhdl_generated_backend_enabled(),
    }
}

fn invalid_argument_diagnostic(message: String) -> ParseDiagnostic {
    ParseDiagnostic {
        code: "E_INVALID_ARGUMENT".to_string(),
        message,
    }
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
        GrammarFamily::SystemVerilog => parse_generated_systemverilog(input),
        GrammarFamily::Vhdl => parse_generated_vhdl(input),
    }
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
    };
    if valid {
        return Ok(());
    }

    Err(ParseDiagnostic {
        code: "E_UNSUPPORTED_PROFILE".to_string(),
        message: format!(
            "profile {:?} is not supported for grammar {:?}",
            profile, grammar
        ),
    })
}

fn validate_input_limits(input: &str, limits: &ParseLimits) -> Result<(), ParseDiagnostic> {
    if limits.max_input_bytes == 0 {
        return Err(ParseDiagnostic {
            code: "E_INVALID_LIMITS".to_string(),
            message: "max_input_bytes must be greater than 0".to_string(),
        });
    }

    let input_len = input.len();
    if input_len > limits.max_input_bytes {
        return Err(ParseDiagnostic {
            code: "E_INPUT_TOO_LARGE".to_string(),
            message: format!(
                "input size {} bytes exceeds max_input_bytes {}",
                input_len, limits.max_input_bytes
            ),
        });
    }

    Ok(())
}

fn parse_bootstrap_return(input: &str) -> Result<(), ParseDiagnostic> {
    let logger = runtime_logger("embedding.bootstrap.return_annotation");
    UnifiedReturnAST::parse_bootstrap(input, &logger)
        .map(|_| ())
        .map_err(|err| ParseDiagnostic {
            code: "E_PARSE_FAILURE".to_string(),
            message: format!("bootstrap return parse failed: {}", err),
        })
}

fn parse_bootstrap_semantic(input: &str) -> Result<(), ParseDiagnostic> {
    let logger = runtime_logger("embedding.bootstrap.semantic_annotation");
    UnifiedSemanticAST::parse_bootstrap(input, &logger)
        .map(|_| ())
        .map_err(|err| ParseDiagnostic {
            code: "E_PARSE_FAILURE".to_string(),
            message: format!("bootstrap semantic parse failed: {}", err),
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
            .map_err(|err| ParseDiagnostic {
                code: "E_PARSE_FAILURE".to_string(),
                message: format!("generated return parse failed: {}", err),
            });
    }
    #[cfg(not(feature = "generated_parsers"))]
    {
        let _ = input;
        Err(ParseDiagnostic {
            code: "E_BACKEND_UNAVAILABLE".to_string(),
            message: "generated parser backend requires feature `generated_parsers`".to_string(),
        })
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
            .map_err(|err| ParseDiagnostic {
                code: "E_PARSE_FAILURE".to_string(),
                message: format!("generated semantic parse failed: {}", err),
            });
    }
    #[cfg(not(feature = "generated_parsers"))]
    {
        let _ = input;
        Err(ParseDiagnostic {
            code: "E_BACKEND_UNAVAILABLE".to_string(),
            message: "generated parser backend requires feature `generated_parsers`".to_string(),
        })
    }
}

fn parse_generated_systemverilog(input: &str) -> Result<(), ParseDiagnostic> {
    #[cfg(all(feature = "generated_parsers", has_generated_systemverilog_parser))]
    {
        use crate::generated_parsers::systemverilog::SystemverilogParser;
        let mut parser = SystemverilogParser::new(
            input,
            crate::ast_pipeline::runtime_logger_box("embedding.generated.systemverilog"),
        );
        return parser
            .parse_full_systemverilog_file()
            .map(|_| ())
            .map_err(|err| ParseDiagnostic {
                code: "E_PARSE_FAILURE".to_string(),
                message: format!("generated systemverilog parse failed: {}", err),
            });
    }
    #[cfg(not(all(feature = "generated_parsers", has_generated_systemverilog_parser)))]
    {
        let _ = input;
        Err(ParseDiagnostic {
            code: "E_BACKEND_UNAVAILABLE".to_string(),
            message:
                "systemverilog parser backend requires `generated_parsers` and generated/systemverilog_parser.rs"
                    .to_string(),
        })
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
            .map_err(|err| ParseDiagnostic {
                code: "E_PARSE_FAILURE".to_string(),
                message: format!("generated vhdl parse failed: {}", err),
            });
    }
    #[cfg(not(all(feature = "generated_parsers", has_generated_vhdl_parser)))]
    {
        let _ = input;
        Err(ParseDiagnostic {
            code: "E_BACKEND_UNAVAILABLE".to_string(),
            message:
                "vhdl parser backend requires `generated_parsers` and generated/vhdl_parser.rs"
                    .to_string(),
        })
    }
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

#[cfg(test)]
mod tests {
    use super::*;

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
            GrammarProfile::from_str("ieee1800-2023")
                .expect("profile alias")
                .as_str(),
            "sv_2023"
        );
    }

    #[test]
    fn parser_embedding_contract_exposes_profile_matrix() {
        let contract = parser_embedding_api_contract();
        assert_eq!(contract.api_version, EMBEDDING_API_VERSION);
        assert_eq!(contract.schema_version, EMBEDDING_API_SCHEMA_VERSION);
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
