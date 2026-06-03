# PARSE-FIDELITY — the parser builds the CORRECT AST (no silent mis-parse; parser-agnostic)

> Task tree. **Metadata** — Status: `proposed` (literature-grounded; awaiting first leaf);
> Created: 2026-06-03; Roadmap lane: parser sign-off pillar **C** (of 4). Owns failure mode
> **(1.c) mis-parse** — the parser *accepts* the input but produces the **wrong AST**
> (silently, with no error signal). The most dangerous mode: a parser that accepts
> everything trivially passes reject/hang checks yet can be deeply wrong.
>
> Director-commissioned 2026-06-03 (one tree per failure mode; research-first). Parser-
> AGNOSTIC. Disciplines: [[feedback_research_grounded_sota_no_trial_and_revert]],
> [[feedback_corpus_expected_from_spec_not_fix]] (oracles must be independent of the fix),
> [[feedback_report_expected_verify_against_oracle]].

## The principle (binding)
A PGEN parser shall produce the AST the language *means*, not merely *an* AST. Because there
is no error signal for a mis-parse, fidelity needs **oracles** — independent ways to detect
"accepted but structurally wrong."

## What this is NOT
Not reject (Pillar A) or hang (Pillar B). The generator's coverage residual (Pillar D) is
the *engine* that exercises fidelity across the whole grammar — see the link below.

## Literature grounding (citations + worked mapping)
- ♻️ **Invertible syntax descriptions** — Rendel & Ostermann (Haskell 2010;
  `invertible-syntax` / `partial-isomorphisms`): derive parser **and** printer from ONE
  description, so round-trip is correct **by construction** (a whole class of mis-parses
  becomes impossible). PGEN's return-annotations already lean bidirectional; this is the
  principled end-state. See [[parse-fidelity-oracles]].
- ♻️ **Reference-free oracles** — **Metamorphic testing** (Chen, Cheung, Yiu 1998) +
  **EMI** (Le, Afshari, Su, PLDI 2014): a semantically-preserving source transform must
  yield an equivalent AST. Catches (1.c) where a reference parser can't (cross-tool ASTs
  differ).
- ✅ **Round-trip / unparse equivalence** — generate → unparse → reparse → identical AST.
  PGEN HAS `test_runner/round_trip_tests.rs` + per-grammar round-trip gates + the
  AST-shape-contract manifests.
- ✅ **Differential on a normalized projection** — full cross-tool AST diff is impractical;
  compare **semantic facts** vs slang's elaboration ("is X a type / net / parameter?").
- **Worked mapping:** PGEN's round-trip gates + shape contracts + semantic gates are the
  current (1.c) oracle; the **A5 `_meta` carrier** (`PARSE-SOTA.11`, designed) unlocks
  per-node `parse(node._meta.source_text) ≡ node`.

## The deep link (why Pillar D matters here)
**(1.c) is exhaustively proven exactly when Pillar D (stimuli coverage) reaches literal-0
AND every generated witness round-trips.** The stimuli closed loop is the mis-parse hunter
across the *entire* grammar; the differential corpus (Pillar A's harness, reused) is the
mis-parse hunter on *real* input. So PARSE-FIDELITY = round-trip ⊗ shape-contracts ⊗
stimuli-at-literal-0 ([[stimuli-residual-coverage-model]]) ⊗ semantic-projection-diff.

## Leaves
### `.1` — inventory existing fidelity oracles (audit, pure docs) — PENDING (next)
Map what round-trip gates + shape contracts + semantic gates already prove, per family, and
where the gaps are. ("Use what we have" before adding.)

### `.2` — per-node source round-trip via A5 `_meta` — PENDING (depends on PARSE-SOTA.11)
`parse(node._meta.source_text) ≡ node` for every node — the strongest local fidelity oracle.

### `.3` — metamorphic / EMI oracle — PENDING
Semantically-preserving source transforms (whitespace, comments, equivalent reorderings)
must preserve the AST. Reference-free.

### `.4` — differential semantic-projection vs slang — PENDING (reuses Pillar A harness)
Agreement on normalized semantic facts (is-X-a-type/net/param) on the real corpora.

### `.5` — invertible-syntax direction (research/design, long-term) — PROPOSED
Move the return-annotation core toward provable parser+printer-from-one-description so a
class of mis-parses is impossible by construction.

## Frontier
`.1` (oracle inventory). Strongest near-term lever is `.2` once `PARSE-SOTA.11` (`_meta`)
lands; exhaustive proof arrives with Pillar D at literal-0.
