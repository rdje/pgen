#!/usr/bin/perl
use strict;
use warnings;
use lib '.';
use Data::Dumper;
use AST::Transform;

# Test the parentheses detection functions
my @test_tokens = (
    ['('],
    ['operator', '('],
    ['group_open', '('],
    [')'],
    ['operator', ')'],
    ['group_close', ')'],
);

print "=== TESTING PARENTHESES DETECTION FUNCTIONS ===\n";

foreach my $token (@test_tokens) {
    print "Token: [" . join(", ", @$token) . "]\n";
    print "  is_group_open: " . (AST::Transform::is_group_open($token) ? "YES" : "NO") . "\n";
    print "  is_group_close: " . (AST::Transform::is_group_close($token) ? "YES" : "NO") . "\n";
    print "\n";
}

print "=== CHECKING OUR ACTUAL TOKEN FORMAT ===\n";
my $actual_open = ['('];
my $actual_close = [')'];

print "Our actual open token ['(']: \n";
print "  is_group_open: " . (AST::Transform::is_group_open($actual_open) ? "YES" : "NO") . "\n";
print "  Token structure: " . Dumper($actual_open);

print "Our actual close token [')']: \n";
print "  is_group_close: " . (AST::Transform::is_group_close($actual_close) ? "YES" : "NO") . "\n";
print "  Token structure: " . Dumper($actual_close);

print "\n=== ANALYZING TOKEN STRUCTURE ===\n";
print "Actual open token ref: " . ref($actual_open) . "\n";
print "Actual open token length: " . scalar(@$actual_open) . "\n";
print "Actual open token [0]: '" . $actual_open->[0] . "'\n";
print "Actual open token [1]: '" . ($actual_open->[1] || 'undef') . "'\n";

# Test what we would need for proper detection
print "\n=== WHAT WE NEED FOR DETECTION ===\n";
# Current is_group_open code:
# return ref($token) eq 'ARRAY' && (
#    ($token->[0] eq 'operator' && $token->[1] eq '(') ||
#    ($token->[0] eq 'group_open' && $token->[1] eq '(')
# );

# But our tokens are just ['('] not ['operator', '(']
print "Current is_group_open expects: ['operator', '('] or ['group_open', '(']\n";
print "But we have: ['(']\n";
print "We need to modify is_group_open to handle: \$token->[0] eq '(' (single element array)\n";
