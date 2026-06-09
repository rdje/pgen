---
name: feedback_fix_parser_bugs_asap_highest_priority
description: "STANDING, emphatic (director 2026-06-09): fixing a clearly-identifiable PARSER BUG is the HIGHEST priority — drop in-progress polish (incl. cert-coverage) and fix it ASAP. The stimuli generator is a bug-finding ORACLE; when it surfaces a parser bug, that bug outranks making the generator's own output prettier."
metadata:
  node_type: memory
  type: feedback
---

Director, 2026-06-09 (emphatic): **"There is no point on cert-cover if a clearly
identifiable parser bug shows up. The highest priority is always to fix parser bugs
ASAP!"**

**Why:** parser correctness is the product. A parser that accepts erroneous input (or
rejects valid input) is a defect that outranks every quality-polish task — including
cert-coverage / stimuli-generator faithfulness work. The stimuli generator's *value* is
precisely that it finds such parser bugs (it produces inputs no human would hand-write,
which expose latent parser permissiveness). So when cert-coverage / the generator surfaces
a parser bug, the response is not "keep polishing cert-coverage" — it is "stop, fix the
parser bug." cert-coverage is a bug-finding oracle, not the end in itself.

**How to apply:** the moment a tool (cert-coverage, a corpus, a differential run, a trace)
surfaces a *clearly-identifiable* parser over-acceptance or false-rejection, RE-PRIORITIZE:
root-cause it tools-first ([[feedback_be_alert_root_cause_fishy_immediately]],
[[feedback_tools_first_no_guessing]]), open/own a task-tree leaf, and fix it before
resuming the polish task that found it. Defer the polish task explicitly (don't drop it —
park it behind the parser fix). Trigger/exemplar: the SV `endmodulemodule b;`
over-acceptance found by the `GRAMMAR-WELLFORMED` cert-coverage diverse pass, which
pre-empted the cert-coverage generator-faithfulness fix and spawned the `SV-PARSE-STRICT`
tree. Composes with [[project_sv_parser_elaborator_boundary]] and
[[feedback_grammar_rules_must_consult_store]].
