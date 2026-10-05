#!/usr/bin/env bash
# scripts/check_grammar_certification.sh
#
# DOCTRINE `GRAMMAR-CERT-CURRENCY` (structural + oracle) — GRAMMAR-CERT-STATUS.2.
#
#   The published per-grammar certification table is a DERIVATION, and a derivation nothing
#   re-derives is a claim that rots. This holds the main book's table current with the tree.
#
# ⛔ WHY IT IS A DOCTRINE AND NOT A MAKE TARGET. This tree exists because the certification roster
# was DOC-ASSERTED (`CHANGES.md`: *"fully-certified-6 roster membership was doc-asserted only"*) and
# rotted silently. `.1` replaced the prose with a derivation and a `--check` that diffs it — and then
# `GATE-REACHABILITY`'s own sentence applied to it verbatim: *a check nothing invokes is
# indistinguishable from one that does not exist.* Nothing invoked it.
#
# ⛔⛔ AND THE CHECK ITSELF HAD ALREADY SILENTLY DIED ONCE (`GRAMMAR-CERT-STATUS.1b`). The commit
# that corrected `.1`'s false table deleted the `--check` implementation and kept the FLAG, so
# `--check` was accepted, printed a table and exited 0 on EVERY input — including a path that does
# not exist — while three surfaces went on publishing that it *"refuses when the two disagree"*.
# That is why tier 1 below does not merely assume the producer works: it TESTS it, both statically
# and behaviourally, on every commit.
#
# ⭐⭐ TWO TIERS, BECAUSE THE ORACLE COSTS 65 s AND A PRE-COMMIT HOOK MAY NOT.
#   Measured 2026-08-23: a full derivation runs the certificate-coverage oracle over 8 families at
#   `--count 40` and takes 65.2 s. Registering that as an every-commit doctrine would tax every
#   commit in the repository for a page that changes rarely. So, the architecture
#   `GENERATED-REPRODUCIBILITY` and `PARSE-COST-RATCHET` already established here:
#
#   TIER 1 (default, every commit, no oracle, no build, < 1 s) — four arms, none of which needs the
#     table's NUMBERS: the block is well-formed; the producer's `--check` is actually WIRED; the
#     published POPULATION equals the families in the tree; and no declared INPUT has run ahead of
#     the page. It cannot prove the numbers are right. It proves that if they were right when they
#     were published, nothing has happened since that could have changed them — and that the
#     instrument which would notice is still alive.
#
#   TIER 2 (`--oracle`, on demand, **64.5 s measured**) — re-derives the whole table and DIFFS it
#     against the published block, and first drives the producer's no-DERIVED-block refusal so a
#     green verdict cannot come from an instrument that says green to everything.
#     ⭐ PREDICTED 65 s AND MEASURED 128.5 s ON THE FIRST RUN, because the red control paid for a
#     WHOLE SECOND DERIVATION to be told something a `grep` knows. The fix was in the producer, not
#     here: the "does this page even have a DERIVED block" question is a property of the PAGE ALONE
#     and now refuses at argument-parse time in 0.01 s, which brought tier 2 back to 64.5 s — one
#     derivation, as predicted. The wrong prediction is kept because re-reading could never have
#     caught it; only running it could.
#
#   ⛔ HONEST BOUND, STATED BEFORE THE CHECK IS TRUSTED: tier 1 inherits whatever tier 2 last
#   established, exactly as `PARSE-COST-RATCHET` says of itself. It is a staleness argument, not a
#   correctness proof. ⛔ And `GENERATED-REPRODUCIBILITY`'s recorded trap applies here too — a cheap
#   tier that PRESCRIBES re-running the oracle inherits the oracle's blind spot — which is why tier
#   1's arms are about the INSTRUMENT and the INPUTS rather than about a recorded number.
#
# CONTRACT (DOCTRINE_ENFORCEMENT.md §4): exit code is the verdict (0 holds / 1 breach / 2 refuses to
# evaluate); explains on stderr; deterministic; read-only; path-agnostic (resolves the repo root
# from its own location); fast in the default tier.
#
# Usage:
#   bash scripts/check_grammar_certification.sh              # tier 1 (the doctrine)
#   bash scripts/check_grammar_certification.sh --oracle     # tier 2: re-derive and diff (65 s)
#   bash scripts/check_grammar_certification.sh --restamp    # tier 2, then record what the table was re-derived
#                                                            # against (the attestation tier 1 reads) — stage the sidecar
#   bash scripts/check_grammar_certification.sh --self-test  # prove the refusal arms fire
#
# ⭐ THE ATTESTATION (GRAMMAR-CERT-STATUS.6, 2026-10-05). Tier 1's fourth arm used to measure a COMMIT
# DISTANCE: how many commits lie between the page's last commit and an input's. That counts how long ago
# the page was TOUCHED, not whether the table is stale — and a re-derivation that reads IDENTICAL leaves
# no byte to stage, so past the budget the prescribed remedy could not reach a commit unless the page
# happened to need another edit (measured twice: -0032 at a lead of 14, and the commit after release
# 1.0.250 at 11, with the table re-derived identical in that release's own ceremony). The arm is now
# keyed on DIGESTS: `--restamp` runs tier 2 and, when the published block equals a fresh derivation,
# writes the sha256 of every declared input (and of the block) to a tracked sidecar. Tier 1 then asks of
# each input "has it moved since the table was last re-derived?" — unmoved inputs need nothing, however
# many commits have passed; a moved one is counted against the same budget, and its remedy (`--restamp`)
# always produces a diff to stage. No sidecar, or one made for another block: the commit-distance arm
# judges alone, exactly as before.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT" || exit 2

# ⭐ The SUBJECT is overridable so the refusal arms below can construct a real breach and drive it
# through this same code path. A self-test that re-implements the check tests the re-implementation.
PAGE="${PGEN_GCC_PAGE:-docs/book/src/grammar-certification-status.md}"
PRODUCER="${PGEN_GCC_PRODUCER:-scripts/report_grammar_certification.sh}"
SV_CONTRACT="rust/test_data/grammar_quality/systemverilog_recognized_cert_union_contract.json"
ATTEST="${PGEN_GCC_ATTEST:-docs/tasks/artifacts/grammar_cert_status/derived_inputs.tsv}"
# ⭐ The lead is BUDGETED, not zero-tolerance. `BASELINE-IDENTITY` (SV-CORPUS-GRAD.13c.2x.4) learned
# this the hard way: its first cut made a moved input a hard failure inside a pre-commit enforcer,
# and ONE COMMENT LINE in the SV grammar blocked every commit in the repository. Staleness inside
# the budget is a printed NOTE; past it, a breach — so rot is still impossible, it just does not
# ambush an unrelated commit.
BUDGET="${PGEN_GRAMMAR_CERT_LEAD_BUDGET:-10}"

MODE=tier1
case "${1:-}" in
  "")           MODE=tier1 ;;
  --oracle)     MODE=oracle ;;
  --restamp)    MODE=restamp ;;
  --self-test)  MODE=selftest ;;
  *) echo "grammar-cert-currency: REFUSED (2) — unknown argument '$1'" >&2; exit 2 ;;
esac

say()  { printf 'grammar-cert-currency: %s\n' "$*" >&2; }

# ---------------------------------------------------------------------------- the attestation
# The content the NEXT COMMIT will hold: the index when the path is in it, the file otherwise (a test
# subject under rust/target is not in the index). This enforcer runs from .githooks/pre-commit and must
# judge the tree the commit WILL create (.5's rule) — a restamped sidecar that was not staged is not
# evidence for that tree. So: the INDEX version when the path is in the index; the working file only for
# a path git IGNORES (a self-test subject); an untracked, un-ignored file yields nothing — "restamped but
# not staged" reads as "no attestation", which is what the commit would contain.
cat_idx()  { git show ":$1" 2>/dev/null && return 0; git check-ignore -q "$1" 2>/dev/null && cat "$1" 2>/dev/null; }
sha_of()   { shasum -a 256 | cut -d' ' -f1; }
block_of() { sed -n '/<!-- BEGIN DERIVED/,/<!-- END DERIVED -->/p'; }
declared_inputs() { # everything the table is a function of
  local inputs="$PRODUCER" f
  [ -f "$SV_CONTRACT" ] && inputs="$inputs $SV_CONTRACT"
  for f in $(block_of < "$PAGE" | sed -n 's/^| `\([a-z0-9_]*\)` |.*/\1/p'); do
    [ -f "grammars/$f.ebnf" ] && inputs="$inputs grammars/$f.ebnf"
  done
  printf '%s\n' $inputs
}
write_attest() { # $1 = sidecar path — called by --restamp AFTER tier 2 passed, and by the self-test's own arms
  local out="$1" f
  mkdir -p "$(dirname "$out")" || return 2
  {
    printf '# what the published per-grammar certification table was last RE-DERIVED against (GRAMMAR-CERT-STATUS.6).\n'
    printf '# Written ONLY by: bash scripts/check_grammar_certification.sh --restamp   (tier 2 re-derives the table; this\n'
    printf '# file is written when the published block equals the fresh derivation). Tier 1 reads it on every commit: an\n'
    printf '# input whose sha256 still equals its row has not moved since that derivation; one that differs is counted\n'
    printf '# against the staleness budget. The `block` row ties the attestation to the block it was made for.\n'
    printf 'block\t%s\t%s\n' "$PAGE" "$(block_of < "$PAGE" | sha_of)"
    for f in $(declared_inputs); do printf 'input\t%s\t%s\n' "$f" "$(sha_of < "$f")"; done
  } > "$out"
}

# ---------------------------------------------------------------------------- tier 1, arm by arm
tier1() {
  local rc=0 notes=0

  # ---- A1 STRUCTURE. The subject must be inspectable. A check that cannot see its subject must
  # SAY SO rather than return green — this driver's own founding rule, applied to itself.
  [ -f "$PAGE" ] || { say "REFUSED (2) — the published page $PAGE is missing"; return 2; }
  [ -f "$PRODUCER" ] || { say "REFUSED (2) — the producer $PRODUCER is missing"; return 2; }
  local nbegin nend
  nbegin=$(grep -c '<!-- BEGIN DERIVED' "$PAGE")
  nend=$(grep -c '<!-- END DERIVED -->' "$PAGE")
  if [ "$nbegin" != "1" ] || [ "$nend" != "1" ]; then
    say "REFUSED (2) — $PAGE must carry exactly ONE <!-- BEGIN DERIVED --> … <!-- END DERIVED -->"
    say "              block; found BEGIN=$nbegin END=$nend. Nothing can be diffed against."
    return 2
  fi

  # ---- A2 THE INSTRUMENT IS ALIVE. Two cheap arms for the `.1b` defect, one static, one
  # behavioural, because neither alone is sufficient:
  #   (a) STATIC — the option variable must be READ, not merely declared and assigned. In the
  #       broken vintage `grep -c '\$CHECK'` was ZERO: `--check` was parsed into a variable nobody
  #       ever looked at. This is the one-line detector that would have caught it instantly.
  #   (b) BEHAVIOURAL — `--check` on a path that cannot exist must REFUSE. A check that returns 0
  #       for a nonexistent file is not measuring anything.
  # ⚠️ HONEST BOUND: (b) is satisfied by the early existence guard alone, so it does NOT prove the
  # DIFF still runs; (a) is a static proxy for that. The arm that really proves the diff needs the
  # 65 s derivation and therefore lives in tier 2, which drives it explicitly.
  local reads
  reads=$(grep -c '\$CHECK' "$PRODUCER")
  if [ "${reads:-0}" -lt 1 ]; then
    say "✗ (1) $PRODUCER parses --check into a variable it NEVER READS (\$CHECK occurs $reads times)."
    say "      That is GRAMMAR-CERT-STATUS.1b exactly: an implementation deleted, its FLAG left"
    say "      behind, so --check is accepted and exits 0 on every input. Restore the diff, or"
    say "      remove the flag too — a surviving flag is what makes the deletion silent."
    rc=1
  fi
  local ghost="rust/target/grammar_cert_currency_nonexistent_page.md"
  if [ -e "$ghost" ]; then
    say "REFUSED (2) — the red-control path $ghost exists; the arm cannot be run"
    return 2
  fi
  bash "$PRODUCER" --check "$ghost" >/dev/null 2>&1
  local ghost_rc=$?
  if [ "$ghost_rc" -eq 0 ]; then
    say "✗ (1) $PRODUCER --check returned 0 for a path that does not exist, so it is inert."
    say "      A check that cannot be made to refuse has not been shown to work (GRAMMAR-CERT-STATUS.1b)."
    rc=1
  fi

  # ---- A3 POPULATION. The published rows must be the families that actually ship. A family added
  # or retired without republishing is drift the NUMBERS cannot show.
  # ⛔ generated/ is UNTRACKED, so a fresh clone cannot answer this. Report NOT EVALUATED, loudly —
  # never as a pass (GENERATED-REPRODUCIBILITY's posture for the same reason).
  if ls generated/*_parser.rs >/dev/null 2>&1; then
    local tree_fams pub_fams
    tree_fams=$(ls generated/*_parser.rs 2>/dev/null | sed 's|generated/||; s|_parser\.rs||' \
                | grep -v '^scratch$' | while read -r g; do [ -f "grammars/$g.ebnf" ] && echo "$g"; done | LC_ALL=C sort)
    pub_fams=$(sed -n '/<!-- BEGIN DERIVED/,/<!-- END DERIVED -->/p' "$PAGE" \
               | sed -n 's/^| `\([a-z0-9_]*\)` |.*/\1/p' | LC_ALL=C sort -u)
    local only_tree only_page
    only_tree=$(LC_ALL=C comm -23 <(printf '%s\n' "$tree_fams") <(printf '%s\n' "$pub_fams"))
    only_page=$(LC_ALL=C comm -13 <(printf '%s\n' "$tree_fams") <(printf '%s\n' "$pub_fams"))
    if [ -n "$only_tree" ] || [ -n "$only_page" ]; then
      [ -n "$only_tree" ] && say "✗ (1) shipped but ABSENT from the published table: $(echo $only_tree)"
      [ -n "$only_page" ] && say "✗ (1) published but NOT shipped: $(echo $only_page)"
      say "      Re-derive and republish: bash $PRODUCER --markdown"
      rc=1
    fi
  else
    say "⚠️ NOT EVALUATED — generated/ holds no parsers, so the published POPULATION cannot be"
    say "   checked against the tree. This is a fresh clone, not a pass. Run:"
    say "   make -C rust SHELL=/bin/bash regenerate_generated_parsers"
  fi

  # ---- A4 NOTHING HAS RUN AHEAD OF THE PAGE. Every declared input is a thing the table is a
  # function of; if one has a newer last-modifying commit than the page, the table MAY be stale.
  # ⛔⛔ STAGED-AWARE, AND THAT IS THE WHOLE POINT (GRAMMAR-CERT-STATUS.5, 2026-08-26). `git log -1`
  # reads the page's last COMMIT, which a staged republish cannot move — and this enforcer runs from
  # .githooks/pre-commit. So once the lead passed the budget, the remedy this arm PRESCRIBES
  # (re-derive, republish, commit) could never reach a commit: the arm blocked the very commit that
  # would clear it, and with it every other commit to the repository. Measured, twice, before the
  # fix. An enforcer that runs at commit time must judge the tree the commit WILL create.
  # Precedent, twice, both the same bug: BASELINE-IDENTITY (SV-CORPUS-GRAD.13c.2x.4 — "ONE COMMENT
  # LINE in the SV grammar BLOCKED EVERY COMMIT") and SV-CONTRACT-CURRENCY tier A (-0242), whose
  # positive form — "add its row in the SAME commit" — this mirrors.
  # ⭐ GRAMMAR-CERT-STATUS.6 — DIGESTS BEFORE DISTANCE. When the tracked attestation was made for the block
  # this page publishes, an input whose sha256 equals its recorded row has NOT moved since the table was
  # last re-derived, and nothing is counted for it. Only a MOVED input (or a tree with no attestation, or
  # one made for another block) is measured by commit distance, from the later of the page's and the
  # attestation's last commits.
  local page_c f in_c lead base_c att att_block cur_block att_c rec cur attested=0 unmoved=0
  if git diff --cached --name-only 2>/dev/null | grep -Fxq "$PAGE" && [ "${PGEN_GCC_MEASURE_STAGED_PAGE:-0}" != "1" ]; then
    say "NOTE — $PAGE is STAGED in this commit, so the block is being republished here and the"
    say "       staleness budget is satisfied by construction. (An UNSTAGED page is still measured:"
    say "       rot past the budget stays impossible for anyone who does not republish.)"
  else
    att=$(cat_idx "$ATTEST")
    cur_block=$(cat_idx "$PAGE" | block_of | sha_of)
    att_block=$(printf '%s\n' "$att" | awk -F'\t' '$1=="block"{print $3; exit}')
    if [ -n "$att_block" ] && [ "$att_block" = "$cur_block" ]; then
      attested=1
    elif [ -n "$att" ]; then
      say "NOTE — $ATTEST attests ANOTHER block than the one $PAGE publishes; it is not evidence for"
      say "       this table, so commit distance judges alone. Re-attest: bash $0 --restamp"
    fi
    page_c=$(git log -1 --format=%H -- "$PAGE" 2>/dev/null)
    base_c="$page_c"
    if [ "$attested" = 1 ]; then
      att_c=$(git log -1 --format=%H -- "$ATTEST" 2>/dev/null)
      if [ -n "$att_c" ] && { [ -z "$base_c" ] || git merge-base --is-ancestor "$base_c" "$att_c" 2>/dev/null; }; then
        base_c="$att_c"
      fi
    fi
    for f in $(declared_inputs); do
      local moved_by_digest=0
      if [ "$attested" = 1 ]; then
        rec=$(printf '%s\n' "$att" | awk -F'\t' -v p="$f" '$1=="input" && $2==p {print $3; exit}')
        cur=$(cat_idx "$f" | sha_of)
        if [ -n "$rec" ] && [ "$rec" = "$cur" ]; then unmoved=$((unmoved + 1)); continue; fi
        moved_by_digest=1
      fi
      if [ -z "$base_c" ]; then
        say "⚠️ NOT EVALUATED — $PAGE has no commit yet, so nothing can be counted against it ($f)."
        continue
      fi
      in_c=$(git log -1 --format=%H -- "$f" 2>/dev/null) || continue
      [ -n "$in_c" ] || continue
      lead=$(git rev-list --count "$base_c".."$in_c" 2>/dev/null) || continue
      # an input that the digests say moved, with no commit ahead yet, moved in the commit being made
      [ "${lead:-0}" -eq 0 ] && [ "$moved_by_digest" = 1 ] && lead=1
      [ "${lead:-0}" -eq 0 ] && continue
      if [ "$lead" -gt "$BUDGET" ]; then
        say "✗ (1) $f leads the published table by $lead commits (budget $BUDGET) — the table may"
        say "      be stale. Re-derive it: bash $0 --restamp   (tier 2; when the table reads identical the"
        say "      recorded digests are refreshed — stage $ATTEST). If it differs, republish the block first:"
        say "      bash $PRODUCER --markdown"
        rc=1
      else
        say "NOTE — $f leads the published table by $lead commit(s), inside the budget of $BUDGET."
        notes=$((notes + 1))
      fi
    done
  fi

  if [ "$rc" -eq 0 ]; then
    local tail=""; [ "$notes" -gt 0 ] && tail=", $notes staleness note(s) inside budget"
    [ "$unmoved" -gt 0 ] && tail="$tail; $unmoved input(s) unmoved since the attested derivation"
    say "OK (tier 1: block well-formed, --check wired and refusing, population agrees, no input past budget$tail)"
  fi
  return "$rc"
}

# ---------------------------------------------------------------------------- tier 2, the oracle
oracle() {
  [ -f "$PAGE" ] || { say "REFUSED (2) — the published page $PAGE is missing"; return 2; }
  # Drive the producer's refusal arms behaviourally FIRST, so a green diff can never come from an
  # instrument that says green to everything. A check proven only on the subject it is supposed to
  # bless has not been proven at all.
  local t="rust/target/grammar_cert_currency_selftest"; mkdir -p "$t" || return 2
  printf '# no derived block here\n' > "$t/noblock.md"
  bash "$PRODUCER" --check "$t/noblock.md" >/dev/null 2>&1
  if [ $? -ne 2 ]; then
    say "REFUSED (2) — $PRODUCER --check did not REFUSE on a page with no DERIVED block, so its"
    say "              verdict on the real page cannot be trusted (GRAMMAR-CERT-STATUS.1b)."
    return 2
  fi
  say "red control OK — --check refuses a page with no DERIVED block"
  bash "$PRODUCER" --check "$PAGE"
  local rc=$?
  [ "$rc" -eq 0 ] && say "OK (tier 2: the published table equals a fresh derivation)"
  return "$rc"
}

# ---------------------------------------------------------------------------- tier 2 + the attestation
restamp() {
  oracle || {
    say "REFUSED (2) — NOT restamped: tier 2 did not find the published block current. Republish it"
    say "              (bash $PRODUCER --markdown, splice the block), then --restamp."
    return 2
  }
  write_attest "$ATTEST" || { say "REFUSED (2) — could not write $ATTEST"; return 2; }
  say "restamped $ATTEST — $(grep -c '^input' "$ATTEST") input digest(s) recorded for the block tier 2 just re-derived."
  say "Stage it with the commit: git add $ATTEST"
}

# ---------------------------------------------------------------------------- the refusal arms
# ⭐ A doctrine that has never been observed FAILING is a doctrine nobody has tested. Each arm below
# constructs the breach it claims to catch and asserts the exit code.
selftest() {
  local t="rust/target/grammar_cert_currency_selftest" pass=0 total=0
  rm -rf "$t"; mkdir -p "$t" || return 2
  arm() { # arm <name> <want-rc> <cmd...>
    local name="$1" want="$2"; shift 2
    total=$((total + 1))
    "$@" >/dev/null 2>&1
    local got=$?
    if [ "$got" = "$want" ]; then pass=$((pass + 1)); printf '  ✓ %-48s rc=%s\n' "$name" "$got" >&2
    else printf '  ✗ %-48s rc=%s (want %s)\n' "$name" "$got" "$want" >&2; fi
  }

  # --- controls. A refusal arm proves nothing unless the green arm proves the check can also pass.
  arm "GREEN control: the real tree holds"          0 bash "$0"
  arm "unknown argument REFUSES"                    2 bash "$0" --no-such-flag

  # --- A1 structure
  arm "missing page REFUSES"                        2 env PGEN_GCC_PAGE="$t/absent.md" bash "$0"
  printf '# page\n<!-- BEGIN DERIVED -->\nno end marker\n' > "$t/half.md"
  arm "BEGIN without END REFUSES"                   2 env PGEN_GCC_PAGE="$t/half.md" bash "$0"
  sed -e 's/<!-- BEGIN DERIVED/<!-- BEGIN DERIVED/' "$PAGE" > "$t/twice.md"
  sed -n '/<!-- BEGIN DERIVED/,/<!-- END DERIVED -->/p' "$PAGE" >> "$t/twice.md"
  arm "two DERIVED blocks REFUSE"                   2 env PGEN_GCC_PAGE="$t/twice.md" bash "$0"

  # --- A2 the instrument is alive: THE `.1b` REGRESSION, reconstructed both ways.
  # (a) a producer that parses --check into a variable it never reads — the exact broken vintage.
  cat > "$t/inert_producer.sh" <<'STUB'
#!/usr/bin/env bash
MODE=text; CHECK=""
while [ $# -gt 0 ]; do case "$1" in --check) CHECK="${2:-}"; shift ;; esac; shift; done
echo "a table"; exit 0
STUB
  chmod +x "$t/inert_producer.sh"
  arm "producer that never reads \$CHECK BREACHES"   1 env PGEN_GCC_PRODUCER="$t/inert_producer.sh" bash "$0"
  # (b) a producer that DOES read $CHECK but still returns 0 for a path that cannot exist.
  cat > "$t/green_producer.sh" <<'STUB'
#!/usr/bin/env bash
CHECK=""
while [ $# -gt 0 ]; do case "$1" in --check) CHECK="${2:-}"; shift ;; esac; shift; done
[ -n "$CHECK" ] && echo "pretending to check $CHECK"
exit 0
STUB
  chmod +x "$t/green_producer.sh"
  arm "producer green on a nonexistent page BREACHES" 1 env PGEN_GCC_PRODUCER="$t/green_producer.sh" bash "$0"
  # (c) the REAL producer must refuse that same arm — otherwise (b) could never fire on it.
  arm "the real producer REFUSES a nonexistent page" 2 bash "$PRODUCER" --check "$t/does-not-exist.md"

  # --- A3 population
  awk '/<!-- END DERIVED -->/ && !d {print "| `bogus_family_xyz` | ⛔ **no** | 1 rules | invented |"; d=1} {print}' \
    "$PAGE" > "$t/extra_family.md"
  arm "a published family that does not ship BREACHES" 1 env PGEN_GCC_PAGE="$t/extra_family.md" bash "$0"

  # --- A4 the attestation (GRAMMAR-CERT-STATUS.6). Each arm measures the REAL page (the staged-page
  # shortcut switched off, so the arm's verdict does not depend on what this commit stages) under a
  # budget of ZERO, where the commit-distance arm alone would refuse whenever any input has a newer commit.
  write_attest "$t/attest_current.tsv"
  arm "inputs attested unmoved HOLD past the budget" 0 env PGEN_GCC_ATTEST="$t/attest_current.tsv" \
      PGEN_GCC_MEASURE_STAGED_PAGE=1 PGEN_GRAMMAR_CERT_LEAD_BUDGET=0 bash "$0"
  awk -F'\t' 'BEGIN{OFS="\t"} $1=="input" && !d {$3="0000000000000000000000000000000000000000000000000000000000000000"; d=1} {print}' \
      "$t/attest_current.tsv" > "$t/attest_moved.tsv"
  arm "an input MOVED since the attestation BREACHES"  1 env PGEN_GCC_ATTEST="$t/attest_moved.tsv" \
      PGEN_GCC_MEASURE_STAGED_PAGE=1 PGEN_GRAMMAR_CERT_LEAD_BUDGET=0 bash "$0"
  awk -F'\t' 'BEGIN{OFS="\t"} $1=="block" {$3="1111111111111111111111111111111111111111111111111111111111111111"} {print}' \
      "$t/attest_current.tsv" > "$t/attest_other_block.tsv"
  arm "an attestation for ANOTHER block is not evidence" 1 env PGEN_GCC_ATTEST="$t/attest_other_block.tsv" \
      PGEN_GCC_MEASURE_STAGED_PAGE=1 PGEN_GRAMMAR_CERT_LEAD_BUDGET=0 bash "$0"
  arm "--restamp REFUSES an unknown extra argument"   2 bash "$0" --restamp-now

  printf '  (the DIFF arm — a drifted table — needs the 65 s derivation and is driven by --oracle,\n' >&2
  printf '   not here. Stated rather than quietly skipped.)\n' >&2
  printf 'grammar-cert-currency self-test: %d/%d arms as expected\n' "$pass" "$total" >&2
  [ "$pass" = "$total" ]
}

case "$MODE" in
  tier1)    tier1 ;;
  oracle)   oracle ;;
  restamp)  restamp ;;
  selftest) selftest ;;
esac
