# GRAMMAR-CERT-STATUS — per-grammar certification status, derived and kept in sync

## Metadata

- **Created**: 2026-08-23 session #261, by **direct director order**.
- **Order (verbatim)**: *"please document which grammar certified and which are not and why and this
  information shall be readily available and queryable (KM card, mdbooks, ...)"* · *"I want for each
  grammar their status in a synthetic way documented in the main mdbook and kept in sync with the
  codebase."* · *"I should be able to really easily access that simple information."*
- **Provenance**: the order followed a director challenge to my answer to *"is the SV parser fully
  certified now?"* The answer took a dozen commands, and the challenge — *"that was your own
  definition of certification"* — was correct.

## Why this tree exists

Two measured failures, not one:

1. **The roster was DOC-ASSERTED.** `CHANGES.md` records rtl_const_expr's *"fully-certified-6 roster
   membership was doc-asserted only"*. A roster nobody derives is a claim that rots silently.
2. **A certification can be GREEN and STALE at once.** SystemVerilog's proof is dated a full day
   BEFORE the parser it describes (`summary.json` 2026-08-22 03:14 vs parser 2026-08-23 12:16;
   pinned `cc874b60…` vs on-disk `e53cb4a2…`) — and every registered doctrine still reported PASS,
   because a stale baseline inside its commit budget is a NOTE, not a failure.

⇒ **a status line without its freshness is not a status**, and a roster without a derivation is not
a roster.

## Leaves

### `.1` — DERIVE the per-grammar status, publish it in the main book, make it queryable (**`done`**, `PGEN-GRAMMAR-CERT-STATUS-0001`, 2026-08-23)

- **What landed**: `scripts/report_grammar_certification.sh` (the single source of truth; population
  DERIVED from `generated/*_parser.rs`, never a hand-kept list), the main-book page
  `docs/book/src/grammar-certification-status.md` carrying a `<!-- BEGIN DERIVED -->` block plus a
  `SUMMARY.md` entry, and the Knowledge Map card
  [[which-pgen-grammars-are-certified-and-which-are-not]] with nine `answers:` phrasings.
- **KEPT IN SYNC, mechanically**: `--check <page>` regenerates and DIFFS against the published block
  and exits nonzero on drift — the derive-and-diff pattern `KNOWLEDGE-MAP` already uses. Proven both
  ways in `.2`'s red control.
- **THE HEADLINE, and it is worse than the question that prompted it**: **certified_and_fresh = 0/9.**
  Only **2 of 9** shipped families carry a certificate-coverage contract at all; the other seven have
  **NO ORACLE** — nothing has ever scored their rule reachability, so there is no certification claim
  to be true or false. ⛔ `NO ORACLE` is not "broken": those families may be fine. The finding is
  that **nobody knows, and until now nothing said so**.
- **The two that were scored are both UNVERIFIED, for different reasons**: `systemverilog` is STALE
  (pins a parser that is not the one in the tree); `rtl_const_expr` is UNPINNED (its contract carries
  no `identity` block at all).
- **Scope stated on the page itself**: certification here is CERTIFICATE COVERAGE — can the generator
  reach every rule of the grammar. It is NOT a correctness claim about the language; that is the
  corpus axis. And for SV it is a UNION over four entry/profile configs, with 7 rules credited by
  PROOF rather than by a generated string and 11 unknown under the canonical config.

### `.2` — register the sync check as an enforced doctrine (`todo`, opened 2026-08-23)

- **WHY, and it is this tree's own lesson turned on itself**: `.1` ships a `--check` mode, a make
  target and a book page — but **nothing RUNS the check automatically**. That is precisely the shape
  that let the doc-asserted roster rot, and `GATE-REACHABILITY` already says *"a check nothing
  invokes is indistinguishable from one that does not exist"*.
- **WHAT IT NEEDS**: registration in `scripts/check_doctrines.sh` (doctrine count 25 → 26, with the
  `<meta:mirror>` list in `DOCTRINE_ENFORCEMENT.md` §10 and the `<meta:book-count>` figure in
  `docs/book/src/gate-flow.md` moved in lockstep — both are two-sided and will fail otherwise).
- ⚠️ **Honest**: until `.2` lands, the page is derived and diffable but **not enforced**. Stated here
  rather than implied by `.1`'s completeness.

### `.3` — give the seven `NO ORACLE` families a certification decision (`todo`, opened 2026-08-23)

- Seven of nine shipped families have never been scored: `json`, `regex`, `return_annotation`,
  `rtl_frontend`, `semantic_annotation`, `systemverilog_preprocessor`, `vhdl`.
- ⛔ **The work is a DECISION per family, not a sweep**: either stand up a certificate-coverage
  contract + gate (the `rtl_const_expr_cert_gate` shape), or record an explicit, reasoned
  `not-applicable` with its owner. **A family may not stay silent** — silence is what this tree
  exists to retire.
- ⚠️ Price it before starting: SV's own gate costs ~2 min/seed × 3 seeds × 4 configs.

## Acceptance Checklist (enforced)

- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow family. WHY: the per-grammar certification status
  had no producer — the roster was prose (`CHANGES.md`: *"doc-asserted only"*) and the one standing
  claim was stale. WHERE: no derivation existed; `git ls-files 'rust/test_data/grammar_quality/*cert*contract*.json'`
  returns **2** for **9** shipped parsers, and `bash scripts/report_grammar_certification.sh` now
  reports that gap by name instead of leaving it silent.
- [x] **ADDRESSED (verified)** — before: no command answered "which grammars are certified". After:
  one command answers it for all nine with a reason per row, the main book publishes the same table,
  a KM card makes it queryable by nine question phrasings, and `--check` proves the published page
  equals a fresh derivation (`grammar-certification: OK`).
- [x] **NO REGRESSION** — the slice touches no grammar, no Rust source, no codegen and no generated
  artifact; `generated/` is byte-identical by construction. All 25 doctrines PASS with the change
  staged, `mdbook_docs_gate` builds the new page, and `clippy` is not applicable — no Rust byte moved.
- **`promotion: declined (the durable lesson is already promoted as the KM card [[which-pgen-grammars-are-certified-and-which-are-not]], which is this slice's product rather than a side note)`**
