#!/usr/bin/env bash
# scripts/check_diagnosis_vocabulary_currency.sh — the acceptance gate's diagnosis VOCABULARY is held
# current against the toolbox that documents the instruments, in BOTH directions, on every run.
#
# ENGINE-UNIVERSAL-SERVICES.81 (2026-09-29). `scripts/check_diagnosis_evidence.sh` scores a ROOT CAUSE box
# by a vocabulary of instrument spellings (`DIAGNOSIS_SIG`). A vocabulary cannot see what it did not list
# (`.77`(b)), and this one had not: measured on 2026-09-29, TOOLBOX.md's §1–§6 section headings document
# 30 instrument spellings, of which 14 were unmatched by the regex — three of them knobs (`--parse`,
# `--supports`, `--trace-log-file`), eleven real instruments including `PGEN_REPORT_STACK_PROBE`, the one
# TOOLBOX.md names for stack defects, which `.79` led with and could not score. The lockstep obligation
# the gate's own header states ("a token joins this list when a real instrument emits it, and TOOLBOX.md's
# five-family table lists it in the same commit") was a promise; this makes it a property:
#
#   ARM 1 — every instrument spelling a §1–§6 `###` heading of TOOLBOX.md documents (a code span shaped
#           `PGEN_*`, `--flag` or `HEADLINE:`), minus the DECLARED non-instruments below, is matched by
#           `DIAGNOSIS_SIG` — a documented instrument the gate cannot score is a breach.
#   ARM 2 — every `PGEN_*` and `HEADLINE:` token `DIAGNOSIS_SIG` registers appears in TOOLBOX.md's
#           five-family table — a registered token the table does not document is a breach.
#
# The regex is read from the gate's UNEXPANDED SOURCE (the `DIAGNOSIS_SIG='…'` line), never re-declared
# here, so the two cannot fork. ARCHETYPE: structural (DOCTRINE_ENFORCEMENT.md §3). CONTRACT (§4): exit
# code is the verdict; explains on stderr; deterministic; read-only; path-agnostic; fast (~50 ms).
#
#   bash scripts/check_diagnosis_vocabulary_currency.sh              # the check (called by the gate)
#   bash scripts/check_diagnosis_vocabulary_currency.sh --self-test  # GREEN on the tree + two RED controls
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT" || exit 2
TOOLBOX="${PGEN_DIAG_VOCAB_TOOLBOX:-TOOLBOX.md}"
GATE="${PGEN_DIAG_VOCAB_GATE:-scripts/check_diagnosis_evidence.sh}"
TAG="DIAG-VOCABULARY"

# Heading spellings that are NOT diagnosis instruments, each with its reason. A knob is not an instrument:
# quoting it proves nothing was run that explains a defect.
#   --parse          the probe's ordinary parse verb — every parse is one; the DIAGNOSIS is what it prints
#                    (`furthest_position=`, the trace), which the gate already scores
#   --supports       a capability query ("is this grammar registered"), not a diagnosis
#   --trace-log-file the SINK of the trace instrument; the instrument is `--trace-rules` /
#                    `PGEN_TRACE_VERBOSITY=`, both scored
NON_INSTRUMENTS='^(--parse|--supports|--trace-log-file)$'

fail=0
note() { printf '%s: ✗ %s\n' "$TAG" "$1" >&2; fail=1; }

[ -f "$TOOLBOX" ] || { note "$TOOLBOX is missing — the arm cannot inspect its subject and must say so"; exit 1; }
[ -f "$GATE" ]    || { note "$GATE is missing — the arm cannot inspect its subject and must say so"; exit 1; }

# The regex, read from the gate's unexpanded source. A gate without the line is a breach, never a pass.
sig="$(grep -m1 "^DIAGNOSIS_SIG='" "$GATE" | sed "s/^DIAGNOSIS_SIG='//; s/'\$//")"
[ -n "$sig" ] || { note "$GATE carries no DIAGNOSIS_SIG='…' line — nothing to hold current"; exit 1; }

# ARM 1 — documented instrument spellings (§1–§6 `###` headings, first word of each shaped code span).
documented="$(awk '/^## 1\. /{p=1} /^## Protocol A/{p=0} p' "$TOOLBOX" \
  | grep -E '^### ' | grep -o '`[^`]*`' | tr -d '`' | awk '{print $1}' \
  | grep -E '^(PGEN_[A-Z_]+|--[a-z][a-z-]+|[A-Z][A-Z0-9-]+:)$' | sort -u || true)"
n_doc=0; n_knob=0; unmatched=()
while IFS= read -r t; do
  [ -n "$t" ] || continue
  if printf '%s' "$t" | grep -qE "$NON_INSTRUMENTS"; then n_knob=$((n_knob + 1)); continue; fi
  n_doc=$((n_doc + 1))
  printf '%s' "$t" | grep -qE -- "$sig" || unmatched+=("$t")
done <<< "$documented"
for t in "${unmatched[@]:-}"; do
  [ -n "$t" ] && note "TOOLBOX.md documents the instrument \`$t\` in a §1–§6 heading and the gate cannot score it — register it in DIAGNOSIS_SIG (and the five-family table) or declare it a knob here, with its reason"
done

# ARM 2 — registered PGEN_* and HEADLINE: tokens must be documented in the five-family table.
table="$(grep -E '^\| [1-5] \| ' "$TOOLBOX" || true)"
[ -n "$table" ] || note "TOOLBOX.md carries no five-family table rows (\`| 1 |\` … \`| 5 |\`) — arm 2 cannot inspect its subject"
registered_pgen="$(printf '%s' "$sig" | grep -oE 'PGEN_[A-Z_]+(\([A-Z_|]+\))?' | sort -u || true)"
n_reg=0; n_head=0
while IFS= read -r t; do
  [ -n "$t" ] || continue
  # expand one parenthesised alternation: PGEN_A_(B|C) -> PGEN_A_B PGEN_A_C
  if [[ "$t" == *"("* ]]; then
    stem="${t%%(*}"; alts="${t#*(}"; alts="${alts%)}"
    IFS='|' read -r -a parts <<< "$alts"
    for a in "${parts[@]}"; do n_reg=$((n_reg + 1)); printf '%s' "$table" | grep -qF -- "\`${stem}${a}" || note "DIAGNOSIS_SIG registers \`${stem}${a}\` and TOOLBOX.md's five-family table does not list it"; done
  else
    n_reg=$((n_reg + 1)); printf '%s' "$table" | grep -qF -- "\`$t" || note "DIAGNOSIS_SIG registers \`$t\` and TOOLBOX.md's five-family table does not list it"
  fi
done <<< "$registered_pgen"
registered_head="$(printf '%s' "$sig" | grep -oE '(^|\|)[A-Z][A-Z0-9-]+:' | sed 's/^|//' | sort -u || true)"
while IFS= read -r t; do
  [ -n "$t" ] || continue
  n_head=$((n_head + 1)); printf '%s' "$table" | grep -qF -- "\`$t" || note "DIAGNOSIS_SIG registers the headline \`$t\` and TOOLBOX.md's five-family table does not list it"
done <<< "$registered_head"

if [ "${1:-}" = "--self-test" ]; then
  [ "$fail" -eq 0 ] || { echo "$TAG self-test: the GREEN control failed on the real tree" >&2; exit 1; }
  scratch="rust/target/diag_vocabulary_self_test"; mkdir -p "$scratch"
  # RED 1 — a planted documented instrument the gate does not score.
  { awk '/^## 1\. /{print; print "### 1.0z `PGEN_PLANTED_SELF_TEST_DUMP`"; next} {print}' "$TOOLBOX"; } > "$scratch/toolbox_planted.md"
  if PGEN_DIAG_VOCAB_TOOLBOX="$scratch/toolbox_planted.md" bash "$0" >/dev/null 2>"$scratch/red1.err"; then
    echo "$TAG self-test: ✗ RED 1 did not fire (a planted documented instrument passed)" >&2; exit 1; fi
  grep -q 'PGEN_PLANTED_SELF_TEST_DUMP' "$scratch/red1.err" || { echo "$TAG self-test: ✗ RED 1 fired without naming the planted instrument" >&2; exit 1; }
  # RED 2 — a planted registered token the table does not document.
  sed "s/^DIAGNOSIS_SIG='/DIAGNOSIS_SIG='PLANTED-SELF-TEST-HEADLINE:|/" "$GATE" > "$scratch/gate_planted.sh"
  if PGEN_DIAG_VOCAB_GATE="$scratch/gate_planted.sh" bash "$0" >/dev/null 2>"$scratch/red2.err"; then
    echo "$TAG self-test: ✗ RED 2 did not fire (a planted registered headline passed)" >&2; exit 1; fi
  grep -q 'PLANTED-SELF-TEST-HEADLINE:' "$scratch/red2.err" || { echo "$TAG self-test: ✗ RED 2 fired without naming the planted token" >&2; exit 1; }
  echo "$TAG self-test: 3/3 as declared (GREEN on the tree; RED 1 planted instrument refused; RED 2 planted token refused)"
  exit 0
fi

if [ "$fail" -ne 0 ]; then
  printf '%s: ✗ FAIL — the gate'"'"'s diagnosis vocabulary and TOOLBOX.md disagree; register or declare, in the same commit\n' "$TAG" >&2
  exit 1
fi
echo "$TAG: OK (documented instruments=$n_doc knobs_declared=$n_knob registered_pgen=$n_reg registered_headlines=$n_head — the gate scores every instrument the toolbox documents, and the table lists every token the gate registers)"
exit 0
