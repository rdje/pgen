use pgen::ast_pipeline::unified_return_ast::{UnifiedReturnAST, ExtractionTarget};
use crate::ast_pipeline::unified_return_ast::UnifiedReturnAST;
use crate::ast_pipeline::unified_semantic_ast::UnifiedSemanticAST;
use crate::test_runner::Parser;
use anyhow::Result;

/// Return Annotation Parser implementation
pub struct ReturnAnnotationParser;

impl ReturnAnnotationParser {
    pub fn new() -> Self {
        Self
    }
}

    /// Convert AST back to original string format for true round-trip validation
    fn unparse_ast(&self, ast: &UnifiedReturnAST) -> String {
        match ast {
            UnifiedReturnAST::PositionalRef { index } => format!("${}", index),
            UnifiedReturnAST::StringLiteral { value } => format!(""{}"", value.replace("\", "\\").replace(""", "\"").replace("
", "\n").replace("	", "\t")),
            UnifiedReturnAST::NumberLiteral { value } => {
                if value.fract() == 0.0 {
                    format!("{:.0}", value)
                } else {
                    format!("{}", value)
                }
            },
            UnifiedReturnAST::BooleanLiteral { value } => format!("{}", value),
            UnifiedReturnAST::Array { elements } => {
                let elem_strs: Vec<String> = elements.iter().map(|e| self.unparse_ast(e)).collect();
                format!("[{}]", elem_strs.join(", "))
            },
            UnifiedReturnAST::Object { properties } => {
                let prop_strs: Vec<String> = properties.iter()
                    .map(|(k, v)| format!("{}: {}", k, self.unparse_ast(v)))
                    .collect();
                format!("{{{}}}", prop_strs.join(", "))
            },
            UnifiedReturnAST::Spread { base } => format!("{}*", self.unparse_ast(base)),
            UnifiedReturnAST::PropertyAccess { base, property } => format!("{}.{}", self.unparse_ast(base), property),
            UnifiedReturnAST::ArrayAccess { base, index } => format!("{}[{}]", self.unparse_ast(base), self.unparse_ast(index)),
            UnifiedReturnAST::QuantifiedExtraction { base, target } => {
                let target_str = match target {
                    ExtractionTarget::Index(idx) => format!("::{}", idx + 1),
                    ExtractionTarget::First => "::first".to_string(),
                    ExtractionTarget::Last => "::last".to_string(),
                };
                format!("{}{}", self.unparse_ast(base), target_str)
            },
            UnifiedReturnAST::Passthrough => "$1".to_string(),
        }
    }

impl Parser for ReturnAnnotationParser {
    fn round_trip(&self, input: &str) -> Result<String> {
        // Parse the return annotation using the bootstrap parser
        let ast = UnifiedReturnAST::parse_bootstrap(input, false)
            .map_err(|e| anyhow::anyhow!("Failed to parse return annotation '{}': {}", input, e))?;
        
        // Convert back to string representation for round-trip validation
        // For now, use the pretty_print representation
        // TODO: Implement proper unparsing that produces canonical string output
        Ok(ast.pretty_print(0).trim().to_string())
    }
}

/// Semantic Annotation Parser implementation  
pub struct SemanticAnnotationParser;

impl SemanticAnnotationParser {
    pub fn new() -> Self {
        Self
    }
}

    /// Convert AST back to original string format for round-trip validation
    fn unparse_ast(&self, ast: &UnifiedSemanticAST) -> String {
        match ast {
            UnifiedSemanticAST::TransformExpr(expr) => expr.clone(),
            UnifiedSemanticAST::Raw(raw) => raw.clone(),
        }
    }

impl Parser for SemanticAnnotationParser {
    fn round_trip(&self, input: &str) -> Result<String> {
        // Parse the semantic annotation using the bootstrap parser
        let ast = UnifiedSemanticAST::parse_bootstrap(input, false)
            .map_err(|e| anyhow::anyhow!("Failed to parse semantic annotation '{}': {}", input, e))?;
        
        // Convert back to string representation for round-trip validation
        // Use the pretty_print representation for consistency
        Ok(ast.pretty_print())
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
        assert!(result.contains("PositionalRef"));
        assert!(result.contains("$1"));
        
        // Test array
        let result = parser.round_trip("[$1, $2]").unwrap();
        assert!(result.contains("Array"));
        
        // Test object
        let result = parser.round_trip(r#"{type: "array", element: $3}"#).unwrap();
        assert!(result.contains("Object"));
    }
    
    #[test]
    fn test_semantic_annotation_parser() {
        let parser = SemanticAnnotationParser::new();
        
        // Test transform expression
        let result = parser.round_trip("str::parse::<f64>().unwrap_or(0.0)").unwrap();
        assert!(result.contains("TransformExpr"));
        
        // Test raw annotation
        let result = parser.round_trip("some_raw_annotation").unwrap();
        assert!(result.contains("Raw"));
    }
}
