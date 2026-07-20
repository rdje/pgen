#!/usr/bin/env bash
set -euo pipefail

artifact_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$artifact_dir/../../../.." && pwd)"
probe="$repo_root/preserved_probes/regex_perf_probe_c1_1d3fa0ee"
source_file="$artifact_dir/pc_sampler.c"
raw_file="$artifact_dir/qualification_raw_pcs.txt"
baseline_file="$artifact_dir/qualification_baseline.txt"
injected_file="$artifact_dir/qualification_injected.txt"
analysis_file="$artifact_dir/qualification_analysis.txt"
provenance_file="$artifact_dir/qualification_provenance.txt"
expected_probe_sha="1d3fa0eebb4de19ff003ace38721d80d71bd94653f8ea3801c0f29ca9581f9b8"
samples=300000
warmup=200
interval_us=250

actual_probe_sha="$(shasum -a 256 "$probe" | awk '{print $1}')"
if [[ "$actual_probe_sha" != "$expected_probe_sha" ]]; then
    echo "REFUSE: probe sha256 $actual_probe_sha != $expected_probe_sha" >&2
    exit 1
fi

work_dir="$(mktemp -d /tmp/pgen-pc-sampler-0181.XXXXXX)"
sampler_dylib="$work_dir/libpgen_pc_sampler.dylib"
cleanup() {
    rm -f "$sampler_dylib"
    rmdir "$work_dir"
}
trap cleanup EXIT

clang -dynamiclib -O2 -std=c11 -Wall -Wextra -Werror \
    -o "$sampler_dylib" "$source_file"
sampler_sha="$(shasum -a 256 "$sampler_dylib" | awk '{print $1}')"

caffeinate -ims "$probe" --samples "$samples" --warmup "$warmup" \
    >"$baseline_file"

caffeinate -ims /bin/bash -c \
    'DYLD_INSERT_LIBRARIES="$1" PGEN_PC_SAMPLE_OUT="$2" PGEN_PC_SAMPLE_INTERVAL_US="$3" exec "$4" --samples "$5" --warmup "$6"' \
    pgen-pc-sampler "$sampler_dylib" "$raw_file" "$interval_us" "$probe" \
    "$samples" "$warmup" >"$injected_file"

{
    echo "probe_sha256=$actual_probe_sha"
    echo "sampler_source_sha256=$(shasum -a 256 "$source_file" | awk '{print $1}')"
    echo "sampler_dylib_sha256=$sampler_sha"
    echo "samples=$samples"
    echo "warmup=$warmup"
    echo "interval_us=$interval_us"
    echo "clang=$(clang --version | sed -n '1p')"
} >"$provenance_file"

python3 "$artifact_dir/analyze_qualification.py" \
    --raw "$raw_file" \
    --baseline "$baseline_file" \
    --injected "$injected_file" \
    --sampler-sha256 "$sampler_sha" >"$analysis_file"

cat "$analysis_file"
