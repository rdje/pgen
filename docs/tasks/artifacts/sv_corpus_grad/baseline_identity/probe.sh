#!/usr/bin/env bash
# SV-CORPUS-GRAD.13c.2x.2 — the adversarial probe for the `BASELINE-IDENTITY` doctrine.
#
# ⛔ WHY IT EXISTS, AND WHY ARM 3 IS THE ONE THAT MATTERS. `SV-CORPUS-GRAD.13i` measured this
# repository shipping SIX oracles that already carried a self-describing identity block: exactly
# ONE was gate-checked and FOUR were measurably stale. A block that is present and unread passes
# every check that only asks whether it EXISTS — so a doctrine about identity blocks that has not
# been observed REFUSING is indistinguishable from the defect it claims to fix
# ([[a-check-whose-inputs-all-pass-has-not-been-tested]]).
#
# Every arm fires one way the doctrine must refuse, and the two CONTROL arms must stay GREEN so a
# blanket-red enforcer cannot score a pass. ⭐ Arm 3 changes the INPUT FILE rather than the
# recorded digest, because that is the property under test: the verifier re-hashes the live tree on
# every run, it does not compare a number against itself.
#
#   bash docs/tasks/artifacts/sv_corpus_grad/baseline_identity/probe.sh
#
# Exit 0 = every arm behaved as specified. Exit 1 = an arm did NOT (a hole). Exit 2 = the probe
# could not run.
#
# ⛔ FIVE levels up: baseline_identity -> sv_corpus_grad -> artifacts -> tasks -> docs -> root.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT" || exit 2
[ -f scripts/check_baseline_identity.sh ] || { echo "probe: not at the repo root ($ROOT)" >&2; exit 2; }

VERIFY="scripts/check_baseline_identity.sh"
QUAL="rust/test_data/grammar_quality"
REGISTER="$QUAL/baseline_identity_register_v0.json"
ADOPTED="$QUAL/systemverilog_recognized_cert_union_contract.json"
CLOSURE="$QUAL/systemverilog_syntax_closure_contract.json"
DEFERRED="$QUAL/regex_broader_corpus_v0.json"

# ⛔ ON-VOLUME BY POLICY (CLAUDE.md §13): the scratch tree is derived from the repo root, never
# from $TMPDIR, which can live on a different filesystem.
WORK="rust/target/baseline_identity_probe"
rm -rf "$WORK"; mkdir -p "$WORK" || exit 2
LOG="$WORK/arm.log"
PASS=0; FAIL=0

cleanup() {
  cp -f "$WORK/backup_register.json" "$REGISTER" 2>/dev/null
  cp -f "$WORK/backup_deferred.json" "$DEFERRED" 2>/dev/null
  rm -f "$QUAL/zz_probe_unregistered_v0.json"
  echo "probe: restored the tracked register + deferred baseline"
  # ⛔ SCOPED TO WHAT THIS PROBE ACTUALLY TOUCHES, not to the whole directory. A residue check
  # that also reports the slice's own legitimate in-flight edits fires on every run, and a warning
  # that always fires is a warning nobody reads.
  local residue=""
  cmp -s "$WORK/backup_register.json" "$REGISTER" || residue="$residue $REGISTER"
  cmp -s "$WORK/backup_deferred.json" "$DEFERRED" || residue="$residue $DEFERRED"
  [ -e "$QUAL/zz_probe_unregistered_v0.json" ] && residue="$residue (the unregistered probe file)"
  if [ -n "$residue" ]; then
    echo "probe: ⛔ RESIDUE LEFT BEHIND —$residue" >&2
  else
    echo "probe: no residue — every file this probe mutated is byte-identical to its backup"
  fi
}
trap cleanup EXIT
cp -f "$REGISTER" "$WORK/backup_register.json" || exit 2
cp -f "$DEFERRED" "$WORK/backup_deferred.json" || exit 2

#   $1 arm name   $2 expected rc   $3 a string the output must contain   $4.. the command
#
# ⛔ ARM NAMES CARRY NO BACKTICKS, DELIBERATELY. They are double-quoted shell words, so a backtick
# opens a COMMAND SUBSTITUTION: the shell ran `expectations` and `adopted` as commands and the
# printed arm name silently lost the word. Measured here while writing this probe, and it is the
# same defect scripts/check_doctrines.sh shipped in its own registry descriptions (`-0241`) —
# `bash -n` is clean throughout, so only running it shows the loss.
arm() {
  local name="$1" want_rc="$2" want_msg="$3"; shift 3
  local rc
  "$@" > "$LOG" 2>&1; rc=$?
  # ⛔ An EMPTY expectation means "rc only". Passing "" to `grep -qF` matches every LINE, which
  # returns 1 on the empty output a silent success produces — the control would have scored a
  # failure for succeeding quietly.
  if [ "$rc" = "$want_rc" ] && { [ -z "$want_msg" ] || grep -qF "$want_msg" "$LOG"; }; then
    echo "  ✓ $name — rc=$rc and the message names it"
    PASS=$((PASS + 1))
  else
    echo "  ✗ $name — expected rc=$want_rc containing:"
    echo "      $want_msg"
    echo "    got rc=$rc:"
    sed 's/^/      /' "$LOG" | tail -14
    FAIL=$((FAIL + 1))
  fi
}

# ── CONTROLS ────────────────────────────────────────────────────────────────────────────────────
# ⛔ A control is the arm that must stay GREEN; it is NOT a refusal and is never counted as one
# (the `-0231` correction: publishing "5 refusal arms … incl. a control" overstated the evidence).
echo "controls — these must stay GREEN"
# ⛔ THE ADOPTED CONTRACT IS `adopted-unconfirmed`: its numbers are known NOT to describe HEAD, so
# `--verify` — the call a GATE makes — must go RED. This was a GREEN control in slice 1 and the
# GREEN was the defect (`-0249`): the block asserted a derivation that had not happened.
arm "arm 0: the union contract's expectations are UNCONFIRMED -> RED for any consumer" 1 \
    "ARE UNCONFIRMED — the artifact says so itself" bash "$VERIFY" --verify "$ADOPTED"
arm "CONTROL: the closed register is consistent at HEAD" 0 "" \
    bash "$VERIFY"

# ── a scratch baseline over a scratch input, so arm 3 can move a real file safely ───────────────
SCRATCH_IN="$WORK/declared_input.txt"
SCRATCH_BL="$WORK/scratch_baseline.json"
printf 'the original bytes\n' > "$SCRATCH_IN"
printf '{\n  "contract": "baseline_identity_probe",\n  "expected_total": 7\n}\n' > "$SCRATCH_BL"
bash "$VERIFY" --stamp "$SCRATCH_BL" --input "$SCRATCH_IN" \
     --confirmed-by "the probe derived expected_total=7 from declared_input.txt in this run" \
     > "$LOG" 2>&1 || {
  echo "probe: could not stamp the scratch baseline" >&2; sed 's/^/  /' "$LOG" >&2; exit 2; }

echo "refusal arms — each must go RED"
arm "CONTROL: a CONFIRMED scratch baseline verifies before it is disturbed" 0 \
    "identity fresh and expectations CONFIRMED" bash "$VERIFY" --verify "$SCRATCH_BL"

# ── arm 3: THE DECLARED INPUT MOVES ─────────────────────────────────────────────────────────────
printf 'the original bytes, plus one\n' > "$SCRATCH_IN"
arm "arm 3: a declared input CHANGED -> STALE" 1 "THE BASELINE IS STALE" \
    bash "$VERIFY" --verify "$SCRATCH_BL"
printf 'the original bytes\n' > "$SCRATCH_IN"

# ── arm 4: THE BLOCK IS REMOVED ─────────────────────────────────────────────────────────────────
python3 - "$SCRATCH_BL" "$WORK/no_block.json" <<'PY'
import json, sys
d = json.load(open(sys.argv[1])); d.pop("identity")
json.dump(d, open(sys.argv[2], "w"), indent=2)
PY
arm "arm 4: the block is REMOVED -> REFUSE" 2 "carries NO identity block" \
    bash "$VERIFY" --verify "$WORK/no_block.json"

# ── arms 5-7: the block is PRESENT but does not mean anything ───────────────────────────────────
python3 - "$SCRATCH_BL" "$WORK" <<'PY'
import json, os, sys
src, out = sys.argv[1], sys.argv[2]
base = json.load(open(src))
def emit(name, mutate):
    d = json.loads(json.dumps(base)); mutate(d["identity"])
    json.dump(d, open(os.path.join(out, name), "w"), indent=2)
def truncate(i):
    k = next(iter(i["inputs"])); i["inputs"][k] = i["inputs"][k][:16]
emit("bad_sha.json", truncate)
emit("no_pointer.json", lambda i: i.pop("_verifier"))
emit("no_state.json", lambda i: i.pop("expectations"))
emit("bad_state.json", lambda i: i.__setitem__("expectations", "probably"))
emit("no_confirmer.json", lambda i: i.pop("confirmed_by"))
emit("alien_commit.json", lambda i: i.__setitem__("verified_at_commit", "0" * 39 + "1"))
emit("absent_input.json",
     lambda i: i.__setitem__("inputs", {"rust/target/baseline_identity_probe/gone.txt":
                                        list(i["inputs"].values())[0]}))
PY
arm "arm 5: a TRUNCATED digest is not a sha256 -> REFUSE" 2 "MALFORMED identity block" \
    bash "$VERIFY" --verify "$WORK/bad_sha.json"
arm "arm 6: the block loses its pointer at the reader -> REFUSE" 2 "MALFORMED identity block" \
    bash "$VERIFY" --verify "$WORK/no_pointer.json"
# ⭐⭐ THE ARMS THAT EXIST BECAUSE SLICE 1 SHIPPED WITHOUT THIS FIELD (`-0249`). A block with no
# `expectations` state asserted that its numbers were derived from the recorded tree; for a
# baseline measured RED by 71 rules that was a confident WRONG answer, and the consuming gate
# printed `identity fresh` before reporting an unattributable drift. There is no honest default.
arm "arm 6b: no expectations state at all -> REFUSE" 2 "MALFORMED identity block" \
    bash "$VERIFY" --verify "$WORK/no_state.json"
arm "arm 6c: an expectations value outside the vocabulary -> REFUSE" 2 "MALFORMED identity block" \
    bash "$VERIFY" --verify "$WORK/bad_state.json"
arm "arm 6d: confirmed with nothing naming the run that confirmed it -> REFUSE" 2 \
    "MALFORMED identity block" bash "$VERIFY" --verify "$WORK/no_confirmer.json"
arm "arm 7: verified_at_commit names a commit not in this history -> RED" 1 \
    "is not a commit this repository contains" \
    bash "$VERIFY" --verify "$WORK/alien_commit.json"

# ── arm 8: NOTHING COULD BE HASHED — a check that cannot see must SAY SO ────────────────────────
arm "arm 8: every declared input absent -> REFUSE, never a vacuous pass" 2 \
    "NONE of them could be hashed here" \
    bash "$VERIFY" --verify "$WORK/absent_input.json"

# ── arms 8b-8d: A STAMP IS AN ASSERTION, so it cannot be made by accident ──────────────────────
arm "arm 8b: --stamp on an unadopted baseline naming NO state -> REFUSE" 2 \
    "there is no honest default" \
    bash "$VERIFY" --stamp "$WORK/no_block.json" --input "$SCRATCH_IN"
arm "arm 8c: --stamp asserting BOTH states -> REFUSE" 2 "contradictory" \
    bash "$VERIFY" --stamp "$WORK/no_block.json" --input "$SCRATCH_IN" \
        --confirmed-by "a run that re-derived the numbers, described at length" \
        --unconfirmed "and simultaneously a reason they are not confirmed at all"
arm "arm 8d: --unconfirmed with no --owner-leaf -> REFUSE" 2 "unowned debt is buried debt" \
    bash "$VERIFY" --stamp "$WORK/no_block.json" --input "$SCRATCH_IN" \
        --unconfirmed "a reason long enough to satisfy the schema check on its own"

# ── arms 9-12: the CLOSED POPULATION, both sides ────────────────────────────────────────────────
printf '{ "contract": "probe" }\n' > "$QUAL/zz_probe_unregistered_v0.json"
arm "arm 9: a NEW baseline with no verdict -> RED" 1 "with NO register row" bash "$VERIFY"
rm -f "$QUAL/zz_probe_unregistered_v0.json"

python3 - "$REGISTER" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
d["entries"].append({"entry": "zz_no_such_file_v0.json", "disposition": "not-a-derived-baseline",
                     "reason": "a probe row that describes nothing at all, to fire the ratchet"})
json.dump(d, open(sys.argv[1], "w"), indent=2, ensure_ascii=False)
PY
arm "arm 10: a row naming NOTHING is a dead exemption -> RED" 1 "names nothing in" bash "$VERIFY"
cp -f "$WORK/backup_register.json" "$REGISTER"

# ⛔⛔ THE ANTI-`.13i` ARM. A `deferred` row that CARRIES a block is a block nothing re-hashes —
# `.13i` verbatim — and must be a hard failure rather than a nudge.
bash "$VERIFY" --stamp "$DEFERRED" --input grammars/regex.ebnf \
     --confirmed-by "a synthetic confirmation written by this probe arm, then reverted" \
     > "$LOG" 2>&1 || {
  echo "probe: could not stamp the deferred baseline for arm 11 — the arm below cannot be" >&2
  echo "       trusted, so it is scored as a FAILURE rather than allowed to pass vacuously" >&2
  sed 's/^/  /' "$LOG" >&2; FAIL=$((FAIL + 1)); }
arm "arm 11: a DEFERRED row carrying a block (the .13i shape) -> RED" 1 \
    "yet CARRIES an identity block" bash "$VERIFY"
cp -f "$WORK/backup_deferred.json" "$DEFERRED"

python3 - "$REGISTER" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
for r in d["entries"]:
    if r["entry"] == "regex_broader_corpus_v0.json":
        r["owner_leaf"] = "NO-SUCH-TREE.1.2.3"
json.dump(d, open(sys.argv[1], "w"), indent=2, ensure_ascii=False)
PY
arm "arm 12: a deferral owned by a leaf nothing mentions -> RED" 1 \
    "which no docs/tasks/ tree mentions" bash "$VERIFY"
cp -f "$WORK/backup_register.json" "$REGISTER"

# ⛔⛔ THE DEBT CLASS MUST NOT BE A HIDING PLACE. Calling an unconfirmed baseline plain `adopted`
# would restore slice 1's false claim, so the register and the artifact are held to agree.
python3 "$(dirname "${BASH_SOURCE[0]}")/promote_row.py" "$REGISTER"
arm "arm 12b: an UNCONFIRMED baseline registered plain adopted -> RED" 1 \
    "ARE UNCONFIRMED" bash "$VERIFY"
cp -f "$WORK/backup_register.json" "$REGISTER"

# ── arm 13: THE READER. Does the CONSUMING GATE actually refuse? ────────────────────────────────
#
# ⛔ THIS IS THE ARM `.13i` MAKES MANDATORY. Arms 3-8 prove the verifier discriminates; only this
# one proves a gate READS it. It also asserts the refusal is CHEAP: the identity stage sits ahead
# of the ~2-minutes-per-seed measurement precisely so an undiagnosable verdict is never paid for.
python3 - "$ADOPTED" "$WORK/stale_contract.json" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
i = d["identity"]
# CONFIRMED on purpose, so the refusal this arm observes is about the INPUT HAVING MOVED and not
# about the real contract's (separately proven, arm 13b) unconfirmed state.
i["expectations"] = "confirmed"
i["confirmed_by"] = "a synthetic confirmation, so this arm isolates the stale-input path"
i.pop("unconfirmed_reason", None)
i.pop("owner_leaf", None)
i["inputs"]["grammars/systemverilog.ebnf"] = "0" * 64
json.dump(d, open(sys.argv[2], "w"), indent=2, ensure_ascii=False)
PY
start=$(date +%s)
# ⛔ EXPORTED, not prefixed: `VAR=x some_shell_function` leaves VAR set in the calling shell in
# bash's default mode, which would silently redirect every later arm at the perturbed contract.
export PGEN_SV_CERT_RECOGNIZED_UNION_CONTRACT_FILE="$ROOT/$WORK/stale_contract.json"
arm "arm 13: the CONSUMING GATE refuses to measure on a STALE-INPUT baseline" 2 \
    "REFUSING TO MEASURE" bash rust/scripts/sv_cert_recognized_union_gate.sh
unset PGEN_SV_CERT_RECOGNIZED_UNION_CONTRACT_FILE
# ⭐ …and on the REAL contract, whose expectations are unconfirmed. THIS is the arm that makes the
# gate's RED diagnosable: it names the reason and the owning leaf in under a second, instead of
# measuring for minutes and reporting a drift nobody can attribute.
arm "arm 13b: the CONSUMING GATE refuses on the REAL unconfirmed contract" 2 \
    "REFUSING TO MEASURE" bash rust/scripts/sv_cert_recognized_union_gate.sh
elapsed=$(( $(date +%s) - start ))
if [ "$elapsed" -le 60 ]; then
  echo "  ✓ arm 13 cost ${elapsed}s — the refusal is ahead of the measurement, as designed"
  PASS=$((PASS + 1))
else
  echo "  ✗ arm 13 cost ${elapsed}s — the identity stage is NOT ahead of the heavy work"
  FAIL=$((FAIL + 1))
fi

# ── arms 14-15: THE SECOND CONSUMING GATE, and the escape that is not a bypass ──────────────────
#
# ⛔ EVERY ADOPTION SHIPS BLOCK **AND** READER **AND** AN OBSERVED REFUSAL — that is this leaf's own
# acceptance rule, and `SV-CORPUS-GRAD.13i` is why. `sv_syntax_closure_gate` is the second reader,
# so it gets the same two arms the first one has.
python3 - "$CLOSURE" "$WORK/stale_closure.json" <<'PYARM'
import json, sys
d = json.load(open(sys.argv[1]))
i = d["identity"]
i["expectations"] = "confirmed"
i["confirmed_by"] = "a synthetic confirmation, so this arm isolates the stale-input path"
i.pop("unconfirmed_reason", None)
i.pop("owner_leaf", None)
i["inputs"]["grammars/systemverilog.ebnf"] = "0" * 64
json.dump(d, open(sys.argv[2], "w"), indent=2, ensure_ascii=False)
PYARM
export PGEN_SV_SYNTAX_CLOSURE_CONTRACT="$ROOT/$WORK/stale_closure.json"
arm "arm 14: the SECOND consuming gate refuses on a STALE-INPUT baseline" 2 \
    "REFUSING TO MEASURE" bash rust/scripts/sv_syntax_closure_gate.sh

# ⛔⛔ AND THE CONFIRMING-RUN ESCAPE MUST NOT BE A BLANKET BYPASS. With the escape set, the identity
# refusal is downgraded to a NOTE — but a REAL contract defect must still stop the run, and it must
# still stop it BEFORE the ~90 s of regeneration. Here the two unreachable fields are made to
# disagree, which is a contract-load check sitting after the identity stage and before the work.
python3 - "$WORK/stale_closure.json" "$WORK/inconsistent_closure.json" <<'PYARM'
import json, sys
d = json.load(open(sys.argv[1]))
d["constraints"]["max_unreachable_rules"] = 99      # disagrees with the 3-entry allow-list
json.dump(d, open(sys.argv[2], "w"), indent=2, ensure_ascii=False)
PYARM
export PGEN_SV_SYNTAX_CLOSURE_CONTRACT="$ROOT/$WORK/inconsistent_closure.json"
export PGEN_SV_SYNTAX_CLOSURE_CONFIRMING_RUN=1
# ⛔ The expected substring must not straddle a NEWLINE — `grep -qF` is line-scoped, and the gate
# wraps this message over two lines. Matching the second line alone is what actually binds.
arm "arm 15: the confirming-run escape downgrades ONLY the identity, not the contract checks" 2 \
    "3-entry constraints.allowed_unreachable_rules list" \
    bash rust/scripts/sv_syntax_closure_gate.sh
unset PGEN_SV_SYNTAX_CLOSURE_CONTRACT PGEN_SV_SYNTAX_CLOSURE_CONFIRMING_RUN

echo ""
echo "probe: $PASS arm(s) behaved as specified, $FAIL did not"
[ "$FAIL" -eq 0 ] || exit 1
