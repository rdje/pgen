#!/usr/bin/env bash
# GAP-CLAIM-CENSUS — a task leaf that ADDS a "nothing checks X" claim must record the CENSUS it
# rests on. Tree: docs/tasks/DOCTRINE-GAP-OWNERSHIP.md (leaf `.15`). Exits NONZERO on breach.
# Called by scripts/check_doctrines.sh via .githooks/pre-commit (E3) and CI (E4).
#
# ⭐⭐ WHY THIS EXISTS — the defect is MINE, it is a HABIT rather than a slip, and it repeated
#   three times inside three days, twice reaching a commit:
#     1. `-0288`  "the `verilog_2005` profile ... only the first is gated"     -> retracted by `-0289`
#     2. `-0290`  "**no instrument in the tree crosses the two**"              -> leaf CLOSED AS
#                 REFUTED by `-0294`: `gather_verified_profile_proof_covered_rules` crosses them and
#                 has since `VERILOG-2005-PROFILE.6.7`; all 323 "unreachable" rules were already
#                 certified, an exact 323-of-323 set match.
#     3. `ENGINE-UNIVERSAL-SERVICES.52` first draft, caught pre-publication 2026-08-26.
#   ⛔ An "X is checked by NOTHING" sentence is not a description — it is a UNIVERSALLY QUANTIFIED
#   CLAIM over the whole tree, i.e. a CENSUS claim, and it is false the moment one reader exists.
#   [[an-x-is-checked-by-nothing-claim-is-a-census-claim]].
#
# ⭐ AND THE CENSUS IS NOT MERELY DEFENSIVE — IT LOCATES THE DEFECT. In occurrence 3 the false claim
#   would have sent a fix at a mechanism that does not exist; running the census instead produced the
#   exact predicate (`needs_rule_body_descent` keying on `o{i}`/`q` segments and so missing an `s{i}`
#   hop) that `.52` then fixed. A vague suspicion became a located defect. That is the argument for
#   blocking rather than warning.
#
# ⛔⛔ STAGED-DIFF-SCOPED, NEVER OVER THE EXISTING CORPUS — and the reason is measured.
#   Re-derived 2026-08-26 over the closed tracked population (321 tracked files under
#   `docs/tasks/`), by this script's own `--all`: **205** claim lines across 29 files, of which **74**
#   carry no census in their own heading section. A blocker firing on 74 pre-existing claims is
#   unusable and teaches bypass — the 91 %-false-positive shape
#   `GENERATED-LINT-CORRECTNESS.6`/`.12` refused. Scoped to what a commit ADDS, it fires on nothing
#   that is already in the tree. This is the condition `SV-CORPUS-GRAD.13e.11` established for a safe
#   blocking guard — *a guard is safe to make blocking over its whole population exactly when the fix
#   precedes it* — with scope-limiting playing the part the fix plays there.
#
# ⭐⭐ CALIBRATED BY REPLAYING REAL HISTORY, NOT AN IMAGINED PHRASING. `ROUTING-EVIDENCE`'s first cut
#   would not have caught its own founding incident because it was calibrated against a phrasing
#   nobody had written. Replayed here over the last 150 commits, judging each commit's ADDED lines
#   against ITS OWN version of the file:
#     claim lines added : 79, in 49 of 150 commits
#     WOULD HAVE BLOCKED: 13 lines, in 11 of 150 commits (7.3 %)
#   Those 13 were adjudicated ONE BY ONE (the `.15`(b) deliverable): 11 genuine unbacked census
#   claims, 1 QUOTATION of an earlier claim, 1 whose census lives in a cross-referenced leaf.
#   ⇒ ~85 % true-positive, the inverse of the shape this repository refuses. `-0290`'s real sentence
#   ("no instrument in the tree crosses the two") IS in the fired set — the founding incident
#   reproduces.
#
# ⚠️ THE 74-OF-205 IS NOT A FALSE-POSITIVE RATE AND MUST NOT BE QUOTED AS ONE. It is the rate at
#   which EXISTING claims lack an in-section census; whether each is WRONG is unmeasured — two
#   audited so far were wrong. The measured false-positive rate is the 2-of-13 above.
#
# ⛔ THE GOVERNED POPULATION IS `docs/tasks/*.md` ONLY, and that is a MEASURED ruling, not taste.
#   Over the same 150 commits: `CHANGES.md` added 20 claim lines of which **18 (90 %)** co-occur with
#   a `docs/tasks` claim in the SAME commit; `DEVELOPMENT_NOTES.md` added 21 of which **16 (76 %)**
#   do. Governing them would double-fire on one underlying claim in the large majority of cases and
#   buy 7 additional lines over 150 commits. ⚠️ Residual, named rather than implied: those 7 escape.
#   `MEMORY.md` is excluded STRUCTURALLY, not by preference — layer A is byte-capped by `MEMORY-ARCH`
#   (7 168 B), so requiring a pasted census command inside it would set two doctrines against each
#   other. A layer-A claim's census belongs in the leaf the pointer names.
#
# ARCHETYPE: evidence (DOCTRINE_ENFORCEMENT.md §3).
#   HONEST LIMIT, stated rather than discovered — this verifies the census was RECORDED, not that it
#   was RUN, not that its population was the right one, and not that its conclusion was correct. It
#   would have caught `-0290`, because `-0290` recorded no census at all; it cannot tell a good
#   census from a bad one. That is the bound every evidence check carries.
# ⚠️ KNOWN FALSE-POSITIVE CLASS, MEASURED at 1 in 13 rather than guessed: a line that QUOTES an
#   earlier claim (or a doctrine's own motto — *"a check that nothing invokes is indistinguishable
#   from one that does not exist"*) fires it. That is the self-referential shape this repository
#   already priced at one site and deliberately did not mechanize around
#   ([[reference_self_referential_assertion_is_unsound]]). Accepted on the same reasoning: the
#   discharge is one pasted command, it errs toward asking rather than staying silent, and narrowing
#   the predicate around quotation marks re-introduces exactly the phrasing-guessing that made
#   `ROUTING-EVIDENCE`'s first cut miss `-0015`.
#
# CONTRACT (DOCTRINE_ENFORCEMENT.md §4): exit code is the verdict; explains on stderr; deterministic
# (no clock, no network, no RNG); read-only; staged-scope-aware; path-agnostic; inert out of scope.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

HEADLINE="GAP-CLAIM-CENSUS"
WORK="rust/target/gap_claim_census"

note() { printf '%s: %s\n' "$HEADLINE" "$1" >&2; }
ok()   { printf '%s: %s\n' "$HEADLINE" "$1"; }

# ── the claim predicate ─────────────────────────────────────────────────────────────────────────
# ⛔ Keyed on `nothing` + a TRANSITIVE VERB, never on the bare word: `nothing` is a house idiom
# occurring on 1 492 lines of `docs/tasks/`, and `nothing else` / `nothing shipped` / `nothing
# technical` are not claims about the tree. The verb is what makes the sentence quantify over it.
CLAIM_RE='nothing (checks|checked|reads|compares|measures|measured|watches|enforces|gates|verifies|validates|asserts|consumes|notices|invokes|runs|ran|re-runs|re-derives|derives|looks|tracks|tracked|reports|counts|sees|knows)|nothing (in|inside) the (repo|repository|tree|codebase|doctrine|gate|registry)|no instrument'

# ── the discharge ───────────────────────────────────────────────────────────────────────────────
# A COMMAND-SHAPED token in the claim's own heading section — the thing that ENUMERATES a population
# — or an explicit `census:` disclosure. ⛔ The bare word "census" is deliberately NOT a discharge:
# it appears inside the claims themselves ("nothing re-runs this census"), so accepting it would let
# a claim discharge itself. Measured: widening the discharge from a bare `grep`/`git` list to this
# one moved the block count from 25 to 13 over the same 150 commits (21 commits -> 11), and every
# one of the 12 removed was a census the author HAD recorded in a spelling the narrow list could not
# see. That control is re-runnable: same claim predicate, discharge list cut back to bare grep/git.
CENSUS_RE='git grep|grep -r|grep -c|grep -l|grep -o|git ls-files|git log -s|git log --grep|git log --oneline|rg -|--dump-|--report-|--lint-grammar|--self-test|scripts/check_|make -c rust|comm -|wc -l|sort -u|uniq -c|sed -n|git show|git diff|cargo |ast_pipeline|a search of|searched (the|all|every)|census:'

# Classify one file. Reads: a newline-separated list of ADDED line numbers, then the file content.
# Emits one `BLOCKED<TAB>lineno<TAB>text` row per undischarged added claim.
# ⛔ Section boundaries are ANY ATX heading, deliberately. `DOCTRINE-GAP-OWNERSHIP.8` measured that
# THREE leaf-heading conventions are in live use here, so a predicate keyed on a leaf-id grammar
# would silently mis-scope; "nearest heading above" needs no such grammar and cannot mis-parse one.
CLASSIFY_AWK='
  FNR==NR { added[$1+0]=1; next }
  { line[FNR]=$0; low[FNR]=tolower($0) }
  END{
    for(i=1;i<=FNR;i++){ if(line[i] ~ /^#{1,6} /) cur=i; sec[i]=cur }
    for(i=1;i<=FNR;i++){ if(low[i] ~ CENSUS_RE) has[sec[i]]=1 }
    for(i=1;i<=FNR;i++){
      if(added[i] && low[i] ~ CLAIM_RE && !has[sec[i]])
        printf "BLOCKED\t%d\t%s\n", i, substr(line[i],1,160)
    }
  }'

# Added-line numbers in the NEW file, from a -U0 diff. `@@ -a,b +c,d @@` — field 3 is `+c[,d]`.
added_line_numbers() { # $1 = diff text on stdin
  awk '
    /^@@/ { split($3,a,","); ln=a[1]+0; next }
    /^\+\+\+/ { next }
    /^\+/ { print ln; ln++; next }'
}

report_rows() { # $1 = file, stdin = BLOCKED rows
  local file="$1" row lineno text
  while IFS=$'\t' read -r _ lineno text; do
    [ -n "$lineno" ] || continue
    printf '  %s:%s\n      %s\n' "$file" "$lineno" "$text" >&2
  done
}

remedy() {
  {
    echo
    echo "  An \"X is checked by NOTHING\" sentence is a CENSUS claim over the whole tree, not a"
    echo "  description — it is false the moment ONE reader exists, and this repository has published"
    echo "  it wrong three times in three days (-0288, -0290 REFUTED by -0294, and one caught in draft)."
    echo
    echo "  Discharge it in the SAME heading section, either way:"
    echo "    1. paste the census you ran, e.g."
    echo "         git grep -n 'left_recursion_unhandled' -- 'rust/src' 'scripts' | wc -l   # -> 0"
    echo "       Any enumerating command counts: git grep / grep -r / git ls-files / git log -S /"
    echo "       a scripts/check_*.sh or --dump-/--report-/--lint-grammar instrument invocation."
    echo "    2. or disclose that you did not run one:"
    echo "         census: not run (<why>) — the claim is REASONED, not measured"
    echo "       An honest 'not run' is a legal answer and a useful signal; a silent claim is not."
    echo
    echo "  ⛔ Census the population that would REFUTE you, not the one that confirms you: -0290 read"
    echo "  two functions, did not look for a third, and the third existed."
  } >&2
}

mode="${1:-}"

# ── --self-test: every arm, observed firing ─────────────────────────────────────────────────────
# A green instrument proves nothing until its refusals fire, and until the GREEN control proves the
# arms are not red for the fixture rather than for the edit.
if [ "$mode" = "--self-test" ]; then
  mkdir -p "$WORK/selftest"
  work="$WORK/selftest"
  arms=0; passed=0
  classify() { # $1 = added-numbers file, $2 = content file
    awk -v CLAIM_RE="$CLAIM_RE" -v CENSUS_RE="$CENSUS_RE" "$CLASSIFY_AWK" "$1" "$2"
  }

  # ARM 1 — GREEN control: a claim WITH a census in its own section is not reported. Without this
  # arm, every red arm below could be red for the fixture rather than for the missing census.
  arms=$((arms + 1))
  cat > "$work/green.md" <<'EOF'
### `.9` — a leaf that measures its own claim
- **THE GAP** — nothing reads the `@sample` annotation.
- **CENSUS** — `git grep -c '@sample' -- 'rust/src' 'scripts'` returns 0 over 2 paths.
EOF
  printf '2\n3\n' > "$work/green.nums"
  if [ -z "$(classify "$work/green.nums" "$work/green.md")" ]; then
    passed=$((passed + 1)); ok "arm 1 GREEN control — a claim backed by a census in its own section passes"
  else
    note "arm 1 FAILED — a censused claim must not be reported"
  fi

  # ARM 2 — RED: the same claim with the census REMOVED is reported.
  arms=$((arms + 1))
  cat > "$work/red.md" <<'EOF'
### `.9` — a leaf that measures its own claim
- **THE GAP** — nothing reads the `@sample` annotation.
EOF
  printf '2\n' > "$work/red.nums"
  if classify "$work/red.nums" "$work/red.md" | grep -q '^BLOCKED'; then
    passed=$((passed + 1)); ok "arm 2 RED control — an unbacked claim is reported"
  else
    note "arm 2 FAILED — an unbacked census claim must be reported"
  fi

  # ARM 3 — THE FOUNDING INCIDENT REPRODUCES. `-0290`'s real sentence, verbatim, must fire. A check
  # calibrated against an imagined phrasing is the `ROUTING-EVIDENCE` first-cut defect; this arm is
  # the standing guard against re-introducing it.
  arms=$((arms + 1))
  cat > "$work/founding.md" <<'EOF'
### `.13e.8` — the v2005 denominator
  grammar, so it cannot see a profile. => **no instrument in the tree crosses the two**, and the
EOF
  printf '2\n' > "$work/founding.nums"
  if classify "$work/founding.nums" "$work/founding.md" | grep -q '^BLOCKED'; then
    passed=$((passed + 1)); ok "arm 3 — the founding incident (-0290, verbatim) is caught"
  else
    note "arm 3 FAILED — the sentence this doctrine exists for must fire"
  fi

  # ARM 4 — a claim already IN the tree, not added by this change, is INERT. This is the whole
  # staged-diff scoping ruling: 74 pre-existing unbacked claims must not block anything.
  arms=$((arms + 1))
  if [ -z "$(printf '' > "$work/none.nums"; classify "$work/none.nums" "$work/red.md")" ]; then
    passed=$((passed + 1)); ok "arm 4 — a pre-existing claim (not in the added set) is INERT"
  else
    note "arm 4 FAILED — the check must judge only what the commit ADDS"
  fi

  # ARM 5 — the house idiom is NOT a claim. `nothing` occurs on 1 492 lines of docs/tasks/; a
  # predicate on the bare word would fire on a third of them.
  arms=$((arms + 1))
  cat > "$work/idiom.md" <<'EOF'
### `.9` — an ordinary leaf
- the totals delta *is* that rule and nothing else. Nothing shipped moved, and nothing technical
  blocks it; there is nothing to do here.
EOF
  printf '2\n3\n' > "$work/idiom.nums"
  if [ -z "$(classify "$work/idiom.nums" "$work/idiom.md")" ]; then
    passed=$((passed + 1)); ok "arm 5 — nothing else / nothing shipped / nothing technical are NOT claims"
  else
    note "arm 5 FAILED — the bare word 'nothing' must not fire; only nothing + a transitive verb"
  fi

  # ARM 6 — the explicit disclosure discharges, so an author who cannot census can still be honest.
  arms=$((arms + 1))
  cat > "$work/disclosed.md" <<'EOF'
### `.9` — a reasoned claim
- **THE GAP** — nothing watches the self-rejection matrix.
- census: not run (the population is the whole engine; priced at ~20 min and declined here)
EOF
  printf '2\n3\n' > "$work/disclosed.nums"
  if [ -z "$(classify "$work/disclosed.nums" "$work/disclosed.md")" ]; then
    passed=$((passed + 1)); ok "arm 6 — an explicit 'census: not run (<why>)' is a legal discharge"
  else
    note "arm 6 FAILED — an honest disclosure must discharge; a silent claim is what this blocks"
  fi

  # ARM 7 — a census in a DIFFERENT section does not discharge. Section scoping is the whole point:
  # a leaf may sit next to twenty commands that have nothing to do with its claim.
  arms=$((arms + 1))
  cat > "$work/other_section.md" <<'EOF'
### `.8` — an earlier leaf
- **CENSUS** — `git grep -c '@sample' -- rust/src` returns 0.

### `.9` — the leaf making the claim
- **THE GAP** — nothing reads the `@sample` annotation.
EOF
  printf '5\n' > "$work/other_section.nums"
  if classify "$work/other_section.nums" "$work/other_section.md" | grep -q '^BLOCKED'; then
    passed=$((passed + 1)); ok "arm 7 — a census in a DIFFERENT section does not discharge the claim"
  else
    note "arm 7 FAILED — the discharge must be section-scoped, not file-scoped"
  fi

  # ARM 8 — the hunk-header parser puts an added line at its NEW-file number. Every verdict above
  # rests on that arithmetic, and it is the one part with no natural oracle, so it is tested
  # directly against a real `git diff -U0` rather than assumed.
  arms=$((arms + 1))
  parsed="$(printf '%s\n' \
    '--- a/x.md' '+++ b/x.md' '@@ -1,0 +2,2 @@' '+alpha' '+beta' '@@ -9,1 +11,1 @@' '-old' '+gamma' \
    | added_line_numbers | tr '\n' ' ')"
  if [ "$parsed" = "2 3 11 " ]; then
    passed=$((passed + 1)); ok "arm 8 — added-line numbers resolve to NEW-file positions (2 3 11)"
  else
    note "arm 8 FAILED — hunk arithmetic yields '$parsed', expected '2 3 11 '"
  fi

  ok "--self-test: arms=${passed}/${arms}"
  [ "$passed" = "$arms" ] || exit 1
  exit 0
fi

# ── --all: the advisory corpus census (never blocks) ────────────────────────────────────────────
# ⛔ ADVISORY BY CONSTRUCTION, and that is the ruling, not a shortcut: blocking on the 74 claims
# already in the tree teaches bypass. This mode exists so the backlog is VISIBLE and countable
# rather than implied — the number that told this doctrine it was warranted.
if [ "$mode" = "--all" ]; then
  total=0; unbacked=0; files=0
  for f in $(git ls-files 'docs/tasks/*.md'); do
    [ -r "$f" ] || continue
    seq 1 "$(wc -l < "$f")" > "$WORK/all.nums" 2>/dev/null || { mkdir -p "$WORK"; seq 1 "$(wc -l < "$f")" > "$WORK/all.nums"; }
    rows="$(awk -v CLAIM_RE="$CLAIM_RE" -v CENSUS_RE="$CENSUS_RE" "$CLASSIFY_AWK" "$WORK/all.nums" "$f")"
    n_claims="$(awk -v CLAIM_RE="$CLAIM_RE" '{ if (tolower($0) ~ CLAIM_RE) c++ } END { print c+0 }' "$f")"
    n_unbacked="$(printf '%s' "$rows" | grep -c '^BLOCKED' || true)"
    total=$((total + n_claims)); unbacked=$((unbacked + n_unbacked))
    [ "$n_claims" -gt 0 ] && { files=$((files + 1)); printf '  %-52s claims=%-4s unbacked=%s\n' "$f" "$n_claims" "$n_unbacked"; }
  done
  ok "--all (ADVISORY, never blocks): ${total} claim line(s) across ${files} file(s), ${unbacked} with no census in their own section"
  ok "  The blocking lane judges ONLY what a commit ADDS. This backlog is reported, not enforced."
  exit 0
fi

# ── the ordinary run: the staged diff ───────────────────────────────────────────────────────────
mkdir -p "$WORK"
staged="$(git diff --cached --name-only --diff-filter=ACM 2>/dev/null \
          | grep -E '^docs/tasks/[^/]*\.md$' || true)"
[ -n "$staged" ] || exit 0

fail=0
checked=0
claims=0
for file in $staged; do
  # ⛔ Judge the INDEX content, never the worktree: the commit ships what is staged, and a worktree
  # edit that adds the census without staging it would otherwise discharge a claim that ships bare.
  git show ":$file" > "$WORK/staged.md" 2>/dev/null || continue
  git diff --cached -U0 -- "$file" 2>/dev/null | added_line_numbers | sort -un > "$WORK/staged.nums"
  [ -s "$WORK/staged.nums" ] || continue
  checked=$((checked + 1))
  rows="$(awk -v CLAIM_RE="$CLAIM_RE" -v CENSUS_RE="$CENSUS_RE" "$CLASSIFY_AWK" \
          "$WORK/staged.nums" "$WORK/staged.md")"
  [ -n "$rows" ] || continue
  n="$(printf '%s\n' "$rows" | grep -c '^BLOCKED')"
  claims=$((claims + n))
  if [ "$fail" = 0 ]; then
    note "a staged task leaf ADDS a \"nothing checks X\" claim with no census in its own section."
  fi
  fail=1
  printf '%s\n' "$rows" | report_rows "$file"
done

if [ "$fail" != 0 ]; then
  remedy
  exit 1
fi

if [ "$checked" = 0 ]; then
  ok "NOT EVALUATED — no staged docs/tasks file carried added lines"
else
  ok "OK — ${checked} staged task file(s), no added claim quantifying over the tree without a census"
fi
exit 0
