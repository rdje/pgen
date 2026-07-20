#!/usr/bin/env bash
# PGEN-RGX-0078-0202 — cascade-error-boundary DESIGN census (read-only).
# Re-derives every structural fact the -0202 design pre-registration cites,
# against the CUSTODY-ASSERTED -0201 floor-vintage artifact. Refuses to
# report on custody mismatch (the -0172/-0173 precedent). Whole-file
# perl -0777 matching per the standing generated-artifact rule (a
# line-oriented count over a generated artifact is WRONG until proven
# otherwise).
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
cd "$ROOT"

ART=generated/regex_parser.rs
WANT_SHA=3814aea1
GOT_SHA=$(shasum -a 256 "$ART" | cut -c1-8)
if [ "$GOT_SHA" != "$WANT_SHA" ]; then
  echo "CUSTODY FAIL: $ART sha256 prefix $GOT_SHA != banked $WANT_SHA — REFUSING to report" >&2
  exit 1
fi
echo "custody: $ART sha256 prefix $GOT_SHA OK (the -0201 floor vintage)"

REGION_START=$(grep -m1 -n 'fn cascade_' "$ART" | cut -d: -f1)
echo "fused region start line: $REGION_START (first fn cascade_*)"
awk -v s="$REGION_START" 'NR>=s' "$ART" > /tmp/pgen_0202_region.rs
awk -v s="$REGION_START" 'NR<s'  "$ART" > /tmp/pgen_0202_prelude.rs

echo "fused fns: cascade=$(grep -c 'fn cascade_' /tmp/pgen_0202_region.rs) scan=$(grep -c 'fn scan_' /tmp/pgen_0202_region.rs)"

echo "-- ParseError constructions INSIDE the fused region:"
for v in Backtrack InvalidSyntax RecursionDepthExceeded ContextualError UnexpectedEof UnexpectedToken; do
  n=$(perl -0777 -ne 'my $c=()=/ParseError\s*::\s*'"$v"'\s*\{/g; print $c' /tmp/pgen_0202_region.rs)
  echo "   ParseError::$v = $n"
done

echo "-- UnexpectedEof/UnexpectedToken constructions across ALL 11 artifacts:"
for f in generated/*_parser.rs generated/ebnf.rs; do
  eof=$(perl -0777 -ne 'my $c=()=/ParseError\s*::\s*UnexpectedEof\s*\{/g; print $c' "$f")
  tok=$(perl -0777 -ne 'my $c=()=/ParseError\s*::\s*UnexpectedToken\s*\{/g; print $c' "$f")
  echo "   $f: UnexpectedEof=$eof UnexpectedToken=$tok"
done

echo "-- fused-region helper-call census (the error-channel producers/edges; whole-file \\s* matching per the generated-artifact rule):"
for pat in 'match_lit_ascii' 'match_string' 'match_regex' 'try_parse_bare'; do
  n=$(perl -0777 -ne 'my $c=()=/parser\s*\.\s*'"$pat"'\s*\(/g; print $c' /tmp/pgen_0202_region.rs)
  echo "   parser.$pat = $n"
done
echo "   parser.scan_* = $(perl -0777 -ne 'my $c=()=/parser\s*\.\s*scan_[a-z_0-9]+\s*\(/g; print $c' /tmp/pgen_0202_region.rs)"
echo "   parser.parse_* (protocol boundary call-outs) = $(perl -0777 -ne 'my $c=()=/parser\s*\.\s*parse_[a-z_0-9]+\s*\(/g; print $c' /tmp/pgen_0202_region.rs)"
echo "   distinct boundary callees:"
perl -0777 -ne 'while (/parser\s*\.\s*(parse_[a-z_0-9]+)\s*\(/g) { print "$1\n" }' /tmp/pgen_0202_region.rs | sort | uniq -c | sort -rn | sed 's/^/     /'

echo "-- the region's only outbound error edge (sub-root orchestrators):"
echo "   cascade fns returning ParseResult<ParseNode> = $(grep -c 'fn cascade_[a-z_0-9]*(&mut self) -> ParseResult<ParseNode' /tmp/pgen_0202_region.rs)"
echo "   .cascade_match_* called from the prelude = $(grep -c '\.cascade_match_' /tmp/pgen_0202_prelude.rs || true) (must be 0)"
echo "   .cascade_* twin-dispatch sites in the prelude = $(grep -c '\.cascade_' /tmp/pgen_0202_prelude.rs)"
echo "   .scan_* protocol-side sites in the prelude = $(grep -c '\.scan_' /tmp/pgen_0202_prelude.rs)"

echo "-- rich-error reachability (inbound): create_contextual_error callers (prelude) = $(grep -c 'create_contextual_error' /tmp/pgen_0202_prelude.rs); inside region = $(grep -c 'create_contextual_error' /tmp/pgen_0202_region.rs || true)"

rm -f /tmp/pgen_0202_region.rs /tmp/pgen_0202_prelude.rs
echo "CENSUS OK"
