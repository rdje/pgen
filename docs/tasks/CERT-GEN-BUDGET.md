# CERT-GEN-BUDGET: certificate-coverage diverse-pass generation is time-unbounded

## Metadata

- Tree ID: `CERT-GEN-BUDGET`
- Status: `active`
- Roadmap lane: `Stimuli-generator / proof-tooling robustness (certificate-coverage)`
- Created: `2026-06-14`
- Last updated: `2026-06-25`
- Owner: repo-local workflow

## Goal

Make the `ast_pipeline --report-certificate-coverage` PASS-1 *diverse*
generation deterministically bounded so it cannot run unboundedly long (or
effectively hang) on deeply-recursive grammars, WITHOUT changing the
byte-identical certification results (`total` / `witness` / `UNKNOWN` /
`sample_parse_failures` / `fully_certified`) for the grammars whose diverse
pass already terminates fast (json / regex / systemverilog / vhdl /
systemverilog_preprocessor / rtl_frontend). Root-cause is `.1` (done); the
bounded-budget fix is `.2`.

## Non-Goals

- Not a shipped-parser change — this is a generation-side / dev-proof-tool
  robustness defect; no parser regen, no release/schema bump, no ledger row.
- Not a change to the cert NUMBERS for the well-behaved grammars (the diverse
  pass is deliberately kept byte-identical to historical behavior — see
  `main.rs:2370` comment); the fix must preserve that.
- Not the bounded plannable-rule reach pass (`PLANNABLE_REACH_ATTEMPT_TIMEOUT_MS
  = 250`), which is already step-budget-bounded per attempt; the SV `--count 40`
  slowness traced to that pass over 645 `UNKNOWN` rules is a separate
  bounded-but-large cost, recorded below as context, not this tree's defect.

## Acceptance Criteria

- A single rtl_const_expr cert sample at the pathological depths (28, 40) is
  deterministically bounded (errors/discards within budget instead of running
  >200s / non-terminating).
- rtl_const_expr stays `fully_certified` at its canonical `--max-depth 32`
  (the depth at which it already completes), deterministic at seeds 0/7/42.
- json / regex / systemverilog / vhdl / systemverilog_preprocessor / rtl_frontend
  cert headlines stay BYTE-IDENTICAL (decisive A/B vs the pre-fix binary).
- Any code change owns a leaf; committed through `COMMIT.md`.

## Task Tree

- ID: `CERT-GEN-BUDGET`
  Status: `active`
  Goal: `bound the cert-coverage diverse-pass generation time deterministically without changing well-behaved grammars' cert results.`
  Children: `CERT-GEN-BUDGET.1`, `CERT-GEN-BUDGET.2`

- ID: `CERT-GEN-BUDGET.1`
  Status: `done`
  Goal: `INVESTIGATION (pure docs, tools-first): root-cause WHY+WHERE the canonical rtl_const_expr cert-coverage run ("--max-depth 32 --entry-rule conditional_expr --count 40") ran ~6.5 min before being killed, and whether it is a hang, a deadlock, or pathological slowness. Characterize the depth/count scaling. Adjudicate severity. Ticket the fix.`
  Acceptance: `WHY+WHERE pinned with decisive tool evidence; hang-vs-slow adjudicated; severity stated; fix ticketed as .2. No code changed.`
  Verification: `2026-06-14 — INVESTIGATION COMPLETE (tools-first, prebuilt debug ast_pipeline --features generated_parsers; the RTL-FE-CLOSURE.6 source edits were NOT yet built, so this used the clean pre-change binary). MEASURED (rtl_const_expr.json --report-certificate-coverage --entry-rule conditional_expr):
  • count=5 depth∈{4,8,12,16}: ALL error instantly (<1s, exit 1) "Stimuli generation depth exceeded max_depth=N while expanding rule 'bit_and_expr'/'relational_expr'" — the operator-precedence chain (conditional_expr → … → primary_expr → ( conditional_expr )) is ~14 levels, so depth <~28 cannot reach a leaf. (This also exposed a MISLEADING test-harness label: my earlier "(timeout/none)" baseline rows at depth 12 were INSTANT errors, not 150s hangs.)
  • count=1 DEPTH SWEEP (seed 0), the decisive datum — generation time is NON-MONOTONIC in --max-depth: depth 24 → instant DepthExceeded on 'identifier'; depth 28 → did NOT terminate within 200s for a SINGLE sample (timeout, rc=124, real=200.01s, 0 cert lines); depth 32 → SUCCEEDS in 18.15s/sample, fully_certified total=48 witness=48 UNKNOWN=0 spf=0; depth 40 → >40s (non-terminating within budget). The depth-28 child was observed at 100% CPU (genuine CPU-bound generation, RSS ~21 MB — NOT a memory leak, NOT a deadlock) then reaped at the 200s cap.
  • count=40 depth=32 ≈ 40 × ~10s ≈ the originally-observed ~6.5 min — i.e. the "stuck" run was the canonical invocation being SLOW (finite at depth 32), not deadlocked.
  ROOT CAUSE (WHERE, code-read): the cert-coverage PASS-1 diverse config (rust/src/main.rs:2370) is built `StimuliConfig { seed, enforce_word_boundary_spacing: true, max_depth, ..Default::default() }`. `..Default::default()` leaves `target_generation_timeout_ms = 0` (StimuliConfig default, stimuli_generator.rs:208). `timeout_budget_from_ms(0)` returns `None` (stimuli_generator.rs:1682-1696), so `generate_many` → `generate_from_entry` never sets `active_generation_deadline`; `generation_deadline_exceeded()` then always returns false (stimuli_generator.rs:1697-1707) ⇒ the DETERMINISTIC step-budget introduced by GRAMMAR-WELLFORMED.B1 (PGEN-GRAMMAR-WELLFORMED-0005) — built precisely to bound generation — is NEVER ARMED for the diverse pass. So the diverse pass is TIME-UNBOUNDED, and on a deeply-recursive grammar (rtl_const_expr) per-sample generation is super-linear / effectively non-terminating at certain depths. WHY it was unbounded: deliberate (the main.rs:2370 comment keeps the diverse pass "byte-identical to the historical behaviour for every grammar" so its sample_parse_failures never regresses); the non-terminating consequence on recursive grammars was simply not anticipated. SEVERITY: moderate — generator/proof-tooling robustness, NOT shipped-parser correctness (no parser/AST/release impact); cert results stay correct + deterministic WHEN the pass completes. But a core proof surface can effectively hang on a recursive grammar, which presents as a critical hang and makes rtl_const_expr's canonical cert impractically slow (~6.5 min at count 40) / unrunnable at neighbouring depths. SECONDARY (context, not this defect): the SV `--count 40` >150s cost traces to the BOUNDED plannable-rule reach pass over 645 UNKNOWN rules (each probed ≤4× at PLANNABLE_REACH_ATTEMPT_TIMEOUT_MS=250ms, main.rs:2490/2526) — bounded-per-probe but many probes; SV's diverse pass at default depth 24 terminates fast. FIX ticketed as .2.`
  Commit: `PGEN-CERT-GEN-BUDGET-0001`

- ID: `CERT-GEN-BUDGET.2`
  Status: `done`
  Goal: `FIX (code leaf): arm a deterministic, generous DEFAULT generation step-budget for the cert-coverage PASS-1 diverse pass so per-sample generation cannot run unboundedly — a budget large enough that every grammar whose diverse pass already terminates fast stays BYTE-IDENTICAL (the budget never bites them), but that deterministically cuts off the rtl_const_expr depth-40 pathology. Parser-agnostic; generator/CLI-side only. Implemented option (a): a default target_generation_timeout_ms for the diverse config translated to a deterministic step deadline via the existing B1 machinery.`
  Acceptance: `rtl_const_expr cert at depth 28 AND depth 40 terminates deterministically within the budget (no >200s spin); rtl_const_expr stays fully_certified at depth 32 (seeds 0/7/42); json/regex/systemverilog/vhdl/systemverilog_preprocessor/rtl_frontend cert headlines byte-identical (decisive A/B); deterministic across reruns; clippy strict-source clean; mdbook/cross-family gates green; book updated if the cert-coverage budget behavior is user-documented.`
  Verification: `2026-06-25 — DONE (PGEN-CERT-GEN-BUDGET-0002). Implementation (parser-agnostic, generator-only): new pub fn StimuliGenerator::generate_many_bounded(count, entry, timeout_ms) arms a per-sample B1 step-budget via generate_from_entry_with_optional_timeout (timeout_ms=0 ≡ generate_many byte-for-byte); the cert PASS-1 (main.rs run_certificate_coverage_report) calls it with env-tunable CERT_DIVERSE_GENERATION_TIMEOUT_MS_DEFAULT=4000 ms (4,000,000 steps), overridable via PGEN_CERT_DIVERSE_GENERATION_TIMEOUT_MS. Byte-identity is STRUCTURAL: generation_deadline_exceeded() advances the step counter whether or not a deadline is armed (stimuli_generator.rs:1815), so a never-reached budget yields identical output. Also added pub fn generation_step_count() + a low-verbosity diverse-pass step-count trace.
  CALIBRATION (seed 0, current binary, via the step-count trace): SV count-40 depth-24 = 93,176 steps total (~2.3k/sample); rtl_const_expr count-40 depth-32 = 8,487,959 steps (~212k avg), and at a 2,000,000-step (2000 ms) budget the run is BYTE-IDENTICAL (same 8,487,959 cumulative, fully_certified) ⇒ per-sample max < 2,000,000. Default 4,000,000 = ~2× that proven ceiling, ~19× the avg.
  ADDRESSED (current-binary pathology, UNBOUNDED `=0`): `conditional_expr --max-depth 40/48` and `rtl_const_expr --max-depth 40` HANG (>40s, rc=124). With the default budget: rtl_const_expr depth-40 cut deterministically in 14.96s, conditional_expr depth-48 in 16.59s (both `TargetTimeout … budget=4000ms`). (The 2026-06-14 `.1` depth-28 case was mitigated by intervening generator work and now completes fast; depth 40/48 remained the live hang.)
  NO REGRESSION — decisive A/B (PGEN_CERT_DIVERSE_GENERATION_TIMEOUT_MS=0 legacy vs default 4000, SAME binary, seed 0, count 40), BYTE-IDENTICAL cert headlines: json total=9/witness=9/UNKNOWN=0; regex 198/198/0; vhdl 216/216/0; systemverilog_preprocessor 74/74/0; rtl_frontend total=169 proof=1 witness=168 UNKNOWN=0; systemverilog total=1288 proof=1 witness=1259 UNKNOWN=28 spf=0 (also =28 at seeds 7/42). rtl_const_expr depth-32 fully_certified (total=48 UNKNOWN=0) at seeds 0/7/42. clippy_source_all_targets → ok (0 errors; the no-features source gate caught a cfg-attachment slip mid-implementation — the const had to carry its OWN #[cfg(feature="generated_parsers")] so it did not steal the gate off run_certificate_coverage_report — fixed + re-verified). No grammar/generated/parser/release/schema/ledger change (generated/*.rs untouched ⇒ external-corpus / parse gates unaffected by construction).`
  Commit: `PGEN-CERT-GEN-BUDGET-0002`

- ID: `CERT-GEN-BUDGET.3`
  Status: `done` (2026-07-02, session #20, `PGEN-CERT-GEN-BUDGET-0003` — ADJUDICATED: **no
  defect, no drift, no red surface; a record-WORDING error**). INVESTIGATION leaf, tools-first,
  zero code. The suspected regression dissolved under systematic elimination:
  - **Code/artifact stability PROVEN:** git-stash A/B (pre-`.6.3.2` binary — identical
    failure); in-place source replay at the exact `.2` base `019eb9ca` (identical failure);
    `generated/rtl_const_expr.json` regenerated + byte-identical; grammar untouched in the
    window; `git log 019eb9ca..HEAD -- rust/src` = only two commits, both exonerated;
    Cargo.lock/toolchain unchanged (rustc 1.95.0, 2026-04-14).
  - **The reconciliation (decisive):** the DEFAULT-entry lane
    (`ast_pipeline generated/rtl_const_expr.json --report-certificate-coverage --count 40
    --max-depth 32`, default budget) reports `total=48 witness=48 UNKNOWN=0
    fully_certified=true` at seeds 0/7/42, with seed-0
    `cumulative_generation_steps=8,487,959` — **byte-for-byte the `.2` calibration number**.
    The `.1`/`.2` records' prose calling `--entry-rule conditional_expr` "the canonical run"
    is the defect: that is the `.1` pathology-PROBE config. On that sub-entry config the run
    needs ~150M steps (measured unbounded) — the 4M budget cutting it is the DESIGNED `.2`
    behavior — and even unbounded it reports `UNKNOWN=1` = `rtl_const_expr` (the grammar's
    ROOT rule, structurally unreachable FROM a sub-entry — an entry-relative artifact, not a
    coverage gap).
  - **Verdict:** the fully-certified-6 roster is INTACT (rce green on its canonical
    default-entry config, deterministic seeds 0/7/42; the other five green at HEAD). Corrected
    the record wording here; the `.6.3.2`-commit docs that flagged a "RED proof surface" are
    superseded by this adjudication (same-day correction).
  - **Follow-up spawned:** `.4` (pending) — the rce fully-certified roster claim is UN-GATED
    (no standing gate pins its cert; only the parser-book gate references rce). Pin the
    canonical default-entry cert (`48/0`, seeds 0/7/42, depth 32) in a repo-standard gate so
    invocation-wording confusion like this fails mechanically instead of narratively.

- ID: `CERT-GEN-BUDGET.4`
  Status: `pending`
  Goal: `GATE leaf: pin the rtl_const_expr canonical certificate-coverage baseline (default entry, --max-depth 32, --count 40, seeds 0/7/42, expected 48/0 fully_certified, default diverse budget) in a repo-standard machine-checkable gate (extend an existing rce gate or add rce_cert_gate), so the fully-certified-6 roster membership for rce is oracle-locked rather than doc-asserted. Cheap (~30 s per seed).`

## Acceptance Checklist (enforced)
- [x] **REPRODUCE / ISSUE** — `ast_pipeline --report-certificate-coverage` PASS-1 diverse pass is time-unbounded: unbounded (`PGEN_CERT_DIVERSE_GENERATION_TIMEOUT_MS=0`) `rtl_const_expr --max-depth 40` and `conditional_expr --max-depth 40/48` HANG (>40s wall, rc=124) on the current binary.
- [x] **ROOT CAUSE (WHY + WHERE)** — diverse config leaves `target_generation_timeout_ms=0` (`stimuli_generator.rs:217`) → `timeout_budget_from_ms(0)=None` (`:1795/1799`) → `generation_deadline_exceeded()` false when unarmed (`:1810`); `generate_many` calls `generate_from_entry` directly, so the B1 step-budget (`enforce_generation_deadline`, woven through the core generation recursion `:8141/8175/9602/9149/…`) is NEVER ARMED for the diverse pass.
- [x] **FIX** — generator-only, parser-agnostic; fix-hierarchy = engine/proof-tool budget (no grammar/declarative surface applies). `generate_many_bounded` arms a per-sample B1 budget; cert PASS-1 default `CERT_DIVERSE_GENERATION_TIMEOUT_MS_DEFAULT=4_000` ms; both new items `#[cfg(feature="generated_parsers")]`-gated.
- [x] **ADDRESSED (verified)** — depth-40 `>40s/hang → 14.96s` deterministic cut; depth-48 → 16.59s; rtl_const_expr `fully_certified` (48/0) at depth 32 seeds 0/7/42.
- [x] **NO REGRESSION** — decisive A/B byte-identical cert for the 6 well-behaved grammars + SV (UNKNOWN=28 seeds 0/7/42, spf=0); rtl_const_expr fully_certified seeds 0/7/42; `clippy_source_all_targets → ok`; `ast_shape_contract` lib tests 9 passed/0 failed (generated parsers untouched ⇒ unaffected by construction); generated/*.rs byte-identical (no codegen change).
- [x] **LOCKSTEP** — `TOOLBOX.md` §4.5 + book `diagnosing-unknowns.md` (new env knob `PGEN_CERT_DIVERSE_GENERATION_TIMEOUT_MS`); this leaf + `CERT-GEN-BUDGET.md`; CHANGES / DEVELOPMENT_NOTES / MEMORY / LIVE_ACHIEVEMENT_STATUS. No contract/ledger/schema/release (not a parser change).

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `CERT-GEN-BUDGET.4` | `pending` | Pin the rce canonical default-entry cert (`48/0`, seeds 0/7/42, depth 32) in a machine-checkable gate — the `.3` adjudication showed the roster claim is currently doc-asserted only, which is exactly how the wording confusion survived. |
| — | `CERT-GEN-BUDGET.3` | `done` (`PGEN-CERT-GEN-BUDGET-0003`) | ADJUDICATED — no defect/drift: the `.1`/`.2` records mislabeled the `conditional_expr` sub-entry probe config as "canonical"; the DEFAULT-entry lane reproduces `48/0 fully_certified` at seeds 0/7/42 with the exact `.2` step count (8,487,959 @ seed 0). Sub-entry `UNKNOWN=1` = the root rule, entry-relative artifact; the budget cutting the sub-entry's ~150M-step run is designed behavior. |
| — | `CERT-GEN-BUDGET.2` | `done` (`PGEN-CERT-GEN-BUDGET-0002`) | The bounded-budget fix LANDED: diverse pass now arms a default 4M-step B1 budget; byte-identical cert for the 6 well-behaved grammars + SV; rtl_const_expr fully_certified at depth 32 (seeds 0/7/42, DEFAULT entry); depth-40/48 runaway cut deterministically (~15s). |
| — | `CERT-GEN-BUDGET.1` | `done` (`PGEN-CERT-GEN-BUDGET-0001`) | Root cause: the diverse-pass config never arms the B1 step-budget (`target_generation_timeout_ms=0`) → unbounded generation, pathological/non-terminating on deeply-recursive rtl_const_expr; non-monotonic in `--max-depth`. |

**Tree status: active (was briefly reopened as a suspected regression 2026-07-02; the `.3`
adjudication cleared it same-day — no drift, roster intact). Remaining scope = the `.4` gate
pin.** NOTE for readers of `.1`/`.2`: where those records say "canonical
`--entry-rule conditional_expr`", read "the `.1` pathology-probe config"; the CERTIFICATION
config is the default entry (`rtl_const_expr`).

## Decisions

- `2026-06-14` (`.1`, `PGEN-CERT-GEN-BUDGET-0001`): the director flagged an `ast_pipeline` run "stuck for 6.5 min" as a misbehavior that must be task-tree tracked + investigated (be-alert / root-cause-fishy-immediately discipline). Tools-first root cause: it is NOT a deadlock — the cert-coverage PASS-1 diverse pass is **time-unbounded** because its `StimuliConfig` (`main.rs:2370`) leaves `target_generation_timeout_ms=0`, so the B1 deterministic step-budget is never armed; on the deeply-recursive `rtl_const_expr` grammar per-sample generation is super-linear / non-terminating at certain depths (depth 28/40 a single sample > 200s; depth 32 ~18s; count 40 at depth 32 ≈ 6.5 min). Severity = generator/proof-tooling robustness, not shipped-parser correctness. Fix ticketed as `.2`; it must preserve byte-identical cert results for the 6 grammars whose diverse pass already terminates fast.

## Open Questions

- For `.2`: what default step-budget leaves all 6 well-behaved grammars byte-identical AND lets rtl_const_expr complete at depth 32 while cutting off the depth-28/40 pathology? (Resolve empirically in `.2` via a decisive A/B; rtl_const_expr at depth 32 needs only ~18s/sample, so a budget comfortably above that single-sample step count but below the depth-28 runaway should exist.)

## Blockers

- None.

## Verification Log

| Date | Leaf | Checks | Result |
| --- | --- | --- | --- |
| `2026-06-14` | `CERT-GEN-BUDGET.1` | `rtl_const_expr.json cert depth sweep (count 5: depths 4/8/12/16 instant DepthExceeded; count 1: depth 24 instant error, depth 28 >200s timeout rc=124, depth 32 18.15s fully_certified, depth 40 >40s); ps (depth-28 child 100% CPU → reaped at 200s, RSS 21 MB, RAM 85% free); code read (main.rs:2370 diverse config ..Default::default(); stimuli_generator.rs:208 default target_generation_timeout_ms=0; :1682 timeout_budget_from_ms(0)=None; :1697 deadline-check None⇒false)` | `Root cause = unbounded diverse pass (B1 step-budget never armed); non-monotonic in depth; not a deadlock; severity moderate (tooling robustness). Fix → .2. No code changed.` |
| `2026-06-25` | `CERT-GEN-BUDGET.2` | `Calibration via low-verbosity step-count trace (SV d24 = 93,176 steps; rce d32 = 8,487,959 steps, byte-identical at 2M budget ⇒ per-sample max < 2M). Current-binary pathology sweep (UNBOUNDED =0, timeout 40s): conditional_expr d40/d48 + rtl_const_expr d40 HANG (rc=124); d28 now fast (intervening-work mitigated). Decisive A/B (=0 vs default 4000): json/regex/vhdl/svpp/rtl_frontend/SV byte-identical (SV UNKNOWN=28 spf=0, also =28 @ seeds 7/42); rce d32 fully_certified @ seeds 0/7/42. Pathology cut @ default: rce d40 = 14.96s, conditional_expr d48 = 16.59s (TargetTimeout budget=4000ms). clippy_source_all_targets → ok (0 errors); full clippy gate ✅; ast_shape_contract 18/18.` | `Fix LANDED: generate_many_bounded + default 4M-step diverse-pass budget. Byte-identical for 6 well-behaved + SV; rce fully_certified; depth-40/48 runaway bounded ~15s. Generator-only; no parser/grammar/generated/release change.` |

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `CERT-GEN-BUDGET.1` | `PGEN-CERT-GEN-BUDGET-0001` | Pure-docs investigation; root cause + severity + fix ticket. |
| `CERT-GEN-BUDGET.2` | `PGEN-CERT-GEN-BUDGET-0002` | The bounded-budget FIX (code): `generate_many_bounded` + default 4M-step diverse-pass budget; byte-identical cert for 6 well-behaved grammars + SV; rtl_const_expr fully_certified at depth 32 (seeds 0/7/42); depth-40/48 runaway cut deterministically. |

## Changelog

- `2026-06-14`: Created (`.1` done, `PGEN-CERT-GEN-BUDGET-0001`) — root-caused the "stuck 6.5-min" `ast_pipeline` cert-coverage run to the time-unbounded PASS-1 diverse pass (no B1 step-budget armed; `main.rs:2370`); pathological/non-terminating on deeply-recursive rtl_const_expr; non-monotonic in `--max-depth`. Fix ticketed as `.2` (arm a generous deterministic step-budget, preserving byte-identical cert for the 6 well-behaved grammars). Frontier → `.2`.
- `2026-06-25`: `.2` DONE (`PGEN-CERT-GEN-BUDGET-0002`) — armed a deterministic 4,000,000-step (4000 ms) per-sample default budget on the cert PASS-1 diverse pass via the new `generate_many_bounded` (env-tunable `PGEN_CERT_DIVERSE_GENERATION_TIMEOUT_MS`). Byte-identical cert for the 6 well-behaved grammars + SV (decisive A/B), rtl_const_expr fully_certified at depth 32 (seeds 0/7/42), and the current depth-40/48 runaway cut deterministically (~15s vs unbounded hang). Generator-only / parser-agnostic; no grammar/generated/release/schema/ledger change. **Tree COMPLETE (all leaves done).**
- `2026-07-02`: Tree REOPENED — `.3` spawned (`pending`, discovered during `VERILOG-2005-PROFILE.6.3.2` verification, recorded in commit `PGEN-VERILOG-2005-PROFILE-0015`): the rtl_const_expr canonical cert (depth 32) now FAILS at HEAD on the deterministic 4M-step budget (`TargetTimeout conditional_expr root/o1 budget=4000ms`); git-stash A/B proved it independent of the `.6.3.2` engine fix ⇒ an intervening engine drift since the `.2` calibration. The other five fully-certified grammars remain green at HEAD (seed 0). Frontier → `.3`.
- `2026-07-02` (same day, later): `.3` DONE (`PGEN-CERT-GEN-BUDGET-0003`) — **ADJUDICATED, no defect**: systematic elimination (stash A/B; in-place source replay at the `.2` base commit — identical failure; JSON regen byte-identical; toolchain/lockfile unchanged) proved code+input stability, and the DEFAULT-entry lane then reconciled everything — `48/0 fully_certified` at seeds 0/7/42 with the `.2` calibration's exact step count (8,487,959 @ seed 0). The `.1`/`.2` prose mislabeling the `conditional_expr` sub-entry probe as "canonical" was the whole defect; sub-entry `UNKNOWN=1` is the root rule (entry-relative artifact) and the budget cutting its ~150M-step run is designed behavior. Roster intact. `.4` spawned (gate-pin the rce canonical cert). Frontier → `.4`.
