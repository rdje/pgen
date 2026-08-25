//! `ENGINE-UNIVERSAL-SERVICES.46` (a) — **REPRODUCE THE PROFILE-GATE / NEGATIVE-LOOKAHEAD
//! INVERSION ON THE SHIPPED ENGINE**, not on the interpreter.
//!
//! WHY THIS FILE EXISTS
//! --------------------
//! `.46` was routed on evidence taken entirely with `--interpret-parse` (TOOLBOX 1.5b). That rung is
//! authoritative **BY VERIFICATION, NOT BY CONSTRUCTION**, and TOOLBOX 1.5b records a measured
//! divergence class where the verification does not hold. Publishing *engine-universal* off it was
//! an inference, not a reading — so the first step of the repair is not a repair at all: it is
//! reproducing the defect through `compile_and_parse` (TOOLBOX 1.4), which runs the **real codegen
//! and the real runtime** and is authoritative **BY CONSTRUCTION**.
//!
//! THE CLAIM UNDER TEST
//! --------------------
//! `!X` means *refuse if `X` matches here*. Gate `X` out of a profile and `X` matches nothing, so
//! `!X` succeeds **VACUOUSLY** — deleting a constraint, which makes the STRICT profile **MORE
//! PERMISSIVE** at that site. The deciding invariant it breaks: **gating a rule out of a profile
//! must only ever REMOVE strings from the language, never ADD them.**
//!
//! THE ARMS. Cases 1-6 reuse the interpreter probe's own grammars and inputs, so the two rungs are
//! directly diffable; cases 7-8 are this leaf's own addition.
//!   * **GATE-ACTIVE CONTROL** (1-2) — a REQUIRED gated rule. If `strict` still accepts, the gate is
//!     not applied and every verdict below is worthless. This arm caught exactly that once
//!     (`@profiles` written INLINE after `:=` is a BRANCH-level gate and does not gate the rule).
//!   * **FINDING** (3-4) — the discriminating shape: a catch-all CAN consume the guarded token, so
//!     the two hypotheses predict OPPOSITE verdicts instead of the same reject.
//!   * **SANITY** (5-6) — the catch-all path itself is live under both profiles.
//!   * ⭐ **RED CONTROL** (7-8) — the finding shape with the gate REMOVED (`tick` admitted to both
//!     profiles). It kills the rival hypothesis that `strict` is simply not wired into this shape at
//!     all: that reading predicts ACCEPT here, and the measurement is REJECT.
//!
//! ⭐⭐ **AND EVERY CASE IS MEASURED ON BOTH RUNGS AND ASSERTED VERDICT-IDENTICAL.** `.46`(c) — the
//! semantics repair — must move codegen and interpreter TOGETHER; this agreement column is what
//! makes a one-sided repair fail loudly instead of silently splitting the two engines.
//!
//! Re-derive with ONE command:
//!   make -C rust SHELL=/bin/bash profile_gate_monotonicity_gate
//! or directly:
//!   cd rust && cargo test --features "generated_parsers ebnf_dual_run" \
//!       --test profile_gate_negative_lookahead_generated_parser -- --nocapture
//! The interpreter-only sizing probe this leaf grew out of remains its own command:
//!   bash docs/tasks/artifacts/sv_corpus_grad/profile_gate_sizing/probe.sh
//!
//! ⛔ **CASE 4 IS A DEFECT PIN.** It asserts today's WRONG verdict on purpose, so `.46`(c) cannot
//! land silently: the repair MUST turn this test RED and the author MUST flip the pin in the same
//! commit, on both rungs at once.

use std::path::{Path, PathBuf};

use pgen::parse_harness::{CompileAndParseOptions, compile_and_parse};

/// The repo root, derived from the crate manifest — never an absolute literal (director policy §12:
/// every path is relative to the repo root, so the tree can be relocated).
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("rust/ has a parent")
        .to_path_buf()
}

/// A persistent, gitignored workdir **on the repository volume** (director policy §13 — project data
/// never lands in the system temp root), so the `pgen` dependency compiles once and stays warm.
fn workdir(slot: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/profile_gate_neg_lookahead")
        .join(slot)
}

/// The codegen binary the compile-and-run harness shells out to. Absent, this probe can measure
/// NOTHING, so it REFUSES with the exact rebuild command instead of reporting a flattering verdict —
/// the same posture `probe.sh` takes when `ast_pipeline` is missing.
fn require_codegen_binary() {
    let bin = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/debug/ast_pipeline");
    assert!(
        bin.is_file(),
        "REFUSED: {} is absent, so this probe cannot measure anything. Build it first:\n  \
         cd rust && cargo build --features \"generated_parsers ebnf_dual_run\" --bin ast_pipeline",
        bin.display()
    );
}

/// The GENERATED-PARSER rung (TOOLBOX 1.4): real codegen, real runtime, the profile requested
/// through the artifact's own `set_grammar_profile` resolution — exactly how a shipped parser gates.
/// Authoritative **by construction**.
fn generated_verdict(grammar: &Path, input: &str, profile: &str, slot: &str) -> bool {
    let opts = CompileAndParseOptions {
        workdir: Some(workdir(slot)),
        keep_workdir: true,
        requested_profile: Some(profile.to_string()),
        ..Default::default()
    };
    compile_and_parse(grammar, input, &opts)
        .unwrap_or_else(|e| panic!("compile_and_parse failed for {}: {e}", grammar.display()))
        .accepted
}

/// The INTERPRETER rung (TOOLBOX 1.5/1.5b), when the `.ebnf` frontend is compiled in. `None` means
/// **NOT EVALUATED** — never a pass, never a block — rather than a silently-missing leg.
#[cfg(feature = "ebnf_dual_run")]
fn interpreted_verdict(grammar: &Path, input: &str, profile: &str) -> Option<bool> {
    use pgen::parse_harness_interpreter::{InterpretOptions, interpret_parse};
    let opts = InterpretOptions {
        profile: Some(profile.to_string()),
        ..Default::default()
    };
    Some(
        interpret_parse(grammar, input, &opts)
            .unwrap_or_else(|e| panic!("interpret_parse failed for {}: {e}", grammar.display()))
            .accepted,
    )
}

#[cfg(not(feature = "ebnf_dual_run"))]
fn interpreted_verdict(_grammar: &Path, _input: &str, _profile: &str) -> Option<bool> {
    None
}

/// One measured case. `expected` is what the arm asserts TODAY — for the defect pin that is the
/// wrong-but-real verdict, which is the point of pinning it.
struct Case {
    label: &'static str,
    grammar: PathBuf,
    slot: &'static str,
    input: &'static str,
    profile: &'static str,
    expected: bool,
    /// Why a mismatch matters, printed on failure so the reader is not left to reconstruct it.
    on_mismatch: &'static str,
}

#[test]
fn a_profile_gate_inverts_a_negative_lookahead_on_the_generated_parser_too() {
    require_codegen_binary();
    let root = repo_root();
    let sizing = root.join("docs/tasks/artifacts/sv_corpus_grad/profile_gate_sizing");
    let owned = root.join("docs/tasks/artifacts/engine_universal_services/profile_gate_neg_lookahead");
    let control = sizing.join("gate_active_control.ebnf");
    let finding = sizing.join("neg_lookahead_discriminating.ebnf");
    let red = owned.join("neg_lookahead_ungated_red_control.ebnf");
    for g in [&control, &finding, &red] {
        assert!(g.is_file(), "missing grammar {}", g.display());
    }

    let cases = vec![
        Case {
            label: "CONTROL  gated rule REQUIRED, X live",
            grammar: control.clone(), slot: "control", input: "1'", profile: "loose", expected: true,
            on_mismatch: "a REQUIRED gated rule must PARSE under the profile it is admitted to",
        },
        Case {
            label: "CONTROL  gated rule REQUIRED, X gated",
            grammar: control.clone(), slot: "control", input: "1'", profile: "strict", expected: false,
            on_mismatch: "THE GATE IS NOT APPLIED, so every verdict below is worthless. Re-check that \
                          @profiles sits on its own line ABOVE the rule (rule-level), not inline after `:=`",
        },
        Case {
            label: "FINDING  !X refuses, X live",
            grammar: finding.clone(), slot: "finding", input: "1'", profile: "loose", expected: false,
            on_mismatch: "`!X` must REFUSE while X is live in the profile",
        },
        Case {
            // ⛔⛔ THE DEFECT PIN. `expected: true` is the WRONG verdict, asserted on purpose.
            label: "FINDING  !X is VACUOUS, X gated  ⛔ DEFECT PIN",
            grammar: finding.clone(), slot: "finding", input: "1'", profile: "strict", expected: true,
            on_mismatch: "DEFECT PIN no longer holds. If `.46`(c) landed, flip this case's `expected` \
                          to `false` — the monotonic verdict — in that same commit; if it did not, a \
                          silent behaviour change occurred and must be root-caused",
        },
        Case {
            label: "SANITY   catch-all consumes a plain char",
            grammar: finding.clone(), slot: "finding", input: "1a", profile: "loose", expected: true,
            on_mismatch: "the catch-all path must be live, or the finding arm proves nothing",
        },
        Case {
            label: "SANITY   catch-all consumes a plain char",
            grammar: finding.clone(), slot: "finding", input: "1a", profile: "strict", expected: true,
            on_mismatch: "the catch-all path must be live under `strict` too",
        },
        Case {
            label: "RED CTL  !X refuses, gate REMOVED",
            grammar: red.clone(), slot: "red_control", input: "1'", profile: "strict", expected: false,
            on_mismatch: "RED CONTROL FAILED — with X ADMITTED to `strict`, `!X` must still refuse. An \
                          accept here means the finding arm is not measuring the gate at all",
        },
        Case {
            label: "RED CTL  !X refuses, gate REMOVED",
            grammar: red.clone(), slot: "red_control", input: "1'", profile: "loose", expected: false,
            on_mismatch: "RED CONTROL FAILED — `!X` must refuse under `loose` too when X is admitted",
        },
    ];

    println!("\nENGINE-UNIVERSAL-SERVICES.46 (a) — the SHIPPED ENGINE, via compile_and_parse (TOOLBOX 1.4)");
    println!("   the interpreter column is TOOLBOX 1.5; `.46`(c) must move both together\n");
    println!(
        "  {:<38} {:<38} {:<7} {:<5}  {:<10} {:<12} {}",
        "case", "grammar", "profile", "input", "generated", "interpreted", "agree"
    );

    let mut failures: Vec<String> = Vec::new();
    let mut interpreter_evaluated = 0usize;
    let mut rungs_agreed = 0usize;
    for case in &cases {
        let generated = generated_verdict(&case.grammar, case.input, case.profile, case.slot);
        let interp = interpreted_verdict(&case.grammar, case.input, case.profile);
        let agree = match interp {
            Some(i) => {
                interpreter_evaluated += 1;
                if i == generated {
                    rungs_agreed += 1;
                    "yes"
                } else {
                    "⛔ NO"
                }
            }
            None => "not-evaluated",
        };
        println!(
            "  {:<38} {:<38} {:<7} {:<5}  {:<10} {:<12} {}",
            case.label,
            case.grammar.file_name().and_then(|s| s.to_str()).unwrap_or("?"),
            case.profile,
            format!("{:?}", case.input),
            generated,
            interp.map(|b| b.to_string()).unwrap_or_else(|| "-".to_string()),
            agree
        );
        if generated != case.expected {
            failures.push(format!(
                "{} [{}]: generated={generated} wanted={} — {}",
                case.label, case.profile, case.expected, case.on_mismatch
            ));
        }
        if let Some(i) = interp
            && i != generated
        {
            failures.push(format!(
                "{} [{}]: RUNG SPLIT — generated={generated} interpreted={i}. The two engines must agree on \
                 this surface; `.46`(c) repairs codegen and interpreter TOGETHER.",
                case.label, case.profile
            ));
        }
    }

    // ⛔ The agreement figure counts AGREEMENTS, never evaluations: an earlier spelling counted the
    // latter and printed "agreed on 8/8" on the very run that had just found two rung splits. A
    // summary line that reads flatteringly under failure is the same defect class as the finding.
    println!(
        "\nRESULT: {} cases on the GENERATED parser; interpreter rung {}.",
        cases.len(),
        if interpreter_evaluated == 0 {
            "NOT EVALUATED (build with --features ebnf_dual_run)".to_string()
        } else {
            format!("agreed on {rungs_agreed}/{interpreter_evaluated} evaluated")
        }
    );
    assert!(failures.is_empty(), "\n  - {}\n", failures.join("\n  - "));
}
