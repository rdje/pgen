---
id: a-red-arm-that-returns-rc-1-from-a-crash-is-not-a-red-arm
title: A RED arm that returns the expected non-zero exit from a CRASH is not a RED arm — assert the refusal's TEXT, never the exit code alone; and pair every RED arm with a GREEN sibling on the same wiring
answers:
  - "my negative control exits 1 as expected — is that enough to trust the gate"
  - "the RED arm passes but the GREEN arm fails — what does that mean"
  - "how should a bank assert that a gate refused for the right reason"
  - "why did a gate start aborting on healthy input after I added a check"
  - "what did the NameError / empty-set / composite-row failures have in common"
tags: [banks, controls, red-green, gates, evidence, sv-corpus-grad]
date: 2026-09-03
status: current
evidence: "Three instances in one day. PGEN-SV-CORPUS-GRAD-0342: the producer-admission gate's derivation died under `set -e` on a grep that found nothing, the derived set printed EMPTY, and the two RED arms still saw rc 1 — because the gate was ABORTING, not refusing; the GREEN sibling arm and four pre-existing banks went red. PGEN-SV-CORPUS-GRAD-0343: the new identity gate read `cost.md`'s composite `sample inputs` row as MOVED (rc 1, the expected number) while the owning doctrine read it fresh. PGEN-SV-CORPUS-GRAD-0344: the permanent-deferral ceiling's first RED run returned rc 1 from `NameError: name 'sys' is not defined` — the refusal path had never imported sys."
reverify: "bash docs/tasks/artifacts/sv_corpus_grad/permanent_deferral_ceilings/probe.sh   # every RED arm asserts a token from the refusal message ('above its recorded ceiling', 'records no ceiling', 'unreadable'), not rc alone"
---

**An exit code is one bit. A refusal and a crash both set it.**

## What happened, three times

| slice | the RED arm saw | what actually happened |
|---|---|---|
| `-0342` | rc 1 on a staged producer edit ✓ | the gate **aborted** under `set -e` on an empty `grep` — the derived set was EMPTY |
| `-0343` | rc 1 on `cost.md` ✓ | the gate re-hashed a **composite** row by its path cell — the owning doctrine read it fresh |
| `-0344` | rc 1 on a ceiling of 89 ✓ | `NameError: name 'sys' is not defined` — the refusal path **crashed** before refusing |

Each time the number was right and the reason was wrong, and each time the same two things told them
apart: **reading the message**, and a **GREEN sibling arm** on the same wiring (a non-producer edit must
read *no code change*; the tracked register must run rc 0) — the GREEN went red while the RED stayed
"green".

## The rules

1. **Every RED arm asserts a token from the refusal's own message** — `grep -qF -- "$token"` beside the
   `rc` test. A refusal names what it refused; a crash names a line number.
2. **Every bank carries at least one GREEN arm on the same wiring.** A crash fails both directions; a
   refusal fails one. Without the GREEN arm the two are indistinguishable.
3. **When you widen an enforcer, re-run every bank that already pins it.** The four pre-existing
   diag-evidence banks were the instrument that separated `-0342`'s abort from a refusal.
4. **Ask WHY a RED arm failed before believing it.** *"A control that cannot fail proves nothing"* has a
   sibling: *a control that fails for the wrong reason proves less than nothing*, because it converts a
   defect into evidence.
