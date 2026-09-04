# `verilog_2005` keyword-faithfulness census

Derived by `stimuli/sv/v2005_keyword_faithfulness_census.py`; do not hand-edit.

## Instrument identity

Re-hash these inputs; if any hash differs from the row below, **this report no longer
describes your tree** and the honest act is to re-measure, not to quote.

| input | sha256 |
| --- | --- |
| `grammars/systemverilog.ebnf` (ebnf_raw_ast) | `1e2ad24a6550f631b0a712fb5f40d5eca70d0920915653cb868af1dca722ccef` |
| `docs/verilog/2005/md/section-Annex_B-normative-list-of-keywords.md` | `08ba9745b0f0791ad48b786284b733414ab26ce7b2b9373d95174d094218025d` |
| `docs/tasks/artifacts/sv_corpus_grad/v2005_keyword_faithfulness/witnesses/MANIFEST.tsv` | `cf3138dcaeb0f7609ea8b028202ff8b4ae19ea19f2c255d407b573d13893c6ce` |

## Population

- rules in the source grammar: **1545**
- of those, SATISFIABLE under `verilog_2005`: **1127** (the bottom-up half — *can this rule derive a string here*)
- of those, REACHABLE under `verilog_2005` from the 4 declared entry roots: **792** (the top-down half — *can a parse get here*)
- **satisfiable but UNREACHABLE: 335** — rules the profile admits that no `verilog_2005` input can arrive at. `kw_void_e9cede9b` is the worked example: the terminal is ungated and trivially satisfiable, while both rules that reference it are `_sv_only`. ⭐ **This is NOT a gap in the certificate accounting, and an earlier version of this report said it was.** The engine already crosses the two directions: `gather_verified_profile_proof_covered_rules` (`VERILOG-2005-PROFILE.6.7`) classifies the WHOLE active rule set, and every profile-entry-unreachable rule gets an explicitly RE-VERIFIED `ProfileEntryUnreachable` certificate — a re-verify failure is reported as a linter bug, never silently covered. Measured: all of these rules appear in the cert pass's `proof` category, exactly. The number below is useful for understanding the profile's live surface; it is not a defect.
- not adjudicated here: **15** `_lr_*` rules the indirect-left-recursion eliminator synthesises, which exist in no `.ebnf` file and so have no source-graph edges. The certificate denominator counts them too, which is why it reads 1142 where this row reads 1127.
- keyword literals reachable under `verilog_2005`: **140**
- of those, reserved by IEEE 1364-2005 Annex B: **124**
- of those, NOT reserved by Annex B: **16** (**11** word-shaped, **5** single-letter / symbol terminals used by the UDP tables, edge descriptors and `PATHPULSE$`, which are 1364-2005 terminals rather than keywords)

### Profile sensitivity — the census carries its own control

The same computation under `sv_2017`, where these keywords legitimately
live, gives the denominator this profile is being judged against:

- word-shaped IEEE-1800-only keywords reachable under `sv_2017`: **136**
- of those, `verilog_2005` GATES **125** and LEAKS **11**

⛔ A census blind to the profile would report the identical population under both, so
this row is what separates *"measured and mostly clean"* from *"the reachability
computation never read the profile"*.

## Verdicts

| verdict | count |
| --- | --- |
| **over-acceptance (confirmed, 3 legs)** | **1** |
| correctly gated | 24 |
| identifier-consumed (refuted by leg 3) | 6 |
| unreachable, no witness required | 1 |

### Confirmed `verilog_2005` over-acceptances

| keyword | witness |
| --- | --- |
| `class_qualifier` | `class_qualifier.sv` |

### Correctly gated (the control set — the census is not stuck on ACCEPT)

`chandle`, `clocking`, `const`, `dist`, `enum`, `extern`, `iff`, `inside`, `interface`, `join_any`, `join_none`, `local`, `matches`, `packed`, `rand`, `ref`, `static`, `std`, `string`, `type`, `unique`, `var`, `virtual`, `with`

### Refuted by leg 3 — the keyword was consumed as an ORDINARY IDENTIFIER

`new`, `null`, `randomize`, `super`, `this`, `tx_path_delay_expression`

None of these is reserved by IEEE 1364-2005, so accepting the text is CORRECT: the
parse binds it as a user identifier, not as a keyword. Legs 1+2 alone called every one
of them a defect.

## Honest bound

This is lens **L1** (keyword admission) only. Two further classes are structurally
invisible to it because neither introduces a keyword Annex B lacks:

- **L2 — position.** A genuine 1364-2005 keyword accepted where 1364-2005 never allows
  it. Measured: `automatic integer i;` at module scope (Annex A permits `automatic`
  only on `function`/`task`).
- **L3 — shape / cardinality.** The right keywords in the wrong arity. Measured:
  `reg [7:0][3:0] r;` — `reg_declaration` carries exactly one `[ range ]`.

Reachability is also a deliberate over-approximation in one remaining direction: a path
gated only by a parse-time `@predicate` or a lookahead still counts as reachable. That
can add false POSITIVES, which the probe legs remove; it cannot hide a true one.
