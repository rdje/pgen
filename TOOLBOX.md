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
| 1 | **correctness** | the parser accepts/rejects the wrong thing | `CERTIFICATE-COVERAGE:`, `[plannable-probe]`, `rejected by post predicate`, `furthest_position=`, `--trace-rules`, `--lint-grammar`, `PGEN_LINT_DUMP_ALL`, `--dump-rule-call-counts`/`--dump-rule-outcome-counts`, `--parse-dump-ast`, `INTERPRET-PARSE:`/`--interpret-parse`, `INDIRECT-LR-SURVEY:`/`--report-indirect-lr-plan`, `PGEN_REACH_PATH_DUMP`, `PGEN_REACH_FORCED_OVERRIDE_DUMP`/`[forced-override]` |
| 2 | **performance / SPEED** | it is correct but slow | `/usr/bin/sample`, `otool` (annotated disassembly), `spindump`, `filtercalltree`, `ITIMER_PROF`, `self-time`, `call-graph attribution`, `cargo flamegraph` |
| 3 | **build integrity** | a target no longer COMPILES — no parse to trace, no run to sample | `error[EXXXX]`, `could not compile` |
| 4 | **codegen emission** | the GENERATOR emits the wrong code — it compiles and parses fine | `GENERATED-CLIPPY-CORRECTNESS:`, `clippy::<lint>`, `PGEN_CLIPPY_GENERATED_STRICT` |
| 5 | **ops / build-flow** | the defect is in the repo's OWN scripts, Makefiles, hooks or tracking state — shell/make, so no rustc error either | `git ls-files`/`log -S`/`rev-list`/`fsck`/`reflog`/`diff-tree`/`merge-base`, `shellcheck`, `bash -n`, `make -n`/`make --dry-run`, `E2BIG`/`ENOSPC`/`EACCES`/`ARG_MAX`, `guard.<pid>.marker`, `reason=rss-budget\|free-floor\|disk-floor\|timeout`, `GATE-REACHABILITY-PROBE:`, `SV-CONTRACT-CURRENCY:` (`scripts/check_sv_contract_currency.sh` — the SV grammar's semantic digest versus the contract's), `ACCEPT-SET-LEDGER:` (`docs/tasks/artifacts/sv_corpus_grad/contract_accept_set_ledger/measure_accept_set_transitions.py` — which grammar commit moved which verdict or which typed AST), `LRM-ANNEX-A-GAP:` (`stimuli/sv/lrm_annex_a_gap_census.py` — the IEEE 1800 productions the CLAUSE bodies define and Annex A, the only surface the extractor reads, does not) |

⛔ **A bare `file.rs:NNN` citation is NOT a signature, and neither is *"verified by grep"*.** Both
were measured and deliberately refused (`GENERATED-LINT-CORRECTNESS.4`): a line number is a
*location*, and "verified by grep" is a *claim* — the box asks for the output of a tool you ran.
Groups 2 and 5 exist because this repo's SPEED campaign and its ops defects were being forced to
waive an otherwise-correct gate; if you find a sixth such class, **open a leaf rather than waive**.

⭐⭐ **A NEW INSTRUMENT MUST BE REGISTERED IN THE SAME COMMIT THAT LANDS IT.** Group 1 is a
*vocabulary of tools*, so an instrument whose token is not in `DIAGNOSIS_SIG`
(`scripts/check_diagnosis_evidence.sh`) is invisible to the gate, and every leaf root-caused with it
is blocked — leaving only two ways out, citing a tool that did **not** produce the diagnosis, or
waiving. Both are dishonest. Measured: the gate correctly refused `ENGINE-UNIVERSAL-SERVICES.11`
slice 1 — *the leaf that built `PGEN_REACH_FORCED_OVERRIDE_DUMP`* — because the token it had just
created was not yet listed. This is not a sixth family and not a relaxation; it is the same
correction group 2 made for the SPEED profilers, a bar aimed at a stale tool list. ⛔ The obligation
is two-way: register a token **only** when a real, runnable instrument emits it, and update this
table in the same commit.

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
| **"Parse an input against an ARBITRARY / synthetic grammar (not registered)?"** | **[1.5b `--interpret-parse`](#15b---interpret-parse--the-interpreter-as-a-cli-one-command-no-codegen-no-compile) — START HERE, it is one command** · [1.3 the `scratch` slot](#13-the-scratch-slot--drive-the-toolbox-on-an-arbitrary-grammar) (full CLI toolbox incl. `--trace-rules`; authoritative by construction) · [1.4 compile-and-run](#14-the-compile-and-run-harness--parse-an-arbitrary-grammar-with-no-registry-edit--no-pgen-rebuild) (Rust API, authoritative by construction) · [1.5 the interpreter](#15-the-grammar-ast-interpreter--parse-an-arbitrary-grammar-in-process-with-no-codegen--no-compile) (Rust API) |
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
| **"The plan FORCES branch N — so why does the probe show branch 0? was the branch ever driven at all?"** — ⛔ `generate_or` falls back SILENTLY, and on the cert-coverage path 6.1 cannot answer (`--coverage-output` is refused there) | [6.4 `PGEN_REACH_FORCED_OVERRIDE_DUMP`](#64-pgen_reach_forced_override_dump--the-reach-plan-forced-a-branch-did-it-actually-render) — pairs the lost directive with the generator's own reason string |
| "Is my grammar well-formed (LR / shadowing / non-terminating)?" | [5.1 `--lint-grammar`](#51---lint-grammar) |
| **"`--lint-grammar` says `left_recursion_unhandled=N` — so WHICH rule could absorb the chain, what suffix, and what does it cost?"** | [5.5 `--report-indirect-lr-plan`](#55---report-indirect-lr-plan) — ⛔ the lint names the cycle, not the fix; picking the wrong base rule is a measured REGRESSION |
| "What IR do the generators actually consume?" | [5.2 `--dump-gen-ast`](#52---dump-gen-ast) |
| "Packrat memo hit/miss perf?" | [3.3 `PGEN_REPORT_MEMO_STATS`](#33-pgen_report_memo_stats) |
| "EXACT per-rule entry counts for a parse (machine-readable)?" | [3.4 `--dump-rule-entry-counts-json`](#34---dump-rule-entry-counts-json) |
| "How much parse work is DISCARDED (failed speculation)? committed vs wasted per rule?" | [3.5 `--dump-rule-outcome-counts-json`](#35---dump-rule-outcome-counts-json) |
| **"Did the whole SV parser get SLOWER — and would anything have told me?"** — ⛔ never answer with an ad-hoc timing script against a remembered number; that produced `~11 %`, then `+24.3 %`, and **BOTH were machine-variance artifacts** (`.26`) | [3.7 the SV parse-cost ratchet](#37-the-sv-parse-cost-ratchet--did-the-whole-parser-get-slower-and-would-anything-have-told-me) |
| **"Two spellings of the same fix — which is correct, and which is cheaper?"** — ⛔ never answer by reasoning from the CURRENT per-rule counters: the fix moves references between rules, which is what the generator's INLINING keys on, so the profile you priced from describes a parser that will not exist. Build the arms | [3.7b parser arms](#37b-parser-arms--two-spellings-of-one-fix-which-is-correct-and-which-is-cheaper) |
| **"WHERE does the parse time actually go?"** — ⛔ the counters cannot answer this: they all route to the PROTOCOL graph, and a production parse runs the FUSED one. Three traps, all silent | [3.8 sampling a parse with `/usr/bin/sample`](#38-sampling-a-parse-with-usrbinsample--the-only-view-of-the-fused-graph-and-its-two-traps) |
| **"Is the memo actually SERVING this rule?" — ⛔ `rule_memo_hit_counts` FUSES success replays with cached failures; 208 "hits" were 208 failures and 0 replays** | [3.6 memo insert/evict/replay census](#36-per-rule-memo-insert--evict--replay-census--is-the-memo-actually-serving-this-rule) |
| **"These two generated parsers differ by N bytes — what moved?"** — ⭐ since `ENGINE-UNIVERSAL-SERVICES.31`(e) the artifact embeds its own output path **ZERO** times (it was **36 346**), so the difference you read IS the source; the normalisation step is now a no-op kept as a tripwire | [5.6 normalise the embedded `-o` path first](#56-comparing-two-generated-parsers--normalise-the-embedded--o-path-first) |
| "Which rules could a derived DFA scanner fuse? the measured ceiling? the choice-site / merged-choice surface?" | [5.3 `--report-fusibility-census`](#53---report-fusibility-census) |
| **"WHICH grammar commit changed what a CONSUMER sees — and is it a WIDEN, a NARROW or a replaced AST SHAPE?"** — ⛔ a verdict-only answer is blind to the largest class: 12 pinned witnesses once moved with ZERO verdict movement | [5.7 the accept-set / AST-shape transition ledger](#57-which-grammar-commit-changed-what-a-consumer-sees--the-accept-set--ast-shape-transition-ledger) |
| "Which rules exist under which `@profiles`? Which rules can a corpus run under profile P ever exercise?" | [5.4 `--dump-rule-profiles`](#54---dump-rule-profiles) |
| **"Does the LRM define a production the SHIPPED grammar has never seen?"** — ⛔ Annex A is the only surface the SV extractor reads, and IEEE captions 60 of its own 2017 syntax boxes `(not in Annex A)` | [5.8 the Annex A gap census](#58-does-the-lrm-define-a-production-the-shipped-grammar-has-never-seen--the-annex-a-gap-census) |
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
  # 1. Edit the grammar BODY of grammars/scratch/scratch.ebnf, BELOW its header block
  #    (keep the entry rule named `scratch`), e.g.:
  #        scratch := "a" | "a" "b"
  make -C rust SHELL=/bin/bash focus_scratch                       # regen the scratch parser artifact
  (cd rust && cargo build --release --features generated_parsers --bin parseability_probe)
  printf 'ab' > /tmp/in.txt
  ./rust/target/release/parseability_probe --parse scratch /tmp/in.txt
  ./rust/target/release/parseability_probe --parse-dump-ast-pretty scratch /tmp/in.txt /tmp/out.json
  PGEN_TRACE_VERBOSITY=debug ./rust/target/release/parseability_probe --parse scratch /tmp/in.txt --trace-rules scratch
  ./rust/target/debug/ast_pipeline grammars/scratch/scratch.ebnf --lint-grammar
  ```
- **OUTPUT:** exactly as for any registered grammar (rc 0 + `parse_full passed`, the typed AST, branch-entry trace, `furthest_position` on reject).
- ⭐⭐ **PRESERVE THE PROBE, THEN RESTORE — in that order** (`GENERATED-LINT-CORRECTNESS.13`). The restore is TOTAL: the slot returns to its committed fixture and your synthetic leaves **no trace in git, by construction**. If the probe backs a claim in a task leaf, snapshot it first — one command, which also `git add`s it and prints the path to cite:
  ```bash
  scripts/preserve_scratch_probe.sh <TREE-ID> <probe_name>   # e.g. ENGINE-UNIVERSAL-SERVICES seed_shadowing
  git checkout grammars/scratch/scratch.ebnf                 # NOW restore the default fixture
  ```
  ⛔ **This is not tidiness.** `PGEN-ENGINE-UNIVERSAL-SERVICES-0005` proved a defect on an 11-rule synthetic and restored the slot; the grammar survived only as a sentence, and session #218 spent **longer rebuilding it from that sentence than the next mechanism's fix took** — on a leaf that had said *"prove it on the synthetic first"* twice. The instruction, not the intent, was what was missing. The script refuses rather than guesses (4 self-test controls, `--self-test`): an already-restored slot, an empty slot, an unsafe name, and an existing artifact with different content each stop with an actionable message.
  ⛔ **There is deliberately NO GATE for this, and the refusal is priced, not preferential.** A gate would have to detect *"an author built and destroyed a probe"*, and that act leaves nothing to key on — a correct workflow restores the slot, so the commit shows no diff there. The only remaining signal is PROSE, measured over the whole corpus: a trigger on scratch-slot phrasing inside ticked acceptance boxes fires on **24** boxes of which **22** would fail, but most are legitimate non-obligations (`PARSE-HARNESS.2` is the leaf that *created* the slot; `CI-PARITY-GATE-ROT.24` names `focus_scratch` only to demonstrate a build-flow trap and never had a probe grammar). 91 % false positives is a gate that teaches waivers — the failure `GENERATED-LINT-CORRECTNESS.6`/`.12` document, and the third chartered addition to that checker refused on measurement after `.4` (2/304) and `.7` (0/307).
- ⛔⛔ **OVERWRITE THE BODY, KEEP THE HEADER — and this half IS gated** (`PARSE-HARNESS.11`, doctrine `SCRATCH-SLOT-HEADER`). Do not confuse it with the un-gateable act above: *preserving a probe* leaves no artifact to key on, but *destroying the header* leaves the destroyed file on disk for as long as the probe is loaded, so it is catchable **at that moment and only then**. The slot's 32-line header is its operating manual (regeneration, the ordering trap below, the git-ignored artifacts, the restore) and whole-file replacement eats it — measured 2026-08-15, with all 19 doctrines green and the scratch integration test passing, because that test asserts the BODY parses. ⇒ `make focus_scratch` now refuses on a header that no longer matches the committed one, and the refusal is a one-command repair that keeps your probe:
  ```bash
  bash scripts/check_scratch_slot_header.sh --restore-header   # re-attach the committed header
  bash scripts/check_scratch_slot_header.sh --self-test        # 8/8 — every refusal proven to fire
  ```
  Rewording the manual is free at commit time (the doctrine tier checks only that it still names its four operational anchors); while probing, declare it with `PGEN_SCRATCH_HEADER_EDIT=1`. ⚠️ Honest limit: the probe-time tier rides `make focus_scratch` and `preserve_scratch_probe.sh`, so invoking `ast_pipeline` on the slot by hand gets only the commit-time tier.
- ⛔⛔ **THE BUILD TOOL ITSELF CAN HAND YOU THE PREVIOUS GRAMMAR — and this is the trap that fires hardest on an AGENT LOOP** (`CI-PARITY-GATE-ROT.32`). `/usr/bin/make` here is **GNU Make 3.81**, which compares mtimes at **WHOLE-SECOND** granularity (sub-second landed in make 4.x), so a grammar rewritten in the same wall-clock second as the previous build's output is INVISIBLE and the rule is skipped at exit 0. A human edits, thinks, types `make` — over a second, every time, so it cannot be seen by hand; a scripted probe loop hits it constantly, **silently and in the PASSING direction**. It was found by a control: an adversarial batch of four scratch grammars reported `expr_lr_base` for a grammar containing no rule `expr`. Deterministic replay with the slot pinned 0.8 s newer than `generated/scratch.json` inside one second: frontend ran **0** times, the parser still declared `alpha` while the slot said `bravo`. ⇒ `focus_scratch` now `rm`s both scratch artifacts and re-enters `make`, so the probe path does not depend on a timestamp comparison at all (cost: always ~2-4 s instead of ~0 s). Every other family carries the exact-window guard `scripts/make_freshness_guard.sh`, which forces a rebuild ONLY where make's comparison is provably wrong (same whole second AND the prerequisite genuinely newer), so `focus_systemverilog` keeps its no-op. ⚠️ **Do not reason "my family is slow, so I am safe"** — the gap a driver must beat is the work AFTER the target is written, not the total build: on the `parser ← json` edge that gap is the 0.006-0.111 s frontend step and **all 10 families are exposed, SV included** (census: `docs/tasks/artifacts/ci_parity_gate_rot/make_freshness_window_census.txt`). ⭐ If you drive ANY generation from a script, assert the work happened (`grep -c 'Generating' log` = 1) rather than trusting `make`'s exit 0 — `-0039`'s probes were audited exactly that way.
- ⛔ **`make focus_scratch` builds `ast_pipeline` BEFORE it regenerates the parser**, so a binary built by that run judges the PREVIOUS grammar. Rebuild it afterwards (`cargo build --features "generated_parsers ebnf_dual_run" --bin ast_pipeline`) before any cert-coverage run on `scratch`. Session #218 hit this: a correct probe grammar reported `UNKNOWN=9, witness=0, every probe parsed=false` — which reads as a broken grammar and was a stale binary. Distinct from the `#140` family in 1.4: the binary has both features, it merely predates the artifact. The target now prints both reminders. Full design + trust architecture: book chapter *The Parse Harness* + `docs/tasks/PARSE-HARNESS.md`.

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
- **OUTPUT:** a `ParseOutcome` whose `ast_json` is **byte-identical** to `parser_registry::parse_sample_ast_json` for a registered grammar (pinned by the integration test `parse_harness::tests::compile_and_run_harness_reproduces_json_registry_verdict_and_ast`). Needs an `ast_pipeline` binary built with `--features ebnf_dual_run` (the standard `target/debug/ast_pipeline`). **Feature-surface tripwire (PARSE-HARNESS.10):** before any codegen the harness probes the binary with `ast_pipeline --report-feature-surface` (a feature-independent pre-clap flag printing `AST-PIPELINE-FEATURE-SURFACE: ebnf_dual_run=<b> generated_parsers=<b>`) and REFUSES a stale single-feature binary up front with the exact dual-feature rebuild command — the #140-class trap (a build at the same path silently overwriting the dual-feature binary) now fails actionably instead of as a generic mid-gate codegen error. ⭐ **The `make focus_*` half of that trap is CLOSED since `CI-PARITY-GATE-ROT.24` slice 2** — the canonical `$(RUST_AST_PIPELINE)` rule now declares `--features "generated_parsers ebnf_dual_run"`, so the 21 targets depending on it can no longer rebuild an under-featured binary (generation-time cost only: codegen has zero `ebnf_dual_run` `cfg` sites and the regenerated parsers are byte-identical). ⛔ **The bootstrap half is NOT closed**: `regex_parser_bootstrap` still writes this path with `ebnf_dual_run` ALONE, dropping `generated_parsers` — tracked as `.24` slice 3, which is why the guard below still earns its place. ⭐ **The same trap fires against ANY ad-hoc measurement, and there the pre-check is yours to call:** `scripts/require_ast_pipeline_features.sh rust/target/debug/ast_pipeline generated_parsers ebnf_dual_run` refuses an under-featured binary with the exact rebuild command, and `--self-test` proves it fires (4 controls: complete / under-featured / missing / cannot-describe-itself). Run it before ANY sweep that scrapes a metric line — an under-featured binary errors on **stderr** and yields an EMPTY metric row on stdout, and two empty result sets **diff clean** (`CI-PARITY-GATE-ROT.24`: 18 empty rows per side nearly published as "zero drift"). First probe compiles `pgen` as a dep (~seconds→minutes cold); reuse `opts.workdir` to keep it warm. Full design: book chapter *The Parse Harness* + `docs/tasks/PARSE-HARNESS.md`.

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

### 1.5b `--interpret-parse` — the interpreter as a CLI, one command, no codegen, no compile
- **WHAT:** `ast_pipeline <grammar.ebnf> --interpret-parse <INPUT_FILE>` — 1.5's interpreter driven from a shell. Prints one greppable verdict line and exits **rc 0 on accept, rc 1 on reject** (the `--parse` convention, so a shell instrument can gate on it directly). Companions: `--interpret-entry-rule <RULE>` (alternate start symbol; the default is the grammar's declared `@entry: true` rule) and `--interpret-parse-ast-json <FILE>` (the typed AST on accept).
- **WHEN:** the "does this ARBITRARY grammar accept this input?" question, from a shell, on a synthetic you are still editing. This is the **fast rung**: no `focus_scratch`, no relink, no Rust test — so a five-rule synthetic is a one-second question instead of a three-minute one. Also the fast way to probe a MID-GRAMMAR rule (`--interpret-entry-rule`) when isolating which rule in a chain fails.
- **HOW:**
  ```bash
  printf "t'(n)" > rust/target/in.txt
  ./rust/target/debug/ast_pipeline docs/tasks/artifacts/engine_universal_services/indirect_lr/p1_knot_a_defect.ebnf \
      --interpret-parse rust/target/in.txt
  # isolate WHICH rule fails — same grammar, different start symbol:
  ./rust/target/debug/ast_pipeline <g>.ebnf --interpret-parse rust/target/in.txt --interpret-entry-rule cast_expr
  ```
- **OUTPUT:**
  ```text
  INTERPRET-PARSE: grammar='p1_knot_a_defect' entry='scratch' profile='<unspecified>' input_bytes=5 accepted=true furthest_position=5
  ```
  (on reject the same line carries `accepted=false` + `error=…`, printed on **stdout before** the rc-1 exit, so the measurement is greppable in both directions).
- ⛔⛔ **AUTHORITATIVE BY VERIFICATION, NOT BY CONSTRUCTION — AND THE VERIFICATION HAS A MEASURED HOLE.** This is the interpreter, whose credibility comes from 1.6/1.7 pinning it byte-identical to the real generated parser. On an **un-eliminated left-recursive cycle** that pinning does not hold: `ENGINE-UNIVERSAL-SERVICES.13` measured it disagreeing with the generated parser **in both directions** on a six-rule indirect-LR synthetic (`t'(n)` gen=accept/interp=reject; `n'(n)` gen=reject/interp=accept), because the generated parsers block on `check_cycle_id` while the interpreter has no cycle guard at all — only a whole-stack depth ceiling. Tracked as `ENGINE-UNIVERSAL-SERVICES.14`. ⇒ **If `--lint-grammar` reports `left_recursion_unhandled>0` for your grammar, cross-check every verdict against 1.3 or 1.4 before quoting it.** Elsewhere on the certified structural + return-annotation surface it is the fast, trustworthy rung; store-gated `@predicate` outcomes are outside it either way.
- ⭐ Runs on the UNFILTERED grammar with `--grammar-profile` applied as the interpreter's runtime profile — the same posture `--lint-grammar` uses, so a `@profiles`-gated rule is gated the way the generated parser gates it rather than stripped at load.

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
- ⛔⛔ **A BARE `--input … --output …` RUNS ARM 1 ONLY AND EXITS 0 — it does NOT compare the two
  frontends** (`LANG-CAPABILITY-AUDIT.10.6a`, measured). Arm 2 runs only when you pass
  `--envelope-differential` (what the gate and the recipes above use) or `--emit-ast-json`. On a
  grammar the hand-written frontend accepts and the generated meta-parser rejects, the three
  invocations give `rc=0 (silent)` / `rc=1` / `rc=1` respectively. ⇒ **never read a bare run's `rc=0`
  as "both frontends agree"** — it is one arm reporting success. This cost real time in session #221:
  a probe harness omitted the flag and printed three green rows where two were rejections. Same shape
  as the under-featured-`ast_pipeline` trap in 1.4 — an incomplete invocation yields a clean-looking
  result, in the passing direction, with nothing on stdout to notice.
- **OUTPUT:** a per-grammar report — `is_envelope_equivalent` (the frontend-REPLACEMENT verdict), `tokens_compared`, `token_matches`, `payload_not_comparable`, `payload_divergences`, `kind_divergences`, `tokens_unverified`, `divergence_total`, `unmapped_arm2_constructs`, `unresolved_include_directives`, and a located `divergences` list (rule name + token index + both arms' tokens, capped at 40 with the uncapped total alongside). Measured at the time of writing: **34 014 token positions across 14 grammars**, with **6 ENVELOPE-EQUIVALENT** — `builtin_return_annotation`, `builtin_semantic_annotation`, **`ebnf` itself (913/913)**, `rtl_const_expr`, `rtl_frontend`, `vhdl`.
- ⭐ **GROUND TRUTH — it refuses rather than guesses.** Every run first executes a **positive** control (a synthetic grammar that must project identically — 27/27 positions) and a **negative** control (a planted mutation the differ must catch exactly once, at exactly that index). A positive miss means the PROJECTION is broken; a negative miss means the DIFFER is blind. Either aborts before a number is published.
- ⭐⭐ **`PGEN_ENVELOPE_DUMP_ALL=1` LIFTS THE 40-ROW DIVERGENCE CAP — reach for it the moment you need
  to know WHICH divergence, not how many** (`SV-CORPUS-GRAD.13c.2i`). The report caps `divergences`
  at 40 and carries the uncapped `divergence_total` alongside, which the source calls *"truncation is
  always visible"* — ⛔ **and visible is not diagnosable.** Measured: with the gate RED on
  `systemverilog` at `151 > ceiling 150`, a set-diff of the ceiling-era report against today's came
  back **empty on both sides** — same first 40 rows, extra row past the cap — so the instrument could
  count the defect and never name it. With the cap lifted, one command names it
  (`use_clause_param_override_sv_only`, `arm1=semantic_annotation_inline` vs
  `arm2=semantic_annotation`). Same failure the lint's per-class cap had before `PGEN_LINT_DUMP_ALL`
  (5.1), which is why the variable is named after it.
  ```bash
  PGEN_ENVELOPE_DUMP_ALL=1 ./target/debug/ebnf_dual_run_diff --input ../grammars/<g>.ebnf \
      --output /tmp/r.json --envelope-differential /tmp/envelope.json
  ```
  ⭐ It is a REPORTING knob and that is asserted, not assumed: `divergence_total`, `tokens_compared`,
  `token_matches` and `kind_divergences` are byte-identical with it set and unset (bank
  `docs/tasks/artifacts/sv_corpus_grad/es13c2i_envelope_cap/probe.sh`, **11/11**, four identity arms).
  `0` and empty mean OFF, so it cannot be enabled by accident.
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
- ✅ **AND THE TWIN IS NOW MEASURED TO AGREE ON THE DERIVATION, NOT JUST THE VERDICT
  (`ENGINE-UNIVERSAL-SERVICES.22` (b), 2026-08-16).** *A verdict agreement is not a derivation
  agreement*, and this is the assumption every counter-based tool in 3.1-3.6 rests on. Measured
  with `parseability_probe --parse-dump-ast --dump-ast-with-coverage` over three arms of ONE
  binary — bare (FUSED), `PGEN_REPORT_MEMO_STATS=1` (PROTOCOL, coverage off), and coverage ON — the
  ASTs are **byte-identical**, one sha256 across all three, on the pathological file and on a
  strided sample (7 identical, 0 differing, tiers `hot=3 lr=2 breadth=4`). Probe:
  `docs/tasks/artifacts/engine_universal_services/derivation_twin/probe.sh`.
- ⛔⛔ **TWO OBVIOUS WAYS TO BUILD THAT A/B PRODUCE A FALSE PASS, AND BOTH WERE MEASURED.**
  (1) `--dump-rule-outcome-counts-json` beside `--parse-dump-ast` is parsed and then dropped: the
  `enable_coverage()` call lives in the `--parse` detail macro, so **no counts file is written and
  both arms come out bare**. (2) `PGEN_REPORT_MEMO_STATS=1` routes the parse but is **not** a
  coverage tell — its aggregate header is byte-identical with and without coverage, and the whole
  visible diff is which members of a tie group the top-30 cutoff prints. ⇒ the flag's own tell is
  the recorder's read-back (`COVERAGE-DUMP-AST: enable_coverage=true exercised_rules=N`), which is
  zero by construction when coverage is off. **If you build an observability A/B, make each arm
  print something only that arm can produce.**
- ✅ **FIXED 2026-08-16 (`ENGINE-UNIVERSAL-SERVICES.22` (e)) — READ THIS BEFORE THE PARAGRAPH
  BELOW.** The memo now stores an INDEX into an append-only side table and a hit pushes ONE tagged
  marker; read-back is a linear multiplicity fold. The file described below dumps in **0.04 s**, the
  full-corpus census is **16 336/16 336 with 0 no-dump**, and the reported numbers are byte-identical
  (the pinned 192-file `entries.tsv` did not move by one byte, and certificate coverage at seed 0 is
  character-identical). ⇒ if a 3.5 dump hangs for you TODAY, it is a NEW defect, not this one.
  The historical account is kept because it is the fastest way to recognise the shape:
- ⛔⛔ **(HISTORICAL) THIS DUMP COULD FAIL TO TERMINATE ON AN INPUT THAT PARSES IN 0.1 s, AND IT WAS
  THE COVERAGE STACK — measured, not suspected (`ENGINE-UNIVERSAL-SERVICES.22`).** `stimuli/sv/subs/Surelog/
  tests/ExponTimeIfElseGen/dut.sv` (**2 787 bytes**) parses BARE in 0.108 s and dumps under **3.4**
  in 0.056 s with 200 975 entries — and under 3.5 it peaks at **13.7 GB RSS inside ONE SECOND** on a
  24 GB machine and never writes a dump. 3.4 and 3.5 take the SAME graph, so the delta is the
  transactional coverage stack alone. WHY: `memoized_call` stores `coverage_stack[checkpoint..]
  .to_vec()` in every MemoEntry and replays it with `extend_from_slice` on every hit, which
  materialises the shared parse DAG as a **TREE** — the memo makes the parse linear by SHARING, and
  the coverage recorder undoes exactly that sharing. Measured growth law on an `else if` ladder
  (`docs/tasks/artifacts/engine_universal_services/coverage_stack_blowup/probe.sh`): rule entries
  **LINEAR** (+19 295 per arm, constant) while the coverage stack is **EXPONENTIAL (×4.14 per
  arm)**, reaching **353 005 042** slots at 7 arms; the corpus file carries 10, extrapolating to
  ~93 GB. ⇒ ⛔ **do not answer a 3.5 hang with a bigger timeout** — no timeout is large enough. If
  3.5 hangs, take the measurement with **3.4** (which has no coverage stack) and say which columns
  you therefore do not have. ⭐ The general lesson outlived the defect:
  [[an-observer-that-replays-a-memoized-result-turns-the-dag-back-into-a-tree]].
- ⛔⛔ **AND THAT BREAKS `raw − committed` AS A GENERAL IDENTITY — THIS HALF IS *NOT* FIXED, AND THE
  FIX IS WHAT PROVES IT.** Making the true multiplicity computable is exactly what let it be
  measured; the sign caveat below therefore stands and is now checkable rather than latent.** The docstring above reads
  *"`raw − committed` = the rule's FAILED-speculation entries"*, and it holds only while memo
  REPLAY is negligible. It is not an invariant: `committed` is `coverage_stack.len()` folded per
  rule, so a memo hit re-appends a whole cached subtree that the parser never re-entered, while
  `raw` counts real invocations only. Measured on the ladder above, `committed / entries` runs
  **0.62× → 2 173.88×** across 8 rungs — i.e. `raw − committed` goes NEGATIVE. It is positive on
  all 192 rows of the pinned parse-cost sample, and that is an empirical fact about those files,
  ⛔ **not a guarantee**. ⚠️ Until `.22`(e) the sample was additionally selected from a census that
  DROPPED exactly the file where it fails; the census is now complete (16 336/16 336), so that
  particular blind spot is closed — the identity is still not one. ⛔⛔ **AND OVER THE WHOLE CORPUS
  THE SIGN IS NOW NEGATIVE, MEASURED**: `entries 899 264 997` vs `committed 5 682 584 657` ⇒
  `entries − committed = −4 783 319 660`, i.e. `committed/entries = 6.32×`. Before the fix the same
  subtraction was POSITIVE (+864 629 799) only because the dominant file was being dropped. ⇒ this
  is no longer a synthetic-ladder hazard; a corpus-wide failed-speculation figure computed this way
  is now flatly wrong. Before quoting a failed-speculation percentage, check the sign.

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

### 3.7 The SV parse-cost RATCHET — "did the whole parser get slower, and would anything have told me?"
- **WHAT:** `stimuli/sv/corpus_parse_cost.py` (the instrument) + `scripts/check_parse_cost_ratchet.sh`
  (the standing gate, doctrine `PARSE-COST-RATCHET`). Measures the SV parser's cost over a **pinned
  192-file corpus sample** — 40 heaviest by entries, 40 heaviest by guarded-admission entries, 112 a
  stratified stride guaranteeing all 14 sub-corpora — and refuses if it RISES.
  `ENGINE-UNIVERSAL-SERVICES.20` acceptance (d).
  ⚠️ **`breadth` is 120 REQUESTED / 112 REALIZED**: the request is spent as an equal per-sub-corpus
  quota, so the file holds `14 × (120 // 14)` rows. `40 + 40 + 120` is not the sample's length.
  ⛔ **The `lr` tier was ranked by the PRE-`.21` classifier**, and `.21`(b) re-derived it under the
  corrected one and **DECLINED to adopt**, on a measurement: `hot` **0/40**, `lr` **5/40**,
  `breadth` **19/112** change, which would move the ratchet's binding baseline **+0.40 %** to buy
  **+0.45 %** family coverage. Declared in the manifest's own header; re-adjudicate with
  `docs/tasks/artifacts/engine_universal_services/sample_rederivation/compare_sample.py` (4 legs,
  incl. an external oracle holding the fresh census to the tracked `entries.tsv`).
- **WHEN:** ⛔ before and after ANY change that can touch the parse hot path — a grammar edit, a
  codegen change, a left-recursion admission policy, a memo change. Also the first thing to read when
  asking *"is the parser slower than it was?"*
- **WHY IT EXISTS:** `.17` slice 9's guarded LR admission shipped and every gate in the repository
  stayed GREEN, because nothing measured parse cost at all. Its deterministic cost, measured
  afterwards: **+10.59 % rule entries**. ⛔ Its *wall-clock* cost was published as **+24.3 %** and is
  **REFUTED** — see the blind-spot block below and `.20` slice 5.
- **HOW:**
  ```bash
  bash scripts/check_parse_cost_ratchet.sh                    # tier 1, all five arms, ~2 s
  make -C rust SHELL=/bin/bash sv_parse_cost_ratchet          # the full ratchet, ~2.5 min
  make -C rust SHELL=/bin/bash sv_parse_cost_rebaseline       # promote a new baseline (deliberate)
  make -C rust SHELL=/bin/bash sv_parse_cost_family_share     # re-derive the family share, ~70 s
  python3 stimuli/sv/corpus_parse_cost.py --verify-probe-fingerprint   # which parser is IN the probe?
  ```
- **OUTPUT:** `docs/tasks/artifacts/engine_universal_services/parse_cost_ratchet/` — `cost.md` (the
  byte-compared report), `entries.tsv` (per-file), `advisory.json` (wall clock), `family_share.json`
  (the corpus-wide LR-family share + the four identity rows it is a function of).
- ⭐⭐ **TIER 1 IS FIVE ARMS, NOT ONE.** (1) baseline identity; (2) `--verify-families` — every
  generated parser's declared `_lr` names are classified (0.1 s); (3) `--verify-family-share` — the
  published corpus share is re-hashed against its tracked derivation (~1 s); (4) co-publication —
  every designated live surface carries that derived pair; (5) `--verify-probe-fingerprint` — the
  PROBE is asked which generated parser it was compiled against (0.02 s). Arms 2-4 were on demand or
  ungated before `.21` (f), which is why the bound below could go stale in four documents at once.
  Their refusals are proven by
  `docs/tasks/artifacts/engine_universal_services/family_share_gate/probe.sh` (**11/11**, incl. a
  RED arm replaying the pre-`.21` classifier, and two added by `.26` for the leg that holds the
  artifact's RAW COUNTS to the share it declares).
- ⭐⭐ **ARM 5 — THE FOUR IDENTITY ROWS ARE ALL SOURCES; ARM 5 IS THE EXECUTABLE (`.24`).** Grammar,
  generated parser, instrument and sampled inputs are inputs; the numbers are produced by
  `rust/target/release/parseability_probe`, an **untracked build artifact** nothing hashed and
  nothing tied to the parser it was compiled from. `.20` slice 4 built one from a guard-suppressed
  experimental arm and the gate printed *"the measurement cannot have moved"* while
  `nm … | grep -c _lr_guard` read **0** against a pinned parser declaring **6**.
  ⛔ **It fails in the PASSING direction, which is why it is a refusal and not a footnote:** on four
  sampled files that wrong binary reports **762,345** rule entries where the shipped one reports
  **11,240,430** (**14.7×**), and this ratchet breaches on a RISE — a FALL is a note reading
  *"an improvement — promote it deliberately"*. So the gap invited a rebaseline that would have
  lowered the ratchet permanently to a number no real parser produces.
  **How:** `rust/build.rs` already resolves each generated parser and already `rerun-if-changed`s
  it, so it re-runs exactly when one moves; it now sha256s each and publishes
  `PGEN_<FAMILY>_PARSER_SHA256`, which `parseability_probe --parser-fingerprint` reports as JSON.
  **ZERO generated bytes** — emitting the fingerprint into the parser instead would move every
  artifact and re-baseline everything keyed on them.
  ⚠️ **Tier 1 NOTES, tier 2 REFUSES.** Tier 1 computes no measurement, so a stale probe misleads
  nothing it prints; failing every commit over an untracked build artifact is how a gate teaches
  people to bypass it. Tier 2 measures, so it refuses before the first file.
  ⚠️ **Measuring an experimental arm on purpose:** `PGEN_PARSE_COST_ALLOW_PROBE_MISMATCH=1`
  downgrades the instrument's refusal to a warning and **stamps the mismatch into `advisory.json`**,
  so the result cannot later pass for a baseline. The gate strips it from the environment — the path
  whose output becomes the tracked reference does not get an escape hatch.
- ⭐⭐ **WHAT BINDS, AND WHY IT IS NOT WALL CLOCK.** The binding numbers are exact integers from 3.5 —
  rule entries, COMMITTED entries, memo hits — each verified deterministic across repeated release
  runs AND byte-identical between the debug and release probes. Wall clock is **advisory only**, on a
  wide ±50 % band that never fails the gate: a wall-clock-primary ratchet inherits the very defect
  this leaf demonstrated **twice**. It first published `~11 %` (compared across *"materially faster
  machine conditions"*), corrected that to `+24.3 %` — and `.20` slice 5 then refuted the correction
  too, as a fixed-arm-order artifact. ⛔ Two wall-clock figures, three sessions apart, both wrong,
  neither caught by anything but a re-analysis of their own raw data.
- ⚠️ **THE PROCESS FLOOR IS SUBTRACTED AND RE-MEASURED EVERY RUN, NEVER ASSUMED.** A probe invocation
  costs ~9.8 ms before it parses anything (fork + exec + SV stdlib preload) and the median corpus file
  takes ~18 ms total — so a raw per-file wall-clock number is mostly a measurement of `fork`.
- ⛔ **THE DECLARED BLIND SPOT — A PROPERTY, NOT A NUMBER (`.26`, 2026-08-16).** The counters tick
  only in the PROTOCOL graph (3.4/3.5 ROUTING); the fused `cascade_*` twins — including
  `cascade_match_casting_type_lr_suffix` and friends — tick nothing. And a counter counts **events**:
  a rise in the cost PER event is invisible to it on any graph. That limit is a property of the
  metric, so unlike a measured bound it cannot go stale.
  **Live LR-family share `2.761`** (corpus-entry share %) — the LR-elimination family's share of all
  corpus rule entries. ⭐ **GATED, not quoted** since `.21` acceptance (f): `PARSE-COST-RATCHET`'s
  every-run tier re-hashes the four inputs it is a function of (grammar, generated parser,
  classifier, corpus) against
  `docs/tasks/artifacts/engine_universal_services/parse_cost_ratchet/family_share.json`, holds this
  paragraph equal to it, and re-checks that the artifact's own raw counts reproduce it. Re-derive
  with `--rederive-family-share` (~70 s); never by editing the digits here.

  ⛔⛔ **This anchor was the PAIR `2.741/8.9` until `.26`, and the retired half was wrong in BOTH
  its terms.** The factor was `24.3 / 2.741`. The **numerator** — a `+24.3 %` wall-clock regression
  — is REFUTED (`.20` slice 5, `PGEN-ENGINE-UNIVERSAL-SERVICES-0050`): not reproducible from the raw
  data of the runs that produced it, a fixed-arm-order artifact. The **denominator** was the wrong
  quantity independently of that — sensitivity is how much the counter MOVED, not how large the rule
  family is. Measured: the flip moved the binding counters **+10.59 %** (ARM 1 812 963 769 → ARM 2
  899 064 022, `guard_ab_entries.txt`), **3.49× larger** than the family's own 24 644 435 entries, so
  the counters saw that change plainly. The published bound had reasoned the opposite from an
  inference — *"the flip's entry delta is strictly smaller"* — that a tracked artifact in its own
  leaf had already refuted. ⇒ the factor is **retired, not re-computed**: it fails under every
  available reading (0.41× on the point estimate, 1.82× on the most adversarial pairing) and `.20`
  (b) established that no admissible wall-clock figure for the change exists to rebuild it from.
  ⭐ The share itself is a correctly measured quantity and is **unaffected**.

  ⚠️ The share read **0.681 %** (and the retired factor `~35×`) until `ENGINE-UNIVERSAL-SERVICES.21`:
  the classifier counted only `_lr_base`/`_lr_suffix`, so it saw 97 of the parser's **127** LR rule
  names — no `_lr_seed`, and, in a family named for the GUARD, **no `_lr_guard` rule at all** —
  leaving **75.1 %** of the family's entries uncounted. ⚠️ That denominator read **128** until `.21`
  slice 2 and was wrong by one — its own decomposition (97 + 24 `_lr_seed` + 6 `_lr_guard`) already
  summed to 127, and three independent surfaces of the generated parser agree (the `RULE_NAMES`
  registry, the `fn parse_*` names, the string literals). It is now GATED, not carried:
  `python3 stimuli/sv/corpus_parse_cost.py --verify-families` re-derives it across all ten generated
  parsers and refuses on drift. It guards STRUCTURAL work exactly; it does not price the fused
  graph. Neither metric alone is sufficient and the report says so every run.

### 3.7b Parser ARMS — "two spellings of one fix: which is correct, and which is cheaper?"
- **WHAT:** `docs/tasks/artifacts/sv_corpus_grad/strictness_cost_arms/` — build a candidate grammar
  arm, regenerate, rebuild, and measure it, then diff two arms per rule and per file.
  `SV-CORPUS-GRAD.13c.2k`.
  * `apply_arm.py --arm {t_only,designB,designA}` — writes the arm's grammar, DERIVED from HEAD's
    text read out of **git** (never the working tree) and refusing to stack arms. Design A's 45
    call sites come from `raw_identifier_census/census.py` at apply time and the count is asserted.
  * `measure_arm.sh <arm>` — apply → `focus_systemverilog` → `cargo build` → measure → restore.
  * `arm_cost.py --measure --arm N --probe P` / `--compare BASE ARM` — totals, the FULL per-rule
    entry/memo/committed breakdown, and **each file's `accepted` verdict**.
  * `arm_graph.py --arm N` — freezes that arm's grammar reference graph; `containment.py` refuses
    unless the graph's grammar sha equals the arm's.
  * `inline_decision.py` — ⭐ **the one to use**: prints the GENERATOR's own verdict per rule per arm
    (`<class> refs=<n> body_nodes=<n> INLINED|over-budget`) beside the `duplication_cap` it is
    judged against, from `--report-fusibility-census` (5.3).
  * `inline_census.sh` — counts `memoized_call` / `inlined_frame_call` sites in the GENERATED
    parser. ⛔ **These are EMITTED sites after transitive expansion, NOT the decision.** Reading
    them as the decision is a measured error: it made `-0236` explain an arm's +1.78 % as an
    inlining change that the census says never happened (corrected in `-0238`). Use it to see where
    a rule ended up, never to infer why.
  * `arm_verdicts.sh [probe]` — the pinned reproducers' verdicts under whatever probe is on disk.
- **WHEN:** whenever a fix is refused on cost, or two spellings of one change must be compared.
  ⛔ **Before writing "irreducible" anywhere.**
- ⭐⭐ **A DEBUG PROBE IS ENOUGH, AND THAT IS WHAT MAKES THIS AFFORDABLE.** Measured this slice at
  HEAD: the debug and release probes agree on all three binding counters AND on all **1,077** rules,
  delta 0. An arm is then ~3 min of `cargo build` instead of ~22, so three arms cost ~20 minutes.
- ⛔ **CHECK VERDICTS BEFORE COMPARING COSTS.** Two arms that accept different languages have
  incomparable costs. `--compare` prints a per-file verdict diff and says so loudly; schema 1 did
  not record verdicts and a `committed` move of `+51,611` was uninterpretable until it did.
- ⛔⛔ **NEVER PRICE A CALL-GRAPH CHANGE FROM THE OLD CALL GRAPH — and get the mechanism from the
  generator, not from its output.** A rule is INLINED (taking **no memo lookup at all**) while its
  reference count × body size stays under a duplication budget the census names. Measured on this
  fix: moving 45 references carried `identifier` 47 → 1 refs (over-budget ⇒ memoized → INLINED) and
  `non_keyword_identifier` 7 → 52 (INLINED → over-budget ⇒ memo-served at 94.5 % hits) — **both
  rules across the budget, in opposite directions**. ⚠️ The bare-alias arm changed NEITHER decision;
  its rise was redirected speculation. Re-derive with `inline_decision.py`.
- ⛔ **THE ARM BASE IS A PINNED COMMIT.** These arms are transforms of a PRE-FIX grammar, so the
  harness reads `ARM_BASE_COMMIT` (`arm_graph.py`), not `HEAD` — which broke the day the fix landed
  and every anchor moved, i.e. a tracked reproduction script that could no longer reproduce. It
  refused loudly, which is the only reason it was caught.
- ⛔ **AN ARM'S IDENTITY IS ITS PROBE FINGERPRINT, NOT THE TREE.** Every arm file records
  `--parser-fingerprint` and a grammar sha DERIVED from the arm NAME. The first cut derived it from
  the working tree and stamped HEAD's grammar onto a designA measurement —
  `ENGINE-UNIVERSAL-SERVICES.20` slice 4 in miniature.
- ⚠️ **TRAPS THIS DIRECTORY HAS ALREADY PAID FOR** (all three are guarded now): a `parents[]` depth
  one level short (twice — the `accepts_invalid` probe and this driver); `${1:?usage: … {a|b}}`,
  where the first `}` closes the expansion and the arm name arrives as `t_only}`; and a verdict
  check that grepped the probe's message for `/accept/` while the probe echoes a path containing
  the word *"accepts"*. **Use `parseability_probe --parse`'s EXIT CODE: 0 = ACCEPT, 1 = REJECT.**
- **ROUTING:** the counters take the PROTOCOL graph (3.4/3.5), so the fused `cascade_*` twins are
  invisible here exactly as they are to the ratchet (3.7).

### 3.8 Sampling a parse with `/usr/bin/sample` — the ONLY view of the fused graph, and its four traps
- **WHAT:** `/usr/bin/sample <pid> <secs> 1 -f out.txt` on a **BARE** parse. ⭐ This is the only
  instrument in the toolbox that observes the FUSED `cascade_*` graph: every counter-based tool
  (3.1/3.4/3.5/3.6) routes the parse to the PROTOCOL graph by construction, so none of them can see
  the code a production parse actually runs.
- **WHEN:** attributing a *speed* result — above all when a counter says a change is small and the
  clock says it is large. `ENGINE-UNIVERSAL-SERVICES.20` slice 2.
- **HOW:**
  ```bash
  ./rust/target/release/parseability_probe --parse systemverilog big.sv --profile sv_2017 & PID=$!
  sleep 0.3 && /usr/bin/sample $PID 4 1 -f /tmp/prof.txt && wait $PID
  ```
  Pick a file with a multi-second parse; `stimuli/sv/characterization/durations.tsv` sorted on
  column 4 names them.
- ⛔ **TRAP 1 — THE DENOMINATOR IS THE WORKER THREAD, NOT THE PROCESS.** The probe runs the parse on
  a spawned thread (`pgen-parseability-probe`) while the main thread blocks in `__ulock_wait`.
  `sample` reports BOTH at the same count, so dividing by the process total **halves every
  percentage**. Take the non-main thread's root count as the denominator; a main thread that is
  100 % `__ulock_wait` is idle, not work.
- ⛔⛔ **TRAP 2 — SELF-TIME ANSWERS "WHERE IS THE CPU", NOT "WHO CAUSED IT".** Measured on the SV LR
  machinery: **self 1.9 %, inclusive 27.3 %** — a 14× gap. A rule that re-routes a hot path costs
  almost nothing itself and everything underneath. Read `Sort by top of stack` for self-time and the
  `Call graph:` section for inclusive, and when summing an inclusive share count **only the
  OUTERMOST** nodes of the family, or nested frames double-count.
- ⛔⛔⛔ **TRAP 3 — THE LINKER FOLDS THE GENERATED PARSERS TOGETHER, SO A PER-FAMILY SYMBOL NAME CAN
  BE A LIE.** A SystemVerilog parse will attribute samples to
  `generated_parsers::rtl_frontend::…::cascade_error_from_parse` — because the binary contains **10**
  generated families and exactly **one** copy of that helper. The generated helpers are byte-identical
  across families (one codegen template), the linker merges them, and the surviving symbol name is
  arbitrary. ⚠️ It fails **silently and plausibly**: it looks like a cross-parser call.
  **Check before trusting any per-family attribution** — and check the specific symbols your
  conclusion rests on, not the binary in general:
  ```bash
  nm -C rust/target/release/parseability_probe | grep '::<helper>::'    # 1 symbol, N families = FOLDED
  nm -C rust/target/release/parseability_probe | grep -E '_lr_(base|suffix|seed|guard)' \
      | grep -oE 'generated_parsers::[a-z_0-9]+::' | sort | uniq -c     # 165 of 174 are SV's own = SAFE
  ```
  Folding is heaviest on the generic helpers (`cascade_error_from_parse`, `byte_window_lossy`) and
  absent on rule methods carrying family-specific literals (`create_contextual_error`: 9 symbols;
  `memoized_call`: 516).
  ⭐ **The MECHANISM is now proven, not inferred** (`ENGINE-UNIVERSAL-SERVICES.20` slice 4). Symbol
  counts alone cannot separate folding from *"this family's copy was INLINED away, so `sample`
  attributed to the nearest preceding symbol"* — the two make opposite call-graph predictions. Decide
  it with `otool` annotated disassembly, and with the PRE-LINK archive:
  ```bash
  otool -tV -p '<mangled SV caller symbol>' rust/target/release/parseability_probe | grep -A0 'bl\t'
  #   bl __ZN4pgen17generated_parsers12rtl_frontend…cascade_error_from_parse…   <- SV really calls it
  strings -a rust/target/release/deps/libpgen-*.rlib | grep -oE '_ZN4pgen17generated_parsers.*24cascade_error_from_parse17h[0-9a-f]+E' | sort -u | wc -l
  #   9   <- the COMPILER emitted 9 per-family copies; the linked binary has 1 ⇒ the LINKER folded
  ```

- ⛔⛔ **TRAP 4 — A `sample` REPORT HAS FOUR SECTIONS AND ONLY THE FIRST IS A CALL GRAPH.** In order:
  `Call graph:` · `Total number in stack (recursive counted multiple, when >=5):` ·
  `Sort by top of stack, same collapsed (when >= 5):` · `Binary Images:`. Section 2 is **not** a tree
  and its rows are indented like one. Measured (`ENGINE-UNIVERSAL-SERVICES.20` slice 4): a parser
  that stopped at sections 3 and 4 but not at section 2 ingested 694 of its lines as call-graph rows,
  re-parented **14 022** samples, and reported LR self-time at **18.12 %** against a true **2.27 %** —
  **8× wrong**, and wrong in the direction that produced a (false) finding.
  ⭐⭐ **And the obvious ground-truth control cannot catch it.** `sum(self) == worker root count` is a
  CONSERVATION identity; a MISASSIGNMENT conserves the total, so it passes. Two controls that DO work,
  both cheap:
  ```bash
  # C-a (free): self-time is non-negative BY DEFINITION — assert it, and refuse rather than print.
  # C-b (binding): your per-symbol self-time must EQUAL sample's own `Sort by top of stack` table
  #                for every symbol it lists (139-209 symbols/report). That table is an INDEPENDENT
  #                oracle sitting in the same file. Its `>= 5` collapse only hides the tail.
  ```
  ⇒ [[a-conservation-control-cannot-catch-a-misassignment]]. ⚠️ And when a re-derivation disagrees
  with a published number, **the re-derivation is the new instrument and carries the heavier burden
  of proof** — it has been run once.

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
- ⭐⭐ **THE LEFT-RECURSION VERDICT IS DERIVED FROM THE ELIMINATION PASS, NOT ASSERTED (`GRAMMAR-WELLFORMED.A2.6`).** The lint runs on the **post-elimination** grammar, so a cycle still present is one the pass **declined**, and the headline says exactly that — `left_recursion_eliminated=N` (the pass's own record, and it NAMES the rules) versus `left_recursion_unhandled=M` (a **warning**). Until `A2.6` this class printed *"handled by PGEN's LR elimination + runtime cycle-breaking (informational, not an error)"* about every cycle it found; measured on the shipped SV grammar that sentence was false for **30 of 30** — the pass had rewritten **2** rules. ⛔ An `unhandled` cycle is not merely untidy: the runtime guard does not handle it, it **rejects** same-position re-entry, so those derivations are unreachable. The demonstrated case was SV's `casting_type -> constant_primary -> constant_cast -> casting_type` — LRM-legal `int'(2)'(3)` was REJECTED (`ENGINE-UNIVERSAL-SERVICES.13`), and `.17` slice 9's guarded admission **CLOSED it**. ⭐ **Current, re-derived over all 11 grammars with a generated parser (`ENGINE-UNIVERSAL-SERVICES.27`, 2026-08-16): every one is `left_recursion_unhandled=0`** — SV `eliminated=5` (**2 direct + 3 indirect**), `ebnf` `eliminated=1` (**0 direct + 1 indirect**), `return_annotation` and `semantic_annotation` `eliminated=1` (1 direct), the other seven `0/0`. ⛔⛔ **THE HEADLINE COUNTED ONLY THE *DIRECT* PASS UNTIL `.27` FIXED IT, so every number this bullet used to publish was an UNDER-count**: `.13` slice 5 added the indirect pass and its outcome fields and never extended the lint call site, which read `eliminated_base_rules` alone. SV published **2** and is **5**; `ebnf` published **0** and is **1**. The headline now prints the split — `left_recursion_eliminated=5 (info — 2 direct + 3 indirect, disjoint by construction)` — and the `[info]` line NAMES the indirect rules (`casting_type (indirect)`, `property_expr (indirect)`, `incomplete_class_scoped_type_sv_2023 (indirect)`), which on a purely-indirect grammar did not print at all before. The ONLY grammar with survivors is `systemverilog_lrm_profiled_wrapper` (**23** unhandled, 4 eliminated), which is the `lrm_extraction_harness` and **not a family** — it also carries 42 profile-orphans, 23 shadowed branches and 5 undefined references, and has never generated a shipped parser. ⛔ The previously published line here (*"SV **30**, wrapper 23, `ebnf` **5**"*) was the PRE-flip state and had gone stale by two of its three numbers; `ebnf` is **0**. ✅ **THAT CONTEST IS RESOLVED (`ENGINE-UNIVERSAL-SERVICES.27`, 2026-08-16) AND `--verify-families` WAS THE CORRECT SIDE.** It reads `generated/ebnf.rs` and finds **6** declared `_lr_*` names (`base=1 seed=1 suffix=4`); the lint said `eliminated=0`. Neither instrument was stale — `generated/ebnf.rs` re-derives **byte-identically** from today's `grammars/ebnf.ebnf` (sha `97c17533…`), so the `.16` seed-divergence hypothesis is REFUTED too. The lint headline was DIRECT-ONLY, and `ebnf` is the only one of the eleven grammars whose elimination is *purely* indirect, so it was the only row where the omission read as *"nothing was eliminated"* — SV's non-zero direct count masked its own three. `--report-indirect-lr-plan` named it all along: `indirect_eliminated_base_rules=1 … ✅ absorbed at 'return_expression'`. The full join is now **11 of 11 consistent**. ⚠️ LR elimination is CORRECT and CLOSED for everything that ships — what remains open is its **cost**, and ⛔ the `+24.3 % parse time` this line used to name is **REFUTED** (`ENGINE-UNIVERSAL-SERVICES.20` slice 5): the measured, deterministic cost is **+10.59 % rule entries**, and no admissible wall-clock figure exists.
- ⭐ **`PGEN_LINT_DUMP_ALL=1` prints EVERY finding of EVERY class** (the per-class print cap is 40; left-recursion's was **10**, and it hid 20 of SV's 30 — the sweep that found `A2.6` had to go around the instrument). The truncation line now names the escape:
  ```bash
  PGEN_LINT_DUMP_ALL=1 ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --lint-grammar
  #   [warn]  … rule 'casting_type' is left-recursive (cycle: …) and PGEN's LR-elimination pass did NOT eliminate it …
  #   [note]  ... and 12 more always-succeeds-alternative notes (set PGEN_LINT_DUMP_ALL=1 to print all 52)
  ```

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

### 5.5 `--report-indirect-lr-plan`
- **WHAT:** (ENGINE-UNIVERSAL-SERVICES.13, `rust/src/ast_pipeline/indirect_lr_plan.rs`) the INDIRECT-left-recursion SURVEY — for every cycle `--lint-grammar` reports as `left_recursion_unhandled`, which rule could absorb the chain (`X := X_lr_base ( X_lr_suffix )*`), what SUFFIX it would iterate, how many CLONE rules that costs, and — the load-bearing column — every **starvation site**: a rule holding the candidate at its left corner with a NON-EMPTY residual, which a greedy non-backtracking `*` can starve. PURE analysis; plans nothing, changes no grammar byte, always rc 0.
- **WHEN:** `--lint-grammar` printed `left_recursion_unhandled=N` and you need to know what a fix would have to DO — ⛔ **the lint names the cycle, never the fix, and the rule it names FIRST is a measured regression to eliminate at**: rewriting SV's `casting_type` turns the accepted `int'(3)` into a rejection (`docs/tasks/artifacts/engine_universal_services/indirect_lr/`, probe P2).
- **HOW:**
  ```bash
  ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --report-indirect-lr-plan
  PGEN_INDIRECT_LR_DUMP_ALL=1 ./rust/target/debug/ast_pipeline grammars/<g>.ebnf \
    --report-indirect-lr-plan --indirect-lr-plan-json /tmp/survey.json

  # the BEFORE column, on the same binary — hold the grammar at the direct/wrapper-only shape
  ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --lint-grammar \
    --no-eliminate-indirect-left-recursion

  # the OTHER before column (`.17` slice 9) — the pass runs, but with the pre-flip ADMISSION
  ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --lint-grammar \
    --indirect-lr-admit-starvation-safe-only
  ```
- ⭐ **`--no-eliminate-indirect-left-recursion` is the A/B switch, and it is a MEASUREMENT switch, not a policy one.** A shipped parser is always generated with the pass on. It exists because every left-recursion counter this repository quotes (`--lint-grammar`'s headline, this survey's census) loses its "before" the moment the pass lands, and a before→after taken from two different binaries is not a measurement.
- ⭐⭐ **`--indirect-lr-admit-starvation-safe-only` IS THE SECOND A/B LEVER, AND THE TWO ARE NOT THE SAME QUESTION** (`.17` slice 9). The first asks *"what if the indirect pass never ran?"*; this one asks *"what if it ran with the pre-flip ADMISSION?"* — the criterion every generated parser shipped under up to `PGEN-ENGINE-UNIVERSAL-SERVICES-0030`. Measured on one binary: `systemverilog` `left_recursion_unhandled` **28 under the lever → 0 shipped**, every other family 0 under both. ⛔ **It NARROWS, and a parser built with it REJECTS LRM-legal SystemVerilog** (`8'(1)` in a constant expression). It prints a loud banner at default verbosity for that reason; it is a measurement arm, never a way to build a deliverable, and no `make` target passes it. ⭐ It is also what keeps the `guard_feasibility` and `guard_dry_run` banks measuring their own subject: a census OF THE UNABSORBED KNOTS is empty once the knots are absorbed, and reading that emptiness as "option (iii) is free" would be exactly backwards.
- **OUTPUT:** `=== INDIRECT-LR-SURVEY: '<grammar>' ===`, then per candidate `routes=/seeds=/clone_cost=/verdict=MAY-ABSORB|STARVED`, each route as `path -> base   suffix: …`, and each starvation site. `PGEN_INDIRECT_LR_DUMP_ALL=1` lifts the per-candidate print caps (3 routes / 5 sites / 5 declines).
- ⭐⭐ **The third header line is what the ENGINE DID, not what it could do** (slice 5): `indirect_eliminated_base_rules=/indirect_clone_rules=/indirect_refusals=/indirect_guard_chains=` (the last one `.17` slice 7; ⛔ **it was `0` everywhere until `.17` slice 9 flipped the admission and is now `3` on `systemverilog` and `0` on every other family** — see the census bullet below), then one `✅ absorbed at '<rule>'` per rewrite, one `🛡 guard '<rule>' [<positions>] chain: … (max_hops=N) residual '…'` per synthesized guard chain with its `sites:`/`rules:` line, and one `⛔ REFUSED '<rule>': <reason>` per decline. ⛔ The `🛡` block on the SHIPPED path is `.17` slice 9's own addition and the flip is why it had to exist: before it, a reader was told *how many* guards their parser contains and never *which*. ⛔ Read it FIRST: the survey runs on the POST-elimination grammar, so a knot the pass already absorbed is simply absent from the candidate list, and a candidate still printed `MAY-ABSORB` is one the pass **refused** — the refusal names why, and for a partial shear it prints the surviving cycle path.
- ⭐ **The verdict is calibrated against measured ground truth, in both directions**: on the P1 synthetic it independently calls `prim` MAY-ABSORB (probe P3 accepts all five inputs on both oracles) and `ct` STARVED (probe P2 regresses) — unit-tested in `indirect_lr_plan.rs` so the criterion cannot drift back.
- ⛔ **DECLINES LOUDLY.** The walk only follows a **bare leading rule reference**; a cycle closing through a nullable prefix, a quantifier or a group is listed under `DECLINED` with its reason (`no_bare_left_corner_route`), never silently dropped.
- **Measured, post-slice-9** (the pass absorbs what it can, so these are the RESIDUALS): `ebnf` 5 → **0**, `systemverilog` 30 → **0** (⛔ **28 until `.17` slice 9**, when the flipped admission absorbed the cast/call and property knots at `casting_type` and `property_expr`; before that only `incomplete_class_scoped_type_sv_2023` was absorbed), `systemverilog_lrm_profiled_wrapper` 23 → 23 (its 13 guard-feasible candidates are refused for a missing return annotation under either admission, which is why it is the flip's blast-radius control). Every other grammar was and stays 0.
- ⛔⛔ **`starvation-safe candidates: 0/28` ON SYSTEMVERILOG WAS THE LINE TO READ UNTIL `.17` SLICE 9, and the shipped report now prints `candidates=0` because the knots are absorbed. Pass `--indirect-lr-admit-starvation-safe-only` to see it. It meant UNPLANNABLE, not unplanned.** Every candidate on the cast/call and property knots is transitively starved, so no choice of base rule helps; the blocker is PGEN's **commit-once** combinators (`ENGINE-UNIVERSAL-SERVICES.17` / `.15`), not the route walk. ⛔ **Read "commit-once", not "the greedy `*`"** — `.17` slice 4 measured the CHOICE committing too (one `best_content` winner, losers discarded, `ast_based_generator.rs:5037`), which is a second, independent starvation source; the earlier framing as a quantifier-only asymmetry is superseded ([[project_pgen_gives_back_at_neither_combinator]]).
- ⭐⭐ **`guard-feasible candidates: N/M` IS THE SECOND HEADLINE, and it answers a DIFFERENT question — read both** (`.17` slice 2). The line above says which rules a rewrite may target **today**; this one says which it could target if the eliminator synthesized a **call-site follow-restriction guard** on the sheared clone (`X_guarded := X_lr_base ( X_lr_suffix &( residual ) )* &( residual )` — ⛔ STRUCTURAL and TWO positions since `.17` slice 4; the byte-set single-position form this line used to print is measured dead). A candidate that is `STARVED` **and** guard-feasible is exactly the population that design would unlock. Measured **under `--indirect-lr-admit-starvation-safe-only`** (⛔ since `.17` slice 9 that population IS what the shipped pass absorbs, so a bare report shows `0/0` — an empty census, not a free one): `systemverilog` **0/28 starvation-safe vs 16/28 guard-feasible**, `systemverilog_lrm_profiled_wrapper` **3/18 vs 13/18**. Both remaining SV knots are guard-feasible at their DOMINATORS with one variant each — `constant_primary` (`max_hops=1`) and `property_expr` (`max_hops=0`).
- **Per-candidate `guard:` line and per-site `[guard=… first=… hops=…]` column** — how to read them:
  - `suffix_first=` the FIRST byte set of the suffix the `*` iterates, unioned over ALL routes (one base rule emits one `*`, so one guard must be complete against all of them).
  - `variants=N {…} {…}` the DISTINCT residual byte tests the surviving sites need — the "do two holders of the same clone disagree?" price. 17 of SV's 28 candidates report exactly 1. ⛔⛔ **IT IS A LOWER BOUND ON THE CHAIN COUNT, NOT THE CHAIN COUNT** (`.17` slice 7, measured on the driver's own first pick): it keys on a FIRST **byte set**, which is what the DEAD byte-test form compared, while the shipped guard is a structural sub-parse. `casting_type` reports `variants=1` and the planner emits **2** chains — `tick lparen constant_expression rparen` and `tick lparen expression rparen` share a FIRST set and are different rules. Read the `🛡` lines for the real count.
  - `max_hops=` how many TRANSPARENT rules the guard must be cloned through to reach the `*`. `0` means the holder names the base directly. ⛔⛔ **IT IS A LOWER BOUND ON THE CLONE COUNT** (`.17` slice 7): `hops` is the SHORTEST transparency distance, and the per-site `chain=` beside it is EVERY rule on any transparent path — they disagree wherever transparency BRANCHES. SystemVerilog's dialect twins do exactly that (`primary` reaches `cast` through `primary_sv_2017` AND `primary_sv_2023`, so five rules get cloned where `hops=3` reads as four), and leaving either twin unguarded leaves a live unguarded route to the same starvation. Measured: **6 of 129** SV sites and **5 of 77** wrapper sites (bank D13/D13b). ⇒ price a guard from `chain=`, never from `max_hops`.
  - per-site verdicts: `guardable` (competing, `FIRST(suffix) ⊆ FIRST(residual)` ⇒ the guard refuses the fatal iteration and provably no earlier one) · `no_competition` (disjoint ⇒ **no guard owed**) · `residual_nullable` (the residual can match empty ⇒ the holder cannot starve ⇒ **no guard owed**) · `guard_incomplete` (competing but not contained ⇒ the guard would cut the loop short at an intermediate iteration) · `undecidable` (a FIRST set is `unresolved`, or the suffix is nullable).
  - `chain=a>b>c` (`.17` slice 7) — the RULES a guard would be cloned through for THIS site, deepest first, ending at the base rule. `hops=` is `len(chain)-1` only when the transparency is a simple path; read the bullet above for what the difference costs.
- ⛔⛔ **THE CENSUS IS NOW APPLIED — `.17` SLICE 9 FLIPPED IT, AND THIS BULLET SAID THE OPPOSITE FOR SIX SLICES.** It read *"THE CENSUS IS REPORTED, NOT APPLIED … consulting the guard census there would change which knots get absorbed, i.e. a real parser change owed a two-sided repro ratchet"*. That ratchet was paid and the change landed: the eliminator's criterion is `guard_admissible_candidates()`, a surviving starvation site is closed by an emitted guarded clone chain instead of declining the whole knot, and `generated/systemverilog_parser.rs` is the one artifact that moved (the other 10 byte-identical). ⭐ The check that used to be *"`indirect_guard_chains=` is 0 everywhere"* is now an EXACT-SET check, which is strictly stronger — `casting_type_lr_guard0[loop]`, `casting_type_lr_guard1[loop]`, `property_expr_lr_guard0[loop+trailing]` on `systemverilog` and **0 on every other family** (bank `guard_dry_run` D11/D11a/D11b). A count of 0 could not tell "the criterion holds" from "the pass stopped running"; a set can. ⛔ Any movement in that set — an emptied set, a fourth chain, a moved position — is a shipped-parser change owed a two-sided repro ratchet and a corpus re-measure, in either direction.
- ⭐⭐ **`--indirect-lr-plan-guard-dry-run` ANSWERS THE NEXT QUESTION — *"of the 16, how many actually reach a PLAN?"*** (`.17` slice 3). The census says a guard is EXPRESSIBLE at a candidate; it says nothing about the three refusals that live DOWNSTREAM of the starvation gate (a hop with no declared return annotation, the trial re-lint, the ambiguity comparison), and those are unobservable for a starved candidate because `plan_elimination` is only ever reached from `safe_candidates()`. The flag admits the guard-feasible candidates into the **real** driver on a CLONE and prints what it did. Measured on the shipped grammar **under `--indirect-lr-admit-starvation-safe-only`**: `would_absorb=2 would_refuse=0 clone_rules=24 left_recursive_rule_rows 28 -> 0` — **`casting_type` and `property_expr`, the two DOMINATORS, clear all 28 rows between them**. ⇒ 16 guard-feasible candidates are 16 rules on TWO knots, not 16 rewrites. ⭐⭐ **AND `.17` SLICE 9 TURNED THAT PREDICTION INTO A SHIPPED MEASUREMENT: the flip absorbs exactly those two rules and the real lint reads `left_recursion_unhandled` 28 → 0.** What the dry run predicted is what the pass did, rule for rule — which is the strongest available evidence that this instrument models the driver rather than approximating it. ⛔ Without the lever the dry run now correctly answers `would_absorb=0`: nothing left to unlock, because it shipped. Its verb is `🛡 would guard` where the shipped block's is `🛡 guard`, so a scraper can tell the two apart in one report.
  - ⛔⛔ **`28 -> 0` IS PLAN-STAGE REACHABILITY, NOT CLOSURE — and the reason NARROWED at `.17` slice 7 rather than going away.** Through slice 6 the dry run emitted no guard at all, so the grammar it built was the measured-regressing one (`casting_type` unguarded turns the accepted `int'(3)` into a rejection, probe P2). It now SYNTHESIZES the guarded clone chains — but nothing here generates a parser or runs an input, so no verdict about what the rewritten grammar accepts or rejects is available from this flag at all. Read it next to the `~` exactness result (0 of 129 sites exact), never instead of it.
  - ⭐⭐ **`guard_chains=N guard_rules=M` AND THE `🛡` LINES ARE THE EMISSION** (`.17` slice 7): per chain, the guarded base rule, its `[positions]` (`loop` · `trailing` · `loop+trailing`), the transparent `chain:` of rules cloned, the `residual` both lookaheads test, the holder `sites:` whose left corner was repointed, and every `rules:` name synthesized. Measured on `systemverilog`: `guard_chains=3 guard_rules=6` — `casting_type_lr_guard0/1` `[loop]` and `property_expr_lr_guard0` `[loop+trailing]`, which is slice 5's per-candidate `seed:` verdict reproduced by a different code path. ⭐ Since `.17` slice 9 those same three are what the SHIPPED pass emits, printed by the same renderer under the `guard` verb. ⛔ **Every SV chain is `max_hops=0`** — printed by the tool since slice 9 rather than counted off `chain:` by eye, and mechanised as bank `guard_dry_run` D11c — so the corpus-scale run **never exercises the guarded HOP clone** `X_lr_guard{v}_<hop>`. Its coverage is the unit test `the_guarded_clone_chain_reproduces_the_hand_written_g7_shape` plus the end-to-end `guard_parses` bank (slice 8), and `.17` acceptance (d) is discharged by that measurement rather than by an SV exercise.
  - ⛔ **READ THE `inputs:` LINE FIRST.** `compose_route_template` returns *"nothing to compose"* on a grammar that declares no annotations, so `would_refuse=0` has two readings and only the annotation census separates them: `systemverilog` **1069** annotated rules vs `systemverilog_lrm_profiled_wrapper` **21**. That is also the whole explanation of the wrapper's contrasting `would_absorb=0 would_refuse=13` (every one *"declares no return annotation"*): its body is generated from the LRM Annex A markdown and carries a single `->` line in 148 KB. ⇒ **the two SV views are structurally comparable and NOT annotation-comparable** — the wrapper is the right oracle for `.15`'s scar-tissue argument and the wrong one for any composability question.
  - **Bank:** `docs/tasks/artifacts/engine_universal_services/guard_dry_run/probe.sh` → `GUARD-DRY-RUN: N/N as declared` (the total is DERIVED from the cases that ran — never read a stored one). Opt-in: without the flag the report carries no dry-run section at all.
- ⛔⛔ **`no_competition=0` ON SYSTEMVERILOG IS A FACT ABOUT THE GRAMMAR'S LAYOUT MODEL, NOT A BUG.** `trivia := (line_comment | block_comment)*` (`grammars/systemverilog.ebnf:619`) is nullable and every token is `trivia "X"`, so **`/` is in the FIRST set of every token** and no two token-led sets are ever disjoint. Two consequences: the disjointness refinement cannot fire here, and a byte-test guard is defeated by an intervening comment (`int'(2)/*c*/'(3)`) — sound (it can only fail to fire) but not exhaustive. A trivia-aware guard would have to be a structural lookahead, not a byte test.
- ⛔⛔ **`~` ON A BYTE SET IS THE QUALIFIER ON EVERY `FEASIBLE` ABOVE — feasible means SOUND, not CLOSED.** `~` marks an OVER-APPROXIMATED FIRST set: the guard passes at positions the residual cannot actually start from, so it silently declines to refuse the fatal iteration. It never becomes unsound (`L(X_guarded) ⊆ L(X)` — an over-permissive guard can only fail to fire, never over-accept), but it is not a proof the knot closes. **Measured: 129 of 129 sites on SystemVerilog and 77 of 77 on the wrapper are `~` — not one exact guard exists** (⛔ the denominator read **157** until `.17` slice 5 corrected it: the bank counted lines containing `first=`, and the per-candidate `suffix_first=` summary line carries that substring, so 28 candidate rows were counted as sites — the conclusion is unaffected, every candidate summary is approximated too), for two compounding reasons: `FirstSetSummary::byte_decided` holds only for single-byte-decided shapes so any multi-element residual is approximate by construction, and the nullable `trivia` above puts `/` in every set. ⇒ a trivia-aware STRUCTURAL lookahead, not the byte test, is the form `.17` slice 3 has to price.
- ⭐⭐ **SLICE 4 MEASURED THAT FORM, AND IT WORKS — but a guard on the `*` ALONE IS NOT ENOUGH.** Bank: `docs/tasks/artifacts/engine_universal_services/guard_effectiveness/probe.sh` → `GUARD-EFFECTIVENESS: N/N as declared` (the total is DERIVED by the run — never read a stored one; ⛔ this bullet said **`37/37`** until `.17` slice 8, for a bank that has been 44 rows since slice 6, while the bullet twelve lines below said `44/44` — two numbers for one bank in one section, which is `CI-PARITY-GATE-ROT.29`'s class living in the TOOLBOX rather than in a bank), every row on BOTH oracles, over a synthetic that carries SystemVerilog's own nullable-`trivia` layout model. Three results, and two of them were not on the leaf's map:
  - **The byte form is DEAD, not merely weaker.** `k = n'(n)/*c*/;` — the starvation case with a comment at the iteration boundary — is ACCEPTed by the structural guard and **REJECTed** by the byte guard, because `/` ∈ `FIRST(residual)` makes the byte test pass exactly where it had to refuse. Next to `exact = 0 of 129`, the cheap form is a proof of closure nowhere on SystemVerilog.
  - ⛔⛔ **There is a SECOND starvation, at the CHOICE, and no guard on the `*` can reach it.** `k = t'(n);` starves under the unguarded, byte-guarded and structurally-guarded loops alike, and still starves with the quantifier deleted outright: the over-long match comes from the rewritten base's own SHEARED CLONE winning the holder's alternation. Shipped analogue: `casting_type := simple_type | constant_primary | …` over a `constant_primary` whose post-rewrite base carries the `constant_cast` clone, so `initial k = int'(1);` — accepted today — would REGRESS. ⇒ absorbing at these knots with the loop guard alone trades one defect for another.
  - ⭐⭐ **The repair has TWO POSITIONS on ONE clone, and they close DISJOINT starvations.** `X_guarded := X_lr_base ( X_lr_suffix &( residual ) )* &( residual )`. Per-iteration stops the LOOP at the right count and cannot touch an over-long seed (there the loop runs zero times); trailing refuses an over-long SEED and cannot touch the loop (by then the possessive `*` has committed, with no give-back to a shorter count). Only both together close all six measured shapes, and both are CALL-SITE scoped — a residual-free holder of the same rule breaks under either.
  - ⭐⭐ **AND THE CALL-SITE-SCOPED SHAPE IS NOW BUILT, not just decided** (`.17` slice 6, bank row `g7_guarded_clone_chain`). Until then the ladder proved the two halves and never the combination: `g3` (guard on the SHARED rule) closes all six inputs, `g4` (same guard plus a residual-free holder) REJECTs `k = n;`, and nothing expressed the shape with `g3`'s power and without `g4`'s damage. `g7` is `g4` with ONE difference — the lookaheads moved onto a clone chain:
    ```text
    outer_cast := ct_guard tick lparen lit rparen     # only the holder's LEFT CORNER is repointed
    ct         := kw | prim                           # the originals stay, for the residual-free holder
    prim       := prim_base ( prim_suffix )*
    ct_guard   := kw | prim_guard                     # one guarded clone per TRANSPARENT hop (= guard_hops)
    prim_guard := prim_base ( prim_suffix &( R ) )* &( R )
    ```
    Measured on BOTH oracles: `g7` accepts all six starvation inputs **and** `e7` (`k = n;`), the row `g4` rejects. ⭐ The fallback IS the mechanism — when the trailing guard refuses the over-long seed, `prim_guard` FAILS and `ct_guard`'s other alternative `kw` wins the tournament with the short match, handing the holder its residual back; a guard placed at the holder's own call would have nothing to fall back to. ⇒ this grammar is the rule-for-rule TARGET a guard planner must synthesize.
    ⛔⛔ **READ `g7`'s SIX ACCEPTS CORRECTLY — ONLY `e7` DISCRIMINATES** (corrected by `.17` slice 8, which found it with a plant). `g7` carries `g4`'s two holders, and this bank already runs `g4`/`g5` on `e1`+`e7` ONLY *"because their second alternative absorbs e2–e6 on its own, so those rows would pass under BOTH hypotheses"* — the same is true of `g7`, whose `e1`–`e6` reach `prim` **unguarded** through that alternative. ⇒ on `g7` those six rows prove the guard does NO DAMAGE; the POWER claim rests on the single-holder rungs `g0`/`g2`/`g6`/`g3`, and the discriminating guarded-chain control is `guard_parses/s2_holder_only.ebnf`. Every verdict in this bank is still correct — what was too strong was the sentence about six of them.
- ⭐⭐ **AND SINCE `.17` SLICE 8 THAT TARGET IS HIT BY PGEN ITSELF, EXECUTED — and since `.17` SLICE 9 it is what `make` BUILDS.** Everything above is measured on grammars a PERSON wrote (`g0`–`g7` are hand-eliminated post-rewrite shapes) or on the emitter's generated AST. Slice 8 took the guard-feasible admission all the way to **codegen** behind an opt-in flag so a parser could be built from PGEN's own guarded emission and fed bytes; slice 9 made that admission the shipped one, so the flag inverted into `--indirect-lr-admit-starvation-safe-only` and the bank's arms swapped which one carries it.
  - **Bank:** `docs/tasks/artifacts/engine_universal_services/guard_parses/probe.sh` → `GUARD-PARSES: N/N as declared` (~7 min; the totals, the flip count and the divergence count are all DERIVED by the run). Source grammar `s1_guard_source.ebnf` is **pre-rewrite** — still left recursive, no helper rules, no lookaheads.
  - **What it measures:** two arms over ONE grammar, differing only in the admission. ARM A adds `--indirect-lr-admit-starvation-safe-only` ⇒ `starvation-safe candidates: 0/3`, nothing absorbed, and the generated parser's runtime cycle guard REJECTs the chained casts. ARM B is `make focus_scratch` (shipped) ⇒ `prim` absorbed **and** guarded, and **all seven inputs ACCEPT** — including `k = n;`, the residual-free holder `g4` rejects. **3 GEN rows flip REJECT → ACCEPT**, and the bank FAILS if that count reaches zero. ⛔ **The arms swapped which one carries the flag at `.17` slice 9 and not one expectation moved** — what used to need an opt-in is what `make` now produces, and only the two BANNER rows inverted, because the warning follows the non-shipped policy.
  - ⭐⭐ **It is the only end-to-end exercise of the guarded HOP clone.** Its holder names `ct`, not the base, so `chain: ct > prim` (`max_hops=1`) and PGEN must emit `prim_lr_guard0_ct`; the bank asserts that function is in the COMPILED parser. Every SystemVerilog chain is `max_hops=0`, so no corpus run reaches that rule.
  - ⛔⛔ **The LEVER is not a deliverable path, in whichever direction it points.** `grep -rn "indirect.lr.admit" rust/Makefile scripts/ .github/` returns **0**. The pass prints a warning banner at DEFAULT verbosity when the flag is on — ⛔ and it must be `std::eprintln!`, because `ast_pipeline/mod.rs:513` shadows `eprintln!` with a `pgen_trace_debug!` forwarder, so a bare `eprintln!` anywhere under `ast_pipeline` is invisible unless `PGEN_TRACE_VERBOSITY=debug`. The first draft of that banner was silent for exactly that reason.
  - ⛔ **ARM A is also the smallest `.14` reproducer in the repository**: three rows where `GEN=REJECT INTERP=ACCEPT` on a 20-line grammar, because the generated parser has a runtime cycle guard and the interpreter has none. The bank declares a verdict PER ORACLE rather than failing on disagreement — the only way to measure an un-eliminated arm at all.
- ⛔ **THE CANDIDATE ORDERING CARRIES NO GUARD TERM AND NO SEED TERM.** `indirect_lr_elimination.rs:330-343` sorts by `covered_rules().len()` desc, then `acyclic_alternative_indices.len()` desc, then `clone_cost().len()` asc, then rule-order position — it never reads the guard census. On SystemVerilog the resulting pick (`casting_type`, `max_hops=0`) happens to be the cheaper guard site, which is exactly the condition under which such a coupling ships unnoticed. ⭐ **The model is no longer the blocker — `.17` slice 5 added the seed term (below), so the ordering CAN now be decided against a complete one.** It still is not, and making it so is a parser-behaviour change owed its own slice.
- ⭐⭐ **THE `seed:` LINE IS THE SECOND GUARD POSITION, AND IT IS A DIFFERENT MEASUREMENT FROM `guard:`** (`.17` slice 5). Read both; neither implies the other, and slice 4 measured them closing DISJOINT starvations.
  - `seed_first=` FIRST of the **seed tail** — what the sheared clone chain can consume after its acyclic left corner. ⛔ That is the route suffix **minus the cycle-closing step's residual**, because `clone_rule` DROPS the closing alternative rather than redirecting it, so its residual reaches the `*` and never the seed.
  - `seed_routes=N/M` how many of the candidate's `M` routes contribute such a tail. A route contributes only if the tail is non-empty **and** its clone chain survives the shear — `clone_rule` returns `None` when every alternative of a rule was sheared away. `0/M` means the rewrite introduces **no over-long seed at this base rule**, which is stronger than an empty byte set.
  - per-site `seed=`: `trailing_guard_required` (the seed can eat this holder's residual ⇒ the trailing guard is MANDATORY here) · `no_seed_tail` / `seed_no_competition` / `seed_residual_nullable` (no trailing guard owed) · `seed_undecidable` (a FIRST set is unresolved ⇒ blocks feasibility, exactly as `undecidable` does on the loop).
  - ⛔ There is deliberately **no `seed_incomplete` tier**. The loop guard can cut a chain short at an intermediate iteration; the trailing guard runs once, at rule exit, on a clone reached only from a holder that wants the residual next — so refusing an over-long seed there loses no derivation the holder had.
- ⛔⛔ **AND IT SPLITS SYSTEMVERILOG'S TWO CANDIDATES ON ONE KNOT, WHICH CORRECTS SLICE 4'S OWN SHIPPED ANALOGUE.** Measured: `casting_type` (the driver's actual first pick) `seed_routes=0/10` — **no trailing guard owed**; `constant_primary` `10/10`; `property_expr` `78/80`; `cast` `8/8`. `casting_type`'s routes CLOSE at `cast` / `constant_cast`, each of which has exactly ONE alternative — the cycle-closing one — so the shear leaves no clone and no seed. Ground truth from the REAL eliminator, not the model: the guard dry run's clone set contains `casting_type_lr_seed_constant_primary` and eleven others but **no `casting_type_lr_seed_cast` and no `casting_type_lr_seed_constant_cast`**. ⇒ slice 4's *"`casting_type_lr_base` carries the sheared `constant_cast` clone"* named the wrong base rule; the rule that carries it is `constant_primary`. Slice 4's DECISION (c) — never ship (iii) without the trailing guard — is unaffected and now per-candidate measured: SV **15/28** candidates need it, the wrapper **13/18**, and `ebnf` — the one knot the pass absorbs today — **0/5**.
- ⛔ **A QUANTIFIED GROUP KEEPS ITS PARENTHESES in every rendered residual and suffix** (`.17` slice 2). `( binary_operator … )* ( question … )?` used to print as `binary_operator … * question … ?`, which reads as a MANDATORY leading element — and next to it a `residual_nullable` verdict looks like a tool bug rather than the grammar's own shape. ⛔ **The fix is in `render_elements_display`, NOT in `render_elements`, and the split is load-bearing**: `indirect_lr_elimination.rs:483` decides an ambiguity refusal by comparing `render_elements` output, so parenthesising there would make the pass absorb knots it refuses today. `render_elements` is FROZEN until `.18` gives that check a structural key.
- ⛔ **A TRUNCATED route set REFUSES a guard verdict.** `suffix_first` is a union over routes and every test reads it as an over-approximation; a partial union is an UNDER-approximation, which would make the containment test pass too easily and print a false `FEASIBLE` that nothing downstream could catch. Truncation therefore poisons the summary to `unresolved` ⇒ every site `undecidable` ⇒ candidate BLOCKED. Never fires on a shipped grammar today (widest knot: 80 routes against a 128 budget).
- ⛔⛔ **THE STARVATION CHECK IS TRANSITIVE, and the version that was not shipped a REGRESSION.** A bare-reference alternative (`casting_type := … | constant_primary`) is *consumption*-transparent as well as AST-transparent, so it inherits the base's greed and `cast := casting_type tick lparen expression rparen` starves — `initial k = 8'(1);` went ACCEPT → REJECT while `parameter logic [7:0] K = 8'(1);` still passed. `rules_transparent_to` closes it. ⇒ when reading a `verdict=`, remember a STARVED holder may never name the candidate at all.
- ⛔ **`seeds=0` IS NOT A DISQUALIFICATION.** The direct/wrapper elimination DROPS a left-recursive alternative, so a rule with no acyclic alternative has nothing left to seed from; the indirect transformation CLONES it with the cycle edge sheared, so the seeds come from *under* the cyclic alternative. The `no_acyclic_seed` decline this report used to print was wrong for its own purpose and is gone — it was excluding `constant_primary`, the only rule that DOMINATES SV's cast/call knot (its two clones carry 13 and 14 seeds), which the trial-then-commit check then identified independently.

### 5.5b "Is `generated/` what HEAD's source produces?" — and why `make`'s exit 0 is not the answer

- **WHAT:** `make -C rust SHELL=/bin/bash generated_reproducibility_gate` (doctrine
  `GENERATED-REPRODUCIBILITY`, tier 2) — re-derives all **eleven** artifacts through the tracked recipe and
  demands byte-identity. `bash scripts/check_generated_reproducibility.sh` alone is tier 1 (identity
  only, every commit); `--self-test` fires all **20** refusal arms.
- **WHEN:** after ANY emission-source change, before quoting *"the parser is what HEAD produces"* in
  a release argument, and ⛔ **whenever you have just run a `make` target that was supposed to
  regenerate something**.
- ⛔⛔ **`make` CAN HAND YOU A STALE GENERATOR AT EXIT 0, AND THE LOG WILL NOT SAY SO.** `/usr/bin/make`
  here is **GNU Make 3.81**, which compares mtimes at WHOLE SECONDS, so an emission source rewritten
  in the same second as the last generator build is INVISIBLE and the rule is skipped. Measured
  2026-08-17: the prerequisite was **127.2 ms newer** than `ast_pipeline_bootstrap`,
  `regenerate_generated_parsers` printed `SOTA parser generated:` for all nine artifacts, exited 0,
  and **two of them came out of the PRE-edit emitter**. Generation has been QUIET since
  `CI-PARITY-GATE-ROT.31`, so log volume is no tell either. ⇒ **assert an sha against an arm
  generated by a binary you built directly, never `make`'s exit code.** The `sources → generator
  binary` edge is not covered by `scripts/make_freshness_guard.sh` (`CI-PARITY-GATE-ROT.37`, open).
  ```bash
  rm -f rust/target/debug/ast_pipeline rust/target/debug/ast_pipeline_bootstrap   # the workaround
  make -C rust SHELL=/bin/bash regenerate_generated_parsers
  make -C rust SHELL=/bin/bash generated_reproducibility_gate                     # then PROVE it
  ```
- ⭐ **The gate itself had the mirror-image hole until `ENGINE-UNIVERSAL-SERVICES.32`** and it is
  worth knowing why the fix is shaped as it is: tier 2 re-derived the eight FAMILY artifacts with
  whatever `ast_pipeline` was on disk, so a stale generator produced BOTH sides and byte-identity was
  guaranteed — measured, it printed *"TIER 2 OK — every checked artifact is what HEAD produces"* over
  eight artifacts that were not. It now asks **cargo** (**0.8 s** when the binary is current), and a
  cohort it cannot check reports **TIER 2 PARTIAL** and names it rather than claiming coverage.
- ⚠️ **Read the headline, not the exit code.** `TIER 2 PARTIAL` exits 0 — NOT EVALUATED is not a
  failure — so a caller that keys on `rc` alone cannot tell a full verification from a two-of-ten one.
- ⛔⛔ **AND UNTIL `ENGINE-UNIVERSAL-SERVICES.33` IT MIRRORED THE MAKEFILE'S GENERATOR FLAGS, WHICH IS
  THE PART TO REMEMBER WHEN YOU CHANGE A RECIPE.** Cargo answers *"is this binary current"*; it sees
  the crate, not the recipe — so the flags were a hand-kept copy. Measured before the fix, with a flag
  added to `RUST_GENERATOR` and the artifacts untouched: tier 1 breached (rc 1) → the operator did
  what the message says and ran `--rebaseline` → tier 2 re-derived with the STALE flags, matched, and
  **RECORDED** it (rc 0) → tier 1 went green over a parser `make` would emit at **130 878 616 B**
  against the **143 072 420 B** on disk. ⇒ **a cheap tier that tells you to re-run the oracle inherits
  the oracle's blind spot, and the recording step makes the wrong answer permanent.** The recipe is now
  DERIVED and PRINTED on every tier-2 run — read that line when you are debugging a re-derivation:
  ```text
  generated-reproducibility: recipe DERIVED from rust/Makefile — families: --generate-parser
      --eliminate-left-recursion | annotation pair: --generate-parser --bootstrap-mode --eliminate-left-recursion
  ```
- ⚠️ **`--eliminate-left-recursion` on that line is INERT and will mislead an A/B** (`.34`, open):
  `main.rs:1104` only ever sets a field `PipelineConfig::default()` already sets `true`, and there is
  no negating flag — json / regex / vhdl / systemverilog re-derive **byte-identically** with and
  without it. It cost a slice: a false-pass demonstration built on that flag reproduced a `TIER 2 OK`
  that was **correct behaviour**. If you need a genuinely emission-affecting generator flag for a
  control, `--indirect-lr-admit-starvation-safe-only` moves SystemVerilog by **12 193 804 B**.
- ⚠️ **Eighteen other places in `scripts/` + `rust/scripts/` still spell the recipe by hand, eleven of
  them substituting their parser as a family's SHIPPED one** — census instrument
  `docs/tasks/artifacts/engine_universal_services/es33_makefile_mirror/census.sh`, routed as
  `CI-PARITY-GATE-ROT.38`. All five current differences are the inert flag, so this is *at risk*, not
  wrong today; re-run the census rather than trusting that sentence.

### 5.6 Comparing two GENERATED PARSERS — normalise the embedded `-o` path FIRST

- **WHAT:** not a tool — a **mandatory pre-step** for any comparison of two generated parser artifacts (byte-identity, size delta, `diff`, a codegen-determinism control, an A/B on emitted bytes). Every generated parser writes its own `-o` destination into the emitted source. ⇒ **the artifact's size is a function of its own output path**, and one extra character in the `-o` spelling adds one byte per embedded site.
- ⭐⭐⭐ **THE SITE COUNT IS NOW ZERO — NO GENERATED PARSER EMBEDS ITS OUTPUT PATH AT ALL.** `.31` slice 3 / acceptance (e) corrected what the emitted constant HOLDS: the path was the *wrong label*, because the `Logger` renders it as `[{file}:{line}]` where `line` is `self.position`, an **input byte offset** — so every trace line read `[../generated/json_parser.rs:0]`, a path beside a number that indexes a different file, in the `file:line` shape it violates. The constant now reads `"<grammar> input byte"` (`[systemverilog input byte:113637]`), the path has no consumer left, and the trap this section exists for is **eliminated rather than reduced**. Two `-o` spellings now produce **byte-identical** parsers with nothing to normalise. ⭐ The old value carried no information either: the family is already on the same line twice, as the module path (`pgen::generated_parsers::json::…`) and as the trace component (`[TRACE][generated.json]`). ⛔ **`scripts/compare_generated_parsers.py` is KEPT as a re-introduction TRIPWIRE, not deleted** — `--sites` reports `0` (the correct answer, no longer a refusal) and `GENERATED-REPRODUCIBILITY` breaches on any non-zero. The progression across the three slices: **63 186 → 60 482 → 11 → 0**.
- ⭐⭐ **THE HISTORY, because every number below sits on one of these eras. The count was 1 per artifact after `.31` SLICE 2** (2026-08-17). The path is emitted once, as `const PGEN_SOURCE_LABEL: &str = "<path>";`, and every `Logger::log_*` site references it by name. Before that it was a literal at every site: **36 346** occurrences in the shipped SystemVerilog parser (1 308 456 B, **0.91 %** of the artifact), **33 249** in the narrow-admission arm, **63 186** across the eleven artifacts (**60 482** after `.31` slice 1 deleted the dead class-D bindings). Measured price of the hoist, two arms into a mimic tree: **−1 131 846 bytes (−0.48 %)** across all eleven, of which **−729 426** is SystemVerilog. ⛔ **The trap is REDUCED, not removed** — a 3-character spelling difference now moves an artifact by **3 bytes** where it moved SystemVerilog by **109 038** — so the pre-step below is still mandatory, and the helper still refuses rather than guesses. ⭐ The load-bearing proof that the hoist changed nothing else: ARM 2 with its `const` declaration deleted and every `PGEN_SOURCE_LABEL` token replaced by the literal it declares is **byte-identical to ARM 1 in all eleven artifacts**, so the `file` argument of every diagnostic still receives the same string. Bank: `docs/tasks/artifacts/engine_universal_services/es31_label_hoist/probe.sh` (6 arms, 2 RED-by-design, ~20 min).
- ⭐ **AND THE STATED REASON FOR FORBIDDING `sites × Δlen` DID NOT MATERIALISE, WHICH IS ITSELF A MEASUREMENT.** `.31` acceptance (b) forbade pricing class L by arithmetic because *"`prettyplease` re-wraps lines when a 38-character literal becomes a short identifier"*. Measured: it does not — every log site already occupies its own line, so shortening one argument moves no line break, and the residual in **all eleven** rows is exactly the length of the constant's own declaration line, with nothing left over. The prohibition was still correct: that is a *result*, and it took two arms to learn it. ⇒ price an emission change by generating both arms; quote the arithmetic only as the **check** on a measurement you already took.
- **WHEN:** ⛔ **before you believe ANY byte-level difference between two generated parsers.** If the two arms were not written through a **character-for-character identical `-o` string**, the difference you are reading is at least partly the path. Reproduce the spelling exactly (`rust/Makefile` passes `-o ../generated/<fam>_parser.rs` from `rust/`, 36 chars for SV; an ad-hoc run from the repo root passes `generated/<fam>_parser.rs`, 33 chars), or normalise before comparing.
- **HOW — ⭐ ONE HOME SINCE `ENGINE-UNIVERSAL-SERVICES.25` (c): `scripts/compare_generated_parsers.py`.**
  Do not hand-roll this again; there were three independent copies and the third was written after
  the first defect was recorded.
  ```bash
  python3 scripts/compare_generated_parsers.py --compare armA.rs armB.rs   # rc 0 same / 1 differ / 2 refuse
  python3 scripts/compare_generated_parsers.py --sites    generated/systemverilog_parser.rs   # 0 (was 36346, then 1)
  python3 scripts/compare_generated_parsers.py --spelling generated/systemverilog_parser.rs
  python3 scripts/compare_generated_parsers.py --describe armA.rs           # every derived figure, JSON
  python3 scripts/compare_generated_parsers.py --self-test                  # 8/8, four RED-by-design
  ```
- ⛔⛔ **NEVER TELL A COMPARISON HELPER WHICH PATH TO NORMALISE — IT MUST READ IT OUT OF THE
  ARTIFACT, AND THE REASON IS A SUBSTRING.** `generated/x_parser.rs` (33 chars, the ad-hoc spelling)
  is a SUBSTRING of `../generated/x_parser.rs` (36 chars, what `rust/Makefile` passes). Measured on
  the shipped SV parser, normalising with the short spelling leaves **36 346 `"../<TOKEN>"` residues
  and 0 normalised sites** — it reports success and normalises nothing, and two arms treated that way
  still differ by 3 bytes per site, which reads as *"something other than the path moved"*. The
  helper derives the spelling instead (every generated parser holds **exactly one** distinct
  `"….rs"` literal, 11/11 exact against `grep -oF`) and REFUSES with exit 2 on zero or on ambiguity.
  ⚠️ One further silent-zero variant it removes: a helper that uses its FILE ARGUMENT as the
  spelling reports `path_sites=0` with unnormalised bytes the moment an artifact is read from
  anywhere but its generation path.
  Full record → [[derive-the-comparison-key-from-the-artifact-not-from-the-caller]].
- ⛔ **AND A SEPARATE HAZARD, WHICH IS NOT THIS ONE: A CARRIED COPY OF A DERIVED COUNT.**
  `run_guard_ab_structural.sh`'s block comment published the SV site count as **43 615** for four
  sessions; it is **36 346**, and the script's own tracked output
  (`docs/tasks/artifacts/engine_universal_services/guard_ab_structural.txt`) recorded
  `path_sites=33249 / 36346 / 36291` in the same commit. ⛔⛔ **`.25` slice 1 first published a
  mechanism for that number — *"36 346 × 42 ÷ 35, the right delta over the wrong width"* — and it is
  RETRACTED**: the instrument never mis-computed, and the arithmetic was fitted (the identity needs
  a ratio of exactly `6/5`, and the real arm-2 numerator implies a non-integer denominator). The
  class is **instrument right, prose copy wrong** (`DERIVED_STATE_CONTAINMENT.md` R1/R3, as in
  `CI-PARITY-GATE-ROT.36`) — a *carried* number, not a mis-derived one. ⇒ derive the count with the
  helper above rather than typing it into prose beside the run that already printed it.
- ⚠️ **CHARACTERS ARE NOT BYTES HERE, AND BOTH ARE PUBLISHED SOMEWHERE.** These artifacts carry emoji
  in their trace strings: `generated/systemverilog_parser.rs` is **143 909 594 bytes** and
  **143 801 151 characters** — a 108 443 gap. `wc -c`/`stat` report bytes; Python `len(str)` reports
  characters, which is the basis `.20` slice 3's three-arm table was published on. The helper reports
  `raw_chars`/`raw_bytes` and `normalised_chars`/`normalised_bytes` under separate names, and its
  `--token` argument exists so a caller comparing against an already-published normalised figure can
  ask for the token that figure was produced with instead of silently re-basing it.
- **Doing it by hand anyway** (the helper is strictly better — this is only for reading its guts):
  ```bash
  # count the sites (occurrences, NOT `grep -c` lines, and -F so `.` is not a wildcard)
  grep -oF '../generated/systemverilog_parser.rs' generated/systemverilog_parser.rs | wc -l

  # normalise arm A's spelling to arm B's, then compare — identity, not arithmetic
  LC_ALL=C sed 's|\.\./generated/systemverilog_parser\.rs|generated/systemverilog_parser.rs|g' \
      armA.rs > armA_normalised.rs
  shasum -a 256 armA_normalised.rs armB.rs
  ```
- **OUTPUT:** equal sha256 ⇒ the path was the ONLY difference. ⭐ Demand **identity**, never a matching byte COUNT: a count that matches is equally consistent with *"the path explains every byte"* and *"the path explains N bytes and something else nets to zero"* — that is an illustration, not a test (`docs/CLAIM_VERIFICATION.md` §3 leg 2). The reference instrument, with a RED control proving the comparison can fail, is `docs/tasks/artifacts/engine_universal_services/es19_path_embedding/probe.sh` (6 arms, ~1 m 44 s, and it never touches the shipped artifact — it generates into a mimic `<work>/root/{generated,rust}` tree so both `-o` strings are byte-identical to the real invocations).
- ⛔⛔ **THIS TRAP HAS INVERTED THREE PUBLISHED READINGS, AND ONE OF THEM FOUNDED A TASK LEAF** (owner: `ENGINE-UNIVERSAL-SERVICES.25`). ⚠️ **`.25` slice 1 reported a FOURTH instance inside the script that carries the warning, and that count is CORRECTED to three under a director challenge.** `run_guard_ab_structural.sh` did publish **43 615** where the true value is **36 346** — but its own tracked output printed 36 346 correctly in the same commit, so nothing was ever *mis-normalised* there. It is a carried prose copy (`DERIVED_STATE_CONTAINMENT.md` R1/R3), a different and separately-listed hazard, **not** a fourth inversion of this trap. ⇒ writing the warning down a third time was not what fixed the trap; deleting the parameter that carries the mistake was. (1) `CI-PARITY-GATE-ROT.32`(d)'s census identity control reported **10/10 families mismatching** their shipped artifacts — read as a codegen-determinism emergency; the census simply wrote to a different path. (2) `.20` slice 3's three-arm table, read from raw `stat` bytes, made the guard-suppressed arm look **203 KB LARGER** than the shipped parser, i.e. *guards make the parser smaller* — the exact opposite of the truth. (3) `.19` was **opened as a leaf** because a re-derivation came out **99 747 bytes** from the tracked artifact, with two hypotheses (the flip's diff moved codegen / the git-ignored input JSON moved) that were **both refuted**: the two arms differ by 3 path characters × 33 249 sites = 99 747, exactly, and normalising makes them sha256-identical. ⛔ The sharpest part of (3) is the TIMING — the mechanism was already known when the leaf was written up, having been found by this same tree in session #238 while `.19` was opened in #232, so wrong hypotheses sat in the queue for three sessions after the answer existed.
- ⚠️ **The input path is NOT embedded, and that was tested rather than assumed** — generating with the input JSON named absolutely and relatively yields a byte-identical parser, and `grep -c 'systemverilog\.json'` over the emitted parser is **0**, as is any `generated_at`/date string. The `-o` path is the only provenance a generated parser carries, which is exactly why it is the only thing to normalise.

---

### 5.7 "WHICH grammar commit changed what a CONSUMER sees?" — the accept-set / AST-shape transition ledger
- **WHAT:** `docs/tasks/artifacts/sv_corpus_grad/contract_accept_set_ledger/measure_accept_set_transitions.py` — replays every pinned reproducer in `stimuli/sv/adjudication_repros/MANIFEST.tsv`, on every profile it declares, against **every** revision of `grammars/systemverilog.ebnf` since the downstream contract was last written (the base is DERIVED with `git log -1 -- <contract>`, never typed), and attributes each transition to the commit that caused it. Reports **two axes**: `ACCEPT-SET` (the verdict moved — `WIDEN`/`NARROW`) and `AST-SHAPE` (the verdict did NOT move and the typed AST did).
- **WHEN:** before writing a contract release section or a bug-ledger row; when asking *"is this grammar commit consumer-visible, and in which direction?"*; when deciding the **schema** question, since the contract's trigger is a REPLACED shape rather than an added one.
- ⛔ **WHY THE SECOND AXIS EXISTS, measured:** the first cut reported `PGEN-SV-CORPUS-GRAD-0233` as *"no verdict moved (comment-only)"*. `bufif0 g(o,i,e);` parsed before that fix and parses after; what changed is that it became a `gate_instantiation` instead of a `udp_instantiation`. **Twelve pinned witnesses moved and zero verdicts did.** A verdict-only ledger is blind to the largest class of consumer break there is — the same lesson the manifest's `arm` column exists for (§1.2, `.13c.2a.2`).
- **HOW:**
  ```bash
  python3 docs/tasks/artifacts/sv_corpus_grad/contract_accept_set_ledger/measure_accept_set_transitions.py
  # ~7 min at PGEN_ACCEPT_SET_JOBS=8; writes accept_set_transitions.tsv beside itself
  ```
- **OUTPUT:**
  ```text
  ACCEPT-SET-LEDGER: base=438c475c candidates=9 repro_checks=151 jobs=8
  ACCEPT-SET-LEDGER: interpreter agrees with the shipped parser on 151/151 verdicts at HEAD — historical arms are quotable
    f9cff55e  PGEN-SV-CORPUS-GRAD-0233      widen=0   narrow=0   shape=12  CONSUMER-VISIBLE CHANGE
  ACCEPT-SET-LEDGER: consumer_visible_commits=7/9 (of which move an AST SHAPE: 4) transitions=71
  ```
- ⭐ **It falsifies its own oracle before quoting it.** Historical grammars are measured on the INTERPRETER (§1.5b), because the shipped parser would need a ~22-minute regeneration per commit. So at HEAD every row is *also* run through the shipped release probe and any disagreement is a hard error — the interpreter is trusted for the arms we cannot see only because it is observed to match on the one we can.
- ⚠️ **HONEST BOUND, printed rather than implied:** the input population is the PINNED MANIFEST, not the language. Measured: `-0220` widened the accept set and moved nothing here, because no row pinned its witness (now pinned). Read `shape=0 widen=0 narrow=0` as *"no PINNED witness moved"*, never as *"nothing changed"*.
- ⭐ **The companion instrument has no such bound and is one command:** the grammar's **semantic digest** — sha256 of the EBNF frontend's own `raw_ast` envelope, i.e. what the code generator consumes, so comments cannot move it. `scripts/check_sv_contract_currency.sh` re-derives it and refuses a tree where the grammar moved and the contract did not. The two classifiers share no parent and agreed 9/9 on the population; that agreement is the evidence, not either one alone.
  ```bash
  ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --emit-raw-ast-json raw.json   # ⛔ a FILE, never /dev/stdout
  python3 -c "import json,hashlib;print(hashlib.sha256(json.dumps(json.load(open('raw.json'))['raw_ast'],sort_keys=True,separators=(',',':')).encode()).hexdigest())"
  bash scripts/check_sv_contract_currency.sh   # SV-CONTRACT-CURRENCY: rows=10 genesis=438c475c newest=… digest=…
  ```

### 5.8 "Does the LRM define a production the SHIPPED grammar has never seen?" — the Annex A gap census

- **WHAT:** `python3 stimuli/sv/lrm_annex_a_gap_census.py` — enumerates every IEEE 1800 production
  that the standard defines in a **clause body** and that **Annex A does not carry**, for both
  tracked editions, and classifies each one. `grammars/systemverilog.ebnf` descends from Annex A
  alone (`tools/extract_systemverilog_lrm_profiles.py`), so that population is a normative surface
  the extraction pipeline is blind to by construction.
- **WHEN:** *"is this construct even modelled?"*, *"where else could a rejects-valid defect be
  hiding that no corpus row has hit yet?"*, or before claiming a family is LRM-complete. ⛔ It is a
  **worklist generator, not a defect list** — most rows are outside the source language.
- **HOW:**
  ```bash
  python3 stimuli/sv/lrm_annex_a_gap_census.py            # gate mode: recompute + fail on drift
  python3 stimuli/sv/lrm_annex_a_gap_census.py --write     # refresh the tracked artifact
  bash docs/tasks/artifacts/sv_corpus_grad/lrm_annex_a_gap/worklist_probes/probe_worklist.sh
  ```
- **OUTPUT:**
  ```text
  LRM-ANNEX-A-GAP: clause_only=294 ieee_labelled_not_in_annex_a=293 sv_source_worklist=14 editions=2017,2023
  ```
  plus `docs/tasks/artifacts/sv_corpus_grad/lrm_annex_a_gap/{census.tsv,census.md}`.
- ⭐⭐ **THE SIGNAL IS IEEE'S OWN, NOT AN INFERENCE.** Every clause syntax box carries a caption that
  states its provenance — `Syntax 10-5—Assignment patterns syntax (excerpt from Annex A)` versus
  `Syntax 18-11—Scope randomize function syntax (not in Annex A)`. The census reads that caption.
  Measured in the 2017 edition: **304** boxes say *excerpt*, **60** say *not in*. Where IEEE writes
  the marker as a `// not in Annex A` comment INSIDE the box instead, the in-box marker wins.
- ⭐ **BOTH SIDES ARE SCRAPED WITH THE EXTRACTOR'S OWN `extract_rules`**, deliberately — a
  divergence this reports cannot be an artifact of a second, differently-buggy scraper.
- ⛔ **IT REFUSES RATHER THAN GUESSING.** A clause-only production with no caption, or a caption
  whose clause number has no declared bucket, exits **1**. On its first run that caught Annex F's
  formal-semantics metavariables (`P ::= strong ( R )`); they are now a declared bucket guarded by a
  proof that Annex A names no single-uppercase-letter production.
- ⛔⛔ **DO NOT KEY ANYTHING HERE ON FILE NAMES.** The PDF→markdown splitter titled sections from
  nearby lines: the 2017 Annex A holds **0** productions in the file named for it and **736** in
  `section-41-data-read-api.md`, and the Annex F metavariables are filed under
  `section-100-time-units.md`. The census keys on the clause number IEEE prints in the caption.
- **FIRST CUSTOMER** (`SV-CORPUS-GRAD.13c.2v` slice 1, ledger `SV-0065`): of the seven
  `sv_source_syntax` rows per edition, six were already covered by a general Annex A rule and
  `scope_randomize` (Syntax 18-11) was **unreachable** — `std::randomize(a, b) with { a < b; }`
  REJECTED in every expression.

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

### 6.4 `PGEN_REACH_FORCED_OVERRIDE_DUMP` — the reach plan forced a branch; did it actually RENDER?

- **WHAT:** `PGEN_REACH_FORCED_OVERRIDE_DUMP=1` makes `generate_or`'s documented
  forced-first-**with-fallback** visible. When a reach-plan-forced branch fails, the OR silently
  renders a sibling and returns `Ok`, so the directive is lost with **no trace at any verbosity**.
  This flag emits two paired stderr records — `outcome=failed reason=…` (the generator's own error
  string for the forced branch, the same text `record_branch_failure` files under `failure_reasons`)
  and `outcome=overridden rendered_branch=M` (what got substituted). Presence-gated print only, read
  once per process; generation stays byte-identical. `ENGINE-UNIVERSAL-SERVICES.11` slice 1.
- **WHEN:** ⛔ **the FIRST thing to run when a probe "did not witness" and you cannot tell whether it
  was ever actually driven down the intended branch** — a `witnessed_target=false` verdict, a
  `parsed-but-routed-elsewhere` residual, or any *"the plan says it forces branch N, so why does the
  sample show branch 0?"* question. ⭐ It is the one that answers WHY on the cert-coverage path,
  where **6.1 cannot**: `--report-certificate-coverage` returns at `rust/src/main.rs:1149` and
  `--coverage-output` is refused alongside it (it `require`s `--generate-stimuli`), so the
  per-branch `failure_reasons` a cert run writes are discarded at process exit. 6.1 stays the tool
  for the closed-loop replay gap report; this is the tool for the witness passes.
- **HOW:**
  ```bash
  PGEN_REACH_FORCED_OVERRIDE_DUMP=1 PGEN_CERT_COVERAGE_DEBUG_PROBES=1 \
    ./rust/target/debug/ast_pipeline grammars/systemverilog.ebnf --report-certificate-coverage \
    --grammar-profile sv_2017 --entry-rule systemverilog_file --count 40 --seed 0 2>&1 \
    | grep "forced-override" | grep "<your residual rule>"
  ```
  Pairs naturally with `PGEN_REACH_PATH_DUMP=1` (4.4): that one prints the plan, this one prints the
  RENDER — the pairing lesson `.10` paid for is that the two need different instruments.
- **OUTPUT (the real one that closed `.11`'s diagnosis, on its first run):**
  ```
  [forced-override] rule='select_expression_lr_suffix' path='root' forced_branch=2/3 outcome=failed
      reason="Stimuli generation depth exceeded max_depth=67 while expanding rule 'real_number'"
  [forced-override] rule='select_expression_lr_suffix' path='root' forced_branch=2/3
      outcome=overridden rendered_branch=0
  ```
  `forced_branch=2/3` = branch index 2 of 3 alternatives. 1 784 override records across that whole
  SV run — so **scope it with `grep`**, exactly like `--trace-rules` (2.2).
- **READING:** an `outcome=failed` line is the directive being LOST; the reason string names the
  mechanism verbatim (`depth exceeded max_depth=N while expanding rule 'R'` = budget,
  `target timeout`/`helper timeout` = time, anything else = a prune). An `outcome=overridden` with no
  preceding `outcome=failed` for the same site means the forced branch was never *attempted* — check
  `suppress_recursive_forced_branch` and the `bypass_fuel` re-admission instead.
- ⚠️ **The reason is only half the answer — the OTHER half is whether the budget is scoped right.**
  A `depth exceeded max_depth=N` prints identically whether the budget is genuinely too small or
  merely measured against the wrong thing, so **read `N`'s provenance, not just that it was hit**.
  `.11` measured `max_depth=67` = `reach_prefix_budget + min_derivation_depths[rule]`, i.e. the
  rule's SHALLOWEST alternative, on a pass that was at that moment forcing a much deeper one. ⭐ That
  instance is CLOSED — the target-own tier now budgets the alternative it forces
  (`min_full_derivation_depth_of_node(alt) + 1`, `witness_target_depth_budget`'s formula), and SV's
  union basis reached `UNKNOWN=0`. The mandatory-CHILD and seed-SIBLING tiers, and
  `generate_structured_witnesses`' own branch loop, still use the flat rule-scoped budget — the same
  under-funding is possible there and no case has been observed, so that is where to look next.
  ⛔ Do NOT reach for a bigger global `--max-depth`: measured on SV it buys `UNKNOWN 1→0` by paying
  `sample_parse_failures 0→8→17` — witness samples the real parser then rejects. Full map:
  `docs/tasks/ENGINE-UNIVERSAL-SERVICES.md` `.11`.

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
3. ⛔ **"Did the whole SV parser get slower?" is a different question, and it has a standing
   instrument** — see 3.7. Do NOT answer it with an ad-hoc timing script against a remembered
   number: this repository did that twice on one change and got `~11 %` and then `+24.3 %`, and
   **both were machine-variance artifacts** (`ENGINE-UNIVERSAL-SERVICES.20` slice 5 / `.26`).
4. ⛔ **"WHERE does the time go?" needs a SAMPLER, not a counter** — 3.8. Every tool in 3.1–3.6
   routes the parse to the PROTOCOL graph, so none of them observes the fused `cascade_*` code a
   production parse runs. Measured consequence: the SV LR machinery is **1.9 % of self-time and
   27.3 % of inclusive time**, and no counter can distinguish those.

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
