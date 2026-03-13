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

impl Design {
    pub fn module(&self, name: &str) -> Option<&Module> {
        self.modules.iter().find(|module| module.name == name)
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
        let config = ElaborationConfig {
            max_depth,
            max_generate_iterations,
        };
        let module = self.module(module_name).ok_or_else(|| {
            FrontendError::new(format!("unknown top module '{}'", module_name), 0)
        })?;
        let parameters = module.evaluate_constant_env(overrides)?;
        let visible_names = module.visible_connection_names();
        let path = module.name.clone();
        let child_instances = self.elaborate_items(
            &module.items,
            &parameters,
            &visible_names,
            &path,
            0,
            &config,
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
        visible_names: &HashSet<String>,
        scope_path: &str,
        depth: usize,
        config: &ElaborationConfig,
    ) -> Result<Vec<ElaboratedInstance>, FrontendError> {
        let mut instances = Vec::new();

        for item in items {
            match item {
                ModuleItem::ModuleInstantiation(instantiation) => {
                    instances.push(self.elaborate_instance(
                        instantiation,
                        symbols,
                        visible_names,
                        scope_path,
                        depth,
                        config,
                    )?);
                }
                ModuleItem::GenerateRegion(region) => {
                    instances.extend(self.elaborate_items(
                        &region.items,
                        symbols,
                        &merge_visible_names(
                            visible_names,
                            &collect_visible_names_from_items(&region.items),
                        ),
                        scope_path,
                        depth,
                        config,
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
                        let nested_visible_names = merge_visible_names(
                            visible_names,
                            &collect_visible_names_from_items(body_items),
                        );
                        let nested_scope = join_path(scope_path, label.unwrap_or(fallback_scope));
                        instances.extend(self.elaborate_items(
                            body_items,
                            symbols,
                            &nested_visible_names,
                            &nested_scope,
                            depth,
                            config,
                        )?);
                    }
                }
                ModuleItem::GenerateFor(generate_for) => {
                    for iteration in
                        generate_for.iteration_values(symbols, config.max_generate_iterations)?
                    {
                        let mut nested_symbols = symbols.clone();
                        nested_symbols.insert(generate_for.index_name.clone(), iteration);
                        let mut nested_visible_names = merge_visible_names(
                            visible_names,
                            &collect_visible_names_from_items(&generate_for.body_items),
                        );
                        nested_visible_names.insert(generate_for.index_name.clone());
                        let scope_name = match &generate_for.label {
                            Some(label) => format!("{label}[{iteration}]"),
                            None => format!("__gen_for_{}[{iteration}]", generate_for.index_name),
                        };
                        let nested_scope = join_path(scope_path, &scope_name);
                        instances.extend(self.elaborate_items(
                            &generate_for.body_items,
                            &nested_symbols,
                            &nested_visible_names,
                            &nested_scope,
                            depth,
                            config,
                        )?);
                    }
                }
                ModuleItem::ParameterDecl(_)
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
        parent_visible_names: &HashSet<String>,
        scope_path: &str,
        depth: usize,
        config: &ElaborationConfig,
    ) -> Result<ElaboratedInstance, FrontendError> {
        if depth >= config.max_depth {
            return Err(FrontendError::new(
                format!(
                    "elaboration depth exceeded max_depth={} while elaborating '{}'",
                    config.max_depth, instantiation.instance_name
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
        let parameters = module.evaluate_constant_env(&parameter_overrides)?;
        let port_bindings =
            instantiation.resolve_port_bindings(module, parent_visible_names, parent_symbols)?;
        let child_visible_names = module.visible_connection_names();
        let path = join_path(scope_path, &instantiation.instance_name);
        let child_instances = self.elaborate_items(
            &module.items,
            &parameters,
            &child_visible_names,
            &path,
            depth + 1,
            config,
        )?;

        Ok(ElaboratedInstance {
            module_name: module.name.clone(),
            instance_name: instantiation.instance_name.clone(),
            path,
            parameters,
            port_bindings,
            child_instances,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub name: String,
    pub parameters: Vec<ParameterDecl>,
    pub ports: Vec<PortDecl>,
    pub items: Vec<ModuleItem>,
}

impl Module {
    pub fn evaluate_constant_env(
        &self,
        overrides: &HashMap<String, i64>,
    ) -> Result<HashMap<String, i64>, FrontendError> {
        let mut symbols = HashMap::new();
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
}

fn collect_visible_names_from_items(items: &[ModuleItem]) -> HashSet<String> {
    let mut names = HashSet::new();

    for item in items {
        match item {
            ModuleItem::ParameterDecl(parameter) => {
                names.insert(parameter.name.clone());
            }
            ModuleItem::GenvarDecl(genvar) => {
                names.extend(genvar.names.iter().cloned());
            }
            ModuleItem::NetDecl(net) => {
                names.extend(net.names.iter().cloned());
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

fn merge_visible_names(base: &HashSet<String>, extra: &HashSet<String>) -> HashSet<String> {
    let mut merged = base.clone();
    merged.extend(extra.iter().cloned());
    merged
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
pub struct DataType {
    pub keyword: String,
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
    pub name: String,
}

impl PortDecl {
    pub fn width(&self, symbols: &HashMap<String, i64>) -> Result<Option<i64>, FrontendError> {
        self.packed_range
            .as_ref()
            .map(|range| range.width(symbols))
            .transpose()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetKind {
    Logic,
    Wire,
    Reg,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetDecl {
    pub kind: NetKind,
    pub packed_range: Option<PackedRange>,
    pub names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenvarDecl {
    pub names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuousAssign {
    pub target: String,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterOverride {
    Ordered(Expr),
    Named { name: String, value: Expr },
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
    Constant(Expr),
}

impl PortActual {
    fn validate(
        &self,
        visible_names: &HashSet<String>,
        symbols: &HashMap<String, i64>,
    ) -> Result<(), FrontendError> {
        match self {
            PortActual::Signal(name) => validate_known_identifier(name, visible_names, symbols),
            PortActual::BitSelect { signal, index } => {
                validate_known_identifier(signal, visible_names, symbols)?;
                validate_expr_identifiers(index, visible_names, symbols)
            }
            PortActual::PartSelect { signal, msb, lsb } => {
                validate_known_identifier(signal, visible_names, symbols)?;
                validate_expr_identifiers(msb, visible_names, symbols)?;
                validate_expr_identifiers(lsb, visible_names, symbols)
            }
            PortActual::Concat(items) => {
                for item in items {
                    item.validate(visible_names, symbols)?;
                }
                Ok(())
            }
            PortActual::Constant(expr) => validate_expr_identifiers(expr, visible_names, symbols),
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
    pub parameter_overrides: Vec<ParameterOverride>,
    pub port_connections: Vec<PortConnection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPortBinding {
    pub port_name: String,
    pub actual: Option<PortActual>,
}

impl ModuleInstantiation {
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
            }
        }

        Ok(overrides)
    }

    pub fn resolve_port_bindings(
        &self,
        module: &Module,
        parent_visible_names: &HashSet<String>,
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
                        actual.validate(parent_visible_names, parent_symbols)?;
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
                        actual.validate(parent_visible_names, parent_symbols)?;
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
                    actual.validate(parent_visible_names, parent_symbols)?;
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProceduralBlock {
    pub kind: ProceduralKind,
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
        target: String,
        kind: AssignmentKind,
        value: Expr,
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
    symbols: &HashMap<String, i64>,
) -> Result<(), FrontendError> {
    if visible_names.contains(name) || symbols.contains_key(name) {
        Ok(())
    } else {
        Err(FrontendError::new(
            format!("unknown parent-scope identifier '{}'", name),
            0,
        ))
    }
}

fn validate_expr_identifiers(
    expr: &Expr,
    visible_names: &HashSet<String>,
    symbols: &HashMap<String, i64>,
) -> Result<(), FrontendError> {
    let mut identifiers = HashSet::new();
    collect_expr_identifiers(expr, &mut identifiers);
    for identifier in identifiers {
        validate_known_identifier(&identifier, visible_names, symbols)?;
    }
    Ok(())
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
        let mut parts = Vec::new();
        for part in split_top_level(inner, ',')? {
            parts.push(parse_port_actual(part)?);
        }
        return Ok(PortActual::Concat(parts));
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

    let expr = parse_expression(trimmed).map_err(map_eval_error)?;
    Ok(match expr {
        Expr::Ident(name) => PortActual::Signal(name),
        other => PortActual::Constant(other),
    })
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
    if signal.is_empty() || !is_identifier_path(signal) {
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

fn is_identifier_path(input: &str) -> bool {
    let mut segments = input.split('.');
    let Some(first) = segments.next() else {
        return false;
    };
    if !is_identifier_segment(first) {
        return false;
    }
    segments.all(is_identifier_segment)
}

fn is_identifier_segment(input: &str) -> bool {
    let mut chars = input.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !is_ident_start(first) {
        return false;
    }
    chars.all(is_ident_continue)
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
}

impl<'a> Parser<'a> {
    fn new(input: &'a str, tokens: Vec<Token>) -> Self {
        Self {
            input,
            tokens,
            index: 0,
        }
    }

    fn parse_design(mut self) -> Result<Design, FrontendError> {
        let mut modules = Vec::new();
        while !self.is_end() {
            modules.push(self.parse_module()?);
        }
        Ok(Design { modules })
    }

    fn parse_module(&mut self) -> Result<Module, FrontendError> {
        self.expect_keyword("module")?;
        let name = self.expect_identifier()?;
        let parameters = if self.consume_symbol('#') {
            self.expect_symbol('(')?;
            self.parse_parameter_list(')')?
        } else {
            Vec::new()
        };

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

        Ok(Module {
            name,
            parameters,
            ports,
            items,
        })
    }

    fn parse_parameter_list(
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

    fn parse_parameter_decl(
        &mut self,
        flavor: ParameterFlavor,
        expr_terminators: &[char],
    ) -> Result<ParameterDecl, FrontendError> {
        let data_type = self.parse_optional_data_type();
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
            let data_type = self.parse_optional_data_type();
            let packed_range = self.parse_optional_packed_range()?;

            loop {
                let name = self.expect_identifier()?;
                ports.push(PortDecl {
                    direction: direction.clone(),
                    data_type: data_type.clone(),
                    packed_range: packed_range.clone(),
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
        if self.peek_keyword("genvar") {
            return Ok(vec![ModuleItem::GenvarDecl(self.parse_genvar_decl()?)]);
        }
        if self.peek_keyword("logic") || self.peek_keyword("wire") || self.peek_keyword("reg") {
            return Ok(vec![ModuleItem::NetDecl(self.parse_net_decl()?)]);
        }
        if self.peek_keyword("assign") {
            return Ok(vec![ModuleItem::ContinuousAssign(
                self.parse_continuous_assign()?,
            )]);
        }
        if self.looks_like_module_instantiation() {
            return self.parse_module_instantiations();
        }
        if self.peek_keyword("always_comb") || self.peek_keyword("always") {
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
        let mut items = Vec::new();
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

            let decl = self.parse_parameter_decl(flavor, &[',', ';'])?;
            items.push(ModuleItem::ParameterDecl(decl));
            if !self.consume_symbol(',') {
                break;
            }
        }

        self.expect_symbol(';')?;
        Ok(items)
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

    fn parse_net_decl(&mut self) -> Result<NetDecl, FrontendError> {
        let kind = if self.consume_keyword("logic") {
            NetKind::Logic
        } else if self.consume_keyword("wire") {
            NetKind::Wire
        } else if self.consume_keyword("reg") {
            NetKind::Reg
        } else {
            return Err(FrontendError::new(
                "expected net declaration",
                self.current().start,
            ));
        };

        let packed_range = self.parse_optional_packed_range()?;
        let mut names = vec![self.expect_identifier()?];
        while self.consume_symbol(',') {
            names.push(self.expect_identifier()?);
        }
        self.expect_symbol(';')?;

        Ok(NetDecl {
            kind,
            packed_range,
            names,
        })
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
            self.expect_symbol('(')?;
            let port_connections = self.parse_port_connections()?;
            items.push(ModuleItem::ModuleInstantiation(ModuleInstantiation {
                module_name: module_name.clone(),
                instance_name,
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
        if self.consume_symbol(')') {
            return Ok(overrides);
        }

        loop {
            if self.consume_symbol('.') {
                let name = self.expect_identifier()?;
                self.expect_symbol('(')?;
                let value = self.parse_expression_until(&[')'])?;
                self.expect_symbol(')')?;
                overrides.push(ParameterOverride::Named { name, value });
            } else {
                overrides.push(ParameterOverride::Ordered(
                    self.parse_expression_until(&[',', ')'])?,
                ));
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
        if self.consume_symbol(')') {
            return Ok(connections);
        }

        loop {
            if self.consume_symbol('.') {
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
        let target = self.expect_identifier()?;
        self.expect_symbol('=')?;
        let value = self.parse_expression_until(&[';'])?;
        self.expect_symbol(';')?;
        Ok(ContinuousAssign { target, value })
    }

    fn parse_procedural_block(&mut self) -> Result<ProceduralBlock, FrontendError> {
        let kind = if self.consume_keyword("always_comb") {
            ProceduralKind::AlwaysComb
        } else {
            self.expect_keyword("always")?;
            self.expect_symbol('@')?;
            if self.consume_symbol('(') {
                self.expect_symbol('*')?;
                self.expect_symbol(')')?;
            } else {
                self.expect_symbol('*')?;
            }
            ProceduralKind::AlwaysStar
        };

        let statement = self.parse_statement()?;
        Ok(ProceduralBlock { kind, statement })
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

        let target = self.expect_identifier()?;
        let kind = if self.consume_operator("<=") {
            AssignmentKind::NonBlocking
        } else {
            self.expect_symbol('=')?;
            AssignmentKind::Blocking
        };
        let value = self.parse_expression_until(&[';'])?;
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

    fn parse_optional_data_type(&mut self) -> Option<DataType> {
        let keyword = self.current_ident()?.to_string();
        if is_data_type_keyword(&keyword) {
            self.advance();
            Some(DataType { keyword })
        } else {
            None
        }
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
                if let TokenKind::Symbol(':') = token.kind {
                    if ternary_depth > 0 {
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

    fn consume_operator(&mut self, operator: &str) -> bool {
        if matches!(self.current().kind, TokenKind::Operator(value) if value == operator) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn current_is_terminator(&self, terminators: &[char]) -> bool {
        matches!(self.current().kind, TokenKind::Symbol(ch) if terminators.contains(&ch))
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

    fn looks_like_module_instantiation(&self) -> bool {
        let Some(name) = self.current_ident() else {
            return false;
        };
        if is_keyword(name) {
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
            | "assign"
            | "begin"
            | "else"
            | "end"
            | "endgenerate"
            | "endmodule"
            | "for"
            | "generate"
            | "genvar"
            | "if"
            | "inout"
            | "input"
            | "localparam"
            | "logic"
            | "module"
            | "output"
            | "parameter"
            | "reg"
            | "wire"
    )
}

fn is_data_type_keyword(value: &str) -> bool {
    matches!(value, "bit" | "int" | "integer" | "logic" | "reg" | "wire")
}

#[cfg(test)]
mod tests {
    use super::{
        GenerateFor, GenerateIf, ModuleItem, ParameterOverride, PortActual, PortConnection,
        ProceduralKind, Statement, parse_design, parse_module,
    };
    use std::collections::HashMap;

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
        match &block.statement {
            Statement::Block { statements, .. } => assert_eq!(statements.len(), 1),
            other => panic!("expected block statement, got {other:?}"),
        }
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
        let bindings = instantiation
            .resolve_port_bindings(child, &visible_names, &HashMap::new())
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
