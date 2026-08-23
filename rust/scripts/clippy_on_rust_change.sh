#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUST_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
ROOT_DIR="$(cd "$RUST_DIR/.." && pwd)"
STATE_DIR="$RUST_DIR/target/clippy_gate"
LOG_DIR="$STATE_DIR/logs"

FORCE_RUN="${PGEN_CLIPPY_FORCE:-0}"
if [[ "${1:-}" == "--force" ]]; then
    FORCE_RUN=1
fi

# GENERATED-LINT-CORRECTNESS.3 — the generated stage is now STRICT BY DEFAULT.
#
# It used to default to 0, and a repo-wide sweep found that NOTHING ever set it to 1: no gate, no
# aggregate, no CI workflow — only prose in COMMIT.md. So when `.1` + `.2` took the generated
# correctness-lint count 291 -> 0, nothing was left to keep it there. Correctness lints are
# deny-by-default in clippy, so with this flag on, THIS stage is what holds the 0.
#
# Set PGEN_CLIPPY_GENERATED_STRICT=0 to deliberately drop back to advisory mode. That is a real
# choice with a real cost, so it is loud (below) rather than silent.
GENERATED_STRICT="${PGEN_CLIPPY_GENERATED_STRICT:-1}"

mkdir -p "$LOG_DIR"

run_stage() {
    local stage_name="$1"
    shift
    local log_file="$LOG_DIR/${stage_name}.log"
    echo "==> $stage_name"
    if "$@" >"$log_file" 2>&1; then
        echo "    ok ($log_file)"
        return 0
    fi
    echo "    fail ($log_file)" >&2
    tail -n 40 "$log_file" >&2 || true
    return 1
}

# CI-PARITY-GATE-ROT.43 — the SHARED codegen-input predicate (one definition, named consumers).
. "$ROOT_DIR/rust/scripts/lib/codegen_input_change.sh"

should_run=0
if [[ "$FORCE_RUN" -eq 1 ]]; then
    should_run=1
else
    mapfile -t changed_paths < <(
        {
            git -C "$ROOT_DIR" diff --name-only
            git -C "$ROOT_DIR" diff --cached --name-only
            git -C "$ROOT_DIR" ls-files --others --exclude-standard
        } | awk 'NF' | sort -u
    )

    # CI-PARITY-GATE-ROT.43 slice 1 — the CODEGEN-INPUT trigger, from the ONE shared definition.
    # A pure grammar edit is exactly what regenerates a parser, and it was invisible here: the
    # `generated/*.rs` pattern below CANNOT fire (generated/ is gitignored, so an untracked scan
    # drops it), so the branch that looked like coverage was unreachable and a grammar-only commit
    # skipped the flow at exit 0. Measured five times across three lanes before it was fixed.
    # ⛔ Add new codegen inputs to rust/scripts/lib/codegen_input_change.sh, NOT to the list below.
    if [[ "$should_run" -ne 1 ]] && pgen_codegen_input_changed "$ROOT_DIR"; then
        should_run=1
    fi

    for path in "${changed_paths[@]}"; do
        # Rust/generated Rust, the manifests, and — since GENERATED-LINT-CORRECTNESS.3 — the two
        # files that GOVERN the generated-parser correctness policy. A change to the pinned lint
        # roster or to the gate that enforces it must re-verify, exactly as a code change does.
        # ⚠️ `generated/*.rs` is RETAINED, not relied on: it can only fire via a deliberate
        # `git add -f`, which COMMIT.md forbids ("never `git add generated/…`"). It is kept because
        # removing it would drop that edge case, NOT because it provides coverage — the coverage is
        # the codegen-input check above. Replacing it with a content hash of generated/ against the
        # last-linted state is still OWED (CI-PARITY-GATE-ROT.43 hole 1, slice 2).
        if [[ "$path" == rust/*.rs \
           || "$path" == generated/*.rs \
           || "$path" == rust/Cargo.toml \
           || "$path" == rust/Cargo.lock \
           || "$path" == rust/scripts/generated_clippy_correctness_gate.sh \
           || "$path" == rust/test_data/grammar_quality/generated_clippy_correctness_contract_v0.json ]]; then
            should_run=1
            break
        fi
    done
fi

if [[ "$should_run" -ne 1 ]]; then
    echo "No Rust/generated Rust changes detected; skipping clippy flow."
    exit 0
fi

echo "Running clippy flow (Rust files amended/generated detected)."

run_stage "clippy_source_all_targets" \
    cargo clippy --manifest-path "$RUST_DIR/Cargo.toml" --all-targets

# The generated-parser CORRECTNESS policy, checked WITHOUT a second clippy pass.
# `--policy-only` verifies (a) the generated artifacts this flow is about to lint are actually on
# disk, and (b) every lint pinned in the tracked contract is still a member of clippy::correctness.
# (b) is the part the clippy run itself CANNOT see: if a future clippy demotes `eq_op` out of
# deny-by-default, the generated stage would quietly stop failing on it and the 0 would rot
# silently. A REFUSE here (exit 2) is not a pass — it means the check could not run soundly.
CORRECTNESS_GATE="$RUST_DIR/scripts/generated_clippy_correctness_gate.sh"
if [[ -x "$CORRECTNESS_GATE" ]]; then
    echo "==> generated_clippy_correctness_policy"
    if ! "$CORRECTNESS_GATE" --policy-only; then
        echo "Generated-parser correctness POLICY check failed — see the output above." >&2
        echo "Full explicit-deny run: make -C rust SHELL=/bin/bash generated_clippy_correctness_gate" >&2
        exit 1
    fi
else
    echo "warning: $CORRECTNESS_GATE not found/executable — correctness policy NOT verified" >&2
fi

if run_stage "clippy_generated_all_targets" \
    cargo clippy --manifest-path "$RUST_DIR/Cargo.toml" --all-targets --features generated_parsers,ebnf_dual_run; then
    echo "Generated-parser clippy stage: pass"
else
    if [[ "$GENERATED_STRICT" == "1" ]]; then
        echo "Generated-parser clippy stage FAILED (strict — the default since GENERATED-LINT-CORRECTNESS.3)." >&2
        echo "clippy's correctness lints are deny-by-default, so this is how the generated-parser" >&2
        echo "correctness floor (0 findings) is held. Per the -0001 adjudication the fix is to change" >&2
        echo "the EMISSION at its codegen site — never an #[allow], never lowering the lint." >&2
        echo "See $LOG_DIR/clippy_generated_all_targets.log" >&2
        exit 1
    fi
    echo "⚠️  Generated-parser clippy stage failed and PGEN_CLIPPY_GENERATED_STRICT=0 was set" >&2
    echo "⚠️  EXPLICITLY, so it is being treated as advisory. This is the posture that let 291" >&2
    echo "⚠️  correctness errors accumulate unnoticed. See $LOG_DIR/clippy_generated_all_targets.log" >&2
fi

echo "Clippy flow completed."
