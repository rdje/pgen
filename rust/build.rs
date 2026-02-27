use std::env;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rustc-check-cfg=cfg(has_generated_systemverilog_parser)");
    println!("cargo:rustc-check-cfg=cfg(has_generated_vhdl_parser)");
    println!("cargo:rerun-if-env-changed=PGEN_SYSTEMVERILOG_PARSER_PATH");
    println!("cargo:rerun-if-env-changed=PGEN_VHDL_PARSER_PATH");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()));
    let systemverilog_configured_path = env::var("PGEN_SYSTEMVERILOG_PARSER_PATH")
        .unwrap_or_else(|_| "../generated/systemverilog_parser.rs".to_string());
    let vhdl_configured_path = env::var("PGEN_VHDL_PARSER_PATH")
        .unwrap_or_else(|_| "../generated/vhdl_parser.rs".to_string());

    let systemverilog_resolved = resolve_path(&manifest_dir, &systemverilog_configured_path);
    println!(
        "cargo:rerun-if-changed={}",
        systemverilog_resolved.to_string_lossy()
    );
    let vhdl_resolved = resolve_path(&manifest_dir, &vhdl_configured_path);
    println!("cargo:rerun-if-changed={}", vhdl_resolved.to_string_lossy());

    if systemverilog_resolved.is_file() {
        println!("cargo:rustc-cfg=has_generated_systemverilog_parser");
        println!(
            "cargo:rustc-env=PGEN_SYSTEMVERILOG_PARSER_PATH_RESOLVED={}",
            systemverilog_resolved.to_string_lossy()
        );
    }

    if vhdl_resolved.is_file() {
        println!("cargo:rustc-cfg=has_generated_vhdl_parser");
        println!(
            "cargo:rustc-env=PGEN_VHDL_PARSER_PATH_RESOLVED={}",
            vhdl_resolved.to_string_lossy()
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
