---
id: a-walk-needs-its-own-terminator-not-just-a-failing-step
title: "`cd ..` at `/` SUCCEEDS, so the classic shell repo-root walk has no terminator — run outside a checkout it spins forever instead of refusing"
answers:
  - "my script hangs with no output and I cannot tell which command is stuck"
  - "a diagnostic script produced nothing at all and I blamed the tool it invokes"
  - "how do I find the repository root from a script without hard-coding a depth"
  - "is `cd .. || exit 1` a safe loop terminator"
  - "why does my helper work in the repo and hang when I copy it elsewhere"
  - "a probe timed out and I assumed the parser was pathological — what else could it be"
  - "what is the difference between a step that can fail and a loop that can end"
tags: [shell, bash, tooling, instruments, ops, hangs, repo-root, evidence]
date: 2026-08-23
status: current
evidence: "SV-CORPUS-GRAD.13c.2b (PGEN-SV-CORPUS-GRAD-0279). Three tracked instruments under docs/tasks/artifacts/sv_corpus_grad/ resolved the repository root with `while [ ! -f CLAUDE.md ] || [ ! -d grammars ]; do cd .. || exit 1; done`. POSIX resolves `/..` to `/`, so `cd ..` at the root succeeds and the `|| exit 1` escape is unreachable. Isolated: `SPIN: 201 iterations, still at pwd=/`. The shipped line copied outside a checkout and run under a 20 s budget exited 124 with no output at all; after adding an explicit `[ \"$d\" = \"/\" ]` terminator all three exit 2 with `REFUSE: no repository root (CLAUDE.md + grammars/) above <path>`. The Python sibling `_repo_root.py` in the same directory never had the bug — it iterates `Path.parents`, which is finite, and raises `REFUSE`."
reverify: "(cd / && cd .. && echo cd-dotdot-at-root exit=$? pwd=$(pwd)); grep -l find_repo_root docs/tasks/artifacts/sv_corpus_grad/*/*.sh | wc -l # exit=0 pwd=/ is the hazard (the escape can never fire); 3 is every walker terminated"
---

**A loop whose only exit is \"a step failed\" has no terminator if that step cannot fail.** The
canonical shell repo-root walk is exactly that shape, and the step it relies on is one of the few in
POSIX that is *defined* to succeed at the boundary.

```bash
# the shape, as it appears in a hundred repositories
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" \
        && while [ ! -f CLAUDE.md ] || [ ! -d grammars ]; do cd .. || exit 1; done; pwd)"
```

`/..` **is** `/`. So at the root `cd ..` returns 0 and changes nothing: the loop condition stays
true, the `|| exit 1` never runs, and the script spins at 100 % CPU producing no output, no warning
and no exit code. Measured on the real line, isolated:

```text
SPIN: 201 iterations, still at pwd=/ — cd .. at / is a NO-OP that never fails
```

and the untouched shipped line, copied outside a checkout under a 20 s budget:

```text
exit 124   (timed out — no output at all)
```

## Why the bug is invisible until the day it is not

Nobody runs a repo helper outside its repo — until something does. A CI job that copies scripts into
a staging directory, a `mktemp` sandbox, a container that mounts only part of the tree, or a red
control that deliberately relocates an instrument to prove it can fail. The walk is correct for
every path the author exercised and unbounded for the first path they did not.

## The failure mode is the worst one a diagnostic can have

A tool that **refuses** tells you it cannot answer. A tool that **hangs** gets its silence
attributed to whatever it invokes. In the founding case the stuck script wrapped a parser probe, and
the first reading was *"the parser is backtracking pathologically on this input"* — a plausible,
entirely wrong root cause that survived until the process list showed **no parser running at all**.
An instrument that hangs does not merely fail to help; it manufactures a false lead.

## The fix: give the walk its own terminator

```bash
find_repo_root() {
  local d
  d="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)" || return 1
  while [ ! -f "$d/CLAUDE.md" ] || [ ! -d "$d/grammars" ]; do
    [ "$d" = "/" ] && return 1          # <-- the terminator the `cd` could never be
    d="$(dirname "$d")"
  done
  printf '%s\n' "$d"
}
ROOT="$(find_repo_root)" || { echo "REFUSE: no repository root above ${BASH_SOURCE[0]}" >&2; exit 2; }
```

Two properties changed, and both matter. The walk now ends because the *state* runs out
(`dirname /` is `/`), not because an operation errored. And it ends by **refusing with a reason**,
which is what a diagnostic owes its caller.

⭐ The correct model usually already exists nearby. Python's `Path.parents` is finite by
construction, so the same helper written in Python cannot have this bug — which is why a repository
can carry both spellings of one contract for a year with only the shell one broken. When two
languages implement the same helper, the one with a bounded iterator is the specification.

## The general rule

Separate the two questions a loop asks:

- *can this step fail?* — an error path,
- *can this loop end?* — a terminator.

`|| exit`, `set -e`, `|| break` and `try/except` all answer the first. None of them answers the
second. Whenever a loop walks a structure — parent directories, a linked list, a symlink chain, a
retry ladder, a graph — name the boundary explicitly and stop there, even when the step you take at
the boundary looks like it must fail. `cd ..` at `/`, `dirname` of `/`, `next` on a self-referencing
node and `realpath` of a symlink loop all succeed quietly.

⚠️ **Bound.** The census behind this record covers *this exact idiom in shell* (`grep -rln 'while \[
! -f <marker> \]' --include='*.sh'`). A walker spelled differently, or written in another language
with an unbounded iterator, is the same class and would not be in that count.
