# Glossary

Terms used throughout this book. The deep references are the
[return_annotation](../../return_annotation_parser_book/src/welcome.md) and
[semantic_annotation](../../semantic_annotation_parser_book/src/welcome.md) books, `TOOLBOX.md`, and the
docs cross-linked from [Welcome](welcome.md).

- **EBNF (PGEN dialect)** — the grammar language documented by this book, defined by the self-hosting
  meta-grammar `grammars/ebnf.ebnf` (version `2.0`) and the Rust frontend `rust/src/ebnf_frontend.rs`.

- **Rule** — `name := expression -> return_annotation`. The unit of a grammar; compiles to one parser
  function. See [Grammar File Structure](grammar-file-structure.md).

- **Rule operator** — `:=` / `::=` / `=` / `:-`, interchangeable at parse time.

- **Terminal** — a literal matcher: string/char literal, regex literal `/…/`, or an `any_char` built-in.
  See [Terminals](terminals.md).

- **`any_char` / `ascii_char`** — native single-character matchers (also `builtin_any_char` /
  `builtin_ascii_char`) that need no regex engine. The basis of regex-engine-independent grammars.

- **Ordered choice (`|`)** — PEG alternation: tries branches left-to-right, commits to the first match.
  Ordering matters. See [Rules and Expressions](rules-and-expressions.md).

- **Quantifier** — repetition postfix `?` `*` `+` `{n}` `{n,m}` `{n,}` `{,m}`, all handled by the Layer-0
  unified engine. See [Quantifiers](quantifiers.md).

- **Optional `[ … ]`** — element-level `( … )?`. **Not** a character class (those live inside `/…/`) and
  **not** a return-annotation array (those live inside `-> …`). See [Terminals](terminals.md).

- **Lookahead** — zero-width assertion: `&X` (must match) / `!X` (must not). See [Lookaheads](lookaheads.md).

- **Lexical follow-restriction** — `[> X ]` / `[>! X ]`: the rule's match must / must-not be followed by
  `X`. A lexical annotation; see [Lookaheads](lookaheads.md) and the platform book's
  [Lexical Annotations](../../book/src/lexical-annotations.md).

- **Return annotation (`-> …`)** — the AST-shaping language: references (`$N`, `$text`/`$0`, `.field`,
  `[i]`), literals, arrays, objects, and the list operators (`$N*`, `$N**`, `$N::n*`). See
  [Return Annotations](return-annotations.md).

- **Implicit / passthrough return** — what a rule returns with **no** `-> …`: a synthetic `$1` for a
  single-element body, raw passthrough otherwise. See
  [The Implicit / Passthrough Return Policy](return-policy.md).

- **`$text` / `$0`** — the whole matched substring of a rule (`$0` is the Perl5 alias).

- **Semantic annotation (`@name: value`)** — the steering language: `@predicate`, `@emit_fact`,
  `@profiles`, `@transform`, … Binds to the following rule. See [Semantic Annotations](semantic-annotations.md).

- **Semantic store** — the fact/scope state semantic annotations read and write to make a parser
  context-aware.

- **Profile** — a named grammar variant (e.g. `sv_2017`, `sv_2023`, `relaxed`) selected with
  `--grammar-profile`; rules can be restricted to profiles with `@profiles`.

- **Include** — `include(…)` / `include_file(…)` / `include_dir(…)` modular composition. See
  [The Include System](includes.md).

- **Bootstrap mode** — the built-in flat-only annotation parser that breaks the annotation grammars'
  chicken-and-egg. See [The Bootstrap Path](bootstrap.md).

- **`--lint-grammar`** — static well-formedness report (left-recursion, non-termination, shadowing).

- **Certificate coverage** — the per-rule proof/witness/UNKNOWN trustworthiness report; `UNKNOWN=0` at
  seeds `0/7/42` is the objective trust number. See [Codegen Mental Model](codegen-model.md).

- **Aspirational construct** — a construct the meta-grammar self-describes but the codegen does **not**
  implement (parametric rules, templates, lexer modes, inheritance, import, error productions, `{? ?}`,
  action blocks, `~` case control, `@N%`). Do not use them. See [Rules and Expressions](rules-and-expressions.md).
