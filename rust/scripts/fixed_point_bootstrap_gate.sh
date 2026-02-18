#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_DIR="$ROOT_DIR/rust"
TOOLS_DIR="$ROOT_DIR/tools"
GRAMMARS_DIR="$ROOT_DIR/grammars"

STATE_DIR="$RUST_DIR/target/fixed_point_gate"
WORK_DIR="$STATE_DIR/work"
SNAPSHOT_DIR="$STATE_DIR/snapshots"

CYCLES=2
KEEP_ARTIFACTS=0

usage() {
    cat <<'EOF'
Usage: fixed_point_bootstrap_gate.sh [--cycles N] [--keep-artifacts]

Regenerates bootstrap return/semantic artifacts for multiple cycles and verifies
that outputs are byte-identical across cycles.

Options:
  --cycles N         Number of generation cycles to run (minimum: 2, default: 2)
  --keep-artifacts   Keep snapshots under rust/target/fixed_point_gate on success
  -h, --help         Show this help
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --cycles)
            if [[ $# -lt 2 ]]; then
                echo "error: --cycles requires a numeric argument" >&2
                exit 2
            fi
            CYCLES="$2"
            shift 2
            ;;
        --keep-artifacts)
            KEEP_ARTIFACTS=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "error: unknown option '$1'" >&2
            usage >&2
            exit 2
            ;;
    esac
done

if ! [[ "$CYCLES" =~ ^[0-9]+$ ]] || [[ "$CYCLES" -lt 2 ]]; then
    echo "error: --cycles must be an integer >= 2" >&2
    exit 2
fi

BOOTSTRAP_BIN="$RUST_DIR/target/debug/ast_pipeline_bootstrap"
SEMANTIC_GRAMMAR="$GRAMMARS_DIR/semantic_annotation.ebnf"
RETURN_GRAMMAR="$GRAMMARS_DIR/return_annotation.ebnf"

mkdir -p "$STATE_DIR"
rm -rf "$SNAPSHOT_DIR"
mkdir -p "$SNAPSHOT_DIR"

echo "==> Building bootstrap pipeline binary"
(cd "$RUST_DIR" && cargo build --bin ast_pipeline_bootstrap --no-default-features --features bootstrap >/dev/null)

if [[ ! -x "$BOOTSTRAP_BIN" ]]; then
    echo "error: bootstrap binary not found at '$BOOTSTRAP_BIN'" >&2
    exit 1
fi

generate_cycle() {
    local cycle="$1"
    local generated_dir="$WORK_DIR/generated"
    local cycle_snapshot="$SNAPSHOT_DIR/cycle_${cycle}"

    rm -rf "$WORK_DIR"
    mkdir -p "$generated_dir"

    echo "==> [cycle ${cycle}/${CYCLES}] EBNF -> JSON"
    perl "$TOOLS_DIR/ebnf_to_json.pl" --pretty --quiet "$SEMANTIC_GRAMMAR" -o "$generated_dir/semantic_annotation.json"
    perl "$TOOLS_DIR/ebnf_to_json.pl" --pretty --quiet "$RETURN_GRAMMAR" -o "$generated_dir/return_annotation.json"

    echo "==> [cycle ${cycle}/${CYCLES}] JSON -> parser.rs (bootstrap mode)"
    "$BOOTSTRAP_BIN" --generate-parser --bootstrap-mode --eliminate-left-recursion \
        "$generated_dir/semantic_annotation.json" \
        -o "$generated_dir/semantic_annotation_parser.rs" >/dev/null 2>&1

    "$BOOTSTRAP_BIN" --generate-parser --bootstrap-mode --eliminate-left-recursion \
        "$generated_dir/return_annotation.json" \
        -o "$generated_dir/return_annotation_parser.rs" >/dev/null 2>&1

    mkdir -p "$cycle_snapshot"
    cp "$generated_dir/semantic_annotation.json" "$cycle_snapshot/semantic_annotation.json"
    cp "$generated_dir/return_annotation.json" "$cycle_snapshot/return_annotation.json"
    cp "$generated_dir/semantic_annotation_parser.rs" "$cycle_snapshot/semantic_annotation_parser.rs"
    cp "$generated_dir/return_annotation_parser.rs" "$cycle_snapshot/return_annotation_parser.rs"

    normalize_json() {
        local input_file="$1"
        local output_file="$2"
        perl -MJSON::PP -e '
            use strict;
            use warnings;
            my ($in, $out) = @ARGV;
            local $/;
            open my $fh, "<", $in or die "open $in: $!";
            my $json = <$fh>;
            close $fh;
            my $data = decode_json($json);
            if (ref($data) eq "HASH" && ref($data->{metadata}) eq "HASH") {
                delete $data->{metadata}->{generated_at};
            }
            my $encoded = JSON::PP->new->ascii->canonical->pretty->encode($data);
            open my $ofh, ">", $out or die "open $out: $!";
            print {$ofh} $encoded;
            close $ofh;
        ' "$input_file" "$output_file"
    }

    normalize_json "$generated_dir/semantic_annotation.json" "$cycle_snapshot/semantic_annotation.normalized.json"
    normalize_json "$generated_dir/return_annotation.json" "$cycle_snapshot/return_annotation.normalized.json"

}

for cycle in $(seq 1 "$CYCLES"); do
    generate_cycle "$cycle"
done

artifacts=(
    "semantic_annotation.normalized.json"
    "return_annotation.normalized.json"
    "semantic_annotation_parser.rs"
    "return_annotation_parser.rs"
)

echo "==> Comparing cycle outputs for byte-identical fixed-point behavior"
mismatch=0
for artifact in "${artifacts[@]}"; do
    baseline="$SNAPSHOT_DIR/cycle_1/$artifact"
    for cycle in $(seq 2 "$CYCLES"); do
        candidate="$SNAPSHOT_DIR/cycle_${cycle}/$artifact"
        if ! cmp -s "$baseline" "$candidate"; then
            echo "❌ MISMATCH: $artifact differs between cycle 1 and cycle $cycle" >&2
            diff -u "$baseline" "$candidate" > "$SNAPSHOT_DIR/${artifact}.cycle1_vs_cycle${cycle}.diff" || true
            echo "   diff: $SNAPSHOT_DIR/${artifact}.cycle1_vs_cycle${cycle}.diff" >&2
            mismatch=1
        fi
    done
done

if [[ "$mismatch" -ne 0 ]]; then
    echo "❌ Fixed-point gate failed: bootstrap outputs are not deterministic." >&2
    echo "   Snapshot root: $SNAPSHOT_DIR" >&2
    exit 1
fi

echo "✅ Fixed-point gate passed: all compared bootstrap artifacts are byte-identical."

if [[ "$KEEP_ARTIFACTS" -eq 0 ]]; then
    rm -rf "$STATE_DIR"
fi
