# project — all project-owned data lives on the repository's own volume

**Category:** `project` (standing director directive)
**Stated:** director, session #211 (2026-07-26) — **stated three times, escalating**; restated in the
session bootstrap prompt as its own numbered policy section. Verbatim shape: *all out-of-repo project
data must live on the SAME VOLUME as the repo; no cross-volume access unless provably necessary.*
**Recorded here:** 2026-07-30, session #230, by `README-POLICY.2`.

## ⛔ Why this record exists at all — it had no tracked home

This directive was carried **only** by a line in layer-A `MEMORY.md` and by a *harness-home* memory
file under `~/.claude/…/memory/`. `MEMORY_ARCHITECTURE.md` §12 lists that second location as an
explicit anti-pattern — *"Memory in a harness home directory (lost on tool switch; untracked)"* —
and layer A is overwrite-only, so neither is durable.

Measured while trimming layer A (`README-POLICY.2`): across all tracked markdown, the only mention of
`/Volumes/SSD` outside `MEMORY.md` was a **measurement table row** in
[`feedback_delete_reclaimable_artifacts_regularly.md`](feedback_delete_reclaimable_artifacts_regularly.md),
not the directive. ⭐ **A standing director directive was one `MEMORY.md` overwrite away from being
lost.** That is precisely the failure mode the four-layer architecture exists to prevent, and it is
why a trim must verify a destination before deleting — see [[feedback_read_prior_art_before_designing]].

## The directive

All data **owned by the project** resides on the same filesystem volume as the repository:

- generated outputs, build artifacts, caches;
- package/dependency stores; logs;
- runtime-created test fixtures and temporary workspaces.

Persisted paths are **relative to the repository root**. Tools derive absolute paths at runtime from
the current root, and must never default to `/private/tmp`, `/tmp`, user-home caches, or any other
off-volume location. This is the storage half of the repo-root-relative path rule: the repository
must remain movable — to another directory or another filesystem — without breaking anything.

**Cross-volume access is forbidden** unless it is strictly necessary, explicitly identified,
read-only where possible, and documented with evidence (a required OS/toolchain dependency, or a
caller-authorized input).

## Migrating existing off-volume data — copy / verify / use / delete

1. **Copy or move** it to a repository-derived location on the repository volume.
2. **Verify** file counts, byte sizes, hashes where material, and a successful workflow run *from the
   new location*.
3. **Delete** the exact old data, then run a **residue census** proving it is gone.

⛔ **Never delete an ambiguous shared global cache.** Instead: populate a project-local cache, stop
accessing the shared copy, and remove only records or directories **provably owned by this project**.

## The ruling that was already made — do not re-propose

`~/.cargo` and `~/.rustup` **stay put**, director-ruled. They are shared by every Rust project on the
machine, so they are not "this project's data". Measured at the time: `~/.rustup/toolchains` was
2.1 GB but **bounded** (an update replaces in place); `~/.cargo/registry` was 774 MB and *is*
unbounded (cargo never GCs) — but pinning a project-scoped `CARGO_HOME` means PGEN adds **zero**
further growth to it. `~/.cargo/registry/{src,cache}` is a pure cache, safe to delete, refetched on
demand.

## How it is applied here

- `CARGO_HOME` and `TMPDIR` are pinned to on-volume paths in the **gitignored**
  `.claude/settings.local.json` — deliberately *not* the committed `settings.json`, because those are
  machine-specific absolute paths and the committed file must stay portable.
- The one-time migration moved 5.7 GB of session data off the boot volume, with the old locations
  verified empty afterwards.

## Consequences

- A tool that writes outside the repository volume is a defect, not a preference.
- Any cross-volume read must name its justification in the leaf that introduces it.
- Reclaiming space follows [[feedback_delete_reclaimable_artifacts_regularly]] — a standing
  authorization to sweep, with the *"only when 100% safe"* proof obligation unchanged.

## Honest limit

This record states the directive and the ruling; it is **not** mechanically enforced. No doctrine
check currently asserts that project-owned data sits on the repository volume, so compliance rests on
the discipline above. Mechanizing it (a check that no tracked script defaults to `/tmp`, `$TMPDIR`,
or `$HOME`) is unclaimed work, not a closed loop.
