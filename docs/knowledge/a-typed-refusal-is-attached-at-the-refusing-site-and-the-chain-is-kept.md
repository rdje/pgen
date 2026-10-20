---
id: a-typed-refusal-is-attached-at-the-refusing-site-and-the-chain-is-kept
title: A typed refusal is attached at the refusing site and the error chain is kept — classify by downcast, never by parsing the message, and never flatten a chain whose Debug rendering a tool prints
answers:
  - "how does PreprocessRefusal know its kind, code, file and line"
  - "why not classify a preprocessor refusal by matching its message text"
  - "why did the first typed-refusal build change ast_pipeline's stderr for seven fixtures"
  - "what is RefusalSite and why is it a context layer rather than a root error"
  - "how to add a typed error to an anyhow chain without changing one byte of what is printed"
  - "how was byte identity of the preprocessing stage proven for SVPP-EXPANSION.9.2"
tags: [svpp, preprocessing-service, anyhow, error-chain, typed-refusal, byte-identity, replay]
date: 2026-10-20
status: current
evidence: "PGEN-SVPP-EXPANSION-0053 (SVPP-EXPANSION.9.2): 29 bail! sites became refuse!(kind, file, line; …) with the same format strings; the first build flattened three with_context chains into one message and the 236-fixture replay (rust/target/svpp9/s92/replay.sh, ast_pipeline + parseability_probe, every output file + exit code + stderr) found 7 real stderr diffs — anyhow's {:?} prints a chain as 'Error: outer / Caused by: inner', and a flattened message has no chain; RefusalSite (a Display-preserving context layer) replaced the flattening and the normalized diff went to 0 over 1,836 files."
reverify: "cd rust && cargo test --features \"generated_parsers ebnf_dual_run\" --lib -- families::systemverilog::preprocessing_service 2>&1 | grep -E '^test result: ok' && grep -c 'refuse!(' src/families/systemverilog/preprocessor.rs | awk '$1 >= 29' && echo 'typed refusal at the site: OK'"
---

**A refusal's kind, code, file and line are facts the refusing site knows; the message is for a human. Attach the facts where they are known, keep the chain exactly as it was, and read the facts back by type.**

## The rules

1. **Classify by downcast, never by message.** A `Refusal { kind, code, file, line, message }` is the ROOT
   error at every `bail!` site (`refuse!(kind, file, line; fmt, …)` keeps the format string byte for byte, so
   `Display` is unchanged); `PreprocessRefusal::from(anyhow::Error)` finds it with `downcast_ref::<Refusal>()`
   through any number of `with_context` layers. A message is a rendering; a regex over it is a second parser of
   the first parser's prose and rots with every rewording.
2. **A site that wraps someone else's error adds a context LAYER that carries the facts, not a flattened
   message.** `RefusalSite { kind, file, line, text }` implements `Display` as the exact string the old
   `with_context(|| format!(…))` produced, so `{:#}` AND `{:?}` print byte-for-byte what they printed before;
   the typed API reads the outermost `RefusalSite` for the line of the directive and the root `Refusal` for the
   kind. The first build instead re-typed the chain as one `Refusal` whose message was `format!("{:#}", err)`
   — identical under `{:#}` (the probe), and WRONG under `{:?}` (`ast_pipeline` returns the error from `main`,
   which prints `Error: outer\n\nCaused by:\n    inner`): seven refusal fixtures changed their stderr.
3. **Prove byte identity with a replay, not an argument.** Build the baseline binaries at HEAD before the first
   edit and keep copies; replay every directive-bearing fixture (here 236: the nexsim `preprocess_*` corpus, the
   curated preprocessor fixtures, the adjudication repros with directives) through both tools, capturing every
   output file, exit code and stderr; normalize only what the replay itself changes (the output directory name)
   and diff the trees. The replay is what found rule 2; the argument had said the text was identical.
4. **One code path under every surface.** The typed API, the socket and the probe call the same `run_unit`
   (configuration derived once from the declared option table, the same per-file / per-unit entry); a test
   pins the typed API's five outputs byte-equal to the socket's. A second code path is a second set of
   verdicts, and the contract promises one.

## Where it lives

- `rust/src/families/systemverilog/preprocessor.rs` — `RefusalKind`, `Refusal`, `RefusalSite`, `refuse!`,
  `refuse_coded!`; every refusal site typed.
- `rust/src/families/systemverilog/preprocessing_service.rs` — `PreprocessRefusal::from_stage_error` (the
  downcast classification), `refusal_code`, the typed API, the tests.
- `rust/src/preprocessing_stage.rs` — the neutral `FileProvider`, `DiskFileProvider`, `MemoryFileProvider`,
  `OUTPUT_SCHEMA_VERSION`, the carrier.
- `docs/tasks/SVPP-EXPANSION.md` leaf `.9.2` — the replay, the seven diffs, the fix.
