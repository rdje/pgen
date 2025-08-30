# TEST_MODE: full_pipeline
# EXPECT: fail
TestRule::
 -> test_rule     {return call(test_rule)}

test_rule: /pattern/    I {
 return {type=>'TEST', content=>$IMATCH}
}

# Duplicate rule definition
test_rule: /another/    I {
 return {type=>'DUPLICATE', content=>$IMATCH}
} 