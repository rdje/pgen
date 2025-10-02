//! Unified Return Annotation AST
//! 
//! This module provides a single, consistent AST representation for return annotations
//! that is used throughout the pipeline:
//! 1. Parsed from text by the external parser or bootstrap parser
//! 2. Pretty-printed for debugging
//! 3. Used directly by the code generator to emit Rust code
//!
//! This eliminates the need for multiple parallel AST representations and parsers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Extraction target for quantified groups
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExtractionTarget {
    /// Extract by index (0-based): $2::2
    Index(usize),
    /// Extract first element: $2::first
    First,
    /// Extract last element: $2::last
    Last,
}

/// The unified AST representation of a return annotation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UnifiedReturnAST {
    /// Positional reference: $1, $2, etc.
    /// These map to captured elements in the parse sequence
    PositionalRef { 
        index: usize,  // 1-based index (e.g., $3 has index: 3)
    },
    
    /// String literal: "array", "object", etc.
    StringLiteral { 
        value: String,
    },
    
    /// Number literal: 42, 3.14
    NumberLiteral {
        value: f64,
    },
    
    /// Boolean literal: true or false
    BooleanLiteral {
        value: bool,
    },
    
    /// Object with key-value pairs: {type: "array", element: $3}
    Object { 
        properties: HashMap<String, Box<UnifiedReturnAST>>,
    },
    
    /// Array: [$1, $2, "literal"]
    Array { 
        elements: Vec<UnifiedReturnAST>,
    },
    
    /// Spread operator: $4* in [$1, $4*]
    /// Unpacks a sequence into individual elements
    Spread { 
        base: Box<UnifiedReturnAST>,
    },
    
    /// Property access: $1.value
    PropertyAccess {
        base: Box<UnifiedReturnAST>,
        property: String,
    },
    
    /// Array index access: $1[0]
    ArrayAccess {
        base: Box<UnifiedReturnAST>,
        index: Box<UnifiedReturnAST>,
    },
    
    /// Extraction from quantified groups: $2::2, $2::first, $2::last
    /// Extracts specific elements from each repetition of a quantified group
    QuantifiedExtraction {
        base: Box<UnifiedReturnAST>,
        target: ExtractionTarget,
    },
    
    /// Passthrough - no explicit return annotation (implicit -> $1)
    Passthrough,
}

impl UnifiedReturnAST {
    /// Parse a return annotation string into the unified AST
    /// This is the bootstrap parser used when the external parser isn't available
    pub fn parse_bootstrap(annotation: &str, debug: bool) -> Result<UnifiedReturnAST, String> {
        if debug {
            println!("[UnifiedReturnAST::parse_bootstrap] Parsing: '{}'", annotation);
        }
        
        // Remove leading "-> " if present
        let cleaned = if annotation.starts_with("-> ") {
            &annotation[3..]
        } else if annotation.starts_with("->") {
            &annotation[2..]
        } else {
            annotation
        }.trim();
        
        // Empty annotation means passthrough
        if cleaned.is_empty() {
            return Ok(UnifiedReturnAST::Passthrough);
        }
        
        let result = Self::parse_value(cleaned, debug)?;
        
        if debug {
            println!("[UnifiedReturnAST::parse_bootstrap] Parsed AST:\n{}", result.pretty_print(2));
        }
        
        Ok(result)
    }
    
    fn parse_value(input: &str, debug: bool) -> Result<UnifiedReturnAST, String> {
        let trimmed = input.trim();
        
        if debug {
            println!("[UnifiedReturnAST::parse_value] Parsing value: '{}'", trimmed);
        }
        
        // Check for positional reference $N (with potential modifiers)
        if trimmed.starts_with('$') {
            return Self::parse_positional_ref(trimmed, debug);
        }
        
        // Check for string literal "..."
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            return Ok(UnifiedReturnAST::StringLiteral {
                value: trimmed[1..trimmed.len()-1].to_string(),
            });
        }
        
        // Check for object {...}
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            return Self::parse_object(&trimmed[1..trimmed.len()-1], debug);
        }
        
        // Check for array [...]
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            return Self::parse_array(&trimmed[1..trimmed.len()-1], debug);
        }
        
        // Check for number
        if let Ok(num) = trimmed.parse::<f64>() {
            return Ok(UnifiedReturnAST::NumberLiteral { value: num });
        }
        
        // Check for boolean
        match trimmed {
            "true" => return Ok(UnifiedReturnAST::BooleanLiteral { value: true }),
            "false" => return Ok(UnifiedReturnAST::BooleanLiteral { value: false }),
            _ => {}
        }
        
        Err(format!("Unable to parse return value: '{}'", trimmed))
    }
    
    fn parse_positional_ref(input: &str, debug: bool) -> Result<UnifiedReturnAST, String> {
        if debug {
            println!("[UnifiedReturnAST::parse_positional_ref] Parsing: '{}'", input);
        }
        
        // Handle $N, $N*, $N.property, $N[index], $N::target, $N::target*
        // Skip the '$' and find where the number ends
        let without_dollar = &input[1..];
        let end_of_number = without_dollar
            .chars()
            .position(|c| !c.is_ascii_digit())
            .unwrap_or(without_dollar.len());
        
        let num_str = &without_dollar[..end_of_number];
        let remaining = &without_dollar[end_of_number..];
        
        if debug {
            println!("[UnifiedReturnAST::parse_positional_ref] Extracted number: '{}', remaining: '{}'", num_str, remaining);
        }
        
        if num_str.is_empty() {
            return Err(format!("Invalid positional reference: '{}'", input));
        }
        
        let index = num_str.parse::<usize>()
            .map_err(|_| format!("Invalid positional index: '{}'", num_str))?;
        
        let mut base = UnifiedReturnAST::PositionalRef { index };
        
        // Check for extraction operator first (::)
        if remaining.starts_with("::") {
            let extraction_part = &remaining[2..];
            
            // Check if there's a spread operator at the end
            let (target_str, has_spread) = if extraction_part.ends_with('*') {
                (&extraction_part[..extraction_part.len()-1], true)
            } else {
                (extraction_part, false)
            };
            
            // Parse the extraction target
            let target = match target_str {
                "first" => ExtractionTarget::First,
                "last" => ExtractionTarget::Last,
                s => {
                    // Try to parse as index
                    match s.parse::<usize>() {
                        Ok(idx) => ExtractionTarget::Index(idx),
                        Err(_) => return Err(format!("Invalid extraction target: '{}'", s)),
                    }
                }
            };
            
            base = UnifiedReturnAST::QuantifiedExtraction {
                base: Box::new(base),
                target,
            };
            
            // Apply spread if present
            if has_spread {
                base = UnifiedReturnAST::Spread { base: Box::new(base) };
            }
        } else if remaining.starts_with('*') {
            if debug {
                println!("[UnifiedReturnAST::parse_positional_ref] Found spread operator '*'");
            }
            base = UnifiedReturnAST::Spread { base: Box::new(base) };
        } else if remaining.starts_with('.') {
            // Property access
            let property = remaining[1..].to_string();
            if property.is_empty() {
                return Err(format!("Invalid property access: '{}'", input));
            }
            base = UnifiedReturnAST::PropertyAccess {
                base: Box::new(base),
                property,
            };
        } else if remaining.starts_with('[') {
            // Array access - for now, simplified
            // TODO: Parse the index expression properly
            if let Some(end) = remaining.find(']') {
                let index_str = &remaining[1..end];
                let index = Self::parse_value(index_str, debug)?;
                base = UnifiedReturnAST::ArrayAccess {
                    base: Box::new(base),
                    index: Box::new(index),
                };
            } else {
                return Err(format!("Unclosed array access: '{}'", input));
            }
        } else if !remaining.is_empty() {
            return Err(format!("Invalid positional reference modifier: '{}'", remaining));
        }
        
        if debug {
            println!("[UnifiedReturnAST::parse_positional_ref] Returning: {:?}", base);
        }
        
        Ok(base)
    }
    
    fn parse_object(content: &str, debug: bool) -> Result<UnifiedReturnAST, String> {
        if debug {
            println!("[UnifiedReturnAST::parse_object] Parsing object content: '{}'", content);
        }
        
        let mut properties = HashMap::new();
        
        // Split by commas, but respect nested structures
        let pairs = Self::split_respecting_nesting(content, ',');
        
        for pair in pairs {
            let parts = Self::split_respecting_nesting(&pair, ':');
            if parts.len() != 2 {
                return Err(format!("Invalid object property: '{}'", pair));
            }
            
            // Parse key (remove quotes if present)
            let key = parts[0].trim();
            let key = if key.starts_with('"') && key.ends_with('"') {
                key[1..key.len()-1].to_string()
            } else {
                key.to_string()
            };
            
            // Parse value
            let value = Self::parse_value(parts[1].trim(), debug)?;
            properties.insert(key, Box::new(value));
        }
        
        Ok(UnifiedReturnAST::Object { properties })
    }
    
    fn parse_array(content: &str, debug: bool) -> Result<UnifiedReturnAST, String> {
        if debug {
            println!("[UnifiedReturnAST::parse_array] Parsing array content: '{}'", content);
        }
        
        let mut elements = Vec::new();
        
        if !content.trim().is_empty() {
            let items = Self::split_respecting_nesting(content, ',');
            
            for item in items {
                let trimmed = item.trim();
                
                // Check for spread operator at the end
                if trimmed.ends_with('*') && !trimmed.starts_with('"') {
                    // It's a spread, but only if not inside a string
                    let base_str = &trimmed[..trimmed.len()-1];
                    let base = Self::parse_value(base_str, debug)?;
                    elements.push(UnifiedReturnAST::Spread { base: Box::new(base) });
                } else {
                    elements.push(Self::parse_value(trimmed, debug)?);
                }
            }
        }
        
        Ok(UnifiedReturnAST::Array { elements })
    }
    
    /// Split a string by delimiter, respecting nesting in [], {}, and ""
    fn split_respecting_nesting(input: &str, delimiter: char) -> Vec<String> {
        let mut result = Vec::new();
        let mut current = String::new();
        let mut depth = 0;
        let mut in_string = false;
        let mut escape_next = false;
        
        for ch in input.chars() {
            if escape_next {
                current.push(ch);
                escape_next = false;
                continue;
            }
            
            match ch {
                '\\' if in_string => escape_next = true,
                '"' => in_string = !in_string,
                '[' | '{' if !in_string => depth += 1,
                ']' | '}' if !in_string => depth -= 1,
                c if c == delimiter && depth == 0 && !in_string => {
                    if !current.trim().is_empty() {
                        result.push(current.trim().to_string());
                    }
                    current.clear();
                    continue;
                }
                _ => {}
            }
            current.push(ch);
        }
        
        if !current.trim().is_empty() {
            result.push(current.trim().to_string());
        }
        
        result
    }
    
    /// Pretty-print this AST for debugging
    pub fn pretty_print(&self, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);
        
        match self {
            UnifiedReturnAST::PositionalRef { index } => {
                format!("{}PositionalRef(${})\n", indent_str, index)
            }
            UnifiedReturnAST::StringLiteral { value } => {
                format!("{}StringLiteral(\"{}\")\n", indent_str, value)
            }
            UnifiedReturnAST::NumberLiteral { value } => {
                format!("{}NumberLiteral({})\n", indent_str, value)
            }
            UnifiedReturnAST::BooleanLiteral { value } => {
                format!("{}BooleanLiteral({})\n", indent_str, value)
            }
            UnifiedReturnAST::Object { properties } => {
                let mut result = format!("{}Object {{\n", indent_str);
                for (key, value) in properties {
                    result.push_str(&format!("{}  {}: \n{}", indent_str, key, value.pretty_print(indent + 2)));
                }
                result.push_str(&format!("{}}}\n", indent_str));
                result
            }
            UnifiedReturnAST::Array { elements } => {
                let mut result = format!("{}Array [\n", indent_str);
                for (i, elem) in elements.iter().enumerate() {
                    result.push_str(&format!("{}  [{}]: \n{}", indent_str, i, elem.pretty_print(indent + 2)));
                }
                result.push_str(&format!("{}]\n", indent_str));
                result
            }
            UnifiedReturnAST::Spread { base } => {
                format!("{}Spread {{\n{}  base: \n{}{}}}\n", 
                    indent_str, indent_str, base.pretty_print(indent + 2), indent_str)
            }
            UnifiedReturnAST::PropertyAccess { base, property } => {
                format!("{}PropertyAccess {{\n{}  base: \n{}{}  property: {}\n{}}}\n",
                    indent_str, indent_str, base.pretty_print(indent + 2), 
                    indent_str, property, indent_str)
            }
            UnifiedReturnAST::ArrayAccess { base, index } => {
                format!("{}ArrayAccess {{\n{}  base: \n{}{}  index: \n{}{}}}\n",
                    indent_str, indent_str, base.pretty_print(indent + 2),
                    indent_str, index.pretty_print(indent + 2), indent_str)
            }
            UnifiedReturnAST::QuantifiedExtraction { base, target } => {
                let target_str = match target {
                    ExtractionTarget::Index(idx) => format!("Index({})", idx),
                    ExtractionTarget::First => "First".to_string(),
                    ExtractionTarget::Last => "Last".to_string(),
                };
                format!("{}QuantifiedExtraction {{\n{}  base: \n{}{}  target: {}\n{}}}\n",
                    indent_str, indent_str, base.pretty_print(indent + 2),
                    indent_str, target_str, indent_str)
            }
            UnifiedReturnAST::Passthrough => {
                format!("{}Passthrough\n", indent_str)
            }
        }
    }
    
    /// Generate Rust code from this return annotation AST
    /// This walks the AST and generates the appropriate code to build the parse result
    pub fn generate_code(&self, captured_vars: &[String], indent: &str, debug: bool) -> Result<String, String> {
        if debug {
            println!("[UnifiedReturnAST::generate_code] Generating code for: {:?}", self);
            println!("[UnifiedReturnAST::generate_code] Available captured vars: {:?}", captured_vars);
        }
        
        match self {
            UnifiedReturnAST::PositionalRef { index } => {
                // Reference to a captured parse result
                if *index > 0 && *index <= captured_vars.len() {
                    // Use the captured variable (already references sequence_elements[N])
                    let var_ref = &captured_vars[index - 1];
                    if debug {
                        println!("[UnifiedReturnAST::generate_code] PositionalRef ${} -> '{}'", index, var_ref);
                    }
                    // If it's a sequence element, get its content
                    if var_ref.starts_with("sequence_elements[") {
                        Ok(format!("{}.content.clone()", var_ref))
                    } else {
                        // For branch alternatives, it's already the content
                        Ok(var_ref.clone())
                    }
                } else {
                    Err(format!("Invalid positional reference: ${} (only {} captures available)", 
                        index, captured_vars.len()))
                }
            }
            
            UnifiedReturnAST::StringLiteral { value } => {
                Ok(format!("{}ParseContent::Terminal(r#\"{}\"#)", indent, value))
            }
            
            UnifiedReturnAST::NumberLiteral { value } => {
                Ok(format!("{}ParseContent::Terminal(r#\"{}\"#)", indent, value))
            }
            
            UnifiedReturnAST::BooleanLiteral { value } => {
                Ok(format!("{}ParseContent::Terminal(r#\"{}\"#)", indent, value))
            }
            
            UnifiedReturnAST::Array { elements } => {
                // Build a sequence node from elements
                let mut code = format!("ParseContent::Sequence(vec![");
                
                for (i, element) in elements.iter().enumerate() {
                    match element {
                        UnifiedReturnAST::Spread { base } => {
                            // Handle spread operator - unpack if it's a sequence
                            let base_code = base.generate_code(captured_vars, &format!("{}    ", indent), debug)?;
                            
                            // Generate code to spread the elements
                            code.push_str(&format!("\n{}    // Spread element\n", indent));
                            code.push_str(&format!("{}    ...(match {} {{\n", indent, base_code));
                            code.push_str(&format!("{}        ParseContent::Sequence(nodes) => nodes,\n", indent));
                            code.push_str(&format!("{}        other => vec![ParseNode {{ rule_name: \"spread_element\", content: other, span: 0..0 }}],\n", indent));
                            code.push_str(&format!("{}    }}),", indent));
                        }
                        _ => {
                            // Regular element
                            if i > 0 || matches!(elements.get(0), Some(UnifiedReturnAST::Spread { .. })) {
                                code.push_str(",");
                            }
                            code.push_str("\n");
                            let elem_code = element.generate_code(captured_vars, &format!("{}    ", indent), debug)?;
                            code.push_str(&format!("{}    ParseNode {{ rule_name: \"element_{}\", content: {}, span: 0..0 }}", 
                                indent, i, elem_code));
                        }
                    }
                }
                
                code.push_str(&format!("\n{}])", indent));
                Ok(code)
            }
            
            UnifiedReturnAST::Object { properties } => {
                // Generate code to build a JSON object at runtime with actual values
                // Note: This generates a block expression that evaluates to ParseContent
                let mut code = String::new();
                code.push_str("{\n");
                code.push_str(&format!("{}    // Building object from return annotation\n", indent));
                code.push_str(&format!("{}    let mut json_obj = serde_json::json!({{}});\n", indent));
                
                for (key, value) in properties {
                    // Generate code to extract the actual value at runtime
                    match value.as_ref() {
                        UnifiedReturnAST::StringLiteral { value: str_val } => {
                            // String literal - use as is
                            code.push_str(&format!("{}    json_obj[r#\"{}\"#] = serde_json::json!(r#\"{}\"#);\n", 
                                indent, key, str_val));
                        }
                        UnifiedReturnAST::PositionalRef { index } => {
                            // Generate code to extract from captured variable
                            if *index > 0 && *index <= captured_vars.len() {
                                let var_ref = &captured_vars[index - 1];
                                // Extract the actual content from the parsed element
                                if var_ref.starts_with("sequence_elements[") {
                                    // For sequence elements, we need to extract the content and convert to string
                                    code.push_str(&format!("{}    json_obj[r#\"{}\"#] = serde_json::json!(\n", indent, key));
                                    code.push_str(&format!("{}        match &{}.content {{\n", indent, var_ref));
                                    code.push_str(&format!("{}            ParseContent::Terminal(s) => s.to_string(),\n", indent));
                                    code.push_str(&format!("{}            ParseContent::Alternative(node) => {{\n", indent));
                                    code.push_str(&format!("{}                match &node.content {{\n", indent));
                                    code.push_str(&format!("{}                    ParseContent::Terminal(s) => s.to_string(),\n", indent));
                                    code.push_str(&format!("{}                    _ => format!(\"{{:?}}\", node.content)\n", indent));
                                    code.push_str(&format!("{}                }}\n", indent));
                                    code.push_str(&format!("{}            }}\n", indent));
                                    code.push_str(&format!("{}            _ => format!(\"{{:?}}\", {}.content)\n", indent, var_ref));
                                    code.push_str(&format!("{}        }}\n", indent));
                                    code.push_str(&format!("{}    );\n", indent));
                                } else {
                                    // Direct reference to content
                                    code.push_str(&format!("{}    json_obj[r#\"{}\"#] = serde_json::json!({}); \n", 
                                        indent, key, var_ref));
                                }
                            } else {
                                // Invalid index - use placeholder
                                code.push_str(&format!("{}    json_obj[r#\"{}\"#] = serde_json::json!(\"<invalid_ref_${}>\");\n", 
                                    indent, key, index));
                            }
                        }
                        UnifiedReturnAST::NumberLiteral { value: num } => {
                            code.push_str(&format!("{}    json_obj[r#\"{}\"#] = serde_json::json!({});\n", 
                                indent, key, num));
                        }
                        UnifiedReturnAST::BooleanLiteral { value: bool_val } => {
                            code.push_str(&format!("{}    json_obj[r#\"{}\"#] = serde_json::json!({});\n", 
                                indent, key, bool_val));
                        }
                        _ => {
                            // For complex nested values, recursively generate code
                            let nested_code = value.generate_code(captured_vars, &format!("{}        ", indent), debug)?;
                            code.push_str(&format!("{}    json_obj[r#\"{}\"#] = serde_json::json!({}); \n", 
                                indent, key, nested_code));
                        }
                    }
                }
                
                code.push_str(&format!("{}    let json_str = serde_json::to_string(&json_obj).unwrap_or_else(|_| \"{{}}\".to_string());\n", indent));
                code.push_str(&format!("{}    ParseContent::Terminal(json_str)\n", indent));
                code.push_str("}");
                Ok(code)
            }
            
            UnifiedReturnAST::Spread { base } => {
                // Spread is typically used within arrays, handled above
                // If used standalone, just return the base
                base.generate_code(captured_vars, indent, debug)
            }
            
            UnifiedReturnAST::PropertyAccess { base, property } => {
                // Generate property access code
                let base_code = base.generate_code(captured_vars, indent, debug)?;
                // For now, this is a placeholder - would need runtime reflection
                Ok(format!("{}// TODO: Property access .{} on {}\n{}ParseContent::Terminal(\"<property_access>\")", 
                    indent, property, base_code, indent))
            }
            
            UnifiedReturnAST::ArrayAccess { base, index } => {
                // Generate array access code
                let base_code = base.generate_code(captured_vars, indent, debug)?;
                let index_code = index.generate_code(captured_vars, indent, debug)?;
                // For now, this is a placeholder - would need runtime indexing
                Ok(format!("{}// TODO: Array access [{}] on {}\n{}ParseContent::Terminal(\"<array_access>\"",
                    indent, index_code, base_code, indent))
            }
            
            UnifiedReturnAST::QuantifiedExtraction { base, target } => {
                // Extract specific elements from a quantified group
                // The base should be a positional reference to a quantified capture
                if let UnifiedReturnAST::PositionalRef { index } = base.as_ref() {
                    if *index > 0 && *index <= captured_vars.len() {
                        let var_ref = &captured_vars[index - 1];
                        
                        // Generate code to extract from each repetition
                        let extraction_idx = match target {
                            ExtractionTarget::Index(idx) => *idx,
                            ExtractionTarget::First => 0,
                            ExtractionTarget::Last => {
                                // This would need runtime determination
                                // For now, generate placeholder
                                return Ok(format!("{}// TODO: Extract last element from quantified group\n{}ParseContent::Terminal(\"<last_extraction>\")",
                                    indent, indent));
                            }
                        };
                        
                        // Generate code to extract from a quantified result
                        // This assumes the quantified result is a Sequence of Sequences
                        let mut code = String::new();
                        code.push_str(&format!("{{\n"));
                        code.push_str(&format!("{}    // Extract element {} from each repetition\n", indent, extraction_idx));
                        code.push_str(&format!("{}    let extracted = match {} {{\n", indent, var_ref));
                        code.push_str(&format!("{}        ParseContent::Sequence(items) => {{\n", indent));
                        code.push_str(&format!("{}            items.iter().filter_map(|item| {{\n", indent));
                        code.push_str(&format!("{}                match &item.content {{\n", indent));
                        code.push_str(&format!("{}                    ParseContent::Sequence(subitems) if subitems.len() > {} => {{\n", indent, extraction_idx));
                        code.push_str(&format!("{}                        Some(subitems[{}].clone())\n", indent, extraction_idx));
                        code.push_str(&format!("{}                    }}\n", indent));
                        code.push_str(&format!("{}                    _ => None\n", indent));
                        code.push_str(&format!("{}                }}\n", indent));
                        code.push_str(&format!("{}            }}).collect::<Vec<_>>()\n", indent));
                        code.push_str(&format!("{}        }}\n", indent));
                        code.push_str(&format!("{}        _ => vec![]\n", indent));
                        code.push_str(&format!("{}    }};\n", indent));
                        code.push_str(&format!("{}    ParseContent::Sequence(extracted)\n", indent));
                        code.push_str(&format!("{}}}", indent));
                        
                        Ok(code)
                    } else {
                        Err(format!("Invalid positional reference in extraction: ${}", index))
                    }
                } else {
                    Err(format!("Quantified extraction requires a positional reference as base"))
                }
            }
            
            UnifiedReturnAST::Passthrough => {
                // Default behavior - return the last captured element or first if only one
                if !captured_vars.is_empty() {
                    let var_ref = captured_vars.last().unwrap();
                    if var_ref.starts_with("sequence_elements[") {
                        Ok(format!("{}.content.clone()", var_ref))
                    } else {
                        Ok(var_ref.clone())
                    }
                } else {
                    Ok(format!("{}ParseContent::Terminal(\"\")", indent))
                }
            }
        }
    }
}

impl fmt::Display for UnifiedReturnAST {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pretty_print(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_positional_ref() {
        let ast = UnifiedReturnAST::parse_bootstrap("$1", false).unwrap();
        assert_eq!(ast, UnifiedReturnAST::PositionalRef { index: 1 });
        
        let ast = UnifiedReturnAST::parse_bootstrap("$42", false).unwrap();
        assert_eq!(ast, UnifiedReturnAST::PositionalRef { index: 42 });
    }
    
    #[test]
    fn test_parse_spread() {
        let ast = UnifiedReturnAST::parse_bootstrap("$3*", false).unwrap();
        assert_eq!(ast, UnifiedReturnAST::Spread {
            base: Box::new(UnifiedReturnAST::PositionalRef { index: 3 })
        });
    }
    
    #[test]
    fn test_parse_array() {
        let ast = UnifiedReturnAST::parse_bootstrap("[$1, $2]", false).unwrap();
        match ast {
            UnifiedReturnAST::Array { elements } => {
                assert_eq!(elements.len(), 2);
                assert_eq!(elements[0], UnifiedReturnAST::PositionalRef { index: 1 });
                assert_eq!(elements[1], UnifiedReturnAST::PositionalRef { index: 2 });
            }
            _ => panic!("Expected Array"),
        }
    }
    
    #[test]
    fn test_parse_array_with_spread() {
        let ast = UnifiedReturnAST::parse_bootstrap("[$1, $3*]", false).unwrap();
        match ast {
            UnifiedReturnAST::Array { elements } => {
                assert_eq!(elements.len(), 2);
                assert_eq!(elements[0], UnifiedReturnAST::PositionalRef { index: 1 });
                assert!(matches!(elements[1], UnifiedReturnAST::Spread { .. }));
            }
            _ => panic!("Expected Array"),
        }
    }
    
    #[test]
    fn test_parse_object() {
        let ast = UnifiedReturnAST::parse_bootstrap(r#"{type: "array", element: $3}"#, false).unwrap();
        match ast {
            UnifiedReturnAST::Object { properties } => {
                assert_eq!(properties.len(), 2);
                assert!(properties.contains_key("type"));
                assert!(properties.contains_key("element"));
                
                match properties.get("type").unwrap().as_ref() {
                    UnifiedReturnAST::StringLiteral { value } => assert_eq!(value, "array"),
                    _ => panic!("Expected StringLiteral for 'type'"),
                }
                
                match properties.get("element").unwrap().as_ref() {
                    UnifiedReturnAST::PositionalRef { index } => assert_eq!(*index, 3),
                    _ => panic!("Expected PositionalRef for 'element'"),
                }
            }
            _ => panic!("Expected Object"),
        }
    }
    
    #[test]
    fn test_code_generation() {
        let captured_vars = vec!["sequence_elements[0]".to_string(), "sequence_elements[1]".to_string()];
        
        // Test positional reference
        let ast = UnifiedReturnAST::PositionalRef { index: 1 };
        let code = ast.generate_code(&captured_vars, "", false).unwrap();
        assert_eq!(code, "sequence_elements[0].content.clone()");
        
        // Test string literal
        let ast = UnifiedReturnAST::StringLiteral { value: "test".to_string() };
        let code = ast.generate_code(&captured_vars, "", false).unwrap();
        assert_eq!(code, "ParseContent::Terminal(r#\"test\"#)");
    }
    
    #[test]
    fn test_parse_extraction_operators() {
        // Test $2::2
        let ast = UnifiedReturnAST::parse_bootstrap("$2::2", false).unwrap();
        match ast {
            UnifiedReturnAST::QuantifiedExtraction { base, target } => {
                assert!(matches!(base.as_ref(), UnifiedReturnAST::PositionalRef { index: 2 }));
                assert_eq!(target, ExtractionTarget::Index(2));
            }
            _ => panic!("Expected QuantifiedExtraction"),
        }
        
        // Test $2::first
        let ast = UnifiedReturnAST::parse_bootstrap("$2::first", false).unwrap();
        match ast {
            UnifiedReturnAST::QuantifiedExtraction { base, target } => {
                assert!(matches!(base.as_ref(), UnifiedReturnAST::PositionalRef { index: 2 }));
                assert_eq!(target, ExtractionTarget::First);
            }
            _ => panic!("Expected QuantifiedExtraction"),
        }
        
        // Test $2::last
        let ast = UnifiedReturnAST::parse_bootstrap("$2::last", false).unwrap();
        match ast {
            UnifiedReturnAST::QuantifiedExtraction { base, target } => {
                assert!(matches!(base.as_ref(), UnifiedReturnAST::PositionalRef { index: 2 }));
                assert_eq!(target, ExtractionTarget::Last);
            }
            _ => panic!("Expected QuantifiedExtraction"),
        }
        
        // Test $2::1* (extraction with spread)
        let ast = UnifiedReturnAST::parse_bootstrap("$2::1*", false).unwrap();
        match ast {
            UnifiedReturnAST::Spread { base } => {
                match base.as_ref() {
                    UnifiedReturnAST::QuantifiedExtraction { base: inner_base, target } => {
                        assert!(matches!(inner_base.as_ref(), UnifiedReturnAST::PositionalRef { index: 2 }));
                        assert_eq!(*target, ExtractionTarget::Index(1));
                    }
                    _ => panic!("Expected QuantifiedExtraction inside Spread"),
                }
            }
            _ => panic!("Expected Spread"),
        }
    }
}
