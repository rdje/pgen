#!/usr/bin/env bash
# scripts/check_diagnostics_and_docpaths.sh
#
# DIAG-SEVERITY.4 (PGEN-DIAG-SEVERITY-0005) enforcement + the DOCPATH live-docs guard.
# Un-bypassable backstop (pre-commit + CI) for two standing principles:
#
#   (1) Severity (warning/error/fatal) must NEVER be gated by a trace/verbosity level —
#       so the always-on diagnostic mechanism (Severity + emit_diagnostic + pgen_warn!/
#       error!/fatal!) MUST exist, and no UNAMBIGUOUS severity (fatal/panic) may be
#       routed through the verbosity-gated trace. (feedback_severity_never_gated_by_verbosity)
#   (2) Every repo-INTERNAL file path in a LIVE/maintained doc surface must be
#       repo-root-RELATIVE, never a checkout-specific absolute path capturing a local home
#       dir. Surfaces: docs/book/src, docs/contracts, PGEN_USER_GUIDE.md, README.md,
#       docs/tasks, docs/decisions, KNOWLEDGE_MAP.md, docs/knowledge,
#       (LIVE_ACHIEVEMENT_STATUS.md was on this list until LIVE-MEANS-LIVE.1c3 deleted the file;
#       its successor surfaces docs/book/src/** and docs/tasks/** are already covered above.)
#       Append-only history (CHANGES.md/DEVELOPMENT_NOTES.md)
#       and repo-EXTERNAL paths (point outside the repo; no repo-relative form) are out of
#       scope. (PGEN-DOCPATH-0001 seeded book+contracts+guide+README; PGEN-DOCPATH-0002 /
#       leaf DOCPATH.1 extended the guarded surface set to the rest of the live docs.)
#
# Sound by design: it passes clean on the current tree (verified at authoring) and only
# flags the unambiguous anti-patterns, so it will not false-block ordinary commits.
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"
fail=0

# (1a) The always-on severity mechanism must be present (cannot be silently removed).
missing=()
for sym in 'pub enum Severity' 'pub fn emit_diagnostic' 'macro_rules! pgen_warn' \
           'macro_rules! pgen_error' 'macro_rules! pgen_fatal'; do
  grep -q "$sym" rust/src/ast_pipeline/mod.rs || missing+=("$sym")
done
if [ "${#missing[@]}" -ne 0 ]; then
  echo "diag-severity: FAIL — the always-on severity mechanism is missing from rust/src/ast_pipeline/mod.rs: ${missing[*]}" >&2
  fail=1
fi

# (1b) No UNAMBIGUOUS severity (fatal/panic) routed through the verbosity-gated trace.
# Best-effort tripwire for the most egregious masking (the soundness guarantee is the
# mechanism in 1a + the unit test); per-attempt info/debug breadcrumbs are allowed.
masked="$(grep -rniE '(pgen_trace|trace\(TraceLevel|trace_log\(TraceLevel|\.trace\()' rust/src 2>/dev/null | grep -iE 'fatal|panic' || true)"
if [ -n "$masked" ]; then
  echo "diag-severity: FAIL — a fatal/panic-severity message is routed through the verbosity-gated trace; use pgen_error!/pgen_fatal! (always-on):" >&2
  echo "$masked" >&2
  fail=1
fi

# (2) LIVE docs must use repo-root-relative paths (no repo-internal absolute path).
# Matches only paths INTO this repo (contain '/pgen/'); repo-external refs and append-only
# history (CHANGES.md/DEVELOPMENT_NOTES.md) are deliberately out of scope.
absolute="$(git grep -nIE '/Users/[^ )`]*/pgen/' -- \
  'docs/book/src/**' 'docs/contracts/**' 'PGEN_USER_GUIDE.md' 'README.md' \
  'docs/tasks/**' 'docs/decisions/**' 'KNOWLEDGE_MAP.md' 'docs/knowledge/**' \
  2>/dev/null || true)"
if [ -n "$absolute" ]; then
  echo "docpath: FAIL — a LIVE doc carries a repo-internal ABSOLUTE path; make it repo-root-relative:" >&2
  echo "$absolute" >&2
  fail=1
fi

if [ "$fail" -ne 0 ]; then
  exit 1
fi
echo "diagnostics+docpaths: OK"
