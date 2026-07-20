# PGEN-RGX-0078-0194 result

## Verdict

**HOLD; no implementation.** A reference-forwarded maximum is lossless but
retains the same memory update. A register-resident by-value maximum requires a
transitive call/return protocol change whose costs are absent from the current
profile. Strict net is zero.

The 39 inner maximum loads provide only a **0.828243916 ns** gross ceiling.
Adding that impossible zero-cost view to the strict held stack leaves merely
**2.090166143 ns** of 30%-capture margin over noise.

## Reproduction

```bash
python3 docs/tasks/artifacts/position_progress_carrier/classify_position_progress.py \
  > /tmp/position_progress.txt
cmp /tmp/position_progress.txt \
  docs/tasks/artifacts/position_progress_carrier/position_progress.txt
```

## Scope

- Runtime/emitter/generated artifact and product behavior: unchanged.
- PCRE2 verdicts, accepted 1,263.4 ns geomean, and settled 483,583 ns MAX:
  unchanged.
- mdBook/contracts/reference architecture/LIVE tracker: unchanged; this is an
  internal read-only feasibility/pricing slice.
- Any parser-state split or transitive call/result experiment requires a later
  owned implementation leaf.
