# PGEN README Stability Policy

> **Adopted 2026-07-30** by direct director order (session #229) from a
> project-neutral *README Stability Policy*. Owned by task tree
> [`docs/tasks/README-POLICY.md`](../tasks/README-POLICY.md).
> Enforced by [`scripts/check_readme_stability.sh`](../../scripts/check_readme_stability.sh),
> registered as a doctrine in [`scripts/check_doctrines.sh`](../../scripts/check_doctrines.sh)
> and mirrored in [`DOCTRINE_ENFORCEMENT.md`](../../DOCTRINE_ENFORCEMENT.md) §10.

This policy keeps `README.md` useful as a **stable landing page** instead of letting
it grow into a changelog, roadmap, gate catalog or documentation inventory.

## Why PGEN adopted it

Measured at adoption (`docs/tasks/artifacts/readme_policy/census_readme_content_classes.sh`):

- `README.md` was **510 lines / 48,811 bytes**.
- **55.6%** of it (240 lines / 27,163 bytes) sat in two sections that are not
  landing-page content at all — an exhaustive file inventory and an operations manual.
- One bullet, `README.md:115`, was **4,369 bytes on a single line** — 9.0% of the whole
  file — and carried a release note, a coverage claim, a schema-migration history and a
  `Done`→`Mostly Done` demotion record, filed under a heading that promises a *path*.
- **6** date-stamped historical annotations were living on the landing page.

⛔ No instrument watched any of this. Every guard that touches `README.md` watches
something else: `check_diagnostics_and_docpaths.sh` audits doc *paths* inside it, and
`ci_workflow_local_gate.sh`'s `audit_root_markdown_surface` audits which root markdown
files *exist*. A README can triple in size with both fully green.

## Content contract

Keep only what a first-time visitor needs:

- purpose, audience, and top-level scope;
- prerequisites and one minimal verified quick start;
- stable architecture at a glance;
- links to canonical documentation, support, and contribution guidance;
- license and other essential repository-level notices.

Route changing detail to its canonical home. **This is the PGEN-specific table** — the
generic policy's classes resolved against this repository's actual documentation surface:

| Content class | Canonical home in PGEN |
|---|---|
| Gate recipes, `make -C rust …` invocations, gate caveats | `docs/book/src/gate-flow.md` |
| Operational procedure (memory guard, hosted-Actions posture, workflow parity) | `docs/book/src/operations-and-governance.md` |
| Repository layout / path inventory | `docs/book/src/developer-architecture.md` § Repository Layout |
| Per-parser book roster and per-book gates | `docs/book/src/parser-families.md` |
| Family status, `Done`-bar claims, demotions | `LIVE_ACHIEVEMENT_STATUS.md` |
| Release history, per-slice change detail | `CHANGES.md`, git history |
| Design rationale and doctrine argument | `docs/decisions/`, `docs/book/src/quality-and-closure-model.md` |
| Deep normative specification | `docs/reference/`, `docs/contracts/` |
| Exhaustive documentation inventories / markdown indexes | `docs/book/src/documentation-model.md`, `docs/book/src/source-map.md` |
| Diagnostics and debug procedure | `TOOLBOX.md`, `docs/book/src/diagnosing-unknowns.md` |
| Current work, priorities, frontier | `docs/tasks/`, `docs/TASK_TREE.md`, `MEMORY.md` |

Change `README.md` only when its **purpose**, **first-use path**, **top-level
architecture**, or **canonical navigation** changes. Ordinary feature work updates the
canonical destination, not the README.

## Mechanical growth guard

Both a line cap and a byte cap are enforced. They are complements, not redundancy:
**neither wrapped prose nor very long lines can bypass the budget.**

| cap | value | measured after the adoption trim | headroom |
|---|---:|---:|---:|
| `README_LINE_CAP` | `220` | 178 lines | ~24% |
| `README_BYTE_CAP` | `10240` | 8,287 bytes | ~24% |

Chosen **after** a deliberate review and trim, leaving modest — and deliberately
*proportional* — headroom, so neither cap is the soft one that absorbs all the growth.
They were not fitted around whatever the file happened to be: the trim came first
(510 → 178 lines, 48,811 → 8,287 bytes; longest line 4,369 → 147 bytes).

⛔ **Never raise a cap to land new content.** Move the detail to its canonical home
instead. A cap increase requires an explicit reviewed decision, recorded in
`docs/tasks/README-POLICY.md`, that the landing-page contract itself expanded.

The check is **non-mutating**, exits nonzero with a routing hint naming the canonical
home, and runs in both the local pre-commit hook (E3) and CI (E4) — it inherits both
lanes automatically by being registered in the doctrine driver, which
`memory-architecture-gate.yml` invokes on every push.

### ⭐ Why both caps, in this repository specifically

PGEN already had the line half of this idea in isolation, and it is measurably bypassed.
`scripts/check_memory_architecture.sh:18-19` caps layer-A `MEMORY.md` at 60 lines and
hard-fails past it. Measured at adoption, 2026-07-30 (the byte figure moves every session —
the invariant is that layer A sits *at* its line cap with no byte bound at all):

| | measured | cap |
|---|---:|---:|
| `MEMORY.md` lines | **60** | 60 — ✅ passes, exactly at the ceiling |
| `MEMORY.md` bytes | **149,779** | *(none)* — ⛔ unbounded |

That is **2,496 bytes per line**. A file described by its own header as a *"bounded
resume pointer … keep ≤ ~50 lines"* is a 150 KB document that satisfies its guard.
The same class was independently found in `README.md:115`. Two instances, no shared
code path ⇒ a defect class in how this repository sizes documents, which is why the
README guard shipped with both caps from day one.

Holding layer A to the same standard is tracked as `README-POLICY.2`.

## Adoption checklist

1. ✅ Remove duplicated status, history, inventories, and deep reference prose.
2. ✅ Verify the retained quick start and links.
3. ✅ Record where each excluded content class belongs (the table above).
4. ✅ Set reviewed line and byte caps with modest headroom.
5. ✅ Commit the deterministic check and wire it into pre-commit and CI.
6. ✅ Require an explicit decision before either cap can increase.
