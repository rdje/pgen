---
id: a-ceremony-that-lists-its-family-gates-misses-a-gate-that-reads-the-grammar
title: A release ceremony that lists its family's gates misses a gate that reads the grammar for another reason — the EBNF dual-run caught two svpp releases only at the push (`SV-CORPUS-GRAD.13c.2v.20`)
answers:
  - "why did the CI mirror fail a grammar the family's gates all passed"
  - "which gates does a grammar release have to run"
  - "why was the svpp dual-run envelope ceiling stale since 1.0.8"
  - "what reads grammars/<family>.ebnf besides the family's own gates"
tags: [ceremony, ci-mirror, dual-run, meta-grammar, preprocessor, gates]
date: 2026-10-18
provenance: SV-CORPUS-GRAD.13c.2v.20 (PGEN-SV-CORPUS-GRAD-0465); LANG-CAPABILITY-AUDIT.10.19; rust/scripts/ebnf_frontend_dual_run_diff_gate.sh
reverify: "make -C rust SHELL=/bin/bash ebnf_frontend_dual_run_diff 2>&1 | grep -E '^systemverilog_preprocessor |passed for all'   # svpp pass, '... passed for all tracked grammars'"
---

# A ceremony that lists its family's gates misses a gate that reads the grammar

**A grammar is an input to more than its family. The EBNF dual-run parses EVERY tracked grammar with the
self-hosted meta-parser and ratchets the envelope against the Rust frontend; it reads
`grammars/systemverilog_preprocessor.ebnf` for the META-grammar's sake, so no svpp gate is it — and the svpp
release ceremony ran the svpp gates.**

## What it cost

- Release `1.0.8` raised the svpp envelope past its ceilings (26 divergences against 15, 23 unmapped against 12 —
  eleven new directive keywords, each a `[>! /\w/]` line arm 2 reads as a character class). Nothing failed: the
  dual-run runs in the push mirror, and no push fell between the two releases.
- Release `1.0.9`'s first commit then failed the mirror outright: arm 2 could not parse two shapes the release
  introduced (a bracket class inside `[>! … ]`; a `[> … ]` line after a rule that ends in a return annotation). The
  fix — a respelling, a redundant restriction removed — changed the generated parser by inlining alone, and that
  still cost a rebuild and a full SystemVerilog re-key, because every identity row names the probe's bytes.

## The rule

- **Derive a release's gate list from the files it changes, not from the family's name.** For a grammar that is: the
  family's gates, plus every gate whose inputs include `grammars/*.ebnf` — the dual-run, the reproducibility rows, the
  certificate table, the parity gates.
- **Run the push mirror's grammar gates BEFORE the identity re-key.** A re-key is the most expensive step of a
  ceremony and is invalidated by any later byte the binaries embed; a gate that can force a grammar edit belongs in
  front of it.
- **Read a refused input on both readers.** The Rust frontend is authoritative and accepted the grammar; the
  meta-grammar refused it. Each refusal was reduced to a three-line grammar (`parseability_probe --parse ebnf`)
  before anything was changed.

Related: [[a-filtered-acceptance-run-is-not-the-module]], [[a-generated-sample-the-parser-refuses-is-a-boundary-or-a-defect]].

## A second instance, the same push (2026-10-19, `SVPP-EXPANSION.6i`)

The mirror's next job after the dual-run, `stimuli-cross-family-platform-gate`, was red too — since 2026-10-08, from a
different commit: `.6g` added a refuse-on-absence contract key to the full SV stimuli contract and not to the BOUNDED
cross-family one, whose only reader is this mirror job. Two gates nothing else runs, red from two commits, both found at
one push. The rule above is not about grammars alone: it is about every gate whose only invocation is the push mirror —
derive that set (`GATE-REACHABILITY` knows it) and run it on each release that touches a gate's inputs.

## A third instance, the same push (`DONE-BAR.5h`)

`sota-exit-gate`'s first prerequisite, the silent-success sentinel gate, took `target/debug/ast_pipeline` as found; the
mirror's preceding jobs leave it built without the generated parsers, so it refused every family — since 2026-09-30, the day
it was wired as the prerequisite, which was after the previous push. Three mirror-only gates, three commits, one push.
Deriving the mirror-only set and running it per release is `CI-PARITY-GATE-ROT.119`.

## A fourth instance, the same push (`DONE-BAR.5i`) — and the half of the rule the third one missed

The fourth run reached the aggregate's SV family-status stage for the first time, and its leg-3 conformance gate refused:
it took `target/release/parseability_probe` as found, and the export has no `target/release` at all. Red since the gate
joined the stage (2026-09-03). Four gates, four commits, one push, each found only after the previous one's ≈5 h re-run.

The fix was NOT the third instance's fix. The sentinel builds its debug pair unconditionally, and that is tolerable
because ordinary builds re-uplift the debug pair anyway and the ceremony restores after them. The RELEASE probe is
different: the published path holds the ceremony's lowmem copy, cargo's `deps/` copy is a different build of the same tree
(this host does not rebuild byte-for-byte), and cargo re-uplifts on EVERY build of a unit, fresh or not — so an
unconditional `cargo build` in this gate would have swapped the identity every tracked lane report names, which is the
`1.0.252` defect `a-ceremonys-measured-binary-is-kept-not-re-derived` records. The rule has two halves and a gate needs
both: **prove the binary you measure with (its own fingerprint, never `-x`), build it when the proof fails, and KEEP it when
the proof holds.** Build-always and find-always are the two ways to get one half.

## A fifth instance, the same push (`CI-PARITY-GATE-ROT.119`(f)) — the next assumption waits behind the one you fixed

Run 7 was the first in which the leg-3 conformance gate got past its probe: the export built one, both corpus lanes
ran, and the adjudicate stage died in `adjudicate_dark_worklist.probe_text` with
`FileNotFoundError: …/export/tmp/tmp….sv`. The function wrote its one-file units into the repo-root `tmp/` that its
own `main()` creates — but the conformance adjudicator imports the function without `main()`, and the export
(`git ls-files` only) has no `tmp/` because `.gitignore` excludes it. In every developer tree the directory had existed
for months, created by some earlier `main()` run. Three things make this the series' shape and not a new one: nothing
but the mirror runs this path on a fresh checkout; each run costs ≈5 h; and the defect was unreachable until the fourth
instance's fix let the stage proceed. The rule it adds: **a function that writes into an ignored directory creates that
directory at the point of use** — an entry point's `mkdir` protects only the callers that go through the entry point,
and a fresh checkout has none of the ignored tree.

Related: [[a-ceremonys-measured-binary-is-kept-not-re-derived]].
