// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// class Cls; class inner; endclass endclass - a nested ORDINARY class
// IEEE 1800-2017 A.1.2 class_item section-41-data-read-api.md:191-200. isolates the nested-interface-class defect to the interface keyword.
class Cls;
  class inner; endclass
endclass
