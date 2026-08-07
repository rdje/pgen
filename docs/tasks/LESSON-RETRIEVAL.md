# LESSON-RETRIEVAL: durable lessons must be RETRIEVABLE, not merely written down

## Metadata

- Tree ID: `LESSON-RETRIEVAL`
- Status: `active` (opened 2026-08-01, session #230, by DIRECTOR DIRECTIVE)
- Roadmap lane: continuity / knowledge capture — layer C+ of `MEMORY_ARCHITECTURE.md`
- Created: `2026-08-01`
- Owner: repo-local workflow

## Why this tree exists (the director's words)

Reviewing `PGEN-SV-EXH-PROOF-0166`, the director asked of a sharp conclusion —
*"A branch is covered if and only if its generation need not produce a `property_expr`"* —

> *"I hope all of these types of conclusion you reach from time to time … are stored, saved,
> accessible somewhere (book? task-tree? KM?)"*

and then, decisively:

> *"Not just this one conclusion, you made a lot of those conclusions, these are, as I wrote,
> **lesson-learned material**."*

⛔ The honest answer was **no** — they are *written down* but not *retrievable*.

## The measured gap (2026-08-01, not estimated)

| surface | count | question-retrievable? |
|---|---:|---|
| `DEVELOPMENT_NOTES.md` dated lesson entries (`^## 2026`) | **1 592** | ❌ **no** — 62 191 lines, and the file is **not a KM scan dir** |
| `docs/decisions/` records | **142** | ❌ **no** — `grep -l '^answers:' docs/decisions/*.md` = **0** |
| `docs/knowledge/` fact cards | **35** | ✅ yes — the ONLY indexed surface |

Reproduce:

```bash
grep -c '^## 2026' DEVELOPMENT_NOTES.md
ls docs/decisions/*.md | grep -v INDEX | wc -l ; grep -l '^answers:' docs/decisions/*.md | wc -l
ls docs/knowledge/*.md | wc -l
```

⭐ **THE STRUCTURAL POINT.** `KNOWLEDGE_MAP.md` is auto-derived (`knowledge-map/scripts/gen_knowledge_map.sh`)
from `KM_SCAN_DIRS = docs/knowledge docs/decisions`, and *"a fact is any `.md` whose front-matter has a
non-empty `answers:` list"*. So:

1. `DEVELOPMENT_NOTES.md` is **outside the scan dirs entirely** — 1 592 lessons can never be indexed
   from where they live.
2. `docs/decisions/` **is** scanned, but its 142 records use a different front-matter shape
   (`name:` / `description:` / `metadata:`) with **no `answers:`**, so every one of them is invisible
   to the question index. The scan dir is wired; the records simply do not opt in.

⇒ The retrieval surface is not broken and does not need replacing. **Almost nothing has been promoted
into it.** 35 of 1 769 candidate items are reachable by question — and the promotion step exists
nowhere in the commit workflow, so the omission is silent and repeats every session.

⚠️ This is the `LIVE-MEANS-LIVE`/`README-POLICY` shape one layer over: a surface that grows without
bound (`DEVELOPMENT_NOTES.md`, 62 191 lines) beside a bounded, gated surface nobody routes into.

## Goal

Make a durable lesson **retrievable by the question it answers**, and make the promotion step
impossible to skip silently — so a future session (or a different harness) finds the lesson by asking,
instead of by knowing which of 1 592 dated entries to read.

## Non-Goals

- **NOT** migrating all 1 592 notes. Most are correctly per-slice HISTORY (layer D) and must stay
  chronological. Promotion is for the DURABLE + GENERAL subset.
- Not replacing `DEVELOPMENT_NOTES.md`, `docs/decisions/` or the KM bundle — the bundle is portable
  and already gate-enforced (derive-and-diff); this tree ROUTES INTO it.
- Not auto-generating fact cards from prose. ⛔ A generated "lesson" is a claim, not a fact
  ([[feedback_generate_facts_not_claims]] shape) — every card carries a `reverify:` command a human or
  a gate can re-run.

## The promotion criterion (proposed — a lesson is KM-PROMOTABLE iff)

1. **Durable** — still true after the slice that found it lands. *(A measured residual partition is
   PERISHABLE and belongs in its task leaf; the mechanism it revealed is durable.)*
2. **General** — reusable beyond the one grammar/family it surfaced in.
3. **Re-verifiable** — a concrete `reverify:` command re-proves it from the current tree.
4. **Question-shaped** — you can write ≥1 natural question a future session would actually ask.

A lesson failing (1) stays in its task leaf. Failing (2) stays in `DEVELOPMENT_NOTES.md`. Failing (3)
is not yet a fact — it is a hypothesis.

## Acceptance Criteria

- The criterion above is recorded as a durable decision record and is itself KM-indexed.
- A measured back-fill of the highest-value existing lessons (scope = director call, `.2`).
- `docs/decisions/` records become reachable by question (they are already in a scan dir).
- The promotion step is enforced, not remembered — a lesson cannot be written to
  `DEVELOPMENT_NOTES.md` without a promote-or-decline decision being recorded.
- `check_knowledge_map.sh` stays green throughout (it already runs in `check_doctrines.sh`).

## Task Tree

- ID: `LESSON-RETRIEVAL`
  Status: `active`
  Goal: durable lessons are retrievable by question, and promotion cannot be silently skipped
  Children: `.1`, `.2`, `.3`, `.4`

- ID: `LESSON-RETRIEVAL.1`
  Status: `done` (`PGEN-LESSON-RETRIEVAL-0001`)
  Goal: `MEASURE the gap and PROVE the mechanism end-to-end on this session's own lessons, so the tree opens with a working exemplar rather than a plan. Promote the two DURABLE+GENERAL lessons from -0165/-0166 into docs/knowledge/ fact cards with answers: + reverify:, regenerate the map, and confirm the doctrine gate stays green.`
  Acceptance: `the gap table above is reproducible by the pasted commands; >=2 new KM facts, each with a reverify: command that actually runs; KNOWLEDGE_MAP.md regenerated by its own generator (never hand-edited); scripts/check_doctrines.sh KNOWLEDGE-MAP green.`
  Verification: `see Verification Log`
  Commit: `PGEN-LESSON-RETRIEVAL-0001`

- ID: `LESSON-RETRIEVAL.2`
  Status: `pending` — ✅ **SCOPE DECIDED BY DIRECTOR 2026-08-01: "Decisions + top-N sweep."** Do `.3`
  in full (all 142 decisions), plus the ~40-60 `DEVELOPMENT_NOTES` entries that state a general,
  evidence-backed rule. ⛔ NOT the full 1 592-entry audit.
  Goal: `BACK-FILL the existing corpus. The inventory is 1 592 dated DEVELOPMENT_NOTES entries + 142 decision records. Three sizings, each honest about cost: (A) FORWARD-ONLY -- promote from here on, back-fill nothing; cheapest, but ~1 700 existing items stay unreachable. (B) TOP-N SWEEP -- back-fill the highest-value N (e.g. the ~40-60 entries that state a GENERAL rule and already carry evidence), one slice per batch. (C) FULL AUDIT -- read all 1 592 and adjudicate each; large, and most will correctly decline. ⛔ Do NOT start a sweep before the director sizes it: (C) is a multi-session lane and would pull hard against the standing "prefer feature work over governance lanes" directive.`
  Acceptance: `pending the scope call`
  Verification: `pending`
  Commit: `pending`

- ID: `LESSON-RETRIEVAL.3`
  Status: `pending`
  Goal: `MAKE docs/decisions/ REACHABLE BY QUESTION. All 142 records already sit in a KM scan dir; none carries answers:, so none is indexed. The change is additive front-matter (answers: + reverify:), not a rewrite -- and it is the cheapest large win available, because the records are ALREADY curated, already durable, and already the layer-C authority. ⛔ VERIFY FIRST, do not assume: confirm the KM generator tolerates the decisions front-matter shape (name/description/metadata) alongside answers:, and that adding answers: to a decision does not change its rendering anywhere else.`
  Acceptance: `pending`
  Verification: `pending`
  Commit: `pending`

- ID: `LESSON-RETRIEVAL.4`
  Status: `done` (`PGEN-LESSON-RETRIEVAL-0003`) — ⭐ **THE LEAK IS CLOSED.** `LESSON-PROMOTION` is the repo's **17th** enforced doctrine; both directions proven on the real script.
  Verification: `done -- scripts/check_lesson_promotion.sh, registered in scripts/check_doctrines.sh and mirrored into DOCTRINE_ENFORCEMENT.md §10 (meta:mirror now reports "exactly the 17 registered doctrines"); ALL 17 PASS.
  ⭐ BOTH DIRECTIONS PROVEN END-TO-END, not asserted: docs/tasks/artifacts/lesson_retrieval/run_lesson_promotion_probes.sh drives the REAL shipping script against a scratch git repo with genuine staged index states (so `git diff --cached` is actually exercised), 6/6 PASS --
  * blocked_no_decision exit 1 -- a new dated lesson with neither promotion nor decline is BLOCKED;
  * promoted_knowledge exit 0 -- a docs/knowledge/ card satisfies it;
  * promoted_decision exit 0 -- `answers:` added to a docs/decisions/ record satisfies it;
  * declined_token exit 0 -- an explicit `promotion: declined (<reason>)` in a task leaf satisfies it;
  * ⭐ edit_not_new_lesson exit 0 and unrelated_commit exit 0 -- the TWO FALSE-POSITIVE CONTROLS. Editing an existing entry is not a new lesson, and a commit that never touches DEVELOPMENT_NOTES.md is untouched. A gate that fires on those would be waived within a week.
  GROUND TRUTH IS ALSO INSIDE THE INSTRUMENT: lesson_promotion_self_check drives 7 pinned cases through the shipping decision function on EVERY invocation and exits 2 on a miss ([[feedback_instrument_needs_ground_truth]]).
  ⚠️ HONEST LIMIT, stated in the script's own header rather than hidden: this verifies a DECISION WAS RECORDED, not that it was correct -- a lazy `promotion: declined (n/a)` passes. What becomes impossible is the SILENT omission, which is the measured failure (1592 times).`
  Commit: `PGEN-LESSON-RETRIEVAL-0003`
  Decision_note: `✅ **DIRECTOR DECIDED 2026-08-01: build the DOCTRINE CHECK** (`scripts/check_lesson_promotion.sh`, wired through `check_doctrines.sh` into the pre-commit hook + CI), not a `COMMIT.md` reminder. Rule: a staged dated `DEVELOPMENT_NOTES.md` entry requires either a `docs/knowledge/` change or an explicit decline token in the owning task leaf.
  Goal: `ENFORCE THE PROMOTION STEP so it cannot be silently skipped -- the actual root cause, since the mechanism has existed and been ignored for 1 592 entries. Candidate designs to PRICE, not to assume: (a) a doctrine check that a commit adding a DEVELOPMENT_NOTES "## <date> - <slice>" entry also touches docs/knowledge/ OR records an explicit decline token in the task leaf; (b) a COMMIT.md workflow step; (c) a periodic drift report counting unpromoted general lessons. ⛔ (b) alone is a reminder, not an enforcer -- and this repo has measured that reminders lose: the toolbox directive needed a git-level hook before it held (DOCTRINE_ENFORCEMENT.md). Prefer (a).`
  Acceptance: `pending`
  Verification: `pending`
  Commit: `pending`

- ID: `LESSON-RETRIEVAL.5`
  Status: `pending` (opened 2026-08-08 by `PGEN-SV-EXH-PROOF-0181`, which hit it twice in one session)
  Goal: `Give the DECLINE token the same wrapped-prose tolerance WAIVER-ROUTING already has. check_lesson_promotion.sh matches `promotion: declined (<reason>)` with a single-line grep (`grep -E "${DECLINE_TOKEN} \(..*\)"`), so a decline written as ordinary wrapped markdown -- opening paren on one line, closing paren three lines later -- does NOT satisfy it. That is the IDENTICAL hazard check_waiver_routing.sh diagnosed and solved with a +/-6-line window, and its source comment already records why: "Markdown prose WRAPS ... A strict same-line rule is unsatisfiable for any wrapped paragraph and would push authors toward deleting the waiver instead of owning it -- the exact outcome this doctrine exists to prevent." The same argument applies verbatim to a decline: the doctrine wants a DECISION recorded, and an author who cannot express one in normal prose will reformat until the grep is happy or stop writing declines.`
  Prerequisites: `NONE.`
  Design sketch (do NOT re-derive): `Reuse check_waiver_routing.sh's proven shape -- locate the trigger line, then search a small window around it for the discharge -- rather than inventing a second mechanism. Here the trigger is `promotion: declined` and the discharge is a non-placeholder reason, so the window search is for a closing paren plus non-empty content within N lines. ⛔ KEEP THE PLACEHOLDER REJECTION: `.4` deliberately refuses the literal `promotion: declined (<reason>)`, because its own first real-world run passed on a leaf that merely DOCUMENTED the token in prose about the gate. A window must not reopen that.`
  Acceptance: `A wrapped multi-line decline with a real reason SATISFIES the gate; the literal `(<reason>)` placeholder still FAILS; a decline in an UNRELATED file still fails; and all 6 existing arms of docs/tasks/artifacts/lesson_retrieval/run_lesson_promotion_probes.sh stay green. Add a 7th arm for the wrapped case -- RED before the change, GREEN after.`
  Evidence: `Both encounters were in PGEN-SV-EXH-PROOF-0180 and -0181, and ⛔ IN BOTH THE BLOCK WAS CORRECT ON THE MERITS -- a real KM card was owed each time and was written (deterministic-artifacts-sort-at-the-serializer, measure-a-policy-where-its-outcome-is). So this is a USABILITY defect in how a decline is expressed, NOT evidence the gate is too strict, and it must not be used to argue for weakening it.`
  Verification: `pending`
  Commit: `pending`

## Leaf `.3` progress — 2026-08-01 (`PGEN-LESSON-RETRIEVAL-0002`), 11 of 142 promoted

⛔ **THE "VERIFY FIRST" CLAUSE PAID OFF TWICE — neither obstacle was guessable from the plan.**

1. **A decision needs FIVE new fields, not one.** `check_knowledge_map.sh` requires `id`, `title`,
   `date` and at least one of `evidence`/`reverify` on anything carrying `answers:`. Decision records
   have `name` / `description` / `metadata` — none of those four. Adding `answers:` alone would have
   turned all 142 into *validation failures*, not facts.
2. ⭐ **81 of 142 records are KM-INVISIBLE BY CONSTRUCTION.** The KM's parser only opens front matter
   when `---` is on **line 1** (`NR==1` in `check_knowledge_map.sh`'s awk). 81 records begin with
   something else — 54 with the 2026-06-02 migration provenance comment, the rest with a heading. For
   those, `answers:` could never have worked at any point. Fix is mechanical and lossless: move the
   front matter to line 1 and keep the provenance comment verbatim immediately below it.

**Done this slice:** the 10 most-cited records (ranked by `[[wikilink]]` citations across `docs/`,
`MEMORY.md`, `CHANGES.md` — top of the distribution at 127, 126, 90, 88, 79, 66, 61, 60, 53, 47)
plus the pilot. 7 of the 11 also needed normalizing. **KM 35 → 48 facts, 241 → 301 question keys**;
`check_knowledge_map.sh` OK; all 16 doctrines PASS.

⭐ **A promoted card's `reverify:` immediately earned its keep** — see the finding routed to
`EBNF-SOURCE-OF-TRUTH.md` this same slice: writing a runnable re-verification for
`project_ebnf_is_single_source_of_truth` surfaced two live shared-codegen breaches of that very
doctrine, one of which overrides a grammar's declared regex pattern. **A `reverify:` is not
documentation; it is a tripwire that runs.** That is the strongest argument yet for finishing `.3`.

**Remaining for `.3`:** 131 records, of which ~74 still need the line-1 normalization. Batch them by
citation rank — the tail is mostly single-citation records where a thin `answers:` list is honest.

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `LESSON-RETRIEVAL.2` | `pending` | ⏳ blocked on the director's scope call (A / B / C) — the only open question in this tree |
| 2 | `LESSON-RETRIEVAL.3` | `pending` | cheapest large win: 142 already-curated records sit in a scan dir and just need `answers:` |
| — | `LESSON-RETRIEVAL.4` | `done` | ⭐ the root-cause fix LANDED — `LESSON-PROMOTION` is doctrine #17, 6/6 probe cases green |
| — | `LESSON-RETRIEVAL.1` | `done` | gap measured, mechanism proven on two real lessons |

## Acceptance Checklist (enforced) — leaf `.1`

- [x] **REPRODUCE / ISSUE** — `grep -c '^## 2026' DEVELOPMENT_NOTES.md` → **1592**;
  `ls docs/decisions/*.md | grep -v INDEX | wc -l` → **142** against
  `grep -l '^answers:' docs/decisions/*.md | wc -l` → **0**; `ls docs/knowledge/*.md | wc -l` → **35**.
  The conclusion the director asked about was findable only in `docs/tasks/SV-EXH-PROOF.md`,
  `CHANGES.md` and `docs/TASK_TREE.md` — no question-indexed surface carried it.
- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow family. `knowledge-map/scripts/knowledge_map.conf`
  pins `KM_SCAN_DIRS := docs/knowledge docs/decisions`, and `check_knowledge_map.sh` defines a fact as
  *"front-matter has a non-empty `answers:` list"*. ⇒ two independent misses, both silent:
  `DEVELOPMENT_NOTES.md` is **not in the scan dirs at all**, and `docs/decisions/` **is** scanned but
  **no record opts in** with `answers:`. The retrieval surface works; nothing routes into it, and no
  gate notices — `KNOWLEDGE-MAP` checks the map is *in sync with its sources*, never that a lesson
  *reached* a source.
- [x] **FIX** — fix-hierarchy tier: **declarative, additive**. Two new `docs/knowledge/` fact cards
  promoting this session's durable lessons, each with `answers:` + a runnable `reverify:`; map
  regenerated by its own generator (never hand-edited). No script, gate or engine change in this leaf —
  the enforcement question is deliberately left to `.4` so this leaf proves the mechanism first.
- [x] **ADDRESSED (verified)** — before→after on the symptom: KM facts **35 → 37**, question keys
  **241 → 254**. Both new cards' `reverify:` commands were executed and reproduce their claim (see
  Verification Log). The `-0166` conclusion is now reachable by asking *"is `selected_but_failed` about
  the parser or the generator"* instead of by knowing which task leaf to open.
- [x] **NO REGRESSION** — `scripts/check_doctrines.sh` ALL 16 PASS, including `KNOWLEDGE-MAP`
  (derive-and-diff: the committed map equals a fresh regeneration, so drift is impossible);
  `MEMORY-ARCH` byte+line caps green. Docs-only — no code, grammar, generated artifact, contract or
  gate touched.
- [x] **LOCKSTEP** — this tree + `docs/TASK_TREE.md` index row + `KNOWLEDGE_MAP.md` (regenerated) +
  `CHANGES.md` + `DEVELOPMENT_NOTES.md` + `MEMORY.md`.

## Acceptance Checklist (enforced) — leaf `.4` lesson-promotion gate

- [x] **REPRODUCE / ISSUE** — `grep -c '^## 2026' DEVELOPMENT_NOTES.md` → **1592** dated lesson
  entries in a file that is not a Knowledge Map scan dir, beside
  `grep -l '^answers:' docs/decisions/*.md | wc -l` → **0** of 142 records in a dir that IS scanned.
  The promotion mechanism was wired and skipped every time, silently.
- [x] **ROOT CAUSE (WHY + WHERE)** — **ops/build-flow family** (a shell/enforcer defect: no rustc
  error and no parse to trace). Census by `git ls-files 'docs/decisions/*.md' | wc -l` → **142**
  tracked records against `git ls-files 'docs/knowledge/*.md' | wc -l` → **35**, with
  `git ls-files | grep -c '^DEVELOPMENT_NOTES.md'` → 1 file holding 1592 dated entries and **not**
  listed in `knowledge-map/scripts/knowledge_map.conf`'s `KM_SCAN_DIRS`. `bash -n` on the enforcer
  roster confirms the registry in `scripts/check_doctrines.sh` had **no** entry covering promotion.
  WHERE: `knowledge-map/scripts/check_knowledge_map.sh` asserts only *"the committed map equals a
  fresh regeneration"* — in sync with its SOURCES, never that a lesson REACHED a source. So the
  omission was invisible in the PASSING direction — the same shape as the closed-loop residual being
  `echo`ed but never compared (`SV-EXH-PROOF.7.4.6.10`), one layer up.
- [x] **FIX** — fix-hierarchy tier: **ops/enforcer, evidence archetype**. New
  `scripts/check_lesson_promotion.sh`, registered in `scripts/check_doctrines.sh` and mirrored into
  `DOCTRINE_ENFORCEMENT.md` §10. It gates the **decision**, not the outcome: a promotion (a
  `docs/knowledge/` card, or `answers:` on a decision record) OR an explicit
  `promotion: declined (…)` both pass.
- [x] **ADDRESSED (verified)** — named re-runnable oracle:
  `bash docs/tasks/artifacts/lesson_retrieval/run_lesson_promotion_probes.sh` drives the REAL script
  against a scratch git repo with genuine staged index states, **7/7 PASS**: `blocked_no_decision`
  exit 1; `promoted_knowledge` / `promoted_decision` / `declined_token` exit 0;
  ⭐ `documentation_mention` exit 1; and the two false-positive controls `edit_not_new_lesson` /
  `unrelated_commit` exit 0. Plus 7 in-script controls that exit 2 on a miss, every invocation.
  Before→after: a lesson could be dropped silently; it now cannot.
- [x] **NO REGRESSION** — `scripts/check_doctrines.sh` **ALL 17 PASS**, `<meta:mirror>` reports
  *"exactly the 17 registered doctrines"* (registry and human mirror agree), `bash -n` clean on both
  new scripts, and `shellcheck`-class review of the quoting defect is pinned by the
  `documentation_mention` probe case. The generated parsers for the six fully-certified grammars are
  **byte-identical**: `git diff --cached --name-only` lists only `docs/`, `scripts/`,
  `DOCTRINE_ENFORCEMENT.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md` and
  `KNOWLEDGE_MAP.md` — **zero** paths under `rust/src/`, `grammars/` or `generated/`, so no codegen
  ran and no parser artifact could move. The two false-positive probe controls prove ordinary
  commits are untouched.
- [x] **LOCKSTEP** — this tree + `DOCTRINE_ENFORCEMENT.md` §10 + `scripts/check_doctrines.sh`
  registry + `CHANGES.md` + `DEVELOPMENT_NOTES.md` + `MEMORY.md`.

⛔ **TWO DEFECTS IN THIS GATE, FOUND AND FIXED BEFORE IT LANDED — recorded rather than quietly
patched.** Both were caught by running it for real rather than by reading it:
1. **The gate was satisfied by its own documentation.** Its first live run PASSED on this very leaf,
   because the leaf *mentions* `promotion: declined (<reason>)` while explaining the gate. A mention
   is not a decision. Fixed by requiring a non-empty reason and rejecting the literal placeholder;
   pinned forever as the `documentation_mention` probe case.
2. **The probe case for (1) initially passed while testing nothing** — its prose contains backticks,
   and inside a nested `bash -c "…"` those became command substitution, so the token was never
   written and the case passed on an empty file. Fixed by staging through a shell function with a
   `grep -q` setup assertion that REFUSES if the fixture was not written as intended. ⇒ **a probe
   needs its own ground truth; "the case passed" is not evidence that the case ran.**

## Decisions

- **The KM bundle is not the problem and is not being replaced.** It is portable, gate-enforced by
  derive-and-diff, and already wired into `check_doctrines.sh`. This tree is a ROUTING problem.
- **Perishable findings stay in their task leaf.** The `property_expr` partition that prompted this
  tree is exactly that: it becomes FALSE the moment `.7.4.6.9` lands. What got promoted is the durable
  mechanism underneath it (reason codes are generator verdicts), not the measurement.

## Open Questions

1. ⏳ **Back-fill scope (`.2`) — DIRECTOR CALL**: forward-only (A), top-N sweep (B), or full audit (C)?
2. Should a promoted lesson's `status:` decay? A card claiming a *current* engine behaviour can rot;
   `reverify:` is the guard, but nothing re-runs it on a schedule. Related: `LIVE-DOC-CURRENCY`.

## Blockers

- None for `.1`/`.3`/`.4`. `.2` awaits the scope call.

## Verification Log

- `2026-08-01` leaf `.1` (`PGEN-LESSON-RETRIEVAL-0001`): gap measured (1592 / 142 / 0 / 35);
  two fact cards added; `KNOWLEDGE_MAP.md` regenerated 35 → 37 facts, 241 → 254 question keys;
  both `reverify:` commands run and reproduce; `scripts/check_doctrines.sh` ALL 16 PASS.

## Commit Log

| Leaf | Commit | Summary |
| --- | --- | --- |
| `.1` | `PGEN-LESSON-RETRIEVAL-0001` | gap measured + mechanism proven on two real lessons (KM 35 → 37) |

## Changelog

- `2026-08-01` — tree opened by director directive after `PGEN-SV-EXH-PROOF-0166`.
