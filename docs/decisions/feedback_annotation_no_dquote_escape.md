<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_annotation_no_dquote_escape.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: Annotation language doesn't support `\"` escape in string literals
description: PGEN's bootstrap annotation parser silently rejects annotations containing `"\""` (escaped double-quote inside a string literal). Failure is silent — codegen emits the rule with a "/* WARNING: Return annotation ... failed to parse */" comment and falls back to the raw shape. Use a different sentinel value (e.g., text label like `"double"`) when you'd otherwise want a literal `"` in an annotation.
type: feedback
originSessionId: f74f4acc-7183-408a-ae5d-dcdce15a103c
---
PGEN's bootstrap annotation parser does NOT support `\"` (escaped double-quote) inside annotation string literals. Annotations like `-> {quote: "\"", ...}` parse to a "failed to parse" warning in the generated code and fall back to returning the raw rule shape.

**Symptoms:**
- Inventory entry (`generated/<grammar>_return_annotations.json`) shows the annotation correctly stored.
- Generated parser code contains a `/* WARNING: Return annotation '...' for rule 'X' failed to parse. */` block instead of the typed shape construction.
- AST output shows the raw rule shape (e.g. 3-element Sequence) instead of the typed object.

**Workaround:**
Use a text-label sentinel instead of the literal character. For an Or-of-N forms slice where you'd want `quote: "<char>"`, switch to `quote: "<text-label>"` for ALL N forms (don't mix — keep the discriminator type uniform). Document the convention clearly so consumers know the field is an enum-like string, not the literal delimiter character.

**Why discovered:** During regex.ebnf slice 33 typing the 8 `callout_string` quote forms (`backtick`/`single`/`double`/`caret`/`percent`/`hash`/`dollar`/`brace`). Initial annotation `callout_double_string -> {quote: "\"", payload: $2}` silently failed; switched all 8 to text-label form (`{quote:"backtick"...}`/etc.) for uniformity.

**How to apply:**
- When designing an annotation that needs a literal `"` in a string-literal field, use a text-label sentinel instead.
- When debugging "annotation present in inventory but raw shape in AST", grep generated parser for `WARNING: Return annotation '...' for rule '<name>' failed to parse` to confirm.
- Filing a separate codegen-fix slice to support `\"` would be valuable but is a parser-bootstrap concern, not a regex-grammar concern.
