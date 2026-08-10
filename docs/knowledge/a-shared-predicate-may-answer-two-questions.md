---
id: a-shared-predicate-may-answer-two-questions
title: "Before making a shared predicate smarter, count its CALLERS — one function can answer two different questions, and sharpening it for one caller silently corrupts the other"
answers:
  - "can I just make this helper consult more context to fix the caller that is wrong"
  - "why did tightening a shared predicate break a path I never touched"
  - "how do I safely change a function that is called from more than one decision"
  - "the whole-file check is too coarse — should I make it positional"
  - "how do I tell whether a coarse heuristic is a bug or the correct answer for some caller"
tags: [refactoring, shared-predicate, corpus-adjudication, silent-failure, systemverilog, instrument-soundness]
date: 2026-08-10
status: current
evidence: stimuli/sv/adjudicate_external_corpus.py preproc_dependency() (kept as a whole-file existence test, with the docstring naming which caller relies on which reading) + svpp_can_explain_failure() (the positional gate, applied on the must_accept arm ONLY); docs/tasks/SV-CORPUS-GRAD.md leaves .12 (the 26-row finding) and .12a ("THE HALF THE LEAF DID NOT SCOPE, AND IT MATTERED"); stimuli/sv/characterization/adjudication_manifest.tsv (exactly 26 rows moved, must_reject arm byte-inert)
reverify: "grep -n 'preproc_dependency(text)' stimuli/sv/adjudicate_external_corpus.py && grep -n 'svpp_can_explain_failure(' stimuli/sv/adjudicate_external_corpus.py && python3 -c \"import sys;sys.path.insert(0,'stimuli/sv');import adjudicate_external_corpus as A;print('whole-file test still coarse:', A.preproc_dependency('module m; \\`FOO endmodule')); print('positional gate disproves a pre-tick failure:', not A.svpp_can_explain_failure('module m; x y z; \\`FOO endmodule', 10))\""
---

`SV-CORPUS-GRAD.12` proved that 26 corpus rows were labelled *"blocked by the preprocessor"* while
the parser demonstrably stopped somewhere the preprocessor cannot reach. The label came from
`preproc_dependency()`, a whole-file regex test — *does this file contain a `` `include ``/macro
use/`` `ifdef `` anywhere?* — read downstream as a causal claim about one parse.

The fix looks like a one-liner: make the predicate consult the failure position. **It is not, and
the reason is the general lesson.**

## Count the callers first

`preproc_dependency()` is consulted on *both* arms of the adjudication, and the arms ask different
questions:

| caller | question | needs a failure position? |
|---|---|---|
| `must_accept` + observed `fail` | *did this file fail BECAUSE it needs the preprocessor?* | **yes** — a causal claim about one parse |
| `must_reject` | *could this file's intended syntax error be hidden until after preprocessing?* | **no** — a property of the whole file |

The second row is the trap. It is asked of rows that **may have no failure position at all**,
because the interesting `must_reject` row is the one that wrongly *passes*. Making the predicate
positional wholesale would have satisfied the first caller and judged every `must_reject` row
against a position that either does not exist or describes something else entirely — and it would
have done so **silently**, since both readings return a plausible-looking flag.

## The shape of the safe change

Leave the shared predicate alone. Add the sharper test as a **separate** function, apply it at the
one call site that needs it, and put a docstring on each saying which caller depends on which
reading:

```python
if expected != "must_accept":
    dep_flag = ""
elif dep_flag and observed == "fail" and not svpp_can_explain_failure(text, furthest):
    dep_flag = ""          # the dependency is real and irrelevant to THIS failure
```

Exactly 26 rows moved; the `must_reject` arm stayed byte-inert.

## The tell to watch for

A coarse-looking heuristic is not automatically a bug. **It may be exactly right for a caller you
have not looked at yet.** Before sharpening one, ask what each caller does with the answer — and if
two callers want different things, that is a signal to *split the change*, not to split the
predicate into a cleverer one that tries to serve both.

The failure mode is quiet by construction: both readings return the same *type*, so nothing crashes,
no test fails on shape, and the damage shows up later as rows that moved for no stated reason.
