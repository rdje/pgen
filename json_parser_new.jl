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
    result = String[]
    push!(result, match_string!(p, "\s*\[\s*"))
push!(result, parse_elements(p))
push!(result, match_string!(p, "\s*\]\s*"))
    return join(result, "")
end
# Parse elements rule  
function parse_elements(p::Parser)
    skip_whitespace!(p)
    return parse_value(p)
end
# Parse object rule
function parse_object(p::Parser)
    skip_whitespace!(p)
    result = String[]
    push!(result, match_string!(p, "\s*\{\s*"))
push!(result, parse_members(p))
push!(result, match_string!(p, "\s*\}\s*"))
    return join(result, "")
end
# Parse members rule  
function parse_members(p::Parser)
    skip_whitespace!(p)
    return parse_pair(p)
end
# Parse value rule
function parse_value(p::Parser)
    skip_whitespace!(p)
    return match_string!(p, "\s*null\s*")
end
# Main parse function
function parse(input::String)
    parser = Parser(input)
    skip_whitespace!(parser)
    result = parse_string(parser)
    
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
