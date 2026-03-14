use std::env;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rustc-check-cfg=cfg(has_generated_systemverilog_parser)");
    println!("cargo:rustc-check-cfg=cfg(has_generated_systemverilog_preprocessor_parser)");
    println!("cargo:rustc-check-cfg=cfg(has_generated_vhdl_parser)");
    println!("cargo:rustc-check-cfg=cfg(has_generated_json_parser)");
    println!("cargo:rustc-check-cfg=cfg(has_generated_regex_parser)");
    println!("cargo:rustc-check-cfg=cfg(has_generated_rtl_const_expr_parser)");
    println!("cargo:rerun-if-env-changed=PGEN_EBNF_PARSER_PATH");
    println!("cargo:rerun-if-env-changed=PGEN_JSON_PARSER_PATH");
    println!("cargo:rerun-if-env-changed=PGEN_REGEX_PARSER_PATH");
    println!("cargo:rerun-if-env-changed=PGEN_SYSTEMVERILOG_PARSER_PATH");
    println!("cargo:rerun-if-env-changed=PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH");
    println!("cargo:rerun-if-env-changed=PGEN_VHDL_PARSER_PATH");
    println!("cargo:rerun-if-env-changed=PGEN_RTL_CONST_EXPR_PARSER_PATH");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()));
    let source_dir = manifest_dir.join("src");
    let bin_source_dir = source_dir.join("bin");
    let ebnf_configured_path =
        env::var("PGEN_EBNF_PARSER_PATH").unwrap_or_else(|_| "../generated/ebnf.rs".to_string());
    let json_configured_path = env::var("PGEN_JSON_PARSER_PATH")
        .unwrap_or_else(|_| "../generated/json_parser.rs".to_string());
    let regex_configured_path = env::var("PGEN_REGEX_PARSER_PATH")
        .unwrap_or_else(|_| "../generated/regex_parser.rs".to_string());
    let systemverilog_configured_path = env::var("PGEN_SYSTEMVERILOG_PARSER_PATH")
        .unwrap_or_else(|_| "../generated/systemverilog_parser.rs".to_string());
    let systemverilog_preprocessor_configured_path =
        env::var("PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH")
            .unwrap_or_else(|_| "../generated/systemverilog_preprocessor_parser.rs".to_string());
    let vhdl_configured_path = env::var("PGEN_VHDL_PARSER_PATH")
        .unwrap_or_else(|_| "../generated/vhdl_parser.rs".to_string());
    let rtl_const_expr_configured_path = env::var("PGEN_RTL_CONST_EXPR_PARSER_PATH")
        .unwrap_or_else(|_| "../generated/rtl_const_expr_parser.rs".to_string());

    let ebnf_resolved = resolve_path(&manifest_dir, &ebnf_configured_path);
    println!("cargo:rerun-if-changed={}", ebnf_resolved.to_string_lossy());
    let json_resolved = resolve_path(&manifest_dir, &json_configured_path);
    println!("cargo:rerun-if-changed={}", json_resolved.to_string_lossy());
    let regex_resolved = resolve_path(&manifest_dir, &regex_configured_path);
    println!("cargo:rerun-if-changed={}", regex_resolved.to_string_lossy());
    let systemverilog_resolved = resolve_path(&manifest_dir, &systemverilog_configured_path);
    println!(
        "cargo:rerun-if-changed={}",
        systemverilog_resolved.to_string_lossy()
    );
    let systemverilog_preprocessor_resolved =
        resolve_path(&manifest_dir, &systemverilog_preprocessor_configured_path);
    println!(
        "cargo:rerun-if-changed={}",
        systemverilog_preprocessor_resolved.to_string_lossy()
    );
    let vhdl_resolved = resolve_path(&manifest_dir, &vhdl_configured_path);
    println!("cargo:rerun-if-changed={}", vhdl_resolved.to_string_lossy());
    let rtl_const_expr_resolved = resolve_path(&manifest_dir, &rtl_const_expr_configured_path);
    println!(
        "cargo:rerun-if-changed={}",
        rtl_const_expr_resolved.to_string_lossy()
    );
    println!(
        "cargo:rustc-env=PGEN_EBNF_PARSER_PATH_RESOLVED={}",
        relativize_for_include(&source_dir, &ebnf_resolved).display()
    );
    println!(
        "cargo:rustc-env=PGEN_EBNF_PARSER_PATH_RESOLVED_BIN={}",
        relativize_for_include(&bin_source_dir, &ebnf_resolved).display()
    );

    if json_resolved.is_file() {
        println!("cargo:rustc-cfg=has_generated_json_parser");
        println!(
            "cargo:rustc-env=PGEN_JSON_PARSER_PATH_RESOLVED={}",
            relativize_for_include(&source_dir, &json_resolved).display()
        );
    }

    if regex_resolved.is_file() {
        println!("cargo:rustc-cfg=has_generated_regex_parser");
        println!(
            "cargo:rustc-env=PGEN_REGEX_PARSER_PATH_RESOLVED={}",
            relativize_for_include(&source_dir, &regex_resolved).display()
        );
    }

    if systemverilog_resolved.is_file() {
        println!("cargo:rustc-cfg=has_generated_systemverilog_parser");
        println!(
            "cargo:rustc-env=PGEN_SYSTEMVERILOG_PARSER_PATH_RESOLVED={}",
            relativize_for_include(&source_dir, &systemverilog_resolved).display()
        );
    }

    if systemverilog_preprocessor_resolved.is_file() {
        println!("cargo:rustc-cfg=has_generated_systemverilog_preprocessor_parser");
        println!(
            "cargo:rustc-env=PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH_RESOLVED={}",
            relativize_for_include(&source_dir, &systemverilog_preprocessor_resolved).display()
        );
    }

    if vhdl_resolved.is_file() {
        println!("cargo:rustc-cfg=has_generated_vhdl_parser");
        println!(
            "cargo:rustc-env=PGEN_VHDL_PARSER_PATH_RESOLVED={}",
            relativize_for_include(&source_dir, &vhdl_resolved).display()
        );
    }

    if rtl_const_expr_resolved.is_file() {
        println!("cargo:rustc-cfg=has_generated_rtl_const_expr_parser");
        println!(
            "cargo:rustc-env=PGEN_RTL_CONST_EXPR_PARSER_PATH_RESOLVED={}",
            relativize_for_include(&source_dir, &rtl_const_expr_resolved).display()
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

fn relativize_for_include(source_dir: &Path, target: &Path) -> PathBuf {
    let normalized_source_dir = source_dir
        .canonicalize()
        .unwrap_or_else(|_| source_dir.to_path_buf());
    let normalized_target = target.canonicalize().unwrap_or_else(|_| target.to_path_buf());
    path_relative_from(&normalized_target, &normalized_source_dir)
        .unwrap_or_else(|| normalized_target.clone())
}

fn path_relative_from(path: &Path, base: &Path) -> Option<PathBuf> {
    let path_components: Vec<_> = path.components().collect();
    let base_components: Vec<_> = base.components().collect();

    if path_components.is_empty() || base_components.is_empty() {
        return Some(path.to_path_buf());
    }

    if path_components.first() != base_components.first() {
        return None;
    }

    let common_prefix_len = path_components
        .iter()
        .zip(base_components.iter())
        .take_while(|(lhs, rhs)| lhs == rhs)
        .count();

    let mut relative = PathBuf::new();
    for _ in common_prefix_len..base_components.len() {
        relative.push("..");
    }
    for component in &path_components[common_prefix_len..] {
        relative.push(component.as_os_str());
    }

    Some(relative)
}
