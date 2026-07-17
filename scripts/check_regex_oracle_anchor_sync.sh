#!/usr/bin/env bash
# scripts/check_regex_oracle_anchor_sync.sh
#
# REGEX-PCRE2-FIDELITY.DOCSYNC.2 (PGEN-REGEX-PCRE2-0050) — the derived drift check the
# DOCSYNC.1 sweep named (feedback_duplicated_metadata_needs_derived_drift_gate): the
# CURRENT PCRE2-oracle real tuple `cells/matches/false-accepts/false-rejects` is
# maintained on a small set of designated LIVE anchor surfaces (the integration
# contract's maintenance anchor is the reference). This check re-derives their mutual
# consistency + their sanity against the tracked oracle ratchet bounds, so a PARTIAL
# anchor update (the likely future drift: one surface bumped at a fidelity slice, the
# others forgotten) fails the pre-commit/CI gate instead of rotting silently — the
# DOCSYNC.1 census found 174 stale-form occurrences had accumulated exactly this way.
#
# WHY the tuple is not asserted against a measurement here: the oracle gate
# (rust/scripts/regex_pcre2_compile_oracle_gate.sh) deliberately asserts only BOUNDS on
# the match/mismatch split (fidelity improvements must not force a re-baseline every
# slice), and its per-run real-tuple summary is untracked. The strongest structural
# claim available is therefore (a) all live anchors agree byte-for-byte, and (b) the
# agreed tuple is consistent with the tracked ratchet env. Era-dated historical records
# (CHANGES/DEV_NOTES/task trees/tracker notes/per-release callouts) are DELIBERATELY
# out of scope — they cite the tuple at their own era (supersede-don't-mutate).
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"; cd "$ROOT"
fail=0
TUPLE_RE='2189/[0-9]+/[0-9]+/[0-9]+'

CONTRACT=docs/contracts/PGEN_REGEX_PARSER_INTEGRATION_CONTRACT.md
CHAPTER=docs/regex_parser_book/src/compile-contract-validator.md
HOWTO=docs/book/src/diagnosing-unknowns.md
MEMORY=MEMORY.md
ENV_FILE=rust/test_data/grammar_quality/regex_pcre2_compile_oracle_lightweight_v0.env

for f in "$CONTRACT" "$CHAPTER" "$HOWTO" "$MEMORY" "$ENV_FILE"; do
  [ -f "$f" ] || { echo "oracle-anchor-sync: FAIL — required file missing: $f" >&2; exit 1; }
done

# (1) The reference: the contract's maintained current-snapshot anchor line.
ref="$(grep -E "The current real tuple .*\*\*\`${TUPLE_RE}\`\*\*" "$CONTRACT" \
       | head -1 | grep -oE "$TUPLE_RE" | head -1 || true)"
if [ -z "$ref" ]; then
  echo "oracle-anchor-sync: FAIL — the contract's current-snapshot anchor line ('The current real tuple … **\`cells/m/fa/fr\`**') is missing from $CONTRACT" >&2
  exit 1
fi

# (2) The validator chapter's snapshot: its BOLD tuple(s) must equal the reference.
# (Bold is the discriminator — the evolution chain + per-release callouts deliberately
# keep era tuples unbolded.)
chap_bold="$(grep -oE "\*\*\`${TUPLE_RE}\`\*\*" "$CHAPTER" | grep -oE "$TUPLE_RE" | sort -u || true)"
if [ "$chap_bold" != "$ref" ]; then
  echo "oracle-anchor-sync: FAIL — $CHAPTER bold current tuple(s) [$chap_bold] != contract reference [$ref]; update every live anchor together" >&2
  fail=1
fi

# (3) The diagnosing-unknowns how-to: exactly ONE tuple occurrence, equal to the reference.
howto_all="$(grep -oE "$TUPLE_RE" "$HOWTO" | sort -u || true)"
howto_n="$(grep -coE "$TUPLE_RE" "$HOWTO" || true)"
if [ "$howto_n" -ne 1 ] || [ "$howto_all" != "$ref" ]; then
  echo "oracle-anchor-sync: FAIL — $HOWTO must cite the current tuple exactly once; found count=$howto_n value(s)=[$howto_all] vs reference [$ref]" >&2
  fail=1
fi

# (4) MEMORY.md (layer A): every tuple occurrence (if any) must equal the reference.
mem_bad="$(grep -oE "$TUPLE_RE" "$MEMORY" | sort -u | grep -v -x "$ref" || true)"
if [ -n "$mem_bad" ]; then
  echo "oracle-anchor-sync: FAIL — $MEMORY carries non-current tuple(s): [$mem_bad] vs reference [$ref]" >&2
  fail=1
fi

# (5) Sanity of the reference against the tracked ratchet env (the bounds the oracle
# gate itself enforces). Fields: cells / matches / false-accepts / false-rejects.
IFS='/' read -r cells matches fa fr <<< "$ref"
env_get() { grep -E "^$1=" "$ENV_FILE" | head -1 | cut -d= -f2; }
exp_cases="$(env_get PGEN_REGEX_PCRE2_COMPILE_ORACLE_EXPECTED_CASES)"
min_match="$(env_get PGEN_REGEX_PCRE2_COMPILE_ORACLE_MIN_MATCH_TOTAL)"
max_fa="$(env_get PGEN_REGEX_PCRE2_COMPILE_ORACLE_MAX_FALSE_ACCEPT_TOTAL)"
max_fr="$(env_get PGEN_REGEX_PCRE2_COMPILE_ORACLE_MAX_FALSE_REJECT_TOTAL)"
if [ "$cells" != "$exp_cases" ]; then
  echo "oracle-anchor-sync: FAIL — anchor cells $cells != tracked EXPECTED_CASES $exp_cases ($ENV_FILE)" >&2
  fail=1
fi
if [ "$matches" -lt "$min_match" ] || [ "$fa" -gt "$max_fa" ] || [ "$fr" -gt "$max_fr" ]; then
  echo "oracle-anchor-sync: FAIL — anchor tuple [$ref] violates the tracked ratchet bounds (MIN_MATCH=$min_match MAX_FA=$max_fa MAX_FR=$max_fr in $ENV_FILE)" >&2
  fail=1
fi
if [ "$((matches + fa + fr))" -ne "$cells" ]; then
  echo "oracle-anchor-sync: FAIL — anchor tuple [$ref] fields do not sum: matches+fa+fr=$((matches + fa + fr)) != cells $cells" >&2
  fail=1
fi

if [ "$fail" -ne 0 ]; then
  exit 1
fi
echo "oracle-anchor-sync: OK (tuple $ref consistent across live anchors + tracked bounds)"
