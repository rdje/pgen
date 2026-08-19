---
id: a-poller-that-matches-its-own-command-line-never-terminates
title: A wait-loop built on `pgrep -f "<pattern>"` matches its OWN command line, so the condition can never go false — the job finishes and the watcher reports it running forever
answers:
  - "my until-pgrep wait loop never exits even though the job finished"
  - "how do I wait for a background build to finish from a shell"
  - "why does pgrep -f find my job when the job is not running"
  - "how do I poll for a long build without leaving zombie shells behind"
  - "my background job status says running but there is no cargo or rustc process"
  - "how should I verify that nothing is in flight before declaring a workspace clean"
tags: [ops, shell, background-jobs, polling, instruments, false-positives]
date: 2026-08-19
status: current
evidence: PGEN session 2026-08-19. Six `until ! pgrep -f "cargo build --release --features generated_parsers"; do sleep 30; done` waiters were spawned to block on ~20-minute release builds. Every one embeds the pattern in its own `bash -c` argv, so `pgrep -f` matched the waiters themselves — measured, `pgrep -f "cargo build --release --features generated_parsers"` returned SEVEN bash pids and ZERO cargo processes, with `pgrep -x cargo` and `pgrep -x rustc` both empty. The builds had all completed normally; the watchers ran for roughly two hours after the last one, at `Ss` with ~2 s of CPU each, and would have run until the session ended.
reverify: "pgrep -f 'until ! pgrep -f' | wc -l   # 0 = no self-matching waiters are alive; and cross-check the WORK with `pgrep -x cargo` / `pgrep -x rustc`, which cannot self-match because -x matches the executable name, not argv"
---

**`pgrep -f` matches the full command line of every process — including the one asking.** A wait
loop whose own `argv` contains the pattern is therefore a tautology: it finds itself, the condition
stays true, and it sleeps until something kills it. The failure is silent and it fails in the
*alarming* direction — the watcher keeps insisting a finished job is still running, so a status
report built on it says "in flight" long after the work is done.

⛔ **The damage is not CPU, it is the false state.** Six sleeping shells cost nothing. What they cost
was two hours of believing a build was in progress, and a handoff banner that read "waiting" when
nothing was in flight. *An instrument that reports work where there is none is as wrong as one that
reports none where there is work, and it is harder to notice because it looks like patience.*

## What to do instead

- **Match the executable, not the command line.** `pgrep -x cargo`, `pgrep -x rustc` — `-x` matches
  the process NAME, which a shell wrapper cannot spoof.
- **Exclude yourself when you must use `-f`:** `pgrep -f "<pattern>" | grep -v "^$$\$"`, or narrow
  the pattern to something only the real process has (an absolute binary path).
- **Better: don't poll at all.** Let the harness tell you the job ended, and use the job's own
  completion record — its exit code, its output file, or the artifact it was supposed to produce.
- **Best: assert the ARTIFACT, not the process.** The question is never really "is cargo running", it
  is "is the binary I need current". `shasum` the output and compare it to the input's expectation;
  that answer is true whether or not any process is alive, and it is the same discipline
  [[your-build-tools-timestamp-resolution-is-part-of-your-correctness-argument]] arrives at from the
  other direction.

⭐ The general shape is worth more than the shell trick: **a probe that can observe itself will
report itself.** It is the same defect as a keyword census whose pattern matched its own `kw_` rules
and reported them as findings, and as a grep-based sweep that counts the line it is written on. When
a detector and its subject live in the same namespace, exclude the detector explicitly — and prove
the exclusion with a run where the subject is absent and the detector must return zero.
