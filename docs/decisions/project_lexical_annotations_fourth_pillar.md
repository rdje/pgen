---
name: project-lexical-annotations-fourth-pillar
description: "Director architecture decision (2026-06-06): PGEN has a FOURTH declarative pillar — LEXICAL (a.k.a. LAYOUT) ANNOTATIONS — beyond EBNF + return annotations + semantic annotations. It governs surface faithfulness (the characters emitted must re-lex to the intended tokens). It can STEER BOTH parsing and generation, but in PGEN today affects generation far more often (the parser already resolves most token boundaries via maximal munch; the generator has no lexical discipline). Parsing also verifies faithfulness by re-parsing generated output. To be designed generally + parser-agnostically, never as a hack."
metadata:
  node_type: memory
  type: project
  director_directive: true
  created: 2026-06-06
  originSessionId: 5737d722-67a3-4fc9-8c42-f79e2ff1db07
---

**Director architecture decision (2026-06-06).** PGEN expresses a language with **four** declarative
pillars, not three:

1. **EBNF grammar** — context-free *structure* (valid token sequences). text → tree.
2. **Return annotations** — structure → *AST/output* shape. tree → data.
3. **Semantic annotations** — context-sensitive *meaning* (types, scopes, the store). tree ⟷ meaning.
4. **Lexical annotations** *(a.k.a. layout annotations)* — *surface faithfulness*: the characters
   emitted must **re-lex/re-parse to exactly the tokens they were meant to be.** tree → text.

Pillars 1–3 fully pin down the **tree** and its meaning but say **nothing** about how that tree is
rendered to **characters** (token separation, mandatory newlines, layout). That gap is the 4th pillar.

**Which direction it serves — BOTH, generation-dominant today.** Lexical-surface constraints are a
property of the language's lexical interface, and *both* directions traverse it, so they can **steer
both parsing and generation**: on the parse side as **disambiguation** (SDF2 follow restrictions /
reject productions resolve longest-match ambiguities and keyword reservation), on the generate side as
**faithful-rendering enforcement** (insert separators so output re-lexes correctly). **In PGEN today
they are needed for generation far more often**, because the parser already resolves most token
boundaries *automatically* via **maximal munch** (longest match, implicit in regex matching) while the
generator has **no** lexical discipline at all. So today's enforcer-most-needed is the generator; the
parser both *honours* the constraints (maximal munch) and *verifies* faithfulness (the certifying gate
re-parses generated output, checking `render(t)` re-lexes to `t`).

This **refines** the director's earlier "affects only generation" reading (director 2026-06-06): the
pillar is **bidirectional** — it may steer both parsing and generation; the generation skew is a fact
about PGEN's *current* state (parser has maximal munch, generator has nothing), not about the pillar's
nature. Practically, the `.2`–`.4` work is generation-side enforcement, designed so the *same*
declarative lexical annotations can serve parse-time disambiguation if ever needed.

**Why pillars 1–3 cannot express it (the no-workarounds justification).** EBNF is token/rule-level and
*assumes* a tokenization; it does not state the character-level separation needed to *achieve* it.
Return annotations describe AST shape. Semantic annotations describe the store (types/scopes). The
lexical/character layer is orthogonal to all three. This is a genuine Level-5 addition (a new general
parser-agnostic primitive) under [[feedback_no_workarounds_fix_hierarchy]], and a legitimate engine
feature under the refined [[feedback_prefer_grammar_leave_engine_alone]] (parser-agnostic, benefits
every grammar).

**Evidence (two real defects, one missing capability).** From the `GRAMMAR-WELLFORMED.G.4`
certificate-coverage gate parsing generator output: (a) token fusion `endprogram`+`module` →
`endprogrammodule`; (b) a generated `//` line comment with no terminating newline swallows the rest of
a single-line sample. Root cause of (b) confirmed at source: `generate_from_regex_hir` treats every
regex anchor (`HirKind::Look(_)`) as the empty string and picks alternation branches uniformly, so for
`line_comment := /\/\/[^\n]*(\n|$)/` it selects the zero-width `$` branch. The `$` is load-bearing for
*parsing* (a file may end `//x` with no newline — proven to parse), so the EBNF is correct; the defect
is purely generation not honouring the anchor. `enforce_word_boundary_spacing` is a one-case,
flag-driven *prototype* of this pillar — evidence it should be first-class and general.

**How to apply.**
- Treat lexical faithfulness as a first-class requirement of any PGEN generator: `render(t)` must
  re-lex/re-parse to `t`'s token stream.
- Design it generally and parser-agnostically — separation **derived from the token regexes** the
  generator already holds, plus an optional declarative **follow-restriction** lexical annotation for
  what derivation can't infer. **No SV-specific hacks, no per-case flags.**
- Subsume `enforce_word_boundary_spacing` and the comment-newline case as instances.
- Verify with the parser (the certifying gate already re-parses output).

**Naming.** Prefer **lexical annotations**; **layout annotations** is an accepted synonym (director
2026-06-06).

**Notation (decided 2026-06-06, with the director).** A lexical annotation is written `[> LIST ]`
("must be followed by") or `[>! LIST ]` ("must NOT be followed by"), where `LIST` is one or more
**items** — each a `/regex/` or a `"string"`, freely mixed — separated by whitespace and/or commas.
The list denotes a **union**: `[>! /\w/, "endmodule"]` = "not followed by *any* of {word-char,
`endmodule`}"; `[> /\n/ /\r/]` = "followed by one of {LF, CR}". A single item is just a list of one.
The enclosing `[ … ]` is precisely what bounds the list cleanly — the reason a bracket is preferred
over a bare sigil + single spec (director rationale 2026-06-06).

**Placement (decided 2026-06-06).** Two positions, mirroring semantic annotations:
- **Before-rule** (in `annotation_list`) — binds the rule (its right boundary). Like `@…` directives.
- **Inline** — a standalone `sequence_element`, *structurally identical* to `inline_semantic_annotation`.
  The **only** difference: an inline **semantic** annotation binds the **following** item, whereas an
  inline **lexical** annotation binds the **preceding** item (because it describes what may follow the
  token it's about). This makes per-branch placement fall out naturally (branches are sequences). E.g.
  `foo := kw identifier [>! /\w/] "="` constrains *that* `identifier`'s right boundary.

It sits in the **directive position** (in `annotation_list`, binding to the following rule, like `@…`)
or inline (binding the preceding element, per above).
Although `[ … ]` is `optional_element` in rule *bodies*, `[>` / `[>!` is **unambiguous** — no
`rule_expression` can start with `>` (its first char is always `"` `/` `[` `(` or a letter), so `[>`
can never be an optional. `[< LIST ]` / `[<! LIST ]` are **reserved** for "preceded by" (lookbehind)
should a grammar ever need it. The `>` semantics echo lookahead assertions; the form is bounded
(visible `[` … `]`) and distinct from `->` (return) and `@` (semantic). Examples: `[>! /\w/]` above an
identifier/keyword (anti-fusion); `[> /\n|$/]` above a line comment (needs a trailing newline);
`[>! "(", "["]` (a token that must not be directly followed by an opening bracket). Chosen over `-/-`
(SDF) and `~>`/`@>` alternatives for being bounded, list-friendly, and readable.

Owned by the [`LEXICAL-ANNOTATIONS`](../tasks/LEXICAL-ANNOTATIONS.md) tree (survey →
[research synthesis](../tasks/LEXICAL-ANNOTATIONS-research-synthesis.md) → design → implement → verify);
book chapter [`lexical-annotations.md`](../book/src/lexical-annotations.md). Feeds
[[project_stimuli_generator_signoff_vision]]; relates to [[feedback_certifying_linter_trustworthiness]]
(the gate is the verifier) and [[feedback_ast_pipeline_parser_agnostic]] (absolute).
