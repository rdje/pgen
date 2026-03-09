use std::collections::HashMap;
use std::ops::Range;
use regex::Regex;
use crate::ast_pipeline::{
    Logger, ParseResult, ParseError, ParseContent, ParseNode, MemoEntry, RuleId,
    CycleType, RecursionGuard,
};
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryMarkerKind {
    PanicUntil,
    Sync,
    EofFallback,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryEvent {
    pub rule_name: String,
    pub parse_start: usize,
    pub previous_position: usize,
    pub new_position: usize,
    pub marker_kind: RecoveryMarkerKind,
    pub marker_position: Option<usize>,
    pub marker_value: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageTargetEvent {
    pub rule_name: String,
    pub parse_start: usize,
    pub parse_end: usize,
    pub branch_index: Option<usize>,
    pub coverage_target_weight: u64,
    pub critical_path: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegativeCaseEvent {
    pub rule_name: String,
    pub parse_start: usize,
    pub failure_position: usize,
    pub negative: bool,
    pub error_kind: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeterministicPartitionEvent {
    pub rule_name: String,
    pub parse_start: usize,
    pub parse_end: usize,
    pub group_key: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeterministicPartitionRuntimeMode {
    AnnotationDriven,
    ForceEnabled,
    ForceDisabled,
}
/// High-performance parser with memoization and zero-copy parsing
pub struct ReturnAnnotationParser<'input> {
    input: &'input str,
    position: usize,
    memo: HashMap<(RuleId, usize), Option<ParseNode<'input>>>,
    recursion_guard: RecursionGuard,
    grammar_profile: Option<String>,
    recovery_events: Vec<RecoveryEvent>,
    recovery_counts: HashMap<String, usize>,
    recovery_parse_count: usize,
    recovery_global_count: usize,
    coverage_target_events: Vec<CoverageTargetEvent>,
    coverage_target_rule_hits: HashMap<String, usize>,
    coverage_target_branch_hits: HashMap<String, usize>,
    negative_case_events: Vec<NegativeCaseEvent>,
    negative_case_rule_hits: HashMap<String, usize>,
    deterministic_partition_events: Vec<DeterministicPartitionEvent>,
    deterministic_partition_rule_hits: HashMap<String, usize>,
    deterministic_partition_runtime_mode: DeterministicPartitionRuntimeMode,
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
            grammar_profile: None,
            recovery_events: Vec::new(),
            recovery_counts: HashMap::new(),
            recovery_parse_count: 0,
            recovery_global_count: 0,
            coverage_target_events: Vec::new(),
            coverage_target_rule_hits: HashMap::new(),
            coverage_target_branch_hits: HashMap::new(),
            negative_case_events: Vec::new(),
            negative_case_rule_hits: HashMap::new(),
            deterministic_partition_events: Vec::new(),
            deterministic_partition_rule_hits: HashMap::new(),
            deterministic_partition_runtime_mode: DeterministicPartitionRuntimeMode::AnnotationDriven,
            logger,
        }
    }
    pub fn parse(&mut self) -> ParseResult<ParseNode<'input>> {
        self.recovery_events.clear();
        self.recovery_counts.clear();
        self.recovery_parse_count = 0;
        self.coverage_target_events.clear();
        self.coverage_target_rule_hits.clear();
        self.coverage_target_branch_hits.clear();
        self.negative_case_events.clear();
        self.negative_case_rule_hits.clear();
        self.deterministic_partition_events.clear();
        self.deterministic_partition_rule_hits.clear();
        self.parse_return_annotation()
    }
    pub fn parse_full(&mut self) -> ParseResult<ParseNode<'input>> {
        let parsed = self.parse()?;
        self.consume_layout_for_terminal("<EOF>");
        if self.position == self.input.len() {
            Ok(parsed)
        } else {
            Err(ParseError::InvalidSyntax {
                message: "Parser did not consume full input",
                position: self.position,
            })
        }
    }
    pub fn parse_full_return_annotation(&mut self) -> ParseResult<ParseNode<'input>> {
        self.parse_full()
    }
    pub fn set_grammar_profile(&mut self, profile: Option<&str>) {
        self.grammar_profile = profile.map(|value| value.to_string());
    }
    pub fn grammar_profile(&self) -> Option<&str> {
        self.grammar_profile.as_deref()
    }
    pub fn recovery_events(&self) -> &[RecoveryEvent] {
        &self.recovery_events
    }
    pub fn take_recovery_events(&mut self) -> Vec<RecoveryEvent> {
        std::mem::take(&mut self.recovery_events)
    }
    pub fn recovery_event_count(&self) -> usize {
        self.recovery_events.len()
    }
    pub fn recovery_parse_count(&self) -> usize {
        self.recovery_parse_count
    }
    pub fn recovery_global_count(&self) -> usize {
        self.recovery_global_count
    }
    pub fn coverage_target_events(&self) -> &[CoverageTargetEvent] {
        &self.coverage_target_events
    }
    pub fn take_coverage_target_events(&mut self) -> Vec<CoverageTargetEvent> {
        std::mem::take(&mut self.coverage_target_events)
    }
    pub fn coverage_target_event_count(&self) -> usize {
        self.coverage_target_events.len()
    }
    pub fn coverage_target_rule_hits(&self) -> &HashMap<String, usize> {
        &self.coverage_target_rule_hits
    }
    pub fn coverage_target_branch_hits(&self) -> &HashMap<String, usize> {
        &self.coverage_target_branch_hits
    }
    pub fn negative_case_events(&self) -> &[NegativeCaseEvent] {
        &self.negative_case_events
    }
    pub fn take_negative_case_events(&mut self) -> Vec<NegativeCaseEvent> {
        std::mem::take(&mut self.negative_case_events)
    }
    pub fn negative_case_event_count(&self) -> usize {
        self.negative_case_events.len()
    }
    pub fn negative_case_rule_hits(&self) -> &HashMap<String, usize> {
        &self.negative_case_rule_hits
    }
    pub fn deterministic_partition_events(&self) -> &[DeterministicPartitionEvent] {
        &self.deterministic_partition_events
    }
    pub fn take_deterministic_partition_events(
        &mut self,
    ) -> Vec<DeterministicPartitionEvent> {
        std::mem::take(&mut self.deterministic_partition_events)
    }
    pub fn deterministic_partition_event_count(&self) -> usize {
        self.deterministic_partition_events.len()
    }
    pub fn deterministic_partition_rule_hits(&self) -> &HashMap<String, usize> {
        &self.deterministic_partition_rule_hits
    }
    pub fn deterministic_partition_runtime_mode(
        &self,
    ) -> DeterministicPartitionRuntimeMode {
        self.deterministic_partition_runtime_mode
    }
    pub fn set_deterministic_partition_runtime_mode(
        &mut self,
        mode: DeterministicPartitionRuntimeMode,
    ) {
        self.deterministic_partition_runtime_mode = mode;
    }
    pub fn parse_return_annotation(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("return_annotation", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let parse_start = parser.position;
                    let mut best_content: Option<ParseContent<'input>> = None;
                    let mut best_end = parse_start;
                    let mut best_priority: i64 = i64::MIN;
                    let mut best_branch_index: usize = 0usize;
                    let mut best_branch = 0usize;
                    let mut nonassoc_tie = false;
                    let mut result = ParseContent::Sequence(Vec::new());
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "return_annotation",
                            "rule.return_annotation",
                        );
                    let deterministic_partition_offset = if deterministic_partition_effective_enabled {
                        parser
                            .deterministic_partition_offset_runtime(
                                &deterministic_partition_effective_group,
                                3usize,
                            )
                    } else {
                        0usize
                    };
                    let mut evaluation_order: Vec<usize> = (0..3usize).collect();
                    if deterministic_partition_effective_enabled && 3usize > 1
                        && deterministic_partition_offset > 0
                    {
                        evaluation_order.rotate_left(deterministic_partition_offset);
                    }
                    for branch_index in evaluation_order {
                        match branch_index {
                            0usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 0usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 1usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    1usize, 3usize, "return_annotation", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            1usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 1usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 2usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    2usize, 3usize, "return_annotation", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            2usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 2usize;
                                        let transformed = {
                                            let content = content;
                                            {
                                                let __pgen_base = (content).clone();
                                                match __pgen_base {
                                                    ParseContent::Sequence(
                                                        elements,
                                                    ) if elements.len() > 1usize => {
                                                        elements[1usize].content.clone()
                                                    }
                                                    ParseContent::Quantified(
                                                        elements,
                                                        _,
                                                    ) if elements.len() > 1usize => {
                                                        elements[1usize].content.clone()
                                                    }
                                                    _ => ParseContent::Terminal("<invalid_sequence_access>"),
                                                }
                                            }
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 3usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    3usize, 3usize, "return_annotation", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    if nonassoc_tie {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    } else if let Some(content) = best_content {
                        parser.position = best_end;
                        semantic_selected_branch_index = Some(best_branch);
                        if parser.logger.is_enabled() {
                            parser
                                .logger
                                .log_info(
                                    "../generated/return_annotation_parser.rs",
                                    0,
                                    &format!(
                                        "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                        "return_annotation", best_branch, 3usize, best_end
                                        .saturating_sub(parse_start), best_priority, "left",
                                        "longest_match"
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
                    parser
                        .record_coverage_target_event(
                            "return_annotation",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "return_annotation",
                            "rule.return_annotation",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "return_annotation",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "return_annotation", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "return_annotation", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "return_annotation", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "return_annotation",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("arrow", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let matched_str = parser.match_string("->")?;
                    let result = ParseContent::Terminal(matched_str);
                    let end_pos = parser.position;
                    parser
                        .record_coverage_target_event(
                            "arrow",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group("arrow", "rule.arrow");
                    parser
                        .record_deterministic_partition_event(
                            "arrow",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "arrow", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "arrow", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "arrow", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "arrow",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("expression", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let parse_start = parser.position;
                    let mut best_content: Option<ParseContent<'input>> = None;
                    let mut best_end = parse_start;
                    let mut best_priority: i64 = i64::MIN;
                    let mut best_branch_index: usize = 0usize;
                    let mut best_branch = 0usize;
                    let mut nonassoc_tie = false;
                    let mut result = ParseContent::Sequence(Vec::new());
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "expression",
                            "rule.expression",
                        );
                    let deterministic_partition_offset = if deterministic_partition_effective_enabled {
                        parser
                            .deterministic_partition_offset_runtime(
                                &deterministic_partition_effective_group,
                                5usize,
                            )
                    } else {
                        0usize
                    };
                    let mut evaluation_order: Vec<usize> = (0..5usize).collect();
                    if deterministic_partition_effective_enabled && 5usize > 1
                        && deterministic_partition_offset > 0
                    {
                        evaluation_order.rotate_left(deterministic_partition_offset);
                    }
                    for branch_index in evaluation_order {
                        match branch_index {
                            0usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 0usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 1usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    1usize, 5usize, "expression", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            1usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 1usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 2usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    2usize, 5usize, "expression", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            2usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 2usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 3usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    3usize, 5usize, "expression", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            3usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 3usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 4usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    4usize, 5usize, "expression", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            4usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 4usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 5usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    5usize, 5usize, "expression", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    if nonassoc_tie {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    } else if let Some(content) = best_content {
                        parser.position = best_end;
                        semantic_selected_branch_index = Some(best_branch);
                        if parser.logger.is_enabled() {
                            parser
                                .logger
                                .log_info(
                                    "../generated/return_annotation_parser.rs",
                                    0,
                                    &format!(
                                        "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                        "expression", best_branch, 5usize, best_end
                                        .saturating_sub(parse_start), best_priority, "left",
                                        "longest_match"
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
                    parser
                        .record_coverage_target_event(
                            "expression",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "expression",
                            "rule.expression",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "expression",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "expression", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "expression", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "expression", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "expression",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self
            .recursion_guard
            .check_cycle("primary_expression", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let parse_start = parser.position;
                    let mut best_content: Option<ParseContent<'input>> = None;
                    let mut best_end = parse_start;
                    let mut best_priority: i64 = i64::MIN;
                    let mut best_branch_index: usize = 0usize;
                    let mut best_branch = 0usize;
                    let mut nonassoc_tie = false;
                    let mut result = ParseContent::Sequence(Vec::new());
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "primary_expression",
                            "rule.primary_expression",
                        );
                    let deterministic_partition_offset = if deterministic_partition_effective_enabled {
                        parser
                            .deterministic_partition_offset_runtime(
                                &deterministic_partition_effective_group,
                                8usize,
                            )
                    } else {
                        0usize
                    };
                    let mut evaluation_order: Vec<usize> = (0..8usize).collect();
                    if deterministic_partition_effective_enabled && 8usize > 1
                        && deterministic_partition_offset > 0
                    {
                        evaluation_order.rotate_left(deterministic_partition_offset);
                    }
                    for branch_index in evaluation_order {
                        match branch_index {
                            0usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 0usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 1usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    1usize, 8usize, "primary_expression", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            1usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 1usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 2usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    2usize, 8usize, "primary_expression", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            2usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 2usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 3usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    3usize, 8usize, "primary_expression", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            3usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 3usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 4usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    4usize, 8usize, "primary_expression", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            4usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 4usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 5usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    5usize, 8usize, "primary_expression", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            5usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 5usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 6usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    6usize, 8usize, "primary_expression", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            6usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 6usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 7usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    7usize, 8usize, "primary_expression", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            7usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 7usize;
                                        let transformed = {
                                            let content = content;
                                            {
                                                let __pgen_base = (content).clone();
                                                match __pgen_base {
                                                    ParseContent::Sequence(
                                                        elements,
                                                    ) if elements.len() > 1usize => {
                                                        elements[1usize].content.clone()
                                                    }
                                                    ParseContent::Quantified(
                                                        elements,
                                                        _,
                                                    ) if elements.len() > 1usize => {
                                                        elements[1usize].content.clone()
                                                    }
                                                    _ => ParseContent::Terminal("<invalid_sequence_access>"),
                                                }
                                            }
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 8usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    8usize, 8usize, "primary_expression", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    if nonassoc_tie {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    } else if let Some(content) = best_content {
                        parser.position = best_end;
                        semantic_selected_branch_index = Some(best_branch);
                        if parser.logger.is_enabled() {
                            parser
                                .logger
                                .log_info(
                                    "../generated/return_annotation_parser.rs",
                                    0,
                                    &format!(
                                        "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                        "primary_expression", best_branch, 8usize, best_end
                                        .saturating_sub(parse_start), best_priority, "left",
                                        "longest_match"
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
                    parser
                        .record_coverage_target_event(
                            "primary_expression",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "primary_expression",
                            "rule.primary_expression",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "primary_expression",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "primary_expression", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "primary_expression", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "primary_expression", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "primary_expression",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self
            .recursion_guard
            .check_cycle("extraction_expression", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
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
                    parser
                        .record_coverage_target_event(
                            "extraction_expression",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "extraction_expression",
                            "rule.extraction_expression",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "extraction_expression",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "extraction_expression", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "extraction_expression", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "extraction_expression", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "extraction_expression",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("extraction_target", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let parse_start = parser.position;
                    let mut best_content: Option<ParseContent<'input>> = None;
                    let mut best_end = parse_start;
                    let mut best_priority: i64 = i64::MIN;
                    let mut best_branch_index: usize = 0usize;
                    let mut best_branch = 0usize;
                    let mut nonassoc_tie = false;
                    let mut result = ParseContent::Sequence(Vec::new());
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "extraction_target",
                            "rule.extraction_target",
                        );
                    let deterministic_partition_offset = if deterministic_partition_effective_enabled {
                        parser
                            .deterministic_partition_offset_runtime(
                                &deterministic_partition_effective_group,
                                3usize,
                            )
                    } else {
                        0usize
                    };
                    let mut evaluation_order: Vec<usize> = (0..3usize).collect();
                    if deterministic_partition_effective_enabled && 3usize > 1
                        && deterministic_partition_offset > 0
                    {
                        evaluation_order.rotate_left(deterministic_partition_offset);
                    }
                    for branch_index in evaluation_order {
                        match branch_index {
                            0usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 0usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 1usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    1usize, 3usize, "extraction_target", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            1usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 1usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 2usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    2usize, 3usize, "extraction_target", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            2usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 2usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 3usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    3usize, 3usize, "extraction_target", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    if nonassoc_tie {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    } else if let Some(content) = best_content {
                        parser.position = best_end;
                        semantic_selected_branch_index = Some(best_branch);
                        if parser.logger.is_enabled() {
                            parser
                                .logger
                                .log_info(
                                    "../generated/return_annotation_parser.rs",
                                    0,
                                    &format!(
                                        "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                        "extraction_target", best_branch, 3usize, best_end
                                        .saturating_sub(parse_start), best_priority, "left",
                                        "longest_match"
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
                    parser
                        .record_coverage_target_event(
                            "extraction_target",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "extraction_target",
                            "rule.extraction_target",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "extraction_target",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "extraction_target", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "extraction_target", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "extraction_target", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "extraction_target",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("positive_integer", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let matched_str = parser.match_regex("[1-9][0-9]*", true)?;
                    let transformed = matched_str.parse::<usize>().unwrap_or(1);
                    if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_debug(
                                file!(),
                                line!(),
                                &format!(
                                    "🎯 Applied semantic transform: parsed '{}' to {}={}",
                                    matched_str, stringify!(usize), transformed
                                ),
                            );
                    }
                    let result = ParseContent::TransformedTerminal(
                        transformed.to_string(),
                    );
                    let end_pos = parser.position;
                    parser
                        .record_coverage_target_event(
                            "positive_integer",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "positive_integer",
                            "rule.positive_integer",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "positive_integer",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "positive_integer", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "positive_integer", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "positive_integer", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "positive_integer",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("spread_expression", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
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
                    parser
                        .record_coverage_target_event(
                            "spread_expression",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "spread_expression",
                            "rule.spread_expression",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "spread_expression",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "spread_expression", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "spread_expression", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "spread_expression", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "spread_expression",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self
            .recursion_guard
            .check_cycle("spreadable_expression", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let parse_start = parser.position;
                    let mut best_content: Option<ParseContent<'input>> = None;
                    let mut best_end = parse_start;
                    let mut best_priority: i64 = i64::MIN;
                    let mut best_branch_index: usize = 0usize;
                    let mut best_branch = 0usize;
                    let mut nonassoc_tie = false;
                    let mut result = ParseContent::Sequence(Vec::new());
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "spreadable_expression",
                            "rule.spreadable_expression",
                        );
                    let deterministic_partition_offset = if deterministic_partition_effective_enabled {
                        parser
                            .deterministic_partition_offset_runtime(
                                &deterministic_partition_effective_group,
                                5usize,
                            )
                    } else {
                        0usize
                    };
                    let mut evaluation_order: Vec<usize> = (0..5usize).collect();
                    if deterministic_partition_effective_enabled && 5usize > 1
                        && deterministic_partition_offset > 0
                    {
                        evaluation_order.rotate_left(deterministic_partition_offset);
                    }
                    for branch_index in evaluation_order {
                        match branch_index {
                            0usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 0usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 1usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    1usize, 5usize, "spreadable_expression", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            1usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 1usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 2usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    2usize, 5usize, "spreadable_expression", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            2usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 2usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 3usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    3usize, 5usize, "spreadable_expression", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            3usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 3usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 4usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    4usize, 5usize, "spreadable_expression", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            4usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 4usize;
                                        let transformed = {
                                            let content = content;
                                            {
                                                let __pgen_base = (content).clone();
                                                match __pgen_base {
                                                    ParseContent::Sequence(
                                                        elements,
                                                    ) if elements.len() > 1usize => {
                                                        elements[1usize].content.clone()
                                                    }
                                                    ParseContent::Quantified(
                                                        elements,
                                                        _,
                                                    ) if elements.len() > 1usize => {
                                                        elements[1usize].content.clone()
                                                    }
                                                    _ => ParseContent::Terminal("<invalid_sequence_access>"),
                                                }
                                            }
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 5usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    5usize, 5usize, "spreadable_expression", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    if nonassoc_tie {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    } else if let Some(content) = best_content {
                        parser.position = best_end;
                        semantic_selected_branch_index = Some(best_branch);
                        if parser.logger.is_enabled() {
                            parser
                                .logger
                                .log_info(
                                    "../generated/return_annotation_parser.rs",
                                    0,
                                    &format!(
                                        "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                        "spreadable_expression", best_branch, 5usize, best_end
                                        .saturating_sub(parse_start), best_priority, "left",
                                        "longest_match"
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
                    parser
                        .record_coverage_target_event(
                            "spreadable_expression",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "spreadable_expression",
                            "rule.spreadable_expression",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "spreadable_expression",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "spreadable_expression", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "spreadable_expression", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "spreadable_expression", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "spreadable_expression",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("spread_suffix", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let matched_str = parser.match_string("*")?;
                    let result = ParseContent::Terminal(matched_str);
                    let end_pos = parser.position;
                    parser
                        .record_coverage_target_event(
                            "spread_suffix",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "spread_suffix",
                            "rule.spread_suffix",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "spread_suffix",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "spread_suffix", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "spread_suffix", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "spread_suffix", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "spread_suffix",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self
            .recursion_guard
            .check_cycle("property_access_expression", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
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
                                        let deterministic_partition_effective_enabled = parser
                                            .effective_deterministic_partition_enabled(false);
                                        let deterministic_partition_effective_group = parser
                                            .effective_deterministic_partition_group(
                                                "property_access_expression",
                                                "rule.property_access_expression",
                                            );
                                        let deterministic_partition_offset = if deterministic_partition_effective_enabled {
                                            parser
                                                .deterministic_partition_offset_runtime(
                                                    &deterministic_partition_effective_group,
                                                    2usize,
                                                )
                                        } else {
                                            0usize
                                        };
                                        let mut evaluation_order: Vec<usize> = (0..2usize)
                                            .collect();
                                        if deterministic_partition_effective_enabled && 2usize > 1
                                            && deterministic_partition_offset > 0
                                        {
                                            evaluation_order
                                                .rotate_left(deterministic_partition_offset);
                                        }
                                        for branch_index in evaluation_order {
                                            match branch_index {
                                                0usize => {
                                                    if "longest_match" == "ordered" && best_content.is_some()
                                                    {} else {
                                                        parser.position = parse_start;
                                                        if let Some(content) = parser
                                                            .try_parse(|p| {
                                                                let parser = p;
                                                                if parser.logger.is_enabled() {
                                                                    parser
                                                                        .logger
                                                                        .log_info(
                                                                            "../generated/return_annotation_parser.rs",
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
                                                                            "../generated/return_annotation_parser.rs",
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
                                                            let current_branch_index: usize = 0usize;
                                                            let transformed = {
                                                                let content = content;
                                                                {
                                                                    let mut json_obj = serde_json::json!({});
                                                                    json_obj["base"] = serde_json::json!(
                                                                        ({ let __pgen_content = { let __pgen_base = (content)
                                                                        .clone(); match __pgen_base {
                                                                        ParseContent::Sequence(elements) if ! elements.is_empty() =>
                                                                        { elements[0usize].content.clone() }
                                                                        ParseContent::Quantified(elements, _) if ! elements
                                                                        .is_empty() => { elements[0usize].content.clone() }
                                                                        ParseContent::Alternative(node) => node.content.clone(),
                                                                        other => other, } }; match __pgen_content {
                                                                        ParseContent::Terminal(s) => s.to_string(),
                                                                        ParseContent::TransformedTerminal(s) => s,
                                                                        ParseContent::Alternative(node) => { match node.content {
                                                                        ParseContent::Terminal(s) => s.to_string(),
                                                                        ParseContent::TransformedTerminal(s) => s, other =>
                                                                        format!("{:?}", other), } } other => format!("{:?}", other),
                                                                        } })
                                                                    );
                                                                    json_obj["property"] = serde_json::json!(
                                                                        ({ let __pgen_content = { let __pgen_base = (content)
                                                                        .clone(); match __pgen_base {
                                                                        ParseContent::Sequence(elements) if elements.len() > 2usize
                                                                        => { elements[2usize].content.clone() }
                                                                        ParseContent::Quantified(elements, _) if elements.len() >
                                                                        2usize => { elements[2usize].content.clone() } _ =>
                                                                        ParseContent::Terminal("<invalid_sequence_access>"), } };
                                                                        match __pgen_content { ParseContent::Terminal(s) => s
                                                                        .to_string(), ParseContent::TransformedTerminal(s) => s,
                                                                        ParseContent::Alternative(node) => { match node.content {
                                                                        ParseContent::Terminal(s) => s.to_string(),
                                                                        ParseContent::TransformedTerminal(s) => s, other =>
                                                                        format!("{:?}", other), } } other => format!("{:?}", other),
                                                                        } })
                                                                    );
                                                                    json_obj["type"] = serde_json::json!(("property_access"));
                                                                    let json_str = serde_json::to_string(&json_obj)
                                                                        .unwrap_or_else(|_| "{}".to_string());
                                                                    ParseContent::TransformedTerminal(json_str)
                                                                }
                                                            };
                                                            let should_take = if "longest_match" == "ordered" {
                                                                best_content.is_none()
                                                            } else if "longest_match" == "priority_first" {
                                                                if best_content.is_none() {
                                                                    true
                                                                } else if candidate_priority > best_priority {
                                                                    true
                                                                } else if candidate_priority < best_priority {
                                                                    false
                                                                } else if candidate_end > best_end {
                                                                    true
                                                                } else if candidate_end < best_end {
                                                                    false
                                                                } else {
                                                                    match "left" {
                                                                        "right" => current_branch_index > best_branch_index,
                                                                        "nonassoc" => {
                                                                            if current_branch_index != best_branch_index {
                                                                                nonassoc_tie = true;
                                                                            }
                                                                            false
                                                                        }
                                                                        _ => false,
                                                                    }
                                                                }
                                                            } else if best_content.is_none() {
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
                                                                    "right" => current_branch_index > best_branch_index,
                                                                    "nonassoc" => {
                                                                        if current_branch_index != best_branch_index {
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
                                                                best_branch_index = current_branch_index;
                                                                best_branch = 1usize;
                                                                best_content = Some(transformed);
                                                            }
                                                        } else if parser.logger.is_enabled() {
                                                            parser
                                                                .logger
                                                                .log_info(
                                                                    "../generated/return_annotation_parser.rs",
                                                                    0,
                                                                    &format!(
                                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                                        1usize, 2usize, "property_access_expression", parser
                                                                        .position
                                                                    ),
                                                                );
                                                        }
                                                    }
                                                }
                                                1usize => {
                                                    if "longest_match" == "ordered" && best_content.is_some()
                                                    {} else {
                                                        parser.position = parse_start;
                                                        if let Some(content) = parser
                                                            .try_parse(|p| {
                                                                let parser = p;
                                                                if parser.logger.is_enabled() {
                                                                    parser
                                                                        .logger
                                                                        .log_info(
                                                                            "../generated/return_annotation_parser.rs",
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
                                                                            "../generated/return_annotation_parser.rs",
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
                                                            let current_branch_index: usize = 1usize;
                                                            let transformed = {
                                                                let content = content;
                                                                content
                                                            };
                                                            let should_take = if "longest_match" == "ordered" {
                                                                best_content.is_none()
                                                            } else if "longest_match" == "priority_first" {
                                                                if best_content.is_none() {
                                                                    true
                                                                } else if candidate_priority > best_priority {
                                                                    true
                                                                } else if candidate_priority < best_priority {
                                                                    false
                                                                } else if candidate_end > best_end {
                                                                    true
                                                                } else if candidate_end < best_end {
                                                                    false
                                                                } else {
                                                                    match "left" {
                                                                        "right" => current_branch_index > best_branch_index,
                                                                        "nonassoc" => {
                                                                            if current_branch_index != best_branch_index {
                                                                                nonassoc_tie = true;
                                                                            }
                                                                            false
                                                                        }
                                                                        _ => false,
                                                                    }
                                                                }
                                                            } else if best_content.is_none() {
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
                                                                    "right" => current_branch_index > best_branch_index,
                                                                    "nonassoc" => {
                                                                        if current_branch_index != best_branch_index {
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
                                                                best_branch_index = current_branch_index;
                                                                best_branch = 2usize;
                                                                best_content = Some(transformed);
                                                            }
                                                        } else if parser.logger.is_enabled() {
                                                            parser
                                                                .logger
                                                                .log_info(
                                                                    "../generated/return_annotation_parser.rs",
                                                                    0,
                                                                    &format!(
                                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                                        2usize, 2usize, "property_access_expression", parser
                                                                        .position
                                                                    ),
                                                                );
                                                        }
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                        if nonassoc_tie {
                                            return Err(ParseError::Backtrack {
                                                position: parse_start,
                                            });
                                        } else if let Some(content) = best_content {
                                            parser.position = best_end;
                                            semantic_selected_branch_index = Some(best_branch);
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                                            "property_access_expression", best_branch, 2usize, best_end
                                                            .saturating_sub(parse_start), best_priority, "left",
                                                            "longest_match"
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
                                                    "../generated/return_annotation_parser.rs",
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
                                        "../generated/return_annotation_parser.rs",
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
                    parser
                        .record_coverage_target_event(
                            "property_access_expression",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "property_access_expression",
                            "rule.property_access_expression",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "property_access_expression",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "property_access_expression", start_pos, node.span.end,
                                    consumed, consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "property_access_expression", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "property_access_expression", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "property_access_expression",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self
            .recursion_guard
            .check_cycle("array_access_expression", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
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
                                        let deterministic_partition_effective_enabled = parser
                                            .effective_deterministic_partition_enabled(false);
                                        let deterministic_partition_effective_group = parser
                                            .effective_deterministic_partition_group(
                                                "array_access_expression",
                                                "rule.array_access_expression",
                                            );
                                        let deterministic_partition_offset = if deterministic_partition_effective_enabled {
                                            parser
                                                .deterministic_partition_offset_runtime(
                                                    &deterministic_partition_effective_group,
                                                    2usize,
                                                )
                                        } else {
                                            0usize
                                        };
                                        let mut evaluation_order: Vec<usize> = (0..2usize)
                                            .collect();
                                        if deterministic_partition_effective_enabled && 2usize > 1
                                            && deterministic_partition_offset > 0
                                        {
                                            evaluation_order
                                                .rotate_left(deterministic_partition_offset);
                                        }
                                        for branch_index in evaluation_order {
                                            match branch_index {
                                                0usize => {
                                                    if "longest_match" == "ordered" && best_content.is_some()
                                                    {} else {
                                                        parser.position = parse_start;
                                                        if let Some(content) = parser
                                                            .try_parse(|p| {
                                                                let parser = p;
                                                                if parser.logger.is_enabled() {
                                                                    parser
                                                                        .logger
                                                                        .log_info(
                                                                            "../generated/return_annotation_parser.rs",
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
                                                                            "../generated/return_annotation_parser.rs",
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
                                                            let current_branch_index: usize = 0usize;
                                                            let transformed = {
                                                                let content = content;
                                                                {
                                                                    let mut json_obj = serde_json::json!({});
                                                                    json_obj["base"] = serde_json::json!(
                                                                        ({ let __pgen_content = { let __pgen_base = (content)
                                                                        .clone(); match __pgen_base {
                                                                        ParseContent::Sequence(elements) if ! elements.is_empty() =>
                                                                        { elements[0usize].content.clone() }
                                                                        ParseContent::Quantified(elements, _) if ! elements
                                                                        .is_empty() => { elements[0usize].content.clone() }
                                                                        ParseContent::Alternative(node) => node.content.clone(),
                                                                        other => other, } }; match __pgen_content {
                                                                        ParseContent::Terminal(s) => s.to_string(),
                                                                        ParseContent::TransformedTerminal(s) => s,
                                                                        ParseContent::Alternative(node) => { match node.content {
                                                                        ParseContent::Terminal(s) => s.to_string(),
                                                                        ParseContent::TransformedTerminal(s) => s, other =>
                                                                        format!("{:?}", other), } } other => format!("{:?}", other),
                                                                        } })
                                                                    );
                                                                    json_obj["index"] = serde_json::json!(
                                                                        ({ let __pgen_content = { let __pgen_base = (content)
                                                                        .clone(); match __pgen_base {
                                                                        ParseContent::Sequence(elements) if elements.len() > 2usize
                                                                        => { elements[2usize].content.clone() }
                                                                        ParseContent::Quantified(elements, _) if elements.len() >
                                                                        2usize => { elements[2usize].content.clone() } _ =>
                                                                        ParseContent::Terminal("<invalid_sequence_access>"), } };
                                                                        match __pgen_content { ParseContent::Terminal(s) => s
                                                                        .to_string(), ParseContent::TransformedTerminal(s) => s,
                                                                        ParseContent::Alternative(node) => { match node.content {
                                                                        ParseContent::Terminal(s) => s.to_string(),
                                                                        ParseContent::TransformedTerminal(s) => s, other =>
                                                                        format!("{:?}", other), } } other => format!("{:?}", other),
                                                                        } })
                                                                    );
                                                                    json_obj["type"] = serde_json::json!(("array_access"));
                                                                    let json_str = serde_json::to_string(&json_obj)
                                                                        .unwrap_or_else(|_| "{}".to_string());
                                                                    ParseContent::TransformedTerminal(json_str)
                                                                }
                                                            };
                                                            let should_take = if "longest_match" == "ordered" {
                                                                best_content.is_none()
                                                            } else if "longest_match" == "priority_first" {
                                                                if best_content.is_none() {
                                                                    true
                                                                } else if candidate_priority > best_priority {
                                                                    true
                                                                } else if candidate_priority < best_priority {
                                                                    false
                                                                } else if candidate_end > best_end {
                                                                    true
                                                                } else if candidate_end < best_end {
                                                                    false
                                                                } else {
                                                                    match "left" {
                                                                        "right" => current_branch_index > best_branch_index,
                                                                        "nonassoc" => {
                                                                            if current_branch_index != best_branch_index {
                                                                                nonassoc_tie = true;
                                                                            }
                                                                            false
                                                                        }
                                                                        _ => false,
                                                                    }
                                                                }
                                                            } else if best_content.is_none() {
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
                                                                    "right" => current_branch_index > best_branch_index,
                                                                    "nonassoc" => {
                                                                        if current_branch_index != best_branch_index {
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
                                                                best_branch_index = current_branch_index;
                                                                best_branch = 1usize;
                                                                best_content = Some(transformed);
                                                            }
                                                        } else if parser.logger.is_enabled() {
                                                            parser
                                                                .logger
                                                                .log_info(
                                                                    "../generated/return_annotation_parser.rs",
                                                                    0,
                                                                    &format!(
                                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                                        1usize, 2usize, "array_access_expression", parser.position
                                                                    ),
                                                                );
                                                        }
                                                    }
                                                }
                                                1usize => {
                                                    if "longest_match" == "ordered" && best_content.is_some()
                                                    {} else {
                                                        parser.position = parse_start;
                                                        if let Some(content) = parser
                                                            .try_parse(|p| {
                                                                let parser = p;
                                                                if parser.logger.is_enabled() {
                                                                    parser
                                                                        .logger
                                                                        .log_info(
                                                                            "../generated/return_annotation_parser.rs",
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
                                                                            "../generated/return_annotation_parser.rs",
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
                                                            let current_branch_index: usize = 1usize;
                                                            let transformed = {
                                                                let content = content;
                                                                content
                                                            };
                                                            let should_take = if "longest_match" == "ordered" {
                                                                best_content.is_none()
                                                            } else if "longest_match" == "priority_first" {
                                                                if best_content.is_none() {
                                                                    true
                                                                } else if candidate_priority > best_priority {
                                                                    true
                                                                } else if candidate_priority < best_priority {
                                                                    false
                                                                } else if candidate_end > best_end {
                                                                    true
                                                                } else if candidate_end < best_end {
                                                                    false
                                                                } else {
                                                                    match "left" {
                                                                        "right" => current_branch_index > best_branch_index,
                                                                        "nonassoc" => {
                                                                            if current_branch_index != best_branch_index {
                                                                                nonassoc_tie = true;
                                                                            }
                                                                            false
                                                                        }
                                                                        _ => false,
                                                                    }
                                                                }
                                                            } else if best_content.is_none() {
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
                                                                    "right" => current_branch_index > best_branch_index,
                                                                    "nonassoc" => {
                                                                        if current_branch_index != best_branch_index {
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
                                                                best_branch_index = current_branch_index;
                                                                best_branch = 2usize;
                                                                best_content = Some(transformed);
                                                            }
                                                        } else if parser.logger.is_enabled() {
                                                            parser
                                                                .logger
                                                                .log_info(
                                                                    "../generated/return_annotation_parser.rs",
                                                                    0,
                                                                    &format!(
                                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                                        2usize, 2usize, "array_access_expression", parser.position
                                                                    ),
                                                                );
                                                        }
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                        if nonassoc_tie {
                                            return Err(ParseError::Backtrack {
                                                position: parse_start,
                                            });
                                        } else if let Some(content) = best_content {
                                            parser.position = best_end;
                                            semantic_selected_branch_index = Some(best_branch);
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                                            "array_access_expression", best_branch, 2usize, best_end
                                                            .saturating_sub(parse_start), best_priority, "left",
                                                            "longest_match"
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
                                                    "../generated/return_annotation_parser.rs",
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
                                        "../generated/return_annotation_parser.rs",
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
                    parser
                        .record_coverage_target_event(
                            "array_access_expression",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "array_access_expression",
                            "rule.array_access_expression",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "array_access_expression",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "array_access_expression", start_pos, node.span.end,
                                    consumed, consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "array_access_expression", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "array_access_expression", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "array_access_expression",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self
            .recursion_guard
            .check_cycle("accessor_base_lr_base", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let parse_start = parser.position;
                    let mut best_content: Option<ParseContent<'input>> = None;
                    let mut best_end = parse_start;
                    let mut best_priority: i64 = i64::MIN;
                    let mut best_branch_index: usize = 0usize;
                    let mut best_branch = 0usize;
                    let mut nonassoc_tie = false;
                    let mut result = ParseContent::Sequence(Vec::new());
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "accessor_base_lr_base",
                            "rule.accessor_base_lr_base",
                        );
                    let deterministic_partition_offset = if deterministic_partition_effective_enabled {
                        parser
                            .deterministic_partition_offset_runtime(
                                &deterministic_partition_effective_group,
                                2usize,
                            )
                    } else {
                        0usize
                    };
                    let mut evaluation_order: Vec<usize> = (0..2usize).collect();
                    if deterministic_partition_effective_enabled && 2usize > 1
                        && deterministic_partition_offset > 0
                    {
                        evaluation_order.rotate_left(deterministic_partition_offset);
                    }
                    for branch_index in evaluation_order {
                        match branch_index {
                            0usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 0usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 1usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    1usize, 2usize, "accessor_base_lr_base", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            1usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 1usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 2usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    2usize, 2usize, "accessor_base_lr_base", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    if nonassoc_tie {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    } else if let Some(content) = best_content {
                        parser.position = best_end;
                        semantic_selected_branch_index = Some(best_branch);
                        if parser.logger.is_enabled() {
                            parser
                                .logger
                                .log_info(
                                    "../generated/return_annotation_parser.rs",
                                    0,
                                    &format!(
                                        "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                        "accessor_base_lr_base", best_branch, 2usize, best_end
                                        .saturating_sub(parse_start), best_priority, "left",
                                        "longest_match"
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
                    parser
                        .record_coverage_target_event(
                            "accessor_base_lr_base",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "accessor_base_lr_base",
                            "rule.accessor_base_lr_base",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "accessor_base_lr_base",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "accessor_base_lr_base", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "accessor_base_lr_base", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "accessor_base_lr_base", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "accessor_base_lr_base",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("accessor_base", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
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
                                        let deterministic_partition_effective_enabled = parser
                                            .effective_deterministic_partition_enabled(false);
                                        let deterministic_partition_effective_group = parser
                                            .effective_deterministic_partition_group(
                                                "accessor_base",
                                                "rule.accessor_base",
                                            );
                                        let deterministic_partition_offset = if deterministic_partition_effective_enabled {
                                            parser
                                                .deterministic_partition_offset_runtime(
                                                    &deterministic_partition_effective_group,
                                                    2usize,
                                                )
                                        } else {
                                            0usize
                                        };
                                        let mut evaluation_order: Vec<usize> = (0..2usize)
                                            .collect();
                                        if deterministic_partition_effective_enabled && 2usize > 1
                                            && deterministic_partition_offset > 0
                                        {
                                            evaluation_order
                                                .rotate_left(deterministic_partition_offset);
                                        }
                                        for branch_index in evaluation_order {
                                            match branch_index {
                                                0usize => {
                                                    if "longest_match" == "ordered" && best_content.is_some()
                                                    {} else {
                                                        parser.position = parse_start;
                                                        if let Some(content) = parser
                                                            .try_parse(|p| {
                                                                let parser = p;
                                                                if parser.logger.is_enabled() {
                                                                    parser
                                                                        .logger
                                                                        .log_info(
                                                                            "../generated/return_annotation_parser.rs",
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
                                                                            "../generated/return_annotation_parser.rs",
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
                                                            let current_branch_index: usize = 0usize;
                                                            let transformed = {
                                                                let content = content;
                                                                content
                                                            };
                                                            let should_take = if "longest_match" == "ordered" {
                                                                best_content.is_none()
                                                            } else if "longest_match" == "priority_first" {
                                                                if best_content.is_none() {
                                                                    true
                                                                } else if candidate_priority > best_priority {
                                                                    true
                                                                } else if candidate_priority < best_priority {
                                                                    false
                                                                } else if candidate_end > best_end {
                                                                    true
                                                                } else if candidate_end < best_end {
                                                                    false
                                                                } else {
                                                                    match "left" {
                                                                        "right" => current_branch_index > best_branch_index,
                                                                        "nonassoc" => {
                                                                            if current_branch_index != best_branch_index {
                                                                                nonassoc_tie = true;
                                                                            }
                                                                            false
                                                                        }
                                                                        _ => false,
                                                                    }
                                                                }
                                                            } else if best_content.is_none() {
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
                                                                    "right" => current_branch_index > best_branch_index,
                                                                    "nonassoc" => {
                                                                        if current_branch_index != best_branch_index {
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
                                                                best_branch_index = current_branch_index;
                                                                best_branch = 1usize;
                                                                best_content = Some(transformed);
                                                            }
                                                        } else if parser.logger.is_enabled() {
                                                            parser
                                                                .logger
                                                                .log_info(
                                                                    "../generated/return_annotation_parser.rs",
                                                                    0,
                                                                    &format!(
                                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                                        1usize, 2usize, "accessor_base", parser.position
                                                                    ),
                                                                );
                                                        }
                                                    }
                                                }
                                                1usize => {
                                                    if "longest_match" == "ordered" && best_content.is_some()
                                                    {} else {
                                                        parser.position = parse_start;
                                                        if let Some(content) = parser
                                                            .try_parse(|p| {
                                                                let parser = p;
                                                                if parser.logger.is_enabled() {
                                                                    parser
                                                                        .logger
                                                                        .log_info(
                                                                            "../generated/return_annotation_parser.rs",
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
                                                                            "../generated/return_annotation_parser.rs",
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
                                                            let current_branch_index: usize = 1usize;
                                                            let transformed = {
                                                                let content = content;
                                                                content
                                                            };
                                                            let should_take = if "longest_match" == "ordered" {
                                                                best_content.is_none()
                                                            } else if "longest_match" == "priority_first" {
                                                                if best_content.is_none() {
                                                                    true
                                                                } else if candidate_priority > best_priority {
                                                                    true
                                                                } else if candidate_priority < best_priority {
                                                                    false
                                                                } else if candidate_end > best_end {
                                                                    true
                                                                } else if candidate_end < best_end {
                                                                    false
                                                                } else {
                                                                    match "left" {
                                                                        "right" => current_branch_index > best_branch_index,
                                                                        "nonassoc" => {
                                                                            if current_branch_index != best_branch_index {
                                                                                nonassoc_tie = true;
                                                                            }
                                                                            false
                                                                        }
                                                                        _ => false,
                                                                    }
                                                                }
                                                            } else if best_content.is_none() {
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
                                                                    "right" => current_branch_index > best_branch_index,
                                                                    "nonassoc" => {
                                                                        if current_branch_index != best_branch_index {
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
                                                                best_branch_index = current_branch_index;
                                                                best_branch = 2usize;
                                                                best_content = Some(transformed);
                                                            }
                                                        } else if parser.logger.is_enabled() {
                                                            parser
                                                                .logger
                                                                .log_info(
                                                                    "../generated/return_annotation_parser.rs",
                                                                    0,
                                                                    &format!(
                                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                                        2usize, 2usize, "accessor_base", parser.position
                                                                    ),
                                                                );
                                                        }
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                        if nonassoc_tie {
                                            return Err(ParseError::Backtrack {
                                                position: parse_start,
                                            });
                                        } else if let Some(content) = best_content {
                                            parser.position = best_end;
                                            semantic_selected_branch_index = Some(best_branch);
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                                            "accessor_base", best_branch, 2usize, best_end
                                                            .saturating_sub(parse_start), best_priority, "left",
                                                            "longest_match"
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
                                                    "../generated/return_annotation_parser.rs",
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
                                        "../generated/return_annotation_parser.rs",
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
                    parser
                        .record_coverage_target_event(
                            "accessor_base",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "accessor_base",
                            "rule.accessor_base",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "accessor_base",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "accessor_base", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "accessor_base", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "accessor_base", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "accessor_base",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self
            .recursion_guard
            .check_cycle("positional_reference", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
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
                    parser
                        .record_coverage_target_event(
                            "positional_reference",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "positional_reference",
                            "rule.positional_reference",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "positional_reference",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "positional_reference", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "positional_reference", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "positional_reference", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "positional_reference",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("string_literal", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let parse_start = parser.position;
                    let mut best_content: Option<ParseContent<'input>> = None;
                    let mut best_end = parse_start;
                    let mut best_priority: i64 = i64::MIN;
                    let mut best_branch_index: usize = 0usize;
                    let mut best_branch = 0usize;
                    let mut nonassoc_tie = false;
                    let mut result = ParseContent::Sequence(Vec::new());
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "string_literal",
                            "rule.string_literal",
                        );
                    let deterministic_partition_offset = if deterministic_partition_effective_enabled {
                        parser
                            .deterministic_partition_offset_runtime(
                                &deterministic_partition_effective_group,
                                2usize,
                            )
                    } else {
                        0usize
                    };
                    let mut evaluation_order: Vec<usize> = (0..2usize).collect();
                    if deterministic_partition_effective_enabled && 2usize > 1
                        && deterministic_partition_offset > 0
                    {
                        evaluation_order.rotate_left(deterministic_partition_offset);
                    }
                    for branch_index in evaluation_order {
                        match branch_index {
                            0usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 0usize;
                                        let transformed = {
                                            let content = content;
                                            {
                                                let mut json_obj = serde_json::json!({});
                                                json_obj["type"] = serde_json::json!(("string"));
                                                json_obj["value"] = serde_json::json!(
                                                    ({ let __pgen_content = { let __pgen_base = (content)
                                                    .clone(); match __pgen_base {
                                                    ParseContent::Sequence(elements) if elements.len() > 1usize
                                                    => { elements[1usize].content.clone() }
                                                    ParseContent::Quantified(elements, _) if elements.len() >
                                                    1usize => { elements[1usize].content.clone() } _ =>
                                                    ParseContent::Terminal("<invalid_sequence_access>"), } };
                                                    match __pgen_content { ParseContent::Terminal(s) => s
                                                    .to_string(), ParseContent::TransformedTerminal(s) => s,
                                                    ParseContent::Alternative(node) => { match node.content {
                                                    ParseContent::Terminal(s) => s.to_string(),
                                                    ParseContent::TransformedTerminal(s) => s, other =>
                                                    format!("{:?}", other), } } other => format!("{:?}", other),
                                                    } })
                                                );
                                                let json_str = serde_json::to_string(&json_obj)
                                                    .unwrap_or_else(|_| "{}".to_string());
                                                ParseContent::TransformedTerminal(json_str)
                                            }
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 1usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    1usize, 2usize, "string_literal", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            1usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 1usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 2usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    2usize, 2usize, "string_literal", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    if nonassoc_tie {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    } else if let Some(content) = best_content {
                        parser.position = best_end;
                        semantic_selected_branch_index = Some(best_branch);
                        if parser.logger.is_enabled() {
                            parser
                                .logger
                                .log_info(
                                    "../generated/return_annotation_parser.rs",
                                    0,
                                    &format!(
                                        "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                        "string_literal", best_branch, 2usize, best_end
                                        .saturating_sub(parse_start), best_priority, "left",
                                        "longest_match"
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
                    parser
                        .record_coverage_target_event(
                            "string_literal",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "string_literal",
                            "rule.string_literal",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "string_literal",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "string_literal", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "string_literal", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "string_literal", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "string_literal",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self
            .recursion_guard
            .check_cycle("string_content_double", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let matched_str = parser.match_regex("[^\"]*", false)?;
                    let result = ParseContent::Terminal(matched_str);
                    let end_pos = parser.position;
                    parser
                        .record_coverage_target_event(
                            "string_content_double",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "string_content_double",
                            "rule.string_content_double",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "string_content_double",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "string_content_double", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "string_content_double", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "string_content_double", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "string_content_double",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self
            .recursion_guard
            .check_cycle("string_content_single", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let matched_str = parser.match_regex("[^']*", false)?;
                    let result = ParseContent::Terminal(matched_str);
                    let end_pos = parser.position;
                    parser
                        .record_coverage_target_event(
                            "string_content_single",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "string_content_single",
                            "rule.string_content_single",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "string_content_single",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "string_content_single", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "string_content_single", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "string_content_single", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "string_content_single",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("number_literal", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let parse_start = parser.position;
                    let mut best_content: Option<ParseContent<'input>> = None;
                    let mut best_end = parse_start;
                    let mut best_priority: i64 = i64::MIN;
                    let mut best_branch_index: usize = 0usize;
                    let mut best_branch = 0usize;
                    let mut nonassoc_tie = false;
                    let mut result = ParseContent::Sequence(Vec::new());
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "number_literal",
                            "rule.number_literal",
                        );
                    let deterministic_partition_offset = if deterministic_partition_effective_enabled {
                        parser
                            .deterministic_partition_offset_runtime(
                                &deterministic_partition_effective_group,
                                2usize,
                            )
                    } else {
                        0usize
                    };
                    let mut evaluation_order: Vec<usize> = (0..2usize).collect();
                    if deterministic_partition_effective_enabled && 2usize > 1
                        && deterministic_partition_offset > 0
                    {
                        evaluation_order.rotate_left(deterministic_partition_offset);
                    }
                    for branch_index in evaluation_order {
                        match branch_index {
                            0usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 0usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 1usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    1usize, 2usize, "number_literal", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            1usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 1usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 2usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    2usize, 2usize, "number_literal", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    if nonassoc_tie {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    } else if let Some(content) = best_content {
                        parser.position = best_end;
                        semantic_selected_branch_index = Some(best_branch);
                        if parser.logger.is_enabled() {
                            parser
                                .logger
                                .log_info(
                                    "../generated/return_annotation_parser.rs",
                                    0,
                                    &format!(
                                        "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                        "number_literal", best_branch, 2usize, best_end
                                        .saturating_sub(parse_start), best_priority, "left",
                                        "longest_match"
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
                    parser
                        .record_coverage_target_event(
                            "number_literal",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "number_literal",
                            "rule.number_literal",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "number_literal",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "number_literal", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "number_literal", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "number_literal", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "number_literal",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("float", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let matched_str = parser
                        .match_regex("[-+]?[0-9]+\\.[0-9]+(?:[eE][-+]?[0-9]+)?", true)?;
                    let transformed = matched_str.parse::<f64>().unwrap_or(0.0);
                    if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_debug(
                                file!(),
                                line!(),
                                &format!(
                                    "🎯 Applied semantic transform: parsed '{}' to {}={}",
                                    matched_str, stringify!(f64), transformed
                                ),
                            );
                    }
                    let result = ParseContent::TransformedTerminal(
                        transformed.to_string(),
                    );
                    let end_pos = parser.position;
                    parser
                        .record_coverage_target_event(
                            "float",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group("float", "rule.float");
                    parser
                        .record_deterministic_partition_event(
                            "float",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "float", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "float", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "float", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "float",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("integer", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let matched_str = parser.match_regex("[-+]?[0-9]+", true)?;
                    let transformed = matched_str.parse::<i64>().unwrap_or(0);
                    if parser.logger.is_enabled() {
                        parser
                            .logger
                            .log_debug(
                                file!(),
                                line!(),
                                &format!(
                                    "🎯 Applied semantic transform: parsed '{}' to {}={}",
                                    matched_str, stringify!(i64), transformed
                                ),
                            );
                    }
                    let result = ParseContent::TransformedTerminal(
                        transformed.to_string(),
                    );
                    let end_pos = parser.position;
                    parser
                        .record_coverage_target_event(
                            "integer",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "integer",
                            "rule.integer",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "integer",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "integer", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "integer", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "integer", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "integer",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("boolean_literal", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let parse_start = parser.position;
                    let mut best_content: Option<ParseContent<'input>> = None;
                    let mut best_end = parse_start;
                    let mut best_priority: i64 = i64::MIN;
                    let mut best_branch_index: usize = 0usize;
                    let mut best_branch = 0usize;
                    let mut nonassoc_tie = false;
                    let mut result = ParseContent::Sequence(Vec::new());
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "boolean_literal",
                            "rule.boolean_literal",
                        );
                    let deterministic_partition_offset = if deterministic_partition_effective_enabled {
                        parser
                            .deterministic_partition_offset_runtime(
                                &deterministic_partition_effective_group,
                                2usize,
                            )
                    } else {
                        0usize
                    };
                    let mut evaluation_order: Vec<usize> = (0..2usize).collect();
                    if deterministic_partition_effective_enabled && 2usize > 1
                        && deterministic_partition_offset > 0
                    {
                        evaluation_order.rotate_left(deterministic_partition_offset);
                    }
                    for branch_index in evaluation_order {
                        match branch_index {
                            0usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 0usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 1usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    1usize, 2usize, "boolean_literal", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            1usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 1usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 2usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    2usize, 2usize, "boolean_literal", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    if nonassoc_tie {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    } else if let Some(content) = best_content {
                        parser.position = best_end;
                        semantic_selected_branch_index = Some(best_branch);
                        if parser.logger.is_enabled() {
                            parser
                                .logger
                                .log_info(
                                    "../generated/return_annotation_parser.rs",
                                    0,
                                    &format!(
                                        "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                        "boolean_literal", best_branch, 2usize, best_end
                                        .saturating_sub(parse_start), best_priority, "left",
                                        "longest_match"
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
                    parser
                        .record_coverage_target_event(
                            "boolean_literal",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "boolean_literal",
                            "rule.boolean_literal",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "boolean_literal",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "boolean_literal", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "boolean_literal", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "boolean_literal", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "boolean_literal",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("identifier", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let matched_str = parser
                        .match_regex("[a-zA-Z_][a-zA-Z0-9_]*", true)?;
                    let result = ParseContent::Terminal(matched_str);
                    let end_pos = parser.position;
                    parser
                        .record_coverage_target_event(
                            "identifier",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "identifier",
                            "rule.identifier",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "identifier",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "identifier", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "identifier", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "identifier", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "identifier",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("object_literal", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
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
                    parser
                        .record_coverage_target_event(
                            "object_literal",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "object_literal",
                            "rule.object_literal",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "object_literal",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "object_literal", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "object_literal", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "object_literal", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "object_literal",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("object_properties", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
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
                                                    "../generated/return_annotation_parser.rs",
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
                                        "../generated/return_annotation_parser.rs",
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
                    parser
                        .record_coverage_target_event(
                            "object_properties",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "object_properties",
                            "rule.object_properties",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "object_properties",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "object_properties", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "object_properties", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "object_properties", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "object_properties",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("object_property", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
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
                    parser
                        .record_coverage_target_event(
                            "object_property",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "object_property",
                            "rule.object_property",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "object_property",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "object_property", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "object_property", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "object_property", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "object_property",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("property_key", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let parse_start = parser.position;
                    let mut best_content: Option<ParseContent<'input>> = None;
                    let mut best_end = parse_start;
                    let mut best_priority: i64 = i64::MIN;
                    let mut best_branch_index: usize = 0usize;
                    let mut best_branch = 0usize;
                    let mut nonassoc_tie = false;
                    let mut result = ParseContent::Sequence(Vec::new());
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "property_key",
                            "rule.property_key",
                        );
                    let deterministic_partition_offset = if deterministic_partition_effective_enabled {
                        parser
                            .deterministic_partition_offset_runtime(
                                &deterministic_partition_effective_group,
                                2usize,
                            )
                    } else {
                        0usize
                    };
                    let mut evaluation_order: Vec<usize> = (0..2usize).collect();
                    if deterministic_partition_effective_enabled && 2usize > 1
                        && deterministic_partition_offset > 0
                    {
                        evaluation_order.rotate_left(deterministic_partition_offset);
                    }
                    for branch_index in evaluation_order {
                        match branch_index {
                            0usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 0usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 1usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    1usize, 2usize, "property_key", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            1usize => {
                                if "longest_match" == "ordered" && best_content.is_some()
                                {} else {
                                    parser.position = parse_start;
                                    if let Some(content) = parser
                                        .try_parse(|p| {
                                            let parser = p;
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "../generated/return_annotation_parser.rs",
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
                                                        "../generated/return_annotation_parser.rs",
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
                                        let current_branch_index: usize = 1usize;
                                        let transformed = {
                                            let content = content;
                                            content
                                        };
                                        let should_take = if "longest_match" == "ordered" {
                                            best_content.is_none()
                                        } else if "longest_match" == "priority_first" {
                                            if best_content.is_none() {
                                                true
                                            } else if candidate_priority > best_priority {
                                                true
                                            } else if candidate_priority < best_priority {
                                                false
                                            } else if candidate_end > best_end {
                                                true
                                            } else if candidate_end < best_end {
                                                false
                                            } else {
                                                match "left" {
                                                    "right" => current_branch_index > best_branch_index,
                                                    "nonassoc" => {
                                                        if current_branch_index != best_branch_index {
                                                            nonassoc_tie = true;
                                                        }
                                                        false
                                                    }
                                                    _ => false,
                                                }
                                            }
                                        } else if best_content.is_none() {
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
                                                "right" => current_branch_index > best_branch_index,
                                                "nonassoc" => {
                                                    if current_branch_index != best_branch_index {
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
                                            best_branch_index = current_branch_index;
                                            best_branch = 2usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "../generated/return_annotation_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    2usize, 2usize, "property_key", parser.position
                                                ),
                                            );
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    if nonassoc_tie {
                        return Err(ParseError::Backtrack {
                            position: parse_start,
                        });
                    } else if let Some(content) = best_content {
                        parser.position = best_end;
                        semantic_selected_branch_index = Some(best_branch);
                        if parser.logger.is_enabled() {
                            parser
                                .logger
                                .log_info(
                                    "../generated/return_annotation_parser.rs",
                                    0,
                                    &format!(
                                        "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                        "property_key", best_branch, 2usize, best_end
                                        .saturating_sub(parse_start), best_priority, "left",
                                        "longest_match"
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
                    parser
                        .record_coverage_target_event(
                            "property_key",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "property_key",
                            "rule.property_key",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "property_key",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "property_key", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "property_key", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "property_key", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "property_key",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("array_literal", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
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
                    parser
                        .record_coverage_target_event(
                            "array_literal",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "array_literal",
                            "rule.array_literal",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "array_literal",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "array_literal", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "array_literal", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "array_literal", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "array_literal",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("array_elements", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
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
                                                    "../generated/return_annotation_parser.rs",
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
                                        "../generated/return_annotation_parser.rs",
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
                    parser
                        .record_coverage_target_event(
                            "array_elements",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "array_elements",
                            "rule.array_elements",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "array_elements",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "array_elements", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "array_elements", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "array_elements", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "array_elements",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("array_element", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let result = ParseContent::Alternative(
                        Box::new(parser.parse_expression()?),
                    );
                    let end_pos = parser.position;
                    parser
                        .record_coverage_target_event(
                            "array_element",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "array_element",
                            "rule.array_element",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "array_element",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "array_element", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "array_element", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "array_element", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "array_element",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
        let filename_str = "../generated/return_annotation_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("parenthesized", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    let mut semantic_selected_branch_index: Option<usize> = None;
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
                    parser
                        .record_coverage_target_event(
                            "parenthesized",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group(
                            "parenthesized",
                            "rule.parenthesized",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "parenthesized",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
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
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "parenthesized", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "../generated/return_annotation_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "parenthesized", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "parenthesized", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "parenthesized",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "../generated/return_annotation_parser.rs",
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
    fn byte_window_lossy(&self, start: usize, end: usize) -> String {
        if start >= end || start >= self.input.len() {
            return String::new();
        }
        let clamped_end = end.min(self.input.len());
        String::from_utf8_lossy(&self.input.as_bytes()[start..clamped_end]).to_string()
    }
    fn rule_profile_is_enabled(&self, allowed_profiles: &[&str]) -> bool {
        if allowed_profiles.is_empty() {
            return true;
        }
        match self.grammar_profile.as_deref() {
            Some(active) => {
                allowed_profiles
                    .iter()
                    .any(|candidate| active.eq_ignore_ascii_case(candidate))
            }
            None => true,
        }
    }
    fn bytes_match_at(&self, start: usize, expected: &[u8]) -> bool {
        let Some(end) = start.checked_add(expected.len()) else {
            return false;
        };
        if end > self.input.len() {
            return false;
        }
        &self.input.as_bytes()[start..end] == expected
    }
    fn find_token_from(&self, start: usize, token: &str) -> Option<usize> {
        if token.is_empty() || start >= self.input.len() {
            return None;
        }
        let token_bytes = token.as_bytes();
        if token_bytes.len() > self.input.len() {
            return None;
        }
        let max_start = self.input.len().saturating_sub(token_bytes.len());
        for idx in start..=max_start {
            if self.bytes_match_at(idx, token_bytes) {
                return Some(idx);
            }
        }
        None
    }
    fn effective_deterministic_partition_enabled(
        &self,
        annotation_enabled: bool,
    ) -> bool {
        match self.deterministic_partition_runtime_mode {
            DeterministicPartitionRuntimeMode::AnnotationDriven => annotation_enabled,
            DeterministicPartitionRuntimeMode::ForceEnabled => true,
            DeterministicPartitionRuntimeMode::ForceDisabled => false,
        }
    }
    fn effective_deterministic_partition_group(
        &self,
        rule_name: &str,
        annotation_group: &str,
    ) -> String {
        let trimmed = annotation_group.trim();
        if !trimmed.is_empty() {
            trimmed.to_string()
        } else {
            format!("rule.{}", rule_name)
        }
    }
    fn deterministic_partition_offset_runtime(
        &self,
        group_key: &str,
        branch_count: usize,
    ) -> usize {
        if branch_count <= 1 {
            return 0;
        }
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for byte in group_key.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
        }
        (hash as usize) % branch_count
    }
    fn record_coverage_target_event(
        &mut self,
        rule_name: &str,
        parse_start: usize,
        parse_end: usize,
        branch_index: Option<usize>,
        coverage_target_weight: u64,
        critical_path: bool,
    ) {
        if coverage_target_weight == 0 {
            return;
        }
        self.coverage_target_events
            .push(CoverageTargetEvent {
                rule_name: rule_name.to_string(),
                parse_start,
                parse_end,
                branch_index,
                coverage_target_weight,
                critical_path,
            });
        *self.coverage_target_rule_hits.entry(rule_name.to_string()).or_insert(0) += 1;
        if let Some(branch) = branch_index {
            let branch_key = format!("{}::{}", rule_name, branch);
            *self.coverage_target_branch_hits.entry(branch_key).or_insert(0) += 1;
        }
        if self.logger.is_enabled() {
            let marker = if critical_path { "critical" } else { "target" };
            self.logger
                .log_info(
                    "../generated/return_annotation_parser.rs",
                    0,
                    &format!(
                        "🎯 SC-10 parser instrumentation: rule='{}' branch={:?} weight={} kind={} span={}..{}",
                        rule_name, branch_index, coverage_target_weight, marker,
                        parse_start, parse_end
                    ),
                );
        }
    }
    fn record_deterministic_partition_event(
        &mut self,
        rule_name: &str,
        parse_start: usize,
        parse_end: usize,
        enabled: bool,
        group_key: &str,
    ) {
        if !enabled {
            return;
        }
        self.deterministic_partition_events
            .push(DeterministicPartitionEvent {
                rule_name: rule_name.to_string(),
                parse_start,
                parse_end,
                group_key: group_key.to_string(),
            });
        *self.deterministic_partition_rule_hits.entry(rule_name.to_string()).or_insert(0)
            += 1;
        if self.logger.is_enabled() {
            self.logger
                .log_info(
                    "../generated/return_annotation_parser.rs",
                    0,
                    &format!(
                        "🧭 SC-12 parser partition: rule='{}' group='{}' span={}..{}",
                        rule_name, group_key, parse_start, parse_end
                    ),
                );
        }
    }
    fn record_negative_case_failure(
        &mut self,
        rule_name: &str,
        parse_start: usize,
        failure_position: usize,
        negative: bool,
        error_kind: &str,
    ) {
        self.negative_case_events
            .push(NegativeCaseEvent {
                rule_name: rule_name.to_string(),
                parse_start,
                failure_position,
                negative,
                error_kind: error_kind.to_string(),
            });
        *self.negative_case_rule_hits.entry(rule_name.to_string()).or_insert(0) += 1;
        if self.logger.is_enabled() {
            let mode = if negative { "near-invalid" } else { "invalid-case" };
            self.logger
                .log_info(
                    "../generated/return_annotation_parser.rs",
                    0,
                    &format!(
                        "⚠️ SC-11 expected-failure path: rule='{}' mode={} start={} failure={} kind={}",
                        rule_name, mode, parse_start, failure_position, error_kind
                    ),
                );
        }
    }
    fn recover_with_hints(
        &mut self,
        rule_name: &str,
        parse_start: usize,
        sync_tokens: &[&str],
        panic_until_tokens: &[&str],
        recover_budget: Option<usize>,
        recover_parse_budget: Option<usize>,
        recover_global_budget: Option<usize>,
    ) -> bool {
        if let Some(limit) = recover_budget {
            let used = self.recovery_counts.get(rule_name).copied().unwrap_or(0);
            if used >= limit {
                if self.logger.is_enabled() {
                    self.logger
                        .log_warning(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🛟 Recovery budget exhausted for rule '{}': used={} limit={}",
                                rule_name, used, limit
                            ),
                        );
                }
                return false;
            }
        }
        if let Some(limit) = recover_parse_budget {
            if self.recovery_parse_count >= limit {
                if self.logger.is_enabled() {
                    self.logger
                        .log_warning(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🛟 Parse-scope recovery budget exhausted for rule '{}': used={} limit={}",
                                rule_name, self.recovery_parse_count, limit
                            ),
                        );
                }
                return false;
            }
        }
        if let Some(limit) = recover_global_budget {
            if self.recovery_global_count >= limit {
                if self.logger.is_enabled() {
                    self.logger
                        .log_warning(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "🛟 Global recovery budget exhausted for rule '{}': used={} limit={}",
                                rule_name, self.recovery_global_count, limit
                            ),
                        );
                }
                return false;
            }
        }
        let recovery_start = parse_start.min(self.input.len());
        let mut best: Option<(usize, usize, u8, String)> = None;
        for token in panic_until_tokens {
            if token.is_empty() {
                continue;
            }
            if let Some(pos) = self.find_token_from(recovery_start, token) {
                let candidate = (pos, token.len(), 0u8, token.to_string());
                let take_candidate = match &best {
                    None => true,
                    Some((best_pos, _best_len, best_priority, _best_token)) => {
                        pos < *best_pos
                            || (pos == *best_pos && candidate.2 < *best_priority)
                    }
                };
                if take_candidate {
                    best = Some(candidate);
                }
            }
        }
        for token in sync_tokens {
            if token.is_empty() {
                continue;
            }
            if let Some(pos) = self.find_token_from(recovery_start, token) {
                let candidate = (pos, token.len(), 1u8, token.to_string());
                let take_candidate = match &best {
                    None => true,
                    Some((best_pos, _best_len, best_priority, _best_token)) => {
                        pos < *best_pos
                            || (pos == *best_pos && candidate.2 < *best_priority)
                    }
                };
                if take_candidate {
                    best = Some(candidate);
                }
            }
        }
        if let Some((token_pos, token_len, token_priority, token_value)) = best {
            let previous = self.position;
            let token_end = token_pos.saturating_add(token_len).min(self.input.len());
            let mut new_position = token_end;
            if new_position <= previous && previous < self.input.len() {
                new_position = previous + 1;
            }
            self.position = new_position.min(self.input.len());
            let marker_kind = if token_priority == 0 {
                RecoveryMarkerKind::PanicUntil
            } else {
                RecoveryMarkerKind::Sync
            };
            self.recovery_events
                .push(RecoveryEvent {
                    rule_name: rule_name.to_string(),
                    parse_start,
                    previous_position: previous,
                    new_position: self.position,
                    marker_kind,
                    marker_position: Some(token_pos),
                    marker_value: Some(token_value.clone()),
                });
            *self.recovery_counts.entry(rule_name.to_string()).or_insert(0) += 1;
            self.recovery_parse_count += 1;
            self.recovery_global_count += 1;
            if self.logger.is_enabled() {
                let marker = if token_priority == 0 { "panic_until" } else { "sync" };
                self.logger
                    .log_warning(
                        "../generated/return_annotation_parser.rs",
                        0,
                        &format!(
                            "🛟 Recovery for rule '{}': moved parser from {} to {} using {} token at {}",
                            rule_name, previous, self.position, marker, token_pos
                        ),
                    );
            }
            return self.position > parse_start;
        }
        if self.position < self.input.len() {
            let previous = self.position;
            self.position = self.input.len();
            self.recovery_events
                .push(RecoveryEvent {
                    rule_name: rule_name.to_string(),
                    parse_start,
                    previous_position: previous,
                    new_position: self.position,
                    marker_kind: RecoveryMarkerKind::EofFallback,
                    marker_position: None,
                    marker_value: None,
                });
            *self.recovery_counts.entry(rule_name.to_string()).or_insert(0) += 1;
            self.recovery_parse_count += 1;
            self.recovery_global_count += 1;
            if self.logger.is_enabled() {
                self.logger
                    .log_warning(
                        "../generated/return_annotation_parser.rs",
                        0,
                        &format!(
                            "🛟 Recovery for rule '{}': no sync/panic token found, skipped to EOF ({} -> {})",
                            rule_name, previous, self.position
                        ),
                    );
            }
            return true;
        }
        false
    }
    fn enforce_relational_requires(
        &self,
        rule_name: &str,
        root_content: &ParseContent<'input>,
        required_references: &[&str],
    ) -> ParseResult<()> {
        for reference in required_references {
            let normalized = reference.trim();
            if normalized.is_empty() {
                continue;
            }
            let Some(value) = self.resolve_semantic_reference(root_content, normalized)
            else {
                return Err(
                    self
                        .create_contextual_error(
                            &format!(
                                "Semantic @requires contract failed for rule '{}': unresolved reference '{}'",
                                rule_name, normalized
                            ),
                        ),
                );
            };
            if value.trim().is_empty() {
                return Err(
                    self
                        .create_contextual_error(
                            &format!(
                                "Semantic @requires contract failed for rule '{}': empty reference '{}'",
                                rule_name, normalized
                            ),
                        ),
                );
            }
        }
        Ok(())
    }
    fn evaluate_relational_expression(
        &self,
        root_content: &ParseContent<'input>,
        expression: &str,
    ) -> ParseResult<bool> {
        let normalized = expression.trim();
        if normalized.is_empty() {
            return Err(
                self
                    .create_contextual_error(
                        "Semantic relational expression cannot be empty",
                    ),
            );
        }
        self.evaluate_relational_expression_inner(root_content, normalized)
    }
    fn evaluate_relational_expression_inner(
        &self,
        root_content: &ParseContent<'input>,
        expression: &str,
    ) -> ParseResult<bool> {
        let mut normalized = expression.trim();
        while self.semantic_encloses_full_parens(normalized) {
            normalized = normalized[1..normalized.len() - 1].trim();
        }
        let disjuncts = self.split_semantic_top_level(normalized, "||");
        if disjuncts.len() > 1 {
            for term in disjuncts {
                if term.is_empty() {
                    continue;
                }
                if self.evaluate_relational_expression_inner(root_content, term)? {
                    return Ok(true);
                }
            }
            return Ok(false);
        }
        let conjuncts = self.split_semantic_top_level(normalized, "&&");
        if conjuncts.len() > 1 {
            for term in conjuncts {
                if term.is_empty() {
                    continue;
                }
                if !self.evaluate_relational_expression_inner(root_content, term)? {
                    return Ok(false);
                }
            }
            return Ok(true);
        }
        if let Some(rest) = normalized.strip_prefix('!') {
            return Ok(!self.evaluate_relational_expression_inner(root_content, rest)?);
        }
        for operator in ["==", "!=", ">=", "<=", ">", "<"] {
            if let Some((left, right)) = self
                .split_semantic_top_level_once(normalized, operator)
            {
                return self
                    .evaluate_relational_comparison(root_content, left, operator, right);
            }
        }
        if self.semantic_reference_syntax(normalized) {
            let value = self
                .resolve_semantic_reference(root_content, normalized)
                .ok_or_else(|| {
                    self.create_contextual_error(
                        &format!(
                            "Semantic relational expression references unresolved capture '{}'",
                            normalized
                        ),
                    )
                })?;
            return Ok(Self::semantic_truthy(&value));
        }
        if let Some(unquoted) = Self::semantic_unquote(normalized) {
            return Ok(Self::semantic_truthy(unquoted));
        }
        if let Ok(number) = normalized.parse::<f64>() {
            return Ok(number != 0.0);
        }
        let lowered = normalized.to_ascii_lowercase();
        if lowered == "true" {
            return Ok(true);
        }
        if lowered == "false" {
            return Ok(false);
        }
        Ok(Self::semantic_truthy(normalized))
    }
    fn evaluate_relational_comparison(
        &self,
        root_content: &ParseContent<'input>,
        left: &str,
        operator: &str,
        right: &str,
    ) -> ParseResult<bool> {
        let lhs = self.resolve_relational_operand(root_content, left)?;
        let rhs = self.resolve_relational_operand(root_content, right)?;
        let lhs_numeric = lhs.parse::<f64>().ok();
        let rhs_numeric = rhs.parse::<f64>().ok();
        match operator {
            "==" => {
                if let (Some(a), Some(b)) = (lhs_numeric, rhs_numeric) {
                    Ok((a - b).abs() <= f64::EPSILON)
                } else {
                    Ok(lhs == rhs)
                }
            }
            "!=" => {
                if let (Some(a), Some(b)) = (lhs_numeric, rhs_numeric) {
                    Ok((a - b).abs() > f64::EPSILON)
                } else {
                    Ok(lhs != rhs)
                }
            }
            ">" => {
                if let (Some(a), Some(b)) = (lhs_numeric, rhs_numeric) {
                    Ok(a > b)
                } else {
                    Ok(lhs > rhs)
                }
            }
            ">=" => {
                if let (Some(a), Some(b)) = (lhs_numeric, rhs_numeric) {
                    Ok(a >= b)
                } else {
                    Ok(lhs >= rhs)
                }
            }
            "<" => {
                if let (Some(a), Some(b)) = (lhs_numeric, rhs_numeric) {
                    Ok(a < b)
                } else {
                    Ok(lhs < rhs)
                }
            }
            "<=" => {
                if let (Some(a), Some(b)) = (lhs_numeric, rhs_numeric) {
                    Ok(a <= b)
                } else {
                    Ok(lhs <= rhs)
                }
            }
            _ => {
                Err(
                    self
                        .create_contextual_error(
                            &format!(
                                "Unsupported semantic comparison operator '{}'", operator
                            ),
                        ),
                )
            }
        }
    }
    fn resolve_relational_operand(
        &self,
        root_content: &ParseContent<'input>,
        operand: &str,
    ) -> ParseResult<String> {
        let normalized = operand.trim();
        if normalized.is_empty() {
            return Err(
                self
                    .create_contextual_error(
                        "Semantic relational operand cannot be empty",
                    ),
            );
        }
        if let Some(unquoted) = Self::semantic_unquote(normalized) {
            return Ok(unquoted.to_string());
        }
        if self.semantic_reference_syntax(normalized) {
            return self
                .resolve_semantic_reference(root_content, normalized)
                .ok_or_else(|| {
                    self.create_contextual_error(
                        &format!(
                            "Semantic relational operand references unresolved capture '{}'",
                            normalized
                        ),
                    )
                });
        }
        Ok(normalized.to_string())
    }
    fn resolve_semantic_reference(
        &self,
        root_content: &ParseContent<'input>,
        reference: &str,
    ) -> Option<String> {
        let normalized = reference.trim();
        if normalized.is_empty() {
            return None;
        }
        let (core_reference, wants_len) = if let Some(stripped) = normalized
            .strip_suffix(".len")
        {
            (stripped, true)
        } else {
            (normalized, false)
        };
        let resolved = if core_reference.starts_with('$') {
            self.resolve_positional_semantic_reference(root_content, core_reference)
        } else {
            self.resolve_named_semantic_reference(root_content, core_reference)
        }?;
        if wants_len {
            Some(resolved.chars().count().to_string())
        } else {
            Some(resolved)
        }
    }
    fn resolve_positional_semantic_reference(
        &self,
        root_content: &ParseContent<'input>,
        reference: &str,
    ) -> Option<String> {
        let (index, path_segments) = self.parse_semantic_reference_segments(reference)?;
        let mut current_node = match root_content {
            ParseContent::Sequence(elements) => elements.get(index.saturating_sub(1))?,
            ParseContent::Alternative(node) => {
                if index == 1 {
                    node.as_ref()
                } else {
                    return None;
                }
            }
            ParseContent::Quantified(elements, _) => {
                elements.get(index.saturating_sub(1))?
            }
            _ => return None,
        };
        for segment in path_segments {
            current_node = self
                .find_semantic_named_descendant(&current_node.content, segment)?;
        }
        self.semantic_node_scalar(current_node)
    }
    fn resolve_named_semantic_reference(
        &self,
        root_content: &ParseContent<'input>,
        reference: &str,
    ) -> Option<String> {
        let mut path_segments = reference
            .split('.')
            .map(str::trim)
            .filter(|segment| !segment.is_empty());
        let first = path_segments.next()?;
        if !self.semantic_identifier(first) {
            return None;
        }
        let mut current_node = self.find_semantic_named_descendant(root_content, first)?;
        for segment in path_segments {
            if !self.semantic_identifier(segment) {
                return None;
            }
            current_node = self
                .find_semantic_named_descendant(&current_node.content, segment)?;
        }
        self.semantic_node_scalar(current_node)
    }
    fn parse_semantic_reference_segments<'a>(
        &self,
        reference: &'a str,
    ) -> Option<(usize, Vec<&'a str>)> {
        let normalized = reference.trim();
        if !normalized.starts_with('$') {
            return None;
        }
        let bytes = normalized.as_bytes();
        let mut index_end = 1usize;
        while index_end < bytes.len() && bytes[index_end].is_ascii_digit() {
            index_end += 1;
        }
        if index_end == 1 {
            return None;
        }
        let index = normalized[1..index_end].parse::<usize>().ok()?;
        if index == 0 {
            return None;
        }
        let mut segments = Vec::new();
        let suffix = normalized[index_end..].trim();
        if suffix.is_empty() {
            return Some((index, segments));
        }
        if !suffix.starts_with('.') {
            return None;
        }
        for segment in suffix[1..].split('.') {
            let normalized_segment = segment.trim();
            if normalized_segment.is_empty()
                || !self.semantic_identifier(normalized_segment)
            {
                return None;
            }
            segments.push(normalized_segment);
        }
        Some((index, segments))
    }
    fn find_semantic_named_descendant<'a>(
        &self,
        content: &'a ParseContent<'input>,
        target_name: &str,
    ) -> Option<&'a ParseNode<'input>> {
        match content {
            ParseContent::Sequence(elements) | ParseContent::Quantified(elements, _) => {
                for node in elements {
                    if node.rule_name == target_name {
                        return Some(node);
                    }
                    if let Some(found) = self
                        .find_semantic_named_descendant(&node.content, target_name)
                    {
                        return Some(found);
                    }
                }
                None
            }
            ParseContent::Alternative(node) => {
                if node.rule_name == target_name {
                    Some(node)
                } else {
                    self.find_semantic_named_descendant(&node.content, target_name)
                }
            }
            _ => None,
        }
    }
    fn semantic_node_scalar(&self, node: &ParseNode<'input>) -> Option<String> {
        self.semantic_content_scalar(&node.content)
    }
    fn semantic_content_scalar(&self, content: &ParseContent<'input>) -> Option<String> {
        match content {
            ParseContent::Terminal(value) => Some((*value).to_string()),
            ParseContent::TransformedTerminal(value) => Some(value.clone()),
            ParseContent::Alternative(node) => self.semantic_node_scalar(node),
            ParseContent::Sequence(elements) | ParseContent::Quantified(elements, _) => {
                let mut merged = String::new();
                for node in elements {
                    if let Some(value) = self.semantic_node_scalar(node) {
                        merged.push_str(&value);
                    }
                }
                if merged.trim().is_empty() { None } else { Some(merged) }
            }
        }
    }
    fn semantic_reference_syntax(&self, reference: &str) -> bool {
        let normalized = reference.trim();
        if normalized.is_empty() {
            return false;
        }
        if normalized.starts_with('$') {
            return self.parse_semantic_reference_segments(normalized).is_some();
        }
        let mut segments = normalized.split('.');
        let Some(first) = segments.next() else {
            return false;
        };
        if !self.semantic_identifier(first) {
            return false;
        }
        segments.all(|segment| self.semantic_identifier(segment))
    }
    fn semantic_identifier(&self, segment: &str) -> bool {
        let bytes = segment.as_bytes();
        let Some(first) = bytes.first() else {
            return false;
        };
        if !(*first == b'_' || (*first as char).is_ascii_alphabetic()) {
            return false;
        }
        bytes[1..].iter().all(|b| *b == b'_' || (*b as char).is_ascii_alphanumeric())
    }
    fn split_semantic_top_level<'a>(
        &self,
        expression: &'a str,
        separator: &str,
    ) -> Vec<&'a str> {
        if separator.is_empty() {
            return vec![expression.trim()];
        }
        let bytes = expression.as_bytes();
        let separator_bytes = separator.as_bytes();
        if separator_bytes.is_empty() || bytes.len() < separator_bytes.len() {
            return vec![expression.trim()];
        }
        let mut parts = Vec::new();
        let mut start = 0usize;
        let mut idx = 0usize;
        let mut depth = 0usize;
        let mut quote: Option<u8> = None;
        while idx < bytes.len() {
            let current = bytes[idx];
            if let Some(active_quote) = quote {
                if current == active_quote && (idx == 0 || bytes[idx - 1] != b'\\') {
                    quote = None;
                }
                idx += 1;
                continue;
            }
            match current {
                b'"' | b'\'' => {
                    quote = Some(current);
                    idx += 1;
                    continue;
                }
                b'(' => {
                    depth += 1;
                    idx += 1;
                    continue;
                }
                b')' => {
                    depth = depth.saturating_sub(1);
                    idx += 1;
                    continue;
                }
                _ => {}
            }
            if depth == 0 && idx + separator_bytes.len() <= bytes.len()
                && &bytes[idx..idx + separator_bytes.len()] == separator_bytes
            {
                parts.push(expression[start..idx].trim());
                idx += separator_bytes.len();
                start = idx;
                continue;
            }
            idx += 1;
        }
        parts.push(expression[start..].trim());
        parts
    }
    fn split_semantic_top_level_once<'a>(
        &self,
        expression: &'a str,
        separator: &str,
    ) -> Option<(&'a str, &'a str)> {
        let pieces = self.split_semantic_top_level(expression, separator);
        if pieces.len() != 2 {
            return None;
        }
        Some((pieces[0], pieces[1]))
    }
    fn semantic_encloses_full_parens(&self, expression: &str) -> bool {
        let normalized = expression.trim();
        if normalized.len() < 2 || !normalized.starts_with('(')
            || !normalized.ends_with(')')
        {
            return false;
        }
        let bytes = normalized.as_bytes();
        let mut depth = 0usize;
        let mut quote: Option<u8> = None;
        for (idx, current) in bytes.iter().enumerate() {
            let current = *current;
            if let Some(active_quote) = quote {
                if current == active_quote && (idx == 0 || bytes[idx - 1] != b'\\') {
                    quote = None;
                }
                continue;
            }
            match current {
                b'"' | b'\'' => {
                    quote = Some(current);
                }
                b'(' => depth += 1,
                b')' => {
                    if depth == 0 {
                        return false;
                    }
                    depth -= 1;
                    if depth == 0 && idx + 1 < bytes.len() {
                        return false;
                    }
                }
                _ => {}
            }
        }
        depth == 0 && quote.is_none()
    }
    fn semantic_unquote(value: &str) -> Option<&str> {
        let normalized = value.trim();
        if normalized.len() >= 2
            && ((normalized.starts_with('"') && normalized.ends_with('"'))
                || (normalized.starts_with('\'') && normalized.ends_with('\'')))
        {
            return Some(&normalized[1..normalized.len() - 1]);
        }
        None
    }
    fn semantic_truthy(value: &str) -> bool {
        let normalized = value.trim();
        if normalized.is_empty() {
            return false;
        }
        let lowered = normalized
            .trim_matches('"')
            .trim_matches('\'')
            .trim()
            .to_ascii_lowercase();
        !matches!(lowered.as_str(), "" | "false" | "0" | "no" | "off" | "none" | "null")
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
    fn consume_horizontal_whitespace(&mut self) {
        while self.position < self.input.len() {
            let b = self.input.as_bytes()[self.position];
            if matches!(b, b' ' | b'\t') {
                self.position += 1;
            } else {
                break;
            }
        }
    }
    fn consume_layout_for_terminal(&mut self, expected: &str) {
        let allow_comment_skip = expected != "#" && expected != "//" && expected != "/*"
            && expected != "/**" && expected != "///" && expected != "/";
        loop {
            let before = self.position;
            self.consume_optional_whitespace();
            if !allow_comment_skip || self.position >= self.input.len() {
                break;
            }
            let bytes = self.input.as_bytes();
            let len = bytes.len();
            if bytes[self.position] == b'#' {
                while self.position < self.input.len() {
                    let b = bytes[self.position];
                    if b == b'\n' || b == b'\r' {
                        break;
                    }
                    self.position += 1;
                }
                continue;
            }
            if self.position + 1 < len && bytes[self.position] == b'/'
                && bytes[self.position + 1] == b'/'
            {
                self.position += 2;
                while self.position < self.input.len() {
                    let b = bytes[self.position];
                    if b == b'\n' || b == b'\r' {
                        break;
                    }
                    self.position += 1;
                }
                continue;
            }
            if self.position + 1 < len && bytes[self.position] == b'/'
                && bytes[self.position + 1] == b'*'
            {
                self.position += 2;
                while self.position + 1 < len
                    && !(bytes[self.position] == b'*'
                        && bytes[self.position + 1] == b'/')
                {
                    self.position += 1;
                }
                if self.position + 1 < len {
                    self.position += 2;
                }
                continue;
            }
            if self.position == before {
                break;
            }
        }
    }
    fn consume_layout_for_regex(&mut self, can_match_empty: bool) {
        if can_match_empty {
            self.consume_horizontal_whitespace();
            return;
        }
        loop {
            let before = self.position;
            self.consume_optional_whitespace();
            if self.position >= self.input.len() {
                break;
            }
            let bytes = self.input.as_bytes();
            let len = bytes.len();
            if bytes[self.position] == b'#' {
                while self.position < self.input.len() {
                    let b = bytes[self.position];
                    if b == b'\n' || b == b'\r' {
                        break;
                    }
                    self.position += 1;
                }
                continue;
            }
            if self.position + 1 < len && bytes[self.position] == b'/'
                && bytes[self.position + 1] == b'/'
            {
                self.position += 2;
                while self.position < self.input.len() {
                    let b = bytes[self.position];
                    if b == b'\n' || b == b'\r' {
                        break;
                    }
                    self.position += 1;
                }
                continue;
            }
            if self.position + 1 < len && bytes[self.position] == b'/'
                && bytes[self.position + 1] == b'*'
            {
                self.position += 2;
                while self.position + 1 < len
                    && !(bytes[self.position] == b'*'
                        && bytes[self.position + 1] == b'/')
                {
                    self.position += 1;
                }
                if self.position + 1 < len {
                    self.position += 2;
                }
                continue;
            }
            if self.position == before {
                break;
            }
        }
    }
    fn looks_like_rule_definition_boundary(&self) -> bool {
        let bytes = self.input.as_bytes();
        let len = bytes.len();
        let mut i = self.position;
        let mut saw_newline = false;
        while i < len {
            match bytes[i] {
                b' ' | b'\t' => {
                    i += 1;
                    continue;
                }
                b'\n' | b'\r' => {
                    saw_newline = true;
                    i += 1;
                    continue;
                }
                b'#' => {
                    while i < len && bytes[i] != b'\n' && bytes[i] != b'\r' {
                        i += 1;
                    }
                    continue;
                }
                b'/' if i + 1 < len && bytes[i + 1] == b'/' => {
                    i += 2;
                    while i < len && bytes[i] != b'\n' && bytes[i] != b'\r' {
                        i += 1;
                    }
                    continue;
                }
                b'/' if i + 1 < len && bytes[i + 1] == b'*' => {
                    i += 2;
                    while i + 1 < len && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                        i += 1;
                    }
                    if i + 1 < len {
                        i += 2;
                    }
                    continue;
                }
                _ => {
                    break;
                }
            }
        }
        if !saw_newline || i >= len {
            return false;
        }
        let is_ident_start = |b: u8| b == b'_' || (b as char).is_ascii_alphabetic();
        let is_ident_continue = |b: u8| b == b'_' || (b as char).is_ascii_alphanumeric();
        if !is_ident_start(bytes[i]) {
            return false;
        }
        i += 1;
        while i < len && is_ident_continue(bytes[i]) {
            i += 1;
        }
        while i < len && matches!(bytes[i], b' ' | b'\t') {
            i += 1;
        }
        (i + 2 <= len && &bytes[i..i + 2] == b":=")
            || (i + 3 <= len && &bytes[i..i + 3] == b"::=")
            || (i + 2 <= len && &bytes[i..i + 2] == b":-")
            || (i + 1 <= len && bytes[i] == b'=')
    }
    fn match_string(&mut self, expected: &str) -> ParseResult<&'input str> {
        self.consume_layout_for_terminal(expected);
        let start = self.position;
        let expected_bytes = expected.as_bytes();
        let end = start + expected_bytes.len();
        if self.logger.is_enabled() {
            self.logger
                .log_debug(
                    "../generated/return_annotation_parser.rs",
                    0,
                    &format!(
                        "🔤 Attempting to match terminal '{}' at position {} (end: {})",
                        expected, start, end
                    ),
                );
        }
        if self.bytes_match_at(start, expected_bytes) {
            if !self.input.is_char_boundary(start) || !self.input.is_char_boundary(end) {
                return Err(
                    self
                        .create_contextual_error(
                            &format!(
                                "Internal UTF-8 boundary mismatch while matching '{}'",
                                expected
                            ),
                        ),
                );
            }
            self.position = end;
            if self.logger.is_enabled() {
                self.logger
                    .log_success(
                        "../generated/return_annotation_parser.rs",
                        0,
                        &format!(
                            "✅ Terminal '{}' matched, advanced to position {}",
                            expected, end
                        ),
                    );
            }
            return Ok(&self.input[start..end]);
        }
        let found_str = if self.position < self.input.len() {
            let end = (self.position + expected_bytes.len()).min(self.input.len());
            self.byte_window_lossy(self.position, end)
        } else {
            "<EOF>".to_string()
        };
        if self.logger.is_enabled() {
            self.logger
                .log_error(
                    "../generated/return_annotation_parser.rs",
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
        let re = regex::Regex::new(pattern)
            .map_err(|e| {
                self
                    .create_contextual_error(
                        &format!("Invalid regex pattern '{}': {}", pattern, e),
                    )
            })?;
        if skip_leading_whitespace {
            let can_match_empty = re
                .find("")
                .map(|m| m.start() == 0 && m.end() == 0)
                .unwrap_or(false);
            self.consume_layout_for_regex(can_match_empty);
        }
        let Some(haystack) = self.input.get(self.position..) else {
            return Err(
                self
                    .create_contextual_error(
                        "Parser position is not on a UTF-8 boundary",
                    ),
            );
        };
        if let Some(mat) = re.find(haystack) {
            if mat.start() == 0 {
                let matched = mat.as_str();
                let start = self.position;
                if self.logger.is_enabled() {
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
                            0,
                            &format!(
                                "✅ Regex '{}' matched '{}' at position {}", pattern,
                                matched, start
                            ),
                        );
                }
                self.position += matched.len();
                if let Some(slice) = self.input.get(start..self.position) {
                    return Ok(slice);
                }
                return Err(
                    self.create_contextual_error("Regex matched invalid UTF-8 span"),
                );
            }
        }
        if self.logger.is_enabled() {
            let preview = if self.position < self.input.len() {
                let end = (self.position + 10).min(self.input.len());
                self.byte_window_lossy(self.position, end)
            } else {
                "<EOF>".to_string()
            };
            self.logger
                .log_error(
                    "../generated/return_annotation_parser.rs",
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
                    "../generated/return_annotation_parser.rs",
                    0,
                    &format!("🔄 Starting speculative parse at position {}", saved_pos),
                );
        }
        match f(self) {
            Ok(result) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_success(
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                            "../generated/return_annotation_parser.rs",
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
                    "../generated/return_annotation_parser.rs",
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
                        "../generated/return_annotation_parser.rs",
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
                        "../generated/return_annotation_parser.rs",
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
        let input_context = self.byte_window_lossy(start, end);
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
        let logger = Box::new(crate::ast_pipeline::NoOpLogger);
        let mut parser = ReturnAnnotationParser::new(input, logger);
        let result = parser.parse();
        assert!(result.is_ok());
    }
}
