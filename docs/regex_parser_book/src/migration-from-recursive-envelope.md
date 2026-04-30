# From the Recursive Envelope (pre-1.1.30)

This chapter is for consumers who built against PGEN regex parser releases **before 1.1.30** and need to migrate to the current annotated shape. If you are starting fresh, you can skip this chapter — go straight to [Walking the AST](walking-the-ast.md).

## What changed

Pre-1.1.30, the regex parser emitted a fully-recursive envelope where every rule was wrapped in `Sequence` / `Alternative` / `Quantified` content variants, and there was no `Json` carrier — every grammar rule produced a `ParseNode` whose `content` was one of the recursive variants. Consumers walked the envelope by structural pattern-matching: descend into `Alternative.0`, iterate `Sequence.0`, etc., until reaching a `Terminal(&str)` leaf.

From 1.1.30 onward, rules can carry **return annotations** in `grammars/regex.ebnf`. Annotated rules emit `ParseContent::Json(serde_json::Value)` carrying the typed shape directly, replacing the recursive envelope at that rule's boundary.

The result for downstream consumers:

- The top-level `regex` node now emits `Json({type: "regex", pattern: ...})` — flat, easy to dispatch.
- `pattern`, `concatenation`, both `piece` branches, and the quoted-run helpers all emit typed Json.
- Quantifier internals at this release are partially annotated — `digits` is integer-typed and `quant_suffix` is enum-typed; the rest of the quantifier subtree is still raw envelope.
- Atom subtree (groups, character classes, escapes, etc.) is still entirely raw envelope at this release. Annotation is rolling out via the task #40 slice campaign.

## Side-by-side: top-level shape

For input `a`:

**Pre-1.1.30 (recursive envelope):**

```json
{
  "rule_name": "regex",
  "content": {
    "Sequence": [
      { "rule_name": "pattern", "content": {
        "Alternative": { "rule_name": "alternation", "content": {
          "Sequence": [
            { "rule_name": "alternative", "content": {
              "Sequence": [
                { "rule_name": "concatenation", "content": {
                  "Quantified": [
                    [{ "rule_name": "piece", "content": {
                      "Sequence": [
                        { "rule_name": "atom", "content": {
                          "Alternative": { "rule_name": "literal_atom", "content": {
                            "Terminal": "a"
                          }}
                        }},
                        { "rule_name": "quantifier", "content": {
                          "Quantified": [[], "?"]
                        }}
                      ]
                    }}],
                    "*"
                  ]
                }}
              ]
            }},
            ...
          ]
        }}
      }}
    ]
  },
  "span": { "start": 0, "end": 1 }
}
```

(Approximation — exact pre-1.1.30 shapes varied across point releases.)

**Current (1.1.30+):**

```json
{
  "rule_name": "regex",
  "content": {
    "Json": {
      "type": "regex",
      "pattern": [
        [[
          { "atom": "a", "quantifier": [], "type": "piece" }
        ]],
        []
      ]
    }
  },
  "span": { "start": 0, "end": 1 }
}
```

The walker that used to descend through 8+ envelope layers to reach the `"a"` Terminal now reads `node.content.Json.pattern[0][0][0].atom` and is done.

## Migration path

### Step 1 — switch to typed-Json reads

Wherever you used to pattern-match `Sequence(...)` / `Alternative(...)` / `Quantified(...)` against the top-level node, replace with `Json(...)`. The `to_json_value()` helper produces the same `serde_json::Value` either way (Alternative-wraps are transparent in the `to_json_value()` mapping), but reading directly from the typed `Json` variant skips the recursion.

```rust
// pre-1.1.30
match &node.content {
    ParseContent::Sequence(children) => walk_recursive(children),
    ParseContent::Alternative(boxed) => walk_recursive_one(boxed),
    ParseContent::Quantified(items, _) => walk_recursive(items),
    ParseContent::Terminal(s) => leaf(s),
    _ => unreachable!(),
}

// 1.1.30+
match &node.content {
    ParseContent::Json(value) => walk_typed(value),
    // recursive variants still apply for unannotated rules — keep handlers
    ParseContent::Sequence(children) => walk_recursive(children),
    ParseContent::Alternative(boxed) => walk_recursive_one(boxed),
    ParseContent::Quantified(items, _) => walk_recursive(items),
    ParseContent::Terminal(s) => leaf(s),
    ParseContent::TransformedTerminal(s) => leaf(s),
}
```

For most regex-walking, the entry point is `walk_typed(value)` where `value` is the inner `serde_json::Value`. Recursive-envelope handlers are still needed because the atom subtree (and most of its descendants) hasn't been annotated yet.

### Step 2 — handle the partial-annotation reality

Today's regex parser is **partially annotated**. That means a single AST traversal will mix both shapes:

- Top-level `regex` and its immediate descendants — typed Json.
- Atoms — recursive envelope (raw `Sequence`, `Alternative`, `Quantified`).
- Some leaves — `Terminal` strings.

A walker has to handle both. The `to_json_value()` helper hides this by collapsing recursive variants to nested `serde_json::Array`s, so consumers that work entirely from the JSON dump see a uniform `serde_json::Value` tree. Consumers walking the `ParseNode` tree directly do see both.

**Recommendation:** for regex-AST walking, prefer `to_json_value()` on the top-level node and walk the resulting unified `serde_json::Value`. This is the simplest path for consumers and the one this book's examples assume.

### Step 3 — re-bind to current rule shapes

The per-rule chapters in this book give the **exact** current shape per rule. Re-implement your walker rule-by-rule:

- Top-level — see [Top-Level Rules](rules-top-level.md).
- Pieces and quoted-runs — see [piece and the Quoted-Run Family](rules-piece.md).
- Quantifier — see [Quantifier Subtree](rules-quantifier.md).
- Everything inside `atom` — see [Atom Subtree](rules-atom.md).

Each chapter also lists which sub-rules are currently annotated vs raw envelope.

### Step 4 — version-pin

Pin your consumer to a specific PGEN release tag. Because the slice campaign is converting raw-envelope shapes into typed shapes one rule at a time, the AST shape evolves with releases. The contract document (`docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md`) lists all shape-affecting changes per release.

## What pre-1.1.30 patterns broke

| Pattern | Pre-1.1.30 | 1.1.30+ |
|---|---|---|
| Top-level `regex` node descend | `Sequence([pattern, ...])` | `Json({type:"regex", pattern, ...})` |
| Reaching a piece's atom | 6+ recursive descents | `piece.atom` field |
| Quantifier `min` | walking digits chars | `digits` is now integer (typed) |
| `\Q...\E`+ quantifier | whole-block-quantified piece (BUGGY) | per-char piece array, last char carries quantifier (FIXED in PGEN-RGX-0074) |
| `quant_suffix` lazy/possessive | structural (which branch) | string `"lazy"` / `"possessive"` |

The PGEN-RGX-0074 fix is **semantically incompatible** with pre-1.1.31 consumers. Pre-1.1.31 consumers had to either implement special-case `\Q...\E` handling that mirrored PCRE2's "quantifier binds to last char" rule, or accept buggy runtime matching. Post-1.1.31 the AST shape itself is correct — consumers can drop their workaround.

## Pre-1.1.30 → current quick reference

If you have a working pre-1.1.30 walker, here's the short-list of changes:

1. Add a `ParseContent::Json(value)` arm to your top-level match.
2. Inside that arm, switch on `value.get("type").as_str()` for the discriminator.
3. Read `pattern` field directly off the regex object — it's a 2-element array `[<head>, <tail>]` where `head` is the first alternative and `tail` is the rest.
4. Keep your recursive-envelope handlers around for the atom subtree — they still apply.
5. Drop any custom `\Q...\E` quantifier-attachment workaround — the parser does it correctly now.

The amount of consumer code that goes away is substantial — most pre-1.1.30 regex walkers were primarily envelope-descent boilerplate. Today the descent only stays where the rules haven't been annotated yet.

## Future migrations

Each task #40 slice that annotates additional rules will reduce the amount of recursive-envelope handling consumers need. The contract document lists each slice's effect on the AST shape. We commit to:

- **No silent shape changes** — every annotation slice gets a contract-version bump and a row in the changelog.
- **Byte-equivalence preservation** — `parse_full_regex().content.to_json_value()` always equals `parse_regex_typed()` for the same input. (See [Schema Versioning](schema-versioning.md).)
- **No annotation rollbacks** — once a rule is annotated, the typed shape is part of the schema. Reverting requires consumer-coordinated migration, not a silent change.
