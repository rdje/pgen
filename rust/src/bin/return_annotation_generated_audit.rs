#[cfg(not(feature = "generated_parsers"))]
fn main() {
    eprintln!("return_annotation_generated_audit requires --features generated_parsers");
    std::process::exit(2);
}

#[cfg(feature = "generated_parsers")]
fn main() {
    if let Err(err) = run() {
        eprintln!("error: {}", err);
        std::process::exit(1);
    }
}

#[cfg(feature = "generated_parsers")]
fn run() -> Result<(), String> {
    use std::collections::BTreeSet;
    use std::env;
    use std::fs;

    let sample_paths: Vec<String> = env::args().skip(1).collect();
    if sample_paths.is_empty() {
        return Err(
            "usage: cargo run --features generated_parsers --bin return_annotation_generated_audit -- <samples.txt> [more_samples.txt ...]".to_string(),
        );
    }

    let mut samples = BTreeSet::new();
    for path in &sample_paths {
        let contents = fs::read_to_string(path)
            .map_err(|err| format!("failed to read sample file '{}': {}", path, err))?;
        for line in contents.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                samples.insert(trimmed.to_string());
            }
        }
    }

    if samples.is_empty() {
        return Err("no non-empty samples found".to_string());
    }

    let mut audited = 0usize;
    for sample in samples {
        let ast = parse_generated_return_annotation(&sample)?;
        serde_json::to_value(&ast).map_err(|err| {
            format!(
                "failed to serialize typed AST for generated sample '{}': {}",
                sample, err
            )
        })?;
        audited += 1;
    }

    println!(
        "return_annotation_generated_audit: audited_unique_samples={}",
        audited
    );
    Ok(())
}

#[cfg(feature = "generated_parsers")]
fn parse_generated_return_annotation(
    sample: &str,
) -> Result<pgen::ast_pipeline::UnifiedReturnAST, String> {
    use pgen::ast_pipeline::{UnifiedReturnAST, runtime_logger_box};
    use pgen::generated_parsers::return_annotation::Return_annotationParser;

    let parser_logger = runtime_logger_box("generated.return_annotation.audit.parse");
    let mut parser = Return_annotationParser::new(sample, parser_logger);
    let parse_tree = parser
        .parse_full_return_annotation()
        .map_err(|err| format!("generated parser failed for '{}': {}", sample, err))?;
    let ast_logger = runtime_logger_box("generated.return_annotation.audit.typed_ast");
    UnifiedReturnAST::parse_generated_return_annotation(sample, &parse_tree, &*ast_logger)
        .map_err(|err| {
            format!(
                "generated parse tree -> typed AST failed for '{}': {}",
                sample, err
            )
        })
}
