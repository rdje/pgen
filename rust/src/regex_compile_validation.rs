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
    // REGEX-PCRE2-FIDELITY.3.1 (PGEN-REGEX-PCRE2-0006): the `\i \F \l \L \u \U` unsupported-escape
    // check (`find_invalid_escape_i`) has been MIGRATED INTO `grammars/regex.ebnf` — the strict
    // variants of `simple_escape`/`class_simple_escape`/`class_range_literal_escape_letter` reject
    // these six in the default (pcre2) profile; the `relaxed` profile re-admits them. The EBNF is now
    // the single source of truth for this rule ([[project_ebnf_is_single_source_of_truth]]), so the
    // out-of-band validator no longer owns it.
    //
    // REGEX-PCRE2-FIDELITY.4.1: the bare Unicode-property-escape check
    // (`find_invalid_property_escape`) has likewise been MIGRATED INTO
    // `grammars/regex.ebnf` — `\p` / `\P` are now whole-letter lookaheads on
    // `simple_escape` / `class_simple_escape_{strict,relaxed}` (and dropped from
    // the `simple_escape_letter_strict` positive set), so a bare property escape
    // that is not a valid one-letter category or `{name}` form (`\pA`, `\P_`,
    // `\p`@EOF, `[\pA]`) hard-REJECTs at the grammar layer. The EBNF is the single
    // source of truth; the out-of-band validator no longer owns it.
    // REGEX-PCRE2-FIDELITY.4.2: the `\k` shape + `\k`/capture-group NAME charset + NAME length ≤ 128
    // check (`find_invalid_named_escape_or_group_name`) has been MIGRATED INTO `grammars/regex.ebnf`
    // and `find_invalid_named_escape_or_group_name` (+ its 5 exclusive helpers + `PCRE2_MAX_NAME_SIZE`)
    // DELETED. The grammar now owns all three: (1) charset — the `name` rule (already grammar-owned);
    // (2) length ≤ 128 code-points — `name`'s `{0,127}`-bounded continue run (RGX is Unicode-only, so
    // the faithful unit is CODE POINTS, matching PCRE2's 32-bit code-unit err 148); (3) `\k` shape —
    // `\k` is always a named-backreference introducer (`!"k"` on `simple_escape` + the three
    // `\k<…>`/`\k'…'`/`\k{…}` `backreference` branches), so a bare/empty `\k` hard-REJECTs. The EBNF is
    // the single source of truth ([[project_ebnf_is_single_source_of_truth]]); the out-of-band validator
    // no longer owns it.
    // REGEX-PCRE2-FIDELITY.4.3: the counted-quantifier min>max ORDER rule (err 104, `x{5,4}`
    // reject) has been MIGRATED INTO `grammars/regex.ebnf` and `find_invalid_counted_quantifier`
    // + `validate_counted_quantifier_body` DELETED. The `{n,m}` range form is now a dedicated
    // `counted_quantifier_range` rule gated by the general RULE-SPAN `value_compare` `@predicate`
    // (`[$1, le, $5]` phase:post, decimal-integer numeric so `{05,4}` = 5>4 rejects), so a
    // min>max range loses its (backtrackable) branch, no other body branch fully-consumes the
    // brace, and the `literal_open_brace` guard blocks the literal fallback ⇒ whole-pattern
    // REJECT. (The earlier `.3.18` VALUE bound (err 105, ≤ 65535) and brace TOKENIZATION model
    // were already grammar-owned via `quant_bound_number` + the `literal_open_brace` guard.) The
    // EBNF is now the single source of truth ([[project_ebnf_is_single_source_of_truth]]).
    // REGEX-PCRE2-FIDELITY.3.16: the numeric-callout range check (`find_invalid_numeric_callout`)
    // was MIGRATED into `grammars/regex.ebnf` — `callout_number` structurally admits only digit
    // runs whose VALUE is ≤ 255 with arbitrary leading zeros (PCRE2 err 138), so `(?C256)`-family
    // forms now reject at the grammar layer and generation is in-range by construction.
    if let Some(error) = find_invalid_verb_construct(input) {
        return Err(error);
    }
    if let Some(error) = find_invalid_char_class_construct(input) {
        return Err(error);
    }
    // REGEX-PCRE2-FIDELITY.3.13: the quantified-anchor check (`find_invalid_quantified_anchor`)
    // was MIGRATED into `grammars/regex.ebnf` — anchors are their own non-quantifiable `piece`
    // branch (`anchor !quantifier`), so `^*`/`$*` AND the previously-missed escape-anchor forms
    // (`\A*` `\b*` `\B?` `\G+` `\z*` `\Z*` `\K*`, err 109) now reject at the grammar layer.
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

    if next == b'x' && bytes.get(start + 2).is_some_and(u8::is_ascii_hexdigit) {
        let mut end = start + 3;
        if bytes.get(end).is_some_and(u8::is_ascii_hexdigit) {
            end += 1;
        }
        return end;
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

    if matches!(next, b'0'..=b'7') {
        let mut end = start + 2;
        while end < bytes.len() && end < start + 4 && matches!(bytes[end], b'0'..=b'7') {
            end += 1;
        }
        return end;
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

// REGEX-PCRE2-FIDELITY.4.1: `find_invalid_property_escape` was removed — the bare
// `\p` / `\P` property-escape acceptance rule is now grammar-owned (see the migration
// note on `validate_regex_compile_contract`). `is_short_unicode_property_letter` is
// retained: `skip_regex_escape` still uses it to advance past a valid `\pL` when the
// OTHER checks below scan the pattern.
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

fn find_invalid_verb_construct(input: &str) -> Option<RegexCompileValidationError> {
    // REGEX-PCRE2-FIDELITY.3.20: the quantified-verb rule (only `(*ACCEPT)` may be quantified) was
    // MIGRATED into `grammars/regex.ebnf` — the non-ACCEPT directives are a non-quantifiable `piece`
    // branch (`directive_verb_nonquant !quantifier`), so `(*PRUNE)+` `(*:x)+` `(*MARK:x)+` `(*UTF)+`
    // `(*LIMIT_HEAP=5)+` all reject at the grammar layer (err-109-faithful). The ONLY rule this
    // function still owns is the start-option POSITION rule (contextual — "every group before this
    // one is a start option" — which the grammar cannot express without a whole-pattern restructure;
    // validator-owned until capstone `.4`). Verb/start-option NAME acceptance and per-name ARGUMENT
    // shapes are already grammar-owned (`.3.2`/`.3.14`); this walk therefore only re-checks position
    // for a recognized start option and advances past every other `(*...)` construct.
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

                if bytes.get(cursor).is_none() {
                    // Unterminated `(*` construct — nothing further to check here.
                    index += 1;
                    continue;
                }
                let name = std::str::from_utf8(&bytes[name_start..cursor]).ok()?;

                if is_non_verb_star_group_name(name) {
                    index += 1;
                    continue;
                }

                if is_pcre2_start_option_name(name) && !is_start_option_position(bytes, index) {
                    // The start-option POSITION rule (contextual; covers bare and `=`-value forms).
                    return Some(RegexCompileValidationError::new(
                        index,
                        "PCRE2 start option must appear at the start-option prefix",
                    ));
                }

                // Every other `(*...)` construct — verbs, MARK, the `(*:x)` shorthand, LIMIT/option
                // argument shapes, quantifiability, and unrecognized names — is grammar-owned now;
                // just advance past it.
                if let Some(group_end) = find_star_verb_end(bytes, index) {
                    index = group_end + 1;
                    continue;
                }
                index += 1;
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

fn is_pcre2_verb_name(name: &str) -> bool {
    // REGEX-PCRE2-FIDELITY.3.14/.3.20: the per-verb argument-shape distinction AND the
    // quantified-verb rule are both grammar-owned now; the validator keeps verb-name recognition
    // solely so `star_directive_group_end_at` can skip a `(*VERB)` group in the lookbehind walk.
    matches!(
        name,
        "MARK" | "ACCEPT" | "F" | "FAIL" | "COMMIT" | "PRUNE" | "SKIP" | "THEN"
    )
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
    Literal(u32),
    NonLiteral,
    ZeroWidth,
}

impl ClassAtomKind {
    fn is_substantive(self) -> bool {
        !matches!(self, Self::ZeroWidth)
    }
}

/// `REGEX-PCRE2-FIDELITY.3.15`: skip the PCRE2-INVISIBLE class items — stray `\E` and the
/// empty `\Q\E` — which PCRE2 drops before reading the class opening (negation caret /
/// initial-`]`-literal detection). Mirrors the grammar's `class_zero_width`.
fn skip_invisible_class_items(bytes: &[u8], mut index: usize) -> usize {
    loop {
        if bytes.get(index) == Some(&b'\\') && bytes.get(index + 1) == Some(&b'E') {
            index += 2;
            continue;
        }
        if bytes.get(index) == Some(&b'\\')
            && bytes.get(index + 1) == Some(&b'Q')
            && bytes.get(index + 2) == Some(&b'\\')
            && bytes.get(index + 3) == Some(&b'E')
        {
            index += 4;
            continue;
        }
        return index;
    }
}

fn scan_char_class(bytes: &[u8], start: usize) -> Result<usize, RegexCompileValidationError> {
    // The PCRE2 class-open model (`REGEX-PCRE2-FIDELITY.3.15`, grammar-aligned): invisible
    // items are skipped; the first non-invisible char may be the negation caret (invisibles
    // may follow it too); a `]` seen before any VISIBLE member is a LITERAL member (it can be
    // a range start — `[\E]-z]`). Class NON-EMPTINESS itself is grammar-owned now
    // (`class_body_nonempty*` / `class_negated_open`), so `]` in the member loop always closes.
    let mut index = start + 1;
    let mut previous_atom: Option<ClassAtomKind> = None;

    index = skip_invisible_class_items(bytes, index);
    if index < bytes.len() && bytes[index] == b'^' {
        index += 1;
        index = skip_invisible_class_items(bytes, index);
    }
    if index < bytes.len() && bytes[index] == b']' {
        previous_atom = Some(ClassAtomKind::Literal(b']' as u32));
        index += 1;
    }

    while index < bytes.len() {
        if bytes[index] == b']' {
            return Ok(index);
        }

        if bytes[index] == b'-'
            && bytes.get(index + 1) != Some(&b']')
            && !dash_is_trailing_literal(bytes, index)
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
            continue;
        }

        if bytes[index] == b'['
            && bytes.get(index + 1) == Some(&b':')
            && let Some(after_posix_class) = scan_posix_class(bytes, index)
        {
            index = after_posix_class;
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
            continue;
        }

        index = after_left;
        previous_atom = Some(left_atom);
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
        // REGEX-PCRE2-FIDELITY.4.4: the escape-in-class rejects (`\A \B \C \G \K \N`-unbraced
        // `\R \X \Z \z`) were MIGRATED into `grammars/regex.ebnf` — `class_escape_unit` drops
        // `single_byte_escape` (`\C`), `class_simple_escape_{strict,relaxed}` carry the guards
        // `!"A" !"B" !"C" !"G" !"K" !"R" !"X" !"Z" !"z" !( "N" !"{" )`, and the range path drops
        // `A`/`G`/`z` from `class_range_literal_escape_letter_strict`. Any class containing one of
        // these escapes is now grammar-rejected before this validator ever runs, so the checks
        // that lived here are dead post-migration and were removed. The EBNF is the single source
        // of truth ([[project_ebnf_is_single_source_of_truth]]).
        let after_escape = skip_regex_escape(bytes, index);
        if is_nonliteral_class_escape(bytes, index, after_escape) {
            return Ok((ClassAtomKind::NonLiteral, after_escape));
        }
        let literal = class_escape_literal_codepoint(bytes, index, after_escape)?;
        return Ok((ClassAtomKind::Literal(literal), after_escape));
    }
    Ok((ClassAtomKind::Literal(bytes[index] as u32), index + 1))
}

fn class_escape_literal_codepoint(
    bytes: &[u8],
    index: usize,
    after_escape: usize,
) -> Result<u32, RegexCompileValidationError> {
    let next = bytes.get(index + 1).copied().unwrap_or_default();
    if next == b'x' {
        if bytes.get(index + 2) == Some(&b'{') && after_escape > index + 4 {
            let digits = std::str::from_utf8(&bytes[index + 3..after_escape - 1]).unwrap_or("");
            let digits = digits.trim();
            if !digits.is_empty() && digits.as_bytes().iter().all(u8::is_ascii_hexdigit) {
                return u32::from_str_radix(digits, 16).map_err(|_| {
                    RegexCompileValidationError::new(
                        index,
                        "invalid braced hex escape in character class range endpoint",
                    )
                });
            }
            return Ok(next as u32);
        }
        let digit_end = after_escape.min(index + 4);
        let digits = std::str::from_utf8(&bytes[index + 2..digit_end]).unwrap_or("");
        if !digits.is_empty() && digits.as_bytes().iter().all(u8::is_ascii_hexdigit) {
            return u32::from_str_radix(digits, 16).map_err(|_| {
                RegexCompileValidationError::new(
                    index,
                    "invalid hex escape in character class range endpoint",
                )
            });
        }
    }
    if next == b'o' && bytes.get(index + 2) == Some(&b'{') && after_escape > index + 4 {
        let digits = std::str::from_utf8(&bytes[index + 3..after_escape - 1]).unwrap_or("");
        let digits = digits.trim();
        if !digits.is_empty()
            && digits
                .as_bytes()
                .iter()
                .all(|byte| matches!(byte, b'0'..=b'7'))
        {
            return u32::from_str_radix(digits, 8).map_err(|_| {
                RegexCompileValidationError::new(
                    index,
                    "invalid braced octal escape in character class range endpoint",
                )
            });
        }
        return Ok(next as u32);
    }
    if next == b'c' && after_escape == index + 3 {
        let control = bytes.get(index + 2).copied().unwrap_or_default();
        return Ok((control as u32) & 0x1f);
    }
    if matches!(next, b'0'..=b'7') && after_escape > index + 1 {
        let digits = std::str::from_utf8(&bytes[index + 1..after_escape]).unwrap_or("");
        if !digits.is_empty()
            && digits
                .as_bytes()
                .iter()
                .all(|byte| matches!(byte, b'0'..=b'7'))
        {
            return u32::from_str_radix(digits, 8).map_err(|_| {
                RegexCompileValidationError::new(
                    index,
                    "invalid octal escape in character class range endpoint",
                )
            });
        }
    }
    let literal = match next {
        b'a' => 0x07,
        b'b' => 0x08,
        b'e' => 0x1b,
        b'f' => 0x0c,
        b'n' => 0x0a,
        b'r' => 0x0d,
        b't' => 0x09,
        _ => next as u32,
    };
    Ok(literal)
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
        // REGEX-PCRE2-FIDELITY.4.5.a: a `[`-introduced POSIX bracket token used as a range's RIGHT
        // endpoint — `[:name:]` (posix class), `[.coll.]` (collating element), or `[=equiv=]`
        // (equivalence class) — is a NON-LITERAL endpoint. PCRE2 10.47 rejects `atom-[:..:]` /
        // `atom-[...]` / `atom-[=..=]` with err 150 "invalid range in character class" REGARDLESS of
        // order (so `[!-[:alpha:]]`, `!` < `[`, still rejects). A bare `[` NOT opening such a token is
        // the literal code point `0x5B` (`read_class_atom` below), which orders normally.
        if let Some(after_token) = scan_class_bracket_token(bytes, index) {
            return Ok((Some(ClassAtomKind::NonLiteral), after_token));
        }
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
        first_literal.get_or_insert(bytes[index] as u32);
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

/// `REGEX-PCRE2-FIDELITY.4.5.a`: recognize a `[`-introduced POSIX bracket token — `[:name:]` (posix
/// class), `[.coll.]` (collating element), or `[=equiv=]` (equivalence class) — as a class-range
/// endpoint. Returns the index past the closing `X]` (`:]`/`.]`/`=]`) if `index` starts such a token,
/// else `None` (a bare `[` is the literal code point `0x5B`). All three are NON-LITERAL range endpoints
/// (PCRE2 err 150). Scanning to the first matching `X]` mirrors `scan_posix_class`'s recognition span.
///
/// Formerly this site held `dash_starts_alt_extended_class_operator`, which suppressed range detection
/// whenever the char after `-` was `[` or `||`. That guard belongs to PCRE2's ALTERNATE extended-class
/// syntax `(?[...])`, but `scan_char_class` is dispatched ONLY for NORMAL classes
/// (`find_invalid_char_class_construct` gates on `!is_extended_class_start`), where `-[` / `-||` is
/// ALWAYS a range — so the guard was unconditionally mis-applied and accepted invalid ranges
/// (`[a-[b]]` descending, `[x-[:alpha:]]` nonliteral, `[~-||]` descending). The guard is removed from
/// both range branches; a `[`-token right endpoint now classifies NON-LITERAL and a bare `[` a literal.
fn scan_class_bracket_token(bytes: &[u8], index: usize) -> Option<usize> {
    let kind = match (bytes.get(index), bytes.get(index + 1)) {
        (Some(b'['), Some(&k @ (b':' | b'.' | b'='))) => k,
        _ => return None,
    };
    let mut cursor = index + 2;
    while cursor + 1 < bytes.len() {
        if bytes[cursor] == kind && bytes[cursor + 1] == b']' {
            return Some(cursor + 2);
        }
        cursor += 1;
    }
    None
}

/// `REGEX-PCRE2-FIDELITY.4.6` (`PGEN-REGEX-PCRE2-0024`): POSIX class NAME validity is now
/// GRAMMAR-owned (`grammars/regex.ebnf` — the `class_member_literal` / `class_member_literal_nocaret`
/// guard rejects a `[:...:]` token whose name is not one of the 14 valid POSIX names). This helper
/// is now a pure structural SPAN scanner: it recognizes a `[:...:]` posix token (any name) so
/// `scan_char_class` can mark it a NON-LITERAL range endpoint — the `[[:alpha:]-z]` / `[a-[:alpha:]]`
/// invalid-range check, owned by `.4.5`. It never rejects a name; an unknown name is caught earlier
/// at the grammar layer, so by the time this pass runs every accepted `[:...:]` span has a valid name.
fn scan_posix_class(bytes: &[u8], start: usize) -> Option<usize> {
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

// REGEX-PCRE2-FIDELITY.3.20: the `quantifier_starts_at` helper was removed with the quantified-verb
// checks it fed (only `(*ACCEPT)` may be quantified is grammar-owned now — the non-quantifiable
// `directive_verb_nonquant` piece branch). `is_pcre2_verb_name` / `find_star_verb_end` stay — still
// used by `star_directive_group_end_at` for the lookbehind walk.

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

    if is_pcre2_start_option_name(name) || is_pcre2_verb_name(name) {
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

    // REGEX-PCRE2-FIDELITY.3.1 (PGEN-REGEX-PCRE2-0006): the `\i \F \l \L \u \U` unsupported-escape
    // rejection has MOVED OUT of this validator INTO `grammars/regex.ebnf` (the strict variants of
    // `simple_escape`/`class_simple_escape`/`class_range_literal_escape_letter` reject them in the
    // default `pcre2` profile; `relaxed` re-admits them). The former `rejects_invalid_escape_i` /
    // `rejects_pcre_unsupported_perl_escapes` validator unit tests were removed accordingly. The
    // behaviour is now proven by the GRAMMAR parse path and the `pcre2test` oracle
    // (`regex_pcre2_compile_oracle_gate`) — the EBNF is the single source of truth
    // ([[project_ebnf_is_single_source_of_truth]]).

    // REGEX-PCRE2-FIDELITY.4.1: the bare Unicode-property-escape acceptance rule has
    // MOVED OUT of this validator INTO `grammars/regex.ebnf` — `\p` / `\P` are whole-letter
    // lookaheads on `simple_escape` / `class_simple_escape_{strict,relaxed}` (and dropped
    // from the `simple_escape_letter_strict` positive set), so a valid one-letter category
    // or `{name}` form parses while `\pA` / `\P_` / `\p`@EOF / `[\pA]` hard-REJECT at the
    // grammar layer. The former `allows_short_unicode_property_escapes` /
    // `rejects_invalid_short_unicode_property_escapes` validator unit tests were removed
    // accordingly; the behaviour is now proven by the GRAMMAR parse path and the `pcre2test`
    // oracle (`regex_pcre2_compile_oracle_gate`) — the EBNF is the single source of truth.

    // REGEX-PCRE2-FIDELITY.4.3: `rejects_invalid_counted_quantifier_order` (`x{5,4}`),
    // `rejects_tab_spaced_counted_quantifier_order` (`a{\t5\t,\t2\t}`), and
    // `allows_newline_brace_as_literal_not_order_violation` (`a{\n5,2\n}`) were removed — the
    // counted-quantifier min>max ORDER reject (err 104) is now GRAMMAR-owned (`grammars/regex.ebnf`
    // `counted_quantifier_range` gated by the RULE-SPAN `value_compare` `@predicate`), and
    // `find_invalid_counted_quantifier` + `validate_counted_quantifier_body` are deleted. The parity
    // pin `regex_counted_quantifier_order_rejects_at_the_grammar_layer_pcre2_faithfully` in
    // `parser_registry.rs` proves the whole parse path still rejects out-of-order bounds (including
    // the tab-spaced form) and accepts the literal newline-brace form.

    #[test]
    fn counted_quantifier_value_limit_is_grammar_owned() {
        // REGEX-PCRE2-FIDELITY.3.18: the > 65535 VALUE bound (PCRE2 err 105)
        // moved to the grammar (`quant_bound_number` + the
        // `literal_open_brace` guard) — the validator no longer owns it, so
        // the raw-text scan passes these; the generated parser rejects them
        // structurally (pinned by the oracle-matrix test in
        // `parser_registry`).
        for input in ["z{65536}", "a{4294967296}", "a{0,65536}"] {
            validate_regex_compile_contract(input)
                .unwrap_or_else(|_| panic!("value bound is grammar-owned for {input:?}"));
        }
    }

    // REGEX-PCRE2-FIDELITY.4.4: `rejects_invalid_class_escape` (`[\B]`),
    // `rejects_keep_out_escape_in_character_class` (`[\K]`), and
    // `rejects_not_newline_escape_in_character_class` (`a[\NB]c`) were removed — the
    // escape-in-class rejects are now GRAMMAR-owned (`grammars/regex.ebnf`), and the parity
    // pin `regex_class_escapes_reject_at_the_grammar_layer_pcre2_faithfully` in
    // `parser_registry.rs` proves the whole parse path still rejects them.

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
    fn rejects_bracket_token_class_range_endpoints() {
        // REGEX-PCRE2-FIDELITY.4.5.a: a range whose RIGHT endpoint begins with `[` (opening a posix
        // `[:..:]`, collating `[...]`, or equivalence `[=..=]` token) is a NON-LITERAL endpoint —
        // `pcre2test` 10.47 err 150 REGARDLESS of order. Formerly the `dash_starts_alt_extended_class_operator`
        // guard suppressed range detection for `-[`, so all of these were accepts-invalid.
        for input in [
            r"[x-[:alpha:]]",
            r"[a-[:digit:]]",
            r"[a-[.-.]]",
            r"[!-[:alpha:]]", // ascending-left (`!` < `[`) still rejects: the endpoint is nonliteral.
            r"[!-[.a.]]",
            r"[!-[=a=]]",
            r"[a-[=a=]]",
            r"[\d-[z]]", // NON-LITERAL left (`\d`) to a `[` right — the guard formerly hid the reject.
            r"[\d-||z]", // NON-LITERAL left (`\d`) to a `||`/`|` right — same masked reject.
            r"[\w-[a]]",
        ] {
            let error = validate_regex_compile_contract(input)
                .expect_err("must reject bracket-token range endpoint");
            assert!(error.message.contains("range"), "{input:?}: {error:?}");
        }
        // A bare `[` (not opening a posix/collating/equivalence token) is the literal `0x5B`, so the
        // range orders normally: descending → reject (err 108), ascending/equal → accept. The `-||`
        // right endpoint `|` (`0x7C`) is likewise a literal, formerly skipped by the same mis-scoped guard.
        for input in [r"[a-[b]]", r"[z-[a]]", r"[a-[]", r"[~-||]", r"[}-||]"] {
            let error = validate_regex_compile_contract(input)
                .expect_err("must reject descending bracket/pipe range endpoint");
            assert!(error.message.contains("descending"), "{input:?}: {error:?}");
        }
        // Ascending literal `-[` / `-||` ranges and the leading-dash carve-outs stay ACCEPT
        // (`pcre2test` 10.47 compiles all of these clean).
        for input in [
            r"[!-[]", r"[+-[]", r"[Z-[]", r"[[-a]", r"[--[]", r"[a-||b]", r"[|-||]",
        ] {
            validate_regex_compile_contract(input)
                .unwrap_or_else(|err| panic!("{input:?} is a valid ascending range: {err:?}"));
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
    fn allows_wide_braced_hex_literal_class_range_endpoint() {
        for input in [
            r"[z-\x{100}]",
            r"[z-\x{200}]",
            r"[Qz-\x{200}]",
            r"[\x{7a}-\x{100}]",
            r"[\x7a-\x{100}]",
        ] {
            validate_regex_compile_contract(input).unwrap_or_else(|err| {
                panic!("{input:?} should compare braced hex endpoints by codepoint: {err:?}")
            });
        }
    }

    #[test]
    fn allows_bare_octal_literal_class_range_endpoints() {
        for input in [
            r"[\000-\037]",
            r"[\000-\017]",
            r"[\010-\037]",
            r"[\000-\047]",
            r"[\000-\057]",
            r"[a-\377]",
            r"[A-\377]",
            r"[4-\377]",
            r"[\001-\x1f]",
            r"[\001-\x{1f}]",
            r"[\001-z]",
        ] {
            validate_regex_compile_contract(input).unwrap_or_else(|err| {
                panic!("{input:?} should compare bare octal endpoints by codepoint: {err:?}")
            });
        }
    }

    #[test]
    fn rejects_descending_wide_braced_hex_literal_class_range_endpoint() {
        let error = validate_regex_compile_contract(r"[\x{100}-z]")
            .expect_err("must reject descending braced-hex class range");
        assert!(error.message.contains("descending"));
    }

    #[test]
    fn rejects_descending_single_byte_hex_literal_class_range_endpoint() {
        let error = validate_regex_compile_contract(r"[A-\x40]")
            .expect_err("must reject descending single-byte hex class range");
        assert!(error.message.contains("descending"));
    }

    #[test]
    fn rejects_descending_decoded_class_range_endpoints() {
        for input in [
            r"[\037-\000]",
            r"[\x1f-\0]",
            r"[\377-a]",
            r"[\x{100}-\377]",
            r"[\cZ-\cA]",
            r"[\e-\a]",
        ] {
            let error = validate_regex_compile_contract(input)
                .expect_err("must reject descending decoded class range endpoint");
            assert!(error.message.contains("descending"));
        }
    }

    #[test]
    fn allows_malformed_braced_class_escapes_as_literal_transport() {
        validate_regex_compile_contract(r"[\j\x{z}\o\gAb\g]")
            .expect("bad_escape_is_literal oracle class keeps malformed braced escapes literal");
    }

    // REGEX-PCRE2-FIDELITY.4.5.a: `allows_alt_extended_class_dash_operators_after_shorthand_escape`
    // (`[\d-[z]]` / `[\d-||z]` "should not be treated as a range") was DELETED — it encoded the
    // accepts-invalid behavior of the removed `dash_starts_alt_extended_class_operator` guard. Both
    // patterns are a range with a NON-LITERAL left endpoint (`\d`) to a `[` / `||` right endpoint, which
    // `pcre2test` 10.47 rejects with err 150 "invalid range in character class". They are now correctly
    // rejected and pinned in `rejects_bracket_token_class_range_endpoints` and in the parser_registry
    // parity pin `regex_class_range_bracket_endpoint_rejects_pcre2_faithfully`.

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

    // REGEX-PCRE2-FIDELITY.4.6 (PGEN-REGEX-PCRE2-0024): the POSIX class NAME-validity rejects
    // (`rejects_unknown_posix_character_class_name` on `[[:foo:]]` and
    // `rejects_mixed_pcre2_posix_word_boundary_alias` on `[a[:<:]]`) moved to the GRAMMAR layer
    // (`grammars/regex.ebnf` `class_member_literal` guard); the validator no longer owns them.
    // The full accept/reject matrix is now pinned end-to-end through the generated parser by
    // `parser_registry::tests::regex_posix_class_names_reject_at_the_grammar_layer_pcre2_faithfully`.
    // The two ACCEPT tests below stay: the validator's `scan_posix_class` recognition + the
    // `[[:<:]]`/`[[:>:]]` word-boundary alias skip are still validator-owned (range analysis, `.4.5`).

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
    fn quantified_verb_check_is_grammar_owned_now() {
        // REGEX-PCRE2-FIDELITY.3.20: the contract layer no longer rejects a quantified non-ACCEPT
        // verb — the grammar does (non-ACCEPT directives are a non-quantifiable `piece` branch,
        // `directive_verb_nonquant !quantifier`). So the validator now ACCEPTS `a(*FAIL)+b` (the
        // grammar parse rejects it before this pass runs). The PCRE2-faithful parse-layer verdict
        // is pinned by `parser_registry::tests::
        // regex_quantified_verb_rejects_at_the_grammar_layer_pcre2_faithfully`.
        validate_regex_compile_contract("a(*FAIL)+b")
            .expect("quantified-verb rejection is grammar-owned now — the validator passes it");
    }

    #[test]
    fn allows_valid_pcre2_verb_shapes() {
        for input in ["(*MARK:pear)apple", "(*:pear)apple", "(*PRUNE)apple"] {
            validate_regex_compile_contract(input)
                .unwrap_or_else(|err| panic!("{input:?} should be accepted: {err:?}"));
        }
    }

    #[test]
    fn rejects_mid_pattern_pcre2_start_option() {
        // REGEX-PCRE2-FIDELITY.3.14: verb/start-option argument SHAPES (`(*MARK)`, `(*:)`,
        // `(*SKIP=)`, `(*LIMIT_MATCH=)`, bare `(*LIMIT_HEAP)`, `(*UTF=5)`, …) are grammar-owned
        // now — proven at the parse layer by the parser_registry arg-shape pin, not here. The
        // contextual POSITION rule stays validator-owned until capstone `.4`, and since `.3.14`
        // it also covers `=`-value forms (ledger REGEX-0090: `a(*LIMIT_HEAP=500)` was wrongly
        // accepted; PCRE2 10.47 rejects err 160).
        for input in ["a(*CR)b", "a(*LIMIT_HEAP=500)", "(*FAIL)(*LIMIT_HEAP=5)a"] {
            let error = validate_regex_compile_contract(input)
                .expect_err("must reject mid-pattern start option");
            assert!(error.message.contains("start-option"));
        }
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

    // REGEX-PCRE2-FIDELITY.4.2: `allows_unicode_capture_names_and_named_backreferences`,
    // `rejects_malformed_named_backreference_escapes`, and `rejects_capture_names_beyond_pcre2_limit`
    // were DELETED — `\k` shape + NAME charset + NAME length ≤ 128 are now grammar-owned (see the
    // migration note on `validate_regex_compile_contract`). The grammar-layer replacement is
    // `parser_registry::tests::regex_named_names_reject_at_the_grammar_layer_pcre2_faithfully`.
    #[test]
    fn rejects_descending_class_range() {
        let error = validate_regex_compile_contract("[z-a]").expect_err("must reject [z-a]");
        assert!(error.message.contains("descending"));
    }

    #[test]
    fn quantified_anchor_check_is_grammar_owned_now() {
        // REGEX-PCRE2-FIDELITY.3.13: the contract layer no longer rejects quantified anchors —
        // the grammar does (the anchor `piece` branch carries `!quantifier`). The parser-level
        // verdict matrix lives in `parser_registry::tests::
        // regex_quantified_anchors_reject_at_the_grammar_layer_pcre2_faithfully`.
        assert!(validate_regex_compile_contract("^*").is_ok());
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
