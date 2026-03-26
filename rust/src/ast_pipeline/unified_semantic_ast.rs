//! Unified Semantic Annotation AST
//!
//! This module provides a single, consistent AST representation for semantic annotations
//! that is used throughout the pipeline:
//! 1. Parsed from text by the external parser or bootstrap parser
//! 2. Pretty-printed for debugging
//! 3. Used directly by the code generator to emit Rust code
//!
//! This eliminates the need for multiple parallel AST representations and parsers.

use super::{Logger, ParseContent, ParseNode};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UnifiedSemanticValue {
    String(String),
    Number(String),
    Boolean(bool),
    Null,
    Identifier(String),
    RuleReference(String),
    Array(Vec<UnifiedSemanticValue>),
    Object(Vec<UnifiedSemanticProperty>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnifiedSemanticProperty {
    pub key: String,
    pub value: UnifiedSemanticValue,
}

/// The unified AST representation of a semantic annotation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UnifiedSemanticAST {
    /// Transform expression: str::parse::<f64>().unwrap_or(0.0)
    /// These map to transformation functions applied to matched terminals
    TransformExpr {
        expression: String, // The raw transform expression for now
    },

    /// Structured payload retained for future directive/runtime use while also
    /// preserving the canonical textual payload for existing directive consumers.
    Structured {
        canonical: String,
        value: UnifiedSemanticValue,
    },

    /// Raw annotation that couldn't be parsed
    Raw { content: String },
}

impl UnifiedSemanticAST {
    pub fn payload_text(&self) -> &str {
        match self {
            UnifiedSemanticAST::TransformExpr { expression } => expression.as_str(),
            UnifiedSemanticAST::Structured { canonical, .. } => canonical.as_str(),
            UnifiedSemanticAST::Raw { content } => content.as_str(),
        }
    }

    pub fn structured_value(&self) -> Option<&UnifiedSemanticValue> {
        match self {
            UnifiedSemanticAST::Structured { value, .. } => Some(value),
            _ => None,
        }
    }

    pub fn from_named_payload(annotation_name: &str, payload: &str) -> Self {
        let trimmed = payload.trim().to_string();
        if annotation_name.trim().eq_ignore_ascii_case("transform") {
            UnifiedSemanticAST::TransformExpr {
                expression: trimmed,
            }
        } else if let Some(value) = Self::parse_structured_payload(&trimmed) {
            UnifiedSemanticAST::Structured {
                canonical: trimmed,
                value,
            }
        } else {
            UnifiedSemanticAST::Raw { content: trimmed }
        }
    }

    /// Build semantic AST from a generated-parser parse tree.
    /// This is used by non-bootstrap generated parser paths.
    pub fn parse_generated_semantic_annotation<'input>(
        input: &'input str,
        parse_tree: &ParseNode<'input>,
        logger: &dyn Logger,
    ) -> Result<Self, String> {
        let (_name, ast) =
            Self::parse_generated_semantic_annotation_entry(input, parse_tree, logger)?;
        Ok(ast)
    }

    /// Build semantic AST entry (name + AST) from a generated-parser parse tree.
    /// This is used when caller needs the parsed directive name in addition to the AST.
    pub fn parse_generated_semantic_annotation_entry<'input>(
        input: &'input str,
        parse_tree: &ParseNode<'input>,
        logger: &dyn Logger,
    ) -> Result<(String, Self), String> {
        let root = if parse_tree.rule_name == "semantic_annotation" {
            parse_tree
        } else if let Some(node) = Self::find_first_rule_node(parse_tree, "semantic_annotation") {
            node
        } else {
            parse_tree
        };

        let name_node = Self::find_first_rule_node(root, "annotation_name")
            .ok_or_else(|| "Generated semantic parse tree missing annotation_name".to_string())?;
        let value_node = Self::find_first_rule_node(root, "annotation_value")
            .ok_or_else(|| "Generated semantic parse tree missing annotation_value".to_string())?;

        let name_text = Self::slice_span(input, &name_node.span)
            .ok_or_else(|| {
                format!(
                    "annotation_name span {}..{} out of bounds for input len {}",
                    name_node.span.start,
                    name_node.span.end,
                    input.len()
                )
            })?
            .trim()
            .to_ascii_lowercase();
        let value_text = Self::slice_span(input, &value_node.span)
            .ok_or_else(|| {
                format!(
                    "annotation_value span {}..{} out of bounds for input len {}",
                    value_node.span.start,
                    value_node.span.end,
                    input.len()
                )
            })?
            .trim()
            .to_string();

        if logger.is_enabled() {
            logger.log_debug(
                "unified_semantic_ast.rs",
                line!(),
                &format!(
                    "Generated semantic parse-tree conversion: name='{}' value='{}'",
                    name_text, value_text
                ),
            );
        }

        let ast = Self::from_named_payload(&name_text, &value_text);

        Ok((name_text, ast))
    }

    /// Parse a semantic annotation in bootstrap mode (minimal support)
    pub fn parse_bootstrap(annotation_value: &str, logger: &dyn Logger) -> Result<Self, String> {
        let trimmed = annotation_value.trim();

        if logger.is_enabled() {
            logger.log_info(
                "unified_semantic_ast.rs",
                line!(),
                &format!(
                    "Parsing semantic annotation in bootstrap mode: '{}'",
                    trimmed
                ),
            );
        }

        // For now, we only support transform expressions as raw strings
        // TODO: Add proper AST parsing for function calls, type parameters, etc.
        if trimmed.contains("::parse::<") && trimmed.contains(">().unwrap_or(") {
            if logger.is_enabled() {
                logger.log_success(
                    "unified_semantic_ast.rs",
                    line!(),
                    "Recognized as transform expression",
                );
            }
            Ok(UnifiedSemanticAST::TransformExpr {
                expression: trimmed.to_string(),
            })
        } else if let Some(value) = Self::parse_structured_payload(trimmed) {
            if logger.is_enabled() {
                logger.log_success(
                    "unified_semantic_ast.rs",
                    line!(),
                    "Recognized as structured semantic payload",
                );
            }
            Ok(UnifiedSemanticAST::Structured {
                canonical: trimmed.to_string(),
                value,
            })
        } else {
            if logger.is_enabled() {
                logger.log_info(
                    "unified_semantic_ast.rs",
                    line!(),
                    "Unrecognized semantic annotation, storing as raw",
                );
            }
            Ok(UnifiedSemanticAST::Raw {
                content: trimmed.to_string(),
            })
        }
    }

    fn parse_structured_payload(input: &str) -> Option<UnifiedSemanticValue> {
        let mut parser = StructuredSemanticValueParser::new(input);
        parser.parse_root()
    }

    fn find_first_rule_node<'a, 'input>(
        node: &'a ParseNode<'input>,
        rule_name: &str,
    ) -> Option<&'a ParseNode<'input>> {
        if node.rule_name == rule_name {
            return Some(node);
        }

        match &node.content {
            ParseContent::Sequence(children) | ParseContent::Quantified(children, _) => {
                for child in children {
                    if let Some(found) = Self::find_first_rule_node(child, rule_name) {
                        return Some(found);
                    }
                }
            }
            ParseContent::Alternative(child) => {
                if let Some(found) = Self::find_first_rule_node(child, rule_name) {
                    return Some(found);
                }
            }
            ParseContent::Terminal(_) | ParseContent::TransformedTerminal(_) => {}
        }

        None
    }

    fn slice_span<'input>(
        input: &'input str,
        span: &std::ops::Range<usize>,
    ) -> Option<&'input str> {
        if span.start <= span.end && span.end <= input.len() {
            input.get(span.start..span.end)
        } else {
            None
        }
    }

    /// Pretty print the AST for debugging
    pub fn pretty_print(&self) -> String {
        match self {
            UnifiedSemanticAST::TransformExpr { expression } => {
                format!("TransformExpr({})", expression)
            }
            UnifiedSemanticAST::Structured { canonical, value } => {
                format!("Structured({}, {:?})", canonical, value)
            }
            UnifiedSemanticAST::Raw { content } => {
                format!("Raw({})", content)
            }
        }
    }
}

struct StructuredSemanticValueParser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> StructuredSemanticValueParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    fn parse_root(&mut self) -> Option<UnifiedSemanticValue> {
        self.skip_ws();
        let value = self.parse_value()?;
        self.skip_ws();
        if self.position == self.input.len() {
            Some(value)
        } else {
            None
        }
    }

    fn parse_value(&mut self) -> Option<UnifiedSemanticValue> {
        self.skip_ws();
        let ch = self.peek_char()?;
        match ch {
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            '"' | '\'' => self.parse_string().map(UnifiedSemanticValue::String),
            '$' => self
                .parse_rule_reference()
                .map(UnifiedSemanticValue::RuleReference),
            '+' | '-' | '0'..='9' => self.parse_number().map(UnifiedSemanticValue::Number),
            _ => {
                let ident = self.parse_identifier_like()?;
                let lowered = ident.to_ascii_lowercase();
                if matches!(
                    lowered.as_str(),
                    "true"
                        | "false"
                        | "yes"
                        | "no"
                        | "on"
                        | "off"
                        | "enabled"
                        | "disabled"
                        | "active"
                        | "inactive"
                ) {
                    let value = matches!(
                        lowered.as_str(),
                        "true" | "yes" | "on" | "enabled" | "active"
                    );
                    Some(UnifiedSemanticValue::Boolean(value))
                } else if matches!(
                    lowered.as_str(),
                    "null" | "nil" | "none" | "undefined" | "void"
                ) {
                    Some(UnifiedSemanticValue::Null)
                } else {
                    Some(UnifiedSemanticValue::Identifier(ident))
                }
            }
        }
    }

    fn parse_array(&mut self) -> Option<UnifiedSemanticValue> {
        self.expect_char('[')?;
        let mut elements = Vec::new();
        loop {
            self.skip_ws();
            if self.consume_char_if(']') {
                break;
            }
            let value = self.parse_value()?;
            elements.push(value);
            self.skip_ws();
            if self.consume_char_if(']') {
                break;
            }
            self.expect_char(',')?;
        }
        Some(UnifiedSemanticValue::Array(elements))
    }

    fn parse_object(&mut self) -> Option<UnifiedSemanticValue> {
        self.expect_char('{')?;
        let mut properties = Vec::new();
        loop {
            self.skip_ws();
            if self.consume_char_if('}') {
                break;
            }
            let key = self.parse_object_key()?;
            self.skip_ws();
            self.expect_char(':')?;
            let value = self.parse_value()?;
            properties.push(UnifiedSemanticProperty { key, value });
            self.skip_ws();
            if self.consume_char_if('}') {
                break;
            }
            self.expect_char(',')?;
        }
        Some(UnifiedSemanticValue::Object(properties))
    }

    fn parse_object_key(&mut self) -> Option<String> {
        self.skip_ws();
        match self.peek_char()? {
            '"' | '\'' => self.parse_string(),
            _ => self.parse_identifier_like(),
        }
    }

    fn parse_string(&mut self) -> Option<String> {
        let quote = self.next_char()?;
        let mut output = String::new();
        loop {
            let ch = self.next_char()?;
            if ch == quote {
                break;
            }
            if ch == '\\' {
                let escaped = self.next_char()?;
                let mapped = match escaped {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\\' => '\\',
                    '\'' => '\'',
                    '"' => '"',
                    other => other,
                };
                output.push(mapped);
            } else {
                output.push(ch);
            }
        }
        Some(output)
    }

    fn parse_rule_reference(&mut self) -> Option<String> {
        self.expect_char('$')?;
        let start = self.position;
        let first = self.peek_char()?;
        if first.is_ascii_digit() {
            self.consume_while(|ch| ch.is_ascii_digit());
        } else if first.is_ascii_alphabetic() || first == '_' {
            self.consume_while(|ch| ch.is_ascii_alphanumeric() || ch == '_');
        } else {
            return None;
        }
        Some(self.input[start..self.position].to_string())
    }

    fn parse_number(&mut self) -> Option<String> {
        let start = self.position;
        self.consume_char_if('+');
        self.consume_char_if('-');
        let int_start = self.position;
        self.consume_while(|ch| ch.is_ascii_digit());
        let mut consumed_any = self.position > int_start;
        if self.consume_char_if('.') {
            let frac_start = self.position;
            self.consume_while(|ch| ch.is_ascii_digit());
            consumed_any |= self.position > frac_start;
        }
        if matches!(self.peek_char(), Some('e' | 'E')) {
            let checkpoint = self.position;
            self.position += 1;
            self.consume_char_if('+');
            self.consume_char_if('-');
            let exp_start = self.position;
            self.consume_while(|ch| ch.is_ascii_digit());
            if self.position == exp_start {
                self.position = checkpoint;
            } else {
                consumed_any = true;
            }
        }
        if !consumed_any {
            return None;
        }
        Some(self.input[start..self.position].to_string())
    }

    fn parse_identifier_like(&mut self) -> Option<String> {
        let start = self.position;
        let first = self.peek_char()?;
        if !(first.is_ascii_alphabetic() || first == '_') {
            return None;
        }
        self.position += first.len_utf8();
        self.consume_while(|ch| ch.is_ascii_alphanumeric() || ch == '_');
        while self.consume_char_if('.') {
            let first = self.peek_char()?;
            if !(first.is_ascii_alphabetic() || first == '_') {
                return None;
            }
            self.position += first.len_utf8();
            self.consume_while(|ch| ch.is_ascii_alphanumeric() || ch == '_');
        }
        Some(self.input[start..self.position].to_string())
    }

    fn skip_ws(&mut self) {
        self.consume_while(|ch| ch.is_whitespace());
    }

    fn consume_while<F>(&mut self, predicate: F)
    where
        F: Fn(char) -> bool,
    {
        while let Some(ch) = self.peek_char() {
            if !predicate(ch) {
                break;
            }
            self.position += ch.len_utf8();
        }
    }

    fn expect_char(&mut self, expected: char) -> Option<()> {
        if self.consume_char_if(expected) {
            Some(())
        } else {
            None
        }
    }

    fn consume_char_if(&mut self, expected: char) -> bool {
        match self.peek_char() {
            Some(ch) if ch == expected => {
                self.position += ch.len_utf8();
                true
            }
            _ => false,
        }
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.position += ch.len_utf8();
        Some(ch)
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }
}

impl fmt::Display for UnifiedSemanticAST {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pretty_print())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_semantic_never_errors_and_falls_back_to_raw() {
        let logger = crate::test_runner::NoOpLogger;
        let parsed = UnifiedSemanticAST::parse_bootstrap("not a transform", &logger)
            .expect("bootstrap semantic parser should not error on unknown syntax");
        assert!(matches!(
            parsed,
            UnifiedSemanticAST::Raw { ref content } if content == "not a transform"
        ));
    }

    #[test]
    fn bootstrap_semantic_detects_transform_by_substring_markers() {
        let logger = crate::test_runner::NoOpLogger;
        let parsed =
            UnifiedSemanticAST::parse_bootstrap("str::parse::<u32>().unwrap_or(0)", &logger)
                .expect("bootstrap semantic parser should detect known transform marker pattern");
        assert!(matches!(parsed, UnifiedSemanticAST::TransformExpr { .. }));
    }

    #[test]
    fn bootstrap_semantic_detection_is_marker_based_not_structural() {
        let logger = crate::test_runner::NoOpLogger;
        let parsed =
            UnifiedSemanticAST::parse_bootstrap("x>().unwrap_or(0) ... ::parse::<u8>", &logger)
                .expect("bootstrap semantic parser only checks marker substrings");
        assert!(matches!(parsed, UnifiedSemanticAST::TransformExpr { .. }));
    }

    #[test]
    fn bootstrap_semantic_trims_outer_whitespace() {
        let logger = crate::test_runner::NoOpLogger;
        let parsed = UnifiedSemanticAST::parse_bootstrap("   value   ", &logger)
            .expect("bootstrap semantic parser should trim outer whitespace");
        assert!(matches!(
            parsed,
            UnifiedSemanticAST::Structured { ref canonical, .. } if canonical == "value"
        ));
    }

    #[test]
    fn bootstrap_semantic_structures_simple_array_payload() {
        let logger = crate::test_runner::NoOpLogger;
        let parsed = UnifiedSemanticAST::parse_bootstrap(" [9, $1, foo] ", &logger)
            .expect("bootstrap semantic parser should retain structured subset payloads");
        assert!(matches!(
            parsed,
            UnifiedSemanticAST::Structured { ref canonical, .. } if canonical == "[9, $1, foo]"
        ));
    }

    #[test]
    fn bootstrap_semantic_structures_object_payload_with_rule_reference() {
        let logger = crate::test_runner::NoOpLogger;
        let parsed = UnifiedSemanticAST::parse_bootstrap(
            " { kind: type_name, name: $class_identifier } ",
            &logger,
        )
        .expect("bootstrap semantic parser should retain structured object payloads");
        assert!(matches!(
            parsed,
            UnifiedSemanticAST::Structured { ref canonical, .. }
                if canonical == "{ kind: type_name, name: $class_identifier }"
        ));
    }

    #[test]
    fn bootstrap_semantic_structures_single_quoted_and_dotted_identifier_payloads() {
        let logger = crate::test_runner::NoOpLogger;

        let single_quoted = UnifiedSemanticAST::parse_bootstrap(" 'scoped_name' ", &logger)
            .expect("bootstrap semantic parser should retain single-quoted strings");
        assert!(matches!(
            single_quoted,
            UnifiedSemanticAST::Structured { ref canonical, .. } if canonical == "'scoped_name'"
        ));

        let dotted_identifier = UnifiedSemanticAST::parse_bootstrap(" lhs.ready ", &logger)
            .expect("bootstrap semantic parser should retain dotted identifiers");
        assert!(matches!(
            dotted_identifier,
            UnifiedSemanticAST::Structured { ref canonical, .. } if canonical == "lhs.ready"
        ));
    }

    #[test]
    fn bootstrap_semantic_invalid_structured_prefix_falls_back_to_raw() {
        let logger = crate::test_runner::NoOpLogger;
        let parsed = UnifiedSemanticAST::parse_bootstrap("{ kind: }", &logger)
            .expect("bootstrap semantic parser should never error on invalid structured syntax");
        assert!(matches!(
            parsed,
            UnifiedSemanticAST::Raw { ref content } if content == "{ kind: }"
        ));
    }

    #[cfg(feature = "generated_parsers")]
    #[test]
    fn generated_semantic_tree_to_ast_supports_transform_and_named_raw() {
        use crate::generated_parsers::semantic_annotation::Semantic_annotationParser;

        let logger = crate::test_runner::NoOpLogger;
        let transform_input = "@transform: $1";
        let mut transform_parser =
            Semantic_annotationParser::new(transform_input, Box::new(crate::NoOpLogger));
        let transform_tree = transform_parser
            .parse_full_semantic_annotation()
            .expect("generated parser should parse transform sample");
        let transform_ast = UnifiedSemanticAST::parse_generated_semantic_annotation(
            transform_input,
            &transform_tree,
            &logger,
        )
        .expect("generated tree -> semantic ast should succeed for transform");
        assert!(matches!(
            transform_ast,
            UnifiedSemanticAST::TransformExpr { ref expression }
                if expression == "$1"
        ));

        let raw_input = "@priority: [9, 1]";
        let mut raw_parser = Semantic_annotationParser::new(raw_input, Box::new(crate::NoOpLogger));
        let raw_tree = raw_parser
            .parse_full_semantic_annotation()
            .expect("generated parser should parse non-transform sample");
        let raw_ast =
            UnifiedSemanticAST::parse_generated_semantic_annotation(raw_input, &raw_tree, &logger)
                .expect("generated tree -> semantic ast should succeed for named raw");
        assert!(matches!(
            raw_ast,
            UnifiedSemanticAST::Structured { ref canonical, .. } if canonical == "[9, 1]"
        ));
    }

    #[cfg(feature = "generated_parsers")]
    #[test]
    fn generated_semantic_tree_to_entry_returns_name_and_payload_ast() {
        use crate::generated_parsers::semantic_annotation::Semantic_annotationParser;

        let logger = crate::test_runner::NoOpLogger;
        let input = "@priority: [9, 1]";
        let mut parser = Semantic_annotationParser::new(input, Box::new(crate::NoOpLogger));
        let parse_tree = parser
            .parse_full_semantic_annotation()
            .expect("generated parser should parse sample");
        let (name, ast) = UnifiedSemanticAST::parse_generated_semantic_annotation_entry(
            input,
            &parse_tree,
            &logger,
        )
        .expect("generated tree -> semantic entry should succeed");
        assert_eq!(name, "priority");
        assert!(matches!(
            ast,
            UnifiedSemanticAST::Structured { ref canonical, .. } if canonical == "[9, 1]"
        ));
    }

    #[cfg(feature = "generated_parsers")]
    #[test]
    fn generated_semantic_tree_to_ast_matches_expected_pass_semantic_corpus_contract() {
        use crate::generated_parsers::semantic_annotation::Semantic_annotationParser;
        use crate::test_runner::round_trip_tests::RoundTripTestRunner;

        fn expectation_is_pass(expectation: &str) -> bool {
            !matches!(
                expectation.trim().to_ascii_lowercase().as_str(),
                "fail" | "expected_fail" | "skip"
            )
        }

        fn is_semantic_parser_type(parser_type: &str) -> bool {
            matches!(
                parser_type.trim().to_ascii_lowercase().as_str(),
                "semantic" | "semantic_annotation" | "semantic_annotations"
            )
        }

        fn ast_payload(ast: &UnifiedSemanticAST) -> &str {
            ast.payload_text()
        }

        let logger = crate::test_runner::NoOpLogger;
        let suites = RoundTripTestRunner::new()
            .discover_test_suites()
            .expect("semantic corpus suites should load");

        let mut checked = 0usize;
        let mut bootstrap_comparable_checked = 0usize;
        for suite in suites {
            for test in suite.tests {
                if test.skip || !is_semantic_parser_type(&test.parser_type) {
                    continue;
                }
                if !expectation_is_pass(&test.expectations.generated_parser) {
                    continue;
                }

                let mut parser =
                    Semantic_annotationParser::new(&test.input, Box::new(crate::NoOpLogger));
                let parse_tree = parser.parse_full_semantic_annotation().unwrap_or_else(|err| {
                    panic!(
                        "generated parser should parse semantic corpus case '{} / {}' (input='{}'): {}",
                        suite.name, test.name, test.input, err
                    )
                });
                let direct_ast = UnifiedSemanticAST::parse_generated_semantic_annotation(
                    &test.input,
                    &parse_tree,
                    &logger,
                )
                .unwrap_or_else(|err| {
                    panic!(
                        "generated semantic tree -> ast should succeed for '{} / {}' (input='{}'): {}",
                        suite.name, test.name, test.input, err
                    )
                });
                let (name, entry_ast) = UnifiedSemanticAST::parse_generated_semantic_annotation_entry(
                    &test.input,
                    &parse_tree,
                    &logger,
                )
                .unwrap_or_else(|err| {
                    panic!(
                        "generated semantic tree -> entry should succeed for '{} / {}' (input='{}'): {}",
                        suite.name, test.name, test.input, err
                    )
                });

                assert_eq!(
                    direct_ast, entry_ast,
                    "entry and direct semantic AST conversion mismatch for '{} / {}' (input='{}')",
                    suite.name, test.name, test.input
                );
                if name == "transform" {
                    assert!(
                        matches!(entry_ast, UnifiedSemanticAST::TransformExpr { .. }),
                        "transform directive should convert to TransformExpr for '{} / {}' (input='{}')",
                        suite.name,
                        test.name,
                        test.input
                    );
                } else {
                    assert!(
                        !matches!(entry_ast, UnifiedSemanticAST::TransformExpr { .. }),
                        "non-transform directive should not convert to TransformExpr for '{} / {}' (input='{}')",
                        suite.name,
                        test.name,
                        test.input
                    );
                }

                let payload = ast_payload(&entry_ast).to_string();
                let canonical = format!("@{}: {}", name, payload);
                let mut canonical_parser =
                    Semantic_annotationParser::new(&canonical, Box::new(crate::NoOpLogger));
                let canonical_tree = canonical_parser
                    .parse_full_semantic_annotation()
                    .unwrap_or_else(|err| {
                        panic!(
                            "canonical semantic annotation should parse for '{} / {}' (canonical='{}'): {}",
                            suite.name, test.name, canonical, err
                        )
                    });
                let (canonical_name, canonical_ast) =
                    UnifiedSemanticAST::parse_generated_semantic_annotation_entry(
                        &canonical,
                        &canonical_tree,
                        &logger,
                    )
                    .unwrap_or_else(|err| {
                        panic!(
                            "canonical semantic tree -> entry should succeed for '{} / {}' (canonical='{}'): {}",
                            suite.name, test.name, canonical, err
                        )
                    });
                assert_eq!(
                    canonical_name, name,
                    "canonical semantic name changed for '{} / {}' (canonical='{}')",
                    suite.name, test.name, canonical
                );
                assert_eq!(
                    canonical_ast, entry_ast,
                    "canonical semantic AST changed for '{} / {}' (canonical='{}')",
                    suite.name, test.name, canonical
                );

                if expectation_is_pass(&test.expectations.bootstrap_parser) && name != "transform" {
                    let bootstrap_ast = UnifiedSemanticAST::parse_bootstrap(&payload, &logger)
                        .unwrap_or_else(|err| {
                            panic!(
                                "bootstrap parser should parse comparable semantic payload for '{} / {}' (payload='{}'): {}",
                                suite.name, test.name, payload, err
                            )
                        });
                    assert_eq!(
                        entry_ast, bootstrap_ast,
                        "generated/bootstrap payload AST mismatch for semantic corpus case '{} / {}' (payload='{}')",
                        suite.name, test.name, payload
                    );
                    bootstrap_comparable_checked += 1;
                }

                checked += 1;
            }
        }

        assert!(
            checked > 0,
            "expected at least one generated-pass semantic corpus case"
        );
        assert!(
            bootstrap_comparable_checked > 0,
            "expected at least one bootstrap-comparable semantic corpus case"
        );
    }
}
