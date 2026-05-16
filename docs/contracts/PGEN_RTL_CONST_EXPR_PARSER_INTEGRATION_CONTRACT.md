# docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md

## Purpose
Define the current downstream integration contract for PGEN's `rtl_const_expr` parser family.

This is the document downstream projects (primarily RTLSyn, for deterministic parameter / width / generate evaluation before elaboration) embedding the PGEN rtl_const_expr parser should read first.

## Contract Identity
- Contract version:
  - `1.0.2`
- Parser release version:
  - `1.0.2`
- Embedding API contract baseline:
  - tracked under `rust/docs/EMBEDDING_API_CONTRACT.md`
- rtl_const_expr AST-dump schema version:
  - `2` (breaking shape correction — see Release 1.0.2 Highlights)
- Annotation count:
  - `26` (19 `return_object` + 7 `return_scalar`; 18 distinct rules)
- Last updated:
  - `2026-05-16`
- Current grammar family label:
  - `rtl_const_expr`
- Per-family mdBook:
  - `docs/rtl_const_expr_parser_book/` (tracked HTML at `docs/rtl_const_expr_parser_book-html/`)
- Per-family gate:
  - `make -C rust SHELL=/opt/homebrew/bin/bash rtl_const_expr_parser_book_gate`
- Per-family ast-shape-contract manifest:
  - `rust/test_data/ast_shape_contract/rtl_const_expr_v1.json`

## Source Of Truth
- Grammar source:
  - `grammars/rtl_const_expr.ebnf`
- Standalone bootstrap crate:
  - `rtl_const_expr/`
- Public host API:
  - `rust/src/embedding_api.rs`
- Public API contract:
  - `rust/docs/EMBEDDING_API_CONTRACT.md`
- Build-time generated parser discovery:
  - `rust/build.rs`
  - `PGEN_RTL_CONST_EXPR_PARSER_PATH`

## Stable Integration Surface
- Grammar family:
  - `rtl_const_expr`
- Stable generic entry points:
  - `parse_grammar_profile(...)`
  - `parse_grammar_profile_result(...)`
  - `parse_grammar_profile_ast_dump(...)`
- Stable diagnostics:
  - `E_BACKEND_UNAVAILABLE`
  - `E_PARSE_FAILURE`
  - `E_INPUT_TOO_LARGE`
  - `E_INVALID_LIMITS`
  - `E_INVALID_ARGUMENT`
  - `E_UNSUPPORTED_PROFILE`

## Validation / Release Gates
- Per-family book gate:
  - `make -C rust SHELL=/opt/homebrew/bin/bash rtl_const_expr_parser_book_gate`
- AST-shape contract:
  - `cargo test --lib --features generated_parsers rtl_const_expr_ast_shape_contract`

## Schema Versioning

The rtl_const_expr parser carries two version axes:

1. **Parser release version** (`1.0.2`). Tracks the parser library's release identity.
2. **AST-dump schema version** (`2`). Tracks the AST output shape.

| Schema version | First parser release | Notable changes |
|---|---|---|
| 2 | 1.0.2 | **RTL-CE correctness fix (breaking).** Three return-annotation bugs corrected, regenerated + gate-locked: (a) the 10 `binop_chain` levels no longer emit `"<invalid_sequence_access>"` on any operator input — the five multi-token inner operator alternations were lifted into named rules (`equality_op`, `relational_op`, `shift_op`, `additive_op`, `multiplicative_op`) mirroring the proven `systemverilog.ebnf` op-chain idiom, and `rest` is now a clean array of `[op-envelope, operand]` iteration entries; (b) `identifier.text` was `$1` (empty leading `trivia`) → `$2`, now the real name; (c) `based_integer` / `decimal_integer` were unannotated (surfacing the `["", "42"]` envelope) → annotated `-> $2`, so `literal.text` is a clean string. Annotation count `24 → 26` (the two new leaf scalar captures). Same accept set. |
| 1.0.0 | 1.0.1 | **RTL-CE-Slice-1** — initial 24-annotation baseline. Expression hierarchy (conditional + 10-rule binop_chain + unary + primary), literal (2 kinds), identifier all typed. **NOTE:** the binop_chain `rest` shape and `identifier`/`literal` `text` in this baseline were defective — see schema `2` for the correction. |
| 0.1.0 | 1.0.0 | Foundation baseline. Grammar (`grammars/rtl_const_expr.ebnf`) with the `rtl_const_expr -> {type, expr}` root, `unary_expr` per-branch typed shapes, `primary_expr` / `literal` typed shapes, and `identifier -> {type, text}` already in place; the 10 binop-chain rules were the unannotated tail. |

## Release 1.0.2 / Contract 1.0.2 Highlights — RTL-CE correctness fix (binop_chain / identifier / literal); schema 1 → 2

Landed 2026-05-16. A worked-example pass surfaced that the `1.0.1`
baseline shipped three return-annotation defects that the (root-keys-only)
shape-contract regression lock did not catch. All three are fixed,
the parser is regenerated, and the manifest inventory is tightened to
the full 26-entry surface so the corrected shapes are now machine-locked.

- **binop_chain `<invalid_sequence_access>` (Issue A).** For any input
  exercising an operator at any of the 10 levels (e.g. `a + b`), the
  `rest` field emitted the literal string `"<invalid_sequence_access>"`
  inside a malformed nested object. Root cause: the five multi-token
  levels used an inline operator alternation as the iteration's lead
  element (`additive_expr := multiplicative_expr ((plus|minus) multiplicative_expr)* -> {... rest:$2}`),
  which corrupts the positional model. **Fix (proven `systemverilog.ebnf`
  idiom):** the inline alternations are lifted into named rules
  (`equality_op := eqeq | ne`, `relational_op := le | lt | ge | gt`,
  `shift_op := shl | shr`, `additive_op := plus | minus`,
  `multiplicative_op := star | slash | percent`); every level is now
  `next (NAMED_op next)* -> {type:"binop_chain", level, lhs:$1, rest:$2}`
  with bare `$2`. `rest` is now a **clean array** of
  `[op-envelope, operand]` iteration entries (operator text at
  `entry[0][1]`), `[]` when no operator at that level. Verified on
  `a+b`, `a-b`, `a*b`, `a%b`, `a<<b`, `a<b`, `a==b`, `a||b`, `a&&b`,
  `a^b`, `a + b * c - d`, `a ? b + c : d * e`, `(a + b) * c`.
- **`identifier.text` empty (Issue B).** `identifier := trivia /re/ -> {type:"identifier", text:$1}` —
  `$1` was the leading `trivia`, so every identifier `text` was `""`.
  **Fix:** `text: $2`. Verified `x1` → `{"type":"identifier","text":"x1"}`.
- **`literal.text` envelope (Issue C).** `based_integer` /
  `decimal_integer` (`:= trivia /re/`) were unannotated, so
  `literal.text` surfaced the envelope `["", "42"]`. **Fix:** annotate
  both leaves `-> $2`; `literal.text` is now a clean string. Verified
  `42` → `{"type":"literal","kind":"decimal","text":"42"}` and
  `8'hFF` → `kind:"based","text":"8'hFF"`.

Annotation count: **26** (was 24; +2 = the `based_integer` /
`decimal_integer` `-> $2` leaf scalar captures). 18 distinct rules; 19
`return_object` + 7 `return_scalar`. Same accept set (no grammar
acceptance change — purely annotation shaping). Schema bumped `1 → 2`
because `binop_chain.rest`, `identifier.text`, and `literal.text` all
changed shape in consumer-visible ways. Gate-locked:
`cargo test --lib --features generated_parsers rtl_const_expr_ast_shape_contract`
(samples=2 aligned=2, regression_lock_failures=0) and
`make -C rust SHELL=/opt/homebrew/bin/bash rtl_const_expr_parser_book_gate`.

> **Systemic note:** the inline-operator-alternation antipattern that
> caused Issue A also exists in `grammars/rtl_frontend.ebnf` and
> `grammars/vhdl.ebnf` binop_chain levels (same `<invalid_sequence_access>`
> empirically confirmed for rtl_frontend `a + b`). Those families'
> corrections are tracked separately as their own slices + bug-ledger
> entries; this release fixes rtl_const_expr only.

## Release 1.0.1 / Contract 1.0.1 Highlights — RTL-CE-Slice-1: binop_chain hierarchy typed (10 rules / 10 annotations)

Initial slice landed on 2026-05-14 (required the codegen outer-branch remap fix `PGEN-PIP-001` to make Pattern-A `digit ( sep | digit )*` work; see `feedback_codegen_outer_branch_remap.md`). The 10 binary-operator chain rules now emit a uniform left-fold shape:

```ebnf
# File root (pre-existing)
rtl_const_expr           -> {type: "rtl_const_expr", expr}

# Expression hierarchy (10 binop_chain levels, slice 1)
logical_or_expr          -> {type: "binop_chain", level: "logical_or",     lhs, rest}
logical_and_expr         -> {type: "binop_chain", level: "logical_and",    lhs, rest}
bit_or_expr              -> {type: "binop_chain", level: "bit_or",         lhs, rest}
bit_xor_expr             -> {type: "binop_chain", level: "bit_xor",        lhs, rest}
bit_and_expr             -> {type: "binop_chain", level: "bit_and",        lhs, rest}
equality_expr            -> {type: "binop_chain", level: "equality",       lhs, rest}
relational_expr          -> {type: "binop_chain", level: "relational",     lhs, rest}
shift_expr               -> {type: "binop_chain", level: "shift",          lhs, rest}
additive_expr            -> {type: "binop_chain", level: "additive",       lhs, rest}
multiplicative_expr      -> {type: "binop_chain", level: "multiplicative", lhs, rest}

# Pre-existing shapes (foundation baseline)
conditional_expr         -> {type: "ternary", condition, then_expr, else_expr}    | passthrough
unary_expr               -> {type: "unary", op: "plus"|"minus"|"logical_not"|"bit_not", expr}    | passthrough
primary_expr             -> passthrough on literal/identifier; {kind, expr} on lparen-rparen
literal                  -> {type: "literal", kind: "based"|"decimal", text}
identifier               -> {type: "identifier", text}
```

### Consumer guidance: the `binop_chain` shape

All 10 binary-operator chain rules emit the same shape: `{type: "binop_chain", level, lhs, rest}` where:

- `lhs` is the leading operand at this precedence level (typed value from the next-lower level).
- `rest` is the iteration array of `(op, operand)` pairs from `(op X)*`.

To evaluate, consumers fold left:

```pseudocode
value = lhs
for (op, operand) in rest:
    value = apply(op, value, operand)
```

`level` discriminates which operator family the node belongs to (e.g. "logical_or", "additive") so consumers can validate operator-vs-level conformance.

Annotation count: **24** (was 14 / foundation baseline). Same accept set.

## AST Envelope and Expression Hierarchy

This section is the consumer-facing dispatch contract: how a downstream
integrator goes from the host AST-dump call to a typed rtl_const_expr
tree, and how the expression-precedence cascade is shaped. Every shape
below is transcribed from the live inventory
`generated/rtl_const_expr_return_annotations.json`
(`version: 1`, `grammar: "rtl_const_expr"`, `annotation_count: 26`),
cross-checked against the embedded copy in
`rust/test_data/ast_shape_contract/rtl_const_expr_v1.json` (content-identical
on the `(rule, branch_index, annotation_type, normalized_text)` tuples;
the live inventory additionally carries a per-entry `raw_text` field),
and is consistent with the curated per-rule reference at
`docs/rtl_const_expr_parser_book/src/rules-top-level.md`.

### The `AstDumpPayload` envelope

The AST-dump host entry points (the generic
`parse_grammar_profile_ast_dump*` family and the named-result form
`parse_grammar_profile_ast_dump_named`) return — on success — an
`AstDumpPayload` (defined in `rust/src/embedding_api.rs`, contract in
`rust/docs/EMBEDDING_API_CONTRACT.md`). It is a canonical-JSON payload
string plus truncation metadata, with exactly four fields:

| Field | Type | Meaning |
|---|---|---|
| `dump_json` | string | The canonical (key-sorted) JSON encoding of the typed rtl_const_expr AST. Parse this string to obtain the `rtl_const_expr` root object described below. |
| `truncated` | bool | `false` for a complete dump; `true` when `max_ast_bytes` was exceeded and `dump_json` instead carries the truncation diagnostic envelope. |
| `full_bytes` | int | Byte length of the full encoded AST payload (before any truncation). |
| `emitted_bytes` | int | Byte length actually placed in `dump_json`. Equals `full_bytes` when not truncated. |

When `truncated` is `true`, `dump_json` is replaced by a deterministic
truncation diagnostic envelope (not the AST). That envelope carries
`pgen_dump_contract_version` (currently `1`), `kind:
"pgen_ast_dump_truncation"`, `truncated: true`, `dump_kind:
"parser_return_ast"`, `max_bytes`, `full_bytes`, and `reason`. Consumers
must check `truncated` (or, equivalently, the presence of
`pgen_dump_contract_version` / `kind == "pgen_ast_dump_truncation"` in
the parsed `dump_json`) before treating `dump_json` as an
rtl_const_expr AST. If `max_ast_bytes` is too small to fit even the
diagnostic envelope, the API returns `E_INVALID_LIMITS` instead.

> Accuracy note: the live `AstDumpPayload` struct exposes precisely
> `dump_json` / `truncated` / `full_bytes` / `emitted_bytes`. The
> `pgen_dump_contract_version` / `schema_version` / `grammar` / `profile` /
> `root` keys are **not** members of `AstDumpPayload` itself —
> `pgen_dump_contract_version` appears only inside the truncation diagnostic
> envelope, the schema axis is the **AST-dump schema version `2`** tracked in
> [Schema Versioning](#schema-versioning), the grammar family is the fixed
> `rtl_const_expr` label, and the profile is the fixed `default` profile (see
> [Stable Integration Surface](#stable-integration-surface)). The "root" is
> the parsed `rtl_const_expr` object documented next. This contract documents
> the surface as it exists in `rust/src/embedding_api.rs`, not an idealized
> envelope.

### The `rtl_const_expr` root

The parsed `dump_json` is, for a successful rtl_const_expr parse, a
single typed root object. Per `grammars/rtl_const_expr.ebnf` (lines
11–12):

```ebnf
rtl_const_expr := conditional_expr
              -> {type: "rtl_const_expr", expr: $1}
```

```json
{
  "type": "rtl_const_expr",
  "expr": { /* conditional_expr shape */ }
}
```

Consumers dispatch on `obj["type"] == "rtl_const_expr"` at the root,
then descend into `obj["expr"]` — the single typed child, whatever shape
`conditional_expr` produced. This is the only rule whose `type` is
`"rtl_const_expr"`; every interior expression node carries a different
`type`. rtl_const_expr uses a `type` discriminator on every typed
object — there is **no** bare `kind` dispatcher anywhere in this
grammar.

### The expression hierarchy

Below the root, the grammar is an expression-precedence cascade:
`conditional_expr` (the ternary head) → a **ten-level** `binop_chain`
cascade → `unary_expr` (the prefix tier) → `primary_expr` (the
all-passthrough leaf dispatcher). The four families and their exact
field lists, transcribed from the live inventory:

#### `conditional_expr` — the ternary head (2 branches)

`grammars/rtl_const_expr.ebnf` lines 14–17:

```ebnf
conditional_expr := logical_or_expr question conditional_expr colon conditional_expr
                 -> {type: "ternary", condition: $1, then_expr: $3, else_expr: $5}
conditional_expr := logical_or_expr
                 -> $1
```

| Branch | `annotation_type` | Shape |
|---|---|---|
| 0 | `return_object` | `{type: "ternary", condition: <logical_or_expr>, then_expr: <conditional_expr>, else_expr: <conditional_expr>}` — the `c ? t : e` form; both arms recurse into `conditional_expr`, so a ternary can nest. |
| 1 | `return_scalar` (`-> $1`) | **Passthrough.** A non-ternary expression surfaces directly as its `logical_or_expr` `binop_chain` with no `conditional_expr` wrapper. |

Because branch 1 is passthrough, any expression slot
(`rtl_const_expr.expr`, a ternary arm, a parenthesized sub-expression)
may hold a `ternary` object **or** directly a `binop_chain` — dispatch
on `obj["type"]`.

#### The ten-level `binop_chain` cascade

`grammars/rtl_const_expr.ebnf` lines 19–38. Each of the **ten** binary
levels emits the identical shape
`{type: "binop_chain", level, lhs: $1, rest: $2}`. The five multi-token
inner operator alternations are lifted into **named, un-annotated**
rules (`equality_op := eqeq | ne`, `relational_op := le | lt | ge | gt`,
`shift_op := shl | shr`, `additive_op := plus | minus`,
`multiplicative_op := star | slash | percent` — lines 40–44); the other
five levels use a single named token rule. This is the `1.0.2`
correctness fix (the proven `systemverilog.ebnf` op-chain idiom) that
made `rest: $2` capture cleanly. The five named operator rules carry
**no** annotation and are therefore **not** in the 26-entry inventory.

```ebnf
logical_or_expr     := logical_and_expr ( logical_or  logical_and_expr )*
                    -> {type: "binop_chain", level: "logical_or",     lhs: $1, rest: $2}
multiplicative_expr := unary_expr       ( multiplicative_op unary_expr )*
                    -> {type: "binop_chain", level: "multiplicative", lhs: $1, rest: $2}
```

| `level` | Rule (`grammars/rtl_const_expr.ebnf`) | Operators (loosest binding first; grammar/precedence order) |
|---|---|---|
| `"logical_or"` | `logical_or_expr` (line 19) | `\|\|` |
| `"logical_and"` | `logical_and_expr` (line 21) | `&&` |
| `"bit_or"` | `bit_or_expr` (line 23) | `\|` |
| `"bit_xor"` | `bit_xor_expr` (line 25) | `^` |
| `"bit_and"` | `bit_and_expr` (line 27) | `&` |
| `"equality"` | `equality_expr` (line 29) | `==` / `!=` (`equality_op`) |
| `"relational"` | `relational_expr` (line 31) | `<=` / `<` / `>=` / `>` (`relational_op`) |
| `"shift"` | `shift_expr` (line 33) | `<<` / `>>` (`shift_op`) |
| `"additive"` | `additive_expr` (line 35) | `+` / `-` (`additive_op`) |
| `"multiplicative"` | `multiplicative_expr` (line 37) | `*` / `/` / `%` (`multiplicative_op`) |

Every level carries exactly the two fields `lhs` and `rest` (plus
`type` and `level`). There is **no** `sign` field on any rtl_const_expr
level (unlike VHDL's `simple_expression`); the prefix `+` / `-` / `!` /
`~` operators live in `unary_expr`, below `multiplicative_expr`.

**Consumer left-fold rule (normative).** `lhs` (`$1`) is the leading
operand at this precedence level — the typed value from the next-tighter
level, bottoming out at a `unary_expr` → passthrough `primary_expr`
leaf. `rest` (`$2`) is, **as of the `1.0.2` correctness fix, a clean
array** of per-iteration entries for the `( <op> <next> )*` tail (never
the pre-`1.0.2` `"<invalid_sequence_access>"`), `[]` when there was no
operator at that level. **Each entry is a two-element array
`[ <op-envelope>, <operand> ]`:** `entry[0]` is the operator envelope
(for a `trivia "<tok>"` operator token this is the two-element array
`["", "<tok>"]`; the operator text is at `entry[0][1]`) and `entry[1]`
is the next-level operand (itself a `binop_chain`). It is **not** a
typed `{op, rhs}` object. Consumers MUST fold `rest`
left-associatively onto `lhs` — evaluate `lhs`, then for each entry in
array order apply the `entry[0]` operator with the running result as the
left side and `entry[1]` as the right side. This left-fold is identical
at all ten levels by construction, so one fold routine walks the whole
binary-expression tree; `level` discriminates which operator family the
node belongs to so consumers can validate operator-vs-level conformance.

#### `unary_expr` — the prefix tier (5 branches)

`grammars/rtl_const_expr.ebnf` lines 46–55:

```ebnf
unary_expr := plus  unary_expr -> {type: "unary", op: "plus",        expr: $2}
unary_expr := minus unary_expr -> {type: "unary", op: "minus",       expr: $2}
unary_expr := bang  unary_expr -> {type: "unary", op: "logical_not", expr: $2}
unary_expr := tilde unary_expr -> {type: "unary", op: "bit_not",     expr: $2}
unary_expr := primary_expr     -> $1
```

| Branch | `annotation_type` | Shape |
|---|---|---|
| 0 | `return_object` | `{type: "unary", op: "plus", expr: <unary_expr>}` (prefix `+`) |
| 1 | `return_object` | `{type: "unary", op: "minus", expr: <unary_expr>}` (prefix `-`) |
| 2 | `return_object` | `{type: "unary", op: "logical_not", expr: <unary_expr>}` (prefix `!`) |
| 3 | `return_object` | `{type: "unary", op: "bit_not", expr: <unary_expr>}` (prefix `~`) |
| 4 | `return_scalar` (`-> $1`) | **Passthrough.** A non-prefixed operand surfaces directly as its `primary_expr` shape with no `unary` wrapper. |

The four operator branches carry the two fields `op` (the typed
operator name — `"plus"`, `"minus"`, `"logical_not"`, `"bit_not"`, not
the raw token) and `expr` (recursing into `unary_expr`, so `--x` /
`!~x` chains nest).

#### `primary_expr` — the all-passthrough leaf dispatcher (3 branches)

`grammars/rtl_const_expr.ebnf` lines 57–62. All three branches are
`return_scalar` passthroughs, so `primary_expr` never appears as its own
wrapper object in the dump:

```ebnf
primary_expr := literal                        -> $1
primary_expr := identifier                     -> $1
primary_expr := lparen conditional_expr rparen -> $2
```

| Branch | `annotation_type` | Shape |
|---|---|---|
| 0 | `return_scalar` (`-> $1`) | **Passthrough** to the `literal` typed object. |
| 1 | `return_scalar` (`-> $1`) | **Passthrough** to the `identifier` typed object. |
| 2 | `return_scalar` (`-> $2`) | **Passthrough** to the parenthesized inner `conditional_expr`; the surrounding `( )` tokens are dropped. |

The two typed leaves it bottoms out at (`grammars/rtl_const_expr.ebnf`
lines 64–74): `literal := based_integer -> {type: "literal", kind:
"based", text: $1}` / `decimal_integer -> {type: "literal", kind:
"decimal", text: $1}` (two `kind` values `"based"` / `"decimal"`), and
`identifier := trivia /…/ -> {type: "identifier", text: $2}`. As of the
`1.0.2` fix `literal.text` and `identifier.text` are both clean source
strings: `based_integer` / `decimal_integer` are now annotated `-> $2`
(were unannotated, surfacing the envelope `["", "42"]`), and
`identifier` binds `text: $2` (was `$1`, the empty leading `trivia`).

The full per-branch field lists, the named-operator-rule table, and the
worked left-fold are enumerated in
`docs/rtl_const_expr_parser_book/src/rules-top-level.md`; the
machine-checkable enumeration of every `(rule, branch_index,
annotation_type, normalized_text)` tuple is
`generated/rtl_const_expr_return_annotations.json` and its embedded copy
`rust/test_data/ast_shape_contract/rtl_const_expr_v1.json`. The full
typed surface of contract `1.0.2` is **26 return annotations across 18
distinct rules** (19 `return_object` + 7 `return_scalar`; 10 of the 18
rules emit `binop_chain`), AST-dump schema version `2`. If this section
and either artifact disagree, the artifact wins; this integration
contract wins over the per-family mdBook.

### Literal and Identifier Leaf Shapes

The expression cascade bottoms out at two typed leaf objects — `literal`
and `identifier` — reached through the all-passthrough `primary_expr`
dispatcher. These are the only nodes a consumer's recursive descent
ever terminates at; every other typed node holds typed children. Both
shapes are transcribed from the live inventory
`generated/rtl_const_expr_return_annotations.json` and cross-checked
against the embedded manifest
`rust/test_data/ast_shape_contract/rtl_const_expr_v1.json`.

#### `literal` — the integer-literal leaf (2 branches)

`grammars/rtl_const_expr.ebnf` lines 64–67:

```ebnf
literal := based_integer   -> {type: "literal", kind: "based",   text: $1}
literal := decimal_integer -> {type: "literal", kind: "decimal", text: $1}
```

| Branch | `annotation_type` | `kind` | Shape |
|---|---|---|---|
| 0 | `return_object` | `"based"` | `{type: "literal", kind: "based", text: <string>}` — a sized/based integer (e.g. `8'hFF`, `4'b1010`, `'d255`). |
| 1 | `return_object` | `"decimal"` | `{type: "literal", kind: "decimal", text: <string>}` — a plain decimal integer (e.g. `42`, `1_000`). |

Both branches emit exactly the three fields `type` (always
`"literal"`), `kind` (the two-valued discriminator `"based"` /
`"decimal"`), and `text`. Consumers dispatch on `obj["type"] ==
"literal"` then branch on `obj["kind"]` to choose decimal-vs-based
numeric parsing of `obj["text"]`.

`text` is bound `$1` — the matched `based_integer` / `decimal_integer`
sub-rule value. **As of the `1.0.2` correctness fix, `text` is a clean
source string**, not the pre-`1.0.2` envelope `["", "42"]`: the leaf
rules `based_integer := trivia /…/ -> $2` and `decimal_integer :=
trivia /…/ -> $2` (`grammars/rtl_const_expr.ebnf` lines 69–72) are now
annotated `-> $2`, so the `$1` `literal.text` resolves to the trimmed
literal text. Verified `42` → `{"type":"literal","kind":"decimal","text":"42"}`
and `8'hFF` → `{"type":"literal","kind":"based","text":"8'hFF"}`.
`based_integer` and `decimal_integer` carry their own `return_scalar`
`-> $2` annotations — these are 2 of the 26 inventory entries (the +2
that took the count `24 → 26`).

#### `identifier` — the name leaf (1 branch)

`grammars/rtl_const_expr.ebnf` lines 73–74:

```ebnf
identifier := trivia /[_A-Za-z][_A-Za-z0-9$]*(?:(?:\.)[…]|::[…])*/
           -> {type: "identifier", text: $2}
```

| Branch | `annotation_type` | Shape |
|---|---|---|
| 0 | `return_object` | `{type: "identifier", text: <string>}` — a bare, dotted (`a.b`), or package-qualified (`pkg::name`) identifier; the whole matched name is one `text` string. |

`identifier` emits exactly the two fields `type` (always
`"identifier"`) and `text`. **As of the `1.0.2` correctness fix `text`
binds `$2`** — the regex-matched name — not the pre-`1.0.2` `$1`, which
was the empty leading `trivia` (so every identifier `text` was `""`).
Verified `x1` → `{"type":"identifier","text":"x1"}`. Dotted /
scope-resolution names are **not** split: `top.sub.WIDTH` and
`pkg::PARAM` each surface as a single `text` string; consumers that need
the path segments split them themselves.

Both leaves are reached only through `primary_expr`'s branch 0
(`literal`) / branch 1 (`identifier`) passthroughs (`-> $1`); a
parenthesized sub-expression (`primary_expr` branch 2, `-> $2`) yields
the inner `conditional_expr` shape directly, never a `literal` /
`identifier` wrapper for the parentheses themselves.

## Scope / Non-Goals
- The stable downstream contract is the host-oriented embedding API, not internal generated parser modules or internal AST types.
- `rtl_const_expr` covers only **constant expressions** (decimal and sized-based integer literals, identifiers, unary `+ - ! ~`, binary arithmetic / shift / comparison / equality / bitwise / logical operators, ternary `?:`). For statements, modules, control flow → see `rtl_frontend`.
- When reporting downstream bugs, follow `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`; accepted released-parser bugs should then be logged in `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`.

## Companion Documentation — rtl_const_expr Parser Integration mdBook

This contract is the **downstream integration surface**: the host-API
envelope, the dispatch/expression-hierarchy shapes a consumer compiles
against, and the release/schema axes. It does not duplicate the per-rule
walkthroughs or worked examples — those live in the companion artifacts
below. Each surface is authoritative for a different thing; consult the
matching one and respect the precedence order stated at the end of this
section.

| Surface | Path | Authoritative for |
|---|---|---|
| **This contract** | `docs/contracts/PGEN_RTL_CONST_EXPR_PARSER_INTEGRATION_CONTRACT.md` | The downstream integration surface: AST-dump envelope, `rtl_const_expr` root, the conditional → ten-level `binop_chain` → unary → primary expression hierarchy, and the `literal` / `identifier` leaf shapes. See [AST Envelope and Expression Hierarchy](#ast-envelope-and-expression-hierarchy) and [Literal and Identifier Leaf Shapes](#literal-and-identifier-leaf-shapes). |
| **Per-parser mdBook** | `docs/rtl_const_expr_parser_book/` (source `src/*.md`; tracked HTML at `docs/rtl_const_expr_parser_book-html/`) | The per-rule reference and teaching surface: build recipe, public API, AST-envelope walkthrough, every rule shape, per-feature worked examples, schema-versioning timeline, glossary, changelog index. Curated, not machine-checked. Listed in `README.md` § "Per-Parser Integration Reference Books". |
| **Shape-contract manifest** | `rust/test_data/ast_shape_contract/rtl_const_expr_v1.json` | The machine-checkable shape lock embedded in the regression test. Content-identical to the live inventory on the `(rule, branch_index, annotation_type, normalized_text)` tuples (the embedded copy omits only the diagnostic `raw_text` field). Drift fails the AST-shape-contract test. |
| **Declared-annotation inventory** | `generated/rtl_const_expr_return_annotations.json` | The live machine-checkable enumeration of every typed-shape annotation the rtl_const_expr grammar emits (`version: 1`, `grammar: "rtl_const_expr"`, `annotation_count: 26`, **18 distinct rules**; 19 `return_object` + 7 `return_scalar`). The generator-side source of truth for the typed surface. |
| **Embedding-API contract** | `rust/docs/EMBEDDING_API_CONTRACT.md` | The canonical host-API truth: the `AstDumpPayload` struct (`dump_json` / `truncated` / `full_bytes` / `emitted_bytes`), the entry-point signatures, the truncation diagnostic envelope, and the stable diagnostics. The struct shape this contract documents is transcribed from there. |
| **Released-parser bug ledger** | `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md` | The accepted-bug log for the released rtl_const_expr parser. Consult before integrating around a suspected parser defect; file new accepted bugs here per `docs/contracts/PGEN_PARSER_ISSUE_REPORTING_PROTOCOL.md`. |

Precedence when surfaces disagree (highest first): the **embedding-API
contract** (`rust/docs/EMBEDDING_API_CONTRACT.md`) wins for the host-API /
`AstDumpPayload` truth; the **declared-annotation inventory**
(`generated/rtl_const_expr_return_annotations.json`) and its embedded
shape-contract manifest copy win for the exact typed-shape enumeration;
**this integration contract** wins over the **per-parser mdBook** for
downstream compliance. Report any disagreement as a documentation bug
rather than silently coding to the lower-precedence surface.

### Gate Recipe

The exact, copy-pasteable per-family commands a downstream integrator or
releaser runs. Each is verified against the repo (`rust/Makefile`,
`docs/rtl_const_expr_parser_book/src/build-recipe.md`,
`rust/src/ast_shape_contract.rs`); none are invented — do not substitute
flags.

**1. On-demand parser regen.** The rtl_const_expr parser is
on-demand-only (not in the default `cargo test --features
generated_parsers` build). Build `ast_pipeline`, then regenerate the
parser from `grammars/rtl_const_expr.ebnf` (run from `rust/`, per
`docs/rtl_const_expr_parser_book/src/build-recipe.md` § "Cold-clone
build"):

```bash
cd rust && cargo build --release --features ebnf_dual_run --bin ast_pipeline
./target/release/ast_pipeline ../grammars/rtl_const_expr.ebnf \
    --generate-parser --output ../generated/rtl_const_expr_parser.rs
```

To wire the regenerated parser into a cargo build, point
`PGEN_RTL_CONST_EXPR_PARSER_PATH` at the absolute path of the generated
file before `cargo build --release --features generated_parsers` (see
`docs/rtl_const_expr_parser_book/src/build-recipe.md` § "Wiring into a
downstream Cargo build").

**2. Per-family book gate.** Builds the rtl_const_expr parser book and
verifies the tracked HTML landing pages (Makefile target
`rtl_const_expr_parser_book_gate`, `rust/Makefile` line 745):

```bash
make -C rust SHELL=/opt/homebrew/bin/bash rtl_const_expr_parser_book_gate
```

**3. AST-shape-contract regression lock.** With the generated backend
wired in (`PGEN_RTL_CONST_EXPR_PARSER_PATH` exported), run the
shape-contract test that diffs the running generated parser against
`rust/test_data/ast_shape_contract/rtl_const_expr_v1.json` (test fn
`rtl_const_expr_ast_shape_contract_holds_against_running_generated_parser`
in the `pgen::ast_shape_contract` library module,
`rust/src/ast_shape_contract.rs` line 741):

```bash
cargo test --lib --features generated_parsers rtl_const_expr_ast_shape_contract
```

The substring `rtl_const_expr_ast_shape_contract` selects exactly the
`rtl_const_expr_ast_shape_contract_holds_against_running_generated_parser`
test. Any drift between the running parser's emitted shapes and the
locked manifest fails this test, surfacing the change before release
(the `1.0.2` correctness fix is locked here: samples=2 aligned=2,
regression_lock_failures=0).

**4. Validation / release gates.** Anyone publishing a parser-release
version bump also runs the per-family gates enumerated in
[Validation / Release Gates](#validation--release-gates) (the per-family
book gate and the AST-shape-contract test above). That section is the
canonical list; it is not repeated here.

## Glossary

Contract-scoped definitions of the terms a downstream integrator needs to
read this document. Where a term has a normative definition, this contract
is authoritative; the per-parser book's
[glossary](../rtl_const_expr_parser_book/src/glossary.md) paraphrases the
same terms for quick lookup. Numbers below are pinned to contract `1.0.2` /
AST-dump schema `2` / parser release `1.0.2` / **26 annotations across 18
distinct rules** (19 `return_object` + 7 `return_scalar`; 10 `binop_chain`).

- **`AstDumpPayload`** — the success return of the rtl_const_expr
  AST-dump host entry points (defined in `rust/src/embedding_api.rs`,
  contract in `rust/docs/EMBEDDING_API_CONTRACT.md`). A canonical-JSON
  payload string plus truncation metadata, with **exactly four fields**:
  `dump_json`, `truncated`, `full_bytes`, `emitted_bytes`. It does
  **not** carry `root` / `schema_version` / `grammar` / `profile`
  members — see [The `AstDumpPayload` envelope](#the-astdumppayload-envelope)
  for the precise accuracy note.
- **`dump_json`** — the `AstDumpPayload` field holding the canonical
  (key-sorted) JSON encoding of the typed rtl_const_expr AST. Parse this
  string to obtain the `rtl_const_expr` root object. When `truncated`
  is `true` this string is replaced by the truncation diagnostic
  envelope, not the AST.
- **Truncation diagnostic envelope** — the deterministic JSON object
  that replaces the AST in `dump_json` when `max_ast_bytes` is exceeded.
  It carries `pgen_dump_contract_version` (currently `1`), `kind:
  "pgen_ast_dump_truncation"`, `truncated: true`, `dump_kind:
  "parser_return_ast"`, `max_bytes`, `full_bytes`, and `reason`.
  Consumers must check `truncated` (or detect `kind ==
  "pgen_ast_dump_truncation"`) before treating `dump_json` as an
  rtl_const_expr AST.
- **AST-dump schema version** — the integer version axis tracking the
  AST output shape, currently `2`, pinned by this contract (see
  [Schema Versioning](#schema-versioning)). It is **not** a field of
  `AstDumpPayload`; it is the contract-tracked axis. It was bumped
  `1 → 2` by the `1.0.2` correctness fix because `binop_chain.rest`,
  `identifier.text`, and `literal.text` all changed shape in
  consumer-visible ways. Pure perf work / internal codegen
  reorganization do not bump it.
- **Parser release version** — the parser library's release identity,
  currently `1.0.2`. Bumped on every functional change (bug fixes, perf
  work, grammar changes). Moves independently of the schema version; the
  `1.0.2` release carries AST-dump schema `2`.
- **Conditional / `binop_chain` / unary / primary expression
  hierarchy** — rtl_const_expr's expression-precedence cascade:
  `conditional_expr` (the ternary head) → a **ten-level** `binop_chain`
  cascade → `unary_expr` (the prefix tier) → `primary_expr` (the
  all-passthrough leaf dispatcher) bottoming out at the `literal` /
  `identifier` leaves. `conditional_expr` (non-`?`) and `unary_expr`
  (non-prefixed) and all three `primary_expr` branches are passthroughs,
  so any expression slot may directly hold a `ternary`, a `binop_chain`,
  a `unary`, a `literal`, or an `identifier` — dispatch on `obj["type"]`.
  See [The expression hierarchy](#the-expression-hierarchy).
- **Ten-level `binop_chain` left-fold** — the consumer-facing contract
  for rtl_const_expr's ten-level operator-precedence cascade
  (`logical_or_expr` → `logical_and_expr` → `bit_or_expr` →
  `bit_xor_expr` → `bit_and_expr` → `equality_expr` → `relational_expr`
  → `shift_expr` → `additive_expr` → `multiplicative_expr`). Every level
  emits `{type: "binop_chain", level, lhs: $1, rest: $2}` (10 rules / 10
  annotations, one per rule); `rest` is, as of `1.0.2`, a **clean
  array** of `[op-envelope, operand]` iteration entries (`[]` when no
  operator at that level), never the pre-`1.0.2`
  `"<invalid_sequence_access>"`. Consumers MUST fold `rest`
  left-associatively onto `lhs`. There is **no `sign` field** on any
  level — prefix operators are factored into the separate `unary_expr`
  tier below `multiplicative_expr`. One fold routine walks the whole
  binary-expression tree. See
  [The ten-level `binop_chain` cascade](#the-ten-level-binop_chain-cascade).
- **Named operator rules** — the five `1.0.2` op-chain helper rules
  `equality_op := eqeq | ne`, `relational_op := le | lt | ge | gt`,
  `shift_op := shl | shr`, `additive_op := plus | minus`,
  `multiplicative_op := star | slash | percent` (`grammars/rtl_const_expr.ebnf`
  lines 40–44). Lifting the previously-inline multi-token operator
  alternations into these named rules (the proven `systemverilog.ebnf`
  idiom) is what made `binop_chain.rest: $2` capture cleanly. They carry
  **no** return annotation and are therefore **not** among the 26
  inventory entries.
- **`literal` / `identifier` leaves** — the two typed leaf objects the
  expression cascade terminates at, reached through `primary_expr`'s
  passthrough branches. `literal` is `{type: "literal", kind, text}`
  with `kind` ∈ `{"based", "decimal"}`; `identifier` is
  `{type: "identifier", text}`. As of `1.0.2` both `text` fields are
  clean source strings (`based_integer` / `decimal_integer` annotated
  `-> $2`; `identifier` binds `text: $2`). See
  [Literal and Identifier Leaf Shapes](#literal-and-identifier-leaf-shapes).
- **Shape-contract manifest** — the embedded machine-checkable shape lock
  `rust/test_data/ast_shape_contract/rtl_const_expr_v1.json`.
  Content-identical to the declared-annotation inventory on the
  `(rule, branch_index, annotation_type, normalized_text)` tuples (omits
  only the diagnostic `raw_text` field). Drift fails the
  `rtl_const_expr_ast_shape_contract_holds_against_running_generated_parser`
  regression test (see [Gate Recipe](#gate-recipe)).
- **Declared-annotation inventory** — the live machine-checkable
  enumeration of every typed-shape annotation the rtl_const_expr grammar
  emits: `generated/rtl_const_expr_return_annotations.json`
  (`version: 1`, `grammar: "rtl_const_expr"`, `annotation_count: 26`,
  **18 distinct rules**; 19 `return_object` + 7 `return_scalar`). The
  generator-side source of truth for the typed surface; mirrored by the
  embedded shape-contract manifest copy. (The `version: 1` field is the
  inventory-file format version, distinct from the AST-dump schema
  version `2` and the parser release version `1.0.2`.)
- **Recursive envelope** — the default JSON shape produced by
  un-annotated rules: a recursive composition of arrays (sequences,
  quantified iterations, the `rest` tail of an op-chain), strings
  (terminal/regex leaves), and matched-branch passthroughs (for
  alternations). Un-matched optionals are the empty array `[]`, never
  `null`. It is what a consumer reaches when descending below the typed
  surface — e.g. each `binop_chain.rest` iteration entry is a two-element
  `[op-envelope, operand]` envelope (the operator token surfaces as the
  two-element array `["", "<tok>"]`, text at `entry[0][1]`), not a typed
  `{op, rhs}` object; and the five un-annotated named operator rules.
- **Generic host AST-dump surface** — the
  `parse_grammar_profile_ast_dump*` family
  (`parse_grammar_profile_ast_dump`, the `*_result` and `*_named`
  forms). The grammar-agnostic entry points that, for the
  `rtl_const_expr` grammar + `default` profile, return the
  `AstDumpPayload`. rtl_const_expr has **no** named-convenience entry
  point (unlike VHDL's `parse_vhdl_1076_2019_ast_dump`); the generic
  family used with grammar family `rtl_const_expr` / profile `default`
  is the integration surface. Signatures are in
  `rust/docs/EMBEDDING_API_CONTRACT.md`; the stable entry-point list is
  in [Stable Integration Surface](#stable-integration-surface).
