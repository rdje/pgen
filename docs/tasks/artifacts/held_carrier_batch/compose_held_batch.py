#!/usr/bin/env python3
"""Reconstruct and classify the held carrier program through RGX-0078-0196.

Every input classifier is SHA-pinned and rerun byte-for-byte.  The composition
separates exact current-side work from caller-attributed regions, contract-
blocked work, and gross replacement ceilings.  It deliberately does not call
any pre-implementation target a measured net saving.
"""

from __future__ import annotations

import hashlib
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


ARTIFACT_DIR = Path(__file__).resolve().parent
TASK_ARTIFACTS = ARTIFACT_DIR.parent
FLOOR_NS = 1263.4
NOISE_NS = 28.8


@dataclass(frozen=True)
class Evidence:
    directory: str
    script: str
    output: str
    script_sha256: str
    output_sha256: str


EVIDENCE = {
    "residual": Evidence(
        "batch1_residual_reprice", "reprice.py", "reprice.txt",
        "ce4cf451e3a450bbe9810e1beebd7d02e152cc57d7396a7c56b4b1f6577e29d6",
        "33c5ac5eb5dda121b981785678537ce86be46b385528c16670be0ddc18f52f74",
    ),
    "reset": Evidence(
        "g3_setup_teardown_pricing", "price_g3.py", "price_g3.txt",
        "2f99a01e03cf7251d2785953aaee83ff2177d684b3a9e968e375125566adc18b",
        "68563fa33c5116565c492dd40a0aa845177dadc861f27105b34ea9020f983efb",
    ),
    "bare": Evidence(
        "bare_diagnostic_expansion", "classify_bare_diagnostics.py", "bare_diagnostic_expansion.txt",
        "96e0293ee8df8fb43c484f2d91c528b3f95d09db6ee82090758fdbe6712c2745",
        "7c258cc03a56173be785477a41e8a1b1943cd940be4c418788ec66f7285408eb",
    ),
    "telemetry": Evidence(
        "semantic_telemetry_expansion", "classify_semantic_telemetry.py", "semantic_telemetry_expansion.txt",
        "d809e1233d36c4fa6f04803976d09866f8b43378db7bca6c24da5e672cf74434",
        "a3d1d37dcf0bf7bb9c4ff7bfbaada659954a71f8f239b3e9aa475bb03adbcea9",
    ),
    "name_fixed": Evidence(
        "recursion_guard_expansion", "classify_recursion_guard.py", "recursion_guard_expansion.txt",
        "ad6372e17beea69b7c3297c3c60e97f160da90e238820cfe42c9c5b8f14a35c5",
        "20b5c63d7e5809473f3f711df915724a9d0d55f534781cd83e627849a3c6ee56",
    ),
    "name_rollback": Evidence(
        "try_parse_name_rollback", "classify_try_parse_name_rollback.py", "try_parse_name_rollback.txt",
        "6b23e7b0037fbb207f7ae8a690ce69014360709cef52e560e473ac7da1705e3d",
        "8da74ed6f2f36e3c25b78634e80fdd3d7ad330b1ae399458a7e1aaa0adf54065",
    ),
    "name_growth": Evidence(
        "recursion_name_growth", "classify_recursion_name_growth.py", "recursion_name_growth.txt",
        "2d796551683cbf356213d77257ea96f55f8d2e9d43dc5cfe12ee60d31cb2d5d4",
        "194c90c9349500665ad700f578271842f303b2c7714453d2f2ad15cb01dc60ca",
    ),
    "event": Evidence(
        "deriv_event_width", "classify_deriv_event_width.py", "deriv_event_width.txt",
        "9c5edbda41cc862095577911ad353b31cb0040f1d839e45929959accde82c213",
        "72ec056f6af02023c96c99edefc3f45aa7294d5fb2d6e732b6049afc870acb23",
    ),
    "checkpoint": Evidence(
        "checkpoint_compaction", "inspect_checkpoint_word.py", "checkpoint_compaction.txt",
        "9b197c4b529fd99d709db5f2c7de3b5411d82c7019dfe65c3d1f3650e80a126a",
        "5d91d6cdb533f98344970d6c35b3717d9d837032704ff717c1a425c7c3f85060",
    ),
    "thin_lookup": Evidence(
        "thin_memo_lookup_expansion", "classify_thin_memo_lookup.py", "thin_memo_lookup_expansion.txt",
        "e4df958c86c3321185703c59080ff89974f15174987d9781d7b43dcbed0a1e3e",
        "a733e1301c3f3ce6b3467931acaa134555ca7bf450c2c2bab6f781fa2a2537fd",
    ),
    "input_view": Evidence(
        "input_view_carrier", "classify_input_view_carrier.py", "input_view_carrier.txt",
        "563e8863a3036235790663c0a58281070b3691c1a42fade41e82f8cd65edb5dc",
        "0becba17290bc37d7ced4bd2704a16c59a5af2a798cb18a6b026b31fe90db236",
    ),
    "position": Evidence(
        "position_progress_carrier", "classify_position_progress.py", "position_progress.txt",
        "dfaf8dee9e458b1673e9c32dd91ce98d642937fc5dc9ede8329cbe02f443d3e4",
        "0cfe5c69bfc0b874704eb6162b41397e00bd61d5d3dd61398d5352a70bdd3acd",
    ),
    "unified": Evidence(
        "unified_deriv_tape", "classify_unified_tape.py", "unified_deriv_tape.txt",
        "67f8f41af8dd52d87d330a2445a34940aeed6f5ae143a513f12717f6033d23cb",
        "41667ada012964cf650bf509edef9e93abd945eb5c8f88ef485b2be65c7f11d7",
    ),
}


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def reproduce() -> dict[str, str]:
    outputs: dict[str, str] = {}
    for label, item in EVIDENCE.items():
        directory = TASK_ARTIFACTS / item.directory
        script = directory / item.script
        output = directory / item.output
        if sha256(script) != item.script_sha256:
            raise SystemExit(f"REFUSE: {label} script SHA differs")
        if sha256(output) != item.output_sha256:
            raise SystemExit(f"REFUSE: {label} output SHA differs")
        actual = subprocess.run(
            [sys.executable, str(script)], check=True, capture_output=True, text=True
        ).stdout
        expected = output.read_text()
        if actual != expected:
            raise SystemExit(f"REFUSE: {label} classifier output does not reproduce")
        outputs[label] = expected
    return outputs


def number(text: str, pattern: str) -> float:
    match = re.search(pattern, text)
    if match is None:
        raise SystemExit(f"REFUSE: value pattern absent: {pattern}")
    return float(match.group(1))


def main() -> None:
    outputs = reproduce()
    terms = [
        ("G1-B_drop_control", number(outputs["residual"], r"drop target used in bundle:\s+([0-9.]+) ns"), "exact-current", "drop-free internal control error"),
        ("G1-C_atom_carrier", number(outputs["residual"], r"G1-C attributed target:\s+([0-9.]+) ns"), "attributed-region", "carrier-core design required"),
        ("memo_segment_copy", number(outputs["residual"], r"memo segment-copy target:\s+([0-9.]+) ns"), "attributed-region", "carrier-core design required"),
        ("G3_reset_ceremony", number(outputs["reset"], r"G3 target used in bundle:\s+([0-9.]+) ns"), "exact-current", "in-place semantic reset"),
        ("bare_diagnostics", number(outputs["bare"], r"mechanism_target_ns=([0-9.]+)"), "exact-current", "bare-path observability design"),
        ("public_telemetry", number(outputs["telemetry"], r"telemetry_target_ns=([0-9.]+)"), "contract-blocked", "public post-parse counters"),
        ("name_fixed", number(outputs["name_fixed"], r"guard_subset_target_ns=([0-9.]+)"), "exact-current", "ID-only generated path + reconstruction"),
        ("name_rollback", number(outputs["name_rollback"], r"rollback_target_ns=([0-9.]+)"), "exact-current", "ID-only generated path + reconstruction"),
        ("name_growth", number(outputs["name_growth"], r"full_corpus_conservative_share=[^\n]+ target_ns=([0-9.]+)"), "exact-current", "ID-only generated path + reconstruction"),
        ("event_second_word", number(outputs["event"], r"strict_removable_instruction_floor_ns=([0-9.]+)"), "exact-current", "packed/unified carrier decode"),
        ("checkpoint_duplicate", number(outputs["checkpoint"], r"pricing source=[^\n]+ strict=([0-9.]+)"), "exact-current", "six-word checkpoint"),
    ]
    headline = sum(value for _, value, _, _ in terms)
    exact_current = sum(value for _, value, tier, _ in terms if tier == "exact-current")
    attributed = sum(value for _, value, tier, _ in terms if tier == "attributed-region")
    blocked = sum(value for _, value, tier, _ in terms if tier == "contract-blocked")
    eligible_headline = headline - blocked
    if abs(headline - 102.138976561) > 0.0000000005:
        raise SystemExit(f"REFUSE: headline does not reconstruct: {headline}")
    if abs(exact_current - 44.511629415) > 0.0000000005:
        raise SystemExit(f"REFUSE: eligible exact-current subtotal differs: {exact_current}")
    if abs(attributed - 57.4) > 0.0000000005:
        raise SystemExit(f"REFUSE: attributed subtotal differs: {attributed}")
    if abs(blocked - 0.227347146) > 0.0000000005:
        raise SystemExit(f"REFUSE: blocked subtotal differs: {blocked}")

    gross = [
        ("thin_lookup_proxy", number(outputs["thin_lookup"], r"hash_exclusive_ns=([0-9.]+)")),
        ("input_view", number(outputs["input_view"], r"gross_current_input_carrier_ceiling_ns=([0-9.]+)")),
        ("position_inner", number(outputs["position"], r"inner_load_ceiling=([0-9.]+)")),
        ("unified_boundary_metadata", number(outputs["unified"], r"boundary_direct_metadata_gross_ceiling_ns=([0-9.]+)")),
    ]
    expanded = eligible_headline + sum(value for _, value in gross)

    print(f"evidence custody: PASS classifiers={len(EVIDENCE)} byte_exact={len(EVIDENCE)}")
    print("HEADLINE RECONSTRUCTION (current-side targets, not measured net savings)")
    print("term                         ns            evidence-tier       replacement boundary")
    for label, value, tier, replacement in terms:
        print(f"{label:28} {value:12.9f} {tier:19} {replacement}")
    print(f"headline_reconstructed_ns={headline:.9f}")
    print(f"exact_current_including_blocked_ns={exact_current + blocked:.9f}")
    print(f"exact_current_eligible_ns={exact_current:.9f}")
    print(f"attributed_region_ns={attributed:.9f}")
    print(f"contract_blocked_ns={blocked:.9f}")
    print(f"eligible_headline_ns={eligible_headline:.9f}")
    print("strict_net_saving_banked_before_implementation_ns=0.000000000")
    print()
    print("ADDITIONAL GROSS / REPLACEMENT-PROXY VIEWS (not in headline)")
    for label, value in gross:
        print(f"{label:28} {value:12.9f}")
    print(f"expanded_nonblocked_target_ns={expanded:.9f}")
    print(
        f"expanded_capture_0.30_ns={0.30 * expanded:.9f} "
        f"noise_margin_ns={0.30 * expanded - NOISE_NS:.9f}"
    )
    print()
    print("ACCOUNTING CORRECTION")
    print(
        "102.138976561ns is a pre-implementation current-side TARGET: "
        "44.738976561ns exact current work + 57.400000000ns attributed regions"
    )
    print(
        "0.227347146ns public telemetry is contract-blocked; every remaining "
        "candidate still owes replacement/codegen/ABI cost"
    )
    print("the historical label 'accumulated strict' is superseded; net is measured only by A/B")
    print()
    print("DIRECTOR GEOMEAN RATCHET (binding)")
    print("accept iff candidate_external_corpus_geomean_ns < immediate_baseline_geomean_ns")
    print("equal_or_higher=REJECT_AND_REVERT before selecting the next fix")
    print("correctness_identity=REQUIRED settled_MAX_ns<=483583=REQUIRED")
    print("bench_or_model_improvement_without_corpus_geomean_reduction=REJECT")
    print("HELD CARRIER COMPOSITION: PASS")


if __name__ == "__main__":
    main()
