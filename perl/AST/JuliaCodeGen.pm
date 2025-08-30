package AST::JuliaCodeGen;

use strict;
use warnings;
use Data::Dumper;

use Exporter 'import';
our @EXPORT_OK = qw(generate_julia_parser_module);

# Global variables for configuration
our $quiet_mode = 0;
our $verbosity = 'normal';

sub generate_julia_parser_module {
    my ($grammar_tree, $rule_order) = @_;
    
    my @function_definitions = ();
    my @regex_definitions = ();
    
    # Generate parsing functions for each rule
    foreach my $rule_name (keys %$grammar_tree) {
        my $rule_def = $grammar_tree->{$rule_name};
        my ($func_code, $regexes) = generate_julia_parser_function($rule_name, $rule_def);
        push @function_definitions, $func_code;
        push @regex_definitions, @$regexes if $regexes;
    }
    
    # Build complete Julia module
    my $main_rule = $rule_order->[0];  # First rule is always the main entry point
    
    my $regex_definitions = join("\n", @regex_definitions);
    my $function_definitions = join("\n\n", @function_definitions);
    
    # Generate the Julia code
    my $julia_code = generate_julia_module_template($main_rule, $regex_definitions, $function_definitions);
    
    return $julia_code;
}

sub generate_julia_module_template {
    my ($main_rule, $regex_definitions, $function_definitions) = @_;
    
    my $template = <<'JULIA';
# Generated Julia Parser Module
# Auto-generated from EBNF grammar

using Base: String, Vector, Dict, Union, Nothing
import Base: show, print

# Error type for parsing failures
struct ParseError <: Exception
    message::String
    position::Int
end

# Result type for parsing - Julia Union type
const ParseResult{T} = Union{Some{T}, Nothing}

# Helper function to create Some() result
parse_some(value) = Some(value)
parse_none() = nothing

# AST node types
abstract type ASTNode end

struct Terminal <: ASTNode
    value::String
end

struct ArrayNode <: ASTNode
    elements::Vector{ASTNode}
end

struct ObjectNode <: ASTNode
    fields::Dict{String, ASTNode}
end

struct NumberNode <: ASTNode
    value::Union{Int64, Float64}
end

struct BoolNode <: ASTNode
    value::Bool
end

struct NullNode <: ASTNode
end

# Input position tracking
mutable struct ParseInput
    text::String
    position::Int
    
    function ParseInput(text::String)
        new(text, 1)  # Julia uses 1-based indexing
    end
end

function current_char(input::ParseInput)::Union{Char, Nothing}
    if input.position <= length(input.text)
        return input.text[input.position]
    else
        return nothing
    end
end

function advance!(input::ParseInput)::Union{Char, Nothing}
    if input.position <= length(input.text)
        ch = input.text[input.position]
        input.position += 1
        return ch
    else
        return nothing
    end
end

function peek(input::ParseInput, offset::Int = 0)::Union{Char, Nothing}
    pos = input.position + offset
    if pos <= length(input.text)
        return input.text[pos]
    else
        return nothing
    end
end

function slice(input::ParseInput, start::Int, stop::Int)::String
    safe_start = max(1, min(start, length(input.text)))
    safe_stop = max(1, min(stop, length(input.text)))
    return input.text[safe_start:safe_stop]
end

function remaining(input::ParseInput)::String
    if input.position <= length(input.text)
        return input.text[input.position:end]
    else
        return ""
    end
end

function save_position(input::ParseInput)::Int
    return input.position
end

function restore_position!(input::ParseInput, pos::Int)
    input.position = pos
end

function is_at_end(input::ParseInput)::Bool
    return input.position > length(input.text)
end

# Compiled regex patterns for speed
const REGEXES = Dict{String, Regex}(
REGEX_DEFINITIONS_PLACEHOLDER
)

# Helper function to match literal strings
function match_literal(input::ParseInput, literal::String)::Union{Some{String}, Nothing}
    remaining_text = remaining(input)
    if startswith(remaining_text, literal)
        input.position += length(literal)
        return parse_some(literal)
    else
        return parse_none()
    end
end

# Helper function to match regex patterns
function match_regex(input::ParseInput, regex_name::String)::Union{Some{String}, Nothing}
    if !haskey(REGEXES, regex_name)
        throw(ParseError("Regex '$regex_name' not found", input.position))
    end
    
    regex = REGEXES[regex_name]
    remaining_text = remaining(input)
    
    m = match(regex, remaining_text)
    if m !== nothing && m.offset == 1  # Must match at current position (1-based)
        matched_text = m.match
        input.position += length(matched_text)
        return parse_some(matched_text)
    else
        return parse_none()
    end
end

# Helper functions for quantified matching
function quantified_match(input::ParseInput, regex_name::String, min_count::Int, max_count::Int)::Union{Some{Vector{String}}, Nothing}
    matches = String[]
    start_pos = save_position(input)
    
    if !haskey(REGEXES, regex_name)
        throw(ParseError("Regex '$regex_name' not found", input.position))
    end
    
    regex = REGEXES[regex_name]
    
    for _ in 1:max_count
        remaining_text = remaining(input)
        m = match(regex, remaining_text)
        
        if m !== nothing && m.offset == 1  # Must match at current position
            matched_text = m.match
            push!(matches, matched_text)
            input.position += length(matched_text)
        else
            break
        end
    end
    
    if length(matches) >= min_count
        return parse_some(matches)
    else
        restore_position!(input, start_pos)
        return parse_none()
    end
end

function quantified_rule(input::ParseInput, rule_func::Function, min_count::Int, max_count::Int)
    results = ASTNode[]
    start_pos = save_position(input)
    
    for _ in 1:max_count
        result = rule_func(input)
        if result !== nothing
            push!(results, something(result))
        else
            break
        end
    end
    
    if length(results) >= min_count
        return parse_some(results)
    else
        restore_position!(input, start_pos)
        return parse_none()
    end
end

FUNCTION_DEFINITIONS_PLACEHOLDER

# Main entry point
function parse(text::String)::Union{Some{ASTNode}, Nothing}
    input = ParseInput(text)
    return MAIN_RULE_PLACEHOLDER(input)
end

# Pretty printing for ASTNode types
function show(io::IO, node::Terminal)
    print(io, "Terminal(\"$(node.value)\")")
end

function show(io::IO, node::ArrayNode)
    print(io, "ArrayNode([")
    for (i, elem) in enumerate(node.elements)
        if i > 1
            print(io, ", ")
        end
        show(io, elem)
    end
    print(io, "])")
end

function show(io::IO, node::ObjectNode)
    print(io, "ObjectNode({")
    first = true
    for (key, value) in node.fields
        if !first
            print(io, ", ")
        end
        print(io, "\"$key\" => ")
        show(io, value)
        first = false
    end
    print(io, "})")
end

function show(io::IO, node::NumberNode)
    print(io, "NumberNode($(node.value))")
end

function show(io::IO, node::BoolNode)
    print(io, "BoolNode($(node.value))")
end

function show(io::IO, node::NullNode)
    print(io, "NullNode()")
end

# Export main functions
export parse, ParseInput, ASTNode, Terminal, ArrayNode, ObjectNode, NumberNode, BoolNode, NullNode

JULIA
    
    # Replace placeholders
    $template =~ s/REGEX_DEFINITIONS_PLACEHOLDER/$regex_definitions/g;
    $template =~ s/FUNCTION_DEFINITIONS_PLACEHOLDER/$function_definitions/g;
    $template =~ s/MAIN_RULE_PLACEHOLDER/parse_$main_rule/g;
    
    return $template;
}

sub generate_julia_parser_function {
    my ($rule_name, $rule_def) = @_;
    my @regexes = ();
    
    # DEBUG: Track what happens to specific rules
    if ($rule_name eq 'index_list' && !$quiet_mode && $verbosity eq 'debug') {
        print STDERR "DEBUG: Generating Julia parser for index_list with rule_def:\n";
        print STDERR Dumper($rule_def);
    }
    
    my $type = $rule_def->{type};
    if ($type eq 'or') {
        return generate_julia_or_parser($rule_name, $rule_def, \@regexes);
    } elsif ($type eq 'sequence') {
        return generate_julia_sequence_parser($rule_name, $rule_def, \@regexes);
    } elsif ($type eq 'atom') {
        return generate_julia_atom_parser($rule_name, $rule_def, \@regexes);
    }
}

sub generate_julia_or_parser {
    my ($rule_name, $rule_def, $regexes) = @_;
    
    print STDERR "DEBUG: Entered generate_julia_or_parser for $rule_name\n" if !$quiet_mode && $verbosity eq 'debug';
    
    my @alternatives = ();
    
    foreach my $alt (@{$rule_def->{alternatives}}) {
        if ($alt->{type} eq 'atom') {
            if (is_terminal($alt->{value})) {
                if (ref($alt->{value}) eq 'ARRAY' && $alt->{value}->[0] eq 'epsilon') {
                    # Epsilon production - always succeeds
                    push @alternatives, "        return parse_some(Terminal(\"\"))";
                } elsif (ref($alt->{value}) eq 'ARRAY' && $alt->{value}->[0] eq 'regex') {
                    # Regex pattern
                    my $pattern = escape_julia_regex($alt->{value}[1]);
                    my $regex_name = "${rule_name}_alt" . @alternatives;
                    push @$regexes, "    \"$regex_name\" => r\"$pattern\"";
                    
                    if ($alt->{return_annotation}) {
                        my ($type, $annotation) = @{$alt->{return_annotation}};
                        push @alternatives, "    result = match_regex(input, \"$regex_name\")";
                        push @alternatives, "    if result !== nothing";
                        push @alternatives, "        " . julia_return_annotation($type, $annotation, "something(result)");
                        push @alternatives, "    end";
                    } else {
                        push @alternatives, "    result = match_regex(input, \"$regex_name\")";
                        push @alternatives, "    if result !== nothing";
                        push @alternatives, "        return parse_some(Terminal(something(result)))";
                        push @alternatives, "    end";
                    }
                } else {
                    # Literal terminal
                    my $literal = extract_literal_value($alt->{value});
                    $literal = escape_julia_string($literal);
                    
                    if ($alt->{return_annotation}) {
                        my ($type, $annotation) = @{$alt->{return_annotation}};
                        push @alternatives, "    result = match_literal(input, \"$literal\")";
                        push @alternatives, "    if result !== nothing";
                        push @alternatives, "        " . julia_return_annotation($type, $annotation, "something(result)");
                        push @alternatives, "    end";
                    } else {
                        push @alternatives, "    result = match_literal(input, \"$literal\")";
                        push @alternatives, "    if result !== nothing";
                        push @alternatives, "        return parse_some(Terminal(something(result)))";
                        push @alternatives, "    end";
                    }
                }
            } else {
                # Non-terminal atom - call other parser function
                my $rule_name_to_call = extract_token_value($alt->{value});
                push @alternatives, "    result = parse_$rule_name_to_call(input)";
                push @alternatives, "    if result !== nothing";
                push @alternatives, "        return result";
                push @alternatives, "    end";
            }
        } elsif ($alt->{type} eq 'sequence') {
            # For sequences in alternatives, generate inline matching code
            push @alternatives, "    # Sequence alternative - simplified implementation";
            push @alternatives, "    checkpoint = save_position(input)";
            push @alternatives, "    # TODO: Implement full sequence matching";
            push @alternatives, "    restore_position!(input, checkpoint)";
        }
    }
    
    my $alternatives_code = join("\n", @alternatives);
    
    my $func_code = <<~JULIA;
    function parse_$rule_name(input::ParseInput)::Union{Some{ASTNode}, Nothing}
        start_pos = save_position(input)
        
        # Try alternatives in order
    $alternatives_code
        
        # No match - restore position
        restore_position!(input, start_pos)
        return parse_none()
    end
    JULIA
    
    return ($func_code, $regexes);
}

sub generate_julia_sequence_parser {
    my ($rule_name, $rule_def, $regexes) = @_;
    
    print STDERR "DEBUG: Entered generate_julia_sequence_parser for $rule_name\n" if !$quiet_mode && $verbosity eq 'debug';
    
    my @sequence_steps = ();
    my @result_assignments = ();
    my $step_num = 0;
    
    foreach my $element (@{$rule_def->{elements}}) {
        $step_num++;
        
        if ($element->{type} eq 'atom') {
            if (is_terminal($element->{value})) {
                if (ref($element->{value}) eq 'ARRAY' && $element->{value}->[0] eq 'regex') {
                    # Regex pattern
                    my $pattern = escape_julia_regex($element->{value}[1]);
                    my $regex_name = "${rule_name}_step$step_num";
                    push @$regexes, "    \"$regex_name\" => r\"$pattern\"";
                    
                    push @sequence_steps, "    result_$step_num = match_regex(input, \"$regex_name\")";
                    push @sequence_steps, "    if result_$step_num === nothing";
                    push @sequence_steps, "        restore_position!(input, start_pos)";
                    push @sequence_steps, "        return parse_none()";
                    push @sequence_steps, "    end";
                    push @result_assignments, "Terminal(something(result_$step_num))";
                } else {
                    # Literal terminal
                    my $literal = extract_literal_value($element->{value});
                    $literal = escape_julia_string($literal);
                    
                    push @sequence_steps, "    result_$step_num = match_literal(input, \"$literal\")";
                    push @sequence_steps, "    if result_$step_num === nothing";
                    push @sequence_steps, "        restore_position!(input, start_pos)";
                    push @sequence_steps, "        return parse_none()";
                    push @sequence_steps, "    end";
                    push @result_assignments, "Terminal(something(result_$step_num))";
                }
            } else {
                # Non-terminal atom
                my $rule_name_to_call = extract_token_value($element->{value});
                push @sequence_steps, "    result_$step_num = parse_$rule_name_to_call(input)";
                push @sequence_steps, "    if result_$step_num === nothing";
                push @sequence_steps, "        restore_position!(input, start_pos)";
                push @sequence_steps, "        return parse_none()";
                push @sequence_steps, "    end";
                push @result_assignments, "something(result_$step_num)";
            }
        } elsif ($element->{type} eq 'quantified') {
            # Handle quantified elements
            my $quant = parse_quantifier($element->{quantifier});
            my $min = $quant->{min};
            my $max = $quant->{max};
            
            if (is_terminal($element->{element})) {
                my $literal = extract_literal_value($element->{element});
                $literal = escape_julia_string($literal);
                
                my $regex_name = "${rule_name}_quant$step_num";
                # Escape the literal for regex
                my $escaped_literal = $literal;
                $escaped_literal =~ s/([.*+?^\${}()|\[\]\\])/\\$1/g;
                push @$regexes, "    \"$regex_name\" => r\"$escaped_literal\"";
                
                push @sequence_steps, "    result_$step_num = quantified_match(input, \"$regex_name\", $min, $max)";
                push @sequence_steps, "    if result_$step_num === nothing";
                push @sequence_steps, "        restore_position!(input, start_pos)";
                push @sequence_steps, "        return parse_none()";
                push @sequence_steps, "    end";
                push @result_assignments, "ArrayNode([Terminal(s) for s in something(result_$step_num)])";
            } else {
                # Quantified rule reference
                my $rule_name_to_call = extract_token_value($element->{element});
                push @sequence_steps, "    result_$step_num = quantified_rule(input, parse_$rule_name_to_call, $min, $max)";
                push @sequence_steps, "    if result_$step_num === nothing";
                push @sequence_steps, "        restore_position!(input, start_pos)";
                push @sequence_steps, "        return parse_none()";
                push @sequence_steps, "    end";
                push @result_assignments, "ArrayNode(something(result_$step_num))";
            }
        }
    }
    
    my $steps_code = join("\n", @sequence_steps);
    
    # Handle return annotations
    my $return_code;
    if ($rule_def->{return_annotation}) {
        my ($type, $annotation) = @{$rule_def->{return_annotation}};
        $return_code = julia_return_annotation($type, $annotation, "results");
    } else {
        my $results_array = join(", ", @result_assignments);
        $return_code = "return parse_some(ArrayNode([$results_array]))";
    }
    
    my $func_code = <<~JULIA;
    function parse_$rule_name(input::ParseInput)::Union{Some{ASTNode}, Nothing}
        start_pos = save_position(input)
        
        # Parse sequence elements in order
    $steps_code
        
        # All elements matched successfully
        $return_code
    end
    JULIA
    
    return ($func_code, $regexes);
}

sub generate_julia_atom_parser {
    my ($rule_name, $rule_def, $regexes) = @_;
    
    if (is_terminal($rule_def->{value})) {
        if (ref($rule_def->{value}) eq 'ARRAY' && $rule_def->{value}->[0] eq 'regex') {
            # Regex atom
            my $pattern = escape_julia_regex($rule_def->{value}[1]);
            my $regex_name = $rule_name;
            push @$regexes, "    \"$regex_name\" => r\"$pattern\"";
            
            my $func_code = <<~JULIA;
            function parse_$rule_name(input::ParseInput)::Union{Some{ASTNode}, Nothing}
                result = match_regex(input, "$regex_name")
                if result !== nothing
                    return parse_some(Terminal(something(result)))
                else
                    return parse_none()
                end
            end
            JULIA
            
            return ($func_code, $regexes);
        } else {
            # Literal atom
            my $literal = extract_literal_value($rule_def->{value});
            $literal = escape_julia_string($literal);
            
            my $func_code = <<~JULIA;
            function parse_$rule_name(input::ParseInput)::Union{Some{ASTNode}, Nothing}
                result = match_literal(input, "$literal")
                if result !== nothing
                    return parse_some(Terminal(something(result)))
                else
                    return parse_none()
                end
            end
            JULIA
            
            return ($func_code, $regexes);
        }
    } else {
        # Rule reference atom
        my $rule_name_to_call = extract_token_value($rule_def->{value});
        
        my $func_code = <<~JULIA;
        function parse_$rule_name(input::ParseInput)::Union{Some{ASTNode}, Nothing}
            return parse_$rule_name_to_call(input)
        end
        JULIA
        
        return ($func_code, $regexes);
    }
}

# Helper functions (reused from RustCodeGen with Julia adaptations)
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

sub escape_julia_string {
    my ($str) = @_;
    $str =~ s/\\/\\\\/g;  # Escape backslashes
    $str =~ s/"/\\"/g;    # Escape double quotes
    $str =~ s/\n/\\n/g;   # Escape newlines
    $str =~ s/\t/\\t/g;   # Escape tabs
    $str =~ s/\r/\\r/g;   # Escape carriage returns
    return $str;
}

sub escape_julia_regex {
    my ($str) = @_;
    # For Julia regex literals r"...", we need minimal escaping
    $str =~ s/"/\\"/g;    # Escape double quotes
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

sub julia_return_annotation {
    my ($type, $annotation, $result_var) = @_;
    
    if ($type eq 'return_scalar') {
        if ($annotation =~ /^\$(\d+)$/) {
            return "return parse_some(result_$1)";
        } else {
            my $literal_value = $annotation;
            $literal_value =~ s/^["']|["']$//g;  # Remove quotes
            $literal_value = escape_julia_string($literal_value);
            return "return parse_some(Terminal(\"$literal_value\"))";
        }
    } elsif ($type eq 'return_array') {
        # Handle array returns like [$1, $3*]
        if ($annotation =~ /^\[([^\]]+)\]$/) {
            my $array_content = $1;
            # This is a simplified implementation - would need more complex parsing for full support
            return "return parse_some(ArrayNode($result_var))";
        }
    } elsif ($type eq 'return_object') {
        # Handle object returns like {type: "array", contents: $3}
        # This is a simplified implementation - would need more complex parsing for full support
        return "return parse_some(ObjectNode(Dict{String, ASTNode}()))";
    }
    
    return "return parse_some($result_var)";
}

1;

__END__

=head1 NAME

AST::JuliaCodeGen - Julia code generator for EBNF parsers

=head1 SYNOPSIS

    use AST::JuliaCodeGen qw(generate_julia_parser_module);
    
    my $julia_code = generate_julia_parser_module($grammar_tree, $rule_order);

=head1 DESCRIPTION

This module generates Julia code for parsing based on EBNF grammar definitions.
It produces a complete Julia module with parsing functions, regex handling,
and proper error handling using Julia's type system and Union types.

Key features:
- Type-safe parsing with Union{Some{T}, Nothing} results
- Regex support with compiled patterns
- Position tracking and error handling
- Multiple AST node types (Terminal, ArrayNode, ObjectNode, etc.)
- Quantifier support for repetition patterns
- Pretty printing for AST nodes

=cut
