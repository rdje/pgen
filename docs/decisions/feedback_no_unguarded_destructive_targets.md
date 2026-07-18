# ⛔ No unguarded destructive targets — destructive operations REFUSE without explicit confirmation

- **Date / source:** 2026-07-19, director (emphatic, in direct response to the 2026-07-18
  `make annotation_parsers` incident report): *"Please, please, please, make this sort of
  incident does not happen again."*
- **Status:** binding, mechanically enforced (OPS-MEMSAFE.3).

## Context (the incident)

During the `PGEN-RGX-0078-0141` regen wave, the engineer invoked
`make -C rust annotation_parsers` — a target whose NAME promises "regenerate the
annotation parser pair". It was an alias for `return_semantic_parsers`, whose recipe
hard-depends on `clean`: it deleted **every** generated artifact (`generated/*`) **and ran
`cargo clean`**, destroying **102.8 GiB** — the entire `rust/target/` tree including every
preserved perf-probe binary from all previous campaign slices. Recovery required a full
cold bootstrap plus two cold fat-LTO probe rebuilds (~2 h of wall time). No *durable*
state was lost — banked numbers in `docs/tasks/artifacts/` are git-tracked, and the
rebuild+floor-validation custody method worked — but the incident class is unacceptable.

## Decision

1. **Destructive build targets refuse by default.** The `clean` recipe (and therefore its
   whole dependent family: `clean-all`, `rebuild`, `return_semantic_parsers`,
   `bootstrap-test`) exits 96 with a loud, self-explaining message unless
   `PGEN_CONFIRM_CLEAN=1` is set. Destruction is now an explicit, deliberate act — no
   alias, dep chain, script, or habit can trigger it accidentally.
2. **No innocuous-sounding alias may route into the destructive family.**
   `annotation_parsers` now points at the two direct, non-destructive per-parser flows.
3. **Mechanical enforcement:** `scripts/check_destructive_target_guard.sh` (registered in
   `scripts/check_doctrines.sh`, so pre-commit E3 + CI E4 run it) structurally asserts:
   the guard exists in `clean:` BEFORE any destructive line; `annotation_parsers` does not
   depend on the destructive family; the set of targets depending on `clean`/`clean-all`
   equals the explicit allowlist. The check caught a second latent member
   (`bootstrap-test: clean-all`) on its very first run.
4. **Preserved binaries live OUTSIDE the blast radius.** Probe binaries whose custody
   matters are parked in the top-level gitignored `preserved_probes/` (named
   `<bin>_<slice-tag>_<sha8>`), which no `cargo clean` touches. `rust/target/generated_logs/`
   is henceforth scratch-only.
5. **Process rule (belt on top of the mechanical suspenders):** before invoking ANY make
   target for the first time, inspect its full recipe + dep chain (`make -n <target>` is
   free). Recorded in `MEMORY.md` and the session-facing memory.

## Consequences

- A legitimate full clean is still one command — just an explicit one:
  `PGEN_CONFIRM_CLEAN=1 make -C rust clean`.
- Any future destructive target must be added to the check's allowlist deliberately, with
  this record cited — silent growth of the destructive family fails the doctrine gate.
- **Why:** the director must never again learn that hours of preserved build state
  vanished because a target's name undersold its recipe.
- **How to apply:** never weaken the guard; extend the allowlist only with a documented
  decision; park anything preservation-worthy outside `rust/target/`.
