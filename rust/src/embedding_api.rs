use crate::ast_pipeline::{UnifiedReturnAST, UnifiedSemanticAST, runtime_logger};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrammarParseOutcome {
    pub api_version: String,
    pub grammar: GrammarFamily,
    pub profile: GrammarProfile,
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
    pub supports_systemverilog_generated_backend: bool,
    pub supports_vhdl_generated_backend: bool,
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
    ParserEmbeddingApiContract {
        api_version: EMBEDDING_API_VERSION.to_string(),
        schema_version: EMBEDDING_API_SCHEMA_VERSION,
        deterministic_by_default: true,
        supported_grammars: vec![GrammarFamily::SystemVerilog, GrammarFamily::Vhdl],
        supported_profiles: vec![
            GrammarProfile::Sv2017,
            GrammarProfile::Sv2023,
            GrammarProfile::Vhdl1076_2019,
        ],
        supports_systemverilog_generated_backend: systemverilog_generated_backend_enabled(),
        supports_vhdl_generated_backend: vhdl_generated_backend_enabled(),
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

/// Parses a full grammar input using a stable profile-aware parser entry point.
pub fn parse_grammar_profile(
    grammar: GrammarFamily,
    profile: GrammarProfile,
    input: &str,
) -> GrammarParseOutcome {
    parse_grammar_profile_with_limits(grammar, profile, input, &ParseLimits::default())
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
        let mut parser =
            VhdlParser::new(input, crate::ast_pipeline::runtime_logger_box("embedding.generated.vhdl"));
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
