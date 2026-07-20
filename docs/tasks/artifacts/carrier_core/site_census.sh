#!/usr/bin/env bash
# PGEN-RGX-0078-0203 — carrier-core site census over the CURRENT generated
# artifacts (custody-asserted; whole-file perl -0777 counts per the -0173
# standing rule: a line-oriented count over a generated artifact is wrong
# until proven otherwise).
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT=$(cd "$SCRIPT_DIR/../../../.." && pwd)
ART="$ROOT/generated/regex_parser.rs"

EXPECTED_SHA="c3d919599e6589ac702420dfb062b3de4b84a44a5548f77941cc75e04bc87b6a"
actual_sha=$(shasum -a 256 "$ART" | awk '{print $1}')
if [[ "$actual_sha" != "$EXPECTED_SHA" ]]; then
    echo "REFUSE: regex artifact sha $actual_sha != expected $EXPECTED_SHA (-0202 vintage)" >&2
    exit 2
fi
echo "custody: generated/regex_parser.rs sha256=$actual_sha OK (-0202 vintage)"
echo

count() {
    local label=$1
    local pattern=$2
    local file=$3
    local n
    n=$(perl -0777 -ne 'my $c = () = /'"$pattern"'/g; print $c' "$file")
    printf '%-58s %6d\n' "$label" "$n"
}

echo "== regex artifact: match-pass tape WRITE sites =="
count "deriv_events.push(  (all)" 'deriv_events\s*\.push\(' "$ART"
count "  OrWinner pushes (placeholder)" 'deriv_events\s*\.push\(\s*crate::ast_pipeline::DerivEvent::OrWinner' "$ART"
count "  QuantCount pushes (placeholder)" 'deriv_events\s*\.push\(\s*crate::ast_pipeline::DerivEvent::QuantCount' "$ART"
count "  OptPresent pushes (placeholder)" 'deriv_events\s*\.push\(\s*crate::ast_pipeline::DerivEvent::OptPresent' "$ART"
count "  TokStart pushes" 'deriv_events\s*\.push\(\s*crate::ast_pipeline::DerivEvent::TokStart' "$ART"
count "  TokEnd pushes" 'deriv_events\s*\.push\(\s*crate::ast_pipeline::DerivEvent::TokEnd' "$ART"
count "deriv_events[..] = (patch sites)" 'deriv_events\[[^\]]*\]\s*=' "$ART"
count "deriv_boundary.push(" 'deriv_boundary\s*\.push\(' "$ART"
echo
echo "== regex artifact: tape RANGE bookkeeping (the second-lane cost) =="
count "deriv_events.len() reads (marks)" 'deriv_events\s*\.len\(\)' "$ART"
count "deriv_boundary.len() reads (marks)" 'deriv_boundary\s*\.len\(\)' "$ART"
count "deriv_events.truncate(" 'deriv_events\s*\.truncate\(' "$ART"
count "deriv_boundary.truncate(" 'deriv_boundary\s*\.truncate\(' "$ART"
count "deriv_events.copy_within(" 'deriv_events\s*\.copy_within\(' "$ART"
count "deriv_boundary.copy_within(" 'deriv_boundary\s*\.copy_within\(' "$ART"
count "deriv_events.extend_from_slice (memo hit)" 'deriv_events\s*\.extend_from_slice\(' "$ART"
count "deriv_boundary.extend_from_slice (memo hit)" 'deriv_boundary\s*\.extend_from_slice\(' "$ART"
count "SmallVec::from_slice (memo insert copies)" 'SmallVec::from_slice\(' "$ART"
count "thin_memo.insert(" 'thin_memo\s*\.insert\(' "$ART"
count "ThinDerivSegMemoEntry uses" 'ThinDerivSegMemoEntry' "$ART"
echo
echo "== regex artifact: build-pass READ sites =="
count "deriv_next_event() calls" 'deriv_next_event\(\)' "$ART"
count "deriv_next_boundary() calls" 'deriv_next_boundary\(\)' "$ART"
count "deriv_ev_cursor mentions" 'deriv_ev_cursor' "$ART"
count "deriv_b_cursor mentions" 'deriv_b_cursor' "$ART"
count "deriv_pos mentions" 'deriv_pos' "$ART"
echo
echo "== regex artifact: region shape =="
count "cascade_match_* fns" 'fn cascade_match_' "$ART"
count "cascade_build_* fns" 'fn cascade_build_' "$ART"
count "scan_* fns" 'fn scan_' "$ART"
count "sub-root orchestrator fns (cascade_<r>, ParseResult)" 'fn cascade_(?!match_|build_)\w+\(&mut self\) -> ParseResult' "$ART"
echo
echo "== all-11 artifacts: carrier-surface presence (DerivEvent / boundary / thin memo / terminal events) =="
for f in "$ROOT"/generated/*_parser.rs "$ROOT"/generated/ebnf.rs; do
    [[ -f "$f" ]] || continue
    base=$(basename "$f")
    ev=$(perl -0777 -ne 'my $c = () = /deriv_events\s*\.push\(/g; print $c' "$f")
    tok=$(perl -0777 -ne 'my $c = () = /DerivEvent::Tok(?:Start|End)/g; print $c' "$f")
    bd=$(perl -0777 -ne 'my $c = () = /deriv_boundary\s*\.push\(/g; print $c' "$f")
    tm=$(perl -0777 -ne 'my $c = () = /ThinDerivSegMemoEntry/g; print $c' "$f")
    printf '%-40s ev_push=%5d tok_ev=%5d b_push=%5d thin_memo_uses=%3d\n' "$base" "$ev" "$tok" "$bd" "$tm"
done
