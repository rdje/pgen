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
pub struct RtlConstExprParser<'input> {
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
impl<'input> RtlConstExprParser<'input> {
    const RULE_RTL_CONST_EXPR: RuleId = 0u16;
    const RULE_CONDITIONAL_EXPR: RuleId = 1u16;
    const RULE_LOGICAL_OR_EXPR: RuleId = 2u16;
    const RULE_LOGICAL_AND_EXPR: RuleId = 3u16;
    const RULE_BIT_OR_EXPR: RuleId = 4u16;
    const RULE_BIT_XOR_EXPR: RuleId = 5u16;
    const RULE_BIT_AND_EXPR: RuleId = 6u16;
    const RULE_EQUALITY_EXPR: RuleId = 7u16;
    const RULE_RELATIONAL_EXPR: RuleId = 8u16;
    const RULE_SHIFT_EXPR: RuleId = 9u16;
    const RULE_ADDITIVE_EXPR: RuleId = 10u16;
    const RULE_MULTIPLICATIVE_EXPR: RuleId = 11u16;
    const RULE_UNARY_EXPR: RuleId = 12u16;
    const RULE_PRIMARY_EXPR: RuleId = 13u16;
    const RULE_LITERAL: RuleId = 14u16;
    const RULE_BASED_INTEGER: RuleId = 15u16;
    const RULE_DECIMAL_INTEGER: RuleId = 16u16;
    const RULE_IDENTIFIER: RuleId = 17u16;
    const RULE_TRIVIA: RuleId = 18u16;
    const RULE_QUESTION: RuleId = 19u16;
    const RULE_COLON: RuleId = 20u16;
    const RULE_LOGICAL_OR: RuleId = 21u16;
    const RULE_LOGICAL_AND: RuleId = 22u16;
    const RULE_BIT_OR: RuleId = 23u16;
    const RULE_BIT_XOR: RuleId = 24u16;
    const RULE_BIT_AND: RuleId = 25u16;
    const RULE_EQEQ: RuleId = 26u16;
    const RULE_NE: RuleId = 27u16;
    const RULE_LE: RuleId = 28u16;
    const RULE_LT: RuleId = 29u16;
    const RULE_GE: RuleId = 30u16;
    const RULE_GT: RuleId = 31u16;
    const RULE_SHL: RuleId = 32u16;
    const RULE_SHR: RuleId = 33u16;
    const RULE_PLUS: RuleId = 34u16;
    const RULE_MINUS: RuleId = 35u16;
    const RULE_STAR: RuleId = 36u16;
    const RULE_SLASH: RuleId = 37u16;
    const RULE_PERCENT: RuleId = 38u16;
    const RULE_BANG: RuleId = 39u16;
    const RULE_TILDE: RuleId = 40u16;
    const RULE_LPAREN: RuleId = 41u16;
    const RULE_RPAREN: RuleId = 42u16;
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
        self.parse_rtl_const_expr()
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
    pub fn parse_full_rtl_const_expr(&mut self) -> ParseResult<ParseNode<'input>> {
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
    pub fn parse_rtl_const_expr(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("rtl_const_expr", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "rtl_const_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "rtl_const_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "rtl_const_expr", position, depth
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
        self.recursion_guard.enter("rtl_const_expr", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_RTL_CONST_EXPR,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let result = ParseContent::Alternative(
                        Box::new(parser.parse_conditional_expr()?),
                    );
                    let end_pos = parser.position;
                    parser
                        .record_coverage_target_event(
                            "rtl_const_expr",
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
                            "rtl_const_expr",
                            "rule.rtl_const_expr",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "rtl_const_expr",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "rtl_const_expr",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "rtl_const_expr", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "rtl_const_expr", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "rtl_const_expr", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "rtl_const_expr",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "rtl_const_expr", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_conditional_expr(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("conditional_expr", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "conditional_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "conditional_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "conditional_expr", position, depth
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
        self.recursion_guard.enter("conditional_expr", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_CONDITIONAL_EXPR,
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
                            "conditional_expr",
                            "rule.conditional_expr",
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
                                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                            1usize, 2usize, "conditional_expr", parser.position
                                                        ),
                                                    );
                                            }
                                            let mut sequence_elements = Vec::with_capacity(5usize);
                                            {
                                                let element_start = parser.position;
                                                let element_content = {
                                                    let result = ParseContent::Alternative(
                                                        Box::new(parser.parse_logical_or_expr()?),
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
                                                        Box::new(parser.parse_question()?),
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
                                                    let result = ParseContent::Alternative(
                                                        Box::new(parser.parse_conditional_expr()?),
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
                                                let element_content = {
                                                    let result = ParseContent::Alternative(
                                                        Box::new(parser.parse_colon()?),
                                                    );
                                                    result
                                                };
                                                let element_end = parser.position;
                                                sequence_elements
                                                    .push(ParseNode {
                                                        rule_name: "element_3",
                                                        content: element_content,
                                                        span: element_start..element_end,
                                                    });
                                            }
                                            {
                                                let element_start = parser.position;
                                                let element_content = {
                                                    let result = ParseContent::Alternative(
                                                        Box::new(parser.parse_conditional_expr()?),
                                                    );
                                                    result
                                                };
                                                let element_end = parser.position;
                                                sequence_elements
                                                    .push(ParseNode {
                                                        rule_name: "element_4",
                                                        content: element_content,
                                                        span: element_start..element_end,
                                                    });
                                            }
                                            let result = ParseContent::Sequence(sequence_elements);
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                        0,
                                                        &format!(
                                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                            1usize, 2usize, "conditional_expr", parser.position
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
                                                json_obj["condition"] = serde_json::json!(
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
                                                json_obj["else_expr"] = serde_json::json!(
                                                    ({ let __pgen_content = { let __pgen_base = (content)
                                                    .clone(); match __pgen_base {
                                                    ParseContent::Sequence(elements) if elements.len() > 4usize
                                                    => { elements[4usize].content.clone() }
                                                    ParseContent::Quantified(elements, _) if elements.len() >
                                                    4usize => { elements[4usize].content.clone() } _ =>
                                                    ParseContent::Terminal("<invalid_sequence_access>"), } };
                                                    match __pgen_content { ParseContent::Terminal(s) => s
                                                    .to_string(), ParseContent::TransformedTerminal(s) => s,
                                                    ParseContent::Alternative(node) => { match node.content {
                                                    ParseContent::Terminal(s) => s.to_string(),
                                                    ParseContent::TransformedTerminal(s) => s, other =>
                                                    format!("{:?}", other), } } other => format!("{:?}", other),
                                                    } })
                                                );
                                                json_obj["then_expr"] = serde_json::json!(
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
                                                json_obj["type"] = serde_json::json!(("ternary"));
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
                                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    1usize, 2usize, "conditional_expr", parser.position
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
                                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                            2usize, 2usize, "conditional_expr", parser.position
                                                        ),
                                                    );
                                            }
                                            let result = ParseContent::Alternative(
                                                Box::new(parser.parse_logical_or_expr()?),
                                            );
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                        0,
                                                        &format!(
                                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                            2usize, 2usize, "conditional_expr", parser.position
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
                                            {
                                                let __pgen_base = (content).clone();
                                                match __pgen_base {
                                                    ParseContent::Sequence(elements) if !elements.is_empty() => {
                                                        elements[0usize].content.clone()
                                                    }
                                                    ParseContent::Quantified(
                                                        elements,
                                                        _,
                                                    ) if !elements.is_empty() => {
                                                        elements[0usize].content.clone()
                                                    }
                                                    ParseContent::Alternative(node) => node.content.clone(),
                                                    other => other,
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
                                            best_branch = 2usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    2usize, 2usize, "conditional_expr", parser.position
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
                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                    0,
                                    &format!(
                                        "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                        "conditional_expr", best_branch, 2usize, best_end
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
                            "conditional_expr",
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
                            "conditional_expr",
                            "rule.conditional_expr",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "conditional_expr",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "conditional_expr",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "conditional_expr", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "conditional_expr", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "conditional_expr", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "conditional_expr",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "conditional_expr", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_logical_or_expr(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("logical_or_expr", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "logical_or_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "logical_or_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "logical_or_expr", position, depth
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
        self.recursion_guard.enter("logical_or_expr", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_LOGICAL_OR_EXPR,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_logical_and_expr()?),
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
                                                let result = ParseContent::Alternative(
                                                    Box::new(parser.parse_logical_or()?),
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
                                                    Box::new(parser.parse_logical_and_expr()?),
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
                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                            "logical_or_expr",
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
                            "logical_or_expr",
                            "rule.logical_or_expr",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "logical_or_expr",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "logical_or_expr",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "logical_or_expr", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "logical_or_expr", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "logical_or_expr", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "logical_or_expr",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "logical_or_expr", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_logical_and_expr(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("logical_and_expr", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "logical_and_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "logical_and_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "logical_and_expr", position, depth
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
        self.recursion_guard.enter("logical_and_expr", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_LOGICAL_AND_EXPR,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_bit_or_expr()?),
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
                                                let result = ParseContent::Alternative(
                                                    Box::new(parser.parse_logical_and()?),
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
                                                    Box::new(parser.parse_bit_or_expr()?),
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
                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                            "logical_and_expr",
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
                            "logical_and_expr",
                            "rule.logical_and_expr",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "logical_and_expr",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "logical_and_expr",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "logical_and_expr", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "logical_and_expr", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "logical_and_expr", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "logical_and_expr",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "logical_and_expr", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_bit_or_expr(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("bit_or_expr", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "bit_or_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "bit_or_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "bit_or_expr", position, depth
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
        self.recursion_guard.enter("bit_or_expr", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_BIT_OR_EXPR,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_bit_xor_expr()?),
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
                                                let result = ParseContent::Alternative(
                                                    Box::new(parser.parse_bit_or()?),
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
                                                    Box::new(parser.parse_bit_xor_expr()?),
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
                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                            "bit_or_expr",
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
                            "bit_or_expr",
                            "rule.bit_or_expr",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "bit_or_expr",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "bit_or_expr",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "bit_or_expr", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "bit_or_expr", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "bit_or_expr", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "bit_or_expr",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "bit_or_expr", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_bit_xor_expr(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("bit_xor_expr", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "bit_xor_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "bit_xor_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "bit_xor_expr", position, depth
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
        self.recursion_guard.enter("bit_xor_expr", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_BIT_XOR_EXPR,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_bit_and_expr()?),
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
                                                let result = ParseContent::Alternative(
                                                    Box::new(parser.parse_bit_xor()?),
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
                                                    Box::new(parser.parse_bit_and_expr()?),
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
                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                            "bit_xor_expr",
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
                            "bit_xor_expr",
                            "rule.bit_xor_expr",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "bit_xor_expr",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "bit_xor_expr",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "bit_xor_expr", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "bit_xor_expr", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "bit_xor_expr", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "bit_xor_expr",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "bit_xor_expr", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_bit_and_expr(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("bit_and_expr", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "bit_and_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "bit_and_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "bit_and_expr", position, depth
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
        self.recursion_guard.enter("bit_and_expr", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_BIT_AND_EXPR,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_equality_expr()?),
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
                                                let result = ParseContent::Alternative(
                                                    Box::new(parser.parse_bit_and()?),
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
                                                    Box::new(parser.parse_equality_expr()?),
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
                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                            "bit_and_expr",
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
                            "bit_and_expr",
                            "rule.bit_and_expr",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "bit_and_expr",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "bit_and_expr",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "bit_and_expr", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "bit_and_expr", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "bit_and_expr", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "bit_and_expr",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "bit_and_expr", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_equality_expr(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("equality_expr", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "equality_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "equality_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "equality_expr", position, depth
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
        self.recursion_guard.enter("equality_expr", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_EQUALITY_EXPR,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_relational_expr()?),
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
                                                        "equality_expr",
                                                        "rule.equality_expr",
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
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                                        1usize, 2usize, "equality_expr", parser.position
                                                                                    ),
                                                                                );
                                                                        }
                                                                        let result = ParseContent::Alternative(
                                                                            Box::new(parser.parse_eqeq()?),
                                                                        );
                                                                        if parser.logger.is_enabled() {
                                                                            parser
                                                                                .logger
                                                                                .log_info(
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                                        1usize, 2usize, "equality_expr", parser.position
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
                                                                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                            0,
                                                                            &format!(
                                                                                "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                                                1usize, 2usize, "equality_expr", parser.position
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
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                                        2usize, 2usize, "equality_expr", parser.position
                                                                                    ),
                                                                                );
                                                                        }
                                                                        let result = ParseContent::Alternative(
                                                                            Box::new(parser.parse_ne()?),
                                                                        );
                                                                        if parser.logger.is_enabled() {
                                                                            parser
                                                                                .logger
                                                                                .log_info(
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                                        2usize, 2usize, "equality_expr", parser.position
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
                                                                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                            0,
                                                                            &format!(
                                                                                "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                                                2usize, 2usize, "equality_expr", parser.position
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
                                                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                0,
                                                                &format!(
                                                                    "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                                                    "equality_expr", best_branch, 2usize, best_end
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
                                                    Box::new(parser.parse_relational_expr()?),
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
                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                            "equality_expr",
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
                            "equality_expr",
                            "rule.equality_expr",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "equality_expr",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "equality_expr",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "equality_expr", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "equality_expr", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "equality_expr", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "equality_expr",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "equality_expr", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_relational_expr(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("relational_expr", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "relational_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "relational_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "relational_expr", position, depth
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
        self.recursion_guard.enter("relational_expr", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_RELATIONAL_EXPR,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_shift_expr()?),
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
                                                        "relational_expr",
                                                        "rule.relational_expr",
                                                    );
                                                let deterministic_partition_offset = if deterministic_partition_effective_enabled {
                                                    parser
                                                        .deterministic_partition_offset_runtime(
                                                            &deterministic_partition_effective_group,
                                                            4usize,
                                                        )
                                                } else {
                                                    0usize
                                                };
                                                let mut evaluation_order: Vec<usize> = (0..4usize)
                                                    .collect();
                                                if deterministic_partition_effective_enabled && 4usize > 1
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
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                                        1usize, 4usize, "relational_expr", parser.position
                                                                                    ),
                                                                                );
                                                                        }
                                                                        let result = ParseContent::Alternative(
                                                                            Box::new(parser.parse_le()?),
                                                                        );
                                                                        if parser.logger.is_enabled() {
                                                                            parser
                                                                                .logger
                                                                                .log_info(
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                                        1usize, 4usize, "relational_expr", parser.position
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
                                                                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                            0,
                                                                            &format!(
                                                                                "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                                                1usize, 4usize, "relational_expr", parser.position
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
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                                        2usize, 4usize, "relational_expr", parser.position
                                                                                    ),
                                                                                );
                                                                        }
                                                                        let result = ParseContent::Alternative(
                                                                            Box::new(parser.parse_lt()?),
                                                                        );
                                                                        if parser.logger.is_enabled() {
                                                                            parser
                                                                                .logger
                                                                                .log_info(
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                                        2usize, 4usize, "relational_expr", parser.position
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
                                                                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                            0,
                                                                            &format!(
                                                                                "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                                                2usize, 4usize, "relational_expr", parser.position
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
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                                        3usize, 4usize, "relational_expr", parser.position
                                                                                    ),
                                                                                );
                                                                        }
                                                                        let result = ParseContent::Alternative(
                                                                            Box::new(parser.parse_ge()?),
                                                                        );
                                                                        if parser.logger.is_enabled() {
                                                                            parser
                                                                                .logger
                                                                                .log_info(
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                                        3usize, 4usize, "relational_expr", parser.position
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
                                                                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                            0,
                                                                            &format!(
                                                                                "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                                                3usize, 4usize, "relational_expr", parser.position
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
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                                        4usize, 4usize, "relational_expr", parser.position
                                                                                    ),
                                                                                );
                                                                        }
                                                                        let result = ParseContent::Alternative(
                                                                            Box::new(parser.parse_gt()?),
                                                                        );
                                                                        if parser.logger.is_enabled() {
                                                                            parser
                                                                                .logger
                                                                                .log_info(
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                                        4usize, 4usize, "relational_expr", parser.position
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
                                                                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                            0,
                                                                            &format!(
                                                                                "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                                                4usize, 4usize, "relational_expr", parser.position
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
                                                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                0,
                                                                &format!(
                                                                    "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                                                    "relational_expr", best_branch, 4usize, best_end
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
                                                    Box::new(parser.parse_shift_expr()?),
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
                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                            "relational_expr",
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
                            "relational_expr",
                            "rule.relational_expr",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "relational_expr",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "relational_expr",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "relational_expr", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "relational_expr", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "relational_expr", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "relational_expr",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "relational_expr", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_shift_expr(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("shift_expr", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "shift_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "shift_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "shift_expr", position, depth
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
        self.recursion_guard.enter("shift_expr", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_SHIFT_EXPR,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_additive_expr()?),
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
                                                        "shift_expr",
                                                        "rule.shift_expr",
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
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                                        1usize, 2usize, "shift_expr", parser.position
                                                                                    ),
                                                                                );
                                                                        }
                                                                        let result = ParseContent::Alternative(
                                                                            Box::new(parser.parse_shl()?),
                                                                        );
                                                                        if parser.logger.is_enabled() {
                                                                            parser
                                                                                .logger
                                                                                .log_info(
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                                        1usize, 2usize, "shift_expr", parser.position
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
                                                                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                            0,
                                                                            &format!(
                                                                                "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                                                1usize, 2usize, "shift_expr", parser.position
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
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                                        2usize, 2usize, "shift_expr", parser.position
                                                                                    ),
                                                                                );
                                                                        }
                                                                        let result = ParseContent::Alternative(
                                                                            Box::new(parser.parse_shr()?),
                                                                        );
                                                                        if parser.logger.is_enabled() {
                                                                            parser
                                                                                .logger
                                                                                .log_info(
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                                        2usize, 2usize, "shift_expr", parser.position
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
                                                                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                            0,
                                                                            &format!(
                                                                                "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                                                2usize, 2usize, "shift_expr", parser.position
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
                                                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                0,
                                                                &format!(
                                                                    "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                                                    "shift_expr", best_branch, 2usize, best_end
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
                                                    Box::new(parser.parse_additive_expr()?),
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
                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                            "shift_expr",
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
                            "shift_expr",
                            "rule.shift_expr",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "shift_expr",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "shift_expr",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "shift_expr", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "shift_expr", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "shift_expr", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "shift_expr",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "shift_expr", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_additive_expr(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("additive_expr", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "additive_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "additive_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "additive_expr", position, depth
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
        self.recursion_guard.enter("additive_expr", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_ADDITIVE_EXPR,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_multiplicative_expr()?),
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
                                                        "additive_expr",
                                                        "rule.additive_expr",
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
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                                        1usize, 2usize, "additive_expr", parser.position
                                                                                    ),
                                                                                );
                                                                        }
                                                                        let result = ParseContent::Alternative(
                                                                            Box::new(parser.parse_plus()?),
                                                                        );
                                                                        if parser.logger.is_enabled() {
                                                                            parser
                                                                                .logger
                                                                                .log_info(
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                                        1usize, 2usize, "additive_expr", parser.position
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
                                                                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                            0,
                                                                            &format!(
                                                                                "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                                                1usize, 2usize, "additive_expr", parser.position
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
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                                        2usize, 2usize, "additive_expr", parser.position
                                                                                    ),
                                                                                );
                                                                        }
                                                                        let result = ParseContent::Alternative(
                                                                            Box::new(parser.parse_minus()?),
                                                                        );
                                                                        if parser.logger.is_enabled() {
                                                                            parser
                                                                                .logger
                                                                                .log_info(
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                                        2usize, 2usize, "additive_expr", parser.position
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
                                                                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                            0,
                                                                            &format!(
                                                                                "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                                                2usize, 2usize, "additive_expr", parser.position
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
                                                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                0,
                                                                &format!(
                                                                    "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                                                    "additive_expr", best_branch, 2usize, best_end
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
                                                    Box::new(parser.parse_multiplicative_expr()?),
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
                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                            "additive_expr",
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
                            "additive_expr",
                            "rule.additive_expr",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "additive_expr",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "additive_expr",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "additive_expr", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "additive_expr", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "additive_expr", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "additive_expr",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "additive_expr", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_multiplicative_expr(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self
            .recursion_guard
            .check_cycle("multiplicative_expr", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "multiplicative_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "multiplicative_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "multiplicative_expr", position, depth
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
        self.recursion_guard.enter("multiplicative_expr", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_MULTIPLICATIVE_EXPR,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_unary_expr()?),
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
                                                        "multiplicative_expr",
                                                        "rule.multiplicative_expr",
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
                                                let mut evaluation_order: Vec<usize> = (0..3usize)
                                                    .collect();
                                                if deterministic_partition_effective_enabled && 3usize > 1
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
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                                        1usize, 3usize, "multiplicative_expr", parser.position
                                                                                    ),
                                                                                );
                                                                        }
                                                                        let result = ParseContent::Alternative(
                                                                            Box::new(parser.parse_star()?),
                                                                        );
                                                                        if parser.logger.is_enabled() {
                                                                            parser
                                                                                .logger
                                                                                .log_info(
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                                        1usize, 3usize, "multiplicative_expr", parser.position
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
                                                                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                            0,
                                                                            &format!(
                                                                                "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                                                1usize, 3usize, "multiplicative_expr", parser.position
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
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                                        2usize, 3usize, "multiplicative_expr", parser.position
                                                                                    ),
                                                                                );
                                                                        }
                                                                        let result = ParseContent::Alternative(
                                                                            Box::new(parser.parse_slash()?),
                                                                        );
                                                                        if parser.logger.is_enabled() {
                                                                            parser
                                                                                .logger
                                                                                .log_info(
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                                        2usize, 3usize, "multiplicative_expr", parser.position
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
                                                                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                            0,
                                                                            &format!(
                                                                                "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                                                2usize, 3usize, "multiplicative_expr", parser.position
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
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                                        3usize, 3usize, "multiplicative_expr", parser.position
                                                                                    ),
                                                                                );
                                                                        }
                                                                        let result = ParseContent::Alternative(
                                                                            Box::new(parser.parse_percent()?),
                                                                        );
                                                                        if parser.logger.is_enabled() {
                                                                            parser
                                                                                .logger
                                                                                .log_info(
                                                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                                    0,
                                                                                    &format!(
                                                                                        "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                                        3usize, 3usize, "multiplicative_expr", parser.position
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
                                                                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                            0,
                                                                            &format!(
                                                                                "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                                                3usize, 3usize, "multiplicative_expr", parser.position
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
                                                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                                0,
                                                                &format!(
                                                                    "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                                                    "multiplicative_expr", best_branch, 3usize, best_end
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
                                                    Box::new(parser.parse_unary_expr()?),
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
                                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                            "multiplicative_expr",
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
                            "multiplicative_expr",
                            "rule.multiplicative_expr",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "multiplicative_expr",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "multiplicative_expr",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "multiplicative_expr", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "multiplicative_expr", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "multiplicative_expr", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "multiplicative_expr",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "multiplicative_expr", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_unary_expr(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("unary_expr", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "unary_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "unary_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "unary_expr", position, depth
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
        self.recursion_guard.enter("unary_expr", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_UNARY_EXPR,
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
                            "unary_expr",
                            "rule.unary_expr",
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
                                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                            1usize, 5usize, "unary_expr", parser.position
                                                        ),
                                                    );
                                            }
                                            let mut sequence_elements = Vec::with_capacity(2usize);
                                            {
                                                let element_start = parser.position;
                                                let element_content = {
                                                    let result = ParseContent::Alternative(
                                                        Box::new(parser.parse_plus()?),
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
                                                        Box::new(parser.parse_unary_expr()?),
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
                                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                        0,
                                                        &format!(
                                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                            1usize, 5usize, "unary_expr", parser.position
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
                                                json_obj["expr"] = serde_json::json!(
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
                                                json_obj["op"] = serde_json::json!(("plus"));
                                                json_obj["type"] = serde_json::json!(("unary"));
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
                                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    1usize, 5usize, "unary_expr", parser.position
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
                                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                            2usize, 5usize, "unary_expr", parser.position
                                                        ),
                                                    );
                                            }
                                            let mut sequence_elements = Vec::with_capacity(2usize);
                                            {
                                                let element_start = parser.position;
                                                let element_content = {
                                                    let result = ParseContent::Alternative(
                                                        Box::new(parser.parse_minus()?),
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
                                                        Box::new(parser.parse_unary_expr()?),
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
                                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                        0,
                                                        &format!(
                                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                            2usize, 5usize, "unary_expr", parser.position
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
                                            {
                                                let mut json_obj = serde_json::json!({});
                                                json_obj["expr"] = serde_json::json!(
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
                                                json_obj["op"] = serde_json::json!(("minus"));
                                                json_obj["type"] = serde_json::json!(("unary"));
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
                                            best_branch = 2usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    2usize, 5usize, "unary_expr", parser.position
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
                                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                            3usize, 5usize, "unary_expr", parser.position
                                                        ),
                                                    );
                                            }
                                            let mut sequence_elements = Vec::with_capacity(2usize);
                                            {
                                                let element_start = parser.position;
                                                let element_content = {
                                                    let result = ParseContent::Alternative(
                                                        Box::new(parser.parse_bang()?),
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
                                                        Box::new(parser.parse_unary_expr()?),
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
                                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                        0,
                                                        &format!(
                                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                            3usize, 5usize, "unary_expr", parser.position
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
                                                let mut json_obj = serde_json::json!({});
                                                json_obj["expr"] = serde_json::json!(
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
                                                json_obj["op"] = serde_json::json!(("logical_not"));
                                                json_obj["type"] = serde_json::json!(("unary"));
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
                                            best_branch = 3usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    3usize, 5usize, "unary_expr", parser.position
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
                                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                            4usize, 5usize, "unary_expr", parser.position
                                                        ),
                                                    );
                                            }
                                            let mut sequence_elements = Vec::with_capacity(2usize);
                                            {
                                                let element_start = parser.position;
                                                let element_content = {
                                                    let result = ParseContent::Alternative(
                                                        Box::new(parser.parse_tilde()?),
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
                                                        Box::new(parser.parse_unary_expr()?),
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
                                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                        0,
                                                        &format!(
                                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                            4usize, 5usize, "unary_expr", parser.position
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
                                            {
                                                let mut json_obj = serde_json::json!({});
                                                json_obj["expr"] = serde_json::json!(
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
                                                json_obj["op"] = serde_json::json!(("bit_not"));
                                                json_obj["type"] = serde_json::json!(("unary"));
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
                                            best_branch = 4usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    4usize, 5usize, "unary_expr", parser.position
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
                                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                            5usize, 5usize, "unary_expr", parser.position
                                                        ),
                                                    );
                                            }
                                            let result = ParseContent::Alternative(
                                                Box::new(parser.parse_primary_expr()?),
                                            );
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                        0,
                                                        &format!(
                                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                            5usize, 5usize, "unary_expr", parser.position
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
                                                    ParseContent::Sequence(elements) if !elements.is_empty() => {
                                                        elements[0usize].content.clone()
                                                    }
                                                    ParseContent::Quantified(
                                                        elements,
                                                        _,
                                                    ) if !elements.is_empty() => {
                                                        elements[0usize].content.clone()
                                                    }
                                                    ParseContent::Alternative(node) => node.content.clone(),
                                                    other => other,
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
                                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    5usize, 5usize, "unary_expr", parser.position
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
                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                    0,
                                    &format!(
                                        "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                        "unary_expr", best_branch, 5usize, best_end
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
                            "unary_expr",
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
                            "unary_expr",
                            "rule.unary_expr",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "unary_expr",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "unary_expr",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "unary_expr", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "unary_expr", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "unary_expr", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "unary_expr",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "unary_expr", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_primary_expr(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("primary_expr", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "primary_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "primary_expr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "primary_expr", position, depth
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
        self.recursion_guard.enter("primary_expr", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_PRIMARY_EXPR,
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
                            "primary_expr",
                            "rule.primary_expr",
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
                                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                            1usize, 3usize, "primary_expr", parser.position
                                                        ),
                                                    );
                                            }
                                            let result = ParseContent::Alternative(
                                                Box::new(parser.parse_literal()?),
                                            );
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                        0,
                                                        &format!(
                                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                            1usize, 3usize, "primary_expr", parser.position
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
                                                let __pgen_base = (content).clone();
                                                match __pgen_base {
                                                    ParseContent::Sequence(elements) if !elements.is_empty() => {
                                                        elements[0usize].content.clone()
                                                    }
                                                    ParseContent::Quantified(
                                                        elements,
                                                        _,
                                                    ) if !elements.is_empty() => {
                                                        elements[0usize].content.clone()
                                                    }
                                                    ParseContent::Alternative(node) => node.content.clone(),
                                                    other => other,
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
                                            best_branch = 1usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    1usize, 3usize, "primary_expr", parser.position
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
                                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                            2usize, 3usize, "primary_expr", parser.position
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
                                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                        0,
                                                        &format!(
                                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                            2usize, 3usize, "primary_expr", parser.position
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
                                            {
                                                let __pgen_base = (content).clone();
                                                match __pgen_base {
                                                    ParseContent::Sequence(elements) if !elements.is_empty() => {
                                                        elements[0usize].content.clone()
                                                    }
                                                    ParseContent::Quantified(
                                                        elements,
                                                        _,
                                                    ) if !elements.is_empty() => {
                                                        elements[0usize].content.clone()
                                                    }
                                                    ParseContent::Alternative(node) => node.content.clone(),
                                                    other => other,
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
                                            best_branch = 2usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    2usize, 3usize, "primary_expr", parser.position
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
                                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                            3usize, 3usize, "primary_expr", parser.position
                                                        ),
                                                    );
                                            }
                                            let mut sequence_elements = Vec::with_capacity(3usize);
                                            {
                                                let element_start = parser.position;
                                                let element_content = {
                                                    let result = ParseContent::Alternative(
                                                        Box::new(parser.parse_lparen()?),
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
                                                        Box::new(parser.parse_conditional_expr()?),
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
                                                    let result = ParseContent::Alternative(
                                                        Box::new(parser.parse_rparen()?),
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
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                        0,
                                                        &format!(
                                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                            3usize, 3usize, "primary_expr", parser.position
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
                                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    3usize, 3usize, "primary_expr", parser.position
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
                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                    0,
                                    &format!(
                                        "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                        "primary_expr", best_branch, 3usize, best_end
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
                            "primary_expr",
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
                            "primary_expr",
                            "rule.primary_expr",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "primary_expr",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "primary_expr",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "primary_expr", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "primary_expr", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "primary_expr", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "primary_expr",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "primary_expr", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_literal(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("literal", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "literal", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "literal", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "literal", position, depth
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
        self.recursion_guard.enter("literal", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_LITERAL,
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
                            "literal",
                            "rule.literal",
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
                                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                            1usize, 2usize, "literal", parser.position
                                                        ),
                                                    );
                                            }
                                            let result = ParseContent::Alternative(
                                                Box::new(parser.parse_based_integer()?),
                                            );
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                        0,
                                                        &format!(
                                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                            1usize, 2usize, "literal", parser.position
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
                                                json_obj["kind"] = serde_json::json!(("based"));
                                                json_obj["text"] = serde_json::json!(
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
                                                json_obj["type"] = serde_json::json!(("literal"));
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
                                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    1usize, 2usize, "literal", parser.position
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
                                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                            2usize, 2usize, "literal", parser.position
                                                        ),
                                                    );
                                            }
                                            let result = ParseContent::Alternative(
                                                Box::new(parser.parse_decimal_integer()?),
                                            );
                                            if parser.logger.is_enabled() {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                        0,
                                                        &format!(
                                                            "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                            2usize, 2usize, "literal", parser.position
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
                                            {
                                                let mut json_obj = serde_json::json!({});
                                                json_obj["kind"] = serde_json::json!(("decimal"));
                                                json_obj["text"] = serde_json::json!(
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
                                                json_obj["type"] = serde_json::json!(("literal"));
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
                                            best_branch = 2usize;
                                            best_content = Some(transformed);
                                        }
                                    } else if parser.logger.is_enabled() {
                                        parser
                                            .logger
                                            .log_info(
                                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                                0,
                                                &format!(
                                                    "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                    2usize, 2usize, "literal", parser.position
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
                                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                    0,
                                    &format!(
                                        "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                        "literal", best_branch, 2usize, best_end
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
                            "literal",
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
                            "literal",
                            "rule.literal",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "literal",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "literal",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "literal", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "literal", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "literal", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "literal",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "literal", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_based_integer(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("based_integer", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "based_integer", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "based_integer", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "based_integer", position, depth
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
        self.recursion_guard.enter("based_integer", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_BASED_INTEGER,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            let matched_str = parser
                                .match_regex(
                                    "[0-9][0-9_]*'[sS]?[bBoOdDhH][0-9a-fA-F_]+",
                                    true,
                                )?;
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
                            "based_integer",
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
                            "based_integer",
                            "rule.based_integer",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "based_integer",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "based_integer",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "based_integer", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "based_integer", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "based_integer", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "based_integer",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "based_integer", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_decimal_integer(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("decimal_integer", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "decimal_integer", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "decimal_integer", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "decimal_integer", position, depth
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
        self.recursion_guard.enter("decimal_integer", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_DECIMAL_INTEGER,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            let matched_str = parser.match_regex("[0-9][0-9_]*", true)?;
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
                            "decimal_integer",
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
                            "decimal_integer",
                            "rule.decimal_integer",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "decimal_integer",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "decimal_integer",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "decimal_integer", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "decimal_integer", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "decimal_integer", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "decimal_integer",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "decimal_integer", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_identifier(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("identifier", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            let matched_str = parser
                                .match_regex(
                                    "[_A-Za-z][_A-Za-z0-9$]*(?:(?:\\.)[_A-Za-z][_A-Za-z0-9$]*|::[_A-Za-z][_A-Za-z0-9$]*)*",
                                    true,
                                )?;
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "identifier", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
    pub fn parse_trivia(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("trivia", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "trivia", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "trivia", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "trivia", position, depth
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
        self.recursion_guard.enter("trivia", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_TRIVIA,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let matched_str = parser.match_regex("[ \\t\\r\\n]*", true)?;
                    let result = ParseContent::Terminal(matched_str);
                    let end_pos = parser.position;
                    parser
                        .record_coverage_target_event(
                            "trivia",
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
                            "trivia",
                            "rule.trivia",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "trivia",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "trivia",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "trivia", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "trivia", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "trivia", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "trivia",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "trivia", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_question(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("question", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "question", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "question", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "question", position, depth
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
        self.recursion_guard.enter("question", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_QUESTION,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            let matched_str = parser.match_string("?")?;
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
                            "question",
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
                            "question",
                            "rule.question",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "question",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "question",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "question", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "question", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "question", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "question",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "question", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_colon(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("colon", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "colon", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "colon", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "colon", position, depth
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
        self.recursion_guard.enter("colon", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_COLON,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                    let result = ParseContent::Sequence(sequence_elements);
                    let end_pos = parser.position;
                    parser
                        .record_coverage_target_event(
                            "colon",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group("colon", "rule.colon");
                    parser
                        .record_deterministic_partition_event(
                            "colon",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "colon",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "colon", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "colon", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "colon", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "colon",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "colon", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_logical_or(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("logical_or", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "logical_or", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "logical_or", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "logical_or", position, depth
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
        self.recursion_guard.enter("logical_or", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_LOGICAL_OR,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            let matched_str = parser.match_string("||")?;
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
                            "logical_or",
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
                            "logical_or",
                            "rule.logical_or",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "logical_or",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "logical_or",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "logical_or", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "logical_or", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "logical_or", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "logical_or",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "logical_or", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_logical_and(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("logical_and", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "logical_and", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "logical_and", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "logical_and", position, depth
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
        self.recursion_guard.enter("logical_and", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_LOGICAL_AND,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            let matched_str = parser.match_string("&&")?;
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
                            "logical_and",
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
                            "logical_and",
                            "rule.logical_and",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "logical_and",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "logical_and",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "logical_and", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "logical_and", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "logical_and", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "logical_and",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "logical_and", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_bit_or(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("bit_or", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "bit_or", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "bit_or", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "bit_or", position, depth
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
        self.recursion_guard.enter("bit_or", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_BIT_OR,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            let matched_str = parser.match_string("|")?;
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
                            "bit_or",
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
                            "bit_or",
                            "rule.bit_or",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "bit_or",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "bit_or",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "bit_or", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "bit_or", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "bit_or", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "bit_or",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "bit_or", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_bit_xor(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("bit_xor", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "bit_xor", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "bit_xor", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "bit_xor", position, depth
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
        self.recursion_guard.enter("bit_xor", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_BIT_XOR,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            let matched_str = parser.match_string("^")?;
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
                            "bit_xor",
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
                            "bit_xor",
                            "rule.bit_xor",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "bit_xor",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "bit_xor",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "bit_xor", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "bit_xor", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "bit_xor", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "bit_xor",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "bit_xor", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_bit_and(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("bit_and", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "bit_and", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "bit_and", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "bit_and", position, depth
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
        self.recursion_guard.enter("bit_and", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_BIT_AND,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            let matched_str = parser.match_string("&")?;
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
                            "bit_and",
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
                            "bit_and",
                            "rule.bit_and",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "bit_and",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "bit_and",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "bit_and", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "bit_and", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "bit_and", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "bit_and",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "bit_and", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_eqeq(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("eqeq", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "eqeq", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "eqeq", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "eqeq", position, depth
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
        self.recursion_guard.enter("eqeq", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_EQEQ,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            let matched_str = parser.match_string("==")?;
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
                            "eqeq",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group("eqeq", "rule.eqeq");
                    parser
                        .record_deterministic_partition_event(
                            "eqeq",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "eqeq",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "eqeq", start_pos, node.span.end, consumed, consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "eqeq", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "eqeq", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "eqeq",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "eqeq", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_ne(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("ne", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "ne", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "ne", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "ne", position, depth
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
        self.recursion_guard.enter("ne", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_NE,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            let matched_str = parser.match_string("!=")?;
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
                            "ne",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group("ne", "rule.ne");
                    parser
                        .record_deterministic_partition_event(
                            "ne",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "ne",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "ne", start_pos, node.span.end, consumed, consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "ne", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "ne", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "ne",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "ne", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_le(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("le", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "le", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "le", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "le", position, depth
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
        self.recursion_guard.enter("le", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_LE,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            let matched_str = parser.match_string("<=")?;
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
                            "le",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group("le", "rule.le");
                    parser
                        .record_deterministic_partition_event(
                            "le",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "le",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "le", start_pos, node.span.end, consumed, consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "le", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "le", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "le",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "le", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_lt(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("lt", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "lt", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "lt", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "lt", position, depth
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
        self.recursion_guard.enter("lt", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_LT,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            let matched_str = parser.match_string("<")?;
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
                            "lt",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group("lt", "rule.lt");
                    parser
                        .record_deterministic_partition_event(
                            "lt",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "lt",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "lt", start_pos, node.span.end, consumed, consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "lt", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "lt", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "lt",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "lt", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_ge(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("ge", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "ge", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "ge", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "ge", position, depth
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
        self.recursion_guard.enter("ge", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_GE,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            let matched_str = parser.match_string(">=")?;
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
                            "ge",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group("ge", "rule.ge");
                    parser
                        .record_deterministic_partition_event(
                            "ge",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "ge",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "ge", start_pos, node.span.end, consumed, consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "ge", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "ge", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "ge",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "ge", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_gt(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("gt", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "gt", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "gt", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "gt", position, depth
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
        self.recursion_guard.enter("gt", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_GT,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            let matched_str = parser.match_string(">")?;
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
                            "gt",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group("gt", "rule.gt");
                    parser
                        .record_deterministic_partition_event(
                            "gt",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "gt",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "gt", start_pos, node.span.end, consumed, consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "gt", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "gt", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "gt",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "gt", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_shl(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("shl", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "shl", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "shl", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "shl", position, depth
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
        self.recursion_guard.enter("shl", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_SHL,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            let matched_str = parser.match_string("<<")?;
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
                            "shl",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group("shl", "rule.shl");
                    parser
                        .record_deterministic_partition_event(
                            "shl",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "shl",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "shl", start_pos, node.span.end, consumed, consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "shl", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "shl", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "shl",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "shl", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_shr(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("shr", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "shr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "shr", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "shr", position, depth
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
        self.recursion_guard.enter("shr", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_SHR,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            let matched_str = parser.match_string(">>")?;
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
                            "shr",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group("shr", "rule.shr");
                    parser
                        .record_deterministic_partition_event(
                            "shr",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "shr",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "shr", start_pos, node.span.end, consumed, consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "shr", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "shr", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "shr",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "shr", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_plus(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("plus", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "plus", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "plus", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "plus", position, depth
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
        self.recursion_guard.enter("plus", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_PLUS,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            let matched_str = parser.match_string("+")?;
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
                            "plus",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group("plus", "rule.plus");
                    parser
                        .record_deterministic_partition_event(
                            "plus",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "plus",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "plus", start_pos, node.span.end, consumed, consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "plus", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "plus", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "plus",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "plus", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_minus(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("minus", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "minus", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "minus", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "minus", position, depth
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
        self.recursion_guard.enter("minus", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_MINUS,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            let matched_str = parser.match_string("-")?;
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
                            "minus",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group("minus", "rule.minus");
                    parser
                        .record_deterministic_partition_event(
                            "minus",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "minus",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "minus", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "minus", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "minus", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "minus",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "minus", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_star(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("star", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "star", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "star", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "star", position, depth
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
        self.recursion_guard.enter("star", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_STAR,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            "star",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group("star", "rule.star");
                    parser
                        .record_deterministic_partition_event(
                            "star",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "star",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "star", start_pos, node.span.end, consumed, consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "star", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "star", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "star",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "star", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_slash(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("slash", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "slash", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "slash", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "slash", position, depth
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
        self.recursion_guard.enter("slash", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_SLASH,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            let matched_str = parser.match_string("/")?;
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
                            "slash",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group("slash", "rule.slash");
                    parser
                        .record_deterministic_partition_event(
                            "slash",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "slash",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "slash", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "slash", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "slash", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "slash",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "slash", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_percent(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("percent", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "percent", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "percent", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "percent", position, depth
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
        self.recursion_guard.enter("percent", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_PERCENT,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            let matched_str = parser.match_string("%")?;
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
                            "percent",
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
                            "percent",
                            "rule.percent",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "percent",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "percent",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "percent", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "percent", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "percent", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "percent",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "percent", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_bang(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("bang", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "bang", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "bang", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "bang", position, depth
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
        self.recursion_guard.enter("bang", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_BANG,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            let matched_str = parser.match_string("!")?;
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
                            "bang",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group("bang", "rule.bang");
                    parser
                        .record_deterministic_partition_event(
                            "bang",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "bang",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "bang", start_pos, node.span.end, consumed, consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "bang", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "bang", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "bang",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "bang", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_tilde(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("tilde", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "tilde", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "tilde", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "tilde", position, depth
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
        self.recursion_guard.enter("tilde", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_TILDE,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            let matched_str = parser.match_string("~")?;
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
                            "tilde",
                            start_pos,
                            end_pos,
                            semantic_selected_branch_index,
                            0u64,
                            false,
                        );
                    let deterministic_partition_effective_enabled = parser
                        .effective_deterministic_partition_enabled(false);
                    let deterministic_partition_effective_group = parser
                        .effective_deterministic_partition_group("tilde", "rule.tilde");
                    parser
                        .record_deterministic_partition_event(
                            "tilde",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "tilde",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "tilde", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "tilde", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "tilde", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "tilde",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "tilde", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_lparen(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("lparen", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "lparen", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "lparen", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "lparen", position, depth
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
        self.recursion_guard.enter("lparen", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_LPAREN,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            let matched_str = parser.match_string("(")?;
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
                            "lparen",
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
                            "lparen",
                            "rule.lparen",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "lparen",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "lparen",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "lparen", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "lparen", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "lparen", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "lparen",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "lparen", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_rparen(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("rparen", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "rparen", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "rparen", position
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "rparen", position, depth
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
        self.recursion_guard.enter("rparen", position);
        let start_pos = self.position;
        let result = self
            .memoized_call(
                Self::RULE_RPAREN,
                |parser| {
                    let mut semantic_selected_branch_index: Option<usize> = None;
                    let mut sequence_elements = Vec::with_capacity(2usize);
                    {
                        let element_start = parser.position;
                        let element_content = {
                            let result = ParseContent::Alternative(
                                Box::new(parser.parse_trivia()?),
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
                            let matched_str = parser.match_string(")")?;
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
                            "rparen",
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
                            "rparen",
                            "rule.rparen",
                        );
                    parser
                        .record_deterministic_partition_event(
                            "rparen",
                            start_pos,
                            end_pos,
                            deterministic_partition_effective_enabled,
                            &deterministic_partition_effective_group,
                        );
                    Ok(ParseNode {
                        rule_name: "rparen",
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
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "rparen", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "rparen", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "rparen", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "rparen",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger.is_enabled() {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "rparen", e, self.position
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
                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
                    0,
                    &format!("🔄 Starting speculative parse at position {}", saved_pos),
                );
        }
        match f(self) {
            Ok(result) => {
                if self.logger.is_enabled() {
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                            "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                    "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
                        "/Users/richarddje/Documents/github/pgen/generated/rtl_const_expr_parser.rs",
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
        let mut parser = RtlConstExprParser::new(input, logger);
        let result = parser.parse();
        assert!(result.is_ok());
    }
}
