//! REGEX-PCRE2-FIDELITY.4.12 — grammar-only proof that a POSIX collating-element
//! (`[.coll.]`) / equivalence-class (`[=equiv=]`) bracket-token is REJECTED by the
//! GRAMMAR wherever a class `[` could otherwise read it as a literal member.
//!
//! `pcre2test` 10.47 rejects a `[.` … `.]` / `[=` … `=]` bracket-token in a character
//! class UNCONDITIONALLY: err 113 "POSIX collating elements are not supported" when it
//! is standalone / a member / a range LEFT endpoint, and err 150 "invalid range in
//! character class" when it is a range RIGHT endpoint (the `-` triggers the range error
//! first). The released parser ACCEPTED all of these — the class `[` reads as a
//! `class_safe_special` literal 0x5B and the `.` / `a` / `=` around it become separate
//! literal members, so no recognizer ever fired (neither the grammar nor the out-of-band
//! `validate_regex_compile_contract`). `.4.12` adds a single `class_bracket_token`
//! recognizer and blocks the shape at three sites:
//!   * member position — a `!class_bracket_token` lookahead on
//!     `class_member_literal` / `class_member_literal_nocaret` (mirrors the `.4.6`
//!     `[:name:]` posix idiom);
//!   * class-OPEN position — a `!class_bracket_token_tail` lookahead right after the
//!     opening `"["` in the no-caret `char_class` alternative (the class-opening bracket
//!     itself forms the `[.` / `[=` opener: `[.a.]` err 113);
//!   * range-RIGHT endpoint — a `class_atom "-" class_bracket_token` alternative on
//!     `invalid_class_range` (`[!-[.a.]]` err 150, an ASCENDING range not otherwise
//!     caught by `.4.5.c`'s descending guard).
//!
//! The PCRE2 tokenization is subtle and every string below was verified against
//! `pcre2test` 10.47 directly (NOT from task-note annotations):
//!   * the opener `[` must be immediately followed by `.` / `=` — a `^` negation or an
//!     invisible `\E` / `\Q\E` between `[` and `.` breaks it (`[^.a.]`, `[\E.a.]` ACCEPT);
//!   * the content scan is escape-aware and stops at an UNESCAPED `]` (the class close):
//!     `[.].]` ACCEPT (class `[.]` closes at the first `]`, then `.]` are body literals),
//!     but `[.\].]` REJECT (the escaped `\]` is crossed as content and the `.]`
//!     terminator is found);
//!   * a single dot / equals is a literal (`[.]` / `[=]` ACCEPT); the token needs a
//!     genuine `.]` / `=]` terminator (`[..]` empty collating REJECT).
//!
//! Behavior-CHANGING (a genuine correctness fix, a released-parser accepts-invalid gap,
//! ledger REGEX-0110) — NOT a neutral validator→grammar migration: the compile-contract
//! validator never recognised a standalone/member/range collating or equivalence token,
//! so every MUST_REJECT string flips ACCEPT→REJECT at the released `--parse` surface.
//! The certified parse-harness interpreter (`interpret_parse`, PARSE-HARNESS.5 —
//! byte-identical to the generated regex parser) runs the grammar WITHOUT the validator,
//! so it proves the grammar-layer verdict directly.

#[cfg(all(feature = "ebnf_dual_run", feature = "generated_parsers"))]
mod grammar_only {
    use pgen::parse_harness_interpreter::{interpret_parse, InterpretOptions};
    use std::path::PathBuf;

    fn regex_grammar() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../grammars/regex.ebnf")
    }

    /// Grammar-only verdict (default `pcre2` profile) via the certified interpreter.
    fn grammar_accepts(input: &str) -> bool {
        interpret_parse(&regex_grammar(), input, &InterpretOptions::default())
            .unwrap_or_else(|e| panic!("interpreter failed on {input:?}: {e}"))
            .accepted
    }

    // A collating / equivalence bracket-token in a class — the grammar MUST reject after
    // `.4.12`. err 113 (standalone / member / range-left) or err 150 (range-right). All
    // verified REJECT by `pcre2test` 10.47.
    const MUST_REJECT: &[&str] = &[
        // class-OPEN opener: the opening `[` itself + `.` / `=`
        "[.a.]",     // class-open collating
        "[=a=]",     // class-open equivalence
        "[.ch.]",    // class-open collating, multi-char content
        "[..]",      // class-open EMPTY collating ([. immediately .])
        "[==]",      // class-open EMPTY equivalence
        "[.-.]",     // class-open collating, `-` as content
        // member `[` opener
        "[[.a.]]",   // member collating (the `.4.12` scoping's standalone example)
        "[[=a=]]",   // member equivalence
        "[[..]]",    // member empty collating
        "[[==]]",    // member empty equivalence
        "[a[.a.]b]", // member collating, surrounded by literals
        "[a[=a=]b]", // member equivalence, surrounded by literals
        "[|[.a.]]",  // member collating after a `|` literal
        // range-LEFT endpoint (no `-` right after the token's `[`, so the member guard fires)
        "[[.a.]-z]", // range-left collating
        "[[=a=]-z]", // range-left equivalence
        // range-RIGHT endpoint — err 150 (the `-` triggers the range error)
        "[a-[.a.]]", // descending `a-[` — also caught by .4.5.c; regression pin
        "[z-[.a.]]", // descending `z-[` — also caught by .4.5.c; regression pin
        "[!-[.a.]]", // ASCENDING `!-[` (33 < 91) — the .4.12 range-right target
        "[!-[=a=]]", // ASCENDING `!-[` equivalence
        "[a-[=a=]]", // descending `a-[` equivalence
        // escape-aware content scan (crosses an escaped `\]`, `.` NOT escaped by a `\`)
        "[.\\].]",   // [.\].]  — escaped `\]` crossed, `.]` terminator found
        "[.a\\.]",   // [.a\.]  — `\` is content, `.]` still terminates
        "[.a\\.b.]", // [.a\.b.] — `\` content, terminator at the end
        "[=a\\=]",   // [=a\=]  — equivalence, `\` content
    ];

    // The grammar MUST keep accepting — no bracket-token forms (single `.`/`=`, no `.]`/`=]`
    // terminator, opener broken by `^`/invisible/escape, or the class closes first). All
    // verified ACCEPT by `pcre2test` 10.47.
    const MUST_ACCEPT: &[&str] = &[
        "[.]",       // single `.` member (one dot, no second terminator dot)
        "[=]",       // single `=` member
        "[a.b]",     // dots as ordinary members
        "[a=b]",     // equals as ordinary members
        "[a.b.]",    // dots as members, no `[.` opener
        "[[]",       // literal `[` member
        "[]]",       // initial-close class then close
        "[[.]",      // member `[.` with NO `.]` terminator
        "[[=]",      // member `[=` with NO `=]` terminator
        "[.ab]",     // class-open `[.` with NO `.]` terminator
        "[ab.]",     // trailing `.]`-looking bytes but no `[.` opener
        "[x.a.]",    // class-open `[x` (not `[.`) — the later `.a.]` is NOT a token
        "[.].]",     // class `[.]` closes at the first `]`, then `.]` are body literals
        "[.a].]",    // class `[.a]` closes at the first `]`, then `.]`
        "[^.a.]",    // `^` negation breaks the class-open opener
        "[^.-.]",    // `^` negation breaks it
        "[\\E.a.]",  // stray `\E` (invisible) breaks the class-open opener
        "[\\Q\\E.a.]", // empty `\Q\E` (invisible) breaks the class-open opener
        "[].a.]",    // initial-close `[]` — the `.` is not immediately after `[`
        "[\\.a.]",   // escaped `\.` at class-open — the `\` breaks the opener
        "[\\[.a.]",  // escaped `\[` member, then dots — the `[` is escaped, no token
        "[[:alpha:]]", // posix class member — unaffected control
        "[a-z]",     // ordinary range — unaffected control
    ];

    #[test]
    fn class_bracket_tokens_reject_at_the_grammar_layer() {
        let mut wrong = Vec::new();
        for &pat in MUST_REJECT {
            if grammar_accepts(pat) {
                wrong.push(format!(
                    "{pat:?} was ACCEPTED (expected grammar REJECT — collating/equivalence token)"
                ));
            }
        }
        for &pat in MUST_ACCEPT {
            if !grammar_accepts(pat) {
                wrong.push(format!("{pat:?} was REJECTED (expected grammar ACCEPT)"));
            }
        }
        assert!(
            wrong.is_empty(),
            "grammar-layer verdicts diverged from pcre2test 10.47:\n  {}",
            wrong.join("\n  ")
        );
    }
}
