---
id: a-staleness-census-is-a-walk-over-git-objects-not-a-recollection
title: "How often did this artifact go stale?" is a walk over git objects, not a recollection — and a byte-stale count must be CLASSIFIED into stale-by-meaning and stale-by-spelling before it is published
answers:
  - "how often has this tracked report been stale against the thing it describes"
  - "how do I size a staleness problem over the project's whole history"
  - "the report records a digest of its input — how do I check it at every past commit"
  - "my stale count includes comment-only edits — how do I separate those"
  - "how do I show that wiring a gate actually changed behaviour, not just that it can refuse"
  - "what is the cheapest before/after measurement of a doctrine's effect"
tags: [provenance, census, git, staleness, corpus, doctrine, evidence, sv-corpus-grad]
date: 2026-09-02
status: current
evidence: "SV-CORPUS-GRAD.13e.6(b)+(c) / PGEN-SV-CORPUS-GRAD-0335. `census.sh` walks the 312 commits touching `grammars/systemverilog.ebnf`, hashing `git show <rev>:grammars/systemverilog.ebnf` and reading the grammar row of `git show <rev>:stimuli/sv/characterization/characterization.md`: scored=35 fresh=24 stale=11 stale_runs=7, longest run FIVE semantic revisions (08-24 → 08-26). `classify.sh` recovers each stale row's measured-against revision by sha lookup and compares SEMANTIC digests (the frontend's raw_ast envelope): 7 DIFFERENT_LANGUAGE, 4 SAME_LANGUAGE. Split at the day the census gate was wired (2026-08-31): 11 stale of 31 before, 0 of 4 after."
reverify: "bash docs/tasks/artifacts/sv_corpus_grad/outcome_staleness_census/census.sh >/dev/null && bash docs/tasks/artifacts/sv_corpus_grad/outcome_staleness_census/classify.sh >/dev/null   # OUTCOME-STALENESS: … stale=11 stale_runs=7 ; OUTCOME-STALENESS-CLASS: … different_language=7 same_language=4"
---

**The population already exists.** A tracked artifact that records a digest of its input carries,
at every historical commit, a claim about the input beside it. `git show <rev>:<path>` for both
files at every revision that touched the input gives the exact stale/fresh history in seconds —
from objects, never from the working tree, so it re-derives identically on any clone.

## The walk

```bash
git log --format='%h %ad' --date=short -- <input>          # the population
git show <rev>:<input> | sha256                              # the input's real digest at <rev>
git show <rev>:<report> | awk '/^\| <label> \|/{print $4}'   # the digest the report RECORDED at <rev>
git diff-tree --no-commit-id --name-only -r <rev> -- <report># did the report move in that commit?
```

A revision where the report does not exist is `no_report`; one whose report predates the identity
block is `no_identity` — both **counted and never scored**, so the headline's denominator is the
scoreable set and says so.

## ⛔ Classify before you publish

A byte digest goes stale on a comment. Of 11 stale readings, **4** were edits whose semantic
digest — the envelope the code generator consumes — was unchanged. Publishing "11 stale" would have
over-stated the defect by four and hidden the fact those four were the **price** the byte-keyed gate
charges (each a blocked commit until two full re-measures run). The classification takes the same
walk one step further: recover the measured-against revision by looking the recorded sha up against
every input blob in history (a miss is `UNCLASSIFIED`, never scored), and compare semantic digests at
the two revisions.

⭐ **The same measurement often prices a sibling leaf for free** — here the 4 same-language rows were
exactly the number the semantic-re-keying leaf had been opened without.

## ⭐ A gate's wiring date is a natural experiment

A RED control proves a gate *can* refuse. Splitting the census at the commit that wired the gate
proves it *did* change behaviour: 11 stale of 31 before, 0 of 4 after, with every post-gate input
revision touching the report in the same commit. Cost: one `awk` over the census you already ran.

## The honest bound to carry with the number

A staleness census counts stale **reports**, not hidden **verdicts**. "7 revisions whose numbers
described a different language" is not "7 wrong bars" — measuring the latter means re-running the
producer at each revision under that revision's parser. State the shape beside the count, every time
it is quoted ([[a-number-read-out-of-context-carries-a-frame-you-did-not-measure]]).
