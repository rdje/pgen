#!/usr/bin/env perl
use strict;
use warnings;
use lib 'tools/generators';
use lib 'perl';
use lib 'fx/perl';

print "Testing ultimate return annotation parser loading...\n";

# Test 1: Try to load the ultimate parser
print "1. Loading ultimate return annotation parser...\n";
eval {
    require 'ultimate_return_annotation_perl_parser.pm';
    print "   ✅ Ultimate parser loaded successfully!\n";
};
if ($@) {
    print "   ❌ Failed to load ultimate parser: $@\n";
    exit 1;
}

# Test 2: Try to parse a simple annotation
print "2. Testing simple return annotation parsing...\n";
my $test_annotation = "-> \$1";
my $result;

eval {
    $result = Merged_ultimate_return_annotation::parse(\$test_annotation);
    if (defined $result) {
        print "   ✅ Parse successful!\n";
        print "   Result: ";
        use Data::Dumper;
        print Dumper($result);
    } else {
        print "   ⚠️  Parse returned undef\n";
    }
};
if ($@) {
    print "   ❌ Parse failed: $@\n";
}

# Test 3: Try a more complex annotation
print "3. Testing complex return annotation parsing...\n";
my $complex_annotation = "-> {key: \$1, value: \$2}";

eval {
    $result = Merged_ultimate_return_annotation::parse(\$complex_annotation);
    if (defined $result) {
        print "   ✅ Complex parse successful!\n";
        print "   Result: ";
        print Dumper($result);
        
        # Check if result contains JavaScript constructs
        my $result_str = Dumper($result);
        if ($result_str =~ /\.map\(|Math\.|\.\.\.|\\.length/) {
            print "   ❌ CRITICAL: JavaScript constructs detected in parser output!\n";
        } else {
            print "   ✅ No JavaScript constructs detected - good!\n";
        }
    } else {
        print "   ⚠️  Complex parse returned undef\n";
    }
};
if ($@) {
    print "   ❌ Complex parse failed: $@\n";
}

print "Ultimate parser test complete.\n";
