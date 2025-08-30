# TEST_MODE: parse_only
# EXPECT: fail
TestRule::
 -> test_rule     {return call(test_rule)}

test_rule: /invalid[regex/    I {
 return {type=>'TEST', content=>$IMATCH}
} 