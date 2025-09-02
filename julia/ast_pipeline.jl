"""
Julia AST Pipeline Implementation

Provides complete EBNF AST transformation pipeline with dual-mode API:
- Same-language optimization: In-memory data structures  
- Cross-language interface: JSON input/output

Implements the 5-stage transformation pipeline equivalent to Perl AST::Transform.
"""

module ASTPipeline

using JSON3
using Dates
using StructTypes

# Configuration for AST transformation pipeline
Base.@kwdef mutable struct PipelineConfig
    debug::Bool = false
    preserve_annotations::Bool = true
    validate_input::Bool = true
    validate_output::Bool = true
    max_recursion_depth::Int = 100
end

# Raw AST types
const Token = Vector{String}
const TokenSequence = Vector{Token}
const RawAST = Vector{TokenSequence}

# Raw AST JSON structure from ebnf_to_json.pl
struct RawASTJson
    grammar_name::String
    raw_ast::RawAST
    metadata::Dict{String, Any}
end
StructTypes.StructType(::Type{RawASTJson}) = StructTypes.Struct()

# AST node types in the transformed AST
abstract type ASTNode end

struct AtomNode <: ASTNode
    type::String
    value::Union{Token, ASTNode}
    
    AtomNode(value) = new("atom", value)
end
StructTypes.StructType(::Type{AtomNode}) = StructTypes.Struct()

struct SequenceNode <: ASTNode
    type::String
    elements::Vector{ASTNode}
    
    SequenceNode(elements) = new("sequence", elements)
end
StructTypes.StructType(::Type{SequenceNode}) = StructTypes.Struct()

struct OrNode <: ASTNode
    type::String
    alternatives::Vector{ASTNode}
    
    OrNode(alternatives) = new("or", alternatives)
end
StructTypes.StructType(::Type{OrNode}) = StructTypes.Struct()

struct QuantifiedNode <: ASTNode
    type::String
    element::ASTNode
    quantifier::String
    
    QuantifiedNode(element, quantifier) = new("quantified", element, quantifier)
end
StructTypes.StructType(::Type{QuantifiedNode}) = StructTypes.Struct()

# Preserved annotations from raw AST
mutable struct Annotations
    semantic_annotations::Dict{String, Vector{String}}
    logging_annotations::Dict{String, Vector{String}}
    return_annotations::Dict{String, String}
    
    Annotations() = new(Dict(), Dict(), Dict())
end
StructTypes.StructType(::Type{Annotations}) = StructTypes.Struct()

# Transformation statistics
mutable struct TransformStats
    rules_processed::Int
    annotations_preserved::Int
    transformations_applied::Int
    
    TransformStats() = new(0, 0, 0)
end
StructTypes.StructType(::Type{TransformStats}) = StructTypes.Struct()

# Transformed AST JSON metadata
struct TransformMetadata
    format::String
    source_format::String
    transformed_at::String
    transformer::String
    pipeline_stage::String
    annotations::Annotations
    stats::TransformStats
end
StructTypes.StructType(::Type{TransformMetadata}) = StructTypes.Struct()

# Transformed AST JSON structure
struct TransformedASTJson
    grammar_name::String
    grammar_tree::Dict{String, ASTNode}
    rule_order::Vector{String}
    metadata::TransformMetadata
end
StructTypes.StructType(::Type{TransformedASTJson}) = StructTypes.Struct()

# Main Julia AST Pipeline implementation
mutable struct JuliaASTPipeline
    config::PipelineConfig
    stats::TransformStats
    annotations::Annotations
    
    JuliaASTPipeline(config::PipelineConfig) = new(config, TransformStats(), Annotations())
end

"""
    load_raw_ast(pipeline::JuliaASTPipeline, file_path::String) -> RawASTJson

Load raw AST JSON from file with validation.
"""
function load_raw_ast(pipeline::JuliaASTPipeline, file_path::String)::RawASTJson
    if pipeline.config.debug
        println("Loading raw AST from: ", file_path)
    end
    
    content = read(file_path, String)
    data = JSON3.read(content, RawASTJson)
    
    if pipeline.config.validate_input
        validate_raw_ast(pipeline, data)
    end
    
    return data
end

"""
    validate_raw_ast(pipeline::JuliaASTPipeline, data::RawASTJson)

Validate raw AST JSON format.
"""
function validate_raw_ast(pipeline::JuliaASTPipeline, data::RawASTJson)
    if isempty(data.grammar_name)
        error("Raw AST JSON missing grammar_name")
    end
    
    if isempty(data.raw_ast)
        error("Raw AST JSON has empty raw_ast array")
    end
    
    if haskey(data.metadata, "format") && data.metadata["format"] != "raw_ast"
        error("metadata.format must be 'raw_ast'")
    end
end

"""
    transform_raw_ast!(pipeline::JuliaASTPipeline, raw_ast::RawAST) -> Tuple{Dict{String, ASTNode}, Vector{String}}

Transform raw AST to semantic AST using the 5-stage pipeline.
"""
function transform_raw_ast!(pipeline::JuliaASTPipeline, raw_ast::RawAST)
    if pipeline.config.debug
        println("=== Julia AST Transformation Pipeline ===")
    end
    
    # Stage 1: Extract annotations
    cleaned_ast = extract_annotations!(pipeline, raw_ast)
    
    # Stage 2: Group by OR operators
    grouped_rules = group_by_or_operators(pipeline, cleaned_ast)
    
    # Stage 2.5: Handle parentheses
    processed_rules = handle_parentheses(pipeline, grouped_rules)
    
    # Stage 3: Parse sequences
    sequenced_rules = parse_sequences(pipeline, processed_rules)
    
    # Stage 4: Handle quantifiers
    quantified_rules = handle_quantifiers(pipeline, sequenced_rules)
    
    # Stage 5: Build tree structure
    grammar_tree, rule_order = build_tree_structure(pipeline, quantified_rules)
    
    pipeline.stats.rules_processed = length(grammar_tree)
    pipeline.stats.transformations_applied = 5
    
    return grammar_tree, rule_order
end

"""
    extract_annotations!(pipeline::JuliaASTPipeline, raw_ast::RawAST) -> RawAST

Stage 1: Extract and preserve annotations from raw AST.
"""
function extract_annotations!(pipeline::JuliaASTPipeline, raw_ast::RawAST)::RawAST
    if pipeline.config.debug
        println("Stage 1: Extracting annotations...")
    end
    
    cleaned_ast = RawAST()
    
    for rule_def in raw_ast
        if isempty(rule_def)
            continue
        end
        
        rule_name = nothing
        cleaned_rule = TokenSequence()
        
        for token in rule_def
            if length(token) != 2
                continue
            end
            
            token_type, token_value = token
            
            if token_type == "rule"
                rule_name = token_value
                push!(cleaned_rule, token)
            elseif token_type in ["semantic_annotation", "logging_annotation"]
                if rule_name !== nothing && pipeline.config.preserve_annotations
                    # Parse annotation format: ["annotation_type", [name, value]] for semantic
                    # or ["annotation_type", [name, [args...]]] for logging
                    try
                        parsed_value = JSON3.read(token_value)
                        if isa(parsed_value, Vector) && length(parsed_value) >= 2
                            annotation_name = string(parsed_value[1])
                            
                            if token_type == "semantic_annotation"
                                if !haskey(pipeline.annotations.semantic_annotations, rule_name)
                                    pipeline.annotations.semantic_annotations[rule_name] = String[]
                                end
                                annotation_value = string(parsed_value[2])
                                formatted_annotation = "$(annotation_name):$(annotation_value)"
                                push!(pipeline.annotations.semantic_annotations[rule_name], formatted_annotation)
                                
                            elseif token_type == "logging_annotation"
                                if !haskey(pipeline.annotations.logging_annotations, rule_name)
                                    pipeline.annotations.logging_annotations[rule_name] = String[]
                                end
                                args = if isa(parsed_value[2], Vector)
                                    join([string(arg) for arg in parsed_value[2]], ",")
                                else
                                    string(parsed_value[2])
                                end
                                formatted_annotation = "$(annotation_name)($(args))"
                                push!(pipeline.annotations.logging_annotations[rule_name], formatted_annotation)
                            end
                        else
                            # Fallback for malformed annotation data
                            if token_type == "semantic_annotation"
                                if !haskey(pipeline.annotations.semantic_annotations, rule_name)
                                    pipeline.annotations.semantic_annotations[rule_name] = String[]
                                end
                                push!(pipeline.annotations.semantic_annotations[rule_name], "raw:$(token_value)")
                            elseif token_type == "logging_annotation"
                                if !haskey(pipeline.annotations.logging_annotations, rule_name)
                                    pipeline.annotations.logging_annotations[rule_name] = String[]
                                end
                                push!(pipeline.annotations.logging_annotations[rule_name], "raw:$(token_value)")
                            end
                        end
                    catch e
                        # Fallback for JSON parsing errors
                        if token_type == "semantic_annotation"
                            if !haskey(pipeline.annotations.semantic_annotations, rule_name)
                                pipeline.annotations.semantic_annotations[rule_name] = String[]
                            end
                            push!(pipeline.annotations.semantic_annotations[rule_name], "raw:$(token_value)")
                        elseif token_type == "logging_annotation"
                            if !haskey(pipeline.annotations.logging_annotations, rule_name)
                                pipeline.annotations.logging_annotations[rule_name] = String[]
                            end
                            push!(pipeline.annotations.logging_annotations[rule_name], "raw:$(token_value)")
                        end
                    end
                    pipeline.stats.annotations_preserved += 1
                end
                # Don't add to cleaned rule
            elseif token_type in ["return_scalar", "return_array", "return_object"]
                if rule_name !== nothing && pipeline.config.preserve_annotations
                    pipeline.annotations.return_annotations[rule_name] = token_type
                end
                # Don't add to cleaned rule  
            else
                push!(cleaned_rule, token)
            end
        end
        
        if !isempty(cleaned_rule)
            push!(cleaned_ast, cleaned_rule)
        end
    end
    
    if pipeline.config.debug
        println("Preserved ", pipeline.stats.annotations_preserved, " annotations")
    end
    
    return cleaned_ast
end

"""
    group_by_or_operators(pipeline::JuliaASTPipeline, ast::RawAST) -> Dict{String, Vector{TokenSequence}}

Stage 2: Group rule definitions by OR operators.
"""
function group_by_or_operators(pipeline::JuliaASTPipeline, ast::RawAST)
    if pipeline.config.debug
        println("Stage 2: Grouping by OR operators...")
    end
    
    grouped = Dict{String, Vector{TokenSequence}}()
    
    for rule_def in ast
        if isempty(rule_def)
            continue
        end
        
        rule_name = nothing
        for token in rule_def
            if length(token) == 2 && token[1] == "rule"
                rule_name = token[2]
                break
            end
        end
        
        if rule_name !== nothing
            alternatives = TokenSequence[]
            current_alt = Token[]
            
            # Skip rule definition token
            for token in rule_def[2:end]
                if length(token) == 2 && token[1] == "operator" && token[2] == "|"
                    if !isempty(current_alt)
                        push!(alternatives, current_alt)
                        current_alt = Token[]
                    end
                else
                    push!(current_alt, token)
                end
            end
            
            if !isempty(current_alt)
                push!(alternatives, current_alt)
            end
            
            if !haskey(grouped, rule_name)
                grouped[rule_name] = TokenSequence[]
            end
            append!(grouped[rule_name], alternatives)
        end
    end
    
    return grouped
end

"""
    handle_parentheses(pipeline::JuliaASTPipeline, grouped_rules::Dict{String, Vector{TokenSequence}}) -> Dict{String, Vector{TokenSequence}}

Stage 2.5: Handle parentheses and grouping.
"""
function handle_parentheses(pipeline::JuliaASTPipeline, grouped_rules::Dict{String, Vector{TokenSequence}})
    if pipeline.config.debug
        println("Stage 2.5: Handling parentheses...")
    end
    
    processed = Dict{String, Vector{TokenSequence}}()
    
    for (rule_name, alternatives) in grouped_rules
        processed_alts = TokenSequence[]
        
        for alt in alternatives
            processed_alt = process_parentheses_in_sequence(alt)
            push!(processed_alts, processed_alt)
        end
        
        processed[rule_name] = processed_alts
    end
    
    return processed
end

"""
    process_parentheses_in_sequence(sequence::TokenSequence) -> TokenSequence

Process parentheses within a token sequence.
"""
function process_parentheses_in_sequence(sequence::TokenSequence)::TokenSequence
    result = TokenSequence()
    i = 1
    
    while i <= length(sequence)
        token = sequence[i]
        
        if length(token) == 2 && token[1] == "group_open"
            # Find matching close
            paren_count = 1
            j = i + 1
            group_content = TokenSequence()
            
            while j <= length(sequence) && paren_count > 0
                if length(sequence[j]) == 2
                    if sequence[j][1] == "group_open"
                        paren_count += 1
                    elseif sequence[j][1] == "group_close"
                        paren_count -= 1
                    end
                end
                
                if paren_count > 0
                    push!(group_content, sequence[j])
                end
                j += 1
            end
            
            if !isempty(group_content)
                # Create group token - serialize content as JSON
                content_json = JSON3.write(group_content)
                push!(result, ["group", content_json])
            end
            
            i = j
        else
            push!(result, token)
            i += 1
        end
    end
    
    return result
end

"""
    parse_sequences(pipeline::JuliaASTPipeline, processed_rules::Dict{String, Vector{TokenSequence}}) -> Dict{String, Vector{ASTNode}}

Stage 3: Parse sequences of grammar elements.
"""
function parse_sequences(pipeline::JuliaASTPipeline, processed_rules::Dict{String, Vector{TokenSequence}})
    if pipeline.config.debug
        println("Stage 3: Parsing sequences...")
    end
    
    sequenced = Dict{String, Vector{ASTNode}}()
    
    for (rule_name, alternatives) in processed_rules
        parsed_alts = ASTNode[]
        
        for alt in alternatives
            parsed_alt = if length(alt) == 1
                parse_single_element(alt[1])
            else
                elements = [parse_single_element(elem) for elem in alt]
                SequenceNode(elements)
            end
            push!(parsed_alts, parsed_alt)
        end
        
        sequenced[rule_name] = parsed_alts
    end
    
    return sequenced
end

"""
    parse_single_element(element::Token) -> ASTNode

Parse a single grammar element.
"""
function parse_single_element(element::Token)::ASTNode
    if length(element) != 2
        return AtomNode(element)
    end
    
    token_type, token_value = element
    
    if token_type == "group"
        # Deserialize group content
        group_content = JSON3.read(token_value, TokenSequence)
        
        if length(group_content) == 1
            return parse_single_element(group_content[1])
        else
            elements = [parse_single_element(elem) for elem in group_content]
            return SequenceNode(elements)
        end
    else
        return AtomNode(element)
    end
end

"""
    handle_quantifiers(pipeline::JuliaASTPipeline, sequenced_rules::Dict{String, Vector{ASTNode}}) -> Dict{String, Vector{ASTNode}}

Stage 4: Handle quantifiers (*, +, ?).
"""
function handle_quantifiers(pipeline::JuliaASTPipeline, sequenced_rules::Dict{String, Vector{ASTNode}})
    if pipeline.config.debug
        println("Stage 4: Handling quantifiers...")
    end
    
    quantified = Dict{String, Vector{ASTNode}}()
    
    for (rule_name, alternatives) in sequenced_rules
        processed_alts = ASTNode[]
        
        for alt in alternatives
            processed_alt = apply_quantifiers_to_node(alt)
            push!(processed_alts, processed_alt)
        end
        
        quantified[rule_name] = processed_alts
    end
    
    return quantified
end

"""
    apply_quantifiers_to_node(node::ASTNode) -> ASTNode

Apply quantifiers to AST node.
"""
function apply_quantifiers_to_node(node::ASTNode)::ASTNode
    if isa(node, SequenceNode)
        new_elements = ASTNode[]
        i = 1
        
        while i <= length(node.elements)
            element = node.elements[i]
            
            # Check if next element is a quantifier
            if i + 1 <= length(node.elements)
                next_elem = node.elements[i + 1]
                if isa(next_elem, AtomNode) && isa(next_elem.value, Token)
                    token = next_elem.value
                    if length(token) == 2 && token[1] == "operator" && token[2] in ["*", "+", "?"]
                        quantifier = token[2]
                        quantified_node = QuantifiedNode(element, quantifier)
                        push!(new_elements, quantified_node)
                        i += 2  # Skip quantifier token
                        continue
                    end
                end
            end
            
            push!(new_elements, element)
            i += 1
        end
        
        return SequenceNode(new_elements)
    else
        return node
    end
end

"""
    build_tree_structure(pipeline::JuliaASTPipeline, quantified_rules::Dict{String, Vector{ASTNode}}) -> Tuple{Dict{String, ASTNode}, Vector{String}}

Stage 5: Build final tree structure.
"""
function build_tree_structure(pipeline::JuliaASTPipeline, quantified_rules::Dict{String, Vector{ASTNode}})
    if pipeline.config.debug
        println("Stage 5: Building tree structure...")
    end
    
    grammar_tree = Dict{String, ASTNode}()
    rule_order = collect(keys(quantified_rules))
    
    for (rule_name, alternatives) in quantified_rules
        final_node = if length(alternatives) == 1
            alternatives[1]
        else
            OrNode(alternatives)
        end
        
        grammar_tree[rule_name] = final_node
    end
    
    return grammar_tree, rule_order
end

"""
    save_transformed_ast(pipeline::JuliaASTPipeline, grammar_tree::Dict{String, ASTNode}, rule_order::Vector{String}, grammar_name::String, output_file::String)

Save transformed AST to JSON file.
"""
function save_transformed_ast(pipeline::JuliaASTPipeline, grammar_tree::Dict{String, ASTNode}, rule_order::Vector{String}, grammar_name::String, output_file::String)
    if pipeline.config.debug
        println("Saving transformed AST to: ", output_file)
    end
    
    metadata = TransformMetadata(
        "transformed_ast",
        "raw_ast", 
        Dates.format(Dates.now(Dates.UTC), "yyyy-mm-ddTHH:MM:SS.sssZ"),
        "Julia AST Pipeline v1.0",
        "transformation",
        pipeline.annotations,
        pipeline.stats
    )
    
    transformed_data = TransformedASTJson(
        grammar_name,
        grammar_tree,
        rule_order,
        metadata
    )
    
    json_str = JSON3.write(transformed_data)
    
    open(output_file, "w") do f
        write(f, json_str)
    end
    
    if pipeline.config.debug
        println("Transformed AST saved successfully")
    end
end

"""
    transform_from_file!(pipeline::JuliaASTPipeline, raw_ast_json_file::String, output_json_file::Union{String, Nothing}=nothing) -> Tuple{Dict{String, ASTNode}, Vector{String}}

Same-language API: Transform raw AST JSON file to in-memory AST with optional JSON output.
"""
function transform_from_file!(pipeline::JuliaASTPipeline, raw_ast_json_file::String, output_json_file::Union{String, Nothing}=nothing)
    raw_data = load_raw_ast(pipeline, raw_ast_json_file)
    grammar_tree, rule_order = transform_raw_ast!(pipeline, raw_data.raw_ast)
    
    if output_json_file !== nothing
        save_transformed_ast(pipeline, grammar_tree, rule_order, raw_data.grammar_name, output_json_file)
    end
    
    return grammar_tree, rule_order
end

"""
    transform_to_json!(pipeline::JuliaASTPipeline, raw_ast_json_file::String, output_json_file::String)

Cross-language API: Transform raw AST JSON file to transformed AST JSON file.
"""
function transform_to_json!(pipeline::JuliaASTPipeline, raw_ast_json_file::String, output_json_file::String)
    grammar_tree, rule_order = transform_from_file!(pipeline, raw_ast_json_file, nothing)
    raw_data = load_raw_ast(pipeline, raw_ast_json_file)
    save_transformed_ast(pipeline, grammar_tree, rule_order, raw_data.grammar_name, output_json_file)
end

end # module
