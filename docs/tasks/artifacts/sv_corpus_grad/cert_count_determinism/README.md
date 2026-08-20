# `certificate_coverage()`'s rule COUNT — is it deterministic? (`SV-CORPUS-GRAD.13c.2x.1`)

## The observation this adjudicates

`.13c.2x` measured `sv_cert_recognized_union_gate` on `.13c.2v`'s `SV-0065` arm and read

```text
canonical total = 1434   (seed 0)
canonical total = 1433   (seeds 7 and 42)
seed=7 signature drift vs seed 0
```

At HEAD all three seeds agreed at 1433. The arm adds exactly **one** grammar rule, so the reading
that fell out of the numbers was *"the new rule was counted at seed 0 and not at seeds 7/42"* —
which no rule count should permit, because `certificate_coverage()`
(`rust/src/ast_pipeline/grammar_wellformedness.rs:2240`) is **pure** and its `total` is
`grammar.rule_order.len()`, a function of the loaded grammar and of nothing else. No witness-search
seed is in scope.

`.13c.2x.1` recorded **two hypotheses and adopted neither**, which is what this probe exists to
settle:

| | hypothesis | blast radius if true |
|---|---|---|
| **H1** | per-PROCESS nondeterminism — the gate runs each seed in a separate process, so "seed 0 vs seed 7" and "process A vs process B" are perfectly confounded in that evidence | **every rule-count baseline in the repository** is pinning noise |
| **H2** | genuine seed-dependence — something in the seeded path appends to `rule_order` | scoped to the seeded path |

## What the probe does

It de-confounds the two axes the original evidence could not separate:

| axis | what varies | what is held | reads on |
|---|---|---|---|
| **A** | `PGEN_X1_REPEATS` separate PROCESSES | the seed, fixed at the contract's first seed | H1 |
| **B** | one process per declared seed | everything else | H2 |

**Varies on A ⇒ H1.** **Stable on A while B moves ⇒ H2.** **Stable on both ⇒ neither**, and the
original variance is not a property of `certificate_coverage()` at all.

## Two design choices that are load-bearing

- ⛔ **It runs the GATE'S OWN configuration, not a simplified one.** The number under test was
  produced by `sv_cert_recognized_union_gate.sh` with a profile, an entry rule, 40 samples and four
  `--cert-union-config` configs. A probe that dropped them would measure a different quantity and
  could agree with the gate by accident. Every parameter is **read from the contract the gate
  reads** (`rust/test_data/grammar_quality/systemverilog_recognized_cert_union_contract.json`) and
  none is typed into the probe — a probe that hard-coded them would keep passing after the contract
  moved.
- ⛔ **The repo root is found by WALKING UP to a sentinel, never by counting `..`.** Three probes in
  this repository have now shipped with the wrong depth — `accepted_rise_gate`'s, `.13c.2w`'s
  containment probe, and this one's first draft, which refused with
  `probe: not at the repo root (…/docs)` on its first execution. Each was caught by its own
  pre-flight, so no phantom result was ever scored, but a defect that recurs three times is a
  **class**, and a depth literal is exactly what rots when a probe is moved one directory.

## Running it

```bash
bash docs/tasks/artifacts/sv_corpus_grad/cert_count_determinism/probe.sh
```

Exit `0` = invariant on both axes · `1` = a divergence (that IS the finding) · `2` = refused
(harness pre-flight failed, so nothing was scored).

## Relationship to the sibling probe

`docs/tasks/artifacts/sv_corpus_grad/rule_count_determinism/probe.sh` (`-0252`) asks the same
question of the **syntax-probe gap report**, a *different* instrument counting the *same*
post-elimination population, and found it invariant on both axes at HEAD. That removed the
repo-wide half of H1 but explicitly could **not** close this leaf: it is not
`certificate_coverage()` and it did not run on the exhibiting arm. This probe closes exactly that
gap.

---

## Result — BOTH recorded hypotheses are REFUTED on the exhibiting state

Run on the `SV-0065` arm itself (`grammars/systemverilog.ebnf` = `5502bf28…`, the banked patch
applied), which is the state the original variance was observed on.

| axis | processes | seed(s) | canonical `total` |
|---|---|---|---|
| **A** (H1) | 3 separate processes | fixed at 0 | **1434 · 1434 · 1434** |
| **B** (H2) | 1 process each | 0 · 7 · 42 | **1434 · 1434 · 1434** |

Every run also agreed on `proof=8 witness=1361 UNKNOWN=65 sample_parse_failures=0`.

- **H1 — per-process nondeterminism: REFUTED.** Three processes at one seed produced the identical
  count. (`-0252` had already refuted the repo-wide half of H1 on a *different* instrument counting
  the same synthesized population; this closes the half that instrument could not reach.)
- **H2 — real seed-dependence: REFUTED.** Three seeds produced the identical count. The
  1434 / 1433 / 1433 pattern does **not** reproduce.

⇒ **The variance is not a property of `certificate_coverage()` at all**, and neither hypothesis the
leaf recorded survives.

### The control — the instrument CAN see the difference it failed to see

Run with **identical flags** on HEAD's grammar (`b0395cc8…`), so arm and control differ in nothing
but the grammar file:

| grammar | seed 0 | seed 7 | seed 42 |
|---|---|---|---|
| HEAD `b0395cc8…` | `total=1433 proof=8 witness=1361 UNKNOWN=64` | same | same |
| arm `5502bf28…` | `total=1434 proof=8 witness=1361 UNKNOWN=65` | same | same |

⭐ **The control does two jobs, and the second is the one that matters.** It proves the probe can go
RED — an instrument that could not tell 1433 from 1434 would have scored "stable" for the wrong
reason. And it lands on `1433 / 8 / 1361 / 64`, which is **exactly** what `.13c.2x` independently
recorded for HEAD before this leaf existed: an oracle this session did not build, agreeing to the
field.

⭐ **Three independent instruments each report exactly +1**, by three different code paths — so the
count is not merely repeatable, it is *correct*:

| instrument | population it counts | HEAD | arm | Δ |
|---|---|---:|---:|---:|
| `--report-certificate-coverage` `total` | `grammar.rule_order` | 1433 | **1434** | +1 |
| `--lint-grammar` rule count | post-elimination rule set | 1610 | **1611** | +1 |
| `parse_cost_containment.py --graph` | frontend `raw_ast` reference graph | 1483 | **1484** | +1 |

⛔ Every cell above was derived in THIS session by running the named command against the two
grammar files — none is carried from another leaf's record. The three populations differ in size
(1433 / 1610 / 1483 at HEAD) because they count different things; what makes them a cross-check is
that all three move by exactly **+1**, through three unrelated code paths, for a patch that adds
exactly one rule.

## So what DID produce 1434 / 1433 / 1433?

⛔ **A third possibility the leaf could not see, because it is not about the tool.** The gate
re-reads `$GRAMMAR_FILE` **once per seed**, and its determinism signature is

```text
canon_total|canon_proof|canon_witness|canon_unknown|canon_spf|union_total|union_proof|union_witness|union_unknown|residual
```

— ten OUTPUT fields and **no input identity whatsoever** (`sv_cert_recognized_union_gate.sh:307`).
The only identity call in that gate is at line 177, *before* the loop opens at line 255, and it
verifies the **contract**, not the grammar. So `signature drift vs seed 0` is exactly as consistent
with *"the grammar file changed under me between iterations"* as with *"the tool is
nondeterministic"* — and a multi-seed run takes minutes, while the session that produced the
observation was applying and reverting an experimental arm throughout.

⭐⭐ **The gate's own arithmetic corroborates it.** `.13c.2x` recorded `unmet_criteria_count=29` on
the arm and **27** at HEAD, and described 27 as *"nine criteria failing on each of three seeds"*.
If all three seeds had seen the arm, the same nine would fail on each — still 27, with no drift
entries. The observed **+2** is exactly the two drift entries the loop appends, one for seed 7 and
one for seed 42. ⇒ **seed 0 saw a different grammar from seeds 7 and 42.**

⚠️ **HONEST BOUND, and it is the whole reason this is written as a mechanism and not a verdict.**
Nothing can prove *after the fact* which bytes were on disk during a run that recorded no input
identity — that is precisely the defect. What is proven here is (i) the tool is deterministic on
both axes, so no tool-side explanation survives, and (ii) the gate cannot distinguish the two
causes. Whether this instance was caused that way is unfalsifiable; that it *can* be is measured.

## The class — 5 gates, 5 blind

`gate_input_pin_census.sh` (output in `gate_input_pin_census.txt`) counts gates that assert
determinism across re-reads of a mutable input, and whether any records that input's identity
inside the comparison loop. At the opening measurement:

```text
GATE-INPUT-PIN-CENSUS: asserts_determinism=5 blind_to_input_change=5
```

✅ **After `(d)` — the SV cert gate now pins its input** (`PGEN-SV-CORPUS-GRAD-0258`):

```text
sv_cert_recognized_union_gate.sh    6 ref(s)   1 ref(s)   pins
GATE-INPUT-PIN-CENSUS: asserts_determinism=5 blind_to_input_change=4
```

⭐ **The census was NOT modified to produce that.** Its predicate scans from the loop opener down,
so the first cut — which hid the digest behind a `sha_of()` helper defined *above* the loop — still
read **BLIND**. The two options were to move the code or to teach the census about the helper, and
teaching the measure to recognise the change it is measuring is the defect rather than the fix. The
digest is therefore taken **inline at the point of use**, and the unmodified instrument sees it.

Three of the five share the *identical* `first_seed_signature` idiom —
`sv_cert_recognized_union_gate`, `rtl_const_expr_cert_gate`, `verilog_2005_conformance_gate` — and
two more (`ast_dump_contract_gate`, `duality_hunt_gate`) implement the same shape as a re-run
tripwire. ⚠️ The census counts REFERENCES: it sizes the population and adjudicates no individual
gate, the same bound `.13c.2x`(d) put on its fifteen-baseline sweep.

⭐ Measured scope of the blast radius so far: `grep -rn "signature drift"` over the whole tracked
record returns **one** hit — this observation. The blind spot is LATENT in the other four and has
fired exactly once, which is why this is a guard to build rather than a backlog to re-adjudicate.
