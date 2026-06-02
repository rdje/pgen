<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_empirical_sweep_prebuild.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: Pre-build parseability_probe for empirical sweeps
description: When running an empirical sweep over many regex inputs, pre-build the probe binary; do not call `cargo run` per iteration.
type: feedback
originSessionId: f74f4acc-7183-408a-ae5d-dcdce15a103c
---
For empirical AST-shape sweeps over multiple regex inputs (typical pattern: 4-10 inputs in a `for` loop calling `parseability_probe`), do **not** invoke `cargo run --release --bin parseability_probe ...` per iteration. Each `cargo run` re-checks the build and adds ~10-15s overhead per invocation even when the binary is already compiled — a 6-input sweep can stretch from a few seconds (actual parse time) to several minutes.

Instead:

```bash
cargo build --release --features generated_parsers --bin parseability_probe
PROBE=/Users/richarddje/Documents/github/pgen/rust/target/release/parseability_probe
for input in ...; do
  printf '%s' "$input" > /tmp/in.txt
  rm -f /tmp/out.json
  timeout 10 "$PROBE" --parse-dump-ast-pretty regex /tmp/in.txt /tmp/out.json --profile regex_default
  ...
done
```

**Why:** Discovered during regex.ebnf slice 17 (property_escape). A 6-input `\p{...}` / `\pX` sweep using `cargo run` per iteration appeared to hang mid-loop and got killed. Re-running with the pre-built probe path completed all 6 in seconds.

**How to apply:** Any time the empirical sweep loop has ≥3 cargo-run invocations on the same binary, pre-build first and call the binary directly. Add a `timeout 10` (or appropriate) to each invocation as a safety net against actual parser pathologies.
