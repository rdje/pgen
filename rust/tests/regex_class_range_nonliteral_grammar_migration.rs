//! REGEX-PCRE2-FIDELITY.4.5.b — grammar-only proof that the non-`[` NONLITERAL
//! class-range reject is owned by the GRAMMAR (not just the out-of-band
//! `validate_regex_compile_contract`).
//!
//! Why the interpreter and not `parseability_probe --parse`: after `.4.5.b` the
//! compile-contract validator STILL rejects these inputs (its range-check
//! deletion is deferred to after `.4.5.c`), so the released `--parse` verdict is
//! UNCHANGED — reject before, reject after. The migration's effect is observable
//! only at the GRAMMAR layer. The parse-harness interpreter
//! (`interpret_parse`, PARSE-HARNESS.5 — CERTIFIED byte-identical to the
//! generated regex parser) runs the grammar WITHOUT the compile-contract
//! validator, so it is the authoritative grammar-only oracle for this migration.
//!
//! Frozen acceptance spec = `pcre2test` 10.47 `/PATTERN/utf` (err 150 "invalid
//! range" for a nonliteral shorthand/property endpoint; ACCEPT for the
//! literal-dash carve-outs). BEFORE the grammar edit this test FAILS on every
//! REJECT case (the grammar accepts the split members); AFTER it passes — that
//! pass/fail transition is the before→after verification for the `.4.5.b` leaf.

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

    // err 150 nonliteral endpoint — the grammar MUST reject after `.4.5.b`.
    // Shape 1 = nonliteral LEFT; shape 2 = literal LEFT / nonliteral RIGHT.
    const MUST_REJECT: &[&str] = &[
        // shape 1 — shorthand LEFT
        r"[\d-x]", r"[\D-x]", r"[\s-x]", r"[\S-x]", r"[\h-x]", r"[\H-x]", r"[\v-x]", r"[\V-x]",
        r"[\w-x]", r"[\W-x]",
        // shape 1 — property LEFT
        r"[\p{Lu}-x]", r"[\P{Lu}-x]", r"[\pL-x]",
        // shape 2 — shorthand RIGHT
        r"[a-\d]", r"[a-\D]", r"[a-\s]", r"[a-\w]", r"[a-\h]", r"[a-\v]",
        // shape 2 — property RIGHT
        r"[a-\p{Lu}]", r"[a-\pL]",
        // both nonliteral
        r"[\d-\w]", r"[\d-\p{Lu}]",
    ];

    // Literal-dash carve-outs + valid ranges — the grammar MUST keep accepting.
    const MUST_ACCEPT: &[&str] = &[
        "[a-]", "[-a]", r"[\d-]", r"[\d\-x]", "[a-z]", "[--/]", "[!--]",
        r"[\d]", r"[\p{Lu}]", r"[\s\w]", r"[a\d]", r"[\da-z]", r"[-\d]", r"[\d\d]",
        // valid literal ranges whose endpoints are ordinary escapes (NOT nonliteral)
        r"[\n-\r]", r"[\x30-\x39]",
    ];

    #[test]
    fn grammar_rejects_nonliteral_class_range_endpoints_and_keeps_carveouts() {
        let mut failures = Vec::new();
        eprintln!("── .4.5.b grammar-only matrix (interpreter, pcre2 profile) ──");
        for &p in MUST_REJECT {
            let acc = grammar_accepts(p);
            eprintln!("  expect REJECT  got {:<7} {p}", if acc { "ACCEPT" } else { "reject" });
            if acc {
                failures.push(format!("{p} — grammar ACCEPTS but must REJECT (err150 nonliteral endpoint)"));
            }
        }
        for &p in MUST_ACCEPT {
            let acc = grammar_accepts(p);
            eprintln!("  expect ACCEPT  got {:<7} {p}", if acc { "accept" } else { "REJECT" });
            if !acc {
                failures.push(format!("{p} — grammar REJECTS but must ACCEPT (carve-out / valid range)"));
            }
        }
        assert!(
            failures.is_empty(),
            "grammar-only class-range verdicts diverge from the pcre2test 10.47 oracle:\n  {}",
            failures.join("\n  ")
        );
    }

    /// Byte-parity pre-check: the `!invalid_class_range … -> $2` guard is zero-width, so every
    /// STILL-ACCEPTED input must keep the EXACT SAME typed AST. Compares the interpreter (which reads
    /// the EDITED grammar) against the currently-generated parser (the PRE-edit `1.1.96` artifact) —
    /// identical ASTs prove the wrapping preserved the shape before the (expensive) regen. Uses only
    /// PCRE2-valid inputs so the registry's compile-contract validator does not reject them.
    #[test]
    fn accepted_class_inputs_keep_byte_identical_ast_through_the_guard() {
        let inputs = [
            "[a-z]", "[abc]", "[^a-z]", "[]a]", "[a-z0-9_]", r"[\d]", r"[\w\s]", r"[a\d]",
            r"[\da-z]", r"[-\d]", "[a-]", "[-a]", r"[\d-]", r"[\d\-x]", "[--/]", "[!--]",
            r"[\n-\r]", r"[\x30-\x39]", r"[[:alpha:]]", r"[\p{Lu}]", "abc", "a[bc]d|e",
        ];
        let mut diffs = Vec::new();
        for input in inputs {
            let interp = interpret_parse(&regex_grammar(), input, &InterpretOptions::default())
                .unwrap_or_else(|e| panic!("interpreter failed on {input:?}: {e}"));
            let generated = pgen::parser_registry::parse_sample_ast_json_with_profile(
                "regex",
                input,
                Some("pcre2"),
            )
            .expect("regex is a registered grammar");
            match (&interp.ast_json, &generated) {
                (Some(a), Ok(b)) if a == b => {}
                (Some(a), Ok(b)) => diffs.push(format!(
                    "{input:?}: AST differs\n    interp(new): {a}\n    gen(old):    {b}"
                )),
                (i, g) => diffs.push(format!("{input:?}: mismatch interp={i:?} gen={g:?}")),
            }
        }
        assert!(
            diffs.is_empty(),
            "the `.4.5.b` guard changed the AST of an accepted input (byte-parity broken):\n  {}",
            diffs.join("\n  ")
        );
    }
}
