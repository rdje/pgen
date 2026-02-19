use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticDirectiveCapability {
    ParsedOnly,
    ParsedAndValidated,
    ParserSteering,
    StimuliSteering,
    ParserAndStimuliSteering,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticAssociativity {
    Left,
    Right,
    NonAssoc,
}

impl SemanticAssociativity {
    pub fn as_str(self) -> &'static str {
        match self {
            SemanticAssociativity::Left => "left",
            SemanticAssociativity::Right => "right",
            SemanticAssociativity::NonAssoc => "nonassoc",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        let normalized = strip_optional_quotes(value).to_ascii_lowercase();
        match normalized.as_str() {
            "left" => Some(SemanticAssociativity::Left),
            "right" => Some(SemanticAssociativity::Right),
            "nonassoc" | "non_assoc" | "non-assoc" | "none" => {
                Some(SemanticAssociativity::NonAssoc)
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SemanticValueConstraints {
    pub enum_values: Vec<String>,
    pub regex_pattern: Option<String>,
    pub min_numeric: Option<f64>,
    pub max_numeric: Option<f64>,
    pub min_len: Option<usize>,
    pub max_len: Option<usize>,
}

impl SemanticValueConstraints {
    pub fn is_empty(&self) -> bool {
        self.enum_values.is_empty()
            && self.regex_pattern.is_none()
            && self.min_numeric.is_none()
            && self.max_numeric.is_none()
            && self.min_len.is_none()
            && self.max_len.is_none()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticDirectiveSpec {
    pub name: &'static str,
    pub capability: SemanticDirectiveCapability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UnknownSemanticDirectivePolicy {
    Ignore,
    #[default]
    Warn,
    Strict,
}

impl UnknownSemanticDirectivePolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            UnknownSemanticDirectivePolicy::Ignore => "ignore",
            UnknownSemanticDirectivePolicy::Warn => "warn",
            UnknownSemanticDirectivePolicy::Strict => "strict",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        let normalized = value.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "ignore" => Some(UnknownSemanticDirectivePolicy::Ignore),
            "warn" | "warning" => Some(UnknownSemanticDirectivePolicy::Warn),
            "strict" | "error" => Some(UnknownSemanticDirectivePolicy::Strict),
            _ => None,
        }
    }
}

const DIRECTIVES: &[SemanticDirectiveSpec] = &[
    // Tier-4 steering path today.
    SemanticDirectiveSpec {
        name: "transform",
        capability: SemanticDirectiveCapability::ParserAndStimuliSteering,
    },
    // Parsed/validated metadata directives.
    SemanticDirectiveSpec {
        name: "type",
        capability: SemanticDirectiveCapability::ParsedAndValidated,
    },
    SemanticDirectiveSpec {
        name: "category",
        capability: SemanticDirectiveCapability::ParsedAndValidated,
    },
    SemanticDirectiveSpec {
        name: "effect",
        capability: SemanticDirectiveCapability::ParsedAndValidated,
    },
    SemanticDirectiveSpec {
        name: "deprecated",
        capability: SemanticDirectiveCapability::ParsedAndValidated,
    },
    // Planned steering directives (registry-first, steering rollout follows).
    SemanticDirectiveSpec {
        name: "sample",
        capability: SemanticDirectiveCapability::ParsedOnly,
    },
    SemanticDirectiveSpec {
        name: "precedence",
        capability: SemanticDirectiveCapability::ParsedOnly,
    },
    SemanticDirectiveSpec {
        name: "associativity",
        capability: SemanticDirectiveCapability::ParsedOnly,
    },
    SemanticDirectiveSpec {
        name: "priority",
        capability: SemanticDirectiveCapability::ParsedOnly,
    },
    SemanticDirectiveSpec {
        name: "weight",
        capability: SemanticDirectiveCapability::ParsedOnly,
    },
    SemanticDirectiveSpec {
        name: "branch_policy",
        capability: SemanticDirectiveCapability::ParsedOnly,
    },
    SemanticDirectiveSpec {
        name: "recover",
        capability: SemanticDirectiveCapability::ParsedOnly,
    },
    SemanticDirectiveSpec {
        name: "sync",
        capability: SemanticDirectiveCapability::ParsedOnly,
    },
    SemanticDirectiveSpec {
        name: "panic_until",
        capability: SemanticDirectiveCapability::ParsedOnly,
    },
    SemanticDirectiveSpec {
        name: "range",
        capability: SemanticDirectiveCapability::ParsedOnly,
    },
    SemanticDirectiveSpec {
        name: "enum",
        capability: SemanticDirectiveCapability::ParsedOnly,
    },
    SemanticDirectiveSpec {
        name: "regex",
        capability: SemanticDirectiveCapability::ParsedOnly,
    },
    SemanticDirectiveSpec {
        name: "len",
        capability: SemanticDirectiveCapability::ParsedOnly,
    },
    SemanticDirectiveSpec {
        name: "constraint",
        capability: SemanticDirectiveCapability::ParsedOnly,
    },
    SemanticDirectiveSpec {
        name: "requires",
        capability: SemanticDirectiveCapability::ParsedOnly,
    },
    SemanticDirectiveSpec {
        name: "implies",
        capability: SemanticDirectiveCapability::ParsedOnly,
    },
    SemanticDirectiveSpec {
        name: "token_class",
        capability: SemanticDirectiveCapability::ParsedOnly,
    },
    SemanticDirectiveSpec {
        name: "charset",
        capability: SemanticDirectiveCapability::ParsedOnly,
    },
    SemanticDirectiveSpec {
        name: "pattern",
        capability: SemanticDirectiveCapability::ParsedOnly,
    },
    SemanticDirectiveSpec {
        name: "coverage_target",
        capability: SemanticDirectiveCapability::ParsedOnly,
    },
    SemanticDirectiveSpec {
        name: "critical_path",
        capability: SemanticDirectiveCapability::ParsedOnly,
    },
    SemanticDirectiveSpec {
        name: "seed_group",
        capability: SemanticDirectiveCapability::ParsedOnly,
    },
    SemanticDirectiveSpec {
        name: "deterministic_group",
        capability: SemanticDirectiveCapability::ParsedOnly,
    },
    // Literal-oriented generation hint directives.
    SemanticDirectiveSpec {
        name: "literal",
        capability: SemanticDirectiveCapability::StimuliSteering,
    },
    SemanticDirectiveSpec {
        name: "example",
        capability: SemanticDirectiveCapability::StimuliSteering,
    },
];

pub fn semantic_directive_spec(name: &str) -> Option<SemanticDirectiveSpec> {
    let normalized = name.trim().to_ascii_lowercase();
    DIRECTIVES
        .iter()
        .copied()
        .find(|spec| spec.name == normalized.as_str())
}

fn directive_name_regex() -> &'static Regex {
    static DIRECTIVE_NAME_RE: OnceLock<Regex> = OnceLock::new();
    DIRECTIVE_NAME_RE.get_or_init(|| {
        Regex::new(r"^\s*@?(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*:")
            .expect("semantic directive name regex must compile")
    })
}

pub fn extract_semantic_directive(content: &str) -> Option<(String, String)> {
    let captures = directive_name_regex().captures(content)?;
    let matched = captures.get(0)?;
    let remainder = &content[matched.end()..];
    if remainder.trim_start().starts_with(':') {
        return None;
    }

    let name = captures.name("name")?.as_str().trim();
    if name.is_empty() {
        return None;
    }

    Some((name.to_ascii_lowercase(), remainder.trim().to_string()))
}

pub fn extract_semantic_directive_name(content: &str) -> Option<String> {
    extract_semantic_directive(content).map(|(name, _)| name)
}

pub fn parse_semantic_numeric_list(payload: &str) -> Option<Vec<i64>> {
    let normalized = payload.trim();
    if normalized.is_empty() {
        return None;
    }

    let inner = if normalized.starts_with('[') && normalized.ends_with(']') {
        &normalized[1..normalized.len() - 1]
    } else {
        normalized
    };

    let mut values = Vec::new();
    for segment in inner.split(',') {
        let raw = strip_optional_quotes(segment);
        if raw.is_empty() {
            continue;
        }
        let parsed = raw.parse::<i64>().ok()?;
        values.push(parsed);
    }

    if values.is_empty() {
        return None;
    }

    Some(values)
}

pub fn parse_semantic_float_list(payload: &str) -> Option<Vec<f64>> {
    let normalized = payload.trim();
    if normalized.is_empty() {
        return None;
    }

    let inner = if normalized.starts_with('[') && normalized.ends_with(']') {
        &normalized[1..normalized.len() - 1]
    } else {
        normalized
    };

    let mut values = Vec::new();
    for segment in inner.split(',') {
        let raw = strip_optional_quotes(segment);
        if raw.is_empty() {
            continue;
        }
        let parsed = raw.parse::<f64>().ok()?;
        values.push(parsed);
    }

    if values.is_empty() {
        return None;
    }

    Some(values)
}

pub fn parse_semantic_string_list(payload: &str) -> Option<Vec<String>> {
    let normalized = payload.trim();
    if normalized.is_empty() {
        return None;
    }

    let inner = if normalized.starts_with('[') && normalized.ends_with(']') {
        &normalized[1..normalized.len() - 1]
    } else {
        normalized
    };

    let mut values = Vec::new();
    for segment in inner.split(',') {
        let raw = normalize_semantic_scalar(segment);
        if raw.is_empty() {
            continue;
        }
        values.push(raw);
    }

    if values.is_empty() {
        return None;
    }

    Some(values)
}

pub fn parse_semantic_numeric_bounds(payload: &str) -> Option<(f64, f64)> {
    let normalized = payload.trim();
    if normalized.is_empty() {
        return None;
    }

    if normalized.contains("..") {
        let mut parts = normalized.splitn(2, "..");
        let lower = normalize_semantic_scalar(parts.next()?);
        let upper = normalize_semantic_scalar(parts.next()?);
        if lower.is_empty() || upper.is_empty() {
            return None;
        }
        let start = lower.parse::<f64>().ok()?;
        let end = upper.parse::<f64>().ok()?;
        return Some(sorted_numeric_bounds(start, end));
    }

    let values = parse_semantic_float_list(normalized)?;
    if values.len() == 1 {
        return Some((values[0], values[0]));
    }
    Some(sorted_numeric_bounds(values[0], values[1]))
}

pub fn parse_semantic_len_bounds(payload: &str) -> Option<(usize, usize)> {
    let normalized = payload.trim();
    if normalized.is_empty() {
        return None;
    }

    if normalized.contains("..") {
        let mut parts = normalized.splitn(2, "..");
        let lower = normalize_semantic_scalar(parts.next()?);
        let upper = normalize_semantic_scalar(parts.next()?);
        if lower.is_empty() || upper.is_empty() {
            return None;
        }
        let start = lower.parse::<usize>().ok()?;
        let end = upper.parse::<usize>().ok()?;
        return Some(sorted_len_bounds(start, end));
    }

    let values = parse_semantic_numeric_list(normalized)?;
    if values.is_empty() {
        return None;
    }
    if values[0] < 0 {
        return None;
    }
    if values.len() == 1 {
        let exact = values[0] as usize;
        return Some((exact, exact));
    }
    if values[1] < 0 {
        return None;
    }
    Some(sorted_len_bounds(values[0] as usize, values[1] as usize))
}

pub fn normalize_semantic_scalar(value: &str) -> String {
    strip_optional_quotes(value).trim().to_string()
}

fn sorted_numeric_bounds(a: f64, b: f64) -> (f64, f64) {
    if a <= b { (a, b) } else { (b, a) }
}

fn sorted_len_bounds(a: usize, b: usize) -> (usize, usize) {
    if a <= b { (a, b) } else { (b, a) }
}

fn strip_optional_quotes(value: &str) -> &str {
    let trimmed = value.trim();
    if trimmed.len() >= 2
        && ((trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\'')))
    {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SemanticAssociativity, UnknownSemanticDirectivePolicy, extract_semantic_directive,
        extract_semantic_directive_name, normalize_semantic_scalar, parse_semantic_float_list,
        parse_semantic_len_bounds, parse_semantic_numeric_bounds, parse_semantic_numeric_list,
        parse_semantic_string_list, semantic_directive_spec,
    };

    #[test]
    fn parses_unknown_directive_policy_strings() {
        assert_eq!(
            UnknownSemanticDirectivePolicy::parse("warn"),
            Some(UnknownSemanticDirectivePolicy::Warn)
        );
        assert_eq!(
            UnknownSemanticDirectivePolicy::parse("strict"),
            Some(UnknownSemanticDirectivePolicy::Strict)
        );
        assert_eq!(
            UnknownSemanticDirectivePolicy::parse("ignore"),
            Some(UnknownSemanticDirectivePolicy::Ignore)
        );
        assert_eq!(UnknownSemanticDirectivePolicy::parse("???"), None);
    }

    #[test]
    fn extracts_semantic_directive_name() {
        assert_eq!(
            extract_semantic_directive_name("@transform: str::parse::<i64>().unwrap_or(0)")
                .as_deref(),
            Some("transform")
        );
        assert_eq!(
            extract_semantic_directive_name("  type: \"Expr\"").as_deref(),
            Some("type")
        );
        assert_eq!(
            extract_semantic_directive_name("str::parse::<i64>().unwrap_or(0)"),
            None
        );
        assert_eq!(extract_semantic_directive_name("no directive"), None);
    }

    #[test]
    fn extracts_semantic_directive_payload() {
        assert_eq!(
            extract_semantic_directive("@priority: [1, 5, 2]"),
            Some(("priority".to_string(), "[1, 5, 2]".to_string()))
        );
        assert_eq!(
            extract_semantic_directive("priority: 9"),
            Some(("priority".to_string(), "9".to_string()))
        );
    }

    #[test]
    fn parses_semantic_numeric_list_payloads() {
        assert_eq!(
            parse_semantic_numeric_list("[1, 2, 3]"),
            Some(vec![1, 2, 3])
        );
        assert_eq!(parse_semantic_numeric_list("9"), Some(vec![9]));
        assert_eq!(parse_semantic_numeric_list("'4', \"6\""), Some(vec![4, 6]));
        assert_eq!(parse_semantic_numeric_list("[]"), None);
        assert_eq!(parse_semantic_numeric_list("[x]"), None);
    }

    #[test]
    fn parses_semantic_float_and_bounds_payloads() {
        assert_eq!(
            parse_semantic_float_list("[1.5, 2.25, 3]"),
            Some(vec![1.5, 2.25, 3.0])
        );
        assert_eq!(parse_semantic_numeric_bounds("[3, 1]"), Some((1.0, 3.0)));
        assert_eq!(parse_semantic_numeric_bounds("2..5"), Some((2.0, 5.0)));
        assert_eq!(parse_semantic_numeric_bounds("4"), Some((4.0, 4.0)));
        assert_eq!(parse_semantic_numeric_bounds("x..5"), None);
    }

    #[test]
    fn parses_semantic_len_bounds_payloads() {
        assert_eq!(parse_semantic_len_bounds("[2, 6]"), Some((2, 6)));
        assert_eq!(parse_semantic_len_bounds("8"), Some((8, 8)));
        assert_eq!(parse_semantic_len_bounds("10..4"), Some((4, 10)));
        assert_eq!(parse_semantic_len_bounds("-1"), None);
    }

    #[test]
    fn parses_semantic_string_lists_and_scalars() {
        assert_eq!(
            parse_semantic_string_list("[\"A\", 'B', C]"),
            Some(vec!["A".to_string(), "B".to_string(), "C".to_string()])
        );
        assert_eq!(
            parse_semantic_string_list("\"single\""),
            Some(vec!["single".to_string()])
        );
        assert_eq!(normalize_semantic_scalar("\"abc\""), "abc".to_string());
    }

    #[test]
    fn parses_semantic_associativity_values() {
        assert_eq!(
            SemanticAssociativity::parse("left"),
            Some(SemanticAssociativity::Left)
        );
        assert_eq!(
            SemanticAssociativity::parse("\"right\""),
            Some(SemanticAssociativity::Right)
        );
        assert_eq!(
            SemanticAssociativity::parse("non-assoc"),
            Some(SemanticAssociativity::NonAssoc)
        );
        assert_eq!(SemanticAssociativity::parse("diagonal"), None);
    }

    #[test]
    fn recognizes_known_directives() {
        assert!(semantic_directive_spec("transform").is_some());
        assert!(semantic_directive_spec("precedence").is_some());
        assert!(semantic_directive_spec("unknown_directive").is_none());
    }
}
