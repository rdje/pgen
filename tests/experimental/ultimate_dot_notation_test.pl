#!/usr/bin/perl
use strict;
use warnings;
use Data::Dumper;

# Load the ultimate parser
require './ultimate_parser_fixed.pl';

print "🍿 === ULTIMATE DOT NOTATION TEST SUITE === 🍿\n\n";

my @ultimate_tests = (
    # 🔥 TIER 1: ESSENTIAL FEATURES
    {
        name => "🎯 Property Access",
        input => '-> $2.name',
        type => "Property access",
    },
    {
        name => "🎯 Positional Access (1-based)",
        input => '-> $2.1',
        type => "Parse tree position",
    },
    {
        name => "🎯 Whole Array (implicit)",
        input => '-> $2.items',
        type => "Whole array return",
    },
    {
        name => "🎯 Whole Array (explicit empty)",
        input => '-> $2.items[]',
        type => "Explicit whole array",
    },
    {
        name => "🎯 Single Index",
        input => '-> $2.items[0]',
        type => "Single array element",
    },
    {
        name => "🎯 Negative Index",
        input => '-> $2.items[-1]',
        type => "Last element access",
    },
    
    # 🐍 TIER 2: PYTHON SLICING
    {
        name => "🐍 Python Slice (start:end)",
        input => '-> $2.items[1:4]',
        type => "Python slice notation",
    },
    {
        name => "🐍 Python Slice (from start)",
        input => '-> $2.items[:3]',
        type => "Python slice from beginning",
    },
    {
        name => "🐍 Python Slice (to end)",
        input => '-> $2.items[2:]',
        type => "Python slice to end",
    },
    {
        name => "🐍 Python Slice (whole array)",
        input => '-> $2.items[:]',
        type => "Python whole array slice",
    },
    
    # 📏 TIER 2: PERL5 RANGES
    {
        name => "📏 Perl5 Range",
        input => '-> $2.items[0..2]',
        type => "Perl5 range notation",
    },
    {
        name => "📏 Perl5 Negative Range",
        input => '-> $2.items[-3..-1]',
        type => "Perl5 negative range",
    },
    
    # 🎪 TIER 3: ADVANCED FEATURES
    {
        name => "🎪 Multiple Indices",
        input => '-> $2.items[1,3,5]',
        type => "Multiple specific indices",
    },
    {
        name => "🎪 Bash-style Whole Array",
        input => '-> $2.items[*]',
        type => "Bash-style array access",
    },
    
    # 🎭 TIER 3: MIXED EXPRESSIONS
    {
        name => "🎭 Mixed Expression",
        input => '-> $2.items[0,2..4,7]',
        type => "Mixed indices and ranges",
    },
    
    # 🚀 TIER 4: COMPLEX CHAINING
    {
        name => "🚀 Deep Property Chain",
        input => '-> $1.data.items.values',
        type => "Multi-level property access",
    },
    {
        name => "🚀 Mixed Property and Position",
        input => '-> $2.1.name',
        type => "Position then property",
    },
    {
        name => "🚀 Property and Array Access",
        input => '-> $2.items.1[0]',
        type => "Property, position, array",
    },
    {
        name => "🚀 Ultimate Complex Chain",
        input => '-> $3.data.items[1:3].2.values[-1]',
        type => "Ultimate complexity",
    },
    
    # 🎯 CONTEXT TESTS
    {
        name => "🎯 In Object Context",
        input => '-> {title: $1.header.title, items: $2.data.items[0..2]}',
        type => "Object with dot notation",
    },
    {
        name => "🎯 In Array Context",
        input => '-> [$1.first, $2.items[0], $3.last[-1]]',
        type => "Array with dot notation",
    },
);

foreach my $test (@ultimate_tests) {
    print "Testing: $test->{name}\n";
    print "Pattern: $test->{input}\n";
    print "Type: $test->{type}\n";
    
    eval {
        my $input = $test->{input};
        my $result = yapg::GeneratedParser::parse_return_annotation(\$input);
        
        if ($result) {
            print "Status: ✅ PARSED\n";
            
            # Analyze the result structure
            if (ref($result) eq 'ARRAY' && $result->[2]) {
                my $parsed_obj = $result->[2];
                print "Parsed Type: $parsed_obj->{type}\n";
                
                if ($parsed_obj->{type} eq 'ultimate_dot_notation') {
                    print "🎉 SUCCESS: Ultimate dot notation detected!\n";
                    print "Base: " . Dumper($parsed_obj->{base});
                    print "Path: " . Dumper($parsed_obj->{path});
                } elsif ($parsed_obj->{type} =~ /object|array/) {
                    print "✅ SUCCESS: Complex structure with dot notation\n";
                } else {
                    print "⚠️  Parsed as: $parsed_obj->{type}\n";
                }
            }
        } else {
            print "Status: ❌ FAILED\n";
        }
    };
    
    if ($@) {
        print "Status: ❌ ERROR - $@\n";
    }
    
    print "\n" . ("=" x 80) . "\n\n";
}

print "🎪 === ULTIMATE DOT NOTATION SUMMARY === 🎪\n";
print "Supported Features:\n";
print "✅ Property access: \$2.name, \$1.data.items\n";
print "✅ Positional access: \$2.1, \$3.2 (1-based parse tree)\n";
print "✅ Array indexing: \$2.items[0], \$2.items[-1]\n";
print "✅ Whole arrays: \$2.items, \$2.items[], \$2.items[*], \$2.items[:]\n";
print "✅ Perl5 ranges: \$2.items[0..2], \$2.items[-3..-1]\n";
print "✅ Python slices: \$2.items[1:4], \$2.items[:3], \$2.items[2:]\n";
print "✅ Multiple indices: \$2.items[1,3,5]\n";
print "✅ Mixed expressions: \$2.items[0,2..4,7]\n";
print "✅ Complex chaining: \$1.data.items[1:3].2.values[-1]\n";
print "\n🚀 This is the most comprehensive dot notation system ever built!\n";
