# `SV-CORPUS-GRAD.13c.2x`(a) — the per-criterion adjudication of `sv_cert_recognized_union_gate`

Evidence for the ruling that HEAD's 27 unmet criteria are **not** uniformly
*correct-and-stale*: one of them — `union UNKNOWN 0 → 53` — is the visible shadow of a
live **over-rejection + over-acceptance** defect in `grammars/systemverilog.ebnf`.

Everything here is re-runnable from the repository root. The measuring binary is
`rust/target/release/parseability_probe`, whose embedded SystemVerilog parser digest was
verified against the tree before use:

```bash
./rust/target/release/parseability_probe --parser-fingerprint | jq -r .parsers.systemverilog
# bfaca030172cd7dd1c66f4aeb586e026d3df7affdf107d80c9e4604148bcb8c5
shasum -a 256 generated/systemverilog_parser.rs
# bfaca030172cd7dd1c66f4aeb586e026d3df7affdf107d80c9e4604148bcb8c5   ← the same parser the
#   union contract's BASELINE-IDENTITY block records as its input
```

## Contents

| path | what it is |
|---|---|
| `sva_probes/*.sv` | **21** minimal IEEE 1800-2017 A.2.10 property probes, one construct each |
| `probe_matrix.tsv` | every probe's verdict under `sv_2017` and `sv_2023` — **15 ok, 3 over-rejection, 3 over-acceptance** |
| `implies_trace.txt` | the scoped `--trace-rules implies` excerpt that names the cause |
| `head_measurement.txt` | the three-seed cert-union tuple at HEAD, with the 53-name residual's sorted sha256 per seed and the roster classified |
| `ab_preflip.sh` + `attribution_arm.txt` | the A/B arm that splits the drift between the grammar and the engine — same binary, same grammar, indirect-LR pass held off. **Re-runnable from its tracked location**: `bash docs/tasks/artifacts/sv_corpus_grad/cert_union_adjudication/ab_preflip.sh` (130 s) |

Re-run the matrix (paths are repo-root-relative; each probe takes a few seconds):

```bash
for f in docs/tasks/artifacts/sv_corpus_grad/cert_union_adjudication/sva_probes/*.sv; do
  for p in sv_2017 sv_2023; do
    ./rust/target/release/parseability_probe --parse systemverilog "$f" --profile "$p" >/dev/null 2>&1 \
      && echo "ACCEPT $p $(basename "$f")" || echo "REJECT $p $(basename "$f")"
  done
done
```

⛔ The **three over-acceptance rows are as load-bearing as the three over-rejections**: `arrow_prop`
(`a -> b`), `arrow_unary_prop` (`-> b`) and `or_assign_prop` (`a |= b`) are constructs IEEE
1800-2017 A.2.10 does not define, and the parser takes all three. The 15 passing rows are the
control — same file shape, same profiles, same binary, one token different.

## The ruling in one line

`grammars/systemverilog.ebnf:6666` defines `implies := trivia "->"` — a token rule named
after an IEEE 1800 keyword — so every `prop_primary_sv_*` branch that the LRM spells with
the **keyword** `implies` is compiled against the **arrow** `->` instead.
