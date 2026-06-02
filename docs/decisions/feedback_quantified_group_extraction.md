<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_quantified_group_extraction.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: Use `::N*` extraction-spread for `X (sep X)*` patterns, not raw `$N`
description: PGEN's annotation language has a first-class extraction operator `$N::M[*]` to pull position M from each entry of a Quantified-of-Sequence. For pure-list patterns like `X (comma X)*`, the right annotation is `[$1, $2::2*]` (clean flat array of items, commas dropped) — NOT `{first: $1, rest: $2}` (which exposes the raw envelope of `[comma, item]` pairs). Audit per-rule case-by-case.
type: feedback
originSessionId: f74f4acc-7183-408a-ae5d-dcdce15a103c
---
For a rule like `tf_port_list := tf_port_item ( comma tf_port_item )*`:

- `$1` = the leading required `tf_port_item` (single value).
- `$2` = the entire `( comma tf_port_item )*` iteration. Shape: `[[comma, item], [comma, item], ...]` — array of 2-element Sequence matches. NOT a clean array of items.

**Why:** A parens-grouped Sequence inside a Quantified produces a 2-element array per iteration entry (positions 1 and 2). `$2` references the whole iteration array; each entry preserves the inner Sequence's positional layout.

**Wrong default:** `-> {first: $1, rest: $2}` — `rest` exposes raw `[[comma, item], ...]`. Consumers must walk past the comma each iteration. This was used widely in early SV slices and was sloppy.

**Right pattern (Category A — pure list with separator):**

```
tf_port_list := tf_port_item ( comma tf_port_item )*
             -> [$1, $2::2*]
```

`$2::2*` = "extract position 2 from each entry of $2, then spread". Result: clean flat `[item1, item2, ...]` — commas dropped.

The annotation language defines this in `grammars/return_annotation.ebnf` (extraction_expression rule) and uses it itself: `object_properties := object_property (',' object_property)* -> [$1, $2::2*]`.

**Three audit categories for `{first, rest}` patterns:**

1. **Category A — pure list with separator** (e.g., `X (comma X)*`): Use `[$1, $2::2*]`. The vast majority of `list_of_*` rules.
2. **Category B — op-chain with payload-per-iteration** (e.g., `expr (op attrs expr)*` in `constant_expression`, `expression_base.operand_chain`, every `binop_chain` level `next (OP next)*`): operators matter for the fold, so `$N::2*` (Category A, drops operators) is WRONG here. **ROOT CAUSE (corrected 2026-05-16 — supersedes the earlier "$2* workaround" guidance, which was empirically falsified):** the corrupting factor is an **INLINE alternation as the lead element of the quantified iteration** — e.g. `additive_expr := mul ((plus|minus) mul)* -> {lhs:$1, rest:$2}`. The inline `(plus|minus)` corrupts the positional model the return-transform builds, so the rule's annotation mis-recurses into the iteration entries and emits a literal `"<invalid_sequence_access>"` + a malformed nested object whenever the tail matches (`a + b`). **`$2*` single-star does NOT fix it** — it re-applies the rule transform per spread entry and reproduces the corruption; it was tried first on rtl_const_expr and REJECTED. `$2**`/`$2::2*` also wrong (flatten / drop operators). **THE CORRECT FIX = the mature `systemverilog.ebnf` idiom:** lift the inline alternation into a **named operator rule** (`additive_op := plus | minus`, mirroring `binary_operator` in `constant_expression`/`expression_base.operand_chain`) so the level becomes `mul (additive_op mul)* -> {lhs:$1, rest:$2}` with **bare `$2`** — then `rest` is a clean array of `[op-envelope, operand]` iteration entries (op text at `entry[0][1]`), `[]` when no operator. Landed + gate-locked in `rtl_const_expr` (schema 1→2, `PGEN-RTL-0002`); see `DEVELOPMENT_NOTES.md` "RTL-CE-Slice-2 root cause". Same class still open in rtl_frontend/vhdl `binop` + sv_preprocessor `pp_if_branch.keyword` (`SVPP-0001`). ALWAYS consult `[[feedback_ebnf_consult_annotation_docs]]` first and verify the regenerated dump with `parseability_probe --parse-dump-ast-pretty <g> <file>` before trusting any op-chain `rest`.
3. **Category C — `X X*` with no separator** (e.g., `production production*` in `randsequence_statement`, `case_item case_item*`): `$2` is already a clean array of X (single-element non-grouped Quantified). `{first, rest}` is fine.

**For named/keyed lists** (e.g., `member_id colon pattern (comma member_id colon pattern)*`):
The inner Sequence has 4 elements: `[comma, name, colon, pattern]`. To get a clean array of `{name, pattern}` objects, use a helper rule that destructures the inner Sequence — `[$N::2*]` only extracts a single position.

**Reference:** `grammars/return_annotation.ebnf` lines 50-58 (extraction_expression definition) and line 158 (self-application).

**Common mistake to avoid:** Defaulting to `{first: $1, rest: $2}` for any `X (sep X)*` rule without checking the inner Sequence shape. The right shape depends on whether you want commas/operators preserved or just the items.
