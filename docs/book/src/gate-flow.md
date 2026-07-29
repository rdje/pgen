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
- **`scripts/check_doctrines.sh`** — the doctrine enforcer, 12 checks, run by
  `.githooks/pre-commit` on **every commit**. This is the only layer that runs
  without a human deciding to (§6).

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

**Logs are bounded where the producer is verbose.** The regeneration recipe runs
the generator with `--debug --trace`; captured in full, one preparation measured
**7.1 GB**. Harmless on a 3.6 TB volume, fatal on a hosted runner with ~14 GB free.
The parity gate bounds that capture to its last 4 MiB — `make` stops *at* the
failing step, so the tail is exactly where the evidence is.

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
| **AUTOMATIC** | a git hook, or a workflow on `push`/`pull_request` | **0** |
| **OPERATOR** | an aggregate, or a `workflow_dispatch`-only workflow | 92 |
| **ORPHAN** | nothing at all | 30 (+1 policy-only) |

> ⚠️ **The automatic tier is zero, and that is the single most important fact about
> this flow.** Hosted Actions are paused to conserve account minutes, so 14 of the
> 15 tracked workflows are `workflow_dispatch`-only, and the one that still
> auto-runs (`memory-architecture-gate.yml`) runs the doctrine driver and no `make`
> target at all. **The automatic layer covers the 14 enforced doctrines and none of
> the 123 gate targets.** Every proof lane described in this chapter runs only when
> a human asks — the 92 "reachable" ones exactly as much as the 30 orphans.
>
> ⭐ Even that doctrine coverage was partial until 2026-07-29: the workflow named
> five enforcers **individually**, so 8 of the then-13 registered doctrines had no
> automatic lane, and a doctrine added afterwards silently got none. It now invokes
> `scripts/check_doctrines.sh`, so the roster is *inherited* from the registry
> rather than re-typed — and invariant 8 below fails the build if that regresses.
> ⚠️ Four of the fourteen judge a **staged diff** and a hosted push has none, so
> they exit 0 having evaluated nothing; the driver prints a `scope:` note naming
> them, because a green tick must not imply they were satisfied.
>
> A direct consequence: **wiring an orphan into an aggregate or a paused workflow
> moves it from ORPHAN to OPERATOR and makes nothing run.** It is worth doing when
> a lane is cheap enough to ride along with something people already run; it is not
> a substitute for the automatic tier.

The 31 orphan and policy-only targets each carry a recorded disposition in
`rust/test_data/grammar_quality/gate_reachability_register_v0.json`. This is a
ratchet, not a report: the orphan set is re-derived every run, an untriaged orphan
fails the check, and a register entry that no longer names an orphan fails too, so
the exemption list can neither be bypassed nor quietly accumulate.

---

## 7. How this flow has actually failed

Six distinct shapes, all measured, all from real incidents. A new gate should be
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

> The unifying principle behind all six: **a gate must report on its subject, and
> only its subject.** It must not report on its own documentation, its own absence,
> somebody else's stale output, or a quantity that has quietly stopped being the one
> its name promises — and when it cannot report at all, it must say so rather than
> return green.

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
enforces nine invariants, each traced to something that actually happened:

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
