# `.8c.3` implementation + verification: the dedicated 256 MiB parse stack (session #196, 2026-07-22)

Companion to the measurement/design doc
[`8c3_stack_ceiling_measurement.md`](8c3_stack_ceiling_measurement.md) (the
tool-pinned WHY+WHERE). This doc records WHAT landed and the measured
before → after verification.

## What landed (the banked design, executed)

New shared primitive `rust/src/dedicated_parse_stack.rs`:

- `DEDICATED_PARSE_STACK_BYTES = 256 MiB` — sized from the measured
  per-frame constants (release ≈2 KB / debug ≈17 KB ⇒ ceiling-real need
  ≈8 MB / ≈70 MB; 256 MiB ≥ 3× the debug worst case). Virtual reservation,
  lazily committed.
- `run_on_dedicated_parse_stack(name, f) -> Result<T, DedicatedParseStackError>`
  — spawn-per-call worker; panics captured as `Panic(payload)`; re-entrant
  calls (already on a dedicated stack) run inline with identical semantics
  (thread-local flag — no nested spawn, no deadlock).
- `run_cli_main_on_dedicated_parse_stack(name, f) -> T` — the CLI wrapper:
  panics `resume_unwind` (identical crash surface to an unwrapped main).
- 5 unit tests incl. a 32 MiB-deep recursion proof that the closure really
  gets the big stack (would overflow the 2 MiB libtest thread otherwise).

Routed boundaries (exactly the three loci from the banked design):

1. **Embedding-API family entries** (`rust/src/embedding_api.rs`):
   `parse_generated_systemverilog{,_ast_json}` and
   `parse_generated_vhdl{,_ast_json}` each run their parser work through the
   new `run_generated_family_on_dedicated_stack` helper (worker panic →
   `E_PARSE_FAILURE` diagnostic, never a host abort). Spawn-per-call chosen
   over the regex-style long-lived worker DELIBERATELY: ~50–100 µs of
   thread setup is noise against ms-scale HDL file parses, and it preserves
   host-side parallelism (no serialization on a shared worker at the Nexsim
   boundary). **The regex path is byte-untouched** (keeps its RGX-0085
   64 MiB worker + `REGEX_MAX_NESTING_DEPTH` pre-check; the perf-floor law
   is not at risk — zero hot-path code changed, and the regex family's
   embedding/probe code paths are not modified at all).
   `EMBEDDING_API_VERSION` `1.3.0` → `1.3.1` (backward-compatible).
2. **`parseability_probe`** (`rust/src/bin/parseability_probe.rs`): whole
   `main` body wrapped (one spawn per process; thread-local trace/dump
   config and parses stay on one thread).
3. **`ast_pipeline`** (`rust/src/main.rs`): whole `main` body wrapped —
   every driver (parse, AST dump, cert coverage, stimuli replay, regen)
   inherits the guaranteed stack in both the `ast_pipeline` and
   `ast_pipeline_bootstrap` binary configurations.

Regression locks added (`rust/src/embedding_api.rs` tests): deep-parens SV
(N=2000, `sv_2017` + `verilog_2005`) and deep-nested VHDL through the PUBLIC
embedding entries on a 2 MiB libtest thread — pass only if the routing is
real; over-deep input must come back `E_PARSE_FAILURE`, never a process
abort.

## Measured before → after

BEFORE (banked in the measurement doc, `PGEN-SV-CORPUS-GRAD-0014`):

| input | build | stack | outcome |
|---|---|---|---|
| deep-parens N=2000 | debug | 8/16/32 MB | rc 134 SIGABRT |
| deep-parens N=2000 | release | 8 MB (default) | rc 134 SIGABRT |
| deep-parens N≈50 | debug | 8 MB | CRASH (≈180 KB/paren) |
| deep-parens N≈400 | release | 8 MB | CRASH (≈21 KB/paren) |
| br_gh330.v | debug | 8 MB | rc 134 (the v2005 lane's 1 `crash` row) |

AFTER (this slice; oracle `run_oracle.sh`, output banked below):

- **I1 — no signal deaths anywhere**: every probe invocation exits 0 or 1;
  rc 134 is gone in BOTH build modes.
- **I2 — the ceiling now bounds the parse**: deep-parens N=2000 is REJECTED
  (rc 1, finite time) under both profiles in both modes. The rejection's
  `furthest_position=702` ≈ 681 parens ≈ 681 × ~6 frames/paren ≈ 4096 —
  the arithmetic signature of the 4096-frame ceiling firing (verilog_2005
  cascade ≈6 logical frames/paren; sv_2017 ≈11/paren, so its boundary sits
  lower — the accept/reject boundary is profile-dependent BY DESIGN, which
  is why N=380/500 rows are recorded observations, not pins).
- **I3 — br_gh330.v ACCEPTS rc 0** in both modes and both profiles: the
  v2005 lane's single `divergence:unexplained_crash` row converts to a
  measured match (150 → 149 unexplained; manifest re-adjudicated below).
- Embedding locks: the new lib tests pass on 2 MiB libtest threads (debug
  build — the ≈70 MB-per-4096-frames worst case), proving the routing.

Honest note on the surfaced text: the over-deep rejection surfaces as the
generic `Parser did not consume full input … furthest_position=…` message —
the `RecursionDepthExceeded` error participates in backtracking like any
branch failure, so the FINAL reported error is positional. The class fix is
complete (graceful bounded rejection, never a process abort); making the
ceiling error surface preferentially in the message is engine
error-priority work, noted as possible future polish, deliberately NOT done
here (boundary-locus slice; the stable engine stays untouched per the fix
hierarchy).

## Oracle output (both modes)

See `8c3_oracle_run.txt` alongside this doc (generated by the scratch
`run_oracle.sh`; inputs: synthetic `module m; assign x = (((…1…))); endmodule`
at N=380/500/2000 plus the real `br_gh330.v`).

## v2005 lane re-adjudication

The v2005 lane re-ran (`stimuli/run_external_corpus.sh sv2005`, rebuilt
debug probe) and the adjudicator re-emitted the manifests. The regenerable
main-lane raw dump `results.tsv` (gitignored, absent on disk) was
reconstructed losslessly from the TRACKED `adjudication_manifest.tsv`
(suite/relpath/observed are the raw dump's exact content); the adjudicator
re-deriving the main manifest BYTE-IDENTICAL from that reconstruction is the
soundness self-check (it re-runs the full answer-key derivation over the
same observed data). Result: the br_gh330 `crash` row → `match`
(`observed=pass`), v2005 unexplained 150 → **149** (135 rejects-valid / 14
accepts-invalid / **0 crash**); all other rows byte-stable; the main
sv_2017 manifest byte-identical (564 baseline untouched).
