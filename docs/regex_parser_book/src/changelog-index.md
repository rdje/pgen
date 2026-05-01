# Changelog Index

This chapter is an index — pointers into other docs that carry the full changelog detail. Use it to find what changed in a given release.

## Where the canonical changelogs live

| Source | Granularity | Purpose |
|---|---|---|
| `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md` | Per-release shape change | The authoritative contract. Each release's section lists the AST shape changes consumers care about. |
| `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` | Per-bug | When a bug is fixed in a release, the ledger entry records the input/output shape change. |
| `CHANGES.md` (root) | Per-release | Human-readable summary of all changes. |
| Git tags + commit log | Commit-by-commit | The most granular source. |

When investigating "what changed and why," start with the contract document, drop down to the ledger for specific bugs, fall back to git for diffs.

## Releases relevant to this book

This book is **live** and tracks current main HEAD. Versioning summary:

- The most recent **published** parser-release section in the contract is **1.1.33 / Contract 1.1.35** (slice 2 of the typed-shape campaign).
- Slices 3 and 4 (typed `counted_quantifier_body` + `null` literal, then typed `counted_quantifier`) are landed on main but the consolidated contract identity bump for them lands together with the next quantifier-subtree slice that closes the outer `quantifier` rule.
- Until then, the contract document still shows `1.1.33 / 1.1.35` while this book describes the post-slice-3+4 shape that's actually emitted by main HEAD.

Below are the shape-change highlights of recent slices, with pointers to the contract sections (where applicable).

### 1.1.34 / Contract 1.1.36 — PGEN-RGX-0075 typed-shape correctness for multi-piece concatenation

**What changed:** The `$N` PositionalRef codegen no longer peels `elements[0]` from a `Quantified` base when the rule body has a single capture position. With this fix, `concatenation = piece+ -> [$1**]` correctly resolves `$1` to the whole `Quantified` (every piece), so `**` flattens all of them into the array.

Compensating grammar change: `regex = pattern -> ...` (was `regex = pattern? -> ...`). The inner `alternative = concatenation?` already handles the empty-input case, so the outer `?` was redundant and only existed to prop up the buggy auto-peel behaviour.

**Consumer impact:** **breaking but in the right direction** — anyone walking `regex.pattern[0][0]` for a multi-piece concatenation now sees every piece. Pre-fix, `"abc"` produced `pattern[0][0] == [piece_a]` (1 piece, buggy); post-fix, `pattern[0][0] == [piece_a, piece_b, piece_c]` (3 pieces, correct). The top-level pattern shape `[<head_alt>, <tail>]` is unchanged. RGX caught this when `Regex::compile("abc")` matched only `"a"` instead of `"abc"`.

**Bug history:** the `\Q...\E` family table covered `\Qab*\E{2,}` (3 pieces — passed incidentally because `piece_quoted_run_quantified`'s annotation pre-built a multi-element Sequence that `**` happened to flatten correctly), but no test asserted plain `"abc"` → 3 pieces in the typed output. PGEN-RGX-0074's empirical evidence focused on `\Q...\E` cases, which masked the underlying `$1`-extraction bug. Fixed in this release with a regression-lock test (`regex_parser_pgen_rgx_0075_multi_piece_concatenation_surfaces_all_pieces`) pinning the empirical shape for `"a"`, `"ab"`, `"abc"`, `"hello"`.

**Contract section:** "Release 1.1.34 / Contract 1.1.36 Highlights".

### Post-1.1.33 main — Task #38 fix: parens-grouped-Or trailing-annotation broadcast

**What changed:** the codegen now correctly applies a trailing return annotation on a parens-grouped Or to **every** alternative inside the group, not just the first. Affects both `extract_rule_annotations` (rust/src/ast_pipeline/mod.rs) and the cross-checker `extract_declared_annotations_from_json` (rust/src/ast_shape_contract.rs).

Pre-fix behaviour: `RULE = (A | B | C) -> ann` applied `ann` to branch 0 only; branches 1, 2 silently fell through to raw passthrough. Documented in `parse_string_literal` of the return_annotation grammar — single-quoted strings produced raw `Sequence` instead of the typed `{type:"string", value:...}` that double-quoted strings produced.

Post-fix behaviour: when a return annotation immediately follows a `group_close`, the annotation broadcasts to every branch that was inside the just-closed group. Per-branch annotations on un-grouped Or rules (`A | B -> ann`, where `-> ann` binds to B only per PEG precedence) still work as before.

**Consumer impact for return_annotation grammar:** single-quoted strings now produce `Json({"type":"string", "value":"..."})` — same as double-quoted. Anyone relying on the buggy raw-Sequence shape needs to update walking code.

**Consumer impact for regex grammar:** none directly from the bugfix. But `quant_base` was refactored to the now-supported factored form `( "*" | "+" | "?" | counted_quantifier ) -> $1` (was per-branch `-> $1` four times) — same JSON output.

**Contract section:** pending bump (will land with the slice that closes `quantifier`).

### Post-1.1.33 main — quant_base annotated (slice 5/N)

**What changed:** `quant_base = "*" | "+" | "?" | counted_quantifier` got per-branch `-> $1` annotations on every alternative. After task #38 fix landed (subsequent commit), the rule was refactored to the factored form `quant_base = ( "*" | "+" | "?" | counted_quantifier ) -> $1` — semantically identical, more elegant.

**Consumer impact:** **none** — JSON output is byte-identical to pre-slice-5. Empirical: `a*` still emits `quantifier: ["*", []]`; `a{2,5}` still emits `quantifier: [{"min":2,"max":5}, []]`. The change is to the rule's emission status (from "raw envelope via codegen default" to "annotated, Tier-2 stable").

**Contract section:** pending bump (will land with the slice that closes `quantifier`).

### Post-1.1.33 main — counted_quantifier typed (slice 4/N)

**What changed:** `counted_quantifier` rule got `-> $3` annotation, lifting `counted_quantifier_body`'s typed `{min, max}` straight through and dropping the surrounding `{`/`}`/whitespace tokens. The brace tokens carry no semantic information beyond "this is a counted quantifier" — context the surrounding `quant_base` already conveys.

**Consumer impact:** the `quant_base` position (visible inside `quantifier`'s `[<base>, <suffix>]` shape) now carries either a bare string `"*"`/`"+"`/`"?"` OR a typed `{min, max}` object directly. No more digging through a 5-element Sequence wrapper to reach the body's typed shape. See [Quantifier Subtree](rules-quantifier.md).

**Contract section:** pending bump (will land with the slice that closes `quantifier`).

### Post-1.1.33 main — counted_quantifier_body typed + null literal (slice 3/N)

**What changed:** restructured `counted_quantifier_body` from 2 branches (with 4 logical cases compressed inside an optional sub-group) into 4 explicit branches each with its own per-branch `-> {min, max}` annotation. Added the `null` literal to the return-annotation language so the unbounded `{n,}` form can encode `max:null` directly.

**Consumer impact:** the body now emits a typed `{min, max}` object regardless of which `{n}`/`{n,}`/`{n,m}`/`{,m}` source form matched. `min` is always a typed integer; `max` is a typed integer OR `null` (only `null` for the unbounded form). See [Quantifier Subtree](rules-quantifier.md), [Quantifiers](examples-quantifiers.md).

**Contract section:** pending bump (will land with the slice that closes `quantifier`).

### 1.1.33 / Contract 1.1.35 — quant_suffix typed (slice 2/N)

**What changed:** `quant_suffix` rule's two branches each got a return annotation: `"?"` → `"lazy"`, `"+"` → `"possessive"`. The `quant_suffix?` slot inside `quantifier` now carries either `[]` (greedy, no suffix matched) or `["lazy"]` / `["possessive"]` (1-element Quantified-?).

**Consumer impact:** consumers reading the quantifier shape can dispatch on the string instead of inferring from which branch matched. See [Quantifier Subtree](rules-quantifier.md).

**Contract section:** "Release 1.1.33 / Contract 1.1.35 Highlights".

### 1.1.32 / Contract 1.1.34 — digits typed (slice 1/N)

**What changed:** `digits` rule got `@transform: str::parse::<usize>().unwrap_or(0)` annotation. The rule now emits a typed integer (`Json(Value::Number(usize))`) instead of a `Terminal` of digit characters.

**Consumer impact:** consumers reading counted-quantifier bounds get the integer directly. See [Quantifier Subtree](rules-quantifier.md).

**Contract section:** "Release 1.1.32 / Contract 1.1.34 Highlights".

### 1.1.31 / Contract 1.1.33 — PGEN-RGX-0074 `\Q...\E` correctness

**What changed:** introduced the `piece_quoted_run_quantified` rule + the `**` flatten-spread operator. Restructured `concatenation` to `piece+ -> [$1**]`. The `\Q...\E quantifier` shape now emits per-char piece array with the trailing piece carrying the quantifier — matching PCRE2's "quantifier binds to last char of \Q...\E" semantics.

**Consumer impact:** **breaking** — any consumer that walked the pre-1.1.31 whole-block-quantified shape will see a different AST. The new shape is what PCRE2 says it should be. Drop any pre-existing `\Q...\E`-quantifier-attachment workaround. See [\Q...\E Quoted Literals](examples-quoted-literal.md).

**Contract section:** "Release 1.1.31 / Contract 1.1.33 Highlights".

### 1.1.30 / Contract 1.1.32 — PGEN-RGX-0073 perf closure

**What changed:** primary focus was performance — closing all 8 patterns under p50 < 50µs. Schema-affecting changes were minor: implicit `-> $1` default tightened to exclude `Quantified` bodies (so single-Quantified rules without an explicit annotation no longer collapse to the inner item). Several rule rewrites of slow Or-of-chars rules using `/.../` regex literals (perf-only, no shape change).

**Consumer impact:** if a consumer relied on the implicit-collapse behavior for a Quantified body, they need to add an explicit `-> $1` annotation to that rule (or update their walker to handle the un-collapsed shape). Vanishingly rare in practice — the regex grammar didn't have any rules in this position. See contract for full details.

**Contract section:** "Release 1.1.30 / Contract 1.1.32 Highlights".

## Earlier releases (pre-1.1.30)

Pre-1.1.30 the regex parser used the recursive-envelope shape exclusively — no `Json` carrier, every rule produced `Sequence` / `Alternative` / `Quantified` / `Terminal`. Consumers who built against pre-1.1.30 should read [Migration from the Recursive Envelope](migration-from-recursive-envelope.md) before reading the rest of this book.

For shape-affecting changes within the pre-1.1.30 envelope era, see the contract document — its release sections go back to the parser's earliest releases.

## Future releases

Upcoming slice campaign work (task #40 — "Annotate regex.ebnf for full AST usability"):

| Slice | Target rule | Expected AST change |
|---|---|---|
| 6 | `quantifier` | Combine quant_base + quant_suffix? into typed `{type:"quantifier", min, max, greediness}` |
| 7+ | atom subtree, char class, group family, escape subtree, ... | One rule per slice |

Each slice will:

- Bump the contract version.
- Add a row to this index.
- Get a CHANGES.md entry.
- Get a corresponding chapter update where applicable.

The eventual schema 1.0.0 milestone will land when every regex.ebnf rule is either annotated or has a deliberate "remain raw envelope" decision documented.

## Looking up a specific input's behavior

If you need to know "what does the parser output for input X across release Y?," the workflow is:

1. Check `git log --oneline grammars/regex.ebnf` to see when the rule for X was last touched.
2. Check the contract section for that release.
3. Check the ledger if there was a bug fix involved.
4. As last resort, check out the relevant tag and run `parseability_probe --parse-dump-ast-pretty regex 'X'` against it.

For consumer-side regression detection, the recommended pattern is the snapshot test described in [Schema Versioning](schema-versioning.md).
