//! Decoding **user source text** — the one place PGEN turns a file's bytes into the `&str` a
//! generated parser consumes.
//!
//! `SV-CORPUS-GRAD.12c.1` (finding F1). Before this module every source-text reader called
//! `std::fs::read_to_string`, which refuses any byte stream that is not valid UTF-8. Thirteen
//! files of the tracked SystemVerilog corpus are ISO-8859-1 — twelve of them carry a single
//! `0xA9` byte, the `©` in `/// Copyright by Syntacore LLC © 2016-2021` — so PGEN refused the
//! **file** rather than rejecting a construct, and those rows testified to nothing.
//!
//! Three facts fix the shape of the fix:
//!
//! 1. **The LRM permits it.** IEEE 1800-2017 §5.4 constrains only where a comment starts and
//!    ends, never its content; string literals and identifiers are separately restricted to
//!    ASCII. A `©` in a header comment is valid SystemVerilog.
//! 2. **The engine is already encoding-blind.** A generated parser holds `input: &'input str`
//!    and indexes it by byte offset, which is why multi-byte UTF-8 has always passed straight
//!    through (100 corpus files prove it). Nothing below the reader needs to change.
//! 3. **This is not "add Unicode support".** UTF-8 already works. The failing files are
//!    *Latin-1*, which is *invalid* UTF-8 — a decoding question, not a character-repertoire one.
//!
//! ## What this module deliberately does NOT do
//!
//! It is **not** a general encoding-tolerant file reader, and it must not be called on files PGEN
//! itself writes (JSON manifests, generated Rust, reports, AST dumps). Those are UTF-8 by
//! construction; a decode failure there means corruption or a truncated write and must stay loud.
//! The categorised enumeration of every reader in the crate is
//! `docs/tasks/artifacts/sv_corpus_grad/source_text_readers/enumeration.md`.
//!
//! ## The decision ladder
//!
//! | input | verdict |
//! |---|---|
//! | `EF BB BF` BOM | UTF-8, BOM stripped; **refuses** if the body is then invalid UTF-8 |
//! | `FF FE` / `FE FF` BOM | UTF-16 LE/BE transcode; **refuses** on an odd length or an unpaired surrogate |
//! | no BOM, valid UTF-8 | UTF-8, byte-identical to disk (the overwhelmingly common case) |
//! | no BOM, invalid UTF-8 | **ISO-8859-1**, a *total* decoding — every byte maps, so it cannot fail |
//!
//! Two properties of that ladder are load-bearing and were chosen, not defaulted into:
//!
//! - **A file that DECLARES its encoding and then contradicts it is refused, not guessed at.** A
//!   BOM is a producer's explicit claim; silently re-reading such a file under another encoding
//!   would substitute a guess for a stated fact. The refusal names the exact byte offset, so it
//!   is actionable — unlike the pre-`.12c.1` message, which named only the file.
//! - **ISO-8859-1, not Windows-1252, is the no-BOM fallback.** ISO-8859-1 is *total* (all 256
//!   byte values map, and `char as u8` round-trips exactly); CP1252 leaves five bytes undefined,
//!   so it can fail and would need its own fallback. Since the LRM already confines non-ASCII
//!   bytes to comments, the glyph choice can never change the token stream — only totality can,
//!   and only one of the two candidates has it.
//!
//! ## The honest caveat on byte offsets
//!
//! Transcoding shifts offsets relative to the file on disk: one extra byte per non-ASCII byte
//! under Latin-1, and a wholesale change under UTF-16. Positions PGEN reports (including
//! `furthest_position`) are offsets into the DECODED text. [`SourceEncoding::preserves_disk_byte_offsets`]
//! answers this per file so a caller never has to assume, and [`DecodedSource::non_utf8_notice`]
//! says it out loud to the operator. Exact on-disk offsets would need a span-mapping table, which
//! is deliberately out of scope here.

use std::path::Path;

/// The encoding a source file was decoded **from**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceEncoding {
    /// No BOM, and the bytes are valid UTF-8. The decoded text is byte-identical to the file.
    Utf8,
    /// A UTF-8 BOM (`EF BB BF`) was present and stripped.
    Utf8Bom,
    /// A UTF-16 little-endian BOM (`FF FE`) was present; the body was transcoded.
    Utf16Le,
    /// A UTF-16 big-endian BOM (`FE FF`) was present; the body was transcoded.
    Utf16Be,
    /// No BOM and not valid UTF-8; decoded byte-per-`char` as ISO-8859-1.
    Latin1,
}

impl SourceEncoding {
    /// The stable operator-facing name. Used in diagnostics and in the preprocessor warning
    /// detail, so it is part of what a downstream reader may match on.
    pub fn as_str(self) -> &'static str {
        match self {
            SourceEncoding::Utf8 => "utf-8",
            SourceEncoding::Utf8Bom => "utf-8-bom",
            SourceEncoding::Utf16Le => "utf-16le",
            SourceEncoding::Utf16Be => "utf-16be",
            SourceEncoding::Latin1 => "iso-8859-1",
        }
    }

    /// `true` when a byte offset into the decoded text is also a byte offset into the file on
    /// disk — i.e. nothing was stripped and nothing was transcoded.
    ///
    /// Only plain [`SourceEncoding::Utf8`] has this property: a stripped BOM shifts everything by
    /// three, Latin-1 adds a byte per non-ASCII byte, and UTF-16 changes the encoding wholesale.
    pub fn preserves_disk_byte_offsets(self) -> bool {
        matches!(self, SourceEncoding::Utf8)
    }
}

impl std::fmt::Display for SourceEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A successfully decoded source file: the text, and how it had to be read to get there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedSource {
    /// The decoded text, ready to hand to a parser.
    pub text: String,
    /// The encoding it was decoded from.
    pub encoding: SourceEncoding,
    /// For [`SourceEncoding::Latin1`] only: the offset **in the file** of the first byte that
    /// forced the fallback. `None` for every other encoding.
    pub first_non_utf8_byte_offset: Option<usize>,
}

impl DecodedSource {
    /// A single-line operator notice, or `None` when the file was plain UTF-8 and there is
    /// nothing to say.
    ///
    /// Callers print this to **stderr**. It is deliberately quiet in the common case: emitting a
    /// line per file would put 16 336 lines into a corpus run and train the operator to ignore
    /// them.
    ///
    /// ⛔ The text must never contain `furthest_position=`. `stimuli/run_external_corpus.sh`
    /// captures probe stderr and extracts the parse position by scanning for that literal.
    pub fn non_utf8_notice(&self, path_display: &str) -> Option<String> {
        if self.encoding == SourceEncoding::Utf8 {
            return None;
        }
        let offsets = if self.encoding.preserves_disk_byte_offsets() {
            ""
        } else {
            "; reported byte offsets are into the decoded text, not the file"
        };
        let first = match self.first_non_utf8_byte_offset {
            Some(at) => format!(" (first non-UTF-8 byte at file offset {at})"),
            None => String::new(),
        };
        Some(format!(
            "source-encoding: {} decoded as {}{}{}",
            path_display,
            self.encoding.as_str(),
            first,
            offsets
        ))
    }
}

/// Why a byte stream could not be decoded.
///
/// Every variant describes a file that **states** its encoding through a BOM and then contradicts
/// it. There is no variant for "not valid UTF-8": that case is not an error, it is the Latin-1
/// fallback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceDecodeError {
    /// A UTF-8 BOM was present but the body is not valid UTF-8, first offending byte at this
    /// offset in the file.
    Utf8BomBodyInvalid { at: usize },
    /// A UTF-16 BOM was present but the body has an odd number of bytes, so it is not a whole
    /// number of code units.
    Utf16OddLength { encoding: SourceEncoding, bytes: usize },
    /// A UTF-16 BOM was present but the body contains an unpaired surrogate at this code-unit
    /// index (counting from the first unit after the BOM).
    Utf16UnpairedSurrogate {
        encoding: SourceEncoding,
        unit_index: usize,
    },
}

impl std::fmt::Display for SourceDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceDecodeError::Utf8BomBodyInvalid { at } => write!(
                f,
                "file starts with a UTF-8 BOM but is not valid UTF-8 (first invalid byte at file offset {at}); \
                 the BOM states the encoding, so PGEN refuses rather than guessing another one"
            ),
            SourceDecodeError::Utf16OddLength { encoding, bytes } => write!(
                f,
                "file starts with a {encoding} BOM but its body is {bytes} bytes, which is not a whole number of 16-bit code units"
            ),
            SourceDecodeError::Utf16UnpairedSurrogate {
                encoding,
                unit_index,
            } => write!(
                f,
                "file starts with a {encoding} BOM but carries an unpaired surrogate at code unit {unit_index}"
            ),
        }
    }
}

impl std::error::Error for SourceDecodeError {}

const BOM_UTF8: [u8; 3] = [0xEF, 0xBB, 0xBF];
const BOM_UTF16_LE: [u8; 2] = [0xFF, 0xFE];
const BOM_UTF16_BE: [u8; 2] = [0xFE, 0xFF];

/// Decode a source file's bytes per the ladder in the module docs.
///
/// Takes the buffer by value so the common path — no BOM, valid UTF-8 — is a move into the
/// `String` rather than a copy of the whole file.
pub fn decode_source_bytes(mut bytes: Vec<u8>) -> Result<DecodedSource, SourceDecodeError> {
    if bytes.starts_with(&BOM_UTF8) {
        // Drain rather than slice-and-copy: the body is the whole file, and it is about to be
        // moved into the `String` either way.
        bytes.drain(..BOM_UTF8.len());
        return match String::from_utf8(bytes) {
            Ok(text) => Ok(DecodedSource {
                text,
                encoding: SourceEncoding::Utf8Bom,
                first_non_utf8_byte_offset: None,
            }),
            Err(err) => Err(SourceDecodeError::Utf8BomBodyInvalid {
                at: BOM_UTF8.len() + err.utf8_error().valid_up_to(),
            }),
        };
    }
    if bytes.starts_with(&BOM_UTF16_LE) {
        return decode_utf16(&bytes[BOM_UTF16_LE.len()..], SourceEncoding::Utf16Le);
    }
    if bytes.starts_with(&BOM_UTF16_BE) {
        return decode_utf16(&bytes[BOM_UTF16_BE.len()..], SourceEncoding::Utf16Be);
    }

    match String::from_utf8(bytes) {
        Ok(text) => Ok(DecodedSource {
            text,
            encoding: SourceEncoding::Utf8,
            first_non_utf8_byte_offset: None,
        }),
        Err(err) => {
            let at = err.utf8_error().valid_up_to();
            let raw = err.into_bytes();
            // ISO-8859-1 is a total decoding: U+0000..=U+00FF are exactly byte values 0..=255.
            Ok(DecodedSource {
                text: raw.iter().map(|&b| b as char).collect(),
                encoding: SourceEncoding::Latin1,
                first_non_utf8_byte_offset: Some(at),
            })
        }
    }
}

fn decode_utf16(body: &[u8], encoding: SourceEncoding) -> Result<DecodedSource, SourceDecodeError> {
    if body.len() % 2 != 0 {
        return Err(SourceDecodeError::Utf16OddLength {
            encoding,
            bytes: body.len(),
        });
    }
    let units: Vec<u16> = body
        .chunks_exact(2)
        .map(|pair| match encoding {
            SourceEncoding::Utf16Be => u16::from_be_bytes([pair[0], pair[1]]),
            _ => u16::from_le_bytes([pair[0], pair[1]]),
        })
        .collect();
    // `from_utf16` reports only "there was a bad unit", so locate it ourselves to keep the
    // refusal actionable — the whole point of refusing instead of substituting U+FFFD.
    match String::from_utf16(&units) {
        Ok(text) => Ok(DecodedSource {
            text,
            encoding,
            first_non_utf8_byte_offset: None,
        }),
        Err(_) => Err(SourceDecodeError::Utf16UnpairedSurrogate {
            encoding,
            unit_index: first_unpaired_surrogate(&units).unwrap_or(0),
        }),
    }
}

/// Index of the first UTF-16 code unit that is a surrogate without its partner.
fn first_unpaired_surrogate(units: &[u16]) -> Option<usize> {
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        if (0xD800..0xDC00).contains(&unit) {
            match units.get(index + 1) {
                Some(low) if (0xDC00..0xE000).contains(low) => index += 2,
                _ => return Some(index),
            }
        } else if (0xDC00..0xE000).contains(&unit) {
            return Some(index);
        } else {
            index += 1;
        }
    }
    None
}

/// Read and decode a user-source file.
///
/// ⛔ Only for files a human or a vendor tool authored. See the module docs for what belongs here
/// and what must keep using `std::fs::read_to_string`.
pub fn read_source_file(path: impl AsRef<Path>) -> anyhow::Result<DecodedSource> {
    use anyhow::Context;
    let path = path.as_ref();
    // Both errors name the path: `std::io::Error` does not carry it, and a caller that only
    // `.expect()`s the result (the compile-and-run harness's emitted probe does) would otherwise
    // report a failure with no way to tell WHICH file failed.
    let bytes =
        std::fs::read(path).with_context(|| format!("failed to read '{}'", path.display()))?;
    decode_source_bytes(bytes).with_context(|| format!("failed to decode '{}'", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode(bytes: &[u8]) -> DecodedSource {
        decode_source_bytes(bytes.to_vec()).expect("decode should succeed")
    }

    #[test]
    fn plain_ascii_is_utf8_and_offset_preserving() {
        let decoded = decode(b"module m; endmodule\n");
        assert_eq!(decoded.encoding, SourceEncoding::Utf8);
        assert_eq!(decoded.text, "module m; endmodule\n");
        assert_eq!(decoded.first_non_utf8_byte_offset, None);
        assert!(decoded.encoding.preserves_disk_byte_offsets());
        assert_eq!(decoded.non_utf8_notice("m.sv"), None);
    }

    #[test]
    fn multibyte_utf8_still_passes_through_untouched() {
        // The pre-existing behaviour this leaf must not disturb: 100 corpus files are UTF-8 with
        // multi-byte sequences and have always parsed.
        let source = "// Copyright © 2016 — µ\nmodule m; endmodule\n";
        let decoded = decode(source.as_bytes());
        assert_eq!(decoded.encoding, SourceEncoding::Utf8);
        assert_eq!(decoded.text, source);
    }

    #[test]
    fn latin1_is_decoded_losslessly_not_replaced() {
        // The exact corpus shape: one 0xA9 byte inside a one-line comment.
        let bytes = b"/// Copyright by Syntacore LLC \xA9 2016-2021\nmodule m; endmodule\n";
        let decoded = decode(bytes);
        assert_eq!(decoded.encoding, SourceEncoding::Latin1);
        assert!(decoded.text.contains('©'), "text: {}", decoded.text);
        assert!(
            !decoded.text.contains('\u{FFFD}'),
            "lossy decoding would substitute U+FFFD"
        );
        assert_eq!(decoded.first_non_utf8_byte_offset, Some(31));
        assert!(!decoded.encoding.preserves_disk_byte_offsets());
    }

    #[test]
    fn latin1_decoding_is_total_and_round_trips() {
        // Every one of the 256 byte values maps, and maps back — the property that makes the
        // fallback unable to fail, and the reason ISO-8859-1 was chosen over CP1252.
        let all_bytes: Vec<u8> = (0u8..=255).collect();
        let decoded = decode(&all_bytes);
        assert_eq!(decoded.encoding, SourceEncoding::Latin1);
        let round_tripped: Vec<u8> = decoded.text.chars().map(|c| c as u32 as u8).collect();
        assert_eq!(round_tripped, all_bytes);
    }

    #[test]
    fn utf8_bom_is_stripped() {
        let mut bytes = BOM_UTF8.to_vec();
        bytes.extend_from_slice(b"module m; endmodule\n");
        let decoded = decode(&bytes);
        assert_eq!(decoded.encoding, SourceEncoding::Utf8Bom);
        assert_eq!(decoded.text, "module m; endmodule\n");
        assert!(!decoded.text.starts_with('\u{FEFF}'));
        assert!(!decoded.encoding.preserves_disk_byte_offsets());
    }

    #[test]
    fn utf8_bom_with_invalid_body_is_refused_with_an_offset() {
        let mut bytes = BOM_UTF8.to_vec();
        bytes.extend_from_slice(b"// \xA9\n");
        let err = decode_source_bytes(bytes).expect_err("a BOM that lies must be refused");
        assert_eq!(err, SourceDecodeError::Utf8BomBodyInvalid { at: 6 });
        assert!(err.to_string().contains("file offset 6"));
    }

    #[test]
    fn utf16le_and_utf16be_transcode() {
        let source = "module m; endmodule\n";
        for (bom, encoding, to_bytes) in [
            (
                BOM_UTF16_LE,
                SourceEncoding::Utf16Le,
                u16::to_le_bytes as fn(u16) -> [u8; 2],
            ),
            (
                BOM_UTF16_BE,
                SourceEncoding::Utf16Be,
                u16::to_be_bytes as fn(u16) -> [u8; 2],
            ),
        ] {
            let mut bytes = bom.to_vec();
            for unit in source.encode_utf16() {
                bytes.extend_from_slice(&to_bytes(unit));
            }
            let decoded = decode(&bytes);
            assert_eq!(decoded.encoding, encoding);
            assert_eq!(decoded.text, source);
            assert!(!decoded.encoding.preserves_disk_byte_offsets());
        }
    }

    #[test]
    fn utf16_odd_length_is_refused() {
        let mut bytes = BOM_UTF16_LE.to_vec();
        bytes.extend_from_slice(&[b'm', 0x00, b'o']);
        let err = decode_source_bytes(bytes).expect_err("half a code unit is not decodable");
        assert_eq!(
            err,
            SourceDecodeError::Utf16OddLength {
                encoding: SourceEncoding::Utf16Le,
                bytes: 3
            }
        );
    }

    #[test]
    fn utf16_unpaired_surrogate_is_refused_and_located() {
        let mut bytes = BOM_UTF16_LE.to_vec();
        for unit in [b'm' as u16, 0xD800, b'x' as u16] {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        let err = decode_source_bytes(bytes).expect_err("an unpaired surrogate is not decodable");
        assert_eq!(
            err,
            SourceDecodeError::Utf16UnpairedSurrogate {
                encoding: SourceEncoding::Utf16Le,
                unit_index: 1
            }
        );
    }

    #[test]
    fn utf16_surrogate_pairs_are_not_mistaken_for_unpaired_ones() {
        // U+1F600 is a legitimate pair; the locator must walk over it, not stop on it.
        let units: Vec<u16> = "a\u{1F600}b".encode_utf16().collect();
        assert_eq!(first_unpaired_surrogate(&units), None);
        assert_eq!(first_unpaired_surrogate(&[0xDC00]), Some(0));
        assert_eq!(first_unpaired_surrogate(&[b'a' as u16, 0xD800]), Some(1));
    }

    #[test]
    fn the_notice_never_carries_the_position_marker_the_corpus_runner_scans_for() {
        // `stimuli/run_external_corpus.sh` extracts the parse position from probe stderr by
        // scanning for `furthest_position=`. A notice containing it would forge a position.
        let bytes = b"// \xA9\n".to_vec();
        let decoded = decode_source_bytes(bytes).expect("latin-1 decode");
        let notice = decoded.non_utf8_notice("f.sv").expect("non-utf8 notice");
        assert!(!notice.contains("furthest_position="));
        assert!(notice.contains("iso-8859-1"));
        assert!(!notice.contains('\n'), "the notice must be one line");
    }

    #[test]
    fn empty_input_decodes_to_empty_utf8() {
        let decoded = decode(b"");
        assert_eq!(decoded.encoding, SourceEncoding::Utf8);
        assert!(decoded.text.is_empty());
    }

    #[test]
    fn a_lone_bom_prefix_is_not_mistaken_for_a_bom() {
        // 0xEF 0xBB alone is not a UTF-8 BOM; it is also not valid UTF-8, so it falls to Latin-1
        // rather than being sliced as if the third byte were there.
        let decoded = decode(&[0xEF, 0xBB]);
        assert_eq!(decoded.encoding, SourceEncoding::Latin1);
        assert_eq!(decoded.text, "\u{EF}\u{BB}");
    }
}
