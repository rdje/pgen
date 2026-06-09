# SV-PARSE-STRICT: the SV parser rejects provably-undeclared type/nettype identifiers at parse time (sound, context-aware store-gating)

## Metadata

- Tree ID: `SV-PARSE-STRICT`
- Status: `active`
- Roadmap lane: SystemVerilog main-parser correctness — parser/elaborator boundary enforcement
- Created: `2026-06-09`
- Last updated: `2026-06-09`
- Owner: repo-local workflow

## Provenance (how this was found)

Found **by the stimuli generator**, as the director immediately recognized. The
`GRAMMAR-WELLFORMED` Phase-H certificate-coverage **diverse pass** (which *is* the
stimuli generator) emitted, for `sv_2017`, samples that fuse a closing keyword with the
next top-level item's opening keyword across the `source_text_item*` boundary —
`endprogram`+`module` → `endprogrammodule`, `endmodule`+`module` → `endmodulemodule`,
`endmodule`+`config` → `endmoduleconfig` (a *generator* faithfulness bug, see "Defect A"
below). The witness round-trip re-parse flagged the `endprogram` variants as
`sample_parse_failures=3`. Chasing *why those fail but the `endmodule` variant passes*
exposed a latent **parser over-acceptance** (this tree's subject). The generator did its
job as a **bug-finding oracle**: it produced an input no human would hand-write, which
then exposed the parser bug.

## The two distinct defects

| | Defect | Effect | Owner |
| --- | --- | --- | --- |
| **A** | The stimuli generator fuses a `\b`-tailed closing keyword with the next item's opening keyword across the `source_text_item*` boundary — the LEXICAL-ANNOTATIONS.5 deferred word-boundary space is dropped because the boundary items are emitted via the whole-rule `@sample` literal-override path (`generate_rule` 5196–5214), which returns the hint **without** updating `last_terminal_word_shaped`. | The `GRAMMAR-WELLFORMED` SV cert-coverage `sample_parse_failures=3`. | `GRAMMAR-WELLFORMED` (generator-faithfulness; deferred behind this tree per the director: "no point on cert-cover if a clearly identifiable parser bug shows up — fix parser bugs ASAP"). |
| **B** | **This tree.** The SV `net_declaration` `nt a;` branch (`systemverilog.ebnf:3279`) accepts a bare **undeclared** identifier as a user-defined nettype — it is the one type-position identifier rule that does NOT consult the store. | `module a; endmodulemodule b; endmodule` parses (to a wrong-but-syntactically-legal structure) instead of being rejected. | This tree. |

## Tools-proven root cause (Defect B)

Decisive A/B + AST + `--trace debug` (`parseability_probe --parse systemverilog … --profile
sv_2017`):

- `module a; endmodulex b; endmodule` → **accept** — proves a fused/odd identifier
  (`endmodulex`, `endmodulemodule`) is parsed as an ordinary identifier. The `\b` in
  `kw_endmodule := trivia /endmodule\b/` correctly prevents `endmodule` from matching
  inside `endmodulemodule`, so it is a *legal identifier*, not the keyword. (Correct.)
- AST dump of `endmodulemodule b;` → `net_declaration` with `net_type_id:
  "endmodulemodule"`, net name `b`. The `--trace debug` smoking gun: every real net-type
  keyword (`wire`/`tri`/`supply`/…) fails, then `net_type_identifier_sv_2017`
  *successfully* consumes `endmodulemodule` (a bare `declaration_identifier`).
- `program p; endprogrammodule m; endprogram` → **accept** (proper closer) and
  `module a; endmodulemodule b; endprogram` → **reject** (wrong closer) — proving the
  program-vs-module asymmetry is only about the *trailing closer*, not the parser
  mishandling the identifier.

Root cause: `net_declaration_sv_2017`'s second branch
(`net_type_identifier ( delay_control )? list_of_net_decl_assignments semi`) has **no
`@predicate`**, and `net_type_identifier := … := declaration_identifier` is a bare
identifier. So it optimistically accepts ANY identifier as a user-defined net type. This
is the exact provisional/ungated identifier-categorization class that
[[feedback_grammar_rules_must_consult_store]] calls "defective by design" and that the
`docs/book/src/grammar-wellformedness.md` chapter flags as the "honest residual" needing
**semantic store-gating** — and it is the concrete instance of the
`GRAMMAR-WELLFORMED.A2.1` deferred residual "(4b) port-header/net-type family needs
nettype/interface STORE-GATING."

## The parser/elaborator boundary (director-confirmed 2026-06-09)

SystemVerilog's grammar is **not context-free** — `a b;`, `a (b);`, `a #(b) c;` parse
differently depending on whether `a` is a type/module/value (the classic type-name
feedback / "lexer hack" problem). So a correct SV parser is *necessarily* context-aware;
PGEN does this via the semantic store. The director **confirmed** the boundary
("Context-aware, sound gating"):

- **Parse phase (PGEN):** lexical + syntactic recognition → AST; plus **sound,
  single-translation-unit name categorization** (type / class / package / nettype …) via
  the store, using only facts establishable *within this parse* — the current TU plus any
  explicitly linked library facts (`--lib-in`). It commits to "is/isn't a type" **only
  when it can prove it**. It **rejects** a *provably*-undeclared type identifier at parse
  time, and **accepts** when it cannot prove (an unresolved `import pkg::*` is in scope) —
  the same *sound-not-complete* discipline the grammar linter uses.
- **Elaborate phase (later):** cross-unit binding (unresolved external `import pkg::*`,
  hierarchy, libraries); type/width/parameter checking; generate unrolling — anything
  needing the *whole design* or value-level semantics.

Under this boundary: `endmodulemodule b;` (no declaration, no `import ::*` escape in
scope) → the parser *can prove* it is undeclared → **reject** (the bug); but
`import foo_pkg::*; foo_type b;` (external, unlinked) → the parser *cannot prove*
`foo_type` is not from `foo_pkg` → **accept**, defer to the elaborator.

## The fix design (sound gating with the wildcard-import escape)

Critical mechanism finding: the *current* ungated `nt a;` branch is **also** the de-facto
escape-hatch that lets an unresolved-import type (`import foo_pkg::*; foo_type b;`) parse
today — because the parallel `data_declaration` type path is already a HARD store-gate
(`checked_type_identifier` → `@predicate has_fact(type_name, $body)`, no escape), so an
unresolved-import type currently falls through to the ungated net branch. Therefore a
naive hard gate on the net branch would fix `endmodulemodule b;` but **unsoundly reject**
`import foo_pkg::*; foo_type b;`. The sound gate must be:

> accept the `nt a;` declaration iff `has_fact(type_name, $1)` **OR** a wildcard import is
> in scope.

Concrete grammar changes (engine untouched — [[feedback_prefer_grammar_leave_engine_alone]]):

1. **Emit a "wildcard import in scope" fact** on the wildcard `package_import_item`
   branch (`package_identifier scope_resolution star`, `systemverilog.ebnf:3593`) — e.g.
   `@emit_fact: { kind: wildcard_import_open, … }` with a `@fact_kind` declaration. (Today
   `pkg::*` only does `@import_from_library` — a fact *merge*, no "open" marker.)
2. **Gate the `nt a;` net-declaration branch** so it matches only when the identifier is a
   known type/nettype OR a wildcard import is open. Two sound implementations to evaluate
   in `.2`: (a) two single-`has_fact` branches (known-type branch + wildcard-escape
   branch) reusing the proven single-predicate idiom; or (b) one composed `@predicate_def`
   disjunction (would be the first `@predicate_def` in the SV grammar).
3. **Emit `type_name {declaration_family: nettype}` from `nettype_declaration`** (today it
   emits nothing) so a *genuinely declared* nettype (`nettype int nt; nt a;`) still parses
   via the known-type branch (binding-before-use; mirrors class/interface/covergroup).

## Non-Goals

- Defect A (the generator fusion / cert-coverage faithfulness) — owned by
  `GRAMMAR-WELLFORMED`, deferred behind this parser bug per the director.
- Full elaboration semantics (type/width/parameter checking, generate unrolling,
  cross-unit binding of unprovided units) — out of the parse-phase boundary by design.
- The *other* ungated identifier-categorization siblings (port-header/net-type family,
  the broader `A2.1` residual) — captured as `.3` (deferred) so the first fix stays a
  reviewable slice.

## Acceptance Criteria

- `module a; endmodulemodule b; endmodule` and `module a; zzqq yy; endmodule` are
  **rejected** at parse (no `import ::*` in scope).
- `nettype int nt; nt a;` (declared nettype) and `import pkg::*; foo_type b;` (wildcard in
  scope) are **accepted** (soundness preserved).
- The SV external corpus stays **14/14** (`sv_external_corpus_triage_gate`,
  incl. uvm_pkg ×2 / scr1 / friscv / veer) — the decisive regression oracle.
- Realistic corpus + `cargo test --features generated_parsers --lib` stay green; SV
  shape-contract gate green (manifest updated if `net_type_id`'s captured shape changes).
- Books ↔ code lockstep (the SV parser book + grammar-wellformedness chapter note the new
  parse-time rejection); release/schema bump decided per the AST-shape delta.

## Task Tree

- ID: `SV-PARSE-STRICT`
  Status: `active`
  Goal: the SV parser rejects provably-undeclared type/nettype identifiers at parse time, soundly.
  Children: `SV-PARSE-STRICT.1`, `SV-PARSE-STRICT.2`, `SV-PARSE-STRICT.3`

- ID: `SV-PARSE-STRICT.1`
  Status: `done`
  Goal: scope + root-cause (tools-first) the generator-found parser over-acceptance; capture the director-confirmed parser/elaborator boundary + the sound-gating design.
  Acceptance: this file records the tools-proven root cause, the confirmed boundary, and the sound fix design (with the wildcard-import escape).
  Verification: A/B + AST dump + `--trace debug` (all in this file); pure-docs, no code change.
  Commit: `PGEN-SV-PARSE-STRICT-0001`

- ID: `SV-PARSE-STRICT.2`
  Status: `done`
  Goal: implement the sound gate on the `net_declaration` `nt a;` branch — known-type/nettype OR wildcard-import-open — plus the `wildcard_import_open` fact emission and the `nettype_declaration` `type_name` emit.
  Acceptance: the Acceptance Criteria above hold; SV external corpus 14/14; decisive A/B; no cross-grammar regression.
  Verification: A/B (release `parseability_probe`): all 4 reject cases REJECT (`endmodulemodule b;`, `module a; zzqq yy;`, `foo_type b;` no-import, bare `nt a;`); all 6 accept cases PASS (declared nettype file/module scope, wildcard-import file/module scope, plain `wire`, plain `logic`) + 3 net-decl sanity (wire-vec, tri/wand, multi-net). SV external corpus **14/14** (`sv_external_corpus_triage_gate`: parse_pass_total=14, parse_fail_total=0). lib `--features generated_parsers` 689/0 (shape-contract incl. the updated fact-kind registry test → 4 kinds). Grammar-only (engine untouched). One mid-implementation regression found+fixed tools-first: an inline `phase: branch` predicate is flattened rule-wide (`branch_predicates_for_rule`) and wrongly gated the `wire a;` branch → moved the wildcard gate to a `phase: post` helper rule (`wildcard_escape_nettype_identifier`), branch-local. KM card `branch-predicate-is-rule-wide`.
  Commit: `PGEN-SV-PARSE-STRICT-0002`

- ID: `SV-PARSE-STRICT.3`
  Status: `deferred`
  Goal: extend sound store-gating to the sibling ungated identifier-categorization rules (port-header / net-type family — the broader `GRAMMAR-WELLFORMED.A2.1` "(4b)" residual), each with its own corpus-verified slice.
  Acceptance: each sibling rule consults the store; corpus 14/14 preserved per slice.
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| — | `SV-PARSE-STRICT.2` | `done` | The sound-gating fix landed (`PGEN-SV-PARSE-STRICT-0002`); the generator-found `endmodulemodule b;` over-acceptance is REJECTED, corpus 14/14. |
| 1 | `SV-PARSE-STRICT.3` | `deferred` | The broader ungated identifier-categorization class (port-header / net-type family, `GRAMMAR-WELLFORMED.A2.1` "(4b)"); `.2` proved the idiom + corpus-safety. Activate on demand. |

The parser bug is FIXED (`.2` done). The only open child (`.3`, the sibling
ungated-categorization rules) is `deferred`.

## Decisions

- `2026-06-09`: **Parser/elaborator boundary = context-aware, sound gating** (director-confirmed via AskUserQuestion). The parser rejects *provably*-undeclared type identifiers at parse time and accepts when it cannot prove (unresolved wildcard import in scope). `endmodulemodule b;` IS a parser bug. Recorded as a standing principle.
- `2026-06-09`: **Highest priority = fix parser bugs ASAP** (director, emphatic): "There is no point on cert-cover if a clearly identifiable parser bug shows up." Defect A (cert-coverage generator faithfulness) is deferred behind this parser fix.
- `2026-06-09`: **Sound gate, not hard gate.** Because the ungated `nt a;` branch is today the de-facto escape for unresolved-import types, the fix must preserve acceptance under an in-scope wildcard import (`has_fact(type_name,$1) OR wildcard_import_open`). A naive hard `has_fact` gate would be unsound.
- `2026-06-09`: **Grammar-only** (engine off-limits, [[feedback_prefer_grammar_leave_engine_alone]]); reuse the proven `@predicate has_fact(type_name, …)` / `@emit_fact type_name` idioms.
- `2026-06-09`: **`.2` now depends on `INLINE-ACTIONS.2`** (director, AskUserQuestion: "Wire branch-start emit first, then fix"). Piece 1 — emit `wildcard_import_open` on the *wildcard* `package_import_item` branch — is a **branch-start** `@emit_fact`, a placement the runtime does not yet fire (tools-proven gap, owned by the new `INLINE-ACTIONS` tree). Rather than reshape the wildcard branch into a dedicated helper rule (the non-elegant workaround), the director directed wiring branch-start emit as a general parser-agnostic feature first; `.2` then expresses piece 1 as a clean inline annotation. (A helper-rule fallback remains available if ever needed.)
- `2026-06-09` (`.2`, tools-first correction): the wildcard escape uses a **`phase: post` helper rule** (`wildcard_escape_nettype_identifier`), NOT an inline `phase: branch` predicate. An inline `phase: branch` predicate is **flattened rule-wide** by `branch_predicates_for_rule` (`semantic_runtime.rs:735`) and evaluated against EVERY branch, so the content-free `fact_count_at_least(wildcard_import_open, 1)` wrongly REJECTED the sibling `wire a;` branch (decisive `--trace-rules` proof). A `phase: post` predicate on a dedicated helper used by only that branch stays branch-local. Captured as KM `branch-predicate-is-rule-wide`.

## Open Questions

- `.2` implementation choice: two single-`has_fact` branches vs. one composed `@predicate_def` disjunction. Resolved in `.2` by whichever keeps the AST shape stable and the change minimal; does not block the frontier.
- Does the SV external corpus exercise the unresolved-wildcard escape, or do its imports all resolve (so a hard gate would also pass)? Answered empirically in `.2` by running the corpus with the gate; the escape is implemented regardless for downstream single-file soundness.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-09` | `SV-PARSE-STRICT.1` | A/B parse (t1–t10), AST dump, `--trace debug` on the fail + pass cases, grammar read of `net_declaration`/`net_type_identifier`/`nettype_declaration`/`package_import_item` | root cause proven; pure-docs |
| `2026-06-09` | `SV-PARSE-STRICT.2` | A/B (4 reject + 6 accept + 3 net-decl sanity); `--trace-rules` root-cause of the `wire a;` regression; SV external corpus triage gate; lib `--features generated_parsers` (shape-contract incl. updated fact-kind test); grammar diff review | reject 4/4, accept 9/9; **corpus 14/14** (parse_pass 14, fail 0); lib **689/0**; pass |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SV-PARSE-STRICT.1` | `PGEN-SV-PARSE-STRICT-0001` | scoping/design (pure docs) |
| `SV-PARSE-STRICT.2` | `PGEN-SV-PARSE-STRICT-0002` | the sound-gating implementation (grammar-only) |

## Changelog

- `2026-06-09`: Created task tree from the director dialogue; `.1` scoping/design done (tools-first root cause + confirmed parser/elaborator boundary + sound-gating fix design); `.2` (implementation) is the frontier.
- `2026-06-09`: `.2` DONE (`PGEN-SV-PARSE-STRICT-0002`) — the sound gate landed (grammar-only, consuming `INLINE-ACTIONS.2`'s branch-start `@emit_fact`). `endmodulemodule b;` / undeclared net-types REJECTED; declared nettype + in-scope `pkg::*` + plain `wire`/`logic` ACCEPTED; SV external corpus 14/14; lib 689/0. A mid-implementation `wire a;` regression was root-caused tools-first (an inline `phase: branch` predicate is flattened rule-wide by `branch_predicates_for_rule`) and fixed by moving the wildcard gate to a `phase: post` helper rule (branch-local). KM card `branch-predicate-is-rule-wide`. `.3` (sibling rules) stays deferred.
