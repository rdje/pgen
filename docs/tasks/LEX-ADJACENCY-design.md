# LEX-ADJACENCY.1 — Design: what the no-layout lexical boundary IS

> **Leaf:** `LEX-ADJACENCY.1` (`PGEN-LEX-ADJACENCY-0001`, session #208, 2026-07-26).
> **Read-only + design.** No code, no grammar, no generated artifact was touched.
> Every claim below is a MEASUREMENT with a re-runnable command; the evidence
> bundle is `docs/tasks/artifacts/lex_adjacency/` (`run_probes.sh`,
> `closure_probe.py`, and the two captured outputs).

---

## 0. The headline — the primitive is not missing, it is UN-DECLARABLE

`SV-CORPUS-GRAD.3.11` concluded (`design_adjudication.txt`, "ROUTE 4 — a
generator-side 'no layout here' primitive ⛔ DOES NOT EXIST") that PGEN has no
no-layout capability on either side of the engine. **That conclusion is wrong on
both halves, and this leaf corrects it.** Both halves already exist and both
already ship:

| half | the capability | how it is requested today | why `time_literal` cannot use it |
|---|---|---|---|
| **PARSE** | `match_regex(pattern, /*skip_leading_whitespace=*/ false)` | a **hard-coded rule-NAME allowlist** in codegen: `!matches!(rule_name, "string_content_double" \| "string_content_single")` | those two names belong to `return_annotation.ebnf`; no grammar can add itself to an engine `matches!` arm |
| **GENERATE** | interior separator suppression (`atomic_token_depth`) | **INFERRED** from the return shape — `-> $text`/`$0` on every branch, or a `@transform` directive | requesting it means giving up the `{value, unit}` AST object, and it *additionally* fuses the rule's LEFT-exterior boundary |

So the gap is not "PGEN cannot do this". The gap is: **there is no EBNF-native way
to ask for it.** That reframes leaf `.2` from *build a new primitive* to *decouple
and scope two mechanisms that already work* — a materially smaller and lower-risk
piece of work, and one that follows a pattern this repo has already executed four
times (`WS-DIRECTIVE.2`, `DEFAULT-PROFILE.2`, `PROFILE-ALIAS.2`, and
`STIMULI-SIGNOFF.12`'s `@quantified_separator`, each of which replaced a hard-coded
engine name-gate with a declarative grammar directive).

The tree's own framing table ("PARSE | layout is skipped before EVERY terminal |
… There is no per-token opt-out") is corrected by this leaf: there **is** a
per-rule opt-out; it is simply not reachable from a grammar.

---

## 1. WHY + WHERE (tool-backed)

Re-runnable: `bash docs/tasks/artifacts/lex_adjacency/run_probes.sh`
(captured verbatim in `design_measurements.txt`; byte-identical across re-runs).

### 1.1 PARSE half — the no-skip path is LIVE, and name-gated

```
rust/src/ast_pipeline/ast_based_generator.rs:5551:  !matches!(rule_name, "string_content_double" | "string_content_single");
rust/src/ast_pipeline/ast_based_generator/cascade.rs:1397        (mirror)
rust/src/ast_pipeline/ast_based_generator/cascade.rs:2060        (mirror)
rust/src/ast_pipeline/ast_based_generator/cascade/value.rs:878   (mirror)
rust/src/ast_pipeline/ast_based_generator/cascade/value.rs:1107  (mirror)
rust/src/ast_pipeline/ast_based_generator/scan.rs:468            (mirror)
rust/src/ast_pipeline/fusibility_census.rs:60   REGEX_ATOM_NO_SKIP_RULES (the census's static mirror)
```

The emission is not dead code — it ships:

```
generated/return_annotation_parser.rs: match_regex(...) total=20   no-skip=10
generated/systemverilog_parser.rs:     match_regex(...) total=1798 no-skip=0
```

`.3.11` measured the SV `0` correctly; what it inferred from it ("there is no
per-token opt-out") does not hold. SV has `0` because no SV rule is spelled
`string_content_double`/`string_content_single` — the two rules of
`grammars/return_annotation.ebnf:131-132`.

⚠️ **This is also a live `EBNF-SOURCE-OF-TRUTH` breach of exactly the class
`scripts/check_ebnf_source_of_truth.sh` exists to prevent**: an acceptance-affecting
behaviour keyed on a rule NAME literal inside the engine instead of declared in the
EBNF. Landing `.2` retires it, which is a doctrine win independent of the SV defect.

### 1.2 GENERATE half — atomicity is inferred, and the two effects are coupled

```
rust/src/ast_pipeline/stimuli_generator.rs:13023  fn rule_is_lexically_atomic(&self, rule_name) -> bool
rust/src/ast_pipeline/stimuli_generator.rs:9421   let is_atomic = self.rule_is_lexically_atomic(rule_name);
rust/src/ast_pipeline/stimuli_generator.rs:9638   self.atomic_token_depth += 1;          // INTERIOR effect
rust/src/ast_pipeline/stimuli_generator.rs:9650   self.last_terminal_from_atomic_rule = is_atomic;  // EXTERIOR effect (×5 sites)
```

`rule_is_lexically_atomic` returns true iff the rule's return is `MatchedText` on
**every** branch, or the rule carries `@transform` — i.e. atomicity is a *side
effect of the declared AST shape*, never a thing a grammar states on purpose.

Both effects are driven by that **one** bool, and they are consumed by two
**independent** conditions in `append_generated_segment`
(`stimuli_generator.rs:13052`):

```rust
if self.config.enforce_word_boundary_spacing
    && self.atomic_token_depth == 0        // INTERIOR: are we inside an atomic rule's body?
    && !segment_from_atomic_rule           // EXTERIOR: is the incoming segment an atomic rule's rendering?
    && prev_tail_word_shaped
    …
{ output.push(' '); }
```

That the two conditions are already separate is what makes `.2` small: the design
needs a third setting of an existing pair, not a new mechanism.

---

## 2. The measured gap: the (interior × exterior) matrix

Probe `probe_atomic_triggers.ebnf` (three shapes of the same `number unit` rule)
and `probe_exterior_boundary.ebnf` (the same, placed after a real keyword, i.e.
the actual SystemVerilog shape `timeunit 10ns;`), both at `--seed 0`:

| setting | interior seam | exterior seam | emitted sample | verdict |
|---|---|---|---|---|
| ordinary object return (today's `time_literal`) | **open** ✗ | separated ✓ | `<A>timeunit 0 ns;` | over-accepting — the `.3.11` defect |
| `-> $text` (inferred atomic) | closed ✓ | **fused** ✗ | `<D>timeunit02s;` | breaks a *different* boundary |
| `@transform` (inferred atomic) | closed ✓ | **fused** ✗ | `E0105fs` | same, and coerces the AST to a scalar |
| **what the LRM requires** | **closed** ✓ | **separated** ✓ | `timeunit 10ns;` | **unreachable today** |

The fourth row is the whole design. It is unreachable because
`stimuli_generator.rs:9421` derives both effects from a single `is_atomic`, so the
generator can only offer (open, separated) or (closed, fused) — never
(closed, separated).

Note `<D>timeunit02s;` is not merely ugly: `kw_timeunit := /timeunit\b/` requires a
word boundary, and `0` is a word character, so the fused text **does not re-parse**.
It would trade `.3.11`'s duality break for a new one.

### 2.1 Why this route beats `.3.11`'s Route 3 (`@sample`), measured

| route | generator coverage on the probe |
|---|---|
| `@sample` pin, sole path to `time_unit` (`.3.11` `gen_probe3.ebnf`) | rules **3/13 (23.08%)**, branches **0/6 (0.00%)** |
| atomic-token route (`probe_exterior_boundary.ebnf`, variant `<D>`) | rules **11/11 (100%)**, branches **5/5 (100%)** |

The `@sample` route was blocked because pinning the only path to `time_unit`
stranded it and its six `kw_*` tokens toward cert `UNKNOWN`, which would have broken
`union_unknown: 0` / `fully_certified_via_union: true` (the
`STRUCTURED-WITNESS-SYNTH` achievement). The atomic route does **not** short-circuit
generation — every unit branch is still generated (`<D>…s;`, `<D>…ns;`, `<D>…ps;`
all appear). **This is the measurement that makes the chosen route viable where
Route 3 was not**, and it is the tree's stated acceptance condition (3).

---

## 3. Decision — the surface

**`@lexical_token: true`, a rule-level semantic annotation.**

> **Semantics.** The annotated rule denotes **one lexical token**. Within its
> derivation: (parse) no terminal skips leading layout, and no layout-consuming
> element is honoured; (generate) no separator is inserted between any two
> elements. The rule's **own outer boundaries are unaffected** — it separates from,
> and is separated by, its neighbours exactly as an ordinary rule does.

The three candidate surfaces the tree listed, priced on evidence:

| candidate | verdict | why |
|---|---|---|
| sequence-level operator (`a . b`) | **rejected** | needs per-ELEMENT plumbing on both halves. Per-element annotations are **not generator-visible** today (book, *Lexical Annotations* → *What the generator does with it*: "per-rule and per-branch annotations are generator-visible; per-element ones are not"). It would need new EBNF syntax *and* a new annotation-visibility tier — strictly more work than the rule-level form, for no expressive gain on any known consumer. |
| **rule-level annotation** | **CHOSEN** | both existing mechanisms are already keyed on the **rule name** (`rule_is_lexically_atomic(rule_name)`; `matches!(rule_name, …)`). A rule-level directive maps 1:1 onto both with no new concept, and it reads the way an LRM states the constraint ("a time literal is one lexical token"). |
| per-terminal no-skip form | **rejected as the surface** | it is the parse-side *implementation*, not an author-facing concept; it also cannot express the generate half at all (the generator inserts separators between *elements*, not inside terminals). Retained as the emission mechanism in §5. |

**Naming.** `@lexical_token` is the rule-level sibling of the existing
grammar-level `@whitespace_sensitive`, and it adopts the vocabulary the engine and
the book already use for this property ("atomic lexical token", `LEXICAL-ANNOTATIONS.6`).
Rejected alternatives: `@atomic` (overloaded — reads as transactional, which is a
different and heavily-used concept in this engine), `@no_layout` (says the
mechanism, not the meaning), and a bracket form like `[. … ]` (the `[> ]`/`[>! ]`
bracket family expresses constraints on what follows a rule's *outside*; this is a
statement about its *inside*, so an `@` directive is the honest fit).

**⛔ EBNF-native, per `project_ebnf_is_single_source_of_truth`.** The directive is
declared in the grammar and consumed by codegen + the generator; nothing is wired
out of band. Landing it also *removes* an existing out-of-band gate (§1.1).

---

## 4. Decision — scope: deep, and interior-only

**Deep (transitive).** `@lexical_token` applies to the annotated rule's whole
derivation, not just its immediate elements — matching the existing
`atomic_token_depth` counter, which is held across `generate_node` for the entire
subtree. This is also the LRM-faithful reading: if a production *is* one token,
nothing inside it may be split.

**Interior-only.** The annotated rule must **not** set the cross-rule cohesion flag
`last_terminal_from_atomic_rule`. That flag is correct for the *inferred* atomicity
it was built for — a `$text`/`@transform` rule is a token **fragment** that must
glue to its neighbour (`recursion_condition` `"R"` + `digits` → `R12`;
`hex_escape` `"x"` + payload → `xAB`) — but it is wrong for a **complete** token
like `time_literal`, as the `<D>timeunit02s;` row measures. The two use-cases are
genuinely different and must stay distinguishable; `.2` therefore introduces a
third state rather than reusing `is_atomic`:

| | interior suppressed | exterior fused | used by |
|---|---|---|---|
| not atomic | no | no | ordinary rules |
| inferred atomic (`$text`/`@transform`) | yes | yes | token **fragments** — unchanged, no regression risk |
| **`@lexical_token`** | **yes** | **no** | complete tokens — the new state |

⚠️ **Named risk for `.3` (do not assume it away).** "Deep" means a
`@lexical_token` rule closes *every* seam beneath it — including seams an LRM
deliberately leaves **open**. IEEE 1800-2017 §5.7.1 (the law `SV-CORPUS-GRAD.3.10`
established) explicitly permits white space at a based literal's size↔apostrophe
and base↔value seams. `time_literal` is safe because A.8.4 restricts its number to
`unsigned_number`/`fixed_point_number`, which have no such seams — but the
footnote-33/48/50 re-sweep `.3` schedules **must** re-check this per rule rather
than applying the directive mechanically. This is the same trap
`SV-CORPUS-GRAD.3.10` disarmed when it ruled `unbased_unsized_literal` correct-as-fused.

---

## 5. Decision — the parse half: STATIC specialization, not a runtime flag

Two ways to make "no layout inside this derivation" true at parse time:

- **(a) runtime depth counter** on the parser, mirroring `atomic_token_depth`.
  Simple and exactly parallel to the generator — but it puts a check on every
  rule entry / terminal match. ⛔ **Rejected on the ⭐ speed north star** (parse
  time is a first-class, continuously-tracked deliverable, monitored "like milk on
  fire" for every generated parser). A primitive used by two rules must not tax
  all 1,798 terminal sites.
- **(b) static specialization** — codegen emits a no-layout twin of each rule in
  the annotated rule's transitive closure, and the annotated rule calls the twins.
  Zero runtime cost; cost is code size, proportional to the closure.

Whether (b) is affordable is a measurement, not a preference. Measured with
`closure_probe.py` on the real grammar (`closure_measurement.txt`):

```
grammar rules: 1475
time_literal     closure size =    14
                 members: block_comment, integral_number, kw_fs_3f4bb586, kw_ms_26cc3217,
                          kw_ns_7320d5b7, kw_ps_c67f1ee1, kw_s_a0f1490a, kw_us_da2b1288,
                          line_comment, number, real_number, time_literal, time_unit, trivia
number           closure size =     3
time_unit        closure size =    10
trivia           closure size =     3
```

**14 of 1,475 rules — under 1%.** Specialization costs ~13 extra rule methods in a
parser that already has 1,475. ⇒ **(b) is chosen**: the no-layout property is
resolved statically at codegen time and costs a bare parse nothing.

`.2` must still gate this: a rule whose closure is large (or which is
mutually recursive with a non-lexical rule) is not a lexical token in any useful
sense. **Proposed lint (`--lint-grammar`, hard error):** `@lexical_token` on a rule
whose closure exceeds a declared bound or re-enters a non-lexical rule is a grammar
authoring error, reported with the offending path. This keeps the primitive honest
and keeps the emission static.

### 5.1 The parse half must neutralise EXPLICIT layout too

The closure above names `trivia`, `line_comment`, `block_comment` — because the SV
unit tokens are written `kw_ns_7320d5b7 := trivia /ns\b/`
(`grammars/systemverilog.ebnf:6458`). So layout is consumed **twice over** inside
this derivation: implicitly by `skip_leading_whitespace`, and explicitly by a
`trivia` element in the rule body.

⇒ Flipping `skip_leading_whitespace` alone is **not sufficient**, and a design that
only did that would silently still accept `10 ns`. The specialized twin must also
drop explicit layout elements. This is the parse-side analogue of `.3.11`'s
measured variant-B finding ("removing `trivia` is not sufficient" on the generate
side) — and it is the single most likely way `.2` gets it wrong, so it is called out
here rather than discovered later.

---

## 6. Acceptance for `.2` (what the gate must prove)

Restating the tree's criteria in now-measurable terms:

1. **PARSER** — with `@lexical_token` on a probe rule, interior layout REJECTS and
   tight text ACCEPTS; without it, byte-identical to today.
2. **GENERATOR** — no separator across the interior (`10ns`), separator preserved
   at the exterior (`timeunit 10ns;`), and `sample_parse_failures = 0` at seeds
   0/7/42.
3. **CERTIFICATE** — every sub-rule still witnessed: on the probe, `rules 11/11,
   branches 5/5` (the Route-3 contrast in §2.1). On SV, `time_unit:5555` and its six
   `kw_*` tokens stay reachable AND witnessed; `union_unknown: 0` /
   `fully_certified_via_union: true` unchanged.
4. **INERTNESS** — no grammar without the directive changes by a single byte
   (the `@quantified_separator` precedent: an empty policy map makes the path inert).
5. **DOCTRINE** — the `string_content_double`/`string_content_single` name-gate is
   retired in the same slice and `return_annotation.ebnf` declares
   `@lexical_token` instead, with its generated parser's `no-skip=10` preserved.
   (This is the migration's own regression test, and it is why `.2` should convert
   that grammar first: a known-good before/after already exists.)
6. **SPEED** — a bare parse of a grammar without the directive is unchanged by
   construction (static emission); confirm no regression on the tracked lane.

---

## 7. Relationship to the `LEXICAL-ANNOTATIONS` tree

`LEX-ADJACENCY` is not a rival tree: it is the **declarability + parse half** of
`LEXICAL-ANNOTATIONS.6`, which landed the interior/exterior cohesion machinery
**generator-only** and inferred the signal from the return shape. That leaf's own
notes call `$text`/`@transform` a "declarative atomicity signal" — it is declarative
by proxy, and this leaf measures the two places that proxy fails (AST hijacking,
exterior fusion).

Ownership stays with `LEX-ADJACENCY` because the parse half — the codegen name-gate
and the specialized-twin emission — is outside `LEXICAL-ANNOTATIONS`' generator-only
scope, and because the consumers span two grammar families. **Lockstep obligation:**
`.2` must update `docs/book/src/lexical-annotations.md` (the owning chapter, which
today documents only the inferred form) in the same slice.

---

## 8. Evidence index

| artifact | what it proves |
|---|---|
| `artifacts/lex_adjacency/run_probes.sh` | re-runnable driver for everything in §1–§2 (deterministic; byte-identical across re-runs) |
| `artifacts/lex_adjacency/design_measurements.txt` | captured output: the (interior × exterior) matrix + the seven engine sites |
| `artifacts/lex_adjacency/probe_atomic_triggers.ebnf` | the two inferred atomicity triggers close the interior seam |
| `artifacts/lex_adjacency/probe_exterior_boundary.ebnf` | …and both fuse the exterior seam against a keyword |
| `artifacts/lex_adjacency/closure_probe.py` | closure sizer (reads the `--dump-gen-ast` IR codegen consumes) |
| `artifacts/lex_adjacency/closure_measurement.txt` | `time_literal` closure = 14/1475 ⇒ static specialization is affordable |
| `artifacts/sv_corpus_grad/time_literal_ws_diag/` | `.3.11`'s upstream diagnosis + the already-written, parser-verified Route-1 fix |
