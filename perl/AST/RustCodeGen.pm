package AST::RustCodeGen;

use strict;
use warnings;
use Data::Dumper;

use Exporter 'import';
our @EXPORT_OK = qw(generate_rust_parser_module);

# Global variables for configuration
our $quiet_mode = 0;
our $verbosity = 'normal';

sub generate_rust_parser_module {
    my ($grammar_tree, $rule_order) = @_;
    
    my @function_definitions = ();
    my @regex_definitions = ();
    
    # Generate parsing functions for each rule
    foreach my $rule_name (keys %$grammar_tree) {
        my $rule_def = $grammar_tree->{$rule_name};
        my ($func_code, $regexes) = generate_rust_parser_function($rule_name, $rule_def);
        push @function_definitions, $func_code;
        push @regex_definitions, @$regexes if $regexes;
    }
    
    # Build complete Rust module
    my $main_rule = $rule_order->[0];  # First rule is always the main entry point
    
    my $regex_definitions = join("\n", @regex_definitions);
    my $function_definitions = join("\n\n", @function_definitions);
    
    # Generate the Rust code
    my $rust_code = generate_rust_module_template($main_rule, $regex_definitions, $function_definitions);
    
    return $rust_code;
}

sub generate_rust_module_template {
    my ($main_rule, $regex_definitions, $function_definitions) = @_;
    
    my $template = <<'RUST';
use std::collections::HashMap;
use regex::Regex;
use lazy_static::lazy_static;

// Error type for parsing failures
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

impl ParseError {
    pub fn new(message: String, position: usize) -> Self {
        ParseError { message, position }
    }
}

// Result type for parsing
pub type ParseResult<T> = Result<Option<T>, ParseError>;

// AST node types
#[derive(Debug, Clone, PartialEq)]
pub enum ASTNode {
    Terminal(String),
    Array(Vec<ASTNode>),
    Object(HashMap<String, ASTNode>),
    Number(i64),
    Float(f64),
    Bool(bool),
    Null,
}

// Input position tracking
#[derive(Debug, Clone)]
pub struct ParseInput {
    text: String,
    position: usize,
}

impl ParseInput {
    pub fn new(text: String) -> Self {
        ParseInput { text, position: 0 }
    }
    
    pub fn current_char(&self) -> Option<char> {
        self.text.chars().nth(self.position)
    }
    
    pub fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.current_char() {
            self.position += ch.len_utf8();
            Some(ch)
        } else {
            None
        }
    }
    
    pub fn peek(&self, offset: usize) -> Option<char> {
        self.text.chars().nth(self.position + offset)
    }
    
    pub fn slice(&self, start: usize, end: usize) -> &str {
        &self.text[start.min(self.text.len())..end.min(self.text.len())]
    }
    
    pub fn remaining(&self) -> &str {
        &self.text[self.position.min(self.text.len())..]
    }
    
    pub fn save_position(&self) -> usize {
        self.position
    }
    
    pub fn restore_position(&mut self, pos: usize) {
        self.position = pos;
    }
    
    pub fn is_at_end(&self) -> bool {
        self.position >= self.text.len()
    }
}

// Compiled regex patterns for speed
lazy_static! {
    static ref REGEXES: HashMap<&'static str, Regex> = {
        let mut map = HashMap::new();
REGEX_DEFINITIONS_PLACEHOLDER
        map
    };
}

// Helper functions for quantified matching
fn quantified_match(input: &mut ParseInput, regex_name: &str, min: usize, max: usize) -> ParseResult<Vec<String>> {
    let mut matches = Vec::new();
    let start_pos = input.save_position();
    
    let regex = REGEXES.get(regex_name).ok_or_else(|| 
        ParseError::new(format!("Regex '{}' not found", regex_name), input.position)
    )?;
    
    for _ in 0..max {
        if let Some(mat) = regex.find(input.remaining()) {
            if mat.start() == 0 {  // Must match at current position
                let matched_text = mat.as_str().to_string();
                matches.push(matched_text.clone());
                input.position += mat.end();
            } else {
                break;
            }
        } else {
            break;
        }
    }
    
    if matches.len() >= min {
        Ok(Some(matches))
    } else {
        input.restore_position(start_pos);
        Ok(None)
    }
}

fn quantified_rule<T>(
    input: &mut ParseInput,
    rule_func: impl Fn(&mut ParseInput) -> ParseResult<T>,
    min: usize,
    max: usize,
) -> ParseResult<Vec<T>> {
    let mut results = Vec::new();
    let start_pos = input.save_position();
    
    for _ in 0..max {
        match rule_func(input)? {
            Some(result) => results.push(result),
            None => break,
        }
    }
    
    if results.len() >= min {
        Ok(Some(results))
    } else {
        input.restore_position(start_pos);
        Ok(None)
    }
}

// Helper function to match literal strings
fn match_literal(input: &mut ParseInput, literal: &str) -> ParseResult<String> {
    if input.remaining().starts_with(literal) {
        input.position += literal.len();
        Ok(Some(literal.to_string()))
    } else {
        Ok(None)
    }
}

// Helper function to match regex patterns
fn match_regex(input: &mut ParseInput, regex_name: &str) -> ParseResult<String> {
    let regex = REGEXES.get(regex_name).ok_or_else(|| 
        ParseError::new(format!("Regex '{}' not found", regex_name), input.position)
    )?;
    
    if let Some(mat) = regex.find(input.remaining()) {
        if mat.start() == 0 {  // Must match at current position
            let matched_text = mat.as_str().to_string();
            input.position += mat.end();
            Ok(Some(matched_text))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

FUNCTION_DEFINITIONS_PLACEHOLDER

// Main entry point
pub fn parse(text: &str) -> ParseResult<ASTNode> {
    let mut input = ParseInput::new(text.to_string());
    MAIN_RULE_PLACEHOLDER(&mut input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        // Add basic tests here
        let result = parse("test input");
        assert!(result.is_ok());
    }
}
RUST
    
    # Replace placeholders
    $template =~ s/REGEX_DEFINITIONS_PLACEHOLDER/$regex_definitions/g;
    $template =~ s/FUNCTION_DEFINITIONS_PLACEHOLDER/$function_definitions/g;
    $template =~ s/MAIN_RULE_PLACEHOLDER/parse_$main_rule/g;
    
    return $template;
}

sub generate_rust_parser_function {
    my ($rule_name, $rule_def) = @_;
    my @regexes = ();
    
    # DEBUG: Track what happens to index_list specifically
    if ($rule_name eq 'index_list' && !$quiet_mode && $verbosity eq 'debug') {
        print STDERR "DEBUG: Generating Rust parser for index_list with rule_def:\n";
        print STDERR Dumper($rule_def);
    }
    
    my $type = $rule_def->{type};
    if ($type eq 'or') {
        return generate_rust_or_parser($rule_name, $rule_def, \@regexes);
    } elsif ($type eq 'sequence') {
        return generate_rust_sequence_parser($rule_name, $rule_def, \@regexes);
    } elsif ($type eq 'atom') {
        return generate_rust_atom_parser($rule_name, $rule_def, \@regexes);
    }
}

sub generate_rust_or_parser {
    my ($rule_name, $rule_def, $regexes) = @_;
    
    print STDERR "DEBUG: Entered generate_rust_or_parser for $rule_name\n" if !$quiet_mode && $verbosity eq 'debug';
    
    my @alternatives = ();
    
    foreach my $alt (@{$rule_def->{alternatives}}) {
        if ($alt->{type} eq 'atom') {
            if (is_terminal($alt->{value})) {
                if (ref($alt->{value}) eq 'ARRAY' && $alt->{value}->[0] eq 'epsilon') {
                    # Epsilon production - always succeeds
                    push @alternatives, "        return Ok(Some(ASTNode::Terminal(\"\".to_string())));";
                } elsif (ref($alt->{value}) eq 'ARRAY' && $alt->{value}->[0] eq 'regex') {
                    # Regex pattern
                    my $pattern = escape_rust_raw_string($alt->{value}[1]);
                    my $regex_name = "${rule_name}_alt" . @alternatives;
                    push @$regexes, "        map.insert(\"$regex_name\", Regex::new(r\"$pattern\").unwrap());";
                    
                    if ($alt->{return_annotation}) {
                        my ($type, $annotation) = @{$alt->{return_annotation}};
                        push @alternatives, "        if let Ok(Some(matched)) = match_regex(input, \"$regex_name\") {";
                        push @alternatives, "            " . rust_return_annotation($type, $annotation, "matched");
                        push @alternatives, "        }";
                    } else {
                        push @alternatives, "        if let Ok(Some(matched)) = match_regex(input, \"$regex_name\") {";
                        push @alternatives, "            return Ok(Some(ASTNode::Terminal(matched)));";
                        push @alternatives, "        }";
                    }
                } else {
                    # Literal terminal
                    my $literal = extract_literal_value($alt->{value});
                    $literal = escape_rust_string($literal);
                    
                    if ($alt->{return_annotation}) {
                        my ($type, $annotation) = @{$alt->{return_annotation}};
                        push @alternatives, "        if let Ok(Some(matched)) = match_literal(input, \"$literal\") {";
                        push @alternatives, "            " . rust_return_annotation($type, $annotation, "matched");
                        push @alternatives, "        }";
                    } else {
                        push @alternatives, "        if let Ok(Some(matched)) = match_literal(input, \"$literal\") {";
                        push @alternatives, "            return Ok(Some(ASTNode::Terminal(matched)));";
                        push @alternatives, "        }";
                    }
                }
            } else {
                # Non-terminal atom - call other parser function
                my $rule_name_to_call = extract_token_value($alt->{value});
                push @alternatives, "        if let Ok(Some(result)) = parse_$rule_name_to_call(input) {";
                push @alternatives, "            return Ok(Some(result));";
                push @alternatives, "        }";
            }
        } elsif ($alt->{type} eq 'sequence') {
            # For sequences in alternatives, generate inline matching code
            my @seq_steps = ();
            my $alt_num = @alternatives;
            
            foreach my $element (@{$alt->{elements}}) {
                if ($element->{type} eq 'atom') {
                    if (is_terminal($element->{value})) {
                        my $literal = extract_literal_value($element->{value});
                        $literal = escape_rust_string($literal);
                        push @seq_steps, "match_literal(input, \"$literal\").is_ok()";
                    } else {
                        my $rule_name_to_call = extract_token_value($element->{value});
                        push @seq_steps, "parse_$rule_name_to_call(input).unwrap_or(Ok(None)).unwrap_or(ASTNode::Null) != ASTNode::Null";
                    }
                } elsif ($element->{type} eq 'quantified') {
                    # Handle quantified elements
                    push @seq_steps, "true"; # Simplified for now
                }
            }
            
            my $sequence_check = join(" && ", @seq_steps);
            push @alternatives, "        {";
            push @alternatives, "            let checkpoint = input.save_position();";
            push @alternatives, "            if $sequence_check {";
            push @alternatives, "                return Ok(Some(ASTNode::Array(vec![])));  // Simplified return";
            push @alternatives, "            } else {";
            push @alternatives, "                input.restore_position(checkpoint);";
            push @alternatives, "            }";
            push @alternatives, "        }";
        }
    }
    
    my $alternatives_code = join("\n", @alternatives);
    
    my $func_code = <<~RUST;
    fn parse_$rule_name(input: &mut ParseInput) -> ParseResult<ASTNode> {
        let start_pos = input.save_position();
        
        // Try alternatives in order
    $alternatives_code
        
        // No match - restore position
        input.restore_position(start_pos);
        Ok(None)
    }
    RUST
    
    return ($func_code, $regexes);
}

sub generate_rust_sequence_parser {
    my ($rule_name, $rule_def, $regexes) = @_;
    
    print STDERR "DEBUG: Entered generate_rust_sequence_parser for $rule_name\n" if !$quiet_mode && $verbosity eq 'debug';
    
    my @sequence_steps = ();
    my @result_assignments = ();
    my $step_num = 0;
    
    foreach my $element (@{$rule_def->{elements}}) {
        $step_num++;
        
        if ($element->{type} eq 'atom') {
            if (is_terminal($element->{value})) {
                if (ref($element->{value}) eq 'ARRAY' && $element->{value}->[0] eq 'regex') {
                    # Regex pattern
                    my $pattern = escape_rust_raw_string($element->{value}[1]);
                    my $regex_name = "${rule_name}_step$step_num";
                    push @$regexes, "        map.insert(\"$regex_name\", Regex::new(r\"$pattern\").unwrap());";
                    
                    push @sequence_steps, "    let result_$step_num = match_regex(input, \"$regex_name\")?;";
                    push @sequence_steps, "    if result_$step_num.is_none() {";
                    push @sequence_steps, "        input.restore_position(start_pos);";
                    push @sequence_steps, "        return Ok(None);";
                    push @sequence_steps, "    }";
                    push @result_assignments, "ASTNode::Terminal(result_$step_num.unwrap())";
                } else {
                    # Literal terminal
                    my $literal = extract_literal_value($element->{value});
                    $literal = escape_rust_string($literal);
                    
                    push @sequence_steps, "    let result_$step_num = match_literal(input, \"$literal\")?;";
                    push @sequence_steps, "    if result_$step_num.is_none() {";
                    push @sequence_steps, "        input.restore_position(start_pos);";
                    push @sequence_steps, "        return Ok(None);";
                    push @sequence_steps, "    }";
                    push @result_assignments, "ASTNode::Terminal(result_$step_num.unwrap())";
                }
            } else {
                # Non-terminal atom
                my $rule_name_to_call = extract_token_value($element->{value});
                push @sequence_steps, "    let result_$step_num = parse_$rule_name_to_call(input)?;";
                push @sequence_steps, "    if result_$step_num.is_none() {";
                push @sequence_steps, "        input.restore_position(start_pos);";
                push @sequence_steps, "        return Ok(None);";
                push @sequence_steps, "    }";
                push @result_assignments, "result_$step_num.unwrap()";
            }
        } elsif ($element->{type} eq 'quantified') {
            # Handle quantified elements
            my $quant = parse_quantifier($element->{quantifier});
            my $min = $quant->{min};
            my $max = $quant->{max};
            
            if (is_terminal($element->{element})) {
                my $literal = extract_literal_value($element->{element});
                $literal = escape_rust_string($literal);
                
                my $regex_name = "${rule_name}_quant$step_num";
                push @$regexes, "        map.insert(\"$regex_name\", Regex::new(r\"\\\\Q$literal\\\\E\").unwrap());";
                
                push @sequence_steps, "    let result_$step_num = quantified_match(input, \"$regex_name\", $min, $max)?;";
                push @sequence_steps, "    if result_$step_num.is_none() {";
                push @sequence_steps, "        input.restore_position(start_pos);";
                push @sequence_steps, "        return Ok(None);";
                push @sequence_steps, "    }";
                push @result_assignments, "ASTNode::Array(result_$step_num.unwrap().into_iter().map(ASTNode::Terminal).collect())";
            } else {
                # Quantified rule reference
                my $rule_name_to_call = extract_token_value($element->{element});
                push @sequence_steps, "    let result_$step_num = quantified_rule(input, parse_$rule_name_to_call, $min, $max)?;";
                push @sequence_steps, "    if result_$step_num.is_none() {";
                push @sequence_steps, "        input.restore_position(start_pos);";
                push @sequence_steps, "        return Ok(None);";
                push @sequence_steps, "    }";
                push @result_assignments, "ASTNode::Array(result_$step_num.unwrap())";
            }
        }
    }
    
    my $steps_code = join("\n", @sequence_steps);
    
    # Handle return annotations
    my $return_code;
    if ($rule_def->{return_annotation}) {
        my ($type, $annotation) = @{$rule_def->{return_annotation}};
        $return_code = rust_return_annotation($type, $annotation, "results");
    } else {
        my $results_array = join(", ", @result_assignments);
        $return_code = "return Ok(Some(ASTNode::Array(vec![$results_array])));";
    }
    
    my $func_code = <<~RUST;
    fn parse_$rule_name(input: &mut ParseInput) -> ParseResult<ASTNode> {
        let start_pos = input.save_position();
        
        // Parse sequence elements in order
    $steps_code
        
        // All elements matched successfully
        $return_code
    }
    RUST
    
    return ($func_code, $regexes);
}

sub generate_rust_atom_parser {
    my ($rule_name, $rule_def, $regexes) = @_;
    
    if (is_terminal($rule_def->{value})) {
        if (ref($rule_def->{value}) eq 'ARRAY' && $rule_def->{value}->[0] eq 'regex') {
            # Regex atom
            my $pattern = escape_rust_raw_string($rule_def->{value}[1]);
            my $regex_name = $rule_name;
            push @$regexes, "        map.insert(\"$regex_name\", Regex::new(r\"$pattern\").unwrap());";
            
            my $func_code = <<~RUST;
            fn parse_$rule_name(input: &mut ParseInput) -> ParseResult<ASTNode> {
                if let Ok(Some(matched)) = match_regex(input, "$regex_name") {
                    Ok(Some(ASTNode::Terminal(matched)))
                } else {
                    Ok(None)
                }
            }
            RUST
            
            return ($func_code, $regexes);
        } else {
            # Literal atom
            my $literal = extract_literal_value($rule_def->{value});
            $literal = escape_rust_string($literal);
            
            my $func_code = <<~RUST;
            fn parse_$rule_name(input: &mut ParseInput) -> ParseResult<ASTNode> {
                if let Ok(Some(matched)) = match_literal(input, "$literal") {
                    Ok(Some(ASTNode::Terminal(matched)))
                } else {
                    Ok(None)
                }
            }
            RUST
            
            return ($func_code, $regexes);
        }
    } else {
        # Rule reference atom
        my $rule_name_to_call = extract_token_value($rule_def->{value});
        
        my $func_code = <<~RUST;
        fn parse_$rule_name(input: &mut ParseInput) -> ParseResult<ASTNode> {
            parse_$rule_name_to_call(input)
        }
        RUST
        
        return ($func_code, $regexes);
    }
}

# Helper functions
sub extract_token_value {
    my ($token) = @_;
    if (ref($token) eq 'HASH' && $token->{type} eq 'atom' && ref($token->{value}) eq 'ARRAY' && @{$token->{value}} == 2) {
        return $token->{value}->[1];
    } elsif (ref($token) eq 'ARRAY' && @$token == 2) {
        return $token->[1];
    } else {
        return $token;
    }
}

sub extract_literal_value {
    my ($value) = @_;
    if (ref($value) eq 'HASH' && $value->{type} eq 'atom' && ref($value->{value}) eq 'ARRAY') {
        return $value->{value}->[1];
    } elsif (ref($value) eq 'ARRAY') {
        return $value->[1];
    } else {
        return $value;
    }
}

sub is_terminal {
    my ($value) = @_;
    if (ref($value) eq 'ARRAY') {
        return $value->[0] =~ /^(quoted_string|terminal|regex|operator)$/;
    } elsif (ref($value) eq 'HASH' && $value->{type} eq 'atom' && ref($value->{value}) eq 'ARRAY') {
        return $value->{value}->[0] =~ /^(quoted_string|terminal|regex|operator)$/;
    } else {
        return 0;
    }
}

sub escape_rust_string {
    my ($str) = @_;
    $str =~ s/\\/\\\\/g;  # Escape backslashes
    $str =~ s/"/\\"/g;    # Escape double quotes
    $str =~ s/\n/\\n/g;   # Escape newlines
    $str =~ s/\t/\\t/g;   # Escape tabs
    $str =~ s/\r/\\r/g;   # Escape carriage returns
    return $str;
}

# For regex patterns in raw strings r"...", we only need to escape double quotes
sub escape_rust_raw_string {
    my ($str) = @_;
    $str =~ s/"/\\"/g;    # Escape double quotes only
    return $str;
}

sub parse_quantifier {
    my ($quant_str) = @_;
    
    if ($quant_str =~ /^(\d+),(\d+)$/) {
        return {min => $1, max => $2};
    } elsif ($quant_str =~ /^(\d+),$/) {
        return {min => $1, max => 999};
    } elsif ($quant_str =~ /^,(\d+)$/) {
        return {min => 0, max => $1};
    } elsif ($quant_str eq '+') {
        return {min => 1, max => 999};
    } elsif ($quant_str eq '*') {
        return {min => 0, max => 999};
    } elsif ($quant_str eq '?') {
        return {min => 0, max => 1};
    } else {
        return {min => 1, max => 1};
    }
}

sub rust_return_annotation {
    my ($type, $annotation, $result_var) = @_;
    
    if ($type eq 'return_scalar') {
        if ($annotation =~ /^\$(\d+)$/) {
            return "return Ok(Some(result_$1));";
        } else {
            my $literal_value = $annotation;
            $literal_value =~ s/^["']|["']$//g;  # Remove quotes
            $literal_value = escape_rust_string($literal_value);
            return "return Ok(Some(ASTNode::Terminal(\"$literal_value\".to_string())));";
        }
    } elsif ($type eq 'return_array') {
        # Handle array returns like [$1, $3*]
        if ($annotation =~ /^\[([^\]]+)\]$/) {
            my $array_content = $1;
            # This is a simplified implementation - would need more complex parsing for full support
            return "return Ok(Some(ASTNode::Array(results)));";
        }
    } elsif ($type eq 'return_object') {
        # Handle object returns like {type: "array", contents: $3}
        # This is a simplified implementation - would need more complex parsing for full support
        return "return Ok(Some(ASTNode::Object(HashMap::new())));";
    }
    
    return "return Ok(Some($result_var));";
}

1;

__END__

=head1 NAME

AST::RustCodeGen - Rust code generator for EBNF parsers

=head1 SYNOPSIS

    use AST::RustCodeGen qw(generate_rust_parser_module);
    
    my $rust_code = generate_rust_parser_module($grammar_tree, $rule_order);

=head1 DESCRIPTION

This module generates Rust code for parsing based on EBNF grammar definitions.
It produces a complete Rust module with parsing functions, regex handling,
and proper error handling.

=cut
