//! IEEE 1800-2017 §22.14 — which identifiers a `` `begin_keywords `` region reserves.
//!
//! `SVPP-EXPANSION.8`. §22.14: *"A pair of directives, `` `begin_keywords `` and `` `end_keywords ``,
//! can be used to specify what identifiers are reserved as keywords within a block of source code,
//! based on a specific version of IEEE Std 1364 or IEEE Std 1800."* The rule is LEXICAL — it changes
//! the reserved-word set, nothing else: *"The directives do not affect the semantics, tokens, and
//! other aspects of the SystemVerilog language."*
//!
//! The generated SystemVerilog parser reserves one set per PROFILE and cannot switch mid-file, so
//! the preprocessor renders the difference instead: inside a region, a token that the parse
//! profile reserves but the region's version does not is emitted as an ESCAPED identifier, which
//! §5.6.2 makes an ordinary identifier (*"A SystemVerilog keyword preceded by an escape character
//! is not interpreted as a keyword"*) and §5.6.1 makes the SAME identifier as the unescaped
//! spelling (*"an escaped identifier `\cpu3` is treated the same as a nonescaped identifier
//! `cpu3`"*). This module owns the version algebra; `sv_preprocessor` owns the rendering.
//!
//! ⛔ The tables below are DERIVED from the in-repo LRM text, never typed:
//! `python3 stimuli/sv/check_sv_keyword_versions.py --write` regenerates them and the
//! `SV-KEYWORD-VERSIONS` doctrine holds them equal to the standard on every commit — including the
//! two split points the PDF→md extraction destroys, each recovered by an oracle blind to the other
//! (IEEE 1364-2005 §19.11's own full tables; the unique strictly-increasing split; Annex B Table
//! B.1 as the union). The instrument's docstring carries the derivation in full.

use std::collections::BTreeSet;

/// One `version_specifier` of §22.14's Syntax 22-10.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeywordVersion {
    /// `"1364-1995"`
    Verilog1995,
    /// `"1364-2001-noconfig"` — §22.14.4: `"1364-2001"` minus the ten configuration keywords.
    Verilog2001NoConfig,
    /// `"1364-2001"`
    Verilog2001,
    /// `"1364-2005"`
    Verilog2005,
    /// `"1800-2005"`
    Sv2005,
    /// `"1800-2009"`
    Sv2009,
    /// `"1800-2012"`
    Sv2012,
    /// `"1800-2017"` — §22.14.9: adds no reserved keyword to `"1800-2012"`.
    Sv2017,
    /// `"1800-2023"` — IEEE 1800-2023 §22.14.9: adds no reserved keyword either.
    Sv2023,
}

// <derived-tables>
// ⛔ DERIVED — regenerate with:
//     python3 stimuli/sv/check_sv_keyword_versions.py --write
// held equal to the LRM text on every commit by the `SV-KEYWORD-VERSIONS` doctrine.

/// IEEE 1800-2017 Table 22-1 — the IEEE 1364-1995 reserved keywords (§22.14.2). 102 words.
const TABLE_22_1: &[&str] = &[
    "always", "and", "assign", "begin", "buf", "bufif0", "bufif1", "case", "casex", "casez",
    "cmos", "deassign", "default", "defparam", "disable", "edge", "else", "end", "endcase",
    "endfunction", "endmodule", "endprimitive", "endspecify", "endtable", "endtask", "event",
    "for", "force", "forever", "fork", "function", "highz0", "highz1", "if", "ifnone", "initial",
    "inout", "input", "integer", "join", "large", "macromodule", "medium", "module", "nand",
    "negedge", "nmos", "nor", "not", "notif0", "notif1", "or", "output", "parameter", "pmos",
    "posedge", "primitive", "pull0", "pull1", "pulldown", "pullup", "rcmos", "real", "realtime",
    "reg", "release", "repeat", "rnmos", "rpmos", "rtran", "rtranif0", "rtranif1", "scalared",
    "small", "specify", "specparam", "strong0", "strong1", "supply0", "supply1", "table", "task",
    "time", "tran", "tranif0", "tranif1", "tri", "tri0", "tri1", "triand", "trior", "trireg",
    "vectored", "wait", "wand", "weak0", "weak1", "while", "wire", "wor", "xnor", "xor",
];

/// Table 22-2 — what "1364-2001" ADDS to "1364-1995" (§22.14.3). 21 words.
const TABLE_22_2: &[&str] = &[
    "automatic", "cell", "config", "design", "endconfig", "endgenerate", "generate", "genvar",
    "incdir", "include", "instance", "liblist", "library", "localparam", "noshowcancelled",
    "pulsestyle_ondetect", "pulsestyle_onevent", "showcancelled", "signed", "unsigned", "use",
];

/// Table 22-3 — what "1364-2005" ADDS to "1364-2001" (§22.14.5). 1 word.
const TABLE_22_3: &[&str] = &[
    "uwire",
];

/// Table 22-4 — what "1800-2005" ADDS to "1364-2005" (§22.14.6). 97 words.
const TABLE_22_4: &[&str] = &[
    "alias", "always_comb", "always_ff", "always_latch", "assert", "assume", "before", "bind",
    "bins", "binsof", "bit", "break", "byte", "chandle", "class", "clocking", "const",
    "constraint", "context", "continue", "cover", "covergroup", "coverpoint", "cross", "dist",
    "do", "endclass", "endclocking", "endgroup", "endinterface", "endpackage", "endprogram",
    "endproperty", "endsequence", "enum", "expect", "export", "extends", "extern", "final",
    "first_match", "foreach", "forkjoin", "iff", "ignore_bins", "illegal_bins", "import",
    "inside", "int", "interface", "intersect", "join_any", "join_none", "local", "logic",
    "longint", "matches", "modport", "new", "null", "package", "packed", "priority", "program",
    "property", "protected", "pure", "rand", "randc", "randcase", "randsequence", "ref", "return",
    "sequence", "shortint", "shortreal", "solve", "static", "string", "struct", "super", "tagged",
    "this", "throughout", "timeprecision", "timeunit", "type", "typedef", "union", "unique",
    "var", "virtual", "void", "wait_order", "wildcard", "with", "within",
];

/// Table 22-5 — what "1800-2009" ADDS to "1800-2005" (§22.14.7). 23 words.
const TABLE_22_5: &[&str] = &[
    "accept_on", "checker", "endchecker", "eventually", "global", "implies", "let", "nexttime",
    "reject_on", "restrict", "s_always", "s_eventually", "s_nexttime", "s_until", "s_until_with",
    "strong", "sync_accept_on", "sync_reject_on", "unique0", "until", "until_with", "untyped",
    "weak",
];

/// Table 22-6 — what "1800-2012" ADDS to "1800-2009" (§22.14.8). 4 words.
const TABLE_22_6: &[&str] = &[
    "implements", "interconnect", "nettype", "soft",
];

/// §22.14.4 — the identifiers "1364-2001-noconfig" EXCLUDES from Table 22-2. 10 words.
const NOCONFIG_EXCLUSIONS: &[&str] = &[
    "cell", "config", "design", "endconfig", "incdir", "include", "instance", "liblist",
    "library", "use",
];

/// `PROFILE-ALIAS.2` — every `--profile` spelling `grammars/systemverilog.ebnf` declares,
/// paired with the keyword version that profile's parser reserves.
const PROFILE_KEYWORD_VERSION: &[(&str, KeywordVersion)] = &[
    ("2017", KeywordVersion::Sv2017),
    ("ieee1800-2017", KeywordVersion::Sv2017),
    ("ieee_1800_2017", KeywordVersion::Sv2017),
    ("sv_2017", KeywordVersion::Sv2017),
    ("2023", KeywordVersion::Sv2023),
    ("ieee1800-2023", KeywordVersion::Sv2023),
    ("ieee_1800_2023", KeywordVersion::Sv2023),
    ("sv_2023", KeywordVersion::Sv2023),
    ("sv_2023_tolerant", KeywordVersion::Sv2023),
    ("1364-2005", KeywordVersion::Verilog2005),
    ("ieee1364-2005", KeywordVersion::Verilog2005),
    ("ieee_1364_2005", KeywordVersion::Verilog2005),
    ("verilog_2005", KeywordVersion::Verilog2005),
];

/// The version specifiers each profile's own EDITION defines (§22.14 Syntax 22-10), read from
/// `keywords_version_specifier_<profile>` in `grammars/systemverilog.ebnf` — the parser refuses
/// any other spelling, so a region naming one is worth a diagnostic here.
const ADMITTED_SPECIFIERS: &[(&str, &[&str])] = &[
    ("sv_2017", &["1800-2017", "1800-2012", "1800-2009", "1800-2005", "1364-2005", "1364-2001-noconfig", "1364-2001", "1364-1995"]),
    ("sv_2023", &["1800-2023", "1800-2017", "1800-2012", "1800-2009", "1800-2005", "1364-2005", "1364-2001-noconfig", "1364-2001", "1364-1995"]),
    ("sv_2023_tolerant", &["1800-2023", "1800-2017", "1800-2012", "1800-2009", "1800-2005", "1364-2005", "1364-2001-noconfig", "1364-2001", "1364-1995"]),
    ("verilog_2005", &["1364-2005", "1364-2001-noconfig", "1364-2001", "1364-1995"]),
];
// </derived-tables>

impl KeywordVersion {
    /// Every specifier §22.14 defines, oldest first.
    pub const ALL: &'static [KeywordVersion] = &[
        KeywordVersion::Verilog1995,
        KeywordVersion::Verilog2001NoConfig,
        KeywordVersion::Verilog2001,
        KeywordVersion::Verilog2005,
        KeywordVersion::Sv2005,
        KeywordVersion::Sv2009,
        KeywordVersion::Sv2012,
        KeywordVersion::Sv2017,
        KeywordVersion::Sv2023,
    ];

    /// The spelling that appears between the quotes of `` `begin_keywords "…" ``.
    pub fn specifier(self) -> &'static str {
        match self {
            KeywordVersion::Verilog1995 => "1364-1995",
            KeywordVersion::Verilog2001NoConfig => "1364-2001-noconfig",
            KeywordVersion::Verilog2001 => "1364-2001",
            KeywordVersion::Verilog2005 => "1364-2005",
            KeywordVersion::Sv2005 => "1800-2005",
            KeywordVersion::Sv2009 => "1800-2009",
            KeywordVersion::Sv2012 => "1800-2012",
            KeywordVersion::Sv2017 => "1800-2017",
            KeywordVersion::Sv2023 => "1800-2023",
        }
    }

    /// §22.14: *"It shall be an error if an implementation does not recognize the version_specifier
    /// used with the `` `begin_keywords `` directive."* — `None` is that error's condition.
    pub fn from_specifier(raw: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|v| v.specifier() == raw)
    }

    /// The reserved-word set this version selects. §22.14.3/.5/.6/.7/.8 each say the version
    /// *"includes the identifiers listed in"* every earlier version *"plus"* its own table, so the
    /// sets are built by accumulation; §22.14.4 is the one SUBTRACTION in the clause.
    pub fn reserved_words(self) -> BTreeSet<&'static str> {
        let mut words: BTreeSet<&'static str> = TABLE_22_1.iter().copied().collect();
        if self == KeywordVersion::Verilog1995 {
            return words;
        }
        words.extend(TABLE_22_2.iter().copied());
        if self == KeywordVersion::Verilog2001NoConfig {
            for excluded in NOCONFIG_EXCLUSIONS {
                words.remove(excluded);
            }
            return words;
        }
        if self == KeywordVersion::Verilog2001 {
            return words;
        }
        words.extend(TABLE_22_3.iter().copied());
        if self == KeywordVersion::Verilog2005 {
            return words;
        }
        words.extend(TABLE_22_4.iter().copied());
        if self == KeywordVersion::Sv2005 {
            return words;
        }
        words.extend(TABLE_22_5.iter().copied());
        if self == KeywordVersion::Sv2009 {
            return words;
        }
        words.extend(TABLE_22_6.iter().copied());
        words
    }
}

/// The words a region must hand to the parser as ESCAPED identifiers: reserved by the profile the
/// expanded text will be parsed under, not reserved by the region's version.
///
/// The reverse difference is not renderable — text cannot turn a non-keyword into a keyword — so a
/// region naming a version RICHER than the profile is reported by the caller, not silently applied.
pub fn words_to_escape(profile: KeywordVersion, region: KeywordVersion) -> BTreeSet<&'static str> {
    let reserved_in_region = region.reserved_words();
    profile
        .reserved_words()
        .into_iter()
        .filter(|word| !reserved_in_region.contains(word))
        .collect()
}

/// `true` when everything `region` reserves is also reserved by `profile` — i.e. the rendering
/// above is a complete account of the difference between them.
pub fn region_within_profile(profile: KeywordVersion, region: KeywordVersion) -> bool {
    let reserved_in_profile = profile.reserved_words();
    region
        .reserved_words()
        .iter()
        .all(|word| reserved_in_profile.contains(word))
}

/// The keyword version the generated parser reserves under one `--profile` spelling.
///
/// `None` — no profile requested — is the parser's UNPROFILED reading, which keeps every dialect
/// alternative live and so reserves the UNION of the profiles' sets; that union is IEEE 1800-2017's
/// Table B.1, because `verilog_2005`'s set is a subset of it (§22.14.6). An unrecognised spelling
/// reads the same way: the parser refuses it on its own, and this stage does not duplicate that.
pub fn version_for_profile(profile: Option<&str>) -> KeywordVersion {
    match profile {
        None => KeywordVersion::Sv2017,
        Some(name) => PROFILE_KEYWORD_VERSION
            .iter()
            .find(|(spelling, _)| spelling.eq_ignore_ascii_case(name))
            .map(|(_, version)| *version)
            .unwrap_or(KeywordVersion::Sv2017),
    }
}

/// Does the EDITION behind this profile define `specifier` at all? §22.14's Syntax 22-10 enumerates
/// the specifiers per edition, and `grammars/systemverilog.ebnf` carries that enumeration per
/// profile, so a region naming one the edition does not define is a fact worth reporting — the
/// parser will refuse the directive itself.
pub fn profile_admits_specifier(profile: Option<&str>, specifier: &str) -> bool {
    let Some(name) = profile else {
        return KeywordVersion::from_specifier(specifier).is_some();
    };
    let canonical = PROFILE_KEYWORD_VERSION
        .iter()
        .find(|(spelling, _)| spelling.eq_ignore_ascii_case(name))
        .map(|(_, version)| *version);
    let Some(version) = canonical else {
        return KeywordVersion::from_specifier(specifier).is_some();
    };
    // the canonical profile name is the one whose own entry maps to itself
    ADMITTED_SPECIFIERS
        .iter()
        .find(|(canonical_name, _)| version_for_profile(Some(canonical_name)) == version)
        .map(|(_, admitted)| admitted.contains(&specifier))
        .unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// §22.14.9: `"1800-2017"` *"include[s] the identifiers listed in all previous versions and
    /// does not add new reserved keywords"* — and Annex B is that combination. 248 is the count
    /// `SV-RESERVED-WORDS` already holds the grammar's own regex to.
    #[test]
    fn the_1800_2017_set_is_annex_b() {
        assert_eq!(KeywordVersion::Sv2017.reserved_words().len(), 248);
        assert_eq!(
            KeywordVersion::Sv2017.reserved_words(),
            KeywordVersion::Sv2023.reserved_words()
        );
        assert_eq!(
            KeywordVersion::Sv2012.reserved_words(),
            KeywordVersion::Sv2017.reserved_words()
        );
    }

    #[test]
    fn the_versions_form_a_chain_and_noconfig_subtracts() {
        assert_eq!(KeywordVersion::Verilog1995.reserved_words().len(), 102);
        assert_eq!(KeywordVersion::Verilog2001.reserved_words().len(), 123);
        assert_eq!(KeywordVersion::Verilog2005.reserved_words().len(), 124);
        assert_eq!(KeywordVersion::Verilog2001NoConfig.reserved_words().len(), 113);
        for pair in KeywordVersion::ALL.windows(2) {
            if pair[0] == KeywordVersion::Verilog2001NoConfig {
                continue; // noconfig is a subtraction from 2001, not a step of the chain
            }
            assert!(
                region_within_profile(pair[1], pair[0]),
                "{} is not a subset of {}",
                pair[0].specifier(),
                pair[1].specifier()
            );
        }
        assert!(region_within_profile(
            KeywordVersion::Verilog2001,
            KeywordVersion::Verilog2001NoConfig
        ));
    }

    /// The four constructs the corpus rows turn on (`SVPP-EXPANSION.8`).
    #[test]
    fn the_corpus_rows_escape_sets_are_the_expected_words() {
        let escaped = words_to_escape(KeywordVersion::Sv2017, KeywordVersion::Verilog2005);
        assert!(escaped.contains("bit"));
        assert!(escaped.contains("logic"));
        assert!(escaped.contains("interface"));
        assert!(escaped.contains("endinterface"));
        assert!(!escaped.contains("uwire"), "uwire is a 1364-2005 keyword");
        assert!(!escaped.contains("automatic"), "automatic is a 1364-2001 keyword");

        let escaped = words_to_escape(KeywordVersion::Sv2017, KeywordVersion::Verilog1995);
        assert!(escaped.contains("automatic"), "automatic is not a 1364-1995 keyword");

        let escaped = words_to_escape(KeywordVersion::Sv2017, KeywordVersion::Verilog2001);
        assert!(escaped.contains("logic"));
        assert!(!escaped.contains("cell"));

        let escaped = words_to_escape(KeywordVersion::Sv2017, KeywordVersion::Verilog2001NoConfig);
        assert!(escaped.contains("cell"), "noconfig excludes `cell` (§22.14.4)");
        assert!(escaped.contains("library"));
        assert!(!escaped.contains("automatic"));
    }

    #[test]
    fn a_profile_spelling_resolves_to_its_parsers_reserved_set() {
        assert_eq!(version_for_profile(Some("sv_2017")), KeywordVersion::Sv2017);
        assert_eq!(version_for_profile(Some("2017")), KeywordVersion::Sv2017);
        assert_eq!(version_for_profile(Some("SV_2023")), KeywordVersion::Sv2023);
        assert_eq!(
            version_for_profile(Some("verilog_2005")),
            KeywordVersion::Verilog2005
        );
        assert_eq!(
            version_for_profile(Some("1364-2005")),
            KeywordVersion::Verilog2005
        );
        // no profile = the parser's union reading, which is Table B.1
        assert_eq!(version_for_profile(None), KeywordVersion::Sv2017);
        assert_eq!(version_for_profile(Some("no-such-profile")), KeywordVersion::Sv2017);
    }

    #[test]
    fn an_edition_admits_only_the_specifiers_its_own_clause_lists() {
        assert!(profile_admits_specifier(Some("sv_2017"), "1364-1995"));
        assert!(!profile_admits_specifier(Some("sv_2017"), "1800-2023"));
        assert!(profile_admits_specifier(Some("sv_2023"), "1800-2023"));
        assert!(profile_admits_specifier(Some("verilog_2005"), "1364-2005"));
        assert!(!profile_admits_specifier(Some("verilog_2005"), "1800-2005"));
        assert!(!profile_admits_specifier(None, "1800-1066"));
    }

    #[test]
    fn an_unknown_specifier_has_no_version() {
        assert_eq!(KeywordVersion::from_specifier("1364-2001"), Some(KeywordVersion::Verilog2001));
        assert_eq!(KeywordVersion::from_specifier("1364-2001-noconfig"), Some(KeywordVersion::Verilog2001NoConfig));
        assert_eq!(KeywordVersion::from_specifier("1364-2010"), None);
        assert_eq!(KeywordVersion::from_specifier(""), None);
    }
}
