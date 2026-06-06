---
name: project-lexical-annotations-fourth-pillar
description: "Director architecture decision (2026-06-06): PGEN has a FOURTH declarative pillar — LEXICAL (a.k.a. LAYOUT) ANNOTATIONS — beyond EBNF + return annotations + semantic annotations. It governs surface faithfulness (the characters emitted must re-lex to the intended tokens). It is enforced in GENERATION; parsing already honours it via maximal munch and serves as the verifier. To be designed generally + parser-agnostically, never as a hack."
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

**Why it was invisible / which direction it serves.** Parsing (text → tree) honours the lexical surface
*automatically* via the lexer's **maximal munch** (longest match) — implicit in regex matching — so the
parser never needed an explicit mechanism and parses valid input fine. Generation (tree → text) has no
maximal munch to lean on, so it must **actively enforce** the separation. Therefore the 4th-pillar
*mechanism* is **a generation concern** (confirmed by the director: "lexical annotations affect only
the generation"). Parsing then doubles as the **verifier**: the certifying gate re-parses generated
output, checking the invariant `render(t)` re-lexes to `t`. Enforcer = generator; verifier = parser.

*Footnote (honest):* the constraints are direction-neutral — SDF2 uses the same declarative "follow
restrictions" to disambiguate *parsing*. So the same lexical annotations could later serve parse-time
disambiguation if maximal munch ever proves insufficient — a latent bonus, not today's role.

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

Owned by the [`LEXICAL-ANNOTATIONS`](../tasks/LEXICAL-ANNOTATIONS.md) tree (survey →
[research synthesis](../tasks/LEXICAL-ANNOTATIONS-research-synthesis.md) → design → implement → verify);
book chapter [`lexical-annotations.md`](../book/src/lexical-annotations.md). Feeds
[[project_stimuli_generator_signoff_vision]]; relates to [[feedback_certifying_linter_trustworthiness]]
(the gate is the verifier) and [[feedback_ast_pipeline_parser_agnostic]] (absolute).
