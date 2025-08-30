#!/usr/bin/env perl
use strict;
use warnings;
use Data::Dumper;

require './json_ws_fixed_parser.pm';

print "Debugging what parse_ws returns...\n\n";

my @ws_tests = (
    '',          # Empty string (should match 0 chars)
    '   ',       # 3 spaces
    '\t\n',      # Tab and newline
    'hello',     # Non-whitespace (should match 0 chars)
);

foreach my $test (@ws_tests) {
    my $test_literal = $test;
    $test_literal =~ s/\\t/\t/g;
    $test_literal =~ s/\\n/\n/g;
    
    print "Testing: " . repr($test) . "\n";
    my $input_copy = $test_literal;
    my $start_pos = pos($input_copy) // 0;
    my $result = yapg::GeneratedParser::parse_ws(\$input_copy);
    my $end_pos = pos($input_copy) // 0;
    
    print "  Result: " . Dumper($result);
    print "  Position: $start_pos -> $end_pos (consumed " . ($end_pos - $start_pos) . " chars)\n";
    print "  Truthiness: " . ($result ? "TRUE" : "FALSE") . "\n";
    print "  Defined: " . (defined $result ? "YES" : "NO") . "\n\n";
}

sub repr {
    my $str = shift;
    $str =~ s/\t/\\t/g;
    $str =~ s/\n/\\n/g;
    return "'$str'";
}

