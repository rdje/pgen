---
name: project-north-star
description: PROJECT (2026-08-09, MEMORY-ARCH.6) — the curated short list of the standing constraints every PGEN decision is measured against, demoted here out of the layer-A resume pointer. It is a CURATION, not a summary: each bullet is a pointer to the authoritative layer-C record, and its job is to say which of ~150 records bind on ordinary work. Goal, the two non-negotiable constraints, the Done bar, accuracy-before-speed, SV strictness, EBNF as sole source of truth, prior-art-before-design, every-finding-is-fixed, instruments need ground truth, data locality.
id: project-north-star
title: The curated short list of standing constraints every PGEN decision is measured against — a curation of ~150 records, not a summary
date: 2026-08-09
reverify: "grep -c '\[\[' docs/decisions/project_north_star.md"
answers:
  - "what are the standing constraints every decision in this project is measured against"
  - "which decision records actually bind on ordinary work here"
  - "what is PGEN's goal and what is it not allowed to trade away"
  - "can I add a capability if it costs a little speed or parser neutrality"
  - "when may I call a grammar family Done"
  - "should I optimize for speed or correctness first"
  - "may the SystemVerilog parser accept a construct the LRM does not derive"
metadata:
  node_type: memory
  type: project
---

⛔ **Pointers only — the linked record is authoritative in every case.** This file exists so that a
resuming agent can find the binding constraints without reading ~150 records, and so that the
layer-A pointer does not have to carry them (`MEMORY-ARCH.6`: the block was **2 273 bytes of a
7 168-byte cap**, and its lifecycle is layer C's, not layer A's).

- **The goal**: parse any precisely-described language with the right EBNF; the bar is *eagerness*,
  not sufficiency — expressive awkwardness is a defect class → [[project_horizon_universal_parser]].
- **The two non-negotiable constraints on reaching it**: parser-neutrality and theoretical peak
  speed. Capability that costs either is REJECTED, not traded; the mechanism is COMPILE AWAY
  (non-users pay ZERO) → [[project_capability_growth_is_zero_cost_and_neutral]].
- **`Done` is first-tier only** — three legs, all measured, never a snapshot;
  `Provisional (ceiling)` / `Provisional (corpus pending)` is the shipping tier →
  [[feedback_done_bar_is_first_tier_only]].
- **Accuracy is the immovable floor; speed is maximized on top** and monitored for every generated
  parser → [[feedback_correctness_before_speed]]; regex closure bar →
  [[project_rgx_0078_sub_1us_closure_target]].
- **SV is 100 % LRM-compliant by default** — over-acceptance is a defect, tolerance is additive and
  future → [[feedback_sv_strict_lrm_compliance_default]].
- **The EBNF is the sole source of truth** and steers the engine at the author's granularity →
  [[project_ebnf_is_single_source_of_truth]], [[project_ebnf_steers_the_engine_at_full_granularity]].
- **Prove no existing surface covers it before designing one**, and RE-MEASURE engine behaviour
  rather than quoting a doc → [[feedback_read_prior_art_before_designing]] (mechanized:
  `DESIGN-PRIOR-ART`).
- **EVERY issue raised must be FIXED** — routing decides WHEN, never WHETHER; a design is a
  schedule, not the deliverable → [[feedback_every_finding_must_be_fixed_not_logged]]; worked NOW
  only when it BLOCKS → [[feedback_flow_findings_are_routed_not_worked]].
- **An instrument with no ground truth is a confident guess** — pin both a positive and a negative
  control inside it and REFUSE on a miss → [[feedback_instrument_needs_ground_truth]].
- **Project data stays on the repo's volume** → [[project_data_locality_same_volume]]; reclaim
  artifacts routinely, proving safety each time → [[feedback_delete_reclaimable_artifacts_regularly]];
  the portable spine is a separate repo → [[project_bedrock_spine_repo]].
