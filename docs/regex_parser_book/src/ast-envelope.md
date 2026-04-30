# AST Envelope Structure

This is the structural overview of what comes back from `parse_full_regex`. Read this chapter once; it sets the vocabulary used in every per-rule chapter.

## The outermost ParseNode

Every parse — from a one-character pattern like `a` to a complex multi-thousand-byte regex — returns a single top-level `ParseNode<'input>`:

```rust
ParseNode {
    rule_name: "regex",
    content: ParseContent::Json(...),
    span: 0..N,
}
```

The `rule_name` of the top-level node is **always** `"regex"`, because that's the entry rule of `grammars/regex.ebnf`. The `span` covers the entire input that was consumed by the parser. `content` carries the typed AST.

## When you serialize, you get this shape

`parseability_probe --parse-dump-ast-pretty regex INPUT OUTPUT.json` produces JSON that mirrors the Rust struct one-to-one:

```json
{
  "content": {
    "Json": {
      "pattern": [...],
      "type": "regex"
    }
  },
  "rule_name": "regex",
  "span": {
    "end": 1,
    "start": 0
  }
}
```

The `Json` key inside `content` indicates which `ParseContent` variant holds the value. (See [ParseContent Variants](parse-content-variants.md) for all six variants and when each appears.)

For most current downstream traversal needs, **the top-level `regex` node has `Json` content**, and consumers walking the AST only need to look at the inner JSON object — they don't need to care about the `Sequence` / `Alternative` / `Quantified` recursive variants except in specific places noted in the per-rule chapters.

## Three layers of structure

Reading from outside in, the AST has three layers:

1. **The Rust envelope** — `ParseNode { rule_name, content, span }`. This is the type-system shell.
2. **The ParseContent variant** — which kind of content this node holds. For annotated rules, this is `Json(...)`; for unannotated multi-element rules it may be `Sequence(...)`, `Quantified(...)`, etc.
3. **The typed-Json shape** — the actual semantic content of the parse, structured by the grammar's return annotations.

Most downstream walking lives at layer 3 — the typed JSON shape. Layers 1 and 2 are infrastructure for the codegen.

## The simplest possible example

For input `a` (one ASCII letter):

```json
{
  "content": {
    "Json": {
      "pattern": [
        [
          [
            {
              "atom": "a",
              "quantifier": [],
              "type": "piece"
            }
          ]
        ],
        []
      ],
      "type": "regex"
    }
  },
  "rule_name": "regex",
  "span": {
    "end": 1,
    "start": 0
  }
}
```

Walking layer by layer:

- **Layer 1**: `ParseNode { rule_name: "regex", span: 0..1, content: ... }`.
- **Layer 2**: `content` is `ParseContent::Json(<value>)`. The `<value>` is whatever the `regex` rule's annotation produced.
- **Layer 3**: The `regex` rule's annotation is `-> {type: "regex", pattern: $1}`, so the value is `{"type": "regex", "pattern": <$1>}` where `$1` is the matched `pattern` content.

The `pattern` field's value `[[[{...}]], []]` is the raw shape of `pattern -> alternation -> alternative -> concatenation`; see [Top-Level Rules](rules-top-level.md) for why it has that nesting depth.

## A quick map of where Json appears

The `Json` carrier replaces the recursive `Sequence`/`Alternative` envelope **wherever a rule has an explicit return annotation in `grammars/regex.ebnf`**. Currently that includes:

- `regex` — top level, always Json.
- `pattern` — `-> $1`, content unwraps to the inner alternation's shape.
- `concatenation` — `-> [$1**]`, content is a flat list.
- `piece` (both branches) — Json piece-object or Sequence-of-pieces (via `piece_quoted_run_quantified`).
- `piece_quoted_run_quantified` — Sequence-of-pieces (lifted via `**`).
- `quoted_run_inner_piece` — Json piece-object.
- `quant_suffix` (both branches) — Json string `"lazy"` or `"possessive"`.
- `digits` — typed integer (via `@transform: str::parse::<usize>`).

Rules that don't yet carry annotations still emit the legacy recursive shape (`Sequence`, `Alternative`, `Quantified`, `Terminal`). These are gradually being annotated through the task #40 ("Annotate regex.ebnf for full AST usability") slice campaign.

For the cumulative cumulative-state reference, see the Per-Rule Shape Reference chapters.

## Span semantics

Every `ParseNode`'s `span` is `start..end` where `start` and `end` are byte offsets into the original input string. For UTF-8 inputs, the offsets are byte-positions, not codepoint indices — multi-byte characters take their full byte width.

Inner synthetic nodes (those created by the codegen for sequence-element packaging or quantifier loop bodies) sometimes carry `0..0` spans because they don't correspond to a specific source range. The OUTER `regex` node, top-level rule nodes, and any node whose content directly reflects matched input always carry meaningful spans.

For consumers that need source-range reporting to surface error locations back to a regex source author, the spans on the rules that matched user-provided text are reliable. The inner-synthetic spans are not — they're an implementation detail.
