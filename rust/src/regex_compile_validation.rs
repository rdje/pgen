#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegexCompileValidationError {
    pub message: String,
    pub byte_offset: usize,
}

const PCRE2_MAX_NAME_SIZE: usize = 128;

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
    if let Some(error) = find_invalid_property_escape(input) {
        return Err(error);
    }
    if let Some(error) = find_invalid_named_escape_or_group_name(input) {
        return Err(error);
    }
    if let Some(error) = find_invalid_counted_quantifier(input) {
        return Err(error);
    }
    if let Some(error) = find_invalid_numeric_callout(input) {
        return Err(error);
    }
    if let Some(error) = find_invalid_verb_construct(input) {
        return Err(error);
    }
    if let Some(error) = find_invalid_char_class_construct(input) {
        return Err(error);
    }
    if let Some(error) = find_invalid_quantified_anchor(input) {
        return Err(error);
    }
    if let Some(error) = find_invalid_scan_substring_capture_list(input) {
        return Err(error);
    }
    if let Some(error) = find_unbounded_quantified_lookbehind(input) {
        return Err(error);
    }
    if let Some(error) = find_invalid_keep_out_escape_in_lookaround(input) {
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

    if next == b'Q' {
        return skip_quoted_literal_escape(bytes, start);
    }

    if matches!(next, b'x' | b'u' | b'o' | b'p' | b'P' | b'k' | b'g' | b'N')
        && bytes.get(start + 2) == Some(&b'{')
        && let Some(close) = bytes[start + 3..].iter().position(|byte| *byte == b'}')
    {
        return start + 4 + close;
    }

    if matches!(next, b'p' | b'P')
        && bytes
            .get(start + 2)
            .copied()
            .is_some_and(is_short_unicode_property_letter)
    {
        return start + 3;
    }

    if next == b'c' && bytes.get(start + 2).is_some() {
        return start.saturating_add(3).min(bytes.len());
    }

    start.saturating_add(2).min(bytes.len())
}

fn skip_quoted_literal_escape(bytes: &[u8], start: usize) -> usize {
    let mut index = start.saturating_add(2).min(bytes.len());
    while index + 1 < bytes.len() {
        if bytes[index] == b'\\' && bytes[index + 1] == b'E' {
            return index + 2;
        }
        index += 1;
    }
    bytes.len()
}

fn find_invalid_escape_i(input: &str) -> Option<RegexCompileValidationError> {
    let bytes = input.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => {
                let unsupported_escape = match bytes.get(index + 1).copied() {
                    Some(b'i') => Some("\\i"),
                    Some(b'F') => Some("\\F"),
                    Some(b'l') => Some("\\l"),
                    Some(b'L') => Some("\\L"),
                    Some(b'u') => Some("\\u"),
                    Some(b'U') => Some("\\U"),
                    _ => None,
                };
                if let Some(escape) = unsupported_escape {
                    return Some(RegexCompileValidationError::new(
                        index,
                        format!("unsupported regex escape {escape}"),
                    ));
                }
                index = skip_regex_escape(bytes, index);
            }
            _ => index += 1,
        }
    }
    None
}

fn find_invalid_property_escape(input: &str) -> Option<RegexCompileValidationError> {
    let bytes = input.as_bytes();
    let mut index = 0usize;

    while index < bytes.len() {
        if bytes[index] != b'\\' {
            index += 1;
            continue;
        }

        match bytes.get(index + 1).copied() {
            Some(b'Q') => {
                index = skip_quoted_literal_escape(bytes, index);
            }
            Some(prefix @ (b'p' | b'P')) => {
                let property_start = index + 2;
                let Some(first) = bytes.get(property_start).copied() else {
                    return Some(RegexCompileValidationError::new(
                        index,
                        "malformed Unicode property escape",
                    ));
                };

                if first == b'{' {
                    index = skip_regex_escape(bytes, index);
                    continue;
                }

                if is_short_unicode_property_letter(first) {
                    index = property_start + 1;
                    continue;
                }

                let escape = if prefix == b'p' { "\\p" } else { "\\P" };
                return Some(RegexCompileValidationError::new(
                    index,
                    format!(
                        "{escape} without braces must use a one-letter Unicode general category"
                    ),
                ));
            }
            _ => {
                index = skip_regex_escape(bytes, index);
            }
        }
    }

    None
}

fn find_invalid_named_escape_or_group_name(input: &str) -> Option<RegexCompileValidationError> {
    let bytes = input.as_bytes();
    let mut index = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'\\' => match bytes.get(index + 1).copied() {
                Some(b'Q') => {
                    index = skip_quoted_literal_escape(bytes, index);
                }
                Some(b'k') => {
                    let name_start = index + 2;
                    let Some((name, after_name)) = read_delimited_name_at(bytes, name_start) else {
                        return Some(RegexCompileValidationError::new(
                            index,
                            "malformed named backreference escape",
                        ));
                    };
                    if !is_pcre2_capture_name(name) {
                        return Some(RegexCompileValidationError::new(
                            index,
                            "invalid named backreference escape",
                        ));
                    }
                    index = after_name;
                }
                _ => index = skip_regex_escape(bytes, index),
            },
            b'[' if !is_extended_class_start(bytes, index) => {
                index = skip_char_class_for_group(bytes, index)
                    .map(|end| end + 1)
                    .unwrap_or(index + 1);
            }
            b'(' => {
                if let Some((name, after_name)) = read_named_group_name_at(bytes, index) {
                    if !is_pcre2_capture_name(name) {
                        return Some(RegexCompileValidationError::new(
                            index,
                            "invalid capture group name",
                        ));
                    }
                    index = after_name;
                } else {
                    index += 1;
                }
            }
            _ => index += 1,
        }
    }

    None
}

fn read_delimited_name_at(bytes: &[u8], start: usize) -> Option<(&str, usize)> {
    let (name_start, terminator) = match bytes.get(start).copied()? {
        b'<' => (start + 1, b'>'),
        b'\'' => (start + 1, b'\''),
        b'{' => (start + 1, b'}'),
        _ => return None,
    };
    let close = bytes[name_start..]
        .iter()
        .position(|byte| *byte == terminator)
        .map(|offset| name_start + offset)?;

    let mut trimmed_start = name_start;
    let mut trimmed_end = close;
    if terminator == b'}' {
        while matches!(bytes.get(trimmed_start), Some(b' ' | b'\t')) && trimmed_start < close {
            trimmed_start += 1;
        }
        while trimmed_end > trimmed_start
            && matches!(bytes.get(trimmed_end - 1), Some(b' ' | b'\t'))
        {
            trimmed_end -= 1;
        }
    }

    let name = std::str::from_utf8(&bytes[trimmed_start..trimmed_end]).ok()?;
    Some((name, close + 1))
}

fn read_named_group_name_at(bytes: &[u8], start: usize) -> Option<(&str, usize)> {
    if bytes.get(start) != Some(&b'(') || bytes.get(start + 1) != Some(&b'?') {
        return None;
    }

    match bytes.get(start + 2).copied()? {
        b'<' => {
            if matches!(bytes.get(start + 3), Some(b'=') | Some(b'!')) {
                return None;
            }
            read_delimited_name_at(bytes, start + 2)
        }
        b'\'' => read_delimited_name_at(bytes, start + 2),
        b'P' if bytes.get(start + 3) == Some(&b'<') => read_delimited_name_at(bytes, start + 3),
        _ => None,
    }
}

fn is_pcre2_capture_name(name: &str) -> bool {
    if name.is_empty() || name.len() > PCRE2_MAX_NAME_SIZE {
        return false;
    }

    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if is_pcre2_name_digit(first) {
        return false;
    }
    if !is_pcre2_name_char(first) {
        return false;
    }

    chars.all(is_pcre2_name_char)
}

fn is_pcre2_name_char(ch: char) -> bool {
    ch == '_' || ch.is_alphabetic() || is_pcre2_name_digit(ch)
}

fn is_pcre2_name_digit(ch: char) -> bool {
    ch.is_ascii_digit() || (!ch.is_ascii() && ch.is_numeric())
}

fn is_short_unicode_property_letter(byte: u8) -> bool {
    matches!(
        byte,
        b'C' | b'L'
            | b'M'
            | b'N'
            | b'P'
            | b'S'
            | b'Z'
            | b'c'
            | b'l'
            | b'm'
            | b'n'
            | b'p'
            | b's'
            | b'z'
    )
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

fn find_invalid_numeric_callout(input: &str) -> Option<RegexCompileValidationError> {
    let bytes = input.as_bytes();
    let mut index = 0usize;

    while index + 3 < bytes.len() {
        match bytes[index] {
            b'\\' => index = skip_regex_escape(bytes, index),
            b'[' if !is_extended_class_start(bytes, index) => {
                index = skip_char_class_for_group(bytes, index)
                    .map(|end| end + 1)
                    .unwrap_or(index + 1);
            }
            b'(' if bytes.get(index + 1) == Some(&b'?') && bytes.get(index + 2) == Some(&b'C') => {
                let mut arg_index = index + 3;
                let mut number = 0u16;
                while let Some(digit) = bytes.get(arg_index).copied() {
                    if !digit.is_ascii_digit() {
                        break;
                    }
                    number = number
                        .saturating_mul(10)
                        .saturating_add(u16::from(digit - b'0'));
                    if number > 255 {
                        return Some(RegexCompileValidationError::new(
                            index,
                            "numeric callout argument exceeds PCRE2 compile limit 255",
                        ));
                    }
                    arg_index += 1;
                }
                index = arg_index.max(index + 1);
            }
            _ => index += 1,
        }
    }

    None
}

fn find_invalid_verb_construct(input: &str) -> Option<RegexCompileValidationError> {
    let bytes = input.as_bytes();
    let mut index = 0usize;

    while index + 2 < bytes.len() {
        match bytes[index] {
            b'\\' => index = skip_regex_escape(bytes, index),
            b'[' if !is_extended_class_start(bytes, index) => {
                index = skip_char_class_for_group(bytes, index)
                    .map(|end| end + 1)
                    .unwrap_or(index + 1);
            }
            b'(' if bytes.get(index + 1) == Some(&b'*') => {
                let name_start = index + 2;
                let mut cursor = name_start;
                while let Some(byte) = bytes.get(cursor).copied() {
                    if matches!(byte, b':' | b')' | b'=') {
                        break;
                    }
                    cursor += 1;
                }

                let Some(delimiter) = bytes.get(cursor).copied() else {
                    index += 1;
                    continue;
                };
                let name = std::str::from_utf8(&bytes[name_start..cursor]).ok()?;

                if is_non_verb_star_group_name(name) {
                    index += 1;
                    continue;
                }

                if name.is_empty() {
                    if delimiter != b':' || bytes.get(cursor + 1) == Some(&b')') {
                        return Some(RegexCompileValidationError::new(
                            index,
                            "MARK shorthand verb requires a non-empty argument",
                        ));
                    }
                    if let Some(group_end) = find_star_verb_end(bytes, index) {
                        if quantifier_starts_at(bytes, group_end + 1).is_some() {
                            return Some(RegexCompileValidationError::new(
                                group_end + 1,
                                "only ACCEPT verb may be quantified by the regex compile contract",
                            ));
                        }
                        index = group_end + 1;
                        continue;
                    }
                }

                if is_pcre2_start_option_name(name) {
                    if delimiter == b'=' {
                        let value_start = cursor + 1;
                        let mut value_end = value_start;
                        while bytes
                            .get(value_end)
                            .is_some_and(|byte| byte.is_ascii_digit())
                        {
                            value_end += 1;
                        }
                        if value_end == value_start || bytes.get(value_end) != Some(&b')') {
                            return Some(RegexCompileValidationError::new(
                                index,
                                "PCRE2 start option with '=' requires a numeric value",
                            ));
                        }
                    } else if delimiter != b')' {
                        return Some(RegexCompileValidationError::new(
                            index,
                            "PCRE2 start option does not accept this delimiter",
                        ));
                    } else if !is_start_option_position(bytes, index) {
                        return Some(RegexCompileValidationError::new(
                            index,
                            "PCRE2 start option must appear at the start-option prefix",
                        ));
                    }
                    if let Some(group_end) = find_star_verb_end(bytes, index) {
                        index = group_end + 1;
                        continue;
                    }
                }

                if let Some(argument_rule) = pcre2_verb_argument_rule(name) {
                    if delimiter != b':' && delimiter != b')' {
                        return Some(RegexCompileValidationError::new(
                            index,
                            "PCRE2 verb is malformed",
                        ));
                    }
                    if argument_rule == VerbArgumentRule::Required
                        && (delimiter != b':' || bytes.get(cursor + 1) == Some(&b')'))
                    {
                        return Some(RegexCompileValidationError::new(
                            index,
                            "MARK verb requires a non-empty argument",
                        ));
                    }
                    if let Some(group_end) = find_star_verb_end(bytes, index) {
                        if !matches!(name, "ACCEPT")
                            && quantifier_starts_at(bytes, group_end + 1).is_some()
                        {
                            return Some(RegexCompileValidationError::new(
                                group_end + 1,
                                "only ACCEPT verb may be quantified by the regex compile contract",
                            ));
                        }
                        index = group_end + 1;
                        continue;
                    }
                } else {
                    return Some(RegexCompileValidationError::new(
                        index,
                        "unrecognized PCRE2 verb or start option",
                    ));
                }
            }
            _ => index += 1,
        }
    }

    None
}

fn find_star_verb_end(bytes: &[u8], start: usize) -> Option<usize> {
    bytes[start + 2..]
        .iter()
        .position(|byte| *byte == b')')
        .map(|offset| start + 2 + offset)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum VerbArgumentRule {
    Optional,
    Required,
}

fn pcre2_verb_argument_rule(name: &str) -> Option<VerbArgumentRule> {
    match name {
        "MARK" => Some(VerbArgumentRule::Required),
        "ACCEPT" | "F" | "FAIL" | "COMMIT" | "PRUNE" | "SKIP" | "THEN" => {
            Some(VerbArgumentRule::Optional)
        }
        _ => None,
    }
}

fn is_non_verb_star_group_name(name: &str) -> bool {
    matches!(
        name,
        "atomic"
            | "pla"
            | "positive_lookahead"
            | "nla"
            | "negative_lookahead"
            | "plb"
            | "positive_lookbehind"
            | "nlb"
            | "negative_lookbehind"
            | "napla"
            | "non_atomic_positive_lookahead"
            | "naplb"
            | "non_atomic_positive_lookbehind"
            | "scs"
            | "scan_substring"
            | "sr"
            | "script_run"
            | "asr"
            | "atomic_script_run"
    )
}

fn is_pcre2_start_option_name(name: &str) -> bool {
    matches!(
        name,
        "UTF"
            | "UTF8"
            | "UTF16"
            | "UTF32"
            | "UCP"
            | "NOTEMPTY"
            | "NOTEMPTY_ATSTART"
            | "NO_AUTO_POSSESS"
            | "NO_DOTSTAR_ANCHOR"
            | "NO_JIT"
            | "NO_START_OPT"
            | "CASELESS_RESTRICT"
            | "TURKISH_CASING"
            | "LIMIT_HEAP"
            | "LIMIT_MATCH"
            | "LIMIT_DEPTH"
            | "LIMIT_RECURSION"
            | "CR"
            | "LF"
            | "CRLF"
            | "ANY"
            | "NUL"
            | "ANYCRLF"
            | "BSR_ANYCRLF"
            | "BSR_UNICODE"
    )
}

fn is_start_option_position(bytes: &[u8], index: usize) -> bool {
    let mut cursor = 0usize;
    while cursor < index {
        if bytes.get(cursor) != Some(&b'(') || bytes.get(cursor + 1) != Some(&b'*') {
            return false;
        }
        let Some(group_end) = find_matching_group_end(bytes, cursor) else {
            return false;
        };
        let mut name_start = cursor + 2;
        while bytes
            .get(name_start)
            .is_some_and(|byte| matches!(*byte, b' ' | b'\t'))
        {
            name_start += 1;
        }
        let mut name_end = name_start;
        while bytes
            .get(name_end)
            .is_some_and(|byte| !matches!(*byte, b')' | b'='))
        {
            name_end += 1;
        }
        let Ok(name) = std::str::from_utf8(&bytes[name_start..name_end]) else {
            return false;
        };
        if !is_pcre2_start_option_name(name) {
            return false;
        }
        cursor = group_end + 1;
    }
    cursor == index
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
            if let Some(bound) = minimum
                && bound > 65_535
            {
                return reject_bound(
                    "counted quantifier minimum exceeds PCRE2 compile limit 65535",
                );
            }
            if let Some(bound) = maximum
                && bound > 65_535
            {
                return reject_bound(
                    "counted quantifier maximum exceeds PCRE2 compile limit 65535",
                );
            }
            if let (Some(minimum), Some(maximum)) = (minimum, maximum)
                && minimum > maximum
            {
                return reject_bound(
                    "counted quantifier minimum cannot exceed counted quantifier maximum",
                );
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
                if let Some(after_alias) = skip_pcre2_posix_word_boundary_alias(bytes, index) {
                    index = after_alias;
                    continue;
                }
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

fn skip_pcre2_posix_word_boundary_alias(bytes: &[u8], start: usize) -> Option<usize> {
    const START_WORD_ALIAS: &[u8] = b"[[:<:]]";
    const END_WORD_ALIAS: &[u8] = b"[[:>:]]";
    let rest = bytes.get(start..)?;
    if rest.starts_with(START_WORD_ALIAS) {
        return Some(start + START_WORD_ALIAS.len());
    }
    if rest.starts_with(END_WORD_ALIAS) {
        return Some(start + END_WORD_ALIAS.len());
    }
    None
}

#[derive(Clone, Copy)]
enum ClassAtomKind {
    Literal(u8),
    NonLiteral,
    ZeroWidth,
}

impl ClassAtomKind {
    fn is_substantive(self) -> bool {
        !matches!(self, Self::ZeroWidth)
    }
}

fn scan_char_class(bytes: &[u8], start: usize) -> Result<usize, RegexCompileValidationError> {
    let mut index = start + 1;
    let mut has_substantive_item = false;
    let mut previous_atom: Option<ClassAtomKind> = None;

    if index < bytes.len() && bytes[index] == b'^' {
        index += 1;
    }
    if index < bytes.len() && bytes[index] == b']' {
        has_substantive_item = true;
        previous_atom = Some(ClassAtomKind::Literal(b']'));
        index += 1;
    }

    while index < bytes.len() {
        if bytes[index] == b']' {
            if has_substantive_item {
                return Ok(index);
            }
            return Err(RegexCompileValidationError::new(
                start,
                "unterminated character class",
            ));
        }

        if bytes[index] == b'^' && !has_substantive_item {
            index += 1;
            continue;
        }

        if bytes[index] == b'-'
            && bytes.get(index + 1) != Some(&b']')
            && !dash_is_trailing_literal(bytes, index)
            && !dash_starts_alt_extended_class_operator(bytes, index)
            && let Some(left) = previous_atom
        {
            let (right, after_right) = read_substantive_class_atom(bytes, index + 1)?;
            let Some(right) = right else {
                return Err(RegexCompileValidationError::new(
                    index,
                    "invalid character class range is not accepted by the regex compile contract",
                ));
            };
            if matches!(left, ClassAtomKind::NonLiteral) {
                return Err(RegexCompileValidationError::new(
                    index,
                    "invalid character class range is not accepted by the regex compile contract",
                ));
            }
            if matches!(right, ClassAtomKind::NonLiteral) {
                return Err(RegexCompileValidationError::new(
                    index,
                    "invalid character class range is not accepted by the regex compile contract",
                ));
            }
            if let (ClassAtomKind::Literal(left), ClassAtomKind::Literal(right)) = (left, right)
                && left > right
            {
                return Err(RegexCompileValidationError::new(
                    index,
                    "descending character class range is not accepted by the regex compile contract",
                ));
            }
            index = after_right;
            previous_atom = None;
            has_substantive_item = true;
            continue;
        }

        if bytes[index] == b'['
            && bytes.get(index + 1) == Some(&b':')
            && let Some(after_posix_class) = scan_posix_class(bytes, index)?
        {
            index = after_posix_class;
            has_substantive_item = true;
            previous_atom = Some(ClassAtomKind::NonLiteral);
            continue;
        }

        let (left_atom, after_left) = read_class_atom(bytes, index)?;
        if !left_atom.is_substantive() {
            index = after_left;
            continue;
        }
        if after_left < bytes.len()
            && bytes[after_left] == b'-'
            && bytes.get(after_left + 1) != Some(&b']')
            && !dash_is_trailing_literal(bytes, after_left)
            && !dash_starts_alt_extended_class_operator(bytes, after_left)
        {
            let (right_atom, after_right) = read_substantive_class_atom(bytes, after_left + 1)?;
            let Some(right_atom) = right_atom else {
                return Err(RegexCompileValidationError::new(
                    after_left,
                    "invalid character class range is not accepted by the regex compile contract",
                ));
            };
            if matches!(left_atom, ClassAtomKind::NonLiteral)
                || matches!(right_atom, ClassAtomKind::NonLiteral)
            {
                return Err(RegexCompileValidationError::new(
                    after_left,
                    "invalid character class range is not accepted by the regex compile contract",
                ));
            }
            if let (ClassAtomKind::Literal(left), ClassAtomKind::Literal(right)) =
                (left_atom, right_atom)
                && left > right
            {
                return Err(RegexCompileValidationError::new(
                    after_left,
                    "descending character class range is not accepted by the regex compile contract",
                ));
            }
            index = after_right;
            previous_atom = None;
            has_substantive_item = true;
            continue;
        }

        index = after_left;
        previous_atom = Some(left_atom);
        has_substantive_item = true;
    }

    Err(RegexCompileValidationError::new(
        start,
        "unterminated character class",
    ))
}

fn read_class_atom(
    bytes: &[u8],
    index: usize,
) -> Result<(ClassAtomKind, usize), RegexCompileValidationError> {
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
        if next == b'Q' {
            return Ok(read_quoted_class_atom(bytes, index));
        }
        if next == b'E' {
            return Ok((ClassAtomKind::ZeroWidth, index + 2));
        }
        if next == b'N' && bytes.get(index + 2) != Some(&b'{') {
            return Err(RegexCompileValidationError::new(
                index,
                "\\N is not accepted inside a character class by the regex compile contract",
            ));
        }
        if matches!(
            next,
            b'A' | b'B' | b'C' | b'G' | b'K' | b'Q' | b'R' | b'X' | b'Z' | b'z'
        ) {
            return Err(RegexCompileValidationError::new(
                index,
                "escape is not accepted inside a character class by the regex compile contract",
            ));
        }
        let after_escape = skip_regex_escape(bytes, index);
        if is_nonliteral_class_escape(bytes, index, after_escape) {
            return Ok((ClassAtomKind::NonLiteral, after_escape));
        }
        return Ok((ClassAtomKind::Literal(next), after_escape));
    }
    Ok((ClassAtomKind::Literal(bytes[index]), index + 1))
}

fn is_nonliteral_class_escape(bytes: &[u8], index: usize, after_escape: usize) -> bool {
    let Some(next) = bytes.get(index + 1).copied() else {
        return false;
    };
    matches!(next, b'd' | b'D' | b'h' | b'H' | b's' | b'S' | b'w' | b'W')
        || matches!(next, b'p' | b'P') && after_escape > index + 2
}

fn read_substantive_class_atom(
    bytes: &[u8],
    mut index: usize,
) -> Result<(Option<ClassAtomKind>, usize), RegexCompileValidationError> {
    while index < bytes.len() && bytes[index] != b']' {
        let (atom, after_atom) = read_class_atom(bytes, index)?;
        if atom.is_substantive() {
            return Ok((Some(atom), after_atom));
        }
        if after_atom <= index {
            return Ok((None, after_atom));
        }
        index = after_atom;
    }
    Ok((None, index))
}

fn read_quoted_class_atom(bytes: &[u8], start: usize) -> (ClassAtomKind, usize) {
    let mut index = start.saturating_add(2).min(bytes.len());
    let mut first_literal = None;
    while index + 1 < bytes.len() {
        if bytes[index] == b'\\' && bytes[index + 1] == b'E' {
            let atom = first_literal
                .map(ClassAtomKind::Literal)
                .unwrap_or(ClassAtomKind::ZeroWidth);
            return (atom, index + 2);
        }
        first_literal.get_or_insert(bytes[index]);
        index += 1;
    }
    let atom = first_literal
        .map(ClassAtomKind::Literal)
        .unwrap_or(ClassAtomKind::ZeroWidth);
    (atom, bytes.len())
}

fn dash_is_trailing_literal(bytes: &[u8], dash_index: usize) -> bool {
    let mut index = dash_index + 1;
    loop {
        while matches!(bytes.get(index), Some(b' ' | b'\t')) {
            index += 1;
        }
        if bytes.get(index) == Some(&b'\\') && bytes.get(index + 1) == Some(&b'Q') {
            let (quoted_atom, after_quote) = read_quoted_class_atom(bytes, index);
            if matches!(quoted_atom, ClassAtomKind::ZeroWidth) {
                index = after_quote;
                continue;
            }
        }
        if bytes.get(index) == Some(&b'\\') && bytes.get(index + 1) == Some(&b'E') {
            index += 2;
            continue;
        }
        break;
    }
    bytes.get(index) == Some(&b']')
}

fn dash_starts_alt_extended_class_operator(bytes: &[u8], dash_index: usize) -> bool {
    bytes.get(dash_index + 1) == Some(&b'[')
        || (bytes.get(dash_index + 1) == Some(&b'|') && bytes.get(dash_index + 2) == Some(&b'|'))
}

fn scan_posix_class(
    bytes: &[u8],
    start: usize,
) -> Result<Option<usize>, RegexCompileValidationError> {
    let mut index = start + 2;
    if index < bytes.len() && bytes[index] == b'^' {
        index += 1;
    }
    let name_start = index;
    while index + 1 < bytes.len() {
        if bytes[index] == b':' && bytes[index + 1] == b']' {
            let name = std::str::from_utf8(&bytes[name_start..index]).unwrap_or("");
            if !is_valid_posix_class_name(name) {
                return Err(RegexCompileValidationError::new(
                    start,
                    "unknown POSIX character class name",
                ));
            }
            return Ok(Some(index + 2));
        }
        index += 1;
    }
    Ok(None)
}

fn is_valid_posix_class_name(name: &str) -> bool {
    matches!(
        name,
        "alnum"
            | "alpha"
            | "ascii"
            | "blank"
            | "cntrl"
            | "digit"
            | "graph"
            | "lower"
            | "print"
            | "punct"
            | "space"
            | "upper"
            | "word"
            | "xdigit"
    )
}

fn find_invalid_scan_substring_capture_list(input: &str) -> Option<RegexCompileValidationError> {
    let bytes = input.as_bytes();
    let mut index = 0usize;
    let full_inventory = capture_inventory_before(bytes, bytes.len());

    while index + 6 < bytes.len() {
        match bytes[index] {
            b'\\' => index = skip_regex_escape(bytes, index),
            b'[' if !is_extended_class_start(bytes, index) => {
                index = skip_char_class_for_group(bytes, index)
                    .map(|end| end + 1)
                    .unwrap_or(index + 1);
            }
            b'(' if bytes.get(index + 1) == Some(&b'*') => {
                let Some(list_start) = scan_substring_capture_list_start(bytes, index) else {
                    index += 1;
                    continue;
                };
                let Some(list_end) = find_matching_group_end(bytes, list_start) else {
                    index += 1;
                    continue;
                };
                let prior_inventory = capture_inventory_before(bytes, index);
                if let Some(error) = validate_scan_substring_capture_refs(
                    bytes,
                    list_start + 1,
                    list_end,
                    &prior_inventory,
                    &full_inventory,
                ) {
                    return Some(error);
                }
                index = list_end + 1;
            }
            _ => index += 1,
        }
    }

    None
}

fn scan_substring_capture_list_start(bytes: &[u8], index: usize) -> Option<usize> {
    for prefix in [b"(*scs:".as_slice(), b"(*scan_substring:".as_slice()] {
        if bytes.get(index..index + prefix.len()) == Some(prefix) {
            let list_start = index + prefix.len();
            return (bytes.get(list_start) == Some(&b'(')).then_some(list_start);
        }
    }
    None
}

#[derive(Default)]
struct CaptureInventory {
    count: usize,
    names: std::collections::BTreeSet<String>,
}

fn capture_inventory_before(bytes: &[u8], end: usize) -> CaptureInventory {
    let mut inventory = CaptureInventory::default();
    let mut index = 0usize;

    while index < end {
        match bytes[index] {
            b'\\' => index = skip_regex_escape(bytes, index),
            b'[' if !is_extended_class_start(bytes, index) => {
                index = skip_char_class_for_group(bytes, index)
                    .map(|class_end| class_end + 1)
                    .unwrap_or(index + 1);
            }
            b'(' if bytes.get(index + 1) == Some(&b'?') => {
                if bytes.get(index + 2) == Some(&b'#') {
                    index = bytes[index + 3..end]
                        .iter()
                        .position(|byte| *byte == b')')
                        .map(|offset| index + 4 + offset)
                        .unwrap_or(end);
                    continue;
                }
                if let Some((name, after_name)) = capture_name_at(bytes, index) {
                    inventory.count += 1;
                    inventory.names.insert(name.to_string());
                    index = after_name;
                } else {
                    index += 1;
                }
            }
            b'(' if bytes.get(index + 1) == Some(&b'*') => {
                if let Some(list_start) = scan_substring_capture_list_start(bytes, index)
                    && let Some(list_end) = find_matching_group_end(bytes, list_start)
                {
                    index = list_end + 1;
                } else {
                    index += 2;
                }
            }
            b'(' => {
                inventory.count += 1;
                index += 1;
            }
            _ => index += 1,
        }
    }

    inventory
}

fn capture_name_at(bytes: &[u8], group_start: usize) -> Option<(&str, usize)> {
    if bytes.get(group_start + 2) == Some(&b'<') {
        if matches!(
            bytes.get(group_start + 3),
            Some(b'=') | Some(b'!') | Some(b'*')
        ) {
            return None;
        }
        let name_start = group_start + 3;
        let name_end = bytes[name_start..]
            .iter()
            .position(|byte| *byte == b'>')
            .map(|offset| name_start + offset)?;
        return std::str::from_utf8(&bytes[name_start..name_end])
            .ok()
            .map(|name| (name, name_end + 1));
    }

    if bytes.get(group_start + 2) == Some(&b'\'') {
        let name_start = group_start + 3;
        let name_end = bytes[name_start..]
            .iter()
            .position(|byte| *byte == b'\'')
            .map(|offset| name_start + offset)?;
        return std::str::from_utf8(&bytes[name_start..name_end])
            .ok()
            .map(|name| (name, name_end + 1));
    }

    if bytes.get(group_start + 2) == Some(&b'P') && bytes.get(group_start + 3) == Some(&b'<') {
        let name_start = group_start + 4;
        let name_end = bytes[name_start..]
            .iter()
            .position(|byte| *byte == b'>')
            .map(|offset| name_start + offset)?;
        return std::str::from_utf8(&bytes[name_start..name_end])
            .ok()
            .map(|name| (name, name_end + 1));
    }

    None
}

fn validate_scan_substring_capture_refs(
    bytes: &[u8],
    start: usize,
    end: usize,
    prior_inventory: &CaptureInventory,
    full_inventory: &CaptureInventory,
) -> Option<RegexCompileValidationError> {
    let raw = std::str::from_utf8(&bytes[start..end]).ok()?;
    for item in raw.split(',').map(str::trim) {
        if item.is_empty() {
            continue;
        }
        if let Some(name) = item
            .strip_prefix('<')
            .and_then(|value| value.strip_suffix('>'))
            .or_else(|| {
                item.strip_prefix('\'')
                    .and_then(|value| value.strip_suffix('\''))
            })
        {
            if !full_inventory.names.contains(name) {
                return Some(RegexCompileValidationError::new(
                    start,
                    "scan_substring capture list references an unknown named capture",
                ));
            }
            continue;
        }

        let numeric_reference = item
            .strip_prefix('+')
            .or_else(|| item.strip_prefix('-'))
            .unwrap_or(item);
        let Ok(reference) = numeric_reference.parse::<usize>() else {
            continue;
        };

        let resolved_reference = if item.starts_with('+') {
            prior_inventory.count.saturating_add(reference)
        } else if item.starts_with('-') {
            if reference > prior_inventory.count {
                0
            } else {
                prior_inventory.count + 1 - reference
            }
        } else {
            reference
        };

        if resolved_reference == 0 || resolved_reference > full_inventory.count {
            return Some(RegexCompileValidationError::new(
                start,
                "scan_substring capture list references an unavailable capture",
            ));
        }
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

fn find_unbounded_quantified_lookbehind(input: &str) -> Option<RegexCompileValidationError> {
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
            if contains_unbounded_quantifier(bytes, body_start, body_end) {
                return Some(RegexCompileValidationError::new(
                    index,
                    "unbounded variable-length lookbehind is not accepted by the regex compile contract",
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

fn find_invalid_keep_out_escape_in_lookaround(input: &str) -> Option<RegexCompileValidationError> {
    let bytes = input.as_bytes();
    let mut index = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index = skip_regex_escape(bytes, index),
            b'[' if !is_extended_class_start(bytes, index) => {
                index = skip_char_class_for_group(bytes, index)
                    .map(|end| end + 1)
                    .unwrap_or(index + 1);
            }
            b'(' => {
                let Some(body_start) = lookaround_body_start_at(bytes, index) else {
                    index += 1;
                    continue;
                };
                let Some(body_end) = find_matching_group_end(bytes, index) else {
                    index += 1;
                    continue;
                };
                if let Some(offset) = find_keep_out_escape(bytes, body_start, body_end) {
                    return Some(RegexCompileValidationError::new(
                        offset,
                        "\\K is not accepted inside a lookaround by the regex compile contract",
                    ));
                }
                index = body_end + 1;
            }
            _ => index += 1,
        }
    }

    None
}

fn lookaround_body_start_at(bytes: &[u8], index: usize) -> Option<usize> {
    if bytes.get(index) != Some(&b'(') {
        return None;
    }

    if bytes.get(index + 1) == Some(&b'?') {
        match bytes.get(index + 2).copied() {
            Some(b'=') | Some(b'!') => return Some(index + 3),
            Some(b'*') => return Some(index + 3),
            Some(b'<') => match bytes.get(index + 3).copied() {
                Some(b'=') | Some(b'!') | Some(b'*') => return Some(index + 4),
                _ => {}
            },
            _ => {}
        }
    }

    if bytes.get(index + 1) == Some(&b'*') {
        return alpha_lookaround_body_start_at(bytes, index);
    }

    None
}

fn alpha_lookaround_body_start_at(bytes: &[u8], index: usize) -> Option<usize> {
    let name_start = index + 2;
    let name_end = bytes[name_start..]
        .iter()
        .position(|byte| matches!(*byte, b':' | b')'))
        .map(|offset| name_start + offset)?;
    if bytes.get(name_end) != Some(&b':') {
        return None;
    }
    let name = std::str::from_utf8(&bytes[name_start..name_end]).ok()?;
    is_alpha_lookaround_name(name).then_some(name_end + 1)
}

fn is_alpha_lookaround_name(name: &str) -> bool {
    matches!(
        name,
        "pla"
            | "positive_lookahead"
            | "nla"
            | "negative_lookahead"
            | "plb"
            | "positive_lookbehind"
            | "nlb"
            | "negative_lookbehind"
            | "napla"
            | "non_atomic_positive_lookahead"
            | "naplb"
            | "non_atomic_positive_lookbehind"
    )
}

fn find_keep_out_escape(bytes: &[u8], start: usize, end: usize) -> Option<usize> {
    let mut index = start;
    while index < end {
        match bytes[index] {
            b'\\' => {
                if bytes.get(index + 1) == Some(&b'K') {
                    return Some(index);
                }
                index = skip_regex_escape(bytes, index);
            }
            b'[' => {
                index = skip_char_class_for_group(bytes, index)
                    .map(|class_end| class_end + 1)
                    .unwrap_or(index + 1);
            }
            _ => index += 1,
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

fn contains_unbounded_quantifier(bytes: &[u8], start: usize, end: usize) -> bool {
    let mut index = start;

    while index < end {
        match bytes[index] {
            b'\\' => index = skip_regex_escape(bytes, index),
            b'[' => {
                index = skip_char_class_for_group(bytes, index)
                    .map(|class_end| class_end + 1)
                    .unwrap_or(index + 1);
            }
            b'(' if is_define_conditional_group_start(bytes, index) => {
                if let Some(group_end) = find_matching_group_end(bytes, index) {
                    index = group_end + 1;
                } else {
                    index += 1;
                }
            }
            b'(' if bytes.get(index + 1) == Some(&b'*') => {
                if let Some(group_end) = star_directive_group_end_at(bytes, index) {
                    index = group_end + 1;
                } else {
                    index += 2;
                }
            }
            b'*' | b'+' => return true,
            b'?' => index += 1,
            b'{' => {
                if let Some(close) = bytes[index + 1..end].iter().position(|byte| *byte == b'}') {
                    let close = index + 1 + close;
                    let body = std::str::from_utf8(&bytes[index + 1..close])
                        .ok()
                        .unwrap_or("");
                    if counted_quantifier_has_unbounded_max(body) {
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

fn is_define_conditional_group_start(bytes: &[u8], start: usize) -> bool {
    const DEFINE_CONDITIONAL_PREFIX: &[u8] = b"(?(DEFINE)";

    bytes
        .get(start..start + DEFINE_CONDITIONAL_PREFIX.len())
        .is_some_and(|candidate| candidate == DEFINE_CONDITIONAL_PREFIX)
}

fn star_directive_group_end_at(bytes: &[u8], start: usize) -> Option<usize> {
    if bytes.get(start) != Some(&b'(') || bytes.get(start + 1) != Some(&b'*') {
        return None;
    }

    let name_start = start + 2;
    let mut cursor = name_start;
    while let Some(byte) = bytes.get(cursor).copied() {
        if matches!(byte, b':' | b')' | b'=') {
            break;
        }
        cursor += 1;
    }

    let delimiter = bytes.get(cursor).copied()?;
    let name = std::str::from_utf8(&bytes[name_start..cursor]).ok()?;

    if is_non_verb_star_group_name(name) {
        return None;
    }

    if name.is_empty() {
        return (delimiter == b':').then(|| find_star_verb_end(bytes, start))?;
    }

    if is_pcre2_start_option_name(name) || pcre2_verb_argument_rule(name).is_some() {
        return find_star_verb_end(bytes, start);
    }

    None
}

fn counted_quantifier_has_unbounded_max(body: &str) -> bool {
    let parts: Vec<&str> = body.split(',').map(str::trim).collect();
    match parts.as_slice() {
        [_, ""] => true,
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
    fn rejects_pcre_unsupported_perl_escapes() {
        for input in [r"\F", r"\l", r"\L", r"\u", r"\U"] {
            let error = validate_regex_compile_contract(input)
                .expect_err("must reject unsupported Perl escape");
            assert!(error.message.contains("unsupported regex escape"));
        }
    }

    #[test]
    fn allows_short_unicode_property_escapes() {
        for input in [r"\pL", r"\PN", r"[\pL\PN]", r"\pl", r"\Pn"] {
            validate_regex_compile_contract(input)
                .expect("PCRE2 accepts short-form one-letter Unicode property escapes");
        }
    }

    #[test]
    fn rejects_invalid_short_unicode_property_escapes() {
        for input in [r"\pA", r"\P_", r"\p"] {
            let error = validate_regex_compile_contract(input)
                .expect_err("must reject malformed short Unicode property escape");
            assert!(
                error.message.contains("Unicode property")
                    || error.message.contains("Unicode general category")
            );
        }
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
    fn rejects_keep_out_escape_in_character_class() {
        let error = validate_regex_compile_contract(r"[\K]").expect_err("must reject \\K");
        assert!(error.message.contains("class"));
    }

    #[test]
    fn rejects_not_newline_escape_in_character_class() {
        let error =
            validate_regex_compile_contract(r"a[\NB]c").expect_err("must reject \\N in class");
        assert!(error.message.contains("\\N"));
    }

    #[test]
    fn rejects_nonliteral_class_range_endpoints() {
        for input in [
            r"[\d-x]",
            r"[\D-x]",
            r"[\h-x]",
            r"[\H-x]",
            r"[\s-x]",
            r"[\S-x]",
            r"[\w-x]",
            r"[\W-x]",
            r"[\pL-x]",
            r"[\PN-x]",
            r"[a-\d]",
            r"[a-\p{Lu}]",
        ] {
            let error = validate_regex_compile_contract(input)
                .expect_err("must reject nonliteral range endpoint");
            assert!(error.message.contains("range"));
        }
    }

    #[test]
    fn allows_single_literal_quoted_class_range_endpoints() {
        validate_regex_compile_contract(r"^[\Qa\E-\Qz\E]+")
            .expect("single literal quoted endpoints form a PCRE2 class range");
    }

    #[test]
    fn allows_braced_hex_literal_class_range_endpoints() {
        validate_regex_compile_contract(r"[\x{7f}-\x{ff}]")
            .expect("braced hex escapes are literal PCRE2 class range endpoints");
    }

    #[test]
    fn allows_alt_extended_class_dash_operators_after_shorthand_escape() {
        for input in [r"[\d-[z]]", r"[\d-||z]"] {
            validate_regex_compile_contract(input).unwrap_or_else(|err| {
                panic!("{input:?} should not be treated as a range: {err:?}")
            });
        }
    }

    #[test]
    fn allows_literal_backslash_inside_quoted_literal() {
        validate_regex_compile_contract(r"\Qabc\$xyz\E")
            .expect("backslash inside quoted literal is literal until \\E");
    }

    #[test]
    fn allows_quote_escapes_in_character_class() {
        for input in [r"[a\Q\E]AAA", r"[z\Qa-d]\E]", r"[ab\Q^$.|?*+(){}\E]+"] {
            validate_regex_compile_contract(input)
                .expect("PCRE2 accepts quoted literal regions inside character classes");
        }
    }

    #[test]
    fn rejects_empty_quote_regions_that_leave_no_class_atom() {
        for input in [
            r"[\Q\E]AAA",
            r"[^\Q\E]AAA",
            r"[\Q\E^]AAA",
            r"[[:digit:]\Q\E-H]+",
        ] {
            let error = validate_regex_compile_contract(input)
                .expect_err("empty class quote regions do not contribute a range/class atom");
            assert!(error.message.contains("class") || error.message.contains("range"));
        }
    }

    #[test]
    fn allows_orphan_quote_end_in_non_empty_character_class() {
        for input in [
            r"^[\Eabc]",
            r"^[a-\Ec]",
            r"^[a\E\E-\Ec]",
            r"^[\E\Qa\E-\Qz\E]+",
        ] {
            validate_regex_compile_contract(input)
                .unwrap_or_else(|err| panic!("{input:?} should be accepted: {err:?}"));
        }
    }

    #[test]
    fn rejects_orphan_quote_end_only_character_class() {
        let error = validate_regex_compile_contract(r"[\E]AAA")
            .expect_err("orphan quote-end escape does not create a class atom by itself");
        assert!(error.message.contains("class"));
    }

    #[test]
    fn allows_quote_escape_pair_outside_character_class() {
        validate_regex_compile_contract(r"abc\Q(*+|\Eabc")
            .expect("quoted literal escapes remain valid outside character classes");
    }

    #[test]
    fn rejects_numeric_callout_above_pcre2_limit() {
        let error = validate_regex_compile_contract("(?C256)ab")
            .expect_err("must reject callout number above 255");
        assert!(error.message.contains("255"));
    }

    #[test]
    fn allows_numeric_callout_at_pcre2_limit() {
        validate_regex_compile_contract("(?C255)ab")
            .expect("numeric callout 255 is accepted by PCRE2");
    }

    #[test]
    fn rejects_keep_out_escape_in_lookaround() {
        for input in [r"(?=a\Kb)ab", r"(?<=\K.)x", r"(*pla:a\Kb)ab"] {
            let error =
                validate_regex_compile_contract(input).expect_err("must reject \\K in lookaround");
            assert!(error.message.contains("\\K"));
        }
    }

    #[test]
    fn allows_keep_out_escape_outside_lookaround() {
        validate_regex_compile_contract(r"\Kword")
            .expect("\\K remains accepted outside lookaround contexts");
    }

    #[test]
    fn rejects_unknown_posix_character_class_name() {
        let error = validate_regex_compile_contract("[[:foo:]]")
            .expect_err("must reject unknown POSIX class");
        assert!(error.message.contains("POSIX"));
    }

    #[test]
    fn allows_known_posix_character_class_name() {
        validate_regex_compile_contract("[[:alpha:]]")
            .expect("known POSIX class should be accepted");
    }

    #[test]
    fn allows_pcre2_posix_word_boundary_aliases() {
        for input in ["[[:<:]]red[[:>:]]", "[[:<:]]+red", "red[[:>:]]+"] {
            validate_regex_compile_contract(input)
                .unwrap_or_else(|err| panic!("{input:?} should be accepted: {err:?}"));
        }
    }

    #[test]
    fn rejects_mixed_pcre2_posix_word_boundary_alias() {
        let error = validate_regex_compile_contract("[a[:<:]] should give error")
            .expect_err("PCRE2 only accepts exact [[:<:]]/[[:>:]] aliases");
        assert!(error.message.contains("POSIX"));
    }

    #[test]
    fn rejects_invalid_pcre2_verb_shapes() {
        for input in ["a(*MARK)b", "abc(*MARK:)pqr", "abc(*:)pqr", "(*ploo:abc)"] {
            let error =
                validate_regex_compile_contract(input).expect_err("must reject invalid verb shape");
            assert!(error.message.contains("verb") || error.message.contains("MARK"));
        }
    }

    #[test]
    fn rejects_quantified_non_accept_verb() {
        let error =
            validate_regex_compile_contract("a(*FAIL)+b").expect_err("must reject quantified FAIL");
        assert!(error.message.contains("ACCEPT"));
    }

    #[test]
    fn allows_valid_pcre2_verb_shapes() {
        for input in ["(*MARK:pear)apple", "(*:pear)apple", "(*PRUNE)apple"] {
            validate_regex_compile_contract(input)
                .unwrap_or_else(|err| panic!("{input:?} should be accepted: {err:?}"));
        }
    }

    #[test]
    fn rejects_empty_pcre2_start_option_value() {
        let error = validate_regex_compile_contract("(*LIMIT_MATCH=)abc")
            .expect_err("must reject empty LIMIT_MATCH");
        assert!(error.message.contains("numeric"));
    }

    #[test]
    fn rejects_mid_pattern_pcre2_start_option() {
        let error =
            validate_regex_compile_contract("a(*CR)b").expect_err("must reject mid-pattern CR");
        assert!(error.message.contains("start-option"));
    }

    #[test]
    fn allows_valid_pcre2_start_options() {
        for input in [
            "(*CRLF)(*LIMIT_MATCH=123)abc",
            "(*UTF8)\\x{1234}",
            "(*UTF16)\\x{1234}",
            "(*UTF32)\\x{1234}",
        ] {
            validate_regex_compile_contract(input)
                .unwrap_or_else(|err| panic!("{input:?} should be accepted: {err:?}"));
        }
    }

    #[test]
    fn allows_define_conditionals_in_lookbehind_length_scan() {
        for input in ["(?<=X(?(DEFINE)(.*))Y).", "(?<!X(?(DEFINE)(.*))Y)."] {
            validate_regex_compile_contract(input)
                .unwrap_or_else(|err| panic!("{input:?} should be accepted: {err:?}"));
        }
    }

    #[test]
    fn rejects_scan_substring_unknown_capture_refs() {
        for input in [
            "(*scs:(1)a|b)",
            "(*scs:(0)a)",
            "(*scs:(<name>)a|b)",
            "()(*scs:(1,2))",
            "()()(*scs:(1,2,'XYZ'))",
        ] {
            let error = validate_regex_compile_contract(input)
                .expect_err("must reject unavailable scan_substring capture");
            assert!(error.message.contains("scan_substring"));
        }
    }

    #[test]
    fn allows_scan_substring_known_capture_refs() {
        validate_regex_compile_contract("(?<name>a)(*scs:(1,<name>)b)")
            .expect("scan_substring may reference already declared captures");
    }

    #[test]
    fn allows_scan_substring_forward_capture_refs() {
        for input in [
            "(*scs:(1)a)(a)|x",
            "(*scs:(1)a)?(a)",
            "(*scs:(1)a)??(a)",
            "(*scs:(<GOOD_NAME>)a)(?<GOOD_NAME>a)",
            "f(?:(*scs:(+1,+2)(?<=(.)))|()){16}",
        ] {
            validate_regex_compile_contract(input)
                .unwrap_or_else(|err| panic!("{input:?} should be accepted: {err:?}"));
        }
    }

    #[test]
    fn allows_unicode_capture_names_and_named_backreferences() {
        validate_regex_compile_contract("(?'ABáC'...)\u{5c}g{ABáC}")
            .expect("PCRE2 UTF-mode names may contain Unicode letters");
        validate_regex_compile_contract("(?<ABáC>...)\u{5c}k<ABáC>")
            .expect("named backreference accepts the same Unicode name shape");
    }

    #[test]
    fn rejects_malformed_named_backreference_escapes() {
        for input in [r"\k", r"\kabc", r"\k''", r"\k<>", r"\k{}"] {
            let error = validate_regex_compile_contract(input)
                .expect_err("malformed named backreference must be rejected");
            assert!(error.message.contains("named backreference"));
        }
    }

    #[test]
    fn rejects_capture_names_beyond_pcre2_limit() {
        let too_long_name = "abcdefghijklmnopqrstuvwxyzABCDEFGabcdefghijklmnopqrstuvwxyzABCDEabcdefghijklmnopqrstuvwxyzABCDEabcdefghijklmnopqrstuvwxyzABCDEFGH";
        let input = format!("(?'{too_long_name}'toolong)");
        let error = validate_regex_compile_contract(&input)
            .expect_err("PCRE2 limits capture names to MAX_NAME_SIZE bytes");
        assert!(error.message.contains("capture group name"));
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
    fn allows_pcre2_variable_length_lookbehind_at_contract_layer() {
        for input in [
            "(?<=a{1,3})b",
            "(?<=ab?c)d",
            "(?<=a(*ACCEPT)b)c",
            "(?<=a(*COMMIT)b)c",
            "(?<=a(*FAIL)b)c",
            "(?<=a(*PRUNE)b)c",
            "(?<=a(*SKIP)b)c",
            "(?<=a(*THEN)b)c",
            "(?<=a(*:MARK)b)c",
        ] {
            validate_regex_compile_contract(input)
                .unwrap_or_else(|err| panic!("{input:?} should be accepted: {err:?}"));
        }
    }

    #[test]
    fn rejects_unbounded_variable_length_lookbehind() {
        for input in [
            "(?<=a+)b",
            "(?<=a*)b",
            "(?<=a{2,})b",
            "(?<=ab(c+)d)ef",
            "(?<=ab(?<=c+)d)ef",
        ] {
            let error = validate_regex_compile_contract(input)
                .expect_err("unbounded lookbehind length must remain rejected");
            assert!(error.message.contains("lookbehind"));
        }
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

    #[test]
    fn allows_malformed_posix_opener_as_class_literals() {
        validate_regex_compile_contract("([[:]+)")
            .expect("malformed POSIX opener should fall back to class literals");
    }
}
