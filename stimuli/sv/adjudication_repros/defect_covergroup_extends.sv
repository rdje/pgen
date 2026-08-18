// SV-CORPUS-GRAD.13c.2o — CONFIRMED DEFECT (rejects-valid), found 2026-08-18 while RE-DERIVING
// .13c.2m's published claim under DIRECTOR CHALLENGE.
// IEEE 1800-2023 A.2.11: `covergroup_declaration ::= … | covergroup extends covergroup_identifier ;`
// — covergroup INHERITANCE, legal inside a class (§19.4). PGEN REJECTS it, because the LRM prints
// that alternative as `| covergroup extends covergroup_identifier ;29` where `29` is a FOOTNOTE
// MARKER ("The extends specification of covergroup is allowed only within a class"), and the
// extractor turned the marker into a MANDATORY literal token — `kw_n_29_7719a1c7 := trivia /29\b/`.
// ⛔ So the ONLY way to parse this construct today is to type the footnote number into your source:
// `accepts_invalid_covergroup_extends_footnote.sv` is that text, and it ACCEPTS.
// sv_2023 ONLY — `covergroup extends` does not exist in 1800-2017, so sv_2017 rejects CORRECTLY
// (measured, both editions' Annex A greped).
// Expected today: REJECT. Expected once .13c.2o lands: ACCEPT.
class c;
  covergroup base;
  endgroup
  covergroup extends base;
  endgroup
endclass
