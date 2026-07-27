#!/usr/bin/env bash
# rust/scripts/generated_clippy_correctness_gate.sh
# GENERATED-LINT-CORRECTNESS.3 — keep the generated-parser CORRECTNESS-lint count at 0 by
# RUNNING the check, not by hoping.
#
# WHY THIS EXISTS (measured, not assumed). `.1` + `.2` took the generated stage from 291
# `deny`-by-default clippy errors to 0. Nothing re-runs that: `PGEN_CLIPPY_GENERATED_STRICT=1`
# is a manual opt-in env var, and a repo-wide sweep found it is set by NO gate, NO aggregate
# and NO CI workflow — only by prose in COMMIT.md. So the 0 was unguarded the moment it landed.
#
# WHAT IT GATES ON. The `clippy::correctness` CATEGORY only. The generated stage also carries
# ~78.8k style/complexity/perf warnings over 210.8 MB of emitted code; gating on those would be
# noise, not signal, and would make this gate unrunnable for exactly the reason the 291 errors
# made the strict stage unrunnable. The pinned roster lives in the tracked contract so the
# subset cannot silently narrow.
#
# ⛔ ANTI-VACUITY — the part that matters. `rust/build.rs` sets each `has_generated_<n>_parser`
# cfg ONLY when the artifact `is_file()`, and `generated/` is untracked. So in a clean checkout,
# on a CI runner, or inside the `ci_workflow_local_gate` export dir (which copies `git ls-files`
# only), `--features generated_parsers` compiles with ZERO cfg-guarded generated parsers and a
# naive "lint the generated stage" step would lint nothing and exit 0. This gate REFUSES in that
# situation instead of returning green — a check that cannot SEE must SAY SO. Proof that the
# artifacts really were compiled in comes from cargo's own `build-script-executed` JSON message,
# which carries the emitted cfg list and is replayed even from a warm cache.
#
# Exit codes:
#   0  pass — artifacts present, compiled in, roster intact, 0 correctness findings
#   1  FAIL — correctness findings were reported (the defect this gate exists to catch)
#   2  REFUSE — the gate could not run soundly (missing artifacts, artifacts not compiled in,
#               roster drift, missing tooling). Never confused with a pass.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
ROOT_DIR="$(cd "${RUST_DIR}/.." && pwd)"

CONTRACT="${PGEN_GENERATED_CLIPPY_CONTRACT:-${RUST_DIR}/test_data/grammar_quality/generated_clippy_correctness_contract_v0.json}"
REPORT_DIR="${PGEN_GENERATED_CLIPPY_REPORT_DIR:-${RUST_DIR}/target/generated_clippy_correctness_gate}"
SUMMARY_TXT="${REPORT_DIR}/summary.txt"
CLIPPY_JSON="${REPORT_DIR}/clippy_messages.json"

# Test seam: feed a pre-recorded clippy JSON stream instead of running cargo. Used by the
# leaf's RED/GREEN driver to exercise the finding census without a 12 GB rebuild.
FIXTURE="${PGEN_GENERATED_CLIPPY_FIXTURE:-}"

# --policy-only: run the checks that need no cargo invocation (artifact presence + roster
# integrity) and stop. This is what the commit workflow uses: `clippy_on_rust_change.sh` already
# runs the generated stage, and clippy's correctness lints are deny-by-default, so that stage
# ALREADY fails on a correctness finding once PGEN_CLIPPY_GENERATED_STRICT=1. What it cannot see
# is a lint being DEMOTED out of deny-by-default — and that is precisely what roster integrity
# catches, for free. So the commit workflow gets the full guarantee without paying for a second
# ~12 GB clippy pass; the explicit-deny run stays available as `make generated_clippy_correctness_gate`.
POLICY_ONLY=0
[ "${1:-}" = "--policy-only" ] && POLICY_ONLY=1

mkdir -p "${REPORT_DIR}"
: >"${SUMMARY_TXT}"

# Every line this gate emits is prefixed so it is a greppable, verbatim tool signature —
# the same discipline as `CERTIFICATE-COVERAGE:` in the certificate-coverage lane.
say() { printf 'GENERATED-CLIPPY-CORRECTNESS: %s\n' "$*" | tee -a "${SUMMARY_TXT}"; }
say_err() { printf 'GENERATED-CLIPPY-CORRECTNESS: %s\n' "$*" | tee -a "${SUMMARY_TXT}" >&2; }

refuse() {
  say_err "⛔ REFUSE — $*"
  say_err "⛔ a REFUSE is NOT a pass: the gate could not run soundly, so it reports nothing rather than green."
  exit 2
}

require_tool() {
  command -v "$1" >/dev/null 2>&1 || refuse "required tool not found on PATH: $1"
}

require_tool jq
require_tool cargo

[ -f "${CONTRACT}" ] || refuse "contract not found: ${CONTRACT}"
jq -e . "${CONTRACT}" >/dev/null 2>&1 || refuse "contract is not valid JSON: ${CONTRACT}"

say "contract: ${CONTRACT#"${ROOT_DIR}/"}"
say "report_dir: ${REPORT_DIR#"${ROOT_DIR}/"}"

# ---------------------------------------------------------------------------
# 1. ANTI-VACUITY (a): every REQUIRED artifact is present and non-empty on disk.
# ---------------------------------------------------------------------------
mapfile -t required_paths < <(jq -r '.generated_artifacts[] | select(.required) | .path' "${CONTRACT}")
mapfile -t optional_paths < <(jq -r '.generated_artifacts[] | select(.required | not) | .path' "${CONTRACT}")
[ "${#required_paths[@]}" -gt 0 ] || refuse "contract lists no required generated artifacts — nothing to gate"

missing=()
for rel in "${required_paths[@]}"; do
  abs="${ROOT_DIR}/${rel}"
  if [ ! -s "${abs}" ]; then
    missing+=("${rel}")
  fi
done

if [ "${#missing[@]}" -gt 0 ]; then
  say_err "the generated artifacts this gate exists to lint are ABSENT (${#missing[@]} of ${#required_paths[@]}):"
  for rel in "${missing[@]}"; do say_err "  missing: ${rel}"; done
  say_err "generated/ is untracked by repository policy — regenerate with the per-grammar targets,"
  say_err "e.g. 'make -C rust SHELL=/bin/bash focus_systemverilog' (see README.md 'Key Project Paths')."
  refuse "cannot lint generated parsers that are not on disk"
fi
say "artifact-presence: ${#required_paths[@]}/${#required_paths[@]} required artifacts present"

present_optional=()
for rel in "${optional_paths[@]}"; do
  if [ -s "${ROOT_DIR}/${rel}" ]; then present_optional+=("${rel}"); fi
done
say "artifact-presence: ${#present_optional[@]}/${#optional_paths[@]} optional artifacts present (linted when present)"

# ---------------------------------------------------------------------------
# 2. ROSTER INTEGRITY: every pinned lint is still a member of the installed
#    clippy's `clippy::correctness` group. This is the ANTI-NARROWING check —
#    denying the group alone would silently stop covering a lint that clippy
#    reclassifies out of it.
# ---------------------------------------------------------------------------
require_tool clippy-driver
LIVE_ROSTER="${REPORT_DIR}/live_correctness_roster.txt"
clippy-driver -Whelp 2>&1 \
  | sed -n '/^Lint groups loaded by this crate:/,$p' \
  | awk '{$1=$1};1' \
  | grep -E '^clippy::correctness ' \
  | sed 's/^clippy::correctness //' \
  | tr ',' '\n' \
  | awk '{$1=$1};1' \
  | grep -E '^clippy::' \
  | sed 's/^clippy:://; s/-/_/g' \
  | sort -u >"${LIVE_ROSTER}" || true

[ -s "${LIVE_ROSTER}" ] || refuse "could not read the installed clippy's clippy::correctness group (clippy-driver -Whelp produced nothing)"

PINNED_ROSTER="${REPORT_DIR}/pinned_correctness_roster.txt"
jq -r '.lint_policy.correctness_roster[]' "${CONTRACT}" | sort -u >"${PINNED_ROSTER}"

pinned_count="$(wc -l <"${PINNED_ROSTER}" | tr -d ' ')"
live_count="$(wc -l <"${LIVE_ROSTER}" | tr -d ' ')"
declared_count="$(jq -r '.lint_policy.correctness_roster_size' "${CONTRACT}")"

[ "${pinned_count}" = "${declared_count}" ] || \
  refuse "contract self-inconsistent: correctness_roster_size=${declared_count} but the roster holds ${pinned_count} names"

mapfile -t departed < <(comm -23 "${PINNED_ROSTER}" "${LIVE_ROSTER}")
if [ "${#departed[@]}" -gt 0 ]; then
  say_err "roster drift — ${#departed[@]} pinned lint(s) are NO LONGER in this clippy's correctness group:"
  for l in "${departed[@]}"; do say_err "  departed: clippy::${l}"; done
  say_err "that is exactly the SILENT NARROWING this pin exists to make loud. Re-adjudicate the"
  say_err "subset in the contract (and say why) rather than deleting the names."
  refuse "pinned correctness roster no longer matches the installed clippy"
fi

# Both anchors must still be correctness lints — they ARE the 291 that opened this tree.
while IFS= read -r anchor; do
  grep -qx "${anchor}" "${LIVE_ROSTER}" || \
    refuse "anchor lint clippy::${anchor} left clippy::correctness — this gate no longer covers the defect class it was built for"
done < <(jq -r '.lint_policy.roster_anchor_lints[]' "${CONTRACT}")

mapfile -t added < <(comm -13 "${PINNED_ROSTER}" "${LIVE_ROSTER}")
say "roster-integrity: ${pinned_count} pinned lints, all still in clippy::correctness (live group: ${live_count})"
if [ "${#added[@]}" -gt 0 ]; then
  say "roster-integrity: NOTE — ${#added[@]} lint(s) joined clippy::correctness since this contract was pinned;"
  say "roster-integrity: they ARE denied (the group is denied too), but are not yet pinned by name:"
  for l in "${added[@]}"; do say "  new-in-group: clippy::${l}"; done
fi

CLIPPY_VERSION="$(clippy-driver --version 2>/dev/null || echo 'unknown')"
say "clippy: ${CLIPPY_VERSION}"

if [ "${POLICY_ONLY}" -eq 1 ]; then
  say "✅ POLICY-ONLY PASS — artifacts present and the pinned correctness roster is intact."
  say "policy-only: the FINDING count is not measured here; the caller's own generated clippy stage"
  say "policy-only: measures it (correctness lints are deny-by-default). Full explicit-deny run:"
  say "policy-only:   make -C rust SHELL=/bin/bash generated_clippy_correctness_gate"
  exit 0
fi

# ---------------------------------------------------------------------------
# 3. RUN the lint, denying the group AND every pinned name.
# ---------------------------------------------------------------------------
lint_args=(-A warnings -D clippy::correctness)
while IFS= read -r l; do lint_args+=(-D "clippy::${l}"); done <"${PINNED_ROSTER}"

FEATURES="$(jq -r '.cargo_invocation.features' "${CONTRACT}")"

if [ -n "${FIXTURE}" ]; then
  say "⚠️  FIXTURE MODE — reading a pre-recorded clippy JSON stream instead of running cargo:"
  say "⚠️  ${FIXTURE}"
  say "⚠️  this exercises the census/verdict logic ONLY; it is NOT a lint of the real tree."
  [ -f "${FIXTURE}" ] || refuse "fixture not found: ${FIXTURE}"
  cp "${FIXTURE}" "${CLIPPY_JSON}"
  clippy_rc=0
else
  say "running: cargo clippy --all-targets --features ${FEATURES} -- -A warnings -D clippy::correctness -D <${pinned_count} pinned lints>"
  clippy_rc=0
  (
    cd "${RUST_DIR}"
    cargo clippy --manifest-path "${RUST_DIR}/Cargo.toml" --all-targets \
      --features "${FEATURES}" --message-format=json -- "${lint_args[@]}"
  ) >"${CLIPPY_JSON}" 2>"${REPORT_DIR}/clippy_stderr.log" || clippy_rc=$?
fi

# ---------------------------------------------------------------------------
# 4. ANTI-VACUITY (b): cargo's OWN testimony that the artifacts were compiled in.
#    `build-script-executed` carries the cfg list build.rs emitted, and cargo
#    replays it even from a warm cache — so this is not a re-reading of the disk.
# ---------------------------------------------------------------------------
CFG_CENSUS="${REPORT_DIR}/build_script_cfgs.txt"
jq -r 'select(.reason == "build-script-executed") | .cfgs[]?' "${CLIPPY_JSON}" 2>/dev/null \
  | sort -u >"${CFG_CENSUS}" || true

not_compiled=()
while IFS=$'\t' read -r rel cfg; do
  [ -n "${cfg}" ] && [ "${cfg}" != "null" ] || continue
  grep -qx "${cfg}" "${CFG_CENSUS}" 2>/dev/null || not_compiled+=("${rel} (${cfg})")
done < <(jq -r '.generated_artifacts[] | select(.required) | select(.cfg_guarded) | [.path, .cfg] | @tsv' "${CONTRACT}")

if [ "${#not_compiled[@]}" -gt 0 ]; then
  say_err "artifacts are on disk but cargo did NOT compile them into the linted unit (${#not_compiled[@]}):"
  for a in "${not_compiled[@]}"; do say_err "  not-compiled-in: ${a}"; done
  say_err "without them the lint run is VACUOUS — it would report 0 findings because it saw no generated code."
  refuse "cfg census does not confirm the generated parsers were linted"
fi
guarded_required="$(jq -r '[.generated_artifacts[] | select(.required) | select(.cfg_guarded)] | length' "${CONTRACT}")"
unguarded_required="$(jq -r '[.generated_artifacts[] | select(.required) | select(.cfg_guarded | not)] | length' "${CONTRACT}")"
say "cfg-census: ${guarded_required}/${guarded_required} cfg-guarded required artifacts confirmed compiled in by cargo"
say "cfg-census: ${unguarded_required} required artifact(s) are include!d unguarded — their absence is a hard compile error, not a silent skip"

# ---------------------------------------------------------------------------
# 5. CENSUS the correctness findings.
# ---------------------------------------------------------------------------
FINDINGS="${REPORT_DIR}/correctness_findings.txt"
jq -r '
  select(.reason == "compiler-message")
  | .message
  | select(.level == "error")
  | . as $m
  | ($m.code.code // "<no-code>") as $code
  | ($m.spans // [] | map(select(.is_primary)) | .[0] // {}) as $s
  | [$code, (($s.file_name) // "<no-file>"), (($s.line_start|tostring) // "0"), $m.message]
  | @tsv
' "${CLIPPY_JSON}" 2>/dev/null | sort >"${FINDINGS}" || true

finding_count="$(wc -l <"${FINDINGS}" | tr -d ' ')"
generated_finding_count="$(awk -F'\t' '$2 ~ /generated\// {n++} END {print n+0}' "${FINDINGS}")"

expected_total="$(jq -r '.expected.correctness_findings_total' "${CONTRACT}")"
expected_generated="$(jq -r '.expected.correctness_findings_in_generated' "${CONTRACT}")"

say "findings: total=${finding_count} (expected ${expected_total})"
say "findings: in generated/=${generated_finding_count} (expected ${expected_generated})"

if [ "${finding_count}" -gt 0 ]; then
  say_err "correctness findings (lint\tfile\tline\tmessage):"
  head -n 50 "${FINDINGS}" | while IFS= read -r line; do say_err "  ${line}"; done
  [ "${finding_count}" -gt 50 ] && say_err "  … $((finding_count - 50)) more in ${FINDINGS}"
  say_err "⛔ FAIL — the generated-parser correctness floor has drifted. These are clippy::correctness"
  say_err "⛔ lints; per the -0001 adjudication the fix is to change the EMISSION at its codegen site,"
  say_err "⛔ never to add an #[allow] or lower the lint."
  exit 1
fi

# A nonzero cargo exit with no censused finding means the run itself broke (a compile error,
# a bad lint name). That is a REFUSE, not a pass — the gate learned nothing.
if [ "${clippy_rc}" -ne 0 ]; then
  say_err "cargo clippy exited ${clippy_rc} but no correctness finding was censused — the run itself failed."
  tail -n 40 "${REPORT_DIR}/clippy_stderr.log" >&2 2>/dev/null || true
  refuse "lint run did not complete (see ${REPORT_DIR#"${ROOT_DIR}/"}/clippy_stderr.log)"
fi

say "✅ PASS — 0 clippy::correctness findings across ${#required_paths[@]} required + ${#present_optional[@]} optional generated artifacts"
exit 0
