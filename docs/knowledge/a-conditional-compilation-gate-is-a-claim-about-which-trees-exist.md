---
id: a-conditional-compilation-gate-is-a-claim-about-which-trees-exist
title: A `#[cfg]` gate keyed on ARTIFACT PRESENCE is a claim about which trees can exist — put it on the data, never on the accessor a feature-gated caller reaches
answers:
  - "my crate builds for me but a fresh clone dies with E0425 cannot find function"
  - "rustc says found an item that was configured out — how did this ever compile"
  - "the target that generates my artifact needs a build that needs the artifact"
  - "why did a build-flow one-liner with byte-identical outputs break the bootstrap"
  - "how do I test the cold-clone build configuration without deleting my generated files"
  - "how can a gate prove a fresh-clone build works when every dev tree is warm"
tags: [build, rust, cfg, bootstrap, codegen, instruments, ci, evidence]
date: 2026-08-25
status: current
evidence: CI-PARITY-GATE-ROT.40. PGEN gates code on two independent axes — cargo FEATURES (caller-chosen) and `has_generated_*` ARTIFACT cfgs that `build.rs` sets from what is on disk. `parse_harness_equivalence` is gated on FEATURES and called `parser_registry::active_grammar_profile`, gated on ARTIFACTS. Every developer tree has a populated `generated/`, so it compiled everywhere for 10 days while a fresh clone died on one `error[E0425]` at the FIRST command README's Quick Start names. Fixed by DELETING the gate from the accessor (the artifact knowledge was already gated one level down, in a cfg'd match arm), not by adding a `cfg(not(...))` twin. Measured before 101 / after 0, then widened to every declared feature set — 9/9 arms.
reverify: "rust/scripts/cold_clone_build_probe.sh --census --self-test   # 9/9; the --self-test RED CONTROL re-injects the founding gate into an isolated copy and must reproduce the identical E0425"
---

**A conditional-compilation gate is not a detail of the build — it is a claim about which trees the
code is allowed to exist in.** When two gating axes are in play and only one of them is visible from
where you are standing, the compiler will happily agree with you on every tree you own and refuse
the one tree you never build.

## The shape

Two axes, independent by construction:

| axis | who sets it | example |
|---|---|---|
| **FEATURES** | the caller, on the command line | `--features "generated_parsers ebnf_dual_run"` |
| **ARTIFACT cfgs** | `build.rs`, from what is on disk | `has_generated_systemverilog_parser` |

Feature-gated code may be compiled on a tree where an artifact cfg is unset. So:

> An item gated on **artifact presence**, referenced from code gated only on **features**, is a
> compile error on every tree where the artifact is missing — and on no other.

The developer population is exactly the population where the artifact is present. That is why this
class survives review, survives CI, and detonates on the first fresh clone.

## Why it is worse than an ordinary build break

The artifact is missing precisely when you are about to **generate** it. The dependency runs in a
circle:

```
generate the parser  →  needs the generator binary
                     →  which is built with the feature set
                     →  which pulls in the module
                     →  which references the item
                     →  which is gated on the parser being there already
```

PGEN's instance was authored by a one-line build-flow fix whose own commit message — *"ZERO shipped
bytes, generated parsers byte-identical"* — was **entirely true**. Neither half of it is a statement
about the cold path. Evidence can be accurate and still be blind to the axis you moved.

## The fix direction that scales: gate the DATA, not the accessor

The instinct is to add a `#[cfg(not(any(...)))]` companion returning a neutral value. It compiles,
and it costs you a second body that no warm build compiles and no warm test runs — a divergence
surface, created to paper over a gate that should not have been there.

Prefer **deleting the gate from the accessor** and leaving it on the artifact-specific *data* one
level down, where it usually already is:

```rust
// the accessor: ungated — a feature-gated caller can always reach it
pub fn active_grammar_profile(name: &str, requested: Option<&str>) -> Option<String> { … }

fn default_generated_grammar_profile(name: &str) -> Option<&'static str> {
    match name {
        #[cfg(has_generated_regex_parser)]     // ← the gate belongs HERE, on the datum
        "regex" => Some(RegexParser::DEFAULT_GRAMMAR_PROFILE),
        _ => None,
    }
}
```

With no artifact the arm vanishes and the lookup answers `None` — which is the *correct* answer, not
a stub's answer. One definition, no twin, no drift. And leave a doc line saying the accessor is
ungated **on purpose**, because the next reader's instinct will be to restore symmetry.

## Reproduce the cold tree WITHOUT destroying your own

This is the part that turns a finding into a gate. The obvious reproductions — wipe `generated/`,
move the artifacts aside — mutate the tree they measure, so no automated check can run them.

If `build.rs` resolves artifacts through env vars (a good idea for other reasons), point them all at
a path that does not exist:

```bash
env CARGO_TARGET_DIR=target/coldprobe \
    PGEN_SYSTEMVERILOG_PARSER_PATH=../generated/__absent__.rs   … \
    cargo check --features "generated_parsers ebnf_dual_run" --lib
```

Same `build.rs` code path, same absent cfgs, **nothing moved, nothing restored**, warm cache
untouched, separate target dir. Non-destructive and repeatable ⇒ **gateable**. That single change is
what let the finding become a checked invariant instead of a paragraph in a task file.

## The census is the compiler, not a grep

"Is this the only such reference, or just the first one found?" is a **census** question, and the
temptation is to answer it with `grep`. Don't: Rust name resolution is a whole-crate pass that
collects *every* unresolved reference before aborting, so the error count in the cold configuration
is already a complete count — and a clean run after the fix proves nothing was masked behind the
first failure, including type errors that resolution failure would have hidden. Then widen the
population deliberately: the census is over **feature sets**, enumerated from `Cargo.toml`'s own
`[features]` block rather than from the ones that came to mind.

## Related

- [[a-bootstrap-check-must-not-block-the-act-that-bootstraps-it]] — the sibling shape: a *check*,
  rather than a *gate*, that forbids the state its own subject must pass through.
- [[your-build-tools-timestamp-resolution-is-part-of-your-correctness-argument]] — the other way a
  build tool silently answers a question about a tree you are not in.
