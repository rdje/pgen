---
id: a-search-path-entry-grants-every-name-in-it-not-the-one-it-was-derived-for
title: A search-path entry grants every name in the directory, not the one it was derived for — scope a derived resolution as narrowly as its justification, and guard the names it resolves by the side door
answers:
  - "I derived an include directory for one header — what else does the preprocessor now find there"
  - "why did my context derivation start resolving an ambiguous include silently"
  - "how can a widening of what an instrument resolves create a wrong answer that no number shows"
  - "where should a resolution that is justified per file live — in the suite table or the per-file table"
  - "what guard belongs with an include-path derivation"
  - "a tie between identical copies of a header — is it ambiguous"
  - "why does pipeline_context.py refuse with exit 4"
tags: [preprocessor, include-path, context-derivation, corpus-lane, guards, scope, silent-wrong-answers]
date: 2026-10-06
status: current
evidence: "SVPP-EXPANSION.4d (`PGEN-SVPP-EXPANSION-0038`). The context derivation learned to resolve a tie between byte-identical copies of a header; its first cut emitted the chosen directory suite-wide. The shadow guard written in the same slice refused the real tree on the first run: OpenTitan's top_darjeeling/ip_autogen/pwrmgr/dv/env/seq_lib, chosen for pwrmgr_vseq_list.sv (4 copies, one text), also holds pwrmgr_base_vseq.sv and pwrmgr_smoke_vseq.sv (3 copies, 2 texts) — every OpenTitan includer of those two would have been handed darjeeling's copy with no number moving in the wrong direction (a refusal becoming a pass reads as progress). Redesigned: the identical tie resolves PER FILE; a per-file directory that holds an ambiguous name its includer also includes is withheld for that includer (one real case, the ip_templates pwrmgr vseq list); a suite directory in that position refuses the derivation. Before the slice the tracked table had 0 such pairs: the hazard was latent, not live, and nothing had read for it"
reverify: "python3 stimuli/sv/pipeline_context.py --self-test 2>&1 | grep -E 'SHADOW|REFUSES|WITHHELD' | grep -c PASS  # expect 3: the shadow is reported, the suite form refuses (exit 4), the per-file form is withheld"
---

**A directory on an include search path is a capability, not a resolution.** The preprocessor that
receives `--include-dir D` will find EVERY name that exists under D, whatever name D was added for.
A derivation that adds D because it holds the one copy of `a.vh` has also, silently, decided every
other `` `include `` whose name D happens to hold — including names the same derivation had just
classified as AMBIGUOUS and promised not to guess.

PGEN's corpus lane derives its include context from the corpus itself and publishes the rule: a
unique directory is emitted, an ambiguous name is reported and never guessed. `SVPP-EXPANSION.4d`
widened the rule — a tie between copies of ONE text is not ambiguous, the choice cannot change the
expansion — and the first cut emitted the chosen directory for the whole suite. That is the moment
the capability outruns the justification: the justification was "for THIS name any copy is the
same"; the grant was "search this directory for everything".

- **The wrong answer makes no number move the wrong way.** A refusal that becomes an acceptance
  reads as progress on every counter the lane publishes; the only way to see it is to ask whether
  an emitted directory holds a name the derivation declared ambiguous. Before the slice the tracked
  table had 0 such pairs — the hazard was latent, and nothing read for it.
- **Scope the resolution as narrowly as its justification.** A per-name, per-includer justification
  belongs in the per-file table (one `--include-dir` for one file), never in the suite table. The
  redesign put the identical tie there, beside the nearest-candidate rule it extends.
- **The guard has two forms because the grant has two scopes.** A SUITE directory that holds an
  ambiguous name is a design error — the derivation refuses (exit 4) and names the pair; a PER-FILE
  directory that holds an ambiguous name its own includer includes is withheld for that includer,
  which keeps its honest refusal, and the withholding is reported. The real tree hit the second form
  once (an `ip_templates` sequence list that includes vseqs that are one text across the tops AND
  one that differs per top).
- **Write the guard in the slice that widens the grant, and run it before measuring.** The guard
  caught the first cut before a lane row was read; measured first, the slice would have published
  pass counts that were partly a guess.

Related: [[porting-a-predicate-to-a-new-population-is-a-re-derivation]] (the sibling defect of the same
slice — the includer population's suffix filter reused for the candidate population),
[[a-zero-in-a-census-can-mean-shadowed-not-absent-and-an-overlap-is-broken-by-a-guard-not-by-reordering]].
