use super::{
    Annotations, SemanticAnnotation, UnifiedSemanticAST, UnifiedSemanticProperty,
    UnifiedSemanticValue,
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticScopeKind {
    Global,
    File,
    Package,
    Class,
    Interface,
    Type,
    Function,
    Task,
    Block,
    Custom(String),
}

impl SemanticScopeKind {
    fn parse(value: &UnifiedSemanticValue) -> Option<Self> {
        let normalized = scalar_text(value)?.to_ascii_lowercase();
        match normalized.as_str() {
            "global" => Some(Self::Global),
            "file" => Some(Self::File),
            "package" => Some(Self::Package),
            "class" => Some(Self::Class),
            "interface" => Some(Self::Interface),
            "type" => Some(Self::Type),
            "function" => Some(Self::Function),
            "task" => Some(Self::Task),
            "block" => Some(Self::Block),
            other if !other.is_empty() => Some(Self::Custom(other.to_string())),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticRuntimeValue {
    String(String),
    Identifier(String),
    RuleReference(String),
    Number(String),
    Boolean(bool),
    Null,
}

impl SemanticRuntimeValue {
    fn from_semantic_value(value: &UnifiedSemanticValue) -> Option<Self> {
        match value {
            UnifiedSemanticValue::String(text) => Some(Self::String(text.clone())),
            UnifiedSemanticValue::Identifier(text) => Some(Self::Identifier(text.clone())),
            UnifiedSemanticValue::RuleReference(text) => Some(Self::RuleReference(text.clone())),
            UnifiedSemanticValue::Number(text) => Some(Self::Number(text.clone())),
            UnifiedSemanticValue::Boolean(value) => Some(Self::Boolean(*value)),
            UnifiedSemanticValue::Null => Some(Self::Null),
            UnifiedSemanticValue::Array(_) | UnifiedSemanticValue::Object(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticScopeSpec {
    pub kind: SemanticScopeKind,
    pub name: Option<SemanticRuntimeValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticCloseScopeSpec {
    pub kind: Option<SemanticScopeKind>,
    pub name: Option<SemanticRuntimeValue>,
}

impl SemanticCloseScopeSpec {
    fn matches(&self, frame: &SemanticScopeFrame) -> bool {
        if let Some(kind) = &self.kind {
            if kind != &frame.kind {
                return false;
            }
        }
        if let Some(name) = &self.name {
            if Some(name) != frame.name.as_ref() {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticFactSpec {
    pub kind: String,
    pub name: SemanticRuntimeValue,
    pub attributes: Vec<UnifiedSemanticProperty>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticPredicateSpec {
    pub name: String,
    pub args: Vec<UnifiedSemanticValue>,
    pub phase: SemanticPredicatePhase,
    pub view: SemanticPredicateContentView,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SemanticPredicatePhase {
    #[default]
    Pre,
    Branch,
    Post,
}

impl SemanticPredicatePhase {
    fn parse(value: &UnifiedSemanticValue) -> Option<Self> {
        let normalized = scalar_text(value)?.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "pre" | "rule_entry" | "rule-entry" => Some(Self::Pre),
            "branch" | "branch_local" | "branch-local" => Some(Self::Branch),
            "post" | "rule_exit" | "rule-exit" => Some(Self::Post),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SemanticPredicateContentView {
    #[default]
    Raw,
    Shaped,
}

impl SemanticPredicateContentView {
    fn parse(value: &UnifiedSemanticValue) -> Option<Self> {
        let normalized = scalar_text(value)?.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "raw" => Some(Self::Raw),
            "shaped" => Some(Self::Shaped),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticRuntimeDirective {
    OpenScope(SemanticScopeSpec),
    CloseScope(SemanticCloseScopeSpec),
    EmitFact(SemanticFactSpec),
    Predicate(SemanticPredicateSpec),
}

impl SemanticRuntimeDirective {
    pub fn predicate_phase(&self) -> Option<SemanticPredicatePhase> {
        match self {
            Self::Predicate(spec) => Some(spec.phase),
            Self::OpenScope(_) | Self::CloseScope(_) | Self::EmitFact(_) => None,
        }
    }

    pub fn is_pre_predicate(&self) -> bool {
        self.predicate_phase() == Some(SemanticPredicatePhase::Pre)
    }

    pub fn is_effect(&self) -> bool {
        matches!(
            self,
            Self::OpenScope(_) | Self::CloseScope(_) | Self::EmitFact(_)
        )
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CompiledSemanticRuntimeAnnotations {
    directives_by_rule: HashMap<String, Vec<SemanticRuntimeDirective>>,
}

impl CompiledSemanticRuntimeAnnotations {
    pub fn compile(annotations: &Annotations) -> Result<Self, String> {
        compile_semantic_runtime_annotations(annotations)
    }

    pub fn from_rule_directives(
        directives_by_rule: HashMap<String, Vec<SemanticRuntimeDirective>>,
    ) -> Self {
        Self { directives_by_rule }
    }

    pub fn is_empty(&self) -> bool {
        self.directives_by_rule.is_empty()
    }

    pub fn len(&self) -> usize {
        self.directives_by_rule.len()
    }

    pub fn has_rule(&self, rule_name: &str) -> bool {
        self.directives_by_rule.contains_key(rule_name)
    }

    pub fn directives_for_rule(&self, rule_name: &str) -> &[SemanticRuntimeDirective] {
        self.directives_by_rule
            .get(rule_name)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn pre_predicates_for_rule<'a>(
        &'a self,
        rule_name: &'a str,
    ) -> impl Iterator<Item = &'a SemanticRuntimeDirective> + 'a {
        self.directives_for_rule(rule_name)
            .iter()
            .filter(|directive| directive.is_pre_predicate())
    }

    pub fn effect_directives_for_rule<'a>(
        &'a self,
        rule_name: &'a str,
    ) -> impl Iterator<Item = &'a SemanticRuntimeDirective> + 'a {
        self.directives_for_rule(rule_name)
            .iter()
            .filter(|directive| directive.is_effect())
    }

    pub fn apply_to_rule(&self, state: &mut SemanticRuntimeState, rule_name: &str) -> usize {
        state.apply_compiled_rule(self, rule_name)
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (&str, &[SemanticRuntimeDirective])> + ExactSizeIterator + '_ {
        self.directives_by_rule
            .iter()
            .map(|(rule_name, directives)| (rule_name.as_str(), directives.as_slice()))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticScopeFrame {
    pub kind: SemanticScopeKind,
    pub name: Option<SemanticRuntimeValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticFactRecord {
    pub kind: String,
    pub name: SemanticRuntimeValue,
    pub scope_depth: usize,
    pub attributes: Vec<UnifiedSemanticProperty>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticRuntimeCheckpoint {
    scope_len: usize,
    fact_len: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticRuntimeState {
    scopes: Vec<SemanticScopeFrame>,
    facts: Vec<SemanticFactRecord>,
}

#[derive(Debug)]
pub struct SemanticRuntimeTransaction<'a> {
    state: &'a mut SemanticRuntimeState,
    checkpoint: SemanticRuntimeCheckpoint,
    committed: bool,
}

impl Default for SemanticRuntimeState {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticRuntimeState {
    pub fn new() -> Self {
        Self {
            scopes: vec![SemanticScopeFrame {
                kind: SemanticScopeKind::Global,
                name: None,
            }],
            facts: Vec::new(),
        }
    }

    pub fn scopes(&self) -> &[SemanticScopeFrame] {
        &self.scopes
    }

    pub fn facts(&self) -> &[SemanticFactRecord] {
        &self.facts
    }

    pub fn current_scope(&self) -> &SemanticScopeFrame {
        self.scopes
            .last()
            .expect("semantic runtime state always maintains at least the global scope")
    }

    pub fn transaction(&mut self) -> SemanticRuntimeTransaction<'_> {
        let checkpoint = self.checkpoint();
        SemanticRuntimeTransaction {
            state: self,
            checkpoint,
            committed: false,
        }
    }

    pub fn transaction_for_rule<'a>(
        &'a mut self,
        compiled: &CompiledSemanticRuntimeAnnotations,
        rule_name: &str,
    ) -> (SemanticRuntimeTransaction<'a>, usize) {
        let mut transaction = self.transaction();
        let applied = transaction.apply_compiled_rule(compiled, rule_name);
        (transaction, applied)
    }

    pub fn checkpoint(&self) -> SemanticRuntimeCheckpoint {
        SemanticRuntimeCheckpoint {
            scope_len: self.scopes.len(),
            fact_len: self.facts.len(),
        }
    }

    pub fn rollback_to(&mut self, checkpoint: SemanticRuntimeCheckpoint) {
        let scope_len = checkpoint.scope_len.max(1).min(self.scopes.len());
        let fact_len = checkpoint.fact_len.min(self.facts.len());
        self.scopes.truncate(scope_len);
        self.facts.truncate(fact_len);
    }

    pub fn commit(&self, checkpoint: SemanticRuntimeCheckpoint) -> bool {
        checkpoint.scope_len <= self.scopes.len() && checkpoint.fact_len <= self.facts.len()
    }

    pub fn open_scope(&mut self, spec: SemanticScopeSpec) {
        self.scopes.push(SemanticScopeFrame {
            kind: spec.kind,
            name: spec.name,
        });
    }

    pub fn close_scope(&mut self, spec: &SemanticCloseScopeSpec) -> bool {
        if self.scopes.len() <= 1 {
            return false;
        }
        if spec.matches(
            self.scopes
                .last()
                .expect("semantic runtime state always maintains at least the global scope"),
        ) {
            self.scopes.pop();
            true
        } else {
            false
        }
    }

    pub fn emit_fact(&mut self, fact: SemanticFactSpec) {
        self.facts.push(SemanticFactRecord {
            kind: fact.kind,
            name: fact.name,
            scope_depth: self.scopes.len() - 1,
            attributes: fact.attributes,
        });
    }

    pub fn apply_directive(&mut self, directive: &SemanticRuntimeDirective) -> bool {
        match directive {
            SemanticRuntimeDirective::OpenScope(spec) => {
                self.open_scope(spec.clone());
                true
            }
            SemanticRuntimeDirective::CloseScope(spec) => self.close_scope(spec),
            SemanticRuntimeDirective::EmitFact(spec) => {
                self.emit_fact(spec.clone());
                true
            }
            SemanticRuntimeDirective::Predicate(_) => true,
        }
    }

    pub fn apply_directives<'b, I>(&mut self, directives: I) -> usize
    where
        I: IntoIterator<Item = &'b SemanticRuntimeDirective>,
    {
        let mut applied = 0;
        for directive in directives {
            if self.apply_directive(directive) {
                applied += 1;
            }
        }
        applied
    }

    pub fn apply_compiled_rule(
        &mut self,
        compiled: &CompiledSemanticRuntimeAnnotations,
        rule_name: &str,
    ) -> usize {
        self.apply_directives(compiled.effect_directives_for_rule(rule_name))
    }

    pub fn evaluate_predicate(&self, predicate: &SemanticPredicateSpec) -> Option<bool> {
        let normalized_name = predicate.name.trim().to_ascii_lowercase();
        match normalized_name.as_str() {
            "current_scope_is" => {
                let expected_kind = predicate.args.first().and_then(SemanticScopeKind::parse)?;
                let current_scope = self.current_scope();
                if current_scope.kind != expected_kind {
                    return Some(false);
                }
                let Some(expected_name_arg) = predicate.args.get(1) else {
                    return Some(true);
                };
                let expected_name = SemanticRuntimeValue::from_semantic_value(expected_name_arg)?;
                Some(current_scope.name.as_ref() == Some(&expected_name))
            }
            "has_fact" => {
                let expected_kind = scalar_text(predicate.args.first()?)?;
                let expected_name =
                    SemanticRuntimeValue::from_semantic_value(predicate.args.get(1)?)?;
                Some(
                    self.facts.iter().any(|fact| {
                        fact.kind.eq_ignore_ascii_case(expected_kind) && fact.name == expected_name
                    }),
                )
            }
            "has_fact_in_current_scope" => {
                let expected_kind = scalar_text(predicate.args.first()?)?;
                let expected_name =
                    SemanticRuntimeValue::from_semantic_value(predicate.args.get(1)?)?;
                let current_depth = self.scopes.len().saturating_sub(1);
                Some(
                    self.facts.iter().any(|fact| {
                        fact.scope_depth == current_depth
                            && fact.kind.eq_ignore_ascii_case(expected_kind)
                            && fact.name == expected_name
                    }),
                )
            }
            "scope_depth_at_least" => {
                let minimum_depth = scalar_text(predicate.args.first()?)?.parse::<usize>().ok()?;
                Some(self.scopes.len().saturating_sub(1) >= minimum_depth)
            }
            _ => None,
        }
    }

    pub fn evaluate_directive_predicate(
        &self,
        directive: &SemanticRuntimeDirective,
    ) -> Option<bool> {
        match directive {
            SemanticRuntimeDirective::Predicate(spec)
                if spec.phase == SemanticPredicatePhase::Pre =>
            {
                self.evaluate_predicate(spec)
            }
            SemanticRuntimeDirective::OpenScope(_)
            | SemanticRuntimeDirective::CloseScope(_)
            | SemanticRuntimeDirective::EmitFact(_)
            | SemanticRuntimeDirective::Predicate(_) => None,
        }
    }
}

impl<'a> SemanticRuntimeTransaction<'a> {
    pub fn state(&self) -> &SemanticRuntimeState {
        self.state
    }

    pub fn checkpoint(&self) -> SemanticRuntimeCheckpoint {
        self.checkpoint
    }

    pub fn apply_directive(&mut self, directive: &SemanticRuntimeDirective) -> bool {
        self.state.apply_directive(directive)
    }

    pub fn apply_directives<'b, I>(&mut self, directives: I) -> usize
    where
        I: IntoIterator<Item = &'b SemanticRuntimeDirective>,
    {
        self.state.apply_directives(directives)
    }

    pub fn apply_compiled_rule(
        &mut self,
        compiled: &CompiledSemanticRuntimeAnnotations,
        rule_name: &str,
    ) -> usize {
        self.state.apply_compiled_rule(compiled, rule_name)
    }

    pub fn apply_annotations<'b, I>(
        &mut self,
        annotations: I,
    ) -> Result<Vec<SemanticRuntimeDirective>, String>
    where
        I: IntoIterator<Item = &'b SemanticAnnotation>,
    {
        let directives = parse_semantic_runtime_directives(annotations)?;
        self.apply_directives(directives.iter());
        Ok(directives)
    }

    pub fn rollback(mut self) {
        self.state.rollback_to(self.checkpoint);
        self.committed = true;
    }

    pub fn commit(mut self) -> bool {
        let committed = self.state.commit(self.checkpoint);
        self.committed = true;
        committed
    }
}

impl Drop for SemanticRuntimeTransaction<'_> {
    fn drop(&mut self) {
        if !self.committed {
            self.state.rollback_to(self.checkpoint);
        }
    }
}

pub fn parse_semantic_runtime_directive(
    annotation: &SemanticAnnotation,
) -> Result<Option<SemanticRuntimeDirective>, String> {
    let Some(name) = annotation.name() else {
        return Ok(None);
    };

    let normalized = name.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "open_scope" => parse_open_scope(annotation.ast()).map(Some),
        "close_scope" => parse_close_scope(annotation.ast()).map(Some),
        "emit_fact" => parse_emit_fact(annotation.ast()).map(Some),
        "predicate" => parse_predicate(annotation.ast()).map(Some),
        _ => Ok(None),
    }
}

pub fn parse_semantic_runtime_directives<'a, I>(
    annotations: I,
) -> Result<Vec<SemanticRuntimeDirective>, String>
where
    I: IntoIterator<Item = &'a SemanticAnnotation>,
{
    let mut directives = Vec::new();
    for annotation in annotations {
        if let Some(directive) = parse_semantic_runtime_directive(annotation)? {
            directives.push(directive);
        }
    }
    Ok(directives)
}

pub fn compile_rule_semantic_runtime_directives<'a, I>(
    rule_name: &str,
    annotations: I,
) -> Result<Vec<SemanticRuntimeDirective>, String>
where
    I: IntoIterator<Item = &'a SemanticAnnotation>,
{
    let mut directives = Vec::new();
    for (index, annotation) in annotations.into_iter().enumerate() {
        match parse_semantic_runtime_directive(annotation) {
            Ok(Some(directive)) => directives.push(directive),
            Ok(None) => {}
            Err(err) => {
                return Err(format!(
                    "Failed to compile semantic runtime directive for rule '{}' at annotation #{}: {}",
                    rule_name, index, err
                ));
            }
        }
    }
    Ok(directives)
}

pub fn compile_semantic_runtime_annotations(
    annotations: &Annotations,
) -> Result<CompiledSemanticRuntimeAnnotations, String> {
    let mut directives_by_rule = HashMap::new();
    for (rule_name, semantic_annotations) in &annotations.semantic_annotations {
        let directives =
            compile_rule_semantic_runtime_directives(rule_name, semantic_annotations.iter())?;
        if !directives.is_empty() {
            directives_by_rule.insert(rule_name.clone(), directives);
        }
    }
    Ok(CompiledSemanticRuntimeAnnotations { directives_by_rule })
}

fn parse_open_scope(ast: &UnifiedSemanticAST) -> Result<SemanticRuntimeDirective, String> {
    let payload = ast.structured_value().ok_or_else(|| {
        "Directive '@open_scope' expects a structured object payload.".to_string()
    })?;
    let properties = object_properties(payload)
        .ok_or_else(|| "Directive '@open_scope' expects an object payload.".to_string())?;
    let kind = property(properties, "kind")
        .ok_or_else(|| "Directive '@open_scope' requires a 'kind' field.".to_string())
        .and_then(|value| {
            SemanticScopeKind::parse(value).ok_or_else(|| {
                "Directive '@open_scope.kind' must be a known scope kind.".to_string()
            })
        })?;
    let name = property(properties, "name")
        .map(|value| {
            SemanticRuntimeValue::from_semantic_value(value).ok_or_else(|| {
                "Directive '@open_scope.name' must be a scalar or rule reference.".to_string()
            })
        })
        .transpose()?;

    Ok(SemanticRuntimeDirective::OpenScope(SemanticScopeSpec {
        kind,
        name,
    }))
}

fn parse_close_scope(ast: &UnifiedSemanticAST) -> Result<SemanticRuntimeDirective, String> {
    let payload = ast.structured_value().ok_or_else(|| {
        "Directive '@close_scope' expects a structured object payload.".to_string()
    })?;
    let properties = object_properties(payload)
        .ok_or_else(|| "Directive '@close_scope' expects an object payload.".to_string())?;

    let kind = property(properties, "kind")
        .map(|value| {
            SemanticScopeKind::parse(value).ok_or_else(|| {
                "Directive '@close_scope.kind' must be a known scope kind.".to_string()
            })
        })
        .transpose()?;
    let name = property(properties, "name")
        .map(|value| {
            SemanticRuntimeValue::from_semantic_value(value).ok_or_else(|| {
                "Directive '@close_scope.name' must be a scalar or rule reference.".to_string()
            })
        })
        .transpose()?;

    Ok(SemanticRuntimeDirective::CloseScope(
        SemanticCloseScopeSpec { kind, name },
    ))
}

fn parse_emit_fact(ast: &UnifiedSemanticAST) -> Result<SemanticRuntimeDirective, String> {
    let payload = ast
        .structured_value()
        .ok_or_else(|| "Directive '@emit_fact' expects a structured object payload.".to_string())?;
    let properties = object_properties(payload)
        .ok_or_else(|| "Directive '@emit_fact' expects an object payload.".to_string())?;

    let kind = property(properties, "kind")
        .and_then(scalar_text)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Directive '@emit_fact' requires a non-empty 'kind' field.".to_string())?
        .to_string();
    let name = property(properties, "name")
        .ok_or_else(|| "Directive '@emit_fact' requires a 'name' field.".to_string())
        .and_then(|value| {
            SemanticRuntimeValue::from_semantic_value(value).ok_or_else(|| {
                "Directive '@emit_fact.name' must be a scalar or rule reference.".to_string()
            })
        })?;

    let mut attributes = match property(properties, "attributes") {
        Some(UnifiedSemanticValue::Object(properties)) => properties.clone(),
        Some(_) => {
            return Err(
                "Directive '@emit_fact.attributes' must be an object when present.".to_string(),
            );
        }
        None => Vec::new(),
    };

    for property in properties {
        if matches!(property.key.as_str(), "kind" | "name" | "attributes") {
            continue;
        }
        attributes.push(property.clone());
    }

    Ok(SemanticRuntimeDirective::EmitFact(SemanticFactSpec {
        kind,
        name,
        attributes,
    }))
}

fn parse_predicate(ast: &UnifiedSemanticAST) -> Result<SemanticRuntimeDirective, String> {
    if let Some(payload) = ast.structured_value() {
        match payload {
            UnifiedSemanticValue::String(text) | UnifiedSemanticValue::Identifier(text) => {
                return Ok(SemanticRuntimeDirective::Predicate(SemanticPredicateSpec {
                    name: text.clone(),
                    args: Vec::new(),
                    phase: SemanticPredicatePhase::Pre,
                    view: SemanticPredicateContentView::Raw,
                }));
            }
            UnifiedSemanticValue::Object(properties) => {
                let name = property(properties, "name")
                    .and_then(scalar_text)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| {
                        "Directive '@predicate' requires a non-empty 'name' field.".to_string()
                    })?
                    .to_string();
                let args = match property(properties, "args") {
                    Some(UnifiedSemanticValue::Array(values)) => values.clone(),
                    Some(_) => {
                        return Err("Directive '@predicate.args' must be an array when present."
                            .to_string());
                    }
                    None => Vec::new(),
                };
                let phase = match property(properties, "phase") {
                    Some(value) => SemanticPredicatePhase::parse(value).ok_or_else(|| {
                        "Directive '@predicate.phase' must be one of 'pre', 'branch', or 'post'."
                            .to_string()
                    })?,
                    None => SemanticPredicatePhase::Pre,
                };
                let view = match property(properties, "view") {
                    Some(value) => {
                        SemanticPredicateContentView::parse(value).ok_or_else(|| {
                            "Directive '@predicate.view' must be either 'raw' or 'shaped'."
                                .to_string()
                        })?
                    }
                    None => SemanticPredicateContentView::Raw,
                };
                return Ok(SemanticRuntimeDirective::Predicate(SemanticPredicateSpec {
                    name,
                    args,
                    phase,
                    view,
                }));
            }
            _ => {}
        }
    }

    Err("Directive '@predicate' expects a predicate name or structured object payload.".to_string())
}

fn object_properties(value: &UnifiedSemanticValue) -> Option<&[UnifiedSemanticProperty]> {
    match value {
        UnifiedSemanticValue::Object(properties) => Some(properties.as_slice()),
        _ => None,
    }
}

fn property<'a>(
    properties: &'a [UnifiedSemanticProperty],
    key: &str,
) -> Option<&'a UnifiedSemanticValue> {
    properties
        .iter()
        .find(|property| property.key.eq_ignore_ascii_case(key))
        .map(|property| &property.value)
}

fn scalar_text(value: &UnifiedSemanticValue) -> Option<&str> {
    match value {
        UnifiedSemanticValue::String(text)
        | UnifiedSemanticValue::Identifier(text)
        | UnifiedSemanticValue::RuleReference(text)
        | UnifiedSemanticValue::Number(text) => Some(text.as_str()),
        UnifiedSemanticValue::Boolean(_) | UnifiedSemanticValue::Null => None,
        UnifiedSemanticValue::Array(_) | UnifiedSemanticValue::Object(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compile_rule_semantic_runtime_directives, compile_semantic_runtime_annotations,
        parse_semantic_runtime_directive, parse_semantic_runtime_directives,
        CompiledSemanticRuntimeAnnotations, SemanticPredicateContentView, SemanticPredicatePhase,
        SemanticPredicateSpec, SemanticRuntimeDirective, SemanticRuntimeState,
        SemanticRuntimeValue, SemanticScopeKind,
    };
    use crate::ast_pipeline::{
        Annotations, SemanticAnnotation, UnifiedSemanticAST, UnifiedSemanticValue,
    };

    fn structured_named(
        name: &str,
        canonical: &str,
        value: UnifiedSemanticValue,
    ) -> SemanticAnnotation {
        SemanticAnnotation::Named {
            name: name.to_string(),
            ast: UnifiedSemanticAST::Structured {
                canonical: canonical.to_string(),
                value,
            },
        }
    }

    #[test]
    fn parses_open_scope_and_emit_fact_runtime_directives() {
        let open_scope = structured_named(
            "open_scope",
            "{ kind: package, name: $1 }",
            UnifiedSemanticValue::Object(vec![
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "kind".to_string(),
                    value: UnifiedSemanticValue::Identifier("package".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "name".to_string(),
                    value: UnifiedSemanticValue::RuleReference("$1".to_string()),
                },
            ]),
        );
        let emit_fact = structured_named(
            "emit_fact",
            "{ kind: typedef, name: $2, declared_in: current_scope }",
            UnifiedSemanticValue::Object(vec![
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "kind".to_string(),
                    value: UnifiedSemanticValue::Identifier("typedef".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "name".to_string(),
                    value: UnifiedSemanticValue::RuleReference("$2".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "declared_in".to_string(),
                    value: UnifiedSemanticValue::Identifier("current_scope".to_string()),
                },
            ]),
        );

        let parsed_scope =
            parse_semantic_runtime_directive(&open_scope).expect("open_scope should parse");
        assert!(matches!(
            parsed_scope,
            Some(SemanticRuntimeDirective::OpenScope(ref spec))
                if spec.kind == SemanticScopeKind::Package
                    && spec.name == Some(SemanticRuntimeValue::RuleReference("$1".to_string()))
        ));

        let parsed_fact =
            parse_semantic_runtime_directive(&emit_fact).expect("emit_fact should parse");
        assert!(matches!(
            parsed_fact,
            Some(SemanticRuntimeDirective::EmitFact(ref spec))
                if spec.kind == "typedef"
                    && spec.name == SemanticRuntimeValue::RuleReference("$2".to_string())
                    && spec.attributes.iter().any(|property| property.key == "declared_in")
        ));
    }

    #[test]
    fn parses_predicate_runtime_directive() {
        let predicate = structured_named(
            "predicate",
            "{ name: is_block_declaration_start, args: [$1, lhs] }",
            UnifiedSemanticValue::Object(vec![
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "name".to_string(),
                    value: UnifiedSemanticValue::Identifier(
                        "is_block_declaration_start".to_string(),
                    ),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "args".to_string(),
                    value: UnifiedSemanticValue::Array(vec![
                        UnifiedSemanticValue::RuleReference("$1".to_string()),
                        UnifiedSemanticValue::Identifier("lhs".to_string()),
                    ]),
                },
            ]),
        );

        let parsed = parse_semantic_runtime_directive(&predicate).expect("predicate should parse");
        assert_eq!(
            parsed,
            Some(SemanticRuntimeDirective::Predicate(SemanticPredicateSpec {
                name: "is_block_declaration_start".to_string(),
                args: vec![
                    UnifiedSemanticValue::RuleReference("$1".to_string()),
                    UnifiedSemanticValue::Identifier("lhs".to_string()),
                ],
                phase: SemanticPredicatePhase::Pre,
                view: SemanticPredicateContentView::Raw,
            }))
        );
    }

    #[test]
    fn parses_predicate_runtime_directive_with_explicit_phase_and_view() {
        let predicate = structured_named(
            "predicate",
            "{ name: current_scope_is, args: [package], phase: post, view: shaped }",
            UnifiedSemanticValue::Object(vec![
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "name".to_string(),
                    value: UnifiedSemanticValue::Identifier("current_scope_is".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "args".to_string(),
                    value: UnifiedSemanticValue::Array(vec![UnifiedSemanticValue::Identifier(
                        "package".to_string(),
                    )]),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "phase".to_string(),
                    value: UnifiedSemanticValue::Identifier("post".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "view".to_string(),
                    value: UnifiedSemanticValue::Identifier("shaped".to_string()),
                },
            ]),
        );

        let parsed = parse_semantic_runtime_directive(&predicate).expect("predicate should parse");
        assert_eq!(
            parsed,
            Some(SemanticRuntimeDirective::Predicate(SemanticPredicateSpec {
                name: "current_scope_is".to_string(),
                args: vec![UnifiedSemanticValue::Identifier("package".to_string())],
                phase: SemanticPredicatePhase::Post,
                view: SemanticPredicateContentView::Shaped,
            }))
        );
    }

    #[test]
    fn built_in_predicates_query_current_scope_and_facts() {
        let open_scope = structured_named(
            "open_scope",
            "{ kind: package, name: top_pkg }",
            UnifiedSemanticValue::Object(vec![
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "kind".to_string(),
                    value: UnifiedSemanticValue::Identifier("package".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "name".to_string(),
                    value: UnifiedSemanticValue::Identifier("top_pkg".to_string()),
                },
            ]),
        );
        let emit_fact = structured_named(
            "emit_fact",
            "{ kind: typedef, name: my_type }",
            UnifiedSemanticValue::Object(vec![
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "kind".to_string(),
                    value: UnifiedSemanticValue::Identifier("typedef".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "name".to_string(),
                    value: UnifiedSemanticValue::Identifier("my_type".to_string()),
                },
            ]),
        );

        let open_directive = parse_semantic_runtime_directive(&open_scope)
            .expect("open_scope should parse")
            .expect("directive should be present");
        let fact_directive = parse_semantic_runtime_directive(&emit_fact)
            .expect("emit_fact should parse")
            .expect("directive should be present");

        let mut state = SemanticRuntimeState::new();
        assert!(state.apply_directive(&open_directive));
        assert!(state.apply_directive(&fact_directive));

        assert_eq!(
            state.evaluate_predicate(&SemanticPredicateSpec {
                name: "current_scope_is".to_string(),
                args: vec![
                    UnifiedSemanticValue::Identifier("package".to_string()),
                    UnifiedSemanticValue::Identifier("top_pkg".to_string()),
                ],
                phase: SemanticPredicatePhase::Pre,
                view: SemanticPredicateContentView::Raw,
            }),
            Some(true)
        );
        assert_eq!(
            state.evaluate_predicate(&SemanticPredicateSpec {
                name: "current_scope_is".to_string(),
                args: vec![UnifiedSemanticValue::Identifier("class".to_string())],
                phase: SemanticPredicatePhase::Pre,
                view: SemanticPredicateContentView::Raw,
            }),
            Some(false)
        );
        assert_eq!(
            state.evaluate_predicate(&SemanticPredicateSpec {
                name: "has_fact".to_string(),
                args: vec![
                    UnifiedSemanticValue::Identifier("typedef".to_string()),
                    UnifiedSemanticValue::Identifier("my_type".to_string()),
                ],
                phase: SemanticPredicatePhase::Pre,
                view: SemanticPredicateContentView::Raw,
            }),
            Some(true)
        );
        assert_eq!(
            state.evaluate_predicate(&SemanticPredicateSpec {
                name: "has_fact_in_current_scope".to_string(),
                args: vec![
                    UnifiedSemanticValue::Identifier("typedef".to_string()),
                    UnifiedSemanticValue::Identifier("my_type".to_string()),
                ],
                phase: SemanticPredicatePhase::Pre,
                view: SemanticPredicateContentView::Raw,
            }),
            Some(true)
        );
        assert_eq!(
            state.evaluate_predicate(&SemanticPredicateSpec {
                name: "scope_depth_at_least".to_string(),
                args: vec![UnifiedSemanticValue::Number("1".to_string())],
                phase: SemanticPredicatePhase::Pre,
                view: SemanticPredicateContentView::Raw,
            }),
            Some(true)
        );
    }

    #[test]
    fn built_in_predicates_respect_scope_changes_and_unknowns() {
        let open_scope = structured_named(
            "open_scope",
            "{ kind: package, name: top_pkg }",
            UnifiedSemanticValue::Object(vec![
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "kind".to_string(),
                    value: UnifiedSemanticValue::Identifier("package".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "name".to_string(),
                    value: UnifiedSemanticValue::Identifier("top_pkg".to_string()),
                },
            ]),
        );
        let emit_fact = structured_named(
            "emit_fact",
            "{ kind: typedef, name: my_type }",
            UnifiedSemanticValue::Object(vec![
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "kind".to_string(),
                    value: UnifiedSemanticValue::Identifier("typedef".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "name".to_string(),
                    value: UnifiedSemanticValue::Identifier("my_type".to_string()),
                },
            ]),
        );
        let close_scope = structured_named(
            "close_scope",
            "{ kind: package, name: top_pkg }",
            UnifiedSemanticValue::Object(vec![
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "kind".to_string(),
                    value: UnifiedSemanticValue::Identifier("package".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "name".to_string(),
                    value: UnifiedSemanticValue::Identifier("top_pkg".to_string()),
                },
            ]),
        );

        let open_directive = parse_semantic_runtime_directive(&open_scope)
            .expect("open_scope should parse")
            .expect("directive should be present");
        let fact_directive = parse_semantic_runtime_directive(&emit_fact)
            .expect("emit_fact should parse")
            .expect("directive should be present");
        let close_directive = parse_semantic_runtime_directive(&close_scope)
            .expect("close_scope should parse")
            .expect("directive should be present");

        let mut state = SemanticRuntimeState::new();
        assert!(state.apply_directive(&open_directive));
        assert!(state.apply_directive(&fact_directive));
        assert!(state.apply_directive(&close_directive));

        assert_eq!(
            state.evaluate_predicate(&SemanticPredicateSpec {
                name: "current_scope_is".to_string(),
                args: vec![UnifiedSemanticValue::Identifier("global".to_string())],
                phase: SemanticPredicatePhase::Pre,
                view: SemanticPredicateContentView::Raw,
            }),
            Some(true)
        );
        assert_eq!(
            state.evaluate_predicate(&SemanticPredicateSpec {
                name: "has_fact_in_current_scope".to_string(),
                args: vec![
                    UnifiedSemanticValue::Identifier("typedef".to_string()),
                    UnifiedSemanticValue::Identifier("my_type".to_string()),
                ],
                phase: SemanticPredicatePhase::Pre,
                view: SemanticPredicateContentView::Raw,
            }),
            Some(false)
        );
        assert_eq!(
            state.evaluate_directive_predicate(&SemanticRuntimeDirective::Predicate(
                SemanticPredicateSpec {
                    name: "unknown_predicate".to_string(),
                    args: vec![],
                    phase: SemanticPredicatePhase::Pre,
                    view: SemanticPredicateContentView::Raw,
                },
            )),
            None
        );
        assert_eq!(
            state.evaluate_predicate(&SemanticPredicateSpec {
                name: "scope_depth_at_least".to_string(),
                args: vec![UnifiedSemanticValue::Identifier("not_a_number".to_string())],
                phase: SemanticPredicatePhase::Pre,
                view: SemanticPredicateContentView::Raw,
            }),
            None
        );
    }

    #[test]
    fn rule_entry_predicate_evaluation_ignores_non_pre_phases() {
        let state = SemanticRuntimeState::new();
        assert_eq!(
            state.evaluate_directive_predicate(&SemanticRuntimeDirective::Predicate(
                SemanticPredicateSpec {
                    name: "current_scope_is".to_string(),
                    args: vec![UnifiedSemanticValue::Identifier("global".to_string())],
                    phase: SemanticPredicatePhase::Branch,
                    view: SemanticPredicateContentView::Raw,
                },
            )),
            None
        );
        assert_eq!(
            state.evaluate_directive_predicate(&SemanticRuntimeDirective::Predicate(
                SemanticPredicateSpec {
                    name: "current_scope_is".to_string(),
                    args: vec![UnifiedSemanticValue::Identifier("global".to_string())],
                    phase: SemanticPredicatePhase::Post,
                    view: SemanticPredicateContentView::Shaped,
                },
            )),
            None
        );
    }

    #[test]
    fn runtime_state_tracks_scope_stack_and_facts() {
        let open_scope = structured_named(
            "open_scope",
            "{ kind: package, name: top_pkg }",
            UnifiedSemanticValue::Object(vec![
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "kind".to_string(),
                    value: UnifiedSemanticValue::Identifier("package".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "name".to_string(),
                    value: UnifiedSemanticValue::Identifier("top_pkg".to_string()),
                },
            ]),
        );
        let emit_fact = structured_named(
            "emit_fact",
            "{ kind: typedef, name: my_type }",
            UnifiedSemanticValue::Object(vec![
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "kind".to_string(),
                    value: UnifiedSemanticValue::Identifier("typedef".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "name".to_string(),
                    value: UnifiedSemanticValue::Identifier("my_type".to_string()),
                },
            ]),
        );
        let close_scope = structured_named(
            "close_scope",
            "{ kind: package, name: top_pkg }",
            UnifiedSemanticValue::Object(vec![
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "kind".to_string(),
                    value: UnifiedSemanticValue::Identifier("package".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "name".to_string(),
                    value: UnifiedSemanticValue::Identifier("top_pkg".to_string()),
                },
            ]),
        );

        let mut state = SemanticRuntimeState::new();
        let open_directive = parse_semantic_runtime_directive(&open_scope)
            .expect("open_scope should parse")
            .expect("directive should be present");
        let fact_directive = parse_semantic_runtime_directive(&emit_fact)
            .expect("emit_fact should parse")
            .expect("directive should be present");
        let close_directive = parse_semantic_runtime_directive(&close_scope)
            .expect("close_scope should parse")
            .expect("directive should be present");

        assert!(state.apply_directive(&open_directive));
        assert!(state.apply_directive(&fact_directive));
        assert!(state.apply_directive(&close_directive));

        assert_eq!(state.scopes().len(), 1);
        assert_eq!(state.facts().len(), 1);
        assert_eq!(state.facts()[0].kind, "typedef");
        assert_eq!(
            state.facts()[0].name,
            SemanticRuntimeValue::Identifier("my_type".to_string())
        );
        assert_eq!(state.facts()[0].scope_depth, 1);
    }

    #[test]
    fn runtime_state_rolls_back_speculative_changes_to_checkpoint() {
        let open_scope = structured_named(
            "open_scope",
            "{ kind: package, name: trial_pkg }",
            UnifiedSemanticValue::Object(vec![
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "kind".to_string(),
                    value: UnifiedSemanticValue::Identifier("package".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "name".to_string(),
                    value: UnifiedSemanticValue::Identifier("trial_pkg".to_string()),
                },
            ]),
        );
        let emit_fact = structured_named(
            "emit_fact",
            "{ kind: typedef, name: transient_type }",
            UnifiedSemanticValue::Object(vec![
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "kind".to_string(),
                    value: UnifiedSemanticValue::Identifier("typedef".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "name".to_string(),
                    value: UnifiedSemanticValue::Identifier("transient_type".to_string()),
                },
            ]),
        );

        let mut state = SemanticRuntimeState::new();
        let checkpoint = state.checkpoint();
        let open_directive = parse_semantic_runtime_directive(&open_scope)
            .expect("open_scope should parse")
            .expect("directive should be present");
        let fact_directive = parse_semantic_runtime_directive(&emit_fact)
            .expect("emit_fact should parse")
            .expect("directive should be present");

        assert!(state.apply_directive(&open_directive));
        assert!(state.apply_directive(&fact_directive));
        assert_eq!(state.scopes().len(), 2);
        assert_eq!(state.facts().len(), 1);

        state.rollback_to(checkpoint);

        assert_eq!(state.scopes().len(), 1);
        assert_eq!(state.facts().len(), 0);
        assert_eq!(state.current_scope().kind, SemanticScopeKind::Global);
    }

    #[test]
    fn runtime_state_commit_keeps_accumulated_changes() {
        let open_scope = structured_named(
            "open_scope",
            "{ kind: package, name: committed_pkg }",
            UnifiedSemanticValue::Object(vec![
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "kind".to_string(),
                    value: UnifiedSemanticValue::Identifier("package".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "name".to_string(),
                    value: UnifiedSemanticValue::Identifier("committed_pkg".to_string()),
                },
            ]),
        );

        let mut state = SemanticRuntimeState::new();
        let checkpoint = state.checkpoint();
        let open_directive = parse_semantic_runtime_directive(&open_scope)
            .expect("open_scope should parse")
            .expect("directive should be present");

        assert!(state.apply_directive(&open_directive));
        assert!(state.commit(checkpoint));
        assert_eq!(state.scopes().len(), 2);
        assert_eq!(
            state.current_scope().name,
            Some(SemanticRuntimeValue::Identifier(
                "committed_pkg".to_string()
            ))
        );
    }

    #[test]
    fn transaction_rolls_back_on_drop_without_commit() {
        let open_scope = structured_named(
            "open_scope",
            "{ kind: package, name: dropped_pkg }",
            UnifiedSemanticValue::Object(vec![
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "kind".to_string(),
                    value: UnifiedSemanticValue::Identifier("package".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "name".to_string(),
                    value: UnifiedSemanticValue::Identifier("dropped_pkg".to_string()),
                },
            ]),
        );

        let mut state = SemanticRuntimeState::new();
        let directive = parse_semantic_runtime_directive(&open_scope)
            .expect("open_scope should parse")
            .expect("directive should be present");

        {
            let mut transaction = state.transaction();
            assert!(transaction.apply_directive(&directive));
            assert_eq!(transaction.state().scopes().len(), 2);
        }

        assert_eq!(state.scopes().len(), 1);
        assert_eq!(state.current_scope().kind, SemanticScopeKind::Global);
    }

    #[test]
    fn transaction_apply_annotations_filters_non_runtime_directives() {
        let annotations = vec![
            structured_named(
                "priority",
                "[1, 2]",
                UnifiedSemanticValue::Array(vec![
                    UnifiedSemanticValue::Number("1".to_string()),
                    UnifiedSemanticValue::Number("2".to_string()),
                ]),
            ),
            structured_named(
                "open_scope",
                "{ kind: package, name: batched_pkg }",
                UnifiedSemanticValue::Object(vec![
                    crate::ast_pipeline::UnifiedSemanticProperty {
                        key: "kind".to_string(),
                        value: UnifiedSemanticValue::Identifier("package".to_string()),
                    },
                    crate::ast_pipeline::UnifiedSemanticProperty {
                        key: "name".to_string(),
                        value: UnifiedSemanticValue::Identifier("batched_pkg".to_string()),
                    },
                ]),
            ),
            structured_named(
                "emit_fact",
                "{ kind: typedef, name: batched_type }",
                UnifiedSemanticValue::Object(vec![
                    crate::ast_pipeline::UnifiedSemanticProperty {
                        key: "kind".to_string(),
                        value: UnifiedSemanticValue::Identifier("typedef".to_string()),
                    },
                    crate::ast_pipeline::UnifiedSemanticProperty {
                        key: "name".to_string(),
                        value: UnifiedSemanticValue::Identifier("batched_type".to_string()),
                    },
                ]),
            ),
        ];

        let directives =
            parse_semantic_runtime_directives(annotations.iter()).expect("directives should parse");
        assert_eq!(directives.len(), 2);

        let mut state = SemanticRuntimeState::new();
        {
            let mut transaction = state.transaction();
            let applied = transaction
                .apply_annotations(annotations.iter())
                .expect("runtime annotations should apply");
            assert_eq!(applied.len(), 2);
            assert!(transaction.commit());
        }

        assert_eq!(state.scopes().len(), 2);
        assert_eq!(state.facts().len(), 1);
        assert_eq!(state.facts()[0].kind, "typedef");
    }

    #[test]
    fn compile_rule_directives_preserves_runtime_order_and_skips_other_annotations() {
        let annotations = vec![
            structured_named(
                "category",
                "\"metadata\"",
                UnifiedSemanticValue::String("metadata".to_string()),
            ),
            structured_named(
                "open_scope",
                "{ kind: package, name: batched_pkg }",
                UnifiedSemanticValue::Object(vec![
                    crate::ast_pipeline::UnifiedSemanticProperty {
                        key: "kind".to_string(),
                        value: UnifiedSemanticValue::Identifier("package".to_string()),
                    },
                    crate::ast_pipeline::UnifiedSemanticProperty {
                        key: "name".to_string(),
                        value: UnifiedSemanticValue::Identifier("batched_pkg".to_string()),
                    },
                ]),
            ),
            structured_named(
                "emit_fact",
                "{ kind: typedef, name: batched_type }",
                UnifiedSemanticValue::Object(vec![
                    crate::ast_pipeline::UnifiedSemanticProperty {
                        key: "kind".to_string(),
                        value: UnifiedSemanticValue::Identifier("typedef".to_string()),
                    },
                    crate::ast_pipeline::UnifiedSemanticProperty {
                        key: "name".to_string(),
                        value: UnifiedSemanticValue::Identifier("batched_type".to_string()),
                    },
                ]),
            ),
        ];

        let directives =
            compile_rule_semantic_runtime_directives("package_declaration", annotations.iter())
                .expect("runtime directives should compile");

        assert_eq!(directives.len(), 2);
        assert!(matches!(
            directives.first(),
            Some(SemanticRuntimeDirective::OpenScope(_))
        ));
        assert!(matches!(
            directives.get(1),
            Some(SemanticRuntimeDirective::EmitFact(_))
        ));
    }

    #[test]
    fn compile_annotations_groups_runtime_directives_by_rule() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "package_declaration".to_string(),
            vec![
                structured_named(
                    "open_scope",
                    "{ kind: package, name: batched_pkg }",
                    UnifiedSemanticValue::Object(vec![
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "kind".to_string(),
                            value: UnifiedSemanticValue::Identifier("package".to_string()),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "name".to_string(),
                            value: UnifiedSemanticValue::Identifier("batched_pkg".to_string()),
                        },
                    ]),
                ),
                structured_named(
                    "emit_fact",
                    "{ kind: typedef, name: batched_type }",
                    UnifiedSemanticValue::Object(vec![
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "kind".to_string(),
                            value: UnifiedSemanticValue::Identifier("typedef".to_string()),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "name".to_string(),
                            value: UnifiedSemanticValue::Identifier("batched_type".to_string()),
                        },
                    ]),
                ),
            ],
        );
        annotations.semantic_annotations.insert(
            "metadata_only_rule".to_string(),
            vec![structured_named(
                "category",
                "\"metadata\"",
                UnifiedSemanticValue::String("metadata".to_string()),
            )],
        );
        annotations.semantic_annotations.insert(
            "function_declaration".to_string(),
            vec![structured_named(
                "predicate",
                "{ name: is_type_reference_start, args: [lhs] }",
                UnifiedSemanticValue::Object(vec![
                    crate::ast_pipeline::UnifiedSemanticProperty {
                        key: "name".to_string(),
                        value: UnifiedSemanticValue::Identifier(
                            "is_type_reference_start".to_string(),
                        ),
                    },
                    crate::ast_pipeline::UnifiedSemanticProperty {
                        key: "args".to_string(),
                        value: UnifiedSemanticValue::Array(vec![UnifiedSemanticValue::Identifier(
                            "lhs".to_string(),
                        )]),
                    },
                ]),
            )],
        );

        let compiled = compile_semantic_runtime_annotations(&annotations)
            .expect("compiled semantic runtime annotations should succeed");

        assert_eq!(compiled.len(), 2);
        assert!(compiled.has_rule("package_declaration"));
        assert!(compiled.has_rule("function_declaration"));
        assert!(!compiled.has_rule("metadata_only_rule"));
        assert_eq!(compiled.directives_for_rule("package_declaration").len(), 2);
        assert_eq!(
            compiled.directives_for_rule("function_declaration").len(),
            1
        );
        assert!(compiled.directives_for_rule("missing_rule").is_empty());
    }

    #[test]
    fn compiled_annotations_split_pre_predicates_from_effects() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "package_declaration".to_string(),
            vec![
                structured_named(
                    "predicate",
                    "{ name: current_scope_is, args: [global] }",
                    UnifiedSemanticValue::Object(vec![
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "name".to_string(),
                            value: UnifiedSemanticValue::Identifier(
                                "current_scope_is".to_string(),
                            ),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "args".to_string(),
                            value: UnifiedSemanticValue::Array(vec![
                                UnifiedSemanticValue::Identifier("global".to_string()),
                            ]),
                        },
                    ]),
                ),
                structured_named(
                    "predicate",
                    "{ name: current_scope_is, args: [package], phase: post, view: shaped }",
                    UnifiedSemanticValue::Object(vec![
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "name".to_string(),
                            value: UnifiedSemanticValue::Identifier(
                                "current_scope_is".to_string(),
                            ),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "args".to_string(),
                            value: UnifiedSemanticValue::Array(vec![
                                UnifiedSemanticValue::Identifier("package".to_string()),
                            ]),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "phase".to_string(),
                            value: UnifiedSemanticValue::Identifier("post".to_string()),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "view".to_string(),
                            value: UnifiedSemanticValue::Identifier("shaped".to_string()),
                        },
                    ]),
                ),
                structured_named(
                    "open_scope",
                    "{ kind: package, name: pkg }",
                    UnifiedSemanticValue::Object(vec![
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "kind".to_string(),
                            value: UnifiedSemanticValue::Identifier("package".to_string()),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "name".to_string(),
                            value: UnifiedSemanticValue::Identifier("pkg".to_string()),
                        },
                    ]),
                ),
            ],
        );

        let compiled = compile_semantic_runtime_annotations(&annotations)
            .expect("compiled semantic runtime annotations should succeed");
        let pre_predicates = compiled
            .pre_predicates_for_rule("package_declaration")
            .collect::<Vec<_>>();
        let effects = compiled
            .effect_directives_for_rule("package_declaration")
            .collect::<Vec<_>>();

        assert_eq!(pre_predicates.len(), 1);
        assert_eq!(effects.len(), 1);
        assert!(matches!(
            pre_predicates[0],
            SemanticRuntimeDirective::Predicate(SemanticPredicateSpec {
                phase: SemanticPredicatePhase::Pre,
                ..
            })
        ));
        assert!(matches!(
            effects[0],
            SemanticRuntimeDirective::OpenScope(_)
        ));
    }

    #[test]
    fn compile_annotations_reports_rule_context_for_invalid_runtime_payloads() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "package_declaration".to_string(),
            vec![structured_named(
                "open_scope",
                "{ name: broken_pkg }",
                UnifiedSemanticValue::Object(vec![crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "name".to_string(),
                    value: UnifiedSemanticValue::Identifier("broken_pkg".to_string()),
                }]),
            )],
        );

        let err = compile_semantic_runtime_annotations(&annotations)
            .expect_err("invalid runtime payload should fail compilation");

        assert!(err.contains("rule 'package_declaration'"));
        assert!(err.contains("annotation #0"));
        assert!(err.contains("@open_scope"));
    }

    #[test]
    fn compiled_annotation_wrapper_matches_free_function() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "package_declaration".to_string(),
            vec![structured_named(
                "emit_fact",
                "{ kind: typedef, name: batched_type }",
                UnifiedSemanticValue::Object(vec![
                    crate::ast_pipeline::UnifiedSemanticProperty {
                        key: "kind".to_string(),
                        value: UnifiedSemanticValue::Identifier("typedef".to_string()),
                    },
                    crate::ast_pipeline::UnifiedSemanticProperty {
                        key: "name".to_string(),
                        value: UnifiedSemanticValue::Identifier("batched_type".to_string()),
                    },
                ]),
            )],
        );

        let via_wrapper = CompiledSemanticRuntimeAnnotations::compile(&annotations)
            .expect("wrapper compilation should succeed");
        let via_function = compile_semantic_runtime_annotations(&annotations)
            .expect("free-function compilation should succeed");

        assert_eq!(via_wrapper, via_function);
    }

    #[test]
    fn compiled_annotations_apply_only_requested_rule() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "package_declaration".to_string(),
            vec![
                structured_named(
                    "open_scope",
                    "{ kind: package, name: pkg_a }",
                    UnifiedSemanticValue::Object(vec![
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "kind".to_string(),
                            value: UnifiedSemanticValue::Identifier("package".to_string()),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "name".to_string(),
                            value: UnifiedSemanticValue::Identifier("pkg_a".to_string()),
                        },
                    ]),
                ),
                structured_named(
                    "emit_fact",
                    "{ kind: typedef, name: type_a }",
                    UnifiedSemanticValue::Object(vec![
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "kind".to_string(),
                            value: UnifiedSemanticValue::Identifier("typedef".to_string()),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "name".to_string(),
                            value: UnifiedSemanticValue::Identifier("type_a".to_string()),
                        },
                    ]),
                ),
            ],
        );
        annotations.semantic_annotations.insert(
            "function_declaration".to_string(),
            vec![structured_named(
                "emit_fact",
                "{ kind: function, name: fn_b }",
                UnifiedSemanticValue::Object(vec![
                    crate::ast_pipeline::UnifiedSemanticProperty {
                        key: "kind".to_string(),
                        value: UnifiedSemanticValue::Identifier("function".to_string()),
                    },
                    crate::ast_pipeline::UnifiedSemanticProperty {
                        key: "name".to_string(),
                        value: UnifiedSemanticValue::Identifier("fn_b".to_string()),
                    },
                ]),
            )],
        );

        let compiled = compile_semantic_runtime_annotations(&annotations)
            .expect("compiled semantic runtime annotations should succeed");
        let mut state = SemanticRuntimeState::new();

        let applied = compiled.apply_to_rule(&mut state, "package_declaration");

        assert_eq!(applied, 2);
        assert_eq!(state.current_scope().kind, SemanticScopeKind::Package);
        assert_eq!(state.facts().len(), 1);
        assert_eq!(state.facts()[0].kind, "typedef");
        assert_eq!(
            state.facts()[0].name,
            SemanticRuntimeValue::Identifier("type_a".to_string())
        );
    }

    #[test]
    fn compiled_annotations_missing_rule_is_a_noop() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "package_declaration".to_string(),
            vec![structured_named(
                "emit_fact",
                "{ kind: typedef, name: type_a }",
                UnifiedSemanticValue::Object(vec![
                    crate::ast_pipeline::UnifiedSemanticProperty {
                        key: "kind".to_string(),
                        value: UnifiedSemanticValue::Identifier("typedef".to_string()),
                    },
                    crate::ast_pipeline::UnifiedSemanticProperty {
                        key: "name".to_string(),
                        value: UnifiedSemanticValue::Identifier("type_a".to_string()),
                    },
                ]),
            )],
        );

        let compiled = compile_semantic_runtime_annotations(&annotations)
            .expect("compiled semantic runtime annotations should succeed");
        let mut state = SemanticRuntimeState::new();

        let applied = state.apply_compiled_rule(&compiled, "missing_rule");

        assert_eq!(applied, 0);
        assert_eq!(state.scopes().len(), 1);
        assert!(state.facts().is_empty());
    }

    #[test]
    fn transaction_apply_compiled_rule_rolls_back_without_commit() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "package_declaration".to_string(),
            vec![
                structured_named(
                    "open_scope",
                    "{ kind: package, name: speculative_pkg }",
                    UnifiedSemanticValue::Object(vec![
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "kind".to_string(),
                            value: UnifiedSemanticValue::Identifier("package".to_string()),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "name".to_string(),
                            value: UnifiedSemanticValue::Identifier("speculative_pkg".to_string()),
                        },
                    ]),
                ),
                structured_named(
                    "predicate",
                    "{ name: current_scope_is, args: [global] }",
                    UnifiedSemanticValue::Object(vec![
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "name".to_string(),
                            value: UnifiedSemanticValue::Identifier(
                                "current_scope_is".to_string(),
                            ),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "args".to_string(),
                            value: UnifiedSemanticValue::Array(vec![
                                UnifiedSemanticValue::Identifier("global".to_string()),
                            ]),
                        },
                    ]),
                ),
                structured_named(
                    "emit_fact",
                    "{ kind: typedef, name: speculative_type }",
                    UnifiedSemanticValue::Object(vec![
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "kind".to_string(),
                            value: UnifiedSemanticValue::Identifier("typedef".to_string()),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "name".to_string(),
                            value: UnifiedSemanticValue::Identifier("speculative_type".to_string()),
                        },
                    ]),
                ),
            ],
        );

        let compiled = compile_semantic_runtime_annotations(&annotations)
            .expect("compiled semantic runtime annotations should succeed");
        let mut state = SemanticRuntimeState::new();

        {
            let mut transaction = state.transaction();
            let applied = transaction.apply_compiled_rule(&compiled, "package_declaration");
            assert_eq!(applied, 2);
            assert_eq!(transaction.state().scopes().len(), 2);
            assert_eq!(transaction.state().facts().len(), 1);
        }

        assert_eq!(state.scopes().len(), 1);
        assert!(state.facts().is_empty());
        assert_eq!(state.current_scope().kind, SemanticScopeKind::Global);
    }

    #[test]
    fn transaction_for_rule_applies_directives_before_commit() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "package_declaration".to_string(),
            vec![
                structured_named(
                    "open_scope",
                    "{ kind: package, name: committed_pkg }",
                    UnifiedSemanticValue::Object(vec![
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "kind".to_string(),
                            value: UnifiedSemanticValue::Identifier("package".to_string()),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "name".to_string(),
                            value: UnifiedSemanticValue::Identifier("committed_pkg".to_string()),
                        },
                    ]),
                ),
                structured_named(
                    "emit_fact",
                    "{ kind: typedef, name: committed_type }",
                    UnifiedSemanticValue::Object(vec![
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "kind".to_string(),
                            value: UnifiedSemanticValue::Identifier("typedef".to_string()),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "name".to_string(),
                            value: UnifiedSemanticValue::Identifier("committed_type".to_string()),
                        },
                    ]),
                ),
            ],
        );

        let compiled = compile_semantic_runtime_annotations(&annotations)
            .expect("compiled semantic runtime annotations should succeed");
        let mut state = SemanticRuntimeState::new();

        {
            let (transaction, applied) =
                state.transaction_for_rule(&compiled, "package_declaration");
            assert_eq!(applied, 2);
            assert_eq!(transaction.state().scopes().len(), 2);
            assert_eq!(transaction.state().facts().len(), 1);
            assert!(transaction.commit());
        }

        assert_eq!(state.current_scope().kind, SemanticScopeKind::Package);
        assert_eq!(state.facts().len(), 1);
        assert_eq!(
            state.facts()[0].name,
            SemanticRuntimeValue::Identifier("committed_type".to_string())
        );
    }

    #[test]
    fn transaction_for_rule_missing_rule_is_still_transaction_safe() {
        let compiled = CompiledSemanticRuntimeAnnotations::default();
        let mut state = SemanticRuntimeState::new();

        {
            let (transaction, applied) = state.transaction_for_rule(&compiled, "missing_rule");
            assert_eq!(applied, 0);
            assert_eq!(transaction.state().scopes().len(), 1);
            assert!(transaction.state().facts().is_empty());
        }

        assert_eq!(state.scopes().len(), 1);
        assert!(state.facts().is_empty());
    }
}
