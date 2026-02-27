use std::env;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rustc-check-cfg=cfg(has_generated_systemverilog_parser)");
    println!("cargo:rerun-if-env-changed=PGEN_SYSTEMVERILOG_PARSER_PATH");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()));
    let configured_path = env::var("PGEN_SYSTEMVERILOG_PARSER_PATH")
        .unwrap_or_else(|_| "../generated/systemverilog_parser.rs".to_string());

    let resolved = resolve_path(&manifest_dir, &configured_path);
    println!("cargo:rerun-if-changed={}", resolved.to_string_lossy());

    if resolved.is_file() {
        println!("cargo:rustc-cfg=has_generated_systemverilog_parser");
        println!(
            "cargo:rustc-env=PGEN_SYSTEMVERILOG_PARSER_PATH_RESOLVED={}",
            resolved.to_string_lossy()
        );
    }
}

fn resolve_path(manifest_dir: &Path, raw: &str) -> PathBuf {
    let path = Path::new(raw);
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        manifest_dir.join(path)
    };
    match joined.canonicalize() {
        Ok(abs) => abs,
        Err(_) => joined,
    }
}
