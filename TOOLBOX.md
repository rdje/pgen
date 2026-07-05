# TOOLBOX.md — PGEN's diagnostic & debug toolbox (USE THIS FIRST)

> **STANDING DIRECTIVE (director, 2026-06-22, emphatic, non-negotiable).** PGEN ships a deep,
> self-explaining debug surface. For **any** `UNKNOWN`, rejected parse, hang, reach gap, or "why
> isn't this witnessed / why was this rejected" question, you **reach for these tools FIRST and
> systematically** — never eyeball a grammar, never guess a root cause, never offer a strategy menu
> before the toolbox has shown you the exact mechanism and source location. *"We are not progressing
> because [the toolbox is ignored]. This needs to stop for good — we have the toolbox, so use it."*
> If the existing tools cannot surface the WHY and WHERE, the next step is to **build** a tool, not
> to speculate.

This file is the **single, authoritative, meticulous catalog**. Each tool entry says **WHAT** it is,
**WHEN** to reach for it, **HOW** to run it (exact command), and **WHAT** the output looks like.

- Mirrored, navigable surfaces (kept in lockstep): the book chapters
  [`docs/book/src/diagnosing-unknowns.md`](docs/book/src/diagnosing-unknowns.md) (master index + the
  `UNKNOWN` protocol) and [`docs/book/src/parseability-probe-debug.md`](docs/book/src/parseability-probe-debug.md)
  (parser-level deep dive); the Knowledge-Map cards under `docs/knowledge/tool-*.md` (one per tool,
  retrieval-indexed by "when/how" questions); the standing decision
  [`docs/decisions/feedback_systematically_use_debug_toolbox.md`](docs/decisions/feedback_systematically_use_debug_toolbox.md).

## Enforcement — this is not optional, and not "trust me"

Using the toolbox is **mechanically enforced**, so it cannot be silently skipped:

- A **code change cannot commit** unless its owning task leaf (`docs/tasks/<TREE>.md`) carries BOTH
  (1) a tool-backed **DIAGNOSIS** (WHY+WHERE — a pasted `CERTIFICATE-COVERAGE:` line, a
  `[plannable-probe]` verdict, a `rejected by post predicate` trace, a `furthest_position=` error,
  or the exact toolbox command) **and** (2) a measured **VERIFICATION** (before→after metric,
  `REJECT→PASS`, determinism at seeds 0/7/42). Enforced by `scripts/check_diagnosis_evidence.sh`,
  run by the general doctrine enforcer `scripts/check_doctrines.sh` via `.githooks/pre-commit` (E3)
  and CI (E4). Demonstrated: a staged code change without that evidence is **blocked**.
- The project's **deterministic gates** (certificate-coverage at seeds 0/7/42, `ast_shape_contract`,
  syntax-closure) **re-run the real tools**, so any number you cite is independently re-verified —
  a fabricated final state does not reproduce and fails. That is the "not trust-me-bro" leg.
- The portable model for how every mechanizable doctrine is enforced is `DOCTRINE_ENFORCEMENT.md`
  (the "rule enforcer" framework: a check-script contract + a registry/driver + the E1→E4 wiring,
  adoptable by any project).
- Reminders fire at **session start** and **before every code edit** (`.claude/settings.json`
  `SessionStart` + `PreToolUse` hooks), so the toolbox-first rule is in context the moment it matters.

### The task-acceptance checklist (required for any code-change commit)

Every code-landing task leaf (`docs/tasks/<TREE>.md`) MUST carry this checklist, each box **ticked
`[x]`** and backed by the cited tool output. An **unticked or missing required box BLOCKS the
commit** (`scripts/check_diagnosis_evidence.sh`, run by the doctrine enforcer). Copy this into the leaf:

```markdown
## Acceptance Checklist (enforced)
- [ ] **REPRODUCE / ISSUE** — <tool command + symptom output proving the issue (parse REJECT, UNKNOWN=N, …)>
- [ ] **ROOT CAUSE (WHY + WHERE)** — <debug-tool output naming the mechanism + location: predicate rejection / [plannable-probe] verdict / furthest_position= + file:line or rule>
- [ ] **FIX** — <the minimal change + fix-hierarchy tier: declarative > grammar > engine>
- [ ] **ADDRESSED (verified)** — <before→after on the symptom: REJECT→PASS, UNKNOWN N→M>
- [ ] **NO REGRESSION** — <cert seeds 0/7/42 spf=0; the 6 fully-certified grammars byte-identical; external corpus 14/14; ast_shape_contract GREEN; clippy clean>
- [ ] **LOCKSTEP** — <book / contract / ledger / schema updated, or N/A + reason>
```

**Hard-gated (required, must be ticked + evidence-backed): ROOT CAUSE, ADDRESSED, NO REGRESSION** —
these are the director's named steps (analyse → root cause → addressed → no regression). REPRODUCE /
FIX / LOCKSTEP are part of the template and good practice, but not hard-blocked, to avoid
false-positives. The whole task-tree's "start→finish" is then the sequence of its leaves, each
passing this checklist, plus the tree's own Acceptance Criteria.

**A box is EARNED, not ticked.** A `[x]` you write is a *claim*; the proof is the **oracle re-run**.
The ADDRESSED and NO-REGRESSION boxes must cite a **named, re-runnable oracle** (the exact gate /
command + its deterministic result — e.g. cert-coverage at seeds 0/7/42, `ast_shape_contract_gate`,
the byte-identical check across the fully-certified grammars, the external corpus 14/14), so CI
re-executes exactly that and earns the box independently of your tick. A self-ticked-but-false box
passes the local presence check and **fails when the oracle is re-run** (locally via the `make`
gates, un-bypassably in CI). The local pre-commit hook is leg 1 (presence) — necessary, not
sufficient; the oracle re-run (`DOCTRINE_ENFORCEMENT.md` §6.1, leg 3) is what makes the box
un-self-tickable. Honest gap: that re-run is at CI, currently manual-only.

## The two binaries

| Binary | Build | Path | Used for |
|---|---|---|---|
| `parseability_probe` | `cargo build --release --features generated_parsers` (from `rust/`) | `./rust/target/release/parseability_probe` | parse a file, dump AST, trace, dashboards |
| `ast_pipeline` | `cargo build --features "generated_parsers ebnf_dual_run"` (from `rust/`, debug) | `./rust/target/debug/ast_pipeline` | certificate-coverage, lint, gen-AST, stimuli, `.ebnf`-direct |

`ast_pipeline` needs `--features ebnf_dual_run` to read a `.ebnf` directly, and `--features
generated_parsers` for certificate-coverage (it verifies witnesses through the real generated parser).

---

## Quick chooser — symptom → tool

| Symptom / question | Go to |
|---|---|
| "Does this file parse? Where does it fail?" | [1.1 `--parse`](#11---parse--supports) + [3.2 furthest-position](#32-furthest-position-error-diagnostic) |
| "What AST did it produce? Is the shape right?" | [1.2 `--parse-dump-ast-pretty`](#12---parse-dump-ast-pretty) |
| **"Parse an input against an ARBITRARY / synthetic grammar (not registered)?"** | [1.3 the `scratch` slot](#13-the-scratch-slot--drive-the-toolbox-on-an-arbitrary-grammar) |
| "A `@predicate` rejected valid input — which one, why?" | [2.4 predicate self-explaining trace](#24-predicate-self-explaining-trace) |
| "The parse is slow / stuck — which rules dominate?" | [3.1 `--dump-rule-call-counts`](#31---dump-rule-call-counts) |
| "I need to watch the parser step by step" | [2.1 trace verbosity](#21-trace-verbosity) + [2.2 `--trace-rules`](#22---trace-rules) |
| "How trustworthy is this grammar? proof/witness/UNKNOWN?" | [4.1 `--report-certificate-coverage`](#41---report-certificate-coverage) |
| "Give me ALL the UNKNOWN rules" | [4.2 `PGEN_CERT_COVERAGE_DUMP_ALL`](#42-pgen_cert_coverage_dump_all) |
| **"WHY is this rule UNKNOWN / not witnessed?"** | [4.3 `PGEN_CERT_COVERAGE_DEBUG_PROBES`](#43-pgen_cert_coverage_debug_probes) → [the 3-step protocol](#protocol-a-diagnose-an-unknown-3-steps) |
| "Which path did the witness planner take?" | [4.4 `PGEN_REACH_PATH_DUMP`](#44-pgen_reach_path_dump) |
| "Which residual `UNKNOWN`s are profile-excluded by construction?" | [4.6 `PGEN_CERT_RESIDUAL_CLASSIFICATION`](#46-pgen_cert_residual_classification) |
| "Is my grammar well-formed (LR / shadowing / non-terminating)?" | [5.1 `--lint-grammar`](#51---lint-grammar) |
| "What IR do the generators actually consume?" | [5.2 `--dump-gen-ast`](#52---dump-gen-ast) |
| "Packrat memo hit/miss perf?" | [3.3 `PGEN_REPORT_MEMO_STATS`](#33-pgen_report_memo_stats) |

---

## 1. Parse & inspect (`parseability_probe`)

### 1.1 `--parse` / `--supports`
- **WHAT:** parse a file with a registered grammar; exit `0` on a fully-consuming parse, non-zero on rejection. `--supports` just checks a grammar is registered.
- **WHEN:** the first thing to run on any "does X parse / where does it fail" question; the minimal reproducer check after you isolate a construct.
- **HOW:**
  ```bash
  ./rust/target/release/parseability_probe --supports systemverilog
  ./rust/target/release/parseability_probe --parse systemverilog file.sv --profile sv_2017
  ```
- **OUTPUT:** `parse_full passed for grammar '…' on '…'` (rc 0), or an error line carrying both the surface position and the `furthest_position` (see 3.2).
- **Profiles:** SV accepts `2017`/`sv_2017`/`ieee1800-2017` and the `2023` equivalents.

### 1.2 `--parse-dump-ast-pretty`
- **WHAT:** parse + write the typed AST as pretty JSON (`--parse-dump-ast` for compact).
- **WHEN:** "did it parse the *right* shape?", verifying a return-annotation `{kind,…}`, or confirming a fact-gated construct realized (e.g. `grep -c context_member_method out.json`).
- **HOW:**
  ```bash
  ./rust/target/release/parseability_probe --parse-dump-ast-pretty systemverilog file.sv out.json --profile sv_2017
  ```
- **OUTPUT:** AST JSON at `out.json` (default `<grammar>_ast.json`). Bound size with `--max-bytes N` / `PGEN_PARSE_DUMP_AST_MAX_BYTES`.
- **ENTRY-RELATIVE (`--entry-rule RULE`, SV-AST-SHAPE-FIDELITY.2.4):** dump the AST parsed from an ALTERNATE start symbol via `parse_full_from`, so a rule that is **PEG-shadowed** or entry-relative (unreachable from the canonical entry) can be inspected in isolation. Essential for the `<invalid_sequence_access>` return-annotation corruption class: `parseability_probe --parse-dump-ast-pretty systemverilog scoped.sv out.json --profile sv_2017 --entry-rule interface_class_type` reveals the sentinel a canonical-entry parse would never reach. Wired for `systemverilog` and `scratch`; omit for the byte-identical canonical dump. (Empty store ⇒ only non-store-gated scoped branches — `this.` / `pkg::` — fire in isolation.)

### 1.3 The `scratch` slot — drive the toolbox on an ARBITRARY grammar
- **WHAT:** a blessed, throwaway registered grammar (`grammar_name` = `scratch`) whose body you overwrite freely, so an **arbitrary / synthetic grammar** becomes a first-class registered parser drivable by the WHOLE toolbox above (`--parse`, `--parse-dump-ast-pretty`, `--trace-rules`, `--report-certificate-coverage`, `--lint-grammar`) — with semantics identical to the shipped generated parsers (authoritative **by construction**). This is the tool for the "which alternative wins on this input?" / linter-soundness / grammar-authoring probe (the `--build-a-tool` answer to the arbitrary-grammar capability gap). PARSE-HARNESS approach 3.
- **WHEN:** you need to parse a one-off / made-up grammar that is NOT registered — e.g. "does `r := \"a\" | \"a\" \"b\"` on `\"ab\"` pick the first or the longest alt?" (the exact `A2.3`-style evidence).
- **HOW:**
  ```bash
  # 1. Edit grammars/scratch/scratch.ebnf (keep the entry rule named `scratch`), e.g.:
  #        scratch := "a" | "a" "b"
  make -C rust SHELL=/bin/bash focus_scratch                       # regen the scratch parser artifact
  (cd rust && cargo build --release --features generated_parsers --bin parseability_probe)
  printf 'ab' > /tmp/in.txt
  ./rust/target/release/parseability_probe --parse scratch /tmp/in.txt
  ./rust/target/release/parseability_probe --parse-dump-ast-pretty scratch /tmp/in.txt /tmp/out.json
  PGEN_TRACE_VERBOSITY=debug ./rust/target/release/parseability_probe --parse scratch /tmp/in.txt --trace-rules scratch
  ./rust/target/debug/ast_pipeline grammars/scratch/scratch.ebnf --lint-grammar
  ```
- **OUTPUT:** exactly as for any registered grammar (rc 0 + `parse_full passed`, the typed AST, branch-entry trace, `furthest_position` on reject). Restore the default fixture (`git checkout grammars/scratch/scratch.ebnf`) before committing. Full design + trust architecture: book chapter *The Parse Harness* + `docs/tasks/PARSE-HARNESS.md`.

---

## 2. Trace — watch the parser explain itself (`parseability_probe`)

### 2.1 Trace verbosity
- **WHAT:** five additive levels — `none` < `low` (🧭 errors/backtracks) < `medium` (🧩 success exits) < `high` (🔎 branch dispatch + **predicate verdicts**) < `debug` (🧠 + predicate mechanism / resolved args).
- **WHEN:** you need to see what the parser *did*. Start at `high` for predicate/branch questions; `debug` when `high` isn't enough.
- **HOW:**
  ```bash
  PGEN_TRACE_VERBOSITY=high ./rust/target/release/parseability_probe --parse systemverilog f.sv --profile sv_2017 --trace
  ```
  `--trace` alone implies `debug`. The `[file:position]` in each line is the **input byte position**, not a source line.
- **⚠️ VOLUME:** full trace on a real input is hundreds of MB — almost always scope it with `--trace-rules` (2.2).

### 2.2 `--trace-rules R1,R2,…`
- **WHAT:** activate trace ONLY inside the call-tree of the listed rules (100–1000× volume reduction). Implies `--trace`.
- **WHEN:** you already suspect a rule (e.g. from the dashboard, 3.1) and want just its story.
- **HOW:**
  ```bash
  PGEN_TRACE_VERBOSITY=debug ./rust/target/release/parseability_probe \
    --parse systemverilog f.sv --profile sv_2017 --trace-rules known_unscoped_covergroup_type_identifier
  ```
- **OUTPUT:** trace lines only within those rules' subtrees. Add `--trace-log-file dbg.log` to capture; strip ANSI with `sed 's/\x1b\[[0-9;?]*[a-zA-Z]//g'`.

### 2.3 `--trace-log-file [FILE]`
- **WHAT:** route trace to a file (default `trace.log`). **WHEN:** large traces you want to grep. **HOW:** append `--trace-log-file dbg.log` to any `--trace`/`--trace-rules` run.

### 2.4 Predicate self-explaining trace
- **WHAT:** at `high`/`debug`, every `@predicate` evaluation emits a verdict (`🛡️ PASSED/REJECTED/INAPPLICABLE`) and, on rejection, the resolved args and the `🚫 Rule '…' rejected by post predicate '…'` line.
- **WHEN:** a parser rejects valid input and you suspect a semantic gate (`has_fact`/`fact_attribute_equals`/`lacks_fact`/…). **This is Step 2 of the `UNKNOWN` protocol.**
- **HOW:** run 2.2 scoped to the suspect rule and grep:
  ```bash
  ... --trace-rules <rule> 2>&1 | grep -iE "🚫|🛡️|has_fact|fact_attribute|NEGATIVE"
  ```
- **OUTPUT (example):**
  ```
  🚫 Rule 'known_unscoped_covergroup_type_identifier' rejected by post predicate
     'fact_attribute_equals [type_name, "\foo", declaration_family, covergroup]'
     ↪ NEGATIVE: 3 facts of kind 'type_name' exist (none matched name "\foo")
  ```
  → the use-site identifier was never declared with the required fact.

---

## 3. Performance, stuck parses, error locus

### 3.1 `--dump-rule-call-counts [N]` (+ `--dump-rule-call-counts-exclude`)
- **WHAT:** live top-N per-rule call-count dashboard (refreshes 250 ms in place); keeps refreshing until a timeout kills the process, so you see the last snapshot.
- **WHEN:** a parse hangs / is slow and you don't yet know which rules dominate; to confirm "is the recursion in `expression`?".
- **HOW:**
  ```bash
  timeout 30 ./rust/target/release/parseability_probe --parse systemverilog f.sv --profile sv_2017 \
    --dump-rule-call-counts 20 --dump-rule-call-counts-exclude "trivia,identifier,lparen,rparen,comma,dot"
  ```
- **OUTPUT:** a live table of rule → call count, exclusion applied before the top-N so noise (`trivia`…) doesn't eat slots.

### 3.2 Furthest-position error diagnostic (always on)
- **WHAT:** every parse-failure error is augmented with `furthest_position` — the deepest byte any branch reached (even backtracked), which is where the real defect lives (the surface position is often megabytes shallower).
- **WHEN:** any "did not consume full input at position N" — map `furthest_position` to a line instead of bisecting.
- **HOW:**
  ```bash
  ./rust/target/release/parseability_probe --parse systemverilog f.sv --profile sv_2017 2>&1 | tail -1
  # → Parser did not consume full input at position 113637 [furthest_position=643297, +529660 bytes deeper]
  F=643297; L=$(head -c "$F" f.sv | wc -l); sed -n "$((L-3)),$((L+3))p" f.sv
  ```

### 3.3 `PGEN_REPORT_MEMO_STATS`
- **WHAT:** print packrat memo hit/miss statistics. **WHEN:** perf triage of a slow parse. **HOW:** `PGEN_REPORT_MEMO_STATS=1 ./rust/target/release/parseability_probe --parse <g> f.sv`.

---

## 4. Certificate-coverage — the `UNKNOWN` / trustworthiness toolbox (`ast_pipeline`)

### 4.1 `--report-certificate-coverage`
- **WHAT:** for every rule, is it covered by a verified unreachability PROOF or a verified reachability WITNESS? Prints `proof/witness/UNKNOWN`. `UNKNOWN=0` with no failures = the objective "trustworthy on this grammar" number.
- **WHEN:** measuring a grammar's trustworthiness; the headline number before/after any grammar or generator change. **Always confirm determinism at seeds 0/7/42.**
- **HOW:**
  ```bash
  ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --report-certificate-coverage \
    --grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed 0
  ```
- **OUTPUT:** `CERTIFICATE-COVERAGE: … total=1300 witness=1244 UNKNOWN=55 (sample_parse_failures=0, proof_reverify_failures=0)`.

### 4.2 `PGEN_CERT_COVERAGE_DUMP_ALL`
- **WHAT:** prints the **full** `UNKNOWN` rule list (the default truncates to "25 of N") plus the `WARNING … NO reach path from the entry` dead-rule-candidate list.
- **WHEN:** Step 0 of the protocol — you need every residual rule by name to categorize them.
- **HOW:** prefix 4.1: `PGEN_CERT_COVERAGE_DUMP_ALL=1 ./rust/target/debug/ast_pipeline … --report-certificate-coverage …`.

### 4.3 `PGEN_CERT_COVERAGE_DEBUG_PROBES`
- **WHAT:** for each targeted `UNKNOWN`, prints a `[plannable-probe] rule='X' parsed=<b> witnessed_target=<b> sample="…"` line — the **forced witness sample** the pass generated and whether it parsed / hit the target.
- **WHEN:** **the WHY tool.** Step 1 of the protocol — tells you *how* each `UNKNOWN` failed.
- **HOW:**
  ```bash
  PGEN_CERT_COVERAGE_DEBUG_PROBES=1 ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf \
    --report-certificate-coverage --grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed 0 \
    > /tmp/probe.txt 2>&1
  grep "rule='<target>'" /tmp/probe.txt
  echo "parsed=false (malformed/store-gate): $(grep -c 'parsed=false' /tmp/probe.txt)"
  echo "parsed=true witnessed=false (reach gap): $(grep -c 'parsed=true witnessed_target=false' /tmp/probe.txt)"
  ```
- **READING:** `parsed=true witnessed_target=true` = witnessed; `parsed=true witnessed_target=false` = **reach/routing gap**; `parsed=false` = **malformed forced sample** (usually a store-gate rejection — confirm with 2.4).

### 4.4 `PGEN_REACH_PATH_DUMP`
- **WHAT:** prints the BFS hop chain (`reach_hops`) the planner installs to steer generation toward each target — the entry→target rule path + chosen branch per hop.
- **WHEN:** a `parsed=true witnessed_target=false` result — to see *which* path stole the bytes.
- **HOW:** prefix any generation/cert command: `PGEN_REACH_PATH_DUMP=1 ./rust/target/debug/ast_pipeline … 2>&1 | grep -i reach`.

### 4.5 Witness-pass knobs (A/B isolation; default-off / default-floor)
- `PGEN_WITNESS_NO_PURDOM=1` — disable Purdom shortest-derivation ordering (A/B: is the ordering the cause?).
- `PGEN_WITNESS_TIMEOUT_FLOOR_MS` — per-target witness budget floor (default 200 ms); raise to give a deep target more budget.
- `PGEN_GENERATION_STEPS_PER_MS` — steps→time calibration for the step budget.
- `PGEN_CERT_DIVERSE_GENERATION_TIMEOUT_MS` — per-sample deterministic step-budget (B1) for the cert-coverage **PASS-1 diverse pass** (default `4000` ms = 4 000 000 steps). Bounds the diverse generation so a deeply-recursive grammar cannot hang the cert (e.g. `rtl_const_expr` / `conditional_expr` at `--max-depth` 40/48 spin unboundedly without it; the budget cuts them deterministically in ~15 s). The default is far above any well-behaved grammar's per-sample cost (byte-identical cert for the 6 fully-certified grammars + SV at their canonical depths). Set `0` for the legacy unbounded pass deliberately. (CERT-GEN-BUDGET.2.)
- **NOT-depth check:** re-run 4.1 at `--max-depth 24`, `32`, `40`; an unchanged `UNKNOWN` set proves the per-target budget is adequate and the cause is a forcing/store-gate bug, not depth.

### 4.6 `PGEN_CERT_RESIDUAL_CLASSIFICATION`
- **WHAT:** read-only machine classification of the residual `UNKNOWN` set under a dialect profile into `profile_entry_unreachable` (P1: not positively reachable from any DECLARED entry over the active profile-filtered tree, satisfiability-honest edges) / `store_unproducible_under_profile` (P2: a mandatory positive store-gate — `has_fact`/`fact_attribute_equals`/`fact_count_at_least`≥1, never `lacks_fact` — whose kind no live rule can emit, cascaded to fixpoint) / `genuine` (the honest remainder). Reports `DEGRADED-INERT` when a live rule carries `@import_from_library`.
- **WHEN:** adjudicating a profiled residual (e.g. `verilog_2005`'s 327) — which entries are profile-excluded BY CONSTRUCTION vs genuinely actionable. Diagnostic only; headlines/generation untouched (byte-identical with the var unset).
- **HOW:** prefix 4.1; the entry universe = the cert entry + every `--cert-union-config` entry present in the active tree, so pass the alternate entries or an entry-relative cohort (`library_text`, …) will honestly read "unreachable from the single entry":
  ```bash
  PGEN_CERT_RESIDUAL_CLASSIFICATION=1 ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf \
    --report-certificate-coverage --grammar-profile verilog_2005 --entry-rule systemverilog_file \
    --count 40 --seed 0 --cert-union-config sv_multi_entry_root:verilog_2005 --cert-union-config library_text:verilog_2005
  ```
- **OUTPUT:** a `RESIDUAL-CLASSIFICATION` block after the `UNKNOWN` list — the three named lists, with per-rule reasons on the store class (`gate kind 'type_name' unproducible` / `mandatory descent forces '<rule>'` / `stranded by the store fixpoint`). Promotion to per-profile `proof` certificates = `VERILOG-2005-PROFILE.6.7`.

---

## 5. Static analysis & generation IR (`ast_pipeline`)

### 5.1 `--lint-grammar`
- **WHAT:** static well-formedness report — left-recursion info, non-terminating ERRORS, ordered-choice shadowing WARNINGS — then exits.
- **WHEN:** after any grammar edit; to adjudicate a `no_path`/dead-rule candidate; "is my grammar well-formed?".
- **HOW:** `./rust/target/debug/ast_pipeline grammars/<g>.ebnf --lint-grammar`.

### 5.2 `--dump-gen-ast` (+ `--dump-gen-ast-pretty`)
- **WHAT:** dump the **normalized generation-input AST** — the grammar IR the parser/stimuli generators actually consume (post LR-elimination etc.).
- **WHEN:** "what does the generator actually see?", debugging a codegen/stimuli surprise, diffing IR across a grammar edit.
- **HOW:**
  ```bash
  ./rust/target/debug/ast_pipeline grammars/<g>.ebnf --generate-parser \
    --dump-gen-ast gen_ast.json --dump-gen-ast-pretty --eliminate-left-recursion --output /tmp/p.rs
  ```

---

## 6. Coverage / gap reports (`ast_pipeline`)

- `--report-k-path-coverage K` — k-path (Havrikov-Zeller) coverage report at depth K (use 2–3); read-only.
- `--gap-report-json FILE` / `--gap-report-text FILE` (+ `--gap-report-threshold`) — emit the coverage-gap / unreachable-rule-debt report consumed downstream.
- `--target-report-input FILE` — load a prior gap report and apply its targets as generation priorities (the witness/closed-loop replay surface).

---

## Protocol A — diagnose an `UNKNOWN` (3 steps)

1. **Full list + honest number** — `PGEN_CERT_COVERAGE_DUMP_ALL=1 … --report-certificate-coverage … --seed 0` (re-run seeds 7, 42 for determinism).
2. **WHY each one failed** — `PGEN_CERT_COVERAGE_DEBUG_PROBES=1 …` → read `parsed` / `witnessed_target` per `[plannable-probe]` line.
3. **Exact rejection** — for a `parsed=false` rule, feed its forced sample back through `--trace-rules <rule>` at `PGEN_TRACE_VERBOSITY=debug` → the `🚫 rejected by post predicate …` line.

Cause map: `NO reach path` = dead-rule candidate (adjudicate via 5.1) · `parsed=true witnessed=false` = reach/routing gap · `parsed=false` + predicate-reject = store-gate (the precondition fact was never generated → store-aware generation).

## Protocol B — a parser rejects valid input
1. `--parse` and read `furthest_position` (3.2) → map to a line.
2. Minimal-reproduce that construct; `--parse` it. Fails on the minimal = localized; passes = contextual (bisect what precedes).
3. If a `@predicate` is suspected, `--trace-rules <suspect>` at `high`/`debug` (2.4) → the verdict names the rejecting predicate and resolved args.

## Protocol C — a parse is slow / hangs
1. `--dump-rule-call-counts 30 --dump-rule-call-counts-exclude "trivia,…"` (3.1) → the dominating rules.
2. `--trace-rules <dominator>` (2.2) → the actual call pattern; `PGEN_REPORT_MEMO_STATS=1` (3.3) for memo behavior.

---

*This catalog is part of the four-surface "impossible to ignore" debug-toolbox capture (book + KM +
decisions + project memory). Keep it in lockstep when a debug surface is added or changed.*
