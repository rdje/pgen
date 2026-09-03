// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// interface class Icls; pure virtual task t(); endclass
// IEEE 1800-2017 A.1.2 section-41-data-read-api.md:201-206. isolates the defect to the member kind.
interface class Icls;
  pure virtual task t();
endclass
