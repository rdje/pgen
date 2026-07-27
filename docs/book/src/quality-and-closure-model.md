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

This is why `LIVE_ACHIEVEMENT_STATUS.md` can keep a family at `Mostly Done` even when it already looks strong to a casual reader. The status labels are meant to reflect proof depth, not enthusiasm.

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
- `LIVE_ACHIEVEMENT_STATUS.md`
- `docs/reference/PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
- `docs/contracts/PGEN_PARSER_INTEGRATION_CONTRACTS.md`
