#!/usr/bin/perl
use strict;
use warnings;

print "=== BASIC STEP DEBUG ===\n";
print "Step 1: Perl basics work\n";
print "2 + 2 = " . (2 + 2) . "\n";

print "Step 2: Testing file access\n";
if (-f "fx/specs/ebnf.spec") {
    print "ebnf.spec file exists\n";
} else {
    print "ERROR: ebnf.spec file not found\n";
    exit 1;
}

print "Step 3: Testing lib path\n";
if (-d "fx/perl") {
    print "fx/perl directory exists\n";
} else {
    print "ERROR: fx/perl directory not found\n";
    exit 1;
}

print "Step 4: Testing LinkedSpec.pm file\n";
if (-f "fx/perl/LinkedSpec.pm") {
    print "LinkedSpec.pm file exists\n";
} else {
    print "ERROR: LinkedSpec.pm file not found\n";
    exit 1;
}

print "Step 5: Adding lib path\n";
use lib 'fx/perl';
print "lib path added successfully\n";

print "Step 6: About to require LinkedSpec\n";
eval {
    require LinkedSpec;
    print "LinkedSpec required successfully\n";
};
if ($@) {
    print "ERROR requiring LinkedSpec: $@\n";
    exit 1;
}

print "Step 7: Testing LinkedSpec::Get exists\n";
if (LinkedSpec->can('Get')) {
    print "LinkedSpec::Get method exists\n";
} else {
    print "ERROR: LinkedSpec::Get method not found\n";
    exit 1;
}

print "=== ALL BASIC STEPS PASSED ===\n";


