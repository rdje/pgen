# Capability work is GREENLIT by standing authorization — stop asking permission for the WHAT, keep asking for the SURFACE

**Category:** feedback · **Established:** 2026-08-10 (director) · sharpens
[[project_horizon_universal_parser]] and narrows one of its own cautions (see §4).

## The directive (director, verbatim)

> *"I greenlight every decisions that would enable EBNF and the AST pipeline to achieve
> greatness in being able to handle any languages we throw at it."*

## 1. What this authorizes

The **capability question is settled standing**: an item whose purpose is to make the EBNF
surface expressive enough — or the AST pipeline capable enough — to parse a new class of
language does **not** need a per-item director approval to be *worked*. Bringing a priced
roadmap row forward is now **execution**, in the same sense
[[feedback_sequence_approved_work_yourself]] already made ordering execution.

Concretely, the priced rows of `LANG-CAPABILITY-AUDIT.4` are authorized *as capabilities*:
P1-3 fact retraction / instance-scoped fact lifetime, P1-4 declarative case-insensitive
keywords, **P2-5 parameterized productions**, P2-6 cross-rule precedence ladder, P2-7
offside rule + lexer modes, P3-8 NFKC normalization.

## 2. What it does NOT authorize — the boundary that keeps it honest

- ⛔ **It is not a syntax blank cheque.** *Whether* to have a capability is greenlit; *what
  notation the grammar author writes* is not. A new author-visible surface still needs a
  design leaf carrying a `PRIOR ART` section (`DESIGN-PRIOR-ART`,
  [[feedback_read_prior_art_before_designing]]) and, where a plausible notation is already
  taken, an explicit surface ruling. **P2-5 is exactly that case** — see §3.
- ⛔ **It does not relax the two NON-NEGOTIABLES.** Parser-neutrality and theoretical peak
  speed are still *rejection* criteria, not trade-offs
  ([[project_capability_growth_is_zero_cost_and_neutral]]): non-users pay ZERO, users pay at
  CODEGEN, runtime cost only where semantically unavoidable. A capability that cannot
  compile away is not greenlit by this — it is refused by that.
- ⛔ **It does not make a primitive "done" at parse-only.** Duality-completeness stands:
  parse **and** sound generation ([[project_horizon_universal_parser]] §2).
- ⛔ **It does not reopen the explicit NON-commitments.** Director #209 scoped
  **extensibility OUT** in the same sentence that scoped composability in, and row 16
  (parse-time-mutable grammar — Raku slangs, Perl 5 `BEGIN`, Prolog `op/3`) is a declared
  HARD BOUND. A general capability greenlight is not a reversal of a specific, deliberate
  exclusion. If either is meant to reopen, it needs saying separately.
- ⚠️ **It does not, on its face, lift the SV lane lock.** The lock
  ([[project_nexsim_sv_signoff_delivery_focus]], director 2026-08-08 ×3) says do not leave
  SV until it is RELEASED to Nexsim. This record is read as authorizing capability work
  **when the lane opens**, not as pre-empting the lock — the conservative reading, because
  the lock was set emphatically and this directive did not mention it. One word from the
  director reverses that reading.

## 3. Why P2-5 (parametric rules) still needs a surface call — and what is NOT already decided

**Measured 2026-08-10 (re-verified, not cited):** no parametric-rule syntax has ever been
decided or committed. What exists is a **placeholder that was written down and never
ratified**, and it is known-bad:

- `grammars/ebnf.ebnf:627` declares
  `parametric_rule := rule_name "[" parameter_list "]"` with
  `parameter_list := identifier_literal ("," identifier_literal)*` (`:630`).
- It is one of `LANG-CAPABILITY-AUDIT.1`'s **27 unreachable productions** — referenced by
  nothing, **0** word-anchored engine consumers.
- The notation it declares **silently miscompiles**: `start := expr[In, Yield]` yields
  `Sequence[ expr, Quantified{ Sequence[In, Yield], "?" } ]` — i.e. `expr ( In Yield )?`,
  comma swallowed — because `[ … ]` is already the optional-element form (`ebnf.ebnf:288`).
  The lint is **CLEAN**: `undefined_references=0`, `non_terminating=0`,
  `unreachable_rules=0`, exit 0.
- **No `docs/decisions/` record adopts a syntax.** The only mention is
  [[project_horizon_universal_parser]] `:167-170`, which is the *opposite* of a commitment —
  it warns that a broad "flexible" directive must **not** be read as a green light for
  `lexer_mode` / `parametric_rule` / `rule_modifier`.
- **No task leaf designed one.** `LANG-CAPABILITY-AUDIT.4` ranks P2-5 *"unscheduled — needs
  a director surface call"*, with the standing instruction that `parametric_rule` be
  **re-surfaced, never deleted** (unlike `~i"…"`, `[a-z]`, `import`/`extends`, which `.6`
  may retire as superseded).
- `PARSE-SOTA` B3 names rust-peg's parametric rules as **prior art**, not as an adopted
  surface.

⇒ the capability is greenlit by this record; the **notation is genuinely open**, and the
obvious one is taken.

## 4. Relationship to the caution it narrows

[[project_horizon_universal_parser]] `:167-170` says do not read "flexible" as authorization
for the declared-and-unwired set. This directive is **more specific than the language that
caution was written against**: it names the *goal* ("handle any languages we throw at it"),
which is precisely the parametric/JS use case. So it **does** supply the authorization that
caution withheld — **for the capability**, and still not for the surface, and still not for
extensibility. Recorded here rather than by editing that record, so the audit trail shows
which sentence moved and when ([[MEMORY_ARCHITECTURE]] §10 supersede-don't-mutate).

## How to apply

- Do not open a leaf asking *"may we build capability X?"* for a priced row — build it, or
  schedule it, and say so.
- **Do** open a design leaf, with prior art, for any new author-visible notation.
- Price every candidate against the compile-away acceptance test **before** designing it,
  not after.
- When a plausible notation collides with an existing one (P2-5's `[ … ]`), that collision
  is a director-level surface question, not an implementation detail to route around.
