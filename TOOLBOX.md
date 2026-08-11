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

⭐ **AND THE ROOT CAUSE BOX MUST ACTUALLY BE A ROOT CAUSE BOX** (`GENERATED-LINT-CORRECTNESS.9`).
The keyword matches `root cause` or `why <connector> where` — **not the bare word "why"**. It used
to, and that was FAILS-OPEN on a named step: a `**FIX**` box writing *"Why no lower tier: …"* and
quoting a command satisfied box 1 in a leaf carrying no ROOT CAUSE box at all. The same alternative
also failed CLOSED, blocking a complete leaf that happened to hold an *unticked* `**FIX**` box
mentioning "why". Write the template header and neither happens.

**Hard-gated (required, must be ticked + evidence-backed): ROOT CAUSE, ADDRESSED, NO REGRESSION** —
these are the director's named steps (analyse → root cause → addressed → no regression). REPRODUCE /
FIX / LOCKSTEP are part of the template and good practice, but not hard-blocked, to avoid
false-positives. The whole task-tree's "start→finish" is then the sequence of its leaves, each
passing this checklist, plus the tree's own Acceptance Criteria.

⭐ **THE SIGNATURE MUST SIT INSIDE THE BOX'S OWN BULLET** (box-scoped since
`GENERATED-LINT-CORRECTNESS.3`). A token elsewhere in the leaf — or in a co-staged tree file — does
not count. Pick the signature family that matches YOUR defect; there are **five**, and a defect that
fits none of them is a signal worth raising, not a reason to waive:

⭐⭐ **AND THE BOX MUST BE ONE THIS CHANGE WROTE** (leaf-scoped since `GENERATED-LINT-CORRECTNESS.7`).
A satisfying box must sit inside a **leaf section** (bounded by headings of level ≤ 3, so a
`#### Acceptance Checklist` block belongs to its `###` leaf) that your staged change **touches**,
and all three required boxes must be met within **one** file. ⛔ Box-scoping alone was *vacuous*:
`box_matches` accepted **any** ticked box in **any** staged task file, so a tree already holding one
compliant leaf supplied the checklist for every later leaf — measured, **33 tracked task files
carried that standing free pass**, and 7 of the last 138 code-change commits passed only by
borrowing. It is deliberately permissive *inside* a leaf: a follow-up commit editing any part of the
same leaf keeps its checklist, and a deletion-only edit still counts as touching it.

| # | family | when it applies | verbatim tokens that count |
|---|---|---|---|
| 1 | **correctness** | the parser accepts/rejects the wrong thing | `CERTIFICATE-COVERAGE:`, `[plannable-probe]`, `rejected by post predicate`, `furthest_position=`, `--trace-rules`, `--lint-grammar`, `--dump-rule-call-counts`/`--dump-rule-outcome-counts`, `--parse-dump-ast` |
| 2 | **performance / SPEED** | it is correct but slow | `/usr/bin/sample`, `otool` (annotated disassembly), `spindump`, `filtercalltree`, `ITIMER_PROF`, `self-time`, `call-graph attribution`, `cargo flamegraph` |
| 3 | **build integrity** | a target no longer COMPILES — no parse to trace, no run to sample | `error[EXXXX]`, `could not compile` |
| 4 | **codegen emission** | the GENERATOR emits the wrong code — it compiles and parses fine | `GENERATED-CLIPPY-CORRECTNESS:`, `clippy::<lint>`, `PGEN_CLIPPY_GENERATED_STRICT` |
| 5 | **ops / build-flow** | the defect is in the repo's OWN scripts, Makefiles, hooks or tracking state — shell/make, so no rustc error either | `git ls-files`/`log -S`/`rev-list`/`fsck`/`reflog`/`diff-tree`/`merge-base`, `shellcheck`, `bash -n`, `make -n`/`make --dry-run`, `E2BIG`/`ENOSPC`/`EACCES`/`ARG_MAX`, `guard.<pid>.marker`, `reason=rss-budget\|free-floor\|disk-floor\|timeout` |

⛔ **A bare `file.rs:NNN` citation is NOT a signature, and neither is *"verified by grep"*.** Both
were measured and deliberately refused (`GENERATED-LINT-CORRECTNESS.4`): a line number is a
*location*, and "verified by grep" is a *claim* — the box asks for the output of a tool you ran.
Groups 2 and 5 exist because this repo's SPEED campaign and its ops defects were being forced to
waive an otherwise-correct gate; if you find a sixth such class, **open a leaf rather than waive**.

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
| **"A rule misbehaves / a quantifier iterates once / my rule seems ignored" — ASK FIRST: is that rule even the ENTRY?** | [1.1 first question](#11---parse--supports) — the entry is the rule DEFINED FIRST; `--lint-grammar` will not tell you, `--report-certificate-coverage` will |
| "Does this file parse? Where does it fail?" | [1.1 `--parse`](#11---parse--supports) + [3.2 furthest-position](#32-furthest-position-error-diagnostic) |
| **"The probe said `stream did not contain valid UTF-8` / the reported position looks shifted"** | the probe DECODES (BOM → UTF-8/UTF-16 → ISO-8859-1) since `SV-CORPUS-GRAD.12c.1` and prints `source-encoding: …` on stderr for anything that is not plain UTF-8. ⛔ Those positions are offsets into the **decoded** text, not the file. A remaining refusal means a BOM that CONTRADICTS the body — the message names the byte offset |
| "What AST did it produce? Is the shape right?" | [1.2 `--parse-dump-ast-pretty`](#12---parse-dump-ast-pretty) |
| **"Parse an input against an ARBITRARY / synthetic grammar (not registered)?"** | [1.3 the `scratch` slot](#13-the-scratch-slot--drive-the-toolbox-on-an-arbitrary-grammar) (full CLI toolbox) · [1.4 compile-and-run](#14-the-compile-and-run-harness--parse-an-arbitrary-grammar-with-no-registry-edit--no-pgen-rebuild) (in-process, authoritative by construction) · [1.5 the interpreter](#15-the-grammar-ast-interpreter--parse-an-arbitrary-grammar-in-process-with-no-codegen--no-compile) (in-process, NO compile) |
| **"Which alternative WINS this choice on this input? Is this branch LIVE or dead?"** | [Protocol D](#protocol-d--which-alternative-wins-this-choice--is-this-branch-live-the-a22a23-class-probe) — scratch slot + the `🏁 selected branch N/M` trace line + the 1.7 policy matrix |
| **"Is the interpreter byte-identical to the generated parser? which input diverges?"** | [1.6 the differential-equivalence gate](#16-the-differential-equivalence-gate--is-the-interpreter-byte-identical-to-the-generated-parser) |
| **"Is the interpreter byte-identical PER COMBINATOR (on a synthetic grammar, in isolation)?"** | [1.7 the structural combinator suite](#17-the-structural-combinator-suite--is-the-interpreter-byte-identical-per-combinator) |
| **"Is the interpreter byte-identical on the SEMANTIC-DIRECTIVE surface (`@predicate`/`@emit_fact`/scope/rollback/memo×store)?"** | [1.8 the semantic-directive orchestration suite](#18-the-semantic-directive-orchestration-suite--is-the-interpreter-byte-identical-on-the-store-gated-surface) |
| **"Can `grammars/ebnf.ebnf` REPLACE the hand-written frontend? Does the meta-parser read this grammar the SAME WAY?"** | [1.9 the envelope differential](#19-the-frontendmeta-parser-envelope-differential--does-ebnfebnf-read-a-grammar-the-same-way-the-hand-written-frontend-does) — ⛔ a parse verdict cannot answer this; it found two live `ebnf.ebnf` defects that both parse `Ok` |
| "A `@predicate` rejected valid input — which one, why?" | [2.4 predicate self-explaining trace](#24-predicate-self-explaining-trace) |
| "The parse is slow / stuck — which rules dominate?" | [3.1 `--dump-rule-call-counts`](#31---dump-rule-call-counts) |
| "I need to watch the parser step by step" | [2.1 trace verbosity](#21-trace-verbosity) + [2.2 `--trace-rules`](#22---trace-rules) |
| "How trustworthy is this grammar? proof/witness/UNKNOWN?" | [4.1 `--report-certificate-coverage`](#41---report-certificate-coverage) |
| "Give me ALL the UNKNOWN rules" | [4.2 `PGEN_CERT_COVERAGE_DUMP_ALL`](#42-pgen_cert_coverage_dump_all) |
| **"WHY is this rule UNKNOWN / not witnessed?"** | [4.3 `PGEN_CERT_COVERAGE_DEBUG_PROBES`](#43-pgen_cert_coverage_debug_probes) → [the 3-step protocol](#protocol-a-diagnose-an-unknown-3-steps) |
| "Which path did the witness planner take?" | [4.4 `PGEN_REACH_PATH_DUMP`](#44-pgen_reach_path_dump) |
| "Which residual `UNKNOWN`s are profile-excluded by construction?" | [4.6 `PGEN_CERT_RESIDUAL_CLASSIFICATION`](#46-pgen_cert_residual_classification) |
| **"WHY is this closed-loop coverage target still residual?" — ⛔ the pass-summary counters are TARGET-scoped and read 0 while branches die** | [6.1 `failure_reasons`](#61-failure_reasons--why-a-residual-coverage-target-survived-already-written-no-re-run) — the per-branch record every gate run already wrote |
| **"`--max-depth` is 20 — why does the log say `max_depth=448`? and does that escalation ever PAY?"** | [6.2 depth-slack retry census](#62-depth-slack-retry-census--does-the-generators-depth-escalation-ever-pay) — the retry adds slack to the LIVE budget, so nesting is cumulative; the census prices each rung |
| **"`store_entry_raises=N` — but how many of those raises were actually NEEDED?"** | [6.3 blocked-verdict census](#63-store-entry-blocked-verdict-census--is-a-witness-entry-raise-actually-needed-or-is-the-verdict-over-approximating) — names each blocked target SPURIOUS vs GENUINE; measured 15 of 16 spurious on `sv_2017` |
| "Is my grammar well-formed (LR / shadowing / non-terminating)?" | [5.1 `--lint-grammar`](#51---lint-grammar) |
| "What IR do the generators actually consume?" | [5.2 `--dump-gen-ast`](#52---dump-gen-ast) |
| "Packrat memo hit/miss perf?" | [3.3 `PGEN_REPORT_MEMO_STATS`](#33-pgen_report_memo_stats) |
| "EXACT per-rule entry counts for a parse (machine-readable)?" | [3.4 `--dump-rule-entry-counts-json`](#34---dump-rule-entry-counts-json) |
| "How much parse work is DISCARDED (failed speculation)? committed vs wasted per rule?" | [3.5 `--dump-rule-outcome-counts-json`](#35---dump-rule-outcome-counts-json) |
| **"Is the memo actually SERVING this rule?" — ⛔ `rule_memo_hit_counts` FUSES success replays with cached failures; 208 "hits" were 208 failures and 0 replays** | [3.6 memo insert/evict/replay census](#36-per-rule-memo-insert--evict--replay-census--is-the-memo-actually-serving-this-rule) |
| "Which rules could a derived DFA scanner fuse? the measured ceiling? the choice-site / merged-choice surface?" | [5.3 `--report-fusibility-census`](#53---report-fusibility-census) |
| "Which rules exist under which `@profiles`? Which rules can a corpus run under profile P ever exercise?" | [5.4 `--dump-rule-profiles`](#54---dump-rule-profiles) |
| "Which LRM chapters/clauses does the keyed corpus target? Where is the negative-axis gap?" | [5.4 companion — `corpus_clause_coverage.py`](#54---dump-rule-profiles) |

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
- **⭐ FIRST QUESTION ON ANY "THIS RULE MISBEHAVES" (`QUANT-PLUS-ITER.1`): *is the rule you are
  testing the rule being run?*** The canonical entry is `rule_order[0]` — **the rule DEFINED FIRST in
  the grammar file** — so a helper rule written above your start symbol silently re-roots the
  grammar, and `--lint-grammar` will NOT tell you (an unreferenced rule counts as a root ⇒
  `unreachable_rules=0`, exit 0). Confirm the entry before interrogating any rule's internals:
  ```bash
  # names the resolved entry on its headline + flags rules with no reach path from it
  ./rust/target/debug/ast_pipeline grammars/<g>.ebnf --report-certificate-coverage --count 40 --seed 0
  # CERTIFICATE-COVERAGE: grammar='…' entry='stmt' … UNKNOWN=1 fully_certified=false
  #   WARNING … 1 UNKNOWN rules have NO reach path from the entry … ["scratch"]
  ```
  (Registered grammars only — it verifies witnesses through a real generated parser.) Session #211
  spent a whole task-tree on a "`+` consumes one occurrence" bug that was this, and every one of its
  six exonerations was correct and irrelevant.
- **⚠️⚠️ ROUTING — `--entry-rule` ALSO CHANGES WHICH ENGINE RUNS (`QUANT-PLUS-ITER.3`, measured
  across all 10 generated parsers).** `parse_from` sets `self.bare_parse = false;` **unconditionally**,
  so an `--entry-rule` run always takes the PROTOCOL graph while a default `--parse` takes the fused
  `cascade_*` graph. ⇒ **a `--entry-rule X` vs default comparison varies TWO things** (start symbol
  AND execution graph). To isolate the start symbol, pass `--entry-rule` on **both** arms — including
  the canonical entry — so only the rule name differs. (See 2.1 for the same hazard under tracing.)
- **OUTPUT:** `parse_full passed for grammar '…' on '…'` (rc 0), or an error line carrying both the surface position and the `furthest_position` (see 3.2).
- **Profiles:** SV accepts `2017`/`sv_2017`/`ieee1800-2017` and the `2023`/`1364-2005` equivalents — the request spellings are GRAMMAR-DECLARED via `@profile_alias` in `systemverilog.ebnf` (PROFILE-ALIAS.2; the generated parser carries them in `GRAMMAR_PROFILE_ALIASES` and resolves them case-insensitively in `set_grammar_profile`).

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

### 1.4 The compile-and-run harness — parse an ARBITRARY grammar with NO registry edit / NO `pgen` rebuild
- **WHAT:** `pgen::parse_harness::compile_and_parse(grammar_ebnf, input, &opts) -> ParseOutcome` — runs the REAL codegen on an arbitrary `.ebnf`, compiles the emitted parser as a **throwaway external crate**, runs it, and returns `{accepted, furthest_position, error, ast_json}`. Authoritative **BY CONSTRUCTION** (the shipped codegen + runtime); self-contained (touches neither the registry nor `pgen`). PARSE-HARNESS approach 2 — the in-process API a gate/oracle can call (vs the scratch slot 1.3, which is the CLI-toolbox path). It IS the CI oracle for the `.5` differential-equivalence gate.
- **WHEN:** you need the verdict + typed AST for a synthetic grammar **programmatically** (a Rust test/gate), without the scratch slot's `focus_scratch` + probe-rebuild ceremony; or to cross-check the scratch slot / interpreter.
- **HOW (Rust):**
  ```rust
  use pgen::parse_harness::{compile_and_parse, CompileAndParseOptions};
  let out = compile_and_parse(
      std::path::Path::new("grammars/scratch/scratch.ebnf"),
      "hello, world!",
      &CompileAndParseOptions::default(), // .entry_rule = alternate start; .workdir = reuse to keep pgen cached
  )?;
  assert!(out.accepted);              // out.furthest_position / out.error on reject; out.ast_json (typed AST) on accept
  ```
- **OUTPUT:** a `ParseOutcome` whose `ast_json` is **byte-identical** to `parser_registry::parse_sample_ast_json` for a registered grammar (pinned by the integration test `parse_harness::tests::compile_and_run_harness_reproduces_json_registry_verdict_and_ast`). Needs an `ast_pipeline` binary built with `--features ebnf_dual_run` (the standard `target/debug/ast_pipeline`). **Feature-surface tripwire (PARSE-HARNESS.10):** before any codegen the harness probes the binary with `ast_pipeline --report-feature-surface` (a feature-independent pre-clap flag printing `AST-PIPELINE-FEATURE-SURFACE: ebnf_dual_run=<b> generated_parsers=<b>`) and REFUSES a stale single-feature binary up front with the exact dual-feature rebuild command — the #140-class trap (`make focus_*` / census-CLI builds silently overwrite the dual-feature binary at the same path) now fails actionably instead of as a generic mid-gate codegen error. First probe compiles `pgen` as a dep (~seconds→minutes cold); reuse `opts.workdir` to keep it warm. Full design: book chapter *The Parse Harness* + `docs/tasks/PARSE-HARNESS.md`.

### 1.5 The grammar-AST interpreter — parse an ARBITRARY grammar IN-PROCESS with NO codegen / NO compile
- **WHAT:** `pgen::parse_harness_interpreter::interpret_parse(grammar_ebnf, input, &opts) -> ParseOutcome` — an in-process **interpreter** that dynamically dispatches over the normalized gen-AST (the `--dump-gen-ast` IR) reusing the shipped `ParseNode`/`ParseContent`/semantic runtime, so it needs **neither codegen nor a compile**. Same `ParseOutcome{accepted, furthest_position, error, ast_json}` as `compile_and_parse` (1.4), so the two are directly diffable. Authoritative **BY VERIFICATION** — a second implementation whose thin dispatch layer is proven byte-identical to the generated parser by a differential oracle. PARSE-HARNESS approach 1 (`.4` core). The fast path for "which alternative wins on this input?" / grammar-authoring / linter-soundness probes when you do NOT want the per-probe `rustc` compile of 1.4.
- **WHEN:** you need the verdict + typed AST for a synthetic grammar **fast and in-process** (a Rust test/gate/REPL-style probe), and the grammar is on the `.4` structural + return-annotation surface (no store-gated `@predicate`/`@emit_fact` parse outcomes — those are the `.5`/`.6` extension). Cross-checks the scratch slot / compile-and-run harness with no compile.
- **HOW (Rust):**
  ```rust
  use pgen::parse_harness_interpreter::{interpret_parse, InterpretOptions};
  let out = interpret_parse(
      std::path::Path::new("grammars/json.ebnf"),
      r#"{"a": [1, true, null]}"#,
      &InterpretOptions::default(), // .entry_rule = Some("rule") for an alternate start symbol
  )?;
  assert!(out.accepted);            // out.furthest_position / out.error on reject; out.ast_json on accept
  // feature-independent core (already-normalized gen-AST): interpret_parse_gen_ast(tree, order, anns, entry, input)
  ```
- **OUTPUT:** a `ParseOutcome` whose `ast_json` is **byte-identical** to `parser_registry::parse_sample_ast_json` for a registered grammar (pinned by `parse_harness_interpreter::tests::interpreter_is_byte_identical_to_the_json_registry_parser`) and to `compile_and_parse` (1.4) on synthetic grammars. Needs `--features ebnf_dual_run` (the `.ebnf` frontend). No `rustc` per probe → the fast oracle-side of the `.5` differential-equivalence gate. Full design: book chapter *The Parse Harness* + `docs/tasks/PARSE-HARNESS.md` §13.

### 1.6 The differential-equivalence gate — is the interpreter byte-identical to the generated parser?
- **WHAT:** `make -C rust SHELL=/bin/bash parse_harness_equivalence_gate` (module `rust/src/parse_harness_equivalence.rs`) — the certifying oracle for the interpreter (1.5). For each registered grammar it runs the interpreter AND the shipped generated parser over ONE deterministic stimuli corpus (seeds 0/7/42, bounded generation, large-stack workers) and asserts **byte-identical** verdict + typed AST. Report-first API (`evaluate_grammar_equivalence` collects divergences, never panics) + 3 enforcing gate tests. PARSE-HARNESS.5.
- **WHEN:** after ANY interpreter change (regression guard); to MEASURE whether the interpreter matches the generated parser on a grammar; to see WHICH input first diverges and WHERE the two ASTs differ (the divergence names the grammar + sample + byte offset). The scouting probes (`cargo test … parse_harness_equivalence::measurement -- --ignored --nocapture`) print the full per-grammar map + a regex-divergence minimizer.
- **HOW:**
  ```bash
  make -C rust SHELL=/bin/bash parse_harness_equivalence_gate
  # measurement / scouting (per-grammar CLEAN/DIVERGE + first divergence):
  cargo test --features "generated_parsers ebnf_dual_run" --lib \
    parse_harness_equivalence::measurement::measure_equivalence_across_registered_grammars -- --ignored --nocapture
  # isolate one grammar under a shell timeout (no-memo interpreter can be slow on deep samples):
  PGEN_PHEQ_ONLY=regex cargo test --features "generated_parsers ebnf_dual_run" --lib \
    parse_harness_equivalence::measurement::measure_equivalence_across_registered_grammars -- --ignored --nocapture
  ```
- **OUTPUT:** the gate tests pass/fail (`certified_grammars_are_byte_identical`, `deferred_grammars_are_still_divergent_or_promote_them`, `every_registered_grammar_is_classified_exactly_once`, `comment_arm_suppression_matrix_is_pinned`). CERTIFIED (byte-identical, **11**): json, semantic_annotation, rtl_frontend, vhdl, systemverilog (sv_2017), scratch, **regex** + **systemverilog_preprocessor** (promoted by `.5.1`, session #42), **ebnf** (promoted by `.5.2`, session #43), **return_annotation** (promoted by `.5.3`, session #44), **rtl_const_expr** (promoted by `.5.5`, session #45 — a CORPUS fix, not a fidelity fix: a curated-input corpus, since its ~16-level precedence cascade is un-generatable within the bounded ladder). DEFERRED: *(empty — all promoted; the ratchet stays wired for a future divergent grammar)*. EXCLUDED (oracle is not own-grammar codegen): builtin_return_annotation, builtin_semantic_annotation. The `.5.1` regex fidelity closure mirrored four codegen decisions in the interpreter — whitespace-sensitive layout policy / unresolved-reference built-ins (`builtin_any_char`/`builtin_ascii_char`) / `@transform` numeric coercion + the PCRE2 post-parse contract / `@profiles` dialect gating — general primitives that also closed svpp. The `.5.2` ebnf closure added a fifth: per-introducer **comment-arm suppression** — the interpreter's layout skippers gate the `#`/`//`/`/*` comment arms via codegen's OWN predicate (`comment_arm_suppression_for_grammar`), so a grammar that claims an introducer as a real token (ebnf's `block_comment := "/*" …`) is matched structurally, not skipped as trivia. The `.5.3` return_annotation closure added a sixth (a SHARED codegen fix, not interpreter-only): the `_pgen_lr_chain` `wrapper_specs` blob (the serialized per-alt `annotation_template`s of an LR-eliminated rule) was serialized from a non-deterministic std `HashMap` (`UnifiedReturnAST::Object`) — codegen froze one arbitrary order, the interpreter re-serialized a fresh non-deterministic order each load; a `serialize_with` SORTED serializer on `UnifiedReturnAST::Object.properties` canonicalizes every site (also closing a latent codegen non-determinism). The `.5.5` rtl_const_expr closure is NOT a fidelity fix but a CORPUS fix: the stimuli generator yields zero usable samples for its ~16-level precedence cascade within the bounded ladder (depths ≤28 fail `depth exceeded`, ~32 emits pathologically-huge expressions, ≥40 hang), so a general parser-agnostic **curated-input corpus** (`CURATED_CORPUS`, keyed by grammar name) gives the differential inputs — the generated parser still supplies the authoritative verdict+AST, so curated INPUTS carry no mirror risk; the interpreter is byte-identical over it (151 samples), confirming a corpus gap, not a divergence. Deep-stress scouting: `PGEN_PHEQ_ONLY=<g> … parse_harness_equivalence::measurement::probe_layout_deep_stress -- --ignored --nocapture` (ladder 6-30, 5 seeds; covers regex/svpp/ebnf); the `.5.3` byte-level scout is `probe_return_annotation_divergence`. Full map: book chapter *The Parse Harness* → *The differential-equivalence gate* + `docs/tasks/PARSE-HARNESS.md` §14/§15/§16/§17/§18.

### 1.7 The structural combinator suite — is the interpreter byte-identical PER COMBINATOR?
- **WHAT:** `make -C rust SHELL=/bin/bash parse_harness_combinator_gate` (module `rust/src/parse_harness_combinator_suite.rs`) — the per-combinator differential (PARSE-HARNESS.6.1). The `.5` gate (1.6) proves byte-identity only over constructs the SHIPPED grammars use; this suite proves it **per structural combinator** on **35 small synthetic isolating grammars** (one per construct the `.4` interpreter dispatches), each run through BOTH the interpreter (1.5) and the compile-and-run oracle (1.4) and asserted byte-identical (verdict + `furthest_position` + typed AST). This is the coverage that upgrades the claim from "trusted on the shipped grammars" to "trusted on ANY grammar built from PGEN's structural constructs" — the answer to the "which alternative wins on this synthetic input?" / A2.2/A2.3 linter-soundness question.
- **WHEN:** after ANY interpreter change (regression guard for the combinator surface); to prove/measure the interpreter on a *specific* structural combinator (choice under each `branch_policy`, `?`/`*`/`+` incl. zero-length guard, the four bounded forms `{N}`/`{N,M}`/`{N,}`/`{,M}`, lookahead `&`/`!`, sequence-backtrack, atom/regex-token, rule-ref, LR-eliminated, the three `@whitespace_sensitive` layout-policy shapes — insensitive default / `true` / `{ regex_tokens: true }`, WS-DIRECTIVE.2 — the two `@default_profile` shapes — permissive unspecified default / declared-default gate, DEFAULT-PROFILE.2 — and the two `@profile_alias` shapes — declared-spelling resolution / unknown-spelling pass-through, PROFILE-ALIAS.2, driven end-to-end via each case's `requested_profile`); to see the A2.2/A2.3 discrimination directly (longest_match accepts `a|ab` on `"ab"`, ordered rejects it — on both implementations, in agreement).
- **HOW:**
  ```bash
  make -C rust SHELL=/bin/bash parse_harness_combinator_gate
  # scouting: the full per-case, per-input interp-vs-oracle map
  cargo test --features "generated_parsers ebnf_dual_run" --lib \
    parse_harness_combinator_suite::measurement::measure_combinator_suite -- --ignored --nocapture
  ```
  ⛔ The `measure_direct_left_recursion_known_divergence` probe that used to live here is **retired**
  (`GRAMMAR-WELLFORMED.A2.5`): its subject is fixed, and direct LR is now certified by three
  first-class GATE cases instead of a print-only `--ignored` probe. See §1.7's honest bound.
- **OUTPUT:** 2 gate tests pass (`every_structural_combinator_is_byte_identical` — 35/35 CLEAN + the folded-in A2.2/A2.3 discrimination proof + the ENGINE-UNIVERSAL-SERVICES.8 declared-shape proof + the GRAMMAR-WELLFORMED.A2.5 direct-LR annotation-hoist proof; `combinator_coverage_is_complete`). Deterministic by construction (fixed grammars × curated inputs, no seeds); runs on a 512 MiB large-stack worker (the interpreter's logical recursion guard fires only past the 2 MiB default stack on the LR case). Bounded quantifiers `{N}`/`{N,M}`/`{N,}`/`{,M}` are covered first-class by the four `quant_bounded_*` cases since **BOUNDED-QUANT.1** closed the codegen half-wire (the canonical `parse_quantifier_bounds` decoder now also accepts the frontend's brace-stripped raw-AST spelling `"2,3"`; previously codegen aborted `Unknown quantifier` while stimuli generation accepted the same grammar — a generator⟷parser duality break). The grammar-level `@whitespace_sensitive` LAYOUT policy is covered first-class by the three `layout_*` cases since **WS-DIRECTIVE.2** replaced the grammar-NAME layout gate with the declarative directive (insensitive default / full `true` / granular `{ regex_tokens: true }` — so a whitespace-sensitive SYNTHETIC/scratch grammar is now expressible and probe-able). The grammar-level `@default_profile` DEFAULT-DIALECT policy is covered first-class by the `profile_unspecified_permissive`/`profile_default_gate` pair since **DEFAULT-PROFILE.2** replaced the regex→`pcre2` name literals with the declarative directive (no directive = permissive unspecified profile / directive = the declared default gates `@profiles` rules — so a profiled SYNTHETIC/scratch grammar can now declare its default and be probe-able). The grammar-level `@profile_alias` REQUEST-SPELLING policy is covered first-class by the `profile_alias_resolves`/`profile_alias_unknown_passthrough` pair since **PROFILE-ALIAS.2** replaced the engine alias tables (`parser_registry.rs` `"systemverilog"` arm + the global `main.rs` spelling table) with the declarative directive (a declared spelling resolves to its canonical profile / an undeclared spelling passes through un-coerced — driven end-to-end through each case's `requested_profile`, which both the interpreter and the compile-and-run oracle apply identically). The packrat memo × RUNTIME CYCLE-BREAKING composition is covered first-class by `recursion_guarded_memo_isolation` since **SV-CORPUS-GRAD.3.12**: an INDIRECT left-recursive cycle the LR eliminator does not rewrite, reaching one rule at the same position both from inside the cycle (guard-blocked) and from outside it (legal, longer match) — the shape in which a cycle-guard rejection cached under the stack-blind `(rule, position)` key refuses a valid parse. ⭐ The LR-eliminated rule's **declared AST** is covered first-class by `left_recursion_folded_ast` since **ENGINE-UNIVERSAL-SERVICES.8** — and it is the case that shows what byte-identity CANNOT prove: the pre-existing `left_recursion` case is annotation-free, so its typed AST is structural, and while every LR-eliminated rule in every grammar was publishing the eliminator's internal `_pgen_lr_chain` record instead of the author's shape, both implementations leaked identically and the suite stayed green. The new case declares object annotations on TWO distinct operators and the gate asserts the EXACT left-nested value against the declaration (plus the absence of any `_pgen_` marker), because agreement between two implementations of the same engine is not evidence either is right. ⭐ Bare DIRECT left-recursion (`A := A x | y`) stopped being an honest-bound and became a CERTIFIED combinator with **GRAMMAR-WELLFORMED.A2.5**: a pre-pass normalizes the inline-in-the-choice shape into the wrapper shape before planning, so the existing eliminator handles it and the old `furthest_position` divergence has no path left to occur. Three cases cover it — `direct_left_recursion` (agrees with the wrapper row input-for-input), `direct_left_recursion_multi_alt` (each alternative gets its own wrapper with its own `$N`), and `direct_left_recursion_folded_ast` (annotations SURVIVE the hoist — the `select_expression` shape, asserted exact against the declaration). ⛔ Honest bound (no silent caps): the retired probe those cases replaced had been unable to measure its oracle half since `QUANT-PLUS-ITER.2` made `@entry: true` mandatory — its grammar declared none, so codegen refused it and the probe printed the error instead of failing. It is `--ignored` and print-only, so nothing reported it; its stale numbers were still quoted here, in the book and in its own docstring. The class is tracked in `LANG-CAPABILITY-AUDIT.10.16`. Full map: book chapter *The Parse Harness* → *The structural combinator suite* + `docs/tasks/PARSE-HARNESS.md` §20 + `docs/tasks/BOUNDED-QUANT.md` + `docs/tasks/WS-DIRECTIVE.md` + `docs/tasks/DEFAULT-PROFILE.md` + `docs/tasks/PROFILE-ALIAS.md`.

### 1.8 The semantic-directive orchestration suite — is the interpreter byte-identical on the STORE-GATED surface?
- **WHAT:** `make -C rust SHELL=/bin/bash parse_harness_semantic_gate` (module `rust/src/parse_harness_semantic_suite.rs`) — the `.6.1` sibling for the **semantic-directive orchestration** surface (PARSE-HARNESS.6.2): **36 isolating grammars**, one or more per store-gated construct — `@predicate` gates in every phase (`pre`/`branch`/`post`, hit AND miss, plus **`final`** — the FINAL-PHASE-PREDICATE.2 whole-input deferred-obligation gate: a legal forward reference ACCEPTs, an undefined reference REJECTs at parse completion, and a losing tournament branch's obligation is rolled back; plus **the FINAL-PHASE-PREDICATE.3 multi-branch shaped-key case** — a `phase: final` shaped-key `$ref` on an `Or` rule under the DEFAULT `view: raw` resolves via the other-view fallback), `@emit_fact` + the full builtin query vocabulary (`has_fact`/`lacks_fact`/`fact_attribute_equals`/`fact_count_at_least`/`has_fact_in_current_scope`/`current_scope_is`), the scope tree (`@open_scope`/`@close_scope`), C3-B tournament rollback (a successful-but-losing branch's emissions must not persist), zero-length-success emission, `$reference` resolution (named raw-tree walk / shaped view / `.len` / the positional-`$N` hard-error parity / **positional `$N` under a `->` transform** — RAWCAP-TRANSFORM-PATH.2, raw captured before the transform for a positional raw-view predicate), the `value_compare` rule-span constraint (six ops + backtrack + under-transform), branch-start inline actions (INLINE-ACTIONS.2), library no-op parity, and **memoization × store** composition (gates re-evaluate fresh on memo hits; the memo is TAINT-GATED with write-epoch VALIDATION since MEMO-STORE-SOUNDNESS.2 — a store-consulting body's outcome is epoch-stamped, replayable only while the store is unchanged, evicted after any store write; all pinned) — each run through BOTH the interpreter (1.5) and the compile-and-run oracle (1.4), asserted **byte-identical** (verdict + `furthest_position` + typed AST). Landing the suite also landed the interpreter's semantic-orchestration mirror + split memo, closing the §13.4/§3.4 honest bound.
- **WHEN:** after ANY interpreter or semantic-runtime orchestration change (regression guard); to prove/measure a *specific* store-gated construct in isolation; to consult the pinned grammar-author facts (unquoted predicate-arg identifiers; inline branch predicates flatten rule-wide; positional `$N` never resolves in directive payloads; the memo is taint-gated with write-epoch validation (stale store-dependent entries never replay)).
- **HOW:**
  ```bash
  make -C rust SHELL=/bin/bash parse_harness_semantic_gate
  # scouting: the full per-construct, per-input interp-vs-oracle map
  cargo test --features "generated_parsers ebnf_dual_run" --lib \
    parse_harness_semantic_suite::measurement::measure_semantic_suite -- --ignored --nocapture
  ```
- **OUTPUT:** 2 gate tests pass (`every_semantic_construct_is_byte_identical` — 36/36 CLEAN; `semantic_construct_coverage_is_complete`). Deterministic by construction (fixed grammars × curated inputs). Honest bounds (§21.3): bootstrap facts + real library I/O are registry-owned (out of harness scope); no coverage lane. Full map: book chapter *The Parse Harness* → *The semantic-directive orchestration suite* + `docs/tasks/PARSE-HARNESS.md` §21.

### 1.9 The frontend⟷meta-parser ENVELOPE differential — does `ebnf.ebnf` read a grammar the SAME WAY the hand-written frontend does?
- **WHAT:** `make -C rust SHELL=/bin/bash ebnf_frontend_dual_run_gate` (module `rust/src/ebnf_envelope_differential.rs`, driver `ebnf_dual_run_diff --envelope-differential`) — the OUTPUT-level comparison between the two EBNF frontends. **Arm 1** is the hand-written Rust frontend (`pgen::ebnf_frontend`), which emits the flat `raw_ast` token envelope; **arm 2** is the parser GENERATED from `grammars/ebnf.ebnf`, whose return annotations shape a tree. The module projects arm 2 into arm 1's 14-kind envelope vocabulary and diffs them **token by token**, classifying every position as match / payload-not-comparable / payload-divergence / kind-divergence. Both arms run in ONE process from one input read, so the two sides can never be compared across stale artifacts. LANG-CAPABILITY-AUDIT.10.6 part 2.
- **WHEN:** ⛔ whenever the question is *"can `ebnf.ebnf` REPLACE the hand-written frontend?"*, or *"does the meta-parser understand this construct the way the frontend does?"*. **A parse verdict cannot answer either** — that is the whole point of this instrument. Reach for it after ANY `grammars/ebnf.ebnf` change; after any `ebnf_frontend.rs` change; and when a grammar behaves differently depending on which frontend read it. Also the right tool to SIZE a meta-grammar defect before fixing it: it names the rule, the token index and both arms' values.
- **HOW:**
  ```bash
  make -C rust SHELL=/bin/bash ebnf_frontend_dual_run_gate      # strict, all 14 grammars, with the ratchet
  make -C rust SHELL=/bin/bash ebnf_frontend_dual_run_diff      # report-only

  # one grammar, straight to the located divergence list:
  cd rust && cargo build --features ebnf_dual_run --bin ebnf_dual_run_diff
  ./target/debug/ebnf_dual_run_diff --input ../grammars/<g>.ebnf \
      --output /tmp/report.json --envelope-differential /tmp/envelope.json
  python3 -c "import json;d=json.load(open('/tmp/envelope.json'))['envelope_differential'];
  print(d['is_envelope_equivalent'], d['tokens_compared'], d['divergence_total']);
  [print(x) for x in d['divergences'][:10]]"

  # arm 2's raw typed AST, when you need to see the shape yourself:
  ./target/debug/ebnf_dual_run_diff --input ../grammars/<g>.ebnf \
      --output /tmp/r.json --emit-ast-json /tmp/arm2.json
  ```
- **OUTPUT:** a per-grammar report — `is_envelope_equivalent` (the frontend-REPLACEMENT verdict), `tokens_compared`, `token_matches`, `payload_not_comparable`, `payload_divergences`, `kind_divergences`, `tokens_unverified`, `divergence_total`, `unmapped_arm2_constructs`, `unresolved_include_directives`, and a located `divergences` list (rule name + token index + both arms' tokens, capped at 40 with the uncapped total alongside). Measured at the time of writing: **34 014 token positions across 14 grammars**, with **6 ENVELOPE-EQUIVALENT** — `builtin_return_annotation`, `builtin_semantic_annotation`, **`ebnf` itself (913/913)**, `rtl_const_expr`, `rtl_frontend`, `vhdl`.
- ⭐ **GROUND TRUTH — it refuses rather than guesses.** Every run first executes a **positive** control (a synthetic grammar that must project identically — 27/27 positions) and a **negative** control (a planted mutation the differ must catch exactly once, at exactly that index). A positive miss means the PROJECTION is broken; a negative miss means the DIFFER is blind. Either aborts before a number is published.
- ⚠️ **HONEST BOUNDS, carried in the report itself:** return-annotation payloads are compared by KIND only (arm 1 has raw source text, arm 2 has a parsed tree; recovering the text would need a pretty-printer whose own bugs would read as findings) and are counted under `payload_not_comparable`; arm 2 is known-blind to `[> … ]` lexical annotations; `systemverilog_lrm_profiled_wrapper`'s low agreement is the include asymmetry (arm 1 resolves `include(…)`, arm 2 stops at the directive), flagged by `unresolved_include_directives`, not a defect rate. A token after a rule's first KIND divergence is counted `tokens_unverified` — never as agreement, never as disagreement.
- **The gate carries a two-sided per-grammar ratchet** (`envelope_divergence_ceiling()` in the gate script): a count above the ceiling fails as a regression, a count *below* it fails too with "lower the ceiling", and a grammar with no declared ceiling fails. Both directions were proven to fire before the ratchet was trusted. Full map: `docs/tasks/LANG-CAPABILITY-AUDIT.md` `.10.6` + book *The Gate Flow* §7.9.

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
- **⚠️⚠️ ROUTING — TRACING CHANGES WHICH ENGINE RUNS (`QUANT-PLUS-ITER.3`, measured).** Every
  generated parser computes, at the top of `parse()`:
  ```rust
  self.bare_parse = !self.coverage_enabled && !self.logger_enabled
      && !self.counters_observed.get()
      && !crate::ast_pipeline::report_memo_stats_enabled();
  ```
  so **enabling a trace makes `logger_enabled` true and routes the parse off the fused `cascade_*`
  graph onto the PROTOCOL graph** — the same observability-twin routing already documented for
  memo-stats (3.3) and the counter dumps (3.4/3.5), but far easier to forget because tracing feels
  passive. ⇒ **a traced run is not necessarily the run that produced the untraced verdict.** The two
  graphs are held byte-identical by the equivalence/AST oracles (1.6), so this is a *reasoning*
  hazard, not a known correctness difference — but when you are chasing a verdict, say which graph
  you observed. Cost this trap real time in session #211.

### 2.2 `--trace-rules R1,R2,…`
- **WHAT:** activate trace ONLY inside the call-tree of the listed rules (100–1000× volume reduction). Implies `--trace`.
- **WHEN:** you already suspect a rule (e.g. from the dashboard, 3.1) and want just its story.
- **HOW:**
  ```bash
  PGEN_TRACE_VERBOSITY=debug ./rust/target/release/parseability_probe \
    --parse systemverilog f.sv --profile sv_2017 --trace-rules known_unscoped_covergroup_type_identifier
  ```
- **OUTPUT:** trace lines only within those rules' subtrees. Add `--trace-log-file dbg.log` to capture; strip ANSI with `sed 's/\x1b\[[0-9;?]*[a-zA-Z]//g'`.
- **⚠️ TRACE THE PARENT, NOT ONLY THE SUSPECT (`SV-CORPUS-GRAD.3.11`, measured).** The filter selects a
  rule's **call-tree**, so a rule's own outcome line is emitted in the frame that CALLED it — naming a
  leaf rule alone can print **nothing at all even while that rule is succeeding**. Measured on
  `module m; logic clk,a,s,b; sequence r; @(posedge clk) a ##1 s ##1 b; endsequence endmodule`:
  `--trace-rules time_literal` → **0** `Rule 'time_literal' successfully parsed` lines;
  `--trace-rules cycle_delay_range` (its caller) → **1**, and that line is the whole root cause.
  ⇒ an empty trace is **NOT** evidence a rule is unreached. Always add the suspected caller(s) —
  or confirm reach first with `--dump-rule-entry-counts-json` (3.4), which is call-site-independent.

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

### 3.2 Furthest-position error diagnostic (always on, EVERY family)
- **WHAT:** every parse-failure error is augmented with `furthest_position` — the deepest byte any branch reached (even backtracked), which is where the real defect lives (the surface position is often megabytes shallower).
- **WHEN:** any "did not consume full input at position N" — map `furthest_position` to a line instead of bisecting.
- **HOW:**
  ```bash
  ./rust/target/release/parseability_probe --parse systemverilog f.sv --profile sv_2017 2>&1 | tail -1
  # → Parser did not consume full input at position 113637 [furthest_position=643297, +529660 bytes deeper]
  F=643297; L=$(head -c "$F" f.sv | wc -l); sed -n "$((L-3)),$((L+3))p" f.sv
  ```
- ⭐ **"ALWAYS ON" IS TRUE SINCE `CORPUS-GRAD-ALL.2.0` — AND WAS FALSE WHEN THIS SECTION FIRST CLAIMED IT.**
  The augmentation was born as an inline `map_err` block in the SystemVerilog detail path
  (`SV-EXH-PROOF.3.3.4.b.6.2.25`) and hand-copied ONCE to `scratch`. Because it was a copied code
  block and not a shared function, it reached **2 of the 12** own-parser detail paths in
  `rust/src/parser_registry.rs` — not even uniformly within SV, since the `--library-in-dir` variant
  lacked it — while this catalog described it as universal. ⛔ **The cost was not theoretical:** on a
  flat item-list entry (`vhdl_file := design_unit*`, `json_file`, `grammar_file`, …) the surface
  position is only where the item list gave up, so every VHDL corpus rejection reported the START of
  the failing design unit. Measured on one OSVVM file: surface `1651` = `package … is` (names nothing)
  vs `furthest_position=3852` = `AxiBus : view … of … ;`, the actual unsupported VHDL-2019 construct —
  **+2 201 bytes, 58 source lines.** Stuck-point CLUSTERING of a corpus population
  (`stimuli/sv/cluster_rejects_valid.py` — **family-neutral despite its `sv/` home**, see below)
  keys on this bracket, so without it a 9 689-file rejection population collapses into two useless
  clusters. It is now ONE shared helper
  (`augment_error_with_furthest_position`) called by all 12, so a new family inherits it by
  construction. **Honest exclusion, by design:** `builtin_semantic_annotation` parses via the
  bootstrap `UnifiedSemanticAST::parse_bootstrap` and owns no parser object, so it has no furthest
  position to report.
- **THE CONSUMER — rank a corpus rejection population into defect classes (ANY family).**
  `stimuli/sv/cluster_rejects_valid.py` probes each rejecting file, reads `furthest_position`, and
  clusters by a normalized 3-token signature at that locus. Family-parameterized since
  `CORPUS-GRAD-ALL.2.1` (a `FAMILIES` table of keywords + multi-char operators + case-folding; add a
  family by adding an entry, never by copying the file). It still lives under `stimuli/sv/` because
  historical task leaves cite that path.
  ```bash
  # raw-fail lane (a family with no adjudication manifest yet), profile-less grammar:
  python3 stimuli/sv/cluster_rejects_valid.py --results stimuli/vhdl/characterization/results.tsv \
      --grammar vhdl --jobs 8 \
      --out docs/tasks/artifacts/corpus_grad_all/vhdl_fail_clusters.tsv \
      --summary docs/tasks/artifacts/corpus_grad_all/vhdl_fail_clusters.md
  # → rows: 9689  clusters: 861   (26 s; the ranked worklist leaves are cut from)
  # adjudicated lane (SV): defaults read the manifest and cluster only unexplained_rejects_valid
  ```
  ⚠️ The **raw-fail** lane SIZES a candidate class; it does not adjudicate one. Some corpus files are
  intentionally invalid, so a fail can be the correct outcome — expected verdicts come from the LRM /
  suite metadata, never from what the parser does today. ⛔ VHDL requires case-folding (it is a
  case-insensitive language); without it `ENTITY`/`entity` are separate clusters and neither shows
  its size. Feed it with `stimuli/run_external_corpus.sh <fam>` (VHDL: 13 720 files in **71 s**),
  whose column 3 is repo-root-relative — the clusterer REFUSES an absolute row.
- **A rejection SIGNATURE is decoration-independent.** `normalize_rejection_signature` (the
  duality-hunt / STIMULI-SIGNOFF clustering key) strips the ` [furthest_position=…]` bracket before
  collapsing digit runs: a signature names the failure CLASS, and after digit normalization the
  bracket is a constant suffix with zero discriminating power. That is why rolling the augmentation
  out to 12 families needed **no rebaseline** of the pinned signatures in
  `rust/test_data/grammar_quality/duality_hunt_gate_contract_v0.json`.

### 3.3 `PGEN_REPORT_MEMO_STATS`
- **WHAT:** print packrat memo hit/miss statistics. **WHEN:** perf triage of a slow parse. **HOW:** `PGEN_REPORT_MEMO_STATS=1 ./rust/target/release/parseability_probe --parse <g> f.sv`.
- **ROUTING (RGX-0078.5.i.7 D2-A):** setting the env routes the parse to the PROTOCOL graph (fused `cascade_*` fns carry no memo lane, so memo stats are truthful only there) — automatic, like every diagnostic consumer under the observability twin.

### 3.4 `--dump-rule-entry-counts-json`
- **WHAT:** after a `--parse`, write the parser's monotone per-rule ENTRY counters (every rule-method entry, successful AND backtracked — the always-on `fetch_add` on rule entry) as JSON `{grammar, accepted, total_entries, rule_entry_counts}`. The machine-readable dual of the live dashboard (3.1), which is stderr-only/refresh-based and useless for a sub-millisecond parse. Counts are a DELTA past a pre-parse baseline (SV's stdlib preload never pollutes them) and deterministic for a deterministic parser ⇒ a re-runnable oracle. Wired for every registered grammar's canonical-entry `--parse` path (not `--entry-rule`). RGX-0078.5.h.1.
- **WHEN:** quantifying rule-entry cost models (the ~262ns/entry rule-cascade analysis); feeding the fusibility census's measured entry share (5.3); any "how many times was rule R actually entered" question needing exact numbers, not a dashboard.
- **HOW:**
  ```bash
  ./rust/target/debug/parseability_probe --parse regex /tmp/pattern.txt --dump-rule-entry-counts-json /tmp/counts.json
  ```
- **OUTPUT:** the JSON file (rules sorted, zero-count rules omitted). Counts are build-mode-independent (a debug probe gives the same numbers as release).
- **ROUTING (RGX-0078.5.i.7 D2-A):** requesting this dump routes the parse to the PROTOCOL graph automatically — the `rule_call_counts()` accessor (grabbed pre-parse for the baseline) marks the parser's counter-consumer flag, so the fused cascade graph (which ticks no per-rule counters) never serves a counted parse. (The regex dedicated-stack lane constructs a fresh worker parser that needs no baseline, so it fires the trigger explicitly pre-parse — `RGX-0078.5.i.8.t1` fixed a lane that skipped it and undercounted to protocol-boundary frames only.)

### 3.5 `--dump-rule-outcome-counts-json`
- **WHAT:** after a `--parse` run with the transactional coverage stack enabled, write the raw monotone per-rule entry counters, the COMMITTED (surviving) per-rule counts, AND the per-rule memo-HIT counts as JSON `{grammar, accepted, total_entries, total_committed, total_memo_hits, rule_entry_counts, rule_committed_counts, rule_memo_hit_counts}` — plus the parse's semantic-store counter DELTAS (`store_counters`: rollbacks / facts emitted / rolled back / scopes± / predicate evaluations — RGX-0078.5.i.1 — and the checkpoint/delta-protocol exposure classification `rollbacks_unchanged` / `rollbacks_tournament` (=`extract_delta_since` calls) / `rollbacks_tournament_unchanged` (=empty-delta extractions) / `rollbacks_nonempty_chain` — RGX-0078.5.i.5, the P3c scout). ⚠️ OBSERVED-PARSE boundary (RGX-0078.5.j.4 `-0201`): this dump's pre-parse opt-in routes the parse to the PROTOCOL graph, so every counter it serializes is exact; a BARE parse (no diagnostic consumer — which this dump can therefore never describe) skips the diagnostic-only `rollbacks_nonempty_chain` classification, while the telemetry quartet and the REQUIRED `predicate_evaluations` memo-taint signal stay exact on every parse — the same documented boundary the per-rule entry counters already have. `raw − committed` = the rule's FAILED-speculation entries (every entry `try_parse` rolled back — the parse's probing waste); `raw − memo_hits` = the rule's BODY executions (a memo hit answers from cache without running the body — RGX-0078.5.i.4, the P1 inline census's lost-hit input; hits are recorded by `memoized_call` under the same coverage opt-in, so ordinary parsing pays nothing). Committed semantics are C3-B: tournament winners AND successful-but-losing branches both survive. Deterministic ⇒ a re-runnable oracle; committed counts are meaningful only for an ACCEPTED parse. RGX-0078.5.h.1b + .5.i.4.
- **WHEN:** quantifying wasted vs productive parse work per rule; feeding the choice-site + inline census's dynamic input (5.3 `--fusibility-outcome-counts`); any "how much of this parse was failing probes" or "how memo-hot is this rule" question.
- **HOW:**
  ```bash
  ./rust/target/debug/parseability_probe --parse regex /tmp/pattern.txt --dump-rule-outcome-counts-json /tmp/outcome.json
  ```
- **OUTPUT:** the JSON file (rules sorted; zero-count rules omitted per map). Opt-in: unset ⇒ the coverage stack stays disabled and behavior is byte-identical. Build-mode-independent like 3.4.
- **ROUTING (RGX-0078.5.i.7 D2-A):** the outcome dump enables coverage ⇒ the parse runs the PROTOCOL graph, so the raw/committed/memo-hit pins stay byte-exact forever under the observability twin (a BARE parse — no coverage/trace/counters/memo-stats consumer — runs the fused `cascade_*` graph instead; its byte-identity is enforced by the equivalence/AST oracles, not by counters).

### 3.6 Per-rule memo INSERT / EVICT / REPLAY census — "is the memo actually serving this rule?"
- **WHAT:** `docs/tasks/artifacts/sv_corpus_grad/memo_insert_evict_census.py` — splits a parse's memo
  activity per rule into **success inserts**, **stale-tainted evictions**, **success replays** and
  **failure hits**. Needs no engine change: the generated `memoized_call` already logs every memo
  transition at `PGEN_TRACE_VERBOSITY=debug`, keyed by numeric rule id; the script joins those lines
  against the parser's own `RULE_NAMES` table. `SV-CORPUS-GRAD.11a`.
- **WHEN:** ⛔ whenever a rule looks memo-served but the parse is still super-linear, and **before
  concluding anything from `rule_memo_hit_counts` (3.5)** — that field **FUSES three different memo
  paths** (replayed success, cached clean failure, cached tainted failure). The distinction is not
  academic: `conditional_statement` reports **208 hits** on the `.11a` chain at n=6, and **all 208
  are cached FAILURES with 0 success replays**. Reading the fused counter as "the memo already
  collapses this rule" is exactly the wrong conclusion, and it was drawn once (`-0194`, corrected by
  `-0195`).
- **HOW:**
  ```bash
  PGEN_TRACE_VERBOSITY=debug ./rust/target/release/parseability_probe --parse systemverilog \
      in.sv --profile sv_2017 --trace --trace-log-file t.log
  ./rust/target/release/parseability_probe --parse systemverilog in.sv \
      --profile sv_2017 --dump-rule-outcome-counts-json oc.json
  python3 docs/tasks/artifacts/sv_corpus_grad/memo_insert_evict_census.py t.log \
      --verify oc.json --rules rule_a,rule_b --top 20
  ```
- **OUTPUT:** a per-rule table over the nine memo transitions + the whole-parse totals. **Success
  replays are the packrat guarantee; everything else is bookkeeping.**
- ⭐ **GROUND TRUTH — it refuses rather than guesses.** `--verify` cross-checks the trace census
  against the independent atomic counters: `miss + hits` must equal `rule_entry_counts` exactly, and
  the id→name join is itself verified (every `RULE_*` const must index its own name). A mismatch
  exits 2 instead of publishing.
- ⛔ **The control is PARTITIONED, and you must know why: NOT EVERY RULE IS MEMOIZED.** A rule
  reached through the generated `inlined_frame_call` helper gets a full observable frame — entry
  counter, coverage push, enter/exit trace — but **no `memoized_call` at all**. On
  `systemverilog.ebnf` that is **663 of 1481 rules across 2871 call sites**, so a flat
  "every rule must balance" control fails on ~305 live rules and teaches you nothing. Strictly
  memoized rules must balance exactly; inlined-reachable rules may only fall short, and the
  shortfall is reported rather than dropped. ⚠️ ⇒ *"rule R has a `memoized_call` in its method"* does
  NOT mean R's executed path was memoized — check the inlined set first.
- **ROUTING:** both runs take the PROTOCOL graph (tracing and counters each clear `bare_parse` —
  2.1), which is the correct graph here: it is the one whose `memoized_call` is under study.

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
- ⛔⛔ **RUN THIS BEFORE YOU BLAME THE PLANNER — a probe SAMPLE cannot tell you what the plan ASKED
  for** (`ENGINE-UNIVERSAL-SERVICES.10`, measured). A `witnessed_target=false` sample shows where
  generation *ended up*; the hop dump shows what it was *instructed to do*. They are different
  instruments and the gap between them is the diagnosis. Reading only the samples produced a
  confident, **wrong** root cause that reached a task leaf *and* a tracked gate contract: SV's two
  `_lr_suffix` residuals were recorded as *"the reach planner cannot route to a rule that did not
  exist when it built its graph"*, when `PGEN_REACH_PATH_DUMP=1` shows a **complete, correct** 21-hop
  chain ending `("bins_selection","root/s3"), ("select_expression","root/s1/q")` — every OR steered,
  the quantifier forced. The real defect was one stack frame away, in the RENDER
  (`generate_quantified` re-forcing its site on every recursive re-entry). ⇒ the cause map below
  splits `parsed=true witnessed=false` into **plan-side** and **render-side**, and this dump is what
  separates them.
- **PAIR IT WITH** `PGEN_TRACE_VERBOSITY=high … | grep "Quantifier decision"` when the path crosses a
  `?`/`*`: a forced site prints `candidates=[1]` (exactly one repeat count, so a downstream failure
  has **no fallback** and propagates to the nearest OR, which *does* fall back — silently). Counting
  those lines is how a runaway forcing loop is caught: 119 at one site in a single probe was the
  `.10` signature, versus 5 after the fix.

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
- **WHAT:** static well-formedness report — left-recursion info; HARD-gated errors for non-terminating rules, ordered-choice shadowing (both verdicts are now SELECTION-SEMANTICS-CONDITIONAL: exact-duplicate fires only where the tie-break provably keeps the EARLIER twin — under `@associativity: right`, a later-higher `@priority`, or `@deterministic_group` evaluation-order rotation the engine SELECTS the later twin, so no verdict; under `@associativity: nonassoc` an equal-priority tie fails the whole choice — a distinct "restructure deliberately" verdict since merge/remove would change acceptance (A2.4); fixed-terminal-prefix ONLY on an `@branch_policy: ordered` rule with no branch-phase predicate and no partition rotation — under the default `longest_match`/`priority_first` the later alternative is LIVE (A2.3)), unreachable rules, **undefined references** (a rule referencing a rule never defined — codegen would emit a never-matching stub; the check runs on the UNFILTERED grammar and allowlists codegen's native builtins), unbound fact-kinds, and profile orphans; nullable-repetition warnings; always-succeeds notes — then exits (nonzero on any error-class finding).
- **WHEN:** after any grammar edit; to adjudicate a `no_path`/dead-rule candidate; **"every parse rejects at `furthest_position=0` and nothing points at the cause"** (the undefined-ref signature); "is my grammar well-formed?".
- **HOW:** `./rust/target/debug/ast_pipeline grammars/<g>.ebnf --lint-grammar`.

### 5.2 `--dump-gen-ast` (+ `--dump-gen-ast-pretty`)
- **WHAT:** dump the **normalized generation-input AST** — the grammar IR the parser/stimuli generators actually consume (post LR-elimination etc.).
- **WHEN:** "what does the generator actually see?", debugging a codegen/stimuli surprise, diffing IR across a grammar edit.
- **HOW:**
  ```bash
  ./rust/target/debug/ast_pipeline grammars/<g>.ebnf --generate-parser \
    --dump-gen-ast gen_ast.json --dump-gen-ast-pretty --eliminate-left-recursion --output /tmp/p.rs
  ```

### 5.3 `--report-fusibility-census`
- **WHAT:** the DERIVED-SCANNER capability-gate classifier (RGX-0078.5.h STEP-0, `rust/src/ast_pipeline/fusibility_census.rs`) — for every rule of the UNFILTERED grammar, is its subtree compilable to a direct-coded DFA `scan_R(pos)` under the increment-1 strict gate (regular + effect-free + text-folding + policy-encodable + layout-contiguous, each criterion mirroring the engine's own resolution)? Prints per-tier counts (`fusible_token`/`fusible_lookahead`/`not_fusible`, plus `shape_only` = language-encodable but value-blocked), the MAXIMAL fusible roots (future `scan_*` sites), a disqualification histogram, and the grammar's regex-literal ATOM-site count (the `match_regex`/self-hosting surface). With `--fusibility-entry-counts F1,F2,…` (files from 3.4) it prints the measured `FUSIBILITY-ENTRY-SHARE` — the share of real parse entries fusion would eliminate and the implied ceiling (a conservative lower bound). Read-only; verdicts are sound under-approximations.
- **WHEN:** gating/sizing any scanner-fusion or token-DFA work (which rules, how many real entries, what ceiling); asking "why is rule R not token-shaped?" (`PGEN_FUSIBILITY_DUMP_ALL=1` per-rule reasons + per-site verdicts); measuring a grammar's "token-shapedness" as a standing metric; measuring the increment-(ii) MERGED-CHOICE surface (every Or site's token-shaped branch subset + the measured discarded-work kill surface — RGX-0078.5.h.1b); sizing the P2 DEGENERATE-DISPATCH surface (RGX-0078.5.i.3 — which rule-top-level choice sites can byte-switch dispatch with the tournament protocol elided, what blocks the rest, and the measured Or-body-execution exposure); sizing the P1 CASCADE/WRAPPER-INLINING surface (RGX-0078.5.i.4 — which rules are inline-eligible under gates a–d: acyclic + directive-free frame + non-entry + no `@profiles`/`@transform`, with the failing gate NAMED per rule, the wrapper-class split, and the measured frame exposure incl. the memo-hit share P1b would re-execute).
- **HOW:**
  ```bash
  ./rust/target/debug/ast_pipeline grammars/regex.ebnf --report-fusibility-census \
    --fusibility-entry-counts /tmp/c1.json,/tmp/c2.json --fusibility-census-json /tmp/census.json
  # the .5.h.1b merged-choice join (files from 3.5):
  ./rust/target/debug/ast_pipeline grammars/regex.ebnf --report-fusibility-census \
    --fusibility-outcome-counts /tmp/o1.json,/tmp/o2.json
  PGEN_FUSIBILITY_DUMP_ALL=1 ./rust/target/debug/ast_pipeline grammars/regex.ebnf --report-fusibility-census
  ```
- **OUTPUT:** `FUSIBILITY-CENSUS: grammar=regex rules=274 fusible=68 (token=40 lookahead=28) not_fusible=206 (shape_only=75) maximal_roots=57 static_share=24.8%` + the roots list + histogram; with entry counts: `FUSIBILITY-ENTRY-SHARE: … eliminated_below_roots=83 at_roots=495 … ceiling≈1.04x`. Always also prints `CHOICE-SITE-CENSUS: … sites=N with_encodable_subset=M all_encodable=A` (every Or site classified for merged-choice); with outcome counts (3.5): `OUTCOME-SHARE: … total_entries=… committed=… discarded=… (on_encodable=… on_structural=…) ceiling≈…x` — discarded = raw − committed (failed-speculation work), and the kill surface = discarded entries on shape-encodable rules (choice/optional/iteration attempts alike), plus the top choice sites by sole-attributable discarded entries. Always also prints `DEGENERACY-CENSUS: … top_level_sites=N degenerate_dispatch=K` (RGX-0078.5.i.3 — the P2 byte-switch gate per top-level site: all branches first-byte-decided + pairwise-disjoint + terminal-ws-sensitive + no branch predicates/effects; blockers NAMED per site under `PGEN_FUSIBILITY_DUMP_ALL=1`, histogram always) and, with outcome counts, `DEGENERACY-EXPOSURE: … degenerate_site_entries=… committed=… discarded=…` — the Or-body executions the degenerate emission strips of tournament protocol. Always also prints `PREFIX2-CENSUS: … blocked_top_level_sites=N prefix2_dispatchable=K (wildcard_limited=W)` (RGX-0078.5.i.7 D1 — the FIRST₂ two-level-dispatch gate per blocked top-level site: the P2 gates minus first-byte disjointness, plus per shared first byte the admitting subset's second-byte facts RESOLVED and its non-WILDCARD members pairwise-disjoint on byte 2 — a byte-2 WILDCARD member is `len1_possible` (a 1-byte match leaves byte 2 unconstrained), nullable, OR `offset1_rule_entry` (the D1 emission's furthest-position-parity license: a byte-2-refuted attempt could still enter a rule at offset ≥1, so the emitted guard never byte-2-prunes it) — wildcards are legal but kill-capping; blockers + `[PREFIX2]` per site under `PGEN_FUSIBILITY_DUMP_ALL=1`, histogram always) and, with outcome counts, `PREFIX2-EXPOSURE: … prefix2_site_entries=… committed=… discarded=…` — the Or-body executions a two-level byte dispatch strips (byte-1-admitted/byte-2-refuted attempts never entered). Note the LANDED D1 emission is per-BRANCH FIRST₂ prune guards (needing NO byte-2 disjointness), so its kill surface EXCEEDS the census's dispatchable-site model; the census remains the site-level dispatchability instrument. Always also prints `INLINE-CENSUS: … rules=N inline_eligible=K (pass_through=… alternation_leaf=… shaped=…)` (RGX-0078.5.i.4 — the P1 rule-inlining gate: acyclic + directive-free frame + non-entry + no `@profiles`/`@transform`; blocker histogram always, per-rule class/refs/body-size + emission decision under `PGEN_FUSIBILITY_DUMP_ALL=1`) followed by `INLINE-DECISIONS: … decided_under_budget=K over_budget=M (expansion_cap=12 duplication_cap=192)` — the P1a EMISSION decision under the shared code-size budget (`fusibility_census::compute_inline_decisions`, the SAME function codegen consumes, so this report and the emitted parser cannot drift; `decided` rules' call sites receive the rule body inline under `inlined_frame_call`, memo preserved) — and, with outcome counts, `INLINE-EXPOSURE: … eligible_entries=… committed=… discarded=… memo_hits_on_eligible=…` — the collapsible wrapper frames plus the memo-hit share a memo-eliding inline (P1b) would re-execute (hits require dumps from a `.5.i.4`+ parser generation) — plus `INLINE-EXPOSURE-DECIDED: … decided_entries=… committed=… discarded=… memo_hits_on_decided=…` — the same sums restricted to the budget-DECIDED emission plan (`compute_inline_decisions`): the frames the landed P1a emission actually collapses, and the lost-hit surface P1b's memo elision re-executes at the budget. Always also prints `QUANT-SITE-CENSUS: … quantified_sites=N min_zero=M guardable=K (bare_ref=… no_refs=… mixed=…)` (RGX-0078.5.i.7 Q-GUARD — every `Quantified` site classified for FIRST-guarded ATTEMPT ELISION at min-0 sites; the frontier split is the `first_set::quantified_element_frontier` furthest-emulation exactness class the LANDED `-0077` emission consumes — `bare_ref` sites get the guard + the exact `furthest` emulation, `no_refs` sites the guard alone, `mixed` sites stay unguarded: gates = min-0 quantifier + terminal whitespace-sensitivity (the raw-byte peek) + element first-byte-decided via the SHARED `first_set::branch_dispatch_first_bytes` predicate + a Branch-predicate/branch-start-effect-free reachable closure; blockers named per site, histogram always; sites censused at ANY nesting — the elision is local to the quantifier loop, and a min-0 quantifier ALWAYS attempts its element exactly once at the current position, so the furthest-position diagnostic is EXACTLY emulable) and, with outcome counts, `QUANT-EXPOSURE: … attributable_rules=… entries=… committed=… discarded=… | shared_rules=…` — POPULATION-attributed: a rule counts only when EVERY grammar-wide reference occurrence sits under a guardable min-0 site's element subtree (occurrences counted at outermost guardable sites, so nesting never double-counts); honest bounds printed alongside (attributable discards OVER-approximate the kill — byte-1-admitted refutations survive; terminal-only sites are invisible to per-rule counters). Always also prints `CASCADE-CENSUS: … cascade_eligible=N (roots=R internal=I internal_cyclic=C) residual=X boundary_edges=B` (RGX-0078.5.i.7 D2 — the FULL-CASCADE-FOLD gate per rule: can it live INSIDE a fused direct-coded region, i.e. is it EFFECT-FREE (no runtime directive any phase, no mid-sequence inline, no follow restriction) + policy-encodable + free of `@profiles`/`@transform`/value-constraints? Deliberately NOT the scanner tier gate: cycles, layout skips, lookahead, and every `UnifiedReturnAST` value shape are all emittable by a fused *matcher* and are named facts, not blockers; regions root at protocol boundaries, references eligible→ineligible are per-rule `boundary_refs` call-outs; blocker histogram always, per-rule ROOT/internal/BLOCKED verdicts under `PGEN_FUSIBILITY_DUMP_ALL=1`) and, with outcome counts, `CASCADE-EXPOSURE: … internal=… (…%) committed=… discarded=… memo_hits=… | roots=… committed=… | residual=… committed=… | committed_floor=…` — the first-order fold model: internal entries' per-entry protocol is eliminated, root entries become one specialized-function call each, `committed_floor` = the post-fold protocol-paying committed entries (the honest pricing denominator); internal memo hits re-execute (the emission design owns the re-probe boundedness proof). Always also prints `CASCADE-PLAN: … increment=A(acyclic-subregions) sub_roots=S internal=I effect_reaching_fused=E` (RGX-0078.5.i.7 D2-A — the SHARED acyclic-sub-region emission plan `fusibility_census::compute_cascade_emission_plan`, the SAME map codegen consumes so report and emission cannot drift: CYCLIC eligible rules stay protocol boundaries in this increment, sub-roots keep the full protocol frame + the observability-twin dispatch, internal rules run as fused `cascade_*` functions on the bare-parse path, and `effect_reaching` marks rules from whose body an ineligible rule is reachable — the conservative C3-B snapshot/island obligation set; the plan additionally carries `effect_targets` (= ineligible rules ∪ effect_reaching — the per-SITE membership test the LANDED `-0087` emitter uses to pick plain-restore vs `try_parse` speculation and plain-vs-island Or sites); per-rule `[cascade-plan]` verdicts under `PGEN_FUSIBILITY_DUMP_ALL=1`). Always also prints `CASCADE-PLAN-B: … increment=B(cyclic-spine) sub_roots=S internal=I thin_memo=T effect_reaching_fused=E` (RGX-0078.5.i.7 D2-B plan seam — the SAME plan function at the `CyclicSpine` increment, the map the D2-B emitter will consume: EVERY cascade-eligible rule fused, sub-roots = the census's own full-fold roots, `thin_memo` = the cycle-participating fused rules, which carry the epoch-stamped thin memo — ⛔ the session-#49 bound, they never lose memo protection; the effect fixpoint is increment-independent; per-rule `[cascade-plan-b]` verdicts under `PGEN_FUSIBILITY_DUMP_ALL=1`). BOTH cascade emitters ARE LANDED — the emitter consumes the CASCADE-PLAN-B map (D2-B, the cyclic-spine increment): every generated parser carries its full fused graph (acyclic sub-regions + the cyclic spine), entered only on bare parses (see the routing notes at 3.3/3.4/3.5 — any diagnostic consumer routes to the protocol graph automatically); cycle-participating internal rules carry the protocol-mirror recursion guard + the taint-classed thin memo (⛔ the #49 bound — see `ThinMemoEntry`). Always also prints `BOUNDARY-SCANNER-PLAN: … rules=N (text=… span_transform=… shaped_object=…; sub_roots=… residual=…; post_predicates=…) dropped_candidates=M` (RGX-0078.5.i.9 D3 — the SHARED boundary-scanner gate `fusibility_census::compute_boundary_scanner_plan`, the SAME map the scan emitter consumes so report and emission cannot drift: a PLAN-B protocol boundary — sub-root or residual — qualifies iff its text surface is scanner-encodable, its directive residue is within {rule-level matched-text `@transform`, read-only post-phase `@predicate`}, it is non-entry, and its committed value is reproducible per class — `text` = the matched span, `span_transform` = the span-fallback `TransformedTerminal`, `shaped_object` = a static-key `-> {…}` template over plan-rule elements; per-rule `[scanner-plan]` verdicts AND every dropped candidate's NAMED reasons under `PGEN_FUSIBILITY_DUMP_ALL=1`). The D3 scan emitter IS LANDED: per plan rule the generated parser carries a direct-coded frameless `scan_<rule>` serving BARE-path call sites (fused `cascade_*` bodies call it in place of the boundary method; protocol bodies dispatch on `bare_parse`), with exact furthest-position emulation and the protocol twin verbatim — diagnostic dumps are unaffected by construction. The full 11-grammar census + method notes live in the `.5.h.1`/`.5.h.1b`/`.5.i.3`/`.5.i.4`/`.5.i.7`/`.5.i.9` sections of `docs/tasks/RGX-0078.md`.

### 5.4 `--dump-rule-profiles`
- **WHAT:** (SV-CORPUS-GRAD.7, parser-agnostic) machine-readable per-profile RULE INVENTORY — for every rule of the UNFILTERED grammar: its declared `@profiles` set (absent = universal) and its DERIVED per-profile satisfiability (`derive_rule_profiles` — the same transitive computation the profile-orphan lint gates on). Deterministic JSON (sorted rules + profile lists).
- **WHEN:** the external-corpus rule-coverage instrument's denominator ("which rules can a corpus run under profile P ever exercise?" — see `stimuli/sv/corpus_rule_coverage.py`); adjudicating whether an unfired/unwitnessed rule is a corpus gap vs N/A-for-profile; auditing a profile-gating edit's satisfiability effect (the `.3.1` interface-class class).
- **HOW:** `./rust/target/debug/ast_pipeline grammars/<g>.ebnf --dump-rule-profiles /tmp/rule_profiles.json`
- **OUTPUT:** `{"grammar", "profiles", "rule_count", "rules": {rule: {"declared_profiles": [...]|null, "satisfiable_under": [...]}}}` — for systemverilog: 1,466 rules; satisfiable_under sv_2017=1,343 / sv_2023=1,362 / verilog_2005=1,115 (the sv_2017/v2005 counts equal the cert-coverage canonical totals — the two instruments cross-confirm).
- **COMPANION — the LRM-structure lens (SV-CORPUS-GRAD.7b):** `stimuli/sv/corpus_clause_coverage.py` maps the *keyed* corpus (sv-tests `:tags:`, ispras clause-encoded filenames) onto the LRM's own chapter/clause structure — the complement to the rule lens. Reads only the committed adjudication manifests (main + v2005; **no parser run**), deterministic. Answers "which LRM chapters/clauses does the keyed corpus target, and where is the negative-axis gap?" — surfaces parse-bearing chapters with no keyed case and chapters with no keyed `must_reject` (the thinnest axis). NOT a competing coverage % (only ~14% of the universe is clause-keyed; `.7a` owns the authoritative parseable-surface denominator). Output: `stimuli/sv/characterization/clause_coverage.md` + per-clause `.tsv`.

---

## 6. Coverage / gap reports (`ast_pipeline`)

- `--report-k-path-coverage K` — k-path (Havrikov-Zeller) coverage report at depth K (use 2–3); read-only.
- `--gap-report-json FILE` / `--gap-report-text FILE` (+ `--gap-report-threshold`) — emit the coverage-gap / unreachable-rule-debt report consumed downstream.
- `--target-report-input FILE` — load a prior gap report and apply its targets as generation priorities (the witness/closed-loop replay surface).

### 6.1 `failure_reasons` — WHY a residual coverage target survived (already written, no re-run)

- **WHAT:** the per-branch failure record. `record_branch_failure`
  (`stimuli_generator.rs:460`, called from every OR-failure path — `:10461`, `:10557`, `:10577`)
  stores the error string for each `(rule::node_path, branch_index)`. It is published twice by any
  run that already emits reports: as `top_failure_reasons` in the gap report (rendered
  `failure_reasons=[reason (count), …]`, ⚠️ **truncated to the top 3**) and **untruncated** in the
  coverage artifact as `branch_groups["<rule>::<path>"].failure_reasons`.
- **WHEN:** ⛔ **the FIRST thing to read on any "why is this coverage target still residual"
  question** — before designing a new probe, and before trusting the pass summary. The witness
  pass's `depth_exceeded=…, target_timeout=…, helper_timeout=…` counters are **target-scoped**: they
  move only when a *whole target* errors. A forced branch that fails is rescued by
  `generate_or`'s sibling fallback, the rule returns `Ok`, and every counter stays **0**. Measured
  signature of that trap: *42 targets unresolved against exactly 1 recorded failure*
  (`SV-EXH-PROOF.7.4.6.9` — the whole 79-branch class was then explained from disk, in minutes).
- **HOW:**
  ```bash
  python3 - <<'EOF'
  import json
  report = json.load(open('rust/target/sv_stimuli_quality_gate/work/profile_2017_replay_gap.json'))
  for debt in report['reachable_branch_debt']:
      print(debt['branch_id'], debt['selected_hits'], debt['success_hits'])
      for reason in debt['top_failure_reasons']:
          print('   %6d  %s' % (reason['count'], reason['reason']))
  EOF
  ```
- **READING:** sum each row's reason counts and compare against `selected_hits` — equal means the
  top-3 cut hides nothing. Each reason names the PASS that produced it via its own budget:
  `budget=<N>ms` on a helper probe is the target-drive helper timeout, and the witness pass runs at
  **≥ 2× the configured `--max-depth`** — since `SV-EXH-PROOF.7.4.6.9` its budget is PER TARGET,
  `2 × --max-depth` (the reach prefix) **plus that target's own minimal derivation depth** (the
  targeted ALTERNATIVE's, for a branch target). So under `--max-depth 20` a witness-pass attempt
  records `max_depth=40` only when the target's addend is 0, and `max_depth=40+k` otherwise; what
  identifies the pass is the value being at or above `2 × --max-depth` and OFF the `+4` slack ladder
  (`20, 24, 28, …`, which belongs to the target-drive/diverse retry). See the KM cards
  `branch-failure-reasons-are-the-witness-why` and `coverage-gap-reason-codes-are-generator-verdicts`.

### 6.2 `Depth-slack retry census` — does the generator's depth ESCALATION ever pay?

- **WHAT:** one stdout line from any `--target-report-input` (closed-loop replay) run, pricing
  `generate_or`'s depth-slack retry (`target_branch_depth_retry_slack`). Per NESTING LEVEL:
  `successes/attempts@max_budget`. Per BRANCH: `branch_retry_max` (the most retries any single
  targeted branch spent) and the `success_ordinal:count` histogram — *at which of a branch's
  retries a success actually landed*. Read-only; it never changes a generation decision.
- **WHEN:** ⛔ the FIRST thing to read before bounding, tuning or trusting any generation retry —
  and the answer to *"`--max-depth` is 20, so why does `failure_reasons` say `max_depth=448`?"*.
  The retry adds its slack to the **live** `config.max_depth`, so nesting makes it CUMULATIVE and
  `--max-depth` bounds only the outermost descent. 6.1's `failure_reasons` shows that ladder's
  FAILURES; only this census shows whether the escalated rungs BUY anything.
- **HOW:** it is already in every closed-loop replay log — no flag, no re-run:
  ```bash
  grep "Depth-slack retry census:" rust/target/sv_stimuli_quality_gate/logs/profile_2017_closed_loop_replay.log
  ```
- **OUTPUT (measured, `systemverilog` at `--max-depth 20`):**
  ```
  Depth-slack retry census: nesting_levels=100 attempts=318117 successes=390
    deepest_paying_level=89 branches_retried=578 branch_retry_max=4096
    success_ordinal_max=3886 [successes/attempts@max_budget] L1:33/602@24 L2:42/488@28 …
    [success_ordinal:count] 1:145 2:22 3:7 …
    | explicit-grant: max_explicit_budget=463 vs max_granted_budget=444 covers_successes=233/252
      at_least_as_generous=302507/302526 success_shortfall_max=1
    | declared-ceiling: ceiling_budget=420 refusals=2429
  ```
- **READING:** `attempts` ≫ `successes` is normal; the number that matters is where the successes
  STOP. `sv_2023` measured `deepest_paying_level=24` against 164 levels climbed — **99.5 % of its
  retry work returned zero successes**. `branch_retry_max` ≫ `success_ordinal_max` is the runaway
  signature (`726 836` vs `103 829` on `sv_2017`; one branch took 37 % of the whole run's retries).
- ⭐ **THE `explicit-grant:` ARM — what a DERIVATION-JUSTIFIED budget would have given at every rung**
  (`SV-EXH-PROOF.7.4.6.13`). The line's tail reads
  `explicit-grant: max_explicit_budget=463 vs max_granted_budget=444 covers_successes=233/252
  at_least_as_generous=302507/302526 success_shortfall_max=1`, where the explicit budget is
  `depth + min_full_derivation_depth(TARGETED ALTERNATIVE)` — the same quantity the closed-loop
  witness pass grants a witness target, evaluated at the rung the retry is standing on.
  ⛔ **Read `max_explicit_budget` against `max_granted_budget` FIRST, and do not assume the explicit
  grant is the tighter one — measured on SV it is the LOOSER one** (`463 > 444` on `sv_2017`,
  `695 > 672` on `sv_2023`), because the grant is `depth + need` and `depth` is the LIVE descent
  position, so it inherits the very ladder it looks like it would bound. `covers_successes` is the
  only sufficiency claim the instrument makes: a success with `explicit >= granted` would certainly
  still have been bought; a shorter one is recorded under `success_shortfall_max` as **UNKNOWN**
  (the generator need not take a minimal derivation), never as a loss.
- ⭐ **THE `declared-ceiling:` ARM — the bound the artifact was produced under, and how often it
  fired** (`SV-EXH-PROOF.7.4.6.13`, the leaf's closing fix). The line ends
  `declared-ceiling: ceiling_budget=420 refusals=2429`. The escalated budget may never exceed
  `DEPTH_SLACK_RETRY_CEILING_MULTIPLE` (**21**) × the **configured** `--max-depth`, so a reader
  never has to infer the bound from the ladder's shape, and `ceiling_budget=disabled` (via
  `PGEN_DEPTH_SLACK_CEILING_MULTIPLE=0`) is distinguishable from a ceiling that is in force and
  simply never bound (`refusals=0`).
  ⛔ **This is the bound `--max-depth` itself was never going to be.** Making `--max-depth` literal
  was measured and REJECTED: it costs residual `0 → 17` / `0 → 10`, because the cumulative
  escalation is load-bearing. Landing the ceiling instead took the ladder `444 → 420` / `672 → 420`
  (both profiles to exactly `nesting_levels=100`) at **zero** coverage and **zero** residual cost.
  Price a multiple for your own grammar off any census line with
  `python3 docs/tasks/artifacts/sv_exh_proof/depth_slack_ceiling_pricing.py <stage.log>` — it
  verifies the `configured + 4L` rung identity at every level and REFUSES a non-arithmetic ladder
  rather than pricing one.
- ⭐ **GROUND TRUTH — it is silent when the retry never fires.** A grammar with no targeted,
  depth-blocked branch prints NOTHING (pinned by
  `depth_slack_retry_census_is_silent_when_the_retry_never_fires`), so a census line in a log is
  evidence the retry ran — not evidence it was compiled in. The positive control lives in
  `target_driven_generation_retries_target_branch_with_depth_slack`; the ceiling's own pair is
  `depth_slack_retry_ceiling_is_a_multiple_of_the_configured_depth_not_the_live_one` and
  `depth_slack_retry_ceiling_refuses_the_rung_that_would_climb_past_it`.
- ⚠️ **HONEST BOUND:** a cap or ceiling read off this histogram prices *the run you measured*.
  Capping changes generation downstream, so the after-run finds different successes —
  `SV-EXH-PROOF.7.4.6.13` measured `sv_2023` successes go **147 → 398** under a cap, and the
  declared ceiling's static pricing was wrong in **both** directions a third time (predicted
  `3 814`/`33 424` refusals and no gain; measured `2 429`/`1 116` and successes **up**
  `252 → 390` / `398 → 426`). Always A/B the residual, never infer it.
  Full map: `docs/tasks/SV-EXH-PROOF.md` `.7.4.6.13` + book *Stimuli and Quality*.

### 6.3 `Store-entry-blocked verdict census` — is a witness-entry RAISE actually NEEDED, or is the verdict over-approximating?

- **WHAT:** one stdout line per **store-entry-blocked** witness target, classifying it `SPURIOUS`
  (the verdict called the target structurally unwitnessable from its own rule, and its own rule
  witnessed it anyway) or `GENUINE` (it did not). Read-only, opt-in, and it perturbs nothing —
  measured byte-identical artifacts with it ON.
- **WHEN:** ⛔ before tightening, trusting or extending `witness_target_is_store_entry_blocked` — and
  the answer to *"`store_entry_raises` says N, but how many of those raises were NEEDED?"* The pass
  summary counts raises; only this names them and says which were unnecessary.
  ⭐ **The general lesson it encodes:** a policy's over-approximation is measurable only where the
  policy's decision AND its outcome are both in hand. A harness that re-evaluated the verdict
  statically would report the verdict and nothing else — "was the raise necessary" is knowable only
  after the own-rule attempt has run, which is why this lives inside the witness pass.
- **HOW:**
  ```bash
  PGEN_WITNESS_BLOCKED_VERDICT_CENSUS=1 bash docs/tasks/artifacts/sv_exh_proof/run_closed_loop_replay_stage.sh 2017 /path/to/out
  grep -c 'classification=SPURIOUS' /path/to/log ; grep -c 'classification=GENUINE' /path/to/log
  ```
- **OUTPUT:**
  ```
  [blocked-verdict-census] target='branch::net_declaration_sv_2017::root#1' rule='net_declaration_sv_2017'
    type=Branch node_path=Some("root") branch=Some(1) verdict=blocked own_rule_attempt=ok
    own_rule_resolved=true classification=SPURIOUS
  ```
- **READING:** `own_rule_attempt=ok` does **not** mean the target was covered — a forced branch whose
  gated content prunes lets a `generate_or` sibling rescue the rule, so the attempt returns `Ok` with
  the branch uncredited (`selected_but_failed`). The discriminator is `own_rule_resolved`, which is
  coverage-based; `attempt=ok` appears on GENUINE rows too.
- ⭐ **GROUND TRUTH — it reconciles against two independently-measured numbers before its list is
  trusted:** its `GENUINE` count must equal the pass summary's own `store_entry_raises`, and its
  TOTAL must equal the pre-`SV-EXH-PROOF.7.4.6.15` raise count. Measured: `sv_2017` **16 = 15 + 1**,
  `sv_2023` **11 = 10 + 1**, both exact, `store_entry_raises=1` on each. A census that fails to
  reconcile is reporting a broken instrument, not a finding.
- **Measured at the time of writing:** **every** blocked target on both profiles is a `type=Branch`
  target — zero rule targets — so the over-approximation lived entirely in the BRANCH arm of
  `target_forces_positive_store_gate`.
- ⭐ **AND THE CENSUS HAS SINCE ANSWERED ITS OWN QUESTION — read this before treating a `SPURIOUS`
  row as normal.** Classified by GATE CLASS the split was perfect: the genuine target is a
  `fact_count_at_least` COUNT gate, every spurious one a `has_fact`/`fact_attribute_equals` NAME
  gate. Root cause: **the generator has exactly one store prune and it is count-only**
  (`gen_count_predicate_satisfiable`), so a name gate can never make a target unwitnessable at
  generation time. The verdict is now scoped to that map (`StoreGateScope::GenerationPruned`), and
  the census reads **`sv_2017` 1 = 0 SPURIOUS + 1 GENUINE**, **`sv_2023` 1 = 0 + 1**, with the stage
  artifacts byte-identical. ⇒ **a `SPURIOUS` row today is a REGRESSION signal, not background
  noise.** The cross-language mirror `docs/tasks/artifacts/sv_exh_proof/class_c_store_entry_closure.py`
  re-derives the same verdict and REFUSES (exit 2) if it drifts back to scoping on polarity.
  Full map: `docs/tasks/SV-EXH-PROOF.md` `.7.4.6.17`.

---

## Protocol A — diagnose an `UNKNOWN` (4 steps)

1. **Full list + honest number** — `PGEN_CERT_COVERAGE_DUMP_ALL=1 … --report-certificate-coverage … --seed 0` (re-run seeds 7, 42 for determinism).
2. **WHY each one failed** — `PGEN_CERT_COVERAGE_DEBUG_PROBES=1 …` → read `parsed` / `witnessed_target` per `[plannable-probe]` line.
3. ⭐ **WHAT THE PLAN ASKED FOR** — for any `parsed=true witnessed_target=false`, `PGEN_REACH_PATH_DUMP=1 …` (4.4). ⛔ **Do not skip to a conclusion after step 2.** Step 2 shows where generation *ended up*; only step 3 shows what it was *instructed to do*, and the whole diagnosis is the gap between them. Skipping it is how `ENGINE-UNIVERSAL-SERVICES.10`'s first root cause was written down wrong — in a task leaf and in a tracked gate contract.
4. **Exact rejection** — for a `parsed=false` rule, feed its forced sample back through `--trace-rules <rule>` at `PGEN_TRACE_VERBOSITY=debug` → the `🚫 rejected by post predicate …` line.

Cause map: `NO reach path` = dead-rule candidate (adjudicate via 5.1) · `parsed=false` + predicate-reject = store-gate (the precondition fact was never generated → store-aware generation) · `parsed=true witnessed=false` splits in **three**, and step 3 is what tells them apart:
- **plan-side** — the hop dump is missing, short, or routes through the wrong carrier ⇒ a reach/routing gap.
- **render-side** — the hop chain is complete and correct but the sample ignores it ⇒ generation was steered and *failed*, then fell back. Look for a forced site that re-fires (4.4's `Quantifier decision` pairing); `generate_or`'s forced-first-**with-fallback** makes this silent by design.
- **parser-side** — the sample renders the target construct correctly and still does not witness ⇒ the PARSER never commits to the target, typically a longest-match sibling that spans the same syntax (`.10` mechanism 2: SV's `select_expression` catch-all reaches the full expression grammar, which parses `&&` itself, so a bare-identifier seed absorbs the operand). Confirm with `--parse-dump-ast-pretty` and read the discriminator `kind` — entry counts alone will show the rule *entered* and mislead you.

## Protocol B — a parser rejects valid input
1. `--parse` and read `furthest_position` (3.2) → map to a line.
2. Minimal-reproduce that construct; `--parse` it. Fails on the minimal = localized; passes = contextual (bisect what precedes).
3. If a `@predicate` is suspected, `--trace-rules <suspect>` at `high`/`debug` (2.4) → the verdict names the rejecting predicate and resolved args.
4. ⭐ **PAIR THE REPRODUCER WITH AN ACCEPTING CONTROL, then PIN BOTH** (`SV-CORPUS-GRAD.13c.2`). A
   reproducer alone proves a rejection; a reproducer plus the nearest construct that DOES parse
   proves *what the defect is*. `parameter logic [7:0] K = 8'(1);` rejecting means little until
   `initial k = 8'(1);` is shown to pass — then the defect is the CONSTANT-expression path, not the
   cast. Add both to `stimuli/sv/adjudication_repros/` + `MANIFEST.tsv`; `stimuli/sv/run_adjudication_repros.py`
   then re-runs them as a **two-sided ratchet**: `class=defect` fails when it starts parsing (flip it
   — the fix landed), `class=invalid` fails when it starts parsing (an **over-acceptance**
   regression), `class=control` fails when it stops parsing (the reproducer no longer isolates one
   difference). ⛔ A verdict of *"the parser is right to reject this"* MUST be pinned as
   `class=invalid` — otherwise the day a rule is relaxed for some other row, that text starts
   passing and the pass rate **improves**.
   ```bash
   python3 stimuli/sv/run_adjudication_repros.py --verbose
   # ADJUDICATION-REPROS: checked=16 listed=16 failures=0
   ```
5. ⭐ **BEFORE CALLING A CORPUS ROW A DEFECT, SUBTRACT THE CHEAP EXPLANATIONS — BY PARSING, not by
   argument** (`stimuli/sv/adjudicate_dark_worklist.py`). It declares the names the parser says it
   lacks, wraps the file in a compilation unit if it needs one, re-parses to a fixed point, then
   MINIMIZES the prelude — so a row is classified `CROSS-FILE-FACTS` / `FRAGMENT-<kind>` /
   `MACRO-BLOCKED` / `RESIDUAL` by what actually parses. ⛔ Each candidate declaration is admitted
   only if it does not REDUCE how far the parse reaches: declaring a speculated name that is really a
   PORT shadows the port and drives the parse **backwards**, which reports "unexplained" for files
   that are fully explained.

## Protocol C — a parse is slow / hangs
1. `--dump-rule-call-counts 30 --dump-rule-call-counts-exclude "trivia,…"` (3.1) → the dominating rules.
2. `--trace-rules <dominator>` (2.2) → the actual call pattern; `PGEN_REPORT_MEMO_STATS=1` (3.3) for memo behavior.

## Protocol D — which alternative WINS this choice? / is this branch LIVE? (the A2.2/A2.3-class probe)

For any "is this ordered-choice branch dead or live / which alt does the engine actually select?"
question — NEVER answer it from the grammar text (PGEN's `|` is a branch TOURNAMENT, default
`longest_match`, NOT PEG first-match commit; two linter verdicts were unsound for exactly this reason).

1. **Isolate the choice in the scratch slot (1.3)** — `scratch := <alt0> | <alt1> | …`, carrying the
   real rule's `@branch_policy` / `@priority` / `@associativity` annotations if any; then
   `make -C rust SHELL=/bin/bash focus_scratch` + rebuild the release probe (it embeds scratch at
   COMPILE time).
2. **Parse the discriminating input and READ THE WINNER** — codegen's own selection line:
   ```bash
   PGEN_TRACE_VERBOSITY=debug ./rust/target/release/parseability_probe \
     --parse scratch /tmp/in.txt --trace-rules scratch 2>&1 | grep 🏁
   # → 🏁 Rule 'scratch' selected branch N/M consuming K chars (priority=…, associativity=…, branch_policy=…)
   ```
   `N` names the winning alternative; `--parse-dump-ast-pretty` (1.2) confirms the winner's shape.
   (For a rule inside a REAL registered grammar, skip the slot and run step 2 directly with
   `--trace-rules <rule>` on the real parser.)
3. **Cross-check against the pinned policy matrix** (`parse_harness_combinator_gate`, 1.7): only
   `ordered` commits to the first success; `longest_match` (the default) tries every alternative and
   keeps the longest (ties → earlier, or later under `@associativity: right`); `priority_first` ranks
   by `@priority` with longest-match tie-break.

Cause map: a branch the engine SELECTS is **LIVE** — any deadness verdict must be conditioned on the
rule's actual selection semantics (precedents: A2.2 `EarlierAlwaysMatches` retired; A2.3
`FixedTerminalPrefix` policy-conditioned — `docs/decisions/project_fixed_terminal_prefix_policy_conditional.md`;
A2.4 `DuplicateAlternative` tie-break-conditioned on `@associativity`/`@priority`/`@deterministic_group` —
`docs/decisions/project_duplicate_alternative_selection_semantics_conditional.md`, the R/P/D/N probes
above are its worked example). With A2.2/A2.3/A2.4 every ordered-choice deadness verdict now names AND
checks its selection semantics.

---

*This catalog is part of the four-surface "impossible to ignore" debug-toolbox capture (book + KM +
decisions + project memory). Keep it in lockstep when a debug surface is added or changed.*
