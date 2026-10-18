# SV-SPEED: SystemVerilog preprocess → parse at or beyond slang and Verilator speed

## Metadata

- Tree ID: `SV-SPEED`
- Status: `proposed` — SEQUENCED by director directive (created 2026-10-18; **precondition: the SystemVerilog parser is released to Nexsim** — the SV
  lane lock, [[project_nexsim_sv_signoff_delivery_focus]]). Not `active` until that release lands.
- Roadmap lane: SystemVerilog released-parser performance (the `systemverilog` + `systemverilog_preprocessor`
  families and the handwritten expansion stage between them)
- Created: `2026-10-18`
- Owner: repo-local workflow
- Priority: **DIRECTOR DIRECTIVE (2026-10-18)** — [[project_sv_speed_on_par_with_slang_and_verilator]]:
  *"After the SV parser is released to NEXSIM we will focus on speed and the SystemVerilog run as fast as it possibly
  can, as we need to be on par with commercial parsers and be lighting fast."* · *"I can't access commercial tool, so
  we will have to be on par or even beat slang and Verilator and go beyond."*

## Goal

PGEN's SystemVerilog pipeline (preprocess → parse, as Nexsim runs it) is at or below the time of the faster of slang
and Verilator for the same work on the same host — per file and in aggregate over the SV corpus — and then goes
beyond that bar.

## ⛔ HARD CONSTRAINT — the correctness floor

A lever that moves ANY verdict, tree, certificate, manifest or pin does not land — no net-win argument
([[feedback_correctness_before_speed]]). Before and after every lever, on the same binaries the ceremony publishes:
`REKEY-MOVERS: IDENTICAL` over the raw and pipeline lane tables, `AST-IDENTITY-SWEEP … ast_moved=0 verdict_moved=0` on
`sv_2017` / `sv_2023` / `verilog_2005`, the three adjudication manifests byte-identical, the certificate sweep
`fully_certified=true`, the svpp census and bank unchanged.

## Non-Goals

- Changing what any parser accepts or the AST it builds (that is correctness work, owned by the SV trees).
- Elaboration or type checking — Nexsim's front end ends at the AST; a reference tool's elaboration time is never
  compared against a parse.

## Acceptance Criteria

- A reproducible benchmark (`.1`) whose every number names the tool, its revision, its mode, the host and the binary
  sha.
- The ratio PGEN ÷ (faster reference) ≤ 1 per corpus suite and in aggregate, measured by that benchmark; then the next
  bar, opened by the leaf that reaches parity.
- The correctness floor green at its pre-lever value after every landed lever.
- The SV book carries the benchmark, its protocol and the current ratios, with the commands to reproduce them.

## Task Tree

- ID: `SV-SPEED`
  Status: `proposed` (sequenced: activates at the Nexsim release)
  Goal: the Goal above
  Children: `SV-SPEED.1` … `SV-SPEED.4`

- ID: `SV-SPEED.1`
  Status: `pending`
  Goal: **BENCHMARK PROTOCOL + REFERENCE BUILDS.** slang exported read-only from `stimuli/sv/subs/slang` @ `4106501b`
  (`git archive`, lazy fetch disabled; the submodule is a sparse `tests/`-only checkout and stays untouched) and built
  Release under `rust/target/`; its `FetchContent` dependencies land in that build directory. Verilator: the host's
  5.050, or a build of the vendored `stimuli/sv/subs/verilator` @ `a534a1d1` (its store is complete too —
  `docs/knowledge/a-vendored-corpus-checkout-is-not-its-object-store.md`); the leaf picks one and records why.
  Fix: the population (the SV corpus rows the pipeline runs, with the include directories and defines the pipeline
  gives each file — the same ones handed to each reference tool); each tool's mode (slang `-E` and `--parse-only`;
  Verilator `-E` and `--lint-only`, recorded per mode since Verilator has no parse-only mode); the allocator question
  (slang links `mimalloc` by default — measured both ways or matched); warm-cache protocol, repeat count and the
  statistic (median of N); jobs = 1 per measurement; the memory guard around every run.
  Acceptance: one instrument with a tracked headline (`SV-SPEED-BENCH:`) and identity rows, its self-test, its TOOLBOX
  entry and evidence-vocabulary row.
  Verification: `pending`
  Commit: `pending`

- ID: `SV-SPEED.2`
  Status: `pending`
  Goal: **BASELINE.** The first reading on the released binaries: per-file ratios, the geometric mean per suite and
  overall, the worst 40 files, peak RSS, and `uvm_pkg.sv` on its own (the largest single preprocess in the corpus
  population).
  Acceptance: the census committed with its identity rows; the SV book's speed chapter with the numbers and the
  commands.
  Verification: `pending`
  Commit: `pending`

- ID: `SV-SPEED.3`
  Status: `pending`
  Goal: **ATTRIBUTION.** Profiler self-time / call-graph attribution of PGEN's time on the worst-ratio files, split
  by stage (preprocessor parse, expansion, SV parse, AST build, I/O), each lever candidate costed against its share
  ([[project-speed-phase-profiler-root-cause-signature]]; `RGX-0078` is the template campaign).
  Acceptance: a ranked lever list, each with its measured share and the stage it lives in.
  Verification: `pending`
  Commit: `pending`

- ID: `SV-SPEED.4`
  Status: `pending`
  Goal: **LEVERS** — one child leaf per lever from `.3`, each landing only with the correctness floor green and its
  before → after ratio measured by `.1`'s instrument.
  Verification: `pending`
  Commit: `pending`

## Current Frontier

| Order | Leaf | Status | Why next |
| --- | --- | --- | --- |
| 1 | `SV-SPEED.1` | `pending` (the tree is sequenced) | the protocol every later number depends on; starts at the Nexsim release |

## Decisions

- `2026-10-18` (`PGEN-SV-SPEED-0001`): tree created from the director's directive
  ([[project_sv_speed_on_par_with_slang_and_verilator]]); sequenced after the Nexsim release; slang and Verilator are
  the reference because no commercial tool is accessible.

## Open Questions

- None blocking: the tree is sequenced, and `.1` fixes the protocol.

## Blockers

- The Nexsim release (sequencing, not a defect).

## Verification Log

- `2026-10-18` (`PGEN-SV-SPEED-0001`): verified for this tree's text — slang's driver at `4106501b` declares
  `-E,--preprocess` and `--parse-only` (`tools/driver/slang_main.cpp`) and `-I,--include-directory,+incdir` /
  `-D,--define-macro,+define` (`source/driver/Driver.cpp`), both read from the submodule's object store with
  `GIT_NO_LAZY_FETCH=1`; that store holds the WHOLE tree at `HEAD` — `git rev-list --objects --missing=print HEAD`
  733 objects, **0 missing** (a shallow clone, not a blob-filtered one, whatever `stimuli/sv/subs/PROVENANCE.md`'s
  acquisition recipe says), so the export needs no network; `external/CMakeLists.txt` fetches `fmt` (and optional
  deps) through `FetchContent` — the build does; `verilator --help` lists `-E`, `--lint-only`, `-I<dir>` and
  `+incdir+<dir>`; `cmake --version` 4.4.3 (slang declares `cmake_minimum_required(VERSION 3.28...4.3)`).

- `2026-10-18` (`PGEN-SV-SPEED-0001`): the status word is `proposed` — the producer of the per-family status blocks reads a
  row's first status word against a closed vocabulary (`scripts/report_family_status.sh` `OPEN_WORDS`), and `sequenced` is
  not in it (it would have published the row as UNREADABLE); attributed to `systemverilog` in
  `rust/test_data/grammar_quality/family_status_block_v0.json`, as `RGX-0078` is to `regex`.

## Related apparatus (already in the tree)

- `PARSE-COST-RATCHET` (a registered doctrine): the SV parser's rule-entry / committed / memo-hit counters on a sampled
  corpus cannot RISE unwatched. A speed lever is expected to LOWER them; a lever that lowers time while raising them
  is read twice.
- `ENGINE-UNIVERSAL-SERVICES.44`(d) (a rejection may cost no more than 2× the deepest accepted parse) and `.84`(e) (the
  deep-nesting peak-RSS term) — engine-tier cost findings parked behind the SV lane lock; `.3`'s attribution says
  whether either is on the corpus's path.
- `RGX-0078` — the regex speed campaign: its baseline / attribution / lever discipline and its ⛔ correctness
  constraint are this tree's template.
