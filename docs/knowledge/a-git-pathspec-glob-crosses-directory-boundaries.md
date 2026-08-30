---
id: a-git-pathspec-glob-crosses-directory-boundaries
title: "`git ls-files 'docs/tasks/*.md'` is not the shell's `docs/tasks/*.md` — a git pathspec `*` matches `/` too, so a census silently sweeps in every subdirectory"
answers:
  - "why does git ls-files with a glob return more files than the shell glob"
  - "my file census counts twice as many files as the directory holds — why"
  - "does a git pathspec asterisk match a slash"
  - "how do I list only the files directly inside a tracked directory"
  - "my population size changed when I moved from ls to git ls-files"
  - "how do I size a corpus before adjudicating it"
tags: [git, census, evidence, instruments, tooling, false-positives]
date: 2026-08-30
status: current
evidence: "DONE-BAR-VISIBILITY.2 (PGEN-DONE-BAR-VISIBILITY-0004). Sizing the task-tree population for a closed two-sided attribution register, `git ls-files 'docs/tasks/*.md' | wc -l` returned 337. The directory holds 164 top-level `.md` files; the other 173 live under `docs/tasks/artifacts/`, which the pattern was written to exclude. The error is silent and inflates in the direction that looks thorough — a 2x larger population to adjudicate, most of it machine-generated evidence files that are not task trees at all. Caught only because a second count, derived by filtering on the path's slash depth, disagreed."
reverify: "git ls-files 'docs/tasks/*.md' | wc -l; git ls-files 'docs/tasks/*.md' | awk -F/ 'NF==3' | wc -l   # the two numbers differ; the second is the top-level population"
---

**A git pathspec is not a shell glob.** Git matches its default (`glob`-magic-free) pathspec
patterns with `wildmatch` **without** `WM_PATHNAME`, so `*` happily matches `/`. The shell's
`docs/tasks/*.md` stops at the directory boundary; git's `docs/tasks/*.md` does not, and quoting the
pattern — which you must do to keep the shell from expanding it — is exactly what hands it to git.

```
$ git ls-files 'docs/tasks/*.md' | wc -l          # 337  — includes docs/tasks/artifacts/**
$ git ls-files 'docs/tasks/*.md' | awk -F/ 'NF==3' | wc -l   # 164 — the top-level population
```

## Why it is dangerous rather than annoying

The failure is **silent** and it inflates in the flattering direction. A census that sweeps in a
subtree looks *more* thorough, so nothing about the number invites suspicion — and if that number is
the population of a closed register (*"every X must be adjudicated exactly once"*), the extra
entries are not merely noise: they are work you will do, or a refusal you will hit, over files that
were never in scope.

## Getting it right

- `:(glob)docs/tasks/*.md` — enable `WM_PATHNAME`, so `*` stops at `/` and `**` is the crossing form.
- Or filter after the fact on path depth, which is obvious to a reader:
  `git ls-files docs/tasks/ | awk -F/ 'NF==3'`.
- Or list the directory itself when only its own children are wanted.

## The general rule

**A census is only as good as its selector, and the selector has two halves** — which FILES and which
LINES. The pattern *inside* the files gets scrutiny because its output is read; the pattern that
chose the files is trusted because it looks like a path. Size the population two independent ways
before adjudicating it, exactly as you would falsify any other measurement.
