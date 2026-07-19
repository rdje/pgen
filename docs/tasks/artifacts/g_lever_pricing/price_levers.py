import json, math
R=[json.loads(l) for l in open('docs/tasks/artifacts/geomean_reprofile/strat60_census.jsonl')]
y=[float(r['min_ns']) for r in R]; b=[float(r['bytes']) for r in R]; e=[float(r['entries']) for r in R]
def mean(v): return sum(v)/len(v)
def cv(u,v):
    mu,mv=mean(u),mean(v); return sum((a-mu)*(c-mv) for a,c in zip(u,v))/len(u)
def fit2(x1,x2,yv):
    c11,c22,c12=cv(x1,x1),cv(x2,x2),cv(x1,x2); det=c11*c22-c12*c12
    b1=(c22*cv(x1,yv)-c12*cv(x2,yv))/det; b2=(c11*cv(x2,yv)-c12*cv(x1,yv))/det
    a=mean(yv)-b1*mean(x1)-b2*mean(x2)
    pred=[a+b1*p+b2*q for p,q in zip(x1,x2)]
    ssr=sum((yy-p)**2 for yy,p in zip(yv,pred)); dof=len(yv)-3; s2=ssr/dof
    se2=math.sqrt(s2*c11/(det*len(yv)))
    return a,b1,b2,se2

def fit1(x,yv):
    sl=cv(x,yv)/cv(x,x); ic=mean(yv)-sl*mean(x)
    pred=[ic+sl*xx for xx in x]
    ssr=sum((a-b_)**2 for a,b_ in zip(yv,pred)); sst=sum((a-mean(yv))**2 for a in yv)
    return ic,sl,1-ssr/sst
def corr(u,v): return cv(u,v)/math.sqrt(cv(u,u)*cv(v,v))
print("="*78); print("  0. REGRESSION DIAGNOSTICS (n=60 stratified census cells)"); print("="*78)
i1,s1,r1=fit1(e,y); print(f"  min_ns ~ entries only : {i1:7.1f} + {s1:5.2f}*entries   R2={r1:.4f}  <- the -0162 BANKED fit, reproduced")
i2,s2,r2=fit1(b,y); print(f"  min_ns ~ bytes   only : {i2:7.1f} + {s2:5.2f}*bytes     R2={r2:.4f}  <- BYTES FIT BETTER than entries")
print(f"  corr(bytes, entries)  : {corr(b,e):.4f}   VIF = {1/(1-corr(b,e)**2):.1f}   => severe collinearity")
print(f"  aggregate entries/byte: {sum(e)/sum(b):.3f}   and 21.28 ~= 111 ns/B / 5.2 entries/B  (the two fits are the SAME fit)")
print()
GEO=1263.4; NOISE=2.28; NOISE_NS=GEO*NOISE/100
ME=mean(e)
_,_,bl,sel = fit2(b,e,y)                                   # linear ns
_,_,bg,seg = fit2(b,e,[math.log(v) for v in y])            # log space

print("="*78); print("  A. ENTRY-COUNT FUSION  — re-priced after removing the bytes/entries confound"); print("="*78)
print(f"  banked -0162 solo fit      : min_ns = 306.0 + 21.28*entries      (reproduced exactly)")
print(f"  corr(bytes, entries)       : 0.900   -> entries is a PROXY for input size, not independent")
print(f"  joint linear coefficient   : {bl:.2f} ns/entry  (+/- {sel:.2f})   <- HALF the banked 21.28")
print(f"  joint log coefficient      : {bg:.5f} /entry    (+/- {seg:.5f})")
print()
for cut in (0.10,0.25,0.40,0.50):
    lin = cut*bl*ME/mean(y)*100
    lg  = (1-math.exp(-bg*cut*ME))*100
    print(f"   -{cut*100:3.0f}% entries : banked claim -{cut*21.28*ME/mean(y)*100:5.1f}%  ->  RE-PRICED -{lin:4.1f}% (linear) / -{lg:4.1f}% (log)"
          f"  = {GEO*lin/100:5.0f}-{GEO*lg/100:3.0f} ns  [{GEO*lin/100/NOISE_NS:.1f}-{GEO*lg/100/NOISE_NS:.1f}x noise]")
need_lin = 20.9/(bl*ME/mean(y)*100)*100
need_log = -math.log(1-0.209)/(bg*ME)*100
print(f"\n   entry cut required to reach geomean <1us (-20.9%): {need_lin:.0f}% (linear) / {need_log:.0f}% (log) of ALL entries")
print(f"   wrapper chains are 30-50% of entries (-0162 table) => PERFECT fusion tops out near the bar, not past it")

print(); print("="*78); print("  B. G1-C  — per-atom 72-B ParseNode materialization + arena copy + tape push"); print("="*78)
W=[(38.0,8.6,5.3),(35.3,6.7,6.5),(26.0,5.3,7.4)]   # log-share, alloc_extend%, memmove%
tot=sum(w for w,_,_ in W)
att=sum(w/tot*(a+m) for w,a,m in W)
print(f"  directly-attributable self-time (typed_arena::alloc_extend + _platform_memmove),")
print(f"  log-share-weighted across the three geomean bands = {att:.1f}% of parse time = {GEO*att/100:.0f} ns/parse")
print(f"  (the broader 27-31% alloc cluster also contains per-parse setup/teardown G1-C does NOT touch)")
print()
for cap,lbl in ((0.30,"conservative"),(0.50,"mid"),(0.70,"optimistic")):
    ns=GEO*att/100*cap; print(f"   capture {cap*100:3.0f}% ({lbl:12s}): -{ns:5.1f} ns = -{ns/GEO*100:4.1f}%  [{ns/NOISE_NS:.1f}x noise]")

print(); print("="*78); print(f"  C. VERDICT vs the measured noise floor  ({NOISE}% p-p same-binary = {NOISE_NS:.1f} ns)"); print("="*78)
print(f"   entry-count fusion @ -25% entries : -129 to -184 ns   4.5-6.4x noise   RUNNABLE")
print(f"   G1-C                              :  -50 to -118 ns   1.7-4.1x noise   runnable, smaller")
print(f"   G1-B (box ParseError)             : one drop_in_place call vs 1263 ns  << 1x noise   UNRUNNABLE")
print(f"   G1-A (landed, measured)           : +0.11%  -- panic scaffolding, never executed   CONFIRMED 0")
print()
print("   OVERLAP WARNING: the two live levers attack the SAME mass from opposite ends")
print("   (fusing an entry also deletes its arena alloc) -- their savings MUST NOT be summed.")
