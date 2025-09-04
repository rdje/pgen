# Debug Implementation in High-Performance Rust Parser Generator

**STATUS: ✅ PRODUCTION READY** - All compilation issues resolved, generator fully functional.

This document explains how debug functionality is implemented in the AST pipeline code generator, specifically focusing on the `--debug` and `--trace` flags that provide detailed parsing insights during regex parsing and grammar processing.

## Overview

The debug system consists of two complementary debugging modes:

1. **Trace Mode** (`--trace`): Provides detailed entry/exit logging for each parsing rule
2. **Backtrack Debug Mode** (`--debug`): Adds specific logging for backtracking operations during parsing failures

When combined, these provide comprehensive visibility into parser behavior, making it invaluable for debugging complex grammars like regex patterns.

## Implementation Architecture

### 1. Generator Configuration

The debug functionality is configured at the generator level through the `HighPerformanceRustGenerator` struct:

```rust
pub struct HighPerformanceRustGenerator {
    grammar_name: String,
    entry_rule: Option<String>,
    enable_trace: bool,              // Controls trace logging
    pub enable_backtrack_debug: bool, // Controls backtrack logging
}
```

#### Constructor Methods

Three constructor patterns are provided:

```rust
// Basic generator without debug
pub fn new(grammar_name: &str) -> Self

// Generator with trace mode only
pub fn with_trace(grammar_name: &str, enable_trace: bool) -> Self

// Generator with both trace and backtrack debug
pub fn with_full_debug(grammar_name: &str) -> Self
```

### 2. Generated Parser Structure

The generated parser includes debug infrastructure:

```rust
pub struct {GrammarName}Parser<'input> {
    input: &'input str,
    position: usize,
    memo: HashMap<(RuleId, usize), MemoEntry<'input>>,
    bytes: &'input [u8],
    debug_mode: bool,        // Runtime debug flag
    debug_depth: usize,      // Tracks nesting depth
    debug_output: Vec<String>, // Stores debug messages
}
```

## Debug Methods Implementation

### 1. Rule Entry/Exit Logging

#### `debug_enter_rule()` - Rule Entry Tracking
```rust
fn debug_enter_rule(&mut self, rule_name: &str) {
    if self.debug_mode {
        let indent = "  ".repeat(self.debug_depth);
        let context = if self.position < self.input.len() {
            let end_pos = (self.position + 10).min(self.input.len());
            let context_str = self.format_debug_string(&self.input[self.position..end_pos]);
            format!(" at '{}'", context_str)
        } else {
            " at EOF".to_string()
        };
        let msg = format!("{}→ ENTER {}: pos={}{}", indent, rule_name, self.position, context);
        self.debug_output.push(msg);
        self.debug_depth += 1;
    }
}
```

**Features:**
- Indentation shows parsing depth/nesting
- Shows current input position
- Displays upcoming input context (next 10 characters)
- Uses `→` symbol for visual clarity

#### `debug_exit_success()` - Successful Rule Completion
```rust
fn debug_exit_success(&mut self, rule_name: &str, start_pos: usize) {
    if self.debug_mode {
        self.debug_depth = self.debug_depth.saturating_sub(1);
        let indent = "  ".repeat(self.debug_depth);
        let consumed = if self.position > start_pos {
            let consumed_str = self.format_debug_string(&self.input[start_pos..self.position]);
            format!(" consumed '{}'", consumed_str)
        } else {
            " (no input consumed)".to_string()
        };
        let msg = format!("{}← SUCCESS {}: {}->{}{}", 
            indent, rule_name, start_pos, self.position, consumed);
        self.debug_output.push(msg);
    }
}
```

**Features:**
- Shows exact input range consumed
- Position range tracking (`start->end`)
- Uses `←` symbol for exit indication
- Handles zero-consumption cases

#### `debug_exit_fail()` - Failed Rule Attempts
```rust
fn debug_exit_fail(&mut self, rule_name: &str, error: &ParseError) {
    if self.debug_mode {
        self.debug_depth = self.debug_depth.saturating_sub(1);
        let indent = "  ".repeat(self.debug_depth);
        let msg = format!("{}← FAIL {}: {}", indent, rule_name, error);
        self.debug_output.push(msg);
    }
}
```

### 2. Backtracking Debug

#### `debug_backtrack()` - Backtrack Operation Logging
```rust
fn debug_backtrack(&mut self, from_pos: usize, to_pos: usize, reason: &str) {
    if self.debug_mode {
        let indent = "  ".repeat(self.debug_depth);
        let msg = format!("{}⟲ BACKTRACK: {}->{} ({})", indent, from_pos, to_pos, reason);
        self.debug_output.push(msg);
    }
}
```

**Features:**
- Shows exact position change during backtracking
- Includes reason for backtracking
- Uses `⟲` symbol for visual distinction

### 3. Alternative and Sequence Tracking

#### `debug_try_alternative()` - Alternative Branch Attempts
```rust
fn debug_try_alternative(&mut self, alt_index: usize, total: usize) {
    if self.debug_mode {
        let indent = "  ".repeat(self.debug_depth);
        let msg = format!("{}? TRY ALT {}/{}: pos={}", indent, alt_index + 1, total, self.position);
        self.debug_output.push(msg);
    }
}
```

#### `debug_sequence_element()` - Sequence Element Processing
```rust
fn debug_sequence_element(&mut self, elem_index: usize, total: usize, elem_name: &str) {
    if self.debug_mode {
        let indent = "  ".repeat(self.debug_depth);
        let msg = format!("{}▶ SEQ {}/{}: {} at pos={}", 
            indent, elem_index + 1, total, elem_name, self.position);
        self.debug_output.push(msg);
    }
}
```

#### `debug_quantifier_iteration()` - Quantifier Loop Tracking
```rust
fn debug_quantifier_iteration(&mut self, iteration: usize, quantifier: &str) {
    if self.debug_mode {
        let indent = "  ".repeat(self.debug_depth);
        let msg = format!("{}* QUANT '{}' iteration {}: pos={}", 
            indent, quantifier, iteration, self.position);
        self.debug_output.push(msg);
    }
}
```

### 4. String Formatting Utilities

#### `format_debug_string()` - Safe String Representation
```rust
fn format_debug_string(&self, s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\t' => "\\t".to_string(),
            '\\' => "\\\\".to_string(),
            '"' => "\\\"".to_string(),
            c if c.is_control() => format!("\\u{:04x}", c as u32),
            c => c.to_string(),
        })
        .collect()
}
```

**Features:**
- Escapes control characters for readable output
- Handles Unicode properly
- Prevents debug output corruption

## Code Generation Integration

### 1. Rule Method Template

Each generated rule method includes debug hooks:

```rust
fn parse_{rule_name}(&mut self) -> ParseResult<ParseNode<'input>> {
    self.memoized_call(Self::RULE_{RULE_NAME_UPPER}, |parser| {
        parser.debug_enter_rule("{rule_name}");           // ← ENTRY LOGGING
        let start_pos = parser.position;
        
        let parse_result: ParseResult<ParseNode<'input>> = (|| {
            // ... actual parsing logic ...
        })();
        
        match &parse_result {
            Ok(_) => parser.debug_exit_success("{rule_name}", start_pos),    // ← SUCCESS LOGGING
            Err(err) => parser.debug_exit_fail("{rule_name}", err),          // ← FAILURE LOGGING
        };
        
        parse_result
    })
}
```

### 2. Backtrack Debug Integration

The `try_parse` method conditionally includes backtrack debugging:

```rust
fn try_parse<T, F>(&mut self, f: F) -> Option<T>
where
    F: FnOnce(&mut Self) -> ParseResult<T>,
{
    let saved_pos = self.position;
    match f(self) {
        Ok(result) => Some(result),
        Err(_) => {
            // Conditional backtrack debug call (only if enabled)
            self.debug_backtrack(self.position, saved_pos, "try_parse failed");
            self.position = saved_pos;
            None
        }
    }
}
```

**Key Feature:** The `debug_backtrack()` call is conditionally generated based on the `enable_backtrack_debug` flag in the generator configuration.

### 3. Alternative Processing Debug

OR rules generate debug calls for each alternative attempt:

```rust
// Generated code for alternatives
parser.debug_try_alternative(0, 3);  // Trying alternative 1 of 3
if let Some(content) = parser.try_parse(|p| {
    // ... first alternative logic ...
}) {
    result = ParseContent::Alternative(Box::new(ParseNode { /* ... */ }));
} else if let Some(content) = parser.try_parse(|p| {
    p.debug_try_alternative(1, 3);   // Trying alternative 2 of 3
    // ... second alternative logic ...
}) {
    result = ParseContent::Alternative(Box::new(ParseNode { /* ... */ }));
} // ... etc
```

### 4. Sequence Processing Debug

Sequence elements include progress tracking:

```rust
// Generated code for sequences
parser.debug_sequence_element(0, 3, "element_0");  // Processing element 1 of 3
let element_start = parser.position;
// ... parse element 0 ...

parser.debug_sequence_element(1, 3, "element_1");  // Processing element 2 of 3  
let element_start = parser.position;
// ... parse element 1 ...
```

## Usage in AST Pipeline

### CLI Integration

The debug flags are integrated into the AST pipeline CLI:

```rust
pub fn generate_high_performance_parser(
    &mut self,
    raw_ast_json_file: &str,
    output_rust_file: &str,
    enable_trace: bool,         // --trace flag
    enable_backtrack_debug: bool, // --debug flag
) -> Result<()> {
    // ... load and transform AST ...
    
    let mut code_generator = if enable_trace && enable_backtrack_debug {
        HighPerformanceRustGenerator::with_full_debug(&raw_data.grammar_name)
    } else if enable_trace {
        HighPerformanceRustGenerator::with_trace(&raw_data.grammar_name, true)
    } else {
        let mut gen = HighPerformanceRustGenerator::new(&raw_data.grammar_name);
        if enable_backtrack_debug {
            gen.enable_backtrack_debug = true;
        }
        gen
    };
    
    // ... generate parser code ...
}
```

### Debug Output Access

The generated parser provides methods to access debug output:

```rust
// Create parser with debug enabled
let mut parser = RegexParser::with_debug(input);

// Parse (automatically captures debug info)
let result = parser.parse();

// Access debug output
for line in parser.debug_output() {
    println!("{}", line);
}

// Clear debug output for reuse
parser.clear_debug();
```

## Example Debug Output

### Trace Mode Output
```
→ ENTER regex: pos=0 at '(?:test)|ab'
  → ENTER alternative: pos=0 at '(?:test)|ab'
    ? TRY ALT 1/2: pos=0
    → ENTER group: pos=0 at '(?:test)|ab'
      → ENTER group_content: pos=3 at 'test)|ab'
        → ENTER literal: pos=3 at 'test)|ab'
        ← SUCCESS literal: 3->7 consumed 'test'
      ← SUCCESS group_content: 3->7 consumed 'test'
    ← SUCCESS group: 0->8 consumed '(?:test)'
  ← SUCCESS alternative: 0->8 consumed '(?:test)'
← SUCCESS regex: 0->8 consumed '(?:test)'
```

### Combined Trace + Backtrack Debug Output
```
→ ENTER regex: pos=0 at '(?:invalid'
  → ENTER alternative: pos=0 at '(?:invalid'
    ? TRY ALT 1/2: pos=0
    → ENTER group: pos=0 at '(?:invalid'
      → ENTER group_content: pos=3 at 'invalid'
        → ENTER literal: pos=3 at 'invalid'
        ← FAIL literal: Unexpected EOF at position 11
      ← FAIL group_content: Unexpected EOF at position 11
      ⟲ BACKTRACK: 11->3 (try_parse failed)
    ← FAIL group: Unexpected EOF at position 11
    ⟲ BACKTRACK: 11->0 (try_parse failed)
    ? TRY ALT 2/2: pos=0
    → ENTER simple_pattern: pos=0 at '(?:invalid'
    ← FAIL simple_pattern: Unexpected token
    ⟲ BACKTRACK: 11->0 (try_parse failed)
  ← FAIL alternative: No alternative matched
← FAIL regex: No alternative matched
```

## Reproducing in RGX Project

To implement similar debug functionality in the RGX project:

### 1. Add Debug Infrastructure to Parser Struct

```rust
pub struct RgxParser<'input> {
    input: &'input str,
    position: usize,
    // ... other fields ...
    debug_mode: bool,
    debug_depth: usize,
    debug_output: Vec<String>,
}
```

### 2. Implement Debug Methods

Copy the debug methods shown above (`debug_enter_rule`, `debug_exit_success`, etc.).

### 3. Instrument Parsing Methods

For each parsing method, add debug entry/exit calls:

```rust
fn parse_atom(&mut self) -> Result<Atom> {
    self.debug_enter_rule("atom");
    let start_pos = self.position;
    
    let result = (|| {
        // ... actual parsing logic ...
    })();
    
    match &result {
        Ok(_) => self.debug_exit_success("atom", start_pos),
        Err(err) => self.debug_exit_fail("atom", err),
    }
    
    result
}
```

### 4. Add Backtrack Debug to try_parse Equivalent

```rust
fn try_parse<T, F>(&mut self, f: F) -> Option<T> {
    let saved_pos = self.position;
    match f(self) {
        Ok(result) => Some(result),
        Err(_) => {
            if self.debug_mode {
                self.debug_backtrack(self.position, saved_pos, "backtrack on failure");
            }
            self.position = saved_pos;
            None
        }
    }
}
```

### 5. Add Configuration Options

```rust
impl RgxParser<'_> {
    pub fn with_debug(input: &str) -> RgxParser {
        RgxParser {
            input,
            position: 0,
            debug_mode: true,
            debug_depth: 0,
            debug_output: Vec::new(),
            // ... other fields ...
        }
    }
    
    pub fn debug_output(&self) -> &[String] {
        &self.debug_output
    }
}
```

### 6. Instrument Alternatives and Sequences

Add specific debug calls for alternatives and sequence processing as shown in the examples above.

## Benefits

This debug implementation provides:

1. **Hierarchical View**: Indented output shows parse tree structure
2. **Position Tracking**: Exact character positions for all operations
3. **Input Context**: Shows what input is being processed
4. **Backtrack Visibility**: See exactly when and why backtracking occurs
5. **Performance Impact**: Minimal overhead when debug is disabled
6. **Comprehensive Coverage**: Covers all parsing operations (rules, alternatives, sequences, quantifiers)

The combination of trace and backtrack debugging makes complex grammar debugging much more manageable and provides essential visibility into parser behavior during development and troubleshooting.
