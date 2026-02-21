use crate::ast_pipeline::UnifiedSemanticAST;
use crate::ast_pipeline::unified_return_ast::{ExtractionTarget, UnifiedReturnAST};

use super::{Logger, Parser};
use anyhow::Result;

/// Return Annotation Parser implementation
pub struct ReturnAnnotationParser {
    logger: Box<dyn Logger>,
}

impl ReturnAnnotationParser {
    pub fn new() -> Self {
        Self {
            logger: Box::new(crate::NoOpLogger),
        }
    }

    /// Convert AST back to original string format for true round-trip validation
    fn unparse_ast(&self, ast: &UnifiedReturnAST) -> String {
        match ast {
            UnifiedReturnAST::PositionalRef { index } => format!("${}", index),
            UnifiedReturnAST::StringLiteral { value } => format!("\"{}\"", value),
            UnifiedReturnAST::NumberLiteral { value } => {
                if value.fract() == 0.0 {
                    format!("{:.0}", value)
                } else {
                    format!("{}", value)
                }
            }
            UnifiedReturnAST::BooleanLiteral { value } => format!("{}", value),
            UnifiedReturnAST::Identifier { name } => name.clone(),
            UnifiedReturnAST::Array { elements } => {
                let elem_strs: Vec<String> = elements.iter().map(|e| self.unparse_ast(e)).collect();
                format!("[{}]", elem_strs.join(", "))
            }
            UnifiedReturnAST::Object { properties } => {
                let mut sorted_keys: Vec<&String> = properties.keys().collect();
                sorted_keys.sort();
                let prop_strs: Vec<String> = sorted_keys
                    .iter()
                    .map(|k| {
                        let v = properties.get(*k).expect("key exists");
                        format!("{}: {}", k, self.unparse_ast(v))
                    })
                    .collect();
                format!("{{{}}}", prop_strs.join(", "))
            }
            UnifiedReturnAST::Spread { base } => format!("{}*", self.unparse_ast(base)),
            UnifiedReturnAST::PropertyAccess { base, property } => {
                format!("{}.{}", self.unparse_ast(base), property)
            }
            UnifiedReturnAST::ArrayAccess { base, index } => {
                format!("{}[{}]", self.unparse_ast(base), self.unparse_ast(index))
            }
            UnifiedReturnAST::QuantifiedExtraction { base, target } => {
                let target_str = match target {
                    ExtractionTarget::Index(idx) => format!("::{}", idx + 1),
                    ExtractionTarget::First => "::first".to_string(),
                    ExtractionTarget::Last => "::last".to_string(),
                };
                format!("{}{}", self.unparse_ast(base), target_str)
            }
            UnifiedReturnAST::Passthrough => "$1".to_string(),
        }
    }
}

impl Parser for ReturnAnnotationParser {
    fn round_trip(&self, input: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.logger.log_info(
            "parsers.rs",
            line!(),
            &format!(
                "Starting return annotation parser round-trip for: '{}'",
                input
            ),
        );
        let has_arrow_prefix = input.trim_start().starts_with("->");

        // Parse the return annotation using the bootstrap parser
        let ast = UnifiedReturnAST::parse_bootstrap(input, &*self.logger).map_err(|e| {
            self.logger.log_error(
                "parsers.rs",
                line!(),
                &format!("Failed to parse return annotation '{}': {}", input, e),
            );
            anyhow::anyhow!("Failed to parse return annotation '{}': {}", input, e)
        })?;

        self.logger.log_success(
            "parsers.rs",
            line!(),
            &format!("Successfully parsed return annotation AST: {:?}", ast),
        );

        // Convert back to string representation for round-trip validation
        // Use proper unparsing that produces the original string format
        let unparsed = self.unparse_ast(&ast);
        let result = if has_arrow_prefix {
            format!("-> {}", unparsed)
        } else {
            unparsed
        };
        self.logger.log_info(
            "parsers.rs",
            line!(),
            &format!("Return annotation round-trip result: '{}'", result),
        );

        Ok(result)
    }

    fn set_logger(&mut self, logger: Box<dyn Logger>) {
        self.logger = logger;
    }

    fn get_logger(&self) -> &dyn Logger {
        &*self.logger
    }
}

/// Semantic Annotation Parser implementation  
pub struct SemanticAnnotationParser {
    logger: Box<dyn Logger>,
}

impl SemanticAnnotationParser {
    pub fn new() -> Self {
        Self {
            logger: Box::new(crate::test_runner::NoOpLogger),
        }
    }

    /// Convert AST back to original string format for round-trip validation
    fn unparse_ast(&self, ast: &UnifiedSemanticAST) -> String {
        match ast {
            UnifiedSemanticAST::TransformExpr { expression } => expression.clone(),
            UnifiedSemanticAST::Raw { content } => content.clone(),
        }
    }
}

impl Parser for SemanticAnnotationParser {
    fn round_trip(&self, input: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.logger.log_info(
            "parsers.rs",
            line!(),
            &format!(
                "Starting semantic annotation parser round-trip for: '{}'",
                input
            ),
        );

        // Parse the semantic annotation using the bootstrap parser
        let ast = UnifiedSemanticAST::parse_bootstrap(input, &*self.logger).map_err(|e| {
            self.logger.log_error(
                "parsers.rs",
                line!(),
                &format!("Failed to parse semantic annotation '{}': {}", input, e),
            );
            anyhow::anyhow!("Failed to parse semantic annotation '{}': {}", input, e)
        })?;

        self.logger.log_success(
            "parsers.rs",
            line!(),
            &format!("Successfully parsed semantic annotation AST: {:?}", ast),
        );

        // Convert back to string representation for round-trip validation
        let result = self.unparse_ast(&ast);
        self.logger.log_info(
            "parsers.rs",
            line!(),
            &format!("Semantic annotation round-trip result: '{}'", result),
        );

        Ok(result)
    }

    fn set_logger(&mut self, logger: Box<dyn Logger>) {
        self.logger = logger;
    }

    fn get_logger(&self) -> &dyn Logger {
        &*self.logger
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_return_annotation_parser() {
        let parser = ReturnAnnotationParser::new();

        // Test basic positional reference
        let result = parser.round_trip("$1").unwrap();
        assert_eq!(result, "$1");

        // Test array
        let result = parser.round_trip("[$1, $2]").unwrap();
        assert_eq!(result, "[$1, $2]");

        // Test object with one property to avoid map-order sensitivity
        let result = parser.round_trip("{type: $3}").unwrap();
        assert_eq!(result, "{type: $3}");

        // Test preserved arrow prefix form
        let result = parser.round_trip("-> $3").unwrap();
        assert_eq!(result, "-> $3");
    }

    #[test]
    fn test_semantic_annotation_parser() {
        let parser = SemanticAnnotationParser::new();

        // Test transform expression
        let result = parser
            .round_trip("str::parse::<f64>().unwrap_or(0.0)")
            .unwrap();
        assert_eq!(result, "str::parse::<f64>().unwrap_or(0.0)");

        // Test raw annotation
        let result = parser.round_trip("some_raw_annotation").unwrap();
        assert_eq!(result, "some_raw_annotation");
    }
}
