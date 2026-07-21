#!/usr/bin/env bash
# PGEN-RGX-0078-0213 three-band raw-PC capture on the landed slimmed-carrier
# representation (preserved probe carrier48_8d392176).  The -0206 recipe
# reused verbatim (script re-targeted): the v3 sampler (SHA-pinned from
# three_band_pc_capture/), 250 us ITIMER_PROF, deterministic pending-SIGPROF
# teardown qualification before the corpus is spent, sequential bands under
# caffeinate + the 16384 MB memory guard.  Band inputs are byte-derived from
# the -0212 rerun_cand2 timing rows.  Per the -0212 hardened custody
# protocol, a load snapshot is banked around every band.
set -euo pipefail

artifact_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$artifact_dir/../../../.." && pwd)"
probe="$repo_root/preserved_probes/regex_perf_probe_carrier48_8d392176"
sampler_source="$repo_root/docs/tasks/artifacts/three_band_pc_capture/pc_sampler.c"
derive="$artifact_dir/derive_band_inputs.py"
guard="$repo_root/scripts/run_with_memory_guard.sh"
expected_probe_sha="8d392176b8456ac66ddaa62b989ad78204029a2c360caef19c4d811a40c1f75c"
expected_source_sha="9bb51a53c4ecd5e4708925ae78f90c503f8e6511c8b46a51612c373db3c3bf85"
interval_us=250
warmup=50

actual_probe_sha="$(shasum -a 256 "$probe" | awk '{print $1}')"
actual_source_sha="$(shasum -a 256 "$sampler_source" | awk '{print $1}')"
if [[ "$actual_probe_sha" != "$expected_probe_sha" ]]; then
    echo "REFUSE: probe sha256 $actual_probe_sha != $expected_probe_sha" >&2
    exit 1
fi
if [[ "$actual_source_sha" != "$expected_source_sha" ]]; then
    echo "REFUSE: sampler source sha256 $actual_source_sha != $expected_source_sha" >&2
    exit 1
fi

available_kib="$(df -Pk "$repo_root" | awk 'NR==2 {print $4}')"
if (( available_kib < 8388608 )); then
    echo "REFUSE: available disk ${available_kib} KiB is below 8 GiB" >&2
    exit 1
fi

work_dir="$(mktemp -d "${TMPDIR:-/tmp}/pgen-carrier48-reprice-0213.XXXXXX")"
sampler_dylib="$work_dir/libpgen_pc_sampler.dylib"
cleanup() {
    rm -f "$sampler_dylib"
    rm -f "$work_dir/band_sub1us.jsonl"
    rm -f "$work_dir/band_1to2p5.jsonl"
    rm -f "$work_dir/band_2p5to20.jsonl"
    rm -f "$work_dir/memory_guard.marker"
    rmdir "$work_dir"
}
trap cleanup EXIT

python3 "$derive" --output-dir "$work_dir" >"$artifact_dir/input_derivation.txt"
clang -dynamiclib -O2 -std=c11 -Wall -Wextra -Werror \
    -o "$sampler_dylib" "$sampler_source"
sampler_dylib_sha="$(shasum -a 256 "$sampler_dylib" | awk '{print $1}')"

DYLD_INSERT_LIBRARIES="$sampler_dylib" \
PGEN_PC_SAMPLE_OUT="$artifact_dir/teardown_qualification_raw_pcs.txt" \
PGEN_PC_SAMPLE_INTERVAL_US="$interval_us" \
PGEN_PC_SAMPLE_FORCE_PENDING_SIGPROF=1 \
    "$probe" --samples 10000 --warmup 20 \
    >"$artifact_dir/teardown_qualification_stdout.txt"
if [[ "$(awk -F= '$1 == "format" {print $2}' "$artifact_dir/teardown_qualification_raw_pcs.txt")" != "pgen-pc-samples-v3" ]] ||
   [[ "$(awk -F= '$1 == "forced_pending_sigprof" {print $2}' "$artifact_dir/teardown_qualification_raw_pcs.txt")" != "1" ]] ||
   [[ "$(awk -F= '$1 == "pending_sigprof_drained" {print $2}' "$artifact_dir/teardown_qualification_raw_pcs.txt")" != "1" ]]; then
    echo "REFUSE: deterministic pending-SIGPROF teardown qualification failed" >&2
    exit 1
fi

snapshot_load() {
    {
        echo "=== $1 at $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
        sysctl -n vm.loadavg
        # awk consumes all of sort's output (no head-induced SIGPIPE under
        # `set -o pipefail` — the rc=141 failure of the first run).
        ps -Ao pcpu,pid,comm | sort -rn | awk 'NR <= 6'
    } >>"$artifact_dir/load_snapshots.txt"
}

run_band() {
    local name="$1"
    local samples="$2"
    local input="$work_dir/band_${name}.jsonl"
    local raw="$artifact_dir/capture_band_${name}_raw_pcs.txt"
    local times="$artifact_dir/capture_band_${name}_times.jsonl"
    local stdout="$artifact_dir/capture_band_${name}_stdout.txt"
    echo "starting band=$name samples=$samples at $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    snapshot_load "before band=$name"
    caffeinate -ims "$guard" --budget-mb 16384 --floor-pct 10 --timeout-s 7200 \
        --marker "$work_dir/memory_guard.marker" -- \
        /bin/bash -c \
        'DYLD_INSERT_LIBRARIES="$1" PGEN_PC_SAMPLE_OUT="$2" PGEN_PC_SAMPLE_INTERVAL_US="$3" exec "$4" --corpus-jsonl "$5" --out-jsonl "$6" --samples "$7" --warmup "$8"' \
        pgen-carrier48-reprice "$sampler_dylib" "$raw" "$interval_us" "$probe" \
        "$input" "$times" "$samples" "$warmup" >"$stdout"
    snapshot_load "after band=$name"
    echo "finished band=$name at $(date -u +%Y-%m-%dT%H:%M:%SZ)"
}

: >"$artifact_dir/load_snapshots.txt"
{
    echo "probe_sha256=$actual_probe_sha"
    echo "sampler_source_sha256=$actual_source_sha"
    echo "sampler_dylib_sha256=$sampler_dylib_sha"
    echo "interval_us=$interval_us"
    echo "warmup=$warmup"
    echo "sub1us_samples=170000"
    echo "1to2p5_samples=90000"
    echo "2p5to20_samples=55000"
    echo "clang=$(clang --version | sed -n '1p')"
} >"$artifact_dir/capture_provenance.txt"

run_band sub1us 170000
run_band 1to2p5 90000
run_band 2p5to20 55000

# Live-doc path doctrine: banked outputs must be repo-root-relative.
for banked in "$artifact_dir"/capture_band_*_stdout.txt \
              "$artifact_dir/teardown_qualification_stdout.txt" \
              "$artifact_dir/input_derivation.txt"; do
    perl -pi -e "s|\\Q$repo_root/\\E||g" "$banked"
done
echo "capture complete at $(date -u +%Y-%m-%dT%H:%M:%SZ)"
