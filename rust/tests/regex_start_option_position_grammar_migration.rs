//! REGEX-PCRE2-FIDELITY.4.8 — grammar-only proof that a PCRE2 start option
//! (`(*UTF)`, `(*LIMIT_HEAP=5)`, `(*CRLF)`, …) is ACCEPTED only as a contiguous run at the
//! very start of the WHOLE pattern, and REJECTED anywhere else — after any construct, in a
//! later alternative, or nested inside any group at any depth.
//!
//! `pcre2test` 10.47 rejects a mis-positioned start option with error 160 "(*VERB) not
//! recognized or malformed" — FLAT (`a(*CR)b`, `a(*LIMIT_HEAP=500)`, `(*FAIL)(*LIMIT_HEAP=5)a`,
//! `(a)(*CRLF)`) and NESTED (`((*CRLF)a)`, `(?:(*CRLF)a)`, `(?=(*CRLF)a)`, `(*CRLF)((*LF)a)`,
//! `(*sr:(*CRLF)a)`, `(*scs:(1)(*CRLF)a)`, `(?i:(*CRLF)a)`, `(?|(*CRLF)a)`) — and binds to the
//! whole pattern, never per-alternative (`(*CRLF)a|(*LF)b` rejects; `(*CRLF)|a` accepts). It
//! ACCEPTS a leading run (`(*CRLF)abc`, `(*CRLF)(*LIMIT_MATCH=123)abc`, `(*CRLF)(a)`,
//! `(*CRLF)a|b`), a start-option-only pattern (`(*CRLF)`), the empty pattern, and verbs
//! anywhere (`((*ACCEPT))` — a VERB, not a start option).
//!
//! This slice migrated that reject from the out-of-band validator
//! (`regex_compile_validation::find_invalid_verb_construct` + `is_start_option_position`, now
//! DELETED) INTO `grammars/regex.ebnf` via a PURELY STRUCTURAL grammar shape (design A″): the
//! `start_option_piece` rule is reachable ONLY from the distinguished `entry_concatenation`
//! (the top-level-first-alternative concatenation off the `entry_alternation` chain). Every
//! later alternative and every nested pattern routes through the shared
//! `alternative`/`concatenation`, which have no start-option branch — so a mis-positioned
//! start option simply has NO derivation and fails to parse. No semantic fact / predicate is
//! used (unlike `.4.10`), so the rule costs nothing on the parse hot path, and the stimuli
//! generator cannot emit a mis-positioned start option either (duality-closed by construction).
//!
//! Behavior-NEUTRAL at the released `--parse` surface (the deleted validator rejected EXACTLY
//! this set; byte-identity of every accepted pattern is proven by the differential-equivalence
//! and ast-shape gates). The certified parse-harness interpreter (`interpret_parse`,
//! PARSE-HARNESS.5 — byte-identical to the generated regex parser) runs the grammar WITHOUT the
//! validator, so it proves the grammar-layer verdict directly (before the migration the grammar
//! ALONE ACCEPTED all of the violations — the single-source-of-truth hole this closes).

#[cfg(all(feature = "ebnf_dual_run", feature = "generated_parsers"))]
mod grammar_only {
    use pgen::parse_harness_interpreter::{interpret_parse, InterpretOptions};
    use std::path::PathBuf;

    fn regex_grammar() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../grammars/regex.ebnf")
    }

    fn grammar_accepts(input: &str) -> bool {
        interpret_parse(&regex_grammar(), input, &InterpretOptions::default())
            .unwrap_or_else(|e| panic!("interpreter failed on {input:?}: {e}"))
            .accepted
    }

    // A start option is valid ONLY as a contiguous prefix of the whole pattern. `pcre2test`
    // 10.47: err 160 for every case below.
    const MUST_REJECT: &[&str] = &[
        // FLAT — a top-level non-start-option precedes the start option.
        r"a(*CR)b",
        r"a(*LIMIT_HEAP=500)",
        r"(*FAIL)(*LIMIT_HEAP=5)a", // a verb ends the start-option prefix
        r"(a)(*CRLF)",              // after a group
        // NESTED — a start option inside ANY group, at any depth.
        r"((*CRLF)a)",       // capture
        r"(?:(*CRLF)a)",     // non-capture
        r"(?=(*CRLF)a)",     // lookaround
        r"(*CRLF)((*LF)a)",  // group after a leading start option, nested one still invalid
        r"(*sr:(*CRLF)a)",   // script-run
        r"(*scs:(1)(*CRLF)a)", // scan-substring
        r"(?i:(*CRLF)a)",    // scoped inline modifiers
        r"(?|(*CRLF)a)",     // branch-reset
        // GLOBAL start — never per-alternative.
        r"(*CRLF)a|(*LF)b",
        r"a(*CRLF)|b",
    ];

    // A leading start-option run (or none) — `pcre2test` 10.47 accepts. `((*ACCEPT))` is a VERB
    // (valid anywhere, incl. nested), NOT a start option.
    const MUST_ACCEPT: &[&str] = &[
        r"(*CRLF)abc",
        r"(*UTF)abc",
        r"(*CRLF)(*LIMIT_MATCH=123)abc",
        r"(*CRLF)(*UTF)(?:x)",
        r"(*CRLF)(a)",
        r"(*CRLF)a|b",
        r"((*ACCEPT))",
        r"(*CRLF)",  // start-option-only pattern (empty body)
        r"(*CRLF)|a",
        r"",         // empty pattern
        r"(*UTF)(*UCP)(*CRLF)(*LF)(*ANYCRLF)x",
        // Verbs / MARK stay valid anywhere (they are NOT start options).
        r"a(*PRUNE)b",
        r"(*MARK:x)y",
        r"(*:mk)x",
        // Ordinary patterns are unaffected.
        r"abc",
        r"(a|b)+",
        r"a\Qbc\Ed*e",
    ];

    #[test]
    fn start_option_rejected_off_the_entry_prefix() {
        for input in MUST_REJECT {
            assert!(
                !grammar_accepts(input),
                "grammar MUST reject mis-positioned start option {input:?} (pcre2test 10.47 err 160)"
            );
        }
    }

    #[test]
    fn start_option_accepted_at_the_entry_prefix() {
        for input in MUST_ACCEPT {
            assert!(
                grammar_accepts(input),
                "grammar MUST accept {input:?} (pcre2test 10.47 accepts)"
            );
        }
    }
}
