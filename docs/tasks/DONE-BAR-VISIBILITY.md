# DONE-BAR-VISIBILITY — make each parser's DISTANCE to `Done` visible in the book

## Metadata

- Tree ID: `DONE-BAR-VISIBILITY`
- Status: `active`
- Roadmap lane: documentation / closure reporting (cross-cutting, all 10 families)
- Created: 2026-08-30
- Origin: **DIRECTOR, 2026-08-30, verbatim**: *"You just flag parser statuses as Mostly Done, In
  progress, … I have no idea for each parser the amount of work to reach full completion (Done). I
  have no idea or clue about (1) the amount of test corpus that are passing (2) a synthesis about
  what's left to be done to reach the Done status. … we are iterating for countless of slices, lanes,
  task-trees and from my point of view there seems to be no end in sight."*

## Goal

⭐⭐ **ONE PRODUCER, TWO RENDERINGS — director, 2026-08-30, clarifying the remark above**: *"I need a
way to track the progress, issues left for a given parser … with brief, synthesized information in
the terminal and thorough in the mdbook(s)."*

- **CHAT — brief, synthesized, reported BY ME.** ⛔⛔ **CORRECTED 2026-08-30 by the director within
  the hour, and the first reading would have built the wrong thing**: *"Not me running in the terminal
  … but something you could display in this chat for me to read."* The director is NOT the operator of
  this surface — **I am**. The deliverable is a producer I run and whose synthesized output I paste
  into my completion messages, not a CLI the director is expected to invoke. A command nobody runs is
  the `GATE-REACHABILITY` defect wearing a status report.
  ⇒ it extends what `COMMIT.md` already obliges every completion message to carry (the live-status
  snapshot) from a bare LABEL to a DISTANCE.
- **BOOK — thorough.** The same producer emits the full section into that family's book page and the
  cross-family view into the main book.
- ⛔ **The two renderings MUST share one derivation.** A terminal summary that can disagree with the
  book is worse than neither, because the reader cannot tell which is lying. This is the
  `report_grammar_certification.sh` shape — one producer, a `--check` mode that re-derives and DIFFS
  against what is published, wired as a gate.

Both renderings answer three questions **from derived data, not prose**:

1. **Where is it against the bar** — which legs of the `Done` bar are met, which are not.
2. **How much corpus passes** — the pass/fail/unadjudicated counts against a named denominator.
3. **What remains** — the ordered, leaf-owned list of work between here and `Done`, with the
   owning leaf id for each item, so "no end in sight" becomes a finite list a reader can count.

## Non-goals

- Not a new status vocabulary. The `Done` / `Provisional` / `Mostly Done` / `In Progress` set stays
  (`quality-and-closure-model.md`); this tree makes the DISTANCE behind each label visible.
- Not hand-written progress prose. A hand-maintained percentage rots — that is `LIVE-MEANS-LIVE.1a`'s
  measured finding (25 of 61 hand-maintained fields were simply wrong). Every number must be DERIVED
  and gate-held, on the `SV-CORPUS-DENOMINATOR` / `PARSE-COST-RATCHET` co-publication pattern.
- Not a re-litigation of any family's status.

## The measured gap this tree exists to close (2026-08-30)

- `done_bar_family_register_v0.json` carries per family exactly: `claimed_status`, `corpus_roots`,
  `gate_prefixes`, `language_owner`, `leg3_surface`, `standard`. **No count, no pass rate, no
  remaining-work list.**
- `docs/book/src/roadmap-and-live-status.md` renders that as, in full, for the flagship family:
  `| systemverilog | Mostly Done | external-standard | (none declared) |`.
- A grep for a distance-to-`Done` view over the main book and all ten parser books returns **no such
  surface**.
- **ONE family has published corpus evidence at all** — SystemVerilog, via `SV-CORPUS-DENOMINATOR`:
  tuple `7556/2606/6174/4366/274` (adjudicated / routed / no-verdict / dark / bar). The other **nine
  families publish no corpus pass count anywhere a reader can reach.**
- ⇒ the director's statement is not an overstatement. For 9 of 10 families the book cannot answer
  *"how far?"*, and for the 10th it answers only through a doctrine gate's stderr, not the book.

## Acceptance criteria (tree)

- [ ] Each of the 10 families' book pages carries a DERIVED status block: bar-leg state, corpus
      counts against a named denominator, and the remaining-work list with owning leaf ids.
- [ ] Every number in that block is derived by a producer and gate-held equal to its source, so it
      cannot drift (the `SV-CORPUS-DENOMINATOR` co-publication shape).
- [ ] A family with no corpus says so LOUDLY as `NOT MEASURED`, never as a blank or a zero —
      `docs/decisions/feedback_a_check_that_cannot_run_must_say_so.md`.
- [ ] The main book gains one cross-family view so the whole picture is one page, not ten.
- [ ] **A producer exists that emits the brief form for one family in a screenful, and I report it
      in completion messages** — the director reads it in chat and is never expected to run it.
- [ ] **Terminal and book cannot disagree**: `--check` re-derives and diffs the published block,
      wired as a gate, so a stale book section fails a commit rather than misleading a reader.

## Task tree

- ID: `DONE-BAR-VISIBILITY.1` — Status: `done` (`PGEN-DONE-BAR-VISIBILITY-0003`)
  Goal: DEFINE the per-family status block AND ship the terminal renderer: exactly which fields,
  which derivation for each, and which are legitimately `NOT MEASURED` today. Deliverable is a schema
  + the producer + one worked family, not ten.
  Acceptance: the schema names a producer for EVERY field; the brief form for SystemVerilog fits a
  screenful and is REPORTED IN CHAT by me; SystemVerilog rendered end to end in both surfaces from
  the one derivation.
- ID: `DONE-BAR-VISIBILITY.2` — Status: `done` (`PGEN-DONE-BAR-VISIBILITY-0004`)
  Goal: the REMAINING-WORK synthesis — derive each family's open items from the task trees that
  already own them, so the list cannot drift from the trees.
  Acceptance: every item carries its owning leaf id; the list is derived, not typed.
  ⚠️ **Acceptance MET at TREE granularity, not LEAF, and the difference is measured rather than
  waived**: every item carries its owning TREE id and the list is derived from a closed two-sided
  attribution over the index. A per-LEAF list is blocked on the leaf-status/leaf-id convention
  defect — leaf status is free prose whose ONLY reader in the whole tree is one hard-coded leaf in
  `audit_done_bar.sh`, censused in `.2`'s leaf record below
  (`docs/knowledge/a-status-that-is-prose-is-a-status-nothing-checks.md`) — and THREE leaf-definition
  conventions are in live use (`DOCTRINE-GAP-OWNERSHIP.8`) — so a per-leaf list today would publish
  those defects as a number. Owned by new leaf `.7`, not waived.
- ID: `DONE-BAR-VISIBILITY.3` — Status: `done` (`PGEN-DONE-BAR-VISIBILITY-0005`)
  Goal: the corpus pass counts for the nine families that publish none, or an explicit
  `NOT MEASURED` with the leaf that owes it.
  Acceptance: MET on the second arm of its own goal — each of the nine now says WHY it has no count,
  in one of five DERIVED classes, and names the leaf that owes one. The counts themselves are `.3b`.
- ID: `DONE-BAR-VISIBILITY.3b` — Status: `pending`
  Goal: turn the two `corpus_wired_no_census` families into real counts. Both have been measured
  once and neither can be re-derived from the tree: `vhdl` read 13 720 files / pass 4 031 / fail
  9 689 on 2026-08-08 (`CORPUS-GRAD-ALL.2.1`) but `results.tsv` is gitignored, and `regex` has a
  PCRE2 oracle tuple whose per-run summary is untracked by that doctrine's own admission.
  Acceptance: a TRACKED census artifact per family plus a freshness gate, on the
  `SV-CORPUS-DENOMINATOR` shape, so the counts are re-derivable rather than remembered. ⛔ The VHDL
  number is also a RAW pass rate — no answer key says which of those files SHOULD parse — so the
  block must not publish it as an adjudicated one.
- ID: `DONE-BAR-VISIBILITY.9` — Status: `done` (`PGEN-DONE-BAR-VISIBILITY-0006`)
  Goal: publish the derived block into the OTHER NINE parser books, as `.1` did for SystemVerilog.
  The producer already renders every family; what is missing is nine pages, nine SUMMARY entries and
  nine `--check` invocations.
  Acceptance: all ten per-parser books carry the block; each is `--check`-verified; the mdbook gate
  passes. ⚠️ `--check`-HELD BY A GATE is `.4`, which is now the frontier — ten pages that a human
  must remember to re-check are ten pages that will go stale.
- ID: `DONE-BAR-VISIBILITY.4` — Status: `done` (`PGEN-DONE-BAR-VISIBILITY-0007`)
  Goal: wire the co-publication gate so the block cannot go stale.
  Acceptance: `FAMILY-STATUS-CURRENCY` is a registered doctrine running on every commit; it refuses
  before it trusts; its RED arms are observed firing.
- ID: `DONE-BAR-VISIBILITY.5` — Status: `pending`
  Goal: the cross-family page in the main book.
- ID: `DONE-BAR-VISIBILITY.7` — Status: `pending`
  Goal: raise the remaining-work list from TREE granularity to LEAF granularity — the shape `.2`'s
  acceptance asked for and the shape a reader wants ("what are the four things left", not "which
  four trees are open").
  Blocked by: leaf status is free prose whose only reader, censused in `.2`'s leaf record, is one
  hard-coded leaf in `audit_done_bar.sh` — so it is unchecked as a CLASS — and three
  leaf-definition conventions are in live use (`DOCTRINE-GAP-OWNERSHIP.8`). Either that lands first,
  or this leaf pays for a machine-readable leaf-status convention itself.
  Acceptance: each open item is a LEAF id with its goal line, derived; no per-leaf status is typed
  into the block.
- ID: `DONE-BAR-VISIBILITY.8` — Status: `pending`
  Goal: reconcile the tracker-status disagreements `.2` measured. Its two prose arms — the index
  row's Status cell and the tree file's own `Metadata` Status — DISAGREE on **4 of 113** trees
  (`VERILOG-2005-PROFILE`, `EBNF-BOOK`, `RGX-0090`, `RGX-0091`), and **7** index rows carry no
  recognisable status word at all (`ENGINE-UNIVERSAL-SERVICES`, `GRAMMAR-WELLFORMED`,
  `LIVE-DOC-CONTAINMENT`, `SV-CORPUS-GRAD`, `MCP-CONTROL`, `JSON-RFC8259`,
  `REJECTS-VALID-DEFENSE`). `.2` counts every one of them OPEN — the safe polarity — so the
  published remaining-work count is an UPPER BOUND today, and saying so is not the same as fixing
  it.
  Acceptance: each of the 11 is adjudicated to one true status in both places; then rule on whether
  a doctrine should hold the two arms equal, or whether one arm should stop being a source.
- ID: `DONE-BAR-VISIBILITY.6` — Status: `pending`
  Goal: the HAND-WRITTEN STATUS PROSE class, found by `.1` and routed rather than logged. The
  SystemVerilog book's `welcome.md` carried three figures and all three were wrong — the AST-dump
  schema published as `10` in one paragraph and `13` in another while the contract carried `26`,
  and the parser described as *"validated each release by the … external-corpus (14/14) … gates"*
  while `audit_done_bar.sh` records the only gate reading that corpus as a TRIAGE gate, leaving
  leg 3 unmet. `.1` fixed that ONE page by replacing the prose with a pointer to the derived
  surface. The CLASS is untouched: nine other parser books may carry the same thing, and
  `PARSER-BOOK-CURRENCY` cannot see it — it holds a book's newest CHANGELOG release equal to the
  contract and says nothing about a status paragraph.
  Acceptance: every per-parser book's status prose is censused; each figure is either DERIVED by a
  producer or DELETED in favour of a pointer; no per-parser book carries a hand-typed maturity
  number afterwards.

## Current frontier

1. `DONE-BAR-VISIBILITY.5` (the cross-family view in the main book — the whole picture on one page
   rather than ten). It is the frontier because the per-family answer is now published AND gated,
   and the one thing a reader still cannot do is compare families without opening ten books.
2. `DONE-BAR-VISIBILITY.3b` (real counts for `vhdl` and `regex` — a tracked census plus a freshness
   gate, the `SV-CORPUS-DENOMINATOR` shape).
3. `DONE-BAR-VISIBILITY.8` (the 4 DISPUTED + 7 unreadable tracker statuses, which make every
   remaining-work count an upper bound).

## Leaf records

### `.1` — the per-family status block: schema, producer, SystemVerilog end to end

Shipped in `PGEN-DONE-BAR-VISIBILITY-0003`.

**What landed.**

- `rust/test_data/grammar_quality/family_status_block_v0.json` — the SCHEMA. Every field carries
  the question it answers and the PRODUCER that answers it; a field with `state: not_measured`
  carries `owed_by`, the leaf that owes the producer. It also holds the per-family corpus-census
  wiring, so `.3` adds a family by adding data rather than by editing the renderer.
- `scripts/report_family_status.sh` — the PRODUCER. `<family>` renders the brief form (a
  screenful, reported in chat by me); `--markdown` renders the book block; `--check FILE`
  re-derives and diffs the published block, exiting 1 on drift and 2 when it cannot evaluate.
- `docs/systemverilog_parser_book/src/status-and-distance.md` — the published SV block, plus the
  SUMMARY entry and a pointer from the main book's live-status chapter.
- `docs/tasks/artifacts/done_bar_visibility/run_family_status_probes.sh` — 8 arms, 1 green control
  and 7 RED, capture in `family_status_probes.txt`.

**Two design rulings worth carrying.**

1. **The book half is derived TRACKED-ONLY; the chat half adds this machine's readings.** Leg 1 and
   the green-NOW half of leg 2 rest on gate artifacts under `rust/target/`, which is untracked — the
   same commit reads `artifact STALE` where the gate ran and `no artifact on this machine` on a
   fresh clone. Publishing either into a gate-held block would make `--check` fail for every reader
   who had not run the gate first. The producer therefore drives the audit through its own
   `PGEN_DONE_BAR_TARGET_DIR` seam pointed at an empty directory — no fork of the audit, no second
   copy of its logic — and the chat form adds the ordinary run's verdicts on top.
2. **A leg is never published without a reason, and an unrecognised reason REFUSES.** The audit's
   sentences are a diagnostic surface free to improve; the block maps them onto a closed vocabulary
   and refuses when a leg's lines match nothing, so a wording change is LOUD here instead of
   silently emptying the published block.

#### Acceptance checklist (`TOOLBOX.md`)

- [x] **ROOT CAUSE (WHY + WHERE)** — measured against HEAD blobs, not the working tree, because the
      first run of this census read the working tree and reported **3 hits** for a surface that
      does not exist at HEAD (my own edits), which is the drift the check exists to catch:
      `for f in $(git ls-files 'docs/*book*/src/*.md'); do git show "HEAD:$f" | grep -qiE 'distance
      to \`?done' && echo HIT; done` returns **0 hits over 177 tracked book pages** — WHERE: no
      page in the main book or any of the ten parser books answers *how far is this family from
      `Done`*. WHY: the done-bar register holds `claimed_status` and no count, the audit that DOES
      derive the legs is an operator command whose output is published nowhere, and the corpus
      census is reachable only through a doctrine gate's stderr. The status LABEL is published and
      the DISTANCE behind it is not, so the director reads a ladder position with no scale.
      ⛔ The same census found the SECOND defect, routed to `.6`:
      `git show HEAD:docs/systemverilog_parser_book/src/welcome.md | grep -nE 'version .1[03].|14/14'`
      returns lines 11 and 31 publishing AST-dump schema `13` and `10`, while
      `git show HEAD:docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` line 827
      carries `26` — two hand-written numbers disagreeing with the contract AND with each other in
      one file, beside a claim of `external-corpus (14/14)` validation that the done-bar audit
      contradicts (the only corpus gate is a TRIAGE gate ⇒ leg 3 UNMET).
- [x] **ADDRESSED (verified)** — before: `bash scripts/report_family_status.sh systemverilog` did
      not exist and no book page carried a distance. After: it prints the SV block in **3.0 s** —
      3 of 3 legs not met with the reason for each, 16 covering gate targets of which 1 is invoked
      by nothing, and 5 845 of 16 336 corpus rows passing (35.8 %) against a named denominator with
      274 known defects, 6 174 rows carrying no verdict and 4 366 of those dark. The same
      derivation is published at `docs/systemverilog_parser_book/src/status-and-distance.md` and
      `--check` on that page returns `OK (published block equals a fresh derivation, 2344 bytes)`.
      All ten families render without refusing. The stale prose in `welcome.md` is replaced by a
      pointer to the derived page rather than by a fresher hand-typed number.
- [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh`: **31/31 PASS** (`BASELINE-IDENTITY`
      correctly refused the new schema file until it was registered with a
      `not-a-derived-baseline` verdict — the closed-population arm doing its job, then green).
      `make -C rust SHELL=/bin/bash mdbook_docs_gate` PASS — all 10 per-parser book gates plus the
      main book. `bash scripts/audit_done_bar.sh` exit 0 with its 13 ground-truth controls
      reproduced and its text report unchanged (this slice adds no code to it; it is driven only
      through its existing env seam). The published block is **byte-identical** to a fresh
      derivation by construction — that is what `--check` asserts, and RED-1 proves it fails when
      one number is mutated.
- [x] **RED CONTROLS** — `bash docs/tasks/artifacts/done_bar_visibility/run_family_status_probes.sh`
      → **8 passed, 0 failed**. The four arms that were mutually indistinguishable in
      `GRAMMAR-CERT-STATUS.1b` are separated here: a mutated pass count FAILS (1), a page with no
      DERIVED block REFUSES (2), a nonexistent path REFUSES (2), another family's block on the page
      FAILS (1). Plus an unknown family and a missing family name (2 each), and the two-sided
      schema arm: a field declared in the schema that nothing renders REFUSES (2).

### `.2` — the remaining-work synthesis: a finite list, derived from the trees that own it

Shipped in `PGEN-DONE-BAR-VISIBILITY-0004`.

**What landed.** `tree_attribution` in `family_status_block_v0.json` — a CLOSED, TWO-SIDED map from
every row of `docs/TASK_TREE.md` to a family, `cross-cutting`, `process`, `future-family` or
`platform`; 113 rows, adjudicated one by one. The producer derives each family's OPEN trees from it
and renders them in both surfaces. `remaining_work` moves from `not_measured` to `derived`.

**The answer, for the first time as a number a reader can count:** SystemVerilog has **5** open
trees of its own and **26** cross-cutting trees that can gate any family; `systemverilog_preprocessor`,
`return_annotation`, `rtl_const_expr`, `ebnf`, `json` and `semantic_annotation` have **1** each;
`regex` has **5**; `vhdl` and `rtl_frontend` have **0** — which is not the same as meeting the bar,
and the block says so.

**Three rulings worth carrying.**

1. **The item is a TREE, not a leaf**, because leaf status is free prose that no instrument derives
   as a class, and three leaf-definition conventions are in live use. A per-leaf list would publish
   those defects as a number. Routed to `.7` rather than waived.
   ⛔ **The census, run against the population that would REFUTE it rather than confirm it — and it
   DID, so the claim is narrowed rather than published as written:**

   ```
   git grep -n -- 'todo' -- 'scripts/*.sh' 'rust/scripts/*.sh' 'rust/scripts/lib/*.sh' \
       '.githooks/*' '.github/workflows/*' | grep -v check_doctrines.sh
     scripts/audit_done_bar.sh:701:  leaf5_open = re.search(r"^### `\.5`.*\(`todo`\)", read(TREE))
     (plus this slice's own producer)
   git grep -ln "Status:" -- 'scripts/*.sh' 'rust/scripts/*.sh' '.githooks/*' '.github/workflows/*'
     -> 0 files
   git grep -ln "docs/tasks" -- 'scripts/*.sh' 'rust/scripts/*.sh' '.githooks/*' '.github/workflows/*'
     -> 34 files, none of which reads a leaf Status line
   ```

   ⇒ **exactly ONE instrument reads a leaf status, and only for ONE hard-coded leaf** (`DONE-BAR.5`,
   so `audit_done_bar.sh` can say the highest attainable tier is `Provisional`). Leaf status is
   unchecked as a CLASS; *"nothing derives it"* would have been false, and the first draft of this
   very leaf record said exactly that. 34 instruments read `docs/tasks/` and not one of them reads a
   leaf's status.
2. **The open/closed verdict is TWO-ARMED and biased OPEN.** Both arms are prose — the index row's
   Status cell and the tree file's own `Metadata` Status. They disagree on 4 of 113 trees, so a list
   built on either arm alone is wrong four times; 7 more rows carry no status word at all. Every
   disagreement and every unreadable row is counted OPEN and NAMED. Over-reporting remaining work is
   the only safe direction for a document whose purpose is to say how much is left. Routed to `.8`.
3. **The tree-file link is searched over the WHOLE ROW, not a fixed cell index.** A row whose cell
   count disagrees with its header — 14 of 67 rows of this very index once did, the four widest
   being the four most active trees — puts the File cell elsewhere, and a fixed index reports
   exactly the most active trees as having no tree file. Caught by reading the first rendered
   output: `SV-CORPUS-GRAD` printed *(no tree file linked)*.

#### Acceptance checklist (`TOOLBOX.md`)

- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: nothing joins a task tree to a parser family, so the
      question *how much is left for THIS parser* has no producer. WHY, measured — and the first two
      censuses I ran were both WRONG, which is the finding, not an aside:
      `git ls-files docs/tasks/` with the pathspec `docs/tasks/*.md` returned **337** files because
      a git pathspec `*` crosses `/`, sweeping in `docs/tasks/artifacts/`; the real top-level
      population is **164**, of which **113** are index rows. Then a heading census for the required
      `Current frontier` section returned **3 of 115** — a case-sensitive regex against a corpus
      that writes `## Current Frontier`; the honest number is **70 heading-form, 5 prose-only, 40
      absent**, so the section `docs/TASK_TREE.md` calls REQUIRED is missing from 40 tree files.
      ⇒ the index row, not the tree file, is the maintained surface, and `COMMIT.md` obliges keeping
      it current. That is the third time this repository has paid for a heading census under a
      heading grammar that did not fit the corpus
      (`docs/knowledge/a-heading-census-is-only-as-good-as-the-heading-grammar.md`).
- [x] **ADDRESSED (verified)** — before: `remaining work ......... NOT MEASURED — owed by
      DONE-BAR-VISIBILITY.2`. After: `5 open tree(s) own this family's work: SV-CORPUS-GRAD,
      SV-EXH-PROOF, SV-PARSE-STRICT, SV-REPLAY-DEBT, VERILOG-2005-PROFILE / + 26 cross-cutting
      tree(s) / ⚠️ 1 status DISPUTED, 1 unreadable — all counted OPEN`, and the book block carries
      the same five with a link to each tree file. All ten families render a finite list.
- [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh` ALL 31 PASS;
      `make -C rust SHELL=/bin/bash mdbook_docs_gate` PASS; probes **10/10**; the republished block
      is **byte-identical** to a fresh derivation (`--check` OK, 3 369 bytes); `audit_done_bar.sh`
      and the corpus census are untouched by this slice and still drive the same numbers.
- [x] **RED CONTROLS** — two new arms prove the closed population BITES: RED-8 drops one tree from
      the attribution and the producer REFUSES naming it; RED-9 adds an attribution for a tree that
      does not exist and it REFUSES naming that. Both through the `PGEN_FAMILY_STATUS_SCHEMA` seam,
      so the tracked schema is never mutated to test itself.

### `.3` — why every family without a corpus count has no count, derived and owned

Shipped in `PGEN-DONE-BAR-VISIBILITY-0005`.

**What landed.** A `corpus_state` section in `family_status_block_v0.json`: five classes, one row per
family, each naming the leaf that owes a count. The producer **derives** the class from the done-bar
register — `language_owner` x whether `corpus_roots` is empty x whether a census is wired — and holds
it EQUAL to the declared class, refusing on drift. Both renderings print the class, the reason and
the owner.

**The nine, after this slice.**

| Family | Class | Owed by |
|---|---|---|
| `vhdl`, `regex` | `corpus_wired_no_census` — roots declared, no tracked census | `DONE-BAR-VISIBILITY.3b` |
| `systemverilog_preprocessor` | `no_corpus_wired` — standardized language, nothing wired | `CORPUS-GRAD-ALL.1` |
| `rtl_frontend`, `rtl_const_expr`, `json` | `ownership_unadjudicated` — a subset language whose corpus question is an open ruling | `DONE-BAR.3` |
| `ebnf`, `return_annotation`, `semantic_annotation` | `no_third_party_corpus` — PGEN authors the language | — (not a measurement gap) |

**Two rulings.**

1. **The class is DERIVED and only CHECKED against the declaration.** A declared-only class becomes a
   lie the moment a corpus is wired, which is the same rot as a hand-maintained status. RED-10 drives
   it: declaring `vhdl` as `no_third_party_corpus` while the register carries a corpus root REFUSES.
2. **A number that was measured once but cannot be re-derived is POINTED AT, never copied.** `vhdl`
   read 13 720 files / pass 4 031 / fail 9 689 on 2026-08-08, and `results.tsv` is gitignored
   (the rule `stimuli/*/characterization/results.tsv`), so the block names `CORPUS-GRAD-ALL.2.1` instead of reprinting the number —
   and says it is a RAW pass rate with no answer key behind it, which is a different claim from
   SystemVerilog's adjudicated one.

#### Acceptance checklist (`TOOLBOX.md`)

- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: the corpus field rendered one generic sentence for nine
      different situations. WHY, measured — `git ls-files` over the census surface and over the raw
      result set the last VHDL measurement produced:

      ```
      git ls-files | grep 'results.tsv'          -> json_corpus_bundle/results/json_corpus_results.tsv   (ONE, and not VHDL's)
      git check-ignore -v stimuli/vhdl/characterization/results.tsv
        -> .gitignore:424:stimuli/*/characterization/results.tsv
      ls -la stimuli/vhdl/characterization/      -> results.tsv, 1 094 165 bytes, present but UNTRACKED
      git ls-files docs/tasks/artifacts/corpus_grad_all/
        -> vhdl_fail_clusters.{md,tsv} (the FAIL side only — no denominator, no pass count)
      ```

      ⇒ the VHDL number exists on this machine and in no clone. ⚠️ The owning tree cites this rule as
      `.gitignore:412`; it is line **424** today — a carried line number that drifted, corrected here
      by citing the RULE instead. The register's `language_owner` × `corpus_roots` fields then split
      the nine into FOUR genuinely different cases — two with a corpus
      root and no census, one standardized with nothing wired, three whose language ownership is an
      open ruling, and three where no third-party corpus can exist at all. Printing `NOT MEASURED`
      over all of them told a reader nothing about which leaf to push, and told the `pgen`-owned
      three that somebody had forgotten to measure something that does not exist.
- [x] **ADDRESSED (verified)** — before: `corpus NOT MEASURED — no census is wired; owed by
      DONE-BAR-VISIBILITY.3` for all nine. After, per family:
      `vhdl` / `regex` → `NOT MEASURED [corpus_wired_no_census] — owed by DONE-BAR-VISIBILITY.3b;
      last measured in CORPUS-GRAD-ALL.2.1 / RGX-0078`; `systemverilog_preprocessor` →
      `[no_corpus_wired] — owed by CORPUS-GRAD-ALL.1`; `json` / `rtl_frontend` / `rtl_const_expr` →
      `[ownership_unadjudicated] — owed by DONE-BAR.3`; `ebnf` / `return_annotation` /
      `semantic_annotation` → `NO THIRD-PARTY CORPUS EXISTS [no_third_party_corpus]`. All ten
      families render; SystemVerilog's block is unchanged (`--check` still green).
- [x] **NO REGRESSION** — probes **12/12**; `bash scripts/check_doctrines.sh` ALL 31 PASS;
      `make -C rust SHELL=/bin/bash mdbook_docs_gate` PASS; the published SV block is
      **byte-identical** to a fresh derivation, which is the point: this slice changes what the nine
      say and must not move the one that already had an answer.
- [x] **RED CONTROLS** — RED-10 (a declared class drifting from the register REFUSES) and RED-11 (a
      family with no declared class REFUSES), both through the `PGEN_FAMILY_STATUS_SCHEMA` seam.

- promotion: declined (both durable halves are already carried by the tree — *derive the class and only CHECK the declaration* is the two-arm rule `done_bar_family_register_v0.json` already states for `claimed_status`, and *a number that cannot be re-derived is pointed at, not copied* is `docs/DERIVED_STATE_CONTAINMENT.md` R1/R3; the per-family class map is a one-artifact fact and lives in the schema it drives)

### `.9` — the block is published in all ten parser books

Shipped in `PGEN-DONE-BAR-VISIBILITY-0006`.

Nine `status-and-distance.md` pages, nine `SUMMARY.md` entries, nine rebuilt (tracked) HTML trees.
Every page carries the same three sections — the three-leg bar explained, the DERIVED block, and how
the page stays true — and the corpus paragraph adapts: a family with a wired census gets the
*count every row, unknown is not clean* note, and the other nine get the note explaining that the
line says WHY there is no count and who owes one.

`--check` verified on all ten, byte for byte: `systemverilog` 3 409 B, `regex` 3 134 B, `vhdl`
2 537 B, `json` 2 205 B, `systemverilog_preprocessor` 2 111 B, `ebnf` 2 123 B, `rtl_frontend`
2 102 B, `return_annotation` 2 088 B, `semantic_annotation` 2 028 B, `rtl_const_expr` 2 000 B.

⛔ **What this leaf deliberately does NOT do is make those ten checks fire.** They are invoked by a
human today, which is the `GATE-REACHABILITY` defect the tree's own goal warns about, and `.4` owns
closing it. Publishing first and gating second is the right order here — `SV-CORPUS-GRAD.13e.11`'s
rule that a guard is safe to make blocking exactly when the fix precedes it — but only if `.4`
actually lands, so it is now the frontier.

#### Acceptance checklist (`TOOLBOX.md`)

- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: nine of the ten per-parser books had no status surface at
      all. WHY: `.1` deliberately shipped ONE worked family, so the producer could render all ten
      while only one book published the answer. Measured before this slice:
      `git ls-files 'docs/*_parser_book/src/status-and-distance.md' | wc -l` → **1**, against
      `git ls-files 'docs/*_parser_book/book.toml' | wc -l` → **10**.
- [x] **ADDRESSED (verified)** — after: the same two `git ls-files` counts read **10** and **10**,
      and `bash scripts/report_family_status.sh <family> --check docs/<family>_parser_book/src/status-and-distance.md`
      returns OK for every one of the ten, with the byte counts listed above.
- [x] **NO REGRESSION** — `make -C rust SHELL=/bin/bash mdbook_docs_gate` PASS (all ten per-parser
      book gates plus the main book, every one rebuilding its tracked HTML);
      `bash scripts/check_doctrines.sh` ALL 31 PASS; probes **12/12**; the SystemVerilog block is
      **byte-identical** to its previous publication — this slice adds nine pages and changes none
      of the numbers.
- promotion: declined (this slice is nine copies of a page shape `.1` already established and whose lessons are already promoted; the durable rule it exercises — publish first, gate second, because a guard is safe to make blocking exactly when the fix precedes it — is `SV-CORPUS-GRAD.13e.11`'s and is already carried by the tree)

### `.4` — the gate: ten published blocks are now re-derived on every commit

Shipped in `PGEN-DONE-BAR-VISIBILITY-0007`. New doctrine **`FAMILY-STATUS-CURRENCY`**
(`scripts/check_family_status.sh`), registered in `scripts/check_doctrines.sh`, mirrored in
`DOCTRINE_ENFORCEMENT.md` §10, and the published doctrine count moved **31 → 32** in the book.

**Three tiers, each blind to what the next one sees.**

- **A — structural, no derivation (~0.05 s).** The population is CLOSED and TWO-SIDED: every
  registered family has a published page, every published page belongs to a registered family, each
  page carries exactly ONE well-formed `DERIVED` block, and that block's `BEGIN` marker names the
  family whose book it sits in — so another family's block pasted into a page is caught statically,
  before a single second of derivation is spent.
- **B — the RED control (~0.01 s), driven BEFORE the oracle is believed.** The producer's `--check`
  on a path that does not exist must exit 2. `GRAMMAR-CERT-STATUS.1b` shipped a producer whose
  `--check` FLAG survived a rewrite that deleted its implementation, and four different inputs all
  exited 0 for a whole commit; a green verdict from an instrument that says green to everything is
  worth nothing.
- **C — the oracle (~3.4 s).** `--check-all` re-derives all ten blocks and DIFFS them.

**The cost decision is what makes it a commit-time gate.** The producer's two `audit_done_bar.sh`
runs are family-INDEPENDENT and are ~2.3 s of a 2.4 s single-family run, so ten separate invocations
would pay for them ten times — 24 s, which is how a check ends up behind a flag nobody sets. The new
`--check-all` mode hoists them: **3.4 s for all ten**.

⚠️ **Honest bound, stated rather than hidden**: tier C's RED is proven at the PRODUCER level
(`run_family_status_probes.sh` RED-1 mutates a published pass count and RED-4 swaps in another
family's block, both FAILING at exit 1), not at the gate level, because the gate's tier C addresses
the ten tracked pages by name and driving it red would mean mutating a tracked file and hoping the
restore runs. Tier A's states ARE constructed, through a `PGEN_FAMILY_STATUS_PAGES_ROOT` seam, so no
tracked page is ever touched.

#### Acceptance checklist (`TOOLBOX.md`)

- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: after `.9`, ten book pages published derived numbers and
      the only thing re-checking them was a human remembering to run `--check`. WHY that is a defect
      and not a preference, measured on this tree's own surface:
      `git ls-files 'scripts/check_*.sh' | xargs grep -l report_family_status | wc -l` → **0**, and
      `git grep -n report_family_status -- '.githooks' '.github/workflows' 'rust/Makefile' | wc -l`
      → **0**. Nothing invoked the check at all, which is `GATE-REACHABILITY`'s founding
      measurement — a gate nothing invokes is indistinguishable from a gate that does not exist —
      arriving on the very surface this tree built to end hand-maintained status.
- [x] **ADDRESSED (verified)** — after: `bash scripts/check_family_status.sh` runs all three tiers in
      **3.4 s** and `bash scripts/check_doctrines.sh` reports **ALL 32 enforced doctrines PASS**,
      with `<meta:mirror>` confirming §10 lists exactly the 32 and `<meta:book-count>` confirming the
      book publishes 32 at both marked sites. Re-run after staging, the first census reads **1**
      (`scripts/check_family_status.sh`), and the invocation the second census was looking for is
      `git grep -c check_family_status -- scripts/check_doctrines.sh` → **1**, i.e. the registry row
      that makes `.githooks/pre-commit` and CI run it without either of them naming it.
- [x] **NO REGRESSION** — `check_doctrines.sh` 32/32 (31 pre-existing, all still green);
      `make -C rust SHELL=/bin/bash mdbook_docs_gate` PASS; producer probes **12/12**; the ten
      published blocks are **byte-identical** to a fresh derivation, which the new gate is precisely
      what asserts.
- [x] **RED CONTROLS** — `bash docs/tasks/artifacts/done_bar_visibility/run_family_status_gate_probes.sh`
      → **7 passed, 0 failed**: two GREEN controls (the real tree; an unmutated mirror) and five RED
      arms — a registered family with no page, a page for no registered family, another family's
      block in this family's book, a page with the block duplicated, and a page with the markers
      stripped. Every RED state is CONSTRUCTED under the scratch pages-root seam; no tracked page is
      mutated.
- promotion: declined (the durable rules this leaf exercises are already carried — a check nothing invokes is GATE-REACHABILITY's founding measurement, refusing before trusting is docs/knowledge/a-control-that-cannot-fail-is-not-a-control.md, and hoisting a family-independent cost so a gate is cheap enough to run every commit is an engineering decision recorded in the leaf and in the doctrine's own header)

## Decisions

- **Derived, never hand-written** — `LIVE-MEANS-LIVE.1a` measured 25 of 61 hand-maintained
  "last updated" fields simply wrong; a hand-maintained percentage is the same class.
- **SystemVerilog first**, because it is the release lane and the only family with corpus evidence
  to render — so `.1` is inside the SV lane lock rather than an escape from it.

## Open questions

- Whether `Provisional (corpus pending)` families should show a corpus block at all before a corpus
  exists, or only the leaf that owes one. Leaning: show the leaf — an empty block reads as zero.

## Blockers

None.

## Verification log

- 2026-08-30 — the gap above measured, not asserted: register schema read, book row quoted verbatim,
  distance-view grep returned nothing, SV denominator tuple read from its own gate.
- 2026-08-30 (`.1`) — the census RE-RUN against HEAD blobs after the working-tree run reported 3
  false hits: **0 of 177** tracked book pages carry a distance-to-`Done` surface. Producer, schema,
  page, SUMMARY and probe driver landed; `--check` green on the published page; 8/8 probe arms;
  31/31 doctrines; mdbook gate PASS; all ten families render.
- 2026-08-30 (`.4`) — `FAMILY-STATUS-CURRENCY` registered; 32/32 doctrines; gate probes 7/7 with
  five RED arms constructed under a seam; 3.4 s for all ten pages.
- 2026-08-30 (`.9`) — nine pages, nine SUMMARY entries, nine rebuilt HTML trees; `--check` green on
  all TEN books; mdbook gate PASS; 31/31 doctrines.
- 2026-08-30 (`.3`) — the nine no-count families split into FOUR derived classes, each naming its
  owing leaf; the class is held equal to the done-bar register and RED-10 proves the drift refuses.
  12/12 probes, 31/31 doctrines, mdbook gate PASS, SV block byte-identical.
- 2026-08-30 (`.2`) — 113 index rows adjudicated, population closed in both directions and PROVEN
  to refuse in both (RED-8/RED-9). Two-arm status check fires: 4 DISPUTED, 7 unreadable, all counted
  OPEN. 10/10 probes, 31/31 doctrines, mdbook gate PASS, `--check` byte-identical at 3 369 bytes.

## Commit log

- tree created — `PGEN-DONE-BAR-VISIBILITY-0001`
- `.1` scope corrected by the director — `PGEN-DONE-BAR-VISIBILITY-0002`
- `.1` DONE (schema + producer + SystemVerilog end to end) — `PGEN-DONE-BAR-VISIBILITY-0003`
- `.2` DONE (the remaining-work synthesis, tree granularity) — `PGEN-DONE-BAR-VISIBILITY-0004`
- `.3` DONE (why each family has no corpus count, derived and owned) — `PGEN-DONE-BAR-VISIBILITY-0005`
- `.9` DONE (the block published in all ten parser books) — `PGEN-DONE-BAR-VISIBILITY-0006`
- `.4` DONE (the FAMILY-STATUS-CURRENCY doctrine, 31 -> 32) — `PGEN-DONE-BAR-VISIBILITY-0007`

## Changelog

- 2026-08-30 — created from a direct director remark; gap measured the same day.
- 2026-08-30 — `.1` DONE. One producer, two renderings, `--check` proven able to go RED four ways.
  New leaf `.6` opened for the hand-written-status-prose class the slice measured on the way past.
- 2026-08-30 — `.4` DONE: the ten published blocks are re-derived and diffed on every commit.
  Frontier moves to `.5`, the cross-family page.
- 2026-08-30 — `.9` DONE: every parser book now publishes its family's distance. Frontier moves to
  `.4`, the co-publication gate, because ten unchecked pages will go stale.
- 2026-08-30 — `.3` DONE: every family says WHY it has no corpus count and who owes one. New
  leaves `.3b` (the real VHDL/regex censuses) and `.9` (publish the block into the other nine books).
- 2026-08-30 — `.2` DONE at TREE granularity. 113 index rows adjudicated into a closed two-sided
  attribution; every family now publishes a finite open-tree list. New leaves `.7` (per-leaf
  granularity, blocked on the leaf-convention defect) and `.8` (the 4 DISPUTED + 7 unreadable
  tracker statuses the two-arm check found).
