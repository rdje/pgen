# PGEN Release Policy (Living)

Last updated: 2026-03-07

## Purpose
Define objective, machine-enforced pass criteria for release-grade validation of PGEN.

This policy is consumed by:
- `rust/scripts/sota_exit_gate.sh`
- `make -C rust SHELL=/bin/bash sota_exit_gate`
- `.github/workflows/sota-exit-gate.yml`

## Machine Policy Source
Tracked policy file:
- `rust/config/sota_exit_policy.env`
- `rust/config/branch_protection_policy.json`

Current policy values:
- `PGEN_SOTA_POLICY_VERSION=1`
- `PGEN_SOTA_POLICY_REQUIRED_CHECKS="differential_baseline_contract fixed_point_gate annotation_contract_gate annotation_nonbootstrap_e2e_gate ebnf_stimuli_quality_gate differential_regression_gate performance_gate embedding_api_gate"`
- `PGEN_SOTA_POLICY_RUN_EBNF_READINESS=1`
- `PGEN_SOTA_POLICY_REQUIRE_EBNF_STRICT=1`
- `PGEN_SOTA_POLICY_ALLOW_INFORMATIONAL_FAILURES=1`

## Required Gate Criteria
The following checks must pass (`required_failures == 0`):
1. `differential_baseline_contract`
2. `fixed_point_gate`
3. `annotation_contract_gate`
4. `annotation_nonbootstrap_e2e_gate`
5. `ebnf_stimuli_quality_gate`
6. `differential_regression_gate`
7. `performance_gate`
8. `embedding_api_gate`

Current informational policy:
- EBNF frontend strict readiness is required (`PGEN_SOTA_POLICY_REQUIRE_EBNF_STRICT=1`).
- Perl-vs-Rust EBNF dual-run differential remains report-mode by default (`PGEN_SOTA_POLICY_REQUIRE_EBNF_DUAL_RUN_STRICT=0`).

## Local Release Validation
Primary command:
```bash
make -C rust SHELL=/bin/bash sota_exit_gate
```

Result artifacts:
- `rust/target/sota_exit_gate/summary.csv`
- `rust/target/sota_exit_gate/summary.txt`
- `rust/target/sota_exit_gate/logs/*.log`

## CI/Branch Protection Policy
Tracked branch-protection contract:
- `rust/config/branch_protection_policy.json`

Validated by:
- `rust/scripts/branch_protection_contract_gate.sh`
- `make -C rust SHELL=/bin/bash branch_protection_contract_gate`
- `.github/workflows/branch-protection-contract-gate.yml`

Current minimum required checks enforced by the contract gate:
1. `sota-exit-gate`
2. `annotation-contract-gate`
3. `differential-regression-gate`
4. `fixed-point-gate`
5. `performance-gate`

`sota-exit-gate` is the aggregate policy check; the individual checks above remain useful for targeted visibility and faster failure triage.

The branch-protection contract gate also enforces that every required check maps to a tracked workflow/job name and that the corresponding workflow runs on `pull_request`.

## Promotion Criteria for Strict EBNF
Set:
- `PGEN_SOTA_POLICY_REQUIRE_EBNF_STRICT=1`

Only after:
1. `make -C rust SHELL=/bin/bash ebnf_frontend_gate` is green for all tracked grammars.
2. `grammars/ebnf.ebnf` compatibility debt is closed.
3. Roadmap Phase H strict-readiness task is marked done.
