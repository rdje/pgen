use std::collections::HashMap;
use std::ops::Range;
use regex::Regex;
use crate::ast_pipeline::{
    Logger, ParseResult, ParseError, ParseContent, ParseNode, MemoEntry, RuleId,
    CycleType, RecursionGuard,
};
/// High-performance parser with memoization and zero-copy parsing
pub struct ReturnAnnotationParser<'input> {
    input: &'input str,
    position: usize,
    memo: HashMap<(RuleId, usize), Option<ParseNode<'input>>>,
    recursion_guard: RecursionGuard,
    logger: Box<dyn Logger>,
}
impl<'input> ReturnAnnotationParser<'input> {
    const RULE_RETURN_ANNOTATION: RuleId = 0u16;
    const RULE_ARROW: RuleId = 1u16;
    const RULE_EXPRESSION: RuleId = 2u16;
    const RULE_PRIMARY_EXPRESSION: RuleId = 3u16;
    const RULE_EXTRACTION_EXPRESSION: RuleId = 4u16;
    const RULE_EXTRACTION_TARGET: RuleId = 5u16;
    const RULE_POSITIVE_INTEGER: RuleId = 6u16;
    const RULE_SPREAD_EXPRESSION: RuleId = 7u16;
    const RULE_SPREADABLE_EXPRESSION: RuleId = 8u16;
    const RULE_SPREAD_SUFFIX: RuleId = 9u16;
    const RULE_PROPERTY_ACCESS_EXPRESSION: RuleId = 10u16;
    const RULE_ARRAY_ACCESS_EXPRESSION: RuleId = 11u16;
    const RULE_ACCESSOR_BASE_LR_BASE: RuleId = 12u16;
    const RULE_ACCESSOR_BASE: RuleId = 13u16;
    const RULE_POSITIONAL_REFERENCE: RuleId = 14u16;
    const RULE_STRING_LITERAL: RuleId = 15u16;
    const RULE_STRING_CONTENT_DOUBLE: RuleId = 16u16;
    const RULE_STRING_CONTENT_SINGLE: RuleId = 17u16;
    const RULE_NUMBER_LITERAL: RuleId = 18u16;
    const RULE_FLOAT: RuleId = 19u16;
    const RULE_INTEGER: RuleId = 20u16;
    const RULE_BOOLEAN_LITERAL: RuleId = 21u16;
    const RULE_IDENTIFIER: RuleId = 22u16;
    const RULE_OBJECT_LITERAL: RuleId = 23u16;
    const RULE_OBJECT_PROPERTIES: RuleId = 24u16;
    const RULE_OBJECT_PROPERTY: RuleId = 25u16;
    const RULE_PROPERTY_KEY: RuleId = 26u16;
    const RULE_ARRAY_LITERAL: RuleId = 27u16;
    const RULE_ARRAY_ELEMENTS: RuleId = 28u16;
    const RULE_ARRAY_ELEMENT: RuleId = 29u16;
    const RULE_PARENTHESIZED: RuleId = 30u16;
    pub fn new(input: &'input str, logger: Box<dyn Logger>) -> Self {
        Self {
            input,
            position: 0,
            memo: HashMap::new(),
            recursion_guard: RecursionGuard::new(100),
            logger,
        }
    }
    pub fn parse(&mut self) -> ParseResult<ParseNode<'input>> {
        self.parse_return_annotation()
    }
    pub fn parse_full(&mut self) -> ParseResult<ParseNode<'input>> {
        let parsed = self.parse_return_annotation()?;
        if parsed.span.end == self.input.len() {
            Ok(parsed)
        } else {
            Err(ParseError::InvalidSyntax {
                message: "Parser did not consume full input",
                position: parsed.span.end,
            })
        }
    }
    pub fn parse_full_return_annotation(&mut self) -> ParseResult<ParseNode<'input>> {
        self.parse_full()
    }
    pub fn parse_return_annotation(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("return_annotation", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "return_annotation", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "return_annotation", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "return_annotation", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("return_annotation", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_RETURN_ANNOTATION,
                |parser| {
                    let parse_start = parser.position;
                    let mut best_content: Option<ParseContent<'input>> = None;
                    let mut best_end = parse_start;
                    let mut best_priority: i64 = i64::MIN;
                    let mut best_branch_index: usize = 0usize;
                    let mut best_branch = 0usize;
                    let mut nonassoc_tie = false;
                    let mut result = ParseContent::Sequence(Vec::new());
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            1usize, 3usize, "return_annotation", parser.position
                                        ),
                                    );
                            }
                            let mut sequence_elements = Vec::with_capacity(2usize);
                            {
                                let element_start = parser.position;
                                let element_content = {
                                    let result = ParseContent::Alternative(
                                        Box::new(parser.parse_arrow()?),
                                    );
                                    result
                                };
                                let element_end = parser.position;
                                sequence_elements
                                    .push(ParseNode {
                                        rule_name: "element_0",
                                        content: element_content,
                                        span: element_start..element_end,
                                    });
                            }
                            {
                                let element_start = parser.position;
                                let element_content = {
                                    let result = ParseContent::Alternative(
                                        Box::new(parser.parse_expression()?),
                                    );
                                    result
                                };
                                let element_end = parser.position;
                                sequence_elements
                                    .push(ParseNode {
                                        rule_name: "element_1",
                                        content: element_content,
                                        span: element_start..element_end,
                                    });
                            }
                            let result = ParseContent::Sequence(sequence_elements);
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            1usize, 3usize, "return_annotation", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 0usize > best_branch_index,
                                "nonassoc" => {
                                    if 0usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 0usize;
                            best_branch = 1usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    1usize, 3usize, "return_annotation", parser.position
                                ),
                            );
                    }
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            2usize, 3usize, "return_annotation", parser.position
                                        ),
                                    );
                            }
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_arrow()?),
                            );
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            2usize, 3usize, "return_annotation", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 1usize > best_branch_index,
                                "nonassoc" => {
                                    if 1usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 1usize;
                            best_branch = 2usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    2usize, 3usize, "return_annotation", parser.position
                                ),
                            );
                    }
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            3usize, 3usize, "return_annotation", parser.position
                                        ),
                                    );
                            }
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_expression()?),
                            );
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            3usize, 3usize, "return_annotation", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 2usize > best_branch_index,
                                "nonassoc" => {
                                    if 2usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 2usize;
                            best_branch = 3usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    3usize, 3usize, "return_annotation", parser.position
                                ),
                            );
                    }
                    if nonassoc_tie {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    } else if let Some(content) = best_content {
                        parser.position = best_end;
                        if parser.logger.is_enabled() {
                            parser
                                .logger
                                .log_info(
                                    "generated/return_annotation_parser.rs",
                                    0,
                                    &format!(
                                        "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={})",
                                        "return_annotation", best_branch, 3usize, best_end
                                        .saturating_sub(parse_start), best_priority, "left"
                                    ),
                                );
                        }
                        result = content;
                    } else {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    };
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "return_annotation",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "return_annotation", start_pos, node.span.end, consumed, &
                                    self.input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "return_annotation", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "return_annotation", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "return_annotation", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_arrow(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("arrow", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "arrow", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "arrow", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "arrow", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("arrow", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_ARROW,
                |parser| {
                    let matched_str = parser.match_string("->")?;
                    let result = ParseContent::Terminal(matched_str);
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "arrow",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "arrow", start_pos, node.span.end, consumed, & self
                                    .input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "arrow", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "arrow", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "arrow", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_expression(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("expression", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "expression", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "expression", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "expression", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("expression", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_EXPRESSION,
                |parser| {
                    let parse_start = parser.position;
                    let mut best_content: Option<ParseContent<'input>> = None;
                    let mut best_end = parse_start;
                    let mut best_priority: i64 = i64::MIN;
                    let mut best_branch_index: usize = 0usize;
                    let mut best_branch = 0usize;
                    let mut nonassoc_tie = false;
                    let mut result = ParseContent::Sequence(Vec::new());
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            1usize, 5usize, "expression", parser.position
                                        ),
                                    );
                            }
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_spread_expression()?),
                            );
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            1usize, 5usize, "expression", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 0usize > best_branch_index,
                                "nonassoc" => {
                                    if 0usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 0usize;
                            best_branch = 1usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    1usize, 5usize, "expression", parser.position
                                ),
                            );
                    }
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            2usize, 5usize, "expression", parser.position
                                        ),
                                    );
                            }
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_extraction_expression()?),
                            );
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            2usize, 5usize, "expression", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 1usize > best_branch_index,
                                "nonassoc" => {
                                    if 1usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 1usize;
                            best_branch = 2usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    2usize, 5usize, "expression", parser.position
                                ),
                            );
                    }
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            3usize, 5usize, "expression", parser.position
                                        ),
                                    );
                            }
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_property_access_expression()?),
                            );
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            3usize, 5usize, "expression", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 2usize > best_branch_index,
                                "nonassoc" => {
                                    if 2usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 2usize;
                            best_branch = 3usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    3usize, 5usize, "expression", parser.position
                                ),
                            );
                    }
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            4usize, 5usize, "expression", parser.position
                                        ),
                                    );
                            }
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_array_access_expression()?),
                            );
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            4usize, 5usize, "expression", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 3usize > best_branch_index,
                                "nonassoc" => {
                                    if 3usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 3usize;
                            best_branch = 4usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    4usize, 5usize, "expression", parser.position
                                ),
                            );
                    }
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            5usize, 5usize, "expression", parser.position
                                        ),
                                    );
                            }
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_primary_expression()?),
                            );
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            5usize, 5usize, "expression", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 4usize > best_branch_index,
                                "nonassoc" => {
                                    if 4usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 4usize;
                            best_branch = 5usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    5usize, 5usize, "expression", parser.position
                                ),
                            );
                    }
                    if nonassoc_tie {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    } else if let Some(content) = best_content {
                        parser.position = best_end;
                        if parser.logger.is_enabled() {
                            parser
                                .logger
                                .log_info(
                                    "generated/return_annotation_parser.rs",
                                    0,
                                    &format!(
                                        "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={})",
                                        "expression", best_branch, 5usize, best_end
                                        .saturating_sub(parse_start), best_priority, "left"
                                    ),
                                );
                        }
                        result = content;
                    } else {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    };
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "expression",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "expression", start_pos, node.span.end, consumed, & self
                                    .input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "expression", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "expression", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "expression", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_primary_expression(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self
            .recursion_guard
            .check_cycle("primary_expression", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "primary_expression", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "primary_expression", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "primary_expression", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("primary_expression", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_PRIMARY_EXPRESSION,
                |parser| {
                    let parse_start = parser.position;
                    let mut best_content: Option<ParseContent<'input>> = None;
                    let mut best_end = parse_start;
                    let mut best_priority: i64 = i64::MIN;
                    let mut best_branch_index: usize = 0usize;
                    let mut best_branch = 0usize;
                    let mut nonassoc_tie = false;
                    let mut result = ParseContent::Sequence(Vec::new());
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            1usize, 8usize, "primary_expression", parser.position
                                        ),
                                    );
                            }
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_object_literal()?),
                            );
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            1usize, 8usize, "primary_expression", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 0usize > best_branch_index,
                                "nonassoc" => {
                                    if 0usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 0usize;
                            best_branch = 1usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    1usize, 8usize, "primary_expression", parser.position
                                ),
                            );
                    }
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            2usize, 8usize, "primary_expression", parser.position
                                        ),
                                    );
                            }
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_array_literal()?),
                            );
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            2usize, 8usize, "primary_expression", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 1usize > best_branch_index,
                                "nonassoc" => {
                                    if 1usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 1usize;
                            best_branch = 2usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    2usize, 8usize, "primary_expression", parser.position
                                ),
                            );
                    }
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            3usize, 8usize, "primary_expression", parser.position
                                        ),
                                    );
                            }
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_positional_reference()?),
                            );
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            3usize, 8usize, "primary_expression", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 2usize > best_branch_index,
                                "nonassoc" => {
                                    if 2usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 2usize;
                            best_branch = 3usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    3usize, 8usize, "primary_expression", parser.position
                                ),
                            );
                    }
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            4usize, 8usize, "primary_expression", parser.position
                                        ),
                                    );
                            }
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_string_literal()?),
                            );
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            4usize, 8usize, "primary_expression", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 3usize > best_branch_index,
                                "nonassoc" => {
                                    if 3usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 3usize;
                            best_branch = 4usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    4usize, 8usize, "primary_expression", parser.position
                                ),
                            );
                    }
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            5usize, 8usize, "primary_expression", parser.position
                                        ),
                                    );
                            }
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_number_literal()?),
                            );
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            5usize, 8usize, "primary_expression", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 4usize > best_branch_index,
                                "nonassoc" => {
                                    if 4usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 4usize;
                            best_branch = 5usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    5usize, 8usize, "primary_expression", parser.position
                                ),
                            );
                    }
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            6usize, 8usize, "primary_expression", parser.position
                                        ),
                                    );
                            }
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_boolean_literal()?),
                            );
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            6usize, 8usize, "primary_expression", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 5usize > best_branch_index,
                                "nonassoc" => {
                                    if 5usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 5usize;
                            best_branch = 6usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    6usize, 8usize, "primary_expression", parser.position
                                ),
                            );
                    }
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            7usize, 8usize, "primary_expression", parser.position
                                        ),
                                    );
                            }
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_identifier()?),
                            );
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            7usize, 8usize, "primary_expression", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 6usize > best_branch_index,
                                "nonassoc" => {
                                    if 6usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 6usize;
                            best_branch = 7usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    7usize, 8usize, "primary_expression", parser.position
                                ),
                            );
                    }
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            8usize, 8usize, "primary_expression", parser.position
                                        ),
                                    );
                            }
                            let mut sequence_elements = Vec::with_capacity(3usize);
                            {
                                let element_start = parser.position;
                                let element_content = {
                                    let matched_str = parser.match_string("(")?;
                                    let result = ParseContent::Terminal(matched_str);
                                    result
                                };
                                let element_end = parser.position;
                                sequence_elements
                                    .push(ParseNode {
                                        rule_name: "element_0",
                                        content: element_content,
                                        span: element_start..element_end,
                                    });
                            }
                            {
                                let element_start = parser.position;
                                let element_content = {
                                    let result = ParseContent::Alternative(
                                        Box::new(parser.parse_expression()?),
                                    );
                                    result
                                };
                                let element_end = parser.position;
                                sequence_elements
                                    .push(ParseNode {
                                        rule_name: "element_1",
                                        content: element_content,
                                        span: element_start..element_end,
                                    });
                            }
                            {
                                let element_start = parser.position;
                                let element_content = {
                                    let matched_str = parser.match_string(")")?;
                                    let result = ParseContent::Terminal(matched_str);
                                    result
                                };
                                let element_end = parser.position;
                                sequence_elements
                                    .push(ParseNode {
                                        rule_name: "element_2",
                                        content: element_content,
                                        span: element_start..element_end,
                                    });
                            }
                            let result = ParseContent::Sequence(sequence_elements);
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            8usize, 8usize, "primary_expression", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 7usize > best_branch_index,
                                "nonassoc" => {
                                    if 7usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 7usize;
                            best_branch = 8usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    8usize, 8usize, "primary_expression", parser.position
                                ),
                            );
                    }
                    if nonassoc_tie {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    } else if let Some(content) = best_content {
                        parser.position = best_end;
                        if parser.logger.is_enabled() {
                            parser
                                .logger
                                .log_info(
                                    "generated/return_annotation_parser.rs",
                                    0,
                                    &format!(
                                        "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={})",
                                        "primary_expression", best_branch, 8usize, best_end
                                        .saturating_sub(parse_start), best_priority, "left"
                                    ),
                                );
                        }
                        result = content;
                    } else {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    };
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "primary_expression",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "primary_expression", start_pos, node.span.end, consumed, &
                                    self.input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "primary_expression", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "primary_expression", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "primary_expression", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_extraction_expression(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self
            .recursion_guard
            .check_cycle("extraction_expression", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "extraction_expression", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "extraction_expression", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "extraction_expression", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("extraction_expression", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_EXTRACTION_EXPRESSION,
                |parser| {
                    let mut sequence_elements = Vec::with_capacity(4usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_positional_reference()?),
                            );
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_0",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let matched_str = parser.match_string("::")?;
                            let result = ParseContent::Terminal(matched_str);
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_1",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_extraction_target()?),
                            );
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_2",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    {
                        let element_start = parser.position;
                        let element_content = if let Some(content) = parser
                            .try_parse(|p| {
                                let parser = p;
                                let result = ParseContent::Alternative(
                                    Box::new(parser.parse_spread_suffix()?),
                                );
                                Ok(result)
                            })
                        {
                            content
                        } else {
                            ParseContent::Sequence(Vec::new())
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_3",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    let result = ParseContent::Sequence(sequence_elements);
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "extraction_expression",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "extraction_expression", start_pos, node.span.end, consumed,
                                    & self.input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "extraction_expression", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "extraction_expression", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "extraction_expression", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_extraction_target(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("extraction_target", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "extraction_target", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "extraction_target", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "extraction_target", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("extraction_target", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_EXTRACTION_TARGET,
                |parser| {
                    let parse_start = parser.position;
                    let mut best_content: Option<ParseContent<'input>> = None;
                    let mut best_end = parse_start;
                    let mut best_priority: i64 = i64::MIN;
                    let mut best_branch_index: usize = 0usize;
                    let mut best_branch = 0usize;
                    let mut nonassoc_tie = false;
                    let mut result = ParseContent::Sequence(Vec::new());
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            1usize, 3usize, "extraction_target", parser.position
                                        ),
                                    );
                            }
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_positive_integer()?),
                            );
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            1usize, 3usize, "extraction_target", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 0usize > best_branch_index,
                                "nonassoc" => {
                                    if 0usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 0usize;
                            best_branch = 1usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    1usize, 3usize, "extraction_target", parser.position
                                ),
                            );
                    }
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            2usize, 3usize, "extraction_target", parser.position
                                        ),
                                    );
                            }
                            let matched_str = parser.match_string("first")?;
                            let result = ParseContent::Terminal(matched_str);
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            2usize, 3usize, "extraction_target", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 1usize > best_branch_index,
                                "nonassoc" => {
                                    if 1usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 1usize;
                            best_branch = 2usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    2usize, 3usize, "extraction_target", parser.position
                                ),
                            );
                    }
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            3usize, 3usize, "extraction_target", parser.position
                                        ),
                                    );
                            }
                            let matched_str = parser.match_string("last")?;
                            let result = ParseContent::Terminal(matched_str);
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            3usize, 3usize, "extraction_target", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 2usize > best_branch_index,
                                "nonassoc" => {
                                    if 2usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 2usize;
                            best_branch = 3usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    3usize, 3usize, "extraction_target", parser.position
                                ),
                            );
                    }
                    if nonassoc_tie {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    } else if let Some(content) = best_content {
                        parser.position = best_end;
                        if parser.logger.is_enabled() {
                            parser
                                .logger
                                .log_info(
                                    "generated/return_annotation_parser.rs",
                                    0,
                                    &format!(
                                        "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={})",
                                        "extraction_target", best_branch, 3usize, best_end
                                        .saturating_sub(parse_start), best_priority, "left"
                                    ),
                                );
                        }
                        result = content;
                    } else {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    };
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "extraction_target",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "extraction_target", start_pos, node.span.end, consumed, &
                                    self.input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "extraction_target", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "extraction_target", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "extraction_target", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_positive_integer(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("positive_integer", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "positive_integer", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "positive_integer", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "positive_integer", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("positive_integer", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_POSITIVE_INTEGER,
                |parser| {
                    let matched_str = parser.match_regex("[1-9][0-9]*", true)?;
                    let result = ParseContent::Terminal(matched_str);
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "positive_integer",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "positive_integer", start_pos, node.span.end, consumed, &
                                    self.input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "positive_integer", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "positive_integer", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "positive_integer", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_spread_expression(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("spread_expression", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "spread_expression", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "spread_expression", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "spread_expression", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("spread_expression", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_SPREAD_EXPRESSION,
                |parser| {
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_spreadable_expression()?),
                            );
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_0",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let matched_str = parser.match_string("*")?;
                            let result = ParseContent::Terminal(matched_str);
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_1",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    let result = ParseContent::Sequence(sequence_elements);
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "spread_expression",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "spread_expression", start_pos, node.span.end, consumed, &
                                    self.input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "spread_expression", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "spread_expression", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "spread_expression", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_spreadable_expression(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self
            .recursion_guard
            .check_cycle("spreadable_expression", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "spreadable_expression", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "spreadable_expression", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "spreadable_expression", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("spreadable_expression", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_SPREADABLE_EXPRESSION,
                |parser| {
                    let parse_start = parser.position;
                    let mut best_content: Option<ParseContent<'input>> = None;
                    let mut best_end = parse_start;
                    let mut best_priority: i64 = i64::MIN;
                    let mut best_branch_index: usize = 0usize;
                    let mut best_branch = 0usize;
                    let mut nonassoc_tie = false;
                    let mut result = ParseContent::Sequence(Vec::new());
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            1usize, 5usize, "spreadable_expression", parser.position
                                        ),
                                    );
                            }
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_extraction_expression()?),
                            );
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            1usize, 5usize, "spreadable_expression", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 0usize > best_branch_index,
                                "nonassoc" => {
                                    if 0usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 0usize;
                            best_branch = 1usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    1usize, 5usize, "spreadable_expression", parser.position
                                ),
                            );
                    }
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            2usize, 5usize, "spreadable_expression", parser.position
                                        ),
                                    );
                            }
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_positional_reference()?),
                            );
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            2usize, 5usize, "spreadable_expression", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 1usize > best_branch_index,
                                "nonassoc" => {
                                    if 1usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 1usize;
                            best_branch = 2usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    2usize, 5usize, "spreadable_expression", parser.position
                                ),
                            );
                    }
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            3usize, 5usize, "spreadable_expression", parser.position
                                        ),
                                    );
                            }
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_property_access_expression()?),
                            );
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            3usize, 5usize, "spreadable_expression", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 2usize > best_branch_index,
                                "nonassoc" => {
                                    if 2usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 2usize;
                            best_branch = 3usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    3usize, 5usize, "spreadable_expression", parser.position
                                ),
                            );
                    }
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            4usize, 5usize, "spreadable_expression", parser.position
                                        ),
                                    );
                            }
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_array_access_expression()?),
                            );
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            4usize, 5usize, "spreadable_expression", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 3usize > best_branch_index,
                                "nonassoc" => {
                                    if 3usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 3usize;
                            best_branch = 4usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    4usize, 5usize, "spreadable_expression", parser.position
                                ),
                            );
                    }
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            5usize, 5usize, "spreadable_expression", parser.position
                                        ),
                                    );
                            }
                            let mut sequence_elements = Vec::with_capacity(3usize);
                            {
                                let element_start = parser.position;
                                let element_content = {
                                    let matched_str = parser.match_string("(")?;
                                    let result = ParseContent::Terminal(matched_str);
                                    result
                                };
                                let element_end = parser.position;
                                sequence_elements
                                    .push(ParseNode {
                                        rule_name: "element_0",
                                        content: element_content,
                                        span: element_start..element_end,
                                    });
                            }
                            {
                                let element_start = parser.position;
                                let element_content = {
                                    let result = ParseContent::Alternative(
                                        Box::new(parser.parse_expression()?),
                                    );
                                    result
                                };
                                let element_end = parser.position;
                                sequence_elements
                                    .push(ParseNode {
                                        rule_name: "element_1",
                                        content: element_content,
                                        span: element_start..element_end,
                                    });
                            }
                            {
                                let element_start = parser.position;
                                let element_content = {
                                    let matched_str = parser.match_string(")")?;
                                    let result = ParseContent::Terminal(matched_str);
                                    result
                                };
                                let element_end = parser.position;
                                sequence_elements
                                    .push(ParseNode {
                                        rule_name: "element_2",
                                        content: element_content,
                                        span: element_start..element_end,
                                    });
                            }
                            let result = ParseContent::Sequence(sequence_elements);
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            5usize, 5usize, "spreadable_expression", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 4usize > best_branch_index,
                                "nonassoc" => {
                                    if 4usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 4usize;
                            best_branch = 5usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    5usize, 5usize, "spreadable_expression", parser.position
                                ),
                            );
                    }
                    if nonassoc_tie {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    } else if let Some(content) = best_content {
                        parser.position = best_end;
                        if parser.logger.is_enabled() {
                            parser
                                .logger
                                .log_info(
                                    "generated/return_annotation_parser.rs",
                                    0,
                                    &format!(
                                        "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={})",
                                        "spreadable_expression", best_branch, 5usize, best_end
                                        .saturating_sub(parse_start), best_priority, "left"
                                    ),
                                );
                        }
                        result = content;
                    } else {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    };
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "spreadable_expression",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "spreadable_expression", start_pos, node.span.end, consumed,
                                    & self.input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "spreadable_expression", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "spreadable_expression", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "spreadable_expression", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_spread_suffix(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("spread_suffix", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "spread_suffix", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "spread_suffix", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "spread_suffix", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("spread_suffix", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_SPREAD_SUFFIX,
                |parser| {
                    let matched_str = parser.match_string("*")?;
                    let result = ParseContent::Terminal(matched_str);
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "spread_suffix",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "spread_suffix", start_pos, node.span.end, consumed, & self
                                    .input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "spread_suffix", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "spread_suffix", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "spread_suffix", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_property_access_expression(
        &mut self,
    ) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self
            .recursion_guard
            .check_cycle("property_access_expression", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "property_access_expression", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "property_access_expression", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "property_access_expression", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("property_access_expression", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_PROPERTY_ACCESS_EXPRESSION,
                |parser| {
                    let mut sequence_elements = Vec::with_capacity(3usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_accessor_base_lr_base()?),
                            );
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_0",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let mut sequence_elements = Vec::with_capacity(2usize);
                            {
                                let element_start = parser.position;
                                let element_content = {
                                    let matched_str = parser.match_string(".")?;
                                    let result = ParseContent::Terminal(matched_str);
                                    result
                                };
                                let element_end = parser.position;
                                sequence_elements
                                    .push(ParseNode {
                                        rule_name: "element_0",
                                        content: element_content,
                                        span: element_start..element_end,
                                    });
                            }
                            {
                                let element_start = parser.position;
                                let element_content = {
                                    let result = ParseContent::Alternative(
                                        Box::new(parser.parse_identifier()?),
                                    );
                                    result
                                };
                                let element_end = parser.position;
                                sequence_elements
                                    .push(ParseNode {
                                        rule_name: "element_1",
                                        content: element_content,
                                        span: element_start..element_end,
                                    });
                            }
                            let result = ParseContent::Sequence(sequence_elements);
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_1",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let mut results = Vec::new();
                            let mut last_position = parser.position;
                            let mut iteration_count = 0;
                            const MAX_ITERATIONS: usize = 10000;
                            while iteration_count < MAX_ITERATIONS {
                                if let Some(node) = parser
                                    .try_parse(|p| {
                                        let parser = p;
                                        let parse_start = parser.position;
                                        let mut best_content: Option<ParseContent<'input>> = None;
                                        let mut best_end = parse_start;
                                        let mut best_priority: i64 = i64::MIN;
                                        let mut best_branch_index: usize = 0usize;
                                        let mut best_branch = 0usize;
                                        let mut nonassoc_tie = false;
                                        let mut result = ParseContent::Sequence(Vec::new());
                                        parser.position = parse_start;
                                        if let Some(content) = parser
                                            .try_parse(|p| {
                                                let parser = p;
                                                if parser.logger.is_enabled() {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "generated/return_annotation_parser.rs",
                                                            0,
                                                            &format!(
                                                                "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                1usize, 2usize, "property_access_expression", parser
                                                                .position
                                                            ),
                                                        );
                                                }
                                                let mut sequence_elements = Vec::with_capacity(2usize);
                                                {
                                                    let element_start = parser.position;
                                                    let element_content = {
                                                        let matched_str = parser.match_string(".")?;
                                                        let result = ParseContent::Terminal(matched_str);
                                                        result
                                                    };
                                                    let element_end = parser.position;
                                                    sequence_elements
                                                        .push(ParseNode {
                                                            rule_name: "element_0",
                                                            content: element_content,
                                                            span: element_start..element_end,
                                                        });
                                                }
                                                {
                                                    let element_start = parser.position;
                                                    let element_content = {
                                                        let result = ParseContent::Alternative(
                                                            Box::new(parser.parse_identifier()?),
                                                        );
                                                        result
                                                    };
                                                    let element_end = parser.position;
                                                    sequence_elements
                                                        .push(ParseNode {
                                                            rule_name: "element_1",
                                                            content: element_content,
                                                            span: element_start..element_end,
                                                        });
                                                }
                                                let result = ParseContent::Sequence(sequence_elements);
                                                if parser.logger.is_enabled() {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "generated/return_annotation_parser.rs",
                                                            0,
                                                            &format!(
                                                                "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                1usize, 2usize, "property_access_expression", parser
                                                                .position
                                                            ),
                                                        );
                                                }
                                                Ok(result)
                                            })
                                        {
                                            let candidate_end = parser.position;
                                            parser.position = parse_start;
                                            let candidate_priority: i64 = 0i64;
                                            let transformed = {
                                                let content = content;
                                                content
                                            };
                                            let should_take = if best_content.is_none() {
                                                true
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => 0usize > best_branch_index,
                                                    "nonassoc" => {
                                                        if 0usize != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            };
                                            if should_take {
                                                best_end = candidate_end;
                                                best_priority = candidate_priority;
                                                best_branch_index = 0usize;
                                                best_branch = 1usize;
                                                best_content = Some(transformed);
                                            }
                                        } else if parser.logger.is_enabled() {
                                            parser
                                                .logger
                                                .log_info(
                                                    "generated/return_annotation_parser.rs",
                                                    0,
                                                    &format!(
                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                        1usize, 2usize, "property_access_expression", parser
                                                        .position
                                                    ),
                                                );
                                        }
                                        parser.position = parse_start;
                                        if let Some(content) = parser
                                            .try_parse(|p| {
                                                let parser = p;
                                                if parser.logger.is_enabled() {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "generated/return_annotation_parser.rs",
                                                            0,
                                                            &format!(
                                                                "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                2usize, 2usize, "property_access_expression", parser
                                                                .position
                                                            ),
                                                        );
                                                }
                                                let mut sequence_elements = Vec::with_capacity(3usize);
                                                {
                                                    let element_start = parser.position;
                                                    let element_content = {
                                                        let matched_str = parser.match_string("[")?;
                                                        let result = ParseContent::Terminal(matched_str);
                                                        result
                                                    };
                                                    let element_end = parser.position;
                                                    sequence_elements
                                                        .push(ParseNode {
                                                            rule_name: "element_0",
                                                            content: element_content,
                                                            span: element_start..element_end,
                                                        });
                                                }
                                                {
                                                    let element_start = parser.position;
                                                    let element_content = {
                                                        let result = ParseContent::Alternative(
                                                            Box::new(parser.parse_expression()?),
                                                        );
                                                        result
                                                    };
                                                    let element_end = parser.position;
                                                    sequence_elements
                                                        .push(ParseNode {
                                                            rule_name: "element_1",
                                                            content: element_content,
                                                            span: element_start..element_end,
                                                        });
                                                }
                                                {
                                                    let element_start = parser.position;
                                                    let element_content = {
                                                        let matched_str = parser.match_string("]")?;
                                                        let result = ParseContent::Terminal(matched_str);
                                                        result
                                                    };
                                                    let element_end = parser.position;
                                                    sequence_elements
                                                        .push(ParseNode {
                                                            rule_name: "element_2",
                                                            content: element_content,
                                                            span: element_start..element_end,
                                                        });
                                                }
                                                let result = ParseContent::Sequence(sequence_elements);
                                                if parser.logger.is_enabled() {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "generated/return_annotation_parser.rs",
                                                            0,
                                                            &format!(
                                                                "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                2usize, 2usize, "property_access_expression", parser
                                                                .position
                                                            ),
                                                        );
                                                }
                                                Ok(result)
                                            })
                                        {
                                            let candidate_end = parser.position;
                                            parser.position = parse_start;
                                            let candidate_priority: i64 = 0i64;
                                            let transformed = {
                                                let content = content;
                                                content
                                            };
                                            let should_take = if best_content.is_none() {
                                                true
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => 1usize > best_branch_index,
                                                    "nonassoc" => {
                                                        if 1usize != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            };
                                            if should_take {
                                                best_end = candidate_end;
                                                best_priority = candidate_priority;
                                                best_branch_index = 1usize;
                                                best_branch = 2usize;
                                                best_content = Some(transformed);
                                            }
                                        } else if parser.logger.is_enabled() {
                                            parser
                                                .logger
                                                .log_info(
                                                    "generated/return_annotation_parser.rs",
                                                    0,
                                                    &format!(
                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                        2usize, 2usize, "property_access_expression", parser
                                                        .position
                                                    ),
                                                );
                                        }
                                        if nonassoc_tie {
                                            return Err(ParseError::Backtrack {
                                                position: parse_start,
                                            });
                                        } else if let Some(content) = best_content {
                                            parser.position = best_end;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "generated/return_annotation_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={})",
                                                            "property_access_expression", best_branch, 2usize, best_end
                                                            .saturating_sub(parse_start), best_priority, "left"
                                                        ),
                                                    );
                                            }
                                            result = content;
                                        } else {
                                            return Err(ParseError::Backtrack {
                                                position: parse_start,
                                            });
                                        };
                                        Ok(ParseNode {
                                            rule_name: "quantified",
                                            content: result,
                                            span: 0..0,
                                        })
                                    })
                                {
                                    let current_position = parser.position;
                                    if current_position == last_position {
                                        if parser.logger.is_enabled() {
                                            parser
                                                .logger
                                                .log_warning(
                                                    "generated/return_annotation_parser.rs",
                                                    0,
                                                    &format!(
                                                        "⚠️ ZERO-LENGTH MATCH in quantifier: Breaking to prevent infinite loop at position {}",
                                                        current_position
                                                    ),
                                                );
                                        }
                                        break;
                                    }
                                    results.push(node);
                                    last_position = current_position;
                                    iteration_count += 1;
                                } else {
                                    break;
                                }
                            }
                            if iteration_count >= MAX_ITERATIONS
                                && parser.logger.is_enabled()
                            {
                                parser
                                    .logger
                                    .log_warning(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "⚠️ MAX ITERATIONS ({}) reached in quantifier",
                                            MAX_ITERATIONS
                                        ),
                                    );
                            }
                            let result = ParseContent::Quantified(results, "*");
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_2",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    let result = ParseContent::Sequence(sequence_elements);
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "property_access_expression",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "property_access_expression", start_pos, node.span.end,
                                    consumed, & self.input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "property_access_expression", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "property_access_expression", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "property_access_expression", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_array_access_expression(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self
            .recursion_guard
            .check_cycle("array_access_expression", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "array_access_expression", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "array_access_expression", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "array_access_expression", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("array_access_expression", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_ARRAY_ACCESS_EXPRESSION,
                |parser| {
                    let mut sequence_elements = Vec::with_capacity(3usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_accessor_base_lr_base()?),
                            );
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_0",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let mut sequence_elements = Vec::with_capacity(3usize);
                            {
                                let element_start = parser.position;
                                let element_content = {
                                    let matched_str = parser.match_string("[")?;
                                    let result = ParseContent::Terminal(matched_str);
                                    result
                                };
                                let element_end = parser.position;
                                sequence_elements
                                    .push(ParseNode {
                                        rule_name: "element_0",
                                        content: element_content,
                                        span: element_start..element_end,
                                    });
                            }
                            {
                                let element_start = parser.position;
                                let element_content = {
                                    let result = ParseContent::Alternative(
                                        Box::new(parser.parse_expression()?),
                                    );
                                    result
                                };
                                let element_end = parser.position;
                                sequence_elements
                                    .push(ParseNode {
                                        rule_name: "element_1",
                                        content: element_content,
                                        span: element_start..element_end,
                                    });
                            }
                            {
                                let element_start = parser.position;
                                let element_content = {
                                    let matched_str = parser.match_string("]")?;
                                    let result = ParseContent::Terminal(matched_str);
                                    result
                                };
                                let element_end = parser.position;
                                sequence_elements
                                    .push(ParseNode {
                                        rule_name: "element_2",
                                        content: element_content,
                                        span: element_start..element_end,
                                    });
                            }
                            let result = ParseContent::Sequence(sequence_elements);
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_1",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let mut results = Vec::new();
                            let mut last_position = parser.position;
                            let mut iteration_count = 0;
                            const MAX_ITERATIONS: usize = 10000;
                            while iteration_count < MAX_ITERATIONS {
                                if let Some(node) = parser
                                    .try_parse(|p| {
                                        let parser = p;
                                        let parse_start = parser.position;
                                        let mut best_content: Option<ParseContent<'input>> = None;
                                        let mut best_end = parse_start;
                                        let mut best_priority: i64 = i64::MIN;
                                        let mut best_branch_index: usize = 0usize;
                                        let mut best_branch = 0usize;
                                        let mut nonassoc_tie = false;
                                        let mut result = ParseContent::Sequence(Vec::new());
                                        parser.position = parse_start;
                                        if let Some(content) = parser
                                            .try_parse(|p| {
                                                let parser = p;
                                                if parser.logger.is_enabled() {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "generated/return_annotation_parser.rs",
                                                            0,
                                                            &format!(
                                                                "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                1usize, 2usize, "array_access_expression", parser.position
                                                            ),
                                                        );
                                                }
                                                let mut sequence_elements = Vec::with_capacity(2usize);
                                                {
                                                    let element_start = parser.position;
                                                    let element_content = {
                                                        let matched_str = parser.match_string(".")?;
                                                        let result = ParseContent::Terminal(matched_str);
                                                        result
                                                    };
                                                    let element_end = parser.position;
                                                    sequence_elements
                                                        .push(ParseNode {
                                                            rule_name: "element_0",
                                                            content: element_content,
                                                            span: element_start..element_end,
                                                        });
                                                }
                                                {
                                                    let element_start = parser.position;
                                                    let element_content = {
                                                        let result = ParseContent::Alternative(
                                                            Box::new(parser.parse_identifier()?),
                                                        );
                                                        result
                                                    };
                                                    let element_end = parser.position;
                                                    sequence_elements
                                                        .push(ParseNode {
                                                            rule_name: "element_1",
                                                            content: element_content,
                                                            span: element_start..element_end,
                                                        });
                                                }
                                                let result = ParseContent::Sequence(sequence_elements);
                                                if parser.logger.is_enabled() {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "generated/return_annotation_parser.rs",
                                                            0,
                                                            &format!(
                                                                "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                1usize, 2usize, "array_access_expression", parser.position
                                                            ),
                                                        );
                                                }
                                                Ok(result)
                                            })
                                        {
                                            let candidate_end = parser.position;
                                            parser.position = parse_start;
                                            let candidate_priority: i64 = 0i64;
                                            let transformed = {
                                                let content = content;
                                                content
                                            };
                                            let should_take = if best_content.is_none() {
                                                true
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => 0usize > best_branch_index,
                                                    "nonassoc" => {
                                                        if 0usize != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            };
                                            if should_take {
                                                best_end = candidate_end;
                                                best_priority = candidate_priority;
                                                best_branch_index = 0usize;
                                                best_branch = 1usize;
                                                best_content = Some(transformed);
                                            }
                                        } else if parser.logger.is_enabled() {
                                            parser
                                                .logger
                                                .log_info(
                                                    "generated/return_annotation_parser.rs",
                                                    0,
                                                    &format!(
                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                        1usize, 2usize, "array_access_expression", parser.position
                                                    ),
                                                );
                                        }
                                        parser.position = parse_start;
                                        if let Some(content) = parser
                                            .try_parse(|p| {
                                                let parser = p;
                                                if parser.logger.is_enabled() {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "generated/return_annotation_parser.rs",
                                                            0,
                                                            &format!(
                                                                "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                2usize, 2usize, "array_access_expression", parser.position
                                                            ),
                                                        );
                                                }
                                                let mut sequence_elements = Vec::with_capacity(3usize);
                                                {
                                                    let element_start = parser.position;
                                                    let element_content = {
                                                        let matched_str = parser.match_string("[")?;
                                                        let result = ParseContent::Terminal(matched_str);
                                                        result
                                                    };
                                                    let element_end = parser.position;
                                                    sequence_elements
                                                        .push(ParseNode {
                                                            rule_name: "element_0",
                                                            content: element_content,
                                                            span: element_start..element_end,
                                                        });
                                                }
                                                {
                                                    let element_start = parser.position;
                                                    let element_content = {
                                                        let result = ParseContent::Alternative(
                                                            Box::new(parser.parse_expression()?),
                                                        );
                                                        result
                                                    };
                                                    let element_end = parser.position;
                                                    sequence_elements
                                                        .push(ParseNode {
                                                            rule_name: "element_1",
                                                            content: element_content,
                                                            span: element_start..element_end,
                                                        });
                                                }
                                                {
                                                    let element_start = parser.position;
                                                    let element_content = {
                                                        let matched_str = parser.match_string("]")?;
                                                        let result = ParseContent::Terminal(matched_str);
                                                        result
                                                    };
                                                    let element_end = parser.position;
                                                    sequence_elements
                                                        .push(ParseNode {
                                                            rule_name: "element_2",
                                                            content: element_content,
                                                            span: element_start..element_end,
                                                        });
                                                }
                                                let result = ParseContent::Sequence(sequence_elements);
                                                if parser.logger.is_enabled() {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "generated/return_annotation_parser.rs",
                                                            0,
                                                            &format!(
                                                                "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                2usize, 2usize, "array_access_expression", parser.position
                                                            ),
                                                        );
                                                }
                                                Ok(result)
                                            })
                                        {
                                            let candidate_end = parser.position;
                                            parser.position = parse_start;
                                            let candidate_priority: i64 = 0i64;
                                            let transformed = {
                                                let content = content;
                                                content
                                            };
                                            let should_take = if best_content.is_none() {
                                                true
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => 1usize > best_branch_index,
                                                    "nonassoc" => {
                                                        if 1usize != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            };
                                            if should_take {
                                                best_end = candidate_end;
                                                best_priority = candidate_priority;
                                                best_branch_index = 1usize;
                                                best_branch = 2usize;
                                                best_content = Some(transformed);
                                            }
                                        } else if parser.logger.is_enabled() {
                                            parser
                                                .logger
                                                .log_info(
                                                    "generated/return_annotation_parser.rs",
                                                    0,
                                                    &format!(
                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                        2usize, 2usize, "array_access_expression", parser.position
                                                    ),
                                                );
                                        }
                                        if nonassoc_tie {
                                            return Err(ParseError::Backtrack {
                                                position: parse_start,
                                            });
                                        } else if let Some(content) = best_content {
                                            parser.position = best_end;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "generated/return_annotation_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={})",
                                                            "array_access_expression", best_branch, 2usize, best_end
                                                            .saturating_sub(parse_start), best_priority, "left"
                                                        ),
                                                    );
                                            }
                                            result = content;
                                        } else {
                                            return Err(ParseError::Backtrack {
                                                position: parse_start,
                                            });
                                        };
                                        Ok(ParseNode {
                                            rule_name: "quantified",
                                            content: result,
                                            span: 0..0,
                                        })
                                    })
                                {
                                    let current_position = parser.position;
                                    if current_position == last_position {
                                        if parser.logger.is_enabled() {
                                            parser
                                                .logger
                                                .log_warning(
                                                    "generated/return_annotation_parser.rs",
                                                    0,
                                                    &format!(
                                                        "⚠️ ZERO-LENGTH MATCH in quantifier: Breaking to prevent infinite loop at position {}",
                                                        current_position
                                                    ),
                                                );
                                        }
                                        break;
                                    }
                                    results.push(node);
                                    last_position = current_position;
                                    iteration_count += 1;
                                } else {
                                    break;
                                }
                            }
                            if iteration_count >= MAX_ITERATIONS
                                && parser.logger.is_enabled()
                            {
                                parser
                                    .logger
                                    .log_warning(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "⚠️ MAX ITERATIONS ({}) reached in quantifier",
                                            MAX_ITERATIONS
                                        ),
                                    );
                            }
                            let result = ParseContent::Quantified(results, "*");
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_2",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    let result = ParseContent::Sequence(sequence_elements);
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "array_access_expression",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "array_access_expression", start_pos, node.span.end,
                                    consumed, & self.input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "array_access_expression", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "array_access_expression", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "array_access_expression", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_accessor_base_lr_base(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self
            .recursion_guard
            .check_cycle("accessor_base_lr_base", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "accessor_base_lr_base", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "accessor_base_lr_base", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "accessor_base_lr_base", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("accessor_base_lr_base", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_ACCESSOR_BASE_LR_BASE,
                |parser| {
                    let parse_start = parser.position;
                    let mut best_content: Option<ParseContent<'input>> = None;
                    let mut best_end = parse_start;
                    let mut best_priority: i64 = i64::MIN;
                    let mut best_branch_index: usize = 0usize;
                    let mut best_branch = 0usize;
                    let mut nonassoc_tie = false;
                    let mut result = ParseContent::Sequence(Vec::new());
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            1usize, 2usize, "accessor_base_lr_base", parser.position
                                        ),
                                    );
                            }
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_positional_reference()?),
                            );
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            1usize, 2usize, "accessor_base_lr_base", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 0usize > best_branch_index,
                                "nonassoc" => {
                                    if 0usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 0usize;
                            best_branch = 1usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    1usize, 2usize, "accessor_base_lr_base", parser.position
                                ),
                            );
                    }
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            2usize, 2usize, "accessor_base_lr_base", parser.position
                                        ),
                                    );
                            }
                            let mut sequence_elements = Vec::with_capacity(3usize);
                            {
                                let element_start = parser.position;
                                let element_content = {
                                    let matched_str = parser.match_string("(")?;
                                    let result = ParseContent::Terminal(matched_str);
                                    result
                                };
                                let element_end = parser.position;
                                sequence_elements
                                    .push(ParseNode {
                                        rule_name: "element_0",
                                        content: element_content,
                                        span: element_start..element_end,
                                    });
                            }
                            {
                                let element_start = parser.position;
                                let element_content = {
                                    let result = ParseContent::Alternative(
                                        Box::new(parser.parse_expression()?),
                                    );
                                    result
                                };
                                let element_end = parser.position;
                                sequence_elements
                                    .push(ParseNode {
                                        rule_name: "element_1",
                                        content: element_content,
                                        span: element_start..element_end,
                                    });
                            }
                            {
                                let element_start = parser.position;
                                let element_content = {
                                    let matched_str = parser.match_string(")")?;
                                    let result = ParseContent::Terminal(matched_str);
                                    result
                                };
                                let element_end = parser.position;
                                sequence_elements
                                    .push(ParseNode {
                                        rule_name: "element_2",
                                        content: element_content,
                                        span: element_start..element_end,
                                    });
                            }
                            let result = ParseContent::Sequence(sequence_elements);
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            2usize, 2usize, "accessor_base_lr_base", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 1usize > best_branch_index,
                                "nonassoc" => {
                                    if 1usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 1usize;
                            best_branch = 2usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    2usize, 2usize, "accessor_base_lr_base", parser.position
                                ),
                            );
                    }
                    if nonassoc_tie {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    } else if let Some(content) = best_content {
                        parser.position = best_end;
                        if parser.logger.is_enabled() {
                            parser
                                .logger
                                .log_info(
                                    "generated/return_annotation_parser.rs",
                                    0,
                                    &format!(
                                        "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={})",
                                        "accessor_base_lr_base", best_branch, 2usize, best_end
                                        .saturating_sub(parse_start), best_priority, "left"
                                    ),
                                );
                        }
                        result = content;
                    } else {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    };
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "accessor_base_lr_base",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "accessor_base_lr_base", start_pos, node.span.end, consumed,
                                    & self.input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "accessor_base_lr_base", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "accessor_base_lr_base", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "accessor_base_lr_base", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_accessor_base(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("accessor_base", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "accessor_base", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "accessor_base", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "accessor_base", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("accessor_base", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_ACCESSOR_BASE,
                |parser| {
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_accessor_base_lr_base()?),
                            );
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_0",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let mut results = Vec::new();
                            let mut last_position = parser.position;
                            let mut iteration_count = 0;
                            const MAX_ITERATIONS: usize = 10000;
                            while iteration_count < MAX_ITERATIONS {
                                if let Some(node) = parser
                                    .try_parse(|p| {
                                        let parser = p;
                                        let parse_start = parser.position;
                                        let mut best_content: Option<ParseContent<'input>> = None;
                                        let mut best_end = parse_start;
                                        let mut best_priority: i64 = i64::MIN;
                                        let mut best_branch_index: usize = 0usize;
                                        let mut best_branch = 0usize;
                                        let mut nonassoc_tie = false;
                                        let mut result = ParseContent::Sequence(Vec::new());
                                        parser.position = parse_start;
                                        if let Some(content) = parser
                                            .try_parse(|p| {
                                                let parser = p;
                                                if parser.logger.is_enabled() {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "generated/return_annotation_parser.rs",
                                                            0,
                                                            &format!(
                                                                "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                1usize, 2usize, "accessor_base", parser.position
                                                            ),
                                                        );
                                                }
                                                let mut sequence_elements = Vec::with_capacity(2usize);
                                                {
                                                    let element_start = parser.position;
                                                    let element_content = {
                                                        let matched_str = parser.match_string(".")?;
                                                        let result = ParseContent::Terminal(matched_str);
                                                        result
                                                    };
                                                    let element_end = parser.position;
                                                    sequence_elements
                                                        .push(ParseNode {
                                                            rule_name: "element_0",
                                                            content: element_content,
                                                            span: element_start..element_end,
                                                        });
                                                }
                                                {
                                                    let element_start = parser.position;
                                                    let element_content = {
                                                        let result = ParseContent::Alternative(
                                                            Box::new(parser.parse_identifier()?),
                                                        );
                                                        result
                                                    };
                                                    let element_end = parser.position;
                                                    sequence_elements
                                                        .push(ParseNode {
                                                            rule_name: "element_1",
                                                            content: element_content,
                                                            span: element_start..element_end,
                                                        });
                                                }
                                                let result = ParseContent::Sequence(sequence_elements);
                                                if parser.logger.is_enabled() {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "generated/return_annotation_parser.rs",
                                                            0,
                                                            &format!(
                                                                "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                1usize, 2usize, "accessor_base", parser.position
                                                            ),
                                                        );
                                                }
                                                Ok(result)
                                            })
                                        {
                                            let candidate_end = parser.position;
                                            parser.position = parse_start;
                                            let candidate_priority: i64 = 0i64;
                                            let transformed = {
                                                let content = content;
                                                content
                                            };
                                            let should_take = if best_content.is_none() {
                                                true
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => 0usize > best_branch_index,
                                                    "nonassoc" => {
                                                        if 0usize != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            };
                                            if should_take {
                                                best_end = candidate_end;
                                                best_priority = candidate_priority;
                                                best_branch_index = 0usize;
                                                best_branch = 1usize;
                                                best_content = Some(transformed);
                                            }
                                        } else if parser.logger.is_enabled() {
                                            parser
                                                .logger
                                                .log_info(
                                                    "generated/return_annotation_parser.rs",
                                                    0,
                                                    &format!(
                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                        1usize, 2usize, "accessor_base", parser.position
                                                    ),
                                                );
                                        }
                                        parser.position = parse_start;
                                        if let Some(content) = parser
                                            .try_parse(|p| {
                                                let parser = p;
                                                if parser.logger.is_enabled() {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "generated/return_annotation_parser.rs",
                                                            0,
                                                            &format!(
                                                                "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                2usize, 2usize, "accessor_base", parser.position
                                                            ),
                                                        );
                                                }
                                                let mut sequence_elements = Vec::with_capacity(3usize);
                                                {
                                                    let element_start = parser.position;
                                                    let element_content = {
                                                        let matched_str = parser.match_string("[")?;
                                                        let result = ParseContent::Terminal(matched_str);
                                                        result
                                                    };
                                                    let element_end = parser.position;
                                                    sequence_elements
                                                        .push(ParseNode {
                                                            rule_name: "element_0",
                                                            content: element_content,
                                                            span: element_start..element_end,
                                                        });
                                                }
                                                {
                                                    let element_start = parser.position;
                                                    let element_content = {
                                                        let result = ParseContent::Alternative(
                                                            Box::new(parser.parse_expression()?),
                                                        );
                                                        result
                                                    };
                                                    let element_end = parser.position;
                                                    sequence_elements
                                                        .push(ParseNode {
                                                            rule_name: "element_1",
                                                            content: element_content,
                                                            span: element_start..element_end,
                                                        });
                                                }
                                                {
                                                    let element_start = parser.position;
                                                    let element_content = {
                                                        let matched_str = parser.match_string("]")?;
                                                        let result = ParseContent::Terminal(matched_str);
                                                        result
                                                    };
                                                    let element_end = parser.position;
                                                    sequence_elements
                                                        .push(ParseNode {
                                                            rule_name: "element_2",
                                                            content: element_content,
                                                            span: element_start..element_end,
                                                        });
                                                }
                                                let result = ParseContent::Sequence(sequence_elements);
                                                if parser.logger.is_enabled() {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "generated/return_annotation_parser.rs",
                                                            0,
                                                            &format!(
                                                                "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                2usize, 2usize, "accessor_base", parser.position
                                                            ),
                                                        );
                                                }
                                                Ok(result)
                                            })
                                        {
                                            let candidate_end = parser.position;
                                            parser.position = parse_start;
                                            let candidate_priority: i64 = 0i64;
                                            let transformed = {
                                                let content = content;
                                                content
                                            };
                                            let should_take = if best_content.is_none() {
                                                true
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => 1usize > best_branch_index,
                                                    "nonassoc" => {
                                                        if 1usize != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            };
                                            if should_take {
                                                best_end = candidate_end;
                                                best_priority = candidate_priority;
                                                best_branch_index = 1usize;
                                                best_branch = 2usize;
                                                best_content = Some(transformed);
                                            }
                                        } else if parser.logger.is_enabled() {
                                            parser
                                                .logger
                                                .log_info(
                                                    "generated/return_annotation_parser.rs",
                                                    0,
                                                    &format!(
                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                        2usize, 2usize, "accessor_base", parser.position
                                                    ),
                                                );
                                        }
                                        if nonassoc_tie {
                                            return Err(ParseError::Backtrack {
                                                position: parse_start,
                                            });
                                        } else if let Some(content) = best_content {
                                            parser.position = best_end;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "generated/return_annotation_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={})",
                                                            "accessor_base", best_branch, 2usize, best_end
                                                            .saturating_sub(parse_start), best_priority, "left"
                                                        ),
                                                    );
                                            }
                                            result = content;
                                        } else {
                                            return Err(ParseError::Backtrack {
                                                position: parse_start,
                                            });
                                        };
                                        Ok(ParseNode {
                                            rule_name: "quantified",
                                            content: result,
                                            span: 0..0,
                                        })
                                    })
                                {
                                    let current_position = parser.position;
                                    if current_position == last_position {
                                        if parser.logger.is_enabled() {
                                            parser
                                                .logger
                                                .log_warning(
                                                    "generated/return_annotation_parser.rs",
                                                    0,
                                                    &format!(
                                                        "⚠️ ZERO-LENGTH MATCH in quantifier: Breaking to prevent infinite loop at position {}",
                                                        current_position
                                                    ),
                                                );
                                        }
                                        break;
                                    }
                                    results.push(node);
                                    last_position = current_position;
                                    iteration_count += 1;
                                } else {
                                    break;
                                }
                            }
                            if iteration_count >= MAX_ITERATIONS
                                && parser.logger.is_enabled()
                            {
                                parser
                                    .logger
                                    .log_warning(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "⚠️ MAX ITERATIONS ({}) reached in quantifier",
                                            MAX_ITERATIONS
                                        ),
                                    );
                            }
                            let result = ParseContent::Quantified(results, "*");
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_1",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    let result = ParseContent::Sequence(sequence_elements);
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "accessor_base",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "accessor_base", start_pos, node.span.end, consumed, & self
                                    .input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "accessor_base", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "accessor_base", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "accessor_base", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_positional_reference(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self
            .recursion_guard
            .check_cycle("positional_reference", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "positional_reference", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "positional_reference", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "positional_reference", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("positional_reference", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_POSITIONAL_REFERENCE,
                |parser| {
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let matched_str = parser.match_string("$")?;
                            let result = ParseContent::Terminal(matched_str);
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_0",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_integer()?),
                            );
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_1",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    let result = ParseContent::Sequence(sequence_elements);
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "positional_reference",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "positional_reference", start_pos, node.span.end, consumed,
                                    & self.input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "positional_reference", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "positional_reference", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "positional_reference", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_string_literal(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("string_literal", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "string_literal", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "string_literal", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "string_literal", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("string_literal", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_STRING_LITERAL,
                |parser| {
                    let parse_start = parser.position;
                    let mut best_content: Option<ParseContent<'input>> = None;
                    let mut best_end = parse_start;
                    let mut best_priority: i64 = i64::MIN;
                    let mut best_branch_index: usize = 0usize;
                    let mut best_branch = 0usize;
                    let mut nonassoc_tie = false;
                    let mut result = ParseContent::Sequence(Vec::new());
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            1usize, 2usize, "string_literal", parser.position
                                        ),
                                    );
                            }
                            let mut sequence_elements = Vec::with_capacity(3usize);
                            {
                                let element_start = parser.position;
                                let element_content = {
                                    let matched_str = parser.match_string("\"")?;
                                    let result = ParseContent::Terminal(matched_str);
                                    result
                                };
                                let element_end = parser.position;
                                sequence_elements
                                    .push(ParseNode {
                                        rule_name: "element_0",
                                        content: element_content,
                                        span: element_start..element_end,
                                    });
                            }
                            {
                                let element_start = parser.position;
                                let element_content = {
                                    let result = ParseContent::Alternative(
                                        Box::new(parser.parse_string_content_double()?),
                                    );
                                    result
                                };
                                let element_end = parser.position;
                                sequence_elements
                                    .push(ParseNode {
                                        rule_name: "element_1",
                                        content: element_content,
                                        span: element_start..element_end,
                                    });
                            }
                            {
                                let element_start = parser.position;
                                let element_content = {
                                    let matched_str = parser.match_string("\"")?;
                                    let result = ParseContent::Terminal(matched_str);
                                    result
                                };
                                let element_end = parser.position;
                                sequence_elements
                                    .push(ParseNode {
                                        rule_name: "element_2",
                                        content: element_content,
                                        span: element_start..element_end,
                                    });
                            }
                            let result = ParseContent::Sequence(sequence_elements);
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            1usize, 2usize, "string_literal", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 0usize > best_branch_index,
                                "nonassoc" => {
                                    if 0usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 0usize;
                            best_branch = 1usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    1usize, 2usize, "string_literal", parser.position
                                ),
                            );
                    }
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            2usize, 2usize, "string_literal", parser.position
                                        ),
                                    );
                            }
                            let mut sequence_elements = Vec::with_capacity(3usize);
                            {
                                let element_start = parser.position;
                                let element_content = {
                                    let matched_str = parser.match_string("'")?;
                                    let result = ParseContent::Terminal(matched_str);
                                    result
                                };
                                let element_end = parser.position;
                                sequence_elements
                                    .push(ParseNode {
                                        rule_name: "element_0",
                                        content: element_content,
                                        span: element_start..element_end,
                                    });
                            }
                            {
                                let element_start = parser.position;
                                let element_content = {
                                    let result = ParseContent::Alternative(
                                        Box::new(parser.parse_string_content_single()?),
                                    );
                                    result
                                };
                                let element_end = parser.position;
                                sequence_elements
                                    .push(ParseNode {
                                        rule_name: "element_1",
                                        content: element_content,
                                        span: element_start..element_end,
                                    });
                            }
                            {
                                let element_start = parser.position;
                                let element_content = {
                                    let matched_str = parser.match_string("'")?;
                                    let result = ParseContent::Terminal(matched_str);
                                    result
                                };
                                let element_end = parser.position;
                                sequence_elements
                                    .push(ParseNode {
                                        rule_name: "element_2",
                                        content: element_content,
                                        span: element_start..element_end,
                                    });
                            }
                            let result = ParseContent::Sequence(sequence_elements);
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            2usize, 2usize, "string_literal", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 1usize > best_branch_index,
                                "nonassoc" => {
                                    if 1usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 1usize;
                            best_branch = 2usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    2usize, 2usize, "string_literal", parser.position
                                ),
                            );
                    }
                    if nonassoc_tie {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    } else if let Some(content) = best_content {
                        parser.position = best_end;
                        if parser.logger.is_enabled() {
                            parser
                                .logger
                                .log_info(
                                    "generated/return_annotation_parser.rs",
                                    0,
                                    &format!(
                                        "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={})",
                                        "string_literal", best_branch, 2usize, best_end
                                        .saturating_sub(parse_start), best_priority, "left"
                                    ),
                                );
                        }
                        result = content;
                    } else {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    };
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "string_literal",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "string_literal", start_pos, node.span.end, consumed, & self
                                    .input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "string_literal", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "string_literal", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "string_literal", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_string_content_double(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self
            .recursion_guard
            .check_cycle("string_content_double", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "string_content_double", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "string_content_double", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "string_content_double", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("string_content_double", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_STRING_CONTENT_DOUBLE,
                |parser| {
                    let matched_str = parser.match_regex("[^\"]*", false)?;
                    let result = ParseContent::Terminal(matched_str);
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "string_content_double",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "string_content_double", start_pos, node.span.end, consumed,
                                    & self.input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "string_content_double", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "string_content_double", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "string_content_double", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_string_content_single(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self
            .recursion_guard
            .check_cycle("string_content_single", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "string_content_single", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "string_content_single", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "string_content_single", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("string_content_single", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_STRING_CONTENT_SINGLE,
                |parser| {
                    let matched_str = parser.match_regex("[^']*", false)?;
                    let result = ParseContent::Terminal(matched_str);
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "string_content_single",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "string_content_single", start_pos, node.span.end, consumed,
                                    & self.input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "string_content_single", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "string_content_single", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "string_content_single", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_number_literal(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("number_literal", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "number_literal", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "number_literal", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "number_literal", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("number_literal", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_NUMBER_LITERAL,
                |parser| {
                    let parse_start = parser.position;
                    let mut best_content: Option<ParseContent<'input>> = None;
                    let mut best_end = parse_start;
                    let mut best_priority: i64 = i64::MIN;
                    let mut best_branch_index: usize = 0usize;
                    let mut best_branch = 0usize;
                    let mut nonassoc_tie = false;
                    let mut result = ParseContent::Sequence(Vec::new());
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            1usize, 2usize, "number_literal", parser.position
                                        ),
                                    );
                            }
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_float()?),
                            );
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            1usize, 2usize, "number_literal", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 0usize > best_branch_index,
                                "nonassoc" => {
                                    if 0usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 0usize;
                            best_branch = 1usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    1usize, 2usize, "number_literal", parser.position
                                ),
                            );
                    }
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            2usize, 2usize, "number_literal", parser.position
                                        ),
                                    );
                            }
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_integer()?),
                            );
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            2usize, 2usize, "number_literal", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 1usize > best_branch_index,
                                "nonassoc" => {
                                    if 1usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 1usize;
                            best_branch = 2usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    2usize, 2usize, "number_literal", parser.position
                                ),
                            );
                    }
                    if nonassoc_tie {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    } else if let Some(content) = best_content {
                        parser.position = best_end;
                        if parser.logger.is_enabled() {
                            parser
                                .logger
                                .log_info(
                                    "generated/return_annotation_parser.rs",
                                    0,
                                    &format!(
                                        "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={})",
                                        "number_literal", best_branch, 2usize, best_end
                                        .saturating_sub(parse_start), best_priority, "left"
                                    ),
                                );
                        }
                        result = content;
                    } else {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    };
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "number_literal",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "number_literal", start_pos, node.span.end, consumed, & self
                                    .input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "number_literal", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "number_literal", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "number_literal", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_float(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("float", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "float", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "float", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "float", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("float", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_FLOAT,
                |parser| {
                    let matched_str = parser
                        .match_regex("[-+]?[0-9]+\\.[0-9]+(?:[eE][-+]?[0-9]+)?", true)?;
                    let result = ParseContent::Terminal(matched_str);
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "float",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "float", start_pos, node.span.end, consumed, & self
                                    .input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "float", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "float", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "float", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_integer(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("integer", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "integer", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "integer", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "integer", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("integer", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_INTEGER,
                |parser| {
                    let matched_str = parser.match_regex("[-+]?[0-9]+", true)?;
                    let result = ParseContent::Terminal(matched_str);
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "integer",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "integer", start_pos, node.span.end, consumed, & self
                                    .input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "integer", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "integer", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "integer", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_boolean_literal(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("boolean_literal", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "boolean_literal", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "boolean_literal", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "boolean_literal", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("boolean_literal", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_BOOLEAN_LITERAL,
                |parser| {
                    let parse_start = parser.position;
                    let mut best_content: Option<ParseContent<'input>> = None;
                    let mut best_end = parse_start;
                    let mut best_priority: i64 = i64::MIN;
                    let mut best_branch_index: usize = 0usize;
                    let mut best_branch = 0usize;
                    let mut nonassoc_tie = false;
                    let mut result = ParseContent::Sequence(Vec::new());
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            1usize, 2usize, "boolean_literal", parser.position
                                        ),
                                    );
                            }
                            let matched_str = parser.match_string("true")?;
                            let result = ParseContent::Terminal(matched_str);
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            1usize, 2usize, "boolean_literal", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 0usize > best_branch_index,
                                "nonassoc" => {
                                    if 0usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 0usize;
                            best_branch = 1usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    1usize, 2usize, "boolean_literal", parser.position
                                ),
                            );
                    }
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            2usize, 2usize, "boolean_literal", parser.position
                                        ),
                                    );
                            }
                            let matched_str = parser.match_string("false")?;
                            let result = ParseContent::Terminal(matched_str);
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            2usize, 2usize, "boolean_literal", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 1usize > best_branch_index,
                                "nonassoc" => {
                                    if 1usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 1usize;
                            best_branch = 2usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    2usize, 2usize, "boolean_literal", parser.position
                                ),
                            );
                    }
                    if nonassoc_tie {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    } else if let Some(content) = best_content {
                        parser.position = best_end;
                        if parser.logger.is_enabled() {
                            parser
                                .logger
                                .log_info(
                                    "generated/return_annotation_parser.rs",
                                    0,
                                    &format!(
                                        "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={})",
                                        "boolean_literal", best_branch, 2usize, best_end
                                        .saturating_sub(parse_start), best_priority, "left"
                                    ),
                                );
                        }
                        result = content;
                    } else {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    };
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "boolean_literal",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "boolean_literal", start_pos, node.span.end, consumed, &
                                    self.input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "boolean_literal", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "boolean_literal", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "boolean_literal", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_identifier(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("identifier", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "identifier", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "identifier", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "identifier", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("identifier", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_IDENTIFIER,
                |parser| {
                    let matched_str = parser
                        .match_regex("[a-zA-Z_][a-zA-Z0-9_]*", true)?;
                    let result = ParseContent::Terminal(matched_str);
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "identifier",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "identifier", start_pos, node.span.end, consumed, & self
                                    .input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "identifier", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "identifier", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "identifier", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_object_literal(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("object_literal", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "object_literal", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "object_literal", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "object_literal", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("object_literal", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_OBJECT_LITERAL,
                |parser| {
                    let mut sequence_elements = Vec::with_capacity(3usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let matched_str = parser.match_string("{")?;
                            let result = ParseContent::Terminal(matched_str);
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_0",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    {
                        let element_start = parser.position;
                        let element_content = if let Some(content) = parser
                            .try_parse(|p| {
                                let parser = p;
                                let result = ParseContent::Alternative(
                                    Box::new(parser.parse_object_properties()?),
                                );
                                Ok(result)
                            })
                        {
                            content
                        } else {
                            ParseContent::Sequence(Vec::new())
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_1",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let matched_str = parser.match_string("}")?;
                            let result = ParseContent::Terminal(matched_str);
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_2",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    let result = ParseContent::Sequence(sequence_elements);
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "object_literal",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "object_literal", start_pos, node.span.end, consumed, & self
                                    .input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "object_literal", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "object_literal", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "object_literal", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_object_properties(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("object_properties", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "object_properties", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "object_properties", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "object_properties", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("object_properties", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_OBJECT_PROPERTIES,
                |parser| {
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_object_property()?),
                            );
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_0",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let mut results = Vec::new();
                            let mut last_position = parser.position;
                            let mut iteration_count = 0;
                            const MAX_ITERATIONS: usize = 10000;
                            while iteration_count < MAX_ITERATIONS {
                                if let Some(node) = parser
                                    .try_parse(|p| {
                                        let parser = p;
                                        let mut sequence_elements = Vec::with_capacity(2usize);
                                        {
                                            let element_start = parser.position;
                                            let element_content = {
                                                let matched_str = parser.match_string(",")?;
                                                let result = ParseContent::Terminal(matched_str);
                                                result
                                            };
                                            let element_end = parser.position;
                                            sequence_elements
                                                .push(ParseNode {
                                                    rule_name: "element_0",
                                                    content: element_content,
                                                    span: element_start..element_end,
                                                });
                                        }
                                        {
                                            let element_start = parser.position;
                                            let element_content = {
                                                let result = ParseContent::Alternative(
                                                    Box::new(parser.parse_object_property()?),
                                                );
                                                result
                                            };
                                            let element_end = parser.position;
                                            sequence_elements
                                                .push(ParseNode {
                                                    rule_name: "element_1",
                                                    content: element_content,
                                                    span: element_start..element_end,
                                                });
                                        }
                                        let result = ParseContent::Sequence(sequence_elements);
                                        Ok(ParseNode {
                                            rule_name: "quantified",
                                            content: result,
                                            span: 0..0,
                                        })
                                    })
                                {
                                    let current_position = parser.position;
                                    if current_position == last_position {
                                        if parser.logger.is_enabled() {
                                            parser
                                                .logger
                                                .log_warning(
                                                    "generated/return_annotation_parser.rs",
                                                    0,
                                                    &format!(
                                                        "⚠️ ZERO-LENGTH MATCH in quantifier: Breaking to prevent infinite loop at position {}",
                                                        current_position
                                                    ),
                                                );
                                        }
                                        break;
                                    }
                                    results.push(node);
                                    last_position = current_position;
                                    iteration_count += 1;
                                } else {
                                    break;
                                }
                            }
                            if iteration_count >= MAX_ITERATIONS
                                && parser.logger.is_enabled()
                            {
                                parser
                                    .logger
                                    .log_warning(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "⚠️ MAX ITERATIONS ({}) reached in quantifier",
                                            MAX_ITERATIONS
                                        ),
                                    );
                            }
                            let result = ParseContent::Quantified(results, "*");
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_1",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    let result = ParseContent::Sequence(sequence_elements);
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "object_properties",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "object_properties", start_pos, node.span.end, consumed, &
                                    self.input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "object_properties", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "object_properties", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "object_properties", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_object_property(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("object_property", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "object_property", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "object_property", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "object_property", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("object_property", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_OBJECT_PROPERTY,
                |parser| {
                    let mut sequence_elements = Vec::with_capacity(3usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_property_key()?),
                            );
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_0",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let matched_str = parser.match_string(":")?;
                            let result = ParseContent::Terminal(matched_str);
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_1",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_expression()?),
                            );
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_2",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    let result = ParseContent::Sequence(sequence_elements);
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "object_property",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "object_property", start_pos, node.span.end, consumed, &
                                    self.input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "object_property", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "object_property", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "object_property", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_property_key(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("property_key", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "property_key", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "property_key", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "property_key", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("property_key", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_PROPERTY_KEY,
                |parser| {
                    let parse_start = parser.position;
                    let mut best_content: Option<ParseContent<'input>> = None;
                    let mut best_end = parse_start;
                    let mut best_priority: i64 = i64::MIN;
                    let mut best_branch_index: usize = 0usize;
                    let mut best_branch = 0usize;
                    let mut nonassoc_tie = false;
                    let mut result = ParseContent::Sequence(Vec::new());
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            1usize, 2usize, "property_key", parser.position
                                        ),
                                    );
                            }
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_identifier()?),
                            );
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            1usize, 2usize, "property_key", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 0usize > best_branch_index,
                                "nonassoc" => {
                                    if 0usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 0usize;
                            best_branch = 1usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    1usize, 2usize, "property_key", parser.position
                                ),
                            );
                    }
                    parser.position = parse_start;
                    if let Some(content) = parser
                        .try_parse(|p| {
                            let parser = p;
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                            2usize, 2usize, "property_key", parser.position
                                        ),
                                    );
                            }
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_string_literal()?),
                            );
                            if parser.logger.is_enabled() {
                                parser
                                    .logger
                                    .log_info(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                            2usize, 2usize, "property_key", parser.position
                                        ),
                                    );
                            }
                            Ok(result)
                        })
                    {
                        let candidate_end = parser.position;
                        parser.position = parse_start;
                        let candidate_priority: i64 = 0i64;
                        let transformed = {
                            let content = content;
                            content
                        };
                        let should_take = if best_content.is_none() {
                            true
                        } else if candidate_end > best_end {
                            true
                        } else if candidate_end < best_end {
                            false
                        } else if candidate_priority > best_priority {
                            true
                        } else if candidate_priority < best_priority {
                            false
                        } else {
                            match "left" {
                                "right" => 1usize > best_branch_index,
                                "nonassoc" => {
                                    if 1usize != best_branch_index {
                                        nonassoc_tie = true;
                                    }
                                    false
                                }
                                _ => false,
                            }
                        };
                        if should_take {
                            best_end = candidate_end;
                            best_priority = candidate_priority;
                            best_branch_index = 1usize;
                            best_branch = 2usize;
                            best_content = Some(transformed);
                        }
                    } else if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_info(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                    2usize, 2usize, "property_key", parser.position
                                ),
                            );
                    }
                    if nonassoc_tie {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    } else if let Some(content) = best_content {
                        parser.position = best_end;
                        if parser.logger.is_enabled() {
                            parser
                                .logger
                                .log_info(
                                    "generated/return_annotation_parser.rs",
                                    0,
                                    &format!(
                                        "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={})",
                                        "property_key", best_branch, 2usize, best_end
                                        .saturating_sub(parse_start), best_priority, "left"
                                    ),
                                );
                        }
                        result = content;
                    } else {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    };
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "property_key",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "property_key", start_pos, node.span.end, consumed, & self
                                    .input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "property_key", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "property_key", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "property_key", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_array_literal(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("array_literal", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "array_literal", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "array_literal", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "array_literal", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("array_literal", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_ARRAY_LITERAL,
                |parser| {
                    let mut sequence_elements = Vec::with_capacity(3usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let matched_str = parser.match_string("[")?;
                            let result = ParseContent::Terminal(matched_str);
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_0",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    {
                        let element_start = parser.position;
                        let element_content = if let Some(content) = parser
                            .try_parse(|p| {
                                let parser = p;
                                let result = ParseContent::Alternative(
                                    Box::new(parser.parse_array_elements()?),
                                );
                                Ok(result)
                            })
                        {
                            content
                        } else {
                            ParseContent::Sequence(Vec::new())
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_1",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let matched_str = parser.match_string("]")?;
                            let result = ParseContent::Terminal(matched_str);
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_2",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    let result = ParseContent::Sequence(sequence_elements);
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "array_literal",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "array_literal", start_pos, node.span.end, consumed, & self
                                    .input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "array_literal", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "array_literal", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "array_literal", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_array_elements(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("array_elements", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "array_elements", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "array_elements", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "array_elements", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("array_elements", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_ARRAY_ELEMENTS,
                |parser| {
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_array_element()?),
                            );
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_0",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let mut results = Vec::new();
                            let mut last_position = parser.position;
                            let mut iteration_count = 0;
                            const MAX_ITERATIONS: usize = 10000;
                            while iteration_count < MAX_ITERATIONS {
                                if let Some(node) = parser
                                    .try_parse(|p| {
                                        let parser = p;
                                        let mut sequence_elements = Vec::with_capacity(2usize);
                                        {
                                            let element_start = parser.position;
                                            let element_content = {
                                                let matched_str = parser.match_string(",")?;
                                                let result = ParseContent::Terminal(matched_str);
                                                result
                                            };
                                            let element_end = parser.position;
                                            sequence_elements
                                                .push(ParseNode {
                                                    rule_name: "element_0",
                                                    content: element_content,
                                                    span: element_start..element_end,
                                                });
                                        }
                                        {
                                            let element_start = parser.position;
                                            let element_content = {
                                                let result = ParseContent::Alternative(
                                                    Box::new(parser.parse_array_element()?),
                                                );
                                                result
                                            };
                                            let element_end = parser.position;
                                            sequence_elements
                                                .push(ParseNode {
                                                    rule_name: "element_1",
                                                    content: element_content,
                                                    span: element_start..element_end,
                                                });
                                        }
                                        let result = ParseContent::Sequence(sequence_elements);
                                        Ok(ParseNode {
                                            rule_name: "quantified",
                                            content: result,
                                            span: 0..0,
                                        })
                                    })
                                {
                                    let current_position = parser.position;
                                    if current_position == last_position {
                                        if parser.logger.is_enabled() {
                                            parser
                                                .logger
                                                .log_warning(
                                                    "generated/return_annotation_parser.rs",
                                                    0,
                                                    &format!(
                                                        "⚠️ ZERO-LENGTH MATCH in quantifier: Breaking to prevent infinite loop at position {}",
                                                        current_position
                                                    ),
                                                );
                                        }
                                        break;
                                    }
                                    results.push(node);
                                    last_position = current_position;
                                    iteration_count += 1;
                                } else {
                                    break;
                                }
                            }
                            if iteration_count >= MAX_ITERATIONS
                                && parser.logger.is_enabled()
                            {
                                parser
                                    .logger
                                    .log_warning(
                                        "generated/return_annotation_parser.rs",
                                        0,
                                        &format!(
                                            "⚠️ MAX ITERATIONS ({}) reached in quantifier",
                                            MAX_ITERATIONS
                                        ),
                                    );
                            }
                            let result = ParseContent::Quantified(results, "*");
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_1",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    let result = ParseContent::Sequence(sequence_elements);
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "array_elements",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "array_elements", start_pos, node.span.end, consumed, & self
                                    .input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "array_elements", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "array_elements", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "array_elements", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_array_element(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("array_element", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "array_element", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "array_element", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "array_element", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("array_element", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_ARRAY_ELEMENT,
                |parser| {
                    let result = ParseContent::Alternative(
                        Box::new(parser.parse_expression()?),
                    );
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "array_element",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "array_element", start_pos, node.span.end, consumed, & self
                                    .input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "array_element", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "array_element", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "array_element", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_parenthesized(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("parenthesized", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "parenthesized", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "parenthesized", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 100 => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "parenthesized", position, depth
                            ),
                        );
                }
                return Err(ParseError::RecursionDepthExceeded {
                    position,
                    depth,
                });
            }
            _ => {}
        }
        self.recursion_guard.enter("parenthesized", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_PARENTHESIZED,
                |parser| {
                    let mut sequence_elements = Vec::with_capacity(3usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let matched_str = parser.match_string("(")?;
                            let result = ParseContent::Terminal(matched_str);
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_0",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_expression()?),
                            );
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_1",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let matched_str = parser.match_string(")")?;
                            let result = ParseContent::Terminal(matched_str);
                            result
                        };
                        let element_end = parser.position;
                        sequence_elements
                            .push(ParseNode {
                                rule_name: "element_2",
                                content: element_content,
                                span: element_start..element_end,
                            });
                    }
                    let result = ParseContent::Sequence(sequence_elements);
                    let end_pos = parser.position;
                    Ok(ParseNode {
                        rule_name: "parenthesized",
                        content: result,
                        span: start_pos..end_pos,
                    })
                },
            );
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger.is_enabled() {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        self.logger
                            .log_success(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} chars: '{}')",
                                    "parenthesized", start_pos, node.span.end, consumed, & self
                                    .input[start_pos..node.span.end]
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "parenthesized", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "parenthesized", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "parenthesized", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    fn consume_optional_whitespace(&mut self) {
        while self.position < self.input.len() {
            let b = self.input.as_bytes()[self.position];
            if matches!(b, b' ' | b'\t' | b'\n' | b'\r') {
                self.position += 1;
            } else {
                break;
            }
        }
    }
    fn match_string(&mut self, expected: &str) -> ParseResult<&'input str> {
        self.consume_optional_whitespace();
        let start = self.position;
        let end = start + expected.len();
        if self.logger.is_enabled() {
            self.logger
                .log_debug(
                    "generated/return_annotation_parser.rs",
                    0,
                    &format!(
                        "🔤 Attempting to match terminal '{}' at position {} (end: {})",
                        expected, start, end
                    ),
                );
        }
        if end <= self.input.len() {
            let slice = &self.input[start..end];
            if slice == expected {
                self.position = end;
                if self.logger.is_enabled() {
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Terminal '{}' matched, advanced to position {}",
                                expected, end
                            ),
                        );
                }
                return Ok(slice);
            }
        }
        let found_str = if self.position < self.input.len() {
            let end = (self.position + expected.len()).min(self.input.len());
            &self.input[self.position..end]
        } else {
            "<EOF>"
        };
        if self.logger.is_enabled() {
            self.logger
                .log_error(
                    "generated/return_annotation_parser.rs",
                    0,
                    &format!(
                        "❌ Terminal '{}' failed at position {} - found '{}'", expected,
                        start, found_str
                    ),
                );
        }
        Err(
            self
                .create_contextual_error(
                    &format!("Expected '{}' but found '{}'", expected, found_str),
                ),
        )
    }
    fn match_regex(
        &mut self,
        pattern: &str,
        skip_leading_whitespace: bool,
    ) -> ParseResult<&'input str> {
        if skip_leading_whitespace {
            self.consume_optional_whitespace();
        }
        let re = regex::Regex::new(pattern)
            .map_err(|e| {
                self
                    .create_contextual_error(
                        &format!("Invalid regex pattern '{}': {}", pattern, e),
                    )
            })?;
        if let Some(mat) = re.find(&self.input[self.position..]) {
            if mat.start() == 0 {
                let matched = mat.as_str();
                let start = self.position;
                if self.logger.is_enabled() {
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Regex '{}' matched '{}' at position {}", pattern,
                                matched, start
                            ),
                        );
                }
                self.position += matched.len();
                return Ok(&self.input[start..self.position]);
            }
        }
        if self.logger.is_enabled() {
            let preview = if self.position < self.input.len() {
                let end = (self.position + 10).min(self.input.len());
                &self.input[self.position..end]
            } else {
                "<EOF>"
            };
            self.logger
                .log_error(
                    "generated/return_annotation_parser.rs",
                    0,
                    &format!(
                        "❌ Regex '{}' no match at position {} (next: '{}')", pattern,
                        self.position, preview
                    ),
                );
        }
        Err(
            self
                .create_contextual_error(
                    &format!("No match for regex pattern '{}'", pattern),
                ),
        )
    }
    fn try_parse<F, T>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&mut Self) -> ParseResult<T>,
    {
        let saved_pos = self.position;
        let saved_stack_len = self.recursion_guard.parse_stack.len();
        if self.logger.is_enabled() {
            self.logger
                .log_debug(
                    "generated/return_annotation_parser.rs",
                    0,
                    &format!("🔄 Starting speculative parse at position {}", saved_pos),
                );
        }
        match f(self) {
            Ok(result) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_success(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔄 Speculative parse succeeded, advanced to position {}",
                                self.position
                            ),
                        );
                }
                Some(result)
            }
            Err(e) => {
                self.position = saved_pos;
                self.recursion_guard.parse_stack.truncate(saved_stack_len);
                if self.logger.is_enabled() {
                    self.logger
                        .log_warning(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🔙 Speculative parse failed with error '{:?}', backtracked to position {}",
                                e, saved_pos
                            ),
                        );
                }
                None
            }
        }
    }
    fn memoized_call<F>(
        &mut self,
        rule_id: RuleId,
        f: F,
    ) -> ParseResult<ParseNode<'input>>
    where
        F: FnOnce(&mut Self) -> ParseResult<ParseNode<'input>>,
    {
        let key = (rule_id, self.position);
        if let Some(cached) = self.memo.get(&key) {
            if let Some(node) = cached {
                self.position = node.span.end;
                if self.logger.is_enabled() {
                    self.logger
                        .log_info(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💾 Memo hit for rule {} at position {} - reusing cached result",
                                rule_id, self.position
                            ),
                        );
                }
                return Ok(node.clone());
            } else {
                if self.logger.is_enabled() {
                    self.logger
                        .log_warning(
                            "generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "💾 Memo miss for rule {} at position {} - cached failure",
                                rule_id, self.position
                            ),
                        );
                }
                return Err(ParseError::Backtrack {
                    position: self.position,
                });
            }
        }
        if self.logger.is_enabled() {
            self.logger
                .log_debug(
                    "generated/return_annotation_parser.rs",
                    0,
                    &format!(
                        "💾 Memo miss for rule {} at position {} - computing fresh result",
                        rule_id, self.position
                    ),
                );
        }
        let result = f(self);
        if let Ok(ref node) = result {
            self.memo.insert(key, Some(node.clone()));
            if self.logger.is_enabled() {
                self.logger
                    .log_info(
                        "generated/return_annotation_parser.rs",
                        0,
                        &format!(
                            "💾 Memoized successful result for rule {} at position {}",
                            rule_id, self.position
                        ),
                    );
            }
        } else {
            self.memo.insert(key, None);
            if self.logger.is_enabled() {
                self.logger
                    .log_warning(
                        "generated/return_annotation_parser.rs",
                        0,
                        &format!(
                            "💾 Memoized failed result for rule {} at position {}",
                            rule_id, self.position
                        ),
                    );
            }
        }
        result
    }
    fn create_contextual_error(&self, message: &str) -> ParseError {
        let position = self.position;
        let rule_stack: Vec<String> = self
            .recursion_guard
            .parse_stack
            .iter()
            .map(|(rule, _)| rule.clone())
            .collect();
        let start = position.saturating_sub(20);
        let end = (position + 20).min(self.input.len());
        let input_context = if start < end {
            self.input[start..end].to_string()
        } else {
            String::new()
        };
        ParseError::ContextualError {
            message: message.to_string(),
            position,
            rule_stack,
            input_context,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use super::Logger;
    #[test]
    fn test_basic_parsing() {
        let input = "$1";
        let logger = Box::new(crate::NoOpLogger);
        let mut parser = ReturnAnnotationParser::new(input, logger);
        let result = parser.parse();
        assert!(result.is_ok());
    }
}
