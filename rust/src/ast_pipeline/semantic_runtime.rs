use super::{SemanticAnnotation, UnifiedSemanticAST, UnifiedSemanticProperty, UnifiedSemanticValue};

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
}

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticRuntimeDirective {
    OpenScope(SemanticScopeSpec),
    CloseScope(SemanticCloseScopeSpec),
    EmitFact(SemanticFactSpec),
    Predicate(SemanticPredicateSpec),
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

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticRuntimeState {
    scopes: Vec<SemanticScopeFrame>,
    facts: Vec<SemanticFactRecord>,
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

fn parse_open_scope(ast: &UnifiedSemanticAST) -> Result<SemanticRuntimeDirective, String> {
    let payload = ast
        .structured_value()
        .ok_or_else(|| "Directive '@open_scope' expects a structured object payload.".to_string())?;
    let properties = object_properties(payload)
        .ok_or_else(|| "Directive '@open_scope' expects an object payload.".to_string())?;
    let kind = property(properties, "kind")
        .ok_or_else(|| "Directive '@open_scope' requires a 'kind' field.".to_string())
        .and_then(|value| {
            SemanticScopeKind::parse(value)
                .ok_or_else(|| "Directive '@open_scope.kind' must be a known scope kind.".to_string())
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
            SemanticScopeKind::parse(value)
                .ok_or_else(|| "Directive '@close_scope.kind' must be a known scope kind.".to_string())
        })
        .transpose()?;
    let name = property(properties, "name")
        .map(|value| {
            SemanticRuntimeValue::from_semantic_value(value).ok_or_else(|| {
                "Directive '@close_scope.name' must be a scalar or rule reference.".to_string()
            })
        })
        .transpose()?;

    Ok(SemanticRuntimeDirective::CloseScope(SemanticCloseScopeSpec {
        kind,
        name,
    }))
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
                        return Err(
                            "Directive '@predicate.args' must be an array when present.".to_string(),
                        );
                    }
                    None => Vec::new(),
                };
                return Ok(SemanticRuntimeDirective::Predicate(SemanticPredicateSpec {
                    name,
                    args,
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
        SemanticPredicateSpec, SemanticRuntimeDirective, SemanticRuntimeState, SemanticRuntimeValue,
        SemanticScopeKind, parse_semantic_runtime_directive,
    };
    use crate::ast_pipeline::{SemanticAnnotation, UnifiedSemanticAST, UnifiedSemanticValue};

    fn structured_named(name: &str, canonical: &str, value: UnifiedSemanticValue) -> SemanticAnnotation {
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

        let parsed =
            parse_semantic_runtime_directive(&predicate).expect("predicate should parse");
        assert_eq!(
            parsed,
            Some(SemanticRuntimeDirective::Predicate(SemanticPredicateSpec {
                name: "is_block_declaration_start".to_string(),
                args: vec![
                    UnifiedSemanticValue::RuleReference("$1".to_string()),
                    UnifiedSemanticValue::Identifier("lhs".to_string()),
                ],
            }))
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
}
