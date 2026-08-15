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
- ⛔ ~~**No new enforcer is needed** — `.2` is dropped. Removing the unbounded container beats
  guarding it.~~ **SUPERSEDED by ANVIL's reopening of `.2`**: deleting the container answers *"is
  this file too big"*; it does not answer *"has any surface stopped being a status document"*, which
  is the question that was actually wanted. `LIVE-DOC-CURRENCY` landed 2026-07-31 — two baseline-free
  instruments plus **route closure over derived edges**, so the criterion is now *every overflow
  destination is watched*, not *the worst one is gone*.

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

### `.1c3` — the delete (`done`)

**`LIVE_ACHIEVEMENT_STATUS.md` is deleted.** Final measurement, taken the moment before:
**1 563 641 B / 1 684 lines / 856 `Tracker note (` entries.**

#### ⭐ The census was RE-MEASURED, not quoted — and the number had moved

`.1a` measured 452 slice IDs. Re-run at this leaf: **467**. The tracker kept accruing citations
between `.1a` and here, so quoting the old figure would have certified a delete against a
15-ID-smaller file than the one actually being removed. This is
[[feedback_read_prior_art_before_designing]] doing real work rather than ceremony.

| census | before the delete | after the delete |
|---|---|---|
| distinct `PGEN-<FAMILY>-<NNNN>` IDs the file cited | **467** | 467 (read from `git show HEAD:…`) |
| reachable from a durable layer WITHOUT it | **467 / 467 — 100.0 %** | **467 / 467 — 100.0 %** ✅ |
| ⛔ orphaned | **0** | **0** ✅ |
| (cross-check) also in git commit messages | 443 / 467 | — |

⇒ **nothing was lost.** 1 765 durable-layer files / 124.3 MB scanned, with the tracker excluded.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — a 1 563 641 B file, 94.7 % dated changelog, named LIVE. `.1a`/`.1b`
  removed every gate read, `.1c1` built the replacement, `.1c2` retired every dependent referrer;
  what remained was the file itself and **2 path-only referents that could not move before it**.
- [x] **ROOT CAUSE (WHY + WHERE)** — ops/build-flow tier. The two referents are not
  content readers, they are **existence** assertions, and one of them is an EXACT-SET comparison:

  ```
  $ awk 'NR>=310 && NR<=322' rust/scripts/ci_workflow_local_gate.sh
      actual_snapshot="$(printf '%s\n' "${actual_root_md[@]}")"     # <- git ls-files, root *.md
      if [[ "$actual_snapshot" != "$expected_snapshot" ]]; then
        fail "root markdown allowlist drift detected; …"
  ```

  **WHY that forces atomicity**: the roster is compared *verbatim* against `git ls-files`. Removing
  the entry while the file is tracked leaves `expected` one element short ⇒ FAIL; deleting the file
  while the entry stands leaves `actual` one short ⇒ FAIL. **Only the simultaneous change passes** —
  so `.1c2` deliberately left both behind rather than splitting them from the delete.
- [x] **FIX** — one commit: `git rm LIVE_ACHIEVEMENT_STATUS.md`, the roster entry removed (with an
  in-place comment recording *why* it had to move in this same commit), and the file dropped from
  `check_diagnostics_and_docpaths.sh`'s guarded pathspec.
- [x] **ADDRESSED (verified)** — measured before → after:

  | measurement | before | after |
  |---|---|---|
  | `LIVE_ACHIEVEMENT_STATUS.md` | 1 563 641 B / 1 684 lines / 856 tracker notes | **deleted** ✅ |
  | slice IDs reachable from a durable layer | 467/467 | **467/467** ✅ |
  | scripts that READ the path | 0 | **0** ✅ |
  | tracked references to the path | 14 files | 14 files — **every one a comment, docstring or JSON prose**, verified line by line ✅ |
  | `audit_root_markdown_surface` (exact-set) | PASS | **PASS** ✅ |
- [x] **THE ATOMICITY WAS REQUIRED — proven, not asserted.** A claim that two edits *had* to land
  together is worth nothing unless the alternative is shown to fail:

  | control | result |
  |---|---|
  | file deleted **and** roster entry removed (what shipped) | **PASS** — *"the exact-set roster matches `git ls-files`"* ✅ |
  | file deleted, roster entry KEPT (the split-commit alternative) | ⛔ **`fail`** — *"root markdown allowlist drift detected"* ✅ |

  ⇒ had `.1c2` retired this referent with the others, it would have shipped a red gate.
- [x] **NO REGRESSION** — `bash -n` clean on both edited shell files (⚠️ `shellcheck` **not
  installed** — stated, not implied). ⛔ No `grammars/*.ebnf`, no `rust/src/*`, no `generated/*`
  touched ⇒ all generated parsers **byte-identical BY CONSTRUCTION**. The register is untouched, so
  no status claim moved. Oracles re-run **after** the delete:

  | oracle | result |
  |---|---|
  | `bash scripts/check_doctrines.sh` | **ALL 15 doctrines PASS** ✅ |
  | `scripts/check_diagnostics_and_docpaths.sh` (its own surface list changed) | **OK** ✅ |
  | `scripts/check_published_version_currency.sh` | **OK** — book snapshot 10/10 == register ✅ |
  | `scripts/check_readme_stability.sh` | **OK** — 178/220 lines, 8 313/10 240 bytes ✅ |
  | `bash scripts/audit_done_bar.sh` | **exit 0** ✅ |
  | `run_done_bar_probes.sh` | **16/16** ✅ |
  | `run_published_version_currency_probes.sh` | **11/11** ✅ |
  | `make -C rust SHELL=/bin/bash mdbook_docs_gate` | **PASS**, and this time it left the tree clean ✅ |
  | `audit_root_markdown_surface` + its negative control | **PASS / correctly FAILS** ✅ |

  ⚠️ `ci_workflow_local_gate` end-to-end is still blocked by the two PRE-EXISTING defects routed to
  `CI-PARITY-GATE-ROT.20` — **unchanged by this leaf, and still not claimed.**
- [x] **LOCKSTEP** — `ci_workflow_local_gate.sh` roster comment, `check_diagnostics_and_docpaths.sh`
  header, `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `docs/TASK_TREE.md`.

⭐ **The tree's thesis, closed:** the file is not purged, it is **gone**, and what replaced it cannot
rot the same way — a schema-bounded JSON field for the claim, a gate-held book table for the view,
and an overflow rule that no longer points at an uncapped prose file.

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

### `.2` — the enforcer: NOT a byte cap. ANVIL's two instruments (`done` — `LIVE-DOC-CURRENCY` landed 2026-07-31)

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

#### ✅ WHAT LANDED — the `LIVE-DOC-CURRENCY` doctrine (2026-07-31)

| artifact | role |
|---|---|
| `scripts/check_live_document_currency.sh` | the enforcer (bash shim + embedded `python3`, the `check_flow_integrity.sh` house style) |
| `rust/test_data/grammar_quality/live_document_currency_register_v0.json` | the ONLY hand-written inputs: each surface's **charter**, and the two debt lists |
| `scripts/check_doctrines.sh` + `DOCTRINE_ENFORCEMENT.md` §10 | registry line + human mirror (the driver's meta-check holds them equal) |

**Instrument A — charter reconcile.** 28 surfaces, 23 chartered `status` (ceiling **20** distinct
dates), the rest `log` or `index`. A `log`/`index` charter is exempt from the ceiling but its `_why`
is **required** — an exemption nobody justified is an exemption nobody reviewed.

⚠️ **The ceiling is a human-chosen number and the leaf says so.** `.3` reported a 50× gap, but that
compared three book chapters against a 1.5 MB tracker. Re-measured over the *whole* registered set
the band is [12, 47] — highest healthy `status` surface 12 (`KNOWLEDGE_MAP.md`,
`RUST_CODEBASE_ANALYSIS.md`, the regex contract), lowest `log`/`index` 47 (`docs/decisions/INDEX.md`)
— because a **versioned** contract legitimately accumulates one date per release. 20 sits at 1.67×
above / 2.35× below. ⛔ Calling that "threshold-free" would have been the comfortable claim; it is a
2× band, not 50×, and the register carries the derivation.

**Instrument B — self-refutation, TOTALLY CLASSIFIED.** ⛔ The check does **not** enumerate
spellings — enumeration is exactly what failed twice. It requires every anchored `Last updated` line
to match a pinned shape and **REFUSES (exit 2)** on any it cannot classify, so a fifth spelling is a
loud refusal rather than an absent row.

**Route closure.** Every `.md` destination a capped enforcer NAMES must be a watched surface, and
the 27 edges are **DERIVED** — from `check_readme_stability.sh`'s `routing_hint()` heredoc and from
`COMMIT.md`'s own *Files Involved* list (globs expanded over tracked files). That answers `.5`'s
third finding: the edge that carried the rot was a **hint string inside an error message**, which no
hand-authored route registry can see.

#### ⭐⭐⭐ THE THIRD CONSECUTIVE MIS-MEASUREMENT — and why it stops here

| pass | population | knew | missed — always **silently, in the passing direction** |
|---|---:|---|---|
| `.4` | **10** | `Last updated: 2026-05-14` | ~50 `docs/tasks/` trees (`` - Last updated: `2026-05-31` ``) + 1 phantom prose row |
| `.5` | **16** | + the backtick spelling | the `docs/contracts/` **continuation** spelling (`- Last updated:` / date on the NEXT line): 6 files |
| `.2` | **18** | + continuation + template, fences and mid-line prose excluded | — (refuses instead) |

⭐ The two files `.5` missed are `PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` and
`PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md` — **published downstream contracts**, the surface class
where a stale claim costs the most and where `DONE-BAR.5a` already measured ~77 releases of drift.

⭐⭐ **The banked lesson**: three passes, three numbers, and every correction was possible only
because a **prior published number** existed to disagree with. An instrument's first output is a
hypothesis ([[feedback_instrument_needs_ground_truth]]). The structural fix is not a longer spelling
list — it is **refusing to publish over anything unclassified**, which is the one property a
spelling list can never have.

#### ⭐ The instrument REFUSED on its own task leaf — and that is how the last false-positive class was found

Writing the ADDRESSED box below meant quoting the enforcer's own OK line. The quotation wrapped, and
`Last updated:, 18 self-refuting, …` landed at the start of an **indented continuation line** —
which the first anchor (`^[ \t]*…`) read as a declaration with no parseable date, so the check
REFUSED on the very leaf documenting it.

⛔ **The cheap fix was to reword the leaf. That was refused**: it leaves the class live for the next
author, and a gate that authors route around is the failure mode `DOCTRINE_ENFORCEMENT.md` §6.1
names. The real fix is that **indented text is quoted material, not the document's own declaration**
— the same reason fenced blocks are excluded. The anchor moved to **column 0** (bare, or behind a
column-0 bullet), which is what all 61 real declarations measurably are.

Cross-checked rather than assumed: a loose-vs-tight differential over all 913 tracked `.md` files
reports **exactly one** line that column-0 anchoring drops, and it is the quotation artifact itself.
Population unchanged at 61 declaring / 18 self-refuting. Pinned by control
`excluded-indented-quotation`, and both directions re-probed — a **column-0** `Last updated: sometime
in the spring` still refuses (exit 2), the **indented** form does not (exit 0).

#### Ground truth — 9 controls, run BEFORE any number is published

In-memory fixtures through the **same** extractor the real scan uses (if the two could diverge, the
controls would prove nothing about the scan): a positive, a negative, one per pinned shape
(bare / backtick / continuation / template), a fenced-block exclusion (pinned on the real
`rust/docs/CLI_REFERENCE.md:161` shape), a mid-line-prose exclusion, an indented-quotation exclusion
(pinned on the shape that fired on this leaf), and one that proves the **refusal path itself is
live**. Any miss aborts with exit 2. No temp files — nothing is written outside the repository.

#### Acceptance Checklist (enforced)

- [x] **REPRODUCE / ISSUE** — `git ls-files -- '*.md'` census (913 tracked files) with the `.4`/`.5`
      extractors reproduced: the two published numbers (10, 16) disagree with each other and with a
      third reading, and no enforcer existed to hold any of them. `bash scripts/check_doctrines.sh`
      listed 15 doctrines, none watching whether a live document is currently *true*.
- [x] **ROOT CAUSE (WHY + WHERE)** — the `git ls-files -- '*.md'` census names both the mechanism and
      the location: the repo carries **four** `Last updated` shapes, not two — `bare` 14, `backtick`
      41, `continuation` 6 (`docs/contracts/`, the shape both prior passes missed), `template` 1 —
      plus one fenced false positive at `rust/docs/CLI_REFERENCE.md:161` and 4 mid-line prose
      matches. An extractor requiring a date immediately after the colon skips the continuation
      shape **as an absent row, not a reported miss**, so it under-reports and looks clean doing it.
      WHERE the doctrine gap was: `scripts/check_doctrines.sh` had no live-document instrument at
      all, and `scripts/check_readme_stability.sh`'s `routing_hint()` names destinations nothing
      watched.
- [x] **FIX** — declarative-tier: one enforcer + one hand-written register. No engine or grammar
      change. The register holds only what cannot be re-derived (charters, owned debt); the surface
      set, the date counts and all 27 route edges are re-derived from the tree on every run.
- [x] **ADDRESSED (verified)** — before: no instrument, three conflicting populations (10 / 16 / 18)
      and no way to tell which held. After: `bash scripts/check_live_document_currency.sh` → exit 0,
      `28 surfaces chartered (23 status, ceiling 20); instrument B: 61/913 tracked .md declare
      Last updated:, 18 self-refuting, all owned by LIVE-MEANS-LIVE.4; 27 derived route edges, all
      watched`. **All 12 probes proven to fire in the right direction** before the doctrine was
      trusted — 7 breaches (exit 1: new rot · paid debt · over-ceiling with no entry · paid
      `out_of_charter` entry · registered surface absent · charter with empty `_why` · unregistered
      route destination), 4 refusals (exit 2: a fifth spelling → *"matched NO pinned shape"* ·
      register missing · `routing_hint()` heredoc unlocatable · a perturbed extractor caught by
      control `shape-backtick`), and 1 **negative** probe proving the anchor fix holds in the other
      direction (the indented form of the refusing line → exit 0). `bash -n` clean on the enforcer
      and on the driver; `shellcheck` is NOT installed on this host, so that arm is unrun rather
      than claimed.
- [x] **NO REGRESSION** — `bash scripts/check_doctrines.sh` → **ALL 16 doctrines PASS** (15
      pre-existing + the new one), including `<meta:mirror>` confirming
      `DOCTRINE_ENFORCEMENT.md` §10 lists exactly the 16. Determinism: three consecutive runs are
      byte-identical (`md5 -q` of the verdict line, 3/3 equal) — no clock, no network, no randomness;
      every date compared comes out of the tree. Non-mutating: `git status --porcelain` after the 11
      probes shows only the three intended paths. Submodule-blind by construction
      (`git ls-files` without `--recurse-submodules`; **1 773** submodule `.md` files deliberately
      not scanned, per `CI-PARITY-GATE-ROT.20a` — submodules are READ-ONLY LINKED REPOS).
- [x] **LOCKSTEP** — `scripts/check_doctrines.sh` registry line, `DOCTRINE_ENFORCEMENT.md` §10 row
      (gate-held equal by the driver's meta-check), the register's own `_why` fields, book chapter
      *Operations and Governance* → *A live document must be currently TRUE, not merely bounded*,
      new layer-C record `docs/decisions/feedback_enumerating_instrument_must_refuse.md` + its
      `INDEX.md` row (`MEMORY-ARCH`'s bidirectional reconcile holds them equal), `docs/TASK_TREE.md`,
      `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`. `mdbook_docs_gate` GREEN (all 10 per-parser
      books + the main book).

#### ⚠️ Honest bounds — stated, not discovered later

- The `status` ceiling **is** a human-chosen number (20) in a **2×** band, not the 50× `.3` implied.
- Instrument A counts dates **including** fenced blocks (it asks *how many dates does this surface
  carry*, and it reproduces `.3`'s published table); instrument B **excludes** them (it asks whether
  a declaration contradicts content the file claims currency over). The asymmetry is deliberate.
- A `log`/`index` charter exempts a surface from instrument A **only**, and that is not a clean bill
  of health — `docs/TASK_TREE.md` is correctly chartered `index` and still carries a 64 450-byte
  table cell (`.6`).
- Route closure derives edges from **two** sources. A capped enforcer added tomorrow with its own
  routing hint is not covered until it is added to `derived_edge_sources`; there is no automatic
  discovery of hint blocks.
- A file whose prose legitimately cites a date newer than its declaration will read as
  self-refuting. That is literally true of the file and is left as a true positive rather than
  papered over with a heuristic.

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

### `.4` — the `Last updated:` field itself (`done` — `.4a` ✅ EXECUTED 2026-07-31, `.4b` ⚖️ ADJUDICATED; both answers were DELETE, so the rule was uniform)

⭐ Opened as *"10 tracked files self-refute their own `Last updated:`"*. The population was corrected twice (10 → 16 → **18**) and then re-framed entirely by a third measurement against `git`: the field is wrong in **25 of 61** files and was never once *ahead*, so the question is not which 18 dates to fix but whether a hand-maintained duplicate of a derivable fact should exist at all. Both correction blocks and the git table are kept below — they are the evidence the split rests on.

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
or to delete a self-declaration nothing maintains.

#### ⛔ CORRECTION — re-measured in `.5`: the population is **16**, not 10, and one row is phantom

Re-run over the whole tracked `.md` surface (`git ls-files -- '*.md'`, submodule-blind by
construction) while building `.5`'s outbound review. The table above is wrong in **both**
directions, and both errors come from the extractor, not from the files:

| correction | measured |
|---|---|
| ⛔ `CHANGES.md` does **not** declare `Last updated:` at all | its 3 matches are *prose* — L264 describes instrument B itself, L11012/L14479 narrate other files' fields. **The row is phantom**; the `2026-05-25` in it corresponds to no declaration in the file. Its advice ("may simply drop its `Last updated:`") is moot — there is nothing to drop |
| ⛔ **8 `docs/tasks/` files self-refute and were never listed** | `OPS-MEMSAFE` (2026-07-18→27), `REGEX-PCRE2-FIDELITY` (2026-06-07→07-30), `RGX-0090`/`RGX-0091` (2026-07-21→27), `SV-EXH-PROOF` (2026-05-31→06-10), `SV-PARSE-STRICT` (2026-06-09→06-10), `INLINE-ALT-FIX` (2026-05-16→17), `STIMULI-SIGNOFF` (2026-07-07→08) |
| ⇒ live population after `.1c3` deleted the tracker | **16** self-refuting of 61 declaring |

#### ⛔ SECOND CORRECTION — re-measured in `.2`: the population is **18**, and the same class of miss caused both

`.2`'s enforcer re-derives the population with an extractor that **refuses rather than skips**. Two
more files self-refute, and they were invisible to `.5` for the *same* reason `.4`'s 8 were invisible
to it — an unpinned declaration spelling:

| file | declares | newest content | spelling that hid it |
|---|---|---|---|
| `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` | 2026-07-04 | **2026-07-25** | `continuation` — `- Last updated:` with the date on the FOLLOWING line |
| `docs/contracts/PGEN_VHDL_PARSER_INTEGRATION_CONTRACT.md` | 2026-06-10 | **2026-07-22** | `continuation` |

⭐ Both are **published downstream integration contracts** — the surface class where a stale claim is
most expensive, and the class `DONE-BAR.5a` already caught ~77 releases stale. The other 4 contracts
using the same spelling are clean (declaration == newest), so the instrument is not firing on a class.

⇒ ⛔ **`.4`'s scope is 18 files, not 10 and not 16**, and its table above is the `.5` snapshot, kept
as the record of what each pass could see. The authoritative live list is
`instrument_b.self_refuting_debt.files` in
`rust/test_data/grammar_quality/live_document_currency_register_v0.json`, which `.2`'s enforcer holds
**two-sided**: a new self-refutation FAILS, and a fixed one still listed FAILS with *"remove it"*. ⇒
`.4` is now a **draining ratchet** rather than a static list that can rot on its own.

#### ⭐⭐⭐ THIRD MEASUREMENT — against `git`, the field has failed in **25 of 61** files, and instrument B sees only 18 of them

Before fixing 18 dates, `.4` asked the prior question the two earlier passes never did: **what does
the repository's own authoritative record say?** `git log -1 --date=short` is that record — it cannot
rot, it has no spelling problem, and it is available for every tracked file.

| population of 61 declaring files | measured 2026-07-31 |
|---|---:|
| declaration **==** git's last-commit date | 36 (59.0 %) |
| ⛔ declaration **older** than git — i.e. simply **wrong** | **25 (41.0 %)** |
| declaration **newer** than git | **0** |

And on the 18 instrument B flags, git is **≥ the newest content date in 18 of 18** (exactly equal in
10, newer in 8), while the declaration is correct in **0 of 18**.

⇒ Three conclusions, none of them a judgement call:

1. **The field is strictly dominated by `git`.** git is always at least as current, correct by
   construction, and free. The hand-maintained copy only ever lags — 0 of 61 was ever ahead.
2. **Instrument B is a LOWER BOUND, not the population.** It finds 18; git finds 25. The 7 it misses
   are stale declarations whose bodies happen to carry no newer date — e.g. `COMMIT.md`'s sibling
   `docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md` (declares 2026-07-21, git says
   2026-07-31). ⚠️ B is *self-refutation*, which is baseline-free and works on a foreign repo (that
   is how `.5` ran it on FSMGen's packet); *staleness against git* is a different, stronger question
   that only works at home. **Both are wanted; neither replaces the other.**
3. ⛔ **Correcting 25 dates is the fix that produced this.** They were correct once. A hand-maintained
   duplicate of a derivable fact is named as an anti-pattern by this repository's own standard —
   `MEMORY_ARCHITECTURE.md` §12: *"❌ Re-narrating git history into prose docs (duplication that goes
   stale)"* and *"❌ Hand-maintained current-state that drifts from reality (prefer derived)."* ⇒ the
   evidence `.4` asked for — *"a self-declaration nothing maintains"* — is now **measured, not
   asserted**, so the pre-authorized fix is **deletion**, not correction.

⚠️ The one class where deletion is NOT obviously right is the **published integration contracts**: a
downstream reader holds the `.md` without the repository, so `git` is not available to them. That
class is split out below rather than decided silently.

### `.4a` — execute the deletion across ALL 61 declaring files (`done` — 2026-07-31, `PGEN-LIVE-MEANS-LIVE-0014`)

> ✅ **APPROVED, EXPLICITLY, AFTER SEEING THE SCOPE.** Director, 2026-07-31, session #229, in
> response to a concrete before/after walkthrough of what the sweep touches (*"So want to delete all
> the files listed above?"* → answered *no files, 64 lines, 4 prefix-strips* → **"ok go-ahead then"**,
> to be executed after a session `/exit`).
>
> ⛔ **DO NOT RE-ASK.** The next session executes this leaf directly
> ([[feedback_routine_decisions_are_not_escalations]]). The scope the director approved is exactly
> the table below; anything wider is a new question.

#### ✅ EXECUTED — the approved scope, and what actually landed against it

| | approved | measured at execution |
|---|---:|---:|
| **files deleted** | **0** | **0** ✅ |
| declaration lines deleted (**2** lines each for the 6 contracts' continuation form) | **64** | **64** ✅ |
| lines **edited, not deleted** — prefix stripped, parenthetical KEPT (the 4 ⚠️ cases) | **4** | **4** ✅ |
| files touched | — | **62** (61 dated + `TEMPLATE.md`) |
| doubled blank lines collapsed (formatting residue, reported separately) | — | **13** |

The 13 blank-line collapses are not scope creep and are counted apart from the 64 deliberately: a
declaration that stood as its own *paragraph* leaves `\n\n\n` behind when the line goes. Only that
case collapses, and only by one line. Every other deletion was a bullet inside a metadata list,
where the list simply closes up.

⭐ **The sweep did not re-implement the extractor — it LIFTED the gate's own.**
`scripts/check_live_document_currency.sh`'s `scan()` was read out of the enforcer and executed, so
the sweep and the check could not disagree about what counts as a declaration. This matters
concretely: that extractor is the *fourth* attempt at this population, and the three before it each
missed a spelling silently. A hand-rolled `grep -v 'Last updated'` would have re-run that entire
error series, and its misses would again have been absent rows.

#### ⚠️ MY OWN PROCEDURE FAILED ONCE, and a pinned control is the only reason it was caught

While running the re-introduction controls, `git checkout -- docs/tasks/TEMPLATE.md` was used to
restore the perturbed file. But the sweep was **uncommitted**, so `checkout` restored the file from
`HEAD` — i.e. it silently **resurrected the deleted declaration** in that file. The gate did not
notice, because a single clean declaration is a legitimate state.

What caught it was a hash comparison pinned *before* the controls ran, which reported the file was
not byte-identical to its swept state. ⇒ the fix was to restore from a working-tree copy in `tmp/`
rather than from `HEAD`, and to re-run the sweep (which is idempotent and found exactly the 1
resurrected file). ⭐ This is [[feedback_instrument_needs_ground_truth]] paying for itself a fourth
time inside one tree, and the lesson generalizes past this leaf: **`git checkout --` is not a
restore in a dirty tree — it is a discard of uncommitted work.** The same mistake then discarded the
register edits a moment later, which the JSON re-read caught.

#### Acceptance Checklist (enforced)

- [x] **ROOT CAUSE (WHY + WHERE)** — the WHY is that the field is a hand-maintained duplicate of a
  fact `git` already owns, and the WHERE is all 62 files carrying it. Measured, not asserted, with
  the ops/build-flow toolbox: `git ls-files -- '*.md'` enumerated the 914-file tracked surface and
  the gate's lifted extractor classified every one of them, giving **61 declaring + 1 template**;
  `git log -1 --date=short -- <file>` supplied the non-rotting ground truth per file. Result:
  **25 of 61 declarations were older than git (41 %), 36 equal, and 0 were EVER ahead** — a
  strictly dominated field. On the 6 published contracts that carried it the stale rate is
  **6 of 6 (100 %)**. A repo-wide sweep of `scripts/`, `rust/scripts/`, `.githooks/`, `.github/`
  and `tools/` for consumers returned **zero** (the `last_updated` hits in `rust/src/test_registry.rs`
  are an unrelated RFC3339 struct field, checked rather than assumed).
- [x] **ADDRESSED (verified)** — before→after on the symptom, from the enforcer itself:
  - before: `instrument B: 61/914 tracked .md declare` … `, 18 self-refuting`
  - after: `instrument B: DORMANT — 0/914 tracked .md declare` … `Still WIRED as a re-introduction
    tripwire and its 9 ground-truth controls ran`
  - The four load-bearing prefix-strips are proven **lossless by reconstruction**: re-attaching the
    stripped prefix reproduces the original line byte for byte in all 4 files, including
    `SV-EXH-PROOF.md:9` whose 7 733-byte parenthetical names that tree's FRONTIER.
  - The tripwire is **proven to fire, not asserted** — 4 live controls, each with its exit code
    checked: re-introduced self-refuting declaration → **exit 1**; re-introduced *clean* declaration
    → **exit 0 and dormancy LIFTS** (so the branch is data-driven, not hardcoded); unclassifiable
    declaration → **exit 2, REFUSED**; re-listing a paid file in the drained debt list → **exit 1**.
- [x] **NO REGRESSION** — `scripts/check_doctrines.sh`: **all 16 doctrines PASS**.
  `mdbook_docs_gate` PASS, including all 10 per-parser book gates. The generated artifacts are
  untouched and **byte-identical**: `git diff --cached --name-only` carries no `grammars/`,
  `rust/src/`, `generated/` or `rust/test_data/ast_shape_contract/` path, and no generation target
  was run (deliberately — `CI-PARITY-GATE-ROT.19` has `regenerate_generated_parsers` failing on any
  warm tree). Instrument A's classification is unmoved: only 3 registered surfaces changed distinct-date
  count at all (`COMMIT.md` 4→3, the SVPP contract 4→3, the VHDL contract 3→2), every one of them far
  under the ceiling of 20, and the SV contract holds at 24 so its `out_of_charter` debt correctly stands.

#### ⛔ Two statements in the register were made FALSE by this leaf, and were corrected forward

Not found by a gate — found by asking what this sweep invalidates, which is the question this whole
tree exists to force:

1. The `out_of_charter` entry for the SV contract said *"instrument B independently flags the same
   file"*. After `.4a` it does not, because the declaration is gone. Left standing, the register
   would have kept a **"two independent instruments agree"** claim with only one instrument left —
   the precise defect class this tree was opened to remove. Reworded to past tense with the
   correction named.
2. Its `measured_bytes: 562756` was stale by the 33 bytes the two deleted lines occupied. Re-measured
   to **562 723** rather than carried forward. Instrument A's `measured_distinct_dates: 24` was
   re-measured too and is **unchanged** — `2026-07-04` occurs elsewhere in the file — so the entry
   stays and `.7` still owns it.

Nothing else in any file moves. The contracts keep their entire `## Contract Identity` block —
`Contract version`, `Parser release version`, the embedding-API baseline, the AST-dump schema
version — which is the stronger signal that made the date redundant in the first place.

Scope: **all 61** declaring files — `docs/tasks/*.md` trees + `docs/tasks/TEMPLATE.md`, the root
workflow/continuity docs, `docs/reference/*`, `docs/book/*`, **and the 6 `docs/contracts/*` published
contracts** (`.4b` ruled these the same way, on the measurement that the date is a weaker duplicate
of the version identity four lines above it). Evidence: the 25/61 table above, `MEMORY_ARCHITECTURE.md`
§12, and `.4b`'s three findings.

⚠️ **This is a repo-wide convention change and it touches the task-tree TEMPLATE**, so it is a
`TASKTREE-GOV`-adjacent decision, not a mechanical sweep — split out for that reason rather than
folded into `.4`. The two banked sweep traps (`LANG-CAPABILITY-AUDIT.10.3`, `CI-PARITY-GATE-ROT.19`)
were read first as required and **neither applies**: both govern *generated-artifact* sweeps, and
this one touches no grammar, no codegen and nothing under `generated/`. `.19` did still bind
negatively — it is why no regeneration target was invoked to "confirm" artifacts that were never in
the diff.

#### ✅ `.4b`'s four consequences — all landed in this commit, none deferred

| # | consequence | what landed |
|---|---|---|
| 1 | `docs/TASK_TREE.md` *mandates* the field | `Required Task File Sections` amended: the metadata list no longer names a last-updated date and now states ⛔ why, with the 25-of-61 measurement. `docs/tasks/TEMPLATE.md` lost the line too, so a new tree cannot re-add it by inheritance |
| 2 | ⚠️ the sweep trap — 4 load-bearing parentheticals | prefix stripped, parenthetical kept, **proven byte-identical by reconstruction** in all 4. `SV-EXH-PROOF.md:9`'s FRONTIER pointer survives intact |
| 3 | drain `instrument_b.self_refuting_debt.files` in the SAME commit | drained to `[]`. The paid-side ratchet fired on all 18 at once first — the ratchet working, verbatim as `.4b` predicted — and the list is kept as an **empty** structure, not deleted, because an empty two-sided ratchet is the tripwire's other half |
| 4 | instrument B must SAY it is dormant | the enforcer now branches at `declaring == 0` and prints `instrument B: DORMANT — 0/914 …  Still WIRED as a re-introduction tripwire and its 9 ground-truth controls ran`. Control 2 proves the branch is data-driven: one clean re-introduced declaration and the ordinary counting report returns |

⭐ **Why dormancy had to be spoken rather than merely true.** `0/914 declare, 0 self-refuting`
printed as a normal OK line is *indistinguishable* from an instrument that has silently stopped
seeing its subject — which is the exact failure this doctrine was built to catch, now pointed at
itself. The 9 ground-truth controls still execute on every invocation, so B is provably alive with
an empty population; that is what makes "dormant" an honest word here rather than a euphemism for
"switched off".

### `.4b` — ⚖️ ADJUDICATED (director delegated the call, 2026-07-31): the published contracts DELETE it too

> *"Take the most sota, signoff decision … You know the project's goals and objectives, so please
> make the right call."*

**RULING: delete. Same answer as `.4a`, so `.4` collapses back into ONE uniform rule with no
carve-out.** My own prior recommendation — *derive the field at publish time* — is **rejected**, and
the measurement that killed it is below.

#### ⭐ The generalizable rule this leaf produced

> **Generate FACTS, never generate CLAIMS.** For a derivable fact duplicated in prose, **delete the
> duplicate and point at the source** — unless the reader genuinely cannot reach that source *and*
> no stronger signal already exists beside it.

This is why "derive the field at publish time" was rejected rather than adopted: auto-populating
`Last updated:` would have *generated a claim*, manufacturing the appearance of currency on a
document nobody had re-verified — the same defect as the hand-written field, now harder to detect
because it would always look fresh. Deleting it and letting `Contract version:` carry the signal
keeps the document honest by construction.

*(Demoted here from `MEMORY.md` layer A by `SV-EXH-PROOF.7.4.6.8` — it had been recorded ONLY in the
bounded resume pointer, i.e. one cap-trim away from being lost. This leaf is its durable home.)*

#### The decisive measurement: the date is a weaker duplicate of a signal already 4 lines above it

Every contract that carries `Last updated:` carries it **inside a `## Contract Identity` block that
already declares stronger, actionable, versioned currency**:

```
## Contract Identity
- Contract version:                 <-- actionable: a consumer can diff versions
- Parser release version:
- Embedding API contract baseline:
- <family> AST-dump schema version:
- Last updated:                     <-- the weaker duplicate, 4 lines below
```

⇒ A date cannot answer the only question an integrator actually asks — *"does my integration still
hold?"*. A **version** can. The date is redundant against a signal that is present, stronger, and —
for regex — already gate-held by `PUBLISHED-VERSION-CURRENCY`.

#### Three findings that settle it, none of them a judgement call

1. ⭐ **10 of the 16 published contract documents already ship WITHOUT the field** — including
   `PGEN_PARSER_INTEGRATION_CONTRACTS.md` (the contract index), the issue-reporting protocol, the
   released-parser bug ledger, and the return-annotation / semantic-annotation integration
   contracts. Only **6** carry it. The premise of the whole question — *"the downstream reader's
   only currency signal"* — is **empirically false**: most consumers have been served without it and
   no gap has ever been reported.
2. ⛔ **Its measured failure direction is to UNDERSTATE liveness.** 0 of 61 declarations was ever
   *ahead* of git; 25 of 61 lag. A contract that says *"last updated 2026-07-04"* when the truth is
   2026-07-31 tells an evaluating adopter the project is staler than it is. For a surface whose job
   is to win downstream adoption, a field that systematically makes PGEN look abandoned is **worse
   than absent** — silence is neutral, a stale date is not.
3. ⛔ **Nothing reads it.** A repo-wide sweep of `scripts/`, `rust/scripts/`, `.githooks/` and
   `.github/` finds **zero** consumers other than `.2`'s own enforcer. It is unread duplication.

#### ⛔ CORRECTION to a number this leaf's own routing note published — and it strengthens the ruling

`.4`'s routing note (and `PGEN-LIVE-MEANS-LIVE-0010`'s changelog entry) said *"3 of 9 integration
contracts stale, 2 self-refuting"*. **The first figure is wrong.** Re-measured exhaustively over all
nine `PGEN_*_PARSER_INTEGRATION_CONTRACT.md` files:

| contract | declares | git says | verdict |
|---|---|---|---|
| `PGEN_PNR_…` | — | 2026-04-10 | **no field** |
| `PGEN_RETURN_ANNOTATION_…` | — | 2026-06-10 | **no field** |
| `PGEN_SEMANTIC_ANNOTATION_…` | — | 2026-06-10 | **no field** |
| `PGEN_REGEX_…` | 2026-07-21 | 2026-07-31 | ⛔ stale |
| `PGEN_RTL_CONST_EXPR_…` | 2026-05-16 | 2026-07-03 | ⛔ stale |
| `PGEN_RTL_FRONTEND_…` | 2026-06-15 | 2026-07-31 | ⛔ stale |
| `PGEN_SYSTEMVERILOG_…` | 2026-07-04 | 2026-07-31 | ⛔ stale **+ self-refutes** |
| `PGEN_SYSTEMVERILOG_PREPROCESSOR_…` | 2026-06-10 | 2026-07-31 | ⛔ stale |
| `PGEN_VHDL_…` | 2026-06-10 | 2026-07-31 | ⛔ stale **+ self-refutes** |

⇒ **6 of the 9 carry the field, and 6 of those 6 — 100 % — are stale against git.** Not 3 of 9.
The repo-wide rate is 41 %; on the *published* contracts, the surface where staleness costs the most,
it is **100 %**. ⛔ `CHANGES.md` is NOT rewritten — it is a record surface whose job is to be a record,
which is the distinction `.0` corrected. The correction is carried forward here and in the next
changelog entry.

⭐ This is the third time in this tree that a re-measurement moved a published number, and the third
time it was caught only because a prior number existed to disagree with
([[feedback_instrument_needs_ground_truth]]). It is also the first time the correction made the
argument *stronger* rather than merely more accurate — which is why the habit is worth its cost even
when the conclusion does not change.

#### Why "derive it at publish time" is rejected — it is this tree's own founding error, repeated

`.0` adjudicated the parent question and the answer generalizes: **removing the unbounded container
beats guarding it.** Building a generator + a publish step to keep a *redundant* field true is
guarding it — new machinery, permanently maintained, so that a weaker duplicate of an adjacent
signal can stay accurate. That is the same shape as capping `README.md` and routing its overflow
into an unwatched file: effort spent preserving the thing that should not exist.

⇒ **Removing the redundant field beats deriving it.**

#### ⭐⭐ The general rule this yields — GENERATE FACTS, NEVER GENERATE CLAIMS · and prefer DELETING a duplicated fact to generating it

`COMMIT.md` forbids auto-populating `claimed_status` from a gate: *"generating the claim makes the
comparison pass by construction."* That rule governs **judgements**, which a gate independently
recomputes from proof surfaces — it is one arm of a two-arm check.

`Last updated:` is **not a judgement**. It is a *fact* with exactly one source (`git`), and there is
no second, independent way to derive it — so generating it would destroy no comparison. The two-arm
principle does not apply, and reaching for it here would have been a category error in the opposite
direction from the one `.0` corrected.

But the right move for a **derivable fact duplicated in prose** is not to generate it either:

> **Delete the duplicate and point at the source — unless the reader provably cannot reach the
> source AND no stronger signal is already present.** Both escape clauses failed here: 10 of 16
> contracts already omit it, and the version identity sits four lines above.

#### ⛔ Consequences that MUST land with the sweep (each is a trap, not a chore)

1. **`docs/TASK_TREE.md` mandates the field** — *Required Task File Sections* → *"Metadata: tree ID,
   status, roadmap lane, created date, last updated date."* Deleting from trees without amending
   that line leaves the governance doc requiring what the trees no longer carry.
   `docs/tasks/TEMPLATE.md` must lose it too, or every new tree re-adds it.
2. ⚠️ **THE SWEEP TRAP — 4 declarations carry LOAD-BEARING trailing prose** that a line-wise delete
   would destroy. Measured, and one of them names a tree's **frontier**:

   | file | residue a naive delete would lose |
   |---|---|
   | `docs/tasks/SV-EXH-PROOF.md:9` | ``(**`.7` is the FRONTIER — close `focused_replay_target_debt_zero` …)`` |
   | `docs/tasks/TASKTREE-GOV.md:9` | `(ALL 4 LEAVES DONE — .1 inventory + .2 9 skeletons + …)` |
   | `docs/tasks/BRANCH-BROADCAST-FIX.md:9` | ``(`.5` done — H.10.2.1 re-applied and closed: regex cert-coverage …)`` |
   | `docs/tasks/MCP-CONTROL.md:11` | `(session #90 refinement — LIVE in-flight introspection + stall detection)` |

   ⇒ strip the `Last updated: <date>` **prefix** and keep the parenthetical; do not delete the line.
   `rust/docs/CLI_REFERENCE.md:161` is a fenced sample timestamp and is not a declaration at all.
3. **Drain `instrument_b.self_refuting_debt.files` in the same commit** — the register's two-sided
   ratchet FAILS on a paid entry that is still listed, which is the ratchet working.
4. ⚠️ **Instrument B goes DORMANT at zero declarations, and the enforcer must SAY so** rather than
   reporting a vacuous pass — the driver already states this principle about its own staged-scope
   doctrines. B stays wired as a **tripwire**: any declaration added later is checked from its first
   commit.

### `.8` — the version identity the contracts DO carry is gate-held for ONE family, not nine (`todo`)

Opened by `.4b`'s ruling. Deleting the date makes the `## Contract Identity` version block the
consumer's currency signal — and `PUBLISHED-VERSION-CURRENCY` currently holds only the **regex**
identity pair (user guide ↔ contract). The other 8 published contracts declare `Contract version`,
`Parser release version` and an AST-dump schema version that **no gate compares against the
repository's actual state**.

⚠️ This is not a regression caused by `.4b` — the gap exists today, and the date never checked
anything either. It is routed rather than worked ([[feedback_flow_findings_are_routed_not_worked]])
because it blocks nothing: no verdict becomes untrustworthy, and no published claim becomes false.

⛔ But it is the *load-bearing* half of the ruling: *"the version is the real signal"* is only true
while the version is true. `DONE-BAR.5a` measured this exact surface **~77 releases stale with no
gate reading it**, so the prior here is bad. Scope: extend `PUBLISHED-VERSION-CURRENCY` from one
family to every contract carrying an identity block.

⭐ **WHY the first pass missed them — the anchor, not the rule.** The repo has **two** declaration
spellings, and `.4` only ever saw one. Root docs write `Last updated: 2026-05-14`; every
`docs/tasks/` tree writes ``- Last updated: `2026-05-31` `` — **backtick-quoted**. An extractor
requiring a digit after the colon matches the first and silently skips ~50 files of the second.
⛔ And it fails *silently in the passing direction*: a missed file is not a reported miss, it is an
absent row, so the instrument under-reports and looks clean doing it.

⭐⭐ **The banked lesson, and it is this tree's own thesis pointed at its own instrument**: the
disagreement was only visible because a PRIOR measurement existed to disagree with. 10 vs 8 was the
signal; without `.4`'s number on the page, the corrected extractor's 16 would have been accepted as
fact on its first run. ⇒ [[feedback_instrument_needs_ground_truth]] — an instrument's first output
is a hypothesis, and the cheapest ground truth available is *the last time somebody measured it*.
⇒ **`.2` MUST pin both spellings and carry a control that fails if either stops matching.**

### `.5` — outbound review of FSMGen's *Live-Document Size Containment* external review packet (`done`)

Director request (2026-07-31, mid-session): *"could review this document and feedback your honest,
no BS opinion?"* — `/Volumes/SSD/Documents/github/fsmgen/docs/LIVE_DOCUMENT_SIZE_CONTAINMENT_EXTERNAL_REVIEW_PACKET.md`
(1 309 lines / 54 068 B, packet v1, snapshot `7f05b41d`).

⭐ **PGEN is the packet's own case history.** Its §*Origin of the problem* cites *"one measured
adopter"* whose status file *"reached 1,547,057 bytes, of which 94.7% was dated changelog"* — that
is byte-for-byte THIS tree's opening measurement of `LIVE_ACHIEVEMENT_STATUS.md`. ⇒ the review is
not an outside opinion, it is the **downstream report from the adopter the architecture was built
on**, and its most valuable payload is what happened next: we did not partition that file, we
**deleted** it (`.1c3`, 467/467 reachable, 0 orphans). That is the packet's own L10 — *"the doctrine
can bias toward preserving obsolete surfaces"* — confirmed empirically rather than suspected.

**Deliverable**: `docs/tasks/artifacts/live_means_live/fsmgen_live_document_size_containment_review.md`
(their requested response template; verdict **accept with changes**, 8 findings, 2 blocking-tier).

#### ⭐⭐⭐ The blocking finding — read the CHECKER, not the packet's prose

⛔ The packet's §*Immutable baseline* states the debt algebra as prose. Read as prose it looks
sound. **Read in the implementation it is a schema defect**, and the defect is already visible in
their committed data:

```
$ grep -n 'baseline .* exceeds' live-document-size/scripts/check_live_document_size.pl
492:  problem("surface $id baseline $baseline_fields[$index] exceeds $budget_fields[$index]")
494:  problem("surface $id transition baseline plus growth exceeds $budget_fields[$index]")
```

⇒ `baseline[d] <= budget[d]` is ENFORCED, so a surface already oversized at adoption can be
admitted as debt **only by setting its budget at or above its sick size**. Measured across all 20
records of `doctrine/live_document_size/surfaces.jsonl`:

| surface | state | `budgets.lines_each` | `baseline.lines_each` |
|---|---|---:|---:|
| `root_documents`, `engineering_rationale` | rollover_debt | **38 000** | 34 509 |
| `change_history` | rollover_debt | **35 000** | 31 799 |
| `fact_index` | structural_debt | **20 000** | 15 541 |
| `active_resume` (no legacy pressure) | normal | **60** | — |
| `enforced_rules` / `diagnostics` / `rationale` | normal | **300 / 400 / 512** | — |

⇒ **the doctrine that exists to bound live documents ships a registry declaring a 38 000-line
per-file hard limit** — a 633× spread inside ONE field, with health targets and quarantine ceilings
indistinguishable to the checker, the reader, and the next adopter who copies the file as a
template. Their L9 (*"seeing 'under hard limit' must not be read as 'well sized'"*) is a **prose
bandage on a schema defect**. Recommended fix: split `budget` (reviewed target, may only decrease)
from `ceiling` (= baseline + allowance, stop-growth), and make `ceiling` a **two-sided ratchet** —
PGEN's `envelope_divergence_ceiling()` shape, which fails above AND fails below with *"lower the
ceiling"*, so a legacy allowance cannot become permanent. That also dissolves their open L2/Q10/Q11
(`hard_pct` loses its job entirely).

#### The second finding — it bounds SIZE and ROUTING, never TRUTH

16 tracked PGEN files (the `.4` correction above) are inside every size bound, correctly routed,
coverage-complete — and self-refuting. All 13 of the packet's *"safety properties claimed"* hold for
every one of them. ⇒ *bounded* and *current* are independent, and only the first is mechanized.
Their L6 (*"semantic quality is not reducible to size"*) does not cover it: **self-refutation is not
semantic quality** — it needs no baseline, no threshold and no judgment.

#### ⭐ The third — route closure validates the routes you REMEMBERED

The packet records that PGEN's pressure *"had merely moved"* but not the **mechanism**, and the
mechanism is the finding: the edge that carried the rot was a **hint string inside an error
message** (`check_readme_stability.sh:83`, retired in `.1c2`). A hand-authored route registry cannot
see it. ⇒ answers their Q4 (*"can an unbounded sink still hide behind this graph?"*) — **yes, not
behind it, beside it** — and the fix is to derive candidate edges from the enforcers' own output
text, since any guard that names a destination is defining a route.

#### Verification (docs-only; no code change ⇒ no acceptance checklist per `COMMIT.md`)

| check | result |
|---|---|
| the packet's motivating figure IS ours | 1 547 057 B / 94.7 % — byte-identical to this tree's line 20-22 ✅ |
| blocking finding read from the IMPLEMENTATION, not the prose | `check_live_document_size.pl:492-495` quoted above ✅ |
| the 38 000 figure re-derived from their tracked data | all 20 `surfaces.jsonl` records parsed, table above ✅ |
| ⭐ both PGEN instruments run ON the packet itself (fairness control) | **1 distinct date, no self-refutation** — it practises what it argues, and the review says so ✅ |
| their 17-row evidence map spot-checked | 5/5 paths resolve ✅ (routed as an observation-tier finding: nothing CHECKS it) |
| cross-repo access | **read-only**, same volume (`/Volumes/SSD`), no write outside PGEN ✅ |

⚠️ **Honest bound**: the review judges the packet, the checker's debt algebra, and the surface
registry. It does **not** re-run FSMGen's test suite or exercise the task-tree sealing extension —
the closing recommendation about that extension is argued from PGEN's own delete-vs-partition
evidence, and is labelled in the deliverable as a peer caution rather than a finding.

⇒ Feeds `.2`: instruments A and B are now specified against a second project's architecture, and
the *"charter pairing is not a refinement, it is what makes the instrument usable"* point (their
Q19) is the same design constraint `.2` already identified for `CHANGES.md`.

### `.6` — ⛔ `docs/TASK_TREE.md` carries a **64 450-byte table cell**, and instrument A is BLIND to it (`todo`)

Found by `.2` while calibrating instrument A's charter for the task-tree index. The index scores
**62 distinct dates over 105 tree rows = 0.6 per row**, which is *index-shaped, not rot* — so
instrument A correctly does NOT fire, and the `index` charter is honest. The rot is on a **different
axis**:

| measured (2026-07-31) | `docs/TASK_TREE.md` | for comparison |
|---|---:|---|
| bytes | **468 401** | `docs/decisions/INDEX.md` 92 082 |
| lines | 454 | 153 |
| mean bytes/line | **1 032** | 602 |
| ⛔ single largest line | **64 450 B** (L108, the `RGX-0078` row) | 2 180 B |

⭐⭐ **The largest single line in the task-tree index is 1.68× the largest line of the 1.5 MB tracker
this whole tree was opened to delete** (38 265 B, `LIVE_ACHIEVEMENT_STATUS.md` L555). One table cell
is **13.8 %** of the file. The top five rows are 64 450 / 36 723 / 33 370 / 27 896 / 25 079 B — the
*Active Task Trees* table has become a place where each tree's changelog lives inside its own cell.

⚠️ **This is the `README-POLICY.2` bypass in a new location**: layer A once passed a 60-line cap
carrying 138 403 bytes at 2 306 B/line. Same shape, same reason — a line-count instrument cannot see
it, and here the *date-count* instrument cannot either.

⭐ **The general lesson this leaf exists to bank**: a charter that exempts a surface from one
instrument must never be read as a clean bill of health. `.2`'s register states that explicitly
(`_exemption_is_not_a_clean_bill_of_health`) and names this leaf, so the exemption carries its own
counter-evidence.

Scope when worked: decide whether the index rows keep only *status + frontier + pointer* (the tree
file already holds the detail), and whether a bytes-per-row instrument belongs beside A and B. ⛔ Do
NOT bulk-truncate rows — the content is reachable in each tree file, but that must be **proven per
row** first, exactly as `.1a`'s 452/452 census proved the tracker delete safe.

#### ⭐⭐ COST + RISK PRICED 2026-08-14 (`PGEN-README-POLICY-0013`, director-asked: *"what would it cost to change the format? is there any foreseeable risk?"*)

⛔ **First, ownership: `README-POLICY.11` was opened for this on 2026-08-14 and is a DUPLICATE of
this leaf.** `.11` is now a pointer here; this leaf remains the owner. The miss is worth naming — the
dedupe rule (`MEMORY_ARCHITECTURE.md` §10, *"before writing a fact, check the index for an existing
record and update it instead of forking a near-duplicate"*) was not run, and the thing that would
have surfaced `.6` in one grep is **the index whose unreadability is this leaf's subject**. The
defect obstructed the search for itself.

**RE-MEASURED AT HEAD — it has grown 29 % in 14 days:**

| | 2026-07-31 (`.6` as opened) | 2026-08-14 | Δ |
|---|---:|---:|---:|
| bytes | 468 401 | **604 021** | **+135 620 (+29 %)** |
| lines | 454 | 463 | +9 |
| largest single cell | 64 450 (`RGX-0078`) | **76 591** (`SV-CORPUS-GRAD`) | +12 141 |

⇒ **cost of delay ≈ +9.7 KB/day of prose that will have to be migrated.**

**ROOT CAUSE (new — `.6` recorded the symptom, not the mechanism).** Nobody authored a long line:
`awk 'length($0)>2000 {if ($0 ~ /^\|/) t++; else o++}'` → **43 table rows, 1 other**. A Markdown
table row *must* be one line, so length is an **emergent** property of (a) an index that is a table
× (b) a status cell used for accumulating narrative. **9.5 % of lines carry 87.2 % of the bytes**;
median line 64 B.

**COST — the mechanical coupling is ~ZERO, measured.** `git ls-files | xargs grep -ln TASK_TREE.md`
returns 28 files; **not one parses its content**:
- `scripts/check_memory_architecture.sh:101` — `[ -f docs/TASK_TREE.md ]`, existence only.
- `rust/scripts/ci_workflow_local_gate.sh:406` — a **sorted allowlist of root-markdown names**; the
  path does not change, so the entry does not either.
- `scripts/check_readme_stability.sh:89` — a hint **string inside an error message**.
- `live_document_currency_register_v0.json` — charters it `index`; instrument A already exempts it,
  and the exemption note already names THIS leaf.
- The remaining ~24 are prose links from the six harness bootstrap files, `README.md`, `COMMIT.md`,
  the book and `docs/decisions/` — all path links, unaffected by a format change.
⇒ **no gate, workflow, hook or Makefile reads a row.** The migration cannot break a consumer,
because there are no content consumers. Target shape `tree | status | frontier leaf | link` measures
**~6 453 B for all 110 rows — 1.1 % of today**, and reads in one agent read, which is the
operational success metric (`README-POLICY.11`'s framing, retained).

**RISK — the identifier half is essentially free; the PROSE half is the real risk and is UNPROVEN.**
Per-row census (the `.1a` / `MEMORY-ARCH.6` method, run before any migration):

```
IDENTIFIER REACHABILITY: 1038/1046 = 99.2% already reachable from the owning tree file / docs/decisions/
rows with >=1 unreachable identifier: 8 of 110   (trees with no owning tree file: 0)
```

All 8 adjudicated individually — **none is a migration risk, and two are pre-existing defects the
census FOUND**:
- **6 are slice ids** (`PGEN-BIN-BUILD-INTEGRITY-0006`, `PGEN-STIMULI-SIGNOFF-0016`,
  `PGEN-RGX-0078-0002`, `PGEN-GRAMMAR-WELLFORMED-0153`, `PGEN-LANG-CAPABILITY-AUDIT-0028`,
  `PGEN-LEXICAL-ANNOTATIONS-0005`) — each returns **exactly 1 commit** from `git log --grep`, i.e.
  they live in layer D and were never the index's to hold.
- **1 is a DANGLING wiki-link**: `INLINE-ACTIONS`' cell cites `[[project_inline_semantic_action_directives]]`
  and `docs/decisions/project_inline_semantic_action_directives.md` **does not exist**. Pre-existing,
  unrelated to format.
- **1 is `.26`**, cited by `ENGINE-UNIVERSAL-SERVICES` as `` `.25`/`.26` `` with no such leaf in the
  tree file — the **third instance** of the routing gap found at `README-POLICY.9` and `.10`: a leaf
  named in prose that its own tree does not define.

⛔⛔ **THE HONEST BOUND, AND IT IS THE WHOLE RISK.** Identifiers are only **28 % of the row bytes**;
strip every `[[link]]`, code-span and slice id and **395 425 B (72 %) of prose remains**, carrying
measured numbers and rationale that no identifier tracks. **A 99.2 % identifier census says nothing
about that 72 %.** ⇒ the go/no-go artifact is a **prose-side proof**, not this one: per row, show the
cell's substantive sentences already exist in the tree file, or MOVE them there first. That is `.6`'s
original *"proven per row, do NOT bulk-truncate"* instruction, now with the number that says how much
is at stake. Anything with no home gets written into its tree **before** the cell is cut, exactly as
`README-POLICY.2` wrote two homeless facts into `docs/decisions/` before trimming layer A.

#### ⭐⭐⭐ THE SPEC, FROM FIRST PRINCIPLES (director, 2026-08-14 — `PGEN-README-POLICY-0015`). ⛔ THIS SUPERSEDES THE OPTION SURVEY BELOW: the survey was shopping for a format before the requirement was written down.

Director, going back to basics: *"What is the function of `TASK_TREE.md`? What is its ultimate goal?
What is it supposed to do and do well? A new task-tree will increase its task-tree count by one, but
its byte size by B bytes and number of lines by L, right? Reading and writing shall be extremely
fast, regardless of its final file size."*

**That is not a question, it is the specification** — and it decides the design, which two rounds of
option-comparison could not.

##### 1. The FUNCTION — it is a DISPATCH TABLE, and it currently does three jobs

The file today fuses three things with different lifecycles, which is the same root error
`MEMORY_ARCHITECTURE.md` §3 names for the memory blob:

| job | lifecycle | belongs in |
|---|---|---|
| **(a)** define what a task-tree IS + the code-change doctrine | ~static | this file, or `TASK_TREE_README.md` |
| **(b)** enumerate every tree, its state, its frontier, its file | one row per tree | **this file — the only real job** |
| **(c)** narrate what happened inside each tree | grows per session | ⛔ `docs/tasks/<TREE>.md`, which already owns it |

**Ultimate goal:** *a fresh agent — any model, any harness, after any interruption — lands on the
correct leaf fast, and no tree can be silently lost.* Two properties it must do **well**:
- **COMPLETENESS** — every tree is enumerated. This is the anti-lost-work guarantee and the reason
  the file exists at all.
- **DISPATCH** — from a tree name to `{status, frontier leaf, file}` in one hop.

Everything else is somebody else's job: rationale → the tree file; history → `git` + `CHANGES.md`;
where-we-are-now → `MEMORY.md` (layer A). ⇒ job **(c)** is 87.2 % of the bytes and **0 % of the
function**.

##### 2. THE GROWTH LAW — measured, and it is the defect stated exactly

The director's model says a new tree costs **+1 tree, +B bytes, +L lines**, with B and L constants.
Measured against 110 rows:

```
correlation(row bytes, commits touching that tree) = 0.741      n = 110
median row 1 384 B   mean 4 997 B   max 75 137 B                (522x spread)
```

| tree | row at BIRTH | row NOW | factor |
|---|---:|---:|---:|
| `SV-CORPUS-GRAD` | 1 827 B | 76 592 B | **×41** |
| `ENGINE-UNIVERSAL-SERVICES` | 1 310 B | 39 107 B | **×29** |
| `README-POLICY` | 2 263 B | 9 591 B | ×4 |

⇒ **L IS already constant (1 line per tree). B IS NOT.** The schema is *right at birth and wrong at
maturity*: a newborn row is 1.3–2.3 KB and grows ×29–41 through ordinary work. The file's size is a
function of **WORK DONE**, not of **TREES TRACKED** — `O(trees × sessions)` where the spec demands
`O(trees)`. That single sentence is the defect, and it is what every earlier framing in this leaf was
circling without stating.

##### 3. THE TWO INVARIANTS the spec implies — and they discriminate PERFECTLY

- **I1 — BOUNDED COST PER UNIT.** Adding/working a tree costs a bounded constant. (the director's B, L)
- **I2 — LOCAL UPDATE.** Changing one tree touches only that tree's region. (the director's *"writing
  shall be extremely fast"* — and what makes `git diff` legible)

| surface | I1 bounded cost/unit | I2 local update |
|---|---|---|
| `docs/TASK_TREE.md` | ❌ r = 0.741 against work | ❌ 75 KB line rewritten for a one-word edit |
| `docs/decisions/INDEX.md` | ❌ rows reach 3 986 B | ✅ one line per record |
| `KNOWLEDGE_MAP.md` | ✅ derived from capped cards | ✅ max line 1 550 B |
| `*_register_v0.json` | ✅ JSON schema | ✅ 73–155 B/line |

⭐⭐ **This retro-corrects the survey below, and the correction matters.** It called
`KNOWLEDGE_MAP.md` a counterexample *because it is 533 KB* — **that was measuring the wrong thing.**
Under the actual spec, 533 KB over 94 capped cards is `O(units)` with a large constant: **compliant.**
Size was never the metric; **growth law and locality** are. `docs/TASK_TREE.md` is the only one of
the four that fails both, and it fails them for the same reason — job (c).

##### 3b. THE FORMAT — measured against five candidates, not chosen by taste (`PGEN-README-POLICY-0016`)

Director: *"What is the ultimate, super efficient, super fast, AI/LLM/agent-friendly format? To me
there should be NO PROSE in `TASK_TREE.md` — prose resides solely in the task-trees themselves. It
shall be super searchable, greppable, because that's how agents work."*

The real 110-row dataset was rendered into five candidate formats and measured.

**Finding 1 — SIZE IS A NON-DIFFERENTIATOR.** Every candidate lands in the same band, and all are
~50–100× better than today's **4 997 B/record mean**:

| candidate | B/record | lines/record |
|---|---:|---:|
| TSV | 57 | 1.0 |
| key=value line records | 86 | 1.0 |
| Markdown table | 97 | 1.0 |
| JSONL | 99 | 1.0 |
| YAML | 93 | **4.0** |

⇒ **the win comes from deleting job (c), not from picking a format.** Any of these is fine on size;
YAML is the only structural loser (4 lines/record breaks I2's *one record = one line*).

**Finding 2 — QUERY COUNT IS ALSO A NON-DIFFERENTIATOR.** Once prose is gone, every agent query is
one grep in every candidate (exact key lookup 1 hit; `active` filter 44/44 exact in all four). ⛔ Worth
stating plainly because it is the intuitive answer and it is **wrong**: greppability is bought by
removing prose, not by the syntax. Real tree-id substring collisions across all 110 ids: **1**
(`RGX-0087` ⊂ `RGX-0087-FIX2`) — so even a naive bare-key grep is nearly safe, and an anchored
`id=` pattern makes it exactly safe.

**Finding 3 — THE TWO AXES THAT DO DISCRIMINATE**, and they select the same winner:

- ⭐ **Is a single grep hit interpretable OUT OF CONTEXT?** This is the decisive agent property: grep
  returns **one line** into the model's context, usually without the header.
  - Markdown table / TSV → **POSITIONAL**; the hit means nothing without the header (a second read,
    or a fragile assumption).
  - `key=value` / JSONL → **carry their own field names**; the hit is self-contained.
- ⭐⭐ **Schema evolution fails SILENTLY in positional formats** — proven, not argued. Insert an
  `owner` column and ask a consumer for field 3:
  ```
  positional: printf 'X\tme\tactive\t.1\tf.md' | cut -f3   ->  active     (expected .1)
  named:      grep -o 'status=[a-z]*'                        ->  status=active  (unaffected)
  ```
  The positional reader returns a **plausible wrong value** with no error. That is the failure
  direction this repository fails on repeatedly, and it disqualifies TSV and the Markdown table for
  a surface meant to last.

##### 3c. ⇒ THE PROPOSED FORMAT — one self-describing record per line

```text
TREE id=SV-CORPUS-GRAD status=active frontier=.20 lane=sv file=docs/tasks/SV-CORPUS-GRAD.md
```

| field | constraint |
|---|---|
| `id=` | `[A-Z][A-Z0-9-]*`, unique, = the tree file basename |
| `status=` | **ENUM** — `active｜todo｜done｜blocked｜parked｜superseded` |
| `frontier=` | a leaf id (`.13i`) or `-` |
| `lane=` | short token |
| `file=` | repo-relative path |

⭐ **Why `key=value` over JSONL**, given both are self-describing: 86 vs 99 B/record (−13 %, and JSON
punctuation tokenizes worse than `=`), no quoting/escaping discipline to get wrong, and no `jq` on
the critical path — `grep 'status=active'` is already exact. JSONL is the better choice **if** a
typed validator or external tooling ever becomes the primary consumer; that is a reversible decision
and the field names are identical either way, so the migration between them is mechanical.

⭐⭐ **It makes COMPLETENESS — the file's whole reason to exist — MECHANICAL**, and this is the
strongest argument for it. Because `file=` names a real path, a checker reconciles **both
directions**: every `file=` must exist, and every `docs/tasks/*.md` must have a row. That is exactly
the layer-C index check `check_memory_architecture.sh` E2.5 already runs for `docs/decisions/`, which
was adopted after it passed at *135 records / 133 rows*. ⛔ Today no such check is possible at all,
because a "row" is 76 KB of prose with no parsable field.

⭐ **Rendering**: wrap the record block in a fenced code block. It stays valid Markdown for the book
and README links, renders as monospace, and `grep` ignores fences entirely.

⛔ **AND THE PROSE HAS A DESTINATION THAT ALREADY EXISTS.** The director's *"no prose"* rule also
evicts job (a) — the workflow definition + Code-Change Doctrine — which is **46 252 B (7 %)** of the
file, the non-table remainder. `docs/TASK_TREE_README.md` already exists at 8 758 B and is already
described as the reusable step-by-step guide. ⇒ job (a) → `TASK_TREE_README.md`, job (b) → the
record block, job (c) → the tree files. **Three jobs, three homes, no prose in the index.**

##### 4. WHAT THE SPEC REQUIRES (the design is now derived, not chosen)

1. **One row per tree, `L = 1`** — already true; keep it.
2. **A MECHANICALLY CAPPED row, `B` a constant** — the missing piece, and the thing every failed
   surface here lacks. `INDEX.md` proves that one-row-per-unit *without* a per-row bound still rots.
3. **Fields, not prose**: `tree | status | frontier leaf | link`. Rationale/narrative stay in the
   tree file, which already owns them.
4. **Detail by DISPATCH, never by inclusion** — the row answers *where to look*, never *what happened*.

Cost at `B = 300 B`: **110 trees → 33 KB · 250 → 73 KB · 1 000 → 293 KB** — one read at every scale,
so *"fast regardless of final file size"* holds with a linear law and a small constant. Sharding is
unnecessary and would be premature.

⭐ **And this RESOLVES the hand-vs-derived question the survey could not decide.** Both satisfy the
spec, so it is not the deciding axis: **derivation is an optimisation (it removes drift and is the
`KNOWLEDGE_MAP.md` pattern already proven here), the per-row cap is the REQUIREMENT.** Ship the cap
first — it is what makes the growth law hold — and derive afterwards if drift proves to be a real
cost. That ordering also lets the migration land without building any new machinery that could rot
(`GATE-REACHABILITY`).

⛔ **UNCHANGED BY THIS SPEC**: the 72 % prose migration is still the dominant risk and still needs a
per-row proof before any cell is cut. The spec says *what to build*; it does not make *getting there*
cheap.

---

#### ⛔ (SUPERSEDED BY THE SPEC ABOVE — retained for provenance) The "pointer table" recommendation is refuted by in-repo counterexample (2026-08-14, `PGEN-README-POLICY-0014`)

Director challenge: *"are you confident there is no other strong and compelling solution? We need
certainty it is the best, stable, sota, signoff, risk-free option, and the format we can use long
term."* — the honest answer is **no**, and the survey that should have preceded the recommendation
**refutes it**. This repository has already solved *"an index over many units"* four different ways,
and the outcomes are measurable:

| surface | bytes | max line | B/line | who writes it | what refuses growth | outcome |
|---|---:|---:|---:|---|---|---|
| `docs/TASK_TREE.md` | 606 509 | **76 591** | 1 309 | hand, free-form cell | **nothing** | ❌ this leaf |
| `docs/decisions/INDEX.md` | 125 813 | 3 986 | 727 | hand, one row per record, **bidirectionally reconciled** | nothing (reconcile ≠ size) | ⚠️ **126 KB anyway** |
| `KNOWLEDGE_MAP.md` | 533 060 | 1 550 | 286 | **DERIVED** + sync check | nothing | ⚠️ **533 KB anyway** |
| the 3 `*_register_v0.json` | 3 203 / 11 390 / 15 557 | — | 73–155 | hand, **JSON schema** | **the schema** | ✅ small and stable |

⛔ **`docs/decisions/INDEX.md` IS the pointer-table option, already running here.** One row per unit,
enforced in both directions by `check_memory_architecture.sh` E2.5 — and it has still reached
**125 813 B with a 3 986-byte line**. 19× better line shape than this file, and *the same trajectory*:
nothing prevents a "summary" cell from becoming a narrative cell. ⇒ **projection choice alone does
not bind.**

⛔ **`KNOWLEDGE_MAP.md` IS the derivation option, already running here** — generator
(`knowledge-map/scripts/gen_knowledge_map.sh`), `AUTO-GENERATED — DO NOT EDIT` banner, sync check in
the doctrine driver — and it is **533 060 B**, within 12 % of the file this leaf exists to fix. What
derivation *did* buy is real and it is not size: **max line 1 550 B vs 76 591 B**, and drift becomes
impossible by construction. ⇒ **derivation bounds SHAPE and DRIFT, never SIZE.**

⭐⭐ **The two refutations expose THREE orthogonal axes that the earlier framing collapsed into one
list of options** — this is the substantive correction:

| axis | question | controls | evidence |
|---|---|---|---|
| **1 — authorship** | hand vs derived | **drift**, write cost | `KNOWLEDGE_MAP.md` cannot drift; `INDEX.md` can |
| **2 — projection** | narrative vs pointer | **size**, read cost | `INDEX.md` projects pointers and is 126 KB |
| **3 — what REFUSES growth** | nothing / cap / **schema** | **long-term stability** | only the JSON registers stayed small |

**Axis 3 is the one the director's question is actually about, and it was never named.** The only
pattern in this repository that has stayed small over time is the **schema-bounded register**,
because a schema *refuses* free-form prose **by construction** — not by policy, not by a cap, not by
review. Every unbounded surface here eventually accreted narrative; no schema-bounded one did.

#### What the evidence actually supports (a COMPOSITE, and explicitly NOT "risk-free")

1. **Source of truth**: a **schema-bounded per-tree status block** — constrained fields (`tree id`,
   `status` **enum**, `frontier leaf`, `owner`, `updated`), prose forbidden.
2. **Index**: **DERIVED** from those blocks, `DO NOT EDIT` banner + sync check — the
   `KNOWLEDGE_MAP.md` pattern, already proven in this repo.
3. **Bound**: a **bytes-per-row / max-line instrument** beside A and B, because axes 1 and 2 were
   each measured *not* to bound size.

⛔ **THE DERIVATION SOURCE DOES NOT EXIST YET — this is the unpriced cost.** Measured over
`docs/tasks/*.md`: **84 of 161 files (52 %) carry a `## Metadata` block.** Where present the core
fields are strikingly consistent — `Status:` **84/84**, `Tree ID:` 83/84, `Created:` 81/84,
`Roadmap lane:` 76/84, `Owner:` 74/84 — so the schema is *discoverable rather than invented*. But it
is **not schema-bounded today**: Metadata block size runs to **19 114 B** (median 575 B) and **9 of
84 have a `Status:` line over 120 characters**, i.e. the same accretion has already started one
layer down. ⇒ the load-bearing work is *define the schema + backfill the trees*, not *write a
generator*.

⛔ **IT IS NOT RISK-FREE, AND CLAIMING SO WOULD BE THE DEFECT THIS TREE EXISTS TO CATCH.** Named risks:
- the generator + sync check are **new machinery that can itself rot** — `GATE-REACHABILITY`'s
  founding subject; it must be registered in the doctrine driver **in the same commit that lands it**
  (`TOOLBOX.md`'s new-instrument rule);
- the **72 % prose migration is unchanged** and remains the dominant risk (see the census above);
- a schema too loose reproduces the disease — **measured, at 19 114 B, inside the Metadata blocks**;
  a schema too tight pushes authors to write prose somewhere unbounded instead;
- backfilling ~110 trees is a large mechanical change whose own review cost is real.

⇒ **NOT READY FOR A DECISION.** The next step is a priced design slice comparing the composite
against (a) cap+demote, (b) hand-maintained pointer table, (c) router+archive split, (d) derived
index, (e) schema-bounded register + derived view — scored on the three axes above with the measured
outcomes already in hand. Picking now would be *"a design built on unmeasured premises"*, which is
the failure `ENGINE-UNIVERSAL-SERVICES.17` is a record of.

**Recommended sequencing when this is worked** (not executed here — this leaf stays `todo`):
1. per-row prose census + move (the expensive, load-bearing step; ~110 rows, and it only grows),
2. rewrite the table to `tree | status | frontier | link`,
3. re-run the identifier census as a regression check (expect ≥ 1038/1046, minus the 2 defects fixed),
4. add the bytes-per-row instrument beside A and B so it cannot recur,
5. fix the two by-product defects above (dangling record, undefined `.26`).


### `.7` — the published SystemVerilog integration contract has accumulated a version log (`todo`)

Found by `.2` on the doctrine's **first run**, by **both** instruments independently:

| instrument | reading on `docs/contracts/PGEN_SYSTEMVERILOG_PARSER_INTEGRATION_CONTRACT.md` |
|---|---|
| **A** (distinct dates vs charter) | **24** — over the `status` ceiling of 20; the file is 562 723 B |
| **B** (self-refutation) | ⚠️ **no longer applies** — it declared `2026-07-04` against its own newest content date of `2026-07-25`, but `.4a` deleted the declaration from this file along with every other |

⭐ Two baseline-free instruments agreeing on one surface was the strongest signal either could give,
and this is the surface class where a stale claim costs the most: a **published downstream
contract**. `PUBLISHED-VERSION-CURRENCY` already exists because this exact class was measured **~77
releases stale with no gate reading either document** (`DONE-BAR.5a`) — that guard holds the regex
identity pair and the family-status row, not the contract's own version history.

⛔ **`.4a` did NOT weaken this leaf, and the distinction matters.** B's agreement is gone because
its *subject* was deleted repo-wide, not because anything here improved. **A's finding is entirely
untouched**: the per-release version log is still inside a document chartered as the CURRENT
contract, still 24 distinct dates over a ceiling of 20 (re-measured after the sweep — `2026-07-04`
occurs elsewhere in the file, so the count did not move). The 33-byte drop is the two deleted
declaration lines and nothing else.

⚠️ It is *registered debt, not a waiver*: `instrument_a.out_of_charter` names this leaf as owner,
and the entry is **two-sided** — paying the debt without removing the entry fails the doctrine too.

Scope when worked: route the per-release history out of the contract body (the `CHANGES.md` /
per-family ledger question), then remove the `out_of_charter` entry. (The `self_refuting_debt` row
this leaf also used to carry was drained by `.4a`, along with all 18.) ⛔ The ceiling is NEVER raised
to make this green.

## Evidence

- Census commands are reproducible from the repo root; all figures above are at commit
  `3cb4b95b` and must be RE-MEASURED, not quoted, when `.1` is worked
  ([[feedback_read_prior_art_before_designing]]).
- `.5`'s figures are at FSMGen snapshot `7f05b41d` and were re-derived from that repository's
  tracked data, not quoted from its packet — which is how the blocking finding was found at all.
