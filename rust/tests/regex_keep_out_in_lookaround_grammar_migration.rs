//! REGEX-PCRE2-FIDELITY.4.10 — grammar-only proof that the `\K` (keep_out) anchor is
//! REJECTED inside any lookaround body, and ACCEPTED everywhere else.
//!
//! `pcre2test` 10.47 rejects `\K` inside a lookaround with error 199 "\K is not allowed
//! in lookarounds" for EVERY lookaround form — positive/negative lookahead (`(?=`/`(?!`),
//! positive/negative lookbehind (`(?<=`/`(?<!`), non-atomic lookahead/lookbehind
//! (`(?*`/`(?<*`), and the alpha forms (`(*pla:` … `(*naplb:`) — INCLUDING when the `\K`
//! sits inside a group nested inside the lookaround (`(?=a(b\Kc))`, `((?=x\Ky))`). It is
//! ACCEPTED bare (`a\Kb`, `\Kword`), inside an atomic / capturing / non-capturing group
//! (`(?>a\Kb)`, `(a\Kb)`, `(?:a\Kb)`), and AFTER a lookaround has closed (`(?=ab)\K`).
//!
//! This slice migrated that reject from the out-of-band validator
//! (`regex_compile_validation::find_invalid_keep_out_escape_in_lookaround`, now DELETED)
//! INTO `grammars/regex.ebnf`: each lookaround opens a `lookaround` semantic scope at its
//! opener (via a small open-marker rule) and closes it after its body, and the extracted
//! `keep_out` rule carries `@predicate not_in_scope_kind(lookaround)` — the parser-agnostic
//! scope-ancestry gate (SCOPE-CONTEXT-PREDICATE.1) that is true only when NO currently-open
//! scope (innermost frame OR any ancestor) is a lookaround. The whole-ancestor walk is what
//! rejects the nested `(?=a(b\Kc))`; the scope closing at the lookaround's end is what keeps
//! the trailing `(?=ab)\K` accepted.
//!
//! Behavior-NEUTRAL at the released `--parse` surface (the validator already rejected the
//! same set — its `find_matching_group_end` is depth-tracked and `find_keep_out_escape`
//! descends into nested groups, and its `is_alpha_lookaround_name` set equals the grammar's
//! `alpha_lookaround_name`); the migration moves ownership into the grammar (single source
//! of truth, [[project_ebnf_is_single_source_of_truth]]) and DELETES the standalone
//! validator check. The certified parse-harness interpreter (`interpret_parse`,
//! PARSE-HARNESS.5 — byte-identical to the generated regex parser) runs the grammar WITHOUT
//! the validator, so it proves the grammar-layer verdict directly (before the migration the
//! grammar ALONE ACCEPTED all of these — the single-source-of-truth hole).

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

    // `\K` inside a lookaround body — the grammar MUST reject after `.4.10`. err 199 by
    // `pcre2test` 10.47. Every lookaround form + the nested cases.
    const MUST_REJECT: &[&str] = &[
        r"(?=a\Kb)",     // positive lookahead
        r"(?!a\Kb)",     // negative lookahead
        r"(?<=a\Kb)",    // positive lookbehind
        r"(?<!a\Kb)",    // negative lookbehind
        r"(?*a\Kb)",     // non-atomic lookahead
        r"(?<*a\Kb)",    // non-atomic lookbehind
        r"(*pla:a\Kb)",  // alpha positive lookahead
        r"(*plb:a\Kb)",  // alpha positive lookbehind
        r"(*nla:a\Kb)",  // alpha negative lookahead
        r"(*nlb:a\Kb)",  // alpha negative lookbehind
        r"(*napla:a\Kb)", // alpha non-atomic positive lookahead
        r"(*naplb:a\Kb)", // alpha non-atomic positive lookbehind
        r"(*positive_lookahead:a\Kb)", // long alpha name
        r"(?=a(b\Kc))",  // \K in a capture group nested in the lookahead (whole-chain walk)
        r"((?=x\Ky))",   // lookahead nested in a capture group
        r"(?=a(?:b\Kc))", // \K in a non-capturing group nested in the lookahead
        r"(?:(?=a\Kb))", // lookahead nested in a non-capturing group
        r"(?=a|b\Kc)",   // \K in an alternation branch inside the lookahead
        r"(?=(?<=x\Ky))", // \K in a lookbehind nested in a lookahead
    ];

    // The grammar MUST keep accepting — `\K` outside any lookaround. All ACCEPT by
    // `pcre2test` 10.47.
    const MUST_ACCEPT: &[&str] = &[
        r"a\Kb",       // bare
        r"\Kword",     // bare leading
        r"\K",         // lone keep_out
        r"(?>a\Kb)",   // atomic group
        r"(a\Kb)",     // capturing group
        r"(?:a\Kb)",   // non-capturing group
        r"(?<name>a\Kb)", // named capturing group
        r"(?=ab)\K",   // AFTER a positive lookahead has closed
        r"(?<=ab)\Kc", // AFTER a positive lookbehind has closed
        r"(*pla:ab)\K", // AFTER an alpha lookahead has closed
        r"(?=a(bc))\Kd", // AFTER a lookahead with an inner capture, \K outside
        r"a(?=b)\Kc",  // \K after a mid-pattern lookahead
        // plain lookarounds without \K must still parse
        r"(?=ab)",
        r"(?<=ab)",
        r"(*pla:ab)",
        r"(?=a(bc))",
    ];

    #[test]
    fn keep_out_in_lookaround_rejects_at_the_grammar_layer() {
        let mut wrong = Vec::new();
        for &pat in MUST_REJECT {
            if grammar_accepts(pat) {
                wrong.push(format!(
                    "{pat:?} was ACCEPTED (expected grammar REJECT — \\K inside a lookaround)"
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
