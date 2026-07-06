//! PARSE-HARNESS.6.2 — the semantic-directive orchestration isolating suite.
//!
//! # What this is
//!
//! The structural sibling (`PARSE-HARNESS.6.1`, [`crate::parse_harness_combinator_suite`]) proves the
//! grammar-AST interpreter byte-identical to the compile-and-run oracle **per structural combinator**.
//! This module is the second half of the `.6` per-construct coverage: the **semantic-directive
//! orchestration** surface — everything that gates or mutates a parse *outcome* through the semantic
//! store — exercised per construct on small isolating grammars:
//!
//! - `@predicate` gates in every phase (`pre` / `branch` / `post`) that CHANGE the verdict (hit AND miss);
//! - `@emit_fact` + the builtin query vocabulary (`has_fact`, `lacks_fact`, `fact_attribute_equals`,
//!   `fact_count_at_least`, `has_fact_in_current_scope`, `current_scope_is`);
//! - the scope tree (`@open_scope` / `@close_scope`) with verdict-observable scope-local queries;
//! - store rollback under speculation (a losing tournament branch's emissions must not leak — the C3-B
//!   discipline; a failed quantifier iteration's emissions must not leak);
//! - `$reference`-against-content resolution (positional `$N.path`, named/dotted, `.len`, `view: raw`
//!   vs `view: shaped`) in both `@emit_fact` payloads and `@predicate` args;
//! - branch-start inline actions (INLINE-ACTIONS.2 — the WINNING branch's `@emit_fact` fires, the
//!   loser's does not);
//! - `@export_to_library` / `@import_from_library` orchestration (the no-library-configured no-op
//!   parity — the compile-and-run oracle sets no library dirs, so both sides must skip identically);
//! - memoization × store composition (the generated parser's transaction-wraps-memo design re-evaluates
//!   a rule's own gates on every memo hit; since MEMO-STORE-SOUNDNESS.2 the memo is TAINT-GATED with
//!   write-epoch VALIDATION — a store-consulting body's outcome is epoch-stamped and replayable only
//!   while the store is unchanged, so same-position retries after a store change honestly re-parse —
//!   all compositions pinned differentially).
//!
//! For every `(grammar, input)` pair it runs **both** the interpreter
//! ([`interpret_parse`](crate::parse_harness_interpreter::interpret_parse)) and the compile-and-run
//! harness ([`compile_and_parse`] — the real codegen + real runtime, authoritative *by construction*)
//! and asserts byte-identical verdict + `furthest_position` + typed AST, exactly as `.6.1` does (the
//! comparator is shared — one source of truth for "byte-identical").
//!
//! # Report-first, never-panic; deterministic by construction
//!
//! Same architecture as `.6.1`: [`run_semantic_case`] / [`evaluate_all_semantic_cases`] collect
//! divergences into reports (the scouting use); the gate tests assert every report `is_clean()` (the
//! enforcement use). The corpus is a fixed curated input set — no randomness, so determinism is by
//! construction.
//!
//! # Honest scope (`PARSE-HARNESS.md` §21.3)
//!
//! - **Bootstrap facts** (`push_fact_record`, the cross-file veer surface) are out of scope: the
//!   harness `ParseOutcome` API has no bootstrap-facts input on either side.
//! - **Real library I/O** is out of scope: the compile-and-run throwaway main configures no
//!   `library_in_dir`/`library_out_dir`, so the generated helpers return early; the suite pins that
//!   no-op parity (real I/O stays proven by the registry-path SV gates).
//! - **Coverage-delta replay** is a registry/cert surface the interpreter does not implement.
//! - **`@define_predicate` composition** beyond the builtin vocabulary is not exercised (no shipped
//!   grammar uses it); a follow-up case is cheap if it becomes load-bearing.

use std::path::{Path, PathBuf};

use crate::parse_harness::{CompileAndParseOptions, compile_and_parse};
#[cfg(feature = "ebnf_dual_run")]
use crate::parse_harness_combinator_suite::compare;
use crate::parse_harness_combinator_suite::{SampleOutcome, run_on_large_stack};
#[cfg(test)]
use crate::parse_harness_combinator_suite::default_ast_pipeline_bin;

/// The semantic-orchestration construct an isolating grammar exercises. One variant per construct the
/// `.6.2` interpreter orchestration mirror dispatches; the completeness gate test asserts every variant
/// is present in [`SEMANTIC_CASES`], so a construct cannot be silently unmeasured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticConstruct {
    /// A `phase: post` `has_fact` gate — the canonical declare-before-use store gate (hit AND miss).
    PostGate,
    /// A `phase: pre` gate — blocks rule entry before the body runs.
    PreGate,
    /// A `phase: branch` inline predicate — branch-LOCAL: it gates ONLY the alternative it is
    /// attached to (`branch_predicates_for_rule_branch`). HISTORY: until `BRANCH-PREDICATE-LOCALITY.2`
    /// (2026-07-06, session #48) the registry fn `branch_predicates_for_rule` flat-mapped every branch
    /// bucket, FLATTENING an inline branch predicate rule-wide (tool-established, session #47,
    /// scratch-slot trace; the SV `net_declaration` comment documented the workaround). That was a
    /// day-one bug contradicting its introducing commit's (`43bbc43c`) "candidate branch only"
    /// semantics; this case now pins the restored branch-LOCAL semantics differentially.
    BranchGate,
    /// The branch-LOCAL gating idiom (the SV `.b.6.2.2` pattern): each alternative delegates to a
    /// helper rule carrying a `phase: post` gate, so the store flips WHICH branch wins — AST-changing
    /// on the same input.
    BranchSelectionViaGate,
    /// `fact_attribute_equals` — the attribute-refined query (hit + attribute-mismatch miss).
    AttributeGate,
    /// `lacks_fact` — the negative query (accept-when-absent, reject-when-present).
    LacksGate,
    /// `fact_count_at_least` — the counting query (the SV wildcard-import pattern).
    CountGate,
    /// `@open_scope`/`@close_scope` + `has_fact_in_current_scope` — a fact emitted inside a closed
    /// scope is invisible to a same-depth query outside it (verdict-observable scope tree).
    ScopeVisibility,
    /// `current_scope_is` — the open-scope kind/name is verdict-observable at rule entry.
    ScopeIs,
    /// C3-B: a successful-but-losing tournament branch's nested `@emit_fact` must NOT persist; the
    /// winning branch's must.
    RollbackLoserBranch,
    /// The zero-length-guard × store composition: a zero-length rule success inside a `*` loop fires
    /// its rule-level effects even though the iteration is structurally discarded.
    ZeroLengthEmit,
    /// Named `$ref` resolution over RAW (un-shaped) rule content — the recursive named-descendant
    /// tree walk (`find_semantic_named_descendant`), on rules WITHOUT a `->`.
    RefRawNamedWalk,
    /// Positional-`$N` resolution in directive payloads — plain `$N`, dotted `$N.name`, and indexed
    /// `$N[M]` — over RAW rule content. HISTORY: until `POSITIONAL-PAYLOAD-REFS.2` (2026-07-06,
    /// session #50) the annotation compiler STRIPPED the `$` sigil (`name: $2` froze as
    /// `RuleReference("2")`), the resolver routed the digit-headed text to the NAMED path, whose
    /// lexer rejects a digit head — positional payload refs could NEVER resolve (tool-established,
    /// session #47; pinned then as the `sem_ref_positional_unresolvable` hard-error parity case).
    /// The fix preserves the sigil for digit-headed refs in the ONE shared payload parser
    /// (`parse_rule_reference`), so the deliberately-built `resolve_positional_semantic_reference`
    /// machinery (SV-EXH-PROOF.3.3.4.a.2) is finally reachable; this case pins the WORKING
    /// semantics differentially.
    RefPositional,
    /// The residual positional hard-error parity pin (the successor of the pre-fix
    /// `sem_ref_positional_unresolvable` case): a dotted segment that walks INTO a
    /// terminal-content element (`$3.word` where position 3 is the literal `"]"` — a `Terminal`
    /// node has no children and the named-descendant walk has nothing to match) still fails
    /// resolution → the directive hard-errors → the rule fails, identically on both
    /// implementations. NOTE (tool-established while re-anchoring, session #50): a positional
    /// element that binds a RULE wraps that rule node in `Alternative` content, and the named walk
    /// SELF-MATCHES the wrapped node's rule name — so `$2.word` on `use := "[" word "]"` RESOLVES
    /// (position 2 is the wrapped `word` itself); only a walk into literal/terminal content is
    /// genuinely dead.
    RefPositionalDeepUnresolvable,
    /// Named/dotted resolution with `view: shaped` against the `->` Json (SEMREF-SHAPED).
    RefShaped,
    /// The `.len` suffix on a resolved reference.
    RefLen,
    /// INLINE-ACTIONS.2: a branch-START `@emit_fact` fires for the WINNING branch only.
    BranchStartEmit,
    /// `@emit_fact` attributes resolved from `$ref`s + `fact_attribute_equals` over them.
    EmitAttributesFromRefs,
    /// `@export_to_library`/`@import_from_library` with no configured library dirs — no-op parity.
    LibraryNoop,
    /// Transaction-wraps-memo: a gated rule re-tried at the same position after a store change must
    /// re-evaluate its own gates fresh (the memo caches the BODY, never the rule's gates).
    MemoGateRetry,
    /// The split-memo failure cache keyed on `(rule, position)` only: an UNANNOTATED wrapper rule over
    /// a store-gated rule fails store-dependently; a same-position retry after a store change must
    /// re-parse fresh. HISTORY: until `MEMO-STORE-SOUNDNESS.2` the failure cache was store-blind and
    /// replayed the STALE failure (valid input rejected — the original pin); the taint gate now
    /// epoch-stamps store-tainted failures and evicts them once the store moves, and this case pins
    /// the SOUND behavior differentially.
    MemoWrapperStaleness,
    /// The SUCCESS-side sibling (`MEMO-STORE-SOUNDNESS.1` finding, `.2` fix): a memo-HIT replays the
    /// cached BODY, whose nested tournament choices were made under the store as it was on FIRST
    /// evaluation — gates re-evaluate fresh, but body CONTENT used to be store-frozen (stale
    /// end-position flipping the verdict; stale tree at equal length). The taint gate epoch-stamps
    /// store-tainted successes too (evicted once the store moves); these cases pin the sound
    /// behavior on both observables.
    MemoSuccessStaleness,
    /// Quoted (String) name args match `$ref`-emitted (Identifier-coerced) fact/scope names
    /// TEXTUALLY. HISTORY: until `FACT-NAME-MATCHING.2` (2026-07-06, session #50) fact-NAME
    /// matching was variant-STRICT (the index keyed on the raw `SemanticRuntimeValue` enum;
    /// `current_scope_is` compared with `==`), so the natural `args: [mode, "special"]` was a
    /// silently dead gate (`has_fact(..., String("special")) → false` WITH the fact present —
    /// the F2 finding) and grammars had to use the unquoted-identifier workaround convention.
    /// Names now normalize to their scalar text ([`FactNameKey`] in the shared index +
    /// `semantic_runtime_values_match` in `current_scope_is`) — the same textual semantics
    /// attribute values always had (`semantic_values_match`); this case pins BOTH unified sites.
    QuotedNameArgTextualMatch,
}

impl SemanticConstruct {
    /// Every construct the suite must cover — the completeness universe.
    pub const ALL: &'static [SemanticConstruct] = &[
        SemanticConstruct::PostGate,
        SemanticConstruct::PreGate,
        SemanticConstruct::BranchGate,
        SemanticConstruct::BranchSelectionViaGate,
        SemanticConstruct::AttributeGate,
        SemanticConstruct::LacksGate,
        SemanticConstruct::CountGate,
        SemanticConstruct::ScopeVisibility,
        SemanticConstruct::ScopeIs,
        SemanticConstruct::RollbackLoserBranch,
        SemanticConstruct::ZeroLengthEmit,
        SemanticConstruct::RefRawNamedWalk,
        SemanticConstruct::RefPositional,
        SemanticConstruct::RefPositionalDeepUnresolvable,
        SemanticConstruct::RefShaped,
        SemanticConstruct::RefLen,
        SemanticConstruct::BranchStartEmit,
        SemanticConstruct::EmitAttributesFromRefs,
        SemanticConstruct::LibraryNoop,
        SemanticConstruct::MemoGateRetry,
        SemanticConstruct::MemoWrapperStaleness,
        SemanticConstruct::MemoSuccessStaleness,
        SemanticConstruct::QuotedNameArgTextualMatch,
    ];
}

/// One isolating grammar + its targeted inputs. Field semantics identical to the `.6.1`
/// `CombinatorCase`: `inputs` carry an INDEPENDENT `expected_accept` anchor reasoned from the
/// directive's documented semantics (NOT the oracle) proving the case is live/discriminating; the
/// load-bearing certification is `interpreter == oracle`.
#[derive(Debug, Clone, Copy)]
pub struct SemanticCase {
    /// A stable, filesystem-safe name (the synthetic `.ebnf` stem).
    pub name: &'static str,
    /// The construct this case isolates.
    pub construct: SemanticConstruct,
    /// The full grammar body (entry rule first).
    pub grammar_body: &'static str,
    /// `(input, expected_accept)` anchors.
    pub inputs: &'static [(&'static str, bool)],
    /// The start symbol, applied identically to both sides; `None` = the canonical entry.
    pub entry_rule: Option<&'static str>,
    /// A short note on what the case proves.
    pub note: &'static str,
}

/// The systematic per-construct table. Every [`SemanticConstruct::ALL`] variant appears at least once.
///
/// Every `grammar_body` was tool-validated to compile through `ast_pipeline --generate-parser` before
/// being locked here, and every `expected_accept` anchor was reasoned from the directive semantics in
/// `semantic_runtime.rs` (the predicate vocabulary at `:2031-2230`, the orchestration skeleton emitted
/// by `ast_based_generator.rs:1661`), then confirmed against the oracle (measure-then-lock).
pub const SEMANTIC_CASES: &[SemanticCase] = &[
    // ── The canonical post gate: declare-before-use ────────────────────────────────────────────────
    SemanticCase {
        name: "sem_post_gate",
        construct: SemanticConstruct::PostGate,
        grammar_body: "@fact_kind: { name: name_decl, attributes: [family], description: \"A declared name.\" }\n\
                       program := decl use\n\
                       @emit_fact: { kind: name_decl, name: $body, family: var }\n\
                       decl := \"decl \" word \";\" -> { body: $2.body }\n\
                       @predicate: { name: has_fact, args: [name_decl, $body], phase: post }\n\
                       use := \"use \" word \";\" -> { body: $2.body }\n\
                       word := /[a-z]+/ -> { body: $1 }\n",
        inputs: &[
            ("decl a;use a;", true),
            ("decl a;use b;", false),
            ("use a;", false),
            ("decl a;", false),
        ],
        entry_rule: None,
        note: "post has_fact gate: use-of-declared ACCEPTs, use-of-undeclared REJECTs (verdict-changing)",
    },
    // ── The pre gate ────────────────────────────────────────────────────────────────────────────────
    SemanticCase {
        name: "sem_pre_gate",
        construct: SemanticConstruct::PreGate,
        grammar_body: "@fact_kind: { name: flag, attributes: [family], description: \"A flag.\" }\n\
                       program := arm? gated\n\
                       @emit_fact: { kind: flag, name: \"on\", family: f }\n\
                       arm := \"arm;\"\n\
                       @predicate: { name: has_fact, args: [flag, \"on\"], phase: pre }\n\
                       gated := \"go\"\n",
        inputs: &[("arm;go", true), ("go", false), ("arm;", false)],
        entry_rule: None,
        note: "pre gate blocks `gated` entry unless `arm;` emitted the flag first",
    },
    // ── The inline branch gate (branch-LOCAL pin — re-anchored by BRANCH-PREDICATE-LOCALITY.2) ────
    SemanticCase {
        name: "sem_branch_gate",
        construct: SemanticConstruct::BranchGate,
        // NOTE the predicate arg is the UNQUOTED identifier `special`: the stored fact name is coerced
        // from the `$body` resolution to `Identifier("special")` (`coerce_semantic_runtime_scalar`),
        // and the fact-index name match is VARIANT-SENSITIVE — a quoted "special" stays
        // `String("special")` and never matches (tool-established, session #47 scratch-slot trace:
        // `has_fact(kind=mode, name=String("special")) → false` with the fact present). SV's grammars
        // use unquoted identifiers in predicate args throughout, for exactly this reason.
        grammar_body: "@fact_kind: { name: mode, attributes: [family], description: \"A mode.\" }\n\
                       program := setmode pick\n\
                       @emit_fact: { kind: mode, name: $body, family: m }\n\
                       setmode := \"mode:\" word \";\" -> { body: $2.body }\n\
                       pick := @predicate: { name: has_fact, args: [mode, special], phase: branch }\n\
                               word \";\" -> { kind: \"special_pick\", w: $1.body }\n\
                             | word \";\" -> { kind: \"normal_pick\", w: $1.body }\n\
                       word := /[a-z]+/ -> { body: $1 }\n",
        inputs: &[
            // The gate passes on its own branch → branch 1 wins the equal-length tie → special_pick.
            ("mode:special;x;", true),
            // The BRANCH-LOCAL pin (re-anchored, BRANCH-PREDICATE-LOCALITY.2): the inline predicate
            // gates ONLY branch 1 — with the gate failing, branch 2 survives and the parse ACCEPTS as
            // "normal_pick". (Pre-fix flattening semantics: the predicate gated BOTH branches and this
            // input REJECTED — the documented old pin, deliberately flipped with the engine fix.)
            ("mode:other;x;", true),
            ("mode:special;x", false),
        ],
        entry_rule: None,
        note: "an INLINE branch-phase predicate is branch-LOCAL (gates only its own alternative; \
               restored by BRANCH-PREDICATE-LOCALITY.2) — the store flips WHICH branch wins, pinned \
               differentially",
    },
    // ── Branch-LOCAL selection via helper post gates (the SV idiom) ────────────────────────────────
    SemanticCase {
        name: "sem_branch_select",
        construct: SemanticConstruct::BranchSelectionViaGate,
        grammar_body: "@fact_kind: { name: mode, attributes: [family], description: \"A mode.\" }\n\
                       program := setmode pick\n\
                       @emit_fact: { kind: mode, name: $body, family: m }\n\
                       setmode := \"mode:\" word \";\" -> { body: $2.body }\n\
                       pick := special_word | normal_word\n\
                       @predicate: { name: has_fact, args: [mode, special], phase: post }\n\
                       special_word := word \";\" -> { kind: \"special_pick\", w: $1.body }\n\
                       normal_word := word \";\" -> { kind: \"normal_pick\", w: $1.body }\n\
                       word := /[a-z]+/ -> { body: $1 }\n",
        inputs: &[
            // Gate passes → both helper branches match the same length → tie keeps branch 1 →
            // `special_pick`.
            ("mode:special;x;", true),
            // Gate rejects `special_word` (branch-LOCAL, on the helper) → `normal_word` wins →
            // `normal_pick`. Same verdict, DIFFERENT AST — the store selects the branch.
            ("mode:other;x;", true),
            ("mode:special;x", false),
        ],
        entry_rule: None,
        note: "branch-LOCAL gating via helper rules with post gates (the SV idiom): the store flips \
               which branch wins — AST-changing on the same accept verdict",
    },
    // ── The attribute-refined query ─────────────────────────────────────────────────────────────────
    SemanticCase {
        name: "sem_attr_gate",
        construct: SemanticConstruct::AttributeGate,
        grammar_body: "@fact_kind: { name: sym, attributes: [family], description: \"A symbol.\" }\n\
                       program := intro use\n\
                       intro := cdecl | fdecl\n\
                       @emit_fact: { kind: sym, name: $body, family: cls }\n\
                       cdecl := \"class \" word \";\" -> { body: $2.body }\n\
                       @emit_fact: { kind: sym, name: $body, family: fn }\n\
                       fdecl := \"func \" word \";\" -> { body: $2.body }\n\
                       @predicate: { name: fact_attribute_equals, args: [sym, $body, family, cls], phase: post }\n\
                       use := \"new \" word \";\" -> { body: $2.body }\n\
                       word := /[a-z]+/ -> { body: $1 }\n",
        inputs: &[
            ("class a;new a;", true),
            ("func a;new a;", false),
            ("class a;new b;", false),
        ],
        entry_rule: None,
        note: "fact_attribute_equals: same-name fact with the WRONG family attribute still rejects",
    },
    // ── The negative query ──────────────────────────────────────────────────────────────────────────
    SemanticCase {
        name: "sem_lacks_gate",
        construct: SemanticConstruct::LacksGate,
        grammar_body: "@fact_kind: { name: taken, attributes: [family], description: \"Taken.\" }\n\
                       program := decl fresh\n\
                       @emit_fact: { kind: taken, name: $body, family: t }\n\
                       decl := \"take \" word \";\" -> { body: $2.body }\n\
                       @predicate: { name: lacks_fact, args: [taken, $body], phase: post }\n\
                       fresh := \"new \" word \";\" -> { body: $2.body }\n\
                       word := /[a-z]+/ -> { body: $1 }\n",
        inputs: &[("take a;new b;", true), ("take a;new a;", false)],
        entry_rule: None,
        note: "lacks_fact: re-taking a taken name rejects; a fresh name accepts",
    },
    // ── The counting query ──────────────────────────────────────────────────────────────────────────
    SemanticCase {
        name: "sem_count_gate",
        construct: SemanticConstruct::CountGate,
        grammar_body: "@fact_kind: { name: item, attributes: [family], description: \"Item.\" }\n\
                       program := decl+ gated\n\
                       @emit_fact: { kind: item, name: $body, family: i }\n\
                       decl := \"item \" word \";\" -> { body: $2.body }\n\
                       @predicate: { name: fact_count_at_least, args: [item, 2], phase: post }\n\
                       gated := \"go\"\n\
                       word := /[a-z]+/ -> { body: $1 }\n",
        inputs: &[
            ("item a;item b;go", true),
            ("item a;go", false),
            ("item a;item b;item c;go", true),
        ],
        entry_rule: None,
        note: "fact_count_at_least(item, 2): fewer than 2 declared items rejects `go`",
    },
    // ── Scope tree: emitted-inside-a-closed-scope is invisible at the outer depth ───────────────────
    SemanticCase {
        name: "sem_scope",
        construct: SemanticConstruct::ScopeVisibility,
        grammar_body: "@fact_kind: { name: localv, attributes: [family], description: \"A block-local name.\" }\n\
                       program := blk use | decl use\n\
                       blk := opener decl closer\n\
                       @open_scope: { kind: block, name: \"b1\" }\n\
                       opener := \"{\"\n\
                       @close_scope: { kind: block }\n\
                       closer := \"}\"\n\
                       @emit_fact: { kind: localv, name: $body, family: l }\n\
                       decl := \"let \" word \";\" -> { body: $2.body }\n\
                       @predicate: { name: has_fact_in_current_scope, args: [localv, $body], phase: post }\n\
                       use := \"use \" word \";\" -> { body: $2.body }\n\
                       word := /[a-z]+/ -> { body: $1 }\n",
        inputs: &[
            ("let a;use a;", true),
            ("{let a;}use a;", false),
            ("let a;use b;", false),
        ],
        entry_rule: None,
        note: "has_fact_in_current_scope: a fact emitted at block depth is invisible to the same query \
               at file depth after `}` closes the scope",
    },
    // ── Scope kind/name observable at rule entry ────────────────────────────────────────────────────
    SemanticCase {
        name: "sem_scope_is",
        construct: SemanticConstruct::ScopeIs,
        grammar_body: "program := opener inner | inner\n\
                       @open_scope: { kind: class, name: \"c\" }\n\
                       opener := \"class{\"\n\
                       @predicate: { name: current_scope_is, args: [class, \"c\"], phase: pre }\n\
                       inner := \"body\"\n",
        inputs: &[("class{body", true), ("body", false)],
        entry_rule: None,
        note: "current_scope_is: `inner` is enterable only inside the class scope `opener` opened",
    },
    // ── FACT-NAME-MATCHING.2: quoted name args match $ref-emitted names TEXTUALLY ──────────────────
    SemanticCase {
        name: "sem_quoted_name_args",
        construct: SemanticConstruct::QuotedNameArgTextualMatch,
        // BOTH unified sites in one grammar: `use`'s post gate queries the `$2`-emitted fact
        // (Identifier("special") via `coerce_semantic_runtime_scalar`) with a QUOTED
        // `"special"` (String); `inner`'s pre gate queries the `$2`-named scope
        // (Identifier via the same coercion) with a QUOTED `"sc"`. Pre-fix both were
        // variant-strict misses (the F2 dead gate); post-fix both match textually.
        grammar_body: "@fact_kind: { name: mode, attributes: [family], description: \"M.\" }\n\
                       program := mk use opener inner\n\
                       @emit_fact: { kind: mode, name: $2, family: m }\n\
                       mk := \"(\" word \")\"\n\
                       @predicate: { name: has_fact, args: [mode, \"special\"], phase: post }\n\
                       use := \"!\"\n\
                       @open_scope: { kind: block, name: $2 }\n\
                       opener := \"{\" word \"}\"\n\
                       @predicate: { name: current_scope_is, args: [block, \"sc\"], phase: pre }\n\
                       inner := \"end\"\n\
                       word := /[a-z]+/\n",
        inputs: &[
            // mk emits mode:Identifier("special"); use's quoted "special" matches textually;
            // opener opens block:Identifier("sc"); inner's quoted "sc" matches textually.
            ("(special)!{sc}end", true),
            // The emitted fact is named "other" → has_fact(mode, "special") false → REJECT.
            ("(other)!{sc}end", false),
            // The scope is named "xx" → current_scope_is(block, "sc") false → REJECT.
            ("(special)!{xx}end", false),
        ],
        entry_rule: None,
        note: "quoted String name args match Identifier-coerced $ref names TEXTUALLY (fact index + \
               current_scope_is) since FACT-NAME-MATCHING.2 — the F2 dead-gate class closed",
    },
    // ── C3-B: loser-branch emissions must not leak; winner's must persist ──────────────────────────
    SemanticCase {
        name: "sem_rollback_loser",
        construct: SemanticConstruct::RollbackLoserBranch,
        grammar_body: "@fact_kind: { name: mark, attributes: [family], description: \"Mark.\" }\n\
                       program := choice check\n\
                       choice := emitter \"y\" | \"e\" \"y\" \"z\" | \"e\" \"x\"\n\
                       @emit_fact: { kind: mark, name: \"m\", family: k }\n\
                       emitter := \"e\"\n\
                       @predicate: { name: lacks_fact, args: [mark, \"m\"], phase: post }\n\
                       check := \"!\"\n",
        inputs: &[
            // Branch 1 (emits) SUCCEEDS with len 2 but branch 2 (no emit) wins at len 3 → C3-B must
            // discard the successful loser's emission → `check` (lacks_fact) ACCEPTs.
            ("eyz!", true),
            // Branch 1 (emits) WINS → the emission persists → `check` REJECTs.
            ("ey!", false),
            // Branch 1 fails mid-branch after `emitter` emitted (needs "y", input has "x") → the
            // branch speculation rolls the emission back → branch 3 wins → `check` ACCEPTs.
            ("ex!", true),
        ],
        entry_rule: None,
        note: "C3-B: only the WINNING branch's semantic delta survives the tournament (successful-loser \
               and failed-branch emissions both roll back)",
    },
    // ── Zero-length-guard × store composition ───────────────────────────────────────────────────────
    SemanticCase {
        name: "sem_zero_len_emit",
        construct: SemanticConstruct::ZeroLengthEmit,
        grammar_body: "@fact_kind: { name: z, attributes: [family], description: \"Z.\" }\n\
                       program := maybe* check\n\
                       @emit_fact: { kind: z, name: \"zz\", family: f }\n\
                       maybe := \"x\"?\n\
                       @predicate: { name: lacks_fact, args: [z, \"zz\"], phase: post }\n\
                       check := \"end\"\n",
        inputs: &[
            // `maybe` matches zero-length (a SUCCESS — its rule-level @emit_fact fires) and the
            // guard then discards the iteration STRUCTURALLY; the emission's fate is exactly what
            // this case pins differentially (anchor set from the oracle, measure-then-lock).
            ("end", false),
            ("xend", false),
        ],
        entry_rule: None,
        note: "zero-length guard × store: a discarded zero-length iteration's rule-level emission — \
               pinned to whatever the generated parser does (measured: the emission persists)",
    },
    // ── $reference resolution: NAMED walk over RAW content ─────────────────────────────────────────
    SemanticCase {
        name: "sem_ref_raw_named",
        construct: SemanticConstruct::RefRawNamedWalk,
        // `mk`/`use`/`word` deliberately carry NO `->`: `$word` resolves through the RAW-tree
        // named-descendant walk (`find_semantic_named_descendant` recursing Sequence/Alternative
        // nodes to the node whose rule_name is `word`), not the shaped-Json path.
        grammar_body: "@fact_kind: { name: pf, attributes: [family], description: \"PF.\" }\n\
                       program := mk use\n\
                       @emit_fact: { kind: pf, name: $word, family: p }\n\
                       mk := \"(\" word \")\"\n\
                       @predicate: { name: has_fact, args: [pf, $word], phase: post }\n\
                       use := \"[\" word \"]\"\n\
                       word := /[a-z]+/\n",
        inputs: &[("(a)[a]", true), ("(a)[b]", false)],
        entry_rule: None,
        note: "named `$word` resolution over RAW (no `->`) content — the recursive named-descendant \
               tree walk, in both the emit payload and the predicate args",
    },
    // ── $reference resolution: WORKING positional $N / $N.name / $N[M] over RAW content ────────────
    SemanticCase {
        name: "sem_ref_positional",
        construct: SemanticConstruct::RefPositional,
        // POSITIONAL-PAYLOAD-REFS.2 re-anchor: the pre-fix `sem_ref_positional_unresolvable` pin
        // (`"(a)[a]"` REJECT — the compiler stripped `$`, resolution always hard-errored) becomes
        // the WORKING positional case. `mk`'s emit uses plain `$2` (position 2 = the wrapped
        // `word`); `use`'s gate uses dotted `$2.word` (position 2 = the wrapped `pair`, then the
        // named-descendant walk to its FIRST `word`); `idx`'s gate uses chained-indexed `$2[0][2]`
        // — the SV-EXH-PROOF.3.3.4.a.2 bracket machinery's FIRST live coverage, exercising BOTH
        // `find_semantic_indexed_child` arms: a positional element wraps its rule node in
        // `Alternative` content (AST-dump-established, session #50), so `[0]` unwraps the wrapper
        // to the `pair` node and `[2]` then picks its 0-based sequence child 2 (the second `word`;
        // child 1 is the `","` literal).
        grammar_body: "@fact_kind: { name: pf, attributes: [family], description: \"PF.\" }\n\
                       program := mk use idx\n\
                       @emit_fact: { kind: pf, name: $2, family: p }\n\
                       mk := \"(\" word \")\"\n\
                       @predicate: { name: has_fact, args: [pf, $2.word], phase: post }\n\
                       use := \"[\" pair \"]\"\n\
                       @predicate: { name: has_fact, args: [pf, $2[0][2]], phase: post }\n\
                       idx := \"{\" pair \"}\"\n\
                       pair := word \",\" word\n\
                       word := /[a-z]+/\n",
        inputs: &[
            // All three resolve to "a": mk emits pf:"a" ($2), use's $2.word finds the FIRST word
            // descendant of `pair` ("a"), idx's $2[0][2] unwraps to `pair` then picks its 0-based
            // child 2 = the SECOND word ("a").
            ("(a)[a,z]{z,a}", true),
            // use's dotted $2.word resolves "b" → has_fact(pf,"b") false → REJECT.
            ("(a)[b,z]{z,a}", false),
            // idx's chained-indexed $2[0][2] resolves "b" → has_fact(pf,"b") false → REJECT.
            ("(a)[a,z]{z,b}", false),
        ],
        entry_rule: None,
        note: "positional payload refs RESOLVE since POSITIONAL-PAYLOAD-REFS.2 (the compiler keeps \
               the `$` for digit-headed refs): plain `$N`, dotted `$N.name`, and chained-indexed \
               `$N[0][M]` (Alternative-unwrap + sequence-index) all live-covered over RAW content",
    },
    // ── $reference resolution: the residual positional hard-error parity pin ───────────────────────
    SemanticCase {
        name: "sem_ref_positional_deep_unresolvable",
        construct: SemanticConstruct::RefPositionalDeepUnresolvable,
        // The successor of the RETIRED pre-fix `sem_ref_positional_unresolvable` case (same grammar
        // shape and inputs; the predicate ref changed `$2.word` → `$3.word`). Pre-fix the stripped
        // `"2"` literal could never even dispatch positionally; post-fix `$2.word` RESOLVES
        // (position 2 is the Alternative-wrapped `word` node and the named walk SELF-matches it —
        // the session #50 AST-dump finding), so the residual hard-error pin moves to `$3.word`:
        // position 3 is the literal `"]"` (Terminal content, no children, nothing named to match)
        // → resolution fails → the predicate hard-errors → `use` fails → REJECT on both sides.
        grammar_body: "@fact_kind: { name: pf, attributes: [family], description: \"PF.\" }\n\
                       program := mk use\n\
                       @emit_fact: { kind: pf, name: $2, family: p }\n\
                       mk := \"(\" word \")\"\n\
                       @predicate: { name: has_fact, args: [pf, $3.word], phase: post }\n\
                       use := \"[\" word \"]\"\n\
                       word := /[a-z]+/\n",
        inputs: &[
            ("(a)[a]", false),
            ("(a)[b]", false),
        ],
        entry_rule: None,
        note: "a positional dotted segment walking INTO a terminal-content (literal) element stays \
               unresolvable → hard-error parity (the successor of the pre-fix strip pin)",
    },
    // ── $reference resolution: named/dotted with view: shaped ───────────────────────────────────────
    SemanticCase {
        name: "sem_ref_shaped",
        construct: SemanticConstruct::RefShaped,
        grammar_body: "@fact_kind: { name: pf, attributes: [family], description: \"PF.\" }\n\
                       program := mk use\n\
                       @emit_fact: { kind: pf, name: $inner.body, family: p }\n\
                       mk := \"(\" word \")\" -> { inner: { body: $2.body } }\n\
                       @predicate: { name: has_fact, args: [pf, $w.body], phase: post, view: shaped }\n\
                       use := \"[\" word \"]\" -> { w: { body: $2.body } }\n\
                       word := /[a-z]+/ -> { body: $1 }\n",
        inputs: &[("(a)[a]", true), ("(a)[b]", false)],
        entry_rule: None,
        note: "SEMREF-SHAPED: dotted `$inner.body` / `$w.body` walk the `->` Json object (view: shaped)",
    },
    // ── $reference resolution: the .len suffix ──────────────────────────────────────────────────────
    SemanticCase {
        name: "sem_ref_len",
        construct: SemanticConstruct::RefLen,
        grammar_body: "@fact_kind: { name: lf, attributes: [family], description: \"LF.\" }\n\
                       program := mk use\n\
                       @emit_fact: { kind: lf, name: $body.len, family: p }\n\
                       mk := \"(\" word \")\" -> { body: $2.body }\n\
                       @predicate: { name: has_fact, args: [lf, $body.len], phase: post }\n\
                       use := \"[\" word \"]\" -> { body: $2.body }\n\
                       word := /[a-z]+/ -> { body: $1 }\n",
        inputs: &[("(ab)[xy]", true), ("(ab)[x]", false)],
        entry_rule: None,
        note: "`.len`: the fact name is the CHARACTER COUNT of the resolved text (len 2 == len 2 accepts; \
               2 vs 1 rejects)",
    },
    // ── INLINE-ACTIONS.2: winning-branch branch-start @emit_fact ────────────────────────────────────
    SemanticCase {
        name: "sem_branch_start_emit",
        construct: SemanticConstruct::BranchStartEmit,
        grammar_body: "@fact_kind: { name: bmark, attributes: [family], description: \"BM.\" }\n\
                       program := tag gated\n\
                       tag := @emit_fact: { kind: bmark, name: \"one\", family: b }\n\
                              \"t1\"\n\
                            | @emit_fact: { kind: bmark, name: \"two\", family: b }\n\
                              \"t2\"\n\
                       @predicate: { name: has_fact, args: [bmark, \"one\"], phase: post }\n\
                       gated := \"!\"\n",
        inputs: &[("t1!", true), ("t2!", false)],
        entry_rule: None,
        note: "INLINE-ACTIONS.2: the WINNING branch's branch-start @emit_fact fires (branch 1 emits \
               \"one\"), the unselected branch's does not (branch 2 would emit \"two\")",
    },
    // ── @emit_fact attributes resolved from $refs ───────────────────────────────────────────────────
    SemanticCase {
        name: "sem_emit_attrs",
        construct: SemanticConstruct::EmitAttributesFromRefs,
        grammar_body: "@fact_kind: { name: typed, attributes: [kindattr], description: \"Typed.\" }\n\
                       program := decl use\n\
                       @emit_fact: { kind: typed, name: $name.body, kindattr: $kind.body }\n\
                       decl := word \":\" word \";\" -> { name: { body: $1.body }, kind: { body: $3.body } }\n\
                       @predicate: { name: fact_attribute_equals, args: [typed, $name.body, kindattr, cls], phase: post, view: shaped }\n\
                       use := \"use \" word \";\" -> { name: { body: $2.body } }\n\
                       word := /[a-z]+/ -> { body: $1 }\n",
        inputs: &[
            ("a:cls;use a;", true),
            ("a:fn;use a;", false),
            ("a:cls;use b;", false),
        ],
        entry_rule: None,
        note: "@emit_fact attributes from $refs: `kindattr` carries the parsed `$kind.body`; the gate \
               matches it against the literal `cls`",
    },
    // ── Library import/export: the no-dirs no-op parity ────────────────────────────────────────────
    SemanticCase {
        name: "sem_library_noop",
        construct: SemanticConstruct::LibraryNoop,
        grammar_body: "@fact_kind: { name: pkg, attributes: [family], exportable: true, description: \"Pkg.\" }\n\
                       program := ex im\n\
                       @export_to_library: { kind: package, name_from: $body }\n\
                       @emit_fact: { kind: pkg, name: $body, family: p }\n\
                       ex := \"pkg \" word \";\" -> { body: $2.body }\n\
                       @import_from_library: { kind: package, name_from: $body }\n\
                       im := \"imp \" word \";\" -> { body: $2.body }\n\
                       word := /[a-z]+/ -> { body: $1 }\n",
        inputs: &[("pkg a;imp b;", true), ("pkg a;", false)],
        entry_rule: None,
        note: "@export_to_library/@import_from_library with NO configured library dirs: both sides \
               must no-op identically (the real-I/O lane is registry-owned, out of harness scope)",
    },
    // ── Transaction-wraps-memo: gates re-evaluate fresh on a memo hit ──────────────────────────────
    SemanticCase {
        name: "sem_memo_gate_retry",
        construct: SemanticConstruct::MemoGateRetry,
        grammar_body: "@fact_kind: { name: g, attributes: [family], description: \"G.\" }\n\
                       program := gated \"?\" | en gated \"!\"\n\
                       @predicate: { name: has_fact, args: [g, \"on\"], phase: post }\n\
                       gated := \"go\"\n\
                       @emit_fact: { kind: g, name: \"on\", family: f }\n\
                       en := \"on\"?\n",
        inputs: &[
            // Branch 1: `gated`'s body parses "go" (memoized) but the post gate MISSes (no fact) →
            // branch fails. Branch 2: `en` matches zero-length AND emits the fact; `gated` at the
            // SAME position memo-HITs the body — and the post gate must re-evaluate FRESH against
            // the changed store → PASS → "!" → ACCEPT. (The gate lives in the rule transaction,
            // which wraps the memo — a rule's own gates are never cached.)
            ("go!", true),
            // Same shape, but branch 2 then fails on "!" vs "?" → the branch rollback also rolls the
            // `en` emission back → REJECT.
            ("go?", false),
            ("ongo!", true),
        ],
        entry_rule: None,
        note: "transaction-wraps-memo: a memo hit on the body still re-evaluates the rule's own post \
               gate against the CURRENT store",
    },
    // ── Split-memo failure cache × store ────────────────────────────────────────────────────────────
    SemanticCase {
        name: "sem_memo_wrapper",
        construct: SemanticConstruct::MemoWrapperStaleness,
        grammar_body: "@fact_kind: { name: g, attributes: [family], description: \"G.\" }\n\
                       program := wrap \"?\" | en wrap \"!\"\n\
                       wrap := gated\n\
                       @predicate: { name: has_fact, args: [g, \"on\"], phase: post }\n\
                       gated := \"go\"\n\
                       @emit_fact: { kind: g, name: \"on\", family: f }\n\
                       en := \"on\"?\n",
        inputs: &[
            // The SOUND re-anchor (MEMO-STORE-SOUNDNESS.2 — deliberate, documented): `wrap`
            // (UNANNOTATED) wraps `gated`. Branch 1: `gated`'s post gate rejects → `wrap`'s BODY
            // fails STORE-TAINTEDLY (the gate evaluation bumped the taint counter inside `wrap`'s
            // body) → the failure is cached EPOCH-STAMPED. Branch 2: `en` emits the fact
            // zero-length (bumping the store write epoch), `wrap` retries at position 0 → the
            // stale stamped failure is EVICTED, the body honestly RE-PARSES, `gated`'s gate now
            // passes → ACCEPT. HISTORY: the pre-fix store-blind failure cache replayed the stale
            // failure and this input was pinned `("go!", false)` (tool-established, session #47) —
            // that stale-REJECT pin flipped to the sound ACCEPT with the `.2` taint gate, both
            // pins recorded.
            ("go!", true),
            // Branch 1 rejects (no fact); branch 2's re-parse now ACCEPTS `wrap` (fact emitted) but
            // then fails on "!" vs "?" — the branch rollback discards the emission → REJECT.
            ("go?", false),
            ("ongo!", true),
        ],
        entry_rule: None,
        note: "taint-gated failure cache (MEMO-STORE-SOUNDNESS.2): a store-tainted body failure is \
               not cached, so a same-position retry after a zero-width store change re-parses fresh \
               and ACCEPTs — the sound composition, pinned differentially",
    },
    // ── Split-memo SUCCESS cache × store (the `.1`-confirmed sibling, promoted probe grammars) ─────
    SemanticCase {
        name: "sem_memo_success_verdict",
        construct: SemanticConstruct::MemoSuccessStaleness,
        grammar_body: "@fact_kind: { name: g, attributes: [family], description: \"G.\" }\n\
                       program := pick \"?\" | en pick \"!\"\n\
                       pick := wide | narrow\n\
                       @predicate: { name: has_fact, args: [g, \"on\"], phase: post }\n\
                       wide := \"gox\"\n\
                       narrow := \"go\"\n\
                       @emit_fact: { kind: g, name: \"on\", family: f }\n\
                       en := \"on\"?\n",
        inputs: &[
            // THE verdict pin (MEMO-STORE-SOUNDNESS.1 probe → .2 sound anchor): branch 1 — `wide`'s
            // gate misses (no fact) so `narrow` wins `pick` store-TAINTEDLY (end 2, cached
            // epoch-stamped); `"?"` misses at byte 2. Branch 2 — `en` emits zero-width (epoch
            // bump), the stale stamped win is EVICTED and `pick` re-parses fresh at position 0:
            // `wide` now passes its gate, wins the tournament at end 3, `"!"` matches → ACCEPT.
            // HISTORY: pre-fix the stale cached narrow win replayed (end 2) and this input
            // REJECTED — the `.1`-measured staleness, deliberately flipped by the taint gate.
            ("gox!", true),
            // Store-free accept control: branch 1 wins directly via `narrow` + `"?"`.
            ("go?", true),
            // Both-reject control: stale or fresh, no branch aligns.
            ("gox?", false),
            // Fresh-position control: `en` consumes "on", `pick` runs at a NEW position — no
            // same-position retry involved.
            ("ongo!", true),
        ],
        entry_rule: None,
        note: "taint-gated SUCCESS cache (MEMO-STORE-SOUNDNESS.2): a store-tainted tournament win is \
               not cached, so the same-position retry after a zero-width emission re-runs the \
               tournament and the LONGER gated branch wins — verdict-observable soundness pin",
    },
    SemanticCase {
        name: "sem_memo_success_ast",
        construct: SemanticConstruct::MemoSuccessStaleness,
        grammar_body: "@fact_kind: { name: g, attributes: [family], description: \"G.\" }\n\
                       program := pick \"?\" | en pick \"!\"\n\
                       pick := special | normal\n\
                       @predicate: { name: has_fact, args: [g, \"on\"], phase: post }\n\
                       special := \"go\" -> { kind: \"special_pick\" }\n\
                       normal := \"go\" -> { kind: \"normal_pick\" }\n\
                       @emit_fact: { kind: g, name: \"on\", family: f }\n\
                       en := \"on\"?\n",
        inputs: &[
            // The tree pin (equal-length branches, shaped `kind` markers): branch 2's fresh re-parse
            // (taint gate — the branch-1 `normal` win was cached epoch-stamped and EVICTED after
            // `en`'s zero-width emission bumped the epoch) lets `special` pass its gate and win the
            // tie → ACCEPT shaped `special_pick`. Pre-fix the stale `normal_pick` tree replayed
            // (same verdict, WRONG tree — the `.1` AST-observable staleness). The verdict anchor
            // alone cannot see the marker; the interpreter-vs-oracle BYTE-IDENTICAL AST comparison
            // is what pins the tree (plus the `--ignored` probe's marker check).
            ("go!", true),
            // Store-free control: accepts as `normal_pick` via branch 1 (gate off, tie → but
            // `special`'s gate rejects, so `normal` wins legitimately).
            ("go?", true),
            // Fresh-position control: `en` consumes "on", the gate passes at the new position →
            // `special_pick` wins the tie.
            ("ongo!", true),
        ],
        entry_rule: None,
        note: "taint-gated SUCCESS cache, tree-observable twin: the same-position retry re-runs the \
               equal-length tournament under the NEW store — byte-identical AST comparison pins the \
               special_pick/normal_pick winner on both implementations",
    },
];

/// The result of running the differential over one semantic case's isolating grammar. Mirrors the
/// `.6.1` report shape (the samples reuse the shared [`SampleOutcome`]).
#[derive(Debug, Clone)]
pub struct SemanticCaseReport {
    /// The case name.
    pub name: String,
    /// The construct isolated.
    pub construct: SemanticConstruct,
    /// A setup failure (grammar write / oracle tool missing) that stopped the run entirely.
    pub load_error: Option<String>,
    /// One entry per input.
    pub samples: Vec<SampleOutcome>,
}

impl SemanticCaseReport {
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
            return format!("{:<24} LOAD-ERROR  {}", self.name, err);
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
            "{:<24} {} samples={} diverge={} anchor_miss={}{}",
            self.name,
            verdict,
            self.samples.len(),
            diverged,
            anchor_miss,
            first,
        )
    }
}

/// Run the interpreter and the compile-and-run oracle over one semantic case, collecting divergences.
/// Never panics — a plumbing failure becomes a `load_error` (setup) or a recorded divergence (per
/// input). Identical driver shape to the `.6.1` [`run_combinator_case`]; the comparator is shared.
#[cfg(feature = "ebnf_dual_run")]
pub fn run_semantic_case(
    case: &SemanticCase,
    grammars_dir: &Path,
    opts: &CompileAndParseOptions,
) -> SemanticCaseReport {
    use crate::parse_harness_interpreter::{InterpretOptions, interpret_parse};

    let mut report = SemanticCaseReport {
        name: case.name.to_string(),
        construct: case.construct,
        load_error: None,
        samples: Vec::new(),
    };

    let grammar_path = grammars_dir.join(format!("{}.ebnf", case.name));
    if let Err(e) = std::fs::write(&grammar_path, case.grammar_body) {
        report.load_error = Some(format!("could not write synthetic grammar: {e}"));
        return report;
    }

    let interp_opts = InterpretOptions {
        entry_rule: case.entry_rule.map(str::to_string),
        ..Default::default()
    };
    let case_opts = CompileAndParseOptions {
        entry_rule: case.entry_rule.map(str::to_string),
        ..opts.clone()
    };

    for (input, expected_accept) in case.inputs {
        let interp = interpret_parse(&grammar_path, input, &interp_opts);
        let oracle = compile_and_parse(&grammar_path, input, &case_opts);
        report.samples.push(compare(input, *expected_accept, interp, oracle));
    }

    report
}

/// Run every [`SEMANTIC_CASES`] entry through [`run_semantic_case`] in a shared oracle workdir (one
/// `pgen` dependency compile for the whole suite). Same contract as the `.6.1` driver.
#[cfg(feature = "ebnf_dual_run")]
pub fn evaluate_all_semantic_cases(
    ast_pipeline_bin: &Path,
    workdir: &Path,
) -> std::io::Result<Vec<SemanticCaseReport>> {
    let grammars_dir = workdir.join("grammars");
    std::fs::create_dir_all(&grammars_dir)?;

    let opts = CompileAndParseOptions {
        ast_pipeline_bin: Some(ast_pipeline_bin.to_path_buf()),
        workdir: Some(workdir.to_path_buf()),
        keep_workdir: true,
        ..Default::default()
    };

    Ok(SEMANTIC_CASES
        .iter()
        .map(|case| run_semantic_case(case, &grammars_dir, &opts))
        .collect())
}

/// [`evaluate_all_semantic_cases`] on a large-stack worker (the shared `.6.1` worker — the interpreter
/// side needs the bigger stack; the oracle runs in its own subprocess).
#[cfg(feature = "ebnf_dual_run")]
pub fn evaluate_all_semantic_cases_on_large_stack(
    ast_pipeline_bin: PathBuf,
    workdir: PathBuf,
) -> std::io::Result<Vec<SemanticCaseReport>> {
    run_on_large_stack(move || evaluate_all_semantic_cases(&ast_pipeline_bin, &workdir))
}

#[cfg(all(test, feature = "ebnf_dual_run"))]
mod gate {
    use super::*;

    /// The shared oracle workdir for the whole suite (one `pgen` compile). Distinct from the `.6.1`
    /// workdir so the two gates never race each other's `compile_and_parse` scratch crates.
    fn suite_workdir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/parse_harness_semantic")
    }

    /// Skip (with a clear message) if the oracle codegen binary is not built.
    fn oracle_bin_or_skip() -> Option<PathBuf> {
        let bin = default_ast_pipeline_bin();
        if bin.is_file() {
            Some(bin)
        } else {
            eprintln!(
                "skipping PARSE-HARNESS.6.2 semantic suite: {} not built (needs `cargo build \
                 --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline`)",
                bin.display()
            );
            None
        }
    }

    /// THE certification: every semantic-orchestration construct's interpreter dispatch is
    /// **byte-identical** to the compile-and-run oracle (verdict + `furthest_position` + typed AST)
    /// across its curated inputs, AND every independent anchor holds.
    #[test]
    fn every_semantic_construct_is_byte_identical() {
        let Some(bin) = oracle_bin_or_skip() else { return };
        let workdir = suite_workdir();

        let reports = evaluate_all_semantic_cases_on_large_stack(bin, workdir)
            .expect("semantic suite plumbing must succeed");

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
            "PARSE-HARNESS.6.2 semantic-orchestration divergences (interpreter != compile-and-run \
             oracle, or anchor miss):\n{}",
            failures.join("\n")
        );
        assert_eq!(
            reports.len(),
            SEMANTIC_CASES.len(),
            "every semantic case must produce a report"
        );
    }

    /// No silent gap: every [`SemanticConstruct::ALL`] variant is exercised by ≥1 case, and every case
    /// name is unique.
    #[test]
    fn semantic_construct_coverage_is_complete() {
        for &c in SemanticConstruct::ALL {
            assert!(
                SEMANTIC_CASES.iter().any(|k| k.construct == c),
                "semantic construct {c:?} has no isolating case — coverage gap"
            );
        }
        let mut names: Vec<&str> = SEMANTIC_CASES.iter().map(|k| k.name).collect();
        names.sort_unstable();
        let before = names.len();
        names.dedup();
        assert_eq!(before, names.len(), "duplicate semantic case name(s)");
    }
}

#[cfg(all(test, feature = "ebnf_dual_run"))]
mod measurement {
    use super::*;

    /// Scouting (`--ignored`): print the full per-case, per-input interpreter-vs-oracle map — used to
    /// establish the honest divergence baseline BEFORE the orchestration mirror lands, and the true
    /// oracle verdicts before locking the anchors (measure-then-lock). Run:
    /// `cargo test --features "generated_parsers ebnf_dual_run" --lib
    ///  parse_harness_semantic_suite::measurement -- --ignored --nocapture`.
    #[test]
    #[ignore = "scouting probe: prints the per-construct semantic differential map; run with --ignored --nocapture"]
    fn measure_semantic_suite() {
        let bin = default_ast_pipeline_bin();
        if !bin.is_file() {
            eprintln!("ast_pipeline not built at {} — cannot measure", bin.display());
            return;
        }
        let workdir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/parse_harness_semantic");
        let reports = evaluate_all_semantic_cases_on_large_stack(bin, workdir).expect("plumbing");

        eprintln!("\n=== PARSE-HARNESS.6.2 semantic-orchestration differential map ===");
        for r in &reports {
            eprintln!("{}", r.summary_line());
            for s in &r.samples {
                eprintln!(
                    "    {:<18} interp={:?} oracle={:?} agreed={} anchor_ok={}{}",
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
        eprintln!("\n{clean}/{} semantic cases CLEAN", reports.len());
    }

    /// Scouting probe (`MEMO-STORE-SOUNDNESS.1`): does the SUCCESS side of the split memo have the
    /// same store-blindness as the failure side (`sem_memo_wrapper`)? A memo-HIT replays the cached
    /// BODY — the node whose nested tournament choices were made under the store AS IT WAS on first
    /// evaluation. Gates re-evaluate fresh (`sem_memo_gate_retry`), but the body CONTENT is
    /// store-frozen. Two isolating variants, both built on the `sem_memo_*` skeleton (an unannotated
    /// rule `pick` whose nested tournament depends on a post gate; a zero-width `@emit_fact` between
    /// the first and second same-position evaluations):
    ///
    /// - **verdict-observable** (`probe_memo_success_verdict`): the gated branch consumes MORE bytes
    ///   (`wide := "gox"` vs `narrow := "go"`), so a stale replay of the narrow win leaves the parse
    ///   misaligned and REJECTS where a fresh evaluation would pick `wide` and ACCEPT `"gox!"`.
    /// - **AST-observable** (`probe_memo_success_ast`): both branches consume the same bytes but shape
    ///   different `kind` markers — a stale replay ACCEPTs with `normal_pick` where a fresh
    ///   evaluation would ACCEPT with `special_pick` (same verdict, wrong tree).
    ///
    /// Prints the per-input interpreter/oracle verdicts + the winning-branch marker, then an explicit
    /// CONFIRMED/ABSENT summary for each side. Run:
    /// `cargo test --features "generated_parsers ebnf_dual_run" --lib
    ///  parse_harness_semantic_suite::measurement::measure_memo_success_side_staleness -- --ignored --nocapture`.
    #[test]
    #[ignore = "scouting probe (MEMO-STORE-SOUNDNESS.1): measures the SUCCESS-side memo-staleness shape; run with --ignored --nocapture"]
    fn measure_memo_success_side_staleness() {
        use crate::parse_harness::ParseOutcome;
        use crate::parse_harness_interpreter::{InterpretOptions, interpret_parse};

        // Variant A — the stale replay flips the VERDICT. First evaluation of `pick` (no fact):
        // `wide`'s body parses but its post gate rejects → `narrow` wins → memo success
        // `(pick, 0) = narrow, end 2`. Branch 2 emits the fact zero-width and retries `pick` at
        // position 0: a stale memo hit replays the narrow win (end 2) so `"!"` misses at byte 2 of
        // `"gox!"` → REJECT; a fresh evaluation would let `wide` pass its gate, win the tournament
        // at end 3, and ACCEPT.
        const VERDICT_GRAMMAR: &str = r#"@fact_kind: { name: g, attributes: [family], description: "G." }
program := pick "?" | en pick "!"
pick := wide | narrow
@predicate: { name: has_fact, args: [g, "on"], phase: post }
wide := "gox"
narrow := "go"
@emit_fact: { kind: g, name: "on", family: f }
en := "on"?
"#;
        // Variant B — same skeleton, equal-length branches with distinct shaped `kind` markers: the
        // stale replay keeps the SAME verdict but the WRONG tree (`normal_pick` instead of
        // `special_pick`) on `"go!"`. `"ongo!"` is the fresh-position control: `en` consumes bytes,
        // so `pick` runs at a new position and the gate steers the tournament to `special_pick`.
        const AST_GRAMMAR: &str = r#"@fact_kind: { name: g, attributes: [family], description: "G." }
program := pick "?" | en pick "!"
pick := special | normal
@predicate: { name: has_fact, args: [g, "on"], phase: post }
special := "go" -> { kind: "special_pick" }
normal := "go" -> { kind: "normal_pick" }
@emit_fact: { kind: g, name: "on", family: f }
en := "on"?
"#;

        let bin = default_ast_pipeline_bin();
        if !bin.is_file() {
            eprintln!("ast_pipeline not built at {} — cannot measure", bin.display());
            return;
        }
        let workdir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/parse_harness_semantic");

        /// Which shaped `kind` marker the winning `pick` branch left in the typed AST (the losing
        /// branch's marker never appears in the tree).
        fn winner_marker(outcome: &Result<ParseOutcome, impl std::fmt::Display>) -> &'static str {
            let Ok(o) = outcome else { return "<error>" };
            let Some(ast) = &o.ast_json else { return "-" };
            let rendered = ast.to_string();
            match (rendered.contains("special_pick"), rendered.contains("normal_pick")) {
                (true, false) => "special_pick",
                (false, true) => "normal_pick",
                (true, true) => "BOTH-MARKERS?!",
                (false, false) => "no-marker",
            }
        }
        fn verdict(outcome: &Result<ParseOutcome, impl std::fmt::Display>) -> String {
            match outcome {
                Ok(o) => format!("accepted={} furthest_position={}", o.accepted, o.furthest_position),
                Err(e) => format!("ERROR: {e}"),
            }
        }

        run_on_large_stack(move || {
            let grammars_dir = workdir.join("grammars");
            std::fs::create_dir_all(&grammars_dir).expect("create probe grammars dir");
            let opts = CompileAndParseOptions {
                ast_pipeline_bin: Some(bin),
                workdir: Some(workdir.clone()),
                keep_workdir: true,
                ..Default::default()
            };
            let interp_opts = InterpretOptions::default();

            let variants: [(&str, &str, &[&str]); 2] = [
                // "gox!" = the verdict discriminator; "go?"/"ongo!" = store-free / fresh-position
                // accept controls; "gox?" rejects under either semantics (both-reject control).
                ("probe_memo_success_verdict", VERDICT_GRAMMAR, &["gox!", "go?", "gox?", "ongo!"][..]),
                // "go!" = the AST discriminator; "go?" accepts store-free as normal_pick; "ongo!"
                // proves the gate steers the tournament to special_pick at a fresh position.
                ("probe_memo_success_ast", AST_GRAMMAR, &["go!", "go?", "ongo!"][..]),
            ];

            let mut oracle_verdict_stale = false;
            let mut interp_verdict_stale = false;
            let mut oracle_ast_stale = false;
            let mut interp_ast_stale = false;

            eprintln!("\n=== MEMO-STORE-SOUNDNESS.1 success-side staleness probe ===");
            for (name, grammar, inputs) in variants {
                let grammar_path = grammars_dir.join(format!("{name}.ebnf"));
                std::fs::write(&grammar_path, grammar).expect("write probe grammar");
                eprintln!("\n--- {name} ---");
                for input in inputs {
                    let interp = interpret_parse(&grammar_path, input, &interp_opts);
                    let oracle = compile_and_parse(&grammar_path, input, &opts);
                    eprintln!(
                        "  {:<9} interp[{} kind={}]  oracle[{} kind={}]",
                        format!("{input:?}"),
                        verdict(&interp),
                        winner_marker(&interp),
                        verdict(&oracle),
                        winner_marker(&oracle),
                    );
                    // The discriminators: a SOUND (store-aware) memo would ACCEPT "gox!" under the
                    // verdict grammar and shape special_pick on "go!" under the AST grammar.
                    if name == "probe_memo_success_verdict" && *input == "gox!" {
                        oracle_verdict_stale = matches!(&oracle, Ok(o) if !o.accepted);
                        interp_verdict_stale = matches!(&interp, Ok(o) if !o.accepted);
                    }
                    if name == "probe_memo_success_ast" && *input == "go!" {
                        oracle_ast_stale = matches!(&oracle, Ok(o) if o.accepted)
                            && winner_marker(&oracle) == "normal_pick";
                        interp_ast_stale = matches!(&interp, Ok(o) if o.accepted)
                            && winner_marker(&interp) == "normal_pick";
                    }
                }
            }

            eprintln!("\n=== summary (sound store-aware memo ⇒ all four read ABSENT) ===");
            eprintln!(
                "  verdict-observable staleness: oracle={} interp={}",
                if oracle_verdict_stale { "CONFIRMED (stale REJECT of \"gox!\")" } else { "ABSENT" },
                if interp_verdict_stale { "CONFIRMED" } else { "ABSENT" },
            );
            eprintln!(
                "  AST-observable staleness:     oracle={} interp={}",
                if oracle_ast_stale { "CONFIRMED (stale normal_pick on \"go!\")" } else { "ABSENT" },
                if interp_ast_stale { "CONFIRMED" } else { "ABSENT" },
            );
        });
    }
}
