#!/usr/bin/env bash
# docs/tasks/artifacts/sv_corpus_grad/outcome_staleness_census/census.sh
#
# SV-CORPUS-GRAD.13e.6(c) — HOW OFTEN has the tracked SV corpus report been STALE against the
# grammar, measured over the whole history rather than guessed.
#
# The question: at every commit that changed `grammars/systemverilog.ebnf`, did the TRACKED report
# `stimuli/sv/characterization/characterization.md` at THAT SAME COMMIT record a grammar digest
# equal to the grammar it sat beside? A `stale` row is a commit where the published corpus numbers
# described a grammar that no longer existed in the tree — the exact state `.13e.5` caught by
# accident (three days, one hidden fix, bar 275 -> 274).
#
# ⛔ Everything is read out of GIT OBJECTS (`git show <rev>:<path>`), never the working tree, so the
# census is a pure function of history and re-derives identically on any clone. The report's
# recorded digest is the `| grammar | ... | `<sha>` |` row of its instrument-identity table; the
# grammar's real digest is sha256 over the blob at that revision. A revision whose report carries no
# identity table (the block was added mid-history) is `no_identity`, counted and never scored.
#
# Output: one TSV row per grammar revision (rev, date, verdict, recorded, actual) on stdout, then a
# `OUTCOME-STALENESS:` headline on stderr. ⭐ Both the population and the counts are DERIVED here;
# nothing is stored. Deterministic, ~5 s over ~300 revisions.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"; cd "$ROOT" || exit 2

GRAMMAR=grammars/systemverilog.ebnf
REPORT=stimuli/sv/characterization/characterization.md
RANGE="${1:-HEAD}"

sha() { if command -v shasum >/dev/null 2>&1; then shasum -a 256 | cut -d' ' -f1; else sha256sum | cut -d' ' -f1; fi; }

fresh=0; stale=0; no_report=0; no_identity=0; total=0
first_stale=""; stale_runs=0; prev_stale=0
printf 'rev\tdate\tverdict\trecorded_grammar_sha\tactual_grammar_sha\treport_touched_same_commit\n'
while read -r rev date; do
  total=$((total+1))
  actual="$(git show "$rev:$GRAMMAR" 2>/dev/null | sha)"
  if ! git cat-file -e "$rev:$REPORT" 2>/dev/null; then
    no_report=$((no_report+1)); verdict=no_report; recorded="-"
  else
    recorded="$(git show "$rev:$REPORT" | awk -F'`' '/^\| grammar \|/{print $4; exit}')"
    if [ -z "$recorded" ]; then
      no_identity=$((no_identity+1)); verdict=no_identity; recorded="-"
    elif [ "$recorded" = "$actual" ]; then
      fresh=$((fresh+1)); verdict=fresh
    else
      stale=$((stale+1)); verdict=stale; [ -z "$first_stale" ] && first_stale="$rev ($date)"
    fi
  fi
  # a stale RUN = consecutive grammar revisions (newest->oldest order here) that are all stale
  if [ "$verdict" = stale ]; then [ "$prev_stale" -eq 0 ] && stale_runs=$((stale_runs+1)); prev_stale=1; else prev_stale=0; fi
  touched=no; git diff-tree --no-commit-id --name-only -r "$rev" -- "$REPORT" 2>/dev/null | grep -q . && touched=yes
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$rev" "$date" "$verdict" "$recorded" "$actual" "$touched"
done < <(git log --format='%h %ad' --date=short "$RANGE" -- "$GRAMMAR")

scored=$((fresh+stale))
printf 'OUTCOME-STALENESS: grammar_revisions=%d scored=%d fresh=%d stale=%d stale_runs=%d no_report=%d no_identity=%d newest_stale=%s\n' \
  "$total" "$scored" "$fresh" "$stale" "$stale_runs" "$no_report" "$no_identity" "${first_stale:-none}" >&2
