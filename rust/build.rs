use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

fn main() {
    // `ENGINE-UNIVERSAL-SERVICES.112` slice 2 — THE PARSER LIST IS DERIVED, NEVER TYPED. Every
    // `../generated/<name>_parser.rs` (plus any `PGEN_<NAME>_PARSER_PATH` override) is a registered
    // parser: this script emits its `has_generated_<name>_parser` cfg, its resolved include path,
    // its build-time fingerprint (`.24` (ii′)), and writes three files into OUT_DIR that the crate
    // includes — the module list (`lib.rs`), the registry table (`parser_registry.rs`) and the
    // family-hook module list (`families/mod.rs`). Nothing under `rust/src` names a language's
    // artifact any more (the `ENGINE-NEUTRALITY` doctrine reads for it); the two meta parsers the
    // engine cannot compile without (the annotation pair) are the only names this file spells,
    // and they are the engine's own languages.
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()));
    let source_dir = manifest_dir.join("src");
    let bin_source_dir = source_dir.join("bin");
    let generated_dir = resolve_path(&manifest_dir, "../generated");
    let families_dir = source_dir.join("families");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    println!("cargo:rerun-if-changed={}", generated_dir.to_string_lossy());
    println!("cargo:rerun-if-changed={}", families_dir.to_string_lossy());

    // The ebnf seed (`generated/ebnf.rs`) — the meta-grammar's own parser, compiled in under the
    // `ebnf_dual_run` feature. Its `include!()` site is gated on `has_generated_ebnf_parser`.
    println!("cargo:rustc-check-cfg=cfg(has_generated_ebnf_parser)");
    println!("cargo:rerun-if-env-changed=PGEN_EBNF_PARSER_PATH");
    let ebnf_configured_path =
        env::var("PGEN_EBNF_PARSER_PATH").unwrap_or_else(|_| "../generated/ebnf.rs".to_string());
    let ebnf_resolved = resolve_path(&manifest_dir, &ebnf_configured_path);
    println!("cargo:rerun-if-changed={}", ebnf_resolved.to_string_lossy());
    if ebnf_resolved.is_file() {
        println!("cargo:rustc-cfg=has_generated_ebnf_parser");
        println!(
            "cargo:rustc-env=PGEN_EBNF_PARSER_PATH_RESOLVED={}",
            relativize_for_include(&source_dir, &ebnf_resolved).display()
        );
        emit_parser_fingerprint("PGEN_EBNF_PARSER_SHA256", &ebnf_resolved);
        // The dual-run diff binary includes the seed from `src/bin/` (a different include base).
        println!(
            "cargo:rustc-env=PGEN_EBNF_PARSER_PATH_RESOLVED_BIN={}",
            relativize_for_include(&bin_source_dir, &ebnf_resolved).display()
        );
    }

    // The derived list: directory stems ∪ override names, sorted; each name is the artifact's
    // registered spelling (the generator writes `<name>_parser.rs` from it).
    let mut names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    if let Ok(entries) = std::fs::read_dir(&generated_dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if let Some(stem) = file_name.strip_suffix("_parser.rs") {
                if !stem.is_empty() {
                    names.insert(stem.to_string());
                }
            }
        }
    }
    for (key, _) in env::vars() {
        if let Some(middle) = key.strip_prefix("PGEN_").and_then(|k| k.strip_suffix("_PARSER_PATH")) {
            if middle != "EBNF" && !middle.is_empty() {
                names.insert(middle.to_ascii_lowercase());
            }
        }
    }
    // The two meta parsers every build needs: their absence is a build error with the remedy,
    // never a silent cfg-off (a clean checkout must run `make regenerate_generated_parsers`).
    for required in ["return_annotation", "semantic_annotation"] {
        if !names.contains(required) {
            panic!(
                "generated/{}_parser.rs is missing — run `make -C rust SHELL=/bin/bash regenerate_generated_parsers` first",
                required
            );
        }
    }
    // Family COMPONENT modules (`.112` slice 2 / `.113`): `src/families/<name>.rs` or
    // `src/families/<name>/mod.rs` present ⇒ that family's `HOOKS` (and, through them, its
    // preprocessing stage). The map is name → the module's root file.
    let mut hook_families: std::collections::BTreeMap<String, PathBuf> = std::collections::BTreeMap::new();
    if let Ok(entries) = std::fs::read_dir(&families_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                let root = path.join("mod.rs");
                if root.is_file() {
                    println!("cargo:rerun-if-changed={}", path.to_string_lossy());
                    hook_families.insert(file_name, root);
                }
            } else if let Some(stem) = file_name.strip_suffix(".rs") {
                if stem != "mod" {
                    hook_families.insert(stem.to_string(), path);
                }
            }
        }
    }
    for family in hook_families.keys() {
        println!("cargo:rustc-check-cfg=cfg(has_generated_{}_parser)", family);
    }

    let mut modules = String::new();
    let mut table = String::new();
    let mut hook_modules = String::new();
    modules.push_str("// DERIVED by build.rs from ../generated — never edited, never tracked.\n");
    table.push_str("// DERIVED by build.rs from ../generated — never edited, never tracked.\n");
    table.push_str("fn derived_rows() -> Vec<GeneratedParserRegistryEntry> {\n    #[allow(unused_mut)]\n    let mut rows: Vec<GeneratedParserRegistryEntry> = Vec::new();\n");
    hook_modules.push_str("// DERIVED by build.rs from src/families — never edited, never tracked.\n");
    for name in &names {
        let upper = name.to_ascii_uppercase();
        let env_key = format!("PGEN_{}_PARSER_PATH", upper);
        println!("cargo:rerun-if-env-changed={}", env_key);
        let configured = env::var(&env_key).unwrap_or_else(|_| format!("../generated/{}_parser.rs", name));
        let resolved = resolve_path(&manifest_dir, &configured);
        println!("cargo:rerun-if-changed={}", resolved.to_string_lossy());
        println!("cargo:rustc-check-cfg=cfg(has_generated_{}_parser)", name);
        if !resolved.is_file() {
            continue;
        }
        let cfg = format!("has_generated_{}_parser", name);
        println!("cargo:rustc-cfg={}", cfg);
        println!("cargo:rustc-env=PGEN_{}_PARSER_PATH_RESOLVED={}", upper, resolved.display());
        emit_parser_fingerprint(&format!("PGEN_{}_PARSER_SHA256", upper), &resolved);
        let type_name = format!("{}Parser", snake_to_pascal(name));
        // The module list (absolute include paths: the file lives in OUT_DIR).
        // The annotation pair keeps its backward-compat snake_case type alias inside its module
        // (`Return_annotationParser` / `Semantic_annotationParser`, used by path across the crate).
        let compat_alias = match name.as_str() {
            "return_annotation" => "    #[allow(non_camel_case_types)]\n    pub type Return_annotationParser<'input> = ReturnAnnotationParser<'input>;\n",
            "semantic_annotation" => "    #[allow(non_camel_case_types)]\n    pub type Semantic_annotationParser<'input> = SemanticAnnotationParser<'input>;\n",
            _ => "",
        };
        modules.push_str(&format!(
            "#[cfg({cfg})]\npub mod {name} {{\n    include!({path:?});\n{alias}}}\n",
            cfg = cfg, name = name, path = resolved.display().to_string(), alias = compat_alias
        ));
        // The registry row.
        let hooks = if hook_families.contains_key(name) {
            format!("&crate::families::{}::HOOKS", name)
        } else {
            "&crate::families::FamilyHooks::NONE".to_string()
        };
        table.push_str(&format!(
            "    #[cfg({cfg})]\n    {{\n        rows.push(generated_parser_row!({name}, crate::generated_parsers::{name}::{ty}, {hooks}, \"PGEN_{upper}_PARSER_SHA256\"));\n    }}\n",
            cfg = cfg, name = name, ty = type_name, hooks = hooks, upper = upper
        ));
    }
    // The ebnf seed's row — the meta-grammar's parser, under the `ebnf_dual_run` feature.
    if ebnf_resolved.is_file() {
        table.push_str("    #[cfg(all(feature = \"ebnf_dual_run\", has_generated_ebnf_parser))]\n    {\n        rows.push(generated_parser_row!(ebnf, crate::ebnf_generated_parser::EbnfParser, &crate::families::FamilyHooks::NONE, \"PGEN_EBNF_PARSER_SHA256\"));\n    }\n");
    }
    table.push_str("    rows\n}\n");
    // Unconditional: a component is a fact of the source tree, not of a generated artifact — the
    // operator CLI reaches a family's preprocessing stage whether or not its parser was generated.
    for (family, root) in &hook_families {
        hook_modules.push_str(&format!(
            "#[path = {p:?}]\npub mod {f};\n",
            f = family, p = root.display().to_string()
        ));
    }
    // The component table: every family module, keyed by its directory name.
    hook_modules.push_str("/// DERIVED by build.rs: every family component under `src/families`, by name.\npub fn components() -> &'static [(&'static str, &'static FamilyHooks)] {\n    &[\n");
    for family in hook_families.keys() {
        hook_modules.push_str(&format!("        ({f:?}, &{f}::HOOKS),\n", f = family));
    }
    hook_modules.push_str("    ]\n}\n");
    std::fs::write(out_dir.join("generated_parser_modules.rs"), modules).expect("write generated_parser_modules.rs");
    std::fs::write(out_dir.join("generated_parser_table.rs"), table).expect("write generated_parser_table.rs");
    std::fs::write(out_dir.join("family_hook_modules.rs"), hook_modules).expect("write family_hook_modules.rs");
}

/// The generator's own type-name rule (`snake_to_pascal(<name>) + "Parser"`), re-implemented
/// here for the IDENTIFIER only — the registered spelling itself is read from the artifact's
/// `GRAMMAR_NAME` at run time and asserted equal to the file stem by a registry test
/// (`ENGINE-UNIVERSAL-SERVICES.31` (e): never re-derive the spelling).
fn snake_to_pascal(name: &str) -> String {
    let mut out = String::new();
    for part in name.split('_') {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.push(first.to_ascii_uppercase());
            out.push_str(chars.as_str());
        }
    }
    out
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
