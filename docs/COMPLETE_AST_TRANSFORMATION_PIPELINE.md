# Complete AST Transformation Pipeline Documentation

**Version:** 3.0 - January 2025  
**Target Audience:** Multi-language parser generator implementers  
**Architecture:** JSON-based with language-independent transformation pipeline

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [JSON Input Format](#json-input-format)
3. [Annotation Systems](#annotation-systems)
4. [5-Step Transformation Pipeline](#5-step-transformation-pipeline)
5. [Left-Recursion Elimination](#left-recursion-elimination)
6. [CodeGenerator Framework](#codegenerator-framework)
7. [DataGenerator Framework](#datagenerator-framework)
8. [Implementation Guidelines](#implementation-guidelines)
9. [Complete Examples](#complete-examples)
10. [Language-Specific Optimizations](#language-specific-optimizations)

---

## Architecture Overview

The modern EBNF parser generator follows a clean 3-stage architecture optimized for multi-language support:

```
┌─────────────────┐    ┌──────────────────────┐    ┌─────────────────────┐
│   EBNF Grammar  │───▶│  ebnf_to_json.pl     │───▶│   Raw AST JSON      │
│   (.ebnf files) │    │  (Universal Parser)  │    │   (Language Agnostic)│
└─────────────────┘    └──────────────────────┘    └─────────────────────┘
                                                              │
                                                              ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                      Language-Specific Generators                           │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐              │
│  │  perl_parser_gen│  │ rust_parser_gen │  │julia_parser_gen │  ... more    │
│  │                 │  │                 │  │                 │              │
│  │ 1. JSON Input   │  │ 1. JSON Input   │  │ 1. JSON Input   │              │
│  │ 2. Transform    │  │ 2. Transform    │  │ 2. Transform    │              │
│  │ 3. CodeGen      │  │ 3. CodeGen      │  │ 3. CodeGen      │              │
│  │ 4. DataGen      │  │ 4. DataGen      │  │ 4. DataGen      │              │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘              │
└─────────────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
              ┌─────────────────────────────────────┐
              │       Generated Parsers             │
              │  (.pm, .rs, .jl, .go, .zig, .ts)   │
              └─────────────────────────────────────┘
```

### Key Design Principles

1. **Universal EBNF Parsing**: Single, stable Perl-based EBNF parser for all languages
2. **Language-Native Optimization**: Each generator optimizes transformation for its language
3. **Clean Separation**: EBNF parsing vs AST transformation vs code generation
4. **Extensibility**: Easy to add new target languages
5. **Feature Completeness**: Full annotation support, left-recursion elimination, multi-output

---

## JSON Input Format

The `ebnf_to_json.pl` tool outputs standardized JSON containing the raw AST exactly as parsed from EBNF.

### JSON Structure

```json
{
    "grammar_name": "json_parser",
    "metadata": {
        "source_file": "json.ebnf",
        "generated_at": "2025-01-02T00:49:18Z", 
        "format": "raw_ast",
        "description": "Direct output from EBNF parser before transformations",
        "ebnf_version": "1.0",
        "generator": "ebnf_to_json.pl"
    },
    "raw_ast": [
        // Array of rule definitions - see below
    ]
}
```

### Raw AST Rule Format

Each rule in `raw_ast` is an array of tokens representing the rule definition:

```json
[
    ["rule", "expression"],
    ["semantic_annotation", ["type", "\"Expression\""]],
    ["semantic_annotation", ["complexity", "\"O(1)\""]],
    ["logging_annotation", ["debug", "\"parsing expression\""]],
    ["rule_reference", "term"],
    ["group_open", "("],
    ["quoted_string", "+"],
    ["operator", "|"],
    ["quoted_string", "-"],
    ["group_close", ")"],
    ["rule_reference", "term"],
    ["operator", "*"],
    ["return_annotation", "-> {op: $2, left: $1, right: $3}"]
]
```

### Token Types

| Token Type | Description | Example |
|------------|-------------|---------|
| `"rule"` | Rule name definition | `["rule", "expression"]` |
| `"rule_reference"` | Reference to another rule | `["rule_reference", "term"]` |
| `"quoted_string"` | Terminal string literal | `["quoted_string", "+"]` |
| `"regex"` | Regular expression pattern | `["regex", "(\\d+)"]` |
| `"operator"` | Grammar operators | `["operator", "|"]` |
| `"quantifier"` | Quantifiers | `["quantifier", "*"]` |
| `"group_open"` | Opening parenthesis | `["group_open", "("]` |
| `"group_close"` | Closing parenthesis | `["group_close", ")"]` |
| `"semantic_annotation"` | Semantic metadata | `["semantic_annotation", ["type", "Expression"]]` |
| `"logging_annotation"` | Runtime logging directives | `["logging_annotation", ["debug", "parsing..."]]` |
| `"return_annotation"` | AST construction rules | `["return_annotation", "-> $1"]` |

---

## Annotation Systems

The parser generator supports three distinct annotation types, each serving different purposes in the pipeline.

### 1. Semantic Annotations (Static Metadata)

**Purpose**: Describe invariant properties of grammar rules for analysis and data generation.

**Format**: `@annotation_name: value`

**Examples**:
```ebnf
@type: "Expression"
@range: {min: 0, max: 1000}
@validation: {regex: "^[\\w.-]+@[\\w.-]+\\.[a-zA-Z]{2,}$"}
@examples: [42, 123, 999]
expression := term ("+" term)*
```

**JSON Representation**:
```json
["semantic_annotation", ["type", "\"Expression\""]]
["semantic_annotation", ["range", "{min: 0, max: 1000}"]]
```

**Processing**: 
- ✅ **Filtered out** during grammar transformation  
- ✅ **Preserved as metadata** in final AST nodes
- ✅ **Available to DataGenerator** for constrained generation

**Implementation Pattern**:
```python
def is_semantic_annotation(token):
    return (isinstance(token, list) and len(token) >= 2 and 
            token[0] == "semantic_annotation")

def extract_semantic_annotations(elements):
    """Separate semantic annotations from grammar elements"""
    annotations = []
    filtered_elements = []
    
    for element in elements:
        if is_semantic_annotation(element):
            annotations.append(element)
        else:
            filtered_elements.append(element)
    
    return annotations, filtered_elements
```

### 2. Logging Annotations (Dynamic Metadata)

**Purpose**: Generate runtime logging and debugging code in parsers.

**Format**: `@log: message`, `@debug: expression`, `@trace: context`

**Examples**:
```ebnf
@debug: "Parsing expression at position {pos}"
@performance: "Parse time: {end_time - start_time}ms"
@trace: "Input: {current_input}"
expression := term ("+" term)*
```

**JSON Representation**:
```json
["logging_annotation", ["debug", "\"Parsing expression at position {pos}\""]]
```

**Processing**:
- ✅ **Filtered out** during grammar transformation
- ✅ **Converted to code** during parser generation
- ✅ **Runtime behavior** - different output per parse

**Implementation Pattern**:
```python
def generate_logging_code(logging_annotations, rule_name):
    """Convert logging annotations to executable code"""
    code_lines = []
    
    for annotation in logging_annotations:
        log_type, message = annotation[1][0], annotation[1][1]
        
        if log_type == "debug":
            code_lines.append(f'log_debug("{rule_name}: {message}");')
        elif log_type == "performance":
            code_lines.append(f'start_timer(); /* {message} */')
        
    return code_lines
```

### 3. Return Annotations (AST Construction)

**Purpose**: Define how to construct AST nodes from parsed elements.

**Format**: `-> expression`

**Examples**:
```ebnf
expression := term ("+" term)* -> {op: $2, left: $1, right: $3}
number := /(\d+)/ -> $1
array := "[" elements "]" -> [$2]
```

**JSON Representation**:
```json
["return_annotation", "-> {op: $2, left: $1, right: $3}"]
```

**Processing**:
- ✅ **Filtered out** during grammar transformation
- ✅ **Converted to code** during parser generation  
- ✅ **Runtime behavior** - different AST per parse

**Implementation Pattern**:
```python
def generate_return_code(return_annotation, results_var="results"):
    """Convert return annotation to AST construction code"""
    expr = return_annotation[1].replace("-> ", "")
    
    if expr.startswith("$"):
        # Simple scalar reference: $1 -> results[0]
        index = int(expr[1:]) - 1
        return f"return {results_var}[{index}];"
    elif expr.startswith("["):
        # Array construction: [$1, $2] -> [results[0], results[1]]
        return f"return [{expand_references(expr, results_var)}];"
    elif expr.startswith("{"):
        # Object construction: {key: $1} -> {"key": results[0]}
        return f"return {{{expand_object_refs(expr, results_var)}}};"
```

### Annotation Processing Summary

| Annotation Type | Static/Dynamic | Pipeline Action | Final Destination |
|------------------|----------------|-----------------|-------------------|
| **Semantic** | Static | Filter + Preserve | AST Metadata → DataGenerator |
| **Logging** | Dynamic | Filter + Generate | Parser Code → Runtime |
| **Return** | Dynamic | Filter + Generate | Parser Code → AST Construction |

---

## 5-Step Transformation Pipeline

Each language generator must implement these exact transformation steps to ensure consistency.

### Step 1: JSON Input Processing

**Purpose**: Load and validate raw AST JSON data.

```python
def load_raw_ast(json_content):
    """Load and validate JSON input"""
    data = json.loads(json_content)
    
    # Validate structure
    if not data.get("grammar_name") or not data.get("raw_ast"):
        raise ValueError("Invalid JSON: missing grammar_name or raw_ast")
    
    return data["grammar_name"], data["raw_ast"]
```

### Step 2: Group by OR Operators

**Purpose**: Split rule alternatives separated by `|` operators into separate branches.

**Algorithm**:
```python
def step2_group_by_or(raw_ast):
    """Group rule alternatives by OR operators"""
    transformed_rules = []
    
    for rule_tokens in raw_ast:
        rule_name = extract_rule_name(rule_tokens)  # First token: ["rule", "name"]
        definition_tokens = rule_tokens[1:]  # Rest are definition
        
        # Split on OR operators
        or_groups = []
        current_group = []
        
        for token in definition_tokens:
            if is_or_operator(token):  # ["operator", "|"]
                if current_group:
                    or_groups.append(current_group)
                    current_group = []
            else:
                current_group.append(token)
        
        # Add final group
        if current_group:
            or_groups.append(current_group)
        
        transformed_rules.append({
            "name": rule_name,
            "or_groups": or_groups
        })
    
    return transformed_rules
```

### Step 2.5: Handle Parentheses

**Purpose**: Group tokens within parentheses and resolve nested structures.

**Algorithm**:
```python
def step2_5_handle_parentheses(step2_result):
    """Process parentheses grouping within each OR alternative"""
    transformed_rules = []
    
    for rule in step2_result:
        processed_or_groups = []
        
        for or_group in rule["or_groups"]:
            processed_group = process_parentheses_in_sequence(or_group)
            processed_or_groups.append(processed_group)
        
        transformed_rules.append({
            "name": rule["name"],
            "or_groups": processed_or_groups
        })
    
    return transformed_rules

def process_parentheses_in_sequence(tokens):
    """Handle parentheses within a token sequence"""
    result = []
    i = 0
    
    while i < len(tokens):
        if is_group_open(tokens[i]):  # ["group_open", "("]
            # Find matching closing parenthesis
            group_tokens, end_pos = extract_group_content(tokens, i)
            
            # Recursively process OR alternatives within group
            group_alternatives = group_by_or_within_group(group_tokens)
            
            result.append(["GROUPED", group_alternatives])
            i = end_pos + 1
        else:
            result.append(tokens[i])
            i += 1
    
    return result
```

### Step 3: Parse Sequences

**Purpose**: Identify ordered sequences of symbols within alternatives.

**Algorithm**:
```python
def step3_parse_sequences(step2_5_result):
    """Convert token groups into sequence structures"""
    transformed_rules = []
    
    for rule in step2_5_result:
        parsed_alternatives = []
        
        for or_group in rule["or_groups"]:
            if len(or_group) == 0:
                continue  # Skip empty alternatives
            elif len(or_group) == 1:
                # Single element
                parsed_alternatives.append({
                    "type": "atom",
                    "value": or_group[0]
                })
            else:
                # Multi-element sequence
                parsed_alternatives.append({
                    "type": "sequence", 
                    "elements": or_group
                })
        
        # Build rule structure based on alternatives
        if len(parsed_alternatives) == 1:
            parsed_rule = {
                "name": rule["name"],
                **parsed_alternatives[0]  # Inherit type and content
            }
        else:
            parsed_rule = {
                "name": rule["name"],
                "type": "or",
                "alternatives": parsed_alternatives
            }
        
        transformed_rules.append(parsed_rule)
    
    return transformed_rules
```

### Step 4: Handle Quantifiers

**Purpose**: Process `?` (optional), `*` (zero-or-more), `+` (one-or-more) quantifiers.

**Algorithm**:
```python
def step4_handle_quantifiers(step3_result):
    """Attach quantifiers to preceding symbols"""
    transformed_rules = []
    
    for rule in step3_result:
        if rule["type"] == "sequence":
            rule["elements"] = process_quantifiers_in_sequence(rule["elements"])
        elif rule["type"] == "or":
            for alternative in rule["alternatives"]:
                if alternative["type"] == "sequence":
                    alternative["elements"] = process_quantifiers_in_sequence(
                        alternative["elements"]
                    )
        
        transformed_rules.append(rule)
    
    return transformed_rules

def process_quantifiers_in_sequence(elements):
    """Apply quantifiers to preceding elements in sequence"""
    result = []
    i = 0
    
    while i < len(elements):
        element = elements[i]
        
        # Look ahead for quantifier
        if (i + 1 < len(elements) and 
            is_quantifier_token(elements[i + 1])):
            
            quantifier = elements[i + 1][1]  # Extract quantifier symbol
            
            result.append({
                "type": "quantified",
                "element": element,
                "quantifier": quantifier
            })
            i += 2  # Skip both element and quantifier
        else:
            result.append(element)
            i += 1
    
    return result
```

### Step 5: Build Tree Structure

**Purpose**: Create final semantic AST tree with annotation preservation.

**Algorithm**:
```python
def step5_build_tree_structure(step4_result):
    """Build semantic tree and preserve annotations"""
    grammar_tree = {}
    rule_order = []
    
    for rule in step4_result:
        rule_name = rule["name"]
        rule_order.append(rule_name)
        
        # Process annotations BEFORE building tree
        semantic_annotations = extract_semantic_annotations_from_rule(rule)
        logging_annotations = extract_logging_annotations_from_rule(rule)
        return_annotations = extract_return_annotations_from_rule(rule)
        
        # Build tree structure
        tree_node = build_rule_tree(rule)
        
        # Attach annotations as metadata
        if semantic_annotations:
            tree_node["semantic_annotations"] = semantic_annotations
        if logging_annotations:
            tree_node["logging_annotations"] = logging_annotations
        if return_annotations:
            tree_node["return_annotations"] = return_annotations
        
        grammar_tree[rule_name] = tree_node
    
    return grammar_tree, rule_order
```

---

## Left-Recursion Elimination

Advanced left-recursion elimination with AST structure preservation has been significantly enhanced.

### Problem

Traditional left-recursive rules like:
```ebnf
expression := expression "+" term | term
```

Cause infinite recursion in top-down parsers.

### Solution Overview

The system uses advanced **serialization/deserialization** to preserve complex AST structures during elimination:

```
Original Rule → Serialize Complex Structures → Eliminate Recursion → Deserialize → Final Rule
```

### Key Improvements

#### 1. Complex Structure Preservation

**Problem**: Previous system lost complex quantified structures:
```
Input:  expression := expression ( "," expr )*
Output: expression := expr QUANTIFIED:HASH(0x1234):*  // BROKEN!
```

**Solution**: Enhanced serialization format:
```
Input:  expression := expression ( "," expr )*  
Output: expression := expr QUANTIFIED:SEQUENCE~TERMINAL:,||expr~*  // PRESERVED!
```

#### 2. Serialization Formats

| Structure Type | Serialization Format | Example |
|---------------|---------------------|---------|
| **Simple Quantifier** | `QUANTIFIED:element:quantifier` | `QUANTIFIED:term:*` |
| **Grouped Sequence** | `QUANTIFIED:SEQUENCE~elem1\|\|elem2~quant` | `QUANTIFIED:SEQUENCE~TERMINAL:,\|\|expr~*` |
| **Grouped Choice** | `QUANTIFIED:GROUP~alt1\|\|alt2~quant` | `QUANTIFIED:GROUP~term\|\|factor~+` |

#### 3. Deserialization Logic

```python
def deserialize_quantified_element(serialized_string):
    """Reconstruct complex quantified structures"""
    if serialized_string.startswith("QUANTIFIED:SEQUENCE~"):
        # Extract: QUANTIFIED:SEQUENCE~elem1||elem2||elem3~*
        _, content, quantifier = serialized_string.split("~")
        elements = content.split("||")
        
        sequence_elements = []
        for elem in elements:
            if elem.startswith("TERMINAL:"):
                sequence_elements.append(["quoted_string", elem[9:]])
            elif elem.startswith("REGEX:"):
                sequence_elements.append(["regex", elem[6:]])
            else:
                sequence_elements.append(["rule_reference", elem])
        
        return {
            "type": "quantified",
            "element": {
                "type": "sequence", 
                "elements": sequence_elements
            },
            "quantifier": quantifier
        }
    
    # Handle other formats...
```

### Implementation Requirements

```python
def eliminate_left_recursion(grammar_tree, rule_order):
    """Enhanced left-recursion elimination with structure preservation"""
    
    # Step 1: Detect left-recursive rules
    left_recursive_rules = find_left_recursive_rules(grammar_tree)
    
    # Step 2: Serialize complex structures
    serialized_grammar = serialize_complex_structures(grammar_tree)
    
    # Step 3: Apply elimination algorithm
    eliminated_grammar = apply_elimination_algorithm(serialized_grammar)
    
    # Step 4: Deserialize structures
    final_grammar = deserialize_complex_structures(eliminated_grammar)
    
    return final_grammar, rule_order
```

---

## CodeGenerator Framework

The CodeGenerator is responsible for converting the transformed AST into executable parser code in the target language.

### CodeGenerator Interface

Each language must implement this interface:

```python
class CodeGenerator:
    def generate_parser(self, grammar_tree, rule_order, options):
        """Main entry point for parser generation"""
        pass
    
    def generate_rule_function(self, rule_name, rule_definition):
        """Generate parsing function for single rule"""
        pass
    
    def generate_terminal_matcher(self, terminal_value):
        """Generate code to match terminal strings"""
        pass
    
    def generate_regex_matcher(self, regex_pattern):
        """Generate code to match regex patterns"""
        pass
    
    def generate_quantifier_handler(self, quantified_element):
        """Generate code for *, +, ? quantifiers"""
        pass
    
    def generate_return_annotation_code(self, return_annotation):
        """Generate AST construction code from return annotations"""
        pass
    
    def generate_logging_code(self, logging_annotations):
        """Generate runtime logging code from logging annotations"""
        pass
```

### Language-Specific Examples

#### Rust CodeGenerator
```rust
impl CodeGenerator for RustCodeGenerator {
    fn generate_quantifier_handler(&self, element: &QuantifiedElement) -> String {
        match element.quantifier {
            Quantifier::ZeroOrMore => format!(
                "let mut results = Vec::new();\n\
                 while let Some(result) = {}(input)? {{\n\
                     results.push(result);\n\
                 }}\n\
                 Ok(Some(results))",
                self.generate_element_parser(&element.element)
            ),
            // Handle other quantifiers...
        }
    }
}
```

#### Julia CodeGenerator
```julia
function generate_quantifier_handler(element::QuantifiedElement)
    if element.quantifier == ZeroOrMore
        return """
        results = Vector{ASTNode}()
        while (result = $(generate_element_parser(element.element))(input)) !== nothing
            push!(results, result)
        end
        return results
        """
    end
    # Handle other quantifiers...
end
```

### Output Formats

Each CodeGenerator produces complete, executable parsers:

| Language | Output Files | Entry Point |
|----------|--------------|-------------|
| **Perl** | `.pm` module + `.pl` wrapper | `parse($input)` |
| **Rust** | `.rs` source + `Cargo.toml` | `pub fn parse(input: &str)` |
| **Julia** | `.jl` module | `parse(input::String)` |
| **Go** | `.go` package | `func Parse(input string)` |
| **Zig** | `.zig` module | `pub fn parse(input: []const u8)` |
| **TypeScript** | `.ts` module + `.d.ts` | `export function parse(input: string)` |

---

## DataGenerator Framework

The DataGenerator creates test data conforming to the grammar using semantic annotations as constraints.

### DataGenerator Interface

```python
class DataGenerator:
    def generate_data(self, grammar_tree, rule_name, constraints=None):
        """Generate data for specific grammar rule"""
        pass
    
    def apply_semantic_constraints(self, rule_def, generated_data):
        """Apply semantic annotation constraints to generated data"""
        pass
    
    def generate_terminal_data(self, terminal_rule, semantic_annotations):
        """Generate data for terminal rules using semantic hints"""
        pass
    
    def generate_structured_data(self, rule_def, depth_limit=10):
        """Generate complex structured data with depth limits"""
        pass
```

### Domain-Specific Implementation

**Important**: DataGenerator implementation is **domain-specific** and cannot be generic.

#### Why Domain-Specific?

Each problem domain has its own semantic vocabulary:

**SystemVerilog Example**:
```ebnf
@bit_width: 32
@signed: true  
@clock_domain: "clk_100mhz"
@drive_strength: "strong"
signal_declaration := ...
```

**VHDL Example**:
```ebnf
@signal_type: "std_logic_vector"
@range_direction: "downto"
@timing_constraint: "setup 2ns"
signal_declaration := ...
```

**JSON Example**:
```ebnf
@type: "string"
@format: "email"
@examples: ["user@example.com", "test@domain.org"]
email_field := ...
```

#### Implementation Strategy

1. **Phase 1**: Create HDL-specific EBNF grammars with appropriate semantic annotations
2. **Phase 2**: Build domain-specific DataGenerators that understand the semantic vocabulary
3. **Phase 3**: Generate meaningful test data using domain knowledge

### Generic DataGenerator Pattern

```python
# This pattern works for any domain once semantic vocabulary is defined
def generate_constrained_data(rule_definition, semantic_annotations):
    """Generic pattern for applying semantic constraints"""
    
    # Extract constraint information
    constraints = {}
    for annotation in semantic_annotations:
        name, value = annotation["value"][1]
        constraints[name] = parse_constraint_value(value)
    
    # Generate base data from grammar structure
    base_data = generate_from_grammar_structure(rule_definition)
    
    # Apply domain-specific constraints
    constrained_data = apply_domain_constraints(base_data, constraints)
    
    return constrained_data
```

---

## Implementation Guidelines

### For Language Implementers

#### 1. Project Structure
```
your_language_parser_gen/
├── src/
│   ├── main.{ext}              # Command-line interface
│   ├── ast_transform.{ext}     # 5-step transformation pipeline  
│   ├── code_generator.{ext}    # Parser code generation
│   ├── data_generator.{ext}    # Test data generation (domain-specific)
│   └── types.{ext}            # AST node type definitions
├── tests/
│   ├── test_pipeline.{ext}     # Test transformation steps
│   └── test_grammars/          # Test EBNF files
└── examples/
    └── json_parser/            # Complete example
```

#### 2. Development Phases

**Phase 1: Core Pipeline** (Required)
- ✅ JSON input parsing
- ✅ 5-step transformation pipeline
- ✅ Annotation filtering and preservation
- ✅ Basic error handling

**Phase 2: Code Generation** (Required)
- ✅ Rule function generation
- ✅ Terminal/regex matching
- ✅ Quantifier handling
- ✅ Return annotation processing
- ✅ Logging annotation processing

**Phase 3: Advanced Features** (Optional)
- ⏸️ Left-recursion elimination
- ⏸️ Domain-specific DataGenerator
- ⏸️ Performance optimizations

#### 3. Testing Strategy

**Unit Tests**: Test each pipeline step independently
```python
def test_step2_group_by_or():
    input_tokens = [["rule", "expr"], ["identifier", "a"], ["operator", "|"], ["identifier", "b"]]
    result = step2_group_by_or([input_tokens])
    
    assert result[0]["name"] == "expr"
    assert len(result[0]["or_groups"]) == 2
    assert result[0]["or_groups"][0] == [["identifier", "a"]]
    assert result[0]["or_groups"][1] == [["identifier", "b"]]
```

**Integration Tests**: Test complete pipeline
```python
def test_complete_pipeline():
    json_data = load_test_grammar("simple_arithmetic.json")
    grammar_tree, rule_order = transform_complete_pipeline(json_data["raw_ast"])
    
    assert "expression" in grammar_tree
    assert grammar_tree["expression"]["type"] in ["sequence", "or", "atom"]
```

**Generated Parser Tests**: Verify generated parsers work
```python
def test_generated_parser():
    # Generate parser from test grammar
    parser_code = generate_parser(test_grammar)
    
    # Compile/import generated parser
    parser = compile_parser(parser_code)
    
    # Test parsing valid input
    result = parser.parse("2 + 3 * 4")
    assert result is not None
    
    # Test parsing invalid input
    result = parser.parse("2 + + 3")
    assert result is None
```

#### 4. Error Handling Best Practices

```python
class TransformationError(Exception):
    def __init__(self, message, rule_name=None, step=None, token_pos=None):
        self.message = message
        self.rule_name = rule_name
        self.step = step
        self.token_pos = token_pos
        super().__init__(self.format_message())
    
    def format_message(self):
        context = []
        if self.rule_name:
            context.append(f"rule '{self.rule_name}'")
        if self.step:
            context.append(f"step {self.step}")
        if self.token_pos is not None:
            context.append(f"position {self.token_pos}")
        
        context_str = " in " + ", ".join(context) if context else ""
        return f"Transformation error: {self.message}{context_str}"
```

---

## Complete Examples

### Example 1: Simple Arithmetic Grammar

**EBNF Input** (`arithmetic.ebnf`):
```ebnf
@description: "Simple arithmetic expressions"
@complexity: "O(n)"
expression := term ("+" term)* -> {op: "+", terms: [$1, $3*]}

@type: "number"  
@range: {min: 0, max: 9999}
term := /(\d+)/ -> $1
```

**Raw AST JSON** (from `ebnf_to_json.pl arithmetic.ebnf`):
```json
{
    "grammar_name": "arithmetic",
    "raw_ast": [
        [
            ["rule", "expression"],
            ["semantic_annotation", ["description", "\"Simple arithmetic expressions\""]],
            ["semantic_annotation", ["complexity", "\"O(n)\""]],
            ["rule_reference", "term"],
            ["group_open", "("],
            ["quoted_string", "+"],
            ["rule_reference", "term"],
            ["group_close", ")"],
            ["operator", "*"],
            ["return_annotation", "-> {op: \"+\", terms: [$1, $3*]}"]
        ],
        [
            ["rule", "term"],
            ["semantic_annotation", ["type", "\"number\""]],
            ["semantic_annotation", ["range", "{min: 0, max: 9999}"]],
            ["regex", "(\\d+)"],
            ["return_annotation", "-> $1"]
        ]
    ]
}
```

**Transformation Pipeline Results**:

After Step 5, the grammar tree looks like:
```json
{
    "expression": {
        "type": "sequence",
        "elements": [
            {
                "type": "atom",
                "value": ["rule_reference", "term"]
            },
            {
                "type": "quantified", 
                "element": {
                    "type": "sequence",
                    "elements": [
                        ["quoted_string", "+"],
                        ["rule_reference", "term"]
                    ]
                },
                "quantifier": "*"
            }
        ],
        "semantic_annotations": [
            {
                "type": "atom",
                "value": ["semantic_annotation", ["description", "\"Simple arithmetic expressions\""]]
            },
            {
                "type": "atom", 
                "value": ["semantic_annotation", ["complexity", "\"O(n)\""]]
            }
        ],
        "return_annotations": ["-> {op: \"+\", terms: [$1, $3*]}"]
    },
    "term": {
        "type": "atom",
        "value": ["regex", "(\\d+)"],
        "semantic_annotations": [
            {
                "type": "atom",
                "value": ["semantic_annotation", ["type", "\"number\""]]
            },
            {
                "type": "atom",
                "value": ["semantic_annotation", ["range", "{min: 0, max: 9999}"]]
            }
        ],
        "return_annotations": ["-> $1"]
    }
}
```

**Generated Parser Code** (Rust example):
```rust
// Generated parser for arithmetic grammar
use std::collections::HashMap;
use regex::Regex;

pub fn parse_expression(input: &mut ParseInput) -> ParseResult<ASTNode> {
    let start_pos = input.save_position();
    let mut results = Vec::new();
    
    // Parse first term
    if let Some(term1) = parse_term(input)? {
        results.push(term1);
        
        // Parse zero or more ("+" term) sequences
        let mut add_terms = Vec::new();
        while let Some(_) = match_literal(input, "+")? {
            if let Some(term) = parse_term(input)? {
                add_terms.push(term);
            } else {
                break;
            }
        }
        
        // Apply return annotation: {op: "+", terms: [$1, $3*]}
        return Ok(Some(ASTNode::Object(hashmap!{
            "op".to_string() => ASTNode::Terminal("+".to_string()),
            "terms".to_string() => ASTNode::Array({
                let mut terms = vec![results[0].clone()];
                terms.extend(add_terms);
                terms
            })
        })));
    }
    
    input.restore_position(start_pos);
    Ok(None)
}

pub fn parse_term(input: &mut ParseInput) -> ParseResult<ASTNode> {
    // Apply semantic constraints from annotations:
    // @type: "number", @range: {min: 0, max: 9999}
    
    if let Some(matched) = match_regex(input, r"(\d+)")? {
        let num_value: i32 = matched.parse().unwrap_or(0);
        
        // Validate semantic constraints
        if num_value >= 0 && num_value <= 9999 {
            return Ok(Some(ASTNode::Number(num_value)));
        }
    }
    
    Ok(None)
}
```

### Example 2: Left-Recursion Elimination

**EBNF Input**:
```ebnf
expression := expression "+" term | term
term := "number"
```

**Before Elimination**:
```json
{
    "expression": {
        "type": "or",
        "alternatives": [
            {
                "type": "sequence",
                "elements": [
                    ["rule_reference", "expression"],
                    ["quoted_string", "+"],
                    ["rule_reference", "term"]
                ]
            },
            {
                "type": "atom",
                "value": ["rule_reference", "term"]
            }
        ]
    }
}
```

**After Elimination**:
```json
{
    "expression": {
        "type": "sequence",
        "elements": [
            ["rule_reference", "term"],
            {
                "type": "quantified",
                "element": {
                    "type": "sequence", 
                    "elements": [
                        ["quoted_string", "+"],
                        ["rule_reference", "term"]
                    ]
                },
                "quantifier": "*"
            }
        ]
    }
}
```

The left-recursive `expression := expression "+" term` becomes the equivalent right-recursive form: `expression := term ("+" term)*`.

---

## Language-Specific Optimizations

### Rust: Zero-Cost Abstractions

```rust
// Use lifetime parameters to avoid copying
pub struct Token<'a> {
    token_type: &'a str,
    value: &'a str,
}

// Compile-time regex compilation
lazy_static! {
    static ref REGEX_CACHE: HashMap<&'static str, Regex> = {
        let mut map = HashMap::new();
        map.insert("number", Regex::new(r"\d+").unwrap());
        map.insert("identifier", Regex::new(r"[a-zA-Z_]\w*").unwrap());
        map
    };
}
```

### Julia: Multiple Dispatch

```julia
# Use multiple dispatch for clean transformation code
transform(::Type{Terminal}, token::Vector{String}) = TerminalNode(token[2])
transform(::Type{Rule}, token::Vector{String}) = RuleNode(token[2])
transform(::Type{Quantified}, element::ASTNode, quant::String) = QuantifiedNode(element, quant)

# Generic transformation dispatcher
function transform_token(token::Vector{String})
    token_type = token[1]
    if token_type == "quoted_string"
        return transform(Terminal, token)
    elseif token_type == "rule_reference"
        return transform(Rule, token)
    else
        error("Unknown token type: $token_type")
    end
end
```

### Go: Concurrent Processing

```go
// Process rules concurrently for large grammars
func TransformGrammarParallel(rules []RawRule) (map[string]ASTNode, error) {
    results := make(map[string]ASTNode)
    errors := make(chan error, len(rules))
    transformed := make(chan TransformedRule, len(rules))
    
    // Start workers
    for _, rule := range rules {
        go func(r RawRule) {
            if result, err := TransformRule(r); err != nil {
                errors <- err
            } else {
                transformed <- TransformedRule{Name: r.Name, AST: result}
            }
        }(rule)
    }
    
    // Collect results
    for i := 0; i < len(rules); i++ {
        select {
        case err := <-errors:
            return nil, err
        case result := <-transformed:
            results[result.Name] = result.AST
        }
    }
    
    return results, nil
}
```

### Zig: Compile-Time Optimization

```zig
// Compile-time generation of parser functions
fn generateParser(comptime grammar: []const Rule) type {
    return struct {
        pub fn parse(input: []const u8) ParseResult {
            // Generate parsing logic at compile time
            inline for (grammar) |rule| {
                if (parseRule(rule, input)) |result| {
                    return result;
                }
            }
            return null;
        }
        
        fn parseRule(comptime rule: Rule, input: []const u8) ?ASTNode {
            // Compile-time rule processing
            switch (rule.type) {
                .terminal => return parseTerminal(rule.value, input),
                .sequence => return parseSequence(rule.elements, input),
                .quantified => return parseQuantified(rule.element, rule.quantifier, input),
            }
        }
    };
}
```

### TypeScript: Type-Safe AST

```typescript
// Strict typing for AST nodes
interface ASTNode {
    readonly type: string;
}

interface TerminalNode extends ASTNode {
    readonly type: "terminal";
    readonly value: string;
}

interface SequenceNode extends ASTNode {
    readonly type: "sequence";
    readonly elements: readonly ASTNode[];
}

interface QuantifiedNode extends ASTNode {
    readonly type: "quantified";
    readonly element: ASTNode;
    readonly quantifier: "?" | "*" | "+";
}

// Type-safe transformation functions
function transformToken(token: [string, string]): ASTNode {
    const [tokenType, tokenValue] = token;
    
    switch (tokenType) {
        case "quoted_string":
            return { type: "terminal", value: tokenValue } as TerminalNode;
        case "rule_reference":
            return { type: "rule", name: tokenValue } as RuleNode;
        default:
            throw new Error(`Unknown token type: ${tokenType}`);
    }
}
```

---

## Conclusion

This comprehensive AST transformation pipeline documentation provides everything needed to implement multi-language parser generators. The key points for implementers:

### ✅ **Requirements Checklist**

**Core Pipeline (Required)**:
- [ ] JSON input parsing and validation
- [ ] 5-step transformation pipeline implementation
- [ ] All three annotation systems (semantic, logging, return)
- [ ] Annotation filtering and preservation
- [ ] Basic error handling with context

**Code Generation (Required)**:
- [ ] Parser function generation for each rule
- [ ] Terminal and regex matching
- [ ] Quantifier handling (?, *, +)
- [ ] Return annotation processing (AST construction)
- [ ] Logging annotation processing (debug code)

**Advanced Features (Optional)**:
- [ ] Left-recursion elimination
- [ ] Performance optimizations
- [ ] Domain-specific DataGenerator

### 🎯 **Next Steps for Implementation**

1. **Choose Your Language** and set up project structure
2. **Implement Core Pipeline** following the 5-step algorithm exactly
3. **Add Code Generation** using language-specific best practices
4. **Test Thoroughly** with provided examples and test cases
5. **Optimize** using your language's strengths
6. **Contribute Back** with improvements and bug fixes

### 📚 **Additional Resources**

- **Reference Implementation**: `tools/generators/perl_parser_gen` (most complete)
- **Test Grammars**: Use JSON, arithmetic, and other examples
- **API Documentation**: See individual language generator files
- **Community**: Join development discussions and contribute

The investment in implementing this pipeline correctly will result in robust, maintainable, and feature-complete parser generators that can handle real-world grammars with full annotation support.

**Happy coding!** 🚀
