use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

use crate::ast_pipeline::runtime_logger_box;
use crate::ast_pipeline::{ParseContent, ParseNode};
use crate::ebnf_generated_parser::EbnfParser;

/// Parse an EBNF file with the Rust-generated EBNF parser and emit an
/// in-memory JSON envelope compatible with the existing raw_ast pipeline input.
pub fn parse_ebnf_file_to_raw_ast_envelope(path: &str) -> Result<Value> {
    let input = fs::read_to_string(path)
        .with_context(|| format!("failed to read EBNF input file '{}'", path))?;
    let grammar_name = Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("unknown")
        .to_string();
    parse_ebnf_text_to_raw_ast_envelope(&input, &grammar_name, Some(path))
}

pub fn parse_ebnf_text_to_raw_ast_envelope(
    input: &str,
    grammar_name: &str,
    source_file: Option<&str>,
) -> Result<Value> {
    let mut parser = EbnfParser::new(input, runtime_logger_box("generated.ebnf_frontend"));
    let root = parser
        .parse_full_grammar_file()
        .map_err(|err| anyhow!("Rust EBNF parser failed: {}", err))?;

    let raw_ast = extract_raw_ast_rules(input, &root)?;
    let source_file_value = source_file.unwrap_or("<memory>");

    Ok(json!({
        "grammar_name": grammar_name,
        "metadata": {
            "format": "raw_ast",
            "source_format": "ebnf_rust_frontend",
            "generator": "ebnf.rs + rust_raw_ast_adapter",
            "generated_at": Utc::now().to_rfc3339(),
            "source_file": source_file_value,
            "description": "Raw AST envelope produced in-memory from Rust EBNF parser output",
            "next_step": "Apply Rust AST pipeline transform_from_raw_ast"
        },
        "raw_ast": raw_ast
    }))
}

fn extract_raw_ast_rules(input: &str, root: &ParseNode<'_>) -> Result<Vec<Value>> {
    let mut grammar_rules = Vec::new();
    collect_descendants_by_rule(root, "grammar_rule", &mut grammar_rules);
    grammar_rules.sort_by_key(|node| node.span.start);

    let mut raw_ast = Vec::with_capacity(grammar_rules.len());
    for grammar_rule in grammar_rules {
        raw_ast.push(convert_grammar_rule(input, grammar_rule)?);
    }

    Ok(raw_ast)
}

fn convert_grammar_rule(input: &str, grammar_rule: &ParseNode<'_>) -> Result<Value> {
    let rule_definition = find_first_descendant_by_rule(grammar_rule, "rule_definition")
        .ok_or_else(|| {
            anyhow!(
                "missing rule_definition in grammar_rule at {:?}",
                grammar_rule.span
            )
        })?;
    let rule_name_node =
        find_first_descendant_by_rule(rule_definition, "rule_name").ok_or_else(|| {
            anyhow!(
                "missing rule_name in rule_definition at {:?}",
                rule_definition.span
            )
        })?;
    let rule_expression_node = find_first_descendant_by_rule(rule_definition, "rule_expression")
        .ok_or_else(|| {
            anyhow!(
                "missing rule_expression in rule_definition at {:?}",
                rule_definition.span
            )
        })?;

    let rule_name = node_text(input, rule_name_node)?;
    if rule_name.trim().is_empty() {
        return Err(anyhow!("empty rule name at {:?}", rule_name_node.span));
    }

    let mut tokens = Vec::new();
    tokens.push(json!(["rule", rule_name.trim().to_string()]));

    if let Some(annotation_list) = find_first_descendant_by_rule(grammar_rule, "annotation_list") {
        let mut annotations = Vec::new();
        collect_descendants_by_rule(annotation_list, "semantic_annotation", &mut annotations);
        annotations.sort_by_key(|node| node.span.start);
        for annotation in annotations {
            if annotation.span.start == annotation.span.end {
                continue;
            }
            let raw_annotation = slice_span(input, annotation.span.clone())?.trim();
            if raw_annotation.is_empty() {
                continue;
            }
            if let Some((name, payload)) = parse_semantic_annotation_text(raw_annotation) {
                tokens.push(json!(["semantic_annotation", [name, payload]]));
            }
        }
    }

    let expression_text = slice_span(input, rule_expression_node.span.clone())?;
    tokens.extend(tokenize_rule_expression(expression_text)?);

    if let Some(return_node) = find_first_descendant_by_rule(rule_definition, "return_annotation") {
        if return_node.span.start < return_node.span.end {
            let raw_return = slice_span(input, return_node.span.clone())?;
            if let Some(return_body) = normalize_return_annotation_text(raw_return) {
                if !return_body.is_empty() {
                    let token_type = classify_return_annotation(&return_body);
                    tokens.push(json!([token_type, return_body]));
                }
            }
        }
    }

    Ok(Value::Array(tokens))
}

fn node_text(input: &str, node: &ParseNode<'_>) -> Result<String> {
    match &node.content {
        ParseContent::Terminal(s) => Ok((*s).to_string()),
        ParseContent::TransformedTerminal(s) => Ok(s.clone()),
        _ => Ok(slice_span(input, node.span.clone())?.to_string()),
    }
}

fn slice_span<'a>(input: &'a str, span: std::ops::Range<usize>) -> Result<&'a str> {
    input.get(span.clone()).ok_or_else(|| {
        anyhow!(
            "invalid parser span {:?} for input length {}",
            span,
            input.len()
        )
    })
}

fn collect_descendants_by_rule<'a>(
    node: &'a ParseNode<'a>,
    rule_name: &str,
    out: &mut Vec<&'a ParseNode<'a>>,
) {
    if node.rule_name == rule_name {
        out.push(node);
    }

    match &node.content {
        ParseContent::Sequence(children) | ParseContent::Quantified(children, _) => {
            for child in children {
                collect_descendants_by_rule(child, rule_name, out);
            }
        }
        ParseContent::Alternative(child) => collect_descendants_by_rule(child, rule_name, out),
        ParseContent::Terminal(_) | ParseContent::TransformedTerminal(_) => {}
    }
}

fn find_first_descendant_by_rule<'a>(
    node: &'a ParseNode<'a>,
    rule_name: &str,
) -> Option<&'a ParseNode<'a>> {
    if node.rule_name == rule_name {
        return Some(node);
    }

    match &node.content {
        ParseContent::Sequence(children) | ParseContent::Quantified(children, _) => {
            for child in children {
                if let Some(found) = find_first_descendant_by_rule(child, rule_name) {
                    return Some(found);
                }
            }
            None
        }
        ParseContent::Alternative(child) => find_first_descendant_by_rule(child, rule_name),
        ParseContent::Terminal(_) | ParseContent::TransformedTerminal(_) => None,
    }
}

fn tokenize_rule_expression(expression: &str) -> Result<Vec<Value>> {
    let mut tokens = Vec::new();
    let bytes = expression.as_bytes();
    let mut idx = 0usize;
    let mut optional_depth = 0usize;

    while idx < bytes.len() {
        let ch = bytes[idx] as char;

        if ch.is_whitespace() {
            idx += 1;
            continue;
        }

        if ch == '#' {
            while idx < bytes.len() && (bytes[idx] as char) != '\n' {
                idx += 1;
            }
            continue;
        }

        match ch {
            '(' => {
                tokens.push(json!(["group_open", "("]));
                idx += 1;
            }
            ')' => {
                tokens.push(json!(["group_close", ")"]));
                idx += 1;
            }
            '[' => {
                optional_depth = optional_depth.saturating_add(1);
                tokens.push(json!(["group_open", "("]));
                idx += 1;
            }
            ']' => {
                if optional_depth > 0 {
                    optional_depth -= 1;
                    tokens.push(json!(["group_close", ")"]));
                    tokens.push(json!(["operator", "?"]));
                }
                idx += 1;
            }
            '|' | '*' | '+' | '?' => {
                tokens.push(json!(["operator", ch.to_string()]));
                idx += 1;
            }
            '{' => {
                if let Some((quantifier, next_idx)) = parse_braced_quantifier(expression, idx) {
                    tokens.push(json!(["quantifier", quantifier]));
                    idx = next_idx;
                } else {
                    idx += 1;
                }
            }
            '@' => {
                if let Some((probability, next_idx)) = parse_probability_quantifier(expression, idx)
                {
                    tokens.push(json!(["probability", probability]));
                    idx = next_idx;
                } else {
                    idx += 1;
                }
            }
            '"' | '\'' => {
                if let Some((literal, next_idx)) = parse_quoted_literal(expression, idx) {
                    tokens.push(json!(["quoted_string", literal]));
                    idx = next_idx;
                } else {
                    return Err(anyhow!(
                        "unterminated quoted literal in expression '{}'",
                        expression
                    ));
                }
            }
            '/' => {
                if let Some((pattern, next_idx)) = parse_regex_literal(expression, idx) {
                    tokens.push(json!(["regex", pattern]));
                    idx = next_idx;
                } else {
                    return Err(anyhow!(
                        "unterminated regex literal in expression '{}'",
                        expression
                    ));
                }
            }
            'r' => {
                if let Some((literal, next_idx)) = parse_raw_string_literal(expression, idx) {
                    tokens.push(json!(["quoted_string", literal]));
                    idx = next_idx;
                } else if let Some((identifier, next_idx)) = parse_identifier(expression, idx) {
                    tokens.push(json!(["rule_reference", identifier]));
                    idx = next_idx;
                } else {
                    idx += 1;
                }
            }
            _ => {
                if let Some((identifier, next_idx)) = parse_identifier(expression, idx) {
                    tokens.push(json!(["rule_reference", identifier]));
                    idx = next_idx;
                } else {
                    idx += 1;
                }
            }
        }
    }

    Ok(tokens)
}

fn parse_quoted_literal(input: &str, start: usize) -> Option<(String, usize)> {
    let bytes = input.as_bytes();
    let quote = *bytes.get(start)? as char;
    if quote != '"' && quote != '\'' {
        return None;
    }

    let mut idx = start + 1;
    let mut escaped = false;
    while idx < bytes.len() {
        let ch = bytes[idx] as char;
        if escaped {
            escaped = false;
            idx += 1;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            idx += 1;
            continue;
        }
        if ch == quote {
            let literal = input.get(start + 1..idx)?.to_string();
            return Some((literal, idx + 1));
        }
        idx += 1;
    }
    None
}

fn parse_raw_string_literal(input: &str, start: usize) -> Option<(String, usize)> {
    let bytes = input.as_bytes();
    if bytes.get(start).copied() != Some(b'r') || bytes.get(start + 1).copied() != Some(b'"') {
        return None;
    }
    let mut idx = start + 2;
    while idx < bytes.len() {
        if bytes[idx] == b'"' {
            let literal = input.get(start + 2..idx)?.to_string();
            return Some((literal, idx + 1));
        }
        idx += 1;
    }
    None
}

fn parse_regex_literal(input: &str, start: usize) -> Option<(String, usize)> {
    let bytes = input.as_bytes();
    if bytes.get(start).copied() != Some(b'/') {
        return None;
    }

    let mut idx = start + 1;
    let mut escaped = false;
    let mut in_class = false;
    while idx < bytes.len() {
        let ch = bytes[idx] as char;
        if escaped {
            escaped = false;
            idx += 1;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            idx += 1;
            continue;
        }
        if ch == '[' {
            in_class = true;
            idx += 1;
            continue;
        }
        if ch == ']' && in_class {
            in_class = false;
            idx += 1;
            continue;
        }
        if ch == '/' && !in_class {
            let pattern = input.get(start + 1..idx)?.to_string();
            return Some((pattern, idx + 1));
        }
        idx += 1;
    }
    None
}

fn parse_identifier(input: &str, start: usize) -> Option<(String, usize)> {
    let bytes = input.as_bytes();
    let first = *bytes.get(start)? as char;
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return None;
    }

    let mut idx = start + 1;
    while idx < bytes.len() {
        let ch = bytes[idx] as char;
        if ch == '_' || ch.is_ascii_alphanumeric() {
            idx += 1;
        } else {
            break;
        }
    }

    Some((input.get(start..idx)?.to_string(), idx))
}

fn parse_braced_quantifier(input: &str, start: usize) -> Option<(String, usize)> {
    let bytes = input.as_bytes();
    if bytes.get(start).copied() != Some(b'{') {
        return None;
    }

    let mut idx = start + 1;
    while idx < bytes.len() && bytes[idx] != b'}' {
        idx += 1;
    }
    if idx >= bytes.len() || bytes[idx] != b'}' {
        return None;
    }

    let content = input.get(start + 1..idx)?.trim().to_string();
    if content.is_empty() {
        return None;
    }
    Some((content, idx + 1))
}

fn parse_probability_quantifier(input: &str, start: usize) -> Option<(String, usize)> {
    let bytes = input.as_bytes();
    if bytes.get(start).copied() != Some(b'@') {
        return None;
    }

    let mut idx = start + 1;
    let digits_start = idx;
    while idx < bytes.len() && (bytes[idx] as char).is_ascii_digit() {
        idx += 1;
    }
    if idx == digits_start {
        return None;
    }
    if idx < bytes.len() && bytes[idx] == b'%' {
        idx += 1;
    }

    Some((
        input
            .get(digits_start..idx)?
            .trim_end_matches('%')
            .to_string(),
        idx,
    ))
}

fn parse_semantic_annotation_text(annotation: &str) -> Option<(String, String)> {
    let trimmed = annotation.trim();
    let content = trimmed.strip_prefix('@')?.trim();
    let separator_idx = find_top_level_colon(content)?;
    let name = content.get(..separator_idx)?.trim();
    let payload = content.get(separator_idx + 1..)?.trim();
    if name.is_empty() {
        return None;
    }
    Some((name.to_string(), payload.to_string()))
}

fn find_top_level_colon(content: &str) -> Option<usize> {
    let mut in_quote: Option<char> = None;
    let mut escaped = false;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;

    for (idx, ch) in content.char_indices() {
        if let Some(quote) = in_quote {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == quote {
                in_quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => in_quote = Some(ch),
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            ':' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                return Some(idx);
            }
            _ => {}
        }
    }

    None
}

fn normalize_return_annotation_text(raw_return: &str) -> Option<String> {
    let trimmed = raw_return.trim();
    let body = trimmed.strip_prefix("->").map(str::trim).or_else(|| {
        trimmed
            .find("->")
            .and_then(|idx| trimmed.get(idx + 2..).map(str::trim))
    })?;
    if body.is_empty() {
        None
    } else {
        Some(body.to_string())
    }
}

fn classify_return_annotation(body: &str) -> &'static str {
    let trimmed = body.trim();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        "return_object"
    } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
        "return_array"
    } else {
        "return_scalar"
    }
}

#[cfg(test)]
mod tests {
    use super::{
        classify_return_annotation, parse_ebnf_text_to_raw_ast_envelope,
        parse_semantic_annotation_text, tokenize_rule_expression,
    };

    #[test]
    fn tokenizes_basic_group_and_quantifier_expression() {
        let tokens =
            tokenize_rule_expression("(a | \"b\")*").expect("expression tokenization should work");
        let rendered = serde_json::to_string(&tokens).expect("json");
        assert!(rendered.contains("[\"group_open\",\"(\"]"));
        assert!(rendered.contains("[\"rule_reference\",\"a\"]"));
        assert!(rendered.contains("[\"operator\",\"|\"]"));
        assert!(rendered.contains("[\"quoted_string\",\"b\"]"));
        assert!(rendered.contains("[\"group_close\",\")\"]"));
        assert!(rendered.contains("[\"operator\",\"*\"]"));
    }

    #[test]
    fn tokenizes_regex_and_bounded_quantifier() {
        let tokens = tokenize_rule_expression("/[A-Z]+/{2,8}")
            .expect("regex and quantifier tokenization should work");
        let rendered = serde_json::to_string(&tokens).expect("json");
        assert!(rendered.contains("[\"regex\",\"[A-Z]+\"]"));
        assert!(rendered.contains("[\"quantifier\",\"2,8\"]"));
    }

    #[test]
    fn tokenizes_optional_brackets_as_group_plus_optional_operator() {
        let tokens =
            tokenize_rule_expression("[foo]").expect("optional bracket tokenization should work");
        let rendered = serde_json::to_string(&tokens).expect("json");
        assert!(rendered.contains("[\"group_open\",\"(\"]"));
        assert!(rendered.contains("[\"rule_reference\",\"foo\"]"));
        assert!(rendered.contains("[\"group_close\",\")\"]"));
        assert!(rendered.contains("[\"operator\",\"?\"]"));
    }

    #[test]
    fn parses_semantic_annotation_name_and_payload() {
        let parsed = parse_semantic_annotation_text("@stop_at_rule_boundary: true");
        assert_eq!(
            parsed,
            Some(("stop_at_rule_boundary".to_string(), "true".to_string()))
        );
    }

    #[test]
    fn parses_semantic_annotation_with_nested_colons() {
        let parsed = parse_semantic_annotation_text(
            "@transform: map(field: \"a:b\", nested: [\"x:y\"], limit: 3)",
        );
        assert_eq!(
            parsed,
            Some((
                "transform".to_string(),
                "map(field: \"a:b\", nested: [\"x:y\"], limit: 3)".to_string(),
            ))
        );
    }

    #[test]
    fn classifies_return_annotation_shapes() {
        assert_eq!(classify_return_annotation("{a: 1}"), "return_object");
        assert_eq!(classify_return_annotation("[a, b]"), "return_array");
        assert_eq!(classify_return_annotation("$1"), "return_scalar");
    }

    #[test]
    fn parses_ebnf_text_into_raw_ast_envelope_with_annotations() {
        let input = r#"
@stop_at_rule_boundary: true
entry := "a" -> {type: "node"}
"#;
        let envelope = parse_ebnf_text_to_raw_ast_envelope(input, "mini", None)
            .expect("frontend parse should succeed");
        let grammar_name = envelope
            .get("grammar_name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("<missing>");
        assert_eq!(grammar_name, "mini");

        let raw_ast = envelope
            .get("raw_ast")
            .and_then(serde_json::Value::as_array)
            .expect("raw_ast array should exist");
        assert_eq!(raw_ast.len(), 1, "expected one rule in raw_ast");
        let first_rule =
            serde_json::to_string(raw_ast[0].as_array().expect("rule array")).expect("rule json");
        assert!(
            first_rule.contains("[\"rule\",\"entry\"]"),
            "expected rule token in raw_ast, got: {}",
            first_rule
        );
        assert!(
            first_rule.contains("[\"semantic_annotation\",[\"stop_at_rule_boundary\",\"true\"]]"),
            "expected semantic annotation token in raw_ast, got: {}",
            first_rule
        );
        assert!(
            first_rule.contains("[\"return_object\",\"{type: \\\"node\\\"}\"]"),
            "expected return annotation token in raw_ast, got: {}",
            first_rule
        );
    }
}
