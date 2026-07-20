#!/usr/bin/env bash
# PGEN-RGX-0078-0203 expected-delta review: every regenerated artifact may
# differ from its banked pre-regen copy ONLY inside the `-0203` surfaces —
# the unified packed tape (`deriv_tape`/`deriv_cursor`/`TapeWord`), the
# one-segment thin memo (`ThinTapeMemoEntry`), the unified marks/truncates/
# compactions, and their doc comments. Any hunk outside those regions fails
# the review. This is a tripwire on top of the manual diff read, not a
# replacement for it.
set -uo pipefail
R=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)
S=${R0203_SCRATCH:?set R0203_SCRATCH to the session scratch dir}
overall=0

ALLOW='RGX-0078\.5\.j\.4|`-0203`|-0203'
# the new unified-tape spellings (either diff side)
ALLOW+='|deriv_tape|deriv_cursor|TapeWord|ThinTapeMemoEntry'
ALLOW+='|__pgen_spec_mark|__pgen_cand_mark|__pgen_cand_len|__pgen_or_mark'
ALLOW+='|__pgen_opt_mark|__pgen_quant_mark|__pgen_iter_mark|__pgen_la_mark'
ALLOW+='|__pgen_orch_mark|__pgen_thin_mark|__pgen_thin_seg'
# the OLD two-lane spellings this fix removes (old side, and any stray add)
ALLOW+='|deriv_events|deriv_boundary|deriv_ev_cursor|deriv_b_cursor'
ALLOW+='|ThinDerivSegMemoEntry'
ALLOW+='|__pgen_spec_ev_mark|__pgen_spec_b_mark|__pgen_cand_ev_start'
ALLOW+='|__pgen_cand_b_start|__pgen_cand_ev_len|__pgen_cand_b_len'
ALLOW+='|__pgen_or_ev_mark|__pgen_or_b_mark|__pgen_opt_ev_mark'
ALLOW+='|__pgen_quant_ev_mark|__pgen_iter_ev_mark|__pgen_iter_b_mark'
ALLOW+='|__pgen_la_ev_mark|__pgen_la_b_mark|__pgen_orch_ev_mark'
ALLOW+='|__pgen_orch_b_mark|__pgen_thin_ev_mark|__pgen_thin_b_mark'
ALLOW+='|__pgen_thin_ev_seg|__pgen_thin_b_seg'
# event vocabulary lines that moved between push/encode forms
ALLOW+='|DerivEvent'
# doc-comment lines of the changed helpers/fields
ALLOW+='|unified packed|packed-word|tag-0 word|tape order|one packed-word lane'
ALLOW+='|ONE unified lane|unified tape|word-range|tape words|decode the word'
ALLOW+='|hard-checked|sole hard-checked|boundary record|arena reference'
ALLOW+='|The patch is total|narrow by construction|bounded by SAFETY_LIMIT'
ALLOW+='|interleave in append order|single memcpy|one copy \+ one truncate'
ALLOW+='|winner.?s interleaved segment|Records carry no absolute'
ALLOW+='|Tape records are|tape-index-free|Boundary words carry'
ALLOW+='|tape vec is|(cleared|clear) keeps capacity|build-walk scratch'
ALLOW+='|two build cursors|two cursors are build-walk|unconsumed tape words'
ALLOW+='|use crate::ast_pipeline::\{CascadeControlError, CascadeResult, TapeWord\};'
ALLOW+='|use crate::ast_pipeline::\{CascadeControlError, CascadeResult\};'
# prettyplease reflow shards of the changed constructs
ALLOW+='|^[<>][[:space:]]*(self|parser|p)$'
ALLOW+='|^[<>][[:space:]]*\.push\('
ALLOW+='|^[<>][[:space:]]*\.len\(\)'
ALLOW+='|^[<>][[:space:]]*\.truncate\('
ALLOW+='|^[<>][[:space:]]*\.copy_within\('
ALLOW+='|^[<>][[:space:]]*\.extend_from_slice\('
ALLOW+='|^[<>][[:space:]]*&mut parser'
ALLOW+='|^[<>][[:space:]]*crate::ast_pipeline::'
ALLOW+='|^[<>][[:space:]]*parser\.position(,| - __pgen_matched\.len\(\),?)?$'
ALLOW+='|^[<>][[:space:]]*parser\.position = parse_start;$'
ALLOW+='|^[<>][[:space:]]*let parser = p;$'
ALLOW+='|^[<>][[:space:]]*if let Some\(\(\)\) = parser$'
ALLOW+='|^[<>][[:space:]]*\.try_parse_bare\(\|p\| \{$'
ALLOW+='|^[<>][[:space:]]*let __pgen_matched = match parser$'
ALLOW+='|^[<>][[:space:]]*let __pgen_alt_child = match parser'
ALLOW+='|^[<>][[:space:]]*Ok\(__pgen_v\) => __pgen_v,$'
ALLOW+='|^[<>][[:space:]]*Err\(__pgen_e\) =>'
ALLOW+='|^[<>][[:space:]]*return Err\(parser\.cascade_error_from_parse\(__pgen_e\)\);$'
ALLOW+='|^[<>][[:space:]]*Ok\(\(\)\)$'
ALLOW+='|^[<>][[:space:]]*(//.*)?$'
ALLOW+='|^[<>][[:space:]]*[]){};,[]*[[:space:]]*$'
# prettyplease reflow shards of the changed constructs:
# - the new wrapped `let __pgen_cand_len = parser.deriv_tape.len() - __pgen_cand_start;`
ALLOW+='|^>[[:space:]]*- __pgen_cand_start;$'
# - the REMOVED second debug_assert pair (its opening, messages, rule-name arg)
ALLOW+='|^<[[:space:]]*debug_assert_eq!\($'
ALLOW+='|unconsumed (tape events|boundary nodes|tape words)'
ALLOW+='|^<[[:space:]]*"[a-z_0-9]+",$'
# - the OLD direct-enum patch payload lines (now wrapped inside narrow_event)
ALLOW+='|^<[[:space:]]*iteration_count,$'
ALLOW+='|^<[[:space:]]*best_branch_index,$'
ALLOW+='|^<[[:space:]]*true,$'
# - the OLD two-segment memo struct-literal opener (the new one-segment form
#   wraps differently)
ALLOW+='|^<[[:space:]]*outcome: Some\(\($'

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
  # Cancel pure block MOVES first: a line removed and re-added with identical
  # content (the diff artifact of shrunken neighborhoods) is not a delta. The
  # residual = the SYMMETRIC DIFFERENCE of the two sides' line multisets,
  # re-annotated with its side and then ALLOW-filtered.
  diff "$pre" "$post" | grep '^< ' | sed 's/^< //' | sort > "$S/.delta_old"
  diff "$pre" "$post" | grep '^> ' | sed 's/^> //' | sort > "$S/.delta_new"
  stray=$(
    {
      comm -23 "$S/.delta_old" "$S/.delta_new" | sed 's/^/< /'
      comm -13 "$S/.delta_old" "$S/.delta_new" | sed 's/^/> /'
    } | grep -vE "$ALLOW" || true
  )
  if [[ -n "$stray" ]]; then
    echo "UNEXPECTED DELTA in $name:"
    echo "$stray" | head -20
    overall=1
  else
    changed=$(diff "$pre" "$post" | grep -cE '^[<>]' || true)
    echo "EXPECTED-ONLY DELTA: $name ($changed changed lines, all in the -0203 surfaces)"
  fi
done

echo "REVIEW COMPLETE overall=$overall"
exit $overall
