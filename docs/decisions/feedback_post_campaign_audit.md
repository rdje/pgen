<!-- Decision record (layer C) — migrated 2026-06-02 from harness-home memory
     (~/.claude/projects/.../memory/feedback_post_campaign_audit.md) by MEMORY-ARCH.2 (PGEN-MEMORY-ARCH-0003).
     Now the tracked system of record; the ~/.claude copy is a cache. Content preserved verbatim below. -->

---
name: SV shape-correctness audit is post-campaign work, not mid-campaign
description: 2026-05-08 user direction — finish typing every SV rule first, then plan a holistic shape-correctness audit at the end. Don't pause the campaign to do mid-flight audits.
type: feedback
originSessionId: f74f4acc-7183-408a-ae5d-dcdce15a103c
---
The SystemVerilog typed-annotation campaign continues until every reachable rule is typed. Don't stop the typing campaign to do interim shape-correctness audits — defer those to post-campaign.

**Why:** User said "we can plan a review or audit of the shaped AST when done and then adjust some rules's return annotations accordingly. But just plan that review or audit when systemverilog's return annotations campaign is over." (in response to my proposal to audit Category B/C uses of `{first, rest}` mid-campaign).

**How to apply:**

1. Continue picking next-untyped rules each slice. Don't pause to audit.
2. If you encounter a shape that looks suboptimal but works, type it well-enough and move on. Note it as a candidate for the post-campaign holistic review (TaskList #49).
3. The exception: a Category A `{first, rest}` misuse (pure `X (sep X)*` exposing raw envelope) is an objective bug worth fixing inline as you encounter it (see `feedback_quantified_group_extraction.md`). Slice 58 already did the historical sweep — going forward, just emit `[$N, $M::2*]` from the start for pure-list patterns.
4. Category B (multi-payload-per-iteration) and Category C (`X X*` no separator) shape questions should NOT be touched mid-campaign. They're queued for the post-campaign review.

**Reference:** TaskList #49 "Post-SV-campaign holistic AST shape audit". feedback_quantified_group_extraction.md for the Category A/B/C taxonomy.
