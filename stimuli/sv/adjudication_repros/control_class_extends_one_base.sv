// SV-CORPUS-GRAD.13x.b - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// class Cls extends Base1;
// IEEE 1800-2017 A.1.2 section-41-data-read-api.md:183-190. isolates the defect to the second base.
class Base1; endclass
class Base2; endclass
class Cls extends Base1;
endclass
