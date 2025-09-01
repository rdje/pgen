#===================================================================
# Copyright (c) 2005-2008 Richard DJE. All rights reserved.
#
# This Perl module is free software, you may redistribute it and/or 
# modify it under the same terms as Perl itself.
#===================================================================
package LinkedSpec;

use 5.010;
use re 'eval';
use Data::Dumper;

use PPlugin; 
use LinkedRE;

# UVM-style verbosity levels
use constant {
    DUMP_NONE   => 0,    # No dumps
    DUMP_LOW    => 100,  # Essential dumps only (errors, final results)
    DUMP_MEDIUM => 200,  # Standard dumps (parse results, generated spec)
    DUMP_HIGH   => 300,  # Detailed dumps (rule info, handlers)
    DUMP_FULL   => 400,  # Very detailed dumps (DSL transformations)
    DUMP_DEBUG  => 500   # Maximum detail (everything)
};

# Global verbosity level - can be set externally for debugging
our $DUMP_VERBOSITY = DUMP_NONE;

# Single logging function that handles everything
sub log_output {
    my ($level, $message, $context) = @_;
    
    # Check if we should log at this level
    return if $level > $DUMP_VERBOSITY;
    
    # Format timestamp
    my ($sec, $min, $hour, $mday, $mon, $year) = localtime();
    my $timestamp = sprintf("%04d-%02d-%02d %02d:%02d:%02d", 
                           $year + 1900, $mon + 1, $mday, $hour, $min, $sec);
    
    # Build log message
    my $log_msg = "[$timestamp] $message\n";
    $log_msg .= "  Context: $context\n" if defined $context;
    
    # Output to console and file
    print $log_msg;
    
    # Try to write to log file if it exists
    if (defined $main::LOG_FILE && -w $main::LOG_FILE) {
        open(my $log_fh, '>>', $main::LOG_FILE) or return;
        print $log_fh $log_msg;
        close($log_fh);
    }
}

# Simple function for Data::Dumper output (no timestamp needed)
sub log_dump {
    my ($message) = @_;
    print $message;
    
    # Try to write to log file if it exists
    if (defined $main::LOG_FILE && -w $main::LOG_FILE) {
        open(my $log_fh, '>>', $main::LOG_FILE) or return;
        print $log_fh $message;
        close($log_fh);
    }
}

# UVM-style verbosity check function
sub should_dump {
    my ($level) = @_;
    return $DUMP_VERBOSITY >= $level;
}

# DSL validation and error reporting functions
sub get_dsl_context {
    my ($spec_content, $position) = @_;
    
    # Find line number and context around the position
    my $before_pos = substr($$spec_content, 0, $position);
    my $line_number = 1 + ($before_pos =~ tr/\n//);
    
    # Get the line containing the position
    my @lines = split(/\n/, $$spec_content);
    my $current_line = $lines[$line_number - 1] || "";
    
    # Get surrounding context (previous and next lines)
    my $prev_line = $line_number > 1 ? $lines[$line_number - 2] : "";
    my $next_line = $line_number < @lines ? $lines[$line_number - 1] : "";
    
    return {
        line_number => $line_number,
        current_line => $current_line,
        prev_line => $prev_line,
        next_line => $next_line,
        position => $position
    };
}

sub report_dsl_error {
    my ($spec_content, $position, $error_msg, $suggestion) = @_;
    
    my $context = get_dsl_context($spec_content, $position);
    
    my $error = "DSL Error at line $context->{line_number}:\n";
    $error .= "  $error_msg\n";
    $error .= "  Line: $context->{current_line}\n";
    
    if ($context->{prev_line}) {
        $error .= "  Previous: $context->{prev_line}\n";
    }
    if ($context->{next_line}) {
        $error .= "  Next: $context->{next_line}\n";
    }
    
    if ($suggestion) {
        $error .= "  Suggestion: $suggestion\n";
    }
    
    log_output(DUMP_NONE, $error, "DSL validation failed");
}

# Input validation functions
sub validate_spec_content {
    my ($spec_content) = @_;
    
    # Check if spec content is a string reference
    unless (ref($spec_content) eq 'SCALAR') {
        log_output(DUMP_NONE, "Invalid spec content type", "Expected SCALAR reference, got " . ref($spec_content));
        return 0;
    }
    
    # Check if spec content is not empty
    unless (length($$spec_content) > 0) {
        log_output(DUMP_NONE, "Spec content is empty", "Spec file must contain content");
        return 0;
    }
    
    # Check for basic .spec file structure (skip comment lines)
    my @lines = split(/\n/, $$spec_content);
    my $found_rule = 0;
    
    foreach my $line (@lines) {
        # Skip empty lines and comment lines
        next if $line =~ /^\s*$/;
        next if $line =~ /^\s*#/;
        
        # Check if this line starts with a rule definition
        if ($line =~ /^\s*\w+::/) {
            $found_rule = 1;
            last;
        }
    }
    
    unless ($found_rule) {
        report_dsl_error($spec_content, 0, 
            "Spec file must start with a rule definition", 
            "Add a rule like 'RuleName::' at the beginning");
        return 0;
    }
    
    return 1;
}

sub validate_rule_definition {
    my ($rule_name, $rule_def) = @_;
    
    # Check if rule definition is a hash reference
    unless (ref($rule_def) eq 'HASH') {
        log_output(DUMP_NONE, "Invalid rule definition for '$rule_name'", "Expected HASH reference, got " . ref($rule_def));
        return 0;
    }
    
    # Check required fields exist
    unless (exists $rule_def->{handler}) {
        log_output(DUMP_NONE, "Rule '$rule_name' missing required 'handler' field", "All rules must define handler code");
        return 0;
    }
    
    # Top-level rules (entry points) may not have 're' field
    if (exists $rule_def->{re}) {
        # Validate regex array
        unless (ref($rule_def->{re}) eq 'ARRAY') {
            log_output(DUMP_NONE, "Rule '$rule_name' 're' field must be an array", "Got " . ref($rule_def->{re}));
            return 0;
        }
        
        # Check regex patterns are valid
        for my $i (0..$#{$rule_def->{re}}) {
            my $regex = $rule_def->{re}[$i];
            eval { qr/$regex/ } or do {
                log_output(DUMP_NONE, "Invalid regex in rule '$rule_name' at index $i", "Error: $@");
                return 0;
            };
        }
    }
    
    return 1;
}

sub validate_gdata_references {
    my ($gdata, $spec) = @_;
    
    # Check if gdata is a hash reference
    unless (ref($gdata) eq 'HASH') {
        log_output(DUMP_NONE, "Invalid gdata structure", "Expected HASH reference, got " . ref($gdata));
        return 0;
    }
    
    # Check if spec is a hash reference
    unless (ref($spec) eq 'HASH') {
        log_output(DUMP_NONE, "Invalid spec structure", "Expected HASH reference, got " . ref($spec));
        return 0;
    }
    
    # Validate each gdata entry (compiled regex objects)
    for my $rule_name (keys %$gdata) {
        my $gdata_entry = $gdata->{$rule_name};
        
        # Check if referenced rule exists in spec
        unless (exists $spec->{$rule_name}) {
            log_output(DUMP_NONE, "Gdata references non-existent rule '$rule_name'", "Rule not found in spec");
            return 0;
        }
        
        # Validate gdata entry is a compiled regex
        unless (ref($gdata_entry) eq 'Regexp') {
            log_output(DUMP_NONE, "Invalid gdata entry for rule '$rule_name'", "Expected compiled regex, got " . ref($gdata_entry));
            return 0;
        }
    }
    
    # Validate spec rule structures
    for my $rule_name (keys %$spec) {
        my $rule_def = $spec->{$rule_name};
        
        # Validate rule definition
        unless (validate_rule_definition($rule_name, $rule_def)) {
            return 0;
        }
        
        # Validate gdata references within each rule
        if (exists $rule_def->{gdata} && ref($rule_def->{gdata}) eq 'ARRAY') {
            for my $i (0..$#{$rule_def->{gdata}}) {
                my $element = $rule_def->{gdata}[$i];
                unless (ref($element) eq 'HASH' && exists $element->{label} && exists $element->{idx}) {
                    log_output(DUMP_NONE, "Invalid gdata element at index $i for rule '$rule_name'", "Expected HASH with 'label' and 'idx' keys");
                    return 0;
                }
                
                # Check if referenced rule exists
                my $ref_rule = $element->{label};
                unless (exists $spec->{$ref_rule}) {
                    log_output(DUMP_NONE, "Gdata element references non-existent rule '$ref_rule'", "Rule not found in spec");
                    return 0;
                }
                
                # Check if regex index is valid
                my $ref_idx = $element->{idx};
                my $ref_rule_def = $spec->{$ref_rule};
                unless (exists $ref_rule_def->{re} && $ref_idx < @{$ref_rule_def->{re}}) {
                    log_output(DUMP_NONE, "Invalid regex index $ref_idx for rule '$ref_rule'", "Index out of bounds");
                    return 0;
                }
            }
        }
    }
    
    return 1;
}

sub validate_dsl_syntax {
    my ($spec_content) = @_;
    
    my @lines = split(/\n/, $$spec_content);
    my @defined_rules = ();
    my @used_rules = ();
    
    # First pass: collect all defined rules and used rules
    for my $line (@lines) {
        # Skip empty lines and comments
        next if $line =~ /^\s*$/;
        next if $line =~ /^\s*#/;
        
        # Check for rule definitions
        if ($line =~ /^\s*(\w+)::/) {
            my $rule_name = $1;
            push @defined_rules, $rule_name;
            
            # Check for duplicate rule definitions
            if (grep { $_ eq $rule_name } @defined_rules[0..$#defined_rules-1]) {
                my $position = index($$spec_content, $line);
                report_dsl_error($spec_content, $position,
                    "Duplicate rule definition: '$rule_name'",
                    "Remove the duplicate rule or rename one of them");
                return 0;
            }
        }
        
        # Collect used rules (for warnings only, not errors)
        if ($line =~ /->\s*(\w+)(?:\[(\d+)\])?/) {
            my $rule_name = $1;
            push @used_rules, $rule_name;
        }
    }
    
    # Second pass: validate syntax
    for my $line (@lines) {
        # Skip empty lines and comments
        next if $line =~ /^\s*$/;
        next if $line =~ /^\s*#/;
        
        # Check for regex patterns
        if ($line =~ /^\s*\w+:\s*(\/.*?\/)/) {
            my $regex_pattern = $1;
            
            # Basic regex validation
            eval { qr/$regex_pattern/ } or do {
                my $position = index($$spec_content, $line);
                report_dsl_error($spec_content, $position,
                    "Invalid regex pattern: $regex_pattern",
                    "Check the regex syntax and ensure proper escaping");
                return 0;
            };
        }
    }
    
    # Check for unused rules (warning only)
    my @unused_rules = grep { !grep { $_ eq $_ } @used_rules } @defined_rules;
    if (@unused_rules) {
        log_output(DUMP_LOW, "Warning: Unused rules detected", "Rules defined but never used: " . join(", ", @unused_rules));
    }
    
    # Check for undefined rules (warning only, since order doesn't matter)
    my @undefined_rules = grep { !grep { $_ eq $_ } @defined_rules } @used_rules;
    if (@undefined_rules) {
        my @unique_undefined = do { my %seen; grep { !$seen{$_}++ } @undefined_rules };
        log_output(DUMP_LOW, "Warning: Undefined rules referenced", "Rules referenced but not defined: " . join(", ", @unique_undefined));
    }
    
    return 1;
}

my $cbrace_index  = 12;
my $node_type     = {
	'&'       => 'AND',
	'|'       => 'OR',
	'+'       => 'REP_PLUS',
	'*'       => 'REP_STAR',
	'?'       => 'REP_OPT'
};

my $rep_nodes_minmax = {
	REP_PLUS=> [1, 10**9],
	REP_STAR=> [0, 10**9],
	REP_OPT => [0, 1]
};

my $pm_drive;
my $spec_descr = [
{# Spec			-0-
 handler=> sub {
  my ($descr, $string, $gdata) = @_;
  my @specentry;
  my @specs;
  while (1) {
   my $minfo = LinkedRE::or($string, $$gdata{startREs});
   unless($minfo) {
	   # print "(Spec) Closing specentry DUE TO EOF\n" if @specentry;
    push @specs, [@specentry] if @specentry;
    return [@specs]
   }

   my $retv = &{$$descr[$$minfo{index}+1]{handler}}($minfo, $descr, $string, $gdata);
   return undef unless $retv;

   unless ($$retv[0] eq 'COMMENT') {
    if ($$retv[0] =~ /ELABEL/o) {
     if (@specentry) {
      # say '(Spec) Closing specentry DUE TO NEW Entry';
      push @specs, [@specentry];
      # print "(Spec) Re-Initializing specentry (@{$retv})\n";
      @specentry = $retv
     } else {
     # say "(Spec) Initializing specentry @{$retv})";
     push @specentry, $retv;
      # Hack
      #$gdata->{_current_entry} = $retv->[1]
     }
    } else {
     # say "(Spec) Pushing in specentry (@{$retv})";
     push @specentry, $retv;
    }
   }

  }
 }
},

{# Entry Label
 re=> [qr/\w+\s*::?(?:&|\||\+|\*|\?)?/o],
 handler=> sub {
  my ($info, undef, undef, $gdata) = @_;
  $$info{match} =~ s/\s*://o;

  #say "\n(Entry Label) ($$info{match})";
  my $target = $$info{match} =~ /:/ ? '_INITIAL' : '';
  $$info{match} =~ s/://o;
  
  $$info{match} =~ s/(\W)//o;
  $gdata->{_current_entry} = $$info{match};
  return ["ELABEL$target", $$info{match}, $1 ? $node_type->{$1} : "default"]
 }
},

{# RE pattern
 re=> [qr/(?<!\\)\/.+?(?<!\\)\//o],
 handler=> sub {
  my ($info) = @_;
  $$info{match} =~ s/^\/|\/$//g;

  #say "(RE pattern) ($$info{match})";
  return ['RE', $$info{match}]
 }
},

{# Action code block
 re=> [qr/->\s*\w+(?:\[\d+\])?\s*\{/o, qr/\}/o],
 handler=> sub {
  my ($info, $descr, $string, $gdata) = @_;

  my $ipos = pos($$string);
  my ($entry_label, $reidx) = $$info{match} =~ /(\w+)(?:\[(\d+)\])?/o; 
  $reidx = $reidx || 0;

  #say "(Action code block)($$info{match})($entry_label, $reidx)";
  while (1) {
   my $minfo = LinkedRE::or($string, $$gdata{cbrace});
   return undef unless $minfo;

   if ($$minfo{index} == 1) {
    # Closing brace, recursion stops here
    #say "(Action code block) (${\(substr($$string, $ipos, pos($$string) - $ipos - 1))}) Closing";
    return ['ACODE', {relabel=>$entry_label, reidx=>$reidx, code=>substr($$string, $ipos, pos($$string) - $ipos - 1)}]
   } elsif ($$minfo{index} == 0) {
    # Opening brace found, triggering recursion
    # say '(Curly BRACE) Recursion';
    &{$$descr[$cbrace_index]{handler}}($minfo, $descr, $string, $gdata);
    # say '(Curly BRACE) Back From Recursion';
   } else {
    #say "QUOTES <$$minfo{match}>"
   }
  }
 }
},

{# Method-like Empty Action code block
 re=> [qr/->\s*(?<ENTRY_LABEL>\w+)\s*(?:\[\s*(?<INDEX>\d+)\s*\]\s*)?\.\s*(?<METHOD>\w+)(?<ARGS>\s*\((?:[^\(\)]++|(?&ARGS))+\))?/o],
 handler=> sub {
  my ($info, $descr, $string, $gdata) = @_;

  my ($entry_label, $reidx, $method, $args) = @{$$info{match_hash}}{qw/ENTRY_LABEL INDEX METHOD ARGS/}; 
  # say "(Method code block) ($entry_label:".($reidx // 0).":$method:".($args // '').")";
  $args =~ s/^\(|\)$//go;
  return ['ACODE', {relabel=>$entry_label, reidx=> $reidx // 0, code=>"$method($entry_label".($args ? ",$args" : '').")"}]
 }
},

{# Empty Action code block
 re=> [qr/->\s*\w+(?:\[0\])?/o],
 handler=> sub {
  my ($info, $descr, $string, $gdata) = @_;

  my ($entry_label) = $$info{match} =~ /(\w+)/o; 
  #print "(Empty Action code block) ($entry_label)\n";
  return ['ACODE', {relabel=>$entry_label, reidx=>0, code=>"call($entry_label)"}]
 }
},


{# Non-Action code block
 re=> [qr/\w+\s*\{/o, qr/\}/o],
 handler=> sub {
  my ($info, $descr, $string, $gdata) = @_;

  my $ipos = pos($$string);
  my ($type) = $$info{match} =~ /(\w+)/o; 
  #print "(Non-Action code block) ($type) Opening\n";

  while (1) {
   my $minfo = LinkedRE::or($string, $$gdata{cbrace});
   return undef unless $minfo;

   if ($$minfo{index} == 1) {
    # Closing brace, recursion stops here
    # print "(Initial/Loop  ($type) code block) (${\(substr($$string, $ipos, pos($$string) - $ipos - 1))}) Closing\n";
    return ["${type}CODE", substr($$string, $ipos, pos($$string) - $ipos - 1)]
   } elsif ($$minfo{index} == 0) {
    # Opening brace found, triggering recursion
    # print "(Curly BRACE) Recursion\n";
    &{$$descr[$cbrace_index]{handler}}($minfo, $descr, $string, $gdata);
    # print "(Curly BRACE) Back From Recursion\n";
   } else {
    #print "QUOTES <$$minfo{match}>\n"
   }
  }
 }
},

{# Comment
 #re=> [qr/(?:\r\n?)?[ \t]*#.*/o],
 re=> [qr/[ \t]*#.*/o],
 handler=> sub {return ['COMMENT']}
},

{# Blind call code block
 re=> [qr/=>\s*\w+\s*\{/o, qr/\}/o],
 handler=> sub {
  my ($info, $descr, $string, $gdata) = @_;

  my $ipos = pos($$string);
  my ($call) = $$info{match} =~ /(\w+)/o; 

  #print "(Blind call code block)($$info{match})($call, $reidx)\n";
  while (1) {
   my $minfo = LinkedRE::or($string, $$gdata{cbrace});
   return undef unless $minfo;

   if ($$minfo{index} == 1) {
    # Closing brace, recursion stops here
    #print "(Action code block) (${\(substr($$string, $ipos, pos($$string) - $ipos - 1))}) Closing\n";
    return ['BCODE', {call=>$call, code=>"\$$gdata->{_current_entry} = call($call);\n".substr($$string, $ipos, pos($$string) - $ipos - 1)}]
   } elsif ($$minfo{index} == 0) {
    # Opening brace found, triggering recursion
    # print "(Curly BRACE) Recursion\n";
    &{$$descr[$cbrace_index]{handler}}($minfo, $descr, $string, $gdata);
    # print "(Curly BRACE) Back From Recursion\n";
   } else {
    #print "QUOTES <$$minfo{match}>\n"
   }
  }
 }
},

{# Split-Like Code
 re=> [qr/@\s*move_pos\b/o],
 handler=> sub {
  # say '(Split-Like Code)';
  return ['MOVE_POS']
 }
},


{# Empty Blind code block
 re=> [qr/=>\s*\w+/o],
 handler=> sub {
  my ($info, $descr, $string, $gdata) = @_;

  my ($call) = $$info{match} =~ /(\w+)/o; 
  #print "(Empty Action code block) ($entry_label)\n";
  return ['BCODE', {call=>$call, code=>"\$$gdata->{_current_entry} = call($call)"}]
 }
},


{# Method-like Empty Non-Action code block
 re=> [qr/(?<TYPE>\w+)\s*\.\s*(?<METHOD>\w+)(?<ARGS>\s*\((?:[^\(\)]++|(?&ARGS))+\))?/o],
 handler=> sub {
  my ($info, $descr, $string, $gdata) = @_;

  my ($type, $method, $args) = @{$$info{match_hash}}{qw/TYPE METHOD ARGS/};
  # say "(Method-like Empty Action code block) ($type)($method)(".($args// '').")";
  return ["${type}CODE", $method."($gdata->{_current_entry}".($args ? ",$args" :  '').')']
 }
},


{# Curly Brace			-7- + dquotes + squotes
 #re=> [qr/(?<!\\)\{/o, qr/(?<!\\)\}/o],
 re=> [qr/(?<!\\)\{/o, qr/(?<!\\)\}/o, qr/(?<!\\)".*?(?<!\\)"/o, qr/(?<!\\)'.*?(?<!\\)'/o],
 handler=> sub {
  my ($info, $descr, $string, $gdata) = @_;

  my $ipos = pos($$string);
  #print "(Curly BRACE) Opening\n";

  while (1) {
   my $minfo = LinkedRE::or($string, $$gdata{cbrace});
   return undef unless $minfo;

   if ($$minfo{index} == 1) {
    # Closing brace, recursion stops here
    #print "(Curly BRACE) Closing <".substr($$string, $ipos, pos($$string) - $ipos - 1).">\n";
    return 1
   } elsif ($$minfo{index} == 0)  {
    # Opening brace found, triggering recursion
    #print "(Curly BRACE) Recursion\n";
    &{$$descr[$cbrace_index]{handler}}($minfo, $descr, $string, $gdata);
    # print "(Curly BRACE) Back From Recursion\n";
   } else {
    #print "QUOTES <$$minfo{match}>\n"
   }
  }
 }
}
];


my $gdata = {
 startREs => LinkedRE::oredRE(map {$$_{re}[0]} grep {exists $$_{re}} @$spec_descr),
 cbrace   => LinkedRE::oredRE(@{$$spec_descr[$cbrace_index]{re}})
};


# my $testdata = "999  + (3 + (7 - 9 + (arr + 99 - ZZAA)))";
# $file = qx(cat ~/specfiletest.txt);
# Get(\$file)->(\$testdata);
my $top_rule;
sub Get {
 log_output(DUMP_LOW, "Starting parser generation", "Processing .spec file");
 
 my %option = @_[1 .. $#_];
 $pm_drive = $option{pm_drive};
 
 # Check for execution mode options
 my $parse_only = $option{parse_only};
 my $generate_only = $option{generate_only};
 my $test_expectation = $option{test_expectation};
 
 # Always run validation, but handle failures differently for parse-only tests
 my $validation_failed = 0;
 
      # Validate input spec content
     unless (validate_spec_content($_[0])) {
         if ($parse_only && $test_expectation eq 'fail') {
             $validation_failed = 1;
             log_output(DUMP_LOW, "Validation failed as expected", "Spec content validation failed - this is expected for this test");
         } else {
             log_output(DUMP_NONE, "CRITICAL ERROR", "Spec content validation failed - terminating parser generation");
             return undef;
         }
     }
 
      # Validate DSL syntax (only if content validation passed)
     unless ($validation_failed) {
         unless (validate_dsl_syntax($_[0])) {
             if ($parse_only && $test_expectation eq 'fail') {
                 $validation_failed = 1;
                 log_output(DUMP_LOW, "Validation failed as expected", "DSL syntax validation failed - this is expected for this test");
             } else {
                 log_output(DUMP_NONE, "CRITICAL ERROR", "DSL syntax validation failed - terminating parser generation");
                 return undef;
             }
         }
     }
 
 my $retv;
 my $parse_success = 1;
 
      # Try to parse the spec file
     log_output(DUMP_LOW, "Starting spec file parsing", "Attempting to parse .spec file content");
     eval {
         $retv = &{$$spec_descr[0]{handler}}($spec_descr, $_[0], $gdata);
     } or do {
         $parse_success = 0;
         my $error = $@;
         log_output(DUMP_NONE, "SPEC PARSING FAILED", "Hardcoded parser failed with error: $error");
     };
     
     if ($parse_success) {
         log_output(DUMP_LOW, "Spec file parsing successful", "Hardcoded parser completed successfully");
     }
 
 # Dump parse result if in dump mode (always for parse-only tests)
 if (should_dump(DUMP_MEDIUM) || $parse_only) {
     log_dump("=== SPEC COMPILE RESULT DUMP ===\n");
     if ($parse_success && defined $retv) {
         log_dump(Dumper($retv));
     } else {
         log_dump("Parse failed - no result available\n");
     }
     log_dump("=== END SPEC COMPILE RESULT DUMP ===\n");
 }
 
      # If parse-only mode, stop here and return undef
     if ($parse_only) {
         log_output(DUMP_LOW, "Parse-only mode", "Stopping after .spec file parsing - no parser generated");
         return undef;
     }
     
     # Start parser generation phase
     log_output(DUMP_LOW, "Starting parser generation", "Converting parsed spec data into executable parser");

 my $auto_descr_spec  = spec_descr($retv);
 my $final_descr      = {spec=>$auto_descr_spec, gdata=>spec_gdata($auto_descr_spec)};
 
      # Validate generated structures
     unless (validate_gdata_references($final_descr->{gdata}, $final_descr->{spec})) {
         log_output(DUMP_NONE, "CRITICAL ERROR", "Generated parser validation failed - terminating parser generation");
         return undef;
     }
 
 log_output(DUMP_LOW, "Parser generation completed", "Generated parser with " . scalar(keys %$auto_descr_spec) . " rules");

 print "\n\nsub Get {&{\$descr->{spec}{$top_rule}}(\$descr, \$_[0])}\n" if $pm_drive;

 # Dump final_descr if in dump mode
 if (should_dump(DUMP_LOW)) {
     log_dump("=== FINAL_DESCR DUMP: top_rule=$top_rule ===\n");
     log_dump(Dumper($final_descr));
     log_dump("=== END FINAL_DESCR DUMP: top_rule=$top_rule ===\n");
 }
 
      # If generate-only mode, stop here and return undef
     if ($generate_only) {
         log_output(DUMP_LOW, "Generate-only mode", "Stopping after parser generation - no functional parser returned");
         return undef;
     }
     
     # Parser generation completed successfully
     log_output(DUMP_LOW, "Parser generation completed successfully", "Returning functional parser for execution");

 return sub {&{$final_descr->{spec}{$top_rule}{handler}}($final_descr, $_[0])}
}

sub spec_descr {
my $specretv = shift;

 print 'my $descr = {
 spec => {'."\n" if $pm_drive;

 my @specinfo = map {spec_entry($_)} @$specretv;
 
 # Debug: Log the specinfo array
 log_output(DUMP_LOW, "Specinfo array contents", "Number of entries: " . scalar(@specinfo));
 for (my $i = 0; $i < @specinfo; $i++) {
     my $entry = $specinfo[$i];
     if (ref($entry) eq 'ARRAY' && @$entry >= 2) {
         log_output(DUMP_LOW, "Entry $i", "Label: '$entry->[0]', Type: " . ref($entry->[1]));
     } else {
         log_output(DUMP_LOW, "Entry $i", "Type: " . ref($entry) . ", Content: " . Dumper($entry));
     }
 }
 
 # Debug: Check for duplicate rules
 my %seen_rules;
 my @duplicate_rules;
 foreach my $pair (@specinfo) {
     if (ref($pair) eq 'ARRAY' && @$pair >= 2) {
         my ($label, $info) = @$pair;
         if (exists $seen_rules{$label}) {
             push @duplicate_rules, $label;
             log_output(DUMP_LOW, "Duplicate rule detected", "Rule '$label' is defined multiple times - second definition will overwrite the first");
         }
         $seen_rules{$label} = 1;
     }
 }
 
 if (@duplicate_rules) {
     log_output(DUMP_LOW, "Duplicate rules summary", "Rules with multiple definitions: " . join(", ", @duplicate_rules));
 }
 
 my $result = {@specinfo};
 
 # Dump generated spec if in dump mode
 if (should_dump(DUMP_MEDIUM)) {
     log_dump("=== GENERATED SPEC DUMP ===\n");
     log_dump(Dumper($result));
     log_dump("=== END GENERATED SPEC DUMP ===\n");
 }

 return $result
}

sub spec_entry {
my $einfo = shift;

 my %info;
 my $label;
 my $node_type;
 my @REs;
 my @icode  ;
 my @ecode  ;
 my @excode ;
 my @itcode ;
 my @lxcode ;
 my @lscode ;
 my @lecode ;
 my @ACODEs;
 my %BCODEs;
 my @BCALLs;
 my @GDATA;
 my %ab_count;
 my %handlers;

 if (should_dump(DUMP_HIGH)) {
     log_dump("=== SPEC ENTRY DUMP ===\n");
     log_dump(Dumper($einfo));
     log_dump("=== END SPEC ENTRY DUMP ===\n");
 }

 foreach my $centry (@$einfo) {
   ($label, $node_type)  = @$centry[1 .. 2] if $$centry[0] =~ /ELABEL/o;
   
   my $entry_type = $$centry[0];
   if ($entry_type =~ /ELABEL_INITIAL/o) {
     $top_rule = $$centry[1];
   }
   elsif ($entry_type eq 'ICODE') {
     push @icode, $$centry[1];
   }
   elsif ($entry_type eq 'ECODE') {
     push @ecode, $$centry[1];
   }
   elsif ($entry_type eq 'EXCODE') {
     push @excode, $$centry[1];
   }
   elsif ($entry_type eq 'ITCODE') {
     push @itcode, $$centry[1];
   }
   elsif ($entry_type eq 'LXCODE') {
     push @lxcode, $$centry[1];
   }
   elsif ($entry_type eq 'LSCODE') {
     push @lscode, $$centry[1];
   }
   elsif ($entry_type eq 'LECODE') {
     push @lecode, $$centry[1];
   }
   elsif ($entry_type eq 'RE') {
     push @REs, qr/$$centry[1]/;
   }
   elsif ($entry_type eq 'ACODE') {
     push @ACODEs, call_spec_handler_subst($label, $$centry[1]{code});
     push @GDATA, {label=>$$centry[1]{relabel}, idx=>$$centry[1]{reidx}};
     ++$ab_count{ACODE};
   }
   elsif ($entry_type eq 'BCODE') {
     push @BCALLs, $$centry[1]{call};
     $BCODEs{$$centry[1]{call}} = call_spec_handler_subst($label, $$centry[1]{code});
     ++$ab_count{BCODE};
   }
   elsif ($entry_type eq 'MOVE_POS') {
     # This is of course a temporary solution
     #push @lscode, 'push @'.$label.', [\'?'.$label.'_others:\', substr $$STRING, $IPOS, $LSPOS - $IPOS - length $LMATCH]';
     push @lecode, '$IPOS = pos $$STRING';
   }
 }


 if ($ab_count{ACODE} && $ab_count{BCODE}) {
  my $error_msg = "Rule '$label': Cannot mix ACTION (->) and BLIND CALL (=>) code blocks";
  my $context = "ACTION blocks: $ab_count{ACODE} found, BLIND CALL blocks: $ab_count{BCODE} found";
  log_output(DUMP_NONE, $error_msg, $context);
  print "  Solution: Use either ACTION blocks OR BLIND CALL blocks, not both\n";
  print "  Example: Use '-> rule_name { code }' OR '=> function_name { code }'\n";
  exit 1
 }

 my $icode  = join ";\n", map {s/\s*;\s*$//o; $_} map {call_spec_handler_subst($label, $_)} @icode ;
 my $ecode  = join ";\n", map {s/\s*;\s*$//o; $_} map {call_spec_handler_subst($label, $_)} @ecode ;
 my $excode = join ";\n", map {s/\s*;\s*$//o; $_} map {call_spec_handler_subst($label, $_)} @excode;
 my $itcode = join ";\n", map {s/\s*;\s*$//o; $_} map {call_spec_handler_subst($label, $_)} @itcode;
 my $lxcode = join ";\n", map {s/\s*;\s*$//o; $_} map {call_spec_handler_subst($label, $_)} @lxcode;
 my $lscode = join ";\n", map {s/\s*;\s*$//o; $_} map {call_spec_handler_subst($label, $_)} @lscode;
 my $lecode = join ";\n", map {s/\s*;\s*$//o; $_} map {call_spec_handler_subst($label, $_)} @lecode;

 # Initial value of the handler code
 my $actual_icode  = $icode  && "$icode;"  || "";
 my $actual_ecode  = $ecode  && "$ecode;"  || "";
 my $actual_excode = $excode && "$excode;" || "";
 my $actual_itcode = $itcode && "$itcode;" || "";

 my $handler = 
'my ($descr, $STRING, $info) = @_; 
my $IMATCH      = $$info{match}; 
my @IMATCH_LIST = @{$$info{match_list} // []};
my %IMATCH_HASH = %{$$info{match_hash} // {}};
my $IINDEX      = $$info{index}; 
my $IPOS        = pos $$STRING;

my @'.$label.';

'.$actual_icode;
 
 my $notvalid_lcodes = qr/^\s*$/o;
 
 if($ab_count{ACODE} || $ab_count{BCODE} || $lxcode !~ $notvalid_lcodes || $lscode !~ $notvalid_lcodes || $lecode !~ $notvalid_lcodes) {
  my $acodes = "";
  my $bcodes = "";
  if ($ab_count{ACODE}) {
   my $once  = 0;
   my $idx   = 0;
   $acodes  .= ($once++ ? " elsif " : "\n   if").'($$minfo{index} == '.$idx++.") {\n    $_\n   }" foreach (@ACODEs)
  }

  if ($ab_count{BCODE}) {
   my $once  = 0;
   $bcodes  .= ($once++ ? " elsif " : "\n   if")."(\$call eq \"$_\") {\n    $BCODEs{$_}\n   }" foreach (@BCALLs)
  }

  my $isAND = $node_type =~ /AND/o;
  my $isOR  = $node_type =~ /OR/o;
  my $isREP = $node_type =~ /REP_/o;

  my $actual_lxcode = $lxcode && "$lxcode;" || "";
  my $actual_lscode = $lscode && "$lscode;" || "";
  my $actual_lecode = $lecode && "$lecode;" || "";

 $handlers{_default} = ' 

 while (1) {
  my $minfo; eval q/$minfo = LinkedRE::or($STRING, $$descr{gdata}{'.$label.'})/;
  if($@) {
   print "\n(LinkedSpec) -E- Rule \''.$label.'\': Error during handler code generation\n";
   print "  Error: $@\n";
   print "  This usually indicates a syntax error in the generated Perl code\n";
   print "  Check your .spec file for malformed code blocks or invalid syntax\n";
   exit 1
  }

  unless($minfo) {
  '.($actual_lxcode || 'return undef').'
  }

  my $LMATCH      = $$minfo{match};
  my @LMATCH_LIST = @{$$minfo{match_list} // []};
  my %LMATCH_HASH = %{$$minfo{match_hash} // {}};
  my $LINDEX      = $$minfo{index};
  my $LSPOS       = pos $$STRING;
  
  '.$actual_lscode.'

  '.   $acodes     .'

  '.$actual_lecode.'

 }' if $acodes;

 $handlers{AND_BCODE} = '

  my $'.$label.';
  my @'.$label.'_collect;
  foreach my $call (qw('."@BCALLs".')) {
   my $current_call = $call;

   '.$bcodes.'

   unless ($'.$label.') {
    '.($actual_lxcode || 'return undef').'
   }
   
   '.($actual_lecode || 'push @'.$label.'_collect, $'.$label).' 
  }

  '.($actual_ecode || 'return \@'.$label.'_collect').'
 ' if $isAND && $bcodes;

 $handlers{AND_ACODE} = ' 

 my @'.$label.'_collect;
 my $idx = 0;
 
 while ($idx < '.scalar(@ACODEs).') {
  my $minfo = LinkedRE::or($STRING, $$descr{gdata}{'.$label.'});
  unless($minfo) {
   '.($actual_lxcode || 'return undef').'
  }
  
  # Only proceed if we match the expected index in sequence
  unless($$minfo{index} == $idx) {
   '.($actual_lxcode || 'return undef').'
  }

  my $LMATCH      = $$minfo{match};
  my @LMATCH_LIST = @{$$minfo{match_list} // []};
  my %LMATCH_HASH = %{$$minfo{match_hash} // {}};
  my $LINDEX      = $$minfo{index};
  my $LSPOS       = pos $$STRING;
  
  '.$actual_lscode.'

  '.   $acodes     .'

  '.$actual_lecode.'
  
  $idx++;
 }
 
 return \@'.$label.'_collect;
 ' if $isAND && $acodes;

 $handlers{OR_ACODE} = ' 

 my $minfo = LinkedRE::or($STRING, $$descr{gdata}{'.$label.'});
 unless($minfo) {
 '.($actual_lxcode || 'return undef').'
 }

 my $LMATCH      = $$minfo{match};
 my @LMATCH_LIST = @{$$minfo{match_list} // []};
 my %LMATCH_HASH = %{$$minfo{match_hash} // {}};
 my $LINDEX      = $$minfo{index};
 my $LSPOS       = pos $$STRING;
 
 '.   $acodes .'
 ' if $isOR && $acodes;

 $handlers{OR_BCODE} = '

  my $'.$label.';
  foreach my $call (qw('."@BCALLs".')) {
   my $current_call = $call;

   '.$bcodes.'

   if ($'.$label.') {
    '.($actual_lxcode || 'return $'.$label).'
   }
  }

  '.($actual_ecode || 'return undef').'
 ' if $isOR && $bcodes;

  $handlers{REP_BCODE} = do {
  my $and_code = '

  my $'.$label.';
  my @'.$label.'_collect;
  foreach my $call (qw('."@BCALLs".')) {
   my $current_call = $call;

   '.$bcodes.'

   unless ($'.$label.') {
    '.($actual_lxcode || 'return undef').'
   }
   
   '.($actual_lecode || 'push @'.$label.'_collect, $'.$label).' 
  }

  return \@'.$label.'_collect
 ';



   '
   my $min='.$rep_nodes_minmax->{$node_type}[0].';
   my $max='.$rep_nodes_minmax->{$node_type}[1].';
   my $'.$label.';
   my @'.$label.'_collect;

   my $ccount = 0;
   my $and_code = sub {eval \''.$and_code.'\'};

   while(1) {
    my $and_ret = $and_code->();
    unless ($and_ret) {
     if ($ccount >= $min) {
      '.($actual_excode || 'return \@'.$label.'_collect').'
     } else {
      return undef
     }
    }

    ++$ccount;

    '.($actual_itcode || 'push @'.$label.'_collect, $and_ret;').'

    last unless $ccount < $max
   }

   '.($actual_ecode || 'return \@'.$label.'_collect').'
   '  
   } if ($isREP && $bcodes);
 
 $handlers{REP_ACODE} = '
 
   my $min='.$rep_nodes_minmax->{$node_type}[0].';
   my $max='.$rep_nodes_minmax->{$node_type}[1].';
   my @'.$label.'_collect;
   my $ccount = 0;

   while(1) {
    my $minfo = LinkedRE::or($STRING, $$descr{gdata}{'.$label.'});
    unless($minfo) {
     if ($ccount >= $min) {
      '.($actual_excode || 'return \@'.$label.'_collect').'
     } else {
      return undef
     }
    }

    my $LMATCH      = $$minfo{match};
    my @LMATCH_LIST = @{$$minfo{match_list} // []};
    my %LMATCH_HASH = %{$$minfo{match_hash} // {}};
    my $LINDEX      = $$minfo{index};
    my $LSPOS       = pos $$STRING;
    
    '.$actual_lscode.'

    '.   $acodes     .'

    '.$actual_lecode.'
    
    ++$ccount;

    '.($actual_itcode || 'push @'.$label.'_collect, $'.$label.';').'

    last unless $ccount < $max
   }

   '.($actual_ecode || 'return \@'.$label.'_collect').'
 ' if ($isREP && $acodes);
 }

 if(@REs) {
  $info{re}      = [@REs];
 }

 # Updating the $handler variable
 my $right_key = (grep {defined} grep {$_ ne '_default'} keys %handlers)[0];
 $handler .= $handlers{$right_key || '_default'} || "";

 my $external_handler = $handler;
 $external_handler =~ s/&{\$\$descr{spec}{(\w+)}{handler}}/&{\$\$descr{spec}{$1}}/g;
 print "\n $label => sub {\n$external_handler\n },\n" if $pm_drive;

 $info{handler} = sub {eval $handler};
 #$info{acode}   = [@ACODEs];
 $info{gdata}   = [@GDATA];

 # Dump individual rule info if in dump mode
 if (should_dump(DUMP_HIGH)) {
     log_dump("\n=== RULE INFO DUMP for $label ===\n");
     log_dump(Dumper(\%info));
     log_dump("=== END RULE INFO DUMP for $label ===\n");
     log_dump("=== HANDLER DUMP for $label ===\n");
     log_dump("{\n$handler\n}\n");
     log_dump("=== END HANDLER DUMP for $label ===\n");
 }

 return ($label, \%info)
}

sub spec_gdata {
my $sg = shift;

 if (should_dump(DUMP_HIGH)) {
     log_dump("=== SPEC GDATA DUMP ===\n");
     log_dump(Dumper($sg));
     log_dump("=== END SPEC GDATA DUMP ===\n");
 }

 my $once=0;
 print ' gdata => {'."\n" if $pm_drive;

 my %gdata;
 foreach my $label (keys %$sg) {
  my @lgdata;
  foreach my $gde (@{$$sg{$label}{gdata}}) {
   if (exists $$sg{$$gde{label}}{re}[$$gde{idx}]) {
    push @lgdata, $$sg{$$gde{label}}{re}[$$gde{idx}]
   } else {
    my $error_msg = "Rule '$label': Referenced rule '$$gde{label}' has no regex at index $$gde{idx}";
    my $context = "Referenced rule: $$gde{label}, Requested index: $$gde{idx}, Available indices: " . 
                  (defined $$sg{$$gde{label}}{re} ? "0.." . ($#{$$sg{$$gde{label}}{re}}) : "none");
    log_output(DUMP_NONE, $error_msg, $context);
    print "  This usually means:\n";
    print "    1. Rule '$$gde{label}' doesn't exist in your .spec file\n";
    print "    2. Rule '$$gde{label}' has fewer regex patterns than expected\n";
    print "    3. There's a mismatch in regex indexing in your .spec file\n";
    if (should_dump(DUMP_HIGH)) {
        log_dump("=== GDATA ERROR CONTEXT ===\n");
        log_dump("label: $label\n");
        log_dump("gde: ".Dumper($gde)."\n");
        log_dump("sg: ".Dumper($sg)."\n");
        log_dump("lgdata: ".Dumper(\@lgdata)."\n");
        log_dump("=== END GDATA ERROR CONTEXT ===\n");
    }
    # exit 1
   }
  }

  if (@lgdata) {
   $gdata{$label} = LinkedRE::oredRE(@lgdata);

   print ''.($once ? ",\n" : "")." $label\t=> qr/$gdata{$label}/o" if $pm_drive;
   ++$once
  }

 }

 print "\n }\n};\n" if $pm_drive;
 
 my $result = \%gdata;
 
 # Dump generated gdata if in dump mode
 if (should_dump(DUMP_MEDIUM)) {
     log_dump("=== GENERATED GDATA DUMP ===\n");
     log_dump(Dumper($result));
     log_dump("=== END GENERATED GDATA DUMP ===\n");
 }
 
 return $result
}

sub call_spec_handler_subst {
my ($label, $code) = @_;

#say "call_spec_handler_subst: BEFORE <$label><$code>";
 $code =~ s/\bcall\((\w+)\)/&{\$\$descr{spec}{$1}{handler}}(\$descr, \$STRING, \$minfo)/g;
 $code =~ s/\bpush\((\w+)\)/push \@$label, &{\$\$descr{spec}{$1}{handler}}(\$descr, \$STRING, \$minfo)/g;
 $code =~ s/\bpush\((\w+)\s*,\s*(\w+)\)/push \@$2, &{\$\$descr{spec}{$1}{handler}}(\$descr, \$STRING, \$minfo)/g;
 $code =~ s/\breturn_a\($label(?:,(?<arg>\s*(?:[^\(\)]++|(?<par>\((?:[^\(\)]++|(?&par))+\)))+))?\)/return ['?$label:', @{[$+{arg} ? "($+{arg}), " : '']}\\\@$label]/g;
 $code =~ s/\breturn\($label,(?<arg>\s*(?:[^\(\)]++|(?<par>\((?:[^\(\)]++|(?&par))+\)))+)\)/return ['?$label:', $+{arg}]/g;
 $code =~ s/\breturn_ma\($label\)/return ['?$label:', \@IMATCH_LIST, \\\@$label]/g;
 $code =~ s/\breturn_m\($label\)/return ['?$label:', \@IMATCH_LIST]/g;
 $code =~ s/\$CAPTURE\b/substr(\$\$STRING, \$IPOS, \$LSPOS - \$IPOS - length \$LMATCH)/g;
 $code =~ s/\bcapture\(\w+\)/push \@$label, substr(\$\$STRING, \$IPOS, \$LSPOS - \$IPOS - length \$LMATCH)/g;
 $code =~ s{\bcapture_if\(\w+\)}{my \$capt = substr(\$\$STRING, \$IPOS, \$LSPOS - \$IPOS - length \$LMATCH); \$capt =~ s/^\\s*|\\s*\$//go; push \@$label, \$capt if \$capt}g;
 $code =~ s{\bCAPTURE_IF\(\)}{my \$capt = substr(\$\$STRING, \$IPOS, \$LSPOS - \$IPOS - length \$LMATCH); \$capt =~ s/^\\s*|\\s*\$//go; push \@$label, \$capt if \$capt}g;
 $code =~ s/\bIBACKTRACK\(\)/pos(\$\$STRING) = \$IPOS  - length \$IMATCH/g;
 $code =~ s/\bBACKTRACK\(\)/pos(\$\$STRING)  = \$LSPOS - length \$LMATCH/g;
 $code =~ s/\bibacktrack\(\w+\)/pos(\$\$STRING) = \$IPOS  - length \$IMATCH/g;
 $code =~ s/\bbacktrack\(\w+\)/pos(\$\$STRING)  = \$LSPOS - length \$LMATCH/g;

# say "call_spec_handler_subst: AFTER <$label><$code>";
 return $code
}


sub get_parser {use PathSearch; Get(\(my $o = do {open (my $f, PathSearch->go($_[0], 'spec')); local $/; <$f>}))}

sub AUTOLOAD   {PPlugin->exec($AUTOLOAD, @_)}

1;
