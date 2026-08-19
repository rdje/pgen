# `rule_fire_probes/` — the minimal LRM constructs behind `ADJUDICATED_UNWITNESSED`

Each file is the evidence for one row of `ADJUDICATED_UNWITNESSED` in
`stimuli/sv/rule_fire_partition.py`: a rule the certificate pass could not witness, adjudicated by
handing it the smallest construct IEEE 1800 Annex A licenses and checking **its own rule fires** —
not merely that the file parses.

⛔ The arm check is the point. `SV-CORPUS-GRAD.13c.2q` measured a construct that parsed through the
WRONG production, invisible to every verdict-only oracle in the repository. Re-adjudicate with:

```bash
./rust/target/debug/parseability_probe --parse systemverilog \
    stimuli/sv/rule_fire_probes/<f>.sv --profile sv_2017 \
    --dump-rule-outcome-counts-json /tmp/o.json          # exit 0 = ACCEPT
python3 -c "import json;d=json.load(open('/tmp/o.json'));print({k:v for k,v in d['rule_entry_counts'].items() if 'kw_' in k})"
```

| file | rule adjudicated | IEEE 1800-2023 Annex A |
|---|---|---|
| `eventually_property_expr.sv` | `kw_eventually_5865020f` | A.2.10 `\| eventually [ constant_range ] property_expr` |
| `s_always_property_expr.sv` | `kw_s_always_0f9aa900` | A.2.10 `\| s_always [ constant_range ] property_expr` |
| `sync_accept_on_property_expr.sv` | `kw_sync_accept_on_b65eda53` | A.2.10 `\| sync_accept_on ( expression_or_dist ) property_expr` |
