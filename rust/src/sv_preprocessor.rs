//! SystemVerilog preprocessor execution stage (baseline).
//!
//! This module provides a deterministic, file-based preprocessing pass:
//! - macro define/undef and expansion (object-like + function-like),
//! - include resolution with bounded depth and cycle detection,
//! - conditional-compilation flow (`ifdef/`ifndef/`elsif/`else/`endif),
//! - source mapping metadata from expanded output back to originating file/line.

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SvPreprocessorConfig {
    pub include_dirs: Vec<PathBuf>,
    pub max_include_depth: usize,
    pub allow_macro_redefine: bool,
}

impl Default for SvPreprocessorConfig {
    fn default() -> Self {
        Self {
            include_dirs: Vec::new(),
            max_include_depth: 64,
            allow_macro_redefine: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
    pub column_start: usize,
    pub column_end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceMapEntry {
    pub output_start: usize,
    pub output_end: usize,
    pub source: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PreprocessorEventKind {
    Define,
    Undef,
    Include,
    Ifdef,
    Ifndef,
    Elsif,
    Else,
    Endif,
    PassthroughLine,
    SkippedLine,
    MacroExpand,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreprocessorEvent {
    pub kind: PreprocessorEventKind,
    pub file: String,
    pub line: usize,
    pub active: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessedOutput {
    pub text: String,
    pub source_map: Vec<SourceMapEntry>,
    pub events: Vec<PreprocessorEvent>,
    pub included_files: Vec<String>,
}

#[derive(Debug, Clone)]
struct MacroDefinition {
    params: Option<Vec<String>>,
    body: String,
}

#[derive(Debug, Clone)]
struct ConditionalFrame {
    parent_active: bool,
    current_active: bool,
    branch_taken: bool,
}

#[derive(Debug)]
struct PreprocessorState {
    config: SvPreprocessorConfig,
    macros: HashMap<String, MacroDefinition>,
    output: String,
    source_map: Vec<SourceMapEntry>,
    events: Vec<PreprocessorEvent>,
    include_stack: Vec<PathBuf>,
    included_files: Vec<String>,
    included_seen: HashSet<String>,
}

impl PreprocessorState {
    fn new(config: SvPreprocessorConfig) -> Self {
        Self {
            config,
            macros: HashMap::new(),
            output: String::new(),
            source_map: Vec::new(),
            events: Vec::new(),
            include_stack: Vec::new(),
            included_files: Vec::new(),
            included_seen: HashSet::new(),
        }
    }

    fn push_event(
        &mut self,
        kind: PreprocessorEventKind,
        file: &Path,
        line: usize,
        active: bool,
        detail: impl Into<String>,
    ) {
        self.events.push(PreprocessorEvent {
            kind,
            file: file.display().to_string(),
            line,
            active,
            detail: detail.into(),
        });
    }

    fn append_output_chunk(
        &mut self,
        chunk: &str,
        file: &Path,
        line: usize,
        column_start: usize,
        column_end: usize,
    ) {
        if chunk.is_empty() {
            return;
        }
        let start = self.output.len();
        self.output.push_str(chunk);
        let end = self.output.len();
        self.source_map.push(SourceMapEntry {
            output_start: start,
            output_end: end,
            source: SourceLocation {
                file: file.display().to_string(),
                line,
                column_start,
                column_end,
            },
        });
    }

    fn record_include_file(&mut self, path: &Path) {
        let key = path.display().to_string();
        if self.included_seen.insert(key.clone()) {
            self.included_files.push(key);
        }
    }
}

pub fn preprocess_systemverilog_file(
    input_path: &Path,
    config: &SvPreprocessorConfig,
) -> Result<PreprocessedOutput> {
    let mut state = PreprocessorState::new(config.clone());
    preprocess_file_internal(input_path, &mut state, 0)?;
    Ok(PreprocessedOutput {
        text: state.output,
        source_map: state.source_map,
        events: state.events,
        included_files: state.included_files,
    })
}

fn preprocess_file_internal(
    path: &Path,
    state: &mut PreprocessorState,
    depth: usize,
) -> Result<()> {
    if depth > state.config.max_include_depth {
        bail!(
            "include depth {} exceeded max {} while entering '{}'",
            depth,
            state.config.max_include_depth,
            path.display()
        );
    }

    let canonical_path = path
        .canonicalize()
        .with_context(|| format!("failed to canonicalize '{}'", path.display()))?;

    if state.include_stack.contains(&canonical_path) {
        let cycle = state
            .include_stack
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(" -> ");
        bail!(
            "include cycle detected: {} -> {}",
            cycle,
            canonical_path.display()
        );
    }

    let content = fs::read_to_string(&canonical_path)
        .with_context(|| format!("failed to read '{}'", canonical_path.display()))?;
    state.include_stack.push(canonical_path.clone());
    state.record_include_file(&canonical_path);
    preprocess_text_internal(&content, &canonical_path, state, depth)?;
    state.include_stack.pop();
    Ok(())
}

fn preprocess_text_internal(
    content: &str,
    file_path: &Path,
    state: &mut PreprocessorState,
    depth: usize,
) -> Result<()> {
    let mut conditionals: Vec<ConditionalFrame> = Vec::new();

    for (line_index, raw_line) in split_lines_preserve_terminator(content)
        .into_iter()
        .enumerate()
    {
        let line_no = line_index + 1;
        let active = conditionals
            .last()
            .map(|c| c.current_active)
            .unwrap_or(true);
        let trimmed = raw_line.trim_start_matches([' ', '\t']);

        if let Some((directive, rest)) = parse_directive_line(trimmed) {
            match directive {
                "define" => {
                    state.push_event(
                        PreprocessorEventKind::Define,
                        file_path,
                        line_no,
                        active,
                        rest.to_string(),
                    );
                    if active {
                        let (name, definition) =
                            parse_define_directive(rest).with_context(|| {
                                format!(
                                    "invalid `define directive at {}:{}",
                                    file_path.display(),
                                    line_no
                                )
                            })?;
                        if !state.config.allow_macro_redefine && state.macros.contains_key(&name) {
                            bail!(
                                "macro '{}' redefined at {}:{} while allow_macro_redefine=false",
                                name,
                                file_path.display(),
                                line_no
                            );
                        }
                        state.macros.insert(name, definition);
                    }
                }
                "undef" => {
                    state.push_event(
                        PreprocessorEventKind::Undef,
                        file_path,
                        line_no,
                        active,
                        rest.to_string(),
                    );
                    if active {
                        if let Some((name, _)) = parse_identifier_token(rest.trim_start()) {
                            state.macros.remove(name);
                        }
                    }
                }
                "include" => {
                    state.push_event(
                        PreprocessorEventKind::Include,
                        file_path,
                        line_no,
                        active,
                        rest.to_string(),
                    );
                    if active {
                        let (include_token, is_angle) = parse_include_directive(rest)
                            .with_context(|| {
                                format!(
                                    "invalid `include directive at {}:{}",
                                    file_path.display(),
                                    line_no
                                )
                            })?;
                        let include_path =
                            resolve_include_path(include_token, is_angle, file_path, state)
                                .with_context(|| {
                                    format!(
                                        "failed to resolve include '{}' at {}:{}",
                                        include_token,
                                        file_path.display(),
                                        line_no
                                    )
                                })?;
                        preprocess_file_internal(&include_path, state, depth + 1)?;
                    }
                }
                "ifdef" => {
                    let name = rest.trim();
                    let cond = state.macros.contains_key(name);
                    let parent_active = active;
                    let current_active = parent_active && cond;
                    conditionals.push(ConditionalFrame {
                        parent_active,
                        current_active,
                        branch_taken: cond,
                    });
                    state.push_event(
                        PreprocessorEventKind::Ifdef,
                        file_path,
                        line_no,
                        current_active,
                        name.to_string(),
                    );
                }
                "ifndef" => {
                    let name = rest.trim();
                    let cond = !state.macros.contains_key(name);
                    let parent_active = active;
                    let current_active = parent_active && cond;
                    conditionals.push(ConditionalFrame {
                        parent_active,
                        current_active,
                        branch_taken: cond,
                    });
                    state.push_event(
                        PreprocessorEventKind::Ifndef,
                        file_path,
                        line_no,
                        current_active,
                        name.to_string(),
                    );
                }
                "elsif" => {
                    let cond_name = rest.trim();
                    let frame = conditionals.last_mut().with_context(|| {
                        format!(
                            "`elsif without matching `ifdef/`ifndef at {}:{}",
                            file_path.display(),
                            line_no
                        )
                    })?;
                    if frame.branch_taken {
                        frame.current_active = false;
                    } else {
                        let cond = state.macros.contains_key(cond_name);
                        frame.current_active = frame.parent_active && cond;
                        if cond {
                            frame.branch_taken = true;
                        }
                    }
                    state.push_event(
                        PreprocessorEventKind::Elsif,
                        file_path,
                        line_no,
                        frame.current_active,
                        cond_name.to_string(),
                    );
                }
                "else" => {
                    let frame = conditionals.last_mut().with_context(|| {
                        format!(
                            "`else without matching `ifdef/`ifndef at {}:{}",
                            file_path.display(),
                            line_no
                        )
                    })?;
                    if frame.branch_taken {
                        frame.current_active = false;
                    } else {
                        frame.current_active = frame.parent_active;
                        frame.branch_taken = true;
                    }
                    state.push_event(
                        PreprocessorEventKind::Else,
                        file_path,
                        line_no,
                        frame.current_active,
                        String::new(),
                    );
                }
                "endif" => {
                    conditionals.pop().with_context(|| {
                        format!(
                            "`endif without matching `ifdef/`ifndef at {}:{}",
                            file_path.display(),
                            line_no
                        )
                    })?;
                    state.push_event(
                        PreprocessorEventKind::Endif,
                        file_path,
                        line_no,
                        active,
                        String::new(),
                    );
                }
                // Keep common non-control directives in output if active.
                "timescale" | "default_nettype" | "celldefine" | "endcelldefine" => {
                    if active {
                        let expanded = expand_macros_in_text(
                            &raw_line,
                            &state.macros,
                            0,
                            &mut state.events,
                            file_path,
                            line_no,
                        );
                        state.push_event(
                            PreprocessorEventKind::PassthroughLine,
                            file_path,
                            line_no,
                            true,
                            directive.to_string(),
                        );
                        state.append_output_chunk(
                            &expanded,
                            file_path,
                            line_no,
                            1,
                            raw_line.chars().count().max(1),
                        );
                    } else {
                        state.push_event(
                            PreprocessorEventKind::SkippedLine,
                            file_path,
                            line_no,
                            false,
                            format!("inactive directive `{}`", directive),
                        );
                    }
                }
                _ => {
                    // Unknown directive: pass through only when active to preserve tool behavior.
                    if active {
                        let expanded = expand_macros_in_text(
                            &raw_line,
                            &state.macros,
                            0,
                            &mut state.events,
                            file_path,
                            line_no,
                        );
                        state.push_event(
                            PreprocessorEventKind::PassthroughLine,
                            file_path,
                            line_no,
                            true,
                            format!("unknown directive `{}`", directive),
                        );
                        state.append_output_chunk(
                            &expanded,
                            file_path,
                            line_no,
                            1,
                            raw_line.chars().count().max(1),
                        );
                    } else {
                        state.push_event(
                            PreprocessorEventKind::SkippedLine,
                            file_path,
                            line_no,
                            false,
                            format!("inactive unknown directive `{}`", directive),
                        );
                    }
                }
            }
            continue;
        }

        if active {
            let expanded = expand_macros_in_text(
                &raw_line,
                &state.macros,
                0,
                &mut state.events,
                file_path,
                line_no,
            );
            state.push_event(
                PreprocessorEventKind::PassthroughLine,
                file_path,
                line_no,
                true,
                String::new(),
            );
            state.append_output_chunk(
                &expanded,
                file_path,
                line_no,
                1,
                raw_line.chars().count().max(1),
            );
        } else {
            state.push_event(
                PreprocessorEventKind::SkippedLine,
                file_path,
                line_no,
                false,
                "inactive branch".to_string(),
            );
        }
    }

    if !conditionals.is_empty() {
        bail!(
            "unterminated conditional block in '{}'",
            file_path.display()
        );
    }
    Ok(())
}

fn parse_directive_line(input: &str) -> Option<(&str, &str)> {
    if !input.starts_with('`') {
        return None;
    }
    let mut end = 1;
    for ch in input[1..].chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            end += ch.len_utf8();
        } else {
            break;
        }
    }
    if end <= 1 {
        return None;
    }
    let name = &input[1..end];
    let rest = input[end..].trim_start();
    Some((name, rest))
}

fn parse_identifier_token(input: &str) -> Option<(&str, &str)> {
    let mut chars = input.char_indices();
    let (_, first) = chars.next()?;
    if !(first.is_ascii_alphabetic() || first == '_') {
        return None;
    }
    let mut end = first.len_utf8();
    for (idx, ch) in chars {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '$' {
            end = idx + ch.len_utf8();
        } else {
            break;
        }
    }
    Some((&input[..end], &input[end..]))
}

fn parse_define_directive(input: &str) -> Result<(String, MacroDefinition)> {
    let trimmed = input.trim_end_matches(['\r', '\n']);
    let (name, rest) = parse_identifier_token(trimmed.trim_start())
        .with_context(|| "missing macro name in `define")?;
    let rest = rest.trim_start();
    if rest.starts_with('(') {
        let (params, after) =
            parse_macro_parameter_list(rest).with_context(|| "invalid macro parameter list")?;
        let body = after.trim_start().to_string();
        Ok((
            name.to_string(),
            MacroDefinition {
                params: Some(params),
                body,
            },
        ))
    } else {
        Ok((
            name.to_string(),
            MacroDefinition {
                params: None,
                body: rest.to_string(),
            },
        ))
    }
}

fn parse_macro_parameter_list(input: &str) -> Result<(Vec<String>, &str)> {
    if !input.starts_with('(') {
        bail!("macro parameter list must start with '('");
    }
    let mut depth = 0usize;
    let mut end_idx = None;
    for (idx, ch) in input.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                if depth == 0 {
                    bail!("unexpected ')' while parsing macro parameter list");
                }
                depth -= 1;
                if depth == 0 {
                    end_idx = Some(idx);
                    break;
                }
            }
            _ => {}
        }
    }
    let close_idx = end_idx.with_context(|| "unterminated macro parameter list")?;
    let param_segment = &input[1..close_idx];
    let mut params = Vec::new();
    for token in param_segment.split(',') {
        let t = token.trim();
        if t.is_empty() {
            continue;
        }
        let (name, trailing) = parse_identifier_token(t)
            .with_context(|| format!("invalid macro parameter token '{}'", t))?;
        if !trailing.trim().is_empty() {
            bail!("invalid trailing text in macro parameter '{}'", t);
        }
        params.push(name.to_string());
    }
    Ok((params, &input[close_idx + 1..]))
}

fn parse_include_directive(input: &str) -> Result<(&str, bool)> {
    let trimmed = input.trim();
    if let Some(rest) = trimmed.strip_prefix('"') {
        let quote_end = rest
            .find('"')
            .with_context(|| "unterminated quoted include path")?;
        let path = &rest[..quote_end];
        if path.is_empty() {
            bail!("empty quoted include path");
        }
        Ok((path, false))
    } else if let Some(rest) = trimmed.strip_prefix('<') {
        let angle_end = rest
            .find('>')
            .with_context(|| "unterminated angle include path")?;
        let path = &rest[..angle_end];
        if path.is_empty() {
            bail!("empty angle include path");
        }
        Ok((path, true))
    } else {
        bail!("include path must be quoted or angle-bracketed");
    }
}

fn resolve_include_path(
    include_token: &str,
    is_angle: bool,
    current_file: &Path,
    state: &PreprocessorState,
) -> Result<PathBuf> {
    let include_path = PathBuf::from(include_token);
    let mut candidates: Vec<PathBuf> = Vec::new();

    let current_parent = current_file
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    if include_path.is_absolute() {
        candidates.push(include_path);
    } else if is_angle {
        for dir in &state.config.include_dirs {
            candidates.push(dir.join(&include_path));
        }
        candidates.push(current_parent.join(&include_path));
    } else {
        candidates.push(current_parent.join(&include_path));
        for dir in &state.config.include_dirs {
            candidates.push(dir.join(&include_path));
        }
    }

    for candidate in candidates {
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    bail!("include '{}' was not found in search paths", include_token)
}

fn split_lines_preserve_terminator(content: &str) -> Vec<String> {
    if content.is_empty() {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let mut start = 0usize;
    for (idx, ch) in content.char_indices() {
        if ch == '\n' {
            lines.push(content[start..idx + 1].to_string());
            start = idx + 1;
        }
    }
    if start < content.len() {
        lines.push(content[start..].to_string());
    }
    lines
}

fn expand_macros_in_text(
    text: &str,
    macros: &HashMap<String, MacroDefinition>,
    depth: usize,
    events: &mut Vec<PreprocessorEvent>,
    file: &Path,
    line: usize,
) -> String {
    if depth > 16 {
        return text.to_string();
    }

    let bytes = text.as_bytes();
    let mut i = 0usize;
    let mut out = String::new();
    let mut expanded_any = false;

    while i < bytes.len() {
        if bytes[i] != b'`' {
            out.push(bytes[i] as char);
            i += 1;
            continue;
        }

        if i + 1 < bytes.len() && bytes[i + 1] == b'`' {
            out.push_str("``");
            i += 2;
            continue;
        }

        if i + 1 < bytes.len() && bytes[i + 1] == b'"' {
            out.push_str("`\"");
            i += 2;
            continue;
        }

        let ident_start = i + 1;
        if ident_start >= bytes.len() || !is_ident_start(bytes[ident_start]) {
            out.push('`');
            i += 1;
            continue;
        }
        let mut ident_end = ident_start + 1;
        while ident_end < bytes.len() && is_ident_continue(bytes[ident_end]) {
            ident_end += 1;
        }
        let name = &text[ident_start..ident_end];
        let Some(def) = macros.get(name) else {
            out.push('`');
            out.push_str(name);
            i = ident_end;
            continue;
        };

        if let Some(params) = &def.params {
            let mut j = ident_end;
            while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
                j += 1;
            }
            let Some((args, next_index)) = parse_macro_invocation_args(text, j) else {
                out.push('`');
                out.push_str(name);
                i = ident_end;
                continue;
            };
            let expanded = expand_function_macro(def, params, &args, macros, depth + 1);
            events.push(PreprocessorEvent {
                kind: PreprocessorEventKind::MacroExpand,
                file: file.display().to_string(),
                line,
                active: true,
                detail: format!("`{}({})", name, args.join(", ")),
            });
            out.push_str(&expanded);
            i = next_index;
            expanded_any = true;
        } else {
            let expanded = expand_object_macro(def, macros, depth + 1);
            events.push(PreprocessorEvent {
                kind: PreprocessorEventKind::MacroExpand,
                file: file.display().to_string(),
                line,
                active: true,
                detail: format!("`{}", name),
            });
            out.push_str(&expanded);
            i = ident_end;
            expanded_any = true;
        }
    }

    if expanded_any && out != text {
        expand_macros_in_text(&out, macros, depth + 1, events, file, line)
    } else {
        out
    }
}

fn parse_macro_invocation_args(input: &str, open_paren_idx: usize) -> Option<(Vec<String>, usize)> {
    let bytes = input.as_bytes();
    if open_paren_idx >= bytes.len() || bytes[open_paren_idx] != b'(' {
        return None;
    }
    let mut depth = 0usize;
    let mut i = open_paren_idx;
    let mut start = open_paren_idx + 1;
    let mut args = Vec::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;

    while i < bytes.len() {
        let ch = bytes[i] as char;
        if escaped {
            escaped = false;
            i += 1;
            continue;
        }
        if ch == '\\' && (in_single_quote || in_double_quote) {
            escaped = true;
            i += 1;
            continue;
        }
        if ch == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
            i += 1;
            continue;
        }
        if ch == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
            i += 1;
            continue;
        }
        if in_single_quote || in_double_quote {
            i += 1;
            continue;
        }
        match ch {
            '(' => {
                depth += 1;
                i += 1;
            }
            ')' => {
                if depth == 0 {
                    return None;
                }
                depth -= 1;
                if depth == 0 {
                    let segment = input[start..i].trim();
                    if !segment.is_empty() || !args.is_empty() {
                        args.push(segment.to_string());
                    }
                    return Some((args, i + 1));
                }
                i += 1;
            }
            ',' if depth == 1 => {
                let segment = input[start..i].trim();
                args.push(segment.to_string());
                i += 1;
                start = i;
            }
            _ => i += 1,
        }
    }
    None
}

fn expand_object_macro(
    definition: &MacroDefinition,
    macros: &HashMap<String, MacroDefinition>,
    depth: usize,
) -> String {
    expand_macros_in_text(
        &definition.body,
        macros,
        depth,
        &mut Vec::new(),
        Path::new("<macro>"),
        0,
    )
}

fn expand_function_macro(
    definition: &MacroDefinition,
    params: &[String],
    args: &[String],
    macros: &HashMap<String, MacroDefinition>,
    depth: usize,
) -> String {
    let mut bindings: HashMap<&str, &str> = HashMap::new();
    for (idx, param) in params.iter().enumerate() {
        let arg = args.get(idx).map(|s| s.as_str()).unwrap_or("");
        bindings.insert(param.as_str(), arg);
    }
    let substituted = substitute_function_macro_body(&definition.body, &bindings);
    let pasted = substituted.replace("``", "");
    expand_macros_in_text(
        &pasted,
        macros,
        depth,
        &mut Vec::new(),
        Path::new("<macro>"),
        0,
    )
}

fn substitute_function_macro_body(body: &str, bindings: &HashMap<&str, &str>) -> String {
    let bytes = body.as_bytes();
    let mut i = 0usize;
    let mut out = String::new();

    while i < bytes.len() {
        if i + 2 <= bytes.len() && bytes[i] == b'`' && bytes[i + 1] == b'"' {
            let mut j = i + 2;
            if j < bytes.len() && is_ident_start(bytes[j]) {
                j += 1;
                while j < bytes.len() && is_ident_continue(bytes[j]) {
                    j += 1;
                }
                let token = &body[i + 2..j];
                if let Some(arg) = bindings.get(token) {
                    out.push('"');
                    out.push_str(arg.trim());
                    out.push('"');
                    i = j;
                    continue;
                }
            }
            out.push_str("`\"");
            i += 2;
            continue;
        }

        if !is_ident_start(bytes[i]) {
            out.push(bytes[i] as char);
            i += 1;
            continue;
        }
        let start = i;
        i += 1;
        while i < bytes.len() && is_ident_continue(bytes[i]) {
            i += 1;
        }
        let token = &body[start..i];
        if let Some(arg) = bindings.get(token) {
            out.push_str(arg.trim());
        } else {
            out.push_str(token);
        }
    }

    out
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_ident_continue(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

#[cfg(test)]
mod tests {
    use super::{SvPreprocessorConfig, preprocess_systemverilog_file};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn create_temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{}_{}", prefix, nanos));
        fs::create_dir_all(&dir).expect("mkdir temp");
        dir
    }

    #[test]
    fn expands_object_macro_define() {
        let dir = create_temp_dir("svpp_define");
        let input = dir.join("top.sv");
        fs::write(&input, "`define WIDTH 16\nlogic [`WIDTH-1:0] data;\n").expect("write input");

        let output = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("preprocess");
        assert!(output.text.contains("logic [16-1:0] data;"));
        assert!(!output.text.contains("`define"));
    }

    #[test]
    fn resolves_include_and_tracks_source_map() {
        let dir = create_temp_dir("svpp_include");
        let inc = dir.join("inc.svh");
        let top = dir.join("top.sv");
        fs::write(&inc, "logic from_inc;\n").expect("write inc");
        fs::write(&top, "`include \"inc.svh\"\nlogic from_top;\n").expect("write top");

        let output = preprocess_systemverilog_file(&top, &SvPreprocessorConfig::default())
            .expect("preprocess include");
        assert!(output.text.contains("logic from_inc;"));
        assert!(output.text.contains("logic from_top;"));
        assert!(output.included_files.iter().any(|p| p.ends_with("inc.svh")));
        assert!(
            output
                .source_map
                .iter()
                .any(|m| m.source.file.ends_with("inc.svh"))
        );
    }

    #[test]
    fn honors_ifdef_else_endif() {
        let dir = create_temp_dir("svpp_ifdef");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define FLAG 1\n`ifdef FLAG\nlogic yes;\n`else\nlogic no;\n`endif\n",
        )
        .expect("write top");

        let output = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("preprocess ifdef");
        assert!(output.text.contains("logic yes;"));
        assert!(!output.text.contains("logic no;"));
    }

    #[test]
    fn expands_function_macro_with_token_paste_and_stringize() {
        let dir = create_temp_dir("svpp_func");
        let input = dir.join("top.sv");
        fs::write(
            &input,
            "`define CAT(a,b) a``b\n`define STR(x) `\"x\nwire `CAT(sig,_id);\nstring s = `STR(hello);\n",
        )
        .expect("write top");

        let output = preprocess_systemverilog_file(&input, &SvPreprocessorConfig::default())
            .expect("preprocess function-like macros");
        assert!(output.text.contains("wire sig_id;"));
        assert!(output.text.contains("string s = \"hello\";"));
    }
}
