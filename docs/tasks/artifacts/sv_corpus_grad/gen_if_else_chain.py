#!/usr/bin/env python3
"""Generate the SV-CORPUS-GRAD.11a reproducer: an N-deep if/else-if chain.

The parse cost is O(2^N) in N (chain depth), NOT in file size -- see the leaf
`.11a` in docs/tasks/SV-CORPUS-GRAD.md. Two variants are needed and the PAIR is
what refutes the semantic-store hypothesis:

  ident  conditions reference a declared identifier (fact lookups happen)
  lit    conditions are pure literals (no identifier => no fact lookups)

Both blow up with the same exponential base, so the driver is structural
ambiguity at `conditional_else_branch`, not the fact/store machinery.

Usage:  gen_if_else_chain.py N {ident|lit} OUT.sv
Measure: /usr/bin/time -l ./rust/target/release/parseability_probe \
             --parse systemverilog OUT.sv --profile sv_2017
Measured on the release probe (ident): n=12 114 MB, n=14 349 MB, n=16 1314 MB.
"""
import sys


def build(n: int, mode: str) -> str:
    out = ["module m;", "  logic [31:0] a;", "  logic [4:0] s;", "  always_comb begin"]
    for i in range(n):
        mask = f"32'h{i:08x}"
        cond = f"((a & ~({mask})) == {mask})" if mode == "ident" \
               else f"(({mask} & ~({mask})) == {mask})"
        out.append(f"    {'if' if i == 0 else 'end else if'} {cond} begin")
        out.append(f"      s = 5'd{i % 32};")
    out += ["    end else begin", "      s = 5'd31;", "    end", "  end", "endmodule"]
    return "\n".join(out) + "\n"


def main() -> int:
    if len(sys.argv) != 4 or sys.argv[2] not in ("ident", "lit"):
        print(__doc__, file=sys.stderr)
        return 2
    with open(sys.argv[3], "w") as fh:
        fh.write(build(int(sys.argv[1]), sys.argv[2]))
    return 0


if __name__ == "__main__":
    sys.exit(main())
