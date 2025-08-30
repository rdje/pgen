# JSON Parser
# 
# Generated from EBNF grammar using julia_parser_gen
# This parser implements a recursive descent parser with comprehensive error handling.

module JsonParser

export parse, ParseError

# Parse error with position information
struct ParseError <: Exception
    message::String
    position::Int
    expected::Vector{String}
end

Base.showerror(io::IO, e::ParseError) = print(io, "Parse error at position $(e.position): $(e.message)")

# Parser state
mutable struct Parser
    input::String
    position::Int
    
    Parser(input::String) = new(input, 1)
end

# Get current position (1-indexed)
position(p::Parser) = p.position

# Check if at end of input
is_at_end(p::Parser) = p.position > length(p.input)

# Peek at current character without consuming
function peek(p::Parser)
    if is_at_end(p)
        return nothing
    else
        return p.input[p.position]
    end
end

# Advance position and return current character
function advance!(p::Parser)
    if is_at_end(p)
        return nothing
    else
        ch = p.input[p.position]
        p.position += 1
        return ch
    end
end

# Match exact string
function match_string!(p::Parser, expected::String)
    start_pos = p.position
    
    for expected_char in expected
        ch = advance!(p)
        if ch === nothing
            p.position = start_pos  # Backtrack
            throw(ParseError("Unexpected end of input, expected '$expected'", p.position, [expected]))
        elseif ch != expected_char
            p.position = start_pos  # Backtrack
            throw(ParseError("Expected '$expected_char', found '$ch'", p.position, [string(expected_char)]))
        end
    end
    
    return expected
end

# Skip whitespace
function skip_whitespace!(p::Parser)
    while !is_at_end(p) && isspace(peek(p))
        advance!(p)
    end
end

# Parse string rule
function parse_string(p::Parser)
    skip_whitespace!(p)
    return match_string!(p, "\s*\"[^\"]*\"\s*")
end
# Parse number rule
function parse_number(p::Parser)
    skip_whitespace!(p)
    return match_string!(p, "\s*-?[0-9]+(\.[0-9]+)?\s*")
end
# Parse json rule  
function parse_json(p::Parser)
    skip_whitespace!(p)
    return parse_value(p)
end
# Parse pair rule
function parse_pair(p::Parser)
    skip_whitespace!(p)
    result = String[]
    push!(result, parse_string(p))
push!(result, match_string!(p, "\s*:\s*"))
push!(result, parse_value(p))
    return join(result, "")
end
# Parse array rule
function parse_array(p::Parser)
    skip_whitespace!(p)
    start_pos = p.position
    errors = ParseError[]
    
    # Try alternatives in order
# Try alternative 1
p.position = start_pos
try
    result = "" * match_string!(p, "\s*\[\s*\]\s*")
    return result
catch e
    push!(errors, e)
end
# Try alternative 2
p.position = start_pos
try
    result = "" * match_string!(p, "\s*\[\s*") * parse_elements(p) * match_string!(p, "\s*\]\s*")
    return result
catch e
    push!(errors, e)
end    
    # All alternatives failed
    throw(ParseError("No valid alternative found for array", start_pos, ["valid array alternative"]))
end
# Parse elements rule
function parse_elements(p::Parser)
    skip_whitespace!(p)
    start_pos = p.position
    errors = ParseError[]
    
    # Try alternatives in order
# Try alternative 1
p.position = start_pos
try
    result = "" * parse_value(p) * match_string!(p, "\s*,\s*") * parse_elements(p)
    return result
catch e
    push!(errors, e)
end
# Try alternative 2
p.position = start_pos
try
    result = "" * parse_value(p)
    return result
catch e
    push!(errors, e)
end    
    # All alternatives failed
    throw(ParseError("No valid alternative found for elements", start_pos, ["valid elements alternative"]))
end
# Parse object rule
function parse_object(p::Parser)
    skip_whitespace!(p)
    start_pos = p.position
    errors = ParseError[]
    
    # Try alternatives in order
# Try alternative 1
p.position = start_pos
try
    result = "" * match_string!(p, "\s*\{\s*\}\s*")
    return result
catch e
    push!(errors, e)
end
# Try alternative 2
p.position = start_pos
try
    result = "" * match_string!(p, "\s*\{\s*") * parse_members(p) * match_string!(p, "\s*\}\s*")
    return result
catch e
    push!(errors, e)
end    
    # All alternatives failed
    throw(ParseError("No valid alternative found for object", start_pos, ["valid object alternative"]))
end
# Parse members rule
function parse_members(p::Parser)
    skip_whitespace!(p)
    start_pos = p.position
    errors = ParseError[]
    
    # Try alternatives in order
# Try alternative 1
p.position = start_pos
try
    result = "" * parse_pair(p) * match_string!(p, "\s*,\s*") * parse_members(p)
    return result
catch e
    push!(errors, e)
end
# Try alternative 2
p.position = start_pos
try
    result = "" * parse_pair(p)
    return result
catch e
    push!(errors, e)
end    
    # All alternatives failed
    throw(ParseError("No valid alternative found for members", start_pos, ["valid members alternative"]))
end
# Parse value rule
function parse_value(p::Parser)
    skip_whitespace!(p)
    start_pos = p.position
    errors = ParseError[]
    
    # Try alternatives in order
# Try alternative 1
p.position = start_pos
try
    result = "" * parse_object(p)
    return result
catch e
    push!(errors, e)
end
# Try alternative 2
p.position = start_pos
try
    result = "" * parse_array(p)
    return result
catch e
    push!(errors, e)
end
# Try alternative 3
p.position = start_pos
try
    result = "" * parse_string(p)
    return result
catch e
    push!(errors, e)
end
# Try alternative 4
p.position = start_pos
try
    result = "" * parse_number(p)
    return result
catch e
    push!(errors, e)
end
# Try alternative 5
p.position = start_pos
try
    result = "" * match_string!(p, "\s*true\s*")
    return result
catch e
    push!(errors, e)
end
# Try alternative 6
p.position = start_pos
try
    result = "" * match_string!(p, "\s*false\s*")
    return result
catch e
    push!(errors, e)
end
# Try alternative 7
p.position = start_pos
try
    result = "" * match_string!(p, "\s*null\s*")
    return result
catch e
    push!(errors, e)
end    
    # All alternatives failed
    throw(ParseError("No valid alternative found for value", start_pos, ["valid value alternative"]))
end
# Main parse function
function parse(input::String)
    parser = Parser(input)
    skip_whitespace!(parser)
    result = parse_json(parser)
    
    skip_whitespace!(parser)
    if !is_at_end(parser)
        throw(ParseError("Unexpected input after parsing", parser.position, ["end of input"]))
    end
    
    return result
end

end # module

# Re-export parse function for convenience
using .JsonParser
export parse

# Example usage (uncomment to test):
# result = parse("your input here")
# println("Parsed result: ", result)
