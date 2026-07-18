#!/usr/bin/env bash
# check_destructive_target_guard.sh — DESTRUCTIVE-TARGET-GUARD doctrine check.
#
# Doctrine (director directive 2026-07-19; incident 2026-07-18, OPS-MEMSAFE.3):
# a destructive build target (one that deletes generated artifacts or runs
# `cargo clean`) must REFUSE unless explicitly confirmed via
# PGEN_CONFIRM_CLEAN=1, and no innocuous-sounding alias may route into the
# destructive family. The 2026-07-18 incident: `make annotation_parsers`
# aliased `return_semantic_parsers`, whose `clean` dep deleted every generated
# artifact AND the entire rust/target/ (102.8 GiB, incl. every preserved perf
# probe binary).
#
# Archetype: STRUCTURAL (re-derives the invariant from the Makefile text).
# Contract: exit 0 = holds; nonzero = breach, message on stderr. Deterministic,
# read-only, repo-root-resolved.
set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MAKEFILE="$ROOT/rust/Makefile"
fail() { echo "DESTRUCTIVE-TARGET-GUARD: $*" >&2; exit 1; }

[ -f "$MAKEFILE" ] || fail "rust/Makefile not found"

# (1) The `clean:` recipe must carry the PGEN_CONFIRM_CLEAN refusal BEFORE any
#     destructive line. Extract the clean recipe block (from the `clean:` rule
#     line to the next non-indented, non-comment line).
recipe="$(awk '/^clean:/{f=1; next} f && /^[^\t#]/{exit} f{print}' "$MAKEFILE")"
[ -n "$recipe" ] || fail "could not extract the clean: recipe from rust/Makefile"
guard_line="$(printf '%s\n' "$recipe" | grep -n 'PGEN_CONFIRM_CLEAN' | head -1 | cut -d: -f1)"
destr_line="$(printf '%s\n' "$recipe" | grep -nE 'rm -f|cargo clean' | head -1 | cut -d: -f1)"
[ -n "$guard_line" ] || fail "the clean: recipe has NO PGEN_CONFIRM_CLEAN guard (the 2026-07-18 incident class is open again)"
[ -n "$destr_line" ] || fail "the clean: recipe has no destructive line — the recipe moved; re-point this check"
[ "$guard_line" -lt "$destr_line" ] || fail "the PGEN_CONFIRM_CLEAN guard sits AFTER the destructive line in clean: — it guards nothing"

# (2) `annotation_parsers` must NOT route into the destructive family.
ap_deps="$(awk -F: '/^annotation_parsers:/{print $2}' "$MAKEFILE" | tr -s ' ')"
case " $ap_deps " in
  *" return_semantic_parsers "*|*" clean "*|*" clean-all "*)
    fail "annotation_parsers depends on the destructive family again ($ap_deps) — the de-fanged alias regressed" ;;
esac

# (3) Only the explicit allowlist may depend on `clean`/`clean-all`. Every
#     allowlisted target is still guarded TRANSITIVELY (its clean dep refuses
#     unconfirmed), so the allowlist is about naming honesty, not bypass.
allow='^(clean-all|rebuild|return_semantic_parsers|bootstrap-test)$'
bad="$(awk -F: '/^[A-Za-z0-9_.-]+:/ && $1 != ".PHONY" {t=$1; d=$2; n=split(d,a," "); for(i=1;i<=n;i++) if(a[i]=="clean"||a[i]=="clean-all") print t}' "$MAKEFILE" \
      | grep -vE "$allow" || true)"
[ -z "$bad" ] || fail "unexpected target(s) depend on clean/clean-all: $bad — extend the allowlist ONLY with a deliberate, documented decision"

echo "DESTRUCTIVE-TARGET-GUARD: ok (clean guarded; alias clean; dep allowlist exact)"
exit 0
