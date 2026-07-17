import re, sys

# RE-PROFILE #16 (PGEN-RGX-0078-0119): the #15 categorizer extended with the
# post-spine needles — the -0118 C3 smallvec inline-small segment machinery
# (ThinDerivSegMemoEntry / SmallVec) stays in TAPE-MACHINERY (the same
# machinery class #15 bucketed the old Vec segments into), and the -0115
# emitted const-width literal helper (match_lit_ascii) stays in
# MATCH/DISPATCH — both checked in a new block AFTER the SCAN check and
# BEFORE ARENA/CONSTRUCT so a SmallVec symbol can never leak into CONSTRUCT
# via an incidental substring. The -0118 C1 id-aware guard methods
# (check_cycle_id / enter_id / truncate_stack) need NO new needle: they are
# RecursionGuard:: methods already caught by the existing SEMANTIC-RT needle.
# ALL #15 needles are KEPT deliberately (the serde-collapse precedent):
# bucket movement across #15 -> #16 is the in-profile confirmation signal
# for the landed termlit + spine-machinery slices.
# Regression obligation: on the #15 sample data this categorizer must
# reproduce the categorize15 bucket table BYTE-IDENTICALLY (the new needles
# are inert on old data — sample15_raw.txt is grep-proven to contain zero
# 'SmallVec'/'smallvec'/'ThinDerivSegMemoEntry'/'match_lit_ascii' symbols).

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
    # semantic runtime (tournament machinery; RecursionGuard also owns the
    # -0118 id-aware methods check_cycle_id / enter_id / truncate_stack)
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
    # #16: the -0118/-0115 needles (provably absent from #14/#15 data).
    # C3 smallvec inline-small segments + the seg-memo entry type belong to
    # the tape machinery class; checked HERE (before ARENA/CONSTRUCT) so no
    # SmallVec inner symbol can leak into CONSTRUCT via 'with_capacity' etc.
    if any(k in s for k in ['ThinDerivSegMemoEntry','SmallVec','smallvec']):
        return 'TAPE-MACHINERY'
    # the -0115 emitted const-width terminal-literal helper (inline(always),
    # normally folded — visible here only if a call site kept a frame)
    if 'match_lit_ascii' in s:
        return 'MATCH/DISPATCH'
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
