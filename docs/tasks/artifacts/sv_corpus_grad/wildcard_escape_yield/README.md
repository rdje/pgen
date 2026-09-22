# The wildcard-escape yield sites (SV-CORPUS-GRAD.27, release 1.0.239)

The `.27` type escape (an undeclared identifier admitted in a type position while an `import pkg::*;` is open) made the same
bytes readable a second way at two sites where a resolution-free reading already stood, and under the default `longest_match`
branch policy the tie went to declaration order — two accepted trees moved on the first candidate:

| site | bytes | through 1.0.238 | first `.27` candidate | final (`1.0.239`) |
|---|---|---|---|---|
| `net_type_declaration` (IEEE 1800-2017 A.2.1.3) | `nettype unknown_t x;` under an open import | a nettype alias (second production, `net_type_identifier`'s escape) | a nettype over a data type (first production) | the alias — `nettype_alias_shape_ahead` |
| `ansi_port_header` (A.1.3) | `foo_t a` (no direction) under an open import | an interface port (`interface_port_header`) | a typed port (`net_port_header`) | the interface port — `wildcard_escape_interface_port_shape` |

The pins pre-screen caught the first (`ast_moved=2`, `control_nettype_alias_wildcard_import_sv2017` / `_sv2023`); the second had no
pin and was found by probing every `ident ident` shape the released grammar accepts under an open import (`probes/p_*.sv`), then
held by `probes/q_*.sv` and five pins. `ab.sh` re-runs the A/B (three grammars × both SV profiles). Two tables are kept: `ab_1.0.239.txt` — HEAD / the FIRST
candidate / the third candidate over 48 files: 92 rows — 59 identity, 5 restored to HEAD's bytes (the two yield sites), 22 movers
(HEAD rejects), 6 mixed probes whose tree differs from the first candidate's in the direction-less ports alone (verified by JSON
diff: only `kind: "named"` interface headers are added); and `ab_1.0.239_final.txt` — HEAD in the first grammar column, the FINAL
`1.0.239` bytes in both others, over 70 files (every probe here and every pin named for interface ports, nettypes or the wildcard
escape): 136 rows — 100 identity, 36 movers (HEAD rejects, the final accepts), and NO tree accepted by HEAD changed by the final
bytes; the untyped randsequence productions the third candidate refused are among the identity rows. A second class, found by the population pre-screen of that candidate (3 files lost acceptance) and by `probes/r_*.sv`: at an
implicit-capable site (`data_type_or_implicit` — tf ports, coverpoint labels, property formals) the escape read a LONE unresolved
identifier as the type where the implicit reading served it as the NAME; coverpoint labels and property formals lost their verdict,
untyped tf ports kept it with a moved tree. The guard is the grammar's own following-name lookahead restricted to the escape
(`wildcard_escape_lone_name_shape`: an unresolved bare identifier, packed dimensions, then NOT a name, `#` or `::` — its first cut
left out `::` and refused scoped types at those sites, 7 losses and 12 lost gains in the second population pre-screen,
`probes/r_scoped_types_at_implicit_sites.sv`); `probes/r_*.sv` are all byte-identical to the released bytes under the final grammar.
The randsequence production's optional return type (`probes/s_randseq_*.sv`) is the same site under another name — found by the
lanes on slang `all.sv` after the population selector (line-anchored) had missed every header-form `module x import pkg::*;` file;
`production_return_type_guarded` carries the guard there.

The rule, in `TOOLBOX.md` §1.5c: an escape yields wherever a resolution-free reading of the same
bytes already stands, and the pre-screen's identity column is read before the build.

`ab_1.0.240.txt` (SV-CORPUS-GRAD.28): the same A/B with HEAD = 1.0.239 and the shipped `.28` candidate (the parameterized-class-type escape confined to class bodies and typedefs), 158 rows over 81 files — 148 identity, 8 movers (the class-member and typedef shapes as pins and probes × two profiles), 2 rows refused on both sides (the module-scope member probe × two profiles; its pin among the identity rows), no tree accepted by HEAD changed (the class-body scope markers included).
