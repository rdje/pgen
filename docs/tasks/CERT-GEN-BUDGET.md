# CERT-GEN-BUDGET: certificate-coverage diverse-pass generation is time-unbounded

## Metadata

- Tree ID: `CERT-GEN-BUDGET`
- Status: `active`
- Roadmap lane: `Stimuli-generator / proof-tooling robustness (certificate-coverage)`
- Created: `2026-06-14`
- Last updated: `2026-06-14`
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
  Status: `pending`
  Goal: `FIX (code leaf): arm a deterministic, generous DEFAULT generation step-budget for the cert-coverage PASS-1 diverse pass (rust/src/main.rs:2370) so per-sample generation cannot run unboundedly — a budget large enough that every grammar whose diverse pass already terminates fast stays BYTE-IDENTICAL (the budget never bites them), but that deterministically cuts off the rtl_const_expr depth-28/40 pathology (it becomes a bounded DepthExceeded/step-budget discard instead of a >200s spin). Parser-agnostic; generator/CLI-side only. Options to weigh in the fix: (a) a default target_generation_timeout_ms for the diverse config translated to a deterministic step deadline via the existing B1 machinery (must verify the step budget is large enough to leave json/regex/SV/vhdl/svpp/rtl_frontend byte-identical AND rtl_const_expr still fully_certified at depth 32 in <~20s/sample); (b) a per-sample step ceiling independent of wall clock. MUST prove byte-identical cert headlines for the 6 well-behaved grammars (decisive git-stash A/B) and rtl_const_expr still fully_certified at depth 32.`
  Acceptance: `rtl_const_expr cert at depth 28 AND depth 40 terminates deterministically within the budget (no >200s spin); rtl_const_expr stays fully_certified at depth 32 (seeds 0/7/42); json/regex/systemverilog/vhdl/systemverilog_preprocessor/rtl_frontend cert headlines byte-identical (decisive A/B); deterministic across reruns; clippy strict-source clean; mdbook/cross-family gates green; book updated if the cert-coverage budget behavior is user-documented.`
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `CERT-GEN-BUDGET.2` | `pending` | The bounded-budget fix. Root cause pinned in `.1`; the fix must preserve byte-identical cert for the 6 well-behaved grammars. |
| — | `CERT-GEN-BUDGET.1` | `done` (`PGEN-CERT-GEN-BUDGET-0001`) | Root cause: the diverse-pass config never arms the B1 step-budget (`target_generation_timeout_ms=0`) → unbounded generation, pathological/non-terminating on deeply-recursive rtl_const_expr; non-monotonic in `--max-depth`. |

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

## Commit Log

| Leaf | Commit subject or reference | Notes |
| --- | --- | --- |
| `CERT-GEN-BUDGET.1` | `PGEN-CERT-GEN-BUDGET-0001` | Pure-docs investigation; root cause + severity + fix ticket. |

## Changelog

- `2026-06-14`: Created (`.1` done, `PGEN-CERT-GEN-BUDGET-0001`) — root-caused the "stuck 6.5-min" `ast_pipeline` cert-coverage run to the time-unbounded PASS-1 diverse pass (no B1 step-budget armed; `main.rs:2370`); pathological/non-terminating on deeply-recursive rtl_const_expr; non-monotonic in `--max-depth`. Fix ticketed as `.2` (arm a generous deterministic step-budget, preserving byte-identical cert for the 6 well-behaved grammars). Frontier → `.2`.
