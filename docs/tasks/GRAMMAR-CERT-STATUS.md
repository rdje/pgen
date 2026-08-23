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
  and exits nonzero on drift — the derive-and-diff pattern `KNOWLEDGE-MAP` already uses.
- ⛔⛔ **THAT SENTENCE WAS TRUE WHEN `.1` LANDED AND WAS MADE FALSE BY `.1a`'s OWN FIX COMMIT, WHICH
  DELETED THE IMPLEMENTATION AND KEPT THE FLAG. See `.1b`.** Its closing words — *"proven both ways
  in `.2`'s red control"* — described a red control that did not exist; `.2` was `todo`. Restored and
  actually proven both ways in `.1b`, five arms.
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

### `.1b` — ⛔⛔ **`.1a`'s CORRECTION DELETED `.1`'s `--check` IMPLEMENTATION AND LEFT THE FLAG BEHIND** (**`done`**, `PGEN-GRAMMAR-CERT-STATUS-0003`)

- **THE DEFECT**: `--check <page>` was **accepted and inert**. `-0002` — the commit that corrected
  `.1`'s false table — rewrote the script's rendering path and removed the derive-and-diff block
  wholesale, but the ARGUMENT PARSER kept `--check`. So the flag went on being accepted, set
  `MODE=markdown`, printed a fresh table to stdout and **exited 0**. It never opened the page.
- ⛔ **A CHECK THAT CANNOT GO RED UNDER ANY INPUT.** Measured at `c06292f4`, four arms, all exit 0
  and mutually indistinguishable:

  | arm | expected of a real check | measured |
  |---|---|---|
  | the real published page (in sync) | exit 0 | **exit 0** |
  | a page with the DERIVED block corrupted (`s/certified/TOTALLY-WRONG/g`) | exit 1 | **exit 0** |
  | a page carrying **no** DERIVED block | REFUSE (2) | **exit 0** |
  | `--check /nonexistent/no/such/page.md` | REFUSE (2) | **exit 0** |

- **WHERE, exactly**: `scripts/report_grammar_certification.sh` — `grep -n CHECK` returns exactly
  **two** lines, the declaration (26) and the assignment (30). The variable is never read again.
  `git log -S CHECK` dates it: the block was present in `-0001` (`7332ece7`) and deleted by `-0002`
  (`c06292f4`).
- ⛔⛔ **THE SHARPEST EVIDENCE IS THAT THE LEAF QUOTED OUTPUT NO CODE COULD PRODUCE.** `.1`'s
  acceptance line claims *"`--check` proves the published page equals a fresh derivation
  (`grammar-certification: OK`)"*. A repo-wide search for that string finds it in **exactly one
  place — this task file**. It is not in the script; it is not in any log. This is the
  `GENERATED-REPRODUCIBILITY` founding defect in the doc tier: **an artifact carrying a line its
  producer cannot emit.**
- ⛔ **THREE SURFACES PUBLISHED THE CAPABILITY WHILE IT DID NOT EXIST**, one of them the director's
  stated review surface: the book page (*"`--check …` refuses when the two disagree"*), this leaf,
  and layer-A `MEMORY.md` (*"`--check` derive-and-diff"*).
- ⭐ **ROOT CAUSE, one line**: **deleting an implementation while leaving its FLAG is worse than
  deleting the flag too** — an unknown-argument error would have been loud on the next run; a
  surviving flag made the loss silent and let three surfaces keep describing it.
- **THE FIX**: the diff is rebuilt on `-0002`'s architecture rather than reverted (the old block
  called `RGC_MODE=markdown emit`, a rendering path `-0002` replaced — a straight revert would not
  have run). The rendered markdown is captured once, and `--check` compares it to the page's
  `<!-- BEGIN DERIVED -->` block. Refusal codes now follow the repository convention the old block
  did not: **exit 2 = could not evaluate**, **exit 1 = evaluated and breached**, exit 0 = equal.
  A fresh derivation that comes back EMPTY is itself a REFUSAL, because comparing against an empty
  string passes by construction.
- **MEASURED BEFORE → AFTER** (same four arms, plus the two that must stay green):

  | arm | before | after |
  |---|---|---|
  | real published page, in sync | exit 0 (vacuous) | **exit 0** — `grammar-certification: OK (… 1579 bytes)` |
  | DERIVED block corrupted | exit 0 | **exit 1** + a unified diff naming both sides |
  | page with no DERIVED block | exit 0 | **exit 2** REFUSED |
  | `--check` on a nonexistent path | exit 0 | **exit 2** REFUSED |
  | a fresh derivation that comes back EMPTY | (no guard existed) | **exit 2** REFUSED |
  | `--markdown` (unchanged path) | 1635 B | **1635 B, byte-identical** |
  | plain text mode (unchanged path) | `certified=6/9` | **`certified=6/9`** |

- ⭐ **A REFUSAL MUST BE CHEAP, AND THE FIRST CUT OF THIS FIX WAS NOT.** It validated `--check`'s
  subject only AFTER rendering, so a typo'd page path cost a full **65 s** derivation to be told the
  file does not exist. The guard moved to argument-parse time and the same arm now refuses in
  **0.00 s** — the posture `sv_cert_recognized_union_gate` already takes (refuse immediately, naming
  the reason).
- ⚠️ **Honest scope of arm 5**: the empty-derivation guard is exercised by extracting the real check
  block (26 lines, verbatim) and running it with `RGC_FRESH=""`, because no input to the shell
  wrapper makes a real derivation empty. It is the shipped code, driven directly — not a
  reimplementation — and that distinction is stated rather than left for a reader to assume.

- ⭐ **The green is NOT vacuous**: the published block and a fresh derivation are byte-equal at
  **1579 bytes** today, and the derivation itself is **deterministic** — two independent runs of
  `--markdown` are byte-identical at 1635 bytes, which is what makes it safe to gate on at all.
- ⚠️ **Cost, measured before designing the gate**: a full derivation is **65.2 s** (8 families
  through the real oracle at `--count 40`). That is why `.2` gets TIERS rather than a per-commit
  oracle run.

- ⭐ **ROUTED FINDING, measured while correcting this tree's index row** — `docs/TASK_TREE.md`'s
  Active table declares **five** columns (`Tree | Status | Roadmap lane | Current frontier | File`)
  and **5 of its 66 rows carried only four**, silently dropping the link to the tree file: this tree,
  plus `ENGINE-UNIVERSAL-SERVICES`, `LIVE-MEANS-LIVE`, `DONE-BAR` and `CI-PARITY-GATE-ROT`. This
  row is fixed here; **4 remain**, and nothing checks the shape — a row that loses its last cell
  renders as a normal row, so the omission is invisible to a reader. ⚠️ Not swept: four of the five
  belong to other trees and the SV lane lock binds WORK, not routing. Detection is one line
  (`"](tasks/" not in row`), which is why it is worth a leaf rather than a note.

- ⛔⛔ **THE ACCEPTANCE CHECKLIST BELOW WAS WRITTEN BUT THE GATE DID NOT BIND IT, AND THAT IS A
  SECOND GAP THIS LEAF EXPOSES.** `TASK-ACCEPTANCE` decides "is this a code change" from a path
  list, and `scripts/report_grammar_certification.sh` matches **none** of its eleven patterns
  (`grammars/*.ebnf`, `rust/src/*`, `generated/*`, `rust/test_data/ast_shape_contract/*.json`,
  `scripts/check_*.sh`, `rust/scripts/*.sh`, `.githooks/*`, `.github/workflows/*.yml`,
  `rust/build.rs`, `Makefile`, `rust/Makefile`) — so it reported
  `diag-evidence: OK (no code change staged; task-acceptance checklist not required)`.
  ⇒ **the one file whose silent breakage this entire leaf is about sits OUTSIDE the gate that exists
  to stop silent breakage.** The checklist is written anyway because the doctrine binds the author
  whether or not the enforcer can see the file; the gap is routed into `.2`, which must widen the
  scope rather than merely add a wrapper that happens to be in it.

#### Acceptance Checklist — `.1b`

- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow family. **WHY**: `-0002` rewrote the rendering
  path and deleted the derive-and-diff block with it, while the argument parser kept `--check`; a
  surviving flag makes a deleted implementation SILENT, where an unknown-argument error would have
  been loud on the next run. **WHERE**: `scripts/report_grammar_certification.sh`, the `--check`
  arm of the option loop. Dated by the tool, not by reading:

  ```
  $ git log --all -S"RGC_MODE" --oneline -- scripts/report_grammar_certification.sh
  c06292f4 PGEN-GRAMMAR-CERT-STATUS-0002 (leaf GRAMMAR-CERT-STATUS.1a …)
  7332ece7 PGEN-GRAMMAR-CERT-STATUS-0001 (tree GRAMMAR-CERT-STATUS created …)
  ```

  — the rendering path `--check` called (`RGC_MODE=markdown emit`) is ADDED by `7332ece7` and
  REMOVED by `c06292f4`, which is why a straight revert would not have run. `bash -n
  scripts/report_grammar_certification.sh` is clean before and after, so no syntax check could ever
  have seen this: the script was always VALID, it just stopped doing the thing.
- [x] **ADDRESSED (verified)** — before→after on the symptom, five arms, each observed:
  **before** all four exit 0 and are mutually indistinguishable (real page / corrupted DERIVED block
  / no DERIVED block / a path that does not exist); **after** exit 0 with
  `grammar-certification: OK (published table equals a fresh derivation, 1579 bytes)`, exit 1 with a
  unified diff naming the drifted row, exit 2 REFUSED, exit 2 REFUSED, plus a fifth arm — an empty
  fresh derivation — refusing at exit 2 rather than passing by construction. The green is not
  vacuous: the published block equals a fresh derivation at **1579 bytes** today, and a typo'd path
  now refuses in **0.00 s** instead of after a 65 s derivation.
- [x] **NO REGRESSION** — the two unchanged output paths were re-measured and are **byte-identical**:
  `--markdown` produces the same **1635** bytes as the pre-change run (`diff -q` clean), and text
  mode still reports `certified=6/9` with `spf=0` on every family. The slice touches **no grammar,
  no Rust source, no codegen and no generated artifact** — `generated/` is byte-identical by
  construction — so `clippy` and the seed-0/7/42 oracle gates are not applicable and are not
  claimed. Determinism was proven before gating on it at all: two independent `--markdown` runs are
  byte-identical.
- **`promotion: promoted`** — the durable lesson is now a retrievable card,
  [[deleting-an-implementation-while-leaving-its-flag-fails-silently]], carrying the four-arm
  measurement, the cheap `grep -n <VAR>` detector and the nonexistent-path red control that any
  inherited gate can be run through.

### `.2` — register the sync check as an enforced doctrine (**`done`**, `PGEN-GRAMMAR-CERT-STATUS-0004`, 2026-08-23)

- **WHY, and it is this tree's own lesson turned on itself**: `.1` ships a `--check` mode, a make
  target and a book page — but **nothing RUNS the check automatically**. That is precisely the shape
  that let the doc-asserted roster rot, and `GATE-REACHABILITY` already says *"a check nothing
  invokes is indistinguishable from one that does not exist"*.
- **WHAT IT NEEDS**: registration in `scripts/check_doctrines.sh` (doctrine count 25 → 26, with the
  `<meta:mirror>` list in `DOCTRINE_ENFORCEMENT.md` §10 and the `<meta:book-count>` figure in
  `docs/book/src/gate-flow.md` moved in lockstep — both are two-sided and will fail otherwise).
- ⚠️ **Honest**: until `.2` lands, the page is derived and diffable but **not enforced**. Stated here
  rather than implied by `.1`'s completeness.
- ⛔⛔ **BINDING REQUIREMENT ADDED BY `.1b`, AND IT IS NOT SATISFIED BY A WRAPPER.** Registering
  `scripts/check_grammar_certification.sh` puts the WRAPPER inside `TASK-ACCEPTANCE`'s path scope
  while the PRODUCER — `scripts/report_grammar_certification.sh`, the file that actually broke —
  stays outside it. A future edit to the producer alone would again require no checklist, which is
  exactly the edit `.1b` is about. `.2` must therefore widen the scope to cover the producer, not
  just land a wrapper that is already in it.
- ⚠️ **PRICE IT BEFORE DESIGNING IT — MEASURED, not estimated**: a full derivation is **65.2 s**
  (8 families through the real oracle at `--count 40`). That is far too slow for a pre-commit hook,
  so this needs the TWO-TIER shape `GENERATED-REPRODUCIBILITY` and `PARSE-COST-RATCHET` already
  established — a cheap every-commit IDENTITY tier over the inputs the table is a function of (the
  grammars, the generated parsers, the producer script, the SV union contract), and the ORACLE tier
  on demand. ⛔ And `GENERATED-REPRODUCIBILITY`'s own recorded trap applies directly: a cheap tier
  that PRESCRIBES re-running the oracle inherits the oracle's blind spot.
- ⭐ **What is already proven and need not be re-established**: the derivation is DETERMINISTIC (two
  independent `--markdown` runs byte-identical at 1635 B), the published block equals a fresh
  derivation at 1579 B, and the check now has five observed arms including two REFUSALS.

#### What landed — `GRAMMAR-CERT-CURRENCY`, doctrine **26**

- **`scripts/check_grammar_certification.sh`**, registered in `scripts/check_doctrines.sh` with its
  `DOCTRINE_ENFORCEMENT.md` §10 mirror row and the `docs/book/src/gate-flow.md`
  `<!-- DOCTRINE-COUNT -->` figure moved in lockstep — **25 → 26**, all three two-sided. The two
  meta-checks were observed FAILING on the intermediate state (registry ahead of both mirrors) and
  passing once they caught up, so the lockstep is measured, not assumed.
- ⭐⭐ **TWO TIERS, PRICED FIRST.** A full derivation is **65.2 s**, which may not be spent on every
  commit. So tier 1 (**0.27 s**, no oracle, no build) answers everything that does not need the
  table's NUMBERS, and tier 2 (`--oracle`, **64.5 s**) re-derives and diffs.
- **TIER 1's FOUR ARMS**: (1) the page carries exactly ONE well-formed `<!-- BEGIN DERIVED -->` …
  `<!-- END DERIVED -->` block; (2) the producer's `--check` is WIRED — statically, the option
  variable must be READ rather than merely declared and assigned, and behaviourally, it must refuse
  a path that does not exist; (3) the published POPULATION equals the families that actually ship;
  (4) no declared input — the producer, each published family's grammar, the SV union contract —
  leads the page past a budget.
- ⛔ **TIER 1 TESTS THE INSTRUMENT RATHER THAN TRUSTING IT, AND THAT IS `.1b` TURNED INTO A GUARD.**
  Arm (2) is the one-line detector from `.1b` promoted into an enforcer: in the broken vintage
  `grep -c '\$CHECK'` was **0**. Two of the nine self-test arms reconstruct that regression from
  both sides — a producer that parses `--check` into a variable it never reads, and one that reads
  it but still returns 0 for a nonexistent page.
- ⭐ **TIER 2 DRIVES A RED CONTROL BEFORE IT BELIEVES A GREEN ONE**: it first requires the producer
  to REFUSE a page with no DERIVED block, so a green diff can never come from an instrument that
  says green to everything.
- ⭐⭐ **PREDICTED 65 s, FIRST MEASURED 128.5 s — AND THE PREDICTION IS KEPT.** The red control was
  paying for a whole second derivation to be told something a `grep` knows. Fixed in the PRODUCER,
  not in the gate: "does this page have a DERIVED block" is a property of the PAGE ALONE and now
  refuses at argument-parse time in **0.01 s**, bringing tier 2 to **64.5 s** — one derivation, as
  predicted. ⛔ Re-reading could never have caught this; only running it could. Same class as the
  `.1b` refusal that cost 65 s before its guard moved.
- ⚠️ **HONEST BOUNDS, stated before the check was trusted, not discovered afterwards**: tier 1 is a
  STALENESS argument and inherits whatever tier 2 last established (`PARSE-COST-RATCHET`'s own words
  about itself); its behavioural `--check` arm is satisfied by the early existence guard alone, so
  the arm that really proves the DIFF still runs needs the derivation and lives in tier 2; and
  `generated/` is untracked, so a fresh clone reports the population arm **NOT EVALUATED, loudly** —
  never as a pass.
- ⭐ **STALENESS IS BUDGETED, NOT ZERO-TOLERANCE** (`PGEN_GRAMMAR_CERT_LEAD_BUDGET`, default 10).
  `BASELINE-IDENTITY` already paid for that lesson: its first cut made a moved input a hard failure
  inside a pre-commit enforcer, and ONE COMMENT LINE in the SV grammar blocked every commit in the
  repository. Inside the budget a lead is a printed NOTE; past it, a breach — so rot stays
  impossible without ambushing an unrelated commit.
- ⭐⭐ **THE SCOPE GAP `.1b` NAMED IS CLOSED, AND IT WAS NOT SATISFIED BY THE WRAPPER.**
  `TASK-ACCEPTANCE` decided "is this a code change" from a path list that caught the ENFORCER
  (`scripts/check_*.sh`) but not the PRODUCER it asks — the file that actually broke. `scripts/report_*.sh`
  is now on the proof surface, on the recorded ground that **the oracle a doctrine consumes is part
  of the proof surface too**.
- **PRICED AS A WIDENING MUST BE** (the `.4`/`.7` bar): `git ls-files 'scripts/report_*.sh'` → **1**
  file; **3** commits in the entire history touch it — `7332ece7` (published a false table),
  `c06292f4` (deleted the check), `b86d95df` (restored it). All three are this defect's own
  incidents and all three staged no other in-scope path, so the pattern binds exactly the commits it
  exists for and nothing else in ~2 600.
- ⛔⛔ **AND THE REPLAY THAT PROVES IT EXPOSED A TRAP IN THE ENFORCER'S OWN REPLAY MODE.**
  `PGEN_DIAG_EVIDENCE_RANGE` takes the changed-line RANGES from history but reads the checklist
  CONTENT from the WORKING TREE, so replaying `7332ece7..c06292f4` in place reported **OK** — on
  today's task file, which now carries `.1b`'s checklist. That is not a historical verdict. Re-run
  as a FAITHFUL replay in a throwaway `git worktree` at `c06292f4`, with a two-arm control:

  | gate | tree | range | exit |
  |---|---|---|---|
  | **widened** (this slice) | `c06292f4` | `7332ece7..c06292f4` | **1 — BLOCKED** |
  | as it actually shipped | `c06292f4` | `7332ece7..c06292f4` | **0** — *"no code change staged"* |

  ⇒ the widening is what turns the verdict, and **the commit that deleted the check would have been
  blocked by it**. ⛔ The in-place reading would have let me publish the opposite conclusion; it was
  caught by asking why a RED I expected came back GREEN, not by review.

#### Acceptance Checklist — `.2`

- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow family. **WHY**: `.1` shipped a derive-and-diff
  and nothing invoked it, which is `GATE-REACHABILITY`'s own founding sentence applied to this
  tree — *a check nothing invokes is indistinguishable from one that does not exist*. **WHERE**: the
  registry in `scripts/check_doctrines.sh` held no row for it, and `TASK-ACCEPTANCE`'s path list did
  not cover the producer. Both re-derived by command, not by reading:

  ```
  $ git ls-files 'scripts/report_*.sh'        ->  scripts/report_grammar_certification.sh   (1 file)
  $ git log --oneline --all -- 'scripts/report_*.sh' | wc -l   ->  3
  ```

  and the faithful worktree replay above, whose two arms differ only in which gate ran. `bash -n` is
  clean on every script touched.
- [x] **ADDRESSED (verified)** — before: nothing ran the check, and an edit to the producer required
  no acceptance checklist at all. After: the doctrine driver reports
  `✓ PASS GRAMMAR-CERT-CURRENCY` among **ALL 26 enforced doctrines PASS**, tier 1 costs 0.27 s,
  tier 2 re-derives and diffs in 64.5 s (`the published table equals a fresh derivation, 1579
  bytes`), `--self-test` reports **9/9 arms as expected** with each breach observed failing for the
  right printed reason, and both meta-checks were seen RED on the intermediate state before going
  green. The widened acceptance gate blocks `c06292f4` in a faithful replay.
- [x] **NO REGRESSION** — the slice touches **no grammar, no Rust source, no codegen and no
  generated artifact**, so `generated/` is **byte-identical** by construction and `clippy` does not
  apply; the seed-0/7/42 oracle gates are untouched and are not claimed. The producer's four
  behavioural arms are unchanged after the early-guard edit (0 / 1 / 2 / 2), its `--markdown` output
  is unchanged, and the certification table itself still reads `certified=6/9` with `spf=0` on every
  family. `mdbook_docs_gate` green.
- **`promotion: declined (the durable lesson of this slice — deleting an implementation while
  leaving its flag — is already promoted by `.1b` as
  [[deleting-an-implementation-while-leaving-its-flag-fails-silently]]; what `.2` adds is the
  enforcement of it, which belongs in the doctrine registry and its mirror rather than in a second
  card)`**

### `.3` — give every family without a clean certification a DECISION, and pin the six that have one (**`done`**, `PGEN-GRAMMAR-CERT-STATUS-0005`, 2026-08-23; RE-SCOPED by `.1a` — the original title said *seven `NO ORACLE` families*, a premise `.1a` refuted)

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

#### The adjudication — all **31** uncertified rules, classified with measured evidence

⭐ **SEED-STABLE BEFORE ANYTHING WAS CONCLUDED FROM IT.** Both the COUNTS and the rule SETS are
byte-identical at seeds **0 / 7 / 42** (`--count 40`), so this is a claim about the grammars, not
about one sample. A small-sample result is a claim about the sample until that is shown.

**`return_annotation` — 2 of 35**

| rule | measured verdict | class |
|---|---|---|
| `accessor_base` | ⭐ **NOT A DEFECT — PGEN's own LR-elimination pass eliminated it.** `--lint-grammar` says so in its own words (*"ELIMINATED 1 left-recursive rule(s) on this grammar: accessor_base"*), and the shipped parser carries its replacements: `parse_accessor_base_lr_base` and `parse_accessor_base_lr_suffix`. The language is covered; the NAME is not. | **A — engine artifact** |
| `parenthesized` | defined at line 196 and **referenced by nothing**. `grep -nw` returns exactly one hit, its own definition. In the shipped parser its only non-definition occurrence is the by-name entry dispatcher (`"parenthesized" => self.parse_parenthesized()`), which is entry selection, not a production call site. | **C — dead orphan rule** |

**`semantic_annotation` — 29 of 119**, and they partition exactly (9 + 20 = 29):

| class | rules | measured verdict |
|---|---|---|
| **B — trivia, unreachable BY DESIGN** (4) | `whitespace`, `line_comment`, `block_comment`, `doc_comment` | the grammar's own section header is *"WHITESPACE AND COMMENTS"* and `whitespace := /\s+/` is commented *"(ignored)"*. These are lexical/skip constructs; no production reaches them and none should. `block_comment` already carries a `GRAMMAR-WELLFORMED.H.17.1` adjudication in the grammar text. |
| **C — a duplicate of the entry rule** (1) | `annotation` (line 36) | ⛔ its right-hand side is **byte-identical** to the entry `semantic_annotation` (line 32) — compared programmatically, not by eye: `"@" /\s*/ annotation_name /\s*/ ":" /\s*/ annotation_value`. It is unreachable from the entry because it **is a second copy of the entry**. |
| **A — LR-elimination artifact** (2) | `union_type`, `intersection_type` | ⛔⛔ **`.3` FIRST PUT THESE IN CLASS D AND THAT WAS WRONG — RETRACTED, see the correction below.** They are alternatives of `type_reference`, which `--lint-grammar` reports as *"ELIMINATED 1 left-recursive rule(s) on this grammar: type_reference"*, and the shipped parser carries `parse_type_reference_lr_base` / `parse_type_reference_lr_suffix`. Same class as `accessor_base`, one grammar over. |
| **D — specialized value shapes never wired in** (22) | `precedence_value`+`precedence_level`+`precedence_associativity` · `constraint_value`+`constraint_type`+`constraint_expression` · `performance_value`+`complexity_spec`+`complexity_expr`+`memory_spec`+`memory_amount`+`memory_unit`+`timing_spec`+`time_amount`+`time_unit` · `version_value`+`semantic_version`+`version_range` · `exception_spec`+`exception_type` · `platform_spec`+`platform_name` | ⛔ **`annotation_value` routes to exactly four families** — `primitive_value \| structured_value \| expression_value \| reference_value` (line 101) — and to **none** of these. Six feature islands, each rooted at an unreferenced orphan, sitting under the grammar's own *"SEMANTIC PATTERNS AND SPECIALIZED VALUES"* header. They are written, and nothing routes to them. |

- ⛔⛔ **THE PARTITION ITSELF WAS MIS-MEASURED ONCE AND CORRECTED BY RE-MEASURING, NOT BY REVIEW.**
  The first census counted rule names occurring **inside COMMENT lines** as references, and reported
  6 orphan roots / 23 island members. `annotation` scored "6 references" — every one of them the
  English word *annotation* in a comment. Re-run over production text only: **9 orphans / 20 island
  members**, and the two halves close on 29 exactly. ⇒ **a census over a file's TEXT measures its
  prose too**; count over the productions, never over the lines.
- ⭐ **NO INSTRUMENT DISAGREEMENT — I checked before claiming one.** `--lint-grammar` reports
  `unreachable_rules=0` for both grammars while the certificate pass reports these rules as having
  no reach path, which looks like a contradiction and is not. `detect_unreachable_rules`'s own
  doc comment defines its roots as *"the canonical entry PLUS every rule that NOTHING references"*
  and is explicitly *"conservative (an unreferenced dead orphan is treated as a root → not flagged;
  only referenced-but-unreachable dead ISLANDS are caught)"*. So the two answer different questions:
  the linter asks *is anything a stranded island*, the certificate pass asks *what can the generator
  reach from the declared entry*. Both are right, and the classes above are the difference between
  them.

#### ⛔⛔ CORRECTION to `.3` as first committed (`3b9e009d`) — **2 of the 29 were in the WRONG CLASS**

- **THE ERROR**: `union_type` and `intersection_type` were classified **D** (*"specialized value
  shapes never wired in"*). They are **A** — PGEN's own LR-elimination artifact — exactly like
  `accessor_base` in `return_annotation`. So class D is **22**, not 24, and the partition is
  **2 + 4 + 1 + 22 = 29**, which still closes.
- **HOW IT WAS CAUGHT**: not by re-reading. Checking whether any downstream consumer referenced the
  22 before deciding their fate turned up `union_type` and `intersection_type` in
  `docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md`, which states that
  *"`type_reference` is left-recursive through four wrapper alternatives (`union_type`,
  `intersection_type`, `array_type`, `optional_type`), so PGEN eliminates the recursion."* Verified
  against the instruments rather than the contract: `--lint-grammar` names `type_reference` as the
  eliminated rule, and the shipped parser carries `parse_type_reference_lr_base` /
  `parse_type_reference_lr_suffix`.
- ⛔ **ROOT CAUSE OF MY MISCLASSIFICATION**: I partitioned by *where the rules sit in the file* — the
  six islands all live under the *"SEMANTIC PATTERNS AND SPECIALIZED VALUES"* banner — and
  `union_type`/`intersection_type` do **not**; they sit at lines 472/475, inside the type system.
  I swept them in with the islands because they shared the *symptom* (no reach path), never asking
  whether they shared the *mechanism*. ⇒ **CLASSIFY BY MECHANISM, NOT BY THE SYMPTOM THEY SHARE** —
  and note the tell was in the numbers all along: they were the only two class-D members with a
  reference count above 1 that were not island-internal.
- ⭐ **THE CONSUMER CHECK IS WHAT FOUND IT, AND IT WAS RUN FOR A DIFFERENT REASON.** It was meant to
  ask *"would removing these break anyone"*; it answered *"two of them are not what you think"*.

#### The decisions — no family and no class stays silent

| class | decision | owner |
|---|---|---|
| **A** `accessor_base` | **`not-applicable`, reasoned**: the rule is an LR-elimination *input*, and its `_lr_*` replacements are witnessed. Certifying the pre-elimination NAME would be certifying a rule the shipped parser deliberately does not have. This is the established `casting_type` precedent, already doctrine-backed by `SV-RULE-FIRE-PARTITION`. | closed here |
| **B** the 4 trivia rules | **`not-applicable`, reasoned**: lexical/skip constructs are unreachable from the entry BY CONSTRUCTION. ⭐ The principled long-term home is a verified unreachability **PROOF** — the definition already admits proof as an alternative to a witness — which would move them out of UNKNOWN honestly rather than by exemption. Priced and **not** taken here: that is engine work on the proof generator, not a grammar edit. | routed → `GRAMMAR-WELLFORMED` (proof-side) |
| **C** `annotation`, `parenthesized` | ⛔ **REAL DEFECTS, and the fix is a GRAMMAR edit**: one is a byte-identical duplicate of its own entry rule, the other is referenced by nothing. Both are dead weight that makes two grammars uncertifiable for no benefit. **NOT done in this slice** — a grammar edit to the annotation pair is a code change that regenerates the parsers every other family's generation depends on, and it is not SV-lane work. | routed → new leaf `.4` |
| **D** the 24 specialized value shapes | ⛔⛔ **A DIRECTOR CALL, NOT MINE — it changes what the annotation grammar MEANS.** Wiring `precedence_value` and friends into `annotation_value` would make `@precedence: 5 left` parse *structurally* instead of falling through to a generic `primitive_value`, i.e. it changes the **AST SHAPE** a downstream consumer reads for those annotations. That is a scope decision with a contract consequence, not an implementation detail. Surfaced. | **awaiting director** → recorded in `.4` |

- ⭐⭐ **AND THE OTHER HALF OF `.3` — "pin the certified six" — IS DISCHARGED BY `.2`, NOT SKIPPED.**
  The stated worry was that the six certified families have no tracked PIN, so *"their green is
  re-measured on demand rather than watched"*. `GRAMMAR-CERT-CURRENCY` now watches them **through the
  table**: the published block carries each family's `total · witness · proof · unknown` and its
  certified verdict, tier 1 flags any grammar that leads the page past its budget, and tier 2
  re-derives and DIFFS every row. A family falling out of certification moves its row and turns the
  diff RED. ⇒ six separate cert contracts would be a **second** home for a fact the table already
  holds, and this repository has measured what a second home costs (`ENGINE-UNIVERSAL-SERVICES.33`,
  the mirrored recipe). **Declined with a reason, which is what this leaf demands — not silence.**
- ⚠️ **HONEST RESIDUAL**: the six are watched for *drift*, not *re-proved on a schedule*. Tier 2 is
  operator-invoked, so "watched" means a diff will catch a change whenever tier 2 runs — it does not
  mean the numbers are re-derived every commit. Stated because `.1`'s failure was exactly a status
  that read stronger than its evidence.

#### Acceptance Checklist — `.3`

- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow + correctness diagnosis. **WHY** each family is
  uncertified is now measured rather than named: `return_annotation`'s 2 are one LR-elimination
  input and one orphan; `semantic_annotation`'s 29 are 4 trivia rules, 1 duplicate of the entry, and
  24 specialized value shapes `annotation_value` never routes to. **WHERE**, by instrument:
  `PGEN_CERT_COVERAGE_DUMP_ALL=1 … --report-certificate-coverage` named all 31 and volunteered
  `WARNING plannable-rule reach pass: N UNKNOWN rules have NO reach path from the entry (dead-rule
  candidates — adjudicate via the linter)`; `--lint-grammar` supplied the LR verdict verbatim
  (*"ELIMINATED 1 left-recursive rule(s) … accessor_base"*); and `grep -nw` over the productions
  located every definition and reference. Seed-stable: identical COUNTS **and** identical rule SETS
  at seeds 0 / 7 / 42.
- [x] **ADDRESSED (verified)** — before: 31 rules sat as an unexplained `UNKNOWN` total on a page the
  director reads, with no owner and no reason. After: all 31 are classified into four classes with
  measured evidence per class, each class carries an explicit decision and an owner, the two that
  are genuine defects are routed to `.4`, the one that is a scope question is surfaced to the
  director rather than silently decided, and `.3`'s second half is **declined with a reason** —
  `.2`'s doctrine already watches the certified six through the published table, so six extra cert
  contracts would be a second home for a fact the table holds. ⭐ One measurement was wrong and was
  corrected by re-measuring: the first orphan/island census counted rule names inside COMMENTS as
  references (6/23); over production text only it is 9/20, and 9+20 closes on 29 exactly.
- [x] **NO REGRESSION** — an adjudication slice: **zero grammar, Rust, codegen and generated bytes**,
  so `generated/` is **byte-identical** by construction and `clippy` does not apply. The
  certification numbers themselves are untouched and re-verified unchanged — `certified=6/9`,
  `spf=0` on every family, `return_annotation` `35/0/33/2` and `semantic_annotation` `119/0/90/29`
  reproducing at seeds 0/7/42.
- **`promotion: declined (the durable, general lesson here — a census over a file's TEXT counts its
  PROSE too, so count over the productions — is a re-statement of the already-promoted
  [[a-heading-census-is-only-as-good-as-the-heading-grammar]] in a different namespace; the
  family-specific adjudication belongs in this leaf and on the book page, not in a retrieval card)`**

### `.4` — the two dead rules, and the director call `.3` surfaced (`todo`, opened 2026-08-23 by `.3`)

- ⛔ **(a) TWO GRAMMARS CARRY A DEAD RULE EACH, AND BOTH ARE WHY THEIR FAMILY CANNOT CERTIFY.**
  `grammars/semantic_annotation.ebnf:36` defines `annotation` with a right-hand side **byte-identical**
  to its own entry rule at line 32; `grammars/return_annotation.ebnf:196` defines `parenthesized`,
  which `grep -nw` shows is referenced by nothing. Removing them would take
  `semantic_annotation` 29 → 28 UNKNOWN and `return_annotation` 2 → 1, and neither can change the
  accepted language: an unreferenced rule has no call site to lose.
- ⚠️ **PRICE AND RISK, stated because this is not a free edit**: both are annotation grammars, so the
  edit regenerates the parser PAIR that the annotation backend links to generate **every other
  family** — `GENERATED-REPRODUCIBILITY`'s founding artifact. It needs a regeneration, an
  `emission_sha` rebaseline, and `parse_harness_equivalence_gate` as the oracle that the shipped
  parse did not move. That is a slice of its own, not a footnote to an adjudication.
- ⛔ **NOT STARTED HERE, DELIBERATELY**: the SV lane lock binds WORK, and this is a non-SV grammar
  change with a codegen blast radius. Sequenced, not abandoned.
- ⭐⭐ **(b) DIRECTOR CALL — `semantic_annotation`'s 24 specialized value shapes.** Six feature
  islands (`precedence`, `constraint`, `performance`/`complexity`/`memory`/`timing`, `version`,
  `exception`, `platform`, plus `union_type`/`intersection_type`) are fully written and
  **`annotation_value` routes to none of them** — it offers exactly
  `primitive_value | structured_value | expression_value | reference_value`. Two readings, and they
  are not equivalent:
  1. **Wire them in.** `@precedence: 5 left` then parses STRUCTURALLY (`{type: "precedence", level,
     associativity}`) instead of falling through to a generic value. ⛔ This **changes the AST shape**
     a downstream consumer reads for those annotation kinds — a contract change, with
     `SV-CONTRACT-CURRENCY`-shaped obligations.
  2. **Declare them future work** and record it, leaving 24 rules honestly uncertifiable until then.
- ✅ **DIRECTOR DELEGATED THE CALL TO ME** (2026-08-23: *"it is yours to make but it got to be sota,
  signoff"*). **DECISION: DO NOT WIRE THEM IN. REMOVE THEM.** They are superseded design, not
  missing wiring. The evidence, measured before deciding:

  1. ⭐⭐ **THE GRAMMAR ALREADY DOCUMENTS THE INTENDED SPELLING FOR EVERY ONE OF THESE ANNOTATION
     KINDS, AND IT IS NOT THE UNWIRED ONE.** Its own examples block (lines 670+) specifies
     `@precedence: {level: 5, associativity: "left"}` · `@constraint: {type: "requires", expression:
     "x > 0"}` · `@performance: {complexity: "O(n)", memory: "O(1)"}` · `@platform: ["web", "mobile",
     "desktop"]` · `@version: "2.1.0"`. Every one is a **structured or primitive** value — precisely
     what `annotation_value` already routes to.
  2. ⭐⭐ **AND ALL OF THEM PARSE TODAY.** Driven through `--interpret-parse` against the real
     grammar, all five documented spellings return `accepted=true`. There is no gap to close.
  3. ⛔⛔ **THE BESPOKE SPELLINGS THE UNWIRED RULES DEFINE ARE REJECTED TODAY** — `@precedence: 5
     left` → `accepted=false … did not consume full input at position 15`; `@version: 1.2.3` →
     `accepted=false … position 13`. (`@performance: O(n)` happens to be accepted, but through the
     generic route, not through `complexity_spec`, which is unreachable.)
  4. ⭐ **NOTHING CONSUMES THEM.** A search of `docs/contracts/` and `rust/src/` for all 22 returns
     **zero** references. (It returned two — `union_type`, `intersection_type` — and those turned
     out not to be class D at all; see the correction above.)
- ⛔⛔ **THEREFORE WIRING THEM IN WOULD NOT COMPLETE THE GRAMMAR — IT WOULD FORK IT.** It would add a
  SECOND accepted syntax for annotation kinds that already have a working, documented one, and with
  it a **second AST shape per kind** (`@precedence: 5 left` → a `precedence` node;
  `@precedence: {level: 5, …}` → a structured node). A consumer would then have to handle both for
  the same annotation. That is a widening of the accept set, bought with a downstream ambiguity,
  requested by nobody, and contradicted by the grammar's own documentation.
- ⛔ **THE CORRECTION THIS DECISION RESTS ON, STATED PLAINLY**: when I surfaced this question I wrote
  that `@precedence: 5 left` *"falls through to a generic `primitive_value`"*. **That was wrong and I
  had not measured it** — it is REJECTED. The mistake mattered: it framed the choice as
  *"structural parse vs. sloppy generic parse"*, which flatters wiring them in, when the real choice
  is *"one documented syntax vs. two competing ones"*, which does not.
- **DECIDED**: remove the 22 class-D rules and the class-C duplicate `annotation`. ⭐ **EFFECT ON
  CERTIFICATION**: `semantic_annotation` goes **29 → 6** UNKNOWN, and both survivors are already
  adjudicated — 2 LR-elimination artifacts and 4 trivia rules, the latter being the class whose
  principled home is a verified unreachability PROOF. The grammar becomes exactly what it documents.
- ⭐ **REVERSIBILITY, because a removal should say how to undo it**: the rules remain in git history
  in full. If a consumer ever wants `@precedence: 5 left`, reviving them is a revert plus the wiring
  — and at that point it would be a deliberate, requested surface addition with a contract bump,
  which is the process this repository already has for such a change.
- ⚠️ **PRIOR ART, per `DESIGN-PRIOR-ART`**: the search was the grammar's own documentation block and
  the published integration contract. Both were read before deciding; the documentation block is
  what settled it.
- ⏭️ **IMPLEMENTATION IS A SEPARATE SLICE AND IS NOT DONE HERE**: it is a grammar edit that
  regenerates the annotation parser PAIR the annotation backend links to generate **every other
  family**, so it needs a regeneration, an `emission_sha` rebaseline and
  `parse_harness_equivalence_gate` as the oracle that the shipped parse did not move.

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
