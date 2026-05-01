# Walking the AST

This chapter shows how to traverse the AST end-to-end. Read it after [AST Envelope Structure](ast-envelope.md), [ParseContent Variants](parse-content-variants.md), and [The Json Carrier](json-carrier.md).

## The recommended walker pattern

A general-purpose walker has three layers, mirroring the AST's structure:

```rust
use pgen::ast_pipeline::{ParseNode, ParseContent};
use serde_json::Value;

fn walk_node(node: &ParseNode<'_>, depth: usize) {
    let indent = "  ".repeat(depth);
    println!(
        "{}rule={} span={:?}",
        indent, node.rule_name, node.span
    );
    walk_content(&node.content, depth + 1);
}

fn walk_content(content: &ParseContent<'_>, depth: usize) {
    let indent = "  ".repeat(depth);
    match content {
        ParseContent::Terminal(s) => {
            println!("{}Terminal({:?})", indent, s);
        }
        ParseContent::TransformedTerminal(s) => {
            println!("{}TransformedTerminal({:?})", indent, s);
        }
        ParseContent::Json(value) => {
            walk_json(value, depth);
        }
        ParseContent::Sequence(nodes) => {
            println!("{}Sequence", indent);
            for n in nodes { walk_node(n, depth + 1); }
        }
        ParseContent::Alternative(boxed) => {
            println!("{}Alternative", indent);
            walk_node(boxed, depth + 1);
        }
        ParseContent::Quantified(nodes, marker) => {
            println!("{}Quantified({})", indent, marker);
            for n in nodes { walk_node(n, depth + 1); }
        }
    }
}

fn walk_json(value: &Value, depth: usize) {
    let indent = "  ".repeat(depth);
    match value {
        Value::Object(map) => {
            // Discriminator on "type" if present
            let kind = map.get("type").and_then(|v| v.as_str());
            println!("{}Object{}", indent, kind.map(|k| format!(" type={}", k)).unwrap_or_default());
            for (k, v) in map {
                if k == "type" { continue; }
                println!("{}  {}:", indent, k);
                walk_json(v, depth + 2);
            }
        }
        Value::Array(items) => {
            println!("{}Array len={}", indent, items.len());
            for item in items { walk_json(item, depth + 1); }
        }
        Value::String(s) => println!("{}String({:?})", indent, s),
        Value::Number(n) => println!("{}Number({})", indent, n),
        Value::Bool(b) => println!("{}Bool({})", indent, b),
        Value::Null => println!("{}Null", indent),
    }
}
```

Adapt this pattern to whatever consumer-side representation you build. The key insight is that **`walk_json` is the part that does the semantic work** — pattern-matching on `"type"` discriminators and field shapes.

## The "discriminator dispatch" pattern

For consumer code that maps the AST to its own internal regex representation, the cleanest pattern is to dispatch on the `"type"` discriminator at each Json object:

```rust
use serde_json::Value;

enum MyRegexNode {
    Regex { pattern: Box<MyRegexNode> },
    Piece { atom: Box<MyAtomNode>, quantifier: Option<MyQuantifier> },
    // ...
}

fn lower(value: &Value) -> Option<MyRegexNode> {
    let map = value.as_object()?;
    match map.get("type").and_then(|v| v.as_str())? {
        "regex" => {
            let pattern = lower(map.get("pattern")?)?;
            Some(MyRegexNode::Regex { pattern: Box::new(pattern) })
        }
        "piece" => {
            let atom = lower_atom(map.get("atom")?)?;
            let quantifier = lower_quantifier(map.get("quantifier")?);
            Some(MyRegexNode::Piece {
                atom: Box::new(atom),
                quantifier,
            })
        }
        _ => None,
    }
}
```

## Skip-vs-descend decisions per variant

When walking a `ParseNode`, here's the practical rule for each variant:

| Variant | What to do |
|---|---|
| `Json(value)` | Descend into the `value` (object → match on `"type"`; array → iterate; scalar → consume) |
| `Sequence(nodes)` | Iterate each child node |
| `Alternative(boxed)` | Descend into `*boxed` — the wrapper is transparent |
| `Quantified(nodes, "?")` | Descend into 0-or-1 child; if 0 elements, the match was absent |
| `Quantified(nodes, "*")` | Iterate 0-or-more |
| `Quantified(nodes, "+")` | Iterate 1-or-more |
| `Terminal(s)` | Leaf — consume `s` as the matched text |
| `TransformedTerminal(s)` | Leaf — `s` is a possibly-coerced string; treat as the matched text or pre-parsed value depending on context |

## The `pattern` field's nesting depth

A common tripping point: the `pattern` field of the top-level `regex` Json object has visible nesting:

```json
"pattern": [[[{...piece...}, {...piece...}]], []]
```

That's NOT four-deep accidental nesting — it's the result of the un-annotated layers between `regex` and `piece` in the grammar:

- Outer level 1: `pattern -> alternation` Sequence (two elements: alternative, restQ).
- Outer level 2: alternative's `concatenation?` Quantified (0 or 1 of concat).
- Outer level 3: `concatenation = piece+ -> [$1**]` array of pieces (this is where the actual content lives).
- Inner: `[]` empty array — the `("|" alternative)*` rest from alternation, empty when there's no top-level `|`.

A consumer can traverse this by either:

1. **Walking deeply** — recurse into each level until you reach the piece array at the bottom.
2. **Pulling pieces directly** — if you only care about pieces, navigate `pattern[0][0]` to get straight to the array of piece-objects (when there are no top-level alternations).

The Per-Rule Shape Reference chapters document each layer in detail.

## Handling alternation

For `a|b`, the `pattern` field looks like:

```json
"pattern": [
  [[{atom: "a", ...}]],   // first alternative's concat
  [
    ["|", [[{atom: "b", ...}]]]   // second alternative as a |-prefixed pair
  ]
]
```

The first alternative is at `pattern[0]`. The remaining alternatives are in `pattern[1]`, which is a `*`-quantified group of `("|" alternative)*` pairs. Each pair's `[1]` slot is the actual alternative.

For your consumer, this means: alternation is a 2-step extraction — pull `pattern[0]` for the first alternative, then iterate `pattern[1]` extracting each pair's index-1 slot.

The shape examples in [Groups and Alternations](examples-groups-alt.md) make this concrete with actual probe output.

## Handling quantifiers

Every piece carries a `quantifier` slot. As of slice 6 (post-1.1.34) the slot is fully typed:

- **No quantifier:** `"quantifier": []` (empty array — the un-matched `quantifier?` slot).
- **With quantifier:** `"quantifier": {"type": "quantifier", "min": <int>, "max": <int|null>, "greediness": <"lazy"|"possessive"|[]>}`.
  - `min` is always a non-negative integer.
  - `max` is a non-negative integer for bounded quantifiers, or JSON `null` for unbounded (`*`, `+`, `{n,}`).
  - `greediness` is `"lazy"` (when source has `?` suffix), `"possessive"` (when `+` suffix), or `[]` (the un-matched `quant_suffix?` slot — corresponds to PCRE2's "greedy" default; consumers map `[]` → `"greedy"`).

The whole quantifier subtree is annotated as of slice 6: `digits`, `quant_suffix`, `counted_quantifier_body`, `counted_quantifier`, `quant_base`, and `quantifier`. Walking is a six-line typed-field read — no Sequence-wrapper digging, no string-vs-object dispatch. See the [Quantifier Subtree](rules-quantifier.md) chapter for the full walker recipe.

## Handling `\Q...\E` quoted runs

For `\Qab*\E{2,}`, the parser emits 3 separate pieces (per PCRE2 semantics, fixed in PGEN-RGX-0074):

```json
"pattern": [
  [[
    {atom: "a", quantifier: [], type: "piece"},
    {atom: "b", quantifier: [], type: "piece"},
    {atom: "*", quantifier: [...{2,}...], type: "piece"}
  ]],
  []
]
```

Each prefix character (`a`, `b`) is its own piece with empty quantifier. The trailing character (`*`) takes the quantifier. Consumers should treat each piece independently — there's no special "this came from a `\Q...\E` block" marker. PCRE2 semantics dictate that's correct: the runtime behavior of `\Qab*\E{2,}` is identical to `ab\*{2,}`.

The [\\Q...\\E Quoted Literals](examples-quoted-literal.md) chapter has the full family table.

## A complete example — walking `a*?`

Input: `a*?`

```json
{
  "content": {
    "Json": {
      "pattern": [
        [[
          {
            "atom": "a",
            "quantifier": ["*", "lazy"],
            "type": "piece"
          }
        ]],
        []
      ],
      "type": "regex"
    }
  },
  "rule_name": "regex",
  "span": { "start": 0, "end": 3 }
}
```

A consumer walks this by:

1. Top-level `regex`: extract `pattern`.
2. Navigate `pattern[0][0]` to reach the array of pieces (one piece in this case).
3. For each piece, dispatch on `type: "piece"`:
   - `atom`: `"a"` (a leaf string — for now; eventually atom will have its own typed shape).
   - `quantifier`: 2-element array `["*", "lazy"]`:
     - Element 0: `quant_base` is `"*"`.
     - Element 1: `quant_suffix` is `"lazy"`.

This produces consumer-side: `Piece(atom="a", quantifier=Star, greediness=Lazy)`.

## When to walk the ParseNode tree vs the Json tree

| Need | Walk |
|---|---|
| Source-byte spans | The `ParseNode` tree (Json carrier loses spans) |
| Just the structural shape | The Json tree (smaller, simpler, fully typed where annotated) |
| Per-rule provenance (which grammar rule produced this node) | The `ParseNode` tree (the `rule_name` field) |
| Pure JSON serialization | Either — `to_json_value()` collapses both to JSON |
| Most consumer-side regex compilation | The Json tree |

For RGX's adapter refactor, walking the Json tree is sufficient for compilation and matching; if you later need to surface error spans back to the regex source author, walk the `ParseNode` tree for those subtrees.
