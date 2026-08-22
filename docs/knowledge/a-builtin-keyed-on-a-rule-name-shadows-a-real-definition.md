---
id: a-builtin-keyed-on-a-rule-name-shadows-a-real-definition
title: A builtin that dispatches on a rule's NAME is a reserved word nobody declared — it silently shadows a grammar's own definition
answers:
  - "my rule is defined in the grammar but the stimuli generator emits nothing for it"
  - "a certificate probe shows parsed=false with an empty sample and the rule looks fine"
  - "why does the generator disagree with the parser about what a rule matches"
  - "is epsilon a reserved rule name in PGEN"
  - "where are the code generator's builtin rule names declared"
  - "how do I add a generator builtin without shadowing a grammar rule"
  - "the generator emits X:= with an empty right-hand side and the parser rejects it"
  - "a rule can never be witnessed but every spelling of it parses by hand"
tags: [stimuli-generation, codegen, builtins, grammar-wellformedness, certificate-coverage, duality]
date: 2026-08-22
status: current
evidence: GRAMMAR-WELLFORMED.H.16.3 (`PGEN-GRAMMAR-WELLFORMED-0166`). `stimuli_generator.rs` matched `rule_name == "epsilon"` and returned `Ok(String::new())` before any grammar lookup, shadowing `grammars/ebnf.ebnf:324`'s real `epsilon := ("ε" | "epsilon" | "empty" | "λ")`. Codegen honours the definition — `epsilon` is NOT in `NATIVE_UNRESOLVED_REFERENCE_BUILTINS`. Every witness rendered `X:=` (empty rhs), which the parser rejects. Fix = gate on `!grammar_tree.contains_key("epsilon")`. Cert `144/0/111/33` → `144/0/112/32`; `spf` 8→4, 11→5, 7→3 at seeds 0/7/42.
reverify: "./rust/target/debug/ast_pipeline grammars/ebnf.ebnf --report-certificate-coverage --entry-rule grammar_file --count 40 --seed 0 | grep -o 'UNKNOWN=[0-9]*'   # 32, and PGEN_CERT_COVERAGE_DEBUG_PROBES=1 … | grep \"rule='epsilon'\" is now EMPTY"
---

**A generator builtin keyed on a rule's name is a reserved word that no grammar author was ever told
about.** PGEN's stimuli generator expanded any rule literally called `epsilon` to the empty string,
and it did so *before* looking the rule up:

```rust
fn generate_rule(&mut self, rule_name: &str, …) -> Result<String> {
    if rule_name == "epsilon" {          // ← before any grammar_tree lookup
        return Ok(String::new());
    }
```

`grammars/ebnf.ebnf` defines that rule for real — `epsilon := ("ε" | "epsilon" | "empty" | "λ")` —
and lost it. No error, no warning, no lint.

## The symptom is a duality failure, and it points the wrong way at first

```text
[plannable-probe] rule='epsilon' parsed=false witnessed_target=false sample="ZC:="
```

`parsed=false` reads as *"the forced witness was malformed"*, which invites a hunt in the witness
planner. The decisive move is to ask the **parser** what the language actually is, with the grammar
itself as the oracle:

```text
X := ε          accepted=true        X := λ          accepted=true
X := epsilon    accepted=true        X := &epsilon   accepted=true
X := empty      accepted=true        X := "a" | λ    accepted=true
X :=            accepted=FALSE   <- the generator's own sample
```

Every spelling the grammar licenses parses. The *only* string that fails is the one the generator
produced. ⇒ the generator was wrong about the grammar, not the planner about the witness.

## The real defect: one name, two meanings, in two halves of one pipeline

Codegen keeps its builtins in an explicit const — `NATIVE_UNRESOLVED_REFERENCE_BUILTINS`, which holds
`builtin_any_char` and `builtin_ascii_char` and **not** `epsilon`. So codegen treated a defined
`epsilon` as an ordinary rule while the generator treated it as a builtin. That divergence, not the
empty-string expansion, is the bug.

⭐ **Key a builtin on ABSENCE, not on the name**, and declare it where the other half can see it:

```rust
if rule_name == "epsilon" && !self.grammar_tree.contains_key("epsilon") { … }
```

A defined rule wins; the builtin stays the fallback it was written to be. Both existing callers were
preserved exactly, because both used grammars in which `epsilon` is *undefined* — which is worth
checking before touching any shared behaviour.

## Two habits this measurement is a good argument for

**Re-measure the before arm; do not quote it.** A seed-0 before-value was already written down, and
publishing `spf 8 → 4` from it would have been easy. Reverting the file, rebuilding and re-running
three seeds cost about ninety seconds and produced **8→4, 11→5, 7→3** — monotone at every seed, a
materially stronger claim than the one nearly published.

**For a grammar with a canonical gate, the gate is the oracle.** An ad-hoc
`--report-certificate-coverage` on `rtl_const_expr` returned `Error: Stimuli generation depth exceeded
max_depth=24` and looked like a regression. It was the DEFAULT depth: `rtl_const_expr_cert_gate` runs
at depth 32 and passes `48/0/48/0`. An ad-hoc command with default flags is a different measurement
wearing the same name.

## And the cheapest no-regression proof was a file built for something else

"A stimuli-generator change cannot move parser bytes" is easy to assert from the architecture and easy
to be wrong about. The `GENERATED-REPRODUCIBILITY` baseline re-derives all eleven artifacts and records
a `parser_sha` for each; after rebaselining, **every one was unchanged** — a falsifiable, already-tracked
check answering a question it was not built to answer.
