#!/usr/bin/env bash
# rust/scripts/lib/drift_attribution.sh — SV-CORPUS-GRAD.13c.2x.1 (d)
#
# ⛔⛔ WHY THIS EXISTS. A gate that re-reads a MUTABLE input once per iteration and then asserts
# "every iteration agreed" is making a claim about the TOOL. It only means that if the INPUT was the
# same each time — and the gates in this repository that make such a claim record nothing about it.
# Measured 2026-08-20 (`gate_input_pin_census.sh`): `asserts_determinism=5 blind_to_input_change=5`.
#
# ⛔ THE COST OF NOT ATTRIBUTING IS NOT THEORETICAL. `sv_cert_recognized_union_gate` reported
# `canonical total` 1434 at seed 0 and 1433 at seeds 7/42 and printed `seed=7 signature drift vs
# seed 0`. Because that message names the axis the gate VARIED rather than the axis that MOVED, the
# owning task leaf wrote down two hypotheses and both were about the tool — per-process
# nondeterminism, and a seeded append site. Re-measured on the exhibiting arm, the count is
# invariant across three processes AND three seeds (1434 six times of six), so BOTH were false. What
# had differed was the grammar FILE: a multi-seed run takes minutes, and an ordinary apply/revert of
# an experimental arm lands inside that window. Two sessions were spent in a hypothesis space that
# did not contain the answer.
#
# ⭐ ONE IMPLEMENTATION, NOT ONE PER GATE. Five gates share this shape; coding the attribution once
# is what stops it becoming five subtly different messages
# ([[one-metric-name-two-predicates-is-a-contract-defect]]).
#
# ⭐ THE DIGEST IS EVIDENCE, NEVER A NEW FAILURE CONDITION. A byte digest that failed on its own
# would fire on a comment-only edit the frontend strips — the `SV-CORPUS-GRAD.13c.2x.4` defect,
# where one comment line blocked every commit. This helper is called ONLY when a signature has
# ALREADY drifted, so it can turn an unattributable RED into a named cause and can never fail a run
# that would otherwise have passed.
#
# See [[a-determinism-check-over-a-re-read-input-must-pin-the-input]].

# attribute_signature_drift <iter_id> <first_iter_id> <iter_sha> <first_sha> <input_label> \
#                           <signature> <first_signature>
# Prints ONE line describing the drift AND what caused it. Returns 2 if called wrong — a helper
# that silently produced a message from missing arguments would be worse than no helper.
attribute_signature_drift() {
    if [[ $# -ne 7 ]]; then
        echo "attribute_signature_drift: expected 7 arguments, got $# — refusing to compose a" \
             "drift message from incomplete evidence" >&2
        return 2
    fi
    local iter="$1" first_iter="$2" sha="$3" first_sha="$4" label="$5" sig="$6" first_sig="$7"
    if [[ -z "$sha" || -z "$first_sha" ]]; then
        # ⛔ An UNRECORDED input is its own finding, and it must not masquerade as either verdict.
        printf '%s\n' "seed=${iter} signature drift vs seed ${first_iter} — ⚠️ UNATTRIBUTABLE: the \
input digest for at least one iteration was not recorded, so this run cannot say whether \`${label}\` \
changed underneath it. That missing record is the defect SV-CORPUS-GRAD.13c.2x.1 (d) exists to \
close. ('${sig}' != '${first_sig}')"
        return 0
    fi
    if [[ "$sha" != "$first_sha" ]]; then
        printf '%s\n' "seed=${iter} signature drift vs seed ${first_iter} — ⛔ THE INPUT CHANGED \
UNDER THIS RUN: \`${label}\` was ${first_sha:0:16}… at seed ${first_iter} and ${sha:0:16}… at seed \
${iter}. This is NOT evidence that the tool is nondeterministic — the run measured two different \
inputs. Re-run on a quiescent tree. ('${sig}' != '${first_sig}')"
    else
        printf '%s\n' "seed=${iter} signature drift vs seed ${first_iter} with the INPUT HELD \
BYTE-IDENTICAL (\`${label}\` ${sha:0:16}… both times) — so this IS a tool-side nondeterminism \
finding. ⛔ Before hypothesising, DE-CONFOUND iteration from PROCESS: each seed here runs in its own \
process, so \"seed\" and \"process\" are one axis in this evidence, not two. \
docs/tasks/artifacts/sv_corpus_grad/cert_count_determinism/probe.sh varies them independently — N \
processes at ONE fixed seed, then one process per seed. ('${sig}' != '${first_sig}')"
    fi
}
