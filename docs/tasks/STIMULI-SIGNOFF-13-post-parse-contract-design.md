# STIMULI-SIGNOFF.13 — Post-parse-contract-aware generation: root cause + design (2026-07-07)

Owning leaf: `STIMULI-SIGNOFF.13` (design slice `.13.1`, `PGEN-STIMULI-SIGNOFF-0015`, docs-only).
Finding source: the `.4.4` duality-break hunter's first real-lane run (`PGEN-STIMULI-SIGNOFF-0014`).
Sibling tree: `REGEX-PCRE2-FIDELITY` (owns the regex-grammar encodings; see §5 ownership split).

## 1. Reproduction (tool-backed)

`ast_pipeline grammars/regex.ebnf --directed-generation-goal duality_break --directed-rounds 20
--directed-samples-per-round 5 --seed {0,7,42}` (2026-07-07, session #60, HEAD `ae8d4c94`):

- seed 0: rejected 6/100, 4 unique breaks — `[\E]` · `$*` · `(*scs:(<_>))` · `(*:)`
- seed 7: rejected 7/100, 5 unique breaks — `(*scs:('_'))` · `(*:)` · `[\E]` · `(*PRUNE=)` · `$*`
- seed 42: rejected 11/100, 4 unique breaks — `(*:)` · `$*` · `(*scan_substring:(<_>))` · `[\E]`

The `.4.4` session-#59 run additionally observed the `(?C262)` callout class and `[\Q]`/`[\E\E]`
spellings — same class set. Reports: deterministic per seed (byte-identical on re-run).

## 2. Root cause (WHY + WHERE, per class — the enforced diagnosis)

**Mechanism (all classes share it):** each reproducer fed to the REAL parser
(`parseability_probe --parse regex`) fails with the **post-parse-contract message, not a grammar
backtrack** — i.e. `parse_full_regex` ACCEPTS the form structurally and
`validate_regex_compile_contract` (`rust/src/regex_compile_validation.rs`) rejects it afterwards,
invoked unconditionally (BOTH profiles) from `parser_registry.rs:394/:407/:1226`. The stimuli
generator consults only the grammar + annotations (grep-verified: zero references to
`regex_compile_validation`/`post_parse_semantic_contract` in `stimuli_generator.rs`), so every
contract-only constraint is **invisible to generation** — the exact
[[project_ebnf_is_single_source_of_truth]] defect class, and the same generation-blindness family
as the lookahead gap (STIMULI-SIGNOFF Decisions 2026-06-10).

| # | Shrunk reproducer | WHY the generator renders it (grammar site) | WHERE the contract rejects (validator site) | PCRE2 10.47 oracle |
|---|---|---|---|---|
| 1 | `(*:)` | `directive_mark_shorthand = ":" directive_payload_simple?` — payload OPTIONAL (`regex.ebnf:1103`) | "MARK shorthand verb requires a non-empty argument" (`regex_compile_validation.rs:439`) | err 166 "(*MARK) must have an argument" |
| 2 | `(*SKIP=)` / `(*PRUNE=)` | `directive_payload_suffix = "=" directive_payload_simple?` attaches `=` to ANY `directive_name` (`regex.ebnf:1107-1109`) | "PCRE2 verb is malformed" — verbs accept only `:`/`)` (`:491`, `pcre2_verb_argument_rule` `:550`) | err 160 "(*VERB) not recognized or malformed" |
| 3 | `(?C262)` | `callout_arg = digits` — unbounded (`regex.ebnf:946`) | "numeric callout argument exceeds PCRE2 compile limit 255" (`:385-390`) | err 138 "number after (?C is greater than 255" (`(?C255)` accepted) |
| 4 | `$*` / `^*` | `piece = atom quantifier?` (`regex.ebnf:54`) with `atom = anchor` (`:149`, `:196-197`) | "quantifier cannot be applied directly to an anchor" (`find_invalid_quantified_anchor`, `:1388`) | err 109 "quantifier does not follow a repeatable item" |
| 5 | `[\E]` / `[\Q]` / `[\E\E]` | `stray_class_end_quote = "\E"` (`:417`), empty `quoted_class_literal` (`:414`), `empty_quoted_class_literal` (`:433`) count as class MEMBERS | class analyzer: \Q\E-invisible members leave the class PCRE2-empty → first-`]`-is-literal → "unterminated character class" (`:794/:900/:911`) | err 106 "missing terminating ] for character class" (`[\E]x]` ACCEPTED — `]` becomes the literal first member) |
| 6 | `(*scs:('_'))` | `returned_capture_group = signed_digits \| name_ref` — name unconstrained (`regex.ebnf:917`) | "scan_substring capture list references an unknown named capture" — checked against the FULL-pattern inventory (`validate_scan_substring_capture_refs`, `:1319`) | err 115 "reference to non-existent subpattern"; **forward refs LEGAL**: `(*scs:('a'))(?<a>x)` accepted |

## 3. 🔎 NEW FINDING while root-causing — a real PARSER-side fidelity divergence

Differential probe (PGEN release probe vs `pcre2test` 10.47, 2026-07-07): PGEN **ACCEPTS** all
seven quantified escape-anchors that PCRE2 **REJECTS** (err 109):
`\A*` `\b*` `\B?` `\G+` `\z*` `\Z*` `\K*`.
`find_invalid_quantified_anchor` covers only `^`/`$` — an incomplete anchor set, so this is an
accepts-invalid divergence in the RELEASED regex parser, latent because the oracle-gate corpus
contains no quantified escape-anchors. Parity confirmed on the POSIX aliases: `[[:<:]]*` /
`[[:>:]]+` are ACCEPTED by BOTH (PCRE2 compiles them to quantifiable sub-groups) — so the
non-quantifiable set is exactly `{^, $, \A, \b, \B, \G, \z, \Z, \K}`.
Routed to **`REGEX-PCRE2-FIDELITY.3.13`** (which also owns the row-8 structural encoding — one
slice fixes the duality class AND the divergence).

Latent same-mechanism classes (oracle-verified, not yet emitted at the 100-sample budget; the
scaled hunter run in `.13.3` will smoke them out): `a{5,2}` (row 4, err 104), `a(*UTF)`
(start-option position, err 160), `(*:x)+` (only `(*ACCEPT)` is quantifiable — `(*ACCEPT)+`
accepted by both), invalid `\p{…}` names (row 2), group names > 128 chars (row 3),
`\K` in lookaround (row 10). **The true class universe = every remaining
`validate_regex_compile_contract` check** — the honest end-state is the REGEX-PCRE2-FIDELITY
capstone `.4` (EBNF encodes all 10 rows, validator deleted, duality-clean BY CONSTRUCTION).

## 4. Fix design (per the no-workarounds hierarchy — declarative first)

Principles: (a) encodings are PLAIN (no profile split) — the contract rejects in BOTH profiles
today, so verdict parity everywhere; (b) each encoding slice removes its matching validator branch
same-slice (the `.3.2` precedent) with `pcre2test` + `regex_pcre2_compile_oracle_gate` parity
proof; (c) engine work only where declarative provably cannot express the constraint, and then as
GENERAL parser-agnostic primitives.

| Class | Fix tier | Encoding |
|---|---|---|
| `(*:)` (1) + `(*SKIP=)` (2) | tier 2 (grammar restructure) | name-class-conditional directive shapes: MARK-shorthand payload REQUIRED non-empty; verbs take `:`-suffix only; `=`-suffix only for the numeric-value start options (digits payload). Owned by `REGEX-PCRE2-FIDELITY.3.14`. |
| `$*` + the §3 divergence (4) | tier 2 (grammar restructure) | split `piece` so the 9-anchor set is non-quantifiable (POSIX aliases stay quantifiable); AST shape preserved. Owned by `REGEX-PCRE2-FIDELITY.3.13`. |
| `[\E]`-family (5) | tier 2 (grammar restructure) | class-member visibility: `\E`-stray/empty-`\Q\E` are INVISIBLE members; the non-empty-class requirement (from `.3.7(a)`) must count only VISIBLE members, with invisible prefixes allowed before the first-`]`-literal form (`[\E]x]` stays accepted). Owned by `REGEX-PCRE2-FIDELITY.3.15`. |
| `(?C262)` (3) | tier 1 (existing annotation) | `@range: [0, 255]` on a dedicated `callout_number` rule — codegen already emits a parse-time numeric guard (`ast_based_generator.rs:7868`) and the generator already samples within bounds (`stimuli_generator.rs:13580`). **Prerequisite:** the interpreter has NO value-constraint mirror (grep: zero `@range`/`numeric_bounds` support in `parse_harness_interpreter.rs`) — adding `@range` to a differential-CERTIFIED grammar without it creates a latent divergence class. `STIMULI-SIGNOFF.13.2` lands the parser-agnostic interpreter mirror (+ combinator-suite cases) FIRST; then `REGEX-PCRE2-FIDELITY.3.16` lands the grammar edit. |
| `(*scs:('_'))` (6) | tier 3/4 (generation-side declarative; parse-time encoding UNSOUND) | forward references are LEGAL (oracle §2 row 6), so a parse-time `has_fact` predicate would newly reject valid patterns — the parse-side check STAYS in the validator (until the capstone finds a two-pass shape). The fix is GENERATION-side: extend store-aware generation so `name_ref` inside `returned_capture_group_list` draws from generation-emitted capture-name facts (the sound already-generated subset, the `.3.12` precedent), via a grammar-declared generation-side gate — design owned by `STIMULI-SIGNOFF.13.4`, regex application by `REGEX-PCRE2-FIDELITY.3.17`. |

## 5. Ownership split (the `.3.12` ↔ STORE-AWARE-GEN mirror)

- **`REGEX-PCRE2-FIDELITY.3.x`** owns every `grammars/regex.ebnf` encoding slice (released-parser
  lockstep: regen + manifest + oracle/conformance gates + book/contract/ledger + version policy).
  New/updated leaves: `.3.13` (anchors + divergence), `.3.14` (verb shapes), `.3.15` (class
  visibility), `.3.16` (callout `@range`), `.3.17` (scs generation-side), `.3.18` (counted-
  quantifier bounds, latent row 4).
- **`STIMULI-SIGNOFF.13`** owns the GENERAL parser-agnostic capabilities + the honesty ratchet:
  `.13.1` this design; `.13.2` interpreter value-constraint mirror; `.13.3` duality-hunt
  regression lane + the cert-spf adjudication; `.13.4` generation-side declarative gate for
  cross-referential constraints.

## 6. Cert-spf config-scope adjudication (the leaf's second mandate) — DECIDED

The regex cert-coverage `sample_parse_failures=0` claim is CONFIG-SCOPED (the steered cert config
avoids the rare forms; the hunter's plain config exposes them — `.4.4` characterization).
Decision (routine, within-principle): **keep the cert config as-is** (its deterministic baselines
and byte-identity discipline are load-bearing across many gates); **add a duality-hunt lane as the
honest plain-config coverage** — `.13.3` wires the hunter (deterministic per seed) into a
repo-standard gate for regex + svpp with expected `unique_breaks=0` once the §4 encodings land,
and the book's honest-bounds paragraph keeps stating the cert claim's config scope (already
landed with `.4.4`). The before→after oracle for the whole program = the §1 hunter command at
seeds 0/7/42 → `unique_breaks → 0` (or an explicitly adjudicated residual).

## 7. Slice plan (batch order)

1. `.13.1` — this record (docs; `PGEN-STIMULI-SIGNOFF-0015`). ✅
2. `REGEX-PCRE2-FIDELITY.3.13` — quantified-anchor encoding (kills the observed class AND the §3
   divergence; grammar-only).
3. `REGEX-PCRE2-FIDELITY.3.14` — directive/verb arg-shape encoding (kills `(*:)` + `(*SKIP=)`).
4. `REGEX-PCRE2-FIDELITY.3.15` — class-member visibility (kills `[\E]`-family).
5. `STIMULI-SIGNOFF.13.2` — interpreter value-constraint mirror (parser-agnostic prerequisite).
6. `REGEX-PCRE2-FIDELITY.3.16` — callout `@range` (kills `(?C262)`).
7. `STIMULI-SIGNOFF.13.4` + `REGEX-PCRE2-FIDELITY.3.17` — scs generation-side gate.
8. `STIMULI-SIGNOFF.13.3` — duality-hunt gate lane + scaled hunter run (enumerate residual/latent
   classes honestly; adjudicate or iterate).
