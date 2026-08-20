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
# ⛔ The union contract is `adopted-unconfirmed`, so `--verify` — the call a GATE makes — must go
# RED. That assertion now lives in arm 14 with its exact exit code; duplicating it here as a
# code-1 arm is what made this probe fail against its own corrected contract.
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
# ⛔⛔ THE CELL A GATE REFUSES ON IS `unconfirmed`, NOT `stale` — SV-CORPUS-GRAD.13c.2x.4.
# An earlier version of this arm forced `confirmed` + a wrong digest to make the gate refuse. Under
# the corrected matrix that combination makes the gate MEASURE — correctly, because running is what
# resolves staleness — so the arm would silently have become a two-minute run inside a probe that
# is meant to take seconds. It did exactly that once while this file was being edited.
python3 - "$ADOPTED" "$WORK/stale_contract.json" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
d["identity"]["inputs"]["grammars/systemverilog.ebnf"] = {
    "kind": "ebnf_raw_ast", "digest": "0" * 64}
json.dump(d, open(sys.argv[2], "w"), indent=2, ensure_ascii=False)
PY
start=$(date +%s)
# ⛔ EXPORTED, not prefixed: `VAR=x some_shell_function` leaves VAR set in the calling shell in
# bash's default mode, which would silently redirect every later arm at the perturbed contract.
export PGEN_SV_CERT_RECOGNIZED_UNION_CONTRACT_FILE="$ROOT/$WORK/stale_contract.json"
arm "arm 13: the CONSUMING GATE refuses on an UNCONFIRMED baseline, before measuring" 2 \
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

# ── arms 14-16: THE FOUR-WAY CONTRACT THE WHOLE MATRIX BRANCHES ON ─────────────────────────────
#
# ⛔ `--verify`'s EXIT CODE **IS** THE CONTRACT — a gate cannot implement the matrix without it, and
# getting it wrong was not theoretical: the first cut let `fail()` collapse UNCONFIRMED (3) into
# STALE (1), so a gate that should have refused in 0 s began a two-minute measurement instead.
python3 - "$CLOSURE" "$WORK/stale_closure.json" <<'PYARM'
import json, sys
d = json.load(open(sys.argv[1]))
# CONFIRMED and stale: the cell where a gate must MEASURE rather than refuse.
d["identity"]["inputs"]["rust/src/ast_pipeline/stimuli_generator.rs"] = "0" * 64
json.dump(d, open(sys.argv[2], "w"), indent=2, ensure_ascii=False)
PYARM
arm "arm 14: an UNCONFIRMED baseline reports 3, not 1 - running would resolve nothing" 3 \
    "ARE UNCONFIRMED" bash "$VERIFY" --verify "$ADOPTED"
arm "arm 15: a STALE baseline reports 1, not 3 - running IS what resolves it" 1 \
    "STALE IS NOT A VERDICT ON THE TREE" bash "$VERIFY" --verify "$WORK/stale_closure.json"

# ⭐⭐ ARM 16 — THE SEMANTIC DIGEST, which is what removes the false staleness at its source.
# A comment-only grammar edit leaves the EBNF frontend's raw_ast envelope — what the code generator
# consumes — byte-identical, so a baseline keyed on it must NOT move. Measured before this landed:
# one comment line made check_doctrines.sh exit 1 and BLOCKED EVERY COMMIT, for a change that
# leaves the generated parser byte-identical.
cp -f grammars/systemverilog.ebnf "$WORK/g.ebnf"
printf '\n(* a comment the frontend strips *)\n' >> "$WORK/g.ebnf"
if python3 - "$WORK/g.ebnf" "$WORK" <<'PYARM'
import hashlib, json, os, subprocess, sys
edited, work = sys.argv[1], sys.argv[2]
def digest(path):
    tmp = os.path.join(work, "ra.json")
    if subprocess.run(["rust/target/debug/ast_pipeline", path, "--emit-raw-ast-json", tmp],
                      capture_output=True).returncode != 0 or not os.path.isfile(tmp):
        sys.exit(2)
    raw = json.load(open(tmp))["raw_ast"]
    os.remove(tmp)
    return hashlib.sha256(
        json.dumps(raw, sort_keys=True, separators=(",", ":")).encode()).hexdigest()
sys.exit(0 if digest(edited) == digest("grammars/systemverilog.ebnf") else 1)
PYARM
then
  echo "  ✓ arm 16: a comment-only grammar edit leaves the raw_ast digest IDENTICAL - the false"
  echo "    staleness that blocked every commit cannot recur"
  PASS=$((PASS + 1))
else
  echo "  ✗ arm 16: the raw_ast digest MOVED on a comment-only edit, or could not be computed -" >&2
  echo "    the semantic kind is not semantic and adoption re-acquires its original cost" >&2
  FAIL=$((FAIL + 1))
fi


# ── arm 17: THE TWO IMPLEMENTATIONS OF THE SEMANTIC DIGEST MUST AGREE ──────────────────────────
#
# ⛔⛔ THIS DOCTRINE DELIBERATELY ADOPTED AN EXISTING DEFINITION RATHER THAN INVENTING A SECOND ONE
# — `scripts/check_sv_contract_currency.sh::sv_semantic_digest` has keyed SV-CONTRACT-CURRENCY on
# the frontend's raw_ast envelope for months. But adopting a definition in a different language
# creates exactly what this repository refuses elsewhere: a SECOND COPY that can drift. Two copies
# are only safe while something compares them, so this arm does.
sv_currency_digest="$(
  # shellcheck disable=SC1090
  ROOT="$ROOT"; PIPELINE="$ROOT/rust/target/debug/ast_pipeline"
  sed -n '/^sv_semantic_digest() {/,/^}/p' scripts/check_sv_contract_currency.sh > "$WORK/dig.sh"
  # shellcheck source=/dev/null
  . "$WORK/dig.sh"
  sv_semantic_digest grammars/systemverilog.ebnf
)"
baseline_identity_digest="$(python3 - <<'PYARM'
import hashlib, json, os, subprocess
tmp = "rust/target/baseline_identity_probe/ra_cmp.json"
if subprocess.run(["rust/target/debug/ast_pipeline", "grammars/systemverilog.ebnf",
                   "--emit-raw-ast-json", tmp], capture_output=True).returncode == 0    and os.path.isfile(tmp):
    raw = json.load(open(tmp))["raw_ast"]
    os.remove(tmp)
    print(hashlib.sha256(
        json.dumps(raw, sort_keys=True, separators=(",", ":")).encode()).hexdigest())
PYARM
)"
if [ -n "$sv_currency_digest" ] && [ "$sv_currency_digest" = "$baseline_identity_digest" ]; then
  echo "  ✓ arm 17: both implementations of the semantic digest agree (${sv_currency_digest:0:16}…)"
  PASS=$((PASS + 1))
elif [ -z "$sv_currency_digest" ] || [ -z "$baseline_identity_digest" ]; then
  echo "  ✗ arm 17: a digest could not be computed, so the two definitions were NOT compared." >&2
  echo "    A check that cannot see must say so, not pass." >&2
  FAIL=$((FAIL + 1))
else
  echo "  ✗ arm 17: THE TWO SEMANTIC-DIGEST IMPLEMENTATIONS HAVE DRIFTED" >&2
  echo "      check_sv_contract_currency.sh : $sv_currency_digest" >&2
  echo "      check_baseline_identity.sh    : $baseline_identity_digest" >&2
  FAIL=$((FAIL + 1))
fi

# ── arms 18-19: THE END-TO-END INVARIANT, IN BOTH DIRECTIONS ────────────────────────────────────
#
# ⛔⛔ THIS IS THE ARM THAT MAKES THE FIX PERMANENT. Everything above tests a mechanism; this tests
# the PROPERTY the whole exercise was for: *editing a comment in a grammar must not stop anyone
# committing.* It ran RED before the fix — one comment line made `check_doctrines.sh` exit 1 across
# TWO doctrines and four separate byte-keyed rows — and it is the only arm that would notice the
# defect coming back through a surface nobody thought to guard.
# ⚠️ It mutates the tracked grammar and restores it byte-identically; the restore is verified.
cp -f grammars/systemverilog.ebnf "$WORK/grammar_backup.ebnf"
printf '\n(* transient probe comment - restored immediately *)\n' >> grammars/systemverilog.ebnf
comment_rc=0
bash scripts/check_doctrines.sh >"$WORK/comment_edit.log" 2>&1 || comment_rc=$?
cp -f "$WORK/grammar_backup.ebnf" grammars/systemverilog.ebnf
if ! git diff --quiet -- grammars/systemverilog.ebnf; then
  echo "  ⛔ THE GRAMMAR WAS NOT RESTORED — fix before committing" >&2
  FAIL=$((FAIL + 1))
elif [ "$comment_rc" = "0" ]; then
  echo "  ✓ arm 18: a COMMENT-ONLY grammar edit leaves every registered doctrine GREEN - nobody is"
  echo "    blocked from committing by an edit the frontend strips"
  PASS=$((PASS + 1))
else
  echo "  ✗ arm 18: a comment-only grammar edit still FAILS a doctrine (rc=$comment_rc):" >&2
  grep -E "✗ FAIL|BASELINE IS STALE|no longer describes" "$WORK/comment_edit.log" | head -4 >&2
  FAIL=$((FAIL + 1))
fi

# ⛔ AND THE CONTROL THAT STOPS ARM 18 BEING SATISFIED BY SIMPLY SWITCHING THE CHECK OFF. A REAL
# semantic edit MUST still be seen. Without this, deleting the guard entirely would score a pass.
printf '\nzz_probe_rule_transient := "zzprobe"\n' >> grammars/systemverilog.ebnf
semantic_seen=0
bash "$VERIFY" >"$WORK/semantic_edit.log" 2>&1 || true
grep -q "STALE" "$WORK/semantic_edit.log" && semantic_seen=1
cp -f "$WORK/grammar_backup.ebnf" grammars/systemverilog.ebnf
if ! git diff --quiet -- grammars/systemverilog.ebnf; then
  echo "  ⛔ THE GRAMMAR WAS NOT RESTORED — fix before committing" >&2
  FAIL=$((FAIL + 1))
elif [ "$semantic_seen" = "1" ]; then
  echo "  ✓ arm 19: a REAL semantic grammar edit is still SEEN as stale - arm 18 was earned, not"
  echo "    bought by weakening the check"
  PASS=$((PASS + 1))
else
  echo "  ✗ arm 19: a real semantic edit went UNNOTICED - the freshness guarantee is gone" >&2
  FAIL=$((FAIL + 1))
fi

echo ""
echo "probe: $PASS arm(s) behaved as specified, $FAIL did not"
[ "$FAIL" -eq 0 ] || exit 1
