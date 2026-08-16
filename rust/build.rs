use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

fn main() {
    println!("cargo:rustc-check-cfg=cfg(has_generated_ebnf_parser)");
    println!("cargo:rustc-check-cfg=cfg(has_generated_systemverilog_parser)");
    println!("cargo:rustc-check-cfg=cfg(has_generated_systemverilog_preprocessor_parser)");
    println!("cargo:rustc-check-cfg=cfg(has_generated_vhdl_parser)");
    println!("cargo:rustc-check-cfg=cfg(has_generated_json_parser)");
    println!("cargo:rustc-check-cfg=cfg(has_generated_regex_parser)");
    println!("cargo:rustc-check-cfg=cfg(has_generated_rtl_const_expr_parser)");
    println!("cargo:rustc-check-cfg=cfg(has_generated_rtl_frontend_parser)");
    // PARSE-HARNESS.2 — the blessed scratch-register slot.
    println!("cargo:rustc-check-cfg=cfg(has_generated_scratch_parser)");
    println!("cargo:rerun-if-env-changed=PGEN_EBNF_PARSER_PATH");
    println!("cargo:rerun-if-env-changed=PGEN_JSON_PARSER_PATH");
    println!("cargo:rerun-if-env-changed=PGEN_REGEX_PARSER_PATH");
    println!("cargo:rerun-if-env-changed=PGEN_SYSTEMVERILOG_PARSER_PATH");
    println!("cargo:rerun-if-env-changed=PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH");
    println!("cargo:rerun-if-env-changed=PGEN_VHDL_PARSER_PATH");
    println!("cargo:rerun-if-env-changed=PGEN_RTL_CONST_EXPR_PARSER_PATH");
    println!("cargo:rerun-if-env-changed=PGEN_RTL_FRONTEND_PARSER_PATH");
    println!("cargo:rerun-if-env-changed=PGEN_SCRATCH_PARSER_PATH");

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
    let rtl_frontend_configured_path = env::var("PGEN_RTL_FRONTEND_PARSER_PATH")
        .unwrap_or_else(|_| "../generated/rtl_frontend_parser.rs".to_string());
    let scratch_configured_path = env::var("PGEN_SCRATCH_PARSER_PATH")
        .unwrap_or_else(|_| "../generated/scratch_parser.rs".to_string());

    let ebnf_resolved = resolve_path(&manifest_dir, &ebnf_configured_path);
    println!("cargo:rerun-if-changed={}", ebnf_resolved.to_string_lossy());
    let json_resolved = resolve_path(&manifest_dir, &json_configured_path);
    println!("cargo:rerun-if-changed={}", json_resolved.to_string_lossy());
    let regex_resolved = resolve_path(&manifest_dir, &regex_configured_path);
    println!(
        "cargo:rerun-if-changed={}",
        regex_resolved.to_string_lossy()
    );
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
    let rtl_frontend_resolved = resolve_path(&manifest_dir, &rtl_frontend_configured_path);
    println!(
        "cargo:rerun-if-changed={}",
        rtl_frontend_resolved.to_string_lossy()
    );
    let scratch_resolved = resolve_path(&manifest_dir, &scratch_configured_path);
    println!(
        "cargo:rerun-if-changed={}",
        scratch_resolved.to_string_lossy()
    );
    // The EBNF generated parser is treated like any other generated parser:
    // its `include!()` site is gated on the `has_generated_ebnf_parser`
    // cfg flag, which is set only when `generated/ebnf.rs` exists on disk.
    // This breaks the cold-clone chicken-and-egg where the binary that
    // GENERATES `generated/ebnf.rs` also needs to compile against it.
    if ebnf_resolved.is_file() {
        println!("cargo:rustc-cfg=has_generated_ebnf_parser");
        println!(
            "cargo:rustc-env=PGEN_EBNF_PARSER_PATH_RESOLVED={}",
            relativize_for_include(&source_dir, &ebnf_resolved).display()
        );
        println!(
            "cargo:rustc-env=PGEN_EBNF_PARSER_PATH_RESOLVED_BIN={}",
            relativize_for_include(&bin_source_dir, &ebnf_resolved).display()
        );
        emit_parser_fingerprint("PGEN_EBNF_PARSER_SHA256", &ebnf_resolved);
    }

    if json_resolved.is_file() {
        println!("cargo:rustc-cfg=has_generated_json_parser");
        println!(
            "cargo:rustc-env=PGEN_JSON_PARSER_PATH_RESOLVED={}",
            relativize_for_include(&source_dir, &json_resolved).display()
        );
        emit_parser_fingerprint("PGEN_JSON_PARSER_SHA256", &json_resolved);
    }

    if regex_resolved.is_file() {
        println!("cargo:rustc-cfg=has_generated_regex_parser");
        println!(
            "cargo:rustc-env=PGEN_REGEX_PARSER_PATH_RESOLVED={}",
            relativize_for_include(&source_dir, &regex_resolved).display()
        );
        emit_parser_fingerprint("PGEN_REGEX_PARSER_SHA256", &regex_resolved);
    }

    if systemverilog_resolved.is_file() {
        println!("cargo:rustc-cfg=has_generated_systemverilog_parser");
        println!(
            "cargo:rustc-env=PGEN_SYSTEMVERILOG_PARSER_PATH_RESOLVED={}",
            relativize_for_include(&source_dir, &systemverilog_resolved).display()
        );
        emit_parser_fingerprint("PGEN_SYSTEMVERILOG_PARSER_SHA256", &systemverilog_resolved);
    }

    if systemverilog_preprocessor_resolved.is_file() {
        println!("cargo:rustc-cfg=has_generated_systemverilog_preprocessor_parser");
        println!(
            "cargo:rustc-env=PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_PATH_RESOLVED={}",
            relativize_for_include(&source_dir, &systemverilog_preprocessor_resolved).display()
        );
        emit_parser_fingerprint(
            "PGEN_SYSTEMVERILOG_PREPROCESSOR_PARSER_SHA256",
            &systemverilog_preprocessor_resolved,
        );
    }

    if vhdl_resolved.is_file() {
        println!("cargo:rustc-cfg=has_generated_vhdl_parser");
        println!(
            "cargo:rustc-env=PGEN_VHDL_PARSER_PATH_RESOLVED={}",
            relativize_for_include(&source_dir, &vhdl_resolved).display()
        );
        emit_parser_fingerprint("PGEN_VHDL_PARSER_SHA256", &vhdl_resolved);
    }

    if rtl_const_expr_resolved.is_file() {
        println!("cargo:rustc-cfg=has_generated_rtl_const_expr_parser");
        println!(
            "cargo:rustc-env=PGEN_RTL_CONST_EXPR_PARSER_PATH_RESOLVED={}",
            relativize_for_include(&source_dir, &rtl_const_expr_resolved).display()
        );
        emit_parser_fingerprint(
            "PGEN_RTL_CONST_EXPR_PARSER_SHA256",
            &rtl_const_expr_resolved,
        );
    }

    if rtl_frontend_resolved.is_file() {
        println!("cargo:rustc-cfg=has_generated_rtl_frontend_parser");
        println!(
            "cargo:rustc-env=PGEN_RTL_FRONTEND_PARSER_PATH_RESOLVED={}",
            relativize_for_include(&source_dir, &rtl_frontend_resolved).display()
        );
        emit_parser_fingerprint("PGEN_RTL_FRONTEND_PARSER_SHA256", &rtl_frontend_resolved);
    }

    // PARSE-HARNESS.2 — the scratch-register slot is gated exactly like the other
    // generated parsers: the cfg + include path are set ONLY when the artifact exists
    // (`make focus_scratch`). With no artifact, the slot is simply absent (additive,
    // never affecting a shipped grammar), so a clean checkout compiles unchanged.
    if scratch_resolved.is_file() {
        println!("cargo:rustc-cfg=has_generated_scratch_parser");
        println!(
            "cargo:rustc-env=PGEN_SCRATCH_PARSER_PATH_RESOLVED={}",
            relativize_for_include(&source_dir, &scratch_resolved).display()
        );
        emit_parser_fingerprint("PGEN_SCRATCH_PARSER_SHA256", &scratch_resolved);
    }
}

/// `ENGINE-UNIVERSAL-SERVICES.24` (ii′) — publish the sha256 of the generated parser THIS BUILD
/// compiled against, as `PGEN_<FAMILY>_PARSER_SHA256`, so the compiled binary can be asked which
/// parser it embeds.
///
/// ⛔ THE DEFECT THIS EXISTS FOR, demonstrated live rather than argued. `PARSE-COST-RATCHET`'s
/// identity table pins four INPUTS — grammar, generated parser, instrument, sample files — and
/// not the EXECUTABLE that produces the numbers. With a release `parseability_probe` on disk
/// built from an experimental arm (left-recursion guard emission suppressed), the every-commit
/// tier printed *"the measurement cannot have moved"* while `nm` read 0 `_lr_guard` symbols in
/// the binary against a pinned parser declaring 6. Measured on four sampled files, that wrong
/// binary reports **762,345** rule entries where the shipped one reports **11,240,430** — and a
/// FALL is not a ratchet breach, it is a note inviting a rebaseline. The gap therefore fails in
/// the passing direction, which is why it is closed at the build rather than left to discipline.
///
/// ⭐ WHY BUILD TIME AND NOT EMIT TIME. The alternative was to emit the fingerprint INTO the
/// generated parser, which moves every generated artifact and re-baselines everything keyed on
/// them (the ratchet identity, the byte-identity controls in six gates, and
/// `GENERATED-REPRODUCIBILITY`). `build.rs` already resolves each parser path and already
/// declares `cargo:rerun-if-changed` for it, so it re-runs exactly when the parser moves: the
/// same property at ZERO generated bytes.
///
/// This is engine-universal on purpose — every resolved generated parser gets a fingerprint, not
/// just the one family whose ratchet needed it first.
fn emit_parser_fingerprint(env_name: &str, resolved: &Path) {
    match sha256_of_file(resolved) {
        Ok(digest) => println!("cargo:rustc-env={}={}", env_name, digest),
        // ⛔ NEVER a placeholder digest. A build that could not read the parser must leave the
        // variable UNSET, so `option_env!` reports absence and every consumer refuses; a
        // stand-in value would compare unequal to a real digest and read as "wrong parser"
        // rather than as "not measured", which is a different — and misleading — verdict.
        Err(err) => println!(
            "cargo:warning=could not fingerprint {} ({}); {} is left unset and any consumer of \
             the build-time parser fingerprint will report it as unavailable",
            resolved.display(),
            err,
            env_name
        ),
    }
}

/// Streaming sha256 of a file. Streamed in 1 MiB chunks because the SystemVerilog parser is
/// ~144 MB and a build script must not need that much resident memory to hash it.
fn sha256_of_file(path: &Path) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex_lower(&hasher.finalize()))
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{:02x}", b));
    }
    out
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
    let normalized_target = target
        .canonicalize()
        .unwrap_or_else(|_| target.to_path_buf());
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
