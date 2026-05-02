# Schema Versioning

This chapter explains how the regex parser's AST shape is versioned, what guarantees consumers can rely on, and how to pin to a known shape.

## Two versioning axes

The regex parser carries **two** version numbers:

1. **Parser release version** — e.g. `1.1.31`. Tracks the parser library's release identity. Bumped on every functional change to the parser, including bug fixes, perf work, and grammar changes.
2. **Schema version** — e.g. `0.7.x`. Tracks the AST output shape. Bumped only when the output shape changes in a way consumers may need to adapt to.

A single parser release can carry the same schema version as the previous release (no shape change) or a bumped schema version (shape changed). The two version numbers move independently.

The contract document `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md` is the authoritative source for both numbers per release.

## What "shape change" means

Any of these triggers a schema version bump:

- A new return annotation lands on a previously-unannotated rule (e.g. slice 1's `digits → integer`, slice 2's `quant_suffix → enum string`).
- An existing return annotation is restructured (e.g. PGEN-RGX-0074's piece-array reshape).
- A grammar rule changes shape in a way that's user-visible (new branch added, branch removed, sub-rule renamed in a way that affects shape).
- The default fall-through behavior of unannotated rules changes (e.g. when `body_has_single_element` was tightened to exclude `Quantified`).

These do NOT trigger a schema bump:

- Pure performance optimizations that produce the same AST.
- Internal codegen reorganization that doesn't reach the output.
- Parser-side bug fixes that produce the same shape consumers were already relying on.

The slice campaign for task #40 ("Annotate regex.ebnf for full AST usability") will produce many small schema bumps as rules are annotated one-by-one. Each slice gets its own contract-version row.

## Byte-equivalence guarantee

For any input the parser accepts, this equality holds:

```rust
parse_full_regex(input).unwrap().content.to_json_value()
    == parse_regex_typed(input).unwrap()
```

That is — walking the typed `Json` content via `to_json_value()` produces the same `serde_json::Value` as the typed parser entry point produces directly. This is a stability invariant we maintain across **all** future shape changes.

In practice this means:

- Consumers can use either entry point (`parse_full_regex` for envelope-with-spans, `parse_regex_typed` for plain `serde_json::Value`).
- Consumers writing JSON snapshots from one entry point can later switch to the other without re-baselining.

If you ever encounter a case where the equality fails, that's a parser bug — please report.

## Stability tiers

The regex parser's behavior is divided into three tiers by stability guarantee:

### Tier 1 — Stable surface (contract-bound)

| Item | Stability |
|---|---|
| `crate::parse_full_regex(input) -> Result<ParseNode<'_>, ParseError>` | Stable signature. Function does not move or rename. |
| `crate::parse_regex_typed(input) -> Result<serde_json::Value, ParseError>` | Stable signature. |
| `ParseNode { rule_name, content, span }` field set | Stable. New fields may be added (additive); existing fields not removed. |
| `ParseContent` six-variant set | Stable. Variants not removed without major version bump and migration window. |
| Schema version field in CHANGES.md | Always present per release. |

### Tier 2 — Annotated rule shapes

Each rule listed in [The Json Carrier](json-carrier.md)'s annotated-rule table is part of the schema once annotated. Removing or substantially changing the typed shape requires:

1. A schema version bump.
2. A contract document update with a "before/after" entry.
3. A `MIGRATION.md` entry for affected consumers.

We commit to NOT silently changing typed shapes. Consumers can rely on a given annotation's output shape across all releases that share the schema version.

### Tier 3 — Unannotated rule shapes

Rules NOT in the annotated list emit raw `Sequence` / `Alternative` / `Quantified` envelopes. Their shape is determined by the codegen's default fall-through and may evolve as part of:

- Grammar reorganization (e.g. sub-rule extraction or merging).
- Implicit `-> $1` default policy changes (unlikely; tightened in PGEN-RGX-0073-era).
- Slice campaign annotation (the rule moves from Tier 3 to Tier 2).

The expected lifecycle of a Tier 3 rule is to be annotated and promoted to Tier 2, after which its shape is locked.

If you walk a Tier 3 rule today, your walker may need to adapt when that rule is annotated. The recommended approach is to centralize Tier 3 walking in one place per rule and update it slice-by-slice.

## Pinning policy

For consumer projects, pin the PGEN parser release in `Cargo.toml`:

```toml
[dependencies]
parseability = { git = "https://github.com/RichardSamWell/pgen.git", tag = "1.1.31" }
```

The release tags are stable git references; once published, they don't move. Pin to a specific tag to lock the AST shape. To upgrade, read the contract changelog for the target version, adjust your walker if needed, and re-pin.

Branch-based pinning (`branch = "main"`) is **not recommended** for consumer projects — it makes shape changes appear without notice. For development experimentation, branch-pinning is fine.

## How to detect schema changes in CI

The recommended consumer-side CI guard is a regex-AST snapshot test:

```rust
#[test]
fn ast_snapshot_for_canonical_inputs() {
    let inputs = [
        r"a", r"a*", r"\d+", r"[a-z]", r"\Qab\E{3}", /* ... */
    ];
    for input in inputs {
        let actual = parseability::parse_regex_typed(input).unwrap();
        let expected = read_snapshot(input);
        assert_eq!(actual, expected, "AST shape changed for input: {input}");
    }
}
```

When you upgrade PGEN, this test will fail loudly on any shape change. Re-baselining the snapshots gives you the diff to inspect.

## Schema version timeline

| Schema version | First parser release | Notable changes |
|---|---|---|
| 0.5.x | pre-1.1.30 | Pure recursive envelope. |
| 0.6.0 | 1.1.30 | First annotated rules: `regex`, `pattern`, `concatenation`, `piece`. |
| 0.7.0 | 1.1.31 | PGEN-RGX-0074 fix: `\Q...\E` per-char piece array. New `**` flatten-spread semantics. |
| 0.7.1 | 1.1.32 | `digits` → integer (slice 1). Within-version-line shape addition (additive). |
| 0.7.2 | 1.1.33 | `quant_suffix` → enum string (slice 2). |
| 0.8.0 | post-1.1.33 main | `counted_quantifier_body` → typed `{min, max}` (slice 3). New `null` literal in the annotation language. |
| 0.8.1 | post-1.1.33 main | `counted_quantifier` → `-> $3` lifts body's typed shape (slice 4). |
| 0.8.2 | post-1.1.33 main | `quant_base` → per-branch `-> $1` annotations (slice 5). Lock-in only; JSON output unchanged. |
| 0.8.3 | post-1.1.33 main | Task #38 fix: parens-grouped-Or trailing annotations broadcast to every inner alternative. Affects `string_literal` in return_annotation grammar (single-quoted strings now produce typed `{type:"string"...}`). Regex grammar consumer-impact: none directly; `quant_base` refactored to the factored form. |
| 0.9.0 | 1.1.34 | **Breaking but correct:** PGEN-RGX-0075 fix — `$N` codegen no longer peels `elements[0]` from a Quantified base. Multi-piece concatenations now surface every piece in `regex.pattern[0][0]` (was: only the first piece). Compensating grammar change: `regex = pattern` (was `regex = pattern?`). Top-level pattern shape `[<head_alt>, <tail>]` unchanged. |
| 0.10.0 | 1.1.35 | **Breaking but correct:** quantifier subtree closure (slice 6/N). `quant_base` reshaped from per-branch `-> $1` (heterogeneous string-vs-object) to per-branch typed `{min, max}` for every alternative. `quantifier` rule annotated `-> {type:"quantifier", min:$1.min, max:$1.max, greediness:$2}`. Piece's `quantifier` field is now a fully typed object (or `[]` for un-quantified pieces); was `[<base>, <suffix>]` 2-tuple. Consumers walking the quantifier shape need to update from string-vs-object dispatch to typed-field read. |
| 0.11.0 | 1.1.36 | **Breaking but correct:** atom subtree slice 7 (`anchor`). `anchor` rule's 9 branches each annotated `-> {type:"anchor", kind:"<name>"}`. Piece atoms for `^`/`$`/`\A`/`\Z`/`\z`/`\b`/`\B`/`\G`/`\K` now emit typed objects with semantic kind names (e.g. `start_of_line`, `word_boundary`) instead of raw escape strings. Consumers walking the anchor shape need to update from `atom.as_str() == "\\b"` to `atom.get("kind").as_str() == "word_boundary"`. |
| 0.12.0 | 1.1.37 | **Breaking but correct:** PGEN-RGX-0076 fix (slice 8). `posix_class` rule's annotation upgraded from `-> $1` (placeholder that emitted only the `"[:"` opener and silently dropped class name + negation) to `-> {type:"posix_class", name:$3, negated:$2}`. `posix_negation` now annotated `-> true`. Inside character class bodies, every `[:name:]` now emits a typed `{type, name, negated}` object — `name` is the POSIX class name string, `negated` is `true` (matched) or `[]` (un-matched, map → `false`). Codegen scalar paths for `BooleanLiteral`/`NumberLiteral` fixed in the same commit (was emitting `Terminal(<str>)` instead of typed `Json(Bool/Number)`). |
| 0.13.0 | 1.1.38 | **Breaking but correct:** atom subtree slice 9 (`posix_word_boundary_alias`). The 2 branches each annotated `-> {type:"anchor", kind:"posix_word_start\|posix_word_end"}` so PCRE2-style `[[:<:]]` and `[[:>:]]` aliases join the same typed anchor family produced by the regular `anchor` rule. Closes the anchor family — all 11 anchor variants now emit `{type:"anchor", kind:<name>}` uniformly. Consumers can drop any string-match fallback they had for the POSIX aliases. |
| 0.14.0 | 1.1.39 | **Breaking but correct:** atom subtree slice 10 (`backreference`). 4 per-branch annotations producing typed `{type:"backreference", kind:<form>, ...}` shapes. `backreference_digits` rewritten as a regex literal with `@transform: str::parse::<usize>` (typed integer). Branch 0 (numeric `\1`) carries `index:<int>`; branches 1-3 carry `ref:<raw sub-rule shape>` until follow-up slices type `name_ref`/`braced_name_ref`/`subroutine_ref`. Consumers walking the backreference shape need to update from `["\\", <digits>]` array dispatch to typed `obj.kind` dispatch. |
| 0.14.1 | 1.1.40 | **Bug fix:** PGEN-RGX-0077 — the `[$1**]` flatten-spread codegen now peels `Alternative` wrapping before inspecting child content for the unwrap decision. Pre-fix every multi-char `\Q...\E quantifier?` source carried one extra wrap layer at `pattern[0][0]`. Post-fix the documented flat shape `[<N pieces>]` is restored. Also adds preventative `Json(Value::Array(_))` arm in the same codegen for any future annotation that produces typed-Json arrays directly. Regex AST schema version stays `1`. |
| 0.15.0 | 1.1.41 | **Breaking but correct:** atom subtree slice 11 (named-ref cleanup). `name` rewritten as a regex literal — emits the matched name string as a clean Terminal directly (was a multi-element `name_start name_continue*` chain producing `[first_char, [rest_chars]]`). `name_ref` (both branches) annotated `-> $2` to extract just the name string from the `<...>`/`'...'` wrappers. `braced_name_ref` annotated `-> $3` to extract from `{...}` wrappers. Cascading effect: every consumer of `name` sees a clean string — `\k<foo>` / `\k{foo}` / `(?<foo>...)` / `(?P<foo>...)` / `(?P=foo)` etc. all surface `"foo"` directly instead of the raw character chain. `subroutine_ref` is still un-annotated but its inner `name` is now a clean string within the otherwise-raw shape. |
| 0.16.0 | 1.1.42 | **Breaking but correct:** atom subtree slice 12 (subroutine_ref cleanup). `subroutine_ref` annotated per-branch (`-> $1`/`-> $2`/`-> $2`/`-> $1`) to drop the angle/quote/brace delimiters and surface the inner `signed_digits_or_name` directly. `braced_subroutine_ref` annotated `-> $3`. Backreference `subroutine` form now emits clean values: `\g<name>`/`\g'name'`/`\g{name}` → `ref:"name"`; `\g<1>`/`\g{42}`/`\g42` → `ref:[<sign?>, <int>]`; `\g+1`/`\g-2` → `ref:[<sign>, <int>]`. Closes the backreference family typing — all 4 backreference forms now produce clean inner values. `signed_digits` and `sign` rules remain un-annotated (raw `[<sign?>, <digit-int>]` for numeric refs); typing them is a separate sub-slice. |
| 0.17.0 | 1.1.43 | **Breaking but correct:** atom subtree slice 13 (signed_digits typing). `signed_digits` annotated `-> {sign: $1, value: $2}` so numeric subroutine refs surface a typed `{sign, value}` object instead of `[<sign?>, <int>]`. `sign` is `"+"`/`"-"` when matched, `[]` when un-matched (consumer maps `[]` → null/unsigned). `value` is the typed integer. Backreference family typing fully closed: every numeric subroutine ref now reads `obj.ref.sign` and `obj.ref.value` as named fields. |
| 0.18.0 | 1.1.44 | **Breaking but correct:** atom subtree slice 14 (escape subtree starts: simple/single_byte/control). `escape` rule annotated `-> $2` (transparent passthrough). `single_byte_escape -> {type:"escape", kind:"single_byte"}`, `simple_escape -> {type:"escape", kind:"shorthand", char:$1}`, `control_escape -> {type:"escape", kind:"control", char:$2}`. PCRE2 `\d`/`\w`/`\s`/`\.`/`\\` (shorthand), `\C` (single_byte), `\cA`/`\cz` (control) atoms now emit typed `{type, kind, char?}` objects. The other 4 escape_unit branches (hex/unicode/octal/property) still emit raw shapes — they pass through `escape -> $2` un-typed; follow-up slices will annotate each. |
| 0.19.0 | 1.1.45 | **Breaking but correct:** atom subtree slice 15 (escape subtree continues: hex/unicode). `hex_escape` annotated per-branch `-> {type:"escape", kind:"hex", digits:<hex-string>}`. `unicode_escape` annotated `-> {type:"escape", kind:"unicode", digits:<hex-string>}`. New `hex_escape_short_payload` regex literal `/([0-9A-Fa-f]{1,2})/` for the short `\xNN` form. `hex_digits` rewritten from `hex_digit+` chain to a regex literal `/([0-9A-Fa-f]+)/`. PCRE2 `\xF`/`\xFF`/`\x{1F}`/`\x{1F600}` (hex) and `\u{NNNN}` (unicode) atoms emit typed `{type, kind, digits}` objects. `digits` is the matched hex string; consumers parse to int via `usize::from_str_radix(digits, 16)`. Int-decoding via the rule's `@transform` would require extending the transform machinery to support `from_str_radix`-style; saved as a separate codegen-feature slice. |
| 0.20.0 | 1.1.46 | **Breaking but correct:** atom subtree slice 16 (escape subtree continues: octal). `octal_escape` annotated per-branch `-> {type:"escape", kind:"octal", digits:<octal-string>}`. New `octal_escape_short_payload` regex literal `/([0-7]{1,3})/` for the bare 1-3-digit form. `octal_digits` rewritten from `octal_digit+` chain to a regex literal `/([0-7]+)/`. Braced `\o{NNN...}` (atom) and bare `\NNN` (in classes via `class_range_escape_unit`) emit typed `{type, kind, digits}` objects. `digits` is the matched octal string; consumers parse with `usize::from_str_radix(digits, 8)`. Bare `\NNN` at atom-level remains parsed as `backreference` (numeric kind) under the existing PEG ordering — pre-existing, not changed by this slice; PEG can't express PCRE2's contextual backref-vs-octal disambiguation directly. 6/7 escape_unit branches now typed; only `property_escape` remains. |
| 0.32.0 | 1.1.58 | **Breaking but correct:** atom subtree slice 28 (`extended_class` typed). `extended_class -> {type:"atom", kind:"extended_class", body:$2}`. `(?[abc])` now emits `{kind:"extended_class", body:["a","b","c"]}` instead of the 3-element `["(?[", <content>, "])"]` Sequence. `body` is the raw extended_class_content shape; sub-rule typing of the recursive set-operation structure is a separate concern. **All recursive atom alternatives now typed: 25/25 directly.** Only the 3 deferred leaf-char alternatives remain. |
| 0.31.0 | 1.1.57 | **Breaking but correct:** atom subtree slice 27 (`conditional` typed). `conditional -> {type:"atom", kind:"conditional", condition:$2, yes_branch:$4, no_branch:$5}` plus `conditional_branch = piece* -> [$1**]` (flat array of pieces, paralleling `concatenation`). `condition` is the heterogeneous Or-of-9 raw shape — typed signed_digits propagates `{sign, value}` for numeric; `"DEFINE"` string for DEFINE; `["R", ...]` for recursion; clean `name` string for named refs. `no_branch` preserves the `|` separator: `[]` (no else-clause) or `["|", <pieces>]` (else-clause present). |
| 0.30.0 | 1.1.56 | **Breaking but correct:** atom subtree slice 26 (`char_class` outer typed). `char_class -> {type:"atom", kind:"char_class", negated:<bool>, initial_close:<bool>, body:<class_body>}`. `negation -> true` and `class_initial_close -> true` (real booleans paralleling `posix_negation`). `body` is raw class_body shape — inner items already typed by earlier slices (posix_class slice 8, class_range escape slices) propagate transparently. `[abc]` now emits `{type:"atom", kind:"char_class", negated:[], initial_close:[], body:["a","b","c"]}` instead of the 5-element `["[", <neg>, <ic>, <body>, "]"]` Sequence. |
| 0.29.0 | 1.1.55 | **Breaking but correct:** atom subtree slice 25 (scan_substring / script_run / subroutine_call typed). 4 annotations across 3 atom alternatives. `scan_substring_group` → `{kind:"scan_substring_group", name, captures, body}`. `script_run_group` → `{kind:"script_run_group", name, body}`. `subroutine_call` (both branches) → `{kind:"subroutine_call", target}` — `target` is the inner shape (returned_capture_subroutine OR subroutine_target). `signed_digits` typing from slice 13 propagates through to `target` for numeric subroutine refs like `(?+1)`. |
| 0.28.0 | 1.1.54 | **Breaking but correct:** atom subtree slice 24 (inline-modifier / callout / directive_verb / code_block typed). 6 annotations across 5 atom alternatives. `inline_modifiers` → `{kind:"inline_modifiers", spec:<modifier_spec>}`. `scoped_inline_modifiers` → `{kind:"scoped_inline_modifiers", spec, body}`. `callout` → `{kind:"callout", arg:<callout_arg>}`. `directive_verb` → `{kind:"directive_verb", body}`. `code_block` (both `code_block_plain` and `code_block_lang` branches) → `{kind:"code_block", lang:<null \| string>, content}`. Sub-rule shapes (modifier_spec, callout_arg, directive_body, code_content) carry raw shapes; per-rule typing is a separate slice. |
| 0.27.0 | 1.1.53 | **Breaking but correct:** atom subtree slice 23 (lookaround family typed — 7 sub-rules in one batch). 7 annotations: `lookahead_pos`/`lookahead_neg` collapse to `kind:"lookahead"` with `positive` boolean (`true`/`false`); same for `lookbehind`; non-atomic forms (`non_atomic_lookahead`/`non_atomic_lookbehind`) get distinct kinds since PCRE2 only supports positive variants; alpha-form `(*<name>:...)` gets `kind:"alpha_lookaround"` with `name` string carrying the alpha_lookaround_name. `(?=foo)` now emits `{type:"atom", kind:"lookahead", positive:true, body:<pattern>}` instead of the 3-element `["(?=", <pattern>, ")"]` Sequence. Lookaround family typed end-to-end. |
| 0.26.0 | 1.1.52 | **Breaking but correct:** atom subtree slice 22 (named groups typed: named/python_named). 3 annotations across 2 rules. `named_group` (both angle `(?<n>...)` and quote `(?'n'...)` forms) → `{type:"atom", kind:"named_group", name:<str>, body:<pattern>}`. `python_named_group` (`(?P<n>...)`) → `kind:"python_named_group"` (distinct from `named_group` for syntactic-origin preservation, paralleling slice 19's `python_named_backreference`). `name` was already typed to a clean string by slice 11. **Group typing now end-to-end** — all 6 group sub-rules (capturing, noncapturing, named, python_named under `group`; branch_reset, atomic standalone) emit typed shapes. |
| 0.25.0 | 1.1.51 | **Breaking but correct:** atom subtree slice 21 (simple groups typed: capturing/noncapturing/branch_reset/atomic). 5 annotations (atomic_group has 2 syntactic branches both producing `kind:"atomic_group"`). All 4 group forms emit `{type:"atom", kind:<group_kind>, body:<pattern>}`. `(abc)` now emits `{type:"atom", kind:"capturing_group", body:<pattern>}`; `(?:abc)` → `kind:"noncapturing_group"`; `(?>abc)` and `(*atomic:abc)` both → `kind:"atomic_group"`; `(?|a|b)` → `kind:"branch_reset_group"`. `body` is the raw pattern shape (pattern outer typing is a separate slice). Empty groups `()` / `(?:)` emit `body:[[], []]`. Largest atom-subtree slice yet — 4 atom alternatives typed in one pass. |
| 0.24.0 | 1.1.50 | **Breaking but correct:** atom subtree slice 20 (`comment_group` typed). `comment_group = "(?#" comment_text ")" -> {type:"atom", kind:"comment", text:$2}`. `comment_text` rewritten from `comment_char*` chain to regex literal `/([^)]*)/`. `(?#hello)` now emits `{type:"atom", kind:"comment", text:"hello"}` instead of the 3-element `["(?#", [<chain>], ")"]` Sequence. Empty comments `(?#)` emit `text:""` (real empty string), not `[]` from an un-matched optional slot — the `?` after `comment_text` in `comment_group` was dropped because the regex literal accepts empty. Char-set coverage of `[^)]*` matches the previous `comment_char*` chain (any char except `)`). |
| 0.23.0 | 1.1.49 | **Breaking but correct:** atom subtree slice 19 (`python_named_backreference` typed). `python_named_backreference = "(?P=" name ")" -> {type:"backreference", kind:"python_named", ref:$2}`. `(?P=foo)` now emits `{type:"backreference", kind:"python_named", ref:"foo"}` instead of the 3-element `["(?P=", "foo", ")"]` Sequence. `kind:"python_named"` is a distinct value from the regular `\k<...>` family's `kind:"named"` — PCRE2 treats them as functionally equivalent for matching, but the syntax origin is preserved for tooling. Consumers normalizing all name-based forms can use `kind in {"named", "named_braced", "python_named"}` → name-based backref; `ref` is the name in all three. Backreference family typing is now end-to-end across all 5 syntactic forms. |
| 0.22.0 | 1.1.48 | **Breaking but correct:** atom subtree slice 18 (`quoted_literal` typed). `quoted_literal` annotated `-> {type:"atom", kind:"quoted_literal", body:$2}`. `\Qhello\E` now emits `{type:"atom", kind:"quoted_literal", body:["h","e","l","l","o"]}` instead of the 3-element `["\\Q", [<chars>], "\\E"]` Sequence. `body` is an array of single-char strings (`quoted_literal_escaped_char` produces 2-char strings — escapes preserved). First atom-subtree slice after the escape closure; consumers now have typed `kind` dispatch on most non-leaf atoms. |
| 0.21.0 | 1.1.47 | **Breaking but correct:** atom subtree slice 17 (escape subtree closes: property). `property_escape` annotated per-branch `-> {type:"escape", kind:"property", name:<string>, negated:<bool>}`. `prop_name` rewritten from `prop_name_chars+` chain to regex literal `/([A-Za-z0-9 \t\n\r\f\v_:\-=&^]+)/`. `short_prop_letter` rewritten from Or-of-single-chars chain to regex literal `/([CLMNPSZclmnpsz])/`. PCRE2 `\p{Lu}`/`\P{Lu}` (braced) and `\pL`/`\PN` (short) atoms emit typed objects. `negated` is a real boolean (`true` for `\P` forms, `false` for `\p` forms) — no need for consumer to inspect leading-token shape. **The escape subtree is now closed: 7/7 escape_unit branches typed** (single_byte, simple, control, hex, unicode, octal, property). |

(Numbers above match the contract document at the time this book was written. The contract is authoritative for the current state — consult it for the live version.)

## Future major version

A schema 1.0.0 milestone will land when the task #40 annotation campaign completes — that is, when every rule in `regex.ebnf` carries either a return annotation or a deliberate decision to remain raw envelope. At that point all shape definitions move to Tier 2 (locked) and no further default fall-through changes are expected.

Pre-1.0 schema versions (0.x.y) follow semver-ish convention — minor bumps for additive changes, patch for purely-additive within-shape additions, breaking changes are explicitly called out and may bump the minor digit.

Post-1.0, breaking schema changes become major-version events with a deprecation cycle.
