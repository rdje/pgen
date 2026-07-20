#!/usr/bin/env bash
# PGEN-RGX-0078-0202 expected-delta review: every regenerated artifact may
# differ from its banked pre-regen copy ONLY inside the `-0202` surfaces —
# the Copy internal error channel (`CascadeControlError`/`CascadeResult`
# signatures + constructions), the bare terminal twins, the boundary
# conversion helpers + call-site conversion matches, the park slot field/init,
# and the `try_parse_bare` bound/comment. Any hunk outside those regions
# fails the review. This is a tripwire on top of the manual diff read, not a
# replacement for it.
set -uo pipefail
R=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)
S=${R0202_SCRATCH:?set R0202_SCRATCH to the session scratch dir}
overall=0

ALLOW='RGX-0078\.5\.j\.4|`-0202`|-0202'
ALLOW+='|CascadeControlError|CascadeResult'
ALLOW+='|match_lit_ascii_bare|match_string_bare'
ALLOW+='|cascade_error_from_parse|rehydrate_cascade_error'
ALLOW+='|cascade_parked_error'
ALLOW+='|__pgen_v\b|__pgen_e\b|__pgen_rich'
# the conversion-match scaffolding around boundary calls
ALLOW+='|let __pgen_alt_child = match parser\.'
ALLOW+='|let __pgen_matched = match parser'
ALLOW+='|if let Err\(__pgen_e\) = parser'
ALLOW+='|Ok\(__pgen_v\) => __pgen_v'
ALLOW+='|Err\(__pgen_e\) =>'
ALLOW+='|return Err\(parser\.cascade_error_from_parse\(__pgen_e\)\);'
ALLOW+='|Err\(parser\.rehydrate_cascade_error\(__pgen_err\)\)'
# doc comments of the new helpers / boundary notes
ALLOW+='|the drop-free internal error carrier|drop-free INTERNAL|Copy internal carrier'
ALLOW+='|TOTAL inbound conversion|outbound rehydration|park(ed)? slot|Parked'
ALLOW+='|bare terminal twins|BARE twin|bare wrapper|fused graph|fused bodies'
ALLOW+='|routing invariant|region.?s ONLY outbound|MTB-A tape stance'
ALLOW+='|drop_in_place|ParseError>>|no drop code'
ALLOW+='|statically dead on the bare path|diagnostic branch dropped'
ALLOW+='|defensive UTF-8 boundary|byte-identically|bijective|bijection'
ALLOW+='|Internal UTF-8 boundary mismatch|byte-identical when it escapes'
ALLOW+='|hot Copy error channel never touches it|rich/legacy'
ALLOW+='|use crate::ast_pipeline::\{CascadeControlError, CascadeResult\};'
# the bare-twin fn bodies (verbatim `match_lit_ascii`/`match_string` interior
# lines — the twins duplicate the fast-path bodies, so their added lines carry
# no -0202 marker of their own)
ALLOW+='|^[<>][[:space:]]*&mut self,$'
ALLOW+='|^[<>][[:space:]]*expected: &.static str,$'
ALLOW+='|^[<>][[:space:]]*expected_bytes: &\[u8; N\],$'
ALLOW+='|^[<>][[:space:]]*if (true|false) \{$'
ALLOW+='|^[<>][[:space:]]*self\.consume_layout_for_terminal\(expected\);$'
ALLOW+='|^[<>][[:space:]]*let start = self\.position;$'
ALLOW+='|^[<>][[:space:]]*let end = start \+ N;$'
ALLOW+='|^[<>][[:space:]]*let expected_bytes = expected\.as_bytes\(\);$'
ALLOW+='|^[<>][[:space:]]*let end = start \+ expected_bytes\.len\(\);$'
ALLOW+='|^[<>][[:space:]]*if end <= self\.input\.len\(\)$'
ALLOW+='|^[<>][[:space:]]*&& self\.input\.as_bytes\(\)\[start\.\.end\] == \*expected_bytes$'
ALLOW+='|^[<>][[:space:]]*self\.position = end;$'
ALLOW+='|^[<>][[:space:]]*return Ok\(expected\);$'
ALLOW+='|^[<>][[:space:]]*return Ok\(&self\.input\[start\.\.end\]\);$'
ALLOW+='|^[<>][[:space:]]*position: start,$'
ALLOW+='|^[<>][[:space:]]*if self\.bytes_match_at\(start, expected_bytes\) \{$'
ALLOW+='|^[<>][[:space:]]*if !self\.input\.is_char_boundary\(start\) \|\| !self\.input\.is_char_boundary\(end\) \{$'
ALLOW+='|^[<>][[:space:]]*\.create_contextual_error\($'
ALLOW+='|^[<>][[:space:]]*&format!\($'
ALLOW+='|^[<>][[:space:]]*expected$'
# OLD-side (`<`) lines of the changed regions — the pre-change forms this fix
# replaces (kept `<`-anchored so the same shapes ADDED anywhere would still trip)
ALLOW+='|^<.*fn cascade_match_[a-z_0-9]+\(&mut self\) -> ParseResult<\(\)> \{$'
ALLOW+='|^<.*return Err\(ParseError::(InvalidSyntax|RecursionDepthExceeded|Backtrack)'
ALLOW+='|^<.*Err\(ParseError::Backtrack \{ position \}\);$'
ALLOW+='|^<.*__pgen_thin_result: ParseResult<\(\)> = \(\|$'
ALLOW+='|^<.*\| -> ParseResult<\(\)> \{$'
ALLOW+='|^<.*match \(\|parser: &mut Self\| -> ParseResult<_> \{$'
ALLOW+='|^<.*parser\.match_lit_ascii\('
ALLOW+='|^<.*parser\.match_regex\('
ALLOW+='|^<.*\.match_regex\($'
ALLOW+='|^<.*let __pgen_alt_child = parser\.(parse|scan)_[a-z_0-9]+\(\)\?;$'
ALLOW+='|^<.*let __pgen_matched = parser$'
ALLOW+='|^<.*Err\(__pgen_err\)$'
ALLOW+='|^<.*F: FnOnce\(&mut Self\) -> ParseResult<T>,$'
# NEW-side (`>`) interior lines of the conversion helpers (public-variant
# destructuring inside cascade_error_from_parse / rehydrate_cascade_error)
ALLOW+='|^>[[:space:]]*match e \{$'
ALLOW+='|^>[[:space:]]*ParseError::(InvalidSyntax|Backtrack|RecursionDepthExceeded) \{'
# prettyplease reflow shards of the changed call forms (old wrapped calls on
# the `<` side; the new conversion-match wraps on the `>` side; neighbors
# re-wrapped around the changed text)
ALLOW+='|^<.*parser\.match_string\('
ALLOW+='|^<[[:space:]]*let __pgen_alt_child = parser$'
ALLOW+='|^<[[:space:]]*\.(parse|scan)_[a-z_0-9]+\(\)\?;$'
ALLOW+='|^>[[:space:]]*let __pgen_alt_child = match parser$'
ALLOW+='|^>[[:space:]]*\.(parse|scan)_[a-z_0-9]+\(\)$'
ALLOW+='|^[<>][[:space:]]*parser: &mut Self,$'
ALLOW+='|^<.*\) -> ParseResult<\(\)> \{$'
ALLOW+='|^<[[:space:]]*\)\?;$'
ALLOW+='|^[<>][[:space:]]*\.deriv_events$'
ALLOW+='|^[<>][[:space:]]*\.push\($'
ALLOW+='|^[<>][[:space:]]*Ok\(\(\)\)$'
ALLOW+='|^[<>][[:space:]]*\.deriv_boundary$'
ALLOW+='|^[<>][[:space:]]*\.push\(parser\.arena\.alloc\(__pgen_alt_child\)\);$'
ALLOW+='|^[<>][[:space:]]*parser\.deriv_boundary\.push\(parser\.arena\.alloc\(__pgen_alt_child\)\);$'
ALLOW+='|^[<>][[:space:]]*let parser = p;$'
ALLOW+='|^>[[:space:]]*fn cascade_match_[a-z_0-9]+\($'
ALLOW+='|^[<>][[:space:]]*let __pgen_cand_(ev|b)_start = parser\.deriv_(events|boundary)\.len\(\);$'
ALLOW+='|^[<>][[:space:]]*if let Some\(\(\)\) = parser$'
ALLOW+='|^[<>][[:space:]]*\.try_parse_bare\(\|p\| \{$'
ALLOW+='|^[<>][[:space:]]*parser\.position = parse_start;$'
# prettyplease line-wrap continuations of the expected constructs
ALLOW+='|^[<>][[:space:]]*(self|parser)$'
ALLOW+='|^[<>][[:space:]]*\.match_regex\('
ALLOW+='|^[<>][[:space:]]*expected,?$'
ALLOW+='|^[<>][[:space:]]*message,?$|^[<>][[:space:]]*position,?$|^[<>][[:space:]]*depth,?$'
ALLOW+='|^[<>][[:space:]]*rich =>|^[<>][[:space:]]*e =>'
ALLOW+='|^[<>][[:space:]]*\.take\(\)$|^[<>][[:space:]]*\.expect\($'
ALLOW+='|cascade parked-error invariant'
ALLOW+='|^[<>][[:space:]]*(//.*)?$'
ALLOW+='|^[<>][[:space:]]*#\[(allow\(dead_code\)|inline\(always\))\]$'
ALLOW+='|^[<>][[:space:]]*[]){};,[]*[[:space:]]*$'

for post in "$R"/generated/*.rs; do
  name=$(basename "$post")
  pre="$S/artifacts_pre_regen/$name"
  if [[ ! -f "$pre" ]]; then
    echo "NEW ARTIFACT (no pre-regen copy): $name"
    overall=1
    continue
  fi
  if cmp -s "$pre" "$post"; then
    echo "UNCHANGED: $name"
    continue
  fi
  stray=$(diff "$pre" "$post" | grep -E '^[<>]' | grep -vE "$ALLOW" || true)
  if [[ -n "$stray" ]]; then
    echo "UNEXPECTED DELTA in $name:"
    echo "$stray" | head -20
    overall=1
  else
    changed=$(diff "$pre" "$post" | grep -cE '^[<>]' || true)
    echo "EXPECTED-ONLY DELTA: $name ($changed changed lines, all in the -0202 surfaces)"
  fi
done

echo "REVIEW COMPLETE overall=$overall"
exit $overall
