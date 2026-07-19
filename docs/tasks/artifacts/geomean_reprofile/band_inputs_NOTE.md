# Band input JSONLs — derivable, not banked

The three band workload inputs (`band_sub1us.jsonl` 939 cells, `band_1to2p5.jsonl`
751, `band_2p5to20.jsonl` 488) are byte-derivable and therefore not banked:

- source corpus: `regex_corpus_bundle/corpus/pcre2/canonical/pcre2_compile_oracle_cases.jsonl`
- selection: filter cells whose banked per-cell `min_ns` (from
  `../k4b_delta/c1_ab/corpus_cand.jsonl`, matched on `id`) falls in
  [0,1000) / [1000,2500) / [2500,20000) ns respectively, preserving corpus order
  and the full original case lines. The 11 cells ≥ 20,000 ns (0.8% log-share)
  are excluded by design (see `step0_analysis.md`).

The band OUTPUTS (`band_*_times.jsonl`, `band_*_stdout.txt`) and the `sample`
captures are the primary evidence and ARE banked.
