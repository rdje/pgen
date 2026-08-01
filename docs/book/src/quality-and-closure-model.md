# Quality and Closure Model

One of the most important things to understand about PGEN is that it is not satisfied with "the parser exists."

PGEN is built around a proof-first closure model.

## The Core Idea

A parser family is not considered mature just because:

- the grammar parses,
- the generated parser compiles,
- a few happy-path samples work.

Instead, PGEN aims to close the gap between "it seems to work" and "we have machine-checkable reasons to trust it."

## What Closure Means In Practice

The exact proof surface varies by family and maturity, but the general doctrine includes:

- EBNF-backed parser generation,
- generated artifacts that are reproducible and tracked,
- stimuli generation,
- parser/stimuli round-trip or comparable proof where applicable,
- coverage and gap analysis,
- deterministic replay,
- family-specific contracts and support boundaries,
- executable gates backing published claims.

This is why the repository talks so much about gates, contracts, and tracked evidence. They are not auxiliary paperwork; they are part of the product.

One concrete instance is the **AST-shape contract** per parser family (`rust/test_data/ast_shape_contract/<grammar>_v1.json`): a tracked set of sample inputs, each pinned to the exact runtime carrier the generated parser must produce (its `content_kind` plus, for typed objects, the required keys). The gate re-parses each sample with the *running generated parser* and fails if the observed shape drifts from the locked one — so a return annotation that silently stops producing its typed `{…}` object (and falls back to a raw token sequence) is caught immediately. A sample may lock the shape of the whole-file root rule **or of a nested rule**: by naming a non-root `rule_under_test`, the harness parses the input *as* that rule's own entry, so even deeply-nested typed carriers are regression-protected. (The SystemVerilog UDP truth-table entries — `combinational_entry` / `sequential_entry` — are locked this way, after a duplicated return annotation once degraded their typed carriers to raw sequences unnoticed.)

## ⚠️ THE EBNF IS THE SINGLE SOURCE OF TRUTH FOR THE ACCEPTED LANGUAGE

This is one of the load-bearing invariants of the whole closure model — state it loud:

> **The EBNF grammar — together with its `@predicate` / `@generate` / `@semantic` annotations — is the
> SINGLE SOURCE OF TRUTH for what a parser accepts. The stimuli generator derives samples from the EBNF
> and *nothing else*. Therefore any acceptance constraint that lives OUTSIDE the EBNF — in a hand-written,
> post-parse validation layer — is INVISIBLE to the generator, and the generator WILL emit
> structurally-valid samples the parser rejects. An out-of-band acceptance gate the generator cannot see
> is a DEFECT.**

**Why this matters.** The proof-first model above leans on the **generator⟷parser duality**: the linter
*proves* a grammar well-formed, and the generator *constructively corroborates* it by emitting samples
that re-parse. That round-trip is only sound if the EBNF is the *complete* specification of the accepted
language. The generator generates *by construction* from the EBNF, so it can only ever produce
EBNF-valid strings. If the parser additionally enforces a validator the EBNF does not encode, then

```
accepted language = (EBNF structure) ∩ (out-of-band validator)
generator's target =  EBNF structure
generated-but-rejected =  EBNF structure \ validator   ← silent inconsistency
```

A grammar-driven generator emitting parser-rejected output is therefore never "normal" — it is a
red flag that the grammar is not the whole spec, and it must be root-caused immediately, not waved through.

**Worked example (regex).** `grammars/regex.ebnf` structurally accepts `\u{…}`
(`unicode_escape = "u{" hex_digits "}"`) and `(*<any-name>)`
(`directive_verb = "(*" directive_body ")"`), so the generator emits them. But a separate hand-written
validator, `rust/src/regex_compile_validation.rs`, rejects `\u` ("unsupported regex escape") and
unrecognized `(*verb)` names — a constraint the EBNF never states and the generator never sees. The fix
direction is always one of: **encode the constraint in the EBNF** (a semantic annotation shared by
generation and parsing — the preferred resolution), or **relax/remove the out-of-band validator**. Never
leave the two out of sync. (PCRE2's actual braced form `\x{…}` parses fine; only the EBNF-modelled-but-
unsupported `\u` and arbitrary verb names fail.)

This rule is owned by the `EBNF-SOURCE-OF-TRUTH` task tree and recorded as a binding decision; you can
inspect what the generator is deriving with `--trace high` (or `--trace debug`).

**Audit scope (2026-06-07).** A repo-wide audit of every grammar's parse path (`parser_registry.rs`,
the `parse_*_detail` dispatch) found that **`regex` is the *only* family that applies an out-of-band
acceptance check** — `validate_regex_compile_contract` (10 PCRE2-compile sub-checks, of which the `\u`
escape and the unrecognized-`(*verb)` cases are the ones the generator currently trips). Every other
family (SystemVerilog, the SV preprocessor, VHDL, the RTL frontends, JSON, EBNF, and the annotation
grammars) drives acceptance purely from its generated parser, with no hand-written post-parse rejection.
So the inconsistency is bounded to one grammar, and the fix is tracked there.

**Enforcement (2026-06-07).** A mechanical gate, `scripts/check_ebnf_source_of_truth.sh` (run in the
pre-commit hook and CI), flags any *new* out-of-band acceptance validator wired into the parser registry
— concretely, a `crate::*_validation::` reference in `rust/src/parser_registry.rs` beyond the one tracked
instance (regex's `validate_regex_compile_contract`, pending its EBNF-encoding). So the defect class
cannot silently reappear: a future grammar cannot quietly grow a hand-written post-parse gate the stimuli
generator can't see.

**Migration progress + capstone re-scope (2026-07-09).** The `REGEX-PCRE2-FIDELITY` tree has since
migrated several of the original checks into `regex.ebnf` (so the `\u`/`(*verb)` cases the generator once
tripped are now grammar-owned and the *generator no longer trips the validator at all* — generation is
faithful). But a tools-first probe (session #72, `PGEN-REGEX-PCRE2-0022`) established the honest remaining
state: `validate_regex_compile_contract` still dispatches **8 live checks that are all LOAD-BEARING** —
for **54 hand-writable inputs** (e.g. `[z-a]`, `x{5,4}`, `\pA`, `(?<=a+)b`, `(?=a\Kb)`, `a(*CR)b`,
`(*scs:(1)a)`) the *grammar accepts* and only the validator rejects. So the validator is NOT residual, and
the capstone that deletes it must first encode all 8 families in the EBNF (several need a new
parser-agnostic primitive: value-comparison over decoded ranges, whole-pattern two-pass capture/start-option
inventories, contextual `\K`/lookbehind analysis). Method + full map:
`docs/decisions/project_regex_validator_deletion_blocked_load_bearing.md` and the
`REGEX-PCRE2-FIDELITY.4` SCOPING LOG. Note the distinction: generator↔parser **duality** is clean (the
generator never *emits* these), but that is not the same as the validator being **deletable** (a user can
*hand-write* them).

## Why PGEN Works This Way

PGEN targets domains where parser behavior materially affects downstream tooling and trust:

- HDL tooling,
- regex engines,
- annotation-driven parser platforms,
- future high-rigor language integrations.

In those environments, parser novelty is not enough. Predictability, observability, and repeatable proof matter.

## Task-Tree Ownership Is Mandatory For Code Changes

PGEN enforces a binding, non-negotiable doctrine (adopted 2026-05-17):

> **No code change is made unless it is first tracked by, or owned by,
> a task-tree leaf.**

A "code change" is any edit to the grammars (`grammars/*.ebnf` — the
grammar files are code), the Rust sources, codegen, generated
artifacts, or the machine-checkable shape-contract manifests — anything
that alters parser, codegen, or generated behavior.

Before any such change, a task-tree leaf must exist that owns it. That
leaf — with its explicit goal, acceptance criteria, independent
verification, blockers, and single owning commit — is the unit of
review. The change is then implemented as exactly that leaf and run
through the full commit workflow, lock-stepped with the contracts and
books.

This is not bureaucracy for its own sake: task-tree ownership has
demonstrably and tremendously improved code review and code quality.
The structure forces every behavior-affecting change to be scoped,
justified, independently verified, and documentation-synchronized
*before* it lands — which is exactly the proof-first closure model this
chapter describes, applied at the granularity of every individual
change.

Pure documentation changes (this book, the contracts, the live-status
trackers, the workflow docs) may still use the lighter
`PGEN-<FAMILY>-<NNNN>` single-slice convention; the doctrine governs
code specifically. The authoritative statement lives in
`docs/TASK_TREE.md` ("Code-Change Doctrine") and `COMMIT.md`.

### The acceptance checklist is machine-enforced, and it names five diagnosis families

The owning leaf must carry a checklist whose **ROOT CAUSE**, **ADDRESSED**
and **NO REGRESSION** boxes are ticked *and* backed by real tool output —
checked at commit time by `scripts/check_diagnosis_evidence.sh`. The
backing signature has to sit **inside that box's own bullet**, so a token
elsewhere in the leaf, or in an unrelated co-staged tree file, does not
count.

It must also be a box **this change actually wrote**: the satisfying box
has to sit inside a leaf section that the staged change touches, and all
three requirements must be met within one file. That second rule sounds
redundant and is not. Scoping the signature to its box, without also
scoping the box to the change, left the check accepting *any* ticked box
in *any* staged task file — so a task tree that already held one
compliant leaf supplied the checklist for every later leaf, indefinitely
and for free. Measured before the fix: **33 tracked task files carried
that standing free pass**, and 7 of the last 138 code-change commits
passed only by borrowing a box from a leaf they never touched. The rule
stays deliberately permissive *inside* a leaf — a follow-up commit
editing any part of the same leaf keeps its checklist, and a
deletion-only edit still counts — because a gate that blocks ordinary
multi-commit work teaches authors to bypass it.

The box also has to *be* the box it claims to be. The ROOT CAUSE
requirement is recognized by the header reading `root cause`, or `why`
joined to `where` — deliberately **not** the bare word *"why"*. That
alternative used to be accepted and it failed open on one of the four
named steps: a `**FIX**` box writing *"Why no lower tier: …"* and quoting
a command satisfied the ROOT CAUSE requirement in a leaf that carried no
ROOT CAUSE box at all. Narrowing it cost nothing measurable — four
headers stopped matching, none of them backed by tool output — and it
turned out the same alternative was failing *closed* as well, blocking
leaves whose real checklist was complete but which happened to hold an
unticked `**FIX**` box mentioning "why". The general lesson is worth more
than the regex: *"nothing in the corpus exploits this today"* is a fact
about the corpus at one instant, not a property of the rule.

Which tool output counts depends on what kind of defect it is, and PGEN
recognizes **five** families, because the right instrument differs:

| family | the defect | the instrument |
|---|---|---|
| correctness | the parser accepts or rejects the wrong input | certificate-coverage, `[plannable-probe]`, predicate-rejection traces |
| performance | correct but slow | a profiler (`/usr/bin/sample`, `otool` disassembly, flamegraph self-time) |
| build integrity | a target no longer compiles | the compiler's own `error[EXXXX]` |
| codegen emission | the generator emits the wrong code — it compiles *and* parses fine | the generated-parser lint lane (`clippy::<lint>`) |
| ops / build-flow | the defect is in PGEN's own scripts, Makefiles, hooks or tracking state | `git ls-files`, `make -n`, `shellcheck`, `E2BIG`/`ARG_MAX`, the memory-guard marker |

The last two exist because a defect can be perfectly real while leaving
*no* parse to trace, *no* run to sample and *no* compiler error. A gate
whose families do not match the work people actually do does not raise
quality — it teaches authors to waive it. That is not hypothetical: the
performance family was measured backing just **two** root-cause boxes in
the whole repository, while the speed campaign's own leaves used a
different profiler entirely, and one author had written a waiver note
directly into a checklist box explaining that no family fitted their
defect.

Two shapes are deliberately **not** accepted, both refused on
measurement: a bare `file.rs:123` citation (a location is not a tool
output — it would reduce the gate to "cite a line number") and prose such
as *"verified by grep"* (a claim, not output).

## Closure Is Normalized Across Families

PGEN does not use different quality philosophies for different parser families.

The doctrine is the same across EBNF-based families:

- regex,
- VHDL,
- SystemVerilog,
- annotation grammars,
- Phase S grammars,
- future families.

What differs is not the quality bar, but how much of the proof surface has already landed.

## What `Done` Means — The Three-Leg Bar

`Done` — the `claimed_status` a family carries in the done-bar register — is the strongest claim
PGEN makes about a parser family, and
it is deliberately hard to hold. A family is `Done` only when **all three** of these hold, currently
and simultaneously:

1. **Stimuli-generator proof** — the family's own generated samples close the loop with **zero**
   residual actionable-target debt.
2. **All our gates** — every gate covering the family is green **now**, and is **actually invoked**
   by something. A gate nothing invokes is indistinguishable from a gate that does not exist.
3. **All the external test corpus, passing** — an officially-recognized third-party corpus, asserted
   as a **pass**.

Three exclusions do most of the work, and each exists because the opposite reading had been made
here at least once:

- ⛔ **A triage gate is not a conformance gate.** Reporting `parse_pass_total == cases_declared`
  over a hand-picked sample is a curated slice grading itself, not the corpus passing.
- ⛔ **A characterization is not a pass.** Measuring how a parser behaves against a standard is not
  the same as meeting it.
- ⛔ **The absence of a corpus is an UNMET leg, not an inapplicable one.** "We could not find one"
  is not a proof that none exists.

### `Provisional` is a shipping tier, and it is always qualified

A family that meets legs 1 and 2 but not leg 3 is **`Provisional`** — and `Provisional` **ships**.
Downstream consumers may use it; what they are owed is an accurate statement of what is proven and
what is not, so the decision is theirs and it is made on facts. It is always written with one of two
qualifiers, because bare `Provisional` withholds the most decision-relevant fact on the page:

- **`Provisional (ceiling)`** — a **finished** row. Leg 3 is unreachable *by construction* because
  the language is PGEN's own (the annotation and meta grammars); there is no third-party corpus and
  there never will be. Holding this is success, not debt.
- **`Provisional (corpus pending)`** — an **unfinished** row. The language is externally
  standardized, so a corpus exists in the world and wiring it is outstanding work.

⛔ `ceiling` is deliberately hard to claim: it is the label that *closes* a row, so without a rule
every awkward family drifts into it. It requires that **PGEN itself defines the language** — no
external standards body, no widely-recognized reference implementation.

### Auditing the bar

`bash scripts/audit_done_bar.sh` reports, per family, the state of each leg and a verdict. Its
design principles are worth knowing before reading its output:

- The **family roster is derived from the product**, not listed. The candidate set is
  `grammars/*.ebnf` — the thing PGEN actually ships, which cannot lie about what exists — and every
  tracked grammar must then be adjudicated **exactly once**, as either a parser family or a recorded
  non-family (`grammar_dispositions` in the done-bar register). A grammar in neither **blocks the
  audit**; so does a grammar claimed both ways, and so does an entry naming a grammar that no longer
  exists. That two-sided shape is the point — see *Why the roster is derived this way round* below.
- The **admission rule is pinned to an independent source**: a grammar is a family iff
  `rust/src/parser_registry.rs` ships a registered generated parser for it, the one exception being
  the two `builtin_*` bootstrap contracts, which exist only to break the annotation-parser cycle
  (and which the parse-harness equivalence gate already excludes by name). So the register cannot
  quietly re-classify a shipping parser as a non-family — that is `MISCALIBRATED`, not a judgement
  call.
- It is **read-only**: it never runs a gate, so it cannot manufacture the green it is auditing.
- **A leg it cannot see is `UNPROVEN`, never `MET`**, and `UNPROVEN` does not satisfy the bar. An
  artifact older than the inputs it judged is likewise not proof.
- It carries **ground-truth controls**. If any fails to reproduce it prints `MISCALIBRATED` and
  offers no verdict at all, because an instrument with no ground truth is a confident guess.
- It **audits; it never demotes**. Moving a status claim is a separate, deliberate act.

#### Why the roster is derived this way round

The roster used to be *tracker rows ∩ `grammars/*.ebnf`*. That reads like a derivation, and it is —
but its **left** side was a prose file, so a grammar with no tracker row contributed nothing and the
derivation never saw it. The only refusal fired on the converse arm (on the tracker, absent from the
register), which means **every check guarding the roster sat on the side that could not fail**.

Measured, it hid three shipped parsers — `ebnf`, `json` and `semantic_annotation`, the last with a
published downstream integration contract under `docs/contracts/` — together with the eight gate
targets attributed to them. The book has described **nine** parser/annotation families plus the
`ebnf` meta-grammar for some time; the register said **seven**. The published surface and the
audited surface disagreed by three, and nothing in the repository could notice.

The lesson generalises past this audit: **a derivation is only as honest as the side that can
fail.** When a check joins two sources, ask which one is authoritative about *existence* — and
derive from that one.

### The family-status gates compute the bar

The audit reports; the **family-status gates enforce**. The three of them
(`regex_parser_family_status_gate`, `sv_parser_family_status_gate` — which computes both
SystemVerilog families — and `vhdl_parser_family_status_gate`) compute a status for their family
and fail if the tracker row disagrees. Since `DONE-BAR.2a` they compute the **new** bar, through
one shared helper (`rust/scripts/lib/parser_family_status_bar.sh`):

- Their vocabulary includes the qualified **`Provisional`** tier. The qualifier is derived from
  the language-ownership field of the done-bar register
  (`rust/test_data/grammar_quality/done_bar_family_register_v0.json`) — `pgen` ⇒ `(ceiling)`,
  `external-standard` ⇒ `(corpus pending)` — and an `unadjudicated` owner makes the gate **refuse**
  rather than guess, because the comfortable label is the one that closes a row.
- They carry the **leg-3 criterion** the old bar lacked: `external_corpus_conformance_pass`. It is
  met only by a surface declared in the register (`leg3_surface`) that passes all three tests every
  corpus-named gate measured by `DONE-BAR.1` failed at least one of: it is a **conformance** gate
  (a `*triage*` name is refused outright), it is **external-backed** (its script reads a declared
  corpus root), and it is **actually invoked** (reachable per `scripts/check_gate_reachability.sh`)
  — and its artifact must satisfy the declared pass assertion. While no surface is declared, leg 3
  is unmet.
- Consequently **`Done` is unreachable while leg 3 is unmet**: a family whose own closure criteria
  all hold computes `Provisional (…)`, never `Done`. The cap only applies at the top — statuses
  below `Done` pass through unchanged.
- They also carry the first consumer-facing disclosure criterion, **`ledger_open_entries_zero`**:
  the released-parser bug ledger is read against its **own** "State Meanings" vocabulary
  (`Released`/`Rejected` are closed), and an OPEN entry naming the family demotes its tier — a
  known, still-open downstream defect means the family's proof surface missed something a consumer
  hit. This is deliberately a *tier* criterion, not a pre-commit check: a genuinely open entry is a
  legitimate repository state that must block the family's claim, not unrelated commits.
- Since `DONE-BAR.5e` they also carry **`no_reachable_silent_success`**, which binds the
  silent-success sentinel gate described below to the family's tier. Until that leaf the gate ran and
  bound *nothing*: a placeholder becoming reachable in VHDL would have failed the aggregate while the
  VHDL row kept every one of its criteria green. The criterion is met only when the family's own
  sweep reached **zero** sentinels *and* `generated/<family>_parser.rs` carries **zero** codegen
  placeholders (attributed by exact filename, so one family is never demoted for another's defect).
  ⭐ The status gate **runs the sentinel gate itself**, into its own state dir, rather than reading
  whatever artifact happens to be lying around — this repository has already had a "fresh" aggregate
  run consume a three-day-old summary as current proof. The sweep is cached per *process* (so the SV
  gate, which computes two families, pays for it once) and never per directory.
- All three criteria above are now **pinned in the matching `*_parser_family_status_contract_gate`**.
  They previously existed only in the producing gate, so deleting one would have left its contract
  sibling green over a quietly narrower bar.
- On a tracker misalignment the gate now **states what it computed** — the full
  `summary.txt`/`summary.json` pair, including the computed status, the leg-3 verdict, and the
  qualifier — and *then* exits nonzero. A gate that died before writing its verdict used to leave
  a 0-byte `summary.txt` that had to be forensically recovered from its log.

This ordering is deliberate: **the gate states the truth, then the tracker agrees with it — never
the reverse.** Editing the tracker first would have manufactured disagreements with instruments
that could not even express the honest status (measured: an honest demotion turned 3 of 3
family-status gates red, two of which passed at the time).

### Silent success: the defect every "did it parse?" gate is blind to

A parse can return `Ok` with **zero diagnostics** and still hand the consumer a **placeholder**
instead of the shape the return annotation promised — a `"<invalid_sequence_access>"` string where
an object should be. Acceptance testing cannot see this **by construction**: the parse *succeeded*,
so every gate that asks "did it parse?" is green. This is not hypothetical — nine such
consumer-visible corruptions were found and fixed in SystemVerilog alone, and they are ledgered
(`SV-0014`..`SV-0020`, `SVPP-0001`, `RTL-FE-0002`, `VHDL-0001`, `RTL-CE-0001`).

`make -C rust SHELL=/bin/bash silent_success_sentinel_gate` is the instrument for it, with two arms:

- **STATIC** — the *codegen* placeholders (`<property_access>`, `<array_access>`,
  `<last_extraction>`), emitted where codegen cannot honour a construct and returns success anyway,
  must be **absent** from every shipped artifact. They are absent today; the arm locks that.
- **DYNAMIC** — each family's own stimuli surface is generated under `--validate-parseability`,
  parsed and AST-dumped. **Any sentinel actually reached fails**, naming the family, the sample and
  the input.

⛔ **The static arm alone would be vacuous, and that is the instructive part.** The three codegen
literals occur **0 times** across all 11 shipped artifacts, so a gate asserting only their absence
passes over an untouched tree and can *never* fail. The sentinels that actually ship are the
**runtime** fallbacks — **3,702 arms across 10 of the 11 artifacts**. A gate must be built against
the surface that exists, not the surface a charter named.

⭐ **Calibration is part of the check.** A detector with no *positive* control cannot distinguish a
clean sweep from a blind one — a zero reads as a pass either way. Three pinned facts must reproduce
before any verdict is offered: a fixed ledger repro reads 0, a known-latent site reads **1** under
`--entry-rule` isolation, and that same site reads 0 from the canonical entry. If they do not, the
gate prints `MISCALIBRATED` and refuses.

⚠️ **What it does and does not prove.** The dynamic arm proves *"not reached by 25 validated samples
at the pinned seed"* — **not** unreachability. Reachability is entry-relative, which is exactly what
the calibration arm demonstrates: the same rule is clean canonically and corrupt in isolation. The
gate prints this bound in its own output rather than letting a green imply more than it earned.

⭐ **And it binds a tier.** Existing was not enough: a gate whose verdict enters no status
computation can go red while every tracker row stays green. Since `DONE-BAR.5e` the four
family-status computations carry `no_reachable_silent_success` (see above), so a reachable
silent success now **demotes the family**, with the reached sample count and the honest bound
recorded in the gate's `summary.json`.

## A Check That Cannot Be Run Reports Nothing

A recurring failure mode in this project is worth naming explicitly, because it produces
green dashboards over real blind spots: **a check that cannot be run strictly is a check
that reports nothing.**

The worked example is the generated-parser lint stage. PGEN runs `clippy` in two stages —
strict over `rust/src/**` (hand-written code), and a separate stage over the artifacts in
`generated/`, which can be made strict with `PGEN_CLIPPY_GENERATED_STRICT=1`. For a long
time the generated stage reported **291 errors**, and all 291 were *correct observations
about degenerate code that was nevertheless doing exactly the right thing*:

- the parser generator resolves each rule's branch policy (`longest_match` / `ordered` /
  `priority_first`) at generation time, and used to interpolate it as a string literal
  and re-compare it in the emitted parser. For a rule whose policy *is* `priority_first`,
  that emitted `"priority_first" == "priority_first"` — a `clippy::eq_op` **correctness**
  error over a tautology that was the intended specialization;
- likewise the `@whitespace_sensitive:` layout policy emitted
  `skip_leading_whitespace && false` for a whitespace-sensitive grammar — a
  `clippy::overly_complex_bool_expr` (*"contains a logic bug"*) over an intentionally
  dead block.

Neither was a parser defect. But because both lints are `deny`-by-default correctness
lints, the strict stage could **never** pass, which meant a *genuine* correctness lint
appearing in a generated parser would have been permanently invisible — buried under
80,000 style warnings across 220 MB of emitted Rust.

The resolution PGEN chose is worth internalizing, because the two obvious alternatives
are both wrong:

- ⛔ **leaving it** is the status quo that makes the strict stage unreachable, and it
  trains every future reader to ignore the stage;
- ⛔ **silencing the lints** converts a *measurable* gap into an *unmeasurable* one, which
  is the entire problem being solved;
- ⭐ **emitting the already-decided form** makes the lint stop firing *because the
  degenerate code is gone*. The generator now emits only the branch cascade the
  configured policy selects, and omits the layout-skip block entirely when layout
  skipping is off.

That fold, and the sweep that followed it, removed **30,880** statically-decided
expressions across the eleven generated parsers and **31.7 MB** of emitted Rust — the
SystemVerilog parser went from 149.8 MB to 130.6 MB — with parse behaviour unchanged.

Two things about *how* that was verified are worth more than the numbers, because they
generalize to any quality claim you will read in this project:

**A green gate is not the same as a green measurement.** The associativity fold's first
two attempts each emitted 7,482 instances of `clippy::needless_bool` — a fresh instance
of the very shape being removed. The gate never noticed: `needless_bool` is a *style*
lint, and the acceptance criterion was an *error* count, which stayed at zero throughout.
It was caught by censusing every warning kind before and after, not by reading the gate's
verdict. The habit to copy: when you remove a defect class, re-measure the whole surface,
not the check you happen to be gated on.

**An oracle that cannot see the change is not evidence.** The largest part of the fold
was the associativity tie-break — and before this work, *nothing* in the repository
exercised `@associativity`: no differential row, no test, and no shipped grammar declares
it. Every gate would have passed a fold that inverted the directive outright. The fix was
to build the missing oracle first: three rows in the parse-harness combinator suite over
a grammar whose two alternatives deliberately tie on consumed length and priority, so the
tie-break is the only thing that can pick a winner. Even then, agreement between the
interpreter and the generated parser is weak evidence — both are produced from the same
generator, so a dropped directive would make them agree. So the suite also asserts that
the three associativities *disagree with each other* in the required way: `nonassoc`
rejects the tied input where `left` and `right` accept it, and `left` and `right` produce
different ASTs. Only then does a passing row mean anything.

The general shape to carry away: **when an instrument cannot run, or structurally cannot
see a defect class, it must say so rather than return green** — and when you find one
that cannot see, building it is part of the fix, not a follow-up.

### The Sequel: Driving A Count To Zero Does Not Keep It There

There is a third failure mode hiding behind the first two, and PGEN walked straight into
it. Once the 291 errors became 0, the natural conclusion was that the strict stage could
now be run — and it could. But a sweep for what actually *sets*
`PGEN_CLIPPY_GENERATED_STRICT=1` found **no gate, no aggregate, no CI workflow and no
`make` target**: every occurrence in the repository was prose in a document. Its default
was `0`. So the hard-won zero was defended by nothing but the intention to type an
environment variable.

**A number is not a guarantee; the thing that re-measures it is.** The fix was to make the
stage strict by default and give the policy its own tracked contract
(`generated_clippy_correctness_contract_v0.json`) plus a repo-standard lane:

```bash
make -C rust SHELL=/bin/bash generated_clippy_correctness_gate
```

Three design choices in that gate generalize beyond lint counts.

**Gate the category, not the noise.** Only `clippy::correctness` is enforced. The same
artifacts still carry ~78,800 style and complexity warnings, and gating on those would
recreate the original problem — an unrunnable check — in a new costume. The neighbouring
`clippy::suspicious` category was measured rather than hand-waved: exactly two findings
exist, both in hand-written source and neither in generated code, so the contract records
it as *considered and deferred* with the file and line of each, and a named condition for
revisiting. A subset that is written down can be argued with; a subset that is implicit
just erodes.

**Pin the roster by name *and* by group.** Denying the group alone means a future `clippy`
that reclassifies `eq_op` out of `correctness` silently stops checking the exact defect
class the gate was built for. Denying only the pinned names means new correctness lints are
never picked up. The gate does both: the group catches additions, the 68 pinned names make
any departure a loud failure rather than a quiet narrowing.

**Refuse rather than pass when the gate cannot see.** This is the same principle as above,
turned on the gate itself — and it was a live trap, not a hypothetical. `generated/` is
not tracked in git, and `build.rs` includes each generated parser only when the artifact
exists on disk. So in a clean checkout, on a CI runner, or inside the local workflow-parity
gate's tracked-files-only export, "lint the generated parsers" would have compiled **zero**
generated parsers, found nothing, and exited **0** — a perfect green over an empty room.
The gate therefore exits `2` (a refusal, explicitly not a pass) unless two independent
checks agree: every required artifact is present, *and* `cargo`'s own build-script output
confirms each one was compiled into the unit that was linted. A skip is never a pass.

## Why Status Labels Stay Conservative

This is why a family's `claimed_status` can stay at `Mostly Done` even when it already looks strong to a casual reader. The status labels are meant to reflect proof depth, not enthusiasm.

Likewise, a family can remain `Done` while still receiving maintenance releases or syntax widening, as long as the published closure doctrine for that family remains satisfied.

## How To Read PGEN Claims

When PGEN says something is closed or production-ready, the right next question is:

"What executable proof surface backs that claim?"

That is the correct lens for:

- gates,
- contracts,
- aggregate reports,
- closure rows,
- maintenance releases.

## Primary Source Docs

- `README.md`
- `rust/test_data/grammar_quality/done_bar_family_register_v0.json` (the status claim)
- `docs/book/src/roadmap-and-live-status.md` (its published view)
- `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
- `docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md`
