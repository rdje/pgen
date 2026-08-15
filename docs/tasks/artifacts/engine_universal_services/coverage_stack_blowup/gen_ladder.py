#!/usr/bin/env python3
"""ENGINE-UNIVERSAL-SERVICES.22 — the DEPTH LADDER for the coverage-stack blow-up.

`stimuli/sv/subs/Surelog/tests/ExponTimeIfElseGen/dut.sv` is a chain of `else if` arms inside a
generate loop — Surelog's own exponential-parse-time regression test. It parses in 0.077 s bare and
in 0.056 s under the TOOLBOX 3.4 entry dump, and NEVER finishes under the 3.5 outcome dump.

⛔ A single pathological file proves a hang, not a GROWTH LAW, and a growth law is what tells you
whether the mechanism is the memo, the guard, or the coverage stack. This emits the same construct
at arm counts 1..N so the two dumps can be measured against the SAME axis: if 3.4 is linear in the
arm count while 3.5 explodes, the blow-up is in the coverage stack and not in the parse.

Usage:  python3 <this> <outdir> [max_arms]
"""
import os
import sys

ARM = """\
            else if( i == ADDR_OFFSET_%d ) begin
                always_ff @(posedge clk) begin
                    if( shift_foo_bar ) begin
                        foo_bar_4[i] <= foo_bar_4[i+1];
                    end
                end
            end
"""

HEAD = """\
module Foo ( );
   parameter ADDR_OFFSET_PART_1 = 1;

    generate
        for( genvar i = 0 ; i <= 17 ; i++ ) begin
            if( i == ADDR_OFFSET_PART_0 ) begin
                always_ff @(posedge clk) begin
                    if( shift_foo_bar ) begin
                        foo_bar_4[i] <= foo_bar_4[i+1];
                    end
                end
            end
"""

TAIL = """\
        end
    endgenerate

endmodule // Foo
"""


def main() -> int:
    outdir = sys.argv[1] if len(sys.argv) > 1 else "rust/target/e22/ladder"
    max_arms = int(sys.argv[2]) if len(sys.argv) > 2 else 10
    os.makedirs(outdir, exist_ok=True)
    for n in range(0, max_arms + 1):
        body = "".join(ARM % k for k in range(n))
        path = os.path.join(outdir, f"arms{n:02d}.sv")
        with open(path, "w", encoding="utf-8") as fh:
            fh.write(HEAD + body + TAIL)
    print(f"ladder: wrote {max_arms + 1} files (0..{max_arms} else-if arms) -> {outdir}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
