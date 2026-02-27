use crate::ast_pipeline::{
    runtime_logger, runtime_logger_box, UnifiedReturnAST, UnifiedSemanticAST,
};
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingApiContract {
    pub api_version: String,
    pub schema_version: u32,
    pub supports_generated_backend: bool,
    pub deterministic_by_default: bool,
    pub supported_families: Vec<AnnotationFamily>,
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
            runtime_logger_box("embedding.generated.return_annotation"),
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
            runtime_logger_box("embedding.generated.semantic_annotation"),
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

#[cfg(feature = "generated_parsers")]
fn generated_backend_enabled() -> bool {
    true
}

#[cfg(not(feature = "generated_parsers"))]
fn generated_backend_enabled() -> bool {
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
        assert!(contract
            .supported_families
            .contains(&AnnotationFamily::Return));
        assert!(contract
            .supported_families
            .contains(&AnnotationFamily::Semantic));
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
