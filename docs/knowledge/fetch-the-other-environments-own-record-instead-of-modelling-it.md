---
id: fetch-the-other-environments-own-record-instead-of-modelling-it
title: When the question is about ANOTHER environment, fetch that environment's own record — modelling it is how four task leaves reasoned for weeks past a gate that had been red the whole time
answers:
  - "my CI passes locally but I do not know what it does on the runner"
  - "how do I find out whether my hosted workflows are actually green"
  - "a gate passes on my machine and fails in CI — where do I start"
  - "why does make die with No such file or directory on a Linux runner"
  - "is my automatic CI tier actually running anything"
  - "how much are my GitHub Actions runs costing me"
  - "how do I check whether a workflow has ever run at all"
tags: [ci, evidence, instruments, ops, github-actions, portability, build]
date: 2026-08-25
status: current
evidence: CI-PARITY-GATE-ROT.47. A whole task-tree existed because the hosted and local sides of one repository disagreed. Four leaves (.4, .40, .46, ENGINE-UNIVERSAL-SERVICES.49) reasoned about hosted behaviour — measuring `on:` blocks, counting workflows, pricing runner minutes — and none ran `gh run list`. One second of that command showed the only doctrine with an automatic hosted lane had been RED on `main` for 11 days, that no regenerating workflow had EVER run hosted, and (via `gh api .../timing`, `billable.UBUNTU.total_ms = 0`) that the spend policy the leaves were reasoning around did not bill at all. Root cause of the red was `rust/Makefile` setting `SHELL=/opt/homebrew/bin/bash`, an absolute macOS path that make resolves before running any recipe, so every target died instantly on Linux.
reverify: "gh run list --limit 20 && gh api /repos/<owner>/<repo>/actions/runs/<run_id>/timing   # actual verdicts and actual billable_ms, instead of a model of them"
---

**If your question is about an environment you are not in, that environment almost certainly keeps
its own record — fetch it.** Reasoning carefully about a remote system is not evidence about the
remote system, no matter how carefully you reason.

## The failure this is drawn from

A repository had an entire task-tree devoted to *"the hosted CI side and the local side disagree"*.
Across four separate leaves it measured `on:` trigger blocks, counted which workflows carried a
regeneration step, priced runner minutes against a documented budget, and reasoned about what a
fresh runner would hand a job. All of it careful. All of it a **model**.

One command was never run:

```bash
gh run list --limit 20
```

It reported, immediately:

- the only doctrine carrying an **automatic** hosted lane had been **failing on `main` for 11 days**;
- **no** workflow requiring artifact regeneration had **ever** run hosted — so the expensive path
  everyone was reasoning about was entirely unexercised;
- and one `gh api /repos/<owner>/<repo>/actions/runs/<id>/timing` later,
  `billable.UBUNTU.total_ms = 0` — the cost the leaves had been carefully conserving was **not
  being billed at all**, because the repository had become public.

The failure that had been red for 11 days took one more command to diagnose, because the runner had
kept the log:

```
make: /opt/homebrew/bin/bash: No such file or directory
```

## Why models of another environment fail in a specific direction

They fail **optimistically about what you cannot see**, and the reason is structural: you build the
model out of the things you can read from where you are standing — the YAML, the Makefile, the
recorded measurements. What you cannot read from here is *what actually happened*, so the model
quietly assumes it was unremarkable.

The `SHELL` defect is the shape in miniature. A `Makefile` whose `SHELL` is an absolute path to a
package-manager binary works forever on the machine that has it, and dies before the first recipe
line on every machine that does not. Nothing in the file *looks* wrong. Nothing local can go red.
Only the other environment can tell you, and it will — in a log — the moment you ask.

## The rule

Before reasoning about another environment, ask what record it keeps, and read that first:

| the environment | its own record |
|---|---|
| hosted CI | `gh run list`, `gh run view <id> --log-failed`, `.../timing` for real billable ms |
| a package registry | the published artifact and its metadata, not your lockfile's opinion of it |
| a deployed service | its logs and health endpoint, not your local run |
| another machine's build | its build log, not a reproduction of what you think it does |

**A cost you believe you are paying is worth one API call before you design around it**, and a
platform's own billing endpoint will answer in a way no policy document can go stale about.

## The companion trap: a workaround applied everywhere hides its own defect

Once found, the `SHELL` bug had a second lesson in it. Every documented command, every workflow and
the composite action all passed `SHELL=/bin/bash` explicitly. The repository had absorbed the defect
as a *habit*, so the broken default was invisible — until exactly one caller forgot the override,
at which point it looked like that caller's bug.

⇒ **when one site fails and a hundred pass, find out why the hundred pass before you fix the one.**
If the answer is "they all carry the same workaround", the bug is in the default, not the caller —
and fixing the caller earns a green while leaving every *user* of the project broken.

## Related

- [[a-conditional-compilation-gate-is-a-claim-about-which-trees-exist]] — the same shape in the
  compiler: correct on every tree you own, broken on the one you never build.
- [[an-x-is-checked-by-nothing-claim-is-a-census-claim]] — the discipline for the other half of this,
  where the claim is about coverage rather than about another machine.
