#! /usr/bin/env perl

use strict;
use warnings;
use Getopt::Long;
use Data::Dumper;

# Add the fx/perl directory to the path
use lib 'fx/perl';

use LinkedSpec;

# Configure Data::Dumper for better output
$Data::Dumper::Indent = 2;
$Data::Dumper::Sortkeys = 1;
$Data::Dumper::Terse = 0;

# Global flag for dumping (legacy support)
our $DUMP_MODE = 0;
# UVM-style verbosity level
our $DUMP_VERBOSITY = 0;
my $LOG_FILE = "run_parser_" . time() . ".log";

# Function to log output to both console and file
sub log_output {
    my ($message) = @_;
    print $message;
    
    # Also write to log file
    open(my $log_fh, '>>', $LOG_FILE) or die "Cannot open log file $LOG_FILE: $!";
    print $log_fh $message;
    close($log_fh);
}

# Function to extract test mode and expectations from spec file
sub extract_test_config {
    my ($spec_content) = @_;
    my $test_mode = 'full_pipeline';  # default
    my $expectation = 'pass';         # default
    
    # Look for TEST_MODE and EXPECT in the first 10 lines
    my @lines = split(/\n/, $$spec_content);
    my $line_count = 0;
    
    foreach my $line (@lines) {
        last if $line_count++ >= 10;  # Only check first 10 lines
        
        if ($line =~ /^\s*#\s*TEST_MODE:\s*(\w+)/i) {
            $test_mode = lc($1);
        }
        elsif ($line =~ /^\s*#\s*EXPECT:\s*(\w+)/i) {
            $expectation = lc($1);
        }
    }
    
    # Validate test mode
    my %valid_modes = (
        'parse_only' => 1,
        'generate_only' => 1,
        'full_pipeline' => 1
    );
    
    unless (exists $valid_modes{$test_mode}) {
        die "Invalid TEST_MODE: '$test_mode'. Valid modes: parse_only, generate_only, full_pipeline\n";
    }
    
    # Validate expectation
    my %valid_expectations = (
        'pass' => 1,
        'fail' => 1
    );
    
    unless (exists $valid_expectations{$expectation}) {
        die "Invalid EXPECT: '$expectation'. Valid expectations: pass, fail\n";
    }
    
    return ($test_mode, $expectation);
}

sub main {
    my ($spec_file, $input_file, $dump_mode, $verbosity);
    
    GetOptions(
        'dump' => \$dump_mode,
        'verbosity=s' => \$verbosity,
        'help' => sub { show_help(); exit 0; }
    ) or die "Error in command line arguments\n";
    
    # Check arguments
    if (@ARGV != 2) {
        show_help();
        exit 1;
    }
    
    ($spec_file, $input_file) = @ARGV;
    
    # Set verbosity level
    if (defined $verbosity) {
        $DUMP_VERBOSITY = parse_verbosity($verbosity);
        $DUMP_MODE = ($DUMP_VERBOSITY > 0) ? 1 : 0;  # Legacy support
    } else {
        $DUMP_MODE = $dump_mode;
        $DUMP_VERBOSITY = $dump_mode ? 300 : 0;  # Default to HIGH if --dump
    }
    
    $LinkedSpec::DUMP_VERBOSITY = $DUMP_VERBOSITY;  # Set the verbosity in LinkedSpec module
    
    # Make LOG_FILE accessible to LinkedSpec
    $main::LOG_FILE = $LOG_FILE;
    
    # Check if files exist
    unless (-f $spec_file) {
        die "Specification file '$spec_file' not found\n";
    }
    
    unless (-f $input_file) {
        die "Input file '$input_file' not found\n";
    }
    
    log_output("=== LinkedSpec Parser Runner ===\n");
    log_output("Log file: $LOG_FILE\n");
    log_output("Spec file: $spec_file\n");
    log_output("Input file: $input_file\n");
    log_output("Dump mode: " . ($DUMP_MODE ? "ON" : "OFF") . "\n");
    log_output("Verbosity level: $DUMP_VERBOSITY\n\n");
    
    # Read the spec file
    my $spec_content = do { local $/; open my $fh, '<', $spec_file or die "Cannot open $spec_file: $!"; <$fh> };
    
    # Extract test configuration from spec file
    my ($test_mode, $expectation) = extract_test_config(\$spec_content);
    
    # Auto-enable medium verbosity for parse-only tests to see $retv dump
    if ($test_mode eq 'parse_only' && $DUMP_VERBOSITY < 200) {
        $DUMP_VERBOSITY = 200;  # MEDIUM level
        $DUMP_MODE = 1;  # Legacy support
        log_output("=== Auto-enabling medium verbosity for parse-only test ===\n");
    }
    
    log_output("=== Test Configuration ===\n");
    log_output("Test mode: $test_mode\n");
    log_output("Expected outcome: $expectation\n\n");
    
    log_output("=== Specification File Content ===\n");
    log_output($spec_content);
    log_output("\n");
    
    if ($DUMP_VERBOSITY > 0) {
        log_output("=== Generating Parser with Verbosity Level $DUMP_VERBOSITY ===\n");
        if ($test_mode eq 'parse_only') {
            log_output("=== Parse-only mode: Will show SPEC COMPILE RESULT DUMP (\$retv) ===\n");
        }
    }
    
    # Generate the parser with execution mode from test configuration
    my $parser;
    my $parser_success = 1;
    
    eval {
        $parser = LinkedSpec::Get(\$spec_content, 
            pm_drive => 0,
            parse_only => ($test_mode eq 'parse_only'),
            generate_only => ($test_mode eq 'generate_only'),
            test_expectation => $expectation
        );
    } or do {
        $parser_success = 0;
        my $error = $@;
        log_output("Parser generation failed: $error");
    };
    
    # Check if we should continue with parsing
    if ($test_mode eq 'generate_only') {
        log_output("Execution stopped due to $test_mode mode\n");
        log_output("No functional parser returned - use full_pipeline mode to parse input files\n");
        exit(0);
    }
    
    # Check parser generation result against expectation
    if ($parser_success && defined $parser) {
        if ($expectation eq 'fail') {
            log_output("❌ TEST FAILED: Expected failure but parser generation succeeded\n");
        } else {
            log_output("✅ TEST PASSED: Expected success and parser generation succeeded\n");
        }
    } else {
        if ($expectation eq 'pass') {
            log_output("❌ TEST FAILED: Expected success but parser generation failed\n");
        } else {
            log_output("✅ TEST PASSED: Expected failure and parser generation failed\n");
        }
    }
    
    log_output("Parser generated successfully!\n\n");
    
    # Read the input file
    my $input_content = do { local $/; open my $fh, '<', $input_file or die "Cannot open $input_file: $!"; <$fh> };
    log_output("=== Input File Content ===\n");
    log_output($input_content);
    log_output("\n");
    
    # Parse the input
    log_output("=== Parsing Input ===\n");
    my $result = $parser->(\$input_content);
    
    if (defined $result) {
        log_output("Parse successful!\n");
        log_output("=== Parse Result ===\n");
        log_output(Dumper($result));
        
        # Check if result matches expectation
        if ($expectation eq 'fail') {
            log_output("❌ TEST FAILED: Expected failure but got success\n");
            exit(1);
        } else {
            log_output("✅ TEST PASSED: Expected success and got success\n");
        }
    } else {
        log_output("Parse failed - returned undef\n");
        
        # Check if result matches expectation
        if ($expectation eq 'pass') {
            log_output("❌ TEST FAILED: Expected success but got failure\n");
            exit(1);
        } else {
            log_output("✅ TEST PASSED: Expected failure and got failure\n");
        }
    }
}

sub parse_verbosity {
    my ($level) = @_;
    my %verbosity_map = (
        'none'   => 0,
        'low'    => 100,
        'medium' => 200,
        'high'   => 300,
        'full'   => 400,
        'debug'  => 500
    );
    
    if (exists $verbosity_map{lc($level)}) {
        return $verbosity_map{lc($level)};
    } elsif ($level =~ /^\d+$/) {
        return int($level);
    } else {
        die "Invalid verbosity level: $level\n";
    }
}



sub show_help {
    print <<EOF;
Usage: $0 [options] <spec_file> <input_file>

Options:
    --dump              Enable data structure dumping during parsing (legacy)
    --verbosity LEVEL   Set verbosity level (UVM-style)
    --help              Show this help message

Verbosity Levels (UVM-style):
    none    (0)   - No dumps
    low     (100) - Essential dumps only (errors, final results)
    medium  (200) - Standard dumps (parse results, generated spec)
    high    (300) - Detailed dumps (rule info, handlers) [default for --dump]
    full    (400) - Very detailed dumps (DSL transformations)
    debug   (500) - Maximum detail (everything)

Description:
    This script uses LinkedSpec to generate a parser from a .spec file
    and then uses that parser to parse an input file.

Examples:
    $0 my_spec.spec my_input.txt
    $0 --dump my_spec.spec my_input.txt
    $0 --verbosity medium my_spec.spec my_input.txt
    $0 --verbosity 300 my_spec.spec my_input.txt

Test Configuration:
    Add to the top of your .spec file:
    # TEST_MODE: full_pipeline|parse_only|generate_only
    # EXPECT: pass|fail

EOF
}

# Override LinkedSpec::Get to add dump functionality
{
    no warnings 'redefine';
    
    my $original_get = \&LinkedSpec::Get;
    
    *LinkedSpec::Get = sub {
        my ($spec_string, %options) = @_;
        
        if ($DUMP_VERBOSITY >= 200) {  # MEDIUM level
            log_output("=== LinkedSpec::Get() called ===\n");
            log_output("Spec string length: " . length($$spec_string) . "\n");
            log_output("Options: " . Dumper(\%options) . "\n");
        }
        
        # Call the original Get function
        my $result = $original_get->($spec_string, %options);
        
        if ($DUMP_VERBOSITY >= 200) {  # MEDIUM level
            log_output("=== Get() returned parser ===\n");
            log_output("Parser type: " . ref($result) . "\n");
        }
        
        return $result;
    };
}

# Add dump functionality to LinkedSpec if verbosity is enabled
if ($DUMP_VERBOSITY > 0) {
    log_output("=== Verbosity Level $DUMP_VERBOSITY Enabled ===\n");
    log_output("Dump points are now controlled by verbosity levels:\n");
    log_output("- LOW (100): Essential dumps (errors, final results)\n");
    log_output("- MEDIUM (200): Standard dumps (parse results, generated spec)\n");
    log_output("- HIGH (300): Detailed dumps (rule info, handlers)\n");
    log_output("- FULL (400): Very detailed dumps (DSL transformations)\n");
    log_output("- DEBUG (500): Maximum detail (everything)\n\n");
}

# Run the main function
main(); 