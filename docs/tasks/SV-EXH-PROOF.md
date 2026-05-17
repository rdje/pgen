# SV-EXH-PROOF: Main-SystemVerilog formally-exhaustive machine-checkable closure proof

## Metadata

- Tree ID: `SV-EXH-PROOF`
- Status: `active`
- Roadmap lane: parser-family exhaustive-proof normalization (the last open parser-family proof debt)
- Created: `2026-05-17`
- Last updated: `2026-05-17`
- Owner: repo-local workflow

## Goal

Re-earn `Done` for the `systemverilog` main-parser family
(`LIVE_ACHIEVEMENT_STATUS.md` rows "`systemverilog` main parser
(Phase P)" + "Parser-family exhaustive proof normalization") by adding
a **formally exhaustive, machine-checkable closure surface for
`grammars/systemverilog.ebnf` itself** — not bounded thresholds,
corpus slices, non-increasing target debt, or curated suites. Mirror
the proven pattern that crossed `systemverilog_preprocessor` from
Mostly-Done → Done: the `*_reachability_closure_gate` +
`*_syntax_closure_gate` + `*_zero_plausible_gap_proof_gate` trio +
their checked-in contracts, tied together by the
`*_formal_exhaustive_closure_gate` umbrella and surfaced through
`sv_parser_family_status_gate` / `sota_exit_gate`.

## Non-Goals

- Not changing parser/grammar acceptance behavior except where a real
  (non-benign) reachability gap is proven and must be closed.
- Not the broader Phase-S build-out (Liberty/SDC crates, rtl_frontend
  parity) — those are separate roadmap workstreams.
- Not re-deriving the proof methodology — it is the established
  sv_preprocessor pattern (the exemplar gates/contracts exist).

## Acceptance Criteria

- `sv_reachability_closure_gate.sh` + `sv_zero_plausible_gap_proof_gate.sh`
  exist (SV analogs of the sv_preprocessor gates) with checked-in
  `systemverilog_*_closure_contract.json` / `_zero_plausible_gap_proof_contract.json`.
- Every syntax-unreachable rule in `grammars/systemverilog.ebnf` is
  classified benign-allowed-pocket (enumerated in the contract, like
  preprocessor's `trivia`) or a real gap that is then closed.
- `sv_formal_exhaustive_closure_gate.sh` requires the zero-gap proof;
  `sv_parser_family_status_gate` + `sota_exit_gate` surface the SV
  exhaustive-closure boolean; the main-SV LIVE rows flip to `Done`
  with the machine-checkable proof surface as evidence.
- All new gates deterministic + green; no regression to existing SV
  aggregate/roundtrip/stimuli gates; full COMMIT.md lockstep per leaf.

## Task Tree

- ID: `SV-EXH-PROOF`
  Status: `active`
  Goal: `Formally-exhaustive machine-checkable closure proof for grammars/systemverilog.ebnf; re-earn Done for the SV main-parser family.`
  Children: `SV-EXH-PROOF.1`, `SV-EXH-PROOF.2`, `SV-EXH-PROOF.3`, `SV-EXH-PROOF.4`

- ID: `SV-EXH-PROOF.1`
  Status: `pending`
  Goal: `Build sv_reachability_closure_gate.sh (SV analog of sv_preprocessor_reachability_closure_gate.sh) + its checked-in contract; emit the deterministic SV reachability sidecar (summary.{covered,reachable}_{rules,branches}); quantify the current SV reachability gap.`
  Acceptance: `Gate exists, Makefile-wired, deterministic; SV reachability sidecar emitted with covered/reachable rule+branch counts; current gap quantified + recorded; no regression to existing SV gates.`
  Verification: `pending`
  Commit: `pending`

- ID: `SV-EXH-PROOF.2`
  Status: `pending`
  Goal: `Syntax-closure contract for SV + classify the syntax-unreachable SV rule set: wire systemverilog_syntax_closure_contract.json through the shared sv_syntax_closure_gate.sh; enumerate every syntax-unreachable rule in grammars/systemverilog.ebnf; classify each benign-allowed-pocket vs real-reachability-gap with rationale.`
  Acceptance: `Full syntax-unreachable SV rule set enumerated + each classified with rationale; benign pockets captured in an allowed_unreachable_rules contract; real gaps listed for .3.`
  Verification: `pending`
  Commit: `pending`

- ID: `SV-EXH-PROOF.3`
  Status: `pending`
  Goal: `Build sv_zero_plausible_gap_proof_gate.sh + systemverilog_zero_plausible_gap_proof_contract.json (SV analogs) proving the only syntax-unreachable surface is the allowed benign pockets, with syntax-closure + aggregate + reachability sidecars green; close any real gap surfaced by .2 (each such grammar fix is its own sub-leaf under the Code-Change Doctrine).`
  Acceptance: `sv_zero_plausible_gap_proof_gate passes; mirrors the sv_preprocessor zero-gap proof structure; any real gap closed + probe-verified + lockstepped.`
  Verification: `pending`
  Commit: `pending`

- ID: `SV-EXH-PROOF.4`
  Status: `pending`
  Goal: `Wire the zero-gap proof into sv_formal_exhaustive_closure_gate.sh (umbrella) + sv_parser_family_status_gate + sota_exit_gate telemetry; flip the main-SV LIVE rows Mostly-Done -> Done with the machine-checkable proof surface as evidence; full closeout.`
  Acceptance: `Umbrella requires the zero-gap proof; family-status gate green; sota_exit_gate parity; LIVE "systemverilog main parser" + "Parser-family exhaustive proof normalization" rows flip to Done with evidence; tree closed.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `SV-EXH-PROOF.1` | `pending` | Reachability-closure gate + sidecar is the foundation the syntax-closure classification (`.2`), the zero-gap proof (`.3`), and the umbrella/family-status flip (`.4`) all build on. |
| 2 | `SV-EXH-PROOF.2` | `pending` | Classify the syntax-unreachable SV rule set (needs `.1`'s reachability sidecar). |
| 3 | `SV-EXH-PROOF.3` | `pending` | Zero-plausible-gap proof gate + contract (needs `.2`'s classification). |
| 4 | `SV-EXH-PROOF.4` | `pending` | Umbrella + family-status + telemetry wiring + LIVE `Done` flip (needs `.3`). |

## Decisions

- `2026-05-17`: User selected this workstream (the largest open
  parser-family debt) from the post-POST-SV-AUDIT strategic fork.
- `2026-05-17`: Methodology is **the proven sv_preprocessor pattern**
  — not re-derived. Exemplars: `rust/scripts/sv_preprocessor_{reachability_closure,syntax_closure,zero_plausible_gap_proof,formal_exhaustive_closure}_gate.sh`
  + `rust/test_data/grammar_quality/systemverilog_preprocessor_{syntax_closure,zero_plausible_gap_proof_,formal_exhaustive_closure_}*contract.json`.
- `2026-05-17`: Empirical scope confirmed — SV already HAS the shared
  `sv_syntax_closure_gate.sh`, the `sv_formal_exhaustive_closure_gate.sh`
  umbrella (currently status-computation only), `sv_parser_family_status_gate.sh`,
  and `systemverilog_formal_exhaustive_closure_contract.json`. SV is
  CONFIRMED MISSING `sv_reachability_closure_gate.sh`,
  `sv_zero_plausible_gap_proof_gate.sh`, and their contracts (+ a
  `systemverilog_syntax_closure_contract.json` and the family-status
  `Done` flip). Those missing pieces ARE the workstream.
- `2026-05-17`: **Code-Change Doctrine compliance** — every gate
  script / contract json / any `grammars/systemverilog.ebnf` fix in
  this tree is task-tree-owned by its leaf (or a sub-leaf split from
  it for a real grammar gap). Gate scripts + contracts are code/proof
  surface → leaf-owned; no out-of-tree code changes.

## Open Questions

- How large is the real (non-benign) SV reachability gap vs benign
  pockets? Resolved empirically in `.1`/`.2` (the LIVE row hints at a
  substantial reachable surface — `reachable rules 46 → 697`). Does
  not block `.1` (building the gate + measuring is the first step).

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-05-17` | `SV-EXH-PROOF` (setup) | task-tree decomposition vs workflow splitting rules; exemplar-pattern + missing-pieces empirical scope; Code-Change-Doctrine precursor compliance | `pass — tree created from the proven sv_preprocessor exemplar; 4 well-defined leaves (reachability gate → syntax-closure classify → zero-gap proof → umbrella/family-status/LIVE flip); confirmed-missing pieces identified` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `SV-EXH-PROOF` (setup) | `PGEN-SV-EXH-PROOF-0000` | tree created + activated; frontier `.1`; sv_preprocessor exemplar + confirmed-missing-pieces scope recorded |

## Changelog

- `2026-05-17`: Created + activated the task tree (user-selected from
  the strategic fork after POST-SV-AUDIT/TaskList #49 closed).
  Decomposed via the proven sv_preprocessor exhaustive-closure pattern
  into `.1` reachability-closure gate → `.2` syntax-unreachable
  classification → `.3` zero-plausible-gap proof gate/contract → `.4`
  umbrella + family-status + telemetry wiring + LIVE `Done` flip.
  Frontier `.1`. Code-Change-Doctrine-compliant (tree-owned).
