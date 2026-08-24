#!/usr/bin/env bash
# scripts/check_doctrines.sh — THE GENERAL DOCTRINE ENFORCER (driver + registry).
#
# PGEN-DIAG-TOOLBOX-0002. Director directive (2026-06-22): make doctrine compliance
# "provable, trackable ... so we do not have to trust your words — rule enforcers that
# literally force the rule, no matter what." This is the single driver that runs EVERY
# mechanizable doctrine check, reports per-doctrine PASS/FAIL, and exits NONZERO on any
# breach. It formalizes the previously ad-hoc `.githooks/pre-commit` check stack into one
# registered, self-reporting framework.
#
# Enforcement layering (MEMORY_ARCHITECTURE.md §9 — defense in depth):
#   E1 discovery   : the doctrine docs (README, TOOLBOX.md, docs/decisions/, the books).
#   E2 self-check  : THIS script + each registered scripts/check_*.sh (single source of truth).
#   E3 git hook    : .githooks/pre-commit calls this (fast local gate; activated via
#                    `git config core.hooksPath .githooks`).
#   E4 CI          : the same script runs server-side (un-bypassable backstop) — invoked by
#                    `.github/workflows/memory-architecture-gate.yml`, the one workflow that
#                    still runs on `push`/`pull_request`. ⭐ IT MUST INVOKE THIS DRIVER, NOT
#                    INDIVIDUAL ENFORCERS: it named five of them until 2026-07-29
#                    (CI-PARITY-GATE-ROT.15), so 8 of the 13 registered doctrines had NO
#                    automatic lane and a doctrine added tomorrow silently got none. The
#                    registry is the single source of the roster, so calling the driver makes
#                    a new doctrine inherit the lane by construction. `FLOW-INTEGRITY`
#                    invariant (8) enforces that; the other 14 workflows stay manual-only
#                    (workflow_dispatch) to conserve Actions minutes.
#                    HONEST GAP: a local hook can be `--no-verify`'d, and the four
#                    STAGED-SCOPE doctrines below evaluate nothing on a hosted push — which
#                    this driver now SAYS rather than passing silently.
#
# What "provable / not trust-me-bro" means here, honestly:
#   - STRUCTURAL doctrines (memory-arch, docpaths, ebnf-SoT, knowledge-map) — the check
#     re-derives the invariant from the tree; pass/fail is a fact about the files.
#   - DETERMINISTIC-ORACLE doctrines (cert-coverage, ast_shape_contract, syntax-closure —
#     run by the broader make gates / CI) re-RUN the real tool at fixed seeds, so a claimed
#     number that does not reproduce FAILS. This is the strongest leg.
#   - EVIDENCE doctrines (diag-toolbox-evidence) require a re-checkable artifact (pasted tool
#     output) in the owning task leaf. A hook cannot prove the agent REASONED from it, but it
#     makes landing a code change with no pasted, reproducible tool output impossible at the gate.
#
# Registry below = the source of truth for "which doctrines are enforced by what". The
# human-readable mirror is DOCTRINE_ENFORCEMENT.md §10 (kept in lockstep).
set -uo pipefail   # deliberately NOT `-e`: run ALL checks, collect every result, then report.
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$ROOT"

# Each entry: "ID|what it proves|relative/path/to/check.sh"
# Add a new doctrine here AND in docs/decisions/DOCTRINE_ENFORCEMENT.md (a meta-check below
# asserts every registered enforcer exists + is executable, so a registry entry cannot be a
# dangling promise).
DOCTRINES=(
  "MEMORY-ARCH|durable 4-layer memory architecture invariants (MEMORY_ARCHITECTURE.md §9), incl. layer A bounded by BOTH a line cap and a byte cap — the line-only form passed 60 lines / 138,403 bytes (README-POLICY.2)|scripts/check_memory_architecture.sh"
  "DIAG-SEVERITY+DOCPATH|severity never masked by verbosity + repo-root-relative live-doc paths|scripts/check_diagnostics_and_docpaths.sh"
  "EBNF-SOURCE-OF-TRUTH|no new out-of-band acceptance validator wired outside the EBNF|scripts/check_ebnf_source_of_truth.sh"
  "REGEX-SELF-HOSTING|the regex grammar self-hosts (gen<->parse duality holds)|scripts/check_regex_self_hosting.sh"
  "KNOWLEDGE-MAP|the derived Knowledge Map is in sync with its fact sources|knowledge-map/scripts/check_knowledge_map.sh"
  "TASK-ACCEPTANCE|a code change's task leaf passes the acceptance checklist (root-cause + addressed + no-regression boxes ticked + evidence-backed)|scripts/check_diagnosis_evidence.sh"
  "REGEX-ORACLE-ANCHOR-SYNC|the live oracle-tuple anchors agree with each other + the tracked ratchet bounds|scripts/check_regex_oracle_anchor_sync.sh"
  "LESSON-PROMOTION|a durable lesson written to DEVELOPMENT_NOTES.md is either PROMOTED into the retrievable layer or EXPLICITLY DECLINED, never silently dropped — the promotion mechanism existed, was wired, and was skipped 1592 times because no gate asked (LESSON-RETRIEVAL.4; KNOWLEDGE-MAP checks the map is in sync with its SOURCES, never that a lesson REACHED a source)|scripts/check_lesson_promotion.sh"
  "DESIGN-PRIOR-ART|a task leaf proposing a NEW annotation/directive surface records a prior-art search first (docs/decisions/feedback_read_prior_art_before_designing.md)|scripts/check_design_prior_art.sh"
  "WAIVER-ROUTING|a task leaf claiming a gate does not apply names the leaf that owns fixing the gate (an unrouted waiver is a bug report about the gate, left inert)|scripts/check_waiver_routing.sh"
  "ROUTING-EVIDENCE|a task leaf routing a finding OUT to another tree records what it MEASURED — above all whether the finding reproduces outside the family it is being sent to (CI-PARITY-GATE-ROT.9: a shared-gate defect was routed to the regex family on a plausible reading, and the deciding evidence was already on disk)|scripts/check_routing_evidence.sh"
  "DESTRUCTIVE-TARGET-GUARD|destructive make targets refuse without PGEN_CONFIRM_CLEAN=1 + no innocuous alias routes into them (OPS-MEMSAFE.3)|scripts/check_destructive_target_guard.sh"
  "GATE-REACHABILITY|every tracked gate target is invoked by something that RUNS, or carries a deliberate disposition — a check nothing invokes is indistinguishable from one that does not exist (CI-PARITY-GATE-ROT.2)|scripts/check_gate_reachability.sh"
  "FLOW-INTEGRITY|the gate flow cannot drift back: workflows that need generated parsers regenerate them and budget for it, the recipe keeps one home, artifact hand-offs never point at a standalone default, no assertion requires a defect to pass, hand-off provenance coverage only improves (CI-PARITY-GATE-ROT.8), the doctrine roster keeps an AUTOMATIC lane through THIS driver rather than a re-typed list of enforcers (CI-PARITY-GATE-ROT.15), a guard tests the artifact it actually READS (CI-PARITY-GATE-ROT.14), the SHIPPING generation recipe stays QUIET — no tracked Makefile invokes --generate-parser carrying --debug/--trace, which cost 6.89 GB of log per regeneration for artifacts the flags provably cannot change (CI-PARITY-GATE-ROT.31, director-ruled), and every EBNF→json rule is guarded against GNU Make 3.81's WHOLE-SECOND mtime comparison, which silently skips a rule whose prerequisite was rewritten in the same second — measured 5 of 10 families exposed on the grammar→json edge and 10 of 10 on json→parser (CI-PARITY-GATE-ROT.32)|scripts/check_flow_integrity.sh"
  "PARSER-BOOK-CURRENCY|a per-parser BOOK publishes the release its family actually SHIPS, in BOTH directions, over a CLOSED population of all ten books — the book is the director's stated review surface (\"my only window into the project — I review the book, not the code\"), and it sat in the seam between the three existing currency doctrines: PUBLISHED-VERSION-CURRENCY holds the regex identity pair and the per-family STATUS row, LIVE-DOC-CURRENCY classifies by date spread, SV-CONTRACT-CURRENCY holds the CONTRACT current with the GRAMMAR, and none of them compares a family book against its family's release (SV-CORPUS-GRAD.13c.2z; measured 2026-08-21: the SystemVerilog book published 1.0.183 against a shipped 1.0.192 and schema 20 against 25 — nine releases and five schemas invisible — and this was the SECOND occurrence, the first having been hand-reconstructed with a prose warning naming the contract as the winner, which bought exactly one cycle before the same sentence drifted a third time). ⛔ The population is CLOSED and every book is CLASSIFIED, because sizing it first is what found the other two: vhdl was a release stale and systemverilog_preprocessor uses a heading convention a SystemVerilog-shaped regex reads as an EMPTY book|scripts/check_parser_book_currency.sh"
  "PUBLISHED-VERSION-CURRENCY|the user guide's published regex identity pair equals the contract's Contract Identity block and its published family status equals the live tracker row — a Provisional/any parser SHIPS on its published state, so the published state must be gate-held true (DONE-BAR.5a; measured ~77 releases stale with no gate reading it)|scripts/check_published_version_currency.sh"
  "LIVE-DOC-CURRENCY|a live document is currently TRUE, not merely bounded: instrument A counts DISTINCT dates per surface PAIRED WITH ITS DECLARED CHARTER (so it classifies rather than alarms — CHANGES.md scores 175 and is correct), instrument B catches a self-declared \`Last updated:\` refuted by the file's own newest content date (no baseline, no threshold), and route closure requires every .md destination a capped enforcer NAMES to be a watched surface — the unwatched-overflow edge is how LIVE_ACHIEVEMENT_STATUS.md reached 1 547 057 B beside a README capped on two axes (LIVE-MEANS-LIVE.2). ⛔ B REFUSES on any unclassified declaration spelling rather than enumerating spellings: enumeration is what made the same population measure 10, then 16, then 18, every miss silent in the passing direction|scripts/check_live_document_currency.sh"
  "SV-CORPUS-DENOMINATOR|the SV corpus DENOMINATOR is re-derived and published beside the defect bar: the tracked verdict-coverage artifacts are byte-identical to a fresh census re-run, and every designated live surface carries the derived adjudicated/routed/no-verdict/dark/bar tuple — a bar without its denominator is not a claim about the corpus (SV-CORPUS-GRAD.13b; the artifact was MEASURED stale one day after landing, published bar 319 vs HEAD's 318, with NOTHING in the repo invoking the instrument)|scripts/check_sv_corpus_denominator.sh"
  "PARSE-COST-RATCHET|the SV parser's parse COST is measured and cannot rise unwatched: the tracked baseline's instrument identity (grammar + generated parser + instrument + a digest of the sampled corpus files) is re-hashed every run alongside FOUR more every-run arms — the LR-family classifier over all ten generated parsers, the published corpus family share re-hashed against its tracked derivation, and CO-PUBLICATION of that derived share on every designated live surface (ENGINE-UNIVERSAL-SERVICES.21 (f)), plus the PROBE FINGERPRINT: those four identity rows are all SOURCES, while the numbers are produced by an untracked probe binary nothing hashed, so rust/build.rs now sha256s every generated parser it already rerun-if-changed's and the probe reports which one it embeds — a gap that failed in the PASSING direction, since a probe built from a guard-suppressed arm reported 762,345 rule entries against the shipped parser's 11,240,430 and a FALL is not a breach here but an invitation to rebaseline (ENGINE-UNIVERSAL-SERVICES.24) — and a re-measure refuses on any rise in the binding counters — entries / committed / memo-hits, each verified deterministic and build-mode-independent before being made binding (ENGINE-UNIVERSAL-SERVICES.20 acceptance (d); the guarded admission shipped while the lint, the two-sided repro ratchet, the corpus pass/fail count and every registered doctrine stayed GREEN, because nothing measured parse cost at all — it cost +10.59 % rule entries, and the +24.3 % PARSE TIME this doctrine was founded on is REFUTED by .20 slice 5 as a fixed-arm-order artifact, which is the argument FOR a re-derivable ratchet, not against one; .26 retired the blind-spot factor built on it)|scripts/check_parse_cost_ratchet.sh"
  "SV-RULE-FIRE-PARTITION|the SV rules that NEVER FIRED on the corpus are CLASSIFIED and the classification is CONSUMED: the tracked partition is held consistent with the coverage artifact it derives from, every rule the certificate pass could not witness carries a hand adjudication with a measured LRM construct behind it, and the unwitnessed class must stay at zero (SV-CORPUS-GRAD.13c.2u; the coverage artifact published a flat never-fired list that NOTHING read, its headline 104 was stale at 137, and a large part of it cannot fire because PGEN's own LR eliminator authored or replaced those rules — \`casting_type\` reads never-fired while its nineteen _lr_* replacements carry every size cast — so an unclassified list reads as a defect surface and sends a reader hunting phantoms)|scripts/check_sv_rule_fire_partition.sh"
  "SCRATCH-SLOT-HEADER|the blessed throwaway scratch slot keeps its OPERATING MANUAL — the header block that is the only record of how to regenerate, drive and restore the slot, and of the rebuild-ast_pipeline-AFTER ordering trap — because the body is meant to be overwritten and the header is not (PARSE-HARNESS.11; measured 2026-08-15: an agent replaced the whole file, and the correct workflow ends in \`git checkout\`, so the loss can NEVER reach a commit for a commit-time gate to see — the probe-time tier in \`make focus_scratch\` is the one that catches it, and this tier catches a removal that IS committed)|scripts/check_scratch_slot_header.sh"
  "GENERATED-REPRODUCIBILITY|every artifact in the untracked generated/ tree is what HEAD's tracked source PRODUCES — not merely what it compiles into, and not merely unchanged since someone recorded a hash (ENGINE-UNIVERSAL-SERVICES.29; measured 2026-08-16: both annotation parsers carried a line the code generator CANNOT emit — grep -c 0, git log -S no commit ever — written from an uncommitted editor state, and they are the pair the annotation backend links to generate every OTHER parser, while layer A recorded 'generated/ FRESH' throughout)|scripts/check_generated_reproducibility.sh"
  "README-STABILITY|README.md stays a stable LANDING PAGE — both a line cap and a byte cap, because a line cap alone is measurably bypassable — at adoption layer-A MEMORY.md passed its 60-line cap carrying 138,403 unbounded bytes, a bypass since CLOSED by giving MEMORY-ARCH the same two caps (README-POLICY.2) — plus a changelog-leakage tripwire and a link back to the reviewed policy (README-POLICY.1; adopted at 510 lines/48,811 bytes with NO instrument watching size: the two guards that touch README.md audit doc PATHS and the root file SET)|scripts/check_readme_stability.sh"
  "SV-CONTRACT-CURRENCY|the SV downstream integration contract describes the grammar that is actually in the tree: every revision of grammars/systemverilog.ebnf since the register genesis is accounted for as a RELEASE (contract section + bug-ledger row) or as ACCEPT-SET-NEUTRAL, and a neutrality claim is CHECKED against the frontend's own raw_ast digest rather than believed (SV-CORPUS-GRAD.13c.2l; measured 2026-08-19: the contract said 1.0.183 while SEVEN semantically-distinct grammar revisions had shipped, FOUR of them REPLACING an AST shape a consumer was already reading, and one of those four moved ZERO verdicts — nothing compared the grammar's last-modified commit with the contract's, and the leaf tracking the population BY HAND went stale twice more inside the same week)|scripts/check_sv_contract_currency.sh"
  "GRAMMAR-CERT-CURRENCY|the published per-grammar certification table stays a DERIVATION of the tree, and the instrument that would notice is still ALIVE: tier 1 (0.27 s, every commit, no oracle) proves the book page carries exactly one well-formed DERIVED block, that the producer's --check is WIRED (statically -- the option variable is READ, not merely assigned -- and behaviourally -- it refuses a path that does not exist), that the published POPULATION equals the families that actually ship, and that no declared input leads the page past a budget; tier 2 (--oracle, 64.5 s MEASURED -- predicted 65 s, first measured 128.5 s because the red control paid for a whole second derivation to be told something a grep knows, fixed in the producer by refusing a block-less page at argument-parse time in 0.01 s) re-derives all nine rows and DIFFS them, after first driving the producer's own refusal arm so a green verdict cannot come from an instrument that says green to everything (GRAMMAR-CERT-STATUS.2). MEASURED twice over: the roster was DOC-ASSERTED and rotted (CHANGES.md: \"fully-certified-6 roster membership was doc-asserted only\"), then the derive-and-diff that replaced it was itself DELETED while its FLAG survived, so --check was accepted, printed a table and exited 0 on EVERY input -- including a nonexistent path -- for a whole commit, while the main book, the owning leaf and layer-A MEMORY.md all published that it \"refuses when the two disagree\" (GRAMMAR-CERT-STATUS.1b). HONEST BOUND, stated before the check is trusted: tier 1 is a staleness argument and inherits whatever tier 2 last established, exactly as PARSE-COST-RATCHET says of itself; generated/ is untracked, so a fresh clone reports the population arm NOT EVALUATED, loudly, never as a pass. 9/9 refusal arms observed firing via --self-test, including the two that reconstruct the .1b regression from both sides|scripts/check_grammar_certification.sh"
  "BASELINE-IDENTITY|a tracked baseline holding a value DERIVED from the tree says WHICH tree, and the enforcer RE-HASHES every input it declares on every run: a generic \`identity\` block (\`verified_at_commit\` + an \`inputs\` map of repo-root-relative path -> sha256) whose dependency set is DATA rather than hard-coded in a script, ONE shared verifier so every gate refuses in provably identical words, and a CLOSED two-sided population over rust/test_data/grammar_quality/ in which an entry with no verdict FAILS and a verdict naming no entry FAILS. ⛔⛔ The block answers TWO questions and the \`expectations\` field (\`confirmed\` or \`unconfirmed\`, with NO default) is the second: input digests say whether the inputs MOVED, never whether the numbers were ever RIGHT about them. Shipping without it inverted this doctrine on its own first customer (-0248, retracted by -0249) -- the gate printed \"identity fresh\" and then an unattributable drift, turning an honest vague RED into a confident WRONG answer. \`unconfirmed\` is RED for every consumer, never a waiver, so a baseline nobody can re-derive yet can still be watched for input drift and still name the leaf that owes the re-derivation. ⭐⭐ AND PROVENANCE DISAMBIGUATES A VERDICT RATHER THAN GATING WORK (SV-CORPUS-GRAD.13c.2x.4): the first cut made a moved input a FAILURE in an enforcer that runs from .githooks/pre-commit, so ONE COMMENT LINE in the SV grammar BLOCKED EVERY COMMIT -- for an edit the frontend strips. Now an input may be digested SEMANTICALLY (\`ebnf_raw_ast\`, the frontend's own envelope, the same definition SV-CONTRACT-CURRENCY uses), staleness is a printed NOTE inside a DERIVED budget of commits touching the declared inputs and a hard failure past it (so ROT stays impossible -- the founding artifact sat 69 revisions stale), and a gate MEASURES a stale baseline instead of refusing: green re-stamps it from that run's own numbers, red refuses as ambiguous, and the env-gated escape the old design needed is DELETED (SV-CORPUS-GRAD.13c.2x.2, director-ordered 2026-08-20; measured 2026-08-20: sv_cert_recognized_union_gate had been RED at HEAD for ELEVEN grammar revisions by 71 rules and 53 UNKNOWNs, every expected_* field an exact function of that grammar and not one byte in the file able to say the baseline was the stale half). ⛔ A block nobody READS is the defect, not the fix — SV-CORPUS-GRAD.13i measured SIX oracles already carrying a self-describing identity block, only ONE gate-checked and FOUR measurably stale — so a row registered anything but adopted that CARRIES a block is a hard failure, and the first adoption shipped with its consuming gate REFUSING TO MEASURE (exit 2, the published refusal code) until the identity verifies|scripts/check_baseline_identity.sh"
  "CORPUS-KEY-INTEGRITY|the corpus ANSWER KEY is checked like the instrument it is, and so is the instrument that checks it: tier A1 (always, corpus-INDEPENDENT, 0.1 s) runs the census self-test and refuses a shrunken arm set, a zero-arm 'pass' or an unparsable summary; tier A4 (also corpus-INDEPENDENT, it reads the tracked manifests) holds the PROVENANCE census the same way and pins its UNMATCHED residual under a ceiling -- a rise means the manifests gained a basis SHAPE nobody classified, so the published trust bound (93.8 % of the answer key rests on evidence OUTSIDE the standard: 627 clause-cited / 2287 tool-testimony / 7172 suite-convention / 4 unmatched, over 10 090 keyed rows) silently stopped covering the population; tier A2/A3 (0.49 s total) requires zero contradictory upstream-message classes, zero bases quoting evidence their own golden lacks, and the tracked census.md to be BYTE-IDENTICAL to a fresh derivation. Founded on the key measurably deciding wrong twice in three days (SV-CORPUS-GRAD.13e.3(a): two files, the same binary ~& operator, OPPOSITE expectations, the difference an iverilog flag the key never read; SV-0068: three rows keyed must_accept for text IEEE 1364-2005 cannot derive, sitting at `match` -- the STRONGEST verdict in the file -- for the whole campaign, because an expectation error in the ACCEPT direction produces no flag at all). ⛔⛔ AND THE FIRST CENSUS BUILT TO CATCH THAT WAS ITSELF WRONG TWICE INSIDE ONE COMMIT (CORPUS-KEY-AUDIT.1(a)) -- a 100 % false-positive rate, 2 of 2, and a message regex requiring an error:/sorry: tag while iverilog emits its bare parse refusal UNTAGGED, so `syntax error`, the most parse-relevant class in the corpus, was absent from the vocabulary its headline was measured over -- which is why A1 tests the census before A2 trusts its verdict. The vendored corpora are submodules, so a fresh clone reports A2/A3 NOT EVALUATED, LOUDLY, never as a pass and never as a commit-blocking failure; that refusal branch is EXERCISED by the self-test (a relocated census copy whose repo-root walk lands on no corpus), not assumed. 13/13 refusal arms observed firing via --self-test, including a GREEN control on the real tree so the refusals are not vacuous|scripts/check_corpus_key_integrity.sh"
)

fail=0
declare -a report=()

for entry in "${DOCTRINES[@]}"; do
  # ⛔ THE FIELD SEPARATOR IS `|`, SO A `|` INSIDE A DESCRIPTION SPLITS THE ROW IN THE WRONG PLACE
  # and `script` ends up holding a fragment of prose. The failure is loud but the MESSAGE is wrong:
  # it reads "registered enforcer missing or not executable: <a paragraph of English>", which sends
  # the reader looking for a missing file. Measured 2026-08-20 while registering `BASELINE-IDENTITY`
  # (`SV-CORPUS-GRAD.13c.2x.2`) — the third escaping defect this registry has had, after the
  # unescaped backticks that ran a command substitution (`-0241`). Diagnose it here instead.
  if [ "$(printf '%s' "$entry" | tr -cd '|' | wc -c | tr -d ' ')" != "2" ]; then
    report+=("✗ FAIL  <meta:registry> — a registry row does not have exactly 2 '|' separators, so"
             "        its fields split in the wrong place. A '|' inside the description is the"
             "        usual cause; the row begins: ${entry:0:60}…")
    fail=1
    continue
  fi
  IFS='|' read -r id proves script <<< "$entry"
  if [ ! -x "$ROOT/$script" ]; then
    report+=("✗ FAIL  ${id} — registered enforcer missing or not executable: ${script}")
    fail=1
    continue
  fi
  if out="$("$ROOT/$script" 2>&1)"; then
    report+=("✓ PASS  ${id} — ${proves}")
  else
    report+=("✗ FAIL  ${id} — ${proves}")
    printf '%s\n' "$out" >&2
    fail=1
  fi
done

# ------------------------------------------------------------------ meta-check: the human mirror
# The registry above is the SOURCE OF TRUTH and the header has always CLAIMED that
# DOCTRINE_ENFORCEMENT.md §10 is "kept in lockstep" with it. MEASURED 2026-07-29
# (CI-PARITY-GATE-ROT.15): it was not — 3 registered doctrines had no row (`TASK-ACCEPTANCE`,
# `ROUTING-EVIDENCE`, `DESTRUCTIVE-TARGET-GUARD`) and 1 row named `DIAG-TOOLBOX-EVIDENCE`, an id the
# registry no longer carries. A documented promise nobody checks is the exact rot this driver exists
# to end, one level up. ⛔ The assertion lives HERE and is about a DIFFERENT file, deliberately: an
# assertion about a file cannot live inside that file as a literal (docs/decisions/
# reference_self_referential_assertion_is_unsound.md).
# ⚠️ Honest limit, stated not discovered: this proves the ID SETS agree, not that each row's prose is
# accurate. Row text is reviewed, not gated.
MIRROR="DOCTRINE_ENFORCEMENT.md"
mirror_ids="$(sed -n '/^## 10\. The live PGEN instance/,/^## 11\./p' "$ROOT/$MIRROR" 2>/dev/null \
              | sed -n 's/^| `\([A-Z][A-Z0-9+-]*\)` |.*/\1/p' | LC_ALL=C sort -u)"
registry_ids="$(printf '%s\n' "${DOCTRINES[@]}" | cut -d'|' -f1 | LC_ALL=C sort -u)"
if [ -z "$mirror_ids" ]; then
  report+=("✗ FAIL  <meta:mirror> — ${MIRROR} §10 yielded ZERO doctrine rows. A check that cannot"
           "        inspect its subject must SAY SO, not pass: either the section was renamed or its"
           "        table shape changed. Fix the extraction or the section, do not delete the check.")
  fail=1
else
  only_registry="$(LC_ALL=C comm -23 <(printf '%s\n' "$registry_ids") <(printf '%s\n' "$mirror_ids"))"
  only_mirror="$(LC_ALL=C comm -13 <(printf '%s\n' "$registry_ids") <(printf '%s\n' "$mirror_ids"))"
  if [ -n "$only_registry" ] || [ -n "$only_mirror" ]; then
    [ -n "$only_registry" ] && printf 'doctrine registered but ABSENT from %s §10: %s\n' \
      "$MIRROR" "$(printf '%s' "$only_registry" | tr '\n' ' ')" >&2
    [ -n "$only_mirror" ] && printf 'row in %s §10 naming a doctrine NOT registered: %s\n' \
      "$MIRROR" "$(printf '%s' "$only_mirror" | tr '\n' ' ')" >&2
    report+=("✗ FAIL  <meta:mirror> — the registry and its ${MIRROR} §10 mirror disagree (see above)")
    fail=1
  else
    report+=("✓ PASS  <meta:mirror> — ${MIRROR} §10 lists exactly the ${#DOCTRINES[@]} registered doctrines")
  fi
fi

# ------------------------------------------------------------------ meta-check: the BOOK's count
# ⛔ A THIRD COPY OF THE ROSTER SIZE LIVES IN THE BOOK, AND IT WAS MEASURABLY STALE.
# `docs/book/src/gate-flow.md` publishes the number of registered doctrines twice — it is the
# reader-facing surface, and per the standing directive the book is the primary window into this
# project. Measured 2026-08-16 (`ENGINE-UNIVERSAL-SERVICES.29`(b)): the book said **19** while the
# registry held **20**. Nothing compared them: the mirror check above governs
# `DOCTRINE_ENFORCEMENT.md` only, so the book was free to drift, and it did — silently, and in the
# direction that makes the project look LESS guarded than it is.
# ⭐ MARKER-SCOPED, NOT SPELLING-MATCHED. The count is wrapped in `<!-- DOCTRINE-COUNT -->` … so
# prose around it can be rewritten freely without touching this check. Enumerating sentence
# spellings is exactly what made `LIVE-DOC-CURRENCY`'s instrument B measure one population as 10,
# then 16, then 18 — every miss silent in the passing direction.
# ⚠️ It asserts the COUNT, not the roster: the book is a narrative surface and is not asked to carry
# 21 ids. `DOCTRINE_ENFORCEMENT.md` §10 is where the id set is gated, above.
BOOK="docs/book/src/gate-flow.md"
if [ -f "$ROOT/$BOOK" ]; then
  book_counts="$(grep -o '<!-- DOCTRINE-COUNT -->\**[0-9]\{1,4\}' "$ROOT/$BOOK" | grep -o '[0-9]\{1,4\}$')"
  if [ -z "$book_counts" ]; then
    report+=("✗ FAIL  <meta:book-count> — ${BOOK} carries no <!-- DOCTRINE-COUNT --> marker. A check that"
             "        cannot inspect its subject must SAY SO, not pass: re-wrap the published count, or"
             "        the book is free to drift from the registry again (it was 19 against a registry of 20).")
    fail=1
  else
    bad_counts=""
    while IFS= read -r n; do
      [ "$n" = "${#DOCTRINES[@]}" ] || bad_counts="$bad_counts $n"
    done <<< "$book_counts"
    if [ -n "$bad_counts" ]; then
      printf 'book publishes doctrine count(s)%s but the registry holds %s — update %s\n' \
        "$bad_counts" "${#DOCTRINES[@]}" "$BOOK" >&2
      report+=("✗ FAIL  <meta:book-count> — ${BOOK} publishes a stale registered-doctrine count (see above)")
      fail=1
    else
      # ⛔ `printf '%s' | wc -l` counts NEWLINES, so it reads one short on a trailing-newline-free
      # string — it printed "1 marked site(s)" for two. Count the lines themselves.
      report+=("✓ PASS  <meta:book-count> — ${BOOK} publishes ${#DOCTRINES[@]}, matching the registry ($(printf '%s\n' "$book_counts" | grep -c .) marked site(s))")
    fi
  fi
else
  report+=("✗ FAIL  <meta:book-count> — ${BOOK} is missing, so the published doctrine count cannot be checked")
  fail=1
fi

printf '\n================ DOCTRINE ENFORCEMENT REPORT ================\n' >&2
for line in "${report[@]}"; do printf '  %s\n' "$line" >&2; done
printf '============================================================\n' >&2

# ------------------------------------------------------------------ scope note: what did NOT run
# ⭐ THIS TREE'S FOUNDING PRINCIPLE, APPLIED TO THE DRIVER ITSELF: *a check that cannot run must SAY
# SO, not return green.* Some registered enforcers judge the STAGED diff. Under the pre-commit hook
# that is the change being made; on a hosted `push` (E4) the index is EMPTY, so they exit 0 having
# evaluated nothing — and a green tick would otherwise imply they were satisfied.
# ⛔ DERIVED from each enforcer's own source, not hand-listed, so a staged-scope doctrine added later
# is named here by construction — the same reason E4 must call this driver rather than enumerate it.
# ⚠️ Honest limit: the classifier is a source-level heuristic (does the enforcer read the index?). A
# mislabel costs an inaccurate NAME in an informational note; it never changes a verdict.
staged_count="$(git -C "$ROOT" diff --cached --name-only 2>/dev/null | wc -l | tr -d ' ')"
if [ "${staged_count:-0}" -eq 0 ]; then
  vacuous=()
  for entry in "${DOCTRINES[@]}"; do
    IFS='|' read -r id _proves script <<< "$entry"
    if grep -qE -- '--cached|--staged|diff-index' "$ROOT/$script" 2>/dev/null; then
      vacuous+=("$id")
    fi
  done
  if [ "${#vacuous[@]}" -gt 0 ]; then
    printf 'scope: no staged changes — %d of %d doctrines are scoped to the STAGED diff and therefore\n' \
      "${#vacuous[@]}" "${#DOCTRINES[@]}" >&2
    printf '       evaluated NOTHING here: %s\n' "$(printf '%s ' "${vacuous[@]}")" >&2
    printf '       Their PASS above is vacuous, not evidence. They bind at commit time (E3) and on a\n' >&2
    printf '       hosted push only for whatever that push actually stages.\n' >&2
  fi
fi
if [ "$fail" -eq 0 ]; then
  printf 'doctrines: ALL %d enforced doctrines PASS.\n' "${#DOCTRINES[@]}" >&2
else
  printf 'doctrines: ✗ one or more doctrines FAILED — commit/merge blocked. Fix above, do not bypass.\n' >&2
fi
exit "$fail"
