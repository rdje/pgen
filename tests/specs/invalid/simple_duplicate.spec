# TEST_MODE: full_pipeline
# EXPECT: fail
TestRule::
 -> rule_a     {return call(rule_a)}

rule_a: /first/    I {
 return {type=>'FIRST', content=>$IMATCH}
}

rule_a: /second/    I {
 return {type=>'SECOND', content=>$IMATCH}
}
