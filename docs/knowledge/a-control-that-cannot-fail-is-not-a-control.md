---
id: a-control-that-cannot-fail-is-not-a-control
title: A two-arm control is worthless unless the arm you changed is an input the instrument actually reads — stash the source, and an instrument that reads a generated artifact gives you the same run twice
answers:
  - "I stashed my change and the two arms came back identical — does that prove my change is innocent"
  - "how do I know a before/after control actually controlled for anything"
  - "a gate went red right after my commit and git stash says it is not mine — can I trust that"
  - "why did reverting my grammar edit not change the tool's output at all"
  - "what makes a control valid rather than just two runs"
  - "how do I tell whether an instrument reads my source file or a generated copy of it"
  - "my two arms agree and I want to publish that the regression is pre-existing"
  - "is a byte-identical diff between arms evidence, or the absence of evidence"
tags: [evidence, controls, two-arm, instruments, generated-artifacts, regression-attribution, claim-verification]
date: 2026-08-22
status: current
evidence: "GRAMMAR-WELLFORMED.H.17.1 (PGEN-GRAMMAR-WELLFORMED-0161). `ebnf_frontend_dual_run_gate` went RED on `systemverilog` (envelope `155 > ceiling 151`) in the same session as a `grammars/ebnf.ebnf` repair. `git stash`-ing the grammar and re-running `ebnf_dual_run_diff` returned byte-identical 155-row divergence sets — an apparent exoneration. It was a no-op: `rust/src/bin/ebnf_dual_run_diff.rs:13` `include!`s `env!(\"PGEN_EBNF_PARSER_PATH_RESOLVED_BIN\")`, which `rust/build.rs:101` resolves to `generated/ebnf.rs`, so arm 2 never reads `grammars/ebnf.ebnf`. Proven by removing the file: rustc reports the env var undefined at `:13` plus `error[E0432]`. The valid control rebuilt `generated/ebnf.rs` from the pre-fix grammar (sha `3028814854e0…`) and the differ with it; it also returned 155, so the conclusion survived — but the first argument for it was void."
reverify: "mv generated/ebnf.rs /tmp/ && (cd rust && cargo build --features ebnf_dual_run --bin ebnf_dual_run_diff); mv /tmp/ebnf.rs generated/   # rustc names the dependency: `environment variable PGEN_EBNF_PARSER_PATH_RESOLVED_BIN not defined` at ebnf_dual_run_diff.rs:13 + error[E0432]. Then, for the no-op itself: git stash push grammars/ebnf.ebnf && shasum -a 256 generated/ebnf.rs && git stash pop   # sha UNCHANGED by the stash => the arm never moved. NB grepping the .rs for `include!` shows only env!(...) — the path lives in rust/build.rs:101, which is exactly why the trap is hard to see."
---

**The test of a control is not "did the two arms differ" — it is "could they have".**

A two-arm control is supposed to answer *did my change cause this?* You revert the change, re-run the
instrument, and compare. When the two arms come back byte-identical, the natural reading is
*"not mine"*. That reading is only valid if the arm you flipped is an input the instrument actually
consumes. If it is not, you have run the same measurement twice and compared it with itself — and the
result is guaranteed identical no matter what the truth is. **The failure is silent and it lands in
the flattering direction**, which is why it is worth a specific habit rather than general care.

In PGEN this appeared while repairing `grammars/ebnf.ebnf`. The envelope-differential gate went RED
on `systemverilog` in the same session:

```text
envelope ratchet: envelope divergences REGRESSED: 155 > ceiling 151
```

The obvious control was to stash the grammar and re-measure:

```bash
git stash push grammars/ebnf.ebnf
PGEN_ENVELOPE_DUMP_ALL=1 ./target/debug/ebnf_dual_run_diff --input ../grammars/systemverilog.ebnf \
    --output r.json --envelope-differential env.json
git stash pop
```

Both arms: `divergence_total 155`, and a set-diff of the two dumps was empty in both directions. Case
closed — except the instrument never read the stashed file:

```rust
// rust/src/bin/ebnf_dual_run_diff.rs:13
mod generated_ebnf { include!(env!("PGEN_EBNF_PARSER_PATH_RESOLVED_BIN")); }
```

⛔ **Read that line again: it does not name the file it pulls in.** The path arrives from the build
script — `rust/build.rs:101` sets `PGEN_EBNF_PARSER_PATH_RESOLVED_BIN` to the resolved location of
`generated/ebnf.rs`, alongside a `has_generated_ebnf_parser` cfg it only emits when that file exists.
**This is what makes the trap sharp rather than merely possible**: grepping the binary's own source
for the artifact it reads returns nothing to grep. The dependency is real, compiled-in and total — it
is simply spelled somewhere else.

Arm 2 is therefore the **generated meta-parser**, not the grammar. `generated/ebnf.rs` is untracked,
so `git stash` does not touch it; the grammar text has no path into the run at all. The two arms were
one arm.

The dependency is not an inference from reading the build script — remove the file and the compiler
says so, which is the whole proof in one command:

```text
$ mv generated/ebnf.rs /tmp/ && cargo build --features ebnf_dual_run --bin ebnf_dual_run_diff
error: environment variable `PGEN_EBNF_PARSER_PATH_RESOLVED_BIN` not defined at compile time
  --> src/bin/ebnf_dual_run_diff.rs:13:14
error[E0432]: unresolved import `generated_ebnf::EbnfParser`
```

The valid control had to move the artifact the instrument reads — regenerate `generated/ebnf.rs` from
the pre-fix grammar, rebuild the differ, and only then measure:

```text
pre-fix  generated/ebnf.rs 3028814854e0…  ->  divergence_total = 155
post-fix generated/ebnf.rs 5cb5fc473cfd…  ->  divergence_total = 155   (same 155 rows)
```

Same conclusion — the red was genuinely pre-existing — reached by an argument that could have come out
the other way.

## The habit

Before quoting a two-arm result, **prove the arm moved**. One cheap check does it: name the artifact
the instrument reads, and hash it in both arms.

- If the sha is **identical across arms**, the control did nothing — whatever it printed is not
  evidence.
- If the sha **differs**, the arms are real and the comparison means something.

This is one line, it runs in a second, and it is the difference between an argument and a coincidence.

## Where it bites hardest

The trap is specific to a build step sitting between your edit and the measurement, so it clusters
where sources are *compiled into* the tool:

- **Generated / vendored code** — a parser, a lexer, a protobuf stub `include!`d or linked into the
  binary under test. Editing the grammar or `.proto` changes nothing until you regenerate.
- **Untracked build outputs** — `git stash`, `git checkout` and `git worktree` all move *tracked*
  files. An artifact in `.gitignore` is invisible to every one of them, so any VCS-based arm-flip
  silently skips it.
- **Compiled-in constants and baked assets** — anything read at build time rather than run time.
- **Caches keyed on something other than content** — the arm changes, the cache does not notice.

## The general form

This is the same shape as two neighbours already documented in this repository: a bare
`ebnf_dual_run_diff --input … --output …` run **exits 0 while running only ONE arm**, and an
under-featured `ast_pipeline` build silently omits the parser you meant to exercise. In all three the
invocation is incomplete, the result looks clean, the failure is in the passing direction, and there
is nothing on stdout to notice.

⇒ **A green control and an absent control are indistinguishable from the outside.** Make the control
prove it can fail before you let it clear anything.
