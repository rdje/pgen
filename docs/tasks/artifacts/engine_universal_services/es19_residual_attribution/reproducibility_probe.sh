#!/usr/bin/env bash
# ENGINE-UNIVERSAL-SERVICES.29 — is `generated/` what HEAD's source produces?
#
# ⛔⛔ THIS PROBE IS **RED TODAY, BY DESIGN**. It reports a real, open defect: the two annotation
# parsers in `generated/` carry a line the tracked code generator cannot emit, so they cannot be
# re-derived from HEAD. `.29` acceptance (a) is what turns it green; until then a nonzero exit is
# the correct verdict and not a broken probe.
#
# WHY THIS AND NOT AN EXISTING GATE. Nothing in the repository asks whether the artifacts ON DISK are
# what the tracked source produces:
#   - a recorded hash proves only that an artifact has not moved since someone recorded the hash;
#   - `fixed_point_gate` proves REGENERATION CONVERGES across its own cycles — it overwrites whatever
#     was on disk in cycle 1, so a stale pre-existing artifact is invisible to it by construction;
#   - a green build proves the artifact COMPILES, not that it is current.
# Re-derive and diff is the only thing that answers the question.
#
# ⭐ IT NORMALISES THE EMBEDDED `-o` PATH BEFORE COMPARING. A generated parser writes its own output
# destination into the emitted source once per rule-entry site (TOOLBOX 5.6), so a re-derivation
# written anywhere but the canonical path differs in SIZE for reasons that have nothing to do with
# the source. This probe re-derives into a mimic tree so the `-o` string is byte-identical to the
# real invocation, and asserts that it is — rather than trusting that it is.
#
# SCOPE (stated, not implied): the ANNOTATION PAIR only. Those are the two artifacts `rust/src/lib.rs`
# includes by literal path and the annotation backend links, i.e. the pair that participates in
# generating every other parser — so they are the ones whose staleness propagates. Extending this to
# the eight family parsers is `.29` acceptance (a), and it costs a full regeneration rather than the
# ~30 s this does.
#
# HOW:  bash docs/tasks/artifacts/engine_universal_services/es19_residual_attribution/reproducibility_probe.sh
# EXIT: 0 = every checked artifact re-derives byte-identically · 1 = at least one diverges · 2 = the
#       probe could not run (missing input, build failure, or a `-o` spelling it could not reproduce)

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../../.." && pwd)"
cd "$ROOT"

WORK="$ROOT/rust/target/es19_reproducibility"
BOOT_TARGET="$ROOT/rust/target/es19_boot"
BOOT="$BOOT_TARGET/debug/ast_pipeline_bootstrap"

diverged=0
note() { printf '  %s\n' "$1"; }

printf '\nES19-REPRODUCIBILITY: is generated/ what HEAD produces?\n'
# ⛔ Truncate by CHARACTERS, not bytes. `cut -c` AND awk's `substr` are both byte-oriented under this
# host's default locale, and commit subjects here routinely carry multi-byte marks, so either one
# prints a mojibake tail. Bash substring expansion IS character-aware once the locale says UTF-8.
HEAD_LINE=$(LC_ALL=en_US.UTF-8 bash -c 's=$(git log -1 --format="%h %s"); [ ${#s} -gt 90 ] && printf "%s…" "${s:0:90}" || printf "%s" "$s"')
printf 'HEAD: %s\n\n' "$HEAD_LINE"

# ── build the generator from HEAD, into its own target dir so the tree's binaries are untouched ────
printf 'building ast_pipeline_bootstrap from HEAD...\n'
( cd "$ROOT/rust" && CARGO_TARGET_DIR="$BOOT_TARGET" cargo build \
    --bin ast_pipeline_bootstrap --no-default-features --features bootstrap ) >/dev/null 2>&1 \
  || { printf 'es19-repro: the bootstrap generator does not BUILD from HEAD — that is a larger finding than this probe measures\n' >&2; exit 2; }
[ -x "$BOOT" ] || { printf 'es19-repro: %s missing after a successful build\n' "$BOOT" >&2; exit 2; }

rm -rf "$WORK"
mkdir -p "$WORK/root/generated" "$WORK/root/rust"

# check <family>
#   Re-derives generated/<family>_parser.rs through the tracked recipe
#   ($(RUST_GENERATOR_BOOTSTRAP) = --generate-parser --bootstrap-mode --eliminate-left-recursion),
#   from cwd `<mimic>/rust` with `-o ../generated/…` — byte-identical to what `rust/Makefile` passes.
check() {
  local fam="$1"
  local out="../generated/${fam}_parser.rs"
  local live="$ROOT/generated/${fam}_parser.rs"
  local json="$ROOT/generated/${fam}.json"
  local fresh="$WORK/root/generated/${fam}_parser.rs"

  [ -f "$live" ] || { printf 'es19-repro: %s is absent — regenerate generated/ first\n' "$live" >&2; exit 2; }
  [ -f "$json" ] || { printf 'es19-repro: %s is absent — regenerate generated/ first\n' "$json" >&2; exit 2; }

  ( cd "$WORK/root/rust" && "$BOOT" "$json" --generate-parser --bootstrap-mode \
      --eliminate-left-recursion -o "$out" ) >/dev/null 2>&1 \
    || { printf 'es19-repro: codegen FAILED for %s\n' "$fam" >&2; exit 2; }

  # The `-o` spelling must be the one the live artifact carries, or the comparison is meaningless.
  local live_sites fresh_sites
  live_sites=$(grep -oF "$out" "$live" | wc -l | tr -d ' ')
  fresh_sites=$(grep -oF "$out" "$fresh" | wc -l | tr -d ' ')
  if [ "$live_sites" = 0 ] || [ "$live_sites" != "$fresh_sites" ]; then
    printf 'es19-repro: cannot compare %s — embedded -o sites live=%s fresh=%s (TOOLBOX 5.6)\n' \
      "$fam" "$live_sites" "$fresh_sites" >&2
    exit 2
  fi

  local a b
  a=$(shasum -a 256 "$live"  | cut -d' ' -f1)
  b=$(shasum -a 256 "$fresh" | cut -d' ' -f1)
  if [ "$a" = "$b" ]; then
    printf '  ✓ %-32s reproduces from HEAD (%s sites, %s)\n' "$fam" "$live_sites" "${a:0:12}…"
  else
    diverged=$((diverged + 1))
    printf '  ✗ %-32s DOES NOT reproduce from HEAD\n' "$fam"
    note "live  $(wc -c < "$live" | tr -d ' ') B  $a"
    note "fresh $(wc -c < "$fresh" | tr -d ' ') B  $b"
    note "diff (live → fresh):"
    diff -u0 "$live" "$fresh" | sed -n '3,23p' | sed 's/^/      /'
  fi
}

check return_annotation
check semantic_annotation

printf '\n'
if [ "$diverged" -eq 0 ]; then
  printf 'ES19-REPRODUCIBILITY: every checked artifact re-derives byte-identically from HEAD\n'
else
  printf 'ES19-REPRODUCIBILITY: %d artifact(s) do NOT re-derive from HEAD — see ENGINE-UNIVERSAL-SERVICES.29\n' "$diverged" >&2
fi
rm -rf "$WORK"
exit $(( diverged > 0 ? 1 : 0 ))
