#!/usr/bin/env bash
# scripts/check_svpp_expansion_contract_currency.sh — doctrine SVPP-EXPANSION-CONTRACT-CURRENCY
# (SVPP-EXPANSION.9.2, 2026-10-20).
#
# ⛔ WHAT THIS PROVES. The SystemVerilog preprocessing-service contract
# (docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_EXPANSION_CONTRACT.md) describes the service that
# is actually in the tree. Every vocabulary the contract publishes is RE-DERIVED from the code on
# every run and compared as a SET in both directions:
#   A — the two stability axes: the service release version (`PREPROCESSING_SERVICE_VERSION`,
#       rust/src/families/systemverilog/preprocessing_service.rs) and the preprocessed-output
#       schema version (`OUTPUT_SCHEMA_VERSION`, rust/src/preprocessing_stage.rs) equal the
#       contract's identity block;
#   B — the declared option table (key, values, default) of the family's stage
#       (rust/src/families/systemverilog/mod.rs `STAGE_OPTIONS`) equals the contract's option table;
#   C — the event kinds (`PreprocessorEventKind::as_str`) equal the contract's "Event kinds" line;
#   D — the diagnostic codes (every `"[EW]_SVPP_…"` literal in the stage) equal the contract's
#       "Diagnostic codes" line;
#   E — the refusal codes (`refusal_code`, one per `RefusalKind`) equal the contract's "Refusal
#       codes" line;
#   F — every surface the contract names exists in the code (the typed API, the provider, the
#       typed refusal, the probe's two emit flags), and the contract names every one of them;
#   G — no `CONTRACTED FOR `.9.2`` tag survives in the contract or in the svpp book chapter: a
#       surface that shipped is LIVE, and a tag that outlives its step is a stale promise.
#
# ⛔ WHY IT EXISTS. PGEN-SERVICES.1 (2026-10-19) measured the service register: the one contracted
# service family (the embedding API) is the model BECAUSE a gate holds its document to the code;
# every other published surface had rotted or never had a contract. A contract without a currency
# gate is a claim; this script is what makes row 5 of the register a BLOCK (PGEN-SERVICES.2's
# doctrine shape). The vocabularies above are exactly the ones the contract's versioning rules
# call ADDITIVE: a new option, kind or code must land in the document in the same commit, or the
# commit is refused here.
#
# HONEST BOUND. This script reads SOURCE TEXT (no build, no binary): it proves the document and
# the code agree about NAMES, versions and defaults; whether the code does what the document says
# is the business of the stage's 146 unit tests and the push mirror's SV lanes.
#
# --self-test drives the RED arms on mutated COPIES of the contract (a removed refusal code, a
# changed service version, a leftover tag, a changed option default, a removed event kind) and a
# GREEN control on the real tree; every arm must fire or the self-test exits 1.
#
# Exit codes: 0 PASS, 1 FAIL, 2 usage.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT"
TAG="svpp-expansion-contract-currency"

CONTRACT="${PGEN_SVPP_EXPANSION_CONTRACT:-docs/contracts/PGEN_SYSTEMVERILOG_PREPROCESSOR_EXPANSION_CONTRACT.md}"
CHAPTER="${PGEN_SVPP_EXPANSION_CHAPTER:-docs/systemverilog_preprocessor_parser_book/src/preprocessing-service.md}"
SERVICE="rust/src/families/systemverilog/preprocessing_service.rs"
STAGE="rust/src/preprocessing_stage.rs"
PP="rust/src/families/systemverilog/preprocessor.rs"
FAMILY="rust/src/families/systemverilog/mod.rs"
PROBE="rust/src/bin/parseability_probe.rs"

fails=()
fail() { fails+=("$1"); }

# ---- set comparison helpers (LC_ALL=C: a set has no order, and `sort`'s order belongs to the host —
# CI-PARITY-GATE-ROT.122) --------------------------------------------------------------------------
sorted() { LC_ALL=C sort -u; }
# bullet_text <file> <the bullet's bold label, literally> — the first bullet whose line carries the
# label and its wrapped continuation lines (indented, not a new bullet), joined with spaces: a list
# the author wraps at 120 columns is one bullet to the reader and one bullet here. The label is
# matched with index(), not as a regex — awk's -v processes backslashes, so a regex with `\*` is
# not portable between GNU and BSD awk.
bullet_text() {
  awk -v key="$2" '
    found && ($0 ~ /^[ \t]*- / || $0 !~ /^[ \t]+[^ \t]/) { exit }
    found { sub(/^[ \t]+/, " "); printf "%s", $0; next }
    $0 ~ /^[ \t]*- / && index($0, key) > 0 { found = 1; printf "%s", $0 }
    END { if (found) printf "\n" }' "$1"
}
diff_sets() { # diff_sets <label> <derived-newline-list> <published-newline-list>
  local label="$1" derived="$2" published="$3" only_code only_doc
  only_code="$(comm -23 <(printf '%s\n' "$derived" | sed '/^$/d' | sorted) <(printf '%s\n' "$published" | sed '/^$/d' | sorted))"
  only_doc="$(comm -13 <(printf '%s\n' "$derived" | sed '/^$/d' | sorted) <(printf '%s\n' "$published" | sed '/^$/d' | sorted))"
  if [ -n "$only_code" ]; then fail "$label: in the code and NOT in the contract: $(printf '%s' "$only_code" | tr '\n' ' ')"; fi
  if [ -n "$only_doc" ]; then fail "$label: in the contract and NOT in the code: $(printf '%s' "$only_doc" | tr '\n' ' ')"; fi
}

run_check() {
  fails=()
  for f in "$CONTRACT" "$CHAPTER" "$SERVICE" "$STAGE" "$PP" "$FAMILY" "$PROBE"; do
    [ -f "$f" ] || fail "missing file: $f"
  done
  if [ "${#fails[@]}" -gt 0 ]; then return 1; fi

  # A — the two axes
  local code_service code_schema doc_service doc_schema
  code_service="$(grep -oE 'pub const PREPROCESSING_SERVICE_VERSION: &str = "[^"]+"' "$SERVICE" | grep -oE '"[^"]+"' | tr -d '"')"
  code_schema="$(grep -oE 'pub const OUTPUT_SCHEMA_VERSION: u32 = [0-9]+' "$STAGE" | grep -oE '[0-9]+$')"
  doc_service="$(grep -oE '^\s*- Service release version: `[^`]+`' "$CONTRACT" | head -n 1 | grep -oE '`[^`]+`' | tr -d '`')"
  doc_schema="$(grep -oE '^\s*- Preprocessed-output schema version: `[0-9]+`' "$CONTRACT" | head -n 1 | grep -oE '`[0-9]+`' | tr -d '`')"
  [ -n "$code_service" ] || fail "A: PREPROCESSING_SERVICE_VERSION not found in $SERVICE"
  [ -n "$code_schema" ] || fail "A: OUTPUT_SCHEMA_VERSION not found in $STAGE"
  [ -n "$doc_service" ] || fail "A: the contract carries no '- Service release version: \`X\`' line"
  [ -n "$doc_schema" ] || fail "A: the contract carries no '- Preprocessed-output schema version: \`N\`' line"
  if [ -n "$code_service" ] && [ -n "$doc_service" ] && [ "$code_service" != "$doc_service" ]; then
    fail "A: service release version — code $code_service, contract $doc_service"
  fi
  if [ -n "$code_schema" ] && [ -n "$doc_schema" ] && [ "$code_schema" != "$doc_schema" ]; then
    fail "A: preprocessed-output schema version — code $code_schema, contract $doc_schema"
  fi

  # B — the option table: key|values|default
  local code_opts doc_opts
  code_opts="$(grep -oE 'StageOptionSpec \{ key: "[^"]+", values: "[^"]+", default: "[^"]*"' "$FAMILY" \
    | sed -E 's/StageOptionSpec \{ key: "([^"]+)", values: "([^"]+)", default: "([^"]*)"/\1|\2|\3/')"
  doc_opts="$(grep -E '^\| `[a-z_]+` \| `[^|]*(\\\|[^|]*)*` \| `[^|]*` \|$' "$CONTRACT" \
    | sed -E 's/\\\|/\x01/g' | sed -E 's/^\| `([^`]+)` \| `([^`]*)` \| `([^`]*)` \|$/\1|\2|\3/' | tr '\001' '|')"
  [ -n "$code_opts" ] || fail "B: no StageOptionSpec rows found in $FAMILY"
  [ -n "$doc_opts" ] || fail "B: no option-table rows (| \`key\` | \`values\` | \`default\` |) found in the contract"
  diff_sets "B options" "$code_opts" "$doc_opts"

  # C — event kinds
  local code_kinds doc_kinds
  code_kinds="$(grep -oE 'PreprocessorEventKind::[A-Za-z]+ => "[a-z_]+"' "$PP" | grep -oE '"[a-z_]+"' | tr -d '"')"
  doc_kinds="$(bullet_text "$CONTRACT" '**Event kinds**' | grep -oE '`[a-z_]+`' | tr -d '`')"
  [ -n "$code_kinds" ] || fail "C: no PreprocessorEventKind::as_str arms found in $PP"
  [ -n "$doc_kinds" ] || fail "C: the contract carries no '- **Event kinds**' line"
  diff_sets "C event kinds" "$code_kinds" "$doc_kinds"

  # D — diagnostic codes
  local code_diag doc_diag
  code_diag="$(grep -oE '"[EW]_SVPP_[A-Z_0-9]+"' "$PP" | tr -d '"')"
  doc_diag="$(bullet_text "$CONTRACT" '**Diagnostic codes**' | grep -oE '`[EW]_SVPP_[A-Z_0-9]+`' | tr -d '`')"
  [ -n "$code_diag" ] || fail "D: no [EW]_SVPP_ literals found in $PP"
  [ -n "$doc_diag" ] || fail "D: the contract carries no '- **Diagnostic codes**' line"
  diff_sets "D diagnostic codes" "$code_diag" "$doc_diag"

  # E — refusal codes
  local code_ref doc_ref
  code_ref="$(grep -oE 'RefusalKind::[A-Za-z]+ => "E_SVPP_[A-Z_0-9]+"' "$SERVICE" | grep -oE '"E_SVPP_[A-Z_0-9]+"' | tr -d '"')"
  doc_ref="$(bullet_text "$CONTRACT" '**Refusal codes**' | grep -oE '`E_SVPP_[A-Z_0-9]+`' | tr -d '`')"
  [ -n "$code_ref" ] || fail "E: no refusal_code arms found in $SERVICE"
  [ -n "$doc_ref" ] || fail "E: the contract carries no '- **Refusal codes**' line"
  diff_sets "E refusal codes" "$code_ref" "$doc_ref"

  # F — surfaces: code ⇒ contract names them; contract names ⇒ code has them
  # <file>|<declaration the code must carry>|<the name the contract must mention in a code span>
  local surfaces=(
    "$SERVICE|pub fn preprocess(|preprocess"
    "$SERVICE|pub fn preprocess_str(|preprocess_str"
    "$SERVICE|pub struct PreprocessContext|PreprocessContext"
    "$SERVICE|pub struct PreprocessRefusal|PreprocessRefusal"
    "$SERVICE|pub fn refusal_code(|refusal_code"
    "$STAGE|pub trait FileProvider|FileProvider"
    "$STAGE|pub struct MemoryFileProvider|MemoryFileProvider"
    "$PROBE|\"--emit-preprocessed\"|--emit-preprocessed"
    "$PROBE|\"--emit-source-map\"|--emit-source-map"
  )
  local entry sfile sdecl sname
  for entry in "${surfaces[@]}"; do
    IFS='|' read -r sfile sdecl sname <<< "$entry"
    grep -qF -- "$sdecl" "$sfile" || fail "F: the code no longer carries '$sdecl' ($sfile) — the contract names \`$sname\`"
    grep -qF -- "\`$sname\`" "$CONTRACT" || fail "F: the contract does not name \`$sname\` (declared in $sfile)"
  done
  for sname in "preprocess_str" "--emit-preprocessed" "FileProvider"; do
    grep -qF -- "\`$sname\`" "$CHAPTER" || fail "F: the svpp book chapter ($CHAPTER) does not name \`$sname\`"
  done

  # G — no stale step tag
  local stale
  stale="$(grep -nF 'CONTRACTED FOR `.9.2`' "$CONTRACT" "$CHAPTER" | head -n 5)"
  [ -z "$stale" ] || fail "G: a 'CONTRACTED FOR \`.9.2\`' tag survives the step that shipped it: $(printf '%s' "$stale" | tr '\n' ' ')"

  if [ "${#fails[@]}" -gt 0 ]; then
    for f in "${fails[@]}"; do echo "$TAG: ✗ $f" >&2; done
    echo "$TAG: FAIL (${#fails[@]} finding(s)) — $CONTRACT vs the tree" >&2
    return 1
  fi
  echo "$TAG: PASS — service $code_service, schema $code_schema, $(printf '%s\n' "$code_opts" | grep -c .) options, $(printf '%s\n' "$code_kinds" | grep -c .) event kinds, $(printf '%s\n' "$code_diag" | sorted | grep -c .) diagnostic codes, $(printf '%s\n' "$code_ref" | grep -c .) refusal codes, ${#surfaces[@]} surfaces, 0 stale tags"
  return 0
}

self_test() {
  local scratch="$ROOT/rust/target/doctrine_checks/svpp_expansion_contract_currency"
  rm -rf "$scratch"; mkdir -p "$scratch"
  local arms=0 fired=0 real_contract="$CONTRACT" real_chapter="$CHAPTER"
  arm() { # arm <name> <expected: 0|1> <contract-path> <chapter-path>
    local name="$1" expected="$2" rc
    arms=$((arms + 1))
    if PGEN_SVPP_EXPANSION_CONTRACT="$3" PGEN_SVPP_EXPANSION_CHAPTER="$4" bash "$0" >"$scratch/$name.log" 2>&1; then rc=0; else rc=1; fi
    if [ "$rc" = "$expected" ]; then fired=$((fired + 1)); echo "$TAG self-test: ✓ $name (rc=$rc as expected)"; else echo "$TAG self-test: ✗ $name (rc=$rc, expected $expected) — $(tail -n 3 "$scratch/$name.log" | tr '\n' ' ')"; fi
  }
  arm "green-control-on-the-tree" 0 "$real_contract" "$real_chapter"
  # RED 1: a refusal code removed from the contract
  sed -E 's/`E_SVPP_INCLUDE_NOT_FOUND`, //' "$real_contract" > "$scratch/no_refusal_code.md"
  arm "red-refusal-code-removed" 1 "$scratch/no_refusal_code.md" "$real_chapter"
  # RED 2: the service version changed in the contract
  sed -E 's/^(\s*- Service release version: `)[^`]+`/\19.9.9`/' "$real_contract" > "$scratch/wrong_version.md"
  arm "red-service-version-moved" 1 "$scratch/wrong_version.md" "$real_chapter"
  # RED 3: a leftover step tag in the chapter
  { cat "$real_chapter"; printf '\nThe file provider is CONTRACTED FOR `.9.2`.\n'; } > "$scratch/stale_tag.md"
  arm "red-stale-step-tag" 1 "$real_contract" "$scratch/stale_tag.md"
  # RED 4: an option default changed in the contract
  sed -E 's/^\| `include_max_depth` \| `N` \| `64` \|$/| `include_max_depth` | `N` | `15` |/' "$real_contract" > "$scratch/wrong_default.md"
  arm "red-option-default-moved" 1 "$scratch/wrong_default.md" "$real_chapter"
  # RED 5: an event kind removed from the contract
  sed -E 's/`keywords_region`//' "$real_contract" > "$scratch/no_event_kind.md"
  arm "red-event-kind-removed" 1 "$scratch/no_event_kind.md" "$real_chapter"
  # RED 6: the contract no longer names a surface
  sed -E 's/`preprocess_str`/`preprocess_text`/g' "$real_contract" > "$scratch/no_surface.md"
  arm "red-surface-unnamed" 1 "$scratch/no_surface.md" "$real_chapter"
  echo "$TAG self-test: $fired/$arms arms observed as expected"
  [ "$fired" = "$arms" ]
}

case "${1:-}" in
  "") run_check ;;
  --self-test) self_test ;;
  -h|--help) sed -n '2,40p' "$0"; exit 0 ;;
  *) echo "usage: $0 [--self-test]" >&2; exit 2 ;;
esac
