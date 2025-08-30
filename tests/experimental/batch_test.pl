#!/usr/bin/env perl
use strict;
use warnings;

print "=== Running Multiple Batches to Assess Success Rate ===\n\n";

my $total_files = 0;
my $successful_files = 0;

for my $batch (1..10) {
    print "Batch $batch: ";
    
    # Clean up
    system("rm -f generated_test_*.ebnf generated_parser_*.pl >/dev/null 2>&1");
    
    # Generate
    system("perl ebnf_generator.pl >/dev/null 2>&1");
    
    # Test each file
    my $batch_success = 0;
    my $batch_total = 0;
    
    for my $i (1..5) {
        my $ebnf_file = "generated_test_${i}.ebnf";
        next unless -f $ebnf_file;
        
        $batch_total++;
        $total_files++;
        
        # Generate parser
        my $result = system("perl ast_transform.pl $ebnf_file > generated_parser_${i}.pl 2>/dev/null");
        next if $result != 0;
        
        # Check syntax
        $result = system("perl -c generated_parser_${i}.pl >/dev/null 2>&1");
        if ($result == 0) {
            $batch_success++;
            $successful_files++;
        }
    }
    
    print "$batch_success/$batch_total successful\n";
}

my $success_rate = int(($successful_files / $total_files) * 100);
print "\n=== Overall Results ===\n";
print "Total files tested: $total_files\n";
print "Successful: $successful_files\n";
print "Success rate: $success_rate%\n";

if ($success_rate >= 80) {
    print "🎉 CONVERGENCE ACHIEVED! Success rate >= 80%\n";
} elsif ($success_rate >= 60) {
    print "🔄 GOOD PROGRESS! Success rate >= 60%\n";
} else {
    print "🔧 NEEDS WORK! Success rate < 60%\n";
}

