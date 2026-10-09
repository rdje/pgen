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

- [x] Each of the 10 families' book pages carries a DERIVED status block: bar-leg state, corpus
      counts against a named denominator, and the remaining-work list with owning leaf ids. (`.9`)
- [x] Every number in that block is derived by a producer and gate-held equal to its source, so it
      cannot drift (the `SV-CORPUS-DENOMINATOR` co-publication shape). (`.4`)
- [x] A family with no corpus says so LOUDLY as `NOT MEASURED`, never as a blank or a zero —
      `docs/decisions/feedback_a_check_that_cannot_run_must_say_so.md`. (`.3`)
- [x] The main book gains one cross-family view so the whole picture is one page, not ten. (`.5`)
- [x] **A producer exists that emits the brief form for one family in a screenful, and I report it
      in completion messages** — the director reads it in chat and is never expected to run it.
      (`.1`; the cross-family brief added by `.5` reports all ten in one screenful.)
- [x] **Terminal and book cannot disagree**: `--check` re-derives and diffs the published block,
      wired as a gate, so a stale book section fails a commit rather than misleading a reader.
      (`.4`, extended to the roster page by `.5`.)

⭐ **All six criteria as written are MET, and the tree stays `active` anyway** — deliberately, not
as bookkeeping drift. Five leaves remain open and every one of them was UNCOVERED by doing this
work rather than planned with it: `.3b` (the two families whose corpus was measured once and cannot
be re-derived), `.6` (the hand-written status prose class, measured at 1 of 9 books), `.7` (raise
the remaining-work item from a TREE to a LEAF), `.8` (the 3 disputed + 7 unreadable tracker
statuses that make every count an upper bound) and `.10` (the `Current frontier` section absent
from 19 of 61 open trees). Closing the tree on its original criteria while those stand would
publish exactly the comfortable label this tree exists to replace with a distance.

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
- ID: `DONE-BAR-VISIBILITY.3b` — Status: `done` (`PGEN-DONE-BAR-VISIBILITY-0010`)
  Goal: turn `vhdl` — the first of the two `corpus_wired_no_census` families — into a real count
  that the tree can re-derive, plus the freshness gate that keeps it true.
  Acceptance: MET. The census was RE-MEASURED on HEAD (13 720 / 4 370 / 9 350, 31.9 %), the tracked
  report is now read by a producer and a doctrine instead of by nobody, and `CORPUS-CENSUS-CURRENCY`
  (`scripts/check_corpus_census.sh`, registry 32 -> 33) re-derives it on every commit. ⛔ The block
  publishes it as a RAW rate with NO defect count — `defects` is `None`, never `0` — and says out
  loud that it moves leg 3 not at all.
  ⚠️ **SCOPE SPLIT, recorded rather than silently narrowed**: the leaf as written named `vhdl` AND
  `regex`. They are two different census SHAPES (a raw parse census over a file tree; an
  answer-keyed oracle over a normalized case set), so they are two gates' worth of RED controls and
  two renderers. `vhdl` ships here with the shared `kind` dispatch it needed; `regex` is now `.3c`
  and reuses that machinery. Landing half a gate to keep one leaf id was the worse trade.
- ID: `DONE-BAR-VISIBILITY.3c` — Status: `done` (`PGEN-DONE-BAR-VISIBILITY-0011`)
  Goal: the second half of `.3b` — turn `regex` into a real count. Unlike VHDL it HAS an answer key
  (PCRE2's own compile verdicts), so its census is `cells / matches / false-accepts / false-rejects`
  and its pass share IS a conformance share.
  Acceptance: MET. `CORPUS-CENSUS-CURRENCY` gained a second `kind` — `oracle_expectation` — whose
  freshness leg re-runs the normalizer into a SCRATCH directory in 0.11 s and compares all three
  tracked artifacts byte-for-byte, and whose sharpest arm holds the published verdict tuple's cell
  count equal to a RE-DERIVED denominator rather than to a stored constant. Published in the regex
  book and the roster page; the roster now reads **3 of 10 families have a corpus count at all**.
  ⛔ The producer defect the leaf was opened on is FIXED: `pcre2_compile_oracle_summary.json` held
  ABSOLUTE paths into a foreign checkout, so the tracked artifact was a function of whose machine
  ran it and the byte-comparison leg was impossible by construction.
- ID: `DONE-BAR-VISIBILITY.2a` — Status: `done` (`PGEN-DONE-BAR-VISIBILITY-0008`)
  Goal: correct the status reader that manufactured a tracker disagreement out of its own bias, and
  correct every surface that published the resulting number.
  Acceptance: the row's own status is the FIRST status token in its cell; the change is priced over
  the whole population; the regression is pinned by a self-check arm.
- ID: `DONE-BAR-VISIBILITY.3d` — Status: `done` (`PGEN-DONE-BAR-VISIBILITY-0012`)
  Goal: a DIRECTOR CHALLENGE to the findings `.3b`/`.3c` published — *"do you still stand by your
  findings?"* — re-audited claim by claim against the artifacts rather than re-asserted.
  Acceptance: MET. Every published number re-derives exactly; **one claim is CORRECTED** (the
  census backing *"nothing read the report"* used a path-literal grep that cannot see a composed
  path, and the corpus runner reads that very report on every run); **one real hole was found and
  closed at zero cost** (the staleness budget's `engine_paths` was too narrow); **two honest limits
  were added** that the shipped gate did not state. No finding is withdrawn.

- ID: `DONE-BAR-VISIBILITY.10` — Status: `pending`
  Goal: the `Current frontier` section that `docs/TASK_TREE.md` calls REQUIRED is absent from **19
  OPEN trees**, under any heading spelling. Measured 2026-08-30 (re-audit): of 113 index-linked tree
  files, 61 are open or carry an unreadable status, and 19 of those carry neither a
  `Current frontier` heading nor any frontier-shaped heading. ⛔ The first number published for this
  was **40 of 115**, which counted CLOSED trees (which need no frontier) and files carrying the
  section under another spelling — the population, not the pattern, was the defect.
  Acceptance: each of the 19 either gains a frontier section or is adjudicated as not needing one
  (closed, or a tree whose frontier lives in the index row); then rule on whether the requirement
  should be enforced or relaxed, since a REQUIRED section absent from 19 of 61 open trees is either
  a rot or a rule nobody believes.
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
- ID: `DONE-BAR-VISIBILITY.5` — Status: `done` (`PGEN-DONE-BAR-VISIBILITY-0009`)
  Goal: the cross-family page in the main book.
  Acceptance: MET — `docs/book/src/roadmap-and-live-status.md` § *The Distance Behind The Labels*
  carries a `DERIVED` block with three tables (distance, corpus, and the attribution partition of
  the open trees), rendered by the SAME `derive()` that produces each family's own page, and
  `--check`-held by `FAMILY-STATUS-CURRENCY` on every commit. The refactor that made one derivation
  serve three surfaces is proven byte-identical on all 20 pre-existing renderings.
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
  row's Status cell and the tree file's own `Metadata` Status — DISAGREE on **3 of 113** trees
  (`EBNF-BOOK`, `RGX-0090`, `RGX-0091` — each an index row reading `done` over a tree file reading
  `active`), and **7** index rows carry no
  recognisable status word at all (`ENGINE-UNIVERSAL-SERVICES`, `GRAMMAR-WELLFORMED`,
  `LIVE-DOC-CONTAINMENT`, `SV-CORPUS-GRAD`, `MCP-CONTROL`, `JSON-RFC8259`,
  `REJECTS-VALID-DEFENSE`). `.2` counts every one of them OPEN — the safe polarity — so the
  published remaining-work count is an UPPER BOUND today, and saying so is not the same as fixing
  it.
  ⛔ It read FOUR when first published; `VERILOG-2005-PROFILE` was manufactured by this tree's own
  reader and is corrected in `.2a`.
  Acceptance: each of the 10 is adjudicated to one true status in both places; then rule on whether
  a doctrine should hold the two arms equal, or whether one arm should stop being a source.
- ID: `DONE-BAR-VISIBILITY.6` — Status: `pending`
  Goal: the HAND-WRITTEN STATUS PROSE class, found by `.1` and routed rather than logged. The
  SystemVerilog book's `welcome.md` carried three figures and all three were wrong — the AST-dump
  schema published as `10` in one paragraph and `13` in another while the contract carried `26`,
  and the parser described as *"validated each release by the … external-corpus (14/14) … gates"*
  while `audit_done_bar.sh` records the only gate reading that corpus as a TRIAGE gate, leaving
  leg 3 unmet. `.1` fixed that ONE page by replacing the prose with a pointer to the derived
  surface. ⭐ **THE CLASS IS NOW MEASURED, not assumed** (2026-08-30, re-audit): of the nine other
  parser books, **ONE** carries a maturity claim that disagrees with today's register —
  `docs/rtl_frontend_parser_book/src/welcome.md:25` publishes *"the rtl_frontend family's
  live-tracker row is **`Done`**"* while the register says `Mostly Done` and the audit reports 3 of
  3 legs unmet. It is era-dated (*"As of `RTL-FE-CLOSURE.8` (2026-06-15)"*), which is the repo's
  supersede-don't-mutate convention, but it sits in a consumer-facing book and reads as current.
  `vhdl`'s schema `3` MATCHES its contract and `json`'s *"first grammar to report
  fully_certified=true"* is historical and still true — so the class is 1 of 9, not a general rot.
  ⛔ And the guard census: of the 32 registered enforcers, only FOUR read a per-parser book at all —
  `check_parser_book_currency.sh` (the changelog surface), `check_family_status.sh` (the DERIVED
  block), `check_regex_oracle_anchor_sync.sh` (one tuple in the regex validator chapter) and
  `check_gate_reachability.sh` (incidentally) — and none of them reads a book's status prose:

  ```
  for s in $(grep -o '|scripts/[a-z_]*\.sh"' scripts/check_doctrines.sh | sed 's/|//;s/"//'); do
      n=$(grep -c _parser_book "$s"); [ "$n" -gt 0 ] && echo "$s $n"; done
    scripts/check_regex_oracle_anchor_sync.sh 1
    scripts/check_gate_reachability.sh 1
    scripts/check_parser_book_currency.sh 7
    scripts/check_family_status.sh 4
  ```
  Acceptance: every per-parser book's status prose is censused; each figure is either DERIVED by a
  producer or DELETED in favour of a pointer; no per-parser book carries a hand-typed maturity
  number afterwards.

## Current frontier

⛔⛔ **THE WHOLE TREE IS PARKED BEHIND THE SV LANE — director correction, 2026-08-30**: *"Do not lose
sight on the main objective to bring the SV parser in a release ready state … I can read your
messages talking about shipping VHDL and regex corpus censuses. I am disputing that."* The director
is right. `.1` was justified as INSIDE the standing lane lock (SystemVerilog first, because it is
the release lane). `.3b`/`.3c` were **VHDL and regex artifact changes**, which the lock forbids, and
they were taken because this frontier said so — **without re-testing the frontier against the lock**.
⇒ **a tree's frontier is not an authorisation.** The three shipped slices stand on their merits and
are gate-held; nothing is withdrawn. The leaves below are ROUTED and OWNED, and are resumed only
when the SV release lane releases them.

1. `DONE-BAR-VISIBILITY.8` (the 3 DISPUTED + 7 unreadable tracker statuses). It is the frontier
   because the corpus axis has run out of cheap wins: with `.3b` and `.3c` shipped, the roster reads
   **3 of 10 families have a corpus count at all**, and the seven that remain are not measurement
   gaps this tree can close — three are `pgen`-authored languages where no third-party corpus can
   exist, three await `DONE-BAR.3`'s ownership ruling, and one awaits `CORPUS-GRAD-ALL.1` wiring a
   corpus at all. What IS still closable here is the honesty of the remaining-work count, which
   every surface currently publishes as an UPPER BOUND.
2. `DONE-BAR-VISIBILITY.6` (the hand-written status prose class — 1 of 9 books measured wrong).
3. `DONE-BAR-VISIBILITY.7` (raise the remaining-work item from a TREE to a LEAF), blocked on the
   leaf-status convention defect it names.

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

### `.2a` — the reader that manufactured a disagreement out of its own bias

Shipped in `PGEN-DONE-BAR-VISIBILITY-0008`, on a director challenge to the findings `.1`–`.4`
published: *"I guess you double, triple checked the accuracy of your finding … ensure that's sota
and signoff."*

**What was wrong.** `polarity()` read a tree's status by taking ANY backticked status word in the
index row's cell, preferring OPEN. The index's convention is `| TREE | status (narration…) |`, so
everything after the first token is narration ABOUT LEAVES. `VERILOG-2005-PROFILE`'s own status is
`complete`, its tree file agrees, and leaf `.6.3` is described as `blocked` deep in the cell — so
the reader called the row OPEN, the second arm called it CLOSED, and the tree published a **tracker
disagreement that does not exist**.

**What it cost.** It was published to the director as one of *four* disagreements, and it inflated
SystemVerilog's open-tree count from 4 to 5.

**The fix and its price.** The first status token in the cell wins; a cell with NO status token stays
UNREADABLE and is still counted OPEN and NAMED, so the safe polarity is kept where it belongs.
Priced across the whole population: of 113 rows exactly **ONE** verdict moves, and it is the one
hand-verified in both arms. ⭐ The regression is pinned by a new `rw_self_check` case carrying the
exact shape — a `complete` row whose narration says `blocked`.

⇒ **A safe-polarity default is still a MEASUREMENT.** Reporting it as a property of the subject
("the tracker's arms disagree") is a different claim from reporting it as a property of the reader,
and only the second one was true here.

#### Acceptance checklist (`TOOLBOX.md`)

- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `polarity()` in `scripts/report_family_status.sh`. WHY,
      re-derived from the tracked artifacts themselves, BOTH arms read directly:

      ```
      git ls-files docs/TASK_TREE.md | xargs grep -o '^| `VERILOG-2005-PROFILE` | `[a-z]*`'
        | `VERILOG-2005-PROFILE` | `complete`
      git ls-files docs/tasks/VERILOG-2005-PROFILE.md | xargs grep -m1 -o '^- Status: `[a-z]*`'
        - Status: `complete`
      ```

      Both arms say `complete`. The row's backticked words in order are `complete`, `done`, `done`,
      …, `blocked`, … — the `blocked` belongs to leaf `.6.3` deep in the narration, and
      any-token-wins reached it. ⇒ the disagreement was manufactured by the READER, not observed in
      the subject. `bash -n scripts/report_family_status.sh` clean before and after the fix.
- [x] **ADDRESSED (verified)** — before: SystemVerilog published `5 open tree(s) … ⚠️ 1 status
      DISPUTED`. After: `4 open tree(s) own this family's work: SV-CORPUS-GRAD, SV-EXH-PROOF,
      SV-PARSE-STRICT, SV-REPLAY-DEBT` and `⚠️ 0 status DISPUTED … 1 unreadable`. The repo-wide
      disagreement census falls 4 → 3, and an independent re-derivation outside the producer agrees
      row for row (`EBNF-BOOK`, `RGX-0090`, `RGX-0091`), each hand-checked against both files.
- [x] **NO REGRESSION** — the change is priced over the whole population: **1 of 113** verdicts
      moves. `bash scripts/check_doctrines.sh` ALL 32 PASS; producer probes 12/12; gate probes 7/7;
      `mdbook_docs_gate` PASS; all ten blocks republished and **byte-identical** to a fresh
      derivation (`check_family_status.sh` green).
- promotion: declined (the durable half is recorded where the director's directive lives — docs/decisions/feedback_verify_a_claim_three_ways_before_publishing_it.md gained a dated section carrying this exact table of what the re-audit found, which is the retrievable home for a lesson about how I publish rather than about the parser)

- ⚠️ **ROUTED 2026-10-09 by `SV-CORPUS-GRAD.49` — the index reader splits a row on EVERY pipe, escaped or not, so the status it reads is the prefix up to the cell's first `\|`.** `scripts/report_family_status.sh:374` (`cells = ln.split("|")`) and `:556-557` (the same shape); the `SV-CORPUS-GRAD` index row carries 50 escaped pipes, and when a `.49` draft placed a regex with `\|` before the cell's first status word, `polarity()` read the truncated prefix, returned `UNREADABLE`, and the SV page's block flipped to *⚠️ the index row carries no status word; counted OPEN* while the cross-family view's unreadable count rose 6 → 7 — the ratchet `TABLE-ARITY-RATCHET` reads the same row GFM-aware (an escaped pipe is NOT a separator) and passed it. The verdict today is right by accident of token order, not by construction. Fix when this tree resumes: split with `re.split(r'(?<!\\)\|', ln)` at both sites and add a RED arm whose row carries `\|` before its status word. Not SV-blocking (the row was rewritten without the regex); parked here with the mechanism named.

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

### `.3b` — the VHDL corpus becomes a count the tree re-derives, not a number somebody remembered

Shipped in `PGEN-DONE-BAR-VISIBILITY-0010`.

**The artifact asked for this check in its own words, and nobody had written it.**
`stimuli/vhdl/characterization/characterization.md` is TRACKED and carries an *Instrument identity*
table under this instruction:

> *"Re-hash these three inputs; if any hash differs from the row below, **this report no longer
> describes your tree** and the honest act is to re-measure, not to quote."*

Nothing in the repository re-hashed them. The census — **1 of 2** corpus counts this project has
at all — had gone stale, in the flattering direction, behind a 56-second measurement.

**What landed.**

1. **The census re-measured on HEAD** — `13 720` files · pass **4 370** · fail **9 350** · timeout 0
   · crash 0 · **31.9 %**, promoted into the tracked report.
2. **`scripts/check_corpus_census.sh`** — the new doctrine `CORPUS-CENSUS-CURRENCY` (registry
   **32 → 33**), five arms, 0.18 s.
3. **A census `kind` dispatch** in `family_status_block_v0.json` and in
   `scripts/report_family_status.sh`, because SystemVerilog's adjudicated census and VHDL's raw one
   are not the same shape and rendering one through the other's template publishes fields that do
   not mean what they say.
4. **The block published** in the VHDL parser book and in the platform book's cross-family roster,
   `--check`-held by `FAMILY-STATUS-CURRENCY` on every commit.

**The number that was there, and how it got there.** Three different VHDL pass counts were live in
this repository at once, and all three are dated 2026-08-08: the tree quoted `4 031 / 9 689`
(`.2.1`'s re-measure), the tracked report published `4 335 / 9 385` (after `.2.2`/`.2.3` grew the
grammar), and the tree today measures `4 370 / 9 350`. The first was superseded and never chased;
the second went stale and nothing could see it.

**⭐ THE DIRECTION IS THE FINDING, NOT THE SIZE.** A rising pass count is the least trustworthy
signal in this set — `PGEN-CORPUS-GRAD-ALL-0006`'s own commit subject says so — so the two runs were
diffed **per file**, not compared as totals:

```
paths only in old: 0   only in new: 0
moved: 35   Counter({('fail', 'pass'): 35})
```

**35 fail→pass, ZERO pass→fail**, over `ghdl` (+34) and `nvc` (+1). Had it been +40/−5 the totals
would have read +35 just the same.

**Two rulings.**

1. **NO SECOND ARTIFACT.** The obvious design — a machine-readable census JSON beside the report,
   with a `BASELINE-IDENTITY` block — was rejected. Two tracked files holding the same counts is a
   second thing to drift, and this doctrine exists *because a number drifted*. The tracked report IS
   the census; the gate and the producer read the same file and their two derivations are held
   equal, which is the SystemVerilog arm's own shape one family over.
2. **`defects` IS `None`, NEVER `0`.** The VHDL corpus has no answer key: many of its files are
   intentionally invalid, so a parse-FAIL is frequently the CORRECT outcome. A `0` in a *known
   defects* cell reads as *nothing is wrong* where the truth is *nobody has looked* — the exact
   shape `docs/decisions/feedback_a_check_that_cannot_run_must_say_so.md` forbids. Both renderers
   and the cross-family table carry the `None` through as `⛔ n/a` beside a `⚠️ raw` pass share,
   and the block says the number moves leg 3 not at all.

**⚠️ Two calibrations, both measured rather than chosen.** The grammar-identity arm is a HARD
failure because `grammars/vhdl.ebnf` has moved **12 times** in the life of this repository, so the
lifetime cost of that arm is at most 12 re-runs. The engine arm is a NOTE on a **derived budget**
because the generated parser moves whenever the ENGINE does — `BASELINE-IDENTITY` learned that a
blocking identity arm meant *one comment line* in the SV grammar blocked every commit. The budget
is **72**, the one drift episode this repository has measured. ⚠️ **HONEST LIMIT, stated in the
gate's own header**: 72 is the size at which drift was DETECTED, not a size proven safe — it may
have crossed earlier and nobody was looking.

#### Acceptance checklist (`TOOLBOX.md`)

- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: the tracked census report was read by NO gate, workflow,
      hook or Makefile target, so its own instrument-identity instruction was unexecutable advice.
      WHY, measured — the census over the whole tracked tree at `HEAD` (excluding `docs/`,
      `CHANGES.md`, `DEVELOPMENT_NOTES.md`, which are era-dated prose):

      ```
      $ git ls-tree -r HEAD --name-only | while read f; do
          git show "HEAD:$f" | grep -lq 'stimuli/vhdl/characterization' && echo "$f"; done
        KNOWLEDGE_MAP.md                      (prose)
        TOOLBOX.md                            (prose)
        stimuli/sv/cluster_rejects_valid.py   (reads results.tsv, invoked by hand)
        stimuli/vhdl/characterization/characterization.md   (itself)
        stimuli/vhdl/subs/PROVENANCE.md       (prose)
      ```

      ⇒ **zero** enforcers, zero gates, zero workflows. And the drift it hid, re-hashed at HEAD:
      recorded generated parser `a90ae37b…` vs tree `48a5d294…` (`grammars/vhdl.ebnf` **identical**
      at `df5b8f94…`), across `git rev-list --count 09576657..HEAD` = **418** commits, **72** of
      them touching `rust/src` and **0** touching the VHDL grammar. ⛔ `results.tsv` being
      gitignored — which is what `.3` recorded as the blocker — turned out to be a red herring for
      the DENOMINATOR question: the totals have been tracked in `characterization.md` the whole
      time. Nobody read them.
- [x] **ADDRESSED (verified)** — before: `vhdl` rendered `⛔ NOT MEASURED — the register declares
      external corpus roots, but no TRACKED census turns them into counts a reader can re-derive.
      Owed by DONE-BAR-VISIBILITY.3b.` After: `13 720 | 4 370 (31.9 %) ⚠️ raw | ⛔ n/a`, with a
      five-row outcome table, a ten-row per-sub-corpus table and the RAW caveat, in both the VHDL
      book and the roster page. The gate: `corpus-census[vhdl]: OK (tuple 13720/4370/9350/0/0)` in
      **0.18 s**; `bash scripts/report_family_status.sh --check-all` → *OK (11 published blocks …
      equal a fresh derivation)*. The cross-family line moved from *1 of the 10 families has a
      corpus count at all* to *2 of the 10 families have*.
- [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh` **ALL 33 PASS** (28.6 s);
      `make -C rust SHELL=/bin/bash mdbook_docs_gate` **PASS** (all ten per-parser book gates + the
      main book); `scripts/check_family_status.sh` 4.0 s (was 3.4 s — the new freshness gate is
      invoked once per VHDL derivation). **The SystemVerilog block is byte-identical**: `--check-all`
      is green on all 11 surfaces and only the two VHDL-bearing pages changed, which is the point —
      this slice adds a family's numbers and must move nobody else's. Per-file no-regression on the
      measurement itself: **0 pass→fail** over 13 720 files. Zero Rust, zero grammar, zero generated
      bytes; the pending run was written to `PGEN_CORPUS_OUT_DIR` and DIFFED before promotion, and
      the canonical run then reproduced its totals exactly — a free determinism control.
- [x] **RED CONTROLS** — the gate's own `--self-test` drives **14** controls in-process (a GREEN
      control, five refusal shapes, the section-scoping and heading-exactness pins, and the
      rounding-tolerance boundary from both sides). Separately, every arm was driven against the
      **REAL tracked report** through the `PGEN_FAMILY_STATUS_SCHEMA` seam, pointing at mutated
      COPIES so nothing tracked was touched: green control `rc=0`; grammar digest moved → FAIL;
      totals rate typed to `91.9%` → FAIL; a sub-corpus file count moved → FAIL; **the denominator
      arm driven ALONE**, with the arithmetic kept perfectly consistent, → FAIL (proving arm 4 is
      not shadowed by arm 2); engine digest moved with 0 commits since HEAD → NOTE, `rc=0`; the same
      drift at 72 commits with the budget at 72 → NOTE, and at budget 0 → FAIL. ⭐ **The controls
      found two defects before the gate shipped**: a heading reader using `endswith`, so
      `## Not Totals` matched `Totals` and a report that had LOST its totals section read as still
      having one; and a `%`-formatted template that broke on the report's own percent signs.
- [x] **LOCKSTEP** — tree + frontier + `docs/TASK_TREE.md` + `DOCTRINE_ENFORCEMENT.md` §10 + the
      book's `DOCTRINE-COUNT` (32 → 33) + the VHDL parser book + the platform book's roster +
      `CHANGES.md` + `DEVELOPMENT_NOTES.md` + `MEMORY.md`.

- promotion: declined (the two durable halves are already carried by records this leaf cites — *an
  instrument with no gate behind it is wrong the next time it is quoted* is `SV-CORPUS-DENOMINATOR`'s
  founding sentence and the reason this doctrine copies its shape, and *a rising pass rate is not
  evidence of correctness* is already a promoted card that this leaf simply obeyed by diffing per
  file; the budget calibration is a one-artifact fact and lives in the gate that applies it)

### `.3c` — the regex corpus, where the answer key is real and the anchor turned out to be right

Shipped in `PGEN-DONE-BAR-VISIBILITY-0011`.

**A census has a KIND.** SystemVerilog's corpus is adjudicated, VHDL's is raw, and regex's has a
REAL answer key — PCRE2 10.47's own recorded compile verdicts — so its pass share IS a conformance
share where VHDL's is not. Rendering the three through one template would publish fields that do not
mean what they say, so `corpus_census` dispatches on `kind` and an unimplemented kind REFUSES.

**⭐ MEASURED BEFORE PUBLISHED, and this time the number held.** `.3b` found the VHDL census stale;
the honest thing was to ask the same question of regex rather than assume the answer generalised. The
full oracle gate was re-run on `HEAD` (80 s, peak 5.5 GB) and produced
`parse_expectation_match_total: 1879` / `false_accept_total: 262` / `false_reject_total: 48` over
`cases_executed: 2189` — **`2189/1879/262/48`, exactly the published anchor**. Two families, the same
question, opposite answers; only running it could tell them apart.

**⛔ THE DEFECT THAT MADE THE FRESHNESS LEG IMPOSSIBLE, and it was found by trying to build the leg.**
`regex_corpus_bundle/corpus/pcre2/canonical/pcre2_compile_oracle_summary.json` is tracked and carried
an `outputs` block holding ABSOLUTE paths into a home-directory checkout on a different VOLUME from
this repository. The normalizer wrote `str(args.output_jsonl)` — the caller's path — so the tracked
artifact was a function of WHOSE MACHINE ran it. Two consequences, and the second is why it blocked
this leaf: it violates the repo-root-relative path policy, and it made the summary non-reproducible
BY CONSTRUCTION, so no gate could ever have compared it byte-for-byte against a fresh run. Fixed by a
`repo_relative()` helper; the regenerated artifact differs from the committed one **only** in those
three strings (`cases.jsonl` and `skips.jsonl` are byte-identical, so the corpus itself never moved).

**The four arms, and what each is blind to.**

1. **FRESHNESS** — the normalizer is re-run into a SCRATCH directory (0.11 s) and all three tracked
   artifacts must be byte-identical. ⛔ SCRATCH, never in place: a check that regenerates the file it
   then compares always passes. ⛔ ONE DECLARED EXCLUSION — the summary's `outputs` key, which
   records WHERE the run wrote and legitimately differs between a scratch and a canonical run. The
   exclusion lives in the schema, not in the script, and it is the technique
   `check_sv_corpus_denominator.sh` already uses for its own corpus-dependent column.
2. **DENOMINATOR ARITHMETIC** — cells + declined = pattern specs detected; both sides of the oracle
   detect the same population; the answer key partitions exactly the emitted cells. ⛔ **A declined
   case is COUNTED, never dropped**: 50 of the 2 239 specs are cases the census refused to ask (48
   carry a suffix modifier under which the raw pattern text is not what `pcre2_compile` sees, 2 fail
   UTF-8 decoding). A case nobody asked is not a case that passed, and publishing 2 189 as the
   denominator without saying where the other 50 went would be the same silence this tree exists to
   end.
3. **THE VERDICT ANCHOR HELD AGAINST A RE-DERIVED DENOMINATOR** — the arm that justifies this
   existing at all rather than deferring to `REGEX-ORACLE-ANCHOR-SYNC`. That doctrine holds the
   anchor's `cells` field equal to a TRACKED CONSTANT in the ratchet env, and every live anchor equal
   to every other. **Both of its arms are stored values**: if the upstream corpus moved, the constant
   and the anchors would still agree with one another and would BOTH be wrong. Here `cells` is
   re-derived from the normalizer on every run and the anchor is held equal to THAT — a stored number
   against a re-derived one — and the tuple must close over its own cells.
4. **THE DELEGATION IS EXISTENCE-CHECKED** — an unchecked reference inside a passing gate's own
   reasoning is how a dependency rots into a dangling one.

**⚠️ HONEST LIMIT, published where a reader will see it.** The VERDICT half is not re-executed per
commit: producing it needs a cargo build of the corpus probe plus a run over every case (80 s, 5.5 GB
peak). The book block says so, names the producing gate and names the doctrine that holds the
anchors, rather than letting a reader assume every figure on the page is re-derived on every commit.

**The class map is deliberately the SAME semantic as SystemVerilog's**, because the two sit side by
side in the cross-family table and a column that means different things per row is worse than no
column: `passing` = the verdict matches the key; `explained` = divergence WITH a recorded explanation
(every false REJECT is a documented strict-default divergence class, recorded case-class by
case-class in the ratchet env's v12 record); `defects` = divergence with NO recorded explanation
(the false ACCEPTS — over-acceptance is a defect outright by the north star).

#### Acceptance checklist (`TOOLBOX.md`)

- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `regex` published `⛔ NOT MEASURED` while its corpus was
      wired, normalized and anchored, and the artifact that would have carried the count recorded the
      INVOCATION rather than the census. WHY, measured in two parts.

      (a) The denominator was already re-derivable and nobody had wired it: the normalizer
      reproduces the tracked `cases.jsonl` byte-identically (`4b990bf5…`) in **0.11 s**, and a
      diff of the summaries with the invocation key removed is identical — so every field agreed
      except the one recording where the run wrote, and that one field is exactly what a
      byte-comparison trips on.

      (b) That field is read by NOTHING, so it was pure liability. The census, over the tracked
      tree:

      ```
      $ git ls-files | xargs grep -ln 'pcre2_compile_oracle_summary'
        CHANGES.md · DEVELOPMENT_NOTES.md · docs/TASK_TREE.md · docs/tasks/DONE-BAR-VISIBILITY.md
        docs/tasks/artifacts/done_bar/run_corpus_scale_census.sh
        regex_corpus_bundle/corpus/pcre2/canonical/pcre2_compile_oracle_summary.json
        regex_corpus_bundle/scripts/normalize_pcre2_compile_oracle.py
        rust/scripts/regex_pcre2_compile_oracle_gate.sh
        rust/test_data/grammar_quality/family_status_block_v0.json
      ```

      Two of those are the artifact and its producer, one is the schema declaring the exclusion this
      slice adds, four are era-dated prose. The two real code readers —
      `rust/scripts/regex_pcre2_compile_oracle_gate.sh` and
      `docs/tasks/artifacts/done_bar/run_corpus_scale_census.sh` — read `cases_emitted`,
      `expected_parse_counts` and `skip_reason_counts`, and **neither reads `outputs`**. ⇒ a field
      nothing consumes was the sole obstacle to a byte-identical freshness leg.
- [x] **ADDRESSED (verified)** — before: `regex | ⛔ NOT MEASURED | ⛔ | ⛔ | the register declares
      external corpus roots, but no TRACKED census turns them into counts a reader can re-derive.`
      After: `regex | 2 189 | 1 879 (85.8 %) | 262 |`, with a four-row outcome table, the declined-50
      accounting and the anchor-held caveat, in the regex book and the roster page. The gate:
      `corpus-census[regex]: OK (tuple 2189/1879/262/48 = cells/matches/false-accepts/false-rejects)`.
      The roster line moved from *2 of the 10 families have a corpus count at all* to **3**.
      The normalizer's `outputs` block is now repo-root-relative in the tracked artifact.
- [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh` **ALL 33 PASS**;
      `make -C rust SHELL=/bin/bash mdbook_docs_gate` **PASS**; `--check-all` green on all **11**
      published surfaces, with the SystemVerilog **and VHDL** blocks byte-identical — this slice adds
      one family's numbers and moves nobody else's. The regenerated regex artifacts differ from the
      committed ones in **3 strings inside one key**; `cases.jsonl` and `skips.jsonl` are
      byte-identical, so the corpus and the answer key provably did not move. The full oracle gate
      re-run exits 0 against its own tracked ratchet bounds. Zero Rust, zero grammar, zero generated
      bytes.
- [x] **RED CONTROLS** — eight driven against the REAL tracked artifacts through the
      `PGEN_FAMILY_STATUS_SCHEMA` seam, pointing at COPIES so nothing tracked was touched: green
      control `rc=0`; a row appended to `cases.jsonl` → STALE; a summary field moved → STALE outside
      the excluded key; the anchor's cell count moved to 2 200 → FAIL, *naming why
      `REGEX-ORACLE-ANCHOR-SYNC` cannot see it*; a tuple that no longer closes (1 878 + 262 + 48) →
      FAIL; the anchor removed → FAIL; the delegated doctrine's script renamed away → FAIL. ⭐ **And
      one control proving arm independence, which needed a normalizer WRAPPER**: any mutation of the
      tracked summary alone is caught upstream by freshness, so the denominator-arithmetic arm can
      only be reached by making BOTH sides produce the same broken identity — done, and it fires
      alone. The `raw_parse` self-test's 14 in-process controls still pass unchanged.
- [x] **LOCKSTEP** — tree + frontier + `docs/TASK_TREE.md` + `DOCTRINE_ENFORCEMENT.md` §10 + the
      registry description + the regex parser book + the platform book's roster + `CHANGES.md` +
      `DEVELOPMENT_NOTES.md` + `MEMORY.md`.

- promotion: declined (the durable half — *a stored number checked against another stored number is
  one arm, not two* — is `BASELINE-IDENTITY`'s founding sentence and `DERIVED_STATE_CONTAINMENT`
  R1/R3, both already retrievable and both cited here; the `outputs`-block defect is a one-artifact
  fact and lives in the producer that emits it)

### `.3d` — the director challenge: every published finding re-audited against the artifacts

Shipped in `PGEN-DONE-BAR-VISIBILITY-0012`. Director, 2026-08-30, verbatim: *"do you still stand by
your findings? the call is yours to make but it got to be sota and signoff."*

⭐ **The answer is YES on the substance and NO on one piece of evidence, and the difference is the
whole point of re-auditing rather than re-asserting.** `.2a` had already shown what a challenge is
worth here: two of five findings moved, both in the direction that made them sound bigger.

**What re-derives EXACTLY.** Every number, anchored at the census's own recorded `HEAD` `41e934da`
(a bare commit count is a number that moves, so the anchor is now stated with it):

| Published | Re-derived | Command |
|---|---|---|
| 418 commits since the stale census | **418** | `git rev-list --count 09576657..41e934da` |
| 72 of them touching the engine | **72** | `… -- rust/src` |
| 0 touching the VHDL grammar | **0** | `… -- grammars/vhdl.ebnf` |
| 12 VHDL grammar edits, all time | **12** | `git log --oneline -- grammars/vhdl.ebnf \| wc -l` |
| registry 32 → 33 | **33 rows** | `grep -c '^  "' scripts/check_doctrines.sh` |
| regex tuple `2189/1879/262/48` | **identical** | the gate's own `summary.txt` |

**⛔ THE ONE CORRECTION, and it is about METHOD, not about the conclusion.** The census backing
*"nothing read this report"* grepped the tracked tree for the LITERAL string
`stimuli/vhdl/characterization`. The corpus runner composes that path as
`$ROOT/stimuli/$FAM/characterization`, so a reader of exactly that shape is invisible to the
pattern — **and there is one**: `stimuli/run_external_corpus.sh` reads the tracked report back on
EVERY run (provenance reconciliation, lines 215-222), and four other tracked scripts read the same
identity table:

```
$ git ls-tree -r HEAD --name-only | grep -E '\.(sh|py|rs|yml|mk)$|Makefile' | while read f; do
    git show "HEAD:$f" | grep -q 'parse binary' && echo "$f"; done
  docs/tasks/artifacts/eus67/measure_parse_rss.sh
  docs/tasks/artifacts/sv_corpus_grad/timeout_reconfirm_control/run_control.sh
  stimuli/run_external_corpus.sh
  stimuli/sv/audit_explained_svpp.py
  stimuli/sv/corpus_parse_cost.py
```

⇒ **the phrasing *"read by nothing"* is FALSE** and it was never the claim worth making. What is
TRUE, and what the doctrine actually rests on, is narrower and was checked directly: the runner
takes **field 2** of that table row — the PATH — and never **field 3**, the DIGEST
(`awk -F'`' '…{print $2}'`, line 222). **Nothing re-hashed the declared inputs**, which is exactly
why the report's own instruction was unexecutable advice. The finding stands; its evidence is now
stated at the strength it actually has.

⭐ The transferable half, and it is the same lesson this repository has paid for before: **a
path-literal grep cannot see a composed path, so before trusting such a census, name one reader it
would score ZERO on.** Here that reader is the producer itself — the most obvious candidate in the
tree, and the one the pattern was structurally blind to.

**⛔ ONE REAL HOLE, FOUND AND CLOSED AT ZERO COST.** The staleness budget counted commits touching
`rust/src` only. The binary that emits the parser also has `rust/build.rs` and `rust/Cargo.toml` as
inputs, so a narrow set UNDERCOUNTS — which delays a NOTE becoming a FAILURE, i.e. it errs in the
PERMISSIVE direction. Widened, after measuring the cost:

```
rust/src        72 commits   rust/build.rs    1   rust/Cargo.toml   3
union of all three ........ 72 commits  (the two extras are subsets)
```

⇒ strictly more conservative, calibration **unchanged**, price **zero**. It is still not PROVEN
exhaustive, and the gate now says so.

**⚠️ TWO HONEST LIMITS THE SHIPPED GATE DID NOT STATE, now in its header.** (1) The grammar-identity
arm hashes ONE FILE, so it is complete only for a grammar with no `include(...)` — measured for the
shipped customer, `grep -c 'include(' grammars/vhdl.ebnf` is **0**, so there is no blind spot today,
but a composing grammar would need the include CLOSURE. (2) `engine_paths` is a declared set and a
declared set can be wrong; the widening above is a measurement, not a proof.

**⚠️ A CAVEAT ON `.3b`'S STRONGEST ARM, which I did not publish and should have.** The
*35 fail→pass, 0 pass→fail* per-file diff was taken against the previous run's `results.tsv`, which
is gitignored and was OVERWRITTEN by the promotion run minutes later. **That diff is RECORDED, not
re-derivable.** What still corroborates it from tracked state is its own arithmetic: the old file
carried 35 fewer passes than the new 4 370, i.e. **4 335** — exactly the number the superseded
tracked report published. So the baseline was provably the right one, and the direction claim rests
on that identity rather than on a file that still exists.

**What is UNCHANGED after the re-audit.** The `REGEX-ORACLE-ANCHOR-SYNC` finding is confirmed
against the WHOLE script (93 lines, not the 80 first read): its five legs compare the contract
anchor, the validator chapter, the how-to, `MEMORY.md` and the tracked ratchet constants, plus a
sum check over the anchor's own fields. **Every input is a stored value** — so *agreement among
stored values is consistency, never currency* holds as published. The `outputs`-block finding is
confirmed with a composition-aware census: the only other candidate,
`regex_pcre2_textsafe_corpus_gate.sh`, reads a DIFFERENT artifact
(`pcre2_textsafe_summary.json`), and both real readers of the compile-oracle summary reference
`outputs` **zero** times.

#### Acceptance checklist (`TOOLBOX.md`)

- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `.3b`'s root-cause census, and the `engine_paths` scope
      in `family_status_block_v0.json`. WHY, measured: the census pattern was a path LITERAL while
      the repository's own producer composes the path, so the instrument was structurally blind to
      the reader most likely to exist. Re-run composition-aware:

      ```
      $ git ls-files | grep -E '\.(sh|py|rs)$' | xargs grep -ln 'characterization'
        -> stimuli/run_external_corpus.sh among them; it reads the tracked report on every run
      $ git rev-list --count 09576657..41e934da -- rust/src rust/build.rs rust/Cargo.toml   -> 72
      $ git rev-list --count 09576657..41e934da -- rust/src                                 -> 72
      ```

      ⇒ the reader census was wrong in its phrasing and the budget scope was narrow in the
      permissive direction; both are corrected here rather than argued away.
- [x] **ADDRESSED (verified)** — `engine_paths` widened to three paths with the union measured at
      the same 72, so the shipped budget is unchanged and strictly more conservative; the gate
      header now carries the census blind spot, the include-closure limit and the declared-set
      limit; the leaf carries the corrected reader list and the `results.tsv` caveat. Gate still
      green on both families: `corpus-census[vhdl]: OK (tuple 13720/4370/9350/0/0)` and
      `corpus-census[regex]: OK (tuple 2189/1879/262/48)`.
- [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh` ALL 33 PASS; `--check-all` green on all
      11 published surfaces, every block **byte-identical** — this slice corrects evidence and
      widens a scope, and must move no published number. Zero Rust, zero grammar, zero generated
      bytes.
- [x] **RED CONTROLS** — the widened scope was proven to still FAIL past budget and NOTE inside it,
      by re-running the two budget controls from `.3b` against the new three-path set; the 14
      in-process self-test controls and the eight `.3c` seam controls are unchanged and green.
- [x] **LOCKSTEP** — tree + `docs/TASK_TREE.md` + the gate header + the schema + `CHANGES.md` +
      `DEVELOPMENT_NOTES.md` + `MEMORY.md`.

- promotion: declined (the durable half is already a promoted card — *before trusting a proxy
  metric, name one instance it would score ZERO on* — and this is its fourth instance, cited rather
  than restated; the widened path set is a one-artifact fact and lives in the schema that declares it)

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

### `.5` — the cross-family view: the whole roster on one page

Shipped in `PGEN-DONE-BAR-VISIBILITY-0009`. The platform book's live-status chapter gains
**§ *The Distance Behind The Labels***, a `DERIVED` block re-derived and diffed by
`FAMILY-STATUS-CURRENCY` on every commit exactly like the ten family pages.

**What landed.**

- `scripts/report_family_status.sh --cross-family` — the chat form (the whole roster in a
  screenful, reported by me, never by the director) — `--cross-family --markdown` (the book block)
  and `--cross-family --check FILE` (re-derive and diff). `--check-all` now covers **11** published
  subjects: the ten family pages and the roster page, in ONE pass.
- **The producer was refactored so that there is exactly ONE derivation.** The per-family pass is
  now a function `derive(family)`; the brief form, the book block and each row of the roster page
  are RENDERINGS of what it returns. A roster table that computed its own numbers would have been
  an eleventh surface to keep in sync — the very class this tree exists to remove — and gating it
  would only have made a disagreement loud. Deriving it makes the disagreement unrepresentable.
- The open/closed verdict for each index row is now computed ONCE at module scope rather than
  per family, for the same reason and one more: ten families × 113 tree files is ten times the
  file reads, and ten recomputations are ten chances for the ten answers to disagree about one row.
- `rust/test_data/grammar_quality/family_status_block_v0.json` declares a THIRD surface, `cross`,
  on the 8 fields the roster publishes. The field-set check is now surface membership in both
  directions, so a field rendered on the roster page with no declared producer refuses exactly as
  it does on a family page. `language_owner` is deliberately NOT on the roster: the label table
  directly above it already publishes that column, and a second copy on one page is a second thing
  to keep true.
- `scripts/check_family_status.sh` tier A now holds the roster page to the same three structural
  questions as a family page, through its own `PGEN_FAMILY_STATUS_CROSS_PAGE` seam — and the marker
  must name `--cross-family`, so a single family's block pasted onto the page that promises the
  whole picture fails statically, before any derivation is paid for.

**What the page answers that ten pages could not.** Three derived tables, in the order a reader
asks them:

1. **Distance** — every family's claimed label beside `legs met`, the three leg marks, and the
   number of open trees owning its remaining work. Two families can now be compared without
   opening two books, which was the whole gap.
2. **Corpus** — rows, passing, known defects and what the number rests on. **1 of 10 families has
   a corpus count at all**; the nine `NOT MEASURED` rows each carry the DERIVED reason and the leaf
   that owes one, and the block says in terms that ⛔ is *the absence of an instrument*, not a zero.
3. **What remains, across the whole index** — the attribution partition of the OPEN trees:
   **62 of 113 trees are open** — 15 attributed to a family, 26 `cross-cutting`, 14 `process`,
   4 `future-family`, 3 `platform`. The bucket descriptions are read from the schema's own
   vocabulary rather than typed, and a bucket with no vocabulary entry REFUSES rather than
   publishing a count nobody can interpret.

⛔ The page states its own limit where a reader will see it: the counts are an **UPPER BOUND**,
because the two prose arms of a tree's status disagree on **3 of 113** rows and **7** index rows
carry no status word the reader recognises, all counted OPEN. `.8` owns reconciling them and `.7`
owns raising the item from a TREE to a LEAF.

**Cost, measured, and it is why the gate can still run on every commit.** Ten separate
`--markdown` renders: **23.771 s**. The roster render, which derives all ten families: **2.597 s**
— **9.2×** cheaper, because the two `audit_done_bar.sh` runs are family-independent and are paid
once. The whole doctrine went **3.4 s → 3.7 s** for an eleventh published surface.

⚠️ **Two guards in the roster renderer have NOT been observed firing, and saying so is the point.**
The partition arm (the per-family lists must sum to the count of family-attributed OPEN rows) and
the agreement arm (all ten families must report the same cross-cutting count) compare two results
of the same closed derivation, so no input can separate them — driving either red would mean
editing the code, not constructing a state. They are defensive invariants, and this leaf records
them as such rather than counting them among the arms that were driven.
`docs/knowledge/a-control-that-cannot-fail-is-not-a-control.md` is why the distinction is written
down instead of quietly enjoyed.

#### Acceptance checklist (`TOOLBOX.md`)

- [x] **ROOT CAUSE (WHY + WHERE)** — WHERE: `docs/book/src/roadmap-and-live-status.md`, which
      published the label table and then said, in its own words, *"The cross-family view that will
      live here is `DONE-BAR-VISIBILITY.5`"*. WHY, measured with the ops/build-flow instruments
      rather than asserted:
      `git ls-files 'docs/*_parser_book/src/status-and-distance.md' | wc -l` → **10** published
      distance pages, one per family and none anywhere else; and the platform book had never
      carried one in the whole history —
      `git log -S'BEGIN DERIVED: scripts/report_family_status.sh' --oneline -- docs/book/src/roadmap-and-live-status.md | wc -l`
      → **0** commits, as does `git log -S'--cross-family' --oneline -- docs/` → **0**. So every
      derived distance in the repository lived on a per-family page, and a reader who wanted to
      compare the roster had to open ten books and hold ten answers in their head. That is the
      director's *"no end in sight"* remark in its cross-family form: each family could say how far
      IT was, and nothing said how far the PROJECT was. (Corroborated by the two greps this leaf
      first cited — `BEGIN DERIVED` count on the page **0 → 1**, and files carrying the producer's
      marker **30 → 31**, the 30 being the ten pages plus their 20 tracked HTML renderings.)
- [x] **ADDRESSED (verified)** — after: the same greps read **1** and **31**;
      `bash scripts/report_family_status.sh --cross-family --check docs/book/src/roadmap-and-live-status.md`
      → `OK (published block equals a fresh derivation, 6975 bytes)`; and
      `bash scripts/check_family_status.sh` reports tier A over *"10 pages + the cross-family view"*
      and tier C over **11** published blocks, in **3.7 s**.
- [x] **NO REGRESSION** — the refactor that made one derivation serve three surfaces is proven
      byte-identical on the surfaces that already existed: all ten families were rendered in BOTH
      modes before the change and again after (`rust/target/dbv5_baseline` vs
      `rust/target/dbv5_after`, 20 renderings), and `diff` reports **0 differing bytes on 20 of
      20**. Beyond that: `bash scripts/check_doctrines.sh` ALL 32 PASS;
      `make -C rust SHELL=/bin/bash mdbook_docs_gate` PASS; producer probes **18/18**; gate probes
      **12/12**.
- [x] **RED CONTROLS** — `run_family_status_probes.sh` grew from 12 arms to **18**: one GREEN
      control (the published roster page equals a fresh derivation) and five RED — hand-written
      prose added inside the published block (caught, exit 1); `--cross-family` with a family named
      (refused); `--cross-family` with `--check-all` (refused, because accepting both would let a
      reader believe two things were checked when one was); an attribution bucket with no
      vocabulary entry (refused); and a `|` inside a schema blurb, which GFM would silently read as
      a column boundary (refused — `TABLE-ARITY-RATCHET`'s founding measurement is 26 of 197 rows
      of a SHIPPED contract losing their consumer-impact column exactly that way).
      `run_family_status_gate_probes.sh` grew from 7 arms to **12**: a GREEN control on a copy of
      the real roster page, then the roster page ABSENT, a single family's block on it, the block
      duplicated, and a **structurally perfect but STALE** page — the last one is the arm that
      proves tier A and tier C are not redundant, since tier A passes it and only the oracle sees
      the drift. Every state is CONSTRUCTED through a seam; no tracked page is mutated.
- promotion: declined (the durable rules exercised here are already carried — one derivation behind
  every rendering is this tree's own founding decision and is recorded in the producer's header,
  refusing before trusting is `docs/knowledge/a-control-that-cannot-fail-is-not-a-control.md`, and
  the silent-pipe class is `TABLE-ARITY-RATCHET`'s own founding measurement)

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
- 2026-08-30 (`.2a`) — director challenge to the published findings. Re-audited all five: two were
  wrong in the direction that makes a finding sound bigger (4 disagreements -> 3, one manufactured
  by this tree's own reader; 40 of 115 missing frontiers -> 19 open trees), one was unmeasured and
  is now measured (1 of 9 books, not nine), two hold. Reader fixed, 1 of 113 verdicts moves.
- 2026-08-30 (`.4`) — `FAMILY-STATUS-CURRENCY` registered; 32/32 doctrines; gate probes 7/7 with
  five RED arms constructed under a seam; 3.4 s for all ten pages.
- 2026-08-30 (`.9`) — nine pages, nine SUMMARY entries, nine rebuilt HTML trees; `--check` green on
  all TEN books; mdbook gate PASS; 31/31 doctrines.
- 2026-08-30 (`.3`) — the nine no-count families split into FOUR derived classes, each naming its
  owing leaf; the class is held equal to the done-bar register and RED-10 proves the drift refuses.
  12/12 probes, 31/31 doctrines, mdbook gate PASS, SV block byte-identical.
- 2026-08-30 (`.3d`) — DIRECTOR CHALLENGE to the published findings. Every number re-derives exactly
  (418/72/0/12/33, anchored at the census's recorded HEAD). ONE claim CORRECTED: the reader census
  used a path-literal grep blind to a composed path, and `stimuli/run_external_corpus.sh` reads the
  report on every run — the narrow claim (nothing re-hashed the DIGESTS) survives, its evidence is
  restated at true strength. ONE hole closed at zero cost (`engine_paths` widened, union still 72).
  TWO honest limits added. Nothing withdrawn. ALL 33 doctrines PASS; all 11 blocks byte-identical.
- 2026-08-30 (`.3c`) — regex census wired: the oracle gate RE-RUN on HEAD (80 s, 5.5 GB peak) read
  2189/1879/262/48, EXACTLY the published anchor — measured before published, and unlike VHDL it
  held. The normalizer's tracked summary carried absolute paths into a foreign off-volume checkout,
  which made the byte-comparison freshness leg impossible; fixed, and the regenerated artifacts
  differ in 3 strings inside one key with `cases.jsonl`/`skips.jsonl` byte-identical. Eight RED/green
  controls through the seam, one of them needing a normalizer wrapper to reach the arithmetic arm
  past the freshness arm. ALL 33 doctrines PASS; mdbook gate PASS; `--check-all` green on 11
  surfaces with the SV and VHDL blocks byte-identical.
- 2026-08-30 (`.3b`) — VHDL re-measured on HEAD: 13 720 / 4 370 / 9 350 (31.9 %), **35 fail→pass
  and 0 pass→fail** on a per-file diff of the two runs. `CORPUS-CENSUS-CURRENCY` registered
  (32 → 33), 0.18 s, 14 self-test controls plus eight RED/NOTE arms driven against the real tracked
  report through the schema seam — two of which found defects in the gate before it shipped. ALL 33
  doctrines PASS (28.6 s); mdbook gate PASS; `--check-all` green on all 11 surfaces with the
  SystemVerilog block byte-identical.
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
- `.2a` DONE (the status reader corrected under a director challenge) — `PGEN-DONE-BAR-VISIBILITY-0008`
- `.5` DONE (the cross-family view — the whole roster on one page) — `PGEN-DONE-BAR-VISIBILITY-0009`
- `.3b` DONE (the VHDL census re-measured, and CORPUS-CENSUS-CURRENCY behind it, 32 -> 33) — `PGEN-DONE-BAR-VISIBILITY-0010`
- `.3c` DONE (the regex census, kind `oracle_expectation`) — `PGEN-DONE-BAR-VISIBILITY-0011`
- `.3d` DONE (the director challenge: findings re-audited, one corrected) — `PGEN-DONE-BAR-VISIBILITY-0012`

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
- 2026-08-30 — `.3c` DONE. Both `corpus_wired_no_census` families now publish real counts, and the
  roster reads **3 of 10 families have a corpus count at all**. The corpus axis of this tree is
  exhausted: the seven remaining families are not measurement gaps it can close. Frontier moves to
  `.8`, the tracker-status disagreements that make every remaining-work count an upper bound.
- 2026-08-30 — `.3b` DONE for `vhdl`. The second family in this project to have a corpus count a
  reader can re-derive, and the first whose census artifact was READ by anything. New leaf `.3c`
  carries the `regex` half, with its freshness leg already proven to reproduce byte-identically in
  0.11 s and one defect already found in its tracked summary (absolute paths into a foreign
  checkout). Frontier moves to `.3c`.
- 2026-08-30 — `.2` DONE at TREE granularity. 113 index rows adjudicated into a closed two-sided
  attribution; every family now publishes a finite open-tree list. New leaves `.7` (per-leaf
  granularity, blocked on the leaf-convention defect) and `.8` (the 4 DISPUTED + 7 unreadable
  tracker statuses the two-arm check found).
