"""PGEN-RGX-0078-0170 — the corrected STACKED-lever arithmetic.

Reproduces (a) why the -0169 call-off conclusion was wrong (compound, don't
sum-and-compare) and (b) why the -0168 sub-noise rule is refuted by the
campaign's own landed history. Stdlib only; no repo inputs beyond the banked
figures quoted in MEMORY.md / the -0162 profile."""
GEO, TARGET = 1263.4, 1000.0
need = 1 - TARGET / GEO

S = [  # (lever, conservative %, optimistic %, mass / source)
    ('G1-C per-atom node/arena/tape',       4.0, 9.0, 'alloc_extend+memmove 13.3% (-0168 §3)'),
    ('G3 per-parse setup+teardown',         4.0, 7.0, 'setup ~3% + teardown ~3% (-0162 §2)'),
    ('spine dispatch self',                 2.0, 5.0, '22-25% self, LARGEST UNPRICED (-0162 §2)'),
    ('build-value to_shaped/cascade_build', 2.0, 5.0, '5-10% cum (-0162 §2)'),
    ('memo insert (hashbrown)',             1.0, 3.0, '3.6-3.8% self (-0162 §2)'),
    ('C2 fact-op constants',                1.0, 3.0, '3.7% ceiling (-0162 §4 refused ALONE)'),
    ('G1-B box ParseError',                 0.5, 1.5, 'drop_in_place on success path (-0168 refused)'),
]

print(f"need: {GEO} -> <{TARGET} ns = -{need*100:.1f}%\n")
print(f"{'lever':38s} {'cons':>6s} {'opt':>6s}   mass")
c = o = 1.0
for n, lo, hi, src in S:
    c *= 1 - lo/100; o *= 1 - hi/100
    print(f"{n:38s} {-lo:5.1f}% {-hi:5.1f}%   {src}")
print(f"\nCOMPOUNDED conservative: -{(1-c)*100:.1f}%  -> {GEO*c:.0f} ns")
print(f"COMPOUNDED optimistic  : -{(1-o)*100:.1f}%  -> {GEO*o:.0f} ns"
      f"   {'<<< CROSSES THE <1us BAR' if GEO*o < TARGET else ''}")
print(f"\nsum-of-parts (the WRONG framing -0169 used): "
      f"-{sum(x[1] for x in S):.1f}% .. -{sum(x[2] for x in S):.1f}%")

print("\n" + "="*70)
print("WHY THE -0168 SUB-NOISE RULE IS REFUTED BY THIS CAMPAIGN'S OWN HISTORY")
print("="*70)
small = [('construction cache', 3.8), ('P-env', 2.3), ('Q quant-guard', 4.3),
         ('terminal-literal', 4.8), ('P2 degenerate dispatch', 5.3), ('FxHash', 5.4)]
c2 = 1.0
for n, v in small:
    c2 *= 1 - v/100
    print(f"   -{v:4.1f}%  {n}   (LANDED; the old rule would have REFUSED it)")
print(f"\n   these six SMALL levers alone compound to -{(1-c2)*100:.1f}%"
      f"  --  vs the -{need*100:.1f}% the whole bar needs.")
