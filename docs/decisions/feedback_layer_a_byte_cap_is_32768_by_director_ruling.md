# Layer A's byte cap is 32,768 by director ruling — raised for HEADROOM, never to land content

- **Date:** 2026-08-14
- **Status:** current (standing directive)
- **Owning work:** `docs/tasks/README-POLICY.md` leaf `.8` (`PGEN-README-POLICY-0008`)
- **Supersedes:** the `7168` value set by `README-POLICY.2` — not its reasoning, only its number
- **Related:** [[a-cap-with-no-headroom-is-a-cap-about-to-be-raised]] · `MEMORY_ARCHITECTURE.md` §6/§9

## Context

`README-POLICY.2` gave layer A (`MEMORY.md`) a byte cap beside its line cap, after measuring
that the line-only form had passed a **60-line / 138,403-byte** file — a *"bounded resume
pointer"* that was a 138 KB document with a green guard. The caps were set **after** a trim, at
`50 lines / 7168 bytes`, being the post-trim 39 lines / 5,720 bytes plus ~28 % and ~25 % headroom.

That headroom was subsequently spent. Measured on 2026-08-14 from layer A's own git history
(`git rev-list` + `git cat-file -s`, `git diff --numstat`):

| measurement | value |
|---|---|
| last 40 layer-A commits — min / median / mean / max bytes | 6,795 / 7,079 / 7,063 / **7,168 = the cap exactly** |
| commits at ≥ 97 % of the byte cap | **36 of 40** |
| commits within 68 bytes of the cap | 18 of 40 |
| headroom at the moment of the ruling | **117 bytes** (7,051 of 7,168) |
| updates whose net size change alone exceeded 117 bytes | 3 of the last 20 |
| `git diff --numstat HEAD~9 HEAD -- MEMORY.md` | **`4 4`** |

The last row is the diagnosis. Over nine consecutive commits the same four lines were rewritten
in place and the file never grew: the cap had stopped bounding the **layer** and started editing
the **prose**. Authors were shaving bytes to fit rather than deciding what belongs in layer A —
the opposite of the cap's purpose, reached by a different route than the 138 KB failure but with
the same root, *a bound that is satisfied without binding on the thing it is meant to bind*.

## Decision

**Director ruling, 2026-08-14:** the layer-A byte cap is **32,768**
(`MEMORY_POINTER_BYTE_CAP`, `scripts/check_memory_architecture.sh`). The line cap is unchanged
at **50**.

The governing distinction, which this record exists to keep unambiguous:

> **A cap is never raised to LAND CONTENT. A cap may be raised to RESTORE HEADROOM, by an
> explicit reviewed decision, taken while the file is PASSING.**

`MEMORY_ARCHITECTURE.md` §6 forbids the first and explicitly permits the second — *"a cap
increase should require an explicit reviewed decision, recorded in the work-tracking system,
that the layer-A contract itself changed."* This ruling is that decision, and it was taken at
the calm moment the project's own lesson card asks for: layer A was **passing**, not blocked.
Once a cap has actually blocked you, the two acts are indistinguishable — which is the reason to
decide early rather than the reason to decide differently.

### What did NOT change

- **The layer-A contract itself.** Layer A holds where-we-are-NOW and nothing else; it is
  OVERWRITE-only; history is git (D), per-unit work is `docs/tasks/` (B), durable facts are
  `docs/decisions/` (C). The raise buys headroom, not permission to accumulate.
- **The portable standard's default.** `MEMORY_ARCHITECTURE.md` §9's reference script keeps
  `MEMORY_POINTER_BYTE_CAP:-7168`, which is the right default for a fresh adopter with a
  freshly-trimmed pointer. 32,768 is *this deployment's* reviewed budget, not a new
  recommendation, and propagating it would export a number without the measurement that earned it.
- **Either cap's ability to fire.** Both were re-proven RED after the change
  (`MEMORY_POINTER_BYTE_CAP=7000` → rc 1; `MEMORY_POINTER_LINE_CAP=20` → rc 1). The caps were
  raised, not disabled.

## Consequences

⚠️ **Honest structural cost.** At `50 lines / 32768 bytes` the byte axis permits ~655 B/line, so
across the 7–32 KB band the **line cap is the only binding axis** and the two-axis design
degrades toward the single-axis form `README-POLICY.2` replaced. The byte cap keeps its original
job — making the 138,403-byte outcome impossible — but it is no longer co-binding at pointer
shape. This is stated rather than discovered later; see the lesson card's own second warning.

⭐ **Mitigation: report headroom, not just compliance.** `scripts/check_memory_architecture.sh`
now prints both axes' utilisation on every passing run:

```text
memory-arch: OK (layer A 7051/32768 bytes = 21% of cap, 30/50 lines = 60% of cap)
```

Layer A sat at ≥ 97 % of cap for 36 consecutive commits and nothing said so, because a passing
gate printed the same three characters at 5,720 bytes as at 7,168. The report adds no failure
path and cannot change a verdict — the caps remain the sole gate — but the metric that predicts
the next failure is now visible on every commit instead of only when the gate finally fires.

⛔ **The next raise needs its own record.** Two changes to this value now exist and both carry a
reviewed decision (`.2` introduced it, `.8` raised it). A third with no such record is the
anti-pattern, regardless of how the number is justified.
