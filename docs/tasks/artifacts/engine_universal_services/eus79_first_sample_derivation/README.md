# `ENGINE-UNIVERSAL-SERVICES.79`(b)(c) — the reservation × depth ladder

Producer for the before → after table in
[`docs/tasks/ENGINE-UNIVERSAL-SERVICES.md`](../../../ENGINE-UNIVERSAL-SERVICES.md), leaf `.79`
(`PGEN-ENGINE-UNIVERSAL-SERVICES-0142`).

## What it measures

Nine nesting depths (200…800 parenthesis levels of a SystemVerilog continuous assignment) against
three parse-stack reservations (96 MiB — the module's own documented floor — 128 MiB, and the
256 MiB shipped default), reporting for each cell the exit code, the wall clock, and the depth
probe's own `stack_probe:` reading (`.69`(d)'s instrument, `TOOLBOX.md` §3.3d).

The cell that matters is `rc`: **134** is an uncatchable `SIGABRT` — the parse thread overflowed its
stack before the recursion guard could refuse — and **1** is a clean, diagnosed rejection.

## Running it

```bash
bash docs/tasks/artifacts/engine_universal_services/eus79_first_sample_derivation/reservation_depth_ladder.sh \
     ./rust/target/debug/parseability_probe
```

⛔ **DEBUG build.** The defect is a debug-per-frame-cost defect (25.3–28.4 KB/frame against release's
≈3.08 KB); a release probe does not reach it at these reservations.

⚠️ **Run it on an idle machine.** The deep rejecting rungs are `ENGINE-UNIVERSAL-SERVICES.68`'s
Θ(depth²) re-exploration and the whole ladder takes ~8 minutes per binary; the verdicts and the
refusal depths are deterministic, the wall clocks are not.

## The two recorded runs

| file | binary | headline |
|---|---|---|
| `ladder_before.txt` | pre-`-0142`, preserved before any rebuild | **7 of 9** rungs at the 96 MiB floor are `rc 134` SIGABRT |
| `ladder_after.txt` | post-`-0142` | **0 of 9** abort; every rung is a clean verdict, and the 128 MiB and 256 MiB groups are unchanged row for row |

⛔ The binaries themselves are **not** tracked (188 MB each, and `generated/` is untracked so they are
not reproducible byte-for-byte from a fresh clone anyway). What is tracked is the producer and both
result tables, which is what a reader needs to re-run the comparison and to check the published
numbers against the raw output.
