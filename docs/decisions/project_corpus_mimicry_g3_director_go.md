---
name: project-corpus-mimicry-g3-director-go
description: "Director GO (2026-07-07, session #58, emphatic 100% agreement): corpus-mimicry (STIMULI-SIGNOFF goal G3) is an APPROVED direction — learn a generation distribution from a real-world corpus (e.g. UVM SV, real regexes) and generate statistically similar, REALISTIC stimuli. The director had independently wanted this capability."
metadata:
  node_type: memory
  type: project
---

**Context.** During the `STIMULI-SIGNOFF.4` FDLOOP design slice (session #58,
`PGEN-STIMULI-SIGNOFF-0009`), the learned-distribution mechanism surfaced a genuinely novel
capability direction not in any prior gap list — **corpus-mimicry (goal G3)**: use the
`.4.1` learned-distribution layer with the external-corpus learning front-end (the gen-AST
interpreter as parser-agnostic derivation counter) to *learn the per-choice-point branch
distribution from a real-world corpus and generate statistically similar, realistic stimuli*
— realistic-workload generation for downstream consumers (e.g. Nexsim), beyond coverage
pushing. Fitness = per-group distribution proximity (design doc
`docs/tasks/STIMULI-SIGNOFF-4-fdloop-directed-generation-design.md` §3.3 G3 / §6).

**Decision (director, 2026-07-07).** Explicit, enthusiastic GO — "You have my go here, and I
agree with you 100% with that decision you proactively took." The director notes he had the
same idea weeks earlier but was unsure it was achievable: "learn the distribution from a
real-world corpus (e.g. UVM SV, real regexes) and generate statistically similar, realistic
stimuli is really one way to generate highly realistic samples, stimuli."

**Consequences / how to apply.**
- G3 corpus-mimicry is APPROVED scope for the `STIMULI-SIGNOFF.4` tree — no further
  authorization needed when `.4.3` (external-corpus learning + mimicry demo) is picked.
- Realistic-stimuli generation is a *valued product direction* in its own right (not merely a
  coverage tactic): weigh it accordingly when ordering future generator work and when writing
  the book's user-facing framing.
- Candidate first corpora: the PCRE2 regex corpus bundle (`regex_corpus_bundle/`) and the SV
  external corpus files (UVM et al.). Keep it parser-agnostic per
  [[feedback_ast_pipeline_parser_agnostic]]; the learning front-end must work for ANY grammar.
- Related: [[project_stimuli_generator_signoff_vision]] (the umbrella vision this serves),
  [[project_external_corpus_doctrine]] (external corpora as a confidence source — G3 turns
  them into a generation TEACHER as well).
