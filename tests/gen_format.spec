# TEST_MODE: parse_only
# EXPECT: pass

gen_file::
-> rule_definition
-> comment
-> whitespace
-> gen_file

rule_definition: /(\w+)\s*->/
-> production_line
-> rule_definition

production_line: /\s*\|?\s*([^|\[\n\r]+)/
-> attributes
-> production_line

attributes: /\[([^\]]+)\]/
-> rule_definition

whitespace: /\s+/
-> gen_file

comment: /#.*\n/
-> gen_file
