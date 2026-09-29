//! `ENGINE-UNIVERSAL-SERVICES.113` — the PREPROCESSING STAGE: what the engine owns of preprocessing.
//!
//! A preprocessor is a fact about one language (SystemVerilog's backtick directives, C's `#`
//! lines, VHDL-2019's tool directives); what every one of them shares is the STAGE — an ordered
//! compilation unit of files in, expanded text with a source map, an event log and diagnostics
//! out — and the context every front end knows: predefined macros, include search directories,
//! the unit's earlier files, and the dialect the expanded text will be parsed under. That shared
//! part is this module; nothing here knows any language's directives, option names or event
//! kinds. A family that has a preprocessor declares a [`PreprocessingStage`] on its
//! [`crate::families::FamilyHooks`] (its own options, its own legacy CLI spellings, its own
//! `run`), and the operator CLIs reach it ONLY through [`stage_for`], keyed by the family's name.
//!
//! The output types are the engine's, used by the family directly, so the JSON a family writes
//! (`--source-map-json`, `--event-log-json`, `--diagnostics-json`) is byte-identical to what it
//! wrote before the stage existed: the shapes below are exactly the SystemVerilog preprocessor's
//! pre-.113 serde shapes, with the event KIND opened from a per-language enum to its serde name.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// One option a family's stage declares — `--stage-opt KEY=VALUE` on the CLIs. The engine
/// validates nothing about it beyond the key's existence; the family reads and refuses values.
#[derive(Debug, Clone, Copy)]
pub struct StageOptionSpec {
    /// The option's key, as spelled in `--stage-opt KEY=VALUE`.
    pub key: &'static str,
    /// The values it takes, for the listing (`allow|warn|error`, `N`, `CODE[,CODE…]`, …).
    pub values: &'static str,
    /// The default the family applies when the option is absent (`""` when there is none).
    pub default: &'static str,
    /// One line for `--list-stage-options`.
    pub doc: &'static str,
}

/// Where a deprecated (pre-`.113`) CLI spelling maps to.
#[derive(Debug, Clone, Copy)]
pub enum LegacyTarget {
    /// The old flag was a universal one under a family prefix: `old VALUE` → `new VALUE`
    /// (or `old` → `new` when neither takes a value).
    Flag(&'static str),
    /// The old flag is a stage option: `old VALUE` → `--stage-opt key=VALUE`; with a fixed
    /// value, the old flag took none: `old` → `--stage-opt key=value` (appended LAST, so it wins
    /// over an earlier `--stage-opt key=…` exactly as the old override flag did).
    StageOpt { key: &'static str, fixed_value: Option<&'static str> },
}

/// A deprecated CLI spelling a family still accepts for one release, and its neutral form.
#[derive(Debug, Clone, Copy)]
pub struct LegacyFlag {
    /// The old spelling, e.g. `--sv-include-dir`.
    pub flag: &'static str,
    /// Whether the old spelling consumed the following argument as its value.
    pub takes_value: bool,
    pub to: LegacyTarget,
}

/// The universal context of one stage run (`.113`(d): the concepts every front end with a
/// preprocessor knows; a family without one of them ignores that field and says so in its
/// option listing).
#[derive(Debug, Clone, Default)]
pub struct StageRequest {
    /// The ordered compilation unit; the LAST file is the input whose expansion the caller wants,
    /// the earlier ones are context (their definitions carry, their text is discarded).
    pub unit: Vec<PathBuf>,
    /// Include search directories, in order.
    pub include_dirs: Vec<PathBuf>,
    /// Predefined macros, in order: `NAME` or `NAME=TEXT`.
    pub defines: Vec<String>,
    /// The profile spelling the EXPANDED text will be parsed under (the family decides what it
    /// means for its stage; `None` is the unprofiled reading).
    pub parse_profile: Option<String>,
    /// The family-declared options, `(key, value)` in command-line order.
    pub options: Vec<(String, String)>,
}

/// Where a run of output bytes came from.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column_start: usize,
    pub column_end: usize,
}

/// One chunk of the source map: output bytes `[output_start, output_end)` came from `source`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceMapEntry {
    pub output_start: usize,
    pub output_end: usize,
    pub source: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StageDiagnosticSeverity {
    Warning,
    Error,
}

/// A diagnostic the stage raised; `code` is the family's own vocabulary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StageDiagnostic {
    pub code: String,
    pub severity: StageDiagnosticSeverity,
    pub file: String,
    pub line: usize,
    pub message: String,
    pub detail: String,
}

/// One event of the stage's log; `kind` is the family's own vocabulary (its serde spelling).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StageEvent {
    pub kind: String,
    pub file: String,
    pub line: usize,
    pub active: bool,
    pub detail: String,
}

/// What one file of the unit expanded to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageOutput {
    pub text: String,
    pub source_map: Vec<SourceMapEntry>,
    pub events: Vec<StageEvent>,
    pub diagnostics: Vec<StageDiagnostic>,
    pub included_files: Vec<String>,
}

impl StageOutput {
    /// `.113`: map a byte offset of the EXPANDED text to the location it was written at — the
    /// chunk holding `position` names the file and line (an included line names the included
    /// file, a macro expansion the line of its USE); the column is the chunk's first column plus
    /// the CHARACTERS between the chunk's start and `position`. A position at or past the end of
    /// the text lands on the last chunk's end; `None` when the map is empty or the position
    /// precedes its first chunk.
    pub fn origin_of_position(&self, position: usize) -> Option<&SourceLocation> {
        let map = &self.source_map;
        if map.is_empty() {
            return None;
        }
        let index = match map.binary_search_by(|entry| entry.output_start.cmp(&position)) {
            Ok(index) => index,
            Err(0) => return None,
            Err(index) => index - 1,
        };
        Some(&map[index].source)
    }
}

/// A family's preprocessing stage, declared once on its hooks.
pub struct PreprocessingStage {
    /// The options this stage reads (`--stage-opt KEY=VALUE`).
    pub options: &'static [StageOptionSpec],
    /// The deprecated spellings this stage still accepts (rewritten by [`rewrite_legacy_args`]).
    pub legacy: &'static [LegacyFlag],
    /// The stage itself: one output per file of `request.unit`, in order; `Err` is a refusal that
    /// names its cause (the parser never runs on it).
    pub run: fn(&StageRequest) -> anyhow::Result<Vec<StageOutput>>,
}

impl PreprocessingStage {
    /// The declared option with this key, if any.
    pub fn option(&self, key: &str) -> Option<&StageOptionSpec> {
        self.options.iter().find(|spec| spec.key == key)
    }

    /// Refuse an option the stage does not declare, naming the ones it does.
    pub fn check_option_keys(&self, request: &StageRequest) -> anyhow::Result<()> {
        for (key, _) in &request.options {
            if self.option(key).is_none() {
                anyhow::bail!(
                    "unknown stage option '{}'; this family's stage declares: {}",
                    key,
                    self.options.iter().map(|spec| spec.key).collect::<Vec<_>>().join(", ")
                );
            }
        }
        Ok(())
    }

    /// Run the stage on `request` after checking its option keys.
    pub fn execute(&self, request: &StageRequest) -> anyhow::Result<Vec<StageOutput>> {
        self.check_option_keys(request)?;
        (self.run)(request)
    }

    /// The `--list-stage-options` text: one line per option, `key=VALUES (default: …) — doc`.
    pub fn describe_options(&self) -> String {
        let mut out = String::new();
        for spec in self.options {
            let default = if spec.default.is_empty() {
                String::new()
            } else {
                format!(" (default: {})", spec.default)
            };
            out.push_str(&format!("{}={}{} — {}\n", spec.key, spec.values, default, spec.doc));
        }
        out
    }
}

/// Split one `--stage-opt KEY=VALUE` argument.
pub fn parse_stage_opt(raw: &str) -> anyhow::Result<(String, String)> {
    match raw.split_once('=') {
        Some((key, value)) if !key.trim().is_empty() => Ok((key.trim().to_string(), value.to_string())),
        _ => anyhow::bail!("--stage-opt takes KEY=VALUE (got '{}')", raw),
    }
}

/// The stage a family declares, by the family's name — through the build-derived component
/// table, so it needs neither the registry nor the family's generated parser.
pub fn stage_for(family: &str) -> Option<&'static PreprocessingStage> {
    crate::families::component(family).and_then(|hooks| hooks.preprocessing)
}

/// Every family that declares a stage, sorted.
pub fn families_with_stage() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = crate::families::components()
        .iter()
        .filter(|(_, hooks)| hooks.preprocessing.is_some())
        .map(|(name, _)| *name)
        .collect();
    names.sort_unstable();
    names
}

/// The release the deprecated spellings are accepted through, for the deprecation line.
pub const LEGACY_FLAGS_ACCEPTED_THROUGH: &str = "1.0.247";

/// `.113`(b): rewrite a family's deprecated CLI spellings into the neutral form, in place, and
/// return one note per DISTINCT old spelling used (the caller prints them on stderr). `family`
/// is the stage the arguments address; with no stage, or no legacy spellings, the arguments
/// come back untouched. A `LegacyTarget::StageOpt` with a fixed value is appended LAST so it
/// wins over an earlier `--stage-opt key=…`, exactly as the old override flag did.
pub fn rewrite_legacy_args(args: &[String], family: &str) -> (Vec<String>, Vec<String>) {
    let Some(stage) = stage_for(family) else {
        return (args.to_vec(), Vec::new());
    };
    let mut out: Vec<String> = Vec::with_capacity(args.len());
    let mut tail: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let mut idx = 0;
    while idx < args.len() {
        let arg = &args[idx];
        let Some(legacy) = stage.legacy.iter().find(|l| l.flag == arg) else {
            out.push(arg.clone());
            idx += 1;
            continue;
        };
        let value = if legacy.takes_value { args.get(idx + 1).cloned() } else { None };
        let (replacement, spelled) = match legacy.to {
            LegacyTarget::Flag(new) => {
                let mut r = vec![new.to_string()];
                if let Some(v) = &value {
                    r.push(v.clone());
                }
                (r, format!("'{}' is '{}'", legacy.flag, new))
            }
            LegacyTarget::StageOpt { key, fixed_value: Some(fixed) } => {
                (vec!["--stage-opt".to_string(), format!("{}={}", key, fixed)], format!("'{}' is '--stage-opt {}={}'", legacy.flag, key, fixed))
            }
            LegacyTarget::StageOpt { key, fixed_value: None } => {
                let v = value.clone().unwrap_or_default();
                (vec!["--stage-opt".to_string(), format!("{}={}", key, v)], format!("'{}' is '--stage-opt {}=VALUE'", legacy.flag, key))
            }
        };
        let note = format!(
            "deprecated: {} since ENGINE-UNIVERSAL-SERVICES.113; the old spelling is accepted through release {} and removed after it",
            spelled, LEGACY_FLAGS_ACCEPTED_THROUGH
        );
        if !notes.contains(&note) {
            notes.push(note);
        }
        let fixed = matches!(legacy.to, LegacyTarget::StageOpt { fixed_value: Some(_), .. });
        if fixed {
            tail.extend(replacement);
        } else {
            out.extend(replacement);
        }
        idx += if legacy.takes_value && value.is_some() { 2 } else { 1 };
    }
    out.extend(tail);
    (out, notes)
}

/// `.113`(b): the family a command line addresses under `--preprocess`, and the same line with a
/// deprecated `--preprocess-<family>` rewritten to `--preprocess <family>`. Derived from the
/// families that declare a stage — the engine spells none of them.
pub fn resolve_preprocess_family(args: &[String]) -> (Vec<String>, Option<String>, Vec<String>) {
    let mut out: Vec<String> = Vec::with_capacity(args.len() + 1);
    let mut family: Option<String> = None;
    let mut notes: Vec<String> = Vec::new();
    let mut idx = 0;
    while idx < args.len() {
        let arg = &args[idx];
        if arg == "--preprocess" {
            family = args.get(idx + 1).cloned();
            out.push(arg.clone());
            idx += 1;
            continue;
        }
        if let Some(name) = arg.strip_prefix("--preprocess-") {
            if stage_for(name).is_some() {
                let note = format!(
                    "deprecated: '--preprocess-{0}' is '--preprocess {0}' since ENGINE-UNIVERSAL-SERVICES.113; the old spelling is accepted through release {1} and removed after it",
                    name, LEGACY_FLAGS_ACCEPTED_THROUGH
                );
                if !notes.contains(&note) {
                    notes.push(note);
                }
                family = Some(name.to_string());
                out.push("--preprocess".to_string());
                out.push(name.to_string());
                idx += 1;
                continue;
            }
        }
        out.push(arg.clone());
        idx += 1;
    }
    (out, family, notes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_opt_parsing_takes_key_equals_value() {
        assert_eq!(parse_stage_opt("a=b=c").unwrap(), ("a".to_string(), "b=c".to_string()));
        assert_eq!(parse_stage_opt("k=").unwrap(), ("k".to_string(), String::new()));
        assert!(parse_stage_opt("novalue").is_err());
        assert!(parse_stage_opt("=v").is_err());
    }

    #[test]
    fn origin_of_position_names_the_chunk_holding_the_byte() {
        let chunk = |output_start, output_end, file: &str, line| SourceMapEntry {
            output_start,
            output_end,
            source: SourceLocation { file: file.to_string(), line, column_start: 1, column_end: 10 },
        };
        let output = StageOutput {
            text: "0123456789abcdefghij\n".to_string(),
            source_map: vec![chunk(0, 10, "/x/a.sv", 1), chunk(10, 21, "/x/b.svh", 7)],
            events: vec![],
            diagnostics: vec![],
            included_files: vec![],
        };
        assert_eq!(output.origin_of_position(3).map(|s| s.line), Some(1));
        assert_eq!(output.origin_of_position(10).map(|s| s.file.as_str()), Some("/x/b.svh"));
        assert_eq!(output.origin_of_position(99).map(|s| s.line), Some(7));
        let empty = StageOutput { text: String::new(), source_map: vec![], events: vec![], diagnostics: vec![], included_files: vec![] };
        assert!(empty.origin_of_position(0).is_none());
    }

    #[test]
    fn a_family_without_a_stage_leaves_the_arguments_alone() {
        let args: Vec<String> = ["--x", "1"].iter().map(|s| s.to_string()).collect();
        let (out, notes) = rewrite_legacy_args(&args, "no_such_family");
        assert_eq!(out, args);
        assert!(notes.is_empty());
        let (out, family, notes) = resolve_preprocess_family(&args);
        assert_eq!(out, args);
        assert!(family.is_none() && notes.is_empty());
    }
}
