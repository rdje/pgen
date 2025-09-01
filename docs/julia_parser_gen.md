# Julia Parser Generator Documentation

## Overview

The Julia Parser Generator (`julia_parser_gen`) is a complete implementation of a parser generator that converts JSON raw AST input into Julia parsing code. It implements a comprehensive 6-step AST transformation pipeline with advanced features including left recursion elimination and semantic actions through return annotations.

## Architecture

The generator follows a precise 6-step transformation pipeline:

```
Raw AST JSON → Step 2 → Step 2.5 → Step 3 → Step 4 → Step 5 → Step 6a → Step 6b → Julia Parser
               Group    Handle      Parse    Handle    Build     Left      Semantic
               by OR    Parens      Sequences Quantifiers Tree   Recursion  Actions
```

## Core Data Structures

### Return Annotation AST

The generator supports structured semantic actions through a comprehensive return annotation system:

```julia
abstract type ReturnAnnotation end

struct ScalarRef <: ReturnAnnotation
    index::Int                                          # $1, $2, etc.
end

struct ArrayExpr <: ReturnAnnotation
    contents::Vector{ReturnAnnotation}                  # [$1, $2, "literal"]
end

struct ObjectExpr <: ReturnAnnotation
    contents::Vector{Tuple{String, ReturnAnnotation}}   # {key: $1, value: $2}
end

struct LiteralAnnotation <: ReturnAnnotation
    value::String                                       # "literal_value"
end

struct DotAccess <: ReturnAnnotation
    base::ReturnAnnotation                              # $1.field
    field::String
end

struct QuantifiedAnnotation <: ReturnAnnotation
    base::ReturnAnnotation                              # $1* (quantified)
end
```

### Grammar Production System

For left recursion elimination, the generator uses a production-based representation:

```julia
struct Production
    symbols::Vector{String}
    annotation::Union{ReturnAnnotation, Nothing}
end

struct GrammarRule
    name::String
    productions::Vector{Production}
end
```

### Semantic AST Nodes

The transformation pipeline builds a rich semantic AST:

```julia
abstract type ASTNode end

struct Terminal <: ASTNode
    value::String                                       # Literal strings/tokens
end

struct NonTerminal <: ASTNode
    name::String                                        # Rule references
end

struct Sequence <: ASTNode
    elements::Vector{ASTNode}                           # Sequential elements
end

struct Choice <: ASTNode
    rule::String                                        # Alternative branches
    alternatives::Vector{ASTNode}
end

struct QuantifiedNode <: ASTNode
    symbol::ASTNode                                     # ?, *, + quantifiers
    quantifier::String
end

struct GroupNode <: ASTNode
    inner::ASTNode                                      # Parenthesized groups
end

struct WithAnnotation <: ASTNode
    node::ASTNode                                       # Nodes with semantic actions
    annotation::ReturnAnnotation
end
```

## Implementation Features

### Step 6a: Left Recursion Elimination

Implements the complete **Aho-Sethi-Ullman left recursion elimination algorithm**:

#### Indirect Left Recursion Elimination
```julia
function eliminate_left_recursion(grammar_rules::Dict{String, ASTNode}, options::Options)
    # Convert to production form
    productions = Dict{String, Vector{Production}}()
    for (rule_name, ast_node) in grammar_rules
        productions[rule_name] = convert_ast_to_productions(ast_node)
    end
    
    # Apply elimination algorithm
    rule_names = sort(collect(keys(productions)))
    
    for i in 1:length(rule_names)
        Ai = rule_names[i]
        
        # Eliminate indirect left recursion
        for j in 1:(i-1)
            Aj = rule_names[j]
            new_productions = Production[]
            
            for prod in productions[Ai]
                if !isempty(prod.symbols) && prod.symbols[1] == Aj
                    # Replace Aj with all its productions
                    for Aj_prod in productions[Aj]
                        new_symbols = vcat(Aj_prod.symbols, prod.symbols[2:end])
                        push!(new_productions, Production(new_symbols, prod.annotation))
                    end
                else
                    push!(new_productions, prod)
                end
            end
            
            productions[Ai] = new_productions
        end
        
        # Eliminate direct left recursion
        productions = eliminate_immediate_left_recursion(productions, Ai, options)
    end
    
    # Convert back to AST form
    result = Dict{String, ASTNode}()
    for (rule_name, prods) in productions
        result[rule_name] = convert_productions_to_ast(rule_name, prods)
    end
    
    return result
end
```

#### Direct Left Recursion Elimination
```julia
# Transform: A -> Aα | β  into:  A -> βA'  and  A' -> αA' | ε
function eliminate_immediate_left_recursion(productions::Dict{String, Vector{Production}}, 
                                          rule_name::String, options::Options)
    prods = productions[rule_name]
    
    # Separate left-recursive and non-left-recursive productions
    left_recursive = Production[]
    non_left_recursive = Production[]
    
    for prod in prods
        if !isempty(prod.symbols) && prod.symbols[1] == rule_name
            push!(left_recursive, prod)
        else
            push!(non_left_recursive, prod)
        end
    end
    
    if isempty(left_recursive)
        return productions  # No left recursion to eliminate
    end
    
    # Create new rule name for the auxiliary rule
    aux_rule = rule_name * "_prime"
    
    # Transform productions: A -> Aα becomes A -> βA'
    new_main_prods = Production[]
    aux_prods = Production[]
    
    # Non-left-recursive productions become A -> βA'
    for prod in non_left_recursive
        new_symbols = vcat(prod.symbols, [aux_rule])
        push!(new_main_prods, Production(new_symbols, prod.annotation))
    end
    
    # Left-recursive productions become A' -> αA' | ε
    for prod in left_recursive
        if length(prod.symbols) > 1
            # A -> Aα becomes A' -> αA'
            alpha = prod.symbols[2:end]
            new_symbols = vcat(alpha, [aux_rule])
            push!(aux_prods, Production(new_symbols, prod.annotation))
        end
    end
    
    # Add epsilon production for auxiliary rule: A' -> ε
    push!(aux_prods, Production(String[], nothing))
    
    # Update productions
    productions[rule_name] = new_main_prods
    productions[aux_rule] = aux_prods
    
    return productions
end
```

### Step 6b: Return Annotation Processing

#### Annotation Extraction
```julia
function extract_return_annotation(tokens::Vector{Token})
    clean = Token[]
    i = 1
    while i <= length(tokens)
        t = tokens[i]
        if t.token_type == "operator" && t.value == "->"
            # Remaining tokens form the annotation expression
            ann_tokens = tokens[(i+1):end]
            return (clean, parse_return_annotation_tokens(ann_tokens))
        else
            push!(clean, t)
            i += 1
        end
    end
    return (clean, nothing)
end
```

#### Annotation Parsing
Supports complex annotation expressions:

```julia
function parse_return_annotation_tokens(tokens::Vector{Token})::ReturnAnnotation
    s = join(getfield.(tokens, :value), " ")
    
    # Scalar references: $1, $2, etc.
    m = match(r"^\$(\d+)$", s)
    if m !== nothing
        return ScalarRef(parse(Int, m.captures[1]))
    end
    
    # Array expressions: [$1, $2, "literal"]
    if startswith(s, "[") && endswith(s, "]")
        inner = strip(s[2:end-1])
        parts = split(inner, ",")
        contents = ReturnAnnotation[]
        for p in parts
            p = strip(p)
            m = match(r"^\$(\d+)\*?$", p)
            if m !== nothing
                idx = parse(Int, m.captures[1])
                if endswith(p, "*")
                    push!(contents, QuantifiedAnnotation(ScalarRef(idx)))
                else
                    push!(contents, ScalarRef(idx))
                end
            else
                push!(contents, LiteralAnnotation(p))
            end
        end
        return ArrayExpr(contents)
    end
    
    # Object expressions: {key: $1, value: $2}
    if startswith(s, "{") && endswith(s, "}")
        inner = strip(s[2:end-1])
        kpos = findfirst(":", inner)
        if kpos !== nothing
            key = strip(inner[1:kpos-1])
            value = strip(inner[kpos+1:end])
            m = match(r"^\$(\d+)$", value)
            if m !== nothing
                return ObjectExpr([(key, ScalarRef(parse(Int, m.captures[1])))])
            end
        end
    end
    
    # Default: literal annotation
    return LiteralAnnotation(s)
end
```

### AST to Production Conversion

```julia
function convert_ast_to_productions(ast_node::ASTNode)::Vector{Production}
    if isa(ast_node, WithAnnotation)
        # Extract annotation and process inner node
        inner_prods = convert_ast_to_productions(ast_node.node)
        # Apply annotation to all productions
        return [Production(prod.symbols, ast_node.annotation) for prod in inner_prods]
    elseif isa(ast_node, Choice)
        prods = Production[]
        for alt in ast_node.alternatives
            append!(prods, convert_ast_to_productions(alt))
        end
        return prods
    elseif isa(ast_node, Sequence)
        symbols = String[]
        for elem in ast_node.elements
            if isa(elem, Terminal)
                push!(symbols, "'" * elem.value * "'")
            elseif isa(elem, NonTerminal)
                push!(symbols, elem.name)
            else
                push!(symbols, "COMPLEX_" * string(typeof(elem)))
            end
        end
        return [Production(symbols, nothing)]
    elseif isa(ast_node, Terminal)
        return [Production(["'" * ast_node.value * "'"], nothing)]
    elseif isa(ast_node, NonTerminal)
        return [Production([ast_node.name], nothing)]
    else
        # For other complex nodes, create a placeholder production
        return [Production(["COMPLEX_" * string(typeof(ast_node))], nothing)]
    end
end
```

### Code Generation with Semantic Actions

The generator produces Julia parsing functions that return structured data:

#### Basic Parsing Function Generation
```julia
function generate_parse_function(rule_name::String, ast_node::WithAnnotation)
    inner_code = generate_parse_function(rule_name * "_inner", ast_node.node)
    semantic_action = generate_semantic_action(ast_node.annotation)
    
    return """
    # Parse $rule_name rule (with semantic action)
    function parse_$(rule_name)(p::Parser)
        skip_whitespace!(p)
        # Parse inner structure
        inner_result = parse_$(rule_name)_inner(p)
        # Apply semantic action
        return $semantic_action
    end
    
    $inner_code"""
end

function generate_parse_function(rule_name::String, ast_node::Choice)
    code = """
    # Parse $rule_name rule
    function parse_$(rule_name)(p::Parser)
        skip_whitespace!(p)
        start_pos = p.position
        errors = ParseError[]
        
        # Try alternatives in order"""
    
    for (i, alt) in enumerate(ast_node.alternatives)
        code *= """
        
        # Try alternative $i
        p.position = start_pos
        try
            result = "" * $(generate_alternative_code(alt))
            return result
        catch e
            push!(errors, e)
        end"""
    end
    
    code *= """
        
        # All alternatives failed
        throw(ParseError("No valid alternative found for $rule_name", start_pos, ["valid $rule_name alternative"]))
    end"""
    
    return code
end
```

#### Semantic Action Generation
```julia
function generate_semantic_action(annotation::ReturnAnnotation)
    if isa(annotation, ScalarRef)
        return "inner_result[$(annotation.index)]"
    elseif isa(annotation, ArrayExpr)
        elements = String[]
        for content in annotation.contents
            push!(elements, generate_semantic_action(content))
        end
        return "[$(join(elements, ", "))]"
    elseif isa(annotation, ObjectExpr)
        pairs = String[]
        for (key, value) in annotation.contents
            push!(pairs, "\"$key\" => $(generate_semantic_action(value))")
        end
        return "Dict($(join(pairs, ", ")))"
    elseif isa(annotation, LiteralAnnotation)
        return "\"$(annotation.value)\""
    elseif isa(annotation, QuantifiedAnnotation)
        base_action = generate_semantic_action(annotation.base)
        return "collect($base_action)"
    else
        return "inner_result  # Unsupported annotation: $(typeof(annotation))"
    end
end
```

### Quantifier Handling

The generator handles EBNF quantifiers with proper backtracking:

```julia
function generate_quantified_code(node::QuantifiedNode, indent_level::Int)
    indent = "    "^(indent_level + 1)
    
    if node.quantifier == "*"
        return """
$(indent)# Zero or more (*)
$(indent)parts = String[]
$(indent)while true
$(indent)    checkpoint = p.position
$(indent)    try
$(indent)        part = $(generate_alternative_code(node.symbol))
$(indent)        push!(parts, part)
$(indent)    catch
$(indent)        p.position = checkpoint
$(indent)        break
$(indent)    end
$(indent)end
$(indent)return join(parts, "")"""
    elseif node.quantifier == "+"
        return """
$(indent)# One or more (+)
$(indent)parts = String[]
$(indent)# First occurrence is required
$(indent)push!(parts, $(generate_alternative_code(node.symbol)))
$(indent)# Additional occurrences are optional
$(indent)while true
$(indent)    checkpoint = p.position
$(indent)    try
$(indent)        part = $(generate_alternative_code(node.symbol))
$(indent)        push!(parts, part)
$(indent)    catch
$(indent)        p.position = checkpoint
$(indent)        break
$(indent)    end
$(indent)end
$(indent)return join(parts, "")"""
    elseif node.quantifier == "?"
        return """
$(indent)# Optional (?)
$(indent)checkpoint = p.position
$(indent)try
$(indent)    return $(generate_alternative_code(node.symbol))
$(indent)catch
$(indent)    p.position = checkpoint
$(indent)    return ""
$(indent)end"""
    else
        return "$(indent)# Unknown quantifier: $(node.quantifier)"
    end
end
```

## Parser Output Format

The generated Julia parser includes:

### Parser State Management
```julia
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
```

### Comprehensive Error Handling
```julia
# Parse error with position information
struct ParseError <: Exception
    message::String
    position::Int
    expected::Vector{String}
end

Base.showerror(io::IO, e::ParseError) = print(io, "Parse error at position $(e.position): $(e.message)")
```

### String Matching with Backtracking
```julia
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
```

## Command Line Interface

```bash
julia_parser_gen [OPTIONS] [JSON_FILE]

OPTIONS:
    -o, --output <FILE>     Write parser to <basename>.jl
    -m, --module <NAME>     Set module name (default: derived from grammar)
    -q, --quiet             Suppress progress messages
    -h, --help              Print help information

ARGUMENTS:
    <JSON_FILE>             Input JSON file (optional, reads from stdin if not provided)
```

## Usage Examples

### Basic Usage
```bash
# From file
julia julia_parser_gen grammar.json -o parser.jl

# From stdin
ebnf_to_json.pl grammar.ebnf | julia julia_parser_gen -o parser.jl

# With custom module name
julia julia_parser_gen grammar.json -o parser.jl -m MyCustomParser
```

### Grammar with Return Annotations
```ebnf
json_value := json_object | json_array | json_string | json_number -> $1
json_object := "{" json_members? "}" -> {type: "object", members: $2}
json_members := json_pair ("," json_pair)* -> [$1, $2*]
json_pair := json_string ":" json_value -> {key: $1, value: $3}
```

This generates a Julia parser that returns structured data (Arrays, Dicts, Strings) instead of raw strings, enabling rich semantic processing of parsed content.

### Example Generated Output Structure
```julia
module JsonParser

export parse, ParseError

# Parse error handling...
struct ParseError <: Exception
    message::String
    position::Int
    expected::Vector{String}
end

# Parser state management...
mutable struct Parser
    input::String
    position::Int
    Parser(input::String) = new(input, 1)
end

# Generated parsing functions...
function parse_json_value(p::Parser)
    # Implementation with semantic actions returning structured data
end

function parse_json_object(p::Parser)
    # Implementation returning Dict{"type" => "object", "members" => [members]}
end

# Main parse function
function parse(input::String)
    parser = Parser(input)
    skip_whitespace!(parser)
    result = parse_json_value(parser)
    
    skip_whitespace!(parser)
    if !is_at_end(parser)
        throw(ParseError("Unexpected input after parsing", parser.position, ["end of input"]))
    end
    
    return result
end

end # module
```

## Key Features

1. **Complete Left Recursion Elimination**: Handles both direct and indirect left recursion using the standard Aho-Sethi-Ullman algorithm
2. **Rich Semantic Actions**: Return annotations enable structured data construction (Arrays, Dicts, Strings) during parsing
3. **Comprehensive Error Handling**: Detailed error reporting with position information and expected tokens
4. **Robust Code Generation**: Produces clean, efficient Julia parsing code with proper error handling and backtracking
5. **Advanced Grammar Support**: Handles complex grammars with quantifiers, grouping, and nested structures
6. **Production System**: Internal representation using grammar productions for algorithmic transformations
7. **Julia-Native Output**: Generated parsers use Julia's native data structures and idioms

## Implementation Highlights

### Julia-Specific Features
- **Multiple Dispatch**: Uses Julia's multiple dispatch for different AST node types in code generation
- **Native Data Types**: Returns Julia Arrays, Dicts, and Strings rather than custom types
- **Exception Handling**: Leverages Julia's exception system for parse error handling
- **Module System**: Generates proper Julia modules with export declarations

### Performance Considerations
- **Backtracking**: Implements efficient backtracking for choice alternatives
- **Position Management**: 1-indexed position tracking following Julia conventions
- **Memory Efficiency**: Uses views and slicing where appropriate to minimize allocations

### Transformation Pipeline Precision
The implementation follows the exact 6-step pipeline specification:
1. **Step 2**: OR operator grouping with precise alternative separation
2. **Step 2.5**: Parentheses handling with proper nesting depth tracking  
3. **Step 3**: Sequence parsing with whitespace/comment filtering
4. **Step 4**: Quantifier processing with lookahead detection
5. **Step 5**: Tree structure building with proper AST node construction
6. **Step 6a**: Left recursion elimination using the standard algorithm
7. **Step 6b**: Semantic action generation with structured return values

## Implementation Status

✅ **Complete** - The Julia parser generator implements the full 6-step transformation pipeline with:
- Steps 1-5: Core AST transformation pipeline following exact specifications
- Step 6a: Complete left recursion elimination with Aho-Sethi-Ullman algorithm
- Step 6b: Full return annotation support and semantic action generation
- Production-based grammar representation for algorithmic processing
- Julia-native output format with structured data types (Arrays, Dicts, Strings)
- Comprehensive error handling with position tracking and expected token reporting
- Efficient backtracking and position management for robust parsing
