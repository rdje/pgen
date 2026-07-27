//! PARSE-HARNESS.6.1 — the structural combinator isolating suite.
//!
//! # What this is
//!
//! The end-to-end differential-equivalence gate (`PARSE-HARNESS.5`,
//! [`crate::parse_harness_equivalence`]) proves the grammar-AST interpreter
//! ([`crate::parse_harness_interpreter`]) byte-identical to the shipped generated parser — but only over
//! the constructs the **shipped** grammars happen to use. To trust the interpreter on an **arbitrary /
//! synthetic** grammar (the whole point of the harness — e.g. the `GRAMMAR-WELLFORMED.A2.3` `a | ab`
//! probe), we need equivalence **per combinator**, in isolation.
//!
//! This module is that suite: a table of **small isolating grammars**, one per structural combinator the
//! interpreter core (`.4`) dispatches, each differentially verified over targeted inputs. For every
//! `(grammar, input)` pair it runs **both** the interpreter ([`interpret_parse`]) and the compile-and-run
//! harness ([`compile_and_parse`], approach 2 — authoritative *by construction*, the real codegen + real
//! runtime) and asserts they agree **byte for byte**: the accept/reject verdict, `furthest_position` on
//! reject, and the typed AST on accept. Any divergence names the combinator, the exact input, and the
//! diff.
//!
//! # Why the compile-and-run oracle (not the registry)
//!
//! The isolating grammars are *synthetic* — they are never registered / compiled in, so the registry
//! oracle the `.5` gate uses does not apply. [`compile_and_parse`] runs the shipped codegen on the
//! synthetic `.ebnf` and compiles the emitted parser as a throwaway crate, so it *is* the real generated
//! parser for that grammar (authoritative by construction, `PARSE-HARNESS.3`). It is the natural oracle
//! for a per-combinator suite over made-up grammars.
//!
//! # Report-first, never-panic
//!
//! [`run_combinator_case`] / [`evaluate_all_combinator_cases`] *collect* divergences into
//! [`CombinatorCaseReport`]s rather than asserting — so a caller can measure the honest state of every
//! combinator in one pass (the scouting use) *and* a gate test can assert every report `is_clean()` (the
//! enforcement use). Same "measure, then lock" discipline the rest of the platform uses.
//!
//! # Determinism
//!
//! Unlike the `.5` gate (whose corpus is the seeded stimuli generator), this suite's corpus is a **fixed
//! curated input set** — the differential is `synthetic grammar × curated input`, with **no randomness**,
//! so it is deterministic by construction (the "deterministic at seeds 0/7/42" requirement is trivially
//! satisfied: there is no seed to vary).
//!
//! # Honest scope (`PARSE-HARNESS.md` §3.3 / §3.4)
//!
//! This is the **structural** combinator surface — exactly what the `.4` interpreter core dispatches:
//! ordered choice under each `branch_policy`, sequence + backtrack, the `?`/`*`/`+` quantifiers (incl. the
//! zero-length guard), the four bounded quantifier forms `{N}`/`{N,M}`/`{N,}`/`{,M}` (first-class since
//! BOUNDED-QUANT.1), lookahead `&`/`!`, atoms/terminals/regex-tokens, rule references, and
//! LR-eliminated left recursion. The **semantic-directive** surface that gates parse *outcomes* on the
//! store (`@predicate`/`@emit_fact`/scope/rollback + memoization) is the sibling leaf `.6.2` and is *not*
//! covered here.
//!
//! (Historical note — the `.6.1` landing documented bounded quantifiers as *unreachable through the
//! oracle*: codegen aborted with `Unknown quantifier: 2` because the canonical
//! `parse_quantifier_bounds` decoder spoke only the braced spelling while the EBNF frontend emits the
//! brace-STRIPPED raw-AST token `["quantifier","2"]`. `BOUNDED-QUANT.1` closed that half-wire in the
//! one shared decoder, and the four `quant_bounded_*` cases below are the per-combinator differential
//! proof.)
//!
//! One construct remains **documented rather than silently dropped** (no silent caps,
//! [[feedback_always_signoff_decisions]]):
//!
//! - **Non-default `branch_policy` still selects a *branch*, not the verdict via the store.** The three
//!   policies below (`longest_match` / `ordered` / `priority_first`) are structural (they pick which
//!   alternative wins purely by consumed-length / source-order / `@priority`); they are in scope. The
//!   `@predicate`-gated verdict changes are `.6.2`.
//! - **Bare *direct* left-recursion `A := A x | y` is a KNOWN divergence, out of scope.** PGEN's
//!   structural LR-elimination only rewrites the **wrapper/indirect** form (`A := wrapper | base`,
//!   `wrapper := A suffix`) — the `left_recursion` case below uses that eliminated form. Bare direct
//!   recursion is left to *runtime cycle-breaking* (`RecursionGuard`), where the interpreter and the
//!   generated parser agree on the verdict but diverge on `furthest_position` (measured). That is a
//!   genuine interpreter-fidelity gap surfaced for follow-up — see
//!   [`DIRECT_LEFT_RECURSION_KNOWN_DIVERGENCE`] — not a `.6.1` regression.

use std::path::{Path, PathBuf};

use crate::parse_harness::{CompileAndParseOptions, ParseOutcome, compile_and_parse};

/// The structural combinator an isolating grammar exercises. One variant per construct the `.4`
/// interpreter core dispatches; the [`combinator_coverage_is_complete`](gate) gate test asserts every
/// variant is present in [`COMBINATOR_CASES`], so a construct cannot be silently unmeasured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Combinator {
    /// Ordered choice under the default (`longest_match`) policy — picks the longest-consuming alt.
    ChoiceLongestMatchDefault,
    /// Ordered choice under an *explicit* `@branch_policy: longest_match` — same semantics, different path.
    ChoiceLongestMatchExplicit,
    /// Ordered choice under `@branch_policy: ordered` — picks the FIRST matching alt (PEG-style).
    ChoiceOrdered,
    /// Ordered choice under `@branch_policy: priority_first` + `@priority` — picks the highest `@priority`.
    ChoicePriorityFirst,
    /// `@associativity: left` (the DEFAULT) — on an end+priority TIE the incumbent (earlier) alt keeps
    /// the win. `GENERATED-LINT-CORRECTNESS.2` added these three rows: the associativity tie-break is
    /// resolved at CODEGEN in the generated parser and at RUNTIME in the interpreter, so this is a
    /// genuine differential over the fold.
    AssociativityLeft,
    /// `@associativity: right` — on a TIE the LATER alt dethrones the incumbent.
    AssociativityRight,
    /// `@associativity: nonassoc` — a TIE fails the WHOLE choice.
    AssociativityNonAssoc,
    /// The `e? | keyword` always-succeeds shape (the `A2.2`/`A2.3` case): a branch that always matches
    /// (possibly empty) does NOT shadow a later branch under backtracking `longest_match`.
    AlwaysSucceeds,
    /// A sequence whose first alt shares a prefix with the second (`"a" "b" | "a" "c"`) — backtrack after
    /// the shared prefix.
    SequenceBacktrack,
    /// The `?` (optional) quantifier.
    QuantifierOptional,
    /// The `*` (zero-or-more) quantifier.
    QuantifierStar,
    /// The `+` (one-or-more) quantifier.
    QuantifierPlus,
    /// The `{N}` exact-count bounded quantifier (BOUNDED-QUANT.1).
    QuantifierBoundedExact,
    /// The `{N,M}` range bounded quantifier (BOUNDED-QUANT.1).
    QuantifierBoundedRange,
    /// The `{N,}` at-least bounded quantifier (BOUNDED-QUANT.1).
    QuantifierBoundedAtLeast,
    /// The `{,M}` at-most bounded quantifier (BOUNDED-QUANT.1).
    QuantifierBoundedAtMost,
    /// A `*` over a *nullable* element — exercises the zero-length guard (the loop must not spin forever
    /// on an element that matches empty).
    QuantifierZeroLengthGuard,
    /// Negative lookahead `!e` (zero-width, must fail if `e` matches).
    LookaheadNegative,
    /// Positive lookahead `&e` (zero-width, must succeed if `e` matches).
    LookaheadPositive,
    /// A plain terminal atom (`"hello"`).
    AtomTerminal,
    /// A regex-token atom (`/[0-9]+/`).
    AtomRegexToken,
    /// A rule reference chain (`start := a b`).
    RuleReference,
    /// Left recursion in the wrapper/indirect form PGEN structurally eliminates
    /// (`expr := wrapper | term`, `wrapper := expr "+" term`), rewritten to `base (suffix)*`.
    LeftRecursion,
    /// The DEFAULT layout policy (no `@whitespace_sensitive:` directive): layout is auto-skipped
    /// before terminals and after the entry rule (WS-DIRECTIVE.2 contrast case).
    LayoutInsensitiveDefault,
    /// The grammar-level `@whitespace_sensitive: true` directive: NO layout skip anywhere —
    /// whitespace is literal input (WS-DIRECTIVE.2; the policy regex declares).
    LayoutWhitespaceSensitiveFull,
    /// The granular `@whitespace_sensitive: { regex_tokens: true }` facet: regex tokens are
    /// whitespace-sensitive while terminals and trailing layout keep the default skip
    /// (WS-DIRECTIVE.2; the policy systemverilog_preprocessor declares).
    LayoutWhitespaceSensitiveRegexTokens,
    /// No `@default_profile:` directive: an unspecified profile leaves the guard PERMISSIVE, so
    /// `@profiles`-gated rules are active (DEFAULT-PROFILE.2 contrast case).
    ProfileUnspecifiedPermissive,
    /// The grammar-level `@default_profile:` directive: an unspecified profile resolves to the
    /// DECLARED default, so `@profiles`-gated rules outside it are excluded by default
    /// (DEFAULT-PROFILE.2; the policy regex declares as `pcre2`).
    ProfileDefaultGate,
    /// The grammar-level `@profile_alias:` directive: a requested ALIAS spelling resolves to its
    /// declared canonical profile, so the `@profiles`-gated rule it names is ACTIVE
    /// (PROFILE-ALIAS.2; the policy systemverilog declares — `2017` → `sv_2017`, …).
    ProfileAliasResolves,
    /// The alias pass-through contrast: an UNDECLARED spelling passes through unresolved, so a
    /// `@profiles`-gated rule stays EXCLUDED (PROFILE-ALIAS.2 — unknown values are not coerced).
    ProfileAliasUnknownPassthrough,
}

impl Combinator {
    /// Every combinator the suite must cover — the completeness universe.
    pub const ALL: &'static [Combinator] = &[
        Combinator::ChoiceLongestMatchDefault,
        Combinator::ChoiceLongestMatchExplicit,
        Combinator::ChoiceOrdered,
        Combinator::ChoicePriorityFirst,
        Combinator::AssociativityLeft,
        Combinator::AssociativityRight,
        Combinator::AssociativityNonAssoc,
        Combinator::AlwaysSucceeds,
        Combinator::SequenceBacktrack,
        Combinator::QuantifierOptional,
        Combinator::QuantifierStar,
        Combinator::QuantifierPlus,
        Combinator::QuantifierBoundedExact,
        Combinator::QuantifierBoundedRange,
        Combinator::QuantifierBoundedAtLeast,
        Combinator::QuantifierBoundedAtMost,
        Combinator::QuantifierZeroLengthGuard,
        Combinator::LookaheadNegative,
        Combinator::LookaheadPositive,
        Combinator::AtomTerminal,
        Combinator::AtomRegexToken,
        Combinator::RuleReference,
        Combinator::LeftRecursion,
        Combinator::LayoutInsensitiveDefault,
        Combinator::LayoutWhitespaceSensitiveFull,
        Combinator::LayoutWhitespaceSensitiveRegexTokens,
        Combinator::ProfileUnspecifiedPermissive,
        Combinator::ProfileDefaultGate,
        Combinator::ProfileAliasResolves,
        Combinator::ProfileAliasUnknownPassthrough,
    ];
}

/// One isolating grammar + its targeted inputs.
#[derive(Debug, Clone, Copy)]
pub struct CombinatorCase {
    /// A stable, filesystem-safe name (the synthetic `.ebnf` stem).
    pub name: &'static str,
    /// The combinator this case isolates.
    pub combinator: Combinator,
    /// The full grammar body (entry rule first).
    pub grammar_body: &'static str,
    /// `(input, expected_accept)` — the `expected_accept` is an INDEPENDENT sanity anchor reasoned from
    /// the combinator's spec (NOT the oracle), proving the case is *live* / discriminating. The
    /// load-bearing certification is `interpreter == oracle`, which needs no anchor.
    pub inputs: &'static [(&'static str, bool)],
    /// The start symbol to parse from, applied **identically** to both the interpreter and the oracle;
    /// `None` = each grammar's canonical entry (`rule_order[0]`). Needed for `left_recursion`: PGEN's
    /// LR-elimination **prepends** the `_lr_base` / `_lr_suffix` helper rules to `rule_order`, so
    /// `rule_order[0]` is no longer the semantic entry `expr` — it becomes the base rule (just `"n"`).
    /// Driving the LR combinator therefore requires naming the real entry explicitly.
    pub entry_rule: Option<&'static str>,
    /// `PROFILE-ALIAS.2`: the dialect-profile SPELLING to request before parsing, applied
    /// **identically** to both sides (the interpreter via `InterpretOptions::profile`, the oracle via
    /// `CompileAndParseOptions::requested_profile` → the probe's `set_grammar_profile`). `None` = no
    /// request (each grammar's constructor posture — the declared `@default_profile` if any, else
    /// unset/permissive). This is how the alias-resolution cases drive a declared spelling end-to-end.
    pub requested_profile: Option<&'static str>,
    /// A short note on what the case proves (surfaced in the scouting report).
    pub note: &'static str,
}

/// The systematic per-combinator table. Every [`Combinator::ALL`] variant appears at least once.
///
/// Every `grammar_body` here was tool-validated to compile through `ast_pipeline --generate-parser`
/// (so the compile-and-run oracle can build it), and every `expected_accept` anchor was reasoned from
/// the branch-policy / quantifier / lookahead spec and confirmed against the oracle.
pub const COMBINATOR_CASES: &[CombinatorCase] = &[
    // ── Ordered choice under each branch_policy — the A2.2/A2.3 load-bearing cases ──────────────────
    CombinatorCase {
        name: "choice_longest_default",
        combinator: Combinator::ChoiceLongestMatchDefault,
        // Default policy = longest_match: on "ab" the LONGER alt (`"a" "b"`) must win.
        grammar_body: "@entry: true\nstart := \"a\" | \"a\" \"b\"\n",
        inputs: &[("ab", true), ("a", true), ("b", false), ("abc", false)],
        entry_rule: None,
        requested_profile: None,
        note: "default longest_match picks the longer alt on \"ab\" (accept)",
    },
    CombinatorCase {
        name: "choice_longest_explicit",
        combinator: Combinator::ChoiceLongestMatchExplicit,
        grammar_body: "@entry: true\n@branch_policy: longest_match\nstart := \"a\" | \"a\" \"b\"\n",
        inputs: &[("ab", true), ("a", true), ("b", false)],
        entry_rule: None,
        requested_profile: None,
        note: "explicit longest_match matches the default (accept \"ab\")",
    },
    CombinatorCase {
        name: "choice_ordered",
        combinator: Combinator::ChoiceOrdered,
        // ordered = PEG first-match: on "ab" the FIRST alt (`"a"`) wins → leaves "b" → full-parse REJECT.
        // This is the decisive A2.3 contrast with longest_match (which accepts "ab").
        grammar_body: "@entry: true\n@branch_policy: ordered\nstart := \"a\" | \"a\" \"b\"\n",
        inputs: &[("ab", false), ("a", true), ("b", false)],
        entry_rule: None,
        requested_profile: None,
        note: "ordered picks the FIRST alt on \"ab\" → leaves \"b\" → reject (the A2.3 contrast)",
    },
    CombinatorCase {
        name: "choice_priority_first",
        combinator: Combinator::ChoicePriorityFirst,
        // priority_first + @priority [1,2]: alt 1 (`"a" "b"`) has the higher priority (2) → it wins over
        // source order → accepts "ab" (distinct from `ordered`, which rejects "ab").
        grammar_body: "@entry: true\n@branch_policy: priority_first\n@priority: [1, 2]\nstart := \"a\" | \"a\" \"b\"\n",
        inputs: &[("ab", true), ("a", true), ("b", false)],
        entry_rule: None,
        requested_profile: None,
        note: "priority_first picks the higher-@priority alt (2) → accepts \"ab\" (reorders vs source)",
    },
    // ── The associativity tie-break (GENERATED-LINT-CORRECTNESS.2) ──────────────────────────────────
    // All three share ONE shape: both alts consume exactly "xy", so end AND priority TIE and the
    // associativity tie-break is the only thing that can pick a winner. The alts build DIFFERENT ASTs
    // (a two-element sequence vs a single fused terminal), so the choice is OBSERVABLE in the compared
    // typed AST rather than merely internal — without that the rows would agree vacuously.
    CombinatorCase {
        name: "assoc_left",
        combinator: Combinator::AssociativityLeft,
        grammar_body: "@entry: true\n@associativity: left\nstart := pair | fused\npair := \"x\" \"y\"\nfused := \"xy\"\n",
        inputs: &[("xy", true), ("x", false), ("z", false)],
        entry_rule: None,
        requested_profile: None,
        note: "left (the default): a TIE does NOT dethrone — the earlier `pair` alt keeps the win",
    },
    CombinatorCase {
        name: "assoc_right",
        combinator: Combinator::AssociativityRight,
        grammar_body: "@entry: true\n@associativity: right\nstart := pair | fused\npair := \"x\" \"y\"\nfused := \"xy\"\n",
        inputs: &[("xy", true), ("x", false), ("z", false)],
        entry_rule: None,
        requested_profile: None,
        note: "right: a TIE dethrones the incumbent — the later `fused` alt wins (different AST)",
    },
    CombinatorCase {
        name: "assoc_nonassoc",
        combinator: Combinator::AssociativityNonAssoc,
        grammar_body: "@entry: true\n@associativity: nonassoc\nstart := pair | fused\npair := \"x\" \"y\"\nfused := \"xy\"\n",
        inputs: &[("xy", false), ("x", false), ("z", false)],
        entry_rule: None,
        requested_profile: None,
        note: "nonassoc: a TIE fails the WHOLE choice — \"xy\" is REJECTED where left/right accept",
    },
    CombinatorCase {
        name: "always_succeeds",
        combinator: Combinator::AlwaysSucceeds,
        // The A2.2 shape: `opt` always succeeds (matches empty), yet under longest_match it does NOT
        // shadow `kw` — on "keyword" the longer `kw` branch wins. On "z" only the empty `opt` matches
        // (0 chars) → "z" left → reject.
        grammar_body: "@entry: true\nstart := opt | kw\nopt := \"x\"?\nkw := \"keyword\"\n",
        inputs: &[("x", true), ("", true), ("keyword", true), ("z", false)],
        entry_rule: None,
        requested_profile: None,
        note: "always-succeeds `opt` does NOT shadow `kw` under longest_match (A2.2 unsoundness point)",
    },
    // ── Sequence + backtrack ─────────────────────────────────────────────────────────────────────────
    CombinatorCase {
        name: "sequence_backtrack",
        combinator: Combinator::SequenceBacktrack,
        grammar_body: "@entry: true\nstart := \"a\" \"b\" | \"a\" \"c\"\n",
        inputs: &[("ab", true), ("ac", true), ("ad", false), ("a", false)],
        entry_rule: None,
        requested_profile: None,
        note: "backtrack after the shared `\"a\"` prefix to try the second alt",
    },
    // ── Quantifiers ?/*/+ and the four bounded forms (first-class since BOUNDED-QUANT.1) ─────────────
    CombinatorCase {
        name: "quant_optional",
        combinator: Combinator::QuantifierOptional,
        grammar_body: "@entry: true\nstart := \"a\" item?\nitem := \"b\"\n",
        inputs: &[("a", true), ("ab", true), ("abb", false), ("", false)],
        entry_rule: None,
        requested_profile: None,
        note: "`?` optionally consumes one `item`",
    },
    CombinatorCase {
        name: "quant_star",
        combinator: Combinator::QuantifierStar,
        grammar_body: "@entry: true\nstart := item*\nitem := \"x\"\n",
        inputs: &[("", true), ("x", true), ("xxx", true), ("xy", false)],
        entry_rule: None,
        requested_profile: None,
        note: "`*` consumes zero-or-more `item`",
    },
    CombinatorCase {
        name: "quant_plus",
        combinator: Combinator::QuantifierPlus,
        grammar_body: "@entry: true\nstart := item+\nitem := \"x\"\n",
        inputs: &[("x", true), ("xxx", true), ("", false), ("xy", false)],
        entry_rule: None,
        requested_profile: None,
        note: "`+` consumes one-or-more `item`",
    },
    CombinatorCase {
        name: "quant_bounded_exact",
        combinator: Combinator::QuantifierBoundedExact,
        // `{2}` = exactly two: fewer fails the min-count (whole quantifier rolls back), more leaves
        // unconsumed input past the max → full-parse reject on both sides of the window.
        grammar_body: "@entry: true\nstart := item{2}\nitem := \"x\"\n",
        inputs: &[("xx", true), ("x", false), ("xxx", false), ("", false)],
        entry_rule: None,
        requested_profile: None,
        note: "`{2}` accepts exactly two `item`s (the BOUNDED-QUANT.1 half-wire closure proof)",
    },
    CombinatorCase {
        name: "quant_bounded_range",
        combinator: Combinator::QuantifierBoundedRange,
        grammar_body: "@entry: true\nstart := item{2,3}\nitem := \"x\"\n",
        inputs: &[("x", false), ("xx", true), ("xxx", true), ("xxxx", false)],
        entry_rule: None,
        requested_profile: None,
        note: "`{2,3}` accepts the 2..=3 window and nothing outside it",
    },
    CombinatorCase {
        name: "quant_bounded_at_least",
        combinator: Combinator::QuantifierBoundedAtLeast,
        grammar_body: "@entry: true\nstart := item{2,}\nitem := \"x\"\n",
        inputs: &[("", false), ("x", false), ("xx", true), ("xxxxx", true)],
        entry_rule: None,
        requested_profile: None,
        note: "`{2,}` needs at least two `item`s, unbounded above",
    },
    CombinatorCase {
        name: "quant_bounded_at_most",
        combinator: Combinator::QuantifierBoundedAtMost,
        grammar_body: "@entry: true\nstart := item{,2}\nitem := \"x\"\n",
        inputs: &[("", true), ("x", true), ("xx", true), ("xxx", false)],
        entry_rule: None,
        requested_profile: None,
        note: "`{,2}` accepts zero through two `item`s; a third is unconsumed → reject",
    },
    CombinatorCase {
        name: "quant_zero_length_guard",
        combinator: Combinator::QuantifierZeroLengthGuard,
        // `item := "x"?` always succeeds (matches empty), so `item*` must invoke the zero-length guard to
        // avoid an infinite loop: it stops the star as soon as an iteration consumes 0 bytes.
        grammar_body: "@entry: true\nstart := item*\nitem := \"x\"?\n",
        inputs: &[("", true), ("x", true), ("xx", true), ("y", false)],
        entry_rule: None,
        requested_profile: None,
        note: "`*` over a nullable `item` must stop at the first zero-length iteration (the guard)",
    },
    // ── Lookahead &/! (zero-width) ───────────────────────────────────────────────────────────────────
    CombinatorCase {
        name: "lookahead_negative",
        combinator: Combinator::LookaheadNegative,
        grammar_body: "@entry: true\nstart := !\"x\" any\nany := \"y\" | \"z\"\n",
        inputs: &[("y", true), ("z", true), ("x", false)],
        entry_rule: None,
        requested_profile: None,
        note: "`!\"x\"` fails when the input starts with \"x\"; zero-width otherwise",
    },
    CombinatorCase {
        name: "lookahead_positive",
        combinator: Combinator::LookaheadPositive,
        grammar_body: "@entry: true\nstart := &digit rest\ndigit := \"1\" | \"2\"\nrest := digit \"!\"\n",
        inputs: &[("1!", true), ("2!", true), ("1", false), ("x!", false)],
        entry_rule: None,
        requested_profile: None,
        note: "`&digit` requires a digit next (zero-width), then `rest` consumes it",
    },
    // ── Atoms: terminal + regex-token ────────────────────────────────────────────────────────────────
    CombinatorCase {
        name: "atom_terminal",
        combinator: Combinator::AtomTerminal,
        grammar_body: "@entry: true\nstart := \"hello\"\n",
        inputs: &[("hello", true), ("hell", false), ("helloo", false), ("", false)],
        entry_rule: None,
        requested_profile: None,
        note: "a plain terminal atom must match exactly",
    },
    CombinatorCase {
        name: "atom_regex_token",
        combinator: Combinator::AtomRegexToken,
        grammar_body: "@entry: true\nstart := /[0-9]+/\n",
        inputs: &[("123", true), ("7", true), ("", false), ("12a", false), ("a", false)],
        entry_rule: None,
        requested_profile: None,
        note: "a `/…/` regex-token atom matches its pattern anchored at the cursor",
    },
    // ── Rule reference ───────────────────────────────────────────────────────────────────────────────
    CombinatorCase {
        name: "rule_reference",
        combinator: Combinator::RuleReference,
        grammar_body: "@entry: true\nstart := a b\na := \"x\"\nb := \"y\"\n",
        inputs: &[("xy", true), ("x", false), ("xyz", false), ("yx", false)],
        entry_rule: None,
        requested_profile: None,
        note: "`start` dispatches to referenced rules `a` then `b`",
    },
    // ── Left recursion (LR-eliminated) ───────────────────────────────────────────────────────────────
    CombinatorCase {
        name: "left_recursion",
        combinator: Combinator::LeftRecursion,
        // The LR-ELIMINATED form PGEN actually rewrites. `expr := wrapper | term` whose first alt is a
        // BARE rule-ref to a wrapper rule `wrapper := expr "+" term` triggers
        // `detect_left_recursive_chain_plan` (`mod.rs`) → codegen rewrites `expr` to
        // `expr_lr_base (expr_lr_suffix)*` (tool-verified: 4 `_lr_base` + 6 `_lr_suffix` helper nodes in
        // the gen-AST). The interpreter and the oracle run the SAME `transform_from_raw_ast`
        // LR-elimination (both default `eliminate_left_recursion = true`), so they see the identical
        // eliminated tree. Bare DIRECT recursion `A := A x | y` does NOT match the wrapper pattern
        // (`extract_rule_reference_name` rejects a multi-element sequence) and is left to runtime
        // cycle-breaking — see [`DIRECT_LEFT_RECURSION_KNOWN_DIVERGENCE`] and the module honest-bounds.
        grammar_body: "@entry: true\nexpr := wrapper | term\nwrapper := expr \"+\" term\nterm := \"n\"\n",
        inputs: &[("n", true), ("n+n", true), ("n+n+n", true), ("n+", false), ("+n", false)],
        entry_rule: Some("expr"),
        requested_profile: None,
        note: "the wrapper/indirect LR form is structurally eliminated to `base (suffix)*`",
    },
    // ── Layout policy — the grammar-level `@whitespace_sensitive:` directive (WS-DIRECTIVE.2) ──────
    CombinatorCase {
        name: "layout_insensitive_default",
        combinator: Combinator::LayoutInsensitiveDefault,
        // No directive = the whitespace-INSENSITIVE default: leading layout is skipped before every
        // terminal and trailing layout is consumed after the entry rule.
        grammar_body: "@entry: true\nstart := \"a\" \"b\"\n",
        inputs: &[
            ("ab", true),
            ("a b", true),
            (" ab", true),
            ("ab ", true),
            ("a x", false),
        ],
        entry_rule: None,
        requested_profile: None,
        note: "default layout policy skips interior/leading layout and consumes trailing layout",
    },
    CombinatorCase {
        name: "layout_ws_sensitive_full",
        combinator: Combinator::LayoutWhitespaceSensitiveFull,
        // `@whitespace_sensitive: true` (the regex.ebnf policy): NO layout skip anywhere — the same
        // inputs the default case accepts must now REJECT whenever they carry whitespace. This is the
        // capability the retired grammar-NAME gate closed off from synthetic grammars entirely.
        grammar_body: "@entry: true\n@whitespace_sensitive: true\nstart := \"a\" \"b\"\n",
        inputs: &[
            ("ab", true),
            ("a b", false),
            (" ab", false),
            ("ab ", false),
        ],
        entry_rule: None,
        requested_profile: None,
        note: "@whitespace_sensitive: true makes every space literal (contrast with the default case)",
    },
    CombinatorCase {
        name: "layout_ws_sensitive_regex_tokens",
        combinator: Combinator::LayoutWhitespaceSensitiveRegexTokens,
        // `@whitespace_sensitive: { regex_tokens: true }` (the systemverilog_preprocessor policy):
        // ONLY regex tokens are whitespace-sensitive — terminals still skip leading layout and
        // trailing layout is still consumed.
        grammar_body: "@entry: true\n@whitespace_sensitive: { regex_tokens: true }\nstart := \"k\" /[a-z]+/\n",
        inputs: &[
            ("kx", true),
            ("k x", false),
            (" kx", true),
            ("kx ", true),
        ],
        entry_rule: None,
        requested_profile: None,
        note: "granular facet: regex tokens sensitive, terminals + trailing keep the default skip",
    },
    // ── Default profile — the grammar-level `@default_profile:` directive (DEFAULT-PROFILE.2) ──────
    CombinatorCase {
        name: "profile_unspecified_permissive",
        combinator: Combinator::ProfileUnspecifiedPermissive,
        // No directive = the permissive default: with no profile requested, the
        // `rule_profile_is_enabled` guard treats `None` as "all rules active", so the
        // `@profiles: ["relaxed"]`-gated branch IS reachable.
        grammar_body: "@entry: true\nstart := base | relaxed_only\nbase := \"b\"\n@profiles: [\"relaxed\"]\nrelaxed_only := \"R\"\n",
        inputs: &[("b", true), ("R", true), ("x", false)],
        entry_rule: None,
        requested_profile: None,
        note: "no @default_profile: an unspecified profile leaves @profiles-gated rules active",
    },
    CombinatorCase {
        name: "profile_default_gate",
        combinator: Combinator::ProfileDefaultGate,
        // `@default_profile: strict` (the regex.ebnf shape, which declares `pcre2`): an
        // UNSPECIFIED profile now resolves to the declared default, so the
        // `@profiles: ["relaxed"]`-gated branch is EXCLUDED — the same inputs the permissive
        // case accepts must now REJECT. This is the capability the retired
        // `== "regex" → "pcre2"` name literals closed off from synthetic grammars entirely.
        grammar_body: "@entry: true\n@default_profile: strict\nstart := base | relaxed_only\nbase := \"b\"\n@profiles: [\"relaxed\"]\nrelaxed_only := \"R\"\n",
        inputs: &[("b", true), ("R", false), ("x", false)],
        entry_rule: None,
        requested_profile: None,
        note: "@default_profile makes the declared default the unspecified-profile resolution (contrast with the permissive case)",
    },
    // ── Profile aliases — the grammar-level `@profile_alias:` directive (PROFILE-ALIAS.2) ──────────
    CombinatorCase {
        name: "profile_alias_resolves",
        combinator: Combinator::ProfileAliasResolves,
        // `@profile_alias: { "old": modern }` (the systemverilog.ebnf shape, which declares
        // `2017` → `sv_2017`, …): requesting the ALIAS spelling `old` resolves to the canonical
        // `modern`, so the `@profiles: ["modern"]`-gated branch IS active — `"R"` accepts. This is
        // the capability the retired engine alias tables (`parser_registry.rs` "systemverilog"
        // arm, the global `main.rs` spelling table) closed off from synthetic grammars entirely.
        grammar_body: "@entry: true\n@profile_alias: { \"old\": modern }\nstart := base | modern_only\nbase := \"b\"\n@profiles: [\"modern\"]\nmodern_only := \"R\"\n",
        inputs: &[("b", true), ("R", true), ("x", false)],
        entry_rule: None,
        requested_profile: Some("old"),
        note: "requesting the declared alias spelling resolves to the canonical profile → the gated rule is ACTIVE",
    },
    CombinatorCase {
        name: "profile_alias_unknown_passthrough",
        combinator: Combinator::ProfileAliasUnknownPassthrough,
        // The pass-through contrast on the SAME grammar: an UNDECLARED spelling is not coerced —
        // it passes through unresolved, matches no `@profiles` list, and the gated branch stays
        // EXCLUDED — `"R"` rejects. (The retired regex name-gate's "coerce any explicit value"
        // quirk is exactly what this pins as NOT happening.)
        grammar_body: "@entry: true\n@profile_alias: { \"old\": modern }\nstart := base | modern_only\nbase := \"b\"\n@profiles: [\"modern\"]\nmodern_only := \"R\"\n",
        inputs: &[("b", true), ("R", false), ("x", false)],
        entry_rule: None,
        requested_profile: Some("unknown"),
        note: "an undeclared spelling passes through unresolved → the gated rule stays EXCLUDED (no coercion)",
    },
];

/// A KNOWN interpreter-vs-oracle divergence that is **out of `.6.1` scope**, kept here as a durable,
/// re-checkable repro (no silent caps, [[feedback_always_signoff_decisions]]). Bare *direct*
/// left-recursion `start := start "+" term | term` is NOT structurally LR-eliminated (PGEN's
/// `detect_left_recursive_chain_plan` only matches the wrapper/indirect form — a multi-element sequence
/// alt like `start "+" term` is not a bare rule-ref, so `extract_rule_reference_name` returns `None`).
/// It is instead left to **runtime cycle-breaking** (`RecursionGuard`). On that path the interpreter and
/// the generated parser AGREE on the verdict (both REJECT `"n+n"`, since the guard kills the descent) but
/// DIVERGE on `furthest_position` (measured: interpreter reaches `2`/`4`, the generated parser stays
/// `0`). This is a genuine interpreter-fidelity gap on the runtime-cycle-breaking path — surfaced for the
/// director and a candidate follow-up (a `.6.x` fidelity fix), distinct from the LR-*eliminated* combinator
/// the suite certifies above. Reproduce via the ignored [`measurement`] probe.
pub const DIRECT_LEFT_RECURSION_KNOWN_DIVERGENCE: &str =
    "start := start \"+\" term | term\nterm := \"n\"\n";

/// The outcome of comparing the interpreter and the oracle on ONE `(grammar, input)` pair.
#[derive(Debug, Clone)]
pub struct SampleOutcome {
    /// The input string.
    pub input: String,
    /// The interpreter's accept/reject verdict (`None` = interpreter setup/plumbing failed).
    pub interp_accepted: Option<bool>,
    /// The oracle's accept/reject verdict (`None` = oracle plumbing failed).
    pub oracle_accepted: Option<bool>,
    /// `true` iff verdict + `furthest_position` + typed AST were byte-identical.
    pub agreed: bool,
    /// `true` iff the interpreter verdict matched the independent `expected_accept` anchor.
    pub anchor_ok: bool,
    /// A human-readable divergence detail on disagreement, else `None`.
    pub divergence: Option<String>,
    /// The interpreter's serialized typed AST (`None` when it produced none).
    ///
    /// `GENERATED-LINT-CORRECTNESS.2` added this so a case can prove it is
    /// **discriminating** and not merely agreeing: two cases that differ only in a
    /// steering directive (the `assoc_left` / `assoc_right` pair) must produce
    /// DIFFERENT ASTs on the same input, or a fold that silently inverted the
    /// directive would pass the suite unnoticed.
    pub interp_ast: Option<String>,
}

/// The result of running the differential over one combinator's isolating grammar.
#[derive(Debug, Clone)]
pub struct CombinatorCaseReport {
    /// The case name.
    pub name: String,
    /// The combinator isolated.
    pub combinator: Combinator,
    /// A setup failure (grammar write / oracle tool missing) that stopped the run entirely.
    pub load_error: Option<String>,
    /// One entry per input.
    pub samples: Vec<SampleOutcome>,
}

impl CombinatorCaseReport {
    /// A case is clean iff it loaded, compared ≥1 sample, every sample was byte-identical
    /// interpreter-vs-oracle, AND every independent anchor held.
    pub fn is_clean(&self) -> bool {
        self.load_error.is_none()
            && !self.samples.is_empty()
            && self.samples.iter().all(|s| s.agreed && s.anchor_ok)
    }

    /// A one-line human summary for the scouting report.
    pub fn summary_line(&self) -> String {
        if let Some(err) = &self.load_error {
            return format!("{:<26} LOAD-ERROR  {}", self.name, err);
        }
        let diverged = self.samples.iter().filter(|s| !s.agreed).count();
        let anchor_miss = self.samples.iter().filter(|s| !s.anchor_ok).count();
        let verdict = if self.is_clean() { "CLEAN " } else { "FAIL  " };
        let first = self
            .samples
            .iter()
            .find(|s| !s.agreed || !s.anchor_ok)
            .and_then(|s| s.divergence.clone())
            .map(|d| format!("  first: {d}"))
            .unwrap_or_default();
        format!(
            "{:<26} {} samples={} diverge={} anchor_miss={}{}",
            self.name,
            verdict,
            self.samples.len(),
            diverged,
            anchor_miss,
            first,
        )
    }
}

/// Compact one-line AST-diff note (mirrors the `.5` reporter): the first byte offset at which the two
/// serialized ASTs differ, with a small window of context from each side. `pub(crate)` so the sibling
/// `.6.2` semantic suite reuses the identical comparator (one source of truth for "byte-identical").
pub(crate) fn ast_diff_note(
    interp: Option<&serde_json::Value>,
    oracle: Option<&serde_json::Value>,
) -> String {
    match (interp, oracle) {
        (Some(i), Some(o)) => {
            let is = i.to_string();
            let os = o.to_string();
            if is == os {
                return "ASTs are byte-identical".to_string();
            }
            let at = is
                .bytes()
                .zip(os.bytes())
                .position(|(a, b)| a != b)
                .unwrap_or_else(|| is.len().min(os.len()));
            format!(
                "AST differs at byte {at}: interp=…{}… oracle=…{}…",
                context_from(&is, at),
                context_from(&os, at),
            )
        }
        (None, Some(_)) => "interpreter produced no AST but oracle did".to_string(),
        (Some(_), None) => "oracle produced no AST but interpreter did".to_string(),
        (None, None) => "neither side produced an AST".to_string(),
    }
}

fn context_from(s: &str, at: usize) -> &str {
    let start = at.saturating_sub(16);
    let end = (at + 24).min(s.len());
    // Snap to char boundaries so the slice is always valid UTF-8.
    let start = (start..=at).find(|&i| s.is_char_boundary(i)).unwrap_or(at);
    let end = (end..=s.len()).find(|&i| s.is_char_boundary(i)).unwrap_or(s.len());
    &s[start..end]
}

/// Run the interpreter and the compile-and-run oracle over one combinator case, collecting divergences.
/// Never panics — a plumbing failure becomes a `load_error` (setup) or a recorded divergence (per input).
///
/// `grammars_dir` is where the synthetic `.ebnf` is written; `opts` supplies the shared oracle workdir
/// (so the `pgen` dependency is compiled once across the whole suite).
#[cfg(feature = "ebnf_dual_run")]
pub fn run_combinator_case(
    case: &CombinatorCase,
    grammars_dir: &Path,
    opts: &CompileAndParseOptions,
) -> CombinatorCaseReport {
    use crate::parse_harness_interpreter::{InterpretOptions, interpret_parse};

    let mut report = CombinatorCaseReport {
        name: case.name.to_string(),
        combinator: case.combinator,
        load_error: None,
        samples: Vec::new(),
    };

    let grammar_path = grammars_dir.join(format!("{}.ebnf", case.name));
    if let Err(e) = std::fs::write(&grammar_path, case.grammar_body) {
        report.load_error = Some(format!("could not write synthetic grammar: {e}"));
        return report;
    }

    // The case's entry rule and requested profile are applied IDENTICALLY to both sides (so the
    // differential stays valid): the interpreter via `InterpretOptions::{entry_rule, profile}`, the
    // oracle via `CompileAndParseOptions::{entry_rule, requested_profile}` (`parse_full_from` /
    // the probe's `set_grammar_profile`). `None` = each grammar's canonical entry / constructor
    // profile posture.
    let interp_opts = InterpretOptions {
        entry_rule: case.entry_rule.map(str::to_string),
        profile: case.requested_profile.map(str::to_string),
    };
    let case_opts = CompileAndParseOptions {
        entry_rule: case.entry_rule.map(str::to_string),
        requested_profile: case.requested_profile.map(str::to_string),
        ..opts.clone()
    };

    for (input, expected_accept) in case.inputs {
        let interp = interpret_parse(&grammar_path, input, &interp_opts);
        let oracle = compile_and_parse(&grammar_path, input, &case_opts);

        let sample = compare(input, *expected_accept, interp, oracle);
        report.samples.push(sample);
    }

    report
}

/// Compare one interpreter outcome against one oracle outcome + the independent anchor.
/// `pub(crate)` so the sibling `.6.2` semantic suite reuses the identical comparator.
#[cfg(feature = "ebnf_dual_run")]
pub(crate) fn compare(
    input: &str,
    expected_accept: bool,
    interp: Result<ParseOutcome, crate::parse_harness_interpreter::InterpretError>,
    oracle: Result<ParseOutcome, crate::parse_harness::HarnessError>,
) -> SampleOutcome {
    match (interp, oracle) {
        (Ok(i), Ok(o)) => {
            let verdict_agree = i.accepted == o.accepted;
            let furthest_agree = i.furthest_position == o.furthest_position;
            let ast_agree = i.ast_json == o.ast_json;
            let agreed = verdict_agree && furthest_agree && ast_agree;
            let anchor_ok = i.accepted == expected_accept;

            let divergence = if agreed {
                None
            } else if !verdict_agree {
                Some(format!(
                    "verdict: interp={} oracle={}",
                    i.accepted, o.accepted
                ))
            } else if !furthest_agree {
                Some(format!(
                    "furthest_position: interp={} oracle={}",
                    i.furthest_position, o.furthest_position
                ))
            } else {
                Some(ast_diff_note(i.ast_json.as_ref(), o.ast_json.as_ref()))
            };

            let interp_ast = i.ast_json.as_ref().map(|v| v.to_string());
            SampleOutcome {
                input: input.to_string(),
                interp_accepted: Some(i.accepted),
                oracle_accepted: Some(o.accepted),
                agreed,
                anchor_ok,
                divergence,
                interp_ast,
            }
        }
        (Err(ie), Ok(o)) => SampleOutcome {
            input: input.to_string(),
            interp_accepted: None,
            oracle_accepted: Some(o.accepted),
            agreed: false,
            anchor_ok: false,
            divergence: Some(format!("interpreter plumbing failed: {ie}")),
            interp_ast: None,
        },
        (Ok(i), Err(oe)) => SampleOutcome {
            input: input.to_string(),
            interp_accepted: Some(i.accepted),
            oracle_accepted: None,
            agreed: false,
            anchor_ok: i.accepted == expected_accept,
            divergence: Some(format!("oracle plumbing failed: {oe}")),
            interp_ast: i.ast_json.as_ref().map(|v| v.to_string()),
        },
        (Err(ie), Err(oe)) => SampleOutcome {
            input: input.to_string(),
            interp_accepted: None,
            oracle_accepted: None,
            agreed: false,
            anchor_ok: false,
            divergence: Some(format!("both sides failed: interp={ie}; oracle={oe}")),
            interp_ast: None,
        },
    }
}

/// Run every [`COMBINATOR_CASES`] entry through [`run_combinator_case`] in a shared oracle workdir (so the
/// `pgen` dependency is compiled exactly once). `ast_pipeline_bin` is the codegen binary the oracle uses
/// (must be built with `--features "generated_parsers ebnf_dual_run"`); `workdir` is the reused oracle
/// scratch dir. Returns one report per case, in table order.
#[cfg(feature = "ebnf_dual_run")]
pub fn evaluate_all_combinator_cases(
    ast_pipeline_bin: &Path,
    workdir: &Path,
) -> std::io::Result<Vec<CombinatorCaseReport>> {
    let grammars_dir = workdir.join("grammars");
    std::fs::create_dir_all(&grammars_dir)?;

    let opts = CompileAndParseOptions {
        ast_pipeline_bin: Some(ast_pipeline_bin.to_path_buf()),
        workdir: Some(workdir.to_path_buf()),
        keep_workdir: true,
        ..Default::default()
    };

    Ok(COMBINATOR_CASES
        .iter()
        .map(|case| run_combinator_case(case, &grammars_dir, &opts))
        .collect())
}

/// The default oracle codegen binary path (`<pgen>/target/debug/ast_pipeline`), matching
/// [`CompileAndParseOptions`]'s own default.
pub fn default_ast_pipeline_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/debug/ast_pipeline")
}

/// The worker-thread stack the differential runs on. The in-process interpreter (approach 1) is a
/// recursive-descent evaluator with a *logical* recursion guard; on a left-recursive / nullable-loop
/// grammar it can nest deep enough to blow the small default test-thread stack (~2 MiB) and abort the
/// process **before** that logical guard fires — the "bound the *real* stack, not just the logical
/// depth" discipline ([[feedback_recursion_ceiling_must_bound_the_real_stack]]). The measurement caught
/// exactly this on the synthetic `left_recursion` case, so — like the `.5` gate — the suite runs on a
/// 512 MiB worker. (The compile-and-run oracle already runs in its own subprocess, so only the
/// interpreter side needs the bigger stack.)
pub const LARGE_STACK_BYTES: usize = 512 * 1024 * 1024;

/// Run `f` on a worker thread with [`LARGE_STACK_BYTES`] of stack and return its result.
pub fn run_on_large_stack<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(LARGE_STACK_BYTES)
        .spawn(f)
        .expect("spawn large-stack worker")
        .join()
        .expect("large-stack worker panicked")
}

/// [`evaluate_all_combinator_cases`] run on a [`LARGE_STACK_BYTES`] worker (owns its inputs). This is the
/// entry the gate + measurement use — see [`LARGE_STACK_BYTES`] for why the bigger stack is required.
#[cfg(feature = "ebnf_dual_run")]
pub fn evaluate_all_combinator_cases_on_large_stack(
    ast_pipeline_bin: PathBuf,
    workdir: PathBuf,
) -> std::io::Result<Vec<CombinatorCaseReport>> {
    run_on_large_stack(move || evaluate_all_combinator_cases(&ast_pipeline_bin, &workdir))
}

#[cfg(all(test, feature = "ebnf_dual_run"))]
mod gate {
    use super::*;

    /// The shared oracle workdir for the whole suite (one `pgen` compile).
    fn suite_workdir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/parse_harness_combinator")
    }

    /// Skip (with a clear message) if the oracle codegen binary is not built — the differential needs it.
    fn oracle_bin_or_skip() -> Option<PathBuf> {
        let bin = default_ast_pipeline_bin();
        if bin.is_file() {
            Some(bin)
        } else {
            eprintln!(
                "skipping PARSE-HARNESS.6.1 combinator suite: {} not built (needs `cargo build \
                 --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline`)",
                bin.display()
            );
            None
        }
    }

    /// THE certification (single oracle test — one shared `compile_and_parse` workdir, so it must not race
    /// a second concurrent oracle test): every structural combinator's interpreter dispatch is
    /// **byte-identical** to the compile-and-run oracle (verdict + `furthest_position` + typed AST) across
    /// its curated inputs, AND every independent spec-reasoned anchor holds. It also folds in the A2.2/A2.3
    /// **branch_policy discrimination proof** (front-loads `.8`): on the SAME `a | ab` grammar,
    /// `longest_match` ACCEPTS "ab" (picks the longer alt) while `ordered` REJECTS it (picks the first alt,
    /// leaves "b") — precisely the fact that makes `FixedTerminalPrefix` shadowing UNSOUND under PGEN's
    /// backtracking engine ([[project_earlier_always_matches_unsound_backtracking]]), proven here on BOTH
    /// the interpreter and the real generated parser, in agreement.
    #[test]
    fn every_structural_combinator_is_byte_identical() {
        let Some(bin) = oracle_bin_or_skip() else { return };
        let workdir = suite_workdir();

        let reports = evaluate_all_combinator_cases_on_large_stack(bin, workdir)
            .expect("combinator suite plumbing must succeed");

        let mut failures = Vec::new();
        for report in &reports {
            eprintln!("{}", report.summary_line());
            if !report.is_clean() {
                for s in &report.samples {
                    if !s.agreed || !s.anchor_ok {
                        failures.push(format!(
                            "  {}/{:?}: agreed={} anchor_ok={} {}",
                            report.name,
                            s.input,
                            s.agreed,
                            s.anchor_ok,
                            s.divergence.clone().unwrap_or_default(),
                        ));
                    }
                }
            }
        }

        assert!(
            failures.is_empty(),
            "PARSE-HARNESS.6.1 combinator divergences (interpreter != compile-and-run oracle, or anchor \
             miss):\n{}",
            failures.join("\n")
        );
        assert_eq!(
            reports.len(),
            COMBINATOR_CASES.len(),
            "every combinator case must produce a report"
        );

        // ── The A2.2/A2.3 branch_policy discrimination proof (reuses the reports above) ──────────────
        let ab_verdict = |c: Combinator| -> bool {
            let report = reports
                .iter()
                .find(|r| r.combinator == c)
                .unwrap_or_else(|| panic!("report for {c:?} present"));
            report
                .samples
                .iter()
                .find(|s| s.input == "ab")
                .and_then(|s| s.interp_accepted)
                .expect("\"ab\" sample present")
        };
        assert!(
            ab_verdict(Combinator::ChoiceLongestMatchDefault),
            "longest_match MUST accept \"ab\" (picks the longer alt) — the A2.3 soundness fact"
        );
        assert!(
            !ab_verdict(Combinator::ChoiceOrdered),
            "ordered MUST reject \"ab\" (picks the first alt, leaves \"b\") — the A2.3 contrast"
        );

        // ── The associativity DISCRIMINATION proof (GENERATED-LINT-CORRECTNESS.2) ────────────────────
        // The three `assoc_*` cases share one grammar shape whose two alts TIE on consumed length and
        // priority, so only the tie-break can pick a winner. Agreement alone would not prove the rows
        // see anything: a fold that inverted or dropped the directive could still make both sides agree
        // (they compile from the same generator). These two assertions are what make the rows
        // load-bearing — one on the VERDICT axis, one on the AST axis.
        let sample_of = |c: Combinator, input: &str| -> SampleOutcome {
            reports
                .iter()
                .find(|r| r.combinator == c)
                .unwrap_or_else(|| panic!("report for {c:?} present"))
                .samples
                .iter()
                .find(|s| s.input == input)
                .unwrap_or_else(|| panic!("{input:?} sample present for {c:?}"))
                .clone()
        };
        let left = sample_of(Combinator::AssociativityLeft, "xy");
        let right = sample_of(Combinator::AssociativityRight, "xy");
        let nonassoc = sample_of(Combinator::AssociativityNonAssoc, "xy");

        assert_eq!(
            (left.interp_accepted, right.interp_accepted, nonassoc.interp_accepted),
            (Some(true), Some(true), Some(false)),
            "a TIE must be REAL: left/right resolve it and accept \"xy\", nonassoc fails the whole choice"
        );
        assert_ne!(
            left.interp_ast, right.interp_ast,
            "left and right must select DIFFERENT alts on a tie — identical ASTs would mean the \
             `assoc_*` rows cannot see the tie-break direction at all (a vacuous pass)"
        );
    }

    /// No silent gap: every [`Combinator::ALL`] variant is exercised by ≥1 case, and every case name is
    /// unique (so a case cannot be silently shadowed / dropped).
    #[test]
    fn combinator_coverage_is_complete() {
        for &c in Combinator::ALL {
            assert!(
                COMBINATOR_CASES.iter().any(|k| k.combinator == c),
                "combinator {c:?} has no isolating case — coverage gap"
            );
        }
        let mut names: Vec<&str> = COMBINATOR_CASES.iter().map(|k| k.name).collect();
        names.sort_unstable();
        let before = names.len();
        names.dedup();
        assert_eq!(before, names.len(), "duplicate combinator case name(s)");
    }
}

#[cfg(all(test, feature = "ebnf_dual_run"))]
mod measurement {
    use super::*;

    /// Scouting (`--ignored`): print the full per-case, per-input interpreter-vs-oracle map — used to
    /// establish the true verdicts before locking the anchors (the measure-then-lock discipline). Run:
    /// `cargo test --features "generated_parsers ebnf_dual_run" --lib
    ///  parse_harness_combinator_suite::measurement -- --ignored --nocapture`.
    #[test]
    #[ignore = "scouting probe: prints the per-combinator differential map; run with --ignored --nocapture"]
    fn measure_combinator_suite() {
        let bin = default_ast_pipeline_bin();
        if !bin.is_file() {
            eprintln!("ast_pipeline not built at {} — cannot measure", bin.display());
            return;
        }
        let workdir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/parse_harness_combinator");
        let reports = evaluate_all_combinator_cases_on_large_stack(bin, workdir).expect("plumbing");

        eprintln!("\n=== PARSE-HARNESS.6.1 combinator differential map ===");
        for r in &reports {
            eprintln!("{}", r.summary_line());
            for s in &r.samples {
                eprintln!(
                    "    {:<10} interp={:?} oracle={:?} agreed={} anchor_ok={}{}",
                    format!("{:?}", s.input),
                    s.interp_accepted,
                    s.oracle_accepted,
                    s.agreed,
                    s.anchor_ok,
                    s.divergence.as_ref().map(|d| format!("  [{d}]")).unwrap_or_default(),
                );
            }
        }
        let clean = reports.iter().filter(|r| r.is_clean()).count();
        eprintln!("\n{clean}/{} combinator cases CLEAN", reports.len());
    }

    /// Scouting (`--ignored`): re-measure the KNOWN direct-left-recursion divergence
    /// ([`DIRECT_LEFT_RECURSION_KNOWN_DIVERGENCE`]) — the durable, re-runnable repro of the
    /// runtime-cycle-breaking `furthest_position` gap (interpreter agrees on the verdict but diverges on
    /// `furthest_position`). Kept `--ignored` because it documents an out-of-`.6.1`-scope finding, not a
    /// gate. Run: `cargo test --features "generated_parsers ebnf_dual_run" --lib
    ///  parse_harness_combinator_suite::measurement::measure_direct_left_recursion_known_divergence
    ///  -- --ignored --nocapture`.
    #[test]
    #[ignore = "documents an out-of-scope KNOWN divergence (direct LR runtime cycle-breaking); run with --ignored --nocapture"]
    fn measure_direct_left_recursion_known_divergence() {
        use crate::parse_harness::CompileAndParseOptions;
        use crate::parse_harness_interpreter::{InterpretOptions, interpret_parse};

        let bin = default_ast_pipeline_bin();
        if !bin.is_file() {
            eprintln!("ast_pipeline not built at {} — cannot measure", bin.display());
            return;
        }
        let workdir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/parse_harness_combinator");
        let grammars_dir = workdir.join("grammars");
        std::fs::create_dir_all(&grammars_dir).expect("create grammars dir");
        let grammar_path = grammars_dir.join("direct_left_recursion.ebnf");
        std::fs::write(&grammar_path, DIRECT_LEFT_RECURSION_KNOWN_DIVERGENCE).expect("write grammar");

        let opts = CompileAndParseOptions {
            ast_pipeline_bin: Some(bin),
            workdir: Some(workdir.clone()),
            keep_workdir: true,
            ..Default::default()
        };

        eprintln!("\n=== direct left-recursion (RUNTIME cycle-breaking, NOT LR-eliminated) — KNOWN divergence ===");
        let inputs = ["n", "n+n", "n+n+n", "n+", "+n"];
        run_on_large_stack(move || {
            for input in inputs {
                let interp = interpret_parse(&grammar_path, input, &InterpretOptions::default());
                let oracle = compile_and_parse(&grammar_path, input, &opts);
                match (interp, oracle) {
                    (Ok(i), Ok(o)) => eprintln!(
                        "    {:<8} interp(accepted={}, furthest={}) oracle(accepted={}, furthest={}) verdict_agree={} furthest_agree={}",
                        format!("{input:?}"),
                        i.accepted, i.furthest_position, o.accepted, o.furthest_position,
                        i.accepted == o.accepted, i.furthest_position == o.furthest_position,
                    ),
                    (i, o) => eprintln!("    {input:?}: plumbing interp={i:?} oracle={o:?}"),
                }
            }
        });
    }
}
