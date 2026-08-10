//! `SV-CORPUS-GRAD.12c.1` — the encoding regression lock for USER SOURCE TEXT.
//!
//! The defect this locks out (finding F1 of `.12c`): PGEN refused any source file that was not
//! valid UTF-8, so thirteen ISO-8859-1 files of the tracked SystemVerilog corpus — twelve of them
//! carrying a single `0xA9` (`©`) inside a header comment — never reached the parser at all. The
//! file was refused; no construct was ever rejected.
//!
//! The property locked here is stronger than "Latin-1 no longer errors": **the encoding a file is
//! written in must not change the parse verdict.** One `must_accept` and one `must_reject`
//! SystemVerilog source, each rendered in five encodings, must produce the same answer five
//! times. That is what makes a future regression visible — a reader that mangles bytes instead of
//! decoding them would flip the accept row, and one that refuses would fail both.
//!
//! ⛔ The fixtures are BUILT HERE from explicit byte sequences rather than tracked as files on
//! disk, and that is deliberate. A tracked non-UTF-8 fixture is exactly the kind of file an
//! editor, a `.gitattributes` text filter or a well-meaning "fix encoding" commit silently
//! normalises to UTF-8 — at which point the lock still passes while testing nothing at all.
//! Bytes constructed in the test cannot rot that way.

#![cfg(feature = "generated_parsers")]

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use pgen::parser_registry;
use pgen::source_text::{SourceEncoding, read_source_file};

const PROFILE: Option<&str> = Some("sv_2017");

/// Valid SystemVerilog whose comment is the exact corpus shape: a copyright line whose `©` is the
/// only non-ASCII character in the file, with a pure-ASCII token stream after it.
const MUST_ACCEPT: &str = "/// Copyright by Syntacore LLC © 2016-2021\nmodule m;\n  logic a;\nendmodule\n";

/// No derivation in IEEE 1800-2017 Annex A: a module body cannot contain a bare `endmodule` after
/// the module has already ended. Same comment line, so the two rows differ only in the code.
const MUST_REJECT: &str = "/// Copyright by Syntacore LLC © 2016-2021\nmodule m;\nendmodule\nendmodule\n";

/// The five renderings of one logical source. `Latin1` is the one the corpus actually exercises;
/// the other four are there so the reader's decision ladder is locked in both directions.
fn encode(text: &str, encoding: SourceEncoding) -> Vec<u8> {
    match encoding {
        SourceEncoding::Utf8 => text.as_bytes().to_vec(),
        SourceEncoding::Utf8Bom => {
            let mut bytes = vec![0xEF, 0xBB, 0xBF];
            bytes.extend_from_slice(text.as_bytes());
            bytes
        }
        SourceEncoding::Latin1 => {
            // Every char of both fixtures is U+0000..=U+00FF, so this is exact — the same
            // property that makes ISO-8859-1 a total decoding on the way back in.
            text.chars()
                .map(|c| {
                    u8::try_from(c as u32).expect("fixture text must be representable in Latin-1")
                })
                .collect()
        }
        SourceEncoding::Utf16Le => {
            let mut bytes = vec![0xFF, 0xFE];
            for unit in text.encode_utf16() {
                bytes.extend_from_slice(&unit.to_le_bytes());
            }
            bytes
        }
        SourceEncoding::Utf16Be => {
            let mut bytes = vec![0xFE, 0xFF];
            for unit in text.encode_utf16() {
                bytes.extend_from_slice(&unit.to_be_bytes());
            }
            bytes
        }
    }
}

/// A scratch directory on the REPOSITORY's own volume (`PGEN_CODEBASE_ROOT` policy: project-owned
/// data never lands in an off-volume system temp).
fn scratch_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("source_text_encoding_regression")
        .join(format!("{name}_{nanos}"));
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

/// Read a file the way the shipped probes do, then parse it. Returns the accept verdict.
fn read_and_parse(path: &Path, expected_encoding: SourceEncoding) -> bool {
    let decoded = read_source_file(path).unwrap_or_else(|err| {
        panic!(
            "reader refused '{}' ({}): {err}",
            path.display(),
            expected_encoding
        )
    });
    assert_eq!(
        decoded.encoding,
        expected_encoding,
        "the reader named the wrong encoding for '{}'",
        path.display()
    );
    parser_registry::parse_sample_detail_with_options(
        "systemverilog",
        &decoded.text,
        PROFILE,
        &parser_registry::LibraryOptions::default(),
    )
    .expect("the systemverilog parser must be registered in this build")
    .is_ok()
}

const ENCODINGS: [SourceEncoding; 5] = [
    SourceEncoding::Utf8,
    SourceEncoding::Utf8Bom,
    SourceEncoding::Latin1,
    SourceEncoding::Utf16Le,
    SourceEncoding::Utf16Be,
];

#[test]
fn the_encoding_never_changes_the_verdict() {
    let dir = scratch_dir("verdict");
    for (label, source, expected_accept) in [
        ("must_accept", MUST_ACCEPT, true),
        ("must_reject", MUST_REJECT, false),
    ] {
        for encoding in ENCODINGS {
            let path = dir.join(format!("{label}_{}.sv", encoding.as_str()));
            std::fs::write(&path, encode(source, encoding)).expect("write fixture");
            assert_eq!(
                read_and_parse(&path, encoding),
                expected_accept,
                "{label} in {encoding} disagreed with the same source in UTF-8"
            );
        }
    }
}

#[test]
fn the_latin1_copyright_byte_survives_as_a_character() {
    // The regression that matters most in practice: a lossy reader would put U+FFFD here and the
    // verdict test above would still pass, because the mangled character sits inside a comment.
    let dir = scratch_dir("roundtrip");
    let path = dir.join("copyright.sv");
    std::fs::write(&path, encode(MUST_ACCEPT, SourceEncoding::Latin1)).expect("write fixture");

    let decoded = read_source_file(&path).expect("Latin-1 source must be readable");
    assert_eq!(decoded.encoding, SourceEncoding::Latin1);
    assert!(decoded.text.contains('©'), "text: {}", decoded.text);
    assert!(
        !decoded.text.contains('\u{FFFD}'),
        "a lossy decode substituted U+FFFD"
    );
    assert_eq!(decoded.text, MUST_ACCEPT);
    assert_eq!(
        decoded.first_non_utf8_byte_offset,
        Some(31),
        "the offset of the `©` byte in the file"
    );
    assert!(!decoded.encoding.preserves_disk_byte_offsets());
}

#[test]
fn a_plain_utf8_file_still_reports_disk_byte_offsets() {
    // The other direction of the honest-offsets contract: the common case must keep the property
    // that a reported position is a position in the file.
    let dir = scratch_dir("offsets");
    let path = dir.join("plain.sv");
    std::fs::write(&path, encode(MUST_ACCEPT, SourceEncoding::Utf8)).expect("write fixture");

    let decoded = read_source_file(&path).expect("UTF-8 source must be readable");
    assert_eq!(decoded.encoding, SourceEncoding::Utf8);
    assert!(decoded.encoding.preserves_disk_byte_offsets());
    assert_eq!(decoded.non_utf8_notice(&path.display().to_string()), None);
}

#[test]
fn the_thirteen_corpus_files_are_readable_at_head() {
    // The population that opened this leaf, asserted against the tracked corpus itself rather
    // than against a copy — a re-vendored submodule that reintroduced the refusal would show up
    // here instead of in a corpus run six steps later.
    //
    // Skipped (not failed) when the submodules are not checked out: this is a regression lock on
    // the reader, not a corpus-availability gate.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repo root");
    // The two shapes the population actually has — the twelve scr1 files carry `0xA9` (`©`), and
    // sv2v's deliberate lexer fixture carries `0xC1 0xE5` (`Áå`). Asserting `©` for all thirteen
    // is wrong, and this table exists because the first draft of this test did exactly that.
    let files = [
        ("stimuli/sv/subs/sv2v/test/lex/latin1.sv", "Áå"),
        ("stimuli/sv/subs/scr1/src/includes/scr1_memif.svh", "©"),
        ("stimuli/sv/subs/scr1/src/includes/scr1_ahb.svh", "©"),
    ];
    for (relative, expected) in files {
        let path = root.join(relative);
        if !path.exists() {
            eprintln!("skipping absent corpus file {relative} (submodule not checked out)");
            continue;
        }
        let decoded =
            read_source_file(&path).unwrap_or_else(|err| panic!("{relative} was refused: {err}"));
        assert_eq!(
            decoded.encoding,
            SourceEncoding::Latin1,
            "{relative} is the ISO-8859-1 population this leaf exists for"
        );
        assert!(
            decoded.text.contains(expected),
            "{relative} lost its {expected} byte(s)"
        );
        assert!(
            !decoded.text.contains('\u{FFFD}'),
            "{relative} was decoded lossily"
        );
    }
}
