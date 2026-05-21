// SV-EXH-PROOF.3.3.4.a — MVP-0 parser-agnostic compilation-artifact library.
//
// Provides JSON-on-disk artifacts for cross-file fact sharing.  Each artifact
// is one entity-kind / name pair (e.g. package `el2_pkg`) whose contents are
// the exported facts emitted while parsing that entity.  Atomic write
// (temp + rename) so a partial write never corrupts the library.
//
// The format is intentionally human-readable JSON for MVP-0 — small files,
// debuggable, easy to diff.  Future increments may switch to a binary format
// behind the `format_version` field.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::ast_pipeline::{
    SemanticFactRecord, SemanticRuntimeValue, UnifiedSemanticProperty, UnifiedSemanticValue,
};

/// Current artifact format version.  Bump when the on-disk shape changes
/// incompatibly; readers must check this before deserialising.
pub const ARTIFACT_FORMAT_VERSION: u32 = 1;

/// Fact kinds that MVP-0 considers exportable across compilation artifacts.
/// Conservative — additional kinds added as future leaves demand them.
pub const MVP0_EXPORTABLE_FACT_KINDS: &[&str] = &["type_name"];

/// Error type for library I/O.  All variants carry a human-readable message
/// for clean propagation through the existing `ParseError::ContextualError`
/// channel.
#[derive(Debug)]
pub enum LibraryError {
    /// The requested artifact file does not exist under the configured
    /// library-in directory.  Message includes kind+name+expected path.
    NotFound(String),
    /// The artifact file exists but cannot be read.
    Io(String),
    /// The artifact file is malformed JSON or fails the schema check.
    Parse(String),
    /// The artifact's `format_version` is newer than this binary supports.
    UnsupportedFormat(String),
    /// Write failed (filesystem error, permission, etc.).
    Write(String),
}

impl std::fmt::Display for LibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LibraryError::NotFound(s)
            | LibraryError::Io(s)
            | LibraryError::Parse(s)
            | LibraryError::UnsupportedFormat(s)
            | LibraryError::Write(s) => f.write_str(s),
        }
    }
}

/// Construct the on-disk path for an artifact of a given kind+name under a
/// library root.  Path: `<lib_dir>/<kind>/<name>.facts.json`.
pub fn artifact_path(lib_dir: &Path, kind: &str, name: &str) -> PathBuf {
    lib_dir.join(kind).join(format!("{}.facts.json", name))
}

/// Write the supplied facts (filtered to MVP-0-exportable kinds) as a JSON
/// artifact under the library directory.  Atomic: writes to a temp file
/// in the same directory then renames to the final path.
pub fn write_artifact(
    lib_dir: &Path,
    kind: &str,
    name: &str,
    facts: &[SemanticFactRecord],
) -> Result<PathBuf, LibraryError> {
    let target_path = artifact_path(lib_dir, kind, name);
    let parent = target_path.parent().ok_or_else(|| {
        LibraryError::Write(format!(
            "artifact path has no parent directory: {}",
            target_path.display()
        ))
    })?;
    fs::create_dir_all(parent).map_err(|e| {
        LibraryError::Write(format!(
            "failed to create library subdirectory {}: {}",
            parent.display(),
            e
        ))
    })?;

    let exportable: Vec<&SemanticFactRecord> = facts
        .iter()
        .filter(|f| MVP0_EXPORTABLE_FACT_KINDS.iter().any(|k| f.kind.eq_ignore_ascii_case(k)))
        .collect();

    let json = build_artifact_json(kind, name, &exportable);
    let body = serde_json::to_string_pretty(&json).map_err(|e| {
        LibraryError::Write(format!("failed to serialise artifact JSON: {}", e))
    })?;

    // Atomic write: temp file in the same directory, then rename.
    let temp_path = parent.join(format!(".{}.facts.json.tmp", name));
    {
        let mut tmp = fs::File::create(&temp_path).map_err(|e| {
            LibraryError::Write(format!(
                "failed to open temp artifact {}: {}",
                temp_path.display(),
                e
            ))
        })?;
        tmp.write_all(body.as_bytes()).map_err(|e| {
            LibraryError::Write(format!(
                "failed to write temp artifact {}: {}",
                temp_path.display(),
                e
            ))
        })?;
        tmp.sync_all().ok();
    }
    fs::rename(&temp_path, &target_path).map_err(|e| {
        LibraryError::Write(format!(
            "failed to rename {} -> {}: {}",
            temp_path.display(),
            target_path.display(),
            e
        ))
    })?;

    Ok(target_path)
}

/// Read the artifact for `(kind, name)` under `lib_dir` and return its facts.
/// Returns `LibraryError::NotFound` if the artifact file does not exist.
pub fn read_artifact(
    lib_dir: &Path,
    kind: &str,
    name: &str,
) -> Result<Vec<SemanticFactRecord>, LibraryError> {
    let path = artifact_path(lib_dir, kind, name);
    if !path.exists() {
        return Err(LibraryError::NotFound(format!(
            "compilation artifact missing: kind={} name={} expected at {}",
            kind,
            name,
            path.display()
        )));
    }
    let body = fs::read_to_string(&path).map_err(|e| {
        LibraryError::Io(format!("failed to read artifact {}: {}", path.display(), e))
    })?;
    let v: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
        LibraryError::Parse(format!(
            "artifact {} is not valid JSON: {}",
            path.display(),
            e
        ))
    })?;
    parse_artifact_json(&v, kind, name)
}

fn build_artifact_json(
    kind: &str,
    name: &str,
    facts: &[&SemanticFactRecord],
) -> serde_json::Value {
    use serde_json::{json, Value};
    let facts_json: Vec<Value> = facts
        .iter()
        .map(|f| {
            let name_json = match &f.name {
                SemanticRuntimeValue::Identifier(s) => json!({"kind": "identifier", "text": s}),
                SemanticRuntimeValue::String(s) => json!({"kind": "string", "text": s}),
                SemanticRuntimeValue::Number(s) => json!({"kind": "number", "text": s}),
                SemanticRuntimeValue::Boolean(b) => json!({"kind": "boolean", "value": b}),
                SemanticRuntimeValue::Null => json!({"kind": "null"}),
                // RuleReference shouldn't normally appear in committed facts
                // (they're values to be resolved, not stored).  Encode as a
                // best-effort marker so artifacts round-trip without panic.
                SemanticRuntimeValue::RuleReference(s) => {
                    json!({"kind": "rule_reference", "text": s})
                }
            };
            let attrs: Vec<Value> = f
                .attributes
                .iter()
                .map(|p| json!({"key": p.key, "value": semantic_value_to_json(&p.value)}))
                .collect();
            json!({
                "kind": f.kind,
                "name": name_json,
                "attributes": attrs,
            })
        })
        .collect();
    json!({
        "format_version": ARTIFACT_FORMAT_VERSION,
        "kind": kind,
        "name": name,
        "facts": facts_json,
    })
}

fn semantic_value_to_json(v: &UnifiedSemanticValue) -> serde_json::Value {
    use serde_json::{json, Value};
    match v {
        UnifiedSemanticValue::Identifier(s) => json!({"kind": "identifier", "text": s}),
        UnifiedSemanticValue::String(s) => json!({"kind": "string", "text": s}),
        UnifiedSemanticValue::Number(s) => json!({"kind": "number", "text": s}),
        UnifiedSemanticValue::Boolean(b) => json!({"kind": "boolean", "value": b}),
        UnifiedSemanticValue::Null => json!({"kind": "null"}),
        UnifiedSemanticValue::Array(items) => {
            let arr: Vec<Value> = items.iter().map(semantic_value_to_json).collect();
            Value::Array(arr)
        }
        // Object attribute values aren't expected in v0 artifacts (we only
        // store primitive attribute values like declaration_family); encode
        // as a marker so round-trip stays lossless-ish.
        UnifiedSemanticValue::Object(_) => json!({"kind": "object_unsupported_in_artifact_v0"}),
        // RuleReference shouldn't have survived resolution by the time facts
        // are committed and exported; emit a debug marker if it did.
        UnifiedSemanticValue::RuleReference(s) => {
            json!({"kind": "rule_reference_unresolved", "text": s})
        }
    }
}

fn parse_artifact_json(
    v: &serde_json::Value,
    expected_kind: &str,
    expected_name: &str,
) -> Result<Vec<SemanticFactRecord>, LibraryError> {
    let obj = v.as_object().ok_or_else(|| {
        LibraryError::Parse(format!("artifact root is not an object: {}", v))
    })?;
    let format_version = obj
        .get("format_version")
        .and_then(|fv| fv.as_u64())
        .ok_or_else(|| {
            LibraryError::Parse("artifact missing or non-integer 'format_version'".into())
        })?;
    if format_version > ARTIFACT_FORMAT_VERSION as u64 {
        return Err(LibraryError::UnsupportedFormat(format!(
            "artifact format_version={} exceeds supported version {}",
            format_version, ARTIFACT_FORMAT_VERSION
        )));
    }
    let kind = obj
        .get("kind")
        .and_then(|k| k.as_str())
        .ok_or_else(|| LibraryError::Parse("artifact missing 'kind' string".into()))?;
    let name = obj
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| LibraryError::Parse("artifact missing 'name' string".into()))?;
    if !kind.eq_ignore_ascii_case(expected_kind) {
        return Err(LibraryError::Parse(format!(
            "artifact kind mismatch: expected {} got {}",
            expected_kind, kind
        )));
    }
    if name != expected_name {
        return Err(LibraryError::Parse(format!(
            "artifact name mismatch: expected {} got {}",
            expected_name, name
        )));
    }
    let facts_arr = obj
        .get("facts")
        .and_then(|f| f.as_array())
        .ok_or_else(|| LibraryError::Parse("artifact missing 'facts' array".into()))?;
    let mut out = Vec::with_capacity(facts_arr.len());
    for (i, f) in facts_arr.iter().enumerate() {
        let fact_obj = f.as_object().ok_or_else(|| {
            LibraryError::Parse(format!("artifact facts[{}] is not an object", i))
        })?;
        let fkind = fact_obj
            .get("kind")
            .and_then(|k| k.as_str())
            .ok_or_else(|| {
                LibraryError::Parse(format!("artifact facts[{}] missing 'kind'", i))
            })?
            .to_string();
        let name_value = parse_name_value(
            fact_obj.get("name").ok_or_else(|| {
                LibraryError::Parse(format!("artifact facts[{}] missing 'name'", i))
            })?,
        )
        .map_err(|e| LibraryError::Parse(format!("artifact facts[{}] bad name: {}", i, e)))?;
        let attrs: Vec<UnifiedSemanticProperty> = fact_obj
            .get("attributes")
            .and_then(|a| a.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|a| {
                        let o = a.as_object()?;
                        let key = o.get("key")?.as_str()?.to_string();
                        let value_json = o.get("value")?;
                        // For MVP-0 we only persist primitive attribute values;
                        // reconstruct the matching UnifiedSemanticValue.
                        let value = parse_attribute_value(value_json)?;
                        Some(UnifiedSemanticProperty { key, value })
                    })
                    .collect()
            })
            .unwrap_or_default();
        out.push(SemanticFactRecord {
            kind: fkind,
            name: name_value,
            // Library-loaded facts land in the importer's current scope; the
            // importer's `has_fact` is a global search so scope_depth is
            // recorded as the current depth at load time (set by the caller).
            // Until then, default to 0 — overwritten on merge.
            scope_depth: 0,
            // `.3.3.4.b.5.1.3`: importing facts always rebase scope_id to
            // the importer's current scope when the import lands; default
            // to ROOT as a safe sentinel pending the rebase in `push_fact_record`.
            scope_id: crate::ast_pipeline::ScopeId::ROOT,
            attributes: attrs,
        });
    }
    Ok(out)
}

fn parse_name_value(v: &serde_json::Value) -> Result<SemanticRuntimeValue, String> {
    let o = v
        .as_object()
        .ok_or_else(|| "name is not an object".to_string())?;
    let k = o
        .get("kind")
        .and_then(|x| x.as_str())
        .ok_or_else(|| "name.kind missing".to_string())?;
    match k {
        "identifier" => Ok(SemanticRuntimeValue::Identifier(
            o.get("text")
                .and_then(|x| x.as_str())
                .ok_or_else(|| "identifier.text missing".to_string())?
                .to_string(),
        )),
        "string" => Ok(SemanticRuntimeValue::String(
            o.get("text")
                .and_then(|x| x.as_str())
                .ok_or_else(|| "string.text missing".to_string())?
                .to_string(),
        )),
        "number" => Ok(SemanticRuntimeValue::Number(
            o.get("text")
                .and_then(|x| x.as_str())
                .ok_or_else(|| "number.text missing".to_string())?
                .to_string(),
        )),
        "boolean" => Ok(SemanticRuntimeValue::Boolean(
            o.get("value")
                .and_then(|x| x.as_bool())
                .ok_or_else(|| "boolean.value missing".to_string())?,
        )),
        "null" => Ok(SemanticRuntimeValue::Null),
        "rule_reference" => Ok(SemanticRuntimeValue::RuleReference(
            o.get("text")
                .and_then(|x| x.as_str())
                .ok_or_else(|| "rule_reference.text missing".to_string())?
                .to_string(),
        )),
        other => Err(format!("unknown name kind '{}'", other)),
    }
}

fn parse_attribute_value(v: &serde_json::Value) -> Option<UnifiedSemanticValue> {
    let o = v.as_object()?;
    let kind = o.get("kind")?.as_str()?;
    match kind {
        "identifier" => Some(UnifiedSemanticValue::Identifier(
            o.get("text")?.as_str()?.to_string(),
        )),
        "string" => Some(UnifiedSemanticValue::String(
            o.get("text")?.as_str()?.to_string(),
        )),
        "number" => Some(UnifiedSemanticValue::Number(
            o.get("text")?.as_str()?.to_string(),
        )),
        "boolean" => Some(UnifiedSemanticValue::Boolean(o.get("value")?.as_bool()?)),
        "null" => Some(UnifiedSemanticValue::Null),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast_pipeline::{
        SemanticFactRecord, SemanticRuntimeValue, UnifiedSemanticProperty, UnifiedSemanticValue,
    };

    fn sample_fact(name: &str, family: &str) -> SemanticFactRecord {
        SemanticFactRecord {
            kind: "type_name".to_string(),
            name: SemanticRuntimeValue::Identifier(name.to_string()),
            scope_depth: 0,
            scope_id: crate::ast_pipeline::ScopeId::ROOT,
            attributes: vec![UnifiedSemanticProperty {
                key: "declaration_family".to_string(),
                value: UnifiedSemanticValue::Identifier(family.to_string()),
            }],
        }
    }

    #[test]
    fn artifact_roundtrip_preserves_type_name_facts() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let facts = vec![
            sample_fact("el2_trigger_pkt_t", "typedef"),
            sample_fact("el2_lsu_pkt_t", "typedef"),
        ];
        write_artifact(dir.path(), "package", "el2_pkg", &facts).expect("write");
        let read = read_artifact(dir.path(), "package", "el2_pkg").expect("read");
        assert_eq!(read.len(), 2);
        assert_eq!(read[0].kind, "type_name");
        assert_eq!(
            read[0].name,
            SemanticRuntimeValue::Identifier("el2_trigger_pkt_t".to_string())
        );
        assert_eq!(read[0].attributes.len(), 1);
    }

    #[test]
    fn non_exportable_kinds_are_filtered_on_write() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let facts = vec![
            sample_fact("el2_trigger_pkt_t", "typedef"),
            SemanticFactRecord {
                kind: "package_name".to_string(),
                name: SemanticRuntimeValue::Identifier("el2_pkg".to_string()),
                scope_depth: 0,
                scope_id: crate::ast_pipeline::ScopeId::ROOT,
                attributes: vec![],
            },
        ];
        write_artifact(dir.path(), "package", "el2_pkg", &facts).expect("write");
        let read = read_artifact(dir.path(), "package", "el2_pkg").expect("read");
        assert_eq!(read.len(), 1, "only type_name should survive v0 filter");
        assert_eq!(read[0].kind, "type_name");
    }

    #[test]
    fn missing_artifact_returns_not_found() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let err = read_artifact(dir.path(), "package", "does_not_exist").expect_err("expect err");
        assert!(matches!(err, LibraryError::NotFound(_)));
    }

    #[test]
    fn artifact_kind_or_name_mismatch_is_parse_error() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let facts = vec![sample_fact("x", "typedef")];
        write_artifact(dir.path(), "package", "p1", &facts).expect("write");
        // Place the file under p1.facts.json but try to read as p2 (different
        // path → NotFound), so use the same name but different kind path —
        // craft a synthetic mismatch by reading then asserting check fires:
        let p = artifact_path(dir.path(), "package", "p1");
        let body = std::fs::read_to_string(&p).unwrap();
        std::fs::write(artifact_path(dir.path(), "package", "p2"), &body).unwrap();
        let err = read_artifact(dir.path(), "package", "p2").expect_err("expect err");
        assert!(matches!(err, LibraryError::Parse(_)));
    }
}
