# TEST_MODE: full_pipeline
# EXPECT: pass
TestRule::
 -> rule_b     {return call(rule_b)}
 -> rule_a     {return call(rule_a)}

# Rule B is referenced before it's defined
rule_b: /b/    I {
 return {type=>'RULE_B', content=>$IMATCH}
}

# Rule A is defined after being referenced
rule_a: /a/    I {
 return {type=>'RULE_A', content=>$IMATCH}
} 