---
id: a-mis-cited-production-reproduces-as-a-success
title: A mis-cited production is more dangerous than a missing one — it reproduces as a SUCCESS, so the test that should catch it passes
answers:
  - "why did a grammar fix pass its test and still be wrong"
  - "should I check the clause number when a grammar comment quotes an LRM production"
  - "why did PGEN reject {} in SystemVerilog"
  - "what is an empty unpacked array concatenation in SystemVerilog"
  - "is it enough to open the clause once when a fix makes two citation claims"
tags: [grammar-authoring, lrm-fidelity, systemverilog, verification, citations, strictness]
date: 2026-08-10
status: current
evidence: docs/tasks/SV-CORPUS-GRAD.md leaves .3.26 (repro, root cause, the 5-row corpus delta), .3.26b (the director ruling that '{} is legal), .3.26c (the re-modelling), .3.26d (this card's correction); grammars/systemverilog.ebnf rules empty_unpacked_array_concatenation (pure A.8.1) and assignment_pattern (the empty-element-list alternative); docs/systemverilog/2017/md/section-41-data-read-api.md:2907 (A.8.1), :3551 (footnote 35); the §11.4.12 notation sentence at docs/systemverilog/2017/md/section-0-defined-as-false-or-if-the-result-is-ambiguous-the-unknown-value-x-the-precedence-of-is-greater.md:520-522; Annex M/VPI at docs/systemverilog/2017/md/section-83-accept-on-operator.md:89; the superseded fix SV-EXH-PROOF.3.3.4.b.6.2.37.8
reverify: "printf 'module m;\\n int q[$];\\n initial q = {};\\nendmodule\\n' > /tmp/e.sv && ./rust/target/release/parseability_probe --parse systemverilog /tmp/e.sv --profile sv_2017"
---

A fix that quotes a standard's production has two independent things that can be wrong: the
**transcription** and the **citation**. Tests check the first. Nothing checks the second — and the
second failing is the worse case, because it produces a passing test.

## The worked instance

`SV-EXH-PROOF.3.3.4.b.6.2.37.8` found `empty_unpacked_array_concatenation := lbrace epsilon rbrace`
— genuinely broken, since `epsilon` is not a rule in this grammar, so the production could never
match anything. It replaced the body with `tick lbrace rbrace`, citing *"§A.6.7"*.

The production is at **A.8.1** and reads:

```ebnf
empty_unpacked_array_concatenation35 ::= { }
```

bare braces, **no apostrophe** — with footnote 35 (*"{ } shall denote an empty unpacked array
concatenation … and shall not be used in any other form of concatenation"*) and §7.10's prose
(*"The empty queue can be denoted by an empty unpacked array concatenation {}"*) both saying the
same thing.

So the shipped rule accepted a literal the cited clause does not contain, and rejected the one it
does — **while its test went green**, because the test used the wrong literal too.

⭐ **Why it survived.** The motivating input was uvm's `return '{};`, which genuinely uses the
apostrophe. The fix was tested against it, it passed, and **a passing test on a wrong literal is
indistinguishable from a passing test on a right one.** A *missing* fix announces itself the next
time the corpus runs; a *mis-cited* fix banks a green result and goes quiet. It took the corpus
moving two unrelated rows' `furthest_position` deeper onto the construct to surface it, two campaigns
later.

## The habit

> **When a fix quotes a production, quote the clause number too — and open the clause.** The
> citation is the one part of the change no test can verify for you.

Corollaries that paid off here:

- **The annex keeps half its rules in footnotes** — see
  [[annex-a-footnotes-license-derivations-the-productions-cannot-derive]] for the sibling case where
  a footnote *licenses* a derivation the productions cannot express. Here footnote 35 *confirms* one.
- **A one-rule fix whose corpus delta is entirely its own construct is the shape to aim for.** This
  one moved exactly 5 rows, all `unexplained_rejects_valid → match`, from four independent suites,
  and none the other way.

## ⛔ The sequel — the SAME fix made a SECOND citation claim, and that one was never opened either

This card originally closed with a section adjudicating `'{}` as *"over-acceptance kept as deliberate
dialect tolerance"*. **That was wrong, and it is instructive that it was wrong in exactly the way the
card warns about.**

The repair to the `{ }` half was verified by opening A.8.1. The claim about the **other** half —
*"`'{}` has no derivation in Annex A, therefore it is over-acceptance"* — was **inferred from the
absence of a production**, never read. Pressed by the director (*"Please find in the LRM where it is
claimed `'{}` wasn't supported"*), the answer is **nowhere**. Measured across both revisions, with
`pymupdf` over the full PDFs and by grep over the markdown, `'{ }` appears exactly twice per
revision and **both occurrences support it**:

- **§11.4.12** — *"Concatenations are enclosed in just braces ( `{ }` ), whereas structure and array
  literals are enclosed in braces that begin with an apostrophe ( **`'{ }`** )."* The standard writes
  the construct's own delimiter pair with the apostrophe.
- **Annex M / VPI** — `#define vpiAssignmentPatternOp 75 /* '{} assignment pattern */`.

No text anywhere forbids it. `'{ }` is **legal SystemVerilog**, a third measured instance of the
Annex-A-incompleteness class alongside `q[a:$]` (footnote 42) and `use #(...)` (clause 33.4.3) — so
the rule was wrong in **one** direction, not two, and the apostrophe form is not tolerated, it is
correct. Since `SV-CORPUS-GRAD.3.26c` it is modelled where it belongs: an `assignment_pattern` whose
element list is empty.

> ⛔ **The standing rule this established: "non-LRM" is a CITATION, never an inference.** Absence
> from a production is not a prohibition. Before claiming a construct is illegal, produce the
> sentence that says so. See `docs/decisions/feedback_sv_strict_lrm_compliance_default.md`
> § BOUNDING RULING.

⭐ **Why this belongs on this card rather than a new one.** The habit above says *quote the clause
number and open the clause*. The session that wrote this card **did open one clause and then
skipped the other** — it had the LRM on disk, searched it correctly minutes earlier to confirm
A.8.1, and asserted the negative claim from reasoning instead of running one more grep. So the rule
is not "check your citation"; it is **check every citation the change makes, including the ones
phrased as a conclusion rather than a quote.** A claim of the form *"X is not legal"* is a citation
with the quote left out.

⚠️ **And the search itself needs a working instrument.** The first negative search was run with
`pdftotext -layout`, which produced 82 000 lines and found neither `'{}` nor the §11.4.12 sentence
that demonstrably exists — a broken extractor's silence reported as evidence, which is the same
error class one level down. Use `pymupdf` (`import fitz`) on the in-repo PDFs, or grep the markdown;
⛔ never `pdftotext`. Note the markdown wraps sentences, so an exact-phrase grep for
*"begin with an apostrophe ( '{ } )"* also finds nothing — search for the distinctive short token
(`apostrophe`, `'{ }`), not the sentence.
