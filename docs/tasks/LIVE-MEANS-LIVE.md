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

### `.1b` — the family ROSTER: derive it from `grammars/*.ebnf`, not from the tracker (`todo`)

⭐ **The current derivation is WEAKER than its own docstring claims.** `audit_done_bar.sh:113-127`
builds the roster as *tracker rows ∩ `grammars/*.ebnf` basenames*, and the register's description
says this exists *"so a family nobody added to a list cannot hide"*. ⛔ **Measured, it hides
exactly one way**: a grammar with **no tracker row** is simply not a family, silently. The
refusal only fires in the other direction (on the tracker but absent from the register).

⇒ invert it: derive candidates from `grammars/*.ebnf` — the product itself, which cannot lie
about what exists — and REFUSE on any grammar with neither a register entry nor an explicit
recorded disposition. That is the `GATE-REACHABILITY` pattern (invoked, or a deliberate
disposition) applied to families.

⚠️ **Costed honestly**: 17 tracked grammars vs 7 register families, so 10 need an adjudicated
disposition (`ebnf` is the meta-grammar; `*_lrm_extracted` are inputs, not shipping families;
`builtin_*` are bootstrap contracts). That adjudication IS the work — it is not a rename.

### `.1c` — the book page, the 96 referrers, and the delete (`todo`)

The human at-a-glance view moves to an mdBook page (gate-checked against the register), all 96
tracked `.md` referrers are re-pointed, `check_readme_stability.sh:83` stops naming an uncapped
overflow destination, and `LIVE_ACHIEVEMENT_STATUS.md` is deleted. ✅ The ID census in `.1a`
already proves the delete is lossless (452/452).

### `.1` — (superseded — split into `.1a` ✅ / `.1b` / `.1c`)

⛔ **RESCUE BEFORE DELETE, in that order.** `PGEN-DEFAULT-PROFILE-0001`,
`PGEN-DEFAULT-PROFILE-0002` and `PGEN-REGEX-PCRE2-0030` are cited ONLY in this file; they go to
`CHANGES.md` first, verified present, and only then may the notes go. Re-run the ID census after
the purge and prove **452 / 452** still reachable from a durable layer.

⚠️ The purge is not `grep -v 'Tracker note'`: some notes carry a durable rule that layer C does
not yet record. Each is routed to `docs/decisions/` or confirmed already there before deletion.

### `.2` — the enforcer: `LIVE-STATUS-CURRENCY` (`superseded by .0` — only needed if the CHEAP path is chosen)

⛔ **`.0`'s revised answer makes this leaf conditional.** If the file is migrated and deleted,
there is no prose surface left to cap and this leaf is dropped entirely — you cannot leak a
changelog into a schema-bounded JSON field. Kept here, unworked, only as the fallback if the
director picks the cheaper purge-and-cap path.

#### (fallback design, if kept)

Mirror `README-STABILITY` onto this file — a line cap AND a byte cap (a line cap alone is
measurably bypassable: `README-POLICY.2` recorded layer A passing a 60-line cap while carrying
138 403 bytes), plus a **changelog-leakage tripwire** that fails on a dated `Tracker note`
entry, on `HISTORICAL` / `superseded` markers, and on prose that reports a *landed slice* rather
than a *current state*.

⚠️ **Sequencing**: `.2` lands AFTER `.1`, with the caps set at the achieved post-purge size.
Landing the enforcer first would fail every commit against a 1.5 MB file.

⭐ **Also close the redirect**: `check_readme_stability.sh:83` should not name an uncapped file
as an overflow destination. Once `.2` exists it is capped, and the README rule becomes honest.

### `.3` — audit the OTHER overflow destinations named by capped enforcers (`todo`)

`README-STABILITY` routes overflow to four places (`docs/book/src/gate-flow.md`,
`docs/book/src/operations-and-governance.md`, `docs/book/src/developer-architecture.md`,
`LIVE_ACHIEVEMENT_STATUS.md`). One of them was measured at 1.5 MB with no instrument. **Measure
the other three before assuming they are fine** — the failure mode is structural, not specific
to this file.

## Evidence

- Census commands are reproducible from the repo root; all figures above are at commit
  `3cb4b95b` and must be RE-MEASURED, not quoted, when `.1` is worked
  ([[feedback_read_prior_art_before_designing]]).
