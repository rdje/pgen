use pgen::ast_pipeline::{
    ParseError, ParseResult, SemanticRuntimeCheckpoint, SemanticRuntimeDelta,
};
use std::mem::{align_of, needs_drop, size_of};

fn row<T>(name: &str) {
    println!(
        "{name:32} size={:3} align={:2} needs_drop={}",
        size_of::<T>(),
        align_of::<T>(),
        needs_drop::<T>(),
    );
}

fn main() {
    row::<ParseError>("ParseError");
    row::<ParseResult<()>>("ParseResult<()> (Result ABI)");
    row::<SemanticRuntimeCheckpoint>("SemanticRuntimeCheckpoint");
    row::<SemanticRuntimeDelta>("SemanticRuntimeDelta");
    row::<Option<SemanticRuntimeDelta>>("Option<SemanticRuntimeDelta>");
}
