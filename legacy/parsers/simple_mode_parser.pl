=== Parsing test_simple_mode.ebnf ===
RAW AST from EBNF parser:
$VAR1 = [
          [
            'mode',
            [
              'regex',
              'in'
            ],
            [
              'return_scalar',
              '"input"'
            ]
          ],
          [
            'mode',
            [
              'regex',
              'out'
            ],
            [
              'return_scalar',
              '"output"'
            ]
          ]
        ];

=== Step 2: Group by OR operators ===
STEP 2 RESULT (Grouped by OR):
$VAR1 = [
          [
            'mode',
            'SEQUENCE',
            [
              [
                'regex',
                'in'
              ],
              [
                'return_scalar',
                '"input"'
              ]
            ]
          ],
          [
            'mode',
            'SEQUENCE',
            [
              [
                'regex',
                'out'
              ],
              [
                'return_scalar',
                '"output"'
              ]
            ]
          ]
        ];

=== Step 2.5: Handle parentheses grouping ===
STEP 2.5 RESULT (Parentheses handled):
$VAR1 = [
          [
            'mode',
            'SEQUENCE',
            [
              [
                'regex',
                'in'
              ],
              [
                'return_scalar',
                '"input"'
              ]
            ]
          ],
          [
            'mode',
            'SEQUENCE',
            [
              [
                'regex',
                'out'
              ],
              [
                'return_scalar',
                '"output"'
              ]
            ]
          ]
        ];

=== Step 3: Parse sequences ===
STEP 3 RESULT (Sequences parsed):
$VAR1 = [
          [
            'mode',
            'SEQUENCE',
            [
              [
                'regex',
                'in'
              ],
              [
                'return_scalar',
                '"input"'
              ]
            ]
          ],
          [
            'mode',
            'SEQUENCE',
            [
              [
                'regex',
                'out'
              ],
              [
                'return_scalar',
                '"output"'
              ]
            ]
          ]
        ];

=== Step 4: Handle quantifiers ===
STEP 4 RESULT (Quantifiers handled):
$VAR1 = [
          [
            'mode',
            'SEQUENCE',
            [
              [
                'regex',
                'in'
              ],
              [
                'return_scalar',
                '"input"'
              ]
            ]
          ],
          [
            'mode',
            'SEQUENCE',
            [
              [
                'regex',
                'out'
              ],
              [
                'return_scalar',
                '"output"'
              ]
            ]
          ]
        ];

=== Step 5: Build tree structure ===
STEP 5 RESULT (Tree structure):
$VAR1 = {
          'mode' => {
                      'alternatives' => [
                                          {
                                            'return_annotation' => [
                                                                     'return_scalar',
                                                                     '"input"'
                                                                   ],
                                            'elements' => [
                                                            {
                                                              'value' => [
                                                                           'regex',
                                                                           'in'
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
                                                                           'regex',
                                                                           'out'
                                                                         ]
                                                            }
                                                          ],
                                            'return_annotation' => [
                                                                     'return_scalar',
                                                                     '"output"'
                                                                   ],
                                            'type' => 'sequence'
                                          }
                                        ],
                      'type' => 'or'
                    }
        };
RULE ORDER: mode

=== Step 6: Generate parser code ===
🚀 DEPLOYING LEFT-RECURSION NUCLEAR ELIMINATOR!
🎯 Target: Complete annihilation of all recursion forms
======================================================================

🔄 Converting AST format to elimination format...
📊 Converted 1 rules
🏷️ Stored annotations for 1 rules
📋 Grammar before elimination:
   mode := REGEX:in | REGEX:out

print() on closed filehandle STDERR at ./integrate_left_recursion_killer.pl line 144.
print() on closed filehandle STDERR at ./integrate_left_recursion_killer.pl line 184.
print() on closed filehandle STDERR at ./integrate_left_recursion_killer.pl line 281.
package yapg::GeneratedParser;
use strict;
use warnings;

# Compiled regex patterns for speed
my %REGEXES = (
    'mode_alt0' => qr/in/o,
    'mode_alt1' => qr/out/o
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
sub parse_mode {
    my ($input) = @_;
    my $start_pos = pos($$input);
    
    # Try alternatives in order (fast backtracking)
    if (my $alt_result = $$input =~ /\G$REGEXES{'mode_alt0'}/gc) { return $alt_result; }
    if (my $alt_result = $$input =~ /\G$REGEXES{'mode_alt1'}/gc) { return $alt_result; }
    
    # No match - restore position
    pos($$input) = $start_pos;
    return undef;
}

# Main entry point
sub parse {
    my ($input_ref) = @_;
    pos($$input_ref) = 0;
    my $result = parse_mode($input_ref);
    
    # Check that entire input was consumed
    if (defined $result && pos($$input_ref) == length($$input_ref)) {
        return $result;
    } else {
        return undef;  # Partial match or unconsumed input
    }
}

1;
