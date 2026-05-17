# SV-EXH-PROOF: Main-SystemVerilog formally-exhaustive machine-checkable closure proof

## Metadata

- Tree ID: `SV-EXH-PROOF`
- Status: `active`
- Roadmap lane: parser-family exhaustive-proof normalization (the last open parser-family proof debt)
- Created: `2026-05-17`
- Last updated: `2026-05-17` (re-scoped from empirical investigation — see Decisions/Verification)
- Owner: repo-local workflow

## Goal

Re-earn `Done` for the `systemverilog` main-parser family
(`LIVE_ACHIEVEMENT_STATUS.md` rows "`systemverilog` main parser
(Phase P)" + "Parser-family exhaustive proof normalization") by
closing the **single primary unmet closure criterion** that SV's own
machine-checkable contract names: `formal_exhaustive_closure_surface_green`,
which is blocked solely on the missing
**`external_corpus_backed_proof_surface`** — a checked-in,
deterministic, repeatable external-corpus-backed proof sidecar that
promotes the bounded `sv_external_corpus_triage_gate` debt-discovery
surface into a formal exhaustive-closure claim (matching the LIVE
"Left To Close": *"promote the new external SV/UVM corpus triage
surface from bounded debt discovery into broader repeatable
realistic-corpus proof stages"*).

## Non-Goals

- **NOT** porting the `systemverilog_preprocessor`
  reachability/syntax/zero-plausible-gap trio — empirically falsified
  as the SV gap (see Decisions `2026-05-17` re-scope): SV **already
  has** `sv_syntax_closure_gate.sh` + `systemverilog_syntax_closure_contract.json`
  (`max_unreachable_rules: 1`, `max_unreachable_branches: 25` — static
  syntax-closure already essentially closed and consumed by
  `sv_parser_family_status_gate`). That surface is a no-regression
  baseline only, not a leaf.
- Not changing parser/grammar acceptance behavior except where a real
  external-corpus parse-failure must be closed to honestly assert the
  proof surface (each such grammar fix is its own sub-leaf under the
  Code-Change Doctrine).
- Not the broader Phase-S build-out (Liberty/SDC crates, rtl_frontend
  parity) — separate roadmap workstreams.

## Acceptance Criteria

- A checked-in `external_corpus_backed_proof_surface` sidecar +
  schema + contract: deterministic, repeatable, promoting
  `sv_external_corpus_triage_gate` from bounded discovery into a
  formal closure surface (declared == executed, zero blocked, every
  external-corpus parse-fail explicitly dispositioned — closed, or
  honestly recorded as a justified bounded carve-out with rationale;
  no false closure claims).
- `systemverilog_formal_exhaustive_closure_contract.json` flipped from
  "`required_surface` missing" to "surface present + its proof path";
  `sv_formal_exhaustive_closure_gate.sh` extended to *require* the
  sidecar; `sv_parser_family_status_gate` closure criteria all
  satisfied; `sota_exit_gate` + `sv_combined_telemetry_contract_gate`
  parity preserved.
- The two LIVE rows flip Mostly-Done / In-Progress → `Done` with the
  machine-checkable surface (not narrative) as evidence; per-parser SV
  book + integration contract in same-commit lockstep.
- All new/changed gates deterministic + green; no regression to the
  existing SV aggregate / roundtrip / stimuli / syntax-closure /
  external-corpus / family-status gates; full COMMIT.md lockstep per
  leaf.

## Task Tree

- ID: `SV-EXH-PROOF`
  Status: `active`
  Goal: `Close SV's single primary unmet closure criterion (formal_exhaustive_closure_surface_green) by building the external_corpus_backed_proof_surface; re-earn Done for the SV main-parser family.`
  Children: `SV-EXH-PROOF.1`, `SV-EXH-PROOF.2`, `SV-EXH-PROOF.3`, `SV-EXH-PROOF.4`

- ID: `SV-EXH-PROOF.1`
  Status: `pending`
  Goal: `Measured baseline + scope lock: run the four existing gates (sv_syntax_closure_gate, sv_external_corpus_triage_gate, sv_parser_family_status_gate, sv_formal_exhaustive_closure_gate) and record a deterministic checked-in baseline proving (a) SV static syntax-closure already meets its contract (no-regression only), (b) the single primary unmet closure criterion is formal_exhaustive_closure_surface_green blocked on the missing external_corpus_backed_proof_surface, (c) the current external-corpus triage summary numbers.`
  Acceptance: `All four gate measurements captured deterministically + the exact primary unmet criterion + required_surface_key + current triage summary recorded in a checked-in baseline; confirms the trio is NOT the gap and the external-corpus-backed proof surface IS.`
  Verification: `pending`
  Commit: `pending`

- ID: `SV-EXH-PROOF.2`
  Status: `pending`
  Goal: `Design + build the external_corpus_backed_proof_surface: a checked-in deterministic generator + schema + contract that promotes sv_external_corpus_triage_gate bounded discovery into a formal closure surface (declared == executed, zero blocked, every parse-fail dispositioned/justified). Each real grammar fix required to close a corpus parse-fail is its own sub-leaf (Code-Change Doctrine).`
  Acceptance: `Sidecar + schema + contract checked in; deterministic + repeatable gate; every external-corpus parse-fail explicitly accounted (closed or honestly justified-bounded with rationale — no false closure).`
  Verification: `pending`
  Commit: `pending`

- ID: `SV-EXH-PROOF.3`
  Status: `pending`
  Goal: `Wire the surface in: flip systemverilog_formal_exhaustive_closure_contract.json "surface missing" -> "surface present + proof path"; extend sv_formal_exhaustive_closure_gate.sh to require the sidecar; sv_parser_family_status_gate + sota_exit_gate + sv_combined_telemetry_contract_gate parity.`
  Acceptance: `Formal-exhaustive gate green requiring the real surface; family-status closure criteria all satisfied; telemetry parity machine-checked; no regression to existing SV gates.`
  Verification: `pending`
  Commit: `pending`

- ID: `SV-EXH-PROOF.4`
  Status: `pending`
  Goal: `Flip the two LIVE rows Mostly-Done / In-Progress -> Done with the machine-checkable surface as evidence; per-parser SV book + integration contract same-commit lockstep; full closeout + tree close.`
  Acceptance: `LIVE "systemverilog main parser" + "Parser-family exhaustive proof normalization" rows Done with evidence; SV book + contract lockstepped; tree closed; promoted to Completed in TASK_TREE.md.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `SV-EXH-PROOF.1` | `pending` | A deterministic measured baseline of the four existing gates locks the honest scope (trio already closed; the external-corpus-backed proof surface is the sole gap) before any sidecar/contract code is written. |
| 2 | `SV-EXH-PROOF.2` | `pending` | Build the external_corpus_backed_proof_surface sidecar + contract (needs `.1`'s measured baseline of the triage surface to promote). |
| 3 | `SV-EXH-PROOF.3` | `pending` | Wire the surface into the formal-exhaustive contract/gate + family-status + telemetry (needs `.2`'s sidecar). |
| 4 | `SV-EXH-PROOF.4` | `pending` | LIVE `Done` flip + book/contract lockstep + closeout (needs `.3` green). |

## Decisions

- `2026-05-17`: User selected this workstream (the largest open
  parser-family debt) from the post-`POST-SV-AUDIT` strategic fork.
- `2026-05-17` (initial): hypothesised the SV gap was a port of the
  `systemverilog_preprocessor` reachability/syntax/zero-plausible-gap
  trio (setup commit `PGEN-SV-EXH-PROOF-0000`).
- `2026-05-17` (**re-scope — hypothesis partially falsified by
  checked-in ground truth**): investigation of the actual proof stack
  proved SV **already has** the shared `sv_syntax_closure_gate.sh` +
  `systemverilog_syntax_closure_contract.json` (`max_unreachable_rules: 1`,
  `max_unreachable_branches: 25`; consumed by
  `sv_parser_family_status_gate` lines 274-275 — static syntax-closure
  essentially closed). SV's own
  `systemverilog_formal_exhaustive_closure_contract.json` names the
  precise gap: `required_surface_key == "external_corpus_backed_proof_surface"`
  with `required_surface_missing_detail` = *"SystemVerilog still lacks
  an explicit checked-in external corpus-backed proof surface sidecar
  that can promote the current bounded family evidence into a formal
  exhaustive closure claim."* `sv_formal_exhaustive_closure_gate.sh`
  validates exactly that key (lines 91-93); `sv_parser_family_status_gate.sh`
  surfaces criterion `formal_exhaustive_closure_surface_green` (line
  311). Conclusion: **the single primary unmet closure criterion is
  the missing `external_corpus_backed_proof_surface`**, not the trio.
  Tree re-decomposed accordingly. This is PNT-eligible engineering
  judgement: the user fixed the *workstream*; correct decomposition is
  the implementer's call and the ground truth is unambiguous + self
  -documenting via SV's own contract.
- `2026-05-17`: **Code-Change Doctrine compliance** — every sidecar
  generator / schema / contract json / gate-script change / any
  `grammars/systemverilog.ebnf` fix is task-tree-owned by its leaf (or
  a sub-leaf split for a real external-corpus grammar gap). No
  out-of-tree code changes.

## Open Questions

- How many external-corpus parse-fails are real closable grammar gaps
  vs honest justified-bounded carve-outs? Resolved empirically in
  `.1` (measure) / `.2` (disposition). Does not block `.1`.

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-17` | `SV-EXH-PROOF` (setup) | task-tree decomposition vs workflow splitting rules; Code-Change-Doctrine precursor compliance | `pass — tree created (initial preprocessor-trio-port hypothesis)` |
| `2026-05-17` | `SV-EXH-PROOF` (re-scope) | empirical audit of the SV proof stack: `systemverilog_syntax_closure_contract.json` exists (`max_unreachable_rules:1`); `systemverilog_formal_exhaustive_closure_contract.json` `required_surface_key == external_corpus_backed_proof_surface` + missing-detail; `sv_formal_exhaustive_closure_gate.sh:91-93`; `sv_parser_family_status_gate.sh:274-275,311`; `sv_external_corpus_triage_gate.sh` summary shape | `pass — initial trio-port hypothesis FALSIFIED against checked-in ground truth; tree re-decomposed to the real single gap (external_corpus_backed_proof_surface); honest auditable re-scope recorded before any code leaf` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SV-EXH-PROOF` (setup) | `PGEN-SV-EXH-PROOF-0000` | tree created + activated (initial trio-port hypothesis) |
| `SV-EXH-PROOF` (re-scope) | `PGEN-SV-EXH-PROOF-0001` | hypothesis falsified by checked-in ground truth; re-decomposed to the real gap (external_corpus_backed_proof_surface); frontier still `.1` |

## Changelog

- `2026-05-17`: Created + activated the task tree (user-selected from
  the strategic fork after POST-SV-AUDIT/TaskList #49 closed); initial
  decomposition hypothesised a preprocessor-trio port.
- `2026-05-17`: **Re-scoped.** Empirical audit of the SV proof stack
  falsified the trio-port hypothesis: SV already has the static
  syntax-closure surface; SV's own formal-exhaustive contract names
  the sole gap as the missing `external_corpus_backed_proof_surface`.
  Re-decomposed into `.1` measured baseline + scope lock → `.2` build
  the external-corpus-backed proof surface sidecar/contract → `.3`
  wire it into the formal-exhaustive contract/gate + family-status +
  telemetry → `.4` LIVE `Done` flip + book/contract lockstep +
  closeout. Frontier `.1`. Code-Change-Doctrine-compliant (tree-owned).
