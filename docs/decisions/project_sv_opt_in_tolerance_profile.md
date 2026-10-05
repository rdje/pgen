---
name: project-sv-opt-in-tolerance-profile
description: "DIRECTOR DECISION (2026-10-02) — the SystemVerilog release carries an OPT-IN tolerance profile for the four simulator-tolerated spellings (foreach over a selected array, solve-before inside a foreach constraint, a dotted name as a cross item, a lifetime in a method prototype). The default profiles sv_2017 / sv_2023 stay strict-LRM and keep rejecting them."
id: project-sv-opt-in-tolerance-profile
title: "The SV release carries an opt-in tolerance profile; the default profiles stay strict"
date: 2026-10-02
evidence: "docs/tasks/SV-CORPUS-GRAD.md leaf .41 (PGEN-SV-CORPUS-GRAD-0427): DIALECT-CONSTRUCTS: scanned=16336 | prototype_lifetime: files=2 statements=3 | cross_dotted: files=3 statements=19 | foreach_selected: files=21 statements=71 | solve_in_foreach: files=5 statements=14; seven OpenTitan package files reject at one of the four under both readings of the pipeline lane; ADJUDICATION-REPROS listed=898 failures=0 with the four spellings pinned REJECT on sv_2017 and sv_2023"
reverify: "python3 docs/tasks/artifacts/sv_corpus_grad/dialect_constructs/scan.py   # the four spellings' corpus counts; and: python3 stimuli/sv/run_adjudication_repros.py   # the pins: REJECT under sv_2017 / sv_2023"
answers:
  - "does PGEN accept foreach (a[i][j]) or a lifetime in a method prototype"
  - "is there a tolerant or relaxed SystemVerilog profile, and is it the default"
  - "why does the SV parser reject code that VCS, Xcelium and Questa accept"
  - "who decided that dialect tolerance is opt-in and which spellings does it cover"
  - "how does a spelling join the tolerance profile"
tags: [systemverilog, profiles, tolerance, strict-lrm, director-decision, opentitan]
status: current
metadata:
  node_type: memory
  type: project
  director_directive: true
  created: 2026-10-02
  owning_tree: SV-CORPUS-GRAD
  owning_leaf: SV-CORPUS-GRAD.43
  bounded_by: feedback_sv_strict_lrm_compliance_default
---

**⛔ DIRECTOR DECISION (2026-10-02), verbatim:**

> About "One decision for you, not blocking - Whether the release should carry an opt-in tolerance profile
> for the four simulator-tolerated spellings. OpenTitan writes them in 24 files, and seven packages stay
> rejected without it.", looks like we should carry an opt-in tolerance profile, so, let's go for it.

**Context.** `SV-CORPUS-GRAD.41` asked the LRM text about every construct at which an OpenTitan DV package
rejects under both the `sv_2017` and the `sv_2023` reading of the pipeline lane. Four spellings are tool
tolerances: the commercial simulators accept them, neither IEEE 1800-2017 nor 1800-2023 derives them, and
the clause text speaks against each (§12.7.3, A.1.10, §19.6, §8.24). They were pinned `REJECT` and the
question of an opt-in layer was surfaced. [[feedback_sv_strict_lrm_compliance_default]] had left that door
open in its own words: *"Dialect tolerance may be ADDED later as an opt-in layer if a real need appears."*
The director judged the need real.

**Decision.** The SystemVerilog parser ships an opt-in tolerance profile that accepts the four spellings.
The default profiles are unchanged: `sv_2017` and `sv_2023` stay strict and keep rejecting all four.

**What the decision fixes, and what it leaves to execution.**

- FIXED by the director: the layer exists; it is OPT-IN; its scope is the four spellings of `.41`.
- Chosen in execution (`SV-CORPUS-GRAD.43`, revisable without re-opening this decision):
  - **One profile, named `sv_2023_tolerant`, built on `sv_2023`.** The newer edition is a measured superset
    of the older (`2017_only_pass = 0` over 16 336 files, on the raw lanes and through the pipeline), and a
    consumer asking for simulator-like acceptance wants the 2023 constructs too. The name carries its base
    so a second tolerant profile can be added without renaming this one.
  - **The existing `@profiles` directive carries it** — the first rung of the neutrality ladder: no engine
    byte. Every rule live under `sv_2023` is also listed for `sv_2023_tolerant`, and a check holds that
    invariant so the tolerant profile can never silently lose a strict construct. A profile-inheritance
    directive is the named alternative, not built: it would be an engine service with one customer.
  - **The four pins keep their `REJECT` rows on `sv_2017` and `sv_2023` and gain `ACCEPT` rows on
    `sv_2023_tolerant`.** A spelling the strict profiles start accepting still turns a pin red.

**Consequences.**

- The membership of the tolerant profile is a CLOSED LIST. A spelling joins it only with a ruling of the
  `.41` kind — its LRM text read, its corpus count, a reproducer pair — never because a file failed to parse.
- "Tolerant" is not "lenient": the profile accepts exactly `sv_2023` plus the listed spellings. Everything
  the strict profiles reject for another reason is rejected here too.
- The primary reading of the corpus and of the pipeline lane stays `sv_2017`; the verdict-coverage tuple is
  not redefined by this decision. The tolerant profile gets its own reading and its own census.
- Users see it in the SV parser book: the profile list, the four spellings with the rewrite the standard
  offers, and the statement that the default is strict.

## Addendum 2026-10-05 — a fifth member, by a delegated call: a hierarchical name inside `binsof(…)`

**⛔ DIRECTOR DELEGATION (2026-10-05), verbatim:**

> This call "The binsof(a.b.c) admission from the earlier slice is still your call and still not blocking."
> is yours to make but better be sota, signoff and production-grade.

**The call, made in execution (`SV-CORPUS-GRAD.45`): ADMIT.** A hierarchical name of three or more parts as
the operand of `binsof(…)` joins the closed list as its fifth member, under its own release and the full
ceremony. The ruling the list requires is on file (`SV-CORPUS-GRAD.44`: §19.6.1's text in both editions, the
corpus count, the reproducer pair).

**Why admit, measured.**

- **It is the REFERENCE form of the third member.** Every OpenTitan statement that writes it — all 8 — names
  a dotted cross item of its OWN cross (`sysrst_ctrl_in_out_inverted_vseq.sv:58-69`,
  `core_ibex_fcov_if.sv:686-689`, `core_ibex_pmp_fcov_if.sv:704-725`). A select expression has no other way
  to name an implicitly covered cross item: `binsof` takes the item's name. A profile that admits the
  declaration and rejects its only reference is not closed under its own construct.
- **The boundary it leaves today is an accident of token shape.** A TWO-part name inside `binsof` is
  syntactically `cover_point . bin` and parses on every profile; a THREE-part name does not. OpenTitan writes
  both on adjacent lines of ONE select expression (`core_ibex_pmp_fcov_if.sv:711-712`:
  `binsof(cs_registers_i.mstatus_q.mpp)` rejected, `binsof(cs_registers_i.priv_mode_id_o)` accepted). A
  boundary nobody chose is not one a release should publish.

**What "sota, signoff and production-grade" binds the admission to** — each a check, not a sentence:

1. **The boundary is the NAME and nothing else**: a `hierarchical_identifier` of three or more parts, the
   node the tolerated cross item already carries. An expression, a trailing select, a call inside `binsof`
   stay rejected under the tolerant profile, each a reproducer pin.
2. **The strict profiles move on no instrument**: zero movers on the three raw lanes, `ast_moved=0` on the
   AST identity sweeps, parse cost flat (the alternative is ONE gated rule, asked for nothing where the gate
   is off — `.43`'s lesson).
3. **Its own AST kind** (`tolerated_hierarchical`, the cross item's shape), and a two-part name keeps the
   standard `cover_point` reading under the tolerant profile, pinned by arm.
4. **Two-sided pins**: `.44`'s invalid pin keeps `sv_2017` / `sv_2023`; its byte twin is `tolerated` on the
   tolerant profile; the superset sweep and doctrine `SV-TOLERANT-PROFILE-SUPERSET` hold the rest.
5. **The full release ceremony and the tolerant reading re-run**, the prediction stated BEFORE the
   measurement — `sysrst_ctrl_env_pkg` and the two ibex coverage interfaces are expected to clear this
   construct; a further barrier behind it is reported, not hidden (`.43` predicted seven packages and
   measured six).
6. **The books, the contract and this record's list change in the same commit** ("four" becomes "five").

**Pre-screened in scratch before any tracked grammar byte moved** (2026-10-05, the interpreter on a scratch
grammar, `rust/target/sv45/`): `--lint-grammar` rc 0 at 1745 rules with the tracked grammar's notes line for
line; the OpenTitan shape, the isolated three-part name, a four-part name, a path with an inner select, and
OpenTitan's own 54-line covergroup read `sv_2017:false sv_2023:false sv_2023_tolerant:true`; an expression, a
trailing select and a call inside `binsof` read `false` under all three; the standard forms read `true` under
all three.

**What this does not do.** It does not open the list to "what a file needs": the next candidate still needs
its own ruling of the `.41` kind, and the list stays closed by name.
