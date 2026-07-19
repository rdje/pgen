//! Regex construction-census probe — the REPRESENTATION-ROAD STEP-0
//! consumed-vs-doomed instrument (RGX-0078.5.j.1, slice PGEN-RGX-0078-0123).
//!
//! QUESTION IT ANSWERS: of everything a bare (fused-graph) regex parse
//! CONSTRUCTS — arena `ParseNode`s, `PgenValue` array items, object pairs,
//! rendered strings — how much survives into the committed AST (CONSUMED)
//! versus is built and then folded away or discarded (SCAFFOLD + DOOMED)?
//! RE-PROFILE #16 (`PGEN-RGX-0078-0120`) pinned the value-construction/
//! allocation cluster at ≈45–50% of the parse window (`NodeArena::alloc`
//! 11.6% + `NeverFreeBump` 9.1% + `ParseContent::clone` 4.7% +
//! `to_shaped_value` 2.7% + `cascade_build_*` 5.8% + TEARDOWN + memmove);
//! this probe measures the POPULATIONS behind those symbols so the road's
//! first slice can be priced per-population on bench minima (the ×5 lesson),
//! never on sampled shares.
//!
//! METHOD (exact counts, deterministic, no sampling):
//!   - ARENA side: `NodeArena::census()` after a bare parse = every item the
//!     four arenas allocated (committed + scaffolding + doomed speculation).
//!     `typed_arena::Arena::len()` counts items, so an `alloc_extend` slice
//!     of N values contributes N.
//!   - CONSUMED side: a pointer-identity walk of the committed root —
//!     unique `ParseNode`s (arena slots reachable from the final AST),
//!     unique `PgenValue` array/object slice items, total reference visits
//!     (a thin-memo boundary splice can share one arena node across several
//!     tree positions, so unique-vs-refs is reported explicitly).
//!   - The difference arena − consumed = the parse's dead construction:
//!     build-pass fold scaffolding (element wrapper nodes + folded-away
//!     intermediate values) + doomed speculative boundary builds + memo-clone
//!     copies whose original or copy never reached the tree.
//!   - Per-rule committed-node histogram: the dynamic weight that joins the
//!     static per-rule fold census (which build functions allocate what) to
//!     corpus populations.
//!
//! The committed ROOT node is returned by value (not arena-allocated), so the
//! walk's unique-node count includes exactly one non-arena node; the printed
//! `consumed_nodes` subtracts it to stay arena-comparable.
//!
//! The canonical `regex_perf_probe` timing instrument and the `-0092`
//! `regex_alloc_census_probe` (malloc-event side) are deliberately UNTOUCHED;
//! this sibling bin reads arena populations the allocator census cannot see
//! (arena bumps are not per-item malloc events).
//!
//! V1 STEP-0 EXTENSION (RGX-0078.5.j.4, `PGEN-RGX-0078-0152`): `--case-file`
//! censuses arbitrary corpus cells (a JSONL of `{"id", "pattern"}` rows — the
//! `regex_perf_probe` corpus-case shape) instead of the built-in 8-pattern
//! bench, and DEBUG builds additionally report the
//! `shaped_conversion_census` exact counters per cell: `to_shaped_value`
//! conversions by input variant (+ `Sequence`/`Quantified` element totals),
//! `ParseContent::clone` populations by variant (+ cloned Vec element
//! totals), and arena slice-allocation calls — the build-value-pass
//! population sizing the `-0151` re-steer demands before any V1 pricing.
//!
//! Usage:
//!   cargo build --features generated_parsers --bin regex_construction_census_probe
//!   ./target/debug/regex_construction_census_probe [--repeat N] [--per-rule] \
//!       [--case-file cells.jsonl]

use std::collections::BTreeMap;

#[cfg(debug_assertions)]
use pgen::ast_pipeline::shaped_conversion_census::{self, ShapedConversionCensus};

/// Same corpus as `regex_perf_probe` (the campaign's 8-pattern bench).
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct Census {
    accepted: bool,
    /// Arena populations (allocated over the whole parse).
    arena_nodes: usize,
    arena_values: usize,
    arena_pairs: usize,
    arena_strings: usize,
    /// Consumed populations (reachable from the committed root).
    consumed_nodes: usize,
    consumed_node_refs: usize,
    consumed_values: usize,
    consumed_pairs: usize,
    /// Per-rule committed (unique) node histogram.
    per_rule: BTreeMap<&'static str, usize>,
    /// Debug-build conversion/clone/alloc counters over the whole parse
    /// (exact, thread-local, reset per census; part of the determinism
    /// assertion via `PartialEq`).
    #[cfg(debug_assertions)]
    conv: ShapedConversionCensus,
}

#[cfg(feature = "generated_parsers")]
mod walk {
    use super::*;
    use pgen::ast_pipeline::pgen_value::PgenValue;
    use pgen::ast_pipeline::{ParseContent, ParseNode};
    use std::collections::HashSet;

    #[derive(Default)]
    pub struct WalkState {
        pub node_ids: HashSet<usize>,
        pub node_refs: usize,
        pub value_slice_ids: HashSet<(usize, usize)>,
        pub value_items: usize,
        pub pair_slice_ids: HashSet<(usize, usize)>,
        pub pair_items: usize,
        pub per_rule: BTreeMap<&'static str, usize>,
    }

    pub fn walk_node<'i>(node: &ParseNode<'i>, st: &mut WalkState) {
        st.node_refs += 1;
        if !st.node_ids.insert(node as *const ParseNode<'i> as usize) {
            // Already counted (a shared thin-memo boundary splice); its
            // subtree is already fully accounted.
            return;
        }
        *st.per_rule.entry(node.rule_name).or_insert(0) += 1;
        match &node.content {
            ParseContent::Terminal(_) | ParseContent::TransformedTerminal(_) => {}
            ParseContent::Shaped(value) => walk_value(value, st),
            ParseContent::Alternative(child) => walk_node(child, st),
            ParseContent::Sequence(children) | ParseContent::Quantified(children, _) => {
                for child in children {
                    walk_node(child, st);
                }
            }
        }
    }

    pub fn walk_value<'i>(value: &PgenValue<'i>, st: &mut WalkState) {
        match value {
            PgenValue::Array(items) => {
                if st
                    .value_slice_ids
                    .insert((items.as_ptr() as usize, items.len()))
                {
                    st.value_items += items.len();
                }
                for item in *items {
                    walk_value(item, st);
                }
            }
            PgenValue::Object(pairs) => {
                if st
                    .pair_slice_ids
                    .insert((pairs.as_ptr() as usize, pairs.len()))
                {
                    st.pair_items += pairs.len();
                }
                for (_, item) in *pairs {
                    walk_value(item, st);
                }
            }
            PgenValue::Null
            | PgenValue::Bool(_)
            | PgenValue::Int(_)
            | PgenValue::UInt(_)
            | PgenValue::Float(_)
            | PgenValue::Str(_) => {}
        }
    }
}

#[cfg(feature = "generated_parsers")]
fn census_one(input: &str) -> Census {
    use pgen::generated_parsers::regex::RegexParser;

    // The exact bare-parse construction `regex_perf_probe::time_one_parse`
    // times (no diagnostic consumer is touched, so the parse stays BARE =
    // the fused cascade graph).
    #[cfg(debug_assertions)]
    shaped_conversion_census::reset();
    let node_arena = pgen::ast_pipeline::NodeArena::new();
    let mut parser = RegexParser::new(
        input,
        &node_arena,
        pgen::ast_pipeline::runtime_logger_box("regex_construction_census_probe"),
    );
    let parsed = parser.parse_full_regex();
    // Snapshot BEFORE the committed-AST walk: the walk is read-only, but
    // keeping the window parse-exact makes the counters attributable to the
    // parse alone by construction.
    #[cfg(debug_assertions)]
    let conv = shaped_conversion_census::snapshot();
    let arena = node_arena.census();

    let mut st = walk::WalkState::default();
    let accepted = match &parsed {
        Ok(root) => {
            walk::walk_node(root, &mut st);
            true
        }
        Err(_) => false,
    };

    // The committed root is a by-value node, not an arena slot — subtract it
    // so consumed_nodes compares against arena_nodes like-for-like. (The
    // root's rule stays in the per-rule histogram: it IS a committed node.)
    let consumed_nodes = st.node_ids.len().saturating_sub(1);

    Census {
        accepted,
        arena_nodes: arena.nodes,
        arena_values: arena.shaped_values,
        arena_pairs: arena.shaped_pairs,
        arena_strings: arena.rendered_strings,
        consumed_nodes,
        consumed_node_refs: st.node_refs,
        consumed_values: st.value_items,
        consumed_pairs: st.pair_items,
        per_rule: st.per_rule,
        #[cfg(debug_assertions)]
        conv,
    }
}

#[cfg(not(feature = "generated_parsers"))]
fn census_one(_input: &str) -> Census {
    panic!("regex_construction_census_probe requires --features generated_parsers");
}

fn parse_args() -> (usize, bool, Option<String>) {
    let mut repeat = 3usize;
    let mut per_rule = false;
    let mut case_file: Option<String> = None;
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--repeat" => {
                i += 1;
                repeat = args[i].parse().expect("--repeat expects integer");
            }
            "--per-rule" => per_rule = true,
            "--case-file" => {
                i += 1;
                case_file = Some(args[i].clone());
            }
            "-h" | "--help" => {
                eprintln!(
                    "regex_construction_census_probe — arena-population vs committed-AST census per bare\nregex parse (the RGX-0078.5.j.1 consumed-vs-doomed instrument; V1 STEP-0 build-value\ncounters in debug builds).\n\nUsage:\n  regex_construction_census_probe [--repeat N] [--per-rule] [--case-file cells.jsonl]\n\n--case-file replaces the built-in 8-pattern bench with the given JSONL of\n{{\"id\", \"pattern\"}} rows (the regex_perf_probe corpus-case shape)."
                );
                std::process::exit(0);
            }
            other => {
                eprintln!("unknown arg: {}", other);
                std::process::exit(2);
            }
        }
        i += 1;
    }
    (repeat.max(1), per_rule, case_file)
}

/// Load `(id, pattern)` rows from a corpus-case JSONL (`id` + `pattern`
/// string fields per line; blank lines skipped) — the same row shape the
/// `regex_perf_probe` corpus mode consumes.
fn load_case_file(path: &str) -> Vec<(String, String)> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("failed to read case file '{}': {}", path, e);
        std::process::exit(2);
    });
    let mut cases = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let row: serde_json::Value = serde_json::from_str(line).unwrap_or_else(|e| {
            eprintln!("failed to decode case at line {}: {}", index + 1, e);
            std::process::exit(2);
        });
        let id = row
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| {
                eprintln!("case at line {} has no string 'id'", index + 1);
                std::process::exit(2);
            })
            .to_string();
        let pattern = row
            .get("pattern")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| {
                eprintln!("case at line {} has no string 'pattern'", index + 1);
                std::process::exit(2);
            })
            .to_string();
        cases.push((id, pattern));
    }
    if cases.is_empty() {
        eprintln!("case file '{}' contains no cases", path);
        std::process::exit(2);
    }
    cases
}

fn main() {
    let (repeat, per_rule, case_file) = parse_args();
    let cases: Vec<(String, String)> = match &case_file {
        Some(path) => load_case_file(path),
        None => PATTERNS
            .iter()
            .map(|(name, input)| (name.to_string(), input.to_string()))
            .collect(),
    };
    println!(
        "# Regex construction-census probe — REPRESENTATION-ROAD STEP-0 (RGX-0078.5.j.1)"
    );
    if let Some(path) = &case_file {
        println!("# case-file mode: {} ({} cases)", path, cases.len());
    }
    println!("# arena_* = items allocated over the parse | cons_* = unique items reachable from the committed AST");
    println!("# dead_* = arena - consumed (fold scaffolding + doomed speculation + unconsumed memo-clone copies)");
    println!("# repeat={} (censuses asserted identical across repeats)", repeat);
    #[cfg(debug_assertions)]
    println!("# debug build: shaped-conversion/clone/alloc counters reported per case (V1 STEP-0)");
    #[cfg(not(debug_assertions))]
    println!("# RELEASE build: shaped-conversion counters unavailable (debug_assertions off)");
    println!();

    // Warmup: one full census of every pattern so lazily-initialized global
    // state (logger, caches) is paid before any reported run.
    for (_, input) in &cases {
        let _ = census_one(input);
    }

    println!(
        "{:<18} {:>3} {:>7} {:>7} {:>7} {:>7} {:>7} {:>7} {:>7} {:>7} {:>7} {:>7} {:>7}",
        "pattern",
        "ok",
        "ar_nod",
        "co_nod",
        "dead_n",
        "dead%",
        "refs",
        "ar_val",
        "co_val",
        "dead_v",
        "ar_pr",
        "co_pr",
        "ar_str",
    );
    println!("{}", "-".repeat(122));

    let mut det_ok = true;
    let mut totals = (0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
    let mut per_rule_rows: Vec<(String, Census)> = Vec::new();
    for (name, input) in &cases {
        let first = census_one(input);
        for r in 1..repeat {
            let again = census_one(input);
            if again != first {
                det_ok = false;
                eprintln!(
                    "DETERMINISM VIOLATION on '{}' repeat {}: {:?} != {:?}",
                    name, r, again, first
                );
            }
        }
        let dead_nodes = first.arena_nodes.saturating_sub(first.consumed_nodes);
        let dead_pct = 100.0 * dead_nodes as f64 / first.arena_nodes.max(1) as f64;
        let dead_values = first.arena_values.saturating_sub(first.consumed_values);
        println!(
            "{:<18} {:>3} {:>7} {:>7} {:>7} {:>6.1}% {:>7} {:>7} {:>7} {:>7} {:>7} {:>7} {:>7}",
            name,
            if first.accepted { "y" } else { "N" },
            first.arena_nodes,
            first.consumed_nodes,
            dead_nodes,
            dead_pct,
            first.consumed_node_refs,
            first.arena_values,
            first.consumed_values,
            dead_values,
            first.arena_pairs,
            first.consumed_pairs,
            first.arena_strings,
        );
        totals.0 += first.arena_nodes;
        totals.1 += first.consumed_nodes;
        totals.2 += first.arena_values;
        totals.3 += first.consumed_values;
        totals.4 += first.arena_pairs;
        totals.5 += first.consumed_pairs;
        totals.6 += first.arena_strings;
        per_rule_rows.push((name.to_string(), first));
    }

    println!("{}", "-".repeat(122));
    println!(
        "{:<18} {:>3} {:>7} {:>7} {:>7} {:>6.1}% {:>7} {:>7} {:>7} {:>7} {:>7} {:>7} {:>7}",
        "TOTAL",
        "",
        totals.0,
        totals.1,
        totals.0 - totals.1,
        100.0 * (totals.0 - totals.1) as f64 / totals.0.max(1) as f64,
        "",
        totals.2,
        totals.3,
        totals.2 - totals.3,
        totals.4,
        totals.5,
        totals.6,
    );

    if per_rule {
        for (name, census) in &per_rule_rows {
            println!();
            println!("== per-rule committed (unique) nodes: {} ==", name);
            let mut rows: Vec<(&&str, &usize)> = census.per_rule.iter().collect();
            rows.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
            for (rule, count) in rows {
                println!("  {:>6}  {}", count, rule);
            }
        }
    }

    #[cfg(debug_assertions)]
    for (name, census) in &per_rule_rows {
        let c = &census.conv;
        println!();
        println!("== shaped-conversion census (V1 STEP-0): {} ==", name);
        println!(
            "  to_shaped_value calls: terminal={} transformed_parsed={} transformed_wrapped={} shaped={} alternative={} sequence={} quantified={} | sequence_items={}",
            c.to_shaped_terminal,
            c.to_shaped_transformed_parsed,
            c.to_shaped_transformed_wrapped,
            c.to_shaped_shaped,
            c.to_shaped_alternative,
            c.to_shaped_sequence,
            c.to_shaped_quantified,
            c.to_shaped_sequence_items,
        );
        println!(
            "  ParseContent::clone:   terminal={} transformed={} shaped={} sequence={} alternative={} quantified={} | cloned_vec_items={}",
            c.clone_terminal,
            c.clone_transformed,
            c.clone_shaped,
            c.clone_sequence,
            c.clone_alternative,
            c.clone_quantified,
            c.clone_sequence_items,
        );
        println!(
            "  arena slice calls:     value_slices={} pair_slices={} rendered_strings={}",
            c.arena_value_slices, c.arena_pair_slices, c.arena_rendered_strings,
        );
    }

    println!();
    if det_ok {
        println!(
            "# determinism: censuses identical across {} repeats for every pattern",
            repeat
        );
    } else {
        println!("# DETERMINISM VIOLATION observed — see stderr");
        std::process::exit(1);
    }
}
