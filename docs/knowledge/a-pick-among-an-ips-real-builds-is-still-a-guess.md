---
id: a-pick-among-an-ips-real-builds-is-still-a-guess
title: A pick among an IP's real builds is still a guess — a derived build list applies only what every build agrees on and reports the rest per build (`SVPP-EXPANSION.4c.3`, the rule that parked `.6d.5`)
answers:
  - "OpenTitan builds an IP several ways — which build's +define+ does the pipeline lane apply to its files"
  - "why does the derived context not define EN_GCM / NUM_END_POINTS / SRAM_WORD_ADDR_WIDTH when the sim_cfg declares them"
  - "the deriver applied the first configuration by name — why was that withdrawn"
  - "how is a dvsim build list derived from *_sim_cfg.hjson (run cfg, base cfg, en_build_modes, build_mode)"
  - "what does a variant line in pipeline_context.py's report mean and what unparks the rows behind it"
  - "a lane chain was stopped mid-run — what keeps the movers census honest"
tags: [systemverilog, pipeline, context, opentitan, derivation, doctrine]
date: 2026-10-17
provenance: SVPP-EXPANSION.4c.3 (PGEN-SVPP-EXPANSION-0049); SVPP-EXPANSION.6d.5 (the parked row and its rule), .4c.1 (the tie rule for verilator drivers), .4b (the ambiguous-include rule)
reverify: "python3 stimuli/sv/pipeline_context.py --self-test   # PIPELINE-CONTEXT-SELF-TEST: checks=35 failed=0; python3 stimuli/sv/pipeline_context.py --check   # PIPELINE-CONTEXT: OK — the tracked context equals a fresh derivation; python3 stimuli/sv/pipeline_context.py --write 2>&1 | grep -c '  variant   '   # 23"
---

# A pick among an IP's real builds is still a guess

**The rule.** The derived context applies to a file only what its own harness provably compiles it with. For
OpenTitan that harness is dvsim, and dvsim declares per-IP defines per BUILD in the IP's `dv/*_sim_cfg.hjson`:
a cfg nobody imports is a run cfg; a cfg another cfg imports (`import_cfgs: ["{proj_root}/…"]`) is a base, run
only through its importers; a run cfg's default build carries the top-level `build_opts` of its import chain
plus the chain's `en_build_modes` (each a `build_modes` entry with `+define+NAME[=VALUE]` tokens); every
`build_mode:` a test of the chain picks is one more build on top of that. The deriver
(`opentitan_sim_cfg_defines` in `stimuli/sv/pipeline_context.py`) lists those builds per cfg directory, and a
define reaches the files the directory owns **only when every build of the directory carries it with one
value**. A name the builds disagree on — another value, or absent from one build — is written to the report as a
`variant` line with each build's value, and never applied. A templated value (`{tl_aw}`) is `computed`, never
emitted.

**The near-miss.** The first cut applied the first build by name and reported the others as "also …". For
`hw/ip/aes/dv` that build was `aes_gcm_masked` (`EN_GCM=1 EN_MASKING=1`); the lane under it moved 78 rows and the
aes package from a context refusal (`` `EN_GCM `` undefined) to a unit-level reject at `aes_base_vseq.sv:202:23`
— `foreach (key[0][i])`, a tool-tolerance shape the strict profiles reject — which two of the IP's four builds
never reach. Every one of the four builds is real; none of them is *the file's*. `.6d.5`'s parked row had
already said so in one sentence: *a disagreement between variants is reported, not resolved by guessing which
build the row meant.* The chain was stopped in its second reading, the rule written into the deriver and its
self-test (8 arms), and the readings re-run; the HEAD dumps copied aside before the first chain are what the
movers are diffed against, so the stop cost time and nothing else.

**What it measures.** 679 rows on 327 files in 4 directories agree (the two tops' `TOP_LEVEL_DV` /
`DISABLE_ROM_INTEGRITY_CHECK`, rv_dm's `USE_DMI_INTERFACE=1`, kmac's `SW_KEY_MASKED=0`); 13 directories disagree
on 23 names — exactly the population `.6d.5` listed by hand (aes `EN_GCM` 1 / 1 / 0 / 0, edn `NUM_END_POINTS` 8 /
1, entropy_src `RNG_BUS_WIDTH` 16 / 4, lc_ctrl `SEC_VOLATILE_RAW_UNLOCK_EN` 1 / 0, sram_ctrl
`SRAM_WORD_ADDR_WIDTH` 14 / 15 / 10, rom_ctrl `ROM_SIZE_BYTES` in three sizes, …) — so of OpenTitan's 145 context
refusals on an undefined macro, none is defined under the rule. **no row moved under any of the three readings** (`PIPELINE-MOVERS moved=0 added=0 lost=0` on `sv_2017`, `sv_2023` and `sv_2023_tolerant` against the HEAD dumps: the agreed defines reach no verdict — the 11 own-file candidates stop at another construct, kmac's `tb.sv` at the variant-only `` `EN_MASKING ``, the two tops' `tb.sv` at a compile-order macro, the five `fail` rows at their class header). What unparks those rows is
not a better pick but a per-variant READING — each row parsed under every build its directory declares, the
verdict their conjunction — routed as `SVPP-EXPANSION.4c.4`, the director's call the parked row names; the build
list that reading needs is now derived.

**Related.** [[an-accept-is-not-a-verdict-until-the-key-expected-it]] (the key side of the same discipline:
nothing is a verdict until something independent of the parser expected it);
[[a-second-profile-lane-is-a-differential-and-the-differential-is-the-finding]] (why the aes reject under one
build is a tolerance census row, not a grammar defect).
