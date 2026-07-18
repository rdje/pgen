<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_correctness_before_speed.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: feedback_correctness_before_speed
description: STANDING POLICY — correctness comes BEFORE speed; only after a parser fully and accurately parses its corpus without errors do we optimize; applies to every parser the pgen engine builds
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 8c2d85c8-f843-4500-981d-c2bbf763bdc7
---

**User-set policy (2026-05-24, emphatic + repeated):** correctness first, speed second. Universal. Applies to every parser pgen generates.

**⛔ HARDENED for the SPEED PHASE (director 2026-07-09, session #73, emphatic):** *"The speed effort shall
not compromise the feature accuracy and parity. Any speed optimization that creates regressions or breaks
parity shall be rejected without any second thought."* This is the non-negotiable operating rule for the
speed phase itself: correctness/parity is a FLOOR, never a variable to trade for latency. A candidate that
is faster but flips ANY correctness/parity oracle — one new PCRE2 false-accept/false-reject, a cert
`fully_certified` loss, an equivalence/duality/ast_shape/conformance break — does NOT land; revert it
immediately and find another lever, no deliberation and no "net win" rationalization. Enforced concretely
as the RGX-0078 tree's ⛔ HARD CONSTRAINT (every correctness oracle green at its pre-optimization value).

**REINFORCED + BROADENED (user 2026-06-03):** *"It is always functionality & accuracy then speed, in that order."* The user SUSPECTS pgen parsers may not run as fast as they possibly can — but speed work waits until a parser is accurate and does its job (parses its target language source). Implication: there is a **deliberate SPEED PHASE, across ALL parser families, that begins once each is accurate.** First instances now entering that phase: **regex** (conformant → `RGX-0078`, [[project_rgx_0078_regex_slowness_followup]], geomean PGEN/PCRE2 < 5×) and **SV's super-linearity** (the parser is accurate — corpus 14/14 — so its O(N²) `with_semantic_runtime_rule_transaction` clone is fair game: `PARSE-TERMINATION.3`, [[stateful-packrat-not-linear]]). Both are SPEED-on-an-already-ACCURATE-parser → in-scope; both must be PROFILED first (tools, not guessing) and must not regress accuracy/conformance.

**⭐ ELEVATED — SPEED is a FIRST-CLASS, CONTINUOUSLY-TRACKED deliverable (director 2026-07-11):**
*"PGEN needs to account speed as a first-class deliverable. Accuracy AND speed are as important, so both
need to be tracked, monitored — like milk on the fire. But there is an order: accuracy comes first, then
speed. Accuracy without speed is a toy, not usable in real life; speed without accuracy is nonsensical.
Accuracy must be achieved first AND MAINTAINED — not regressed through speed optimization."* Two shifts
from the prior framing: (1) speed is no longer a one-off "phase" bolted after accuracy — it is a
**standing, co-equal, continuously-monitored deliverable** (track a speed metric alongside the accuracy
oracles, watched constantly, so a regression is caught the moment it appears); (2) the ORDER is a strict
precedence, not a weighting — **accuracy is the immovable FLOOR** (achieved first, then never traded), and
speed is maximized ONLY on top of a maintained-accurate parser. A usable real-world parser needs BOTH:
accuracy-without-speed = a toy; speed-without-accuracy = nonsense. **Observed at SCALE:** RGX surfaced the
slowness by running the PCRE2 conformance suite over the **FULL PCRE2 test data** (not just the 8-pattern
bench) — so the speed metric should be measured/tracked over the full conformance corpus, and the
slowness is hypothesized SYSTEMIC across ALL PGEN-generated parsers (see
[[project_rgx_0078_regex_slowness_followup]] reframing). Toward the [[project_horizon_universal_parser]]
north star, a universal parser platform must be respected on BOTH axes.

**Frame of reference:**
- Commercial SV compilers parse `uvm_pkg.sv` (3MB, ~90K preprocessed lines) in **<1 second**.
- A chip design contains **hundreds to thousands** of SV/VHDL/Verilog files; full compilation must still be fast (commercial tools do whole-design elaboration in seconds-to-minutes).
- Our current state: uvm_pkg takes 99s and rejects with a real grammar error. That is **NOT "fast"** — it's "not catastrophic-backtracking anymore."

**The discipline:**
1. **Get to fully-clean parse FIRST.** Every file in the corpus (uvm, uvm_compat, veer, friscv, scr1, …) accepted without errors. Surface-position triage at PASS=14/14, not 10/14.
2. **Only THEN look at speed.** Profile, identify hot paths, optimize.
3. **Don't conflate "no longer hanging" with "fast".** A 99s parse that completes is a correctness step (no more catastrophic backtracking), NOT a speed achievement.

**Why this order:**
- Optimizing a buggy parser bakes in the bugs.
- A wrong-fast parser is worse than a slow-correct one (users get wrong AST silently).
- Speed-first work on grammars/engines that aren't yet correct introduces premature couplings that block later correctness fixes.

**How to apply:**
- When reporting progress, distinguish "advanced N more lines" (correctness progress) from "got faster" (speed progress).
- When proposing engine fixes, document whether they target CORRECTNESS (e.g. fixing a parse failure) or PERFORMANCE (e.g. better caching). Correctness fixes are always-on; perf fixes wait until the campaign clears.
- Universal Packrat memoization (`.b.6.2.15`) is unusual — it was BOTH a correctness fix (eliminated catastrophic backtracking that masquerades as failure) AND a perf fix (true O(N)). Both motivations justified.
- Future tooling proposals (`--trace-rules`, AST-aware bisection, etc.) are CORRECTNESS-enabling — they help find the next bug. They're in-scope during the correctness phase.

**Cross-references:** strengthens [[feedback_no_workarounds_fix_hierarchy]] (no quick-and-dirty fixes that trade correctness for any reason); pairs with [[feedback_post_campaign_audit.md]] (defer shape-correctness audits until the campaign closes; same shape — defer perf until correctness closes).

**Concrete current state (as of 2026-05-24):**
- SV corpus: 10/14 pass; uvm_pkg ×{2017,2023} + uvm_compat_pkg ×{2017,2023} fail.
- Cumulative uvm_pkg deep parse advance this session: 5521 → 19378 (~15% of 90K).
- Speed is NOT YET a concern. Get the remaining ~85% of uvm_pkg parsed correctly first.

---

**⭐ UNIVERSALIZED — "MILK ON FIRE" PER-PARSER SPEED MONITORING (director 2026-07-18,
session #150, verbatim):** *"Parsing time will be a thing to monitor like milk on fire for
every PGEN generated parser, meaning they shall run as fast as possibly can, no compromise,
no exception. Meaning once they reach 100% accuracy with their EBNF as the sole source of
truth, then we should ensure the run lighting fast. With both accuracy and speed requirement
achieved PGEN will be taken very seriously."*

Consequences (standing, all parser families):
1. **The two-phase order is now the universal per-parser lifecycle:** (a) 100% accuracy with
   the EBNF as the sole source of truth (the immovable floor, unchanged), then (b) a SPEED
   campaign driving the generated parser as fast as it can go — no compromise, no exception.
2. **Continuous monitoring is part of the deliverable:** every generated parser family gets
   parse-time tracking as a first-class, continuously-watched metric ("like milk on fire"),
   not a one-off benchmark — the RGX-0078 apparatus (bench corpus + geomean steering +
   land-iff-faster gates + the corpus max-time closure/hold gate from the same-day bar
   redefinition) is the TEMPLATE to replicate per family.
3. **PGEN's credibility claim = accuracy AND speed together** — consistent with the
   2026-07-11 co-equal north star and the 2026-07-18 existential ≤1µs bar; this record
   extends both from "regex campaign" to "every PGEN generated parser".
4. Practical sequencing stays the locked program: the RGX-0078 closure first; the per-family
   speed campaigns follow their accuracy closures (SV reaching `Done` remains the gating
   accuracy debt for its family).
