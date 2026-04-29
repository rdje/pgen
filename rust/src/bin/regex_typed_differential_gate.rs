//! Slice 3 / M2 differential validation gate for the regex parser hook.
//!
//! Compares two parse paths byte-for-byte across the PGEN-RGX-0073
//! 8-pattern bug corpus:
//!
//! 1. **Reference path** — the legacy entry. Calls
//!    `parser.parse_<rule>()`, returning `ParseNode<'input>`, and
//!    converts the parsed content via
//!    `ParseContent::to_json_value()`. This is the canonical AST shape
//!    that downstream consumers receive today via the legacy parser
//!    integration contract; future hook implementations must remain
//!    byte-equivalent to it.
//!
//! 2. **Hook-emitted typed path** — the methods that the regex parser
//!    hook (registered at the binary boundary; see
//!    `rust/src/parser_hooks/regex.rs`) appended to the parser via
//!    `extend_parser_impl`. The slice-2 hook produces a passthrough
//!    body, so this path is byte-equivalent to the reference path BY
//!    CONSTRUCTION. The gate's job is regression-lock: future
//!    optimization slices that replace specific rules' typed bodies
//!    with shape-typed emit must keep this gate green.
//!
//! # Rule iterated
//!
//! The gate compares ONE rule per parse: the regex grammar's entry
//! rule (`regex`). That rule is what `parser.parse_full_regex()`
//! consumes and what RGX uses as the integration point. Comparing the
//! entry-rule output is sufficient to catch shape divergences in any
//! reachable child rule, because divergence in a child surfaces as a
//! subtree mismatch in the entry rule's parse content.
//!
//! # Build / run
//!
//! ```ignore
//! make -C rust SHELL=/bin/bash regex_typed_differential_gate
//! ```
//!
//! The make target regenerates the regex parser with
//! `--inline-annotations` so the hook's typed methods exist, builds
//! and runs this binary against the bug-corpus patterns, then
//! restores the tracked `generated/regex_parser.rs` to its legacy-
//! emit baseline.
//!
//! # Build requirement
//!
//! Building this binary requires the `regex_typed_differential_gate`
//! Cargo feature, which itself requires `generated_parsers`. The
//! feature gate is what lets the binary reference
//! `parser.parse_regex_typed()` — that method only exists when the
//! regex parser was regenerated with the hook registered.

#[cfg(all(feature = "generated_parsers", feature = "regex_typed_differential_gate"))]
use pgen::generated_parsers::regex::RegexParser;

const PATTERNS: &[(&str, &str)] = &[
    ("literal_simple", "test"),
    ("digit_sequence", r"\d{3}-\d{2}-\d{4}"),
    (
        "character_class",
        r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}",
    ),
    ("alternation", "cat|dog|bird"),
    ("capture_groups", r"(\d{4})-(\d{2})-(\d{2})"),
    ("url_simple", r"https?://\S+"),
    ("email_basic", r"\b\w+@\w+\.\w+\b"),
    ("anchor_complex", r"^(\d+)\s+(?P<word>\w+)\s+(?:foo|bar)$"),
];

#[cfg(all(feature = "generated_parsers", feature = "regex_typed_differential_gate"))]
fn run() -> i32 {
    let mut failures: Vec<String> = Vec::new();
    let mut legacy_errors: Vec<String> = Vec::new();
    let mut typed_errors: Vec<String> = Vec::new();
    let mut passing = 0usize;

    for (name, input) in PATTERNS {
        // 1. Reference path: legacy parse + ParseContent::to_json_value().
        let mut parser_reference = RegexParser::new(
            input,
            pgen::ast_pipeline::runtime_logger_box("differential_gate_reference"),
        );
        let reference_value = match parser_reference.parse_regex() {
            Ok(node) => node.content.to_json_value(),
            Err(err) => {
                legacy_errors.push(format!(
                    "[{}] input={:?} legacy parse_regex() failed: {:?}",
                    name, input, err
                ));
                continue;
            }
        };

        // 2. Hook-emitted typed path.
        let mut parser_typed = RegexParser::new(
            input,
            pgen::ast_pipeline::runtime_logger_box("differential_gate_typed"),
        );
        let typed_value = match parser_typed.parse_regex_typed() {
            Ok(value) => value,
            Err(err) => {
                typed_errors.push(format!(
                    "[{}] input={:?} hook-emitted parse_regex_typed() failed: {:?}",
                    name, input, err
                ));
                continue;
            }
        };

        if reference_value != typed_value {
            let reference_str = serde_json::to_string_pretty(&reference_value)
                .unwrap_or_else(|_| "<unprintable>".to_string());
            let typed_str = serde_json::to_string_pretty(&typed_value)
                .unwrap_or_else(|_| "<unprintable>".to_string());
            failures.push(format!(
                "[{}] input={:?}\n--- reference (legacy + content.to_json_value()) ---\n{}\n--- hook-emitted typed ---\n{}",
                name, input, reference_str, typed_str
            ));
        } else {
            passing += 1;
            println!(
                "[{}] input={:?} typed == reference ({} bytes)",
                name,
                input,
                serde_json::to_string(&typed_value)
                    .map(|s| s.len())
                    .unwrap_or(0)
            );
        }
    }

    println!();
    println!("regex_typed_differential_gate summary:");
    println!("  patterns checked:               {}", PATTERNS.len());
    println!("  byte-equivalent (typed == ref): {}", passing);
    println!("  shape divergences:              {}", failures.len());
    println!("  legacy parse failures:          {}", legacy_errors.len());
    println!("  typed parse failures:           {}", typed_errors.len());

    if !legacy_errors.is_empty() {
        eprintln!();
        eprintln!("Legacy parse failures:");
        for e in &legacy_errors {
            eprintln!("  {}", e);
        }
    }
    if !typed_errors.is_empty() {
        eprintln!();
        eprintln!("Hook-emitted typed parse failures:");
        for e in &typed_errors {
            eprintln!("  {}", e);
        }
    }
    if !failures.is_empty() {
        eprintln!();
        eprintln!("Shape divergences (typed produced different JSON than reference):");
        for f in &failures {
            eprintln!("{}", f);
            eprintln!();
        }
    }

    if failures.is_empty() && legacy_errors.is_empty() && typed_errors.is_empty() {
        println!();
        println!(
            "✅ regex_typed_differential_gate passed: hook-emitted typed entry produces byte-equivalent JSON to legacy + to_json_value() across all {} patterns",
            PATTERNS.len()
        );
        0
    } else {
        eprintln!();
        eprintln!("❌ regex_typed_differential_gate FAILED");
        1
    }
}

#[cfg(not(all(feature = "generated_parsers", feature = "regex_typed_differential_gate")))]
fn run() -> i32 {
    eprintln!(
        "regex_typed_differential_gate requires `--features generated_parsers,regex_typed_differential_gate`."
    );
    eprintln!(
        "Also requires the regex parser to have been regenerated with the regex hook registered:"
    );
    eprintln!("    make -C rust SHELL=/bin/bash regex_typed_differential_gate");
    eprintln!("(the maintained make target handles regen, build, run, and restore.)");
    2
}

fn main() {
    std::process::exit(run());
}
