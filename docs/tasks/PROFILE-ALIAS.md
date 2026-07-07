# PROFILE-ALIAS — `@profile_alias`: replace the SV profile-ALIAS name-gates with a declarative grammar-level directive

- Status: `active` (created 2026-07-07, session #56). `.1` DESIGN done (this document);
  FRONTIER = `.2` IMPLEMENT.
- Roadmap lane: cross-cutting engine correctness / EBNF-single-source-of-truth — WHICH
  request spellings a dialect-profile name accepts (`--profile 2017` ⇒ `sv_2017`) is
  parser-surface behavior documented to users (book: `parseability-probe-debug.md:89`,
  `embedding-and-downstream-integration.md:32/:38`; `TOOLBOX.md:122`), and today it is
  decided by THREE independent engine spelling tables — one gated on the grammar NAME
  `"systemverilog"` — invisible to the `.ebnf`. Same two standing doctrines the completed
  sibling trees `WS-DIRECTIVE` and `DEFAULT-PROFILE` enforced:
  [[feedback_features_parser_agnostic_enable_all_parsers]] (2026-06-08: *"capability-gated,
  never grammar-name-gated"*) and [[project_ebnf_is_single_source_of_truth]]. Plus the
  triplicated-table drift risk [[feedback_duplicated_metadata_needs_derived_drift_gate]]
  warns about — and the drift is ALREADY observable (§1, divergences).
- Origin: routed finding **F-A** of `DEFAULT-PROFILE` §6 (established tools-first
  2026-07-07, session #55; listed there for director visibility). Picked up by PNT
  (session #56) as the top routed layer-A frontier candidate — the third and last member of
  the name-gate retirement series (layout policy → default profile → request-spelling
  aliases), with the directive shape already precedent-endorsed twice.

## 1. Problem statement (tool-backed, verified against HEAD `b163330b`, 2026-07-07)

The knowledge "request spellings `2017` / `ieee1800-2017` / `ieee_1800_2017` mean
`sv_2017` (and the `2023` / `verilog_2005` equivalents) for the grammar named
`systemverilog`" lives in THREE engine string tables, NOT in `grammars/systemverilog.ebnf`:

- **Parse side** — `rust/src/parser_registry.rs:149-168`
  (`normalize_generated_grammar_profile`): `match grammar_name { "systemverilog" => … }`
  over the lowercased request — `"2017"|"ieee1800-2017"|"ieee_1800_2017"` ⇒ `sv_2017`
  (`:159`), the `2023` group ⇒ `sv_2023` (`:160`), `"verilog_2005"|"1364-2005"|
  "ieee1364-2005"|"ieee_1364_2005"` ⇒ `verilog_2005` (`:161-163`); every other grammar and
  every unmatched spelling passes through UNCHANGED (original case). Exposed as
  `active_grammar_profile` (`:127-131`) — the profile oracle the parse-harness interpreter
  (`parse_harness_interpreter.rs:127`, `:212`) and the differential-equivalence gate
  (`parse_harness_equivalence.rs:332`, `:1003`) resolve through; re-invoked with hardcoded
  grammar names inside the registry's own wrappers (`"systemverilog"` at
  `:558/:626/:670/:728/:804/:832`; `"regex"` at `:372/:385/:409/:1340`).
- **Generation side** — `rust/src/main.rs:2119-2130` (`normalize_grammar_profile_name`):
  a GLOBAL table — not even name-gated — that lowercases the request and applies the SV
  aliases (`:2122-2123`) AND VHDL aliases (`"1076_2019"|"1076-2019"|"ieee1076-2019"|
  "ieee_1076_2019"` ⇒ `vhdl_1076_2019`, `:2124-2126`) to ANY grammar, returning the
  LOWERCASED name for unmatched values (`:2128`). Call sites: `:2153/:2169/:2180/:2280`
  (the `@profiles` filter lanes for stimuli/cert).
- **Embedding side** — `rust/src/embedding_api.rs:285-305` (`GrammarProfile::FromStr`):
  the third copy — SV aliases (`:290-293`), VHDL aliases (`:295-296`), plus
  `"regex_default"|"regex"` (`:298`) — parsed into the typed `GrammarProfile` enum
  (`:186`), canonicalized by its name method (`:227-235`), flowing into
  `set_grammar_profile` at `:1425`.

Observable cross-copy DIVERGENCES (the drift the duplicated-metadata doctrine predicts):

1. **VHDL aliases exist only in copies 2+3.** The parse-side registry (copy 1) passes
   `"1076-2019"` through raw. Inert TODAY only because `grammars/vhdl.ebnf` declares ZERO
   `@profiles` rules (`grep -c "@profiles" grammars/vhdl.ebnf` = 0, this session;
   systemverilog = 392, regex = 6) — no guard ever consults VHDL's profile. The moment
   VHDL gains one profile-gated rule, parse side and generation side diverge observably.
2. **Cross-grammar alias leakage in copy 2.** `normalize_grammar_profile_name` applies
   `"2017"` ⇒ `"sv_2017"` to EVERY grammar: any grammar declaring a profile literally
   named `2017` would have generation-side requests silently rewritten to another name.
3. **Case handling differs.** Copy 2 lowercases unmatched values (`:2128`); copy 1 passes
   them through in original case (`:166`). Masked in acceptance only because the emitted
   guard compares `eq_ignore_ascii_case` (codegen `ast_based_generator.rs`
   `rule_profile_is_enabled`), but the profile STRING observers report differs by lane.
4. **`regex_default`/`regex` spellings exist only in copy 3** — and `regex_default` names
   a profile the regex grammar never declares (its `@profiles` universe is `relaxed`;
   default `pcre2` via `@default_profile`). Acceptance-equivalent today (unknown profile
   disables the same `relaxed`-gated set as `pcre2`), semantically stale since
   `DEFAULT-PROFILE.2`.

Consequences (why this is a defect, not a style nit):

1. **The `.ebnf` lies by omission.** `grammars/systemverilog.ebnf` declares its three
   canonical profiles at 392 `@profiles` sites, but NOTHING declares the ten accepted
   request spellings users are told to rely on. The stimuli generator, the interpreter,
   and any gen-AST consumer cannot see them.
2. **The capability is closed to every other grammar.** No other grammar — shipped,
   scratch, or synthetic — can declare request aliases; a future profiled deliverable
   (e.g. VHDL-2008 vs 2019) needs an ENGINE edit in three files (the anti-pattern the
   2026-06-08 doctrine forbids).
3. **Three drift-prone copies, already diverged** (the four divergences above), each
   maintained by hand with no gate parsing an authoritative source.

NOT in this class (verified): `parser_registry.rs` per-grammar registry-table dispatch
(blessed data-driven boundary); `default_generated_grammar_profile` (`:142-148`, the
landed sibling capability); the embedding `GrammarFamily` enum (typed FAMILY selection,
not profile-spelling normalization).

## 2. Design (leaf `.1`, this document)

### D1 — Surface

A **grammar-level** semantic directive, following the `@whitespace_sensitive` /
`@default_profile` precedent (free-standing `@name: payload` line; compiled by a
grammar-wide sweep; structured map payload already precedented by
`@whitespace_sensitive: { regex_tokens: true }`):

```ebnf
@profile_alias: { "2017": sv_2017, "ieee1800-2017": sv_2017, "ieee_1800_2017": sv_2017 }
@profile_alias: { "2023": sv_2023, "ieee1800-2023": sv_2023, "ieee_1800_2023": sv_2023 }
@profile_alias: { "1364-2005": verilog_2005, "ieee1364-2005": verilog_2005, "ieee_1364_2005": verilog_2005 }
```

Payload = a map of `alias-spelling → canonical-profile-name`. Multiple declarations
MERGE (unlike `@default_profile`'s single-scalar rule — an alias table is naturally
declared in groups). Parses through the existing generic annotation surface — **zero
meta-grammar change expected**; the dual-run gate verifies.

### D2 — Semantics and validation

- Absent directive ⇒ no aliasing — generic pass-through (today's behavior for every
  grammar except systemverilog; all other grammars untouched by construction).
- Alias lookup is CASE-INSENSITIVE on the request spelling (both existing tables lowercase
  before matching); a matched request resolves to the canonical target; unmatched requests
  pass through UNCHANGED (copy 1's behavior — the acceptance-neutral one, since the
  emitted guard is already case-insensitive; `.2` pins the copy-2 lowercasing delta with a
  probe).
- Re-declaring the SAME alias with the SAME target is tolerated; with a DIFFERENT target
  is a hard compile error (V-DECL-1 mirror).
- An alias TARGET must be a canonical profile the grammar actually declares — validated
  against the grammar's profile universe (the union of every `@profiles` list payload +
  the `@default_profile` payload). Outside the universe ⇒ hard compile error. This also
  forbids alias→alias chains by construction.
- An alias KEY that equals (case-insensitively) a declared canonical profile name ⇒ hard
  compile error (an alias may not shadow a real profile).
- Malformed payload (non-map, empty map, empty key/target) ⇒ hard compile error naming
  the directive and the accepted form.

### D3 — The artifact carries its aliases (parse side)

Codegen, for a directive-bearing grammar ONLY, emits into the generated parser:

1. `pub const GRAMMAR_PROFILE_ALIASES: &[(&str, &str)] = &[…];` — sorted by key,
   deterministic ([[project_earlier_always_matches_unsound_backtracking]] §HashMap lesson:
   any serialized map is emitted in canonical order);
2. alias resolution INSIDE `set_grammar_profile` — the setter resolves an alias spelling
   to its canonical name before storing (artifact-owned normalization, the same ownership
   model as `set_grammar_profile(None)` restoring `DEFAULT_GRAMMAR_PROFILE`) — so EVERY
   consumer (registry, embedding, direct construction, future callers) gets aliasing for
   free and the embedder-must-remember class cannot recur.

Non-bearing grammars emit nothing — regeneration must be `cmp`-byte-identical for all
non-SV parsers (D6 acceptance).

### D4 — Registry + interpreter (delete-not-fallback)

- `normalize_generated_grammar_profile`'s `"systemverilog"` match arm
  (`parser_registry.rs:157-165`) is DELETED; replaced by a data-driven per-grammar datum
  (`generated_grammar_profile_aliases(grammar_name)` sourcing
  `SystemVerilogParser::GRAMMAR_PROFILE_ALIASES`) in the same blessed table boundary as
  `default_generated_grammar_profile`. `active_grammar_profile` resolves
  alias-then-default, so the interpreter and the equivalence gate stay byte-for-byte with
  `parse_sample` (they already resolve through it).
- The interpreter additionally consults `compile_profile_aliases(annotations)` (D5's ONE
  compile fn) for NON-registered (synthetic/scratch) grammars, exactly as it consults the
  compiled default profile today.

### D5 — Generation side (ONE shared compile fn)

`semantic_runtime::compile_profile_aliases(annotations) -> Result<Option<BTreeMap<String,
String>>, String>` — the single compile path consumed by the validator, codegen, the
generation-side `@profiles` filter, and the interpreter. `main.rs`'s global
`normalize_grammar_profile_name` (`:2119-2130`) is DELETED; its call sites resolve through
the grammar's own compiled alias map (+ generic pass-through). This kills divergences
2 and 3 (§1) — no cross-grammar leakage, one case rule.

### D6 — Embedding side

The typed `GrammarProfile` enum STAYS (a public typed contract is not a name-gate). Its
`FromStr` alias arms are the third hand-copy; `.2` decides between:
(a) sourcing the arms from the generated constants (`SystemVerilogParser::
GRAMMAR_PROFILE_ALIASES`) under the existing `has_generated_*` cfg gates, or
(b) keeping the literal arms but adding a DRIFT GATE test asserting the embedding
spellings == the artifact constants (the duplicated-metadata doctrine's required shape
when a copy must exist — e.g. if cfg-gating `FromStr` would break the no-features build).
Also in `.2` scope: `RegexDefault`'s canonical name (`"regex_default"`, a profile the
grammar never declares) — post-`DEFAULT-PROFILE.2`, mapping it to `set_grammar_profile
(None)` (restore-declared-default) is the principled resolution; verify
acceptance-equivalence with a probe.

### D7 — VHDL alias arms are deleted as inert

`grammars/vhdl.ebnf` declares zero `@profiles` rules, so VHDL profile aliasing gates
nothing anywhere. The VHDL arms in copies 2+3 are DELETED (delete-not-fallback), with a
`.2` probe proving behavior-neutrality (a VHDL parse/generation run with `--grammar-profile
1076-2019` before/after: same acceptance, since no guard consults it). The
`Vhdl1076_2019` enum variant + canonical name stay (typed surface). When VHDL someday
gains profile-gated rules, its aliases will be declared in `vhdl.ebnf` like everyone
else's.

### D8 — Out of scope

- Generation-side spacing policy (WS-DIRECTIVE D7) and the svpp stimuli separator
  heuristic (STIMULI-SIGNOFF `.12`) — unrelated residuals of the sibling trees.
- Any new profile capability (profile inheritance, wildcards) — aliasing only.
- The `GrammarFamily` enum and grammar-NAME registry dispatch — blessed typed/data-driven
  boundaries.

### D9 — Verification plan (leaf `.2`)

1. REPRODUCE probe (pre-fix): a synthetic `@profiles`-gated grammar declaring
   `@profile_alias` generates with the directive silently ignored (no carrier; alias
   request gates rules off instead of resolving).
2. Combinator-suite growth: an `alias_resolves` case (aliased request == canonical
   request, byte-identical accept) — count/shape decided with the suite's determinism
   constraints in view (the `.2` twin of DEFAULT-PROFILE D7 item 10).
3. The full SV lane matrix: canonical cert + union gate pins unchanged; `verilog_2005`
   conformance gate (its contract exercises profile spellings) green; SV external corpus
   14/14; alias probe matrix — every documented spelling × {parse, generation, embedding}
   resolves identically before/after.
4. Regen ALL parsers; the 10 non-SV artifacts `cmp` BYTE-IDENTICAL; the SV delta is
   exactly the alias carrier + setter resolution.
5. Equivalence gate 11 CERTIFIED; semantic + combinator gates green; features-on lib
   green; dual-run gate green (zero meta-grammar change).
6. Lockstep (same commit): ebnf parser book (directive section + catalog row), top book
   `annotation-system.md` + `parseability-probe-debug.md:89` + `embedding-and-downstream-
   integration.md:32/:38` (aliases become "grammar-declared"), `TOOLBOX.md:122`,
   `PGEN_ANNOTATION_NORMATIVE_SPEC.md`, `PGEN_SEMANTIC_STEERING_CONTROL_MATRIX.md`,
   SV parser book/contract if the alias list is stated there, continuity docs, this tree.

## 3. Tree

- `PROFILE-ALIAS.1` — DESIGN (this document; docs-only commit). **DONE**
  (`PGEN-PROFILE-ALIAS-0001`, session #56).
- `PROFILE-ALIAS.2` — IMPLEMENT end-to-end per D1–D7 + full lockstep + verification (D9).
  **FRONTIER.**

## 4. Tree-level acceptance criteria

1. `@profile_alias` compiles from the EBNF through ONE shared compile fn consumed by the
   validator, codegen, the generation-side filter, and the interpreter (no second table,
   no shadow copy without a drift gate).
2. `grammars/systemverilog.ebnf` declares the ten alias spellings; the `"systemverilog"`
   arm in `parser_registry.rs`, the global `main.rs` table, and (per the D6 decision) the
   hand-copied embedding arms are DELETED or drift-gated.
3. Behavior preserved: every documented alias spelling resolves identically on all three
   boundaries (probe matrix); SV cert/union/v2005-conformance/external-corpus lanes
   unchanged; case-handling divergence (§1.3) closed to ONE rule with the delta pinned.
4. The 10 non-SV parsers regenerate BYTE-IDENTICAL (`cmp`); only the SV artifact changes,
   and only by the D3 carrier/setter delta.
5. The capability is OPEN to any grammar: a synthetic `@profiles`-gated grammar with
   `@profile_alias` resolves the aliased request to the canonical profile, interpreter
   and compile-and-run oracle agreeing byte-for-byte.
6. The §1 divergences 1–4 are all closed (VHDL arms deleted-inert, no cross-grammar
   leakage, one case rule, `regex_default` resolved principled).
7. Books / normative spec / steering matrix lockstep in the same commit.

## 5. Acceptance Checklist (enforced) — leaf `.2` (copy, tick with evidence when earned)

- [ ] **REPRODUCE / ISSUE** — pre-fix probe per D9.1 + the §1 grep evidence at HEAD.
- [ ] **ROOT CAUSE (WHY + WHERE)** — §1: the three spelling tables, file:line-pinned.
- [ ] **FIX** — D1–D7; fix-hierarchy level 3 (new annotation) + parser-agnostic wiring.
- [ ] **ADDRESSED (verified)** — alias probe matrix before→after; combinator case CLEAN.
- [ ] **NO REGRESSION** — cert seeds 0/7/42; 10 parsers `cmp` byte-identical; SV lanes
      (canonical/union/v2005/corpus 14/14); equivalence/semantic/combinator gates;
      features-on lib; clippy source-strict.
- [ ] **LOCKSTEP** — D9.6 list, same commit.

## 6. Findings routed elsewhere (surfacing directive)

*(none yet)*

## 7. Verification log

- 2026-07-07 (session #56, `.1`): §1 fact base established by direct reads/greps at HEAD
  `b163330b` — the three spelling tables + all call sites (`parser_registry.rs:149-168` +
  wrapper invocations; `main.rs:2119-2130` + `:2153/:2169/:2180/:2280`;
  `embedding_api.rs:285-305/:227-235/:1425`); the `@profiles` census (SV 392 / regex 6 /
  vhdl 0 — the VHDL-inertness evidence); the four cross-copy divergences; the
  user-facing alias documentation surface (`parseability-probe-debug.md:89`,
  `embedding-and-downstream-integration.md:32/:38`, `TOOLBOX.md:122`); the precedent
  chain (`compile_default_profile` at `semantic_runtime.rs:310-359`, directive-registry
  ParserSteering entries at `semantic_directive_registry.rs:371-388`, the
  `DEFAULT_GRAMMAR_PROFILE` carrier + `default_generated_grammar_profile` at
  `parser_registry.rs:142-148`).
