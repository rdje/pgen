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

  ✅ **CONFIRMATORY ORACLE LANDED** (`PGEN-LIVE-MEANS-LIVE-0004`, ~1 h 5 m — it drives the full VHDL
  family contract). It was still running when `-0003` was committed and was recorded there as
  *verification-pending* rather than claimed; **`make -C rust SHELL=/bin/bash
  vhdl_parser_family_status_gate` → exit 0**, and its `summary.txt` (6 169 B) carries the three
  lines that matter:

  ```
  live_tracker_file: …/rust/test_data/grammar_quality/done_bar_family_register_v0.json
  vhdl_status: Provisional (corpus pending)
  vhdl_tracker_alignment_ok: true
  ```

  ⭐ `live_tracker_file` resolving to the **register** is the load-bearing confirmation: the gate's
  independent arm computed a status and compared it against the hand-authored claim in its new home,
  end-to-end, and they agreed. The one criterion it reports UNMET —
  `external_corpus_conformance_pass=false (leg3_surface=<none>)` — is `DONE-BAR.3`'s open work and
  is **unchanged by this leaf**. ✅ Re-run against that FRESH artifact, `audit_done_bar.sh` still
  exits 0 and the probe driver is still **16/16**, so the staleness path is exercised in both
  directions (the artifact is now newer than the register, where before it was older).
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

### `.1c` — the book page, the referrers, and the delete (SPLIT into `.1c1` / `.1c2` / `.1c3`)

The human at-a-glance view moves to an mdBook page (gate-checked against the register),
`check_readme_stability.sh:83` stops naming an uncapped overflow destination, and
`LIVE_ACHIEVEMENT_STATUS.md` is deleted. ✅ The ID census in `.1a` already proves the delete is
lossless (452/452).

#### ⛔⛔ CORRECTION — *"blocked by ONE script"* was WRONG. Re-measured: **TWO**, and the second is live

The ANVIL re-measurement below applied the right RULE (*classify referents by what they REQUIRE*)
but assigned `ci_workflow_local_gate.sh` to the **path-only** class on the strength of *"×5
`assert_tracked`"* — a mechanism it never opened. Measured at commit `f359a59c`, site by site:

```
$ for n in 286 2674 2742 2935 3032; do sed -n "$((n-2)),$n p" rust/scripts/ci_workflow_local_gate.sh; done
  L286   -> root-md roster entry .......................... requires PATH ONLY
  L2674  -> assert_file_contains LIVE_ACHIEVEMENT_STATUS.md  READS CONTENT
  L2742  -> assert_file_contains LIVE_ACHIEVEMENT_STATUS.md  READS CONTENT
  L2935  -> assert_file_contains LIVE_ACHIEVEMENT_STATUS.md  READS CONTENT
  L3032  -> assert_file_contains LIVE_ACHIEVEMENT_STATUS.md  READS CONTENT
```

⇒ **1 of the 5 is path-only; 4 are `assert_file_contains` on exact tracker PROSE**, and all four
asserted strings are **PRESENT in the file today**, so they are green now and go **RED on delete**.

| what the referent REQUIRES | `.1c`'s figure | **re-measured** |
|---|---|---|
| ⛔ reads the file's CONTENT | 1 (`run_demotion_impact_probe.sh`, inert) | **2** — that probe **plus 4 live assertions in `ci_workflow_local_gate.sh`** |
| requires only that the PATH exist | 2 | 2 (`ci_workflow_local_gate.sh` root-md roster · `check_diagnostics_and_docpaths.sh` pathspec) |
| HINT / routing text | 1 | 1 (`check_readme_stability.sh:83,112,116`) |
| pure COMMENT / provenance | 5 | 5 (unchanged) |

⭐⭐ **The lesson, and it is the THIRD instance of the same error class in this one tree** (`.0`'s
too-narrow census → `.1c`'s inflated 96 → this). The rule *"classify by requirement"* was correct;
what failed is that the classification was itself produced by a **convenient query** — one `grep -l`
per file and an assumed mechanism — instead of by reading the call sites. ⇒ **a classification is a
measurement and needs the same evidence bar as the count it replaces.** Routed to
[[feedback_classify_referents_by_requirement]] as a sharpening, not a new record.

#### The four lockstep assertions — measured before deciding what happens to them

Each tracker assertion sits in a *group* asserting one landed feature across many surfaces. Counted
by walking each enclosing function:

| site | function | arms | tracker arms | surviving DOC arms after the delete |
|---|---|---|---|---|
| L2674 | `audit_regex_corpus_bundle_surface` | 21 | 1 | guide ×3, roadmap, `RUST_CODEBASE_ANALYSIS`, `README.md` |
| L2742 | `audit_regex_pcre2_compile_oracle_surface` | 20 | 1 | guide ×2, roadmap, `RUST_CODEBASE_ANALYSIS` ×2, `README.md` |
| L2935 | `audit_sv_formal_exhaustive_closure_surface` | 15 | 1 | guide, roadmap, `RUST_CODEBASE_ANALYSIS` |
| L3032 | `audit_sv_preprocessor_formal_exhaustive_closure_surface` | 37 | 1 | guide, roadmap, `RUST_CODEBASE_ANALYSIS` |

⇒ the tracker is **one arm of 15–37**, and every group keeps **at least three other documentation
surfaces** asserting the same feature. Dropping the tracker arm therefore retires a *redundant*
arm, not the proposition — and that is a measurement, which is the bar `.1c` failed to meet above.

#### The `.md` referrer split, by charter

| class | count | disposition |
|---|---|---|
| LIVE surfaces (book, contracts, reference, root docs) | **33** | re-point → `.1c2` |
| append-only history (`CHANGES.md`, `DEVELOPMENT_NOTES.md`) | 2 | ⛔ leave verbatim — a history surface keeps its references raw |
| work record (`docs/tasks/` 54, `docs/decisions/` 8) | 62 | ⛔ leave verbatim — same reason |

### `.1c1` — the mdBook becomes the human at-a-glance status view, gate-held against the register (`done`)

Build the replacement **before** the delete. `docs/book/src/roadmap-and-live-status.md` gains the
per-family snapshot table sourced from `done_bar_family_register_v0.json`, and
`check_published_version_currency.sh` — already the doctrine holding *a published surface's status
claim == the register* for `PGEN_USER_GUIDE.md` — is extended to hold the book page to the same bar.

⭐ **Prior art, searched before designing** ([[feedback_read_prior_art_before_designing]]): the
status *rules* (three-leg bar, the qualified `Provisional` tier, the roster derivation) are ALREADY
in `docs/book/src/quality-and-closure-model.md` §§197-330. The book is missing only the **snapshot**.
⇒ the page links to that chapter and does **not** restate the ladder — no second copy to drift.

⛔ **This does not collapse the two-arm check.** The gate compares *book text* against
*`claimed_status`* — a presentation-currency check, the same shape `PUBLISHED-VERSION-CURRENCY`
already applies to the guide. The independent arms are unchanged: a human authors `claimed_status`,
the three family-status gates compute truth from proof surfaces.

#### ⭐ What the snapshot table SHOWED once it existed — no family is `Done`, and nothing said so

Rendering all 10 families side by side surfaced a fact no individual row made obvious: **`leg3_surface`
is `<none>` for every family in the register**, so leg 3 is unmet across the board and `Done` is
currently unreachable *for the whole product*, not just for the rows that happen to be discussed.
The retired tracker could not show this — it carried 14 rows across 5 sections, only 4 of them
parser families, and never listed the other 6. ⇒ recorded here rather than routed: it is `DONE-BAR.3`'s
existing work (the leg-3 wiring), not a new finding, but the *scope* of it was being under-read.

⚠️ Equally: **3 of 10 families are `language_owner: unadjudicated`** (`rtl_frontend`, `rtl_const_expr`,
`json`) — each a PGEN-delimited SUBSET of somebody else's standard. A status gate asked to qualify one
REFUSES rather than guessing. That is one third of the product whose `Provisional` qualifier cannot be
computed at all, and `DONE-BAR.3` owes the ruling.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — the human at-a-glance status view existed **only** inside the file
  `.1c3` deletes. Measured on the book, before the change:

  ```
  $ git grep -c 'LIVE_ACHIEVEMENT_STATUS' -- 'docs/book/src/**' | wc -l          # 11 book pages refer to it
  $ git grep -ln 'claimed_status\|done_bar_family_register' -- 'docs/book/src/**'
  docs/book/src/quality-and-closure-model.md                                     # 1 page, 1 line (:288)
  ```

  ⇒ the book explained the status *rules* thoroughly and published **no status**. Deleting the
  tracker without this leaf would have removed the only at-a-glance view PGEN had.
- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow tier. Located with `git ls-files` + `git grep`
  over the tracked enforcer surface, so no reader is missed by a path guess:

  ```
  $ git ls-files -- 'scripts/*.sh' 'rust/scripts/*.sh' | xargs grep -ln claimed_status_for_family
  rust/scripts/lib/parser_family_status_bar.sh
  rust/scripts/regex_parser_family_status_gate.sh
  rust/scripts/sv_parser_family_status_gate.sh
  rust/scripts/vhdl_parser_family_status_gate.sh
  scripts/check_published_version_currency.sh
  ```

  **WHERE**: `scripts/check_published_version_currency.sh` — the doctrine already holding *a
  published surface's status claim == the register*, but pointed at exactly **one** surface
  (`PGEN_USER_GUIDE.md`, one family). **WHY that is the root cause**: the doctrine's own charter is
  *"a parser SHIPS on its published state, so the published state must be gate-held true"*, and the
  book is the published state for the reader the project is written for (*"I review the book, not
  the code"*). The charter covered the book; the implementation did not reach it. So a status table
  placed in the book would have been **unheld prose** — the same class of container that let
  `LIVE_ACHIEVEMENT_STATUS.md` rot in the first place. ⇒ the page and its guard land together, in
  one leaf, deliberately.
- [x] **FIX** — declarative tier first. (a) `docs/book/src/roadmap-and-live-status.md` rewritten as
  the snapshot view: all 10 families inside an explicit `LIVE-STATUS-SNAPSHOT:BEGIN/END` marker
  pair, plus the 7 `grammar_dispositions`, plus *how this page stays true*; (b) the ladder is
  **linked, not restated** — `quality-and-closure-model.md` §§197-236 already carries it, so no
  second copy exists to drift ([[feedback_read_prior_art_before_designing]]); (c)
  `check_published_version_currency.sh` gains a **two-sided** family-by-family comparison against
  the register plus the `PGEN_PVC_BOOK_PAGE` seam.
- [x] **ADDRESSED (verified)** — measured before → after:

  | measurement | before | after |
  |---|---|---|
  | families whose status the BOOK publishes | **0** | **10** ✅ |
  | published surfaces held to the register by `PUBLISHED-VERSION-CURRENCY` | 1 (guide, 1 family) | **2** (guide + book, **11 comparisons**) ✅ |
  | copies of the status ladder in the book | 1 (`quality-and-closure-model.md`) | 1 — unchanged, the new page LINKS it ✅ |
  | `bash scripts/check_published_version_currency.sh` | `OK` (2 comparisons) | **`OK`** — *"book snapshot 10/10 families == register"* ✅ |
  | `run_published_version_currency_probes.sh` | **5/5** | **11/11** ✅ |
- [x] **THE NEW ARM FIRES — proven in every direction, not assumed.** A currency check that cannot
  fail republishes stale claims with a green tick, which is worse than no check:

  | negative control | result |
  |---|---|
  | `RED-5` book publishes a status the register does not claim | **`rc=1`** — *"publishes 'Done' for family 'vhdl' but the register's claimed_status is 'Provisional (corpus pending)'"*, naming BOTH arms ✅ |
  | `RED-6` a family silently dropped from the table (register→table arm) | **`rc=1`** — *"'json' is in the register but ABSENT from the book snapshot"* ✅ |
  | `RED-7` a phantom family published (table→register arm) | **`rc=1`** — names `phantom_family` ✅ |
  | `RED-8` the marker pair removed ⇒ table unlocatable | **REFUSAL** — *"An unlocatable table must fail, never pass by absence"* ✅ |
  | `RED-9` markers kept, ZERO rows | **REFUSAL** — *"an empty table agrees with everything and proves nothing"* ✅ |
  | `RED-10` unreadable register | **REFUSAL** — never treats "no claims" as agreement ✅ |

  ⭐ `RED-6` is the arm that matters most and it is the one `.1b` had to add to `audit_done_bar.sh`
  for the same reason: **without a register→table direction, a snapshot could shrink to one row and
  still read green.** Both directions were made to fail on purpose before either was trusted.
- [x] **NO REGRESSION** — `bash -n` clean on both edited shell files (⚠️ `shellcheck` is **not
  installed** on this machine — stated rather than implied); the 5 pre-existing probe arms still
  pass unchanged (11/11 includes CTRL-1 + RED-1..4); the register is **not touched**, so no status
  claim moved and `claimed_status` is byte-identical; `LIVE_ACHIEVEMENT_STATUS.md` is **not touched**
  either, so the 4 live `assert_file_contains` in `ci_workflow_local_gate.sh` are unaffected by this
  leaf (they are `.1c2`'s work). Oracles re-run:

  | oracle | result |
  |---|---|
  | `bash scripts/check_doctrines.sh` (staged diff in scope) | **ALL 15 doctrines PASS** ✅ |
  | `bash docs/tasks/artifacts/done_bar/run_published_version_currency_probes.sh` | **11/11, exit 0** ✅ |
  | `bash scripts/audit_done_bar.sh` | **exit 0**, 10 families, 13 controls ✅ |
  | `bash docs/tasks/artifacts/done_bar/run_done_bar_probes.sh` | **16/16, exit 0** ✅ |
  | `make -C rust SHELL=/bin/bash mdbook_docs_gate` | **PASS** ✅ |

  ⛔ No `grammars/*.ebnf`, no `rust/src/*`, no `generated/*` touched ⇒ all generated parsers
  **byte-identical BY CONSTRUCTION**.
- [x] **LOCKSTEP** — the book page itself, the enforcer's WHY block, the probe driver's WHY block,
  `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`. ⚠️ **Deliberately NOT here**: the 33 live `.md`
  surfaces still linking to `LIVE_ACHIEVEMENT_STATUS.md` — including
  `quality-and-closure-model.md:199,489,510`, which this page now links TO. The file still exists and
  those links still resolve; re-pointing them is `.1c2`, and doing it here would mix building the
  replacement with retiring the original.

### `.1c2` — retire the referrers (`done`)

The 4 live `assert_file_contains` (dropped — measured redundant above), the hint text at
`check_readme_stability.sh:83,112,116`, the dead `run_demotion_impact_probe.sh`, the
`.claude/settings.json` post-compaction resume list, and the **33** live `.md` surfaces.

⚠️ **The 2 PATH-ONLY referents are deliberately NOT here** — `ci_workflow_local_gate.sh`'s root-md
roster and `check_diagnostics_and_docpaths.sh`'s pathspec. Measured: the roster is an **exact-set**
comparison against `git ls-files` (`ci_workflow_local_gate.sh:315-320`), so removing the entry while
the file is still tracked FAILS the audit. Those two must move **atomically with the delete** ⇒ `.1c3`.

#### ⭐ `run_demotion_impact_probe.sh` is DELETED, and the measurement is why

It was classified in `.1c` as *"the ONE consumer, already inert"*. Measured further, it is **doubly
dead** — it cannot run even if its guard is removed:

```
$ git grep -n markdown_table_status_for_row -- 'rust/scripts/**' 'scripts/**'
rust/scripts/lib/parser_family_status_bar.sh:17:#   - `markdown_table_status_for_row` — …   (comment)
rust/scripts/lib/parser_family_status_bar.sh:55:# It replaces `markdown_table_status_for_row`, …  (comment)
$ git grep -ln run_demotion_impact_probe            # invoked by: nothing. 6 prose mentions only.
```

⇒ (1) its historical guard exits 0 before reading anything; (2) `.1a` **removed the function** its
step-2 replay extracts, so line 71's `sed -n '/^markdown_table_status_for_row() {/,/^}/p'` yields an
empty file and it would `exit 3 MISCALIBRATED`; (3) no gate, make target or driver invokes it.

⛔ **Its evidence is NOT lost** — the captured BEFORE record `demotion_impact_probe.txt` (2 124 B)
holds the full finding (vocabulary table, 3-row replay, verdict) and is **kept**. ⇒ delete the
script, keep the record: a script that cannot reproduce its own output is not an instrument, it is
prose with a shebang. That is this tree's own thesis applied to itself.

#### The 3 references that REMAIN, deliberately

| site | why it stays |
|---|---|
| `docs/TASK_TREE.md:98` | the director's order quoted **verbatim** — it names the file, that is the quote |
| `COMMIT.md:37` | records where `claimed_status` came FROM; naming the deleted file is the content |
| `docs/book/src/roadmap-and-live-status.md:38` | the book explaining *why* the file is gone |

⇒ all three **narrate the deletion**; none depends on the file. History (`CHANGES.md`,
`DEVELOPMENT_NOTES.md`), `docs/tasks/` (54) and `docs/decisions/` (8) keep their references raw.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — 4 live `assert_file_contains` on tracker prose would go RED the moment
  `.1c3` deletes the file, and 33 live doc surfaces would carry dangling links. Measured green-now:

  ```
  $ for s in "$(the 4 asserted strings)"; do grep -qF "$s" LIVE_ACHIEVEMENT_STATUS.md && echo PRESENT; done
  PRESENT ×4
  ```
- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow tier, located with `git ls-files` + `git grep`
  rather than a path guess:

  ```
  $ git ls-files -- '*.sh' | xargs grep -n LIVE_ACHIEVEMENT_STATUS
  rust/scripts/ci_workflow_local_gate.sh:286,2674,2742,2935,3032   ← 1 roster + 4 assert_file_contains
  scripts/check_readme_stability.sh:83,112,116                     ← hint / routing text
  scripts/check_diagnostics_and_docpaths.sh:15,53                  ← pathspec
  docs/tasks/artifacts/done_bar/run_demotion_impact_probe.sh:31,87 ← content read (dead)
  … 5 further files: comments only
  ```

  **WHY**: the tracker accumulated three *different kinds* of dependency behind one filename —
  content assertions, path-existence entries, and prose — and a `grep -l` renders all three
  identically. That is precisely how `.1c` sized the work at 1 blocker when it is 2 (see the
  correction above).
- [x] **FIX** — declarative first, in dependency order. (a) the 4 redundant assertion arms dropped,
  each with an in-place comment recording the arm count it left behind; (b)
  `check_readme_stability.sh`'s overflow routing points at the **schema-bounded register** instead of
  an uncapped prose file — ⭐ **this is the leaf that closes the actual root cause of the rot**: the
  README cap redirected pressure into a file with no instrument; (c) the dead probe deleted, its
  record kept; (d) `.claude/settings.json` and 33 `.md` surfaces re-pointed; (e) the two exact-set
  referents deferred to `.1c3`.
- [x] **ADDRESSED (verified)** — measured before → after:

  | measurement | before | after |
  |---|---|---|
  | live `assert_file_contains` on tracker prose | **4** | **0** ✅ |
  | live `.md` surfaces with a tracker reference | **33** | **3**, all narrating the deletion ✅ |
  | scripts that READ the file's content | 2 | **0** ✅ |
  | `check_readme_stability.sh` overflow destination | an uncapped 1.55 MB prose file | a **schema-bounded JSON field** ✅ |
  | `.claude/settings.json` resume list | names the doomed file | names the register + its book view ✅ |
  | lockstep arms lost per assertion group | — | **1 of 15-37**, ≥3 doc surfaces surviving in each ✅ |
- [x] **NO REGRESSION** — `bash -n` clean on both edited shell files (⚠️ `shellcheck` is **not
  installed** — stated, not implied). ⛔ No `grammars/*.ebnf`, no `rust/src/*`, no `generated/*`
  touched ⇒ all generated parsers **byte-identical BY CONSTRUCTION**. The register is untouched, so
  no status claim moved. Oracles re-run:

  | oracle | result |
  |---|---|
  | `bash scripts/check_doctrines.sh` | **ALL 15 doctrines PASS** ✅ |
  | `scripts/check_readme_stability.sh` | **OK** — README 178/220 lines, 8 313/10 240 bytes (the 3 link re-points kept it under BOTH caps) ✅ |
  | `scripts/check_diagnostics_and_docpaths.sh` | **OK** ✅ |
  | `scripts/check_published_version_currency.sh` | **OK** — book snapshot 10/10 == register ✅ |
  | `make -C rust SHELL=/bin/bash mdbook_docs_gate` | **PASS** (all 10 per-parser books + top level) ✅ |
  | `make -C rust SHELL=/bin/bash ci_workflow_local_gate` | ⛔ **CANNOT COMPLETE — and it could not before this leaf either.** Not claimed. See the differential below |

  #### ⛔ The gate I most wanted is BLOCKED, so I measured what it would have measured

  `ci_workflow_local_gate` owns the 4 assertion sets this leaf edits, so it is the natural oracle.
  It **aborts in its audit phase on two PRE-EXISTING defects**, neither caused here:

  | # | blocker | evidence it is pre-existing |
  |---|---|---|
  | 1 | *"absolute PGEN checkout path found in markdown docs"* | the hit is `stimuli/generators/anvil/docs/tasks/LOCAL-REFERENCE-CACHE.md`, inside the **`anvil` SUBMODULE** (`git submodule status` → `ecda0e78`). `git diff --cached --name-only \| grep -c anvil` = **0** — this leaf's diff does not touch it |
  | 2 | `README.md` must contain *"`regex_corpus_bundle/`: PCRE2-first …"* | `git log -S` names `README-POLICY.1` (`d29c3dd7`) as the commit that REMOVED the line when the README was cut to a landing page. The assertion was never updated |

  ⭐ Blocker 1 is a **doctrine enforced twice, by two tools that disagree**:
  `check_diagnostics_and_docpaths.sh` uses `git grep` (submodule-blind ⇒ **OK**), while this gate uses
  `rg` (descends into the submodule ⇒ **FAIL**). Same rule, same tree, opposite verdicts — so one of
  them is wrong about scope, and PGEN cannot fix a file it does not own from here.

  ⇒ **Rather than claim a green I do not have, I built the differential the gate would have run.**
  Every `assert_file_contains` arm in the 4 edited functions, evaluated against the working tree AND
  against `HEAD` (the state before this leaf):

  | arms in the 4 edited audits | count |
  |---|---|
  | total after this leaf | **89** (93 before − the 4 retired tracker arms, exactly) |
  | PASS now | **87** |
  | ⛔ fail now but PASSED at `HEAD` ⇒ **caused by this leaf** | **0** ✅ |
  | ⚠️ fail now AND at `HEAD` ⇒ pre-existing | **2** — both the README arms above |

  ⇒ this leaf removed **exactly** the 4 arms it intended and regressed **nothing**. The driver is
  `rust/target/ci_audit_subset.sh` (untracked scratch — it sources the gate's own helper and audit
  function bodies rather than restating them, so it cannot test a rule the gate does not apply).
  ⚠️ **Honest bound**: this covers the audit phase for the 4 edited functions only, not the gate's
  replay phase. It is a substitute for a blocked oracle, not an equal of it.
- [x] **⚠️ A TRAP I WALKED INTO, AND THE REPORTING BUG THAT ALMOST HID IT.** Editing 6 per-parser
  book `.md` sources changed each book's mdBook **content-hashed search index**
  (`searchindex-<hash>.js`), and that HTML is **tracked**. So `ci_workflow_local_gate.sh`'s
  tracked-tree export died on `cp: cannot stat '…/searchindex-10339143.js'` — the tracked name no
  longer existed on disk. ⛔ **And I nearly missed it**: I had run the gate as
  `guard … > log; echo "rc=$?"`, so the reported code was `echo`'s, not the gate's — the log said
  `make: *** Error 1` / `memory-guard: completed exit=2` while the wrapper reported success.
  ⇒ **never read a pipeline's exit status from a trailing `echo`.** Fixed by co-committing the
  regenerated HTML (4 renamed search indexes + 116 modified pages), which is this repo's established
  convention — verified against a prior book-source commit's own file list rather than assumed.
- [x] **LOCKSTEP** — `COMMIT.md` (the `Files Involved` entry now names the register as the CLAIM and
  the book page as its gate-held VIEW), `docs/TASK_TREE.md`, `docs/TASK_TREE_README.md`, `README.md`,
  `SESSION_BOOTSTRAP.md`, `PGEN_USER_GUIDE.md`, `QUICKSTART_AI_ONBOARDING.md`, 11 book chapters, 5
  integration contracts, 6 per-parser book pages + their tracked HTML, `PGEN_README_STABILITY_POLICY.md`,
  the roadmap, `RUST_CODEBASE_ANALYSIS.md`, `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`.

#### ⚠️ Found while sweeping — ROUTED, not worked

`docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md:998` states *"the current measured sidecar now
computes `regex=Done`"*. The register claims `In Progress` and `DONE-BAR.2b` demoted it on
2026-07-29. ⇒ a **live false claim** on the roadmap. It blocks no gate and no published contract, so
per [[feedback_flow_findings_are_routed_not_worked]] it is routed to **`.4`** — whose own table
already names this roadmap as the worst self-refuting offender (~3.3 months). Only the file
reference was corrected here; the stale verdict was deliberately left for `.4` to adjudicate.

### `.1c3` — the delete (`todo`)

Delete the file, re-run the 452-ID census, and re-run the full doctrine enforcer + the done-bar
audit and probe driver to prove nothing went red.

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
