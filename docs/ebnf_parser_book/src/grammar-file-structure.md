# Grammar File Structure

A PGEN grammar is a plain-text `.ebnf` file. At the top level it is a sequence of **rules**, optionally
interleaved with **include directives**, **semantic annotations**, **comments**, and **whitespace**
(`grammars/ebnf.ebnf`, rule `grammar_file`). Rule order does not affect **name resolution** — every rule
name in the file is in scope for every other rule, so you may reference a rule defined further down. It
does, however, decide one thing, and it is the most important thing in the file: **the entry rule**.

## The entry rule — declare it with `@entry: true`

A grammar's **entry rule** (start symbol) is where parsing begins. Everything follows from it:
reachability analysis, the linter's dead-rule verdicts, certificate coverage and the emitted `parse()`
all anchor on it.

**Declare it by attaching `@entry: true` to the rule itself:**

```ebnf
statement := "a" ";"        # helper rules may live wherever reads best

@entry: true
program := statement+       # ← this rule is the entry
```

The rule is identified by **what the annotation is attached to**, so the payload is just `true` — it
does not name the rule. (A payload is required because PGEN's annotation syntax always takes one; see
[Semantic Annotations](semantic-annotations.md). A bare `@entry` with no payload is *silently ignored*,
so always write `: true`.)

> **A main EBNF file shall contain one and only one `@entry: true`, attached to the proper rule.**
> Declaring it on two rules is a hard error naming both. Writing it *inside* a rule body (branch-start
> or mid-sequence) is also a hard error — it marks a whole rule, so it belongs directly above one.

### ⚠️ Without a declaration, the entry is the FIRST rule you define

If no rule declares `@entry`, PGEN falls back to `rule_order[0]` — **whichever rule appears first in the
file**. That fallback is why declaring the entry matters:

```ebnf
# ✅ entry is `program` — the parser parses a whole program
program := statement+
statement := "a" ";"
```

```ebnf
# ⛔ SAME two rules, reordered — entry is now `statement`.
#    The parser parses exactly ONE statement; `program` is never entered.
#    `a;a;` is REJECTED at position 2. No diagnostic says so.
statement := "a" ";"
program := statement+
```

Both grammars lint clean (`unreachable_rules=0`, exit 0) — an unreferenced rule such as `program` counts
as a *root*, so it is never reported as unreachable. Under the positional fallback, moving a helper rule
above your start symbol therefore changes the language your parser accepts, silently. **Declare
`@entry: true` and file order stops mattering** — which is the point: in EBNF a grammar is a *set* of
productions, so rule order should carry no meaning.

### Checking, and overriding, the entry

`--lint-grammar` reports the resolved entry rule and says which way it was resolved:

```text
$ ast_pipeline grammars/mine.ebnf --lint-grammar
grammar lint: 'mine' (2 rules) — left_recursive=0 …
  [info] entry rule 'program' — DECLARED via `@entry: true`
```

```text
  [info] entry rule 'statement' — POSITIONAL (the first rule defined; declare it with
         `@entry: true` to make file order irrelevant)
```

`--entry-rule RULE` (on `ast_pipeline` and `parseability_probe`) parses from an **alternate** start
symbol and **takes precedence over a declared `@entry`**. It is an out-of-band override for probing and
for entry-relative inspection — not the normal way to select the entry. Naming a rule the grammar does
not define is a hard error, not a silent fallback.

Certificate coverage also prints `entry='…'` on its headline and flags rules with no reach path from it —
but it verifies witnesses through a real generated parser, so it only runs for a **registered** grammar
and refuses an arbitrary `.ebnf` with `no generated parser is registered for grammar '…'`.

See also [Includes](includes.md), where the same rule is what stops an included file from re-rooting your
grammar: main-file rules are always spliced first.

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

> Whitespace *inside the language you are describing* is governed by the grammar's **layout policy**.
> By default a generated parser is whitespace-INSENSITIVE: it automatically skips layout (whitespace
> and unclaimed comment introducers) before each string terminal and regex token, and consumes
> trailing layout after the entry rule — so `start := "a" "b"` accepts `a b` out of the box. A
> whitespace-SENSITIVE language (regex is the canonical example: every space is a literal atom) opts
> out with the grammar-level [`@whitespace_sensitive` directive](semantic-annotations.md#layout-policy--whitespace_sensitive).
> Grammars can additionally thread an explicit `trivia` rule through their sequences when they need
> the skipped layout to be *visible* in the AST or to control it rule-by-rule.

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
