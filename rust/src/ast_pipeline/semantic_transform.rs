use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalSemanticTransform {
    pub target_type: String,
    pub default_expr: String,
}

fn canonical_transform_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(
            r"^\s*str::parse::<(?P<target>[A-Za-z_][A-Za-z0-9_:]*)>\(\)\.unwrap_or\((?P<default>.+)\)\s*$",
        )
        .expect("canonical semantic transform regex must compile")
    })
}

pub fn parse_canonical_transform_expression(
    expression: &str,
) -> Option<CanonicalSemanticTransform> {
    let caps = canonical_transform_regex().captures(expression)?;
    let target_type = caps.name("target")?.as_str().trim();
    let default_expr = caps.name("default")?.as_str().trim();

    if target_type.is_empty() || default_expr.is_empty() {
        return None;
    }

    Some(CanonicalSemanticTransform {
        target_type: target_type.to_string(),
        default_expr: default_expr.to_string(),
    })
}

pub fn stimuli_hint_for_target_type(target_type: &str) -> Option<&'static str> {
    let leaf_type = target_type.rsplit("::").next().unwrap_or(target_type).trim();
    match leaf_type {
        "f32" | "f64" => Some("1.0"),
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64"
        | "u128" | "usize" => Some("1"),
        "bool" => Some("true"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        parse_canonical_transform_expression, stimuli_hint_for_target_type,
    };

    #[test]
    fn parses_canonical_transform_expression() {
        let parsed = parse_canonical_transform_expression("str::parse::<i64>().unwrap_or(0)")
            .expect("canonical transform should parse");
        assert_eq!(parsed.target_type, "i64");
        assert_eq!(parsed.default_expr, "0");
    }

    #[test]
    fn parses_canonical_transform_with_whitespace() {
        let parsed = parse_canonical_transform_expression("  str::parse::<f64>().unwrap_or(0.0) ")
            .expect("canonical transform should parse with outer whitespace");
        assert_eq!(parsed.target_type, "f64");
        assert_eq!(parsed.default_expr, "0.0");
    }

    #[test]
    fn rejects_noncanonical_transform_expression() {
        assert!(
            parse_canonical_transform_expression("str::parse::<i64>().unwrap_or_default()").is_none()
        );
    }

    #[test]
    fn derives_stimuli_hint_for_leaf_target_type() {
        assert_eq!(stimuli_hint_for_target_type("i64"), Some("1"));
        assert_eq!(
            stimuli_hint_for_target_type("std::primitive::f64"),
            Some("1.0")
        );
        assert_eq!(stimuli_hint_for_target_type("bool"), Some("true"));
        assert_eq!(stimuli_hint_for_target_type("String"), None);
    }
}
