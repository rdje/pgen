# LIVE-MEANS-LIVE — `LIVE_ACHIEVEMENT_STATUS.md` is 94.7 % changelog, and nothing watches it

- **Status: `active`** — opened 2026-07-31, session #229, by **DIRECT DIRECTOR ORDER**:
  > *"the file is called LIVE_ACHIEVEMENT_STATUS.md — if it keeps antique quotes, it ceases to
  > be live, it becomes a museum. It feels to me as if we shouldn't keep those historical
  > quotes around, they are just misleading. Live means live, fullstop."*
- **Trigger**: `LANG-CAPABILITY-AUDIT.10.6` part 2 routed a stale tracker cell to the director
  rather than rewriting it, on the reading that *"history surfaces are not rewritten"*.
  ⛔ **That reading was WRONG and the director corrected it.** The rule exists for `CHANGES.md`,
  `DEVELOPMENT_NOTES.md` and git — surfaces whose JOB is to be a record. Applying an
  append-only rule to a file named LIVE is a category error.
- **Axis**: the *published live-status surface*. Distinct from `README-POLICY` (which governs
  `README.md`) and from `DONE-BAR` (which governs what a status VALUE means). This tree governs
  whether the file's contents are **currently true**.

## The measurement (2026-07-31, at commit `3cb4b95b`)

| quantity | measured |
|---|---|
| file size | **1 547 057 B / 1 685 lines** |
| `Tracker note (<date>):` entries | **856 lines, 1 465 550 B — 94.7 % of the file** |
| the actual live status tables | **24 rows, 42 854 B — 2.8 %** |
| tracker-note date span | 2026-03 → 2026-07 (62 / 332 / 119 / 171 / 172 per month) |
| explicit staleness markers | 29 × `HISTORICAL`, 13 × `superseded`, 5 × *"Historical record of the landed proof follows"*, 1 × *"no longer true"* |
| single largest line | **38 265 B** (L555, a 2026-07-01 tracker note) |

⇒ **it is not a live tracker containing some history; it is a changelog containing a live
tracker at 2.8 %.**

### ⭐⭐⭐ The history is a DUPLICATE — this is what makes the purge safe

452 distinct `PGEN-<FAMILY>-<NNNN>` slice IDs are cited by the 856 tracker notes. ⚠️ The first
census below scanned only two files and was **too narrow**; the corrected one is authoritative.

| census | scope | reachable without the tracker |
|---|---|---|
| ⛔ first (superseded) | `CHANGES.md` + `DEVELOPMENT_NOTES.md` only | 449 / 452 — 99.3 %, apparently **3 orphans** |
| ✅ **corrected (`.1a`)** | **every durable layer** — the two above plus all of `docs/tasks/` and `docs/decisions/` (1 764 tracked files, 124.2 MB) | **452 / 452 — 100.0 %** |
| (cross-check) | git commit messages | 428 / 452 — 94.7 % |

⛔ **There are ZERO orphans**, and the rescue precondition `.0` imposed is dropped. The three IDs
the narrow census flagged — `PGEN-DEFAULT-PROFILE-0001`, `PGEN-DEFAULT-PROFILE-0002`,
`PGEN-REGEX-PCRE2-0030` — are all in `docs/tasks/`, the very trees their tracker notes cite for
the full analysis.

⇒ deleting the tracker notes does not destroy history. It removes a **second, diverged copy**
of a changelog that already exists — and the divergence is exactly what the director caught.

⭐ **The lesson, banked**: when a census decides whether a DELETE is safe, enumerate the
destinations from the layer model (`MEMORY_ARCHITECTURE.md`'s four layers), not from the two
files that came to mind.

### ⛔ WHY IT GREW — the README cap redirected the pressure instead of removing it

`README-STABILITY` caps `README.md` by BOTH a line count and a byte count, and its own overflow
rule (`scripts/check_readme_stability.sh:83`) routes *"family status / Done-bar claims"* **into
`LIVE_ACHIEVEMENT_STATUS.md`**.

A census of the enforcers that name this file:

| enforcer | what it checks about the tracker |
|---|---|
| `check_published_version_currency.sh` | the regex identity pair + ONE family-status string |
| `check_diagnostics_and_docpaths.sh` | repo-relative doc PATHS inside it |
| `check_readme_stability.sh` | names it only as a **destination** for overflow |

⇒ **no size cap, no staleness check, no changelog-leakage tripwire — nothing.** `README.md` is
capped on two axes; the file it overflows INTO is capped on none. That is the mechanism, and it
means a one-time purge without an enforcer only resets the clock.

⭐ **The general lesson, and it is the same one `LANG-CAPABILITY-AUDIT.10.6` just paid for**: a
cap that redirects content rather than deleting it has not solved anything — it has moved the
problem to whichever neighbouring surface has no instrument. Capping a file is only half a fix;
the other half is checking where the overflow lands.

## ⚠️ A measurement of mine that FAILED — recorded so nobody re-uses it

I tried to split the 856 notes into "standing directive" vs "pure landed-slice history" with a
keyword classifier (`director|DOCTRINE|standing|policy|MUST|binding|…`). It reported
243 / 349 / 264. ⛔ **The numbers are WRONG and are not used by this tree**: the first six
"directive-bearing" hits were all ordinary landed-slice reports that merely contain the word
*"director"*. A classifier with no ground truth is a guess
([[feedback_instrument_needs_ground_truth]]) — and this one was checked against its own first
sample and failed.

⇒ **the sound basis is routing, not keyword classification**: `docs/decisions/` (139 records) is
the authoritative home for standing directives. The test for each note is therefore *"is the
durable rule this note carries already recorded in layer C?"* — migrate it if not, then delete
the note. That is the project's own **copy → verify → use → delete** discipline applied to prose.

## Acceptance Criteria (tree)

⚠️ **REVISED by `.0` after the director's second question.** The original criteria assumed the
file survives and gets policed; the adjudication concluded it should be migrated and deleted, so
they are restated against that target. The superseded wording is not kept here — this is a
task-tree, not a history surface, which is the tree's own point.

- **No free-form prose surface carries live status.** The hand-authored claim is a
  `claimed_status` field in `done_bar_family_register_v0.json` — schema-bounded, so it cannot
  accumulate a changelog.
- **The two-arm check survives intact**: a human still authors the claim, the 3 family-status
  gates still compute truth independently, and a mismatch still fails. ⛔ Nothing becomes
  generated-from-the-thing-it-checks.
- **The human view lives in the mdBook** — the director's stated only window — and is
  gate-checked against the register.
- **Nothing is lost**: every durable rule reachable from `docs/decisions/`, every slice record
  from `CHANGES.md` / `DEVELOPMENT_NOTES.md` / `docs/tasks/`, proven by an ID-level census before
  AND after (✅ **452/452 measured in `.1a` — zero orphans**, so no rescue is required).
- **No dangling references**: all 96 tracked `.md` referrers re-pointed, and every enforcer that
  names the file (`check_published_version_currency.sh`, `audit_done_bar.sh`,
  `check_diagnostics_and_docpaths.sh`, `check_readme_stability.sh`,
  `ci_workflow_local_gate.sh`) reads the register instead.
- ⭐ **No new enforcer is needed** — `.2` is dropped. Removing the unbounded container beats
  guarding it.

## Leaves

### `.0` — ⚖️ DIRECTOR QUESTION: *"is `LIVE_ACHIEVEMENT_STATUS.md` even needed anymore?"* (`done` — adjudicated on measurement, 2026-07-31)

⛔ **Asked mid-`.1`, and it correctly BLOCKS `.1`** — purging a file that should be deleted is
wasted work, so this is adjudicated first.

**ANSWER: the FILE is needed. 97 % of its CONTENT is not. Two different questions, and only the
second one has an obvious answer.**

Measured — who actually consumes it:

| consumer | what it reads |
|---|---|
| `sv_parser_family_status_gate.sh:443`-class (×3: sv / vhdl / regex) | **ONE status word per family row**, via `markdown_table_status_for_row`, compared against the status the gate itself COMPUTED (`*_tracker_alignment_ok`) |
| `check_published_version_currency.sh` | the regex family-status string + the identity pair, cross-checked against `PGEN_USER_GUIDE.md` |
| `audit_done_bar.sh`, `done_bar_family_register_v0.json` | the family rows |
| `check_diagnostics_and_docpaths.sh` | doc PATHS inside it |
| `ci_workflow_local_gate.sh`, `check_readme_stability.sh` | its existence / its role as an overflow destination |
| 96 tracked `.md` files | link to it |

⇒ **the load-bearing content is 24 table rows — really one word per row.** The 856 tracker notes
and every evidence narrative are read by **nothing**, mechanical or human-required.

#### ⭐⭐⭐ But do NOT "just generate it" — that is the trap

The obvious next thought is: the gates already compute the status, so emit the table from their
`summary.json` and stop hand-maintaining it. **That would destroy the check.**

The gate computes what is **TRUE** from proof surfaces. The file states what we **CLAIM**,
written by a human. `vhdl_tracker_alignment_ok` fails when those differ — which is exactly how a
published false claim gets caught, and it is the same mechanism `PUBLISHED-VERSION-CURRENCY`
uses to hold `PGEN_USER_GUIDE.md` honest.

Generate the file from the gate and both arms become the same arm: the comparison passes by
construction and can never fail again. ⛔ **That is precisely the vacuous-floor failure
`LANG-CAPABILITY-AUDIT.10.9` had to repair** (`276 < 0` can never fire), and the same shape as
the one-sided ratchet `.10.6` part 2 refused to ship. A hand-written claim is not duplication
here — **being independently authored is the whole point.**

#### ⛔ CORRECTION — the director asked the sharper question and the answer above is WRONG

*"What added value does it bring that can't be derived via other means (task-tree, KM, ADR, …)?"*

The reasoning above establishes that **an independently-authored claim** must exist. It does
**not** establish that a 1.5 MB free-form Markdown file in the repo root must be it. Those are
different propositions and the first was quietly used to justify the second.

Measured against every other surface:

| surface | can it hold the claim? |
|---|---|
| `KNOWLEDGE_MAP.md` | ⛔ no — **derived** (regenerated by the pre-commit hook, 35 facts / 241 question keys). A derived file can never be the independent arm. |
| `docs/decisions/` (ADR, 139 records) | ⛔ no — records a **decision at a point in time**. A status that moves weekly is not a decision. |
| `docs/tasks/` (task-trees) | ⛔ no — tracks **work**, not the deliverable's state. Related, not the same. |
| `CHANGES.md` / `DEVELOPMENT_NOTES.md` | ⛔ no — **history** by charter. That is the confusion this whole tree exists to undo. |
| ✅ `rust/test_data/grammar_quality/done_bar_family_register_v0.json` | **YES** — already tracked, already read by the done-bar machinery, already carries this exact family set (`gate_prefixes` / `corpus_roots` / `language_owner` / `standard` / `leg3_surface`). It carries the status **inputs** and simply lacks a `claimed_status` field. |

⇒ **the one load-bearing value belongs in the REGISTER, not in a prose file** — and the register
is *structurally better*, not merely equivalent: **a JSON field with a schema cannot accumulate
856 tracker notes.** Free-form Markdown with no schema and no cap is not just where the rot
happened, it is *why* it happened. `.2`'s enforcer would be a guard policing a format that
invites the very thing it guards against.

And the human-readable at-a-glance view has an owner already: **the mdBook**, which is the
director's stated only window into the project (*"I review the book, not the code"*). A
root-level status file is, by that standing directive, a surface they do not read.

#### ✅ REVISED ANSWER — **NO, the file is not needed.** Migrate and delete it

| what it does today | where it goes |
|---|---|
| the hand-authored status claim (24 rows, 1 word each) | `done_bar_family_register_v0.json` → new `claimed_status` field, gate-compared exactly as today |
| the at-a-glance human view | an mdBook page, gate-checked against the register |
| the 856 tracker notes (94.7 %) | **deleted** — 99.3 % already in `CHANGES.md`; 3 rescued first |
| the evidence narratives | **deleted** — read by nothing; the gates' own `summary.json` is the evidence |

⭐ **This keeps the two-arm check intact** — a human still authors `claimed_status`, the gate
still computes truth, and a mismatch still fails. Nothing becomes a tautology. What is removed is
the *unbounded prose container*, so `.2`'s cap-and-tripwire enforcer is **no longer needed at
all**: you cannot leak a changelog into a JSON field with a schema.

⚠️ **Priced honestly — this is the more expensive option**: it touches the 3 family-status gates,
`check_published_version_currency.sh`, `audit_done_bar.sh`, `ci_workflow_local_gate.sh`,
`check_readme_stability.sh`'s overflow rule, a new book page, and re-points 96 `.md` references.
The cheaper path (purge to 24 rows + add the `LIVE-STATUS-CURRENCY` enforcer) is a smaller diff
that leaves a prose file to police forever. ⇒ **Recommended: migrate and delete.** The cheap path
buys a guard; this one removes the thing needing guarding.

#### ⚠️ Pre-check `.1` MUST run first — link rot

96 tracked `.md` files reference this file. Before deleting 94.7 % of it, measure how many
**deep-link into a section or anchor that the purge removes**. A link to the file survives; a
link to a purged heading does not. Census first, fix or re-point, then purge.

### `.1a` — migrate the family-status CLAIM into the DONE-BAR register (`done`)

- **Status: `done`** (`PGEN-LIVE-MEANS-LIVE-0002`), on the director's explicit order:
  *"Ok, go for migrate-and-delete"*. This is the load-bearing half — after it, **no gate reads
  `LIVE_ACHIEVEMENT_STATUS.md` for a status value**.

#### ⛔ CORRECTION — there are ZERO orphans, not 3. The delete is provably lossless

`.0` reported 3 slice IDs *"cited nowhere else"* and made rescuing them a precondition. ⛔ **That
was an artifact of a too-narrow census**: it scanned only `CHANGES.md` and
`DEVELOPMENT_NOTES.md`. Re-run across **every** durable layer (both of those, plus all of
`docs/tasks/` and `docs/decisions/` — 1 764 tracked files, 124.2 MB):

| census | result |
|---|---|
| slice IDs cited by the 856 tracker notes | **452** |
| reachable from a durable layer **without** the tracker | **452 / 452 — 100.0 %** |
| also reachable from git commit messages | 428 / 452 — 94.7 % |
| ⛔ orphaned in BOTH docs and git | **0** |

⇒ **nothing needs rescuing.** The rescue precondition is dropped. `PGEN-DEFAULT-PROFILE-0001`,
`PGEN-DEFAULT-PROFILE-0002` and `PGEN-REGEX-PCRE2-0030` are all in `docs/tasks/` — the very trees
their tracker notes cite for the full analysis.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — the family-status CLAIM lived in a free-form Markdown table cell of
  a 1 547 057 B file that was 94.7 % dated changelog, scraped by
  `markdown_table_status_for_row` via `grep -F` + `awk -F'|' '{print $3}'`. A prose surface with
  no schema is what let 856 tracker notes accumulate around a value that is one word long.
- [x] **ROOT CAUSE (WHY + WHERE)** — the claim's *format*, not its content. Located with the
  ops/build-flow tool `git ls-files`, which enumerates the tracked shell surface so a reader
  cannot be missed by a path guess:

  ```
  $ git ls-files -- 'rust/scripts/*.sh' 'scripts/*.sh' | xargs grep -n markdown_table_status_for_row
  rust/scripts/lib/parser_family_status_bar.sh:44:markdown_table_status_for_row() {
  rust/scripts/sv_parser_family_status_gate.sh:876:  …"$(markdown_table_status_for_row "| `systemverilog` main parser" "$LIVE_TRACKER_FILE")"
  rust/scripts/sv_parser_family_status_gate.sh:877:  …"$(markdown_table_status_for_row "| `systemverilog_preprocessor` frontend" …)"
  rust/scripts/vhdl_parser_family_status_gate.sh:443: …"$(markdown_table_status_for_row "| `vhdl` parser family |" …)"
  rust/scripts/regex_parser_family_status_gate.sh:452:…"$(markdown_table_status_for_row "| `regex` parser family |" …)"
  scripts/check_published_version_currency.sh:60:      …"$(markdown_table_status_for_row '| `regex` parser family |' "$TRACKER")"
  ```

  ⇒ **5 call sites**, all funnelling through the one reader at
  `rust/scripts/lib/parser_family_status_bar.sh:44`, which does
  `grep -F "$row_match" "$path" | awk -F'|' '{print $3}'` — it extracts a one-word value from the
  third pipe-cell of a Markdown row. **WHY that is the root cause**: the value's container is a
  free-form Markdown file with no schema and no cap, so nothing bounded what else could accrete
  around it — and 856 dated changelog entries did, reaching 94.7 % of a 1 547 057 B file. The
  same `git ls-files` census re-run AFTER the fix returns **empty** (exit 1) outside the library,
  i.e. zero remaining Markdown-cell readers.
- [x] **FIX** — declarative tier. `claimed_status` added to all 7 families in
  `rust/test_data/grammar_quality/done_bar_family_register_v0.json`;
  `markdown_table_status_for_row` replaced by `claimed_status_for_family FAMILY`, which REFUSES
  (never defaults) on a missing register, a missing family, or an absent/empty `claimed_status`.
  ⛔ The register's `policy` block records, in the file itself, that this field is hand-authored
  and **must never be auto-populated** — generating it from the gates would make the comparison
  pass by construction.
- [x] **ADDRESSED (verified)** — measured before → after:

  | measurement | before | after |
  |---|---|---|
  | gates reading a status value from `LIVE_ACHIEVEMENT_STATUS.md` | 4 (sv ×2, vhdl, regex) + the currency check | **0** ✅ |
  | the claim's container | a Markdown cell in a 1.55 MB unbounded file | a schema-bounded JSON field ✅ |
  | `claimed_status_for_family` over all 7 families | n/a | all 7 return their exact tracker value ✅ |
  | `make vhdl_parser_family_status_gate` | pass | **pass**, `vhdl_tracker_alignment_ok: true`, `live_tracker_file` → the register ✅ |
  | `scripts/check_published_version_currency.sh` | pass | **pass** — *"published status 'In Progress' == tracker"* ✅ |
- [x] **THE TWO-ARM CHECK STILL FIRES — proven in both places, not assumed.** This is the whole
  risk of the migration: a check that silently stopped comparing would be worse than the file it
  replaced.

  | negative control | result |
  |---|---|
  | register claim mutated `Provisional (corpus pending)` → `Done`, gate re-run | **`rc=1`**, `vhdl_tracker_alignment_ok: false`, *"error: VHDL tracker alignment mismatch: computed 'Provisional (corpus pending)' but tracker says 'Done'"* — and it names BOTH arms ✅ |
  | register claim mutated `In Progress` → `Done`, currency check | **`rc=1`**, naming guide-vs-register ✅ |
  | `claimed_status` key deleted entirely | **refusal**, *"is the hand-authored arm of the status check and is never defaulted"* ✅ |
  | family absent from the register | **refusal**, *"must BLOCK the gate, not score well by absence"* ✅ |
- [x] **NO REGRESSION** — `bash -n` clean on all 5 edited scripts (`parser_family_status_bar.sh`,
  the three `*_parser_family_status_gate.sh`, `check_published_version_currency.sh`); register is
  valid JSON with 7 families; **`make regex_parser_family_status_gate` PASS** with
  `regex_tracker_alignment_ok: true`; no `grammars/*.ebnf`, no `rust/src/*`, no `generated/*`
  touched ⇒ all 11 generated parsers byte-identical BY CONSTRUCTION. ⚠️ The **sv** family-status
  gate was NOT re-run to completion (it drives the full SV family contract); it uses the identical
  shared reader, which two gates exercised end-to-end and one in both polarities.
- [x] **⚠️ A BUG I INTRODUCED, CAUGHT BY RUNNING IT** — the first edit put
  `LIVE_TRACKER_FILE="$(done_bar_register_path)"` at line 14, **before** the library defining that
  function is sourced. `make vhdl_parser_family_status_gate` → `command not found`, `Error 127`.
  It failed loudly rather than silently defaulting, which is the correct polarity; the assignment
  now sits after the `source` in all three gates with a comment saying why it must.
- [x] **LOCKSTEP** — the register's `description` + a new `policy.claimed_status` clause,
  `parser_family_status_bar.sh` header, `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`.

#### ⚠️ Deliberately NOT done in `.1a` — the file still exists

`audit_done_bar.sh` still derives the family ROSTER from the tracker, so the file cannot be
deleted yet. That is `.1b`, and it is a strengthening, not a transcription — see below.

### `.1b` — the family ROSTER: derive it from `grammars/*.ebnf`, not from the tracker (`done`)

⭐ **The current derivation is WEAKER than its own docstring claims.** `audit_done_bar.sh:113-127`
builds the roster as *tracker rows ∩ `grammars/*.ebnf` basenames*, and the register's description
says this exists *"so a family nobody added to a list cannot hide"*. ⛔ **Measured, it hides
exactly one way**: a grammar with **no tracker row** is simply not a family, silently. The
refusal only fires in the other direction (on the tracker but absent from the register).

⇒ invert it: derive candidates from `grammars/*.ebnf` — the product itself, which cannot lie
about what exists — and REFUSE on any grammar with neither a register entry nor an explicit
recorded disposition. That is the `GATE-REACHABILITY` pattern (invoked, or a deliberate
disposition) applied to families.

#### ⛔ THE HOLE IS LIVE, NOT HYPOTHETICAL — three shipped parsers were invisible

The pre-work sketch above guessed the 10 undisposed grammars were all inputs, bootstrap contracts
and the meta-grammar. ⛔ **Measured, that guess was wrong**, and the correction is the finding:

```
$ grep -n 'grammar_name: "' rust/src/parser_registry.rs | wc -l      # 13 registered parsers
$ python3 -c "…set(grammars) - set(register['families'])…"           # 10 invisible grammars
```

| grammar | registered generated parser? | own gated mdBook? | integration contract? | in the register? |
|---|---|---|---|---|
| `semantic_annotation` | ✅ `parser_registry.rs:1571` | ✅ `semantic_annotation_parser_book_gate` | ✅ `docs/contracts/PGEN_SEMANTIC_ANNOTATION_PARSER_INTEGRATION_CONTRACT.md` | ⛔ **NO** |
| `json` | ✅ `parser_registry.rs:1597` | ✅ `json_parser_book_gate` | — | ⛔ **NO** |
| `ebnf` | ✅ `parser_registry.rs:1590` | ✅ `ebnf_parser_book_gate` | — | ⛔ **NO** |

⇒ a family with a **published downstream integration contract** was outside the done-bar audit
entirely, and so were **8 gate targets** (`ebnf_*` ×6, `json_parser_book_gate`,
`semantic_annotation_parser_book_gate`) — attributed to no family, therefore under no leg-2
accounting. ⭐ The book already calls these families: *"every PGEN grammar — the **nine**
parser/annotation families above plus the `ebnf` meta-grammar"*
(`docs/book/src/parser-families.md`). The register said **seven**. The published surface and the
audited surface disagreed by three, and nothing could see it — because the roster was joined
against a tracker none of the three had a row in.

#### The adjudication — 10 families, 7 dispositions

Each disposition is recorded IN the register (`grammar_dispositions`), so it is a schema-bounded
fact, not a comment. **Admission test, applied uniformly:** does the grammar ship a *registered
generated parser*? That is the product, and it cannot be argued with.

| grammar | verdict | basis |
|---|---|---|
| `semantic_annotation` | **family**, `Mostly Done` | registered parser + published contract + gated book; exact sibling of `return_annotation`, which claims `Mostly Done` on the identical evidence shape (`DONE-BAR.2b`) |
| `json` | **family**, `Mostly Done` | registered parser + gated book; **measured `fully_certified=true`, `UNKNOWN=0`** (below). A deliberately simplified built-in — but that bounds its *scope*, not its *status* |
| `ebnf` | **family**, `In Progress` | registered parser + gated book + 6 gates; ⛔ but the `.10.6` envelope differential found **2 live defects that both parse `Ok`**, and only 6 of 14 grammars are envelope-equivalent ⇒ *"core capabilities or validation still missing"*, the Status Rules' own words |
| `builtin_return_annotation`, `builtin_semantic_annotation` | disposition `bootstrap_contract` | bootstrap-safe contracts that break the annotation-parser cycle (`README.md`); **EXCLUDED by name** from the parse-harness equivalence gate because the oracle is not own-grammar codegen |
| `systemverilog_2017_lrm_extracted`, `systemverilog_2023_lrm_extracted`, `verilog_2005_lrm_extracted` | disposition `lrm_extraction_input` | no registered parser; machine-extracted LRM grammar text consumed as INPUT |
| `systemverilog_lrm_profiled_generated` | disposition `derived_artifact` | no registered parser; generated FROM the extractions |
| `systemverilog_lrm_profiled_wrapper` | disposition `lrm_extraction_harness` | no registered parser; an `include(…)` wrapper over the extractions (its low envelope agreement is the include asymmetry, `TOOLBOX.md` 1.9) |

⭐ The verdict column is not free-form: **`family` ⟺ `rust/src/parser_registry.rs` ships a
registered generated parser**, with exactly one recorded exception — the two `builtin_*` bootstrap
contracts, which ARE registered (`:1577`, `:1583`) but only to break the annotation-parser cycle,
and which `parse_harness_equivalence_gate` already EXCLUDES BY NAME for that same reason. So:
12 non-`scratch` registered parsers − 2 bootstrap contracts = **10 families**, and the 5 grammars
with no registered parser are the other 5 dispositions. ⚠️ My first pass wrote *"no dispositioned
grammar has a registered parser"* — **that was wrong**, the `builtin_*` pair does; the rule needs
its stated exception to be true, and control `C8a` encodes the exception rather than hiding it.
Because the rule is pinned against the Rust registry — a tracked source the register cannot edit —
the roster is checkable rather than curated.

#### ⛔ Why the claims are hand-authored, and what they are NOT

`claimed_status` is the independent arm of the two-arm check (`.1a`). These three families had
**never carried a claim anywhere** — not in the tracker, not in the register — so there was no
prior human claim to migrate; I authored them against the published Status Rules vocabulary and
recorded the basis above. ⛔ **Not derived from any gate**, and no family-status gate computes
any of the three, so nothing here can pass by construction.

#### ⚠️ A SECOND defect, found by running the harness — and it was RED BEFORE I touched anything

`docs/tasks/artifacts/done_bar/run_done_bar_probes.sh` exited **1 (11/12)** at commit `ce1df2b0`,
before any edit of mine. Root-caused rather than assumed:

```
$ wc -c rust/target/regex_parser_family_status_gate/summary.txt      # 7377  (non-empty)
$ grep -c '^error:' rust/target/sota_exit_gate/logs/regex_parser_family_status_gate.log
0
```

`CTRL-4a` pins the substring `computed 'In Progress' but tracker says 'Done'`, recoverable only
via `gate_ran_and_failed()` — a path taken only when `find_artifact()` finds **no** summary. The
regex gate has since been re-run and PASSED, so it now has a 7 377-byte summary and a log with
**zero** `error:` lines. The pinned state is unreachable.

⭐⭐ **The general lesson, banked — a ground-truth control pinned to UNTRACKED state decays
silently.** `CTRL-4a`'s ground truth lived in `rust/target/`, which is regenerable build output:
the arm was green only until someone re-ran the gate, and then it went red for a reason having
nothing to do with the instrument it guards. Its sibling `CTRL-4b` pins a fact derived from
*tracked script text* and is still green. ⇒ **a control must either pin a tracked fact or
CONSTRUCT the state it observes.** Fixed the second way: a new `PGEN_DONE_BAR_TARGET_DIR` seam
lets the arm build the exact 0-byte-summary + error-log shape `DONE-BAR.1a` was written for, so it
now reproduces on a fresh clone. ⇒ routed as [[feedback_ground_truth_control_must_not_pin_untracked_state]].

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `bash scripts/audit_done_bar.sh` reported `families derived from
  LIVE_ACHIEVEMENT_STATUS.md x grammars/*.ebnf: 7` and exit 0, while 17 grammars are tracked and 12
  non-`scratch` generated parsers are registered. Independently, the negative-control driver
  `bash docs/tasks/artifacts/done_bar/run_done_bar_probes.sh` exited **1 — 11/12**, `CTRL-4a` red.
- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow tier. Located with `git ls-files` + the tracked
  registry, so no reader is missed by a path guess:

  ```
  $ git ls-files -- 'grammars/*.ebnf' | wc -l                              # 17
  $ grep -c 'grammar_name: "' rust/src/parser_registry.rs                  # 13 (incl. scratch)
  $ python3 -c "print(sorted(set(grammars) - set(register['families'])))"  # 10 invisible
      builtin_return_annotation builtin_semantic_annotation ebnf json
      semantic_annotation systemverilog_2017_lrm_extracted
      systemverilog_2023_lrm_extracted systemverilog_lrm_profiled_generated
      systemverilog_lrm_profiled_wrapper verilog_2005_lrm_extracted
  ```

  **WHERE**: `scripts/audit_done_bar.sh:114-131` — `for line in read(TRACKER).splitlines()` builds
  the roster from tracker rows and intersects with `GRAMMARS`. **WHY that is the root cause**: the
  join's LEFT side is the tracker, so a grammar absent from it contributes nothing and the loop
  simply never sees it. The refusal at `:136-143` fires only on the converse (on the tracker,
  absent from the register), and control `C8` pins only `register ⊆ roster` — so **every check
  guarding this derivation was on the arm that could not fail**. Second defect located the same
  way: `wc -c` = 7 377 on the regex summary + `grep -c '^error:'` = 0 on its log makes `CTRL-4a`'s
  pinned string unreachable through `gate_ran_and_failed()` (`:275-299`, reached only when
  `find_artifact()` returns `None`).
- [x] **FIX** — declarative tier first, code second. (a) The register gains 3 families and a
  `grammar_dispositions` block naming all 7 non-families with a reason; (b) the roster is derived
  from `grammars/*.ebnf` and REFUSES on an undisposed grammar, on a grammar claimed BOTH ways, and
  on a family or disposition naming a grammar that does not exist — a two-sided check, where the
  old one had a single fallible side; (c) `TRACKER` is deleted from the audit entirely (the claim
  now comes from `claimed_status`), including as a staleness input; (d) two testability seams
  (`PGEN_DONE_BAR_GRAMMARS_DIR`, `PGEN_DONE_BAR_TARGET_DIR`) so the new controls are provable
  without mutating the tracked tree.
- [x] **ADDRESSED (verified)** — measured before → after:

  | measurement | before | after |
  |---|---|---|
  | roster derivation | tracker rows ∩ grammars | **`grammars/*.ebnf`, the product itself** ✅ |
  | tracked grammars with NO adjudicated disposition | **10** | **0** ✅ |
  | audited families | 7 | **10** ✅ |
  | gate targets attributed to a family | 59 / 124 | **67 / 124** ✅ (+8: `ebnf_*` ×6, `json_*`, `semantic_annotation_*`) |
  | families with a published integration contract but no register entry | **1** (`semantic_annotation`) | **0** ✅ |
  | reads of `LIVE_ACHIEVEMENT_STATUS.md` by `audit_done_bar.sh` | 2 (roster, staleness) | **0** ✅ |
  | `bash scripts/audit_done_bar.sh` | exit 0, 10 controls | **exit 0, 12 controls** ✅ |
  | `run_done_bar_probes.sh` | **exit 1 — 11/12** | **exit 0 — 15/15** ✅ |
- [x] **THE NEW REFUSAL FIRES — proven, not assumed.** A derivation that cannot refuse is the
  vacuous floor `LANG-CAPABILITY-AUDIT.10.9` had to repair, so each arm was made to fail on purpose:

  | negative control | result |
  |---|---|
  | `RED-1` a grammar in NEITHER `families` NOR `grammar_dispositions` | **`rc=2`** — *"no register entry and no recorded disposition"*, naming the grammar ✅ |
  | `RED-1b` a grammar claimed in BOTH at once | **`rc=2`** — contradiction refusal ✅ |
  | `RED-1c` a disposition naming a grammar that does not exist | **`rc=2`** — stale-entry refusal (the converse arm) ✅ |
  | `RED-2` no `grammars/*.ebnf` at all | **`rc=2`** — an empty roster is a refusal, never a pass ✅ |
  | `CTRL-3` a dispositioned grammar is NOT admitted as a family | **exit 0, roster stays 10** ✅ |
  | `CTRL-1b` a claim re-promoted to `Done` in the REGISTER still fails the bar | **`rc=1`** ✅ |
  | `C11`/`C12` (new controls) mutated | **`rc=3` MISCALIBRATED**, each naming its own control ✅ |
- [x] **NO REGRESSION** — `bash -n` clean on both edited shell files (⚠️ `shellcheck` is **not
  installed** on this machine — stated rather than implied); register is valid JSON and
  **10 families + 7 dispositions = 17 = the tracked grammar count exactly**; the 7 pre-existing
  families are **byte-identical** (`json.load` equality against `git show HEAD:…`), so no prior
  claim moved. Oracles re-run:

  | oracle | result |
  |---|---|
  | `bash scripts/check_doctrines.sh` (staged diff in scope) | **ALL 15 doctrines PASS** ✅ |
  | `scripts/check_published_version_currency.sh` | **PASS** — *"published status 'In Progress' == tracker"*, and it reaches the register through the same shared library ✅ |
  | `claimed_status_for_family` over **all 10** families | every one returns its exact claim; an unknown family still **refuses** (`rc=1`, *"must BLOCK the gate, not score well by absence"*) ✅ |
  | `scripts/check_memory_architecture.sh` | **OK** (layer A re-trimmed to 7 064 B rather than raising the cap when it hit 7 200 B) ✅ |
  | `bash scripts/audit_done_bar.sh` | **exit 0**, 10 families, 13 controls ✅ |
  | `run_done_bar_probes.sh` | **16/16, exit 0** ✅ |

  ⏳ `make -C rust vhdl_parser_family_status_gate` was ALSO launched as the confirmatory end-to-end
  oracle and is still running at commit time (>1 h; it drives the full VHDL family contract) —
  **verification-pending, and its result is reported in the next commit** rather than claimed here.
  It is confirmatory, not load-bearing: it consumes exactly one register value (`vhdl`), which is
  byte-identical, through the shared reader two other oracles above already exercised.
  ⛔ No `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` touched ⇒ all generated parsers
  **byte-identical BY CONSTRUCTION**.
- [x] **LOCKSTEP** — register `description` + `policy.grammar_dispositions`, the audit's header
  comment, the probe driver's WHY block, `docs/book/src/quality-and-closure-model.md`, `CHANGES.md`,
  `DEVELOPMENT_NOTES.md`, `MEMORY.md`.

#### ⚠️ Deliberately NOT done in `.1b` — routed, not worked

- **`.1d`** (new) — 65 of 124 gate targets are attributed to no family at all. Most are legitimately
  cross-family (`mdbook_docs_gate`, `sota_exit_gate`, `clippy_on_rust_change`), but that is a
  *claim*, not a measurement, and leg 2 is only as strong as the attribution. Census owed.
- **`.1e`** (new) — `scripts/check_diagnosis_evidence.sh:73-76` does not count
  `scripts/audit_done_bar.sh` or `rust/test_data/grammar_quality/*.json` as a code change, so an
  edit to `claimed_status` — the hand-authored arm deciding whether a published status claim is
  checked — requires no acceptance checklist. Found while confirming this leaf's own obligations.

### `.1c` — the book page, the referrers, and the delete (`todo`)

The human at-a-glance view moves to an mdBook page (gate-checked against the register),
`check_readme_stability.sh:83` stops naming an uncapped overflow destination, and
`LIVE_ACHIEVEMENT_STATUS.md` is deleted. ✅ The ID census in `.1a` already proves the delete is
lossless (452/452).

#### ⭐⭐⭐ ANVIL CORRECTION (2026-07-31) — *"reference count is not a dependency measure"*

> *"Reference count is not a dependency measure. It inflates with hint text and with append-only
> history that must keep its references raw. Classify referents by what they REQUIRE."*

⛔ **This leaf's own "96 `.md` referrers must be re-pointed" was that inflated measure**, and it was
sized as if all 96 were blockers. Re-measured by REQUIREMENT, over the shell surface
(`git ls-files -- '*.sh' | xargs grep -ln`), **after** `.1b` landed:

| what the referent REQUIRES | count | files |
|---|---|---|
| ⛔ **reads the file's CONTENT** | **1** | `docs/tasks/artifacts/done_bar/run_demotion_impact_probe.sh` (greps a table row, copies the file) |
| requires only that the PATH EXIST | 2 | `ci_workflow_local_gate.sh` (×5 `assert_tracked`), `check_diagnostics_and_docpaths.sh` (a doc-path glob) |
| names it in a HINT / routing message | 1 | `check_readme_stability.sh:83,112,116` |
| pure COMMENT / prose provenance | 5 | `parser_family_status_bar.sh`, the 3 `*_parser_family_status_gate.sh`, `check_published_version_currency.sh`, and `audit_done_bar.sh` itself |

⇒ **the delete is blocked by ONE script, not ninety-six**, and it is a probe artifact rather than a
gate. ⚠️ ANVIL named `audit_done_bar.sh` as that one consumer — true when they wrote it, and `.1b`
has since removed it: the audit now reads the tracker for **nothing**. The remaining consumer is
`run_demotion_impact_probe.sh`, and it is **already inert** — its precondition greps for
`` | `vhdl` parser family | Done | `` while the row has read `Provisional (corpus pending)` since
`DONE-BAR.2b`, so it refuses before reading anything.

⭐ **The banked lesson**: *count what a referent REQUIRES, not how many referents there are.* A
grep -l is an upper bound made of comments, hint strings and append-only history that must keep its
references verbatim. This is the same error class as `.0`'s too-narrow census — a number produced
by the convenient query rather than the right one — and it inflated a cost estimate by ~96×.
⇒ [[feedback_classify_referents_by_requirement]].

### `.1` — (superseded — split into `.1a` ✅ / `.1b` / `.1c`)

⛔ **RESCUE BEFORE DELETE, in that order.** `PGEN-DEFAULT-PROFILE-0001`,
`PGEN-DEFAULT-PROFILE-0002` and `PGEN-REGEX-PCRE2-0030` are cited ONLY in this file; they go to
`CHANGES.md` first, verified present, and only then may the notes go. Re-run the ID census after
the purge and prove **452 / 452** still reachable from a durable layer.

⚠️ The purge is not `grep -v 'Tracker note'`: some notes carry a durable rule that layer C does
not yet record. Each is routed to `docs/decisions/` or confirmed already there before deletion.

### `.2` — the enforcer: NOT a byte cap. ANVIL's two instruments (`todo` — REOPENED and re-specified 2026-07-31)

⛔ **`.0` had dropped this leaf**: migrate-and-delete removes the prose container, so there is
nothing left to cap. **ANVIL reopened it on a better argument** — the point was never to bound
`LIVE_ACHIEVEMENT_STATUS.md`, it was to detect *any* surface that stops being a status document:

> *"A byte cap would have told you the file was big. Either of these tells you it stopped being a
> status document — which is the thing you actually wanted to know."*

#### ⭐⭐⭐ The two instruments — cheap, derivable, and needing NO baseline

| # | instrument | what it separates | measured first run |
|---|---|---|---|
| **A** | **count DISTINCT dates inside a surface** | one date = a status view · 108 = a log. Separates the mixed category **without anyone choosing a number** | `.3`'s table — the three book destinations score 1/1/2, the tracker 108. A clean 50× gap, no threshold picked |
| **B** | **a self-declared `Last updated:` that disagrees with the file's newest content date** | SELF-REFUTING: the file's own two halves contradict each other, so no external baseline is needed at all | **10 files** caught on the first run (`.4`), 5 clean — including the ROADMAP at ~3.3 months of self-declared drift |

⭐ **Why these beat the byte cap I had designed.** The `.2` fallback below proposed a line cap AND a
byte cap AND a hand-written `Tracker note` / `HISTORICAL` keyword tripwire. Every one of those needs
a number or a word-list chosen by a human, and the keyword arm is the same *keyword classifier*
shape this tree already recorded as FAILED (see "A measurement of mine that FAILED" above — a
classifier with no ground truth is a guess). Instruments A and B choose nothing: A reads a gap that
is 50× wide, B compares the file against **itself**. ⛔ And a byte cap is actively misleading here —
`gate-flow.md` is the biggest of the three book destinations and the healthiest.

⚠️ **Charter, not defect.** `CHANGES.md` scores 175 on instrument A and is CORRECT: being a log is
its charter. So the enforcer must pair each surface with its declared kind — instrument A
classifies, the charter says which classification is permitted. That pairing is the leaf's real
design work, and it is what stops A from firing on the changelog forever.

⭐ **Also close the redirect**: `check_readme_stability.sh:83` should not name an unwatched file as
an overflow destination. Once instrument A watches every destination, the README rule becomes honest
— and `.1c`'s delete removes the worst destination outright.

#### (superseded fallback design — kept only to record what was rejected and why)

Mirror `README-STABILITY` onto this file — a line cap AND a byte cap (a line cap alone is
measurably bypassable: `README-POLICY.2` recorded layer A passing a 60-line cap while carrying
138 403 bytes), plus a **changelog-leakage tripwire** on dated `Tracker note` entries and
`HISTORICAL` / `superseded` markers. ⛔ **Rejected**: three human-chosen numbers and a keyword
list, to answer a question two baseline-free instruments answer better.

### `.3` — audit the OTHER overflow destinations named by capped enforcers (`done` — measured, all three clean)

`README-STABILITY` routes overflow to four places (`docs/book/src/gate-flow.md`,
`docs/book/src/operations-and-governance.md`, `docs/book/src/developer-architecture.md`,
`LIVE_ACHIEVEMENT_STATUS.md`). One of them was measured at 1.5 MB with no instrument. **Measure
the other three before assuming they are fine** — the failure mode is structural, not specific
to this file.

#### ✅ MEASURED (2026-07-31), using ANVIL's date-count instrument (`.2` below)

| surface | distinct dates | bytes | reading |
|---|---|---|---|
| `docs/book/src/gate-flow.md` | **2** | 34 343 | status/reference doc — clean |
| `docs/book/src/operations-and-governance.md` | **1** | 17 915 | clean |
| `docs/book/src/developer-architecture.md` | **1** | 16 471 | clean |
| `README.md` | **0** | 8 286 | landing page — clean |
| ⛔ `LIVE_ACHIEVEMENT_STATUS.md` | **108** | 1 563 641 | a log |
| (charter check) `CHANGES.md` | 175 | 5 726 849 | a log **BY CHARTER** — correctly classified, not a defect |

⇒ **the other three destinations are fine, and this is now a measurement rather than an assumption.**
⭐ Note what a BYTE cap would have said: `gate-flow.md` is the LARGEST of the three at 34 KB and
would have ranked worst; its date count says it is healthy. The instrument separates *big* from
*rotted*, which is the actual question. And it does not fire on `CHANGES.md`, because being a log
is that file's charter — the instrument classifies, the charter says which classification is right.

### `.4` — 10 tracked files SELF-REFUTE their own `Last updated:` (`todo`)

Found by ANVIL's second instrument (`.2`) on its first run — no baseline, no threshold:

| file | declares | newest content date | drift |
|---|---|---|---|
| `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md` | 2026-04-22 | 2026-07-31 | **~3.3 months** |
| `PGEN_USER_GUIDE.md` | 2026-04-17 | 2026-07-29 | ~3.4 months |
| `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md` | 2026-03-26 | 2026-07-09 | ~3.5 months |
| `CHANGES.md` | 2026-05-25 | 2026-07-31 | ~2.2 months |
| `COMMIT.md` | 2026-05-14 | 2026-07-30 | ~2.5 months |
| `LIVE_ACHIEVEMENT_STATUS.md` | 2026-06-02 | 2026-07-31 | ~2 months |
| `docs/reference/PGEN_COMPILER_ELABORATOR_ENABLEMENT_ROADMAP.md` | 2026-04-18 | 2026-06-21 | ~2.1 months |
| `docs/reference/PGEN_STIMULI_MODULE_NORMATIVE_SPEC.md` | 2026-04-12 | 2026-05-18 | ~1.2 months |
| `docs/book/src/annotation-system.md` | 2026-04-26 | 2026-06-10 | ~1.5 months |
| `docs/reference/RUST_CODEBASE_ANALYSIS.md` | 2026-07-22 | 2026-07-27 | 5 days |

5 files pass (declaration == newest content), so the instrument is not firing on everything.
⚠️ **The ROADMAP is the worst offender** — the document the session bootstrap makes mandatory
reading declares itself 3.3 months staler than its own content. ⛔ Do NOT bulk-rewrite the dates:
the honest fix per file is either to update the declaration *because the content really did move*,
or to delete a self-declaration nothing maintains. `CHANGES.md` may simply drop its `Last updated:`
— an append-only changelog's newest entry IS its date, so the field is a second place to be wrong.

## Evidence

- Census commands are reproducible from the repo root; all figures above are at commit
  `3cb4b95b` and must be RE-MEASURED, not quoted, when `.1` is worked
  ([[feedback_read_prior_art_before_designing]]).
