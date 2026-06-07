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
- ID: `.3`  Status: `pending`  Goal: convert the SIMPLE positive char-classes (single + quantified) in
  `regex.ebnf` to literal alternations — one batch (or a few), each regen + oracle byte-identical + lib +
  RGX conformance + shape-contract. Removes ~20 of 36 `"regex"` nodes.
- ID: `.4`  Status: `pending`  Goal: convert the NEGATED / content-until-delimiter / any-char classes using
  `!"X" any_char` + the primitive (`unicode_char`, `name`, callout payloads, `directive_payload_*`,
  `comment_text`, `any_char`). Removes the remaining ~16.
- ID: `.5`  Status: `pending`  Goal: CAPSTONE — assert `regex.ebnf` has zero `/.../` and
  `generated/regex_parser.rs` contains no `match_regex`/`regex::Regex` call; add a gate/test that fails if a
  `/.../` reappears in `regex.ebnf`; lockstep (regex book "self-hosting" note + contract if a surface
  changed). THEN the tree is done and regex cert-coverage-clean resumes (director sequencing).

## Decisions

- `2026-06-07`: director directive — regex.ebnf `"..."`-only / no `/.../`, self-hosting regex parser; do
  this BEFORE resuming regex cert-coverage-clean. Decision record [[project_regex_self_hosting_no_slash_literals]].
- `2026-06-07` (`.1`): full self-hosting requires ONE new parser-agnostic engine primitive (native
  any-single-character matcher); native `[...]` char-classes do NOT help (they compile to Rust regex). The
  rest is mechanical literal-alternation conversion. Engine change is justified (parser-agnostic, general).
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

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `.1` | `PGEN-REGEX-SELF-HOST-0001` | scoping (pure docs); tool-backed feasibility + plan |
| `.2` decision | `PGEN-REGEX-SELF-HOST-0002` | record director's any-char spelling = built-in `any_char` rule (docs) |
| `.2` impl | `PGEN-REGEX-SELF-HOST-0003` | native `any_char` built-in in codegen + unit test; additive/dormant (byte-identical); end-to-end proven |

## Changelog

- `2026-06-07`: tree created + `.1` scoping done (`PGEN-REGEX-SELF-HOST-0001`) per the director directive to
  make `regex.ebnf` `"..."`-only / Rust-regex-free.
- `2026-06-07`: `.2` design decision (`PGEN-REGEX-SELF-HOST-0002`) — any-char primitive = built-in reserved
  rule name `any_char` (director pick); then `.2` IMPLEMENTATION DONE (`PGEN-REGEX-SELF-HOST-0003`): native
  `any_char` built-in added to the codegen (emitted for a `rule_reference` to `any_char` with no grammar
  definition); additive/dormant (regex still defines `any_char` → byte-identical, 214 rules); verified by
  unit test + lib 613/0 + clippy strict + end-to-end throwaway-grammar generation. No user-facing change
  (no book/contract). Frontier → `.3` (convert the ~20 simple positive char-classes in `regex.ebnf` to
  literal `'c'` alternations + EBNF quantifiers; each regen + oracle byte-identical). After the tree
  closes, resume regex cert-coverage-clean (`REGEX-PCRE2-FIDELITY.3.7` empty-`[]` → regex default 0, then
  GRAMMAR-WELLFORMED Phase H).
