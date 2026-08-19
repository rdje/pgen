---
id: an-accept-set-watch-cannot-see-a-replaced-ast-shape
title: A parser change that REPLACES an AST shape moves no verdict, so an accept-set watch is blind to the break that hurts a consumer most — watch the tree, not just the answer
answers:
  - "my grammar change accepts and rejects exactly the same inputs — is it safe to ship without a contract entry"
  - "how do I tell whether a grammar commit is consumer-visible"
  - "when does a parser change need a schema bump rather than just a release bump"
  - "why did a pass-rate-green grammar fix break a downstream consumer"
  - "how do I derive which commits changed a parser's downstream behaviour, rather than remembering them"
  - "what identity should a contract record so it can tell when the grammar has moved past it"
  - "why is comparing grammar file bytes the wrong staleness check"
  - "how do I prove that a grammar edit is really comment-only"
tags: [contracts, downstream-integration, instrument-soundness, ast-shape, schema-versioning, doctrine, claim-verification, systemverilog]
date: 2026-08-19
status: current
evidence: docs/tasks/SV-CORPUS-GRAD.md leaf .13c.2l (PGEN-SV-CORPUS-GRAD-0241); docs/tasks/artifacts/sv_corpus_grad/contract_accept_set_ledger/{README.md,measure_accept_set_transitions.py,accept_set_transitions.tsv}; docs/contracts/PGEN_SV_GRAMMAR_REVISION_REGISTER.tsv; scripts/check_sv_contract_currency.sh; ledger rows SV-0054-SV-0062
reverify: "bash scripts/check_sv_contract_currency.sh   # SV-CONTRACT-CURRENCY: rows=… genesis=… newest=… digest=…  — then prove it can bite: bash docs/tasks/artifacts/sv_corpus_grad/contract_accept_set_ledger/probe_contract_currency.sh (7 arms, incl. the comment-only complement that must stay GREEN)"
---

A parser's downstream promise has **two** surfaces, and only one of them is a verdict.

1. **The accept set** — which texts parse. A change here is loud: a corpus pass rate moves, a
   reproducer flips, a consumer gets an error.
2. **The emitted tree** — what a parsed text *becomes*. A change here is silent: the same input,
   the same `ACCEPT`, a different shape. The consumer does not get an error; it gets wrong data.

Every instrument in a typical repository watches the first one.

## Measured

Between 2026-08-12 and 2026-08-19 the SystemVerilog grammar shipped **seven** semantically-distinct
revisions past a contract that still said release `1.0.183`. Replaying every pinned reproducer, on
every profile it declares, against every revision:

| revision | verdicts moved | **trees moved** |
|---|---|---|
| `-0220` cross-body function | (widen, unpinned) | **7** |
| `-0221` `PATHPULSE$` | 15 widen + 1 narrow | 0 |
| `-0227` dead receiver guard | 10 widen | **3** |
| `-0229` scope prefix | 4 widen | 0 |
| `-0232` covergroup footnote | 1 widen + 1 narrow | 0 |
| **`-0233` gate keywords + starving star** | **0** | **12** |
| `-0237` reserved-keyword hole | 16 narrow | **2** |

⛔ **`-0233` moved twelve trees and zero verdicts.** `bufif0 g(o, i, e);` parsed before the fix and
parses after; what changed is that it stopped arriving as a `udp_instantiation` — with the terminals
in an undifferentiated `inputs[]` list — and started arriving as a `gate_instantiation` with named
`output` / `input` / `enable` fields. A consumer matching on node type silently stops finding gates.
Four of the seven revisions replaced a shape this way; a watch built only on the accept set would
have caught **none** of the four, and the loudest of them not at all.

## The rule this gives you

- **Release bump** ⟸ any consumer-visible change, either surface.
- **Schema bump** ⟸ a **REPLACED** shape: text that parsed before now yields a different tree.
  Text that could not parse before is *additive* — it replaces nothing, so no schema move. Text
  that stops parsing is *removed* — also no schema move, because there is no surviving shape to
  compare.

That test is decidable by measurement (dump the tree on both sides of the change) and is not
decidable by reading a diff.

## Deriving the population — two instruments, no shared parent

A hand-kept list of "which commits are consumer-visible" is the same object as the debt it tracks:
the list in this very case went stale **twice inside the week it existed to repair staleness**, so a
change was published as *"the fifth"* when it was the seventh. Derive it instead, twice, in ways
that fail differently ([[a-conservation-control-cannot-catch-a-misassignment]] is the same principle
one level down):

- **A producer-derived identity.** Not the grammar file's bytes — hash what the *code generator
  consumes*. In PGEN that is the EBNF frontend's `raw_ast` envelope
  (`ast_pipeline --emit-raw-ast-json`, hashed over `raw_ast` alone). Comments never reach it, so a
  comment-only rewrite provably moves nothing, and anything a generated parser can observe provably
  moves it. Blind to behaviour: it says a revision *can* be observed, never that anything did.
- **A behavioural sweep.** Replay pinned reproducers × profiles × revisions and record **both**
  the verdict and the typed AST. Blind to grammar text, and bounded by what is pinned.

They agreed 9/9 on the partition, and their near-disagreement was the most useful output: one
revision moved the digest and no witness, which is how a **missing reproducer** was found. ⇒ report
`widen=0` as *"no PINNED witness moved"*, never as *"nothing changed"*.

## Then make it a gate, because the habit is what failed

Nothing in the repository compared the grammar's last-modified commit with the contract's — the
check was one `git log` away and did not exist. The durable form is a **revision register**: one row
per grammar revision, each either a `RELEASE` (contract section + ledger row) or `NEUTRAL`
(comment-only), and each carrying the producer-derived digest — so a `NEUTRAL` claim is refuted by
the register's own numbers when false, with no binary needed. Four tiers cover what each other
cannot: history (what already landed), the staged diff (the commit being made, which `git log`
cannot yet see), neutrality (self-refuting claims), and a re-derivation of the working tree.

⭐ Fire the complement arm. *"The digest is comment-insensitive"* is a claim until a comment-only
edit is fed to the gate and it stays **green** — without that arm you have only shown the gate can
fail, not that it fails for the right reason
([[a-check-whose-inputs-all-pass-has-not-been-tested]]).

Related: [[a-catch-all-alternative-makes-an-accept-meaningless]] (the same verdict-vs-arm blindness,
one level down, inside a single rule), [[a-rising-pass-rate-is-not-evidence-of-correctness]],
[[a-check-whose-inputs-all-pass-has-not-been-tested]],
[[a-conservation-control-cannot-catch-a-misassignment]].
