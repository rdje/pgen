#!/usr/bin/env bash
# SV-CORPUS-GRAD.25 (2026-09-17, on the director's request for ONE number per turn): the SystemVerilog
# RELEASE DISTANCE — how many open items stand between this tree and the SV parser's `Done`, derived from
# tracked artifacts only, appended per release to a tracked history, and turned into a trailing-rate ETA
# that is labeled an ESTIMATE. Nothing here runs a gate; a reader can re-derive every figure from git.
#
#   items = legs_unmet + bar_rows + open_leaves
#     legs_unmet   the SV parser book's own "Distance to `Done`: N of the 3 legs" line (held current by
#                  FAMILY-STATUS-CURRENCY; docs/systemverilog_parser_book/src/status-and-distance.md)
#     bar_rows     `divergence:unexplained_*` rows in the three adjudication manifests (the axis-2 bar the
#                  director's directive names: rows the parser must clear on sv_2017 / sv_2023 / verilog_2005)
#     open_leaves  `##### ⛔|⏳` leaves in docs/tasks/SV-CORPUS-GRAD.md plus `### ⛔` leaves in
#                  docs/tasks/ENGINE-UNIVERSAL-SERVICES.md opened by SV-CORPUS-GRAD (engine work SV owes)
#   corpus_pass_pct / known_defects  the sv_2017 manifest: `match` rows over all rows; unexplained rows
#   ETA          over the last <=5 history rows: rate = items closed per release; ETA = items / rate releases,
#                and days at the observed releases-per-day of the same window. ESTIMATE: it assumes the trailing
#                rate; a structural leg (a corpus conformance gate, the generator proof) is ONE item that costs
#                more than a bar row, and the no-verdict rows (an answer-key gap) are outside `items`.
#
# usage: report_sv_release_distance.sh                 one summary line (for a turn banner)
#        report_sv_release_distance.sh --markdown      the derived block for the SV parser book
#        report_sv_release_distance.sh --check <page>  re-derive and refuse if the page's block differs (rc 2)
#        report_sv_release_distance.sh --record        append this tree's row to the history (idempotent per commit)
#        report_sv_release_distance.sh --record-at <commit> <release>  back-fill one row from git history
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT"
HIST="docs/tasks/artifacts/sv_corpus_grad/release_distance/history.tsv"
PAGE_DEFAULT="docs/systemverilog_parser_book/src/status-and-distance.md"
BEGIN='<!-- BEGIN DERIVED: scripts/report_sv_release_distance.sh --markdown -->'
END='<!-- END DERIVED: scripts/report_sv_release_distance.sh -->'

# read a tracked file at HEAD (working tree) or at a commit
rd() { local at="$1" path="$2"; if [ "$at" = "tree" ]; then cat "$path"; else git show "$at:$path"; fi; }

derive() {  # $1 = tree | <commit>; prints key=value lines
  local at="$1"
  local page; page="$(rd "$at" "$PAGE_DEFAULT")"
  local legs; legs="$(printf '%s\n' "$page" | grep -oE 'Distance to `Done`: [0-9]+ of the 3 legs' | head -1 | grep -oE '^Distance to `Done`: [0-9]+' | grep -oE '[0-9]+$' || true)"
  [ -n "$legs" ] || { echo "error: the status page carries no 'Distance to \`Done\`' line (FAMILY-STATUS-CURRENCY owns it)" >&2; return 2; }
  local b17 b23 b05
  b17="$(rd "$at" stimuli/sv/characterization/adjudication_manifest.tsv | grep -c 'divergence:unexplained' || true)"
  b23="$(rd "$at" stimuli/sv/characterization/adjudication_manifest_sv2023.tsv | grep -c 'divergence:unexplained' || true)"
  b05="$(rd "$at" stimuli/sv/characterization/adjudication_manifest_v2005.tsv | grep -c 'divergence:unexplained' || true)"
  local sv_leaves eng_leaves
  sv_leaves="$(rd "$at" docs/tasks/SV-CORPUS-GRAD.md | grep -cE '^##### (⛔|⏳) `' || true)"
  eng_leaves="$(rd "$at" docs/tasks/ENGINE-UNIVERSAL-SERVICES.md | grep -E '^### ⛔ `' | grep -cE 'opened [0-9-]+ by `SV-CORPUS-GRAD' || true)"
  local rows match
  rows="$(rd "$at" stimuli/sv/characterization/adjudication_manifest.tsv | awk -F'\t' 'NR>1 && NF>=5' | wc -l | tr -d ' ')"
  match="$(rd "$at" stimuli/sv/characterization/adjudication_manifest.tsv | awk -F'\t' 'NR>1 && $5=="match"' | wc -l | tr -d ' ')"
  local pct; pct="$(awk -v m="$match" -v r="$rows" 'BEGIN{ if (r>0) printf "%.1f", 100*m/r; else print "0.0" }')"
  local bar=$((b17 + b23 + b05)) leaves=$((sv_leaves + eng_leaves))
  echo "legs_unmet=$legs"; echo "bar_rows=$bar"; echo "bar_sv_2017=$b17"; echo "bar_sv_2023=$b23"; echo "bar_verilog_2005=$b05"
  echo "open_leaves=$leaves"; echo "open_sv_leaves=$sv_leaves"; echo "open_engine_leaves=$eng_leaves"
  echo "items=$((legs + bar + leaves))"; echo "corpus_rows=$rows"; echo "corpus_match=$match"; echo "corpus_pass_pct=$pct"; echo "known_defects=$b17"
}

eta() {  # reads the history; prints rate/eta fields
  [ -f "$HIST" ] || { echo "rate=n/a"; echo "eta_releases=n/a"; echo "eta_days=n/a"; echo "window=0"; return 0; }
  tail -n +2 "$HIST" | tail -n 5 | awk -F'\t' '
    { n++; d[n]=$1; it[n]=$4; if (!($3 in seen)) { seen[$3]=1; labels++ } }
    END {
      if (n < 2) { print "rate=n/a"; print "eta_releases=n/a"; print "eta_days=n/a"; print "window=" n; exit }
      # rel = RELEASES in the window, not rows: a commit outside a release that moves the distance (a leaf
      # closed or opened) records a row under the shipped release label (SV-CORPUS-GRAD.23, 2026-09-24),
      # so rows can share a label and the per-release rate must not count them twice.
      closed = it[1] - it[n]; rel = labels - 1
      if (rel < 1) { print "rate=n/a"; print "eta_releases=n/a (one release in the window)"; print "eta_days=n/a"; print "window=" n; exit }
      cmd = "date -j -f %Y-%m-%dT%H:%M:%S " substr(d[1],1,19) " +%s 2>/dev/null || date -d " d[1] " +%s"; cmd | getline t1; close(cmd)
      cmd = "date -j -f %Y-%m-%dT%H:%M:%S " substr(d[n],1,19) " +%s 2>/dev/null || date -d " d[n] " +%s"; cmd | getline t2; close(cmd)
      days = (t2 - t1) / 86400.0; rpd = (days > 0) ? rel / days : 0
      rate = closed / rel
      printf "rate=%.2f\n", rate; printf "window=%d\n", n
      if (rate <= 0) { print "eta_releases=n/a (no net closure over the window)"; print "eta_days=n/a" }
      else { er = it[n] / rate; printf "eta_releases=%.1f\n", er; if (rpd > 0) printf "eta_days=%.1f\n", er / rpd; else print "eta_days=n/a" }
    }'
}

summary_line() {
  local -A v; while IFS='=' read -r k val; do v[$k]="$val"; done < <(derive tree)
  local -A e; while IFS='=' read -r k val; do e[$k]="$val"; done < <(eta)
  printf 'SV-RELEASE-DISTANCE: items=%s (legs_unmet=%s bar_rows=%s [sv_2017=%s sv_2023=%s verilog_2005=%s] open_leaves=%s [sv=%s engine=%s]) corpus_pass_pct=%s known_defects=%s | trailing rate=%s items/release over %s rows, ETA≈%s releases ≈ %s days (ESTIMATE)\n' \
    "${v[items]}" "${v[legs_unmet]}" "${v[bar_rows]}" "${v[bar_sv_2017]}" "${v[bar_sv_2023]}" "${v[bar_verilog_2005]}" "${v[open_leaves]}" "${v[open_sv_leaves]}" "${v[open_engine_leaves]}" "${v[corpus_pass_pct]}" "${v[known_defects]}" "${e[rate]}" "${e[window]}" "${e[eta_releases]}" "${e[eta_days]}"
}

markdown_block() {
  local -A v; while IFS='=' read -r k val; do v[$k]="$val"; done < <(derive tree)
  local -A e; while IFS='=' read -r k val; do e[$k]="$val"; done < <(eta)
  echo "$BEGIN"
  echo "**SV release distance: ${v[items]} items** (\`legs_unmet + bar_rows + open_leaves\`)."
  echo
  echo "| Component | Count | Derived from |"
  echo "|---|---:|---|"
  echo "| Done-bar legs unmet | ${v[legs_unmet]} of 3 | the derived block above (\`report_family_status.sh\`) |"
  echo "| Corpus bar rows (unexplained divergences) | ${v[bar_rows]} (sv_2017 ${v[bar_sv_2017]}, sv_2023 ${v[bar_sv_2023]}, verilog_2005 ${v[bar_verilog_2005]}) | \`stimuli/sv/characterization/adjudication_manifest*.tsv\` |"
  echo "| Open leaves owning SV work | ${v[open_leaves]} (SV-CORPUS-GRAD ${v[open_sv_leaves]}, engine leaves SV opened ${v[open_engine_leaves]}) | \`docs/tasks/SV-CORPUS-GRAD.md\`, \`docs/tasks/ENGINE-UNIVERSAL-SERVICES.md\` |"
  echo "| Corpus passing (sv_2017, \`match\` rows) | ${v[corpus_match]} of ${v[corpus_rows]} (${v[corpus_pass_pct]} %) | the sv_2017 manifest |"
  echo "| Known defects (unexplained, sv_2017) | ${v[known_defects]} | the sv_2017 manifest |"
  echo
  echo "**Trailing closure rate:** ${e[rate]} items per release over the last ${e[window]} recorded release(s); **ETA ≈ ${e[eta_releases]} releases ≈ ${e[eta_days]} days** — an ESTIMATE that assumes the trailing rate holds; a structural leg is one item that costs more than a bar row, and the no-verdict rows are outside the count."
  echo
  echo "History: [\`release_distance/history.tsv\`](../../tasks/artifacts/sv_corpus_grad/release_distance/history.tsv) (one row per release, appended by \`--record\` — and one more, under the shipped release's label, when a commit outside a release moves the distance; the rate above divides by distinct releases; the last row must equal this tree, checked by \`--check\`)."
  echo "$END"
}

record() {  # $1 = tree|<commit>, $2 = release label, $3 = iso date (optional; defaults to commit date)
  local at="$1" release="$2" when="${3:-}" commit
  if [ "$at" = "tree" ]; then commit="$(git rev-parse --short=9 HEAD)"; when="${when:-$(date +%Y-%m-%dT%H:%M:%S)}"; else commit="$(git rev-parse --short=9 "$at")"; when="${when:-$(git show -s --format=%cI "$at" | cut -c1-19)}"; fi
  local -A v; while IFS='=' read -r k val; do v[$k]="$val"; done < <(derive "$at")
  [ -f "$HIST" ] || printf 'date\tcommit\trelease\titems\tlegs_unmet\tbar_rows\tbar_sv_2017\tbar_sv_2023\tbar_verilog_2005\topen_leaves\topen_sv_leaves\topen_engine_leaves\tcorpus_pass_pct\tknown_defects\n' > "$HIST"
  if grep -q "	$commit	" "$HIST"; then echo "history: row for $commit already recorded"; return 0; fi
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$when" "$commit" "$release" "${v[items]}" "${v[legs_unmet]}" "${v[bar_rows]}" "${v[bar_sv_2017]}" "${v[bar_sv_2023]}" "${v[bar_verilog_2005]}" "${v[open_leaves]}" "${v[open_sv_leaves]}" "${v[open_engine_leaves]}" "${v[corpus_pass_pct]}" "${v[known_defects]}" >> "$HIST"
  echo "history: recorded $release at $commit — items=${v[items]}"
}

case "${1:-}" in
  --markdown) markdown_block ;;
  --check)
    page="${2:-$PAGE_DEFAULT}"; [ -f "$page" ] || { echo "SV-RELEASE-DISTANCE: ✗ page missing: $page" >&2; exit 2; }
    have="$(awk -v b="$BEGIN" -v e="$END" '$0==b{f=1} f{print} $0==e{f=0}' "$page")"
    [ -n "$have" ] || { echo "SV-RELEASE-DISTANCE: ✗ $page carries no derived block ($BEGIN)" >&2; exit 2; }
    want="$(markdown_block)"
    if [ "$have" != "$want" ]; then echo "SV-RELEASE-DISTANCE: ✗ $page's block is NOT what the tree derives — re-run --markdown and republish (diff below)" >&2; diff <(printf '%s\n' "$have") <(printf '%s\n' "$want") >&2 || true; exit 2; fi
    # the history's last row must describe this tree
    if [ -f "$HIST" ]; then
      last_items="$(tail -n 1 "$HIST" | cut -f4)"; now_items="$(derive tree | grep -oE '^items=[0-9]+' | cut -d= -f2)"
      [ "$last_items" = "$now_items" ] || { echo "SV-RELEASE-DISTANCE: ✗ the history's last row says items=$last_items but the tree derives $now_items — run --record for this release" >&2; exit 2; }
    fi
    echo "SV-RELEASE-DISTANCE: OK ($(derive tree | grep -oE '^items=[0-9]+'); the published block equals a fresh derivation; the history's last row matches the tree)" ;;
  --record) record tree "${2:-$(git log -1 --format=%s | grep -oE 'release [0-9.]+' | head -1 | cut -d' ' -f2)}" ;;
  --record-at) record "$2" "$3" ;;
  "") summary_line ;;
  *) echo "usage: $0 [--markdown | --check <page> | --record [release] | --record-at <commit> <release>]" >&2; exit 2 ;;
esac
