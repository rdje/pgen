#!/usr/bin/env perl
use strict;
use warnings;
use FindBin qw($RealBin);
use lib $RealBin;
use Test_advanced_perl_parser; # Placeholder, will be replaced by tools/ast_transform.pl
use Getopt::Long;
use Pod::Usage;

# Command-line options
my %options = (
    input_file => '',
    output_file => '',
    pretty => 0,
    help => 0,
);

GetOptions(
    'input|i=s'    => \$options{input_file},
    'output|o=s'   => \$options{output_file},
    'pretty|p'     => \$options{pretty},
    'help|h'       => \$options{help},
) or pod2usage(2);

pod2usage(1) if $options{help};

# Get input file from positional argument or --input option
my $input_file = $options{input_file} || $ARGV[0];
pod2usage("Error: No input file specified") unless $input_file;

# Read input file
open my $fh, '<', $input_file or die "Cannot open $input_file: $!";
my $content = do { local $/; <$fh> };
close $fh;

# Parse the content
my $result = Test_advanced_perl_parser::parse(\$content); # Placeholder, will be replaced by tools/ast_transform.pl

# Output result
if ($result) {
    if ($options{output_file}) {
        open my $out_fh, '>', $options{output_file} or die "Cannot write to $options{output_file}: $!";
        if ($options{pretty}) {
            require Data::Dumper;
            print $out_fh Data::Dumper->Dump([$result], ['result']);
        } else {
            print $out_fh "$result\n";
        }
        close $out_fh;
        print STDERR "✅ Parse result written to: $options{output_file}\n";
    } else {
        if ($options{pretty}) {
            require Data::Dumper;
            print Data::Dumper->Dump([$result], ['result']);
        } else {
            print "$result\n";
        }
    }
} else {
    print STDERR "❌ Parse failed\n";
    exit 1;
}

__END__

=head1 NAME

generated_parser.pl - Parse input using generated parser

=head1 SYNOPSIS

generated_parser.pl [options] input_file

Options:
  -i, --input     Input file to parse
  -o, --output    Output file for parse result
  -p, --pretty    Pretty-print output using Data::Dumper
  -h, --help      Show this help message

=head1 DESCRIPTION

This script uses the generated parser to parse input files and output the results.

=cut
