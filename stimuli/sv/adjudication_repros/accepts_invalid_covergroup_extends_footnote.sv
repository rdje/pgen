// SV-CORPUS-GRAD.13c.2o — the OVER-ACCEPTANCE half of the same extraction defect, and the reason
// the reject half is invisible without it: this is `defect_covergroup_extends.sv` with the LRM's
// FOOTNOTE NUMBER typed into the source after the semicolon. No Annex A production derives it, and
// it is the only spelling PGEN accepts. Pinned so the two halves flip together.
// Expected today: ACCEPT on sv_2023. When .13c.2o lands the runner FAILS with "flip it to invalid".
class c;
  covergroup base;
  endgroup
  covergroup extends base;29
  endgroup
endclass
