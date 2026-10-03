// SV-CORPUS-GRAD.43 - a TOLERATED pin - text no IEEE 1800 edition derives, ACCEPTED under `sv_2023_tolerant` only; its strict twin must keep REJECTING.
// pure virtual task automatic t(); - a lifetime in a class method prototype
// no IEEE 1800 production derives it (the strict twin invalid_method_prototype_lifetime_pure_virtual.sv carries the citations); admitted by DIRECTOR DECISION 2026-10-02, docs/decisions/project_sv_opt_in_tolerance_profile.md
virtual class b;
  pure virtual task automatic t();
endclass
