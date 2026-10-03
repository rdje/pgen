// SV-CORPUS-GRAD.43 - an INVALID pin - the boundary of the tolerant profile's closed list; must keep REJECTING on every profile, `sv_2023_tolerant` included.
// interface class ic; pure virtual task automatic t(); endclass - a lifetime in an INTERFACE-CLASS method prototype
// IEEE 1800-2017 A.2.7 task_prototype docs/systemverilog/2017/txt/section-41-data-read-api.txt:753; the tolerant profile admits the lifetime in CLASS method prototypes only
interface class ic;
  pure virtual task automatic t();
endclass
