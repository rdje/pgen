# Doctrine Enforcement Architecture

A portable, **project-agnostic** standard for turning written rules ("doctrines") into
**mechanically enforced** ones — so compliance is *provable and re-checkable*, never a
"trust me" claim. Drop the kit (§8) into any repository and a non-compliant change cannot
land: a local git hook blocks it, and CI makes it un-mergeable.

> **👉 Adopting this in your project? THIS is the only document you need to follow.** Go straight to
> **§8 — The portable replay manifest**: copy 6 files (Group A), adapt a handful of knobs (Group B),
> add your harness's bootstrap pointer (Group C), run the 3 setup commands. Sections 1–7 are the
> rationale + the check-script contract; §9 is the honest limits; §10 is a worked reference instance.

> One-line thesis: **a doctrine that is not mechanically checked is not enforced — it is a
> suggestion.** The fix is to pair every doctrine with a deterministic check, run all checks
> from one registry/driver, and gate commits + CI on it.

This file is the **4th portable architecture** a project adopts, alongside the three it already has:

| # | Portable architecture | Owns | Standard |
|---|---|---|---|
| 1 | **Task-trees** | per-unit work memory (goal/frontier/acceptance/verification) | `docs/TASK_TREE.md` |
| 2 | **Memory-architecture** | durable harness-agnostic agent memory (4 layers) | `MEMORY_ARCHITECTURE.md` |
| 3 | **Knowledge-map** | a retrieval layer over fact cards | `knowledge-map/` |
| 4 | **Doctrine-enforcement** | turning every rule into a mechanically-gated check | **this file** |

All four are **project- and harness-agnostic**: a project backed by Codex, Claude Code, Gemini, or a
human adopts each by replaying its standard. This one is the sibling of `MEMORY_ARCHITECTURE.md` —
that standard mechanizes the *memory* doctrine; this one generalizes the *same E1→E4
defense-in-depth* to **every** doctrine. The enforcement is **git-level** (hooks + CI), so it fires
identically no matter which harness made the commit.

---

## 0. How to use this file

1. Read it once. Adopt the **check-script contract** (§4) and the **driver+registry** (§5).
2. Copy the agnostic kit (§8): the driver, one example check, the hook, the CI step.
3. For each doctrine you want enforced, write a `check_<doctrine>.sh` and register it.
4. Run the three setup commands (§8). From then on, non-compliance fails fast (hook) and cannot
   merge (CI).

If you remember one rule: **route every doctrine to a check, register it, gate on the driver.**

---

## 1. The problem

Most doctrines live as prose (a README section, a decision record, a code comment). Prose is
**discoverable but not enforceable** — an agent or human can read it and still ignore it, and
nothing catches the violation until much later (or never). The two failure modes:

- **"Trust me" compliance** — a change claims it followed the rule; no artifact proves it.
- **Silent drift** — a rule erodes one exception at a time because nothing re-checks it.

The cure is not more prose. It is to make the **compliant path the gated path**: every doctrine
gets a check that *re-derives the truth from the repository*, and the gates run that check.

---

## 2. The core idea

> **doctrine = a rule + a deterministic check that exits nonzero on any breach.**

Once a doctrine has such a check, enforcement is mechanical:

- one **driver** runs every registered check and reports per-doctrine PASS/FAIL (§5);
- the **git hook** runs the driver (fast local gate, E3);
- **CI** runs the *same* driver (un-bypassable backstop, E4).

The check is the single source of truth for the rule; the prose doc explains *why*, the check
decides *whether*.

---

## 3. The three check archetypes (pick one per doctrine)

Every mechanizable doctrine fits one of three shapes. Pick by what makes the proof real.

| Archetype | The check… | Proof strength | Cost / where to run | Example |
|---|---|---|---|---|
| **Structural** | re-derives an invariant from the tree (allowlist match, file presence, lockstep/derived-artifact sync) | a fact about the files — cannot be faked | cheap → pre-commit | "the root-markdown set equals the tracked allowlist"; "the derived map is regenerated + staged" |
| **Oracle (re-run)** | re-EXECUTES a deterministic tool at fixed inputs (fixed seeds / golden inputs) and asserts the result | strongest — a fabricated claim does not reproduce | may be heavy → defer to CI | "certificate-coverage `UNKNOWN=0` at seeds 0/7/42"; "the shape-contract gate is green" |
| **Evidence (artifact)** | requires a re-checkable artifact for an action that cannot be re-derived (e.g. *how* a bug was diagnosed) — pasted tool output in a tracked location, ideally with the cited command re-run | medium → strong (strong when the cited command is re-run) | cheap (presence) / heavy (re-run) | "a code change's task leaf carries a tool-output WHY+WHERE + a measured before→after" |

Rule of thumb: prefer **structural** (cannot be faked) → then **oracle** (re-run beats trust) →
use **evidence** only where the thing being enforced is an *action/process* that leaves no other
re-derivable trace. For evidence checks, make them as oracle-like as possible (re-run the cited
command) so they are not bypassable by pasting fake output.

---

## 4. The check-script contract (precise — this is what makes it portable)

A doctrine check is **any executable** that obeys this contract. Get this right and any project,
any language, can add doctrines that "just work" with the driver.

1. **Exit code is the verdict.** `exit 0` ⟺ the doctrine holds; **any nonzero** ⟺ a breach.
2. **Explain on breach.** On nonzero, print a human-actionable message to **stderr** (what broke,
   where, how to fix). On pass, stay quiet or print one OK line.
3. **Deterministic.** Same repository state → same verdict. No clocks, no network, no randomness
   (or pin the seed). This is what lets the gate be trusted and CI re-run it.
4. **Reads the repository (+ `git`), mutates nothing** (a *derive-and-stage* step — like
   regenerating a derived artifact — is allowed but must be idempotent and explicit).
5. **Scope-aware where relevant.** A check about a *change* should look at the staged set
   (`git diff --cached --name-only`) or an explicit range, and **exempt** changes it does not
   govern (e.g. a code-only doctrine exempts pure-docs commits) — so it never blocks unrelated work.
6. **Self-contained + path-agnostic.** Resolve the repo root from the script's own location
   (`ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"`); reference repo-relative paths only.
7. **Fast, or deferred.** If a check is too slow for pre-commit, keep it in the registry but mark
   it CI-only (run the cheap structural proxy locally, the full oracle in CI).

A check that obeys (1)–(7) is portable: the driver does not care what it checks or how.

---

## 5. The registry + driver (the general enforcer)

One driver owns the list of doctrines and runs them all. The **registry is the source of truth**
for "which doctrines are enforced by what"; a human-readable manifest mirrors it.

- **Registry**: a list of `id | what-it-proves | path/to/check.sh`.
- **Driver**: runs every check (collecting *all* results, not stopping at the first failure),
  prints a per-doctrine report, and exits nonzero iff any failed. It also **meta-checks** that
  every registered check exists and is executable — so a registry entry can never be a dangling
  promise.
- **Adding a doctrine** = write a `check_*.sh` obeying §4 + add one registry line. Nothing else.

This repo ships the reference driver at [`scripts/check_doctrines.sh`](scripts/check_doctrines.sh)
and an evidence-archetype check at
[`scripts/check_diagnosis_evidence.sh`](scripts/check_diagnosis_evidence.sh).

---

## 6. The "reasoned-from-evidence" pattern (process made checkable)

The hardest doctrine to enforce is a *process* ("you followed a root-cause procedure and reasoned
from the evidence"). You cannot read an author's mind — so reframe it into something mechanical:

> **A correct diagnosis is one whose documented cause→fix→effect chain REPRODUCES under
> independent re-execution.**

Mechanize it as a **two-signal evidence check** (the procedure made checkable):

1. **DIAGNOSIS signal (WHY+WHERE)** — the leaf pastes output from the tool that *located and
   explained* the cause (e.g. a profiler line, an error with a precise locus, a rejection trace).
2. **VERIFICATION signal (effect)** — the leaf pastes the *measured before→after* of the fix
   (a metric delta, a REJECT→PASS, determinism across fixed seeds).

The gate requires **both** (you must have located the cause *and* measured the effect). The
**oracle leg** then re-runs the cited deterministic commands in CI: a fabricated cause→fix→effect
chain will not reproduce, so it fails. At that point the distinction between "reasoned" and
"fabricated" collapses — *a reproducible chain is, operationally, a correct diagnosis.* That is the
scientific-method standard, and it is the strongest enforceable proxy for "reasoned from evidence."

### 6.1 A box is EARNED, not ticked (self-ticking is not proof)

A checklist `[x]` an author writes is a **claim**, not proof — a task could tick "NO REGRESSION" and
move on without earning it. So **ticking must never be the proof; the oracle re-run is.** Three legs,
in increasing strength:

1. **Presence (cheap, local hook):** the box exists and is ticked, with a tool-output *signature*
   next to it. This catches "forgot to do the step." It is, by itself, *self-tickable* — be honest
   about that; it is necessary, not sufficient.
2. **Evidence-shape:** the box co-occurs with a string only the real tools emit (a cert header, a
   probe verdict, a trace rejection). Raises the cost of faking, does not eliminate it. The string
   must sit **inside that box's own bullet** (box-scoped since `GENERATED-LINT-CORRECTNESS.3`), and
   it must belong to one of **five** diagnosis families — correctness, performance, build-integrity,
   codegen-emission, ops/build-flow — enumerated with their verbatim tokens in `TOOLBOX.md`.
   ⚠️ **A signature family that does not match the real corpus is a gate that teaches authors to
   waive it.** Measured in `GENERATED-LINT-CORRECTNESS.4`: the performance family named a generic
   Rust vocabulary this repo does not use and backed **2** boxes repo-wide, while an author had
   hand-written a waiver note *inside a ticked box* stating that no family fitted their defect
   class — and it sat unread. When extending, **price the candidate against the whole corpus before
   adopting it**: `.4`'s own chartered hypothesis, adopted on one clean sample, would have admitted
   **2 of 304** boxes.
3. **Oracle re-run (un-fakeable, CI / `make` gates):** the gate **re-executes the deterministic
   oracle the box claims** — e.g. a "NO REGRESSION" box is *earned* only when re-running
   certificate-coverage at seeds 0/7/42, the shape-contract gate, the byte-identical check across the
   stable grammars, and the external corpus all reproduce green. A self-ticked-but-false box passes
   leg 1 and dies at leg 3. **This is the leg that makes the box un-self-tickable.**

Therefore: every gated box **must cite a NAMED, re-runnable oracle** (a gate/command + its
deterministic result), so CI can re-run exactly that and *earn* the box independently of the tick. A
box with no re-runnable oracle (e.g. a subjective "LOCKSTEP") stays advisory, never hard-gated on the
tick alone. **Honest limit:** leg 3 lives at CI (E4); if CI is paused/manual, the un-fakeable re-run
only happens when someone runs the gate — so self-ticking is caught at the next gate run, not
instantly. Re-enabling an auto CI oracle job is what makes "earned, not ticked" hold *no matter what*.

---

## 7. Enforcement layering (E1→E4 — defense in depth)

Same model as `MEMORY_ARCHITECTURE.md` §9. Each layer catches what the last misses.

- **E1 — Discovery.** The doctrine is unmissable: named in the entrypoint docs (`README`,
  `TOOLBOX.md`, `docs/decisions/`), and (for an agent harness) re-injected at session start / on
  the relevant tool use via hooks. Discovery alone is *not* enforcement.
- **E2 — Self-check.** Each `check_*.sh` (the single source of truth for one doctrine) + the driver.
- **E3 — Git hook.** `.githooks/pre-commit` runs the driver; a non-compliant tree cannot commit
  locally. *Honest limit:* a local hook can be `--no-verify`'d or skipped if `core.hooksPath` is
  not set — it catches the common case cheaply; it is **not** the backstop.
- **E4 — CI.** The **same** driver runs server-side; `--no-verify` cannot reach it, so a
  non-compliant branch **cannot merge**. This is the un-bypassable layer — *only as strong as CI
  actually running.* If hosted CI is paused/manual, that is a real gap: re-enable an auto
  doctrine-gate job, or the "no matter what" guarantee degrades to "no matter what, until the next
  manual run."

To land non-compliant work, an author would have to defeat all four — and E4 cannot be defeated
from a clone.

---

## 8. The portable replay manifest (any project, any harness — "it just works")

Reproducible by replay: this is the **exact list of artifacts** a project copies/writes and the
**three commands** it runs. Path-agnostic and copy-pasteable, exactly like `MEMORY_ARCHITECTURE.md`
§9.1. Group A is verbatim; Group B is one tiny adapt; Group C is per-harness discovery; Group D is
your own doctrines.

### A — CORE, copy VERBATIM (project- and harness-neutral)
| Artifact | Role |
|---|---|
| `scripts/check_doctrines.sh` | the registry+driver — runs every check, reports, exits nonzero on any breach |
| `scripts/check_diagnosis_evidence.sh` | reference EVIDENCE check (the task-acceptance checklist gate) |
| `.githooks/pre-commit` | E3 local gate: regenerate derived artifacts, then run the driver |
| `.githooks/commit-msg` | E3: require an identifier-shaped work-unit id in the subject |
| `DOCTRINE_ENFORCEMENT.md` | this standard |
| `TOOLBOX.md` | the debug-toolbox catalog + the **acceptance-checklist template** a code change must satisfy |

### B — ADAPT (the only project-specific knobs)
- `scripts/check_doctrines.sh`: edit the `DOCTRINES=(…)` array (your doctrine ids → your check scripts).
- `scripts/check_diagnosis_evidence.sh`: the "what counts as a code change" path globs + the evidence/checklist signature regexes (your tools' output strings).
- `TOOLBOX.md`: your project's tools + the required checklist boxes.
- which heavy checks are CI-only vs pre-commit.

### C — DISCOVERY, one bootstrap pointer per harness (all IDENTICAL content; each points at README + MEMORY_ARCHITECTURE + TOOLBOX + this file)
`AGENTS.md` (Codex / Amp / common), `CLAUDE.md` (Claude Code), `GEMINI.md` (Gemini CLI),
`.cursorrules` (Cursor), `.windsurfrules` (Windsurf), `.github/copilot-instructions.md` (Copilot).
Ship whichever harnesses your team uses; keep them byte-identical.

### D — OPTIONAL harness hooks (a bonus where supported — NOT required for enforcement)
`.claude/settings.json` (Claude Code `SessionStart`/`PreToolUse` reminders). **Codex and other
harnesses without a hook system rely on Group C discovery + the git-level enforcement (A), which is
harness-neutral.** The reminders only *nudge*; the gate is what *enforces*.

### E — PER-PROJECT, write your own
- `scripts/check_<doctrine>.sh` per doctrine (the §4 contract) + one registry line in the driver.
- `docs/decisions/<directive>.md` for the human "why".

### The three commands (once)
```bash
chmod +x scripts/check_*.sh
git config core.hooksPath .githooks          # activate the local gate (E3)
# add ONE line to your CI pipeline (E4):  bash scripts/check_doctrines.sh
```

**Harness-agnostic guarantee.** The ENFORCEMENT (A) is git-level: `.githooks/pre-commit` + CI run
`check_doctrines.sh` regardless of whether the commit came from Codex, Claude Code, Gemini, or a
human. DISCOVERY (C) is per-harness via the bootstrap pointer files. Optional hooks (D) add in-context
reminders where the harness supports them. So a project backed by **Codex or Claude Code (or both)**
gets the **same** four-layer gate — non-compliant work lands only by defeating all four, and E4
cannot be defeated from a clone.

---

## 9. Honest limits (state them; do not over-claim)

- **Local hooks are bypassable** (`--no-verify`, unset `hooksPath`). CI is the real backstop; if CI
  is paused, enforcement is only as strong as the next CI/manual run. *Re-enabling auto CI is the
  true "no matter what."*
- **Evidence-presence can be gamed** by pasting fake tool output — *unless* the check re-runs the
  cited command (the oracle leg). Prefer structural and oracle checks; make evidence checks
  re-execute where possible.
- **A check cannot prove intent / understanding** — only that the *artifacts and oracles reproduce*.
  That reproducibility is the point: a reproducible cause→fix→effect chain is the operational
  definition of a correct fix, regardless of how it was produced.
- **Goal is expensive-and-visible non-compliance, not literal impossibility** — defense in depth,
  not a single unbreakable wall.

---

## 10. The live PGEN instance (this repo's registry)

The reference deployment. Enforced by [`scripts/check_doctrines.sh`](scripts/check_doctrines.sh)
via [`.githooks/pre-commit`](.githooks/pre-commit) (E3) + CI (E4).

| Doctrine | Archetype | Check | Proves |
|---|---|---|---|
| `MEMORY-ARCH` | structural | `scripts/check_memory_architecture.sh` | the durable 4-layer memory architecture invariants (`MEMORY_ARCHITECTURE.md` §9), including **layer A bounded by a line cap AND a byte cap**. ⭐ **Both, because the line-only form was measurably not binding — in this repository, for the whole life of the doctrine**: `MEMORY.md` sat at **60 lines (passing, exactly at its 60-line ceiling) and 138,403 bytes** — 2,306 bytes per line, one line of 18,816 bytes — so a *"bounded resume pointer"* was a 138 KB document with a green guard. Its `## Current state` block, whose heading reads *"OVERWRITE this block each update — do not append"*, had accumulated **18 distinct sessions and 81.3% of the file**. ⛔ The defect was in the **portable standard**, not just this deployment: `MEMORY_ARCHITECTURE.md` §9's own reference script prescribed the line-only cap, so every adopter inherited the bypass — §6/§9/§9.1 were corrected together. ⭐ The control that proves the fix earned rather than assumed: the **retired guard is re-executed from `git show HEAD:`** against the **real** pre-trim `MEMORY.md` and reports `memory-arch: OK` over those 138,403 bytes (`README-POLICY.2`, probes 8/0). ⭐ **Layer C is now a real reconcile too** (`README-POLICY.7`): every record must have its OWN ROW in `docs/decisions/INDEX.md` and every row must name a record that exists. The previous form asserted only that the index had *more than zero* rows and passed at **135 records / 133 rows** — two records invisible to their own index, doctrine green. **Adopted from the portable spine repo, where the stronger version already existed** (transfer runs both ways), then strengthened: row-anchored, because a bare basename match false-passes a record merely mentioned in a neighbouring row's prose, and bidirectional. Probes 6/0. Trim before cap: **60→39 lines, 138,403→5,720 bytes**; caps then set at 50 / 7168 with proportional headroom, the line cap *lowered* 60→50 to match the *"≤ ~50 lines"* the standard already stated |
| `DIAG-SEVERITY+DOCPATH` | structural | `scripts/check_diagnostics_and_docpaths.sh` | severity never masked by verbosity + repo-root-relative live-doc paths |
| `EBNF-SOURCE-OF-TRUTH` | structural | `scripts/check_ebnf_source_of_truth.sh` | no new out-of-band acceptance validator wired outside the EBNF |
| `REGEX-SELF-HOSTING` | oracle | `scripts/check_regex_self_hosting.sh` | the regex grammar self-hosts (gen↔parse duality) |
| `KNOWLEDGE-MAP` | structural | `knowledge-map/scripts/check_knowledge_map.sh` | the derived Knowledge Map is in sync with its fact sources |
| `TASK-ACCEPTANCE` | evidence | `scripts/check_diagnosis_evidence.sh` | a code change carries tool-backed WHY+WHERE diagnosis + measured verification in its task leaf (see `TOOLBOX.md`) |
| `WAIVER-ROUTING` | evidence | `scripts/check_waiver_routing.sh` | a task leaf claiming a gate's SIGNATURE SURFACE cannot express its evidence names the leaf that owns fixing the gate — ⭐ **an author writing a waiver IS the gate reporting a missing capability**, the highest-signal defect report a gate can receive, and `RGX-0090`'s sat unread for months (see `docs/decisions/project_waiver_is_a_gate_bug_report.md`) |
| `DESIGN-PRIOR-ART` | evidence | `scripts/check_design_prior_art.sh` | a task leaf proposing a NEW annotation/directive surface records a prior-art search over `grammars/ebnf.ebnf` / `docs/decisions/` / `docs/tasks/` / `docs/book/` first (see `docs/decisions/feedback_read_prior_art_before_designing.md`) |
| `FLOW-INTEGRITY` | structural | `scripts/check_flow_integrity.sh` | the gate flow cannot drift back. Nine invariants, **each traced to an incident that actually happened**: workflows needing generated parsers declare the regeneration step (14 of 15 could not build) and budget ≥30 min for it (the flagship budgeted 60 min for a 143-min job); the recipe keeps ONE home (it was about to be copy-pasted into ten more files); the parity gate's preparation stays on by default (the identical default eroded once before); no artifact hand-off points at a gate's STANDALONE default dir (a run consumed a **three-day-old** artifact as current proof); no assertion requires a defect to pass (a gate that passed only when the parser FAILED); hand-off provenance coverage only improves (1 of 23 at adoption, ratcheted); the doctrine roster keeps an AUTOMATIC lane **through this driver** — no auto-triggered workflow may re-type a list of enforcer names (the one auto-running workflow named 5, so 8 of 13 doctrines had no automatic lane and every doctrine registered afterwards inherited none — `CI-PARITY-GATE-ROT.15`); and a guard tests the artifact it actually READS (6 sites guarded on `summary.txt` then `jq`-read `summary.json`, so a sub-gate dying mid-run left a 0-byte `summary.txt`, the guard read FALSE, and the run died on a missing file four lines below the real cause — `CI-PARITY-GATE-ROT.14`). ⭐ **Adopted because the measurement was uncomfortable**: after `CI-PARITY-GATE-ROT` mechanized each repair, **four of the five new invariants lived in `ci_workflow_local_gate`** — OPERATOR tier, run only when a human asks. The flow had been fixed with checks that could themselves rot. This moves them to the automatic tier. ⛔ Derived, not hand-listed; the only two written-down inputs live in `flow_integrity_register_v0.json`, which the parity gate reads too so the rules cannot drift apart |
| `GATE-REACHABILITY` | structural | `scripts/check_gate_reachability.sh` | every tracked gate target is invoked by something that RUNS (an aggregate, a CI workflow, a git hook) or carries a deliberate disposition in a tracked register — ⭐ *a check that nothing INVOKES is indistinguishable from a check that does not exist.* Adopted after this repository found three such gates **by accident, one per session** (`ast_dump_contract_gate` RED for four sessions; `PGEN_CLIPPY_GENERATED_STRICT` set by nothing; `ci_workflow_local_gate` unable to complete for 1,371 commits). ⛔ A **ratchet, not a report** — the orphan set is re-derived every run and joined against the register, so it can neither be bypassed nor accumulate dead exemptions. ⚠️ Its own numbers are guarded by **ground-truth controls**: the instrument produced six different confident answers while being written, each from a real calibration defect, and every one was caught by requiring it to reproduce facts the project had already measured (see `docs/tasks/CI-PARITY-GATE-ROT.md` `.2`) |
| `REGEX-ORACLE-ANCHOR-SYNC` | structural | `scripts/check_regex_oracle_anchor_sync.sh` | the live PCRE2-oracle-tuple anchors (contract snapshot + validator chapter + `diagnosing-unknowns.md` + `MEMORY.md`) agree byte-for-byte and satisfy the tracked ratchet bounds (`REGEX-PCRE2-FIDELITY.DOCSYNC.2`) |
| `ROUTING-EVIDENCE` | evidence | `scripts/check_routing_evidence.sh` | a task leaf routing a finding OUT to another tree records what it MEASURED — above all whether the finding **reproduces outside** the family it is being sent to. Adopted after `CI-PARITY-GATE-ROT.9`: a shared-gate defect was routed to the regex family on a plausible reading, and the deciding evidence was already on disk (see `docs/tasks/CI-PARITY-GATE-ROT.md` `.12`) |
| `DESTRUCTIVE-TARGET-GUARD` | structural | `scripts/check_destructive_target_guard.sh` | destructive `make` targets refuse without `PGEN_CONFIRM_CLEAN=1`, and no innocuous-looking alias routes into one (`OPS-MEMSAFE.3`) |
| `PUBLISHED-VERSION-CURRENCY` | structural | `scripts/check_published_version_currency.sh` | the user guide's published regex identity pair equals the integration contract's Contract Identity block, and its published family status equals the live tracker row — ⭐ *a disclosure nobody checks is a claim*: `Provisional` SHIPS on its published state (director 2026-07-29), and this exact surface was measured **~77 releases stale** (`1.1.29`/`1.1.31` published vs `1.1.106`/`1.1.109` declared) with **no gate reading either document** (`DONE-BAR.5a`). An empty extraction FAILS rather than comparing empty strings |
| `README-STABILITY` | structural | `scripts/check_readme_stability.sh` | `README.md` stays a **stable landing page** rather than growing into a changelog, roadmap, gate catalogue or documentation inventory. Enforces a **line cap AND a byte cap**, a changelog-leakage tripwire, and a link back to the reviewed policy (`docs/reference/PGEN_README_STABILITY_POLICY.md`). ⭐ **Adopted because nothing was watching**: the README reached **510 lines / 48,811 bytes** while both guards that touch it looked elsewhere — `check_diagnostics_and_docpaths.sh` audits doc *paths written inside it*, `ci_workflow_local_gate.sh:277` audits which root markdown files *exist*; a README can triple in size with both green. **55.6%** of it was a file inventory plus an operations manual, and one bullet was **4,369 bytes on a single line**. ⛔ **Both caps, because the line-only shape was measurably bypassed in this very repo**: `check_memory_architecture.sh` capped layer-A `MEMORY.md` at 60 lines only, and it passed at **60 lines / 138,403 bytes** — 2,306 bytes per line, a "bounded resume pointer" that was a ~138 KB document. ✅ **That bypass is now CLOSED**: `README-POLICY.2` trimmed layer A to 39 lines / 5,720 bytes and gave `MEMORY-ARCH` the same two caps (see its row above), fixing the portable standard as well as this deployment. The precedent stands as the reason this guard shipped with both caps on day one. ⚠️ Honest bound printed in its own source: the changelog tripwire detects **one measured leakage class** (the dated historical annotation, 6 at adoption), not "changelog content" in general. A cap is **never** raised to land content — only by a reviewed decision recorded in `docs/tasks/README-POLICY.md` |

Deterministic-oracle doctrines that run via the broader `make` gates / CI (cert-coverage at seeds
0/7/42, `ast_shape_contract`, syntax-closure, the external-corpus triage) are the strongest leg —
they re-execute the real tools, so cited numbers are independently re-verified.

To add a doctrine here: write `scripts/check_<id>.sh` (§4 contract), add one line to the driver's
`DOCTRINES` array, and add a row above. The driver's meta-check fails if the script is missing.

---

## 11. Anti-patterns

- ❌ A doctrine that lives only as prose, with no check.
- ❌ "Trust me, I followed the procedure" with no re-checkable artifact.
- ❌ An evidence check that greps for a signature but never re-runs the oracle (fakeable).
- ❌ A registry entry pointing at a check that does not exist (a dangling promise — the meta-check catches this).
- ❌ A check with side effects / nondeterminism (then the gate cannot be trusted).
- ❌ Relying on the local hook as the backstop (it is bypassable — CI is the backstop).
- ❌ Over-claiming "impossible to violate" — the honest claim is "expensive, visible, and blocked at every active gate."

---

*This document is itself an instance of the architecture it describes: a portable, in-repo,
git-tracked standard backed by a runnable driver and mechanical gates — adoptable by any project
by following §8.*
