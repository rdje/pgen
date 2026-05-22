use super::{
    Annotations, ParseContent, SemanticAnnotation, UnifiedSemanticAST, UnifiedSemanticProperty,
    UnifiedSemanticValue,
};
use super::predicate_expr::{
    PredicateDef, PredicateExpr, PredicateValue, PrimitiveCall, parse_predicate_expression,
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

    /// `SV-EXH-PROOF.3.3.4.b.5.1.2`: canonical lowercase label for this scope
    /// kind. Used by `@fact_kind`'s V-DECL-6 check (does this kind's
    /// `scope_kind` field reference a known scope label?).
    pub fn label(&self) -> &str {
        match self {
            Self::Global => "global",
            Self::File => "file",
            Self::Package => "package",
            Self::Class => "class",
            Self::Interface => "interface",
            Self::Type => "type",
            Self::Function => "function",
            Self::Task => "task",
            Self::Block => "block",
            Self::Custom(s) => s.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

    /// `SV-EXH-PROOF.3.3.4.a` MVP-0: borrow the underlying text for the
    /// text-bearing variants (`String` / `Identifier` / `RuleReference` /
    /// `Number`). Returns `None` for `Boolean` / `Null` (no canonical text
    /// suitable for use as an artifact name / file-system path component).
    ///
    /// Used by the library import/export helpers to convert a resolved
    /// `name_from` value into a string suitable for `library::artifact_path`.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::String(text)
            | Self::Identifier(text)
            | Self::RuleReference(text)
            | Self::Number(text) => Some(text.as_str()),
            Self::Boolean(_) | Self::Null => None,
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

/// Spec for the parser-agnostic `@export_to_library` directive
/// (`SV-EXH-PROOF.3.3.4.a` MVP-0; see
/// `docs/design/SV-EXH-PROOF-3-3-4-a-MVP-0-library-and-artifact.md`).
///
/// On rule **successful commit**, the generator-emitted
/// `with_semantic_runtime_rule_transaction` (the IIFE-wrapped version after
/// `.3.3.3`) snapshots the rule's emitted facts and writes them as a JSON
/// artifact under the configured library-out directory. `kind` is a static
/// classifier (e.g. `package` / `module` / `entity`); `name` is resolved
/// against the rule's parse content (typically a field reference like `$body`).
///
/// This directive is engine-agnostic: it expresses "this rule's commit boundary
/// defines a library-exportable scope-creating entity"; the engine handles I/O.
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticLibraryExportSpec {
    pub kind: String,
    pub name: SemanticRuntimeValue,
}

/// Spec for the parser-agnostic `@import_from_library` directive
/// (`SV-EXH-PROOF.3.3.4.a` MVP-0).
///
/// On rule **entry** (before pre-predicates fire), the generator-emitted
/// `with_semantic_runtime_rule_transaction` reads the artifact for
/// `<kind>/<name>` from the configured library-in directory and merges its
/// facts into the current `SemanticRuntimeState`, making them visible to any
/// subsequent `@predicate has_fact(...)` check in the same parse.
///
/// A missing artifact is an `Err`; the `.3.3.3` IIFE ensures the rule's
/// semantic-state restore fires cleanly on that error path.
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticLibraryImportSpec {
    pub kind: String,
    pub name: SemanticRuntimeValue,
}

/// `SV-EXH-PROOF.3.3.4.b.5.1.2`: payload of a `@fact_kind:` declaration.
///
/// Surface syntax (per `PGEN_SEMANTIC_STORE_SCHEMA_LANGUAGE_SPEC.md` §4):
///
/// ```text
/// @fact_kind: {
///   name:           variable_binding,
///   attributes:     [name, type_kind, type_ref, declared_in],
///   required:       [name, type_kind],
///   indexes:        [(scope, name), (scope, type_kind), (name)],
///   scope_kind:     enclosing_block,
///   exportable:     true,
///   artefact_kind:  bindings,
///   description:    "A bound identifier with its declared type."
/// }
/// ```
///
/// Mandatory fields:    `name`, `attributes`.
/// Optional fields:     `required` (default `[]`), `indexes` (default
/// `[(scope, kind, name)]`), `scope_kind` (default `"current"`),
/// `exportable` (default `false`), `artefact_kind` (default = `name`),
/// `description` (default empty).
///
/// Validation rules (V-DECL-1..7 from the schema spec) — codegen-time errors:
/// 1. `name` unique across all `@fact_kind` declarations in this grammar.
///    (Cross-declaration check; performed in
///    `compile_semantic_runtime_annotations` once all parses are in.)
/// 2. `attributes` non-empty.
/// 3. Every name in `required` appears in `attributes`.
/// 4. Every name in every index tuple appears in `attributes` ∪
///    `{"scope", "kind"}` (the implicit attributes).
/// 5. Each index tuple is non-empty and contains no duplicate attribute names.
/// 6. `scope_kind` (if specified) matches a label used in some `@open_scope`
///    directive **or** an engine-reserved label (`global` / `current` /
///    `enclosing_block` / `enclosing_function` / `enclosing_class` /
///    `enclosing_package` / `enclosing_file`). Cross-declaration check; warn-only
///    at compile time (lenient — grammars in motion may add `@open_scope`
///    rules incrementally).
/// 7. `name` and `artefact_kind` (if specified) are valid path components:
///    no `/`, no `..`, no leading dot.
///
/// Per-instance validation (V-DECL-2..5, V-DECL-7) lives in `validate_local`;
/// V-DECL-1 / V-DECL-6 are deferred to `compile_semantic_runtime_annotations`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FactKindDecl {
    pub name: String,
    pub attributes: Vec<String>,
    pub required: Vec<String>,
    /// Composite-key index specifications. Each inner `Vec<String>` is a
    /// tuple of attribute names that together index a fact. Default
    /// `[(scope, kind, name)]` is conventional; grammar authors may add or
    /// override (e.g. `(scope, name)`, `(container)`, `(name)`).
    pub indexes: Vec<Vec<String>>,
    pub scope_kind: Option<String>,
    pub exportable: bool,
    pub artefact_kind: Option<String>,
    pub description: Option<String>,
}

impl FactKindDecl {
    /// Per-declaration validation (V-DECL-2, V-DECL-3, V-DECL-4, V-DECL-5,
    /// V-DECL-7). Returns Err with a precise message if any rule is violated.
    /// Cross-declaration rules (V-DECL-1 uniqueness, V-DECL-6 scope_kind
    /// label) are checked in the compile pass.
    pub fn validate_local(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err(format!(
                "@fact_kind: 'name' field is required (V-DECL-2 surface check)."
            ));
        }
        // V-DECL-2: attributes non-empty.
        if self.attributes.is_empty() {
            return Err(format!(
                "@fact_kind '{}': V-DECL-2 violation — 'attributes' must be non-empty.",
                self.name
            ));
        }
        // V-DECL-3: every name in `required` appears in `attributes`.
        for attr in &self.required {
            if !self.attributes.iter().any(|a| a == attr) {
                return Err(format!(
                    "@fact_kind '{}': V-DECL-3 violation — 'required' lists '{}' but it is not in 'attributes' (declared attributes: {:?}).",
                    self.name, attr, self.attributes
                ));
            }
        }
        // V-DECL-4 + V-DECL-5: per-index tuple validation.
        for (i, index_tuple) in self.indexes.iter().enumerate() {
            // V-DECL-5: non-empty.
            if index_tuple.is_empty() {
                return Err(format!(
                    "@fact_kind '{}': V-DECL-5 violation — index tuple #{} is empty.",
                    self.name, i
                ));
            }
            // V-DECL-5: no duplicate attribute names within a tuple.
            for j in 0..index_tuple.len() {
                for k in (j + 1)..index_tuple.len() {
                    if index_tuple[j] == index_tuple[k] {
                        return Err(format!(
                            "@fact_kind '{}': V-DECL-5 violation — index tuple #{} repeats attribute '{}'.",
                            self.name, i, index_tuple[j]
                        ));
                    }
                }
            }
            // V-DECL-4: every name in tuple ∈ attributes ∪ {scope, kind}.
            for attr in index_tuple {
                let is_implicit = attr == "scope" || attr == "kind";
                let is_declared = self.attributes.iter().any(|a| a == attr);
                if !is_implicit && !is_declared {
                    return Err(format!(
                        "@fact_kind '{}': V-DECL-4 violation — index tuple #{} references '{}' which is neither a declared attribute nor one of the implicit attributes 'scope'/'kind'.",
                        self.name, i, attr
                    ));
                }
            }
        }
        // V-DECL-7: `name` and `artefact_kind` are valid path components.
        validate_path_component(&self.name, "name", &self.name)?;
        if let Some(artefact_kind) = self.artefact_kind.as_deref() {
            validate_path_component(artefact_kind, "artefact_kind", &self.name)?;
        }
        Ok(())
    }

    /// `artefact_kind` resolved against the default (= `name`).
    pub fn resolved_artefact_kind(&self) -> &str {
        self.artefact_kind.as_deref().unwrap_or(&self.name)
    }
}

/// V-DECL-7 helper: a valid path component contains no `/`, no `..`, no
/// leading dot, no embedded NUL.
fn validate_path_component(value: &str, field_name: &str, kind_name: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!(
            "@fact_kind '{}': V-DECL-7 violation — '{}' is empty.",
            kind_name, field_name
        ));
    }
    if value.contains('/') || value.contains('\\') {
        return Err(format!(
            "@fact_kind '{}': V-DECL-7 violation — '{}' = '{}' contains a path separator.",
            kind_name, field_name, value
        ));
    }
    if value == ".." || value == "." {
        return Err(format!(
            "@fact_kind '{}': V-DECL-7 violation — '{}' = '{}' is not a valid path component.",
            kind_name, field_name, value
        ));
    }
    if value.starts_with('.') {
        return Err(format!(
            "@fact_kind '{}': V-DECL-7 violation — '{}' = '{}' begins with a dot (would become a hidden file).",
            kind_name, field_name, value
        ));
    }
    if value.contains('\0') {
        return Err(format!(
            "@fact_kind '{}': V-DECL-7 violation — '{}' contains an embedded NUL byte.",
            kind_name, field_name
        ));
    }
    Ok(())
}

/// V-DECL-6: scope_kind labels reserved by the engine. Grammar-level `@open_scope`
/// declarations may add their own labels; the union is checked at compile time.
const ENGINE_RESERVED_SCOPE_KINDS: &[&str] = &[
    "global",
    "current",
    "enclosing_block",
    "enclosing_function",
    "enclosing_class",
    "enclosing_package",
    "enclosing_file",
];

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
    /// `SV-EXH-PROOF.3.3.4.a` MVP-0: parser-agnostic library export directive.
    /// Fires on rule **successful commit** (its own phase, not `is_effect`);
    /// see `is_library_export`.
    ExportToLibrary(SemanticLibraryExportSpec),
    /// `SV-EXH-PROOF.3.3.4.a` MVP-0: parser-agnostic library import directive.
    /// Fires on rule **entry**, before pre-predicates (its own phase, not
    /// `is_pre_predicate`); see `is_library_import`.
    ImportFromLibrary(SemanticLibraryImportSpec),
    /// `SV-EXH-PROOF.3.3.4.b.5.1.2`: grammar-level fact-kind declaration
    /// (Stage 1 of the lifecycle protocol per `CONTEXT_AWARE_PARSING_DESIGN.md`
    /// §4). Declares a new semantic-fact type with its attributes, required-set,
    /// secondary indexes, default scope, and library-export eligibility. The
    /// directive is **compile-time only**: at runtime it's a no-op
    /// (`apply_directive` short-circuits). At codegen time the compiled
    /// annotations carry a registry of all declared fact-kinds for downstream
    /// consumers (library export, schema-aware diagnostics, future scope-tree
    /// emission rules).
    DeclareFactKind(FactKindDecl),
    /// `SV-EXH-PROOF.3.3.4.b.5.1.5`: grammar-level composed-predicate
    /// definition (Stage 3 of the lifecycle protocol — the `@predicate_def:`
    /// block). Defines a named boolean predicate whose body is an expression
    /// over the built-in primitives. Like `DeclareFactKind` this directive
    /// is **compile-time only**: a no-op at `apply_directive`; the compiled
    /// annotations carry a registry of all predicate definitions so a
    /// `@predicate <user-name>` call site can resolve + evaluate the
    /// composed body.
    DefinePredicate(PredicateDef),
}

impl SemanticRuntimeDirective {
    pub fn predicate_phase(&self) -> Option<SemanticPredicatePhase> {
        match self {
            Self::Predicate(spec) => Some(spec.phase),
            Self::OpenScope(_)
            | Self::CloseScope(_)
            | Self::EmitFact(_)
            | Self::ExportToLibrary(_)
            | Self::ImportFromLibrary(_)
            | Self::DeclareFactKind(_)
            | Self::DefinePredicate(_) => None,
        }
    }

    pub fn predicate_view(&self) -> Option<SemanticPredicateContentView> {
        match self {
            Self::Predicate(spec) => Some(spec.view),
            Self::OpenScope(_)
            | Self::CloseScope(_)
            | Self::EmitFact(_)
            | Self::ExportToLibrary(_)
            | Self::ImportFromLibrary(_)
            | Self::DeclareFactKind(_)
            | Self::DefinePredicate(_) => None,
        }
    }

    /// `SV-EXH-PROOF.3.3.4.b.5.1.2`: true for `DeclareFactKind` only.
    /// Declarations are compile-time only; the generator collects them into
    /// the schema registry. At runtime `apply_directive` short-circuits.
    pub fn is_fact_kind_declaration(&self) -> bool {
        matches!(self, Self::DeclareFactKind(_))
    }

    pub fn is_pre_predicate(&self) -> bool {
        self.predicate_phase() == Some(SemanticPredicatePhase::Pre)
    }

    pub fn is_post_predicate(&self) -> bool {
        self.predicate_phase() == Some(SemanticPredicatePhase::Post)
    }

    pub fn is_branch_predicate(&self) -> bool {
        self.predicate_phase() == Some(SemanticPredicatePhase::Branch)
    }

    pub fn is_effect(&self) -> bool {
        matches!(
            self,
            Self::OpenScope(_) | Self::CloseScope(_) | Self::EmitFact(_)
        )
    }

    /// `SV-EXH-PROOF.3.3.4.a` MVP-0: true for `ImportFromLibrary` only.
    /// Library imports are a distinct phase from `is_pre_predicate` so the
    /// generator can guarantee they fire **before** pre-predicates evaluate
    /// (so a pre-predicate `has_fact(type_name, X)` can see the imported
    /// facts in the same rule's transaction).
    pub fn is_library_import(&self) -> bool {
        matches!(self, Self::ImportFromLibrary(_))
    }

    /// `SV-EXH-PROOF.3.3.4.a` MVP-0: true for `ExportToLibrary` only.
    /// Library exports are a distinct phase from `is_effect` so the generator
    /// can guarantee they fire **after** the rule's body emit_facts have all
    /// been applied (the export captures the rule's emitted facts delta).
    pub fn is_library_export(&self) -> bool {
        matches!(self, Self::ExportToLibrary(_))
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CompiledSemanticRuntimeAnnotations {
    directives_by_rule: HashMap<String, Vec<SemanticRuntimeDirective>>,
    branch_directives_by_rule: HashMap<String, Vec<Vec<SemanticRuntimeDirective>>>,
    /// `SV-EXH-PROOF.3.3.4.b.5.1.2`: registry of `@fact_kind:` declarations
    /// collected at compile time. Keys are kind names (case-sensitive, as
    /// declared); values are the parsed + locally-validated `FactKindDecl`.
    /// Cross-declaration uniqueness (V-DECL-1) is enforced when this map is
    /// populated. Downstream consumers (library export, scope-tree emitters,
    /// schema-aware diagnostics) look up declarations by name.
    fact_kinds: HashMap<String, FactKindDecl>,
    /// `SV-EXH-PROOF.3.3.4.b.5.1.5`: registry of `@predicate_def:` composed
    /// predicate definitions collected at compile time. Keys are predicate
    /// names; values are the parsed + locally-validated `PredicateDef`.
    /// V-QDEF-1 (uniqueness + non-shadowing of built-ins) is enforced when
    /// this map is populated. A `@predicate <user-name>` call site resolves
    /// against this registry.
    predicate_defs: HashMap<String, PredicateDef>,
}

impl CompiledSemanticRuntimeAnnotations {
    pub fn compile(annotations: &Annotations) -> Result<Self, String> {
        compile_semantic_runtime_annotations(annotations)
    }

    pub fn from_rule_directives(
        directives_by_rule: HashMap<String, Vec<SemanticRuntimeDirective>>,
    ) -> Self {
        Self {
            directives_by_rule,
            branch_directives_by_rule: HashMap::new(),
            fact_kinds: HashMap::new(),
            predicate_defs: HashMap::new(),
        }
    }

    pub fn from_parts(
        directives_by_rule: HashMap<String, Vec<SemanticRuntimeDirective>>,
        branch_directives_by_rule: HashMap<String, Vec<Vec<SemanticRuntimeDirective>>>,
    ) -> Self {
        Self {
            directives_by_rule,
            branch_directives_by_rule,
            fact_kinds: HashMap::new(),
            predicate_defs: HashMap::new(),
        }
    }

    /// `SV-EXH-PROOF.3.3.4.b.5.1.5`: accessor for the predicate-def registry.
    pub fn predicate_def(&self, name: &str) -> Option<&PredicateDef> {
        self.predicate_defs.get(name)
    }

    /// `SV-EXH-PROOF.3.3.4.b.5.1.5`: iterate every composed predicate.
    pub fn predicate_defs(&self) -> impl Iterator<Item = (&str, &PredicateDef)> {
        self.predicate_defs
            .iter()
            .map(|(name, decl)| (name.as_str(), decl))
    }

    /// `SV-EXH-PROOF.3.3.4.b.5.1.5`: number of composed predicates.
    pub fn predicate_defs_len(&self) -> usize {
        self.predicate_defs.len()
    }

    /// `SV-EXH-PROOF.3.3.4.b.5.1.5.c`: clone the predicate-def registry for
    /// seeding into a `SemanticRuntimeState` via `set_predicate_defs`. Called
    /// once at generated-parser construction (and after each `parse()`
    /// reset). The registry is bounded by the grammar's `@predicate_def:`
    /// count — tiny — so the clone is cheap.
    pub fn clone_predicate_defs(&self) -> HashMap<String, PredicateDef> {
        self.predicate_defs.clone()
    }

    /// `SV-EXH-PROOF.3.3.4.b.5.1.2`: accessor for the fact-kind registry.
    /// Returns the declared `FactKindDecl` for `name`, or `None` if no
    /// `@fact_kind:` block declared this name.
    pub fn fact_kind(&self, name: &str) -> Option<&FactKindDecl> {
        self.fact_kinds.get(name)
    }

    /// `SV-EXH-PROOF.3.3.4.b.5.1.2`: iterate every declared fact-kind.
    /// Order is unspecified (the registry is a HashMap); consumers that need
    /// deterministic ordering should sort by `decl.name`.
    pub fn fact_kinds(&self) -> impl Iterator<Item = (&str, &FactKindDecl)> {
        self.fact_kinds.iter().map(|(name, decl)| (name.as_str(), decl))
    }

    /// `SV-EXH-PROOF.3.3.4.b.5.1.2`: number of declared fact-kinds.
    pub fn fact_kinds_len(&self) -> usize {
        self.fact_kinds.len()
    }

    pub fn is_empty(&self) -> bool {
        self.directives_by_rule.is_empty() && self.branch_directives_by_rule.is_empty()
    }

    pub fn len(&self) -> usize {
        let mut rule_names = self.directives_by_rule.keys().collect::<Vec<_>>();
        for rule_name in self.branch_directives_by_rule.keys() {
            if !rule_names.iter().any(|existing| *existing == rule_name) {
                rule_names.push(rule_name);
            }
        }
        rule_names.len()
    }

    pub fn has_rule(&self, rule_name: &str) -> bool {
        self.directives_by_rule.contains_key(rule_name)
            || self.branch_directives_by_rule.contains_key(rule_name)
    }

    pub fn directives_for_rule(&self, rule_name: &str) -> &[SemanticRuntimeDirective] {
        self.directives_by_rule
            .get(rule_name)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn branch_directives_for_rule(&self, rule_name: &str) -> &[Vec<SemanticRuntimeDirective>] {
        self.branch_directives_by_rule
            .get(rule_name)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn branch_directives_for_rule_branch(
        &self,
        rule_name: &str,
        branch_index: usize,
    ) -> &[SemanticRuntimeDirective] {
        self.branch_directives_for_rule(rule_name)
            .get(branch_index)
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

    pub fn post_predicates_for_rule<'a>(
        &'a self,
        rule_name: &'a str,
    ) -> impl Iterator<Item = &'a SemanticRuntimeDirective> + 'a {
        self.directives_for_rule(rule_name)
            .iter()
            .filter(|directive| directive.is_post_predicate())
    }

    /// `SV-EXH-PROOF.3.3.4.a` MVP-0: library imports for the rule (fired at
    /// rule entry, before pre-predicates, by the generator-emitted
    /// `with_semantic_runtime_rule_transaction`).
    pub fn library_imports_for_rule<'a>(
        &'a self,
        rule_name: &'a str,
    ) -> impl Iterator<Item = &'a SemanticRuntimeDirective> + 'a {
        self.directives_for_rule(rule_name)
            .iter()
            .filter(|directive| directive.is_library_import())
    }

    /// `SV-EXH-PROOF.3.3.4.a` MVP-0: library exports for the rule (fired at
    /// rule successful commit, after effect directives, by the generator-emitted
    /// `with_semantic_runtime_rule_transaction`).
    pub fn library_exports_for_rule<'a>(
        &'a self,
        rule_name: &'a str,
    ) -> impl Iterator<Item = &'a SemanticRuntimeDirective> + 'a {
        self.directives_for_rule(rule_name)
            .iter()
            .filter(|directive| directive.is_library_export())
    }

    pub fn branch_predicates_for_rule<'a>(
        &'a self,
        rule_name: &'a str,
    ) -> impl Iterator<Item = &'a SemanticRuntimeDirective> + 'a {
        self.directives_for_rule(rule_name)
            .iter()
            .chain(
                self.branch_directives_for_rule(rule_name)
                    .iter()
                    .flat_map(|directives| directives.iter()),
            )
            .filter(|directive| directive.is_branch_predicate())
    }

    pub fn branch_predicates_for_rule_branch<'a>(
        &'a self,
        rule_name: &'a str,
        branch_index: usize,
    ) -> impl Iterator<Item = &'a SemanticRuntimeDirective> + 'a {
        self.branch_directives_for_rule_branch(rule_name, branch_index)
            .iter()
            .filter(|directive| directive.is_branch_predicate())
    }

    pub fn has_post_predicates_for_rule(&self, rule_name: &str) -> bool {
        self.post_predicates_for_rule(rule_name).next().is_some()
    }

    pub fn needs_raw_post_capture_for_rule(&self, rule_name: &str) -> bool {
        self.post_predicates_for_rule(rule_name)
            .any(|directive| directive.predicate_view() == Some(SemanticPredicateContentView::Raw))
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

    pub fn branch_iter(
        &self,
    ) -> impl Iterator<Item = (&str, &[Vec<SemanticRuntimeDirective>])> + ExactSizeIterator + '_
    {
        self.branch_directives_by_rule
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
    /// Legacy field: the depth (= position in the active chain) of the scope
    /// the fact was emitted into. Maintained alongside `scope_id` for
    /// backward compatibility with `.3.3.4.a` library export/import paths
    /// and the `.b.5.1.1` `(scope_depth, name)` index keys.
    pub scope_depth: usize,
    /// `SV-EXH-PROOF.3.3.4.b.5.1.3`: precise scope-arena identifier of the
    /// scope this fact belongs to. Set on emit/import; preserved across
    /// rollback (a rolled-back emit removes the fact entirely, so its
    /// scope_id never appears in the live store after rollback). Used by
    /// future tree-walking queries (resolve_path in .b.5.1.4 onwards).
    /// `ScopeId::ROOT` (0) on facts emitted while only the global scope
    /// is active.
    pub scope_id: ScopeId,
    pub attributes: Vec<UnifiedSemanticProperty>,
}

/// `SV-EXH-PROOF.3.3.4.b.5.1.3`: dense identifier for a scope node in the
/// `scope_arena`. `ScopeId(0)` is reserved for the global (root) scope —
/// `SemanticRuntimeState::new` always allocates it as arena entry 0 so
/// `current_scope_id()` is total.
///
/// Newtype rather than a bare `usize` so accidental arithmetic doesn't
/// compile and the type signature of every tree-walking API is unambiguous.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ScopeId(pub u32);

impl ScopeId {
    /// The implicit global scope, allocated as arena entry 0 at construction.
    pub const ROOT: ScopeId = ScopeId(0);

    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// `SV-EXH-PROOF.3.3.4.b.5.1.4`: result of a `resolve_path` walk through
/// the scope tree.
///
/// `Resolved` is returned when every segment of the dotted path successfully
/// looked up a fact. `Unresolved` is returned when some prefix resolved but
/// a later segment failed — the `resolved_prefix` field captures how far
/// the walk got, and `last_kind` records the kind of the last successfully
/// resolved fact (useful for diagnostics: "the path resolves up to `a.b`
/// which is a `variable_binding`, but `.c` could not be resolved against
/// the referenced type's scope").
#[derive(Debug, Clone, PartialEq)]
pub enum ResolveResult {
    Resolved {
        kind: String,
        name: SemanticRuntimeValue,
        scope_id: ScopeId,
        attributes: Vec<UnifiedSemanticProperty>,
    },
    Unresolved {
        /// Segments that were successfully resolved before the walk failed.
        /// Empty if even the first segment failed.
        resolved_prefix: Vec<String>,
        /// Kind of the deepest successfully-resolved fact, if any.
        last_kind: Option<String>,
    },
}

impl ResolveResult {
    pub fn is_resolved(&self) -> bool {
        matches!(self, Self::Resolved { .. })
    }

    /// Look up an attribute by key on the resolved fact (case-insensitive,
    /// matching the engine's predicate convention). Returns `None` if the
    /// path is `Unresolved` or the attribute is absent. Used by composed
    /// predicates that drill into the resolved fact (e.g.,
    /// `resolve_path($p).attribute("type_kind")`).
    pub fn attribute(&self, key: &str) -> Option<&UnifiedSemanticValue> {
        match self {
            Self::Resolved { attributes, .. } => attributes
                .iter()
                .find(|p| p.key.eq_ignore_ascii_case(key))
                .map(|p| &p.value),
            Self::Unresolved { .. } => None,
        }
    }
}

/// `SV-EXH-PROOF.3.3.4.b.5.1.3`: one node in the scope tree.
///
/// The tree is the canonical structure for the lifecycle protocol's Stage 4
/// (SCOPE) and its consumers (resolve_path in .b.5.1.4, library export in
/// .b.5.1.3 onwards). Each node knows its parent; children are derived on
/// demand by scanning the arena. The arena is append-only during a
/// transaction; rollback truncates back to the checkpoint length.
///
/// Closed scopes (`closed == true`) stay in the arena — they are queryable
/// for archived-scope lookups (e.g., "list all variable bindings declared
/// inside class Foo, even after the class scope has closed"). The runtime's
/// `active_chain` tracks only currently-open scopes; the arena tracks
/// everything ever opened (and not rolled back).
#[derive(Debug, Clone, PartialEq)]
pub struct ScopeNode {
    pub id: ScopeId,
    /// Parent scope id. `None` only for the root (`ScopeId::ROOT`).
    pub parent: Option<ScopeId>,
    pub kind: SemanticScopeKind,
    pub name: Option<SemanticRuntimeValue>,
    /// True after `close_scope` matched and popped this node from the
    /// `active_chain`. The node remains in the arena.
    pub closed: bool,
    /// Depth of this scope in the active chain at the moment it was
    /// opened (root = 0). Recorded for backward-compat with
    /// `SemanticFactRecord.scope_depth`.
    pub depth_when_opened: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticRuntimeCheckpoint {
    scope_len: usize,
    fact_len: usize,
    /// `SV-EXH-PROOF.3.3.4.b.5.1.3`: arena length at checkpoint time. On
    /// rollback the arena truncates back to this point — nodes allocated
    /// during the speculative tx vanish.
    scope_arena_len: usize,
    /// `SV-EXH-PROOF.3.3.4.b.5.1.3`: snapshot of the active-scope chain at
    /// checkpoint time. On rollback the chain is restored to exactly this
    /// sequence — every scope that the speculative tx closed is re-opened
    /// (its `closed` flag is reset). The chain is at most `scope_arena_len`
    /// long. (Removing `Copy` from this struct to accommodate the Vec; all
    /// uses of `SemanticRuntimeCheckpoint` are owned values that can clone
    /// cheaply since the snapshot is bounded by parser nesting depth.)
    active_chain_snapshot: Vec<ScopeId>,
}

impl SemanticRuntimeCheckpoint {
    /// `SV-EXH-PROOF.3.3.4.a` MVP-0: number of facts in the state at checkpoint
    /// time. Exposed so `@export_to_library` can snapshot the delta — the facts
    /// emitted by the current rule's body — via
    /// `&state.facts()[checkpoint.fact_len()..]`.
    pub fn fact_len(&self) -> usize {
        self.fact_len
    }

    /// `SV-EXH-PROOF.3.3.4.a` MVP-0 companion accessor for scope depth at
    /// checkpoint. Not used by the import/export path today but mirrors
    /// `fact_len` for completeness and future use.
    pub fn scope_len(&self) -> usize {
        self.scope_len
    }

    /// `SV-EXH-PROOF.3.3.4.b.5.1.3`: arena length at checkpoint time.
    pub fn scope_arena_len(&self) -> usize {
        self.scope_arena_len
    }
}

/// `.3.3.4.b.5.1.1`: per-kind secondary index for O(1)-avg fact queries.
///
/// Before this slice, every predicate on `SemanticRuntimeState` walked the
/// full `facts: Vec<SemanticFactRecord>` with a linear scan
/// (`self.facts.iter().filter(...)`). At uvm-scale fact populations (tens of
/// thousands of variable bindings, class members, typedefs, …) those linear
/// scans become the parser's hottest cost — that's precisely the
/// "bottleneck" the user mandate rules out
/// (`CONTEXT_AWARE_PARSING_DESIGN.md` §3.5 / §11; `PGEN_SEMANTIC_STORE_PERFORMANCE_CONTRACT.md` §3.3).
///
/// This index is maintained ALONGSIDE the master `facts` Vec (which remains
/// the source of truth in insertion order — required by the `.3.3.4.a`
/// library-export path which slices `&state.facts()[checkpoint.fact_len()..]`
/// for the per-rule delta). The Vec stays; the index makes lookups cheap.
///
/// Shape (per fact-kind, kind comparison case-insensitive — kind keys are
/// stored lowercased to match the existing `eq_ignore_ascii_case` predicate
/// semantics):
/// - `by_scope_and_name: HashMap<(usize, SemanticRuntimeValue), Vec<usize>>`
///   maps `(scope_depth, name)` to the positions in `facts` Vec. A Vec of
///   positions handles the duplicate-emit case (allowed today; semantics
///   for duplicate-in-scope are still `[TBC]` per the API contract §7).
/// - `total_count: usize` — fast `fact_count_at_least` (counts ALL facts of
///   this kind, regardless of scope; the legacy semantic).
///
/// Rollback (`rollback_to`) extends to the index: when the master Vec
/// truncates from `facts.len()` to `fact_len`, every fact at position
/// `>= fact_len` is removed from its index buckets. Cost is O(emitted-in-tx),
/// not O(store), matching the performance contract §3.7.
#[derive(Debug, Clone, Default, PartialEq)]
struct FactIndex {
    by_kind: HashMap<String, FactKindIndex>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct FactKindIndex {
    by_scope_and_name: HashMap<(usize, SemanticRuntimeValue), Vec<usize>>,
    total_count: usize,
}

impl FactIndex {
    /// Insert `position` (the master `facts` Vec index) for the given fact.
    /// Kind is normalised lowercase to match `eq_ignore_ascii_case` query
    /// semantics already in place.
    fn insert(&mut self, kind: &str, scope_depth: usize, name: &SemanticRuntimeValue, position: usize) {
        let kind_index = self.by_kind.entry(kind.to_ascii_lowercase()).or_default();
        kind_index
            .by_scope_and_name
            .entry((scope_depth, name.clone()))
            .or_default()
            .push(position);
        kind_index.total_count += 1;
    }

    /// Remove the index entry for a fact at `position`. Returns true if the
    /// entry was present. Used by `rollback_to` to undo emits.
    fn remove(&mut self, kind: &str, scope_depth: usize, name: &SemanticRuntimeValue, position: usize) -> bool {
        let kind_normalised = kind.to_ascii_lowercase();
        let Some(kind_index) = self.by_kind.get_mut(&kind_normalised) else {
            return false;
        };
        let key = (scope_depth, name.clone());
        let Some(positions) = kind_index.by_scope_and_name.get_mut(&key) else {
            return false;
        };
        let Some(idx) = positions.iter().rposition(|&p| p == position) else {
            return false;
        };
        positions.swap_remove(idx);
        if positions.is_empty() {
            kind_index.by_scope_and_name.remove(&key);
        }
        kind_index.total_count -= 1;
        if kind_index.total_count == 0 && kind_index.by_scope_and_name.is_empty() {
            self.by_kind.remove(&kind_normalised);
        }
        true
    }

    /// O(1)-average existence check across the whole store: is there any fact
    /// of `kind` with this `name` (in any scope_depth)?
    fn any_with_name(&self, kind: &str, name: &SemanticRuntimeValue) -> bool {
        let kind_normalised = kind.to_ascii_lowercase();
        let Some(kind_index) = self.by_kind.get(&kind_normalised) else {
            return false;
        };
        kind_index
            .by_scope_and_name
            .iter()
            .any(|((_, n), positions)| n == name && !positions.is_empty())
    }

    /// O(1) existence check at a specific scope depth.
    fn any_with_name_at_scope(
        &self,
        kind: &str,
        scope_depth: usize,
        name: &SemanticRuntimeValue,
    ) -> bool {
        let kind_normalised = kind.to_ascii_lowercase();
        let Some(kind_index) = self.by_kind.get(&kind_normalised) else {
            return false;
        };
        kind_index
            .by_scope_and_name
            .get(&(scope_depth, name.clone()))
            .is_some_and(|positions| !positions.is_empty())
    }

    /// Enumerate positions in the master Vec for facts matching
    /// `(kind, name)` across ALL scope depths. Used by attribute-existence
    /// and attribute-value-equality predicates which need to look up the
    /// fact's `attributes` payload.
    fn positions_for_name<'a>(
        &'a self,
        kind: &str,
        name: &'a SemanticRuntimeValue,
    ) -> impl Iterator<Item = usize> + 'a {
        let kind_normalised = kind.to_ascii_lowercase();
        self.by_kind
            .get(&kind_normalised)
            .into_iter()
            .flat_map(move |kind_index| {
                kind_index
                    .by_scope_and_name
                    .iter()
                    .filter(move |((_, n), _)| n == name)
                    .flat_map(|(_, positions)| positions.iter().copied())
            })
    }

    /// O(1) total fact count for a given kind.
    fn count_for_kind(&self, kind: &str) -> usize {
        let kind_normalised = kind.to_ascii_lowercase();
        self.by_kind
            .get(&kind_normalised)
            .map_or(0, |k| k.total_count)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SemanticRuntimeState {
    scopes: Vec<SemanticScopeFrame>,
    facts: Vec<SemanticFactRecord>,
    /// `.3.3.4.b.5.1.1`: secondary index for O(1)-avg queries by `(kind, name)`
    /// and `(kind, scope_depth, name)`. Maintained in lockstep with `facts`.
    fact_index: FactIndex,
    /// `.3.3.4.b.5.1.3`: append-only arena of every scope node ever opened
    /// (and not rolled back). Indexed by `ScopeId`. Entry 0 is the global
    /// (root) scope, allocated by `new`. Closed scopes stay in the arena
    /// with `closed: true` for archived queries.
    scope_arena: Vec<ScopeNode>,
    /// `.3.3.4.b.5.1.3`: stack of currently-open scope ids — the active
    /// chain. Innermost is `last()`. `[ScopeId::ROOT]` initially. Maintained
    /// in lockstep with the legacy `scopes` Vec (one entry per active
    /// scope) so existing consumers (predicates that read scope_depth /
    /// current_scope) continue to work unchanged.
    active_chain: Vec<ScopeId>,
    /// `.3.3.4.b.5.1.5.c`: composed-predicate registry, seeded at parser
    /// construction from `CompiledSemanticRuntimeAnnotations.predicate_defs`
    /// via `set_predicate_defs`. `evaluate_predicate` consults this when a
    /// predicate name is not a built-in — that's how a runtime
    /// `@predicate <user-defined-name>` call dispatches to its
    /// `@predicate_def:` body. Empty in a freshly-`new`'d state.
    predicate_defs: HashMap<String, PredicateDef>,
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
        // `.3.3.4.b.5.1.3`: the global (root) scope is arena entry 0. The
        // legacy `scopes` Vec and the new `active_chain` Vec are populated
        // in lockstep — both reference the same global frame.
        let root_node = ScopeNode {
            id: ScopeId::ROOT,
            parent: None,
            kind: SemanticScopeKind::Global,
            name: None,
            closed: false,
            depth_when_opened: 0,
        };
        Self {
            scopes: vec![SemanticScopeFrame {
                kind: SemanticScopeKind::Global,
                name: None,
            }],
            facts: Vec::new(),
            fact_index: FactIndex::default(),
            scope_arena: vec![root_node],
            active_chain: vec![ScopeId::ROOT],
            predicate_defs: HashMap::new(),
        }
    }

    /// `SV-EXH-PROOF.3.3.4.b.5.1.5.c`: seed the composed-predicate registry.
    /// Called once at parser construction (and after each `parse()` reset)
    /// so a runtime `@predicate <user-defined-name>` call can resolve its
    /// `@predicate_def:` body. Built-in predicate names never reach this
    /// registry — they are handled directly by `evaluate_predicate`.
    pub fn set_predicate_defs(&mut self, defs: HashMap<String, PredicateDef>) {
        self.predicate_defs = defs;
    }

    // -------------------------------------------------------------------------
    // `.3.3.4.b.5.1.3` scope-tree accessors
    // -------------------------------------------------------------------------

    /// Read-only access to the scope arena (every scope ever opened, alive
    /// or closed).
    pub fn scope_arena(&self) -> &[ScopeNode] {
        &self.scope_arena
    }

    /// Lookup a scope node by id. Returns `None` if the id is out of bounds
    /// (e.g., a stale id captured before rollback).
    pub fn scope_node(&self, id: ScopeId) -> Option<&ScopeNode> {
        self.scope_arena.get(id.index())
    }

    /// The id of the innermost currently-open scope. Always at least
    /// `ScopeId::ROOT` because the global scope is never closed.
    pub fn current_scope_id(&self) -> ScopeId {
        *self
            .active_chain
            .last()
            .expect("semantic runtime state always maintains at least the root scope")
    }

    /// The active scope chain from outermost (root) to innermost. Used by
    /// queries that need to walk up the visibility stack.
    pub fn active_chain(&self) -> &[ScopeId] {
        &self.active_chain
    }

    /// Iterate the direct children of `id` in the arena. O(arena.len())
    /// scan; future optimisation can maintain a per-node children list if
    /// query patterns demand it.
    pub fn scope_children(&self, id: ScopeId) -> impl Iterator<Item = &ScopeNode> + '_ {
        self.scope_arena
            .iter()
            .filter(move |node| node.parent == Some(id))
    }

    // -------------------------------------------------------------------------
    // `.3.3.4.b.5.1.4` resolve_path — multi-segment dotted lookup through
    // the scope tree.
    // -------------------------------------------------------------------------

    /// `SV-EXH-PROOF.3.3.4.b.5.1.4`: resolve a dotted path like
    /// `seed_map.seed_table.exists` through the scope tree.
    ///
    /// **Algorithm (segment-by-segment walk):**
    ///
    /// - Split `path` on `.`. Empty path or any empty segment → `Unresolved`.
    /// - **Segment 0:** walk the active chain innermost → outermost. For
    ///   each active scope, scan its emitted facts. The first fact whose
    ///   name matches `segments[0]` wins. If no scope contains a match,
    ///   the path is `Unresolved` with empty prefix.
    /// - **Segment N+1:** the previous fact must carry a `type_ref`
    ///   attribute naming a scope. Find any scope whose name matches that
    ///   value (the convention: `type_ref` names a class / module /
    ///   interface / package). Look up `segments[N+1]` in that scope's
    ///   emitted facts. First match wins; absence is `Unresolved` with the
    ///   prefix walked so far.
    /// - When all segments resolve, return `Resolved` with the deepest
    ///   fact's kind / name / scope_id / attributes.
    ///
    /// **Convention:** the `type_ref` attribute names a scope by name (e.g.,
    /// the class name for an instance variable). Grammar authors emit
    /// `@emit_fact { kind: variable_binding, name: $x, type_kind: class, type_ref: <class-name> }`
    /// to wire this up. Variants on the convention (e.g., `class_ref`
    /// or `package_ref`) can layer atop this primitive in future leaves.
    ///
    /// **Complexity:** O(active_chain_depth × facts_per_scope) for segment
    /// 0; O(arena_size) for finding each subsequent scope by name +
    /// O(facts_in_target_scope) per inner lookup. The future optimisation
    /// (a `(name)`-keyed index across scopes) is tracked under
    /// `feedback_universal_semantic_store`; for now linear scans are
    /// adequate at uvm-scale fact populations (sub-microsecond walks
    /// per the perf contract §3.3 budget for `resolve_path` ≤ 1 µs at
    /// depth 5).
    pub fn resolve_path(&self, path: &str) -> ResolveResult {
        let segments: Vec<&str> = path.split('.').collect();
        if segments.is_empty() || segments.iter().any(|s| s.is_empty()) {
            return ResolveResult::Unresolved {
                resolved_prefix: Vec::new(),
                last_kind: None,
            };
        }
        // Segment 0: walk active chain innermost-first looking for a fact
        // with name == segments[0] in any active scope.
        let mut current_fact_idx: Option<usize> = None;
        for &scope_id in self.active_chain.iter().rev() {
            if let Some(idx) = self.find_fact_position_by_name_in_scope(scope_id, segments[0]) {
                current_fact_idx = Some(idx);
                break;
            }
        }
        let Some(mut fact_idx) = current_fact_idx else {
            return ResolveResult::Unresolved {
                resolved_prefix: Vec::new(),
                last_kind: None,
            };
        };
        let mut resolved_prefix = vec![segments[0].to_string()];
        // Each subsequent segment: follow type_ref to a scope, look up
        // the segment in that scope.
        for segment in &segments[1..] {
            let current_fact = &self.facts[fact_idx];
            let Some(type_ref) = attribute_text(current_fact, "type_ref") else {
                return ResolveResult::Unresolved {
                    resolved_prefix,
                    last_kind: Some(current_fact.kind.clone()),
                };
            };
            let Some(target_scope_id) = self.find_scope_by_name(&type_ref) else {
                return ResolveResult::Unresolved {
                    resolved_prefix,
                    last_kind: Some(current_fact.kind.clone()),
                };
            };
            let Some(next_idx) =
                self.find_fact_position_by_name_in_scope(target_scope_id, segment)
            else {
                return ResolveResult::Unresolved {
                    resolved_prefix,
                    last_kind: Some(current_fact.kind.clone()),
                };
            };
            fact_idx = next_idx;
            resolved_prefix.push((*segment).to_string());
        }
        let final_fact = &self.facts[fact_idx];
        ResolveResult::Resolved {
            kind: final_fact.kind.clone(),
            name: final_fact.name.clone(),
            scope_id: final_fact.scope_id,
            attributes: final_fact.attributes.clone(),
        }
    }

    /// Helper: scan `self.facts` for the first fact in `scope_id` whose
    /// textual `name` matches. Returns the position in `self.facts` so the
    /// caller can borrow the record via index without re-fetching.
    fn find_fact_position_by_name_in_scope(&self, scope_id: ScopeId, name: &str) -> Option<usize> {
        self.facts.iter().position(|f| {
            f.scope_id == scope_id && fact_name_matches(&f.name, name)
        })
    }

    /// Helper: find any scope in the arena (active OR closed) whose `name`
    /// text matches the supplied value. First match wins; ties go to the
    /// earliest arena insertion order (outermost / oldest). Returns
    /// `ScopeId::ROOT` is not a match unless the root's name happens to
    /// equal `name` — typically root.name is None.
    fn find_scope_by_name(&self, name: &str) -> Option<ScopeId> {
        for node in &self.scope_arena {
            if let Some(scope_name) = &node.name {
                if fact_name_matches(scope_name, name) {
                    return Some(node.id);
                }
            }
        }
        None
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
            scope_arena_len: self.scope_arena.len(),
            active_chain_snapshot: self.active_chain.clone(),
        }
    }

    pub fn rollback_to(&mut self, checkpoint: SemanticRuntimeCheckpoint) {
        let fact_len = checkpoint.fact_len.min(self.facts.len());
        let scope_arena_len = checkpoint.scope_arena_len.max(1).min(self.scope_arena.len());
        // `.3.3.4.b.5.1.1`: extend rollback to undo the per-kind index entries
        // for every fact at position `>= fact_len`. The master Vec is the
        // source of truth, so we iterate it in reverse to find the kinds /
        // names / scopes of the facts being discarded, then remove their
        // index entries one by one. Cost is O(facts emitted in the rolled-
        // back portion) — the performance contract's required bound for the
        // rollback primitive (`PGEN_SEMANTIC_STORE_PERFORMANCE_CONTRACT.md`
        // §3.7).
        for position in (fact_len..self.facts.len()).rev() {
            let record = &self.facts[position];
            self.fact_index.remove(
                &record.kind,
                record.scope_depth,
                &record.name,
                position,
            );
        }
        self.facts.truncate(fact_len);
        // `.3.3.4.b.5.1.3`: truncate the arena to checkpoint length —
        // nodes opened during the rolled-back tx are discarded.
        self.scope_arena.truncate(scope_arena_len);
        // Restore the active chain to its checkpoint snapshot exactly.
        // Every entry in the snapshot must reference a node that survives
        // truncation (invariant: nodes in the active chain at checkpoint
        // time had id < arena.len() at that time = scope_arena_len).
        self.active_chain = checkpoint.active_chain_snapshot.clone();
        // Re-open every node in the restored active chain — the rolled-back
        // tx may have called `close_scope` on any subset of them. Reset
        // their `closed` flag to mirror the checkpoint state.
        for &id in &self.active_chain {
            if let Some(node) = self.scope_arena.get_mut(id.index()) {
                node.closed = false;
            }
        }
        // Rebuild the legacy `scopes` Vec in lockstep with the restored
        // active_chain (one frame per active node, in chain order).
        self.scopes = self
            .active_chain
            .iter()
            .map(|&id| {
                let node = &self.scope_arena[id.index()];
                SemanticScopeFrame {
                    kind: node.kind.clone(),
                    name: node.name.clone(),
                }
            })
            .collect();
        debug_assert_eq!(self.scopes.len(), checkpoint.scope_len);
    }

    pub fn commit(&self, checkpoint: SemanticRuntimeCheckpoint) -> bool {
        checkpoint.scope_len <= self.scopes.len()
            && checkpoint.fact_len <= self.facts.len()
            && checkpoint.scope_arena_len <= self.scope_arena.len()
    }

    pub fn open_scope(&mut self, spec: SemanticScopeSpec) {
        // `.3.3.4.b.5.1.3`: allocate a new arena node, parent = current
        // active leaf. Maintain the legacy `scopes` Vec in lockstep so
        // existing predicates that consult `scopes` keep working.
        let parent = Some(self.current_scope_id());
        let depth_when_opened = self.active_chain.len();
        let new_id = ScopeId(self.scope_arena.len() as u32);
        let node = ScopeNode {
            id: new_id,
            parent,
            kind: spec.kind.clone(),
            name: spec.name.clone(),
            closed: false,
            depth_when_opened,
        };
        self.scope_arena.push(node);
        self.active_chain.push(new_id);
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
            // `.3.3.4.b.5.1.3`: pop the active chain AND mark the arena
            // node closed (it stays in the arena, queryable via
            // `scope_arena()` / `scope_children()`).
            if let Some(popped_id) = self.active_chain.pop() {
                if let Some(node) = self.scope_arena.get_mut(popped_id.index()) {
                    node.closed = true;
                }
            }
            self.scopes.pop();
            true
        } else {
            false
        }
    }

    pub fn emit_fact(&mut self, fact: SemanticFactSpec) {
        let scope_depth = self.scopes.len() - 1;
        let scope_id = self.current_scope_id();
        let position = self.facts.len();
        // `.3.3.4.b.5.1.1`: mirror the insertion into the secondary index.
        self.fact_index.insert(&fact.kind, scope_depth, &fact.name, position);
        self.facts.push(SemanticFactRecord {
            kind: fact.kind,
            name: fact.name,
            scope_depth,
            scope_id,
            attributes: fact.attributes,
        });
    }

    /// `SV-EXH-PROOF.3.3.4.a` MVP-0: append an already-formed `SemanticFactRecord`
    /// to the state.
    ///
    /// Used by the library-import path: artifacts contain `SemanticFactRecord`s
    /// that have ALREADY been resolved by the original exporting parser run, so
    /// there is no `SemanticFactSpec` shape to evaluate here — we just merge
    /// them in. `scope_depth` AND `scope_id` are both rebased to the current
    /// scope (the original exporter's depth/id have no meaning in the
    /// importer's scope tree); MVP-0 imports merge at the current active
    /// scope.
    pub fn push_fact_record(&mut self, mut record: SemanticFactRecord) {
        record.scope_depth = self.scopes.len() - 1;
        record.scope_id = self.current_scope_id();
        let position = self.facts.len();
        // `.3.3.4.b.5.1.1`: mirror the import into the secondary index.
        self.fact_index.insert(&record.kind, record.scope_depth, &record.name, position);
        self.facts.push(record);
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
            // `SV-EXH-PROOF.3.3.4.a` MVP-0: library import/export need parser
            // context (the configured lib-in / lib-out dir, the current rule's
            // emitted-fact delta, and library I/O). They are wired into the
            // generator-emitted `with_semantic_runtime_rule_transaction` and
            // are no-ops at the state level (mirroring how `Predicate` is a
            // no-op here — the generator evaluates it with content access).
            SemanticRuntimeDirective::ExportToLibrary(_)
            | SemanticRuntimeDirective::ImportFromLibrary(_) => true,
            // `SV-EXH-PROOF.3.3.4.b.5.1.2`: declarations are compile-time
            // only; at runtime they are no-ops (mirroring `Predicate`
            // and the library directives — actual handling lives in the
            // generator-emitted wrapper).
            SemanticRuntimeDirective::DeclareFactKind(_)
            | SemanticRuntimeDirective::DefinePredicate(_) => true,
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
            // `.3.3.4.b.5.1.1`: index-backed O(1)-avg existence check.
            "has_fact" => {
                let expected_kind = scalar_text(predicate.args.first()?)?;
                let expected_name =
                    SemanticRuntimeValue::from_semantic_value(predicate.args.get(1)?)?;
                Some(self.fact_index.any_with_name(expected_kind, &expected_name))
            }
            // `.3.3.4.b.5.1.1`: index-backed; complement of has_fact.
            "lacks_fact" => {
                let expected_kind = scalar_text(predicate.args.first()?)?;
                let expected_name =
                    SemanticRuntimeValue::from_semantic_value(predicate.args.get(1)?)?;
                Some(!self.fact_index.any_with_name(expected_kind, &expected_name))
            }
            // `.3.3.4.b.5.1.1`: index-backed at exact scope depth via
            // `any_with_name_at_scope`.
            "has_fact_in_current_scope" => {
                let expected_kind = scalar_text(predicate.args.first()?)?;
                let expected_name =
                    SemanticRuntimeValue::from_semantic_value(predicate.args.get(1)?)?;
                let current_depth = self.scopes.len().saturating_sub(1);
                Some(self.fact_index.any_with_name_at_scope(
                    expected_kind,
                    current_depth,
                    &expected_name,
                ))
            }
            // `.3.3.4.b.5.1.1`: index narrows to candidate positions; only
            // the matching facts' attribute lists need scanning. Worst-case
            // is bounded by the number of facts with this (kind, name) tuple
            // — which is normally O(1) for unique-name kinds.
            "has_fact_attribute" => {
                let expected_kind = scalar_text(predicate.args.first()?)?;
                let expected_name =
                    SemanticRuntimeValue::from_semantic_value(predicate.args.get(1)?)?;
                let expected_key = scalar_text(predicate.args.get(2)?)?;
                Some(
                    self.fact_index
                        .positions_for_name(expected_kind, &expected_name)
                        .any(|position| {
                            self.facts[position]
                                .attributes
                                .iter()
                                .any(|property| property.key.eq_ignore_ascii_case(expected_key))
                        }),
                )
            }
            // `.3.3.4.b.5.1.1`: index-backed candidate positions; per-candidate
            // attribute scan as above.
            "fact_attribute_equals" => {
                let expected_kind = scalar_text(predicate.args.first()?)?;
                let expected_name =
                    SemanticRuntimeValue::from_semantic_value(predicate.args.get(1)?)?;
                let expected_key = scalar_text(predicate.args.get(2)?)?;
                let expected_value = predicate.args.get(3)?;
                Some(
                    self.fact_index
                        .positions_for_name(expected_kind, &expected_name)
                        .any(|position| {
                            self.facts[position].attributes.iter().any(|property| {
                                property.key.eq_ignore_ascii_case(expected_key)
                                    && semantic_values_match(&property.value, expected_value)
                            })
                        }),
                )
            }
            // `.3.3.4.b.5.1.1`: index-backed; complement of fact_attribute_equals.
            "lacks_fact_attribute_equals" => {
                let expected_kind = scalar_text(predicate.args.first()?)?;
                let expected_name =
                    SemanticRuntimeValue::from_semantic_value(predicate.args.get(1)?)?;
                let expected_key = scalar_text(predicate.args.get(2)?)?;
                let expected_value = predicate.args.get(3)?;
                Some(
                    !self
                        .fact_index
                        .positions_for_name(expected_kind, &expected_name)
                        .any(|position| {
                            self.facts[position].attributes.iter().any(|property| {
                                property.key.eq_ignore_ascii_case(expected_key)
                                    && semantic_values_match(&property.value, expected_value)
                            })
                        }),
                )
            }
            "scope_depth_at_least" => {
                let minimum_depth = scalar_text(predicate.args.first()?)?
                    .parse::<usize>()
                    .ok()?;
                Some(self.scopes.len().saturating_sub(1) >= minimum_depth)
            }
            // Parser-agnostic, general context-steering primitive: is the
            // number of facts of `kind` emitted so far (in source order, as
            // the parser has progressed) at least the integer threshold?
            // A strict generalization of `has_fact` (which is the
            // `count(kind, name) >= 1` special case): `fact_count_at_least`
            // counts by `kind` only, so `name` is irrelevant. Usable by ANY
            // grammar for context-sensitive decisions — declaration-count
            // -before-use, nesting/recursion bounds, ordinal-sensitive
            // parsing, etc. No parser-specific concept lives here; a grammar
            // chooses which rule `@emit_fact`s which `kind` and which rule
            // `@predicate`-gates on the running count (e.g. the regex grammar
            // emits a capture-group fact and gates `\NN` backref-vs-octal
            // per PCRE2 — `PGEN-RGX-0084`).
            //
            // `.3.3.4.b.5.1.1`: now O(1) via the index's per-kind counter
            // (was O(N) linear scan).
            "fact_count_at_least" => {
                let expected_kind = scalar_text(predicate.args.first()?)?;
                let minimum = scalar_text(predicate.args.get(1)?)?
                    .trim()
                    .parse::<usize>()
                    .ok()?;
                Some(self.fact_index.count_for_kind(expected_kind) >= minimum)
            }
            // `SV-EXH-PROOF.3.3.4.b.5.1.4`: built-in `resolve_path` predicate.
            // Walks the supplied dotted path through the scope tree (per the
            // algorithm documented on `resolve_path`). Returns true iff
            // every segment resolves; false otherwise. Composed predicates
            // (`.b.5.1.5` onwards) can call `resolve_path` and drill into
            // the resulting fact via `.attribute("key")` — that surface is
            // exposed via the `ResolveResult` enum on `resolve_path`.
            "resolve_path" => {
                let path = scalar_text(predicate.args.first()?)?;
                Some(self.resolve_path(path).is_resolved())
            }
            // `SV-EXH-PROOF.3.3.4.b.5.1.5.c`: not a built-in predicate name.
            // Dispatch to a composed `@predicate_def:` if one is registered
            // under this name. The predicate-def registry is keyed by the
            // name as declared (case-sensitive), so look up the trimmed
            // original name (not the lowercased `normalized_name`). The call
            // site's args are already resolved to concrete values by
            // `resolve_semantic_predicate_spec_against_content` before this
            // point; extract their text and bind positionally.
            _ => {
                let def = self.predicate_defs.get(predicate.name.trim())?;
                let call_args: Vec<String> = predicate
                    .args
                    .iter()
                    .filter_map(|a| scalar_text(a).map(str::to_string))
                    .collect();
                self.evaluate_composed_predicate(def, &call_args)
            }
        }
    }

    // -------------------------------------------------------------------------
    // `.3.3.4.b.5.1.5` composed-predicate evaluation.
    //
    // A `@predicate_def:` body is a `PredicateExpr` tree over the four
    // built-in primitives. `evaluate_composed_predicate` binds the call
    // site's args to the def's parameter names, then `eval_predicate_expr`
    // walks the tree.
    // -------------------------------------------------------------------------

    /// Evaluate a composed predicate `def` with `call_args` bound positionally
    /// to `def.args`. Returns `None` on arity mismatch or any sub-evaluation
    /// that cannot be resolved (treated as "indeterminate", same convention
    /// as `evaluate_predicate`).
    pub fn evaluate_composed_predicate(
        &self,
        def: &PredicateDef,
        call_args: &[String],
    ) -> Option<bool> {
        if call_args.len() != def.args.len() {
            // Arity mismatch between the call site and the definition.
            return None;
        }
        let bindings: HashMap<String, String> = def
            .args
            .iter()
            .cloned()
            .zip(call_args.iter().cloned())
            .collect();
        self.eval_predicate_expr(&def.body, &bindings)
    }

    /// Evaluate a `PredicateExpr` boolean tree against this state, with
    /// `$arg` references resolved through `bindings`.
    pub fn eval_predicate_expr(
        &self,
        expr: &PredicateExpr,
        bindings: &HashMap<String, String>,
    ) -> Option<bool> {
        match expr {
            PredicateExpr::Call(call) => self.eval_primitive_call_as_bool(call, bindings),
            PredicateExpr::Not(inner) => Some(!self.eval_predicate_expr(inner, bindings)?),
            PredicateExpr::And(a, b) => {
                let left = self.eval_predicate_expr(a, bindings)?;
                if !left {
                    return Some(false); // short-circuit
                }
                self.eval_predicate_expr(b, bindings)
            }
            PredicateExpr::Or(a, b) => {
                let left = self.eval_predicate_expr(a, bindings)?;
                if left {
                    return Some(true); // short-circuit
                }
                self.eval_predicate_expr(b, bindings)
            }
            PredicateExpr::Compare { lhs, op, rhs } => {
                let lhs_text = self.eval_predicate_value(lhs, bindings)?;
                let rhs_text = self.eval_predicate_value(rhs, bindings)?;
                Some(compare_predicate_values(&lhs_text, *op, &rhs_text))
            }
            PredicateExpr::In { lhs, set } => {
                let lhs_text = self.eval_predicate_value(lhs, bindings)?;
                for member in set {
                    let member_text = self.eval_predicate_value(member, bindings)?;
                    if lhs_text == member_text {
                        return Some(true);
                    }
                }
                Some(false)
            }
        }
    }

    /// Evaluate a value-bearing sub-expression to its textual content.
    fn eval_predicate_value(
        &self,
        value: &PredicateValue,
        bindings: &HashMap<String, String>,
    ) -> Option<String> {
        match value {
            PredicateValue::ArgRef(name) => bindings.get(name).cloned(),
            PredicateValue::StringLit(s) => Some(s.clone()),
            PredicateValue::IntLit(n) => Some(n.to_string()),
            PredicateValue::IdentLit(s) => Some(s.clone()),
            PredicateValue::AttributeOf { call, key } => {
                // Per V-QDEF-5 the call is always `resolve_path`.
                let path = self
                    .eval_predicate_value(call.args.first()?, bindings)?;
                self.resolve_path(&path)
                    .attribute(key)
                    .and_then(|v| scalar_text(v).map(str::to_string))
            }
        }
    }

    /// Evaluate a bare primitive call used as a boolean. Builds a
    /// `SemanticPredicateSpec` and delegates to `evaluate_predicate` so the
    /// composed-predicate path reuses the exact same primitive semantics.
    fn eval_primitive_call_as_bool(
        &self,
        call: &PrimitiveCall,
        bindings: &HashMap<String, String>,
    ) -> Option<bool> {
        let mut spec_args = Vec::with_capacity(call.args.len());
        for arg in &call.args {
            let text = self.eval_predicate_value(arg, bindings)?;
            spec_args.push(UnifiedSemanticValue::Identifier(text));
        }
        let spec = SemanticPredicateSpec {
            name: call.name.clone(),
            args: spec_args,
            phase: SemanticPredicatePhase::Pre,
            view: SemanticPredicateContentView::Raw,
        };
        self.evaluate_predicate(&spec)
    }

    pub fn evaluate_content_aware_predicate<'input>(
        &self,
        predicate: &SemanticPredicateSpec,
        raw_content: &ParseContent<'input>,
        shaped_content: &ParseContent<'input>,
    ) -> Option<bool> {
        let selected_content = match predicate.view {
            SemanticPredicateContentView::Raw => raw_content,
            SemanticPredicateContentView::Shaped => shaped_content,
        };

        let normalized_name = predicate.name.trim().to_ascii_lowercase();
        match normalized_name.as_str() {
            "content_kind_is" => {
                let expected = scalar_text(predicate.args.first()?)?
                    .trim()
                    .to_ascii_lowercase();
                Some(content_kind_name(selected_content) == expected)
            }
            _ => self.evaluate_predicate(predicate),
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
            | SemanticRuntimeDirective::Predicate(_)
            | SemanticRuntimeDirective::ExportToLibrary(_)
            | SemanticRuntimeDirective::ImportFromLibrary(_)
            | SemanticRuntimeDirective::DeclareFactKind(_)
            | SemanticRuntimeDirective::DefinePredicate(_) => None,
        }
    }

    pub fn evaluate_post_directive_predicate<'input>(
        &self,
        directive: &SemanticRuntimeDirective,
        raw_content: &ParseContent<'input>,
        shaped_content: &ParseContent<'input>,
    ) -> Option<bool> {
        match directive {
            SemanticRuntimeDirective::Predicate(spec)
                if spec.phase == SemanticPredicatePhase::Post =>
            {
                self.evaluate_content_aware_predicate(spec, raw_content, shaped_content)
            }
            SemanticRuntimeDirective::OpenScope(_)
            | SemanticRuntimeDirective::CloseScope(_)
            | SemanticRuntimeDirective::EmitFact(_)
            | SemanticRuntimeDirective::Predicate(_)
            | SemanticRuntimeDirective::ExportToLibrary(_)
            | SemanticRuntimeDirective::ImportFromLibrary(_)
            | SemanticRuntimeDirective::DeclareFactKind(_)
            | SemanticRuntimeDirective::DefinePredicate(_) => None,
        }
    }
}

impl<'a> SemanticRuntimeTransaction<'a> {
    pub fn state(&self) -> &SemanticRuntimeState {
        self.state
    }

    /// `SV-EXH-PROOF.3.3.4.a` MVP-0: mutable access to the transaction's state.
    /// Used by `@import_from_library` to merge facts read from a library
    /// artifact into the in-progress transaction. Any mutation goes through the
    /// transaction's checkpoint, so a later rollback unwinds it cleanly.
    pub fn state_mut(&mut self) -> &mut SemanticRuntimeState {
        self.state
    }

    pub fn checkpoint(&self) -> SemanticRuntimeCheckpoint {
        self.checkpoint.clone()
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
        self.state.rollback_to(self.checkpoint.clone());
        self.committed = true;
    }

    pub fn commit(mut self) -> bool {
        let committed = self.state.commit(self.checkpoint.clone());
        self.committed = true;
        committed
    }
}

impl Drop for SemanticRuntimeTransaction<'_> {
    fn drop(&mut self) {
        if !self.committed {
            self.state.rollback_to(self.checkpoint.clone());
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
        // `SV-EXH-PROOF.3.3.4.a` MVP-0 parser-agnostic library directives.
        "export_to_library" => parse_export_to_library(annotation.ast()).map(Some),
        "import_from_library" => parse_import_from_library(annotation.ast()).map(Some),
        // `SV-EXH-PROOF.3.3.4.b.5.1.2`: grammar-level fact-kind declaration
        // (Stage 1 of the lifecycle protocol).
        "fact_kind" => parse_fact_kind(annotation.ast()).map(Some),
        // `SV-EXH-PROOF.3.3.4.b.5.1.5`: grammar-level composed-predicate
        // definition (Stage 3 of the lifecycle protocol).
        "predicate_def" => parse_predicate_def(annotation.ast()).map(Some),
        _ => Ok(None),
    }
}

/// `SV-EXH-PROOF.3.3.4.b.5.1.5`: parses `@predicate_def: { name: <ident>,
/// args: [<idents>], body: <string> }`.
///
/// The `body` field is a quoted string whose content is the predicate
/// expression (parsed by `predicate_expr::parse_predicate_expression`).
/// Per-declaration validation (V-QDEF-2..5) runs here via
/// `PredicateDef::validate_local`; V-QDEF-1 (uniqueness + non-shadowing)
/// is deferred to `compile_semantic_runtime_annotations`.
fn parse_predicate_def(ast: &UnifiedSemanticAST) -> Result<SemanticRuntimeDirective, String> {
    let payload = ast.structured_value().ok_or_else(|| {
        "Directive '@predicate_def' expects a structured object payload.".to_string()
    })?;
    let properties = object_properties(payload)
        .ok_or_else(|| "Directive '@predicate_def' expects an object payload.".to_string())?;

    let mut name: Option<String> = None;
    let mut args: Vec<String> = Vec::new();
    let mut body_text: Option<String> = None;

    for property in properties {
        match property.key.as_str() {
            "name" => {
                name = Some(
                    scalar_text(&property.value)
                        .ok_or_else(|| {
                            format!(
                                "@predicate_def 'name' must be a scalar identifier; got {:?}.",
                                property.value
                            )
                        })?
                        .to_string(),
                );
            }
            "args" => {
                args = parse_string_list("args", &property.value)?;
            }
            "body" => {
                body_text = Some(
                    scalar_text(&property.value)
                        .ok_or_else(|| {
                            "@predicate_def 'body' must be a (quoted) string expression."
                                .to_string()
                        })?
                        .to_string(),
                );
            }
            other => {
                return Err(format!(
                    "@predicate_def: unknown field '{}' (accepted: name, args, body).",
                    other
                ));
            }
        }
    }

    let name =
        name.ok_or_else(|| "@predicate_def: required field 'name' is missing.".to_string())?;
    let body_text = body_text.ok_or_else(|| {
        format!("@predicate_def '{}': required field 'body' is missing.", name)
    })?;

    let body = parse_predicate_expression(&body_text)
        .map_err(|err| format!("@predicate_def '{}': body parse failed — {}", name, err))?;

    let decl = PredicateDef { name, args, body };
    // V-QDEF-2..5 (per-declaration).
    decl.validate_local()?;

    Ok(SemanticRuntimeDirective::DefinePredicate(decl))
}

/// `SV-EXH-PROOF.3.3.4.b.5.1.2`: parses `@fact_kind: { name: <ident>, attributes: [...],
/// required: [...]?, indexes: [(...), ...]?, scope_kind: <ident>?,
/// exportable: <bool>?, artefact_kind: <ident>?, description: <string>? }`.
///
/// Per-declaration validation (V-DECL-2..5, V-DECL-7) runs at parse time via
/// `FactKindDecl::validate_local`. Cross-declaration rules (V-DECL-1
/// uniqueness, V-DECL-6 scope_kind label) are deferred to
/// `compile_semantic_runtime_annotations`.
fn parse_fact_kind(ast: &UnifiedSemanticAST) -> Result<SemanticRuntimeDirective, String> {
    let payload = ast.structured_value().ok_or_else(|| {
        "Directive '@fact_kind' expects a structured object payload.".to_string()
    })?;
    let properties = object_properties(payload)
        .ok_or_else(|| "Directive '@fact_kind' expects an object payload.".to_string())?;

    let mut decl = FactKindDecl::default();
    let mut seen_attributes = false;

    for property in properties {
        match property.key.as_str() {
            "name" => {
                decl.name = scalar_text(&property.value)
                    .ok_or_else(|| {
                        format!(
                            "@fact_kind 'name' field must be a scalar identifier; got {:?}.",
                            property.value
                        )
                    })?
                    .to_string();
            }
            "attributes" => {
                decl.attributes = parse_string_list("attributes", &property.value)?;
                seen_attributes = true;
            }
            "required" => {
                decl.required = parse_string_list("required", &property.value)?;
            }
            "indexes" => {
                decl.indexes = parse_index_list("indexes", &property.value)?;
            }
            "scope_kind" => {
                decl.scope_kind = Some(
                    scalar_text(&property.value)
                        .ok_or_else(|| {
                            format!(
                                "@fact_kind 'scope_kind' field must be a scalar; got {:?}.",
                                property.value
                            )
                        })?
                        .to_string(),
                );
            }
            "exportable" => {
                decl.exportable = parse_bool_literal("exportable", &property.value)?;
            }
            "artefact_kind" => {
                decl.artefact_kind = Some(
                    scalar_text(&property.value)
                        .ok_or_else(|| {
                            format!(
                                "@fact_kind 'artefact_kind' field must be a scalar; got {:?}.",
                                property.value
                            )
                        })?
                        .to_string(),
                );
            }
            "description" => {
                decl.description = Some(
                    scalar_text(&property.value)
                        .ok_or_else(|| {
                            format!(
                                "@fact_kind 'description' field must be a scalar string; got {:?}.",
                                property.value
                            )
                        })?
                        .to_string(),
                );
            }
            other => {
                return Err(format!(
                    "@fact_kind: unknown field '{}' (accepted: name, attributes, required, indexes, scope_kind, exportable, artefact_kind, description).",
                    other
                ));
            }
        }
    }

    // Mandatory-field check (surface-level — V-DECL-2 fires deeper for the
    // attributes content check; here we only catch the totally-missing case).
    if decl.name.is_empty() {
        return Err("@fact_kind: required field 'name' is missing.".to_string());
    }
    if !seen_attributes {
        return Err(format!(
            "@fact_kind '{}': required field 'attributes' is missing.",
            decl.name
        ));
    }

    // V-DECL-2 / -3 / -4 / -5 / -7 (per-declaration).
    decl.validate_local()?;

    Ok(SemanticRuntimeDirective::DeclareFactKind(decl))
}

/// Parse a UnifiedSemanticValue::Array of scalar strings into a `Vec<String>`.
/// Helper used by `@fact_kind`'s `attributes` and `required` fields.
fn parse_string_list(field_name: &str, value: &UnifiedSemanticValue) -> Result<Vec<String>, String> {
    let UnifiedSemanticValue::Array(items) = value else {
        return Err(format!(
            "@fact_kind '{}' field must be a list; got {:?}.",
            field_name, value
        ));
    };
    items
        .iter()
        .map(|item| {
            scalar_text(item)
                .map(str::to_string)
                .ok_or_else(|| {
                    format!(
                        "@fact_kind '{}' list element must be a scalar identifier; got {:?}.",
                        field_name, item
                    )
                })
        })
        .collect()
}

/// Parse the `indexes:` field: a list of tuples, each tuple a list of
/// attribute names. Today UnifiedSemanticValue has no native tuple form;
/// we accept nested arrays (`[[name, scope], [container]]`) as the surface.
fn parse_index_list(
    field_name: &str,
    value: &UnifiedSemanticValue,
) -> Result<Vec<Vec<String>>, String> {
    let UnifiedSemanticValue::Array(items) = value else {
        return Err(format!(
            "@fact_kind '{}' field must be a list of tuples (nested lists); got {:?}.",
            field_name, value
        ));
    };
    items
        .iter()
        .map(|tuple| parse_string_list("indexes (inner tuple)", tuple))
        .collect()
}

/// Parse a boolean literal from a UnifiedSemanticValue. Accepts `true`/`false`
/// (UnifiedSemanticValue::Boolean) and the identifier forms `true` / `false`
/// (UnifiedSemanticValue::Identifier) for grammar-author convenience.
fn parse_bool_literal(field_name: &str, value: &UnifiedSemanticValue) -> Result<bool, String> {
    match value {
        UnifiedSemanticValue::Boolean(b) => Ok(*b),
        UnifiedSemanticValue::Identifier(s) if s.eq_ignore_ascii_case("true") => Ok(true),
        UnifiedSemanticValue::Identifier(s) if s.eq_ignore_ascii_case("false") => Ok(false),
        _ => Err(format!(
            "@fact_kind '{}' field must be a boolean (true/false); got {:?}.",
            field_name, value
        )),
    }
}

/// `SV-EXH-PROOF.3.3.4.a` MVP-0: parses `@export_to_library: { kind: <static>,
/// name_from: <field-or-positional-expr> }`. `kind` must be a non-empty scalar
/// (e.g. `package`, `module`, `entity`); `name_from` is resolved against the
/// rule's parse content at directive-evaluation time (typically a field
/// reference like `$body` or `$package_identifier.body`).
fn parse_export_to_library(
    ast: &UnifiedSemanticAST,
) -> Result<SemanticRuntimeDirective, String> {
    let payload = ast.structured_value().ok_or_else(|| {
        "Directive '@export_to_library' expects a structured object payload.".to_string()
    })?;
    let properties = object_properties(payload).ok_or_else(|| {
        "Directive '@export_to_library' expects an object payload.".to_string()
    })?;

    let kind = property(properties, "kind")
        .and_then(scalar_text)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            "Directive '@export_to_library' requires a non-empty 'kind' field.".to_string()
        })?
        .to_string();
    let name = property(properties, "name_from")
        .ok_or_else(|| {
            "Directive '@export_to_library' requires a 'name_from' field.".to_string()
        })
        .and_then(|value| {
            SemanticRuntimeValue::from_semantic_value(value).ok_or_else(|| {
                "Directive '@export_to_library.name_from' must be a scalar or rule reference."
                    .to_string()
            })
        })?;

    for property in properties {
        if matches!(property.key.as_str(), "kind" | "name_from") {
            continue;
        }
        return Err(format!(
            "Directive '@export_to_library' rejects unknown field '{}' (MVP-0 supports 'kind' and 'name_from').",
            property.key
        ));
    }

    Ok(SemanticRuntimeDirective::ExportToLibrary(
        SemanticLibraryExportSpec { kind, name },
    ))
}

/// `SV-EXH-PROOF.3.3.4.a` MVP-0: parses `@import_from_library: { kind: <static>,
/// name_from: <field-or-positional-expr> }`. Same payload shape as
/// `@export_to_library`.
fn parse_import_from_library(
    ast: &UnifiedSemanticAST,
) -> Result<SemanticRuntimeDirective, String> {
    let payload = ast.structured_value().ok_or_else(|| {
        "Directive '@import_from_library' expects a structured object payload.".to_string()
    })?;
    let properties = object_properties(payload).ok_or_else(|| {
        "Directive '@import_from_library' expects an object payload.".to_string()
    })?;

    let kind = property(properties, "kind")
        .and_then(scalar_text)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            "Directive '@import_from_library' requires a non-empty 'kind' field.".to_string()
        })?
        .to_string();
    let name = property(properties, "name_from")
        .ok_or_else(|| {
            "Directive '@import_from_library' requires a 'name_from' field.".to_string()
        })
        .and_then(|value| {
            SemanticRuntimeValue::from_semantic_value(value).ok_or_else(|| {
                "Directive '@import_from_library.name_from' must be a scalar or rule reference."
                    .to_string()
            })
        })?;

    for property in properties {
        if matches!(property.key.as_str(), "kind" | "name_from") {
            continue;
        }
        return Err(format!(
            "Directive '@import_from_library' rejects unknown field '{}' (MVP-0 supports 'kind' and 'name_from').",
            property.key
        ));
    }

    Ok(SemanticRuntimeDirective::ImportFromLibrary(
        SemanticLibraryImportSpec { kind, name },
    ))
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
    let mut branch_directives_by_rule = HashMap::new();
    // `SV-EXH-PROOF.3.3.4.b.5.1.2`: collect `@fact_kind:` declarations
    // from every rule's annotations. V-DECL-1 (uniqueness across the grammar)
    // is checked HERE; V-DECL-2..5, V-DECL-7 are already enforced at parse
    // time via `FactKindDecl::validate_local`.
    let mut fact_kinds: HashMap<String, FactKindDecl> = HashMap::new();
    // `SV-EXH-PROOF.3.3.4.b.5.1.5`: registry of `@predicate_def:` composed
    // predicates. V-QDEF-1 (uniqueness + non-shadowing) checked as it fills.
    let mut predicate_defs: HashMap<String, PredicateDef> = HashMap::new();
    // Track all scope-kind labels referenced by `@open_scope` directives so
    // V-DECL-6 (scope_kind ∈ open_scope labels ∪ engine reserved) can be
    // validated after the full pass. Warn-only per design (lenient).
    let mut declared_scope_kinds: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (rule_name, semantic_annotations) in &annotations.semantic_annotations {
        let directives =
            compile_rule_semantic_runtime_directives(rule_name, semantic_annotations.iter())?;
        collect_fact_kind_decls_and_scope_kinds(
            &directives,
            &mut fact_kinds,
            &mut predicate_defs,
            &mut declared_scope_kinds,
        )?;
        if !directives.is_empty() {
            directives_by_rule.insert(rule_name.clone(), directives);
        }
    }
    for (rule_name, branch_semantic_annotations) in &annotations.branch_semantic_annotations {
        let mut compiled_branches = Vec::with_capacity(branch_semantic_annotations.len());
        let mut has_runtime_directive = false;
        for (branch_index, semantic_annotations) in branch_semantic_annotations.iter().enumerate() {
            let directives = compile_rule_semantic_runtime_directives(
                rule_name,
                semantic_annotations.iter(),
            )
            .map_err(|err| {
                format!(
                    "Failed to compile branch semantic runtime directives for rule '{}' branch #{}: {}",
                    rule_name,
                    branch_index + 1,
                    err
                )
            })?;
            collect_fact_kind_decls_and_scope_kinds(
                &directives,
                &mut fact_kinds,
                &mut predicate_defs,
                &mut declared_scope_kinds,
            )?;
            if !directives.is_empty() {
                has_runtime_directive = true;
            }
            compiled_branches.push(directives);
        }
        if has_runtime_directive {
            branch_directives_by_rule.insert(rule_name.clone(), compiled_branches);
        }
    }

    // V-DECL-6: warn (do not error) if any `@fact_kind` references a
    // `scope_kind` label not found in `@open_scope` directives and not in
    // the engine-reserved set. We don't have an output stream here — emit
    // via eprintln! at compile time. Test suites assert via the (private)
    // helper `validate_scope_kind_label_for_test`; grammar authors will see
    // the warning during codegen.
    for decl in fact_kinds.values() {
        if let Some(scope_kind) = &decl.scope_kind {
            if !ENGINE_RESERVED_SCOPE_KINDS.contains(&scope_kind.as_str())
                && !declared_scope_kinds.contains(scope_kind)
            {
                eprintln!(
                    "warning: @fact_kind '{}': V-DECL-6 — scope_kind '{}' is not in the engine-reserved set ({:?}) and not declared by any @open_scope directive in this grammar. This will silently match no scope at runtime.",
                    decl.name, scope_kind, ENGINE_RESERVED_SCOPE_KINDS
                );
            }
        }
    }

    Ok(CompiledSemanticRuntimeAnnotations {
        directives_by_rule,
        branch_directives_by_rule,
        fact_kinds,
        predicate_defs,
    })
}

/// `SV-EXH-PROOF.3.3.4.b.5.1.5`: every predicate name the engine recognises
/// as a built-in (evaluated directly by `evaluate_predicate` /
/// `evaluate_content_aware_predicate`). A `@predicate_def` may not shadow
/// any of these (V-QDEF-1).
const ENGINE_BUILTIN_PREDICATE_NAMES: &[&str] = &[
    "current_scope_is",
    "has_fact",
    "lacks_fact",
    "has_fact_in_current_scope",
    "has_fact_attribute",
    "fact_attribute_equals",
    "lacks_fact_attribute_equals",
    "scope_depth_at_least",
    "fact_count_at_least",
    "resolve_path",
    "content_kind_is",
];

/// `SV-EXH-PROOF.3.3.4.b.5.1.2` + `.b.5.1.5`: scan a list of directives,
/// extracting `DeclareFactKind` payloads into the fact-kind registry
/// (V-DECL-1) and `DefinePredicate` payloads into the predicate-def registry
/// (V-QDEF-1), and accumulating `OpenScope` kinds into the per-grammar label
/// set for V-DECL-6's deferred check.
fn collect_fact_kind_decls_and_scope_kinds(
    directives: &[SemanticRuntimeDirective],
    fact_kinds: &mut HashMap<String, FactKindDecl>,
    predicate_defs: &mut HashMap<String, PredicateDef>,
    scope_kinds: &mut std::collections::HashSet<String>,
) -> Result<(), String> {
    for directive in directives {
        match directive {
            SemanticRuntimeDirective::DeclareFactKind(decl) => {
                // V-DECL-1: uniqueness across the grammar. If the same kind
                // name is declared twice, the second declaration is rejected
                // with a precise message naming the conflicting kind.
                if let Some(prior) = fact_kinds.get(&decl.name) {
                    if prior != decl {
                        return Err(format!(
                            "@fact_kind '{}': V-DECL-1 violation — declared more than once with conflicting payloads (first: {:?}; second: {:?}).",
                            decl.name, prior, decl
                        ));
                    }
                    // Identical re-declarations are allowed (e.g., the same
                    // @fact_kind: block attached to two rules for visibility).
                    // This is a deliberate convenience; downstream consumers
                    // see a single registry entry.
                    continue;
                }
                fact_kinds.insert(decl.name.clone(), decl.clone());
            }
            SemanticRuntimeDirective::DefinePredicate(decl) => {
                // V-QDEF-1: must not shadow a built-in predicate.
                if ENGINE_BUILTIN_PREDICATE_NAMES.contains(&decl.name.as_str()) {
                    return Err(format!(
                        "@predicate_def '{}': V-QDEF-1 violation — name shadows a built-in predicate ({:?}).",
                        decl.name, ENGINE_BUILTIN_PREDICATE_NAMES
                    ));
                }
                // V-QDEF-1: uniqueness across the grammar. Identical
                // re-declarations are allowed (same convenience carve-out
                // as @fact_kind); conflicting payloads are rejected.
                if let Some(prior) = predicate_defs.get(&decl.name) {
                    if prior != decl {
                        return Err(format!(
                            "@predicate_def '{}': V-QDEF-1 violation — defined more than once with conflicting payloads.",
                            decl.name
                        ));
                    }
                    continue;
                }
                predicate_defs.insert(decl.name.clone(), decl.clone());
            }
            SemanticRuntimeDirective::OpenScope(spec) => {
                // Accumulate user-declared scope-kind labels for V-DECL-6.
                let label = spec.kind.label();
                scope_kinds.insert(label.to_string());
            }
            _ => {}
        }
    }
    Ok(())
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
                    Some(value) => SemanticPredicateContentView::parse(value).ok_or_else(|| {
                        "Directive '@predicate.view' must be either 'raw' or 'shaped'.".to_string()
                    })?,
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

/// `SV-EXH-PROOF.3.3.4.b.5.1.4`: textual content of a `SemanticRuntimeValue`
/// for path-resolution comparisons. Matches `scalar_text` semantics but
/// operates on the `SemanticRuntimeValue` enum (used by fact names, scope
/// names) rather than `UnifiedSemanticValue`.
fn semantic_runtime_value_text(value: &SemanticRuntimeValue) -> Option<&str> {
    match value {
        SemanticRuntimeValue::String(s)
        | SemanticRuntimeValue::Identifier(s)
        | SemanticRuntimeValue::RuleReference(s)
        | SemanticRuntimeValue::Number(s) => Some(s.as_str()),
        SemanticRuntimeValue::Boolean(_) | SemanticRuntimeValue::Null => None,
    }
}

/// `SV-EXH-PROOF.3.3.4.b.5.1.4`: true when `value`'s textual content equals
/// `name` (case-sensitive — names in SV are case-sensitive). Returns false
/// for non-textual variants (Boolean, Null).
fn fact_name_matches(value: &SemanticRuntimeValue, name: &str) -> bool {
    semantic_runtime_value_text(value).is_some_and(|t| t == name)
}

/// `SV-EXH-PROOF.3.3.4.b.5.1.4`: look up an attribute's textual value on a
/// fact record. Returns `None` if the attribute is absent or its value is
/// non-scalar (e.g., an embedded array or object).
fn attribute_text(fact: &SemanticFactRecord, key: &str) -> Option<String> {
    fact.attributes
        .iter()
        .find(|p| p.key.eq_ignore_ascii_case(key))
        .and_then(|p| scalar_text(&p.value).map(str::to_string))
}

/// `SV-EXH-PROOF.3.3.4.b.5.1.5`: compare two textual values per a
/// `CompareOp`. `==` / `!=` are string equality. The ordering operators
/// (`<` / `<=` / `>` / `>=`) compare numerically when BOTH operands parse
/// as `i64`, and fall back to lexical ordering otherwise.
fn compare_predicate_values(
    lhs: &str,
    op: super::predicate_expr::CompareOp,
    rhs: &str,
) -> bool {
    use super::predicate_expr::CompareOp;
    match op {
        CompareOp::Eq => lhs == rhs,
        CompareOp::Ne => lhs != rhs,
        CompareOp::Lt | CompareOp::Le | CompareOp::Gt | CompareOp::Ge => {
            match (lhs.parse::<i64>(), rhs.parse::<i64>()) {
                (Ok(l), Ok(r)) => match op {
                    CompareOp::Lt => l < r,
                    CompareOp::Le => l <= r,
                    CompareOp::Gt => l > r,
                    CompareOp::Ge => l >= r,
                    _ => unreachable!(),
                },
                _ => match op {
                    CompareOp::Lt => lhs < rhs,
                    CompareOp::Le => lhs <= rhs,
                    CompareOp::Gt => lhs > rhs,
                    CompareOp::Ge => lhs >= rhs,
                    _ => unreachable!(),
                },
            }
        }
    }
}

fn semantic_values_match(left: &UnifiedSemanticValue, right: &UnifiedSemanticValue) -> bool {
    match (scalar_text(left), scalar_text(right)) {
        (Some(left_text), Some(right_text)) => left_text == right_text,
        _ => left == right,
    }
}

fn content_kind_name(content: &ParseContent<'_>) -> &'static str {
    match content {
        ParseContent::Terminal(_) => "terminal",
        ParseContent::TransformedTerminal(_) => "transformed_terminal",
        ParseContent::Json(_) => "json",
        ParseContent::Sequence(_) => "sequence",
        ParseContent::Alternative(_) => "alternative",
        ParseContent::Quantified(_, _) => "quantified",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CompiledSemanticRuntimeAnnotations, SemanticFactSpec, SemanticLibraryExportSpec,
        SemanticLibraryImportSpec, SemanticPredicateContentView, SemanticPredicatePhase,
        SemanticPredicateSpec, SemanticRuntimeDirective, SemanticRuntimeState, SemanticRuntimeValue,
        SemanticScopeKind, compile_rule_semantic_runtime_directives,
        compile_semantic_runtime_annotations, parse_semantic_runtime_directive,
        parse_semantic_runtime_directives,
    };
    use crate::ast_pipeline::{
        Annotations, ParseContent, ParseNode, SemanticAnnotation, UnifiedSemanticAST,
        UnifiedSemanticValue,
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

    // `SV-EXH-PROOF.3.3.4.a` MVP-0 directive-parser tests for the
    // parser-agnostic `@export_to_library` / `@import_from_library` directives.

    #[test]
    fn parses_export_to_library_runtime_directive() {
        let export = structured_named(
            "export_to_library",
            "{ kind: package, name_from: $body }",
            UnifiedSemanticValue::Object(vec![
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "kind".to_string(),
                    value: UnifiedSemanticValue::Identifier("package".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "name_from".to_string(),
                    value: UnifiedSemanticValue::RuleReference("$body".to_string()),
                },
            ]),
        );

        let parsed =
            parse_semantic_runtime_directive(&export).expect("export_to_library should parse");
        assert_eq!(
            parsed,
            Some(SemanticRuntimeDirective::ExportToLibrary(
                SemanticLibraryExportSpec {
                    kind: "package".to_string(),
                    name: SemanticRuntimeValue::RuleReference("$body".to_string()),
                }
            ))
        );
        // Classifier coverage: not a predicate, not a generic effect.
        let directive = match parsed.as_ref().unwrap() {
            d @ SemanticRuntimeDirective::ExportToLibrary(_) => d,
            other => panic!("unexpected variant: {:?}", other),
        };
        assert!(directive.is_library_export());
        assert!(!directive.is_library_import());
        assert!(!directive.is_effect());
        assert!(directive.predicate_phase().is_none());
    }

    #[test]
    fn parses_import_from_library_runtime_directive() {
        let import = structured_named(
            "import_from_library",
            "{ kind: package, name_from: $package_identifier.body }",
            UnifiedSemanticValue::Object(vec![
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "kind".to_string(),
                    value: UnifiedSemanticValue::Identifier("package".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "name_from".to_string(),
                    value: UnifiedSemanticValue::RuleReference(
                        "$package_identifier.body".to_string(),
                    ),
                },
            ]),
        );

        let parsed =
            parse_semantic_runtime_directive(&import).expect("import_from_library should parse");
        assert_eq!(
            parsed,
            Some(SemanticRuntimeDirective::ImportFromLibrary(
                SemanticLibraryImportSpec {
                    kind: "package".to_string(),
                    name: SemanticRuntimeValue::RuleReference(
                        "$package_identifier.body".to_string()
                    ),
                }
            ))
        );
        let directive = parsed.as_ref().unwrap();
        assert!(directive.is_library_import());
        assert!(!directive.is_library_export());
        assert!(!directive.is_pre_predicate());
    }

    #[test]
    fn library_directive_rejects_unknown_field() {
        let export = structured_named(
            "export_to_library",
            "{ kind: package, name_from: $body, attributes: {} }",
            UnifiedSemanticValue::Object(vec![
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "kind".to_string(),
                    value: UnifiedSemanticValue::Identifier("package".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "name_from".to_string(),
                    value: UnifiedSemanticValue::RuleReference("$body".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "attributes".to_string(),
                    value: UnifiedSemanticValue::Object(Vec::new()),
                },
            ]),
        );

        let err = parse_semantic_runtime_directive(&export)
            .expect_err("unknown field should cause an error");
        assert!(err.contains("rejects unknown field"));
        assert!(err.contains("attributes"));
    }

    #[test]
    fn library_directive_requires_kind() {
        let export = structured_named(
            "export_to_library",
            "{ name_from: $body }",
            UnifiedSemanticValue::Object(vec![
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "name_from".to_string(),
                    value: UnifiedSemanticValue::RuleReference("$body".to_string()),
                },
            ]),
        );

        let err = parse_semantic_runtime_directive(&export)
            .expect_err("missing kind should cause an error");
        assert!(err.contains("requires a non-empty 'kind' field"));
    }

    #[test]
    fn library_directive_requires_name_from() {
        let import = structured_named(
            "import_from_library",
            "{ kind: package }",
            UnifiedSemanticValue::Object(vec![
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "kind".to_string(),
                    value: UnifiedSemanticValue::Identifier("package".to_string()),
                },
            ]),
        );

        let err = parse_semantic_runtime_directive(&import)
            .expect_err("missing name_from should cause an error");
        assert!(err.contains("requires a 'name_from' field"));
    }

    #[test]
    fn compiled_library_directives_register_under_their_rule() {
        // The compiler should route the new directives through
        // `directives_by_rule` so the generator's `library_imports_for_rule`
        // and `library_exports_for_rule` iterators find them. We test via the
        // compile entry point (which the generator uses) to lock the shape.
        let import = structured_named(
            "import_from_library",
            "{ kind: package, name_from: $package_identifier.body }",
            UnifiedSemanticValue::Object(vec![
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "kind".to_string(),
                    value: UnifiedSemanticValue::Identifier("package".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "name_from".to_string(),
                    value: UnifiedSemanticValue::RuleReference(
                        "$package_identifier.body".to_string(),
                    ),
                },
            ]),
        );
        let directives = compile_rule_semantic_runtime_directives(
            "package_import_item",
            std::iter::once(&import),
        )
        .expect("compile should succeed");
        let mut by_rule = std::collections::HashMap::new();
        by_rule.insert("package_import_item".to_string(), directives);
        let compiled = CompiledSemanticRuntimeAnnotations::from_rule_directives(by_rule);

        let imports: Vec<&SemanticRuntimeDirective> = compiled
            .library_imports_for_rule("package_import_item")
            .collect();
        assert_eq!(imports.len(), 1);
        assert!(imports[0].is_library_import());

        // Sanity: an unrelated rule has no library imports.
        assert_eq!(
            compiled.library_imports_for_rule("some_other_rule").count(),
            0
        );
        // Library directives must not leak into the predicate/effect lanes.
        assert_eq!(
            compiled
                .pre_predicates_for_rule("package_import_item")
                .count(),
            0
        );
        assert_eq!(
            compiled
                .effect_directives_for_rule("package_import_item")
                .count(),
            0
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
            "{ kind: typedef, name: my_type, declaration_family: typedef }",
            UnifiedSemanticValue::Object(vec![
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "kind".to_string(),
                    value: UnifiedSemanticValue::Identifier("typedef".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "name".to_string(),
                    value: UnifiedSemanticValue::Identifier("my_type".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "declaration_family".to_string(),
                    value: UnifiedSemanticValue::Identifier("typedef".to_string()),
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
                name: "lacks_fact".to_string(),
                args: vec![
                    UnifiedSemanticValue::Identifier("typedef".to_string()),
                    UnifiedSemanticValue::Identifier("missing_type".to_string()),
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
                name: "has_fact_attribute".to_string(),
                args: vec![
                    UnifiedSemanticValue::Identifier("typedef".to_string()),
                    UnifiedSemanticValue::Identifier("my_type".to_string()),
                    UnifiedSemanticValue::Identifier("declaration_family".to_string()),
                ],
                phase: SemanticPredicatePhase::Pre,
                view: SemanticPredicateContentView::Raw,
            }),
            Some(true)
        );
        assert_eq!(
            state.evaluate_predicate(&SemanticPredicateSpec {
                name: "fact_attribute_equals".to_string(),
                args: vec![
                    UnifiedSemanticValue::Identifier("typedef".to_string()),
                    UnifiedSemanticValue::Identifier("my_type".to_string()),
                    UnifiedSemanticValue::Identifier("declaration_family".to_string()),
                    UnifiedSemanticValue::String("typedef".to_string()),
                ],
                phase: SemanticPredicatePhase::Pre,
                view: SemanticPredicateContentView::Raw,
            }),
            Some(true)
        );
        assert_eq!(
            state.evaluate_predicate(&SemanticPredicateSpec {
                name: "lacks_fact_attribute_equals".to_string(),
                args: vec![
                    UnifiedSemanticValue::Identifier("typedef".to_string()),
                    UnifiedSemanticValue::Identifier("my_type".to_string()),
                    UnifiedSemanticValue::Identifier("declaration_family".to_string()),
                    UnifiedSemanticValue::String("class".to_string()),
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
            "{ kind: typedef, name: my_type, declaration_family: typedef }",
            UnifiedSemanticValue::Object(vec![
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "kind".to_string(),
                    value: UnifiedSemanticValue::Identifier("typedef".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "name".to_string(),
                    value: UnifiedSemanticValue::Identifier("my_type".to_string()),
                },
                crate::ast_pipeline::UnifiedSemanticProperty {
                    key: "declaration_family".to_string(),
                    value: UnifiedSemanticValue::Identifier("typedef".to_string()),
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
            state.evaluate_predicate(&SemanticPredicateSpec {
                name: "has_fact_attribute".to_string(),
                args: vec![
                    UnifiedSemanticValue::Identifier("typedef".to_string()),
                    UnifiedSemanticValue::Identifier("my_type".to_string()),
                    UnifiedSemanticValue::Identifier("missing_attribute".to_string()),
                ],
                phase: SemanticPredicatePhase::Pre,
                view: SemanticPredicateContentView::Raw,
            }),
            Some(false)
        );
        assert_eq!(
            state.evaluate_predicate(&SemanticPredicateSpec {
                name: "fact_attribute_equals".to_string(),
                args: vec![
                    UnifiedSemanticValue::Identifier("typedef".to_string()),
                    UnifiedSemanticValue::Identifier("my_type".to_string()),
                    UnifiedSemanticValue::Identifier("declaration_family".to_string()),
                    UnifiedSemanticValue::Identifier("class".to_string()),
                ],
                phase: SemanticPredicatePhase::Pre,
                view: SemanticPredicateContentView::Raw,
            }),
            Some(false)
        );
        assert_eq!(
            state.evaluate_predicate(&SemanticPredicateSpec {
                name: "lacks_fact".to_string(),
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
            state.evaluate_predicate(&SemanticPredicateSpec {
                name: "lacks_fact_attribute_equals".to_string(),
                args: vec![
                    UnifiedSemanticValue::Identifier("typedef".to_string()),
                    UnifiedSemanticValue::Identifier("my_type".to_string()),
                    UnifiedSemanticValue::Identifier("declaration_family".to_string()),
                    UnifiedSemanticValue::Identifier("typedef".to_string()),
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
        assert!(
            compiled
                .branch_directives_for_rule("missing_rule")
                .is_empty()
        );
    }

    #[test]
    fn compile_annotations_groups_branch_runtime_directives_by_rule_and_branch() {
        let mut annotations = Annotations::default();
        annotations.branch_semantic_annotations.insert(
            "statement_or_decl".to_string(),
            vec![
                vec![structured_named(
                    "predicate",
                    "{ name: content_kind_is, args: [sequence], phase: branch, view: raw }",
                    UnifiedSemanticValue::Object(vec![
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "name".to_string(),
                            value: UnifiedSemanticValue::Identifier("content_kind_is".to_string()),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "args".to_string(),
                            value: UnifiedSemanticValue::Array(vec![
                                UnifiedSemanticValue::Identifier("sequence".to_string()),
                            ]),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "phase".to_string(),
                            value: UnifiedSemanticValue::Identifier("branch".to_string()),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "view".to_string(),
                            value: UnifiedSemanticValue::Identifier("raw".to_string()),
                        },
                    ]),
                )],
                vec![structured_named(
                    "category",
                    "\"metadata\"",
                    UnifiedSemanticValue::String("metadata".to_string()),
                )],
                vec![structured_named(
                    "predicate",
                    "{ name: content_kind_is, args: [terminal], phase: branch, view: raw }",
                    UnifiedSemanticValue::Object(vec![
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "name".to_string(),
                            value: UnifiedSemanticValue::Identifier("content_kind_is".to_string()),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "args".to_string(),
                            value: UnifiedSemanticValue::Array(vec![
                                UnifiedSemanticValue::Identifier("terminal".to_string()),
                            ]),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "phase".to_string(),
                            value: UnifiedSemanticValue::Identifier("branch".to_string()),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "view".to_string(),
                            value: UnifiedSemanticValue::Identifier("raw".to_string()),
                        },
                    ]),
                )],
            ],
        );

        let compiled = compile_semantic_runtime_annotations(&annotations)
            .expect("compiled semantic runtime annotations should succeed");

        assert_eq!(compiled.len(), 1);
        assert!(compiled.has_rule("statement_or_decl"));
        assert_eq!(
            compiled
                .branch_directives_for_rule("statement_or_decl")
                .len(),
            3
        );
        assert_eq!(
            compiled
                .branch_directives_for_rule_branch("statement_or_decl", 0)
                .len(),
            1
        );
        assert_eq!(
            compiled
                .branch_directives_for_rule_branch("statement_or_decl", 1)
                .len(),
            0
        );
        assert_eq!(
            compiled
                .branch_directives_for_rule_branch("statement_or_decl", 2)
                .len(),
            1
        );
        assert_eq!(
            compiled
                .branch_predicates_for_rule_branch("statement_or_decl", 0)
                .count(),
            1
        );
        assert_eq!(
            compiled
                .branch_predicates_for_rule_branch("statement_or_decl", 1)
                .count(),
            0
        );
        assert_eq!(
            compiled
                .branch_predicates_for_rule_branch("statement_or_decl", 2)
                .count(),
            1
        );
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
                            value: UnifiedSemanticValue::Identifier("current_scope_is".to_string()),
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
                            value: UnifiedSemanticValue::Identifier("current_scope_is".to_string()),
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
        assert!(matches!(effects[0], SemanticRuntimeDirective::OpenScope(_)));
        assert!(compiled.has_post_predicates_for_rule("package_declaration"));
        assert!(!compiled.needs_raw_post_capture_for_rule("package_declaration"));
    }

    #[test]
    fn post_predicates_can_inspect_raw_or_shaped_content_kind() {
        let state = SemanticRuntimeState::new();
        let raw_content = ParseContent::Sequence(vec![ParseNode {
            rule_name: "inner",
            content: ParseContent::Terminal("pkg"),
            span: 0..3,
        }]);
        let shaped_content = ParseContent::TransformedTerminal("pkg".to_string());

        assert_eq!(
            state.evaluate_content_aware_predicate(
                &SemanticPredicateSpec {
                    name: "content_kind_is".to_string(),
                    args: vec![UnifiedSemanticValue::Identifier("sequence".to_string())],
                    phase: SemanticPredicatePhase::Post,
                    view: SemanticPredicateContentView::Raw,
                },
                &raw_content,
                &shaped_content,
            ),
            Some(true)
        );
        assert_eq!(
            state.evaluate_content_aware_predicate(
                &SemanticPredicateSpec {
                    name: "content_kind_is".to_string(),
                    args: vec![UnifiedSemanticValue::Identifier(
                        "transformed_terminal".to_string(),
                    )],
                    phase: SemanticPredicatePhase::Post,
                    view: SemanticPredicateContentView::Shaped,
                },
                &raw_content,
                &shaped_content,
            ),
            Some(true)
        );
        assert_eq!(
            state.evaluate_post_directive_predicate(
                &SemanticRuntimeDirective::Predicate(SemanticPredicateSpec {
                    name: "content_kind_is".to_string(),
                    args: vec![UnifiedSemanticValue::Identifier("terminal".to_string())],
                    phase: SemanticPredicatePhase::Post,
                    view: SemanticPredicateContentView::Raw,
                }),
                &raw_content,
                &shaped_content,
            ),
            Some(false)
        );
    }

    #[test]
    fn compiled_annotations_expose_branch_predicates_separately() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "statement_or_decl".to_string(),
            vec![
                structured_named(
                    "predicate",
                    "{ name: content_kind_is, args: [sequence], phase: branch, view: raw }",
                    UnifiedSemanticValue::Object(vec![
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "name".to_string(),
                            value: UnifiedSemanticValue::Identifier("content_kind_is".to_string()),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "args".to_string(),
                            value: UnifiedSemanticValue::Array(vec![
                                UnifiedSemanticValue::Identifier("sequence".to_string()),
                            ]),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "phase".to_string(),
                            value: UnifiedSemanticValue::Identifier("branch".to_string()),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "view".to_string(),
                            value: UnifiedSemanticValue::Identifier("raw".to_string()),
                        },
                    ]),
                ),
                structured_named(
                    "predicate",
                    "{ name: current_scope_is, args: [global] }",
                    UnifiedSemanticValue::Object(vec![
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "name".to_string(),
                            value: UnifiedSemanticValue::Identifier("current_scope_is".to_string()),
                        },
                        crate::ast_pipeline::UnifiedSemanticProperty {
                            key: "args".to_string(),
                            value: UnifiedSemanticValue::Array(vec![
                                UnifiedSemanticValue::Identifier("global".to_string()),
                            ]),
                        },
                    ]),
                ),
            ],
        );

        let compiled = compile_semantic_runtime_annotations(&annotations)
            .expect("compiled semantic runtime annotations should succeed");
        let branch_predicates = compiled
            .branch_predicates_for_rule("statement_or_decl")
            .collect::<Vec<_>>();
        let pre_predicates = compiled
            .pre_predicates_for_rule("statement_or_decl")
            .collect::<Vec<_>>();

        assert_eq!(branch_predicates.len(), 1);
        assert_eq!(pre_predicates.len(), 1);
        assert!(matches!(
            branch_predicates[0],
            SemanticRuntimeDirective::Predicate(SemanticPredicateSpec {
                phase: SemanticPredicatePhase::Branch,
                ..
            })
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
                            value: UnifiedSemanticValue::Identifier("current_scope_is".to_string()),
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

    #[test]
    fn fact_count_at_least_is_parser_agnostic_generalization_of_has_fact() {
        // `fact_count_at_least` counts facts by `kind` only — a strict
        // generalization of `has_fact` (the `count(kind, name) >= 1`
        // special case). It is a GENERAL context-steering primitive: any
        // grammar may emit a fact and gate on its running count. This test
        // deliberately uses a neutral kind (NOT a regex/capture-group
        // concept) to assert the engine stays parser-agnostic; the
        // PGEN-RGX-0084 regex fix is just one consumer.
        let mut state = SemanticRuntimeState::new();
        let pred = |kind: &str, n: &str| SemanticPredicateSpec {
            name: "fact_count_at_least".to_string(),
            args: vec![
                UnifiedSemanticValue::Identifier(kind.to_string()),
                UnifiedSemanticValue::Identifier(n.to_string()),
            ],
            phase: SemanticPredicatePhase::Pre,
            view: SemanticPredicateContentView::Raw,
        };

        // No facts yet → count 0.
        assert_eq!(
            state.evaluate_predicate(&pred("opened_construct", "1")),
            Some(false)
        );

        // Emit two facts of the kind (names differ — count is by `kind`).
        for nm in ["a", "b"] {
            state.emit_fact(SemanticFactSpec {
                kind: "opened_construct".to_string(),
                name: SemanticRuntimeValue::Identifier(nm.to_string()),
                attributes: Vec::new(),
            });
        }

        assert_eq!(
            state.evaluate_predicate(&pred("opened_construct", "1")),
            Some(true)
        );
        assert_eq!(
            state.evaluate_predicate(&pred("opened_construct", "2")),
            Some(true)
        );
        assert_eq!(
            state.evaluate_predicate(&pred("opened_construct", "3")),
            Some(false)
        );
        // A different kind is independent (parser-agnostic: `kind` is an
        // arbitrary grammar-chosen string the engine never interprets).
        assert_eq!(
            state.evaluate_predicate(&pred("other_kind", "1")),
            Some(false)
        );

        // Strict generalization: count >= 1 ⟺ has_fact(kind, <a name>).
        assert_eq!(
            state.evaluate_predicate(&SemanticPredicateSpec {
                name: "has_fact".to_string(),
                args: vec![
                    UnifiedSemanticValue::Identifier("opened_construct".to_string()),
                    UnifiedSemanticValue::Identifier("a".to_string()),
                ],
                phase: SemanticPredicatePhase::Pre,
                view: SemanticPredicateContentView::Raw,
            }),
            Some(true)
        );
    }

    // -------------------------------------------------------------------------
    // `.3.3.4.b.5.1.1` multi-index tests (U-EMIT-*, U-QUERY-HAS-*, P-EMIT-*)
    //
    // Verify the per-kind index machinery: emit-mirrors-to-index,
    // query-uses-index, rollback-undoes-index, count-by-kind is O(1).
    // -------------------------------------------------------------------------

    use super::SemanticFactRecord;

    fn ident(s: &str) -> SemanticRuntimeValue {
        SemanticRuntimeValue::Identifier(s.to_string())
    }

    fn kind_arg(kind: &str) -> UnifiedSemanticValue {
        UnifiedSemanticValue::Identifier(kind.to_string())
    }

    fn name_arg(name: &str) -> UnifiedSemanticValue {
        UnifiedSemanticValue::Identifier(name.to_string())
    }

    fn has_fact_predicate(kind: &str, name: &str) -> SemanticPredicateSpec {
        SemanticPredicateSpec {
            name: "has_fact".to_string(),
            args: vec![kind_arg(kind), name_arg(name)],
            phase: SemanticPredicatePhase::Pre,
            view: SemanticPredicateContentView::Raw,
        }
    }

    fn count_at_least(kind: &str, n: usize) -> SemanticPredicateSpec {
        SemanticPredicateSpec {
            name: "fact_count_at_least".to_string(),
            args: vec![
                kind_arg(kind),
                UnifiedSemanticValue::Number(n.to_string()),
            ],
            phase: SemanticPredicatePhase::Pre,
            view: SemanticPredicateContentView::Raw,
        }
    }

    #[test]
    fn multi_index_u_emit_1_emit_then_has_fact_succeeds() {
        // U-EMIT-1: emit a fact; has_fact returns true immediately.
        let mut state = SemanticRuntimeState::new();
        state.emit_fact(SemanticFactSpec {
            kind: "variable_binding".to_string(),
            name: ident("seed_map"),
            attributes: vec![],
        });
        assert_eq!(
            state.evaluate_predicate(&has_fact_predicate("variable_binding", "seed_map")),
            Some(true)
        );
    }

    #[test]
    fn multi_index_u_query_has_2_nonexistent_returns_false() {
        // U-QUERY-HAS-2: missing fact returns false (NOT None — predicate is well-formed).
        let state = SemanticRuntimeState::new();
        assert_eq!(
            state.evaluate_predicate(&has_fact_predicate("variable_binding", "absent")),
            Some(false)
        );
    }

    #[test]
    fn multi_index_kind_is_case_insensitive() {
        // The legacy predicate semantics use `eq_ignore_ascii_case` on kind;
        // the index normalises kinds to lowercase on insert + on query.
        let mut state = SemanticRuntimeState::new();
        state.emit_fact(SemanticFactSpec {
            kind: "Variable_Binding".to_string(),
            name: ident("x"),
            attributes: vec![],
        });
        assert_eq!(
            state.evaluate_predicate(&has_fact_predicate("variable_binding", "x")),
            Some(true)
        );
        assert_eq!(
            state.evaluate_predicate(&has_fact_predicate("VARIABLE_BINDING", "x")),
            Some(true)
        );
    }

    #[test]
    fn multi_index_p_tx_1_rollback_leaves_no_trace() {
        // P-TX-1 (THE KEY INVARIANT): after rollback the store is byte-identical
        // to its pre-tx state. Verified by checking the predicate result + the
        // fact-count-by-kind shortcut return to baseline.
        let mut state = SemanticRuntimeState::new();
        state.emit_fact(SemanticFactSpec {
            kind: "variable_binding".to_string(),
            name: ident("pre_tx"),
            attributes: vec![],
        });
        let checkpoint = state.checkpoint();
        // Emit several facts inside the speculative tx.
        for i in 0..5 {
            state.emit_fact(SemanticFactSpec {
                kind: "variable_binding".to_string(),
                name: ident(&format!("speculative_{}", i)),
                attributes: vec![],
            });
        }
        // Tx-inserted facts are visible mid-tx.
        assert_eq!(
            state.evaluate_predicate(&has_fact_predicate("variable_binding", "speculative_3")),
            Some(true)
        );
        assert_eq!(state.fact_index.count_for_kind("variable_binding"), 6);
        // Rollback.
        state.rollback_to(checkpoint);
        // Tx-inserted facts are GONE from both the master Vec and the index.
        assert_eq!(state.facts().len(), 1);
        assert_eq!(state.fact_index.count_for_kind("variable_binding"), 1);
        assert_eq!(
            state.evaluate_predicate(&has_fact_predicate("variable_binding", "speculative_3")),
            Some(false)
        );
        // The pre-tx fact is still present.
        assert_eq!(
            state.evaluate_predicate(&has_fact_predicate("variable_binding", "pre_tx")),
            Some(true)
        );
    }

    #[test]
    fn multi_index_fact_count_at_least_is_o1_via_counter() {
        // fact_count_at_least previously linear-scanned the whole Vec; now
        // backed by the per-kind counter. Verify behaviour preserved across
        // edge cases.
        let mut state = SemanticRuntimeState::new();
        assert_eq!(
            state.evaluate_predicate(&count_at_least("variable_binding", 0)),
            Some(true)
        );
        assert_eq!(
            state.evaluate_predicate(&count_at_least("variable_binding", 1)),
            Some(false)
        );
        for i in 0..3 {
            state.emit_fact(SemanticFactSpec {
                kind: "variable_binding".to_string(),
                name: ident(&format!("v{}", i)),
                attributes: vec![],
            });
        }
        assert_eq!(
            state.evaluate_predicate(&count_at_least("variable_binding", 3)),
            Some(true)
        );
        assert_eq!(
            state.evaluate_predicate(&count_at_least("variable_binding", 4)),
            Some(false)
        );
        // Different kind is independent.
        assert_eq!(
            state.evaluate_predicate(&count_at_least("type_binding", 1)),
            Some(false)
        );
    }

    #[test]
    fn multi_index_p_emit_2_inserts_into_every_index_path() {
        // P-EMIT-2 (simplified): for each emit, both the master Vec AND the
        // per-kind index see the new entry. Verified via the public
        // `facts()` slice (Vec) + the private counter (index).
        let mut state = SemanticRuntimeState::new();
        for i in 0..10 {
            state.emit_fact(SemanticFactSpec {
                kind: "type_binding".to_string(),
                name: ident(&format!("T{}", i)),
                attributes: vec![],
            });
        }
        assert_eq!(state.facts().len(), 10);
        assert_eq!(state.fact_index.count_for_kind("type_binding"), 10);
    }

    #[test]
    fn multi_index_push_fact_record_also_indexes() {
        // The library-import path uses `push_fact_record` (not `emit_fact`);
        // it must ALSO maintain the index so imported facts are queryable.
        let mut state = SemanticRuntimeState::new();
        state.push_fact_record(SemanticFactRecord {
            kind: "type_binding".to_string(),
            name: ident("ImportedT"),
            scope_depth: 0,
            scope_id: super::ScopeId::ROOT,
            attributes: vec![],
        });
        assert_eq!(
            state.evaluate_predicate(&has_fact_predicate("type_binding", "ImportedT")),
            Some(true)
        );
        assert_eq!(state.fact_index.count_for_kind("type_binding"), 1);
    }

    #[test]
    fn multi_index_duplicate_emits_both_tracked() {
        // Today's semantics allow duplicate-in-scope emits (the U-EMIT-7 [TBC]
        // policy stays "allow" in this sub-leaf). The counter tracks both;
        // has_fact returns true; rollback restores correctly.
        let mut state = SemanticRuntimeState::new();
        for _ in 0..3 {
            state.emit_fact(SemanticFactSpec {
                kind: "variable_binding".to_string(),
                name: ident("dup"),
                attributes: vec![],
            });
        }
        assert_eq!(state.fact_index.count_for_kind("variable_binding"), 3);
        assert_eq!(
            state.evaluate_predicate(&has_fact_predicate("variable_binding", "dup")),
            Some(true)
        );
    }

    // -------------------------------------------------------------------------
    // `.3.3.4.b.5.1.2` @fact_kind: declaration tests
    //
    // Cover the parse path + V-DECL-1..7 validation rules. Test names follow
    // the U-DECL-N / V-DECL-N convention from PGEN_SEMANTIC_STORE_TEST_PLAN.md.
    // -------------------------------------------------------------------------

    use super::{FactKindDecl, parse_fact_kind};
    use crate::ast_pipeline::UnifiedSemanticProperty;

    /// Build a `UnifiedSemanticValue::Object` from `(key, value)` pairs.
    fn object(pairs: Vec<(&str, UnifiedSemanticValue)>) -> UnifiedSemanticValue {
        UnifiedSemanticValue::Object(
            pairs
                .into_iter()
                .map(|(k, v)| UnifiedSemanticProperty {
                    key: k.to_string(),
                    value: v,
                })
                .collect(),
        )
    }

    fn ident_val(s: &str) -> UnifiedSemanticValue {
        UnifiedSemanticValue::Identifier(s.to_string())
    }

    fn string_val(s: &str) -> UnifiedSemanticValue {
        UnifiedSemanticValue::String(s.to_string())
    }

    fn arr(items: Vec<UnifiedSemanticValue>) -> UnifiedSemanticValue {
        UnifiedSemanticValue::Array(items)
    }

    fn ident_list(items: &[&str]) -> UnifiedSemanticValue {
        arr(items.iter().map(|s| ident_val(s)).collect())
    }

    fn bool_val(b: bool) -> UnifiedSemanticValue {
        UnifiedSemanticValue::Boolean(b)
    }

    /// Wrap an object payload in the `UnifiedSemanticAST::Structured` shape
    /// and feed it through `parse_fact_kind`.
    fn parse_fact_kind_payload(payload: UnifiedSemanticValue) -> Result<FactKindDecl, String> {
        let ast = UnifiedSemanticAST::Structured {
            canonical: String::new(),
            value: payload,
        };
        match parse_fact_kind(&ast)? {
            SemanticRuntimeDirective::DeclareFactKind(decl) => Ok(decl),
            other => Err(format!("expected DeclareFactKind, got {:?}", other)),
        }
    }

    fn fact_kind_annotation(payload: UnifiedSemanticValue) -> SemanticAnnotation {
        SemanticAnnotation::Named {
            name: "fact_kind".to_string(),
            ast: UnifiedSemanticAST::Structured {
                canonical: String::new(),
                value: payload,
            },
        }
    }

    #[test]
    fn u_decl_1_minimal_well_formed_declaration_parses() {
        // U-DECL-1: a well-formed declaration returns Ok with the parsed payload.
        let decl = parse_fact_kind_payload(object(vec![
            ("name", ident_val("variable_binding")),
            ("attributes", ident_list(&["name", "type_kind"])),
        ]))
        .expect("well-formed @fact_kind: should parse");
        assert_eq!(decl.name, "variable_binding");
        assert_eq!(decl.attributes, vec!["name", "type_kind"]);
        assert_eq!(decl.required, Vec::<String>::new());
        assert!(decl.indexes.is_empty());
        assert!(!decl.exportable);
        assert!(decl.description.is_none());
    }

    #[test]
    fn u_decl_full_declaration_parses_all_fields() {
        let decl = parse_fact_kind_payload(object(vec![
            ("name", ident_val("variable_binding")),
            ("attributes", ident_list(&["name", "type_kind", "type_ref"])),
            ("required", ident_list(&["name", "type_kind"])),
            (
                "indexes",
                arr(vec![ident_list(&["scope", "name"]), ident_list(&["scope", "type_kind"])]),
            ),
            ("scope_kind", ident_val("enclosing_block")),
            ("exportable", bool_val(true)),
            ("artefact_kind", ident_val("bindings")),
            ("description", string_val("A bound identifier with its declared type.")),
        ]))
        .expect("full @fact_kind: should parse");
        assert_eq!(decl.name, "variable_binding");
        assert_eq!(decl.attributes, vec!["name", "type_kind", "type_ref"]);
        assert_eq!(decl.required, vec!["name", "type_kind"]);
        assert_eq!(
            decl.indexes,
            vec![
                vec!["scope".to_string(), "name".to_string()],
                vec!["scope".to_string(), "type_kind".to_string()],
            ]
        );
        assert_eq!(decl.scope_kind.as_deref(), Some("enclosing_block"));
        assert!(decl.exportable);
        assert_eq!(decl.artefact_kind.as_deref(), Some("bindings"));
        assert_eq!(decl.resolved_artefact_kind(), "bindings");
        assert_eq!(
            decl.description.as_deref(),
            Some("A bound identifier with its declared type.")
        );
    }

    #[test]
    fn u_decl_resolved_artefact_kind_defaults_to_name() {
        let decl = parse_fact_kind_payload(object(vec![
            ("name", ident_val("foo")),
            ("attributes", ident_list(&["x"])),
            ("exportable", bool_val(true)),
        ]))
        .expect("parse");
        assert_eq!(decl.resolved_artefact_kind(), "foo");
    }

    #[test]
    fn v_decl_2_empty_attributes_rejected() {
        // V-DECL-2: attributes must be non-empty.
        let err = parse_fact_kind_payload(object(vec![
            ("name", ident_val("foo")),
            ("attributes", arr(vec![])),
        ]))
        .expect_err("V-DECL-2 should reject");
        assert!(err.contains("V-DECL-2"), "got: {}", err);
    }

    #[test]
    fn v_decl_3_required_not_in_attributes_rejected() {
        // V-DECL-3: every name in `required` must appear in `attributes`.
        let err = parse_fact_kind_payload(object(vec![
            ("name", ident_val("foo")),
            ("attributes", ident_list(&["name"])),
            ("required", ident_list(&["type_kind"])),
        ]))
        .expect_err("V-DECL-3 should reject");
        assert!(err.contains("V-DECL-3"), "got: {}", err);
        assert!(err.contains("type_kind"), "got: {}", err);
    }

    #[test]
    fn v_decl_4_index_attr_not_in_attributes_rejected() {
        // V-DECL-4: every name in an index tuple must be in `attributes` ∪ {scope, kind}.
        let err = parse_fact_kind_payload(object(vec![
            ("name", ident_val("foo")),
            ("attributes", ident_list(&["name"])),
            ("indexes", arr(vec![ident_list(&["scope", "missing_attr"])])),
        ]))
        .expect_err("V-DECL-4 should reject");
        assert!(err.contains("V-DECL-4"), "got: {}", err);
        assert!(err.contains("missing_attr"), "got: {}", err);
    }

    #[test]
    fn v_decl_4_scope_and_kind_are_implicit() {
        // V-DECL-4 carve-out: 'scope' and 'kind' are always indexable.
        let _decl = parse_fact_kind_payload(object(vec![
            ("name", ident_val("foo")),
            ("attributes", ident_list(&["name"])),
            ("indexes", arr(vec![ident_list(&["scope", "kind", "name"])])),
        ]))
        .expect("scope+kind should be accepted as implicit attributes");
    }

    #[test]
    fn v_decl_5_empty_index_tuple_rejected() {
        // V-DECL-5: index tuple non-empty.
        let err = parse_fact_kind_payload(object(vec![
            ("name", ident_val("foo")),
            ("attributes", ident_list(&["name"])),
            ("indexes", arr(vec![arr(vec![])])),
        ]))
        .expect_err("V-DECL-5 (empty tuple) should reject");
        assert!(err.contains("V-DECL-5"), "got: {}", err);
    }

    #[test]
    fn v_decl_5_duplicate_in_tuple_rejected() {
        // V-DECL-5: index tuple no duplicates.
        let err = parse_fact_kind_payload(object(vec![
            ("name", ident_val("foo")),
            ("attributes", ident_list(&["name"])),
            ("indexes", arr(vec![ident_list(&["name", "name"])])),
        ]))
        .expect_err("V-DECL-5 (duplicate) should reject");
        assert!(err.contains("V-DECL-5"), "got: {}", err);
    }

    #[test]
    fn v_decl_7_name_path_components_validated() {
        // V-DECL-7: name with slash rejected.
        let err = parse_fact_kind_payload(object(vec![
            ("name", string_val("foo/bar")),
            ("attributes", ident_list(&["x"])),
        ]))
        .expect_err("V-DECL-7 should reject slash");
        assert!(err.contains("V-DECL-7"), "got: {}", err);

        // V-DECL-7: artefact_kind with leading dot rejected.
        let err = parse_fact_kind_payload(object(vec![
            ("name", ident_val("foo")),
            ("attributes", ident_list(&["x"])),
            ("artefact_kind", string_val(".hidden")),
        ]))
        .expect_err("V-DECL-7 should reject leading dot");
        assert!(err.contains("V-DECL-7"), "got: {}", err);
        assert!(err.contains("artefact_kind"), "got: {}", err);
    }

    #[test]
    fn unknown_field_rejected() {
        // Unknown fields are codegen errors (typo protection).
        let err = parse_fact_kind_payload(object(vec![
            ("name", ident_val("foo")),
            ("attributes", ident_list(&["x"])),
            ("gibberish", UnifiedSemanticValue::Number("1".to_string())),
        ]))
        .expect_err("unknown field should reject");
        assert!(err.contains("gibberish"), "got: {}", err);
    }

    #[test]
    fn missing_attributes_field_rejected() {
        // Surface-level required-field check.
        let err = parse_fact_kind_payload(object(vec![("name", ident_val("foo"))]))
            .expect_err("missing attributes should reject");
        assert!(err.contains("attributes"), "got: {}", err);
    }

    #[test]
    fn missing_name_field_rejected() {
        let err = parse_fact_kind_payload(object(vec![("attributes", ident_list(&["x"]))]))
            .expect_err("missing name should reject");
        assert!(err.contains("name"), "got: {}", err);
    }

    #[test]
    fn v_decl_1_duplicate_declaration_with_conflicting_payloads_rejected() {
        // V-DECL-1 cross-declaration uniqueness check via compile_semantic_runtime_annotations.
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "rule_alpha".to_string(),
            vec![fact_kind_annotation(object(vec![
                ("name", ident_val("foo")),
                ("attributes", ident_list(&["a"])),
            ]))],
        );
        annotations.semantic_annotations.insert(
            "rule_beta".to_string(),
            vec![fact_kind_annotation(object(vec![
                ("name", ident_val("foo")),
                ("attributes", ident_list(&["b"])),
            ]))],
        );
        let err = compile_semantic_runtime_annotations(&annotations)
            .expect_err("V-DECL-1 should reject duplicate kind with conflicting payloads");
        assert!(err.contains("V-DECL-1"), "got: {}", err);
    }

    #[test]
    fn v_decl_1_identical_redeclaration_allowed() {
        // V-DECL-1 deliberate carve-out: identical re-declarations (same payload)
        // are allowed for the convenience of attaching the same block to multiple rules.
        let mut annotations = Annotations::default();
        let payload = || {
            object(vec![
                ("name", ident_val("foo")),
                ("attributes", ident_list(&["a", "b"])),
            ])
        };
        annotations
            .semantic_annotations
            .insert("rule_alpha".to_string(), vec![fact_kind_annotation(payload())]);
        annotations
            .semantic_annotations
            .insert("rule_beta".to_string(), vec![fact_kind_annotation(payload())]);
        let compiled = compile_semantic_runtime_annotations(&annotations)
            .expect("identical re-declarations should be OK");
        assert_eq!(compiled.fact_kinds_len(), 1);
        let decl = compiled.fact_kind("foo").expect("registered");
        assert_eq!(decl.attributes, vec!["a", "b"]);
    }

    // -------------------------------------------------------------------------
    // `.3.3.4.b.5.1.3` scope-tree tests
    //
    // Verify the scope arena + active_chain + tree-walking accessors:
    // root invariant, open/close propagation, closed-scope survival,
    // rollback truncation, scope_id on emitted facts.
    // -------------------------------------------------------------------------

    use super::{SemanticCloseScopeSpec, SemanticScopeSpec};

    #[test]
    fn scope_tree_root_invariants() {
        // The root scope is always present with id 0 and depth 0; it is
        // never closed by the engine.
        let state = SemanticRuntimeState::new();
        assert_eq!(state.scope_arena().len(), 1);
        let root = state.scope_node(super::ScopeId::ROOT).expect("root present");
        assert_eq!(root.id, super::ScopeId::ROOT);
        assert!(root.parent.is_none());
        assert_eq!(root.depth_when_opened, 0);
        assert!(!root.closed);
        assert_eq!(state.current_scope_id(), super::ScopeId::ROOT);
        assert_eq!(state.active_chain(), &[super::ScopeId::ROOT]);
    }

    #[test]
    fn scope_tree_open_pushes_node_with_parent() {
        // Open a class scope; arena grows; new node parent = root; active chain extends.
        let mut state = SemanticRuntimeState::new();
        state.open_scope(SemanticScopeSpec {
            kind: SemanticScopeKind::Class,
            name: Some(ident("Foo")),
        });
        assert_eq!(state.scope_arena().len(), 2);
        let class_id = state.current_scope_id();
        assert_eq!(class_id, super::ScopeId(1));
        let class_node = state.scope_node(class_id).expect("class node");
        assert_eq!(class_node.parent, Some(super::ScopeId::ROOT));
        assert_eq!(class_node.kind, SemanticScopeKind::Class);
        assert_eq!(class_node.name, Some(ident("Foo")));
        assert_eq!(class_node.depth_when_opened, 1);
        assert!(!class_node.closed);
        assert_eq!(
            state.active_chain(),
            &[super::ScopeId::ROOT, super::ScopeId(1)]
        );
    }

    #[test]
    fn scope_tree_close_marks_node_closed_but_keeps_it_in_arena() {
        // After close_scope, the node is still in the arena (queryable for
        // archived lookups) but marked closed = true; active_chain pops it.
        let mut state = SemanticRuntimeState::new();
        state.open_scope(SemanticScopeSpec {
            kind: SemanticScopeKind::Class,
            name: Some(ident("Foo")),
        });
        let class_id = state.current_scope_id();
        let closed = state.close_scope(&SemanticCloseScopeSpec { kind: None, name: None });
        assert!(closed, "close_scope should match");
        // Arena length is unchanged: closed scopes stay.
        assert_eq!(state.scope_arena().len(), 2);
        // Node is marked closed.
        let class_node = state.scope_node(class_id).expect("class node still present");
        assert!(class_node.closed);
        // Active chain is back to just the root.
        assert_eq!(state.active_chain(), &[super::ScopeId::ROOT]);
        assert_eq!(state.current_scope_id(), super::ScopeId::ROOT);
    }

    #[test]
    fn scope_tree_nested_scopes_form_a_proper_tree() {
        // Open class -> function -> block; each step's node parent points
        // back one level. After closing function, the block (already closed
        // by then) survives in the arena.
        let mut state = SemanticRuntimeState::new();
        state.open_scope(SemanticScopeSpec {
            kind: SemanticScopeKind::Class,
            name: Some(ident("Foo")),
        });
        state.open_scope(SemanticScopeSpec {
            kind: SemanticScopeKind::Function,
            name: Some(ident("bar")),
        });
        state.open_scope(SemanticScopeSpec {
            kind: SemanticScopeKind::Block,
            name: None,
        });
        let block_id = state.current_scope_id();
        let function_id = state.scope_arena()[2].id;
        let class_id = state.scope_arena()[1].id;
        assert_eq!(state.scope_arena().len(), 4);
        assert_eq!(state.scope_node(block_id).unwrap().parent, Some(function_id));
        assert_eq!(state.scope_node(function_id).unwrap().parent, Some(class_id));
        assert_eq!(state.scope_node(class_id).unwrap().parent, Some(super::ScopeId::ROOT));
        // Children scan: root has one direct child (the class).
        let root_children: Vec<_> = state.scope_children(super::ScopeId::ROOT).collect();
        assert_eq!(root_children.len(), 1);
        assert_eq!(root_children[0].id, class_id);
        // Class has one direct child (function).
        let class_children: Vec<_> = state.scope_children(class_id).collect();
        assert_eq!(class_children.len(), 1);
        assert_eq!(class_children[0].id, function_id);
    }

    #[test]
    fn scope_tree_rollback_truncates_arena_and_restores_open_state() {
        // P-TX-1 extension: rollback restores arena to checkpoint state AND
        // re-opens any scope that was speculatively closed inside the tx.
        let mut state = SemanticRuntimeState::new();
        state.open_scope(SemanticScopeSpec {
            kind: SemanticScopeKind::Class,
            name: Some(ident("Foo")),
        });
        let pre_tx_class_id = state.current_scope_id();
        let checkpoint = state.checkpoint();
        // Speculate: open and close several nested scopes.
        state.open_scope(SemanticScopeSpec {
            kind: SemanticScopeKind::Function,
            name: Some(ident("bar")),
        });
        state.open_scope(SemanticScopeSpec {
            kind: SemanticScopeKind::Block,
            name: None,
        });
        // Close the inner two.
        state.close_scope(&SemanticCloseScopeSpec { kind: None, name: None });
        state.close_scope(&SemanticCloseScopeSpec { kind: None, name: None });
        // The class scope is the only one ALSO open at checkpoint time, so
        // close it speculatively too — that's the "premature close" case
        // rollback must reverse.
        state.close_scope(&SemanticCloseScopeSpec { kind: None, name: None });
        // Roll back.
        state.rollback_to(checkpoint);
        // Arena truncated back to (root, class).
        assert_eq!(state.scope_arena().len(), 2);
        // Class scope is OPEN again — its `closed` flag was reset by rollback.
        let class_node = state.scope_node(pre_tx_class_id).expect("class survives rollback");
        assert!(!class_node.closed, "class scope should be re-opened by rollback");
        // Active chain restored: root + class.
        assert_eq!(state.active_chain(), &[super::ScopeId::ROOT, pre_tx_class_id]);
        assert_eq!(state.current_scope_id(), pre_tx_class_id);
    }

    #[test]
    fn scope_tree_emitted_fact_carries_scope_id() {
        // Emitting a fact while inside a class scope records the class's
        // scope_id on the fact record.
        let mut state = SemanticRuntimeState::new();
        state.open_scope(SemanticScopeSpec {
            kind: SemanticScopeKind::Class,
            name: Some(ident("Foo")),
        });
        let class_id = state.current_scope_id();
        state.emit_fact(SemanticFactSpec {
            kind: "variable_binding".to_string(),
            name: ident("local_var"),
            attributes: vec![],
        });
        assert_eq!(state.facts().len(), 1);
        let fact = &state.facts()[0];
        assert_eq!(fact.scope_id, class_id);
        // Legacy scope_depth is also correct.
        assert_eq!(fact.scope_depth, 1);
    }

    // -------------------------------------------------------------------------
    // `.3.3.4.b.5.1.4` resolve_path tests
    //
    // Cover the dotted-path walk through the scope tree: single-segment
    // active-chain lookups, multi-segment via type_ref traversal, partial
    // resolution with `Unresolved { resolved_prefix }`, error cases, and
    // the built-in predicate wrapping.
    // -------------------------------------------------------------------------

    use super::{ResolveResult, attribute_text};

    fn type_ref_attr(scope_name: &str) -> UnifiedSemanticProperty {
        UnifiedSemanticProperty {
            key: "type_ref".to_string(),
            value: UnifiedSemanticValue::Identifier(scope_name.to_string()),
        }
    }

    fn type_kind_attr(kind: &str) -> UnifiedSemanticProperty {
        UnifiedSemanticProperty {
            key: "type_kind".to_string(),
            value: UnifiedSemanticValue::Identifier(kind.to_string()),
        }
    }

    fn emit_with_attrs(
        state: &mut SemanticRuntimeState,
        kind: &str,
        name: &str,
        attrs: Vec<UnifiedSemanticProperty>,
    ) {
        state.emit_fact(SemanticFactSpec {
            kind: kind.to_string(),
            name: ident(name),
            attributes: attrs,
        });
    }

    #[test]
    fn resolve_path_single_segment_in_current_scope() {
        // Resolve "x" → finds the variable_binding emitted in the active
        // (root) scope. Resolved variant carries the fact's data.
        let mut state = SemanticRuntimeState::new();
        emit_with_attrs(
            &mut state,
            "variable_binding",
            "x",
            vec![type_kind_attr("int")],
        );
        let result = state.resolve_path("x");
        match result {
            ResolveResult::Resolved { kind, name, scope_id, attributes } => {
                assert_eq!(kind, "variable_binding");
                assert_eq!(name, ident("x"));
                assert_eq!(scope_id, super::ScopeId::ROOT);
                assert_eq!(attributes.len(), 1);
                assert_eq!(attributes[0].key, "type_kind");
            }
            other => panic!("expected Resolved, got {:?}", other),
        }
    }

    #[test]
    fn resolve_path_single_segment_walks_active_chain_outward() {
        // Emit fact in root scope; open a class scope; resolve "x" finds
        // the root scope's fact (visible from inner scope via outward walk).
        let mut state = SemanticRuntimeState::new();
        emit_with_attrs(&mut state, "variable_binding", "x", vec![]);
        state.open_scope(SemanticScopeSpec {
            kind: SemanticScopeKind::Class,
            name: Some(ident("Foo")),
        });
        let result = state.resolve_path("x");
        assert!(result.is_resolved(), "outer scope's x should be visible inside class");
    }

    #[test]
    fn resolve_path_missing_segment_returns_unresolved() {
        let state = SemanticRuntimeState::new();
        let result = state.resolve_path("ghost");
        match result {
            ResolveResult::Unresolved { resolved_prefix, last_kind } => {
                assert!(resolved_prefix.is_empty());
                assert!(last_kind.is_none());
            }
            other => panic!("expected Unresolved, got {:?}", other),
        }
    }

    #[test]
    fn resolve_path_multi_segment_via_type_ref() {
        // Walk seed_map.seed_table.exists:
        //   - seed_map is a class instance of Container (in root scope).
        //   - Container's scope contains seed_table (an array).
        //   - But array doesn't have methods named "exists" as facts here;
        //     we'll test 2-segment first.
        let mut state = SemanticRuntimeState::new();
        // Container is a class scope with a single member "field_x".
        state.open_scope(SemanticScopeSpec {
            kind: SemanticScopeKind::Class,
            name: Some(ident("Container")),
        });
        emit_with_attrs(
            &mut state,
            "class_member",
            "field_x",
            vec![type_kind_attr("int")],
        );
        let close_spec = super::SemanticCloseScopeSpec { kind: None, name: None };
        state.close_scope(&close_spec);
        // In root scope, declare seed_map of type Container.
        emit_with_attrs(
            &mut state,
            "variable_binding",
            "seed_map",
            vec![type_kind_attr("class"), type_ref_attr("Container")],
        );
        // Resolve seed_map.field_x.
        let result = state.resolve_path("seed_map.field_x");
        match result {
            ResolveResult::Resolved { kind, name, .. } => {
                assert_eq!(kind, "class_member");
                assert_eq!(name, ident("field_x"));
            }
            other => panic!("expected Resolved, got {:?}", other),
        }
    }

    #[test]
    fn resolve_path_partial_returns_unresolved_with_prefix() {
        // seed_map resolves (root-scope variable_binding) but seed_map's
        // type_ref points to a scope that doesn't contain "nonexistent".
        let mut state = SemanticRuntimeState::new();
        state.open_scope(SemanticScopeSpec {
            kind: SemanticScopeKind::Class,
            name: Some(ident("Container")),
        });
        let close_spec = super::SemanticCloseScopeSpec { kind: None, name: None };
        state.close_scope(&close_spec);
        emit_with_attrs(
            &mut state,
            "variable_binding",
            "seed_map",
            vec![type_ref_attr("Container")],
        );
        let result = state.resolve_path("seed_map.nonexistent");
        match result {
            ResolveResult::Unresolved { resolved_prefix, last_kind } => {
                assert_eq!(resolved_prefix, vec!["seed_map".to_string()]);
                assert_eq!(last_kind.as_deref(), Some("variable_binding"));
            }
            other => panic!("expected Unresolved with prefix, got {:?}", other),
        }
    }

    #[test]
    fn resolve_path_missing_type_ref_blocks_descent() {
        // seed_map exists but has no type_ref attribute; the second
        // segment can't be walked.
        let mut state = SemanticRuntimeState::new();
        emit_with_attrs(&mut state, "variable_binding", "seed_map", vec![]);
        let result = state.resolve_path("seed_map.anything");
        match result {
            ResolveResult::Unresolved { resolved_prefix, last_kind } => {
                assert_eq!(resolved_prefix, vec!["seed_map".to_string()]);
                assert_eq!(last_kind.as_deref(), Some("variable_binding"));
            }
            other => panic!("expected Unresolved, got {:?}", other),
        }
    }

    #[test]
    fn resolve_path_empty_path_and_empty_segments_rejected() {
        let state = SemanticRuntimeState::new();
        assert!(!state.resolve_path("").is_resolved());
        assert!(!state.resolve_path(".x").is_resolved());
        assert!(!state.resolve_path("x.").is_resolved());
        assert!(!state.resolve_path("x..y").is_resolved());
    }

    #[test]
    fn resolve_path_resolved_attribute_accessor() {
        // ResolveResult::attribute() drill-down used by composed predicates.
        let mut state = SemanticRuntimeState::new();
        emit_with_attrs(
            &mut state,
            "variable_binding",
            "x",
            vec![type_kind_attr("array")],
        );
        let result = state.resolve_path("x");
        let type_kind = result.attribute("type_kind");
        assert!(type_kind.is_some());
        match type_kind.unwrap() {
            UnifiedSemanticValue::Identifier(s) => assert_eq!(s, "array"),
            other => panic!("expected Identifier, got {:?}", other),
        }
        // Missing attribute returns None.
        assert!(result.attribute("nonexistent").is_none());
    }

    #[test]
    fn resolve_path_as_built_in_predicate_returns_bool() {
        // The "resolve_path" predicate name returns Some(true) on Resolved,
        // Some(false) on Unresolved.
        let mut state = SemanticRuntimeState::new();
        emit_with_attrs(&mut state, "variable_binding", "x", vec![]);

        let pred_resolved = SemanticPredicateSpec {
            name: "resolve_path".to_string(),
            args: vec![UnifiedSemanticValue::Identifier("x".to_string())],
            phase: SemanticPredicatePhase::Pre,
            view: SemanticPredicateContentView::Raw,
        };
        let pred_unresolved = SemanticPredicateSpec {
            name: "resolve_path".to_string(),
            args: vec![UnifiedSemanticValue::Identifier("ghost".to_string())],
            phase: SemanticPredicatePhase::Pre,
            view: SemanticPredicateContentView::Raw,
        };

        assert_eq!(state.evaluate_predicate(&pred_resolved), Some(true));
        assert_eq!(state.evaluate_predicate(&pred_unresolved), Some(false));
    }

    #[test]
    fn attribute_text_helper_handles_case_insensitive_keys() {
        // attribute_text() uses eq_ignore_ascii_case so grammar authors can
        // write attribute keys in any case at the use site.
        let fact = SemanticFactRecord {
            kind: "x".to_string(),
            name: ident("y"),
            scope_depth: 0,
            scope_id: super::ScopeId::ROOT,
            attributes: vec![type_kind_attr("class")],
        };
        assert_eq!(attribute_text(&fact, "type_kind"), Some("class".to_string()));
        assert_eq!(attribute_text(&fact, "TYPE_KIND"), Some("class".to_string()));
        assert_eq!(attribute_text(&fact, "nonexistent"), None);
    }

    #[test]
    fn scope_tree_imported_fact_rebases_scope_id() {
        // push_fact_record (the library-import path) rebases scope_id to
        // the importer's current scope, ignoring any scope_id carried on
        // the artefact.
        let mut state = SemanticRuntimeState::new();
        state.open_scope(SemanticScopeSpec {
            kind: SemanticScopeKind::Package,
            name: Some(ident("p")),
        });
        let pkg_id = state.current_scope_id();
        state.push_fact_record(SemanticFactRecord {
            kind: "type_binding".to_string(),
            name: ident("ImportedType"),
            scope_depth: 0,                                // ignored on import
            scope_id: super::ScopeId(999),                 // ignored on import
            attributes: vec![],
        });
        let fact = &state.facts()[0];
        assert_eq!(fact.scope_id, pkg_id);
        assert_eq!(fact.scope_depth, 1);
    }

    // -------------------------------------------------------------------------
    // `.3.3.4.b.5.1.5` @predicate_def: composed-predicate tests
    //
    // parse_predicate_def + V-QDEF-1..5 + the evaluator
    // (evaluate_composed_predicate / eval_predicate_expr).
    // -------------------------------------------------------------------------

    use super::parse_predicate_def;
    use crate::ast_pipeline::PredicateDef;

    fn parse_predicate_def_payload(
        payload: UnifiedSemanticValue,
    ) -> Result<PredicateDef, String> {
        let ast = UnifiedSemanticAST::Structured {
            canonical: String::new(),
            value: payload,
        };
        match parse_predicate_def(&ast)? {
            SemanticRuntimeDirective::DefinePredicate(def) => Ok(def),
            other => Err(format!("expected DefinePredicate, got {:?}", other)),
        }
    }

    fn predicate_def_annotation(payload: UnifiedSemanticValue) -> SemanticAnnotation {
        SemanticAnnotation::Named {
            name: "predicate_def".to_string(),
            ast: UnifiedSemanticAST::Structured {
                canonical: String::new(),
                value: payload,
            },
        }
    }

    #[test]
    fn predicate_def_well_formed_parses() {
        let def = parse_predicate_def_payload(object(vec![
            ("name", ident_val("receiver_is_array")),
            ("args", ident_list(&["receiver_path"])),
            (
                "body",
                string_val(
                    "resolve_path($receiver_path).attribute('type_kind') in ['array','queue']",
                ),
            ),
        ]))
        .expect("well-formed @predicate_def: should parse");
        assert_eq!(def.name, "receiver_is_array");
        assert_eq!(def.args, vec!["receiver_path"]);
    }

    #[test]
    fn predicate_def_missing_name_rejected() {
        let err = parse_predicate_def_payload(object(vec![
            ("args", ident_list(&["p"])),
            ("body", string_val("has_fact(k, $p)")),
        ]))
        .expect_err("missing name should reject");
        assert!(err.contains("name"), "got: {}", err);
    }

    #[test]
    fn predicate_def_missing_body_rejected() {
        let err = parse_predicate_def_payload(object(vec![
            ("name", ident_val("foo")),
            ("args", ident_list(&["p"])),
        ]))
        .expect_err("missing body should reject");
        assert!(err.contains("body"), "got: {}", err);
    }

    #[test]
    fn predicate_def_unknown_field_rejected() {
        let err = parse_predicate_def_payload(object(vec![
            ("name", ident_val("foo")),
            ("args", ident_list(&["p"])),
            ("body", string_val("has_fact(k, $p)")),
            ("gibberish", bool_val(true)),
        ]))
        .expect_err("unknown field should reject");
        assert!(err.contains("gibberish"), "got: {}", err);
    }

    #[test]
    fn v_qdef_2_duplicate_args_rejected() {
        let err = parse_predicate_def_payload(object(vec![
            ("name", ident_val("foo")),
            ("args", ident_list(&["p", "p"])),
            ("body", string_val("has_fact(k, $p)")),
        ]))
        .expect_err("V-QDEF-2 should reject");
        assert!(err.contains("V-QDEF-2"), "got: {}", err);
    }

    #[test]
    fn v_qdef_3_unknown_primitive_rejected() {
        let err = parse_predicate_def_payload(object(vec![
            ("name", ident_val("foo")),
            ("args", ident_list(&["p"])),
            ("body", string_val("not_a_primitive($p)")),
        ]))
        .expect_err("V-QDEF-3 (unknown primitive) should reject");
        assert!(err.contains("V-QDEF-3"), "got: {}", err);
    }

    #[test]
    fn v_qdef_3_wrong_arity_rejected() {
        // has_fact takes 2 args; supply 1.
        let err = parse_predicate_def_payload(object(vec![
            ("name", ident_val("foo")),
            ("args", ident_list(&["p"])),
            ("body", string_val("has_fact($p)")),
        ]))
        .expect_err("V-QDEF-3 (wrong arity) should reject");
        assert!(err.contains("V-QDEF-3"), "got: {}", err);
    }

    #[test]
    fn v_qdef_4_unbound_arg_ref_rejected() {
        // body references $other but args only has $p.
        let err = parse_predicate_def_payload(object(vec![
            ("name", ident_val("foo")),
            ("args", ident_list(&["p"])),
            ("body", string_val("has_fact(k, $other)")),
        ]))
        .expect_err("V-QDEF-4 should reject");
        assert!(err.contains("V-QDEF-4"), "got: {}", err);
        assert!(err.contains("other"), "got: {}", err);
    }

    #[test]
    fn v_qdef_5_attribute_on_non_resolve_path_rejected() {
        // .attribute() only on resolve_path; has_fact.attribute(...) is bad.
        let err = parse_predicate_def_payload(object(vec![
            ("name", ident_val("foo")),
            ("args", ident_list(&["p"])),
            ("body", string_val("has_fact(k, $p).attribute('x') == 'y'")),
        ]))
        .expect_err("V-QDEF-5 should reject");
        assert!(err.contains("V-QDEF-5"), "got: {}", err);
    }

    #[test]
    fn v_qdef_1_shadowing_builtin_rejected() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "rule_a".to_string(),
            vec![predicate_def_annotation(object(vec![
                ("name", ident_val("has_fact")), // shadows a built-in
                ("args", ident_list(&["p"])),
                ("body", string_val("resolve_path($p)")),
            ]))],
        );
        let err = compile_semantic_runtime_annotations(&annotations)
            .expect_err("V-QDEF-1 (shadowing) should reject");
        assert!(err.contains("V-QDEF-1"), "got: {}", err);
        assert!(err.contains("shadows"), "got: {}", err);
    }

    #[test]
    fn v_qdef_1_conflicting_redeclaration_rejected() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "rule_a".to_string(),
            vec![predicate_def_annotation(object(vec![
                ("name", ident_val("foo")),
                ("args", ident_list(&["p"])),
                ("body", string_val("has_fact(ka, $p)")),
            ]))],
        );
        annotations.semantic_annotations.insert(
            "rule_b".to_string(),
            vec![predicate_def_annotation(object(vec![
                ("name", ident_val("foo")),
                ("args", ident_list(&["p"])),
                ("body", string_val("has_fact(kb, $p)")),
            ]))],
        );
        let err = compile_semantic_runtime_annotations(&annotations)
            .expect_err("V-QDEF-1 (conflicting redecl) should reject");
        assert!(err.contains("V-QDEF-1"), "got: {}", err);
    }

    #[test]
    fn predicate_def_registry_populated_and_identical_redecl_allowed() {
        let mut annotations = Annotations::default();
        let payload = || {
            object(vec![
                ("name", ident_val("receiver_is_array")),
                ("args", ident_list(&["p"])),
                (
                    "body",
                    string_val("resolve_path($p).attribute('type_kind') in ['array']"),
                ),
            ])
        };
        annotations
            .semantic_annotations
            .insert("rule_a".to_string(), vec![predicate_def_annotation(payload())]);
        annotations
            .semantic_annotations
            .insert("rule_b".to_string(), vec![predicate_def_annotation(payload())]);
        let compiled = compile_semantic_runtime_annotations(&annotations)
            .expect("identical re-decls should be OK");
        assert_eq!(compiled.predicate_defs_len(), 1);
        let def = compiled.predicate_def("receiver_is_array").expect("registered");
        assert_eq!(def.args, vec!["p"]);
    }

    #[test]
    fn evaluate_composed_predicate_receiver_is_array() {
        // The motivating end-to-end: a receiver_is_array composed predicate
        // evaluated against a state where `x` is an array and `y` is an int.
        let def = parse_predicate_def_payload(object(vec![
            ("name", ident_val("receiver_is_array")),
            ("args", ident_list(&["path"])),
            (
                "body",
                string_val(
                    "resolve_path($path).attribute('type_kind') in ['array','queue','dynamic_array']",
                ),
            ),
        ]))
        .expect("parse");

        let mut state = SemanticRuntimeState::new();
        state.emit_fact(SemanticFactSpec {
            kind: "variable_binding".to_string(),
            name: ident("x"),
            attributes: vec![UnifiedSemanticProperty {
                key: "type_kind".to_string(),
                value: UnifiedSemanticValue::Identifier("array".to_string()),
            }],
        });
        state.emit_fact(SemanticFactSpec {
            kind: "variable_binding".to_string(),
            name: ident("y"),
            attributes: vec![UnifiedSemanticProperty {
                key: "type_kind".to_string(),
                value: UnifiedSemanticValue::Identifier("int".to_string()),
            }],
        });

        // x is an array → predicate true.
        assert_eq!(
            state.evaluate_composed_predicate(&def, &["x".to_string()]),
            Some(true)
        );
        // y is an int → predicate false.
        assert_eq!(
            state.evaluate_composed_predicate(&def, &["y".to_string()]),
            Some(false)
        );
        // Unknown receiver → resolve_path Unresolved → attribute None →
        // membership cannot be evaluated → None (indeterminate).
        assert_eq!(
            state.evaluate_composed_predicate(&def, &["ghost".to_string()]),
            None
        );
    }

    #[test]
    fn evaluate_composed_predicate_arity_mismatch_returns_none() {
        let def = PredicateDef {
            name: "foo".to_string(),
            args: vec!["a".to_string(), "b".to_string()],
            body: crate::ast_pipeline::parse_predicate_expression("has_fact($a, $b)")
                .expect("parse body"),
        };
        let state = SemanticRuntimeState::new();
        // Call with 1 arg but def expects 2.
        assert_eq!(
            state.evaluate_composed_predicate(&def, &["only_one".to_string()]),
            None
        );
    }

    #[test]
    fn composed_predicate_dispatches_via_evaluate_predicate() {
        // `.b.5.1.5.c`: after set_predicate_defs, evaluate_predicate
        // dispatches a non-built-in predicate name to its @predicate_def:
        // body. This is the wiring that makes a runtime
        // `@predicate <user-defined-name>` call actually fire.
        let def = parse_predicate_def_payload(object(vec![
            ("name", ident_val("receiver_is_array")),
            ("args", ident_list(&["path"])),
            (
                "body",
                string_val("resolve_path($path).attribute('type_kind') in ['array']"),
            ),
        ]))
        .expect("parse");

        let mut state = SemanticRuntimeState::new();
        let mut defs = std::collections::HashMap::new();
        defs.insert("receiver_is_array".to_string(), def);
        state.set_predicate_defs(defs);

        state.emit_fact(SemanticFactSpec {
            kind: "variable_binding".to_string(),
            name: ident("arr"),
            attributes: vec![UnifiedSemanticProperty {
                key: "type_kind".to_string(),
                value: UnifiedSemanticValue::Identifier("array".to_string()),
            }],
        });

        // evaluate_predicate with the user-defined name dispatches to the
        // composed body — "arr" is an array → true.
        let spec = SemanticPredicateSpec {
            name: "receiver_is_array".to_string(),
            args: vec![UnifiedSemanticValue::Identifier("arr".to_string())],
            phase: SemanticPredicatePhase::Pre,
            view: SemanticPredicateContentView::Raw,
        };
        assert_eq!(state.evaluate_predicate(&spec), Some(true));

        // A non-built-in name with no registered @predicate_def → None
        // (indeterminate — unchanged from pre-.c behaviour).
        let unknown = SemanticPredicateSpec {
            name: "totally_unknown".to_string(),
            args: vec![],
            phase: SemanticPredicatePhase::Pre,
            view: SemanticPredicateContentView::Raw,
        };
        assert_eq!(state.evaluate_predicate(&unknown), None);

        // Without set_predicate_defs, even a known name is not dispatched.
        let bare_state = SemanticRuntimeState::new();
        assert_eq!(bare_state.evaluate_predicate(&spec), None);
    }

    #[test]
    fn eval_predicate_expr_boolean_combinators() {
        // && / || / ! over has_fact calls.
        let mut state = SemanticRuntimeState::new();
        state.emit_fact(SemanticFactSpec {
            kind: "k".to_string(),
            name: ident("present"),
            attributes: vec![],
        });
        let def = |body: &str| PredicateDef {
            name: "p".to_string(),
            args: vec![],
            body: crate::ast_pipeline::parse_predicate_expression(body).expect("parse"),
        };
        // has_fact(k, present) && !has_fact(k, absent)  → true && !false → true
        assert_eq!(
            state.evaluate_composed_predicate(
                &def("has_fact(k, present) && !has_fact(k, absent)"),
                &[]
            ),
            Some(true)
        );
        // has_fact(k, absent) || has_fact(k, present)  → false || true → true
        assert_eq!(
            state.evaluate_composed_predicate(
                &def("has_fact(k, absent) || has_fact(k, present)"),
                &[]
            ),
            Some(true)
        );
        // has_fact(k, absent) && has_fact(k, present)  → false && _ → false
        assert_eq!(
            state.evaluate_composed_predicate(
                &def("has_fact(k, absent) && has_fact(k, present)"),
                &[]
            ),
            Some(false)
        );
    }

    #[test]
    fn fact_kind_registry_populated_from_directives() {
        let mut annotations = Annotations::default();
        annotations.semantic_annotations.insert(
            "some_rule".to_string(),
            vec![fact_kind_annotation(object(vec![
                ("name", ident_val("variable_binding")),
                ("attributes", ident_list(&["name", "type_kind"])),
                ("required", ident_list(&["name"])),
                ("exportable", bool_val(true)),
            ]))],
        );
        let compiled = compile_semantic_runtime_annotations(&annotations).expect("compile");
        assert_eq!(compiled.fact_kinds_len(), 1);
        let decl = compiled.fact_kind("variable_binding").expect("registered");
        assert_eq!(decl.attributes, vec!["name", "type_kind"]);
        assert_eq!(decl.required, vec!["name"]);
        assert!(decl.exportable);
    }
}
