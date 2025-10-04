//! Normalization utilities for round-trip testing
//! Handles formatting differences between input and generated output

#[derive(Debug, Clone, Copy)]
pub enum Normalizer {
    Text,
    Float,
    Json,
    Identifier,
}

impl Normalizer {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "float" => Self::Float,
            "json" => Self::Json,
            "identifier" => Self::Identifier,
            _ => Self::Text,
        }
    }
}

pub fn apply_normalizer(normalizer: Normalizer, input: &str, float_precision: Option<usize>) -> String {
    match normalizer {
        Normalizer::Float => normalize_float(input, float_precision.unwrap_or(10)),
        Normalizer::Json => normalize_json(input),
        Normalizer::Identifier => normalize_identifier(input),
        Normalizer::Text => normalize_text(input),
    }
}

fn normalize_text(input: &str) -> String {
    input.trim().to_string()
}

fn normalize_float(input: &str, precision: usize) -> String {
    let trimmed = input.trim();
    if let Ok(f) = trimmed.parse::<f64>() {
        // Format to specified precision and remove trailing zeros
        let formatted = format!("{:.prec$}", f, prec = precision);
        formatted
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    } else {
        normalize_text(trimmed)
    }
}

fn normalize_json(input: &str) -> String {
    // For now, just trim - full JSON normalization would require serde
    normalize_text(input)
}

fn normalize_identifier(input: &str) -> String {
    // Normalize whitespace and case for identifiers
    input.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_float() {
        assert_eq!(normalize_float("3.14000", 10), "3.14");
        assert_eq!(normalize_float("3.0", 10), "3");
        assert_eq!(normalize_float("123.000", 2), "123");
    }

    #[test]
    fn test_normalizer_from_str() {
        assert!(matches!(Normalizer::from_str("float"), Normalizer::Float));
        assert!(matches!(Normalizer::from_str("text"), Normalizer::Text));
        assert!(matches!(Normalizer::from_str("unknown"), Normalizer::Text));
    }
}
