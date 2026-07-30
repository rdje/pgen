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
- Source policy: a project-neutral "README Stability Policy" authored in a sibling repo on
  the same volume; read-only, and **copied in** rather than referenced across a boundary, so
  PGEN never depends on a path outside its own root (CLAUDE.md §12/§13). Two tracked
  artifacts result, and the distinction is load-bearing:
  - **`README_POLICY.md`** (repo root) — the **neutral standard**, verbatim, beside the other
    portable standards (`MEMORY_ARCHITECTURE.md`, `DOCTRINE_ENFORCEMENT.md`, `TOOLBOX.md`).
    Placed there by director order (`.4`); the policy's own `## Storage location` section now
    requires exactly this.
  - **`docs/reference/PGEN_README_STABILITY_POLICY.md`** — PGEN's **instance**: the resolved
    routing table, the reviewed cap values, the adoption evidence.
  ⚠️ The source is a *live* file in another repo and it changed mid-session — re-`cmp` before
  claiming a copy is verbatim (`.4`).
- **Current frontier: none — tree work is complete for now** (`.1`, `.2`, `.4`, `.5`, `.7` done; `.3`, `.6` routed `todo`, waiting behind product).

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

### `.2` — the layer-A cap is LINE-ONLY, and it is already bypassed (`done`)

- **Status: `done`** (`PGEN-README-POLICY-0002`, session #230, 2026-07-30). Found by the
  `.1` prior-art search, **not** by being told. Touches the **proof surface**
  (`scripts/check_*.sh`, `rust/scripts/*.sh`) ⇒ a CODE change under
  `check_diagnosis_evidence.sh`'s scoping, so the acceptance checklist below is
  hard-required and enforced. No `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` ⇒
  **all 11 generated parsers byte-identical BY CONSTRUCTION.**

#### ⭐⭐⭐ THE MEASUREMENT THAT SHAPED THE TRIM — the block that says "do not append" held 81.3% of the file

Census driver (tracked, re-runnable, self-calibrating):
`docs/tasks/artifacts/readme_policy/census_layer_a_ownership.sh`

| block | lines | bytes | share |
|---|---:|---:|---:|
| header (the layer-A contract statement) | 7 | 365 | 0.3% |
| `## How to resume` | 6 | 664 | 0.5% |
| `## North star` | 12 | 24,911 | 18.0% |
| **`## Current state (OVERWRITE this block each update — do not append)`** | 35 | **112,463** | **81.3%** |

⭐ **The heading is the finding.** That block carried **18 distinct sessions** (#202 → #229)
under an instruction to overwrite it. The single largest entry was **18,816 bytes on one
line**. ⇒ the growth was not "long prose"; it was *history accumulating in the one place
the architecture says history must never go*.

#### ⛔ THE DECIDING QUESTION THE LEAF REFUSED TO ASSUME — *is this content owned anywhere else?*

`.1`'s most expensive lesson was the routing row it got wrong: `Key Project Paths` was sent
to `source-map.md` on the strength of that chapter's NAME, and measuring found **0** path
hits there. ⭐ **A canonical home named from a plausible title is not a verified
destination.** Layer A is the same shape at 34× the scale, so ownership was **measured per
entry**, not assumed:

| verdict | entries | basis |
|---|---:|---|
| ✅ owned in layer D + B | **20** | every `PGEN-<FAMILY>-<NNNN>` slice id resolves to a commit subject **and** its `docs/tasks/<FAMILY>.md` exists |
| ⚠️ unanchored → resolved individually | **14** | no slice id; each checked against a named destination (below) |
| ⛔ **no durable home anywhere** | **2** | **written to `docs/decisions/` BEFORE removal** |

**The 2 with no home — the entire reason the census existed:**

1. **The same-volume storage directive** — a director directive *stated three times,
   escalating*, whose "durable record" was a **harness-home memory file**, i.e. the exact
   anti-pattern `MEMORY_ARCHITECTURE.md` §12 names (*"lost on tool switch; untracked"*).
   Measured: across all tracked markdown the only other mention of `/Volumes/SSD` was a
   **measurement table row**, not the directive. → `docs/decisions/project_data_locality_same_volume.md`.
2. **`[[project_bedrock_spine_repo]]` was a DANGLING link** — that string appeared in
   **exactly one place in the whole repository: `MEMORY.md` itself** (0 hits elsewhere).
   → `docs/decisions/project_bedrock_spine_repo.md`.

⭐ **A `[[link]]` is a promise, not a proof — a dangling one marks content that is LESS safe
to prune, not more.**

**Verified destinations for the other 12 unanchored entries** (measured, not named):

| entry | destination | measured |
|---|---|---|
| director work order #218→#220 | `docs/tasks/CI-PARITY-GATE-ROT.md` | 4 leaves present; its "next frontier" list was **stale** |
| `QUANT-PLUS-ITER` + the trace-engine trap | `docs/tasks/QUANT-PLUS-ITER.md` + `TOOLBOX.md` §2.1 | trap landed verbatim, 2 hits |
| mimalloc landing mechanism | `docs/tasks/RGX-0078.md` | 63 hits |
| host-RAM directive incl. the 16384 recalibration | `docs/decisions/feedback_host_ram_budget_all_jobs.md` | carries `16384` |
| speed landed-history deltas | `docs/book/src/speed-journey.md` | 38 delta tokens |
| no-parser-hooks ruling | `docs/tasks/PARSER-NEUTRALITY.md` + its record | both present |
| on-disk / build-tree state | `DEVELOPMENT_NOTES.md`, `docs/tasks/OPS-MEMSAFE.md`, `TOOLBOX.md` | 5 / 3 / 1 hits |
| deferred list | `docs/tasks/REGEX-PCRE2-FIDELITY.md`, `MCP-CONTROL.md`, `STIMULI-SIGNOFF.md` | 9 hits + trees |
| cert axis / `locked_program` | `LIVE_ACHIEVEMENT_STATUS.md` | 142 hits (the *label* is layer-A shorthand; the content is owned) |
| push cadence rule | `docs/decisions/feedback_push_pacing.md` | 8 hits on `300` |

⚠️ **ONE ENTRY WAS DELIBERATELY KEPT**: the regex oracle tuple **`2189/1879/262/48`**. The
`REGEX-ORACLE-ANCHOR-SYNC` doctrine reads `MEMORY.md` as one of four anchors. Its check is
*"every occurrence (if any)"*, so removing the line would still have **passed** — and that
is precisely why it stays: dropping it would have silenced an anchor while the gate went on
reporting green. *A check that can no longer see must not be made to look satisfied.*

⚠️⚠️ **MY OWN INSTRUMENT WAS WRONG TWICE, AND BOTH TIMES A CONTROL CAUGHT IT — not review.**

1. **The census reported `owned-in-git=0` for all 34 entries**, including
   `PGEN-README-POLICY-0001`, which is the subject of `HEAD`. Root-caused rather than
   patched: `PIPESTATUS=(141 0)` — `grep -qF` **matched** (0) and exited at line 1 of 2,679,
   the upstream `printf` still holding ~549 KB against a ~64 KiB pipe buffer took
   **SIGPIPE (141)**, and `set -o pipefail` promoted 141 to the pipeline's status.
   ⭐ **The polarity is inverted exactly where it hurts: the earlier the match — i.e. the
   more recently the slice was committed — the more reliably it is reported NOT owned.**
   Reproduced **5/5**; a bug that reproduces every time is the opposite of flaky. Fixed the
   prescribed way (grep a FILE, so there is no upstream writer to kill). ⛔ This class was
   **already root-caused in this repository** at `scripts/check_diagnosis_evidence.sh:101-109`
   (`STORE-AWARE-GEN.4b.12`) and fixed *there only* — see the routed leaf `.6`.
2. **A destination verified as `0 hits` was a false negative from my own grep**: the speed
   deltas are written with **U+2212 MINUS**, my pattern used ASCII hyphen. Had it been
   trusted, `docs/book/src/speed-journey.md` would have been declared unable to receive
   content it already holds 38 times. ⭐ Same shape as `.1`'s `source-map` row, reached from
   the opposite direction — *the proxy, not the destination, was wrong.*

#### The caps, and why these numbers

Set **after** the trim, per the policy (*"Choose them after a deliberate review and trim"*):

| cap | old | new | measured after trim | headroom |
|---|---:|---:|---:|---:|
| lines (`MEMORY_POINTER_LINE_CAP`) | 60 | **50** | 39 | ~28% |
| bytes (`MEMORY_POINTER_BYTE_CAP`) | *(none)* | **7168** | 5,720 | ~25% |

⭐ The line cap is **lowered**, not raised: `MEMORY_ARCHITECTURE.md` §6 and `MEMORY.md`'s own
header both said *"≤ ~50 lines"*, so the enforcer's 60 was looser than the rule it policed.
At 50 lines the byte cap allows ~143 B/line — the shape of an actual pointer file — so the
two caps bind at the same style of document rather than one shadowing the other.

#### ⭐⭐ THE DEFECT WAS IN THE PORTABLE STANDARD, NOT ONLY IN THIS DEPLOYMENT

`MEMORY_ARCHITECTURE.md` §9's own reference check prescribed
`CAP="${MEMORY_POINTER_LINE_CAP:-60}"` with no byte bound, and §6 said *"Size cap — keep it
to roughly one screen"*. ⇒ **every project adopting the standard inherited the same
bypass.** Fixing only PGEN's copy would have left the defect in the artifact meant to be
reused. §6, §9 and §9.1 were corrected together.

#### What shipped

1. `MEMORY.md` trimmed to a genuine resume pointer (39 lines / 5,720 bytes).
2. `scripts/check_memory_architecture.sh` — byte cap beside the line cap (E2.2), line cap
   60→50, with the measured rationale in-source.
3. `MEMORY_ARCHITECTURE.md` §6 / §9 / §9.1 — the portable standard now specifies **both**
   caps and names the measured bypass.
4. `docs/decisions/project_data_locality_same_volume.md` + `project_bedrock_spine_repo.md`
   — the two homeless facts, **created before the removal**.
5. `docs/decisions/INDEX.md` — the 2 new rows **plus 2 pre-existing unindexed records**
   found by the census (`feedback_stimuli_gen_rejects_valid_blindspot`,
   `project_json_rfc8259_full_standard_commitment`). Layer C is now **137 records / 137
   index rows / 0 unindexed**.
6. `DOCTRINE_ENFORCEMENT.md` §10 + the driver registry descriptions, in lockstep.
7. `docs/book/src/documentation-model.md` — new *Layer A is capped the same way* section.

#### ACCEPTANCE CHECKLIST (enforced by `scripts/check_diagnosis_evidence.sh`)

- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow diagnosis family, verbatim invocations
  and their real output. **WHY — the layer-A guard has NEVER had a byte bound**; the
  pickaxe over *all refs* finds no commit that introduced one:

  ```
  $ git log --all -S'MEMORY_POINTER_BYTE_CAP' -- scripts/ MEMORY_ARCHITECTURE.md | wc -l
  0
  $ git log --all -S'wc -c < MEMORY.md'       -- scripts/ MEMORY_ARCHITECTURE.md | wc -l
  0
  ```

  **WHERE — `scripts/check_memory_architecture.sh` E2.2, which measured lines only:**

  ```
  $ git show HEAD:scripts/check_memory_architecture.sh | sed -n '17,19p'
  if [ -f MEMORY.md ]; then
    n=$(wc -l < MEMORY.md)
    [ "$n" -le "$CAP" ] || note "MEMORY.md is $n lines (> cap $CAP) ..."
  ```

  **MAGNITUDE — taken from git, not the worktree:**

  ```
  $ git show HEAD:MEMORY.md | wc -l ; git show HEAD:MEMORY.md | wc -c
  60
  138403
  ```

  60 lines against a 60-line cap = **PASSING**, at 2,306 bytes/line, longest line 18,816 B.
- [x] **ADDRESSED (verified)** — measured before→after with the same census driver, and the
  fix proven non-vacuous by **re-executing the retired guard**:
  `git show HEAD:scripts/check_memory_architecture.sh` run against the **real** pre-trim
  `MEMORY.md` prints `memory-arch: OK` over 138,403 bytes (probe `CTRL-2`), while the new
  guard rejects the identical file on bytes (`CTRL-1`). Probes **8 pass / 0 fail**
  (`run_layer_a_cap_probes.sh`). Layer A: **60→39 lines, 138,403→5,720 bytes.**
- [x] **NO REGRESSION** — measured with everything staged, so the staged-scope doctrines
  actually bind rather than passing vacuously:
  - `bash scripts/check_doctrines.sh` → **ALL 15 doctrines PASS** + `<meta:mirror>` PASS
    (the mirror proves the registry and `DOCTRINE_ENFORCEMENT.md` §10 did not drift apart
    while both descriptions were rewritten).
  - `bash scripts/check_regex_oracle_anchor_sync.sh` → `OK (tuple 2189/1879/262/48
    consistent across live anchors + tracked bounds)` — the doctrine that reads this very
    file still agrees after the rewrite.
  - `make -C rust SHELL=/bin/bash mdbook_docs_gate` → exit 0, **10 per-parser book gates +
    the main book** green.
  - the root-markdown allowlist audit re-run standalone → PASS with `README_POLICY.md`
    admitted; `bash -n` clean on all 4 edited/added shell scripts.
  - layer C reconciled 137/137, **0 unindexed**; every `[[link]]` in the new `MEMORY.md`
    resolves to an existing record.
  - ⛔ no `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` staged ⇒ **all 11 generated
    parsers byte-identical BY CONSTRUCTION** (an assertion about the staged set, not a
    re-measurement). **No tracker row moved.**
- [x] **LOCKSTEP** — `MEMORY_ARCHITECTURE.md` (the standard itself), `DOCTRINE_ENFORCEMENT.md`
  §10, the driver registry, `docs/book/src/documentation-model.md`, `docs/decisions/INDEX.md`,
  `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `docs/TASK_TREE.md`.

#### Measured before → after

| | before | after | delta |
|---|---:|---:|---:|
| `MEMORY.md` lines | 60 | **39** | −35% |
| `MEMORY.md` bytes | 138,403 | **5,720** | **−95.9%** |
| bytes per line | 2,306 | **146** | −94% |
| longest single line | 18,816 B | **401 B** | −98% |
| distinct `SESSION #N` entries in layer A | 18 | **0** | — |
| caps bounding layer A | **1** (lines) | **2** (lines + bytes) | — |
| layer-C records unindexed | 2 | **0** | — |

#### Probe results — 8 pass / 0 fail

`docs/tasks/artifacts/readme_policy/run_layer_a_cap_probes.sh`

| probe | asserts |
|---|---|
| `GREEN-1` | the trimmed live pointer passes |
| `CTRL-4` | the harness is **not vacuously green** — removing an unrelated doctrine leg still fails, so `GREEN-1` means the guard RAN |
| ⭐ `CTRL-1` | the **real** pre-trim `HEAD:MEMORY.md` (60 lines / 138,403 B) is REJECTED **on bytes** |
| ⭐⭐ `CTRL-2` | the **real retired guard**, extracted with `git show HEAD:`, **PASSES that same file** and prints `memory-arch: OK` — the change is proven necessary by execution, not by argument. Asserts the success line, not merely exit 0, because a guard that died early would also be 0 |
| `RED-1` | the line cap rejects (80 short lines) |
| ⭐ `RED-2` | the byte cap rejects **10 lines / 12,121 bytes** — the few-lines/enormous-bytes shape |
| ⭐ `CTRL-3` | a line-only check **PASSES** that same fixture (10 ≤ 50) ⇒ the byte cap is load-bearing, not decorative |
| `ABSENT-1` | a missing pointer is still reported (the pre-existing leg is intact) |

#### Evidence

- census: `docs/tasks/artifacts/readme_policy/census_layer_a_ownership.sh` (self-calibrating,
  5/5 arms, **REFUSES** its own output on a calibration miss)
- probes: `docs/tasks/artifacts/readme_policy/run_layer_a_cap_probes.sh`
- captures: `docs/tasks/artifacts/readme_policy/capture_layer_a_census.txt`,
  `capture_layer_a_probes.txt`

---

### `.4` — put the project-NEUTRAL `README_POLICY.md` at the repository root (`done`)

- **Status: `done`** (`PGEN-README-POLICY-0002`, session #230, 2026-07-30, **BY DIRECT
  DIRECTOR ORDER** mid-session: *"I noticed that you did not copy the policy .md file in PGEN
  root directory"*, clarified *"I mean README_POLICY.md"*).

`.1` adopted the policy as `docs/reference/PGEN_README_STABILITY_POLICY.md` — PGEN's
**instance**, carrying the resolved routing table, the chosen caps and the adoption
evidence. What it did **not** do was keep the project-neutral **standard** itself in the
tree.

⭐ **The director's placement is the architecturally correct one, and it matches a pattern
this repo already follows**: the repository root holds the *portable standards*
(`MEMORY_ARCHITECTURE.md`, `DOCTRINE_ENFORCEMENT.md`, `TOOLBOX.md`), while `docs/` holds
each one's PGEN-specific instance. `README_POLICY.md` is a portable standard and belongs
beside its siblings. **Standard at the root; instance under `docs/`.**

- Copied **verbatim** from the sibling repo that authored it. Per the data-locality rule it
  is **copied in, never referenced across a repository boundary** (both repos sit on the same
  volume, and the source was read-only).
- ⚠️ **THE SOURCE MOVED MID-SLICE, AND ONLY RE-MEASURING CAUGHT IT.** The first copy was
  2,425 B; minutes later the same path measured **2,920 B** — the author had added a
  **`## Storage location`** section and a new adoption step 1, both stating exactly what the
  director had asked for in words: *"Store the adopting project's canonical copy as the
  git-tracked `<repository-root>/README_POLICY.md`, alongside `README.md`"*, and *"a
  user-home, machine-global, or other external copy … must not replace the project-owned
  repository copy."* ⇒ the placement is now **normative in the policy itself**, not just a
  preference. The stale copy was refreshed in `PGEN-README-POLICY-0003`.
  ⭐ **A verbatim copy is only verbatim as of a measurement** — `cmp -s` was re-run rather
  than assumed, which is the only reason the drift was seen at all. All three copies now hash
  identically (`091e6922a97b…`), and PGEN satisfies the new clause: tracked, at the repo root,
  beside `README.md`.
- `rust/scripts/ci_workflow_local_gate.sh` — `audit_root_markdown_surface` compares the root
  markdown set **verbatim**, so the allowlist gained `README_POLICY.md`.
  ⚠️ **Position was MEASURED, not reasoned**: it sorts **before** `README.md` (exactly as
  `MEMORY_ARCHITECTURE.md` sorts before `MEMORY.md`), which ASCII collation would have got
  backwards. Verified by running the audit's own pipeline, then the audit itself → PASS.
- `docs/reference/PGEN_README_STABILITY_POLICY.md` now opens by declaring itself the
  instance and linking the root standard, so neither can be mistaken for the other.
  ⛔ Keep the root copy neutral: if a PGEN noun would have to appear in it, it belongs in
  the instance instead.

---

### `.5` — port the README stability policy to `bedrock` (`done`)

- **Status: `done`** (bedrock `BEDROCK-MAINTENANCE.2.1`, commits `e3cb82b` + `5f0a7dc`),
  opened and closed 2026-07-30 session #230 **BY DIRECT DIRECTOR ORDER**: *"Please update the
  bedrock with this new README.md policy whenever you can. bedrock repo is located here
  /Volumes/SSD/Documents/github/bedrock."*
- ✅ **LANDED IN BEDROCK, under its own discipline** (task-tree leaf first, `make gate`,
  `COMMIT.md`): `README_POLICY.md` + a neutralized `scripts/check_readme_stability.sh`
  registered as its **4th doctrine**; the layer-A byte cap added to *its*
  `check_memory_architecture.sh` **and** to `MEMORY_ARCHITECTURE.md` §6/§9/§9.1; both new
  files added to the `update_scaffold.sh` NEUTRAL allow-list so existing consumers can pull
  them; `DOCTRINE_VERSION` **0.1.0 → 0.2.0**; `CHANGELOG.md` noted. Gate **6/6 green**.
- ⛔ **THE PORT CONFIRMED THE DIAGNOSIS AT THE SOURCE**: bedrock's own layer-A check was
  `lines=$(wc -l < MEMORY.md)` with a cap of **120** and **no byte bound** — looser than the
  deployment that failed — and its `MEMORY_ARCHITECTURE.md:129/249/315` carried the same
  line-only form. ⇒ every project adopting bedrock was inheriting the bypass. The control arm
  was re-run there too: bedrock's **retired** guard returns exit 0 over a 19,304-byte fixture
  the new one rejects.
- ⭐ **CAPS DELIBERATELY DIFFER FROM PGEN'S, and that is a real design point**: a template's
  caps ship to a README that is *not* bedrock's, so they default to the policy's own published
  example (300 lines / 16384 bytes) with the policy instructing the consumer to tighten after
  their own trim. Only the **layer-A** cap was tightened (120 → 50), because the standard
  already said *"≤ ~50 lines"*.
- ⭐⭐ **REVERSE-FLOW FINDING, brought back — see `.7`**: bedrock's layer-C check is *stronger*
  than PGEN's. The documented flow is *"PGEN → generalize → bedrock"*; this went the other way.
- ⛔ **Sequencing was not discretionary**: `bedrock` is a **separate repository**, so working
  there is a pivot, and the pivot rule forbids pivoting while this tree's repo is dirty. So
  `.2`/`.4` landed and committed first (`2db1e18a`), and only then did the port start — the
  director's *"whenever you can"* explicitly permitted that ordering.

⭐ **This port carries MORE than the README policy, and that is the whole point.** `bedrock`
is where the portable spine is maintained ([[project_bedrock_spine_repo]]), and `.2` proved
the spine itself shipped the defect: `MEMORY_ARCHITECTURE.md` §9's reference check
prescribed a **line-only** cap, so every adopter inherits a bound that does not bind. So the
port owes **two** things:

1. `README_POLICY.md` — the neutral README stability policy (the root copy, already
   verbatim-identical to the source, so it ports unchanged).
2. The **layer-A both-caps correction** to `MEMORY_ARCHITECTURE.md` §6 / §9 / §9.1 —
   otherwise bedrock keeps handing new projects the bypass PGEN just measured.

Per the porting discipline: strip any domain noun, route through bedrock's own
`BEDROCK-MAINTENANCE` tree, keep `update_scaffold` neutral, bump `DOCTRINE_VERSION` — all
done, and each step verified against bedrock's **actual** tree rather than assumed. Neutrality
was measured, not asserted: `grep -ciE 'pgen|grammar|parser|ebnf|systemverilog|regex'` over the
ported policy → **0**, and the guard cites its evidence as *"a real project running this
spine"* so the number carries the argument without naming a domain.

---

### `.6` — ROUTED: `pipefail` + `grep -q` fails OPEN in two doctrine enforcers (`todo`)

- **Status: `todo`**. Found by `.2`'s census tripping the class on itself (above).

Under `set -o pipefail`, `producer | grep -q PATTERN` returns **failure on success** once
the producer's output exceeds the pipe buffer and the match is early: `grep -q` exits at the
first match, the producer takes SIGPIPE, and `pipefail` promotes 141. Measured directly:
`PIPESTATUS=(141 0)`.

⛔ **The repository already root-caused this class and fixed it in exactly one place** —
`scripts/check_diagnosis_evidence.sh:101-109` documents it verbatim (`STORE-AWARE-GEN.4b.12`,
*"this race began false-failing once the owning task leaf grew past ~64 KB"*). Two enforcers
still carry the idiom, with **opposite failure polarities**:

| site | polarity | consequence |
|---|---|---|
| `scripts/check_waiver_routing.sh:72` (`printf "$added" \| grep -qE "$WAIVER_RE" \|\| continue`) | ⛔ **fails OPEN** | the file is silently **skipped** ⇒ a newly-added waiver goes unchecked |
| `scripts/check_waiver_routing.sh:89` (`sed window \| grep -qE "$OWNER_RE"`) | fails CLOSED | a discharged waiver reported undischarged (loud, not silent) |
| `scripts/check_design_prior_art.sh:55` (`printf "$known_names" \| grep -qx`) | fails OPEN | a known `@name` misreported as novel — small producer, low exposure |

⚠️ **LATENT, NOT LIVE — priced before routing.** Across the last **120** commits the largest
`$added` for a `docs/tasks/*.md` file is **28,720 bytes**; **0** commits in the last 40 exceed
the threshold. So no verdict is *currently* untrustworthy, which is why this is routed rather
than worked now per [[feedback_flow_findings_are_routed_not_worked]]. ⛔ But the trend is the
wrong way — task leaves only grow — and the fail-open site is the one that goes **silent**.

⚠️⚠️ **CORRECTION TO THIS LEAF'S OWN NUMBER — the threshold is NOT a flat ~64 KiB**, as first
written here. Measured directly on this platform, same producer/consumer as the real site:

```
  65,606 B  ->  PIPESTATUS=(0 0)     no SIGPIPE — the writer fits in the pipe + grep's read-ahead
 131,139 B  ->  PIPESTATUS=(141 0)   SIGPIPE — the fail-open
```

⇒ the effective bound is pipe capacity **plus whatever the consumer buffers before exiting**, so
the real headroom is *larger* than this leaf originally claimed and the routing decision is
**more** clearly right, not less. ⭐ Found because a probe built on the flat-64 KiB assumption
**failed to reproduce the defect** — the fixture, not the theory, was wrong. *A number quoted
from a mental model is not a measurement.*

✅ **THE FIXED IMPLEMENTATION NOW EXISTS — ADOPT IT, DO NOT WRITE IT.** `WAIVER-ROUTING` was
ported to the spine repo (`BEDROCK-MAINTENANCE.2.2`) and the fail-open was **fixed on the way in
rather than inherited**: both sites write to a file so there is no upstream writer to kill. That
version carries a `CTRL-1` probe in which the shipped form CATCHES a 343,376-byte staged addition
while the unfixed pipe form MISSES it (exit 0). ⇒ this leaf is now an **adoption**, the second
instance of transfer running backwards in one session — see [[project_bedrock_spine_repo]].

⚠️ **Does it reproduce outside the family?** (`ROUTING-EVIDENCE`) — **YES, measured**: the
idiom appears in **18** tracked scripts that set `pipefail`, of which **3 are doctrine
enforcers**; and the class was independently hit by `.2`'s own census with no shared code
path. That is what makes it a defect class rather than one script's bug.

---

### `.7` — the layer-C index check is satisfied without binding (`done`)

- **Status: `done`** (`PGEN-README-POLICY-0004`, session #230). Found by `.2`'s census, and it is the
  **same disease as `.2` itself, one check down in the same file.**

`scripts/check_memory_architecture.sh` E2.5 asserts only that `docs/decisions/INDEX.md`
lists **more than zero** rows when records exist:

```sh
idx_rows=$(grep -cE '^\| \[' docs/decisions/INDEX.md || true)
[ "$idx_rows" -gt 0 ] || note "... has $rec_count records but INDEX.md lists none (out of sync)"
```

Measured at the start of `.2`: **135 records / 133 index rows** — two records
(`feedback_stimuli_gen_rejects_valid_blindspot`, `project_json_rfc8259_full_standard_commitment`)
were **invisible to layer C's own index while the doctrine reported OK**. Its in-source
comment is honest about this (*"a cheap in-sync sanity check, not a full reconcile"*), so
this is an accepted bound rather than a hidden bug — but it is the identical shape the byte
cap just closed: **a bound satisfied without binding.**

`.2` reconciled the data (137/137, 0 unindexed) but deliberately did **not** change the
check — the fix is a real reconcile (every record has a row, every row a record), and
choosing its failure mode is its own decision, not a drive-by edit. ⚠️ Note the trap: a
naive reconcile must not be self-referential about `INDEX.md`, per
`reference_self_referential_assertion_is_unsound.md`.

⭐⭐ **THE IMPLEMENTATION ALREADY EXISTS — IN `bedrock`, AND IT ARRIVED BY THE FLOW RUNNING
BACKWARDS.** Found while executing `.5`: bedrock's `scripts/check_memory_architecture.sh:23-29`
already iterates **every** record under `docs/decisions/` and fails unless each is listed in
`INDEX.md` — the exact record→row direction PGEN is missing. bedrock's `MAINTAINING.md` states
the flow as *"PGEN → (generalize) → bedrock"* and describes PGEN as *"the reference
implementation / proving ground"*; here the spine was **ahead of its own reference
deployment**, and PGEN's `>0` check would have gone on passing indefinitely. ⇒ `.7` should
**adopt bedrock's implementation** rather than design one, and the tree that owns the
relationship ([[project_bedrock_spine_repo]]) should record that transfer is **bidirectional**
— a claim its own porting discipline does not currently make. ⚠️ Honest bound, carried over
from the bedrock leaf: that check is **one-directional** — it catches a record with no row,
never a row with no record.

#### What shipped — ADOPTED, then strengthened twice

E2.5 is now a real reconcile, and both strengthenings were **measured, not stylistic**:

1. **Row-anchored**, not a bare basename grep. bedrock's form matches the filename *anywhere*
   in `INDEX.md`, so a record with no row of its own still passes if a neighbouring row's prose
   names it. ⚠️ That shape is **already present** in PGEN's live index:
   `project_json_full_standard_proof.md` occurs **twice** (its own row + a cross-reference in
   another row). No live false-pass today; the hazard is structural and one edit away.
2. **Bidirectional** — a row naming a record that no longer exists is an index that lies in the
   other direction. That closes the half bedrock's version is honest about not covering.

⇒ the improved form was then **sent back to bedrock**, so the spine keeps the better one. See
[[project_bedrock_spine_repo]] — transfer runs both ways, director-confirmed.

#### Probe results — 6 pass / 0 fail

`docs/tasks/artifacts/readme_policy/run_layer_c_reconcile_probes.sh`

| probe | asserts |
|---|---|
| `GREEN-1` | a reconciled index passes |
| `RED-1` | a record with **no row** is rejected |
| `RED-2` | a row naming a **missing record** is rejected |
| ⭐ `CTRL-1` | the **retired** check, executed from `git show HEAD:`, reports `memory-arch: OK` with a record missing from the index ⇒ it was genuinely blind |
| ⭐⭐ `CTRL-2` | a **bare-basename** matcher PASSES a record hidden behind a neighbour's prose mention while the shipped row-anchored form REJECTS ⇒ the anchoring is load-bearing, not stylistic |
| `CTRL-3` | removing `INDEX.md` still fails ⇒ the harness is not vacuously green |

#### ACCEPTANCE CHECKLIST (enforced by `scripts/check_diagnosis_evidence.sh`)

- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow family, real invocations:

  ```
  $ git show HEAD:scripts/check_memory_architecture.sh | sed -n '/E2.5/,/^fi$/p' | grep idx_rows
    idx_rows=$(grep -cE '^\| \[' docs/decisions/INDEX.md || true)
    [ "$idx_rows" -gt 0 ] || note "... INDEX.md lists none (out of sync)"
  ```

  ⇒ the bound was *"more than zero rows"*, so it could not see a partial index. Measured at
  the start of `.2`: **135 records / 133 index rows**, doctrine green, two records unindexed.
- [x] **ADDRESSED (verified)** — `git ls-files docs/decisions/ | wc -l` and the row extractor
  now agree at **137 / 137**, and the guard REJECTS both failure directions. Proven
  non-vacuous by re-executing the retired check from `git show HEAD:`, which reports
  `memory-arch: OK` over a fixture holding an unindexed record (`CTRL-1`). Probes **6/0**.
- [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh` → **ALL 15 doctrines PASS** +
  `<meta:mirror>`; layer A 39/50 lines, 6,267/7,168 bytes; `bash -n` clean on the enforcer and
  the new probe driver. ⛔ no `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` staged ⇒
  **all 11 generated parsers byte-identical BY CONSTRUCTION.** No tracker row moved.
- [x] **LOCKSTEP** — `DOCTRINE_ENFORCEMENT.md` §10 `MEMORY-ARCH` row,
  `docs/decisions/project_bedrock_spine_repo.md` (transfer runs both ways), `CHANGES.md`,
  `DEVELOPMENT_NOTES.md`, `MEMORY.md`.
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
