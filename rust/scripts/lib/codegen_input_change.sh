#!/usr/bin/env bash
# rust/scripts/lib/codegen_input_change.sh — THE single definition of
# "this change set touches a CODEGEN INPUT".
#
# ⛔ WHY THIS IS A SHARED LIBRARY AND NOT A GLOB COPIED INTO EACH CALLER
# (CI-PARITY-GATE-ROT.43, 2026-08-23). The same missing trigger was measured and
# logged by THREE separate leaves over fourteen days — GENERATED-LINT-CORRECTNESS.11
# (2026-08-09), CI-PARITY-GATE-ROT.43 (2026-08-22) and GRAMMAR-WELLFORMED.H.20.2
# (2026-08-23) — and the underlying defect reproduced FIVE times without being fixed,
# because each lane logged it against its own gate rather than against the shared
# predicate none of them owned. A second copy of this pattern list is how that recurs:
# the next consumer must WIRE ITSELF HERE, not re-derive the globs.
#
# ⛔ THE TRAP THIS EXISTS TO AVOID, stated so a future editor does not "simplify" it back:
# `generated/` is GITIGNORED (.gitignore:24), so `git ls-files --others --exclude-standard`
# drops it and a `generated/*.rs` pattern can never fire from an untracked scan. A trigger
# keyed on the OUTPUT is therefore unreachable; this predicate keys on the tracked INPUT
# (`grammars/*.ebnf`) that PRODUCES that output, which is observable.
#
# Usage (source it, do not execute):
#   . "$ROOT_DIR/rust/scripts/lib/codegen_input_change.sh"
#   if pgen_codegen_input_changed "$ROOT_DIR"; then ... fi
#   pgen_codegen_input_changed_paths "$ROOT_DIR"   # prints the matching paths
#
# Both functions consider the working-tree diff, the staged diff and untracked files, so
# they answer "is a codegen input in play right now?" both before and after `git add`.

# The tracked codegen INPUTS. A change to any of these can change generated Rust.
# ⚠️ Keep this list in ONE place. If you are about to add a glob in a caller, add it here.
pgen_codegen_input_globs() {
    printf '%s\n' \
        'grammars/*.ebnf'
}

pgen_codegen_input_changed_paths() {
    local root="${1:-.}" path glob
    local -a changed globs
    mapfile -t globs < <(pgen_codegen_input_globs)
    mapfile -t changed < <(
        {
            git -C "$root" diff --name-only
            git -C "$root" diff --cached --name-only
            git -C "$root" ls-files --others --exclude-standard
        } 2>/dev/null | awk 'NF' | sort -u
    )
    for path in "${changed[@]}"; do
        for glob in "${globs[@]}"; do
            # shellcheck disable=SC2053  # RHS is a glob on purpose
            if [[ "$path" == $glob ]]; then
                printf '%s\n' "$path"
                break
            fi
        done
    done
}

pgen_codegen_input_changed() {
    local root="${1:-.}"
    [[ -n "$(pgen_codegen_input_changed_paths "$root")" ]]
}
