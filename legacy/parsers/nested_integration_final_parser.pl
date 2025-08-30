=== Parsing nested_integration_test.ebnf ===
RAW AST from EBNF parser:
$VAR1 = [
          [
            [
              'rule',
              'items'
            ],
            [
              'rule_reference',
              'item'
            ],
            [
              'operator',
              '*'
            ],
            [
              'return_array',
              '[$1*]'
            ]
          ],
          [
            [
              'rule',
              'data_set'
            ],
            [
              'rule_reference',
              'header'
            ],
            [
              'rule_reference',
              'item'
            ],
            [
              'operator',
              '*'
            ],
            [
              'rule_reference',
              'footer'
            ],
            [
              'return_object',
              '{header: $1, items: [$2*], footer: $3}'
            ]
          ],
          [
            [
              'rule',
              'item'
            ],
            [
              'regex',
              '(\\w+)'
            ],
            [
              'return_scalar',
              '$1'
            ]
          ],
          [
            [
              'rule',
              'header'
            ],
            [
              'regex',
              '(BEGIN)'
            ],
            [
              'return_scalar',
              '$1'
            ]
          ],
          [
            [
              'rule',
              'footer'
            ],
            [
              'regex',
              '(END)'
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
            'items',
            'SEQUENCE',
            [
              [
                'rule_reference',
                'item'
              ],
              [
                'operator',
                '*'
              ],
              [
                'return_array',
                '[$1*]'
              ]
            ]
          ],
          [
            'data_set',
            'SEQUENCE',
            [
              [
                'rule_reference',
                'header'
              ],
              [
                'rule_reference',
                'item'
              ],
              [
                'operator',
                '*'
              ],
              [
                'rule_reference',
                'footer'
              ],
              [
                'return_object',
                '{header: $1, items: [$2*], footer: $3}'
              ]
            ]
          ],
          [
            'item',
            'SEQUENCE',
            [
              [
                'regex',
                '(\\w+)'
              ],
              [
                'return_scalar',
                '$1'
              ]
            ]
          ],
          [
            'header',
            'SEQUENCE',
            [
              [
                'regex',
                '(BEGIN)'
              ],
              [
                'return_scalar',
                '$1'
              ]
            ]
          ],
          [
            'footer',
            'SEQUENCE',
            [
              [
                'regex',
                '(END)'
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
            'items',
            'SEQUENCE',
            [
              [
                'rule_reference',
                'item'
              ],
              [
                'operator',
                '*'
              ],
              [
                'return_array',
                '[$1*]'
              ]
            ]
          ],
          [
            'data_set',
            'SEQUENCE',
            [
              [
                'rule_reference',
                'header'
              ],
              [
                'rule_reference',
                'item'
              ],
              [
                'operator',
                '*'
              ],
              [
                'rule_reference',
                'footer'
              ],
              [
                'return_object',
                '{header: $1, items: [$2*], footer: $3}'
              ]
            ]
          ],
          [
            'item',
            'SEQUENCE',
            [
              [
                'regex',
                '(\\w+)'
              ],
              [
                'return_scalar',
                '$1'
              ]
            ]
          ],
          [
            'header',
            'SEQUENCE',
            [
              [
                'regex',
                '(BEGIN)'
              ],
              [
                'return_scalar',
                '$1'
              ]
            ]
          ],
          [
            'footer',
            'SEQUENCE',
            [
              [
                'regex',
                '(END)'
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
            'items',
            'SEQUENCE',
            [
              [
                'rule_reference',
                'item'
              ],
              [
                'operator',
                '*'
              ],
              [
                'return_array',
                '[$1*]'
              ]
            ]
          ],
          [
            'data_set',
            'SEQUENCE',
            [
              [
                'rule_reference',
                'header'
              ],
              [
                'rule_reference',
                'item'
              ],
              [
                'operator',
                '*'
              ],
              [
                'rule_reference',
                'footer'
              ],
              [
                'return_object',
                '{header: $1, items: [$2*], footer: $3}'
              ]
            ]
          ],
          [
            'item',
            'SEQUENCE',
            [
              [
                'regex',
                '(\\w+)'
              ],
              [
                'return_scalar',
                '$1'
              ]
            ]
          ],
          [
            'header',
            'SEQUENCE',
            [
              [
                'regex',
                '(BEGIN)'
              ],
              [
                'return_scalar',
                '$1'
              ]
            ]
          ],
          [
            'footer',
            'SEQUENCE',
            [
              [
                'regex',
                '(END)'
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
            'items',
            'SEQUENCE',
            [
              [
                'QUANTIFIED',
                [
                  'rule_reference',
                  'item'
                ],
                '*'
              ],
              [
                'return_array',
                '[$1*]'
              ]
            ]
          ],
          [
            'data_set',
            'SEQUENCE',
            [
              [
                'rule_reference',
                'header'
              ],
              [
                'QUANTIFIED',
                [
                  'rule_reference',
                  'item'
                ],
                '*'
              ],
              [
                'rule_reference',
                'footer'
              ],
              [
                'return_object',
                '{header: $1, items: [$2*], footer: $3}'
              ]
            ]
          ],
          [
            'item',
            'SEQUENCE',
            [
              [
                'regex',
                '(\\w+)'
              ],
              [
                'return_scalar',
                '$1'
              ]
            ]
          ],
          [
            'header',
            'SEQUENCE',
            [
              [
                'regex',
                '(BEGIN)'
              ],
              [
                'return_scalar',
                '$1'
              ]
            ]
          ],
          [
            'footer',
            'SEQUENCE',
            [
              [
                'regex',
                '(END)'
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
          'header' => {
                        'return_annotation' => [
                                                 'return_scalar',
                                                 '$1'
                                               ],
                        'elements' => [
                                        {
                                          'type' => 'atom',
                                          'value' => [
                                                       'regex',
                                                       '(BEGIN)'
                                                     ]
                                        }
                                      ],
                        'type' => 'sequence'
                      },
          'items' => {
                       'return_annotation' => [
                                                'return_array',
                                                '[$1*]'
                                              ],
                       'elements' => [
                                       {
                                         'type' => 'quantified',
                                         'quantifier' => '*',
                                         'element' => [
                                                        'rule_reference',
                                                        'item'
                                                      ]
                                       }
                                     ],
                       'type' => 'sequence'
                     },
          'data_set' => {
                          'return_annotation' => [
                                                   'return_object',
                                                   '{header: $1, items: [$2*], footer: $3}'
                                                 ],
                          'elements' => [
                                          {
                                            'value' => [
                                                         'rule_reference',
                                                         'header'
                                                       ],
                                            'type' => 'atom'
                                          },
                                          {
                                            'element' => [
                                                           'rule_reference',
                                                           'item'
                                                         ],
                                            'quantifier' => '*',
                                            'type' => 'quantified'
                                          },
                                          {
                                            'type' => 'atom',
                                            'value' => [
                                                         'rule_reference',
                                                         'footer'
                                                       ]
                                          }
                                        ],
                          'type' => 'sequence'
                        },
          'item' => {
                      'return_annotation' => [
                                               'return_scalar',
                                               '$1'
                                             ],
                      'elements' => [
                                      {
                                        'value' => [
                                                     'regex',
                                                     '(\\w+)'
                                                   ],
                                        'type' => 'atom'
                                      }
                                    ],
                      'type' => 'sequence'
                    },
          'footer' => {
                        'type' => 'sequence',
                        'elements' => [
                                        {
                                          'type' => 'atom',
                                          'value' => [
                                                       'regex',
                                                       '(END)'
                                                     ]
                                        }
                                      ],
                        'return_annotation' => [
                                                 'return_scalar',
                                                 '$1'
                                               ]
                      }
        };
RULE ORDER: items, data_set, item, header, footer

=== Step 6: Generate parser code ===
🚀 DEPLOYING LEFT-RECURSION NUCLEAR ELIMINATOR!
🎯 Target: Complete annihilation of all recursion forms
======================================================================

🔄 Converting AST format to elimination format...
📊 Converted 5 rules
🏷️ Stored annotations for 5 rules
📋 Grammar before elimination:
   data_set := rule_reference:header QUANTIFIED:item:* rule_reference:footer
   footer := REGEX:(END)
   header := REGEX:(BEGIN)
   item := REGEX:(\w+)
   items := QUANTIFIED:item:*

print() on closed filehandle STDERR at ./integrate_left_recursion_killer.pl line 180.
print() on closed filehandle STDERR at ./integrate_left_recursion_killer.pl line 225.
print() on closed filehandle STDERR at ./integrate_left_recursion_killer.pl line 362.
DEBUG: Grammar after left-recursion elimination:
$VAR1 = {
          'header' => {
                        'type' => 'sequence',
                        'elements' => [
                                        {
                                          'type' => 'atom',
                                          'value' => [
                                                       'regex',
                                                       '(BEGIN)'
                                                     ]
                                        }
                                      ],
                        'return_annotation' => [
                                                 'return_scalar',
                                                 '$1'
                                               ]
                      },
          'items' => {
                       'return_annotation' => [
                                                'return_array',
                                                '[$1*]'
                                              ],
                       'type' => 'sequence',
                       'elements' => [
                                       {
                                         'element' => 'item',
                                         'type' => 'quantified',
                                         'quantifier' => '*'
                                       }
                                     ]
                     },
          'footer' => {
                        'elements' => [
                                        {
                                          'value' => [
                                                       'regex',
                                                       '(END)'
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
          'data_set' => {
                          'return_annotation' => [
                                                   'return_object',
                                                   '{header: $1, items: [$2*], footer: $3}'
                                                 ],
                          'type' => 'sequence',
                          'elements' => [
                                          {
                                            'type' => 'atom',
                                            'value' => [
                                                         'rule_reference',
                                                         'header'
                                                       ]
                                          },
                                          {
                                            'value' => [
                                                         'quantified_element',
                                                         'item',
                                                         '*'
                                                       ],
                                            'type' => 'atom'
                                          },
                                          {
                                            'value' => [
                                                         'rule_reference',
                                                         'footer'
                                                       ],
                                            'type' => 'atom'
                                          }
                                        ]
                        },
          'item' => {
                      'return_annotation' => [
                                               'return_scalar',
                                               '$1'
                                             ],
                      'type' => 'sequence',
                      'elements' => [
                                      {
                                        'value' => [
                                                     'regex',
                                                     '(\\w+)'
                                                   ],
                                        'type' => 'atom'
                                      }
                                    ]
                    }
        };
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'type' => 'atom',
            'value' => [
                         'regex',
                         '(BEGIN)'
                       ]
          }
        ];

DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'regex',
                       '(BEGIN)'
                     ]
        };

DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'element' => 'item',
            'type' => 'quantified',
            'quantifier' => '*'
          }
        ];

DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'element' => 'item',
          'type' => 'quantified',
          'quantifier' => '*'
        };

DEBUG: Found quantified element: $VAR1 = {
          'element' => 'item',
          'type' => 'quantified',
          'quantifier' => '*'
        };

DEBUG generate_quantified_code: element=$VAR1 = {
          'element' => 'item',
          'type' => 'quantified',
          'quantifier' => '*'
        };

DEBUG generate_quantified_code: quantifier=*, parsed quant=$VAR1 = {
          'max' => 999,
          'min' => 0
        };

DEBUG: Generated quantified code: quantified_rule($input, \&parse_item, 0, 999)
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'value' => [
                         'regex',
                         '(END)'
                       ],
            'type' => 'atom'
          }
        ];

DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'value' => [
                       'regex',
                       '(END)'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'type' => 'atom',
            'value' => [
                         'rule_reference',
                         'header'
                       ]
          },
          {
            'value' => [
                         'quantified_element',
                         'item',
                         '*'
                       ],
            'type' => 'atom'
          },
          {
            'value' => [
                         'rule_reference',
                         'footer'
                       ],
            'type' => 'atom'
          }
        ];

DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'type' => 'atom',
          'value' => [
                       'rule_reference',
                       'header'
                     ]
        };

DEBUG generate_sequence_rule: processing element 2: $VAR1 = {
          'value' => [
                       'quantified_element',
                       'item',
                       '*'
                     ],
          'type' => 'atom'
        };

DEBUG generate_sequence_rule: processing element 3: $VAR1 = {
          'value' => [
                       'rule_reference',
                       'footer'
                     ],
          'type' => 'atom'
        };

EBNF parser failed for annotation: -> {header: $1, items: [$2*], footer: $3}, falling back to regex at ast_transform.pl line 1112.
DEBUG generate_sequence_rule: processing sequence with filtered_elements=$VAR1 = [
          {
            'value' => [
                         'regex',
                         '(\\w+)'
                       ],
            'type' => 'atom'
          }
        ];

DEBUG generate_sequence_rule: processing element 1: $VAR1 = {
          'value' => [
                       'regex',
                       '(\\w+)'
                     ],
          'type' => 'atom'
        };

package yapg::GeneratedParser;
use strict;
use warnings;

# Compiled regex patterns for speed
my %REGEXES = (
    'header_step1' => qr/(BEGIN)/o,
    'footer_step1' => qr/(END)/o,
    'item_step1' => qr/(\w+)/o
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
sub parse_header {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'header_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return $results[0];
}


sub parse_items {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = quantified_rule($input, \&parse_item, 0, 999);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    
    return collect_quantified_results(1, \@results);
}


sub parse_footer {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'footer_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return $results[0];
}


sub parse_data_set {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    my $result_1 = parse_header($input);
    unless (defined $result_1) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_1;
    my $result_2 = parse_ARRAY(0x7fa0b329e348)($input);
    unless (defined $result_2) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_2;
    my $result_3 = parse_footer($input);
    unless (defined $result_3) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $result_3;
    
    return {"header" => ($results[1-1] // undef), "items" => collect_quantified_results(2, \@results), "footer" => ($results[3-1] // undef)};
}


sub parse_item {
    my ($input) = @_;
    my $start_pos = pos($$input);
    my @results = ();
    
    # Parse sequence elements in order
    unless ($$input =~ /\G$REGEXES{'item_step1'}/gc) {
        pos($$input) = $start_pos;
        return undef;
    }
    push @results, $1;  # Capture regex result
    
    return $results[0];
}


# Main entry point
sub parse {
    my ($input_ref) = @_;
    pos($$input_ref) = 0;
    my $result = parse_items($input_ref);
    
    # Check that entire input was consumed
    if (defined $result && pos($$input_ref) == length($$input_ref)) {
        return $result;
    } else {
        return undef;  # Partial match or unconsumed input
    }
}

1;
