#!/usr/bin/env bash
# SV-CORPUS-GRAD.13c.2l — the adversarial probe for the SV-CONTRACT-CURRENCY doctrine.
#
# ⛔ WHY IT EXISTS. The doctrine passes today, and a check whose inputs all pass has not been tested
# ([[a-check-whose-inputs-all-pass-has-not-been-tested]], `docs/CLAIM_VERIFICATION.md` leg 2:
# *"make the control go RED on purpose"*). Each of the four tiers is fired against the state it
# exists to refuse, and the unperturbed tree is fired too — without that control, a probe that simply
# broke the checker would score 4/4.
#
#   bash docs/tasks/artifacts/sv_corpus_grad/contract_accept_set_ledger/probe_contract_currency.sh
#
# Exit 0 = every arm behaved as specified. Exit 1 = an arm did not.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT" || exit 2
[ -x scripts/check_sv_contract_currency.sh ] || { echo "probe: not at the repo root ($ROOT)" >&2; exit 2; }

CHECK="scripts/check_sv_contract_currency.sh"
REGISTER="docs/contracts/PGEN_SV_GRAMMAR_REVISION_REGISTER.tsv"
GRAMMAR="grammars/systemverilog.ebnf"

# ⛔ The staged arm writes to the git INDEX. Refuse to run on a tree that already has staged work,
# rather than risk clobbering a commit in progress.
if [ -n "$(git diff --cached --name-only)" ]; then
  echo "probe: REFUSING — the index already carries staged files. Commit or unstage first; the" >&2
  echo "       staged arm must be free to add and remove entries without disturbing real work." >&2
  exit 2
fi

BACKUP="$(mktemp -d "${TMPDIR:-/tmp}/sv_contract_currency_probe.XXXXXX")"
LOG="$BACKUP/run.log"
PASS=0
FAIL=0

cleanup() {
  cp -f "$BACKUP/register.tsv" "$REGISTER" 2>/dev/null
  cp -f "$BACKUP/grammar.ebnf" "$GRAMMAR" 2>/dev/null
  git restore --staged "$GRAMMAR" "$REGISTER" 2>/dev/null
  echo "probe: restored $REGISTER, $GRAMMAR and the index"
}
trap cleanup EXIT
cp -f "$REGISTER" "$BACKUP/register.tsv"
cp -f "$GRAMMAR" "$BACKUP/grammar.ebnf"

arm() {
  local name="$1" want_rc="$2" want_msg="$3" rc
  bash "$CHECK" > "$LOG" 2>&1
  rc=$?
  if [ "$rc" = "$want_rc" ] && grep -qF "$want_msg" "$LOG"; then
    echo "  ✓ $name — rc=$rc and the message names it"
    PASS=$((PASS + 1))
  else
    echo "  ✗ $name — expected rc=$want_rc containing:"
    echo "      $want_msg"
    echo "    got rc=$rc:"
    sed 's/^/      /' "$LOG" | tail -12
    FAIL=$((FAIL + 1))
  fi
}
restore() { cp -f "$BACKUP/register.tsv" "$REGISTER"; cp -f "$BACKUP/grammar.ebnf" "$GRAMMAR"; }

# ── tier A — a grammar commit with no register row ───────────────────────────────────────────────
echo "tier A: a grammar revision nobody registered"
grep -v '^e28cc856' "$BACKUP/register.tsv" > "$REGISTER"
arm "an unregistered grammar commit is REPORTED" 1 "with NO row in"
restore

# ── tier C — a NEUTRAL claim that the register's own digests refute ──────────────────────────────
echo "tier C: a neutrality claim that is false"
awk -F'\t' 'BEGIN{OFS="\t"} $5=="NEUTRAL" && !done {$4="0000000000000000000000000000000000000000000000000000000000000000"; done=1} {print}' \
  "$BACKUP/register.tsv" > "$REGISTER"
arm "a refuted NEUTRAL claim is REPORTED" 1 "but its semantic digest"
restore

# ── tier C — an unreadable disposition must be refused, never skipped ────────────────────────────
echo "tier C: an unknown disposition"
awk -F'\t' 'BEGIN{OFS="\t"} $5=="RELEASE" && !done {$5="RELESE"; done=1} {print}' \
  "$BACKUP/register.tsv" > "$REGISTER"
arm "an unknown disposition is REFUSED" 1 "only RELEASE and NEUTRAL are defined"
restore

# ── tier D — the working tree's grammar moved and no row describes it ────────────────────────────
# A SEMANTIC edit, not a comment: the whole point of the digest is that a comment cannot move it,
# so a comment here would (correctly) leave the doctrine green and prove nothing.
echo "tier D: an unregistered semantic edit in the working tree"
printf '\nprobe_only_unreferenced_rule_sv_contract_currency := trivia /zzz_probe_only\\b/\n' >> "$GRAMMAR"
arm "an unregistered working-tree grammar edit is REPORTED" 1 "and no register row describes it yet"
restore

# ⛔ THE COMPLEMENT OF TIER D, and it is what makes the digest worth having: the SAME file edited
# with a COMMENT must stay green. Without this arm, "comment-insensitive" is a claim, not a result.
echo "tier D complement: a comment-only edit must NOT trip the doctrine"
printf '\n# probe-only comment, restored by this script — SV-CONTRACT-CURRENCY must not see it\n' >> "$GRAMMAR"
arm "a comment-only grammar edit stays GREEN" 0 "SV-CONTRACT-CURRENCY: rows="
restore

# ── tier B — a staged grammar edit with no staged register ───────────────────────────────────────
echo "tier B: a staged grammar edit that does not stage the register"
printf '\n# probe-only staged comment, restored by this script\n' >> "$GRAMMAR"
git add "$GRAMMAR"
arm "a staged grammar edit without the register is REPORTED" 1 "is staged and"
git restore --staged "$GRAMMAR"
restore

# ── the CONTROL — the unperturbed tree must be GREEN ─────────────────────────────────────────────
echo "control: the unperturbed tree"
arm "the unperturbed tree HOLDS" 0 "SV-CONTRACT-CURRENCY: rows="

echo "SV-CONTRACT-CURRENCY-PROBE: arms_passed=$PASS arms_failed=$FAIL"
[ "$FAIL" = 0 ] || exit 1
