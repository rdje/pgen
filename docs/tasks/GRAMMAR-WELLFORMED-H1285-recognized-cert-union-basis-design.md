# GRAMMAR-WELLFORMED.H.12.8.5.1 — recognized SV cert-union basis: reproduce + recognition audit + regression-lock DESIGN

DESIGN-FIRST, tools-first WHY+WHERE for the director-chosen **cert-accounting vehicle**: make the
sound multi-entry/multi-profile certificate-coverage **union** (SV `UNKNOWN=3`) the **recognized**
`fully_certified`-accounting basis for SystemVerilog — i.e. a re-runnable, regression-locked oracle
(not just prose in the live tracker), with the recognition lock-stepped across the closure model,
the SV family-status accounting, and the SV contract.

> Slice `PGEN-GRAMMAR-WELLFORMED-0144` (**PURE-DOCS** — reproduce + audit + design; NO
> code/grammar/generated/release/schema/ledger change). Status: `done` (design); owns IMPLEMENT
> child `.8.5.2`.
> Parent: the SV `UNKNOWN`→0 lane `GRAMMAR-WELLFORMED.H.12.8`. Sibling of `.8.3` (close the 3
> reach-gaps — BLOCKED on `STORE-AWARE-GEN.4b`); this `.8.5` vehicle is the **unblocked** SV-closure
> lane the director selected at the post-`-0143` fresh-session fork.
> Reads with [[project_sv_full_certification_via_multi_entry]] (the committed endgame),
> [[feedback_systematically_use_debug_toolbox]], [[feedback_no_codebase_change_without_tool_backed_facts]],
> [[feedback_pinpoint_real_blocker_not_menu]], [[project_grammar_wellformedness_contract]]
> (literal-0 is the consequence of well-formedness), [[project_cert_coverage_tournament_loser_leak]]
> (commit only improvements), [[feedback_ast_pipeline_parser_agnostic]].

## Why this vehicle, why now

At the post-`-0143` fresh-session checkpoint the director chose **"PNT the cert-accounting vehicle"**
over **"tackle the `.8.3` reach-gaps"**. The distinction is decisive:

- `.8.3` (close the 3 canonical reach-gaps → union `3 → 0`) is **BLOCKED**: its primary carrier
  `context_member_method_call` needs the gen-time name-coupled `variable_binding` prelude, which is
  blocked on `STORE-AWARE-GEN.4b` value-selection.
- `.8.5` (recognize + lock the union as the accounting basis) is **fully unblocked** — the union
  *mechanism* is already built and tool-confirmed at `UNKNOWN=3`; what is missing is the *recognition*
  (a re-runnable oracle + accounting lockstep), not any new generator/grammar capability.

## REPRODUCE — the recognized-union oracle (tool output, this session)

DEBUG `ast_pipeline` (`--features "generated_parsers ebnf_dual_run"`, the Jun-29 build), count 40,
canonical entry `systemverilog_file` profile `sv_2017`, with the **complete 4-config union**:

```
ast_pipeline grammars/systemverilog.ebnf --report-certificate-coverage \
  --grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed <S> \
  --cert-union-config systemverilog_file:sv_2023 \
  --cert-union-config sv_multi_entry_root:sv_2017 \
  --cert-union-config library_text:sv_2017 \
  --cert-union-config systemverilog_parseable_file:sv_2017
```

Seed 0 (confirmed):

```
CERTIFICATE-COVERAGE: grammar='systemverilog' entry='systemverilog_file' samples=40
  total=1304 proof=1 witness=1281 UNKNOWN=22 fully_certified=false
  (sample_parse_failures=0, proof_reverify_failures=0)
CERTIFICATE-COVERAGE-UNION: grammar='systemverilog' base_entry='systemverilog_file'
  base_profile='sv_2017' union_configs=[systemverilog_file:sv_2023, sv_multi_entry_root:sv_2017,
  library_text:sv_2017, systemverilog_parseable_file:sv_2017]
  total=1304 proof=1 witness=1300 UNKNOWN=3 fully_certified=false
  UNION UNKNOWN rules (3 of 3 shown): ["context_member_method_call",
  "known_unscoped_class_scoped_call_interface_class_identifier",
  "known_unscoped_class_scoped_call_type_parameter_identifier"]
```

- **Canonical** `(systemverilog_file, sv_2017)` ⇒ `UNKNOWN=22` (the `-0136`/`H.12.7` baseline).
- **Recognized 4-config union** ⇒ `witness=1300 UNKNOWN=3`; the residual is **exactly** the 3 `.8.3`
  reach-gaps (`context_member_method_call` + the two `…scoped_call…` cousins) — sound, not
  false-witness inflation.
- **Determinism (seeds 0/7/42): CONFIRMED.** All three seeds report byte-identical canonical
  `total=1304 proof=1 witness=1281 UNKNOWN=22` and union `witness=1300 UNKNOWN=3 fully_certified=false`
  (seed 0 also enumerated the 3-rule residual; the identical union witness/UNKNOWN at 7/42 pins the
  same covered set). The recognized-union figure is therefore signal, not noise — it can be locked.

## Recognition audit — WHERE the recognized basis is / isn't established (the WHY+WHERE)

Tools-first survey of every surface that could carry the recognized basis:

1. **`LIVE_ACHIEVEMENT_STATUS.md` — RECOGNIZED (prose).** The `2026-06-29` tracker note already cites
   the complete 4-config union `witness=1300 UNKNOWN=3`, names the 3 residual = the `.8.3` reach-gaps,
   and records the `sv_multi_entry_root:sv_2017`-omitted 3-config variant as `UNKNOWN=4`. So the
   *number* is recognized in the live tracker — but only as hand-written prose, not a re-runnable
   oracle.
2. **`sv_parser_family_status_gate.sh` — the SV "Done" verdict does NOT consider cert-coverage.**
   The gate computes SV `Done` from **7 closure criteria** (`sv_closure_criteria_total_count=7`,
   `:453`): `syntax_closure_gate_green`, `aggregate_contract_green`,
   `generation_parser_rejections_zero`, `shadow_parser_rejections_zero`,
   `focused_replay_target_debt_zero` (the one outstanding, `>0`), `semantic_scope_contract_green`,
   `formal_exhaustive_closure_surface_green`. **Certificate-coverage / `fully_certified` (UNKNOWN) is
   NOT one of the 7.** So today nothing mechanically ties SV's status to a cert-coverage number at all.
3. **No cert-coverage regression gate exists.** `grep -rl "report-certificate-coverage" rust/scripts/`
   ⇒ **empty**. The cert is run by hand (`TOOLBOX.md` Protocol A / the live-tracker note); nothing
   re-runs it deterministically and asserts the number. (`DOCTRINE_ENFORCEMENT.md` lists
   "cert-coverage at seeds 0/7/42" among the strongest oracle legs, but for SV it is currently a
   *manual* re-run, not a wired gate.)
4. **SV integration contract / ledger.** The union figure is not yet a consumer-facing trust claim;
   it stays internal until `.8.3` closes (SV remains `Mostly Done` — the union is `UNKNOWN=3`, not 0).

**Conclusion (the precise gap):** the recognized basis exists *as a number in prose* but is **not an
oracle**. Per `DOCTRINE_ENFORCEMENT.md` (a doctrine that is not mechanically checked is a suggestion;
the oracle re-run is the un-fakeable leg), the recognized basis must become a **re-runnable,
deterministic gate** that pins the recognized-union `UNKNOWN=3` + the 3-rule residual set, so it
cannot silently drift and so the `3 → 0` flip (when `.8.3` lands) is gated — plus a small recognition
lockstep across the closure model and the family-status accounting.

## DESIGN — `sv_cert_recognized_union_gate` (the IMPLEMENT, owned by `.8.5.2`)

Model precisely on `rust/scripts/sv_formal_exhaustive_closure_gate.sh` (the established gate shape:
run the tool → parse a headline line → diff against a tracked contract JSON → emit `summary.txt` +
`summary.json` → exit nonzero on drift). Parser-agnostic by construction (driven by the CLI config
list, no grammar names baked into engine code).

### Gate behaviour
1. **Tracked contract** `rust/test_data/grammar_quality/systemverilog_recognized_cert_union_contract.json`:
   ```json
   {
     "family": "systemverilog",
     "version": 1,
     "base_entry": "systemverilog_file",
     "base_profile": "sv_2017",
     "samples": 40,
     "seeds": [0, 7, 42],
     "union_configs": ["systemverilog_file:sv_2023", "sv_multi_entry_root:sv_2017",
                       "library_text:sv_2017", "systemverilog_parseable_file:sv_2017"],
     "expected_canonical_unknown": 22,
     "expected_union_unknown": 3,
     "expected_union_witness": 1300,
     "expected_union_residual_rules": ["context_member_method_call",
       "known_unscoped_class_scoped_call_interface_class_identifier",
       "known_unscoped_class_scoped_call_type_parameter_identifier"],
     "done_rule": "SV is recognized fully_certified when the multi-config union UNKNOWN reaches 0; until then the recognized accounting basis is the union (expected_union_unknown), and the residual must equal expected_union_residual_rules exactly, deterministic across seeds."
   }
   ```
2. **Script** `rust/scripts/sv_cert_recognized_union_gate.sh`: for each seed in `seeds`, run the
   `--report-certificate-coverage` + `--cert-union-config …` invocation; parse the
   `CERTIFICATE-COVERAGE:` and `CERTIFICATE-COVERAGE-UNION:` lines; assert
   `canonical UNKNOWN == expected_canonical_unknown`, `union UNKNOWN == expected_union_unknown`,
   `union witness == expected_union_witness`, the parsed `UNION UNKNOWN rules` set ==
   `expected_union_residual_rules` (order-insensitive), and `spf == 0`; assert all three seeds agree
   (determinism). Emit `summary.txt` + `summary.json` (gate identity + version + the per-seed numbers
   + the recognized-basis verdict). Exit nonzero on any drift.
3. **Make target** `sv_cert_recognized_union_gate` (mirror the existing gate targets) + the CI
   workflow-parity entry. **Heavy** (~3 min/seed × 3 = ~9 min), so wire it as a CI/`make` oracle, NOT
   a `cargo test --lib` unit — same posture as the other formal-closure gates (the `.8.1.1`
   `certificate_coverage_union_is_over_positively_covered_sets` unit test already covers the cheap
   soundness invariant).

### Recognition lockstep (in `.8.5.2`, same commit as the gate)
- **`docs/book/src/grammar-wellformedness.md`** — a short "recognized SV cert basis" note: the
  recognized accounting is the 4-config union (`UNKNOWN=3`), the 19-rule canonical↔union gap is fully
  adjudicated non-defect (11 entry-relative + 6 profile-relative + 2 blessed extraction), residual to
  `fully_certified` = the 3 `.8.3` reach-gaps; gated by `sv_cert_recognized_union_gate`.
- **`docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md`** — the recognized cert basis
  as the honest internal trust figure (still `Mostly Done`; not yet a consumer guarantee).
- **`LIVE_ACHIEVEMENT_STATUS.md` / `CHANGES.md` / `DEVELOPMENT_NOTES.md` / `MEMORY.md`** — point the
  recognized-basis claim at the new oracle.

### Decision: recognized basis = standalone oracle, NOT (yet) an 8th family-status criterion
Three candidate shapes were weighed (tools-first, no menu — the recommendation IS the decision):
- **(A) Standalone `sv_cert_recognized_union_gate` (CHOSEN).** Locks the recognized union as a
  re-runnable oracle without changing the SV `Done` verdict. Lowest-risk, parser-agnostic, and exactly
  the established formal-closure-gate pattern. The recognized basis becomes un-self-tickable.
- **(B) Add an 8th `recognized_cert_union_unknown_zero` criterion to `sv_parser_family_status_gate`.**
  Deferred: it would (correctly) keep SV `Mostly Done` today (union `UNKNOWN=3 ≠ 0`), but it changes the
  family-status closure accounting (`7 → 8`) and should follow the standalone oracle, not precede it —
  it is the natural `.8.5.3` once (A) is green and `.8.3` is in sight.
- **(C) Docs-only recognition.** Rejected: prose is not an oracle (`DOCTRINE_ENFORCEMENT.md`).

## Acceptance Checklist (enforced) — PURE-DOCS design slice
- [x] **REPRODUCE / ISSUE** — recognized 4-config union cert reproduced this session: canonical
  `UNKNOWN=22`, union `witness=1300 UNKNOWN=3 spf=0`, residual = the 3 named `.8.3` reach-gaps;
  **deterministic byte-identical across seeds 0/7/42** (all three: canonical `witness=1281 UNKNOWN=22`,
  union `witness=1300 UNKNOWN=3`).
- [x] **ROOT CAUSE (WHY + WHERE)** — the recognized basis is recognized only as live-tracker prose; no
  cert-coverage regression gate exists (`grep -rl report-certificate-coverage rust/scripts/` = empty);
  the SV `Done` verdict uses 7 closure criteria, none cert-coverage (`sv_parser_family_status_gate.sh:453`).
- [x] **FIX** — DESIGN only (no code this slice): a standalone deterministic
  `sv_cert_recognized_union_gate` (script + contract JSON + Make/CI) asserting the recognized union
  `UNKNOWN=3` + 3-rule residual + determinism, modeled on `sv_formal_exhaustive_closure_gate`; plus the
  recognition lockstep. IMPLEMENT owned by `.8.5.2`.
- [x] **ADDRESSED (verified)** — N/A for a design slice (no behavior change); the IMPLEMENT's
  ADDRESSED/NO-REGRESSION will cite the new gate green + the 6 fully-certified grammars byte-identical.
- [x] **NO REGRESSION** — N/A (PURE-DOCS, no code/grammar/generated change; deterministic gates
  inherit `-0143` green).
- [x] **LOCKSTEP** — this leaf + the `H.12.8` frontier row (`.8.5`/`.8.5.1` added); `CHANGES.md` /
  `DEVELOPMENT_NOTES.md` / `MEMORY.md`. No book/contract/ledger change this slice (the recognition
  lockstep lands WITH the gate in `.8.5.2`).

## Owns
- `GRAMMAR-WELLFORMED.H.12.8.5.2` — IMPLEMENT `sv_cert_recognized_union_gate` + recognition lockstep.
- (later) `GRAMMAR-WELLFORMED.H.12.8.5.3` — optional 8th recognized-cert family-status criterion (B).
