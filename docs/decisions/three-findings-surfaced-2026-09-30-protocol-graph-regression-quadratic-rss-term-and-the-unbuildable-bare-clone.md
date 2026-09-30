# Director callout 2026-09-30 — three non-routine findings from the `.112` refactor window, and one ruling the distance needs

- **Category:** project (director callout — a digest of findings already fixed or ruled in their leaves)
- **Date:** 2026-09-30
- **Found by:** `ENGINE-UNIVERSAL-SERVICES.116` (`-0185`), `.84` (`-0185`), `.117` (`-0189`)

Surfaced at the push-lane stopping point, per the session directive ("surface non-routine
findings in a labeled callout and a durable file"). Each is fixed or ruled and recorded in its
leaf; this record is the digest the director asked to see, not a new claim.

## 1. Every registry parse ran the PROTOCOL graph for two days (`ENGINE-UNIVERSAL-SERVICES.116`, `-0185`)

`.112` slice 2 (`f04f5b807`, 2026-09-28) served the canonical entry through
`parse_full_from(GRAMMAR_ENTRY_RULE)`, on the generator's own word that the entry-relative path is
"behavior-identical to `parse()`" — true of the verdict and the AST, false of the graph: that path
pins `bare_parse = false`. From `f04f5b807` until `.116` every registry parse (`--parse`, the corpus
runs, the embedding API) ran the protocol graph, not the fused cascade graph the speed campaign
ships. Verdict- and AST-identical, so the identity sweeps that pinned the slice could not see it;
12–26 % wall and 11–37 % RSS dearer on SystemVerilog. Fixed: `None ⇒ parse_full()`, held by two
registry unit tests and the default-graph probe. **The lesson:** an identity sweep pins WHAT a
parse returns, never WHICH graph produced it; the graph needs its own instrument (arena node count
+ `/usr/bin/sample` frames), now in `TOOLBOX.md` §3.3b.

## 2. The deep-nesting peak-RSS term is REAL, located, and parked (`.84`, `-0185`)

A deeply nested input costs peak RSS quadratic in its nesting depth on `return_annotation` (1 174 MB
for an 8 KB file) while the AST arena grows linearly. `.84`(a)–(d) located it: `create_contextual_error`
materialises the rule stack per failed alternative, and the fused cascade holds one such
materialisation live per frame. Item 1 had MASKED it (the protocol graph does not take that path),
which is why it first read as fixed. The O(1) error channel is routed as `.84`(e), parked with
`.68` behind the SV lane lock — **not SV-release-blocking** (the corpus's deepest file is far below
the depth where the term dominates), but it is the first thing a pathological consumer input will hit.

## 3. A bare clone could not build since `.112` — the README Quick Start was false for two days (`.117`, `-0189`)

Two defects, one behind the other, both invisible on every warm tree: `build.rs` panicked for the
two meta parsers under the bootstrap posture that has none by design, and a `#[cfg]` attribute
left above `generated_backend_enabled` when its own item was deleted removed both arms of that
function at the chain's first full-feature build. Nothing ran the cold path because the CI mirror's
preflight derived its required artifacts from a `lib.rs` form the same slice retired (an empty
set, reported "present"). Fixed and proven end to end in a `git ls-files` export (11/11 artifacts
byte-identical); the preflight derivation widened. **The structural point:** a refactor of what the
build REQUIRES is verified from a bare tree, and the only instrument that walks one runs at the
300-commit push cadence — fifty-one commits passed over it. Whether the cold chain should run more
often than the push lane (it costs ~8 min under the guard) is a spend call — **yours**; the default
until ruled is unchanged (push cadence).

## 4. Five engine leaves are parked by your 2026-08-21 ruling and still counted in the release distance

`scripts/report_sv_release_distance.sh` counts `open_engine_leaves = 5` — `.90`, `.92`, `.95`,
`.96`, `.102` — every one of them opened by an `SV-CORPUS-GRAD` leaf, every one of them priced with
a fix not built, and every one of them marked *ROUTED + PARKED behind the SV lane lock (director
2026-08-21), NOT SV-release-blocking*. The counter and the ruling disagree about whether the SV
release owes them: as long as both stand, the distance cannot fall below its floor of 5 engine
items without either lifting the lock for engine work or changing the counter's rule. This is a
**ruling, not a judgement I can make** (`[DBINP]`): (a) lift the lock for these five, in the order
their trees give; (b) rule them out of the distance (the counter then reads the SV-owed engine
work as 0); or (c) leave both and read the distance as *legs + bar rows + 2 SV leaves + 5 parked*.
Until ruled, the default is (c) and I do not work them.

## 5. The push lane is blocked on `SVPP-EXPANSION.5`(d) — a ruling you have had since 2026-09-09

The full CI mirror (directive 16's gate before any push) replays `sota-exit-gate`; its required
`sv_parser_family_status_gate` runs the SV formal-exhaustive-closure gate, whose external-corpus proof
surface forbids parse failures, and the two UVM triage cases fail in both profiles on the one
construct the ruling is about: `const local uvm_event_pool events = new("events");`
(`uvm_transaction.svh:443`). The parser rejects it LRM-correctly (Annex A's `const`-first class
property takes a `constant_expression`, and `new` is a reserved word since the 2026-09-04 Annex B
fix); every commercial simulator accepts it; your 2026-08-10 bounding ruling's operative test asks
whether a change makes the parser reject what the ecosystem writes — and this rejection takes the
whole UVM library with it. I built and measured the widening (one arm on `class_property`,
constructor first) and withdrew it before regenerating, because the record marks it your call.

**Options, unchanged from the 2026-09-09 question:** (a) the documented tool-extension arm, ON by
default, with a deviation record — recommended; (b) the arm behind the `SV-PARSE-STRICT` knob, OFF
by default, so the strict profile keeps the letter. Either lands as an SV release under the
ceremony; until one does, the mirror cannot be green and the push (313 commits) waits. Everything
else in the queue continues meanwhile.

## What I am asking

Nothing blocks. Items 1 and 3 are closed; item 2's fix is parked by the lane lock by its own text.
Three open calls, in urgency order: item 5 (the `.5`(d) ruling — it blocks the push), item 4 (the parked-but-counted five), item 3 (the cold-chain / `sota-exit-gate` cadence).
