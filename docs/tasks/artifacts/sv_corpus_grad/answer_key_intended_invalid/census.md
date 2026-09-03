# Suite-intended-invalid `must_accept` census — SV-CORPUS-GRAD.13x

> DERIVED. Re-run: `python3 docs/tasks/artifacts/sv_corpus_grad/answer_key_intended_invalid/census.py`
>
> ⛔ **A MEASUREMENT, NOT A RULING.** Every row's `expected` / `adjudication` is read back from the
> tracked manifest; the rulings live in `stimuli/sv/adjudicate_external_corpus.py`'s pin tables and
> move only on an LRM citation (`.13k`'s discipline). This table shows the population before and
> after each ruling slice.

**Population: 10 rows** = `_bad|/error/|should_fail|_fail` on a `divergence:unexplained_rejects_valid` + `must_accept` row (10) + named EXTRA (0).

| suite | rows |
|---|---:|
| `verilator` | 7 |
| `sv2v` | 2 |
| `iverilog` | 1 |

| adjudication (manifest, as of this run) | rows |
|---|---:|
| `divergence:unexplained_rejects_valid` | 10 |

Tracked `positions.tsv` agrees with the live probe on **10 of 10** rows.
Edition contrast (`sv_2017` fail, `sv_2023` pass): **2** rows — `test_regress/t/t_class_override_bad.v`, `test_regress/t/t_param_type_fwd_bad.v`

## Rows

| # | suite | relpath | line:col | sv_2017 | sv_2023 | source at furthest | upstream testimony | expected | adjudication |
|---:|---|---|---|---|---|---|---|---|---|
| 1 | `iverilog` | `ivtest/ivltests/sv_class_new_typed_fail4.v` | 13:11 | fail@207 | fail@207 | `c = T::new; // This should fail, T is not a class` |  | `must_accept` | `divergence:unexplained_rejects_valid` |
| 2 | `sv2v` | `test/error/typedef_not_type_net.sv` | 5:7 | fail@132 | fail@132 | `w x;` | expected typename, but found net identifier "w" | `must_accept` | `divergence:unexplained_rejects_valid` |
| 3 | `sv2v` | `test/error/typedef_not_type_var.sv` | 5:7 | fail@131 | fail@131 | `v x;` | expected typename, but found var identifier "v" | `must_accept` | `divergence:unexplained_rejects_valid` |
| 4 | `verilator` | `test_regress/t/t_assign_automatic_bad.v` | 60:5 | fail@2126 | fail@2126 | `].s_dyn <= '1;  // OK: direct nonblocking assignment to dynamically-si` | %Error: t/t_assign_automatic_bad.v:36:10: Dynamically-sized variable not allowed in continuous assignment (IEE | `must_accept` | `divergence:unexplained_rejects_valid` |
| 5 | `verilator` | `test_regress/t/t_class_extends_pkg_bad.v` | 22:23 | fail@413 | fail@413 | `class bar extends pkg;` | %Error-UNSUPPORTED: t/t_class_extends_pkg_bad.v:22:21: Unsupported: PACKAGE 'pkg' in 'class extends' | `must_accept` | `divergence:unexplained_rejects_valid` |
| 6 | `verilator` | `test_regress/t/t_class_override_bad.v` | 23:10 | fail@819 | pass@-1 | `function :extends int get_e; return 1; endfunction  // Bad` | %Error: t/t_class_override_bad.v:23:25: Member 'get_e' marked ':extends' but no base class function is being e | `must_accept` | `divergence:unexplained_rejects_valid` |
| 7 | `verilator` | `test_regress/t/t_implements_noninterface_bad.v` | 10:32 | fail@260 | fail@260 | `class ClsBad1 implements NotIcls;` | %Error: t/t_implements_noninterface_bad.v:10:26: Attempting to implement from non-interface class 'NotIcls' | `must_accept` | `divergence:unexplained_rejects_valid` |
| 8 | `verilator` | `test_regress/t/t_implements_notfound_bad.v` | 7:31 | fail@234 | fail@234 | `class ClsI implements Inotfound;` | %Error: t/t_implements_notfound_bad.v:7:23: Class for 'implements' not found: 'Inotfound' | `must_accept` | `divergence:unexplained_rejects_valid` |
| 9 | `verilator` | `test_regress/t/t_param_type_fwd_bad.v` | 10:16 | fail@251 | pass@-1 | `parameter type enum E_t;` | %Error: t/t_param_type_fwd_bad.v:25:10: Parameter type expression type 'int' violates parameter's forwarding t | `must_accept` | `divergence:unexplained_rejects_valid` |
| 10 | `verilator` | `test_regress/t/t_recursive_typedef_bad.v` | 12:24 | fail@422 | fail@422 | `parameter type A = B,` | %Error: t/t_recursive_typedef_bad.v:12:20: Recursive type definition | `must_accept` | `divergence:unexplained_rejects_valid` |

## Instrument identity

Re-hash these inputs; if any hash differs from the row below, **this report no longer
describes your tree** and the honest act is to re-measure, not to quote.

| input | sha256 |
| --- | --- |
| `stimuli/sv/characterization/adjudication_manifest.tsv` | `e6fa8d0b21f5614b639b7bd90136ad1f63b632f551660d6cda63d3cd3874db45` |
| `stimuli/sv/characterization/positions.tsv` | `75dd16941ff3de8113bd0f9144031e5fa8e00980384a6c4c2633636af199538d` |
| `docs/tasks/artifacts/sv_corpus_grad/answer_key_intended_invalid/census.py` | `23f68af1ded2b07a8137cbd94a445d776b6e0ecf739494a07701a53d0330e48b` |
| `rust/target/release/parseability_probe` (parse binary) | `5f670862bad6f80de1f10402ea7c85be80e23a362f73e472cf2abbe76a6c6bac` |

