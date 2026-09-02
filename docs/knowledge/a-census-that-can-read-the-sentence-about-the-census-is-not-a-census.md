---
id: a-census-that-can-read-the-sentence-about-the-census-is-not-a-census
title: A census that can read the sentence about the census is not a census — derive a population from the registry's DATA column, exclude the classifier from its own population, and under set -e guard every grep that may legitimately find nothing
answers:
  - "my derived set has one member too many and it is the file I named as a counter-example"
  - "how do I derive a population of scripts without hand-listing them"
  - "can a classifier scan scripts/check_*.sh for references when it is itself one of those scripts"
  - "why did my derived set come out EMPTY after a refactor"
  - "a gate under set -euo pipefail started failing on healthy input after I added a grep"
  - "two RED arms pass but the GREEN arm fails — what does that mean"
  - "how do I widen an acceptance gate's scope without a hand-maintained path list"
tags: [census, classifier, self-reference, set-e, gates, banks, doctrine, sv-corpus-grad]
date: 2026-09-03
status: current
evidence: "SV-CORPUS-GRAD.13l / PGEN-SV-CORPUS-GRAD-0342. `check_diagnosis_evidence.sh` learned to admit every stimuli script a registered oracle references. First cut: hop 1 = grep over `scripts/check_*.sh` — which includes the classifier itself, whose new comment named `audit_explained_svpp.py` and `adjudicate_dark_worklist.py` as *negative* examples; derived set 7 instead of 6, the non-producer control arm RED. Second cut: per-oracle grep inside `$( … )` under `set -euo pipefail`; an oracle with no stimuli reference returned 1, the subshell died, the set printed EMPTY, the gate ABORTED rc 1 on any staged stimuli path — two RED arms passed for the wrong reason and four pre-existing diag-evidence banks went RED (1/1/1/3 failed). Final: oracles = the registry's script column + report_*.sh minus the classifier; every grep guarded; bank 6/6; regression banks 0 failed."
reverify: "bash scripts/check_diagnosis_evidence.sh --producers | grep -c '^  stimuli/'   # 6 today, none typed; bash docs/tasks/artifacts/sv_corpus_grad/producer_admission/probe.sh  # PRODUCER-ADMISSION: 6/6"
---

**Three rules, each paid for in one slice.**

## 1. The scanner's own text is in the corpus

Derive a population by scanning files for references and the file doing the scanning is one of the
files — and it will, sooner or later, contain a sentence *about* the things it scans for. The
comment that explained what stays out of scope put those very paths into scope.

⇒ Derive from **data**, not from a glob: the registry's script column names the oracles; the
classifier is not an oracle and is **excluded by name**, with the reason beside the exclusion.

## 2. Under `set -euo pipefail`, "found nothing" is exit 1

A `grep` that legitimately matches nothing returns 1. Inside `$( … )` that is the substitution's
status; under `set -e` the assignment aborts the shell. The derived set came out **empty**, and a
gate keyed on it aborted on every healthy input.

⇒ Every search that may find nothing is guarded (`|| :`), and the deriving functions `return 0`.

## 3. A RED arm that passes must be asked WHY it failed

With the empty set, the two arms that expect a refusal still saw rc 1 — because the gate was
*crashing*, not refusing. Their green proved nothing. The GREEN sibling arm (a non-producer edit
must read *no code change*) went red, and so did four pre-existing banks pinning the same enforcer.

⇒ Every bank carries a GREEN arm beside its RED ones, and a widening to an enforcer **re-runs every
bank that already pins that enforcer** — the same instruments that found both defects here.
