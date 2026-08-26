---
name: size-the-population-by-provenance-before-choosing-gate-or-ratchet
description: "DISCIPLINE (2026-08-27, LIVE-DOC-CONTAINMENT.8 step 4 / PGEN-LIVE-DOC-CONTAINMENT-0015) — before making a new check BLOCKING, size its existing population and break it down by WHO AUTHORED IT. The total tells you the defect matters; only the provenance breakdown tells you what a blocking rule would do to the people it fires on. MEASURED: a markdown table-arity defect covers 51 of 1 167 tracked files (136 OVER + 49 UNDER). As a total that argues for a blanket gate. By class it argues the opposite and the two classes have OPPOSITE shapes: 22 machine-converted LRM files hold 92 % of the UNDER population (45 of 49) and nobody in the repository authored them, while 29 project-authored files hold 72 % of the OVER (98 of 136). A blanket gate would have fired mostly on documents no author could be asked to fix — the 91 %-false-positive shape this repository had already refused twice. The shape chosen instead was a RATCHET against git: staged content vs HEAD content of the same file, so a file already carrying defects may be EDITED but not WORSENED, and a NEW file is compared against zero because its author has just written every row. ⇒ a severity argument decides WHETHER to check; a provenance-sized population decides whether the check can BLOCK, and what its baseline is."
id: size-the-population-by-provenance-before-choosing-gate-or-ratchet
title: "Size the population by provenance before choosing between a gate and a ratchet"
date: 2026-08-27
evidence: "docs/tasks/LIVE-DOC-CONTAINMENT.md leaf .8, fields `Step_4_GATE_LANDED_2026-08-27` (the priced-and-refused blanket gate) and `Population_SIZED_repo_wide_2026-08-27` (the mid-repair sizing that produced the shape argument); the shipped enforcer scripts/check_table_arity.sh and its registry row in scripts/check_doctrines.sh"
reverify: "bash docs/tasks/artifacts/live_means_live/table_arity_population_sweep.sh   # 1.8 s over the whole tracked tree; prints files/OVER/UNDER per provenance class and the two shares"
answers:
  - "should my new check block or just warn"
  - "how do I decide between a blanket gate and a ratchet"
  - "what do I measure before making a lint blocking"
  - "my check would fire on hundreds of existing files, now what"
  - "why did a blocking rule teach people to bypass it"
metadata:
  node_type: memory
  type: feedback
  created: 2026-08-27
---

## The rule

Before a new check is allowed to **block**, size the population it would fire on **and break that
population down by who authored it**. A severity argument decides *whether to check at all*. The
provenance-sized population decides *whether the check can block, and against what baseline*.

## Why — the measurement

The defect was strong on severity: a markdown table row with more cells than its header has the
surplus silently discarded, and one such row sat in a contract shipped to a downstream consumer,
losing the column that carries the consumer-impact statement.

The population, as a total, said **51 of 1 167 tracked files, 136 OVER + 49 UNDER**. That reads like
an argument for a blanket gate against zero.

Broken down by provenance it says the opposite, and the two halves point in different directions:

| class | files | OVER | UNDER |
| --- | --- | --- | --- |
| machine-converted LRM text (nobody here authored it) | 22 | 38 | **45** |
| project-authored | 29 | **98** | 4 |

**92 % of the UNDER population is in documents no author in this repository can be asked to fix**,
while **72 % of the OVER is in text the project wrote**. A blanket gate would have spent most of its
firing budget on the half nobody owns — which is how a blocking rule teaches bypass rather than
compliance, and it is a shape this repository had already refused twice on false-positive grounds.

## The shape that survived

A **ratchet against git**: staged content vs `HEAD` content of the same file.

- a file already carrying defects may be **edited**; it may not **gain** another;
- a **new** file is compared against **zero**, because its author has just written every row;
- there is no tracked baseline to go stale, no `--rebaseline` step, and no identity block to verify.

That makes blocking affordable on day one without repairing 51 files first — while still closing the
door on the next one. The two files the defect was found in were repaired to zero *before* the gate
shipped, so it fires on nothing at the moment it lands.

## The transferable shape

Whenever a check's population is mostly **inherited** rather than **authored**, a ratchet is the
honest shape and a gate is a bypass generator. The provenance split is what distinguishes the two
cases, and a total can never show it. Common inherited classes: vendored or converted third-party
text, generated artifacts, historical migrations, and anything under a directory whose contents are
produced by a tool rather than typed.

⚠️ Note what is NOT the lesson: *"too many existing violations, so warn instead of blocking"*. That
gives up. The ratchet blocks — it just measures the delta rather than the level.

Related: [[the-optional-half-of-a-delimiter-is-where-a-counter-goes-blind]] and
[[a-delimiter-imbalance-is-not-a-local-defect]] — the defect this gate was built for, and why its
population was mis-measured twice before it was sized correctly.
