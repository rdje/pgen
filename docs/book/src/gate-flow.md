# The Gate Flow — Reference

PGEN's claims are made by **gates**: deterministic programs that re-derive a fact
from the repository and exit nonzero if it does not hold. There are 123 of them,
they call each other, they hand artifacts to each other, and until recently
nothing wrote down how that machinery is supposed to work.

This chapter is that reference. It describes the anatomy of a gate, the layers
they compose into, what flows in and what flows out, the artifact hand-off
protocol between them, who invokes what — and, because the flow has been the
source of several real defects, a catalogue of the ways it has actually failed
and the contract a new gate must satisfy so it does not fail those ways again.

> **Why it needed writing down.** Over five sessions this surface produced: a
> parity gate that could not complete for 1,371 commits; eight of eleven workflow
> replays dying on one missing artifact; eleven of fifteen hosted workflows unable
> to build at all; a required sub-gate that could only pass when the parser it
> watched *failed*; a flagship aggregate budgeting 60 minutes for a 143-minute job;
> and an aggregate consuming a three-day-old artifact as current proof. None of
> these were parser bugs. All of them were flow bugs.

---

## 1. Anatomy of a gate

Every gate has the same three-part shape.

```
make -C rust SHELL=/bin/bash <gate_name>      ← the entry point
        └── rust/scripts/<gate_name>.sh       ← the implementation
                └── rust/target/<gate_name>/  ← the state directory (its output)
```

**The `make` target** is the only supported entry point. It exists so the gate can
be invoked identically from a shell, a hosted workflow, an aggregate, or the local
parity gate. Targets are `.PHONY` and the recipe is normally one line:
`cd $(RUST_DIR) && ./scripts/<gate_name>.sh`.

**The script** owns everything else. It resolves the repository root from its own
location (`ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"`) so a
relocated checkout keeps working — a hard requirement, since the repository is
expected to move between volumes.

**The state directory** is where a gate puts everything it produces. The layout is
conventional across the surface:

| path | contains | measured usage |
|---|---|---|
| `$STATE_DIR/summary.txt` | human-readable `key: value` lines | 54 of 91 gate scripts |
| `$STATE_DIR/summary.json` | the machine-readable record other gates parse | 38 of 91 |
| `$STATE_DIR/logs/` | one log per stage, named after the stage | 69 of 91 |
| `$STATE_DIR/work/` | intermediate artifacts and sub-gate state dirs | 63 of 91 |

`$STATE_DIR` itself defaults to `rust/target/<gate_name>/` and is overridable by a
`PGEN_<GATE>_STATE_DIR` environment variable. **That override is the hinge the
whole composition model turns on** — see §5.

### Exit codes

| code | meaning |
|---|---|
| `0` | the claim holds |
| `1` | the claim does not hold — a real finding |
| `2` | the gate **refuses**: it cannot run, or cannot see what it is meant to check |

The third is not decoration. A gate that cannot evaluate its subject must refuse
rather than pass, because a skip that reports success is indistinguishable from a
proof. `generated_clippy_correctness_gate` exits 2 when the artifacts it lints are
absent, precisely so that a clean checkout cannot lint an empty room and print
zero findings.

### Environment knobs

Every knob is `PGEN_`-prefixed and named after its gate. Three families recur:

- `PGEN_<GATE>_STATE_DIR` — relocate the output;
- `PGEN_<GATE>_CONTRACT_FILE` — point at a different tracked contract;
- `PGEN_<GATE>_EXISTING_<UPSTREAM>_STATE_DIR` — the artifact hand-off (§5).

### The summary contract

`summary.json` is the interface between gates. Its shape is stable:

```json
{
  "gate": "sv_parser_aggregate_contract_gate",
  "version": "...",
  "generated_at_utc": "2026-07-28T20:11:02Z",
  "state_dir": "...",
  "proof_surfaces": { "generation_report_json": "...", "...": "..." },
  "metrics": { "generation_parser_rejections_total": 0, "...": 0 }
}
```

- **`proof_surfaces`** are absolute paths to the artifacts this gate *consumed or
  produced*. A downstream gate should read paths from here rather than
  reconstructing them, so it can never judge a different run than the one whose
  numbers it is reading.
- **`metrics`** are the derived numbers. A downstream gate should read these
  rather than re-deriving them from raw artifacts, for the same reason.

---

## 2. The layers

Gates compose into four layers. Each has a different job and different rules.

### Layer 1 — leaf gates

A leaf gate proves one thing about one surface: `fixed_point_gate` (bootstrap
determinism), `performance_gate` (benchmark thresholds), `mdbook_docs_gate` (the
book builds), `regex_pcre2_compile_oracle_gate` (regex matches PCRE2's verdicts).
It has no sub-gates and produces one state directory.

### Layer 2 — family aggregates

A family aggregate runs several leaves and reasons over their combined output:
`sv_parser_aggregate_contract_gate`, `annotation_contract_gate` (which composes
twenty sub-gates), `regex_parser_family_contract_gate`,
`vhdl_parser_family_status_gate`. These both *run* sub-gates and *consume* their
summaries — which is where hand-offs (§5) appear.

### Layer 3 — the flagship aggregate

```bash
make -C rust SHELL=/bin/bash sota_exit_gate
```

The release gate. It runs ~40 stages, each declared **required** or
**informational**, driven by the tracked policy `rust/config/sota_exit_policy.env`:

```
PGEN_SOTA_POLICY_REQUIRED_CHECKS="differential_baseline_contract fixed_point_gate
  annotation_contract_gate annotation_100_gate annotation_nonbootstrap_e2e_gate
  ebnf_stimuli_quality_gate stimuli_module_parity_gate differential_regression_gate
  performance_gate embedding_api_gate"
```

plus per-family switches (`PGEN_SOTA_POLICY_RUN_SV_STIMULI_QUALITY=1`,
`..._REQUIRE_SV_STIMULI_QUALITY_STRICT=1`, …) that promote individual stages
between informational and required. Each stage runs through a `run_check` helper
that logs to `$STATE_DIR/logs/<stage>.log` and records a row in `summary.csv`.

**Measured cost: 4,249 s (71 min) locally** for a run that reached stage 21 of the
SV block. It is not a per-commit gate and is not meant to be.

### Layer 4 — the meta-gates

Two gates check the *proof surface itself* rather than the product:

- **`ci_workflow_local_gate`** — the local stand-in for the paused hosted CI. It
  exports a tracked-files-only worktree (`git ls-files`, the same shape
  `actions/checkout` produces), runs **33 surface audits** over the real
  repository, then **replays the command each tracked workflow runs** inside the
  export. See §3 for why the export is the interesting part.
- **`scripts/check_doctrines.sh`** — the doctrine enforcer, <!-- DOCTRINE-COUNT -->**21**<!-- /DOCTRINE-COUNT --> registered
  checks, run by `.githooks/pre-commit` on **every commit**. This is the only layer
  that runs without a human deciding to (§6). The registry inside it is the single
  source of the roster; `DOCTRINE_ENFORCEMENT.md` §10 is its reviewed mirror and a
  meta-check fails if the two ID sets disagree.

---

## 3. Inputs

A gate's inputs come from four places, and only three of them are in git.

### Tracked inputs

| input | where | notes |
|---|---|---|
| grammars | `grammars/*.ebnf` | the single source of truth for every parser |
| contracts | `rust/test_data/grammar_quality/*.json` | 38 tracked contracts pinning expected values |
| policy | `rust/config/*.env`, `rust/config/*.json` | which stages are required, branch-protection minimums |
| corpora | `stimuli/`, `regex_corpus_bundle/`, `json_corpus_bundle/` | external test corpora |

### The untracked input, and why it matters more than the others

```
generated/   ← pipeline output: parsers, AST JSON, annotation inventories
             ← NOT tracked in git (.gitignore:24)
```

`generated/` holds the compiled parsers. It is deliberately untracked — it is
build output, regenerated from the grammars. **Every consequence below follows
from that one fact.**

`rust/src/lib.rs` includes generated parsers **two different ways**:

```rust
#[cfg(feature = "generated_parsers")]
pub mod generated_parsers {
    pub mod return_annotation {
        include!("../../generated/return_annotation_parser.rs");   // ← literal path, NO cfg
    }
    #[cfg(has_generated_systemverilog_parser)]
    pub mod systemverilog {
        include!(env!("PGEN_SYSTEMVERILOG_PARSER_PATH_RESOLVED")); // ← cfg-guarded
    }
    // …
}
```

Nine sites are behind a `has_generated_*` cfg that `rust/build.rs` sets only when
the artifact `is_file()` — their absence merely **disables** a parser. **Two are
included by literal path with no such cfg**, so under `--features
generated_parsers` their absence is a hard `error: couldn't read …` that takes the
whole crate down.

⇒ **any command that compiles the crate with that feature requires `generated/` to
exist first.** Measured: **11 of the 15** tracked workflows, and 8 of the 11
workflow replays.

### Regenerating it — the single recipe

```bash
make -C rust SHELL=/bin/bash regenerate_generated_parsers
```

Seeds `generated/ebnf.rs` via the cold-clone bootstrap, emits the annotation pair,
then the seven grammar families. **Measured 236 s from a bare tracked tree.**

This recipe has exactly one definition. Everything that needs it calls that
target: hosted workflows through the composite action
`.github/actions/regenerate-parsers`, and the local parity gate through its
preparation step (`PGEN_CI_WORKFLOW_LOCAL_PREPARE`, default `true`). Copies of a
recipe drift; one definition cannot.

### Is what's on disk what the source produces? — `GENERATED-REPRODUCIBILITY`

Being untracked has a second consequence, and it went unguarded far longer than
the first: **nothing in git can report on `generated/`, so nothing noticed when an
artifact stopped matching the source that is supposed to produce it.**

Measured 2026-08-16: both annotation parsers — the two included by literal path
above, i.e. the pair the annotation backend links to generate *every other parser*
— carried a line the code generator **cannot emit**:

```rust
self.coverage_deltas.clear();
```

`grep -c` over the generator → **0**. `git log -S` → **no commit, ever**. The
emitter's own source comment at that site defends the omission deliberately. The
artifacts had been written from an uncommitted editor state about eighty minutes
before the commit that finalised the emission, and the project's resume pointer
recorded *"`generated/` FRESH"* the whole time.

Each existing candidate was measured, and none could see it:

| candidate | what it actually proves | why it misses this |
|---|---|---|
| a recorded hash | the artifact has not moved since someone recorded the hash | detects **movement**, never **staleness** |
| `fixed_point_gate` | regeneration converges across its own cycles | it **overwrites** the on-disk artifact in cycle 1 — the stale copy is invisible *by construction* |
| `PARSE-COST-RATCHET` tier 1 | the SV parser's cost inputs are unmoved | same movement-vs-staleness gap |
| a green build | the artifact **compiles** | compiling is not being current |

**Re-derive and diff is the only thing that answers the question.** The doctrine
does exactly that, in two tiers:

```bash
bash scripts/check_doctrines.sh                                  # tier 1, ~1.0 s, every commit
make -C rust SHELL=/bin/bash generated_reproducibility_gate      # tier 2, ~57 s, on demand
make -C rust SHELL=/bin/bash generated_reproducibility_rebaseline
```

**Tier 1** re-hashes the three things each artifact is a function of — the
artifact, its input JSON, and a digest over the tracked emission sources *plus the
recipe that invokes them*. If none moved, the artifacts cannot have become stale;
if one moved, the gate demands a re-verify rather than guessing which way. The
emission source set is **derived** (`git ls-files rust/src/ast_pipeline` +
`rust/Makefile`), so a generator file added tomorrow joins by construction, and it
is deliberately over-inclusive: its failure direction is a spurious re-verify, not
silent staleness.

**Tier 2** re-derives all ten artifacts through the tracked recipe and demands
byte-identity.

⛔ Both tiers **assert the embedded `-o` path site count before trusting a hash**,
and refuse rather than compare when the two sides disagree. A generated parser
writes its own output path into the emitted source — once per artifact today, and
36 346 times in the SystemVerilog parser until the label was hoisted to a module
constant on 2026-08-17 — so re-deriving to a different filename changes the
artifact's size for reasons that have nothing to do with the source. The hoist
shrank that effect by four orders of magnitude but did not remove it, so the
assertion stays. That trap has inverted three published readings; see
[Diagnosing Unknowns → Comparing two generated parsers](diagnosing-unknowns.md#comparing-two-generated-parsers).

⚠️ Honest bound, and it is the same one `PARSE-COST-RATCHET` states about itself:
tier 1 proves *"nothing that could have changed the artifacts has changed"*, not
*"the artifacts are correct"*. It inherits whatever tier 2 last established. On a
fresh clone `generated/` is absent, and the check reports **NOT EVALUATED** —
loudly, never as a pass.

---

## 4. Outputs

A gate produces four kinds of output, with different audiences and lifetimes.

| output | audience | lifetime |
|---|---|---|
| **exit code** | the caller | immediate |
| **`summary.txt`** | a human triaging a failure | until the next run |
| **`summary.json`** | downstream gates | until the next run |
| **`logs/<stage>.log`** | a human root-causing a failure | retained on failure, often pruned on success |
| **`work/`** | downstream gates, and the gate's own later stages | until the next run |

Two conventions are load-bearing:

**Successful runs may prune, failed runs must retain.** `ci_workflow_local_gate`
deletes its export directory on success and keeps it on failure — the evidence is
only interesting when something broke. `PGEN_CI_WORKFLOW_LOCAL_KEEP_RUNS=1`
overrides.

**Logs are bounded where the producer is verbose — and the loudest producer was
silenced rather than merely bounded.** The regeneration recipe used to run the
generator with `--debug --trace`, so one preparation captured in full measured
**7.1 GB**: harmless on a 3.6 TB volume, fatal on a hosted runner with ~14 GB
free, and streamed into the Actions log of every workflow that regenerates. A
step whose output is truncated is a step whose failure evidence may not survive,
which is the opposite of what a gate flow is for.

Those flags left the shipping path in `CI-PARITY-GATE-ROT.31`. The same sequence
now emits **14.19 MB**, and all but 4 838 B of that is one `ast_pipeline` cargo
rebuild — a **485.7×** reduction end to end, with every generated artifact
byte-identical across the change (33 of 33, against a same-session determinism
control). `FLOW-INTEGRITY` invariant (10) fails any tracked Makefile line that
invokes `--generate-parser` while carrying `--debug` or `--trace`, so they cannot
drift back.

The trace was moved off the default path, not deleted. Any generation target
takes the engine's existing verbosity knob:

```bash
PGEN_TRACE_VERBOSITY=debug make -C rust SHELL=/bin/bash focus_systemverilog
PGEN_TRACE_VERBOSITY=high  make -C rust SHELL=/bin/bash regenerate_generated_parsers
```

`debug` reproduces exactly what `--debug --trace` used to force (21 955 687 B on
`focus_json`, against 21 955 519 B measured direct from the old flags); `high`
reproduces the old bootstrap level. The parity gate still bounds its capture to
the last 4 MiB — now cheap insurance rather than the thing standing between a
hosted runner and its disk.

---

## 5. The artifact hand-off protocol

This is the intricate part, and the part that has produced the subtlest defects.

An aggregate often already has an artifact a sub-gate would otherwise spend
minutes reproducing. The protocol for saying so:

```bash
PGEN_SV_FAMILY_STATUS_EXISTING_SV_PARSER_AGGREGATE_STATE_DIR="$SV_STIMULI_AGGREGATE_CONTRACT_STAGE_STATE_DIR" \
  make -C rust SHELL=/bin/bash sv_parser_family_status_gate
```

The consumer branches on it:

```bash
if [[ -n "$EXISTING_SV_PARSER_AGGREGATE_STATE_DIR" ]]; then
    sv_parser_gate_state_dir="$EXISTING_SV_PARSER_AGGREGATE_STATE_DIR"   # reuse
else
    run_logged "sv_parser_aggregate_contract_gate" …                     # produce it
fi
```

**23 of the 91 gate scripts accept hand-offs.** The semantics are exactly:

> **An empty value means "not supplied — produce it yourself".
> A non-empty value means "this artifact exists; do not produce it".**

### The three legitimate shapes

1. **Empty** — `"${PGEN_SOTA_EXISTING_VHDL_STIMULI_QUALITY_STATE_DIR:-}"`. The
   sub-gate produces its own artifact. This is the default and the correct shape
   when the caller has nothing to offer.
2. **An in-run stage directory** — `"$SV_STIMULI_QUALITY_STAGE_STATE_DIR"`, a
   directory this same run produced a few stages earlier. This is the reuse the
   protocol exists for.
3. **An operator-supplied directory** — passed explicitly by someone who knows the
   artifact is current.

### ⛔ The shape that is always wrong

```bash
# NEVER do this:
PGEN_..._EXISTING_SV_SYNTAX_CLOSURE_STATE_DIR="$RUST_DIR/target/sv_syntax_closure_gate"
```

That is a gate's **standalone default** directory. It exists only if somebody once
ran that gate by hand. Pointing a hand-off at it asserts *"this artifact exists"*
on the strength of a hope, and it fails two ways:

- **On a clean tree the directory is absent.** The value is non-empty, so the
  consumer skips the branch that would have *produced* the artifact, and then dies
  asserting on a file nothing wrote. A default that turns "not supplied" into
  "supplied but nonexistent" **disables the machinery that would have made it
  real**.
- **On a developer machine the directory often exists, and is old.** Measured
  2026-07-28: a `sota_exit_gate` run consumed
  `rust/target/sv_syntax_closure_gate/summary.txt` dated **three days earlier**.
  Had the sibling directories also been present, the run would have gone **fully
  green on evidence of unknown vintage** — the release gate certifying today's tree
  with last week's proof, leaving no red to notice.

### The provenance rule

A hand-off is now **verified, not trusted**:

1. the directory and its `summary.txt` must exist and be non-empty — refused up
   front, naming the caller's variable, rather than dying later on a confusing
   downstream assertion;
2. when the caller declares a run epoch via `PGEN_GATE_ARTIFACT_MIN_EPOCH`, the
   artifact must be **at least that new**. Artifacts produced earlier in the same
   run pass; a pre-run leftover is refused.

The aggregate exports that epoch once, at the top of the run:

```bash
export PGEN_GATE_ARTIFACT_MIN_EPOCH="$(date +%s)"
```

Exported rather than passed per call, so a hand-off added later inherits the
guarantee without anyone remembering to wire it. Standalone operator runs set no
epoch and keep today's behaviour — inventing a reference point there would refuse
legitimate reuse.

> **Portability note, learned the hard way.** Reading an artifact's mtime is not
> portable: BSD `stat` spells it `-f %m`, GNU coreutils spells it `-c %Y` and reads
> `-f` as *file system information*, which **succeeds** at printing six lines of
> block counts. A BSD-first chain on a GNU host does not fall through — it captures
> that block as the "timestamp". Try both and **validate the result is a bare
> integer**; a reordered guess is still a guess.

### The measurement-parameter rule — for an artifact that is *tracked*, not handed off

The rule above governs an artifact passed **between gates inside one run**. A different
rule governs the artifacts a run **publishes into the repository** — the external-corpus
characterizations under `stimuli/<family>/characterization/`, which are graduation
oracles: the adjudication manifest and the whole rejects-valid burn-down are computed
from them.

For those, recording the provenance is only half the job. Writing down *how* a number
was measured buys nothing if the next run never reads it back — and for a while, nothing
did:

```bash
stimuli/run_external_corpus.sh sv          # the documented invocation
```

All three tracked artifacts record `60 8 0` against
`rust/target/release/parseability_probe`. The script's own defaults were **20 s** against
`rust/target/debug/parseability_probe`, and one corpus file parses in **12 s release vs
127 s debug**. So the documented command re-measured the corpus under a ~10× slower binary
at a 3× tighter deadline and published the result over the tracked oracle without a word.

⛔ **The direction of the error is what makes it a bar defect, not a nuisance.** A file
that times out is later adjudicated `divergence:explained_timeout` and *leaves*
`unexplained_rejects_valid`. So a slow machine **improves the burn-down number** — the
parser looks better because the hardware was busier. Nothing guarded that direction.

Since `SV-CORPUS-GRAD.3.27` the recorded parameters **bind**:

| the caller… | the runner… |
|---|---|
| omits an argument | **adopts** the value from the artifact's provenance block |
| supplies a *matching* value | proceeds |
| supplies a *differing* value | **refuses, exit 5**, printing both sets, before parsing anything |
| has no tracked artifact yet | uses the script defaults, and says so |

Two explicit escape hatches, both loud:

```bash
PGEN_CORPUS_OUT_DIR=rust/target/scratch stimuli/run_external_corpus.sh sv   # measure, then DIFF
PGEN_CORPUS_REBASELINE=1              stimuli/run_external_corpus.sh sv 30  # new baseline, on purpose
```

Two supporting properties make the comparison the refusal protects actually practical:

- **Every timeout is re-run alone** at the same deadline before it is recorded, because a
  timeout under `-P 8` is a claim about the machine as much as about the parser. Per-file
  durations land in a `durations.tsv` sidecar — never as a fourth column of `results.tsv`,
  whose three-field shape three consumers unpack positionally and a fourth silently skips.
- **Both artifacts are sorted** by `(sub-corpus, path)`. `xargs -P` appends in completion
  order, so the tracked file's row order used to be nondeterministic: a re-run with
  identical verdicts still produced thousands of moved lines. An oracle you cannot diff
  against its own replacement is one nobody diffs.

---

## 6. Invocation — who actually runs what

A gate that nothing invokes is indistinguishable from a gate that does not exist.
This repository found three such gates by accident, one per session, before it
started measuring. The inventory:

```bash
bash scripts/check_gate_reachability.sh --report
```

It derives the whole picture on every run — the universe of targets from
`rust/Makefile`, the edges from the Makefile, the gate scripts, the tracked
workflows, the git hooks and `COMMIT.md` — and sorts targets into three tiers.

| tier | invoked by | count today |
|---|---|---|
| **AUTOMATIC** | a git hook, or a workflow on `push`/`pull_request` | 14 |
| **OPERATOR** | an aggregate, or a `workflow_dispatch`-only workflow | 79 |
| **ORPHAN** | nothing at all | 31 (+1 policy-only) |

> ⚠️ **The automatic tier is thin, and that is the single most important fact about
> this flow.** It was **zero** until 2026-07-30, when `DONE-BAR.4` enabled `push:`
> on the three lanes that need no `generated/` regeneration and so run on a bare
> checkout — `branch-protection-contract-gate`, `fixed-point-gate` and
> `mdbook-docs-gate`. Those three, plus `parser_books_gate` and the ten per-parser
> book gates `mdbook_docs_gate` pulls in, are the whole automatic tier.
> The other 11 tracked workflows stay `workflow_dispatch`-only to conserve account
> minutes, and `memory-architecture-gate.yml` — the only one also on
> `pull_request` — runs the doctrine driver and no `make` target at all. **The
> automatic layer covers the <!-- DOCTRINE-COUNT -->21<!-- /DOCTRINE-COUNT --> enforced doctrines and 14 of the 118 gate
> targets.** Every other proof lane in this chapter runs only when a human asks —
> the 79 operator-reachable ones exactly as much as the 31 orphans.
>
> ⛔ **Read those counts from the instrument, not from here.** Four prose copies of
> this table's figures went stale the day `DONE-BAR.4` landed and still asserted a
> zero automatic tier two weeks later, while `--report` had been deriving 14 on
> every run — tracked as `CI-PARITY-GATE-ROT.36`.
>
> ⭐ Even that doctrine coverage was partial until 2026-07-29: the workflow named
> five enforcers **individually**, so 8 of the then-13 registered doctrines had no
> automatic lane, and a doctrine added afterwards silently got none. It now invokes
> `scripts/check_doctrines.sh`, so the roster is *inherited* from the registry
> rather than re-typed — and invariant 8 below fails the build if that regresses.
> ⚠️ Five of the eighteen judge a **staged diff** and a hosted push has none, so
> they exit 0 having evaluated nothing; the driver prints a `scope:` note naming
> them, because a green tick must not imply they were satisfied.
>
> A direct consequence: **wiring an orphan into an aggregate or a paused workflow
> moves it from ORPHAN to OPERATOR and makes nothing run.** It is worth doing when
> a lane is cheap enough to ride along with something people already run; it is not
> a substitute for the automatic tier.

The 32 orphan and policy-only targets each carry a recorded disposition in
`rust/test_data/grammar_quality/gate_reachability_register_v0.json`. This is a
ratchet, not a report: the orphan set is re-derived every run, an untriaged orphan
fails the check, and a register entry that no longer names an orphan fails too, so
the exemption list can neither be bypassed nor quietly accumulate.

⛔ **A target named inside a check script's error MESSAGE is not invoked by it, and
reading it as an invocation is worse than cosmetic.** `check_generated_reproducibility.sh`
prints an actionable refusal ending `make -C rust SHELL=/bin/bash
generated_reproducibility_gate`; because that message spans lines, the line-by-line
reader took its third line for a command and certified the target reachable in the
strongest class. The ratchet is two-sided, so the false badge then *blocked* the
honest `accepted-operator-invoked` disposition — a mis-classification that keeps the
truth out of the register. `CI-PARITY-GATE-ROT.34` fixed the **reader**, never the
message: a refusal that does not say how to fix itself is a worse refusal. The
reader now models the four contexts a shell body has — `NORMAL`, `'…'`, `"…"` (with
`$(` and backticks re-entering `NORMAL`) and heredoc bodies — and suppresses only
lines that *begin* inside a quoted string or heredoc payload, so a quoted command
word such as `bash "$ROOT/scripts/x.sh"` keeps its edge. Six synthetic ground-truth
arms replay the defect and the three live corpus shapes a naive quote-parity fix
breaks; each was watched failing against a mutated reader before being trusted.

---

## 7. How this flow has actually failed

Eleven distinct shapes, all measured, all from real incidents. A new gate should be
read against this list.

### 1. A check that *cannot run* and returns green

`ci_workflow_local_gate` accepted any string as a workflow filter. A mistyped name
skipped all eleven replays and printed `✅ … parity gate passed` with exit 0 — and
because the 33 audits *had* run, the green looked earned.

**Rule:** an unknown selector, or a run that ends up doing zero work, must refuse
and print the roster.

### 2. A check that *cannot see* and returns green

`generated_clippy_correctness_gate` lints the generated parsers. In a clean
checkout `generated/` is absent, so `--features generated_parsers` compiles zero
parsers and a naive lint reports zero findings.

**Rule:** verify the subject was actually examined — that gate reads cargo's own
build-script cfg census — and **exit 2 rather than 0** when it was not.

### 3. A check that *nothing invokes*

`ast_dump_contract_gate` was red for four sessions. No aggregate, no workflow and
no hook referenced it, so nobody found out.

**Rule:** §6. Be reachable, or carry a recorded disposition.

### 4. A check that *requires a defect in order to pass*

`sv_failure_context_contract_gate` asserted *"at least one generation
failure-context excerpt"* against a surface whose contract is documented
one-profile, one-sample and whose parser accepts its own generated sample. It
could only pass **if the parser rejected its own output**: it passed when the
system was broken and failed when it worked.

**Rule:** a zero must be **earned**, not forbidden — require that the surface was
exercised (`attempts_total >= 1`), that the zero is consistent (no rejections and
no errors), and that a present excerpt is well-formed. That is strictly stronger:
it fails on a vacuous run and on a broken capture path, neither of which the
"at least one" form could distinguish from a healthy one.

### 5. A check that *reuses evidence it did not produce*

§5. The aggregate consumed a three-day-old artifact as current proof.

**Rule:** a hand-off proves its provenance or refuses.

### 6. A metric that stopped meaning its own name

The closed-loop stimuli gates report `resolved_targets` by grepping the pipeline's
`Target-driven generation: resolved N/M targets` line. Correct when written — but
`SV-EXH-PROOF.7.4.3` later appended a **second pass** to the same invocation, the
minimal-witness pass, which resolves more of the same targets and reports them on
its own line. Nothing updated the reader.

So `resolved_targets` became a snapshot taken *before* the last pass, while
`final_targets` stayed the residual measured *after* it. The downstream assertion
`resolved + final == initial` — a true statement about the pipeline — went
permanently false: `723 + 31 != 1033`, when the loop had in fact resolved 1,002.

⭐ **The gate went red because the pipeline got better**, and stayed red invisibly
for two months because it is reachable only from `sota_exit_gate`, which had never
got that far.

**Rule:** a metric read out of a log is a **coupling to a pass structure**, and the
coupling must be checked, not assumed. The reader now requires the final pass's
line to be present (absent ⇒ refuse — reporting an intermediate figure is worse
than refusing), and cross-checks that its baseline and total agree with the earlier
pass, so appending a stage tears the check instead of silently shifting a number.
And an accounting invariant belongs at the **producer**, where it covers every
grammar, rather than in one family gate that covers one.

### 7. A value read from prose when a structured artifact was sitting next to it

Every stimuli gate ends by `cat`-ing its own `summary.txt`, so its **stage log**
contains that block *plus every other line the gate printed*. The flagship aggregate
read the SystemVerilog and VHDL closed-loop telemetry — 52 values — out of that log
with `sed -nE "s/^${key}: (.*)$/\1/p" | tail -n 1`.

Two things were wrong with that, and only the second one had already broken something:

1. **The log is ambiguous.** Measured on a real run's artifacts, **20 of 155** SV keys
   and **20 of 74** VHDL keys match more than one line — the gate's own banner echoes
   the same keys before the summary. `tail -n 1` is right only for as long as the
   summary stays the last thing printed, which is failure 6 waiting to happen again.
2. **The log is not always there.** When the aggregate is pointed at an existing stage
   state dir (`PGEN_SOTA_EXISTING_*_STATE_DIR`) it does not re-run the stage — it runs a
   `test -s .../summary.txt` probe, and `run_check` redirects that probe's output *over
   the stage log*. The haystack became a 33-byte `Bash profile loaded successfully`, so
   **22 SV telemetry values were published as `unknown`** while the aggregate printed
   `✅ SOTA exit gate passed.` — with every one of those values present in the
   `summary.txt` right beside it.

⭐ The VHDL reader escaped **by luck**: it tried the log first but had the artifact as a
fallback, so an empty log fell through to the right answer. Two stages of the same
aggregate, the same shape of data, one silently wrong — *a family whose telemetry is
correct by accident is not evidence that the reader is correct.*

**Rule:** **a gate reads the structured artifact its producer wrote, and falls back to
prose only when that artifact does not exist.** The log is a human surface; the
`summary.txt`/`summary.json` is the interface (§1, *The summary contract*). And when the
two disagree, that is a **finding to report**, not something a selector should quietly
resolve — the aggregate now asserts every key in a stage's artifact against the log and
fails naming each divergence, so drift becomes visible instead of merely harmless.

### 8. A consumer that outlived its producer's schema

`LANG-CAPABILITY-AUDIT.10.6` retired the Perl arm of the EBNF frontend dual-run gate.
The producer stopped emitting four keys. **Five** gate scripts kept reading them, and
`jq -r '… | .absent_key'` does not fail — it prints the bare word `null`.

The damage was not uniform, and that is the whole lesson. The same missing value
reached three different consumption shapes in one script:

| site | shape | outcome |
|---|---|---|
| `assert_equal "pass" "$…"` | string compare | **fails loudly** — `"pass"` ≠ `"null"` |
| `--argjson … "$…"` into the summary | publish | **publishes `null`** — a reported value for a measurement that no longer exists |
| `(( rust_rule_count < perl_rule_count ))` | bash arithmetic | **passes vacuously** — `(( ))` reads `null` as an unset name, i.e. **0**, so `276 < 0` is false |

⭐ **The third is the dangerous one.** It is a *regression floor* — "the Rust frontend
must not report fewer rules than Perl did" — and it silently became a comparison against
a constant zero. It could never fire again. Fixing the loud two without noticing it would
have restored a green gate with a dead assertion inside it, which is failure 2 all over
again.

Measured directly: `bash -c 'a=276; b=""; (( a < b )) && echo LT || echo GE'` → `GE`,
`rc=0`.

**Rule:** **an extraction must distinguish "the producer measured null" from "the producer
no longer emits this key".** The reader now goes through a helper that refuses an absent
key, a null value, or anything but exactly one matching entry, naming the key — so the
next schema change is loud in *every* consumer rather than loud in some and silent in
others. And when a comparison loses its second operand for good, **delete it and say so**
(`dual_run_regex_cross_frontend_floor: retired` is published in the summary) rather than
leaving arithmetic that implies a floor nobody is computing. A one-sided ratchet against a
pinned baseline is an honest replacement — a constant right-hand side cannot go null —
but it must not be described as the cross-implementation check it replaced.

### 9. A check that compares a *verdict* where the claim is about *output*

For most of its life the EBNF frontend dual-run gate asserted that the parser generated
from `grammars/ebnf.ebnf` **accepted** each tracked grammar — `12/12`, and green. The claim
the project actually needs is stronger: that generated parser is meant to *replace* the
hand-written frontend, and a replacement must produce the **same output**, not merely
agree that the input is well-formed.

The distinction is not academic. `LANG-CAPABILITY-AUDIT.10.6` part 2 built the missing
output-level differential (`pgen::ebnf_envelope_differential`, §*Where the pieces live*) and
it found two live `grammars/ebnf.ebnf` defects **on its first run** — both of which parse
`Ok`, which is exactly why a decade of verdict-level green never saw them:

| defect | what a verdict-level gate saw | what the envelope differential saw |
|---|---|---|
| `regex_flags := /([gimsuyx]*)/` matches across trivia | `Ok` | `a := /x/ members /y/` resolves the reference to **`embers`**; `xylophone` loses **two** characters |
| a leading `@annotation` binds to the previous rule | `Ok` | **47** annotations across the tracked grammars steer the WRONG rule |

⭐ **The rule:** when a gate's *purpose* is a claim about equivalence, an accept/reject
comparison is not a weak version of that claim — it is a **different claim**, and passing it
says nothing about the one you meant. Ask what a green run would let you *do*: if the answer
is "replace one implementation with the other", the gate has to compare what they produce.

⚠️ The corollary is that the new instrument's own first RED is as likely to be the
instrument as the subject — which is why it carries a **positive and a negative control**
and refuses to publish if either misses (§*The contract for a new gate*). Two of the first
sweep's ~100 divergences were the projection's own bugs, and the positive control is
precisely what distinguishes those from findings.

### 10. A measurement that cannot say what produced it

The tracked SystemVerilog external-corpus report published a headline — `16 336 files,
59.3 % pass` — and named its instrument as *"generated by … against `parseability_probe`"*.
That basename is not an identity. It does not say which build, which grammar, or which
commit, so **the only way to challenge the number was to compare the git commit dates of
two unrelated files**: the report was committed 2026-07-25, `grammars/systemverilog.ebnf`
last changed 2026-07-26, therefore the report provably could not describe `HEAD`
(`SV-CORPUS-GRAD` § *axis-2 freshness audit*).

That reconstruction is expensive, easy to skip, and it only ever yields a *suspicion*. When
`SV-CORPUS-GRAD.10` actually re-ran the corpus, the verdict set turned out to be
**identical** — zero pass→fail, zero fail→pass across all 16 336 files. The report was stale
by provenance and correct by content, and **nothing short of re-measuring could tell those
two apart**.

⭐ **The rule:** a published measurement must carry the identity of what produced it, as
content hashes rather than a name — for this runner, the sha256 of the parse binary, the
grammar and the generated parser, plus the measuring `HEAD`. Then "is this number still
mine?" is one command instead of an archaeology, and a reader who re-hashes three files
knows immediately whether to quote it or re-run it.

⚠️ **The same leaf showed why the instrument's own settings belong in that identity.** The
`timeout` column — and only that column — moved with the per-file budget (9 timeouts at 20 s,
6 at 60 s) and with the build (4 on a release binary, whose extra speed lets two more files
finish). The pass/fail sets never moved. So a timeout is a statement about the instrument,
never about the parser, and a report that hides which instrument ran invites a budget effect
to be read as a corpus regression.

### 11. A recorded parameter that nothing ever read back

Failure 10's fix worked: the corpus reports began carrying content hashes and the
invocation they were produced with. And then that record sat there, **write-only**, for
every run that followed — because recording provenance and *checking* provenance are two
different pieces of work, and only the first one had been done.

The consequence is in the previous section: the documented bare invocation ran at the
script's defaults (20 s, debug binary) while all three tracked artifacts recorded 60 s and
the release binary, and re-published over them silently. `SV-CORPUS-GRAD.3.25` measured
the damage on one such accident — **6 rows moved into `divergence:explained_timeout`,
timeouts `4 → 10`, on a parser change that provably could not touch them.** They landed on
already-deferred rows that run, so the burn-down number happened to survive; landing on
`must_accept` rows, the identical drift would have *removed* files from
`unexplained_rejects_valid` and improved the headline.

The root cause is one line of shell. `TIMEOUT_S="${2:-20}"` collapses *"the caller asked
for 20"* and *"nobody said, so the default is 20"* into one value — so the information a
drift check would need is destroyed before any check could run. Capturing `"${2-}"` first
and defaulting afterwards is the whole fix; the refusal is what that distinction makes
possible.

⭐ **The rule:** provenance is not a record, it is a **precondition**. If an artifact
states the conditions it was measured under, the run that would replace it must be held to
them — adopt when the caller is silent, refuse when the caller disagrees. A provenance
block nobody reads back is a comment.

> The unifying principle behind all eleven: **a gate must report on its subject, and
> only its subject** — and it must report on *the claim being made*, not a cheaper claim
> nearby. It must not report on its own documentation, its own absence, somebody else's
> stale output, a quantity that has quietly stopped being the one its name promises, a
> number scraped out of prose while the producer's own artifact sits unread, or a value its
> producer stopped measuring — and when it cannot report at all, it must say so rather than
> return green. Nor may it report a number while withholding what produced it: an
> unattributable measurement can only be trusted by re-running it.

---

## 8. What stops it drifting back

Fixing the flow is not the same as keeping it fixed, and the difference turned out
to be measurable. After every repair above had been mechanized, a census of *where
those mechanisms lived* found:

| invariant | lived in | tier |
|---|---|---|
| regeneration coverage | `ci_workflow_local_gate` | **OPERATOR** |
| workflow timeout floor | `ci_workflow_local_gate` | **OPERATOR** |
| one home for the recipe | `ci_workflow_local_gate` | **OPERATOR** |
| preparation stays on | `ci_workflow_local_gate` | **OPERATOR** |
| gate reachability | `scripts/check_gate_reachability.sh` | AUTOMATIC |

**Four of the five sat in the operator tier** — inside a gate that, per §6, nothing
runs automatically. The flow had been repaired with checks that could themselves
rot, which is the exact failure the repairs existed to end.

So they moved:

```bash
bash scripts/check_flow_integrity.sh --report
```

`FLOW-INTEGRITY` is an enforced doctrine, run by `.githooks/pre-commit` on **every
commit**. It is deliberately cheap — file reads and greps, no cargo, no build, no
network — because a check nobody minds running is a check that keeps running. It
enforces eleven invariants, each traced to something that actually happened:

| # | invariant | the incident |
|---|---|---|
| 1 | a workflow running a `make -C rust` gate declares the regeneration step — and a measured-exempt one does **not** | 14 of 15 workflows could not build |
| 2 | any job carrying that step budgets ≥ 30 minutes | the flagship budgeted 60 min for a 143-min job |
| 3 | the recipe keeps one home; no workflow re-inlines it | it was about to be copy-pasted into ten more files |
| 4 | the parity gate's preparation stays on by default | the identical default eroded once before, unguarded |
| 5 | no hand-off points at a gate's standalone default dir | a run consumed a three-day-old artifact as current proof |
| 6 | no assertion requires a defect in order to pass | a required sub-gate passed only when the parser failed |
| 7 | hand-off provenance coverage only improves | 1 of 23 consumers verify; the list may only shrink |
| 8 | the doctrine roster keeps an automatic lane **through the driver**; no auto-triggered workflow re-types enforcer names | the one auto-running workflow named 5 of 13, so 8 doctrines had no automatic lane and every one added later inherited none |
| 9 | a guard tests the artifact it actually **reads** | 6 blocks guarded on `summary.txt` then read `summary.json`; a sub-gate dying mid-run left a 0-byte `summary.txt`, so the guard read false and a 5-hour run ended on a missing-file error four lines below the real cause |
| 10 | no tracked Makefile line invokes `--generate-parser` carrying `--debug` or `--trace` | the shipping recipe asked the generator to narrate itself: **6.89 GB** per regeneration, local and streamed into the Actions log of the 11 workflows that reach it, for artifacts the flags cannot change |
| 11 | every `$(<FAM>_JSON): $(<FAM>_EBNF)` rule is guarded against GNU Make 3.81's **whole-second** mtime comparison | a grammar rewritten in the same second as the previous build's output was invisible to make: the rule was skipped at exit 0 and a probe judged the *previous* grammar. Measured across the ten families: 5 exposed on the `json ← grammar` edge, **10 of 10** on `parser ← json` |

### Invariant 11 in detail — and the premise it refuted

Make 3.81 skips a rule iff `floor(mtime(target)) >= floor(mtime(prereq))`, so its
decision is *wrong* exactly when the two share a whole second and the prerequisite
is genuinely newer. The acute fix (`CI-PARITY-GATE-ROT.32`) repaired the scratch
probe path and reasoned that the minute-long families were unlikely to be hit. The
sweep measured that reasoning and found it names the wrong duration.

A sequential driver cannot touch a grammar before `make` returns, so **the gap it
must beat is the work after the target is written**, not the whole build:

| edge | the gap | measured |
|---|---|---|
| `json ← grammar` | the generator step | 0.06-28.5 s ⇒ **5 of 10 exposed** |
| `parser ← json` | the frontend step, on the *next* build | 0.006-0.111 s ⇒ **10 of 10 exposed** |

The second edge produces a *fresh* json beside a *stale* parser — the artifact the
gates inspect is current while the artifact that actually parses is not. It reaches
SystemVerilog, whose regeneration takes 28.5 s but whose frontend step takes 0.054 s.

The fix is an exact-window guard rather than an unconditional `rm`:
`scripts/make_freshness_guard.sh` removes an artifact only when make would be
provably wrong, so an up-to-date tree pays nothing. That matters because three
tracked gates call `focus_*` purely to *ensure* an artifact exists
(`sv_cert_recognized_union_gate.sh`, `verilog_2005_conformance_gate.sh`,
`rtl_const_expr_cert_gate.sh`) and would otherwise pay a full regeneration every
run. `focus_scratch` keeps the stronger unconditional `rm`, because its caller has
just edited the slot by construction — and the invariant accepts either shape.

Requiring `make >= 4.0` would fix all 65 file rules at once; it is **priced, not
adopted** — it costs every contributor an install plus a `gmake`-vs-`make` rename
through the docs, and hosted Linux runners ship make 4.x while hosted macOS runners
ship 3.81, so it buys a CI-parity split as well.

Two design choices make it hard to defeat:

**Derived, not hand-listed.** The workflow roster, the hand-off consumer set and
both forbidden shapes are re-read from the repository every run. Only two inputs
are written down — the measured-exempt workflows and the not-yet-verifying
consumers — because those are human decisions nothing can re-derive.

**One source for the rules.** Both readers — the doctrine check and the parity
gate's audit — read the exemption set from the same
`flow_integrity_register_v0.json`. Two lists that must agree are two lists that can
disagree; twelve assertions once rotted on exactly that shape.

> ⚠️ **Honest limits.** Invariant 7 is a ratchet over an accepted risk: 22 of 23
> hand-off consumers still do not verify what they are handed. The ratchet stops
> that number growing and forces it down one gate at a time; it does not pretend
> the gap is closed. And a pre-commit hook is bypassable (`--no-verify`) — CI is
> the un-bypassable layer, and while hosted Actions are paused (§6) the honest
> statement is that this holds at every commit made through the hook, not "no
> matter what".

## 9. The contract for a new gate

A checklist, derived from the failures above.

**Structure**
- [ ] a `.PHONY` `make` target in `rust/Makefile` whose recipe runs
      `./scripts/<name>.sh` — the only supported entry point;
- [ ] the script resolves `ROOT_DIR` from its own location, never from `$PWD`;
- [ ] `$STATE_DIR` defaults to `rust/target/<name>/` and honours
      `PGEN_<NAME>_STATE_DIR`;
- [ ] it emits `summary.txt` and, if anything downstream will read it,
      `summary.json` carrying `proof_surfaces` and `metrics`.

**Behaviour**
- [ ] exit `0` / `1` / `2` per §1 — and **refuse (2) rather than pass** when it
      cannot evaluate its subject;
- [ ] every value it pins comes from a tracked contract, not from a literal in the
      script — a value designed to move must not be duplicated into an assertion;
- [ ] any zero it accepts is *earned* (§7.4);
- [ ] any artifact it is handed is *verified* (§5), never trusted.

**Wiring**
- [ ] it is reachable from an aggregate, a workflow or a hook — or has a recorded
      disposition in the reachability register (§6);
- [ ] if the workflow that runs it compiles the crate, that workflow declares
      `uses: ./.github/actions/regenerate-parsers` (§3) and budgets at least 30
      minutes;
- [ ] it satisfies `bash scripts/check_flow_integrity.sh` (§8);
- [ ] it is exercised by RED / GREEN / CONTROL probe arms, where the RED arms
      break the invariant and the CONTROL arms prove it does not misfire on
      unrelated changes.

**The probe requirement is not optional.** Every defect in §7 was found by a probe arm or a
control, and several were found in the *fixing* leaf's own work — an audit tripping
over its own source, a driver measuring a stale copy of the rule it verified, an
instrument that reported eight RED arms green over a check that evaluated none of
them. A gate without adversarial arms is a claim, not a proof.

---

## Where the pieces live

| thing | path |
|---|---|
| gate scripts | `rust/scripts/*.sh` |
| entry points | `rust/Makefile` |
| tracked contracts | `rust/test_data/grammar_quality/*.json` |
| release policy | `rust/config/sota_exit_policy.env` |
| hosted workflows | `.github/workflows/*.yml` |
| the regeneration action | `.github/actions/regenerate-parsers/action.yml` |
| doctrine enforcer | `scripts/check_doctrines.sh` + `scripts/check_*.sh` |
| reachability register | `rust/test_data/grammar_quality/gate_reachability_register_v0.json` |
| the incident record | `docs/tasks/CI-PARITY-GATE-ROT.md` |
| the frontend⟷meta-parser envelope differential (§7.9) | `rust/src/ebnf_envelope_differential.rs`, driven by `ebnf_dual_run_diff --envelope-differential` |
