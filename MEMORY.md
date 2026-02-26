# MEMORY.md

Last updated: 2026-02-27 (+0100)

## Purpose
Live session-continuity file for fast crash recovery and AI handoff.

Use this file to resume work without replaying full chat history.

## Resume Checklist (Read In Order)
1. `git status -sb`
2. Read latest entries in:
   - `CHANGES.md`
   - `DEVELOPMENT_NOTES.md`
   - `PGEN_SOTA_IMPLEMENTATION_ROADMAP.md`
3. Confirm current policy in:
   - `rust/config/sota_exit_policy.env`
4. Confirm untracked-policy files still untracked:
   - `git_message_brief.txt`
   - `questions_keep_untracked.txt`
5. If generated artifacts are needed, regenerate; do not assume they are committed.
6. Continue with highest-priority pending task (see "Next Likely Tasks").

## Current Technical Snapshot
- Branch: `main` (ahead of `origin/main` by 130 commits at last update).
- Worktree: clean at last update.
- Latest commit: `854d115` (`Promote EBNF dual-run strict mode to required SOTA aggregate policy`).
- SOTA policy status:
  - strict EBNF readiness required: `PGEN_SOTA_POLICY_REQUIRE_EBNF_STRICT=1`
  - strict EBNF dual-run required: `PGEN_SOTA_POLICY_REQUIRE_EBNF_DUAL_RUN_STRICT=1`
- Non-annotation parseability contract:
  - `ebnf` is now `require_parseability=true` (with `ebnf_dual_run` adapter path).

## Binding Workflow Rules (Do Not Break)
- After each completed task, run commit workflow automatically.
- Commit workflow is:
  1. amend `git_message_brief.txt` with concise summary
  2. stage intended tracked files only
  3. `git commit -F git_message_brief.txt`
  4. clear `git_message_brief.txt` to 0 bytes
  5. keep `git_message_brief.txt` untracked
- `questions_keep_untracked.txt` must remain untracked.
- Generated artifacts under `generated/` are not authoritative state and may be overwritten/regenerated.
- `--bootstrap-mode` is reserved for generating:
  - `generated/return_annotation_parser.rs`
  - `generated/semantic_annotation_parser.rs`
- For other grammars (`json`, `regex`, `ebnf`, generic `foolang`), use non-bootstrap path.

## Recent Work Summaries (Root Cause -> Fix -> Validation)

### 2026-02-26: EBNF parseability promotion in non-annotation loop
- Root cause:
  - `ebnf` parseability was optional in contract due to missing executable registry path.
- Fix:
  - Added feature-gated `ebnf` parseability adapter in `rust/src/parser_registry.rs`.
  - Promoted `ebnf.require_parseability=true` in `rust/test_data/grammar_quality/ebnf_stimuli_contract.json`.
  - Hardened `rust/scripts/ebnf_stimuli_quality_gate.sh`:
    - bootstrap `generated/ebnf.json` and `generated/ebnf.rs` when required,
    - rebuild `ast_pipeline` with `generated_parsers + ebnf_dual_run`.
- Validation:
  - targeted parser_registry tests passed
  - `PGEN_EBNF_STIMULI_QUALITY_COUNT=3 bash rust/scripts/ebnf_stimuli_quality_gate.sh` passed.

### 2026-02-26: Dual-run strict promotion to required aggregate policy
- Root cause:
  - Dual-run check was still informational in aggregate policy despite strict gate being green.
- Fix:
  - `rust/config/sota_exit_policy.env` updated:
    - `PGEN_SOTA_POLICY_REQUIRE_EBNF_DUAL_RUN_STRICT=1`
  - Docs synchronized (`CHANGES.md`, `DEVELOPMENT_NOTES.md`, `PGEN_USER_GUIDE.md`, roadmap).
- Validation:
  - `make -C rust SHELL=/bin/bash ebnf_frontend_dual_run_gate` passed
  - focused `sota_exit_gate` policy-path run passed with dual-run as required.

## Next Likely Tasks (Priority)
1. Start Pillar 5 (Industrial Frontend Support) kickoff:
   - define first executable SV/VHDL readiness contract and gate skeleton.
2. Continue Rust-native EBNF migration hardening:
   - reduce reliance on Perl frontend where safe, while preserving strict parity gates.
3. Expand parser-registry coverage beyond annotations/ebnf:
   - onboard `json` and `regex` parseability adapters once generated parser integration path is stable.
4. Keep User Guide expansion in sync with advanced steering/gate behavior and operator workflows.

## Known Gaps / Risks
- Pipeline is still hybrid (`ebnf_to_json.pl` remains active in core/gate flows).
- Rust EBNF frontend exists and is validated via dual-run, but is not full replacement yet.
- Pillar 5 (`SV/VHDL readiness`) is still marked `Not Started`.

## Quick Commands
- Strict dual-run check:
  - `make -C rust SHELL=/bin/bash ebnf_frontend_dual_run_gate`
- Non-annotation closed-loop quality:
  - `PGEN_EBNF_STIMULI_QUALITY_COUNT=3 bash rust/scripts/ebnf_stimuli_quality_gate.sh`
- Aggregate gate:
  - `make -C rust SHELL=/bin/bash sota_exit_gate`
