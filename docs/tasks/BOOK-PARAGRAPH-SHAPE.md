# BOOK-PARAGRAPH-SHAPE: the rendered book has walls of text — paragraphs are not stitched by mdBook, they are written that way

> **Director directive, 2026-08-01 (session #231), verbatim:** *"Make sure that paragraphs in the
> book are not stitched together as one big blob in sections or chapters, I have seen that a lot in
> some mdbooks. It is really painful to read like this. This visual defect appears of course in the
> rendered HTML version of the book."*
>
> **And, verbatim, on urgency:** *"The book defect we see sometimes is not urgent, I didn't want to
> forget about it that is why I mentioned it now."*
>
> ⇒ **ROUTED, NOT WORKED.** This tree exists so the finding survives the session. It is measured
> (below) so that whoever picks it up starts from evidence, not from a re-scan. Do **not** pull it
> ahead of product work → [[feedback_prefer_feature_work_over_governance_lanes]].

## Metadata

- Tree ID: `BOOK-PARAGRAPH-SHAPE`
- Status: `active` (opened 2026-08-01, session #231, by direct director report) — **frontier `.1`**
- Family / slice-id prefix: `PGEN-BOOK-PARAGRAPH-SHAPE-<NNNN>`
- Created: `2026-08-01`
- Owner: repo-local docs workflow
- Surface: `docs/book/` + the 10 per-parser books (`docs/<family>_parser_book/`), rendered to
  `docs/*-html/` by `mdbook_docs_gate` / the per-parser book gates.

## The measurement taken when the finding was reported (read this before designing anything)

Run over the **rendered HTML** (`<p>` blocks inside `<main>`, tags stripped, whitespace collapsed)
and then back over the **markdown source**, both at `341596cc`:

| surface | measure | value |
|---|---|---|
| rendered, all books, real pages (`print.html` excluded) | `<p>` blocks ≥ 1 200 chars | **167** |
| rendered, worst single `<p>` | `docs/systemverilog_parser_book-html/rules-top-level.html` | **19 146 chars** |
| rendered, worst in the main book | `docs/book-html/parseability-probe-debug.html` | **7 118 chars** |
| source, all book `src/*.md` | blank-line-separated paragraphs ≥ 1 200 chars | **181** |
| source, worst prose paragraph | `docs/book/src/parseability-probe-debug.md` | **7 255 chars** |

⭐ **THE DEFECT IS SOURCE-SIDE, NOT A RENDERING ARTIFACT.** The director's phrasing — *"stitched
together"* — describes the symptom exactly, but mdBook is not doing the stitching: CommonMark is
faithfully rendering paragraphs that were **written** as one block. The source and rendered numbers
track each other (181 vs 167; the worst main-book paragraph is 7 255 source / 7 118 rendered — the
delta is markup). ⇒ **no renderer/CSS/preprocessor fix can address this**; the fix is in the `.md`.

### The three sub-classes are NOT the same defect and must not be fixed the same way

1. ⭐⭐ **APPEND-ONLY STATUS BLOBS — one source LINE grown over dozens of slices.** The 19 146-char
   record holder is a single `> **Status:** …` blockquote line in
   `docs/systemverilog_parser_book/src/rules-top-level.md` that has been appended to once per slice
   from `SV-Slice-1` to `SV-Slice-59`. It is a **changelog living inside a reference chapter**, on
   one line. This is the **same disease** as `LIVE_ACHIEVEMENT_STATUS.md` reaching 1 547 057 B
   (`LIVE-MEANS-LIVE.1c3`) and the README growing unbounded (`README-POLICY.1`): an append-only
   surface with **no cap and no routing rule**. ⇒ the fix is *routing* (changelog → the book's
   changelog page / `CHANGES.md`), not reflowing.
2. **GENUINELY LONG PROSE PARAGRAPHS** — 181 source paragraphs ≥ 1 200 chars of real explanatory
   text (e.g. `parseability-probe-debug.md`, `grammar-wellformedness.md`,
   `compile-contract-validator.md`). ⇒ the fix is *editorial* — split at the argument's own seams,
   or promote the enumerated parts to a list/table. ⛔ Mechanical wrapping would be worse than the
   disease.
3. **LINE-STITCHING (consecutive source lines with no blank separator fusing into one `<p>`)** —
   **NOT YET OBSERVED.** The probe that looked for it found the worst candidate fusing from a single
   source line, i.e. class 1. ⛔ **Do not assume class 3 exists**; measure before designing for it.

### Reproduce (both surfaces, no build needed beyond the existing gate output)

```bash
make -C rust SHELL=/bin/bash mdbook_docs_gate          # refresh docs/*-html/
python3 - <<'EOF'
import re, glob, os
items=[]
for f in glob.glob('docs/*-html/**/*.html', recursive=True):
    if os.path.basename(f) == 'print.html': continue          # print.html concatenates every page
    html = open(f, encoding='utf-8', errors='replace').read()
    m = re.search(r'<main>(.*?)</main>', html, re.S)
    for p in re.findall(r'<p>(.*?)</p>', m.group(1) if m else html, re.S):
        t = re.sub(r'\s+', ' ', re.sub(r'<[^>]+>', '', p)).strip()
        if len(t) >= 1200: items.append((len(t), os.path.relpath(f), t[:80]))
items.sort(reverse=True)
print(f"rendered paragraphs >=1200 chars: {len(items)}")
for n, f, s in items[:10]: print(f"{n:>6}  {f}\n        {s}...")
EOF
```

## Why this tree exists (rather than a note in an existing tree)

The book is the **director's only window into the project** (session bootstrap §7): they review the
book, not the code. A reference chapter whose status section is a 19 000-character wall is not a
documentation-polish issue — it is the review surface failing at its one job. No existing tree owns
book *readability*: `LIVE-MEANS-LIVE` owns doc *currency* (is it still true), `README-POLICY` owns
`README.md`'s *size and routing*, and the book gates own *build success and link integrity*. Nothing
asks whether the rendered page can be read. That gap is exactly why this reached 19 146 characters
with every gate green.

## Leaves

### `.1` — decide the shape rule and where it is enforced (`todo`, ROUTED — do not pull ahead of product)

- **Not yet started.** The measurement above is the whole of the work done so far; nothing is
  designed and nothing is assumed.
- ⛔ **Prior art must be read first** → [[feedback_read_prior_art_before_designing]]: `README-POLICY`
  (a two-axis cap plus a routing table, and the measured lesson that a **line cap alone is
  bypassable** — layer-A `MEMORY.md` passed a 60-line cap at 138 403 bytes) and `LIVE-MEANS-LIVE.2`
  (the unwatched-overflow edge: a surface no instrument watches is where content goes to rot). Both
  say the same thing and it applies here: **cap the axis the content actually grows along**, and
  give the overflow a NAMED destination or the cap just relocates the problem.
- **The open questions, in order:**
  1. Is the enforced unit the **rendered `<p>`** or the **source paragraph**? Rendered is what the
     director sees; source is what an author edits and what a pre-commit hook can read without a
     build. They disagree by markup, and the disagreement is small (7 255 vs 7 118) — so this is a
     real choice, not a technicality.
  2. What threshold, and is it one threshold or per-class? Class 1 has no defensible threshold at
     all (a changelog does not belong in the chapter at any length); class 2 plausibly sits near
     1 000–1 500 chars. ⛔ **Pick it from the measured distribution, not from taste** — and publish
     the population it would flag, the way `.4`/`.7`/`.9` of `GENERATED-LINT-CORRECTNESS` priced
     every candidate before adopting it.
  3. Does it become an **18th doctrine**, a lane inside `mdbook_docs_gate`, or a report-only
     instrument with a ratchet? ⚠️ A hard cap adopted at a population of 167 would block on day one
     — so a **two-sided ratchet** (fail above the ceiling, and fail *below* it with "lower the
     ceiling", the `ebnf_frontend_dual_run_gate` pattern) is the shape that lets the debt be paid
     down without a flag day.
  4. ⛔ **Ground truth before any number is published** → [[feedback_instrument_needs_ground_truth]]:
     a positive control (a page that must pass) and a negative control (a planted 5 000-char
     paragraph the instrument must catch, exactly once) pinned inside the checker, refusing on a
     miss. The census in `GENERATED-LINT-CORRECTNESS` published headline numbers for two leaves with
     no ground truth; do not repeat it.
- **Explicitly NOT in scope for `.1`:** editing any prose. Measure and decide the rule first; the
  clean-up is `.2`+, and class 1 (routing a changelog out of a reference chapter) is separable from
  class 2 (editorial splitting) and should land first — it is the largest win and the least
  judgement.

## ROUTING EVIDENCE

⚠️ **First, honestly: the `ROUTING-EVIDENCE` enforcer flagged this file on the phrase *"class 1 …
is routed out of reference chapters"*, which is about routing CONTENT out of a chapter, not about
routing a FINDING to another tree.** That is a keyword over-match of the same shape as the
`\bwhy\b` one just fixed in `GENERATED-LINT-CORRECTNESS.9` — it fails CLOSED, so it is a nuisance
and not a soundness hole, and it is recorded here as an observation rather than acted on (the
director's standing preference is product over governance lanes). ⛔ Recorded, not waived: the
section below is written on its merits, because the placement question is real.

**The placement question that IS real: should this be its own tree, or a leaf of `README-POLICY` /
`LIVE-MEANS-LIVE`?**

1. **Does the finding reproduce outside the family it is being placed in?** ⭐ **Yes — measured, and
   that is the argument for a separate tree.** Rendered paragraphs ≥ 1 200 chars appear in **8 of
   the 11 built books**: main (178 in `print.html`, 7 118 worst on a real page), systemverilog (44 /
   19 146), regex (75 / 4 184), rtl_frontend (12 / 2 120), vhdl (4 / 1 949), svpp (18 / 1 861),
   rtl_const_expr (2 / 1 332), ebnf (0 / 1 162 — under threshold); clean: return_annotation,
   semantic_annotation, json. It is **not** an SV defect and **not** a main-book defect — it is a
   property of how every book in this repo is written. A leaf hung under a single family's tree
   would have mis-scoped it from the start.
2. **What was MEASURED to place it here, rather than what makes it plausible?** The source-vs-
   rendered comparison (181 vs 167, worst 7 255 source / 7 118 rendered) — that is what rules out
   `mdbook_docs_gate`'s build/link lane and any renderer-side owner, and rules **in** an authoring
   rule. And the 19 146-char case was traced to its source line before being classified: it is a
   `> **Status:**` blockquote appended to once per slice, i.e. class 1, not class 2. Had that not
   been traced, this tree would have been chartered as an editorial clean-up and would have missed
   the only sub-class with a structural fix.
3. **What would make this placement WRONG, and was it checked?** It would be wrong if the dominant
   population were `README.md`-shaped (one file, one cap ⇒ `README-POLICY`'s existing instrument
   already covers it) or currency-shaped (stale content ⇒ `LIVE-MEANS-LIVE`). **Checked, and it is
   neither:** the population is 167 paragraphs spread over 8 books and ~40 pages, and every one of
   them is *currently true* — the defect is shape, on a surface neither instrument watches. ⛔ It
   would ALSO be wrong if class 1 turned out to be the whole population, in which case this is
   simply `README-POLICY`'s append-only disease on a second surface and belongs there. **Not yet
   checked** — class 1 has one confirmed member (the 19 146-char line) and class 2 has 181, but the
   181 have not been re-read one by one to confirm none is a disguised changelog. `.1` must do that
   census before the tree's independence is treated as settled.

### `.2` — route class 1 out: the append-only `> **Status:**` blob (`todo`, PRIORITY-FIRST when this lane is unparked)

- ⭐ **DIRECTOR-DELEGATED DECISION, taken 2026-08-01** (*"the decision is your to make, you have see
  the problem, it is not urgent … task-tree own that HTML book's blob issue and continue on
  SV-EXH-PROOF-7.4.6.12"*). **The decision: do NOT interleave it with product work — but when this
  lane is unparked, `.2` goes FIRST, before `.1`.** That inverts the usual measure-then-act order
  deliberately, and the reason is that `.2` needs no rule: a per-slice changelog does not belong in
  a reference chapter **at any threshold**, so nothing `.1` decides can change `.2`'s verdict.
  Waiting for `.1` would hold the single largest win hostage to a policy question it does not
  depend on.
- **Scope:** the `> **Status:**` line in `docs/systemverilog_parser_book/src/rules-top-level.md`
  (19 146 chars, SV-Slice-1 → SV-Slice-59) moves to that book's changelog page; the chapter keeps a
  one-line pointer. ⛔ **ROUTED, NEVER DELETED** — the slice history is real content and the book's
  changelog page is its home, exactly as `README-POLICY.1` routed rather than trimmed.
- **Then close the loop so it cannot re-accumulate:** the same edit must make the *next* slice's
  author write to the changelog page instead of the chapter, or the line simply regrows. That is
  the `LIVE-MEANS-LIVE.2` unwatched-overflow lesson and it is the only part of `.2` that needs care.
- **Expected effect, to be re-measured not assumed:** worst rendered `<p>` on that page
  **19 146 → ~311** (the next-longest paragraph already in the file). The repo-wide worst then
  becomes the main book's 7 118, which is class 2 and belongs to `.1`.
- ⚠️ **Sweep first:** `.2` must check the other 10 books for the same append-only shape before
  declaring class 1 closed — one confirmed member is not a census (see ROUTING EVIDENCE §3, the
  condition that would make this tree's independence wrong and has not yet been checked).

## Acceptance Criteria (tree)

`the shape rule is chosen from the measured distribution with the flagged population published;
class 1 (append-only status blobs) is routed out of reference chapters and cannot re-accumulate;
class 2 is paid down under a two-sided ratchet so a regression fails and an improvement must be
banked; the instrument carries a positive AND a negative control and REFUSES on a miss; the book
gates stay green and no chapter loses information (routed, never deleted).`

promotion: declined (the measurement is a per-slice finding about this repo's book at one commit —
the durable, general lesson it instances is already in the retrievable layer as
`README-POLICY`'s two-axis-cap record and `LIVE-MEANS-LIVE.2`'s unwatched-overflow edge; re-promoting
it here would duplicate, not add. Promote from `.1` instead, once the shape rule is decided and there
is a re-verifiable claim to promote.)

## Evidence

- Measurement commands + numbers: this file, *"The measurement taken when the finding was reported"*.
- Taken at `341596cc` against the `docs/*-html/` output refreshed by `mdbook_docs_gate` (green).

## Commit log

| slice | leaf | commit subject |
|---|---|---|
| `PGEN-BOOK-PARAGRAPH-SHAPE-0001` | (tree opened) | the book's walls of text are written, not stitched — 167 rendered paragraphs ≥1200 chars, worst 19 146, and the worst is an append-only changelog on one line |
| `PGEN-BOOK-PARAGRAPH-SHAPE-0002` | `.2` (opened) | the director-delegated decision, taken: `.2` goes FIRST when unparked — a per-slice changelog does not belong in a reference chapter at any threshold, so it does not depend on `.1`'s rule |
