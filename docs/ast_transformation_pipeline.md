## Handling Logging Annotations in the AST Transformation Pipeline

### Background

During parser generation, **logging annotations** provide metadata to allow insertion of logging hooks or debugging aids in the generated parser code. Proper handling of these annotations is crucial to prevent them from being mistakenly interpreted as grammar elements, which can break the parser generation process.

### Recognition of Logging Annotations

Logging annotations can appear in two common forms within the raw AST and intermediate representations:

- **Direct array format**:  
  ```perl
  ['logging_annotation', ...]
  ```

- **Structured atom format**:  
  ```perl
  {
    type => 'atom',
    value => ['logging_annotation', ...]
  }
  ```

Failure to recognize both formats resulted in logging annotations leaking into grammar element processing, causing invalid parser function calls like `parse_ARRAY(0x...)` and invalid regex patterns.

### Solution

- The pipeline's `is_logging_annotation` function was updated to detect both forms, ensuring logging annotations are **filtered out** from grammar elements early in the `build_sequence_elements` and related transformation steps.

- Logging annotations are **preserved as metadata** attached to AST nodes, allowing future transformation steps or code generation phases to emit logging calls based on this metadata.

### Impact

- Eliminates invalid grammar tokens containing memory references from logging annotations.

- Prevents the generation of incorrect parse function calls and regex patterns.

- Establishes a **clean separation** between grammar elements and logging metadata in AST representations.

- Sets the foundation for future implementation of logging code generation that will leverage these annotations.

# AST Transformation Pipeline Documentation

🎯 **For Language Implementers** - Complete guide to implementing the AST transformation pipeline in your target language.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Raw AST Format](#raw-ast-format)  
3. [Transformation Pipeline](#transformation-pipeline)
4. [Step-by-Step Algorithms](#step-by-step-algorithms)
5. [Complete Worked Example](#complete-worked-example)
6. [Implementation Best Practices](#implementation-best-practices)
7. [Common Pitfalls](#common-pitfalls)
8. [Testing Strategy](#testing-strategy)

---

## Architecture Overview

The new JSON-based parser generation architecture follows this flow:

```
EBNF Grammar → [ebnf_to_json.pl] → Raw AST JSON → [Language Generator] → Target Parser
                                                        ↓
                                    1. Load Raw AST JSON
                                    2. Transform AST (5 steps)  
                                    3. Generate Parser Code
```

**Key Principle**: Each language generator implements its own transformation pipeline optimized for that language's idioms and type system.

### Benefits of This Approach

- **Language Independence**: No dependency on Perl transformation logic
- **Optimization Freedom**: Each generator can optimize for its target language  
- **Innovation Friendly**: New transformation approaches can be experimented with
- **Type Safety**: Generators can ensure full type safety in target language
- **Maintainability**: Clear separation of concerns

---

## Raw AST Format

The `ebnf_to_json.pl` tool outputs JSON containing the raw AST exactly as returned by the EBNF parser (built from `ebnf.spec` via `LinkedSpec.pm`).

### JSON Structure

```json
{
    "grammar_name": "json", 
    "raw_ast": [
        // Array of rules, each rule is array of tokens
    ],
    "metadata": {
        "source_file": "json.ebnf",
        "format": "raw_ast", 
        "description": "Direct EBNF parser output before transformations"
    }
}
```

### Token Structure

Each token in the raw AST is a 2-element array: `[token_type, token_value]`
### Token Types:**
- `"rule"` - Rule name definition  
- `"quoted_string"` - Terminal in quotes
- `"rule_reference"` - Non-terminal reference (identifier that refers to another rule)
- `"regex"` - Regular expression pattern (used for terminals)
- `"identifier"` - General identifier
- `"operator"` - `|`, `(`, `)`, `[`, `]`, `{`, `}`
- `"quantifier"` - `?`, `*`, `+`
- `"comment"` - Comment text
- `"whitespace"` - Spaces, newlines (usually filtered)
- `"return_scalar"`, `"return_array"`, `"return_object"` - Return annotations
- `"logging_annotation"` - Logging annotations (e.g., `@log_entry`, `@debug_trace`)
- `"whitespace"` - Spaces, newlines (usually filtered)

### Rule Structure

There are **two equivalent ways** to represent rule alternatives in the raw AST:

#### Format 1: Single Rule with OR Operators

For EBNF rules written with explicit OR operators:
```ebnf
value := object | array | string | number
```

The raw AST will be a **single array**:
```json
[
  ["rule", "value"],
  ["rule_reference", "object"],
  ["operator", "|"],
  ["rule_reference", "array"],
  ["operator", "|"],
  ["rule_reference", "string"],
  ["operator", "|"],
  ["rule_reference", "number"]
]
```

#### Format 2: Multiple Rule Definitions

For EBNF rules written as separate definitions:
```ebnf
value := object
value := array
value := string
value := number
```

The raw AST will contain **multiple rule arrays**:
```json
[
  [["rule", "value"], ["rule_reference", "object"]],
  [["rule", "value"], ["rule_reference", "array"]],
  [["rule", "value"], ["rule_reference", "string"]],
  [["rule", "value"], ["rule_reference", "number"]]
]
```

#### Both Formats Are Equivalent!

⚠️ **Critical for Language Implementers**: Both formats represent the **same logical structure** and must produce **identical semantic trees** after transformation.

The 5-step transformation pipeline is designed to handle both formats and normalize them into the same semantic representation.

### Handling Multiple Rule Definitions

When encountering Format 2 (multiple rule definitions), language generators must first normalize to Format 1 before applying the 5-step pipeline:

```python
def normalize_raw_ast(raw_ast):
    # Check if we have multiple rule arrays (Format 2)
    if isinstance(raw_ast[0][0], list):
        # Format 2: Group alternatives by rule name
        rule_definitions = {}
        for rule_array in raw_ast:
            rule_name = rule_array[0][1]  # Extract rule name
            definition_tokens = rule_array[1:]  # Rest are definition tokens
            
            if rule_name not in rule_definitions:
                rule_definitions[rule_name] = []
            rule_definitions[rule_name].append(definition_tokens)
        
        # Convert to Format 1: OR-separated token streams
        normalized_rules = []
        for rule_name, alternatives in rule_definitions.items():
            combined_tokens = [["rule", rule_name]]
            for i, alt_tokens in enumerate(alternatives):
                if i > 0:
                    combined_tokens.append(["operator", "|"])
                combined_tokens.extend(alt_tokens)
            normalized_rules.append(combined_tokens)
        
        return normalized_rules
    else:
        # Format 1: Already normalized
        return raw_ast

# Then apply 5-step pipeline to each normalized rule
for rule_tokens in normalize_raw_ast(raw_ast):
    transformed_rule = apply_5_step_pipeline(rule_tokens)
```

---

## Transformation Pipeline

The transformation pipeline consists of 5 sequential steps that convert the raw token stream into a semantic tree structure suitable for parser generation.

### Pipeline Overview

```
Raw AST → Step 2 → Step 2.5 → Step 3 → Step 4 → Step 5 → Semantic Tree
         Group    Handle      Parse    Handle    Build
         by OR    Parens      Sequences Quantifiers Tree
```

### Why These Steps?

1. **Step 2 (Group by OR)**: Separates alternatives in grammar rules
2. **Step 2.5 (Handle Parentheses)**: Resolves grouping and precedence  
3. **Step 3 (Parse Sequences)**: Identifies ordered sequences of symbols
4. **Step 4 (Handle Quantifiers)**: Processes `?`, `*`, `+` modifiers
5. **Step 5 (Build Tree)**: Creates final semantic tree structure

Each step builds upon the previous, gradually adding semantic structure.

---

## Step-by-Step Algorithms

### Step 2: Group by OR Operators

**Purpose**: Split rule alternatives separated by `|` operators.

**Input**: Raw token stream for a rule
**Output**: Array of alternative branches

**Algorithm**:
```python
def group_by_or(tokens):
    alternatives = []
    current_alt = []
    
    for token in tokens:
        if token[0] == "operator" and token[1] == "|":
            if current_alt:  # Don't add empty alternatives
                alternatives.append(current_alt)
            current_alt = []
        else:
            current_alt.append(token)
    
    # Add final alternative
    if current_alt:
        alternatives.append(current_alt)
    
    return alternatives
```

**Example**:
```
Input:  [["identifier", "value"], ["operator", "|"], ["identifier", "object"]]
Output: [
    [["identifier", "value"]], 
    [["identifier", "object"]]
]
```

### Step 2.5: Handle Parentheses

**Purpose**: Group tokens within parentheses and handle nested structures.

**Input**: Token stream with parentheses operators
**Output**: Token stream with parentheses resolved into groups

**Algorithm**:
```python
def handle_parentheses(tokens):
    result = []
    i = 0
    
    while i < len(tokens):
        token = tokens[i]
        
        if token[0] == "operator" and token[1] == "(":
            # Find matching closing parenthesis
            group, end_pos = extract_parentheses_group(tokens, i)
            result.append(["group", group])
            i = end_pos + 1
        else:
            result.append(token)
            i += 1
    
    return result

def extract_parentheses_group(tokens, start_pos):
    depth = 0
    group = []
    
    for i in range(start_pos, len(tokens)):
        token = tokens[i]
        
        if token[0] == "operator" and token[1] == "(":
            if depth > 0:  # Don't include the opening paren
                group.append(token)
            depth += 1
        elif token[0] == "operator" and token[1] == ")":
            depth -= 1
            if depth == 0:
                return group, i  # Found matching close
            else:
                group.append(token)
        else:
            if depth > 0:
                group.append(token)
    
    raise Exception("Unmatched parentheses")
```

**Example**:
```
Input:  [["operator", "("], ["identifier", "a"], ["operator", "|"], ["identifier", "b"], ["operator", ")"]]
Output: [["group", [["identifier", "a"], ["operator", "|"], ["identifier", "b"]]]]
```

### Step 3: Parse Sequences  

**Purpose**: Identify ordered sequences of symbols within alternatives.

**Input**: Grouped alternatives from Step 2/2.5
**Output**: Alternatives with sequence structure

**Algorithm**:
```python
def parse_sequences(alternatives):
    result = []
    
    for alt in alternatives:
        # Filter out whitespace and comments
        filtered = [t for t in alt if t[0] not in ["whitespace", "comment"]]
        
        if len(filtered) == 0:
            continue  # Skip empty alternatives
        elif len(filtered) == 1:
            result.append(["single", filtered[0]])
        else:
            result.append(["sequence", filtered])
    
    return result
```

**Example**:
```
Input:  [[["identifier", "string"], ["identifier", "number"]]]
Output: [["sequence", [["identifier", "string"], ["identifier", "number"]]]]
```

### Step 4: Handle Quantifiers

**Purpose**: Process `?` (optional), `*` (zero-or-more), `+` (one-or-more) quantifiers.

**Input**: Sequences with potential quantifier tokens  
**Output**: Sequences with quantifiers attached to symbols

**Algorithm**:
```python
def handle_quantifiers(alternatives):
    result = []
    
    for alt in alternatives:
        if alt[0] == "sequence":
            new_sequence = process_quantifiers_in_sequence(alt[1])
            result.append(["sequence", new_sequence])
        elif alt[0] == "single":
            # Check if single item has quantifier
            quantified = process_single_quantifier(alt[1])
            result.append(["single", quantified])
        else:
            result.append(alt)
    
    return result

def process_quantifiers_in_sequence(sequence):
    result = []
    i = 0
    
    while i < len(sequence):
        token = sequence[i]
        
        # Look ahead for quantifier
        if i + 1 < len(sequence) and sequence[i + 1][0] == "quantifier":
            quantifier = sequence[i + 1][1]
            result.append(["quantified", token, quantifier])
            i += 2  # Skip both token and quantifier
        else:
            result.append(token)
            i += 1
    
    return result

def process_single_quantifier(token):
    # Single tokens don't have following quantifiers in this context
    return token
```

**Example**:
```
Input:  [["sequence", [["identifier", "item"], ["quantifier", "*"]]]]
Output: [["sequence", [["quantified", ["identifier", "item"], "*"]]]]
```

### Step 5: Build Tree Structure

**Purpose**: Create final semantic tree suitable for parser generation.

**Input**: Quantified alternatives 
**Output**: Complete semantic AST tree

**Algorithm**:
```python
def build_tree_structure(rule_name, alternatives):
    if len(alternatives) == 1:
        # Single alternative - no choice needed
        return build_alternative_tree(alternatives[0])
    else:
        # Multiple alternatives - create choice node
        choice_branches = []
        for alt in alternatives:
            choice_branches.append(build_alternative_tree(alt))
        
        return {
            "type": "choice",
            "rule": rule_name,
            "alternatives": choice_branches
        }

def build_alternative_tree(alternative):
    alt_type = alternative[0]
    
    if alt_type == "single":
        return build_symbol_tree(alternative[1])
    elif alt_type == "sequence": 
        return {
            "type": "sequence",
            "elements": [build_symbol_tree(elem) for elem in alternative[1]]
        }
    else:
        raise Exception(f"Unknown alternative type: {alt_type}")

def build_symbol_tree(symbol):
    if symbol[0] == "identifier":
        return {
            "type": "non_terminal", 
            "name": symbol[1]
        }
    elif symbol[0] == "quoted_string":
        return {
            "type": "terminal",
            "value": symbol[1]
        }
    elif symbol[0] == "quantified":
        base_symbol = build_symbol_tree(symbol[1])
        quantifier = symbol[2]
        
        return {
            "type": "quantified",
            "symbol": base_symbol,
            "quantifier": quantifier
        }
    elif symbol[0] == "group":
        # Process group recursively
        grouped_alts = group_by_or(symbol[1])
        return build_tree_structure("group", grouped_alts)
    else:
        raise Exception(f"Unknown symbol type: {symbol[0]}")
```

**Example Final Tree**:
```json
{
    "type": "choice",
    "rule": "json",
    "alternatives": [
        {"type": "non_terminal", "name": "value"},
        {"type": "non_terminal", "name": "object"},
        {"type": "non_terminal", "name": "array"}
    ]
}
```

---

## Complete Worked Example

Let's trace through a complete transformation using **real data** from `ebnf_to_json.pl` for the JSON grammar `value` rule:

```ebnf
value = object | array | string | number | "true" | "false" | "null"
```

### Actual Raw AST Input (from ebnf_to_json.pl)
```json
[
  [["rule", "value"], ["rule_reference", "object"]],
  [["rule", "value"], ["rule_reference", "array"]],
  [["rule", "value"], ["rule_reference", "string"]],
  [["rule", "value"], ["rule_reference", "number"]],
  [["rule", "value"], ["regex", "\\s*true\\s*"]],
  [["rule", "value"], ["regex", "\\s*false\\s*"]],
  [["rule", "value"], ["regex", "\\s*null\\s*"]]
]
```

### Pre-Processing: Group by Rule Name
First, we group alternatives by rule name and reconstruct OR-separated tokens:

```json
[
  ["rule_reference", "object"],
  ["operator", "|"],
  ["rule_reference", "array"],
  ["operator", "|"],
  ["rule_reference", "string"],
  ["operator", "|"],
  ["rule_reference", "number"],
  ["operator", "|"],
  ["regex", "\\s*true\\s*"],
  ["operator", "|"],
  ["regex", "\\s*false\\s*"],
  ["operator", "|"],
  ["regex", "\\s*null\\s*"]
]
```

### Step 2: Group by OR
No OR operators, so single alternative:
```json
[
  [
    ["quoted_string", "["],
    ["identifier", "element"],
    ["quantifier", "*"], 
    ["quoted_string", "]"]
  ]
]
```

### Step 2.5: Handle Parentheses  
No parentheses, passes through unchanged.

### Step 3: Parse Sequences
Four tokens = sequence:
```json
[
  ["sequence", [
    ["quoted_string", "["],
    ["identifier", "element"],
    ["quantifier", "*"],
    ["quoted_string", "]"]
  ]]
]
```

### Step 4: Handle Quantifiers
Quantifier `*` applies to preceding `element`:
```json
[
  ["sequence", [
    ["quoted_string", "["],
    ["quantified", ["identifier", "element"], "*"],
    ["quoted_string", "]"]
  ]]
]
```

### Step 5: Build Tree Structure
Final semantic tree:
```json
{
  "type": "sequence", 
  "elements": [
    {
      "type": "terminal",
      "value": "["
    },
    {
      "type": "quantified",
      "symbol": {
        "type": "non_terminal", 
        "name": "element"
      },
      "quantifier": "*"
    },
    {
      "type": "terminal", 
      "value": "]"
    }
  ]
}
```

---

## Implementation Best Practices

### 1. Type Safety First

**Strongly Typed AST Nodes**: Define clear types for all AST node variants:

```rust
// Rust example
#[derive(Debug, Clone)]
pub enum ASTNode {
    Terminal(String),
    NonTerminal(String), 
    Sequence(Vec<ASTNode>),
    Choice(Vec<ASTNode>),
    Quantified { symbol: Box<ASTNode>, quantifier: Quantifier },
    Group(Box<ASTNode>),
}

#[derive(Debug, Clone)]
pub enum Quantifier { Optional, ZeroOrMore, OneOrMore }
```

```julia
# Julia example
abstract type ASTNode end

struct Terminal <: ASTNode
    value::String
end

struct NonTerminal <: ASTNode  
    name::String
end

struct Sequence <: ASTNode
    elements::Vector{ASTNode}
end

@enum Quantifier Optional ZeroOrMore OneOrMore
```

### 2. Robust Error Handling

**Comprehensive Error Messages**: Include context and position information:

```python
class ParseError(Exception):
    def __init__(self, message, rule_name=None, token_position=None):
        self.message = message
        self.rule_name = rule_name  
        self.token_position = token_position
        super().__init__(self.format_message())
    
    def format_message(self):
        context = ""
        if self.rule_name:
            context += f" in rule '{self.rule_name}'"
        if self.token_position is not None:
            context += f" at position {self.token_position}"
        return f"Parse error: {self.message}{context}"
```

### 3. Incremental Validation

**Validate at Each Step**: Don't wait until the end to catch errors:

```go
func (p *Parser) validateSequence(seq []Token) error {
    if len(seq) == 0 {
        return errors.New("empty sequence not allowed")
    }
    
    // Check for invalid token combinations
    for i, token := range seq {
        if token.Type == "quantifier" && i == 0 {
            return errors.New("quantifier cannot appear at start of sequence")
        }
    }
    
    return nil
}
```

### 4. Memory Efficiency 

**Avoid Deep Copying**: Use references/pointers where appropriate:

```cpp
class ASTNode {
public:
    virtual ~ASTNode() = default;
    virtual std::unique_ptr<ASTNode> clone() const = 0;
};

// Use smart pointers to manage memory automatically
using ASTNodePtr = std::unique_ptr<ASTNode>;
```

---

## Common Pitfalls

### 1. **Parentheses Precedence**
❌ **Wrong**: Processing quantifiers before resolving parentheses
```
"(a | b)*" → Process * first → Error!
```

✅ **Correct**: Handle parentheses in Step 2.5, quantifiers in Step 4
```
"(a | b)*" → Group (a | b) → Apply * to group
```

### 2. **Empty Alternatives**
❌ **Wrong**: Including empty alternatives from consecutive `|` operators
```
"a | | b" → [["a"], [], ["b"]]  // Empty alternative!
```

✅ **Correct**: Filter empty alternatives
```
"a | | b" → [["a"], ["b"]]  // Skip empty
```

### 3. **Quantifier Scope**
❌ **Wrong**: Applying quantifiers to entire sequences
```
"a b*" → Apply * to entire "a b" sequence
```

✅ **Correct**: Quantifiers bind to immediately preceding symbol
```
"a b*" → "a" followed by "b*"
```

### 4. **Nested Group Handling**  
❌ **Wrong**: Flattening nested parentheses
```
"((a | b) | c)" → "a | b | c"  // Lost structure!
```

✅ **Correct**: Preserve nesting structure
```
"((a | b) | c)" → Choice[Group[Choice[a, b]], c]
```

### 5. **Token Type Confusion**
❌ **Wrong**: Treating identifiers as terminals
```json
["identifier", "value"] → Terminal("value")  // Wrong!
```

✅ **Correct**: Map token types correctly
```json
["identifier", "value"] → NonTerminal("value")  // Correct
["quoted_string", "\"hello\""] → Terminal("\"hello\"")  // Correct
```

---

## Testing Strategy

### 1. Unit Test Each Step

Test each transformation step in isolation:

```python
def test_group_by_or():
    # Single alternative
    assert group_by_or([["id", "a"]]) == [[["id", "a"]]]
    
    # Multiple alternatives  
    tokens = [["id", "a"], ["op", "|"], ["id", "b"]]
    assert group_by_or(tokens) == [[["id", "a"]], [["id", "b"]]]
    
    # Empty alternatives (edge case)
    tokens = [["op", "|"], ["id", "a"], ["op", "|"]]
    assert group_by_or(tokens) == [[["id", "a"]]]
```

### 2. Integration Tests

Test complete pipeline with real grammar rules:

```python
def test_complete_pipeline():
    # Test array rule: array = "[" element* "]"
    raw_tokens = [
        ["rule", "array"],
        ["quoted_string", "["], 
        ["identifier", "element"],
        ["quantifier", "*"],
        ["quoted_string", "]"]
    ]
    
    result = run_complete_pipeline("array", raw_tokens)
    
    # Verify structure
    assert result["type"] == "sequence"
    assert len(result["elements"]) == 3
    assert result["elements"][1]["type"] == "quantified"
```

### 3. Error Case Testing

Ensure robust error handling:

```python
def test_error_cases():
    # Unmatched parentheses
    with pytest.raises(ParseError, match="Unmatched parentheses"):
        handle_parentheses([["op", "("], ["id", "a"]])
    
    # Quantifier without symbol
    with pytest.raises(ParseError, match="Quantifier at start"):
        parse_sequences([[["quantifier", "*"], ["id", "a"]]])
```

### 4. Grammar Validation Tests

Test with real-world grammars:

```python
# Test with actual JSON grammar
def test_json_grammar():
    json_grammar = load_test_grammar("json.ebnf")
    result = transform_grammar(json_grammar)
    
    # Verify all rules processed correctly
    assert "json" in result
    assert "object" in result  
    assert "array" in result
    
    # Verify structure correctness
    json_rule = result["json"]
    assert json_rule["type"] == "choice"
```

---

## Performance Considerations

### 1. **Avoid Quadratic Complexity**

Use efficient algorithms for token processing:

```python
# ❌ Quadratic - repeatedly scanning from start
def bad_group_by_or(tokens):
    result = []
    start = 0
    for i, token in enumerate(tokens):
        if token[1] == "|":
            result.append(tokens[start:i])  # Creates new array each time
            start = i + 1
    return result

# ✅ Linear - single pass with accumulator  
def good_group_by_or(tokens):
    alternatives = []
    current = []
    for token in tokens:
        if token[1] == "|":
            alternatives.append(current)
            current = []  # Start new, don't copy
        else:
            current.append(token)
    alternatives.append(current)
    return alternatives
```

### 2. **Memory-Efficient Tree Building**

Use builders/factories to avoid redundant allocations:

```rust
pub struct ASTBuilder {
    node_pool: Vec<ASTNode>,
}

impl ASTBuilder {
    pub fn create_sequence(&mut self, elements: Vec<usize>) -> usize {
        let node_id = self.node_pool.len();
        self.node_pool.push(ASTNode::Sequence { element_ids: elements });
        node_id
    }
    
    pub fn get_node(&self, id: usize) -> &ASTNode {
        &self.node_pool[id]
    }
}
```

### 3. **Lazy Evaluation for Large Grammars**

Don't process all rules upfront - process on demand:

```python
class LazyTransformer:
    def __init__(self, raw_ast):
        self.raw_ast = raw_ast
        self.transformed_cache = {}
    
    def get_rule(self, rule_name):
        if rule_name not in self.transformed_cache:
            raw_rule = self.find_raw_rule(rule_name)
            self.transformed_cache[rule_name] = self.transform_rule(raw_rule)
        return self.transformed_cache[rule_name]
```

---

## Language-Specific Optimizations

### Rust: Zero-Copy with Lifetimes

```rust
// Use string slices to avoid copying
pub struct Token<'a> {
    token_type: &'a str,
    value: &'a str,
}

pub fn parse_tokens<'a>(input: &'a [Token<'a>]) -> Result<ASTNode<'a>, ParseError> {
    // Process without copying strings
}
```

### Julia: Multiple Dispatch

```julia
# Use Julia's multiple dispatch for clean code
transform(::Type{Sequence}, tokens::Vector{Token}) = # Handle sequences
transform(::Type{Choice}, tokens::Vector{Token}) = # Handle choices  
transform(::Type{Quantified}, tokens::Vector{Token}) = # Handle quantifiers

# Generic transformer
function transform_tokens(tokens::Vector{Token})
    # Dispatch based on pattern detection
    pattern_type = detect_pattern(tokens)
    return transform(pattern_type, tokens)
end
```

### Python: Generator-Based Processing

```python
def transform_stream(tokens):
    """Generator-based transformation for memory efficiency"""
    for rule_tokens in chunk_by_rules(tokens):
        yield transform_rule(rule_tokens)

# Memory-efficient processing of large grammars        
for transformed_rule in transform_stream(raw_ast):
    generate_parser_code(transformed_rule)
```

---

## Conclusion

This transformation pipeline is the heart of the new JSON-based parser architecture. By implementing these algorithms correctly in your target language, you can generate robust, type-safe parsers from EBNF grammars.

**Remember:**
- ✅ Follow the 5-step sequence exactly
- ✅ Implement comprehensive error handling  
- ✅ Use your language's type system effectively
- ✅ Test each step thoroughly
- ✅ Optimize for your language's strengths

The investment in getting this pipeline right will pay dividends in parser quality, maintainability, and extensibility.

**Next Steps:**
1. Implement the transformation pipeline in your target language
2. Create comprehensive tests using simple grammars first
3. Validate with complex grammars like JSON, XML, or programming languages
4. Optimize for performance and memory usage
5. Generate parser code from the transformed AST

Good luck building your language generator! 🚀
