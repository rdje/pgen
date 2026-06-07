---
name: project-regex-self-hosting-no-slash-literals
description: Director directive 2026-06-07 — grammars/regex.ebnf SHALL use only literal terminals ("..."/'x'/alternations), NO `/.../` regex-literals, so the generated regex parser does NOT depend on Rust's `regex` engine to parse regexes (self-hosting; no regex-to-parse-regex circularity; acceptance decoupled from Rust-regex semantics). Tool-backed finding: regex.ebnf has 36 `/.../` → 36 `"regex"` nodes → Rust `regex`; the codegen has only `match_string` (native) + `match_regex` (Rust); native `[...]` char-classes ALSO compile to regex; ~20 classes convert to literal alternations but ~16 negated/any-char classes need ONE new parser-agnostic engine primitive (a native any-single-character matcher; then `[^X]` = `!"X" any_char`). Do this BEFORE resuming regex cert-coverage-clean. Owned by REGEX-SELF-HOSTING.
metadata:
  node_type: memory
  type: project
  director_directive: true
  created: 2026-06-07
  owning_tree: REGEX-SELF-HOSTING
---

**THE DIRECTIVE (director, 2026-06-07).** "PGEN regex parser EBNF shall not use the `/.../` construct …
ideally we should build a from-scratch regex parser and not use Rust's own regex engine at all." +
"I should only use literal string `"..."` in `regex.ebnf`." + sequencing: "When this `/.../` dependency is
removed from regex.ebnf then we can continue with PGEN regex parser cert-coverage clean."

**WHY IT'S A GENUINE CONCERN.** PGEN's regex parser parsing regexes *with another regex engine* is a
self-referential dependency that is (1) inelegant for a signoff-grade regex tool, and (2) couples PGEN's
accepted regex language to Rust-regex semantics (anchoring, Unicode classes, char vs byte) rather than to
PGEN's own engine. A self-hosting regex parser removes both.

**TOOL-BACKED STATE (REGEX-SELF-HOSTING.1, 2026-06-07).**
- `generated/regex.json` has exactly **36 `"regex"` nodes** (one per `/.../` in `regex.ebnf`); each →
  `match_regex` → `regex::Regex::new(r"\A(?:{pattern})")` in the generated parser. So the generated regex
  parser embeds and runs Rust's `regex` engine.
- The codegen (`ast_based_generator.rs`) has exactly TWO terminal matchers: `match_string` (native byte
  comparison, used for `"string"`/`'char'` literal nodes — Rust-regex-free) and `match_regex` (Rust regex,
  used for `"regex"` nodes). There is NO native character-class / character-range / any-character matcher.
- Native EBNF `[...]`/`[^...]`/`[a-z]` (defined in the meta-grammar `ebnf.ebnf`) are translated to `"regex"`
  nodes upstream ⇒ **switching `/.../` to native `[...]` does NOT remove the dependency.** Only the
  literal-string `match_string` path is Rust-regex-free.

**THE PLAN.**
1. ~20 simple positive char-classes (`digit`/`letter`/`hex_digit`/… and their `+`/`*` forms) → ordered-choice
   of `'c'` char literals + EBNF quantifiers (native `match_string` + Layer-0 quantifier engine). Mechanical.
2. ~16 negated / content-until-delimiter / true-any-char classes (`unicode_char [^\x00-\x7F]`, `name`, the 7
   `callout_*_payload [^X]|XX`, `directive_payload_*`, `comment_text`, `any_char`) → **need ONE new
   parser-agnostic engine primitive: a native any-single-character matcher.** Then `[^X]` = `!"X" any_char`,
   `[^X]*` = `( !"X" any_char )*`, "any char" = `any_char`. The primitive is GENERAL (benefits any grammar
   wanting `/.../`-free), so the engine change is justified per [[feedback_prefer_grammar_leave_engine_alone]]
   (parser-agnostic-feature carve-out) + [[feedback_ast_pipeline_parser_agnostic]].
3. Capstone: assert zero `/.../` in `regex.ebnf` and zero `match_regex`/`regex::Regex` in
   `generated/regex_parser.rs`; add a gate so `/.../` cannot reappear in `regex.ebnf`.

**VERIFICATION.** Every conversion byte-identical on the `pcre2test` compile oracle
(`regex_pcre2_compile_oracle_gate`) + RGX conformance + the regex AST-shape contract; no version bump unless
a surface changes (the `.3.1`/`.3.2` precedent: grammar-internal restructures can be surface-neutral).

**SEQUENCING.** This tree runs BEFORE resuming the regex cert-coverage-clean push
([[project_regex_pcre2_faithful_by_default_relaxed_optout]]'s `.3.7` empty-`[]` → regex default cert-cov 0,
then GRAMMAR-WELLFORMED Phase H for the other grammars). Composes with — does not replace — the PCRE2
fidelity campaign (the `.3.1`/`.3.2` strict paths are already `/.../`-free).
