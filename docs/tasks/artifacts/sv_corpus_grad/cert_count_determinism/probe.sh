#!/usr/bin/env bash
# SV-CORPUS-GRAD.13c.2x.1 — THE DISCRIMINATING PROBE.
#
# ⛔ THE OBSERVATION IT ADJUDICATES. On `.13c.2v`'s `SV-0065` arm, `sv_cert_recognized_union_gate`
# read `canonical total` = 1434 at seed 0 and 1433 at seeds 7 and 42, and its own determinism check
# fired. `certificate_coverage()` is PURE and counts `grammar.rule_order`, so a WITNESS-SEARCH SEED
# cannot legitimately move it — and 25 of the rules it counts are SYNTHESIZED by the LR eliminator
# rather than written in the grammar.
#
# ⛔⛔ THE TWO AXES WERE PERFECTLY CONFOUNDED IN THE ORIGINAL EVIDENCE: the gate runs each seed in a
# SEPARATE PROCESS, so "seed 0 vs seed 7" and "process A vs process B" could not be told apart.
#   axis A (H1, per-process nondeterminism) : N processes, seed HELD FIXED
#   axis B (H2, real seed-dependence)       : one process per seed, seeds VARIED
# Varies on axis A ⇒ H1, and the blast radius is EVERY rule-count baseline in the repository.
# Stable on axis A while axis B moves ⇒ H2, scoped to the seeded path.
#
# ⛔ IT RUNS THE GATE'S OWN CONFIGURATION, NOT A SIMPLIFIED ONE. The number under test was produced
# by `sv_cert_recognized_union_gate.sh` with a profile, an entry rule, 40 samples and four union
# configs; a probe that dropped them would measure a different quantity and could agree with the
# gate by accident. Every parameter below is READ FROM THE CONTRACT the gate reads, never typed
# here — a probe that hard-coded them would keep passing after the contract moved.
set -euo pipefail

# ⛔ THE ROOT IS FOUND BY WALKING UP TO A SENTINEL, NOT BY COUNTING `..`. Three probes in this
# repository have now shipped with the WRONG `..` depth — `accepted_rise_gate`'s, `.13c.2w`'s
# containment probe, and this one's first draft, which refused with `not at the repo root (…/docs)`
# on its first execution. Each was caught by its own pre-flight, so no phantom result was ever
# scored; but a defect that recurs three times is a CLASS, and a depth literal is the thing that
# rots when a probe is moved one directory. Walking up is depth-independent by construction.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
while [[ "$ROOT" != "/" && ! ( -f "$ROOT/grammars/systemverilog.ebnf" && -f "$ROOT/scripts/check_doctrines.sh" ) ]]; do
    ROOT="$(dirname "$ROOT")"
done
[[ -f "$ROOT/grammars/systemverilog.ebnf" && -f "$ROOT/scripts/check_doctrines.sh" ]] || {
    echo "probe: no repo root above $(dirname "${BASH_SOURCE[0]}") (no grammars/systemverilog.ebnf + scripts/check_doctrines.sh)" >&2; exit 2; }
cd "$ROOT" || exit 2

BIN="$ROOT/rust/target/debug/ast_pipeline"
CONTRACT="$ROOT/rust/test_data/grammar_quality/systemverilog_recognized_cert_union_contract.json"
OUT="${1:-$ROOT/rust/target/x1_cert}"
REPEATS="${PGEN_X1_REPEATS:-3}"
mkdir -p "$OUT"

[[ -x "$BIN" ]] || { echo "probe: $BIN missing — build with --features 'generated_parsers ebnf_dual_run'" >&2; exit 2; }
[[ -f "$CONTRACT" ]] || { echo "probe: contract missing: $CONTRACT" >&2; exit 2; }

entry="$(jq -r '.base_entry'   "$CONTRACT")"
profile="$(jq -r '.base_profile' "$CONTRACT")"
samples="$(jq -r '.samples'    "$CONTRACT")"
mapfile -t seeds < <(jq -r '.seeds[]' "$CONTRACT")
union_args=()
while IFS= read -r cfg; do union_args+=(--cert-union-config "$cfg"); done < <(jq -r '.union_configs[]' "$CONTRACT")

fixed_seed="${seeds[0]}"
echo "probe: grammar_sha=$(shasum -a 256 "$ROOT/grammars/systemverilog.ebnf" | cut -c1-16)…"
echo "probe: entry=$entry profile=$profile samples=$samples seeds=${seeds[*]} repeats=$REPEATS"

run_one() {  # $1=label $2=seed
    "$BIN" "$ROOT/grammars/systemverilog.ebnf" --report-certificate-coverage \
        --grammar-profile "$profile" --entry-rule "$entry" \
        --count "$samples" --seed "$2" "${union_args[@]}" > "$OUT/$1.log" 2>&1 || true
}

# ── axis A (H1): REPEATS separate processes, seed held fixed ───────────────────────────────
for i in $(seq 1 "$REPEATS"); do run_one "axisA_seed${fixed_seed}_run${i}" "$fixed_seed" & done
# ── axis B (H2): one process per declared seed ────────────────────────────────────────────
for s in "${seeds[@]}"; do run_one "axisB_seed${s}" "$s" & done
wait

# ⛔ The canonical line is the one the gate asserts `expected_total` against; the union line is
# reported too, because a count that moved in only one of them would be a different defect.
report() {  # $1=log
    local canon union
    canon="$(grep -m1 '^CERTIFICATE-COVERAGE:' "$OUT/$1.log" 2>/dev/null || echo '(missing)')"
    union="$(grep -m1 '^CERTIFICATE-COVERAGE-UNION:' "$OUT/$1.log" 2>/dev/null || echo '(missing)')"
    printf '%-26s canonical[%s]  union[%s]\n' "$1" \
        "$(sed -E 's/.*(total=[0-9]+).*(proof=[0-9]+).*(witness=[0-9]+).*(UNKNOWN=[0-9]+).*/\1 \2 \3 \4/' <<<"$canon")" \
        "$(sed -E 's/.*(total=[0-9]+).*(UNKNOWN=[0-9]+).*/\1 \2/' <<<"$union")"
}
echo; echo "── axis A — ${REPEATS} PROCESSES, seed FIXED at ${fixed_seed} (varies ⇒ H1) ──"
for i in $(seq 1 "$REPEATS"); do report "axisA_seed${fixed_seed}_run${i}"; done
echo; echo "── axis B — one process per seed (moves while A is stable ⇒ H2) ──"
for s in "${seeds[@]}"; do report "axisB_seed${s}"; done

# ── the verdict, computed rather than eyeballed ────────────────────────────────────────────
totals_A="$(for i in $(seq 1 "$REPEATS"); do grep -m1 -o 'total=[0-9]*' "$OUT/axisA_seed${fixed_seed}_run${i}.log" | head -1; done | sort -u | tr '\n' ' ')"
totals_B="$(for s in "${seeds[@]}"; do grep -m1 -o 'total=[0-9]*' "$OUT/axisB_seed${s}.log" | head -1; done | sort -u | tr '\n' ' ')"
nA="$(wc -w <<<"$totals_A")"; nB="$(wc -w <<<"$totals_B")"
echo; echo "X1-CERT-DETERMINISM: axisA_distinct_totals=${nA} [${totals_A}] axisB_distinct_totals=${nB} [${totals_B}]"
if   [[ "$nA" -gt 1 ]]; then echo "VERDICT: H1 — the canonical total is PER-PROCESS NONDETERMINISTIC; every rule-count baseline is affected"; exit 1
elif [[ "$nB" -gt 1 ]]; then echo "VERDICT: H2 — the canonical total is STABLE per process but SEED-DEPENDENT; name the seeded append site"; exit 1
else echo "VERDICT: NEITHER — the canonical total is invariant across processes AND across seeds"; exit 0; fi
