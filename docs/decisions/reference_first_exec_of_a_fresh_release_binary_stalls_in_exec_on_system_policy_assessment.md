---
name: reference-first-exec-of-a-fresh-release-binary-stalls-in-exec-on-system-policy-assessment
description: REFERENCE (2026-09-03, SV-CORPUS-GRAD.3.29) — on this macOS host the FIRST exec of a freshly linked 70 MB release `parseability_probe` blocked inside exec (sample = `_dyld_start`, 0 MB resident, 0 % CPU, state S, no dyld library ever printed) for minutes while `syspolicyd` ran at 57–148 % CPU; every short-timeout retry re-queued the assessment and read as a hang on EVERY input including `module m; endmodule`. The fix is to WAIT ONCE (one un-killed run finished, then every run took < 1 s). A binary-wide, input-independent "hang" right after a release build is this, not a parser defect.
metadata:
  node_type: memory
  type: reference
---

**The case.** `SV-CORPUS-GRAD.3.29` rebuilt `rust/target/release/parseability_probe` (70 239 488 B,
Mach-O arm64, `adhoc,linker-signed`, xattr `com.apple.provenance`). The first invocation — inside a
2-minute tool timeout — printed nothing and was killed. Every later invocation with a 15–30 s cap
returned `rc 124` on every input: the four pins, the corpus rows, `module m; endmodule`, `--supports`,
a JSON file, a regex. That shape — *every family, every input, even `--supports`* — is not a parse.

**What the tools said, in order.** `/usr/bin/sample` on the stuck process: the whole stack is
`_dyld_start (in dyld)`, `rss=0 MB cpu=0.0 state=S` — the process had been created but the image
was not yet activated. `DYLD_PRINT_LIBRARIES=1` printed **no library at all**, so the stall is before
dyld runs, i.e. inside the kernel's exec. The previous release binary (Aug 31, same directory) ran in
1 s. `ps` showed `/usr/libexec/syspolicyd` at **57.4 %** then **147.6 %** CPU, state R — the system
policy daemon assessing something large. A copy of the binary (same cdhash) stalled identically.

**The resolution, measured.** One run left alone under a 240 s cap:
`generated parseability adapter available for grammar 'systemverilog'` / **`rc=0 after 0s`** — by the
time it started, the assessment had completed and been cached against the binary's code directory
hash. Every run since is instant. Nothing was changed on disk; the binary that "hung" is the binary
that now works.

**The rule.** A freshly linked large binary can sit in exec for minutes on this host while
`syspolicyd` assesses it, and killing the waiting client does not cancel the assessment — it queues
another. So, after a release build: run the binary ONCE with no timeout (or a long one, ≥ 4 min)
before any timed measurement, and read a binary-wide, input-independent stall as this before
opening a parser leaf. The corpus runner's per-file timeouts, the ratchet's timed cells and any
`timeout N` in a probe loop are all downstream of that first exec and would otherwise record a
false timeout population against a parser that never ran.

**Honest bound.** The mechanism is inferred from the process state, the daemon's CPU and the
resolution-by-waiting; `log show` for `syspolicyd` in the window showed Gatekeeper evaluation lines
(`GK evaluateScanResult`) but none naming the probe, so which assessment (XProtect scan,
notarization lookup, provenance check) is not pinned. What IS measured: exec-level stall, zero
resident pages, daemon busy, resolved by waiting, cached thereafter.
