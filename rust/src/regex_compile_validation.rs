#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegexCompileValidationError {
    pub message: String,
    pub byte_offset: usize,
}

impl RegexCompileValidationError {
    fn new(byte_offset: usize, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            byte_offset,
        }
    }
}

pub fn validate_regex_compile_contract(input: &str) -> Result<(), RegexCompileValidationError> {
    if let Some(error) = find_invalid_escape_i(input) {
        return Err(error);
    }
    if let Some(error) = find_invalid_counted_quantifier(input) {
        return Err(error);
    }
    if let Some(error) = find_invalid_char_class_construct(input) {
        return Err(error);
    }
    if let Some(error) = find_invalid_quantified_anchor(input) {
        return Err(error);
    }
    if let Some(error) = find_variable_length_lookbehind(input) {
        return Err(error);
    }
    Ok(())
}

fn skip_regex_escape(bytes: &[u8], start: usize) -> usize {
    if bytes.get(start) != Some(&b'\\') {
        return start.saturating_add(1).min(bytes.len());
    }

    let Some(next) = bytes.get(start + 1).copied() else {
        return bytes.len();
    };

    if matches!(next, b'x' | b'u' | b'o' | b'p' | b'P' | b'k' | b'g')
        && bytes.get(start + 2) == Some(&b'{')
    {
        if let Some(close) = bytes[start + 3..].iter().position(|byte| *byte == b'}') {
            return start + 4 + close;
        }
    }

    if next == b'c' && bytes.get(start + 2).is_some() {
        return start.saturating_add(3).min(bytes.len());
    }

    start.saturating_add(2).min(bytes.len())
}

fn find_invalid_escape_i(input: &str) -> Option<RegexCompileValidationError> {
    let bytes = input.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => {
                if index + 1 < bytes.len() && bytes[index + 1] == b'i' {
                    return Some(RegexCompileValidationError::new(
                        index,
                        "unsupported regex escape \\i",
                    ));
                }
                index = skip_regex_escape(bytes, index);
            }
            _ => index += 1,
        }
    }
    None
}

fn find_invalid_counted_quantifier(input: &str) -> Option<RegexCompileValidationError> {
    let bytes = input.as_bytes();
    let mut index = 0usize;
    let mut in_char_class = false;

    while index < bytes.len() {
        if in_char_class {
            if bytes[index] == b'\\' {
                index = skip_regex_escape(bytes, index);
                continue;
            }
            if bytes[index] == b']' {
                in_char_class = false;
            }
            index += 1;
            continue;
        }

        match bytes[index] {
            b'\\' => index = skip_regex_escape(bytes, index),
            b'[' => {
                if is_extended_class_start(bytes, index) {
                    index += 1;
                } else {
                    in_char_class = true;
                    index += 1;
                }
            }
            b'{' => {
                if let Some(end) = bytes[index + 1..].iter().position(|byte| *byte == b'}') {
                    let end = index + 1 + end;
                    if let Some(error) = validate_counted_quantifier_body(bytes, index + 1, end) {
                        return Some(error);
                    }
                    index = end + 1;
                } else {
                    index += 1;
                }
            }
            _ => index += 1,
        }
    }

    None
}

fn validate_counted_quantifier_body(
    bytes: &[u8],
    body_start: usize,
    body_end: usize,
) -> Option<RegexCompileValidationError> {
    let body = std::str::from_utf8(&bytes[body_start..body_end])
        .ok()?
        .trim();
    if body.is_empty()
        || !body
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b',' | b' '))
    {
        return None;
    }

    let parts: Vec<&str> = body.split(',').map(str::trim).collect();
    if parts.len() > 2 {
        return None;
    }

    let parse_bound = |raw: &str| -> Option<u32> {
        if raw.is_empty() {
            None
        } else {
            raw.parse::<u32>().ok()
        }
    };

    let reject_bound = |message: &str| {
        Some(RegexCompileValidationError::new(
            body_start.saturating_sub(1),
            message,
        ))
    };

    match parts.as_slice() {
        [single] => {
            let bound = parse_bound(single)?;
            if bound > 65_535 {
                return reject_bound("counted quantifier bound exceeds PCRE2 compile limit 65535");
            }
        }
        [left, right] => {
            let minimum = parse_bound(left);
            let maximum = parse_bound(right);
            if let Some(bound) = minimum {
                if bound > 65_535 {
                    return reject_bound(
                        "counted quantifier minimum exceeds PCRE2 compile limit 65535",
                    );
                }
            }
            if let Some(bound) = maximum {
                if bound > 65_535 {
                    return reject_bound(
                        "counted quantifier maximum exceeds PCRE2 compile limit 65535",
                    );
                }
            }
            if let (Some(minimum), Some(maximum)) = (minimum, maximum) {
                if minimum > maximum {
                    return reject_bound(
                        "counted quantifier minimum cannot exceed counted quantifier maximum",
                    );
                }
            }
        }
        _ => {}
    }

    None
}

fn find_invalid_char_class_construct(input: &str) -> Option<RegexCompileValidationError> {
    let bytes = input.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index = skip_regex_escape(bytes, index),
            b'[' if !is_extended_class_start(bytes, index) => {
                let end = match scan_char_class(bytes, index) {
                    Ok(end) => end,
                    Err(error) => return Some(error),
                };
                index = end + 1;
            }
            _ => index += 1,
        }
    }
    None
}

fn scan_char_class(bytes: &[u8], start: usize) -> Result<usize, RegexCompileValidationError> {
    let mut index = start + 1;

    if index < bytes.len() && bytes[index] == b'^' {
        index += 1;
    }
    if index < bytes.len() && bytes[index] == b']' {
        index += 1;
    }

    while index < bytes.len() {
        if bytes[index] == b']' {
            return Ok(index);
        }

        if bytes[index] == b'[' && bytes.get(index + 1) == Some(&b':') {
            index = skip_posix_class(bytes, index).ok_or_else(|| {
                RegexCompileValidationError::new(start, "unterminated POSIX character class")
            })?;
            continue;
        }

        let (left_atom, after_left) = read_class_atom(bytes, index)?;
        if after_left < bytes.len()
            && bytes[after_left] == b'-'
            && bytes.get(after_left + 1) != Some(&b']')
        {
            let (right_atom, after_right) = read_class_atom(bytes, after_left + 1)?;
            if let (Some(left), Some(right)) = (left_atom, right_atom) {
                if left > right {
                    return Err(RegexCompileValidationError::new(
                        after_left,
                        "descending character class range is not accepted by the regex compile contract",
                    ));
                }
            }
            index = after_right;
            continue;
        }

        index = after_left;
    }

    Err(RegexCompileValidationError::new(
        start,
        "unterminated character class",
    ))
}

fn read_class_atom(
    bytes: &[u8],
    index: usize,
) -> Result<(Option<u8>, usize), RegexCompileValidationError> {
    if index >= bytes.len() {
        return Err(RegexCompileValidationError::new(
            index,
            "unterminated character class",
        ));
    }
    if bytes[index] == b'\\' {
        let next = *bytes.get(index + 1).ok_or_else(|| {
            RegexCompileValidationError::new(index, "unterminated escape in character class")
        })?;
        if matches!(next, b'B' | b'R' | b'X') {
            return Err(RegexCompileValidationError::new(
                index,
                "escape is not accepted inside a character class by the regex compile contract",
            ));
        }
        let after_escape = skip_regex_escape(bytes, index);
        if after_escape > index + 2 {
            return Ok((None, after_escape));
        }
        return Ok((Some(next), after_escape));
    }
    Ok((Some(bytes[index]), index + 1))
}

fn skip_posix_class(bytes: &[u8], start: usize) -> Option<usize> {
    let mut index = start + 2;
    if index < bytes.len() && bytes[index] == b'^' {
        index += 1;
    }
    while index + 1 < bytes.len() {
        if bytes[index] == b':' && bytes[index + 1] == b']' {
            return Some(index + 2);
        }
        index += 1;
    }
    None
}

fn find_invalid_quantified_anchor(input: &str) -> Option<RegexCompileValidationError> {
    let bytes = input.as_bytes();
    let mut index = 0usize;
    let mut in_char_class = false;

    while index < bytes.len() {
        if in_char_class {
            if bytes[index] == b'\\' {
                index = skip_regex_escape(bytes, index);
                continue;
            }
            if bytes[index] == b']' {
                in_char_class = false;
            }
            index += 1;
            continue;
        }

        match bytes[index] {
            b'\\' => index = skip_regex_escape(bytes, index),
            b'[' => {
                if is_extended_class_start(bytes, index) {
                    index += 1;
                } else {
                    in_char_class = true;
                    index += 1;
                }
            }
            b'^' | b'$' => {
                if let Some(quantifier_offset) = quantifier_starts_at(bytes, index + 1) {
                    return Some(RegexCompileValidationError::new(
                        quantifier_offset,
                        "quantifier cannot be applied directly to an anchor",
                    ));
                }
                index += 1;
            }
            _ => index += 1,
        }
    }

    None
}

fn quantifier_starts_at(bytes: &[u8], index: usize) -> Option<usize> {
    if index >= bytes.len() {
        return None;
    }
    match bytes[index] {
        b'*' | b'+' | b'?' => Some(index),
        b'{' => bytes[index + 1..]
            .iter()
            .position(|byte| *byte == b'}')
            .map(|_| index),
        _ => None,
    }
}

fn find_variable_length_lookbehind(input: &str) -> Option<RegexCompileValidationError> {
    let bytes = input.as_bytes();
    let mut index = 0usize;

    while index + 3 < bytes.len() {
        if bytes[index] == b'('
            && bytes.get(index + 1) == Some(&b'?')
            && bytes.get(index + 2) == Some(&b'<')
            && matches!(bytes.get(index + 3), Some(b'=') | Some(b'!'))
        {
            let body_start = index + 4;
            let body_end = find_matching_group_end(bytes, index)?;
            if contains_variable_length_quantifier(bytes, body_start, body_end) {
                return Some(RegexCompileValidationError::new(
                    index,
                    "variable-length lookbehind is not accepted by the regex compile contract",
                ));
            }
            index = body_end + 1;
            continue;
        }
        if bytes[index] == b'\\' {
            index = skip_regex_escape(bytes, index);
        } else {
            index += 1;
        }
    }

    None
}

fn find_matching_group_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut depth = 1usize;
    let mut index = start + 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index = skip_regex_escape(bytes, index),
            b'[' => {
                if let Some(end) = skip_char_class_for_group(bytes, index) {
                    index = end + 1;
                } else {
                    return None;
                }
            }
            b'(' => {
                depth += 1;
                index += 1;
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
                index += 1;
            }
            _ => index += 1,
        }
    }
    None
}

fn skip_char_class_for_group(bytes: &[u8], start: usize) -> Option<usize> {
    let mut index = start + 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index = skip_regex_escape(bytes, index),
            b']' => return Some(index),
            _ => index += 1,
        }
    }
    None
}

fn contains_variable_length_quantifier(bytes: &[u8], start: usize, end: usize) -> bool {
    let mut index = start;
    let mut in_char_class = false;

    while index < end {
        if in_char_class {
            if bytes[index] == b'\\' {
                index = skip_regex_escape(bytes, index);
                continue;
            }
            if bytes[index] == b']' {
                in_char_class = false;
            }
            index += 1;
            continue;
        }

        match bytes[index] {
            b'\\' => index = skip_regex_escape(bytes, index),
            b'[' => {
                in_char_class = true;
                index += 1;
            }
            b'*' | b'+' => return true,
            b'?' => {
                if bytes.get(index.wrapping_sub(1)) != Some(&b'(') {
                    return true;
                }
                index += 1;
            }
            b'{' => {
                if let Some(close) = bytes[index + 1..end].iter().position(|byte| *byte == b'}') {
                    let close = index + 1 + close;
                    let body = std::str::from_utf8(&bytes[index + 1..close])
                        .ok()
                        .unwrap_or("");
                    if counted_quantifier_is_variable(body) {
                        return true;
                    }
                    index = close + 1;
                } else {
                    index += 1;
                }
            }
            _ => index += 1,
        }
    }

    false
}

fn counted_quantifier_is_variable(body: &str) -> bool {
    let body = body.trim();
    if body.is_empty() {
        return false;
    }
    let parts: Vec<&str> = body.split(',').map(str::trim).collect();
    match parts.as_slice() {
        [single] => single.is_empty(),
        [left, right] => {
            if left.is_empty() || right.is_empty() {
                true
            } else {
                left != right
            }
        }
        _ => false,
    }
}

fn is_extended_class_start(bytes: &[u8], index: usize) -> bool {
    index >= 2 && bytes[index - 1] == b'?' && bytes[index - 2] == b'('
}

#[cfg(test)]
mod tests {
    use super::validate_regex_compile_contract;

    #[test]
    fn rejects_invalid_escape_i() {
        let error = validate_regex_compile_contract(r"ab\idef").expect_err("must reject \\i");
        assert!(error.message.contains("\\i"));
    }

    #[test]
    fn rejects_invalid_counted_quantifier_order() {
        let error = validate_regex_compile_contract("x{5,4}").expect_err("must reject {5,4}");
        assert!(error.message.contains("minimum"));
    }

    #[test]
    fn rejects_invalid_counted_quantifier_limit() {
        let error = validate_regex_compile_contract("z{65536}").expect_err("must reject {65536}");
        assert!(error.message.contains("65535"));
    }

    #[test]
    fn rejects_invalid_class_escape() {
        let error = validate_regex_compile_contract(r"[\B]").expect_err("must reject \\B");
        assert!(error.message.contains("class"));
    }

    #[test]
    fn rejects_descending_class_range() {
        let error = validate_regex_compile_contract("[z-a]").expect_err("must reject [z-a]");
        assert!(error.message.contains("descending"));
    }

    #[test]
    fn rejects_quantified_anchor() {
        let error = validate_regex_compile_contract("^*").expect_err("must reject ^*");
        assert!(error.message.contains("anchor"));
    }

    #[test]
    fn rejects_variable_length_lookbehind() {
        let error = validate_regex_compile_contract("(?<=a+)b")
            .expect_err("must reject variable lookbehind");
        assert!(error.message.contains("lookbehind"));
    }

    #[test]
    fn allows_fixed_length_lookbehind_and_quantifier() {
        validate_regex_compile_contract("(?<=a{2})b").expect("fixed-length lookbehind is valid");
    }

    #[test]
    fn allows_braced_octal_escape_without_counted_quantifier_rejection() {
        validate_regex_compile_contract(r"\o{65536}")
            .expect("braced octal escape must not be misclassified as a counted quantifier");
    }

    #[test]
    fn allows_pcre2_literal_malformed_counted_quantifier_forms() {
        for input in [
            "a{1,2,3}b",
            "a{65536",
            "X{",
            "X{A",
            "X{}",
            "X{1234",
            "X{12ABC}",
            "X{1,",
            "X{,9",
            "X{,9]",
            "a{(?#XYZ),2}",
        ] {
            validate_regex_compile_contract(input).unwrap_or_else(|err| {
                panic!("{input:?} should validate as literal braces: {err:?}")
            });
        }
    }

    #[test]
    fn allows_pcre2_control_escape_targets_that_look_like_syntax() {
        validate_regex_compile_contract(r"^\ca\cA\c[;\c:")
            .expect("PCRE2 control escapes should consume their target byte");
    }
}
