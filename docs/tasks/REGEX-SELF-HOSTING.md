# REGEX-SELF-HOSTING: make `regex.ebnf` use only literal terminals — no `/.../`, no Rust regex-engine dependency

## Metadata

- Tree ID: `REGEX-SELF-HOSTING`
- Status: `active`
- Family / slice-id prefix: `PGEN-REGEX-SELF-HOST-<NNNN>`
- Roadmap lane: regex parser quality / architecture — a self-hosting regex parser that does not depend on
  Rust's `regex` engine to parse regexes
- Created: `2026-06-07`
- Director directive: 2026-06-07 — "PGEN regex parser EBNF shall not use the `/.../` construct … ideally we
  should build a from-scratch regex parser and not use Rust's own regex engine at all" + "I should only use
  literal string `"..."` in `regex.ebnf`." Sequenced FIRST; THEN resume regex cert-coverage-clean.

## Goal

`grammars/regex.ebnf` SHALL contain **no `/.../` regex-literals**. The generated `generated/regex_parser.rs`
SHALL therefore not compile or call any `regex::Regex` (no `match_regex`) — the regex parser parses regexes
with PGEN's own engine only (self-hosting; no regex-to-parse-regex circularity; acceptance no longer
coupled to Rust-regex semantics). Terminals are expressed with literal strings (`"..."`), char literals
(`'x'`), ordered-choice alternations, EBNF quantifiers, lookaheads, and (where needed) a native
**any-single-character** primitive. Behaviour-preserving: every conversion is verified byte-identical on the
`pcre2test` compile oracle (`regex_pcre2_compile_oracle_gate`) + RGX conformance + the regex AST-shape
contract; no version bump unless a surface changes.

## Non-Goals

- Re-implementing PCRE2's *matching* engine (this is about how the PARSER's grammar is expressed, not
  runtime match semantics).
- Removing `/.../` from OTHER grammars — the `/.../` EBNF feature stays available for grammars that want the
  convenience. Only `regex.ebnf` opts out (for self-hosting). Any new primitive added must be GENERAL /
  parser-agnostic.

## `.1` SCOPING result (2026-06-07, `PGEN-REGEX-SELF-HOST-0001`) — tool-backed

**The dependency is real and measured.** `generated/regex.json` contains exactly **36 `"regex"` nodes**
(one per `/.../` in `regex.ebnf`). Each compiles, in `ast_based_generator.rs`, to
`parser.match_regex(<pattern>, …)`, and `match_regex` builds `regex::Regex::new(r"\A(?:{pattern})")`
(`~:5270`) — i.e. the generated regex parser embeds and runs Rust's `regex` engine for those 36 token
classes.

**The codegen has exactly two terminal matchers** (verified — no others):
- `match_string(expected)` (`~:5180`) — native byte/`str` comparison. Used for `"string"` terminal nodes
  (the `"(*"`, `":"`, `'x'`, keyword terminals). **No Rust regex.**
- `match_regex(pattern, …)` (`~:5228`) — Rust `regex`. Used for `"regex"` nodes (`/.../`).

There is **NO** native character-class, character-range, or "any character" matcher in the codegen. Native
EBNF `[...]`/`[^...]`/`[a-z]` (which the meta-grammar `ebnf.ebnf:226-266` DOES define) are translated to
`"regex"` nodes upstream — so **switching `/.../` to native `[...]` would NOT remove the Rust-regex
dependency.** The only Rust-regex-free terminal today is the literal-string `match_string` path.

**Classification of the 36 (the conversion plan):**

| Class | Count (approx) | Examples | Conversion (Rust-regex-free) |
| --- | --- | --- | --- |
| Simple positive char-class | ~9 | `letter [A-Za-z]`, `digit [0-9]`, `nonzero_digit [1-9]`, `hex_digit`, `octal_digit`, `whitespace`, `short_prop_letter`, `backreference_digits_single` | ordered-choice of `'c'` char literals → `match_string` (native). Mechanical. |
| Quantified positive class | ~7 | `digits [0-9]+`, `hex_digits`, `octal_digits`, `prop_name [...]+`, `backreference_digits [1-9][0-9]+`, `hex_escape_short_payload [0-9A-Fa-f]{1,2}`, `octal_escape_short_payload [0-7]{1,3}` | the converted base rule + EBNF `+`/`*`/`{n,m}` (Layer-0 quantifier engine, already native). |
| Big positive literal-char set | ~4 | `literal_char`, `class_literal`, `any_char` (positive enumerated form), `special_char` | ordered-choice of `'c'` literals (large but mechanical) — OR the any-char primitive below for `any_char`. |
| **Negated / content-until-delimiter / true any-char** | **~16** | `unicode_char [^\x00-\x7F]`, `name ((?:[A-Za-z_]|[^\x00-\x7F])…)`, the 7 `callout_*_payload [^X]|XX`, `directive_payload_simple [^)]*`, `directive_payload_char [^)]`, `comment_text [^)]*` | **NEEDS a new native "any single character" primitive.** `[^X]` ⇒ `!"X" any_char`; `[^X]*` ⇒ `( !"X" any_char )*`; "any char" ⇒ `any_char`. Cannot be enumerated with literals. |

**KEY FINDING — one parser-agnostic engine primitive is required.** A native **any-single-character**
terminal (a `match_any_char`-style matcher: consume exactly one input char with no Rust regex) is the
enabling primitive. With it, every negated class becomes the lookahead idiom `!"X" any_char` (pure
`"..."`-literal + the primitive), and `any_char` is the primitive directly. This is a GENERAL,
parser-agnostic engine feature (any grammar can use it to avoid `/.../`), so it fits the standing doctrine
([[feedback_prefer_grammar_leave_engine_alone]] refinement — engine changes are on the table when they are
parser-agnostic features that let the EBNF express what a language needs; [[feedback_ast_pipeline_parser_agnostic]]).
Open sub-question for `.2`: surface syntax for the primitive (a reserved rule name `any_char`? a `.`
token? a `<any>` keyword?) — pick the cleanest parser-agnostic spelling; it must NOT collide with literal
content.

**Risk / care items:**
- **Behaviour preservation per conversion.** Each `/.../`→literal conversion must be byte-identical on the
  oracle (e.g. `whitespace [ \t\n\r\f\v]` must enumerate exactly those 6; `unicode_char [^\x00-\x7F]` must
  match exactly the non-ASCII set — the primitive must agree on char vs byte semantics + UTF-8 handling).
- **`any_char` is currently positive-enumerated** (`[A-Za-z0-9 \t…!@…]`) — verify whether it should become
  the true any-char primitive (likely yes) and whether that widens acceptance anywhere (oracle-check).
- **`name`** uses a Perl `(?:…)` non-capturing group + Unicode — convert to explicit rules + the any-char
  (non-ASCII) idiom.
- **Performance.** `match_regex` was anchored+cached (`PARSE-TERMINATION.7.1`); the literal/any-char path is
  also O(1)/O(len) — re-measure RGX speed (RGX-0078 watches regex perf) but expect parity or better.

## Task Tree

- ID: `REGEX-SELF-HOSTING`  Status: `active`
- ID: `.1`  Status: **`done`** (`PGEN-REGEX-SELF-HOST-0001`) — scoping: measured 36 `/.../`→`"regex"`→Rust
  `regex`; codegen has only `match_string`/`match_regex`; native `[...]` also compiles to regex; ~20 simple
  classes convert to literal alternations, ~16 negated/any-char classes NEED a native any-char primitive.
- ID: `.2`  Status: **`done`** (`PGEN-REGEX-SELF-HOST-0003`) — added the native `any_char` built-in to the
  codegen (`generate_unresolved_reference_method` arm): a `rule_reference` to `any_char` with no grammar
  definition emits `parse_any_char` = read one char (`self.input[pos..].chars().next()`), advance by
  `len_utf8()`, `Backtrack` at EOF, NO `regex::Regex`; no whitespace skip. ADDITIVE/byte-identical (only
  regex references `any_char`, and it DEFINES it → built-in dormant → regex parser unchanged, 214 rules).
  VERIFIED: codegen unit test `unresolved_reference_codegen_emits_native_any_char_matcher` (asserts native
  matcher, no regex); `cargo test --lib` 613/0; clippy strict source ✓; regex regen unchanged
  (parse_any_char still `match_regex`); END-TO-END on the real generate path — throwaway grammar
  `test_rule = "a" any_char "b"` (any_char undefined) generated a native `parse_any_char` (chars()/len_utf8,
  no regex). No book/contract change (dormant; no user-facing change until `.4`/`.5`).
  **SURFACE DECIDED (director 2026-06-07): a built-in reserved rule name `any_char`** — NO new
  EBNF punctuation/token; grammars just reference `any_char` and `[^X]` becomes `!"X" any_char`. Plan:
  codegen recognizes a `rule_reference` to `any_char` that has **no grammar definition** and emits a native
  `parse_any_char()` calling a new `match_any_char` helper (consume exactly one char, advance by its UTF-8
  byte length, Backtrack at EOF — NO `regex::Regex`). ADDITIVE + collision-free: the built-in activates ONLY
  when `any_char` is referenced-but-undefined, so existing grammars (regex.ebnf still DEFINES
  `any_char = /.../` until `.4`) are byte-identical. Prove on a throwaway grammar (`r = "a" any_char "b"`
  matches `aXb`; `r = !"x" any_char` rejects `x`, accepts others). Regen ALL parsers → byte-identical.
  Lockstep: document the `any_char` built-in in the EBNF reference + the platform book (new general EBNF
  feature). No regex.ebnf change in `.2`. ⚠️ char-vs-byte/UTF-8 semantics fixed here so `.4`'s
  `[^\x00-\x7F]`-style conversions are exact (range-negation like `unicode_char` may also need an ascii
  guard or a char-value constraint — a `.4` design item).
- ID: `.3`  Status: `DONE` for `$text` (part-1 `-0005` surface A; part-2 `-0006` surface B). **`-> $text`
  works END-TO-END on BOTH surfaces** (bootstrap + generated `return_annotation.ebnf`): a throwaway
  `num = digit+ -> $text` now generates `ParseContent::Terminal(&parser.input[start_pos..parser.position])`.
  part-2 added `matched_text_reference := '$' 'text' -> {type:"matched_text"}` to `return_annotation.ebnf`
  (wired into `primary_expression`), regen'd the generated return-annotation parser, mapped the
  `{type:"matched_text"}` node → `MatchedText` (`from_json`), and updated the shape-contract manifest
  (`return_annotation_v1.json`: new `matched_text_reference` entry + `primary_expression` branch 8→9).
  Verified: lib 614/0, generated_parsers 653/0, clippy strict ✓, **regex oracle byte-identical**
  (additive — no grammar uses `$text` yet). **`$0` alias DEFERRED** (`.3a` follow-up): it collides with the
  positional-index-0 machinery (`$00`, `$0::first`, accessor/extraction bases) where bootstrap + generated
  disagree; reverted the `$0`→MatchedText paths to keep them consistent (the regression stress corpus
  `$+0.A.A000[($0::first)[$00]]` stays positional). `$0` needs a not-followed-by-digit/modifier lookahead
  guard in `matched_text_reference` to ship safely. Docs lockstep: `RETURN_ANNOTATIONS_REFERENCE.md`,
  `PGEN_ANNOTATION_NORMATIVE_SPEC.md`, `docs/book/src/annotation-system.md`.
  (Original goal text:) **2nd enabling primitive — the `-> $text` return-annotation** (with
  `$0` as an alias, director 2026-06-07).
  **TWO SURFACES (tool-backed finding — both required, per [[feedback_semantic_annotation_no_dotted_refs]]):**
  return annotations are parsed by TWO parsers: (A) the hand-rolled `parse_bootstrap` (bootstrap-mode
  grammars + `embedding_api`), and (B) — for NON-bootstrap grammars like **regex** — the GENERATED
  `Return_annotationParser` (built from `grammars/return_annotation.ebnf`) via
  `mod.rs::parse_return_annotation_ast` → `parse_generated_return_annotation`. **Regex's `.4` uses surface B**,
  so B is the critical path. (Diagnosed end-to-end: a throwaway `num = digit+ -> $text` parsed to
  `Passthrough` — `result.clone()` in the generated `parse_num` — because B didn't recognize `$text`.)
  **part-1 DONE (`-0005`, surface A + the shared machinery):** added `UnifiedReturnAST::MatchedText` +
  `parse_bootstrap` `$text`/`$0` → MatchedText + codegen (`AstReturnTransformer` →
  `ParseContent::Terminal(&parser.input[start_pos..parser.position])`) + the 8 exhaustiveness sites + a unit
  test. ADDITIVE/dormant (no grammar uses `$text` yet → all parsers byte-identical; `cargo test --lib` 614/0).
  **part-2 PENDING (surface B + `$0`):** (1) `return_annotation.ebnf` — add `matched_text_reference :=
  "$text" -> {type:"matched_text"}` (or `"$0"`) wired into `primary_expression` (+ flat_spread/accessor
  bases); (2) regen `generated/return_annotation_parser.rs` (bootstrap regen); (3) conversion —
  `parse_generated_return_annotation_node`/`parse_generated_value_node` map the matched_text node → MatchedText,
  AND the `from_json` "positional" arm maps `index == 0` → MatchedText (so `$0` works, no grammar change for
  the alias); (4) re-test the throwaway `num = digit+ -> $text` → generated `parse_num` emits
  `ParseContent::Terminal(&parser.input[start_pos..parser.position])`; regen-all byte-identical. **WHY (tool-backed finding):**
  the ~8 quantified payloads (`octal_digits`/`hex_digits`/`digits`/`prop_name`/`backreference_digits`/
  `hex_escape_short_payload`/`octal_escape_short_payload`/`name`) use `/.../`-with-capture SPECIFICALLY to
  emit the whole match as ONE flat string (`octal_digits` → `"777"`; the grammar's own comment at
  `regex.ebnf:1186-1188` documents the deliberate switch from a char-rule chain that "emit[ted]
  `[first_digit, [rest_digits]]`" to the regex literal "to emit a clean string Terminal"). So `char+` alone
  would re-introduce the structure and break the AST contract (`digits: $N` must stay `"777"`, NOT a
  Quantified node). `-> $text` makes a rule return its full matched span text as a string Terminal (native
  equivalent of a `/.../` capture; uses the span the parser already tracks). Implement: add a `MatchedText`
  variant to the return-annotation AST (`unified_return_ast.rs`) + parse `$text`/`$0` (bootstrap parser +
  `return_annotation.ebnf`) + codegen (emit `ParseContent::Terminal(&self.input[node.span])`). Prove on a
  throwaway grammar (`r = digit+ -> $text` over `"123"` → `"123"`). ADDITIVE (no existing annotation uses
  `$text`/`$0`) → all parsers byte-identical. Lockstep: `docs/RETURN_ANNOTATIONS_REFERENCE.md` +
  `PGEN_ANNOTATION_NORMATIVE_SPEC.md` + the return-annotation contract + annotation book chapter.
- ID: `.4`  Status: `pending`  Goal: convert the POSITIVE char-classes in `regex.ebnf` to literals — single
  ones (`digit`/`letter`/`hex_digit`/`octal_digit`/`whitespace`/`short_prop_letter`/… → `'c'` ordered-choice;
  shape-safe one-char Terminal) and quantified ones (`digits = digit+ -> $text`, etc.; uses `.3`'s `$text`).
  Batches, each regen + oracle byte-identical + lib + shape-contract + RGX conformance. Removes ~20 of 36.
- ID: `.5`  Status: `pending`  Goal: convert the NEGATED / content-until-delimiter / any-char classes using
  `!"X" any_char` (`.2`'s primitive) + `$text` for multi-char runs (`unicode_char`, `name`, callout
  payloads, `directive_payload_*`, `comment_text`, `any_char`). ⚠️ range-negations like
  `unicode_char [^\x00-\x7F]` may need an ascii guard. Removes the remaining ~16.
- ID: `.6`  Status: `pending`  Goal: CAPSTONE — assert `regex.ebnf` has zero `/.../` and
  `generated/regex_parser.rs` contains no `match_regex`/`regex::Regex` call; add a gate/test that fails if a
  `/.../` reappears in `regex.ebnf`; lockstep (regex book "self-hosting" note + contract if a surface
  changed). THEN the tree is done and regex cert-coverage-clean resumes (director sequencing).
- ID: `.3a`  Status: `DONE` (`PGEN-REGEX-SELF-HOST-0007`)  Goal: enable the **`$0` whole-match alias** of
  `$text` (director: "in Perl5 `$0` = whole match, `$1..$N` = captures — a clean extension"). RESOLVED the
  earlier collision the RIGHT way (not a guard): a tools-first trace showed `$00`/`$+0`/`$0::first` exist
  ONLY in synthetic stress probes, and positional index 0 was ALREADY a hard error (`E_RET_POS_ZERO`) — so
  nothing real is lost by making **any index-0 spelling (`$0`/`$00`/`$+0`) → `MatchedText`** consistently in
  BOTH surfaces (bootstrap `parse_positional_ref` + `from_json`). `$0` is valid only as a BASE value;
  composing extraction/accessor on it (`$0::first`/`$0.x`/`$0[i]`) is a new semantic error
  `E_RET_WHOLE_MATCH_NOT_COMPOSABLE` (a flat whole-match has no children). Retired `E_RET_POS_ZERO`
  (index 0 is now meaningful). Re-pointed the synthetic stress tests + chain-walker tests to the new
  semantics ($0→MatchedText). Verified: lib 614/0, generated_parsers 653/0, clippy ✓, regex oracle
  byte-identical, END-TO-END `$0`/`$00`/`$+0` → whole-match span Terminal. Docs lockstep
  (RETURN_ANNOTATIONS_REFERENCE / NORMATIVE_SPEC / book annotation table). NO grammar change/regen needed
  ($0 goes through the existing `positional_reference` → host maps index 0 → MatchedText).

## Decisions

- `2026-06-07`: director directive — regex.ebnf `"..."`-only / no `/.../`, self-hosting regex parser; do
  this BEFORE resuming regex cert-coverage-clean. Decision record [[project_regex_self_hosting_no_slash_literals]].
- `2026-06-07` (`.1`): full self-hosting requires ONE new parser-agnostic engine primitive (native
  any-single-character matcher); native `[...]` char-classes do NOT help (they compile to Rust regex). The
  rest is mechanical literal-alternation conversion. Engine change is justified (parser-agnostic, general).
- `2026-06-07` (`.3` finding + decision, director): self-hosting needs a SECOND parser-agnostic primitive —
  a **`-> $text` return-annotation** (with **`$0` as an alias**) that emits a rule's full matched span text as
  one string Terminal. TOOL-BACKED: the quantified payloads use `/.../`-capture to return a flat string
  (`octal_digits` → `"777"`), confirmed by the grammar's own comment (`regex.ebnf:1186-1188`: deliberate
  switch from a char-rule chain emitting `[first,[rest]]` to a regex literal "to emit a clean string
  Terminal"); a bare `char+` would re-break the AST contract. `$text` is the native equivalent of a `/.../`
  capture (uses the parser's span). Additive (no existing annotation uses `$text`/`$0`). Spelling = `$text`
  primary + `$0` alias (director).
- `2026-06-07` (`.2` design, director): the any-char primitive is spelled as a **built-in reserved rule name
  `any_char`** (no new EBNF token/punctuation; closest to the `"..."`-only intent — grammars reference
  `any_char`, and negated classes are `!"X" any_char`). The codegen emits the native matcher for a
  `rule_reference` to `any_char` that has no grammar definition (additive; activates when a grammar drops
  its own `any_char` definition).

## Open Questions

- ~~`.2`: surface spelling of the any-char primitive~~ **RESOLVED (director 2026-06-07): built-in reserved
  rule name `any_char`** (no new EBNF punctuation; `[^X]` = `!"X" any_char`). Char-vs-byte/UTF-8 semantics
  still to be fixed in `.2` implementation so `[^\x00-\x7F]`-style conversions are exact.
- Whether to keep `any_char`'s historical positive enumeration anywhere for bounded behaviour, or make it
  the unbounded primitive everywhere (oracle decides per site).

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-07` | `.1` | grep `regex.json` node types (36 `"regex"`); codegen matcher audit (`match_string`/`match_regex` only; no native char-class/any-char); meta-grammar `[...]`→regex; per-`/.../ ` classification | SCOPING DONE — 1 engine primitive needed + mechanical conversions |
| `2026-06-07` | `.2` (`-0003`) | codegen unit test (native matcher: chars/len_utf8, no regex); `cargo test --lib` 613/0; clippy strict source ✓; additivity (only regex refs `any_char` + DEFINES it → built-in dormant → regex regen 214 rules unchanged, parse_any_char still `match_regex`); END-TO-END throwaway grammar `"a" any_char "b"` → native `parse_any_char` on the real generate path | **DONE** — additive, no user-facing change (no book/contract) |
| `2026-06-07` | `.3` part-1 (`-0005`) | `MatchedText` variant + `parse_bootstrap` `$text`/`$0` + codegen span-Terminal + 8 exhaustiveness sites + unit test `matched_text_dollar_text_and_dollar_zero_emit_span_terminal`; `cargo test --lib` 614/0; additive/dormant. END-TO-END diagnosis: throwaway `num = digit+ -> $text` still emits passthrough via the GENERATED return_annotation parser (surface B) → part-2 needed | **PART-1 DONE** (surface A + machinery); part-2 (surface B) pending |
| `2026-06-07` | `.3` part-2 (`-0006`) | `return_annotation.ebnf` `matched_text_reference` + regen + `from_json` `{type:"matched_text"}`→MatchedText; manifest `return_annotation_v1.json` (matched_text entry + primary_expression branch 8→9); reverted `$0`→MatchedText (collision); lib 614/0, generated_parsers 653/0, clippy ✓, regex oracle byte-identical; END-TO-END throwaway `num = digit+ -> $text` → `Terminal(&parser.input[start_pos..parser.position])`; docs lockstep (3 surfaces) | **`.3` DONE for `$text`**; `$0` deferred (`.3a`) |
| `2026-06-07` | `.3a` (`-0007`) | `$0` (any index-0 spelling) → MatchedText in BOTH surfaces (bootstrap `parse_positional_ref` + `from_json`); new `E_RET_WHOLE_MATCH_NOT_COMPOSABLE` for `$0::first`/`$0.x`/`$0[i]`; retired `E_RET_POS_ZERO`; re-pointed stress + chain-walker tests; lib 614/0, generated_parsers 653/0, clippy ✓, regex byte-identical; END-TO-END `$0`/`$00`/`$+0` → span Terminal; docs lockstep | **`.3a` DONE — `$0` enabled (Perl5)** |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.1` | `PGEN-REGEX-SELF-HOST-0001` | scoping (pure docs); tool-backed feasibility + plan |
| `.2` decision | `PGEN-REGEX-SELF-HOST-0002` | record director's any-char spelling = built-in `any_char` rule (docs) |
| `.2` impl | `PGEN-REGEX-SELF-HOST-0003` | native `any_char` built-in in codegen + unit test; additive/dormant (byte-identical); end-to-end proven |
| `.3` part-1 | `PGEN-REGEX-SELF-HOST-0005` | `$text`/`$0` surface A (bootstrap) + `MatchedText` AST + codegen + 8 exhaustiveness + test; additive/dormant; surface B (generated parser) diagnosed as the remaining critical path |
| `.3` part-2 | `PGEN-REGEX-SELF-HOST-0006` | `$text` surface B (`return_annotation.ebnf` + regen + `from_json` mapping + manifest); end-to-end works; regex byte-identical; `$0` deferred (collision). `.3` DONE for `$text` |
| `.3a` | `PGEN-REGEX-SELF-HOST-0007` | `$0` whole-match alias enabled (Perl5): index-0→MatchedText both surfaces + E_RET_WHOLE_MATCH_NOT_COMPOSABLE + retired E_RET_POS_ZERO; tests re-pointed; byte-identical. `$0` DONE |

## Changelog

- `2026-06-07`: tree created + `.1` scoping done (`PGEN-REGEX-SELF-HOST-0001`) per the director directive to
  make `regex.ebnf` `"..."`-only / Rust-regex-free.
- `2026-06-07`: `.3a` (`PGEN-REGEX-SELF-HOST-0007`) — enabled the `$0` whole-match alias (Perl5: `$0`=whole
  match, `$1..$N`=captures). A tools-first trace showed `$00`/`$+0`/`$0::first` exist only in synthetic
  stress probes and positional index 0 was already a hard error (`E_RET_POS_ZERO`), so any index-0 spelling
  now lowers to `MatchedText` consistently in both surfaces (bootstrap `parse_positional_ref` + `from_json`).
  Added `E_RET_WHOLE_MATCH_NOT_COMPOSABLE` (composing `::`/`.`/`[]` on the whole match is invalid — no "0th
  child"); retired `E_RET_POS_ZERO`; re-pointed the stress + chain-walker tests. lib 614/0, generated_parsers
  653/0, clippy ✓, regex byte-identical, END-TO-END `$0`/`$00`/`$+0` → span Terminal. No grammar change/regen
  (the existing `positional_reference` parses `$0`; the host maps index 0 → MatchedText). Docs lockstep.
- `2026-06-07`: `.3` part-2 (`PGEN-REGEX-SELF-HOST-0006`) — completed `$text` on the GENERATED
  return-annotation surface: added `matched_text_reference` to `return_annotation.ebnf`, regen'd the
  generated parser, mapped `{type:"matched_text"}` → `MatchedText`, and re-pinned the shape-contract
  manifest (matched_text entry + `primary_expression` branch 8→9). `-> $text` now works END-TO-END on both
  surfaces (throwaway `num = digit+ -> $text` → `Terminal(&parser.input[start_pos..parser.position])`). lib
  614/0, generated_parsers 653/0, clippy ✓, regex oracle byte-identical (additive). `$0` alias DEFERRED to
  `.3a` (collides with positional-index-0; the `$0`→MatchedText paths were reverted to keep bootstrap +
  generated consistent). Docs lockstep (RETURN_ANNOTATIONS_REFERENCE, NORMATIVE_SPEC, platform book
  annotation chapter). `.3` is DONE for `$text`; frontier → `.4` (convert the regex char-classes).
- `2026-06-07`: `.3` part-1 (`PGEN-REGEX-SELF-HOST-0005`) — implemented the `$text`/`$0` primitive on
  surface A (the hand-rolled `parse_bootstrap` + the shared `MatchedText` AST variant + codegen + 8
  exhaustiveness sites + a unit test); additive/dormant (614/0). END-TO-END diagnosis revealed that
  NON-bootstrap grammars (regex) parse return annotations via the GENERATED `Return_annotationParser`
  (surface B), so a throwaway `num = digit+ -> $text` still produced a passthrough; part-2 (extend
  `return_annotation.ebnf` + regen + the generated→AST conversion + `$0`-index-0 mapping) is the remaining
  critical path before `.4` can use `-> $text` on regex.
- `2026-06-07`: `.2` design decision (`PGEN-REGEX-SELF-HOST-0002`) — any-char primitive = built-in reserved
  rule name `any_char` (director pick); then `.2` IMPLEMENTATION DONE (`PGEN-REGEX-SELF-HOST-0003`): native
  `any_char` built-in added to the codegen (emitted for a `rule_reference` to `any_char` with no grammar
  definition); additive/dormant (regex still defines `any_char` → byte-identical, 214 rules); verified by
  unit test + lib 613/0 + clippy strict + end-to-end throwaway-grammar generation. No user-facing change
  (no book/contract). Frontier → `.3` (convert the ~20 simple positive char-classes in `regex.ebnf` to
  literal `'c'` alternations + EBNF quantifiers; each regen + oracle byte-identical). After the tree
  closes, resume regex cert-coverage-clean (`REGEX-PCRE2-FIDELITY.3.7` empty-`[]` → regex default 0, then
  GRAMMAR-WELLFORMED Phase H).
