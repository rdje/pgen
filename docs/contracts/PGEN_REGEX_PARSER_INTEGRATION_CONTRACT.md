# docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md

## Purpose
Define the downstream integration contract for PGEN's `regex` parser family.

This is the document downstream projects such as RGX should read first when deciding how to embed the PGEN regex parser.

## Contract Identity
- Contract version:
  - `1.1.59`
- Parser release version:
  - `1.1.57`
- Embedding API contract baseline:
  - `1.2.0`
- Regex AST-dump schema version:
  - `1`
- Last updated:
  - `2026-05-01`
- Current grammar family label:
  - `regex`
- Current stable host profile:
  - `regex_default`
- Current live status:
  - `Done` for the currently tracked grammar contract in `LIVE_ACHIEVEMENT_STATUS.md`

## Current Trust Statement
- PGEN currently treats the published regex flavor, when consumed through the stable `pgen::embedding_api` host surface, as closure-grade and fit for downstream parser consumption.
- That statement applies to the published regex parser contract documented here and in the regex-flavor section of `PGEN_USER_GUIDE.md`.
- It does not automatically cover every regex dialect or every future contract widening.

## Companion Documentation — Regex Parser Integration mdBook
- The regex-parser integration mdBook lives at `docs/regex_parser_book/` and is the **canonical AST reference** for downstream consumers (RGX in particular).
- The book documents: cold-clone build recipe, public API, the full AST envelope, every annotated/un-annotated rule shape, worked examples for every regex feature, migration from the pre-1.1.30 recursive envelope, schema versioning, glossary, and a release-by-release index.
- Build it with `make regex_parser_book_gate` (uses `mdbook build docs/regex_parser_book`).
- Where the book and this contract disagree, **the contract wins** for compliance — but please report the disagreement as a documentation bug.

## Release 1.1.57 / Contract 1.1.59 Highlights — atom subtree slice 27: conditional typed

- **Internal-driven shape work** (no downstream report).
- **Rules changed:**
  - `conditional = "(?(" condition ")" yes_branch ("|" no_branch)? ")" -> {type:"atom", kind:"conditional", condition:$2, yes_branch:$4, no_branch:$5}`
  - `conditional_branch = piece* -> [$1**]` (flat array of pieces, paralleling `concatenation`).
- **AST shape change (consumer-visible):**
  - Before: `(?(1)abc)` → 5-element Sequence `["(?(", <condition>, ")", <yes_branch>, [], ")"]`.
  - After: `(?(1)abc)` → `{type:"atom", kind:"conditional", condition:{sign:[], value:1}, yes_branch:[<3 piece objects>], no_branch:[]}`.
  - With else-clause `(?(1)abc|xyz)` → `no_branch:["|", [<3 piece objects>]]` — the parens-grouped pair preserves the `|` separator.
- **`condition` is the heterogeneous Or-of-9 raw shape:** typed `signed_digits` `{sign, value}` for numeric refs (slice 13 propagation); `"DEFINE"` string for `(?(DEFINE)...)`; `["R", []]` 2-element Sequence for recursion conditions; `name` string for named-group refs (already typed by slice 11); etc. Sub-rule typing of `condition` is a separate concern.
- **`no_branch` shape:** `[]` when no else-clause, `["|", <pieces>]` when matched. Consumer reads `no_branch[1]` to extract the actual no-branch pieces, or maps `[]` to null. The `|` separator preservation is intentional — disambiguates "no else-clause" (null/missing) from "empty else-clause" (`["|", []]`).
- **`conditional_branch` flat-piece-array shape** parallels `concatenation`'s `[$1**]` form. Consumer iterates the array directly.
- **Recommended RGX integration steps:**
  1. Update PGEN dependency to the post-`1.1.57` commit on `main`.
  2. Regenerate the regex parser via `make regex_parser` or `make regex_parser_fresh`.
  3. Update any code that walked the raw `["(?(", <condition>, ")", <yes>, <no?>, ")"]` shape to use typed `obj.kind == "conditional"` dispatch + `obj.condition` (heterogeneous shape) + `obj.yes_branch` (piece array) + `obj.no_branch` (`[]` or `["|", <pieces>]`).
- Public API surface unchanged.
- Regex AST schema version stays `1`.
- **Atom subtree campaign progress:** 24/25 atom alternatives directly typed. Remaining 1: `extended_class`. (Plus the 3 deferred leaf-char alternatives.)

## Release 1.1.56 / Contract 1.1.58 Highlights — atom subtree slice 26: char_class outer typed

- **Internal-driven shape work** (no downstream report).
- **Rules changed:**
  - `char_class = "[" negation? class_initial_close? class_body "]" -> {type:"atom", kind:"char_class", negated:$2, initial_close:$3, body:$4}`
  - `negation = "^" -> true` (paralleling `posix_negation` from slice 8 / PGEN-RGX-0076).
  - `class_initial_close = "]" -> true` (`true` for matched, `[]` for un-matched optional slot).
- **AST shape change (consumer-visible):**
  - `[abc]` → `{type:"atom", kind:"char_class", negated:[], initial_close:[], body:["a","b","c"]}`. Was `["[", [], [], <class_body>, "]"]` (5-element Sequence).
  - `[^abc]` → `{kind:"char_class", negated:true, body:["a","b","c"]}`.
  - `[]abc]` (literal `]` first char) → `{kind:"char_class", initial_close:true, body:["a","b","c"]}`.
  - `[^]abc]` → `{negated:true, initial_close:true, body:["a","b","c"]}`.
  - `[[:alpha:]]` → `{kind:"char_class", body:[{type:"posix_class", name:"alpha", negated:[]}]}` — the already-typed `posix_class` shape (slice 8) propagates inside `body`.
- **`negated` and `initial_close` are real booleans** (matched) OR `[]` (un-matched optional slot — consumer maps to false, same convention as `posix_negation`/`negated` in posix_class).
- **`body` is the raw `class_body` shape.** The `class_body` rule is `class_item*` — Quantified of class_items. Each class_item is one of `posix_class` (typed by slice 8), `class_range`, `quoted_class_literal`, `class_literal`, `class_escape`. The typed inner shapes propagate; `class_body` per-rule typing is its own concern.
- **Recommended RGX integration steps:**
  1. Update PGEN dependency to the post-`1.1.56` commit on `main`.
  2. Regenerate the regex parser via `make regex_parser` or `make regex_parser_fresh`.
  3. Update any code that walked the raw `["[", <negation?>, <initial_close?>, <class_body>, "]"]` shape to use typed `obj.kind == "char_class"` dispatch + `obj.negated` + `obj.initial_close` boolean reads + `obj.body` array iteration.
- Public API surface unchanged.
- Regex AST schema version stays `1`.
- **Atom subtree campaign progress:** 23/25 atom alternatives directly typed. Remaining 2 atom alternatives:
  - `conditional` (multi-branch with condition + yes_branch + no_branch).
  - `extended_class` (recursive nested classes).
  - (Plus the 3 deferred leaf-char alternatives — separate decision.)

## Release 1.1.55 / Contract 1.1.57 Highlights — atom subtree slice 25: scan_substring / script_run / subroutine_call typed

- **Internal-driven shape work** (no downstream report). Batched slice — 4 annotations across 3 atom alternatives.
- **Rules changed:**
  - `scan_substring_group = "(*" scan_substring_name ":" returned_capture_group_list pattern? ")" -> {type:"atom", kind:"scan_substring_group", name:$2, captures:$4, body:$5}`
  - `script_run_group = "(*" script_run_name ":" pattern? ")" -> {type:"atom", kind:"script_run_group", name:$2, body:$4}`
  - `subroutine_call` per-branch (both branches collapse to same kind):
    - Branch 0 (with returned captures): `"(?" returned_capture_subroutine ")" -> {type:"atom", kind:"subroutine_call", target:$2}`
    - Branch 1 (plain target): `"(?" subroutine_target ")" -> {type:"atom", kind:"subroutine_call", target:$2}`
- **AST shape change (consumer-visible):**
  - `(*sr:abc)` → `{kind:"script_run_group", name:"sr", body:<pattern>}`. `name` carries `sr`/`script_run`/`asr`/`atomic_script_run` directly.
  - `(*scs:(1)abc)` (with surrounding `(a)` capture so the validator accepts) → `{kind:"scan_substring_group", name:"scs", captures:<raw_capture_list>, body:<pattern>}`.
  - `(?&name)` → `{kind:"subroutine_call", target:["&", "name"]}`.
  - `(?P>name)` → `{kind:"subroutine_call", target:["P>", "name"]}`.
  - `(?R)` → `{kind:"subroutine_call", target:"R"}`.
  - `(?+1)` → `{kind:"subroutine_call", target:{sign:"+", value:1}}` — `signed_digits` already typed by slice 13 propagates through.
- **`subroutine_call` two-branch collapse:** Both branches produce `kind:"subroutine_call"` with `target` carrying the inner shape. Branch 0's `target` is a `returned_capture_subroutine` (subroutine_target + returned_capture_group_list); branch 1's `target` is just a subroutine_target. Consumer inspects `target` shape to determine the syntactic form.
- **Sub-rule shapes deferred:** `returned_capture_group_list`, `returned_capture_subroutine`, `subroutine_target` carry raw shapes. Per-rule typing is a separate concern.
- **Pre-existing host-validator note:** PGEN's host-side compile validator rejects scan_substring capture-list references that don't have a corresponding group in the surrounding pattern. The slice 25 annotation is correct when the validator allows the input through; for inputs the validator rejects, no AST is produced. Out of scope for this slice.
- Public API surface unchanged.
- Regex AST schema version stays `1`.
- **Atom subtree campaign progress:** 22/25 atom alternatives directly typed. Remaining 3 atom alternatives:
  - **Leaf chars (deferred — high-volume):** `literal`, `whitespace_literal`, `dot`.
  - **Recursive structures:** `char_class` outer, `conditional`, `extended_class`.
  - (Note: counted differently — the leaves are 3 separate alternatives but conceptually one decision; recursive structures are 3 separate alternatives.)

## Release 1.1.54 / Contract 1.1.56 Highlights — atom subtree slice 24: inline-modifier / callout / directive_verb / code_block typed

- **Internal-driven shape work** (no downstream report). Batched slice — 6 annotations across 5 atom alternatives (code_block has 2 branches both producing `kind:"code_block"`).
- **Rules changed:**
  - `inline_modifiers = "(?" modifier_spec? ")" -> {type:"atom", kind:"inline_modifiers", spec:$2}`
  - `scoped_inline_modifiers = "(?" modifier_spec ":" pattern? ")" -> {type:"atom", kind:"scoped_inline_modifiers", spec:$2, body:$4}`
  - `callout = "(?C" callout_arg? ")" -> {type:"atom", kind:"callout", arg:$2}`
  - `directive_verb = "(*" directive_body ")" -> {type:"atom", kind:"directive_verb", body:$2}`
  - `code_block_plain = "(?{" code_content "})" -> {type:"atom", kind:"code_block", lang:null, content:$2}`
  - `code_block_lang = "(?{" code_lang ":" ws? code_content "})" -> {type:"atom", kind:"code_block", lang:$2, content:$4}`
- **AST shape change (consumer-visible):**
  - `(?i)` → `{type:"atom", kind:"inline_modifiers", spec:[["i"], []]}`. `spec` carries raw `modifier_spec` shape (sub-rule typing is a separate concern).
  - `(?i:abc)` → `{type:"atom", kind:"scoped_inline_modifiers", spec:<modifier_spec>, body:<pattern>}`.
  - `(?C42)` → `{type:"atom", kind:"callout", arg:42}` (callout_arg's digits sub-rule is already typed-int via `@transform`).
  - `(*MARK:foo)` → `{type:"atom", kind:"directive_verb", body:<raw_directive_body>}`.
  - `(?{print})` → `{type:"atom", kind:"code_block", lang:null, content:<chars>}`. `lang:null` distinguishes plain form.
  - `(?{lua: print})` → `{type:"atom", kind:"code_block", lang:"lua", content:<chars>}`.
- **`code_block` two-branch collapse:** Both `code_block_plain` and `code_block_lang` produce `kind:"code_block"`, distinguished by `lang` (null vs string). Consumer always reads `obj.lang` and `obj.content` — no need to dispatch on which branch matched.
- **`spec`/`body`/`arg`/`content` carry raw shapes.** Sub-rule typing (modifier_spec, callout_arg, directive_body, code_content) is left to follow-up slices. Atom-level dispatch on `kind` is what slice 24 delivers.
- Public API surface unchanged.
- Regex AST schema version stays `1`.
- **Atom subtree campaign progress:** 19/25 atom alternatives directly typed (counting code_block as 1 atom alternative; the 2 branches collapse). Remaining ~6 atom alternatives: literal, whitespace_literal, dot, char_class outer, conditional, scan_substring_group, script_run_group, extended_class, subroutine_call.

## Release 1.1.53 / Contract 1.1.55 Highlights — atom subtree slice 23: lookaround family typed (7 sub-rules)

- **Internal-driven shape work** (no downstream report). Largest slice yet — 7 annotations across 7 lookaround sub-rules in one batch.
- **Rules changed:**
  - `lookahead_pos = "(?=" pattern ")" -> {type:"atom", kind:"lookahead", positive:true, body:$2}`
  - `lookahead_neg = "(?!" pattern ")" -> {type:"atom", kind:"lookahead", positive:false, body:$2}`
  - `lookbehind_pos = "(?<=" pattern ")" -> {type:"atom", kind:"lookbehind", positive:true, body:$2}`
  - `lookbehind_neg = "(?<!" pattern ")" -> {type:"atom", kind:"lookbehind", positive:false, body:$2}`
  - `non_atomic_lookahead_pos = "(?*" pattern ")" -> {type:"atom", kind:"non_atomic_lookahead", positive:true, body:$2}`
  - `non_atomic_lookbehind_pos = "(?<*" pattern ")" -> {type:"atom", kind:"non_atomic_lookbehind", positive:true, body:$2}`
  - `alpha_lookaround = "(*" alpha_lookaround_name ":" pattern? ")" -> {type:"atom", kind:"alpha_lookaround", name:$2, body:$4}`
- **AST shape change (consumer-visible):**
  - Before: `(?=foo)` → `["(?=", <pattern>, ")"]` (3-element Sequence).
  - After: `(?=foo)` → `{type:"atom", kind:"lookahead", positive:true, body:<pattern>}`.
- **`kind` + `positive` design:** `lookahead_pos`/`lookahead_neg` collapse to `kind:"lookahead"` with `positive:true`/`false` (consistent with property_escape's `negated` field convention). Same for `lookbehind`. Non-atomic forms get distinct `kind` values (`non_atomic_lookahead`/`non_atomic_lookbehind`) since PCRE2 only supports positive variants for them — no need for a `positive:true` boolean (it's always true), but the field is included for uniform consumer code.
- **Alpha-form `(*<name>:...)` carries `alpha_lookaround_name` in `name` field.** PCRE2 admits `pla` / `positive_lookahead` / `nla` / `negative_lookahead` / `plb` / `positive_lookbehind` / `nlb` / `negative_lookbehind` / `napla` / `non_atomic_positive_lookahead` / `naplb` / `non_atomic_positive_lookbehind`. Consumers map by `name` to dispatch on the semantic equivalent.
- **Recommended RGX integration steps:**
  1. Update PGEN dependency to the post-`1.1.53` commit on `main`.
  2. Regenerate the regex parser via `make regex_parser` or `make regex_parser_fresh`.
  3. Update any code that walked the raw `["(?=", <pattern>, ")"]` (or `(?!`/`(?<=`/`(?<!`/`(?*`/`(?<*`/`(*<name>:` equivalents) to use typed `obj.kind` dispatch + `obj.positive` (where present) + `obj.body` (always) + `obj.name` (alpha-form only) field reads.
- Public API surface unchanged.
- Regex AST schema version stays `1`.
- **Atom subtree campaign progress:** 14/25 atom alternatives directly typed (counting `lookaround` as 1 atom alternative now that all 7 sub-rules are typed via `lookaround`'s implicit `-> $1` propagation). Group typing end-to-end. Lookaround family typed end-to-end. 7/7 escape_unit branches typed.

## Release 1.1.52 / Contract 1.1.54 Highlights — atom subtree slice 22: named groups typed (named/python_named)

- **Internal-driven shape work** (no downstream report). Continues the group-typing campaign from slice 21.
- **Rules changed:**
  - `named_group` per-branch annotations producing typed `{type:"atom", kind:"named_group", name:<string>, body:<pattern>}`. Both `(?<name>...)` (angle) and `(?'name'...)` (quote) syntactic forms collapse to `kind:"named_group"`.
  - `python_named_group = "(?P<" name ">" pattern? ")" -> {type:"atom", kind:"python_named_group", name:<string>, body:<pattern>}`.
- **AST shape change (consumer-visible):**
  - Before: `(?<foo>abc)` → `["(?<", "foo", ">", <pattern>, ")"]` (5-element Sequence).
  - After: `(?<foo>abc)` → `{type:"atom", kind:"named_group", name:"foo", body:<pattern>}` (typed object).
- **`kind:"python_named_group"` distinct from `kind:"named_group"`** (paralleling slice 19's `python_named_backreference`/`backreference` distinction). PCRE2 treats them as functionally equivalent for matching, but the syntactic origin is preserved. Consumers normalizing across all name-based group forms: `kind in {"named_group", "python_named_group"}` → name-based group; `name` is the name string in both.
- **Recommended RGX integration steps:**
  1. Update PGEN dependency to the post-`1.1.52` commit on `main`.
  2. Regenerate the regex parser via `make regex_parser` or `make regex_parser_fresh`.
  3. Update any code that walked the raw `["(?<", "foo", ">", <pattern>, ")"]` (or `(?'...'` / `(?P<...>` equivalents) to use typed `obj.kind` dispatch + `obj.name` + `obj.body` field reads.
- Public API surface unchanged.
- Regex AST schema version stays `1`.
- **Atom subtree campaign progress:** 13/25 atom alternatives directly typed (capturing_group, noncapturing_group, named_group, python_named_group, branch_reset_group, atomic_group all counted via `group`'s `-> $1` propagation; the count is "atom alternatives that surface a typed atom shape"). All 6 group sub-rules (capturing/noncapturing/named/python_named under `group`; branch_reset and atomic standalone) are now typed. Group typing is now end-to-end.

## Release 1.1.51 / Contract 1.1.53 Highlights — atom subtree slice 21: simple groups typed (capturing/noncapturing/branch_reset/atomic)

- **Internal-driven shape work** (no downstream report). Largest atom-subtree slice yet — types 4 group forms (5 annotations counting atomic_group's 2 syntactic branches) in one pass.
- **Rules changed:**
  - `capturing_group = "(" pattern? ")" -> {type:"atom", kind:"capturing_group", body:$2}`.
  - `noncapturing_group = "(?:" pattern? ")" -> {type:"atom", kind:"noncapturing_group", body:$2}`.
  - `branch_reset_group = "(?|" pattern? ")" -> {type:"atom", kind:"branch_reset_group", body:$2}`.
  - `atomic_group = "(?>" pattern? ")" -> {type:"atom", kind:"atomic_group", body:$2}` and `"(*atomic:" pattern? ")" -> {type:"atom", kind:"atomic_group", body:$2}` — both syntactic forms produce the same typed shape (PCRE2 treats them as semantically equivalent).
- **AST shape change (consumer-visible):**
  - Before: `(abc)` → `["(", <pattern>, ")"]` (3-element Sequence).
  - After: `(abc)` → `{type:"atom", kind:"capturing_group", body:<pattern>}` (typed object).
- **`body` is the raw pattern shape**, NOT itself typed. Pattern outer typing is a separate slice (the `pattern → alternation → alternative → concatenation → piece+` chain has its own rule-typing story).
- **Empty groups** `()` / `(?:)` emit `body: [[], []]` (the empty alternation shape from `pattern? = alternation?` matched-empty).
- **Recommended RGX integration steps:**
  1. Update PGEN dependency to the post-`1.1.51` commit on `main`.
  2. Regenerate the regex parser via `make regex_parser` or `make regex_parser_fresh`.
  3. Update any code that walked the raw `["(", <pattern>, ")"]` (or sub-rule equivalents) to use typed `obj.kind` dispatch + `obj.body` field read for the pattern content.
- Public API surface unchanged.
- Regex AST schema version stays `1`.
- **Atom subtree campaign progress:** 11/25 atom alternatives directly typed (counting `capturing_group` and `noncapturing_group` separately even though both are sub-rules of `group`; the typing reaches `group`'s output transparently). 7/7 escape_unit branches typed; backreference family fully typed.

## Release 1.1.50 / Contract 1.1.52 Highlights — atom subtree slice 20: comment_group typed

- **Internal-driven shape work** (no downstream report).
- **Rules changed:**
  - `comment_group` annotated `-> {type:"atom", kind:"comment", text:$2}`.
  - `comment_text` rewritten from `comment_char*` (multi-element chain) to `/([^)]*)/` (regex literal) — emits a clean string Terminal of the comment body.
  - The `?` after `comment_text` in `comment_group` body is dropped (the regex literal accepts the empty match, so `?` was redundant).
- **AST shape change:** `(?#hello)` → `{type:"atom", kind:"comment", text:"hello"}` (was `["(?#", [<comment_char chain>], ")"]` 3-element Sequence).
- **Empty comment** `(?#)` now emits `text:""` (a real empty string), not `[]` from the un-matched `?` slot.
- **Char-set coverage:** `comment_text = /([^)]*)/` matches any char except `)`, which is the same semantic coverage as the previous `comment_char*` chain (which enumerated letter/digit/whitespace/special/unicode_char alternatives that together excluded `)`).
- Public API surface unchanged.
- Regex AST schema version stays `1`.
- **Atom subtree campaign progress:** 8/25 atom alternatives directly typed; 7/7 escape_unit branches typed.

## Release 1.1.49 / Contract 1.1.51 Highlights — atom subtree slice 19: python_named_backreference typed

- **Internal-driven shape work** (no downstream report).
- **Rule changed:** `python_named_backreference = "(?P=" name ")" -> {type:"backreference", kind:"python_named", ref:$2}`.
- **AST shape change:** `(?P=foo)` → `{type:"backreference", kind:"python_named", ref:"foo"}` (was `["(?P=", "foo", ")"]` 3-element Sequence; `name` was already a clean string after slice 11).
- **Why a separate `kind` value:** PCRE2 treats `(?P=foo)` as functionally equivalent to `\k<foo>` for matching purposes, but the syntax origin (Python-specific `(?P=...)` vs PCRE2 `\k<...>`) is preserved in `kind` for tooling. Consumers that don't care about syntax origin can normalize: `kind in {"named", "named_braced", "python_named"}` → "name-based backref"; `ref` is the name in all three.
- Public API surface unchanged.
- Regex AST schema version stays `1`.
- **Atom subtree campaign progress:** 7/25 atom alternatives directly typed; 7/7 escape_unit branches typed. Backreference family typing is now end-to-end across all 5 syntactic forms (numeric `\N`, named `\k<...>`/`\k'...'`, named_braced `\k{...}`, subroutine `\g<...>`/`\g'...'`/`\g{...}`/`\g+digit`, python_named `(?P=...)`).

## Release 1.1.48 / Contract 1.1.50 Highlights — atom subtree slice 18: quoted_literal typed

- **Internal-driven shape work** (no downstream report). First atom-subtree slice after the escape subtree closure.
- **Rule changed:**
  - `quoted_literal` annotated `-> {type:"atom", kind:"quoted_literal", body:$2}`. `body` is the array of `quoted_literal_char*` matched chars.
- **AST shape change (consumer-visible):**
  - Before: `\Qhello\E` → `["\\Q", ["h","e","l","l","o"], "\\E"]` (3-element Sequence).
  - After: `\Qhello\E` → `{type:"atom", kind:"quoted_literal", body:["h","e","l","l","o"]}` (typed object).
- **`body` is an array of single-char strings**, one per `quoted_literal_char` match. `quoted_literal_escaped_char` produces 2 chars (the `\` and the tail). Consumers that want the raw string join the array; consumers with semantic needs can distinguish escaped vs raw chars from the array element shapes (escaped chars stay as 2-char strings).
- **Recommended RGX integration steps:**
  1. Update PGEN dependency to the post-`1.1.48` commit on `main`.
  2. Regenerate the regex parser via `make regex_parser` or `make regex_parser_fresh`.
  3. Update any code that walked the raw `["\\Q", <chars>, "\\E"]` shape to use typed `obj.kind == "quoted_literal"` dispatch + `obj.body.join("")` to recover the literal string.
- Public API surface unchanged.
- Regex AST schema version stays `1`.
- **Atom subtree campaign progress:** 6/25 alternatives directly typed (+1 from slice 17 to count quoted_literal); 7/7 escape_unit branches typed (escape subtree closed). Remaining atom alternatives: literal, whitespace_literal, dot, char_class outer, group/conditional/lookaround/atomic_group/scan_substring_group/script_run_group/inline_modifiers/scoped_inline_modifiers/branch_reset_group/callout/directive_verb/extended_class/code_block/comment_group/python_named_backreference/group/subroutine_call.

## Release 1.1.47 / Contract 1.1.49 Highlights — atom subtree slice 17: escape subtree closes (property)

- **Internal-driven shape work** (no downstream report). Closes the escape-subtree typed-shape campaign — all 7 `escape_unit` branches are now typed.
- **Rules changed:**
  - `property_escape` per-branch annotations producing typed `{type:"escape", kind:"property", name:<string>, negated:<bool>}`.
  - `prop_name` rewritten from `prop_name_chars+` chain to `/([A-Za-z0-9 \t\n\r\f\v_:\-=&^]+)/` regex literal — emits clean string Terminal.
  - `short_prop_letter` rewritten from Or-of-single-chars chain to `/([CLMNPSZclmnpsz])/` regex literal.
- **AST shape change (consumer-visible):** PCRE2 `\p{...}`/`\P{...}` (braced) and `\pX`/`\PX` (short) atoms now emit typed `{type, kind, name, negated}` objects. `negated` is a real boolean (`true` for `\P` forms, `false` for `\p` forms) rather than presence-of-`P` structural inference.
  - Before: `\p{Lu}` → `["\\", ["p{", [["L"], ["u"]], "}"]]` (3-level chain with multi-element prop_name).
  - After: `\p{Lu}` → `{type:"escape", kind:"property", name:"Lu", negated:false}` (typed object).
- **Recommended RGX integration steps:**
  1. Update PGEN dependency to the post-`1.1.47` commit on `main`.
  2. Regenerate the regex parser via `make regex_parser` or `make regex_parser_fresh`.
  3. Update any code that walked the raw `["p{", <prop_name chain>, "}"]` shape or that inferred negation from the leading `"p"` vs `"P"` token to use typed `obj.kind == "property"` dispatch + `obj.name` field read + `obj.negated` boolean.
- Public API surface unchanged.
- Regex AST schema version stays `1`.
- **Atom subtree campaign progress:** 5/25 alternatives directly typed; **7/7 escape_unit branches typed** (single_byte, simple, control, hex, unicode, octal, property). The escape subtree is now closed. Remaining atom alternatives: literal, whitespace_literal, dot, quoted_literal, char_class outer, group/modifier/conditional/lookaround/atomic_group/scan_substring_group/script_run_group/directive_verb/extended_class/code_block/comment_group/python_named_backreference/group/inline_modifiers/scoped_inline_modifiers/branch_reset_group/callout/subroutine_call.

## Release 1.1.46 / Contract 1.1.48 Highlights — atom subtree slice 16: escape subtree continues (octal)

- **Internal-driven shape work** (no downstream report). Continues the escape-subtree typed-shape campaign from slice 15.
- **Rules changed:**
  - `octal_escape` per-branch annotations producing typed `{type:"escape", kind:"octal", digits:<octal-string>}`.
  - New `octal_escape_short_payload = /([0-7]{1,3})/` regex literal for the bare 1-3-digit form.
  - `octal_digits` rewritten from `octal_digit+` chain to `/([0-7]+)/` regex literal.
  - Removed duplicate inline `octal_digits` definition near `octal_escape` (was defined twice; kept the canonical definition in the char-categories section).
- **AST shape change (consumer-visible):**
  - Braced `\o{NNN...}` → `{type:"escape", kind:"octal", digits:"NNN..."}`.
  - Bare `\NNN` *inside character classes* (where the `class_range_escape_unit` path reaches the bare octal branch) → `{type:"escape", kind:"octal", digits:"NNN"}`.
  - Bare `\NNN` at atom-level remains parsed as `backreference` (`{type:"backreference", kind:"numeric", index:NNN}`) under the existing PEG ordering — the numeric backref branch shadows the bare-octal branch in `atom`. Pre-existing behavior; not changed by this slice. Disambiguating numeric-backref vs bare-octal at atom-level requires PCRE2-style contextual logic ("if NNN ≤ 9 OR there are NNN capture groups, treat as backref; else octal") that PEG cannot express directly — separate slice if/when warranted.
- **`digits` is a string, not an int.** Consumers parse with `usize::from_str_radix(digits, 8)`. Same constraint as hex/unicode; extending `@transform` to support `from_str_radix` is a separate codegen-feature slice.
- **Recommended RGX integration steps:**
  1. Update PGEN dependency to the post-`1.1.46` commit on `main`.
  2. Regenerate the regex parser via `make regex_parser` or `make regex_parser_fresh`.
  3. Update any code that walked the raw `["o{", ..., <digits chain>, ..., "}"]` shape for braced-octal escapes to use typed `obj.kind == "octal"` dispatch + `obj.digits` field read. Parse with `usize::from_str_radix(obj.digits, 8)`.
- Public API surface unchanged.
- Regex AST schema version stays `1`.
- Atom subtree campaign progress: 5/25 alternatives directly typed; 6/7 escape_unit branches typed. Only remaining un-typed escape_unit branch: `property_escape`.

## Release 1.1.45 / Contract 1.1.47 Highlights — atom subtree slice 15: escape subtree continues (hex/unicode)

- **Internal-driven shape work** (no downstream report). Continues the escape-subtree typed-shape campaign from slice 14.
- **Rules changed:**
  - `hex_escape` per-branch annotations producing typed `{type:"escape", kind:"hex", digits:<hex-string>}`.
  - `unicode_escape` annotated `-> {type:"escape", kind:"unicode", digits:<hex-string>}`.
  - New `hex_escape_short_payload = /([0-9A-Fa-f]{1,2})/` regex literal for the short `\xNN` form.
  - `hex_digits` rewritten from `hex_digit+` (multi-element chain) to `/([0-9A-Fa-f]+)/` (regex literal emitting clean string Terminal).
- **AST shape change (consumer-visible):** PCRE2 `\xF`/`\xFF`/`\x{NNN...}` and `\u{NNN...}` atoms now emit typed `{type, kind, digits}` objects.
  - Before: `\xFF` → `["\\", ["x", "F", [["F"]]]]` (4-level chain).
  - After: `\xFF` → `{type:"escape", kind:"hex", digits:"FF"}` (typed object).
- **`digits` is a string, not an int.** Consumers parse with `usize::from_str_radix(digits, 16)`. The rule's `@transform` is currently hard-coded to `str::parse::<TYPE>().unwrap_or(DEFAULT)`-style which doesn't accommodate `from_str_radix(s, 16)`. Extending the transform machinery is a separate codegen-feature slice.
- **`\u{...}` validator note:** PGEN's host-side compile validator currently rejects `\u{...}` escapes ("unsupported regex escape `\u`"). The annotation IS in place and works correctly when the validator permits the escape; for inputs the validator rejects, no AST is produced. That validator behavior is pre-existing and out of scope for this slice.
- **Recommended RGX integration steps:**
  1. Update PGEN dependency to the post-`1.1.45` commit on `main`.
  2. Regenerate the regex parser via `make regex_parser` or `make regex_parser_fresh`.
  3. Update any code that walked the raw `["\\", ["x", ...]]` shape for hex escapes to use typed `obj.kind == "hex"` dispatch + `obj.digits` field read. Parse with `usize::from_str_radix(obj.digits, 16)`.
- Public API surface unchanged.
- Regex AST schema version stays `1`.
- Atom subtree campaign progress: 5/25 alternatives directly typed; 5/7 escape_unit branches typed.

## Release 1.1.44 / Contract 1.1.46 Highlights — atom subtree slice 14: escape subtree starts (simple/single_byte/control)

- **Internal-driven shape work** (no downstream report). First slice of the escape-subtree typed-shape campaign.
- **Rules changed:** `escape` and 3 of its 7 `escape_unit` branches in `grammars/regex.ebnf`.
  - `escape = "\\" escape_unit -> $2` — transparent wrapper.
  - `single_byte_escape -> {type:"escape", kind:"single_byte"}` — `\C`.
  - `simple_escape -> {type:"escape", kind:"shorthand", char:$1}` — `\d`/`\w`/`\s`/`\.`/`\\`/etc.
  - `control_escape -> {type:"escape", kind:"control", char:$2}` — `\cA`/`\cZ`/`\cz`.
- **AST shape change (consumer-visible):** the most common escape forms (shorthand classes like `\d`/`\w`/`\s`, escaped metacharacters like `\.`/`\\`, single-byte `\C`, control escapes `\cX`) now emit a typed `{type:"escape", kind:<form>, char?}` object directly.
  - Before: `\d` → atom is `["\\", [[[[[ "d" ]]]]]]` (5-level un-annotated chain).
  - After: `\d` → atom is `{"type":"escape","kind":"shorthand","char":"d"}` (typed object, single field read).
- **Limitation — 4 escape_unit branches still raw.** Hex (`\xFF`/`\x{...}`), unicode (`\u{...}`), octal (`\377`/`\o{...}`), and property (`\p{...}`/`\PL`) escapes still emit their pre-fix raw shapes. Each requires digit-decoding or property-name extraction. Follow-up slices will type them one by one.
- **Recommended RGX integration steps:**
  1. Update PGEN dependency to the post-`1.1.44` commit on `main`.
  2. Regenerate the regex parser via `make regex_parser` or `make regex_parser_fresh`.
  3. Update any code that walked the `["\\", <inner-chain>]` shape for shorthand/control/single_byte escapes to use typed `obj.kind` dispatch + `obj.char` field read.
  4. Keep the existing raw-walking path for hex/unicode/octal/property until follow-up slices type them.
- Public API surface unchanged.
- Regex AST schema version stays `1`.
- Atom subtree campaign progress: 5/25 alternatives directly typed (anchor, posix_class, posix_word_boundary_alias, backreference, escape outer). 3/7 escape_unit branches typed.

## Release 1.1.43 / Contract 1.1.45 Highlights — atom subtree slice 13: signed_digits typing (backref family fully typed end-to-end)

- **Internal-driven shape work** (no downstream report). Direct follow-up to slices 11+12; types the last raw shape in the backref family.
- **Rule changed:** `signed_digits` in `grammars/regex.ebnf`. Annotated `-> {sign: $1, value: $2}` so numeric subroutine refs surface a typed `{sign, value}` object.
- **AST shape change (consumer-visible):** numeric subroutine refs (`\g<1>`, `\g+1`, `\g-3`, `\g42`, etc.) now produce `ref:{sign:..., value:...}` typed objects.
  - Before: `\g<-2>` → `ref:["-", 2]`. After: `ref:{"sign":"-","value":2}`.
  - Before: `\g+1` → `ref:["+", 1]`. After: `ref:{"sign":"+","value":1}`.
  - Before: `\g42` → `ref:[[], 42]`. After: `ref:{"sign":[],"value":42}`.
- **`sign` convention:** `"+"` or `"-"` when matched, `[]` (empty array — un-matched `Quantified-?` slot) when no sign was present. Consumers map `[]` → `null`/unsigned. Same convention as `quantifier.greediness` and `posix_class.negated`. Future coalesce-operator slice will let the rule emit `null` directly.
- **Backreference family typed end-to-end.** All shapes are field-readable:
  - `kind:"numeric"` → read `obj.index` as integer.
  - `kind:"named"` / `kind:"named_braced"` → read `obj.ref` as string.
  - `kind:"subroutine"` → read `obj.ref` as string (named) OR `obj.ref.sign` + `obj.ref.value` (numeric).
- **Recommended RGX integration steps:**
  1. Update PGEN dependency to the post-`1.1.43` commit on `main`.
  2. Regenerate the regex parser via `make regex_parser` or `make regex_parser_fresh`.
  3. Update any code that walked the `[<sign?>, <int>]` 2-element array shape for numeric subroutine refs to read `obj.ref.sign` and `obj.ref.value` directly.
- Public API surface unchanged.
- Regex AST schema version stays `1`.

## Release 1.1.42 / Contract 1.1.44 Highlights — atom subtree slice 12: subroutine_ref cleanup (closes backref family)

- **Internal-driven shape work** (no downstream report). Direct follow-up to slice 11 — closes the named/subroutine-reference family typing started in slice 10.
- **Rules changed:**
  - `subroutine_ref` in `grammars/regex.ebnf` — 4 per-branch annotations (`-> $1` / `-> $2` / `-> $2` / `-> $1`) to drop the angle/quote/brace delimiters and surface the inner `signed_digits_or_name` directly.
  - `braced_subroutine_ref` in `grammars/regex.ebnf` — annotated `-> $3` to extract from `{...}` wrappers.
- **AST shape change (consumer-visible):** `\g<...>` family backreferences now surface clean inner values:
  - `\g<name>` → `ref:"name"` (was `["<", "name", ">"]`).
  - `\g{42}` → `ref:[[], 42]` (was `["{", _, [[], 42], _, "}"]`).
  - `\g<-2>` → `ref:["-", 2]` (was `["<", ["-", 2], ">"]`).
  - `\g+1` → `ref:["+", 1]` (already raw signed_digits, unchanged).
- **Backreference family typing closed.** All 4 backreference kinds now produce clean inner values:
  - `kind:"named"` / `kind:"named_braced"` → `ref:<string>`.
  - `kind:"numeric"` → `index:<int>` (typed integer).
  - `kind:"subroutine"` → `ref:<string>` (named form) OR `[<sign?>, <digit-int>]` (numeric form).
- **Limitation — `signed_digits` still raw.** `signed_digits = sign? digits` is un-annotated; numeric subroutine refs surface as `[<sign?-Quantified>, <typed-int>]`. Consumer dispatch by `ref.is_string()` (named) vs `ref.is_array()` (numeric). Future sub-slice will type `signed_digits` to `{sign:<"+"|"-"|null>, value:<int>}` for cleaner ergonomics.
- **Recommended RGX integration steps:**
  1. Update PGEN dependency to the post-`1.1.42` commit on `main`.
  2. Regenerate the regex parser via `make regex_parser` or `make regex_parser_fresh`.
  3. Update any code that walked subroutine_ref's wrappers (`["<", <inner>, ">"]` etc.) to read the inner directly. For named refs: `obj.ref.as_str()`. For numeric refs: walk `obj.ref.as_array()` for `[<sign>, <int>]`.
- Public API surface unchanged.
- Regex AST schema version stays `1`.

## Release 1.1.41 / Contract 1.1.43 Highlights — atom subtree slice 11: named-ref cleanup (clean name strings)

- **Internal-driven shape work** (no downstream report). Direct follow-up to slice 10's `backreference` typing — surfaces clean name strings throughout the named-reference family.
- **Rules changed:**
  - `name` in `grammars/regex.ebnf` — rewritten from `name_start name_continue*` (multi-element chain producing `[first_char, [rest_chars]]`) to a single regex literal `/((?:[A-Za-z_]|[^\x00-\x7F])(?:[A-Za-z0-9_]|[^\x00-\x7F])*)/` that emits the matched name string as a Terminal directly. `name_start`/`name_continue`/`letter`/`digit`/`unicode_char` sub-rules retained for compatibility but no longer participate in `name`'s body.
  - `name_ref` in `grammars/regex.ebnf` — both branches annotated `-> $2` to extract just the matched name (drops the `<...>` / `'...'` wrappers).
  - `braced_name_ref` in `grammars/regex.ebnf` — annotated `-> $3` to extract from `{...}` wrappers.
- **AST shape change (consumer-visible):** every consumer of `name` now sees a clean name string instead of a character-chain Sequence.
  - `\k<foo>` → `{type:"backreference", kind:"named", ref:"foo"}` (was `ref:["<", <chain>, ">"]`).
  - `\k{foo}` → `{type:"backreference", kind:"named_braced", ref:"foo"}` (was `ref:["{", _, <chain>, _, "}"]`).
  - `(?<foo>...)`, `(?P<foo>...)`, `(?'foo'...)`, `(?P=foo)` — every named group form's `name` slot is now a clean string.
  - Subroutine targets (`(?&name)`, `(?P>name)`) — `name` slot is now clean.
  - Conditions (`(?(name)...)`, `(?(R&name)...)`) — `name` slot is now clean.
- **`subroutine_ref` still un-annotated.** `\g<name>` etc. emit `ref: ["<", "name", ">"]` (the inner `name` is clean, but the angle/brace/quote wrappers remain). Follow-up slice will type `subroutine_ref` to drop the wrappers.
- **Recommended RGX integration steps:**
  1. Update PGEN dependency to the post-`1.1.41` commit on `main`.
  2. Regenerate the regex parser via `make regex_parser` or `make regex_parser_fresh`.
  3. Update any code that walked the raw `name` character chain to read the matched value as a string directly. For backreferences: `obj.kind == "named"` → read `obj.ref.as_str()`. For named groups: read the `name` field of the group atom directly as a string.
  4. For subroutine refs (`\g<name>` etc.), inner `name` is now a clean string within the otherwise-raw `["<", <inner>, ">"]` shape; consumers walking subroutine refs benefit from the partial cleanup but full typing waits on the next slice.
- Public API surface unchanged: `RegexParser::new`, `parser.parse_full_regex()`, `parser.parse_regex()`, `parse_regex_typed()`, `parse_regex_default_ast_dump_named()` keep the same signatures. `ParseNode` and `ParseContent` enum unchanged.
- Regex AST schema version stays `1`.

## Release 1.1.40 / Contract 1.1.42 Highlights — PGEN-RGX-0077 typed-shape fix for `\Q...\E quantifier?` pieces

- **Driven by RGX bug report PGEN-RGX-0077** (`/Users/richarddje/Documents/github/rgx/pgen-issues/PGEN-RGX-0077.yaml`).
- **What was wrong:** for `\Q...\E quantifier?` matched runs, `pattern[0][0]` carried one extra wrapping layer — `[[<N pieces>]]` (1-element array containing the pieces array) instead of the documented flat `[<N pieces>]`. The PCRE2 family-table from the PGEN-RGX-0074 fix (`\Qab*\E{2,}` → 3 pieces, `\Qabc\E?` → 3 pieces, etc.) was structurally correct but consumers walking `pattern[0][0]` saw a 1-element array. RGX's typed-shape walker surfaced this as `pgen AST contract mismatch: expected typed piece object, got array`.
- **Root cause and scope:** the `[$1**]` flatten-spread codegen in `rust/src/ast_pipeline/ast_return_transform.rs` did not peel `Alternative` wrapping before inspecting child content for the unwrap decision. The codegen wraps Or-rule and rule-reference branch results in `Alternative(boxed_inner)`; for `concatenation = piece+ -> [$1**]`, each piece node arrives as `Alternative(piece_inner_node)`. Pre-fix, the inner `match node.content` saw `Alternative` and fell into the `other_content` "push as-is" arm, wrapping the whole Sequence-of-pieces (from `piece_quoted_run_quantified -> [$2**, ...]`) as a single element instead of spreading. Adjacent regression to PGEN-RGX-0075 (which fixed `$1`-on-Quantified auto-peel) on a different codegen path.
- **Fix:** the FlattenSpread codegen now peels `Alternative` recursively before inspecting child content. Also adds a `ParseContent::Json(Value::Array(_))` arm in case a future annotation produces a typed-Json array directly (preventative; not exercised by current grammar but guards against the same family of regressions for any rule that builds `[$N**, ...]` shapes).
- **AST shape change (consumer-visible):** every multi-char `\Q...\E quantifier?` now produces a flat `[<N pieces>]` typed array at `pattern[0][0]`. Empirical proof from the bug-report family table:
  - `\Qab*\E{2,}` → 3 pieces (a, b, *) flat. The trailing `*` carries `quantifier:{type:"quantifier",min:2,max:null,greediness:[]}` — PGEN-RGX-0074's "quantifier binds to last char" doctrine preserved.
  - `\Qab\E{3}` → 2 pieces (a, b{3}) flat.
  - `\Qabcdef\E+` → 6 pieces (a-f) flat.
  - `\Qa\E{3}` and `\Q\E{2}` (degenerate cases) unchanged — they hit the atom-fallback path, not `piece_quoted_run_quantified`, so they were never affected by the bug.
- **Regression-lock test:** `regex_parser_pgen_rgx_0077_quoted_run_quantified_pieces_flat_in_concatenation` in `rust/src/embedding_api.rs` pins the family-table coverage from the bug report (9 multi-char `\Q...\E quantifier?` shapes). Asserts piece count + atom values + quantifier-attached-to-last-piece + no-quantifier-on-inner-pieces. The bug cannot regress without the test failing.
- **Recommended RGX integration steps:**
  1. Update PGEN dependency to the post-`1.1.40` commit on `main`.
  2. Regenerate the regex parser via `make regex_parser` or `make regex_parser_fresh`.
  3. RGX's typed-shape walker should now see flat piece arrays at `pattern[0][0]` for every `\Q...\E quantifier?` source — no adapter changes needed if RGX was already walking the documented flat shape; the bug surface goes away.
- Public API surface unchanged: `RegexParser::new`, `parser.parse_full_regex()`, `parser.parse_regex()`, `parse_regex_typed()`, `parse_regex_default_ast_dump_named()` keep the same signatures. `ParseNode` and `ParseContent` enum unchanged.
- Regex AST schema version stays `1`.

## Release 1.1.39 / Contract 1.1.41 Highlights — atom subtree slice 10: typed `backreference` shape

- **Internal-driven shape work** (no downstream report). Continues the atom-subtree campaign.
- **Rules changed:**
  - `backreference` in `grammars/regex.ebnf` — 4 per-branch annotations producing typed `{type:"backreference", kind:<form>, ...}` objects. Forms: `numeric` (`\1`, `\23`), `named` (`\k<name>`/`\k'name'`), `named_braced` (`\k{name}`), `subroutine` (`\g<...>`/`\g'...'`/`\g{...}`/`\g+1`/`\g42`).
  - `backreference_digits` in `grammars/regex.ebnf` — rewritten from `nonzero_digit digit*` to a regex literal `/([1-9][0-9]*)/` with `@transform: str::parse::<usize>().unwrap_or(0)`, so the rule emits a typed integer directly. Mirrors how `digits` was typed in slice 1.
- **AST shape change (consumer-visible):** every backreference atom is now a typed `{type, kind, ...}` object instead of a 2-element `["\\<prefix>", <inner>]` array.
  - Before: `\1` → `["\\", ["1"]]`; `\23` → `["\\", ["2", ["3"]]]`; `\k<foo>` → `["\\k", ["<", <name-chain>, ">"]]`.
  - After: `\1` → `{"type":"backreference","kind":"numeric","index":1}` (typed integer); `\23` → `{"type":"backreference","kind":"numeric","index":23}`; `\k<foo>` → `{"type":"backreference","kind":"named","ref":<raw name_ref shape>}`.
- **Limitation (deliberate scope):** for branches 1-3 (`named` / `named_braced` / `subroutine`), the `ref` field carries the inner sub-rule's RAW shape — `name_ref`, `braced_name_ref`, and `subroutine_ref` are still un-annotated. Consumers walking the name string need to descend the raw chain. A follow-up slice will type those rules so `ref` becomes a typed `{name: <str>}` (for named) or `{kind:..., value:...}` (for subroutine refs).
- **Recommended RGX integration steps:**
  1. Update PGEN dependency to the post-`1.1.39` commit on `main`.
  2. Regenerate the regex parser via `make regex_parser` or `make regex_parser_fresh`.
  3. Update any code that pattern-matched on the `["\\", <inner>]` 2-tuple at backreference positions to use the typed `obj.kind` dispatch. The numeric form's `index` is now a typed integer ready to use; the named/subroutine forms still require walking `ref` (raw inner shape) for now.
- Public API surface unchanged: `RegexParser::new`, `parser.parse_full_regex()`, `parser.parse_regex()`, `parse_regex_typed()`, `parse_regex_default_ast_dump_named()` keep the same signatures. `ParseNode` and `ParseContent` enum unchanged.
- Regex AST schema version stays `1`.
- Atom subtree campaign progress: 4 of 25 alternatives annotated (anchor, posix_class, posix_word_boundary_alias, backreference).

## Release 1.1.38 / Contract 1.1.40 Highlights — atom subtree slice 9: typed `posix_word_boundary_alias` (closes anchor family)

- **Internal-driven shape work** (no downstream report). Closes the anchor-family typing started in slice 7.
- **Rule changed:** `posix_word_boundary_alias` in `grammars/regex.ebnf`. The 2 branches each got a per-branch `-> {type: "anchor", kind: "<name>"}` annotation. PCRE2's BSD-style word-boundary aliases `[[:<:]]` and `[[:>:]]` now emit the same typed anchor shape as the regular `anchor` rule.
- **AST shape change (consumer-visible):** `[[:<:]]` and `[[:>:]]` atoms emit typed `{type:"anchor", kind:<name>}` objects instead of bare 7-char terminal strings.
  - Before: `[[:<:]]foo[[:>:]]` → `[<bare-string-anchor>, "f", "o", "o", <bare-string-anchor>]`.
  - After: `[[:<:]]foo[[:>:]]` → 5 pieces with anchors as `{"type":"anchor","kind":"posix_word_start"}` and `{"type":"anchor","kind":"posix_word_end"}`.
- **Anchor family closed.** All 11 anchor variants — 9 from `anchor` (slice 7) + 2 from `posix_word_boundary_alias` (this slice) — now emit the same typed `{type:"anchor", kind:<name>}` shape:
  - `^` → `start_of_line`, `$` → `end_of_line`, `\A` → `start_of_input`, `\Z` → `end_of_input_or_before_last_newline`, `\z` → `end_of_input`, `\b` → `word_boundary`, `\B` → `non_word_boundary`, `\G` → `match_start`, `\K` → `keep_out`, `[[:<:]]` → `posix_word_start`, `[[:>:]]` → `posix_word_end`.
- **Recommended RGX integration steps:**
  1. Update PGEN dependency to the post-`1.1.38` commit on `main`.
  2. Regenerate the regex parser via `make regex_parser` or `make regex_parser_fresh`.
  3. Update any code that pattern-matched on the raw `"[[:<:]]"` / `"[[:>:]]"` 7-char terminal strings to use the typed kind dispatch (`obj.get("kind").as_str() == "posix_word_start"` etc.). Drop any string-match fallback.
- Public API surface unchanged: `RegexParser::new`, `parser.parse_full_regex()`, `parser.parse_regex()`, `parse_regex_typed()`, `parse_regex_default_ast_dump_named()` keep the same signatures. `ParseNode` and `ParseContent` enum unchanged.
- Regex AST schema version stays `1`.
- Atom subtree campaign progress: 3 of 25 alternatives annotated (anchor, posix_class, posix_word_boundary_alias). Anchor family is now closed; remaining work covers `literal`, `whitespace_literal`, `dot`, `backreference`, `quoted_literal`, `escape`, `char_class` (outer rule), and the group/modifier/conditional/lookaround families.

## Release 1.1.37 / Contract 1.1.39 Highlights — PGEN-RGX-0076 typed-shape fix for `posix_class`

- **Driven by RGX bug report PGEN-RGX-0076** (`/Users/richarddje/Documents/github/rgx/pgen-issues/PGEN-RGX-0076.yaml`).
- **What was wrong:** the `posix_class` rule used a placeholder annotation `-> $1` which extracted only the first element of the rule body (the literal `"[:"` opener), silently discarding the matched POSIX class name and the optional `posix_negation`. Every POSIX class inside `[...]` collapsed to the same string `"[:"` in the typed AST. RGX could not distinguish `[:alpha:]` from `[:digit:]`; multi-class bodies like `[[:alpha:][:digit:]]` produced two identical-looking truncated entries.
- **Root cause and scope:** mostly a grammar-side fix (the annotation was a known placeholder per the regex-parser-book inline acknowledgment). Surfaced two latent codegen bugs in `rust/src/ast_pipeline/ast_return_transform.rs`:
  - `BooleanLiteral` rule-level scalar path emitted `ParseContent::Terminal(<bool_str>)` (a string Terminal `"true"`) instead of `ParseContent::Json(serde_json::Value::Bool(...))`. Detected because `posix_negation -> true` produced the string `"true"` initially.
  - `NumberLiteral` rule-level scalar path had the analogous bug (preventative fix; no current grammar exercises it).
- **Fix:** grammar annotation upgraded from placeholder to typed object construction, and codegen now emits typed `Json(Bool)` / `Json(Number)` for rule-level scalar literals.
  ```ebnf
  posix_class = "[:" posix_negation? posix_name ":]"
  -> {type: "posix_class", name: $3, negated: $2}
  posix_negation = "^" -> true
  ```
- **AST shape change (consumer-visible):** every `posix_class` element inside a character class body now emits a typed `{type, name, negated}` object.
  - Before: `[[:alpha:]]` → `class_body[0] = "[:"`.
  - After: `[[:alpha:]]` → `class_body[0] = {"type":"posix_class","name":"alpha","negated":[]}`.
  - Negated case: `[[:^alpha:]]` → `class_body[0] = {"type":"posix_class","name":"alpha","negated":true}`.
  - Multi-class: `[[:alpha:][:digit:]]` → 2-element class_body, both POSIX classes typed and disambiguated.
- **`negated` convention:** typed boolean `true` when `^` matched, empty array `[]` when un-matched (the `posix_negation?` slot's runtime shape). Consumers map `[]` → `false`. Same convention as `quantifier.greediness`. A future coalesce-operator slice will let the rule emit a bare `false` directly.
- **Recommended RGX integration steps:**
  1. Update PGEN dependency to the post-`1.1.37` commit on `main`.
  2. Regenerate the regex parser via `make regex_parser` or `make regex_parser_fresh`.
  3. Update RGX's adapter to read `class_item.get("type") == "posix_class"`, then `class_item.get("name").as_str()` for the POSIX name and `class_item.get("negated").as_bool().unwrap_or(false)` for the negation flag. Drop any source-span fallback used to recover the POSIX name.
  4. RGX's regression-pin tests `tests::ucp_pragma_unicodefies_posix_classes` and `tests::ucp_graph_includes_format_and_private_use` should pass after the bump.
- Public API surface unchanged: `RegexParser::new`, `parser.parse_full_regex()`, `parser.parse_regex()`, `parse_regex_typed()`, `parse_regex_default_ast_dump_named()` keep the same signatures. `ParseNode` and `ParseContent` enum unchanged.
- Regex AST schema version stays `1`.

## Release 1.1.36 / Contract 1.1.38 Highlights — atom subtree slice 7: typed `anchor` shape

- **Internal-driven shape work** (no downstream report). First slice of the atom-subtree typed-shape campaign.
- **Rule changed:** `anchor` in `grammars/regex.ebnf`. The 9 branches each got a per-branch `-> {type: "anchor", kind: "<name>"}` annotation. Previously the rule emitted a `Terminal` of the matched escape text; now it emits a typed object with the semantic anchor kind directly.
- **AST shape change (consumer-visible):** every piece atom for an anchor (`^`, `$`, `\A`, `\Z`, `\z`, `\b`, `\B`, `\G`, `\K`) is now a typed `{type:"anchor", kind:"<name>"}` object instead of a bare escape string.
  - Before: `^foo$` → `[anchor "^", literal "f", literal "o", literal "o", anchor "$"]` with anchors as `"^"` / `"$"` strings.
  - After: `^foo$` → same 5 pieces but anchors are `{"type":"anchor","kind":"start_of_line"}` and `{"type":"anchor","kind":"end_of_line"}`.
- **Anchor kind names** are stable identifiers, not the raw escape text:
  - `^` → `start_of_line`
  - `$` → `end_of_line`
  - `\A` → `start_of_input`
  - `\Z` → `end_of_input_or_before_last_newline`
  - `\z` → `end_of_input`
  - `\b` → `word_boundary`
  - `\B` → `non_word_boundary`
  - `\G` → `match_start`
  - `\K` → `keep_out`
- **`posix_word_boundary_alias` NOT YET typed.** The POSIX aliases `[[:<:]]` and `[[:>:]]` (handled by a separate rule) still emit raw 7-char terminals. They will join the typed `anchor` family in a follow-up slice. For now consumers walking those need a fallback string-match.
- **Recommended RGX integration steps:**
  1. Update PGEN dependency to the post-`1.1.36` commit on `main`.
  2. Regenerate the regex parser via `make regex_parser` or `make regex_parser_fresh`.
  3. Update any code that pattern-matched on the raw anchor strings (`atom.as_str() == "\\b"` etc.) to use the typed kind dispatch (`obj.get("kind").as_str() == "word_boundary"`).
  4. Keep the `[[:<:]]` / `[[:>:]]` string-match path for now — it'll converge with the typed family in a later slice.
- Public API surface unchanged: `RegexParser::new`, `parser.parse_full_regex()`, `parser.parse_regex()`, `parse_regex_typed()`, `parse_regex_default_ast_dump_named()` keep the same signatures. `ParseNode` and `ParseContent` enum unchanged.
- Regex AST schema version stays `1`.
- This is slice **7 of N** in the broader regex.ebnf annotation campaign (task #40). The quantifier subtree closed in slice 6; the atom subtree starts here.

## Release 1.1.35 / Contract 1.1.37 Highlights — quantifier-subtree typed-shape closure (slice 6/N)

- **Internal-driven shape work** (no downstream report). Final slice of the quantifier-subtree typed-shape campaign. Consolidates and closes the subtree.
- **Rules changed:**
  - `quant_base` in `grammars/regex.ebnf` — per-branch annotations replaced positional-passthrough with typed `{min, max}` for every alternative. Shorthand quantifiers expand to PCRE2-equivalent bounds (`*` → `{0,null}`, `+` → `{1,null}`, `?` → `{0,1}`); the counted-quantifier branch passes through via `$1`.
  - `quantifier` in `grammars/regex.ebnf` — new annotation `-> {type: "quantifier", min: $1.min, max: $1.max, greediness: $2}` produces a typed shape directly.
- **AST shape change (consumer-visible):** the piece's `quantifier` field is now a fully typed object instead of a `[<base>, <suffix>]` 2-tuple.
  - Before: `a*` → `quantifier: ["*", []]`; `a{2,5}` → `quantifier: [{"min":2,"max":5}, []]`; `a+?` → `quantifier: ["+", "lazy"]`.
  - After: `a*` → `quantifier: {"type":"quantifier","min":0,"max":null,"greediness":[]}`; `a{2,5}` → `quantifier: {"type":"quantifier","min":2,"max":5,"greediness":[]}`; `a+?` → `quantifier: {"type":"quantifier","min":1,"max":null,"greediness":"lazy"}`.
  - Pieces with no quantifier still have `quantifier: []` (empty `quantifier?` slot — unchanged).
- **Greediness convention:** `greediness` carries `"lazy"` (when source has `?` suffix) or `"possessive"` (when `+` suffix); the un-matched `quant_suffix?` slot surfaces as `[]` (empty array). Consumers MUST map `[]` → `"greedy"` (the PCRE2 default). A future slice will introduce a coalesce operator in the annotation language so `greediness: $2 ?? "greedy"` becomes a literal `"greedy"` string directly. Until then, this is a one-line consumer-side mapping.
- **Quantifier-subtree campaign closed.** All six rules (`digits`, `quant_suffix`, `counted_quantifier_body`, `counted_quantifier`, `quant_base`, `quantifier`) are now annotated. Consumer-side `extract_quantifier` walker collapses to a six-line typed-field read — see the [Quantifier Subtree](../regex_parser_book/src/rules-quantifier.md#putting-it-together) chapter in the regex parser mdBook for the canonical recipe.
- **Recommended RGX integration steps:**
  1. Update PGEN dependency to the post-`1.1.35` commit on `main`.
  2. Regenerate the regex parser via `make regex_parser` (default emit) or `make regex_parser_fresh` for a clean rebuild.
  3. Update any code that pattern-matched on the `[<base>, <suffix>]` 2-tuple at the `quantifier` position. Replace string-vs-object dispatch with a single typed-object read on `quantifier`. Map `greediness: []` → `"greedy"`.
- Public API surface unchanged: `RegexParser::new`, `parser.parse_full_regex()`, `parser.parse_regex()`, `parse_regex_typed()`, `parse_regex_default_ast_dump_named()` keep the same signatures. `ParseNode` and `ParseContent` enum unchanged.
- Regex AST schema version stays `1`.

## Release 1.1.34 / Contract 1.1.36 Highlights — PGEN-RGX-0075 typed-shape correctness for multi-piece concatenation

- **Driven by RGX bug report PGEN-RGX-0075** (`/Users/richarddje/Documents/github/rgx/pgen-issues/PGEN-RGX-0075.yaml`).
- **What was wrong:** for inputs like `"abc"`, the typed `regex.pattern` field surfaced only the first piece — `"b"` and `"c"` were silently dropped. Span correctly covered the full input (so the parser DID match all three pieces), but the typed-shape rendering of `concatenation = piece+ -> [$1**]` collapsed to a 1-element array instead of the documented 3-element flat array.
- **Root cause:** the `$N` codegen in `rust/src/ast_pipeline/ast_return_transform.rs` peeled `elements[0].content.clone()` from a `Quantified` base when `captured_vars.len() == 1`, treating the Quantified like a single-element Sequence wrapper. This violated the doctrine in `docs/book/src/annotation-system.md` which says `$1` on a Quantified body is "the whole capture group". The fix removes the `Quantified` peel-arm in three codegen sites (`generate_positional_ref`, `generate_value_extraction`, `generate_quantified_extraction`); Quantified bases now fall through to `other.clone()`, passing the whole Quantified content to the caller (e.g. flatten-spread `[$1**]` then iterates correctly).
- **Grammar adjustment (compensating change):** the regex entry rule `regex = pattern? -> {type: "regex", pattern: $1}` previously relied on the buggy auto-peel to unwrap the `Quantified-?`. With the codegen fix, `$1` correctly returned the wrapper `[pattern_value]` instead of the bare pattern. Rule changed to `regex = pattern -> ...` (the inner `alternative = concatenation?` already handles emptiness, so `pattern` is total). Empty input still parses successfully with span `0..0`.
- **AST shape (consumer-visible):**
  - Before fix: input `"abc"` → `pattern: [[[ {atom:"a",...} ]], []]` (single-piece array — buggy).
  - After fix: input `"abc"` → `pattern: [[[ {atom:"a",...}, {atom:"b",...}, {atom:"c",...} ]], []]` (all three pieces — correct).
  - Top-level pattern shape `[<head_alt>, <tail>]` is **unchanged** thanks to the compensating grammar change. The fix purely surfaces the missing pieces.
- **No public API surface change.** `RegexParser::new`, `parser.parse_full_regex()`, `parser.parse_regex()`, `parse_regex_typed()`, `parse_regex_default_ast_dump_named()` keep the same signatures. `ParseNode` and `ParseContent` enum unchanged.
- **Regression-lock test:** `regex_parser_pgen_rgx_0075_multi_piece_concatenation_surfaces_all_pieces` in `rust/src/embedding_api.rs` pins the empirical shape for `"a"`, `"ab"`, `"abc"`, `"hello"`. The bug cannot return without the test failing.
- **RGX integration impact:** consumers walking `regex.pattern[0][0]` get the documented flat array of all pieces. The earlier observation "RGX lib tests dropped to ~73% pass rate purely from this single PGEN issue" should be addressed when RGX bumps PGEN to a release containing this fix and re-runs the test suite.
- Regex AST schema version stays `1` — the byte-shape change is a buggy-to-correct fix within the schema, not a structural redesign.

## Release 1.1.33 / Contract 1.1.35 Highlights — quantifier-subtree typed-shape rollout (slice 2/N: `quant_suffix`)

- Internal-driven shape work (no downstream report). Part of an ongoing campaign to make the regex AST fully typed — every field directly usable, no consumer-side parsing/extraction needed (per project doctrine).
- **Rule changed: `quant_suffix`** in `grammars/regex.ebnf`. Per-branch annotations replaced raw token output with semantic strings:
  - `quant_suffix = "?" -> "lazy" | "+" -> "possessive"`.
- **AST shape change (consumer-visible)**: the `quant_suffix?` slot inside `quantifier` now carries the typed string `"lazy"` / `"possessive"` directly. Empirical:
  - Before: input `a*?` → `quantifier: ["*", "?"]` (raw `Terminal("?")` in the Quantified-? slot).
  - After:  input `a*?` → `quantifier: ["*", "lazy"]` (typed string).
  - Same for `a*+` → `["*", "possessive"]`. Inputs without a suffix keep the existing `[]` shape (empty `Quantified-?`).
- Public API surface unchanged: `RegexParser::new`, `parser.parse_regex()`, `parser.parse_full_regex()`, `parse_regex_typed()` keep the same signatures. `ParseNode` and `ParseContent` enum unchanged.
- This is a **partial step** in the quantifier-subtree work. The final published `quantifier` shape (target: `{type:"quantifier", min, max, greediness}`) is still pending the slices that close `counted_quantifier_body`, `counted_quantifier`, `quant_base`, and `quantifier`. Until then, RGX integrating against this version sees:
  - Typed `min`/`max` integer scalars at the digits positions inside `counted_quantifier`.
  - Typed `"lazy"` / `"possessive"` strings at the `quant_suffix` position in `quantifier`.
  - Raw shape for the `quant_base` and outer `quantifier` slots (still `Sequence`/`Quantified` carriers).
- Recommended RGX integration steps:
  1. Update PGEN dependency to the post-`1.1.33` commit on `main`.
  2. Regenerate the regex parser via `make regex_parser` (default emit).
  3. Update any code that pattern-matched on `Terminal("?")` / `Terminal("+")` at the `quant_suffix` position to match the typed strings instead.
- Regex AST schema version stays `1`.

## Release 1.1.32 / Contract 1.1.34 Highlights — quantifier-subtree typed-shape rollout (slice 1/N: `digits`)

- Internal-driven shape work (no downstream report). First slice of the quantifier-subtree typed-shape campaign.
- **Rule changed: `digits`** in `grammars/regex.ebnf`. Rewrote `digits = digit+` to `digits = /([0-9]+)/` paired with `@transform: str::parse::<usize>().unwrap_or(0)`. Output is now a typed integer at the rule directly.
- **AST shape change (consumer-visible)**: every position in the AST that contained a `digits` sub-tree now contains a typed integer instead of a raw `Quantified(Terminal-per-char)` carrier. Affected positions:
  - `counted_quantifier_body`'s digit slots (the `min` and `max` of `{n}`, `{n,}`, `{n,m}`, `{,m}`).
  - `version_number`'s digit slots inside conditional version checks `(?(VERSION>=1.55)...)`.
  - `recursion_condition`'s optional `digits?` slot inside `(?(R)...)` style conditions.
  - `callout_arg`'s numeric form `(?C12)`.
  - `backreference_digits = nonzero_digit digit*` is **NOT** affected — it uses the per-char `digit` rule.
- Empirical: input `\Qab*\E{2,}` → `quantifier`'s digit position is `2` (integer) instead of `[["2"]]` (raw nested array).
- Other digits-collection rules (`hex_digits`, `octal_digits`) are NOT included in this slice — they keep their existing per-char emit, queued as future slices in the campaign.
- Public API surface unchanged. Regex AST schema version stays `1`.
- Recommended RGX integration steps:
  1. Update PGEN dependency to the post-`1.1.32` commit on `main`.
  2. Regenerate the regex parser via `make regex_parser`.
  3. Update any code that walked digits positions expecting a `Quantified`/`Terminal-per-char` shape — these positions now carry typed integers directly.

## Release 1.1.31 / Contract 1.1.33 Highlights — PGEN-RGX-0074 `\Q...\E` quantifier-attachment correctness

- `1.1.31` closes RGX correctness report **PGEN-RGX-0074**.
- Per `pcre2pattern(3)` §"Backslash", a quantifier following `\Q...\E` applies only to the **last character** of the literal sequence; PGEN previously bound the quantifier to the entire quoted block.
- AST shape change for `\Q...\E quantifier?` patterns (consumer-visible):
  - **Before:** one `piece` with `atom = quoted_literal(\Q...\E)` and the trailing quantifier attached to the whole block.
  - **After:** N pieces under the parent `concatenation` — one independent `{type:"piece", atom, quantifier:[]}` per prefix char and one final `{type:"piece", atom:<last>, quantifier:<the trailing quantifier>}` for the last char.
  - Single-char `\Qx\E quantifier?` and empty `\Q\E quantifier?` remain handled by the original `atom quantifier?` branch (no behavior change for those degenerate cases).
- Family-table proof, all matching PCRE2 semantics: `\Qab*\E{2,}` -> 3 pieces (a, b, *{2,}); `\Qabc\E{2}` -> 3 pieces (a, b, c{2}); `\Qab\E{3}` -> 2 pieces (a, b{3}); `\Qa\E{3}` -> 1 piece; `\Q\E{2}` -> 1 piece (atom-fallback).
- Parser-agnostic implementation:
  - New `**` (flatten-spread) primitive added to the return-annotation language (`grammars/return_annotation.ebnf`). Like single-`*` spread, but for each pushed child whose `content` is `Sequence`/`Quantified`, unwraps one level so a child rule may produce either a single value OR an array of values that appear flat in the parent's accumulator.
  - New `piece` rule alternative `piece_quoted_run_quantified = "\\Q" quoted_run_inner_piece* quoted_literal_char "\\E" quantifier -> [$2**, {type:"piece", atom:$3, quantifier:$5}]`. The trailing `!"\\E"` negative lookahead in `quoted_run_inner_piece` keeps the greedy `*` from over-consuming the last char.
  - `concatenation = piece+ -> [$1**]` flattens piece arrays into a flat piece accumulator.
- Public API surface unchanged: `RegexParser::new`, `parser.parse_regex()`, `parser.parse_full_regex()`, `parse_regex_typed()` keep the same signatures. `ParseNode` and `ParseContent` enum unchanged. `ParseContent::to_json_value() -> serde_json::Value` produces the corrected attachment shape across the family.
- **Quantifier-shape typing remains a separate slice** (tracked under task #40 — "Annotate regex.ebnf for full AST usability"). This release fixes attachment correctness; the quantifier subtree (`quantifier`, `quant_base`, `counted_quantifier`, `counted_quantifier_body`, `quant_suffix`, `digits`) still leaks raw parse-tree shape pending the dedicated typing slice.
- Recommended RGX integration steps:
  1. Update PGEN dependency to the post-`1.1.31` commit on `main`.
  2. Regenerate the regex parser via `make regex_parser` (default emit).
  3. Audit any code that walks `pattern[*]` expecting a single piece for `\Q...\E quantifier?` inputs — the AST now exposes N pieces with attachment on the last only.
  4. Run existing tests; the new pieces shape matches PCRE2 semantics so behavioral tests should align.
- Regex AST schema version stays `1`.

## Release 1.1.30 / Contract 1.1.32 Highlights — PGEN-RGX-0073 perf closure
- `1.1.30` is the PGEN-RGX-0073 perf-closure release over parser release `1.1.29`; regex AST dump schema version stays `1` and JSON output via `ParseContent::to_json_value()` is byte-equivalent to `1.1.29`.
- This specifically covers RGX perf report `PGEN-RGX-0073` (regex parse times in the ms range vs the PRIMARY <50µs target).
- **Closure result on the 8-pattern bug corpus** (Apple M-series, release, mimalloc allocator, 5000 samples / 200 warmup, p50): `literal_simple` 13µs, `digit_sequence` 28µs, `character_class` 35µs, `alternation` 26µs, `capture_groups` 51µs, `url_simple` 23µs, `email_basic` 36µs, `anchor_complex` 76µs. **7/8 patterns under PRIMARY 50µs on the legacy `parser.parse_full_regex()` path.** `capture_groups` is at the line (50.9µs, 0.9µs over — sample noise can flip it under or over). `anchor_complex` retains a 26µs structural gap from the `atom` rule's 25-way Or chain over different rule shapes (cannot be collapsed to a single regex; closure would require grammar restructuring beyond the current contract).
- **Cumulative speedup vs the original PGEN-RGX-0073 bug-bundle baseline**: literal_simple 22×, digit_sequence 20×, **character_class 53×**, alternation 29×, capture_groups 22×, url_simple 28×, email_basic 23×, **anchor_complex 27×**. Geomean ~25× faster across all 8 patterns. The biggest single-pattern win is `character_class` (originally 1.87ms, now 35µs).
- **Public API surface unchanged**: `RegexParser::new`, `parser.parse_regex()`, `parser.parse_full_regex()` keep the same signatures. `ParseNode { content: ParseContent, rule_name, span }` and the `ParseContent` enum (`Terminal`, `TransformedTerminal`, `Json`, `Sequence`, `Alternative`, `Quantified`) are unchanged. `ParseContent::to_json_value() -> serde_json::Value` produces byte-equivalent output across the 8 corpus patterns (machine-verified by the maintained `make regex_typed_differential_gate` target on every regen with `--enable-parser-hooks`).
- **`ParseNode` shape change for 11 leaf rules** (the only consumer-visible change): the rules `letter`, `digit`, `nonzero_digit`, `hex_digit`, `octal_digit`, `whitespace`, `literal_char`, `class_literal`, `class_safe_special`, `any_char`, `special_char` were rewritten in `grammars/regex.ebnf` from `Or`-of-single-character alternatives to `/.../` regex literals. The legacy emit's `ParseNode` for these rules now carries `ParseContent::Terminal(matched_str)` directly instead of `ParseContent::Alternative(child_node)` (where `child_node.content` was the `Terminal`). Consumers that rely on `to_json_value()` (or any code path that consumes JSON transitively) see no change. Consumers that pattern-match on `ParseContent::Alternative` for these specific 11 rules need to update to match `ParseContent::Terminal` instead.
- **New optional typed entry-point** (opt-in): `parser.parse_regex_typed() -> ParseResult<serde_json::Value>` and per-rule `parse_<rule>_typed()` methods bypass `ParseNode` allocation and return `serde_json::Value` directly. Available only when the parser was regenerated with `--enable-parser-hooks` (registers the regex parser hook at `rust/src/parser_hooks/regex.rs`). Default `make regex_parser` does NOT register the hook → the default emit doesn't carry these methods. Output is byte-equivalent to `parse_regex()?.content.to_json_value()` (verified by `make regex_typed_differential_gate` 8/8 across the corpus). Consumers can migrate to this fast path for the JSON-direct case; otherwise no action required.
- **Recommended RGX integration steps**:
  1. Update PGEN dependency to the post-`1.1.30` commit on `main`.
  2. Regenerate the regex parser via `make regex_parser` (default emit, legacy API).
  3. Audit any code that destructures `ParseContent` for the 11 leaf rules listed above; update `Alternative` → `Terminal` matches as needed.
  4. Run existing tests; JSON output via `to_json_value()` is byte-equivalent so JSON-consuming tests should pass unchanged.
  5. Optional: migrate to `parser.parse_regex_typed()` for the JSON-direct fast path (regenerate with `--enable-parser-hooks`).
- **Differential gate verification**: `make regex_typed_differential_gate` ran 8/8 byte-equivalent at every slice of the 16-slice closure campaign. The maintained gate compares `parser.parse_regex()?.content.to_json_value()` (legacy reference) against `parser.parse_regex_typed()` (typed entry, when the hook is registered) byte-for-byte across the 8 PGEN-RGX-0073 corpus patterns. The gate is the regression-lock for any future grammar change touching the 11 rewritten rules.
- This release is a perf-closure release over the already-closed regex family row, not a reopening of the family status. The regex family proof remains green (frontend overall `pass`, dual-run overall `pass`, parser-backed stimuli `5911/5197/714`, closed target debt `804 -> 0`); this release does not alter any conformance test counts.

## Release 1.1.29 / Contract 1.1.31 Highlights
- `1.1.29` is a generated-host compile-contract maintenance release over parser release `1.1.28`; regex AST dump schema version stays `1`.
- This specifically covers RGX PCRE2 conformance report `PGEN-RGX-0072`.
- Bare octal class-range endpoints now participate in the same decoded-codepoint ordering contract as braced hex, single-byte hex, braced octal, control escapes, simple escaped literals, and plain literal codepoints.
- `\NNN` escapes are now consumed as one character-class atom with one to three octal digits before range validation. This keeps trailing octal digits from being re-read as separate class literals.
- Ascending ranges such as `[\000-\037]`, `[\010-\037]`, `[a-\377]`, `[\001-\x1f]`, and `[\001-\x{1f}]` are accepted by comparing decoded endpoint values.
- Descending decoded endpoint pairs such as `[\x1f-\0]`, `[\037-\000]`, and `[\377-a]` are rejected by the generated-host compile contract.
- The regex integration contract now has `93` success samples and `25` failure samples, including `bare_octal_class_range_endpoint`, `literal_to_bare_octal_class_range_endpoint`, and `descending_decoded_class_range_endpoint`.
- This release is compatibility maintenance over the already-closed regex family row, not a reopening of the family status.

## Release 1.1.28 / Contract 1.1.30 Highlights
- `1.1.28` is a generated-host compile-contract maintenance release over parser release `1.1.27`; regex AST dump schema version stays `1`.
- This specifically covers RGX PCRE2 conformance report `PGEN-RGX-0071`.
- Braced hex class-range endpoints are now ordered by decoded codepoint value, not by the escaped payload's leading byte. `[z-\x{100}]` is accepted because `U+007A < U+0100`, while `[\x{100}-z]` remains rejected as a descending range.
- The same endpoint decoder now handles single-byte hex, braced octal, control escapes, and simple escaped literals consistently for class-range ordering.
- Malformed braced class escapes still preserve the existing PCRE2 `bad_escape_is_literal` oracle behavior; they do not become invented validator failures when they are not valid numeric endpoint payloads.
- The regex integration contract now has `91` success samples and `24` failure samples, including `wide_braced_hex_class_range_endpoint` and `descending_wide_braced_hex_class_range`.
- The maintained `regex_pcre2_compile_oracle_gate` baseline remains green at the measured `pcre2-10.47` slice: `2195` cases executed, `1613` compile-ok cases, `582` compile-fail cases, `1843` expectation matches, `352` mismatches, `307` false accepts, and `45` false rejects.
- The refreshed regex family proof still computes `Done`: frontend overall `pass`, dual-run overall `pass`, `perl_rule_count=104`, `rust_rule_count=194`, parser-backed stimuli `5911/5197/714`, diagnostic target-drive parser rejections `714`, and closed target debt `804 -> 0` after `6526` target-drive attempts. This release is compatibility maintenance over the already-closed regex family row, not a reopening of the family status.

## Release 1.1.27 / Contract 1.1.29 Highlights
- `1.1.27` is a PCRE2 source-derived grammar and generated-host compile-contract maintenance release over parser release `1.1.26`; regex AST dump schema version stays `1`.
- This specifically covers RGX PCRE2 conformance reports `PGEN-RGX-0067`, `PGEN-RGX-0068`, `PGEN-RGX-0069`, and `PGEN-RGX-0070`.
- Plain `\N` is now rejected inside character classes by the generated-host compile contract, matching PCRE2's class scanner where `\N` is not a class escape unless it is a braced named-character form.
- Single-character `\Q...\E` regions inside character classes can now serve as range endpoints. `^[\Qa\E-\Qz\E]+` transports the range as `class_range` with two `quoted_class_range_atom` endpoints rather than as two independent `quoted_class_literal` items around a literal dash.
- Shorthand and property escapes are rejected as character range endpoints. The contract pins both `[\\d-x]` and `[\\pL-x]` as failures so downstream consumers can rely on the general nonliteral-endpoint rule rather than a one-off repro spelling.
- Backslashes inside a `\Q...\E` quoted literal body are preserved as literal quoted content until the terminating `\E`. The sample `\Qabc\$xyz\E` transports as one `quoted_literal` and includes `quoted_literal_escaped_char = "\\$"` instead of degrading to `simple_escape`.
- The regex integration contract now has `90` success samples and `23` failure samples, including `quoted_class_literal_single_char_range_endpoints`, `quoted_literal_body_keeps_literal_backslash`, `not_newline_escape_forbidden_in_class`, `shorthand_escape_forbidden_as_class_range_endpoint`, and `property_escape_forbidden_as_class_range_endpoint`.
- The maintained `regex_pcre2_compile_oracle_gate` baseline is ratcheted to the measured `pcre2-10.47` slice: `2195` cases executed, `1613` compile-ok cases, `582` compile-fail cases, `1843` expectation matches, `352` mismatches, `307` false accepts, and `45` false rejects.
- The refreshed regex family proof still computes `Done`: frontend overall `pass`, dual-run overall `pass`, `perl_rule_count=104`, `rust_rule_count=194`, parser-backed stimuli `5911/5197/714`, diagnostic target-drive parser rejections `714`, and closed target debt `804 -> 0` after `6526` target-drive attempts. This release is compatibility maintenance over the already-closed regex family row, not a reopening of the family status.

## Release 1.1.26 / Contract 1.1.28 Highlights
- `1.1.26` is a PCRE2 source-derived generated-host compile-contract maintenance release over parser release `1.1.25`; regex AST dump schema version stays `1`.
- This specifically covers RGX PCRE2 conformance reports `PGEN-RGX-0065` and `PGEN-RGX-0066`.
- PCRE2 UTF width start-option aliases are now accepted by the generated-host compile contract. The published `regex_default` profile accepts `(*UTF8)` alongside `(*UTF)`, and also admits the documented width-family aliases `(*UTF16)` and `(*UTF32)` as start options so downstream consumers do not have to special-case those spellings before PGEN parsing.
- Scan-substring capture-list validation now resolves against the full pattern capture inventory for absolute and named references, matching PCRE2's ability to accept forward references such as `(*scs:(1)a)(a)|x` and `(*scs:(<GOOD_NAME>)a)(?<GOOD_NAME>a)`.
- Relative scan-substring references retain their directionality: `+n` is resolved against later captures visible in the full inventory, while `-n` still requires enough captures before the scan-substring group.
- The regex integration contract now has `88` success samples and `20` failure samples, including `pcre2_utf8_start_option_alias`, `scan_substring_forward_numeric_capture_ref`, and `scan_substring_forward_named_capture_ref`.
- The maintained `regex_pcre2_compile_oracle_gate` baseline is ratcheted to the measured `pcre2-10.47` slice: `2195` cases executed, `1613` compile-ok cases, `582` compile-fail cases, `1840` expectation matches, `355` mismatches, `309` false accepts, and `46` false rejects.
- The focused generated-host regression coverage also exercises optional/lazy scan-substring forms and the PCRE2 `(+1,+2)` forward-relative grouplist shape from RGX's `testinput2` cluster.
- The refreshed regex family proof still computes `Done`; this release is compatibility maintenance over the already-closed regex family row, not a reopening of the family status.

## Release 1.1.25 / Contract 1.1.27 Highlights
- `1.1.25` is a PCRE2 source-derived grammar and compile-contract maintenance release over parser release `1.1.24`; regex AST dump schema version stays `1`.
- This specifically covers RGX PCRE2 conformance reports `PGEN-RGX-0063` and `PGEN-RGX-0064`.
- PCRE2 POSIX word-boundary aliases are now accepted only in their exact standalone forms `[[:<:]]` and `[[:>:]]`, modeled as `posix_word_boundary_alias` atoms. Mixed character classes such as `[a[:<:]]` remain rejected, matching `pcre2pattern(3)` and the dedicated `pcre2_compile.c` branch for these BSD/POSIX compatibility aliases.
- The generated-host compile contract now treats `(?(DEFINE)...)` conditionals as declarative zero-width groups while scanning lookbehind bodies for unbounded quantifiers. This keeps `(?<=X(?(DEFINE)(.*))Y).` accepted because the runtime lookbehind body is fixed-width `XY`, while ordinary unbounded lookbehind forms remain rejected.
- The regex integration contract now has `85` success samples and `20` failure samples, including `pcre2_posix_word_boundary_aliases`, `define_conditional_is_zero_width_for_lookbehind_length`, and `mixed_posix_word_boundary_alias_is_not_pcre2`.
- The maintained `regex_pcre2_compile_oracle_gate` baseline is ratcheted to the measured `pcre2-10.47` slice: `2195` cases executed, `1613` compile-ok cases, `582` compile-fail cases, `1834` expectation matches, `361` mismatches, `309` false accepts, and `52` false rejects.
- The refreshed regex family proof still computes `Done` with frontend overall `pass`, dual-run overall `pass`, `perl_rule_count=100`, `rust_rule_count=189`, parser-backed stimuli `5292/4677/615`, diagnostic target-drive parser rejections `615`, and closed target debt `758 -> 0` after `5825` target-drive attempts.

## Release 1.1.24 / Contract 1.1.26 Highlights
- `1.1.24` is a PCRE2 source-derived grammar and AST-contract maintenance release over parser release `1.1.23`; regex AST dump schema version stays `1`.
- This specifically covers RGX PCRE2 conformance reports `PGEN-RGX-0061` and `PGEN-RGX-0062`.
- PCRE2 `\C` now transports through `escape` -> `escape_unit` -> `single_byte_escape`, not through generic `simple_escape("C")`. This preserves the downstream distinction between PCRE2 single-code-unit matching and ordinary escaped literal fallback.
- Conditional assertion tests may now be preceded by an explicit callout, matching PCRE2's documented and source-implemented form. Representative accepted forms are `^(?(?C25)(?=abc)abcd|xyz)` and `^(?(?C$abc$)(?=abc)abcd|xyz)`.
- The conditional-callout assertion contract preserves both pieces: `condition_callout` carries the numeric or string callout payload via the existing `callout_arg` shape, and `condition_assertion` carries the following assertion body.
- The regex integration contract now has `83` success samples and `19` failure samples, including `single_byte_escape_code_unit`, `conditional_numeric_callout_assertion`, and `conditional_string_callout_assertion`.
- The maintained `regex_pcre2_compile_oracle_gate` baseline is ratcheted to the measured `pcre2-10.47` slice: `2195` cases executed, `1613` compile-ok cases, `582` compile-fail cases, `1832` expectation matches, `363` mismatches, `309` false accepts, and `54` false rejects.
- The refreshed regex family proof still computes `Done` with frontend overall `pass`, dual-run overall `pass`, `perl_rule_count=100`, `rust_rule_count=188`, parser-backed stimuli `5266/4615/651`, diagnostic target-drive parser rejections `651`, and closed target debt `750 -> 0` after `5812` target-drive attempts.

## Release 1.1.23 / Contract 1.1.25 Highlights
- `1.1.23` is a PCRE2 source-derived grammar and compile-contract maintenance release over parser release `1.1.22`; regex AST dump schema version stays `1`.
- This specifically covers RGX PCRE2 conformance reports `PGEN-RGX-0058`, `PGEN-RGX-0059`, and `PGEN-RGX-0060`.
- Bounded variable-length lookbehind is now accepted through the generated-host compile contract. Representative accepted forms include `(?<=a{1,3})b` and control-verb-bearing lookbehind such as `(?<=a(*ACCEPT)b)c`.
- Unbounded variable-length lookbehind remains rejected. Forms such as `(?<=a+)b`, `(?<=a*)b`, and `(?<=a{2,})b` still fail the generated-host compile contract.
- PCRE2 directive/control-verb groups such as `(*ACCEPT)`, `(*COMMIT)`, `(*FAIL)`, `(*PRUNE)`, `(*SKIP)`, `(*THEN)`, and `(*:MARK)` are skipped as directive groups while scanning lookbehind bodies for unbounded quantifiers. Their leading `*` is not treated as a regex repetition operator.
- Capture names and named-reference payloads now follow the UTF-shaped PCRE2 name contract: names are non-empty, must not start with a digit, may contain `_`, alphabetic characters, and numeric characters, and are bounded by PCRE2's `MAX_NAME_SIZE=128` byte limit. Representative accepted form: `(?'ABáC'...)\g{ABáC}`.
- Malformed named-reference spellings such as bare `\k`, empty `\k{}`, and overlong capture names are rejected by the generated-host compile contract instead of falling through as false accepts.
- Inside a character class, orphan `\E` is modeled as a zero-width scanner marker, matching the `pcre2_compile.c` `ESC_E` class branch. It may appear inside a non-empty class and around range dashes, so `^[\Eabc]` and `^[a-\Ec]` are accepted, but `[\E]` is still rejected because it has no substantive class atom.
- The regex integration contract now has `80` success samples and `19` failure samples, including bounded lookbehind, lookbehind control verbs, Unicode capture names, orphan class `\E`, unbounded-lookbehind failures, malformed named-reference failures, and overlong-name failures.
- The maintained `regex_pcre2_compile_oracle_gate` baseline is ratcheted to the measured `pcre2-10.47` slice: `2195` cases executed, `1613` compile-ok cases, `582` compile-fail cases, `1828` expectation matches, `367` mismatches, `309` false accepts, and `58` false rejects.
- The regex family contract gate now exposes a bounded `stimuli_target_max_attempts=10000` budget so the enlarged grammar target surface can still close to `final_targets=0` without relaxing the `Done` status rule.

## Release 1.1.22 / Contract 1.1.24 Highlights
- `1.1.22` is a PCRE2 source-derived grammar and compile-contract maintenance release over parser release `1.1.21`; regex AST dump schema version stays `1`.
- This specifically covers RGX PCRE2 conformance reports `PGEN-RGX-0056` and `PGEN-RGX-0057`.
- Short Unicode property escapes now transport through `property_escape` instead of falling back to `simple_escape` plus a following literal. The supported unbraced one-letter general-category family is `C`, `L`, `M`, `N`, `P`, `S`, and `Z`, in either case, matching PCRE2's `\pX` / `\PX` shorthand for `\p{X}` / `\P{X}`.
- Character classes now accept quoted literal regions through `quoted_class_literal`. For example, `[z\Qa-d]\E]` preserves `\Qa-d]\E` as quoted class literal text, so the `]` inside the quote belongs to the class rather than closing it.
- The generated-host compile contract rejects malformed short properties such as `\pA`, `\P_`, and bare `\p`.
- Empty `\Q\E` inside a character class is treated as zero-width: it is allowed when another class atom already exists, but it is rejected when it would make an otherwise empty class valid or act as a range endpoint. This keeps PCRE2's scanner nuance without pretending the empty quote is a substantive class item.
- At `1.1.22`, orphan `\E` inside a character class remained outside the PGEN contract and was rejected by the generated-host validator. Release `1.1.23` supersedes that limitation by modeling orphan class `\E` as a zero-width scanner marker.
- The regex integration contract now has `75` success samples and `16` failure samples, including short property escapes, quoted class literals, invalid short properties, and empty-quote/range edge cases.
- The maintained `regex_pcre2_compile_oracle_gate` baseline is ratcheted to the measured `pcre2-10.47` slice: `2195` cases executed, `1613` compile-ok cases, `582` compile-fail cases, `1814` expectation matches, `381` mismatches, `314` false accepts, and `67` false rejects.

## Release 1.1.21 / Contract 1.1.23 Highlights
- `1.1.21` is a PCRE2 source-derived grammar and compile-contract audit release over parser release `1.1.20`; regex AST dump schema version stays `1`.
- This release follows the source-of-truth workflow in `docs/reference/REGEX_BOOTSTRAP_ARCHITECTURE.md`: use `pcre2syntax(3)` / `pcre2pattern(3)` for prose intent, `src/pcre2_compile.c` for edge-case authority, and PCRE2 `testdata/testinput*` through `regex_pcre2_compile_oracle_gate` as executable regression evidence.
- Grammar-level additions and corrections include:
  - `\K` as an anchor-like atom outside forbidden contexts
  - one-digit `\xA` and whitespace-braced `\x{ 41 }` / `\o{ 101 }` escape payloads
  - string callouts such as `(?C"alpha""beta")` and `(?C{left}}right})`, with doubled delimiter escaping
  - `(*atomic:...)`, non-atomic symbolic lookarounds `(?*...)` / `(?<*...)`, and non-atomic alpha lookarounds such as `(*napla:...)`
  - `(*scs:...)` / `(*scan_substring:...)` scan-substring groups and `(*sr:...)` / `(*script_run:...)` / `(*asr:...)` / `(*atomic_script_run:...)` script-run groups
  - stricter inline modifier spelling, including ASCII restrictors such as `(?aD)` and extended-mode `(?xx:...)`
  - strict PCRE2 VERSION conditional syntax with no whitespace around the operator, such as `(?(VERSION=10)cat|dog)` and `(?(VERSION>=10.0)cat|dog)`
  - comma-only `{,}` as literal text rather than a counted quantifier, while valid left-open counted quantifiers such as `{,4}` remain counted quantifiers
- Generated-host compile-contract additions include:
  - rejecting numeric callouts above PCRE2's `255` limit
  - validating the PCRE2 start-option and verb-name tables derived from `pcre2_compile.c`
  - rejecting quantified non-ACCEPT verbs
  - rejecting `\K` inside lookaround assertions
  - rejecting forbidden character-class escapes such as `\A`, `\B`, `\C`, `\E`, `\G`, `\K`, `\Q`, `\R`, `\X`, `\Z`, and `\z`
  - validating POSIX class names while still allowing malformed opener text to fall back as literals where PCRE2 does
  - validating scan-substring capture lists against the whole-pattern capture inventory for absolute/named/forward-positive references, while retaining prior-capture requirements for negative relative references
  - rejecting unsupported default-mode escapes such as `\i`, `\F`, `\l`, `\L`, `\u`, and `\U`
  - rejecting suffixed whole-pattern recursion syntax such as `(?R1)` while retaining whole-pattern recursion `(?R)` and proper returned-capture calls such as `(?R(1))`
- The maintained `regex_pcre2_compile_oracle_gate` baseline was ratcheted to the then-measured `pcre2-10.47` slice: `2195` cases executed, `1613` compile-ok cases, `582` compile-fail cases, `1806` expectation matches, `389` mismatches, `318` false accepts, and `71` false rejects. Release `1.1.22` ratchets this further.

## Release 1.1.20 / Contract 1.1.22 Highlights
- `1.1.20` is a generated-regex resource-depth hardening release over parser release `1.1.19`; regex AST dump schema version stays `1`.
- This specifically covers RGX PCRE2 conformance reports `PGEN-RGX-0054` and `PGEN-RGX-0055`.
- `PGEN-RGX-0054` is a legal PCRE2 pattern with `80` nested capturing groups followed by `\80`. The earlier generated-host path aborted on the worker stack or stopped at byte `0` after the runtime recursion guard was exceeded. The published regex host now uses a larger bounded generated-regex worker stack (`64 MiB`) and a widened generated parser recursion guard (`4096`), so the pattern parses successfully instead of aborting.
- `PGEN-RGX-0055` is a legal PCRE2 pattern modeled after a Python variable-interpolation grammar, with mutually recursive named groups and nested `\g<...>` references. The same resource-depth fix keeps that grammar-like regex parseable through the stable `regex_default` profile.
- The code-generation guard remains bounded: this is not an unbounded recursion allowance. It is a deliberately larger generated parser headroom target for real PCRE2 conformance inputs that are legal but syntactically deep.
- Integration contract `1.1.22` adds representative manifest samples for both shapes so downstream consumers can detect a regression through the normal regex integration contract gate.

## Contract 1.1.21 Highlights
- `1.1.21` is a downstream AST-contract clarification over parser release `1.1.19`; it does not change the regex grammar, parser release version, or AST dump schema version.
- This specifically covers RGX PCRE2 conformance report `PGEN-RGX-0053`, `[[:digit:]-   ]`.
- PCRE2 treats `[:digit:]` as a complete POSIX character-class item. A following `-` after a POSIX class is not a range opener; in the reported spelling it transports as a literal `-`, followed by three literal spaces, before the class closes.
- PGEN already emits this shape as `class_item` -> `posix_class` with `posix_name = "digit"`, followed by separate `class_item` -> `class_literal` nodes for `-`, ` `, ` `, and ` `. The contract now pins that exact mixed class shape so downstream adapters can handle it explicitly.

## Release 1.1.19 / Contract 1.1.20 Highlights
- `1.1.19` is a PCRE2-conformance braced `\k{...}` named backreference transport patch over the `1.1.18` parser release.
- This specifically covers RGX PCRE2 conformance report `PGEN-RGX-0051` and models the general PCRE2 brace-delimited name-reference shape rather than only the concrete `\k{ name }` spelling.
- PCRE2's brace-delimited subpattern-name reader permits spaces or tabs after `{` and before `}`. PGEN now models that parser-side shape as `braced_name_ref`, with the named payload preserved under `name`.
- Patterns such as `(?'name'ab)\k{ name }(?P=name)` now transport `\k{ name }` through `backreference` -> `braced_name_ref`, instead of degrading `\k` to `simple_escape` and treating `{ name }` as literals. Regex AST schema version stays `1`.

## Release 1.1.18 / Contract 1.1.19 Highlights
- `1.1.18` is a PCRE2-conformance braced `\g{...}` backreference transport patch over the `1.1.17` parser release.
- This specifically covers RGX PCRE2 conformance report `PGEN-RGX-0050` and models the general PCRE2 braced numeric `\g{...}` shape rather than only the concrete `\g{ -2 }` spelling.
- PCRE2 permits spaces or tabs after `{` and before `}` for braced `\g{...}` numeric references. PGEN now models that parser-side shape as `subroutine_ref` -> `braced_subroutine_ref`, with the signed or unsigned numeric payload preserved under `signed_digits`.
- Patterns such as `(A)(\g{ -2 }B)` now transport `\g{ -2 }` through `backreference` -> `subroutine_ref` -> `braced_subroutine_ref` -> `signed_digits`, with `signed_digits = "-2"`, instead of degrading `\g` to `simple_escape` and treating `{ -2 }` as literals. Regex AST schema version stays `1`.

## Release 1.1.17 / Contract 1.1.18 Highlights
- `1.1.17` is a PCRE2-conformance alpha-lookaround assertion patch over the `1.1.16` parser release.
- This specifically covers RGX PCRE2 conformance report `PGEN-RGX-0034` and models the general atomic alpha-lookaround family rather than only the concrete `(*pla:...)` spelling.
- PCRE2 supports lower-case assertion aliases for the four atomic lookaround forms: `(*pla:...)` / `(*positive_lookahead:...)`, `(*nla:...)` / `(*negative_lookahead:...)`, `(*plb:...)` / `(*positive_lookbehind:...)`, and `(*nlb:...)` / `(*negative_lookbehind:...)`.
- In conditional assertion position, patterns such as `(?(*pla:foo).{6}|a..)` now transport the condition through `conditional` -> `condition_assertion` -> `alpha_condition_assertion` -> `atomic_alpha_lookaround_name`, with the `yes_branch` and `no_branch` preserved under the existing conditional shape.
- Non-atomic lookaround aliases such as `(*napla:...)` / `(*naplb:...)` and script-run alpha assertions remain outside this atomic-lookaround condition contract for now. Regex AST schema version stays `1`.

## Release 1.1.16 / Contract 1.1.17 Highlights
- `1.1.16` is a PCRE2-conformance directive-payload generalization over the `1.1.15` parser release.
- This specifically covers RGX PCRE2 conformance reports `PGEN-RGX-0031` and `PGEN-RGX-0032` and broadens the contract for the backtracking-control verb payload family instead of pinning only the concrete failing payload string.
- PCRE2's default verb-name rule is that `(*VERB:NAME)` payload text is a sequence of characters up to the verb-closing `)`: `MARK` requires a non-empty payload/name, while `PRUNE`, `SKIP`, and `THEN` accept optional payload/name text. PGEN now models that parser-side payload shape conceptually as `directive_payload_char = /([^)])/`: any character except `)` is payload text in default `regex_default` directive parsing. This is a family-level grammar rule derived from PCRE2's default backtracking-control verb-name semantics, not a special case for `m(m` or any one RGX repro payload.
- MARK shorthand still transports through `directive_verb` -> `directive_mark_shorthand` -> `directive_payload_simple`; named backtracking-control verb payloads transport through `directive_verb` -> `directive_named` -> `directive_payload_suffix` -> `directive_payload_simple`. Regex AST schema version stays `1`.
- PGEN does not yet model PCRE2 `PCRE2_ALT_VERBNAMES` escape-processing semantics for verb names; under the stable `regex_default` profile, an unescaped `)` terminates the directive payload.

## Release 1.1.15 / Contract 1.1.16 Highlights
- `1.1.15` is a PCRE2-conformance directive-payload patch over the `1.1.14` parser release.
- This specifically covers RGX PCRE2 conformance reports `PGEN-RGX-0029` and `PGEN-RGX-0030`.
- MARK shorthand directives such as `(*:m(m)` and named directives such as `(*PRUNE:m(m)` now accept literal `(` inside the directive payload, so full patterns like `(*:m(m)(?&y)(?(DEFINE)(?<y>b))` and `(*PRUNE:m(m)(?&y)(?(DEFINE)(?<y>b))` parse as a directive atom followed by the named subroutine call and the `DEFINE` conditional.
- MARK shorthand payloads transport through `directive_verb` -> `directive_mark_shorthand` -> `directive_payload_simple`; named directive payloads transport through `directive_verb` -> `directive_named` -> `directive_payload_suffix` -> `directive_payload_simple`; regex AST schema version stays `1`.

## Release 1.1.14 / Contract 1.1.15 Highlights
- `1.1.14` is a PCRE2-conformance quoted-literal patch over the `1.1.13` parser release; integration contract `1.1.15` pins the downstream AST shape.
- This specifically covers RGX PCRE2 conformance report `PGEN-RGX-0023`.
- `abc\Q(*+|\Eabc` now treats the metacharacters inside `\Q...\E` as quoted literal payload instead of re-reading `(`, `*`, `+`, and `|` as active regex syntax.
- The quoted segment transports as a first-class `quoted_literal` atom whose rule text includes the delimiters, for example `\Q(*+|\E`.
- Downstream AST adapters should handle `quoted_literal` alongside `literal` and `escape`; regex AST schema version stays `1`.

## Contract 1.1.14 Highlights
- `1.1.14` is a downstream AST-contract clarification over parser release `1.1.13`; it does not change the regex grammar, parser release version, or AST dump schema version.
- This specifically covers RGX PCRE2 conformance reports `PGEN-RGX-0021`, `PGEN-RGX-0022`, and the mixed literal/POSIX-class reports `PGEN-RGX-0027` and `PGEN-RGX-0028`.
- `[[:space:]]+`, `[[:blank:]]+`, `^[:a[:digit:]]+`, `^[:a[:digit:]:b]+`, and `[[:digit:]-]+` emit a `class_item` containing the first-class `posix_class` variant, with `posix_name = "space"`, `posix_name = "blank"`, or `posix_name = "digit"` respectively.
- Downstream AST adapters that walk character classes must handle `posix_class` alongside `class_range`, `class_literal`, and `class_escape` instead of treating it as an unknown `class_item` shape.

## Release 1.1.13 Highlights
- `1.1.13` is a PCRE2-conformance character-class recovery patch over the `1.1.12` downstream handoff.
- The headline change in `1.1.13` is accepting malformed POSIX-class opener text inside a character class when PCRE2 treats the second `[` as a literal fallback, such as `([[:]+)`.
- This specifically covers RGX PCRE2 conformance report `PGEN-RGX-0018`.
- The same bracket-literal fallback also covers the related malformed equivalence-opener spelling from `PGEN-RGX-0019`, `([[=]+)`, malformed collating-opener spelling from `PGEN-RGX-0020`, `([[.]+)`, nested literal-bracket class spelling from `PGEN-RGX-0024`, `[[,abc,]+]`, malformed POSIX-class body spelling from `PGEN-RGX-0025`, `[[:abcd:xyz]]`, and malformed POSIX-looking class text with an escaped `]` from `PGEN-RGX-0026`, `[abc[:x\]pqr]`.
- The fix is deliberately narrow:
  - `posix_class` now wins before literal fallback inside `class_item`
  - `[` is allowed as a `class_literal` fallback only after the stricter POSIX-class path fails
  - the compile-style validator now mirrors that fallback instead of reporting an unterminated POSIX class for the same malformed opener text
  - regex AST schema version stays `1`
- `1.1.13` carries forward the `1.1.12` control-escape validator hardening and all prior regex parser contract guarantees.

## Release 1.1.12 Highlights
- `1.1.12` is a PCRE2-conformance validator patch over the `1.1.11` downstream handoff.
- The headline change in `1.1.12` is accepting PCRE2 control escapes whose target byte looks like regex syntax, such as `^\ca\cA\c[;\c:`.
- This specifically covers RGX PCRE2 conformance report `PGEN-RGX-0017`.
- The fix is deliberately narrow:
  - the regex grammar already transports `\cX` forms through `control_escape`
  - the compile-style validator now skips the complete `\cX` escape instead of leaving the target byte behind for later class/quantifier scans
  - regex AST schema version stays `1`
- `1.1.12` carries forward the `1.1.11` malformed counted-quantifier compatibility and all prior regex parser contract guarantees.

## Release 1.1.11 Highlights
- `1.1.11` is a PCRE2-conformance syntax-compatibility patch over the `1.1.10` downstream handoff.
- The headline change in `1.1.11` is accepting malformed counted-quantifier spellings as ordinary literal text when PCRE2 treats them that way, rather than rejecting the pattern during PGEN parseability.
- This specifically covers the RGX PCRE2 conformance cluster `PGEN-RGX-0040` through `PGEN-RGX-0049`, plus `PGEN-RGX-0052`, including forms such as:
  - `a{1,2,3}b`
  - `a{65536`
  - `X{`
  - `X{}`
  - `X{12ABC}`
  - `X{,9]`
  - `a{(?#XYZ),2}`
- The grammar change is deliberately narrow:
  - valid counted quantifiers still bind through `quantifier` before literal fallback
  - malformed brace forms now fall back through `literal_char`, so downstream RGX receives the literal pattern surface PCRE2 accepts
  - the compile-style validator still rejects truly invalid closed counted quantifiers such as inverted ranges and closed bounds above `65535`
- `1.1.11` strengthens the upstream regression surface for that widening:
  - `regex_parser_integration_contract_v1.json` now declares representative malformed counted-quantifier literal samples
  - the generated-backend parseability adapter now covers the full fixed RGX cluster
  - the compile validator now has an explicit PCRE2 literal-malformed-counted-quantifier regression test
- `1.1.11` carries forward the `1.1.10` PCRE2 VERSION conditional support and all prior regex parser contract guarantees.

## Release 1.1.10 Highlights
- `1.1.10` is a syntax-widening patch over the `1.1.9` downstream handoff.
- The headline change in `1.1.10` is publishing PCRE2 VERSION conditionals requested in RGX bug report `PGEN-RGX-0016`.
- `1.1.10` adds support for conditional forms such as:
  - `(?(VERSION>=10.0)cat|dog)`
  - `(?(VERSION>=10)cat|dog)`
- `1.1.10` transports those condition bodies structurally as:
  - `version_condition`
  - `version_operator`
  - `version_number`
- This keeps RGX's parse-time short-circuit path simple: downstream can slice the complete `condition` body and evaluate it against its fixed PCRE2 compatibility version before constructing a runtime conditional node.
- `1.1.10` strengthens the upstream regression surface for that widening:
  - the published regex integration manifest now explicitly includes:
    - a compact VERSION comparison sample
    - a whitespace-bearing VERSION comparison sample with a missing minor component
  - the generated-backend integration tests now assert:
    - correct `version_condition`, `version_operator`, and `version_number` text preservation
    - absence of bare-name fallback on VERSION condition bodies
- `1.1.10` carries forward the `1.1.9` returned-capture subroutine widening:
  - `1.1.9` published PCRE2 `10.47+` returned-capture subroutine syntax requested in RGX feature request `PGEN-RGX-0015`.
  - `1.1.9` adds support for parenthesized subroutine-return forms such as:
    - `(?1(1))`
    - `(?&callee(+1,<cap>,'alt'))`
  - `1.1.9` transports those forms structurally as:
    - `subroutine_call`
    - `returned_capture_subroutine`
    - `subroutine_target`
    - `returned_capture_group_list`
    - `returned_capture_group`
  - `1.1.9` strengthens the upstream regression surface for that widening:
    - the published regex integration manifest now explicitly includes:
      - a numeric returned-capture subroutine sample
      - a named-target returned-capture subroutine sample with mixed numeric/named grouplist entries
    - the generated-backend integration tests now assert:
      - correct returned-capture spans for `(?1(1))`
      - absence of `inline_modifiers` misclassification on the widened subroutine surface
- `1.1.10` carries forward the `1.1.8` syntax and depth-resilience fixes:
  - non-ASCII literal atoms such as `🎉` now parse as real `literal` nodes instead of rejecting at byte `0`
  - mixed ASCII/UTF-8 literal runs such as `café` now preserve `literal = ["c", "a", "f", "é"]` instead of stopping at the first multibyte codepoint
  - nested capturing groups now accept at least `50` levels cleanly instead of tripping the generated parser's overly conservative recursion guard around depth `12`
- `1.1.10` also carries forward the public regex host hardening from `1.1.8`:
  - generated regex entrypoints now execute on a dedicated larger-stack worker thread
  - the generated recursion guard is widened but still bounded (`512`)
- `1.1.10` also carries forward the `1.1.8` regression-surface strengthening:
  - the published regex integration manifest now explicitly includes:
    - a pure Unicode literal sample (`🎉`)
    - a mixed ASCII/Unicode literal sample (`café`)
    - a `50`-level nested capturing-group sample
  - the generated-backend integration tests now assert:
    - exact literal text preservation for Unicode samples
    - exact nested capturing-group count for the `50`-level sample
- `1.1.10` carries forward the `1.1.7` accepted-tree disambiguation fix:
  - `(?(R)a|b)` and `(a)(?(R1)b|c)` now transport `condition` through `recursion_condition` instead of falling back to bare `name`
- `1.1.10` carries forward the `1.1.6` accepted-tree span-integrity fix:
  - tagged payloads such as `(?{native:validate_word})` now preserve `code_content = "validate_word"` instead of dropping the first payload byte and transporting `"alidate_word"`
- `1.1.10` carries forward the `1.1.5` accepted-tree fix and tagged-syntax widening:
  - tagged payloads such as `(?{lua:return true})` now transport as `code_block_lang` containing `code_lang` and `code_content` instead of being shadowed by `code_block_plain`
  - `rhai` is now published alongside `lua`, `js`, and `javascript` as a structurally preserved tagged source-body form
  - `native` and `wasm` are now published as structurally preserved tagged payload forms, while runtime/reference validation remains downstream-owned
- `1.1.10` carries forward the `1.1.4` accepted-tree fix:
  - numeric angle forms such as `\g<1>` now transport as `backreference` containing `subroutine_ref` / `signed_digits` instead of `simple_escape("g")` plus literal `<`, `1`, `>`
- `1.1.10` carries forward the `1.1.3` accepted-tree and host-validation fixes:
  - braced octal escapes such as `\o{101}` now transport as `escape` containing `octal_escape` / `octal_digits` instead of `simple_escape` plus counted quantifier
  - brace-style numeric escapes are skipped atomically during post-parse validation, so they are no longer re-read as counted quantifiers
- `1.1.10` carries forward the `1.1.2` syntax unblock:
  - named recursion conditions such as `(?(R&word)a|b)` now parse and transport as `conditional` plus `recursion_condition`
- `1.1.10` also carries forward the `1.1.1` accepted-tree correctness fixes:
  - whole-pattern recursion `(?R)` now classifies as `subroutine_call` / `subroutine_target` instead of `inline_modifiers`
  - numeric backreferences such as `\1` now classify as `backreference` instead of generic `escape`
  - explicit conditional false branches such as `(?(1)a|b)` now preserve separate `yes_branch` and `no_branch` spans
  - trailing quantifiers now bind to the final literal atom instead of an entire preceding literal run, so `ab+` now transports as `a` plus `b+`, not `(ab)+`-style grouping
- `1.1.10` also carries forward the `1.1.0` published syntax coverage:
  - negated POSIX classes such as `[[:^alnum:]]`
  - braced named backreferences such as `\k{name}`
  - bare-name and signed numeric conditional references
  - named recursion conditions such as `R&name` inside conditionals
  - left-open counted quantifiers such as `{,4}`
- The generated regex host path in `1.1.10` also continues to enforce the compile-style validation contract added in `1.1.0`, so obvious compile-invalid forms no longer slip through as successful parses.
- The release is additionally backed by the maintained PCRE2 compile-oracle lane documented in `PGEN_USER_GUIDE.md`.

## Supporting Documents
- Public host API:
  - `rust/src/embedding_api.rs`
- Public API contract:
  - `rust/docs/EMBEDDING_API_CONTRACT.md`
- Published regex flavor and operator-facing guidance:
  - `PGEN_USER_GUIDE.md`
- Shared issue-reporting protocol:
  - `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`
- Canonical released-parser bug ledger:
  - `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`
- Family proof/status surfaces:
  - `LIVE_ACHIEVEMENT_STATUS.md`
  - `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`

## Stable Integration Surface
- Grammar family:
  - `regex`
- Stable host profile:
  - `regex_default`
- Stable metadata calls:
  - `embedding_api_contract()`
  - `parser_embedding_api_contract()`
- Stable regex convenience parse entry points:
  - `parse_regex_default(...)`
  - `parse_regex_default_with_limits(...)`
  - `parse_regex_default_result(...)`
  - `parse_regex_default_with_limits_result(...)`
- Stable regex convenience AST-dump entry points:
  - `parse_regex_default_ast_dump(...)`
  - `parse_regex_default_ast_dump_with_limits(...)`
- Stable generic grammar parse entry points:
  - `parse_grammar_profile(...)`
  - `parse_grammar_profile_with_limits(...)`
  - `parse_grammar_profile_result(...)`
  - `parse_grammar_profile_with_limits_result(...)`
- Stable generic grammar AST-dump entry points:
  - `parse_grammar_profile_ast_dump(...)`
  - `parse_grammar_profile_ast_dump_with_limits(...)`
  - `parse_grammar_profile_ast_dump_result(...)`
  - `parse_grammar_profile_ast_dump_with_limits_result(...)`
- Stable named-string entry points for bindings and adapters:
  - `parse_grammar_profile_named(...)`
  - `parse_grammar_profile_named_with_limits(...)`
  - `parse_grammar_profile_named_with_limits_result(...)`
  - `parse_grammar_profile_ast_dump_named(...)`
  - `parse_grammar_profile_ast_dump_named_with_limits(...)`
  - `parse_grammar_profile_ast_dump_named_with_limits_result(...)`
- Stable regex contract metadata available through `parser_embedding_api_contract()`:
  - `supports_regex_generated_backend`
  - `regex_integration_contract_version`
  - `regex_parser_release_version`
  - `regex_ast_dump_schema_version`
- Stable integration invariants:
  - `input_ownership_model=borrowed_str`
  - `parse_session_model=stateless_per_call`
    - this is a host-surface guarantee, not a claim that the generated parser implementation is internally stateless
    - the generated Rust parser remains stateful per parser instance during a call, carrying parse cursor, memoization, recursion-guard, and semantic-runtime state
    - downstream consumers should treat each public parse API call as an independent session and should not rely on cross-call parser state
  - `zero_copy_input_boundary=true`
  - deterministic by default

## Build / Availability Requirements
- Real downstream use should require the generated regex backend.
- Startup or build validation should inspect:
  - `parser_embedding_api_contract().supports_regex_generated_backend`
- If building directly from a PGEN checkout, enable the generated parser surface rather than relying on bootstrap-only builds.
- If the generated backend is unavailable, the stable failure mode is:
  - `E_BACKEND_UNAVAILABLE`

## Generated Parser Build Recipe (for downstream consumers like RGX)

`generated/*` is intentionally NOT git-tracked (per the policy in `LIVE_ACHIEVEMENT_STATUS.md` 2026-04-29 slice-5 entry). A fresh PGEN clone does NOT have `generated/regex_parser.rs` on disk. Downstream consumers must regenerate the parser before linking against `--features generated_parsers`. The build recipe below is the maintained, owner-supported path.

### TL;DR — two commands cover everything RGX needs

| When | Command |
|---|---|
| **Normal build** (incremental, idempotent) | `make -C subs/pgen/rust SHELL=/bin/bash regex_parser_bootstrap` |
| **Fresh start** (wipe + rebuild from absolute zero) | `make -C subs/pgen/rust SHELL=/bin/bash regex_parser_fresh` |

**`regex_parser_bootstrap`** handles every state:

- Fresh checkout, no `generated/` files at all → seeds everything from `grammars/*.ebnf` and produces `generated/regex_parser.rs`.
- Warm tree, `generated/ebnf.rs` already present → skips the seed step and just runs the standard `regex_parser` chain.
- Pulling new PGEN commits → run again; Make's incremental dep graph rebuilds only what changed.

It is **idempotent** — safe to run unconditionally on every RGX build.

**`regex_parser_fresh`** is the nuclear option — it runs `clean-all` (drops the entire `generated/` directory and `cargo clean`s `rust/target/`) and then runs `regex_parser_bootstrap`. Use this when something has gotten weird and you want a guaranteed-fresh rebuild from absolute zero.

Subsequent sections explain prerequisites, internals, and recovery paths.

### Prerequisites

| Tool | Version | How to verify |
|---|---|---|
| Rust toolchain | `1.95` or newer (project MSRV) | `rustc --version` |
| `cargo` | (ships with rustup) | `cargo --version` |
| GNU `make` | any recent | `make --version` |
| `bash` | any recent | `bash --version` |

The Rust toolchain floor is documented in `README.md` and enforced via `Cargo.toml`'s `rust-version`.

### Fresh start (clean reset of the PGEN submodule under RGX)

If RGX needs to wipe any cached state in `subs/pgen/` and rebuild the regex parser from absolute scratch (e.g. PGEN's `generated/` ended up in a broken intermediate state, or a partial pull left stale artifacts on disk), the simplest path is the combined wipe-and-rebuild target:

```bash
make -C subs/pgen/rust SHELL=/bin/bash regex_parser_fresh
```

This is **`clean-all` + `regex_parser_bootstrap` in one go**:

1. Drops the entire `generated/` directory.
2. Runs `cargo clean` to drop everything under `rust/target/`.
3. Bootstraps `generated/ebnf.rs` from `grammars/ebnf.ebnf` via the Rust frontend.
4. Runs the full `regex_parser` chain end-to-end.

Result: `generated/regex_parser.rs` is fresh from absolute zero. Typical wall time on a modern laptop: 4–6 minutes (the cargo full rebuild is the long pole).

#### Lower-level alternatives

If you want to clean without immediately rebuilding (e.g. to inspect state, or to clean before a different build sequence):

```bash
# Option 1 — Makefile clean targets:
make -C subs/pgen/rust SHELL=/bin/bash clean       # generated/*.{pl,json,rs,placeholder} + `cargo clean`
make -C subs/pgen/rust SHELL=/bin/bash clean-all   # the above PLUS rm -rf generated/

# Option 2 — manual wipe (if Make targets misbehave or you want ironclad isolation):
rm -rf subs/pgen/generated/                         # all PGEN-generated artifacts
rm -rf subs/pgen/rust/target/                       # all Rust build artifacts

# Option 3 — also reset uncommitted local changes inside subs/pgen.
# WARNING: destructive. Only do this when nothing local is worth keeping.
git -C subs/pgen reset --hard HEAD
git -C subs/pgen clean -fdx generated/ rust/target/
```

After any of these manual cleans, run `make -C subs/pgen/rust SHELL=/bin/bash regex_parser_bootstrap` to repopulate `generated/`.

> **Submodule pinning reminder for RGX**: keep `subs/pgen` pinned to a specific PGEN commit on `main`. After a fresh start + rebuild, the resulting `generated/regex_parser.rs` is fully a function of (a) the pinned commit's PGEN sources and (b) the generated-parser feature flags. Any drift between RGX deployments points to one of those two inputs, not to a non-deterministic build.

### Standard regen (when `generated/ebnf.rs` is already on disk)

The Make dependency graph chains the entire downstream build. From the PGEN repo root (or RGX's `subs/pgen/` submodule path), one command:

```bash
make -C rust SHELL=/bin/bash regex_parser
```

This transitively pulls in the bootstrap-mode AST pipeline binary, the bootstrap-safe annotation parsers (`return_annotation_parser` and `semantic_annotation_parser`), the EBNF frontend binary, the JSON intermediate (`generated/regex.json`), and finally `generated/regex_parser.rs`. The Makefile encodes the dependency chain:

| Target | Depends on | Output |
|---|---|---|
| `regex_parser` | `$(REGEX_JSON) $(RUST_AST_PIPELINE)` | `generated/regex_parser.rs` |
| `$(REGEX_JSON)` | `grammars/regex.ebnf $(RUST_EBNF_FRONTEND_BIN)` | `generated/regex.json` |
| `$(RUST_AST_PIPELINE)` | `$(AST_PIPELINE_SOURCES) $(SEMANTIC_ANNOTATION_PARSER) $(RETURN_ANNOTATION_PARSER)` | `rust/target/debug/ast_pipeline` |
| `$(SEMANTIC_ANNOTATION_PARSER)` | `$(SEMANTIC_ANNOTATION_JSON) $(RUST_AST_PIPELINE_BOOTSTRAP)` | `generated/semantic_annotation_parser.rs` |
| `$(RETURN_ANNOTATION_PARSER)` | `$(RETURN_ANNOTATION_JSON) $(RUST_AST_PIPELINE_BOOTSTRAP)` | `generated/return_annotation_parser.rs` |
| `$(RUST_AST_PIPELINE_BOOTSTRAP)` | `$(AST_PIPELINE_SOURCES)` | `rust/target/debug/ast_pipeline_bootstrap` |
| `$(RUST_EBNF_FRONTEND_BIN)` | `$(AST_PIPELINE_SOURCES) generated/ebnf.rs rust/build.rs` | the EBNF-to-JSON binary |

After `make regex_parser` completes, `generated/regex_parser.rs` is on disk and ready for `--features generated_parsers` builds.

### Cold-zero-clone bootstrap (when `generated/ebnf.rs` does NOT yet exist)

A single make target — `regex_parser_bootstrap` — handles the full cold-clone build, including seeding `generated/ebnf.rs` if it isn't on disk. The target is **idempotent**; running it on a warm tree skips the seed step and just runs the standard `regex_parser` chain.

```bash
make -C rust SHELL=/bin/bash regex_parser_bootstrap
```

That's the recommended single command for downstream consumers. RGX should call this from its build script when bringing up a fresh `subs/pgen/`.

#### What the target does internally

The chicken-and-egg used to be: building `ast_pipeline` with `--features ebnf_dual_run` required `include!("generated/ebnf.rs")`, but that file is itself produced by `ast_pipeline`. The fix (landed alongside this contract version) splits the two: `build.rs` now sets a `has_generated_ebnf_parser` cfg flag only when `generated/ebnf.rs` exists, and the `include!()` site in `lib.rs` is gated on that cfg. The hand-written Rust EBNF frontend (`rust/src/ebnf_frontend.rs`) — whose main parsing path is hand-written and only uses the generated `EbnfParser` as an optional cross-check — is now also gated, so the cross-check is skipped when `generated/ebnf.rs` isn't built yet.

Result: `cargo build --features ebnf_dual_run --bin ast_pipeline` succeeds even with no `generated/` files. The Rust EBNF frontend produces `generated/ebnf.json` from `grammars/ebnf.ebnf`, and the same binary then converts JSON → `generated/ebnf.rs`. After that initial seed, the binary is rebuilt (now with the cross-check active) and the normal `regex_parser` chain runs to completion.

The Perl-based fallback `tools/ebnf_to_json.pl` is retained but is **not** the recommended path — it has known feature-coverage limitations that the Rust frontend doesn't share. Don't reach for it unless you have no Rust toolchain.

#### Manual decomposition (if you want to debug each step)

```bash
# Step A — build ast_pipeline. Compiles even with no generated/ebnf.rs
# because has_generated_ebnf_parser is unset; the EbnfParser cross-check
# inside ebnf_frontend.rs is cfg-gated and elided.
cargo build --manifest-path rust/Cargo.toml --features ebnf_dual_run --bin ast_pipeline

# Step B — Rust EBNF frontend: grammars/ebnf.ebnf -> generated/ebnf.json.
mkdir -p generated
rust/target/debug/ast_pipeline grammars/ebnf.ebnf --emit-raw-ast-json generated/ebnf.json

# Step C — JSON -> generated/ebnf.rs.
rust/target/debug/ast_pipeline \
    --generate-parser --debug --eliminate-left-recursion \
    generated/ebnf.json -o generated/ebnf.rs

# Step D — rebuild ast_pipeline. Now has_generated_ebnf_parser is set;
# subsequent runs of the EBNF frontend cross-check the generated parser
# against the hand-written walk for verification.
cargo build --manifest-path rust/Cargo.toml --features ebnf_dual_run --bin ast_pipeline

# Step E — standard regex_parser chain.
make -C rust SHELL=/bin/bash regex_parser
```

A downstream consumer like RGX should typically:
- Call `make -C subs/pgen/rust SHELL=/bin/bash regex_parser_bootstrap` from its build script. Idempotent — safe to run on every build.
- Treat the seed step as a rare event tied to PGEN bootstrap-grammar changes; subsequent runs on a warm tree skip step A-C and just run step D-E.

### Downstream usage

```bash
# Build the embedding-API library + binaries that include the regex parser.
cargo build --manifest-path rust/Cargo.toml --features generated_parsers

# Probe the regex parser end-to-end.
echo -n 'a*' > /tmp/sample.txt
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin parseability_probe -- \
    --parse regex /tmp/sample.txt --profile regex_default
cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin parseability_probe -- \
    --parse-dump-ast-pretty regex /tmp/sample.txt /tmp/sample_ast.json --profile regex_default
```

### Determinism check

Per slice 5's verification policy, the regen path is deterministic given the same input. To confirm a clean regen:

```bash
make -C rust SHELL=/bin/bash regex_parser \
    && shasum -a 256 generated/regex_parser.rs > /tmp/sha1
make -C rust SHELL=/bin/bash regex_parser \
    && shasum -a 256 generated/regex_parser.rs > /tmp/sha2
diff /tmp/sha1 /tmp/sha2   # must be empty
```

Two consecutive regens against the same `grammars/regex.ebnf` and the same PGEN source must produce byte-identical `generated/regex_parser.rs`. The SHA itself shifts whenever the grammar source or the PGEN pipeline source legitimately changes; the contract is determinism, not a fixed SHA.

### Pulling a new PGEN

When a downstream consumer pulls new PGEN commits, regenerate the parser to pick up any grammar/codegen changes:

```bash
git -C <pgen-submodule-path> pull --ff-only
make -C rust SHELL=/bin/bash regex_parser     # incremental: only rebuilds if inputs changed
```

If the changes touch the bootstrap chain (return_annotation, semantic_annotation grammars or any pipeline code those parsers depend on), redo steps 2-4 of the cold-checkout recipe above.

### What can go wrong

| Symptom | Cause | Fix |
|---|---|---|
| `cargo build --features generated_parsers` errors `file not found: generated/regex_parser.rs` | bootstrap target not run yet on a fresh clone | run `make -C rust SHELL=/bin/bash regex_parser_bootstrap` |
| `make regex_parser` errors at the EBNF-to-JSON step (`ast_pipeline: not found`) | the bin hasn't been built yet | use `make regex_parser_bootstrap` instead — it builds `ast_pipeline` first |
| `regex_parser_bootstrap` fails at the seed step with a Rust compile error mentioning `ebnf_generated_parser` or `EbnfParser` | likely an outdated PGEN checkout where the cfg gating wasn't yet in place | pull PGEN to a commit at or after the contract `1.1.35` cold-clone fix |
| Two consecutive `make regex_parser_bootstrap` produce different `generated/regex_parser.rs` SHAs | non-determinism bug — please file an issue | report; do NOT ship the unstable parser |
| Grammar parse error during the EBNF-to-JSON step | a stale `grammars/regex.ebnf` from a partial pull, or a hand-edit drift | `git -C subs/pgen checkout grammars/regex.ebnf`; rerun bootstrap |
| `make: *** No rule to make target ...` | called from the wrong directory | always use `make -C subs/pgen/rust ...` (or `cd subs/pgen && make -C rust ...`) |
| Build succeeds but `parser_embedding_api_contract().supports_regex_generated_backend` is `false` at runtime | RGX's own build wasn't rebuilt with `--features generated_parsers` after the bootstrap | rebuild RGX with the feature enabled |

### Optional: typed-entry-point fast path

To get the opt-in `parse_regex_typed()` typed entry point (see release `1.1.30`/`1.1.32` highlights), regenerate with `--enable-parser-hooks`:

```bash
# Replace step 4 with:
rust/target/debug/ast_pipeline \
    --generate-parser --debug --trace --eliminate-left-recursion \
    --enable-parser-hooks \
    generated/regex.json -o generated/regex_parser.rs
```

The default `make regex_parser` target does NOT register the hook, so the default emit doesn't carry the typed methods. The legacy `parse_regex()` API is unchanged either way.

## Stable Diagnostics Contract
- Stable diagnostic codes:
  - `E_BACKEND_UNAVAILABLE`
  - `E_PARSE_FAILURE`
  - `E_INPUT_TOO_LARGE`
  - `E_INVALID_LIMITS`
  - `E_INVALID_ARGUMENT`
  - `E_UNSUPPORTED_PROFILE`
- Parse diagnostics now expose a stable optional machine-localizable location object:
  - `location.byte_offset`
  - `location.line`
  - `location.column`
- The location object is emitted when the selected parser backend can localize the parse failure precisely.
- Regex parse failures through the generated regex backend are expected to populate this location object.

## Stable AST-Dump Schema Contract
- Regex AST-dump JSON is transport-stable and schema-stable at schema version `1`.
- Schema version `1` stabilizes the recursive node envelope and variant encoding:
  - top-level node object fields:
    - `rule_name`
    - `span`
    - `content`
  - `span` fields:
    - `start`
    - `end`
  - `content` is an externally tagged JSON object with exactly one active variant:
    - `Terminal`
    - `TransformedTerminal`
    - `Sequence`
    - `Alternative`
    - `Quantified`
- Representative shape:

```json
{
  "rule_name": "regex",
  "span": {
    "start": 0,
    "end": 3
  },
  "content": {
    "Sequence": [
      {
        "rule_name": "atom",
        "span": {
          "start": 0,
          "end": 1
        },
        "content": {
          "Terminal": "a"
        }
      }
    ]
  }
}
```

- This schema contract is about JSON shape, field names, and variant encoding.
- Integration contract `1.1.21` explicitly guarantees the POSIX-class-plus-literals character-class shape:
  - `[[:digit:]-   ]` transports `[:digit:]` through `posix_class` with `posix_name = "digit"`
  - the following `-` and ordinary spaces transport as separate `class_literal` items, not as a `class_range`
  - this is a contract clarification over the existing PCRE2 character-class grammar shape; parser release remains `1.1.19` and regex AST schema version stays `1`
- Integration contract `1.1.22` explicitly guarantees resource-depth resilience for two legal PCRE2 conformance shapes:
  - a deeply nested `80`-capture pattern followed by `\80`
  - a grammar-like recursive named-group interpolation pattern using nested `\g<...>` references
  - this is a host/code-generation resilience guarantee over parser release `1.1.20`; it does not introduce a new AST schema version
- Integration contract `1.1.23` explicitly guarantees the 2026-04-14 PCRE2 source-derived audit slice:
  - grammar additions include `\K`, one-digit and whitespace-braced hex/octal escapes, string callouts, `(*atomic:...)`, non-atomic lookarounds, scan-substring groups, script-run groups, strict modifier forms, and strict no-whitespace VERSION conditionals
  - compile-contract validation now covers numeric callout bounds, PCRE2 start-option and verb tables, non-ACCEPT verb quantification, `\K` lookaround restrictions, forbidden class escapes, POSIX class-name validation, scan-substring reference existence, unsupported default escapes, and `(?R1)` rejection
  - comma-only `{,}` is now a literal brace/comma/brace sequence, not a counted quantifier
- Integration contract `1.1.24` explicitly guarantees the short-property and quoted-class PCRE2 source slice:
  - `\pL` and `\PN` transport through `property_escape`, not through `simple_escape` fallback plus a literal property letter
  - supported short property letters are the PCRE2 one-letter general categories `C`, `L`, `M`, `N`, `P`, `S`, and `Z`, in either case
  - `[z\Qa-d]\E]` transports the quoted class region through `quoted_class_literal`, including the quoted `]` as class content
  - malformed short property escapes and empty quoted class regions that leave no substantive class item are rejected by the generated-host compile contract
- Integration contract `1.1.25` explicitly guarantees the bounded-lookbehind, Unicode-name, and orphan-class-`\E` PCRE2 source slice:
  - `(?<=a{1,3})b` and `(?<=a(*ACCEPT)b)c` are accepted, while unbounded forms such as `(?<=a+)b` remain rejected
  - `(?'ABáC'...)\g{ABáC}` transports the Unicode capture name through the generated regex backend, with PCRE2's non-empty, non-leading-digit, and `128`-byte name limit enforced by the generated-host compile contract
  - `^[\Eabc]` transports orphan class `\E` through `stray_class_end_quote` while keeping the substantive class atoms as literals
  - `^[a-\Ec]` permits zero-width class markers before the range right endpoint without treating `\E` as a substantive escaped endpoint
  - malformed `\k` named-reference spellings and overlong capture names are rejected by the generated-host compile contract
- Integration contract `1.1.26` explicitly guarantees the single-code-unit escape and conditional-callout assertion PCRE2 source slice:
  - `ab\Cde` transports `\C` as `single_byte_escape`, not `simple_escape`
  - `^(?(?C25)(?=abc)abcd|xyz)` transports the condition through `condition_callout_assertion`, preserving `condition_callout = "?C25)"`, `callout_arg = "25"`, and `condition_assertion = "?=abc"`
  - `^(?(?C$abc$)(?=abc)abcd|xyz)` uses the same conditional-callout assertion shape while preserving the dollar-delimited string callout payload through `callout_arg`, `callout_string`, and `callout_dollar_string`
  - compile-option-sensitive behavior for `\C`, such as PCRE2's `PCRE2_NEVER_BACKSLASH_C` mode or UTF lookbehind restrictions, remains outside the stable `regex_default` syntax contract unless PGEN publishes a profile option for it
- Integration contract `1.1.27` explicitly guarantees the PCRE2 POSIX word-boundary alias and DEFINE-in-lookbehind source slice:
  - `[[:<:]]red[[:>:]]` transports `[[:<:]]` and `[[:>:]]` through exact `posix_word_boundary_alias` atoms
  - `[a[:<:]]` remains rejected because PCRE2 recognizes only the exact standalone alias sequences, not `<` or `>` as general `posix_name` values
  - `(?<=X(?(DEFINE)(.*))Y).` is accepted because `(?(DEFINE)...)` is declarative and zero-width for lookbehind-length validation
  - unbounded quantifiers in ordinary lookbehind bodies remain rejected by the generated-host compile contract
- Integration contract `1.1.28` explicitly guarantees the PCRE2 UTF-alias and scan-substring forward-reference source slice:
  - `(*UTF8)\x{1234}` is accepted as a UTF start-option alias rather than rejected as an unknown verb/start option
  - `(*scs:(1)a)(a)|x` is accepted because absolute scan-substring references are checked against the full pattern capture count, not only captures declared before the scan-substring group
  - `(*scs:(<GOOD_NAME>)a)(?<GOOD_NAME>a)` is accepted because named scan-substring references are checked against the full pattern name inventory
  - forward relative scan-substring references such as `+1` and `+2` are accepted when they resolve into the full capture inventory; negative relative references still require an already-available prior capture
- Integration contract `1.1.29` explicitly guarantees the PCRE2 class-quote and quoted-literal source slice:
  - `a[\NB]c` is rejected because plain `\N` is not a legal escape inside a character class
  - `^[\Qa\E-\Qz\E]+` transports the class body as one `class_range` with two `quoted_class_range_atom` endpoints
  - `[\d-x]` and `[\pL-x]` are rejected because shorthand and property escapes are not literal character range endpoints
  - `\Qabc\$xyz\E` transports as one `quoted_literal`, preserving the body backslash-dollar through `quoted_literal_escaped_char` rather than `simple_escape`
- Integration contract `1.1.30` explicitly guarantees the wide braced-hex class-range endpoint source slice:
  - `[z-\x{100}]` is accepted because the generated-host compile contract compares `z` against decoded codepoint `0x100`, not against the byte value for `x`
  - `[\x{100}-z]` remains rejected as a descending range
  - malformed braced class escapes remain literal transport for the PCRE2 `bad_escape_is_literal` oracle lane rather than becoming invented numeric-decoder failures
- Integration contract `1.1.31` explicitly guarantees the bare-octal class-range endpoint source slice:
  - `[\000-\037]`, `[\010-\037]`, and `[a-\377]` are accepted because bare `\NNN` endpoints are consumed atomically and compared by decoded octal codepoint
  - `[\001-\x1f]` and `[\001-\x{1f}]` are accepted because bare-octal endpoints compare correctly against single-byte and braced hex endpoints
  - `[\x1f-\0]` is rejected as a descending range because decoded endpoints are `31..0`
- Integration contract `1.1.20` explicitly guarantees PCRE2 braced `\k{...}` named backreferences with optional space/tab padding:
  - `(?'name'ab)\k{ name }(?P=name)` transports the padded named backreference as `backreference` -> `braced_name_ref`
  - the named payload transports under `name = "name"` while the enclosing `braced_name_ref` rule text remains `{ name }`
  - the contract slice is shape-based for the PCRE2 brace-delimited name-reference family, not a literal special case for `name`
- Integration contract `1.1.19` explicitly guarantees PCRE2 braced `\g{...}` numeric backreferences with optional space/tab padding:
  - `(A)(\g{ -2 }B)` transports the reference as `backreference` -> `subroutine_ref` -> `braced_subroutine_ref` -> `signed_digits`
  - the signed numeric payload transports as `signed_digits = "-2"` while the enclosing `subroutine_ref` rule text remains `{ -2 }`
  - `\g{...}` with brace delimiters remains a backreference form; `\g<...>` and `\g'...'` remain Oniguruma-style subroutine-reference forms
  - this contract slice is shape-based for the PCRE2 braced numeric reference family, not a literal special case for `-2`
- Integration contract `1.1.18` explicitly guarantees PCRE2 atomic alpha-lookaround assertion aliases in conditional assertion position:
  - `(?(*pla:foo).{6}|a..)` transports the condition as `condition_assertion` -> `alpha_condition_assertion` -> `atomic_alpha_lookaround_name`, with `atomic_alpha_lookaround_name = "pla"`
  - the modeled family includes `pla` / `positive_lookahead`, `nla` / `negative_lookahead`, `plb` / `positive_lookbehind`, and `nlb` / `negative_lookbehind`
  - non-atomic lookaround and script-run alpha assertion aliases are not part of this `regex_default` contract slice yet
- Integration contract `1.1.17` explicitly guarantees the default PCRE2 backtracking-control directive payload shape, including MARK/PRUNE/SKIP/THEN:
  - directive payload characters are any characters except the verb-closing `)` under the stable `regex_default` profile
  - `(*MARK:m'm)(*PRUNE:p"p)(*SKIP:s(s)` transports the three payloads as `directive_payload_simple` values `m'm`, `p"p`, and `s(s`
  - `(*THEN:m(m)(?&y)(?(DEFINE)(?<y>b))` transports the leading directive as `directive_name = "THEN"` and `directive_payload_simple = "m(m"`
  - MARK shorthand still uses `(*:NAME)`; named backtracking-control verbs use `(*VERB:NAME)` and transport the name through `directive_payload_suffix`
  - `PCRE2_ALT_VERBNAMES` escape-processing semantics are not modeled by `regex_default`
  - this contract slice is intentionally shape-based: the listed payload samples are witnesses for the non-`)` rule, not the complete accepted payload set
- Integration contract `1.1.16` explicitly guarantees PCRE2 MARK shorthand directive payloads with literal `(`:
  - `(*:m(m)(?&y)(?(DEFINE)(?<y>b))` transports the leading directive as `directive_verb` with `directive_mark_shorthand` payload `m(m`
  - `(*PRUNE:m(m)(?&y)(?(DEFINE)(?<y>b))` transports the leading directive as `directive_verb` with `directive_name = "PRUNE"` and `directive_payload_simple = "m(m"`
  - downstream consumers should treat literal `(` as payload text inside directive verbs until the verb-closing `)` is reached
- Integration contract `1.1.15` explicitly guarantees PCRE2 quoted literals as first-class `quoted_literal` atoms:
  - `abc\Q(*+|\Eabc` transports the quoted section through `quoted_literal` with rule text `\Q(*+|\E`
  - downstream consumers should treat `quoted_literal` as an atom-level payload sibling of `literal` and `escape`
- Integration contract `1.1.14` explicitly guarantees that valid POSIX character classes inside character classes remain transported as `class_item` -> `posix_class` rather than being degraded to literal text:
  - `[[:space:]]+` transports the POSIX class through `posix_class` with `posix_name = "space"`
  - `[[:blank:]]+` transports the POSIX class through `posix_class` with `posix_name = "blank"`
  - `^[:a[:digit:]]+` transports the leading `:` and `a` as `class_literal` items and the embedded POSIX class through `posix_class` with `posix_name = "digit"`
  - `^[:a[:digit:]:b]+` transports the surrounding `:`, `a`, `:`, and `b` as `class_literal` items while preserving the embedded `digit` POSIX class through `posix_class`
  - `[[:digit:]-]+` transports `[:digit:]` through `posix_class` with `posix_name = "digit"` and the trailing `-` as a separate `class_literal`
  - `[[:digit:]-   ]` extends the same guarantee to the PCRE2 class item shape where the trailing `-` and ordinary spaces are separate `class_literal` items rather than a `class_range`
  - downstream consumers should treat `posix_class` as a first-class `class_item` variant alongside `class_range`, `class_literal`, and `class_escape`
- Contract `1.1.31` publishes parser release `1.1.29` bare-octal class-range endpoint ordering by decoded octal codepoint value, carries forward parser release `1.1.28` braced-hex class-range endpoint ordering, parser release `1.1.27` PCRE2 character-class quoted range endpoints, literal backslash preservation inside `\Q...\E`, plain-class-`\N` rejection, and nonliteral class-range endpoint rejection, parser release `1.1.26` PCRE2 UTF width start-option aliases and scan-substring forward-reference validation, parser release `1.1.25` PCRE2 POSIX word-boundary aliases and DEFINE-in-lookbehind length handling, parser release `1.1.24` PCRE2 single-code-unit `\C` transport and callout-prefixed conditional assertion support, parser release `1.1.23` PCRE2 bounded-lookbehind, Unicode-name, and orphan-class-`\E` support, parser release `1.1.22` PCRE2 short-property and quoted-class literal support, parser release `1.1.21` PCRE2 source-derived syntax and compile-contract alignment, and parser release `1.1.20` resource-depth resilience for legal deep PCRE2 conformance inputs, contract `1.1.21` for the `[[:digit:]-   ]` POSIX-class-plus-literals AST shape, parser release `1.1.19` PCRE2-compatible braced `\k{...}` named backreference whitespace handling, parser release `1.1.18` braced `\g{...}` numeric backreference whitespace handling, parser release `1.1.17` atomic alpha-lookaround assertion aliases, parser release `1.1.16` generalized directive payload transport to the default non-`)` verb-name shape, parser release `1.1.15` literal-`(` directive payload support, parser release `1.1.14` PCRE2-compatible `\Q...\E` quoted literal transport, parser release `1.1.13` PCRE2-compatible fallback for malformed POSIX-class opener text inside character classes, control-escape validator hardening, malformed counted-quantifier literal spellings, returned-capture subroutine syntax, Unicode literal support, and deeper nested-group headroom, all while keeping this JSON schema version stable:
  - `[\000-\037]` and `[a-\377]` now pass the generated-host compile contract because bare-octal endpoints are decoded before range-order comparison
  - `[\x1f-\0]` is now rejected by the generated-host compile contract as a descending range
  - `[z-\x{100}]` now passes the generated-host compile contract because braced hex endpoints are decoded before range-order comparison
  - `[\x{100}-z]` remains rejected by the generated-host compile contract as a descending range
  - `a[\NB]c` is now rejected by the generated-host compile contract because plain `\N` is not legal inside a PCRE2 character class
  - `^[\Qa\E-\Qz\E]+` now transports the class body as `class_range` with `quoted_class_range_atom` endpoints
  - `[\d-x]` and `[\pL-x]` are now rejected because shorthand/property escapes are not literal class-range endpoints
  - `\Qabc\$xyz\E` now transports as one `quoted_literal` preserving `\$` as literal quoted content
  - `(*UTF8)\x{1234}` now passes the generated-host PCRE2 start-option validator as a UTF width alias
  - `(*scs:(1)a)(a)|x` now passes scan-substring capture-list validation even though capture group `1` appears later in the pattern
  - `(*scs:(<GOOD_NAME>)a)(?<GOOD_NAME>a)` now passes named scan-substring capture-list validation against a later named group
  - `[[:<:]]red[[:>:]]` now transports PCRE2 word-boundary aliases as exact `posix_word_boundary_alias` atoms
  - `(?<=X(?(DEFINE)(.*))Y).` now passes the generated-host lookbehind compile contract because `DEFINE` conditionals are declarative and zero-width for length analysis
  - `ab\Cde` now transports `\C` as `single_byte_escape`
  - `^(?(?C25)(?=abc)abcd|xyz)` now transports the condition through `condition_callout_assertion`
  - `^(?(?C$abc$)(?=abc)abcd|xyz)` now preserves the same conditional-callout assertion shape with a string callout payload
  - `(?<=a{1,3})b` and `(?<=a(*ACCEPT)b)c` now pass the generated-host compile contract
  - `(?<=a+)b`, `(?<=a*)b`, and `(?<=a{2,})b` remain rejected as unbounded variable-length lookbehind
  - `(?'ABáC'...)\g{ABáC}` now transports through the Unicode-name path instead of failing at the first non-ASCII name byte
  - `^[\Eabc]` now treats orphan class `\E` as a zero-width `stray_class_end_quote`
  - `^[a-\Ec]` now skips the zero-width `\E` marker before the range right endpoint
  - malformed `\k`, empty `\k{}`, names beginning with digits, and names longer than PCRE2's `128`-byte limit are rejected by the generated-host compile contract
  - `\pL` now transports as `property_escape` with `short_prop_letter`, not as `simple_escape("p")` plus literal `L`
  - `[z\Qa-d]\E]` now transports the class quote as `quoted_class_literal`
  - the `PGEN-RGX-0054` pattern with `80` nested capturing groups plus `\80` now parses without host stack abort or generated recursion-guard rejection
  - the `PGEN-RGX-0055` recursive named-group interpolation pattern now parses without host stack abort
  - `(?'name'ab)\k{ name }(?P=name)` now transports the padded named backreference as `backreference` -> `braced_name_ref`, not `simple_escape("k")` plus literal `{ name }`
  - `(A)(\g{ -2 }B)` now transports the padded relative reference as `backreference` -> `subroutine_ref` -> `braced_subroutine_ref` -> `signed_digits`, not `simple_escape("g")` plus literal `{ -2 }`
  - `(?(*pla:foo).{6}|a..)` now parses the leading conditional alpha-lookahead assertion condition instead of rejecting at byte `0`
  - `(*:m(m)(?&y)(?(DEFINE)(?<y>b))` now parses the leading `(*:m(m)` directive before the following subroutine call and `DEFINE` conditional instead of rejecting at byte `0`
  - `(*PRUNE:m(m)(?&y)(?(DEFINE)(?<y>b))` now parses the leading `(*PRUNE:m(m)` directive through the same payload-character widening
  - `abc\Q(*+|\Eabc` now transports the quoted metacharacter segment through `quoted_literal` instead of treating `(` as active group syntax
  - `([[:]+)` now treats the inner `[` and `:` as ordinary character-class literals once the stricter POSIX-class form fails
  - `([[=]+)` now likewise treats the inner `[` and `=` as ordinary character-class literals
  - `([[.]+)` now likewise treats the inner `[` and `.` as ordinary character-class literals
  - `[[,abc,]+]` now treats the inner `[` and comma-separated payload as ordinary character-class literals, then parses the trailing `]` as a literal atom after the class quantifier
  - `[[:abcd:xyz]]` now treats the malformed POSIX-class body as ordinary class literals, then parses the trailing `]` as a literal atom after the class
  - `^\ca\cA\c[;\c:` now treats `\c[` as a complete control escape instead of re-reading `[` as an unterminated character class opener
  - `a{1,2,3}b` now transports the malformed counted-quantifier body as literal text instead of rejecting after `a`
  - `X{`, `X{A`, `X{1234`, and `X{1,` now preserve the unterminated brace spellings as literals
  - `X{12ABC}`, `X{,9`, and `X{,9]` now preserve malformed alphanumeric/left-open brace forms as literals
  - `a{(?#XYZ),2}` now preserves the surrounding brace/comma/digit text while still transporting `(?#XYZ)` through `comment_group`
  - `(?(VERSION>=10.0)cat|dog)` now transports the condition as `version_condition` with `version_operator = ">="` and `version_number = "10.0"`
  - `(?(VERSION=10)cat|dog)` now transports as a strict PCRE2 `version_condition`, while whitespace-bearing forms such as `(?(VERSION >= 10)cat|dog)` are rejected by the generated-host contract
  - `a{,}b` now transports `{`, `,`, and `}` as literal text rather than a `counted_quantifier`
  - `(?C"alpha""beta")` and `(?C{left}}right})` now transport as string callouts with doubled-delimiter payload escaping
  - `(?*foo)`, `(*napla:foo)`, `(*atomic:foo)`, `(*sr:foo)`, and `(.)(*scs:(1)foo)` now parse through the corresponding PCRE2 group families
  - `(?R1)` is rejected; use `(?R)` for whole-pattern recursion or `(?R(1))` for returned-capture whole-pattern recursion
  - `(?1(1))` now transports as `subroutine_call` containing `returned_capture_subroutine`, `subroutine_target`, `returned_capture_group_list`, and `returned_capture_group`
  - `(?&callee(+1,<cap>,'alt'))` now preserves mixed numeric and named returned-capture grouplist entries without falling back to `inline_modifiers`
  - `🎉` now transports as a single `literal` node spanning the full UTF-8 codepoint
  - `café` now transports as four `literal` nodes, preserving `é` as the final multibyte atom
  - nested capturing groups remain accepted at least through `50` levels
  - `(?(R)a|b)` now transports `condition` through `recursion_condition`
  - `(a)(?(R1)b|c)` now transports `condition` through `recursion_condition` instead of `name`
  - `(?{native:validate_word})` now preserves `code_content = "validate_word"` instead of starting one byte late
  - `(?{lua:return true})` now transports as `code_block_lang` plus `code_lang`, not `code_block_plain`
  - `(?{rhai:...})`, `(?{native:...})`, and `(?{wasm:...})` are now accepted through the same tagged code-block structure
  - `\g<1>` now transports as outer `backreference` plus inner `subroutine_ref` / `signed_digits`, not `simple_escape("g")` plus literal `<`, `1`, `>`
  - `\o{101}` now transports as outer `escape` plus inner `octal_escape` / `octal_digits`, not `simple_escape` plus counted `quantifier`
  - `(?R)` now transports as `subroutine_call`
  - `\1` now transports as `backreference`
  - `(?(1)a|b)` now transports with separate `yes_branch` / `no_branch`
  - `(?(R&word)a|b)` now parses and transports `R&word` as `recursion_condition` while preserving nested `name = "word"`
  - `ab+` now transports with final-atom quantifier binding
  - brace-style numeric escapes no longer trip counted-quantifier validation during post-parse compile checks
- Downstream consumers that interpret specific `rule_name` values should pin to a parser release version and rerun their own AST compatibility suite on upgrade.
- This document does not promise stable internal Rust AST node types.

## Current External Hardening Baseline
- The maintained PCRE2 compile-oracle lane is now part of the downstream trust story for the regex release.
- Current tracked baseline:
  - `cases_executed=2195`
  - `expected_parse_ok_total=1613`
  - `expected_parse_fail_total=582`
  - `parse_expectation_match_total=1843`
  - `parse_expectation_mismatch_total=352`
  - `false_accept_total=307`
  - `false_reject_total=45`
- This does not reopen the closed `regex` family row by itself, but it is the main maintained future hardening lane for downstream trust widening.

## Published Regex Flavor Summary
- The currently published regex parser accepts:
  - empty regex
  - raw regex bodies, not host-language delimiter wrappers
  - alternation and concatenation
  - whole-pattern recursion `(?R)`
  - returned-capture subroutine calls such as `(?1(1))` and `(?&name(+1,<cap>))`
  - capturing, noncapturing, named, and atomic groups
  - lookahead and lookbehind assertions
  - bounded variable-length lookbehind such as `(?<=a{1,3})b`
  - PCRE2 control verbs inside lookbehind such as `(?<=a(*ACCEPT)b)c`
  - greedy, lazy, and possessive quantifiers
  - counted quantifier forms such as `{3}`, `{2,}`, `{2,4}`, and `{,4}`
  - comma-only `{,}` as literal text rather than a counted quantifier
  - final-atom quantifier binding for literal runs, so `ab+` means literal `a` followed by quantified `b`
  - char classes, negated char classes, ranges, quoted class literals, orphan class `\E` as a zero-width marker, and POSIX classes
  - negated POSIX classes such as `[[:^alnum:]]`
  - PCRE2 POSIX word-boundary aliases `[[:<:]]` and `[[:>:]]`
  - anchors including `^`, `$`, `\A`, `\Z`, `\z`, `\b`, `\B`, and `\G`
  - PCRE2 single-code-unit escape `\C`, transported as `single_byte_escape`
  - Unicode property escapes including braced forms such as `\p{L}` and short forms such as `\pL`
  - backreferences including `\1`, `\k<name>`, `\k'name'`, and `\k{name}`, with numeric forms preserved as backreference constructs rather than generic escapes
  - Unicode capture names and named references such as `(?'ABáC'...)\g{ABáC}`, under PCRE2's non-empty, non-leading-digit, and `128`-byte name contract
  - subroutine-reference forms such as `\g{1}` and `\g<1>`, with numeric angle form preserved as `backreference` plus `subroutine_ref`
  - parenthesized returned-capture subroutine forms that preserve a comma-separated return grouplist
  - inline modifiers and scoped modifiers
  - conditional regex forms whose condition may be:
    - a lookaround assertion
    - a PCRE2 VERSION comparison such as `VERSION>=10.0` or `VERSION=10`
    - a bare name
    - an explicit name reference
    - digits
    - signed digits
    - a recursion condition such as `R`, `R1`, or `R&name`
    - a callout-prefixed assertion such as `?C25)(?=abc` inside `^(?(?C25)(?=abc)abcd|xyz)`
    - a `DEFINE` declaration such as `(?(DEFINE)(?<name>...))`
  - explicit conditional false-branch transport, so `(?(1)a|b)` preserves distinct `yes_branch` and `no_branch` spans
  - named recursion conditions such as `(?(R&word)a|b)`
  - embedded code-block syntax such as `(?{...})` and language-tagged variants
- Embedded code-block parser-layer contract:
  - plain `(?{...})` is preserved as opaque generic payload
  - `lua`, `js`, `javascript`, and `rhai` payloads are preserved as opaque source-body payloads
  - `native` and `wasm` payloads are preserved as tagged opaque/reference-style payloads
  - parser-layer structural handling currently guarantees:
    - balanced braces
    - single-quoted strings
    - double-quoted strings
    - escaped characters
- Generated-host compile-contract safeguards:
  - the generated regex host path additionally rejects several obvious compile-invalid forms even if the raw grammar shape parsed successfully
  - currently enforced rejections include:
    - unsupported `\i`
    - counted quantifier minimum/maximum inversions such as `{5,4}`
    - counted quantifier bounds above `65535`
    - forbidden character-class escapes such as `[\B]`, `[\R]`, and `[\X]`
    - descending character-class ranges such as `[z-a]`
    - quantified anchors such as `^+`
    - unbounded variable-length lookbehind such as `(?<=a+)b`
    - `\K` inside lookarounds
    - numeric callouts above `255`
    - invalid PCRE2 verb/start-option spellings and quantified non-ACCEPT verbs
    - malformed short Unicode property escapes such as `\pA`
    - empty quoted class regions when they leave no substantive class atom or form an invalid range endpoint
  - malformed named-reference escapes such as bare `\k`, empty `\k{}`, names beginning with digits, and overlong capture names
  - scan-substring references to captures that do not exist in the whole pattern, while PCRE2-compatible forward absolute, named, and positive-relative references remain accepted
- Character-class AST adapter contract:
  - `class_item` variants currently include `class_range`, `class_literal`, `class_escape`, `quoted_class_literal`, `stray_class_end_quote`, and `posix_class`
  - `stray_class_end_quote` is a zero-width PCRE2 scanner marker for orphan `\E`; downstream adapters should not treat it as a literal `E` atom
  - `posix_class` carries standard POSIX class spellings through `posix_name`, including names such as `space`, `blank`, `digit`, `alnum`, and `xdigit`
  - PCRE2's word-boundary compatibility aliases are not general POSIX names; exact `[[:<:]]` and `[[:>:]]` sequences transport through `posix_word_boundary_alias`
  - valid POSIX classes are intentionally not flattened into literal text, because downstream engines need to preserve their range semantics
- The current detailed flavor description and measured operational baseline live in `PGEN_USER_GUIDE.md`.
- Representative accepted examples for the current published flavor include:
  - `ab+`
  - `\pL`
  - `[z\Qa-d]\E]`
  - `^[\Eabc]`
  - `^[a-\Ec]`
  - `(?<=a{1,3})b`
  - `(?<=a(*ACCEPT)b)c`
  - `[[:<:]]red[[:>:]]`
  - `(?<=X(?(DEFINE)(.*))Y).`
  - `(?'ABáC'...)\g{ABáC}`
  - `\o{101}`
  - `\g<1>`
  - `(?1(1))`
  - `(?&callee(+1,<cap>,'alt'))`
  - `(?(VERSION>=10.0)cat|dog)`
  - `(?(VERSION=10)cat|dog)`
  - `(?{lua:return true})`
  - `(?{rhai:let x = 1;})`
  - `(?{native:callback_name})`
  - `(?{wasm:module:function})`
  - `(?R)`
  - `(a)\1`
  - `(?(1)a|b)`
  - `(?<A>foo)-\k{A}`
- The current parser contract does not promise:
  - slash-delimited host literal parsing such as `/pattern/flags` as a dedicated wrapper syntax
  - arbitrary valid Lua, JavaScript, or Rhai source acceptance beyond the structural forms listed above
  - JavaScript comment/template-literal shielding or Lua long-bracket shielding as part of the current published parser contract
  - parser-owned validation of `native` / `wasm` reference payload formats
  - runtime execution semantics for embedded code blocks
  - semantic equivalence with every regex engine on earth
  - a stable typed Rust AST API beyond the JSON schema described above

## Downstream Integration Checklist
1. Depend on the stable host module `pgen::embedding_api`.
2. Use profile `regex_default`.
3. Check `parser_embedding_api_contract().supports_regex_generated_backend` at startup or build validation time.
4. Record both:
   - `parser_embedding_api_contract().regex_parser_release_version`
   - `parser_embedding_api_contract().regex_integration_contract_version`
   - and copy those exact values into bug reports instead of inferring them from memory or stale docs
5. Use `parse_regex_default_result(...)` for accept/reject + diagnostics flows.
6. Use `parse_regex_default_ast_dump(...)` only if JSON AST transport is genuinely needed.
7. Treat `E_BACKEND_UNAVAILABLE`, `E_PARSE_FAILURE`, and `E_INPUT_TOO_LARGE` as first-class expected error modes.
8. Keep a downstream-owned regex acceptance/rejection corpus and run it alongside PGEN's own gate stack.
9. When reporting a bug, follow the quick path below and the full protocol in `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`.
10. Once a real bug is confirmed, expect it to be tracked under the regex family/profile in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` until the fix is released.

## Issue Reporting Quick Path
- Every actionable regex parser bug report should include:
  - PGEN commit ID or released crate version
  - regex parser release version
  - regex integration contract version
  - parser family/profile:
    - `regex`
    - `regex_default`
  - exact failing input file
  - expected vs actual behavior
  - pretty-printed `parser_embedding_api_contract()` JSON
  - pretty-printed parse outcome JSON
  - pretty-printed AST dump JSON when the parse succeeds but the structure is wrong
- If the downstream project can run a PGEN checkout, also capture parser trace artifacts:

```bash
export PGEN_TRACE_VERBOSITY=debug

cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin parseability_probe -- \
  --parse regex repro_input.txt \
  --profile regex_default \
  --trace \
  --trace-log-file pgen_trace.log

cargo run --manifest-path rust/Cargo.toml --features generated_parsers --bin parseability_probe -- \
  --parse-dump-ast-pretty regex repro_input.txt pgen_ast_dump.json \
  --profile regex_default \
  --trace \
  --trace-log-file pgen_trace.log
```

- If the downstream project only has the Rust embedding API, capture:
  - `parser_embedding_api_contract()`
  - `parse_regex_default(...)` or `parse_regex_default_result(...)`
  - `parse_regex_default_ast_dump(...)` when AST structure is relevant
- For version fields, copy directly from:
  - `parser_embedding_api_contract().regex_parser_release_version`
  - `parser_embedding_api_contract().regex_integration_contract_version`
- For a broader description of what the published regex parser is expected to accept, consult the regex-flavor section in `PGEN_USER_GUIDE.md`.
- Full reporting procedure:
  - `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`

## Minimal Rust Example
```rust
use pgen::embedding_api::{
    parse_regex_default_result, parser_embedding_api_contract,
};

let contract = parser_embedding_api_contract();
assert!(contract.supports_regex_generated_backend);
assert_eq!(contract.regex_parser_release_version, "1.1.29");

parse_regex_default_result(r"https?://[^\s]+")?;
```

## Validation / Release Gates
- Public host API stability:
  - `make -C rust embedding_api_gate`
  - `make -C rust regex_parser_integration_contract_gate`
- Embedded code-block structural contract:
  - `make -C rust regex_embedded_code_block_contract_gate`
- External corpus hardening:
  - `make -C rust regex_pcre2_textsafe_corpus_gate`
  - `make -C rust regex_pcre2_compile_oracle_gate`
- Family proof/closure:
  - `make -C rust regex_parser_family_status_gate`
  - `make -C rust regex_parser_family_status_contract_gate`
  - `make -C rust regex_combined_telemetry_contract_gate`

## What This Does Not Promise
- It does not promise stable internal generated parser types.
- It does not promise runtime execution semantics for embedded code blocks such as `(?{...})`.
- It does not promise arbitrary valid Lua/JavaScript/Rhai payload acceptance beyond the explicitly published structural forms.
- It does not promise parser-owned validation of `native` / `wasm` reference payload formats.
- It does not promise every regex dialect already supported elsewhere in the ecosystem.
- It does not promise that downstream consumers can ignore parser release versioning when depending on AST details.
