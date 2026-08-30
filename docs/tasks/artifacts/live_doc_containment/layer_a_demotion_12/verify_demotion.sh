#!/usr/bin/env bash
# `LIVE-DOC-CONTAINMENT.12` — PROVE the layer-A demotion was LOSSLESS.
#
# A demotion is only safe if every fact it removes from layer A still exists in
# a durable layer. This re-derives that, fact by fact, instead of asserting it.
#
# ⛔ THE CENSUS IS ONLY AS GOOD AS ITS TOKEN. Building this list, 2 of 21 probes
# were FALSE NEGATIVES because the token assumed a spelling layer B does not
# use: `484 cells` (layer B writes `484 (input, column) cells`) and
# `SV-CORPUS-GRAD.13c.2x.8` (a tree refers to its own leaves BARE, as `.13c.2x.8`
# — the undercount `DOCTRINE-GAP-OWNERSHIP.8` already records). Both failed in
# the SAFE direction (they would have blocked a sound demotion, not hidden a
# loss), and the tokens below are the corrected ones.
#
# Usage: bash docs/tasks/artifacts/live_doc_containment/layer_a_demotion_12/verify_demotion.sh
set -uo pipefail
cd "$(git rev-parse --show-toplevel)" || exit 2

# Layers B/C/D only — layer A (MEMORY.md) is EXCLUDED by construction, because
# "still in the file we are demoting from" is not durability.
HAYSTACK=(docs CHANGES.md DEVELOPMENT_NOTES.md TOOLBOX.md)

# ⛔ THIS FILE MUST EXCLUDE ITSELF. It lives under docs/ and it QUOTES every token
# it searches for, so without this it is its own durable layer and every probe
# passes vacuously. Caught by the RED control below on the very first run — which
# is the whole argument for having one.
SELF="${BASH_SOURCE[0]#./}"
SELF="${SELF#"$(git rev-parse --show-toplevel)/"}"

fail=0
probe () { # $1 = fact label, $2 = distinguishing regex
  local hit
  hit=$(grep -rIl -E "$2" "${HAYSTACK[@]}" 2>/dev/null \
          | grep -v -e '^MEMORY.md$' -e "^${SELF}\$" | head -2 | tr '\n' ' ')
  if [ -n "$hit" ]; then
    printf '  ok    %-46s -> %s\n' "$1" "$hit"
  else
    printf '  LOST  %-46s -> found in NO durable layer\n' "$1"
    fail=1
  fi
}

echo "layer-a-demotion: every fact removed from MEMORY.md by LIVE-DOC-CONTAINMENT.12"
probe "in_flight: .67 ingestion bound"           '2 ?595×|8 ?158×'
probe "in_flight: .67 40 000 netlist instances"  '40 ?000 netlist'
probe "in_flight: -0131 determinism evidence"    '484 \(input, column\) cells'
probe "in_flight: director problem-solving rule" 'logging, reporting mindset|never_just_logged'
probe "in_flight: -0091 mandatory_node_gated"    'mandatory_node_gated'
probe "in_flight: -0091 SEVEN functions"         'SEVEN functions reach'
probe "in_flight: DGO.18 slice-id collisions"    'census_slice_id_collisions'
probe "in_flight: DGO.18 62 ids >1 commit"       '62 ids name more than one'
probe "in_flight: DGO.19 decline arm"            'EXEMPT FOREVER'
probe "blockers: -0294 13e.8 refuted"            'gather_verified_profile_proof_covered_rules'
probe "blockers: its 323-of-323 set match"       '323 of 323'
probe "blockers: satisfiable_under trap"         'satisfiable_under'
probe "blockers: LIVE-DOC-CONTAINMENT.8"         'LIVE-DOC-CONTAINMENT'
probe "blockers: routed leaf .13c.2x.8"          '13c\.2x\.8'
probe "blockers: CI-PARITY-GATE-ROT.40/.41/.42"  'CI-PARITY-GATE-ROT\.4[012]'
probe "blockers: ENGINE-UNIVERSAL-SERVICES.44"   'create_contextual_error'
probe "blockers: DGO.15 76 of 108"               '76 of 108'
probe "blockers: SV-CORPUS-GRAD.13e.12(d)"       '13e\.12'
probe "blockers: DGO.16 registry backticks"      'DOCTRINE-GAP-OWNERSHIP\.16'
probe "blockers: 3 open director calls"          'LESSON-RETRIEVAL\.2'

# RED CONTROL — a control never seen fail is not known to work. This token is
# deliberately absent from the repository; if the probe reports it as FOUND the
# search is matching something it should not and every `ok` above is suspect.
echo "layer-a-demotion: RED control (must report LOST)"
before=$fail
probe "RED control (must be LOST)"               'zzz-layer-a-demotion-control-must-not-exist-zzz'
if [ "$fail" -eq "$before" ]; then
  echo "layer-a-demotion: ✗ the RED control was FOUND — the census cannot fail, so it proves nothing"
  exit 3
fi
fail=$before   # the control is expected to be LOST; do not count it as a loss

if [ "$fail" -ne 0 ]; then
  echo "layer-a-demotion: ✗ at least one demoted fact exists in NO durable layer — restore it before demoting"
  exit 1
fi
echo "layer-a-demotion: OK — all 20 demoted facts are durable outside layer A, and the control can go RED"
