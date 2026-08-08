---
id: a-memo-key-must-name-every-context-the-outcome-depends-on
title: A packrat entry keyed `(rule, position)` may only hold outcomes that are a function of `(rule, position)` — two context axes have now violated that, and on both of them the obvious "just don't cache it" cure was measured unshippable
answers:
  - "my parser rejects an input that the same rule parses fine in isolation — where do I look"
  - "a rule parses under --entry-rule but the whole file rejects at that construct — what is going on"
  - "longest_match picked the shorter alternative even though a longer one exists — why"
  - "is it safe to memoize a parse failure caused by the recursion guard"
  - "should a memo refuse tainted successes as well as tainted failures"
  - "removing a cache entry made my parser reject input it used to accept — how is that possible"
  - "does my packrat memo affect which inputs the parser accepts, or only how fast it runs"
  - "what state may a packrat memo key omit"
  - "I found a memo soundness bug — should I just stop caching the tainted entries"
  - "why did excluding tainted entries from the packrat memo make the parser slower instead of just safer"
  - "how do I tell a grammar defect from an engine defect when a construct will not parse"
  - "the cycle guard fired and now a legal parse is refused — is the guard wrong"
tags: [engine, memoization, packrat, recursion, soundness, taint, performance, diagnosis]
date: 2026-08-08
status: current
evidence: rust/src/ast_pipeline/ast_based_generator.rs (`recursion_block_floor` + the `memoized_call` taint gate); rust/src/ast_pipeline/ast_based_generator/cascade.rs (the thin memo's mirror); rust/src/ast_pipeline/mod.rs (`RecursionGuard::last_block_frame`); rust/src/parse_harness_combinator_suite.rs (case `recursion_guarded_memo_isolation`); docs/tasks/SV-CORPUS-GRAD.md leaf .3.12; docs/tasks/MEMO-STORE-SOUNDNESS.md (the sibling axis)
reverify: "grep -q 'recursion_block_floor' rust/src/ast_pipeline/ast_based_generator.rs && grep -q 'last_block_frame' rust/src/ast_pipeline/mod.rs && grep -q 'recursion_guarded_memo_isolation' rust/src/parse_harness_combinator_suite.rs && echo RECURSION-TAINT-WIRED"
---

**The invariant, stated once:** a cache keyed `K` may only hold outcomes that are a function of `K`.
PGEN's packrat memo is keyed `(rule_id, position)`. Every piece of *other* mutable state that can
change whether a rule body succeeds is therefore a soundness obligation, and each one has to be
either folded into the key, validated at replay, or kept out of the cache.

Two independent axes have now been found violating it, years apart in the code and months apart in
the finding, with the **same shape and the same wrong first cure**:

| axis | the state the key omits | symptom | tree |
|---|---|---|---|
| semantic store | facts/scopes a `@predicate` reads | a valid input REJECTS because a failure cached before an `@emit_fact` is replayed after it | `MEMO-STORE-SOUNDNESS` (F1) |
| recursion guard | which rules are on the live PARSE STACK | a valid input REJECTS because a cycle-guard rejection is replayed from a stack where the guard would not fire | `SV-CORPUS-GRAD.3.12` |

## The diagnosis signature — how you recognise it in minutes

⭐ **The rule parses the input in isolation and rejects it in context.** That pair is the tell, and it
is one command apart:

```bash
parseability_probe --parse-dump-ast-pretty systemverilog in.txt out.json --profile sv_2017 --entry-rule cast     # PASS
parseability_probe --parse-dump-ast-pretty systemverilog in.txt out.json --profile sv_2017 --entry-rule primary  # REJECT, stops short
```

A rule that parses standalone and fails as a branch of its own parent is **not a grammar defect** —
grammar text does not change with the caller. Trace the PARENT (never only the suspect,
`TOOLBOX.md` §2.2) and read for a `Memo hit … cached failure` immediately under the branch entry:

```
🚪 Entering branch 9/15 for rule 'primary_sv_2017' at position 0
💾 Memo hit for rule 110 at position 0 - cached failure
🏁 Rule 'primary_sv_2017' selected branch 2/15 consuming 9 chars (branch_policy=longest_match)
```

A `longest_match` tournament that picks the SHORTER alternative is impossible unless the longer one
never ran. The cache is what stopped it running.

## The trap: "then just don't cache the tainted entries"

That is the obvious cure, it is sound, and it has now been **measured unshippable on both axes**:

| axis | exclusion cost, measured |
|---|---|
| store | SV `scr1_core_top` **1 484 ms → 173 580 ms (117×)**; the shape-contract gate to ~2.9 h |
| recursion | SV `t_math_synmul_mul.v`, a file that passes inside a 60 s budget, ran **past 300 s** |

The reason is the same both times: **entry counts are not attempt counts.** In a deeply
mutually-recursive grammar almost every body transitively touches the tainted state, so a
subtree-wide exclusion deletes the packrat protection exactly where it is holding PEG backtracking
down. ⛔ Do not price an exclusion by "what fraction of rules read this state" — measure it.

## The cure that works: scope the taint to what actually varies

Neither axis needs exclusion; each needs a **precise** statement of when a replay is legal.

- **Store axis — VALIDATE.** The store has a monotone write epoch. Stamp a tainted entry with the
  epoch and replay it only while the epoch is unchanged; evict and re-parse otherwise.
- **Recursion axis — FRAME-SCOPE.** There is no "recursion epoch" to stamp: the guard's verdict is a
  function of the live stack, which no scalar summarizes. But the dependence is narrower than it
  looks. A block whose blocking frame is *the memoized rule itself, or one of its own descendants*
  is re-created identically by every replay of that body and is harmless. Only a block caused by a
  **strict ancestor** makes the outcome caller-dependent. So record the blocking frame's stack index
  (both guard scans return on the first, shallowest match, so that index is exactly the floor),
  reset it around each body, and refuse to cache only when `floor < entry_depth - 1`. Handing the
  floor up min'd with the caller's makes taint propagate exactly as far as the ancestor that owns
  the frame, and stop there.

## ⛔ Do NOT assume the two axes are symmetric — they are opposite, and the corpus proved it

The recursion fix was first written to refuse BOTH polarities, by analogy with the store axis where
a stale SUCCESS was measured flipping both the verdict and the tree (`sem_memo_success_verdict` /
`sem_memo_success_ast`). The corpus refuted the analogy: **4 files regressed pass→fail**, every one a
cast inside an index in a cyclic expression context (`foo[const'(i)]`, `shuffle[ctr+Word'(i)]`,
`be_mask[int'({…}) +: 2]`). Ask what each condition actually establishes:

| | a cached FAILURE says | a cached SUCCESS says |
|---|---|---|
| under a **store** condition | "no derivation *given this store*" — and the store moves | "this derivation exists *given this store*" — and the store moves ⇒ **both stale** |
| under a **recursion** condition | "the search stopped here", NOT "no derivation exists" ⇒ **unsound to replay** | "this derivation exists, and was found" — still true from any stack ⇒ **sound, and needed** |

The store changes what the CORRECT ANSWER IS. A guard changes only what the SEARCH CAN REACH. So the
store axis must validate successes and the recursion axis must **preserve** them.

⭐ **And the success replay is load-bearing, not merely harmless: on cyclic rules the packrat memo is
part of the ACCEPTANCE SEMANTICS.** It is how this engine parses indirect left-recursive constructs at
all — the guard refuses to re-derive them, and the cache supplies the derivation that was found
before the guard applied. Two things follow. Acceptance on such rules is **evaluation-order-dependent**
(the same construct parses or not according to which context reached that `(rule, position)` first),
and every *"dropping a cache cannot change a correct parse"* argument is true only of **acyclic**
rules. Check which kind of rule you are reasoning about before reusing that argument.

## One more generalisation worth keeping

**A guard that fires correctly can still cause a wrong answer.** Every cycle-guard rejection in this
defect was right — the parse would not have terminated otherwise. The defect was entirely in what was
done with that rejection afterwards. When a guard and a cache meet, audit the cache.

## The standing question to ask

Whenever you add state that a rule body can read or that can make it fail — a store, a guard, a
depth budget, a profile, a lookahead cache — ask the one question: **is this in the memo key?** If
not, it needs a validation rule or a scoping rule, and the answer belongs in the same commit. The
regression pin for this axis is the `recursion_guarded_memo_isolation` case in the structural
combinator suite (`make -C rust parse_harness_combinator_gate`), placed there deliberately rather
than in a bare `#[test]`, because `cargo test --lib` is RED on HEAD and no gate reads it
(`CI-PARITY-GATE-ROT.21`) — a pin nothing runs is a pin that does not exist.
