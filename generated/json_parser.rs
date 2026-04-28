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
pub struct JsonParser<'input> {
    input: &'input str,
    position: usize,
    memo: rustc_hash::FxHashMap<(RuleId, usize), MemoEntry<'input>>,
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
    semantic_runtime_annotations: crate::ast_pipeline::CompiledSemanticRuntimeAnnotations,
    semantic_runtime_state: crate::ast_pipeline::SemanticRuntimeState,
    logger: Box<dyn Logger>,
    logger_enabled: bool,
}
impl<'input> JsonParser<'input> {
    const RULE_JSON: RuleId = 0u16;
    const RULE_VALUE: RuleId = 1u16;
    const RULE_OBJECT: RuleId = 2u16;
    const RULE_MEMBERS: RuleId = 3u16;
    const RULE_PAIR: RuleId = 4u16;
    const RULE_ARRAY: RuleId = 5u16;
    const RULE_ELEMENTS: RuleId = 6u16;
    const RULE_STRING: RuleId = 7u16;
    const RULE_NUMBER: RuleId = 8u16;
    pub fn new(input: &'input str, logger: Box<dyn Logger>) -> Self {
        let logger_enabled = logger.is_enabled();
        Self {
            input,
            position: 0,
            memo: rustc_hash::FxHashMap::with_capacity_and_hasher(
                256,
                Default::default(),
            ),
            recursion_guard: RecursionGuard::new(4096usize),
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
            semantic_runtime_annotations: crate::ast_pipeline::CompiledSemanticRuntimeAnnotations::default(),
            semantic_runtime_state: crate::ast_pipeline::SemanticRuntimeState::new(),
            logger,
            logger_enabled,
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
        self.semantic_runtime_state = crate::ast_pipeline::SemanticRuntimeState::new();
        self.parse_json()
    }
    pub fn parse_full(&mut self) -> ParseResult<ParseNode<'input>> {
        let parsed = self.parse()?;
        if true {
            self.consume_layout_for_terminal("<EOF>");
        }
        if self.position == self.input.len() {
            Ok(parsed)
        } else {
            Err(ParseError::InvalidSyntax {
                message: "Parser did not consume full input",
                position: self.position,
            })
        }
    }
    pub fn parse_full_json(&mut self) -> ParseResult<ParseNode<'input>> {
        self.parse_full()
    }
    pub fn set_grammar_profile(&mut self, profile: Option<&str>) {
        self.grammar_profile = profile.map(|value| value.to_string());
    }
    pub fn grammar_profile(&self) -> Option<&str> {
        self.grammar_profile.as_deref()
    }
    pub fn semantic_runtime_annotations(
        &self,
    ) -> &crate::ast_pipeline::CompiledSemanticRuntimeAnnotations {
        &self.semantic_runtime_annotations
    }
    pub fn semantic_runtime_state(&self) -> &crate::ast_pipeline::SemanticRuntimeState {
        &self.semantic_runtime_state
    }
    pub fn semantic_runtime_state_mut(
        &mut self,
    ) -> &mut crate::ast_pipeline::SemanticRuntimeState {
        &mut self.semantic_runtime_state
    }
    pub fn semantic_runtime_transaction_for_rule(
        &mut self,
        rule_name: &str,
    ) -> (crate::ast_pipeline::SemanticRuntimeTransaction<'_>, usize) {
        self.semantic_runtime_state
            .transaction_for_rule(&self.semantic_runtime_annotations, rule_name)
    }
    fn semantic_predicate_debug_label(
        &self,
        spec: &crate::ast_pipeline::SemanticPredicateSpec,
    ) -> String {
        format!("{} {:?}", spec.name, spec.args)
    }
    pub fn with_semantic_runtime_rule_transaction<F>(
        &mut self,
        rule_name: &str,
        f: F,
    ) -> ParseResult<ParseNode<'input>>
    where
        F: FnOnce(
            &mut Self,
        ) -> ParseResult<(ParseNode<'input>, Option<ParseContent<'input>>)>,
    {
        if self.semantic_runtime_annotations.is_empty()
            || !self.semantic_runtime_annotations.has_rule(rule_name)
        {
            let (node, _raw) = f(self)?;
            return Ok(node);
        }
        let original_semantic_runtime_state = std::mem::take(
            &mut self.semantic_runtime_state,
        );
        self.semantic_runtime_state = original_semantic_runtime_state.clone();
        let result = {
            let mut predicate_blocked = false;
            for directive in self
                .semantic_runtime_annotations
                .pre_predicates_for_rule(rule_name)
            {
                match self.semantic_runtime_state.evaluate_directive_predicate(directive)
                {
                    Some(true) => {}
                    Some(false) => {
                        if self.logger_enabled {
                            if let crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                spec,
                            ) = directive {
                                self.logger
                                    .log_info(
                                        file!(),
                                        line!(),
                                        &format!(
                                            "🚫 Rule '{}' rejected by pre predicate '{}'", rule_name,
                                            self.semantic_predicate_debug_label(spec),
                                        ),
                                    );
                            }
                        }
                        predicate_blocked = true;
                        break;
                    }
                    None => {}
                }
            }
            if predicate_blocked {
                Err(ParseError::Backtrack {
                    position: self.position,
                })
            } else {
                let (node, semantic_raw_content) = f(self)?;
                let semantic_raw_content = semantic_raw_content
                    .as_ref()
                    .unwrap_or(&node.content);
                let mut semantic_runtime_state = std::mem::take(
                    &mut self.semantic_runtime_state,
                );
                let mut semantic_runtime_transaction = semantic_runtime_state
                    .transaction();
                for directive in self
                    .semantic_runtime_annotations
                    .effect_directives_for_rule(rule_name)
                {
                    let _ = self
                        .apply_semantic_runtime_effect_directive(
                            &mut semantic_runtime_transaction,
                            directive,
                            &node.content,
                        )?;
                }
                let mut post_predicate_blocked = false;
                let mut blocked_post_predicate: Option<String> = None;
                for directive in self
                    .semantic_runtime_annotations
                    .post_predicates_for_rule(rule_name)
                {
                    match directive {
                        crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                            spec,
                        ) if spec.phase
                            == crate::ast_pipeline::SemanticPredicatePhase::Post => {
                            let resolved_spec = self
                                .resolve_semantic_predicate_spec_against_content(
                                    spec,
                                    semantic_raw_content,
                                    &node.content,
                                )?;
                            match semantic_runtime_transaction
                                .state()
                                .evaluate_content_aware_predicate(
                                    &resolved_spec,
                                    semantic_raw_content,
                                    &node.content,
                                )
                            {
                                Some(true) => {}
                                Some(false) => {
                                    blocked_post_predicate = Some(
                                        self.semantic_predicate_debug_label(&resolved_spec),
                                    );
                                    post_predicate_blocked = true;
                                    break;
                                }
                                None => {}
                            }
                        }
                        crate::ast_pipeline::SemanticRuntimeDirective::OpenScope(_)
                        | crate::ast_pipeline::SemanticRuntimeDirective::CloseScope(_)
                        | crate::ast_pipeline::SemanticRuntimeDirective::EmitFact(_)
                        | crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                            _,
                        ) => {}
                    }
                }
                if post_predicate_blocked {
                    if self.logger_enabled {
                        self.logger
                            .log_info(
                                file!(),
                                line!(),
                                &format!(
                                    "🚫 Rule '{}' rejected by post predicate '{}'", rule_name,
                                    blocked_post_predicate.as_deref().unwrap_or("<unknown>"),
                                ),
                            );
                    }
                    Err(ParseError::Backtrack {
                        position: node.span.start,
                    })
                } else {
                    let _ = semantic_runtime_transaction.commit();
                    self.semantic_runtime_state = semantic_runtime_state;
                    Ok(node)
                }
            }
        };
        if result.is_err() {
            self.semantic_runtime_state = original_semantic_runtime_state;
        }
        result
    }
    fn apply_semantic_runtime_effect_directive(
        &self,
        transaction: &mut crate::ast_pipeline::SemanticRuntimeTransaction<'_>,
        directive: &crate::ast_pipeline::SemanticRuntimeDirective,
        root_content: &ParseContent<'input>,
    ) -> ParseResult<bool> {
        match directive {
            crate::ast_pipeline::SemanticRuntimeDirective::Predicate(_) => Ok(false),
            crate::ast_pipeline::SemanticRuntimeDirective::OpenScope(spec) => {
                let resolved_name = spec
                    .name
                    .as_ref()
                    .map(|value| {
                        self.resolve_semantic_runtime_value_against_content(
                                value,
                                root_content,
                            )
                            .ok_or_else(|| {
                                self.create_contextual_error(
                                    &format!(
                                        "Semantic runtime could not resolve scope name for directive in current parse result"
                                    ),
                                )
                            })
                    })
                    .transpose()?;
                Ok(
                    transaction
                        .apply_directive(
                            &crate::ast_pipeline::SemanticRuntimeDirective::OpenScope(crate::ast_pipeline::SemanticScopeSpec {
                                kind: spec.kind.clone(),
                                name: resolved_name,
                            }),
                        ),
                )
            }
            crate::ast_pipeline::SemanticRuntimeDirective::CloseScope(spec) => {
                let resolved_name = spec
                    .name
                    .as_ref()
                    .map(|value| {
                        self.resolve_semantic_runtime_value_against_content(
                                value,
                                root_content,
                            )
                            .ok_or_else(|| {
                                self.create_contextual_error(
                                    &format!(
                                        "Semantic runtime could not resolve close-scope name for directive in current parse result"
                                    ),
                                )
                            })
                    })
                    .transpose()?;
                Ok(
                    transaction
                        .apply_directive(
                            &crate::ast_pipeline::SemanticRuntimeDirective::CloseScope(crate::ast_pipeline::SemanticCloseScopeSpec {
                                kind: spec.kind.clone(),
                                name: resolved_name,
                            }),
                        ),
                )
            }
            crate::ast_pipeline::SemanticRuntimeDirective::EmitFact(spec) => {
                let resolved_name = self
                    .resolve_semantic_runtime_value_against_content(
                        &spec.name,
                        root_content,
                    )
                    .ok_or_else(|| {
                        self.create_contextual_error(
                            &format!(
                                "Semantic runtime could not resolve fact name for directive in current parse result"
                            ),
                        )
                    })?;
                let resolved_attributes = self
                    .resolve_unified_semantic_properties_against_content(
                        &spec.attributes,
                        root_content,
                    )?;
                Ok(
                    transaction
                        .apply_directive(
                            &crate::ast_pipeline::SemanticRuntimeDirective::EmitFact(crate::ast_pipeline::SemanticFactSpec {
                                kind: spec.kind.clone(),
                                name: resolved_name,
                                attributes: resolved_attributes,
                            }),
                        ),
                )
            }
        }
    }
    fn resolve_semantic_runtime_value_against_content(
        &self,
        value: &crate::ast_pipeline::SemanticRuntimeValue,
        root_content: &ParseContent<'input>,
    ) -> Option<crate::ast_pipeline::SemanticRuntimeValue> {
        match value {
            crate::ast_pipeline::SemanticRuntimeValue::RuleReference(reference) => {
                self.resolve_semantic_reference(root_content, reference)
                    .map(|resolved| self.coerce_semantic_runtime_scalar(&resolved))
            }
            crate::ast_pipeline::SemanticRuntimeValue::String(text) => {
                Some(crate::ast_pipeline::SemanticRuntimeValue::String(text.clone()))
            }
            crate::ast_pipeline::SemanticRuntimeValue::Identifier(text) => {
                Some(crate::ast_pipeline::SemanticRuntimeValue::Identifier(text.clone()))
            }
            crate::ast_pipeline::SemanticRuntimeValue::Number(text) => {
                Some(crate::ast_pipeline::SemanticRuntimeValue::Number(text.clone()))
            }
            crate::ast_pipeline::SemanticRuntimeValue::Boolean(value) => {
                Some(crate::ast_pipeline::SemanticRuntimeValue::Boolean(*value))
            }
            crate::ast_pipeline::SemanticRuntimeValue::Null => {
                Some(crate::ast_pipeline::SemanticRuntimeValue::Null)
            }
        }
    }
    fn resolve_unified_semantic_properties_against_content(
        &self,
        properties: &[crate::ast_pipeline::UnifiedSemanticProperty],
        root_content: &ParseContent<'input>,
    ) -> ParseResult<Vec<crate::ast_pipeline::UnifiedSemanticProperty>> {
        let mut resolved = Vec::with_capacity(properties.len());
        for property in properties {
            resolved
                .push(crate::ast_pipeline::UnifiedSemanticProperty {
                    key: property.key.clone(),
                    value: self
                        .resolve_unified_semantic_value_against_content(
                            &property.value,
                            root_content,
                        )?,
                });
        }
        Ok(resolved)
    }
    fn resolve_semantic_predicate_spec_against_content(
        &self,
        spec: &crate::ast_pipeline::SemanticPredicateSpec,
        raw_content: &ParseContent<'input>,
        shaped_content: &ParseContent<'input>,
    ) -> ParseResult<crate::ast_pipeline::SemanticPredicateSpec> {
        let selected_content = match spec.view {
            crate::ast_pipeline::SemanticPredicateContentView::Raw => raw_content,
            crate::ast_pipeline::SemanticPredicateContentView::Shaped => shaped_content,
        };
        let mut resolved_args = Vec::with_capacity(spec.args.len());
        for arg in &spec.args {
            resolved_args
                .push(
                    self
                        .resolve_unified_semantic_value_against_content(
                            arg,
                            selected_content,
                        )?,
                );
        }
        Ok(crate::ast_pipeline::SemanticPredicateSpec {
            name: spec.name.clone(),
            args: resolved_args,
            phase: spec.phase,
            view: spec.view,
        })
    }
    fn try_resolve_semantic_predicate_spec_against_content(
        &self,
        spec: &crate::ast_pipeline::SemanticPredicateSpec,
        raw_content: &ParseContent<'input>,
        shaped_content: &ParseContent<'input>,
    ) -> ParseResult<Option<crate::ast_pipeline::SemanticPredicateSpec>> {
        let selected_content = match spec.view {
            crate::ast_pipeline::SemanticPredicateContentView::Raw => raw_content,
            crate::ast_pipeline::SemanticPredicateContentView::Shaped => shaped_content,
        };
        let mut resolved_args = Vec::with_capacity(spec.args.len());
        for arg in &spec.args {
            let Some(resolved_arg) = self
                .try_resolve_unified_semantic_value_against_content(
                    arg,
                    selected_content,
                )? else {
                return Ok(None);
            };
            resolved_args.push(resolved_arg);
        }
        Ok(
            Some(crate::ast_pipeline::SemanticPredicateSpec {
                name: spec.name.clone(),
                args: resolved_args,
                phase: spec.phase,
                view: spec.view,
            }),
        )
    }
    fn resolve_unified_semantic_value_against_content(
        &self,
        value: &crate::ast_pipeline::UnifiedSemanticValue,
        root_content: &ParseContent<'input>,
    ) -> ParseResult<crate::ast_pipeline::UnifiedSemanticValue> {
        match value {
            crate::ast_pipeline::UnifiedSemanticValue::RuleReference(reference) => {
                self.resolve_semantic_reference(root_content, reference)
                    .map(|resolved| self.coerce_unified_semantic_scalar(&resolved))
                    .ok_or_else(|| {
                        self.create_contextual_error(
                            &format!(
                                "Semantic runtime could not resolve attribute reference '{}'",
                                reference
                            ),
                        )
                    })
            }
            crate::ast_pipeline::UnifiedSemanticValue::String(text) => {
                Ok(crate::ast_pipeline::UnifiedSemanticValue::String(text.clone()))
            }
            crate::ast_pipeline::UnifiedSemanticValue::Identifier(text) => {
                Ok(crate::ast_pipeline::UnifiedSemanticValue::Identifier(text.clone()))
            }
            crate::ast_pipeline::UnifiedSemanticValue::Number(text) => {
                Ok(crate::ast_pipeline::UnifiedSemanticValue::Number(text.clone()))
            }
            crate::ast_pipeline::UnifiedSemanticValue::Boolean(value) => {
                Ok(crate::ast_pipeline::UnifiedSemanticValue::Boolean(*value))
            }
            crate::ast_pipeline::UnifiedSemanticValue::Null => {
                Ok(crate::ast_pipeline::UnifiedSemanticValue::Null)
            }
            crate::ast_pipeline::UnifiedSemanticValue::Array(elements) => {
                let mut resolved = Vec::with_capacity(elements.len());
                for element in elements {
                    resolved
                        .push(
                            self
                                .resolve_unified_semantic_value_against_content(
                                    element,
                                    root_content,
                                )?,
                        );
                }
                Ok(crate::ast_pipeline::UnifiedSemanticValue::Array(resolved))
            }
            crate::ast_pipeline::UnifiedSemanticValue::Object(properties) => {
                Ok(
                    crate::ast_pipeline::UnifiedSemanticValue::Object(
                        self
                            .resolve_unified_semantic_properties_against_content(
                                properties,
                                root_content,
                            )?,
                    ),
                )
            }
        }
    }
    fn try_resolve_unified_semantic_value_against_content(
        &self,
        value: &crate::ast_pipeline::UnifiedSemanticValue,
        root_content: &ParseContent<'input>,
    ) -> ParseResult<Option<crate::ast_pipeline::UnifiedSemanticValue>> {
        match value {
            crate::ast_pipeline::UnifiedSemanticValue::RuleReference(reference) => {
                Ok(
                    self
                        .resolve_semantic_reference(root_content, reference)
                        .map(|resolved| self.coerce_unified_semantic_scalar(&resolved)),
                )
            }
            crate::ast_pipeline::UnifiedSemanticValue::String(text) => {
                Ok(Some(crate::ast_pipeline::UnifiedSemanticValue::String(text.clone())))
            }
            crate::ast_pipeline::UnifiedSemanticValue::Identifier(text) => {
                Ok(
                    Some(
                        crate::ast_pipeline::UnifiedSemanticValue::Identifier(
                            text.clone(),
                        ),
                    ),
                )
            }
            crate::ast_pipeline::UnifiedSemanticValue::Number(text) => {
                Ok(Some(crate::ast_pipeline::UnifiedSemanticValue::Number(text.clone())))
            }
            crate::ast_pipeline::UnifiedSemanticValue::Boolean(value) => {
                Ok(Some(crate::ast_pipeline::UnifiedSemanticValue::Boolean(*value)))
            }
            crate::ast_pipeline::UnifiedSemanticValue::Null => {
                Ok(Some(crate::ast_pipeline::UnifiedSemanticValue::Null))
            }
            crate::ast_pipeline::UnifiedSemanticValue::Array(elements) => {
                let mut resolved = Vec::with_capacity(elements.len());
                for element in elements {
                    let Some(resolved_element) = self
                        .try_resolve_unified_semantic_value_against_content(
                            element,
                            root_content,
                        )? else {
                        return Ok(None);
                    };
                    resolved.push(resolved_element);
                }
                Ok(Some(crate::ast_pipeline::UnifiedSemanticValue::Array(resolved)))
            }
            crate::ast_pipeline::UnifiedSemanticValue::Object(properties) => {
                let mut resolved = Vec::with_capacity(properties.len());
                for property in properties {
                    let Some(resolved_value) = self
                        .try_resolve_unified_semantic_value_against_content(
                            &property.value,
                            root_content,
                        )? else {
                        return Ok(None);
                    };
                    resolved
                        .push(crate::ast_pipeline::UnifiedSemanticProperty {
                            key: property.key.clone(),
                            value: resolved_value,
                        });
                }
                Ok(Some(crate::ast_pipeline::UnifiedSemanticValue::Object(resolved)))
            }
        }
    }
    fn coerce_semantic_runtime_scalar(
        &self,
        value: &str,
    ) -> crate::ast_pipeline::SemanticRuntimeValue {
        let normalized = value.trim();
        if normalized.eq_ignore_ascii_case("true") {
            return crate::ast_pipeline::SemanticRuntimeValue::Boolean(true);
        }
        if normalized.eq_ignore_ascii_case("false") {
            return crate::ast_pipeline::SemanticRuntimeValue::Boolean(false);
        }
        if normalized.parse::<f64>().is_ok() {
            return crate::ast_pipeline::SemanticRuntimeValue::Number(
                normalized.to_string(),
            );
        }
        if self.semantic_identifier(normalized) {
            return crate::ast_pipeline::SemanticRuntimeValue::Identifier(
                normalized.to_string(),
            );
        }
        crate::ast_pipeline::SemanticRuntimeValue::String(normalized.to_string())
    }
    fn coerce_unified_semantic_scalar(
        &self,
        value: &str,
    ) -> crate::ast_pipeline::UnifiedSemanticValue {
        let normalized = value.trim();
        if normalized.eq_ignore_ascii_case("true") {
            return crate::ast_pipeline::UnifiedSemanticValue::Boolean(true);
        }
        if normalized.eq_ignore_ascii_case("false") {
            return crate::ast_pipeline::UnifiedSemanticValue::Boolean(false);
        }
        if normalized.parse::<f64>().is_ok() {
            return crate::ast_pipeline::UnifiedSemanticValue::Number(
                normalized.to_string(),
            );
        }
        if self.semantic_identifier(normalized) {
            return crate::ast_pipeline::UnifiedSemanticValue::Identifier(
                normalized.to_string(),
            );
        }
        crate::ast_pipeline::UnifiedSemanticValue::String(normalized.to_string())
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
    pub fn parse_json(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("json", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "json", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "json", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 4096usize => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "json", position, depth
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
        self.recursion_guard.enter("json", position);
        let start_pos = self.position;
        let result: ParseResult<ParseNode<'input>> = (|parser: &mut Self| {
            let inner_result = parser
                .memoized_call(
                    Self::RULE_JSON,
                    |parser| {
                        let semantic_capture_raw_for_post = parser
                            .semantic_runtime_annotations
                            .needs_raw_post_capture_for_rule("json");
                        let mut semantic_selected_branch_index: Option<usize> = None;
                        let mut semantic_raw_content: Option<ParseContent<'input>> = None;
                        let result = ParseContent::Alternative(
                            Box::new(parser.parse_value()?),
                        );
                        let result = {
                            {
                                let mut __pgen_obj = serde_json::Map::new();
                                __pgen_obj
                                    .insert(
                                        "type".to_string(),
                                        serde_json::Value::String("json".to_string()),
                                    );
                                __pgen_obj
                                    .insert(
                                        "value".to_string(),
                                        {
                                            let __pgen_content = {
                                                match &result {
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
                                                    other => other.clone(),
                                                }
                                            };
                                            __pgen_content.to_json_value()
                                        },
                                    );
                                ParseContent::Json(serde_json::Value::Object(__pgen_obj))
                            }
                        };
                        let end_pos = parser.position;
                        parser
                            .record_coverage_target_event(
                                "json",
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
                                "json",
                                "rule.json",
                            );
                        parser
                            .record_deterministic_partition_event(
                                "json",
                                start_pos,
                                end_pos,
                                deterministic_partition_effective_enabled,
                                &deterministic_partition_effective_group,
                            );
                        Ok((
                            ParseNode {
                                rule_name: "json",
                                content: result,
                                span: start_pos..end_pos,
                            },
                            semantic_raw_content,
                        ))
                    },
                );
            inner_result.map(|(node, _raw)| node)
        })(self);
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger_enabled {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "json", start_pos, node.span.end, consumed, consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "json", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "json", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "json",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "json", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_value(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("value", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "value", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "value", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 4096usize => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "value", position, depth
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
        self.recursion_guard.enter("value", position);
        let start_pos = self.position;
        let result: ParseResult<ParseNode<'input>> = (|parser: &mut Self| {
            let inner_result = parser
                .memoized_call(
                    Self::RULE_VALUE,
                    |parser| {
                        let semantic_capture_raw_for_post = parser
                            .semantic_runtime_annotations
                            .needs_raw_post_capture_for_rule("value");
                        let mut semantic_selected_branch_index: Option<usize> = None;
                        let mut semantic_raw_content: Option<ParseContent<'input>> = None;
                        let parse_start = parser.position;
                        let mut best_content: Option<ParseContent<'input>> = None;
                        let mut best_raw_content: Option<ParseContent<'input>> = None;
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
                                "value",
                                "rule.value",
                            );
                        let deterministic_partition_offset = if deterministic_partition_effective_enabled {
                            parser
                                .deterministic_partition_offset_runtime(
                                    &deterministic_partition_effective_group,
                                    7usize,
                                )
                        } else {
                            0usize
                        };
                        let mut evaluation_order: Vec<usize> = (0..7usize).collect();
                        if deterministic_partition_effective_enabled && 7usize > 1
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
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                1usize, 7usize, "value", parser.position
                                                            ),
                                                        );
                                                }
                                                let result = ParseContent::Alternative(
                                                    Box::new(parser.parse_object()?),
                                                );
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                1usize, 7usize, "value", parser.position
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
                                            let raw_content = content;
                                            let transformed = {
                                                let content = raw_content.clone();
                                                {
                                                    match &content {
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
                                                        other => other.clone(),
                                                    }
                                                }
                                            };
                                            let mut branch_predicate_blocked = false;
                                            let mut blocked_branch_predicate: Option<String> = None;
                                            for directive in parser
                                                .semantic_runtime_annotations
                                                .branch_predicates_for_rule("value")
                                                .chain(
                                                    parser
                                                        .semantic_runtime_annotations
                                                        .branch_predicates_for_rule_branch(
                                                            "value",
                                                            current_branch_index,
                                                        ),
                                                )
                                            {
                                                match directive {
                                                    crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        spec,
                                                    ) if spec.phase
                                                        == crate::ast_pipeline::SemanticPredicatePhase::Branch => {
                                                        let Some(resolved_spec) = parser
                                                            .try_resolve_semantic_predicate_spec_against_content(
                                                                spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )? else {
                                                            blocked_branch_predicate = Some(
                                                                parser.semantic_predicate_debug_label(spec),
                                                            );
                                                            branch_predicate_blocked = true;
                                                            break;
                                                        };
                                                        match parser
                                                            .semantic_runtime_state
                                                            .evaluate_content_aware_predicate(
                                                                &resolved_spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )
                                                        {
                                                            Some(true) => {}
                                                            Some(false) => {
                                                                blocked_branch_predicate = Some(
                                                                    parser.semantic_predicate_debug_label(&resolved_spec),
                                                                );
                                                                branch_predicate_blocked = true;
                                                                break;
                                                            }
                                                            None => {}
                                                        }
                                                    }
                                                    crate::ast_pipeline::SemanticRuntimeDirective::OpenScope(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::CloseScope(
                                                        _,
                                                    )
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::EmitFact(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        _,
                                                    ) => {}
                                                }
                                            }
                                            let should_take = if branch_predicate_blocked {
                                                false
                                            } else if "longest_match" == "ordered" {
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
                                                if semantic_capture_raw_for_post {
                                                    best_raw_content = Some(raw_content.clone());
                                                }
                                                best_content = Some(transformed);
                                            } else if branch_predicate_blocked && parser.logger_enabled
                                            {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚫 Branch {}/{} for rule '{}' rejected by branch predicate '{}' at position {}",
                                                            1usize, 7usize, "value", blocked_branch_predicate.as_deref()
                                                            .unwrap_or("<unknown>"), candidate_end
                                                        ),
                                                    );
                                            }
                                        } else if parser.logger_enabled {
                                            parser
                                                .logger
                                                .log_info(
                                                    "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                    0,
                                                    &format!(
                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                        1usize, 7usize, "value", parser.position
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
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                2usize, 7usize, "value", parser.position
                                                            ),
                                                        );
                                                }
                                                let result = ParseContent::Alternative(
                                                    Box::new(parser.parse_array()?),
                                                );
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                2usize, 7usize, "value", parser.position
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
                                            let raw_content = content;
                                            let transformed = {
                                                let content = raw_content.clone();
                                                {
                                                    match &content {
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
                                                        other => other.clone(),
                                                    }
                                                }
                                            };
                                            let mut branch_predicate_blocked = false;
                                            let mut blocked_branch_predicate: Option<String> = None;
                                            for directive in parser
                                                .semantic_runtime_annotations
                                                .branch_predicates_for_rule("value")
                                                .chain(
                                                    parser
                                                        .semantic_runtime_annotations
                                                        .branch_predicates_for_rule_branch(
                                                            "value",
                                                            current_branch_index,
                                                        ),
                                                )
                                            {
                                                match directive {
                                                    crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        spec,
                                                    ) if spec.phase
                                                        == crate::ast_pipeline::SemanticPredicatePhase::Branch => {
                                                        let Some(resolved_spec) = parser
                                                            .try_resolve_semantic_predicate_spec_against_content(
                                                                spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )? else {
                                                            blocked_branch_predicate = Some(
                                                                parser.semantic_predicate_debug_label(spec),
                                                            );
                                                            branch_predicate_blocked = true;
                                                            break;
                                                        };
                                                        match parser
                                                            .semantic_runtime_state
                                                            .evaluate_content_aware_predicate(
                                                                &resolved_spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )
                                                        {
                                                            Some(true) => {}
                                                            Some(false) => {
                                                                blocked_branch_predicate = Some(
                                                                    parser.semantic_predicate_debug_label(&resolved_spec),
                                                                );
                                                                branch_predicate_blocked = true;
                                                                break;
                                                            }
                                                            None => {}
                                                        }
                                                    }
                                                    crate::ast_pipeline::SemanticRuntimeDirective::OpenScope(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::CloseScope(
                                                        _,
                                                    )
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::EmitFact(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        _,
                                                    ) => {}
                                                }
                                            }
                                            let should_take = if branch_predicate_blocked {
                                                false
                                            } else if "longest_match" == "ordered" {
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
                                                if semantic_capture_raw_for_post {
                                                    best_raw_content = Some(raw_content.clone());
                                                }
                                                best_content = Some(transformed);
                                            } else if branch_predicate_blocked && parser.logger_enabled
                                            {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚫 Branch {}/{} for rule '{}' rejected by branch predicate '{}' at position {}",
                                                            2usize, 7usize, "value", blocked_branch_predicate.as_deref()
                                                            .unwrap_or("<unknown>"), candidate_end
                                                        ),
                                                    );
                                            }
                                        } else if parser.logger_enabled {
                                            parser
                                                .logger
                                                .log_info(
                                                    "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                    0,
                                                    &format!(
                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                        2usize, 7usize, "value", parser.position
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
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                3usize, 7usize, "value", parser.position
                                                            ),
                                                        );
                                                }
                                                let result = ParseContent::Alternative(
                                                    Box::new(parser.parse_string()?),
                                                );
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                3usize, 7usize, "value", parser.position
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
                                            let raw_content = content;
                                            let transformed = {
                                                let content = raw_content.clone();
                                                {
                                                    let mut __pgen_obj = serde_json::Map::new();
                                                    __pgen_obj
                                                        .insert(
                                                            "type".to_string(),
                                                            serde_json::Value::String("string".to_string()),
                                                        );
                                                    __pgen_obj
                                                        .insert(
                                                            "value".to_string(),
                                                            {
                                                                let __pgen_content = {
                                                                    match &content {
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
                                                                        other => other.clone(),
                                                                    }
                                                                };
                                                                __pgen_content.to_json_value()
                                                            },
                                                        );
                                                    ParseContent::Json(serde_json::Value::Object(__pgen_obj))
                                                }
                                            };
                                            let mut branch_predicate_blocked = false;
                                            let mut blocked_branch_predicate: Option<String> = None;
                                            for directive in parser
                                                .semantic_runtime_annotations
                                                .branch_predicates_for_rule("value")
                                                .chain(
                                                    parser
                                                        .semantic_runtime_annotations
                                                        .branch_predicates_for_rule_branch(
                                                            "value",
                                                            current_branch_index,
                                                        ),
                                                )
                                            {
                                                match directive {
                                                    crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        spec,
                                                    ) if spec.phase
                                                        == crate::ast_pipeline::SemanticPredicatePhase::Branch => {
                                                        let Some(resolved_spec) = parser
                                                            .try_resolve_semantic_predicate_spec_against_content(
                                                                spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )? else {
                                                            blocked_branch_predicate = Some(
                                                                parser.semantic_predicate_debug_label(spec),
                                                            );
                                                            branch_predicate_blocked = true;
                                                            break;
                                                        };
                                                        match parser
                                                            .semantic_runtime_state
                                                            .evaluate_content_aware_predicate(
                                                                &resolved_spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )
                                                        {
                                                            Some(true) => {}
                                                            Some(false) => {
                                                                blocked_branch_predicate = Some(
                                                                    parser.semantic_predicate_debug_label(&resolved_spec),
                                                                );
                                                                branch_predicate_blocked = true;
                                                                break;
                                                            }
                                                            None => {}
                                                        }
                                                    }
                                                    crate::ast_pipeline::SemanticRuntimeDirective::OpenScope(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::CloseScope(
                                                        _,
                                                    )
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::EmitFact(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        _,
                                                    ) => {}
                                                }
                                            }
                                            let should_take = if branch_predicate_blocked {
                                                false
                                            } else if "longest_match" == "ordered" {
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
                                                if semantic_capture_raw_for_post {
                                                    best_raw_content = Some(raw_content.clone());
                                                }
                                                best_content = Some(transformed);
                                            } else if branch_predicate_blocked && parser.logger_enabled
                                            {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚫 Branch {}/{} for rule '{}' rejected by branch predicate '{}' at position {}",
                                                            3usize, 7usize, "value", blocked_branch_predicate.as_deref()
                                                            .unwrap_or("<unknown>"), candidate_end
                                                        ),
                                                    );
                                            }
                                        } else if parser.logger_enabled {
                                            parser
                                                .logger
                                                .log_info(
                                                    "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                    0,
                                                    &format!(
                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                        3usize, 7usize, "value", parser.position
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
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                4usize, 7usize, "value", parser.position
                                                            ),
                                                        );
                                                }
                                                let result = ParseContent::Alternative(
                                                    Box::new(parser.parse_number()?),
                                                );
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                4usize, 7usize, "value", parser.position
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
                                            let raw_content = content;
                                            let transformed = {
                                                let content = raw_content.clone();
                                                {
                                                    let mut __pgen_obj = serde_json::Map::new();
                                                    __pgen_obj
                                                        .insert(
                                                            "type".to_string(),
                                                            serde_json::Value::String("number".to_string()),
                                                        );
                                                    __pgen_obj
                                                        .insert(
                                                            "value".to_string(),
                                                            {
                                                                let __pgen_content = {
                                                                    match &content {
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
                                                                        other => other.clone(),
                                                                    }
                                                                };
                                                                __pgen_content.to_json_value()
                                                            },
                                                        );
                                                    ParseContent::Json(serde_json::Value::Object(__pgen_obj))
                                                }
                                            };
                                            let mut branch_predicate_blocked = false;
                                            let mut blocked_branch_predicate: Option<String> = None;
                                            for directive in parser
                                                .semantic_runtime_annotations
                                                .branch_predicates_for_rule("value")
                                                .chain(
                                                    parser
                                                        .semantic_runtime_annotations
                                                        .branch_predicates_for_rule_branch(
                                                            "value",
                                                            current_branch_index,
                                                        ),
                                                )
                                            {
                                                match directive {
                                                    crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        spec,
                                                    ) if spec.phase
                                                        == crate::ast_pipeline::SemanticPredicatePhase::Branch => {
                                                        let Some(resolved_spec) = parser
                                                            .try_resolve_semantic_predicate_spec_against_content(
                                                                spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )? else {
                                                            blocked_branch_predicate = Some(
                                                                parser.semantic_predicate_debug_label(spec),
                                                            );
                                                            branch_predicate_blocked = true;
                                                            break;
                                                        };
                                                        match parser
                                                            .semantic_runtime_state
                                                            .evaluate_content_aware_predicate(
                                                                &resolved_spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )
                                                        {
                                                            Some(true) => {}
                                                            Some(false) => {
                                                                blocked_branch_predicate = Some(
                                                                    parser.semantic_predicate_debug_label(&resolved_spec),
                                                                );
                                                                branch_predicate_blocked = true;
                                                                break;
                                                            }
                                                            None => {}
                                                        }
                                                    }
                                                    crate::ast_pipeline::SemanticRuntimeDirective::OpenScope(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::CloseScope(
                                                        _,
                                                    )
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::EmitFact(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        _,
                                                    ) => {}
                                                }
                                            }
                                            let should_take = if branch_predicate_blocked {
                                                false
                                            } else if "longest_match" == "ordered" {
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
                                                if semantic_capture_raw_for_post {
                                                    best_raw_content = Some(raw_content.clone());
                                                }
                                                best_content = Some(transformed);
                                            } else if branch_predicate_blocked && parser.logger_enabled
                                            {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚫 Branch {}/{} for rule '{}' rejected by branch predicate '{}' at position {}",
                                                            4usize, 7usize, "value", blocked_branch_predicate.as_deref()
                                                            .unwrap_or("<unknown>"), candidate_end
                                                        ),
                                                    );
                                            }
                                        } else if parser.logger_enabled {
                                            parser
                                                .logger
                                                .log_info(
                                                    "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                    0,
                                                    &format!(
                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                        4usize, 7usize, "value", parser.position
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
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                5usize, 7usize, "value", parser.position
                                                            ),
                                                        );
                                                }
                                                let matched_str = parser.match_regex("\\s*true\\s*", true)?;
                                                let result = ParseContent::Terminal(matched_str);
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                5usize, 7usize, "value", parser.position
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
                                            let raw_content = content;
                                            let transformed = {
                                                let content = raw_content.clone();
                                                {
                                                    let mut __pgen_obj = serde_json::Map::new();
                                                    __pgen_obj
                                                        .insert(
                                                            "type".to_string(),
                                                            serde_json::Value::String("boolean".to_string()),
                                                        );
                                                    __pgen_obj
                                                        .insert("value".to_string(), serde_json::Value::Bool(true));
                                                    ParseContent::Json(serde_json::Value::Object(__pgen_obj))
                                                }
                                            };
                                            let mut branch_predicate_blocked = false;
                                            let mut blocked_branch_predicate: Option<String> = None;
                                            for directive in parser
                                                .semantic_runtime_annotations
                                                .branch_predicates_for_rule("value")
                                                .chain(
                                                    parser
                                                        .semantic_runtime_annotations
                                                        .branch_predicates_for_rule_branch(
                                                            "value",
                                                            current_branch_index,
                                                        ),
                                                )
                                            {
                                                match directive {
                                                    crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        spec,
                                                    ) if spec.phase
                                                        == crate::ast_pipeline::SemanticPredicatePhase::Branch => {
                                                        let Some(resolved_spec) = parser
                                                            .try_resolve_semantic_predicate_spec_against_content(
                                                                spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )? else {
                                                            blocked_branch_predicate = Some(
                                                                parser.semantic_predicate_debug_label(spec),
                                                            );
                                                            branch_predicate_blocked = true;
                                                            break;
                                                        };
                                                        match parser
                                                            .semantic_runtime_state
                                                            .evaluate_content_aware_predicate(
                                                                &resolved_spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )
                                                        {
                                                            Some(true) => {}
                                                            Some(false) => {
                                                                blocked_branch_predicate = Some(
                                                                    parser.semantic_predicate_debug_label(&resolved_spec),
                                                                );
                                                                branch_predicate_blocked = true;
                                                                break;
                                                            }
                                                            None => {}
                                                        }
                                                    }
                                                    crate::ast_pipeline::SemanticRuntimeDirective::OpenScope(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::CloseScope(
                                                        _,
                                                    )
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::EmitFact(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        _,
                                                    ) => {}
                                                }
                                            }
                                            let should_take = if branch_predicate_blocked {
                                                false
                                            } else if "longest_match" == "ordered" {
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
                                                if semantic_capture_raw_for_post {
                                                    best_raw_content = Some(raw_content.clone());
                                                }
                                                best_content = Some(transformed);
                                            } else if branch_predicate_blocked && parser.logger_enabled
                                            {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚫 Branch {}/{} for rule '{}' rejected by branch predicate '{}' at position {}",
                                                            5usize, 7usize, "value", blocked_branch_predicate.as_deref()
                                                            .unwrap_or("<unknown>"), candidate_end
                                                        ),
                                                    );
                                            }
                                        } else if parser.logger_enabled {
                                            parser
                                                .logger
                                                .log_info(
                                                    "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                    0,
                                                    &format!(
                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                        5usize, 7usize, "value", parser.position
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
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                6usize, 7usize, "value", parser.position
                                                            ),
                                                        );
                                                }
                                                let matched_str = parser
                                                    .match_regex("\\s*false\\s*", true)?;
                                                let result = ParseContent::Terminal(matched_str);
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                6usize, 7usize, "value", parser.position
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
                                            let raw_content = content;
                                            let transformed = {
                                                let content = raw_content.clone();
                                                {
                                                    let mut __pgen_obj = serde_json::Map::new();
                                                    __pgen_obj
                                                        .insert(
                                                            "type".to_string(),
                                                            serde_json::Value::String("boolean".to_string()),
                                                        );
                                                    __pgen_obj
                                                        .insert(
                                                            "value".to_string(),
                                                            serde_json::Value::Bool(false),
                                                        );
                                                    ParseContent::Json(serde_json::Value::Object(__pgen_obj))
                                                }
                                            };
                                            let mut branch_predicate_blocked = false;
                                            let mut blocked_branch_predicate: Option<String> = None;
                                            for directive in parser
                                                .semantic_runtime_annotations
                                                .branch_predicates_for_rule("value")
                                                .chain(
                                                    parser
                                                        .semantic_runtime_annotations
                                                        .branch_predicates_for_rule_branch(
                                                            "value",
                                                            current_branch_index,
                                                        ),
                                                )
                                            {
                                                match directive {
                                                    crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        spec,
                                                    ) if spec.phase
                                                        == crate::ast_pipeline::SemanticPredicatePhase::Branch => {
                                                        let Some(resolved_spec) = parser
                                                            .try_resolve_semantic_predicate_spec_against_content(
                                                                spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )? else {
                                                            blocked_branch_predicate = Some(
                                                                parser.semantic_predicate_debug_label(spec),
                                                            );
                                                            branch_predicate_blocked = true;
                                                            break;
                                                        };
                                                        match parser
                                                            .semantic_runtime_state
                                                            .evaluate_content_aware_predicate(
                                                                &resolved_spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )
                                                        {
                                                            Some(true) => {}
                                                            Some(false) => {
                                                                blocked_branch_predicate = Some(
                                                                    parser.semantic_predicate_debug_label(&resolved_spec),
                                                                );
                                                                branch_predicate_blocked = true;
                                                                break;
                                                            }
                                                            None => {}
                                                        }
                                                    }
                                                    crate::ast_pipeline::SemanticRuntimeDirective::OpenScope(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::CloseScope(
                                                        _,
                                                    )
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::EmitFact(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        _,
                                                    ) => {}
                                                }
                                            }
                                            let should_take = if branch_predicate_blocked {
                                                false
                                            } else if "longest_match" == "ordered" {
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
                                                if semantic_capture_raw_for_post {
                                                    best_raw_content = Some(raw_content.clone());
                                                }
                                                best_content = Some(transformed);
                                            } else if branch_predicate_blocked && parser.logger_enabled
                                            {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚫 Branch {}/{} for rule '{}' rejected by branch predicate '{}' at position {}",
                                                            6usize, 7usize, "value", blocked_branch_predicate.as_deref()
                                                            .unwrap_or("<unknown>"), candidate_end
                                                        ),
                                                    );
                                            }
                                        } else if parser.logger_enabled {
                                            parser
                                                .logger
                                                .log_info(
                                                    "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                    0,
                                                    &format!(
                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                        6usize, 7usize, "value", parser.position
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
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                7usize, 7usize, "value", parser.position
                                                            ),
                                                        );
                                                }
                                                let matched_str = parser.match_regex("\\s*null\\s*", true)?;
                                                let result = ParseContent::Terminal(matched_str);
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                7usize, 7usize, "value", parser.position
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
                                            let raw_content = content;
                                            let transformed = {
                                                let content = raw_content.clone();
                                                {
                                                    let mut __pgen_obj = serde_json::Map::new();
                                                    __pgen_obj
                                                        .insert(
                                                            "type".to_string(),
                                                            serde_json::Value::String("null".to_string()),
                                                        );
                                                    ParseContent::Json(serde_json::Value::Object(__pgen_obj))
                                                }
                                            };
                                            let mut branch_predicate_blocked = false;
                                            let mut blocked_branch_predicate: Option<String> = None;
                                            for directive in parser
                                                .semantic_runtime_annotations
                                                .branch_predicates_for_rule("value")
                                                .chain(
                                                    parser
                                                        .semantic_runtime_annotations
                                                        .branch_predicates_for_rule_branch(
                                                            "value",
                                                            current_branch_index,
                                                        ),
                                                )
                                            {
                                                match directive {
                                                    crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        spec,
                                                    ) if spec.phase
                                                        == crate::ast_pipeline::SemanticPredicatePhase::Branch => {
                                                        let Some(resolved_spec) = parser
                                                            .try_resolve_semantic_predicate_spec_against_content(
                                                                spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )? else {
                                                            blocked_branch_predicate = Some(
                                                                parser.semantic_predicate_debug_label(spec),
                                                            );
                                                            branch_predicate_blocked = true;
                                                            break;
                                                        };
                                                        match parser
                                                            .semantic_runtime_state
                                                            .evaluate_content_aware_predicate(
                                                                &resolved_spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )
                                                        {
                                                            Some(true) => {}
                                                            Some(false) => {
                                                                blocked_branch_predicate = Some(
                                                                    parser.semantic_predicate_debug_label(&resolved_spec),
                                                                );
                                                                branch_predicate_blocked = true;
                                                                break;
                                                            }
                                                            None => {}
                                                        }
                                                    }
                                                    crate::ast_pipeline::SemanticRuntimeDirective::OpenScope(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::CloseScope(
                                                        _,
                                                    )
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::EmitFact(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        _,
                                                    ) => {}
                                                }
                                            }
                                            let should_take = if branch_predicate_blocked {
                                                false
                                            } else if "longest_match" == "ordered" {
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
                                                if semantic_capture_raw_for_post {
                                                    best_raw_content = Some(raw_content.clone());
                                                }
                                                best_content = Some(transformed);
                                            } else if branch_predicate_blocked && parser.logger_enabled
                                            {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚫 Branch {}/{} for rule '{}' rejected by branch predicate '{}' at position {}",
                                                            7usize, 7usize, "value", blocked_branch_predicate.as_deref()
                                                            .unwrap_or("<unknown>"), candidate_end
                                                        ),
                                                    );
                                            }
                                        } else if parser.logger_enabled {
                                            parser
                                                .logger
                                                .log_info(
                                                    "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                    0,
                                                    &format!(
                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                        7usize, 7usize, "value", parser.position
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
                            if parser.logger_enabled {
                                parser
                                    .logger
                                    .log_info(
                                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                        0,
                                        &format!(
                                            "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                            "value", best_branch, 7usize, best_end
                                            .saturating_sub(parse_start), best_priority, "left",
                                            "longest_match"
                                        ),
                                    );
                            }
                            result = content;
                            semantic_raw_content = best_raw_content;
                        } else {
                            return Err(ParseError::Backtrack {
                                position: parse_start,
                            });
                        };
                        let end_pos = parser.position;
                        parser
                            .record_coverage_target_event(
                                "value",
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
                                "value",
                                "rule.value",
                            );
                        parser
                            .record_deterministic_partition_event(
                                "value",
                                start_pos,
                                end_pos,
                                deterministic_partition_effective_enabled,
                                &deterministic_partition_effective_group,
                            );
                        Ok((
                            ParseNode {
                                rule_name: "value",
                                content: result,
                                span: start_pos..end_pos,
                            },
                            semantic_raw_content,
                        ))
                    },
                );
            inner_result.map(|(node, _raw)| node)
        })(self);
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger_enabled {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "value", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "value", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "value", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "value",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "value", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_object(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("object", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "object", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "object", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 4096usize => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "object", position, depth
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
        self.recursion_guard.enter("object", position);
        let start_pos = self.position;
        let result: ParseResult<ParseNode<'input>> = (|parser: &mut Self| {
            let inner_result = parser
                .memoized_call(
                    Self::RULE_OBJECT,
                    |parser| {
                        let semantic_capture_raw_for_post = parser
                            .semantic_runtime_annotations
                            .needs_raw_post_capture_for_rule("object");
                        let mut semantic_selected_branch_index: Option<usize> = None;
                        let mut semantic_raw_content: Option<ParseContent<'input>> = None;
                        let parse_start = parser.position;
                        let mut best_content: Option<ParseContent<'input>> = None;
                        let mut best_raw_content: Option<ParseContent<'input>> = None;
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
                                "object",
                                "rule.object",
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
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                1usize, 2usize, "object", parser.position
                                                            ),
                                                        );
                                                }
                                                let matched_str = parser
                                                    .match_regex("\\s*\\{\\s*\\}\\s*", true)?;
                                                let result = ParseContent::Terminal(matched_str);
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                1usize, 2usize, "object", parser.position
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
                                            let raw_content = content;
                                            let transformed = {
                                                let content = raw_content.clone();
                                                {
                                                    let mut __pgen_obj = serde_json::Map::new();
                                                    __pgen_obj
                                                        .insert(
                                                            "members".to_string(),
                                                            {
                                                                let __pgen_content = {
                                                                    let mut array_elements = Vec::new();
                                                                    ParseContent::Sequence(array_elements)
                                                                };
                                                                __pgen_content.to_json_value()
                                                            },
                                                        );
                                                    __pgen_obj
                                                        .insert(
                                                            "type".to_string(),
                                                            serde_json::Value::String("object".to_string()),
                                                        );
                                                    ParseContent::Json(serde_json::Value::Object(__pgen_obj))
                                                }
                                            };
                                            let mut branch_predicate_blocked = false;
                                            let mut blocked_branch_predicate: Option<String> = None;
                                            for directive in parser
                                                .semantic_runtime_annotations
                                                .branch_predicates_for_rule("object")
                                                .chain(
                                                    parser
                                                        .semantic_runtime_annotations
                                                        .branch_predicates_for_rule_branch(
                                                            "object",
                                                            current_branch_index,
                                                        ),
                                                )
                                            {
                                                match directive {
                                                    crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        spec,
                                                    ) if spec.phase
                                                        == crate::ast_pipeline::SemanticPredicatePhase::Branch => {
                                                        let Some(resolved_spec) = parser
                                                            .try_resolve_semantic_predicate_spec_against_content(
                                                                spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )? else {
                                                            blocked_branch_predicate = Some(
                                                                parser.semantic_predicate_debug_label(spec),
                                                            );
                                                            branch_predicate_blocked = true;
                                                            break;
                                                        };
                                                        match parser
                                                            .semantic_runtime_state
                                                            .evaluate_content_aware_predicate(
                                                                &resolved_spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )
                                                        {
                                                            Some(true) => {}
                                                            Some(false) => {
                                                                blocked_branch_predicate = Some(
                                                                    parser.semantic_predicate_debug_label(&resolved_spec),
                                                                );
                                                                branch_predicate_blocked = true;
                                                                break;
                                                            }
                                                            None => {}
                                                        }
                                                    }
                                                    crate::ast_pipeline::SemanticRuntimeDirective::OpenScope(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::CloseScope(
                                                        _,
                                                    )
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::EmitFact(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        _,
                                                    ) => {}
                                                }
                                            }
                                            let should_take = if branch_predicate_blocked {
                                                false
                                            } else if "longest_match" == "ordered" {
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
                                                if semantic_capture_raw_for_post {
                                                    best_raw_content = Some(raw_content.clone());
                                                }
                                                best_content = Some(transformed);
                                            } else if branch_predicate_blocked && parser.logger_enabled
                                            {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚫 Branch {}/{} for rule '{}' rejected by branch predicate '{}' at position {}",
                                                            1usize, 2usize, "object", blocked_branch_predicate
                                                            .as_deref().unwrap_or("<unknown>"), candidate_end
                                                        ),
                                                    );
                                            }
                                        } else if parser.logger_enabled {
                                            parser
                                                .logger
                                                .log_info(
                                                    "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                    0,
                                                    &format!(
                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                        1usize, 2usize, "object", parser.position
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
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                2usize, 2usize, "object", parser.position
                                                            ),
                                                        );
                                                }
                                                let mut sequence_elements = Vec::with_capacity(3usize);
                                                {
                                                    let element_start = parser.position;
                                                    let element_content = {
                                                        let matched_str = parser.match_regex("\\s*\\{\\s*", true)?;
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
                                                            Box::new(parser.parse_members()?),
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
                                                        let matched_str = parser.match_regex("\\s*\\}\\s*", true)?;
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
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                2usize, 2usize, "object", parser.position
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
                                            let raw_content = content;
                                            let transformed = {
                                                let content = raw_content.clone();
                                                {
                                                    let mut __pgen_obj = serde_json::Map::new();
                                                    __pgen_obj
                                                        .insert(
                                                            "members".to_string(),
                                                            {
                                                                let __pgen_content = {
                                                                    match &content {
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
                                                                };
                                                                __pgen_content.to_json_value()
                                                            },
                                                        );
                                                    __pgen_obj
                                                        .insert(
                                                            "type".to_string(),
                                                            serde_json::Value::String("object".to_string()),
                                                        );
                                                    ParseContent::Json(serde_json::Value::Object(__pgen_obj))
                                                }
                                            };
                                            let mut branch_predicate_blocked = false;
                                            let mut blocked_branch_predicate: Option<String> = None;
                                            for directive in parser
                                                .semantic_runtime_annotations
                                                .branch_predicates_for_rule("object")
                                                .chain(
                                                    parser
                                                        .semantic_runtime_annotations
                                                        .branch_predicates_for_rule_branch(
                                                            "object",
                                                            current_branch_index,
                                                        ),
                                                )
                                            {
                                                match directive {
                                                    crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        spec,
                                                    ) if spec.phase
                                                        == crate::ast_pipeline::SemanticPredicatePhase::Branch => {
                                                        let Some(resolved_spec) = parser
                                                            .try_resolve_semantic_predicate_spec_against_content(
                                                                spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )? else {
                                                            blocked_branch_predicate = Some(
                                                                parser.semantic_predicate_debug_label(spec),
                                                            );
                                                            branch_predicate_blocked = true;
                                                            break;
                                                        };
                                                        match parser
                                                            .semantic_runtime_state
                                                            .evaluate_content_aware_predicate(
                                                                &resolved_spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )
                                                        {
                                                            Some(true) => {}
                                                            Some(false) => {
                                                                blocked_branch_predicate = Some(
                                                                    parser.semantic_predicate_debug_label(&resolved_spec),
                                                                );
                                                                branch_predicate_blocked = true;
                                                                break;
                                                            }
                                                            None => {}
                                                        }
                                                    }
                                                    crate::ast_pipeline::SemanticRuntimeDirective::OpenScope(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::CloseScope(
                                                        _,
                                                    )
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::EmitFact(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        _,
                                                    ) => {}
                                                }
                                            }
                                            let should_take = if branch_predicate_blocked {
                                                false
                                            } else if "longest_match" == "ordered" {
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
                                                if semantic_capture_raw_for_post {
                                                    best_raw_content = Some(raw_content.clone());
                                                }
                                                best_content = Some(transformed);
                                            } else if branch_predicate_blocked && parser.logger_enabled
                                            {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚫 Branch {}/{} for rule '{}' rejected by branch predicate '{}' at position {}",
                                                            2usize, 2usize, "object", blocked_branch_predicate
                                                            .as_deref().unwrap_or("<unknown>"), candidate_end
                                                        ),
                                                    );
                                            }
                                        } else if parser.logger_enabled {
                                            parser
                                                .logger
                                                .log_info(
                                                    "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                    0,
                                                    &format!(
                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                        2usize, 2usize, "object", parser.position
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
                            if parser.logger_enabled {
                                parser
                                    .logger
                                    .log_info(
                                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                        0,
                                        &format!(
                                            "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                            "object", best_branch, 2usize, best_end
                                            .saturating_sub(parse_start), best_priority, "left",
                                            "longest_match"
                                        ),
                                    );
                            }
                            result = content;
                            semantic_raw_content = best_raw_content;
                        } else {
                            return Err(ParseError::Backtrack {
                                position: parse_start,
                            });
                        };
                        let end_pos = parser.position;
                        parser
                            .record_coverage_target_event(
                                "object",
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
                                "object",
                                "rule.object",
                            );
                        parser
                            .record_deterministic_partition_event(
                                "object",
                                start_pos,
                                end_pos,
                                deterministic_partition_effective_enabled,
                                &deterministic_partition_effective_group,
                            );
                        Ok((
                            ParseNode {
                                rule_name: "object",
                                content: result,
                                span: start_pos..end_pos,
                            },
                            semantic_raw_content,
                        ))
                    },
                );
            inner_result.map(|(node, _raw)| node)
        })(self);
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger_enabled {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "object", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "object", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "object", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "object",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "object", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_members(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("members", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "members", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "members", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 4096usize => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "members", position, depth
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
        self.recursion_guard.enter("members", position);
        let start_pos = self.position;
        let result: ParseResult<ParseNode<'input>> = (|parser: &mut Self| {
            let inner_result = parser
                .memoized_call(
                    Self::RULE_MEMBERS,
                    |parser| {
                        let semantic_capture_raw_for_post = parser
                            .semantic_runtime_annotations
                            .needs_raw_post_capture_for_rule("members");
                        let mut semantic_selected_branch_index: Option<usize> = None;
                        let mut semantic_raw_content: Option<ParseContent<'input>> = None;
                        let parse_start = parser.position;
                        let mut best_content: Option<ParseContent<'input>> = None;
                        let mut best_raw_content: Option<ParseContent<'input>> = None;
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
                                "members",
                                "rule.members",
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
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                1usize, 2usize, "members", parser.position
                                                            ),
                                                        );
                                                }
                                                let mut sequence_elements = Vec::with_capacity(3usize);
                                                {
                                                    let element_start = parser.position;
                                                    let element_content = {
                                                        let result = ParseContent::Alternative(
                                                            Box::new(parser.parse_pair()?),
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
                                                        let matched_str = parser.match_regex("\\s*,\\s*", true)?;
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
                                                            Box::new(parser.parse_members()?),
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
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                1usize, 2usize, "members", parser.position
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
                                            let raw_content = content;
                                            let transformed = {
                                                let content = raw_content.clone();
                                                {
                                                    let mut array_elements = Vec::new();
                                                    array_elements
                                                        .push(ParseNode {
                                                            rule_name: "element_0",
                                                            content: {
                                                                match &content {
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
                                                                    other => other.clone(),
                                                                }
                                                            },
                                                            span: 0..0,
                                                        });
                                                    match {
                                                        match &content {
                                                            ParseContent::Sequence(
                                                                elements,
                                                            ) if elements.len() > 2usize => {
                                                                elements[2usize].content.clone()
                                                            }
                                                            ParseContent::Quantified(
                                                                elements,
                                                                _,
                                                            ) if elements.len() > 2usize => {
                                                                elements[2usize].content.clone()
                                                            }
                                                            _ => ParseContent::Terminal("<invalid_sequence_access>"),
                                                        }
                                                    } {
                                                        ParseContent::Sequence(nodes) => {
                                                            for node in nodes {
                                                                array_elements.push(node);
                                                            }
                                                        }
                                                        ParseContent::Quantified(nodes, _) => {
                                                            for node in nodes {
                                                                array_elements.push(node);
                                                            }
                                                        }
                                                        other => {
                                                            array_elements
                                                                .push(ParseNode {
                                                                    rule_name: "spread_element",
                                                                    content: other,
                                                                    span: 0..0,
                                                                });
                                                        }
                                                    }
                                                    ParseContent::Sequence(array_elements)
                                                }
                                            };
                                            let mut branch_predicate_blocked = false;
                                            let mut blocked_branch_predicate: Option<String> = None;
                                            for directive in parser
                                                .semantic_runtime_annotations
                                                .branch_predicates_for_rule("members")
                                                .chain(
                                                    parser
                                                        .semantic_runtime_annotations
                                                        .branch_predicates_for_rule_branch(
                                                            "members",
                                                            current_branch_index,
                                                        ),
                                                )
                                            {
                                                match directive {
                                                    crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        spec,
                                                    ) if spec.phase
                                                        == crate::ast_pipeline::SemanticPredicatePhase::Branch => {
                                                        let Some(resolved_spec) = parser
                                                            .try_resolve_semantic_predicate_spec_against_content(
                                                                spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )? else {
                                                            blocked_branch_predicate = Some(
                                                                parser.semantic_predicate_debug_label(spec),
                                                            );
                                                            branch_predicate_blocked = true;
                                                            break;
                                                        };
                                                        match parser
                                                            .semantic_runtime_state
                                                            .evaluate_content_aware_predicate(
                                                                &resolved_spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )
                                                        {
                                                            Some(true) => {}
                                                            Some(false) => {
                                                                blocked_branch_predicate = Some(
                                                                    parser.semantic_predicate_debug_label(&resolved_spec),
                                                                );
                                                                branch_predicate_blocked = true;
                                                                break;
                                                            }
                                                            None => {}
                                                        }
                                                    }
                                                    crate::ast_pipeline::SemanticRuntimeDirective::OpenScope(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::CloseScope(
                                                        _,
                                                    )
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::EmitFact(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        _,
                                                    ) => {}
                                                }
                                            }
                                            let should_take = if branch_predicate_blocked {
                                                false
                                            } else if "longest_match" == "ordered" {
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
                                                if semantic_capture_raw_for_post {
                                                    best_raw_content = Some(raw_content.clone());
                                                }
                                                best_content = Some(transformed);
                                            } else if branch_predicate_blocked && parser.logger_enabled
                                            {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚫 Branch {}/{} for rule '{}' rejected by branch predicate '{}' at position {}",
                                                            1usize, 2usize, "members", blocked_branch_predicate
                                                            .as_deref().unwrap_or("<unknown>"), candidate_end
                                                        ),
                                                    );
                                            }
                                        } else if parser.logger_enabled {
                                            parser
                                                .logger
                                                .log_info(
                                                    "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                    0,
                                                    &format!(
                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                        1usize, 2usize, "members", parser.position
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
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                2usize, 2usize, "members", parser.position
                                                            ),
                                                        );
                                                }
                                                let result = ParseContent::Alternative(
                                                    Box::new(parser.parse_pair()?),
                                                );
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                2usize, 2usize, "members", parser.position
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
                                            let raw_content = content;
                                            let transformed = {
                                                let content = raw_content.clone();
                                                {
                                                    let mut array_elements = Vec::new();
                                                    array_elements
                                                        .push(ParseNode {
                                                            rule_name: "element_0",
                                                            content: {
                                                                match &content {
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
                                                                    other => other.clone(),
                                                                }
                                                            },
                                                            span: 0..0,
                                                        });
                                                    ParseContent::Sequence(array_elements)
                                                }
                                            };
                                            let mut branch_predicate_blocked = false;
                                            let mut blocked_branch_predicate: Option<String> = None;
                                            for directive in parser
                                                .semantic_runtime_annotations
                                                .branch_predicates_for_rule("members")
                                                .chain(
                                                    parser
                                                        .semantic_runtime_annotations
                                                        .branch_predicates_for_rule_branch(
                                                            "members",
                                                            current_branch_index,
                                                        ),
                                                )
                                            {
                                                match directive {
                                                    crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        spec,
                                                    ) if spec.phase
                                                        == crate::ast_pipeline::SemanticPredicatePhase::Branch => {
                                                        let Some(resolved_spec) = parser
                                                            .try_resolve_semantic_predicate_spec_against_content(
                                                                spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )? else {
                                                            blocked_branch_predicate = Some(
                                                                parser.semantic_predicate_debug_label(spec),
                                                            );
                                                            branch_predicate_blocked = true;
                                                            break;
                                                        };
                                                        match parser
                                                            .semantic_runtime_state
                                                            .evaluate_content_aware_predicate(
                                                                &resolved_spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )
                                                        {
                                                            Some(true) => {}
                                                            Some(false) => {
                                                                blocked_branch_predicate = Some(
                                                                    parser.semantic_predicate_debug_label(&resolved_spec),
                                                                );
                                                                branch_predicate_blocked = true;
                                                                break;
                                                            }
                                                            None => {}
                                                        }
                                                    }
                                                    crate::ast_pipeline::SemanticRuntimeDirective::OpenScope(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::CloseScope(
                                                        _,
                                                    )
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::EmitFact(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        _,
                                                    ) => {}
                                                }
                                            }
                                            let should_take = if branch_predicate_blocked {
                                                false
                                            } else if "longest_match" == "ordered" {
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
                                                if semantic_capture_raw_for_post {
                                                    best_raw_content = Some(raw_content.clone());
                                                }
                                                best_content = Some(transformed);
                                            } else if branch_predicate_blocked && parser.logger_enabled
                                            {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚫 Branch {}/{} for rule '{}' rejected by branch predicate '{}' at position {}",
                                                            2usize, 2usize, "members", blocked_branch_predicate
                                                            .as_deref().unwrap_or("<unknown>"), candidate_end
                                                        ),
                                                    );
                                            }
                                        } else if parser.logger_enabled {
                                            parser
                                                .logger
                                                .log_info(
                                                    "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                    0,
                                                    &format!(
                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                        2usize, 2usize, "members", parser.position
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
                            if parser.logger_enabled {
                                parser
                                    .logger
                                    .log_info(
                                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                        0,
                                        &format!(
                                            "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                            "members", best_branch, 2usize, best_end
                                            .saturating_sub(parse_start), best_priority, "left",
                                            "longest_match"
                                        ),
                                    );
                            }
                            result = content;
                            semantic_raw_content = best_raw_content;
                        } else {
                            return Err(ParseError::Backtrack {
                                position: parse_start,
                            });
                        };
                        let end_pos = parser.position;
                        parser
                            .record_coverage_target_event(
                                "members",
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
                                "members",
                                "rule.members",
                            );
                        parser
                            .record_deterministic_partition_event(
                                "members",
                                start_pos,
                                end_pos,
                                deterministic_partition_effective_enabled,
                                &deterministic_partition_effective_group,
                            );
                        Ok((
                            ParseNode {
                                rule_name: "members",
                                content: result,
                                span: start_pos..end_pos,
                            },
                            semantic_raw_content,
                        ))
                    },
                );
            inner_result.map(|(node, _raw)| node)
        })(self);
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger_enabled {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "members", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "members", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "members", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "members",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "members", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_pair(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("pair", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "pair", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "pair", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 4096usize => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "pair", position, depth
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
        self.recursion_guard.enter("pair", position);
        let start_pos = self.position;
        let result: ParseResult<ParseNode<'input>> = (|parser: &mut Self| {
            let inner_result = parser
                .memoized_call(
                    Self::RULE_PAIR,
                    |parser| {
                        let semantic_capture_raw_for_post = parser
                            .semantic_runtime_annotations
                            .needs_raw_post_capture_for_rule("pair");
                        let mut semantic_selected_branch_index: Option<usize> = None;
                        let mut semantic_raw_content: Option<ParseContent<'input>> = None;
                        let mut sequence_elements = Vec::with_capacity(3usize);
                        {
                            let element_start = parser.position;
                            let element_content = {
                                let result = ParseContent::Alternative(
                                    Box::new(parser.parse_string()?),
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
                                let matched_str = parser.match_regex("\\s*:\\s*", true)?;
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
                                    Box::new(parser.parse_value()?),
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
                        let result = {
                            {
                                let mut __pgen_obj = serde_json::Map::new();
                                __pgen_obj
                                    .insert(
                                        "key".to_string(),
                                        {
                                            let __pgen_content = {
                                                match &result {
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
                                                    other => other.clone(),
                                                }
                                            };
                                            __pgen_content.to_json_value()
                                        },
                                    );
                                __pgen_obj
                                    .insert(
                                        "type".to_string(),
                                        serde_json::Value::String("pair".to_string()),
                                    );
                                __pgen_obj
                                    .insert(
                                        "value".to_string(),
                                        {
                                            let __pgen_content = {
                                                match &result {
                                                    ParseContent::Sequence(
                                                        elements,
                                                    ) if elements.len() > 2usize => {
                                                        elements[2usize].content.clone()
                                                    }
                                                    ParseContent::Quantified(
                                                        elements,
                                                        _,
                                                    ) if elements.len() > 2usize => {
                                                        elements[2usize].content.clone()
                                                    }
                                                    _ => ParseContent::Terminal("<invalid_sequence_access>"),
                                                }
                                            };
                                            __pgen_content.to_json_value()
                                        },
                                    );
                                ParseContent::Json(serde_json::Value::Object(__pgen_obj))
                            }
                        };
                        let end_pos = parser.position;
                        parser
                            .record_coverage_target_event(
                                "pair",
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
                                "pair",
                                "rule.pair",
                            );
                        parser
                            .record_deterministic_partition_event(
                                "pair",
                                start_pos,
                                end_pos,
                                deterministic_partition_effective_enabled,
                                &deterministic_partition_effective_group,
                            );
                        Ok((
                            ParseNode {
                                rule_name: "pair",
                                content: result,
                                span: start_pos..end_pos,
                            },
                            semantic_raw_content,
                        ))
                    },
                );
            inner_result.map(|(node, _raw)| node)
        })(self);
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger_enabled {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "pair", start_pos, node.span.end, consumed, consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "pair", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "pair", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "pair",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "pair", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_array(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("array", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "array", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "array", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 4096usize => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "array", position, depth
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
        self.recursion_guard.enter("array", position);
        let start_pos = self.position;
        let result: ParseResult<ParseNode<'input>> = (|parser: &mut Self| {
            let inner_result = parser
                .memoized_call(
                    Self::RULE_ARRAY,
                    |parser| {
                        let semantic_capture_raw_for_post = parser
                            .semantic_runtime_annotations
                            .needs_raw_post_capture_for_rule("array");
                        let mut semantic_selected_branch_index: Option<usize> = None;
                        let mut semantic_raw_content: Option<ParseContent<'input>> = None;
                        let parse_start = parser.position;
                        let mut best_content: Option<ParseContent<'input>> = None;
                        let mut best_raw_content: Option<ParseContent<'input>> = None;
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
                                "array",
                                "rule.array",
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
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                1usize, 2usize, "array", parser.position
                                                            ),
                                                        );
                                                }
                                                let matched_str = parser
                                                    .match_regex("\\s*\\[\\s*\\]\\s*", true)?;
                                                let result = ParseContent::Terminal(matched_str);
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                1usize, 2usize, "array", parser.position
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
                                            let raw_content = content;
                                            let transformed = {
                                                let content = raw_content.clone();
                                                {
                                                    let mut __pgen_obj = serde_json::Map::new();
                                                    __pgen_obj
                                                        .insert(
                                                            "elements".to_string(),
                                                            {
                                                                let __pgen_content = {
                                                                    let mut array_elements = Vec::new();
                                                                    ParseContent::Sequence(array_elements)
                                                                };
                                                                __pgen_content.to_json_value()
                                                            },
                                                        );
                                                    __pgen_obj
                                                        .insert(
                                                            "type".to_string(),
                                                            serde_json::Value::String("array".to_string()),
                                                        );
                                                    ParseContent::Json(serde_json::Value::Object(__pgen_obj))
                                                }
                                            };
                                            let mut branch_predicate_blocked = false;
                                            let mut blocked_branch_predicate: Option<String> = None;
                                            for directive in parser
                                                .semantic_runtime_annotations
                                                .branch_predicates_for_rule("array")
                                                .chain(
                                                    parser
                                                        .semantic_runtime_annotations
                                                        .branch_predicates_for_rule_branch(
                                                            "array",
                                                            current_branch_index,
                                                        ),
                                                )
                                            {
                                                match directive {
                                                    crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        spec,
                                                    ) if spec.phase
                                                        == crate::ast_pipeline::SemanticPredicatePhase::Branch => {
                                                        let Some(resolved_spec) = parser
                                                            .try_resolve_semantic_predicate_spec_against_content(
                                                                spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )? else {
                                                            blocked_branch_predicate = Some(
                                                                parser.semantic_predicate_debug_label(spec),
                                                            );
                                                            branch_predicate_blocked = true;
                                                            break;
                                                        };
                                                        match parser
                                                            .semantic_runtime_state
                                                            .evaluate_content_aware_predicate(
                                                                &resolved_spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )
                                                        {
                                                            Some(true) => {}
                                                            Some(false) => {
                                                                blocked_branch_predicate = Some(
                                                                    parser.semantic_predicate_debug_label(&resolved_spec),
                                                                );
                                                                branch_predicate_blocked = true;
                                                                break;
                                                            }
                                                            None => {}
                                                        }
                                                    }
                                                    crate::ast_pipeline::SemanticRuntimeDirective::OpenScope(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::CloseScope(
                                                        _,
                                                    )
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::EmitFact(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        _,
                                                    ) => {}
                                                }
                                            }
                                            let should_take = if branch_predicate_blocked {
                                                false
                                            } else if "longest_match" == "ordered" {
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
                                                if semantic_capture_raw_for_post {
                                                    best_raw_content = Some(raw_content.clone());
                                                }
                                                best_content = Some(transformed);
                                            } else if branch_predicate_blocked && parser.logger_enabled
                                            {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚫 Branch {}/{} for rule '{}' rejected by branch predicate '{}' at position {}",
                                                            1usize, 2usize, "array", blocked_branch_predicate.as_deref()
                                                            .unwrap_or("<unknown>"), candidate_end
                                                        ),
                                                    );
                                            }
                                        } else if parser.logger_enabled {
                                            parser
                                                .logger
                                                .log_info(
                                                    "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                    0,
                                                    &format!(
                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                        1usize, 2usize, "array", parser.position
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
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                2usize, 2usize, "array", parser.position
                                                            ),
                                                        );
                                                }
                                                let mut sequence_elements = Vec::with_capacity(3usize);
                                                {
                                                    let element_start = parser.position;
                                                    let element_content = {
                                                        let matched_str = parser.match_regex("\\s*\\[\\s*", true)?;
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
                                                            Box::new(parser.parse_elements()?),
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
                                                        let matched_str = parser.match_regex("\\s*\\]\\s*", true)?;
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
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                2usize, 2usize, "array", parser.position
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
                                            let raw_content = content;
                                            let transformed = {
                                                let content = raw_content.clone();
                                                {
                                                    let mut __pgen_obj = serde_json::Map::new();
                                                    __pgen_obj
                                                        .insert(
                                                            "elements".to_string(),
                                                            {
                                                                let __pgen_content = {
                                                                    match &content {
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
                                                                };
                                                                __pgen_content.to_json_value()
                                                            },
                                                        );
                                                    __pgen_obj
                                                        .insert(
                                                            "type".to_string(),
                                                            serde_json::Value::String("array".to_string()),
                                                        );
                                                    ParseContent::Json(serde_json::Value::Object(__pgen_obj))
                                                }
                                            };
                                            let mut branch_predicate_blocked = false;
                                            let mut blocked_branch_predicate: Option<String> = None;
                                            for directive in parser
                                                .semantic_runtime_annotations
                                                .branch_predicates_for_rule("array")
                                                .chain(
                                                    parser
                                                        .semantic_runtime_annotations
                                                        .branch_predicates_for_rule_branch(
                                                            "array",
                                                            current_branch_index,
                                                        ),
                                                )
                                            {
                                                match directive {
                                                    crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        spec,
                                                    ) if spec.phase
                                                        == crate::ast_pipeline::SemanticPredicatePhase::Branch => {
                                                        let Some(resolved_spec) = parser
                                                            .try_resolve_semantic_predicate_spec_against_content(
                                                                spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )? else {
                                                            blocked_branch_predicate = Some(
                                                                parser.semantic_predicate_debug_label(spec),
                                                            );
                                                            branch_predicate_blocked = true;
                                                            break;
                                                        };
                                                        match parser
                                                            .semantic_runtime_state
                                                            .evaluate_content_aware_predicate(
                                                                &resolved_spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )
                                                        {
                                                            Some(true) => {}
                                                            Some(false) => {
                                                                blocked_branch_predicate = Some(
                                                                    parser.semantic_predicate_debug_label(&resolved_spec),
                                                                );
                                                                branch_predicate_blocked = true;
                                                                break;
                                                            }
                                                            None => {}
                                                        }
                                                    }
                                                    crate::ast_pipeline::SemanticRuntimeDirective::OpenScope(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::CloseScope(
                                                        _,
                                                    )
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::EmitFact(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        _,
                                                    ) => {}
                                                }
                                            }
                                            let should_take = if branch_predicate_blocked {
                                                false
                                            } else if "longest_match" == "ordered" {
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
                                                if semantic_capture_raw_for_post {
                                                    best_raw_content = Some(raw_content.clone());
                                                }
                                                best_content = Some(transformed);
                                            } else if branch_predicate_blocked && parser.logger_enabled
                                            {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚫 Branch {}/{} for rule '{}' rejected by branch predicate '{}' at position {}",
                                                            2usize, 2usize, "array", blocked_branch_predicate.as_deref()
                                                            .unwrap_or("<unknown>"), candidate_end
                                                        ),
                                                    );
                                            }
                                        } else if parser.logger_enabled {
                                            parser
                                                .logger
                                                .log_info(
                                                    "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                    0,
                                                    &format!(
                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                        2usize, 2usize, "array", parser.position
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
                            if parser.logger_enabled {
                                parser
                                    .logger
                                    .log_info(
                                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                        0,
                                        &format!(
                                            "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                            "array", best_branch, 2usize, best_end
                                            .saturating_sub(parse_start), best_priority, "left",
                                            "longest_match"
                                        ),
                                    );
                            }
                            result = content;
                            semantic_raw_content = best_raw_content;
                        } else {
                            return Err(ParseError::Backtrack {
                                position: parse_start,
                            });
                        };
                        let end_pos = parser.position;
                        parser
                            .record_coverage_target_event(
                                "array",
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
                                "array",
                                "rule.array",
                            );
                        parser
                            .record_deterministic_partition_event(
                                "array",
                                start_pos,
                                end_pos,
                                deterministic_partition_effective_enabled,
                                &deterministic_partition_effective_group,
                            );
                        Ok((
                            ParseNode {
                                rule_name: "array",
                                content: result,
                                span: start_pos..end_pos,
                            },
                            semantic_raw_content,
                        ))
                    },
                );
            inner_result.map(|(node, _raw)| node)
        })(self);
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger_enabled {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "array", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "array", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "array", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "array",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "array", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_elements(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("elements", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "elements", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "elements", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 4096usize => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "elements", position, depth
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
        self.recursion_guard.enter("elements", position);
        let start_pos = self.position;
        let result: ParseResult<ParseNode<'input>> = (|parser: &mut Self| {
            let inner_result = parser
                .memoized_call(
                    Self::RULE_ELEMENTS,
                    |parser| {
                        let semantic_capture_raw_for_post = parser
                            .semantic_runtime_annotations
                            .needs_raw_post_capture_for_rule("elements");
                        let mut semantic_selected_branch_index: Option<usize> = None;
                        let mut semantic_raw_content: Option<ParseContent<'input>> = None;
                        let parse_start = parser.position;
                        let mut best_content: Option<ParseContent<'input>> = None;
                        let mut best_raw_content: Option<ParseContent<'input>> = None;
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
                                "elements",
                                "rule.elements",
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
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                1usize, 2usize, "elements", parser.position
                                                            ),
                                                        );
                                                }
                                                let mut sequence_elements = Vec::with_capacity(3usize);
                                                {
                                                    let element_start = parser.position;
                                                    let element_content = {
                                                        let result = ParseContent::Alternative(
                                                            Box::new(parser.parse_value()?),
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
                                                        let matched_str = parser.match_regex("\\s*,\\s*", true)?;
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
                                                            Box::new(parser.parse_elements()?),
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
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                1usize, 2usize, "elements", parser.position
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
                                            let raw_content = content;
                                            let transformed = {
                                                let content = raw_content.clone();
                                                {
                                                    let mut array_elements = Vec::new();
                                                    array_elements
                                                        .push(ParseNode {
                                                            rule_name: "element_0",
                                                            content: {
                                                                match &content {
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
                                                                    other => other.clone(),
                                                                }
                                                            },
                                                            span: 0..0,
                                                        });
                                                    match {
                                                        match &content {
                                                            ParseContent::Sequence(
                                                                elements,
                                                            ) if elements.len() > 2usize => {
                                                                elements[2usize].content.clone()
                                                            }
                                                            ParseContent::Quantified(
                                                                elements,
                                                                _,
                                                            ) if elements.len() > 2usize => {
                                                                elements[2usize].content.clone()
                                                            }
                                                            _ => ParseContent::Terminal("<invalid_sequence_access>"),
                                                        }
                                                    } {
                                                        ParseContent::Sequence(nodes) => {
                                                            for node in nodes {
                                                                array_elements.push(node);
                                                            }
                                                        }
                                                        ParseContent::Quantified(nodes, _) => {
                                                            for node in nodes {
                                                                array_elements.push(node);
                                                            }
                                                        }
                                                        other => {
                                                            array_elements
                                                                .push(ParseNode {
                                                                    rule_name: "spread_element",
                                                                    content: other,
                                                                    span: 0..0,
                                                                });
                                                        }
                                                    }
                                                    ParseContent::Sequence(array_elements)
                                                }
                                            };
                                            let mut branch_predicate_blocked = false;
                                            let mut blocked_branch_predicate: Option<String> = None;
                                            for directive in parser
                                                .semantic_runtime_annotations
                                                .branch_predicates_for_rule("elements")
                                                .chain(
                                                    parser
                                                        .semantic_runtime_annotations
                                                        .branch_predicates_for_rule_branch(
                                                            "elements",
                                                            current_branch_index,
                                                        ),
                                                )
                                            {
                                                match directive {
                                                    crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        spec,
                                                    ) if spec.phase
                                                        == crate::ast_pipeline::SemanticPredicatePhase::Branch => {
                                                        let Some(resolved_spec) = parser
                                                            .try_resolve_semantic_predicate_spec_against_content(
                                                                spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )? else {
                                                            blocked_branch_predicate = Some(
                                                                parser.semantic_predicate_debug_label(spec),
                                                            );
                                                            branch_predicate_blocked = true;
                                                            break;
                                                        };
                                                        match parser
                                                            .semantic_runtime_state
                                                            .evaluate_content_aware_predicate(
                                                                &resolved_spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )
                                                        {
                                                            Some(true) => {}
                                                            Some(false) => {
                                                                blocked_branch_predicate = Some(
                                                                    parser.semantic_predicate_debug_label(&resolved_spec),
                                                                );
                                                                branch_predicate_blocked = true;
                                                                break;
                                                            }
                                                            None => {}
                                                        }
                                                    }
                                                    crate::ast_pipeline::SemanticRuntimeDirective::OpenScope(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::CloseScope(
                                                        _,
                                                    )
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::EmitFact(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        _,
                                                    ) => {}
                                                }
                                            }
                                            let should_take = if branch_predicate_blocked {
                                                false
                                            } else if "longest_match" == "ordered" {
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
                                                if semantic_capture_raw_for_post {
                                                    best_raw_content = Some(raw_content.clone());
                                                }
                                                best_content = Some(transformed);
                                            } else if branch_predicate_blocked && parser.logger_enabled
                                            {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚫 Branch {}/{} for rule '{}' rejected by branch predicate '{}' at position {}",
                                                            1usize, 2usize, "elements", blocked_branch_predicate
                                                            .as_deref().unwrap_or("<unknown>"), candidate_end
                                                        ),
                                                    );
                                            }
                                        } else if parser.logger_enabled {
                                            parser
                                                .logger
                                                .log_info(
                                                    "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                    0,
                                                    &format!(
                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                        1usize, 2usize, "elements", parser.position
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
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "🚪 Entering branch {}/{} for rule '{}' at position {}",
                                                                2usize, 2usize, "elements", parser.position
                                                            ),
                                                        );
                                                }
                                                let result = ParseContent::Alternative(
                                                    Box::new(parser.parse_value()?),
                                                );
                                                if parser.logger_enabled {
                                                    parser
                                                        .logger
                                                        .log_info(
                                                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                            0,
                                                            &format!(
                                                                "✅ Leaving branch {}/{} for rule '{}' at position {} (success)",
                                                                2usize, 2usize, "elements", parser.position
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
                                            let raw_content = content;
                                            let transformed = {
                                                let content = raw_content.clone();
                                                {
                                                    let mut array_elements = Vec::new();
                                                    array_elements
                                                        .push(ParseNode {
                                                            rule_name: "element_0",
                                                            content: {
                                                                match &content {
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
                                                                    other => other.clone(),
                                                                }
                                                            },
                                                            span: 0..0,
                                                        });
                                                    ParseContent::Sequence(array_elements)
                                                }
                                            };
                                            let mut branch_predicate_blocked = false;
                                            let mut blocked_branch_predicate: Option<String> = None;
                                            for directive in parser
                                                .semantic_runtime_annotations
                                                .branch_predicates_for_rule("elements")
                                                .chain(
                                                    parser
                                                        .semantic_runtime_annotations
                                                        .branch_predicates_for_rule_branch(
                                                            "elements",
                                                            current_branch_index,
                                                        ),
                                                )
                                            {
                                                match directive {
                                                    crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        spec,
                                                    ) if spec.phase
                                                        == crate::ast_pipeline::SemanticPredicatePhase::Branch => {
                                                        let Some(resolved_spec) = parser
                                                            .try_resolve_semantic_predicate_spec_against_content(
                                                                spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )? else {
                                                            blocked_branch_predicate = Some(
                                                                parser.semantic_predicate_debug_label(spec),
                                                            );
                                                            branch_predicate_blocked = true;
                                                            break;
                                                        };
                                                        match parser
                                                            .semantic_runtime_state
                                                            .evaluate_content_aware_predicate(
                                                                &resolved_spec,
                                                                &raw_content,
                                                                &transformed,
                                                            )
                                                        {
                                                            Some(true) => {}
                                                            Some(false) => {
                                                                blocked_branch_predicate = Some(
                                                                    parser.semantic_predicate_debug_label(&resolved_spec),
                                                                );
                                                                branch_predicate_blocked = true;
                                                                break;
                                                            }
                                                            None => {}
                                                        }
                                                    }
                                                    crate::ast_pipeline::SemanticRuntimeDirective::OpenScope(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::CloseScope(
                                                        _,
                                                    )
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::EmitFact(_)
                                                    | crate::ast_pipeline::SemanticRuntimeDirective::Predicate(
                                                        _,
                                                    ) => {}
                                                }
                                            }
                                            let should_take = if branch_predicate_blocked {
                                                false
                                            } else if "longest_match" == "ordered" {
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
                                                if semantic_capture_raw_for_post {
                                                    best_raw_content = Some(raw_content.clone());
                                                }
                                                best_content = Some(transformed);
                                            } else if branch_predicate_blocked && parser.logger_enabled
                                            {
                                                parser
                                                    .logger
                                                    .log_info(
                                                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                        0,
                                                        &format!(
                                                            "🚫 Branch {}/{} for rule '{}' rejected by branch predicate '{}' at position {}",
                                                            2usize, 2usize, "elements", blocked_branch_predicate
                                                            .as_deref().unwrap_or("<unknown>"), candidate_end
                                                        ),
                                                    );
                                            }
                                        } else if parser.logger_enabled {
                                            parser
                                                .logger
                                                .log_info(
                                                    "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                                    0,
                                                    &format!(
                                                        "❌ Branch {}/{} for rule '{}' failed at position {}",
                                                        2usize, 2usize, "elements", parser.position
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
                            if parser.logger_enabled {
                                parser
                                    .logger
                                    .log_info(
                                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                        0,
                                        &format!(
                                            "🏁 Rule '{}' selected branch {}/{} consuming {} chars (priority={}, associativity={}, branch_policy={})",
                                            "elements", best_branch, 2usize, best_end
                                            .saturating_sub(parse_start), best_priority, "left",
                                            "longest_match"
                                        ),
                                    );
                            }
                            result = content;
                            semantic_raw_content = best_raw_content;
                        } else {
                            return Err(ParseError::Backtrack {
                                position: parse_start,
                            });
                        };
                        let end_pos = parser.position;
                        parser
                            .record_coverage_target_event(
                                "elements",
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
                                "elements",
                                "rule.elements",
                            );
                        parser
                            .record_deterministic_partition_event(
                                "elements",
                                start_pos,
                                end_pos,
                                deterministic_partition_effective_enabled,
                                &deterministic_partition_effective_group,
                            );
                        Ok((
                            ParseNode {
                                rule_name: "elements",
                                content: result,
                                span: start_pos..end_pos,
                            },
                            semantic_raw_content,
                        ))
                    },
                );
            inner_result.map(|(node, _raw)| node)
        })(self);
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger_enabled {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "elements", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "elements", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "elements", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "elements",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "elements", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_string(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("string", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "string", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "string", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 4096usize => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "string", position, depth
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
        self.recursion_guard.enter("string", position);
        let start_pos = self.position;
        let result: ParseResult<ParseNode<'input>> = (|parser: &mut Self| {
            let inner_result = parser
                .memoized_call(
                    Self::RULE_STRING,
                    |parser| {
                        let semantic_capture_raw_for_post = parser
                            .semantic_runtime_annotations
                            .needs_raw_post_capture_for_rule("string");
                        let mut semantic_selected_branch_index: Option<usize> = None;
                        let mut semantic_raw_content: Option<ParseContent<'input>> = None;
                        let matched_str = parser
                            .match_regex("\\s*\"[^\"]*\"\\s*", true)?;
                        let result = ParseContent::Terminal(matched_str);
                        let result = {
                            {
                                match &result {
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
                                    other => other.clone(),
                                }
                            }
                        };
                        let end_pos = parser.position;
                        parser
                            .record_coverage_target_event(
                                "string",
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
                                "string",
                                "rule.string",
                            );
                        parser
                            .record_deterministic_partition_event(
                                "string",
                                start_pos,
                                end_pos,
                                deterministic_partition_effective_enabled,
                                &deterministic_partition_effective_group,
                            );
                        Ok((
                            ParseNode {
                                rule_name: "string",
                                content: result,
                                span: start_pos..end_pos,
                            },
                            semantic_raw_content,
                        ))
                    },
                );
            inner_result.map(|(node, _raw)| node)
        })(self);
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger_enabled {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "string", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "string", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "string", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "string",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "string", e, self.position
                            ),
                        );
                }
            }
        }
        result
    }
    pub fn parse_number(&mut self) -> ParseResult<ParseNode<'input>> {
        let filename_str = "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs";
        let position = self.position;
        let cycle_type = self.recursion_guard.check_cycle("number", position);
        match cycle_type {
            CycleType::Infinite => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "💥 Infinite recursion detected in rule '{}' at position {}",
                                "number", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Infinite recursion detected",
                    position,
                });
            }
            CycleType::LeftRecursive => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "🔄 Left recursion detected in rule '{}' at position {}",
                                "number", position
                            ),
                        );
                }
                return Err(ParseError::InvalidSyntax {
                    message: "Left recursion detected",
                    position,
                });
            }
            CycleType::MutualRecursive { depth, ref rules } if depth >= 4096usize => {
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "🔃 Recursion depth exceeded in rule '{}' at position {} (depth: {})",
                                "number", position, depth
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
        self.recursion_guard.enter("number", position);
        let start_pos = self.position;
        let result: ParseResult<ParseNode<'input>> = (|parser: &mut Self| {
            let inner_result = parser
                .memoized_call(
                    Self::RULE_NUMBER,
                    |parser| {
                        let semantic_capture_raw_for_post = parser
                            .semantic_runtime_annotations
                            .needs_raw_post_capture_for_rule("number");
                        let mut semantic_selected_branch_index: Option<usize> = None;
                        let mut semantic_raw_content: Option<ParseContent<'input>> = None;
                        let matched_str = parser
                            .match_regex("\\s*-?[0-9]+(\\.[0-9]+)?\\s*", true)?;
                        let result = ParseContent::Terminal(matched_str);
                        let result = {
                            {
                                match &result {
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
                                    other => other.clone(),
                                }
                            }
                        };
                        let end_pos = parser.position;
                        parser
                            .record_coverage_target_event(
                                "number",
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
                                "number",
                                "rule.number",
                            );
                        parser
                            .record_deterministic_partition_event(
                                "number",
                                start_pos,
                                end_pos,
                                deterministic_partition_effective_enabled,
                                &deterministic_partition_effective_group,
                            );
                        Ok((
                            ParseNode {
                                rule_name: "number",
                                content: result,
                                span: start_pos..end_pos,
                            },
                            semantic_raw_content,
                        ))
                    },
                );
            inner_result.map(|(node, _raw)| node)
        })(self);
        self.recursion_guard.exit();
        match &result {
            Ok(node) => {
                if self.logger_enabled {
                    let consumed = node.span.end - start_pos;
                    if consumed > 0 {
                        let consumed_preview = self
                            .byte_window_lossy(start_pos, node.span.end);
                        self.logger
                            .log_success(
                                "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                0,
                                &format!(
                                    "✅ Rule '{}' successfully parsed from {} to {} (consumed {} bytes: '{}')",
                                    "number", start_pos, node.span.end, consumed,
                                    consumed_preview
                                ),
                            );
                    } else {
                        self.logger
                            .log_warning(
                                "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                                0,
                                &format!(
                                    "⚠️ Rule '{}' matched with zero length at position {}",
                                    "number", start_pos
                                ),
                            );
                    }
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "✅ Exiting rule '{}' successfully - advanced from {} to {}",
                                "number", start_pos, self.position
                            ),
                        );
                }
            }
            Err(e) => {
                if false {
                    self.record_negative_case_failure(
                        "number",
                        start_pos,
                        self.position,
                        false,
                        &format!("{:?}", e),
                    );
                }
                if self.logger_enabled {
                    self.logger
                        .log_error(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "❌ Exiting rule '{}' with error: {:?} - backtracked to {}",
                                "number", e, self.position
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
        if self.logger_enabled {
            let marker = if critical_path { "critical" } else { "target" };
            self.logger
                .log_info(
                    "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
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
        if self.logger_enabled {
            self.logger
                .log_info(
                    "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
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
        if self.logger_enabled {
            let mode = if negative { "near-invalid" } else { "invalid-case" };
            self.logger
                .log_info(
                    "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
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
                if self.logger_enabled {
                    self.logger
                        .log_warning(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
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
                if self.logger_enabled {
                    self.logger
                        .log_warning(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
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
                if self.logger_enabled {
                    self.logger
                        .log_warning(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
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
            if self.logger_enabled {
                let marker = if token_priority == 0 { "panic_until" } else { "sync" };
                self.logger
                    .log_warning(
                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
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
            if self.logger_enabled {
                self.logger
                    .log_warning(
                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
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
            let dollar_reference_body = core_reference[1..].trim();
            let dollar_reference_is_positional = dollar_reference_body
                .as_bytes()
                .first()
                .map(|byte| byte.is_ascii_digit())
                .unwrap_or(false);
            if dollar_reference_is_positional {
                self.resolve_positional_semantic_reference(root_content, core_reference)
            } else {
                self.resolve_named_semantic_reference(
                    root_content,
                    dollar_reference_body,
                )
            }
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
            ParseContent::Json(value) => {
                match value {
                    serde_json::Value::String(s) => Some(s.clone()),
                    serde_json::Value::Null => None,
                    other => Some(other.to_string()),
                }
            }
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
            let dollar_reference_body = normalized[1..].trim();
            if dollar_reference_body.is_empty() {
                return false;
            }
            let dollar_reference_is_positional = dollar_reference_body
                .as_bytes()
                .first()
                .map(|byte| byte.is_ascii_digit())
                .unwrap_or(false);
            if dollar_reference_is_positional {
                return self.parse_semantic_reference_segments(normalized).is_some();
            }
            let mut segments = dollar_reference_body.split('.');
            let Some(first) = segments.next() else {
                return false;
            };
            if !self.semantic_identifier(first) {
                return false;
            }
            return segments.all(|segment| self.semantic_identifier(segment));
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
        if true {
            self.consume_layout_for_terminal(expected);
        }
        let start = self.position;
        let expected_bytes = expected.as_bytes();
        let end = start + expected_bytes.len();
        if self.logger_enabled {
            self.logger
                .log_debug(
                    "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
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
            if self.logger_enabled {
                self.logger
                    .log_success(
                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                        0,
                        &format!(
                            "✅ Terminal '{}' matched, advanced to position {}",
                            expected, end
                        ),
                    );
            }
            return Ok(&self.input[start..end]);
        }
        if self.logger_enabled {
            let found_str = if self.position < self.input.len() {
                let end = (self.position + expected_bytes.len()).min(self.input.len());
                self.byte_window_lossy(self.position, end)
            } else {
                "<EOF>".to_string()
            };
            self.logger
                .log_error(
                    "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                    0,
                    &format!(
                        "❌ Terminal '{}' failed at position {} - found '{}'", expected,
                        start, found_str
                    ),
                );
        }
        Err(ParseError::Backtrack {
            position: start,
        })
    }
    fn match_regex(
        &mut self,
        pattern: &str,
        skip_leading_whitespace: bool,
    ) -> ParseResult<&'input str> {
        use std::cell::RefCell;
        use std::collections::HashMap;
        thread_local! {
            static REGEX_CACHE : RefCell < HashMap < String, regex::Regex >> =
            RefCell::new(HashMap::new());
        }
        let can_match_empty: bool = REGEX_CACHE
            .with(|cache| -> Result<bool, regex::Error> {
                let mut cache = cache.borrow_mut();
                if !cache.contains_key(pattern) {
                    let compiled = regex::Regex::new(pattern)?;
                    cache.insert(pattern.to_string(), compiled);
                }
                let re = cache.get(pattern).expect("just inserted");
                if true {
                    Ok(
                        re
                            .find("")
                            .map(|m| m.start() == 0 && m.end() == 0)
                            .unwrap_or(false),
                    )
                } else {
                    Ok(false)
                }
            })
            .map_err(|e| {
                self
                    .create_contextual_error(
                        &format!("Invalid regex pattern '{}': {}", pattern, e),
                    )
            })?;
        if skip_leading_whitespace && true {
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
        let match_end: Option<usize> = REGEX_CACHE
            .with(|cache| {
                let cache = cache.borrow();
                let re = cache.get(pattern).expect("compiled in phase 1");
                re.find(haystack).filter(|m| m.start() == 0).map(|m| m.end())
            });
        if let Some(end_offset) = match_end {
            let start = self.position;
            self.position += end_offset;
            if self.logger_enabled {
                self.logger
                    .log_success(
                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                        0,
                        &format!(
                            "✅ Regex '{}' matched at position {} (len {})", pattern,
                            start, end_offset
                        ),
                    );
            }
            if let Some(slice) = self.input.get(start..self.position) {
                return Ok(slice);
            }
            return Err(self.create_contextual_error("Regex matched invalid UTF-8 span"));
        }
        if self.logger_enabled {
            let preview = if self.position < self.input.len() {
                let end = (self.position + 10).min(self.input.len());
                self.byte_window_lossy(self.position, end)
            } else {
                "<EOF>".to_string()
            };
            self.logger
                .log_error(
                    "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
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
        if self.logger_enabled {
            self.logger
                .log_debug(
                    "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                    0,
                    &format!("🔄 Starting speculative parse at position {}", saved_pos),
                );
        }
        match f(self) {
            Ok(result) => {
                if self.logger_enabled {
                    self.logger
                        .log_success(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
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
                if self.logger_enabled {
                    self.logger
                        .log_warning(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
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
    ) -> ParseResult<(ParseNode<'input>, Option<ParseContent<'input>>)>
    where
        F: FnOnce(
            &mut Self,
        ) -> ParseResult<(ParseNode<'input>, Option<ParseContent<'input>>)>,
    {
        let key = (rule_id, self.position);
        if let Some(entry) = self.memo.get(&key) {
            if let Some(node) = &entry.result {
                self.position = entry.end_pos;
                if self.logger_enabled {
                    self.logger
                        .log_info(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "💾 Memo hit for rule {} at position {} - reusing cached result",
                                rule_id, self.position
                            ),
                        );
                }
                return Ok((node.clone(), entry.raw_semantic_content.clone()));
            } else {
                if self.logger_enabled {
                    self.logger
                        .log_warning(
                            "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                            0,
                            &format!(
                                "💾 Memo miss for rule {} at position {} - cached failure",
                                rule_id, self.position
                            ),
                        );
                }
                self.position = entry.end_pos;
                return Err(ParseError::Backtrack {
                    position: entry.end_pos,
                });
            }
        }
        if self.logger_enabled {
            self.logger
                .log_debug(
                    "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                    0,
                    &format!(
                        "💾 Memo miss for rule {} at position {} - computing fresh result",
                        rule_id, self.position
                    ),
                );
        }
        let start_pos = key.1;
        let result = f(self);
        if let Ok((node, raw_semantic_content)) = &result {
            self.memo
                .insert(
                    key,
                    MemoEntry {
                        result: Some(node.clone()),
                        raw_semantic_content: raw_semantic_content.clone(),
                        end_pos: node.span.end,
                    },
                );
            if self.logger_enabled {
                self.logger
                    .log_info(
                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
                        0,
                        &format!(
                            "💾 Memoized successful result for rule {} at position {}",
                            rule_id, self.position
                        ),
                    );
            }
        } else {
            self.memo
                .insert(
                    key,
                    MemoEntry {
                        result: None,
                        raw_semantic_content: None,
                        end_pos: start_pos,
                    },
                );
            if self.logger_enabled {
                self.logger
                    .log_warning(
                        "/Users/richarddje/Documents/github/pgen/generated/json_parser.rs",
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
        let rule_stack: Vec<&'static str> = self
            .recursion_guard
            .parse_stack
            .iter()
            .map(|(rule, _)| *rule)
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
        let mut parser = JsonParser::new(input, logger);
        let _ = parser.parse();
    }
}
