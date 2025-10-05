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
