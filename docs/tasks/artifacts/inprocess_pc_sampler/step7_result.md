# In-process PC sampler qualification — PASS with exact raw addresses

`PGEN-RGX-0078-0181`, leaf `RGX-0078.5.j.4`, session #170,
2026-07-20. Instrumentation-only qualification over the immutable preserved C1
probe. No parser, emitter, generated artifact, corpus, or release floor changed.

## Result

The no-root preload profiler passes the pre-registered qualification:

- **1,603** signals seen, **1,603** raw PCs stored, **zero dropped**;
- exact location re-sum: `piece` 85 + `atom_closure` 62 + other main text
  1,294 + external/injected 162 = **1,603**;
- **147/147 target samples = 100.0000%** one-PC retention, clearing the 95%
  bar;
- every main-image PC normalizes with the captured slide and maps to one exact
  instruction in the full-SHA pinned probe disassembly;
- paired eight-pattern observer check: injected/base p50 geomean ratio
  **0.980749x**, within the campaign's 2.28% same-binary noise span and showing
  no detectable gross penalty at this qualification scale.

The target residue classified dynamically as 130 memory / 9 control / 0 call /
8 other = **88.4354% memory**. That is interesting directional evidence that
memory instructions are disproportionately sampled inside the fused regions,
but 147 samples on the eight-pattern qualification workload are not the PCRE2
geomean bands. **No mechanism or nanosecond price is derived here.**

## Signal-safety boundary

`pc_sampler.c` is injected with `DYLD_INSERT_LIBRARIES`; the preserved probe is
byte-unchanged. Its handler:

1. reads the interrupted arm64 PC from `ucontext_t`;
2. reserves a slot with a relaxed atomic whose lock-free property is required
   at compile time;
3. writes one `uintptr_t` into a fixed 1,048,576-entry static buffer.

There is no signal-context allocation, lock, formatting, file I/O, symbol
lookup, or disassembly. `SIGPROF` is included in the handler mask; the atomic
also prevents slot collisions if delivery ever involves another thread. At
normal teardown the probe's single-threaded process blocks `SIGPROF`, disables
`ITIMER_PROF`, restores the previous handler, then writes the raw text artifact.
The handler cannot itself be sampled by the same blocked signal; all
out-of-main-image PCs are nevertheless retained and counted rather than
silently discarded.

Strict compilation is part of qualification:

```sh
clang -fsyntax-only -std=c11 -Wall -Wextra -Werror \
  docs/tasks/artifacts/inprocess_pc_sampler/pc_sampler.c
```

## Found and corrected during qualification

The first run stored 1,504 PCs but found no target sample because the initial
instrument assumed dyld image index 0 was the executable. Injection changes
image ordering on this host: index 0 supplied the wrong slide
(`0x104bc4000`). The sampler now iterates loaded Mach-O headers and selects the
one whose `filetype == MH_EXECUTE`. The rerun captured slide `0x4470000`; all
main PCs then normalized and disassembled exactly. This is why ASLR custody is
an asserted run-time property rather than a naming convention.

## Delivery-rate caveat

The requested interval was 250 us, but 7,555,213 us of process CPU time
delivered 1,603 signals: effective interval **4,713.171 us**. macOS coalesces or
quantizes this process timer well above the request. Retention quality passes;
sample volume must therefore be governed by observed counts, not nominal
duration.

The full-band leaf is pre-registered to require **at least 10,000 stored PCs and
1,000 target PCs per band**. At the observed rate this means roughly 47–55 CPU
seconds per band. A capture that misses either count is extended, not priced.

## Reproduction and custody

```sh
docs/tasks/artifacts/inprocess_pc_sampler/run_qualification.sh
```

The runner:

- asserts the full probe SHA-256;
- builds the tiny dylib with strict warnings;
- runs baseline and injected workloads under `caffeinate`;
- records source/dylib hashes and compiler identity;
- runs the offline analyzer, which asserts raw/header re-sums, target symbol
  starts, ASLR normalization, exact disassembly, target presence, and >=95%
  retention;
- deletes only its explicit temporary dylib directory.

`qualification_analysis.txt` reproduces byte-for-byte from the banked raw,
paired stdout, and recorded dylib hash. The dylib itself is derivable and is not
banked; its SHA-256 for this run is
`7065418bc58f6c4c85c9af63e6186e3ef16ebdc9a665166e4c9f59b753f40a10`.

## Next

`PGEN-RGX-0078-0182` reconstructs the three byte-derivable band inputs and runs
one band at a time under this qualified profiler until each clears both sample
count floors. Only then may exact target PCs be grouped into named removable
mechanisms, with G1-C and memo-copy overlap explicitly subtracted.
