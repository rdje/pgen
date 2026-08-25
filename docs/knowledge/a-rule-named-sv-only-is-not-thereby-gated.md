---
id: a-rule-named-sv-only-is-not-thereby-gated
title: A rule named for a dialect is not thereby gated to it — the name and the @profiles annotation are independent, and only one of them is enforced
answers:
  - "a rule is named _sv_2017 / _sv_only — is it actually restricted to that dialect"
  - "why does an SV-only construct still parse under my strict dialect profile"
  - "how do I audit which dialect-named rules are actually gated"
  - "I fixed one rule whose name promised a gate it did not have — is the class closed"
  - "what should a naming convention be allowed to carry"
tags: [profiles, dialects, grammar, naming, census, sv-corpus-grad, engine-universal]
date: 2026-08-25
status: current
evidence: "Measured in PGEN's SystemVerilog grammar. SV-CORPUS-GRAD.13e.7 (PGEN-SV-CORPUS-GRAD-0290..0293) found `data_declaration_sv_2017` — named for IEEE 1800 — carrying @profiles that ADMIT verilog_2005, and it was the door for 9 measured over-acceptances; the repair was 13 named _sv_only gates over 19 SITES INSIDE it, NOT removing the admission. SV-CORPUS-GRAD.13e.10(c4) sizing (PGEN-SV-CORPUS-GRAD-0303) then found `.*` reaching verilog_2005 through three more dialect-named-but-admitted hosts (module_declaration_sv_2017, named_port_connection_sv_2017, udp_declaration_sv_2017), while the other four dot_star hosts are correctly [\"sv_2017\"] only. THE CLASS CENSUS, run in that leaf: 27 rules whose name carries a dialect suffix are @profiles-admitted to verilog_2005. ⛔ THAT COUNT IS NOT A DEFECT COUNT AND MUST NOT BE QUOTED AS ONE — admission usually means the rule genuinely SERVES both profiles despite its name (module_declaration and specify-path descriptions exist in IEEE 1364-2005 too). What the mismatch marks is a SITE WHERE AN SV-ONLY ELEMENT CAN LEAK, which is exactly what .13e.7 and .13e.10 each found. 2 of the 27 have been audited; 25 have not."
reverify: "python3 - <<'PY'\nimport re,pathlib\nlines=pathlib.Path('grammars/systemverilog.ebnf').read_text().splitlines()\nfor i,l in enumerate(lines):\n    m=re.match(r'^([a-zA-Z_][a-zA-Z0-9_]*(?:_sv_2017|_sv_2023|_sv_only))\\s*:=',l)\n    if not m: continue\n    j,prof=i-1,None\n    while j>=0 and (not lines[j].strip() or lines[j].lstrip().startswith(('#','@'))):\n        p=re.match(r'^@profiles:\\s*(\\[.*\\])',lines[j].strip())\n        if p: prof=p.group(1); break\n        j-=1\n    if prof and 'verilog_2005' in prof: print(f'MISMATCH L{i+1} {m.group(1)} -> {prof}')\nPY"
---

# A rule named for a dialect is not thereby gated to it

**Question it answers:** the rule is called `foo_sv_2017`. Is it restricted to the SV profiles?

**Answer:** not unless its `@profiles` annotation says so, and the two are independent. The name is a
comment. Only the annotation is enforced.

## Measured, twice, in the same grammar

| when | rule | name promises | `@profiles` actually says | what it cost |
|---|---|---|---|---|
| `SV-CORPUS-GRAD.13e.7` | `data_declaration_sv_2017` | IEEE 1800 only | admits `verilog_2005` | the door for **9** measured over-acceptances |
| `SV-CORPUS-GRAD.13e.10`(c4) | `module_declaration_sv_2017` | IEEE 1800 only | admits `verilog_2005` | `.*` reaches the strict profile |
| | `named_port_connection_sv_2017` | IEEE 1800 only | admits `verilog_2005` | ″ |
| | `udp_declaration_sv_2017` | IEEE 1800 only | admits `verilog_2005` | ″ |

The other four `dot_star` hosts are correctly `["sv_2017"]`. **Fixing the first instance did not
shrink the class** — `.13e.7` diagnosed the mismatch precisely and closed its own door with 13 named
gates, while rules with the identical shape sat untouched in the same file, because what was fixed
was an *instance* and what existed was a *category*
([[feedback_a_named_call_site_is_a_category_of_call_sites]]).

## ⛔⛔ The class census — and what its number is NOT

**27** rules whose name carries a dialect suffix are `@profiles`-admitted to `verilog_2005`
(command in `reverify`).

⛔ **That is not a defect count and must never be quoted as one.** Admission usually means the rule
genuinely **serves both profiles despite its name** — `module_declaration` and the specify-path
descriptions exist in IEEE 1364-2005 too; the suffix records *where the production text was
extracted from*, not *which dialect may use it*. Removing the admission would break legal Verilog.

⭐ **What the mismatch actually marks is a SITE WHERE AN SV-ONLY ELEMENT CAN LEAK** — a rule the strict
profile enters, whose body was authored against the permissive standard. That is precisely what both
measurements found, and it is why the repair is a **per-element** gate inside the rule, never a
profile change on the rule. **2 of the 27 have been audited. 25 have not.**

## Why it survives

## Why it survives

- **Nothing censuses it.** No lint, no gate, no doctrine compares a rule's dialect suffix against its
  `@profiles`. The mismatch is invisible to every check the repository runs.
- **It fails in the accepting direction.** A rule that is *more* admitted than its name suggests
  produces extra accepts, never a rejected-valid — so a positive-only corpus stays green.
- **The name is load-bearing for humans and inert for the engine**, which is the worst combination: a
  reader auditing "is this SV-only?" reads the name, agrees, and moves on.

## The rule of thumb

**Let a naming convention carry intent, never enforcement.** The name cannot gate anything, so when a
dialect-named rule is admitted to another dialect, treat it as a **flag to audit the body**, not as a
bug to close. The census is one command; the work it schedules is reading 25 rule bodies for SV-only
elements, which is what actually finds the leaks.

⚠️ And be careful how you close it: gating the terminal is not always the right repair, because
gating a rule also silently inverts every negative lookahead on it →
[[a-profile-gate-inverts-every-negative-lookahead-on-the-gated-rule]].
