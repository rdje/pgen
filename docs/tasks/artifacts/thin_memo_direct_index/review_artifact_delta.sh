#!/usr/bin/env bash
# PGEN-RGX-0078-0205 expected-delta review: every regenerated artifact may
# differ from its banked pre-regen copy ONLY inside the `-0205` surfaces —
# the thin memo's direct-index carrier (`thin_scratch`/`thin_entries`/
# `thin_stride`, the `THIN_ROW_*`/`THIN_RULE_COUNT` consts, the slot
# lookup/store/clear frame) replacing the `thin_memo` FxHashMap container,
# and their doc comments. Any hunk outside those regions fails the review.
# This is a tripwire on top of the manual diff read, not a replacement for
# it.
set -uo pipefail
R=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)
S=${R0205_SCRATCH:?set R0205_SCRATCH to the session scratch dir}
overall=0

ALLOW='RGX-0078\.5\.j\.4|`-0205`|-0205'
# the NEW direct-index spellings (either diff side)
ALLOW+='|thin_scratch|thin_entries|thin_stride|THIN_ROW_|THIN_RULE_COUNT'
ALLOW+='|ThinMemoScratchLease|__pgen_thin_lease|__pgen_thin_slot|__pgen_thin_idx'
# the OLD container spellings this fix removes (old side, and any stray add)
ALLOW+='|thin_memo|__pgen_thin_key|FxHashMap'
# the shared frame locals whose lines reflow around the container swap
ALLOW+='|__pgen_thin_entry|__pgen_thin_valid|__pgen_thin_stale|__pgen_thin_seg'
ALLOW+='|__pgen_thin_end|__pgen_thin_mark|__pgen_thin_result|ThinTapeMemoEntry'
# the OLD map-capacity ctor argument lines (the formula moved onto the new
# entries-vec line) and the map hasher arg
ALLOW+='|\(\(input\.len\(\) \+ 1\) \* 6\)\.min\(32768\)'
ALLOW+='|Default::default\(\),'
# prettyplease wrap shards: the new slot expression's second line, the OLD
# map field's generic-parameter lines, and the OLD key tuple's wrapped
# members (long rule-const names force the tuple onto multiple lines)
ALLOW+='|^>[[:space:]]*\+ position;$'
ALLOW+='|^<[[:space:]]*>,$'
ALLOW+='|^<[[:space:]]*\(RuleId, usize\),$'
ALLOW+='|^<[[:space:]]*position,$'
ALLOW+='|^<[[:space:]]*Self::RULE_[A-Z_0-9]+,$'
# doc-comment lines of the changed constructs
ALLOW+='|direct-index|generation-stamped|generation compare|row table'
ALLOW+='|memo skip|dense entry|bounded garbage|K3a formula|store-epoch churn'
ALLOW+='|group probe|slot load|slot store|slot clear|slot encoding'
ALLOW+='|recycled|per-byte bound|Stale eviction'
# prettyplease reflow shards of the changed constructs
ALLOW+='|^[<>][[:space:]]*(self|parser|p)$'
ALLOW+='|^[<>][[:space:]]*\.push\('
ALLOW+='|^[<>][[:space:]]*\.len\(\)'
ALLOW+='|^[<>][[:space:]]*\.insert\('
ALLOW+='|^[<>][[:space:]]*\.extend_from_slice\('
ALLOW+='|^[<>][[:space:]]*&mut parser'
ALLOW+='|^[<>][[:space:]]*crate::ast_pipeline::'
ALLOW+='|^[<>][[:space:]]*parser\.position,?$'
ALLOW+='|^[<>][[:space:]]*stamp: __pgen_thin_stamp,$'
ALLOW+='|^[<>][[:space:]]*outcome: (None|Some\(\()'
ALLOW+='|^[<>][[:space:]]*Ok\(\(\)\) => \{$'
ALLOW+='|^[<>][[:space:]]*Err\(_\) => \{$'
ALLOW+='|^[<>][[:space:]]*(//.*)?$'
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
    echo "EXPECTED-ONLY DELTA: $name ($changed changed lines, all in the -0205 surfaces)"
  fi
done

echo "REVIEW COMPLETE overall=$overall"
exit $overall
