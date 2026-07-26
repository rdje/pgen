# QUANT-PLUS-ITER: a `+` over a rule reference iterates ONCE in the scratch shape — and VHDL's identical idiom does not

## Metadata

- Tree ID: `QUANT-PLUS-ITER`
- Status: `active` (opened 2026-07-26, session #211)
- Family / slice-id prefix: `PGEN-QUANT-PLUS-ITER-<NNNN>`
- Created: `2026-07-26`
- Owner: repo-local workflow
- Opened by: `LANG-CAPABILITY-AUDIT.8`, which surfaced this **while probing something
  else** and deliberately did NOT absorb it (recovery was proven not to be the cause).

## Why this tree exists

While building the end-to-end recovery probe for `LANG-CAPABILITY-AUDIT.8`, a
minimal scratch-slot grammar whose entry rule quantifies a rule reference
**consumed exactly one occurrence** and then stopped. This is a potential
parser-correctness defect in the most basic construct PGEN has, so it gets its own
tree rather than a footnote.

⛔ **It is NOT "PGEN's `+` is broken".** The single most important measurement in
this file is the VHDL control, which shows the same idiom working at gate scale.
Whoever takes this must start from that tension, not from the alarming half.

## What was MEASURED (all tool-backed, session #211)

### The failing shape

```ebnf
stmt := "a" ";"
      | "b" ";"

scratch := stmt+
```

| input | verdict |
|---|---|
| `a;` | ACCEPT |
| `a;a;` | ⛔ reject — *did not consume full input at position 2* |
| `a;b;` | ⛔ reject — *at position 2* |
| `b;a;b;a;` | ⛔ reject — *at position 2* |

Position 2 = exactly one `stmt`. The quantifier stops after one iteration.

### Four shape variants — the behaviour does NOT move

| variant | result |
|---|---|
| `scratch := stmt+` (quantifier IS the entry rule's whole body) | one iteration |
| `list := stmt+` / `scratch := list` (one level below entry) | one iteration |
| `stmt` **single-branch** (`stmt := "a" ";"`) | one iteration |
| `scratch := stmt+ "!"` (quantifier has a **follower**) | one iteration — and even `a;!` rejects at position 2 |

⇒ independent of entry-rule position, of nesting depth, and of branch count.

### It is NOT caused by `@recover` (the control that matters for `.8`)

The same grammar with **all recovery annotations removed** rejects `a;a;` at
position 2 **identically**. Recovery is exonerated — which is why
`LANG-CAPABILITY-AUDIT.8` could close.

### It is NOT a cascade-vs-memoized duality break

`bare_parse` (`generated/*_parser.rs:295`) selects the **cascade** engine when
tracing is off and the **memoized** engine when a logger is enabled — so an
untraced run and a traced run execute *different code*. Both were compared
directly on the same inputs:

```text
input      CASCADE (untraced)     MEMOIZED (traced)
a;!        reject                 reject
a;a;!      reject                 reject
a;a;a;!    reject                 reject
```

Both engines agree. (⚠️ **Toolbox note worth propagating regardless:** tracing
changes which engine runs. Anyone diagnosing with `PGEN_TRACE_VERBOSITY` is not
necessarily observing the path that produced the untraced verdict. This trap cost
real time in session #211 and belongs in `TOOLBOX.md`.)

### Things ruled OUT by reading the generated source

- **Memo key** — `memoized_call` keys on `(rule_id, self.position)` captured at
  ENTRY (`:4057`); correct. The trace line *"Memoized successful result for rule 0
  at position 2"* prints `self.position` (the END) and is merely **cosmetically
  misleading**, not evidence of a mis-keyed memo.
- **`try_parse` backtracking** — the `Err` arm restores `self.position = saved_pos`
  correctly.
- **Codegen structure** — the emitted `+` loop is present and well-formed
  (`SAFETY_LIMIT`, zero-length guard, `results.push`, `iteration_count`), and for
  `stmt+ "!"` the **follower is emitted too** (`match_lit_ascii("!")`) inside a
  proper `ParseContent::Sequence`. The raw AST is also correct:
  `[["rule","scratch"],["rule_reference","stmt"],["operator","+"],["quoted_string","!"]]`.
- **Layout sensitivity** — adding whitespace between items (`a; a; !`) changes nothing.

### ⭐⭐ THE CONTROL THAT REFRAMES EVERYTHING: VHDL does this correctly

`grammars/vhdl.ebnf:208` — `record_type_definition := kw_record record_element_declaration+ kw_end kw_record`
— is a bare rule-reference `+`, and the **shipped** VHDL parser iterates it fine:

```text
v1 (1 record element)   rc=0  passed for grammar
v2 (3 record elements)  rc=0  passed for grammar
```

Same for `view_element+` (`:133`) and `case_statement_alternative+` (`:320`).

⇒ **the defect is shape-specific, and the distinguishing factor is NOT YET
IDENTIFIED.** Candidate differences between the two, none confirmed:
the scratch grammar declares no trivia/layout rule at all; VHDL's quantified rule is
reached from a much deeper context; VHDL's followers are keyword tokens rather than
a bare literal; the scratch grammar is whitespace-**sensitive** by inference while
VHDL is not (`layout_sensitivity().terminals`, which also gates FIRST-set predictive
dispatch at `ast_based_generator.rs:4287`).

Note also that `grammars/json.ebnf` expresses repetition by **recursion**
(`members := pair /,/ members`), not by `+` — so JSON's healthy iteration is not
evidence about `+` either way.

## Leaves

### `.1` — Identify the distinguishing factor between the scratch shape and the VHDL shape (`todo`)

- **Start from the tension, not from the failure.** The deliverable is the ONE
  property that differs and flips the behaviour — bisect the scratch grammar
  *towards* the VHDL shape (add a layout rule; wrap the quantified rule deeper; swap
  the follower literal for a keyword-ish token) until it starts iterating, one
  variable at a time.
- ⛔ Do NOT begin by reading the `+` codegen again — `.0` already read it and it is
  structurally correct. The cause is upstream of the loop or in a guard around it.
- Suspect to check FIRST (cheapest, and the only one with a known conditional
  emission): the FIRST-set predictive dispatch prune guard, emitted only when
  `top_level && self.layout_sensitivity().terminals` — a condition the tiny
  whitespace-insensitive scratch grammar satisfies and VHDL does not.
- ⚠️ Each variant costs a `make -C rust focus_scratch` + `parseability_probe` rebuild
  (~3.5 min). Batch the variants into as few rebuilds as the question allows.

### `.2` — Adjudicate and fix, or document the shape as unsupported (`todo`)

- Blocked on `.1`. If the distinguishing factor is a real bug, fix it with the usual
  acceptance checklist and add a regression pin. If the scratch shape is genuinely
  ill-formed, the linter must **say so** rather than silently producing a parser that
  iterates once — that is the `ANNOTATION-PLACEMENT` family (*a check that cannot see
  a defect class must say so, not return green*).

### `.3` — Record the trace-changes-the-engine trap in `TOOLBOX.md` (`todo`)

- Independent of the outcome above and useful immediately: `bare_parse` means
  enabling a trace switches the parser from the cascade engine to the memoized one.
  A diagnostic session can therefore observe a path that did not produce the verdict
  being investigated. This belongs in the toolbox as a standing caveat.

## Acceptance Criteria (tree)

- The distinguishing factor between the failing and working shapes is **named and
  demonstrated** by a flip, not hypothesized.
- Either a fix with before→after evidence, or a diagnostic that refuses the shape.
- No shipped parser's behaviour changes without byte-identity evidence.

## Evidence

- Measurements are reproduced inline above; every one was taken through the
  PARSE-HARNESS scratch slot (`grammars/scratch/scratch.ebnf`, restored to its
  committed fixture afterwards) and the shipped `vhdl` registry parser.
- Companion context: `docs/tasks/LANG-CAPABILITY-AUDIT.md` leaf `.8`.
