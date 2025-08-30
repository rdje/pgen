#!/usr/bin/env perl
use strict;
use warnings;
use Data::Dumper;

require './json_fixed_truthiness_parser.pm';

print "Debugging why parse_value fails on '{}'...\n\n";

my $simple_obj = '{}';
print "Testing: '$simple_obj'\n\n";

# Test parse_object directly
print "1. Testing parse_object directly:\n";
my $input_copy = $simple_obj;
my $obj_result = yapg::GeneratedParser::parse_object(\$input_copy);
print "   Result: " . Dumper($obj_result);
print "   Position: " . (pos($input_copy) // 0) . "\n\n";

# Test parse_value (which should call parse_object)
print "2. Testing parse_value:\n";
$input_copy = $simple_obj;
my $val_result = yapg::GeneratedParser::parse_value(\$input_copy);
print "   Result: " . Dumper($val_result);
print "   Position: " . (pos($input_copy) // 0) . "\n\n";

# Let's see what parse_object looks like
print "3. Checking parse_object implementation:\n";
print "   Looking at the pattern our object grammar should generate...\n";
print "   object := ws '{' ws '}' ws\n";
print "   This should be a sequence that matches: ws + '{' + ws + '}' + ws\n\n";

# Test components of parse_object manually
print "4. Testing parse_object components step by step:\n";
$input_copy = $simple_obj;
pos($input_copy) = 0;

print "   Step 1 - parse_ws:\n";
my $ws1 = yapg::GeneratedParser::parse_ws(\$input_copy);
print "     Result: " . Dumper($ws1) . "     Position: " . pos($input_copy) . "\n";

print "   Step 2 - match '{':\n";
my $brace_match = $input_copy =~ /\G\{/gc;
print "     Match: $brace_match, Position: " . pos($input_copy) . "\n";

print "   Step 3 - parse_ws:\n";
my $ws2 = yapg::GeneratedParser::parse_ws(\$input_copy);
print "     Result: " . Dumper($ws2) . "     Position: " . pos($input_copy) . "\n";

print "   Step 4 - match '}':\n";
my $close_brace = $input_copy =~ /\G\}/gc;
print "     Match: $close_brace, Position: " . pos($input_copy) . "\n";

print "   Step 5 - parse_ws:\n";
my $ws3 = yapg::GeneratedParser::parse_ws(\$input_copy);
print "     Result: " . Dumper($ws3) . "     Position: " . pos($input_copy) . "\n";

print "\nAll steps work manually: " . 
      (defined($ws1) && $brace_match && defined($ws2) && $close_brace && defined($ws3) ? "YES" : "NO") . "\n";
