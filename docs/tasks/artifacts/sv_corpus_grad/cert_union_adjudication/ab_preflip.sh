#!/usr/bin/env bash
# SV-CORPUS-GRAD.13c.2x(a) — THE ATTRIBUTION ARM.
# Same cert-union invocation as the shipped gate, seed 0 only, with the indirect-LR pass
# HELD OFF (`--no-eliminate-indirect-left-recursion`, the A/B measurement lever built by
# ENGINE-UNIVERSAL-SERVICES.13 — not by this slice). The contract's baseline was derived at
# 3056381a (2026-08-12), a commit at which rust/src/ast_pipeline/indirect_lr_elimination.rs
# DID NOT EXIST, so this arm is the faithful "before" for the engine axis.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
OUT="$ROOT/rust/target/sv_cert_union_adjudication"
BIN="$ROOT/rust/target/debug/ast_pipeline"
G="$ROOT/grammars/systemverilog.ebnf"
echo "==> preflip arm seed=0 grammar_sha=$(shasum -a 256 "$G" | awk '{print $1}')"
PGEN_CERT_COVERAGE_DUMP_ALL=1 "$BIN" "$G" --report-certificate-coverage \
  --no-eliminate-indirect-left-recursion \
  --grammar-profile sv_2017 --entry-rule systemverilog_file \
  --count 40 --seed 0 \
  --cert-union-config systemverilog_file:sv_2023 \
  --cert-union-config sv_multi_entry_root:sv_2017 \
  --cert-union-config library_text:sv_2017 \
  --cert-union-config systemverilog_parseable_file:sv_2017 \
  >"$OUT/cert_preflip_seed_0.log" 2>&1
grep -E '^CERTIFICATE-COVERAGE' "$OUT/cert_preflip_seed_0.log" | cut -c1-260
echo "PREFLIP ARM DONE"
