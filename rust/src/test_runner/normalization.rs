/// Normalization utilities for round-trip testing
/// Handles formatting differences between input and generated output
use crate::ast_pipeline::unified_return_ast::{ExtractionTarget, UnifiedReturnAST};

#[derive(Debug, Clone, Copy)]
pub enum Normalizer {
    Text,
    Float,
    Json,
    Identifier,
    ReturnAst,
}

impl Normalizer {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "float" => Self::Float,
            "json" => Self::Json,
            "identifier" => Self::Identifier,
            "return_ast" | "return" | "return_annotation" => Self::ReturnAst,
            _ => Self::Text,
        }
    }
}

pub fn apply_normalizer(
    normalizer: Normalizer,
    input: &str,
    float_precision: Option<usize>,
) -> String {
    match normalizer {
        Normalizer::Float => normalize_float(input, float_precision.unwrap_or(10)),
        Normalizer::Json => normalize_json(input),
        Normalizer::Identifier => normalize_identifier(input),
        Normalizer::ReturnAst => normalize_return_ast(input),
        Normalizer::Text => normalize_text(input),
    }
}

fn normalize_text(input: &str) -> String {
    input.trim().to_string()
}

fn normalize_float(input: &str, precision: usize) -> String {
    let trimmed = input.trim();
    if let Ok(f) = trimmed.parse::<f64>() {
        // Format to specified precision and remove trailing zeros
        let formatted = format!("{:.prec$}", f, prec = precision);
        formatted
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    } else {
        normalize_text(trimmed)
    }
}

fn normalize_json(input: &str) -> String {
    // For now, just trim - full JSON normalization would require serde
    normalize_text(input)
}

fn normalize_identifier(input: &str) -> String {
    // Keep identifier case-sensitive; normalize surrounding whitespace only
    input.trim().to_string()
}

fn normalize_return_ast(input: &str) -> String {
    let logger = crate::NoOpLogger;
    match UnifiedReturnAST::parse_bootstrap(input, &logger) {
        Ok(ast) => canonicalize_return_ast(&ast),
        Err(_) => normalize_text(input),
    }
}

fn canonicalize_return_ast(ast: &UnifiedReturnAST) -> String {
    match ast {
        UnifiedReturnAST::PositionalRef { index } => format!("${}", index),
        UnifiedReturnAST::StringLiteral { value } => {
            format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
        }
        UnifiedReturnAST::NumberLiteral { value } => {
            if value.fract() == 0.0 {
                format!("{:.0}", value)
            } else {
                value.to_string()
            }
        }
        UnifiedReturnAST::BooleanLiteral { value } => value.to_string(),
        UnifiedReturnAST::Identifier { name } => name.clone(),
        UnifiedReturnAST::Object { properties } => {
            let mut keys: Vec<&String> = properties.keys().collect();
            keys.sort();
            let parts: Vec<String> = keys
                .iter()
                .map(|k| {
                    let key = canonicalize_object_key(k);
                    let value = canonicalize_return_ast(
                        properties.get(*k).expect("existing key in canonicalizer"),
                    );
                    format!("{}: {}", key, value)
                })
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
        UnifiedReturnAST::Array { elements } => {
            let parts: Vec<String> = elements.iter().map(canonicalize_return_ast).collect();
            format!("[{}]", parts.join(", "))
        }
        UnifiedReturnAST::Spread { base } => format!("{}*", canonicalize_return_ast(base)),
        UnifiedReturnAST::FlattenSpread { base } => {
            format!("{}**", canonicalize_return_ast(base))
        }
        UnifiedReturnAST::PropertyAccess { base, property } => {
            format!("{}.{}", canonicalize_return_ast(base), property)
        }
        UnifiedReturnAST::ArrayAccess { base, index } => {
            format!(
                "{}[{}]",
                canonicalize_return_ast(base),
                canonicalize_return_ast(index)
            )
        }
        UnifiedReturnAST::QuantifiedExtraction { base, target } => {
            let target_str = match target {
                ExtractionTarget::Index(idx) => (idx + 1).to_string(),
                ExtractionTarget::First => "first".to_string(),
                ExtractionTarget::Last => "last".to_string(),
            };
            format!("{}::{}", canonicalize_return_ast(base), target_str)
        }
        UnifiedReturnAST::Passthrough => "$1".to_string(),
    }
}

fn canonicalize_object_key(key: &str) -> String {
    let mut chars = key.chars();
    let Some(first) = chars.next() else {
        return "\"\"".to_string();
    };
    let is_identifier = (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric());
    if is_identifier {
        key.to_string()
    } else {
        format!("\"{}\"", key.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_float() {
        assert_eq!(normalize_float("3.14000", 10), "3.14");
        assert_eq!(normalize_float("3.0", 10), "3");
        assert_eq!(normalize_float("123.000", 2), "123");
    }

    #[test]
    fn test_normalizer_from_str() {
        assert!(matches!(Normalizer::from_str("float"), Normalizer::Float));
        assert!(matches!(Normalizer::from_str("text"), Normalizer::Text));
        assert!(matches!(Normalizer::from_str("unknown"), Normalizer::Text));
    }

    #[test]
    fn test_normalize_identifier_preserves_case() {
        assert_eq!(normalize_identifier("  FooBar_123  "), "FooBar_123");
    }
}
