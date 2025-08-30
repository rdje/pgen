=== Parsing step11_simple_nested.ebnf ===
RAW AST from EBNF parser:
$VAR1 = [
          [
            [
              'rule',
              'return_annotation'
            ],
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
          ],
          [
            [
              'rule',
              'return_expression'
            ],
            [
              'rule_reference',
              'simple_nested_object'
            ],
            [
              'operator',
              '|'
            ],
            [
              'rule_reference',
              'multi_property_object'
            ],
            [
              'operator',
              '|'
            ],
            [
              'rule_reference',
              'quantified_array'
            ],
            [
              'operator',
              '|'
            ],
            [
              'rule_reference',
              'simple_array'
            ],
            [
              'operator',
              '|'
            ],
            [
              'rule_reference',
              'simple_object'
            ],
            [
              'operator',
              '|'
            ],
            [
              'rule_reference',
              'scalar_ref'
            ]
          ],
          [
            [
              'rule',
              'simple_nested_object'
            ],
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
          [
            [
              'rule',
              'inner_object'
            ],
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
          ],
          [
            [
              'rule',
              'inner_value'
            ],
            [
              'rule_reference',
              'quantified_array'
            ],
            [
              'operator',
              '|'
            ],
            [
              'rule_reference',
              'simple_array'
            ],
            [
              'operator',
              '|'
            ],
            [
              'rule_reference',
              'scalar_ref'
            ],
            [
              'operator',
              '|'
            ],
            [
              'rule_reference',
              'literal'
            ]
          ],
          [
            [
              'rule',
              'multi_property_object'
            ],
            [
              'rule_reference',
              'two_property_object'
            ],
            [
              'operator',
              '|'
            ],
            [
              'rule_reference',
              'three_property_object'
            ]
          ],
          [
            [
              'rule',
              'two_property_object'
            ],
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
          [
            [
              'rule',
              'three_property_object'
            ],
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
          [
            [
              'rule',
              'property'
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
              'property_value'
            ],
            [
              'return_object',
              '{key: $1, value: $5}'
            ]
          ],
          [
            [
              'rule',
              'property_value'
            ],
            [
              'rule_reference',
              'quantified_array'
            ],
            [
              'operator',
              '|'
            ],
            [
              'rule_reference',
              'simple_array'
            ],
            [
              'operator',
              '|'
            ],
            [
              'rule_reference',
              'scalar_ref'
            ],
            [
              'operator',
              '|'
            ],
            [
              'rule_reference',
              'literal'
            ]
          ],
          [
            [
              'rule',
              'quantified_array'
            ],
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
          [
            [
              'rule',
              'quantified_element'
            ],
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
          [
            [
              'rule',
              'quantifier'
            ],
            [
              'quoted_string',
              '*'
            ],
            [
              'return_scalar',
              '"*"'
            ],
            [
              'operator',
              '|'
            ],
            [
              'quoted_string',
              '+'
            ],
            [
              'return_scalar',
              '"+"'
            ],
            [
              'operator',
              '|'
            ],
            [
              'quoted_string',
              '?'
            ],
            [
              'return_scalar',
              '"?"'
            ],
            [
              'operator',
              '|'
            ],
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
              'number'
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
            ],
            [
              'operator',
              '|'
            ],
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
              'number'
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
            ],
            [
              'operator',
              '|'
            ],
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
              'number'
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
              'number'
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
              'rule',
              'simple_array'
            ],
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
              'scalar_ref'
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
          [
            [
              'rule',
              'simple_object'
            ],
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
          [
            [
              'rule',
              'object_key'
            ],
            [
              'rule_reference',
              'identifier'
            ],
            [
              'operator',
              '|'
            ],
            [
              'rule_reference',
              'quoted_string'
            ]
          ],
          [
            [
              'rule',
              'outer_key'
            ],
            [
              'rule_reference',
              'identifier'
            ],
            [
              'operator',
              '|'
            ],
            [
              'rule_reference',
              'quoted_string'
            ]
          ],
          [
            [
              'rule',
              'inner_key'
            ],
            [
              'rule_reference',
              'identifier'
            ],
            [
              'operator',
              '|'
            ],
            [
              'rule_reference',
              'quoted_string'
            ]
          ],
          [
            [
              'rule',
              'object_value'
            ],
            [
              'rule_reference',
              'scalar_ref'
            ],
            [
              'operator',
              '|'
            ],
            [
              'rule_reference',
              'literal'
            ]
          ],
          [
            [
              'rule',
              'literal'
            ],
            [
              'rule_reference',
              'quoted_string'
            ],
            [
              'operator',
              '|'
            ],
            [
              'rule_reference',
              'number'
            ]
          ],
          [
            [
              'rule',
              'scalar_ref'
            ],
            [
              'quoted_string',
              '$'
            ],
            [
              'rule_reference',
              'number'
            ],
            [
              'return_object',
              '{type: "scalar_ref", index: $2}'
            ]
          ],
          [
            [
              'rule',
              'quoted_string'
            ],
            [
              'regex',
              '"([^"]*)"'
            ],
            [
              'return_scalar',
              '$1'
            ]
          ],
          [
            [
              'rule',
              'number'
            ],
            [
              'regex',
              '(\\d+)'
            ],
            [
              'return_scalar',
              '$1'
            ]
          ],
          [
            [
              'rule',
              'identifier'
            ],
            [
              'regex',
              '([a-zA-Z_]\\w*)'
            ],
            [
              'return_scalar',
              '$1'
            ]
          ]
        ];

=== Step 2: Group by OR operators ===
STEP 2 RESULT (Grouped by OR):
$VAR1 = [
          [
            'return_annotation',
            'SEQUENCE',
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
          [
            'return_expression',
            'OR',
            [
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
                  'scalar_ref'
                ]
              ]
            ]
          ],
          [
            'simple_nested_object',
            'SEQUENCE',
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
          [
            'inner_object',
            'SEQUENCE',
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
          ],
          [
            'inner_value',
            'OR',
            [
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
          ],
          [
            'multi_property_object',
            'OR',
            [
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
          ],
          [
            'two_property_object',
            'SEQUENCE',
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
          [
            'three_property_object',
            'SEQUENCE',
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
          [
            'property',
            'SEQUENCE',
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
          [
            'property_value',
            'OR',
            [
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
          ],
          [
            'quantified_array',
            'SEQUENCE',
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
          [
            'quantified_element',
            'SEQUENCE',
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
          ],
          [
            'quantifier',
            'OR',
            [
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
                  'number'
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
                  'number'
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
                  'number'
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
                  'number'
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
            ]
          ],
          [
            'simple_array',
            'SEQUENCE',
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
                'scalar_ref'
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
          [
            'simple_object',
            'SEQUENCE',
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
          [
            'object_key',
            'OR',
            [
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
          ],
          [
            'outer_key',
            'OR',
            [
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
          ],
          [
            'inner_key',
            'OR',
            [
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
          ],
          [
            'object_value',
            'OR',
            [
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
          ],
          [
            'literal',
            'OR',
            [
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
          ],
          [
            'scalar_ref',
            'SEQUENCE',
            [
              [
                'quoted_string',
                '$'
              ],
              [
                'rule_reference',
                'number'
              ],
              [
                'return_object',
                '{type: "scalar_ref", index: $2}'
              ]
            ]
          ],
          [
            'quoted_string',
            'SEQUENCE',
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
          [
            'number',
            'SEQUENCE',
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
          [
            'identifier',
            'SEQUENCE',
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
        ];

=== Step 2.5: Handle parentheses grouping ===
STEP 2.5 RESULT (Parentheses handled):
$VAR1 = [
          [
            'return_annotation',
            'SEQUENCE',
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
          [
            'return_expression',
            'OR',
            [
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
                  'scalar_ref'
                ]
              ]
            ]
          ],
          [
            'simple_nested_object',
            'SEQUENCE',
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
          [
            'inner_object',
            'SEQUENCE',
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
          ],
          [
            'inner_value',
            'OR',
            [
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
          ],
          [
            'multi_property_object',
            'OR',
            [
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
          ],
          [
            'two_property_object',
            'SEQUENCE',
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
          [
            'three_property_object',
            'SEQUENCE',
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
          [
            'property',
            'SEQUENCE',
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
          [
            'property_value',
            'OR',
            [
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
          ],
          [
            'quantified_array',
            'SEQUENCE',
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
          [
            'quantified_element',
            'SEQUENCE',
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
          ],
          [
            'quantifier',
            'OR',
            [
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
                  'number'
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
                  'number'
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
                  'number'
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
                  'number'
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
            ]
          ],
          [
            'simple_array',
            'SEQUENCE',
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
                'scalar_ref'
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
          [
            'simple_object',
            'SEQUENCE',
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
          [
            'object_key',
            'OR',
            [
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
          ],
          [
            'outer_key',
            'OR',
            [
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
          ],
          [
            'inner_key',
            'OR',
            [
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
          ],
          [
            'object_value',
            'OR',
            [
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
          ],
          [
            'literal',
            'OR',
            [
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
          ],
          [
            'scalar_ref',
            'SEQUENCE',
            [
              [
                'quoted_string',
                '$'
              ],
              [
                'rule_reference',
                'number'
              ],
              [
                'return_object',
                '{type: "scalar_ref", index: $2}'
              ]
            ]
          ],
          [
            'quoted_string',
            'SEQUENCE',
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
          [
            'number',
            'SEQUENCE',
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
          [
            'identifier',
            'SEQUENCE',
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
        ];

=== Step 3: Parse sequences ===
STEP 3 RESULT (Sequences parsed):
$VAR1 = [
          [
            'return_annotation',
            'SEQUENCE',
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
          [
            'return_expression',
            'OR',
            [
              [
                'ATOM',
                [
                  'rule_reference',
                  'simple_nested_object'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'multi_property_object'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'quantified_array'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'simple_array'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'simple_object'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'scalar_ref'
                ]
              ]
            ]
          ],
          [
            'simple_nested_object',
            'SEQUENCE',
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
          [
            'inner_object',
            'SEQUENCE',
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
          ],
          [
            'inner_value',
            'OR',
            [
              [
                'ATOM',
                [
                  'rule_reference',
                  'quantified_array'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'simple_array'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'scalar_ref'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'literal'
                ]
              ]
            ]
          ],
          [
            'multi_property_object',
            'OR',
            [
              [
                'ATOM',
                [
                  'rule_reference',
                  'two_property_object'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'three_property_object'
                ]
              ]
            ]
          ],
          [
            'two_property_object',
            'SEQUENCE',
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
          [
            'three_property_object',
            'SEQUENCE',
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
          [
            'property',
            'SEQUENCE',
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
          [
            'property_value',
            'OR',
            [
              [
                'ATOM',
                [
                  'rule_reference',
                  'quantified_array'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'simple_array'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'scalar_ref'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'literal'
                ]
              ]
            ]
          ],
          [
            'quantified_array',
            'SEQUENCE',
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
          [
            'quantified_element',
            'SEQUENCE',
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
          ],
          [
            'quantifier',
            'OR',
            [
              [
                'SEQUENCE',
                [
                  [
                    'quoted_string',
                    '*'
                  ],
                  [
                    'return_scalar',
                    '"*"'
                  ]
                ]
              ],
              [
                'SEQUENCE',
                [
                  [
                    'quoted_string',
                    '+'
                  ],
                  [
                    'return_scalar',
                    '"+"'
                  ]
                ]
              ],
              [
                'SEQUENCE',
                [
                  [
                    'quoted_string',
                    '?'
                  ],
                  [
                    'return_scalar',
                    '"?"'
                  ]
                ]
              ],
              [
                'SEQUENCE',
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
                    'number'
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
              ],
              [
                'SEQUENCE',
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
                    'number'
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
                ]
              ],
              [
                'SEQUENCE',
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
                    'number'
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
                    'number'
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
              ]
            ]
          ],
          [
            'simple_array',
            'SEQUENCE',
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
                'scalar_ref'
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
          [
            'simple_object',
            'SEQUENCE',
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
          [
            'object_key',
            'OR',
            [
              [
                'ATOM',
                [
                  'rule_reference',
                  'identifier'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'quoted_string'
                ]
              ]
            ]
          ],
          [
            'outer_key',
            'OR',
            [
              [
                'ATOM',
                [
                  'rule_reference',
                  'identifier'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'quoted_string'
                ]
              ]
            ]
          ],
          [
            'inner_key',
            'OR',
            [
              [
                'ATOM',
                [
                  'rule_reference',
                  'identifier'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'quoted_string'
                ]
              ]
            ]
          ],
          [
            'object_value',
            'OR',
            [
              [
                'ATOM',
                [
                  'rule_reference',
                  'scalar_ref'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'literal'
                ]
              ]
            ]
          ],
          [
            'literal',
            'OR',
            [
              [
                'ATOM',
                [
                  'rule_reference',
                  'quoted_string'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'number'
                ]
              ]
            ]
          ],
          [
            'scalar_ref',
            'SEQUENCE',
            [
              [
                'quoted_string',
                '$'
              ],
              [
                'rule_reference',
                'number'
              ],
              [
                'return_object',
                '{type: "scalar_ref", index: $2}'
              ]
            ]
          ],
          [
            'quoted_string',
            'SEQUENCE',
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
          [
            'number',
            'SEQUENCE',
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
          [
            'identifier',
            'SEQUENCE',
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
        ];

=== Step 4: Handle quantifiers ===
STEP 4 RESULT (Quantifiers handled):
$VAR1 = [
          [
            'return_annotation',
            'SEQUENCE',
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
          [
            'return_expression',
            'OR',
            [
              [
                'ATOM',
                [
                  'rule_reference',
                  'simple_nested_object'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'multi_property_object'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'quantified_array'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'simple_array'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'simple_object'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'scalar_ref'
                ]
              ]
            ]
          ],
          [
            'simple_nested_object',
            'SEQUENCE',
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
          [
            'inner_object',
            'SEQUENCE',
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
          ],
          [
            'inner_value',
            'OR',
            [
              [
                'ATOM',
                [
                  'rule_reference',
                  'quantified_array'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'simple_array'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'scalar_ref'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'literal'
                ]
              ]
            ]
          ],
          [
            'multi_property_object',
            'OR',
            [
              [
                'ATOM',
                [
                  'rule_reference',
                  'two_property_object'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'three_property_object'
                ]
              ]
            ]
          ],
          [
            'two_property_object',
            'SEQUENCE',
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
          [
            'three_property_object',
            'SEQUENCE',
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
          [
            'property',
            'SEQUENCE',
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
          [
            'property_value',
            'OR',
            [
              [
                'ATOM',
                [
                  'rule_reference',
                  'quantified_array'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'simple_array'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'scalar_ref'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'literal'
                ]
              ]
            ]
          ],
          [
            'quantified_array',
            'SEQUENCE',
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
          [
            'quantified_element',
            'SEQUENCE',
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
          ],
          [
            'quantifier',
            'OR',
            [
              [
                'SEQUENCE',
                [
                  [
                    'quoted_string',
                    '*'
                  ],
                  [
                    'return_scalar',
                    '"*"'
                  ]
                ]
              ],
              [
                'SEQUENCE',
                [
                  [
                    'quoted_string',
                    '+'
                  ],
                  [
                    'return_scalar',
                    '"+"'
                  ]
                ]
              ],
              [
                'SEQUENCE',
                [
                  [
                    'quoted_string',
                    '?'
                  ],
                  [
                    'return_scalar',
                    '"?"'
                  ]
                ]
              ],
              [
                'SEQUENCE',
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
                    'number'
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
              ],
              [
                'SEQUENCE',
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
                    'number'
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
                ]
              ],
              [
                'SEQUENCE',
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
                    'number'
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
                    'number'
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
              ]
            ]
          ],
          [
            'simple_array',
            'SEQUENCE',
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
                'scalar_ref'
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
          [
            'simple_object',
            'SEQUENCE',
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
          [
            'object_key',
            'OR',
            [
              [
                'ATOM',
                [
                  'rule_reference',
                  'identifier'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'quoted_string'
                ]
              ]
            ]
          ],
          [
            'outer_key',
            'OR',
            [
              [
                'ATOM',
                [
                  'rule_reference',
                  'identifier'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'quoted_string'
                ]
              ]
            ]
          ],
          [
            'inner_key',
            'OR',
            [
              [
                'ATOM',
                [
                  'rule_reference',
                  'identifier'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'quoted_string'
                ]
              ]
            ]
          ],
          [
            'object_value',
            'OR',
            [
              [
                'ATOM',
                [
                  'rule_reference',
                  'scalar_ref'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'literal'
                ]
              ]
            ]
          ],
          [
            'literal',
            'OR',
            [
              [
                'ATOM',
                [
                  'rule_reference',
                  'quoted_string'
                ]
              ],
              [
                'ATOM',
                [
                  'rule_reference',
                  'number'
                ]
              ]
            ]
          ],
          [
            'scalar_ref',
            'SEQUENCE',
            [
              [
                'quoted_string',
                '$'
              ],
              [
                'rule_reference',
                'number'
              ],
              [
                'return_object',
                '{type: "scalar_ref", index: $2}'
              ]
            ]
          ],
          [
            'quoted_string',
            'SEQUENCE',
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
          [
            'number',
            'SEQUENCE',
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
          [
            'identifier',
            'SEQUENCE',
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
        ];

=== Step 5: Build tree structure ===
STEP 5 RESULT (Tree structure):
$VAR1 = {
          'simple_object' => {
                               'type' => 'sequence',
                               'return_annotation' => [
                                                        'return_object',
                                                        '{type: "object", key: $3, value: $7}'
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
                                                 'type' => 'atom',
                                                 'value' => [
                                                              'regex',
                                                              '\\s*'
                                                            ]
                                               },
                                               {
                                                 'value' => [
                                                              'rule_reference',
                                                              'object_key'
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
                                                 'value' => [
                                                              'regex',
                                                              '\\s*'
                                                            ],
                                                 'type' => 'atom'
                                               },
                                               {
                                                 'value' => [
                                                              'rule_reference',
                                                              'object_value'
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
                                                              '}'
                                                            ]
                                               }
                                             ]
                             },
          'return_expression' => {
                                   'type' => 'or',
                                   'alternatives' => [
                                                       {
                                                         'value' => [
                                                                      'rule_reference',
                                                                      'simple_nested_object'
                                                                    ],
                                                         'type' => 'atom'
                                                       },
                                                       {
                                                         'value' => [
                                                                      'rule_reference',
                                                                      'multi_property_object'
                                                                    ],
                                                         'type' => 'atom'
                                                       },
                                                       {
                                                         'type' => 'atom',
                                                         'value' => [
                                                                      'rule_reference',
                                                                      'quantified_array'
                                                                    ]
                                                       },
                                                       {
                                                         'type' => 'atom',
                                                         'value' => [
                                                                      'rule_reference',
                                                                      'simple_array'
                                                                    ]
                                                       },
                                                       {
                                                         'value' => [
                                                                      'rule_reference',
                                                                      'simple_object'
                                                                    ],
                                                         'type' => 'atom'
                                                       },
                                                       {
                                                         'type' => 'atom',
                                                         'value' => [
                                                                      'rule_reference',
                                                                      'scalar_ref'
                                                                    ]
                                                       }
                                                     ]
                                 },
          'literal' => {
                         'type' => 'or',
                         'alternatives' => [
                                             {
                                               'value' => [
                                                            'rule_reference',
                                                            'quoted_string'
                                                          ],
                                               'type' => 'atom'
                                             },
                                             {
                                               'type' => 'atom',
                                               'value' => [
                                                            'rule_reference',
                                                            'number'
                                                          ]
                                             }
                                           ]
                       },
          'scalar_ref' => {
                            'type' => 'sequence',
                            'elements' => [
                                            {
                                              'type' => 'atom',
                                              'value' => [
                                                           'quoted_string',
                                                           '$'
                                                         ]
                                            },
                                            {
                                              'value' => [
                                                           'rule_reference',
                                                           'number'
                                                         ],
                                              'type' => 'atom'
                                            }
                                          ],
                            'return_annotation' => [
                                                     'return_object',
                                                     '{type: "scalar_ref", index: $2}'
                                                   ]
                          },
          'property' => {
                          'type' => 'sequence',
                          'return_annotation' => [
                                                   'return_object',
                                                   '{key: $1, value: $5}'
                                                 ],
                          'elements' => [
                                          {
                                            'value' => [
                                                         'rule_reference',
                                                         'object_key'
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
                                            'value' => [
                                                         'rule_reference',
                                                         'property_value'
                                                       ],
                                            'type' => 'atom'
                                          }
                                        ]
                        },
          'property_value' => {
                                'alternatives' => [
                                                    {
                                                      'type' => 'atom',
                                                      'value' => [
                                                                   'rule_reference',
                                                                   'quantified_array'
                                                                 ]
                                                    },
                                                    {
                                                      'type' => 'atom',
                                                      'value' => [
                                                                   'rule_reference',
                                                                   'simple_array'
                                                                 ]
                                                    },
                                                    {
                                                      'value' => [
                                                                   'rule_reference',
                                                                   'scalar_ref'
                                                                 ],
                                                      'type' => 'atom'
                                                    },
                                                    {
                                                      'value' => [
                                                                   'rule_reference',
                                                                   'literal'
                                                                 ],
                                                      'type' => 'atom'
                                                    }
                                                  ],
                                'type' => 'or'
                              },
          'quantifier' => {
                            'type' => 'or',
                            'alternatives' => [
                                                {
                                                  'type' => 'sequence',
                                                  'elements' => [
                                                                  {
                                                                    'value' => [
                                                                                 'quoted_string',
                                                                                 '*'
                                                                               ],
                                                                    'type' => 'atom'
                                                                  }
                                                                ],
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
                                                  'elements' => [
                                                                  {
                                                                    'type' => 'atom',
                                                                    'value' => [
                                                                                 'quoted_string',
                                                                                 '+'
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
                                                                                 'quoted_string',
                                                                                 '?'
                                                                               ],
                                                                    'type' => 'atom'
                                                                  }
                                                                ],
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
                                                                                 'number'
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
                                                  'return_annotation' => [
                                                                           'return_object',
                                                                           '{min: $3, max: $3}'
                                                                         ],
                                                  'type' => 'sequence'
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
                                                                                 'number'
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
                                                                                 ','
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
                                                  'return_annotation' => [
                                                                           'return_object',
                                                                           '{min: $3, max: "inf"}'
                                                                         ]
                                                },
                                                {
                                                  'type' => 'sequence',
                                                  'return_annotation' => [
                                                                           'return_object',
                                                                           '{min: $3, max: $7}'
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
                                                                    'type' => 'atom',
                                                                    'value' => [
                                                                                 'rule_reference',
                                                                                 'number'
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
                                                                    'value' => [
                                                                                 'rule_reference',
                                                                                 'number'
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
                                                                                 '}'
                                                                               ]
                                                                  }
                                                                ]
                                                }
                                              ]
                          },
          'return_annotation' => {
                                   'type' => 'sequence',
                                   'elements' => [
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
                                                     'value' => [
                                                                  'rule_reference',
                                                                  'return_expression'
                                                                ],
                                                     'type' => 'atom'
                                                   }
                                                 ],
                                   'return_annotation' => undef
                                 },
          'quantified_array' => {
                                  'return_annotation' => [
                                                           'return_object',
                                                           '{type: "quantified_array", element: $3}'
                                                         ],
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
                                                                 'quantified_element'
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
                                                                 ']'
                                                               ]
                                                  }
                                                ],
                                  'type' => 'sequence'
                                },
          'identifier' => {
                            'return_annotation' => [
                                                     'return_scalar',
                                                     '$1'
                                                   ],
                            'elements' => [
                                            {
                                              'type' => 'atom',
                                              'value' => [
                                                           'regex',
                                                           '([a-zA-Z_]\\w*)'
                                                         ]
                                            }
                                          ],
                            'type' => 'sequence'
                          },
          'outer_key' => {
                           'type' => 'or',
                           'alternatives' => [
                                               {
                                                 'value' => [
                                                              'rule_reference',
                                                              'identifier'
                                                            ],
                                                 'type' => 'atom'
                                               },
                                               {
                                                 'value' => [
                                                              'rule_reference',
                                                              'quoted_string'
                                                            ],
                                                 'type' => 'atom'
                                               }
                                             ]
                         },
          'inner_value' => {
                             'type' => 'or',
                             'alternatives' => [
                                                 {
                                                   'type' => 'atom',
                                                   'value' => [
                                                                'rule_reference',
                                                                'quantified_array'
                                                              ]
                                                 },
                                                 {
                                                   'type' => 'atom',
                                                   'value' => [
                                                                'rule_reference',
                                                                'simple_array'
                                                              ]
                                                 },
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
                                                                'literal'
                                                              ]
                                                 }
                                               ]
                           },
          'inner_key' => {
                           'alternatives' => [
                                               {
                                                 'type' => 'atom',
                                                 'value' => [
                                                              'rule_reference',
                                                              'identifier'
                                                            ]
                                               },
                                               {
                                                 'value' => [
                                                              'rule_reference',
                                                              'quoted_string'
                                                            ],
                                                 'type' => 'atom'
                                               }
                                             ],
                           'type' => 'or'
                         },
          'inner_object' => {
                              'type' => 'sequence',
                              'return_annotation' => [
                                                       'return_object',
                                                       '{type: "inner_object", key: $3, value: $7}'
                                                     ],
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
                                                             'inner_key'
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
                                                             ':'
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
                                            ]
                            },
          'number' => {
                        'return_annotation' => [
                                                 'return_scalar',
                                                 '$1'
                                               ],
                        'elements' => [
                                        {
                                          'value' => [
                                                       'regex',
                                                       '(\\d+)'
                                                     ],
                                          'type' => 'atom'
                                        }
                                      ],
                        'type' => 'sequence'
                      },
          'two_property_object' => {
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
                                     'return_annotation' => [
                                                              'return_object',
                                                              '{type: "multi_object", prop1: $3, prop2: $6}'
                                                            ],
                                     'type' => 'sequence'
                                   },
          'three_property_object' => {
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
                                                                      'property'
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
                                                         'type' => 'atom',
                                                         'value' => [
                                                                      'quoted_string',
                                                                      ','
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
                                                                      'property'
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
                                                                      '}'
                                                                    ]
                                                       }
                                                     ],
                                       'return_annotation' => [
                                                                'return_object',
                                                                '{type: "multi_object", prop1: $3, prop2: $6, prop3: $9}'
                                                              ],
                                       'type' => 'sequence'
                                     },
          'object_key' => {
                            'alternatives' => [
                                                {
                                                  'value' => [
                                                               'rule_reference',
                                                               'identifier'
                                                             ],
                                                  'type' => 'atom'
                                                },
                                                {
                                                  'type' => 'atom',
                                                  'value' => [
                                                               'rule_reference',
                                                               'quoted_string'
                                                             ]
                                                }
                                              ],
                            'type' => 'or'
                          },
          'simple_array' => {
                              'type' => 'sequence',
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
                                                'value' => [
                                                             'regex',
                                                             '\\s*'
                                                           ],
                                                'type' => 'atom'
                                              },
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
                                                             'regex',
                                                             '\\s*'
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
          'quantified_element' => {
                                    'elements' => [
                                                    {
                                                      'type' => 'atom',
                                                      'value' => [
                                                                   'rule_reference',
                                                                   'scalar_ref'
                                                                 ]
                                                    },
                                                    {
                                                      'value' => [
                                                                   'rule_reference',
                                                                   'quantifier'
                                                                 ],
                                                      'type' => 'atom'
                                                    }
                                                  ],
                                    'return_annotation' => [
                                                             'return_object',
                                                             '{scalar: $1, quantifier: $2}'
                                                           ],
                                    'type' => 'sequence'
                                  },
          'quoted_string' => {
                               'return_annotation' => [
                                                        'return_scalar',
                                                        '$1'
                                                      ],
                               'elements' => [
                                               {
                                                 'value' => [
                                                              'regex',
                                                              '"([^"]*)"'
                                                            ],
                                                 'type' => 'atom'
                                               }
                                             ],
                               'type' => 'sequence'
                             },
          'object_value' => {
                              'type' => 'or',
                              'alternatives' => [
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'scalar_ref'
                                                               ],
                                                    'type' => 'atom'
                                                  },
                                                  {
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'literal'
                                                               ],
                                                    'type' => 'atom'
                                                  }
                                                ]
                            },
          'multi_property_object' => {
                                       'type' => 'or',
                                       'alternatives' => [
                                                           {
                                                             'value' => [
                                                                          'rule_reference',
                                                                          'two_property_object'
                                                                        ],
                                                             'type' => 'atom'
                                                           },
                                                           {
                                                             'value' => [
                                                                          'rule_reference',
                                                                          'three_property_object'
                                                                        ],
                                                             'type' => 'atom'
                                                           }
                                                         ]
                                     },
          'simple_nested_object' => {
                                      'type' => 'sequence',
                                      'return_annotation' => [
                                                               'return_object',
                                                               '{type: "nested_object", key: $3, value: $7}'
                                                             ],
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
                                                        'type' => 'atom',
                                                        'value' => [
                                                                     'rule_reference',
                                                                     'outer_key'
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
                                                        'type' => 'atom',
                                                        'value' => [
                                                                     'rule_reference',
                                                                     'inner_object'
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
                                    }
        };
RULE ORDER: return_annotation, return_expression, simple_nested_object, inner_object, inner_value, multi_property_object, two_property_object, three_property_object, property, property_value, quantified_array, quantified_element, quantifier, simple_array, simple_object, object_key, outer_key, inner_key, object_value, literal, scalar_ref, quoted_string, number, identifier

=== Step 6: Generate parser code ===
🚀 DEPLOYING LEFT-RECURSION NUCLEAR ELIMINATOR!
🎯 Target: Complete annihilation of all recursion forms
======================================================================

🔄 Converting AST format to elimination format...
📊 Converted 24 rules
🏷️ Stored annotations for 23 rules
📋 Grammar before elimination:
   identifier := REGEX:([a-zA-Z_]\w*)
   inner_key := rule_reference:identifier | rule_reference:quoted_string
   inner_object := quoted_string:{ REGEX:\s* rule_reference:inner_key REGEX:\s* quoted_string:: REGEX:\s* rule_reference:inner_value REGEX:\s* quoted_string:}
   inner_value := rule_reference:quantified_array | rule_reference:simple_array | rule_reference:scalar_ref | rule_reference:literal
   literal := rule_reference:quoted_string | rule_reference:number
   multi_property_object := rule_reference:two_property_object | rule_reference:three_property_object
   number := REGEX:(\d+)
   object_key := rule_reference:identifier | rule_reference:quoted_string
   object_value := rule_reference:scalar_ref | rule_reference:literal
   outer_key := rule_reference:identifier | rule_reference:quoted_string
   property := rule_reference:object_key REGEX:\s* quoted_string:: REGEX:\s* rule_reference:property_value
   property_value := rule_reference:quantified_array | rule_reference:simple_array | rule_reference:scalar_ref | rule_reference:literal
   quantified_array := quoted_string:[ REGEX:\s* rule_reference:quantified_element REGEX:\s* quoted_string:]
   quantified_element := rule_reference:scalar_ref rule_reference:quantifier
   quantifier := quoted_string:* | quoted_string:+ | quoted_string:? | quoted_string:{ REGEX:\s* rule_reference:number REGEX:\s* quoted_string:} | quoted_string:{ REGEX:\s* rule_reference:number REGEX:\s* quoted_string:, REGEX:\s* quoted_string:} | quoted_string:{ REGEX:\s* rule_reference:number REGEX:\s* quoted_string:, REGEX:\s* rule_reference:number REGEX:\s* quoted_string:}
   quoted_string := REGEX:"([^"]*)"
   return_annotation := quoted_string:-> REGEX:\s* rule_reference:return_expression
   return_expression := rule_reference:simple_nested_object | rule_reference:multi_property_object | rule_reference:quantified_array | rule_reference:simple_array | rule_reference:simple_object | rule_reference:scalar_ref
   scalar_ref := quoted_string:$ rule_reference:number
   simple_array := quoted_string:[ REGEX:\s* rule_reference:scalar_ref REGEX:\s* quoted_string:]
   simple_nested_object := quoted_string:{ REGEX:\s* rule_reference:outer_key REGEX:\s* quoted_string:: REGEX:\s* rule_reference:inner_object REGEX:\s* quoted_string:}
   simple_object := quoted_string:{ REGEX:\s* rule_reference:object_key REGEX:\s* quoted_string:: REGEX:\s* rule_reference:object_value REGEX:\s* quoted_string:}
   three_property_object := quoted_string:{ REGEX:\s* rule_reference:property quoted_string:, REGEX:\s* rule_reference:property quoted_string:, REGEX:\s* rule_reference:property REGEX:\s* quoted_string:}
   two_property_object := quoted_string:{ REGEX:\s* rule_reference:property quoted_string:, REGEX:\s* rule_reference:property REGEX:\s* quoted_string:}

print() on closed filehandle STDERR at ./integrate_left_recursion_killer.pl line 180.
print() on closed filehandle STDERR at ./integrate_left_recursion_killer.pl line 225.
print() on closed filehandle STDERR at ./integrate_left_recursion_killer.pl line 374.
DEBUG: Grammar after left-recursion elimination:
$VAR1 = {
          'object_value' => {
                              'type' => 'or',
                              'alternatives' => [
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
                                                                 'literal'
                                                               ]
                                                  }
                                                ]
                            },
          'multi_property_object' => {
                                       'alternatives' => [
                                                           {
                                                             'value' => [
                                                                          'rule_reference',
                                                                          'two_property_object'
                                                                        ],
                                                             'type' => 'atom'
                                                           },
                                                           {
                                                             'type' => 'atom',
                                                             'value' => [
                                                                          'rule_reference',
                                                                          'three_property_object'
                                                                        ]
                                                           }
                                                         ],
                                       'type' => 'or'
                                     },
          'simple_nested_object' => {
                                      'return_annotation' => [
                                                               'return_object',
                                                               '{type: "nested_object", key: $3, value: $7}'
                                                             ],
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
                                                        'value' => [
                                                                     'rule_reference',
                                                                     'inner_object'
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
                                      'type' => 'sequence'
                                    },
          'simple_array' => {
                              'return_annotation' => [
                                                       'return_object',
                                                       '{type: "array", element: $3}'
                                                     ],
                              'elements' => [
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
                                                'value' => [
                                                             'rule_reference',
                                                             'scalar_ref'
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
                              'type' => 'sequence'
                            },
          'quoted_string' => {
                               'return_annotation' => [
                                                        'return_scalar',
                                                        '$1'
                                                      ],
                               'elements' => [
                                               {
                                                 'type' => 'atom',
                                                 'value' => [
                                                              'regex',
                                                              '"([^"]*)"'
                                                            ]
                                               }
                                             ],
                               'type' => 'sequence'
                             },
          'quantified_element' => {
                                    'type' => 'sequence',
                                    'elements' => [
                                                    {
                                                      'value' => [
                                                                   'rule_reference',
                                                                   'scalar_ref'
                                                                 ],
                                                      'type' => 'atom'
                                                    },
                                                    {
                                                      'value' => [
                                                                   'rule_reference',
                                                                   'quantifier'
                                                                 ],
                                                      'type' => 'atom'
                                                    }
                                                  ],
                                    'return_annotation' => [
                                                             'return_object',
                                                             '{scalar: $1, quantifier: $2}'
                                                           ]
                                  },
          'object_key' => {
                            'type' => 'or',
                            'alternatives' => [
                                                {
                                                  'type' => 'atom',
                                                  'value' => [
                                                               'rule_reference',
                                                               'identifier'
                                                             ]
                                                },
                                                {
                                                  'type' => 'atom',
                                                  'value' => [
                                                               'rule_reference',
                                                               'quoted_string'
                                                             ]
                                                }
                                              ]
                          },
          'number' => {
                        'return_annotation' => [
                                                 'return_scalar',
                                                 '$1'
                                               ],
                        'elements' => [
                                        {
                                          'value' => [
                                                       'regex',
                                                       '(\\d+)'
                                                     ],
                                          'type' => 'atom'
                                        }
                                      ],
                        'type' => 'sequence'
                      },
          'two_property_object' => {
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
                                     'return_annotation' => [
                                                              'return_object',
                                                              '{type: "multi_object", prop1: $3, prop2: $6}'
                                                            ]
                                   },
          'three_property_object' => {
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
                                                         'type' => 'atom',
                                                         'value' => [
                                                                      'rule_reference',
                                                                      'property'
                                                                    ]
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
                                       'return_annotation' => [
                                                                'return_object',
                                                                '{type: "multi_object", prop1: $3, prop2: $6, prop3: $9}'
                                                              ]
                                     },
          'inner_value' => {
                             'alternatives' => [
                                                 {
                                                   'type' => 'atom',
                                                   'value' => [
                                                                'rule_reference',
                                                                'quantified_array'
                                                              ]
                                                 },
                                                 {
                                                   'value' => [
                                                                'rule_reference',
                                                                'simple_array'
                                                              ],
                                                   'type' => 'atom'
                                                 },
                                                 {
                                                   'value' => [
                                                                'rule_reference',
                                                                'scalar_ref'
                                                              ],
                                                   'type' => 'atom'
                                                 },
                                                 {
                                                   'value' => [
                                                                'rule_reference',
                                                                'literal'
                                                              ],
                                                   'type' => 'atom'
                                                 }
                                               ],
                             'type' => 'or'
                           },
          'inner_object' => {
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
                                                'value' => [
                                                             'rule_reference',
                                                             'inner_key'
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
                                                'value' => [
                                                             'rule_reference',
                                                             'inner_value'
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
                              'return_annotation' => [
                                                       'return_object',
                                                       '{type: "inner_object", key: $3, value: $7}'
                                                     ]
                            },
          'inner_key' => {
                           'type' => 'or',
                           'alternatives' => [
                                               {
                                                 'value' => [
                                                              'rule_reference',
                                                              'identifier'
                                                            ],
                                                 'type' => 'atom'
                                               },
                                               {
                                                 'value' => [
                                                              'rule_reference',
                                                              'quoted_string'
                                                            ],
                                                 'type' => 'atom'
                                               }
                                             ]
                         },
          'quantified_array' => {
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
                                                    'type' => 'atom',
                                                    'value' => [
                                                                 'rule_reference',
                                                                 'quantified_element'
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
                                                                 ']'
                                                               ]
                                                  }
                                                ],
                                  'return_annotation' => [
                                                           'return_object',
                                                           '{type: "quantified_array", element: $3}'
                                                         ],
                                  'type' => 'sequence'
                                },
          'identifier' => {
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
                          },
          'outer_key' => {
                           'alternatives' => [
                                               {
                                                 'value' => [
                                                              'rule_reference',
                                                              'identifier'
                                                            ],
                                                 'type' => 'atom'
                                               },
                                               {
                                                 'value' => [
                                                              'rule_reference',
                                                              'quoted_string'
                                                            ],
                                                 'type' => 'atom'
                                               }
                                             ],
                           'type' => 'or'
                         },
          'literal' => {
                         'alternatives' => [
                                             {
                                               'type' => 'atom',
                                               'value' => [
                                                            'rule_reference',
                                                            'quoted_string'
                                                          ]
                                             },
                                             {
                                               'type' => 'atom',
                                               'value' => [
                                                            'rule_reference',
                                                            'number'
                                                          ]
                                             }
                                           ],
                         'type' => 'or'
                       },
          'scalar_ref' => {
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
                                              'value' => [
                                                           'rule_reference',
                                                           'number'
                                                         ],
                                              'type' => 'atom'
                                            }
                                          ],
                            'return_annotation' => [
                                                     'return_object',
                                                     '{type: "scalar_ref", index: $2}'
                                                   ]
                          },
          'property_value' => {
                                'type' => 'or',
                                'alternatives' => [
                                                    {
                                                      'value' => [
                                                                   'rule_reference',
                                                                   'quantified_array'
                                                                 ],
                                                      'type' => 'atom'
                                                    },
                                                    {
                                                      'type' => 'atom',
                                                      'value' => [
                                                                   'rule_reference',
                                                                   'simple_array'
                                                                 ]
                                                    },
                                                    {
                                                      'type' => 'atom',
                                                      'value' => [
                                                                   'rule_reference',
                                                                   'scalar_ref'
                                                                 ]
                                                    },
                                                    {
                                                      'value' => [
                                                                   'rule_reference',
                                                                   'literal'
                                                                 ],
                                                      'type' => 'atom'
                                                    }
                                                  ]
                              },
          'property' => {
                          'type' => 'sequence',
                          'elements' => [
                                          {
                                            'value' => [
                                                         'rule_reference',
                                                         'object_key'
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
                          'return_annotation' => [
                                                   'return_object',
                                                   '{key: $1, value: $5}'
                                                 ]
                        },
          'quantifier' => {
                            'type' => 'or',
                            'alternatives' => [
                                                {
                                                  'type' => 'sequence',
                                                  'elements' => [
                                                                  {
                                                                    'value' => [
                                                                                 'quoted_string',
                                                                                 '*'
                                                                               ],
                                                                    'type' => 'atom'
                                                                  }
                                                                ],
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
                                                  'elements' => [
                                                                  {
                                                                    'value' => [
                                                                                 'quoted_string',
                                                                                 '+'
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
                                                                                 'quoted_string',
                                                                                 '?'
                                                                               ],
                                                                    'type' => 'atom'
                                                                  }
                                                                ],
                                                  'return_annotation' => [
                                                                           'return_scalar',
                                                                           '"?"'
                                                                         ]
                                                },
                                                {
                                                  'type' => 'sequence',
                                                  'return_annotation' => [
                                                                           'return_object',
                                                                           '{min: $3, max: $3}'
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
                                                                    'type' => 'atom',
                                                                    'value' => [
                                                                                 'regex',
                                                                                 '\\s*'
                                                                               ]
                                                                  },
                                                                  {
                                                                    'value' => [
                                                                                 'rule_reference',
                                                                                 'number'
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
                                                                                 '}'
                                                                               ]
                                                                  }
                                                                ]
                                                },
                                                {
                                                  'type' => 'sequence',
                                                  'return_annotation' => [
                                                                           'return_object',
                                                                           '{min: $3, max: "inf"}'
                                                                         ],
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
                                                                                 'number'
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
                                                                    'type' => 'atom',
                                                                    'value' => [
                                                                                 'regex',
                                                                                 '\\s*'
                                                                               ]
                                                                  },
                                                                  {
                                                                    'value' => [
                                                                                 'rule_reference',
                                                                                 'number'
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
                                                                    'value' => [
                                                                                 'rule_reference',
                                                                                 'number'
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
                                                  'return_annotation' => [
                                                                           'return_object',
                                                                           '{min: $3, max: $7}'
                                                                         ],
                                                  'type' => 'sequence'
                                                }
                                              ]
                          },
          'return_annotation' => {
                                   'elements' => [
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
                                                     'value' => [
                                                                  'rule_reference',
                                                                  'return_expression'
                                                                ],
                                                     'type' => 'atom'
                                                   }
                                                 ],
                                   'type' => 'sequence'
                                 },
          'simple_object' => {
                               'type' => 'sequence',
                               'return_annotation' => [
                                                        'return_object',
                                                        '{type: "object", key: $3, value: $7}'
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
                                                 'type' => 'atom',
                                                 'value' => [
                                                              'regex',
                                                              '\\s*'
                                                            ]
                                               },
                                               {
                                                 'value' => [
                                                              'rule_reference',
                                                              'object_value'
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
          'return_expression' => {
                                   'type' => 'or',
                                   'alternatives' => [
                                                       {
                                                         'type' => 'atom',
                                                         'value' => [
                                                                      'rule_reference',
                                                                      'simple_nested_object'
                                                                    ]
                                                       },
                                                       {
                                                         'value' => [
                                                                      'rule_reference',
                                                                      'multi_property_object'
                                                                    ],
                                                         'type' => 'atom'
                                                       },
                                                       {
                                                         'value' => [
                                                                      'rule_reference',
                                                                      'quantified_array'
                                                                    ],
                                                         'type' => 'atom'
                                                       },
                                                       {
                                                         'value' => [
                                                                      'rule_reference',
                                                                      'simple_array'
                                                                    ],
                                                         'type' => 'atom'
                                                       },
                                                       {
                                                         'type' => 'atom',
                                                         'value' => [
                                                                      'rule_reference',
                                                                      'simple_object'
                                                                    ]
                                                       },
                                                       {
                                                         'type' => 'atom',
                                                         'value' => [
                                                                      'rule_reference',
                                                                      'scalar_ref'
                                                                    ]
                                                       }
                                                     ]
                                 }
        };
DEBUG: Entered generate_or_parser for object_value
DEBUG: Entered generate_or_parser for multi_property_object
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
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
            'value' => [
                         'rule_reference',
                         'inner_object'
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
        ];

DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'quoted_string',
                       '{'
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
          'value' => [
                       'rule_reference',
                       'outer_key'
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
          'type' => 'atom',
          'value' => [
                       'rule_reference',
                       'inner_object'
                     ]
        };

DEBUG generate_sequence_rule: processing element 8: $VAR1 = {
          'value' => [
                       'regex',
                       '\\s*'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 9: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'quoted_string',
                       '}'
                     ]
        };

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
            'value' => [
                         'rule_reference',
                         'scalar_ref'
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
        ];

DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'quoted_string',
                       '['
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
          'value' => [
                       'rule_reference',
                       'scalar_ref'
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
          'value' => [
                       'quoted_string',
                       ']'
                     ]
        };

DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'type' => 'atom',
            'value' => [
                         'regex',
                         '"([^"]*)"'
                       ]
          }
        ];

DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'regex',
                       '"([^"]*)"'
                     ]
        };

DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'value' => [
                         'rule_reference',
                         'scalar_ref'
                       ],
            'type' => 'atom'
          },
          {
            'value' => [
                         'rule_reference',
                         'quantifier'
                       ],
            'type' => 'atom'
          }
        ];

DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'value' => [
                       'rule_reference',
                       'scalar_ref'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 2: $VAR1 = {
          'value' => [
                       'rule_reference',
                       'quantifier'
                     ],
          'type' => 'atom'
        };

DEBUG: Entered generate_or_parser for object_key
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'value' => [
                         'regex',
                         '(\\d+)'
                       ],
            'type' => 'atom'
          }
        ];

DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'value' => [
                       'regex',
                       '(\\d+)'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
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

DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'quoted_string',
                       '{'
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
          'value' => [
                       'rule_reference',
                       'property'
                     ]
        };

DEBUG generate_sequence_rule: processing element 4: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'quoted_string',
                       ','
                     ]
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
          'value' => [
                       'rule_reference',
                       'property'
                     ]
        };

DEBUG generate_sequence_rule: processing element 7: $VAR1 = {
          'value' => [
                       'regex',
                       '\\s*'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 8: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'quoted_string',
                       '}'
                     ]
        };

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
            'type' => 'atom',
            'value' => [
                         'rule_reference',
                         'property'
                       ]
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
          'type' => 'atom',
          'value' => [
                       'rule_reference',
                       'property'
                     ]
        };

DEBUG generate_sequence_rule: processing element 4: $VAR1 = {
          'value' => [
                       'quoted_string',
                       ','
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 5: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'regex',
                       '\\s*'
                     ]
        };

DEBUG generate_sequence_rule: processing element 6: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'rule_reference',
                       'property'
                     ]
        };

DEBUG generate_sequence_rule: processing element 7: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'quoted_string',
                       ','
                     ]
        };

DEBUG generate_sequence_rule: processing element 8: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'regex',
                       '\\s*'
                     ]
        };

DEBUG generate_sequence_rule: processing element 9: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'rule_reference',
                       'property'
                     ]
        };

DEBUG generate_sequence_rule: processing element 10: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'regex',
                       '\\s*'
                     ]
        };

DEBUG generate_sequence_rule: processing element 11: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'quoted_string',
                       '}'
                     ]
        };

EBNF parser failed for annotation: -> {type: "multi_object", prop1: $3, prop2: $6, prop3: $9}, falling back to regex at ast_transform.pl line 1112.
DEBUG: Entered generate_or_parser for inner_value
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
            'value' => [
                         'rule_reference',
                         'inner_key'
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
            'value' => [
                         'rule_reference',
                         'inner_value'
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
        ];

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
          'value' => [
                       'rule_reference',
                       'inner_key'
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
          'value' => [
                       'rule_reference',
                       'inner_value'
                     ],
          'type' => 'atom'
        };

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

DEBUG: Entered generate_or_parser for inner_key
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
            'value' => [
                         'rule_reference',
                         'quantified_element'
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
                         ']'
                       ]
          }
        ];

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
          'value' => [
                       'rule_reference',
                       'quantified_element'
                     ]
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
          'value' => [
                       'quoted_string',
                       ']'
                     ]
        };

DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'value' => [
                         'regex',
                         '([a-zA-Z_]\\w*)'
                       ],
            'type' => 'atom'
          }
        ];

DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'value' => [
                       'regex',
                       '([a-zA-Z_]\\w*)'
                     ],
          'type' => 'atom'
        };

DEBUG: Entered generate_or_parser for outer_key
DEBUG: Entered generate_or_parser for literal
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'value' => [
                         'quoted_string',
                         '$'
                       ],
            'type' => 'atom'
          },
          {
            'value' => [
                         'rule_reference',
                         'number'
                       ],
            'type' => 'atom'
          }
        ];

DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'value' => [
                       'quoted_string',
                       '$'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 2: $VAR1 = {
          'value' => [
                       'rule_reference',
                       'number'
                     ],
          'type' => 'atom'
        };

DEBUG: Entered generate_or_parser for property_value
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'value' => [
                         'rule_reference',
                         'object_key'
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
        ];

DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'value' => [
                       'rule_reference',
                       'object_key'
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
          'value' => [
                       'rule_reference',
                       'property_value'
                     ]
        };

DEBUG: Entered generate_or_parser for quantifier
DEBUG: Found return annotation - disabling optimization
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
            'value' => [
                         'rule_reference',
                         'return_expression'
                       ],
            'type' => 'atom'
          }
        ];

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
          'value' => [
                       'rule_reference',
                       'return_expression'
                     ],
          'type' => 'atom'
        };

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
            'type' => 'atom',
            'value' => [
                         'regex',
                         '\\s*'
                       ]
          },
          {
            'value' => [
                         'rule_reference',
                         'object_value'
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
        ];

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
          'type' => 'atom',
          'value' => [
                       'rule_reference',
                       'object_key'
                     ]
        };

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
          'value' => [
                       'rule_reference',
                       'object_value'
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
          'value' => [
                       'quoted_string',
                       '}'
                     ],
          'type' => 'atom'
        };

DEBUG: Entered generate_or_parser for return_expression
package yapg::GeneratedParser;
use strict;
use warnings;

# Compiled regex patterns for speed
my %REGEXES = (
    'simple_nested_object_step1' => qr/\Q{\E/o,
    'simple_nested_object_step2' => qr/\s*/o,
    'simple_nested_object_step4' => qr/\s*/o,
    'simple_nested_object_step5' => qr/\Q:\E/o,
    'simple_nested_object_step6' => qr/\s*/o,
    'simple_nested_object_step8' => qr/\s*/o,
    'simple_nested_object_step9' => qr/\Q}\E/o,
    'simple_array_step1' => qr/\Q[\E/o,
    'simple_array_step2' => qr/\s*/o,
    'simple_array_step4' => qr/\s*/o,
    'simple_array_step5' => qr/\Q]\E/o,
    'quoted_string_step1' => qr/"([^"]*)"/o,
    'number_step1' => qr/(\d+)/o,
    'two_property_object_step1' => qr/\Q{\E/o,
    'two_property_object_step2' => qr/\s*/o,
    'two_property_object_step4' => qr/\Q,\E/o,
    'two_property_object_step5' => qr/\s*/o,
    'two_property_object_step7' => qr/\s*/o,
    'two_property_object_step8' => qr/\Q}\E/o,
    'three_property_object_step1' => qr/\Q{\E/o,
    'three_property_object_step2' => qr/\s*/o,
    'three_property_object_step4' => qr/\Q,\E/o,
    'three_property_object_step5' => qr/\s*/o,
    'three_property_object_step7' => qr/\Q,\E/o,
    'three_property_object_step8' => qr/\s*/o,
    'three_property_object_step10' => qr/\s*/o,
    'three_property_object_step11' => qr/\Q}\E/o,
    'inner_object_step1' => qr/\Q{\E/o,
    'inner_object_step2' => qr/\s*/o,
    'inner_object_step4' => qr/\s*/o,
    'inner_object_step5' => qr/\Q:\E/o,
    'inner_object_step6' => qr/\s*/o,
    'inner_object_step8' => qr/\s*/o,
    'inner_object_step9' => qr/\Q}\E/o,
    'quantified_array_step1' => qr/\Q[\E/o,
    'quantified_array_step2' => qr/\s*/o,
    'quantified_array_step4' => qr/\s*/o,
    'quantified_array_step5' => qr/\Q]\E/o,
    'identifier_step1' => qr/([a-zA-Z_]\w*)/o,
    'scalar_ref_step1' => qr/\$/o,
    'property_step2' => qr/\s*/o,
    'property_step3' => qr/\Q:\E/o,
    'property_step4' => qr/\s*/o,
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
    'return_annotation_step1' => qr/\Q->\E/o,
    'return_annotation_step2' => qr/\s*/o,
    'simple_object_step1' => qr/\Q{\E/o,
    'simple_object_step2' => qr/\s*/o,
    'simple_object_step4' => qr/\s*/o,
    'simple_object_step5' => qr/\Q:\E/o,
    'simple_object_step6' => qr/\s*/o,
    'simple_object_step8' => qr/\s*/o,
    'simple_object_step9' => qr/\Q}\E/o
);

# Runtime helper functions
sub quantified_match {
    my ($input, $regex, $min, $max) = @_;
    my $count = 0;
    my $pos = pos($$input);
    
    while ($count < $max && $$input =~ /\G$regex/gc) {
        $count++;
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
    my $pos = pos($$input);
    my @results = ();
    
    while ($count < $max) {
        my $result = $rule_ref->($input);
        if (defined $result) {
            push @results, $result;
            $count++;
        } else {
            last;
        }
    }
    
    if ($count >= $min) {
        return \@results;
    } else {
        # Restore position on failure
        pos($$input) = $pos;
        return undef;
    }
}

sub collect_quantified_results {
    # Helper function to collect results from quantified elements
    my ($element_num, $results_ref) = @_;
    my $element_index = $element_num - 1;
    
    # If the element is a quantified result (array), return it
    # If it's undef (zero matches), return empty array
    # Otherwise return single element in array
    my $element = $results_ref->[$element_index];
    
    if (!defined $element) {
        return [];  # Zero matches
    } elsif (ref($element) eq 'ARRAY') {
        return $element;  # Already an array from quantifier
    } else {
        return [$element];  # Single element, wrap in array
    }
}

# Fast parsing subroutines
sub parse_object_value {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_scalar_ref($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_literal($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
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
    
    return {"type" => "nested_object", "key" => ($results[2] // undef), "value" => ($results[6] // undef)};
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
    my $result_3 = parse_scalar_ref($input);
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
    
    return {"type" => "array", "element" => ($results[2] // undef)};
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
    
    return $results[0];
}


sub parse_quantified_element {
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
    my $result_2 = parse_quantifier($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    
    return {"scalar" => ($results[0] // undef), "quantifier" => ($results[1] // undef)};
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
    
    return $results[0];
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
    
    return {"type" => "multi_object", "prop1" => ($results[2] // undef), "prop2" => ($results[5] // undef)};
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
    
    return {"type" => "multi_object", "prop1" => ($results[3-1] // undef), "prop2" => ($results[6-1] // undef), "prop3" => ($results[9-1] // undef)};
}


sub parse_inner_value {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_quantified_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_simple_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_scalar_ref($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_literal($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
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
    unless ($$input =~ /\G$REGEXES{'inner_object_step2'}/gc) {
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
    
    return {"type" => "inner_object", "key" => ($results[2] // undef), "value" => ($results[6] // undef)};
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
    
    return {"type" => "quantified_array", "element" => ($results[2] // undef)};
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
    
    return $results[0];
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
    my $result_2 = parse_number($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    
    return {"type" => "scalar_ref", "index" => ($results[1] // undef)};
}


sub parse_property_value {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_quantified_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_simple_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_scalar_ref($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_literal($input))) { return $alt_result; }
    
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
    
    return {"key" => ($results[0] // undef), "value" => ($results[4] // undef)};
}


sub parse_quantifier {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt0_0'}/gc) && ("*") || (pos($$input) = $seq_pos, undef) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt1_0'}/gc) && ("+") || (pos($$input) = $seq_pos, undef) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt2_0'}/gc) && ("?") || (pos($$input) = $seq_pos, undef) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt3_0'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt3_1'}/gc) && (parse_number($input)) && ($$input =~ /\G$REGEXES{'quantifier_alt3_3'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt3_4'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt4_0'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt4_1'}/gc) && (parse_number($input)) && ($$input =~ /\G$REGEXES{'quantifier_alt4_3'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt4_4'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt4_5'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt4_6'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    if (my $alt_result = do { my $seq_pos = pos($$input); ($$input =~ /\G$REGEXES{'quantifier_alt5_0'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt5_1'}/gc) && (parse_number($input)) && ($$input =~ /\G$REGEXES{'quantifier_alt5_3'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt5_4'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt5_5'}/gc) && (parse_number($input)) && ($$input =~ /\G$REGEXES{'quantifier_alt5_7'}/gc) && ($$input =~ /\G$REGEXES{'quantifier_alt5_8'}/gc) && 1 || (pos($$input) = $seq_pos, 0) }) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
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
    
    return {"type" => "object", "key" => ($results[2] // undef), "value" => ($results[6] // undef)};
}


sub parse_return_expression {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (defined(my $alt_result = parse_simple_nested_object($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_multi_property_object($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_quantified_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_simple_array($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_simple_object($input))) { return $alt_result; }
    if (defined(my $alt_result = parse_scalar_ref($input))) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

# Main entry point
sub parse {
    my ($input_ref) = @_;
    pos($$input_ref) = 0;
    my $result = parse_return_annotation($input_ref);
    
    # Check that entire input was consumed
    if (defined $result && pos($$input_ref) == length($$input_ref)) {
        return $result;
    } else {
        return undef;  # Partial match or unconsumed input
    }
}

1;
