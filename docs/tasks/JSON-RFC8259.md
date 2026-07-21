# JSON-RFC8259: the JSON parser equals the FULL official standard (RFC 8259 / ECMA-404), zero restriction

## Metadata

- Tree ID: `JSON-RFC8259`
- Status: **`parked`** (created 2026-07-22, session #189 — director-directed
  commitment, then explicitly parked the same day: "Do not switch to the JSON
  full support yet. Log the decision in a task-tree then move on." ⛔ NO work —
  not even the read-only `.1` audit — until an explicit director GO.)
- Roadmap lane: director commitment 2026-07-22
  ([[project_json_rfc8259_full_standard_commitment]], verbatim in the decision
  record): the current `grammars/json.ebnf` is a self-described "Simplified JSON
  Grammar" — honest cert, toy language. The committed target is the OFFICIAL
  standard with zero restriction, both fidelity directions. Also the first
  worked exemplar of the horizon goal ([[project_horizon_universal_parser]]).
- Created: `2026-07-22`
- Owner: repo-local workflow

## Goal (the tree's single deliverable)

`grammars/json.ebnf` accepts exactly the RFC 8259 / ECMA-404 language: full
string escapes (`\"` `\\` `\/` `\b` `\f` `\n` `\r` `\t` `\uXXXX` incl.
surrogate pairs), full number grammar (optional minus, `0` or nonzero-lead
integer part — NO leading zeros, optional fraction, optional exponent
`e|E[+|-]digits`), RFC whitespace exactly (space/tab/LF/CR), and the RFC
rejection surface (raw control chars U+0000–U+001F inside strings, leading
zeros, trailing commas, lone surrogates as the honest documented boundary if
byte-level validation is out of grammar scope) — verified by a spec-derived
conformance gate (accept + reject matrices, expecteds from the RFC text, never
from the implementation), cert `fully_certified` re-earned at the new grammar,
and full book/contract lockstep.

## Known gap map (pre-audit reading of `grammars/json.ebnf`, 48 lines)

- Strings `/"[^"]*"/`: NO escapes (under-accept: `"a\"b"` truncates; `\uXXXX`
  absent) + raw control chars accepted (over-accept vs RFC §7).
- Numbers `/-?[0-9]+(\.[0-9]+)?/`: NO exponent (under-accept: `1e10`, `1E+5`)
  + leading zeros accepted (over-accept vs RFC §6: `int = zero / digit1-9 *DIGIT`).
- Whitespace `\s`: PCRE2 `\s` ⊃ RFC ws (over-accept: `\f`/`\v` as separators;
  RFC §2 allows only %x20 / %x09 / %x0A / %x0D).
- Structure (object/array/pair recursion, top-level any-value) looks RFC-shaped;
  to be confirmed by the `.1` matrix.

## Leaves

### `.1` — Tools-first conformance audit (read-only)

- Status: `next`. Build the RFC-derived accept/reject probe matrix (expecteds
  from the RFC text per [[feedback_corpus_expected_from_spec_not_fix]]) and run
  it through the REAL generated json parser (`--parse` / AST dumps); bank the
  measured gap matrix as evidence under
  `docs/tasks/artifacts/json_rfc8259/`. Output = the verified gap list that
  scopes `.2`.

### `.2` — DESIGN (grammar rewrite plan, annotations incl.)

- The RFC-faithful `json.ebnf` architecture: token-level string rule with the
  escape alternation + `\uXXXX`; the RFC number rule; explicit `ws` handling;
  AST-shape plan (keep the emitted shape stable where possible — schema-bump
  adjudication happens here); regen/bootstrap impact check (json is one of the
  11 shipped artifacts); fix-hierarchy level = EBNF (Level 1 — engine untouched
  per [[feedback_prefer_grammar_leave_engine_alone]]).

### `.3` — IMPLEMENT (grammar + regen + cert)

- Land the new grammar via canonical `make focus_json`; cert seeds 0/7/42
  `fully_certified spf=0` re-earned at the NEW rule census; other 10 grammars
  byte-inert; lib tests; clippy; release/schema/ledger bumps as adjudicated in
  `.2` (accepted-language change ⇒ bump expected).

### `.4` — VERIFY: the standing JSON conformance gate

- A spec-derived accept+reject corpus locked as a re-runnable gate (the
  per-family conformance-gate pattern); both fidelity directions asserted;
  determinism across seeds.

### `.5` — Lockstep

- json book + umbrella/family contract + LIVE tracker + MEMORY/CHANGES/
  TASK_TREE; the decision record cross-referenced.

## Acceptance Criteria (tree)

1. The grammar's accepted language equals RFC 8259/ECMA-404 on the locked
   conformance matrices (both directions), expecteds spec-derived.
2. Cert `fully_certified` (UNKNOWN=0, spf=0, seeds 0/7/42) at the new census.
3. All other grammars/parsers byte-inert; all standing gates green.
4. Honest boundaries (if any — e.g. lone-surrogate handling) DOCUMENTED in the
   book + contract, never silent.
5. Every slice tool-backed WHY+WHERE + measured before→after in its leaf.

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `JSON-RFC8259.1` (conformance audit) | `pending` — ⛔ PARKED with the tree | First leaf on GO: tools-first measured gap matrix (read-only). |
| — | `.2` DESIGN → `.3` IMPLEMENT → `.4` VERIFY → `.5` lockstep | `pending` | Sequential, after `.1`. |

## Log

- `2026-07-22` (session #189): Tree created on the director's verbatim
  commitment ([[project_json_rfc8259_full_standard_commitment]]). Pre-audit
  gap reading banked above (strings/numbers/whitespace, both fidelity
  directions).
- `2026-07-22` (same session, director, verbatim): "Do not switch to the JSON
  full support yet. Log the decision in a task-tree then move on." ⇒ tree
  PARKED at creation; no leaf opened; resumes only on an explicit director GO.
