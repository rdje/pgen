# Vendored: the PGEN-RGX-0078 iteration flow (from RGX)

These files are the RGX-provided, PGEN-vendored compile-time perf gate for the
`PGEN-RGX-0078` bug report (source bundle:
`rgx/pgen-issues/artifacts/PGEN-RGX-0078/pgen_iteration_flow/`, vendored
2026-07-21 per the bundle's own README "Vendor these files into PGEN's tree").

- `../examples/pgen_pcre2_compile_ratio.rs` — the canonical Rust microbench:
  PGEN parse p50 vs `pcre2`-crate compile p50 on the frozen 8-pattern corpus;
  prints the ratio table + the `< 5x` closure verdict.
- `pcre2_compile_baseline.c` / `pcre2_compile_jit_baseline.c` — standalone C
  cross-checks (`-lpcre2-8`).
- `patterns.tsv` — the frozen corpus (stable since PGEN-RGX-0073).
- `run_perf_gate.sh` — one-shot driver (C baselines + the Rust microbench).

Run (from `rust/`): `make SHELL=/bin/bash regex_pcre2_compile_perf_gate`
or `bash perf/run_perf_gate.sh`. Requires system libpcre2-8 (brew install pcre2).
