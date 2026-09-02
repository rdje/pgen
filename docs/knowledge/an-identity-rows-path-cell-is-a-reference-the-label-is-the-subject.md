---
id: an-identity-rows-path-cell-is-a-reference-the-label-is-the-subject
title: In an instrument-identity table the PATH cell says where to look and the LABEL says what was hashed — read the label; and a new gate must not re-judge a row under a definition it does not own — defer to the owning oracle
answers:
  - "an identity table row re-hashes as MOVED but the owning gate says it is fresh — who is right"
  - "two identity rows share a path with different digests — is the table corrupt"
  - "how do I check an identity block generically when some rows are semantic or composite digests"
  - "should my new gate re-implement another doctrine's digest definition"
  - "what does raw ast in an identity row's label mean"
  - "how do I add a currency gate over artifacts other doctrines already read without double-judging them"
tags: [provenance, identity, digests, gates, doctrine, sv-corpus-grad]
date: 2026-09-03
status: current
evidence: "Three instances, one leaf's history. (1) SV-CORPUS-GRAD.13j(ii): `cost.md` and the v2005 census carry rows whose path cell is `grammars/systemverilog.ebnf` while the digest is of the frontend's raw AST — a literal re-hash reports both stale; neither is. (2) PGEN-SV-CORPUS-GRAD-0340: the adjudicator refused a correct report as 'TWO different digests for grammars/systemverilog.ebnf' because the byte `grammar` row and the semantic `grammar raw ast` row share a path. (3) PGEN-SV-CORPUS-GRAD-0343: the new IDENTITY-CARRIER-CURRENCY gate read `cost.md`'s `sample inputs` as MOVED (c3e01f2d… vs a byte re-hash of `stimuli/sv/parse_cost_sample.tsv`) while PARSE-COST-RATCHET read it fresh — that row digests the manifest ORDER plus every sampled file's bytes, as the prose beside it says."
reverify: "bash scripts/check_identity_carriers.sh --report | grep -c 'deferred to'   # >= 1: the composite row is deferred, not failed; bash docs/tasks/artifacts/sv_corpus_grad/identity_carriers/probe.sh  # 8/8"
---

**Two rules, three instances.**

## 1. The label is the subject

An identity row `| <label> | `path` | `sha` |` records *a digest that the producer defined*. The path
cell is where a reader should look; the label is what was hashed. Three shapes already live in this
repository:

| label | what the digest is of |
|---|---|
| `grammar`, `manifest`, `results (output)` | the bytes of the file at the path |
| `grammar raw ast` · `` `path` (ebnf_raw_ast) `` | the frontend's `raw_ast` envelope of that grammar — comments and layout cannot move it |
| `sample inputs` | the manifest's ORDER plus every listed file's bytes — a composite the path cell only points at |

A reader that hashes the path cell for every row is wrong on the second and third shapes, and wrong in
the **failing** direction — it reports a fresh report stale, which teaches people to waive the gate.

## 2. Defer, do not re-implement

The tempting fix for the composite row is to teach the new gate the ratchet's digest definition. That is
a second copy of a definition — `SV-CONTRACT-CURRENCY` and `BASELINE-IDENTITY` share one `ebnf_raw_ast`
implementation precisely so the copies cannot drift. The sound rule:

> A byte mismatch on a carrier that **another registered oracle already names** is that oracle's
> verdict — report it as a note naming the owner, never as a breach. A mismatch on a carrier **nobody
> else reads** is the breach it always was.

Owners are derived from the registry's script column on every run, the deferring gate excluded, so the
gate stays exactly the guard on un-owned carriers and never judges a row under a definition it does not
own ([[a-census-that-can-read-the-sentence-about-the-census-is-not-a-census]] for why the exclusion is
explicit).
