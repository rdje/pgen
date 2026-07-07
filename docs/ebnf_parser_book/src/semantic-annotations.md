# Semantic Annotations

Where a [return annotation](return-annotations.md) (`-> …`) shapes the *value a rule returns*, a
**semantic annotation** (`@name: value`) shapes the *parser-generation behaviour* — gating a rule on facts
already seen, emitting facts into a semantic store, restricting a rule to a profile, transforming a
matched value, or steering stimuli generation. They are the normative mechanism for making a generated
parser **context-aware**.

This chapter is the grammar-author overview. The deep reference — the full `@name: value` value language,
the complete directive catalog, and the semantic-store lifecycle — is the dedicated
[semantic_annotation parser book](../../semantic_annotation_parser_book/src/welcome.md),
`docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md`, and
`docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`.

## Placement

A standalone `@…` directive binds to the rule that **follows** it (authoritative: `grammars/ebnf.ebnf`,
`grammar_rule := annotation_list? rule_definition`). Place the directive directly above its rule; more than
one may stack:

```ebnf
@emit_fact: type_name
type_declaration := "typedef" data_type identifier ";"
```

```ebnf
@predicate: has_fact(type_name, $head)
known_type_identifier := identifier
```

The text after `@` is *semantic-annotation source* — its own small value language (literals, structured
`{…}` values, function calls, and `$ref` references). Its *meaning* per directive is the steering
contract.

## The directive catalog (overview)

The directives the AST pipeline interprets, grouped by what they do:

| Group | Directives | Purpose |
| --- | --- | --- |
| Store: declare | `@fact_kind` | declare a fact kind and its attributes (exportable, …) |
| Store: emit | `@emit_fact`, `@open_scope` | record a fact / open a lexical scope as the parser matches |
| Store: query (gate) | `@predicate` (+ `has_fact` / `lacks_fact` / `fact_attribute_equals` / `len_bounds` / `numeric_bounds` / `fact_count_at_least` / …) | accept or reject a rule based on the store |
| Store: compose | `@predicate_def` | name a reusable composed predicate |
| Cross-file | `@export_to_library`, `@import_from_library` | publish / consume facts across compilation artifacts |
| Profiles | `@profiles` | restrict a rule to a grammar profile (e.g. `sv_2017` vs `sv_2023`) |
| Value | `@transform`, `@semantic_value` | post-process a matched value |
| Stimuli | `@generate`, `@sample`, `@dispatch_table`, … | steer stimuli generation |
| Layout | `@whitespace_sensitive` | grammar-level layout policy: disable the automatic layout skip (whole grammar or per facet) |
| Profiles | `@default_profile` | grammar-level default dialect profile: what an *unspecified* requested profile resolves to |
| Profiles | `@profile_alias` | grammar-level request-spelling map: which requested spellings resolve to which canonical profile names |
| Pragmas | `@stop_at_rule_boundary` | bound how far a sequence consumes |

The store-backed gating directives are the heart of context-aware parsing — for example, "only treat this
bare identifier as a type name if a `typedef` already emitted a `type_name` fact for it". A grammar rule
that categorizes a bare identifier into a tracked category (type / class / package / …) **should** consult
the store via a `@predicate` rather than matching the identifier unconditionally.

## A worked shape

```ebnf
# declare the fact kind
@fact_kind: { name: type_name, exportable: true }

# a typedef EMITS a type_name fact for the new name
@emit_fact: type_name
type_declaration := "typedef" data_type identifier ";" -> {kind: "typedef", name: $3}

# a use-site is a known type ONLY if that fact exists
@predicate: has_fact(type_name, $1)
known_type := identifier -> {kind: "type_ref", name: $1}
```

## Layout policy — `@whitespace_sensitive`

By default a generated parser is **whitespace-INSENSITIVE**: it silently skips layout (whitespace and
unclaimed comment introducers) before every string terminal and regex token, and consumes trailing
layout after the entry rule. `start := "a" "b"` therefore accepts `ab`, `a b`, ` ab`, and `ab `.

A whitespace-SENSITIVE language opts out with this **grammar-level** directive (declare it once,
directly above a rule — conventionally the entry rule):

```ebnf
# the whole grammar is whitespace-sensitive: every space is literal input
@whitespace_sensitive: true
regex = pattern
```

With `true`, none of the three skips happen — `start := "a" "b"` now accepts **only** `ab`. This is
the policy `grammars/regex.ebnf` declares: in a regex, ` ` is an atom, so `a b` must parse as
three atoms, and a trailing space must not be silently discarded.

The granular form enables individual facets (an absent field means "keep the default skip"):

```ebnf
# only regex tokens are whitespace-sensitive; terminals and trailing layout keep the skip
@whitespace_sensitive: { regex_tokens: true }
systemverilog_preprocessor_file := pp_item*
```

| Field | `true` means the generated parser must NOT … |
| --- | --- |
| `terminals` | skip leading layout before a string terminal (`match_string`) |
| `regex_tokens` | skip leading layout before a regex token (`match_regex`) |
| `trailing` | consume trailing layout after the entry rule (`parse_full`) |

Rules of the road:

- **One declaration per grammar.** Identical duplicates are tolerated; *conflicting* payloads are a
  hard generation error. Malformed payloads (an unknown field, a non-boolean value) are hard errors
  too, and the annotation validator lints them early (`W_SEM_INVALID_WHITESPACE_SENSITIVE_PAYLOAD`).
- **Compile-time only.** The directive carries no runtime semantics — the policy is burned into the
  emitted parser code, and the parse-harness interpreter derives its layout policy from the same
  compiled declaration, so both implementations agree by construction.
- **Provenance.** This directive replaced an engine-internal gate that keyed the layout policy on the
  grammar's *file name* — the policy is now declared in the grammar itself, and any grammar
  (including a scratch/probe grammar) can be whitespace-sensitive. The isolating proof cases live in
  the structural combinator suite (`layout_insensitive_default`, `layout_ws_sensitive_full`,
  `layout_ws_sensitive_regex_tokens`).

## Default profile — `@default_profile`

A grammar that gates rules by dialect profile (`@profiles`) can also declare what an **unspecified**
requested profile means. Without a declaration, requesting no profile leaves the profile guard
permissive — every `@profiles`-gated rule stays active. With this **grammar-level** directive
(declare it once, directly above a rule — conventionally the entry rule):

```ebnf
# an unspecified requested profile means strict pcre2; `relaxed` is the opt-out
@default_profile: pcre2
regex = pattern
```

an unspecified or empty requested profile resolves to the declared default everywhere: the generated
parser's constructor starts on it, `set_grammar_profile(None)` *restores* it (never a permissive
unset state), generation-side profile filtering uses it, and the parse-harness interpreter resolves
it from the same compiled declaration. An **explicit** requested profile always wins.

This is the policy `grammars/regex.ebnf` declares: regex is PCRE2-faithful by default, so the
`@profiles: ["relaxed"]`-gated constructs (`\u`, …) are excluded unless `relaxed` is requested.

The payload is ONE profile name — a bare identifier-shaped scalar (`pcre2`, `sv_2017`) or a quoted
string for names with other characters (`"verilog-2005"`).

Rules of the road:

- **One declaration per grammar.** Identical duplicates are tolerated; *conflicting* payloads are a
  hard generation error. Malformed payloads (empty, non-scalar, non-identifier-shaped) are hard
  errors too, and the annotation validator lints them early
  (`W_SEM_INVALID_DEFAULT_PROFILE_PAYLOAD`).
- **Compile-time only.** The directive carries no runtime semantics — the default is burned into
  the emitted parser (a `DEFAULT_GRAMMAR_PROFILE` constant + the constructor/setter), and every
  other consumer derives from the same compiled declaration.
- **Provenance.** This directive replaced engine-internal gates that keyed the regex→`pcre2`
  default on the grammar's *name* — the default is now declared in the grammar itself, and any
  grammar (including a scratch/probe grammar) can declare one. The isolating proof cases live in
  the structural combinator suite (`profile_unspecified_permissive`, `profile_default_gate`).

## Profile aliases — `@profile_alias`

A profiled grammar can also declare which requested **spellings** resolve to its canonical profile
names, with the **grammar-level** map directive (declare it directly above a rule — conventionally
the entry rule; multiple declarations merge):

```ebnf
# accepted request spellings for this grammar's canonical profiles
@profile_alias: { "2017": sv_2017, "ieee1800-2017": sv_2017, "ieee_1800_2017": sv_2017 }
@profile_alias: { "2023": sv_2023, "ieee1800-2023": sv_2023, "ieee_1800_2023": sv_2023 }
systemverilog_file := trivia source_text trivia
```

A requested profile spelling is looked up **case-insensitively**; a declared spelling resolves to
its canonical target everywhere (the generated parser resolves it inside `set_grammar_profile`, the
registry's profile oracle, generation-side profile filtering, and the parse-harness interpreter all
derive from the same compiled declaration). An **undeclared** spelling passes through un-coerced —
it simply matches no `@profiles` list.

Keys may be quoted (needed for spellings like `"1364-2005"`) or bare identifier-shaped; targets are
identifier-shaped canonical profile names.

Rules of the road:

- **Merge semantics.** Multiple `@profile_alias` declarations merge into one map (an alias table is
  naturally written in groups). Re-declaring the same spelling with the SAME target is tolerated;
  a *different* target is a hard generation error.
- **Targets must be real.** Every alias target must be a profile the grammar actually declares
  (the union of its `@profiles` list payloads and its `@default_profile`) — a typo'd target is a
  hard error, and alias→alias chains are impossible by construction. A spelling may not shadow a
  canonical profile name. Malformed payloads (non-object, empty object, non-scalar targets) are
  hard errors too, linted early (`W_SEM_INVALID_PROFILE_ALIAS_PAYLOAD`).
- **Compile-time only.** The map is burned into the emitted parser — a sorted
  `GRAMMAR_PROFILE_ALIASES` constant plus case-insensitive resolution in `set_grammar_profile` —
  so every entry point accepts the declared spellings with no caller cooperation.
- **Provenance.** This directive replaced engine alias tables that keyed the SystemVerilog
  request spellings (`2017` → `sv_2017`, …) on the grammar's *name* — the spellings are now
  declared in `grammars/systemverilog.ebnf` itself, and any grammar can declare its own. The
  isolating proof cases live in the structural combinator suite (`profile_alias_resolves`,
  `profile_alias_unknown_passthrough`).

## Lexical annotations

The `[> … ]` / `[>! … ]` follow-restriction is a *lexical* annotation in the same family; it is documented
in [Lookaheads](lookaheads.md) and the platform book's
[Lexical Annotations](../../book/src/lexical-annotations.md) chapter.

## Where to go deep

- [semantic_annotation parser book](../../semantic_annotation_parser_book/src/welcome.md) — the value
  language, the full directive catalog, and the seven-stage semantic-store lifecycle.
- `docs/reference/PGEN_ANNOTATION_NORMATIVE_SPEC.md` — the normative directive semantics.
- `docs/reference/PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md` — which knob steers which behaviour.
- The platform book's [Annotation System](../../book/src/annotation-system.md) chapter — the
  two-annotation-family overview.

> Reminder: PGEN has no `{? … ?}` semantic-predicate *syntax* (see
> [Rules and Expressions](rules-and-expressions.md)). Context gating is done with a `@predicate`
> annotation, which the codegen genuinely consumes.
