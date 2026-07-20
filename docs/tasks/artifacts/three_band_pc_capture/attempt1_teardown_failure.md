# Attempt 1 — complete capture rejected after teardown SIGPROF

The first `PGEN-RGX-0078-0182` run used the qualified v2 sampler source from
`docs/tasks/artifacts/inprocess_pc_sampler/pc_sampler.c`. The sub-1 us and
1–2.5 us processes exited zero. The 2.5–20 us workload wrote all 488 timing rows,
its complete 17-line summary, and a raw header with **37,202 seen/stored, zero
dropped, 134,323,686 us CPU**. It then terminated with memory-guard exit 155 and
the shell diagnostic `Profiling timer expired: 27`.

## Root cause

Darwin defines signal 27 as `SIGPROF`. The v2 destructor blocked `SIGPROF`,
disabled `ITIMER_PROF`, restored the prior/default signal action, emitted the raw
file, and finally restored the prior signal mask. A profiling signal already
pending at the block/disable boundary therefore acquired the default action and
terminated the process when the old mask was restored. The raw evidence was
complete, but an abnormal workload exit is not admissible.

The separately owned v3 source in this directory closes the race with Darwin's
`sigpending` + `sigwait`: it drains any pending `SIGPROF` synchronously while
the signal remains blocked and the timer is disabled, records the drained count
in its v3 header, and only then restores the prior action/mask. Handler behavior
is unchanged. A hard-disabled test hook raises one `SIGPROF` at that exact
blocked/disabled boundary so the runner can require one drained signal and a
zero process exit before spending the corpus captures; normal captures assert
that the hook is off.

## Rejected-attempt custody

- sub-1 us raw SHA-256: `c87a5903b58b4d9379aa6102859dfb9454f9cb5943d2fae02aa4c85e2c6ef1f4`
- 1–2.5 us raw SHA-256: `5e7032833724f11abf8bbee8beab542c473ff1fad76bdfaba25494739e3310a9`
- 2.5–20 us raw SHA-256: `d632d5ef0106f9d84f3b80b457c02ace85f76c0e7b8d3de3185801f8fb037eb4`
- sub-1 us timing SHA-256: `d97d853febd9167de8b6684e9779b90d7b7c5e1bd22ea0d3638726d433f7ba65`
- 1–2.5 us timing SHA-256: `355198acd1db7c36734b8be64e87864e6136cf4c8fe2b62134d92d9855d7c46e`
- 2.5–20 us timing SHA-256: `b5f19cd8a62c9d584d55428464f6e14ade6b4cf1f68589519bb0349456612ac0`

The second run overwrites the working capture files with v3 evidence. These
hashes preserve the rejected run without banking roughly 1.5 MiB of redundant
raw lines.
