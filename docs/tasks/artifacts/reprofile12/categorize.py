import re, sys

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
    # value construction (build data structures: btree/vec/string/serde/hashmap)
    if any(k in s for k in ['btree','BTreeMap','RawVec','raw_vec','try_allocate_in','spec_extend','spec_from_iter','from_iter',
                            'String','ParseContent$u20$as$u20$core..clone','vec..Vec$LT$T$C$A$GT$$u20$as$u20$core..clone',
                            'hashbrown','HashMap','hash_one','reserve_rehash','to_json_value','value..Value',
                            'iterator..Iterator$GT$::fold','with_capacity','fallible_with_capacity','finish_grow','grow_one',
                            'LazyLeafRange','init_front','Equivalent','contains_key']):
        return 'CONSTRUCT'
    # matching / dispatch (the parsing floor)
    if any(k in s for k in ['cascade_','parse_regex','parse_letter','parse_full','match_string','memcmp',
                            'memoized_call','prepare_parse_state','RegexParser::new','call_once','FnOnce','LocalKey']):
        return 'MATCH/DISPATCH'
    # arena
    if 'typed_arena' in s or 'Arena' in s:
        return 'ARENA'
    return 'UNKNOWN'

for label, path, marker in [('NFA (malloc-free)', sys.argv[1], 'Sort by top of stack'),
                            ('MIMALLOC (baseline)', sys.argv[2], 'Sort by top of stack')]:
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
