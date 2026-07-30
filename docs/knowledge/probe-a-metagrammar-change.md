---
id: probe-a-metagrammar-change
title: "How to test a change to grammars/ebnf.ebnf END-TO-END without touching the repo — PGEN_EBNF_PARSER_PATH, and the two traps that make it fail"
answers:
  - "how do I test a change to the ebnf meta-grammar before committing it"
  - "how do I build ebnf_dual_run_diff against a different ebnf parser"
  - "what does PGEN_EBNF_PARSER_PATH do"
  - "unresolved import crate::ebnf_generated_parser::EbnfParser"
  - "why does my generated parser define EbnfProbeParser instead of EbnfParser"
  - "how do I regenerate generated/ebnf.rs after editing ebnf.ebnf"
  - "why did my probe grammar fail to build from /tmp"
tags: [ebnf, meta-grammar, probe, toolbox, bootstrap, build, how-to]
date: 2026-07-30
status: current
evidence: "rust/build.rs:29 reads PGEN_EBNF_PARSER_PATH (default ../generated/ebnf.rs), :86-99 sets cfg has_generated_ebnf_parser + PGEN_EBNF_PARSER_PATH_RESOLVED{,_BIN} via relativize_for_include(source_dir, resolved) only when the file exists; rust/src/lib.rs:63-65 include!s it as pub mod ebnf_generated_parser. rust/scripts/ebnf_frontend_dual_run_diff_gate.sh uses the same env to build against a bootstrap-generated parser. WORKED INSTANCE (LANG-CAPABILITY-AUDIT.10.2 option-B probe, session #228): a 7-rule addition to a copy of ebnf.ebnf took the tracked-corpus verdict from 11/12 to 12/12 (regex.ebnf REJECT@55925 -> ACCEPT) with zero repo edits; lint stayed clean at 138 rules, all counters 0."
reverify: "sed -n '25,40p;85,100p' rust/build.rs; grep -n 'PGEN_EBNF_PARSER_PATH' rust/scripts/ebnf_frontend_dual_run_diff_gate.sh"
---

`generated/ebnf.rs` is seeded **once** (`rust/Makefile` `regex_parser_bootstrap` skips the
step when the file exists), so editing `grammars/ebnf.ebnf` changes nothing until you
regenerate. To try a meta-grammar change **without editing the repo at all**:

```bash
P=rust/target/optionB_probe          # MUST be on the repo's own volume — see trap 2
mkdir -p $P && cp grammars/ebnf.ebnf $P/ebnf.ebnf   # MUST be named ebnf.ebnf — see trap 1
$EDITOR $P/ebnf.ebnf

# same two steps the Makefile's one-time seed uses (input AND output paths pinned)
./rust/target/debug/ast_pipeline $P/ebnf.ebnf --emit-raw-ast-json $P/ebnf.json
./rust/target/debug/ast_pipeline --generate-parser --bootstrap-mode --debug \
    --eliminate-left-recursion $P/ebnf.json -o $P/ebnf.rs

# point the build at it, then run the real differential tool
(cd rust && PGEN_EBNF_PARSER_PATH=target/optionB_probe/ebnf.rs \
    cargo build --features "generated_parsers ebnf_dual_run" --bin ebnf_dual_run_diff)
for g in grammars/*.ebnf; do ./rust/target/debug/ebnf_dual_run_diff --input "$g" --output /tmp/r.json; done
```

⛔ **Rebuild `ebnf_dual_run_diff` WITHOUT the env when finished**, or the toolbox binary
silently keeps answering for the probe grammar.

## Trap 1 — the grammar's FILENAME picks the struct name

Codegen derives the parser struct from the grammar name, so `ebnf_probe.ebnf` emits
`pub struct EbnfProbeParser` while `rust/src/lib.rs` includes it expecting `EbnfParser`.
The failure is a confusing pair of:

```
error[E0432]: unresolved import `crate::ebnf_generated_parser::EbnfParser`
```

— the module *was* included; it just does not define that name. **Name the probe file
`ebnf.ebnf`** (in its own directory) and it resolves. Confirm with
`grep -o "pub struct [A-Za-z]*Parser" $P/ebnf.rs`.

## Trap 2 — the probe must live on the repository's own volume

`build.rs` emits the include path via `relativize_for_include(source_dir, resolved)`. A
path on a *different filesystem volume* (e.g. a `/private/tmp` scratchpad when the repo is
on `/Volumes/...`) has no relative form from `rust/src`, so the cfg/env wiring produces
nothing usable and you get the same `E0432`. Keep probes under `rust/target/` — untracked,
and on the repo volume, which the project's data-locality policy requires anyway.

## What the probe proves, and what it does not

It exercises the **real** codegen and the **real** generated meta-parser, so a
`parse_full.ok` verdict over `grammars/*.ebnf` is the same measurement
`ebnf_frontend_dual_run_gate` makes — see [[ebnf-self-hosting-what-it-means]] for what that
number is worth. It says nothing about the hand-written frontend, which is what actually
parses grammars in production ([[ebnf-frontend-architecture]]).
