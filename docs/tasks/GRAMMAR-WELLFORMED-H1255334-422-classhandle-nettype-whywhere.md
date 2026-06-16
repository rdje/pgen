# GRAMMAR-WELLFORMED.H.12.5.5.3.3.4.2.2 — class-handle head WHY+WHERE (a real store-gating parser bug)

Tools-first WHY+WHERE for the secondary finding `-0098` (`.4.2`) surfaced: a declared **class-handle**
head does not witness the indexed member-method chain. Slice `PGEN-GRAMMAR-WELLFORMED-0099`
(PURE-DOCS investigation). Opened by `H.12.5.5.3.3.4.2`. **Conclusion: a real released-parser bug
in the M2 store-gating family** — fix routed to child `.4.2.2.1`. No code change in this slice.
([[feedback_grammar_rules_must_consult_store]], [[feedback_why_and_where_before_solution]],
[[feedback_tools_first_no_guessing]], [[feedback_be_alert_root_cause_fishy_immediately]].)

## The symptom (from `-0098`)

| head decl | chain | result | route |
|---|---|---|---|
| `int a;` / `logic a;` / `bit a;` | `a.b[0].c()` | PARSE OK | `context_member_method` |
| `C a;` (C a *declared class*), module scope | `a.b[0].c()` | **REJECT** (at the `[`) | — |
| `C a;` (C a declared class), **class scope** (class property) | `a.b[0].c()` | **PARSE OK** | `context_member_method` |

The chain witnesses whenever the head `a` is bound as a *variable*. The reject is specific to a
class-handle head **at module scope**.

## WHY — `C a;` at module scope is mis-parsed as a `net_declaration`

`parseability_probe --parse-dump-ast-pretty` on `class C; endclass module m; C a; endmodule`:
the declaration of `a` carries `"kind": "net_declaration"` (whereas `int a;` carries
`data_declaration` → `variable_decl`). A `net_declaration` does **not** run
`variable_decl_assignment`, so it never emits the `variable_binding` fact for `a`. Therefore
`context_member_method_call`'s `@predicate has_fact(variable_binding, $head)` (phase post) fails for
the class-handle head, and the indexed 3-level chain falls through every `call_primary` sibling →
full-input reject. (A non-indexed `a.b.c()` survives via `ident_postfix_chain`/`split_hierarchical`,
which need no binding — which is why only the *indexed* chain exposes the gap.)

## WHERE — `checked_nettype_identifier`'s store gate is UNDER-specified

`--trace-rules` on the same input: `checked_nettype_identifier` matches `C` (consumes ` C`) and
`net_declaration_sv_2017` branch 2/4 succeeds. The rule and its gate (`systemverilog.ebnf:3310-3311`):

```ebnf
@predicate: { name: has_fact, args: [type_name, $body], phase: post }
checked_nettype_identifier := declaration_identifier -> { body: $1.body }
```

Reached via the `net_declaration` user-nettype branch (`:3225`):
`| checked_nettype_identifier ( delay_control )? list_of_net_decl_assignments semi`
(comment `:3224`: *"a user net-type identifier must be a KNOWN declared type/nettype (sound
store-gate)"*).

**The defect:** the gate is `has_fact(type_name, $body)` — it accepts the identifier as a nettype if
it is *any* declared type. But a **class** is also a `type_name` (`class C` emits `type_name` with
`declaration_family: class`), so a class name *satisfies the gate* and is consumed as a nettype. The
rule's own neighbouring comment (`:3313-3315`) states the intent — a nettype declaration emits
`type_name` *with `declaration_family: nettype`* so a later `checked_nettype_identifier` "resolves" —
but the predicate never checks `declaration_family`, so it does not actually distinguish a nettype
from a class (or any other type). This is the classic store-consultation defect: the rule consults
the store but with too weak a predicate ([[feedback_grammar_rules_must_consult_store]]).
`checked_nettype_identifier` is itself in the cert-coverage UNKNOWN set, consistent with it being a
suspect rule.

LRM grounding (single source of truth): a `net_declaration` requires a `net_type` — a built-in net
(`wire`/`tri`/…) or a **user-declared nettype** (`nettype data_type T;`). A class is not a nettype, so
`C a;` (C a class) is a `data_declaration` (a class-handle variable), never a `net_declaration`.

## Fix direction (verified-sound; → child `.4.2.2.1`)

Tighten the gate to require the nettype `declaration_family`, mirroring the proven
`known_unscoped_block_class_type` / `checked_type_identifier` pattern the comment cites:

```ebnf
@predicate: { name: fact_attribute_equals, args: [type_name, $body, declaration_family, nettype], phase: post }
checked_nettype_identifier := declaration_identifier -> { body: $1.body }
```

So a class (or any non-nettype) name no longer matches as a nettype; `C a;` then routes to
`data_declaration` (class variable), binds `a`, and the member-method chain parses — exactly as it
already does in class scope.

**Both preconditions tool-verified this session (so the fix is non-regressing in principle):**

- **P1 — real nettypes preserved:** `nettype logic NT; module m; NT a; endmodule` → PARSE OK today;
  `NT` is declared with `declaration_family: nettype`, so the tightened `fact_attribute_equals` gate
  still matches it.
- **P2 — class var binds + chain witnesses when not stolen by net:** `class C; endclass class D; C a;
  … a.b[0].c() …` (class property — `net_declaration` is not a `class_item`, so only
  `data_declaration` can match) → PARSE OK, AST node `context_member_method`. This is precisely the
  outcome the module-scope fix reproduces once the net branch stops stealing `C a;`.

This is a **language-changing grammar fix** (`C a;` at module scope: `net_declaration` →
`data_declaration`): full lockstep required in `.4.2.2.1` — regen SV parser, cert seeds 0/7/42
(GLOBAL no-regress; likely *also* witnesses `context_member_method_call` for a class-handle prelude
and may close other M2 nettype rules), external corpus 14/14, `stimuli_cross_family_platform_gate`,
`--lint-grammar`, SV release bump + `PGEN_RELEASED_PARSER_BUG_LEDGER` row + SV contract/parser-book +
grammar-wellformedness book narrative + AST-shape/schema check. Real released-parser bug ⇒ HIGH
priority (fix parser bugs ASAP). This is squarely the M2 store-gating class (`H.12.5.6`); track the
fix child under that umbrella's discipline (the over-gen-vs-bug adjudication is *done* here — it is a
real bug).

## Conclusion

- WHY: `C a;` (C a declared class) at module scope is consumed by `net_declaration` because
  `checked_nettype_identifier` accepts any `type_name` (including a class) as a nettype; the net path
  emits no `variable_binding`, so `context_member_method_call`'s post-predicate fails and the indexed
  chain rejects.
- WHERE: `systemverilog.ebnf:3310` — `@predicate has_fact(type_name, $body)` on
  `checked_nettype_identifier` (`:3311`); under-specified (missing `declaration_family: nettype`).
- FIX: `fact_attribute_equals(type_name, $body, declaration_family, nettype)` → child `.4.2.2.1`
  (grammar fix, language-changing, full lockstep, M2 store-gating family). Preconditions P1/P2 verified.
- This does NOT block the `.4.2.1` cert witness for `context_member_method_call` (the integral-head
  witness from `-0098` already proves the rule reachable + witnessable).

## Artifacts (this session, scratch — not tracked)

`rust/target/generated_logs/h1255334_2/` — `d_C_decl.sv`/`.ast.json` (the net_declaration mis-parse),
`d_C_decl.trace.txt` (`checked_nettype_identifier` matching `C`), `d_int_decl.ast.json` (the
data_declaration baseline), `p1_nettype.sv`, `p2_classprop.sv`, `p2_member_chain.sv`/`.ast.json`
(class-scope witness).
