//! Registry for generated parser adapters used by parseability and round-trip checks.
//!
//! This centralizes grammar-name dispatch so new generated grammars are added in one place.

use crate::ast_pipeline::{UnifiedSemanticAST, runtime_logger, runtime_logger_box};
#[cfg(feature = "ebnf_dual_run")]
use crate::ebnf_generated_parser::EbnfParser;
use crate::generated_parsers::{
    return_annotation::Return_annotationParser, semantic_annotation::Semantic_annotationParser,
};

type ParseSampleFn = fn(&str) -> bool;

#[derive(Clone, Copy, Debug)]
pub struct GeneratedParserRegistryEntry {
    pub grammar_name: &'static str,
    parse_sample: ParseSampleFn,
}

impl GeneratedParserRegistryEntry {
    fn parse(&self, sample: &str) -> bool {
        (self.parse_sample)(sample)
    }
}

fn parse_with_return_annotation(sample: &str) -> bool {
    let mut parser =
        Return_annotationParser::new(sample, runtime_logger_box("generated.return_annotation"));
    parser.parse_full_return_annotation().is_ok()
}

fn parse_with_semantic_annotation(sample: &str) -> bool {
    let mut parser =
        Semantic_annotationParser::new(sample, runtime_logger_box("generated.semantic_annotation"));
    parser.parse_full_semantic_annotation().is_ok()
}

fn parse_with_builtin_return_annotation(sample: &str) -> bool {
    // Built-in return grammar is a strict subset of return_annotation grammar.
    parse_with_return_annotation(sample)
}

fn parse_with_builtin_semantic_annotation(sample: &str) -> bool {
    // Built-in semantic parser behavior is intentionally permissive and marker-based.
    // Parseability for builtin_semantic_annotation must follow this bootstrap contract,
    // not the stricter full semantic_annotation grammar.
    let logger = runtime_logger("bootstrap.semantic_annotation");
    UnifiedSemanticAST::parse_bootstrap(sample, &logger).is_ok()
}

#[cfg(feature = "ebnf_dual_run")]
fn parse_with_ebnf(sample: &str) -> bool {
    let mut parser = EbnfParser::new(sample, runtime_logger_box("generated.ebnf"));
    parser.parse_full_grammar_file().is_ok()
}

static GENERATED_PARSER_REGISTRY: &[GeneratedParserRegistryEntry] = &[
    GeneratedParserRegistryEntry {
        grammar_name: "return_annotation",
        parse_sample: parse_with_return_annotation,
    },
    GeneratedParserRegistryEntry {
        grammar_name: "semantic_annotation",
        parse_sample: parse_with_semantic_annotation,
    },
    GeneratedParserRegistryEntry {
        grammar_name: "builtin_return_annotation",
        parse_sample: parse_with_builtin_return_annotation,
    },
    GeneratedParserRegistryEntry {
        grammar_name: "builtin_semantic_annotation",
        parse_sample: parse_with_builtin_semantic_annotation,
    },
    #[cfg(feature = "ebnf_dual_run")]
    GeneratedParserRegistryEntry {
        grammar_name: "ebnf",
        parse_sample: parse_with_ebnf,
    },
    // Add future grammars here once their generated parser artifacts compile cleanly.
    // Examples: json, regex, systemverilog, vhdl.
];

fn find_entry(grammar_name: &str) -> Option<&'static GeneratedParserRegistryEntry> {
    GENERATED_PARSER_REGISTRY
        .iter()
        .find(|entry| entry.grammar_name == grammar_name)
}

pub fn supports_grammar(grammar_name: &str) -> bool {
    find_entry(grammar_name).is_some()
}

pub fn parse_sample(grammar_name: &str, sample: &str) -> Option<bool> {
    find_entry(grammar_name).map(|entry| entry.parse(sample))
}

pub fn registered_grammars() -> Vec<&'static str> {
    GENERATED_PARSER_REGISTRY
        .iter()
        .map(|entry| entry.grammar_name)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{parse_sample, registered_grammars, supports_grammar};

    #[test]
    fn registry_exposes_expected_annotation_grammars() {
        let grammars = registered_grammars();
        assert!(grammars.contains(&"return_annotation"));
        assert!(grammars.contains(&"semantic_annotation"));
        assert!(grammars.contains(&"builtin_return_annotation"));
        assert!(grammars.contains(&"builtin_semantic_annotation"));
    }

    #[cfg(feature = "ebnf_dual_run")]
    #[test]
    fn registry_exposes_ebnf_when_dual_run_enabled() {
        let grammars = registered_grammars();
        assert!(grammars.contains(&"ebnf"));
    }

    #[test]
    fn unknown_grammar_is_not_supported() {
        assert!(!supports_grammar("unknown"));
        assert!(parse_sample("unknown", "anything").is_none());
    }

    #[test]
    fn builtin_semantic_parseability_adapter_accepts_marker_and_raw_inputs() {
        assert_eq!(
            parse_sample("builtin_semantic_annotation", "@priority: [9, 1]"),
            Some(true)
        );
        assert_eq!(
            parse_sample(
                "builtin_semantic_annotation",
                "str::parse::<u32>().unwrap_or(0)"
            ),
            Some(true)
        );
    }

    #[cfg(feature = "ebnf_dual_run")]
    #[test]
    fn ebnf_parseability_adapter_accepts_valid_rule_and_rejects_garbage() {
        assert_eq!(
            parse_sample("ebnf", r#"rule_name := /([a-zA-Z_][a-zA-Z0-9_]*)/"#),
            Some(true)
        );
        assert_eq!(parse_sample("ebnf", ":::not-ebnf:::"), Some(false));
    }
}
