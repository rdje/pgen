---
id: a-build-script-panic-must-respect-the-bootstrap-posture
title: A build-script panic must respect the bootstrap posture — a refactor of what the build requires is verified from a bare tree, not a warm one
answers:
  - "why does cargo build fail with generated/return_annotation_parser.rs is missing on a fresh clone"
  - "why does make regenerate_generated_parsers fail at regex_parser_bootstrap with a build.rs panic"
  - "how does a cold clone bootstrap the annotation parsers when build.rs requires them"
  - "how does build.rs know which cargo features are enabled"
  - "why does cargo build fail with cannot find function generated_backend_enabled on a fresh clone"
  - "what happens to a cfg attribute when the item under it is deleted"
  - "how do I find a cfg attribute that was left behind by a refactor"
tags: [build, bootstrap, build.rs, cold-clone, ci_workflow_local_gate, features, cfg, attributes]
date: 2026-09-30
status: current
evidence: "ENGINE-UNIVERSAL-SERVICES.117 (PGEN-ENGINE-UNIVERSAL-SERVICES-0189): the CI mirror's export (a git ls-files worktree) died at regenerate_generated_parsers' first cargo build — build.rs:73 panicked for the meta parsers under --features ebnf_dual_run, the bootstrap posture that has no generated parser by design; behind it, the chain's first full-feature build died on error[E0425] cannot find function generated_backend_enabled — a #[cfg] attribute .112 slice 3 left above the function when it deleted the item it belonged to had removed both arms; gating the panic on CARGO_FEATURE_GENERATED_PARSERS and deleting the two stray attributes (the scan orphan_cfg_scan.py reads sites=2 on the parent, 0 after) restored the cold path"
reverify: "d=$(mktemp -d rust/target/pgen-cold.XXXX); git archive HEAD | tar -x -C \"$d\" && (cd \"$d\" && make -C rust SHELL=/bin/bash regenerate_generated_parsers >/dev/null 2>&1 && ls generated/*_parser.rs | wc -l)  # a bare tracked tree on the repo volume; expect the full artifact count and no build.rs panic"
---

`rust/build.rs` derives the parser registry from `generated/` and, since
`ENGINE-UNIVERSAL-SERVICES.112` slice 2, refuses a build that lacks the two meta parsers
(`return_annotation`, `semantic_annotation`) — a build error naming the remedy rather than a
silent cfg-off. That refusal is right **under the `generated_parsers` feature** and wrong
without it: `regenerate_generated_parsers` on a bare tree begins with `regex_parser_bootstrap`,
which builds `ast_pipeline` with `--features ebnf_dual_run` only, no generated parser at all —
the annotation pair does not exist yet, annotations route through the hand-rolled bootstrap
surface by design, and `lib.rs` gates the whole `generated_parsers` module on the feature. An
unconditional panic there means the generator that would emit the parsers cannot be built
because the parsers are missing: the remedy the message names is the command that is failing.

- **A build script sees the feature set through `CARGO_FEATURE_<NAME>`** — cargo exports one
  such variable per enabled feature. A requirement that exists only under a feature is gated on
  that variable (and declares `cargo:rerun-if-env-changed` for it); nothing else in a build
  script knows the posture.
- **An attribute binds to the next item across blank lines.** Delete the item under a
  `#[cfg(…)]` and leave the attribute, and it silently gates whatever follows — `.112` slice 3
  left `#[cfg(all(feature = "generated_parsers", any(has_generated_systemverilog_parser,
  has_generated_vhdl_parser)))]` above `generated_backend_enabled`'s first arm, so under the full
  feature set with only the meta parsers present neither arm existed (`error[E0425]`); `.113`
  left one above `usage()` in the parseability probe. The control is
  `docs/tasks/artifacts/engine_universal_services/eus117_bare_clone/orphan_cfg_scan.py`
  (`rust/src`, exit 1 with the sites named); run it after any deletion under an attribute.
- **A warm tree cannot see this class.** Every working tree carried its `generated/` for
  months, so fifty-one commits passed over the panic. The only instrument that exercises the
  bootstrap is a tracked-files-only worktree: the CI mirror's export (`ci_workflow_local_gate`,
  run at the 400-commit push cadence) or the reverify command above. A refactor of what the
  build REQUIRES is verified there, not on the tree the refactor was written in.
- **Two rosters hid it for two days.** The mirror's preflight derived its required artifacts
  from `lib.rs`'s literal includes — empty after the same slice — so it reported the export
  "present" and skipped preparation; the first replay then died in `build.rs` with an error aimed
  at the wrong subject. `CI-PARITY-GATE-ROT.67` widened the derivation to `build.rs`'s own
  mandatory list; that is what let this defect surface.

Related: [[a-new-tracked-top-level-doc-is-an-allowlist-event]],
[[your-build-tools-timestamp-resolution-is-part-of-your-correctness-argument]].
