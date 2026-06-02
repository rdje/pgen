<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_annotation_no_mixed_spread.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: Annotation language `**` flatten-spread doesn't work in mixed array form
description: PGEN's annotation language `**` flatten-spread works in the all-spread form `[$X**]` (e.g. `[$1**]` for slice 23+) but NOT in the mixed form `[<literal>, $X**]` or `[$X**, <literal>]`. In mixed forms, `**` is silently treated as no-op spread — the array element is wrapped instead of flattened.
type: feedback
originSessionId: f74f4acc-7183-408a-ae5d-dcdce15a103c
---
PGEN's annotation language supports `**` flatten-spread in the **all-spread** array form:

```
concatenation = piece+ -> [$1**]   # Quantified-+ flattened to flat array — works
```

But NOT in **mixed** array forms:

```
returned_capture_group_tail = group "," tail -> [$1, $3**]   # mixed — doesn't flatten
```

In the mixed form, `**` is silently treated as no-op (the value is wrapped, not spread). Empirical for the recursive-pair test: `[$1, $3**]` for input `1,2,3` produced nested `[1, [2, [3]]]` instead of flat `[1, 2, 3]`.

**Symptoms:**
- Annotation parses without warning (no `/* WARNING: Return annotation ... failed to parse */` block).
- Test contract passes.
- AST output has nested arrays where consumer expected a flat array.

**Workaround:**
- Avoid mixed `[<literal>, <spread>]` forms in array annotations.
- For comma-separated lists with arbitrary length, leave the raw parens-grouped shape and have consumers walk the structure manually.
- For finite-length sequences, list each element explicitly.

**Why discovered:** During regex.ebnf slice 41 attempting to flatten `returned_capture_group_list`'s comma-separated capture list. Recursive-pair restructure with `[$1, $3**]` left the output nested. Reverted slice 41; captures remain raw.

**Fix direction (codegen-bootstrap concern, not regex-grammar):**
Extend `**` flatten-spread to recursively flatten arrays it spreads — currently flattens Quantified bases but not nested array shapes from rule references. Filed informally as a follow-up codegen-extension target if/when the limitation becomes load-bearing.
