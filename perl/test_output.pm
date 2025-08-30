🚀 Generating Perl parser from: /Users/richarddje/Documents/github/airefactored/legacy/grammars/merged_ultimate_return_annotation.ebnf

=== Step 2: Group by OR ===

🔍 STEP 2 DEBUG: Looking for dot_path rule in input...
🎯 STEP 2: Found dot_path rule in input: [rule,dot_path], [rule_reference,accessor], [operator,+], [return_array,[$1*]]
✅ STEP 2: dot_path rule found in output: $VAR1 = {
          'name' => 'dot_path',
          'or_groups' => [
                           [
                             [
                               'rule_reference',
                               'accessor'
                             ],
                             [
                               'operator',
                               '+'
                             ],
                             [
                               'return_array',
                               '[$1*]'
                             ]
                           ]
                         ]
        };

STEP 2 RESULT (OR groups):
$VAR1 = [
          {
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '->'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'return_expression'
                               ]
                             ]
                           ],
            'name' => 'return_annotation'
          },
          {
            'name' => 'return_expression',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'nested_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'nested_object'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'grouped_quantified_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'simple_nested_object'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'multi_property_object'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'quantified_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'simple_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'simple_object'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'ultimate_dot_notation'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'scalar_ref'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'literal'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '['
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'array_contents'
                               ],
                               [
                                 'operator',
                                 '?'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ']'
                               ],
                               [
                                 'rule_reference',
                                 'quantifier'
                               ],
                               [
                                 'operator',
                                 '?'
                               ],
                               [
                                 'return_object',
                                 '{type: "array", contents: $3, quantified: $6}'
                               ]
                             ]
                           ],
            'name' => 'nested_array'
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'return_expression'
                               ],
                               [
                                 'group_open',
                                 '('
                               ],
                               [
                                 'quoted_string',
                                 ','
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'return_expression'
                               ],
                               [
                                 'group_close',
                                 ')'
                               ],
                               [
                                 'operator',
                                 '*'
                               ],
                               [
                                 'return_array',
                                 '[$1, $3*]'
                               ]
                             ]
                           ],
            'name' => 'array_contents'
          },
          {
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '{'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'object_contents'
                               ],
                               [
                                 'operator',
                                 '?'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 '}'
                               ],
                               [
                                 'rule_reference',
                                 'quantifier'
                               ],
                               [
                                 'operator',
                                 '?'
                               ],
                               [
                                 'return_object',
                                 '{type: "object", contents: $3, quantified: $6}'
                               ]
                             ]
                           ],
            'name' => 'nested_object'
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'object_pair'
                               ],
                               [
                                 'group_open',
                                 '('
                               ],
                               [
                                 'quoted_string',
                                 ','
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'object_pair'
                               ],
                               [
                                 'group_close',
                                 ')'
                               ],
                               [
                                 'operator',
                                 '*'
                               ],
                               [
                                 'return_array',
                                 '[$1, $3*]'
                               ]
                             ]
                           ],
            'name' => 'object_contents'
          },
          {
            'name' => 'object_pair',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'object_key'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ':'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'return_expression'
                               ],
                               [
                                 'return_object',
                                 '{key: $1, value: $5}'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'ultimate_dot_notation',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'scalar_ref'
                               ],
                               [
                                 'rule_reference',
                                 'dot_path'
                               ],
                               [
                                 'return_object',
                                 '{type: "ultimate_dot_notation", base: $1, path: $2}'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'dot_path',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'accessor'
                               ],
                               [
                                 'operator',
                                 '+'
                               ],
                               [
                                 'return_array',
                                 '[$1*]'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'accessor',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'property_accessor'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'positional_accessor'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'array_accessor'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'property_accessor',
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '.'
                               ],
                               [
                                 'rule_reference',
                                 'identifier'
                               ],
                               [
                                 'return_object',
                                 '{type: "property", name: $2}'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'positional_accessor',
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '.'
                               ],
                               [
                                 'rule_reference',
                                 'positive_number'
                               ],
                               [
                                 'return_object',
                                 '{type: "position", index: $2}'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'array_accessor',
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '['
                               ],
                               [
                                 'rule_reference',
                                 'array_spec'
                               ],
                               [
                                 'quoted_string',
                                 ']'
                               ],
                               [
                                 'return_object',
                                 '{type: "array_access", spec: $2}'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'array_spec',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'empty_spec'
                               ],
                               [
                                 'return_object',
                                 '{type: "whole_array", style: "implicit"}'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'star_spec'
                               ],
                               [
                                 'return_object',
                                 '{type: "whole_array", style: "bash"}'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'colon_spec'
                               ],
                               [
                                 'return_object',
                                 '{type: "whole_array", style: "python"}'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'single_index'
                               ],
                               [
                                 'return_scalar',
                                 '$1'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'perl_range'
                               ],
                               [
                                 'return_scalar',
                                 '$1'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'python_slice'
                               ],
                               [
                                 'return_scalar',
                                 '$1'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'python_slice_with_step'
                               ],
                               [
                                 'return_scalar',
                                 '$1'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'index_list'
                               ],
                               [
                                 'return_object',
                                 '{type: "multi_index", indices: $1}'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'mixed_expression'
                               ],
                               [
                                 'return_object',
                                 '{type: "mixed_expression", elements: $1}'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'regex',
                                 '(?=\\])'
                               ]
                             ]
                           ],
            'name' => 'empty_spec'
          },
          {
            'name' => 'star_spec',
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '*'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 ':'
                               ]
                             ]
                           ],
            'name' => 'colon_spec'
          },
          {
            'name' => 'single_index',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'index'
                               ],
                               [
                                 'return_object',
                                 '{type: "single_index", value: $1}'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'perl_range',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'index'
                               ],
                               [
                                 'quoted_string',
                                 '..'
                               ],
                               [
                                 'rule_reference',
                                 'index'
                               ],
                               [
                                 'return_object',
                                 '{type: "perl_range", start: $1, end: $3}'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'python_slice_start'
                               ],
                               [
                                 'quoted_string',
                                 ':'
                               ],
                               [
                                 'rule_reference',
                                 'python_slice_end'
                               ],
                               [
                                 'return_object',
                                 '{type: "python_slice", start: $1, end: $3}'
                               ]
                             ]
                           ],
            'name' => 'python_slice'
          },
          {
            'name' => 'python_slice_with_step',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'python_slice_start'
                               ],
                               [
                                 'quoted_string',
                                 ':'
                               ],
                               [
                                 'rule_reference',
                                 'python_slice_end'
                               ],
                               [
                                 'quoted_string',
                                 ':'
                               ],
                               [
                                 'rule_reference',
                                 'step'
                               ],
                               [
                                 'return_object',
                                 '{type: "python_slice_step", start: $1, end: $3, step: $5}'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'python_slice_start',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'index'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'empty_slice_part'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'index'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'empty_slice_part'
                               ]
                             ]
                           ],
            'name' => 'python_slice_end'
          },
          {
            'name' => 'step',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'index'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'empty_slice_part',
            'or_groups' => [
                             [
                               [
                                 'regex',
                                 '(?=:)'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'index'
                               ],
                               [
                                 'group_open',
                                 '('
                               ],
                               [
                                 'quoted_string',
                                 ','
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'index'
                               ],
                               [
                                 'group_close',
                                 ')'
                               ],
                               [
                                 'operator',
                                 '*'
                               ],
                               [
                                 'return_array',
                                 '[$1, $3*]'
                               ]
                             ]
                           ],
            'name' => 'index_list'
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'mixed_element'
                               ],
                               [
                                 'group_open',
                                 '('
                               ],
                               [
                                 'quoted_string',
                                 ','
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'mixed_element'
                               ],
                               [
                                 'group_close',
                                 ')'
                               ],
                               [
                                 'operator',
                                 '*'
                               ],
                               [
                                 'return_array',
                                 '[$1, $3*]'
                               ]
                             ]
                           ],
            'name' => 'mixed_expression'
          },
          {
            'name' => 'mixed_element',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'single_index'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'perl_range'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'python_slice'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'python_slice_with_step'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'positive_number'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'negative_number'
                               ]
                             ]
                           ],
            'name' => 'index'
          },
          {
            'name' => 'positive_number',
            'or_groups' => [
                             [
                               [
                                 'regex',
                                 '(\\d+)'
                               ],
                               [
                                 'return_object',
                                 '{type: "positive", value: $1}'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '-'
                               ],
                               [
                                 'rule_reference',
                                 'positive_number'
                               ],
                               [
                                 'return_object',
                                 '{type: "negative", value: $2}'
                               ]
                             ]
                           ],
            'name' => 'negative_number'
          },
          {
            'name' => 'grouped_quantified_array',
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '['
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'grouped_element_list'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ']'
                               ],
                               [
                                 'return_object',
                                 '{type: "grouped_quantified_array", groups: $3}'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'grouped_element_item'
                               ],
                               [
                                 'group_open',
                                 '('
                               ],
                               [
                                 'quoted_string',
                                 ','
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'grouped_element_item'
                               ],
                               [
                                 'group_close',
                                 ')'
                               ],
                               [
                                 'operator',
                                 '*'
                               ],
                               [
                                 'return_array',
                                 '[$1, $3*]'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'grouped_element_item'
                               ],
                               [
                                 'return_array',
                                 '[$1]'
                               ]
                             ]
                           ],
            'name' => 'grouped_element_list'
          },
          {
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '('
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'group_content'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ')'
                               ],
                               [
                                 'rule_reference',
                                 'quantifier'
                               ],
                               [
                                 'return_object',
                                 '{type: "grouped_quantified", content: $3, quantifier: $6}'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'element_sequence'
                               ],
                               [
                                 'rule_reference',
                                 'quantifier'
                               ],
                               [
                                 'return_object',
                                 '{type: "sequence_quantified", content: $1, quantifier: $2}'
                               ]
                             ]
                           ],
            'name' => 'grouped_element_item'
          },
          {
            'name' => 'group_content',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'element_sequence'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'element_sequence',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'element_item'
                               ],
                               [
                                 'group_open',
                                 '('
                               ],
                               [
                                 'regex',
                                 '\\s+'
                               ],
                               [
                                 'rule_reference',
                                 'element_item'
                               ],
                               [
                                 'group_close',
                                 ')'
                               ],
                               [
                                 'operator',
                                 '*'
                               ],
                               [
                                 'return_array',
                                 '[$1, $3*]'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'element_item'
                               ],
                               [
                                 'return_array',
                                 '[$1]'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'identifier'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'literal'
                               ]
                             ]
                           ],
            'name' => 'element_item'
          },
          {
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '{'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'outer_key'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ':'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'inner_object'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 '}'
                               ],
                               [
                                 'return_object',
                                 '{type: "nested_object", key: $3, value: $7}'
                               ]
                             ]
                           ],
            'name' => 'simple_nested_object'
          },
          {
            'name' => 'inner_object',
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '{'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'inner_key'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ':'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'inner_value'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 '}'
                               ],
                               [
                                 'return_object',
                                 '{type: "inner_object", key: $3, value: $7}'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'nested_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'nested_object'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'grouped_quantified_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'quantified_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'simple_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'ultimate_dot_notation'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'scalar_ref'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'literal'
                               ]
                             ]
                           ],
            'name' => 'inner_value'
          },
          {
            'name' => 'multi_property_object',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'two_property_object'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'three_property_object'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '{'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'property'
                               ],
                               [
                                 'quoted_string',
                                 ','
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'property'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 '}'
                               ],
                               [
                                 'return_object',
                                 '{type: "multi_object", prop1: $3, prop2: $6}'
                               ]
                             ]
                           ],
            'name' => 'two_property_object'
          },
          {
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '{'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'property'
                               ],
                               [
                                 'quoted_string',
                                 ','
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'property'
                               ],
                               [
                                 'quoted_string',
                                 ','
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'property'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 '}'
                               ],
                               [
                                 'return_object',
                                 '{type: "multi_object", prop1: $3, prop2: $6, prop3: $9}'
                               ]
                             ]
                           ],
            'name' => 'three_property_object'
          },
          {
            'name' => 'property',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'object_key'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ':'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'property_value'
                               ],
                               [
                                 'return_object',
                                 '{key: $1, value: $5}'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'property_value',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'nested_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'nested_object'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'grouped_quantified_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'quantified_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'simple_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'ultimate_dot_notation'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'scalar_ref'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'literal'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '['
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'quantified_element'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ']'
                               ],
                               [
                                 'return_object',
                                 '{type: "quantified_array", element: $3}'
                               ]
                             ]
                           ],
            'name' => 'quantified_array'
          },
          {
            'name' => 'quantified_element',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'ultimate_dot_notation'
                               ],
                               [
                                 'rule_reference',
                                 'quantifier'
                               ],
                               [
                                 'return_object',
                                 '{scalar: $1, quantifier: $2}'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'scalar_ref'
                               ],
                               [
                                 'rule_reference',
                                 'quantifier'
                               ],
                               [
                                 'return_object',
                                 '{scalar: $1, quantifier: $2}'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '*'
                               ],
                               [
                                 'return_scalar',
                                 '"*"'
                               ]
                             ],
                             [
                               [
                                 'quoted_string',
                                 '+'
                               ],
                               [
                                 'return_scalar',
                                 '"+"'
                               ]
                             ],
                             [
                               [
                                 'quoted_string',
                                 '?'
                               ],
                               [
                                 'return_scalar',
                                 '"?"'
                               ]
                             ],
                             [
                               [
                                 'quoted_string',
                                 '{'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'positive_number'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 '}'
                               ],
                               [
                                 'return_object',
                                 '{min: $3, max: $3}'
                               ]
                             ],
                             [
                               [
                                 'quoted_string',
                                 '{'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'positive_number'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ','
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 '}'
                               ],
                               [
                                 'return_object',
                                 '{min: $3, max: "inf"}'
                               ]
                             ],
                             [
                               [
                                 'quoted_string',
                                 '{'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'positive_number'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ','
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'positive_number'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 '}'
                               ],
                               [
                                 'return_object',
                                 '{min: $3, max: $7}'
                               ]
                             ],
                             [
                               [
                                 'quoted_string',
                                 '{'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ','
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'positive_number'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 '}'
                               ],
                               [
                                 'return_object',
                                 '{min: 0, max: $5}'
                               ]
                             ]
                           ],
            'name' => 'quantifier'
          },
          {
            'name' => 'simple_array',
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '['
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'array_element'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ']'
                               ],
                               [
                                 'return_object',
                                 '{type: "array", element: $3}'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'array_element',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'ultimate_dot_notation'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'scalar_ref'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'literal'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'nested_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'nested_object'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'simple_object',
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '{'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'object_key'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ':'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'object_value'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 '}'
                               ],
                               [
                                 'return_object',
                                 '{type: "object", key: $3, value: $7}'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'object_key',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'identifier'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'quoted_string'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'identifier'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'quoted_string'
                               ]
                             ]
                           ],
            'name' => 'outer_key'
          },
          {
            'name' => 'inner_key',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'identifier'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'quoted_string'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'object_value',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'ultimate_dot_notation'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'scalar_ref'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'literal'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'nested_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'nested_object'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'quoted_string'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'number'
                               ]
                             ]
                           ],
            'name' => 'literal'
          },
          {
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '$'
                               ],
                               [
                                 'rule_reference',
                                 'positive_number'
                               ],
                               [
                                 'return_object',
                                 '{type: "scalar_ref", index: $2}'
                               ]
                             ]
                           ],
            'name' => 'scalar_ref'
          },
          {
            'or_groups' => [
                             [
                               [
                                 'regex',
                                 '"([^"]*)"'
                               ],
                               [
                                 'return_scalar',
                                 '$1'
                               ]
                             ]
                           ],
            'name' => 'quoted_string'
          },
          {
            'or_groups' => [
                             [
                               [
                                 'regex',
                                 '(\\d+)'
                               ],
                               [
                                 'return_scalar',
                                 '$1'
                               ]
                             ]
                           ],
            'name' => 'number'
          },
          {
            'name' => 'identifier',
            'or_groups' => [
                             [
                               [
                                 'regex',
                                 '([a-zA-Z_]\\w*)'
                               ],
                               [
                                 'return_scalar',
                                 '$1'
                               ]
                             ]
                           ]
          }
        ];

=== Step 2.5: Handle parentheses ===

🔍 STEP 2.5 DEBUG: Looking for dot_path rule in input...
🎯 STEP 2.5: Found dot_path rule in input: $VAR1 = {
          'name' => 'dot_path',
          'or_groups' => [
                           [
                             [
                               'rule_reference',
                               'accessor'
                             ],
                             [
                               'operator',
                               '+'
                             ],
                             [
                               'return_array',
                               '[$1*]'
                             ]
                           ]
                         ]
        };

✅ STEP 2.5: dot_path rule found in output: $VAR1 = {
          'name' => 'dot_path',
          'or_groups' => [
                           [
                             [
                               'rule_reference',
                               'accessor'
                             ],
                             [
                               'operator',
                               '+'
                             ],
                             [
                               'return_array',
                               '[$1*]'
                             ]
                           ]
                         ]
        };

STEP 2.5 RESULT (Parentheses handled):
$VAR1 = [
          {
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '->'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'return_expression'
                               ]
                             ]
                           ],
            'name' => 'return_annotation'
          },
          {
            'name' => 'return_expression',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'nested_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'nested_object'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'grouped_quantified_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'simple_nested_object'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'multi_property_object'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'quantified_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'simple_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'simple_object'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'ultimate_dot_notation'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'scalar_ref'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'literal'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '['
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'array_contents'
                               ],
                               [
                                 'operator',
                                 '?'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ']'
                               ],
                               [
                                 'rule_reference',
                                 'quantifier'
                               ],
                               [
                                 'operator',
                                 '?'
                               ],
                               [
                                 'return_object',
                                 '{type: "array", contents: $3, quantified: $6}'
                               ]
                             ]
                           ],
            'name' => 'nested_array'
          },
          {
            'name' => 'array_contents',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'return_expression'
                               ],
                               [
                                 'GROUPED',
                                 [
                                   [
                                     'quoted_string',
                                     ','
                                   ],
                                   [
                                     'regex',
                                     '\\s*'
                                   ],
                                   [
                                     'rule_reference',
                                     'return_expression'
                                   ]
                                 ]
                               ],
                               [
                                 'operator',
                                 '*'
                               ],
                               [
                                 'return_array',
                                 '[$1, $3*]'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '{'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'object_contents'
                               ],
                               [
                                 'operator',
                                 '?'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 '}'
                               ],
                               [
                                 'rule_reference',
                                 'quantifier'
                               ],
                               [
                                 'operator',
                                 '?'
                               ],
                               [
                                 'return_object',
                                 '{type: "object", contents: $3, quantified: $6}'
                               ]
                             ]
                           ],
            'name' => 'nested_object'
          },
          {
            'name' => 'object_contents',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'object_pair'
                               ],
                               [
                                 'GROUPED',
                                 [
                                   [
                                     'quoted_string',
                                     ','
                                   ],
                                   [
                                     'regex',
                                     '\\s*'
                                   ],
                                   [
                                     'rule_reference',
                                     'object_pair'
                                   ]
                                 ]
                               ],
                               [
                                 'operator',
                                 '*'
                               ],
                               [
                                 'return_array',
                                 '[$1, $3*]'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'object_key'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ':'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'return_expression'
                               ],
                               [
                                 'return_object',
                                 '{key: $1, value: $5}'
                               ]
                             ]
                           ],
            'name' => 'object_pair'
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'scalar_ref'
                               ],
                               [
                                 'rule_reference',
                                 'dot_path'
                               ],
                               [
                                 'return_object',
                                 '{type: "ultimate_dot_notation", base: $1, path: $2}'
                               ]
                             ]
                           ],
            'name' => 'ultimate_dot_notation'
          },
          {
            'name' => 'dot_path',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'accessor'
                               ],
                               [
                                 'operator',
                                 '+'
                               ],
                               [
                                 'return_array',
                                 '[$1*]'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'property_accessor'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'positional_accessor'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'array_accessor'
                               ]
                             ]
                           ],
            'name' => 'accessor'
          },
          {
            'name' => 'property_accessor',
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '.'
                               ],
                               [
                                 'rule_reference',
                                 'identifier'
                               ],
                               [
                                 'return_object',
                                 '{type: "property", name: $2}'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '.'
                               ],
                               [
                                 'rule_reference',
                                 'positive_number'
                               ],
                               [
                                 'return_object',
                                 '{type: "position", index: $2}'
                               ]
                             ]
                           ],
            'name' => 'positional_accessor'
          },
          {
            'name' => 'array_accessor',
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '['
                               ],
                               [
                                 'rule_reference',
                                 'array_spec'
                               ],
                               [
                                 'quoted_string',
                                 ']'
                               ],
                               [
                                 'return_object',
                                 '{type: "array_access", spec: $2}'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'empty_spec'
                               ],
                               [
                                 'return_object',
                                 '{type: "whole_array", style: "implicit"}'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'star_spec'
                               ],
                               [
                                 'return_object',
                                 '{type: "whole_array", style: "bash"}'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'colon_spec'
                               ],
                               [
                                 'return_object',
                                 '{type: "whole_array", style: "python"}'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'single_index'
                               ],
                               [
                                 'return_scalar',
                                 '$1'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'perl_range'
                               ],
                               [
                                 'return_scalar',
                                 '$1'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'python_slice'
                               ],
                               [
                                 'return_scalar',
                                 '$1'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'python_slice_with_step'
                               ],
                               [
                                 'return_scalar',
                                 '$1'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'index_list'
                               ],
                               [
                                 'return_object',
                                 '{type: "multi_index", indices: $1}'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'mixed_expression'
                               ],
                               [
                                 'return_object',
                                 '{type: "mixed_expression", elements: $1}'
                               ]
                             ]
                           ],
            'name' => 'array_spec'
          },
          {
            'or_groups' => [
                             [
                               [
                                 'regex',
                                 '(?=\\])'
                               ]
                             ]
                           ],
            'name' => 'empty_spec'
          },
          {
            'name' => 'star_spec',
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '*'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'colon_spec',
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 ':'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'index'
                               ],
                               [
                                 'return_object',
                                 '{type: "single_index", value: $1}'
                               ]
                             ]
                           ],
            'name' => 'single_index'
          },
          {
            'name' => 'perl_range',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'index'
                               ],
                               [
                                 'quoted_string',
                                 '..'
                               ],
                               [
                                 'rule_reference',
                                 'index'
                               ],
                               [
                                 'return_object',
                                 '{type: "perl_range", start: $1, end: $3}'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'python_slice_start'
                               ],
                               [
                                 'quoted_string',
                                 ':'
                               ],
                               [
                                 'rule_reference',
                                 'python_slice_end'
                               ],
                               [
                                 'return_object',
                                 '{type: "python_slice", start: $1, end: $3}'
                               ]
                             ]
                           ],
            'name' => 'python_slice'
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'python_slice_start'
                               ],
                               [
                                 'quoted_string',
                                 ':'
                               ],
                               [
                                 'rule_reference',
                                 'python_slice_end'
                               ],
                               [
                                 'quoted_string',
                                 ':'
                               ],
                               [
                                 'rule_reference',
                                 'step'
                               ],
                               [
                                 'return_object',
                                 '{type: "python_slice_step", start: $1, end: $3, step: $5}'
                               ]
                             ]
                           ],
            'name' => 'python_slice_with_step'
          },
          {
            'name' => 'python_slice_start',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'index'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'empty_slice_part'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'python_slice_end',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'index'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'empty_slice_part'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'step',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'index'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'regex',
                                 '(?=:)'
                               ]
                             ]
                           ],
            'name' => 'empty_slice_part'
          },
          {
            'name' => 'index_list',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'index'
                               ],
                               [
                                 'GROUPED',
                                 [
                                   [
                                     'quoted_string',
                                     ','
                                   ],
                                   [
                                     'regex',
                                     '\\s*'
                                   ],
                                   [
                                     'rule_reference',
                                     'index'
                                   ]
                                 ]
                               ],
                               [
                                 'operator',
                                 '*'
                               ],
                               [
                                 'return_array',
                                 '[$1, $3*]'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'mixed_expression',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'mixed_element'
                               ],
                               [
                                 'GROUPED',
                                 [
                                   [
                                     'quoted_string',
                                     ','
                                   ],
                                   [
                                     'regex',
                                     '\\s*'
                                   ],
                                   [
                                     'rule_reference',
                                     'mixed_element'
                                   ]
                                 ]
                               ],
                               [
                                 'operator',
                                 '*'
                               ],
                               [
                                 'return_array',
                                 '[$1, $3*]'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'single_index'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'perl_range'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'python_slice'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'python_slice_with_step'
                               ]
                             ]
                           ],
            'name' => 'mixed_element'
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'positive_number'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'negative_number'
                               ]
                             ]
                           ],
            'name' => 'index'
          },
          {
            'or_groups' => [
                             [
                               [
                                 'regex',
                                 '(\\d+)'
                               ],
                               [
                                 'return_object',
                                 '{type: "positive", value: $1}'
                               ]
                             ]
                           ],
            'name' => 'positive_number'
          },
          {
            'name' => 'negative_number',
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '-'
                               ],
                               [
                                 'rule_reference',
                                 'positive_number'
                               ],
                               [
                                 'return_object',
                                 '{type: "negative", value: $2}'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '['
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'grouped_element_list'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ']'
                               ],
                               [
                                 'return_object',
                                 '{type: "grouped_quantified_array", groups: $3}'
                               ]
                             ]
                           ],
            'name' => 'grouped_quantified_array'
          },
          {
            'name' => 'grouped_element_list',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'grouped_element_item'
                               ],
                               [
                                 'GROUPED',
                                 [
                                   [
                                     'quoted_string',
                                     ','
                                   ],
                                   [
                                     'regex',
                                     '\\s*'
                                   ],
                                   [
                                     'rule_reference',
                                     'grouped_element_item'
                                   ]
                                 ]
                               ],
                               [
                                 'operator',
                                 '*'
                               ],
                               [
                                 'return_array',
                                 '[$1, $3*]'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'grouped_element_item'
                               ],
                               [
                                 'return_array',
                                 '[$1]'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'grouped_element_item',
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '('
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'group_content'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ')'
                               ],
                               [
                                 'rule_reference',
                                 'quantifier'
                               ],
                               [
                                 'return_object',
                                 '{type: "grouped_quantified", content: $3, quantifier: $6}'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'element_sequence'
                               ],
                               [
                                 'rule_reference',
                                 'quantifier'
                               ],
                               [
                                 'return_object',
                                 '{type: "sequence_quantified", content: $1, quantifier: $2}'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'element_sequence'
                               ]
                             ]
                           ],
            'name' => 'group_content'
          },
          {
            'name' => 'element_sequence',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'element_item'
                               ],
                               [
                                 'GROUPED',
                                 [
                                   [
                                     'regex',
                                     '\\s+'
                                   ],
                                   [
                                     'rule_reference',
                                     'element_item'
                                   ]
                                 ]
                               ],
                               [
                                 'operator',
                                 '*'
                               ],
                               [
                                 'return_array',
                                 '[$1, $3*]'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'element_item'
                               ],
                               [
                                 'return_array',
                                 '[$1]'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'element_item',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'identifier'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'literal'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '{'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'outer_key'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ':'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'inner_object'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 '}'
                               ],
                               [
                                 'return_object',
                                 '{type: "nested_object", key: $3, value: $7}'
                               ]
                             ]
                           ],
            'name' => 'simple_nested_object'
          },
          {
            'name' => 'inner_object',
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '{'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'inner_key'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ':'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'inner_value'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 '}'
                               ],
                               [
                                 'return_object',
                                 '{type: "inner_object", key: $3, value: $7}'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'inner_value',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'nested_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'nested_object'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'grouped_quantified_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'quantified_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'simple_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'ultimate_dot_notation'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'scalar_ref'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'literal'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'multi_property_object',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'two_property_object'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'three_property_object'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'two_property_object',
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '{'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'property'
                               ],
                               [
                                 'quoted_string',
                                 ','
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'property'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 '}'
                               ],
                               [
                                 'return_object',
                                 '{type: "multi_object", prop1: $3, prop2: $6}'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'three_property_object',
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '{'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'property'
                               ],
                               [
                                 'quoted_string',
                                 ','
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'property'
                               ],
                               [
                                 'quoted_string',
                                 ','
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'property'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 '}'
                               ],
                               [
                                 'return_object',
                                 '{type: "multi_object", prop1: $3, prop2: $6, prop3: $9}'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'object_key'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ':'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'property_value'
                               ],
                               [
                                 'return_object',
                                 '{key: $1, value: $5}'
                               ]
                             ]
                           ],
            'name' => 'property'
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'nested_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'nested_object'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'grouped_quantified_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'quantified_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'simple_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'ultimate_dot_notation'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'scalar_ref'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'literal'
                               ]
                             ]
                           ],
            'name' => 'property_value'
          },
          {
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '['
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'quantified_element'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ']'
                               ],
                               [
                                 'return_object',
                                 '{type: "quantified_array", element: $3}'
                               ]
                             ]
                           ],
            'name' => 'quantified_array'
          },
          {
            'name' => 'quantified_element',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'ultimate_dot_notation'
                               ],
                               [
                                 'rule_reference',
                                 'quantifier'
                               ],
                               [
                                 'return_object',
                                 '{scalar: $1, quantifier: $2}'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'scalar_ref'
                               ],
                               [
                                 'rule_reference',
                                 'quantifier'
                               ],
                               [
                                 'return_object',
                                 '{scalar: $1, quantifier: $2}'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '*'
                               ],
                               [
                                 'return_scalar',
                                 '"*"'
                               ]
                             ],
                             [
                               [
                                 'quoted_string',
                                 '+'
                               ],
                               [
                                 'return_scalar',
                                 '"+"'
                               ]
                             ],
                             [
                               [
                                 'quoted_string',
                                 '?'
                               ],
                               [
                                 'return_scalar',
                                 '"?"'
                               ]
                             ],
                             [
                               [
                                 'quoted_string',
                                 '{'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'positive_number'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 '}'
                               ],
                               [
                                 'return_object',
                                 '{min: $3, max: $3}'
                               ]
                             ],
                             [
                               [
                                 'quoted_string',
                                 '{'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'positive_number'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ','
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 '}'
                               ],
                               [
                                 'return_object',
                                 '{min: $3, max: "inf"}'
                               ]
                             ],
                             [
                               [
                                 'quoted_string',
                                 '{'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'positive_number'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ','
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'positive_number'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 '}'
                               ],
                               [
                                 'return_object',
                                 '{min: $3, max: $7}'
                               ]
                             ],
                             [
                               [
                                 'quoted_string',
                                 '{'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ','
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'positive_number'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 '}'
                               ],
                               [
                                 'return_object',
                                 '{min: 0, max: $5}'
                               ]
                             ]
                           ],
            'name' => 'quantifier'
          },
          {
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '['
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'array_element'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ']'
                               ],
                               [
                                 'return_object',
                                 '{type: "array", element: $3}'
                               ]
                             ]
                           ],
            'name' => 'simple_array'
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'ultimate_dot_notation'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'scalar_ref'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'literal'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'nested_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'nested_object'
                               ]
                             ]
                           ],
            'name' => 'array_element'
          },
          {
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '{'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'object_key'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 ':'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'rule_reference',
                                 'object_value'
                               ],
                               [
                                 'regex',
                                 '\\s*'
                               ],
                               [
                                 'quoted_string',
                                 '}'
                               ],
                               [
                                 'return_object',
                                 '{type: "object", key: $3, value: $7}'
                               ]
                             ]
                           ],
            'name' => 'simple_object'
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'identifier'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'quoted_string'
                               ]
                             ]
                           ],
            'name' => 'object_key'
          },
          {
            'name' => 'outer_key',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'identifier'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'quoted_string'
                               ]
                             ]
                           ]
          },
          {
            'name' => 'inner_key',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'identifier'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'quoted_string'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'ultimate_dot_notation'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'scalar_ref'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'literal'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'nested_array'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'nested_object'
                               ]
                             ]
                           ],
            'name' => 'object_value'
          },
          {
            'name' => 'literal',
            'or_groups' => [
                             [
                               [
                                 'rule_reference',
                                 'quoted_string'
                               ]
                             ],
                             [
                               [
                                 'rule_reference',
                                 'number'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'quoted_string',
                                 '$'
                               ],
                               [
                                 'rule_reference',
                                 'positive_number'
                               ],
                               [
                                 'return_object',
                                 '{type: "scalar_ref", index: $2}'
                               ]
                             ]
                           ],
            'name' => 'scalar_ref'
          },
          {
            'name' => 'quoted_string',
            'or_groups' => [
                             [
                               [
                                 'regex',
                                 '"([^"]*)"'
                               ],
                               [
                                 'return_scalar',
                                 '$1'
                               ]
                             ]
                           ]
          },
          {
            'or_groups' => [
                             [
                               [
                                 'regex',
                                 '(\\d+)'
                               ],
                               [
                                 'return_scalar',
                                 '$1'
                               ]
                             ]
                           ],
            'name' => 'number'
          },
          {
            'or_groups' => [
                             [
                               [
                                 'regex',
                                 '([a-zA-Z_]\\w*)'
                               ],
                               [
                                 'return_scalar',
                                 '$1'
                               ]
                             ]
                           ],
            'name' => 'identifier'
          }
        ];

=== Step 3: Parse sequences ===
DEBUG: index_list before step3:
$VAR1 = {
          'name' => 'index_list',
          'or_groups' => [
                           [
                             [
                               'rule_reference',
                               'index'
                             ],
                             [
                               'GROUPED',
                               [
                                 [
                                   'quoted_string',
                                   ','
                                 ],
                                 [
                                   'regex',
                                   '\\s*'
                                 ],
                                 [
                                   'rule_reference',
                                   'index'
                                 ]
                               ]
                             ],
                             [
                               'operator',
                               '*'
                             ],
                             [
                               'return_array',
                               '[$1, $3*]'
                             ]
                           ]
                         ]
        };

🔍 STEP 3 DEBUG: Looking for dot_path rule in input...
🎯 STEP 3: Found dot_path rule in input: $VAR1 = {
          'name' => 'dot_path',
          'or_groups' => [
                           [
                             [
                               'rule_reference',
                               'accessor'
                             ],
                             [
                               'operator',
                               '+'
                             ],
                             [
                               'return_array',
                               '[$1*]'
                             ]
                           ]
                         ]
        };

✅ STEP 3: dot_path rule found in output: $VAR1 = {
          'type' => 'sequence',
          'elements' => [
                          [
                            'rule_reference',
                            'accessor'
                          ],
                          [
                            'operator',
                            '+'
                          ],
                          [
                            'return_array',
                            '[$1*]'
                          ]
                        ],
          'name' => 'dot_path'
        };

STEP 3 RESULT (Sequences parsed):
$VAR1 = [
          {
            'name' => 'return_annotation',
            'type' => 'sequence',
            'elements' => [
                            [
                              'quoted_string',
                              '->'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'rule_reference',
                              'return_expression'
                            ]
                          ]
          },
          {
            'type' => 'or',
            'alternatives' => [
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'nested_array'
                                                  ]
                                                ]
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'nested_object'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'grouped_quantified_array'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'simple_nested_object'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'multi_property_object'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'quantified_array'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'simple_array'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'simple_object'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'ultimate_dot_notation'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'scalar_ref'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'literal'
                                                  ]
                                                ]
                                }
                              ],
            'name' => 'return_expression'
          },
          {
            'type' => 'sequence',
            'elements' => [
                            [
                              'quoted_string',
                              '['
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'rule_reference',
                              'array_contents'
                            ],
                            [
                              'operator',
                              '?'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'quoted_string',
                              ']'
                            ],
                            [
                              'rule_reference',
                              'quantifier'
                            ],
                            [
                              'operator',
                              '?'
                            ],
                            [
                              'return_object',
                              '{type: "array", contents: $3, quantified: $6}'
                            ]
                          ],
            'name' => 'nested_array'
          },
          {
            'name' => 'array_contents',
            'type' => 'sequence',
            'elements' => [
                            [
                              'rule_reference',
                              'return_expression'
                            ],
                            [
                              'GROUPED',
                              [
                                [
                                  'quoted_string',
                                  ','
                                ],
                                [
                                  'regex',
                                  '\\s*'
                                ],
                                [
                                  'rule_reference',
                                  'return_expression'
                                ]
                              ]
                            ],
                            [
                              'operator',
                              '*'
                            ],
                            [
                              'return_array',
                              '[$1, $3*]'
                            ]
                          ]
          },
          {
            'name' => 'nested_object',
            'type' => 'sequence',
            'elements' => [
                            [
                              'quoted_string',
                              '{'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'rule_reference',
                              'object_contents'
                            ],
                            [
                              'operator',
                              '?'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'quoted_string',
                              '}'
                            ],
                            [
                              'rule_reference',
                              'quantifier'
                            ],
                            [
                              'operator',
                              '?'
                            ],
                            [
                              'return_object',
                              '{type: "object", contents: $3, quantified: $6}'
                            ]
                          ]
          },
          {
            'elements' => [
                            [
                              'rule_reference',
                              'object_pair'
                            ],
                            [
                              'GROUPED',
                              [
                                [
                                  'quoted_string',
                                  ','
                                ],
                                [
                                  'regex',
                                  '\\s*'
                                ],
                                [
                                  'rule_reference',
                                  'object_pair'
                                ]
                              ]
                            ],
                            [
                              'operator',
                              '*'
                            ],
                            [
                              'return_array',
                              '[$1, $3*]'
                            ]
                          ],
            'type' => 'sequence',
            'name' => 'object_contents'
          },
          {
            'name' => 'object_pair',
            'type' => 'sequence',
            'elements' => [
                            [
                              'rule_reference',
                              'object_key'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'quoted_string',
                              ':'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'rule_reference',
                              'return_expression'
                            ],
                            [
                              'return_object',
                              '{key: $1, value: $5}'
                            ]
                          ]
          },
          {
            'name' => 'ultimate_dot_notation',
            'elements' => [
                            [
                              'rule_reference',
                              'scalar_ref'
                            ],
                            [
                              'rule_reference',
                              'dot_path'
                            ],
                            [
                              'return_object',
                              '{type: "ultimate_dot_notation", base: $1, path: $2}'
                            ]
                          ],
            'type' => 'sequence'
          },
          {
            'type' => 'sequence',
            'elements' => [
                            [
                              'rule_reference',
                              'accessor'
                            ],
                            [
                              'operator',
                              '+'
                            ],
                            [
                              'return_array',
                              '[$1*]'
                            ]
                          ],
            'name' => 'dot_path'
          },
          {
            'alternatives' => [
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'property_accessor'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'positional_accessor'
                                                  ]
                                                ]
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'array_accessor'
                                                  ]
                                                ]
                                }
                              ],
            'name' => 'accessor',
            'type' => 'or'
          },
          {
            'elements' => [
                            [
                              'quoted_string',
                              '.'
                            ],
                            [
                              'rule_reference',
                              'identifier'
                            ],
                            [
                              'return_object',
                              '{type: "property", name: $2}'
                            ]
                          ],
            'type' => 'sequence',
            'name' => 'property_accessor'
          },
          {
            'name' => 'positional_accessor',
            'elements' => [
                            [
                              'quoted_string',
                              '.'
                            ],
                            [
                              'rule_reference',
                              'positive_number'
                            ],
                            [
                              'return_object',
                              '{type: "position", index: $2}'
                            ]
                          ],
            'type' => 'sequence'
          },
          {
            'name' => 'array_accessor',
            'type' => 'sequence',
            'elements' => [
                            [
                              'quoted_string',
                              '['
                            ],
                            [
                              'rule_reference',
                              'array_spec'
                            ],
                            [
                              'quoted_string',
                              ']'
                            ],
                            [
                              'return_object',
                              '{type: "array_access", spec: $2}'
                            ]
                          ]
          },
          {
            'name' => 'array_spec',
            'alternatives' => [
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'empty_spec'
                                                  ],
                                                  [
                                                    'return_object',
                                                    '{type: "whole_array", style: "implicit"}'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'star_spec'
                                                  ],
                                                  [
                                                    'return_object',
                                                    '{type: "whole_array", style: "bash"}'
                                                  ]
                                                ]
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'colon_spec'
                                                  ],
                                                  [
                                                    'return_object',
                                                    '{type: "whole_array", style: "python"}'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'single_index'
                                                  ],
                                                  [
                                                    'return_scalar',
                                                    '$1'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'perl_range'
                                                  ],
                                                  [
                                                    'return_scalar',
                                                    '$1'
                                                  ]
                                                ]
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'python_slice'
                                                  ],
                                                  [
                                                    'return_scalar',
                                                    '$1'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'python_slice_with_step'
                                                  ],
                                                  [
                                                    'return_scalar',
                                                    '$1'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'index_list'
                                                  ],
                                                  [
                                                    'return_object',
                                                    '{type: "multi_index", indices: $1}'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'mixed_expression'
                                                  ],
                                                  [
                                                    'return_object',
                                                    '{type: "mixed_expression", elements: $1}'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                }
                              ],
            'type' => 'or'
          },
          {
            'name' => 'empty_spec',
            'elements' => [
                            [
                              'regex',
                              '(?=\\])'
                            ]
                          ],
            'type' => 'sequence'
          },
          {
            'name' => 'star_spec',
            'type' => 'sequence',
            'elements' => [
                            [
                              'quoted_string',
                              '*'
                            ]
                          ]
          },
          {
            'name' => 'colon_spec',
            'type' => 'sequence',
            'elements' => [
                            [
                              'quoted_string',
                              ':'
                            ]
                          ]
          },
          {
            'name' => 'single_index',
            'elements' => [
                            [
                              'rule_reference',
                              'index'
                            ],
                            [
                              'return_object',
                              '{type: "single_index", value: $1}'
                            ]
                          ],
            'type' => 'sequence'
          },
          {
            'name' => 'perl_range',
            'elements' => [
                            [
                              'rule_reference',
                              'index'
                            ],
                            [
                              'quoted_string',
                              '..'
                            ],
                            [
                              'rule_reference',
                              'index'
                            ],
                            [
                              'return_object',
                              '{type: "perl_range", start: $1, end: $3}'
                            ]
                          ],
            'type' => 'sequence'
          },
          {
            'name' => 'python_slice',
            'elements' => [
                            [
                              'rule_reference',
                              'python_slice_start'
                            ],
                            [
                              'quoted_string',
                              ':'
                            ],
                            [
                              'rule_reference',
                              'python_slice_end'
                            ],
                            [
                              'return_object',
                              '{type: "python_slice", start: $1, end: $3}'
                            ]
                          ],
            'type' => 'sequence'
          },
          {
            'name' => 'python_slice_with_step',
            'elements' => [
                            [
                              'rule_reference',
                              'python_slice_start'
                            ],
                            [
                              'quoted_string',
                              ':'
                            ],
                            [
                              'rule_reference',
                              'python_slice_end'
                            ],
                            [
                              'quoted_string',
                              ':'
                            ],
                            [
                              'rule_reference',
                              'step'
                            ],
                            [
                              'return_object',
                              '{type: "python_slice_step", start: $1, end: $3, step: $5}'
                            ]
                          ],
            'type' => 'sequence'
          },
          {
            'name' => 'python_slice_start',
            'alternatives' => [
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'index'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'empty_slice_part'
                                                  ]
                                                ]
                                }
                              ],
            'type' => 'or'
          },
          {
            'name' => 'python_slice_end',
            'alternatives' => [
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'index'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'empty_slice_part'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                }
                              ],
            'type' => 'or'
          },
          {
            'name' => 'step',
            'type' => 'sequence',
            'elements' => [
                            [
                              'rule_reference',
                              'index'
                            ]
                          ]
          },
          {
            'name' => 'empty_slice_part',
            'elements' => [
                            [
                              'regex',
                              '(?=:)'
                            ]
                          ],
            'type' => 'sequence'
          },
          {
            'elements' => [
                            [
                              'rule_reference',
                              'index'
                            ],
                            [
                              'GROUPED',
                              [
                                [
                                  'quoted_string',
                                  ','
                                ],
                                [
                                  'regex',
                                  '\\s*'
                                ],
                                [
                                  'rule_reference',
                                  'index'
                                ]
                              ]
                            ],
                            [
                              'operator',
                              '*'
                            ],
                            [
                              'return_array',
                              '[$1, $3*]'
                            ]
                          ],
            'type' => 'sequence',
            'name' => 'index_list'
          },
          {
            'name' => 'mixed_expression',
            'type' => 'sequence',
            'elements' => [
                            [
                              'rule_reference',
                              'mixed_element'
                            ],
                            [
                              'GROUPED',
                              [
                                [
                                  'quoted_string',
                                  ','
                                ],
                                [
                                  'regex',
                                  '\\s*'
                                ],
                                [
                                  'rule_reference',
                                  'mixed_element'
                                ]
                              ]
                            ],
                            [
                              'operator',
                              '*'
                            ],
                            [
                              'return_array',
                              '[$1, $3*]'
                            ]
                          ]
          },
          {
            'type' => 'or',
            'alternatives' => [
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'single_index'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'perl_range'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'python_slice'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'python_slice_with_step'
                                                  ]
                                                ]
                                }
                              ],
            'name' => 'mixed_element'
          },
          {
            'type' => 'or',
            'alternatives' => [
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'positive_number'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'negative_number'
                                                  ]
                                                ]
                                }
                              ],
            'name' => 'index'
          },
          {
            'name' => 'positive_number',
            'elements' => [
                            [
                              'regex',
                              '(\\d+)'
                            ],
                            [
                              'return_object',
                              '{type: "positive", value: $1}'
                            ]
                          ],
            'type' => 'sequence'
          },
          {
            'elements' => [
                            [
                              'quoted_string',
                              '-'
                            ],
                            [
                              'rule_reference',
                              'positive_number'
                            ],
                            [
                              'return_object',
                              '{type: "negative", value: $2}'
                            ]
                          ],
            'type' => 'sequence',
            'name' => 'negative_number'
          },
          {
            'type' => 'sequence',
            'elements' => [
                            [
                              'quoted_string',
                              '['
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'rule_reference',
                              'grouped_element_list'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'quoted_string',
                              ']'
                            ],
                            [
                              'return_object',
                              '{type: "grouped_quantified_array", groups: $3}'
                            ]
                          ],
            'name' => 'grouped_quantified_array'
          },
          {
            'alternatives' => [
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'grouped_element_item'
                                                  ],
                                                  [
                                                    'GROUPED',
                                                    [
                                                      [
                                                        'quoted_string',
                                                        ','
                                                      ],
                                                      [
                                                        'regex',
                                                        '\\s*'
                                                      ],
                                                      [
                                                        'rule_reference',
                                                        'grouped_element_item'
                                                      ]
                                                    ]
                                                  ],
                                                  [
                                                    'operator',
                                                    '*'
                                                  ],
                                                  [
                                                    'return_array',
                                                    '[$1, $3*]'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'grouped_element_item'
                                                  ],
                                                  [
                                                    'return_array',
                                                    '[$1]'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                }
                              ],
            'name' => 'grouped_element_list',
            'type' => 'or'
          },
          {
            'alternatives' => [
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'quoted_string',
                                                    '('
                                                  ],
                                                  [
                                                    'regex',
                                                    '\\s*'
                                                  ],
                                                  [
                                                    'rule_reference',
                                                    'group_content'
                                                  ],
                                                  [
                                                    'regex',
                                                    '\\s*'
                                                  ],
                                                  [
                                                    'quoted_string',
                                                    ')'
                                                  ],
                                                  [
                                                    'rule_reference',
                                                    'quantifier'
                                                  ],
                                                  [
                                                    'return_object',
                                                    '{type: "grouped_quantified", content: $3, quantifier: $6}'
                                                  ]
                                                ]
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'element_sequence'
                                                  ],
                                                  [
                                                    'rule_reference',
                                                    'quantifier'
                                                  ],
                                                  [
                                                    'return_object',
                                                    '{type: "sequence_quantified", content: $1, quantifier: $2}'
                                                  ]
                                                ]
                                }
                              ],
            'name' => 'grouped_element_item',
            'type' => 'or'
          },
          {
            'elements' => [
                            [
                              'rule_reference',
                              'element_sequence'
                            ]
                          ],
            'type' => 'sequence',
            'name' => 'group_content'
          },
          {
            'alternatives' => [
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'element_item'
                                                  ],
                                                  [
                                                    'GROUPED',
                                                    [
                                                      [
                                                        'regex',
                                                        '\\s+'
                                                      ],
                                                      [
                                                        'rule_reference',
                                                        'element_item'
                                                      ]
                                                    ]
                                                  ],
                                                  [
                                                    'operator',
                                                    '*'
                                                  ],
                                                  [
                                                    'return_array',
                                                    '[$1, $3*]'
                                                  ]
                                                ]
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'element_item'
                                                  ],
                                                  [
                                                    'return_array',
                                                    '[$1]'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                }
                              ],
            'name' => 'element_sequence',
            'type' => 'or'
          },
          {
            'alternatives' => [
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'identifier'
                                                  ]
                                                ]
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'literal'
                                                  ]
                                                ]
                                }
                              ],
            'name' => 'element_item',
            'type' => 'or'
          },
          {
            'elements' => [
                            [
                              'quoted_string',
                              '{'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'rule_reference',
                              'outer_key'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'quoted_string',
                              ':'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'rule_reference',
                              'inner_object'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'quoted_string',
                              '}'
                            ],
                            [
                              'return_object',
                              '{type: "nested_object", key: $3, value: $7}'
                            ]
                          ],
            'type' => 'sequence',
            'name' => 'simple_nested_object'
          },
          {
            'name' => 'inner_object',
            'type' => 'sequence',
            'elements' => [
                            [
                              'quoted_string',
                              '{'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'rule_reference',
                              'inner_key'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'quoted_string',
                              ':'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'rule_reference',
                              'inner_value'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'quoted_string',
                              '}'
                            ],
                            [
                              'return_object',
                              '{type: "inner_object", key: $3, value: $7}'
                            ]
                          ]
          },
          {
            'name' => 'inner_value',
            'alternatives' => [
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'nested_array'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'nested_object'
                                                  ]
                                                ]
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'grouped_quantified_array'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'quantified_array'
                                                  ]
                                                ]
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'simple_array'
                                                  ]
                                                ]
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'ultimate_dot_notation'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'scalar_ref'
                                                  ]
                                                ]
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'literal'
                                                  ]
                                                ]
                                }
                              ],
            'type' => 'or'
          },
          {
            'name' => 'multi_property_object',
            'alternatives' => [
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'two_property_object'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'three_property_object'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                }
                              ],
            'type' => 'or'
          },
          {
            'elements' => [
                            [
                              'quoted_string',
                              '{'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'rule_reference',
                              'property'
                            ],
                            [
                              'quoted_string',
                              ','
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'rule_reference',
                              'property'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'quoted_string',
                              '}'
                            ],
                            [
                              'return_object',
                              '{type: "multi_object", prop1: $3, prop2: $6}'
                            ]
                          ],
            'type' => 'sequence',
            'name' => 'two_property_object'
          },
          {
            'name' => 'three_property_object',
            'elements' => [
                            [
                              'quoted_string',
                              '{'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'rule_reference',
                              'property'
                            ],
                            [
                              'quoted_string',
                              ','
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'rule_reference',
                              'property'
                            ],
                            [
                              'quoted_string',
                              ','
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'rule_reference',
                              'property'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'quoted_string',
                              '}'
                            ],
                            [
                              'return_object',
                              '{type: "multi_object", prop1: $3, prop2: $6, prop3: $9}'
                            ]
                          ],
            'type' => 'sequence'
          },
          {
            'name' => 'property',
            'type' => 'sequence',
            'elements' => [
                            [
                              'rule_reference',
                              'object_key'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'quoted_string',
                              ':'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'rule_reference',
                              'property_value'
                            ],
                            [
                              'return_object',
                              '{key: $1, value: $5}'
                            ]
                          ]
          },
          {
            'type' => 'or',
            'alternatives' => [
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'nested_array'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'nested_object'
                                                  ]
                                                ]
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'grouped_quantified_array'
                                                  ]
                                                ]
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'quantified_array'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'simple_array'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'ultimate_dot_notation'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'scalar_ref'
                                                  ]
                                                ]
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'literal'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                }
                              ],
            'name' => 'property_value'
          },
          {
            'elements' => [
                            [
                              'quoted_string',
                              '['
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'rule_reference',
                              'quantified_element'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'quoted_string',
                              ']'
                            ],
                            [
                              'return_object',
                              '{type: "quantified_array", element: $3}'
                            ]
                          ],
            'type' => 'sequence',
            'name' => 'quantified_array'
          },
          {
            'type' => 'or',
            'alternatives' => [
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'ultimate_dot_notation'
                                                  ],
                                                  [
                                                    'rule_reference',
                                                    'quantifier'
                                                  ],
                                                  [
                                                    'return_object',
                                                    '{scalar: $1, quantifier: $2}'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'scalar_ref'
                                                  ],
                                                  [
                                                    'rule_reference',
                                                    'quantifier'
                                                  ],
                                                  [
                                                    'return_object',
                                                    '{scalar: $1, quantifier: $2}'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                }
                              ],
            'name' => 'quantified_element'
          },
          {
            'type' => 'or',
            'alternatives' => [
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'quoted_string',
                                                    '*'
                                                  ],
                                                  [
                                                    'return_scalar',
                                                    '"*"'
                                                  ]
                                                ]
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'quoted_string',
                                                    '+'
                                                  ],
                                                  [
                                                    'return_scalar',
                                                    '"+"'
                                                  ]
                                                ]
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'quoted_string',
                                                    '?'
                                                  ],
                                                  [
                                                    'return_scalar',
                                                    '"?"'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'quoted_string',
                                                    '{'
                                                  ],
                                                  [
                                                    'regex',
                                                    '\\s*'
                                                  ],
                                                  [
                                                    'rule_reference',
                                                    'positive_number'
                                                  ],
                                                  [
                                                    'regex',
                                                    '\\s*'
                                                  ],
                                                  [
                                                    'quoted_string',
                                                    '}'
                                                  ],
                                                  [
                                                    'return_object',
                                                    '{min: $3, max: $3}'
                                                  ]
                                                ]
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'quoted_string',
                                                    '{'
                                                  ],
                                                  [
                                                    'regex',
                                                    '\\s*'
                                                  ],
                                                  [
                                                    'rule_reference',
                                                    'positive_number'
                                                  ],
                                                  [
                                                    'regex',
                                                    '\\s*'
                                                  ],
                                                  [
                                                    'quoted_string',
                                                    ','
                                                  ],
                                                  [
                                                    'regex',
                                                    '\\s*'
                                                  ],
                                                  [
                                                    'quoted_string',
                                                    '}'
                                                  ],
                                                  [
                                                    'return_object',
                                                    '{min: $3, max: "inf"}'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'quoted_string',
                                                    '{'
                                                  ],
                                                  [
                                                    'regex',
                                                    '\\s*'
                                                  ],
                                                  [
                                                    'rule_reference',
                                                    'positive_number'
                                                  ],
                                                  [
                                                    'regex',
                                                    '\\s*'
                                                  ],
                                                  [
                                                    'quoted_string',
                                                    ','
                                                  ],
                                                  [
                                                    'regex',
                                                    '\\s*'
                                                  ],
                                                  [
                                                    'rule_reference',
                                                    'positive_number'
                                                  ],
                                                  [
                                                    'regex',
                                                    '\\s*'
                                                  ],
                                                  [
                                                    'quoted_string',
                                                    '}'
                                                  ],
                                                  [
                                                    'return_object',
                                                    '{min: $3, max: $7}'
                                                  ]
                                                ]
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'quoted_string',
                                                    '{'
                                                  ],
                                                  [
                                                    'regex',
                                                    '\\s*'
                                                  ],
                                                  [
                                                    'quoted_string',
                                                    ','
                                                  ],
                                                  [
                                                    'regex',
                                                    '\\s*'
                                                  ],
                                                  [
                                                    'rule_reference',
                                                    'positive_number'
                                                  ],
                                                  [
                                                    'regex',
                                                    '\\s*'
                                                  ],
                                                  [
                                                    'quoted_string',
                                                    '}'
                                                  ],
                                                  [
                                                    'return_object',
                                                    '{min: 0, max: $5}'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                }
                              ],
            'name' => 'quantifier'
          },
          {
            'elements' => [
                            [
                              'quoted_string',
                              '['
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'rule_reference',
                              'array_element'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'quoted_string',
                              ']'
                            ],
                            [
                              'return_object',
                              '{type: "array", element: $3}'
                            ]
                          ],
            'type' => 'sequence',
            'name' => 'simple_array'
          },
          {
            'type' => 'or',
            'alternatives' => [
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'ultimate_dot_notation'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'scalar_ref'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'literal'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'nested_array'
                                                  ]
                                                ]
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'nested_object'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                }
                              ],
            'name' => 'array_element'
          },
          {
            'elements' => [
                            [
                              'quoted_string',
                              '{'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'rule_reference',
                              'object_key'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'quoted_string',
                              ':'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'rule_reference',
                              'object_value'
                            ],
                            [
                              'regex',
                              '\\s*'
                            ],
                            [
                              'quoted_string',
                              '}'
                            ],
                            [
                              'return_object',
                              '{type: "object", key: $3, value: $7}'
                            ]
                          ],
            'type' => 'sequence',
            'name' => 'simple_object'
          },
          {
            'type' => 'or',
            'alternatives' => [
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'identifier'
                                                  ]
                                                ]
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'quoted_string'
                                                  ]
                                                ]
                                }
                              ],
            'name' => 'object_key'
          },
          {
            'alternatives' => [
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'identifier'
                                                  ]
                                                ]
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'quoted_string'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                }
                              ],
            'name' => 'outer_key',
            'type' => 'or'
          },
          {
            'alternatives' => [
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'identifier'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'quoted_string'
                                                  ]
                                                ]
                                }
                              ],
            'name' => 'inner_key',
            'type' => 'or'
          },
          {
            'alternatives' => [
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'ultimate_dot_notation'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'scalar_ref'
                                                  ]
                                                ]
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'literal'
                                                  ]
                                                ]
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'nested_array'
                                                  ]
                                                ]
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'nested_object'
                                                  ]
                                                ]
                                }
                              ],
            'name' => 'object_value',
            'type' => 'or'
          },
          {
            'type' => 'or',
            'name' => 'literal',
            'alternatives' => [
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'quoted_string'
                                                  ]
                                                ]
                                },
                                {
                                  'elements' => [
                                                  [
                                                    'rule_reference',
                                                    'number'
                                                  ]
                                                ],
                                  'type' => 'sequence'
                                }
                              ]
          },
          {
            'type' => 'sequence',
            'elements' => [
                            [
                              'quoted_string',
                              '$'
                            ],
                            [
                              'rule_reference',
                              'positive_number'
                            ],
                            [
                              'return_object',
                              '{type: "scalar_ref", index: $2}'
                            ]
                          ],
            'name' => 'scalar_ref'
          },
          {
            'name' => 'quoted_string',
            'elements' => [
                            [
                              'regex',
                              '"([^"]*)"'
                            ],
                            [
                              'return_scalar',
                              '$1'
                            ]
                          ],
            'type' => 'sequence'
          },
          {
            'name' => 'number',
            'elements' => [
                            [
                              'regex',
                              '(\\d+)'
                            ],
                            [
                              'return_scalar',
                              '$1'
                            ]
                          ],
            'type' => 'sequence'
          },
          {
            'type' => 'sequence',
            'elements' => [
                            [
                              'regex',
                              '([a-zA-Z_]\\w*)'
                            ],
                            [
                              'return_scalar',
                              '$1'
                            ]
                          ],
            'name' => 'identifier'
          }
        ];
DEBUG: index_list after step3:
$VAR1 = {
          'elements' => [
                          [
                            'rule_reference',
                            'index'
                          ],
                          [
                            'GROUPED',
                            [
                              [
                                'quoted_string',
                                ','
                              ],
                              [
                                'regex',
                                '\\s*'
                              ],
                              [
                                'rule_reference',
                                'index'
                              ]
                            ]
                          ],
                          [
                            'operator',
                            '*'
                          ],
                          [
                            'return_array',
                            '[$1, $3*]'
                          ]
                        ],
          'type' => 'sequence',
          'name' => 'index_list'
        };

=== Step 4: Handle quantifiers ===
DEBUG: index_list before step4:
$VAR1 = {
          'elements' => [
                          [
                            'rule_reference',
                            'index'
                          ],
                          [
                            'GROUPED',
                            [
                              [
                                'quoted_string',
                                ','
                              ],
                              [
                                'regex',
                                '\\s*'
                              ],
                              [
                                'rule_reference',
                                'index'
                              ]
                            ]
                          ],
                          [
                            'operator',
                            '*'
                          ],
                          [
                            'return_array',
                            '[$1, $3*]'
                          ]
                        ],
          'type' => 'sequence',
          'name' => 'index_list'
        };

🔍 STEP 4 DEBUG: Looking for dot_path rule in input...
🎯 STEP 4: Found dot_path rule in input: $VAR1 = {
          'type' => 'sequence',
          'elements' => [
                          [
                            'rule_reference',
                            'accessor'
                          ],
                          [
                            'operator',
                            '+'
                          ],
                          [
                            'return_array',
                            '[$1*]'
                          ]
                        ],
          'name' => 'dot_path'
        };

✅ STEP 4: dot_path rule found in output: $VAR1 = {
          'return_annotation' => [
                                   'return_array',
                                   '[$1*]'
                                 ],
          'name' => 'dot_path',
          'elements' => [
                          {
                            'type' => 'quantified',
                            'element' => [
                                           'rule_reference',
                                           'accessor'
                                         ],
                            'quantifier' => '+'
                          }
                        ],
          'type' => 'sequence'
        };

STEP 4 RESULT (Quantifiers handled):
$VAR1 = [
          {
            'name' => 'return_annotation',
            'elements' => [
                            {
                              'value' => [
                                           'quoted_string',
                                           '->'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'return_expression'
                                         ]
                            }
                          ],
            'type' => 'sequence'
          },
          {
            'type' => 'or',
            'alternatives' => [
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'nested_array'
                                                               ]
                                                  }
                                                ]
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'nested_object'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ]
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'grouped_quantified_array'
                                                               ]
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'simple_nested_object'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ]
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'multi_property_object'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'quantified_array'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'simple_array'
                                                               ]
                                                  }
                                                ]
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'simple_object'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ]
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'ultimate_dot_notation'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'scalar_ref'
                                                               ]
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'literal'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'type' => 'sequence'
                                }
                              ],
            'name' => 'return_expression'
          },
          {
            'elements' => [
                            {
                              'type' => 'atom',
                              'value' => [
                                           'quoted_string',
                                           '['
                                         ]
                            },
                            {
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'element' => [
                                             'rule_reference',
                                             'array_contents'
                                           ],
                              'quantifier' => '?',
                              'type' => 'quantified'
                            },
                            {
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'quoted_string',
                                           ']'
                                         ]
                            },
                            {
                              'type' => 'quantified',
                              'element' => [
                                             'rule_reference',
                                             'quantifier'
                                           ],
                              'quantifier' => '?'
                            }
                          ],
            'type' => 'sequence',
            'name' => 'nested_array',
            'return_annotation' => [
                                     'return_object',
                                     '{type: "array", contents: $3, quantified: $6}'
                                   ]
          },
          {
            'return_annotation' => [
                                     'return_array',
                                     '[$1, $3*]'
                                   ],
            'elements' => [
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'return_expression'
                                         ]
                            },
                            {
                              'type' => 'quantified',
                              'element' => {
                                             'type' => 'sequence',
                                             'elements' => [
                                                             [
                                                               'quoted_string',
                                                               ','
                                                             ],
                                                             [
                                                               'regex',
                                                               '\\s*'
                                                             ],
                                                             [
                                                               'rule_reference',
                                                               'return_expression'
                                                             ]
                                                           ]
                                           },
                              'quantifier' => '*'
                            }
                          ],
            'type' => 'sequence',
            'name' => 'array_contents'
          },
          {
            'return_annotation' => [
                                     'return_object',
                                     '{type: "object", contents: $3, quantified: $6}'
                                   ],
            'name' => 'nested_object',
            'elements' => [
                            {
                              'value' => [
                                           'quoted_string',
                                           '{'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ]
                            },
                            {
                              'element' => [
                                             'rule_reference',
                                             'object_contents'
                                           ],
                              'quantifier' => '?',
                              'type' => 'quantified'
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ]
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'quoted_string',
                                           '}'
                                         ]
                            },
                            {
                              'type' => 'quantified',
                              'quantifier' => '?',
                              'element' => [
                                             'rule_reference',
                                             'quantifier'
                                           ]
                            }
                          ],
            'type' => 'sequence'
          },
          {
            'elements' => [
                            {
                              'value' => [
                                           'rule_reference',
                                           'object_pair'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'quantifier' => '*',
                              'element' => {
                                             'type' => 'sequence',
                                             'elements' => [
                                                             [
                                                               'quoted_string',
                                                               ','
                                                             ],
                                                             [
                                                               'regex',
                                                               '\\s*'
                                                             ],
                                                             [
                                                               'rule_reference',
                                                               'object_pair'
                                                             ]
                                                           ]
                                           },
                              'type' => 'quantified'
                            }
                          ],
            'type' => 'sequence',
            'name' => 'object_contents',
            'return_annotation' => [
                                     'return_array',
                                     '[$1, $3*]'
                                   ]
          },
          {
            'type' => 'sequence',
            'elements' => [
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'object_key'
                                         ]
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ]
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'quoted_string',
                                           ':'
                                         ]
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ]
                            },
                            {
                              'value' => [
                                           'rule_reference',
                                           'return_expression'
                                         ],
                              'type' => 'atom'
                            }
                          ],
            'name' => 'object_pair',
            'return_annotation' => [
                                     'return_object',
                                     '{key: $1, value: $5}'
                                   ]
          },
          {
            'return_annotation' => [
                                     'return_object',
                                     '{type: "ultimate_dot_notation", base: $1, path: $2}'
                                   ],
            'name' => 'ultimate_dot_notation',
            'elements' => [
                            {
                              'value' => [
                                           'rule_reference',
                                           'scalar_ref'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'dot_path'
                                         ]
                            }
                          ],
            'type' => 'sequence'
          },
          {
            'return_annotation' => [
                                     'return_array',
                                     '[$1*]'
                                   ],
            'name' => 'dot_path',
            'elements' => [
                            {
                              'type' => 'quantified',
                              'element' => [
                                             'rule_reference',
                                             'accessor'
                                           ],
                              'quantifier' => '+'
                            }
                          ],
            'type' => 'sequence'
          },
          {
            'name' => 'accessor',
            'alternatives' => [
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'property_accessor'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ]
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'positional_accessor'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'array_accessor'
                                                               ]
                                                  }
                                                ]
                                }
                              ],
            'type' => 'or'
          },
          {
            'return_annotation' => [
                                     'return_object',
                                     '{type: "property", name: $2}'
                                   ],
            'name' => 'property_accessor',
            'elements' => [
                            {
                              'type' => 'atom',
                              'value' => [
                                           'quoted_string',
                                           '.'
                                         ]
                            },
                            {
                              'value' => [
                                           'rule_reference',
                                           'identifier'
                                         ],
                              'type' => 'atom'
                            }
                          ],
            'type' => 'sequence'
          },
          {
            'return_annotation' => [
                                     'return_object',
                                     '{type: "position", index: $2}'
                                   ],
            'type' => 'sequence',
            'elements' => [
                            {
                              'type' => 'atom',
                              'value' => [
                                           'quoted_string',
                                           '.'
                                         ]
                            },
                            {
                              'value' => [
                                           'rule_reference',
                                           'positive_number'
                                         ],
                              'type' => 'atom'
                            }
                          ],
            'name' => 'positional_accessor'
          },
          {
            'return_annotation' => [
                                     'return_object',
                                     '{type: "array_access", spec: $2}'
                                   ],
            'name' => 'array_accessor',
            'type' => 'sequence',
            'elements' => [
                            {
                              'value' => [
                                           'quoted_string',
                                           '['
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'array_spec'
                                         ]
                            },
                            {
                              'value' => [
                                           'quoted_string',
                                           ']'
                                         ],
                              'type' => 'atom'
                            }
                          ]
          },
          {
            'type' => 'or',
            'name' => 'array_spec',
            'alternatives' => [
                                {
                                  'return_annotation' => [
                                                           'return_object',
                                                           '{type: "whole_array", style: "implicit"}'
                                                         ],
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'empty_spec'
                                                               ]
                                                  }
                                                ]
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'star_spec'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'type' => 'sequence',
                                  'return_annotation' => [
                                                           'return_object',
                                                           '{type: "whole_array", style: "bash"}'
                                                         ]
                                },
                                {
                                  'return_annotation' => [
                                                           'return_object',
                                                           '{type: "whole_array", style: "python"}'
                                                         ],
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'colon_spec'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'return_annotation' => [
                                                           'return_scalar',
                                                           '$1'
                                                         ],
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'single_index'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ]
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'perl_range'
                                                               ]
                                                  }
                                                ],
                                  'return_annotation' => [
                                                           'return_scalar',
                                                           '$1'
                                                         ]
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'python_slice'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'return_annotation' => [
                                                           'return_scalar',
                                                           '$1'
                                                         ]
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'python_slice_with_step'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'return_annotation' => [
                                                           'return_scalar',
                                                           '$1'
                                                         ]
                                },
                                {
                                  'return_annotation' => [
                                                           'return_object',
                                                           '{type: "multi_index", indices: $1}'
                                                         ],
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'index_list'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'mixed_expression'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'return_annotation' => [
                                                           'return_object',
                                                           '{type: "mixed_expression", elements: $1}'
                                                         ]
                                }
                              ]
          },
          {
            'elements' => [
                            {
                              'value' => [
                                           'regex',
                                           '(?=\\])'
                                         ],
                              'type' => 'atom'
                            }
                          ],
            'type' => 'sequence',
            'name' => 'empty_spec'
          },
          {
            'name' => 'star_spec',
            'elements' => [
                            {
                              'value' => [
                                           'quoted_string',
                                           '*'
                                         ],
                              'type' => 'atom'
                            }
                          ],
            'type' => 'sequence'
          },
          {
            'name' => 'colon_spec',
            'type' => 'sequence',
            'elements' => [
                            {
                              'type' => 'atom',
                              'value' => [
                                           'quoted_string',
                                           ':'
                                         ]
                            }
                          ]
          },
          {
            'name' => 'single_index',
            'type' => 'sequence',
            'elements' => [
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'index'
                                         ]
                            }
                          ],
            'return_annotation' => [
                                     'return_object',
                                     '{type: "single_index", value: $1}'
                                   ]
          },
          {
            'return_annotation' => [
                                     'return_object',
                                     '{type: "perl_range", start: $1, end: $3}'
                                   ],
            'elements' => [
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'index'
                                         ]
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'quoted_string',
                                           '..'
                                         ]
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'index'
                                         ]
                            }
                          ],
            'type' => 'sequence',
            'name' => 'perl_range'
          },
          {
            'return_annotation' => [
                                     'return_object',
                                     '{type: "python_slice", start: $1, end: $3}'
                                   ],
            'name' => 'python_slice',
            'elements' => [
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'python_slice_start'
                                         ]
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'quoted_string',
                                           ':'
                                         ]
                            },
                            {
                              'value' => [
                                           'rule_reference',
                                           'python_slice_end'
                                         ],
                              'type' => 'atom'
                            }
                          ],
            'type' => 'sequence'
          },
          {
            'name' => 'python_slice_with_step',
            'elements' => [
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'python_slice_start'
                                         ]
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'quoted_string',
                                           ':'
                                         ]
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'python_slice_end'
                                         ]
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'quoted_string',
                                           ':'
                                         ]
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'step'
                                         ]
                            }
                          ],
            'type' => 'sequence',
            'return_annotation' => [
                                     'return_object',
                                     '{type: "python_slice_step", start: $1, end: $3, step: $5}'
                                   ]
          },
          {
            'type' => 'or',
            'alternatives' => [
                                {
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'index'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'empty_slice_part'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'type' => 'sequence'
                                }
                              ],
            'name' => 'python_slice_start'
          },
          {
            'name' => 'python_slice_end',
            'alternatives' => [
                                {
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'index'
                                                               ]
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'empty_slice_part'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'type' => 'sequence'
                                }
                              ],
            'type' => 'or'
          },
          {
            'name' => 'step',
            'type' => 'sequence',
            'elements' => [
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'index'
                                         ]
                            }
                          ]
          },
          {
            'elements' => [
                            {
                              'type' => 'atom',
                              'value' => [
                                           'regex',
                                           '(?=:)'
                                         ]
                            }
                          ],
            'type' => 'sequence',
            'name' => 'empty_slice_part'
          },
          {
            'elements' => [
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'index'
                                         ]
                            },
                            {
                              'type' => 'quantified',
                              'element' => {
                                             'elements' => [
                                                             [
                                                               'quoted_string',
                                                               ','
                                                             ],
                                                             [
                                                               'regex',
                                                               '\\s*'
                                                             ],
                                                             [
                                                               'rule_reference',
                                                               'index'
                                                             ]
                                                           ],
                                             'type' => 'sequence'
                                           },
                              'quantifier' => '*'
                            }
                          ],
            'type' => 'sequence',
            'name' => 'index_list',
            'return_annotation' => [
                                     'return_array',
                                     '[$1, $3*]'
                                   ]
          },
          {
            'name' => 'mixed_expression',
            'type' => 'sequence',
            'elements' => [
                            {
                              'value' => [
                                           'rule_reference',
                                           'mixed_element'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'type' => 'quantified',
                              'element' => {
                                             'elements' => [
                                                             [
                                                               'quoted_string',
                                                               ','
                                                             ],
                                                             [
                                                               'regex',
                                                               '\\s*'
                                                             ],
                                                             [
                                                               'rule_reference',
                                                               'mixed_element'
                                                             ]
                                                           ],
                                             'type' => 'sequence'
                                           },
                              'quantifier' => '*'
                            }
                          ],
            'return_annotation' => [
                                     'return_array',
                                     '[$1, $3*]'
                                   ]
          },
          {
            'name' => 'mixed_element',
            'alternatives' => [
                                {
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'single_index'
                                                               ]
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'perl_range'
                                                               ]
                                                  }
                                                ]
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'python_slice'
                                                               ]
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'python_slice_with_step'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'type' => 'sequence'
                                }
                              ],
            'type' => 'or'
          },
          {
            'type' => 'or',
            'alternatives' => [
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'positive_number'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ]
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'negative_number'
                                                               ]
                                                  }
                                                ],
                                  'type' => 'sequence'
                                }
                              ],
            'name' => 'index'
          },
          {
            'elements' => [
                            {
                              'type' => 'atom',
                              'value' => [
                                           'regex',
                                           '(\\d+)'
                                         ]
                            }
                          ],
            'type' => 'sequence',
            'name' => 'positive_number',
            'return_annotation' => [
                                     'return_object',
                                     '{type: "positive", value: $1}'
                                   ]
          },
          {
            'return_annotation' => [
                                     'return_object',
                                     '{type: "negative", value: $2}'
                                   ],
            'elements' => [
                            {
                              'type' => 'atom',
                              'value' => [
                                           'quoted_string',
                                           '-'
                                         ]
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'positive_number'
                                         ]
                            }
                          ],
            'type' => 'sequence',
            'name' => 'negative_number'
          },
          {
            'type' => 'sequence',
            'elements' => [
                            {
                              'type' => 'atom',
                              'value' => [
                                           'quoted_string',
                                           '['
                                         ]
                            },
                            {
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'value' => [
                                           'rule_reference',
                                           'grouped_element_list'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ]
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'quoted_string',
                                           ']'
                                         ]
                            }
                          ],
            'name' => 'grouped_quantified_array',
            'return_annotation' => [
                                     'return_object',
                                     '{type: "grouped_quantified_array", groups: $3}'
                                   ]
          },
          {
            'type' => 'or',
            'alternatives' => [
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'grouped_element_item'
                                                               ],
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'quantifier' => '*',
                                                    'element' => {
                                                                   'type' => 'sequence',
                                                                   'elements' => [
                                                                                   [
                                                                                     'quoted_string',
                                                                                     ','
                                                                                   ],
                                                                                   [
                                                                                     'regex',
                                                                                     '\\s*'
                                                                                   ],
                                                                                   [
                                                                                     'rule_reference',
                                                                                     'grouped_element_item'
                                                                                   ]
                                                                                 ]
                                                                 },
                                                    'type' => 'quantified'
                                                  }
                                                ],
                                  'return_annotation' => [
                                                           'return_array',
                                                           '[$1, $3*]'
                                                         ]
                                },
                                {
                                  'return_annotation' => [
                                                           'return_array',
                                                           '[$1]'
                                                         ],
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'grouped_element_item'
                                                               ]
                                                  }
                                                ]
                                }
                              ],
            'name' => 'grouped_element_list'
          },
          {
            'type' => 'or',
            'alternatives' => [
                                {
                                  'return_annotation' => [
                                                           'return_object',
                                                           '{type: "grouped_quantified", content: $3, quantifier: $6}'
                                                         ],
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'quoted_string',
                                                                 '('
                                                               ],
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'value' => [
                                                                 'regex',
                                                                 '\\s*'
                                                               ],
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'group_content'
                                                               ]
                                                  },
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'regex',
                                                                 '\\s*'
                                                               ]
                                                  },
                                                  {
                                                    'value' => [
                                                                 'quoted_string',
                                                                 ')'
                                                               ],
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'quantifier'
                                                               ]
                                                  }
                                                ]
                                },
                                {
                                  'return_annotation' => [
                                                           'return_object',
                                                           '{type: "sequence_quantified", content: $1, quantifier: $2}'
                                                         ],
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'element_sequence'
                                                               ],
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'quantifier'
                                                               ]
                                                  }
                                                ],
                                  'type' => 'sequence'
                                }
                              ],
            'name' => 'grouped_element_item'
          },
          {
            'type' => 'sequence',
            'elements' => [
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'element_sequence'
                                         ]
                            }
                          ],
            'name' => 'group_content'
          },
          {
            'type' => 'or',
            'name' => 'element_sequence',
            'alternatives' => [
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'element_item'
                                                               ],
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'quantifier' => '*',
                                                    'element' => {
                                                                   'elements' => [
                                                                                   [
                                                                                     'regex',
                                                                                     '\\s+'
                                                                                   ],
                                                                                   [
                                                                                     'rule_reference',
                                                                                     'element_item'
                                                                                   ]
                                                                                 ],
                                                                   'type' => 'sequence'
                                                                 },
                                                    'type' => 'quantified'
                                                  }
                                                ],
                                  'return_annotation' => [
                                                           'return_array',
                                                           '[$1, $3*]'
                                                         ]
                                },
                                {
                                  'return_annotation' => [
                                                           'return_array',
                                                           '[$1]'
                                                         ],
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'element_item'
                                                               ]
                                                  }
                                                ],
                                  'type' => 'sequence'
                                }
                              ]
          },
          {
            'type' => 'or',
            'alternatives' => [
                                {
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'identifier'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'literal'
                                                               ]
                                                  }
                                                ]
                                }
                              ],
            'name' => 'element_item'
          },
          {
            'type' => 'sequence',
            'elements' => [
                            {
                              'type' => 'atom',
                              'value' => [
                                           'quoted_string',
                                           '{'
                                         ]
                            },
                            {
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'value' => [
                                           'rule_reference',
                                           'outer_key'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ]
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'quoted_string',
                                           ':'
                                         ]
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ]
                            },
                            {
                              'value' => [
                                           'rule_reference',
                                           'inner_object'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ]
                            },
                            {
                              'value' => [
                                           'quoted_string',
                                           '}'
                                         ],
                              'type' => 'atom'
                            }
                          ],
            'name' => 'simple_nested_object',
            'return_annotation' => [
                                     'return_object',
                                     '{type: "nested_object", key: $3, value: $7}'
                                   ]
          },
          {
            'return_annotation' => [
                                     'return_object',
                                     '{type: "inner_object", key: $3, value: $7}'
                                   ],
            'type' => 'sequence',
            'elements' => [
                            {
                              'type' => 'atom',
                              'value' => [
                                           'quoted_string',
                                           '{'
                                         ]
                            },
                            {
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'inner_key'
                                         ]
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ]
                            },
                            {
                              'value' => [
                                           'quoted_string',
                                           ':'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'inner_value'
                                         ]
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ]
                            },
                            {
                              'value' => [
                                           'quoted_string',
                                           '}'
                                         ],
                              'type' => 'atom'
                            }
                          ],
            'name' => 'inner_object'
          },
          {
            'type' => 'or',
            'name' => 'inner_value',
            'alternatives' => [
                                {
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'nested_array'
                                                               ]
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'nested_object'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ]
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'grouped_quantified_array'
                                                               ]
                                                  }
                                                ]
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'quantified_array'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ]
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'simple_array'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'ultimate_dot_notation'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'scalar_ref'
                                                               ]
                                                  }
                                                ]
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'literal'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ]
                                }
                              ]
          },
          {
            'type' => 'or',
            'name' => 'multi_property_object',
            'alternatives' => [
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'two_property_object'
                                                               ]
                                                  }
                                                ]
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'three_property_object'
                                                               ]
                                                  }
                                                ],
                                  'type' => 'sequence'
                                }
                              ]
          },
          {
            'return_annotation' => [
                                     'return_object',
                                     '{type: "multi_object", prop1: $3, prop2: $6}'
                                   ],
            'name' => 'two_property_object',
            'type' => 'sequence',
            'elements' => [
                            {
                              'value' => [
                                           'quoted_string',
                                           '{'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ]
                            },
                            {
                              'value' => [
                                           'rule_reference',
                                           'property'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'value' => [
                                           'quoted_string',
                                           ','
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'value' => [
                                           'rule_reference',
                                           'property'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'value' => [
                                           'quoted_string',
                                           '}'
                                         ],
                              'type' => 'atom'
                            }
                          ]
          },
          {
            'elements' => [
                            {
                              'value' => [
                                           'quoted_string',
                                           '{'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ]
                            },
                            {
                              'value' => [
                                           'rule_reference',
                                           'property'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'value' => [
                                           'quoted_string',
                                           ','
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ]
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'property'
                                         ]
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'quoted_string',
                                           ','
                                         ]
                            },
                            {
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'property'
                                         ]
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ]
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'quoted_string',
                                           '}'
                                         ]
                            }
                          ],
            'type' => 'sequence',
            'name' => 'three_property_object',
            'return_annotation' => [
                                     'return_object',
                                     '{type: "multi_object", prop1: $3, prop2: $6, prop3: $9}'
                                   ]
          },
          {
            'return_annotation' => [
                                     'return_object',
                                     '{key: $1, value: $5}'
                                   ],
            'name' => 'property',
            'elements' => [
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'object_key'
                                         ]
                            },
                            {
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'value' => [
                                           'quoted_string',
                                           ':'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'property_value'
                                         ]
                            }
                          ],
            'type' => 'sequence'
          },
          {
            'type' => 'or',
            'alternatives' => [
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'nested_array'
                                                               ]
                                                  }
                                                ]
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'nested_object'
                                                               ]
                                                  }
                                                ]
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'grouped_quantified_array'
                                                               ]
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'quantified_array'
                                                               ]
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'simple_array'
                                                               ]
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'ultimate_dot_notation'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'scalar_ref'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'literal'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'type' => 'sequence'
                                }
                              ],
            'name' => 'property_value'
          },
          {
            'elements' => [
                            {
                              'value' => [
                                           'quoted_string',
                                           '['
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'quantified_element'
                                         ]
                            },
                            {
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'value' => [
                                           'quoted_string',
                                           ']'
                                         ],
                              'type' => 'atom'
                            }
                          ],
            'type' => 'sequence',
            'name' => 'quantified_array',
            'return_annotation' => [
                                     'return_object',
                                     '{type: "quantified_array", element: $3}'
                                   ]
          },
          {
            'type' => 'or',
            'alternatives' => [
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'ultimate_dot_notation'
                                                               ],
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'quantifier'
                                                               ]
                                                  }
                                                ],
                                  'return_annotation' => [
                                                           'return_object',
                                                           '{scalar: $1, quantifier: $2}'
                                                         ]
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'scalar_ref'
                                                               ]
                                                  },
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'quantifier'
                                                               ]
                                                  }
                                                ],
                                  'return_annotation' => [
                                                           'return_object',
                                                           '{scalar: $1, quantifier: $2}'
                                                         ]
                                }
                              ],
            'name' => 'quantified_element'
          },
          {
            'type' => 'or',
            'name' => 'quantifier',
            'alternatives' => [
                                {
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'quoted_string',
                                                                 '*'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'type' => 'sequence',
                                  'return_annotation' => [
                                                           'return_scalar',
                                                           '"*"'
                                                         ]
                                },
                                {
                                  'return_annotation' => [
                                                           'return_scalar',
                                                           '"+"'
                                                         ],
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'quoted_string',
                                                                 '+'
                                                               ]
                                                  }
                                                ]
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'quoted_string',
                                                                 '?'
                                                               ]
                                                  }
                                                ],
                                  'type' => 'sequence',
                                  'return_annotation' => [
                                                           'return_scalar',
                                                           '"?"'
                                                         ]
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'quoted_string',
                                                                 '{'
                                                               ]
                                                  },
                                                  {
                                                    'value' => [
                                                                 'regex',
                                                                 '\\s*'
                                                               ],
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'positive_number'
                                                               ],
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'value' => [
                                                                 'regex',
                                                                 '\\s*'
                                                               ],
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'value' => [
                                                                 'quoted_string',
                                                                 '}'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'type' => 'sequence',
                                  'return_annotation' => [
                                                           'return_object',
                                                           '{min: $3, max: $3}'
                                                         ]
                                },
                                {
                                  'return_annotation' => [
                                                           'return_object',
                                                           '{min: $3, max: "inf"}'
                                                         ],
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'quoted_string',
                                                                 '{'
                                                               ],
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'value' => [
                                                                 'regex',
                                                                 '\\s*'
                                                               ],
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'positive_number'
                                                               ]
                                                  },
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'regex',
                                                                 '\\s*'
                                                               ]
                                                  },
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'quoted_string',
                                                                 ','
                                                               ]
                                                  },
                                                  {
                                                    'value' => [
                                                                 'regex',
                                                                 '\\s*'
                                                               ],
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'quoted_string',
                                                                 '}'
                                                               ]
                                                  }
                                                ]
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'quoted_string',
                                                                 '{'
                                                               ]
                                                  },
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'regex',
                                                                 '\\s*'
                                                               ]
                                                  },
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'positive_number'
                                                               ],
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'value' => [
                                                                 'regex',
                                                                 '\\s*'
                                                               ],
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'quoted_string',
                                                                 ','
                                                               ]
                                                  },
                                                  {
                                                    'value' => [
                                                                 'regex',
                                                                 '\\s*'
                                                               ],
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'positive_number'
                                                               ]
                                                  },
                                                  {
                                                    'value' => [
                                                                 'regex',
                                                                 '\\s*'
                                                               ],
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'value' => [
                                                                 'quoted_string',
                                                                 '}'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'return_annotation' => [
                                                           'return_object',
                                                           '{min: $3, max: $7}'
                                                         ]
                                },
                                {
                                  'return_annotation' => [
                                                           'return_object',
                                                           '{min: 0, max: $5}'
                                                         ],
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'quoted_string',
                                                                 '{'
                                                               ],
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'value' => [
                                                                 'regex',
                                                                 '\\s*'
                                                               ],
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'value' => [
                                                                 'quoted_string',
                                                                 ','
                                                               ],
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'value' => [
                                                                 'regex',
                                                                 '\\s*'
                                                               ],
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'positive_number'
                                                               ]
                                                  },
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'regex',
                                                                 '\\s*'
                                                               ]
                                                  },
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'quoted_string',
                                                                 '}'
                                                               ]
                                                  }
                                                ],
                                  'type' => 'sequence'
                                }
                              ]
          },
          {
            'return_annotation' => [
                                     'return_object',
                                     '{type: "array", element: $3}'
                                   ],
            'elements' => [
                            {
                              'value' => [
                                           'quoted_string',
                                           '['
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ]
                            },
                            {
                              'value' => [
                                           'rule_reference',
                                           'array_element'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ]
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'quoted_string',
                                           ']'
                                         ]
                            }
                          ],
            'type' => 'sequence',
            'name' => 'simple_array'
          },
          {
            'type' => 'or',
            'alternatives' => [
                                {
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'ultimate_dot_notation'
                                                               ]
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'scalar_ref'
                                                               ]
                                                  }
                                                ]
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'literal'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ]
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'nested_array'
                                                               ]
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'nested_object'
                                                               ]
                                                  }
                                                ],
                                  'type' => 'sequence'
                                }
                              ],
            'name' => 'array_element'
          },
          {
            'name' => 'simple_object',
            'elements' => [
                            {
                              'value' => [
                                           'quoted_string',
                                           '{'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'object_key'
                                         ]
                            },
                            {
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'value' => [
                                           'quoted_string',
                                           ':'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'object_value'
                                         ]
                            },
                            {
                              'value' => [
                                           'regex',
                                           '\\s*'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'quoted_string',
                                           '}'
                                         ]
                            }
                          ],
            'type' => 'sequence',
            'return_annotation' => [
                                     'return_object',
                                     '{type: "object", key: $3, value: $7}'
                                   ]
          },
          {
            'type' => 'or',
            'alternatives' => [
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'identifier'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ]
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'quoted_string'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ]
                                }
                              ],
            'name' => 'object_key'
          },
          {
            'type' => 'or',
            'alternatives' => [
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'identifier'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ]
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'quoted_string'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ]
                                }
                              ],
            'name' => 'outer_key'
          },
          {
            'type' => 'or',
            'alternatives' => [
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'identifier'
                                                               ]
                                                  }
                                                ]
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'quoted_string'
                                                               ]
                                                  }
                                                ],
                                  'type' => 'sequence'
                                }
                              ],
            'name' => 'inner_key'
          },
          {
            'type' => 'or',
            'name' => 'object_value',
            'alternatives' => [
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'ultimate_dot_notation'
                                                               ]
                                                  }
                                                ]
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'scalar_ref'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'literal'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'nested_array'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ]
                                },
                                {
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'nested_object'
                                                               ]
                                                  }
                                                ],
                                  'type' => 'sequence'
                                }
                              ]
          },
          {
            'alternatives' => [
                                {
                                  'elements' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'quoted_string'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
                                {
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'number'
                                                               ]
                                                  }
                                                ]
                                }
                              ],
            'name' => 'literal',
            'type' => 'or'
          },
          {
            'name' => 'scalar_ref',
            'type' => 'sequence',
            'elements' => [
                            {
                              'value' => [
                                           'quoted_string',
                                           '$'
                                         ],
                              'type' => 'atom'
                            },
                            {
                              'type' => 'atom',
                              'value' => [
                                           'rule_reference',
                                           'positive_number'
                                         ]
                            }
                          ],
            'return_annotation' => [
                                     'return_object',
                                     '{type: "scalar_ref", index: $2}'
                                   ]
          },
          {
            'return_annotation' => [
                                     'return_scalar',
                                     '$1'
                                   ],
            'name' => 'quoted_string',
            'type' => 'sequence',
            'elements' => [
                            {
                              'type' => 'atom',
                              'value' => [
                                           'regex',
                                           '"([^"]*)"'
                                         ]
                            }
                          ]
          },
          {
            'name' => 'number',
            'elements' => [
                            {
                              'value' => [
                                           'regex',
                                           '(\\d+)'
                                         ],
                              'type' => 'atom'
                            }
                          ],
            'type' => 'sequence',
            'return_annotation' => [
                                     'return_scalar',
                                     '$1'
                                   ]
          },
          {
            'name' => 'identifier',
            'type' => 'sequence',
            'elements' => [
                            {
                              'value' => [
                                           'regex',
                                           '([a-zA-Z_]\\w*)'
                                         ],
                              'type' => 'atom'
                            }
                          ],
            'return_annotation' => [
                                     'return_scalar',
                                     '$1'
                                   ]
          }
        ];
DEBUG: index_list after step4:
$VAR1 = {
          'elements' => [
                          {
                            'type' => 'atom',
                            'value' => [
                                         'rule_reference',
                                         'index'
                                       ]
                          },
                          {
                            'type' => 'quantified',
                            'element' => {
                                           'elements' => [
                                                           [
                                                             'quoted_string',
                                                             ','
                                                           ],
                                                           [
                                                             'regex',
                                                             '\\s*'
                                                           ],
                                                           [
                                                             'rule_reference',
                                                             'index'
                                                           ]
                                                         ],
                                           'type' => 'sequence'
                                         },
                            'quantifier' => '*'
                          }
                        ],
          'type' => 'sequence',
          'name' => 'index_list',
          'return_annotation' => [
                                   'return_array',
                                   '[$1, $3*]'
                                 ]
        };

=== Step 5: Build tree structure ===
DEBUG: index_list before step5:
$VAR1 = {
          'elements' => [
                          {
                            'type' => 'atom',
                            'value' => [
                                         'rule_reference',
                                         'index'
                                       ]
                          },
                          {
                            'type' => 'quantified',
                            'element' => {
                                           'elements' => [
                                                           [
                                                             'quoted_string',
                                                             ','
                                                           ],
                                                           [
                                                             'regex',
                                                             '\\s*'
                                                           ],
                                                           [
                                                             'rule_reference',
                                                             'index'
                                                           ]
                                                         ],
                                           'type' => 'sequence'
                                         },
                            'quantifier' => '*'
                          }
                        ],
          'type' => 'sequence',
          'name' => 'index_list',
          'return_annotation' => [
                                   'return_array',
                                   '[$1, $3*]'
                                 ]
        };

🔍 STEP 5 DEBUG: Looking for dot_path rule in input...
🎯 STEP 5: Found dot_path rule in input: $VAR1 = {
          'return_annotation' => [
                                   'return_array',
                                   '[$1*]'
                                 ],
          'name' => 'dot_path',
          'elements' => [
                          {
                            'type' => 'quantified',
                            'element' => [
                                           'rule_reference',
                                           'accessor'
                                         ],
                            'quantifier' => '+'
                          }
                        ],
          'type' => 'sequence'
        };

✅ STEP 5: dot_path rule found in output: $VAR1 = {
          'element' => {
                         'type' => 'atom',
                         'value' => [
                                      'rule_reference',
                                      'accessor'
                                    ]
                       },
          'quantifier' => '+',
          'return_annotation' => [
                                   'return_array',
                                   '[$1*]'
                                 ],
          'type' => 'quantified'
        };

STEP 5 RESULT (Tree structure):
$VAR1 = {
          'simple_object' => {
                               'return_annotation' => [
                                                        'return_object',
                                                        '{type: "object", key: $3, value: $7}'
                                                      ],
                               'type' => 'sequence',
                               'elements' => [
                                               {
                                                 'value' => {
                                                              'value' => [
                                                                           'quoted_string',
                                                                           '{'
                                                                         ],
                                                              'type' => 'atom'
                                                            },
                                                 'type' => 'atom'
                                               },
                                               {
                                                 'value' => {
                                                              'value' => [
                                                                           'regex',
                                                                           '\\s*'
                                                                         ],
                                                              'type' => 'atom'
                                                            },
                                                 'type' => 'atom'
                                               },
                                               {
                                                 'type' => 'atom',
                                                 'value' => {
                                                              'type' => 'atom',
                                                              'value' => [
                                                                           'rule_reference',
                                                                           'object_key'
                                                                         ]
                                                            }
                                               },
                                               {
                                                 'type' => 'atom',
                                                 'value' => {
                                                              'value' => [
                                                                           'regex',
                                                                           '\\s*'
                                                                         ],
                                                              'type' => 'atom'
                                                            }
                                               },
                                               {
                                                 'value' => {
                                                              'value' => [
                                                                           'quoted_string',
                                                                           ':'
                                                                         ],
                                                              'type' => 'atom'
                                                            },
                                                 'type' => 'atom'
                                               },
                                               {
                                                 'value' => {
                                                              'value' => [
                                                                           'regex',
                                                                           '\\s*'
                                                                         ],
                                                              'type' => 'atom'
                                                            },
                                                 'type' => 'atom'
                                               },
                                               {
                                                 'type' => 'atom',
                                                 'value' => {
                                                              'type' => 'atom',
                                                              'value' => [
                                                                           'rule_reference',
                                                                           'object_value'
                                                                         ]
                                                            }
                                               },
                                               {
                                                 'value' => {
                                                              'value' => [
                                                                           'regex',
                                                                           '\\s*'
                                                                         ],
                                                              'type' => 'atom'
                                                            },
                                                 'type' => 'atom'
                                               },
                                               {
                                                 'type' => 'atom',
                                                 'value' => {
                                                              'type' => 'atom',
                                                              'value' => [
                                                                           'quoted_string',
                                                                           '}'
                                                                         ]
                                                            }
                                               }
                                             ]
                             },
          'python_slice_end' => {
                                  'alternatives' => [
                                                      {
                                                        'value' => {
                                                                     'type' => 'atom',
                                                                     'value' => [
                                                                                  'rule_reference',
                                                                                  'index'
                                                                                ]
                                                                   },
                                                        'type' => 'atom'
                                                      },
                                                      {
                                                        'type' => 'atom',
                                                        'value' => {
                                                                     'value' => [
                                                                                  'rule_reference',
                                                                                  'empty_slice_part'
                                                                                ],
                                                                     'type' => 'atom'
                                                                   }
                                                      }
                                                    ],
                                  'type' => 'or'
                                },
          'quoted_string' => {
                               'return_annotation' => [
                                                        'return_scalar',
                                                        '$1'
                                                      ],
                               'value' => {
                                            'type' => 'atom',
                                            'value' => [
                                                         'regex',
                                                         '"([^"]*)"'
                                                       ]
                                          },
                               'type' => 'atom'
                             },
          'array_accessor' => {
                                'elements' => [
                                                {
                                                  'type' => 'atom',
                                                  'value' => {
                                                               'value' => [
                                                                            'quoted_string',
                                                                            '['
                                                                          ],
                                                               'type' => 'atom'
                                                             }
                                                },
                                                {
                                                  'type' => 'atom',
                                                  'value' => {
                                                               'type' => 'atom',
                                                               'value' => [
                                                                            'rule_reference',
                                                                            'array_spec'
                                                                          ]
                                                             }
                                                },
                                                {
                                                  'value' => {
                                                               'value' => [
                                                                            'quoted_string',
                                                                            ']'
                                                                          ],
                                                               'type' => 'atom'
                                                             },
                                                  'type' => 'atom'
                                                }
                                              ],
                                'type' => 'sequence',
                                'return_annotation' => [
                                                         'return_object',
                                                         '{type: "array_access", spec: $2}'
                                                       ]
                              },
          'grouped_quantified_array' => {
                                          'return_annotation' => [
                                                                   'return_object',
                                                                   '{type: "grouped_quantified_array", groups: $3}'
                                                                 ],
                                          'elements' => [
                                                          {
                                                            'type' => 'atom',
                                                            'value' => {
                                                                         'type' => 'atom',
                                                                         'value' => [
                                                                                      'quoted_string',
                                                                                      '['
                                                                                    ]
                                                                       }
                                                          },
                                                          {
                                                            'value' => {
                                                                         'value' => [
                                                                                      'regex',
                                                                                      '\\s*'
                                                                                    ],
                                                                         'type' => 'atom'
                                                                       },
                                                            'type' => 'atom'
                                                          },
                                                          {
                                                            'value' => {
                                                                         'value' => [
                                                                                      'rule_reference',
                                                                                      'grouped_element_list'
                                                                                    ],
                                                                         'type' => 'atom'
                                                                       },
                                                            'type' => 'atom'
                                                          },
                                                          {
                                                            'value' => {
                                                                         'type' => 'atom',
                                                                         'value' => [
                                                                                      'regex',
                                                                                      '\\s*'
                                                                                    ]
                                                                       },
                                                            'type' => 'atom'
                                                          },
                                                          {
                                                            'value' => {
                                                                         'type' => 'atom',
                                                                         'value' => [
                                                                                      'quoted_string',
                                                                                      ']'
                                                                                    ]
                                                                       },
                                                            'type' => 'atom'
                                                          }
                                                        ],
                                          'type' => 'sequence'
                                        },
          'return_annotation' => {
                                   'elements' => [
                                                   {
                                                     'type' => 'atom',
                                                     'value' => {
                                                                  'value' => [
                                                                               'quoted_string',
                                                                               '->'
                                                                             ],
                                                                  'type' => 'atom'
                                                                }
                                                   },
                                                   {
                                                     'value' => {
                                                                  'value' => [
                                                                               'regex',
                                                                               '\\s*'
                                                                             ],
                                                                  'type' => 'atom'
                                                                },
                                                     'type' => 'atom'
                                                   },
                                                   {
                                                     'type' => 'atom',
                                                     'value' => {
                                                                  'type' => 'atom',
                                                                  'value' => [
                                                                               'rule_reference',
                                                                               'return_expression'
                                                                             ]
                                                                }
                                                   }
                                                 ],
                                   'type' => 'sequence'
                                 },
          'outer_key' => {
                           'alternatives' => [
                                               {
                                                 'value' => {
                                                              'value' => [
                                                                           'rule_reference',
                                                                           'identifier'
                                                                         ],
                                                              'type' => 'atom'
                                                            },
                                                 'type' => 'atom'
                                               },
                                               {
                                                 'value' => {
                                                              'value' => [
                                                                           'rule_reference',
                                                                           'quoted_string'
                                                                         ],
                                                              'type' => 'atom'
                                                            },
                                                 'type' => 'atom'
                                               }
                                             ],
                           'type' => 'or'
                         },
          'object_contents' => {
                                 'type' => 'sequence',
                                 'elements' => [
                                                 {
                                                   'value' => {
                                                                'value' => [
                                                                             'rule_reference',
                                                                             'object_pair'
                                                                           ],
                                                                'type' => 'atom'
                                                              },
                                                   'type' => 'atom'
                                                 },
                                                 {
                                                   'quantifier' => '*',
                                                   'element' => {
                                                                  'type' => 'atom',
                                                                  'value' => {
                                                                               'type' => 'sequence',
                                                                               'elements' => [
                                                                                               [
                                                                                                 'quoted_string',
                                                                                                 ','
                                                                                               ],
                                                                                               [
                                                                                                 'regex',
                                                                                                 '\\s*'
                                                                                               ],
                                                                                               [
                                                                                                 'rule_reference',
                                                                                                 'object_pair'
                                                                                               ]
                                                                                             ]
                                                                             }
                                                                },
                                                   'type' => 'quantified'
                                                 }
                                               ],
                                 'return_annotation' => [
                                                          'return_array',
                                                          '[$1, $3*]'
                                                        ]
                               },
          'array_contents' => {
                                'return_annotation' => [
                                                         'return_array',
                                                         '[$1, $3*]'
                                                       ],
                                'type' => 'sequence',
                                'elements' => [
                                                {
                                                  'value' => {
                                                               'type' => 'atom',
                                                               'value' => [
                                                                            'rule_reference',
                                                                            'return_expression'
                                                                          ]
                                                             },
                                                  'type' => 'atom'
                                                },
                                                {
                                                  'type' => 'quantified',
                                                  'quantifier' => '*',
                                                  'element' => {
                                                                 'value' => {
                                                                              'type' => 'sequence',
                                                                              'elements' => [
                                                                                              [
                                                                                                'quoted_string',
                                                                                                ','
                                                                                              ],
                                                                                              [
                                                                                                'regex',
                                                                                                '\\s*'
                                                                                              ],
                                                                                              [
                                                                                                'rule_reference',
                                                                                                'return_expression'
                                                                                              ]
                                                                                            ]
                                                                            },
                                                                 'type' => 'atom'
                                                               }
                                                }
                                              ]
                              },
          'literal' => {
                         'alternatives' => [
                                             {
                                               'value' => {
                                                            'value' => [
                                                                         'rule_reference',
                                                                         'quoted_string'
                                                                       ],
                                                            'type' => 'atom'
                                                          },
                                               'type' => 'atom'
                                             },
                                             {
                                               'value' => {
                                                            'type' => 'atom',
                                                            'value' => [
                                                                         'rule_reference',
                                                                         'number'
                                                                       ]
                                                          },
                                               'type' => 'atom'
                                             }
                                           ],
                         'type' => 'or'
                       },
          'ultimate_dot_notation' => {
                                       'return_annotation' => [
                                                                'return_object',
                                                                '{type: "ultimate_dot_notation", base: $1, path: $2}'
                                                              ],
                                       'type' => 'sequence',
                                       'elements' => [
                                                       {
                                                         'value' => {
                                                                      'value' => [
                                                                                   'rule_reference',
                                                                                   'scalar_ref'
                                                                                 ],
                                                                      'type' => 'atom'
                                                                    },
                                                         'type' => 'atom'
                                                       },
                                                       {
                                                         'type' => 'atom',
                                                         'value' => {
                                                                      'type' => 'atom',
                                                                      'value' => [
                                                                                   'rule_reference',
                                                                                   'dot_path'
                                                                                 ]
                                                                    }
                                                       }
                                                     ]
                                     },
          'star_spec' => {
                           'value' => {
                                        'value' => [
                                                     'quoted_string',
                                                     '*'
                                                   ],
                                        'type' => 'atom'
                                      },
                           'type' => 'atom'
                         },
          'mixed_element' => {
                               'alternatives' => [
                                                   {
                                                     'value' => {
                                                                  'type' => 'atom',
                                                                  'value' => [
                                                                               'rule_reference',
                                                                               'single_index'
                                                                             ]
                                                                },
                                                     'type' => 'atom'
                                                   },
                                                   {
                                                     'type' => 'atom',
                                                     'value' => {
                                                                  'type' => 'atom',
                                                                  'value' => [
                                                                               'rule_reference',
                                                                               'perl_range'
                                                                             ]
                                                                }
                                                   },
                                                   {
                                                     'type' => 'atom',
                                                     'value' => {
                                                                  'type' => 'atom',
                                                                  'value' => [
                                                                               'rule_reference',
                                                                               'python_slice'
                                                                             ]
                                                                }
                                                   },
                                                   {
                                                     'value' => {
                                                                  'value' => [
                                                                               'rule_reference',
                                                                               'python_slice_with_step'
                                                                             ],
                                                                  'type' => 'atom'
                                                                },
                                                     'type' => 'atom'
                                                   }
                                                 ],
                               'type' => 'or'
                             },
          'array_spec' => {
                            'alternatives' => [
                                                {
                                                  'type' => 'atom',
                                                  'value' => {
                                                               'type' => 'atom',
                                                               'value' => [
                                                                            'rule_reference',
                                                                            'empty_spec'
                                                                          ]
                                                             },
                                                  'return_annotation' => [
                                                                           'return_object',
                                                                           '{type: "whole_array", style: "implicit"}'
                                                                         ]
                                                },
                                                {
                                                  'return_annotation' => [
                                                                           'return_object',
                                                                           '{type: "whole_array", style: "bash"}'
                                                                         ],
                                                  'value' => {
                                                               'value' => [
                                                                            'rule_reference',
                                                                            'star_spec'
                                                                          ],
                                                               'type' => 'atom'
                                                             },
                                                  'type' => 'atom'
                                                },
                                                {
                                                  'value' => {
                                                               'value' => [
                                                                            'rule_reference',
                                                                            'colon_spec'
                                                                          ],
                                                               'type' => 'atom'
                                                             },
                                                  'type' => 'atom',
                                                  'return_annotation' => [
                                                                           'return_object',
                                                                           '{type: "whole_array", style: "python"}'
                                                                         ]
                                                },
                                                {
                                                  'type' => 'atom',
                                                  'value' => {
                                                               'value' => [
                                                                            'rule_reference',
                                                                            'single_index'
                                                                          ],
                                                               'type' => 'atom'
                                                             },
                                                  'return_annotation' => [
                                                                           'return_scalar',
                                                                           '$1'
                                                                         ]
                                                },
                                                {
                                                  'type' => 'atom',
                                                  'value' => {
                                                               'type' => 'atom',
                                                               'value' => [
                                                                            'rule_reference',
                                                                            'perl_range'
                                                                          ]
                                                             },
                                                  'return_annotation' => [
                                                                           'return_scalar',
                                                                           '$1'
                                                                         ]
                                                },
                                                {
                                                  'type' => 'atom',
                                                  'value' => {
                                                               'value' => [
                                                                            'rule_reference',
                                                                            'python_slice'
                                                                          ],
                                                               'type' => 'atom'
                                                             },
                                                  'return_annotation' => [
                                                                           'return_scalar',
                                                                           '$1'
                                                                         ]
                                                },
                                                {
                                                  'return_annotation' => [
                                                                           'return_scalar',
                                                                           '$1'
                                                                         ],
                                                  'type' => 'atom',
                                                  'value' => {
                                                               'value' => [
                                                                            'rule_reference',
                                                                            'python_slice_with_step'
                                                                          ],
                                                               'type' => 'atom'
                                                             }
                                                },
                                                {
                                                  'type' => 'atom',
                                                  'value' => {
                                                               'value' => [
                                                                            'rule_reference',
                                                                            'index_list'
                                                                          ],
                                                               'type' => 'atom'
                                                             },
                                                  'return_annotation' => [
                                                                           'return_object',
                                                                           '{type: "multi_index", indices: $1}'
                                                                         ]
                                                },
                                                {
                                                  'value' => {
                                                               'value' => [
                                                                            'rule_reference',
                                                                            'mixed_expression'
                                                                          ],
                                                               'type' => 'atom'
                                                             },
                                                  'type' => 'atom',
                                                  'return_annotation' => [
                                                                           'return_object',
                                                                           '{type: "mixed_expression", elements: $1}'
                                                                         ]
                                                }
                                              ],
                            'type' => 'or'
                          },
          'grouped_element_item' => {
                                      'alternatives' => [
                                                          {
                                                            'return_annotation' => [
                                                                                     'return_object',
                                                                                     '{type: "grouped_quantified", content: $3, quantifier: $6}'
                                                                                   ],
                                                            'type' => 'sequence',
                                                            'elements' => [
                                                                            {
                                                                              'value' => {
                                                                                           'value' => [
                                                                                                        'quoted_string',
                                                                                                        '('
                                                                                                      ],
                                                                                           'type' => 'atom'
                                                                                         },
                                                                              'type' => 'atom'
                                                                            },
                                                                            {
                                                                              'type' => 'atom',
                                                                              'value' => {
                                                                                           'value' => [
                                                                                                        'regex',
                                                                                                        '\\s*'
                                                                                                      ],
                                                                                           'type' => 'atom'
                                                                                         }
                                                                            },
                                                                            {
                                                                              'type' => 'atom',
                                                                              'value' => {
                                                                                           'type' => 'atom',
                                                                                           'value' => [
                                                                                                        'rule_reference',
                                                                                                        'group_content'
                                                                                                      ]
                                                                                         }
                                                                            },
                                                                            {
                                                                              'value' => {
                                                                                           'type' => 'atom',
                                                                                           'value' => [
                                                                                                        'regex',
                                                                                                        '\\s*'
                                                                                                      ]
                                                                                         },
                                                                              'type' => 'atom'
                                                                            },
                                                                            {
                                                                              'value' => {
                                                                                           'value' => [
                                                                                                        'quoted_string',
                                                                                                        ')'
                                                                                                      ],
                                                                                           'type' => 'atom'
                                                                                         },
                                                                              'type' => 'atom'
                                                                            },
                                                                            {
                                                                              'type' => 'atom',
                                                                              'value' => {
                                                                                           'type' => 'atom',
                                                                                           'value' => [
                                                                                                        'rule_reference',
                                                                                                        'quantifier'
                                                                                                      ]
                                                                                         }
                                                                            }
                                                                          ]
                                                          },
                                                          {
                                                            'return_annotation' => [
                                                                                     'return_object',
                                                                                     '{type: "sequence_quantified", content: $1, quantifier: $2}'
                                                                                   ],
                                                            'type' => 'sequence',
                                                            'elements' => [
                                                                            {
                                                                              'type' => 'atom',
                                                                              'value' => {
                                                                                           'value' => [
                                                                                                        'rule_reference',
                                                                                                        'element_sequence'
                                                                                                      ],
                                                                                           'type' => 'atom'
                                                                                         }
                                                                            },
                                                                            {
                                                                              'type' => 'atom',
                                                                              'value' => {
                                                                                           'type' => 'atom',
                                                                                           'value' => [
                                                                                                        'rule_reference',
                                                                                                        'quantifier'
                                                                                                      ]
                                                                                         }
                                                                            }
                                                                          ]
                                                          }
                                                        ],
                                      'type' => 'or'
                                    },
          'mixed_expression' => {
                                  'return_annotation' => [
                                                           'return_array',
                                                           '[$1, $3*]'
                                                         ],
                                  'elements' => [
                                                  {
                                                    'value' => {
                                                                 'value' => [
                                                                              'rule_reference',
                                                                              'mixed_element'
                                                                            ],
                                                                 'type' => 'atom'
                                                               },
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'quantifier' => '*',
                                                    'element' => {
                                                                   'type' => 'atom',
                                                                   'value' => {
                                                                                'elements' => [
                                                                                                [
                                                                                                  'quoted_string',
                                                                                                  ','
                                                                                                ],
                                                                                                [
                                                                                                  'regex',
                                                                                                  '\\s*'
                                                                                                ],
                                                                                                [
                                                                                                  'rule_reference',
                                                                                                  'mixed_element'
                                                                                                ]
                                                                                              ],
                                                                                'type' => 'sequence'
                                                                              }
                                                                 },
                                                    'type' => 'quantified'
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
          'quantified_element' => {
                                    'type' => 'or',
                                    'alternatives' => [
                                                        {
                                                          'return_annotation' => [
                                                                                   'return_object',
                                                                                   '{scalar: $1, quantifier: $2}'
                                                                                 ],
                                                          'elements' => [
                                                                          {
                                                                            'value' => {
                                                                                         'value' => [
                                                                                                      'rule_reference',
                                                                                                      'ultimate_dot_notation'
                                                                                                    ],
                                                                                         'type' => 'atom'
                                                                                       },
                                                                            'type' => 'atom'
                                                                          },
                                                                          {
                                                                            'type' => 'atom',
                                                                            'value' => {
                                                                                         'type' => 'atom',
                                                                                         'value' => [
                                                                                                      'rule_reference',
                                                                                                      'quantifier'
                                                                                                    ]
                                                                                       }
                                                                          }
                                                                        ],
                                                          'type' => 'sequence'
                                                        },
                                                        {
                                                          'type' => 'sequence',
                                                          'elements' => [
                                                                          {
                                                                            'type' => 'atom',
                                                                            'value' => {
                                                                                         'type' => 'atom',
                                                                                         'value' => [
                                                                                                      'rule_reference',
                                                                                                      'scalar_ref'
                                                                                                    ]
                                                                                       }
                                                                          },
                                                                          {
                                                                            'type' => 'atom',
                                                                            'value' => {
                                                                                         'type' => 'atom',
                                                                                         'value' => [
                                                                                                      'rule_reference',
                                                                                                      'quantifier'
                                                                                                    ]
                                                                                       }
                                                                          }
                                                                        ],
                                                          'return_annotation' => [
                                                                                   'return_object',
                                                                                   '{scalar: $1, quantifier: $2}'
                                                                                 ]
                                                        }
                                                      ]
                                  },
          'three_property_object' => {
                                       'return_annotation' => [
                                                                'return_object',
                                                                '{type: "multi_object", prop1: $3, prop2: $6, prop3: $9}'
                                                              ],
                                       'type' => 'sequence',
                                       'elements' => [
                                                       {
                                                         'value' => {
                                                                      'value' => [
                                                                                   'quoted_string',
                                                                                   '{'
                                                                                 ],
                                                                      'type' => 'atom'
                                                                    },
                                                         'type' => 'atom'
                                                       },
                                                       {
                                                         'value' => {
                                                                      'type' => 'atom',
                                                                      'value' => [
                                                                                   'regex',
                                                                                   '\\s*'
                                                                                 ]
                                                                    },
                                                         'type' => 'atom'
                                                       },
                                                       {
                                                         'type' => 'atom',
                                                         'value' => {
                                                                      'value' => [
                                                                                   'rule_reference',
                                                                                   'property'
                                                                                 ],
                                                                      'type' => 'atom'
                                                                    }
                                                       },
                                                       {
                                                         'value' => {
                                                                      'value' => [
                                                                                   'quoted_string',
                                                                                   ','
                                                                                 ],
                                                                      'type' => 'atom'
                                                                    },
                                                         'type' => 'atom'
                                                       },
                                                       {
                                                         'type' => 'atom',
                                                         'value' => {
                                                                      'type' => 'atom',
                                                                      'value' => [
                                                                                   'regex',
                                                                                   '\\s*'
                                                                                 ]
                                                                    }
                                                       },
                                                       {
                                                         'value' => {
                                                                      'type' => 'atom',
                                                                      'value' => [
                                                                                   'rule_reference',
                                                                                   'property'
                                                                                 ]
                                                                    },
                                                         'type' => 'atom'
                                                       },
                                                       {
                                                         'type' => 'atom',
                                                         'value' => {
                                                                      'type' => 'atom',
                                                                      'value' => [
                                                                                   'quoted_string',
                                                                                   ','
                                                                                 ]
                                                                    }
                                                       },
                                                       {
                                                         'type' => 'atom',
                                                         'value' => {
                                                                      'value' => [
                                                                                   'regex',
                                                                                   '\\s*'
                                                                                 ],
                                                                      'type' => 'atom'
                                                                    }
                                                       },
                                                       {
                                                         'type' => 'atom',
                                                         'value' => {
                                                                      'type' => 'atom',
                                                                      'value' => [
                                                                                   'rule_reference',
                                                                                   'property'
                                                                                 ]
                                                                    }
                                                       },
                                                       {
                                                         'type' => 'atom',
                                                         'value' => {
                                                                      'type' => 'atom',
                                                                      'value' => [
                                                                                   'regex',
                                                                                   '\\s*'
                                                                                 ]
                                                                    }
                                                       },
                                                       {
                                                         'value' => {
                                                                      'type' => 'atom',
                                                                      'value' => [
                                                                                   'quoted_string',
                                                                                   '}'
                                                                                 ]
                                                                    },
                                                         'type' => 'atom'
                                                       }
                                                     ]
                                     },
          'two_property_object' => {
                                     'return_annotation' => [
                                                              'return_object',
                                                              '{type: "multi_object", prop1: $3, prop2: $6}'
                                                            ],
                                     'elements' => [
                                                     {
                                                       'value' => {
                                                                    'value' => [
                                                                                 'quoted_string',
                                                                                 '{'
                                                                               ],
                                                                    'type' => 'atom'
                                                                  },
                                                       'type' => 'atom'
                                                     },
                                                     {
                                                       'value' => {
                                                                    'type' => 'atom',
                                                                    'value' => [
                                                                                 'regex',
                                                                                 '\\s*'
                                                                               ]
                                                                  },
                                                       'type' => 'atom'
                                                     },
                                                     {
                                                       'value' => {
                                                                    'value' => [
                                                                                 'rule_reference',
                                                                                 'property'
                                                                               ],
                                                                    'type' => 'atom'
                                                                  },
                                                       'type' => 'atom'
                                                     },
                                                     {
                                                       'value' => {
                                                                    'value' => [
                                                                                 'quoted_string',
                                                                                 ','
                                                                               ],
                                                                    'type' => 'atom'
                                                                  },
                                                       'type' => 'atom'
                                                     },
                                                     {
                                                       'value' => {
                                                                    'value' => [
                                                                                 'regex',
                                                                                 '\\s*'
                                                                               ],
                                                                    'type' => 'atom'
                                                                  },
                                                       'type' => 'atom'
                                                     },
                                                     {
                                                       'value' => {
                                                                    'value' => [
                                                                                 'rule_reference',
                                                                                 'property'
                                                                               ],
                                                                    'type' => 'atom'
                                                                  },
                                                       'type' => 'atom'
                                                     },
                                                     {
                                                       'value' => {
                                                                    'value' => [
                                                                                 'regex',
                                                                                 '\\s*'
                                                                               ],
                                                                    'type' => 'atom'
                                                                  },
                                                       'type' => 'atom'
                                                     },
                                                     {
                                                       'value' => {
                                                                    'value' => [
                                                                                 'quoted_string',
                                                                                 '}'
                                                                               ],
                                                                    'type' => 'atom'
                                                                  },
                                                       'type' => 'atom'
                                                     }
                                                   ],
                                     'type' => 'sequence'
                                   },
          'property_accessor' => {
                                   'return_annotation' => [
                                                            'return_object',
                                                            '{type: "property", name: $2}'
                                                          ],
                                   'elements' => [
                                                   {
                                                     'type' => 'atom',
                                                     'value' => {
                                                                  'type' => 'atom',
                                                                  'value' => [
                                                                               'quoted_string',
                                                                               '.'
                                                                             ]
                                                                }
                                                   },
                                                   {
                                                     'value' => {
                                                                  'value' => [
                                                                               'rule_reference',
                                                                               'identifier'
                                                                             ],
                                                                  'type' => 'atom'
                                                                },
                                                     'type' => 'atom'
                                                   }
                                                 ],
                                   'type' => 'sequence'
                                 },
          'nested_object' => {
                               'type' => 'sequence',
                               'elements' => [
                                               {
                                                 'type' => 'atom',
                                                 'value' => {
                                                              'value' => [
                                                                           'quoted_string',
                                                                           '{'
                                                                         ],
                                                              'type' => 'atom'
                                                            }
                                               },
                                               {
                                                 'value' => {
                                                              'type' => 'atom',
                                                              'value' => [
                                                                           'regex',
                                                                           '\\s*'
                                                                         ]
                                                            },
                                                 'type' => 'atom'
                                               },
                                               {
                                                 'type' => 'quantified',
                                                 'quantifier' => '?',
                                                 'element' => {
                                                                'value' => [
                                                                             'rule_reference',
                                                                             'object_contents'
                                                                           ],
                                                                'type' => 'atom'
                                                              }
                                               },
                                               {
                                                 'value' => {
                                                              'type' => 'atom',
                                                              'value' => [
                                                                           'regex',
                                                                           '\\s*'
                                                                         ]
                                                            },
                                                 'type' => 'atom'
                                               },
                                               {
                                                 'type' => 'atom',
                                                 'value' => {
                                                              'type' => 'atom',
                                                              'value' => [
                                                                           'quoted_string',
                                                                           '}'
                                                                         ]
                                                            }
                                               },
                                               {
                                                 'type' => 'quantified',
                                                 'element' => {
                                                                'value' => [
                                                                             'rule_reference',
                                                                             'quantifier'
                                                                           ],
                                                                'type' => 'atom'
                                                              },
                                                 'quantifier' => '?'
                                               }
                                             ],
                               'return_annotation' => [
                                                        'return_object',
                                                        '{type: "object", contents: $3, quantified: $6}'
                                                      ]
                             },
          'quantified_array' => {
                                  'return_annotation' => [
                                                           'return_object',
                                                           '{type: "quantified_array", element: $3}'
                                                         ],
                                  'type' => 'sequence',
                                  'elements' => [
                                                  {
                                                    'value' => {
                                                                 'value' => [
                                                                              'quoted_string',
                                                                              '['
                                                                            ],
                                                                 'type' => 'atom'
                                                               },
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'type' => 'atom',
                                                    'value' => {
                                                                 'value' => [
                                                                              'regex',
                                                                              '\\s*'
                                                                            ],
                                                                 'type' => 'atom'
                                                               }
                                                  },
                                                  {
                                                    'value' => {
                                                                 'type' => 'atom',
                                                                 'value' => [
                                                                              'rule_reference',
                                                                              'quantified_element'
                                                                            ]
                                                               },
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'value' => {
                                                                 'value' => [
                                                                              'regex',
                                                                              '\\s*'
                                                                            ],
                                                                 'type' => 'atom'
                                                               },
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'value' => {
                                                                 'value' => [
                                                                              'quoted_string',
                                                                              ']'
                                                                            ],
                                                                 'type' => 'atom'
                                                               },
                                                    'type' => 'atom'
                                                  }
                                                ]
                                },
          'python_slice_with_step' => {
                                        'return_annotation' => [
                                                                 'return_object',
                                                                 '{type: "python_slice_step", start: $1, end: $3, step: $5}'
                                                               ],
                                        'type' => 'sequence',
                                        'elements' => [
                                                        {
                                                          'type' => 'atom',
                                                          'value' => {
                                                                       'type' => 'atom',
                                                                       'value' => [
                                                                                    'rule_reference',
                                                                                    'python_slice_start'
                                                                                  ]
                                                                     }
                                                        },
                                                        {
                                                          'value' => {
                                                                       'type' => 'atom',
                                                                       'value' => [
                                                                                    'quoted_string',
                                                                                    ':'
                                                                                  ]
                                                                     },
                                                          'type' => 'atom'
                                                        },
                                                        {
                                                          'type' => 'atom',
                                                          'value' => {
                                                                       'type' => 'atom',
                                                                       'value' => [
                                                                                    'rule_reference',
                                                                                    'python_slice_end'
                                                                                  ]
                                                                     }
                                                        },
                                                        {
                                                          'value' => {
                                                                       'type' => 'atom',
                                                                       'value' => [
                                                                                    'quoted_string',
                                                                                    ':'
                                                                                  ]
                                                                     },
                                                          'type' => 'atom'
                                                        },
                                                        {
                                                          'type' => 'atom',
                                                          'value' => {
                                                                       'type' => 'atom',
                                                                       'value' => [
                                                                                    'rule_reference',
                                                                                    'step'
                                                                                  ]
                                                                     }
                                                        }
                                                      ]
                                      },
          'positive_number' => {
                                 'return_annotation' => [
                                                          'return_object',
                                                          '{type: "positive", value: $1}'
                                                        ],
                                 'value' => {
                                              'type' => 'atom',
                                              'value' => [
                                                           'regex',
                                                           '(\\d+)'
                                                         ]
                                            },
                                 'type' => 'atom'
                               },
          'return_expression' => {
                                   'type' => 'or',
                                   'alternatives' => [
                                                       {
                                                         'value' => {
                                                                      'type' => 'atom',
                                                                      'value' => [
                                                                                   'rule_reference',
                                                                                   'nested_array'
                                                                                 ]
                                                                    },
                                                         'type' => 'atom'
                                                       },
                                                       {
                                                         'type' => 'atom',
                                                         'value' => {
                                                                      'value' => [
                                                                                   'rule_reference',
                                                                                   'nested_object'
                                                                                 ],
                                                                      'type' => 'atom'
                                                                    }
                                                       },
                                                       {
                                                         'type' => 'atom',
                                                         'value' => {
                                                                      'type' => 'atom',
                                                                      'value' => [
                                                                                   'rule_reference',
                                                                                   'grouped_quantified_array'
                                                                                 ]
                                                                    }
                                                       },
                                                       {
                                                         'value' => {
                                                                      'value' => [
                                                                                   'rule_reference',
                                                                                   'simple_nested_object'
                                                                                 ],
                                                                      'type' => 'atom'
                                                                    },
                                                         'type' => 'atom'
                                                       },
                                                       {
                                                         'type' => 'atom',
                                                         'value' => {
                                                                      'value' => [
                                                                                   'rule_reference',
                                                                                   'multi_property_object'
                                                                                 ],
                                                                      'type' => 'atom'
                                                                    }
                                                       },
                                                       {
                                                         'value' => {
                                                                      'value' => [
                                                                                   'rule_reference',
                                                                                   'quantified_array'
                                                                                 ],
                                                                      'type' => 'atom'
                                                                    },
                                                         'type' => 'atom'
                                                       },
                                                       {
                                                         'type' => 'atom',
                                                         'value' => {
                                                                      'type' => 'atom',
                                                                      'value' => [
                                                                                   'rule_reference',
                                                                                   'simple_array'
                                                                                 ]
                                                                    }
                                                       },
                                                       {
                                                         'type' => 'atom',
                                                         'value' => {
                                                                      'value' => [
                                                                                   'rule_reference',
                                                                                   'simple_object'
                                                                                 ],
                                                                      'type' => 'atom'
                                                                    }
                                                       },
                                                       {
                                                         'type' => 'atom',
                                                         'value' => {
                                                                      'value' => [
                                                                                   'rule_reference',
                                                                                   'ultimate_dot_notation'
                                                                                 ],
                                                                      'type' => 'atom'
                                                                    }
                                                       },
                                                       {
                                                         'value' => {
                                                                      'type' => 'atom',
                                                                      'value' => [
                                                                                   'rule_reference',
                                                                                   'scalar_ref'
                                                                                 ]
                                                                    },
                                                         'type' => 'atom'
                                                       },
                                                       {
                                                         'value' => {
                                                                      'value' => [
                                                                                   'rule_reference',
                                                                                   'literal'
                                                                                 ],
                                                                      'type' => 'atom'
                                                                    },
                                                         'type' => 'atom'
                                                       }
                                                     ]
                                 },
          'nested_array' => {
                              'return_annotation' => [
                                                       'return_object',
                                                       '{type: "array", contents: $3, quantified: $6}'
                                                     ],
                              'type' => 'sequence',
                              'elements' => [
                                              {
                                                'value' => {
                                                             'type' => 'atom',
                                                             'value' => [
                                                                          'quoted_string',
                                                                          '['
                                                                        ]
                                                           },
                                                'type' => 'atom'
                                              },
                                              {
                                                'value' => {
                                                             'value' => [
                                                                          'regex',
                                                                          '\\s*'
                                                                        ],
                                                             'type' => 'atom'
                                                           },
                                                'type' => 'atom'
                                              },
                                              {
                                                'element' => {
                                                               'value' => [
                                                                            'rule_reference',
                                                                            'array_contents'
                                                                          ],
                                                               'type' => 'atom'
                                                             },
                                                'quantifier' => '?',
                                                'type' => 'quantified'
                                              },
                                              {
                                                'value' => {
                                                             'value' => [
                                                                          'regex',
                                                                          '\\s*'
                                                                        ],
                                                             'type' => 'atom'
                                                           },
                                                'type' => 'atom'
                                              },
                                              {
                                                'value' => {
                                                             'type' => 'atom',
                                                             'value' => [
                                                                          'quoted_string',
                                                                          ']'
                                                                        ]
                                                           },
                                                'type' => 'atom'
                                              },
                                              {
                                                'element' => {
                                                               'type' => 'atom',
                                                               'value' => [
                                                                            'rule_reference',
                                                                            'quantifier'
                                                                          ]
                                                             },
                                                'quantifier' => '?',
                                                'type' => 'quantified'
                                              }
                                            ]
                            },
          'positional_accessor' => {
                                     'return_annotation' => [
                                                              'return_object',
                                                              '{type: "position", index: $2}'
                                                            ],
                                     'elements' => [
                                                     {
                                                       'type' => 'atom',
                                                       'value' => {
                                                                    'type' => 'atom',
                                                                    'value' => [
                                                                                 'quoted_string',
                                                                                 '.'
                                                                               ]
                                                                  }
                                                     },
                                                     {
                                                       'type' => 'atom',
                                                       'value' => {
                                                                    'value' => [
                                                                                 'rule_reference',
                                                                                 'positive_number'
                                                                               ],
                                                                    'type' => 'atom'
                                                                  }
                                                     }
                                                   ],
                                     'type' => 'sequence'
                                   },
          'multi_property_object' => {
                                       'alternatives' => [
                                                           {
                                                             'type' => 'atom',
                                                             'value' => {
                                                                          'type' => 'atom',
                                                                          'value' => [
                                                                                       'rule_reference',
                                                                                       'two_property_object'
                                                                                     ]
                                                                        }
                                                           },
                                                           {
                                                             'type' => 'atom',
                                                             'value' => {
                                                                          'type' => 'atom',
                                                                          'value' => [
                                                                                       'rule_reference',
                                                                                       'three_property_object'
                                                                                     ]
                                                                        }
                                                           }
                                                         ],
                                       'type' => 'or'
                                     },
          'identifier' => {
                            'return_annotation' => [
                                                     'return_scalar',
                                                     '$1'
                                                   ],
                            'type' => 'atom',
                            'value' => {
                                         'value' => [
                                                      'regex',
                                                      '([a-zA-Z_]\\w*)'
                                                    ],
                                         'type' => 'atom'
                                       }
                          },
          'number' => {
                        'return_annotation' => [
                                                 'return_scalar',
                                                 '$1'
                                               ],
                        'value' => {
                                     'value' => [
                                                  'regex',
                                                  '(\\d+)'
                                                ],
                                     'type' => 'atom'
                                   },
                        'type' => 'atom'
                      },
          'grouped_element_list' => {
                                      'type' => 'or',
                                      'alternatives' => [
                                                          {
                                                            'return_annotation' => [
                                                                                     'return_array',
                                                                                     '[$1, $3*]'
                                                                                   ],
                                                            'elements' => [
                                                                            {
                                                                              'type' => 'atom',
                                                                              'value' => {
                                                                                           'value' => [
                                                                                                        'rule_reference',
                                                                                                        'grouped_element_item'
                                                                                                      ],
                                                                                           'type' => 'atom'
                                                                                         }
                                                                            },
                                                                            {
                                                                              'type' => 'quantified',
                                                                              'element' => {
                                                                                             'type' => 'atom',
                                                                                             'value' => {
                                                                                                          'type' => 'sequence',
                                                                                                          'elements' => [
                                                                                                                          [
                                                                                                                            'quoted_string',
                                                                                                                            ','
                                                                                                                          ],
                                                                                                                          [
                                                                                                                            'regex',
                                                                                                                            '\\s*'
                                                                                                                          ],
                                                                                                                          [
                                                                                                                            'rule_reference',
                                                                                                                            'grouped_element_item'
                                                                                                                          ]
                                                                                                                        ]
                                                                                                        }
                                                                                           },
                                                                              'quantifier' => '*'
                                                                            }
                                                                          ],
                                                            'type' => 'sequence'
                                                          },
                                                          {
                                                            'return_annotation' => [
                                                                                     'return_array',
                                                                                     '[$1]'
                                                                                   ],
                                                            'type' => 'atom',
                                                            'value' => {
                                                                         'type' => 'atom',
                                                                         'value' => [
                                                                                      'rule_reference',
                                                                                      'grouped_element_item'
                                                                                    ]
                                                                       }
                                                          }
                                                        ]
                                    },
          'index' => {
                       'type' => 'or',
                       'alternatives' => [
                                           {
                                             'type' => 'atom',
                                             'value' => {
                                                          'value' => [
                                                                       'rule_reference',
                                                                       'positive_number'
                                                                     ],
                                                          'type' => 'atom'
                                                        }
                                           },
                                           {
                                             'type' => 'atom',
                                             'value' => {
                                                          'type' => 'atom',
                                                          'value' => [
                                                                       'rule_reference',
                                                                       'negative_number'
                                                                     ]
                                                        }
                                           }
                                         ]
                     },
          'inner_value' => {
                             'alternatives' => [
                                                 {
                                                   'type' => 'atom',
                                                   'value' => {
                                                                'type' => 'atom',
                                                                'value' => [
                                                                             'rule_reference',
                                                                             'nested_array'
                                                                           ]
                                                              }
                                                 },
                                                 {
                                                   'value' => {
                                                                'value' => [
                                                                             'rule_reference',
                                                                             'nested_object'
                                                                           ],
                                                                'type' => 'atom'
                                                              },
                                                   'type' => 'atom'
                                                 },
                                                 {
                                                   'value' => {
                                                                'type' => 'atom',
                                                                'value' => [
                                                                             'rule_reference',
                                                                             'grouped_quantified_array'
                                                                           ]
                                                              },
                                                   'type' => 'atom'
                                                 },
                                                 {
                                                   'value' => {
                                                                'value' => [
                                                                             'rule_reference',
                                                                             'quantified_array'
                                                                           ],
                                                                'type' => 'atom'
                                                              },
                                                   'type' => 'atom'
                                                 },
                                                 {
                                                   'value' => {
                                                                'value' => [
                                                                             'rule_reference',
                                                                             'simple_array'
                                                                           ],
                                                                'type' => 'atom'
                                                              },
                                                   'type' => 'atom'
                                                 },
                                                 {
                                                   'value' => {
                                                                'value' => [
                                                                             'rule_reference',
                                                                             'ultimate_dot_notation'
                                                                           ],
                                                                'type' => 'atom'
                                                              },
                                                   'type' => 'atom'
                                                 },
                                                 {
                                                   'type' => 'atom',
                                                   'value' => {
                                                                'type' => 'atom',
                                                                'value' => [
                                                                             'rule_reference',
                                                                             'scalar_ref'
                                                                           ]
                                                              }
                                                 },
                                                 {
                                                   'value' => {
                                                                'value' => [
                                                                             'rule_reference',
                                                                             'literal'
                                                                           ],
                                                                'type' => 'atom'
                                                              },
                                                   'type' => 'atom'
                                                 }
                                               ],
                             'type' => 'or'
                           },
          'object_value' => {
                              'type' => 'or',
                              'alternatives' => [
                                                  {
                                                    'value' => {
                                                                 'type' => 'atom',
                                                                 'value' => [
                                                                              'rule_reference',
                                                                              'ultimate_dot_notation'
                                                                            ]
                                                               },
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'value' => {
                                                                 'value' => [
                                                                              'rule_reference',
                                                                              'scalar_ref'
                                                                            ],
                                                                 'type' => 'atom'
                                                               },
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'type' => 'atom',
                                                    'value' => {
                                                                 'value' => [
                                                                              'rule_reference',
                                                                              'literal'
                                                                            ],
                                                                 'type' => 'atom'
                                                               }
                                                  },
                                                  {
                                                    'value' => {
                                                                 'value' => [
                                                                              'rule_reference',
                                                                              'nested_array'
                                                                            ],
                                                                 'type' => 'atom'
                                                               },
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'type' => 'atom',
                                                    'value' => {
                                                                 'type' => 'atom',
                                                                 'value' => [
                                                                              'rule_reference',
                                                                              'nested_object'
                                                                            ]
                                                               }
                                                  }
                                                ]
                            },
          'single_index' => {
                              'value' => {
                                           'type' => 'atom',
                                           'value' => [
                                                        'rule_reference',
                                                        'index'
                                                      ]
                                         },
                              'type' => 'atom',
                              'return_annotation' => [
                                                       'return_object',
                                                       '{type: "single_index", value: $1}'
                                                     ]
                            },
          'quantifier' => {
                            'alternatives' => [
                                                {
                                                  'value' => {
                                                               'value' => [
                                                                            'quoted_string',
                                                                            '*'
                                                                          ],
                                                               'type' => 'atom'
                                                             },
                                                  'type' => 'atom',
                                                  'return_annotation' => [
                                                                           'return_scalar',
                                                                           '"*"'
                                                                         ]
                                                },
                                                {
                                                  'return_annotation' => [
                                                                           'return_scalar',
                                                                           '"+"'
                                                                         ],
                                                  'type' => 'atom',
                                                  'value' => {
                                                               'type' => 'atom',
                                                               'value' => [
                                                                            'quoted_string',
                                                                            '+'
                                                                          ]
                                                             }
                                                },
                                                {
                                                  'return_annotation' => [
                                                                           'return_scalar',
                                                                           '"?"'
                                                                         ],
                                                  'value' => {
                                                               'type' => 'atom',
                                                               'value' => [
                                                                            'quoted_string',
                                                                            '?'
                                                                          ]
                                                             },
                                                  'type' => 'atom'
                                                },
                                                {
                                                  'elements' => [
                                                                  {
                                                                    'type' => 'atom',
                                                                    'value' => {
                                                                                 'type' => 'atom',
                                                                                 'value' => [
                                                                                              'quoted_string',
                                                                                              '{'
                                                                                            ]
                                                                               }
                                                                  },
                                                                  {
                                                                    'type' => 'atom',
                                                                    'value' => {
                                                                                 'value' => [
                                                                                              'regex',
                                                                                              '\\s*'
                                                                                            ],
                                                                                 'type' => 'atom'
                                                                               }
                                                                  },
                                                                  {
                                                                    'type' => 'atom',
                                                                    'value' => {
                                                                                 'value' => [
                                                                                              'rule_reference',
                                                                                              'positive_number'
                                                                                            ],
                                                                                 'type' => 'atom'
                                                                               }
                                                                  },
                                                                  {
                                                                    'type' => 'atom',
                                                                    'value' => {
                                                                                 'value' => [
                                                                                              'regex',
                                                                                              '\\s*'
                                                                                            ],
                                                                                 'type' => 'atom'
                                                                               }
                                                                  },
                                                                  {
                                                                    'type' => 'atom',
                                                                    'value' => {
                                                                                 'value' => [
                                                                                              'quoted_string',
                                                                                              '}'
                                                                                            ],
                                                                                 'type' => 'atom'
                                                                               }
                                                                  }
                                                                ],
                                                  'type' => 'sequence',
                                                  'return_annotation' => [
                                                                           'return_object',
                                                                           '{min: $3, max: $3}'
                                                                         ]
                                                },
                                                {
                                                  'type' => 'sequence',
                                                  'elements' => [
                                                                  {
                                                                    'type' => 'atom',
                                                                    'value' => {
                                                                                 'value' => [
                                                                                              'quoted_string',
                                                                                              '{'
                                                                                            ],
                                                                                 'type' => 'atom'
                                                                               }
                                                                  },
                                                                  {
                                                                    'type' => 'atom',
                                                                    'value' => {
                                                                                 'value' => [
                                                                                              'regex',
                                                                                              '\\s*'
                                                                                            ],
                                                                                 'type' => 'atom'
                                                                               }
                                                                  },
                                                                  {
                                                                    'type' => 'atom',
                                                                    'value' => {
                                                                                 'type' => 'atom',
                                                                                 'value' => [
                                                                                              'rule_reference',
                                                                                              'positive_number'
                                                                                            ]
                                                                               }
                                                                  },
                                                                  {
                                                                    'type' => 'atom',
                                                                    'value' => {
                                                                                 'type' => 'atom',
                                                                                 'value' => [
                                                                                              'regex',
                                                                                              '\\s*'
                                                                                            ]
                                                                               }
                                                                  },
                                                                  {
                                                                    'value' => {
                                                                                 'type' => 'atom',
                                                                                 'value' => [
                                                                                              'quoted_string',
                                                                                              ','
                                                                                            ]
                                                                               },
                                                                    'type' => 'atom'
                                                                  },
                                                                  {
                                                                    'type' => 'atom',
                                                                    'value' => {
                                                                                 'value' => [
                                                                                              'regex',
                                                                                              '\\s*'
                                                                                            ],
                                                                                 'type' => 'atom'
                                                                               }
                                                                  },
                                                                  {
                                                                    'value' => {
                                                                                 'type' => 'atom',
                                                                                 'value' => [
                                                                                              'quoted_string',
                                                                                              '}'
                                                                                            ]
                                                                               },
                                                                    'type' => 'atom'
                                                                  }
                                                                ],
                                                  'return_annotation' => [
                                                                           'return_object',
                                                                           '{min: $3, max: "inf"}'
                                                                         ]
                                                },
                                                {
                                                  'type' => 'sequence',
                                                  'elements' => [
                                                                  {
                                                                    'type' => 'atom',
                                                                    'value' => {
                                                                                 'type' => 'atom',
                                                                                 'value' => [
                                                                                              'quoted_string',
                                                                                              '{'
                                                                                            ]
                                                                               }
                                                                  },
                                                                  {
                                                                    'type' => 'atom',
                                                                    'value' => {
                                                                                 'type' => 'atom',
                                                                                 'value' => [
                                                                                              'regex',
                                                                                              '\\s*'
                                                                                            ]
                                                                               }
                                                                  },
                                                                  {
                                                                    'value' => {
                                                                                 'value' => [
                                                                                              'rule_reference',
                                                                                              'positive_number'
                                                                                            ],
                                                                                 'type' => 'atom'
                                                                               },
                                                                    'type' => 'atom'
                                                                  },
                                                                  {
                                                                    'value' => {
                                                                                 'value' => [
                                                                                              'regex',
                                                                                              '\\s*'
                                                                                            ],
                                                                                 'type' => 'atom'
                                                                               },
                                                                    'type' => 'atom'
                                                                  },
                                                                  {
                                                                    'type' => 'atom',
                                                                    'value' => {
                                                                                 'type' => 'atom',
                                                                                 'value' => [
                                                                                              'quoted_string',
                                                                                              ','
                                                                                            ]
                                                                               }
                                                                  },
                                                                  {
                                                                    'type' => 'atom',
                                                                    'value' => {
                                                                                 'value' => [
                                                                                              'regex',
                                                                                              '\\s*'
                                                                                            ],
                                                                                 'type' => 'atom'
                                                                               }
                                                                  },
                                                                  {
                                                                    'value' => {
                                                                                 'type' => 'atom',
                                                                                 'value' => [
                                                                                              'rule_reference',
                                                                                              'positive_number'
                                                                                            ]
                                                                               },
                                                                    'type' => 'atom'
                                                                  },
                                                                  {
                                                                    'value' => {
                                                                                 'value' => [
                                                                                              'regex',
                                                                                              '\\s*'
                                                                                            ],
                                                                                 'type' => 'atom'
                                                                               },
                                                                    'type' => 'atom'
                                                                  },
                                                                  {
                                                                    'value' => {
                                                                                 'value' => [
                                                                                              'quoted_string',
                                                                                              '}'
                                                                                            ],
                                                                                 'type' => 'atom'
                                                                               },
                                                                    'type' => 'atom'
                                                                  }
                                                                ],
                                                  'return_annotation' => [
                                                                           'return_object',
                                                                           '{min: $3, max: $7}'
                                                                         ]
                                                },
                                                {
                                                  'return_annotation' => [
                                                                           'return_object',
                                                                           '{min: 0, max: $5}'
                                                                         ],
                                                  'elements' => [
                                                                  {
                                                                    'type' => 'atom',
                                                                    'value' => {
                                                                                 'value' => [
                                                                                              'quoted_string',
                                                                                              '{'
                                                                                            ],
                                                                                 'type' => 'atom'
                                                                               }
                                                                  },
                                                                  {
                                                                    'type' => 'atom',
                                                                    'value' => {
                                                                                 'value' => [
                                                                                              'regex',
                                                                                              '\\s*'
                                                                                            ],
                                                                                 'type' => 'atom'
                                                                               }
                                                                  },
                                                                  {
                                                                    'value' => {
                                                                                 'value' => [
                                                                                              'quoted_string',
                                                                                              ','
                                                                                            ],
                                                                                 'type' => 'atom'
                                                                               },
                                                                    'type' => 'atom'
                                                                  },
                                                                  {
                                                                    'type' => 'atom',
                                                                    'value' => {
                                                                                 'value' => [
                                                                                              'regex',
                                                                                              '\\s*'
                                                                                            ],
                                                                                 'type' => 'atom'
                                                                               }
                                                                  },
                                                                  {
                                                                    'value' => {
                                                                                 'type' => 'atom',
                                                                                 'value' => [
                                                                                              'rule_reference',
                                                                                              'positive_number'
                                                                                            ]
                                                                               },
                                                                    'type' => 'atom'
                                                                  },
                                                                  {
                                                                    'value' => {
                                                                                 'type' => 'atom',
                                                                                 'value' => [
                                                                                              'regex',
                                                                                              '\\s*'
                                                                                            ]
                                                                               },
                                                                    'type' => 'atom'
                                                                  },
                                                                  {
                                                                    'type' => 'atom',
                                                                    'value' => {
                                                                                 'type' => 'atom',
                                                                                 'value' => [
                                                                                              'quoted_string',
                                                                                              '}'
                                                                                            ]
                                                                               }
                                                                  }
                                                                ],
                                                  'type' => 'sequence'
                                                }
                                              ],
                            'type' => 'or'
                          },
          'object_pair' => {
                             'type' => 'sequence',
                             'elements' => [
                                             {
                                               'value' => {
                                                            'type' => 'atom',
                                                            'value' => [
                                                                         'rule_reference',
                                                                         'object_key'
                                                                       ]
                                                          },
                                               'type' => 'atom'
                                             },
                                             {
                                               'value' => {
                                                            'type' => 'atom',
                                                            'value' => [
                                                                         'regex',
                                                                         '\\s*'
                                                                       ]
                                                          },
                                               'type' => 'atom'
                                             },
                                             {
                                               'value' => {
                                                            'type' => 'atom',
                                                            'value' => [
                                                                         'quoted_string',
                                                                         ':'
                                                                       ]
                                                          },
                                               'type' => 'atom'
                                             },
                                             {
                                               'type' => 'atom',
                                               'value' => {
                                                            'type' => 'atom',
                                                            'value' => [
                                                                         'regex',
                                                                         '\\s*'
                                                                       ]
                                                          }
                                             },
                                             {
                                               'type' => 'atom',
                                               'value' => {
                                                            'value' => [
                                                                         'rule_reference',
                                                                         'return_expression'
                                                                       ],
                                                            'type' => 'atom'
                                                          }
                                             }
                                           ],
                             'return_annotation' => [
                                                      'return_object',
                                                      '{key: $1, value: $5}'
                                                    ]
                           },
          'step' => {
                      'value' => {
                                   'type' => 'atom',
                                   'value' => [
                                                'rule_reference',
                                                'index'
                                              ]
                                 },
                      'type' => 'atom'
                    },
          'element_item' => {
                              'type' => 'or',
                              'alternatives' => [
                                                  {
                                                    'type' => 'atom',
                                                    'value' => {
                                                                 'value' => [
                                                                              'rule_reference',
                                                                              'identifier'
                                                                            ],
                                                                 'type' => 'atom'
                                                               }
                                                  },
                                                  {
                                                    'value' => {
                                                                 'type' => 'atom',
                                                                 'value' => [
                                                                              'rule_reference',
                                                                              'literal'
                                                                            ]
                                                               },
                                                    'type' => 'atom'
                                                  }
                                                ]
                            },
          'empty_slice_part' => {
                                  'value' => {
                                               'type' => 'atom',
                                               'value' => [
                                                            'regex',
                                                            '(?=:)'
                                                          ]
                                             },
                                  'type' => 'atom'
                                },
          'empty_spec' => {
                            'type' => 'atom',
                            'value' => {
                                         'value' => [
                                                      'regex',
                                                      '(?=\\])'
                                                    ],
                                         'type' => 'atom'
                                       }
                          },
          'inner_key' => {
                           'alternatives' => [
                                               {
                                                 'value' => {
                                                              'type' => 'atom',
                                                              'value' => [
                                                                           'rule_reference',
                                                                           'identifier'
                                                                         ]
                                                            },
                                                 'type' => 'atom'
                                               },
                                               {
                                                 'type' => 'atom',
                                                 'value' => {
                                                              'type' => 'atom',
                                                              'value' => [
                                                                           'rule_reference',
                                                                           'quoted_string'
                                                                         ]
                                                            }
                                               }
                                             ],
                           'type' => 'or'
                         },
          'object_key' => {
                            'alternatives' => [
                                                {
                                                  'type' => 'atom',
                                                  'value' => {
                                                               'value' => [
                                                                            'rule_reference',
                                                                            'identifier'
                                                                          ],
                                                               'type' => 'atom'
                                                             }
                                                },
                                                {
                                                  'value' => {
                                                               'value' => [
                                                                            'rule_reference',
                                                                            'quoted_string'
                                                                          ],
                                                               'type' => 'atom'
                                                             },
                                                  'type' => 'atom'
                                                }
                                              ],
                            'type' => 'or'
                          },
          'property' => {
                          'type' => 'sequence',
                          'elements' => [
                                          {
                                            'type' => 'atom',
                                            'value' => {
                                                         'type' => 'atom',
                                                         'value' => [
                                                                      'rule_reference',
                                                                      'object_key'
                                                                    ]
                                                       }
                                          },
                                          {
                                            'type' => 'atom',
                                            'value' => {
                                                         'value' => [
                                                                      'regex',
                                                                      '\\s*'
                                                                    ],
                                                         'type' => 'atom'
                                                       }
                                          },
                                          {
                                            'type' => 'atom',
                                            'value' => {
                                                         'value' => [
                                                                      'quoted_string',
                                                                      ':'
                                                                    ],
                                                         'type' => 'atom'
                                                       }
                                          },
                                          {
                                            'value' => {
                                                         'value' => [
                                                                      'regex',
                                                                      '\\s*'
                                                                    ],
                                                         'type' => 'atom'
                                                       },
                                            'type' => 'atom'
                                          },
                                          {
                                            'value' => {
                                                         'type' => 'atom',
                                                         'value' => [
                                                                      'rule_reference',
                                                                      'property_value'
                                                                    ]
                                                       },
                                            'type' => 'atom'
                                          }
                                        ],
                          'return_annotation' => [
                                                   'return_object',
                                                   '{key: $1, value: $5}'
                                                 ]
                        },
          'python_slice_start' => {
                                    'type' => 'or',
                                    'alternatives' => [
                                                        {
                                                          'value' => {
                                                                       'value' => [
                                                                                    'rule_reference',
                                                                                    'index'
                                                                                  ],
                                                                       'type' => 'atom'
                                                                     },
                                                          'type' => 'atom'
                                                        },
                                                        {
                                                          'type' => 'atom',
                                                          'value' => {
                                                                       'value' => [
                                                                                    'rule_reference',
                                                                                    'empty_slice_part'
                                                                                  ],
                                                                       'type' => 'atom'
                                                                     }
                                                        }
                                                      ]
                                  },
          'array_element' => {
                               'alternatives' => [
                                                   {
                                                     'type' => 'atom',
                                                     'value' => {
                                                                  'type' => 'atom',
                                                                  'value' => [
                                                                               'rule_reference',
                                                                               'ultimate_dot_notation'
                                                                             ]
                                                                }
                                                   },
                                                   {
                                                     'value' => {
                                                                  'type' => 'atom',
                                                                  'value' => [
                                                                               'rule_reference',
                                                                               'scalar_ref'
                                                                             ]
                                                                },
                                                     'type' => 'atom'
                                                   },
                                                   {
                                                     'value' => {
                                                                  'value' => [
                                                                               'rule_reference',
                                                                               'literal'
                                                                             ],
                                                                  'type' => 'atom'
                                                                },
                                                     'type' => 'atom'
                                                   },
                                                   {
                                                     'value' => {
                                                                  'type' => 'atom',
                                                                  'value' => [
                                                                               'rule_reference',
                                                                               'nested_array'
                                                                             ]
                                                                },
                                                     'type' => 'atom'
                                                   },
                                                   {
                                                     'value' => {
                                                                  'type' => 'atom',
                                                                  'value' => [
                                                                               'rule_reference',
                                                                               'nested_object'
                                                                             ]
                                                                },
                                                     'type' => 'atom'
                                                   }
                                                 ],
                               'type' => 'or'
                             },
          'python_slice' => {
                              'type' => 'sequence',
                              'elements' => [
                                              {
                                                'type' => 'atom',
                                                'value' => {
                                                             'type' => 'atom',
                                                             'value' => [
                                                                          'rule_reference',
                                                                          'python_slice_start'
                                                                        ]
                                                           }
                                              },
                                              {
                                                'type' => 'atom',
                                                'value' => {
                                                             'type' => 'atom',
                                                             'value' => [
                                                                          'quoted_string',
                                                                          ':'
                                                                        ]
                                                           }
                                              },
                                              {
                                                'value' => {
                                                             'value' => [
                                                                          'rule_reference',
                                                                          'python_slice_end'
                                                                        ],
                                                             'type' => 'atom'
                                                           },
                                                'type' => 'atom'
                                              }
                                            ],
                              'return_annotation' => [
                                                       'return_object',
                                                       '{type: "python_slice", start: $1, end: $3}'
                                                     ]
                            },
          'accessor' => {
                          'alternatives' => [
                                              {
                                                'type' => 'atom',
                                                'value' => {
                                                             'value' => [
                                                                          'rule_reference',
                                                                          'property_accessor'
                                                                        ],
                                                             'type' => 'atom'
                                                           }
                                              },
                                              {
                                                'type' => 'atom',
                                                'value' => {
                                                             'value' => [
                                                                          'rule_reference',
                                                                          'positional_accessor'
                                                                        ],
                                                             'type' => 'atom'
                                                           }
                                              },
                                              {
                                                'type' => 'atom',
                                                'value' => {
                                                             'type' => 'atom',
                                                             'value' => [
                                                                          'rule_reference',
                                                                          'array_accessor'
                                                                        ]
                                                           }
                                              }
                                            ],
                          'type' => 'or'
                        },
          'dot_path' => {
                          'element' => {
                                         'type' => 'atom',
                                         'value' => [
                                                      'rule_reference',
                                                      'accessor'
                                                    ]
                                       },
                          'quantifier' => '+',
                          'return_annotation' => [
                                                   'return_array',
                                                   '[$1*]'
                                                 ],
                          'type' => 'quantified'
                        },
          'property_value' => {
                                'type' => 'or',
                                'alternatives' => [
                                                    {
                                                      'value' => {
                                                                   'type' => 'atom',
                                                                   'value' => [
                                                                                'rule_reference',
                                                                                'nested_array'
                                                                              ]
                                                                 },
                                                      'type' => 'atom'
                                                    },
                                                    {
                                                      'value' => {
                                                                   'type' => 'atom',
                                                                   'value' => [
                                                                                'rule_reference',
                                                                                'nested_object'
                                                                              ]
                                                                 },
                                                      'type' => 'atom'
                                                    },
                                                    {
                                                      'type' => 'atom',
                                                      'value' => {
                                                                   'type' => 'atom',
                                                                   'value' => [
                                                                                'rule_reference',
                                                                                'grouped_quantified_array'
                                                                              ]
                                                                 }
                                                    },
                                                    {
                                                      'value' => {
                                                                   'type' => 'atom',
                                                                   'value' => [
                                                                                'rule_reference',
                                                                                'quantified_array'
                                                                              ]
                                                                 },
                                                      'type' => 'atom'
                                                    },
                                                    {
                                                      'value' => {
                                                                   'type' => 'atom',
                                                                   'value' => [
                                                                                'rule_reference',
                                                                                'simple_array'
                                                                              ]
                                                                 },
                                                      'type' => 'atom'
                                                    },
                                                    {
                                                      'value' => {
                                                                   'value' => [
                                                                                'rule_reference',
                                                                                'ultimate_dot_notation'
                                                                              ],
                                                                   'type' => 'atom'
                                                                 },
                                                      'type' => 'atom'
                                                    },
                                                    {
                                                      'value' => {
                                                                   'value' => [
                                                                                'rule_reference',
                                                                                'scalar_ref'
                                                                              ],
                                                                   'type' => 'atom'
                                                                 },
                                                      'type' => 'atom'
                                                    },
                                                    {
                                                      'value' => {
                                                                   'value' => [
                                                                                'rule_reference',
                                                                                'literal'
                                                                              ],
                                                                   'type' => 'atom'
                                                                 },
                                                      'type' => 'atom'
                                                    }
                                                  ]
                              },
          'index_list' => {
                            'type' => 'sequence',
                            'elements' => [
                                            {
                                              'type' => 'atom',
                                              'value' => {
                                                           'type' => 'atom',
                                                           'value' => [
                                                                        'rule_reference',
                                                                        'index'
                                                                      ]
                                                         }
                                            },
                                            {
                                              'type' => 'quantified',
                                              'element' => {
                                                             'value' => {
                                                                          'elements' => [
                                                                                          [
                                                                                            'quoted_string',
                                                                                            ','
                                                                                          ],
                                                                                          [
                                                                                            'regex',
                                                                                            '\\s*'
                                                                                          ],
                                                                                          [
                                                                                            'rule_reference',
                                                                                            'index'
                                                                                          ]
                                                                                        ],
                                                                          'type' => 'sequence'
                                                                        },
                                                             'type' => 'atom'
                                                           },
                                              'quantifier' => '*'
                                            }
                                          ],
                            'return_annotation' => [
                                                     'return_array',
                                                     '[$1, $3*]'
                                                   ]
                          },
          'perl_range' => {
                            'type' => 'sequence',
                            'elements' => [
                                            {
                                              'type' => 'atom',
                                              'value' => {
                                                           'type' => 'atom',
                                                           'value' => [
                                                                        'rule_reference',
                                                                        'index'
                                                                      ]
                                                         }
                                            },
                                            {
                                              'value' => {
                                                           'type' => 'atom',
                                                           'value' => [
                                                                        'quoted_string',
                                                                        '..'
                                                                      ]
                                                         },
                                              'type' => 'atom'
                                            },
                                            {
                                              'type' => 'atom',
                                              'value' => {
                                                           'type' => 'atom',
                                                           'value' => [
                                                                        'rule_reference',
                                                                        'index'
                                                                      ]
                                                         }
                                            }
                                          ],
                            'return_annotation' => [
                                                     'return_object',
                                                     '{type: "perl_range", start: $1, end: $3}'
                                                   ]
                          },
          'negative_number' => {
                                 'elements' => [
                                                 {
                                                   'value' => {
                                                                'type' => 'atom',
                                                                'value' => [
                                                                             'quoted_string',
                                                                             '-'
                                                                           ]
                                                              },
                                                   'type' => 'atom'
                                                 },
                                                 {
                                                   'type' => 'atom',
                                                   'value' => {
                                                                'type' => 'atom',
                                                                'value' => [
                                                                             'rule_reference',
                                                                             'positive_number'
                                                                           ]
                                                              }
                                                 }
                                               ],
                                 'type' => 'sequence',
                                 'return_annotation' => [
                                                          'return_object',
                                                          '{type: "negative", value: $2}'
                                                        ]
                               },
          'colon_spec' => {
                            'type' => 'atom',
                            'value' => {
                                         'type' => 'atom',
                                         'value' => [
                                                      'quoted_string',
                                                      ':'
                                                    ]
                                       }
                          },
          'element_sequence' => {
                                  'type' => 'or',
                                  'alternatives' => [
                                                      {
                                                        'return_annotation' => [
                                                                                 'return_array',
                                                                                 '[$1, $3*]'
                                                                               ],
                                                        'type' => 'sequence',
                                                        'elements' => [
                                                                        {
                                                                          'value' => {
                                                                                       'value' => [
                                                                                                    'rule_reference',
                                                                                                    'element_item'
                                                                                                  ],
                                                                                       'type' => 'atom'
                                                                                     },
                                                                          'type' => 'atom'
                                                                        },
                                                                        {
                                                                          'type' => 'quantified',
                                                                          'element' => {
                                                                                         'value' => {
                                                                                                      'elements' => [
                                                                                                                      [
                                                                                                                        'regex',
                                                                                                                        '\\s+'
                                                                                                                      ],
                                                                                                                      [
                                                                                                                        'rule_reference',
                                                                                                                        'element_item'
                                                                                                                      ]
                                                                                                                    ],
                                                                                                      'type' => 'sequence'
                                                                                                    },
                                                                                         'type' => 'atom'
                                                                                       },
                                                                          'quantifier' => '*'
                                                                        }
                                                                      ]
                                                      },
                                                      {
                                                        'value' => {
                                                                     'type' => 'atom',
                                                                     'value' => [
                                                                                  'rule_reference',
                                                                                  'element_item'
                                                                                ]
                                                                   },
                                                        'type' => 'atom',
                                                        'return_annotation' => [
                                                                                 'return_array',
                                                                                 '[$1]'
                                                                               ]
                                                      }
                                                    ]
                                },
          'simple_array' => {
                              'type' => 'sequence',
                              'elements' => [
                                              {
                                                'value' => {
                                                             'value' => [
                                                                          'quoted_string',
                                                                          '['
                                                                        ],
                                                             'type' => 'atom'
                                                           },
                                                'type' => 'atom'
                                              },
                                              {
                                                'value' => {
                                                             'type' => 'atom',
                                                             'value' => [
                                                                          'regex',
                                                                          '\\s*'
                                                                        ]
                                                           },
                                                'type' => 'atom'
                                              },
                                              {
                                                'type' => 'atom',
                                                'value' => {
                                                             'value' => [
                                                                          'rule_reference',
                                                                          'array_element'
                                                                        ],
                                                             'type' => 'atom'
                                                           }
                                              },
                                              {
                                                'value' => {
                                                             'type' => 'atom',
                                                             'value' => [
                                                                          'regex',
                                                                          '\\s*'
                                                                        ]
                                                           },
                                                'type' => 'atom'
                                              },
                                              {
                                                'type' => 'atom',
                                                'value' => {
                                                             'type' => 'atom',
                                                             'value' => [
                                                                          'quoted_string',
                                                                          ']'
                                                                        ]
                                                           }
                                              }
                                            ],
                              'return_annotation' => [
                                                       'return_object',
                                                       '{type: "array", element: $3}'
                                                     ]
                            },
          'simple_nested_object' => {
                                      'return_annotation' => [
                                                               'return_object',
                                                               '{type: "nested_object", key: $3, value: $7}'
                                                             ],
                                      'elements' => [
                                                      {
                                                        'type' => 'atom',
                                                        'value' => {
                                                                     'type' => 'atom',
                                                                     'value' => [
                                                                                  'quoted_string',
                                                                                  '{'
                                                                                ]
                                                                   }
                                                      },
                                                      {
                                                        'type' => 'atom',
                                                        'value' => {
                                                                     'value' => [
                                                                                  'regex',
                                                                                  '\\s*'
                                                                                ],
                                                                     'type' => 'atom'
                                                                   }
                                                      },
                                                      {
                                                        'type' => 'atom',
                                                        'value' => {
                                                                     'value' => [
                                                                                  'rule_reference',
                                                                                  'outer_key'
                                                                                ],
                                                                     'type' => 'atom'
                                                                   }
                                                      },
                                                      {
                                                        'value' => {
                                                                     'type' => 'atom',
                                                                     'value' => [
                                                                                  'regex',
                                                                                  '\\s*'
                                                                                ]
                                                                   },
                                                        'type' => 'atom'
                                                      },
                                                      {
                                                        'value' => {
                                                                     'type' => 'atom',
                                                                     'value' => [
                                                                                  'quoted_string',
                                                                                  ':'
                                                                                ]
                                                                   },
                                                        'type' => 'atom'
                                                      },
                                                      {
                                                        'type' => 'atom',
                                                        'value' => {
                                                                     'type' => 'atom',
                                                                     'value' => [
                                                                                  'regex',
                                                                                  '\\s*'
                                                                                ]
                                                                   }
                                                      },
                                                      {
                                                        'type' => 'atom',
                                                        'value' => {
                                                                     'value' => [
                                                                                  'rule_reference',
                                                                                  'inner_object'
                                                                                ],
                                                                     'type' => 'atom'
                                                                   }
                                                      },
                                                      {
                                                        'value' => {
                                                                     'type' => 'atom',
                                                                     'value' => [
                                                                                  'regex',
                                                                                  '\\s*'
                                                                                ]
                                                                   },
                                                        'type' => 'atom'
                                                      },
                                                      {
                                                        'value' => {
                                                                     'value' => [
                                                                                  'quoted_string',
                                                                                  '}'
                                                                                ],
                                                                     'type' => 'atom'
                                                                   },
                                                        'type' => 'atom'
                                                      }
                                                    ],
                                      'type' => 'sequence'
                                    },
          'inner_object' => {
                              'return_annotation' => [
                                                       'return_object',
                                                       '{type: "inner_object", key: $3, value: $7}'
                                                     ],
                              'elements' => [
                                              {
                                                'value' => {
                                                             'type' => 'atom',
                                                             'value' => [
                                                                          'quoted_string',
                                                                          '{'
                                                                        ]
                                                           },
                                                'type' => 'atom'
                                              },
                                              {
                                                'value' => {
                                                             'value' => [
                                                                          'regex',
                                                                          '\\s*'
                                                                        ],
                                                             'type' => 'atom'
                                                           },
                                                'type' => 'atom'
                                              },
                                              {
                                                'value' => {
                                                             'type' => 'atom',
                                                             'value' => [
                                                                          'rule_reference',
                                                                          'inner_key'
                                                                        ]
                                                           },
                                                'type' => 'atom'
                                              },
                                              {
                                                'value' => {
                                                             'type' => 'atom',
                                                             'value' => [
                                                                          'regex',
                                                                          '\\s*'
                                                                        ]
                                                           },
                                                'type' => 'atom'
                                              },
                                              {
                                                'type' => 'atom',
                                                'value' => {
                                                             'value' => [
                                                                          'quoted_string',
                                                                          ':'
                                                                        ],
                                                             'type' => 'atom'
                                                           }
                                              },
                                              {
                                                'type' => 'atom',
                                                'value' => {
                                                             'value' => [
                                                                          'regex',
                                                                          '\\s*'
                                                                        ],
                                                             'type' => 'atom'
                                                           }
                                              },
                                              {
                                                'value' => {
                                                             'type' => 'atom',
                                                             'value' => [
                                                                          'rule_reference',
                                                                          'inner_value'
                                                                        ]
                                                           },
                                                'type' => 'atom'
                                              },
                                              {
                                                'type' => 'atom',
                                                'value' => {
                                                             'type' => 'atom',
                                                             'value' => [
                                                                          'regex',
                                                                          '\\s*'
                                                                        ]
                                                           }
                                              },
                                              {
                                                'value' => {
                                                             'value' => [
                                                                          'quoted_string',
                                                                          '}'
                                                                        ],
                                                             'type' => 'atom'
                                                           },
                                                'type' => 'atom'
                                              }
                                            ],
                              'type' => 'sequence'
                            },
          'group_content' => {
                               'type' => 'atom',
                               'value' => {
                                            'type' => 'atom',
                                            'value' => [
                                                         'rule_reference',
                                                         'element_sequence'
                                                       ]
                                          }
                             },
          'scalar_ref' => {
                            'return_annotation' => [
                                                     'return_object',
                                                     '{type: "scalar_ref", index: $2}'
                                                   ],
                            'elements' => [
                                            {
                                              'value' => {
                                                           'value' => [
                                                                        'quoted_string',
                                                                        '$'
                                                                      ],
                                                           'type' => 'atom'
                                                         },
                                              'type' => 'atom'
                                            },
                                            {
                                              'value' => {
                                                           'type' => 'atom',
                                                           'value' => [
                                                                        'rule_reference',
                                                                        'positive_number'
                                                                      ]
                                                         },
                                              'type' => 'atom'
                                            }
                                          ],
                            'type' => 'sequence'
                          }
        };
RULE ORDER: return_annotation, return_expression, nested_array, array_contents, nested_object, object_contents, object_pair, ultimate_dot_notation, dot_path, accessor, property_accessor, positional_accessor, array_accessor, array_spec, empty_spec, star_spec, colon_spec, single_index, perl_range, python_slice, python_slice_with_step, python_slice_start, python_slice_end, step, empty_slice_part, index_list, mixed_expression, mixed_element, index, positive_number, negative_number, grouped_quantified_array, grouped_element_list, grouped_element_item, group_content, element_sequence, element_item, simple_nested_object, inner_object, inner_value, multi_property_object, two_property_object, three_property_object, property, property_value, quantified_array, quantified_element, quantifier, simple_array, array_element, simple_object, object_key, outer_key, inner_key, object_value, literal, scalar_ref, quoted_string, number, identifier
DEBUG: index_list after step5:
$VAR1 = {
          'type' => 'sequence',
          'elements' => [
                          {
                            'type' => 'atom',
                            'value' => {
                                         'type' => 'atom',
                                         'value' => [
                                                      'rule_reference',
                                                      'index'
                                                    ]
                                       }
                          },
                          {
                            'type' => 'quantified',
                            'element' => {
                                           'value' => {
                                                        'elements' => [
                                                                        [
                                                                          'quoted_string',
                                                                          ','
                                                                        ],
                                                                        [
                                                                          'regex',
                                                                          '\\s*'
                                                                        ],
                                                                        [
                                                                          'rule_reference',
                                                                          'index'
                                                                        ]
                                                                      ],
                                                        'type' => 'sequence'
                                                      },
                                           'type' => 'atom'
                                         },
                            'quantifier' => '*'
                          }
                        ],
          'return_annotation' => [
                                   'return_array',
                                   '[$1, $3*]'
                                 ]
        };
DEBUG: index rule after step5:
$VAR1 = {
          'type' => 'or',
          'alternatives' => [
                              {
                                'type' => 'atom',
                                'value' => {
                                             'value' => [
                                                          'rule_reference',
                                                          'positive_number'
                                                        ],
                                             'type' => 'atom'
                                           }
                              },
                              {
                                'type' => 'atom',
                                'value' => {
                                             'type' => 'atom',
                                             'value' => [
                                                          'rule_reference',
                                                          'negative_number'
                                                        ]
                                           }
                              }
                            ]
        };

=== Step 6: Generate parser code ===
DEBUG: Keys in step5_result before step6: accessor, array_accessor, array_contents, array_element, array_spec, colon_spec, dot_path, element_item, element_sequence, empty_slice_part, empty_spec, group_content, grouped_element_item, grouped_element_list, grouped_quantified_array, identifier, index, index_list, inner_key, inner_object, inner_value, literal, mixed_element, mixed_expression, multi_property_object, negative_number, nested_array, nested_object, number, object_contents, object_key, object_pair, object_value, outer_key, perl_range, positional_accessor, positive_number, property, property_accessor, property_value, python_slice, python_slice_end, python_slice_start, python_slice_with_step, quantified_array, quantified_element, quantifier, quoted_string, return_annotation, return_expression, scalar_ref, simple_array, simple_nested_object, simple_object, single_index, star_spec, step, three_property_object, two_property_object, ultimate_dot_notation
🚀 DEPLOYING LEFT-RECURSION NUCLEAR ELIMINATOR!
🎯 Target: Complete annihilation of all recursion forms
======================================================================

🔄 Converting AST format to elimination format...
🎯 FOUND dot_path rule in grammar_tree: $VAR1 = {
          'element' => {
                         'type' => 'atom',
                         'value' => [
                                      'rule_reference',
                                      'accessor'
                                    ]
                       },
          'quantifier' => '+',
          'return_annotation' => [
                                   'return_array',
                                   '[$1*]'
                                 ],
          'type' => 'quantified'
        };

📊 Converted 60 rules
🏷️ Stored annotations for 53 rules
📋 Grammar before elimination:
   accessor := property_accessor | positional_accessor | array_accessor
   array_accessor := TERMINAL:[ array_spec TERMINAL:]
   array_contents := return_expression QUANTIFIED:HASH(0x12581c170):*
   array_element := ultimate_dot_notation | scalar_ref | literal | nested_array | nested_object
   array_spec := empty_spec | star_spec | colon_spec | single_index | perl_range | python_slice | python_slice_with_step | index_list | mixed_expression
   colon_spec := TERMINAL::
   dot_path := QUANTIFIED:accessor:+
   element_item := identifier | literal
   element_sequence := element_item QUANTIFIED:HASH(0x1259f10e0):* | element_item
   empty_slice_part := REGEX:(?=:)
   empty_spec := REGEX:(?=\])
   group_content := element_sequence
   grouped_element_item := TERMINAL:( REGEX:\s* group_content REGEX:\s* TERMINAL:) quantifier | element_sequence quantifier
   grouped_element_list := grouped_element_item QUANTIFIED:HASH(0x125a26e50):* | grouped_element_item
   grouped_quantified_array := TERMINAL:[ REGEX:\s* grouped_element_list REGEX:\s* TERMINAL:]
   identifier := REGEX:([a-zA-Z_]\w*)
   index := positive_number | negative_number
   index_list := index QUANTIFIED:HASH(0x1259fcf68):*
   inner_key := identifier | quoted_string
   inner_object := TERMINAL:{ REGEX:\s* inner_key REGEX:\s* TERMINAL:: REGEX:\s* inner_value REGEX:\s* TERMINAL:}
   inner_value := nested_array | nested_object | grouped_quantified_array | quantified_array | simple_array | ultimate_dot_notation | scalar_ref | literal
   literal := quoted_string | number
   mixed_element := single_index | perl_range | python_slice | python_slice_with_step
   mixed_expression := mixed_element QUANTIFIED:HASH(0x125a16388):*
   multi_property_object := two_property_object | three_property_object
   negative_number := TERMINAL:- positive_number
   nested_array := TERMINAL:[ REGEX:\s* QUANTIFIED:array_contents:? REGEX:\s* TERMINAL:] QUANTIFIED:quantifier:?
   nested_object := TERMINAL:{ REGEX:\s* QUANTIFIED:object_contents:? REGEX:\s* TERMINAL:} QUANTIFIED:quantifier:?
   number := REGEX:(\d+)
   object_contents := object_pair QUANTIFIED:HASH(0x1259fb730):*
   object_key := identifier | quoted_string
   object_pair := object_key REGEX:\s* TERMINAL:: REGEX:\s* return_expression
   object_value := ultimate_dot_notation | scalar_ref | literal | nested_array | nested_object
   outer_key := identifier | quoted_string
   perl_range := index TERMINAL:.. index
   positional_accessor := TERMINAL:. positive_number
   positive_number := REGEX:(\d+)
   property := object_key REGEX:\s* TERMINAL:: REGEX:\s* property_value
   property_accessor := TERMINAL:. identifier
   property_value := nested_array | nested_object | grouped_quantified_array | quantified_array | simple_array | ultimate_dot_notation | scalar_ref | literal
   python_slice := python_slice_start TERMINAL:: python_slice_end
   python_slice_end := index | empty_slice_part
   python_slice_start := index | empty_slice_part
   python_slice_with_step := python_slice_start TERMINAL:: python_slice_end TERMINAL:: step
   quantified_array := TERMINAL:[ REGEX:\s* quantified_element REGEX:\s* TERMINAL:]
   quantified_element := ultimate_dot_notation quantifier | scalar_ref quantifier
   quantifier := TERMINAL:* | TERMINAL:+ | TERMINAL:? | TERMINAL:{ REGEX:\s* positive_number REGEX:\s* TERMINAL:} | TERMINAL:{ REGEX:\s* positive_number REGEX:\s* TERMINAL:, REGEX:\s* TERMINAL:} | TERMINAL:{ REGEX:\s* positive_number REGEX:\s* TERMINAL:, REGEX:\s* positive_number REGEX:\s* TERMINAL:} | TERMINAL:{ REGEX:\s* TERMINAL:, REGEX:\s* positive_number REGEX:\s* TERMINAL:}
   quoted_string := REGEX:"([^"]*)"
   return_annotation := TERMINAL:-> REGEX:\s* return_expression
   return_expression := nested_array | nested_object | grouped_quantified_array | simple_nested_object | multi_property_object | quantified_array | simple_array | simple_object | ultimate_dot_notation | scalar_ref | literal
   scalar_ref := TERMINAL:$ positive_number
   simple_array := TERMINAL:[ REGEX:\s* array_element REGEX:\s* TERMINAL:]
   simple_nested_object := TERMINAL:{ REGEX:\s* outer_key REGEX:\s* TERMINAL:: REGEX:\s* inner_object REGEX:\s* TERMINAL:}
   simple_object := TERMINAL:{ REGEX:\s* object_key REGEX:\s* TERMINAL:: REGEX:\s* object_value REGEX:\s* TERMINAL:}
   single_index := index
   star_spec := TERMINAL:*
   step := index
   three_property_object := TERMINAL:{ REGEX:\s* property TERMINAL:, REGEX:\s* property TERMINAL:, REGEX:\s* property REGEX:\s* TERMINAL:}
   two_property_object := TERMINAL:{ REGEX:\s* property TERMINAL:, REGEX:\s* property REGEX:\s* TERMINAL:}
   ultimate_dot_notation := scalar_ref dot_path

🔄 Converting elimination result back to AST format...
📊 Converted back 60 rules

💀 LEFT-RECURSION STATUS: COMPLETELY ANNIHILATED!
DEBUG: Entered generate_sequence_parser for quoted_string
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'type' => 'atom',
            'value' => [
                         'regex',
                         '"([^"]*)"'
                       ]
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 1
DEBUG: Not enough elements for grouped quantifier check
DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'regex',
                       '"([^"]*)"'
                     ]
        };


==== STAGE 2: RETURN CODE GENERATION ====
Rule: quoted_string
Return annotation input: $VAR1 = [
          'return_scalar',
          '$1'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> $1'
DEBUG: Ultimate parser FAILED for '-> $1': Undefined subroutine &ultimate_return_annotation_perl_parser::parse_dot_path called at /Users/richarddje/Documents/github/airefactored/tools/generators/ultimate_return_annotation_perl_parser.pm line 1575.

Generated return code output: 'return $1;'
DEBUG: Entered generate_sequence_parser for array_accessor
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'type' => 'atom',
            'value' => [
                         'quoted_string',
                         '['
                       ]
          },
          {
            'value' => 'array_spec',
            'type' => 'atom'
          },
          {
            'type' => 'atom',
            'value' => [
                         'quoted_string',
                         ']'
                       ]
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 3
DEBUG: Not enough elements for grouped quantifier check
DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'quoted_string',
                       '['
                     ]
        };

DEBUG generate_sequence_rule: processing element 2: $VAR1 = {
          'value' => 'array_spec',
          'type' => 'atom'
        };

DEBUG: element->{value} = $VAR1 = 'array_spec';

DEBUG: rule_name_to_call = 'array_spec'
DEBUG generate_sequence_rule: processing element 3: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'quoted_string',
                       ']'
                     ]
        };


==== STAGE 2: RETURN CODE GENERATION ====
Rule: array_accessor
Return annotation input: $VAR1 = [
          'return_object',
          '{type: "array_access", spec: $2}'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> {type: "array_access", spec: $2}'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> {type: "array_access", spec: $2}'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_sequence_parser for simple_object
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'value' => [
                         'quoted_string',
                         '{'
                       ],
            'type' => 'atom'
          },
          {
            'value' => [
                         'regex',
                         '\\s*'
                       ],
            'type' => 'atom'
          },
          {
            'value' => 'object_key',
            'type' => 'atom'
          },
          {
            'type' => 'atom',
            'value' => [
                         'regex',
                         '\\s*'
                       ]
          },
          {
            'value' => [
                         'quoted_string',
                         ':'
                       ],
            'type' => 'atom'
          },
          {
            'type' => 'atom',
            'value' => [
                         'regex',
                         '\\s*'
                       ]
          },
          {
            'value' => 'object_value',
            'type' => 'atom'
          },
          {
            'type' => 'atom',
            'value' => [
                         'regex',
                         '\\s*'
                       ]
          },
          {
            'value' => [
                         'quoted_string',
                         '}'
                       ],
            'type' => 'atom'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 9
DEBUG: Last element: $VAR1 = {
          'value' => [
                       'quoted_string',
                       '}'
                     ],
          'type' => 'atom'
        };

DEBUG: Second last element: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'regex',
                       '\\s*'
                     ]
        };

DEBUG: Grouped quantifier pattern not detected
DEBUG: last_element->{type} eq 'atom': true
DEBUG: ref(last_element->{value}) eq 'HASH': false
DEBUG: second_last_element->{type} eq 'atom': true
DEBUG: ref(second_last_element->{value}) eq 'HASH': false
DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'value' => [
                       'quoted_string',
                       '{'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 2: $VAR1 = {
          'value' => [
                       'regex',
                       '\\s*'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 3: $VAR1 = {
          'value' => 'object_key',
          'type' => 'atom'
        };

DEBUG: element->{value} = $VAR1 = 'object_key';

DEBUG: rule_name_to_call = 'object_key'
DEBUG generate_sequence_rule: processing element 4: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'regex',
                       '\\s*'
                     ]
        };

DEBUG generate_sequence_rule: processing element 5: $VAR1 = {
          'value' => [
                       'quoted_string',
                       ':'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 6: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'regex',
                       '\\s*'
                     ]
        };

DEBUG generate_sequence_rule: processing element 7: $VAR1 = {
          'value' => 'object_value',
          'type' => 'atom'
        };

DEBUG: element->{value} = $VAR1 = 'object_value';

DEBUG: rule_name_to_call = 'object_value'
DEBUG generate_sequence_rule: processing element 8: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'regex',
                       '\\s*'
                     ]
        };

DEBUG generate_sequence_rule: processing element 9: $VAR1 = {
          'value' => [
                       'quoted_string',
                       '}'
                     ],
          'type' => 'atom'
        };


==== STAGE 2: RETURN CODE GENERATION ====
Rule: simple_object
Return annotation input: $VAR1 = [
          'return_object',
          '{type: "object", key: $3, value: $7}'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> {type: "object", key: $3, value: $7}'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> {type: "object", key: $3, value: $7}'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_or_parser for python_slice_end
DEBUG: Entered generate_sequence_parser for grouped_quantified_array
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'value' => [
                         'quoted_string',
                         '['
                       ],
            'type' => 'atom'
          },
          {
            'type' => 'atom',
            'value' => [
                         'regex',
                         '\\s*'
                       ]
          },
          {
            'type' => 'atom',
            'value' => 'grouped_element_list'
          },
          {
            'value' => [
                         'regex',
                         '\\s*'
                       ],
            'type' => 'atom'
          },
          {
            'value' => [
                         'quoted_string',
                         ']'
                       ],
            'type' => 'atom'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 5
DEBUG: Not enough elements for grouped quantifier check
DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'value' => [
                       'quoted_string',
                       '['
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 2: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'regex',
                       '\\s*'
                     ]
        };

DEBUG generate_sequence_rule: processing element 3: $VAR1 = {
          'type' => 'atom',
          'value' => 'grouped_element_list'
        };

DEBUG: element->{value} = $VAR1 = 'grouped_element_list';

DEBUG: rule_name_to_call = 'grouped_element_list'
DEBUG generate_sequence_rule: processing element 4: $VAR1 = {
          'value' => [
                       'regex',
                       '\\s*'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 5: $VAR1 = {
          'value' => [
                       'quoted_string',
                       ']'
                     ],
          'type' => 'atom'
        };


==== STAGE 2: RETURN CODE GENERATION ====
Rule: grouped_quantified_array
Return annotation input: $VAR1 = [
          'return_object',
          '{type: "grouped_quantified_array", groups: $3}'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> {type: "grouped_quantified_array", groups: $3}'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> {type: "grouped_quantified_array", groups: $3}'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_sequence_parser for return_annotation
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'type' => 'atom',
            'value' => [
                         'quoted_string',
                         '->'
                       ]
          },
          {
            'type' => 'atom',
            'value' => [
                         'regex',
                         '\\s*'
                       ]
          },
          {
            'type' => 'atom',
            'value' => 'return_expression'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 3
DEBUG: Not enough elements for grouped quantifier check
DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'quoted_string',
                       '->'
                     ]
        };

DEBUG generate_sequence_rule: processing element 2: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'regex',
                       '\\s*'
                     ]
        };

DEBUG generate_sequence_rule: processing element 3: $VAR1 = {
          'type' => 'atom',
          'value' => 'return_expression'
        };

DEBUG: element->{value} = $VAR1 = 'return_expression';

DEBUG: rule_name_to_call = 'return_expression'
DEBUG: Entered generate_sequence_parser for object_contents
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'value' => 'object_pair',
            'type' => 'atom'
          },
          {
            'quantifier' => '*',
            'element' => 'HASH(0x1259fb730)',
            'type' => 'quantified'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 2
DEBUG: Not enough elements for grouped quantifier check
DEBUG: Found quantified element in sequence: $VAR1 = {
          'quantifier' => '*',
          'element' => 'HASH(0x1259fb730)',
          'type' => 'quantified'
        };

DEBUG: Generating quantified sequence loop

==== STAGE 2: RETURN CODE GENERATION ====
Rule: object_contents
Return annotation input: $VAR1 = [
          'return_array',
          '[$1, $3*]'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> [$1, $3*]'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> [$1, $3*]'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_sequence_parser for array_contents
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'value' => 'return_expression',
            'type' => 'atom'
          },
          {
            'type' => 'quantified',
            'element' => 'HASH(0x12581c170)',
            'quantifier' => '*'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 2
DEBUG: Not enough elements for grouped quantifier check
DEBUG: Found quantified element in sequence: $VAR1 = {
          'type' => 'quantified',
          'element' => 'HASH(0x12581c170)',
          'quantifier' => '*'
        };

DEBUG: Generating quantified sequence loop

==== STAGE 2: RETURN CODE GENERATION ====
Rule: array_contents
Return annotation input: $VAR1 = [
          'return_array',
          '[$1, $3*]'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> [$1, $3*]'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> [$1, $3*]'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_or_parser for literal
DEBUG: Entered generate_sequence_parser for ultimate_dot_notation
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'type' => 'atom',
            'value' => 'scalar_ref'
          },
          {
            'value' => 'dot_path',
            'type' => 'atom'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 2
DEBUG: Not enough elements for grouped quantifier check
DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'type' => 'atom',
          'value' => 'scalar_ref'
        };

DEBUG: element->{value} = $VAR1 = 'scalar_ref';

DEBUG: rule_name_to_call = 'scalar_ref'
DEBUG generate_sequence_rule: processing element 2: $VAR1 = {
          'value' => 'dot_path',
          'type' => 'atom'
        };

DEBUG: element->{value} = $VAR1 = 'dot_path';

DEBUG: rule_name_to_call = 'dot_path'

==== STAGE 2: RETURN CODE GENERATION ====
Rule: ultimate_dot_notation
Return annotation input: $VAR1 = [
          'return_object',
          '{type: "ultimate_dot_notation", base: $1, path: $2}'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> {type: "ultimate_dot_notation", base: $1, path: $2}'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> {type: "ultimate_dot_notation", base: $1, path: $2}'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_or_parser for outer_key
DEBUG: Entered generate_or_parser for array_spec
DEBUG: Found return annotation - disabling optimization
DEBUG: Entered generate_or_parser for mixed_element
DEBUG: Entered generate_or_parser for quantified_element
DEBUG: Found return annotation - disabling optimization
DEBUG: Entered generate_or_parser for grouped_element_item
DEBUG: Found return annotation - disabling optimization
DEBUG: Entered generate_sequence_parser for mixed_expression
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'type' => 'atom',
            'value' => 'mixed_element'
          },
          {
            'type' => 'quantified',
            'element' => 'HASH(0x125a16388)',
            'quantifier' => '*'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 2
DEBUG: Not enough elements for grouped quantifier check
DEBUG: Found quantified element in sequence: $VAR1 = {
          'type' => 'quantified',
          'element' => 'HASH(0x125a16388)',
          'quantifier' => '*'
        };

DEBUG: Generating quantified sequence loop

==== STAGE 2: RETURN CODE GENERATION ====
Rule: mixed_expression
Return annotation input: $VAR1 = [
          'return_array',
          '[$1, $3*]'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> [$1, $3*]'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> [$1, $3*]'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_sequence_parser for two_property_object
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'value' => [
                         'quoted_string',
                         '{'
                       ],
            'type' => 'atom'
          },
          {
            'type' => 'atom',
            'value' => [
                         'regex',
                         '\\s*'
                       ]
          },
          {
            'value' => 'property',
            'type' => 'atom'
          },
          {
            'value' => [
                         'quoted_string',
                         ','
                       ],
            'type' => 'atom'
          },
          {
            'value' => [
                         'regex',
                         '\\s*'
                       ],
            'type' => 'atom'
          },
          {
            'type' => 'atom',
            'value' => 'property'
          },
          {
            'type' => 'atom',
            'value' => [
                         'regex',
                         '\\s*'
                       ]
          },
          {
            'type' => 'atom',
            'value' => [
                         'quoted_string',
                         '}'
                       ]
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 8
DEBUG: Last element: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'quoted_string',
                       '}'
                     ]
        };

DEBUG: Second last element: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'regex',
                       '\\s*'
                     ]
        };

DEBUG: Grouped quantifier pattern not detected
DEBUG: last_element->{type} eq 'atom': true
DEBUG: ref(last_element->{value}) eq 'HASH': false
DEBUG: second_last_element->{type} eq 'atom': true
DEBUG: ref(second_last_element->{value}) eq 'HASH': false
DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'value' => [
                       'quoted_string',
                       '{'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 2: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'regex',
                       '\\s*'
                     ]
        };

DEBUG generate_sequence_rule: processing element 3: $VAR1 = {
          'value' => 'property',
          'type' => 'atom'
        };

DEBUG: element->{value} = $VAR1 = 'property';

DEBUG: rule_name_to_call = 'property'
DEBUG generate_sequence_rule: processing element 4: $VAR1 = {
          'value' => [
                       'quoted_string',
                       ','
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 5: $VAR1 = {
          'value' => [
                       'regex',
                       '\\s*'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 6: $VAR1 = {
          'type' => 'atom',
          'value' => 'property'
        };

DEBUG: element->{value} = $VAR1 = 'property';

DEBUG: rule_name_to_call = 'property'
DEBUG generate_sequence_rule: processing element 7: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'regex',
                       '\\s*'
                     ]
        };

DEBUG generate_sequence_rule: processing element 8: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'quoted_string',
                       '}'
                     ]
        };


==== STAGE 2: RETURN CODE GENERATION ====
Rule: two_property_object
Return annotation input: $VAR1 = [
          'return_object',
          '{type: "multi_object", prop1: $3, prop2: $6}'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> {type: "multi_object", prop1: $3, prop2: $6}'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> {type: "multi_object", prop1: $3, prop2: $6}'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_sequence_parser for property_accessor
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'value' => [
                         'quoted_string',
                         '.'
                       ],
            'type' => 'atom'
          },
          {
            'type' => 'atom',
            'value' => 'identifier'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 2
DEBUG: Not enough elements for grouped quantifier check
DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'value' => [
                       'quoted_string',
                       '.'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 2: $VAR1 = {
          'type' => 'atom',
          'value' => 'identifier'
        };

DEBUG: element->{value} = $VAR1 = 'identifier';

DEBUG: rule_name_to_call = 'identifier'

==== STAGE 2: RETURN CODE GENERATION ====
Rule: property_accessor
Return annotation input: $VAR1 = [
          'return_object',
          '{type: "property", name: $2}'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> {type: "property", name: $2}'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> {type: "property", name: $2}'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_sequence_parser for three_property_object
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'value' => [
                         'quoted_string',
                         '{'
                       ],
            'type' => 'atom'
          },
          {
            'value' => [
                         'regex',
                         '\\s*'
                       ],
            'type' => 'atom'
          },
          {
            'value' => 'property',
            'type' => 'atom'
          },
          {
            'value' => [
                         'quoted_string',
                         ','
                       ],
            'type' => 'atom'
          },
          {
            'value' => [
                         'regex',
                         '\\s*'
                       ],
            'type' => 'atom'
          },
          {
            'type' => 'atom',
            'value' => 'property'
          },
          {
            'value' => [
                         'quoted_string',
                         ','
                       ],
            'type' => 'atom'
          },
          {
            'value' => [
                         'regex',
                         '\\s*'
                       ],
            'type' => 'atom'
          },
          {
            'value' => 'property',
            'type' => 'atom'
          },
          {
            'value' => [
                         'regex',
                         '\\s*'
                       ],
            'type' => 'atom'
          },
          {
            'type' => 'atom',
            'value' => [
                         'quoted_string',
                         '}'
                       ]
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 11
DEBUG: Last element: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'quoted_string',
                       '}'
                     ]
        };

DEBUG: Second last element: $VAR1 = {
          'value' => [
                       'regex',
                       '\\s*'
                     ],
          'type' => 'atom'
        };

DEBUG: Grouped quantifier pattern not detected
DEBUG: last_element->{type} eq 'atom': true
DEBUG: ref(last_element->{value}) eq 'HASH': false
DEBUG: second_last_element->{type} eq 'atom': true
DEBUG: ref(second_last_element->{value}) eq 'HASH': false
DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'value' => [
                       'quoted_string',
                       '{'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 2: $VAR1 = {
          'value' => [
                       'regex',
                       '\\s*'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 3: $VAR1 = {
          'value' => 'property',
          'type' => 'atom'
        };

DEBUG: element->{value} = $VAR1 = 'property';

DEBUG: rule_name_to_call = 'property'
DEBUG generate_sequence_rule: processing element 4: $VAR1 = {
          'value' => [
                       'quoted_string',
                       ','
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 5: $VAR1 = {
          'value' => [
                       'regex',
                       '\\s*'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 6: $VAR1 = {
          'type' => 'atom',
          'value' => 'property'
        };

DEBUG: element->{value} = $VAR1 = 'property';

DEBUG: rule_name_to_call = 'property'
DEBUG generate_sequence_rule: processing element 7: $VAR1 = {
          'value' => [
                       'quoted_string',
                       ','
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 8: $VAR1 = {
          'value' => [
                       'regex',
                       '\\s*'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 9: $VAR1 = {
          'value' => 'property',
          'type' => 'atom'
        };

DEBUG: element->{value} = $VAR1 = 'property';

DEBUG: rule_name_to_call = 'property'
DEBUG generate_sequence_rule: processing element 10: $VAR1 = {
          'value' => [
                       'regex',
                       '\\s*'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 11: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'quoted_string',
                       '}'
                     ]
        };


==== STAGE 2: RETURN CODE GENERATION ====
Rule: three_property_object
Return annotation input: $VAR1 = [
          'return_object',
          '{type: "multi_object", prop1: $3, prop2: $6, prop3: $9}'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> {type: "multi_object", prop1: $3, prop2: $6, prop3: $9}'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> {type: "multi_object", prop1: $3, prop2: $6, prop3: $9}'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_sequence_parser for nested_object
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'value' => [
                         'quoted_string',
                         '{'
                       ],
            'type' => 'atom'
          },
          {
            'value' => [
                         'regex',
                         '\\s*'
                       ],
            'type' => 'atom'
          },
          {
            'type' => 'quantified',
            'quantifier' => '?',
            'element' => 'object_contents'
          },
          {
            'type' => 'atom',
            'value' => [
                         'regex',
                         '\\s*'
                       ]
          },
          {
            'type' => 'atom',
            'value' => [
                         'quoted_string',
                         '}'
                       ]
          },
          {
            'type' => 'quantified',
            'quantifier' => '?',
            'element' => 'quantifier'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 6
DEBUG: Not enough elements for grouped quantifier check
DEBUG: Found quantified element in sequence: $VAR1 = {
          'type' => 'quantified',
          'quantifier' => '?',
          'element' => 'object_contents'
        };

DEBUG: Generating quantified sequence loop

==== STAGE 2: RETURN CODE GENERATION ====
Rule: nested_object
Return annotation input: $VAR1 = [
          'return_object',
          '{type: "object", contents: $3, quantified: $6}'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> {type: "object", contents: $3, quantified: $6}'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> {type: "object", contents: $3, quantified: $6}'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_sequence_parser for positive_number
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'value' => [
                         'regex',
                         '(\\d+)'
                       ],
            'type' => 'atom'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 1
DEBUG: Not enough elements for grouped quantifier check
DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'value' => [
                       'regex',
                       '(\\d+)'
                     ],
          'type' => 'atom'
        };


==== STAGE 2: RETURN CODE GENERATION ====
Rule: positive_number
Return annotation input: $VAR1 = [
          'return_object',
          '{type: "positive", value: $1}'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> {type: "positive", value: $1}'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> {type: "positive", value: $1}'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_sequence_parser for quantified_array
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'value' => [
                         'quoted_string',
                         '['
                       ],
            'type' => 'atom'
          },
          {
            'value' => [
                         'regex',
                         '\\s*'
                       ],
            'type' => 'atom'
          },
          {
            'value' => 'quantified_element',
            'type' => 'atom'
          },
          {
            'value' => [
                         'regex',
                         '\\s*'
                       ],
            'type' => 'atom'
          },
          {
            'value' => [
                         'quoted_string',
                         ']'
                       ],
            'type' => 'atom'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 5
DEBUG: Not enough elements for grouped quantifier check
DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'value' => [
                       'quoted_string',
                       '['
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 2: $VAR1 = {
          'value' => [
                       'regex',
                       '\\s*'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 3: $VAR1 = {
          'value' => 'quantified_element',
          'type' => 'atom'
        };

DEBUG: element->{value} = $VAR1 = 'quantified_element';

DEBUG: rule_name_to_call = 'quantified_element'
DEBUG generate_sequence_rule: processing element 4: $VAR1 = {
          'value' => [
                       'regex',
                       '\\s*'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 5: $VAR1 = {
          'value' => [
                       'quoted_string',
                       ']'
                     ],
          'type' => 'atom'
        };


==== STAGE 2: RETURN CODE GENERATION ====
Rule: quantified_array
Return annotation input: $VAR1 = [
          'return_object',
          '{type: "quantified_array", element: $3}'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> {type: "quantified_array", element: $3}'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> {type: "quantified_array", element: $3}'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_sequence_parser for python_slice_with_step
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'value' => 'python_slice_start',
            'type' => 'atom'
          },
          {
            'value' => [
                         'quoted_string',
                         ':'
                       ],
            'type' => 'atom'
          },
          {
            'type' => 'atom',
            'value' => 'python_slice_end'
          },
          {
            'type' => 'atom',
            'value' => [
                         'quoted_string',
                         ':'
                       ]
          },
          {
            'value' => 'step',
            'type' => 'atom'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 5
DEBUG: Not enough elements for grouped quantifier check
DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'value' => 'python_slice_start',
          'type' => 'atom'
        };

DEBUG: element->{value} = $VAR1 = 'python_slice_start';

DEBUG: rule_name_to_call = 'python_slice_start'
DEBUG generate_sequence_rule: processing element 2: $VAR1 = {
          'value' => [
                       'quoted_string',
                       ':'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 3: $VAR1 = {
          'type' => 'atom',
          'value' => 'python_slice_end'
        };

DEBUG: element->{value} = $VAR1 = 'python_slice_end';

DEBUG: rule_name_to_call = 'python_slice_end'
DEBUG generate_sequence_rule: processing element 4: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'quoted_string',
                       ':'
                     ]
        };

DEBUG generate_sequence_rule: processing element 5: $VAR1 = {
          'value' => 'step',
          'type' => 'atom'
        };

DEBUG: element->{value} = $VAR1 = 'step';

DEBUG: rule_name_to_call = 'step'

==== STAGE 2: RETURN CODE GENERATION ====
Rule: python_slice_with_step
Return annotation input: $VAR1 = [
          'return_object',
          '{type: "python_slice_step", start: $1, end: $3, step: $5}'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> {type: "python_slice_step", start: $1, end: $3, step: $5}'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> {type: "python_slice_step", start: $1, end: $3, step: $5}'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_sequence_parser for positional_accessor
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'type' => 'atom',
            'value' => [
                         'quoted_string',
                         '.'
                       ]
          },
          {
            'type' => 'atom',
            'value' => 'positive_number'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 2
DEBUG: Not enough elements for grouped quantifier check
DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'quoted_string',
                       '.'
                     ]
        };

DEBUG generate_sequence_rule: processing element 2: $VAR1 = {
          'type' => 'atom',
          'value' => 'positive_number'
        };

DEBUG: element->{value} = $VAR1 = 'positive_number';

DEBUG: rule_name_to_call = 'positive_number'

==== STAGE 2: RETURN CODE GENERATION ====
Rule: positional_accessor
Return annotation input: $VAR1 = [
          'return_object',
          '{type: "position", index: $2}'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> {type: "position", index: $2}'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> {type: "position", index: $2}'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_or_parser for multi_property_object
DEBUG: Entered generate_or_parser for return_expression
DEBUG: Entered generate_sequence_parser for nested_array
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'type' => 'atom',
            'value' => [
                         'quoted_string',
                         '['
                       ]
          },
          {
            'type' => 'atom',
            'value' => [
                         'regex',
                         '\\s*'
                       ]
          },
          {
            'quantifier' => '?',
            'element' => 'array_contents',
            'type' => 'quantified'
          },
          {
            'type' => 'atom',
            'value' => [
                         'regex',
                         '\\s*'
                       ]
          },
          {
            'type' => 'atom',
            'value' => [
                         'quoted_string',
                         ']'
                       ]
          },
          {
            'type' => 'quantified',
            'quantifier' => '?',
            'element' => 'quantifier'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 6
DEBUG: Not enough elements for grouped quantifier check
DEBUG: Found quantified element in sequence: $VAR1 = {
          'quantifier' => '?',
          'element' => 'array_contents',
          'type' => 'quantified'
        };

DEBUG: Generating quantified sequence loop

==== STAGE 2: RETURN CODE GENERATION ====
Rule: nested_array
Return annotation input: $VAR1 = [
          'return_object',
          '{type: "array", contents: $3, quantified: $6}'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> {type: "array", contents: $3, quantified: $6}'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> {type: "array", contents: $3, quantified: $6}'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_sequence_parser for identifier
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'value' => [
                         'regex',
                         '([a-zA-Z_]\\w*)'
                       ],
            'type' => 'atom'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 1
DEBUG: Not enough elements for grouped quantifier check
DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'value' => [
                       'regex',
                       '([a-zA-Z_]\\w*)'
                     ],
          'type' => 'atom'
        };


==== STAGE 2: RETURN CODE GENERATION ====
Rule: identifier
Return annotation input: $VAR1 = [
          'return_scalar',
          '$1'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> $1'
DEBUG: Ultimate parser FAILED for '-> $1': Undefined subroutine &ultimate_return_annotation_perl_parser::parse_dot_path called at /Users/richarddje/Documents/github/airefactored/tools/generators/ultimate_return_annotation_perl_parser.pm line 1575.

Generated return code output: 'return $1;'
DEBUG: Entered generate_sequence_parser for number
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'type' => 'atom',
            'value' => [
                         'regex',
                         '(\\d+)'
                       ]
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 1
DEBUG: Not enough elements for grouped quantifier check
DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'regex',
                       '(\\d+)'
                     ]
        };


==== STAGE 2: RETURN CODE GENERATION ====
Rule: number
Return annotation input: $VAR1 = [
          'return_scalar',
          '$1'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> $1'
DEBUG: Ultimate parser FAILED for '-> $1': Undefined subroutine &ultimate_return_annotation_perl_parser::parse_dot_path called at /Users/richarddje/Documents/github/airefactored/tools/generators/ultimate_return_annotation_perl_parser.pm line 1575.

Generated return code output: 'return $1;'
DEBUG: Entered generate_or_parser for grouped_element_list
DEBUG: Found return annotation - disabling optimization
DEBUG generate_quantified_code: element=$VAR1 = {
          'type' => 'quantified',
          'element' => 'HASH(0x125a26e50)',
          'quantifier' => '*'
        };

DEBUG generate_quantified_code: quantifier=*, parsed quant=$VAR1 = {
          'min' => 0,
          'max' => 999
        };

WARNING: Unhandled quantified element type in generate_quantified_code: $VAR1 = 'HASH(0x125a26e50)';

DEBUG: Entered generate_or_parser for inner_value
DEBUG: Entered generate_or_parser for object_value
DEBUG: Entered generate_sequence_parser for single_index
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'value' => 'index',
            'type' => 'atom'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 1
DEBUG: Not enough elements for grouped quantifier check
DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'value' => 'index',
          'type' => 'atom'
        };

DEBUG: element->{value} = $VAR1 = 'index';

DEBUG: rule_name_to_call = 'index'

==== STAGE 2: RETURN CODE GENERATION ====
Rule: single_index
Return annotation input: $VAR1 = [
          'return_object',
          '{type: "single_index", value: $1}'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> {type: "single_index", value: $1}'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> {type: "single_index", value: $1}'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_or_parser for index
DEBUG: Entered generate_or_parser for element_item
DEBUG: Entered generate_or_parser for quantifier
DEBUG: Found return annotation - disabling optimization
DEBUG: Entered generate_sequence_parser for object_pair
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'value' => 'object_key',
            'type' => 'atom'
          },
          {
            'value' => [
                         'regex',
                         '\\s*'
                       ],
            'type' => 'atom'
          },
          {
            'value' => [
                         'quoted_string',
                         ':'
                       ],
            'type' => 'atom'
          },
          {
            'type' => 'atom',
            'value' => [
                         'regex',
                         '\\s*'
                       ]
          },
          {
            'type' => 'atom',
            'value' => 'return_expression'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 5
DEBUG: Not enough elements for grouped quantifier check
DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'value' => 'object_key',
          'type' => 'atom'
        };

DEBUG: element->{value} = $VAR1 = 'object_key';

DEBUG: rule_name_to_call = 'object_key'
DEBUG generate_sequence_rule: processing element 2: $VAR1 = {
          'value' => [
                       'regex',
                       '\\s*'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 3: $VAR1 = {
          'value' => [
                       'quoted_string',
                       ':'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 4: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'regex',
                       '\\s*'
                     ]
        };

DEBUG generate_sequence_rule: processing element 5: $VAR1 = {
          'type' => 'atom',
          'value' => 'return_expression'
        };

DEBUG: element->{value} = $VAR1 = 'return_expression';

DEBUG: rule_name_to_call = 'return_expression'

==== STAGE 2: RETURN CODE GENERATION ====
Rule: object_pair
Return annotation input: $VAR1 = [
          'return_object',
          '{key: $1, value: $5}'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> {key: $1, value: $5}'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> {key: $1, value: $5}'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_or_parser for object_key
DEBUG: Entered generate_sequence_parser for property
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'type' => 'atom',
            'value' => 'object_key'
          },
          {
            'type' => 'atom',
            'value' => [
                         'regex',
                         '\\s*'
                       ]
          },
          {
            'value' => [
                         'quoted_string',
                         ':'
                       ],
            'type' => 'atom'
          },
          {
            'value' => [
                         'regex',
                         '\\s*'
                       ],
            'type' => 'atom'
          },
          {
            'type' => 'atom',
            'value' => 'property_value'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 5
DEBUG: Not enough elements for grouped quantifier check
DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'type' => 'atom',
          'value' => 'object_key'
        };

DEBUG: element->{value} = $VAR1 = 'object_key';

DEBUG: rule_name_to_call = 'object_key'
DEBUG generate_sequence_rule: processing element 2: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'regex',
                       '\\s*'
                     ]
        };

DEBUG generate_sequence_rule: processing element 3: $VAR1 = {
          'value' => [
                       'quoted_string',
                       ':'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 4: $VAR1 = {
          'value' => [
                       'regex',
                       '\\s*'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 5: $VAR1 = {
          'type' => 'atom',
          'value' => 'property_value'
        };

DEBUG: element->{value} = $VAR1 = 'property_value';

DEBUG: rule_name_to_call = 'property_value'

==== STAGE 2: RETURN CODE GENERATION ====
Rule: property
Return annotation input: $VAR1 = [
          'return_object',
          '{key: $1, value: $5}'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> {key: $1, value: $5}'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> {key: $1, value: $5}'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_or_parser for array_element
DEBUG: Entered generate_or_parser for python_slice_start
DEBUG: Entered generate_sequence_parser for python_slice
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'type' => 'atom',
            'value' => 'python_slice_start'
          },
          {
            'type' => 'atom',
            'value' => [
                         'quoted_string',
                         ':'
                       ]
          },
          {
            'value' => 'python_slice_end',
            'type' => 'atom'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 3
DEBUG: Not enough elements for grouped quantifier check
DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'type' => 'atom',
          'value' => 'python_slice_start'
        };

DEBUG: element->{value} = $VAR1 = 'python_slice_start';

DEBUG: rule_name_to_call = 'python_slice_start'
DEBUG generate_sequence_rule: processing element 2: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'quoted_string',
                       ':'
                     ]
        };

DEBUG generate_sequence_rule: processing element 3: $VAR1 = {
          'value' => 'python_slice_end',
          'type' => 'atom'
        };

DEBUG: element->{value} = $VAR1 = 'python_slice_end';

DEBUG: rule_name_to_call = 'python_slice_end'

==== STAGE 2: RETURN CODE GENERATION ====
Rule: python_slice
Return annotation input: $VAR1 = [
          'return_object',
          '{type: "python_slice", start: $1, end: $3}'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> {type: "python_slice", start: $1, end: $3}'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> {type: "python_slice", start: $1, end: $3}'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_or_parser for accessor
DEBUG: Entered generate_or_parser for inner_key
DEBUG: Entered generate_sequence_parser for perl_range
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'value' => 'index',
            'type' => 'atom'
          },
          {
            'value' => [
                         'quoted_string',
                         '..'
                       ],
            'type' => 'atom'
          },
          {
            'value' => 'index',
            'type' => 'atom'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 3
DEBUG: Not enough elements for grouped quantifier check
DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'value' => 'index',
          'type' => 'atom'
        };

DEBUG: element->{value} = $VAR1 = 'index';

DEBUG: rule_name_to_call = 'index'
DEBUG generate_sequence_rule: processing element 2: $VAR1 = {
          'value' => [
                       'quoted_string',
                       '..'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 3: $VAR1 = {
          'value' => 'index',
          'type' => 'atom'
        };

DEBUG: element->{value} = $VAR1 = 'index';

DEBUG: rule_name_to_call = 'index'

==== STAGE 2: RETURN CODE GENERATION ====
Rule: perl_range
Return annotation input: $VAR1 = [
          'return_object',
          '{type: "perl_range", start: $1, end: $3}'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> {type: "perl_range", start: $1, end: $3}'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> {type: "perl_range", start: $1, end: $3}'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_sequence_parser for negative_number
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'type' => 'atom',
            'value' => [
                         'quoted_string',
                         '-'
                       ]
          },
          {
            'value' => 'positive_number',
            'type' => 'atom'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 2
DEBUG: Not enough elements for grouped quantifier check
DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'quoted_string',
                       '-'
                     ]
        };

DEBUG generate_sequence_rule: processing element 2: $VAR1 = {
          'value' => 'positive_number',
          'type' => 'atom'
        };

DEBUG: element->{value} = $VAR1 = 'positive_number';

DEBUG: rule_name_to_call = 'positive_number'

==== STAGE 2: RETURN CODE GENERATION ====
Rule: negative_number
Return annotation input: $VAR1 = [
          'return_object',
          '{type: "negative", value: $2}'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> {type: "negative", value: $2}'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> {type: "negative", value: $2}'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_or_parser for element_sequence
DEBUG: Found return annotation - disabling optimization
DEBUG generate_quantified_code: element=$VAR1 = {
          'type' => 'quantified',
          'element' => 'HASH(0x1259f10e0)',
          'quantifier' => '*'
        };

DEBUG generate_quantified_code: quantifier=*, parsed quant=$VAR1 = {
          'max' => 999,
          'min' => 0
        };

WARNING: Unhandled quantified element type in generate_quantified_code: $VAR1 = 'HASH(0x1259f10e0)';

DEBUG: Entered generate_sequence_parser for dot_path
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'type' => 'quantified',
            'quantifier' => '+',
            'element' => 'accessor'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 1
DEBUG: Not enough elements for grouped quantifier check
DEBUG: Found quantified element in sequence: $VAR1 = {
          'type' => 'quantified',
          'quantifier' => '+',
          'element' => 'accessor'
        };

DEBUG: Generating quantified sequence loop

==== STAGE 2: RETURN CODE GENERATION ====
Rule: dot_path
Return annotation input: $VAR1 = [
          'return_array',
          '[$1*]'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> [$1*]'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> [$1*]'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_or_parser for property_value
DEBUG: Generating parser for index_list with rule_def:
$VAR1 = {
          'elements' => [
                          {
                            'type' => 'atom',
                            'value' => 'index'
                          },
                          {
                            'type' => 'quantified',
                            'quantifier' => '*',
                            'element' => 'HASH(0x1259fcf68)'
                          }
                        ],
          'type' => 'sequence',
          'return_annotation' => [
                                   'return_array',
                                   '[$1, $3*]'
                                 ]
        };
DEBUG: Entered generate_sequence_parser for index_list
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'type' => 'atom',
            'value' => 'index'
          },
          {
            'type' => 'quantified',
            'quantifier' => '*',
            'element' => 'HASH(0x1259fcf68)'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 2
DEBUG: Not enough elements for grouped quantifier check
DEBUG: Found quantified element in sequence: $VAR1 = {
          'type' => 'quantified',
          'quantifier' => '*',
          'element' => 'HASH(0x1259fcf68)'
        };

DEBUG: Generating quantified sequence loop

==== STAGE 2: RETURN CODE GENERATION ====
Rule: index_list
Return annotation input: $VAR1 = [
          'return_array',
          '[$1, $3*]'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> [$1, $3*]'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> [$1, $3*]'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_sequence_parser for simple_array
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'type' => 'atom',
            'value' => [
                         'quoted_string',
                         '['
                       ]
          },
          {
            'value' => [
                         'regex',
                         '\\s*'
                       ],
            'type' => 'atom'
          },
          {
            'type' => 'atom',
            'value' => 'array_element'
          },
          {
            'type' => 'atom',
            'value' => [
                         'regex',
                         '\\s*'
                       ]
          },
          {
            'type' => 'atom',
            'value' => [
                         'quoted_string',
                         ']'
                       ]
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 5
DEBUG: Not enough elements for grouped quantifier check
DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'quoted_string',
                       '['
                     ]
        };

DEBUG generate_sequence_rule: processing element 2: $VAR1 = {
          'value' => [
                       'regex',
                       '\\s*'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 3: $VAR1 = {
          'type' => 'atom',
          'value' => 'array_element'
        };

DEBUG: element->{value} = $VAR1 = 'array_element';

DEBUG: rule_name_to_call = 'array_element'
DEBUG generate_sequence_rule: processing element 4: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'regex',
                       '\\s*'
                     ]
        };

DEBUG generate_sequence_rule: processing element 5: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'quoted_string',
                       ']'
                     ]
        };


==== STAGE 2: RETURN CODE GENERATION ====
Rule: simple_array
Return annotation input: $VAR1 = [
          'return_object',
          '{type: "array", element: $3}'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> {type: "array", element: $3}'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> {type: "array", element: $3}'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_sequence_parser for simple_nested_object
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'value' => [
                         'quoted_string',
                         '{'
                       ],
            'type' => 'atom'
          },
          {
            'value' => [
                         'regex',
                         '\\s*'
                       ],
            'type' => 'atom'
          },
          {
            'value' => 'outer_key',
            'type' => 'atom'
          },
          {
            'type' => 'atom',
            'value' => [
                         'regex',
                         '\\s*'
                       ]
          },
          {
            'value' => [
                         'quoted_string',
                         ':'
                       ],
            'type' => 'atom'
          },
          {
            'value' => [
                         'regex',
                         '\\s*'
                       ],
            'type' => 'atom'
          },
          {
            'type' => 'atom',
            'value' => 'inner_object'
          },
          {
            'value' => [
                         'regex',
                         '\\s*'
                       ],
            'type' => 'atom'
          },
          {
            'value' => [
                         'quoted_string',
                         '}'
                       ],
            'type' => 'atom'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 9
DEBUG: Last element: $VAR1 = {
          'value' => [
                       'quoted_string',
                       '}'
                     ],
          'type' => 'atom'
        };

DEBUG: Second last element: $VAR1 = {
          'value' => [
                       'regex',
                       '\\s*'
                     ],
          'type' => 'atom'
        };

DEBUG: Grouped quantifier pattern not detected
DEBUG: last_element->{type} eq 'atom': true
DEBUG: ref(last_element->{value}) eq 'HASH': false
DEBUG: second_last_element->{type} eq 'atom': true
DEBUG: ref(second_last_element->{value}) eq 'HASH': false
DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'value' => [
                       'quoted_string',
                       '{'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 2: $VAR1 = {
          'value' => [
                       'regex',
                       '\\s*'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 3: $VAR1 = {
          'value' => 'outer_key',
          'type' => 'atom'
        };

DEBUG: element->{value} = $VAR1 = 'outer_key';

DEBUG: rule_name_to_call = 'outer_key'
DEBUG generate_sequence_rule: processing element 4: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'regex',
                       '\\s*'
                     ]
        };

DEBUG generate_sequence_rule: processing element 5: $VAR1 = {
          'value' => [
                       'quoted_string',
                       ':'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 6: $VAR1 = {
          'value' => [
                       'regex',
                       '\\s*'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 7: $VAR1 = {
          'type' => 'atom',
          'value' => 'inner_object'
        };

DEBUG: element->{value} = $VAR1 = 'inner_object';

DEBUG: rule_name_to_call = 'inner_object'
DEBUG generate_sequence_rule: processing element 8: $VAR1 = {
          'value' => [
                       'regex',
                       '\\s*'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 9: $VAR1 = {
          'value' => [
                       'quoted_string',
                       '}'
                     ],
          'type' => 'atom'
        };


==== STAGE 2: RETURN CODE GENERATION ====
Rule: simple_nested_object
Return annotation input: $VAR1 = [
          'return_object',
          '{type: "nested_object", key: $3, value: $7}'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> {type: "nested_object", key: $3, value: $7}'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> {type: "nested_object", key: $3, value: $7}'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_sequence_parser for inner_object
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'value' => [
                         'quoted_string',
                         '{'
                       ],
            'type' => 'atom'
          },
          {
            'type' => 'atom',
            'value' => [
                         'regex',
                         '\\s*'
                       ]
          },
          {
            'value' => 'inner_key',
            'type' => 'atom'
          },
          {
            'value' => [
                         'regex',
                         '\\s*'
                       ],
            'type' => 'atom'
          },
          {
            'type' => 'atom',
            'value' => [
                         'quoted_string',
                         ':'
                       ]
          },
          {
            'type' => 'atom',
            'value' => [
                         'regex',
                         '\\s*'
                       ]
          },
          {
            'value' => 'inner_value',
            'type' => 'atom'
          },
          {
            'type' => 'atom',
            'value' => [
                         'regex',
                         '\\s*'
                       ]
          },
          {
            'value' => [
                         'quoted_string',
                         '}'
                       ],
            'type' => 'atom'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 9
DEBUG: Last element: $VAR1 = {
          'value' => [
                       'quoted_string',
                       '}'
                     ],
          'type' => 'atom'
        };

DEBUG: Second last element: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'regex',
                       '\\s*'
                     ]
        };

DEBUG: Grouped quantifier pattern not detected
DEBUG: last_element->{type} eq 'atom': true
DEBUG: ref(last_element->{value}) eq 'HASH': false
DEBUG: second_last_element->{type} eq 'atom': true
DEBUG: ref(second_last_element->{value}) eq 'HASH': false
DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'value' => [
                       'quoted_string',
                       '{'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 2: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'regex',
                       '\\s*'
                     ]
        };

DEBUG generate_sequence_rule: processing element 3: $VAR1 = {
          'value' => 'inner_key',
          'type' => 'atom'
        };

DEBUG: element->{value} = $VAR1 = 'inner_key';

DEBUG: rule_name_to_call = 'inner_key'
DEBUG generate_sequence_rule: processing element 4: $VAR1 = {
          'value' => [
                       'regex',
                       '\\s*'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 5: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'quoted_string',
                       ':'
                     ]
        };

DEBUG generate_sequence_rule: processing element 6: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'regex',
                       '\\s*'
                     ]
        };

DEBUG generate_sequence_rule: processing element 7: $VAR1 = {
          'value' => 'inner_value',
          'type' => 'atom'
        };

DEBUG: element->{value} = $VAR1 = 'inner_value';

DEBUG: rule_name_to_call = 'inner_value'
DEBUG generate_sequence_rule: processing element 8: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'regex',
                       '\\s*'
                     ]
        };

DEBUG generate_sequence_rule: processing element 9: $VAR1 = {
          'value' => [
                       'quoted_string',
                       '}'
                     ],
          'type' => 'atom'
        };


==== STAGE 2: RETURN CODE GENERATION ====
Rule: inner_object
Return annotation input: $VAR1 = [
          'return_object',
          '{type: "inner_object", key: $3, value: $7}'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> {type: "inner_object", key: $3, value: $7}'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> {type: "inner_object", key: $3, value: $7}'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
DEBUG: Entered generate_sequence_parser for scalar_ref
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'type' => 'atom',
            'value' => [
                         'quoted_string',
                         '$'
                       ]
          },
          {
            'value' => 'positive_number',
            'type' => 'atom'
          }
        ];

DEBUG: Checking for grouped quantifier, filtered_elements count: 2
DEBUG: Not enough elements for grouped quantifier check
DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'quoted_string',
                       '$'
                     ]
        };

DEBUG generate_sequence_rule: processing element 2: $VAR1 = {
          'value' => 'positive_number',
          'type' => 'atom'
        };

DEBUG: element->{value} = $VAR1 = 'positive_number';

DEBUG: rule_name_to_call = 'positive_number'

==== STAGE 2: RETURN CODE GENERATION ====
Rule: scalar_ref
Return annotation input: $VAR1 = [
          'return_object',
          '{type: "scalar_ref", index: $2}'
        ];

DEBUG: Attempting to load ultimate return annotation parser...
DEBUG: Successfully loaded ultimate return annotation parser
DEBUG: Attempting to parse annotation: '-> {type: "scalar_ref", index: $2}'
DEBUG: Parse call completed, result defined: yes

==== STAGE 1: RAW PARSER OUTPUT ====
Input annotation: '-> {type: "scalar_ref", index: $2}'
Raw parser result:
$VAR1 = [
          undef,
          undef,
          [
            undef
          ]
        ];


==== STAGE 3: AST TO CODE CONVERSION ====
Return expression AST: $VAR1 = [
          undef
        ];

Return expression AST type: ARRAY
DEBUG: Unsupported AST structure in return_code_from_ast, falling back to legacy
Generated return code output: 'return \@results;  # Fallback for unsupported AST'
package Merged_ultimate_return_annotation; # Placeholder, will be replaced by tools/ast_transform.pl
use strict;
use warnings;

# Compiled regex patterns for speed
our %REGEXES = (
    'quoted_string_step1' => qr/"([^"]*)"/o,
    'array_accessor_step1' => qr/\Q[\E/o,
    'array_accessor_step3' => qr/\Q]\E/o,
    'simple_object_step1' => qr/\Q{\E/o,
    'simple_object_step2' => qr/\s*/o,
    'simple_object_step4' => qr/\s*/o,
    'simple_object_step5' => qr/\Q:\E/o,
    'simple_object_step6' => qr/\s*/o,
    'simple_object_step8' => qr/\s*/o,
    'simple_object_step9' => qr/\Q}\E/o,
    'grouped_quantified_array_step1' => qr/\Q[\E/o,
    'grouped_quantified_array_step2' => qr/\s*/o,
    'grouped_quantified_array_step4' => qr/\s*/o,
    'grouped_quantified_array_step5' => qr/\Q]\E/o,
    'return_annotation_step1' => qr/\Q->\E/o,
    'return_annotation_step2' => qr/\s*/o,
    'star_spec' => qr/\Q*\E/o,
    'grouped_element_item_alt0_0' => qr/\Q(\E/o,
    'grouped_element_item_alt0_1' => qr/\s*/o,
    'grouped_element_item_alt0_3' => qr/\s*/o,
    'grouped_element_item_alt0_4' => qr/\Q)\E/o,
    'two_property_object_step1' => qr/\Q{\E/o,
    'two_property_object_step2' => qr/\s*/o,
    'two_property_object_step4' => qr/\Q,\E/o,
    'two_property_object_step5' => qr/\s*/o,
    'two_property_object_step7' => qr/\s*/o,
    'two_property_object_step8' => qr/\Q}\E/o,
    'property_accessor_step1' => qr/\Q.\E/o,
    'three_property_object_step1' => qr/\Q{\E/o,
    'three_property_object_step2' => qr/\s*/o,
    'three_property_object_step4' => qr/\Q,\E/o,
    'three_property_object_step5' => qr/\s*/o,
    'three_property_object_step7' => qr/\Q,\E/o,
    'three_property_object_step8' => qr/\s*/o,
    'three_property_object_step10' => qr/\s*/o,
    'three_property_object_step11' => qr/\Q}\E/o,
    'nested_object_first' => qr/\Q{\E/o,
    'positive_number_step1' => qr/(\d+)/o,
    'quantified_array_step1' => qr/\Q[\E/o,
    'quantified_array_step2' => qr/\s*/o,
    'quantified_array_step4' => qr/\s*/o,
    'quantified_array_step5' => qr/\Q]\E/o,
    'python_slice_with_step_step2' => qr/\Q:\E/o,
    'python_slice_with_step_step4' => qr/\Q:\E/o,
    'positional_accessor_step1' => qr/\Q.\E/o,
    'nested_array_first' => qr/\Q[\E/o,
    'identifier_step1' => qr/([a-zA-Z_]\w*)/o,
    'number_step1' => qr/(\d+)/o,
    'quantifier_alt0_0' => qr/\Q*\E/o,
    'quantifier_alt1_0' => qr/\Q+\E/o,
    'quantifier_alt2_0' => qr/\Q?\E/o,
    'quantifier_alt3_0' => qr/\Q{\E/o,
    'quantifier_alt3_1' => qr/\s*/o,
    'quantifier_alt3_3' => qr/\s*/o,
    'quantifier_alt3_4' => qr/\Q}\E/o,
    'quantifier_alt4_0' => qr/\Q{\E/o,
    'quantifier_alt4_1' => qr/\s*/o,
    'quantifier_alt4_3' => qr/\s*/o,
    'quantifier_alt4_4' => qr/\Q,\E/o,
    'quantifier_alt4_5' => qr/\s*/o,
    'quantifier_alt4_6' => qr/\Q}\E/o,
    'quantifier_alt5_0' => qr/\Q{\E/o,
    'quantifier_alt5_1' => qr/\s*/o,
    'quantifier_alt5_3' => qr/\s*/o,
    'quantifier_alt5_4' => qr/\Q,\E/o,
    'quantifier_alt5_5' => qr/\s*/o,
    'quantifier_alt5_7' => qr/\s*/o,
    'quantifier_alt5_8' => qr/\Q}\E/o,
    'quantifier_alt6_0' => qr/\Q{\E/o,
    'quantifier_alt6_1' => qr/\s*/o,
    'quantifier_alt6_2' => qr/\Q,\E/o,
    'quantifier_alt6_3' => qr/\s*/o,
    'quantifier_alt6_5' => qr/\s*/o,
    'quantifier_alt6_6' => qr/\Q}\E/o,
    'object_pair_step2' => qr/\s*/o,
    'object_pair_step3' => qr/\Q:\E/o,
    'object_pair_step4' => qr/\s*/o,
    'property_step2' => qr/\s*/o,
    'property_step3' => qr/\Q:\E/o,
    'property_step4' => qr/\s*/o,
    'python_slice_step2' => qr/\Q:\E/o,
    'empty_slice_part' => qr/(?=:)/o,
    'empty_spec' => qr/(?=\])/o,
    'perl_range_step2' => qr/\Q..\E/o,
    'negative_number_step1' => qr/\Q-\E/o,
    'colon_spec' => qr/\Q:\E/o,
    'simple_array_step1' => qr/\Q[\E/o,
    'simple_array_step2' => qr/\s*/o,
    'simple_array_step4' => qr/\s*/o,
    'simple_array_step5' => qr/\Q]\E/o,
    'simple_nested_object_step1' => qr/\Q{\E/o,
    'simple_nested_object_step2' => qr/\s*/o,
    'simple_nested_object_step4' => qr/\s*/o,
    'simple_nested_object_step5' => qr/\Q:\E/o,
    'simple_nested_object_step6' => qr/\s*/o,
    'simple_nested_object_step8' => qr/\s*/o,
    'simple_nested_object_step9' => qr/\Q}\E/o,
    'inner_object_step1' => qr/\Q{\E/o,
    'inner_object_step2' => qr/\s*/o,
    'inner_object_step4' => qr/\s*/o,
    'inner_object_step5' => qr/\Q:\E/o,
    'inner_object_step6' => qr/\s*/o,
    'inner_object_step8' => qr/\s*/o,
    'inner_object_step9' => qr/\Q}\E/o,
    'scalar_ref_step1' => qr/\$/o
);

# Runtime helper functions
sub quantified_match {
    my ($input, $regex, $min, $max) = @_;
    my $count = 0;
    my $pos = pos($$input);
    
    # Optimized: Pre-compile regex with cache
    my $compiled_regex = qr/$regex/o;
    
    # Optimized: Tighter loop with fewer operations
    while ($count < $max) {
        if ($$input =~ /\G$compiled_regex/gc) {
            $count++;
        } else {
            last;
        }
    }
    
    if ($count >= $min) {
        return $count;
    } else {
        # Restore position on failure
        pos($$input) = $pos;
        return undef;
    }
}

sub quantified_rule {
    my ($input, $rule_ref, $min, $max) = @_;
    my $count = 0;
    my $checkpoint = pos($$input);
    
    # Optimized: Pre-allocate array for better performance
    my @results;
    $#results = $max - 1 if $max < 1000; # Pre-allocate for reasonable sizes
    
    my $result_idx = 0;
    while ($count < $max) {
        my $result = $rule_ref->($input);
        if (defined $result) {
            $results[$result_idx++] = $result;
            $count++;
        } else {
            last;
        }
    }
    
    if ($count >= $min) {
        # Optimized: Trim array to actual size
        $#results = $count - 1;
        return \@results;
    } else {
        # Restore position on failure
        pos($$input) = $checkpoint;
        return undef;
    }
}

sub collect_quantified_results {
    my ($element_num, $results_ref) = @_;
    my $element_index = $element_num - 1;
    
    # If the element is a quantified result (array), return it
    # If it's undef (zero matches), return empty array
    # Otherwise return single element in array
    my $element = $results_ref->[$element_index];
    
    if (!defined $element) {
        return [];
    } elsif (ref($element) eq 'ARRAY') {
        return $element;
    } else {
        return [$element];
    }
}

sub parse_quoted_string {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'quoted_string_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return $1;
}


sub parse_array_accessor {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'array_accessor_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_2 = parse_array_spec($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    unless ($$input =~ /\G$REGEXES{'array_accessor_step3'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return \@results;  # Fallback for unsupported AST
}


sub parse_simple_object {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'simple_object_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'simple_object_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_object_key($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'simple_object_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'simple_object_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'simple_object_step6'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_7 = parse_object_value($input);
    unless (defined $result_7) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_7;
    unless ($$input =~ /\G$REGEXES{'simple_object_step8'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'simple_object_step9'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return \@results;  # Fallback for unsupported AST
}


sub parse_python_slice_end {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_index($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_empty_slice_part($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_grouped_quantified_array {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'grouped_quantified_array_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'grouped_quantified_array_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_grouped_element_list($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'grouped_quantified_array_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'grouped_quantified_array_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return \@results;  # Fallback for unsupported AST
}


sub parse_return_annotation {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'return_annotation_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'return_annotation_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_return_expression($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    
    return \@results;
}


sub parse_object_contents {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Parse first required element
    my $result_1 = parse_object_pair($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    
    # Parse optional repeated grouped elements in loop
    while (1) {
        my $loop_start_pos = pos($$input);
        
        # Exit loop if no group code available
        if (!q{}) {
            pos($$input) = $loop_start_pos;
            last;
        }
        

    }

    
    return \@results;  # Fallback for unsupported AST
}


sub parse_array_contents {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Parse first required element
    my $result_1 = parse_return_expression($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    
    # Parse optional repeated grouped elements in loop
    while (1) {
        my $loop_start_pos = pos($$input);
        
        # Exit loop if no group code available
        if (!q{}) {
            pos($$input) = $loop_start_pos;
            last;
        }
        

    }

    
    return \@results;  # Fallback for unsupported AST
}


sub parse_literal {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_quoted_string($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_number($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_ultimate_dot_notation {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_scalar_ref($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    my $result_2 = parse_dot_path($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    
    return \@results;  # Fallback for unsupported AST
}


sub parse_outer_key {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_identifier($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_quoted_string($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_array_spec {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_empty_spec($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_star_spec($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_colon_spec($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_single_index($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_perl_range($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_python_slice($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_python_slice_with_step($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_index_list($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_mixed_expression($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_mixed_element {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_single_index($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_perl_range($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_python_slice($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_python_slice_with_step($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_star_spec {
    my ($input) = @_;
    return 1 if $$input =~ /\G$REGEXES{'star_spec'}/gc;
    return undef;
}


sub parse_quantified_element {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_ultimate_dot_notation($input)) && (parse_quantifier($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_scalar_ref($input)) && (parse_quantifier($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_grouped_element_item {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'grouped_element_item_alt0_0'}/gc) && ($$input =~ /\G$REGEXES{'grouped_element_item_alt0_1'}/gc) && (parse_group_content($input)) && ($$input =~ /\G$REGEXES{'grouped_element_item_alt0_3'}/gc) && ($$input =~ /\G$REGEXES{'grouped_element_item_alt0_4'}/gc) && (parse_quantifier($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_element_sequence($input)) && (parse_quantifier($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_mixed_expression {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Parse first required element
    my $result_1 = parse_mixed_element($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    
    # Parse optional repeated grouped elements in loop
    while (1) {
        my $loop_start_pos = pos($$input);
        
        # Exit loop if no group code available
        if (!q{}) {
            pos($$input) = $loop_start_pos;
            last;
        }
        

    }

    
    return \@results;  # Fallback for unsupported AST
}


sub parse_two_property_object {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'two_property_object_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'two_property_object_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_property($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'two_property_object_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'two_property_object_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_6 = parse_property($input);
    unless (defined $result_6) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_6;
    unless ($$input =~ /\G$REGEXES{'two_property_object_step7'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'two_property_object_step8'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return \@results;  # Fallback for unsupported AST
}


sub parse_property_accessor {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'property_accessor_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_2 = parse_identifier($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    
    return \@results;  # Fallback for unsupported AST
}


sub parse_three_property_object {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'three_property_object_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'three_property_object_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_property($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'three_property_object_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'three_property_object_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_6 = parse_property($input);
    unless (defined $result_6) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_6;
    unless ($$input =~ /\G$REGEXES{'three_property_object_step7'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'three_property_object_step8'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_9 = parse_property($input);
    unless (defined $result_9) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_9;
    unless ($$input =~ /\G$REGEXES{'three_property_object_step10'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'three_property_object_step11'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return \@results;  # Fallback for unsupported AST
}


sub parse_nested_object {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Parse first required element (regex match)
    unless ($$input =~ /\G$REGEXES{'nested_object_first'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture from first regex
    
    # Parse optional repeated grouped elements in loop
    while (1) {
        my $loop_start_pos = pos($$input);
        
        # Exit loop if no group code available
        if (!q{}) {
            pos($$input) = $loop_start_pos;
            last;
        }
        

    }

    
    return \@results;  # Fallback for unsupported AST
}


sub parse_positive_number {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'positive_number_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return \@results;  # Fallback for unsupported AST
}


sub parse_quantified_array {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'quantified_array_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'quantified_array_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_quantified_element($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'quantified_array_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'quantified_array_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return \@results;  # Fallback for unsupported AST
}


sub parse_python_slice_with_step {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_python_slice_start($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    unless ($$input =~ /\G$REGEXES{'python_slice_with_step_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_python_slice_end($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'python_slice_with_step_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_5 = parse_step($input);
    unless (defined $result_5) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_5;
    
    return \@results;  # Fallback for unsupported AST
}


sub parse_positional_accessor {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'positional_accessor_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_2 = parse_positive_number($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    
    return \@results;  # Fallback for unsupported AST
}


sub parse_multi_property_object {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_two_property_object($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_three_property_object($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_return_expression {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_nested_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_nested_object($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_grouped_quantified_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_simple_nested_object($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_multi_property_object($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_quantified_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_simple_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_simple_object($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_ultimate_dot_notation($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_scalar_ref($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_literal($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_nested_array {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Parse first required element (regex match)
    unless ($$input =~ /\G$REGEXES{'nested_array_first'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture from first regex
    
    # Parse optional repeated grouped elements in loop
    while (1) {
        my $loop_start_pos = pos($$input);
        
        # Exit loop if no group code available
        if (!q{}) {
            pos($$input) = $loop_start_pos;
            last;
        }
        

    }

    
    return \@results;  # Fallback for unsupported AST
}


sub parse_identifier {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'identifier_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return $1;
}


sub parse_number {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'number_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return $1;
}


sub parse_grouped_element_list {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_grouped_element_item($input)) && (1) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_grouped_element_item($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_inner_value {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_nested_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_nested_object($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_grouped_quantified_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_quantified_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_simple_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_ultimate_dot_notation($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_scalar_ref($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_literal($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_object_value {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_ultimate_dot_notation($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_scalar_ref($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_literal($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_nested_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_nested_object($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_single_index {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_index($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    
    return \@results;  # Fallback for unsupported AST
}


sub parse_index {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_positive_number($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_negative_number($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_step {
    my ($input) = @_;
    my $result = parse_index($input);
    if (defined $result) {
        return $result;
    }
    return undef;
}


sub parse_element_item {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_identifier($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_literal($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_quantifier {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt0_0'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt1_0'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt2_0'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt3_0'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt3_1'}/gc) && (parse_positive_number($input)) && ($$input =~ /\G$REGEXES{'quantifier_alt3_3'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt3_4'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt4_0'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt4_1'}/gc) && (parse_positive_number($input)) && ($$input =~ /\G$REGEXES{'quantifier_alt4_3'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt4_4'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt4_5'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt4_6'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt5_0'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt5_1'}/gc) && (parse_positive_number($input)) && ($$input =~ /\G$REGEXES{'quantifier_alt5_3'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt5_4'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt5_5'}/gc) && (parse_positive_number($input)) && ($$input =~ /\G$REGEXES{'quantifier_alt5_7'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt5_8'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt6_0'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt6_1'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt6_2'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt6_3'}/gc) && (parse_positive_number($input)) && ($$input =~ /\G$REGEXES{'quantifier_alt6_5'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt6_6'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_object_pair {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_object_key($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    unless ($$input =~ /\G$REGEXES{'object_pair_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'object_pair_step3'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'object_pair_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_5 = parse_return_expression($input);
    unless (defined $result_5) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_5;
    
    return \@results;  # Fallback for unsupported AST
}


sub parse_object_key {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_identifier($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_quoted_string($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_property {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_object_key($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    unless ($$input =~ /\G$REGEXES{'property_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'property_step3'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'property_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_5 = parse_property_value($input);
    unless (defined $result_5) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_5;
    
    return \@results;  # Fallback for unsupported AST
}


sub parse_array_element {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_ultimate_dot_notation($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_scalar_ref($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_literal($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_nested_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_nested_object($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_python_slice_start {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_index($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_empty_slice_part($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_python_slice {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_python_slice_start($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    unless ($$input =~ /\G$REGEXES{'python_slice_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_python_slice_end($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    
    return \@results;  # Fallback for unsupported AST
}


sub parse_accessor {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_property_accessor($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_positional_accessor($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_array_accessor($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_empty_slice_part {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'empty_slice_part'}/gc) {
        my @results = ($1);  # Capture regex result
        return 1;
    }
    return undef;
}


sub parse_empty_spec {
    my ($input) = @_;
    if ($$input =~ /\G$REGEXES{'empty_spec'}/gc) {
        my @results = ($1);  # Capture regex result
        return 1;
    }
    return undef;
}


sub parse_inner_key {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_identifier($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_quoted_string($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_perl_range {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_index($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    unless ($$input =~ /\G$REGEXES{'perl_range_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_index($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    
    return \@results;  # Fallback for unsupported AST
}


sub parse_negative_number {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'negative_number_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_2 = parse_positive_number($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    
    return \@results;  # Fallback for unsupported AST
}


sub parse_colon_spec {
    my ($input) = @_;
    return 1 if $$input =~ /\G$REGEXES{'colon_spec'}/gc;
    return undef;
}


sub parse_element_sequence {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_element_item($input)) && (1) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); (parse_element_item($input)) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_dot_path {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless (    # Standard sequence processing would go here) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return \@results;  # Fallback for unsupported AST
}


sub parse_property_value {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_nested_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_nested_object($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_grouped_quantified_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_quantified_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_simple_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_ultimate_dot_notation($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_scalar_ref($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_literal($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

sub parse_index_list {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    # Parse first required element
    my $result_1 = parse_index($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    
    # Parse optional repeated grouped elements in loop
    while (1) {
        my $loop_start_pos = pos($$input);
        
        # Exit loop if no group code available
        if (!q{}) {
            pos($$input) = $loop_start_pos;
            last;
        }
        

    }

    
    return \@results;  # Fallback for unsupported AST
}


sub parse_group_content {
    my ($input) = @_;
    my $result = parse_element_sequence($input);
    if (defined $result) {
        return $result;
    }
    return undef;
}


sub parse_simple_array {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'simple_array_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'simple_array_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_array_element($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'simple_array_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'simple_array_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return \@results;  # Fallback for unsupported AST
}


sub parse_simple_nested_object {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'simple_nested_object_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'simple_nested_object_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_outer_key($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'simple_nested_object_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'simple_nested_object_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'simple_nested_object_step6'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_7 = parse_inner_object($input);
    unless (defined $result_7) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_7;
    unless ($$input =~ /\G$REGEXES{'simple_nested_object_step8'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'simple_nested_object_step9'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return \@results;  # Fallback for unsupported AST
}


sub parse_inner_object {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'inner_object_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unl🎉 Success!
ess ($$input =~ /\G$REGEXES{'inner_object_step2'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_3 = parse_inner_key($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    unless ($$input =~ /\G$REGEXES{'inner_object_step4'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'inner_object_step5'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'inner_object_step6'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_7 = parse_inner_value($input);
    unless (defined $result_7) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_7;
    unless ($$input =~ /\G$REGEXES{'inner_object_step8'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    unless ($$input =~ /\G$REGEXES{'inner_object_step9'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return \@results;  # Fallback for unsupported AST
}


sub parse_scalar_ref {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'scalar_ref_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    my $result_2 = parse_positive_number($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    
    return \@results;  # Fallback for unsupported AST
}


# Main entry point
sub parse {
    my ($input) = @_;
    pos($$input) = 0;
    return parse_return_annotation($input);
}

1;
