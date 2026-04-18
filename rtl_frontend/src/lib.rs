//! Initial synthesizable-RTL frontend baseline for planned RTLSyn work.
//!
//! The current scope is intentionally narrow:
//! - module headers with optional parameter and ANSI port lists
//! - parameter/localparam declarations
//! - packed ranges backed by `rtl_const_expr`
//! - net declarations, continuous assigns, and basic procedural blocks
//! - explicit `generate` regions with `if` / `for` constructs
//!
//! This is not a full SystemVerilog frontend. The goal is to establish the
//! frontend/elaboration boundary and wire constant-expression parsing into it.

use rtl_const_expr::{EvalError, Expr, parse_expression};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendError {
    pub message: String,
    pub position: usize,
}

impl FrontendError {
    fn new(message: impl Into<String>, position: usize) -> Self {
        Self {
            message: message.into(),
            position,
        }
    }
}

impl fmt::Display for FrontendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at byte {}", self.message, self.position)
    }
}

impl Error for FrontendError {}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Design {
    pub typedefs: Vec<TypedefDecl>,
    pub packages: Vec<PackageDecl>,
    pub modules: Vec<Module>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElaboratedModule {
    pub module_name: String,
    pub path: String,
    pub parameters: HashMap<String, i64>,
    pub child_instances: Vec<ElaboratedInstance>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElaboratedInstance {
    pub module_name: String,
    pub instance_name: String,
    pub path: String,
    pub parameters: HashMap<String, i64>,
    pub port_bindings: Vec<ResolvedPortBinding>,
    pub child_instances: Vec<ElaboratedInstance>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ElaborationConfig {
    max_depth: usize,
    max_generate_iterations: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ElaborationState {
    config: ElaborationConfig,
    package_envs: HashMap<String, HashMap<String, i64>>,
    package_symbols: HashMap<String, i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VisibleScope {
    names: HashSet<String>,
    types: HashMap<String, DeclType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeclType {
    data_type: DataType,
    unpacked_dimensions: usize,
}

impl VisibleScope {
    fn merged_with_items(&self, items: &[ModuleItem]) -> Self {
        Self {
            names: merge_visible_names(&self.names, &collect_visible_names_from_items(items)),
            types: merge_declared_types(&self.types, &collect_declared_types_from_items(items)),
        }
    }

    fn with_name(&self, name: &str) -> Self {
        let mut names = self.names.clone();
        names.insert(name.to_string());
        Self {
            names,
            types: self.types.clone(),
        }
    }
}

impl Design {
    pub fn module(&self, name: &str) -> Option<&Module> {
        self.modules.iter().find(|module| module.name == name)
    }

    pub fn package(&self, name: &str) -> Option<&PackageDecl> {
        self.packages.iter().find(|package| package.name == name)
    }

    pub fn elaborate_top(
        &self,
        module_name: &str,
        overrides: &HashMap<String, i64>,
    ) -> Result<ElaboratedModule, FrontendError> {
        self.elaborate_top_with_limits(module_name, overrides, 64, 64)
    }

    pub fn elaborate_top_with_limits(
        &self,
        module_name: &str,
        overrides: &HashMap<String, i64>,
        max_depth: usize,
        max_generate_iterations: usize,
    ) -> Result<ElaboratedModule, FrontendError> {
        let module = self.module(module_name).ok_or_else(|| {
            FrontendError::new(format!("unknown top module '{}'", module_name), 0)
        })?;
        let package_envs = self.evaluate_package_constant_envs()?;
        let package_symbols = flatten_package_constant_envs(&package_envs);
        let state = ElaborationState {
            config: ElaborationConfig {
                max_depth,
                max_generate_iterations,
            },
            package_envs,
            package_symbols,
        };
        let imported_constants =
            self.resolve_imported_constant_symbols(module.all_imports(), &state.package_envs)?;
        let full_symbols = module.evaluate_full_constant_env(
            &state.package_symbols,
            &imported_constants,
            overrides,
        )?;
        module.validate_header_types(&full_symbols)?;
        let parameters = module.extract_owned_constant_env(&full_symbols);
        let visible_scope = module.visible_connection_scope();
        let path = module.name.clone();
        let child_instances = self.elaborate_items(
            &module.items,
            &full_symbols,
            &visible_scope,
            &path,
            0,
            &state,
        )?;
        Ok(ElaboratedModule {
            module_name: module.name.clone(),
            path,
            parameters,
            child_instances,
        })
    }

    fn elaborate_items(
        &self,
        items: &[ModuleItem],
        symbols: &HashMap<String, i64>,
        visible_scope: &VisibleScope,
        scope_path: &str,
        depth: usize,
        state: &ElaborationState,
    ) -> Result<Vec<ElaboratedInstance>, FrontendError> {
        let mut instances = Vec::new();

        for item in items {
            validate_module_item_types(item, visible_scope, symbols)?;
            match item {
                ModuleItem::ModuleInstantiation(instantiation) => {
                    instances.extend(self.elaborate_instance(
                        instantiation,
                        symbols,
                        visible_scope,
                        scope_path,
                        depth,
                        state,
                    )?);
                }
                ModuleItem::GenerateRegion(region) => {
                    let nested_visible_scope = visible_scope.merged_with_items(&region.items);
                    instances.extend(self.elaborate_items(
                        &region.items,
                        symbols,
                        &nested_visible_scope,
                        scope_path,
                        depth,
                        state,
                    )?);
                }
                ModuleItem::GenerateIf(generate_if) => {
                    let enabled = generate_if.is_enabled(symbols)?;
                    let (label, body_items, fallback_scope) = if enabled {
                        (
                            generate_if.then_label.as_deref(),
                            generate_if.then_items.as_slice(),
                            "__gen_if_true",
                        )
                    } else {
                        (
                            generate_if.else_label.as_deref(),
                            generate_if.else_items.as_slice(),
                            "__gen_if_false",
                        )
                    };

                    if !body_items.is_empty() {
                        let nested_visible_scope = visible_scope.merged_with_items(body_items);
                        let nested_scope = join_path(scope_path, label.unwrap_or(fallback_scope));
                        instances.extend(self.elaborate_items(
                            body_items,
                            symbols,
                            &nested_visible_scope,
                            &nested_scope,
                            depth,
                            state,
                        )?);
                    }
                }
                ModuleItem::GenerateFor(generate_for) => {
                    for iteration in generate_for
                        .iteration_values(symbols, state.config.max_generate_iterations)?
                    {
                        let mut nested_symbols = symbols.clone();
                        nested_symbols.insert(generate_for.index_name.clone(), iteration);
                        let nested_visible_scope = visible_scope
                            .merged_with_items(&generate_for.body_items)
                            .with_name(&generate_for.index_name);
                        let scope_name = match &generate_for.label {
                            Some(label) => format!("{label}[{iteration}]"),
                            None => format!("__gen_for_{}[{iteration}]", generate_for.index_name),
                        };
                        let nested_scope = join_path(scope_path, &scope_name);
                        instances.extend(self.elaborate_items(
                            &generate_for.body_items,
                            &nested_symbols,
                            &nested_visible_scope,
                            &nested_scope,
                            depth,
                            state,
                        )?);
                    }
                }
                ModuleItem::ParameterDecl(_)
                | ModuleItem::TypedefDecl(_)
                | ModuleItem::ImportDecl(_)
                | ModuleItem::GenvarDecl(_)
                | ModuleItem::NetDecl(_)
                | ModuleItem::ContinuousAssign(_)
                | ModuleItem::ProceduralBlock(_) => {}
            }
        }

        Ok(instances)
    }

    fn elaborate_instance(
        &self,
        instantiation: &ModuleInstantiation,
        parent_symbols: &HashMap<String, i64>,
        parent_scope: &VisibleScope,
        scope_path: &str,
        depth: usize,
        state: &ElaborationState,
    ) -> Result<Vec<ElaboratedInstance>, FrontendError> {
        if depth >= state.config.max_depth {
            return Err(FrontendError::new(
                format!(
                    "elaboration depth exceeded max_depth={} while elaborating '{}'",
                    state.config.max_depth, instantiation.instance_name
                ),
                0,
            ));
        }

        let module = self.module(&instantiation.module_name).ok_or_else(|| {
            FrontendError::new(
                format!(
                    "unknown module '{}' instantiated as '{}'",
                    instantiation.module_name, instantiation.instance_name
                ),
                0,
            )
        })?;

        let parameter_overrides =
            instantiation.resolve_parameter_overrides(module, parent_symbols)?;
        let imported_constants =
            self.resolve_imported_constant_symbols(module.all_imports(), &state.package_envs)?;
        let full_symbols = module.evaluate_full_constant_env(
            &state.package_symbols,
            &imported_constants,
            &parameter_overrides,
        )?;
        module.validate_header_types(&full_symbols)?;
        let parameters = module.extract_owned_constant_env(&full_symbols);
        let port_bindings = instantiation.resolve_port_bindings(
            module,
            &parent_scope.names,
            &parent_scope.types,
            parent_symbols,
        )?;
        let child_visible_scope = module.visible_connection_scope();
        let mut instances = Vec::new();

        for element_name in instantiation.element_instance_names(parent_symbols)? {
            let path = join_path(scope_path, &element_name);
            let child_instances = self.elaborate_items(
                &module.items,
                &full_symbols,
                &child_visible_scope,
                &path,
                depth + 1,
                state,
            )?;

            instances.push(ElaboratedInstance {
                module_name: module.name.clone(),
                instance_name: element_name,
                path,
                parameters: parameters.clone(),
                port_bindings: port_bindings.clone(),
                child_instances,
            });
        }

        Ok(instances)
    }

    fn evaluate_package_constant_envs(
        &self,
    ) -> Result<HashMap<String, HashMap<String, i64>>, FrontendError> {
        let mut envs = HashMap::new();
        let mut visible_symbols = HashMap::new();

        for package in &self.packages {
            let env = package.evaluate_constant_env(&visible_symbols)?;
            for (name, value) in &env {
                visible_symbols.insert(format!("{}::{name}", package.name), *value);
            }
            envs.insert(package.name.clone(), env);
        }

        Ok(envs)
    }

    fn resolve_imported_constant_symbols<'a>(
        &self,
        imports: impl IntoIterator<Item = &'a ImportDecl>,
        package_envs: &HashMap<String, HashMap<String, i64>>,
    ) -> Result<HashMap<String, i64>, FrontendError> {
        let mut symbols = HashMap::new();

        for import_decl in imports {
            let package_constants = package_envs.get(&import_decl.package).ok_or_else(|| {
                FrontendError::new(
                    format!(
                        "unknown package '{}' in import declaration",
                        import_decl.package
                    ),
                    0,
                )
            })?;
            match &import_decl.imported_name {
                None => {
                    symbols.extend(package_constants.clone());
                }
                Some(name) => {
                    if let Some(value) = package_constants.get(name) {
                        symbols.insert(name.clone(), *value);
                    }
                }
            }
        }

        Ok(symbols)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub name: String,
    pub header_imports: Vec<ImportDecl>,
    pub parameters: Vec<ParameterDecl>,
    pub ports: Vec<PortDecl>,
    pub items: Vec<ModuleItem>,
}

impl Module {
    pub fn evaluate_constant_env(
        &self,
        overrides: &HashMap<String, i64>,
    ) -> Result<HashMap<String, i64>, FrontendError> {
        self.evaluate_full_constant_env(&HashMap::new(), &HashMap::new(), overrides)
            .map(|symbols| self.extract_owned_constant_env(&symbols))
    }

    fn overrideable_parameters(&self) -> Vec<&ParameterDecl> {
        let mut parameters: Vec<&ParameterDecl> = self
            .parameters
            .iter()
            .filter(|decl| decl.flavor == ParameterFlavor::Parameter)
            .collect();

        for item in &self.items {
            if let ModuleItem::ParameterDecl(decl) = item
                && decl.flavor == ParameterFlavor::Parameter
            {
                parameters.push(decl);
            }
        }

        parameters
    }

    fn visible_connection_names(&self) -> HashSet<String> {
        let mut names = HashSet::new();

        for parameter in &self.parameters {
            names.insert(parameter.name.clone());
        }
        for port in &self.ports {
            names.insert(port.name.clone());
        }

        names.extend(collect_visible_names_from_items(&self.items));
        names
    }

    fn visible_connection_types(&self) -> HashMap<String, DeclType> {
        let mut types = HashMap::new();

        for port in &self.ports {
            if let Some(data_type) = &port.data_type {
                types.insert(
                    port.name.clone(),
                    DeclType {
                        data_type: data_type.clone(),
                        unpacked_dimensions: port.unpacked_dimensions.len(),
                    },
                );
            }
        }

        types.extend(collect_declared_types_from_items(&self.items));
        types
    }

    fn visible_connection_scope(&self) -> VisibleScope {
        VisibleScope {
            names: self.visible_connection_names(),
            types: self.visible_connection_types(),
        }
    }

    fn all_imports(&self) -> impl Iterator<Item = &ImportDecl> {
        self.header_imports
            .iter()
            .chain(self.items.iter().filter_map(|item| {
                if let ModuleItem::ImportDecl(import_decl) = item {
                    Some(import_decl)
                } else {
                    None
                }
            }))
    }

    fn evaluate_full_constant_env(
        &self,
        seed_symbols: &HashMap<String, i64>,
        imported_constants: &HashMap<String, i64>,
        overrides: &HashMap<String, i64>,
    ) -> Result<HashMap<String, i64>, FrontendError> {
        let mut symbols = seed_symbols.clone();
        symbols.extend(imported_constants.clone());
        for (name, value) in overrides {
            symbols.insert(name.clone(), *value);
        }

        for parameter in &self.parameters {
            apply_parameter_decl(&mut symbols, overrides, parameter)?;
        }
        for item in &self.items {
            if let ModuleItem::ParameterDecl(parameter) = item {
                apply_parameter_decl(&mut symbols, overrides, parameter)?;
            }
        }

        Ok(symbols)
    }

    fn validate_header_types(&self, symbols: &HashMap<String, i64>) -> Result<(), FrontendError> {
        for parameter in &self.parameters {
            if let Some(data_type) = &parameter.data_type {
                data_type.validate(symbols)?;
            }
        }
        for port in &self.ports {
            if let Some(data_type) = &port.data_type {
                data_type.validate(symbols)?;
            }
        }
        Ok(())
    }

    fn extract_owned_constant_env(&self, symbols: &HashMap<String, i64>) -> HashMap<String, i64> {
        let mut env = HashMap::new();

        for parameter in &self.parameters {
            if let Some(value) = symbols.get(&parameter.name) {
                env.insert(parameter.name.clone(), *value);
            }
        }
        for item in &self.items {
            if let ModuleItem::ParameterDecl(parameter) = item
                && let Some(value) = symbols.get(&parameter.name)
            {
                env.insert(parameter.name.clone(), *value);
            }
        }

        env
    }
}

fn validate_module_item_types(
    item: &ModuleItem,
    visible_scope: &VisibleScope,
    symbols: &HashMap<String, i64>,
) -> Result<(), FrontendError> {
    match item {
        ModuleItem::ParameterDecl(parameter) => {
            if let Some(data_type) = &parameter.data_type {
                data_type.validate(symbols)?;
            }
        }
        ModuleItem::TypedefDecl(typedef_decl) => {
            typedef_decl.data_type.validate(symbols)?;
        }
        ModuleItem::NetDecl(net) => {
            if let Some(data_type) = &net.data_type {
                data_type.validate(symbols)?;
            }
        }
        ModuleItem::ContinuousAssign(assign) => {
            assign
                .target
                .validate(&visible_scope.names, &visible_scope.types, symbols)?;
            assign
                .value
                .validate(&visible_scope.names, &visible_scope.types, symbols)?;
        }
        ModuleItem::ProceduralBlock(block) => {
            validate_procedural_block(block, visible_scope, symbols)?;
        }
        ModuleItem::ImportDecl(_)
        | ModuleItem::GenvarDecl(_)
        | ModuleItem::ModuleInstantiation(_)
        | ModuleItem::GenerateRegion(_)
        | ModuleItem::GenerateIf(_)
        | ModuleItem::GenerateFor(_) => {}
    }

    Ok(())
}

fn validate_procedural_block(
    block: &ProceduralBlock,
    visible_scope: &VisibleScope,
    symbols: &HashMap<String, i64>,
) -> Result<(), FrontendError> {
    match block.kind {
        ProceduralKind::AlwaysFf => {
            let event_control = block.event_control.as_ref().ok_or_else(|| {
                FrontendError::new("always_ff block is missing required event control", 0)
            })?;
            if event_control.is_empty() {
                return Err(FrontendError::new(
                    "always_ff block requires at least one event-control item",
                    0,
                ));
            }
            for item in event_control {
                validate_expr_identifiers(
                    &item.expr,
                    &visible_scope.names,
                    &visible_scope.types,
                    symbols,
                )?;
            }
            validate_statement(
                &block.statement,
                visible_scope,
                symbols,
                ProceduralPolicy {
                    allow_blocking_assignments: false,
                },
            )
        }
        ProceduralKind::AlwaysComb | ProceduralKind::AlwaysStar | ProceduralKind::AlwaysLatch => {
            validate_statement(
                &block.statement,
                visible_scope,
                symbols,
                ProceduralPolicy {
                    allow_blocking_assignments: true,
                },
            )
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ProceduralPolicy {
    allow_blocking_assignments: bool,
}

fn validate_statement(
    statement: &Statement,
    visible_scope: &VisibleScope,
    symbols: &HashMap<String, i64>,
    policy: ProceduralPolicy,
) -> Result<(), FrontendError> {
    match statement {
        Statement::Block { statements, .. } => {
            for statement in statements {
                validate_statement(statement, visible_scope, symbols, policy)?;
            }
            Ok(())
        }
        Statement::If {
            condition,
            then_branch,
            else_branch,
        } => {
            validate_expr_identifiers(
                condition,
                &visible_scope.names,
                &visible_scope.types,
                symbols,
            )?;
            validate_statement(then_branch, visible_scope, symbols, policy)?;
            if let Some(else_branch) = else_branch {
                validate_statement(else_branch, visible_scope, symbols, policy)?;
            }
            Ok(())
        }
        Statement::Assignment {
            target,
            kind,
            value,
        } => {
            target.validate(&visible_scope.names, &visible_scope.types, symbols)?;
            value.validate(&visible_scope.names, &visible_scope.types, symbols)?;
            if *kind == AssignmentKind::Blocking && !policy.allow_blocking_assignments {
                return Err(FrontendError::new(
                    format!(
                        "always_ff block does not allow blocking assignment to '{}'",
                        target.describe()
                    ),
                    0,
                ));
            }
            Ok(())
        }
        Statement::Empty => Ok(()),
    }
}

fn first_blocking_assignment_target(statement: &Statement) -> Option<&AssignmentTarget> {
    match statement {
        Statement::Block { statements, .. } => {
            statements.iter().find_map(first_blocking_assignment_target)
        }
        Statement::If {
            then_branch,
            else_branch,
            ..
        } => first_blocking_assignment_target(then_branch).or_else(|| {
            else_branch
                .as_deref()
                .and_then(first_blocking_assignment_target)
        }),
        Statement::Assignment {
            target,
            kind: AssignmentKind::Blocking,
            ..
        } => Some(target),
        Statement::Assignment { .. } | Statement::Empty => None,
    }
}

fn collect_visible_names_from_items(items: &[ModuleItem]) -> HashSet<String> {
    let mut names = HashSet::new();

    for item in items {
        match item {
            ModuleItem::ParameterDecl(parameter) => {
                names.insert(parameter.name.clone());
            }
            ModuleItem::TypedefDecl(_) => {}
            ModuleItem::ImportDecl(_) => {}
            ModuleItem::GenvarDecl(genvar) => {
                names.extend(genvar.names.iter().cloned());
            }
            ModuleItem::NetDecl(net) => {
                names.insert(net.name.clone());
            }
            ModuleItem::GenerateRegion(region) => {
                names.extend(collect_visible_names_from_items(&region.items));
            }
            ModuleItem::GenerateIf(generate_if) => {
                names.extend(collect_visible_names_from_items(&generate_if.then_items));
                names.extend(collect_visible_names_from_items(&generate_if.else_items));
            }
            ModuleItem::GenerateFor(generate_for) => {
                names.insert(generate_for.index_name.clone());
                names.extend(collect_visible_names_from_items(&generate_for.body_items));
            }
            ModuleItem::ContinuousAssign(_)
            | ModuleItem::ModuleInstantiation(_)
            | ModuleItem::ProceduralBlock(_) => {}
        }
    }

    names
}

fn collect_declared_types_from_items(items: &[ModuleItem]) -> HashMap<String, DeclType> {
    let mut types = HashMap::new();

    for item in items {
        match item {
            ModuleItem::NetDecl(net) => {
                if let Some(data_type) = &net.data_type {
                    types.insert(
                        net.name.clone(),
                        DeclType {
                            data_type: data_type.clone(),
                            unpacked_dimensions: net.unpacked_dimensions.len(),
                        },
                    );
                }
            }
            ModuleItem::GenerateRegion(region) => {
                types.extend(collect_declared_types_from_items(&region.items));
            }
            ModuleItem::GenerateIf(generate_if) => {
                types.extend(collect_declared_types_from_items(&generate_if.then_items));
                types.extend(collect_declared_types_from_items(&generate_if.else_items));
            }
            ModuleItem::GenerateFor(generate_for) => {
                types.extend(collect_declared_types_from_items(&generate_for.body_items));
            }
            ModuleItem::ParameterDecl(_)
            | ModuleItem::TypedefDecl(_)
            | ModuleItem::ImportDecl(_)
            | ModuleItem::GenvarDecl(_)
            | ModuleItem::ContinuousAssign(_)
            | ModuleItem::ModuleInstantiation(_)
            | ModuleItem::ProceduralBlock(_) => {}
        }
    }

    types
}

fn merge_visible_names(base: &HashSet<String>, extra: &HashSet<String>) -> HashSet<String> {
    let mut merged = base.clone();
    merged.extend(extra.iter().cloned());
    merged
}

fn merge_declared_types(
    base: &HashMap<String, DeclType>,
    extra: &HashMap<String, DeclType>,
) -> HashMap<String, DeclType> {
    let mut merged = base.clone();
    merged.extend(extra.clone());
    merged
}

fn flatten_package_constant_envs(
    package_envs: &HashMap<String, HashMap<String, i64>>,
) -> HashMap<String, i64> {
    let mut symbols = HashMap::new();
    for (package_name, constants) in package_envs {
        for (name, value) in constants {
            symbols.insert(format!("{package_name}::{name}"), *value);
        }
    }
    symbols
}

fn apply_parameter_decl(
    symbols: &mut HashMap<String, i64>,
    overrides: &HashMap<String, i64>,
    decl: &ParameterDecl,
) -> Result<(), FrontendError> {
    if decl.flavor == ParameterFlavor::Parameter
        && let Some(value) = overrides.get(&decl.name)
    {
        symbols.insert(decl.name.clone(), *value);
        return Ok(());
    }

    let Some(expr) = decl.value.as_ref() else {
        return Err(FrontendError::new(
            format!(
                "missing default value for constant declaration '{}'",
                decl.name
            ),
            0,
        ));
    };

    let value = expr.eval(symbols).map_err(|err| {
        FrontendError::new(
            format!(
                "failed to evaluate constant declaration '{}': {}",
                decl.name, err.message
            ),
            err.position,
        )
    })?;
    symbols.insert(decl.name.clone(), value);
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    Builtin {
        keyword: String,
    },
    Packed {
        data_type: Box<DataType>,
        packed_range: PackedRange,
    },
    Enum {
        base_type: Box<DataType>,
        variants: Vec<EnumVariant>,
    },
    Union {
        packed: bool,
        fields: Vec<StructField>,
    },
    Struct {
        packed: bool,
        fields: Vec<StructField>,
    },
}

impl DataType {
    fn field_type(&self, name: &str) -> Option<&DataType> {
        match self {
            DataType::Struct { fields, .. } => {
                for field in fields {
                    if field.names.iter().any(|field_name| field_name == name) {
                        return Some(&field.data_type);
                    }
                }
                None
            }
            DataType::Union { fields, .. } => {
                for field in fields {
                    if field.names.iter().any(|field_name| field_name == name) {
                        return Some(&field.data_type);
                    }
                }
                None
            }
            DataType::Packed { data_type, .. } => data_type.field_type(name),
            DataType::Builtin { .. } | DataType::Enum { .. } => None,
        }
    }

    fn validate(&self, symbols: &HashMap<String, i64>) -> Result<(), FrontendError> {
        match self {
            DataType::Builtin { .. } => Ok(()),
            DataType::Packed {
                data_type,
                packed_range,
            } => {
                data_type.validate(symbols)?;
                let _ = packed_range.width(symbols)?;
                let _ = data_type.bit_width(symbols)?;
                Ok(())
            }
            DataType::Enum {
                base_type,
                variants,
            } => {
                base_type.validate(symbols)?;
                let _ = base_type.bit_width(symbols)?;
                for variant in variants {
                    if let Some(value) = &variant.value {
                        let _ = value.eval(symbols).map_err(map_eval_error)?;
                    }
                }
                Ok(())
            }
            DataType::Union { packed, fields } => {
                let mut expected_width = None;
                let mut expected_name = None;
                for field in fields {
                    field.validate(symbols)?;
                    if *packed {
                        let width = field.bit_width(symbols)?;
                        match expected_width {
                            None => {
                                expected_width = Some(width);
                                expected_name = field.names.first().cloned();
                            }
                            Some(expected_width) if expected_width != width => {
                                return Err(FrontendError::new(
                                    format!(
                                        "packed union field '{}' width {} does not match field '{}' width {}",
                                        field.first_name(),
                                        width,
                                        expected_name.as_deref().unwrap_or("<unnamed>"),
                                        expected_width
                                    ),
                                    0,
                                ));
                            }
                            Some(_) => {}
                        }
                    }
                }
                Ok(())
            }
            DataType::Struct { packed, fields } => {
                for field in fields {
                    field.validate(symbols)?;
                }
                if *packed {
                    for field in fields {
                        let _ = field.bit_width(symbols)?;
                    }
                }
                Ok(())
            }
        }
    }

    fn bit_width(&self, symbols: &HashMap<String, i64>) -> Result<i64, FrontendError> {
        match self {
            DataType::Builtin { keyword } => match keyword.as_str() {
                "bit" | "logic" | "reg" | "wire" => Ok(1),
                "byte" => Ok(8),
                "shortint" => Ok(16),
                "int" | "integer" => Ok(32),
                "longint" => Ok(64),
                _ => Err(FrontendError::new(
                    format!("unsupported width query for builtin type '{}'", keyword),
                    0,
                )),
            },
            DataType::Packed {
                data_type,
                packed_range,
            } => Ok(data_type.bit_width(symbols)? * packed_range.width(symbols)?),
            DataType::Enum { base_type, .. } => base_type.bit_width(symbols),
            DataType::Union { packed, fields } => {
                if !packed {
                    return Err(FrontendError::new(
                        "cannot compute width of unpacked union type",
                        0,
                    ));
                }
                let Some(first_field) = fields.first() else {
                    return Err(FrontendError::new("packed union has no fields", 0));
                };
                first_field.bit_width(symbols)
            }
            DataType::Struct { packed, fields } => {
                if !packed {
                    return Err(FrontendError::new(
                        "cannot compute width of unpacked struct type",
                        0,
                    ));
                }
                let mut total = 0i64;
                for field in fields {
                    total += field.bit_width(symbols)?;
                }
                Ok(total)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedefDecl {
    pub name: String,
    pub data_type: DataType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageDecl {
    pub name: String,
    pub typedefs: Vec<TypedefDecl>,
    pub constants: Vec<ParameterDecl>,
}

impl PackageDecl {
    fn evaluate_constant_env(
        &self,
        seed_symbols: &HashMap<String, i64>,
    ) -> Result<HashMap<String, i64>, FrontendError> {
        let mut symbols = seed_symbols.clone();
        let overrides = HashMap::new();
        for constant in &self.constants {
            apply_parameter_decl(&mut symbols, &overrides, constant)?;
        }

        let mut env = HashMap::new();
        for constant in &self.constants {
            if let Some(value) = symbols.get(&constant.name) {
                env.insert(constant.name.clone(), *value);
            }
        }

        Ok(env)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportDecl {
    pub package: String,
    pub imported_name: Option<String>,
    pub wildcard: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructField {
    pub data_type: DataType,
    pub packed_range: Option<PackedRange>,
    pub names: Vec<String>,
}

impl StructField {
    fn validate(&self, symbols: &HashMap<String, i64>) -> Result<(), FrontendError> {
        self.data_type.validate(symbols)?;
        if let Some(range) = &self.packed_range {
            let _ = range.width(symbols)?;
        }
        Ok(())
    }

    fn bit_width(&self, symbols: &HashMap<String, i64>) -> Result<i64, FrontendError> {
        let mut width = self.data_type.bit_width(symbols)?;
        if let Some(range) = &self.packed_range {
            width *= range.width(symbols)?;
        }
        Ok(width)
    }

    fn first_name(&self) -> &str {
        self.names
            .first()
            .map(String::as_str)
            .unwrap_or("<unnamed>")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedRange {
    pub msb: Expr,
    pub lsb: Expr,
}

impl PackedRange {
    pub fn width(&self, symbols: &HashMap<String, i64>) -> Result<i64, FrontendError> {
        let msb = self.msb.eval(symbols).map_err(map_eval_error)?;
        let lsb = self.lsb.eval(symbols).map_err(map_eval_error)?;
        Ok((msb - lsb).abs() + 1)
    }

    pub fn indices(&self, symbols: &HashMap<String, i64>) -> Result<Vec<i64>, FrontendError> {
        let msb = self.msb.eval(symbols).map_err(map_eval_error)?;
        let lsb = self.lsb.eval(symbols).map_err(map_eval_error)?;
        let mut values = Vec::new();

        if msb <= lsb {
            for value in msb..=lsb {
                values.push(value);
            }
        } else {
            for value in (lsb..=msb).rev() {
                values.push(value);
            }
        }

        Ok(values)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterFlavor {
    Parameter,
    Localparam,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterDecl {
    pub flavor: ParameterFlavor,
    pub data_type: Option<DataType>,
    pub name: String,
    pub value: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortDirection {
    Input,
    Output,
    Inout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortDecl {
    pub direction: PortDirection,
    pub data_type: Option<DataType>,
    pub packed_range: Option<PackedRange>,
    pub unpacked_dimensions: Vec<PackedRange>,
    pub name: String,
}

impl PortDecl {
    pub fn width(&self, symbols: &HashMap<String, i64>) -> Result<Option<i64>, FrontendError> {
        self.packed_range
            .as_ref()
            .map(|range| range.width(symbols))
            .transpose()
    }

    pub fn unpacked_shape(
        &self,
        symbols: &HashMap<String, i64>,
    ) -> Result<Vec<i64>, FrontendError> {
        self.unpacked_dimensions
            .iter()
            .map(|range| range.width(symbols))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetKind {
    Logic,
    Wire,
    Reg,
    Typed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetDecl {
    pub kind: NetKind,
    pub data_type: Option<DataType>,
    pub packed_range: Option<PackedRange>,
    pub unpacked_dimensions: Vec<PackedRange>,
    pub name: String,
}

impl NetDecl {
    pub fn width(&self, symbols: &HashMap<String, i64>) -> Result<Option<i64>, FrontendError> {
        self.packed_range
            .as_ref()
            .map(|range| range.width(symbols))
            .transpose()
    }

    pub fn unpacked_shape(
        &self,
        symbols: &HashMap<String, i64>,
    ) -> Result<Vec<i64>, FrontendError> {
        self.unpacked_dimensions
            .iter()
            .map(|range| range.width(symbols))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenvarDecl {
    pub names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuousAssign {
    pub target: AssignmentTarget,
    pub value: ValueExpr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterOverride {
    Ordered(Expr),
    Named { name: String, value: Expr },
    OrderedSyntax(String),
    NamedSyntax { name: String, value: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssignmentTarget {
    Signal(String),
    BitSelect {
        signal: String,
        index: Expr,
    },
    PartSelect {
        signal: String,
        msb: Expr,
        lsb: Expr,
    },
    Concat(Vec<AssignmentTarget>),
}

impl AssignmentTarget {
    fn validate(
        &self,
        visible_names: &HashSet<String>,
        visible_types: &HashMap<String, DeclType>,
        symbols: &HashMap<String, i64>,
    ) -> Result<(), FrontendError> {
        match self {
            AssignmentTarget::Signal(name) => {
                validate_known_identifier(name, visible_names, visible_types, symbols)
            }
            AssignmentTarget::BitSelect { signal, index } => {
                validate_known_identifier(signal, visible_names, visible_types, symbols)?;
                validate_expr_identifiers(index, visible_names, visible_types, symbols)
            }
            AssignmentTarget::PartSelect { signal, msb, lsb } => {
                validate_known_identifier(signal, visible_names, visible_types, symbols)?;
                validate_expr_identifiers(msb, visible_names, visible_types, symbols)?;
                validate_expr_identifiers(lsb, visible_names, visible_types, symbols)
            }
            AssignmentTarget::Concat(items) => {
                for item in items {
                    item.validate(visible_names, visible_types, symbols)?;
                }
                Ok(())
            }
        }
    }

    fn describe(&self) -> String {
        match self {
            AssignmentTarget::Signal(name)
            | AssignmentTarget::BitSelect { signal: name, .. }
            | AssignmentTarget::PartSelect { signal: name, .. } => name.clone(),
            AssignmentTarget::Concat(_) => "{...}".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueExpr {
    Signal(String),
    BitSelect {
        signal: String,
        index: Expr,
    },
    PartSelect {
        signal: String,
        msb: Expr,
        lsb: Expr,
    },
    Concat(Vec<ValueExpr>),
    Repeat {
        count: Expr,
        value: Box<ValueExpr>,
    },
    Expression(Expr),
    ExpressionText(String),
}

impl ValueExpr {
    fn validate(
        &self,
        visible_names: &HashSet<String>,
        visible_types: &HashMap<String, DeclType>,
        symbols: &HashMap<String, i64>,
    ) -> Result<(), FrontendError> {
        match self {
            ValueExpr::Signal(name) => {
                validate_known_identifier(name, visible_names, visible_types, symbols)
            }
            ValueExpr::BitSelect { signal, index } => {
                validate_known_identifier(signal, visible_names, visible_types, symbols)?;
                validate_expr_identifiers(index, visible_names, visible_types, symbols)
            }
            ValueExpr::PartSelect { signal, msb, lsb } => {
                validate_known_identifier(signal, visible_names, visible_types, symbols)?;
                validate_expr_identifiers(msb, visible_names, visible_types, symbols)?;
                validate_expr_identifiers(lsb, visible_names, visible_types, symbols)
            }
            ValueExpr::Concat(items) => {
                for item in items {
                    item.validate(visible_names, visible_types, symbols)?;
                }
                Ok(())
            }
            ValueExpr::Repeat { count, value } => {
                validate_expr_identifiers(count, visible_names, visible_types, symbols)?;
                value.validate(visible_names, visible_types, symbols)
            }
            ValueExpr::Expression(expr) => {
                validate_expr_identifiers(expr, visible_names, visible_types, symbols)
            }
            ValueExpr::ExpressionText(text) => {
                validate_expression_text_identifiers(text, visible_names, visible_types, symbols)
            }
        }
    }
}

impl From<PortActual> for ValueExpr {
    fn from(value: PortActual) -> Self {
        match value {
            PortActual::Signal(name) => ValueExpr::Signal(name),
            PortActual::BitSelect { signal, index } => ValueExpr::BitSelect { signal, index },
            PortActual::PartSelect { signal, msb, lsb } => {
                ValueExpr::PartSelect { signal, msb, lsb }
            }
            PortActual::Concat(items) => {
                ValueExpr::Concat(items.into_iter().map(ValueExpr::from).collect())
            }
            PortActual::Repeat { count, value } => ValueExpr::Repeat {
                count,
                value: Box::new(ValueExpr::from(*value)),
            },
            PortActual::Expression(expr) => ValueExpr::Expression(expr),
            PortActual::ExpressionText(text) => ValueExpr::ExpressionText(text),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortActual {
    Signal(String),
    BitSelect {
        signal: String,
        index: Expr,
    },
    PartSelect {
        signal: String,
        msb: Expr,
        lsb: Expr,
    },
    Concat(Vec<PortActual>),
    Repeat {
        count: Expr,
        value: Box<PortActual>,
    },
    Expression(Expr),
    ExpressionText(String),
}

impl PortActual {
    fn validate(
        &self,
        visible_names: &HashSet<String>,
        visible_types: &HashMap<String, DeclType>,
        symbols: &HashMap<String, i64>,
    ) -> Result<(), FrontendError> {
        match self {
            PortActual::Signal(name) => {
                validate_known_identifier(name, visible_names, visible_types, symbols)
            }
            PortActual::BitSelect { signal, index } => {
                validate_known_identifier(signal, visible_names, visible_types, symbols)?;
                validate_expr_identifiers(index, visible_names, visible_types, symbols)
            }
            PortActual::PartSelect { signal, msb, lsb } => {
                validate_known_identifier(signal, visible_names, visible_types, symbols)?;
                validate_expr_identifiers(msb, visible_names, visible_types, symbols)?;
                validate_expr_identifiers(lsb, visible_names, visible_types, symbols)
            }
            PortActual::Concat(items) => {
                for item in items {
                    item.validate(visible_names, visible_types, symbols)?;
                }
                Ok(())
            }
            PortActual::Repeat { count, value } => {
                validate_expr_identifiers(count, visible_names, visible_types, symbols)?;
                value.validate(visible_names, visible_types, symbols)
            }
            PortActual::Expression(expr) => {
                validate_expr_identifiers(expr, visible_names, visible_types, symbols)
            }
            PortActual::ExpressionText(text) => {
                validate_expression_text_identifiers(text, visible_names, visible_types, symbols)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortConnection {
    Ordered {
        actual: Option<PortActual>,
    },
    Named {
        port: String,
        actual: Option<PortActual>,
    },
    Wildcard,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleInstantiation {
    pub module_name: String,
    pub instance_name: String,
    pub instance_range: Option<PackedRange>,
    pub parameter_overrides: Vec<ParameterOverride>,
    pub port_connections: Vec<PortConnection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPortBinding {
    pub port_name: String,
    pub actual: Option<PortActual>,
}

impl ModuleInstantiation {
    pub fn element_instance_names(
        &self,
        parent_symbols: &HashMap<String, i64>,
    ) -> Result<Vec<String>, FrontendError> {
        let Some(range) = &self.instance_range else {
            return Ok(vec![self.instance_name.clone()]);
        };

        let mut names = Vec::new();
        for index in range.indices(parent_symbols)? {
            names.push(format!("{}[{index}]", self.instance_name));
        }
        Ok(names)
    }

    pub fn resolve_parameter_overrides(
        &self,
        module: &Module,
        parent_symbols: &HashMap<String, i64>,
    ) -> Result<HashMap<String, i64>, FrontendError> {
        let parameter_targets = module.overrideable_parameters();
        let mut overrides = HashMap::new();
        let mut saw_named = false;
        let mut saw_ordered = false;
        let mut ordered_index = 0usize;

        for override_entry in &self.parameter_overrides {
            match override_entry {
                ParameterOverride::Ordered(expr) => {
                    if saw_named {
                        return Err(FrontendError::new(
                            format!(
                                "cannot mix positional and named parameter overrides on instance '{}'",
                                self.instance_name
                            ),
                            0,
                        ));
                    }
                    saw_ordered = true;
                    let Some(target) = parameter_targets.get(ordered_index) else {
                        return Err(FrontendError::new(
                            format!(
                                "too many positional parameter overrides on instance '{}'",
                                self.instance_name
                            ),
                            0,
                        ));
                    };
                    let value = expr.eval(parent_symbols).map_err(|err| {
                        FrontendError::new(
                            format!(
                                "failed to evaluate positional parameter override {} on instance '{}': {}",
                                ordered_index + 1,
                                self.instance_name,
                                err.message
                            ),
                            err.position,
                        )
                    })?;
                    overrides.insert(target.name.clone(), value);
                    ordered_index += 1;
                }
                ParameterOverride::Named { name, value } => {
                    if saw_ordered {
                        return Err(FrontendError::new(
                            format!(
                                "cannot mix positional and named parameter overrides on instance '{}'",
                                self.instance_name
                            ),
                            0,
                        ));
                    }
                    saw_named = true;
                    if overrides.contains_key(name) {
                        return Err(FrontendError::new(
                            format!(
                                "duplicate named parameter override '{}' on instance '{}'",
                                name, self.instance_name
                            ),
                            0,
                        ));
                    }
                    if !parameter_targets.iter().any(|decl| decl.name == *name) {
                        return Err(FrontendError::new(
                            format!(
                                "unknown parameter '{}' on instance '{}' of module '{}'",
                                name, self.instance_name, module.name
                            ),
                            0,
                        ));
                    }
                    let evaluated = value.eval(parent_symbols).map_err(|err| {
                        FrontendError::new(
                            format!(
                                "failed to evaluate named parameter override '{}' on instance '{}': {}",
                                name, self.instance_name, err.message
                            ),
                            err.position,
                        )
                    })?;
                    overrides.insert(name.clone(), evaluated);
                }
                ParameterOverride::OrderedSyntax(value) => {
                    if saw_named {
                        return Err(FrontendError::new(
                            format!(
                                "cannot mix positional and named parameter overrides on instance '{}'",
                                self.instance_name
                            ),
                            0,
                        ));
                    }
                    let Some(target) = parameter_targets.get(ordered_index) else {
                        return Err(FrontendError::new(
                            format!(
                                "too many positional parameter overrides on instance '{}'",
                                self.instance_name
                            ),
                            0,
                        ));
                    };
                    return Err(FrontendError::new(
                        format!(
                            "cannot evaluate syntax-only positional parameter override {} ('{}') for parameter '{}' on instance '{}'",
                            ordered_index + 1,
                            value,
                            target.name,
                            self.instance_name
                        ),
                        0,
                    ));
                }
                ParameterOverride::NamedSyntax { name, value } => {
                    if saw_ordered {
                        return Err(FrontendError::new(
                            format!(
                                "cannot mix positional and named parameter overrides on instance '{}'",
                                self.instance_name
                            ),
                            0,
                        ));
                    }
                    if overrides.contains_key(name) {
                        return Err(FrontendError::new(
                            format!(
                                "duplicate named parameter override '{}' on instance '{}'",
                                name, self.instance_name
                            ),
                            0,
                        ));
                    }
                    if !parameter_targets.iter().any(|decl| decl.name == *name) {
                        return Err(FrontendError::new(
                            format!(
                                "unknown parameter '{}' on instance '{}' of module '{}'",
                                name, self.instance_name, module.name
                            ),
                            0,
                        ));
                    }
                    return Err(FrontendError::new(
                        format!(
                            "cannot evaluate syntax-only named parameter override '{}' ('{}') on instance '{}'",
                            name, value, self.instance_name
                        ),
                        0,
                    ));
                }
            }
        }

        Ok(overrides)
    }

    fn resolve_port_bindings(
        &self,
        module: &Module,
        parent_visible_names: &HashSet<String>,
        parent_visible_types: &HashMap<String, DeclType>,
        parent_symbols: &HashMap<String, i64>,
    ) -> Result<Vec<ResolvedPortBinding>, FrontendError> {
        let mut bindings = Vec::new();
        let mut bound_ports = HashSet::new();
        let mut ordered_index = 0usize;
        let mut saw_named = false;
        let mut saw_ordered = false;
        let mut wildcard_seen = false;

        for connection in &self.port_connections {
            match connection {
                PortConnection::Ordered { actual } => {
                    if saw_named {
                        return Err(FrontendError::new(
                            format!(
                                "cannot mix positional and named port connections on instance '{}'",
                                self.instance_name
                            ),
                            0,
                        ));
                    }
                    saw_ordered = true;
                    let Some(port) = module.ports.get(ordered_index) else {
                        return Err(FrontendError::new(
                            format!(
                                "too many positional port connections on instance '{}'",
                                self.instance_name
                            ),
                            0,
                        ));
                    };
                    if let Some(actual) = actual {
                        actual.validate(
                            parent_visible_names,
                            parent_visible_types,
                            parent_symbols,
                        )?;
                    }
                    bindings.push(ResolvedPortBinding {
                        port_name: port.name.clone(),
                        actual: actual.clone(),
                    });
                    bound_ports.insert(port.name.clone());
                    ordered_index += 1;
                }
                PortConnection::Named { port, actual } => {
                    if saw_ordered {
                        return Err(FrontendError::new(
                            format!(
                                "cannot mix positional and named port connections on instance '{}'",
                                self.instance_name
                            ),
                            0,
                        ));
                    }
                    saw_named = true;
                    if !module.ports.iter().any(|decl| decl.name == *port) {
                        return Err(FrontendError::new(
                            format!(
                                "unknown port '{}' on instance '{}' of module '{}'",
                                port, self.instance_name, module.name
                            ),
                            0,
                        ));
                    }
                    if !bound_ports.insert(port.clone()) {
                        return Err(FrontendError::new(
                            format!(
                                "duplicate binding for port '{}' on instance '{}'",
                                port, self.instance_name
                            ),
                            0,
                        ));
                    }
                    if let Some(actual) = actual {
                        actual.validate(
                            parent_visible_names,
                            parent_visible_types,
                            parent_symbols,
                        )?;
                    }
                    bindings.push(ResolvedPortBinding {
                        port_name: port.clone(),
                        actual: actual.clone(),
                    });
                }
                PortConnection::Wildcard => {
                    if saw_ordered {
                        return Err(FrontendError::new(
                            format!(
                                "cannot mix positional and wildcard/named port connections on instance '{}'",
                                self.instance_name
                            ),
                            0,
                        ));
                    }
                    saw_named = true;
                    if wildcard_seen {
                        return Err(FrontendError::new(
                            format!(
                                "duplicate wildcard port connection on instance '{}'",
                                self.instance_name
                            ),
                            0,
                        ));
                    }
                    wildcard_seen = true;
                }
            }
        }

        if wildcard_seen {
            for port in &module.ports {
                if bound_ports.insert(port.name.clone()) {
                    let actual = PortActual::Signal(port.name.clone());
                    actual.validate(parent_visible_names, parent_visible_types, parent_symbols)?;
                    bindings.push(ResolvedPortBinding {
                        port_name: port.name.clone(),
                        actual: Some(actual),
                    });
                }
            }
        }

        Ok(bindings)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProceduralKind {
    AlwaysComb,
    AlwaysStar,
    AlwaysLatch,
    AlwaysFf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventEdge {
    Posedge,
    Negedge,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventControlItem {
    pub edge: Option<EventEdge>,
    pub expr: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProceduralBlock {
    pub kind: ProceduralKind,
    pub event_control: Option<Vec<EventControlItem>>,
    pub statement: Statement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssignmentKind {
    Blocking,
    NonBlocking,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Block {
        label: Option<String>,
        statements: Vec<Statement>,
    },
    If {
        condition: Expr,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    Assignment {
        target: AssignmentTarget,
        kind: AssignmentKind,
        value: ValueExpr,
    },
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateRegion {
    pub items: Vec<ModuleItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateIf {
    pub condition: Expr,
    pub then_label: Option<String>,
    pub then_items: Vec<ModuleItem>,
    pub else_label: Option<String>,
    pub else_items: Vec<ModuleItem>,
}

impl GenerateIf {
    pub fn is_enabled(&self, symbols: &HashMap<String, i64>) -> Result<bool, FrontendError> {
        Ok(self.condition.eval(symbols).map_err(map_eval_error)? != 0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateFor {
    pub index_name: String,
    pub declares_genvar: bool,
    pub init: Expr,
    pub condition: Expr,
    pub step: Expr,
    pub label: Option<String>,
    pub body_items: Vec<ModuleItem>,
}

impl GenerateFor {
    pub fn iteration_values(
        &self,
        symbols: &HashMap<String, i64>,
        max_iterations: usize,
    ) -> Result<Vec<i64>, FrontendError> {
        let mut local = symbols.clone();
        let initial = self.init.eval(symbols).map_err(map_eval_error)?;
        local.insert(self.index_name.clone(), initial);

        let mut values = Vec::new();
        for _ in 0..max_iterations {
            let current = *local
                .get(&self.index_name)
                .ok_or_else(|| FrontendError::new("missing loop index value", 0))?;
            if self.condition.eval(&local).map_err(map_eval_error)? == 0 {
                return Ok(values);
            }
            values.push(current);
            let next = self.step.eval(&local).map_err(map_eval_error)?;
            local.insert(self.index_name.clone(), next);
        }

        Err(FrontendError::new(
            format!(
                "generate-for loop exceeded max_iterations={} while unrolling '{}'",
                max_iterations, self.index_name
            ),
            0,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleItem {
    ParameterDecl(ParameterDecl),
    TypedefDecl(TypedefDecl),
    ImportDecl(ImportDecl),
    GenvarDecl(GenvarDecl),
    NetDecl(NetDecl),
    ContinuousAssign(ContinuousAssign),
    ModuleInstantiation(ModuleInstantiation),
    ProceduralBlock(ProceduralBlock),
    GenerateRegion(GenerateRegion),
    GenerateIf(GenerateIf),
    GenerateFor(GenerateFor),
}

pub fn parse_design(input: &str) -> Result<Design, FrontendError> {
    let tokens = Lexer::new(input).tokenize()?;
    Parser::new(input, tokens).parse_design()
}

pub fn parse_module(input: &str) -> Result<Module, FrontendError> {
    let design = parse_design(input)?;
    match design.modules.as_slice() {
        [module] => Ok(module.clone()),
        [] => Err(FrontendError::new(
            "expected exactly one module, found none",
            0,
        )),
        _ => Err(FrontendError::new(
            format!(
                "expected exactly one module, found {}",
                design.modules.len()
            ),
            0,
        )),
    }
}

fn join_path(base: &str, segment: &str) -> String {
    if base.is_empty() {
        segment.to_string()
    } else {
        format!("{base}.{segment}")
    }
}

fn map_eval_error(err: EvalError) -> FrontendError {
    FrontendError::new(err.message, err.position)
}

fn validate_known_identifier(
    name: &str,
    visible_names: &HashSet<String>,
    visible_types: &HashMap<String, DeclType>,
    symbols: &HashMap<String, i64>,
) -> Result<(), FrontendError> {
    if visible_names.contains(name) || symbols.contains_key(name) {
        return Ok(());
    }

    if let Some(result) = validate_typed_signal_path(name, visible_names, visible_types, symbols) {
        return result;
    }

    let root = identifier_root(name);
    if visible_names.contains(root) || symbols.contains_key(root) {
        return Ok(());
    }

    Err(FrontendError::new(
        format!("unknown parent-scope identifier '{}'", name),
        0,
    ))
}

fn validate_expr_identifiers(
    expr: &Expr,
    visible_names: &HashSet<String>,
    visible_types: &HashMap<String, DeclType>,
    symbols: &HashMap<String, i64>,
) -> Result<(), FrontendError> {
    let mut identifiers = HashSet::new();
    collect_expr_identifiers(expr, &mut identifiers);
    for identifier in identifiers {
        validate_known_identifier(&identifier, visible_names, visible_types, symbols)?;
    }
    Ok(())
}

fn validate_typed_signal_path(
    name: &str,
    visible_names: &HashSet<String>,
    visible_types: &HashMap<String, DeclType>,
    symbols: &HashMap<String, i64>,
) -> Option<Result<(), FrontendError>> {
    let path = parse_signal_path(name).ok()??;
    let declared = visible_types.get(&path.root)?;
    let mut current = &declared.data_type;
    let mut unpacked_dimensions = declared.unpacked_dimensions;
    let mut resolved_struct_member = false;

    for op in &path.ops {
        match op {
            SignalPathOp::Member(segment) => {
                if unpacked_dimensions > 0 {
                    return Some(Err(FrontendError::new(
                        format!(
                            "cannot resolve member '{}' through unpacked-array parent-scope identifier '{}'",
                            segment, path.root
                        ),
                        0,
                    )));
                }
                let Some(next) = current.field_type(segment) else {
                    return match current {
                        DataType::Struct { .. } => Some(Err(FrontendError::new(
                            format!(
                                "unknown member '{}' on parent-scope identifier '{}'",
                                segment, path.root
                            ),
                            0,
                        ))),
                        DataType::Union { .. } => Some(Err(FrontendError::new(
                            format!(
                                "unknown member '{}' on parent-scope identifier '{}'",
                                segment, path.root
                            ),
                            0,
                        ))),
                        _ if resolved_struct_member || unpacked_dimensions > 0 => {
                            Some(Err(FrontendError::new(
                                format!(
                                    "cannot resolve member '{}' through non-aggregate parent-scope identifier '{}'",
                                    segment, path.root
                                ),
                                0,
                            )))
                        }
                        _ => None,
                    };
                };
                resolved_struct_member = true;
                current = next;
            }
            SignalPathOp::Index(index) => {
                if let Err(err) =
                    validate_expr_identifiers(index, visible_names, visible_types, symbols)
                {
                    return Some(Err(err));
                }
                if unpacked_dimensions > 0 {
                    unpacked_dimensions -= 1;
                    continue;
                }
                return Some(Err(FrontendError::new(
                    format!(
                        "cannot index non-array parent-scope identifier '{}'",
                        path.root
                    ),
                    0,
                )));
            }
        }
    }

    Some(Ok(()))
}

fn collect_expr_identifiers(expr: &Expr, identifiers: &mut HashSet<String>) {
    match expr {
        Expr::Literal(_) => {}
        Expr::Ident(name) => {
            identifiers.insert(name.clone());
        }
        Expr::Unary { expr, .. } => collect_expr_identifiers(expr, identifiers),
        Expr::Binary { left, right, .. } => {
            collect_expr_identifiers(left, identifiers);
            collect_expr_identifiers(right, identifiers);
        }
        Expr::Ternary {
            condition,
            then_expr,
            else_expr,
        } => {
            collect_expr_identifiers(condition, identifiers);
            collect_expr_identifiers(then_expr, identifiers);
            collect_expr_identifiers(else_expr, identifiers);
        }
    }
}

fn parse_port_actual(input: &str) -> Result<PortActual, FrontendError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(FrontendError::new("expected port actual", 0));
    }

    if trimmed.starts_with('{') && trimmed.ends_with('}') && is_wrapped_by(trimmed, '{', '}') {
        let inner = trimmed[1..trimmed.len() - 1].trim();
        if inner.is_empty() {
            return Err(FrontendError::new(
                "empty concatenation is not supported",
                0,
            ));
        }
        let parts = split_top_level(inner, ',')?;
        if parts.len() == 1
            && let Some((count_text, body_text)) = split_repetition_count_and_value(inner)?
        {
            return Ok(PortActual::Repeat {
                count: parse_expression(count_text).map_err(map_eval_error)?,
                value: Box::new(parse_port_actual(body_text)?),
            });
        }
        let mut parsed_parts = Vec::new();
        for part in parts {
            parsed_parts.push(parse_port_actual(part)?);
        }
        return Ok(PortActual::Concat(parsed_parts));
    }

    if let Some((signal, inner)) = split_signal_suffix(trimmed)? {
        if let Some((msb, lsb)) = split_top_level_once(inner, ':')? {
            return Ok(PortActual::PartSelect {
                signal,
                msb: parse_expression(msb).map_err(map_eval_error)?,
                lsb: parse_expression(lsb).map_err(map_eval_error)?,
            });
        }
        return Ok(PortActual::BitSelect {
            signal,
            index: parse_expression(inner).map_err(map_eval_error)?,
        });
    }

    if parse_signal_path(trimmed)?.is_some() {
        return Ok(PortActual::Signal(trimmed.to_string()));
    }

    match parse_expression(trimmed) {
        Ok(expr) => Ok(match expr {
            Expr::Ident(name) => PortActual::Signal(name),
            other => PortActual::Expression(other),
        }),
        Err(err) => {
            parse_rich_expression_text(trimmed).map_err(|rich_err| {
                if contains_rich_expression_syntax(trimmed) {
                    rich_err
                } else {
                    map_eval_error(err)
                }
            })?;
            Ok(PortActual::ExpressionText(trimmed.to_string()))
        }
    }
}

fn parse_assignment_target(input: &str) -> Result<AssignmentTarget, FrontendError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(FrontendError::new("expected assignment target", 0));
    }

    if trimmed.starts_with('{') && trimmed.ends_with('}') && is_wrapped_by(trimmed, '{', '}') {
        let inner = trimmed[1..trimmed.len() - 1].trim();
        if inner.is_empty() {
            return Err(FrontendError::new(
                "empty assignment-target concatenation is not supported",
                0,
            ));
        }

        let parts = split_top_level(inner, ',')?;
        let mut parsed_parts = Vec::new();
        for part in parts {
            parsed_parts.push(parse_assignment_target(part)?);
        }
        return Ok(AssignmentTarget::Concat(parsed_parts));
    }

    if let Some((signal, inner)) = split_signal_suffix(trimmed)? {
        if let Some((msb, lsb)) = split_top_level_once(inner, ':')? {
            return Ok(AssignmentTarget::PartSelect {
                signal,
                msb: parse_expression(msb).map_err(map_eval_error)?,
                lsb: parse_expression(lsb).map_err(map_eval_error)?,
            });
        }
        return Ok(AssignmentTarget::BitSelect {
            signal,
            index: parse_expression(inner).map_err(map_eval_error)?,
        });
    }

    if parse_signal_path(trimmed)?.is_some() {
        return Ok(AssignmentTarget::Signal(trimmed.to_string()));
    }

    Err(FrontendError::new(
        format!("unsupported assignment target '{}'", trimmed),
        0,
    ))
}

fn parse_value_expr(input: &str) -> Result<ValueExpr, FrontendError> {
    Ok(ValueExpr::from(parse_port_actual(input)?))
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParameterOverrideValue {
    Const(Expr),
    Syntax(String),
}

fn parse_parameter_override_value(input: &str) -> Result<ParameterOverrideValue, FrontendError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(FrontendError::new("expected parameter override value", 0));
    }

    match parse_expression(trimmed) {
        Ok(expr) => Ok(ParameterOverrideValue::Const(expr)),
        Err(err) => {
            parse_rich_expression_text(trimmed).map_err(|rich_err| {
                if contains_rich_expression_syntax(trimmed) {
                    rich_err
                } else {
                    map_eval_error(err)
                }
            })?;
            Ok(ParameterOverrideValue::Syntax(trimmed.to_string()))
        }
    }
}

fn contains_rich_expression_syntax(input: &str) -> bool {
    input.contains('[') || input.contains('{')
}

fn parse_rich_expression_text(input: &str) -> Result<(), FrontendError> {
    let trimmed = input.trim();
    if !contains_rich_expression_syntax(trimmed) {
        return Err(FrontendError::new(
            format!("unsupported expression '{}'", trimmed),
            0,
        ));
    }
    if trimmed.starts_with('{') && trimmed.ends_with('}') && is_wrapped_by(trimmed, '{', '}') {
        parse_port_actual(trimmed)?;
        return Ok(());
    }

    let normalized = normalize_rich_expression_text(trimmed)?;
    parse_expression(&normalized).map_err(|err| {
        FrontendError::new(
            format!("failed to parse expression '{}': {}", trimmed, err.message),
            err.position,
        )
    })?;
    Ok(())
}

fn normalize_rich_expression_text(input: &str) -> Result<String, FrontendError> {
    let mut normalized = String::new();
    let mut cursor = 0usize;
    while cursor < input.len() {
        let ch = input[cursor..]
            .chars()
            .next()
            .ok_or_else(|| FrontendError::new("invalid expression cursor", cursor))?;
        match ch {
            '{' => {
                let next = skip_balanced_delimiter(input, cursor, '{', '}')?;
                parse_port_actual(&input[cursor..next])?;
                cursor = next;
                normalized.push('0');
            }
            '[' => {
                let next = skip_balanced_delimiter(input, cursor, '[', ']')?;
                validate_selector_text(input, cursor, next)?;
                cursor = next;
            }
            _ => {
                normalized.push(ch);
                cursor += ch.len_utf8();
            }
        }
    }
    Ok(normalized)
}

fn validate_selector_text(input: &str, start: usize, end: usize) -> Result<(), FrontendError> {
    let inner_end = end.saturating_sub(']'.len_utf8());
    let inner = input[start + '['.len_utf8()..inner_end].trim();
    if inner.is_empty() {
        return Err(FrontendError::new(
            format!("empty bracket selector in '{}'", input),
            start,
        ));
    }
    if let Some((msb, lsb)) = split_top_level_once(inner, ':')? {
        parse_expression(msb).map_err(map_eval_error)?;
        parse_expression(lsb).map_err(map_eval_error)?;
    } else {
        parse_expression(inner).map_err(map_eval_error)?;
    }
    Ok(())
}

fn skip_balanced_delimiter(
    input: &str,
    start: usize,
    open: char,
    close: char,
) -> Result<usize, FrontendError> {
    let mut depth = 0usize;
    for (offset, ch) in input[start..].char_indices() {
        match ch {
            c if c == open => depth += 1,
            c if c == close => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Ok(start + offset + ch.len_utf8());
                }
            }
            _ => {}
        }
    }
    Err(FrontendError::new(
        format!("unbalanced '{}' in expression '{}'", open, input),
        start,
    ))
}

fn validate_expression_text_identifiers(
    text: &str,
    visible_names: &HashSet<String>,
    visible_types: &HashMap<String, DeclType>,
    symbols: &HashMap<String, i64>,
) -> Result<(), FrontendError> {
    for identifier in collect_expression_text_identifiers(text) {
        validate_known_identifier(&identifier, visible_names, visible_types, symbols)?;
    }
    Ok(())
}

fn collect_expression_text_identifiers(text: &str) -> HashSet<String> {
    let mut identifiers = HashSet::new();
    let mut cursor = 0usize;
    while cursor < text.len() {
        let Some(ch) = text[cursor..].chars().next() else {
            break;
        };
        if !is_ident_start(ch) {
            cursor += ch.len_utf8();
            continue;
        }

        let previous = text[..cursor].chars().next_back();
        let member_segment = previous == Some('.');
        let based_literal_segment = previous == Some('\'');
        let (mut identifier, next) = parse_signal_identifier_segment(text, cursor)
            .map(|(segment, next)| (segment.to_string(), next))
            .unwrap_or_else(|| (String::new(), cursor + ch.len_utf8()));
        cursor = next;
        if text[cursor..].starts_with("::")
            && let Some((segment, next)) = parse_signal_identifier_segment(text, cursor + 2)
        {
            identifier.push_str("::");
            identifier.push_str(segment);
            cursor = next;
        }
        if !member_segment && !based_literal_segment && !identifier.is_empty() {
            identifiers.insert(identifier);
        }
    }
    identifiers
}

fn split_repetition_count_and_value(input: &str) -> Result<Option<(&str, &str)>, FrontendError> {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut ternary_depth = 0usize;

    for (idx, ch) in input.char_indices() {
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '{' if paren_depth == 0
                && bracket_depth == 0
                && brace_depth == 0
                && ternary_depth == 0 =>
            {
                let count = input[..idx].trim();
                let body = input[idx..].trim();
                if count.is_empty() {
                    return Ok(None);
                }
                if !is_wrapped_by(body, '{', '}') {
                    return Err(FrontendError::new(
                        format!("malformed repetition actual '{}'", input),
                        0,
                    ));
                }
                return Ok(Some((count, body)));
            }
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            '?' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                ternary_depth += 1;
            }
            ':' if paren_depth == 0
                && bracket_depth == 0
                && brace_depth == 0
                && !is_double_colon_at(input, idx)
                && ternary_depth > 0 =>
            {
                ternary_depth -= 1;
            }
            _ => {}
        }
    }

    Ok(None)
}

fn split_signal_suffix(input: &str) -> Result<Option<(String, &str)>, FrontendError> {
    let trimmed = input.trim();
    if !trimmed.ends_with(']') {
        return Ok(None);
    }

    let mut bracket_depth = 0usize;
    let mut suffix_start = None;
    for (idx, ch) in trimmed.char_indices().rev() {
        match ch {
            ']' => bracket_depth += 1,
            '[' => {
                bracket_depth = bracket_depth.saturating_sub(1);
                if bracket_depth == 0 {
                    suffix_start = Some(idx);
                    break;
                }
            }
            _ => {}
        }
    }

    let Some(suffix_start) = suffix_start else {
        return Err(FrontendError::new(
            format!("unbalanced bracket suffix in '{}'", trimmed),
            0,
        ));
    };

    let signal = trimmed[..suffix_start].trim();
    let inner = trimmed[suffix_start + 1..trimmed.len() - 1].trim();
    if signal.is_empty() || parse_signal_path(signal)?.is_none() {
        return Ok(None);
    }
    if inner.is_empty() {
        return Err(FrontendError::new(
            format!("empty bracket selector in '{}'", trimmed),
            0,
        ));
    }

    Ok(Some((signal.to_string(), inner)))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SignalPath {
    root: String,
    ops: Vec<SignalPathOp>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SignalPathOp {
    Member(String),
    Index(Expr),
}

fn parse_signal_path(input: &str) -> Result<Option<SignalPath>, FrontendError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let mut cursor = 0usize;
    let Some((root, next)) = parse_signal_identifier_segment(trimmed, cursor) else {
        return Ok(None);
    };
    cursor = next;
    let mut ops = Vec::new();

    while cursor < trimmed.len() {
        let rest = &trimmed[cursor..];
        if rest.starts_with('.') {
            cursor += 1;
            let Some((member, next)) = parse_signal_identifier_segment(trimmed, cursor) else {
                return Err(FrontendError::new(
                    format!("expected member name in signal path '{}'", trimmed),
                    cursor,
                ));
            };
            ops.push(SignalPathOp::Member(member.to_string()));
            cursor = next;
            continue;
        }
        if rest.starts_with('[') {
            let (index_text, next) = extract_bracket_segment(trimmed, cursor)?;
            let index = parse_expression(index_text).map_err(map_eval_error)?;
            ops.push(SignalPathOp::Index(index));
            cursor = next;
            continue;
        }
        return Ok(None);
    }

    Ok(Some(SignalPath {
        root: root.to_string(),
        ops,
    }))
}

fn parse_signal_identifier_segment(input: &str, start: usize) -> Option<(&str, usize)> {
    let rest = input.get(start..)?;
    let mut chars = rest.char_indices();
    let (_, first) = chars.next()?;
    if !is_ident_start(first) {
        return None;
    }

    let mut end = start + first.len_utf8();
    for (offset, ch) in chars {
        if !is_ident_continue(ch) {
            break;
        }
        end = start + offset + ch.len_utf8();
    }

    Some((&input[start..end], end))
}

fn extract_bracket_segment(input: &str, start: usize) -> Result<(&str, usize), FrontendError> {
    let bytes = input.as_bytes();
    if bytes.get(start) != Some(&b'[') {
        return Err(FrontendError::new(
            format!("expected '[' in signal path '{}'", input),
            start,
        ));
    }

    let mut depth = 0usize;
    let mut end = None;
    for (offset, ch) in input[start..].char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    end = Some(start + offset);
                    break;
                }
            }
            _ => {}
        }
    }

    let Some(end) = end else {
        return Err(FrontendError::new(
            format!("unbalanced bracket expression in signal path '{}'", input),
            start,
        ));
    };
    let inner = input[start + 1..end].trim();
    if inner.is_empty() {
        return Err(FrontendError::new(
            format!("empty bracket selector in '{}'", input),
            start,
        ));
    }
    Ok((inner, end + 1))
}

fn identifier_root(input: &str) -> &str {
    input
        .find(['.', '['])
        .map(|index| &input[..index])
        .unwrap_or(input)
}

fn is_wrapped_by(input: &str, open: char, close: char) -> bool {
    if !input.starts_with(open) || !input.ends_with(close) {
        return false;
    }

    let mut depth = 0usize;
    for (idx, ch) in input.char_indices() {
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth = depth.saturating_sub(1);
            if depth == 0 && idx + ch.len_utf8() != input.len() {
                return false;
            }
        }
    }
    depth == 0
}

fn split_top_level(input: &str, delimiter: char) -> Result<Vec<&str>, FrontendError> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut ternary_depth = 0usize;

    for (idx, ch) in input.char_indices() {
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            '?' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                ternary_depth += 1;
            }
            ':' if paren_depth == 0
                && bracket_depth == 0
                && brace_depth == 0
                && !is_double_colon_at(input, idx)
                && ternary_depth > 0 =>
            {
                ternary_depth -= 1;
            }
            _ => {}
        }

        if ch == delimiter
            && paren_depth == 0
            && bracket_depth == 0
            && brace_depth == 0
            && ternary_depth == 0
            && !(delimiter == ':' && is_double_colon_at(input, idx))
        {
            parts.push(input[start..idx].trim());
            start = idx + ch.len_utf8();
        }
    }

    parts.push(input[start..].trim());
    if parts.iter().any(|part| part.is_empty()) {
        return Err(FrontendError::new(
            format!("empty top-level segment in '{}'", input),
            0,
        ));
    }
    Ok(parts)
}

fn split_top_level_once(
    input: &str,
    delimiter: char,
) -> Result<Option<(&str, &str)>, FrontendError> {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut ternary_depth = 0usize;

    for (idx, ch) in input.char_indices() {
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            '?' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                ternary_depth += 1;
            }
            ':' if paren_depth == 0
                && bracket_depth == 0
                && brace_depth == 0
                && !is_double_colon_at(input, idx)
                && ternary_depth > 0 =>
            {
                ternary_depth -= 1;
            }
            _ => {}
        }

        if ch == delimiter
            && paren_depth == 0
            && bracket_depth == 0
            && brace_depth == 0
            && ternary_depth == 0
            && !(delimiter == ':' && is_double_colon_at(input, idx))
        {
            let left = input[..idx].trim();
            let right = input[idx + ch.len_utf8()..].trim();
            if left.is_empty() || right.is_empty() {
                return Err(FrontendError::new(
                    format!("empty top-level segment in '{}'", input),
                    0,
                ));
            }
            return Ok(Some((left, right)));
        }
    }

    Ok(None)
}

fn is_double_colon_at(input: &str, idx: usize) -> bool {
    let bytes = input.as_bytes();
    bytes.get(idx) == Some(&b':')
        && (bytes.get(idx + 1) == Some(&b':') || idx > 0 && bytes.get(idx - 1) == Some(&b':'))
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenKind {
    Ident(String),
    Number(String),
    Symbol(char),
    Operator(&'static str),
    End,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token {
    kind: TokenKind,
    start: usize,
    end: usize,
}

struct Lexer<'a> {
    input: &'a str,
    index: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, index: 0 }
    }

    fn tokenize(mut self) -> Result<Vec<Token>, FrontendError> {
        let mut tokens = Vec::new();
        loop {
            self.skip_ws_and_comments()?;
            let start = self.index;
            let Some(ch) = self.peek_char() else {
                tokens.push(Token {
                    kind: TokenKind::End,
                    start,
                    end: start,
                });
                break;
            };

            let token = if is_ident_start(ch) {
                self.lex_identifier()
            } else if ch.is_ascii_digit() {
                self.lex_number()
            } else {
                self.lex_punctuation()?
            };
            tokens.push(token);
        }

        Ok(tokens)
    }

    fn skip_ws_and_comments(&mut self) -> Result<(), FrontendError> {
        loop {
            while matches!(self.peek_char(), Some(ch) if ch.is_ascii_whitespace()) {
                self.bump_char();
            }

            if self.peek_char() == Some('/') && self.peek_next_char() == Some('/') {
                while let Some(ch) = self.peek_char() {
                    self.bump_char();
                    if ch == '\n' {
                        break;
                    }
                }
                continue;
            }

            if self.peek_char() == Some('/') && self.peek_next_char() == Some('*') {
                let start = self.index;
                self.bump_char();
                self.bump_char();
                loop {
                    match (self.peek_char(), self.peek_next_char()) {
                        (Some('*'), Some('/')) => {
                            self.bump_char();
                            self.bump_char();
                            break;
                        }
                        (Some(_), _) => {
                            self.bump_char();
                        }
                        (None, _) => {
                            return Err(FrontendError::new("unterminated block comment", start));
                        }
                    }
                }
                continue;
            }

            break;
        }

        Ok(())
    }

    fn lex_identifier(&mut self) -> Token {
        let start = self.index;
        let mut text = String::new();
        while let Some(ch) = self.peek_char() {
            if !is_ident_continue(ch) {
                break;
            }
            text.push(ch);
            self.bump_char();
        }
        Token {
            kind: TokenKind::Ident(text),
            start,
            end: self.index,
        }
    }

    fn lex_number(&mut self) -> Token {
        let start = self.index;
        let mut text = String::new();
        while let Some(ch) = self.peek_char() {
            if !(ch.is_ascii_digit() || ch == '_') {
                break;
            }
            text.push(ch);
            self.bump_char();
        }

        if self.peek_char() == Some('\'') {
            text.push('\'');
            self.bump_char();
            if matches!(self.peek_char(), Some('s' | 'S')) {
                let ch = self.peek_char().unwrap_or('s');
                text.push(ch);
                self.bump_char();
            }
            if let Some(ch) = self.peek_char() {
                text.push(ch);
                self.bump_char();
            }
            while let Some(ch) = self.peek_char() {
                if !(ch.is_ascii_alphanumeric() || ch == '_') {
                    break;
                }
                text.push(ch);
                self.bump_char();
            }
        }

        Token {
            kind: TokenKind::Number(text),
            start,
            end: self.index,
        }
    }

    fn lex_punctuation(&mut self) -> Result<Token, FrontendError> {
        let start = self.index;
        let Some(ch) = self.peek_char() else {
            return Err(FrontendError::new("unexpected end of input", start));
        };

        let token = match ch {
            '<' => {
                self.bump_char();
                if self.peek_char() == Some('=') {
                    self.bump_char();
                    TokenKind::Operator("<=")
                } else if self.peek_char() == Some('<') {
                    self.bump_char();
                    TokenKind::Operator("<<")
                } else {
                    TokenKind::Symbol('<')
                }
            }
            '>' => {
                self.bump_char();
                if self.peek_char() == Some('=') {
                    self.bump_char();
                    TokenKind::Operator(">=")
                } else if self.peek_char() == Some('>') {
                    self.bump_char();
                    TokenKind::Operator(">>")
                } else {
                    TokenKind::Symbol('>')
                }
            }
            '=' => {
                self.bump_char();
                if self.peek_char() == Some('=') {
                    self.bump_char();
                    TokenKind::Operator("==")
                } else {
                    TokenKind::Symbol('=')
                }
            }
            '!' => {
                self.bump_char();
                if self.peek_char() == Some('=') {
                    self.bump_char();
                    TokenKind::Operator("!=")
                } else {
                    TokenKind::Symbol('!')
                }
            }
            '&' => {
                self.bump_char();
                if self.peek_char() == Some('&') {
                    self.bump_char();
                    TokenKind::Operator("&&")
                } else {
                    TokenKind::Symbol('&')
                }
            }
            '|' => {
                self.bump_char();
                if self.peek_char() == Some('|') {
                    self.bump_char();
                    TokenKind::Operator("||")
                } else {
                    TokenKind::Symbol('|')
                }
            }
            _ => {
                self.bump_char();
                TokenKind::Symbol(ch)
            }
        };

        Ok(Token {
            kind: token,
            start,
            end: self.index,
        })
    }

    fn peek_char(&self) -> Option<char> {
        self.input.get(self.index..)?.chars().next()
    }

    fn peek_next_char(&self) -> Option<char> {
        let mut chars = self.input.get(self.index..)?.chars();
        chars.next()?;
        chars.next()
    }

    fn bump_char(&mut self) {
        if let Some(ch) = self.peek_char() {
            self.index += ch.len_utf8();
        }
    }
}

struct Parser<'a> {
    input: &'a str,
    tokens: Vec<Token>,
    index: usize,
    global_type_aliases: HashMap<String, DataType>,
    package_type_aliases: HashMap<String, HashMap<String, DataType>>,
    package_constant_names: HashMap<String, HashSet<String>>,
    type_aliases: HashMap<String, DataType>,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str, tokens: Vec<Token>) -> Self {
        Self {
            input,
            tokens,
            index: 0,
            global_type_aliases: HashMap::new(),
            package_type_aliases: HashMap::new(),
            package_constant_names: HashMap::new(),
            type_aliases: HashMap::new(),
        }
    }

    fn parse_design(mut self) -> Result<Design, FrontendError> {
        let mut typedefs = Vec::new();
        let mut packages = Vec::new();
        let mut modules = Vec::new();
        while !self.is_end() {
            if self.peek_keyword("typedef") {
                self.type_aliases = self.global_type_aliases.clone();
                let typedef_decl = self.parse_typedef_decl()?;
                self.global_type_aliases
                    .insert(typedef_decl.name.clone(), typedef_decl.data_type.clone());
                typedefs.push(typedef_decl);
            } else if self.peek_keyword("package") {
                packages.push(self.parse_package_decl()?);
            } else {
                modules.push(self.parse_module()?);
            }
        }
        Ok(Design {
            typedefs,
            packages,
            modules,
        })
    }

    fn parse_module(&mut self) -> Result<Module, FrontendError> {
        self.type_aliases = self.global_type_aliases.clone();
        self.expect_keyword("module")?;
        let name = self.expect_identifier()?;
        let mut header_imports = self.parse_header_imports()?;
        let parameters = if self.consume_symbol('#') {
            self.expect_symbol('(')?;
            self.parse_parameter_list(')')?
        } else {
            Vec::new()
        };
        header_imports.extend(self.parse_header_imports()?);

        let ports = if self.consume_symbol('(') {
            self.parse_port_list()?
        } else {
            Vec::new()
        };

        self.expect_symbol(';')?;
        let mut items = Vec::new();
        while !self.peek_keyword("endmodule") {
            items.extend(self.parse_item_group(false)?);
        }
        self.expect_keyword("endmodule")?;
        self.type_aliases = self.global_type_aliases.clone();

        Ok(Module {
            name,
            header_imports,
            parameters,
            ports,
            items,
        })
    }

    fn parse_header_imports(&mut self) -> Result<Vec<ImportDecl>, FrontendError> {
        let mut imports = Vec::new();
        while self.peek_keyword("import") {
            imports.push(self.parse_import_decl()?);
        }
        Ok(imports)
    }

    fn parse_parameter_list(
        &mut self,
        terminator: char,
    ) -> Result<Vec<ParameterDecl>, FrontendError> {
        self.parse_parameter_decl_sequence(terminator)
    }

    fn parse_parameter_decl(
        &mut self,
        flavor: ParameterFlavor,
        expr_terminators: &[char],
    ) -> Result<ParameterDecl, FrontendError> {
        let data_type = self.parse_optional_data_type()?;
        let name = self.expect_identifier()?;
        let value = if self.consume_symbol('=') {
            Some(self.parse_expression_until(expr_terminators)?)
        } else {
            None
        };

        Ok(ParameterDecl {
            flavor,
            data_type,
            name,
            value,
        })
    }

    fn parse_port_list(&mut self) -> Result<Vec<PortDecl>, FrontendError> {
        let mut ports = Vec::new();
        if self.consume_symbol(')') {
            return Ok(ports);
        }

        loop {
            let direction = self.parse_port_direction()?;
            let data_type = self.parse_optional_data_type()?;
            let packed_range = self.parse_optional_packed_range()?;

            loop {
                let name = self.expect_identifier()?;
                let unpacked_dimensions = self.parse_unpacked_dimensions()?;
                ports.push(PortDecl {
                    direction: direction.clone(),
                    data_type: data_type.clone(),
                    packed_range: packed_range.clone(),
                    unpacked_dimensions,
                    name,
                });

                if !self.consume_symbol(',') {
                    break;
                }
                if self.peek_symbol(')') || self.peek_direction_keyword() {
                    break;
                }
            }

            if self.peek_symbol(')') {
                break;
            }
        }

        self.expect_symbol(')')?;
        Ok(ports)
    }

    fn parse_item_group(
        &mut self,
        allow_generate_constructs: bool,
    ) -> Result<Vec<ModuleItem>, FrontendError> {
        if self.peek_keyword("parameter") || self.peek_keyword("localparam") {
            return self.parse_parameter_items();
        }
        if self.peek_keyword("import") {
            if allow_generate_constructs {
                return Err(FrontendError::new(
                    "import declarations inside generate bodies are not yet supported",
                    self.current().start,
                ));
            }
            return Ok(vec![ModuleItem::ImportDecl(self.parse_import_decl()?)]);
        }
        if self.peek_keyword("typedef") {
            if allow_generate_constructs {
                return Err(FrontendError::new(
                    "typedef declarations inside generate bodies are not yet supported",
                    self.current().start,
                ));
            }
            return Ok(vec![ModuleItem::TypedefDecl(self.parse_typedef_decl()?)]);
        }
        if self.peek_keyword("genvar") {
            return Ok(vec![ModuleItem::GenvarDecl(self.parse_genvar_decl()?)]);
        }
        if matches!(self.current_ident(), Some(value) if is_data_type_keyword(value))
            || self.peek_keyword("enum")
            || self.peek_keyword("union")
            || self.peek_keyword("struct")
            || self.peek_typedef_name()
            || self.peek_package_qualified_type()
        {
            return self.parse_net_decls();
        }
        if self.peek_keyword("assign") {
            return Ok(vec![ModuleItem::ContinuousAssign(
                self.parse_continuous_assign()?,
            )]);
        }
        if self.looks_like_module_instantiation() {
            return self.parse_module_instantiations();
        }
        if self.peek_keyword("always_comb")
            || self.peek_keyword("always")
            || self.peek_keyword("always_ff")
            || self.peek_keyword("always_latch")
        {
            return Ok(vec![ModuleItem::ProceduralBlock(
                self.parse_procedural_block()?,
            )]);
        }
        if self.peek_keyword("generate") {
            return Ok(vec![ModuleItem::GenerateRegion(
                self.parse_generate_region()?,
            )]);
        }
        if allow_generate_constructs && self.peek_keyword("if") {
            return Ok(vec![ModuleItem::GenerateIf(self.parse_generate_if()?)]);
        }
        if allow_generate_constructs && self.peek_keyword("for") {
            return Ok(vec![ModuleItem::GenerateFor(self.parse_generate_for()?)]);
        }

        Err(FrontendError::new(
            format!(
                "unsupported module item starting with {}",
                self.describe_current()
            ),
            self.current().start,
        ))
    }

    fn parse_parameter_items(&mut self) -> Result<Vec<ModuleItem>, FrontendError> {
        Ok(self
            .parse_parameter_decl_sequence(';')?
            .into_iter()
            .map(ModuleItem::ParameterDecl)
            .collect())
    }

    fn parse_parameter_decl_sequence(
        &mut self,
        terminator: char,
    ) -> Result<Vec<ParameterDecl>, FrontendError> {
        let mut params = Vec::new();
        if self.consume_symbol(terminator) {
            return Ok(params);
        }

        let mut current_flavor: Option<ParameterFlavor> = None;
        loop {
            if self.consume_keyword("parameter") {
                current_flavor = Some(ParameterFlavor::Parameter);
            } else if self.consume_keyword("localparam") {
                current_flavor = Some(ParameterFlavor::Localparam);
            }

            let Some(flavor) = current_flavor.clone() else {
                return Err(FrontendError::new(
                    "expected parameter/localparam declaration",
                    self.current().start,
                ));
            };

            params.push(self.parse_parameter_decl(flavor, &[',', terminator])?);
            if !self.consume_symbol(',') {
                break;
            }
        }

        self.expect_symbol(terminator)?;
        Ok(params)
    }

    fn parse_genvar_decl(&mut self) -> Result<GenvarDecl, FrontendError> {
        self.expect_keyword("genvar")?;
        let mut names = vec![self.expect_identifier()?];
        while self.consume_symbol(',') {
            names.push(self.expect_identifier()?);
        }
        self.expect_symbol(';')?;
        Ok(GenvarDecl { names })
    }

    fn parse_package_decl(&mut self) -> Result<PackageDecl, FrontendError> {
        self.type_aliases = self.global_type_aliases.clone();
        self.expect_keyword("package")?;
        let name = self.expect_identifier()?;
        self.expect_symbol(';')?;

        let mut typedefs = Vec::new();
        let mut package_aliases = HashMap::new();
        let mut constants = Vec::new();
        let mut constant_names = HashSet::new();
        while !self.peek_keyword("endpackage") {
            if self.peek_keyword("typedef") {
                let typedef_decl = self.parse_typedef_decl()?;
                package_aliases.insert(typedef_decl.name.clone(), typedef_decl.data_type.clone());
                typedefs.push(typedef_decl);
            } else if self.peek_keyword("parameter") || self.peek_keyword("localparam") {
                for constant in self.parse_parameter_decl_sequence(';')? {
                    constant_names.insert(constant.name.clone());
                    constants.push(constant);
                }
            } else {
                return Err(FrontendError::new(
                    format!(
                        "unsupported package item starting with {}",
                        self.describe_current()
                    ),
                    self.current().start,
                ));
            }
        }

        self.expect_keyword("endpackage")?;
        self.package_type_aliases
            .insert(name.clone(), package_aliases);
        self.package_constant_names
            .insert(name.clone(), constant_names);
        self.type_aliases = self.global_type_aliases.clone();

        Ok(PackageDecl {
            name,
            typedefs,
            constants,
        })
    }

    fn parse_typedef_decl(&mut self) -> Result<TypedefDecl, FrontendError> {
        self.expect_keyword("typedef")?;
        let mut data_type = self.parse_optional_data_type()?.ok_or_else(|| {
            FrontendError::new("expected typedef data type", self.current().start)
        })?;
        if let Some(packed_range) = self.parse_optional_packed_range()? {
            data_type = DataType::Packed {
                data_type: Box::new(data_type),
                packed_range,
            };
        }
        let name = self.expect_identifier()?;
        self.expect_symbol(';')?;
        self.type_aliases.insert(name.clone(), data_type.clone());
        Ok(TypedefDecl { name, data_type })
    }

    fn parse_import_decl(&mut self) -> Result<ImportDecl, FrontendError> {
        self.expect_keyword("import")?;
        let package = self.expect_identifier()?;
        self.expect_symbol(':')?;
        self.expect_symbol(':')?;
        let imported_name = if self.consume_symbol('*') {
            None
        } else {
            Some(self.expect_identifier()?)
        };
        self.expect_symbol(';')?;

        let aliases = self
            .package_type_aliases
            .get(&package)
            .cloned()
            .ok_or_else(|| {
                FrontendError::new(
                    format!("unknown package '{}' in import declaration", package),
                    0,
                )
            })?;
        let constant_names = self
            .package_constant_names
            .get(&package)
            .cloned()
            .unwrap_or_default();
        match &imported_name {
            None => {
                self.type_aliases.extend(aliases);
            }
            Some(name) => {
                let mut found = false;
                if let Some(data_type) = aliases.get(name).cloned() {
                    self.type_aliases.insert(name.clone(), data_type);
                    found = true;
                }
                if constant_names.contains(name) {
                    found = true;
                }
                if !found {
                    return Err(FrontendError::new(
                        format!(
                            "unknown import '{}::{}' in import declaration",
                            package, name
                        ),
                        0,
                    ));
                }
            }
        }

        let wildcard = imported_name.is_none();
        Ok(ImportDecl {
            package,
            imported_name,
            wildcard,
        })
    }

    fn parse_net_decls(&mut self) -> Result<Vec<ModuleItem>, FrontendError> {
        let (kind, data_type) = if self.consume_keyword("logic") {
            (
                NetKind::Logic,
                Some(DataType::Builtin {
                    keyword: "logic".to_string(),
                }),
            )
        } else if self.consume_keyword("wire") {
            (
                NetKind::Wire,
                Some(DataType::Builtin {
                    keyword: "wire".to_string(),
                }),
            )
        } else if self.consume_keyword("reg") {
            (
                NetKind::Reg,
                Some(DataType::Builtin {
                    keyword: "reg".to_string(),
                }),
            )
        } else if matches!(self.current_ident(), Some(value) if is_data_type_keyword(value)) {
            (
                NetKind::Typed,
                Some(
                    self.parse_optional_data_type()?
                        .ok_or_else(|| FrontendError::new("expected builtin data type", 0))?,
                ),
            )
        } else if self.peek_keyword("enum") {
            (
                NetKind::Typed,
                Some(
                    self.parse_optional_data_type()?
                        .ok_or_else(|| FrontendError::new("expected enum data type", 0))?,
                ),
            )
        } else if self.peek_keyword("union") {
            (
                NetKind::Typed,
                Some(
                    self.parse_optional_data_type()?
                        .ok_or_else(|| FrontendError::new("expected union data type", 0))?,
                ),
            )
        } else if self.peek_keyword("struct") {
            (
                NetKind::Typed,
                Some(
                    self.parse_optional_data_type()?
                        .ok_or_else(|| FrontendError::new("expected struct data type", 0))?,
                ),
            )
        } else if self.peek_typedef_name() {
            (
                NetKind::Typed,
                Some(
                    self.parse_optional_data_type()?
                        .ok_or_else(|| FrontendError::new("expected named data type", 0))?,
                ),
            )
        } else if self.peek_package_qualified_type() {
            (
                NetKind::Typed,
                Some(self.parse_optional_data_type()?.ok_or_else(|| {
                    FrontendError::new("expected package-qualified data type", 0)
                })?),
            )
        } else {
            return Err(FrontendError::new(
                "expected net declaration",
                self.current().start,
            ));
        };

        let packed_range = self.parse_optional_packed_range()?;
        let mut declarations = Vec::new();
        loop {
            let name = self.expect_identifier()?;
            let unpacked_dimensions = self.parse_unpacked_dimensions()?;
            declarations.push(ModuleItem::NetDecl(NetDecl {
                kind: kind.clone(),
                data_type: data_type.clone(),
                packed_range: packed_range.clone(),
                unpacked_dimensions,
                name,
            }));
            if !self.consume_symbol(',') {
                break;
            }
        }
        self.expect_symbol(';')?;

        Ok(declarations)
    }

    fn parse_module_instantiations(&mut self) -> Result<Vec<ModuleItem>, FrontendError> {
        let module_name = self.expect_identifier()?;
        let parameter_overrides = if self.consume_symbol('#') {
            self.expect_symbol('(')?;
            self.parse_parameter_overrides()?
        } else {
            Vec::new()
        };

        let mut items = Vec::new();
        loop {
            let instance_name = self.expect_identifier()?;
            let instance_range = self.parse_optional_packed_range()?;
            self.expect_symbol('(')?;
            let port_connections = self.parse_port_connections()?;
            items.push(ModuleItem::ModuleInstantiation(ModuleInstantiation {
                module_name: module_name.clone(),
                instance_name,
                instance_range,
                parameter_overrides: parameter_overrides.clone(),
                port_connections,
            }));
            if !self.consume_symbol(',') {
                break;
            }
        }
        self.expect_symbol(';')?;
        Ok(items)
    }

    fn parse_parameter_overrides(&mut self) -> Result<Vec<ParameterOverride>, FrontendError> {
        let mut overrides = Vec::new();
        let mut saw_named = false;
        let mut saw_ordered = false;
        if self.consume_symbol(')') {
            return Ok(overrides);
        }

        loop {
            if self.consume_symbol('.') {
                if saw_ordered {
                    return Err(FrontendError::new(
                        "cannot mix positional and named parameter overrides",
                        self.current().start,
                    ));
                }
                saw_named = true;
                let name = self.expect_identifier()?;
                self.expect_symbol('(')?;
                let value_text = self.parse_raw_text_until(&[')'])?;
                let value = parse_parameter_override_value(&value_text)?;
                self.expect_symbol(')')?;
                match value {
                    ParameterOverrideValue::Const(value) => {
                        overrides.push(ParameterOverride::Named { name, value });
                    }
                    ParameterOverrideValue::Syntax(value) => {
                        overrides.push(ParameterOverride::NamedSyntax { name, value });
                    }
                }
            } else {
                if saw_named {
                    return Err(FrontendError::new(
                        "cannot mix positional and named parameter overrides",
                        self.current().start,
                    ));
                }
                saw_ordered = true;
                let value_text = self.parse_raw_text_until(&[',', ')'])?;
                match parse_parameter_override_value(&value_text)? {
                    ParameterOverrideValue::Const(value) => {
                        overrides.push(ParameterOverride::Ordered(value));
                    }
                    ParameterOverrideValue::Syntax(value) => {
                        overrides.push(ParameterOverride::OrderedSyntax(value));
                    }
                }
            }

            if !self.consume_symbol(',') {
                break;
            }
        }

        self.expect_symbol(')')?;
        Ok(overrides)
    }

    fn parse_port_connections(&mut self) -> Result<Vec<PortConnection>, FrontendError> {
        let mut connections = Vec::new();
        let mut saw_named_or_wildcard = false;
        let mut saw_ordered = false;
        if self.consume_symbol(')') {
            return Ok(connections);
        }

        loop {
            if self.consume_symbol('.') {
                if saw_ordered {
                    return Err(FrontendError::new(
                        "cannot mix positional and named port connections",
                        self.current().start,
                    ));
                }
                saw_named_or_wildcard = true;
                if self.consume_symbol('*') {
                    connections.push(PortConnection::Wildcard);
                } else {
                    let port = self.expect_identifier()?;
                    self.expect_symbol('(')?;
                    let actual = self.parse_optional_port_actual_until(&[')'])?;
                    self.expect_symbol(')')?;
                    connections.push(PortConnection::Named { port, actual });
                }
            } else {
                if saw_named_or_wildcard {
                    return Err(FrontendError::new(
                        "cannot mix positional and named port connections",
                        self.current().start,
                    ));
                }
                saw_ordered = true;
                let actual = self.parse_optional_port_actual_until(&[',', ')'])?;
                connections.push(PortConnection::Ordered { actual });
            }

            if !self.consume_symbol(',') {
                break;
            }
        }

        self.expect_symbol(')')?;
        Ok(connections)
    }

    fn parse_continuous_assign(&mut self) -> Result<ContinuousAssign, FrontendError> {
        self.expect_keyword("assign")?;
        let (target, kind) = self.parse_assignment_target_and_kind()?;
        if kind != AssignmentKind::Blocking {
            return Err(FrontendError::new(
                "continuous assign requires '=' assignment operator",
                self.current().start,
            ));
        }
        let value = self.parse_value_expr_until(&[';'])?;
        self.expect_symbol(';')?;
        Ok(ContinuousAssign { target, value })
    }

    fn parse_procedural_block(&mut self) -> Result<ProceduralBlock, FrontendError> {
        let (kind, event_control) = if self.consume_keyword("always_comb") {
            (ProceduralKind::AlwaysComb, None)
        } else if self.consume_keyword("always_latch") {
            (ProceduralKind::AlwaysLatch, None)
        } else if self.consume_keyword("always_ff") {
            (
                ProceduralKind::AlwaysFf,
                Some(self.parse_event_control_list()?),
            )
        } else {
            self.expect_keyword("always")?;
            self.expect_symbol('@')?;
            if self.consume_symbol('(') {
                self.expect_symbol('*')?;
                self.expect_symbol(')')?;
            } else {
                self.expect_symbol('*')?;
            }
            (ProceduralKind::AlwaysStar, None)
        };

        let statement = self.parse_statement()?;
        if kind == ProceduralKind::AlwaysFf
            && let Some(target) = first_blocking_assignment_target(&statement)
        {
            return Err(FrontendError::new(
                format!(
                    "always_ff block does not allow blocking assignment to '{}'",
                    target.describe()
                ),
                0,
            ));
        }
        Ok(ProceduralBlock {
            kind,
            event_control,
            statement,
        })
    }

    fn parse_event_control_list(&mut self) -> Result<Vec<EventControlItem>, FrontendError> {
        self.expect_symbol('@')?;
        self.expect_symbol('(')?;

        let mut items = Vec::new();
        loop {
            let edge = if self.consume_keyword("posedge") {
                Some(EventEdge::Posedge)
            } else if self.consume_keyword("negedge") {
                Some(EventEdge::Negedge)
            } else {
                None
            };
            let expr =
                self.parse_expression_until_with_keyword_terminators(&[',', ')'], &["or"])?;
            items.push(EventControlItem { edge, expr });

            if self.consume_symbol(',') || self.consume_keyword("or") {
                continue;
            }
            break;
        }

        self.expect_symbol(')')?;
        Ok(items)
    }

    fn parse_statement(&mut self) -> Result<Statement, FrontendError> {
        if self.consume_symbol(';') {
            return Ok(Statement::Empty);
        }
        if self.consume_keyword("begin") {
            return self.parse_statement_block();
        }
        if self.consume_keyword("if") {
            return self.parse_if_statement();
        }

        let (target, kind) = self.parse_assignment_target_and_kind()?;
        let value = self.parse_value_expr_until(&[';'])?;
        self.expect_symbol(';')?;
        Ok(Statement::Assignment {
            target,
            kind,
            value,
        })
    }

    fn parse_statement_block(&mut self) -> Result<Statement, FrontendError> {
        let label = self.parse_optional_label()?;
        let mut statements = Vec::new();
        while !self.peek_keyword("end") {
            statements.push(self.parse_statement()?);
        }
        self.expect_keyword("end")?;
        Ok(Statement::Block { label, statements })
    }

    fn parse_if_statement(&mut self) -> Result<Statement, FrontendError> {
        self.expect_symbol('(')?;
        let condition = self.parse_expression_until(&[')'])?;
        self.expect_symbol(')')?;
        let then_branch = self.parse_statement()?;
        let else_branch = if self.consume_keyword("else") {
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };
        Ok(Statement::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    fn parse_assignment_target_and_kind(
        &mut self,
    ) -> Result<(AssignmentTarget, AssignmentKind), FrontendError> {
        if self.is_end() {
            return Err(FrontendError::new(
                "expected assignment target, found end of input",
                0,
            ));
        }

        let start = self.current().start;
        let mut end = start;
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut brace_depth = 0usize;

        while !self.is_end() {
            let token = self.current();
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;

            if top_level {
                let kind = match token.kind {
                    TokenKind::Operator("<=") => Some(AssignmentKind::NonBlocking),
                    TokenKind::Symbol('=') => Some(AssignmentKind::Blocking),
                    _ => None,
                };
                if let Some(kind) = kind {
                    let text = self
                        .input
                        .get(start..end)
                        .unwrap_or_default()
                        .trim()
                        .to_string();
                    if text.is_empty() {
                        return Err(FrontendError::new("expected assignment target", start));
                    }
                    let target = parse_assignment_target(&text)?;
                    self.advance();
                    return Ok((target, kind));
                }
            }

            match token.kind {
                TokenKind::Symbol('(') => paren_depth += 1,
                TokenKind::Symbol(')') => paren_depth = paren_depth.saturating_sub(1),
                TokenKind::Symbol('[') => bracket_depth += 1,
                TokenKind::Symbol(']') => bracket_depth = bracket_depth.saturating_sub(1),
                TokenKind::Symbol('{') => brace_depth += 1,
                TokenKind::Symbol('}') => brace_depth = brace_depth.saturating_sub(1),
                _ => {}
            }

            end = token.end;
            self.advance();
        }

        Err(FrontendError::new("expected assignment operator", end))
    }

    fn parse_generate_region(&mut self) -> Result<GenerateRegion, FrontendError> {
        self.expect_keyword("generate")?;
        let mut items = Vec::new();
        while !self.peek_keyword("endgenerate") {
            items.extend(self.parse_item_group(true)?);
        }
        self.expect_keyword("endgenerate")?;
        Ok(GenerateRegion { items })
    }

    fn parse_generate_if(&mut self) -> Result<GenerateIf, FrontendError> {
        self.expect_keyword("if")?;
        self.expect_symbol('(')?;
        let condition = self.parse_expression_until(&[')'])?;
        self.expect_symbol(')')?;
        let (then_label, then_items) = self.parse_generate_body()?;
        let (else_label, else_items) = if self.consume_keyword("else") {
            let (label, items) = self.parse_generate_body()?;
            (label, items)
        } else {
            (None, Vec::new())
        };

        Ok(GenerateIf {
            condition,
            then_label,
            then_items,
            else_label,
            else_items,
        })
    }

    fn parse_generate_for(&mut self) -> Result<GenerateFor, FrontendError> {
        self.expect_keyword("for")?;
        self.expect_symbol('(')?;
        let declares_genvar = self.consume_keyword("genvar");
        let index_name = self.expect_identifier()?;
        self.expect_symbol('=')?;
        let init = self.parse_expression_until(&[';'])?;
        self.expect_symbol(';')?;
        let condition = self.parse_expression_until(&[';'])?;
        self.expect_symbol(';')?;
        let step_name = self.expect_identifier()?;
        if step_name != index_name {
            return Err(FrontendError::new(
                format!(
                    "generate-for step target '{}' does not match loop variable '{}'",
                    step_name, index_name
                ),
                self.current().start,
            ));
        }
        self.expect_symbol('=')?;
        let step = self.parse_expression_until(&[')'])?;
        self.expect_symbol(')')?;
        let (label, body_items) = self.parse_generate_body()?;

        Ok(GenerateFor {
            index_name,
            declares_genvar,
            init,
            condition,
            step,
            label,
            body_items,
        })
    }

    fn parse_generate_body(&mut self) -> Result<(Option<String>, Vec<ModuleItem>), FrontendError> {
        if self.consume_keyword("begin") {
            let label = self.parse_optional_label()?;
            let mut items = Vec::new();
            while !self.peek_keyword("end") {
                items.extend(self.parse_item_group(true)?);
            }
            self.expect_keyword("end")?;
            Ok((label, items))
        } else {
            Ok((None, self.parse_item_group(true)?))
        }
    }

    fn parse_optional_data_type(&mut self) -> Result<Option<DataType>, FrontendError> {
        if self.peek_keyword("enum") {
            return self.parse_enum_data_type().map(Some);
        }
        if self.peek_keyword("union") {
            return self.parse_union_data_type().map(Some);
        }
        if self.peek_keyword("struct") {
            return self.parse_struct_data_type().map(Some);
        }
        if self.peek_package_qualified_type() {
            return self.parse_package_qualified_data_type().map(Some);
        }

        let Some(keyword) = self.current_ident().map(str::to_string) else {
            return Ok(None);
        };
        if is_data_type_keyword(&keyword) {
            self.advance();
            Ok(Some(DataType::Builtin { keyword }))
        } else if let Some(data_type) = self.type_aliases.get(&keyword).cloned() {
            self.advance();
            Ok(Some(data_type))
        } else {
            Ok(None)
        }
    }

    fn parse_enum_data_type(&mut self) -> Result<DataType, FrontendError> {
        self.expect_keyword("enum")?;
        let mut base_type = self
            .parse_optional_data_type()?
            .unwrap_or_else(|| DataType::Builtin {
                keyword: "int".to_string(),
            });
        if let Some(packed_range) = self.parse_optional_packed_range()? {
            base_type = DataType::Packed {
                data_type: Box::new(base_type),
                packed_range,
            };
        }

        self.expect_symbol('{')?;
        let mut variants = Vec::new();
        loop {
            let name = self.expect_identifier()?;
            let value = if self.consume_symbol('=') {
                Some(self.parse_expression_until(&[',', '}'])?)
            } else {
                None
            };
            variants.push(EnumVariant { name, value });

            if self.consume_symbol('}') {
                break;
            }
            self.expect_symbol(',')?;
            if self.consume_symbol('}') {
                break;
            }
        }

        Ok(DataType::Enum {
            base_type: Box::new(base_type),
            variants,
        })
    }

    fn parse_union_data_type(&mut self) -> Result<DataType, FrontendError> {
        self.expect_keyword("union")?;
        let packed = self.consume_keyword("packed");
        self.expect_symbol('{')?;

        let mut fields = Vec::new();
        while !self.consume_symbol('}') {
            let data_type = self.parse_optional_data_type()?.ok_or_else(|| {
                FrontendError::new("expected union field data type", self.current().start)
            })?;
            let packed_range = self.parse_optional_packed_range()?;
            let mut names = vec![self.expect_identifier()?];
            while self.consume_symbol(',') {
                names.push(self.expect_identifier()?);
            }
            self.expect_symbol(';')?;
            fields.push(StructField {
                data_type,
                packed_range,
                names,
            });
        }

        Ok(DataType::Union { packed, fields })
    }

    fn parse_struct_data_type(&mut self) -> Result<DataType, FrontendError> {
        self.expect_keyword("struct")?;
        let packed = self.consume_keyword("packed");
        self.expect_symbol('{')?;

        let mut fields = Vec::new();
        while !self.consume_symbol('}') {
            let data_type = self.parse_optional_data_type()?.ok_or_else(|| {
                FrontendError::new("expected struct field data type", self.current().start)
            })?;
            let packed_range = self.parse_optional_packed_range()?;
            let mut names = vec![self.expect_identifier()?];
            while self.consume_symbol(',') {
                names.push(self.expect_identifier()?);
            }
            self.expect_symbol(';')?;
            fields.push(StructField {
                data_type,
                packed_range,
                names,
            });
        }

        Ok(DataType::Struct { packed, fields })
    }

    fn parse_package_qualified_data_type(&mut self) -> Result<DataType, FrontendError> {
        let package = self.expect_identifier()?;
        self.expect_symbol(':')?;
        self.expect_symbol(':')?;
        let name = self.expect_identifier()?;

        let package_aliases = self.package_type_aliases.get(&package).ok_or_else(|| {
            FrontendError::new(
                format!("unknown package '{}'", package),
                self.current().start,
            )
        })?;
        package_aliases.get(&name).cloned().ok_or_else(|| {
            FrontendError::new(
                format!("unknown type '{}::{}'", package, name),
                self.current().start,
            )
        })
    }

    fn parse_optional_packed_range(&mut self) -> Result<Option<PackedRange>, FrontendError> {
        if !self.consume_symbol('[') {
            return Ok(None);
        }
        let msb = self.parse_expression_until(&[':'])?;
        self.expect_symbol(':')?;
        let lsb = self.parse_expression_until(&[']'])?;
        self.expect_symbol(']')?;
        Ok(Some(PackedRange { msb, lsb }))
    }

    fn parse_unpacked_dimensions(&mut self) -> Result<Vec<PackedRange>, FrontendError> {
        let mut dimensions = Vec::new();
        while let Some(range) = self.parse_optional_packed_range()? {
            dimensions.push(range);
        }
        Ok(dimensions)
    }

    fn parse_optional_port_actual_until(
        &mut self,
        terminators: &[char],
    ) -> Result<Option<PortActual>, FrontendError> {
        if self.current_is_terminator(terminators) {
            return Ok(None);
        }
        let text = self.parse_raw_text_until(terminators)?;
        if text.is_empty() {
            Ok(None)
        } else {
            parse_port_actual(&text).map(Some)
        }
    }

    fn parse_value_expr_until(&mut self, terminators: &[char]) -> Result<ValueExpr, FrontendError> {
        let text = self.parse_raw_text_until(terminators)?;
        if text.is_empty() {
            Err(FrontendError::new(
                "expected value expression",
                self.current().start,
            ))
        } else {
            parse_value_expr(&text)
        }
    }

    fn parse_raw_text_until(&mut self, terminators: &[char]) -> Result<String, FrontendError> {
        if self.is_end() {
            return Ok(String::new());
        }

        let start = self.current().start;
        let mut end = start;
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut brace_depth = 0usize;

        while !self.is_end() {
            let token = self.current();
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;

            if top_level
                && let TokenKind::Symbol(ch) = token.kind
                && terminators.contains(&ch)
            {
                break;
            }

            match token.kind {
                TokenKind::Symbol('(') => paren_depth += 1,
                TokenKind::Symbol(')') => paren_depth = paren_depth.saturating_sub(1),
                TokenKind::Symbol('[') => bracket_depth += 1,
                TokenKind::Symbol(']') => bracket_depth = bracket_depth.saturating_sub(1),
                TokenKind::Symbol('{') => brace_depth += 1,
                TokenKind::Symbol('}') => brace_depth = brace_depth.saturating_sub(1),
                _ => {}
            }

            end = token.end;
            self.advance();
        }

        Ok(self
            .input
            .get(start..end)
            .unwrap_or_default()
            .trim()
            .to_string())
    }

    fn parse_expression_until(&mut self, terminators: &[char]) -> Result<Expr, FrontendError> {
        self.parse_expression_until_with_keyword_terminators(terminators, &[])
    }

    fn parse_expression_until_with_keyword_terminators(
        &mut self,
        terminators: &[char],
        keyword_terminators: &[&str],
    ) -> Result<Expr, FrontendError> {
        if self.is_end() {
            return Err(FrontendError::new(
                "expected expression, found end of input",
                0,
            ));
        }

        let start = self.current().start;
        let mut end = start;
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut brace_depth = 0usize;
        let mut ternary_depth = 0usize;

        while !self.is_end() {
            let token = self.current();
            let top_level = paren_depth == 0 && bracket_depth == 0 && brace_depth == 0;

            if top_level {
                if let Some(keyword) = self.current_ident()
                    && keyword_terminators.contains(&keyword)
                {
                    break;
                }
                if let TokenKind::Symbol(':') = token.kind {
                    if self.current_is_double_colon() {
                    } else if ternary_depth > 0 {
                        ternary_depth -= 1;
                    } else if terminators.contains(&':') {
                        break;
                    }
                } else if let TokenKind::Symbol(ch) = token.kind
                    && terminators.contains(&ch)
                {
                    break;
                }
            }

            match token.kind {
                TokenKind::Symbol('(') => paren_depth += 1,
                TokenKind::Symbol(')') => paren_depth = paren_depth.saturating_sub(1),
                TokenKind::Symbol('[') => bracket_depth += 1,
                TokenKind::Symbol(']') => bracket_depth = bracket_depth.saturating_sub(1),
                TokenKind::Symbol('{') => brace_depth += 1,
                TokenKind::Symbol('}') => brace_depth = brace_depth.saturating_sub(1),
                TokenKind::Symbol('?') if top_level => ternary_depth += 1,
                _ => {}
            }

            end = token.end;
            self.advance();
        }

        let text = self
            .input
            .get(start..end)
            .unwrap_or_default()
            .trim()
            .to_string();
        if text.is_empty() {
            return Err(FrontendError::new("expected expression", start));
        }

        parse_expression(&text).map_err(|err| {
            FrontendError::new(
                format!("failed to parse expression '{}': {}", text, err.message),
                start + err.position,
            )
        })
    }

    fn parse_port_direction(&mut self) -> Result<PortDirection, FrontendError> {
        if self.consume_keyword("input") {
            Ok(PortDirection::Input)
        } else if self.consume_keyword("output") {
            Ok(PortDirection::Output)
        } else if self.consume_keyword("inout") {
            Ok(PortDirection::Inout)
        } else {
            Err(FrontendError::new(
                "expected port direction",
                self.current().start,
            ))
        }
    }

    fn parse_optional_label(&mut self) -> Result<Option<String>, FrontendError> {
        if self.consume_symbol(':') {
            Ok(Some(self.expect_identifier()?))
        } else {
            Ok(None)
        }
    }

    fn current(&self) -> &Token {
        &self.tokens[self.index]
    }

    fn current_ident(&self) -> Option<&str> {
        match &self.current().kind {
            TokenKind::Ident(value) => Some(value.as_str()),
            _ => None,
        }
    }

    fn peek_kind(&self, offset: usize) -> Option<&TokenKind> {
        self.tokens
            .get(self.index + offset)
            .map(|token| &token.kind)
    }

    fn is_end(&self) -> bool {
        matches!(self.current().kind, TokenKind::End)
    }

    fn advance(&mut self) {
        if !self.is_end() {
            self.index += 1;
        }
    }

    fn peek_keyword(&self, keyword: &str) -> bool {
        matches!(self.current_ident(), Some(value) if value == keyword)
    }

    fn consume_keyword(&mut self, keyword: &str) -> bool {
        if self.peek_keyword(keyword) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_keyword(&mut self, keyword: &str) -> Result<(), FrontendError> {
        if self.consume_keyword(keyword) {
            Ok(())
        } else {
            Err(FrontendError::new(
                format!("expected keyword '{}'", keyword),
                self.current().start,
            ))
        }
    }

    fn peek_symbol(&self, symbol: char) -> bool {
        matches!(self.current().kind, TokenKind::Symbol(ch) if ch == symbol)
    }

    fn consume_symbol(&mut self, symbol: char) -> bool {
        if self.peek_symbol(symbol) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_symbol(&mut self, symbol: char) -> Result<(), FrontendError> {
        if self.consume_symbol(symbol) {
            Ok(())
        } else {
            Err(FrontendError::new(
                format!("expected symbol '{}'", symbol),
                self.current().start,
            ))
        }
    }

    fn current_is_terminator(&self, terminators: &[char]) -> bool {
        matches!(self.current().kind, TokenKind::Symbol(ch) if terminators.contains(&ch))
    }

    fn current_is_double_colon(&self) -> bool {
        matches!(self.current().kind, TokenKind::Symbol(':'))
            && (matches!(self.peek_kind(1), Some(TokenKind::Symbol(':')))
                || self.index > 0
                    && matches!(
                        self.tokens.get(self.index - 1).map(|token| &token.kind),
                        Some(TokenKind::Symbol(':'))
                    ))
    }

    fn expect_identifier(&mut self) -> Result<String, FrontendError> {
        match &self.current().kind {
            TokenKind::Ident(value) if !is_keyword(value) => {
                let value = value.clone();
                self.advance();
                Ok(value)
            }
            _ => Err(FrontendError::new(
                "expected identifier",
                self.current().start,
            )),
        }
    }

    fn peek_direction_keyword(&self) -> bool {
        self.peek_keyword("input") || self.peek_keyword("output") || self.peek_keyword("inout")
    }

    fn peek_typedef_name(&self) -> bool {
        matches!(self.current_ident(), Some(value) if self.type_aliases.contains_key(value))
    }

    fn peek_package_qualified_type(&self) -> bool {
        let Some(package) = self.current_ident() else {
            return false;
        };
        let Some(aliases) = self.package_type_aliases.get(package) else {
            return false;
        };
        matches!(self.peek_kind(1), Some(TokenKind::Symbol(':')))
            && matches!(self.peek_kind(2), Some(TokenKind::Symbol(':')))
            && matches!(self.peek_kind(3), Some(TokenKind::Ident(name)) if aliases.contains_key(name))
    }

    fn looks_like_module_instantiation(&self) -> bool {
        let Some(name) = self.current_ident() else {
            return false;
        };
        if is_keyword(name) {
            return false;
        }
        if self.type_aliases.contains_key(name) {
            return false;
        }
        if (self.package_type_aliases.contains_key(name)
            || self.package_constant_names.contains_key(name))
            && matches!(self.peek_kind(1), Some(TokenKind::Symbol(':')))
        {
            return false;
        }

        matches!(
            self.peek_kind(1),
            Some(TokenKind::Ident(_)) | Some(TokenKind::Symbol('#'))
        )
    }

    fn describe_current(&self) -> String {
        match &self.current().kind {
            TokenKind::Ident(value) => format!("identifier '{}'", value),
            TokenKind::Number(value) => format!("number '{}'", value),
            TokenKind::Symbol(value) => format!("symbol '{}'", value),
            TokenKind::Operator(value) => format!("operator '{}'", value),
            TokenKind::End => "end of input".to_string(),
        }
    }
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    is_ident_start(ch) || ch.is_ascii_digit() || ch == '$'
}

fn is_keyword(value: &str) -> bool {
    matches!(
        value,
        "always"
            | "always_comb"
            | "always_ff"
            | "always_latch"
            | "assign"
            | "begin"
            | "else"
            | "end"
            | "endgenerate"
            | "endmodule"
            | "for"
            | "generate"
            | "genvar"
            | "import"
            | "if"
            | "inout"
            | "input"
            | "localparam"
            | "logic"
            | "module"
            | "negedge"
            | "or"
            | "output"
            | "packed"
            | "package"
            | "parameter"
            | "posedge"
            | "reg"
            | "struct"
            | "typedef"
            | "union"
            | "wire"
            | "endpackage"
    )
}

fn is_data_type_keyword(value: &str) -> bool {
    matches!(
        value,
        "bit" | "byte" | "shortint" | "int" | "integer" | "longint" | "logic" | "reg" | "wire"
    )
}

#[cfg(test)]
mod tests {
    use super::{
        AssignmentTarget, DataType, EventEdge, GenerateFor, GenerateIf, ModuleItem, NetKind,
        ParameterOverride, PortActual, PortConnection, ProceduralKind, Statement, ValueExpr,
        parse_design, parse_module,
    };
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;

    #[derive(Debug, Deserialize)]
    struct GeneratedContractManifest {
        contract_version: String,
        grammar_name: String,
        samples: Vec<GeneratedContractSample>,
    }

    #[derive(Debug, Deserialize)]
    struct GeneratedContractSample {
        label: String,
        expected_parse_ok: bool,
        #[serde(default)]
        expected_handwritten_parse_ok: Option<bool>,
        #[serde(default)]
        expected_elaboration: Option<GeneratedContractElaboration>,
        sample: String,
    }

    #[derive(Debug, Deserialize)]
    struct GeneratedContractElaboration {
        #[serde(default)]
        top: Option<String>,
        ok: bool,
        #[serde(default)]
        child_instance_count: Option<usize>,
        #[serde(default)]
        error_contains: Option<String>,
    }

    const MIN_GENERATED_CONTRACT_ELABORATION_SAMPLES: usize = 37;
    const MIN_GENERATED_CONTRACT_ELABORATION_ACCEPTS: usize = 27;
    const MIN_GENERATED_CONTRACT_ELABORATION_REJECTS: usize = 10;

    fn load_generated_contract_manifest() -> GeneratedContractManifest {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../rust/test_data/grammar_quality/rtl_frontend_generated_parity_contract_v0.json",
        );
        let raw = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        serde_json::from_str(&raw)
            .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()))
    }

    #[test]
    fn generated_contract_manifest_matches_handwritten_parse_surface() {
        let manifest = load_generated_contract_manifest();
        assert_eq!(manifest.contract_version, "0.1.0");
        assert_eq!(manifest.grammar_name, "rtl_frontend");
        assert!(
            !manifest.samples.is_empty(),
            "generated contract manifest must contain samples"
        );

        let mut expected_accepts = 0usize;
        let mut expected_rejects = 0usize;
        let mut explicit_handwritten_divergences = 0usize;
        let mut mismatches = Vec::new();
        for sample in &manifest.samples {
            let handwritten_ok = parse_design(&sample.sample).is_ok();
            let expected_handwritten_ok = sample
                .expected_handwritten_parse_ok
                .unwrap_or(sample.expected_parse_ok);
            if sample.expected_parse_ok {
                expected_accepts += 1;
            } else {
                expected_rejects += 1;
            }
            if expected_handwritten_ok != sample.expected_parse_ok {
                explicit_handwritten_divergences += 1;
            }
            if handwritten_ok != expected_handwritten_ok {
                mismatches.push(format!(
                    "{}: expected {}, handwritten parse returned {}",
                    sample.label, expected_handwritten_ok, handwritten_ok
                ));
            }
        }

        assert!(
            !manifest.samples.is_empty() && expected_accepts > 0 && expected_rejects > 0,
            "generated contract manifest should retain both positive and negative replay samples"
        );
        assert!(
            mismatches.is_empty(),
            "handwritten rtl_frontend replay drifted from generated contract manifest over {} samples ({} expected accept, {} expected reject):\n{}",
            manifest.samples.len(),
            expected_accepts,
            expected_rejects,
            mismatches.join("\n")
        );
        assert_eq!(
            explicit_handwritten_divergences, 0,
            "current rtl_frontend generated contract samples should parse the same way through the generated and handwritten frontends"
        );
    }

    #[test]
    fn generated_contract_manifest_matches_handwritten_elaboration_surface() {
        let manifest = load_generated_contract_manifest();
        assert_eq!(manifest.contract_version, "0.1.0");
        assert_eq!(manifest.grammar_name, "rtl_frontend");

        let mut checked = 0usize;
        let mut expected_accepts = 0usize;
        let mut expected_rejects = 0usize;
        for sample in &manifest.samples {
            let Some(expectation) = &sample.expected_elaboration else {
                continue;
            };
            checked += 1;
            assert!(
                sample.expected_parse_ok,
                "sample '{}' cannot carry elaboration expectations unless generated parse is expected to pass",
                sample.label
            );

            let design = parse_design(&sample.sample).unwrap_or_else(|err| {
                panic!(
                    "sample '{}' should parse before elaboration replay: {err}",
                    sample.label
                )
            });
            let top = expectation.top.as_deref().unwrap_or("top");
            let result = design.elaborate_top(top, &HashMap::new());
            if expectation.ok {
                expected_accepts += 1;
                let elaborated = result.unwrap_or_else(|err| {
                    panic!(
                        "sample '{}' should elaborate top '{}': {err}",
                        sample.label, top
                    )
                });
                if let Some(expected_count) = expectation.child_instance_count {
                    assert_eq!(
                        elaborated.child_instances.len(),
                        expected_count,
                        "sample '{}' elaborated unexpected immediate child instance count",
                        sample.label
                    );
                }
            } else {
                expected_rejects += 1;
                let error = match result {
                    Ok(_) => panic!(
                        "sample '{}' should reject elaboration for top '{}'",
                        sample.label, top
                    ),
                    Err(error) => error,
                };
                if let Some(expected_message) = &expectation.error_contains {
                    assert!(
                        error.message.contains(expected_message),
                        "sample '{}' elaboration error should contain {:?}, got {:?}",
                        sample.label,
                        expected_message,
                        error.message
                    );
                }
            }
        }

        assert!(
            checked >= MIN_GENERATED_CONTRACT_ELABORATION_SAMPLES,
            "generated contract manifest should retain at least {MIN_GENERATED_CONTRACT_ELABORATION_SAMPLES} elaboration replay samples, got {checked}"
        );
        assert!(
            expected_accepts >= MIN_GENERATED_CONTRACT_ELABORATION_ACCEPTS,
            "generated contract manifest should retain at least {MIN_GENERATED_CONTRACT_ELABORATION_ACCEPTS} positive elaboration replay samples, got {expected_accepts}"
        );
        assert!(
            expected_rejects >= MIN_GENERATED_CONTRACT_ELABORATION_REJECTS,
            "generated contract manifest should retain at least {MIN_GENERATED_CONTRACT_ELABORATION_REJECTS} negative elaboration replay samples, got {expected_rejects}"
        );
    }

    #[test]
    fn parses_module_shape_and_evaluates_constants() {
        let module = parse_module(
            r#"
            module arithmetic #(
                parameter WIDTH = 8,
                parameter DEPTH = WIDTH + 4
            ) (
                input logic [WIDTH-1:0] a,
                input logic [WIDTH-1:0] b,
                output logic [DEPTH-1:0] y
            );
            parameter EXTRA = DEPTH + 1;
            localparam TOTAL = WIDTH * 2;
            logic [WIDTH-1:0] data, scratch;
            assign y = DEPTH > WIDTH ? DEPTH : WIDTH;
            always_comb begin : comb_blk
                data = WIDTH + TOTAL;
                if (EXTRA > 0)
                    scratch = TOTAL;
                else
                    scratch = 0;
            end
            generate
                if (WIDTH > 4) begin : has_extra
                    logic [TOTAL-1:0] extra;
                end else begin : no_extra
                    logic dummy;
                end
                for (genvar i = 0; i < 3; i = i + 1) begin : lanes
                    logic tap;
                end
            endgenerate
            endmodule
            "#,
        )
        .expect("module should parse");

        assert_eq!(module.name, "arithmetic");
        assert_eq!(module.parameters.len(), 2);
        assert_eq!(module.ports.len(), 3);

        let env = module
            .evaluate_constant_env(&HashMap::new())
            .expect("constant env should resolve");
        assert_eq!(env.get("WIDTH"), Some(&8));
        assert_eq!(env.get("DEPTH"), Some(&12));
        assert_eq!(env.get("EXTRA"), Some(&13));
        assert_eq!(env.get("TOTAL"), Some(&16));
        assert_eq!(module.ports[2].width(&env).unwrap(), Some(12));

        let generate_region = module
            .items
            .iter()
            .find_map(|item| match item {
                ModuleItem::GenerateRegion(region) => Some(region),
                _ => None,
            })
            .expect("generate region should exist");

        match &generate_region.items[0] {
            ModuleItem::GenerateIf(generate_if) => {
                assert!(generate_if.is_enabled(&env).unwrap());
                assert_eq!(generate_if.then_label.as_deref(), Some("has_extra"));
                assert_eq!(generate_if.else_label.as_deref(), Some("no_extra"));
            }
            other => panic!("expected generate-if, got {other:?}"),
        }

        match &generate_region.items[1] {
            ModuleItem::GenerateFor(generate_for) => {
                assert_eq!(generate_for.label.as_deref(), Some("lanes"));
                assert_eq!(
                    generate_for.iteration_values(&env, 8).unwrap(),
                    vec![0, 1, 2]
                );
            }
            other => panic!("expected generate-for, got {other:?}"),
        }
    }

    #[test]
    fn parses_always_star_blocks() {
        let module = parse_module(
            r#"
            module star (
                input logic a,
                output logic y
            );
            always @(*) begin
                if (a)
                    y = 1;
                else
                    y = 0;
            end
            endmodule
            "#,
        )
        .expect("module should parse");

        let block = module
            .items
            .iter()
            .find_map(|item| match item {
                ModuleItem::ProceduralBlock(block) => Some(block),
                _ => None,
            })
            .expect("procedural block should exist");

        assert_eq!(block.kind, ProceduralKind::AlwaysStar);
        assert_eq!(block.event_control, None);
        match &block.statement {
            Statement::Block { statements, .. } => assert_eq!(statements.len(), 1),
            other => panic!("expected block statement, got {other:?}"),
        }
    }

    #[test]
    fn parses_always_ff_blocks_with_edge_event_controls() {
        let module = parse_module(
            r#"
            module seq (
                input logic clk,
                input logic rst_n,
                input logic d,
                output logic q
            );
            always_ff @(posedge clk or negedge rst_n) begin
                if (!rst_n)
                    q <= 0;
                else
                    q <= d;
            end
            endmodule
            "#,
        )
        .expect("module should parse");

        let block = module
            .items
            .iter()
            .find_map(|item| match item {
                ModuleItem::ProceduralBlock(block) => Some(block),
                _ => None,
            })
            .expect("procedural block should exist");

        assert_eq!(block.kind, ProceduralKind::AlwaysFf);
        let event_control = block
            .event_control
            .as_ref()
            .expect("always_ff should retain event control");
        assert_eq!(event_control.len(), 2);
        assert_eq!(event_control[0].edge, Some(EventEdge::Posedge));
        assert_eq!(
            event_control[0].expr,
            rtl_const_expr::parse_expression("clk").unwrap()
        );
        assert_eq!(event_control[1].edge, Some(EventEdge::Negedge));
        assert_eq!(
            event_control[1].expr,
            rtl_const_expr::parse_expression("rst_n").unwrap()
        );
        match &block.statement {
            Statement::Block { statements, .. } => assert_eq!(statements.len(), 1),
            other => panic!("expected block statement, got {other:?}"),
        }
    }

    #[test]
    fn parses_always_latch_blocks() {
        let module = parse_module(
            r#"
            module latchy (
                input logic en,
                input logic d,
                output logic q
            );
            always_latch begin
                if (en)
                    q <= d;
            end
            endmodule
            "#,
        )
        .expect("module should parse");

        let block = module
            .items
            .iter()
            .find_map(|item| match item {
                ModuleItem::ProceduralBlock(block) => Some(block),
                _ => None,
            })
            .expect("procedural block should exist");

        assert_eq!(block.kind, ProceduralKind::AlwaysLatch);
        assert_eq!(block.event_control, None);
        match &block.statement {
            Statement::Block { statements, .. } => assert_eq!(statements.len(), 1),
            other => panic!("expected block statement, got {other:?}"),
        }
    }

    #[test]
    fn parses_typed_continuous_assign_targets() {
        let module = parse_module(
            r#"
            module top #(
                parameter BIT = 1
            ) (
                input logic d
            );
            struct packed {
                logic [7:0] data;
                logic valid;
            } cfg;
            assign cfg.data[BIT] = d;
            endmodule
            "#,
        )
        .expect("module should parse");

        let assign = module
            .items
            .iter()
            .find_map(|item| match item {
                ModuleItem::ContinuousAssign(assign) => Some(assign),
                _ => None,
            })
            .expect("continuous assign should exist");

        assert_eq!(
            assign.target,
            AssignmentTarget::BitSelect {
                signal: "cfg.data".to_string(),
                index: rtl_const_expr::parse_expression("BIT").unwrap(),
            }
        );
    }

    #[test]
    fn parses_concatenated_continuous_assign_targets() {
        let module = parse_module(
            r#"
            module top #(
                parameter BIT = 1
            ) (
                input logic d
            );
            struct packed {
                logic [7:0] data;
                logic valid;
            } cfg;
            assign {cfg.data[BIT], cfg.valid} = d;
            endmodule
            "#,
        )
        .expect("module should parse");

        let assign = module
            .items
            .iter()
            .find_map(|item| match item {
                ModuleItem::ContinuousAssign(assign) => Some(assign),
                _ => None,
            })
            .expect("continuous assign should exist");

        assert_eq!(
            assign.target,
            AssignmentTarget::Concat(vec![
                AssignmentTarget::BitSelect {
                    signal: "cfg.data".to_string(),
                    index: rtl_const_expr::parse_expression("BIT").unwrap(),
                },
                AssignmentTarget::Signal("cfg.valid".to_string()),
            ])
        );
    }

    #[test]
    fn parses_structured_continuous_assign_values() {
        let module = parse_module(
            r#"
            module top #(
                parameter BIT = 1
            ) (
                input logic d
            );
            struct packed {
                logic [7:0] data;
                logic valid;
            } cfg;
            assign cfg.valid = {cfg.data[BIT], cfg.data[0]};
            endmodule
            "#,
        )
        .expect("module should parse");

        let assign = module
            .items
            .iter()
            .find_map(|item| match item {
                ModuleItem::ContinuousAssign(assign) => Some(assign),
                _ => None,
            })
            .expect("continuous assign should exist");

        assert_eq!(
            assign.value,
            ValueExpr::Concat(vec![
                ValueExpr::BitSelect {
                    signal: "cfg.data".to_string(),
                    index: rtl_const_expr::parse_expression("BIT").unwrap(),
                },
                ValueExpr::BitSelect {
                    signal: "cfg.data".to_string(),
                    index: rtl_const_expr::parse_expression("0").unwrap(),
                },
            ])
        );
    }

    #[test]
    fn elaboration_accepts_well_formed_always_ff_blocks() {
        let design = parse_design(
            r#"
            module top (
                input logic clk,
                input logic rst_n,
                input logic d,
                output logic q
            );
            always_ff @(posedge clk or negedge rst_n) begin
                if (!rst_n)
                    q <= 0;
                else
                    q <= d;
            end
            endmodule
            "#,
        )
        .expect("design should parse");

        design
            .elaborate_top("top", &HashMap::new())
            .expect("elaboration should accept well-formed always_ff blocks");
    }

    #[test]
    fn elaboration_accepts_typed_procedural_assignment_targets() {
        let design = parse_design(
            r#"
            module top #(
                parameter IDX = 1,
                parameter BIT = 2
            ) (
                input logic clk,
                input logic d
            );
            struct packed {
                logic [7:0] data;
                logic valid;
            } cfgs [0:1];
            always_ff @(posedge clk) begin
                cfgs[IDX].data[BIT] <= d;
            end
            endmodule
            "#,
        )
        .expect("design should parse");

        design
            .elaborate_top("top", &HashMap::new())
            .expect("elaboration should accept typed procedural assignment targets");
    }

    #[test]
    fn elaboration_accepts_concatenated_procedural_assignment_targets() {
        let design = parse_design(
            r#"
            module top #(
                parameter IDX = 1,
                parameter BIT = 2
            ) (
                input logic d
            );
            struct packed {
                logic [7:0] data;
                logic valid;
            } cfgs [0:1];
            always_comb begin
                {cfgs[IDX].data[BIT], cfgs[IDX].valid} = d;
            end
            endmodule
            "#,
        )
        .expect("design should parse");

        design
            .elaborate_top("top", &HashMap::new())
            .expect("elaboration should accept concatenated procedural assignment targets");
    }

    #[test]
    fn elaboration_accepts_structured_assignment_values() {
        let design = parse_design(
            r#"
            module top #(
                parameter IDX = 1,
                parameter BIT = 2
            ) (
                input logic clk,
                input logic d
            );
            struct packed {
                logic [7:0] data;
                logic valid;
            } cfgs [0:1];
            always_ff @(posedge clk) begin
                cfgs[IDX].valid <= {cfgs[IDX].data[BIT], d};
            end
            endmodule
            "#,
        )
        .expect("design should parse");

        design
            .elaborate_top("top", &HashMap::new())
            .expect("elaboration should accept structured assignment values");
    }

    #[test]
    fn parse_design_rejects_blocking_assignments_in_always_ff_blocks() {
        let error = parse_design(
            r#"
            module top (
                input logic clk,
                output logic q
            );
            always_ff @(posedge clk) begin
                q = 1;
            end
            endmodule
            "#,
        )
        .expect_err("parser should reject blocking assignments in always_ff");

        assert!(
            error
                .message
                .contains("always_ff block does not allow blocking assignment to 'q'")
        );
    }

    #[test]
    fn elaboration_rejects_unknown_identifiers_in_always_ff_event_controls() {
        let design = parse_design(
            r#"
            module top (
                output logic q
            );
            always_ff @(posedge clk_missing) begin
                q <= 1;
            end
            endmodule
            "#,
        )
        .expect("design should parse");

        let error = design
            .elaborate_top("top", &HashMap::new())
            .expect_err("elaboration should reject unknown always_ff event-control identifiers");
        assert!(
            error
                .message
                .contains("unknown parent-scope identifier 'clk_missing'")
        );
    }

    #[test]
    fn elaboration_rejects_unknown_members_in_continuous_assign_targets() {
        let design = parse_design(
            r#"
            module top (
                input logic d
            );
            struct packed {
                logic [7:0] data;
                logic valid;
            } cfg;
            assign cfg.missing = d;
            endmodule
            "#,
        )
        .expect("design should parse");

        let error = design
            .elaborate_top("top", &HashMap::new())
            .expect_err("elaboration should reject unknown members in assign targets");
        assert!(
            error
                .message
                .contains("unknown member 'missing' on parent-scope identifier 'cfg'")
        );
    }

    #[test]
    fn elaboration_rejects_unknown_members_in_assignment_values() {
        let design = parse_design(
            r#"
            module top (
                input logic d
            );
            struct packed {
                logic [7:0] data;
                logic valid;
            } cfg;
            assign cfg.valid = {cfg.data, cfg.missing};
            endmodule
            "#,
        )
        .expect("design should parse");

        let error = design
            .elaborate_top("top", &HashMap::new())
            .expect_err("elaboration should reject unknown members in assignment values");
        assert!(
            error
                .message
                .contains("unknown member 'missing' on parent-scope identifier 'cfg'")
        );
    }

    #[test]
    fn elaboration_rejects_unknown_members_in_concat_assign_targets() {
        let design = parse_design(
            r#"
            module top (
                input logic d
            );
            struct packed {
                logic [7:0] data;
                logic valid;
            } cfg;
            assign {cfg.data, cfg.missing} = d;
            endmodule
            "#,
        )
        .expect("design should parse");

        let error = design
            .elaborate_top("top", &HashMap::new())
            .expect_err("elaboration should reject unknown members in concatenated assign targets");
        assert!(
            error
                .message
                .contains("unknown member 'missing' on parent-scope identifier 'cfg'")
        );
    }

    #[test]
    fn elaboration_rejects_unknown_identifiers_in_always_latch_blocks() {
        let design = parse_design(
            r#"
            module top (
                input logic en,
                output logic q
            );
            always_latch begin
                if (en)
                    q <= missing;
            end
            endmodule
            "#,
        )
        .expect("design should parse");

        let error = design
            .elaborate_top("top", &HashMap::new())
            .expect_err("elaboration should reject unknown identifiers in latch bodies");
        assert!(
            error
                .message
                .contains("unknown parent-scope identifier 'missing'")
        );
    }

    #[test]
    fn parse_design_supports_multiple_modules() {
        let design = parse_design(
            r#"
            module first;
            endmodule

            module second;
            endmodule
            "#,
        )
        .expect("design should parse");

        assert_eq!(design.modules.len(), 2);
        assert_eq!(design.modules[0].name, "first");
        assert_eq!(design.modules[1].name, "second");
    }

    #[test]
    fn parse_design_supports_file_scope_typedefs_for_later_modules() {
        let design = parse_design(
            r#"
            typedef struct packed {
                logic [7:0] data;
                logic valid;
            } cfg_t;

            module child (
                input cfg_t cfg,
                output logic y
            );
            endmodule

            module top (
                output logic y
            );
            cfg_t cfg;
            endmodule
            "#,
        )
        .expect("design should parse");

        assert_eq!(design.typedefs.len(), 1);
        assert_eq!(design.typedefs[0].name, "cfg_t");
        match &design.typedefs[0].data_type {
            DataType::Struct { packed, fields } => {
                assert!(*packed);
                assert_eq!(fields.len(), 2);
            }
            other => panic!("expected file-scope typedef struct, got {other:?}"),
        }

        match design.modules[0].ports[0]
            .data_type
            .as_ref()
            .expect("child port type should exist")
        {
            DataType::Struct { fields, .. } => {
                assert_eq!(fields[0].names, vec!["data".to_string()]);
                assert_eq!(fields[1].names, vec!["valid".to_string()]);
            }
            other => panic!("expected resolved child port struct type, got {other:?}"),
        }

        match &design.modules[1].items[0] {
            ModuleItem::NetDecl(net) => {
                match net.data_type.as_ref().expect("net type should exist") {
                    DataType::Struct { fields, .. } => {
                        assert_eq!(fields[0].names, vec!["data".to_string()]);
                        assert_eq!(fields[1].names, vec!["valid".to_string()]);
                    }
                    other => panic!("expected resolved top net struct type, got {other:?}"),
                }
            }
            other => panic!("expected net declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_design_supports_package_typedefs_for_later_modules() {
        let design = parse_design(
            r#"
            package cfg_pkg;
                typedef struct packed {
                    logic [7:0] data;
                    logic valid;
                } cfg_t;
            endpackage

            module child (
                input cfg_pkg::cfg_t cfg,
                output logic y
            );
            endmodule

            module top (
                output logic y
            );
            import cfg_pkg::*;
            cfg_t cfg;
            endmodule
            "#,
        )
        .expect("design should parse");

        assert_eq!(design.packages.len(), 1);
        assert_eq!(design.packages[0].name, "cfg_pkg");
        assert_eq!(design.packages[0].typedefs.len(), 1);

        match design.modules[0].ports[0]
            .data_type
            .as_ref()
            .expect("child port type should exist")
        {
            DataType::Struct { fields, .. } => {
                assert_eq!(fields[0].names, vec!["data".to_string()]);
                assert_eq!(fields[1].names, vec!["valid".to_string()]);
            }
            other => panic!("expected package-qualified struct port type, got {other:?}"),
        }

        match &design.modules[1].items[0] {
            ModuleItem::ImportDecl(import_decl) => {
                assert_eq!(import_decl.package, "cfg_pkg");
                assert!(import_decl.wildcard);
            }
            other => panic!("expected import declaration, got {other:?}"),
        }

        match &design.modules[1].items[1] {
            ModuleItem::NetDecl(net) => {
                match net.data_type.as_ref().expect("net type should exist") {
                    DataType::Struct { fields, .. } => {
                        assert_eq!(fields[0].names, vec!["data".to_string()]);
                        assert_eq!(fields[1].names, vec!["valid".to_string()]);
                    }
                    other => panic!("expected imported struct net type, got {other:?}"),
                }
            }
            other => panic!("expected net declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_design_supports_named_package_imports_for_later_declarations() {
        let design = parse_design(
            r#"
            package cfg_pkg;
                typedef struct packed {
                    logic [7:0] data;
                    logic valid;
                } cfg_t;
            endpackage

            module top (
                output logic y
            );
            import cfg_pkg::cfg_t;
            cfg_t cfg;
            endmodule
            "#,
        )
        .expect("design should parse");

        match &design.modules[0].items[0] {
            ModuleItem::ImportDecl(import_decl) => {
                assert_eq!(import_decl.package, "cfg_pkg");
                assert_eq!(import_decl.imported_name.as_deref(), Some("cfg_t"));
                assert!(!import_decl.wildcard);
            }
            other => panic!("expected import declaration, got {other:?}"),
        }

        match &design.modules[0].items[1] {
            ModuleItem::NetDecl(net) => {
                match net.data_type.as_ref().expect("net type should exist") {
                    DataType::Struct { fields, .. } => {
                        assert_eq!(fields[0].names, vec!["data".to_string()]);
                        assert_eq!(fields[1].names, vec!["valid".to_string()]);
                    }
                    other => panic!("expected named-import struct net type, got {other:?}"),
                }
            }
            other => panic!("expected net declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_design_supports_header_named_package_imports_for_port_lists() {
        let design = parse_design(
            r#"
            package cfg_pkg;
                typedef struct packed {
                    logic [7:0] data;
                    logic valid;
                } cfg_t;
            endpackage

            module top import cfg_pkg::cfg_t; (
                input cfg_t cfg,
                output logic y
            );
            endmodule
            "#,
        )
        .expect("design should parse");

        assert_eq!(design.modules.len(), 1);
        assert_eq!(design.modules[0].header_imports.len(), 1);
        assert_eq!(design.modules[0].header_imports[0].package, "cfg_pkg");
        assert_eq!(
            design.modules[0].header_imports[0].imported_name.as_deref(),
            Some("cfg_t")
        );
        assert!(!design.modules[0].header_imports[0].wildcard);

        match design.modules[0].ports[0]
            .data_type
            .as_ref()
            .expect("port type should exist")
        {
            DataType::Struct { fields, .. } => {
                assert_eq!(fields[0].names, vec!["data".to_string()]);
                assert_eq!(fields[1].names, vec!["valid".to_string()]);
            }
            other => panic!("expected header-imported struct port type, got {other:?}"),
        }
    }

    #[test]
    fn parses_unpacked_array_ports_and_nets() {
        let module = parse_module(
            r#"
            module top #(
                parameter DEPTH = 4
            ) (
                input logic [7:0] a [0:DEPTH-1],
                input logic [7:0] b [1:DEPTH],
                output logic y
            );
            logic [7:0] bank0 [0:DEPTH-1], bank1 [1:DEPTH];
            endmodule
            "#,
        )
        .expect("module should parse");

        let env = module
            .evaluate_constant_env(&HashMap::new())
            .expect("constant env should resolve");

        assert_eq!(module.ports[0].width(&env).unwrap(), Some(8));
        assert_eq!(module.ports[0].unpacked_shape(&env).unwrap(), vec![4]);
        assert_eq!(module.ports[1].unpacked_shape(&env).unwrap(), vec![4]);
        assert!(module.ports[2].unpacked_shape(&env).unwrap().is_empty());

        match &module.items[0] {
            ModuleItem::NetDecl(net) => {
                assert_eq!(net.name, "bank0");
                assert_eq!(net.width(&env).unwrap(), Some(8));
                assert_eq!(net.unpacked_shape(&env).unwrap(), vec![4]);
            }
            other => panic!("expected first unpacked net declaration, got {other:?}"),
        }

        match &module.items[1] {
            ModuleItem::NetDecl(net) => {
                assert_eq!(net.name, "bank1");
                assert_eq!(net.unpacked_shape(&env).unwrap(), vec![4]);
            }
            other => panic!("expected second unpacked net declaration, got {other:?}"),
        }
    }

    #[test]
    fn elaboration_accepts_unpacked_array_element_actuals() {
        let design = parse_design(
            r#"
            module child (
                input logic [7:0] a,
                output logic y
            );
            endmodule

            module top #(
                parameter DEPTH = 2,
                parameter IDX = 1
            ) (
                output logic y
            );
            logic [7:0] banks [0:DEPTH-1];
            child u_child (.a(banks[IDX]), .y(y));
            endmodule
            "#,
        )
        .expect("design should parse");

        let elaborated = design
            .elaborate_top("top", &HashMap::new())
            .expect("elaboration should succeed");

        assert_eq!(
            elaborated.child_instances[0].port_bindings[0].actual,
            Some(PortActual::BitSelect {
                signal: "banks".to_string(),
                index: rtl_const_expr::parse_expression("IDX").unwrap(),
            })
        );
    }

    #[test]
    fn elaborate_top_supports_package_qualified_constants() {
        let design = parse_design(
            r#"
            package cfg_pkg;
                localparam WIDTH = 8;
                parameter DEPTH = WIDTH + 2;
            endpackage

            module child #(parameter W = 1) (
                input logic [W-1:0] a,
                output logic y
            );
            endmodule

            module top #(
                parameter LANES = cfg_pkg::WIDTH / 4
            ) (
                input logic [cfg_pkg::DEPTH-1:0] data,
                output logic y
            );
            localparam TOTAL = cfg_pkg::DEPTH + LANES;
            child #(.W(TOTAL)) u_child (.a(data[cfg_pkg::DEPTH-1:0]), .y(y));
            endmodule
            "#,
        )
        .expect("design should parse");

        assert_eq!(design.packages.len(), 1);
        assert_eq!(design.packages[0].constants.len(), 2);

        let elaborated = design
            .elaborate_top("top", &HashMap::new())
            .expect("elaboration should succeed");

        assert_eq!(elaborated.parameters.get("LANES"), Some(&2));
        assert_eq!(elaborated.parameters.get("TOTAL"), Some(&12));
        assert_eq!(elaborated.child_instances[0].parameters.get("W"), Some(&12));
        assert_eq!(
            elaborated.child_instances[0].port_bindings[0].actual,
            Some(PortActual::PartSelect {
                signal: "data".to_string(),
                msb: rtl_const_expr::parse_expression("cfg_pkg::DEPTH - 1").unwrap(),
                lsb: rtl_const_expr::parse_expression("0").unwrap(),
            })
        );
    }

    #[test]
    fn elaborate_top_supports_header_imported_package_constants() {
        let design = parse_design(
            r#"
            package cfg_pkg;
                localparam WIDTH = 8;
                localparam DEPTH = WIDTH + 1;
            endpackage

            module child #(parameter W = 1) (
                input logic [W-1:0] a,
                output logic y
            );
            endmodule

            module top import cfg_pkg::*; #(
                parameter LANES = WIDTH / 4
            ) (
                input logic [DEPTH-1:0] data,
                output logic y
            );
            localparam TOTAL = DEPTH + LANES;
            child #(.W(TOTAL)) u_child (.a(data[WIDTH-1:0]), .y(y));
            endmodule
            "#,
        )
        .expect("design should parse");

        let elaborated = design
            .elaborate_top("top", &HashMap::new())
            .expect("elaboration should succeed");

        assert_eq!(elaborated.parameters.get("LANES"), Some(&2));
        assert_eq!(elaborated.parameters.get("TOTAL"), Some(&11));
        assert_eq!(elaborated.child_instances[0].parameters.get("W"), Some(&11));
        assert_eq!(
            elaborated.child_instances[0].port_bindings[0].actual,
            Some(PortActual::PartSelect {
                signal: "data".to_string(),
                msb: rtl_const_expr::parse_expression("WIDTH - 1").unwrap(),
                lsb: rtl_const_expr::parse_expression("0").unwrap(),
            })
        );
    }

    #[test]
    fn elaborate_top_supports_named_package_constant_imports() {
        let design = parse_design(
            r#"
            package cfg_pkg;
                localparam WIDTH = 8;
            endpackage

            module child #(parameter W = 1) (
                input logic [W-1:0] a,
                output logic y
            );
            endmodule

            module top (
                input logic [8:0] data,
                output logic y
            );
            import cfg_pkg::WIDTH;
            localparam TOTAL = WIDTH + 1;
            child #(.W(TOTAL)) u_child (.a(data[WIDTH:0]), .y(y));
            endmodule
            "#,
        )
        .expect("design should parse");

        let elaborated = design
            .elaborate_top("top", &HashMap::new())
            .expect("elaboration should succeed");

        assert_eq!(elaborated.parameters.get("TOTAL"), Some(&9));
        assert_eq!(elaborated.child_instances[0].parameters.get("W"), Some(&9));
        assert_eq!(
            elaborated.child_instances[0].port_bindings[0].actual,
            Some(PortActual::PartSelect {
                signal: "data".to_string(),
                msb: rtl_const_expr::parse_expression("WIDTH").unwrap(),
                lsb: rtl_const_expr::parse_expression("0").unwrap(),
            })
        );
    }

    #[test]
    fn generate_if_condition_evaluates_from_symbols() {
        let generate_if = GenerateIf {
            condition: rtl_const_expr::parse_expression("WIDTH > 1").unwrap(),
            then_label: None,
            then_items: Vec::new(),
            else_label: None,
            else_items: Vec::new(),
        };

        let symbols = HashMap::from([("WIDTH".to_string(), 4)]);
        assert!(generate_if.is_enabled(&symbols).unwrap());
    }

    #[test]
    fn generate_for_unrolls_with_bounded_iteration() {
        let generate_for = GenerateFor {
            index_name: "i".to_string(),
            declares_genvar: true,
            init: rtl_const_expr::parse_expression("0").unwrap(),
            condition: rtl_const_expr::parse_expression("i < LIMIT").unwrap(),
            step: rtl_const_expr::parse_expression("i + 2").unwrap(),
            label: Some("step2".to_string()),
            body_items: Vec::new(),
        };

        let symbols = HashMap::from([("LIMIT".to_string(), 5)]);
        assert_eq!(
            generate_for.iteration_values(&symbols, 8).unwrap(),
            vec![0, 2, 4]
        );
    }

    #[test]
    fn parses_module_instantiations_with_named_overrides() {
        let design = parse_design(
            r#"
            module child #(parameter WIDTH = 4) (
                input logic [WIDTH-1:0] a,
                output logic [WIDTH-1:0] y
            );
            endmodule

            module top #(parameter TOP_W = 8) (
                input logic [TOP_W-1:0] a,
                output logic [TOP_W-1:0] y
            );
            child #(.WIDTH(TOP_W)) u_child (.a(a), .y(y));
            endmodule
            "#,
        )
        .expect("design should parse");

        let top = &design.modules[1];
        match &top.items[0] {
            ModuleItem::ModuleInstantiation(instantiation) => {
                assert_eq!(instantiation.module_name, "child");
                assert_eq!(instantiation.instance_name, "u_child");
                assert_eq!(instantiation.parameter_overrides.len(), 1);
                assert_eq!(instantiation.port_connections.len(), 2);
                match &instantiation.parameter_overrides[0] {
                    ParameterOverride::Named { name, .. } => assert_eq!(name, "WIDTH"),
                    other => panic!("expected named parameter override, got {other:?}"),
                }
                match &instantiation.port_connections[0] {
                    PortConnection::Named { port, actual } => {
                        assert_eq!(port, "a");
                        assert_eq!(actual, &Some(PortActual::Signal("a".to_string())));
                    }
                    other => panic!("expected named port connection, got {other:?}"),
                }
            }
            other => panic!("expected module instantiation, got {other:?}"),
        }
    }

    #[test]
    fn parses_module_instantiation_arrays() {
        let design = parse_design(
            r#"
            module child (
                input logic a,
                output logic y
            );
            endmodule

            module top (
                input logic a,
                output logic y
            );
            child lanes[1:0] (.a(a), .y(y));
            endmodule
            "#,
        )
        .expect("design should parse");

        let top = &design.modules[1];
        match &top.items[0] {
            ModuleItem::ModuleInstantiation(instantiation) => {
                assert_eq!(instantiation.instance_name, "lanes");
                assert_eq!(
                    instantiation
                        .instance_range
                        .as_ref()
                        .expect("instance array range should exist")
                        .indices(&HashMap::new())
                        .unwrap(),
                    vec![1, 0]
                );
            }
            other => panic!("expected module instantiation, got {other:?}"),
        }
    }

    #[test]
    fn parse_design_rejects_mixed_named_and_ordered_port_connections() {
        let err = parse_design(
            r#"
            module child (
                input logic a,
                input logic b,
                output logic y
            );
            endmodule

            module top (
                input logic a,
                input logic b,
                output logic y
            );
            child u_child (.a(a), b, .y(y));
            endmodule
            "#,
        )
        .expect_err("mixed named and ordered port connections should fail at parse time");

        assert!(
            err.message
                .contains("cannot mix positional and named port connections")
        );
    }

    #[test]
    fn parse_design_rejects_mixed_ordered_and_named_parameter_overrides() {
        let err = parse_design(
            r#"
            module child #(
                parameter WIDTH = 1,
                parameter LANES = 2
            ) (
                input logic a,
                output logic y
            );
            endmodule

            module top (
                input logic a,
                output logic y
            );
            child #(8, .LANES(2)) u_child (a, y);
            endmodule
            "#,
        )
        .expect_err("mixed ordered and named parameter overrides should fail at parse time");

        assert!(
            err.message
                .contains("cannot mix positional and named parameter overrides")
        );
    }

    #[test]
    fn elaborate_top_resolves_child_parameters_and_generate_instances() {
        let design = parse_design(
            r#"
            module leaf #(
                parameter WIDTH = 1,
                parameter OFFSET = WIDTH + 1
            ) (
                input logic [WIDTH-1:0] a,
                output logic [OFFSET-1:0] y
            );
            endmodule

            module top #(
                parameter TOP_W = 8,
                parameter USE_LEAF = 1
            ) (
                input logic [TOP_W-1:0] a,
                output logic [TOP_W:0] y
            );
            leaf #(.WIDTH(TOP_W)) direct (.a(a), .y(y));
            generate
                if (USE_LEAF) begin : gated_scope
                    leaf #(.WIDTH(TOP_W - 1)) gated (.a(a), .y(y));
                end
                for (genvar i = 0; i < 2; i = i + 1) begin : lanes
                    leaf #(.WIDTH(i + 1)) lane (.a(a), .y(y));
                end
            endgenerate
            endmodule
            "#,
        )
        .expect("design should parse");

        let elaborated = design
            .elaborate_top("top", &HashMap::new())
            .expect("top elaboration should succeed");

        assert_eq!(elaborated.parameters.get("TOP_W"), Some(&8));
        assert_eq!(elaborated.child_instances.len(), 4);

        assert_eq!(elaborated.child_instances[0].path, "top.direct");
        assert_eq!(
            elaborated.child_instances[0].parameters.get("WIDTH"),
            Some(&8)
        );
        assert_eq!(
            elaborated.child_instances[0].parameters.get("OFFSET"),
            Some(&9)
        );
        assert_eq!(
            elaborated.child_instances[0].port_bindings[0].actual,
            Some(PortActual::Signal("a".to_string()))
        );

        assert_eq!(elaborated.child_instances[1].path, "top.gated_scope.gated");
        assert_eq!(
            elaborated.child_instances[1].parameters.get("WIDTH"),
            Some(&7)
        );
        assert_eq!(
            elaborated.child_instances[1].parameters.get("OFFSET"),
            Some(&8)
        );

        assert_eq!(elaborated.child_instances[2].path, "top.lanes[0].lane");
        assert_eq!(
            elaborated.child_instances[2].parameters.get("WIDTH"),
            Some(&1)
        );
        assert_eq!(elaborated.child_instances[3].path, "top.lanes[1].lane");
        assert_eq!(
            elaborated.child_instances[3].parameters.get("WIDTH"),
            Some(&2)
        );
    }

    #[test]
    fn elaborate_top_expands_instance_arrays() {
        let design = parse_design(
            r#"
            module child #(parameter WIDTH = 1) (
                input logic a,
                output logic y
            );
            endmodule

            module top #(parameter LANES = 2) (
                input logic a,
                output logic y
            );
            child #(.WIDTH(LANES)) lane[0:LANES-1] (.a(a), .y(y));
            endmodule
            "#,
        )
        .expect("design should parse");

        let elaborated = design
            .elaborate_top("top", &HashMap::new())
            .expect("top elaboration should succeed");

        assert_eq!(elaborated.child_instances.len(), 2);
        assert_eq!(elaborated.child_instances[0].instance_name, "lane[0]");
        assert_eq!(elaborated.child_instances[0].path, "top.lane[0]");
        assert_eq!(
            elaborated.child_instances[0].parameters.get("WIDTH"),
            Some(&2)
        );
        assert_eq!(elaborated.child_instances[1].instance_name, "lane[1]");
        assert_eq!(elaborated.child_instances[1].path, "top.lane[1]");
        assert_eq!(
            elaborated.child_instances[1].port_bindings[0].actual,
            Some(PortActual::Signal("a".to_string()))
        );
    }

    #[test]
    fn wildcard_port_connections_expand_against_child_ports() {
        let design = parse_design(
            r#"
            module child (
                input logic a,
                input logic b,
                output logic y
            );
            endmodule

            module top (
                input logic a,
                input logic b,
                output logic y
            );
            child u_child (.*);
            endmodule
            "#,
        )
        .expect("design should parse");

        let top = &design.modules[1];
        let instantiation = match &top.items[0] {
            ModuleItem::ModuleInstantiation(instantiation) => instantiation,
            other => panic!("expected module instantiation, got {other:?}"),
        };

        let child = design.module("child").expect("child module should exist");
        let visible_names = top.visible_connection_names();
        let visible_types = top.visible_connection_types();
        let bindings = instantiation
            .resolve_port_bindings(child, &visible_names, &visible_types, &HashMap::new())
            .expect("wildcard binding resolution should succeed");

        assert_eq!(
            bindings,
            vec![
                super::ResolvedPortBinding {
                    port_name: "a".to_string(),
                    actual: Some(PortActual::Signal("a".to_string())),
                },
                super::ResolvedPortBinding {
                    port_name: "b".to_string(),
                    actual: Some(PortActual::Signal("b".to_string())),
                },
                super::ResolvedPortBinding {
                    port_name: "y".to_string(),
                    actual: Some(PortActual::Signal("y".to_string())),
                },
            ]
        );
    }

    #[test]
    fn elaboration_preserves_typed_parent_actuals() {
        let design = parse_design(
            r#"
            module child (
                input logic [7:0] a,
                output logic [7:0] y
            );
            endmodule

            module top #(parameter IDX = 3) (
                input logic [7:0] a,
                input logic [7:0] b,
                output logic [7:0] y
            );
            logic [7:0] bus;
            child u_child (.a(bus[IDX]), .y({a, b}));
            endmodule
            "#,
        )
        .expect("design should parse");

        let elaborated = design
            .elaborate_top("top", &HashMap::new())
            .expect("elaboration should succeed");
        let bindings = &elaborated.child_instances[0].port_bindings;

        assert_eq!(
            bindings[0].actual,
            Some(PortActual::BitSelect {
                signal: "bus".to_string(),
                index: rtl_const_expr::parse_expression("IDX").unwrap(),
            })
        );
        assert_eq!(
            bindings[1].actual,
            Some(PortActual::Concat(vec![
                PortActual::Signal("a".to_string()),
                PortActual::Signal("b".to_string()),
            ]))
        );
    }

    #[test]
    fn elaboration_preserves_member_path_and_repeat_actuals() {
        let design = parse_design(
            r#"
            module child (
                input logic [7:0] a,
                output logic [15:0] y
            );
            endmodule

            module top #(
                parameter IDX = 1,
                parameter LANES = 2
            ) (
                input logic [7:0] a,
                output logic [15:0] y
            );
            logic cfg;
            child u_child (.a(cfg.data[IDX]), .y({LANES{a}}));
            endmodule
            "#,
        )
        .expect("design should parse");

        let elaborated = design
            .elaborate_top("top", &HashMap::new())
            .expect("elaboration should succeed");
        let bindings = &elaborated.child_instances[0].port_bindings;

        assert_eq!(
            bindings[0].actual,
            Some(PortActual::BitSelect {
                signal: "cfg.data".to_string(),
                index: rtl_const_expr::parse_expression("IDX").unwrap(),
            })
        );
        assert_eq!(
            bindings[1].actual,
            Some(PortActual::Repeat {
                count: rtl_const_expr::parse_expression("LANES").unwrap(),
                value: Box::new(PortActual::Concat(vec![PortActual::Signal(
                    "a".to_string(),
                )])),
            })
        );
    }

    #[test]
    fn parses_struct_typed_net_declarations() {
        let module = parse_module(
            r#"
            module top (
                output logic y
            );
            struct packed {
                logic [7:0] data;
                logic valid;
            } cfg;
            endmodule
            "#,
        )
        .expect("module should parse");

        match &module.items[0] {
            ModuleItem::NetDecl(net) => {
                assert_eq!(net.kind, NetKind::Typed);
                match net.data_type.as_ref().expect("net type should exist") {
                    DataType::Struct { packed, fields } => {
                        assert!(*packed);
                        assert_eq!(fields.len(), 2);
                        assert_eq!(fields[0].names, vec!["data".to_string()]);
                        assert_eq!(fields[1].names, vec!["valid".to_string()]);
                    }
                    other => panic!("expected struct data type, got {other:?}"),
                }
            }
            other => panic!("expected net declaration, got {other:?}"),
        }
    }

    #[test]
    fn parses_inline_enum_typed_net_declarations() {
        let module = parse_module(
            r#"
            module top (
                output logic y
            );
            enum logic [1:0] {
                IDLE = 0,
                BUSY = 1
            } state;
            endmodule
            "#,
        )
        .expect("module should parse");

        match &module.items[0] {
            ModuleItem::NetDecl(net) => {
                assert_eq!(net.kind, NetKind::Typed);
                match net.data_type.as_ref().expect("net type should exist") {
                    DataType::Enum {
                        base_type,
                        variants,
                    } => {
                        match base_type.as_ref() {
                            DataType::Packed { data_type, .. } => match data_type.as_ref() {
                                DataType::Builtin { keyword } => assert_eq!(keyword, "logic"),
                                other => panic!("expected logic enum base type, got {other:?}"),
                            },
                            other => panic!("expected packed enum base type, got {other:?}"),
                        }
                        assert_eq!(variants.len(), 2);
                        assert_eq!(variants[0].name, "IDLE");
                        assert_eq!(
                            variants[0].value,
                            Some(rtl_const_expr::parse_expression("0").unwrap())
                        );
                        assert_eq!(variants[1].name, "BUSY");
                    }
                    other => panic!("expected enum data type, got {other:?}"),
                }
            }
            other => panic!("expected net declaration, got {other:?}"),
        }
    }

    #[test]
    fn parses_builtin_integral_typed_net_declarations() {
        let module = parse_module(
            r#"
            module top (
                output logic y
            );
            byte lane;
            shortint offset;
            longint cycles;
            endmodule
            "#,
        )
        .expect("module should parse");

        assert_eq!(module.items.len(), 3);
        for (item, expected_keyword) in module
            .items
            .iter()
            .zip(["byte", "shortint", "longint"].into_iter())
        {
            match item {
                ModuleItem::NetDecl(net) => {
                    assert_eq!(net.kind, NetKind::Typed);
                    match net.data_type.as_ref().expect("net type should exist") {
                        DataType::Builtin { keyword } => assert_eq!(keyword, expected_keyword),
                        other => panic!("expected builtin data type, got {other:?}"),
                    }
                }
                other => panic!("expected net declaration, got {other:?}"),
            }
        }
    }

    #[test]
    fn parses_inline_enum_with_byte_base_type() {
        let module = parse_module(
            r#"
            module top (
                output logic y
            );
            enum byte {
                IDLE = 0,
                BUSY = 1
            } state;
            endmodule
            "#,
        )
        .expect("module should parse");

        match &module.items[0] {
            ModuleItem::NetDecl(net) => {
                match net.data_type.as_ref().expect("net type should exist") {
                    DataType::Enum {
                        base_type,
                        variants,
                    } => {
                        assert_eq!(
                            base_type.as_ref(),
                            &DataType::Builtin {
                                keyword: "byte".to_string()
                            }
                        );
                        assert_eq!(variants.len(), 2);
                        assert_eq!(variants[0].name, "IDLE");
                        assert_eq!(variants[1].name, "BUSY");
                    }
                    other => panic!("expected enum data type, got {other:?}"),
                }
            }
            other => panic!("expected net declaration, got {other:?}"),
        }
    }

    #[test]
    fn parses_inline_union_typed_net_declarations() {
        let module = parse_module(
            r#"
            module top (
                output logic y
            );
            union packed {
                logic [7:0] data;
                logic [7:0] byte;
            } payload;
            endmodule
            "#,
        )
        .expect("module should parse");

        match &module.items[0] {
            ModuleItem::NetDecl(net) => {
                assert_eq!(net.kind, NetKind::Typed);
                match net.data_type.as_ref().expect("net type should exist") {
                    DataType::Union { packed, fields } => {
                        assert!(*packed);
                        assert_eq!(fields.len(), 2);
                        assert_eq!(fields[0].names, vec!["data".to_string()]);
                        assert_eq!(fields[1].names, vec!["byte".to_string()]);
                    }
                    other => panic!("expected union data type, got {other:?}"),
                }
            }
            other => panic!("expected net declaration, got {other:?}"),
        }
    }

    #[test]
    fn parses_typedef_struct_declarations_and_named_uses() {
        let module = parse_module(
            r#"
            module top (
                output logic y
            );
            typedef struct packed {
                logic [7:0] data;
                logic valid;
            } cfg_t;
            cfg_t cfg;
            endmodule
            "#,
        )
        .expect("module should parse");

        match &module.items[0] {
            ModuleItem::TypedefDecl(typedef_decl) => {
                assert_eq!(typedef_decl.name, "cfg_t");
                match &typedef_decl.data_type {
                    DataType::Struct { packed, fields } => {
                        assert!(*packed);
                        assert_eq!(fields.len(), 2);
                    }
                    other => panic!("expected typedef struct data type, got {other:?}"),
                }
            }
            other => panic!("expected typedef declaration, got {other:?}"),
        }

        match &module.items[1] {
            ModuleItem::NetDecl(net) => {
                match net.data_type.as_ref().expect("net type should exist") {
                    DataType::Struct { fields, .. } => {
                        assert_eq!(fields[0].names, vec!["data".to_string()]);
                        assert_eq!(fields[1].names, vec!["valid".to_string()]);
                    }
                    other => panic!("expected resolved struct data type, got {other:?}"),
                }
            }
            other => panic!("expected net declaration, got {other:?}"),
        }
    }

    #[test]
    fn parses_typedef_enum_declarations_and_named_uses() {
        let module = parse_module(
            r#"
            module top (
                output logic y
            );
            typedef enum logic [1:0] {
                IDLE = 0,
                BUSY = 1
            } state_t;
            state_t state;
            endmodule
            "#,
        )
        .expect("module should parse");

        match &module.items[0] {
            ModuleItem::TypedefDecl(typedef_decl) => {
                assert_eq!(typedef_decl.name, "state_t");
                match &typedef_decl.data_type {
                    DataType::Enum { variants, .. } => {
                        assert_eq!(variants.len(), 2);
                        assert_eq!(variants[0].name, "IDLE");
                        assert_eq!(variants[1].name, "BUSY");
                    }
                    other => panic!("expected typedef enum data type, got {other:?}"),
                }
            }
            other => panic!("expected typedef declaration, got {other:?}"),
        }

        match &module.items[1] {
            ModuleItem::NetDecl(net) => {
                match net.data_type.as_ref().expect("net type should exist") {
                    DataType::Enum { variants, .. } => {
                        assert_eq!(variants.len(), 2);
                        assert_eq!(variants[0].name, "IDLE");
                        assert_eq!(variants[1].name, "BUSY");
                    }
                    other => panic!("expected resolved enum data type, got {other:?}"),
                }
            }
            other => panic!("expected net declaration, got {other:?}"),
        }
    }

    #[test]
    fn parses_typedef_union_declarations_and_named_uses() {
        let module = parse_module(
            r#"
            module top (
                output logic y
            );
            typedef union packed {
                logic [7:0] data;
                logic [7:0] byte;
            } payload_t;
            payload_t payload;
            endmodule
            "#,
        )
        .expect("module should parse");

        match &module.items[0] {
            ModuleItem::TypedefDecl(typedef_decl) => {
                assert_eq!(typedef_decl.name, "payload_t");
                match &typedef_decl.data_type {
                    DataType::Union { packed, fields } => {
                        assert!(*packed);
                        assert_eq!(fields.len(), 2);
                    }
                    other => panic!("expected typedef union data type, got {other:?}"),
                }
            }
            other => panic!("expected typedef declaration, got {other:?}"),
        }

        match &module.items[1] {
            ModuleItem::NetDecl(net) => {
                match net.data_type.as_ref().expect("net type should exist") {
                    DataType::Union { fields, .. } => {
                        assert_eq!(fields[0].names, vec!["data".to_string()]);
                        assert_eq!(fields[1].names, vec!["byte".to_string()]);
                    }
                    other => panic!("expected resolved union data type, got {other:?}"),
                }
            }
            other => panic!("expected net declaration, got {other:?}"),
        }
    }

    #[test]
    fn parse_design_supports_header_imported_enum_typedefs_for_port_lists() {
        let design = parse_design(
            r#"
            package state_pkg;
                typedef enum logic [1:0] {
                    IDLE = 0,
                    BUSY = 1
                } state_t;
            endpackage

            module top import state_pkg::state_t; (
                input state_t state,
                output logic y
            );
            endmodule
            "#,
        )
        .expect("design should parse");

        match design.modules[0].ports[0]
            .data_type
            .as_ref()
            .expect("port type should exist")
        {
            DataType::Enum { variants, .. } => {
                assert_eq!(variants.len(), 2);
                assert_eq!(variants[0].name, "IDLE");
                assert_eq!(variants[1].name, "BUSY");
            }
            other => panic!("expected header-imported enum type, got {other:?}"),
        }
    }

    #[test]
    fn parse_design_supports_header_imported_union_typedefs_for_port_lists() {
        let design = parse_design(
            r#"
            package payload_pkg;
                typedef union packed {
                    logic [7:0] data;
                    logic [7:0] byte;
                } payload_t;
            endpackage

            module top import payload_pkg::payload_t; (
                input payload_t payload,
                output logic y
            );
            endmodule
            "#,
        )
        .expect("design should parse");

        match design.modules[0].ports[0]
            .data_type
            .as_ref()
            .expect("port type should exist")
        {
            DataType::Union { packed, fields } => {
                assert!(*packed);
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].names, vec!["data".to_string()]);
                assert_eq!(fields[1].names, vec!["byte".to_string()]);
            }
            other => panic!("expected header-imported union type, got {other:?}"),
        }
    }

    #[test]
    fn elaboration_accepts_known_struct_members() {
        let design = parse_design(
            r#"
            module child (
                input logic [7:0] a,
                output logic y
            );
            endmodule

            module top (
                output logic y
            );
            struct packed {
                logic [7:0] data;
                logic valid;
            } cfg;
            child u_child (.a(cfg.data), .y(y));
            endmodule
            "#,
        )
        .expect("design should parse");

        let elaborated = design
            .elaborate_top("top", &HashMap::new())
            .expect("elaboration should succeed");

        assert_eq!(
            elaborated.child_instances[0].port_bindings[0].actual,
            Some(PortActual::Signal("cfg.data".to_string()))
        );
    }

    #[test]
    fn elaboration_accepts_union_members() {
        let design = parse_design(
            r#"
            module child (
                input logic [7:0] a,
                output logic y
            );
            endmodule

            module top (
                output logic y
            );
            union packed {
                logic [7:0] data;
                logic [7:0] byte;
            } payload;
            child u_child (.a(payload.data), .y(y));
            endmodule
            "#,
        )
        .expect("design should parse");

        let elaborated = design
            .elaborate_top("top", &HashMap::new())
            .expect("elaboration should succeed");

        assert_eq!(
            elaborated.child_instances[0].port_bindings[0].actual,
            Some(PortActual::Signal("payload.data".to_string()))
        );
    }

    #[test]
    fn elaboration_rejects_packed_union_width_mismatches() {
        let design = parse_design(
            r#"
            module top (
                output logic y
            );
            union packed {
                logic [7:0] data;
                logic [15:0] word;
            } payload;
            endmodule
            "#,
        )
        .expect("design should parse");

        let error = design
            .elaborate_top("top", &HashMap::new())
            .expect_err("elaboration should reject packed-union width mismatches");
        assert!(
            error
                .message
                .contains("packed union field 'word' width 16 does not match field 'data' width 8")
        );
    }

    #[test]
    fn elaboration_rejects_typedef_backed_packed_union_width_mismatches() {
        let design = parse_design(
            r#"
            module top (
                output logic y
            );
            typedef union packed {
                logic [7:0] data;
                logic [15:0] word;
            } payload_t;
            payload_t payload;
            endmodule
            "#,
        )
        .expect("design should parse");

        let error = design
            .elaborate_top("top", &HashMap::new())
            .expect_err("elaboration should reject typedef-backed packed-union width mismatches");
        assert!(
            error
                .message
                .contains("packed union field 'word' width 16 does not match field 'data' width 8")
        );
    }

    #[test]
    fn elaboration_rejects_builtin_integral_packed_union_width_mismatches() {
        let design = parse_design(
            r#"
            module top (
                output logic y
            );
            union packed {
                byte data;
                shortint word;
            } payload;
            endmodule
            "#,
        )
        .expect("design should parse");

        let error = design
            .elaborate_top("top", &HashMap::new())
            .expect_err("elaboration should reject builtin packed-union width mismatches");
        assert!(
            error
                .message
                .contains("packed union field 'word' width 16 does not match field 'data' width 8")
        );
    }

    #[test]
    fn elaboration_accepts_unpacked_array_struct_members() {
        let design = parse_design(
            r#"
            module child (
                input logic [7:0] a,
                output logic y
            );
            endmodule

            module top #(
                parameter IDX = 1
            ) (
                output logic y
            );
            struct packed {
                logic [7:0] data;
                logic valid;
            } cfgs [0:1];
            child u_child (.a(cfgs[IDX].data), .y(y));
            endmodule
            "#,
        )
        .expect("design should parse");

        let elaborated = design
            .elaborate_top("top", &HashMap::new())
            .expect("elaboration should succeed");

        assert_eq!(
            elaborated.child_instances[0].port_bindings[0].actual,
            Some(PortActual::Signal("cfgs[IDX].data".to_string()))
        );
    }

    #[test]
    fn elaboration_preserves_bitselects_through_unpacked_array_struct_members() {
        let design = parse_design(
            r#"
            module child (
                input logic a,
                output logic y
            );
            endmodule

            module top #(
                parameter IDX = 1,
                parameter BIT = 3
            ) (
                output logic y
            );
            struct packed {
                logic [7:0] data;
                logic valid;
            } cfgs [0:1];
            child u_child (.a(cfgs[IDX].data[BIT]), .y(y));
            endmodule
            "#,
        )
        .expect("design should parse");

        let elaborated = design
            .elaborate_top("top", &HashMap::new())
            .expect("elaboration should succeed");

        assert_eq!(
            elaborated.child_instances[0].port_bindings[0].actual,
            Some(PortActual::BitSelect {
                signal: "cfgs[IDX].data".to_string(),
                index: rtl_const_expr::parse_expression("BIT").unwrap(),
            })
        );
    }

    #[test]
    fn elaboration_accepts_typedef_backed_struct_members() {
        let design = parse_design(
            r#"
            module child (
                input logic [7:0] a,
                output logic y
            );
            endmodule

            module top (
                output logic y
            );
            typedef struct packed {
                logic [7:0] data;
                logic valid;
            } cfg_t;
            cfg_t cfg;
            child u_child (.a(cfg.data), .y(y));
            endmodule
            "#,
        )
        .expect("design should parse");

        let elaborated = design
            .elaborate_top("top", &HashMap::new())
            .expect("elaboration should succeed");

        assert_eq!(
            elaborated.child_instances[0].port_bindings[0].actual,
            Some(PortActual::Signal("cfg.data".to_string()))
        );
    }

    #[test]
    fn elaboration_accepts_file_scope_typedef_backed_struct_members() {
        let design = parse_design(
            r#"
            typedef struct packed {
                logic [7:0] data;
                logic valid;
            } cfg_t;

            module child (
                input logic [7:0] a,
                output logic y
            );
            endmodule

            module top (
                output logic y
            );
            cfg_t cfg;
            child u_child (.a(cfg.data), .y(y));
            endmodule
            "#,
        )
        .expect("design should parse");

        let elaborated = design
            .elaborate_top("top", &HashMap::new())
            .expect("elaboration should succeed");

        assert_eq!(
            elaborated.child_instances[0].port_bindings[0].actual,
            Some(PortActual::Signal("cfg.data".to_string()))
        );
    }

    #[test]
    fn elaboration_accepts_package_import_backed_struct_members() {
        let design = parse_design(
            r#"
            package cfg_pkg;
                typedef struct packed {
                    logic [7:0] data;
                    logic valid;
                } cfg_t;
            endpackage

            module child (
                input logic [7:0] a,
                output logic y
            );
            endmodule

            module top (
                output logic y
            );
            import cfg_pkg::*;
            cfg_t cfg;
            child u_child (.a(cfg.data), .y(y));
            endmodule
            "#,
        )
        .expect("design should parse");

        let elaborated = design
            .elaborate_top("top", &HashMap::new())
            .expect("elaboration should succeed");

        assert_eq!(
            elaborated.child_instances[0].port_bindings[0].actual,
            Some(PortActual::Signal("cfg.data".to_string()))
        );
    }

    #[test]
    fn elaboration_accepts_named_package_import_backed_struct_members() {
        let design = parse_design(
            r#"
            package cfg_pkg;
                typedef struct packed {
                    logic [7:0] data;
                    logic valid;
                } cfg_t;
            endpackage

            module child (
                input logic [7:0] a,
                output logic y
            );
            endmodule

            module top (
                output logic y
            );
            import cfg_pkg::cfg_t;
            cfg_t cfg;
            child u_child (.a(cfg.data), .y(y));
            endmodule
            "#,
        )
        .expect("design should parse");

        let elaborated = design
            .elaborate_top("top", &HashMap::new())
            .expect("elaboration should succeed");

        assert_eq!(
            elaborated.child_instances[0].port_bindings[0].actual,
            Some(PortActual::Signal("cfg.data".to_string()))
        );
    }

    #[test]
    fn elaboration_accepts_header_named_package_import_backed_struct_members() {
        let design = parse_design(
            r#"
            package cfg_pkg;
                typedef struct packed {
                    logic [7:0] data;
                    logic valid;
                } cfg_t;
            endpackage

            module child (
                input logic [7:0] a,
                output logic y
            );
            endmodule

            module top import cfg_pkg::cfg_t; (
                output logic y
            );
            cfg_t cfg;
            child u_child (.a(cfg.data), .y(y));
            endmodule
            "#,
        )
        .expect("design should parse");

        let elaborated = design
            .elaborate_top("top", &HashMap::new())
            .expect("elaboration should succeed");

        assert_eq!(
            elaborated.child_instances[0].port_bindings[0].actual,
            Some(PortActual::Signal("cfg.data".to_string()))
        );
    }

    #[test]
    fn elaboration_rejects_unknown_struct_members() {
        let design = parse_design(
            r#"
            module child (
                input logic [7:0] a,
                output logic y
            );
            endmodule

            module top (
                output logic y
            );
            struct packed {
                logic [7:0] data;
                logic valid;
            } cfg;
            child u_child (.a(cfg.missing), .y(y));
            endmodule
            "#,
        )
        .expect("design should parse");

        let error = design
            .elaborate_top("top", &HashMap::new())
            .expect_err("elaboration should reject unknown struct members");
        assert!(
            error
                .message
                .contains("unknown member 'missing' on parent-scope identifier 'cfg'")
        );
    }

    #[test]
    fn elaboration_rejects_unknown_union_members() {
        let design = parse_design(
            r#"
            module child (
                input logic [7:0] a,
                output logic y
            );
            endmodule

            module top (
                output logic y
            );
            union packed {
                logic [7:0] data;
                logic [7:0] byte;
            } payload;
            child u_child (.a(payload.missing), .y(y));
            endmodule
            "#,
        )
        .expect("design should parse");

        let error = design
            .elaborate_top("top", &HashMap::new())
            .expect_err("elaboration should reject unknown union members");
        assert!(
            error
                .message
                .contains("unknown member 'missing' on parent-scope identifier 'payload'")
        );
    }

    #[test]
    fn elaboration_rejects_unindexed_unpacked_array_members() {
        let design = parse_design(
            r#"
            module child (
                input logic [7:0] a,
                output logic y
            );
            endmodule

            module top (
                output logic y
            );
            struct packed {
                logic [7:0] data;
                logic valid;
            } cfgs [0:1];
            child u_child (.a(cfgs.data), .y(y));
            endmodule
            "#,
        )
        .expect("design should parse");

        let error = design
            .elaborate_top("top", &HashMap::new())
            .expect_err("elaboration should reject unindexed unpacked-array members");
        assert!(error.message.contains(
            "cannot resolve member 'data' through unpacked-array parent-scope identifier 'cfgs'"
        ));
    }

    #[test]
    fn elaboration_rejects_unknown_typedef_backed_struct_members() {
        let design = parse_design(
            r#"
            module child (
                input logic [7:0] a,
                output logic y
            );
            endmodule

            module top (
                output logic y
            );
            typedef struct packed {
                logic [7:0] data;
                logic valid;
            } cfg_t;
            cfg_t cfg;
            child u_child (.a(cfg.missing), .y(y));
            endmodule
            "#,
        )
        .expect("design should parse");

        let error = design
            .elaborate_top("top", &HashMap::new())
            .expect_err("elaboration should reject unknown typedef-backed struct members");
        assert!(
            error
                .message
                .contains("unknown member 'missing' on parent-scope identifier 'cfg'")
        );
    }

    #[test]
    fn elaboration_preserves_expression_actuals_with_member_paths() {
        let design = parse_design(
            r#"
            module child (
                input logic [7:0] a,
                output logic [7:0] y
            );
            endmodule

            module top #(
                parameter SEL = 1
            ) (
                output logic [7:0] y
            );
            logic cfg, backup;
            child u_child (.a(SEL ? cfg.data : backup.data), .y(y));
            endmodule
            "#,
        )
        .expect("design should parse");

        let elaborated = design
            .elaborate_top("top", &HashMap::new())
            .expect("elaboration should succeed");
        let bindings = &elaborated.child_instances[0].port_bindings;

        assert_eq!(
            bindings[0].actual,
            Some(PortActual::Expression(
                rtl_const_expr::parse_expression("SEL ? cfg.data : backup.data").unwrap()
            ))
        );
    }

    #[test]
    fn elaboration_rejects_unknown_parent_actual_identifiers() {
        let design = parse_design(
            r#"
            module child (
                input logic a,
                output logic y
            );
            endmodule

            module top (
                input logic a,
                output logic y
            );
            child u_child (.a(missing_signal), .y(y));
            endmodule
            "#,
        )
        .expect("design should parse");

        let error = design
            .elaborate_top("top", &HashMap::new())
            .expect_err("elaboration should reject unknown actual names");
        assert!(
            error
                .message
                .contains("unknown parent-scope identifier 'missing_signal'")
        );
    }
}
