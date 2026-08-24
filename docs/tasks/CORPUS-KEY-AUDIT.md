# CORPUS-KEY-AUDIT: the answer key is an instrument, and nothing was auditing it

## Metadata

- Tree ID: `CORPUS-KEY-AUDIT`
- Status: `active` (opened 2026-08-24, **by director approval** of the finding below)
- Family / slice-id prefix: `PGEN-CORPUS-KEY-AUDIT-<NNNN>`
- Created: `2026-08-24`
- Owner: repo-local corpus-oracle workflow
- **Frontier: `.2`** (the key-PROVENANCE census). ✅ `.1` is CLOSED — (a)–(d) done and (e) wired
  as doctrine `CORPUS-KEY-INTEGRITY`

## ⛔ THE DIRECTOR'S APPROVAL (2026-08-24)

Shown the session result that auditing EXPECTATIONS out-yielded burning down the RESIDUAL:

> *"I am ok with your key-auditing suggestion … if this can make PGEN even more sota and
> signoff."*

⇒ the lane is chartered, and the condition is the bar: **every leaf here must make a
published number more trustworthy, not merely produce more work.**

## ⭐ THE PROVENANCE (measured, not asserted — all from session 2026-08-23/24)

| fact | evidence |
|---|---|
| the key manufactured a defect | `br_gh552.v` and `real_invalid_ops.v` use the **same** operator (`~&`) and carried **opposite** expectations; the difference was an `iverilog` flag the key never read (`SV-CORPUS-GRAD.13e.3`(a), `PGEN-SV-CORPUS-GRAD-0285`) |
| the key hid THREE real defects | of 394 ivtest `CE` rows, 84 carry a usable golden and only **6** say the words *"syntax error"* — the sole test applied. Enumerating what the compiler actually says surfaced `br1027a/c/e` as genuine over-acceptances (`.13e.3`(b), `-0286`), fixed as ledger `SV-0068` (`.13e.5`, `-0287`) |
| a wrong `must_accept` is CAMOUFLAGED | a row keyed `must_accept` that the parser accepts reads `match` — the strongest verdict in the file. All three `br1027*` rows sat at `match` for the whole campaign |
| the yield ratio | in one session, auditing expectations **found or corrected 5** defects while residual burn-down **fixed 3** |

⛔ **THE ASYMMETRY IS THE WHOLE ARGUMENT.** Residual burn-down only ever looks at rows already
flagged. An expectation error in the *accept* direction produces no flag at all — it produces a
`match`. So the population this lane audits is, by construction, invisible to the lane that
precedes it.

## Goal

Make the SV (and later every family's) corpus **answer key** a checked instrument rather than a
trusted one, so that the published defect bar means what it says: every `must_accept` /
`must_reject` is traceable to a clause cite or to upstream evidence that has been shown capable of
being wrong, and disagreements inside the corpus are surfaced mechanically instead of by accident.

## Non-goals

- ⛔ **Not a second burn-down.** This tree does not fix parser defects; it routes them to the
  owning family tree (`SV-CORPUS-GRAD` today) with a clause cite. Finding work is not the product.
- ⛔ **Not a re-adjudication of every row by hand.** The corpus is 16 336 rows. Every leaf here
  must be a *mechanical* audit with a stated false-positive rate, not a reading marathon.
- ⛔ **Not a replacement for `SV-CORPUS-DENOMINATOR`.** That doctrine re-derives the published
  NUMBERS from the manifests and says so in its own header (*"it does not re-adjudicate the
  corpus"*). This tree is the layer underneath it.

## Acceptance criteria

1. Every expectation in the SV manifests is classified by the KIND of evidence that produced it —
   clause cite, upstream tool testimony, or suite convention — and the population of each is
   published and re-derivable.
2. A mechanical contradiction check runs over the corpus and its false-positive rate is measured
   and stated, not assumed.
3. The audits that earn it are wired to a gate, so the key cannot silently rot the way the outcome
   files did (`SV-CORPUS-GRAD.13e.6`).
4. Every defect this lane surfaces is ROUTED to an owning leaf with a clause cite — never reported
   and left.

## Task tree

### `.1` — the CONTRADICTION census: do two rows keyed from the same evidence disagree? (`done` 2026-08-24, gate-held by `CORPUS-KEY-INTEGRITY`)

**The question is mechanical and nobody was asking it.** A corpus large enough to contain a
contradiction is an oracle you already own: group keyed rows by the upstream messages their key
reads, and flag any message class whose rows disagree about `must_accept` vs `must_reject`.
`.13e.3` found exactly such a pair BY HAND, and only because one of them happened to be
adjudicated for an unrelated reason.

**LANDED AND MEASURED** — `docs/tasks/artifacts/corpus_key_audit/key_contradiction_census.py`:

```text
KEY-CONTRADICTION-CENSUS: rows=499 messages=168 contradictory_classes=2
```

⛔⛔ **AND IT OVER-REPORTS BY CONSTRUCTION — SAYING SO IS HALF ITS VALUE.** A row carries ONE
verdict; its golden may carry SEVERAL messages. Attributing every message to the row's verdict
manufactures a disagreement whenever a file was pinned for a reason unrelated to most of what its
golden says — and **both** reported classes are measurably that shape: `br_gh1087b` is pinned for a
NET DATA TYPE (A.2.1.3) while its golden *also* reports a multiple-driver **elaboration** error, and
`pr1704726a/c/d` are pinned for a bare generate block (A.4.2) while their goldens *also* report a
duplicate declaration. ⇒ **the output is a candidate worklist, never a defect count** — the same
lesson `.13e.3`(b) learned when a coarse text filter was 10/11 false positives and only the parser
could tell.

#### `.1`(a)–(d) — DONE 2026-08-24 (`PGEN-CORPUS-KEY-AUDIT-0002`)

⛔⛔ **(a) THE PLANNED FIX WAS THE WRONG FIX, AND MEASURING IT IS WHAT SHOWED THAT.** The leaf
owed *"attribute a row to the message the parser's own `furthest_position` lands on"*. Two
independent reasons that plan fails, both measured before any code was written:

1. **It does not reach the actual candidates.** `br_gh1087b` is pinned at **line 3**
   (`wire bool [7:0] b;`) and the only message in its golden sits at **line 6**. Positional
   narrowing finds nothing to attribute and falls back to the whole golden — the false positive
   survives.
2. **It is circular.** Asking the parser under test which message keyed its own answer key is not
   evidence about either.

⭐ **THE KEY ALREADY RECORDS ITS OWN DECIDING EVIDENCE — IN THE BASIS STRING.** Three shapes, all
mechanical, none needing a parse:

| provenance | what the basis says | attribution | rows |
|---|---|---|---|
| `clause-cited` | names an IEEE clause / Annex A production / an LRM doc path | **no** golden message — the clause decided it | 9 |
| `quoted-decider` | quotes it verbatim: `PARSE-stage refusal ('…')` | only the messages containing that fragment | 11 |
| `whole-golden:accept-claim` | a `must_accept` key | every message — the claim IS that each is post-parse | 479 |

`pr1704726a`'s basis states the ranking in its own words: *"spec outranks the ivtest driver key"*.

⛔⛔ **AND THE EXTRACTOR ITSELF WAS BLIND — THE SHARPER HALF.** The founding `MSG` regex required a
literal `error:`/`sorry:` tag, but iverilog emits its bare parse refusal untagged:
`./ivltests/br_gh79.v:6: syntax error`. **WHY+WHERE:** `MSG` at
`docs/tasks/artifacts/corpus_key_audit/key_contradiction_census.py:52`. So the ONE class that most
directly answers a parse-stage question was **absent from the vocabulary the founding
`messages=168` was measured over** — together with `Net data type requires SystemVerilog or
-gxtypes.`, the dialect-gate class `SV-CORPUS-GRAD.13e.3`(a) had just caught the key mis-reading.
Corrected: **176** classes (**+8, none dropped**). It also repaired the integrity check itself —
6 of 11 quoted deciders were unresolvable against the old vocabulary and would have been reported
as key defects that are really instrument blindness (**hit 5/miss 6 → hit 11/miss 0**).

**(b) FALSE-POSITIVE RATE, BEFORE→AFTER — measured in the same run, never carried:**

```text
KEY-CONTRADICTION-CENSUS: rows=499 messages=176 contradictory_classes=0 (naive=2)
                          key_integrity_findings=0
```

⇒ the founding report's false-positive rate is **2 of 2 = 100 %**. The artifact keeps the naive
table beside the refined one so the correction is auditable rather than asserted.

**(c) NOTHING SURVIVES TO ROUTE.** 0 contradictions, 0 key-integrity findings. No `SV-CORPUS-GRAD`
row is owed.

⛔ **(c′) HONEST POWER BOUND — PUBLISHED, BECAUSE `0` READS FAR STRONGER WITHOUT IT.** A
contradiction needs a class on BOTH sides, so the census reaches only the intersection: after
attribution the reject side is **4 classes** wide against **124** on the accept side.

| reject-side deciding class | `must_reject` rows | `must_accept` rows carrying it |
|---|---|---|
| `Missing task/function port direction.` | 3 | 0 |
| `X X  is not a valid expression. Please use operator X instead.` | 1 | 0 |
| `generate/endgenerate regions cannot nest.` | 1 | 0 |
| `syntax error` | 6 | 0 |

⭐ The last row is the one worth having: a `must_accept` row whose golden carried the upstream's own
bare `syntax error` would be a near-certain key defect, and the founding extractor could not see the
class at all — the question was **unaskable, not answered**. It is now asked over all 479 accept
rows and the answer is **0**.

**(d) DECIDED — YES, cheap enough to gate: 0.242 s** (`time`, whole census) plus a 5-arm self-test.
The manifests are edited by nearly every SV burn-down slice, so the population this watches moves
constantly. ⇒ `.1`(e) below owns the wiring; the decision is not left as a note
(`CI-PARITY-GATE-ROT.44` is the standing example of a tracked instrument no gate invokes).

### `.1`(e) — DONE 2026-08-24 (`PGEN-CORPUS-KEY-AUDIT-0003`): doctrine `CORPUS-KEY-INTEGRITY`

`scripts/check_corpus_key_integrity.sh`, registered as the repository's **27th** doctrine. ⇒ the
durability leg `.1`(a)–(d) had to NAME as open is now satisfied: **the claim is watched.**

⭐⭐ **TWO TIERS, BECAUSE THEY NEED DIFFERENT THINGS PRESENT** — and A1 is the one that matters most
here, because `.1`(a) proved the *census itself* can be the wrong half:

- **A1 INSTRUMENT ALIVE** (always, corpus-INDEPENDENT, ~0.1 s) — runs the census's own
  `--self-test`. It refuses three ways a naive check would pass: a **shrunken** arm set (coverage
  lost silently), a **zero-arm** run reporting `failed=0` (a control that cannot fail), and an
  **unparsable** summary (a reader that treats what it cannot read as *no findings* fails in the
  passing direction — the exact family this lane exists to catch).
- **A2/A3 KEY CLEAN + ARTIFACT CURRENT** (corpus-dependent) — zero contradictory classes, zero bases
  quoting evidence their golden lacks, a **non-empty population** (*a clean verdict over zero rows is
  not a clean key*), and the tracked `census.md` **byte-identical** to a fresh derivation. A3 exists
  because `SV-CORPUS-DENOMINATOR` was founded on a derived artifact measured stale one day after
  landing.

⛔ **NOT EVALUATED, not blocked, on a corpus-less clone.** The vendored corpora are submodules, so
A2/A3 announce themselves loudly and A1 still binds — the posture `GRAMMAR-CERT-CURRENCY` takes for
the untracked `generated/` tree. ⭐ That branch is **exercised, not assumed**: a self-test arm
relocates a copy of the census so its own `parents[4]` repo-root walk lands on no corpus and requires
exit 2. Otherwise the fresh-clone path would be the one branch of the gate nobody had ever seen run.

⛔⛔ **BUILDING THE GATE FOUND TWO LIVE DEFECTS THAT ONLY A REAL CONSUMER COULD EXPOSE.** Both were
in code that had already been committed and measured green:

1. **The census crashed on its own documented `--md` option** whenever the path lay outside the repo
   — `args.md.relative_to(ROOT)` raises `ValueError`. The failure is the nastiest shape: the summary
   line had already printed and the file had already been written, so the *work succeeded* and the
   *exit code said failure*. Nothing had ever passed an out-of-repo `--md` because nothing had ever
   driven the census as a gate does. Fixed with a `try/except` that falls back to the plain path.
2. **The enforcer's first cut put its scratch file in `$TMPDIR`**, violating the same-volume data
   policy by construction on any host where `TMPDIR` is off the repo volume. Now repo-derived
   (`rust/target/`, git-ignored) — the choice every other self-test here already makes.

**MEASURED:**

```text
bash scripts/check_corpus_key_integrity.sh --self-test   -> self-test 13 passed, 0 failed
bash scripts/check_corpus_key_integrity.sh               -> OK, 0.311 s, tree unmutated
bash scripts/check_doctrines.sh                          -> ALL 27 enforced doctrines PASS
```

**RED CONTROL THROUGH THE REAL DRIVER, not through the enforcer alone**: appending one line to the
tracked `census.md` turns `scripts/check_doctrines.sh` to
`✗ FAIL CORPUS-KEY-INTEGRITY … commit/merge blocked`; restoring the file returns it to
`ALL 27 enforced doctrines PASS`. A doctrine proven only by its own `--self-test` has not been shown
to reach the surface that actually blocks a commit.

⚠️ **HONEST BOUND, carried from `.1`(c′) into the doctrine's own registry text**: a green A2 is
**not** *"the key is correct"*. The census's power is the intersection of its two sides — 4
reject-side deciding classes against 124 accept-side — and the artifact publishes that ratio beside
its zero. `.2`/`.3` are what widen it.

## Acceptance Checklist (enforced) — leaf `.1`, both landings

### `.1`(a)–(d) — `PGEN-CORPUS-KEY-AUDIT-0002`

- [x] **REPRODUCE / ISSUE** — `python3 docs/tasks/artifacts/corpus_key_audit/key_contradiction_census.py`
  at parent `b551244d` → `rows=499 messages=168 contradictory_classes=2`; both classes hand-shown
  to be multi-message artifacts, i.e. a 2-of-2 false-positive report.
- [x] **ROOT CAUSE (WHY + WHERE)** — TWO, both tool-measured. **(1) attribution:** the census
  attributed every golden message to the row's single verdict; `grep` of the four candidate rows'
  `basis` fields shows `br_gh1087b`/`pr1704726a|c|d` are **clause-cited** (`IEEE 1364-2005 A.2.1.3`,
  `A.4.2`) and never read their goldens — WHERE: the attribution loop in
  `key_contradiction_census.py`. **(2) extractor:** `MSG` at `key_contradiction_census.py:52`
  required an `error:`/`sorry:` tag; `cat …/gold/br_gh79*.gold` shows iverilog's untagged
  `./ivltests/br_gh79.v:6: syntax error`, so `syntax error` was outside the measured vocabulary —
  proven by a differential run of the old vs corrected regex over the same 499 rows
  (`OLD=168 NEW=176 adds=8 drops=0`).
- [x] **FIX** — declarative tier (no grammar, no Rust, no codegen, zero generated bytes):
  `deciding_messages()` reads the row's own recorded provenance; `MSG` accepts the untagged form
  while still excluding `warning:` lines and `:      : It was declared here …` continuations.
- [x] **ADDRESSED (verified)** — `contradictory_classes 2 → 0` (`naive=2` retained in-run as the
  before number), `messages 168 → 176`, quoted-decider resolution `hit 5/miss 6 → hit 11/miss 0`,
  `key_integrity_findings=0`.
- [x] **NO REGRESSION** — ZERO grammar / Rust / codegen / generated bytes touched
  (`git diff --stat`: two files under `docs/tasks/`, one under `docs/tasks/artifacts/`).
  `--self-test` 5/5 arms green, including the arm that proves the detector goes RED and the arm
  that proves the clause exclusion — not luck — is what turns it green. `bash scripts/check_doctrines.sh`
  GREEN.
- [x] **LOCKSTEP** — book (`docs/book/src/`), `CHANGES.md`, `DEVELOPMENT_NOTES.md`, `MEMORY.md`
  updated. No contract / ledger / AST-schema surface touched: this leaf changes no parser
  behaviour and no published family status.

### `.1`(e) — `PGEN-CORPUS-KEY-AUDIT-0003`

- [x] **REPRODUCE / ISSUE** — `.1`(a)–(d) had to publish verification leg 3 (durability) as
  NAMED-not-satisfied because nothing invoked the census. `CI-PARITY-GATE-ROT.44` is the standing
  example of the same shape: a tracked instrument no gate runs.
- [x] **ROOT CAUSE (WHY + WHERE)** — not a parse defect: a **coverage gap in the enforcement layer**,
  measured at parent `631c9f74` with the ops/build-flow family:

  ```bash
  git ls-files 'scripts/check_*.sh' 'rust/Makefile*' '.github/workflows/*' \
    | xargs grep -l key_contradiction_census | wc -l   # -> 0
  git ls-files 'scripts/check_*.sh' | wc -l            # -> 26 enforcers, none of them this one
  ```

  **WHY:** the `DOCTRINES` registry in `scripts/check_doctrines.sh` is the only automatic lane, and
  it had no row for the corpus answer key. **WHERE:** `scripts/check_doctrines.sh`, the `DOCTRINES`
  array. ⭐ Wiring it then exposed **two live code defects only a real consumer could surface**,
  both tool-measured: (1) `python3 …/key_contradiction_census.py --md <path-outside-repo>` →
  `ValueError: '…' is not in the subpath of '…/pgen'` from `args.md.relative_to(ROOT)` at the
  reporting line — raised *after* the summary had printed and the file had been written, i.e.
  **work succeeded, exit code said failure**; (2) the enforcer's first cut wrote its scratch file
  under `$TMPDIR`, off the repository volume by construction on any host whose `TMPDIR` is
  elsewhere. `bash -n scripts/check_corpus_key_integrity.sh` clean throughout, which is why neither
  was a syntax-level catch.
- [x] **FIX** — declarative/instrument tier, no grammar / Rust / codegen / generated bytes: a new
  `scripts/check_corpus_key_integrity.sh` (A1 corpus-independent instrument check; A2/A3 key +
  published-artifact check), one registry row, the `DOCTRINE_ENFORCEMENT.md` §10 mirror row, the
  book's two marked `DOCTRINE-COUNT` sites, plus the two defects above (`try/except` on the
  reported path; repo-derived `rust/target/` scratch).
- [x] **ADDRESSED (verified)** — `bash scripts/check_corpus_key_integrity.sh --self-test` →
  **13 passed, 0 failed**; the doctrine itself → `OK` in **0.311 s** with `git status --porcelain`
  showing the gate mutated nothing; `bash scripts/check_doctrines.sh` → **ALL 27 enforced doctrines
  PASS** (26 → 27). ⭐ **RED CONTROL THROUGH THE REAL DRIVER**, not through the enforcer alone:
  appending one line to the tracked `census.md` turns the driver to
  `✗ FAIL CORPUS-KEY-INTEGRITY … commit/merge blocked`, and restoring it returns
  `ALL 27 enforced doctrines PASS`.
- [x] **NO REGRESSION** — all 27 doctrines PASS including the two meta-checks that would catch a
  half-registration (`<meta:mirror>` = `DOCTRINE_ENFORCEMENT.md` §10 lists exactly the 27;
  `<meta:book-count>` = `gate-flow.md` publishes 27 at 2 marked sites). ZERO grammar / Rust /
  codegen / generated bytes. `mdbook_docs_gate` PASS. The census's own output is byte-identical
  before and after the `--md` fix on the in-repo path (`contradictory_classes=0`,
  `key_integrity_findings=0`), so the repair touched the reporting line only.
- [x] **LOCKSTEP** — `DOCTRINE_ENFORCEMENT.md` §10, `docs/book/src/gate-flow.md` (count) and
  `docs/book/src/grammar-wellformedness.md` (the doctrine's own paragraph), `CHANGES.md`,
  `DEVELOPMENT_NOTES.md`, `MEMORY.md`, `docs/TASK_TREE.md`. No contract / ledger / AST-schema
  surface: no parser behaviour and no published family status changed.

### `.2` — the key-PROVENANCE census: which expectations rest on a CLAUSE, and which on TOOL TESTIMONY? (`todo`)

Every basis string already names its evidence. Sweep both manifests and classify each expectation:
**clause-cited** (a pin naming an Annex A / clause line), **tool testimony** (an upstream driver key
or golden log), or **suite convention**. ⛔ Only the second class can be wrong in the way `.13e.3`
found twice, so its size IS the trust bound on the published bar — and today that number is
unpublished. Publish it beside the bar.

### `.3` — the CAMOUFLAGED-match audit: over-acceptances hide inside `match` (`todo`)

⛔ **The direction the corpus structurally cannot report.** A wrong `must_accept` on a file the
parser accepts reads `match`. `SV-0068`'s three rows sat there for the whole campaign, and
`SV-CORPUS-GRAD.13e.4` was found only because a rejected file was read line by line for another
reason. Sample `match` rows whose text carries a construct on the known non-derivable list (the
`V2005_LRM_PINNED` clause set is the seed) and measure the hit rate. If it is non-zero, the bar
under-reports and this lane says by how much.

### `.4` — generalize the parse-stage vocabulary beyond ivtest and verilator (`todo`)

`.3.24` gave verilator a parse-stage message vocabulary; `.13e.3`(b) gave ivtest one, **split by
edition** because several of its refusals are dialect gates. sv2v, Surelog and ispras have had
neither treatment. ⛔ Do NOT copy the flat shape: `.13e.3`(b) measured that an unsplit list would
have manufactured five false defects.

## Current frontier

1. `.2` — the provenance census (cheap, and it produces the trust bound the bar is missing).
   ⭐ `.1`(a) already built and measured the classifier over the 499 golden-mapped rows
   (`clause-cited` 9 / `quoted-decider` 11 / `whole-golden` 479); `.2` widens it to all **3 134**
   keyed iverilog rows — the 2 635 with no golden are exactly the ones whose provenance is
   currently unmeasured.
2. `.3`, `.4`.

## Decisions

- **2026-08-24 — the lane is a TREE, not a leaf under `SV-CORPUS-GRAD`.** The technique is
  family-agnostic (every family gets an external corpus with an answer key under
  `CORPUS-GRAD-ALL`), while `SV-CORPUS-GRAD` owns one family's burn-down. Keeping it separate is
  what lets `.4` reach sv2v and Surelog without widening an SV leaf.
- **2026-08-24 — the founding number ships with its own instrument.** `.1`'s census is tracked and
  re-runnable from the first commit, because a lane premised on *"published numbers must be
  re-derivable"* cannot open with a hand-counted one.
- **2026-08-24 — over-reporting is DECLARED, not discovered later.** The census artifact states its
  own false-positive mechanism in its header, so a reader cannot mistake the worklist for a count.
- **2026-08-24 (`.1`(a)) — the deciding evidence is read from the KEY, never from the PARSER.** The
  leaf had planned to use the parser's `furthest_position`. Measurement killed it twice: it does not
  reach the candidates (`br_gh1087b` is pinned at line 3, its only golden message is at line 6) and
  it is circular. The basis string already records what the key read; that is the authority.
- **2026-08-24 (`.1`(e)) — A1 checks the CENSUS before A2 trusts it, and that ordering is the
  doctrine's point.** `.1`(a) had just proved the census can be the wrong half of the comparison, so
  a gate that read only its verdict would gate on a broken oracle. A1 is also the tier that survives
  a corpus-less clone, which makes the instrument's health the part that is checked *most* often.
- **2026-08-24 (`.1`(e)) — a corpus-less clone gets NOT EVALUATED, never a pass and never a block.**
  `BASELINE-IDENTITY` already learned the other way round: its first cut made a moved input a hard
  failure inside a pre-commit enforcer and one comment line blocked every commit. A doctrine that
  cannot see its subject must say so, loudly, and let the tiers that can still bind.
- **2026-08-24 (`.1`(c′)) — the census publishes its own POWER BOUND beside its result.** `0
  contradictions` over a 4-class reject side is a much weaker statement than `0` alone implies, and
  the artifact says so in the same table. A clean number whose reach is unstated is the
  flattering-direction failure this tree exists to catch.

## Open questions

- ⭐ `.1`(a) measured the reject side of the contradiction check at **4 classes**. Widening it is
  not free: it means more `must_reject` rows carrying a *quoted* decider, which is a re-keying
  campaign, not an instrument change. Is that worth a leaf, or is the accept-side `syntax error`
  sweep (0 of 479) already the check that carries the weight? Currently unresolved — sized here so
  it is not silently answered by inaction.

- Is `.3` affordable? The `match` population is ~5 845 rows on the SV lane alone; the audit must be
  a sampled or construct-keyed sweep, and the sampling design is the leaf's first job.
- Should `.2`'s trust bound be published beside the bar (a co-publication like
  `SV-CORPUS-DENOMINATOR`'s tuple), or reported on demand? Co-publication is the stronger claim and
  the more expensive one.

## Blockers

None. `.1` is closed and gate-held; `.2` is unblocked and needs no new tooling.

## Verification log

- **2026-08-24, `.1` instrument** — `python3 docs/tasks/artifacts/corpus_key_audit/key_contradiction_census.py`
  → `rows=499 messages=168 contradictory_classes=2`, artifact written. Both reported classes
  hand-adjudicated the same day as multi-message artifacts (see `.1` above), so the honest reading
  today is **0 confirmed key contradictions and 2 candidates awaiting deciding-message
  attribution** — stated this way because the reverse reading would inflate the lane's own founding
  number, which is the failure mode this tree exists to catch.
- **2026-08-24, `.1`(a)–(d)** — three-way verification per the standing directive:
  1. **RE-DERIVE** — `… key_contradiction_census.py` →
     `rows=499 messages=176 contradictory_classes=0 (naive=2) key_integrity_findings=0`, artifact
     rewritten. The reject-side per-class counts (3 / 1 / 1 / 6, accept-side hits 0 / 0 / 0 / 0)
     were produced **independently** by an ad-hoc probe importing the module before the report
     table existed, and agree exactly. ⛔ They did NOT agree on the first render: the table read the
     *contradictory subset* rather than the full attribution and printed `0` on both sides for
     every class — a power-bound table silently claiming the census had no reach at all. Caught by
     comparing the rendered artifact against the independent probe, which is the only reason a
     second derivation is worth running.
  2. **FALSIFY** — `… key_contradiction_census.py --self-test` → `arms=5 failed=0`. The arms are
     fully synthetic, so they run with the corpora absent and cannot be quietly satisfied by
     whatever the manifests happen to contain: arm 1 proves the detector **goes RED**; arm 2 proves
     the *clause exclusion* is what turns it green (same rows, clause cite added → 1 → 0); arm 3
     proves a quoted decider narrows; arm 4 proves an unresolvable quote is REPORTED and does not
     silence its row; arm 5 pins the untagged `syntax error` in the vocabulary. The external oracle
     is the vendored iverilog goldens, which this lane does not author.
  3. **DURABILITY** — producer TRACKED (`key_contradiction_census.py`, `census.md`, both in git).
     ⛔ **Claim NOT YET WATCHED at the time of that commit — named rather than published
     unqualified**: no gate invoked the census. ✅ **Closed by `.1`(e) the same day** (below).
- **2026-08-24, `.1`(e)** — three-way verification of the DOCTRINE:
  1. **RE-DERIVE** — `bash scripts/check_corpus_key_integrity.sh` → `OK` in **0.311 s**, and
     `git status --porcelain` unchanged across the run, so the gate's *mutates nothing* contract is
     measured rather than asserted. `bash scripts/check_doctrines.sh` → **ALL 27 enforced doctrines
     PASS**, both meta-checks included.
  2. **FALSIFY** — `--self-test` → **13 passed, 0 failed**, every arm a distinct refusal path
     (shrunken arm set / zero-arm "pass" / unparsable summary on both readers / contradiction /
     integrity finding / empty population / hand-edited artifact), plus GREEN controls so none is
     vacuous, plus the corpus-less REFUSAL arm that drives a relocated census copy and requires
     exit 2. ⭐ And the control that matters most ran through the **real driver**, not the enforcer:
     one appended line in `census.md` → `✗ FAIL CORPUS-KEY-INTEGRITY … commit/merge blocked`;
     restore → `ALL 27 enforced doctrines PASS`.
  3. **DURABILITY** — ✅ **satisfied**: the producer is tracked and the claim is now WATCHED by a
     registered doctrine that `.githooks/pre-commit` runs, with the `<meta:mirror>` and
     `<meta:book-count>` checks holding the registry, `DOCTRINE_ENFORCEMENT.md` §10 and the book's
     published count equal at 27.

## Commit log

- `.1` instrument landed: `PGEN-CORPUS-KEY-AUDIT-0001` (2026-08-24).
- `.1`(a)–(d) done: `PGEN-CORPUS-KEY-AUDIT-0002` (2026-08-24).
- `.1`(e) done, leaf `.1` CLOSED: `PGEN-CORPUS-KEY-AUDIT-0003` (2026-08-24).

## Changelog

- **2026-08-24** — tree created by director approval; `.1` instrument landed and measured; `.2`,
  `.3`, `.4` opened.
- **2026-08-24** — `.1`(a)–(d) done. Deciding-evidence attribution read from the row's own basis
  (the planned `furthest_position` route was measured wrong AND circular); the message extractor's
  blindness to iverilog's untagged `syntax error` fixed (168 → 176 classes); contradictions
  2 → 0 with the 100 % false-positive rate and the 4-vs-124 power bound both published; `.1`(e)
  opened to wire the gate.
- **2026-08-24** — `.1`(e) done and leaf `.1` CLOSED. Doctrine `CORPUS-KEY-INTEGRITY` registered
  (the repository's 27th), 13/13 refusal arms firing, RED control proven through the real driver.
  Wiring it surfaced two live defects only a real consumer could expose: the census crashed on its
  own documented `--md` when the path lay outside the repo (after the work had succeeded), and the
  enforcer's first scratch path was off the repository volume. Frontier → `.2`.
