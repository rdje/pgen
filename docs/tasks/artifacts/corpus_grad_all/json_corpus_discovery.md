# CORPUS-GRAD-ALL.1 — json family: exhaustive external-corpus discovery (2026-07-22)

Research-grounded discovery (web agent, primary sources; ~44 searches/fetches;
counts API-verified where stated). Baseline: json_corpus_bundle vendors
nst/JSONTestSuite test_parsing (318 y_/n_/i_ files, MIT).

## Headline findings

1. ⭐ **The vendored 318 files are NOT all of JSONTestSuite** — upstream has a
   second data dir `test_transform` (22 files, API-verified): parse-RESULT
   divergence cases (huge numbers, duplicate keys, NUL, escaped invalid
   strings). No strict y_/n_ key — an i_-class divergence catalog
   (seriot.ch/json/transform.html). The missing half of the Minefield
   companion materials.
2. **JSON_checker (json.org) is license-encumbered** — the raw zip (36 files,
   zip-verified) carries the non-free "Good, not Evil" JSON License (per
   RapidJSON's license.txt), and 2 cases are RFC-wrong (fail1 top-level
   scalar — legal since RFC 7159; fail18 depth — unspecified). Adopt via
   nativejson-benchmark's MIT curation (fail01/fail18 pre-excluded) or
   briandfoy/json-acceptance-tests (Artistic-2.0).
3. **No second corpus of JSONTestSuite's stature exists** — it IS the field
   standard; the real increments are (a) test_transform, (b) value-fidelity
   oracles, (c) dialect-rejection oracles.

## Ranked ADD recommendations

1. nativejson-benchmark conformance pack (miloyip, MIT): 27 roundtrip files
   (parse+reserialize identity) + 66 parse-double + 9 parse-string exact-value
   cases (embedded in src/main.cpp — extraction step) + the RFC-corrected
   JSON_checker curation. Extends signoff BEYOND accept/reject into value
   fidelity.
2. JSONTestSuite test_transform (22 files, same upstream/license): treat as
   i_-class — record + freeze PGEN's chosen behavior per file.
3. JSON5 test suite AS A REJECT ORACLE (json5/json5-tests, MIT, 114 files
   API-verified): 88 must-REJECT for an RFC parser (57 .json5 + 6 .js +
   25 .txt — comments, single quotes, unquoted keys, hex, NaN/Inf, trailing
   commas) + 26 must-accept .json. Exactly the over-permissiveness oracle.
   Lock each expected via the spec (one-time verification pass).
4. RFC 8259 §13 / ECMA-404 in-spec examples (~5 texts): trivial; earns the
   "parses every example in the spec itself" signoff sentence.
5. jansson suites (MIT; 36 valid + 51 invalid + 19 invalid-unicode dirs,
   API-verified): directory-classified accept/reject; the invalid-unicode 19
   are the differentiated part; per-case output/error texts are
   jansson-specific (ignore).
6. JSON_checker — ONLY via the nativejson-benchmark curation (license + the
   two RFC corrections).

Conditional/marginal: YAJL cases (token-stream oracle, filter extension
flags); simdjson-data jsonexamples (must-accept pool; license ambiguity);
parse-number-fxx (only if owning decimal->double; sample, don't vendor);
BLNS/Kuhn-UTF-8 (garnish; JSONTestSuite already covers the decisive classes).

## Rejects with cause

- JSON Schema Test Suite: validates SCHEMA semantics over already-parsed
  JSON — wrong layer for parser signoff.
- json-patch-tests (RFC 6902 semantics); leadpony/jsonp (Java API-shaped);
  Boost.JSON / System.Text.Json / serde inline tests (no extractable
  classified corpus; their conformance data IS JSONTestSuite); ruby/json
  fixtures (verified duplicate); nlohmann/json_test_data (aggregate of
  duplicates); test262 JSON.parse (77 files, JS-harness-bound, mostly
  reviver/API semantics); OSS-Fuzz/go-fuzz corpora (no answer key —
  robustness pools); BishopFox labs (quirk taxonomy, not corpus).

## Key primary sources

github.com/nst/JSONTestSuite (+ seriot.ch minefield/transform pages);
json.org/JSON_checker; raw.githubusercontent.com/Tencent/rapidjson/master/license.txt;
github.com/miloyip/nativejson-benchmark; github.com/json5/json5-tests;
github.com/akheron/jansson test/suites; rfc-editor.org/rfc/rfc8259;
github.com/tc39/test262 built-ins/JSON/parse; github.com/lloyd/yajl.

(Full agent report with the 24-candidate table preserved in the session task
output; this file is the durable roster-feeding extract.)
