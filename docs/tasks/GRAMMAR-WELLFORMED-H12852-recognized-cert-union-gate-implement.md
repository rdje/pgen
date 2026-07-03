# GRAMMAR-WELLFORMED.H.12.8.5.2 — IMPLEMENT `sv_cert_recognized_union_gate` + recognition lockstep

IMPLEMENT the `.8.5.1`-designed (`-0144`) **recognized SV cert-union accounting basis** as a
re-runnable, deterministic, regression-locked oracle — turning the live-tracker prose figure
("the sound multi-config certificate-coverage union is SV's recognized `fully_certified`-accounting
basis") into a mechanically-checked gate, plus the recognition lockstep across the closure model,
the SV contract, and the continuity docs.

> Slice `PGEN-GRAMMAR-WELLFORMED-0147` (**CODE** — new gate script + tracked contract JSON +
> Make target + CI workflow-parity YAML; the gate ADDS a proof surface and changes NO
> grammar/parser/codegen/generated/engine behavior). Status: `done`.
> Parent: `GRAMMAR-WELLFORMED.H.12.8.5` (recognize the SV multi-config cert-union as the
> `fully_certified`-accounting basis). Design + ownership: the `.8.5.1` design doc
> [GRAMMAR-WELLFORMED-H1285-recognized-cert-union-basis-design.md](GRAMMAR-WELLFORMED-H1285-recognized-cert-union-basis-design.md).
> Disciplines: [[feedback_systematically_use_debug_toolbox]],
> [[feedback_no_codebase_change_without_tool_backed_facts]] (the contract pins TOOL-VERIFIED
> current numbers, not the design's pre-`.8.3.1` numbers),
> [[feedback_corpus_expected_from_spec_not_fix]] (the expected values come from the live cert
> oracle, re-run independently — they are not a self-authored mirror),
> [[feedback_ast_pipeline_parser_agnostic]] (the gate is driven entirely by the CLI
> `--cert-union-config` list — no grammar names are baked into engine code).

## Tool-verified current state (the WHY + the pinned numbers) — toolbox-first

The `.8.5.1` design contract was written at `union UNKNOWN=3` / a 3-rule residual. That is **stale**:
`.8.3.1` (`-0146`, committed AFTER `-0144`) closed two of those three cousins. So this IMPLEMENT
re-ran the oracle this session (DEBUG `ast_pipeline`, `--features "generated_parsers ebnf_dual_run"`,
`count 40`, the complete 4-config union) and pins the **current** numbers, byte-identical across
seeds 0/7/42:

```
CERTIFICATE-COVERAGE:       total=1304 proof=1 witness=1283 UNKNOWN=20 spf=0
CERTIFICATE-COVERAGE-UNION: total=1304 proof=1 witness=1302 UNKNOWN=1  spf=0
  UNION UNKNOWN rules (1 of 1 shown): ["context_member_method_call"]
```

- canonical `(systemverilog_file, sv_2017)` ⇒ `UNKNOWN=20` (the `-0146`/`H.12.8.3.1` baseline).
- recognized 4-config union ⇒ `witness=1302 UNKNOWN=1`; the residual is **exactly** the one
  remaining `.8.3` reach-gap (`context_member_method_call`, the store-gated declaration-hosting
  carrier blocked on `STORE-AWARE-GEN.4b`) — sound, not false-witness inflation.
- determinism (seeds 0/7/42): **CONFIRMED** byte-identical (canonical `witness=1283 UNKNOWN=20`,
  union `witness=1302 UNKNOWN=1`). The figure is signal, lockable.

The recognition gap (the WHERE, re-confirmed from `.8.5.1`): the recognized basis exists only as
live-tracker prose; `grep -rl report-certificate-coverage rust/scripts/` was **empty** (no
cert-coverage regression gate existed); the SV `Done` verdict uses 7 closure criteria, none
cert-coverage (`sv_parser_family_status_gate.sh`). Per `DOCTRINE_ENFORCEMENT.md` (a doctrine that
is not mechanically checked is a suggestion; the oracle re-run is the un-fakeable leg), the
recognized basis must become a re-runnable gate.

## What this slice landed

1. **Tracked contract** `rust/test_data/grammar_quality/systemverilog_recognized_cert_union_contract.json`
   (`version 1`) — pins `base_entry`/`base_profile`/`samples`/`seeds`/`union_configs`, the canonical
   headline (`total=1304 proof=1 witness=1283 UNKNOWN=20`), the union headline
   (`total=1304 proof=1 witness=1302 UNKNOWN=1`), the union residual rule set
   (`["context_member_method_call"]`), and the `done_rule`.
2. **Gate script** `rust/scripts/sv_cert_recognized_union_gate.sh` — modeled on the established
   formal-closure-gate shape (`sv_formal_exhaustive_closure_gate.sh`): validate the contract with
   `jq`, ensure the generated SV parser (`make focus_systemverilog`), build the DEBUG `ast_pipeline`
   (`generated_parsers ebnf_dual_run`), run the `--report-certificate-coverage` + 4× `--cert-union-config`
   invocation **for each declared seed**, parse the `CERTIFICATE-COVERAGE:` + `CERTIFICATE-COVERAGE-UNION:`
   lines + the `UNION UNKNOWN rules` set, assert every field equals the contract (the residual set
   order-insensitively) **and** that all seeds agree (determinism) **and** `spf=0`, emit
   `summary.txt` + `summary.json`, exit nonzero on any drift.
3. **Make target** `sv_cert_recognized_union_gate` (`rust/Makefile`, + help text).
4. **CI workflow-parity** `.github/workflows/sv-cert-recognized-union-gate.yml` (`workflow_dispatch`,
   manual-only — same posture as the other per-gate workflows while hosted Actions are paused).
5. **Recognition lockstep**: book `grammar-wellformedness.md` "recognized SV cert basis" note; SV
   integration contract; `LIVE_ACHIEVEMENT_STATUS.md` / `CHANGES.md` / `DEVELOPMENT_NOTES.md` /
   `MEMORY.md` point the recognized-basis claim at the new oracle.

## Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — recognized basis was prose-only, not a gate
  (`grep -rl report-certificate-coverage rust/scripts/` empty pre-slice); recognized 4-config union
  cert reproduced this session (canonical `UNKNOWN=20`, union `witness=1302 UNKNOWN=1 spf=0`,
  residual `["context_member_method_call"]`), byte-identical across seeds 0/7/42.
- [x] **ROOT CAUSE (WHY + WHERE)** — no cert-coverage regression gate existed; the recognized basis
  was un-checked prose, so it could silently drift and the `1 → 0` flip (when `.8.3.2` lands) would
  be ungated. `DOCTRINE_ENFORCEMENT.md`: the oracle re-run is the un-fakeable leg.
- [x] **FIX** — a standalone deterministic `sv_cert_recognized_union_gate` (script + contract JSON +
  Make + CI YAML) modeled on `sv_formal_exhaustive_closure_gate.sh`; recognition lockstep. No
  grammar/parser/codegen/generated/engine change (the gate is a proof surface only).
- [x] **ADDRESSED (verified)** — `make -C rust SHELL=/bin/bash sv_cert_recognized_union_gate`
  exits 0 ("✅ …passed"), `summary.json` records canonical `UNKNOWN=20` + union `UNKNOWN=1` +
  residual `["context_member_method_call"]`, determinism across seeds 0/7/42; an injected-drift
  dry-run (mutated contract) exits nonzero (oracle is real, not a presence check).
- [x] **NO REGRESSION** — additive proof surface only: no engine/grammar/generated change, so the 6
  fully-certified grammars are untouched by construction; the cert numbers are read-only
  measurements. The new gate is deterministic at seeds 0/7/42.
- [x] **LOCKSTEP** — book `grammar-wellformedness.md` recognized-basis note; SV contract;
  `LIVE_ACHIEVEMENT_STATUS.md` / `CHANGES.md` / `DEVELOPMENT_NOTES.md` / `MEMORY.md`; this leaf +
  the `H.12.8.5` frontier row.

## Owns / follow-ups
- `GRAMMAR-WELLFORMED.H.12.8.5.3` — optional 8th `recognized_cert_union_unknown_zero` SV
  family-status criterion (design choice B, deferred until `.8.3.2` is in sight).
- The `1 → 0` union flip is owned by `GRAMMAR-WELLFORMED.H.12.8.3.2` (close
  `context_member_method_call`, blocked on `STORE-AWARE-GEN.4b`); when it lands, this gate's
  contract is re-baselined to `expected_union_unknown=0` + empty residual in the same slice.
- `GRAMMAR-WELLFORMED.H.12.8.5.4` — stage-log pruning port (see below).

## Leaf `GRAMMAR-WELLFORMED.H.12.8.5.4` — port `prune_log` to the cert gates' stage-log seam

- ID: `GRAMMAR-WELLFORMED.H.12.8.5.4`
  Status: `done` (2026-07-03, `PGEN-GRAMMAR-WELLFORMED-0148`)
  Goal: `Port the VERILOG-2005-PROFILE.4.3 prune_log idiom (10 MB threshold → 2000-line tail, success AND failure paths) from verilog_2005_conformance_gate.sh into the stage-log helpers of sv_cert_recognized_union_gate.sh (whose ensure_generated_systemverilog_parser stage log is ~5 GB per run — hand-deleted twice, sessions #22/#23) and rtl_const_expr_cert_gate.sh (whose ensure stage log measured 40 MB this session). Disk space is a critical resource; a gate must not leave multi-GB scratch per run. Per-seed cert logs (the parsed evidence artifacts, ~4 KB) deliberately untouched.`

### Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — measured this session: `rust/target/rtl_const_expr_cert_gate/logs/ensure_generated_rtl_const_expr_artifacts.log` = **40 MB** after one gate run (du); the union gate's `ensure_generated_systemverilog_parser.log` ~**5 GB** per run is twice-documented (`VERILOG-2005-PROFILE.md:1514/1583` + MEMORY session notes; absent from the current logs dir only because it was hand-deleted last session). Host disk 84% used, 77 GB free (df).
- [x] **ROOT CAUSE (WHY + WHERE)** — the two gates' `run_logged`/`run_logged_rust` helpers redirect a stage's FULL stdout+stderr to a per-stage log with no size bound (`sv_cert_recognized_union_gate.sh:59-85`, `rtl_const_expr_cert_gate.sh:59-73`); `make focus_systemverilog` emits ~5 GB of generation debug output. The `.4.3`-landed `prune_log` (`verilog_2005_conformance_gate.sh:59-70`) exists but was never ported when the newer gates were modeled on the older un-pruned script.
- [x] **FIX** — proof-surface script tier: identical `PRUNE_THRESHOLD_BYTES=10485760` + `prune_log` helper (gate-named prune banner) in both scripts, invoked on BOTH success and failure paths of every stage-log helper. Per-seed cert logs untouched (parsed evidence; ~4 KB).
- [x] **ADDRESSED (verified)** — fresh end-to-end re-runs of BOTH gates: `rtl_const_expr_cert_gate` GREEN with the ensure stage log 40 MB → pruned tail (banner + last 2000 lines); `sv_cert_recognized_union_gate` GREEN (canonical `1341/2/1319/20`, union `1338/1`, residual exact, seeds 0/7/42) with the ~5 GB `ensure_generated_systemverilog_parser.log` → pruned tail; logs dir total now MBs, not GBs.
- [x] **NO REGRESSION** — both gates GREEN on the same contracts (the pins are the oracle; pruning happens strictly AFTER a stage completes/fails and never touches the grep'd per-seed cert logs); no engine/grammar/generated change ⇒ parsers byte-identical by construction; `bash -n` clean on both scripts.
- [x] **LOCKSTEP** — this leaf + the `H.12.8.5` frontier row in `GRAMMAR-WELLFORMED.md`; `VERILOG-2005-PROFILE.md` open-note satisfied (`.5` NOTE + `.6.x` recurrences); CHANGES / MEMORY. Book N/A (gate-internal scratch-log hygiene; no user-facing behavior/command change — same posture as the `.4.3` landing).
