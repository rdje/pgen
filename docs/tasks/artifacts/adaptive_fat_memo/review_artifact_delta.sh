#!/usr/bin/env bash
# PGEN-RGX-0078-0214 expected-delta review: every regenerated artifact may
# differ from its banked pre-regen (`-0212`-vintage) copy ONLY in the
# protocol-memo constructor pre-size (the fixed 256 -> the adaptive
# `(input.len() + 1).min(256)`) and its rewritten doc comment. Any hunk
# outside that region fails the review. Tripwire on top of the manual diff
# read, not a replacement.
set -uo pipefail
R=$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)
S=${R0214_SCRATCH:?set R0214_SCRATCH to the session scratch dir}
overall=0

# OLD side: the fixed-256 constructor line + the "Optim #7" comment block.
ALLOW='memo: rustc_hash::FxHashMap::with_capacity_and_hasher\(256, Default::default\(\)\),'
ALLOW+='|Optim #7|rehash$|rehash |memo entries; large ones produce more|case without growth'
ALLOW+='|a one-time allocation at parser construction'
# NEW side: the adaptive constructor + the `-0214` comment block.
ALLOW+='|memo: rustc_hash::FxHashMap::with_capacity_and_hasher\($'
ALLOW+='|\(input\.len\(\) \+ 1\)\.min\(256\),'
ALLOW+='|RGX-0078\.5\.j\.4|-0214|-0213|ADAPTIVE protocol-memo pre-size'
ALLOW+='|thin-memo$|thin-memo |thin rules bypass|MemoEntry|tiny-input hazard'
ALLOW+='|alloc\+free EVERY parse|machine-pinned|SV-hot|K3a|no-growth guarantee'
ALLOW+='|BYTE-IDENTICAL for|inputs .* 255 B|sizes$|grows on demand'
# shared reflow shards
ALLOW+='|^[<>][[:space:]]*Default::default\(\),?$'
ALLOW+='|^[<>][[:space:]]*\),?$'
ALLOW+='|^[<>][[:space:]]*(//.*)?$'
# the artifact-side old constructor argument renders as a bare `256,` line
# (prettyplease had already split the call; the emitter comment is not
# carried into artifacts)
ALLOW+='|^[<>][[:space:]]*256,$'

for post in "$R"/generated/*.rs; do
  name=$(basename "$post")
  pre="$S/artifacts_pre_regen/$name"
  if [[ ! -f "$pre" ]]; then
    echo "NEW ARTIFACT (no pre-regen copy): $name"
    overall=1
    continue
  fi
  if cmp -s "$pre" "$post"; then
    echo "REFUSE: $name unchanged — the emitter change did not reach it"
    overall=1
    continue
  fi
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
    echo "EXPECTED-ONLY DELTA: $name ($changed changed lines, all in the -0214 surface)"
  fi
done

echo "REVIEW COMPLETE overall=$overall"
exit $overall
