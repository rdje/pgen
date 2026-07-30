# README-POLICY: keep `README.md` a stable landing page, mechanically

> **Director directive, 2026-07-30 (session #229), verbatim:** *"Once ramp up is
> complete please consider adopting this README policy to avoid having a README.md
> that grow and grow in size /Volumes/SSD/Documents/github/fsmgen/README_POLICY.md"*

## Metadata

- Tree ID: `README-POLICY`
- Status: `active` (opened 2026-07-30, session #229, by direct director order)
- Family / slice-id prefix: `PGEN-README-POLICY-<NNNN>`
- Created: `2026-07-30`
- Owner: repo-local workflow
- Source policy: `/Volumes/SSD/Documents/github/fsmgen/README_POLICY.md` — a
  project-neutral "README Stability Policy". Same volume as this repo, read-only,
  read once at adoption time; the adopted copy is tracked in-repo at
  `docs/reference/PGEN_README_STABILITY_POLICY.md` so PGEN never depends on a path
  outside its own root (CLAUDE.md §12/§13).
- **Current frontier: `.2`** (`.1` done).

## PRIOR ART (per [[feedback_read_prior_art_before_designing]])

Searched before proposing any new surface, in the authority order the doctrine
requires:

| source | searched for | found |
|---|---|---|
| `grammars/ebnf.ebnf` | n/a | not applicable — this is a docs-governance doctrine, not an EBNF-author surface. No new `@annotation` is proposed by this tree. |
| `scripts/` | `README`, `line_cap`, `byte_cap`, `MAX_LINES`, `wc -c` | ⛔ **no size or growth check exists anywhere in the repo.** `check_diagnostics_and_docpaths.sh:52` names `README.md`, but only to audit repo-root-relative doc *paths* inside it. |
| `rust/scripts/` | `README.md`, root markdown allowlist | `ci_workflow_local_gate.sh:277` audits the *set* of root markdown files (`audit_root_markdown_surface`) — **which files exist, never how big any one is.** A README can triple in size with that audit fully green. Complementary, not duplicate. |
| `scripts/check_memory_architecture.sh` | the closest mechanism in-repo | ⭐ **direct prior art for the MECHANISM**: `:9,:18-19` caps layer-A `MEMORY.md` at 60 lines and hard-fails past it. This tree reuses that shape (a cap in the doctrine driver) rather than inventing a new one — **and `.2` records that its line-only form is measurably insufficient.** |
| `docs/decisions/` | readme / documentation governance | no record owns README size. `docs/book/src/documentation-model.md` defines *what belongs in the book*; nothing defines what belongs in the README. |
| `docs/tasks/` | readme / docs surface | `CI-PARITY-GATE-ROT.1` widened the root/`docs` allowlists — again the file *set*, not file *size*. |

⇒ No existing surface caps README growth. This tree is new work, and it plugs into
the existing 14-doctrine driver rather than standing up a parallel mechanism.

## Why this tree exists

`README.md` had grown to **510 lines / 48,811 bytes** with no instrument watching it,
because every instrument that touches it watches something else (paths, file-set).
The policy's diagnosis matches the measurement exactly: the file had become a
changelog, a roadmap, a gate catalog and a documentation inventory wearing a landing
page's name.

---

## Leaves

### `.1` — Adopt the policy: route, trim, cap, enforce (`done`)

- **Status: `done`** (`PGEN-README-POLICY-0001`, session #229, 2026-07-30).
  Touches the **proof surface** (`scripts/check_*.sh`, `rust/scripts/*.sh`) ⇒ a CODE
  change under `check_diagnosis_evidence.sh`'s scoping, so the acceptance checklist
  below is hard-required and enforced. No `grammars/*.ebnf`, no `rust/src/*`, no
  `generated/*` ⇒ **all 11 generated parsers byte-identical BY CONSTRUCTION.**

#### ⭐⭐⭐ THE MEASUREMENT THAT SHAPED THE TRIM — two sections were 55.6% of the file

Census driver (tracked, re-runnable):
`docs/tasks/artifacts/readme_policy/census_readme_content_classes.sh`

| section | lines | bytes | class under the content contract |
|---|---:|---:|---|
| Project Objective | 49 | 3,429 | KEEP (scope) — doctrine prose routes out |
| Canonical Flow | 26 | 3,130 | KEEP (architecture at a glance) — gate detail routes out |
| Fast Ramp-Up | 27 | 1,996 | ✅ KEEP — this *is* the canonical navigation |
| **Key Project Paths** | 43 | **11,181** | ⛔ ROUTE — exhaustive inventory |
| Diagnostic & Debug Toolbox | 9 | 1,615 | KEEP (trim to the pointer) |
| **Standard Commands** | 197 | **15,982** | ⛔ ROUTE — operational procedures |
| Documentation Book | 16 | 795 | KEEP (trim to links) |
| Per-Parser Reference Books | 32 | 2,678 | ⛔ ROUTE — inventory |
| Documentation Status | 29 | 1,611 | ⛔ ROUTE — inventory |
| Documentation Structure | 20 | 1,998 | ⛔ ROUTE — inventory |
| Active Markdown Index | 58 | 4,319 | ⛔ ROUTE — exhaustive index |

⭐ **`Key Project Paths` + `Standard Commands` alone = 240 lines / 27,163 bytes =
55.6% of the file**, and neither is landing-page content: one is a file inventory,
the other an operations manual.

#### ⭐⭐ THE SINGLE LINE THAT PROVES THE POLICY'S POINT — 4,369 bytes, 9.0% of the README

`README.md:115` (the `rtl_frontend/` bullet) measured **4,369 bytes on ONE line**.
It is simultaneously a release note, a contract inventory, a coverage claim, a
schema-migration history and a `Done`→`Mostly Done` demotion record with a date and
a leaf id — filed under a heading that promises a **path**.

⇒ This is why the policy insists on **both** caps. A line-count guard cannot see it:
that bullet is *one line*. Only the byte cap does.

⚠️ It is also **status that goes stale**: it carries `2026-07-29 (DONE-BAR.2b)`
demotion prose that `LIVE_ACHIEVEMENT_STATUS.md` already owns. 6 date-stamped
historical annotations were measured in the README
(`grep -cE '20[0-9]{2}-[0-9]{2}-[0-9]{2}' README.md` → **6**). Every one is a
changelog entry living on a landing page.

#### Routing table — where each excluded class went (adoption checklist step 3)

⛔ Nothing was DELETED without a destination that already existed and already
covered the material. Verified per row before removal:

| removed from README | canonical home | verified present |
|---|---|---|
| Gate catalog / `make -C rust …` recipes + their caveats | `docs/book/src/gate-flow.md` (§1 anatomy, §6 invocation, §8 what stops drift) | 627 lines, §6 "Invocation — who actually runs what" |
| Memory-guard + hosted-Actions operational posture | `docs/book/src/operations-and-governance.md` | §"Host-Resource Governance", §"Workflow Parity" |
| Per-parser book roster + per-book gates | `docs/book/src/parser-families.md` §"Per-Parser Integration Reference Books" | present |
| Family status claims / demotions / release history | `LIVE_ACHIEVEMENT_STATUS.md`, `CHANGES.md` | tracker rows own every claim removed |
| `rtl_frontend` capability + contract prose | `docs/contracts/…`, `docs/rtl_frontend_parser_book/` | both tracked |
| Exhaustive markdown/doc inventories | `docs/book/src/documentation-model.md`, `docs/book/src/source-map.md` | 126 + 123 lines |
| Deep path inventory | `docs/book/src/developer-architecture.md` § Repository Layout | ⛔ **did not exist — CREATED by this leaf** (see below) |
| Doctrine rationale (parser-proof, tracing, annotation) | `docs/book/src/quality-and-closure-model.md`, `docs/reference/…` | present |

⚠️⚠️ **ONE ROW OF THIS TABLE WAS WRONG WHEN FIRST WRITTEN, AND VERIFYING IT IS THE ONLY
REASON IT DID NOT BECOME A SILENT DELETION.** The draft routed `Key Project Paths` (43
lines / 11,181 bytes) to `docs/book/src/source-map.md` on the strength of its *name*.
Measured — `grep -n "grammars/\|rust/src\|generated/" source-map.md` → **0 hits**: that
chapter maps *book chapters to source docs*, not repository paths to their contents. **No
book chapter carried a path inventory at all** (widest hit was `platform-overview.md` with
8 incidental mentions).

⇒ Had the table been trusted instead of checked, 11,181 bytes of the repository's only
path inventory would have been deleted into a chapter that could not receive it. The
section was **created** in `developer-architecture.md` *before* the README was trimmed.
⭐ **A canonical home named from a plausible chapter title is not a verified destination**
— the same "designing from a prose summary instead of the source" shape recorded in
[[feedback_read_prior_art_before_designing]].

#### ⭐ TWO DOC DEFECTS FOUND EN ROUTE, MEASURED AND FIXED (not absorbed silently)

Both were factually false statements about tracked state, found while verifying
destinations — and the first would have made the new section **self-contradictory**:

| where | claimed | measured | action |
|---|---|---|---|
| `docs/book/src/developer-architecture.md` § Generated artifact policy | *"Generated artifacts are tracked on purpose."* | `git ls-files generated/` → **0 files** | corrected in place, with the true property (deterministic codegen) kept |
| `docs/book/src/getting-started.md` § What To Expect | *"generated artifacts are version controlled"* | same | corrected, with the cold-clone regeneration step named |

⛔ The first sat three lines below where this leaf added *"`generated/` … **Not tracked in
git**"*. Shipping the addition without the fix would have published a chapter that
contradicted itself on the same screen.

#### The caps, and why these numbers (adoption checklist step 4)

Set **after** the trim, per the policy (*"Choose them after a deliberate review and
trim, leaving only modest headroom"*):

| cap | value | measured after trim | headroom |
|---|---:|---:|---:|
| lines | `220` | see `.1` evidence | modest |
| bytes | `12288` | see `.1` evidence | modest |

⛔ **A cap is never raised to land content.** Raising either requires an explicit
reviewed decision recorded in this tree that the landing-page contract itself
expanded — enforced socially by that rule and mechanically by the fact that the caps
are constants in a tracked, doctrine-registered script.

#### What shipped

1. `docs/reference/PGEN_README_STABILITY_POLICY.md` — the adopted policy, in-repo,
   with the PGEN routing table and the caps rationale above.
2. `README.md` trimmed to the landing-page contract.
3. `scripts/check_readme_stability.sh` — the deterministic, **non-mutating** guard.
   Enforces both caps and prints a routing hint naming the canonical home on failure.
4. Registered as the **15th doctrine** in `scripts/check_doctrines.sh`, mirrored in
   `DOCTRINE_ENFORCEMENT.md` §10 (the driver's `<meta:mirror>` check fails if the two
   disagree, so they cannot drift).
   ⇒ pre-commit (E3) **and** CI (E4, `memory-architecture-gate.yml`, which auto-runs
   on push since `DONE-BAR.4a`) — both halves the policy asks for, for free.
5. `rust/scripts/ci_workflow_local_gate.sh` — `docs/reference/` allowlist updated for
   the new tracked policy doc (in `sort` order; that list is compared verbatim).

#### ACCEPTANCE CHECKLIST (enforced by `scripts/check_diagnosis_evidence.sh`)

- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow diagnosis family, verbatim
  invocations and their real output:

  **WHY — no README size cap has EVER existed in this repository.** Not "is missing
  today": the pickaxe over *all refs* finds no commit that ever introduced one.

  ```
  $ git log --all -S'README_LINE_CAP' -- scripts/ rust/scripts/ | wc -l
  0
  $ git log --all -S'README_BYTE_CAP' -- scripts/ rust/scripts/ | wc -l
  0
  $ git log --all -S'line_cap'        -- scripts/ rust/scripts/ | wc -l
  0
  $ git log --all -S'byte_cap'        -- scripts/ rust/scripts/ | wc -l
  0
  ```

  **WHERE — the guards that *do* touch `README.md` measure something else.** Every
  tracked enforcer naming the file, enumerated from the index rather than from memory:

  ```
  $ git ls-files 'scripts/check_*.sh' | xargs grep -c 'README\.md'
  scripts/check_diagnostics_and_docpaths.sh:2     <- audits doc PATHS written inside it
  scripts/check_doctrines.sh:1                    <- a header comment
  ```

  plus `rust/scripts/ci_workflow_local_gate.sh:277` (`audit_root_markdown_surface`),
  which compares the *set* of root markdown files. ⇒ **a README can triple in size with
  every one of them green** — the growth is invisible to them by construction, not by
  oversight.

  **MAGNITUDE — the tracked blob size, taken from git rather than the worktree:**

  ```
  $ git cat-file -s HEAD:README.md
  48811
  ```

  510 lines. The census driver attributes **55.6%** (240 lines / 27,163 B) to two
  non-landing-page sections, and `README.md:115` is a **4,369-byte single line**.
- [x] **ADDRESSED (verified)** — before→after on the symptom, measured with the same
  driver: see `.1` evidence block below. The guard is proven RED before GREEN
  (`run_readme_stability_probes.sh`), so it is not vacuous.
- [x] **NO REGRESSION** — measured with everything staged, so the staged-scope
  doctrines actually bound rather than passing vacuously:
  - `scripts/check_doctrines.sh` → **ALL 15 doctrines PASS** + `<meta:mirror>` PASS
    (the mirror check is what proves the registry and `DOCTRINE_ENFORCEMENT.md` §10
    did not drift apart).
  - `scripts/check_flow_integrity.sh --report` → `OK (… all 15 doctrines on the
    automatic lane via the driver …)` — the new doctrine inherited the CI lane by
    construction, which is the invariant `CI-PARITY-GATE-ROT.15` exists to hold.
  - `make -C rust SHELL=/bin/bash mdbook_docs_gate` → exit 0, **10 per-parser book
    gates + the main book** all green (3 book chapters were edited).
  - `bash -n` clean on all 5 edited/added shell scripts; `docs/reference/` allowlist
    verified still in `sort` order (it is compared verbatim).
  - all **25** relative links in the trimmed README resolve; docpath doctrine OK.
  - ⛔ no `grammars/*.ebnf`, no `rust/src/*`, no `generated/*`, no
    `rust/test_data/ast_shape_contract/*` staged ⇒ **all 11 generated parsers
    byte-identical BY CONSTRUCTION** (an assertion about the staged set, not a
    re-measurement). **No tracker row moved.**
- [x] **LOCKSTEP** — `DOCTRINE_ENFORCEMENT.md` §10 mirror row added (driver enforces
  it), book `documentation-model.md` gained the README contract, `CHANGES.md` /
  `DEVELOPMENT_NOTES.md` / `MEMORY.md` / `docs/TASK_TREE.md` updated.

#### Measured before → after

| | before | after | delta |
|---|---:|---:|---:|
| `README.md` lines | 510 | **178** | −65% |
| `README.md` bytes | 48,811 | **8,287** | −83% |
| longest single line | 4,369 B | **147 B** | −97% |
| date-stamped historical annotations | 6 | **0** | — |
| instruments watching README size | **0** | **1** (`README-STABILITY`, automatic tier) | — |

#### Probe results — 9 pass / 0 fail

`docs/tasks/artifacts/readme_policy/run_readme_stability_probes.sh`
(capture: `capture_probes.txt`). Every fixture is built in a synthetic mini-root so the
real README is never touched.

| probe | asserts |
|---|---|
| `GREEN-1` | the live landing page passes |
| ⭐ `CTRL-1` | **the REAL pre-trim `HEAD:README.md` (510 lines / 48,811 B) is REJECTED** — replayed from git, never a lookalike. Without this arm a clean sweep and a blind detector are indistinguishable |
| `RED-1` | line cap rejects (300 lines) |
| ⭐⭐ `RED-2` | **byte cap rejects a fixture that is 22 lines / 18,282 bytes** — the few-lines/enormous-bytes shape |
| ⭐⭐ `CTRL-2` | **a line-only check PASSES that same fixture (22 ≤ 220)** ⇒ the byte cap is load-bearing, not decorative. This is the arm that justifies the design over copying `check_memory_architecture.sh`'s shape |
| `RED-3` | changelog leakage rejected (a verbatim replay of a real removed line) |
| `RED-4` | dropping the policy link rejected — the caps must stay traceable |
| `REFUSE-1/2` | missing README or missing policy ⇒ **exit 2, never 0**. A skip is never a pass |

#### ⚠️ A DEFECT I INTRODUCED TWICE INTO MY OWN SCRIPT, BY DOCUMENTING ITS CORRECTNESS

The driver's vacuity classifier text-matches an enforcer's **source, comments included**,
for the git index-diff flags. The guard reads the working tree, so it should never be
listed as vacuous — yet the driver reported `README-STABILITY … evaluated NOTHING`:

1. the header explained the check does *not* read the index, **naming the flag**;
2. the fix documenting (1) **quoted the classifier's own regex.**

Both revisions tripped it. Third revision names neither and the driver now reports **4 of
15** vacuous, correctly excluding this one. ⭐ **A script can be mislabelled by describing
its own behaviour** when the classifier reads prose as code. Verdicts were never affected
(the driver states this limit honestly in its own comments), so this is a reporting
defect, routed to `.3` — deliberately **not** worked around inside the guard.

#### Evidence

- census: `docs/tasks/artifacts/readme_policy/census_readme_content_classes.sh`
- probes: `docs/tasks/artifacts/readme_policy/run_readme_stability_probes.sh`
- captures: `docs/tasks/artifacts/readme_policy/capture_probes.txt`,
  `capture_census_before_after.txt`

---

### `.2` — ⛔ ROUTED FINDING: the layer-A cap is LINE-ONLY, and it is already bypassed (`todo`)

- **Status: `todo`**. Found by the `.1` prior-art search, **not** by being told.
  Deliberately NOT absorbed into `.1` — the fix is one line, but choosing the cap is
  not, and scope creep on a director-scoped ask is its own defect.

`scripts/check_memory_architecture.sh:18-19` enforces exactly one bound on layer A:

```sh
n=$(wc -l < MEMORY.md)
[ "$n" -le "$CAP" ] || note "MEMORY.md is $n lines (> cap $CAP) …"
```

Measured against the live file, 2026-07-30, at the start of `.1`:

| | measured | cap | verdict |
|---|---:|---:|---|
| `MEMORY.md` lines | **60** | 60 | ✅ PASSES — exactly at the ceiling |
| `MEMORY.md` bytes | **149,779** | *(none)* | ⛔ **unbounded** |

⇒ **2,496 bytes per line on average.** The cap is holding the line count and holding
back nothing.

⚠️ **The byte figure moves every session — do not treat it as a fixed target.** `.1`'s
own commit-workflow update to this file prepended a session entry *and* pruned the
16,664-byte `SESSION #222` line (durably owned by `docs/tasks/CI-PARITY-GATE-ROT.md:922`,
the `DOCTRINE_ENFORCEMENT.md` §10 `FLOW-INTEGRITY` row, and git), leaving **60 lines /
138,403 bytes**. Still at the line ceiling, still unbounded in bytes — a net −11,376 B
that changes nothing structural. ⇒ **The invariant to act on is *layer A sits at its line
cap with no byte bound*, not any particular byte count.** `MEMORY_ARCHITECTURE.md` calls layer A a *"bounded resume pointer"* and
`MEMORY.md:1` says *"keep ≤ ~50 lines"*; the file is a 150 KB document that satisfies
both statements.

⭐ **This is the policy's own rationale reproducing in a second file, in the same
repo, today** — verbatim: *"Line and byte checks complement each other: neither
wrapped prose nor very long lines can bypass the budget."* PGEN implemented the line
half in isolation and got precisely the bypass the policy predicts.

⚠️ **Does it reproduce outside the family?** (the `ROUTING-EVIDENCE` doctrine's central
requirement — `scripts/check_routing_evidence.sh`) — **YES, measured**: the same class was
independently found in
`README.md:115` (4,369 B on one line) with no shared code path, no shared author
convention and no shared cap. Two independent instances ⇒ this is a **defect class in
how PGEN sizes documents**, not a `MEMORY.md` quirk. That is why `.1` shipped **both**
caps rather than copying the existing line-only shape.

⛔ **Do NOT fix by simply adding a byte cap at the current size** — that would ratify
150 KB as "bounded" and is the "raise the cap to land the content" move the policy
forbids. The leaf owes: (1) a deliberate review-and-trim of layer A, (2) a cap chosen
after it with modest headroom, (3) the byte check wired next to the line check, (4) a
RED probe proving it would have caught today's file.

⚠️ Sequencing note: `MEMORY.md` is written by every session's commit workflow, so this
leaf must land in a session that is not mid-batch, or the trim races the next update.

---

### `.3` — ROUTED: the driver's vacuity classifier reads comments as code (`todo`)

- **Status: `todo`**. Reporting-only defect; **no verdict is affected**. Found by `.1`
  tripping it twice (above).

`scripts/check_doctrines.sh` classifies an enforcer as staged-diff-scoped by grepping its
**whole source file**, comments included, for the git index-diff flags. Any enforcer that
merely *mentions* those flags — including one explaining that it does not use them — is
reported as having *"evaluated NOTHING"*.

⚠️ **Does it reproduce outside this family?** (the `ROUTING-EVIDENCE` requirement)
— **Not yet, measured**: of the 15 registered enforcers, the classifier's verdict was
checked against each one's actual behaviour and only `check_readme_stability.sh` was
mislabelled, because it is the only one that discusses index-scoping without doing it. So
today the blast radius is one script. ⛔ But the trigger is *writing prose about scoping*,
which costs nothing and will recur — the classifier's own comment already concedes the
heuristic ("a mislabel costs an inaccurate NAME … never a verdict").

The leaf owes an adjudication, not a presumed fix:

- **Option A** — strip comments before matching (`sed 's/#.*//'`). Cheap; but a heredoc or
  a quoted string still fools it, so it narrows the hole rather than closing it.
- **Option B** — have each enforcer *declare* its scope (a machine-readable header line the
  driver reads), replacing inference with a stated fact. Correct, but touches all 15.
- **Option C** — accept it and document it, since it never changes a verdict.

⛔ Do **not** default to A because it is smallest: the driver's own founding principle is
*a check that cannot see must SAY SO, not return green*, and an inference that silently
mislabels is the same shape one level up. Price B before choosing.

---

## Acceptance Criteria (tree)

- [x] The policy is adopted in-repo, not referenced across a volume boundary.
- [x] Every content class removed from `README.md` has a named, verified canonical home.
- [x] Both caps enforced, non-mutating, nonzero-with-routing-hint on failure.
- [x] Wired into pre-commit **and** CI via the existing doctrine driver.
- [ ] `.2` — layer A held to the same standard the README is now held to.

## Evidence

`docs/tasks/artifacts/readme_policy/`
