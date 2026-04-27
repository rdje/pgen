//! Dump the declared-annotation inventory for a grammar's frontend JSON in
//! the manifest format the AST-shape contract gate expects. Run as:
//!
//! ```bash
//! cargo run --bin dump_declared_annotation_inventory -- generated/regex.json
//! ```
//!
//! Prints a JSON array of `{rule, branch_index, annotation_type,
//! normalized_text}` records suitable for embedding under the manifest's
//! `declared_annotation_inventory.annotations` field. This is the helper
//! used to produce the initial inventory snapshot for each manifest; once
//! tracked, the manifest's inventory is regression-locked against future
//! grammar changes by `ast_shape_contract::run_manifest`.

use pgen::ast_shape_contract::extract_declared_annotations_from_json;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!(
            "usage: dump_declared_annotation_inventory <path/to/generated/<grammar>.json>"
        );
        std::process::exit(2);
    }
    match extract_declared_annotations_from_json(&args[1]) {
        Ok(inventory) => {
            let json_records: Vec<serde_json::Value> = inventory
                .iter()
                .map(|ann| {
                    serde_json::json!({
                        "rule": ann.rule,
                        "branch_index": ann.branch_index,
                        "annotation_type": ann.annotation_type,
                        "normalized_text": ann.normalized_text,
                    })
                })
                .collect();
            let pretty = serde_json::to_string_pretty(&json_records)
                .expect("serialise inventory to pretty JSON");
            println!("{}", pretty);
            eprintln!("[dump_declared_annotation_inventory] {} annotations", inventory.len());
        }
        Err(err) => {
            eprintln!("error: {}", err);
            std::process::exit(1);
        }
    }
}
