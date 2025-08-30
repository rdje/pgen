# 📋 EBNF Grammar Rules and Best Practices

## Critical Rule: Regex Capturing Groups

### Correct: Use Capturing Groups for $n References
```ebnf
// When using $1, $2, etc., the regex MUST have capturing groups
identifier := /([a-zA-Z][a-zA-Z0-9_]*)/ -> $1
string_literal := /"([^"]*)"/ -> $1
number := /(\d+)/ -> $1
bit_value := /'([01])'/ -> $1
```

### 🚫 **WRONG: $n Without Capturing Groups**
```ebnf
// These will cause $1 to be undefined!
identifier := /[a-zA-Z][a-zA-Z0-9_]*/ -> $1  // NO CAPTURING GROUPS!
bit_literal := /'[01]'/ -> $1                // NO CAPTURING GROUPS!
```

## 🔧 **EBNF vs LinkedSpec.pm Differences**

### **EBNF System** (Our Self-Hosting Parser)
```ebnf
// Only $n with capturing groups
terminal := /(pattern)/ -> $1
sequence := element1 element2 -> [$1, $2]
```

### **LinkedSpec.pm System** (Legacy .spec files)
```spec
// Has $IMATCH for full match
terminal: /pattern/ I {return $IMATCH}
terminal: /(pattern)/ I {return $1}
```

**Do not mix these syntaxes.**

## 📚 **EBNF Grammar Best Practices**

### **1. Regex Terminals**
```ebnf
// GOOD: Clear capturing groups
identifier := /([a-zA-Z_]\w*)/ -> $1
quoted_string := /"([^"]*)"/ -> $1
hex_number := /(0x[0-9a-fA-F]+)/ -> $1

// BAD: Missing capturing groups
identifier := /[a-zA-Z_]\w*/ -> $1     // $1 undefined!
```

### **2. Return Annotations**
```ebnf
// Simple values
name := identifier -> $1

// Arrays
list := item (',' item)* -> [$1, $2*]

// Objects  
declaration := 'var' identifier ':' type -> {name: $2, type: $4}

// Nested structures
complex := outer '{' inner+ '}' -> {type: $1, contents: [$3*]}
```

### **3. Quantifier Patterns**
```ebnf
// Zero or more: *
items := item* -> [$1*]

// One or more: +  
items := item+ -> [$1*]

// Optional: ?
optional := item? -> $1

// Grouped quantifiers
csv_list := item (',' item)* -> [$1, $2*]
```

### **4. Alternative Patterns**
```ebnf
// Simple alternatives
bool_value := 'true' | 'false' -> $1

// Complex alternatives with different returns
statement := assignment | declaration | expression -> $1

// Alternatives with specific returns
literal := string_literal -> {type: "string", value: $1}
        | numeric_literal -> {type: "number", value: $1}
```

## Common Mistakes to Avoid

### **1. Missing Capturing Groups**
```ebnf
// WRONG
number := /\d+/ -> $1

// CORRECT  
number := /(\d+)/ -> $1
```

### **2. Mixing EBNF and LinkedSpec Syntax**
```ebnf
// WRONG (LinkedSpec syntax in EBNF)
identifier := /\w+/ I {return $IMATCH}

// CORRECT (Pure EBNF)
identifier := /(\w+)/ -> $1
```

### **3. Incorrect $n References**
```ebnf
// WRONG - $3 doesn't exist (only 2 capturing groups)
pattern := /(group1)-(group2)/ -> $3

// CORRECT
pattern := /(group1)-(group2)/ -> [$1, $2]
```

### **4. Whitespace Handling**
```ebnf
// GOOD: Explicit whitespace handling
expression := term ('+' term)* -> [$1, $2*]

// BETTER: Dedicated whitespace rule (if needed)
ws := /\s*/ -> ""
expression := term (ws '+' ws term)* -> [$1, $4*]
```

## Advanced Patterns

### **Nested Return Annotations**
```ebnf
function_def := 'function' identifier '(' params ')' block 
             -> {
                  type: "function",
                  name: $2,
                  parameters: $4,
                  body: $5
                }
```

### **Conditional Returns**
```ebnf
// Different returns based on pattern matched
declaration := 'const' identifier '=' value -> {type: "const", name: $2, value: $4}
            | 'var' identifier -> {type: "var", name: $2}
```

### **Ultimate Dot Notation**
```ebnf
// Using our advanced dot notation capabilities
complex_return := data_structure -> {
                    items: [$1.items*],
                    metadata: $1.meta,
                    count: $1.items.length
                  }
```

## Validation Checklist

Before using any EBNF grammar:

- [ ] Every `$n` reference has a corresponding capturing group `(...)`
- [ ] No LinkedSpec.pm syntax (`$IMATCH`, `I {...}`) in EBNF files
- [ ] Return annotations match the actual rule structure
- [ ] Quantifier references use `$n*` for arrays
- [ ] All regex patterns are properly escaped

## Testing Your Grammar

```bash
# Test with our parser generator
cd fx && perl ast_transform.pl ../your_grammar.ebnf

# Look for these error patterns:
# - "Undefined $1" - Missing capturing group
# - "parse_ARRAY errors" - Incorrect structure expectations
# - "Syntax errors" - Mixed EBNF/LinkedSpec syntax
```

## 📖 **Summary**

The **golden rule** for EBNF grammars:

> **Every `$n` reference MUST have a corresponding `(...)` capturing group in the regex pattern.**

This ensures your grammar will work correctly with our self-hosting EBNF parser generator!