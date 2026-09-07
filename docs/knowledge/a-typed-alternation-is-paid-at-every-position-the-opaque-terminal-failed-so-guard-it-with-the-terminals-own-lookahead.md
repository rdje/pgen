---
id: a-typed-alternation-is-paid-at-every-position-the-opaque-terminal-failed-so-guard-it-with-the-terminals-own-lookahead
title: Replacing one opaque terminal with an alternation of typed productions is paid at every position the terminal was TRIED and failed, not where it matched — the per-rule parse-cost table names the positions, one positive lookahead on the shape the terminal already certified makes the typed spelling cheaper than the opaque one, and the parse-cost ratchet accepts the typed frames only under an invariant that refuses the unguarded spelling by construction
answers:
  - "typing the compiler directives made the SV parse-cost ratchet refuse with entries +0.36 % and memo_hits +0.25 % — where did the cost go"
  - "why does a typed alternation of thirteen directive productions cost more than the opaque regex it replaced at positions with no directive"
  - "how do I attribute a parse-cost rise to the positions that pay it — rule_costs.tsv HEAD vs fresh"
  - "what does &opaque_directive_line guard in source_text_item, class_item and non_port_module_item"
  - "why is compiler_directive @branch_policy ordered with the opaque residue FIRST"
  - "does a positive lookahead's match count as a commit in the parse-cost counters"
  - "which parse-cost invariant covers a committed rise where pre-existing tokens (time_unit, kw_ns, trivia) commit more"
  - "what is typed_replacement_of_opaque_terminal — the twelfth parse-cost invariant"
  - "is a parse-cost fall enough to accept a leaf's grammar change when committed still rises"
  - "how much does a typed AST cost per node in the parse-cost sample"
tags: [systemverilog, grammar, parse-cost, ratchet, invariants, lookahead, branch-policy, directives, instruments]
date: 2026-09-07
status: current
evidence: "SV-CORPUS-GRAD.3.39 (release 1.0.217, ledger SV-0090). PARSE-COST-RATCHET against 1.0.216 on the pinned sample: the first typed spelling read entries 403,247,088 -> 404,701,890 (+1,454,802, +0.36 %), memo_hits 175,051,163 -> 175,488,328 (+437,165), committed +264; rule_costs.tsv HEAD vs fresh: trivia +437,260 entries (+437,245 memo hits), kw_dir_pragma +143,944, each of the seven in-scope directive productions and their kw_dir_* tokens +71,972, lrm_compiler_directive and its file-level siblings +601 — and in_scope_compiler_directive commits 0 times on the sample (rust/target/s339/step_cost_ratchet.log, rule_costs.tsv). Guard-only (each host's directive alternative behind &opaque_directive_line, body: $1 -> $2): entries 403,113,991 (-133,097 vs 1.0.216), memo_hits 174,982,736 (-68,427), committed +685 = opaque_directive_line 361 (174 residue commits + 187 lookahead matches) + lrm 13 + 48 typed frames + trivia 202 + line_comment 47 + time_unit 6 + kw_ns/kw_ps 3+3 + the two separators 3+3 - compiler_directive 2 - source_text_item 2 (rust/target/s339/step_cost_guard_only.log, rule_costs_guard_only.tsv); interpreter A/B over the 41 directive pins x 3 profiles: 29 accepting arms byte-identical, 94 rejecting arms at identical positions, for the guard and again for the residue-first order. typed_replacement_of_opaque_terminal in scripts/parse_cost_containment.py (14 self-test arms) HOLDS on the guard-only table with the real HEAD and candidate graphs (84-rule typed sub-graph, 7 in-scope committed risers, 0 outside, entry rises +108 covered by 212,437 retired) and its coverage clause refuses the unguarded table (+437k trivia rise against 72k retired)."
reverify: "PGEN_PARSE_COST_REMEASURE=1 bash scripts/check_parse_cost_ratchet.sh 2>&1 | grep 'BINDING metric'   # entries and memo_hits FELL or hold vs the promoted 1.0.217 baseline; python3 scripts/parse_cost_containment.py --self-test 2>&1 | grep TYPED   # arms=14 failed=0; grep -n '&opaque_directive_line' grammars/systemverilog.ebnf   # three hosts"
---

**Where the cost of a typed alternation sits.** An opaque terminal `` /`[^\r\n]*/ `` tried at an item
position that holds no directive costs one memoized `trivia` lookup and one regex miss. Replace it
with an alternation of seven typed productions and the same position costs seven rule frames, eight
`kw_dir_*` regex attempts (an ordered rule tries its leading token once per alternative) and eight
memoized lookups — and the corpus has 71 972 such positions on the parse-cost sample against
fifty-one that hold an in-scope directive line and none that commits one. The rise the ratchet
refused (+0.36 % of every entry the parser makes) was entirely speculation at positions the
construct can never match. Nothing about the typed productions themselves was expensive; what was
expensive was ASKING at every position.

**How the positions were named.** Not from the totals — three totals cannot distinguish "the new
construct speculates inside its own sub-graph" from "the parser speculates everywhere" — but from
the per-rule table the ratchet keeps beside them (`rule_costs.tsv`, HEAD vs fresh): `trivia`
+437 260 entries of which +437 245 memo hits (every typed token attempt buys one prefix lookup),
`kw_dir_pragma` +143 944 (exactly twice the others: `pragma_directive` is ordered and tries its
token in both arms), each in-scope production and its token +71 972, the file-level siblings +601,
and the in-scope host committing zero times. Read that way the table is a map of where the parser
asks and is told no.

**The fix is the terminal's own shape as a guard.** Every host — `source_text_item`, `class_item`,
`non_port_module_item` — spells its directive alternative as `&opaque_directive_line …` with
`body: $1 → $2`: one positive lookahead on the backtick line the residue already certifies, which
fails at exactly the price the opaque regex used to fail at and admits the alternation only where a
directive line starts. Inside `compiler_directive` the residue comes FIRST under
`@branch_policy: ordered`, because the two arms are disjoint by the negative lookahead
`!lrm_directive_name` (no verdict can move) and the common case is an opaque line — on the sample
174 opaque lines commit against 13 typed, and the tournament had tried thirteen typed productions
at every one of them. Guard alone: entries −133 097 and memo hits −68 427 BELOW the 1.0.216
baseline — the typed spelling is cheaper than the opaque one it replaced.

**What a typed AST still owes, and what accepts it.** A lookahead's match counts as a commit
(`opaque_directive_line` commits 361 times: 174 residue lines plus 187 guard matches), a typed
derivation commits one frame per production and per token where the regex committed one, and the
pre-existing tokens the derivation now consumes one by one — `time_unit`, `kw_ns`, `kw_ps`, the
separators, `trivia`, the `line_comment` the regex used to swallow — commit more. Every earlier
invariant is right to refuse a pre-existing `committed` rise, so this shape has its own, the
twelfth (`typed_replacement_of_opaque_terminal`): every pre-existing rule that commits or is
entered more must sit INSIDE the typed sub-graph reachable from the introduced rules; those consumed
commits are at most the introduced frames; `Δcommitted` is exactly introduced + consumed − lost;
the pre-existing entry rises are covered by what the guard retired; the memo rise is at most the
typed attempts. Its coverage clause refuses the unguarded spelling (a +437 k `trivia` rise against
72 k retired entries) — the lesson is code, not a commit message. Honest bound, in the predicate's
docstring: a typed AST costs one frame per node by construction, and a dispatch on ONE directive-name
token would retire the remaining attempts at real directive lines; neither is proven minimal here.
