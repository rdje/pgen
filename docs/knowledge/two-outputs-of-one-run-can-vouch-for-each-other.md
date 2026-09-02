---
id: two-outputs-of-one-run-can-vouch-for-each-other
title: An identity block on a derivation cannot vouch for its own input — but two outputs of ONE run can vouch for each other, and the vouching must be a digest, not a reconciliation
answers:
  - "my analysis reads an untracked intermediate — how do I know it is the right one"
  - "how do I give an ignored or regenerable file a provenance it can be checked against"
  - "is holding the report's counts equal to the raw dump good enough"
  - "a tracked report has an identity block — does that cover the files it was derived from"
  - "how do I make a stale input loud without tracking a 1.4 MB dump"
  - "what should a consumer do when the provenance row it wants is missing"
tags: [provenance, identity, corpus, instruments, gitignore, evidence, sv-corpus-grad]
date: 2026-09-02
status: current
evidence: "SV-CORPUS-GRAD.13e.6(a) / PGEN-SV-CORPUS-GRAD-0333. `stimuli/sv/characterization/results.tsv` and `positions.tsv` are `.gitignore`d (`git ls-files --error-unmatch` exits 1; `git check-ignore -v` names `.gitignore:424` and `:432`), so they carry neither an identity block nor a history — and they are the input to every number the SV campaign publishes. `.13e.5` measured the cost: `t_property_unsup.v` sat `fail` for three days after `SV-0066` fixed the construct, so the axis-2 bar counted a defect that no longer existed (275 -> 274)."
reverify: "bash docs/tasks/artifacts/sv_corpus_grad/outcome_identity/probe.sh   # OUTCOME-IDENTITY: 8/8 as declared — six predicate arms plus two end-to-end"
---

**An identity block on a derivation says which tree the derivation describes. It cannot say that its
own INPUT came from the same run** — which is the whole question when that input is an ignored,
regenerable dump with no history at all.

## The move

Stop looking for a place to put the dump's identity *in the dump*. The report and the dump are
**two outputs of one run**, so have the run digest the dump into the report:

```text
| grammar            | grammars/x.ebnf    | b2ca001f… |   <- INPUT rows: does this report
| generated parser   | generated/x.rs     | c09b4eda… |      still describe your tree?
| results (output)   | …/results.tsv      | ed240b8f… |   <- OUTPUT rows: is the dump on your
| positions (output) | …/positions.tsv    | 02f49b65… |      disk the one it describes?
```

The consumer re-hashes the output rows before it reads a row of the dump, and **refuses** on a
mismatch. The identity now transfers through the tracked file in both directions: the report says
*which parser*, and the digest says *the dump is that parser's*.

## ⛔ A digest, not a reconciliation

The cheap-looking alternative is to hold the report's per-category counts equal to the dump — same
totals, same per-bucket numbers, no hashing. **It has a cancellation hole.** Two items swapping
categories inside one bucket leave every published count identical, so the check passes on a dump
that disagrees with the report item for item. That is the multiset trap in general form, and this
repository has paid for it once already in a corpus diff that compared `(bucket, verdict)` multisets
and had to be re-keyed onto the path.

A digest is exact, costs one streamed read, and has no bucket to hide in.

## The three verdicts, and why the third is not a pass

| the report… | the consumer… |
|---|---|
| records a digest that matches | proceeds, **and says so** — a silent pass is not a reading |
| records a digest that differs | **REFUSES**, naming the exact re-run |
| records no output row at all | reports **NOT EVALUATED**, loudly, naming the leaf that owes the stamp |

The third row is what lets the mechanism land before every artifact carries the stamp: an older tree
behaves exactly as it did, and says that it is unchecked rather than implying it is clean
([[feedback_a_check_that_cannot_run_must_say_so]]).

⭐ And prove the **wiring**, not only the predicate: arms that import the function test the rule,
while a correct function nothing calls is the reachability defect wearing a check. Drive the real
consumer end-to-end against a fixture whose digest is wrong, with every output redirected to scratch
so nothing tracked is mutated.
