# DESIGN-PRIOR-ART: a design leaf proposing a NEW surface must prove no existing one covers it

## Metadata

- Tree ID: `DESIGN-PRIOR-ART`
- Status: `active` (opened 2026-07-26, session #208, by **direct director directive**)
- Family / slice-id prefix: `PGEN-DESIGN-PRIOR-ART-<NNNN>`
- Roadmap lane: cross-cutting **doctrine enforcement** (`DOCTRINE_ENFORCEMENT.md`) —
  turning a standing directive into a mechanical gate.
- Created: `2026-07-26`
- Owner: repo-local workflow

## Provenance

Director, verbatim (2026-07-26, session #208), after `LEX-ADJACENCY.1` proposed a
brand-new `@lexical_token` annotation for a constraint the repo could already express:

> *"`.1`'s `@lexical_token` was reinventing an already-designed surface — that is why
> you should always make you read and understood the existing codebase, book and
> task-trees."*

The directive and its full root-cause analysis (four failures in one session, all
from designing/asserting off a prose summary instead of the source) are recorded in
[[feedback_read_prior_art_before_designing]]. **This tree makes it mechanical**, per
the repo's own standard: *"a doctrine that is not mechanically checked is not
enforced — it is a suggestion."*

## Leaves

### `.1` — The `DESIGN-PRIOR-ART` doctrine check

- **Status: `done`** (`PGEN-DESIGN-PRIOR-ART-0001`, session #208).
- **What it gates.** When a staged `docs/tasks/*.md` **introduces a proposed new
  annotation/directive token** — a backticked `@name` that exists in neither
  `rust/src/ast_pipeline/semantic_directive_registry.rs` nor any `grammars/*.ebnf` —
  that file MUST carry a **`PRIOR ART`** section. Otherwise the commit is blocked.
- **Why that trigger.** It targets the exact failure that occurred: a novel `@name`
  proposed in a task file with no search of the four authoritative sources. It is
  precise (a token either exists in the registry/grammars or it does not), so
  false positives are structurally unlikely.
- **Archetype:** evidence (`DOCTRINE_ENFORCEMENT.md` §3). ⚠️ **Honest limit, stated
  not hidden:** it verifies the search was *recorded*, not that it was *thorough*.
  That is the same bound every evidence-archetype check carries.
- **Cost:** pure `git`/`grep` over staged files; no build, no parser run.

- **Acceptance Checklist (enforced)**
  - [x] **REPRODUCE / ISSUE** — `LEX-ADJACENCY.1` proposed `@lexical_token` and
    landed (commit `f6ff35c8`) with no prior-art search; the surface it invented was
    already designed 2026-06-06 in `project_lexical_annotations_fourth_pillar` and
    documented in the live book as "still to come". Nothing blocked it.
  - [x] **ROOT CAUSE (WHY + WHERE)** — no doctrine covered *design* inputs. The
    registry in `scripts/check_doctrines.sh:41-50` gated diagnosis evidence
    (`TASK-ACCEPTANCE`), memory architecture, EBNF-source-of-truth and five others —
    but nothing required a design leaf to search existing surfaces before proposing
    a new one.
  - [x] **FIX** — new `scripts/check_design_prior_art.sh` (§4 contract: exit-code
    verdict, stderr explanation, deterministic, read-only, staged-scope-aware,
    path-agnostic) + one registry line. Fix tier: **doctrine/process**, no engine
    or grammar change.
  - [x] **ADDRESSED (verified)** — three paths run through the real check (see the
    Verification Log): RED (a staged task file proposing `@totally_novel_directive`
    with no `PRIOR ART` section) → **exit 1**, naming the file and the token; GREEN
    (same file + the section) → **exit 0**; CONTROL (a file citing the EXISTING
    `@quantified_separator` / `@profiles`) → **exit 0**, no false positive.
    ⚠️ Verified with a purpose-built probe, NOT with `LEX-ADJACENCY.md` — that file
    had already been committed, so it had no staged diff to judge. Stated because an
    earlier draft of this box claimed otherwise.
  - [x] **NO REGRESSION** — no `grammars/`, `rust/`, codegen, generated artifact or
    contract touched; the other 8 doctrines re-run PASS unchanged. Zero parse/build
    cost ([[project_capability_growth_is_zero_cost_and_neutral]] — a lint-time check).
  - [x] **LOCKSTEP** — `DOCTRINE_ENFORCEMENT.md` §10 registry table updated;
    decision record added and indexed; `LEX-ADJACENCY.1` retro-fitted with the
    `PRIOR ART` section it should have had.

## Acceptance Criteria (tree)

- A design leaf proposing a new annotation/directive cannot land without a recorded
  prior-art search over: `grammars/ebnf.ebnf`, `docs/decisions/`, `docs/tasks/`,
  `docs/book/`.
- The check is deterministic, read-only, and adds no build or parse cost.
- Red path demonstrated, not asserted.

## Verification Log

All three paths run through the real check, session #208:

| path | input | result |
|---|---|---|
| **RED** | staged task file proposing `@totally_novel_directive`, no `PRIOR ART` section | **exit 1** — names the file AND the novel token, prints the 4-source search order and the motivating case |
| **GREEN** | the same file + a `PRIOR ART` section | **exit 0** |
| **CONTROL** | staged task file mentioning the EXISTING `@quantified_separator` / `@profiles` | **exit 0** — known names do not trip it (no false positive) |

Full driver re-run: `bash scripts/check_doctrines.sh` → **ALL 9 enforced doctrines
PASS** (was 8; this leaf adds the 9th).

⚠️ Recorded honestly: the motivating leaf `LEX-ADJACENCY.1` had ALREADY landed
(commit `f6ff35c8`) before this check existed, so the gate did not block it. Its
`PRIOR ART` section was **retro-fitted** and explicitly marked as such rather than
back-dated — the failure stays visible in the record.

## Changelog

- 2026-07-26 — tree created and `.1` landed (`PGEN-DESIGN-PRIOR-ART-0001`).
