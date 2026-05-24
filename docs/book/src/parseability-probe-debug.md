# Debugging With `parseability_probe`

`parseability_probe` is pgen's primary diagnostic CLI for any parser the engine generates. This chapter is the **complete reference** for its debug surfaces — every flag, every environment variable, every output marker.

If a parser of yours rejects valid input, hangs, or behaves unexpectedly, **start here**. The debug features described below are designed to make every parser self-explain its behavior — no source-spelunking required.

---

## Table of Contents

- [Quick Reference Card](#quick-reference-card)
- [Basic Usage](#basic-usage)
- [Trace Verbosity Levels](#trace-verbosity-levels)
- [Rule-Scoped Trace (`--trace-rules`)](#rule-scoped-trace---trace-rules)
- [Trace Output Format](#trace-output-format)
- [Per-Rule Call-Count Dashboard (`--dump-rule-call-counts`)](#per-rule-call-count-dashboard---dump-rule-call-counts)
- [Excluding Noisy Rules from the Dashboard](#excluding-noisy-rules-from-the-dashboard)
- [Furthest-Position Error Diagnostic](#furthest-position-error-diagnostic)
- [Predicate Self-Explaining Trace](#predicate-self-explaining-trace)
- [Workflow Recipes](#workflow-recipes)
- [Library Plumbing (`--lib-in`, `--lib-out`)](#library-plumbing---lib-in---lib-out)
- [Environment Variables](#environment-variables)
- [Cost / Performance](#cost--performance)

---

## Quick Reference Card

| Flag | Purpose | Section |
|---|---|---|
| `--supports <grammar>` | Check whether a grammar is registered | [Basic](#basic-usage) |
| `--parse <grammar> <file>` | Parse a file, report success/failure | [Basic](#basic-usage) |
| `--parse-dump-ast <grammar> <file>` | Parse + dump AST as JSON | [Basic](#basic-usage) |
| `--parse-dump-ast-pretty <grammar> <file>` | Parse + dump AST as pretty JSON | [Basic](#basic-usage) |
| `--profile <name>` | Select a grammar profile (e.g. SV `2017` vs `2023`) | [Basic](#basic-usage) |
| `--trace` | Enable parser trace at maximum verbosity (DBG) | [Trace](#trace-verbosity-levels) |
| `--trace-rules R1,R2,...` | Scope the trace to specific rules' call-trees | [Rule-scoped trace](#rule-scoped-trace---trace-rules) |
| `--trace-log-file [FILE]` | Write trace to a file (defaults to `trace.log`) | [Trace](#trace-output-format) |
| `--dump-rule-call-counts [N]` | Live top-N per-rule call dashboard | [Dashboard](#per-rule-call-count-dashboard---dump-rule-call-counts) |
| `--dump-rule-call-counts-exclude R1,R2,...` | Hide noisy rules from dashboard | [Exclusion](#excluding-noisy-rules-from-the-dashboard) |
| `--max-bytes N` | Bound AST dump size | [Basic](#basic-usage) |
| `--lib-in DIR` | Read `@import_from_library` artifacts from `DIR` | [Library](#library-plumbing---lib-in---lib-out) |
| `--lib-out DIR` | Write `@export_to_library` artifacts to `DIR` | [Library](#library-plumbing---lib-in---lib-out) |

Environment fallbacks:

| Variable | Purpose |
|---|---|
| `PGEN_TRACE_VERBOSITY` | Set trace level (`none`/`low`/`medium`/`high`/`debug`) |
| `PGEN_PARSE_DUMP_AST_MAX_BYTES` | Default for `--max-bytes` |

---

## Basic Usage

### Check grammar support

```bash
parseability_probe --supports systemverilog
```

Exits `0` if the named grammar is registered, non-zero otherwise.

### Parse a file (success/failure only)

```bash
parseability_probe --parse systemverilog path/to/file.sv
```

Exits `0` on a fully-consuming parse, `1` on any parse rejection. The error message (printed to stderr on failure) includes both the **surface position** (where the outermost failing rule started) and the **furthest position** the parser actually reached — see [Furthest-Position Error Diagnostic](#furthest-position-error-diagnostic) for why both matter.

### Parse a file + dump the AST

```bash
parseability_probe --parse-dump-ast systemverilog path/to/file.sv          # JSON
parseability_probe --parse-dump-ast-pretty systemverilog path/to/file.sv   # pretty JSON
```

Default output file is `<grammar>_ast.json` in the current directory. Pass an explicit second positional arg to override.

### Select a grammar profile

```bash
parseability_probe --parse systemverilog file.sv --profile 2017
parseability_probe --parse systemverilog file.sv --profile 2023
```

For SystemVerilog, the recognized profile names are `2017`, `ieee1800-2017`, `ieee_1800_2017`, `2023`, `ieee1800-2023`, `ieee_1800_2023`. Other grammars may define their own profile names.

---

## Trace Verbosity Levels

The trace system has **five additive levels**. Each level adds STRICTLY NEW content on top of all lower levels — there is no duplication. The audit in `.b.6.2.18` empirically verified strict additivity (every tagged-line count at a lower level is preserved exactly at all higher levels).

| Level | Marker | What it adds | Typical use |
|---|---|---|---|
| `none` | (no output) | Nothing. Default. | Production / no diagnostics |
| `low` | `[PGEN][LOW] 🧭` | Errors + warnings: rule-exit-with-error, regex no-match, backtrack events | "Did anything go wrong?" |
| `medium` | `[PGEN][MED] 🧩` | Successful rule exits: `Exiting rule X successfully - advanced from N to M` | "What did the parser succeed at?" |
| `high` | `[PGEN][HIGH] 🔎` | Branch dispatch + **predicate verdicts**: which alternation branches were tried + which `@predicate` accepted/rejected | "Which alternative won?" / "Why did a predicate reject?" |
| `debug` | `[PGEN][DBG] 🧠` | Speculation boundaries + **predicate mechanism**: resolved arg values, fact-lookup detail | Deepest diagnostic — when HIGH isn't enough |

### How to set the level

Three precedence layers (highest precedence first):

1. **CLI**: `--trace` always implies `debug`. There is no `--trace=LEVEL` form on `parseability_probe` directly — use the env var for finer control.
2. **Env var**: `PGEN_TRACE_VERBOSITY={none|low|medium|high|debug}` (also accepts `off`/`0` for none, `1`/`2`/`3`/`4` for low/medium/high/debug, and `trace` as alias for debug).
3. **Default**: `none` (no trace).

```bash
# Maximum verbosity:
parseability_probe --parse systemverilog f.sv --trace

# Medium verbosity (success exits + errors only):
PGEN_TRACE_VERBOSITY=medium parseability_probe --parse systemverilog f.sv --trace

# Suppress everything except errors:
PGEN_TRACE_VERBOSITY=low parseability_probe --parse systemverilog f.sv --trace
```

> **Volume warning.** Full `--trace` on a non-trivial input produces hundreds of MB of output. Use [`--trace-rules`](#rule-scoped-trace---trace-rules) to scope to specific rules' call-trees instead — 100-1000× volume reduction.

---

## Rule-Scoped Trace (`--trace-rules`)

When you already know (or strongly suspect) which rule is misbehaving, scope the trace to ONLY that rule's call-tree:

```bash
parseability_probe --parse systemverilog file.sv \
  --trace-rules expression,statement
```

**Semantics:** trace activates the moment the parser enters any rule listed in `--trace-rules`, and stays active until that rule returns. Rules called from within (sub-parses, recursion) are also traced. Outside the scope: completely silent.

**Implies `--trace`** (rule-scoped trace would be meaningless without the logger enabled).

**Empirical volume reduction:** small SV probe with full `--trace` produced 145,199 lines; the same probe with `--trace-rules class_scope_type` produced 6,876 lines — **21× reduction** on a tiny input. On uvm_pkg-scale inputs, 100-1000× is realistic.

### Picking the rule list

The [call-count dashboard](#per-rule-call-count-dashboard---dump-rule-call-counts) is the natural way to identify which rules to scope on:

```bash
# Step 1: find which rules dominate
parseability_probe --parse systemverilog file.sv --dump-rule-call-counts 20

# Step 2: scope trace to the suspect(s)
parseability_probe --parse systemverilog file.sv \
  --trace-rules <suspect-rule-from-dashboard> \
  --trace-log-file dbg.log
```

---

## Trace Output Format

Every trace line follows this structure:

```
[PGEN][<LEVEL>] <emoji> [<file>:<position>] [<function>] <icon> <message>
```

Field-by-field:

| Field | Meaning |
|---|---|
| `[PGEN][<LEVEL>]` | The verbosity level emitting this line (`LOW`, `MED`, `HIGH`, `DBG`). |
| `<emoji>` | Per-level marker: 🧭 LOW, 🧩 MED, 🔎 HIGH, 🧠 DBG. |
| `[<file>:<position>]` | The generated parser file + **the parser's input byte position at emit time**. NOT a source-file line number. Use this to find where in your input the parser was. |
| `[<function>]` | The actual generated parser function that emitted the trace (a rule's `parse_<rule_name>` or a logger trait impl). |
| `<icon>` | Per-event marker: 🚪 entering branch, ✅ success exit, ❌ failure/backtrack, 🚫 predicate rejection, 💾 memoization, 💥 recursion guard, 🛡️ predicate verdict, 🎯 transform applied. |
| `<message>` | The event-specific payload (rule name, position, branch number, etc.). |

**Important:** the `[<file>:<position>]` value is the **parser's INPUT POSITION**, not a source-file line number. The format reads like `[parser.rs:1247]` but `1247` means "the parser was at input byte 1247 when this fired." This is far more navigationally useful than a generated-source line.

### Writing trace to a file

```bash
parseability_probe --parse systemverilog file.sv --trace --trace-log-file dbg.log
# Or with default filename "trace.log":
parseability_probe --parse systemverilog file.sv --trace --trace-log-file
```

Trace files contain ANSI escape codes. View with `less -R` or strip them with `sed 's/\x1b\[[0-9;?]*[a-zA-Z]//g'`.

---

## Per-Rule Call-Count Dashboard (`--dump-rule-call-counts`)

A live dashboard showing the top-N rules by call count. Refreshes every 250ms in-place using ANSI cursor control — only the count numbers update, line positions stay stable.

```bash
parseability_probe --parse systemverilog file.sv --dump-rule-call-counts        # default: top-20
parseability_probe --parse systemverilog file.sv --dump-rule-call-counts 50     # top-50
parseability_probe --parse systemverilog file.sv --dump-rule-call-counts 10     # top-10
```

Live display example:

```
=== Rule call counts (live, top 10) ===
  expression                              42,891,234
  identifier                              28,567,123
  primary                                 19,234,891
  hierarchical_identifier                 12,456,789
  statement                                8,891,234
  declaration                              7,234,567
  ...
                                              [updates in place every 250ms]
```

### When to use it

- **Stuck parse** — you don't know which rules are dominating; this tells you in real time.
- **Slow parse** — identify the cost centers.
- **Suspect rule** — confirm a hypothesis ("is the recursion in `expression`?").
- **Timeout case** — the dashboard keeps refreshing right up until the parse is killed by SIGTERM, so you see the latest snapshot before the timeout fires.

### Cost

The per-rule counter is **always-on** (incremented on every rule entry, ~1 nanosecond per `fetch_add` — lock-free). Enabling the dashboard only spawns the display thread; the counter cost is the same with or without the flag.

> Counter increment cost is currently in **release builds too**. Deferred per project policy until current parsers are released — see Task `.b.6.2.23`.

---

## Excluding Noisy Rules from the Dashboard

SystemVerilog has ~1500 rules. Many low-level rules (`trivia`, `identifier`, `lparen`, etc.) are called orders of magnitude more than the rules you actually care about. Filter them out:

```bash
parseability_probe --parse systemverilog file.sv \
  --dump-rule-call-counts 20 \
  --dump-rule-call-counts-exclude "trivia,identifier,lparen,rparen,lbrack,rbrack,semicolon,comma,dot"
```

**Semantics:** the exclusion filter applies BEFORE the top-N selection, so the user-visible top-N has N diagnostically-useful rows instead of being dominated by noise.

**Effect:** on a small SV probe, default top-5 shows:
```
trivia                          1,493
non_typedef_package_scope         156
declaration_identifier            126
attribute_instance                125
identifier                        121
```

With `--dump-rule-call-counts-exclude "trivia,lparen,lbrack,dot,at_sign,attribute_instance"`, top-5 becomes:
```
non_typedef_package_scope         156
declaration_identifier            126
identifier                        121
kw_typedef                         72
lbrace                             70
```

— exactly the diagnostically interesting rules.

### Dynamic ranking

Counters are monotone (only grow), but their **rank changes over time**. Early in a parse, structural rules like `white_space`/`identifier` dominate; mid-parse may shift to `expression`/`statement`; late may shift to type-resolution rules. The dashboard re-snapshots and re-sorts every 250ms, so a previously-top rule can drop out of the top-N as the parse moves through different grammar regions. Empty display slots are blanked, not stale.

---

## Furthest-Position Error Diagnostic

When a parse fails, the standard PEG error message reports the position where the OUTERMOST failing rule started — which is often megabytes shallower than the actual defective construct. pgen's `furthest_position` tracking augments every error message with the deepest byte position any branch reached during the parse (even on backtracked branches).

**Example error message:**
```
Parser did not consume full input at position 113637
  [furthest_position=643297, +529660 bytes deeper than surface position]
```

Field meanings:

| Field | Meaning |
|---|---|
| `position 113637` | Where the outermost failing rule started — often uninformative. |
| `furthest_position=643297` | The deepest byte any branch reached. The actual defective construct lives at or near this position. |
| `+529660 bytes deeper` | How much deeper the real failure is than the surface position. Large delta = "the real bug is far from where the error appears." |

**To find what's at `furthest_position`:**

```bash
# Get the byte offset from the error, then map to a line:
FURTHEST=643297
head -c "$FURTHEST" file.sv | wc -l
# Now sed to that line +/- a few:
sed -n '19880,19895p' file.sv
```

### When this matters most

For PEG grammars, "did not consume full input at position N" can be deeply misleading. Without furthest-position, finding the real bug typically requires ~10-15 bisection iterations on the failing input. With it, the right line is one diagnostic run away.

**Always on** — no flag required. The cost is one `max` update per rule entry (~1-2 nanoseconds, no atomics, monotone — never restored on speculation).

---

## Predicate Self-Explaining Trace

Semantic-annotation predicates (`@predicate { name: ..., args: [...], phase: ... }`) can reject branches at parse time. When this happens, the trace explains the verdict clearly.

At `--trace high` (or `--trace`), every branch-predicate evaluation emits a one-line verdict:

```
🛡️ predicate 'lacks_fact_attribute_equals' PASSED branch 2/3 of rule 'scoped_or_hierarchical_tf_identifier'
🛡️ predicate 'has_fact' REJECTED branch 1/3 of rule 'method_call_initial'
🛡️ predicate 'X' REJECTED branch K/N — reason: at least one $reference could not be resolved (unresolved args: [...])
🛡️ predicate 'X' INAPPLICABLE (treated as no-op) branch K/N
```

The four possible verdicts:

| Verdict | Meaning |
|---|---|
| `PASSED` | Predicate returned `Some(true)`; branch is committed. |
| `REJECTED` | Predicate returned `Some(false)`; branch is discarded. |
| `INAPPLICABLE (treated as no-op)` | Predicate returned `None`; branch proceeds as if no predicate were attached. |
| `REJECTED ... reason: at least one $reference could not be resolved` | The predicate's arguments couldn't be resolved against the current parse content (typically a `$path.to.field` reference that didn't match the produced shape). Treated as rejection. |

At `--trace debug`, additional context lines show:

```
   ↪ resolved spec: lacks_fact_attribute_equals [String("type_name"), String("uvm_pkg"), String("declaration_family"), String("typedef")] | phase: Branch | view: Shaped
```

### When this matters

If a parser rejects valid input and you suspect a `@predicate` is the cause, run with `--trace high` (rule-scoped via `--trace-rules <suspect>` to keep volume down). The trace will tell you exactly which predicate rejected which branch, with what resolved arguments. No source-spelunking needed.

---

## Workflow Recipes

### Recipe 1: Parser hangs or runs slowly

```bash
# Step 1: identify which rules dominate
timeout 30 parseability_probe --parse <grammar> file.sv \
  --dump-rule-call-counts 30 \
  --dump-rule-call-counts-exclude "trivia,identifier,lparen,rparen,semicolon,comma,dot"

# Step 2: dashboard tells you the top-N rules. Pick suspects, drill in:
timeout 30 parseability_probe --parse <grammar> file.sv \
  --trace-rules <suspect-rule-1>,<suspect-rule-2> \
  --trace --trace-log-file dbg.log

# Step 3: inspect dbg.log for the actual call pattern
sed 's/\x1b\[[0-9;?]*[a-zA-Z]//g' dbg.log | grep -E "🚪|❌|✅" | head -100
```

### Recipe 2: Parser rejects valid input ("did not consume full input")

```bash
# Step 1: get both positions from the error message:
parseability_probe --parse <grammar> file.sv 2>&1 | tail -1
# Example output:
#   Parser did not consume full input at position 113637
#     [furthest_position=643297, +529660 bytes deeper than surface position]

# Step 2: map furthest_position to a line:
FURTHEST=643297
LINE=$(head -c "$FURTHEST" file.sv | wc -l)
echo "Defect locus: line $LINE"
sed -n "$((LINE - 3)),$((LINE + 3))p" file.sv

# Step 3: minimal repro the syntax you see and feed it through:
echo "the_minimal_repro" > /tmp/repro.sv
parseability_probe --parse <grammar> /tmp/repro.sv
# If it fails on the minimal repro: bug is localized.
# If it passes: the bug is contextual — bisect what came before.
```

### Recipe 3: Predicate-rejection mystery

```bash
# Step 1: trace high, scoped to the suspect rule:
parseability_probe --parse <grammar> file.sv \
  --trace --trace-rules <suspect-rule> \
  --trace-log-file dbg.log 2>&1 | tail -1

# Step 2: look for the predicate verdict lines:
sed 's/\x1b\[[0-9;?]*[a-zA-Z]//g' dbg.log | grep '🛡️'

# Step 3: if the verdict says "could not be resolved", look at the
# unresolved arg's $reference — it tells you which field path the
# predicate expected but couldn't find. Common cause: a sub-rule
# without an explicit `-> {field: ...}` transform whose result
# doesn't have the field the predicate's $path expects.
```

---

## Library Plumbing (`--lib-in`, `--lib-out`)

Two flags (introduced in `SV-EXH-PROOF.3.3.4.a` MVP-0) wire the `@import_from_library` / `@export_to_library` directives to disk:

```bash
# Read imported artifacts from <dir>/<kind>/<name>.facts.json:
parseability_probe --parse <grammar> file.sv --lib-in path/to/artifacts/

# Write exported artifacts to <dir>/<kind>/<name>.facts.json:
parseability_probe --parse <grammar> file.sv --lib-out path/to/artifacts/

# Multi-file parse with prior-file artifacts feeding the next file:
parseability_probe --parse <grammar> file1.sv --lib-out artifacts/
parseability_probe --parse <grammar> file2.sv --lib-in artifacts/
```

Both default to `None` (no library I/O), keeping single-file behaviour byte-identical to runs without these flags.

---

## Environment Variables

| Variable | Purpose | Accepted values |
|---|---|---|
| `PGEN_TRACE_VERBOSITY` | Trace verbosity (overridden by CLI but consulted when CLI doesn't specify) | `none`, `low`, `medium` (or `med`), `high`, `debug` (or `trace`), `off`, `0`-`4` |
| `PGEN_PARSE_DUMP_AST_MAX_BYTES` | Default for `--max-bytes` | Positive integer |

---

## Cost / Performance

Quick reference for each debug feature's runtime cost when **disabled** (the common case):

| Feature | Cost when off | Cost when on |
|---|---|---|
| `--trace` | Zero (single bool check per log site) | Significant (hundreds of MB of output on real inputs) |
| `--trace-rules` | Zero | Same per-line cost as `--trace`, but volume scoped to listed rules' call-trees (~100-1000× less than full `--trace`) |
| `--dump-rule-call-counts` | Counter increment (~1 ns per rule entry, always-on) | Same counter cost + dashboard thread (~1 wakeup per 250 ms) |
| Furthest-position tracking | One `max` update per rule entry (~1-2 ns, always-on) | Same — no flag |
| Predicate self-explaining trace | Zero (gated by `--trace`) | Bundled with `--trace` verbosity |

**Always-on cost summary:** per rule entry, the parser does (1) a `fetch_add` for the call counter, and (2) a `max` update for furthest_position. Together: ~2-3 nanoseconds per rule entry. For a 22,000-call parse, that's ~50-70 microseconds total — undetectable.

---

## Related Documentation

- The [CLI and Workflows](cli-and-workflows.md) chapter covers the broader pgen tooling landscape.
- The [Annotation System](annotation-system.md) chapter explains `@predicate`, `@emit_fact`, and the other semantic-annotation directives that the trace surfaces.
- The [Semantic Store: Parser Memory](semantic-store.md) chapter explains the facts/scope model the predicates query.
