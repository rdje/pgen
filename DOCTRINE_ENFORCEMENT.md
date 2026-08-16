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
| 5 | **Claim-verification** | what "checked" means before a number is PUBLISHED — re-derive · falsify · make durable | `docs/CLAIM_VERIFICATION.md` |

⭐ **(5) comes logically BEFORE this file, and was adopted after it for the usual reason.** This
standard asks *"is this rule enforced?"*; claim-verification asks *"is this **number** earned?"* — and
a gate built on an unverified measurement enforces the wrong thing precisely, forever. Measured in
this deployment: a published blind-spot bound of `~35×` was corrected to `~8.9×`, because the
classifier that produced it was written from the design's own prose and its ten controls were drawn
from that same prose, so they could only ever agree with each other — and ⛔ **the corrected figure
was then RETIRED too** (`ENGINE-UNIVERSAL-SERVICES.26`), because *both* of its terms turned out to
be wrong: a refuted wall-clock numerator over a denominator that was the wrong quantity. Two
corrections, three sessions apart, on one number that no gate had ever re-derived.

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
   must sit **inside that box's own bullet** (box-scoped since `GENERATED-LINT-CORRECTNESS.3`), the
   box must sit in a **leaf section the change actually touches** (leaf-scoped since
   `GENERATED-LINT-CORRECTNESS.7`), and it must belong to one of **five** diagnosis families —
   correctness, performance, build-integrity, codegen-emission, ops/build-flow — enumerated with
   their verbatim tokens in `TOOLBOX.md`.
   ⛔⛔ **Box-scoping without leaf-scoping is VACUOUS, and it was — for every mature tree in this
   repository.** `.3` tied the signature to its box but never tied the box to the change, so `any`
   ticked box in `any` staged task file satisfied a requirement: a new leaf could carry no checklist
   at all and inherit a finished leaf's. Measured at `.7` — **33 tracked task files held that
   standing free pass**, 7 of the last 138 code-change commits passed *only* by borrowing, and the
   leaf that fixed it would itself have passed box 1 on four of its own tree's historical boxes.
   ⚠️ The first implementation of the fix was *also* vacuous against the most common edit in the
   repo — appending a new leaf begins with the blank separator line that still belongs to the
   PREVIOUS section — and only a RED probe caught it. See
   `docs/decisions/project_acceptance_box_must_be_written_by_the_change.md`.
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
| `MEMORY-ARCH` | structural | `scripts/check_memory_architecture.sh` | the durable 4-layer memory architecture invariants (`MEMORY_ARCHITECTURE.md` §9), including **layer A bounded by a line cap AND a byte cap**. ⭐ **Both, because the line-only form was measurably not binding — in this repository, for the whole life of the doctrine**: `MEMORY.md` sat at **60 lines (passing, exactly at its 60-line ceiling) and 138,403 bytes** — 2,306 bytes per line, one line of 18,816 bytes — so a *"bounded resume pointer"* was a 138 KB document with a green guard. Its `## Current state` block, whose heading reads *"OVERWRITE this block each update — do not append"*, had accumulated **18 distinct sessions and 81.3% of the file**. ⛔ The defect was in the **portable standard**, not just this deployment: `MEMORY_ARCHITECTURE.md` §9's own reference script prescribed the line-only cap, so every adopter inherited the bypass — §6/§9/§9.1 were corrected together. ⭐ The control that proves the fix earned rather than assumed: the **retired guard is re-executed from `git show HEAD:`** against the **real** pre-trim `MEMORY.md` and reports `memory-arch: OK` over those 138,403 bytes (`README-POLICY.2`, probes 8/0). ⭐ **Layer C is now a real reconcile too** (`README-POLICY.7`): every record must have its OWN ROW in `docs/decisions/INDEX.md` and every row must name a record that exists. The previous form asserted only that the index had *more than zero* rows and passed at **135 records / 133 rows** — two records invisible to their own index, doctrine green. **Adopted from the portable spine repo, where the stronger version already existed** (transfer runs both ways), then strengthened: row-anchored, because a bare basename match false-passes a record merely mentioned in a neighbouring row's prose, and bidirectional. Probes 6/0. Trim before cap: **60→39 lines, 138,403→5,720 bytes**; caps then set at 50 / 7168 with proportional headroom, the line cap *lowered* 60→50 to match the *"≤ ~50 lines"* the standard already stated. ⭐⭐ **The byte cap is 32768 since 2026-08-14 by DIRECTOR RULING (`README-POLICY.8`) — raised to restore HEADROOM, which is a different act from raising a cap to LAND CONTENT, and permitted by §6 only as *"an explicit reviewed decision recorded in the work-tracking system"*.** The `.2` headroom was spent: over layer A's last 40 commits `git rev-list`+`git cat-file -s` measured min 6 795 / median 7 079 / **max 7 168 = the cap exactly**, with **36 of 40 at ≥ 97 %** and 117 bytes left — and the diagnosis is not any single size but `git diff --numstat HEAD~9 HEAD` reporting **`4 4`**, the same four lines rewritten in place and never grown, i.e. **the cap had stopped bounding the LAYER and started editing the PROSE**. The ruling was taken while layer A was PASSING, which is the whole point: this repository's own card `a-cap-with-no-headroom-is-a-cap-about-to-be-raised` says *"the moment a cap blocks you is the worst possible moment to decide policy about it"*, and that card's `reverify:` had been RED for weeks because **nothing executes a card's re-verification command** (routed → `README-POLICY.9`). ⚠️ Honest cost, stated not discovered: at 50 lines / 32768 bytes the byte axis permits ~655 B/line, so the **line cap is the sole binding axis** in the 7–32 KB band and the two-axis design degrades toward the single-axis form `.2` replaced; the byte cap keeps only its original job of making the 138,403-byte outcome impossible. ⭐ Mitigated by REPORTING rather than by another bound — the enforcer now prints `memory-arch: OK (layer A 7051/32768 bytes = 21% of cap, 30/50 lines = 60% of cap)` on every passing run, because 36 consecutive commits hard against the edge went unremarked while a passing gate printed the same three characters at 5,720 bytes as at 7,168. The report adds no failure path: the retired enforcer re-executed from `git show HEAD:` is **byte-identical including `rc`** on the failing path. ⛔ The portable standard's default stays **7168** — 32768 is this deployment's reviewed budget, and exporting a number without the measurement that earned it is the defect `.2` repaired in the standard, restated |
| `DIAG-SEVERITY+DOCPATH` | structural | `scripts/check_diagnostics_and_docpaths.sh` | severity never masked by verbosity + repo-root-relative live-doc paths |
| `EBNF-SOURCE-OF-TRUTH` | structural | `scripts/check_ebnf_source_of_truth.sh` | no new out-of-band acceptance validator wired outside the EBNF |
| `REGEX-SELF-HOSTING` | oracle | `scripts/check_regex_self_hosting.sh` | the regex grammar self-hosts (gen↔parse duality) |
| `KNOWLEDGE-MAP` | structural | `knowledge-map/scripts/check_knowledge_map.sh` | the derived Knowledge Map is in sync with its fact sources |
| `TASK-ACCEPTANCE` | evidence | `scripts/check_diagnosis_evidence.sh` | a code change carries tool-backed WHY+WHERE diagnosis + measured verification in its task leaf (see `TOOLBOX.md`). ⭐⭐ **The box must be one THIS change wrote** (`GENERATED-LINT-CORRECTNESS.7`): a satisfying box must sit in a leaf section (headings ≤ 3) the staged change TOUCHES, and all three requirements must be met within ONE file. Until `.7`, box-scoping was **vacuous for every mature tree** — `box_matches` accepted any ticked box in any staged task file, so **33 tracked task files carried a standing free pass** and **7 of the last 138 code-change commits passed ONLY by borrowing** (5 whose own NO REGRESSION box carried no gate signature, 2 with no qualifying box at all); all 7 were read individually and are genuine, so the strengthening has **0 false positives**. ⛔ The same leaf **REFUSED a sixth signature family** at **0–3 of 307 (0 %)** — below `.4`'s refused 2/304 — because the gap is **97 % PLACEMENT** (299 OUT-OF-BOX vs 8 NO-EVIDENCE) and no token set closes a placement gap. ⚠️ Its first implementation was itself vacuous against appending a new leaf (the blank separator line belongs to the *previous* section) and only a RED probe caught it; probes **9/9 after vs 6/9 before**, with every GREEN/CONTROL arm byte-identical on both sides. Routed → `.8`: `NOREGRESS_SIG` repeats group 2's wrong-vocabulary defect (**120 of 416, 29 %** unbacked) |
| `WAIVER-ROUTING` | evidence | `scripts/check_waiver_routing.sh` | a task leaf claiming a gate's SIGNATURE SURFACE cannot express its evidence names the leaf that owns fixing the gate — ⭐ **an author writing a waiver IS the gate reporting a missing capability**, the highest-signal defect report a gate can receive, and `RGX-0090`'s sat unread for months (see `docs/decisions/project_waiver_is_a_gate_bug_report.md`) |
| `LESSON-PROMOTION` | evidence | `scripts/check_lesson_promotion.sh` | a commit adding a dated lesson entry to `DEVELOPMENT_NOTES.md` must ALSO stage a promotion (a `docs/knowledge/` card, or `answers:` on a `docs/decisions/` record) OR an explicit `promotion: declined (<reason>)` in its task leaf. Declining is a first-class outcome — the gate demands a DECISION, not a promotion. Measured provenance: 1592 lesson entries accumulated in a file that is not a Knowledge Map scan dir, and 0 of 142 decision records carried `answers:`; the mechanism was wired and skipped silently every time because `KNOWLEDGE-MAP` verifies the map is in sync with its SOURCES and never that a lesson REACHED a source (`LESSON-RETRIEVAL.4`, director call 2026-08-01 — a doctrine check, explicitly NOT a `COMMIT.md` reminder, because a reminder had already lost 1592 times) |
| `DESIGN-PRIOR-ART` | evidence | `scripts/check_design_prior_art.sh` | a task leaf proposing a NEW annotation/directive surface records a prior-art search over `grammars/ebnf.ebnf` / `docs/decisions/` / `docs/tasks/` / `docs/book/` first (see `docs/decisions/feedback_read_prior_art_before_designing.md`) |
| `FLOW-INTEGRITY` | structural | `scripts/check_flow_integrity.sh` | the gate flow cannot drift back. Ten invariants, **each traced to an incident that actually happened**: workflows needing generated parsers declare the regeneration step (14 of 15 could not build) and budget ≥30 min for it (the flagship budgeted 60 min for a 143-min job); the recipe keeps ONE home (it was about to be copy-pasted into ten more files); the parity gate's preparation stays on by default (the identical default eroded once before); no artifact hand-off points at a gate's STANDALONE default dir (a run consumed a **three-day-old** artifact as current proof); no assertion requires a defect to pass (a gate that passed only when the parser FAILED); hand-off provenance coverage only improves (1 of 23 at adoption, ratcheted); the doctrine roster keeps an AUTOMATIC lane **through this driver** — no auto-triggered workflow may re-type a list of enforcer names (the one auto-running workflow named 5, so 8 of 13 doctrines had no automatic lane and every doctrine registered afterwards inherited none — `CI-PARITY-GATE-ROT.15`); and a guard tests the artifact it actually READS (6 sites guarded on `summary.txt` then `jq`-read `summary.json`, so a sub-gate dying mid-run left a 0-byte `summary.txt`, the guard read FALSE, and the run died on a missing file four lines below the real cause — `CI-PARITY-GATE-ROT.14`); and the SHIPPING generation recipe stays QUIET — no tracked Makefile line invokes `--generate-parser` carrying `--debug`/`--trace` (the recipe every deliverable parser is generated by asked the generator to narrate itself: **6 894 576 244 B** per full regeneration, locally AND streamed into the Actions log of the 11 workflows reaching it through the composite action, for artifacts the flags provably cannot change — the `PipelineConfig.debug`/`.trace` fields they set are written and never read. Director-ruled 2026-08-14; `CI-PARITY-GATE-ROT.31`). ⭐ **Adopted because the measurement was uncomfortable**: after `CI-PARITY-GATE-ROT` mechanized each repair, **four of the five new invariants lived in `ci_workflow_local_gate`** — OPERATOR tier, run only when a human asks. The flow had been fixed with checks that could themselves rot. This moves them to the automatic tier. ⛔ Derived, not hand-listed; the only two written-down inputs live in `flow_integrity_register_v0.json`, which the parity gate reads too so the rules cannot drift apart |
| `GATE-REACHABILITY` | structural | `scripts/check_gate_reachability.sh` | every tracked gate target is invoked by something that RUNS (an aggregate, a CI workflow, a git hook) or carries a deliberate disposition in a tracked register — ⭐ *a check that nothing INVOKES is indistinguishable from a check that does not exist.* Adopted after this repository found three such gates **by accident, one per session** (`ast_dump_contract_gate` RED for four sessions; `PGEN_CLIPPY_GENERATED_STRICT` set by nothing; `ci_workflow_local_gate` unable to complete for 1,371 commits). ⛔ A **ratchet, not a report** — the orphan set is re-derived every run and joined against the register, so it can neither be bypassed nor accumulate dead exemptions. ⚠️ Its own numbers are guarded by **ground-truth controls**: the instrument produced six different confident answers while being written, each from a real calibration defect, and every one was caught by requiring it to reproduce facts the project had already measured (see `docs/tasks/CI-PARITY-GATE-ROT.md` `.2`) |
| `REGEX-ORACLE-ANCHOR-SYNC` | structural | `scripts/check_regex_oracle_anchor_sync.sh` | the live PCRE2-oracle-tuple anchors (contract snapshot + validator chapter + `diagnosing-unknowns.md` + `MEMORY.md`) agree byte-for-byte and satisfy the tracked ratchet bounds (`REGEX-PCRE2-FIDELITY.DOCSYNC.2`) |
| `ROUTING-EVIDENCE` | evidence | `scripts/check_routing_evidence.sh` | a task leaf routing a finding OUT to another tree records what it MEASURED — above all whether the finding **reproduces outside** the family it is being sent to. Adopted after `CI-PARITY-GATE-ROT.9`: a shared-gate defect was routed to the regex family on a plausible reading, and the deciding evidence was already on disk (see `docs/tasks/CI-PARITY-GATE-ROT.md` `.12`) |
| `DESTRUCTIVE-TARGET-GUARD` | structural | `scripts/check_destructive_target_guard.sh` | destructive `make` targets refuse without `PGEN_CONFIRM_CLEAN=1`, and no innocuous-looking alias routes into one (`OPS-MEMSAFE.3`) |
| `PUBLISHED-VERSION-CURRENCY` | structural | `scripts/check_published_version_currency.sh` | the user guide's published regex identity pair equals the integration contract's Contract Identity block, and its published family status equals the live tracker row — ⭐ *a disclosure nobody checks is a claim*: `Provisional` SHIPS on its published state (director 2026-07-29), and this exact surface was measured **~77 releases stale** (`1.1.29`/`1.1.31` published vs `1.1.106`/`1.1.109` declared) with **no gate reading either document** (`DONE-BAR.5a`). An empty extraction FAILS rather than comparing empty strings |
| `LIVE-DOC-CURRENCY` | structural | `scripts/check_live_document_currency.sh` | a live document is currently **TRUE**, not merely bounded. Two baseline-free instruments plus route closure (`LIVE-MEANS-LIVE.2`). **A** — count DISTINCT dates in a surface, **paired with its declared charter**: the instrument classifies, the charter says which classification is permitted, which is what stops it firing on `CHANGES.md` (175 dates, and correct — being a log is its charter). **B** — a self-declared `Last updated:` refuted by the file's own newest content date: no external baseline, no threshold, because the file's two halves contradict each other. ⭐⭐ **B is DORMANT since `LIVE-MEANS-LIVE.4a` and the check SAYS SO rather than passing vacuously** — the field was deleted from all 62 files that carried it once `git log -1 --date=short` was measured to dominate it strictly (**25 of 61 declarations were simply wrong and NOT ONE was ever ahead of git**; on the published contracts, **6 of 6 stale**). A hand-maintained duplicate of a derivable fact can only ever lag, and its failure direction is to make the project look more abandoned than it is. B stays **fully wired as a re-introduction tripwire** — a declaration added tomorrow is checked from its first commit — and its ground-truth controls run every invocation, so it is provably alive with an empty population. ⛔ `0/914 declare, 0 self-refuting` printed as an ordinary OK line would be indistinguishable from an instrument that had silently stopped seeing its subject, which is the very failure this doctrine exists to catch. **Route closure** — every `.md` destination a capped enforcer NAMES must be a watched surface, with the edges **DERIVED from the enforcers' own output text**, because the edge that carried the rot was a hint string inside an error message (`check_readme_stability.sh:83`) that no hand-authored route registry could see. ⭐ **Adopted because a cap that redirects has not fixed anything**: `README.md` was capped on two axes while the file its overflow rule named reached **1 547 057 B, 94.7 % dated changelog**, with no instrument at all. ⛔ **And a byte cap is the wrong half** — `gate-flow.md` is the LARGEST of the three healthy overflow destinations and the healthiest; the question is not *big*, it is *has this stopped being a status document*. ⭐⭐ **B REFUSES (exit 2) on any anchored declaration it cannot classify, instead of enumerating spellings** — enumeration is precisely what failed, twice, on one population: `.4` measured **10** knowing one spelling (blind to ~50 `docs/tasks/` trees written `` - Last updated: `2026-05-31` ``, plus one phantom row from prose), `.5` corrected to **16** knowing two (blind to the `docs/contracts/` continuation spelling — 6 files, **2 of them self-refuting published downstream contracts**), and `.2` measures **18**. Every miss was an **absent row**, i.e. silent in the passing direction, and each was caught only because a prior published number existed to disagree with. ⚠️ Its numbers are guarded by **8 ground-truth controls** run before any measurement — one per pinned shape, a positive, a negative, a fenced-block exclusion, a mid-line-prose exclusion, and one that proves the refusal path itself is live; any miss aborts with exit 2 rather than publishing. Both debt lists are **two-sided ratchets** (a new breach fails; a *paid* one still listed also fails), and all 11 failure paths were **proven to fire** before the doctrine was trusted |
| `SV-CORPUS-DENOMINATOR` | evidence | `scripts/check_sv_corpus_denominator.sh` | the SystemVerilog corpus **denominator** is re-derived and published beside the defect bar — ⭐ *a bar without its denominator is not a claim about the corpus.* The graduation bar counts DIVERGENCES (318) and is silent on what fraction of the corpus was asked a question it could answer (46.3 % adjudicated; **4 398 rows / 26.9 % fail inside a deferral nobody has looked at**, `SV-CORPUS-GRAD.13a`). Two legs: (1) the tracked verdict-coverage artifacts are **byte-identical to a fresh re-run** of `stimuli/sv/corpus_verdict_coverage.py` — regenerated into a SCRATCH directory, because a check that rewrites the file it then compares always passes; (2) every designated LIVE surface carries the derived `adjudicated/routed/no-verdict/dark/axis-2-bar` tuple, so the bar cannot be re-published without its denominator moving with it (marker-scoped, as `REGEX-ORACLE-ANCHOR-SYNC` uses bold — era-dated citations in prose stay history). ⛔ **Adopted because the rot was MEASURED, not imagined**: the census artifact landed in `PGEN-SV-CORPUS-GRAD-0202` and was already wrong **one day later** (published `match 5 804` / bar **319** against HEAD's `5 805` / **318** after the `.12c.1` Latin-1 fix), and `git ls-files 'scripts/*.sh' 'rust/scripts/*.sh' '.github/workflows/*.yml' 'rust/Makefile' '.githooks/*' | xargs grep -ln corpus_verdict_coverage` returned **nothing**. ⛔ The class→bucket map is NOT duplicated in the enforcer — it INVOKES the instrument, which itself refuses on an unclassified adjudication class. ⚠️ Honest limits in its own source: it re-derives the published NUMBERS from tracked inputs, never re-adjudicating the corpus; and the vendored corpora are git **submodules**, so a checkout without them reports the DARK-half backtick leg as **NOT EVALUATED** — loudly, never as a pass — while the corpus-independent legs still bind. All six arms were fired before it was trusted (stale artifact / stale anchor / missing anchor / instrument refusal / corpus-absent mode / a deliberately broken ground-truth control, which exits 2) |
| `PARSE-COST-RATCHET` | structural + oracle | `scripts/check_parse_cost_ratchet.sh` | the SystemVerilog parser's **parse cost** is measured, and cannot rise unwatched — ⭐ *a cost nothing measures is a cost that grows.* `ENGINE-UNIVERSAL-SERVICES.17` slice 9 shipped the guarded left-recursion admission, and the generated lint, the two-sided repro ratchet, the corpus pass/fail count and **every registered doctrine stayed GREEN** across it, because nothing in the tree measured parse cost at all. Its deterministic cost, measured afterwards: **+10.59 % rule entries**. ⛔⛔ **This row founded the doctrine on a `+24.3 % parse time` figure, and `.20` slice 5 REFUTED it** (2026-08-16): the number is not reproducible from the raw data of the runs that produced it — seven estimator × era combinations put ARM2/ARM1 in **[0.9909, 1.0433]** — and its mechanism was a fixed-arm-order measurement artifact. ⭐ The rationale is **strengthened, not weakened**: an unmeasured wall-clock number was carried for three sessions, through five slices and a director-facing ruling, and was wrong by ~20 points, which is precisely the case for a ratchet keyed on quantities a reader can re-derive. `.26` propagated the refutation and retired the blind-spot factor built on it. **Two tiers, and the cheap one is a proof rather than a shortcut**: (1) IDENTITY, every run (~2 s over five arms) — re-hash the four inputs the baseline names (grammar, generated parser, instrument, and a digest over the sampled corpus files); the binding metric is an exact function of exactly those, so if none moved the measurement *cannot* have moved, and if one did the gate demands a re-measure instead of guessing; (2) the RATCHET, on demand (`make -C rust sv_parse_cost_ratchet`, ~2.5 min) — re-run the instrument into a SCRATCH directory and refuse on any RISE in the binding counters. ⭐⭐ **ARM 5 IS THE EXECUTABLE, AND IT EXISTS BECAUSE THE OTHER FOUR ARE ALL SOURCES** (`ENGINE-UNIVERSAL-SERVICES.24`, 2026-08-17). The numbers are produced by `rust/target/release/parseability_probe`, an UNTRACKED build artifact nothing hashed and nothing tied to the parser it was compiled from; `.20` slice 4 built one from a guard-suppressed experimental arm and this gate printed *"the measurement cannot have moved"* while `nm … | grep -c _lr_guard` read **0** against a pinned parser declaring **6**. ⛔ **The gap failed in the PASSING direction**, which is why it is closed rather than documented: on four sampled files the wrong binary reports **762,345** rule entries where the shipped one reports **11,240,430** (14.7×), and this ratchet breaches on a RISE — a FALL is a note reading *"an improvement — promote it deliberately"*, so the gap invited a rebaseline that would have lowered the ratchet permanently to a number no real parser produces. **The fix costs ZERO generated bytes**: `rust/build.rs` already resolves each generated parser and already declares `cargo:rerun-if-changed` for it, so it re-runs exactly when one moves — it now sha256s each and publishes `PGEN_<FAMILY>_PARSER_SHA256`, which `parseability_probe --parser-fingerprint` reports as JSON and the instrument's `--verify-probe-fingerprint` compares (0 matches / 1 differs / 3 NOT EVALUATED). Emitting the fingerprint INTO the parser instead — the option the leaf originally framed — moves every generated artifact and re-baselines everything keyed on them, including `GENERATED-REPRODUCIBILITY`. ⭐ **Two independent sha256 implementations agree and are re-checked on every run**: Rust `sha2` in `build.rs`, Python `hashlib` (OpenSSL) in the instrument — verified 9/9 across every resolved parser. ⚠️ **Tier 1 NOTES, tier 2 REFUSES**: tier 1 computes no measurement, so a stale probe misleads nothing it prints, and failing every commit over an untracked build artifact is how a gate teaches people to bypass it. ⚠️ `PGEN_PARSE_COST_ALLOW_PROBE_MISMATCH=1` lets an operator measure an EXPERIMENTAL arm on purpose — the workflow `.20` slices 3-5 exist because of — and STAMPS the mismatch into `advisory.json` so the result cannot later pass for a baseline; the gate strips it from the environment, because an escape hatch that reaches the gate is a hole. ⭐⭐ **The binding metric is NOT wall clock, and that is the substantive call** (director-delegated ruling C, 2026-08-14): this leaf's first cost figure, `~11 %`, was wrong precisely because it compared wall clock across *"materially faster machine conditions"*, so a wall-clock-primary ratchet inherits the defect that misled it. It binds instead on **exact integers** — rule entries, COMMITTED entries and memo hits, each verified deterministic across repeated release runs *and* byte-identical between the debug and release probes before being trusted. ⭐ It reads the **outcome** dump (TOOLBOX 3.5), not the plain entry dump, because that was measured to cost the same and adds `raw − committed` = the parse's **failed-speculation** work — the mechanism a structural guard actually spends through (98.3 % of sampled entries; the LR-elimination family commits **514 of 12 440 690**). ⚠️ **Honest limit, stated before the instrument was trusted, not after**: the counters tick only in the PROTOCOL graph, so they cannot describe the fused `cascade_*` graph a production parse runs — and a counter counts EVENTS, so a rise in the cost PER event is invisible to it on any graph. ⭐ That is a **property** of the metric, not a measurement of it, so it cannot go stale. **Live LR-family share `2.741`** (corpus-entry share %) — the LR-elimination family's share of all corpus rule entries, re-hashed against its tracked derivation and co-published verbatim on every live surface. ⛔⛔ **That anchor was the PAIR `2.741/8.9` until `.26`, and the retired half was wrong in BOTH terms**: the numerator was the refuted `+24.3 %`, and the family SHARE was the wrong denominator independently of that — sensitivity is how much the counter MOVED, not how large the family is. Measured, in this leaf's own tracked artifact: the flip moved the binding counters **+10.59 %**, **3.49× LARGER** than the family's whole entry count, so the counters saw it plainly; the retired bound had inferred the opposite. It is retired rather than re-computed because it fails under every available reading (0.41× / 1.82×) and no admissible wall-clock figure survives to rebuild it from. ⛔ The share read **0.681 %** until `ENGINE-UNIVERSAL-SERVICES.21`, because the classifier measuring it matched only `_lr_base`/`_lr_suffix`: 97 of the parser's **127** LR rule names, **no `_lr_seed`, and in a family named for the GUARD not one `_lr_guard` rule**, leaving **75.1 %** of the family's entries uncounted. ⭐ The BINDING counters were unaffected (entries/committed/memo-hits are byte-identical across the correction), and the instrument is now part of its own baseline's IDENTITY so the same staleness cannot recur silently. ⚠️ That denominator was published as **128** and was wrong by one until `.21` slice 2 — its own decomposition (97 + 24 + 6) already summed to 127 — which is the point restated: a number carried into four surfaces is stale in four surfaces. It is now GATED across **all ten** generated parsers by the instrument's `--verify-families` mode (131 declared LR names, 131 classified; the roster is derived from the artifacts, not hand-listed), proven by five arms including a RED run of the OLD predicate that correctly leaves **30** names unclassified. ⭐⭐ **`ENGINE-UNIVERSAL-SERVICES.21` acceptance (f) then moved tier 1 from one arm to four**, because the two things that had gone stale were precisely the two nothing ran on a commit: `--verify-families` rode the ON-DEMAND tier 2, and the family share was a hand-carried constant referenced only by the file defining it and guarded by a *comment* — `docs/CLAIM_VERIFICATION.md` §6 names *“replacing a wrong unwatched number with a right unwatched number”* as an anti-pattern in its own right. Tier 1 now also (3) re-hashes the four inputs the share is a function of — grammar, generated parser, CLASSIFIER pattern and a digest of all 16 336 corpus files — against the tracked `family_share.json` derivation, and (4) holds every designated live surface equal to it, the co-publication leg `SV-CORPUS-DENOMINATOR` uses. Measured cost of the three new arms: **~1.5 s**; there was never a cost argument for deferring them. **11/11** refusal arms observed firing (`docs/tasks/artifacts/engine_universal_services/family_share_gate/probe.sh`), including one that replays the pre-`.21` narrow classifier and is refused, and two added by `.26` for the leg holding the artifact's RAW COUNTS to the share it declares. ⭐ They REPLACE an arm that mutated the now-retired `blind_spot_factor` field: measured, that mutation exits **0** where the arm asserts **1**, so the suite would have reported it ✗ — the probe caught its own obsolescence loudly rather than testing a field that no longer exists. It binds STRUCTURAL work exactly and says so; the floor-subtracted wall-clock advisory is the only view of the other graph and is reported against a wide ±50 % band that **never fails the gate**. Neither metric alone is sufficient, and the report says so on every run. ⚠️ The corpora are git **submodules**, so a checkout without them reports NOT EVALUATED — loudly, never as a pass. Nine adversarial arms fired before it was trusted (grammar / sampled-corpus-file / recorded-hash perturbations; a deleted identity table and a renamed column, both of which must REFUSE with exit 2 rather than pass; a risen and a fallen binding counter; plus an unrelated-file control that must not misfire) |
| `SCRATCH-SLOT-HEADER` | structural | `scripts/check_scratch_slot_header.sh` | the blessed throwaway scratch slot (`grammars/scratch/scratch.ebnf`, TOOLBOX 1.3) keeps its **operating manual**. The file's BODY is meant to be overwritten freely; its 32-line header is the only record of how to regenerate the slot, how to drive it, the ⛔ rebuild-`ast_pipeline`-AFTER ordering trap, that `generated/scratch*` is git-ignored, and how to restore the default fixture. ⭐ **Adopted because the loss is structurally invisible to a commit-time gate**: measured 2026-08-15, an agent loading a probe grammar replaced the whole file — header included — and the correct probe workflow *ends* in `git checkout grammars/scratch/scratch.ebnf`, so the commit shows **no diff there at all**. The scratch integration test could not see it either: it asserts the default fixture parses `"hello, world!"`, which a header-less file with the same body does. ⛔⛔ **And the remedy is deliberately NOT another sentence in the header** — the header ALREADY said *"Edit the grammar body below"*, in its own HOW TO USE section, and was overwritten anyway; §1 of this document is that a rule nothing checks is a suggestion, so a second suggestion in a file whose first was just ignored buys nothing. **Two tiers, split by when each failure is visible**: STRUCTURAL (this doctrine, every commit) requires the banner-delimited block plus the four operational **anchors** — `focus_scratch`, `parseability_probe --parse scratch`, `git-ignored`, `git checkout grammars/scratch/scratch.ebnf` — and deliberately does *not* diff against HEAD, so rewording the manual stays free; `--probe-time` (called from the `$(SCRATCH_JSON)` rule, which re-runs exactly when the slot has changed, and from `preserve_scratch_probe.sh`) additionally requires byte-identity with the committed header, **derived from `git show HEAD:` rather than from a second copy**, with `PGEN_SCRATCH_HEADER_EDIT=1` as the declared escape. ⭐ The anchors are commands and paths rather than prose phrases, because enumerating wordings is what made `LIVE-DOC-CURRENCY`'s instrument B measure one population as 10, then 16, then 18. ⭐ `--restore-header` re-attaches the committed header to whatever body is loaded, so the refusal is a one-command repair instead of an invitation to reconstruct a manual by hand. `--self-test` fires all eight arms (including the incident reproduced exactly, a single missing anchor, and a proof that the repair preserves the probe body) |
| `GENERATED-REPRODUCIBILITY` | structural + oracle | `scripts/check_generated_reproducibility.sh` | every artifact in the untracked `generated/` tree is **what HEAD's tracked source produces** — ⭐ *not merely what it compiles into, and not merely unchanged since someone recorded a hash.* ⛔ **Adopted because the rot was MEASURED and nothing in the repository could see it** (`ENGINE-UNIVERSAL-SERVICES.29`, 2026-08-16): both annotation parsers carried the line `self.coverage_deltas.clear();`, which the code generator **cannot emit** — `grep -c` over `ast_based_generator.rs` → **0**, `git log -S` → **no commit ever** — and whose absence the emitter's own source comment deliberately defends. They had been written from an uncommitted editor state ~80 minutes before the commit that finalised the emission, and they are the pair `rust/src/lib.rs` includes by literal path and the annotation backend links, i.e. **the pair that participates in generating every other parser**. The project's layer-A resume pointer recorded *"`generated/` FRESH"* throughout. ⛔⛔ **Each existing candidate was measured and none could see it**: a recorded HASH proves only that an artifact has not moved since someone recorded the hash; `fixed_point_gate` proves regeneration converges **across its own cycles** and OVERWRITES the on-disk artifact in cycle 1, so a stale copy is invisible to it *by construction*; `PARSE-COST-RATCHET` tier 1 re-hashes the SV parser, which detects MOVEMENT, never STALENESS; and a green build proves the artifact COMPILES. **Re-derive and diff is the only thing that answers the question.** **Two tiers, the cheap one a proof rather than a shortcut** (the architecture `PARSE-COST-RATCHET` established): (1) IDENTITY, every commit, **1.0 s**, no build — re-hash the three things each artifact is a function of (the artifact, its input JSON, and a digest over the tracked emission sources **plus the recipe that invokes them**), so if none moved the artifacts *cannot* have become stale, and if one moved the gate demands a re-verify rather than guessing which way; (2) the ORACLE, on demand (`make -C rust generated_reproducibility_gate`, **57 s**) — re-derive all **10** artifacts through the tracked recipe and demand byte-identity. ⭐ **The emission source set is DERIVED, never hand-listed** (`git ls-files rust/src/ast_pipeline` + `rust/Makefile`), so a code-generator file added tomorrow joins the identity by construction; it is deliberately **over-inclusive**, because its failure direction is then a spurious re-verify rather than silent staleness. ⭐⭐ **Every comparison ASSERTS the embedded `-o` site count before trusting a hash and REFUSES (exit 2) on a mismatch** — a generated parser embeds its own output path once per rule-entry site (36 346 times in SystemVerilog), so re-deriving to a scratch filename changes the artifact's size for reasons unrelated to the source; that trap has inverted **three** published readings in this repository, one of which founded a task leaf on two wrong hypotheses (`.25`, TOOLBOX 5.6). ⚠️ **Honest bound, stated before the check was trusted**: tier 1 proves *"nothing that could have changed the artifacts has changed"*, **not** *"the artifacts are correct"* — it inherits whatever tier 2 last established, exactly as `PARSE-COST-RATCHET` says of itself. ⚠️ `generated/` is untracked, so a fresh clone reports **NOT EVALUATED** — loudly, never as a pass; the same for an absent or under-featured `ast_pipeline`, because an under-featured binary cannot generate at all and a loop reading a missing file as *"no difference"* would report a clean pass. **7/7 refusal arms observed firing** (`--self-test`): two GREEN controls, moved emission sources, a moved artifact, an artifact with no recorded row, a baseline carrying no `emission_sha` (which must REFUSE with exit 2 rather than compare against an empty string and pass by construction), and an unknown argument |
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
