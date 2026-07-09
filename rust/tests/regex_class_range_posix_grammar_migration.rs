//! REGEX-PCRE2-FIDELITY.4.5.d — grammar-only proof that the POSIX-class
//! (`[:name:]`) nonliteral class-range endpoint reject is owned by the GRAMMAR
//! (not just the out-of-band `validate_regex_compile_contract`).
//!
//! A `class_range` whose LEFT or RIGHT endpoint is a valid POSIX class is PCRE2
//! err 150 "invalid range in character class" (`pcre2test` 10.47):
//! `[[:alpha:]-z]` (posix LEFT), `[!-[:alpha:]]` (posix RIGHT). The grammar
//! ACCEPTED these before this slice — `class_atom` reads the bracket's `[` as a
//! `class_safe_special` literal 0x5B, so an ASCENDING `!-[` range forms and the
//! trailing `:alpha:]` become separate members — so only the compile-contract
//! validator rejected them. `.4.5.d` adds two `invalid_class_range` alternatives
//! (`posix_class - class_atom` / `class_atom - posix_class`) that reject the range
//! at the GRAMMAR layer, referencing the EXISTING positively-reachable
//! `posix_class` rule.
//!
//! The DESCENDING posix-right cases (`[x-[:alpha:]]`, left>`[`=0x5B) already
//! reject via `.4.5.c`'s `descending_class_range`; they are pinned here as
//! MUST_REJECT controls so a future refactor cannot silently un-own them.
//!
//! NOT fully behavior-neutral — a genuine correctness fix for one subset. For MOST
//! posix-endpoint cases the out-of-band validator ALREADY rejected them, so the
//! released `--parse` verdict is unchanged (observable only at the grammar layer).
//! But the validator has an accepts-invalid HOLE: `dash_is_trailing_literal` skips
//! whitespace/zero-width after the dash and treats it as a literal trailing dash even
//! for a NonLiteral (posix) LEFT endpoint — so `[[:digit:]-   ]` (dash + only
//! whitespace before `]`) was ACCEPTED by the released parser (grammar + validator),
//! where `pcre2test` 10.47 rejects it err 150. This slice's grammar migration flips
//! that subset ACCEPT→REJECT at the released `--parse` surface, PCRE2-convergently
//! (one contract success sample moved to failure). The certified parse-harness
//! interpreter (`interpret_parse`, PARSE-HARNESS.5 — byte-identical to the generated
//! regex parser) runs the grammar WITHOUT the validator, so it proves the grammar-layer
//! verdict directly; every MUST_REJECT / MUST_ACCEPT string was verified against
//! `pcre2test` 10.47.

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

    // err 150 — the grammar MUST reject after `.4.5.d`. A valid POSIX class as a
    // range endpoint, both orders, plus the descending posix-right cases already
    // owned by `.4.5.c` (pinned as regression controls). All verified REJECT by
    // `pcre2test` 10.47.
    const MUST_REJECT: &[&str] = &[
        "[!-[:alpha:]]", // ascending posix RIGHT (! 33 < [ 91) — the .4.5.d target
        "[[:alpha:]-z]", // posix LEFT
        "[[:alpha:]-a]", // posix LEFT, member right
        "[[:digit:]-9]", // posix LEFT, digit-name
        "[[:^alpha:]-z]", // negated posix LEFT
        "[[:alpha:]-[:digit:]]", // posix LEFT and RIGHT
        "[a-[:alpha:]]", // posix RIGHT (a 97 > [ 91 — also caught by .4.5.c descending)
        "[x-[:alpha:]]", // descending posix RIGHT (owned by .4.5.c; regression pin)
        "[a-[:digit:]]", // descending posix RIGHT (owned by .4.5.c; regression pin)
        // The BEHAVIOR-CHANGING flip subset: a posix LEFT endpoint where the dash is
        // followed only by whitespace before `]`. The released parser ACCEPTED these
        // before `.4.5.d` — the validator's `dash_is_trailing_literal` skips whitespace
        // and wrongly treated the dash as a literal trailing dash even for a NonLiteral
        // (posix) left endpoint — but `pcre2test` 10.47 rejects them err 150.
        "[[:digit:]-   ]", // dash + 3 spaces + ]
        "[[:alpha:]- ]",   // dash + 1 space + ]
        "[[:digit:]-\t]",  // dash + tab + ]
    ];

    // The grammar MUST keep accepting — no range forms, or the range is valid. All
    // verified ACCEPT by `pcre2test` 10.47.
    const MUST_ACCEPT: &[&str] = &[
        "[[:alpha:]]",          // bare posix class member
        "[[:alpha:]-]",         // posix then LITERAL trailing dash (carve-out: no atom after `-`)
        "[[:alpha:]-\\Q\\E]",   // posix then dash then ZERO-WIDTH \Q\E — dash effectively trailing (PCRE2 ACCEPT)
        "[[:alpha:]a]",         // posix then member (no dash)
        "[a[:alpha:]]",         // member then posix (no dash)
        "[a-z[:digit:]]",       // real range then posix member
        "[[:alpha:][:digit:]]", // two posix members, no range
        "[!-[]",                // ascending literal `!-[` range (33 < 91), NOT a posix token
        "[a-z]",                // ordinary range
        "[ -!]",                // ascending whitespace range (unaffected)
    ];

    #[test]
    fn posix_class_range_endpoints_reject_at_the_grammar_layer() {
        let mut wrong = Vec::new();
        for &pat in MUST_REJECT {
            if grammar_accepts(pat) {
                wrong.push(format!(
                    "{pat:?} was ACCEPTED (expected grammar REJECT — posix endpoint / descending)"
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
            "grammar-layer verdicts diverged:\n  {}",
            wrong.join("\n  ")
        );
    }
}
