# `SV-CORPUS-GRAD.10` — axis-2 re-measure evidence bundle

The FRESHNESS AUDIT (tree preamble) proved the tracked SV corpus report could not describe
`HEAD`; this bundle is the re-measure that answered it. Owning leaf: `docs/tasks/SV-CORPUS-GRAD.md`
`.10`.

## The driver

`analyze_transitions.py` — the per-FILE transition census between two `results.tsv` files. It
exists because a pass-COUNT delta nets a heal against a regression and reports zero
(`docs/knowledge/a-rising-pass-rate-is-not-evidence-of-correctness.md`); the question the campaign
needs answered is *which files changed verdict*, not *how many passed*.

It carries **ground truth** and both controls were run before any number here was quoted:

| control | construction | result |
|---|---|---|
| positive | identity self-join of the baseline against itself | 16 336 common, **0** changed, pass count reproduces the tracked `9 693` — which also proves the preserved baseline IS the artifact behind the tracked report |
| negative | one planted status flip in row 1 | caught **exactly once**, at exactly that file |

```bash
python3 docs/tasks/artifacts/sv_corpus_grad/axis2_remeasure/analyze_transitions.py \
    --before rust/target/sv_axis2_baseline/results.before.tsv \
    --after  stimuli/sv/characterization/results.tsv \
    --out    docs/tasks/artifacts/sv_corpus_grad/axis2_remeasure/transitions.txt
```

Column 3 is normalized to a repo-root-relative key on both sides, so an older artifact carrying
absolute paths still joins (before `CORPUS-GRAD-ALL.2.1` the runner emitted absolute paths).

## The banked censuses

| file | comparison | what it isolates |
|---|---|---|
| `transitions.txt` | tracked baseline → **the tracked HEAD run** (release probe, 60 s) | the headline: **0 pass→fail, 0 fail→pass** |
| `transitions_runA_debug20.txt` | baseline → debug probe, 20 s | reproduces the baseline's *conditions*, so any difference would be the parser |
| `transitions_runB_debug60.txt` | baseline → debug probe, 60 s | removes the per-file *budget* as a variable |
| `transitions_debug_vs_release.txt` | debug 60 s → release 60 s | removes the *build* as a variable |
| `transitions_v2005.txt` | baseline → HEAD, `verilog_2005` lane | the sibling profile lane (2 459 files) |

Across every one of them the pass/fail sets are identical and **only the `timeout` column moves**
(9 → 6 → 4 as the budget and then the binary stop being the limit). That is the measured basis for
the rule stated in the book (*The Gate Flow* §7.10): **a timeout is a statement about the
instrument, never about the parser.**

## The preserved raw runs

Under `rust/target/sv_axis2_baseline/` (untracked, on the repo volume — the runner OVERWRITES
`results.tsv` in place, so preserving it before a re-run is mandatory):

- `results.before.tsv` / `characterization.before.md` — the tracked 2026-07-25 baseline
- `results.head_t20.tsv` / `characterization.head_t20.md` — run A
- `results.head_t60_debug.tsv` / `characterization.head_t60_debug.md` — run B
- `results_v2005.before.tsv` / `characterization_v2005.before.md` — the v2005 lane baseline
- `systemverilog_parser.prev.rs` / `…_return_annotations.prev.json` — the pre-regen generated
  artifacts, kept to prove `focus_systemverilog` reproduces them byte-identically at `HEAD`
