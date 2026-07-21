# `PGEN-RGX-0078-0208` result — post-FxHash-swap re-pricing (read-only measurement)

Leaf `RGX-0078.5.j.4`, session #181, 2026-07-21. The `-0207` NEXT pointer +
the 2026-07-21 director GO executed: re-price on the NEW 1,037.8 ns floor
before selecting the next fix.

## Capture (three bands, the `-0182`/`-0204`/`-0206` recipe verbatim)

| gate | result |
|---|---|
| probe custody | `fxstore_a4067793` SHA asserted before every step |
| band inputs | byte-derived from the SHA-pinned `-0207` timing rows (`7204a541…`); partition **1,073 / 731 / 374** cells, 11 ≥20 µs excluded; geomean reproduces **1037.804231341058** exactly |
| band weights (log-shares) | 0.435976 / 0.352565 / 0.203675 |
| teardown qualification | deterministic pending-SIGPROF drain PASS before the corpus was spent |
| stored samples | 56,959 / 54,119 / 29,545 — **zero drops**, all ≥10,000 |
| in-target samples | 6,499 / 6,408 / 4,074 — all ≥1,000; 100% disassembly pinning |
| verdict identity | PASS vs the `-0207` floor sweep on all 2,178 band rows |

## The role map — re-derived from scratch (structurally forced this round)

The `-0207` swap shrank three hasher states (std `RandomState` 16 B →
`FxBuildHasher` 0 B), two on the parser-reachable state path
(`predicate_defs` + `by_kind`). The fresh derivation proves it in-disassembly:
**28 offsets / 9 roles as before; the low block (guard `0x8–0x60`, checkpoint
`0xF0–0x198`) unchanged; every offset ≥ `0x250` shifted by exactly −0x20**
(taint `0x250→0x230`, checkpoint 7th source `0x258→0x238`, rows
`0x280/0x288/0x290/0x538 → 0x260/0x268/0x270/0x518`, entries
`0x2A0/0x2A8 → 0x280/0x288`, tape `→ 0x290/0x298/0x2A0`, input
`→ 0x340/0x348`, arena `→ 0x350`, position `→ 0x4C8/0x510`). Machine
qualification PASS (every fingerprint re-verified at the shifted offsets;
REFUSE-on-unknown; the 12 pre-shrink offsets asserted GONE).

## The `-0207` fix structurally verified in-profile

- The `-0206` hot sip-write monomorph: **0 static bl-sites** (asserted) and
  **dynamically GONE** (was 10.274 ns).
- Store-side `hash_one` monomorphs: `hc6b44c1d…` gone from the binary;
  `hafc51c93…` survives only on the regex-cold custom-`@predicate_def` path
  (no ≥20-sample presence in any band). New residue `hfd6d598…` 1.21 ns.
- `-0207` delivered −7.25 ns against the ≈17.36 ns priced mechanism (42%);
  the surviving sub-threshold hash residue is consistent with the un-captured
  fraction living on cold/rehash paths.

## Re-pricing on the 1037.804231341058 ns floor (noise carried: 23.657 ns)

Tier-1 in-target lanes — **every lane sub-noise again** (ns): input view
13.286 (CLOSED-irreducible transport), checkpoint 9.031, position 5.673,
guard 4.928, tape residue 3.327, rows 1.723, entries 0.968 + `thin_entry_push`
16.076 (the `-0203`-owned payload copy), arena G1-C 0.976, taint 0.008.

Out-of-target populations (caller-attributed shares, not per-site prices):
spine_other 203.75 (no designed mechanism — research-class), allocator
126.08, external/injected 118.30, other_text 80.04, build_value 73.74,
arena_alloc 72.17, harness 54.73 (46.08 = `parse_once_timed`, which INLINES
`RegexParser::new` — i.e. largely construction cost), teardown_drop 51.13,
semantic_runtime 49.15, vec_growth 26.71, tape_helpers 19.85, hashbrown
13.93. Protocol-memo inserts re-price at 12.30 ns (+1.21 hash residue) —
sub-noise, still out of scope (shared observed-parse surface).

## ADJUDICATION — the ONE fix selected

**The director-GO'd teardown/construction recycling unit** (the `-0205`
TLS-lease pattern generalized): recycle the parser's **lifetime-free**
construction/teardown set `{SemanticRuntimeState, memo_fail,
memo_fail_tainted, RecursionGuard}` across parses through per-component
thread-local leases held as parser fields (take at construction, return on
drop, auto-deref keeps every access site unchanged), with a lib-side full
recycle-reset — `reset_for_new_parse` deliberately PRESERVES facts (the
library-import contract), so the lease must clear facts before reuse or an
unrelated parse's facts would leak.

Mechanism (honest, zero-overlap where exact): state-family drops ≈10.0 +
`SemanticRuntimeState::new` 6.10 = **16.1 ns exact**, plus bounded in-scope
shares of `RegexParser` drop (≤13.80, its inline container frees), `_mi_free`
(≤36.28), and the construction alloc paths (zeroed/aligned mi lanes + the
inlined-new mass inside the 46.08 harness bucket). Conservative gross
**≈30–55 ns ≈ 1.3–2.3× noise** — the first above-noise-capable lever in
three sessions.

Refused/deferred within the fork: the `'input`-bound containers (`memo`,
`thin_entries`, `deriv_tape` — would need unsafe lifetime erasure),
`NodeArena` (typed_arena is drop-only + `'input`; its 10.40 drop + ≈72
`alloc_extend` stay the residual), `rule_call_counts` (Arc sharing
semantics), `grammar_profile` (tiny). All recorded as banked follow-ups.

Adjudication contract: the `-0197` ratchet — strict unrounded same-session
corpus-geomean decrease vs base `fxstore_a4067793`, flips 0/2,189, MAX ≤ the
settled 425,000 ns, full battery + all-11 regen train (artifacts EXPECTED to
change: emitter-side field/construction change — the `-0205` expected-delta
review precedent).

## Reproduction

```sh
bash docs/tasks/artifacts/fxstore_reprice/run_capture.sh
python3 docs/tasks/artifacts/fxstore_reprice/analyze_capture.py
python3 docs/tasks/artifacts/fxstore_reprice/classify_and_reprice.py
```
