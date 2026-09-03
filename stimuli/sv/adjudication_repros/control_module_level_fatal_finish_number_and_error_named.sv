// SV-CORPUS-GRAD.13x.c - a CONTROL: the nearest LEGAL twin (one token different) - must keep ACCEPTING.
// $fatal(1, "x"); and $error(.x("x")); at module level - a finish_number first, and the $error arm's bare list_of_arguments
// IEEE 1800-2017 A.1.4 elaboration_system_task section-41-data-read-api.md:301-309. isolates the defect to the first argument of $fatal.
module top;
  $fatal(1, "x");
  $error(.x("x"));
endmodule
