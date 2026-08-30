---
id: an-instrument-inside-its-own-measurement-window-measures-itself
title: An instrument inside its own measurement window measures itself — vary the SPELLING, hold the VALUE, to find out
answers:
  - "my measurement moved and no source changed — where do I even start"
  - "the same input reports a different number depending on how I invoke the tool"
  - "is this small constant difference a real perturbation or an artifact"
  - "how do I test whether a numeric flag acts through its value or its representation"
  - "my instrument is deterministic — is that determinism a property of the subject or of the run"
  - "a diff moved one counter by a constant across every row — what does that mean"
  - "how do I know my probe's own startup is not inside what it reports"
  - "is peak memory a pure function of the input"
tags: [instruments, measurement, controls, evidence, allocators, determinism, argv]
date: 2026-08-29
status: current
evidence: "ENGINE-UNIVERSAL-SERVICES.76 (PGEN-ENGINE-UNIVERSAL-SERVICES-0134). rust/src/bin/parse_alloc_census_probe.rs reports peak_live — the headline quantity behind .67's parse-memory ingestion bound — as a peak of live heap bytes that INCLUDES the process's own argv Strings. On a fixed input, peak_live = 2344413 + len(argv value of --tape-min-bytes), exactly 1 byte per character, and the input PATH enters at 2 bytes per character (tmp/eus67/floor.sv 18 ch -> 2344418; tmp/kc/peak_argv.sv 19 ch -> 2344420; ./tmp/eus67/floor.sv 20 ch -> 2344422; the absolute path 53 ch -> 2344488). The effect is 0.004 % of the peak and does NOT disturb .67's bound, which stands; what it refutes is that peak_live is a function of (input, grammar, profile) alone. Found by chasing a uniform -1 that an orphaned background job's re-run left on 11 of 11 rows while peak_at_event_seq and peak_at_alloc_calls were identical on 11 of 11."
reverify: "mkdir -p tmp/kc && printf 'module m; endmodule\\n' > tmp/kc/peak_argv.sv && for v in 8192 08192 16384; do printf '%-6s ' \"$v\"; PGEN_REPORT_PARSE_FOOTPRINT=1 rust/target/release/parse_alloc_census_probe systemverilog tmp/kc/peak_argv.sv --profile sv_2017 --tape-min-bytes $v 2>/dev/null | grep -o 'peak_live=[0-9]*'; done   # 8192 -> ...419, 08192 -> ...420, 16384 -> ...420. 08192 and 16384 are DIFFERENT thresholds with an IDENTICAL peak; 08192 and 8192 are the SAME threshold with different peaks. The varying quantity is the STRING, not the number."
---

**A number your instrument reports about a subject is only about that subject if the instrument's
own existence sits outside the window it measures. Process startup — argv, environment, the paths
you typed — is inside the window unless someone deliberately put it outside.**

## The measured instance

An allocation-census probe reports `peak_live`: the high-water mark of live heap bytes during a
parse. It is the quantity a whole leaf's memory bound rests on, and it is prized precisely because
it is **deterministic** where peak RSS is not — an earlier slice used its byte-identical
reproducibility across two runs as the control proving RSS noise was noise.

Then the same 11 inputs re-ran and every single `peak_live` fell by **exactly 1 byte**.

What made that worth chasing rather than shrugging at was the shape of the constant:

- `peak_at_event_seq` and `peak_at_alloc_calls` were **identical on 11 of 11 rows** — the peak
  happened at the same instant, after the same number of allocations.
- The one configuration difference between the runs changed the number of recorded events by
  **+28 on one input and +220 797 on another** — a range of nearly 8 000×.

**A perturbation scales with the thing that varies. An offset does not.** A −1 that is identical
across an 8 000× range is not the recording disturbing the parse; it is something constant being
counted, and constants do not come from the subject.

## The control that settled it in one line

The suspect parameter was a threshold passed as `--tape-min-bytes 16384` versus `8192`. Two
hypotheses: the *value* changes what the parse does, or the *string* is being allocated and counted.

Both are consistent with `16384` → X and `8192` → X−1. So separate them: **hold the value fixed and
change only its spelling.**

| argv value | chars | threshold | `peak_live` |
|---|---|---|---|
| `8192` | 4 | 8192 | 2 344 417 |
| `16384` | 5 | 16384 | 2 344 418 |
| `08192` | 5 | **8192** | **2 344 418** |
| `000008192` | 9 | 8192 | 2 344 422 |
| `0000000016384` | 13 | 16384 | 2 344 426 |

`08192` and `16384` are **different thresholds with an identical peak**. `08192` and `8192` are the
**same threshold with different peaks**. The value is irrelevant; the character count is everything.
That is not weak evidence for the argv hypothesis — it is a **refutation** of the other one.

The same probe turns out to carry the input path at 2 bytes per character, so *the same file,
parsed identically, reports a different peak depending on how you spell its path* — and any
cross-file comparison silently compares path lengths alongside parse costs.

## The rule

- **Leading-zero trick.** To test whether a numeric parameter acts through its value or its
  representation, re-spell the value without changing it. Any difference is representational. This
  is cheap, decisive, and needs no instrumentation of the tool.
- **A constant offset is a different animal from a perturbation.** Before explaining a small
  uniform delta as noise or as disturbance, check whether it varies with the thing you changed. If
  it does not, stop looking at the subject and start looking at the instrument.
- **Give an instrument an explicit measurement epoch.** Latch the counter after argument parsing and
  report relative to that baseline, so the reported quantity is a function of the work and not of
  the command line. Re-baselining is the cost; a quantity that means what it says is the payoff.
- **"Deterministic" is a claim with an unstated scope.** This probe *is* byte-identical run to run —
  as long as argv never changes. Determinism observed while holding something fixed is evidence
  about the held-fixed configuration, not about the subject.

✅ **The accounting closed, and how it closed is part of the lesson.** As first written this card
carried an honest limit: *"48 bytes remain unattributed"*. That caveat had an unmeasured term of its
own — it compared a flag REMOVAL against a rate measured only by varying a flag's VALUE, so the cost
of an argv ENTRY had never been measured at all. Measuring it settled the whole thing in one step:
**adding** a 19-character flag pair costs **+67 B**, **removing** a 21-character one saves **−69 B**,
and `67 − 19 = 69 − 21 = 48` — the same constant from two independent directions. So the model is

```text
peak_live = base + Σ over argv entries ( ~24 bytes of Vec<String> slot + len(entry) )
```

which also explains the asymmetry the first cut could not: changing a flag's value varies one string
body (1× per character), while the input path is allocated twice (2× per character), and neither
changes the argument COUNT, so neither pays the 24-byte slot. ⇒ **a caveat is a claim too, and it
can carry its own unmeasured term.** Writing *"48 bytes are unattributed"* felt like rigour; the
rigorous act was measuring the quantity the sentence quietly assumed.

Related: [[a-control-validates-the-property-it-is-built-from]] — there a control was silent about a
new property; here a *determinism* result was silent about the configuration it was holding fixed.
Both are the same mistake: taking evidence earned under one scope as coverage of another.
