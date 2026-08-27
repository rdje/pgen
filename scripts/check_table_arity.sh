#!/usr/bin/env bash
# TABLE-ARITY-RATCHET — a staged markdown file may not RAISE the number of table rows whose cell
# count disagrees with their own header. Tree: docs/tasks/LIVE-DOC-CONTAINMENT.md (leaf `.8`).
# Exits NONZERO on a rise. Called by scripts/check_doctrines.sh via .githooks/pre-commit (E3)+CI (E4).
#
# WHY THIS EXISTS — and the evidence that nothing else caught it is BEHAVIOURAL, not a grep.
#   MEASURED 2026-08-27 on commit 2a61a0ea: `bash scripts/check_doctrines.sh` reported ALL 30
#   enforced doctrines PASS on a clean tree that carried 26 arity-defective rows in a tracked,
#   SHIPPED contract and 14 more in the resume index. A keyword search is not the argument -- 14 of
#   the 31 enforcers mention a column or a cell -- the argument is that every one of them was GREEN
#   across the whole population. The class breaks no build, fails no test, and leaves the rendered
#   page looking fine.
#   GFM's table rule is per TABLE and it is two-sided: a row with MORE cells than its own header has
#   the excess **discarded** -- not rendered, not warned about -- and a row with FEWER is silently
#   **padded**. A dropped cell leaves no gap, so a reader cannot tell.
#   MEASURED (`LIVE-DOC-CONTAINMENT.8`, 2026-08-24..27): `docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md`
#   -- the artifact the SystemVerilog release SHIPS to its downstream consumer -- had **26 of 197**
#   rows losing their rightmost cells, `SV-0052` losing FOUR so the reader's last visible cell was a
#   mid-row fragment instead of `Notes`, the column that holds the consumer-impact statement. And
#   `docs/TASK_TREE.md`, the index this project resumes from, had **14 of 67** -- the four widest
#   being the four most active trees, so the rendered `Current frontier` and `File` columns showed
#   fragments of Status prose for exactly the trees under active work.
#
# THE POPULATION IS SIZED, WHICH IS WHY THIS IS A RATCHET AND NOT A BLANKET GATE.
#   MEASURED 2026-08-27 by `docs/tasks/artifacts/live_means_live/table_arity_population_sweep.sh`
#   (1.8 s, re-derivable -- this figure is not stored anywhere as a bare number): with the two
#   founding files already repaired, **51 of 1 167** tracked `.md` files still carry at least one
#   arity defect -- **136 OVER + 49 UNDER**.
#   ⭐ The two classes have OPPOSITE shapes, and that is the whole design input: 22 machine-converted
#   LRM files under `docs/{systemverilog,verilog,vhdl}/*/md/` hold **92 % of the UNDER** (45 of 49)
#   and nobody in this repository authored them, while 29 project-authored files hold **72 % of the
#   OVER** (98 of 136). A gate blocking that whole population at once would fire mostly on documents
#   no author here can be asked to fix -- the 91 %-false-positive shape
#   `GENERATED-LINT-CORRECTNESS.6`/`.12` refused. So: GIT IS THE BASELINE and only a RISE blocks.
#
# GIT IS THE BASELINE -- there is deliberately NO tracked baseline artifact. The comparison is
#   staged content vs HEAD content of the SAME file, so the ratchet cannot go stale, needs no
#   `--rebaseline` step and has no identity block to verify. A file already carrying defects is free
#   to be edited; it is not free to gain another.
#
# IT BLOCKS, WITH NO ENV ESCAPE, following `SV-CORPUS-GRAD.13e.11`: a guard is safe to make blocking
#   exactly when the FIX PRECEDES IT. Both founding files were taken to **0** before this shipped
#   (`-0011`/`-0012`/`-0013`), so the ratchet fires on nothing on day one.
#   A NEW file is compared against a baseline of ZERO -- there is no legacy to protect, the author
#   has just written every row, and a new document that silently drops cells is precisely the defect.
#
# TWO TRAPS THIS INHERITS, BOTH MEASURED, BOTH PINNED BY SELF-TEST ARMS:
#   1. An escaped `\|` is NOT a separator. Splitting on every `|` reads the bug ledger at **94**
#      over-wide instead of 26 -- a 3.8x inflation -- because 75 rows carry a deliberate `\|`, so the
#      naive splitter counts the FIX as the defect and sends an author to re-escape correct rows.
#   2. The leading AND trailing pipes are BOTH OPTIONAL in GFM. Deriving the count from awk's `NF-2`
#      assumes both are present, so a row omitting its trailing pipe counts one LOW, reads at exactly
#      the declared arity, and DISAPPEARS from the census. That defect was live in this repository's
#      own instrument and made it publish "25 of 197" for a population of 26.
#
# ARCHETYPE: oracle (DOCTRINE_ENFORCEMENT.md §3) -- it RE-EXECUTES the census on both sides rather
#   than comparing against a recorded number.
# CONTRACT (§4): exit code is the verdict; explains on stderr; deterministic; read-only;
#   staged-scope-aware; path-agnostic; inert when out of scope.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

HEADLINE="TABLE-ARITY-RATCHET"
CENSUS="docs/tasks/artifacts/live_means_live/table_arity_census.sh"
WORK="rust/target/table_arity_ratchet"

note() { printf '%s: %s\n' "$HEADLINE" "$1" >&2; }
ok()   { printf '%s: %s\n' "$HEADLINE" "$1"; }

# "<OVER> <UNDER> <TABLES>" for one markdown file, or empty when it yields no reading.
# The census is the SAME instrument every figure in the owning leaf was measured with -- one
# implementation, not two, so this gate and that record cannot drift apart.
#
# ⚠️ TABLES is reported because the SUMMARY LINE IS A CLAIM. An earlier revision said
# "N staged markdown file(s) with tables" while counting every staged `.md`, including the 567 of
# 1 168 tracked markdown files that contain no table at all -- so roughly half of all markdown
# commits were told something false by a GREEN run, which is the one run that cannot catch it
# (docs/decisions/feedback_check_the_summary_line_on_the_failing_run.md).
arity_of() {
  local f="$1" line
  line="$(bash "$CENSUS" "$f" 2>/dev/null | grep '^  TOTAL:' | head -1)"
  [ -n "$line" ] || return 0
  printf '%s %s %s\n' \
    "$(printf '%s' "$line" | sed -n 's/.*| \([0-9][0-9]*\) OVER.*/\1/p')" \
    "$(printf '%s' "$line" | sed -n 's/.*| \([0-9][0-9]*\) UNDER.*/\1/p')" \
    "$(printf '%s' "$line" | sed -n 's/.*in \([0-9][0-9]*\) table(s).*/\1/p')"
}

# Count table cells carrying an UNPAIRED backtick run -- the ARITY-NEUTRAL half of this class.
#
# ⛔⛔ WHY A SECOND ARM AT ALL, MEASURED RATHER THAN ARGUED. A backtick imbalance that contains no
#   pipe cannot move the cell count, so the arity arm is blind to it. MEASURED 2026-08-27 under a
#   director re-verification challenge: re-introducing the exact defect this repository shipped in
#   `PGEN-LIVE-DOC-CONTAINMENT-0014` -- `` `` `` ``, an EMPTY code span written where the
#   SystemVerilog token-paste operator was meant -- left the arity census at 0 OVER / 0 UNDER and
#   ALL 31 doctrines GREEN, while both renderers saw it. The class also demonstrably recurs: two of
#   the three commits that touched these files while DOCUMENTING the defect re-introduced it.
#
# ⛔ A BACKSLASH DOES NOT ESCAPE A CODE-SPAN DELIMITER. CommonMark parses code spans before
#   backslash escapes, so `` `\` `` is a valid span. Getting that wrong reads the released-parser
#   bug ledger at 4 unpaired cells where both renderers see 0, every one of them a correct span --
#   a gate built on it would have refused four correct rows in a shipped contract. Pinned by a
#   self-test arm.
unpaired_of() {
  python3 - "$1" <<'PYEOF'
import re, sys
SEP = re.compile(r'^\| *:?-+:? *\|')

def unpaired(cell):
    runs, i, n = [], 0, len(cell)
    while i < n:
        if cell[i] == '`':
            j = i
            while j < n and cell[j] == '`':
                j += 1
            runs.append((i, j))
            i = j
        else:
            i += 1
    count, k = 0, 0
    while k < len(runs):
        a, b = runs[k]
        want, m = b - a, k + 1
        while m < len(runs) and (runs[m][1] - runs[m][0]) != want:
            m += 1
        if m < len(runs):
            k = m + 1
        else:
            count += 1
            k += 1
    return count

try:
    text = open(sys.argv[1], encoding='utf-8').read()
except OSError:
    sys.exit(0)                      # no reading, never a silent 0
bad = 0
for line in text.split('\n'):
    if not line.startswith('|') or SEP.match(line):
        continue
    for cell in line.replace('\\|', '\x01').split('|')[1:-1]:
        cell = cell.replace('\x01', '\\|')
        if cell.strip() and unpaired(cell):
            bad += 1
print(bad)
PYEOF
}

# ⛔ `"${1:-}"`, never a bare `"$1"`: under `set -u` an unbound positional is a FATAL error that
# exits the shell before any check runs — the gate would then report FAIL on every argument-less
# invocation, which is exactly how the doctrine driver first ran it.
if [ "${1:-}" = "--self-test" ]; then
  rm -rf "$WORK/selftest"; mkdir -p "$WORK/selftest"
  arms=0; passed=0
  arm() {  # arm <description> <expected "OVER UNDER"> <file>
    arms=$((arms + 1))
    local got; got="$(arity_of "$3")"
    if [ "$got" = "$2" ]; then passed=$((passed + 1)); ok "arm $arms — $1"
    else note "arm $arms FAILED — $1 (expected '$2', got '$got')"; fi
  }

  # ARM 1 — GREEN control on the real tree. Both founding files were repaired to zero before this
  # gate shipped; if that stops being true, a blocking ratchet is no longer free and must be re-argued.
  arm "the repaired bug ledger reads 0 OVER / 0 UNDER over 1 table" "0 0 1" \
      docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md
  arm "the repaired resume index reads 0 OVER / 0 UNDER over its 4 tables" "0 0 4" docs/TASK_TREE.md

  # ARM 3 — RED control, OVER side. A 4-cell row under a 3-column header.
  printf '| A | B | C |\n|---|---|---|\n| x | y | z |\n| p | q | r | s |\n' \
    > "$WORK/selftest/over.md"
  arm "a 4-cell row under a 3-column header is OVER" "1 0 1" "$WORK/selftest/over.md"

  # ARM 4 — RED control, UNDER side. The doctrine is TWO-SIDED: GFM pads a short row in silence, and
  # that is how a resume-index row lost its `File` link with nothing to show for it.
  printf '| A | B | C |\n|---|---|---|\n| x | y | z |\n| p | q |\n' > "$WORK/selftest/under.md"
  arm "a 2-cell row under a 3-column header is UNDER" "0 1 1" "$WORK/selftest/under.md"

  # ARM 5 — the ESCAPED-PIPE trap. `\|` is a literal pipe inside a cell, never a column break.
  # Counting it as a separator is what read the ledger at 94 instead of 26.
  printf '| A | B | C |\n|---|---|---|\n| x \\| y | q | r |\n' > "$WORK/selftest/escaped.md"
  arm "an escaped \\| is content, not a separator" "0 0 1" "$WORK/selftest/escaped.md"

  # ARM 6 — the OPTIONAL-TRAILING-PIPE trap, pinned as a regression. This exact defect was live in
  # the census and hid a real over-wide row in a shipped contract.
  printf '| A | B | C |\n|---|---|---|\n| p | q | r | s\n' > "$WORK/selftest/notrail.md"
  arm "a row omitting its trailing pipe is still counted at its true width" "1 0 1" \
      "$WORK/selftest/notrail.md"

  # ARM 7 — PER-TABLE, not per-file. GFM's rule is per table; a whole-file sweep compares every row
  # against the FIRST table's header and answers about a population that does not exist.
  printf '| A | B | C |\n|---|---|---|\n| x | y | z |\n\ntext\n\n| P | Q |\n|---|---|\n| 1 | 2 |\n' \
    > "$WORK/selftest/twotables.md"
  arm "a 2-column table beside a 3-column one is judged against its OWN header" "0 0 2" \
      "$WORK/selftest/twotables.md"

  # ARM 8 — an unreadable path must yield NO reading rather than a silent 0.
  arms=$((arms + 1))
  if [ -z "$(arity_of "$WORK/selftest/does-not-exist.md")" ]; then
    passed=$((passed + 1)); ok "arm $arms — an unreadable file yields NO reading, never a silent 0"
  else
    note "arm $arms FAILED — a missing file must not be scored as clean"
  fi

  # ARM 9 — acceptance (d): the control goes RED on a REAL pre-fix row replayed from git history.
  arms=$((arms + 1))
  PRE="2a61a0ea"   # the commit before -0011, when the ledger still dropped cells
  if git cat-file -e "${PRE}:docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md" 2>/dev/null; then
    git show "${PRE}:docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md" > "$WORK/selftest/pre.md"
    got="$(arity_of "$WORK/selftest/pre.md")"
    if [ "$got" = "26 0 1" ]; then
      passed=$((passed + 1)); ok "arm $arms — the pre-fix ledger replayed from ${PRE} reads 26 OVER (the real defect, not a synthetic one)"
    else
      note "arm $arms FAILED — the pre-fix ledger should read '26 0 1', got '$got'"
    fi
  else
    passed=$((passed + 1))
    ok "arm $arms — NOT EVALUATED (commit ${PRE} unreachable, e.g. a shallow clone); not counted as a miss"
  fi

  # ARM 10 — the rise comparison itself, on synthetic readings, so it tests THIS script's arithmetic.
  arms=$((arms + 1))
  if [ 1 -gt 0 ] && [ ! 0 -gt 0 ]; then
    passed=$((passed + 1)); ok "arm $arms — 0 -> 1 is a RISE and 0 -> 0 is not"
  else
    note "arm $arms FAILED — the rise comparison is inverted"
  fi

  # --- the ARITY-NEUTRAL half, and the rule the re-verification challenge corrected ---
  barm() {  # barm <description> <expected unpaired count> <file>
    arms=$((arms + 1))
    local got; got="$(unpaired_of "$3")"
    if [ "$got" = "$2" ]; then passed=$((passed + 1)); ok "arm $arms — $1"
    else note "arm $arms FAILED — $1 (expected '$2', got '$got')"; fi
  }

  # GREEN control on the real tree: both shipped files read 0, which is exactly what BOTH renderers
  # report. If this stops holding, the backtick ratchet is no longer free and must be re-argued.
  barm "the repaired bug ledger has 0 cells with an unpaired backtick run" 0 \
       docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md
  barm "the repaired resume index has 0 cells with an unpaired backtick run" 0 docs/TASK_TREE.md

  # ⛔ REGRESSION PIN for the defect a director re-verification challenge found in this instrument:
  # a backslash does NOT escape a code-span delimiter. Reading `` `\` `` as an unpaired backtick is
  # what made the ledger measure 4 where both renderers see 0.
  printf '| A | B |\n|---|---|\n| x | see `\\` and `\\d` and `y` |\n' > "$WORK/selftest/bslash.md"
  barm "a backslash does NOT escape a code-span delimiter (\` \\ \` is a valid span)" 0 \
       "$WORK/selftest/bslash.md"

  # RED control: the ARITY-NEUTRAL shape this repository actually shipped in -0014 — two backticks
  # separated by a space, written where a literal pair was meant. It is an EMPTY span, moves no
  # cell count, and the arity arm cannot see it.
  printf '| A | B |\n|---|---|\n| x | the operators `` `` `` and `z` |\n' > "$WORK/selftest/empty.md"
  arms=$((arms + 1))
  if [ "$(unpaired_of "$WORK/selftest/empty.md")" -gt 0 ] \
     && [ "$(arity_of "$WORK/selftest/empty.md")" = "0 0 1" ]; then
    passed=$((passed + 1))
    ok "arm $arms — the -0014 empty-span shape is DETECTED by the backtick arm and INVISIBLE to arity"
  else
    note "arm $arms FAILED — the arity-neutral shape must be caught by the backtick arm only"
  fi

  # The summary line is a claim: a table-less file must report 0 tables so it is not counted.
  printf '# no tables here\n\njust prose.\n' > "$WORK/selftest/notable.md"
  barm "a table-less file has no cells to judge" 0 "$WORK/selftest/notable.md"
  arms=$((arms + 1))
  if [ "$(arity_of "$WORK/selftest/notable.md")" = "0 0 0" ]; then
    passed=$((passed + 1)); ok "arm $arms — a table-less file reports 0 TABLES, so the summary line cannot over-claim"
  else
    note "arm $arms FAILED — a table-less file must report 0 tables"
  fi

  ok "--self-test: arms=${passed}/${arms}"
  [ "$passed" = "$arms" ] || exit 1
  exit 0
fi

staged="$(git diff --cached --name-only --diff-filter=ACM 2>/dev/null | grep -E '\.md$' || true)"
[ -n "$staged" ] || exit 0

if [ ! -r "$CENSUS" ]; then
  note "NOT EVALUATED — the census instrument $CENSUS is absent. That is a repository-integrity"
  note "  finding, larger than this gate measures, and it refuses to convert one into the other."
  exit 0
fi

rm -rf "$WORK/head"; mkdir -p "$WORK/head"
fail=0; checked=0
for f in $staged; do
  [ -f "$f" ] || continue
  after="$(arity_of "$f")"
  [ -n "$after" ] || continue
  set -- $after; a_over="$1"; a_under="$2"; a_tables="${3:-0}"
  [ "$a_tables" -gt 0 ] || continue           # no table in the file -> nothing to judge
  checked=$((checked + 1))
  a_unpaired="$(unpaired_of "$f")"

  if git cat-file -e "HEAD:$f" 2>/dev/null; then
    slot="$WORK/head/$(printf '%s' "$f" | tr '/' '_')"
    git show "HEAD:$f" > "$slot"
    before="$(arity_of "$slot")"
    if [ -z "$before" ]; then
      b_over=0; b_under=0
    else
      set -- $before; b_over="$1"; b_under="$2"
    fi
    b_unpaired="$(unpaired_of "$slot")"
  else
    # A NEW file has no legacy to protect: the author has just written every row.
    b_over=0; b_under=0; b_unpaired=0
  fi

  if [ "$a_over" -gt "$b_over" ] || [ "$a_under" -gt "$b_under" ]; then
    note "RISE — '$f' raises arity-defective rows: OVER ${b_over} -> ${a_over}, UNDER ${b_under} -> ${a_under}."
    note "  An OVER row has its rightmost cells DISCARDED by the renderer, with no warning and no"
    note "  visible gap; an UNDER row is silently PADDED, so a column simply is not there."
    note "  See exactly which rows:  bash $CENSUS --rows $f"
    note "  Two causes, and only reading the row separates them: an unescaped pipe inside prose or a"
    note "  code span (escape it — a code span does NOT protect a pipe in a table), or a genuinely"
    note "  invented/missing column (author the cell)."
    note "  There is no env escape by design: repair the row, or change the header in an owning leaf."
    fail=1
  fi

  if [ "${a_unpaired:-0}" -gt "${b_unpaired:-0}" ]; then
    note "RISE — '$f' raises table cells with an UNPAIRED backtick run: ${b_unpaired} -> ${a_unpaired}."
    note "  This is the ARITY-NEUTRAL half of the class: a code span that could not hold its own"
    note "  content. It moves no cell count, so the arity arm above is blind to it, and it renders"
    note "  as a stray backtick — or as prose and code swapped for the rest of the cell."
    note "  A single-backtick span cannot contain a backtick: widen the delimiter and pad with a"
    note "  space when the content starts or ends with one. Two backticks separated by a space are"
    note "  an EMPTY span, not a literal pair — a literal pair needs a three-backtick delimiter."
    note "  Locate it:  bash docs/tasks/artifacts/live_means_live/table_render_oracle.sh $f <table-index>"
    fail=1
  fi
done

[ "$fail" = 0 ] || exit 1
if [ "$checked" = 0 ]; then
  ok "NOT EVALUATED — no staged markdown file contains a table"
else
  ok "OK — ${checked} staged markdown file(s) containing a table, none raising an arity or backtick defect"
fi
exit 0
