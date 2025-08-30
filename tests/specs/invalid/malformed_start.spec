# TEST_MODE: parse_only
# EXPECT: fail
# This file doesn't start with a rule definition
some_invalid_content

TestRule::
 -> test_rule     {return call(test_rule)}

test_rule: /pattern/    I {
 return {type=>'TEST', content=>$IMATCH}
} 