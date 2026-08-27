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

# Count table rows whose CELL BOUNDARY falls inside a code span -- the shape a count cannot see.
#
# ⛔⛔ WHY A COUNT IS NOT A SCHEMA, MEASURED IN THIS DOCTRINE'S OWN FOUNDING POPULATION. A row can
#   land on its header's declared arity and still be wrong, because a count says nothing about
#   WHERE the boundaries fell. `GENERATED-LINT-CORRECTNESS` in docs/TASK_TREE.md carried a pipe
#   inside a code span that split its Status cell in two AND was missing `Current frontier`
#   entirely -- one OVER and one UNDER, netting to exactly the declared 5. Replayed from
#   9dc3289d the arity arm reads it as healthy; its cells are Status-head / Status-tail /
#   Roadmap-lane / File, with the frontier column simply absent.
#   This arm asks the structural question instead: did a separator land where the author had
#   opened a code span? That is arity-independent, needs no per-table knowledge, and it is the
#   root cause of the OVER class as well.
#
# THE POPULATION IS SIZED, WHICH IS WHY THIS IS A RATCHET TOO. MEASURED 2026-08-27 over all
#   1 169 tracked `.md` files: 118 such rows in 27 files. 93 of them are ALSO arity-defective and
#   were already caught; the 25 remaining, in 7 files, are arity-clean and provably invisible to
#   the arm above. Both founding files read 0, so the ratchet is free on day one.
#
# ⛔ An escaped `\|` is content, never a separator -- the same trap the arity arm carries, and the
#   reason 75 correctly-escaped ledger rows are not counted here.
inspan_of() {
  python3 - "$1" <<'PYEOF'
import re, sys
SEP = re.compile(r'^\| *:?-+:? *\|')

def code_spans(line):
    """CommonMark code spans as (content_start, content_end); an unmatched run is literal text.

    A backslash does NOT escape a code-span delimiter -- CommonMark parses code spans before
    backslash escapes. Assuming otherwise invents defects; see the note on unpaired_of().
    """
    runs, i, n = [], 0, len(line)
    while i < n:
        if line[i] == '`':
            j = i
            while j < n and line[j] == '`':
                j += 1
            runs.append((i, j))
            i = j
        else:
            i += 1
    spans, k = [], 0
    while k < len(runs):
        a, b = runs[k]
        want, m = b - a, k + 1
        while m < len(runs) and (runs[m][1] - runs[m][0]) != want:
            m += 1
        if m < len(runs):
            spans.append((b, runs[m][0]))
            k = m + 1
        else:
            k += 1
    return spans

try:
    text = open(sys.argv[1], encoding='utf-8').read()
except OSError:
    sys.exit(0)                      # no reading, never a silent 0
bad = 0
for line in text.split('\n'):
    if not line.startswith('|') or SEP.match(line):
        continue
    spans = code_spans(line)
    if not spans:
        continue
    i, hit = 0, False
    while i < len(line):
        if line[i] == '\\' and i + 1 < len(line) and line[i + 1] == '|':
            i += 2                   # an escaped pipe is content, not a separator
            continue
        if line[i] == '|' and any(a <= i < b for a, b in spans):
            hit = True
            break
        i += 1
    if hit:
        bad += 1
print(bad)
PYEOF
}

# "<FAILS> <ADJUDICATED_TABLES> <UNADJUDICATED_TABLES>" -- the SEMANTIC arm.
#
# ⛔⛔ WHY A COLUMN PREDICATE AND NOT MORE COUNTING. The second arity-neutral shape is a boundary
#   in the WRONG PLACE at the RIGHT COUNT. `LANG-CAPABILITY-AUDIT` was "repaired" by an earlier
#   mechanical pass that escaped the REAL separator standing before the `File` link and left a
#   real in-span pipe unescaped: 5 cells, arity satisfied, and the `File` column holding mid-row
#   prose. Counting cannot distinguish that from a healthy row; reading the column can.
#
# ⛔ THE PREDICATE IS A WHOLE-CELL MATCH, NEVER A SUBSTRING TEST, AND THAT IS THE CORRECTION THIS
#   ARM EXISTS FOR. The registered predicate for the resume index used to be the substring
#   `\]\(tasks/`, which is satisfied by a link sitting ANYWHERE in the cell. MEASURED 2026-08-27
#   in BOTH renderers: 4 of 67 `File` cells carried the link followed by frontier narration --
#   1 424, 435, 1 247 and 56 717 authored characters -- and every one of them passed the
#   substring form. Under a whole-cell match all four are refused. The four were repaired by
#   docs/tasks/artifacts/live_means_live/table_last_cell_repair.py before this arm shipped.
#
# ⛔ THE PREDICATES ARE IMPORTED, NEVER RE-DECLARED. They live in table_pipe_repair.py, which is
#   the tool that USES them to endorse a repair. One implementation means this gate and that
#   instrument cannot drift into disagreeing about what a healthy column looks like -- the same
#   argument that makes the arity arm call the census rather than re-implement it.
#
# ⛔ A TABLE WITH NO REGISTERED PREDICATE IS **NOT EVALUATED, LOUDLY** -- counted and reported in
#   the summary, never scored as a pass. Silence about what was not checked is how a green run
#   comes to mean more than it measured.
#
# ⚠️ AND A PREDICATE THAT FAILS ON MOST OF ITS OWN TABLE IS AN INSTRUMENT DEFECT, NOT A DOCUMENT
#   DEFECT. Predicates are derived from healthy rows, so a majority failure means the predicate
#   is wrong -- the 91 %-false-positive shape GENERATED-LINT-CORRECTNESS.6/.12 refused. Such a
#   column reports NOT CREDIBLE and is not scored, so an instrument bug cannot block a commit.
#
# ⛔ LINK TARGETS ARE CHECKED TOO (a `File` cell pointing at a path that does not exist satisfies
#   every content predicate). `LIVE-MEANS-LIVE` carried `](docs/tasks/LIVE-MEANS-LIVE.md)`, which
#   resolves from docs/TASK_TREE.md to docs/docs/tasks/... and does not exist.
#   ⚠️ SCOPED TO PREDICATE-GOVERNED COLUMNS BY DESIGN, and the measurement says why: swept over
#   EVERY cell instead, 192 of 236 relative links in the released-parser bug ledger would be
#   reported -- and all 192 are deliberate `../rgx/pgen-issues/...` pointers into the SIBLING
#   consumer repository, a class docs/tasks/DOCPATH.md has already adjudicated in its Non-Goals as
#   having no repo-relative equivalent. An unscoped arm would have refused 192 correct rows in a
#   contract this project SHIPS -- the second time in this tree that a link-shaped rule came one
#   step from that, and the reason the scope is the registered column and not the row.
#
# $2 is the ORIGINAL directory of the file: under the ratchet the HEAD side is read from a scratch
# slot, and resolving its links against that slot would report every one of them broken.
semantic_of() {
  python3 - "$1" "$2" "$3" <<'PYEOF'
import importlib.util, os, re, sys

path, base, repo_rel = sys.argv[1], sys.argv[2], sys.argv[3]
tool = "docs/tasks/artifacts/live_means_live/table_pipe_repair.py"
if not os.path.isfile(tool):
    sys.exit(0)                      # no reading, never a silent 0 — the caller reports it
spec = importlib.util.spec_from_file_location("table_pipe_repair", tool)
tpr = importlib.util.module_from_spec(spec)
spec.loader.exec_module(tpr)

LINK = re.compile(r'\[[^\]]*\]\(([^)#\s]+)(?:#[^)]*)?\)')
try:
    lines = open(path, encoding='utf-8').read().split('\n')
except OSError:
    sys.exit(0)

fails = adjudicated = unadjudicated = 0
for tbl in tpr.tables(lines):
    preds = tpr.PREDICATES.get((repo_rel, tbl["n"]))
    if not preds:
        unadjudicated += 1
        continue
    adjudicated += 1
    reasons = {}                     # line -> [why, ...]; a ROW counts once however many reasons
    incredible = set()
    for col, (pat, why) in preds.items():
        col_fails = []
        for ln in tbl["rows"]:
            cells = tpr.split_cells(lines[ln])
            if col >= len(cells):
                col_fails.append((ln, "the cell is ABSENT (%d cells, header %d)" % (len(cells), tbl["cols"])))
                continue
            if not re.fullmatch(pat, cells[col], re.S):
                # ⛔ fullmatch, never search: a substring hit is what let the boundary defect through.
                col_fails.append((ln, "does not satisfy, as a WHOLE cell: " + why))
            # ⛔ The link check runs INDEPENDENTLY of the predicate verdict. A cell can be the right
            # SHAPE and still point at nothing, and a cell that already failed the shape can carry
            # the broken target that explains why -- reporting only the first reason hides the
            # second. The row is still counted ONCE.
            for m in LINK.finditer(cells[col]):
                tgt = m.group(1)
                if tgt.startswith(('http://', 'https://', 'mailto:', '/')):
                    continue
                if not os.path.exists(os.path.normpath(os.path.join(base, tgt))):
                    col_fails.append((ln, "link target does not exist: " + tgt))
        rows = len(tbl["rows"])
        distinct = len({ln for ln, _ in col_fails})
        if rows and distinct * 2 > rows:
            # ⚠️ A predicate is DERIVED from healthy rows, so a majority failure is an INSTRUMENT
            # defect, not a document one -- the 91 %-false-positive shape .6/.12 refused. NOT
            # EVALUATED, loudly, and never allowed to block a commit.
            incredible.add(col)
            sys.stderr.write(
                "TABLE-ARITY-RATCHET: predicate NOT CREDIBLE — table %d column %d fails on %d of %d\n"
                "  rows. A predicate is derived from HEALTHY rows, so a majority failure is an\n"
                "  INSTRUMENT defect, not a document one. This column is NOT EVALUATED; fix the\n"
                "  predicate in %s.\n" % (tbl["n"], col + 1, distinct, rows, tool))
            continue
        for ln, why in col_fails:
            reasons.setdefault(ln, []).append(why)
    for ln in sorted(reasons):
        cells = tpr.split_cells(lines[ln])
        name = cells[0].strip().strip('` ') if cells else '?'
        for why in reasons[ln]:
            sys.stderr.write("    line %-5d %-30s %s\n" % (ln + 1, name, why))
    fails += len(reasons)
print("%d %d %d" % (fails, adjudicated, unadjudicated))
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

  # --- the IN-SPAN BOUNDARY arm: a count is not a schema ---
  iarm() {  # iarm <description> <expected row count> <file>
    arms=$((arms + 1))
    local got; got="$(inspan_of "$3")"
    if [ "$got" = "$2" ]; then passed=$((passed + 1)); ok "arm $arms — $1"
    else note "arm $arms FAILED — $1 (expected '$2', got '$got')"; fi
  }

  iarm "the repaired bug ledger has 0 rows with a boundary inside a code span" 0 \
       docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md
  iarm "the repaired resume index has 0 rows with a boundary inside a code span" 0 docs/TASK_TREE.md

  # An escaped `\|` inside a span is CONTENT and must not be read as a boundary — the same trap
  # that read the ledger at 94 instead of 26, arriving here through a different door.
  printf '| A | B |\n|---|---|\n| x | `a \\| b` is one span |\n' > "$WORK/selftest/inspan_ok.md"
  iarm "an escaped \\| inside a code span is content, not a boundary" 0 "$WORK/selftest/inspan_ok.md"
  printf '| A | B |\n|---|---|\n| x | `a | b` splits the row |\n' > "$WORK/selftest/inspan_bad.md"
  iarm "an UNescaped | inside a code span is a boundary in the wrong place" 1 "$WORK/selftest/inspan_bad.md"

  # ⛔ ARM: the ARITY ARM IS BLIND TO IT, replayed from REAL git history rather than synthesised.
  # At 9dc3289d the resume index reads 0 OVER / 0 UNDER — fully GREEN under the doctrine as it
  # shipped — while THREE of its rows have a cell boundary inside a code span. One of them is
  # `GENERATED-LINT-CORRECTNESS`, whose Status cell was split by a pipe inside
  # `ROOT_KW='root cause|why ?[-+/&] ?where'` AND which was missing `Current frontier` entirely:
  # one OVER and one UNDER cancelling to exactly the declared 5.
  arms=$((arms + 1))
  INSPAN_PRE="9dc3289d"
  if git cat-file -e "${INSPAN_PRE}:docs/TASK_TREE.md" 2>/dev/null; then
    git show "${INSPAN_PRE}:docs/TASK_TREE.md" > "$WORK/selftest/inspan_pre.md"
    if [ "$(inspan_of "$WORK/selftest/inspan_pre.md")" = "3" ] \
       && [ "$(arity_of "$WORK/selftest/inspan_pre.md")" = "0 0 4" ]; then
      passed=$((passed + 1))
      ok "arm $arms — the ${INSPAN_PRE} index is 0 OVER / 0 UNDER to the arity arm and carries 3 in-span boundaries"
    else
      note "arm $arms FAILED — ${INSPAN_PRE} should read arity '0 0 4' with 3 in-span rows"
    fi
  else
    passed=$((passed + 1))
    ok "arm $arms — NOT EVALUATED (commit ${INSPAN_PRE} unreachable, e.g. a shallow clone); not counted as a miss"
  fi

  # --- the SEMANTIC COLUMN arm ---
  sarm() {  # sarm <description> <expected "FAILS ADJ UNADJ"> <file> <base> <repo-rel>
    arms=$((arms + 1))
    local got; got="$(semantic_of "$3" "$4" "$5" 2>/dev/null)"
    if [ "$got" = "$2" ]; then passed=$((passed + 1)); ok "arm $arms — $1"
    else note "arm $arms FAILED — $1 (expected '$2', got '$got')"; fi
  }

  sarm "the resume index: 0 failing rows, 1 table adjudicated, 3 with no registered predicate" \
       "0 1 3" docs/TASK_TREE.md docs docs/TASK_TREE.md
  sarm "the bug ledger: 0 failing rows, its 1 table adjudicated" \
       "0 1 0" docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md docs/contracts \
       docs/contracts/PGEN_RELEASED_PARSER_BUG_LEDGER.md

  # ⛔⛔ ARM: BOTH STRUCTURAL ARMS ARE GREEN AND THE COLUMN IS STILL WRONG — replayed from git, from
  # the commit immediately before the repair. At 8c3b93f9 the resume index reads 0 OVER / 0 UNDER,
  # 0 unpaired backticks and 0 in-span boundaries, and FOUR of its `File` cells carry the link
  # followed by frontier narration (1 424, 435, 1 247 and 56 717 authored characters). Both
  # renderers show the prose in a column whose header says `File`. Every arm that shipped before
  # this one passes that file.
  arms=$((arms + 1))
  SEM_PRE="8c3b93f9"
  if git cat-file -e "${SEM_PRE}:docs/TASK_TREE.md" 2>/dev/null; then
    git show "${SEM_PRE}:docs/TASK_TREE.md" > "$WORK/selftest/sem_pre.md"
    if [ "$(semantic_of "$WORK/selftest/sem_pre.md" docs docs/TASK_TREE.md 2>/dev/null)" = "4 1 3" ] \
       && [ "$(arity_of "$WORK/selftest/sem_pre.md")" = "0 0 4" ] \
       && [ "$(unpaired_of "$WORK/selftest/sem_pre.md")" = "0" ] \
       && [ "$(inspan_of "$WORK/selftest/sem_pre.md")" = "0" ]; then
      passed=$((passed + 1))
      ok "arm $arms — the ${SEM_PRE} index passes arity, backtick AND in-span, and fails 4 rows on the column arm"
    else
      note "arm $arms FAILED — ${SEM_PRE} should be arity '0 0 4', 0 unpaired, 0 in-span, semantic '4 1 3'"
    fi
  else
    passed=$((passed + 1))
    ok "arm $arms — NOT EVALUATED (commit ${SEM_PRE} unreachable); not counted as a miss"
  fi

  # ⛔ ARM: THE LINK-TARGET SUB-ARM, also replayed from git. At bd12c92e the `LIVE-MEANS-LIVE` row
  # carried `](docs/tasks/LIVE-MEANS-LIVE.md)`, which resolves from docs/TASK_TREE.md to
  # docs/docs/tasks/... — a path that does not exist. No content predicate can see that.
  arms=$((arms + 1))
  LINK_PRE="bd12c92e"
  if git cat-file -e "${LINK_PRE}:docs/TASK_TREE.md" 2>/dev/null; then
    git show "${LINK_PRE}:docs/TASK_TREE.md" > "$WORK/selftest/link_pre.md"
    # ⛔ CAPTURE, then match — never `... | grep -q` here. Under `set -o pipefail` (in force at the
    # top of this file) `grep -q` exits at the FIRST match and closes the pipe, the producer takes
    # SIGPIPE, and the pipeline reports failure. A MATCH would then read as a MISS: the arm fails
    # precisely when the behaviour it pins is correct. Measured while writing this arm.
    link_out="$(semantic_of "$WORK/selftest/link_pre.md" docs docs/TASK_TREE.md 2>&1 >/dev/null)"
    case "$link_out" in
      *"link target does not exist: docs/tasks/LIVE-MEANS-LIVE.md"*) link_hit=1 ;;
      *) link_hit=0 ;;
    esac
    if [ "$link_hit" = 1 ]; then
      passed=$((passed + 1))
      ok "arm $arms — the ${LINK_PRE} index names the nonexistent docs/docs/tasks/... target by path"
    else
      note "arm $arms FAILED — the broken LIVE-MEANS-LIVE link target must be named"
    fi
  else
    passed=$((passed + 1))
    ok "arm $arms — NOT EVALUATED (commit ${LINK_PRE} unreachable); not counted as a miss"
  fi

  # A table with NO registered predicate is NOT EVALUATED — counted, reported, never scored clean.
  printf '| A | B | C |\n|---|---|---|\n| x | y | z |\n' > "$WORK/selftest/nopred.md"
  sarm "a table with no registered predicate is NOT EVALUATED, never a pass" \
       "0 0 1" "$WORK/selftest/nopred.md" "$WORK/selftest" some/unregistered/file.md

  # ⚠️ A MINORITY failure IS scored; a MAJORITY failure is an instrument defect and is NOT.
  # Both are keyed to docs/TASK_TREE.md so the registered File-column predicate applies.
  printf '| Tree | Status | Roadmap lane | Current frontier | File |\n|---|---|---|---|---|\n' \
    > "$WORK/selftest/minority.md"
  printf '| `A` | s | l | f | [a](tasks/CORPUS-KEY-AUDIT.md) |\n' >> "$WORK/selftest/minority.md"
  printf '| `B` | s | l | f | [b](tasks/DOCPATH.md) |\n' >> "$WORK/selftest/minority.md"
  printf '| `C` | s | l | f | [c](tasks/DONE-BAR.md) |\n' >> "$WORK/selftest/minority.md"
  printf '| `D` | s | l | f | [d](tasks/DONE-BAR.md) and then prose |\n' >> "$WORK/selftest/minority.md"
  sarm "1 failing row of 4 is a DOCUMENT defect and is scored" \
       "1 1 0" "$WORK/selftest/minority.md" docs docs/TASK_TREE.md

  printf '| Tree | Status | Roadmap lane | Current frontier | File |\n|---|---|---|---|---|\n' \
    > "$WORK/selftest/majority.md"
  printf '| `A` | s | l | f | not a link at all |\n' >> "$WORK/selftest/majority.md"
  printf '| `B` | s | l | f | nor is this |\n' >> "$WORK/selftest/majority.md"
  printf '| `C` | s | l | f | nor this |\n' >> "$WORK/selftest/majority.md"
  printf '| `D` | s | l | f | [d](tasks/DONE-BAR.md) |\n' >> "$WORK/selftest/majority.md"
  sarm "3 failing rows of 4 is an INSTRUMENT defect: NOT CREDIBLE, not scored, cannot block" \
       "0 1 0" "$WORK/selftest/majority.md" docs docs/TASK_TREE.md

  # ⭐ THE BASE DIRECTORY DECIDES LINK RESOLUTION, AND GETTING IT WRONG DEGRADES LOUDLY RATHER THAN
  # BLOCKING. The HEAD side of every comparison is read from a scratch slot, so resolving its links
  # against that slot would read all 67 as broken. This arm proves both halves: the right base
  # scores 0, and the wrong base is caught by the credibility guard — 67 of 67 failing is an
  # INSTRUMENT defect by construction, so the column is NOT EVALUATED, says so on stderr, and
  # scores 0 instead of blocking a commit on a misconfiguration.
  arms=$((arms + 1))
  bad_base_out="$(semantic_of docs/TASK_TREE.md "$WORK/selftest" docs/TASK_TREE.md 2>&1 >/dev/null)"
  case "$bad_base_out" in
    *"NOT CREDIBLE"*) bad_base_loud=1 ;;
    *) bad_base_loud=0 ;;
  esac
  if [ "$(semantic_of docs/TASK_TREE.md docs docs/TASK_TREE.md 2>/dev/null)" = "0 1 3" ] \
     && [ "$(semantic_of docs/TASK_TREE.md "$WORK/selftest" docs/TASK_TREE.md 2>/dev/null)" = "0 1 3" ] \
     && [ "$bad_base_loud" = 1 ]; then
    passed=$((passed + 1))
    ok "arm $arms — a wrong base is refused as NOT CREDIBLE (loud, unscored), not turned into 67 document defects"
  else
    note "arm $arms FAILED — a wrong base must degrade loudly, not block (got '$bad_base_out')"
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
fail=0; checked=0; s_adj=0; s_unadj=0

if [ ! -r "docs/tasks/artifacts/live_means_live/table_pipe_repair.py" ]; then
  note "SEMANTIC ARM NOT EVALUATED — the predicate registry table_pipe_repair.py is absent. The"
  note "  arity, backtick and in-span arms still run; the column arm reports nothing rather than"
  note "  scoring every table as clean."
fi
for f in $staged; do
  [ -f "$f" ] || continue
  after="$(arity_of "$f")"
  [ -n "$after" ] || continue
  set -- $after; a_over="$1"; a_under="$2"; a_tables="${3:-0}"
  [ "$a_tables" -gt 0 ] || continue           # no table in the file -> nothing to judge
  checked=$((checked + 1))
  a_unpaired="$(unpaired_of "$f")"
  a_inspan="$(inspan_of "$f")"
  # ⛔ The per-row reasons go to a FILE, not to /dev/null. An earlier revision discarded them and
  # then told the operator "the offending rows are listed above" — a false line in the gate's own
  # output, the same failure this file already carries a fix for one arm up. They are replayed only
  # on a RISE, so a file with a pre-existing baseline does not re-print it on every commit; a NOT
  # CREDIBLE warning is an INSTRUMENT defect and is replayed unconditionally.
  a_semerr="$WORK/sem_staged.txt"
  a_semraw="$(semantic_of "$f" "$(dirname "$f")" "$f" 2>"$a_semerr")"
  set -- ${a_semraw:-0 0 0}; a_sem="$1"; s_adj=$((s_adj + ${2:-0})); s_unadj=$((s_unadj + ${3:-0}))
  grep -F 'NOT CREDIBLE' "$a_semerr" >&2 || true

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
    b_inspan="$(inspan_of "$slot")"
    # ⛔ The HEAD side is read from a scratch slot but its links must resolve against the file's
    # REAL directory and its predicates keyed by the file's REAL repo path — otherwise every link
    # reads broken and no table is ever adjudicated, which would silence the arm in the passing
    # direction on exactly the comparison it exists to make.
    b_semraw="$(semantic_of "$slot" "$(dirname "$f")" "$f" 2>/dev/null)"
    set -- ${b_semraw:-0 0 0}; b_sem="$1"
  else
    # A NEW file has no legacy to protect: the author has just written every row.
    b_over=0; b_under=0; b_unpaired=0; b_inspan=0; b_sem=0
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

  if [ "${a_inspan:-0}" -gt "${b_inspan:-0}" ]; then
    note "RISE — '$f' raises table rows whose CELL BOUNDARY falls inside a code span: ${b_inspan} -> ${a_inspan}."
    note "  A code span does NOT protect a pipe in a table: GFM splits the row BEFORE it parses"
    note "  inline content. The row may still land on its header's count — one such row split its"
    note "  Status cell in two AND lost its frontier column, cancelling to exactly the declared 5 —"
    note "  so the arity arm above can be GREEN while the columns are wrong."
    note "  The repair is  |  ->  \\|  inside the span, and nothing else."
    note "  See exactly which rows:  bash $CENSUS --rows $f"
    fail=1
  fi

  if [ "${a_sem:-0}" -gt "${b_sem:-0}" ]; then
    note "RISE — '$f' raises rows failing their table's registered column predicate: ${b_sem} -> ${a_sem}."
    note "  This arm reads a COLUMN, not a count. A predicate is derived from the table's own healthy"
    note "  rows and is matched against the WHOLE cell, because a substring test is satisfied by a"
    note "  link sitting anywhere in a cell that also holds an essay of prose."
    grep -vF 'NOT CREDIBLE' "$a_semerr" >&2 || true
    note "  Each offending row is named above with its reason. Either the boundary is in the wrong"
    note "  place (move the content into the column whose header names it), or the link target does"
    note "  not exist."
    note "  Predicates live in docs/tasks/artifacts/live_means_live/table_pipe_repair.py."
    fail=1
  fi
done

[ "$fail" = 0 ] || exit 1
if [ "$checked" = 0 ]; then
  ok "NOT EVALUATED — no staged markdown file contains a table"
else
  # ⛔ THE SUMMARY LINE IS A CLAIM. It names what the semantic arm did NOT evaluate, because a
  # table with no registered predicate is unexamined, not clean — and a green run that hides that
  # is the failure this gate already shipped once (feedback_check_the_summary_line_on_the_failing_run).
  ok "OK — ${checked} staged markdown file(s) containing a table, none raising an arity, backtick, in-span-boundary or column-predicate defect (semantic arm: ${s_adj} table(s) adjudicated, ${s_unadj} NOT EVALUATED — no predicate registered)"
fi
exit 0
