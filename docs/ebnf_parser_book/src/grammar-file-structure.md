# Grammar File Structure

A PGEN grammar is a plain-text `.ebnf` file. At the top level it is a sequence of **rules**, optionally
interleaved with **include directives**, **semantic annotations**, **comments**, and **whitespace**
(`grammars/ebnf.ebnf`, rule `grammar_file`). The order of rules does not matter for resolution — every
rule name in the file is in scope for every other rule — but the **first reachable rule from the entry
rule** anchors the grammar (you choose the entry with `--entry-rule`).

## A rule

The core unit is a rule definition (`grammars/ebnf.ebnf`, rule `rule_definition`):

```ebnf
rule_name  :=  expression  -> return_annotation
```

- `rule_name` is an identifier matching `[a-zA-Z_][a-zA-Z0-9_]*`.
- `:=` is the rule operator (see below).
- `expression` is the rule body — terminals, rule references, ordered choice, sequences, quantifiers,
  grouping (see [Rules and Expressions](rules-and-expressions.md)).
- `-> return_annotation` is **optional**; when omitted, the rule returns its body under the
  [implicit / passthrough policy](return-policy.md).

A rule may be preceded by one or more `@…` [semantic annotations](semantic-annotations.md) that steer how
it is generated:

```ebnf
@emit_fact: type_name
type_declaration := "typedef" data_type identifier ";"
```

## Rule operators

PGEN's EBNF frontend accepts **four** rule operators, and treats them as **interchangeable** at parse time
(`rust/src/ebnf_frontend.rs`). The semantic distinctions some EBNF dialects attach to them are *not*
modelled — pick one and be consistent within a grammar.

| Operator | Name | Used by shipped grammars |
| --- | --- | --- |
| `:=` | standard EBNF assignment | dominant — e.g. ~1480 rules in `grammars/systemverilog.ebnf` |
| `::=` | alternative assignment | yes — 8 rules in `grammars/systemverilog.ebnf` |
| `=` | simple assignment | yes — `grammars/regex.ebnf` (e.g. `control_escape = "c" any_char -> …`) |
| `:-` | Prolog-style assignment | accepted; not used by a shipped grammar |

The convention across most of the tracked grammars is `:=`; `grammars/regex.ebnf` consistently uses `=`.
Both are equally valid.

## Comments

Comments are ordinary top-level content and may appear between rules or at the end of a line:

```ebnf
# line comment (hash form)
// line comment (slash form)

/* block comment */
```

The hash form (`#`) is by far the most common in the tracked grammars; the shipped grammars use it for the
header banners and per-rule rationale you see throughout `grammars/*.ebnf`. Documentation-comment forms
(`/** … */`, `///`) are recognized by the meta-grammar; the shipped grammars do not rely on them.

## Whitespace

Whitespace (spaces, tabs, newlines) separates tokens and is otherwise insignificant at the grammar-author
level — a rule may span multiple lines, and alternatives are commonly laid out one per line for
readability:

```ebnf
rule_operator := (
    ":=" |
    "::=" |
    "=" |
    ":-"
)
```

> Whitespace *inside the language you are describing* is a different matter — your grammar must match the
> input's own whitespace explicitly (typically via a `trivia`/`whitespace` rule you call between tokens).
> PGEN does not insert implicit token-skipping for you. See how the shipped grammars thread a `trivia`
> rule through their sequences.

## Includes and annotations at the top

Include directives and grammar-level semantic annotations conventionally sit near the top of the file:

```ebnf
# pull in shared rule files
include("tokens", "expressions")

# a grammar-level pragma
@stop_at_rule_boundary: true
sequence := sequence_element+
```

See [The Include System](includes.md) for composition and [Semantic Annotations](semantic-annotations.md)
for the steering directives. The `@stop_at_rule_boundary` pragma is a genuine, codegen-consumed grammar
annotation (it bounds how far a sequence will consume); it appears in `grammars/ebnf.ebnf` itself.
