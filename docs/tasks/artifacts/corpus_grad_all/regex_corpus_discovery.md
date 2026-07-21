# CORPUS-GRAD-ALL.1 — regex family: exhaustive external-corpus discovery (2026-07-22)

Research-grounded discovery (web agent, primary sources; ~37 searches/fetches).
Baseline: regex_corpus_bundle pins pcre2-10.47 (full testdata incl. the
auto-generated Unicode-property test block) + php-8.4.19 ext/pcre (both tiers
already vendored, manifests/upstreams.lock.json).

## Ranked ADD recommendations (for the frozen roster; adjudication pending)

1. Perl t/re/re_tests (~2,000 keyed lines, result-code `c` = expected compile
   error + message) + reg_mesg.t (error/warning message pairs) — Perl/perl5,
   Artistic/GPL. Highest dialect fidelity; PCRE2 itself maintains the Perl
   interop lane (perltest.sh). Filter: Perl-only constructs + known divergences.
2. Lingua Franca polyglot corpus (SBULeeLab/LinguaFranca-FSE19,
   data/production-regexes/uniq-regexes-8.json, MIT): 537,806 unique REAL-WORLD
   regexes from 193,524 projects / 8 languages — the biggest confidence gain
   against the REJECTS-VALID blind spot; expecteds derived via the pcre2test
   oracle (never from corpus metadata), supportedLangs as a prior only.
   Recognized: ESEC/FSE'19; cited by .NET for test augmentation.
3. .NET RegexParserTests (dotnet/runtime, MIT): the only external suite keyed
   as TYPED PARSE-ERROR KINDS + offsets (Roslyn-ported bank). Shared-syntax
   subset with an explicit .NET-divergence filter.
4. test262 RegExp early-error files (negative:{phase:parse}) +
   property-escapes/generated (UCD-derived property coverage) — heavy
   ECMAScript dialect filter; take filtered slices only.
5. RE2 parse_test.cc (BSD-3): pattern -> expected AST dump on the
   backref/lookaround-free subset (the only AST-keyed external corpus).
6. (optional) CPython re_tests.py (~350 tuples with SYNTAX_ERROR markers) —
   same lineage as Perl's; cheap secondary.

## Rejects with cause (recorded)

- UTS#18: NO conformance corpus exists (spec's own statement) — requirements
  checklist only; coverage via PCRE2 test 26/27 + test262 generated.
- POSIX/OpenGroup (VSX-PCTS paywalled; glibc PTESTS/rxspencer), AT&T testregex,
  Go regexp testdata, Tcl ARE: wrong dialect (BRE/ERE, leftmost-longest);
  PCRE2's own POSIX-interface tests 18/19 already vendored.
- Oniguruma/Boost.Regex/ICU: divergent dialects, matcher-keyed, C/C++-embedded;
  PCRE2 testinput2 already carries the relevant intersection.
- Rust regex TOML suite: subset dialect, span-only keys — dominated.
- OSS-Fuzz pcre2 corpora: ACCESS-RESTRICTED + answer-key-free; keep the
  public pcre2_fuzzer.dict (already in vendored testdata) for in-house
  differential fuzzing.
- rebar (perf barometer; later interest for the speed doctrine, not parse
  signoff), regexlib (no key/license hygiene), browser JS suites (test262
  duplicates in the wrong dialect).

## Key primary sources

- pcre2 testdata + RunTest titles: github.com/PCRE2Project/pcre2 (testdata/, RunTest)
- Perl: github.com/Perl/perl5 t/re/re_tests + reg_mesg.t
- Lingua Franca: github.com/SBULeeLab/LinguaFranca-FSE19 + doi 10.1145/3338906.3338909
- .NET: github.com/dotnet/runtime System.Text.RegularExpressions/tests (corefx PR #29178)
- test262: github.com/tc39/test262 built-ins/RegExp + language/literals/regexp
- RE2: github.com/google/re2 re2/testing/parse_test.cc
- OSS-Fuzz policy: google.github.io/oss-fuzz/advanced-topics/corpora/

(Full agent report with the complete 18-candidate table preserved in the
session task output; this file is the durable roster-feeding extract.)
