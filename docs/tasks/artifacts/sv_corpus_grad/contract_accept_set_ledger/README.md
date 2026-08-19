# `SV-CORPUS-GRAD.13c.2l` — which grammar revisions are consumer-visible, and how do we know?

The SV downstream contract was **seven grammar revisions stale**. This directory holds the two
instruments that derived that number, the measured result, and the adversarial probe for the gate
that now prevents it recurring.

| file | what it is |
|---|---|
| `measure_accept_set_transitions.py` | the two-axis sweep: every pinned reproducer × every profile it declares × every grammar revision since the contract's last commit |
| `accept_set_transitions.tsv` | its output — one row per moved witness, attributed to the commit that moved it |
| `probe_contract_currency.sh` | the adversarial probe for `scripts/check_sv_contract_currency.sh` — seven arms, each firing the tier it exists to refuse, plus a control |

## The finding, in one table

Two instruments that share no parent, agreeing 9/9 on the partition of the nine grammar commits in
`438c475c..HEAD`:

| commit | slice | semantic digest moved? | widen | narrow | **shape** |
|---|---|---|---|---|---|
| `6d1a18b3` | `-0220` | YES | (unpinned) | 0 | **7** |
| `cb78d01f` | `-0221` | YES | 15 | (unpinned) | 0 |
| `39d281ff` | `-0227` | YES | 10 | 0 | **3** |
| `2b949800` | `-0229` | YES | 4 | 0 | 0 |
| `4a2703cf` | `-0232` | YES | 1 | 1 | 0 |
| `f9cff55e` | `-0233` | YES | 0 | 0 | **12** |
| `6ad18e48` | `-0234` | **NO** | 0 | 0 | 0 |
| `fc6aa8f9` | `-0237` | YES | 0 | 16 | **2** |
| `e28cc856` | `-0238` | **NO** | 0 | 0 | 0 |

## Why there are two axes

⛔ The first cut of the sweep measured **verdicts only** and reported `-0233` as *"no verdict moved
(comment-only)"*. `bufif0 g(o, i, e);` parsed before that fix and parses after; what changed is that
it became a `gate_instantiation` instead of a `udp_instantiation`. **Twelve pinned witnesses moved
and zero verdicts did** — so the largest consumer break in the batch was invisible to the instrument
built to find consumer breaks. That is the same lesson `run_adjudication_repros.py` grew its `arm`
column for, arrived at independently, one level up.

## Why two instruments rather than one run twice

`docs/CLAIM_VERIFICATION.md`: *checking a claim twice does not make it twice as verified.* The two
here fail differently on purpose.

- The **semantic digest** (`ast_pipeline --emit-raw-ast-json`, hashed over `raw_ast`) is derived
  from the PRODUCER and is blind to behaviour: it says a revision *can* be observed, never that
  anything observed it.
- The **behavioural sweep** is blind to grammar text and bounded by what is pinned: it says a
  witness moved, and cannot see a change nobody pinned.

Their agreement is evidence. Their *disagreement* would have been the more interesting result, and
in one place it nearly was: `-0220` moved the digest and no witness, which is how the missing
`fixed_cross_body_function_lrm_19_6_1.sv` reproducer was found.

## Honest bounds, printed rather than implied

- The sweep's input population is the **pinned manifest**, not the language. `widen=0` means *no
  PINNED witness moved*, never *nothing changed*.
- Historical revisions are measured on the **interpreter**, because the shipped parser would need a
  ~22-minute regeneration per commit. The script therefore re-runs every row at HEAD through the
  shipped release probe and treats any disagreement as a hard error — it agreed **151/151**. The
  interpreter's one measured hole is un-eliminated left recursion, and `--lint-grammar` reports
  `left_recursion_unhandled=0` for this grammar.
- `--emit-raw-ast-json` must be given a **file**, never `/dev/stdout`: the tool writes progress to
  stdout too, so the stream carries valid JSON followed by more text.

## Re-run

```bash
python3 docs/tasks/artifacts/sv_corpus_grad/contract_accept_set_ledger/measure_accept_set_transitions.py
bash    docs/tasks/artifacts/sv_corpus_grad/contract_accept_set_ledger/probe_contract_currency.sh
bash    scripts/check_sv_contract_currency.sh
```

The sweep takes ~7 minutes at `PGEN_ACCEPT_SET_JOBS=8`; the other two are instant.
