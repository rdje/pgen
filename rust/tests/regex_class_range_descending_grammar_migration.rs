//! REGEX-PCRE2-FIDELITY.4.5.c — grammar-only proof that the non-`[` DESCENDING
//! class-range reject is owned by the GRAMMAR (not just the out-of-band
//! `validate_regex_compile_contract`).
//!
//! A `class_range` of two LITERAL endpoints whose left code point exceeds the
//! right (`[z-a]`, `[9-0]`, `[\x39-\x30]`, `[\x{100}-a]`) is PCRE2 err 108
//! "range out of order in character class" (`pcre2test` 10.47). The grammar
//! ACCEPTED these before this slice — the `@validate: ord($1)<=ord($5)` on
//! `class_range` is a NON-parse-gating codegen annotation (proven `.4.5.a`) — so
//! only the compile-contract validator rejected them. `.4.5.c` adds a
//! `value_compare_codepoint` `@predicate` `post` gate (`args:[$1, le, $5]`,
//! RULE-SPAN-VALUE-CONSTRAINT.3) that decodes the two RAW endpoint spellings to
//! their code points and rejects left > right at the GRAMMAR layer.
//!
//! Why the interpreter and not `parseability_probe --parse`: the validator STILL
//! rejects these (its range-check deletion is deferred to `.4.5.d`), so the
//! released `--parse` verdict is UNCHANGED — reject before, reject after. The
//! migration's effect is observable only at the GRAMMAR layer, and the certified
//! parse-harness interpreter (`interpret_parse`, PARSE-HARNESS.5 — byte-identical
//! to the generated regex parser) runs the grammar WITHOUT the validator. Its
//! ACCEPT→REJECT transition on the descending cases is the before→after proof
//! AND the load-bearing check that raw positional `$1`/`$5` resolve to the
//! endpoint spellings on `class_range`'s 5-element `->` shape (the two
//! `class_zero_width*` slots between the atoms) via RAWCAP-TRANSFORM-PATH.2.

#[cfg(all(feature = "ebnf_dual_run", feature = "generated_parsers"))]
mod grammar_only {
    use pgen::parse_harness_interpreter::{interpret_parse, InterpretOptions};
    use std::path::PathBuf;

    fn regex_grammar() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../grammars/regex.ebnf")
    }

    /// Grammar-only verdict (default `pcre2` profile) via the certified interpreter.
    fn grammar_accepts(input: &str) -> bool {
        let out = interpret_parse(&regex_grammar(), input, &InterpretOptions::default())
            .unwrap_or_else(|e| panic!("interpreter failed on {input:?}: {e}"));
        out.accepted
    }

    // err 108 descending — the grammar MUST reject after `.4.5.c`. Two LITERAL
    // endpoints with left code point > right, across bare / hex / octal / braced
    // spellings (the value_compare_codepoint decode vocabulary).
    const MUST_REJECT: &[&str] = &[
        "[z-a]",       // 122 > 97 (bare)
        "[9-0]",       // 57 > 48
        "[b-a]",       // 98 > 97
        "[~-!]",       // 126 > 33
        "[Z-A]",       // 90 > 65
        r"[\x39-\x30]", // 57 > 48 (hex)
        r"[\x7a-\x61]", // 122 > 97 (hex)
        r"[\132-\101]", // octal 90 > 65 (Z-A)
        r"[\x{100}-a]", // 256 > 97 (braced hex left) — the CODE-POINT discriminator:
        //                 textual "\x{100}" < "a" ('\'=92 < 'a'=97) would ACCEPT.
        r"[\x{7a}-\x{61}]", // 122 > 97 (both braced)
    ];

    // Ascending, equal, and dash carve-outs (no range forms) — grammar MUST keep
    // accepting.
    const MUST_ACCEPT: &[&str] = &[
        "[a-z]",        // 97 < 122
        "[a-a]",        // 97 == 97 (equal endpoints, `le` holds)
        "[0-9]",        // 48 < 57
        "[!-~]",        // 33 < 126
        "[A-Z]",        // 65 < 90
        r"[\x30-\x39]",  // hex ascending
        r"[\x61-\x7a]",  // hex ascending
        r"[\101-\132]",  // octal ascending (A-Z)
        r"[a-\x{100}]",  // 97 < 256 (braced hex right)
        r"[\x{61}-\x{7a}]", // braced ascending
        // dash carve-outs — no `class_range` forms, so the gate never fires:
        "[a-]",
        "[-a]",
        r"[\d-]",
        r"[\d\-x]",     // escaped dash is a member
        "[abc]",        // no range at all
        "[--/]",        // 45 <= 47 (dash range, ascending)
    ];

    #[test]
    fn descending_literal_class_ranges_reject_at_the_grammar_layer() {
        let mut wrong = Vec::new();
        for &pat in MUST_REJECT {
            if grammar_accepts(pat) {
                wrong.push(format!("{pat:?} was ACCEPTED (expected grammar REJECT — descending)"));
            }
        }
        for &pat in MUST_ACCEPT {
            if !grammar_accepts(pat) {
                wrong.push(format!("{pat:?} was REJECTED (expected grammar ACCEPT)"));
            }
        }
        assert!(wrong.is_empty(), "grammar-layer verdicts diverged:\n  {}", wrong.join("\n  "));
    }
}
