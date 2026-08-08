---
id: normalize-a-shared-convention-once-at-ingestion-and-enumerate-its-consumers
title: Changing a shared data convention is a claim about a SET of consumers — enumerate them, and normalize the convention once at ingestion rather than at each use site
answers:
  - "I changed a path/format convention in a shared artifact — how do I know what I broke"
  - "how do I safely change the column format of a file several scripts read"
  - "why did my consumer die with 'is not in the subpath of' after a path change"
  - "should I normalize an input format at ingestion or at each place I use it"
  - "my script failed at the very last step after doing all the expensive work — how do I avoid that"
  - "how do I validate the shape of an input before paying for a long run"
  - "is it enough to test one consumer of a convention I changed"
  - "a comment says the change was verified backward-compatible but it broke anyway — why"
tags: [conventions, refactoring, scripts, corpus, ops, verification]
date: 2026-08-08
status: current
evidence: stimuli/run_external_corpus.sh (column 3 made repo-root-relative by CORPUS-GRAD-ALL.2.1); stimuli/sv/adjudicate_external_corpus.py (UVM_FOLD_MARKER, fixed in SV-CORPUS-GRAD.10); stimuli/sv/corpus_rule_coverage.py (repo_relative(), fixed in SV-CORPUS-GRAD.7c); docs/tasks/SV-CORPUS-GRAD.md leaves .10/.7c/.11b
reverify: "grep -q 'def repo_relative' stimuli/sv/corpus_rule_coverage.py && grep -q 'repo_relative(p)' stimuli/sv/corpus_rule_coverage.py && grep -q 'UVM_FOLD_MARKER = \"stimuli/sv/uvm/\"' stimuli/sv/adjudicate_external_corpus.py && echo CONSUMERS-ACCEPT-BOTH-SPELLINGS"
---

**One commit made column 3 of a shared results file repo-root-relative instead of absolute. It
carried a comment saying the consumers were "verified before changing this" — and it broke three
of them.**

| consumer | how it broke | found |
|---|---|---|
| `adjudicate_external_corpus.py` | split on `"/stimuli/sv/uvm/"` — a marker whose **leading slash** no relative path contains | `SV-CORPUS-GRAD.10`, on the first re-run |
| `corpus_rule_coverage.py` | `Path(p).relative_to(ROOT)` → `ValueError: … is not in the subpath of …` | `SV-CORPUS-GRAD.7c` |
| `cluster_rejects_valid.py` | rebuilds paths as `subs_root/suite/rel`, wrong for the one corpus that lives outside `subs/` | routed as `.11b`, still dormant |

⛔ **The comment was not dishonest — it was scoped to N=1 and written about N=3.** The check really
was run, on the `/subs/<suite>/` arm, where a relative path *does* still contain the infix. That
result was then generalized to an arm where it was never true. **A compatibility claim is a claim
about a SET; the evidence has to cover the set.** Before changing a shared convention, enumerate
its consumers — `grep -rn <artifact-name>` takes seconds and would have named all three.

⭐ **Normalize once at ingestion, never per use site.** The instinct when the traceback points at
line 266 is to guard line 266. But the convention enters the program at exactly one place — where
the file is read — and every downstream use inherits whatever spelling arrived. A single
`repo_relative()` (accept relative as-is; relativize an absolute inside the repo; pass a foreign
absolute through unchanged) makes both spellings legal forever and gives the convention **one site
to extend**, which is the same argument that made the stuck-point clusterer family-parameterized
rather than forked ([[a-copied-diagnostic-covers-only-where-it-was-pasted]]).

⭐⭐ **And rank failure modes by how much work precedes them.** The `relative_to` call sat in the
**report-emit** stage, so the run probed 9 694 corpus files, spent 167 s, and discarded every
result at the last statement. A shape check costs the same whether it runs first or last; paying
for it last costs a whole measurement. **Validate the shape of your inputs before doing the
expensive thing with them.**
