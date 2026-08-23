# Grammar Certification Status

**One question, answered for every shipped grammar: is it certified, and if not, why.**

> This page is **DERIVED BY MEASUREMENT**. The table is produced by
> [`scripts/report_grammar_certification.sh`](../../../scripts/report_grammar_certification.sh),
> which **runs the certificate-coverage oracle** against the generated parsers in the tree. Do not
> hand-edit it — re-run with `--markdown` and republish;
> `--check docs/book/src/grammar-certification-status.md` refuses when the two disagree.
> ⭐ **Since 2026-08-23 this is ENFORCED, not merely available**: `GRAMMAR-CERT-CURRENCY` is a
> registered doctrine and runs on every commit. See *Kept in sync, mechanically* below.

## The definition

PGEN's own, from `GRAMMAR-WELLFORMED.G.4` — the linter⟷generator duality capstone — quoted from the
oracle's own help text:

> For every rule, is it covered by a verified unreachability **PROOF** or a verified reachability
> **WITNESS** (a clean diverse `--count` sample that parses through the real parser and exercises
> it)? `UNKNOWN`=0 with no failures = the objective *"trustworthy on this grammar"* number.

So a grammar is **certified** iff, for every rule, either a verified proof says no input can reach
it, or a generated sample witnesses it **through the real generated parser** — with `UNKNOWN=0`,
`sample_parse_failures=0` and `proof_reverify_failures=0`.

⛔ **It is not a correctness claim about the language.** A certified grammar can still accept invalid
input or reject valid input; that is the *corpus* axis, tracked per family in
[Roadmap and Live Status](roadmap-and-live-status.md). SystemVerilog's corpus axis currently reads
**46.3 % adjudicated** with **275** known defects.

## Status

<!-- BEGIN DERIVED: scripts/report_grammar_certification.sh --markdown -->

Measured at seed 0, `--count 40`, against the generated parsers in the tree.

| grammar | certified | coverage | notes |
|---|---|---|---|
| `json` | ✅ **yes** | 9 rules · 9 witness · 0 proof · 0 unknown | every rule proven or witnessed |
| `regex` | ✅ **yes** | 269 rules · 260 witness · 9 proof · 0 unknown | every rule proven or witnessed |
| `return_annotation` | ⛔ **no** | 35 rules · 33 witness · 0 proof · 2 unknown | **2 rule(s) UNKNOWN** — no generated sample reaches them and no proof covers them |
| `rtl_const_expr` | ✅ **yes** | 48 rules · 48 witness · 0 proof · 0 unknown | every rule proven or witnessed |
| `rtl_frontend` | ✅ **yes** | 169 rules · 168 witness · 1 proof · 0 unknown | every rule proven or witnessed |
| `semantic_annotation` | ⛔ **no** | 119 rules · 90 witness · 0 proof · 29 unknown | **29 rule(s) UNKNOWN** — no generated sample reaches them and no proof covers them |
| `systemverilog_preprocessor` | ✅ **yes** | 74 rules · 74 witness · 0 proof · 0 unknown | every rule proven or witnessed |
| `vhdl` | ✅ **yes** | 225 rules · 225 witness · 0 proof · 0 unknown | every rule proven or witnessed |
| `systemverilog` | ⚠️ **unverified** | 1385 rules · 1378 witness · 7 proof · 0 unknown (canonical unknown 11) | from the tracked union contract, **not re-measured here** (~2 min/seed × 3 seeds × 4 configs). Proof freshness vs the parser in the tree: **STALE** |

**6 of 9 shipped grammars are certified** — every rule proven or witnessed, UNKNOWN=0.

<!-- END DERIVED -->

## The two that are not certified — and exactly why

**31 rules** are uncertified across the two families. Every one of them is classified below, and
none of the classes is *"we don't know"*. The counts **and the rule sets** reproduce identically at
seeds 0, 7 and 42, so this describes the grammars rather than one sample.

### `return_annotation` — 2 unknown of 35

| rule | why it is unknown |
|---|---|
| `accessor_base` | ⭐ **Not a defect.** PGEN's own left-recursion elimination pass *replaced* it: the shipped parser carries `parse_accessor_base_lr_base` and `parse_accessor_base_lr_suffix`, which is where the language actually lives. Certifying the pre-elimination name would mean certifying a rule the parser deliberately does not have. |
| `parenthesized` | **A dead rule.** It is defined once and referenced by nothing, so no input can reach it from the entry. |

### `semantic_annotation` — 29 unknown of 119

| class | count | why |
|---|---|---|
| **Whitespace and comments** | 4 | `whitespace`, `line_comment`, `block_comment`, `doc_comment` are lexical constructs the grammar itself marks *"(ignored)"*. No production reaches them and none should. |
| **A duplicate of the entry rule** | 1 | `annotation` has a right-hand side **byte-identical** to the entry rule `semantic_annotation`. It is unreachable because it is a second copy of the start symbol. |
| **Specialized value shapes that are written but not wired in** | 24 | Six complete feature islands — precedence, constraint, performance/complexity/memory/timing, version, exception, platform, plus union and intersection types — none of which `annotation_value` routes to. It offers exactly `primitive_value`, `structured_value`, `expression_value` and `reference_value`. |

⭐ **The third class is a real product question, not a bug.** Wiring those shapes in would make
`@precedence: 5 left` parse *structurally* rather than as a generic value — which changes the AST
shape a downstream consumer reads for those annotations. It is therefore a scope decision with a
contract consequence, and it is recorded as an open question rather than settled quietly.

⛔ **Two of the 31 are genuine dead weight** — `annotation` and `parenthesized` — and removing them
would take these families to 28 and 1 unknown respectively without changing the accepted language.
Both are owned by `GRAMMAR-CERT-STATUS.4`; the edit regenerates the annotation parser pair that
every other family's generation depends on, so it is sequenced as its own slice rather than folded
into a status update.

⭐ **These numbers track live development, which is the point of deriving them.**
`semantic_annotation` moved `115 / 84 witnessed / 31 unknown` → `119 / 90 / 29` when
`GRAMMAR-WELLFORMED.H.16.6b` added the `map_key` rules: the total rose because the grammar grew, and
unknown fell because more rules became witnessed.

## SystemVerilog, stated precisely

Its accounting is a **union over four** entry/profile configurations
(`systemverilog_file:sv_2023`, `sv_multi_entry_root:sv_2017`, `library_text:sv_2017`,
`systemverilog_parseable_file:sv_2017`), not a single run. Under that union the last recorded run
reached **0 unknown**; under the **single canonical config, 11 rules remain unknown**, and **7 rules
are credited by PROOF rather than by a generated string**.

⚠️ That result is **not verified against the parser now in the tree**: the contract pins one
`systemverilog_parser.rs` digest and the tree holds another, because engine-universal codegen moved
under it (the grammar itself did not — 0 commits). Re-running the gate is owned by
`SV-CORPUS-GRAD.13c.2x.10`.

## ⛔ Correction, 2026-08-23

The first version of this page reported **`certified_and_fresh = 0/9`** with seven families as
`NO ORACLE`. **That was wrong.** It asked *"does a `*cert*contract*.json` file exist for this
family?"* — a filesystem question — when the certificate-coverage **report is** the oracle and a
tracked contract is only a pin on top of it. Five families that certify cleanly were reported as
never scored. ⇒ **ask the instrument, not the filesystem.** The table above is now produced by
running the oracle.

## ⛔ Correction, 2026-08-23 — the second one, on the same page

The fix above shipped with a defect of its own, found the same day and owned by
`GRAMMAR-CERT-STATUS.1b`: the rewrite deleted the `--check` **implementation** but left the **flag**
in the argument parser. `--check` went on being accepted, printed a fresh table to stdout and
**exited 0 without ever opening this page** — so the sentence at the top of this page,
*"refuses when the two disagree"*, described a capability that had been deleted. Four arms measured
at that commit — this page, a page with a corrupted table, a page with no derived block, and a path
that does not exist — **all exited 0 and were mutually indistinguishable.**

⇒ **deleting an implementation while leaving its flag is worse than deleting the flag too**: an
unknown-argument error would have been loud on the very next run. Now restored, with five arms
observed: in sync → `OK`; drifted → exit 1 with a unified diff naming the row; no derived block →
refuse; missing page → refuse; an empty derivation → refuse rather than compare against nothing.

## Kept in sync, mechanically

A derivation nothing re-derives is a claim that rots — which is exactly how the roster this page
replaced went stale. So `GRAMMAR-CERT-CURRENCY` (doctrine 26) holds the page to the tree, in two
tiers, because the oracle costs about a minute and a pre-commit hook may not:

| tier | when | cost | what it proves |
|---|---|---|---|
| **1** (default) | **every commit**, via `.githooks/pre-commit` | **0.27 s** | the page carries exactly one well-formed derived block; the producer's `--check` is genuinely **wired**; the published **population** equals the families that actually ship; and no input the table depends on has run ahead of the page beyond a budget |
| **2** (`--oracle`) | on demand | **64.5 s** | the whole table is **re-derived and diffed** against the published block — after first proving the producer still **refuses** a page it cannot read |

```bash
bash scripts/check_grammar_certification.sh              # tier 1 — the doctrine
bash scripts/check_grammar_certification.sh --oracle     # tier 2 — re-derive and diff
bash scripts/check_grammar_certification.sh --self-test  # prove the refusal arms fire (9/9)
```

⚠️ **Stated plainly, because a gate's limits belong beside its green tick**: tier 1 is a *staleness*
argument, not a correctness proof — it inherits whatever tier 2 last established. And because
`generated/` is not tracked, a fresh clone reports the population arm as **NOT EVALUATED**, loudly,
never as a pass.

⭐ **Tier 1 tests the instrument rather than trusting it**, and that is not defensive
over-engineering — it is this page's own history. The check that keeps this table honest was once
deleted while its flag survived, and nothing noticed for a commit. Two of the nine self-test arms
reconstruct that exact failure from both sides.

## Re-derive it yourself

```bash
bash scripts/report_grammar_certification.sh              # human-readable
bash scripts/report_grammar_certification.sh --markdown   # the table above
bash scripts/report_grammar_certification.sh --check docs/book/src/grammar-certification-status.md
```

Exit codes follow the repository convention: **0** the published table equals a fresh derivation,
**1** it has drifted (a unified diff naming the row is printed), **2** the check could not evaluate
its subject at all — a missing page, or one carrying no derived block.
