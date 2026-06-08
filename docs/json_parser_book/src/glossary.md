# Glossary

- **Simplified subset grammar** — a grammar that models the *shape* of a language without implementing
  every lexical/conformance rule. `json.ebnf` is one; it is honest about being one (see
  [Welcome](welcome.md)).
- **Certificate coverage / `fully_certified`** — the EBNF-internal quality property: every grammar
  fragment carries a checked reachability witness or unreachability proof, with zero `UNKNOWN`. `json`
  reaches `fully_certified=true`. This is *grammar-internal* well-formedness, distinct from external
  conformance. See the platform book's Grammar Well-Formedness chapter.
- **External corpus** — an officially-recognized test set authored independently of PGEN's grammar (for
  `json`, JSONTestSuite). The oracle that exposes the gap between the grammar and the real language.
- **Cons-list shape** — `[head, [next, [next, …]]]`: the nested list shape PGEN's `-> [$1, $3*]` return
  annotation produces for `members` and `elements`. Walk it recursively.
- **`y_` / `n_` / `i_`** — JSONTestSuite verdict classes: must-accept / must-reject / implementation-defined.
- **RFC 8259 / ECMA-404** — the JSON standard. `json.ebnf` does **not** currently conform to it (see
  [External-Corpus Characterization](external-corpus-characterization.md)).
