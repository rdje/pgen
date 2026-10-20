# A check that cannot run must SAY SO — never green, never red

**Kind:** feedback (standing rule). **In force since:** 2026-08 — six gates and two task trees have cited this path
since `DONE-BAR-VISIBILITY.3`. **Card written:** 2026-10-20 (`CI-PARITY-GATE-ROT.121`), when a reader found that the
eleven references pointed at a file that had never existed: the rule was law in the instruments and absent from the
layer that explains it.

## Context

A doctrine or gate judges a SUBJECT (a tracked baseline, a published number, a grammar) with INPUTS that are not all
tracked: a build product under `generated/` or `rust/target/`, a lane output that `.gitignore` names, a vendored corpus
that is a git submodule nobody initialised, a tool on this host's `PATH`, a kernel limit this host does not have. On a
fresh clone or a hosted runner some of those inputs are absent, and a check that scores their absence as either verdict
is wrong in a way nothing else catches: a green that evaluated nothing is a false pass (`presence is not a pass`), and a
red that evaluated nothing sends the operator to "fix" a tree that has no defect.

**Measured 2026-10-20** (`CI-PARITY-GATE-ROT.121`, the first hosted doctrine run in 56 days, `memory-architecture-gate`
run `39370056063`): five doctrines red on a `ubuntu-latest` runner, the local hook green twice on the same tree.
`PARSE-COST-RATCHET` scored an instrument's `die("corpus manifest missing")` (exit 2, a cannot-run on a `.gitignored`
lane output) as *the published LR-family share no longer describes this tree*; `BASELINE-IDENTITY` died in a traceback
because a corpus fixture overflowed Python 3.12's JSON decoder and `RecursionError` is not a `ValueError`;
`PIPELINE-LANE-CURRENCY` derived a lane context from thirteen EMPTY submodule directories and called the difference
DRIFT; `CI-MIRROR-DOCS-SURFACES` refused correctly (`rg not found on PATH`) but no workflow provided the tool; the book
chapter check handed a 152 KB page to `python3` as one environment string and Linux's 128 KB per-string limit turned
`Argument list too long` into "OUT OF SYNC with the registry". Then the operator followed the ratchet's own guidance for
a moved instrument digest — `make sv_parse_cost_ratchet` — and after 341 s got the same breach back: a re-measure cannot
clear a moved identity; only the rebaseline re-keys it.

## Decision

1. **Three verdicts, not two.** Every check reports PASS, FAIL or NOT EVALUATED. NOT EVALUATED names the absent
   input and the command that would make it evaluable, on every run, on stderr, and the doctrine driver counts it in
   its scope block. It is never folded into PASS and never reported as FAIL.
2. **A refusal is not a breach.** An instrument that cannot judge exits 2 and says why; a subject that fails exits 1.
   A caller that runs an instrument branches on that distinction and carries the refusal's text into its own
   NOT EVALUATED list — it never scores exit 2 as a failure of the subject.
3. **Absent-by-design inputs are NOT EVALUATED on sight.** A `.gitignored` file, an untracked build product, an
   uninitialised submodule (which IS a directory — empty), a tool a stock runner lacks: the check tests for them before
   deriving anything, because a derivation over an empty corpus produces numbers, and numbers compare.
4. **A crash is a defect of the check.** An exception class the instrument did not anticipate (a `RuntimeError` behind
   a `ValueError` guard) is caught, counted and SAID; the subject it could not inspect is listed, not skipped silently.
5. **Guidance must be followable.** The command a breach prints clears the condition it reports; a command that cannot
   (measured) is a defect of the message. Where the clearing act is deliberate (a rebaseline), the message says so and
   says why.
6. **Every workflow declares what it needs.** The environment a check requires (tools, submodules, build products) is
   provided by the job that runs it, never assumed from the host that wrote it (`CI-PARITY-GATE-ROT.120`(b)).

## Consequences

- The push mirror's export carries this host's submodules, tools and kernel; the hosted runner carries none of them.
  A doctrine that runs only in CI (`.119`(c)) is therefore verified on the hosted run and nowhere else until the
  mirror replays it in a CI-shaped export — the checkout `CI-PARITY-GATE-ROT.121` built by hand (a detached worktree,
  empty submodule directories, no `generated/`, no binaries, a `PATH` without the host's extras).
- Self-tests drive the NOT EVALUATED arms as deliberately as the RED ones: a refusal never observed firing is not
  known to work (`check_pipeline_lane_currency.sh --self-test`, `pipeline_context.py --self-test`).
- Related: `docs/knowledge/presence-is-not-a-pass-a-conformance-surface-records-its-own-verdict.md` (the green side
  of the same rule), `docs/knowledge/two-outputs-of-one-run-can-vouch-for-each-other.md`.
