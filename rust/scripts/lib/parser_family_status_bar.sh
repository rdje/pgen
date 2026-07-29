# parser_family_status_bar.sh — the ONE shared home of the family-status DONE-BAR logic
# (task-tree leaf `DONE-BAR.2a`; sourced by the three *_parser_family_status_gate.sh scripts).
#
# WHY THIS FILE EXISTS. The `Done` bar is three legs (docs/tasks/DONE-BAR.md):
#   leg 1  stimuli-generator proof, zero residual actionable-target debt;
#   leg 2  every covering gate green NOW and actually invoked;
#   leg 3  an officially-recognized external corpus, PASSING — a TRIAGE gate is not a conformance
#          gate, a characterization is not a pass, and the ABSENCE of a corpus is an UNMET leg.
# Until `DONE-BAR.2a`, the family-status gates implemented the OLD bar: their computable vocabulary
# was Done / Mostly Done / In Progress / Not Started, they carried no leg-3 criterion at all, and an
# honest demotion to `Provisional` therefore turned them RED (measured:
# docs/tasks/artifacts/done_bar/run_demotion_impact_probe.sh). This library adds what they lacked:
#   - `family_done_bar_leg3`         — evaluate leg 3 for a family from the DONE-BAR register;
#   - `family_apply_done_bar_status` — cap the legacy ladder: `Done` is unreachable while leg 3 is
#                                      unmet; the computed answer is then the QUALIFIED Provisional
#                                      tier (`Provisional (ceiling)` / `Provisional (corpus pending)`);
#   - `markdown_table_status_for_row` — the live-tracker row reader, previously copy-pasted
#                                      byte-identically into all three gates (the `.10` duplication
#                                      lesson): this file is now its single home.
#
# QUALIFIER RULE (never defaulted — DONE-BAR: "the comfortable label is the one that closes the
# row"): derived from the register's `language_owner`, exactly as `DONE-BAR.1`'s audit derives it:
#   pgen              => `(ceiling)`        — leg 3 unreachable BY CONSTRUCTION; a FINISHED row;
#   external-standard => `(corpus pending)` — a recognized corpus exists; wiring it is outstanding;
#   unadjudicated     => REFUSE (exit 2)    — a status gate may not guess; DONE-BAR.3 owes the ruling.
#
# REFUSAL POLARITY (exit 2, matching scripts/audit_done_bar.sh): an input this library cannot judge
# honestly — a missing register, an unregistered family, an unadjudicated owner, or a declared
# leg3_surface that contradicts the register (a triage gate, a script reading no declared corpus
# root) — REFUSES rather than scoring the comfortable answer. A skip is never a pass.
#
# TESTABILITY SEAMS (probe driver: docs/tasks/artifacts/done_bar/run_family_status_bar_probes.sh):
#   PGEN_FAMILY_STATUS_DONE_BAR_REGISTER  — override the register path (default: the tracked one);
#   PGEN_FAMILY_STATUS_SCRIPTS_DIR        — override the gate-scripts dir for the external-backed test;
#   PGEN_FAMILY_STATUS_REACHABILITY_JSON  — supply a reachability JSON instead of running the
#                                           instrument (scripts/check_gate_reachability.sh --json).

PGEN_FAMILY_STATUS_BAR_LIB_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"

markdown_table_status_for_row() {
    local row_match="$1"
    local path="$2"
    local line
    line="$(grep -F "$row_match" "$path" | head -n 1 || true)"
    if [[ -z "$line" ]]; then
        echo "error: missing live-tracker row containing '$row_match' in '$path'" >&2
        exit 1
    fi
    awk -F'|' '{print $3}' <<<"$line" | xargs
}

done_bar_register_path() {
    printf '%s\n' "${PGEN_FAMILY_STATUS_DONE_BAR_REGISTER:-$PGEN_FAMILY_STATUS_BAR_LIB_ROOT/rust/test_data/grammar_quality/done_bar_family_register_v0.json}"
}

# family_done_bar_leg3 FAMILY STATE_DIR
#
# Evaluates leg 3 of the DONE-BAR bar for FAMILY. STATE_DIR is the calling gate's state dir, used
# only to hold a fresh reachability derivation when a leg3_surface is declared and no seam supplies
# one. Sets four globals (a bash function cannot return a struct; callers snapshot them immediately,
# before any second family is evaluated):
#   DONE_BAR_LEG3_MET                 true|false — leg 3 asserted as a pass by a valid surface;
#   DONE_BAR_PROVISIONAL_QUALIFIER    "(ceiling)" | "(corpus pending)";
#   DONE_BAR_LEG3_SURFACE_GATE        the declared conformance gate, or "<none>";
#   DONE_BAR_LEG3_DETAIL              one human-readable line naming what is (un)met and why.
#
# A DECLARED surface (`leg3_surface: {gate, summary_json, pass_query}` in the register) must satisfy
# all three of leg 3's tests — the ones DONE-BAR.1 measured every current corpus-named gate failing:
#   (a) CONFORMANCE, not triage — a `*triage*` gate name REFUSES (register error, not an unmet leg);
#   (b) EXTERNAL-BACKED — the gate's script must reference one of the family's declared
#       corpus_roots, else the register contradicts itself and the call REFUSES;
#   (c) ACTUALLY INVOKED — the gate must be `reachable` per scripts/check_gate_reachability.sh;
#       an orphan surface leaves leg 3 UNMET (a gate nothing invokes is indistinguishable from a
#       gate that does not exist);
# and its artifact must satisfy the declared `pass_query` — only then is DONE_BAR_LEG3_MET=true.
family_done_bar_leg3() {
    local family="$1"
    local state_dir="$2"
    local register scripts_dir
    register="$(done_bar_register_path)"
    scripts_dir="${PGEN_FAMILY_STATUS_SCRIPTS_DIR:-$PGEN_FAMILY_STATUS_BAR_LIB_ROOT/rust/scripts}"

    DONE_BAR_LEG3_MET=false
    DONE_BAR_PROVISIONAL_QUALIFIER=""
    DONE_BAR_LEG3_SURFACE_GATE="<none>"
    DONE_BAR_LEG3_DETAIL=""

    if [[ ! -s "$register" ]]; then
        echo "error: done-bar register '$register' is missing or empty; leg 3 cannot be judged (DONE-BAR.2a)" >&2
        exit 2
    fi
    if [[ "$(jq -r --arg f "$family" '.families | has($f)' "$register")" != "true" ]]; then
        echo "error: family '$family' is absent from the done-bar register '$register'." >&2
        echo "       An unregistered family REFUSES rather than being scored (DONE-BAR.1's polarity: a" >&2
        echo "       skip would let a new row score well by being invisible)." >&2
        exit 2
    fi

    local owner standard
    owner="$(jq -r --arg f "$family" '.families[$f].language_owner // ""' "$register")"
    standard="$(jq -r --arg f "$family" '.families[$f].standard // ""' "$register")"
    case "$owner" in
        pgen)
            DONE_BAR_PROVISIONAL_QUALIFIER="(ceiling)"
            ;;
        external-standard)
            DONE_BAR_PROVISIONAL_QUALIFIER="(corpus pending)"
            ;;
        unadjudicated)
            echo "error: family '$family' language ownership is UNADJUDICATED in the done-bar register;" >&2
            echo "       a status gate may not guess the Provisional qualifier ('the comfortable label is" >&2
            echo "       the one that closes the row'). DONE-BAR.3 owes the ruling." >&2
            exit 2
            ;;
        *)
            echo "error: family '$family' has unknown language_owner '$owner' in the done-bar register '$register'" >&2
            exit 2
            ;;
    esac

    local surface
    surface="$(jq -c --arg f "$family" '.families[$f].leg3_surface // null' "$register")"
    if [[ "$surface" == "null" ]]; then
        if [[ "$owner" == "pgen" ]]; then
            DONE_BAR_LEG3_DETAIL="leg 3 is unreachable BY CONSTRUCTION: the language is PGEN's own, so no third-party corpus exists or ever will; Provisional (ceiling) is this family's FINISHED ceiling"
        else
            DONE_BAR_LEG3_DETAIL="no external-corpus conformance surface is declared (register leg3_surface=null) although the language is externally standardized (${standard}); wiring one is DONE-BAR.3's work, and absence is an UNMET leg, never an inapplicable one"
        fi
        return 0
    fi

    local gate summary_json pass_query
    gate="$(jq -r '.gate // ""' <<<"$surface")"
    summary_json="$(jq -r '.summary_json // ""' <<<"$surface")"
    pass_query="$(jq -r '.pass_query // ""' <<<"$surface")"
    if [[ -z "$gate" || -z "$summary_json" || -z "$pass_query" ]]; then
        echo "error: family '$family' leg3_surface is malformed — it must declare gate + summary_json + pass_query (see the register's policy.leg3_surface)" >&2
        exit 2
    fi
    if [[ "$gate" == *triage* ]]; then
        echo "error: family '$family' leg3_surface names '$gate' — a TRIAGE gate is not a conformance" >&2
        echo "       gate (DONE-BAR leg 3; the vhdl 0.058% lesson). Refusing the register entry." >&2
        exit 2
    fi
    DONE_BAR_LEG3_SURFACE_GATE="$gate"

    local gate_script="$scripts_dir/${gate}.sh"
    if [[ ! -f "$gate_script" ]]; then
        echo "error: family '$family' leg3_surface gate script '$gate_script' does not exist; the register names a surface the repository does not ship" >&2
        exit 2
    fi
    local root external_backed=false
    while IFS= read -r root; do
        [[ -n "$root" ]] || continue
        if grep -q -- "$root" "$gate_script"; then
            external_backed=true
            break
        fi
    done < <(jq -r --arg f "$family" '.families[$f].corpus_roots[]?' "$register")
    if [[ "$external_backed" != true ]]; then
        echo "error: family '$family' leg3_surface gate '$gate' references NONE of the family's declared" >&2
        echo "       corpus_roots — not external-backed. A gate that merely has 'corpus' in its name" >&2
        echo "       while reading a repo-authored fixture is the exact DONE-BAR.1-measured trap." >&2
        echo "       Refusing the register entry." >&2
        exit 2
    fi

    local reach_json="${PGEN_FAMILY_STATUS_REACHABILITY_JSON:-}"
    if [[ -z "$reach_json" ]]; then
        reach_json="$state_dir/gate_reachability.json"
        if ! bash "$PGEN_FAMILY_STATUS_BAR_LIB_ROOT/scripts/check_gate_reachability.sh" --json "$reach_json" >"$state_dir/gate_reachability.log" 2>&1; then
            echo "error: scripts/check_gate_reachability.sh failed, so the leg-3 'actually invoked' test" >&2
            echo "       cannot be judged (log: $state_dir/gate_reachability.log). A test that cannot run" >&2
            echo "       must say so, not return green." >&2
            exit 2
        fi
    fi
    local reach_status
    reach_status="$(jq -r --arg g "$gate" '[.rows[] | select(.target == $g)][0].status // "absent-from-universe"' "$reach_json")"
    if [[ "$reach_status" != "reachable" ]]; then
        DONE_BAR_LEG3_DETAIL="declared conformance surface '$gate' is not invoked by anything that runs (reachability: ${reach_status}); a gate nothing invokes is indistinguishable from a gate that does not exist"
        return 0
    fi

    local summary_path="$PGEN_FAMILY_STATUS_BAR_LIB_ROOT/$summary_json"
    if [[ ! -s "$summary_path" ]]; then
        DONE_BAR_LEG3_DETAIL="declared conformance surface '$gate' has no artifact at ${summary_json}; leg 3 is UNPROVEN here, and unproven does not satisfy the bar"
        return 0
    fi
    if jq -e "$pass_query" "$summary_path" >/dev/null 2>&1; then
        DONE_BAR_LEG3_MET=true
        DONE_BAR_LEG3_DETAIL="external-corpus conformance surface '$gate' asserts a pass (${summary_json}), is external-backed, and is actually invoked"
    else
        DONE_BAR_LEG3_DETAIL="declared conformance surface '$gate' has an artifact at ${summary_json} that does NOT satisfy its declared pass assertion; leg 3 is UNMET"
    fi
    return 0
}

# family_apply_done_bar_status LEGACY_STATUS
#
# The DONE-BAR cap over the legacy ladder: echoes the final computed status. `Done` is unreachable
# while leg 3 is unmet — the computed answer is then the QUALIFIED Provisional tier. Statuses below
# `Done` pass through unchanged (`Provisional` asserts legs 1-2 hold, which is exactly what the
# legacy ladder's own criteria establish before it would say `Done`).
#
# Call `family_done_bar_leg3` for the SAME family first — this reads its globals.
family_apply_done_bar_status() {
    local legacy="$1"
    if [[ -z "$DONE_BAR_PROVISIONAL_QUALIFIER" ]]; then
        echo "error: family_apply_done_bar_status called before family_done_bar_leg3 (no qualifier derived)" >&2
        exit 2
    fi
    if [[ "$legacy" == "Done" && "$DONE_BAR_LEG3_MET" != true ]]; then
        printf 'Provisional %s\n' "$DONE_BAR_PROVISIONAL_QUALIFIER"
    else
        printf '%s\n' "$legacy"
    fi
}
