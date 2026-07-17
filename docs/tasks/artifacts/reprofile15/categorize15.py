import re, sys

# RE-PROFILE #15 (PGEN-RGX-0078-0111): the #14 categorizer extended with the
# post-D3 needles — the frameless boundary scanners the -0110 slice emitted
# (scan_<rule> / scan_rec_* / scan_post_predicate_gate) get their OWN 'SCAN'
# bucket so the landed D3 population's in-profile cost is directly visible.
# ALL #14 needles are KEPT deliberately (the serde-collapse precedent): bucket
# movement across #14 -> #15 is the in-profile confirmation signal for D3.
# Regression obligation: on the #14 sample data this categorizer must
# reproduce decomposition14.txt EXACTLY (the new needles are inert on old
# data — no pre-D3 symbol contains the scan_ prefixes).

def load(path, start_marker):
    lines = open(path).read().splitlines()
    i = next(k for k,l in enumerate(lines) if start_marker in l)
    out = []
    for l in lines[i+1:]:
        if l.strip().startswith('Binary Images'): break
        m = re.match(r'^\s+(.*\S)\s+(\d+)\s*$', l)
        if not m: continue
        sym, cnt = m.group(1), int(m.group(2))
        out.append((sym, cnt))
    return out

def bucket(sym):
    s = sym
    # allocator surface (malloc/free) — what a real allocator owns
    if any(k in s for k in ['NeverFreeBump','mi_free','mi_malloc','_mi_malloc','mi_theap','__rust_dealloc','no_alloc_shim','__rust_alloc']):
        return 'ALLOC/FREE'
    # timing (probe stopwatch — EXCLUDE from parse)
    if any(k in s for k in ['mach_absolute_time','clock_gettime','Timespec','mach_timebase']):
        return 'z-TIMING(excl)'
    # env var lookups (PGEN_* checks in the hot path)
    if any(k in s for k in ['__findenv_locked','env_read_lock','3env4__var','env4getenv','getenv','from_bytes_with_nul','5c_str']):
        return 'ENV-LOOKUP'
    # memops
    if any(k in s for k in ['_platform_memmove','_platform_memset','memcpy','swap_nonoverlapping']):
        return 'MEMOPS'
    # teardown / drop of value trees (doomed + committed)
    if ('drop_in_place' in s or 'ops..drop..Drop' in s or 'Drop$GT$::drop' in s
        or 'dying_next' in s or 'deallocating' in s or 'take_front' in s
        or ('Dying' in s) or 'IntoIter' in s):
        return 'TEARDOWN/DROP'
    # semantic runtime (tournament machinery)
    if any(k in s for k in ['rollback_to_labeled','extract_delta_since','::checkpoint','rule_context','predicate_defs','SemanticRuntimeState','SemanticRuntimeDelta','RecursionGuard']):
        return 'SEMANTIC-RT'
    # #15: the D3 frameless boundary scanners (scan_<rule> + the scan_rec_*
    # recognizer helpers + the scan_post_predicate_gate tail) — checked BEFORE
    # CONSTRUCT and MATCH/DISPATCH so the D3 population is its own visible
    # bucket. sample output is fully demangled (RegexParser::scan_x::h...) and
    # the #14 data contains ZERO 'scan' symbols, so the plain needle is
    # provably inert on old data (verified by the regression run below).
    if 'scan_' in s:
        return 'SCAN'
    # #14: the shaped-value arena surface (NodeArena bump lanes) — checked BEFORE
    # CONSTRUCT so 'alloc_rendered_string' is not caught by the 'String' needle
    if any(k in s for k in ['NodeArena','alloc_shaped_','alloc_rendered_string']):
        return 'ARENA'
    # value construction (build data structures: btree/vec/string/serde/hashmap
    # + #14: the PgenValue committed-value lanes that replaced them)
    if any(k in s for k in ['btree','BTreeMap','RawVec','raw_vec','try_allocate_in','spec_extend','spec_from_iter','from_iter',
                            'String','ParseContent$u20$as$u20$core..clone','vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..clone',
                            'hashbrown','HashMap','hash_one','reserve_rehash','to_json_value','value..Value',
                            'iterator..Iterator$GT$::fold','with_capacity','fallible_with_capacity','finish_grow','grow_one',
                            'LazyLeafRange','init_front','Equivalent','contains_key',
                            'PgenValue','pgen_value','to_shaped_value','insert_object_pair','from_serde','binary_search']):
        return 'CONSTRUCT'
    # MTB-B: the BUILD pass constructs values — it belongs to CONSTRUCT
    if 'cascade_build_' in s:
        return 'CONSTRUCT'
    # MTB-B: derivation-tape machinery (events, segments, splices, cursors)
    if any(k in s for k in ['deriv_', 'DerivEvent', 'ThinDerivMemoEntry', 'extend_from_slice', 'copy_within']):
        return 'TAPE-MACHINERY'
    # matching / dispatch (the parsing floor; cascade_match_ + orchestrators)
    if any(k in s for k in ['cascade_','parse_regex','parse_letter','parse_full','match_string','memcmp',
                            'memoized_call','prepare_parse_state','RegexParser::new','call_once','FnOnce','LocalKey']):
        return 'MATCH/DISPATCH'
    # arena
    if 'typed_arena' in s or 'Arena' in s:
        return 'ARENA'
    return 'UNKNOWN'

for label, path, marker in [(sys.argv[1], sys.argv[2], 'Sort by top of stack')]:
    data = load(path, marker)
    total = sum(c for _,c in data)
    buckets = {}
    for sym,c in data:
        b = bucket(sym)
        buckets.setdefault(b, [0,0])
        buckets[b][0]+=c; buckets[b][1]+=1
    print(f"\n===== {label} — total leaf samples in section: {total} =====")
    # parse-only total excludes timing
    excl = buckets.get('z-TIMING(excl)',[0,0])[0]
    ptotal = total - excl
    for b in sorted(buckets, key=lambda k:-buckets[k][0]):
        cnt,n = buckets[b]
        print(f"  {b:16} {cnt:6}  {100*cnt/total:5.1f}% of all  {100*cnt/ptotal:5.1f}% of parse   ({n} syms)")
    print(f"  parse-only total (excl timing): {ptotal}")
