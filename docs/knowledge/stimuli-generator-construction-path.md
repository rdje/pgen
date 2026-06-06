---
id: stimuli-generator-construction-path
title: How main.rs builds the StimuliGenerator from a grammar + runs the witness pass (production path)
answers:
  - "how does main.rs construct the stimuli generator from the ebnf"
  - "how is the StimuliGenerator built from a grammar"
  - "what is the production stimuli-generation path"
  - "how do I build a StimuliGenerator programmatically"
  - "how is a grammar loaded into the generator"
  - "what is LoadedGrammar"
  - "how does the witness pass get invoked"
  - "how do I run generate_target_witnesses"
  - "where are grammar_tree and rule_order available in main.rs"
tags: [stimuli, generator, construction, witness-pass, main, LoadedGrammar, certifying-linter]
date: 2026-06-06
status: current
evidence: "rust/src/main.rs (StimuliGenerator::new at :1083/:1187/:1261/:2194; generate_gap_report at :1210/:1661; generate_target_witnesses at :1474); rust/src/ast_pipeline/stimuli_generator.rs (StimuliGenerator::new, generate_gap_report, generate_target_witnesses, witness_certificates)"
reverify: "grep -n 'StimuliGenerator::new\\|generate_gap_report\\|generate_target_witnesses' rust/src/main.rs"
---

**The production stimuli/witness path** (what `ast_pipeline` does for `--generate-stimuli` and the
gap/witness report modes), and the integration point for the certifying linter's witness side
([[grammar-linter-trustworthiness]], GRAMMAR-WELLFORMED.G.4.3):

1. **Load the grammar → `LoadedGrammar`.** `main.rs` parses the input into
   `LoadedGrammar { grammar_name: String, grammar_tree: HashMap<String, ASTNode>, rule_order:
   Vec<String>, annotations: Option<Annotations> }`. For an `.ebnf` input this goes through the Rust
   `ebnf_frontend` (requires `--features ebnf_dual_run`) → raw_ast → the normalized gen-AST
   (`grammar_tree`); for a `.json` input the AST is loaded directly. So `grammar.grammar_tree` +
   `grammar.rule_order` + `grammar.annotations` are the canonical grammar surfaces, available
   throughout the command handlers.

2. **Build the generator.** `let mut generator = StimuliGenerator::new(&grammar.grammar_tree,
   &grammar.rule_order, <config/seed/...>)` (main.rs :1083, :1187, :1261, :2194, …). The generator
   borrows the grammar; config carries seed, depth/visit limits, profile, etc.

3. **Coverage + witnesses.** `generator.generate_gap_report(Some(entry_rule), k)` → a
   `StimuliCoverageGapReport` whose `.targets` are the uncovered (rule/branch) coverage targets;
   `generator.generate_target_witnesses(&report.targets)` (main.rs :1474) runs the appended
   minimal-witness pass (SV-EXH-PROOF.7.4.x — one dedicated witness per still-unresolved REACHABLE
   target, rooted at the target's own rule). Diverse samples come from `generator.generate_many(count,
   Some(entry))`.

4. **Certifying-linter hook (G.3.2/G.4.3).** After the witness pass, `generator.witness_certificates()`
   returns the captured `ReachabilityWitness { fragment, input }`s. At the witness-path site
   (main.rs ~:1474) `grammar.grammar_tree`, `grammar.rule_order`, `args.grammar_profile`, and the
   generator are all in scope — so the certificate-coverage gate (`gather_verified_witness_covered` via
   `parser_registry::parse_and_cover_systemverilog` + `gather_verified_proof_covered_rules` →
   `certificate_coverage`) wires in directly. The witness side needs `--features generated_parsers`
   too; `ast_pipeline` compiles with `ebnf_dual_run,generated_parsers` together.

Note: building from `.ebnf` is `ebnf_dual_run`-gated; verifying witnesses through the real parser is
`generated_parsers`/`has_generated_systemverilog_parser`-gated. The closed-loop gate
(`sv_stimuli_quality_gate.sh`) historically splits these across two binaries (ast_pipeline generates,
parseability_probe parses); G.4.3 can instead do both in one dual-feature `ast_pipeline` build.
