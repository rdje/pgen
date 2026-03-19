use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use serde_json::{Value, json};
use std::fs;
use std::path::Path;

use crate::ast_pipeline::runtime_logger_box;
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
    let scanned_rules = scan_top_level_rules(input)?;
    let has_multiline_annotations = scanned_rules
        .iter()
        .flat_map(|rule| rule.annotations.iter())
        .any(|annotation| annotation.contains('\n'));
    let mut raw_ast = Vec::with_capacity(scanned_rules.len());
    for rule in &scanned_rules {
        raw_ast.push(convert_scanned_rule(rule)?);
    }

    let mut parser = EbnfParser::new(input, runtime_logger_box("generated.ebnf_frontend"));
    if let Err(err) = parser.parse_full_grammar_file() {
        if !has_multiline_annotations {
            return Err(anyhow!("Rust EBNF parser failed: {}", err));
        }
    }
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

#[derive(Debug)]
struct ScannedRule {
    name: String,
    annotations: Vec<String>,
    expression: String,
    return_annotation: Option<String>,
}

fn scan_top_level_rules(input: &str) -> Result<Vec<ScannedRule>> {
    let lines: Vec<&str> = input.lines().collect();
    let mut rules = Vec::new();
    let mut pending_annotations = Vec::new();
    let mut idx = 0usize;

    while idx < lines.len() {
        let line = lines[idx];
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            idx += 1;
            continue;
        }

        if leading_whitespace_len(line) != 0 {
            idx += 1;
            continue;
        }

        if is_include_directive(trimmed) {
            idx += 1;
            continue;
        }

        if trimmed.starts_with('@') {
            let (annotation, next_idx) = collect_annotation_block(&lines, idx)?;
            pending_annotations.push(annotation);
            idx = next_idx;
            continue;
        }

        if let Some((rule_name, first_body)) = parse_rule_header(trimmed) {
            let (body, next_idx) = collect_rule_body(&lines, idx, first_body);
            let (expression, return_annotation) =
                split_rule_expression_and_return_annotation(&body);
            rules.push(ScannedRule {
                name: rule_name,
                annotations: std::mem::take(&mut pending_annotations),
                expression,
                return_annotation,
            });
            idx = next_idx;
            continue;
        }

        idx += 1;
    }

    Ok(rules)
}

fn convert_scanned_rule(rule: &ScannedRule) -> Result<Value> {
    let mut tokens = Vec::new();
    tokens.push(json!(["rule", rule.name.clone()]));

    for raw_annotation in &rule.annotations {
        if raw_annotation.trim().is_empty() {
            continue;
        }
        if let Some((name, payload)) = parse_semantic_annotation_text(raw_annotation) {
            tokens.push(json!(["semantic_annotation", [name, payload]]));
        }
    }

    tokens.extend(tokenize_rule_expression(&rule.expression)?);

    if let Some(return_body) = &rule.return_annotation {
        if !return_body.is_empty() {
            let token_type = classify_return_annotation(return_body);
            tokens.push(json!([token_type, return_body]));
        }
    }

    Ok(Value::Array(tokens))
}

fn leading_whitespace_len(line: &str) -> usize {
    line.chars().take_while(|ch| ch.is_whitespace()).count()
}

fn is_include_directive(line: &str) -> bool {
    matches!(
        line,
        s if s.starts_with("include(")
            || s.starts_with("include_file(")
            || s.starts_with("include_dir(")
            || s.starts_with("file(")
            || s.starts_with("dir(")
    )
}

fn parse_rule_header(line: &str) -> Option<(String, String)> {
    let mut name_end = 0usize;
    for (idx, ch) in line.char_indices() {
        if idx == 0 {
            if !(ch == '_' || ch.is_ascii_alphabetic()) {
                return None;
            }
            name_end = ch.len_utf8();
            continue;
        }

        if ch == '_' || ch.is_ascii_alphanumeric() {
            name_end = idx + ch.len_utf8();
        } else {
            break;
        }
    }

    if name_end == 0 {
        return None;
    }

    let rule_name = line.get(..name_end)?.to_string();
    let rest = line.get(name_end..)?.trim_start();
    for operator in ["::=", ":=", ":-", "="] {
        if let Some(body) = rest.strip_prefix(operator) {
            return Some((rule_name, body.trim_start().to_string()));
        }
    }
    None
}

fn collect_annotation_block(lines: &[&str], start_idx: usize) -> Result<(String, usize)> {
    let mut annotation = lines
        .get(start_idx)
        .copied()
        .unwrap_or_default()
        .trim_end()
        .to_string();
    let mut idx = start_idx + 1;

    while annotation_requires_continuation(&annotation)? && idx < lines.len() {
        annotation.push('\n');
        annotation.push_str(lines[idx].trim_end());
        idx += 1;
    }

    Ok((annotation, idx))
}

fn annotation_requires_continuation(annotation: &str) -> Result<bool> {
    let Some((_name, payload)) = parse_semantic_annotation_text(annotation) else {
        return Ok(false);
    };
    let tracker = DelimiterTracker::scan_annotations(&payload)?;
    Ok(tracker.is_open())
}

fn collect_rule_body(lines: &[&str], start_idx: usize, first_body: String) -> (String, usize) {
    let mut body_lines = vec![first_body.trim_end().to_string()];
    let mut idx = start_idx + 1;

    while idx < lines.len() {
        let line = lines[idx];
        let trimmed = line.trim();
        let is_top_level = leading_whitespace_len(line) == 0;
        if is_top_level
            && (trimmed.is_empty()
                || trimmed.starts_with('#')
                || trimmed.starts_with('@')
                || is_include_directive(trimmed)
                || parse_rule_header(trimmed).is_some())
        {
            break;
        }

        if trimmed.is_empty() && is_top_level {
            break;
        }

        body_lines.push(line.trim_end().to_string());
        idx += 1;
    }

    (body_lines.join("\n"), idx)
}

fn split_rule_expression_and_return_annotation(body: &str) -> (String, Option<String>) {
    let Some(idx) = find_top_level_return_annotation(body) else {
        return (body.trim().to_string(), None);
    };

    let expression = body
        .get(..idx)
        .map(str::trim)
        .unwrap_or_default()
        .to_string();
    let return_annotation = body
        .get(idx + 2..)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    (expression, return_annotation)
}

fn find_top_level_return_annotation(body: &str) -> Option<usize> {
    let mut tracker = DelimiterTracker::default();
    let mut chars = body.char_indices().peekable();

    while let Some((idx, ch)) = chars.next() {
        if tracker.consume(
            ch,
            chars.peek().map(|(_, next)| *next),
            ScanMode::RuleExpression,
        ) {
            continue;
        }
        if ch == '-' && matches!(chars.peek(), Some((_, '>'))) && tracker.is_top_level() {
            return Some(idx);
        }
    }

    None
}

#[derive(Clone, Copy, Debug)]
enum ScanMode {
    Annotations,
    RuleExpression,
}

#[derive(Default, Debug)]
struct DelimiterTracker {
    in_quote: Option<char>,
    escaped: bool,
    in_regex: bool,
    regex_in_class: bool,
    line_comment: bool,
    paren_depth: usize,
    bracket_depth: usize,
    brace_depth: usize,
}

impl DelimiterTracker {
    fn scan_annotations(input: &str) -> Result<Self> {
        let mut tracker = Self::default();
        let mut chars = input.char_indices().peekable();
        while let Some((_idx, ch)) = chars.next() {
            tracker.consume(
                ch,
                chars.peek().map(|(_, next)| *next),
                ScanMode::Annotations,
            );
        }
        if tracker.in_quote.is_some() {
            return Err(anyhow!("unterminated quote in semantic annotation payload"));
        }
        Ok(tracker)
    }

    fn consume(&mut self, ch: char, next: Option<char>, mode: ScanMode) -> bool {
        if self.line_comment {
            if ch == '\n' {
                self.line_comment = false;
            }
            return true;
        }

        if let Some(quote) = self.in_quote {
            if self.escaped {
                self.escaped = false;
                return true;
            }
            if ch == '\\' {
                self.escaped = true;
                return true;
            }
            if ch == quote {
                self.in_quote = None;
            }
            return true;
        }

        if self.in_regex {
            if self.escaped {
                self.escaped = false;
                return true;
            }
            match ch {
                '\\' => {
                    self.escaped = true;
                    return true;
                }
                '[' => {
                    self.regex_in_class = true;
                    return true;
                }
                ']' if self.regex_in_class => {
                    self.regex_in_class = false;
                    return true;
                }
                '/' if !self.regex_in_class => {
                    self.in_regex = false;
                    return true;
                }
                _ => return true,
            }
        }

        match ch {
            '"' | '\'' => {
                self.in_quote = Some(ch);
                true
            }
            '#' if matches!(mode, ScanMode::RuleExpression) => {
                self.line_comment = true;
                true
            }
            '/' if matches!(mode, ScanMode::RuleExpression)
                && next.is_some()
                && self.is_top_level() =>
            {
                self.in_regex = true;
                true
            }
            '(' => {
                self.paren_depth += 1;
                false
            }
            ')' => {
                self.paren_depth = self.paren_depth.saturating_sub(1);
                false
            }
            '[' => {
                self.bracket_depth += 1;
                false
            }
            ']' => {
                self.bracket_depth = self.bracket_depth.saturating_sub(1);
                false
            }
            '{' => {
                self.brace_depth += 1;
                false
            }
            '}' => {
                self.brace_depth = self.brace_depth.saturating_sub(1);
                false
            }
            _ => false,
        }
    }

    fn is_top_level(&self) -> bool {
        self.paren_depth == 0 && self.bracket_depth == 0 && self.brace_depth == 0
    }

    fn is_open(&self) -> bool {
        self.in_quote.is_some()
            || self.in_regex
            || self.paren_depth > 0
            || self.bracket_depth > 0
            || self.brace_depth > 0
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
            '|' | '*' | '+' | '?' | '!' | '&' => {
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
        parse_ebnf_file_to_raw_ast_envelope,
        parse_semantic_annotation_text, tokenize_rule_expression,
    };
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    fn collect_rule_names(envelope: &serde_json::Value) -> BTreeSet<String> {
        envelope
            .get("raw_ast")
            .and_then(serde_json::Value::as_array)
            .expect("raw_ast array should exist")
            .iter()
            .filter_map(|rule| {
                let head = rule.as_array()?.first()?.as_array()?;
                let token_type = head.first()?.as_str()?;
                let rule_name = head.get(1)?.as_str()?;
                (token_type == "rule").then(|| rule_name.to_string())
            })
            .collect()
    }

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
    fn tokenizes_lookahead_operators() {
        let tokens =
            tokenize_rule_expression("!keyword &identifier").expect("lookahead tokenization");
        let rendered = serde_json::to_string(&tokens).expect("json");
        assert!(rendered.contains("[\"operator\",\"!\"]"));
        assert!(rendered.contains("[\"operator\",\"&\"]"));
        assert!(rendered.contains("[\"rule_reference\",\"keyword\"]"));
        assert!(rendered.contains("[\"rule_reference\",\"identifier\"]"));
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

    #[test]
    fn preserves_multiline_semantic_annotation_blocks() {
        let input = r#"
@dispatch: {
    "range": make_range($1),
    "literal": make_literal($1)
}
entry = alpha
      | beta
"#;
        let envelope = parse_ebnf_text_to_raw_ast_envelope(input, "multi", None)
            .expect("frontend parse should succeed");
        let raw_ast = envelope
            .get("raw_ast")
            .and_then(serde_json::Value::as_array)
            .expect("raw_ast array should exist");
        assert_eq!(raw_ast.len(), 1, "expected one rule in raw_ast");
        let first_rule =
            serde_json::to_string(raw_ast[0].as_array().expect("rule array")).expect("rule json");
        assert!(
            first_rule.contains("[\"semantic_annotation\",[\"dispatch\",\"{\\n    \\\"range\\\": make_range($1),\\n    \\\"literal\\\": make_literal($1)\\n}\"]]"),
            "expected multiline dispatch annotation token in raw_ast, got: {}",
            first_rule
        );
        assert!(
            first_rule.contains("[\"rule_reference\",\"alpha\"]"),
            "expected first alternative in raw_ast, got: {}",
            first_rule
        );
        assert!(
            first_rule.contains("[\"rule_reference\",\"beta\"]"),
            "expected second alternative in raw_ast, got: {}",
            first_rule
        );
    }

    #[test]
    fn regex_frontend_keeps_trailing_helper_rules_present_in_source_grammar() {
        let grammar_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../grammars/regex.ebnf");
        let envelope =
            parse_ebnf_file_to_raw_ast_envelope(grammar_path.to_str().expect("utf8 path"))
                .expect("frontend parse should succeed");
        let rule_names = collect_rule_names(&envelope);

        for expected in [
            "code_not_squote_or_backslash",
            "code_safe_special",
            "letter",
            "digit",
            "hex_digit",
            "octal_digit",
            "whitespace",
            "any_char",
            "special_char",
        ] {
            assert!(
                rule_names.contains(expected),
                "expected regex helper rule '{}' to remain present in Rust raw_ast export",
                expected
            );
        }
    }
}
