# feedback_exit_safety_explicit_banner — every end-of-turn banner states /exit-safety EXPLICITLY and LOUDLY

- **Category:** feedback (standing director directive)
- **Date:** 2026-07-16 (session #133, RGX-0078 `.5.i.7` RE-PROFILE #13)
- **Status:** ACTIVE, non-negotiable

## Context

The RE-PROFILE #13 STEP-2 probe build (a ~35-min non-LTO profiling compile) was
externally killed THREE times (2× at 323s, 1× at 2084s), always with healthy
memory, always `INT/TERM` into the memory guard — the sender was unidentifiable
from the host. The director then resolved it: **they had run `/exit` after the
session reported itself safe/parked — never a deliberate kill.** Root cause:
**session exit tears down the session's process tree, including harness-tracked
background jobs** (the harness delivers INT/TERM to the whole tree; the memory
guard records it as `guard-interrupted`, exit 130). A "safe to end the session"
signal that only considers the git tree is therefore WRONG whenever a
long-running background job must survive.

## Decision (director, verbatim intent)

> "Tell me explicitly, next time, when it is 100% safe to /exit without risking
> your probe build. Be explicit and loud in whatever you want or need."

1. **Every end-of-turn status banner MUST carry an explicit EXIT-SAFETY line**,
   one of exactly two forms, impossible to miss:
   - `✅ 100% SAFE to /exit` — no live background job of any kind, OR
   - `⛔ DO NOT /exit (or /clear) — <job> in flight, ~<ETA>; I will say loudly when it is safe`
2. A live background/probe job makes the session **exit-UNSAFE by definition**,
   even when the git tree is clean and all durable layers are committed.
3. Long builds additionally run **checkpoint-resumable** where the toolchain
   allows it (for cargo release profiles: `CARGO_INCREMENTAL=1` on profiling
   throwaway builds), so an accidental exit costs minutes, not the monolith.

## Consequences

- The pause-signal protocol (✅ DONE / ⏳ WAITING) is EXTENDED, not replaced:
  DONE/WAITING describes durable-layer/continuity state; the EXIT-SAFETY line
  describes process-tree state. Both are required.
- Environment fact, durable: harness-tracked background jobs do NOT survive
  `/exit` on this host. A job that must survive a session boundary needs either
  the explicit ⛔ banner holding the session open, or full detachment
  (`setsid`/`nohup` + marker files) at the cost of harness auto-notification.
- Related: `feedback_host_ram_budget_all_jobs.md` (guard wrapping is unchanged);
  the harness-memory observability rule (marker + liveness + bounded timeout)
  remains in force — markers are what made all three kills diagnosable.
