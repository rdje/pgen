// EBNF-FRONTEND-SILENT-TRUNCATION.6 — parsed with --entry-rule systemverilog_parseable_file (TOOLBOX §1.2).
// grammars/systemverilog.ebnf:225-229: `parseable_source_item := semi -> {kind: "semi"}` followed by THREE `//` lines
// (`| package_import_declaration`, `| timeunits_declaration`, `| compiler_directive`) commented out on 2026-04-25 (795e9954).
// grammars/ebnf.ebnf:583-586 defines `//` as a line comment; the hand-written frontend knew only `#`, read those lines as
// live alternatives (each with an empty regex token from the `//`), and the released parser accepted them from that entry.
// Pinned against release 1.0.204 BEFORE the fix (expected ACCEPT, ratchet checked=270 failures=0); FLIPPED to class=invalid
// on the rebuilt release 1.0.205, where it is rc 1 on every profile — the row now guards the fix.
import p::*;
