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

use super::Logger;

/// Extraction target for quantified groups
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExtractionTarget {
    /// Extract by index (1-based): $2::2 means second element (array index 1)
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
        index: usize, // 1-based index (e.g., $3 has index: 3)
    },

    /// String literal: "array", "object", etc.
    StringLiteral { value: String },

    /// Number literal: 42, 3.14
    NumberLiteral { value: f64 },

    /// Boolean literal: true or false
    BooleanLiteral { value: bool },

    /// Identifier literal: foo, bar_baz
    Identifier { name: String },

    /// Object with key-value pairs: {type: "array", element: $3}
    Object {
        properties: HashMap<String, Box<UnifiedReturnAST>>,
    },

    /// Array: [$1, $2, "literal"]
    Array { elements: Vec<UnifiedReturnAST> },

    /// Spread operator: $4* in [$1, $4*]
    /// Unpacks a sequence into individual elements
    Spread { base: Box<UnifiedReturnAST> },

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
    pub fn parse_bootstrap(
        annotation: &str,
        logger: &dyn Logger,
    ) -> Result<UnifiedReturnAST, String> {
        if logger.is_enabled() {
            logger.log_info(
                "unified_return_ast.rs",
                line!(),
                &format!("Parsing return annotation: '{}'", annotation),
            );
        }

        // Remove leading "-> " if present after leading whitespace normalization.
        let normalized_leading = annotation.trim_start();
        let cleaned = if normalized_leading.starts_with("-> ") {
            &normalized_leading[3..]
        } else if normalized_leading.starts_with("->") {
            &normalized_leading[2..]
        } else {
            annotation
        }
        .trim();

        // Empty annotation means passthrough
        if cleaned.is_empty() {
            return Ok(UnifiedReturnAST::Passthrough);
        }

        let result = Self::parse_value(cleaned, logger)?;

        if logger.is_enabled() {
            logger.log_success(
                "unified_return_ast.rs",
                line!(),
                &format!("Parsed AST: {}", result.pretty_print(0).trim()),
            );
        }

        Ok(result)
    }

    fn parse_value(input: &str, logger: &dyn Logger) -> Result<UnifiedReturnAST, String> {
        let trimmed = input.trim();

        if logger.is_enabled() {
            logger.log_debug(
                "unified_return_ast.rs",
                line!(),
                &format!("Parsing value: '{}'", trimmed),
            );
        }

        // Check for positional reference $N (with potential modifiers)
        if trimmed.starts_with('$') {
            return Self::parse_positional_ref(trimmed, logger);
        }

        // Parenthesized expression, potentially followed by postfix accessors
        if trimmed.starts_with('(') {
            if let Some(close_idx) = Self::find_matching_closer(trimmed, '(', ')') {
                if close_idx == trimmed.len() - 1 {
                    return Self::parse_value(&trimmed[1..close_idx], logger);
                }
                let base = Self::parse_value(&trimmed[1..close_idx], logger)?;
                return Self::parse_postfix_chain(base, &trimmed[close_idx + 1..], logger);
            }
        }

        // Check for string literal ("..." or '...')
        if let Some(value) = Self::parse_quoted_string(trimmed) {
            return Ok(UnifiedReturnAST::StringLiteral {
                value,
            });
        }

        // Check for object {...}
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            return Self::parse_object(&trimmed[1..trimmed.len() - 1], logger);
        }

        // Check for array [...]
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            return Self::parse_array(&trimmed[1..trimmed.len() - 1], logger);
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

        // Check for identifier literal
        if Self::is_identifier_literal(trimmed) {
            return Ok(UnifiedReturnAST::Identifier {
                name: trimmed.to_string(),
            });
        }

        Err(format!("Unable to parse return value: '{}'", trimmed))
    }

    fn parse_positional_ref(input: &str, logger: &dyn Logger) -> Result<UnifiedReturnAST, String> {
        if logger.is_enabled() {
            logger.log_debug(
                "unified_return_ast.rs",
                line!(),
                &format!("Parsing positional reference: '{}'", input),
            );
        }

        let without_dollar = &input[1..];
        let mut index_end = 0usize;
        let mut chars = without_dollar.char_indices();

        if let Some((_, first)) = chars.next() {
            if first == '+' || first == '-' {
                index_end = 1;
            }
        }

        let digit_start = index_end;
        for (idx, ch) in without_dollar[digit_start..].char_indices() {
            if ch.is_ascii_digit() {
                index_end = digit_start + idx + ch.len_utf8();
            } else {
                break;
            }
        }

        let num_str = &without_dollar[..index_end];
        let remaining = &without_dollar[index_end..];

        if logger.is_enabled() {
            logger.log_debug(
                "unified_return_ast.rs",
                line!(),
                &format!(
                    "  Extracted number: '{}', remaining: '{}'",
                    num_str, remaining
                ),
            );
        }

        if num_str.is_empty() || index_end == digit_start {
            logger.log_error(
                "unified_return_ast.rs",
                line!(),
                &format!("Invalid positional reference: '{}'", input),
            );
            return Err(format!("Invalid positional reference: '{}'", input));
        }

        let signed_index = num_str.parse::<i64>().map_err(|_| {
            logger.log_error(
                "unified_return_ast.rs",
                line!(),
                &format!("Invalid positional index: '{}'", num_str),
            );
            format!("Invalid positional index: '{}'", num_str)
        })?;
        if signed_index < 0 {
            logger.log_error(
                "unified_return_ast.rs",
                line!(),
                &format!("Invalid positional index: '{}'", num_str),
            );
            return Err(format!("Invalid positional index: '{}'", num_str));
        }

        let base = UnifiedReturnAST::PositionalRef {
            index: signed_index as usize,
        };
        let parsed = Self::parse_postfix_chain(base, remaining, logger)?;

        if logger.is_enabled() {
            logger.log_debug(
                "unified_return_ast.rs",
                line!(),
                &format!("  Returning: {}", parsed.pretty_print(0).trim()),
            );
        }

        Ok(parsed)
    }

    fn parse_postfix_chain(
        mut base: UnifiedReturnAST,
        mut remaining: &str,
        logger: &dyn Logger,
    ) -> Result<UnifiedReturnAST, String> {
        while !remaining.is_empty() {
            if let Some(after_extraction_prefix) = remaining.strip_prefix("::") {
                let mut target_end = 0usize;
                for (idx, ch) in after_extraction_prefix.char_indices() {
                    if ch.is_ascii_alphanumeric() || ch == '_' {
                        target_end = idx + ch.len_utf8();
                    } else {
                        break;
                    }
                }
                if target_end == 0 {
                    logger.log_error(
                        "unified_return_ast.rs",
                        line!(),
                        &format!("Invalid extraction target: '{}'", after_extraction_prefix),
                    );
                    return Err(format!(
                        "Invalid extraction target: '{}'",
                        after_extraction_prefix
                    ));
                }

                let target_str = &after_extraction_prefix[..target_end];
                let target = match target_str {
                    "first" => ExtractionTarget::First,
                    "last" => ExtractionTarget::Last,
                    _ => match target_str.parse::<usize>() {
                        Ok(user_idx) if user_idx > 0 => ExtractionTarget::Index(user_idx - 1),
                        _ => {
                            logger.log_error(
                                "unified_return_ast.rs",
                                line!(),
                                &format!("Invalid extraction target: '{}'", target_str),
                            );
                            return Err(format!("Invalid extraction target: '{}'", target_str));
                        }
                    },
                };

                base = UnifiedReturnAST::QuantifiedExtraction {
                    base: Box::new(base),
                    target,
                };
                remaining = &after_extraction_prefix[target_end..];
                continue;
            }

            if let Some(rest) = remaining.strip_prefix('*') {
                if !rest.is_empty() {
                    logger.log_error(
                        "unified_return_ast.rs",
                        line!(),
                        &format!("Invalid positional reference modifier: '{}'", remaining),
                    );
                    return Err(format!(
                        "Invalid positional reference modifier: '{}'",
                        remaining
                    ));
                }
                base = UnifiedReturnAST::Spread {
                    base: Box::new(base),
                };
                remaining = rest;
                continue;
            }

            if let Some(property_source) = remaining.strip_prefix('.') {
                let mut property_end = 0usize;
                for (idx, ch) in property_source.char_indices() {
                    let valid = if idx == 0 {
                        ch == '_' || ch.is_ascii_alphabetic()
                    } else {
                        ch == '_' || ch.is_ascii_alphanumeric()
                    };
                    if valid {
                        property_end = idx + ch.len_utf8();
                    } else {
                        break;
                    }
                }
                if property_end == 0 {
                    logger.log_error(
                        "unified_return_ast.rs",
                        line!(),
                        &format!("Invalid positional reference modifier: '{}'", remaining),
                    );
                    return Err(format!(
                        "Invalid positional reference modifier: '{}'",
                        remaining
                    ));
                }

                let property = property_source[..property_end].to_string();
                base = UnifiedReturnAST::PropertyAccess {
                    base: Box::new(base),
                    property,
                };
                remaining = &property_source[property_end..];
                continue;
            }

            if remaining.starts_with('[') {
                let Some(close_idx) = Self::find_matching_closer(remaining, '[', ']') else {
                    logger.log_error(
                        "unified_return_ast.rs",
                        line!(),
                        &format!("Unclosed array access: '{}'", remaining),
                    );
                    return Err(format!("Unclosed array access: '{}'", remaining));
                };

                let index_expr = &remaining[1..close_idx];
                let index = Self::parse_value(index_expr, logger)?;
                base = UnifiedReturnAST::ArrayAccess {
                    base: Box::new(base),
                    index: Box::new(index),
                };
                remaining = &remaining[close_idx + 1..];
                continue;
            }

            logger.log_error(
                "unified_return_ast.rs",
                line!(),
                &format!("Invalid positional reference modifier: '{}'", remaining),
            );
            return Err(format!(
                "Invalid positional reference modifier: '{}'",
                remaining
            ));
        }

        Ok(base)
    }

    fn find_matching_closer(input: &str, open: char, close: char) -> Option<usize> {
        if !input.starts_with(open) {
            return None;
        }
        let mut depth = 0usize;
        let mut string_delim: Option<char> = None;
        let mut escape_next = false;

        for (idx, ch) in input.char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }

            if let Some(delim) = string_delim {
                match ch {
                    '\\' => escape_next = true,
                    c if c == delim => string_delim = None,
                    _ => {}
                }
                continue;
            }

            match ch {
                '"' | '\'' => string_delim = Some(ch),
                c if c == open => depth += 1,
                c if c == close => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return Some(idx);
                    }
                }
                _ => {}
            }
        }

        None
    }

    fn parse_object(content: &str, logger: &dyn Logger) -> Result<UnifiedReturnAST, String> {
        if logger.is_enabled() {
            logger.log_debug(
                "unified_return_ast.rs",
                line!(),
                &format!("Parsing object content: '{}'", content),
            );
        }

        let mut properties = HashMap::new();

        // Split by commas, but respect nested structures and reject empty segments.
        let pairs = Self::split_respecting_nesting_strict(content, ',')?;

        for pair in pairs {
            let Some((raw_key, raw_value)) = Self::split_object_property(&pair) else {
                logger.log_error(
                    "unified_return_ast.rs",
                    line!(),
                    &format!("Invalid object property: '{}'", pair),
                );
                return Err(format!("Invalid object property: '{}'", pair));
            };

            // Parse key (remove quotes if present)
            let key = raw_key.trim();
            let key = Self::parse_quoted_string(key).unwrap_or_else(|| key.to_string());

            // Parse value
            let value = Self::parse_value(raw_value.trim(), logger)?;
            properties.insert(key, Box::new(value));
        }

        Ok(UnifiedReturnAST::Object { properties })
    }

    fn parse_array(content: &str, logger: &dyn Logger) -> Result<UnifiedReturnAST, String> {
        if logger.is_enabled() {
            logger.log_debug(
                "unified_return_ast.rs",
                line!(),
                &format!("Parsing array content: '{}'", content),
            );
        }

        let mut elements = Vec::new();

        if !content.trim().is_empty() {
            let items = Self::split_respecting_nesting_strict(content, ',')?;

            for item in items {
                let trimmed = item.trim();

                // Check for spread operator at the end
                if trimmed.ends_with('*') && !Self::is_quoted_literal(trimmed) {
                    // It's a spread, but only if not inside a string
                    let base_str = &trimmed[..trimmed.len() - 1];
                    let base = Self::parse_value(base_str, logger)?;
                    elements.push(UnifiedReturnAST::Spread {
                        base: Box::new(base),
                    });
                } else {
                    elements.push(Self::parse_value(trimmed, logger)?);
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
        let mut string_delim: Option<char> = None;
        let mut escape_next = false;

        for ch in input.chars() {
            if escape_next {
                current.push(ch);
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if string_delim.is_some() => escape_next = true,
                '"' | '\'' if string_delim.is_none() => string_delim = Some(ch),
                c if Some(c) == string_delim => string_delim = None,
                '[' | '{' if string_delim.is_none() => depth += 1,
                ']' | '}' if string_delim.is_none() => depth -= 1,
                c if c == delimiter && depth == 0 && string_delim.is_none() => {
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

    /// Strict variant of split_respecting_nesting:
    /// rejects leading/trailing/consecutive top-level delimiters.
    fn split_respecting_nesting_strict(input: &str, delimiter: char) -> Result<Vec<String>, String> {
        let mut result = Vec::new();
        let mut current = String::new();
        let mut depth = 0;
        let mut string_delim: Option<char> = None;
        let mut escape_next = false;
        let mut seen_delimiter = false;

        for ch in input.chars() {
            if escape_next {
                current.push(ch);
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if string_delim.is_some() => escape_next = true,
                '"' | '\'' if string_delim.is_none() => string_delim = Some(ch),
                c if Some(c) == string_delim => string_delim = None,
                '[' | '{' if string_delim.is_none() => depth += 1,
                ']' | '}' if string_delim.is_none() => depth -= 1,
                c if c == delimiter && depth == 0 && string_delim.is_none() => {
                    seen_delimiter = true;
                    if current.trim().is_empty() {
                        return Err(format!(
                            "Empty segment in '{}' separated list: '{}'",
                            delimiter, input
                        ));
                    }
                    result.push(current.trim().to_string());
                    current.clear();
                    continue;
                }
                _ => {}
            }
            current.push(ch);
        }

        if current.trim().is_empty() {
            if seen_delimiter {
                return Err(format!(
                    "Empty segment in '{}' separated list: '{}'",
                    delimiter, input
                ));
            }
        } else {
            result.push(current.trim().to_string());
        }

        Ok(result)
    }

    /// Split a single object property into key/value at the first top-level ':'
    /// while ignoring extraction operators ('::') and nested structures.
    fn split_object_property(input: &str) -> Option<(String, String)> {
        let mut depth = 0;
        let mut string_delim: Option<char> = None;
        let mut escape_next = false;

        for (idx, ch) in input.char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if string_delim.is_some() => {
                    escape_next = true;
                }
                '"' | '\'' if string_delim.is_none() => {
                    string_delim = Some(ch);
                }
                c if Some(c) == string_delim => {
                    string_delim = None;
                }
                '[' | '{' if string_delim.is_none() => {
                    depth += 1;
                }
                ']' | '}' if string_delim.is_none() => {
                    if depth > 0 {
                        depth -= 1;
                    }
                }
                ':' if string_delim.is_none() && depth == 0 => {
                    // Skip extraction operator (::) delimiters in values like "$2::first"
                    let prev_is_colon = idx > 0 && input.as_bytes()[idx - 1] == b':';
                    let next_is_colon =
                        (idx + 1) < input.len() && input.as_bytes()[idx + 1] == b':';
                    if prev_is_colon || next_is_colon {
                        continue;
                    }

                    let key = input[..idx].trim().to_string();
                    let value = input[idx + 1..].trim().to_string();
                    if key.is_empty() || value.is_empty() {
                        return None;
                    }
                    return Some((key, value));
                }
                _ => {}
            }
        }

        None
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
            UnifiedReturnAST::Identifier { name } => {
                format!("{}Identifier({})\n", indent_str, name)
            }
            UnifiedReturnAST::Object { properties } => {
                let mut result = format!("{}Object {{\n", indent_str);
                for (key, value) in properties {
                    result.push_str(&format!(
                        "{}  {}: \n{}",
                        indent_str,
                        key,
                        value.pretty_print(indent + 2)
                    ));
                }
                result.push_str(&format!("{}}}\n", indent_str));
                result
            }
            UnifiedReturnAST::Array { elements } => {
                let mut result = format!("{}Array [\n", indent_str);
                for (i, elem) in elements.iter().enumerate() {
                    result.push_str(&format!(
                        "{}  [{}]: \n{}",
                        indent_str,
                        i,
                        elem.pretty_print(indent + 2)
                    ));
                }
                result.push_str(&format!("{}]\n", indent_str));
                result
            }
            UnifiedReturnAST::Spread { base } => {
                format!(
                    "{}Spread {{\n{}  base: \n{}{}}}\n",
                    indent_str,
                    indent_str,
                    base.pretty_print(indent + 2),
                    indent_str
                )
            }
            UnifiedReturnAST::PropertyAccess { base, property } => {
                format!(
                    "{}PropertyAccess {{\n{}  base: \n{}{}  property: {}\n{}}}\n",
                    indent_str,
                    indent_str,
                    base.pretty_print(indent + 2),
                    indent_str,
                    property,
                    indent_str
                )
            }
            UnifiedReturnAST::ArrayAccess { base, index } => {
                format!(
                    "{}ArrayAccess {{\n{}  base: \n{}{}  index: \n{}{}}}\n",
                    indent_str,
                    indent_str,
                    base.pretty_print(indent + 2),
                    indent_str,
                    index.pretty_print(indent + 2),
                    indent_str
                )
            }
            UnifiedReturnAST::QuantifiedExtraction { base, target } => {
                let target_str = match target {
                    ExtractionTarget::Index(idx) => format!("Index({})", idx),
                    ExtractionTarget::First => "First".to_string(),
                    ExtractionTarget::Last => "Last".to_string(),
                };
                format!(
                    "{}QuantifiedExtraction {{\n{}  base: \n{}{}  target: {}\n{}}}\n",
                    indent_str,
                    indent_str,
                    base.pretty_print(indent + 2),
                    indent_str,
                    target_str,
                    indent_str
                )
            }
            UnifiedReturnAST::Passthrough => {
                format!("{}Passthrough\n", indent_str)
            }
        }
    }

    /// Generate Rust code from this return annotation AST
    /// This walks the AST and generates the appropriate code to build the parse result
    pub fn generate_code(
        &self,
        captured_vars: &[String],
        indent: &str,
        logger: &dyn Logger,
    ) -> Result<String, String> {
        if logger.is_enabled() {
            logger.log_debug(
                "unified_return_ast.rs",
                line!(),
                &format!("Generating code for: {:?}", self),
            );
            logger.log_debug(
                "unified_return_ast.rs",
                line!(),
                &format!("Available captured vars: {:?}", captured_vars),
            );
        }

        match self {
            UnifiedReturnAST::PositionalRef { index } => {
                // Reference to a captured parse result
                if *index > 0 && *index <= captured_vars.len() {
                    // Use the captured variable (already references sequence_elements[N])
                    let var_ref = &captured_vars[index - 1];
                    if logger.is_enabled() {
                        logger.log_debug(
                            "unified_return_ast.rs",
                            line!(),
                            &format!("PositionalRef ${} -> '{}'", index, var_ref),
                        );
                    }
                    // If it's a sequence element, get its content
                    if var_ref.starts_with("sequence_elements[") {
                        Ok(format!("{}.content.clone()", var_ref))
                    } else {
                        // For branch alternatives, it's already the content
                        Ok(var_ref.clone())
                    }
                } else {
                    Err(format!(
                        "Invalid positional reference: ${} (only {} captures available)",
                        index,
                        captured_vars.len()
                    ))
                }
            }

            UnifiedReturnAST::StringLiteral { value } => Ok(format!(
                "{}ParseContent::Terminal(r#\"{}\"#)",
                indent, value
            )),

            UnifiedReturnAST::NumberLiteral { value } => Ok(format!(
                "{}ParseContent::Terminal(r#\"{}\"#)",
                indent, value
            )),

            UnifiedReturnAST::BooleanLiteral { value } => Ok(format!(
                "{}ParseContent::Terminal(r#\"{}\"#)",
                indent, value
            )),

            UnifiedReturnAST::Identifier { name } => Ok(format!(
                "{}ParseContent::Terminal(r#\"{}\"#)",
                indent, name
            )),

            UnifiedReturnAST::Array { elements } => {
                // Build a sequence node from elements
                let mut code = format!("ParseContent::Sequence(vec![");

                for (i, element) in elements.iter().enumerate() {
                    match element {
                        UnifiedReturnAST::Spread { base } => {
                            // Handle spread operator - unpack if it's a sequence
                            let base_code = base.generate_code(
                                captured_vars,
                                &format!("{}    ", indent),
                                logger,
                            )?;

                            // Generate code to spread the elements
                            code.push_str(&format!("\n{}    // Spread element\n", indent));
                            code.push_str(&format!("{}    ...(match {} {{\n", indent, base_code));
                            code.push_str(&format!(
                                "{}        ParseContent::Sequence(nodes) => nodes,\n",
                                indent
                            ));
                            code.push_str(&format!("{}        other => vec![ParseNode {{ rule_name: \"spread_element\", content: other, span: 0..0 }}],\n", indent));
                            code.push_str(&format!("{}    }}),", indent));
                        }
                        _ => {
                            // Regular element
                            if i > 0
                                || matches!(elements.get(0), Some(UnifiedReturnAST::Spread { .. }))
                            {
                                code.push_str(",");
                            }
                            code.push_str("\n");
                            let elem_code = element.generate_code(
                                captured_vars,
                                &format!("{}    ", indent),
                                logger,
                            )?;
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
                // Note: This needs to be generated as inline code suitable for assignment to let result = ...
                // We'll use an immediately-invoked closure to maintain variable scoping
                let mut code = String::new();
                code.push_str("(|| {\n");
                code.push_str(&format!(
                    "{}    // Building object from return annotation\n",
                    indent
                ));
                code.push_str(&format!(
                    "{}    let mut json_obj = serde_json::json!({{}});\n",
                    indent
                ));

                for (key, value) in properties {
                    // Generate code to extract the actual value at runtime
                    match value.as_ref() {
                        UnifiedReturnAST::StringLiteral { value: str_val } => {
                            // String literal - use as is
                            code.push_str(&format!(
                                "{}    json_obj[r#\"{}\"#] = serde_json::json!(r#\"{}\"#);\n",
                                indent, key, str_val
                            ));
                        }
                        UnifiedReturnAST::PositionalRef { index } => {
                            // Generate code to extract from captured variable
                            if *index > 0 && *index <= captured_vars.len() {
                                let var_ref = &captured_vars[index - 1];
                                // Extract the actual content from the parsed element
                                if var_ref.starts_with("sequence_elements[") {
                                    // For sequence elements, we need to extract the content and convert to string
                                    code.push_str(&format!(
                                        "{}    json_obj[r#\"{}\"#] = serde_json::json!(\n",
                                        indent, key
                                    ));
                                    code.push_str(&format!(
                                        "{}        match &{}.content {{\n",
                                        indent, var_ref
                                    ));
                                    code.push_str(&format!("{}            ParseContent::Terminal(s) => s.to_string(),\n", indent));
                                    code.push_str(&format!(
                                        "{}            ParseContent::Alternative(node) => {{\n",
                                        indent
                                    ));
                                    code.push_str(&format!(
                                        "{}                match &node.content {{\n",
                                        indent
                                    ));
                                    code.push_str(&format!("{}                    ParseContent::Terminal(s) => s.to_string(),\n", indent));
                                    code.push_str(&format!("{}                    _ => format!(\"{{:?}}\", node.content)\n", indent));
                                    code.push_str(&format!("{}                }}\n", indent));
                                    code.push_str(&format!("{}            }}\n", indent));
                                    code.push_str(&format!(
                                        "{}            _ => format!(\"{{:?}}\", {}.content)\n",
                                        indent, var_ref
                                    ));
                                    code.push_str(&format!("{}        }}\n", indent));
                                    code.push_str(&format!("{}    );\n", indent));
                                } else {
                                    // Direct reference to content
                                    code.push_str(&format!(
                                        "{}    json_obj[r#\"{}\"#] = serde_json::json!({}); \n",
                                        indent, key, var_ref
                                    ));
                                }
                            } else {
                                // Invalid index - use placeholder
                                code.push_str(&format!("{}    json_obj[r#\"{}\"#] = serde_json::json!(\"<invalid_ref_${}>\");\n", 
                                    indent, key, index));
                            }
                        }
                        UnifiedReturnAST::NumberLiteral { value: num } => {
                            code.push_str(&format!(
                                "{}    json_obj[r#\"{}\"#] = serde_json::json!({});\n",
                                indent, key, num
                            ));
                        }
                        UnifiedReturnAST::BooleanLiteral { value: bool_val } => {
                            code.push_str(&format!(
                                "{}    json_obj[r#\"{}\"#] = serde_json::json!({});\n",
                                indent, key, bool_val
                            ));
                        }
                        UnifiedReturnAST::Identifier { name } => {
                            code.push_str(&format!(
                                "{}    json_obj[r#\"{}\"#] = serde_json::json!(r#\"{}\"#);\n",
                                indent, key, name
                            ));
                        }
                        _ => {
                            // For complex nested values, recursively generate code
                            let nested_code = value.generate_code(
                                captured_vars,
                                &format!("{}        ", indent),
                                logger,
                            )?;
                            code.push_str(&format!(
                                "{}    json_obj[r#\"{}\"#] = serde_json::json!({}); \n",
                                indent, key, nested_code
                            ));
                        }
                    }
                }

                code.push_str(&format!("{}    let json_str = serde_json::to_string(&json_obj).unwrap_or_else(|_| \"{{}}\".to_string());\n", indent));
                code.push_str(&format!("{}    ParseContent::Terminal(json_str)\n", indent));
                code.push_str("})()");
                Ok(code)
            }

            UnifiedReturnAST::Spread { base } => {
                // Spread is typically used within arrays, handled above
                // If used standalone, just return the base
                base.generate_code(captured_vars, indent, logger)
            }

            UnifiedReturnAST::PropertyAccess { base, property } => {
                // Generate property access code
                let base_code = base.generate_code(captured_vars, indent, logger)?;
                // For now, this is a placeholder - would need runtime reflection
                Ok(format!(
                    "{}// TODO: Property access .{} on {}\n{}ParseContent::Terminal(\"<property_access>\")",
                    indent, property, base_code, indent
                ))
            }

            UnifiedReturnAST::ArrayAccess { base, index } => {
                // Generate array access code
                let base_code = base.generate_code(captured_vars, indent, logger)?;
                let index_code = index.generate_code(captured_vars, indent, logger)?;
                // For now, this is a placeholder - would need runtime indexing
                Ok(format!(
                    "{}// TODO: Array access [{}] on {}\n{}ParseContent::Terminal(\"<array_access>\"",
                    indent, index_code, base_code, indent
                ))
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
                                return Ok(format!(
                                    "{}// TODO: Extract last element from quantified group\n{}ParseContent::Terminal(\"<last_extraction>\")",
                                    indent, indent
                                ));
                            }
                        };

                        // Generate code to extract from a quantified result
                        // This assumes the quantified result is a Sequence of Sequences
                        let mut code = String::new();
                        code.push_str(&format!("{{\n"));
                        code.push_str(&format!(
                            "{}    // Extract element {} from each repetition\n",
                            indent, extraction_idx
                        ));
                        code.push_str(&format!(
                            "{}    let extracted = match {} {{\n",
                            indent, var_ref
                        ));
                        code.push_str(&format!(
                            "{}        ParseContent::Sequence(items) => {{\n",
                            indent
                        ));
                        code.push_str(&format!(
                            "{}            items.iter().filter_map(|item| {{\n",
                            indent
                        ));
                        code.push_str(&format!(
                            "{}                match &item.content {{\n",
                            indent
                        ));
                        code.push_str(&format!("{}                    ParseContent::Sequence(subitems) if subitems.len() > {} => {{\n", indent, extraction_idx));
                        code.push_str(&format!(
                            "{}                        Some(subitems[{}].clone())\n",
                            indent, extraction_idx
                        ));
                        code.push_str(&format!("{}                    }}\n", indent));
                        code.push_str(&format!("{}                    _ => None\n", indent));
                        code.push_str(&format!("{}                }}\n", indent));
                        code.push_str(&format!("{}            }}).collect::<Vec<_>>()\n", indent));
                        code.push_str(&format!("{}        }}\n", indent));
                        code.push_str(&format!("{}        _ => vec![]\n", indent));
                        code.push_str(&format!("{}    }};\n", indent));
                        code.push_str(&format!(
                            "{}    ParseContent::Sequence(extracted)\n",
                            indent
                        ));
                        code.push_str(&format!("{}}}", indent));

                        Ok(code)
                    } else {
                        Err(format!(
                            "Invalid positional reference in extraction: ${}",
                            index
                        ))
                    }
                } else {
                    Err(format!(
                        "Quantified extraction requires a positional reference as base"
                    ))
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

    fn parse_quoted_string(input: &str) -> Option<String> {
        if input.len() < 2 {
            return None;
        }
        let first = input.chars().next()?;
        let last = input.chars().last()?;
        if (first == '"' || first == '\'') && first == last {
            return Some(input[1..input.len() - 1].to_string());
        }
        None
    }

    fn is_quoted_literal(input: &str) -> bool {
        Self::parse_quoted_string(input).is_some()
    }

    fn is_identifier_literal(input: &str) -> bool {
        let mut chars = input.chars();
        let Some(first) = chars.next() else {
            return false;
        };
        if !(first == '_' || first.is_ascii_alphabetic()) {
            return false;
        }
        chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
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
        let logger = crate::test_runner::NoOpLogger;
        let ast = UnifiedReturnAST::parse_bootstrap("$1", &logger).unwrap();
        assert_eq!(ast, UnifiedReturnAST::PositionalRef { index: 1 });

        let ast = UnifiedReturnAST::parse_bootstrap("$42", &logger).unwrap();
        assert_eq!(ast, UnifiedReturnAST::PositionalRef { index: 42 });
    }

    #[test]
    fn test_parse_spread() {
        let logger = crate::test_runner::NoOpLogger;
        let ast = UnifiedReturnAST::parse_bootstrap("$3*", &logger).unwrap();
        assert_eq!(
            ast,
            UnifiedReturnAST::Spread {
                base: Box::new(UnifiedReturnAST::PositionalRef { index: 3 })
            }
        );
    }

    #[test]
    fn test_parse_array() {
        let logger = crate::test_runner::NoOpLogger;
        let ast = UnifiedReturnAST::parse_bootstrap("[$1, $2]", &logger).unwrap();
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
        let logger = crate::test_runner::NoOpLogger;
        let ast = UnifiedReturnAST::parse_bootstrap("[$1, $3*]", &logger).unwrap();
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
        let logger = crate::test_runner::NoOpLogger;
        let ast =
            UnifiedReturnAST::parse_bootstrap(r#"{type: "array", element: $3}"#, &logger).unwrap();
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
        let logger = crate::test_runner::NoOpLogger;
        let captured_vars = vec![
            "sequence_elements[0]".to_string(),
            "sequence_elements[1]".to_string(),
        ];

        // Test positional reference
        let ast = UnifiedReturnAST::PositionalRef { index: 1 };
        let code = ast.generate_code(&captured_vars, "", &logger).unwrap();
        assert_eq!(code, "sequence_elements[0].content.clone()");

        // Test string literal
        let ast = UnifiedReturnAST::StringLiteral {
            value: "test".to_string(),
        };
        let code = ast.generate_code(&captured_vars, "", &logger).unwrap();
        assert_eq!(code, "ParseContent::Terminal(r#\"test\"#)");
    }

    #[test]
    fn test_parse_extraction_operators() {
        let logger = crate::test_runner::NoOpLogger;
        // Test $2::2 (should extract second element, stored as index 1)
        let ast = UnifiedReturnAST::parse_bootstrap("$2::2", &logger).unwrap();
        match ast {
            UnifiedReturnAST::QuantifiedExtraction { base, target } => {
                assert!(matches!(
                    base.as_ref(),
                    UnifiedReturnAST::PositionalRef { index: 2 }
                ));
                assert_eq!(target, ExtractionTarget::Index(1)); // 2-1 = 1 (0-based)
            }
            _ => panic!("Expected QuantifiedExtraction"),
        }

        // Test $2::first
        let ast = UnifiedReturnAST::parse_bootstrap("$2::first", &logger).unwrap();
        match ast {
            UnifiedReturnAST::QuantifiedExtraction { base, target } => {
                assert!(matches!(
                    base.as_ref(),
                    UnifiedReturnAST::PositionalRef { index: 2 }
                ));
                assert_eq!(target, ExtractionTarget::First);
            }
            _ => panic!("Expected QuantifiedExtraction"),
        }

        // Test $2::last
        let ast = UnifiedReturnAST::parse_bootstrap("$2::last", &logger).unwrap();
        match ast {
            UnifiedReturnAST::QuantifiedExtraction { base, target } => {
                assert!(matches!(
                    base.as_ref(),
                    UnifiedReturnAST::PositionalRef { index: 2 }
                ));
                assert_eq!(target, ExtractionTarget::Last);
            }
            _ => panic!("Expected QuantifiedExtraction"),
        }

        // Test $2::1* (extraction with spread, should extract first element at index 0)
        let ast = UnifiedReturnAST::parse_bootstrap("$2::1*", &logger).unwrap();
        match ast {
            UnifiedReturnAST::Spread { base } => {
                match base.as_ref() {
                    UnifiedReturnAST::QuantifiedExtraction {
                        base: inner_base,
                        target,
                    } => {
                        assert!(matches!(
                            inner_base.as_ref(),
                            UnifiedReturnAST::PositionalRef { index: 2 }
                        ));
                        assert_eq!(*target, ExtractionTarget::Index(0)); // 1-1 = 0 (0-based)
                    }
                    _ => panic!("Expected QuantifiedExtraction inside Spread"),
                }
            }
            _ => panic!("Expected Spread"),
        }
    }

    #[test]
    fn bootstrap_leading_whitespace_before_arrow_is_normalized() {
        let logger = crate::test_runner::NoOpLogger;
        let ast = UnifiedReturnAST::parse_bootstrap("  -> $1", &logger).expect(
            "leading whitespace before '->' should still normalize and parse as arrow form",
        );
        assert_eq!(ast, UnifiedReturnAST::PositionalRef { index: 1 });
    }

    #[test]
    fn bootstrap_positional_spread_rejects_trailing_text_after_star() {
        let logger = crate::test_runner::NoOpLogger;
        let err = UnifiedReturnAST::parse_bootstrap("$1*trailing", &logger).expect_err(
            "bootstrap parser should reject trailing text after positional spread star",
        );
        assert_eq!(
            err,
            "Invalid positional reference modifier: '*trailing'"
        );
    }

    #[test]
    fn bootstrap_array_access_rejects_trailing_text_after_closing_bracket() {
        let logger = crate::test_runner::NoOpLogger;
        let err = UnifiedReturnAST::parse_bootstrap("$1[0]trailing", &logger).expect_err(
            "bootstrap parser should reject trailing text after array access closing bracket",
        );
        assert_eq!(err, "Invalid positional reference modifier: 'trailing'");
    }

    #[test]
    fn bootstrap_array_spread_is_not_applied_to_quoted_strings() {
        let logger = crate::test_runner::NoOpLogger;
        let ast = UnifiedReturnAST::parse_bootstrap("[\"$1*\"]", &logger)
            .expect("quoted string ending in '*' should remain string literal");
        match ast {
            UnifiedReturnAST::Array { elements } => {
                assert_eq!(elements.len(), 1);
                assert!(matches!(
                    elements[0],
                    UnifiedReturnAST::StringLiteral { ref value } if value == "$1*"
                ));
            }
            other => panic!("expected Array, got {:?}", other),
        }
    }

    #[test]
    fn bootstrap_array_spread_is_not_applied_to_single_quoted_strings() {
        let logger = crate::test_runner::NoOpLogger;
        let ast = UnifiedReturnAST::parse_bootstrap("['$1*']", &logger)
            .expect("single-quoted string ending in '*' should remain string literal");
        match ast {
            UnifiedReturnAST::Array { elements } => {
                assert_eq!(elements.len(), 1);
                assert!(matches!(
                    elements[0],
                    UnifiedReturnAST::StringLiteral { ref value } if value == "$1*"
                ));
            }
            other => panic!("expected Array, got {:?}", other),
        }
    }

    #[test]
    fn bootstrap_parses_identifier_literal() {
        let logger = crate::test_runner::NoOpLogger;
        let ast = UnifiedReturnAST::parse_bootstrap("my_ident_01", &logger)
            .expect("identifier literal should parse");
        assert_eq!(
            ast,
            UnifiedReturnAST::Identifier {
                name: "my_ident_01".to_string()
            }
        );
    }

    #[test]
    fn bootstrap_parses_single_quoted_object_keys() {
        let logger = crate::test_runner::NoOpLogger;
        let ast = UnifiedReturnAST::parse_bootstrap("{'k': $1}", &logger)
            .expect("single-quoted object key should parse");
        match ast {
            UnifiedReturnAST::Object { properties } => {
                assert_eq!(properties.len(), 1);
                assert!(matches!(
                    properties.get("k").map(|v| v.as_ref()),
                    Some(UnifiedReturnAST::PositionalRef { index: 1 })
                ));
            }
            other => panic!("expected Object, got {:?}", other),
        }
    }

    #[test]
    fn bootstrap_array_rejects_empty_segments_from_extra_commas() {
        let logger = crate::test_runner::NoOpLogger;
        let err = UnifiedReturnAST::parse_bootstrap("[, $1,, $2,]", &logger).expect_err(
            "bootstrap array parser should reject empty comma segments",
        );
        assert!(
            err.contains("Empty segment"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn bootstrap_object_rejects_empty_segments_from_extra_commas() {
        let logger = crate::test_runner::NoOpLogger;
        let err = UnifiedReturnAST::parse_bootstrap("{, a: $1,, b: $2,}", &logger).expect_err(
            "bootstrap object parser should reject empty comma segments",
        );
        assert!(
            err.contains("Empty segment"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn bootstrap_object_duplicate_keys_keep_last_value() {
        let logger = crate::test_runner::NoOpLogger;
        let ast = UnifiedReturnAST::parse_bootstrap("{a: $1, a: $2}", &logger)
            .expect("bootstrap object parser should accept duplicate keys");
        match ast {
            UnifiedReturnAST::Object { properties } => {
                assert_eq!(properties.len(), 1);
                assert!(matches!(
                    properties.get("a").map(|v| v.as_ref()),
                    Some(UnifiedReturnAST::PositionalRef { index: 2 })
                ));
            }
            other => panic!("expected Object, got {:?}", other),
        }
    }

    #[test]
    fn bootstrap_accepts_signed_positional_with_chained_accessor_and_nested_index_expr() {
        let logger = crate::test_runner::NoOpLogger;
        let ast = UnifiedReturnAST::parse_bootstrap("$+0.A.A000[($0::first)[$00]]", &logger)
            .expect("bootstrap parser should support signed positional chained accessor sample");

        match ast {
            UnifiedReturnAST::ArrayAccess { base, index } => {
                // base: $0.A.A000
                match base.as_ref() {
                    UnifiedReturnAST::PropertyAccess {
                        base: level1,
                        property,
                    } => {
                        assert_eq!(property, "A000");
                        match level1.as_ref() {
                            UnifiedReturnAST::PropertyAccess {
                                base: level0,
                                property,
                            } => {
                                assert_eq!(property, "A");
                                assert!(matches!(
                                    level0.as_ref(),
                                    UnifiedReturnAST::PositionalRef { index: 0 }
                                ));
                            }
                            other => panic!("expected first property access level, got {:?}", other),
                        }
                    }
                    other => panic!("expected property access base, got {:?}", other),
                }

                // index: ($0::first)[$00]
                match index.as_ref() {
                    UnifiedReturnAST::ArrayAccess {
                        base: nested_base,
                        index: nested_index,
                    } => {
                        assert!(matches!(
                            nested_base.as_ref(),
                            UnifiedReturnAST::QuantifiedExtraction {
                                base,
                                target: ExtractionTarget::First
                            } if matches!(base.as_ref(), UnifiedReturnAST::PositionalRef { index: 0 })
                        ));
                        assert!(matches!(
                            nested_index.as_ref(),
                            UnifiedReturnAST::PositionalRef { index: 0 }
                        ));
                    }
                    other => panic!("expected nested array access index, got {:?}", other),
                }
            }
            other => panic!("expected top-level ArrayAccess, got {:?}", other),
        }
    }

    #[test]
    fn bootstrap_accepts_leading_whitespace_on_signed_accessor_chain() {
        let logger = crate::test_runner::NoOpLogger;
        let ast = UnifiedReturnAST::parse_bootstrap("   $+0.A.A000[($0::first)[$00]]", &logger)
            .expect("leading whitespace should be tolerated before accessor chain");
        assert!(matches!(ast, UnifiedReturnAST::ArrayAccess { .. }));
    }
}
