# Task Tree: SV-OVER-REJECTION-FIDELITY

Owner tree for the three remaining **over-REJECTION** defects that bound the SystemVerilog /
`verilog_2005` LRM-fidelity claim — spec-valid input that `grammars/systemverilog.ebnf` wrongly
REJECTS (the worst defect class: a parser rejecting known-valid input, per
`feedback_fix_parser_bugs_asap_highest_priority` / `feedback_uvm_is_valid_sv`). Surfaced
2026-07-02 by the `VERILOG-2005-PROFILE.4.1`/`.4.2` conformance-corpus probes and ledgered
`SV-0021`/`SV-0022`/`SV-0023`; carved into their own focused tree 2026-07-04 (session #34) from the
standing SV/`verilog_2005` FINALIZATION lane, mirroring the sibling all-profile SV-correctness trees
(`SV-KEYWORD-PRIMARY-FIDELITY`, `SV-LRM-SHAPE-FIDELITY`, `SV-DOLLAR-LRM-FIDELITY`). One concern per
leaf, one leaf per commit.

- **`SV-0021`** — mixed ANSI port list: an implicit-type ANSI port followed by an explicitly-typed
  one — `module m (input a, output reg b);` — wrongly REJECTED under **every** profile. Valid IEEE
  1800 (A.1.3) and valid IEEE 1364-2005 (ANSI port declarations with a `reg` type). **Already FIXED
  as a consequence of `SV-0037`** (`SV-KEYWORD-PRIMARY-FIDELITY.3.1`, `1.0.162`→`1.0.163`): the
  greedy ungated `net_type_identifier` in `net_port_type_sv_2017/2023` was eating the implicit
  port's name under the global `longest_match` policy, corrupting the first port into a `nonansi`
  shape so a following typed ANSI port had no valid continuation → over-rejection. This tree's `.1`
  leaf ADJUDICATES `SV-0021` closed and adds the missing regression lock.
- **`SV-0022`** — `bind` double-`semi`: a spec-valid single-`semi` `bind … ;` never parses under any
  SV profile while the invalid `;;` form accepts. `bind_directive` (`grammars/systemverilog.ebnf:621`)
  requires a trailing `semi` after `bind_instantiation`, but every `bind_instantiation` alternative
  already consumes its own `;`. Over-rejection **+** over-acceptance pair. *(open — grammar fix)*
- **`SV-0023`** — event triggers: the IEEE 1800 §9.4.4 NONBLOCKING trigger `->>` is unsupported (the
  literal `->>` is absent from the grammar); the optional `[delay_or_event_control]` is mis-attached
  to a second `->` branch (`-> #5 e;` wrongly accepts); and the 1364-2005 event-array trigger
  `-> e[0];` rejects. Under-support + over-acceptance complex; the swapped `kind` labels are
  consumer-visible → schema-bump candidate. *(open — grammar fix)*

## Acceptance Criteria (tree)

- Each spec-valid form ACCEPTS in every profile whose LRM admits it, at ALL carriers (re-probe after
  each fix for sibling leaks, cf. `VERILOG-2005-PROFILE.6.11`'s cascade discipline).
- Each spec-invalid sibling (e.g. the `bind` `;;` form) REJECTS where the LRM forbids it.
- Every fixed / adjudicated form is pinned by a re-runnable regression lock in the
  `verilog_2005_conformance` corpus + contract so it cannot silently regress.
- Full no-regression where a parser/grammar change is made: canonical/union cert (seeds 0/7/42),
  `ast_shape_contract`, the external corpus, the `verilog_2005` conformance matrix + cert,
  `--lint-grammar` 0 profile-orphans, the 6 fully-certified grammars byte-identical, clippy
  source-clean. For an adjudication-only leaf (no parser change), the conformance gate is the
  necessary + sufficient oracle and the parser-behavior gates are inert by construction (proven by a
  grammar/generated-free diff).
- Release/schema per `PGEN_RELEASE_POLICY`: an accept↔reject behavior change on shipped SV profiles →
  release bump; schema unchanged unless a NEW AST envelope shape is introduced (the `SV-0023` label
  correction is the schema-bump candidate).

## Current Frontier

- ID: `SV-OVER-REJECTION-FIDELITY.1` — Status: `done` (2026-07-04, session #34,
  `PGEN-SV-OVER-REJECTION-FIDELITY-0001`, CORPUS/adjudication leaf — no parser change) — closes
  `SV-0021`. The ledgered all-profile over-rejection `module m (input a, output reg b);` no longer
  reproduces on HEAD (`1.0.165`); tools-first proved it was fixed as a consequence of `SV-0037`
  (see "`.1` Findings" below). This leaf adds the missing regression lock
  (`accept/ansi_implicit_then_typed_port.v`, accept under all 3 profiles) + adjudicates the ledger
  row `Root Caused`→`Released`. No grammar/parser/schema/release change. See "Acceptance Checklist —
  `.1`" below.
- ID: `SV-OVER-REJECTION-FIDELITY.2` — Status: `pending` — close `SV-0022` (`bind` double-`semi`).
- ID: `SV-OVER-REJECTION-FIDELITY.3` — Status: `pending` — close `SV-0023` (event-trigger complex;
  schema-bump candidate).

## `.1` Findings (tools-first, 2026-07-04 session #34 — SV-0021 mixed ANSI port list)

**REPRODUCE (ledger repro, now on HEAD release binary `1.0.165`):**
```
printf 'module m (input a, output reg b); endmodule\n' \
  | parseability_probe --parse systemverilog /dev/stdin --profile sv_2017   → parse_full passed (rc0)
                                                         --profile sv_2023   → parse_full passed (rc0)
                                                         --profile verilog_2005 → parse_full passed (rc0)
```
The exact ledgered failing case (which historically REJECTED at `furthest_position=25` on
`1.0.0`–`1.0.158`) now ACCEPTS on all three profiles, deterministically (3×). A broad 20-case
mixed-ANSI permutation battery (implicit→typed, typed→implicit, ranged, `inout`, `logic`/`wire`/`reg`
typed) is 20/20 ACCEPT across `sv_2017`/`sv_2023`. The AST is a coherent `ansi` header (port `a`
implicit `net_or_interface`, port `b` typed with `data_type → reg`).

**ROOT CAUSE (WHY + WHERE) — fixed as a consequence of `SV-0037`:** git provenance pins the fixing
commit `19fc7ed9` (`PGEN-SV-KEYWORD-PRIMARY-FIDELITY-0004`, `SV-KEYWORD-PRIMARY-FIDELITY.3.1`,
`1.0.162`→`1.0.163`). Its tools-first trace showed `net_port_type_sv_2017` (`grammars/systemverilog.ebnf:3474`,
alt 1 `net_type_identifier`) / `net_port_type_sv_2023` (`:3479`) run under the GLOBAL DEFAULT
`branch_policy=longest_match` (not ordered-choice), so the ungated `net_type_identifier` matching a
bare implicit-port name (2 bytes) out-consumed alt 0's empty implicit match (0 bytes) and ATE the
port name — corrupting the first implicit-type ANSI port into a `nonansi` shape. When the next port
was explicitly typed (`output reg b`), the `nonansi` framing had no valid continuation → the whole
ANSI header over-rejected (`SV-0021`). The `.3.1` fix routed both alts to the store-gated
`checked_nettype_identifier` (`:3537`, the `SV-PARSE-STRICT.2` declared-nettype gate) so alt 0's
implicit `ansi` path correctly wins and the name binds; the mixed implicit-then-typed header then
parses. `SV-0021` and `SV-0037` share the identical mechanism and rule seam; `SV-0021` is the
over-rejection surface of the same defect, and both were closed by the one `.3.1` change.

**FIX (this leaf):** no parser change — the behavior is already correct. Add the missing regression
lock so the mixed-ANSI-header acceptance cannot silently regress:
`rust/test_data/grammar_quality/verilog_2005_conformance/accept/ansi_implicit_then_typed_port.v`
(`module m (input a, output reg b); endmodule`) + its contract case (accept/accept/accept, alongside
the sibling port locks `accept/port_concat.v` / `reject/port_bare_multi_id.sv`).

## Acceptance Checklist (`.1`, enforced)

- [x] **REPRODUCE / ISSUE** — ledger repro `module m (input a, output reg b);` now `parse_full passed`
  on `sv_2017`/`sv_2023`/`verilog_2005` (rc0), deterministic 3×; 20/20 mixed-ANSI battery ACCEPT. The
  historical over-rejection (`furthest_position=25`) is gone.
- [x] **ROOT CAUSE (WHY + WHERE)** — fixed as a consequence of `SV-0037`: the ungated
  `net_type_identifier` (`net_port_type_sv_2017/2023`, `grammars/systemverilog.ebnf:3474`/`:3479`)
  ate the implicit port name under `longest_match`, corrupting the first port to `nonansi` so a
  following typed ANSI port had no continuation. Fixing commit `19fc7ed9`
  (`SV-KEYWORD-PRIMARY-FIDELITY.3.1`, routed both alts to store-gated `checked_nettype_identifier`
  `:3537`).
- [x] **FIX** — declarative/test tier: add the `verilog_2005_conformance` accept-lock
  `accept/ansi_implicit_then_typed_port.v` + contract case; no grammar/parser/schema/release change
  (parser already correct at `1.0.165`).
- [x] **ADDRESSED (verified)** — `verilog_2005_conformance_gate` GREEN (`gate_green: true`, 0 unmet):
  the new lock `accept/ansi_implicit_then_typed_port.v` parses `observed=accept` under all 3 profiles
  (`verilog_2005`/`sv_2017`/`sv_2023`), matrix `228`→`231` file×profile checks / **0 mismatches**.
- [x] **NO REGRESSION** — grammar/generated-free diff (only `rust/test_data/…` corpus+contract + docs
  touched → parser behavior byte-identical → canonical/union cert, `ast_shape_contract`, external
  corpus, the 6 fully-certified grammars all inert by construction). `verilog_2005_conformance_gate`
  re-verified GREEN seeds 0/7/42: cert `1117/4/773/340` byte-identical (corpus-independent), matrix
  `231/0`, `--lint-grammar` orphans 0. *(pasted below.)*
- [x] **LOCKSTEP** — ledger `SV-0021` `Root Caused`→`Released`; LIVE dialect block, `CHANGES.md`,
  `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `docs/TASK_TREE.md`, the conformance contract `note`, and the
  matrix-count mentions (`228`→`231`) updated. No user-facing parser-behavior change (already shipped
  at `1.0.163`) → no book chapter drift.

## Decisions

- `2026-07-04`: `SV-0021` is closed by ADJUDICATION + a regression lock, not a new fix, because
  tools-first proof (git provenance + the 20/20 battery) shows it was already resolved as a
  consequence of `SV-0037`. Owning it in its own leaf keeps the ledger honest and pins the
  acceptance so it cannot silently regress.
- `2026-07-04`: the tree is carved separately from `SV-LRM-SHAPE-FIDELITY` (over-ACCEPTANCES) and
  `SV-KEYWORD-PRIMARY-FIDELITY` (keyword-as-primary) because over-REJECTIONS are a distinct defect
  class (spec-valid input rejected) with distinct mechanisms; grouping `SV-0021`/`SV-0022`/`SV-0023`
  matches the `VERILOG-2005-PROFILE` LIVE framing of "the remaining all-profile over-REJECTION
  complex."

## Open Questions

- None blocking the frontier. `SV-0022`/`SV-0023` (`.2`/`.3`) are genuine open grammar fixes deferred
  to their own leaves.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-07-04` | `SV-OVER-REJECTION-FIDELITY.1` | ledger repro + 20-case mixed-ANSI battery (release `1.0.165`) | `20/20 ACCEPT` on `sv_2017`/`sv_2023`; ledger repro `parse_full passed` on all 3 profiles, deterministic 3× |
| `2026-07-04` | `SV-OVER-REJECTION-FIDELITY.1` | `verilog_2005_conformance_gate` (rebuild + matrix + cert seeds 0/7/42) | **GREEN** (`gate_green: true`, 0 unmet): new lock ACCEPT all 3 profiles, matrix `231`/0 mismatches, lint `profile_orphans=0` (1465 rules), cert `1117/4/773/340` spf=0 byte-identical seeds 0/7/42 |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SV-OVER-REJECTION-FIDELITY.1` | `PGEN-SV-OVER-REJECTION-FIDELITY-0001 (SV-OVER-REJECTION-FIDELITY.1): SV-0021 CLOSED — mixed implicit-then-typed ANSI port list adjudicated Released (fixed as consequence of SV-0037) + regression-locked` | Corpus/adjudication leaf — no grammar/parser/schema/release change |

## Changelog

- `2026-07-04`: Created task tree (session #34) for the `SV-0021`/`SV-0022`/`SV-0023` over-rejection
  complex; opened `.1` (SV-0021 adjudication + regression lock).
