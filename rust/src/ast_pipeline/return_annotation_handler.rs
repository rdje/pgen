// Return Annotation Handler for Code Generation
// Supports both bootstrap mode (limited) and full mode

use serde_json::{Value, json};
use std::collections::HashMap;

/// Return annotation processing modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReturnAnnotationMode {
    /// Bootstrap mode - limited subset for self-bootstrapping
    Bootstrap,
    /// Full mode - complete return annotation support
    Full,
}

/// Represents a parsed return annotation
#[derive(Debug, Clone)]
pub enum ReturnAnnotation {
    /// Scalar reference: $1, $2, etc.
    ScalarRef { index: usize },

    /// Array of elements: [$1, $3*]
    Array { elements: Vec<ArrayElement> },

    /// Object with properties: {type: "array", contents: $3}
    Object {
        properties: HashMap<String, ReturnAnnotation>,
    },

    /// String literal: "array", "object", etc.
    Literal { value: String },

    /// Quantified reference: $3* (the * is stored separately)
    Quantified {
        base: Box<ReturnAnnotation>,
        quantifier: String,
    },

    /// Pass-through - no return annotation
    Passthrough,
}

#[derive(Debug, Clone)]
pub enum ArrayElement {
    Single(ReturnAnnotation),
    Spread(ReturnAnnotation), // For $3* notation
}

/// Return annotation handler
pub struct ReturnAnnotationHandler {
    mode: ReturnAnnotationMode,
    debug: bool,
}

impl ReturnAnnotationHandler {
    pub fn new(mode: ReturnAnnotationMode, debug: bool) -> Self {
        Self { mode, debug }
    }

    /// Parse a return annotation string into structured form
    /// Bootstrap mode supports: -> $1, -> [$1, $2*], -> {key: value}, -> "literal"
    /// Also handles the format used by return_annotation.ebnf: {type: "scalar", index: ...}
    pub fn parse_return_annotation(&self, annotation: &str) -> Result<ReturnAnnotation, String> {
        if self.debug {
            println!(
                "[ReturnAnnotationHandler] Parsing annotation: {}",
                annotation
            );
        }

        // First, strip the -> prefix if present
        let trimmed = annotation.trim();
        let trimmed = if trimmed.starts_with("-> ") {
            trimmed[3..].trim() // Skip "-> " (3 chars)
        } else if trimmed.starts_with("->") {
            trimmed[2..].trim() // Skip "->" (2 chars)
        } else {
            trimmed // No prefix, use as-is for backward compatibility
        };

        // Check for scalar reference: $1, $2, etc.
        if trimmed.starts_with('$') {
            return self.parse_scalar_ref(trimmed);
        }

        // Check for array: [...]
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            return self.parse_array(trimmed);
        }

        // Check for object: {...}
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            // Check if it's a structured format from return_annotation.ebnf
            if trimmed.contains("type:") {
                return self.parse_structured_object(trimmed);
            }
            return self.parse_object(trimmed);
        }

        // Check for string literal: "..."
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            return Ok(ReturnAnnotation::Literal {
                value: trimmed[1..trimmed.len() - 1].to_string(),
            });
        }

        // Default to passthrough
        Ok(ReturnAnnotation::Passthrough)
    }

    fn parse_scalar_ref(&self, input: &str) -> Result<ReturnAnnotation, String> {
        // Parse $1, $2, etc.
        if let Some(num_str) = input.strip_prefix('$') {
            if let Ok(index) = num_str.parse::<usize>() {
                return Ok(ReturnAnnotation::ScalarRef { index });
            }
        }
        Err(format!("Invalid scalar reference: {}", input))
    }

    fn parse_array(&self, input: &str) -> Result<ReturnAnnotation, String> {
        // Remove brackets and parse contents
        let contents = &input[1..input.len() - 1].trim();

        if contents.is_empty() {
            return Ok(ReturnAnnotation::Array { elements: vec![] });
        }

        let mut elements = Vec::new();
        let parts = self.split_array_elements(contents);

        for part in parts {
            let trimmed_part = part.trim();

            // Check if it's a spread element (ends with *)
            if trimmed_part.ends_with('*') {
                let base = &trimmed_part[..trimmed_part.len() - 1];
                let annotation = self.parse_return_annotation(base)?;
                elements.push(ArrayElement::Spread(annotation));
            } else {
                let annotation = self.parse_return_annotation(trimmed_part)?;
                elements.push(ArrayElement::Single(annotation));
            }
        }

        Ok(ReturnAnnotation::Array { elements })
    }

    fn parse_object(&self, input: &str) -> Result<ReturnAnnotation, String> {
        // Remove braces and parse key-value pairs
        let contents = &input[1..input.len() - 1].trim();

        if contents.is_empty() {
            return Ok(ReturnAnnotation::Object {
                properties: HashMap::new(),
            });
        }

        let mut properties = HashMap::new();
        let pairs = self.split_object_pairs(contents);

        for pair in pairs {
            let parts: Vec<&str> = pair.splitn(2, ':').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid object pair: {}", pair));
            }

            let key = parts[0].trim().trim_matches('"');
            let value = self.parse_return_annotation(parts[1].trim())?;
            properties.insert(key.to_string(), value);
        }

        Ok(ReturnAnnotation::Object { properties })
    }

    fn split_array_elements(&self, contents: &str) -> Vec<String> {
        // Simple comma splitting for bootstrap mode
        // In full mode, this would handle nested structures properly
        let mut elements = Vec::new();
        let mut current = String::new();
        let mut depth = 0;
        let mut in_string = false;

        for ch in contents.chars() {
            match ch {
                '"' if !in_string => in_string = true,
                '"' if in_string => in_string = false,
                '[' | '{' if !in_string => depth += 1,
                ']' | '}' if !in_string => depth -= 1,
                ',' if depth == 0 && !in_string => {
                    elements.push(current.trim().to_string());
                    current.clear();
                    continue;
                }
                _ => {}
            }
            current.push(ch);
        }

        if !current.trim().is_empty() {
            elements.push(current.trim().to_string());
        }

        elements
    }

    fn split_object_pairs(&self, contents: &str) -> Vec<String> {
        // Same logic as split_array_elements for now
        self.split_array_elements(contents)
    }

    /// Parse structured objects from return_annotation.ebnf format
    /// Examples: {type: "scalar", index: $2}, {type: "array", contents: ...}
    fn parse_structured_object(&self, input: &str) -> Result<ReturnAnnotation, String> {
        // Remove braces and parse the object
        let contents = &input[1..input.len() - 1].trim();

        // Parse key-value pairs
        let pairs = self.split_object_pairs(contents);

        // Look for the type field
        let mut type_value = None;
        let mut properties = HashMap::new();

        for pair in pairs {
            let parts: Vec<&str> = pair.splitn(2, ':').collect();
            if parts.len() != 2 {
                continue;
            }

            let key = parts[0].trim().trim_matches('"');
            let value = parts[1].trim();

            if key == "type" {
                type_value = Some(value.trim_matches('"').to_string());
            } else {
                properties.insert(key.to_string(), value.to_string());
            }
        }

        // Handle based on type
        match type_value.as_deref() {
            Some("scalar") => {
                // {type: "scalar", index: $2} or {type: "scalar", index: {type: "pos", value: "2"}}
                if let Some(index_str) = properties.get("index") {
                    // Check if index is a scalar reference like $2
                    if index_str.starts_with('$') {
                        return self.parse_scalar_ref(index_str);
                    }
                    // Check if index is itself a structured object
                    if index_str.starts_with('{') && index_str.ends_with('}') {
                        // For now, extract the numeric value from nested structures
                        if index_str.contains("value:") {
                            // Extract value from {type: "pos", value: "2"}
                            let value_regex =
                                regex::Regex::new(r#"value:\s*["']?(\d+)["']?"#).unwrap();
                            if let Some(caps) = value_regex.captures(index_str) {
                                if let Ok(idx) = caps.get(1).unwrap().as_str().parse::<usize>() {
                                    return Ok(ReturnAnnotation::ScalarRef { index: idx });
                                }
                            }
                        }
                    }
                    // Try to parse as direct number
                    if let Ok(idx) = index_str.trim_matches('"').parse::<usize>() {
                        return Ok(ReturnAnnotation::ScalarRef { index: idx });
                    }
                }
            }
            Some("array") => {
                // {type: "array", contents: ...} or {type: "array", element: ...}
                if let Some(contents_str) = properties
                    .get("contents")
                    .or_else(|| properties.get("element"))
                {
                    // Recursively parse the contents
                    if contents_str.starts_with('[') {
                        return self.parse_array(contents_str);
                    } else if contents_str.starts_with('$') {
                        // Simple scalar reference as element
                        let elem = self.parse_scalar_ref(contents_str)?;
                        return Ok(ReturnAnnotation::Array {
                            elements: vec![ArrayElement::Single(elem)],
                        });
                    }
                }
                // Empty array if no contents
                return Ok(ReturnAnnotation::Array { elements: vec![] });
            }
            Some("object") => {
                // {type: "object", contents: ...} or other object structures
                // For now, convert to a simple object with the properties
                let mut result_props = HashMap::new();
                for (k, v) in properties {
                    if k != "type" {
                        // Try to parse each value
                        let parsed_value = if v.starts_with('$') {
                            self.parse_scalar_ref(&v)?
                        } else if v.starts_with('"') && v.ends_with('"') {
                            ReturnAnnotation::Literal {
                                value: v[1..v.len() - 1].to_string(),
                            }
                        } else {
                            ReturnAnnotation::Literal { value: v }
                        };
                        result_props.insert(k, parsed_value);
                    }
                }
                return Ok(ReturnAnnotation::Object {
                    properties: result_props,
                });
            }
            _ => {
                // Unknown type or no type field, treat as regular object
                return self.parse_object(input);
            }
        }

        Err(format!("Failed to parse structured object: {}", input))
    }

    /// Generate code to build the AST based on return annotation
    pub fn generate_ast_builder_code(
        &self,
        annotation: &ReturnAnnotation,
        captured_vars: &[String],
        indent: &str,
    ) -> Result<String, String> {
        match annotation {
            ReturnAnnotation::ScalarRef { index } => {
                // Reference to a captured parse result
                if *index > 0 && *index <= captured_vars.len() {
                    // In bootstrap mode within branches, the captured variable is already a ParseContent,
                    // not a ParseNode, so we return it directly without accessing .content
                    // This is a special case for branch return annotations in bootstrap mode
                    Ok(captured_vars[index - 1].clone())
                } else {
                    Err(format!("Invalid scalar reference: ${}", index))
                }
            }

            ReturnAnnotation::Array { elements } => {
                // Build a sequence node from elements
                let mut code = format!("ParseContent::Sequence(vec![\n");

                for element in elements {
                    match element {
                        ArrayElement::Single(ann) => {
                            let elem_code = self.generate_ast_builder_code(
                                ann,
                                captured_vars,
                                &format!("{}    ", indent),
                            )?;
                            // Create a ParseNode wrapping the content
                            code.push_str(&format!("{}    ParseNode {{ rule_name: \"element\", content: {}, span: 0..0 }},\n", indent, elem_code));
                        }
                        ArrayElement::Spread(ann) => {
                            // For spread elements, unpack if it's a sequence
                            let elem_code = self.generate_ast_builder_code(
                                ann,
                                captured_vars,
                                &format!("{}    ", indent),
                            )?;
                            // In bootstrap mode, check if it's a sequence and spread it
                            code.push_str(&format!(
                                "{}    // Spread element: {}*\n",
                                indent, elem_code
                            ));
                            code.push_str(&format!(
                                "{}    /* TODO: Implement spread for: {} */\n",
                                indent, elem_code
                            ));
                        }
                    }
                }

                code.push_str(&format!("{}])", indent));
                Ok(code)
            }

            ReturnAnnotation::Object { properties } => {
                // Build an object-like structure using JSON
                // In bootstrap mode, we'll create a Terminal with JSON representation
                let mut json_obj = json!({});

                for (key, value) in properties {
                    // For bootstrap, create simple string representation
                    let value_str = match value {
                        ReturnAnnotation::Literal { value } => value.clone(),
                        ReturnAnnotation::ScalarRef { index } => {
                            if *index > 0 && *index <= captured_vars.len() {
                                format!("${}", index) // Placeholder for now
                            } else {
                                format!("$?")
                            }
                        }
                        _ => "complex".to_string(),
                    };
                    json_obj[key] = json!(value_str);
                }

                let json_str = serde_json::to_string(&json_obj).unwrap();
                Ok(format!(
                    "{}ParseContent::Terminal(r#\"{}\"#)",
                    indent, json_str
                ))
            }

            ReturnAnnotation::Literal { value } => Ok(format!(
                "{}ParseContent::Terminal(r#\"{}\"#)",
                indent, value
            )),

            ReturnAnnotation::Quantified { base, quantifier } => {
                // Handle quantified elements
                let base_code = self.generate_ast_builder_code(base, captured_vars, indent)?;
                Ok(format!(
                    "{}ParseContent::Quantified(vec![{}], \"{}\")",
                    indent, base_code, quantifier
                ))
            }

            ReturnAnnotation::Passthrough => {
                // Default behavior - return the last captured element
                if !captured_vars.is_empty() {
                    Ok(captured_vars.last().unwrap().to_string())
                } else {
                    Ok(format!("{}ParseContent::Terminal(\"\")", indent))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scalar_ref() {
        let handler = ReturnAnnotationHandler::new(ReturnAnnotationMode::Bootstrap, false);

        match handler.parse_return_annotation("$1") {
            Ok(ReturnAnnotation::ScalarRef { index }) => assert_eq!(index, 1),
            _ => panic!("Failed to parse scalar ref"),
        }
    }

    #[test]
    fn test_parse_array() {
        let handler = ReturnAnnotationHandler::new(ReturnAnnotationMode::Bootstrap, false);

        match handler.parse_return_annotation("[$1, $3*]") {
            Ok(ReturnAnnotation::Array { elements }) => {
                assert_eq!(elements.len(), 2);
            }
            _ => panic!("Failed to parse array"),
        }
    }

    #[test]
    fn test_parse_object() {
        let handler = ReturnAnnotationHandler::new(ReturnAnnotationMode::Bootstrap, false);

        match handler.parse_return_annotation("{type: \"array\", contents: $3}") {
            Ok(ReturnAnnotation::Array { elements }) => {
                assert_eq!(elements.len(), 1);
            }
            Ok(ReturnAnnotation::Object { properties }) => {
                assert!(properties.contains_key("type"));
                assert!(properties.contains_key("contents"));
            }
            _ => panic!("Failed to parse object"),
        }
    }
}
