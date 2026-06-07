<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_cargo_sweep_cadence.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: Run cargo sweep regularly
description: Per user direction (repeated 2026-05-13 and 2026-05-15), run `cargo sweep --time 1` from `rust/` whenever it can run safely. Keeps target/ from bloating; safe between cargo invocations (current build artifacts under 1 day stay).
type: feedback
originSessionId: f74f4acc-7183-408a-ae5d-dcdce15a103c
---
**When to run:**

- After any cycle that did multiple `cargo build` / `cargo test` invocations.
- After regenerating any generated parser (since parser-rebuild leaves old artifacts).
- Between major task lanes when the next lane will trigger fresh compilation anyway.
- Whenever a sweep would naturally fit between cargo operations and won't interrupt a build in flight.

**When NOT to run:**

- Mid-build (the running compile holds its own artifacts; sweep won't touch them but interleaving I/O is wasteful).
- If `cargo sweep` itself isn't installed (it's a separate cargo subcommand) — surfaces as `error: no such subcommand`, in which case skip and note for the user.

**How to apply:**

```bash
cd rust && cargo sweep --time 1   # from the repo root
```

`--time 1` keeps artifacts newer than 1 day. The user prefers this conservative threshold so a fresh release-mode probe build (~5 minutes) doesn't get cleaned when iterating on a slice.

Don't ask before running — the user has asked twice now. Just run it when the moment fits.
