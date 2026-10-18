---
id: a-scratch-directory-name-is-not-a-fresh-directory
title: A scratch directory name is not a fresh directory — a ceremony waiter satisfied by an earlier session's marker (`SV-CORPUS-GRAD.13c.2v.2`)
answers:
  - "why did a release chain start before its build finished"
  - "how should a ceremony chain wait for the previous chain"
  - "can a rust/target scratch directory be reused across sessions"
tags: [ops, ceremony, scratch, chains, release]
date: 2026-10-17
provenance: SV-CORPUS-GRAD.13c.2v.2 (PGEN-SV-CORPUS-GRAD-0458); SVPP-EXPANSION.8 (the 2026-09-14 owner of rust/target/svpp8/)
reverify: "ls rust/target/svpp8/*.svpp-expansion-8-2026-09-14   # the two renamed markers"
---

# A scratch directory name is not a fresh directory

**What happened.** The `systemverilog_preprocessor` release `1.0.8` used `rust/target/svpp8/` as its ceremony scratch
directory. The name had been used before: `SVPP-EXPANSION.8` (2026-09-14) left its `build_chain.done` and
`sha_final.txt` there. The release's svpp chain waited with `while [ ! -f $S/build_chain.done ]` — and passed at once,
while the release probe was still compiling. It ran the reproducer bank and a corpus census on the OLD binary before it
was stopped. Nothing it wrote survived (the census re-ran on the published binary), and the stale markers were renamed,
not deleted, because the older leaf cites the directory's `raw_*` dumps.

**The rule.** Before using `rust/target/<name>/` for a chain, check it does not exist (or is empty); and key every waiter
on a marker the chain removes at its own start, so no file from another run can satisfy it. The `.53` snapshot itself was
safe — `sv_ceremony_binaries.sh snapshot` overwrites `bin/` and `sha_final.txt` — but a waiter is only as fresh as the
file it polls.
