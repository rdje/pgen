#!/usr/bin/env bash
# SV-CORPUS-GRAD.13c.2f(d) + .13c.2g — the refusal arms of the LRM extractor's output guard.
#
# ⛔ WHY A GUARD AT ALL. `tools/extract_systemverilog_lrm_profiles.py` SEEDS grammars; it does not
# maintain them. Measured 2026-08-17: the tracked sibling it "auto-generates" differs from a fresh
# run by 289 lines, and the DELIVERABLE `grammars/systemverilog.ebnf` — which the tool can be
# pointed at with `--output-active-ebnf`, exactly as `DEVELOPMENT_NOTES.md` records — differs by
# 7 277 lines, 121 rules and, decisively, **1 090 return annotations against 0**. A blind write
# deletes the AST contract every downstream consumer is shaped by and re-introduces this tool's own
# 21 `$`-transliterated terminals over their hand fixes.
#
# ⛔ EVERY ARM IS RUN AGAINST A COPY FIRST, then against the real tracked paths — because an arm
# that only ever runs on a copy proves the guard fires on copies. The real-path arms assert the
# tracked file's sha256 is UNCHANGED afterwards, which is the property that actually matters.
#
#   bash docs/tasks/artifacts/sv_corpus_grad/es13c2f4d_extractor_guard/probe.sh
#   # EXTRACTOR-GUARD: N/N as declared
#
# The total is DERIVED from the arms that ran — never a stored number.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT"

TOOL="tools/extract_systemverilog_lrm_profiles.py"
MD17="docs/systemverilog/2017/md/section-41-data-read-api.md"
MD23="docs/systemverilog/2023/md/section-Annex_A-normative-formal-syntax.md"
SHIPPED="grammars/systemverilog.ebnf"
SIBLING="grammars/systemverilog_lrm_profiled_generated.ebnf"
W="rust/target/es13c2f4d_guard_probe"

rm -rf "$W"; mkdir -p "$W"
pass=0; total=0

sha() { shasum -a 256 "$1" | cut -d' ' -f1; }

run_tool() {  # run_tool <out-ebnf> <out-active|-> <out-report> [extra…]
  local out="$1" active="$2" report="$3"; shift 3
  if [ "$active" = "-" ]; then
    python3 "$TOOL" --md-2017 "$MD17" --md-2023 "$MD23" \
      --output-ebnf "$out" --output-report "$report" "$@" 2>&1
  else
    python3 "$TOOL" --md-2017 "$MD17" --md-2023 "$MD23" \
      --output-ebnf "$out" --output-active-ebnf "$active" --output-report "$report" "$@" 2>&1
  fi
}

arm() {  # arm <name> <expect-rc 0|nonzero> <expect-substring> <command…>
  local name="$1" want_rc="$2" want_txt="$3"; shift 3
  total=$((total+1))
  local out rc
  out="$("$@" 2>&1)"; rc=$?
  local ok=1
  if [ "$want_rc" = "0" ]; then [ "$rc" -eq 0 ] || ok=0; else [ "$rc" -ne 0 ] || ok=0; fi
  case "$out" in *"$want_txt"*) ;; *) ok=0 ;; esac
  if [ "$ok" = "1" ]; then pass=$((pass+1)); printf '  ✓ %-46s rc=%s\n' "$name" "$rc"
  else printf '  ✗ %-46s rc=%s (wanted rc %s + %s)\n     %s\n' "$name" "$rc" "$want_rc" "$want_txt" "${out:0:400}"; fi
}

echo "EXTRACTOR-GUARD — SV-CORPUS-GRAD.13c.2f(d) / .13c.2g"
echo

# ── A1 GREEN: fresh paths write, and the header is repo-root-RELATIVE (.13c.2g) ─────────────────
arm "A1 fresh scratch outputs are written" 0 "output_ebnf: written" \
    run_tool "$W/a1.ebnf" "$W/a1_active.ebnf" "$W/a1.json"
total=$((total+1))
if head -8 "$W/a1.ebnf" | grep -q '^# - sv_2017: docs/systemverilog/' \
   && ! head -8 "$W/a1.ebnf" | grep -q '^# - sv_2017: /'; then
  pass=$((pass+1)); printf '  ✓ %-46s\n' "A2 header cites sources repo-relatively"
else printf '  ✗ %-46s\n' "A2 header cites sources repo-relatively"; fi

# ── A3 GREEN: an identical re-run is a no-op, not a refusal ──────────────────────────────────────
arm "A3 identical re-run reports unchanged" 0 "unchanged (byte-identical)" \
    run_tool "$W/a1.ebnf" "$W/a1_active.ebnf" "$W/a1.json"

# ── A4/A5 RED on COPIES: a differing target is refused ───────────────────────────────────────────
cp "$SHIPPED" "$W/copy_active.ebnf"; cp "$SIBLING" "$W/copy_sibling.ebnf"
before_active="$(sha "$W/copy_active.ebnf")"; before_sibling="$(sha "$W/copy_sibling.ebnf")"
arm "A4 copy of the DELIVERABLE is refused" 1 "REFUSING to overwrite" \
    run_tool "$W/a4.ebnf" "$W/copy_active.ebnf" "$W/a4.json"
arm "A5 copy of the SIBLING is refused" 1 "REFUSING to overwrite" \
    run_tool "$W/copy_sibling.ebnf" "-" "$W/a5.json"
total=$((total+1))
if [ "$(sha "$W/copy_active.ebnf")" = "$before_active" ] \
   && [ "$(sha "$W/copy_sibling.ebnf")" = "$before_sibling" ]; then
  pass=$((pass+1)); printf '  ✓ %-46s\n' "A6 refused copies are byte-unchanged"
else printf '  ✗ %-46s — THE GUARD WROTE ANYWAY\n' "A6 refused copies are byte-unchanged"; fi

# ── A7/A8 RED on the REAL TRACKED PATHS, with the sha asserted afterwards ────────────────────────
real_active="$(sha "$SHIPPED")"; real_sibling="$(sha "$SIBLING")"
arm "A7 the REAL deliverable path is refused" 1 "REFUSING to overwrite" \
    run_tool "$W/a7.ebnf" "$SHIPPED" "$W/a7.json"
arm "A8 the REAL sibling path is refused" 1 "REFUSING to overwrite" \
    run_tool "$SIBLING" "-" "$W/a8.json"
total=$((total+1))
if [ "$(sha "$SHIPPED")" = "$real_active" ] && [ "$(sha "$SIBLING")" = "$real_sibling" ]; then
  pass=$((pass+1)); printf '  ✓ %-46s\n' "A9 tracked grammars are byte-unchanged"
else printf '  ✗ %-46s — RESTORE FROM GIT NOW\n' "A9 tracked grammars are byte-unchanged"; fi

# ── A10 the escape hatch is real, and only on a scratch path ─────────────────────────────────────
printf 'not the extractor output\n' > "$W/a10.ebnf"
arm "A10 --promote-outputs overwrites deliberately" 0 "output_ebnf: written" \
    run_tool "$W/a10.ebnf" "-" "$W/a10.json" --promote-outputs

# ── A11 the census the post-condition owes: it is REPORTED, and non-empty ────────────────────────
total=$((total+1))
n="$(python3 -c "import json,sys; print(json.load(open('$W/a1.json'))['transliterated_dollar_token_count'])" 2>/dev/null || echo -1)"
if [ "$n" -gt 0 ] 2>/dev/null; then
  pass=$((pass+1)); printf '  ✓ %-46s (%s tokens)\n' "A11 report carries the \$-transliteration census" "$n"
else printf '  ✗ %-46s (%s)\n' "A11 report carries the \$-transliteration census" "$n"; fi

# ── A12 the shipped deliverable carries NONE of them — the fix this guard protects ───────────────
total=$((total+1))
s="$(grep -cE '^kw_[A-Za-z0-9_]+ := trivia /[A-Za-z0-9_]*_dollar' "$SHIPPED" || true)"
if [ "$s" = "0" ]; then
  pass=$((pass+1)); printf '  ✓ %-46s\n' "A12 shipped grammar has 0 _dollar terminals"
else printf '  ✗ %-46s (%s found)\n' "A12 shipped grammar has 0 _dollar terminals" "$s"; fi

# ── A13/A14 THE RED CONTROL — the version WITHOUT the guard must destroy the same copy ───────────
# ⛔ Without this, every refusal above is equally consistent with "the guard works" and "nothing was
# ever going to be written here". The subject is the PREVIOUS tool, taken from git rather than
# reconstructed, so the only difference between the two arms is the change under test.
# ⚠️ It is looked up by CONTENT, not by revision: once the guard is committed, HEAD carries it, so a
# hard-coded `git show HEAD:` would silently start testing the guarded version and pass vacuously.
prev=""
for rev in $(git rev-list -n 40 HEAD -- "$TOOL"); do
  if ! git show "$rev:$TOOL" 2>/dev/null | grep -q "REFUSING to overwrite"; then prev="$rev"; break; fi
done
total=$((total+1))
if [ -n "$prev" ]; then
  pass=$((pass+1)); printf '  ✓ %-46s (%s)\n' "A13 an UNGUARDED revision of the tool exists" "${prev:0:8}"
  git show "$prev:$TOOL" > "$W/unguarded.py"
  cp "$SHIPPED" "$W/victim.ebnf"
  v_before="$(sha "$W/victim.ebnf")"; l_before="$(grep -c '^\s*-> ' "$W/victim.ebnf" || true)"
  python3 "$W/unguarded.py" --md-2017 "$MD17" --md-2023 "$MD23" \
      --output-ebnf "$W/unguarded.ebnf" --output-active-ebnf "$W/victim.ebnf" \
      --output-report "$W/unguarded.json" >/dev/null 2>&1
  urc=$?
  l_after="$(grep -c '^\s*-> ' "$W/victim.ebnf" || true)"
  total=$((total+1))
  if [ "$urc" -eq 0 ] && [ "$(sha "$W/victim.ebnf")" != "$v_before" ] && [ "$l_after" = "0" ]; then
    pass=$((pass+1))
    printf '  ✓ %-46s rc=%s, return annotations %s -> %s\n' \
      "A14 the unguarded tool DESTROYS it, silently" "$urc" "$l_before" "$l_after"
  else
    printf '  ✗ %-46s rc=%s, annotations %s -> %s (the RED arm did not go RED)\n' \
      "A14 the unguarded tool DESTROYS it, silently" "$urc" "$l_before" "$l_after"
  fi
else
  printf '  ✗ %-46s — cannot prove the guard changed anything\n' "A13 an UNGUARDED revision of the tool exists"
fi

echo
echo "EXTRACTOR-GUARD: $pass/$total as declared"
[ "$pass" = "$total" ]
