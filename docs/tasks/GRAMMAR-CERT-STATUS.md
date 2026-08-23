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
- ⛔⛔ **THE HEADLINE `.1` PUBLISHED — `certified_and_fresh = 0/9`, seven families `NO ORACLE` — WAS
  FALSE AND IS RETRACTED. See `.1a`.** The measured truth is **6 of 9 certified**. Kept here rather
  than deleted because the failure, not the number, is what this tree exists to prevent.
- **What survives from `.1`**: `systemverilog` really is UNVERIFIED — its contract pins a parser that
  is not the one in the tree. That row was right for the right reason.
- **Scope stated on the page itself**: certification here is CERTIFICATE COVERAGE — can the generator
  reach every rule of the grammar. It is NOT a correctness claim about the language; that is the
  corpus axis. And for SV it is a UNION over four entry/profile configs, with 7 rules credited by
  PROOF rather than by a generated string and 11 unknown under the canonical config.

### `.1a` — ⛔⛔ **`.1` PUBLISHED A FALSE TABLE TO THE MAIN BOOK AND WAS CORRECTED THE SAME HOUR, UNDER A DIRECTOR CHALLENGE** (**`done`**, `PGEN-GRAMMAR-CERT-STATUS-0002`)

- **THE FALSE CLAIM**: `.1` published `certified_and_fresh = 0/9` with SEVEN families as `NO ORACLE`
  — *"nothing has ever scored their rule reachability"*. **Every part of that is wrong.**
- **THE CHALLENGE**: *"are you serious, some weeks back you claim that all of them except the
  systemverilog were certified and now you are saying none are … is the project even real, are we
  even progressing? please look in git log for when you made those claims."* The director's memory
  was correct and my table was not.
- **ROOT CAUSE, one line**: the script asked *"does a `rust/test_data/grammar_quality/*cert*contract*.json`
  file exist for this family?"* — **a filesystem question**. But the CERTIFICATE-COVERAGE REPORT is
  the oracle; a tracked contract is only a PIN on top of it, and only two families have ever needed
  one. ⇒ **ask the instrument, never the filesystem.**
- **MEASURED TRUTH, by running the oracle at HEAD** — and it reproduces the "fully-certified-6" the
  director remembered, recorded in `CHANGES.md` as *"the fully-certified set holds"*:

  | grammar | measured | verdict |
  |---|---|---|
  | `json` | `9/0/9/0` | ✅ certified |
  | `regex` | `269/9/260/0` | ✅ certified |
  | `rtl_const_expr` | `48/0/48/0` (at `--max-depth 32`) | ✅ certified |
  | `rtl_frontend` | `169/1/168/0` | ✅ certified |
  | `systemverilog_preprocessor` | `74/0/74/0` | ✅ certified |
  | `vhdl` | `225/0/225/0` | ✅ certified |
  | `return_annotation` | `35/0/33/`**`2`** | ⛔ 2 unknown |
  | `semantic_annotation` | `119/0/90/`**`29`** | ⛔ 29 unknown |
  | `systemverilog` | union 0 / canonical 11, proof STALE | ⚠️ unverified |

  **6 of 9 certified.** ⭐ And the project is measurably MOVING: `semantic_annotation` went
  `115/0/84/31` → `119/0/90/29` across this session's own `H.16.6b` work — total up because
  `map_key` added rules, unknown down because more are witnessed.
- ⛔⛔ **THIS IS THE THIRD TIME IN ONE SESSION THAT A VERDICT WAS TAKEN UNDER MY CHOSEN PARAMETERS
  INSTEAD OF THE SUBJECT'S OWN**, and it is the same defect each time: (1) answering *"is SV
  certified?"* out of a self-authored contract; (2) declaring `rtl_const_expr` broken at MY default
  `--max-depth 24` when its contract declares 32; (3) this table, asking the filesystem instead of
  the oracle. ⇒ the new script now reads each family's declared parameters, and the KM card carries
  the lesson as its closing section.
- ⚠️ **The damage was to the director's own review surface** — the main mdBook — which is exactly the
  surface `PARSER-BOOK-CURRENCY` exists to protect. Corrected in place with the retraction kept
  visible on the page rather than quietly overwritten.

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

- ⛔ **RE-SCOPED after `.1a`**: the premise *"seven families have never been scored"* was FALSE — six
  of them certify cleanly today. What actually remains is narrower and real: `return_annotation`
  (2 unknown) and `semantic_annotation` (29 unknown) are NOT certified and neither has an owning
  leaf, and six certified families have no tracked PIN, so their green is re-measured on demand
  rather than watched.
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
