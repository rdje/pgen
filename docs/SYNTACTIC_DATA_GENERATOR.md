# Syntactic DataGenerator Framework

**Version:** 1.0 - January 2025  
**Purpose:** Generate syntactically valid input files from EBNF grammars for parser testing  
**Scope:** Syntax-only (semantic correctness deferred to domain-specific implementations)

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Generation Algorithms](#generation-algorithms)
4. [Implementation Framework](#implementation-framework)
5. [Configuration Options](#configuration-options)
6. [Testing Strategy](#testing-strategy)
7. [Usage Examples](#usage-examples)

---

## Overview

### Problem Statement

To thoroughly test multi-language parser generators, we need **comprehensive test inputs** that:

- ✅ **Cover all grammar constructs** (terminals, sequences, alternatives, quantifiers)
- ✅ **Exercise edge cases** (empty sequences, deeply nested structures, long repetitions)
- ✅ **Scale appropriately** (from minimal to complex inputs)
- ✅ **Generate deterministically** (reproducible test cases)
- ✅ **Work universally** (any EBNF grammar, not domain-specific)

### Solution Approach

**Syntactic DataGenerator** generates valid inputs by:

1. **Traversing the transformed AST** from the 5-step pipeline
2. **Making random choices** at OR alternatives 
3. **Generating random quantities** for quantifiers (*, +, ?)
4. **Producing terminal content** for strings and regexes
5. **Controlling depth/complexity** to avoid infinite recursion

### Key Characteristics

- **Syntax-Only**: Ignores semantic annotations (for now)
- **Grammar-Driven**: Uses transformed AST structure directly
- **Configurable**: Controls depth, quantity, randomness
- **Deterministic**: Seeded randomness for reproducible results
- **Universal**: Works with any EBNF grammar

---

## Architecture

### DataGenerator Pipeline

```
Transformed AST → Generation Rules → Random Choices → Syntactic Output
     (Step 5)         (Templates)      (Seeded)         (Valid)
```

### Core Components

```python
class SyntacticDataGenerator:
    def __init__(self, grammar_tree, rule_order, config=None):
        self.grammar_tree = grammar_tree
        self.rule_order = rule_order
        self.config = config or GeneratorConfig()
        self.random = Random(config.seed)
        self.call_stack = []  # Track recursion depth
    
    def generate(self, rule_name=None, max_depth=10):
        """Generate input for grammar starting from rule_name"""
        pass
    
    def generate_rule(self, rule_name, depth):
        """Generate content for specific rule"""
        pass
    
    def generate_alternative(self, alternatives, depth):
        """Choose and generate from OR alternatives"""
        pass
    
    def generate_sequence(self, elements, depth):
        """Generate content for sequence elements"""
        pass
    
    def generate_quantified(self, element, quantifier, depth):
        """Generate repeated content for *, +, ? quantifiers"""
        pass
    
    def generate_terminal(self, terminal_value):
        """Generate content for terminal strings"""
        pass
    
    def generate_regex(self, regex_pattern):
        """Generate content matching regex pattern"""
        pass
```

### Configuration System

```python
@dataclass
class GeneratorConfig:
    # Random seed for reproducible results
    seed: int = 42
    
    # Depth control
    max_depth: int = 10
    recursion_limit: int = 5
    
    # Quantifier behavior
    quantifier_min: Dict[str, int] = field(default_factory=lambda: {
        '*': 0, '+': 1, '?': 0
    })
    quantifier_max: Dict[str, int] = field(default_factory=lambda: {
        '*': 5, '+': 5, '?': 1  
    })
    
    # Alternative selection
    alternative_weights: Dict[str, float] = field(default_factory=dict)
    prefer_shorter_alternatives: bool = True
    
    # Terminal generation
    string_length_range: Tuple[int, int] = (1, 20)
    regex_generation_attempts: int = 10
    
    # Output formatting
    pretty_print: bool = True
    indent_size: int = 2
```

---

## Generation Algorithms

### 1. Rule Generation

```python
def generate_rule(self, rule_name: str, depth: int) -> str:
    """Generate content for a specific grammar rule"""
    
    # Check depth limits to prevent infinite recursion
    if depth > self.config.max_depth:
        return self._generate_minimal_for_rule(rule_name)
    
    # Track recursion to detect direct/indirect left recursion
    if self.call_stack.count(rule_name) > self.config.recursion_limit:
        return self._generate_minimal_for_rule(rule_name)
    
    self.call_stack.append(rule_name)
    
    try:
        rule_def = self.grammar_tree[rule_name]
        result = self._generate_from_rule_definition(rule_def, depth)
        return result
    finally:
        self.call_stack.pop()

def _generate_from_rule_definition(self, rule_def: Dict, depth: int) -> str:
    """Generate content based on rule definition type"""
    
    rule_type = rule_def.get("type")
    
    if rule_type == "atom":
        return self._generate_atom(rule_def["value"], depth)
    elif rule_type == "sequence":
        return self._generate_sequence(rule_def["elements"], depth)
    elif rule_type == "or":
        return self._generate_alternative(rule_def["alternatives"], depth)
    elif rule_type == "quantified":
        return self._generate_quantified(
            rule_def["element"], rule_def["quantifier"], depth
        )
    else:
        raise ValueError(f"Unknown rule type: {rule_type}")
```

### 2. Alternative Selection

```python
def _generate_alternative(self, alternatives: List[Dict], depth: int) -> str:
    """Choose one alternative from OR choices"""
    
    # Apply weighting strategies
    weights = self._calculate_alternative_weights(alternatives, depth)
    
    # Choose alternative based on weights
    chosen_alt = self.random.choices(alternatives, weights=weights)[0]
    
    return self._generate_from_rule_definition(chosen_alt, depth + 1)

def _calculate_alternative_weights(self, alternatives: List[Dict], depth: int) -> List[float]:
    """Calculate selection weights for alternatives"""
    weights = []
    
    for alt in alternatives:
        base_weight = 1.0
        
        # Prefer shorter alternatives at higher depths
        if self.config.prefer_shorter_alternatives and depth > 5:
            complexity = self._estimate_complexity(alt)
            base_weight = 1.0 / (1.0 + complexity * 0.1)
        
        # Apply custom weights from config
        alt_key = self._alternative_key(alt)
        if alt_key in self.config.alternative_weights:
            base_weight *= self.config.alternative_weights[alt_key]
        
        weights.append(base_weight)
    
    return weights
```

### 3. Quantifier Handling

```python
def _generate_quantified(self, element: Dict, quantifier: str, depth: int) -> str:
    """Generate repeated content for quantified elements"""
    
    # Determine quantity based on quantifier and config
    min_count = self.config.quantifier_min[quantifier]
    max_count = self.config.quantifier_max[quantifier]
    
    # Adjust for depth to prevent explosion
    if depth > 7:
        max_count = min(max_count, 2)
    
    count = self.random.randint(min_count, max_count)
    
    results = []
    for i in range(count):
        # Generate each repetition
        result = self._generate_from_rule_definition(element, depth + 1)
        results.append(result)
        
        # Add separators for readability if configured
        if i < count - 1 and self.config.pretty_print:
            if self._needs_separator(element):
                results.append(" ")
    
    return "".join(results)
```

### 4. Terminal Generation

```python
def _generate_atom(self, atom_value: List, depth: int) -> str:
    """Generate content for atomic elements"""
    
    token_type, token_value = atom_value[0], atom_value[1] 
    
    if token_type == "quoted_string":
        return token_value.strip('"\'')  # Return literal string
    elif token_type == "regex":
        return self._generate_from_regex(token_value)
    elif token_type == "rule_reference":
        return self.generate_rule(token_value, depth)
    else:
        raise ValueError(f"Unknown atom type: {token_type}")

def _generate_from_regex(self, regex_pattern: str) -> str:
    """Generate string matching regex pattern"""
    
    # For now, implement basic regex generation
    # Future: Use more sophisticated regex-to-string generator
    
    if regex_pattern == r"(\d+)":
        return str(self.random.randint(1, 9999))
    elif regex_pattern == r"([a-zA-Z_]\w*)":
        return self._generate_identifier()
    elif regex_pattern == r'"([^"\\]|\\.)*"':
        return f'"{self._generate_string_content()}"'
    else:
        # Fallback: try to extract character classes and quantifiers
        return self._generate_from_regex_pattern(regex_pattern)

def _generate_identifier(self) -> str:
    """Generate valid identifier"""
    first_chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_"
    rest_chars = first_chars + "0123456789"
    
    length = self.random.randint(1, 15)
    result = [self.random.choice(first_chars)]
    
    for _ in range(length - 1):
        result.append(self.random.choice(rest_chars))
    
    return "".join(result)
```

### 5. Depth and Recursion Control

```python
def _generate_minimal_for_rule(self, rule_name: str) -> str:
    """Generate minimal valid content when depth/recursion limits hit"""
    
    rule_def = self.grammar_tree[rule_name]
    return self._generate_minimal_from_rule_def(rule_def)

def _generate_minimal_from_rule_def(self, rule_def: Dict) -> str:
    """Generate shortest possible valid content"""
    
    rule_type = rule_def.get("type")
    
    if rule_type == "atom":
        atom_value = rule_def["value"]
        if atom_value[0] == "quoted_string":
            return atom_value[1].strip("\"'")
        elif atom_value[0] == "regex":
            return self._minimal_regex_match(atom_value[1])
        else:
            return ""  # Minimal for rule reference
    
    elif rule_type == "sequence":
        # Generate minimal for each element
        results = []
        for element in rule_def["elements"]:
            minimal = self._generate_minimal_from_rule_def(element)
            results.append(minimal)
        return "".join(results)
    
    elif rule_type == "or":
        # Choose shortest alternative
        alternatives = rule_def["alternatives"]
        shortest = min(alternatives, key=self._estimate_minimal_length)
        return self._generate_minimal_from_rule_def(shortest)
    
    elif rule_type == "quantified":
        quantifier = rule_def["quantifier"]
        if quantifier in ["*", "?"]:
            return ""  # Zero repetitions
        else:  # "+"
            return self._generate_minimal_from_rule_def(rule_def["element"])
    
    return ""
```

---

## Implementation Framework

### Base Implementation (Language-Agnostic)

```python
#!/usr/bin/env python3
"""
Syntactic DataGenerator - Universal EBNF Input Generator

Generates syntactically valid input files from transformed AST grammars.
Works with any EBNF grammar without requiring domain-specific knowledge.
"""

import json
import random
import argparse
from typing import Dict, List, Any, Optional, Tuple
from dataclasses import dataclass, field
from pathlib import Path

class SyntacticDataGenerator:
    """Universal syntactic data generator for EBNF grammars"""
    
    def __init__(self, grammar_tree: Dict, rule_order: List[str], config=None):
        self.grammar_tree = grammar_tree
        self.rule_order = rule_order
        self.config = config or GeneratorConfig()
        self.random = random.Random(self.config.seed)
        self.call_stack = []
        
        # Statistics tracking
        self.stats = {
            'rules_generated': 0,
            'alternatives_chosen': 0,
            'quantifiers_expanded': 0,
            'max_depth_reached': 0
        }
    
    def generate(self, start_rule: str = None, count: int = 1) -> List[str]:
        """Generate multiple input samples"""
        start_rule = start_rule or self.rule_order[0]
        
        samples = []
        for i in range(count):
            # Use different seed for each sample
            self.random.seed(self.config.seed + i)
            self.call_stack = []
            self.stats = {k: 0 for k in self.stats}
            
            sample = self.generate_rule(start_rule, 0)
            samples.append(sample)
        
        return samples
    
    def generate_rule(self, rule_name: str, depth: int) -> str:
        """Generate content for specific rule"""
        # ... implementation as shown above
        pass
    
    def generate_to_file(self, output_file: str, start_rule: str = None, count: int = 10):
        """Generate samples and write to file"""
        samples = self.generate(start_rule, count)
        
        with open(output_file, 'w') as f:
            for i, sample in enumerate(samples):
                f.write(f"# Sample {i + 1}\n")
                f.write(sample)
                f.write("\n\n")
        
        return len(samples)

def main():
    parser = argparse.ArgumentParser(
        description="Generate syntactically valid input files from EBNF grammars"
    )
    parser.add_argument("grammar_json", help="Transformed grammar JSON file")
    parser.add_argument("--output", "-o", help="Output file (default: stdout)")
    parser.add_argument("--count", "-n", type=int, default=10, help="Number of samples")
    parser.add_argument("--rule", "-r", help="Start rule (default: first rule)")
    parser.add_argument("--seed", type=int, default=42, help="Random seed")
    parser.add_argument("--max-depth", type=int, default=10, help="Max recursion depth")
    parser.add_argument("--config", help="Configuration JSON file")
    
    args = parser.parse_args()
    
    # Load grammar
    with open(args.grammar_json) as f:
        grammar_data = json.load(f)
    
    # Load configuration
    config = GeneratorConfig()
    if args.config:
        with open(args.config) as f:
            config_dict = json.load(f)
            config = GeneratorConfig(**config_dict)
    
    # Apply command line overrides
    config.seed = args.seed
    config.max_depth = args.max_depth
    
    # Initialize generator
    generator = SyntacticDataGenerator(
        grammar_data["grammar_tree"],
        grammar_data["rule_order"], 
        config
    )
    
    # Generate samples
    if args.output:
        count = generator.generate_to_file(args.output, args.rule, args.count)
        print(f"Generated {count} samples to {args.output}")
    else:
        samples = generator.generate(args.rule, args.count)
        for i, sample in enumerate(samples):
            print(f"# Sample {i + 1}")
            print(sample)
            print()

if __name__ == "__main__":
    main()
```

---

## Configuration Options

### Example Configuration File

```json
{
    "seed": 42,
    "max_depth": 8,
    "recursion_limit": 3,
    
    "quantifier_min": {
        "*": 0,
        "+": 1, 
        "?": 0
    },
    "quantifier_max": {
        "*": 3,
        "+": 3,
        "?": 1
    },
    
    "alternative_weights": {
        "simple_expression": 2.0,
        "complex_expression": 0.5
    },
    
    "prefer_shorter_alternatives": true,
    "string_length_range": [3, 15],
    "regex_generation_attempts": 5,
    
    "pretty_print": true,
    "indent_size": 2
}
```

### Advanced Configuration Options

```python
@dataclass 
class AdvancedConfig:
    # Complexity control
    complexity_bias: float = 0.7  # 0.0 = simple, 1.0 = complex
    terminal_variety: float = 0.8  # How varied terminal content should be
    
    # Specific rule overrides
    rule_depth_limits: Dict[str, int] = field(default_factory=dict)
    rule_generation_hooks: Dict[str, callable] = field(default_factory=dict)
    
    # Output control  
    format_output: bool = True
    output_comments: bool = True
    sample_metadata: bool = False
    
    # Performance
    generation_timeout: float = 30.0  # seconds
    max_output_size: int = 1_000_000  # characters
```

---

## Testing Strategy

### 1. Parser Validation Tests

```python
def test_generated_inputs_with_parsers(grammar_file, parser_generators):
    """Test that all generated inputs parse successfully with all parsers"""
    
    # Generate inputs
    generator = create_generator_from_ebnf(grammar_file)
    test_inputs = generator.generate(count=100)
    
    # Test with each parser
    results = {}
    for lang, parser_gen in parser_generators.items():
        # Generate parser for this language
        parser = parser_gen.generate_parser_from_ebnf(grammar_file)
        
        # Test all inputs
        successes = 0
        failures = []
        
        for i, test_input in enumerate(test_inputs):
            try:
                result = parser.parse(test_input)
                if result is not None:
                    successes += 1
                else:
                    failures.append((i, "Parse returned None"))
            except Exception as e:
                failures.append((i, str(e)))
        
        results[lang] = {
            'successes': successes,
            'failures': failures,
            'success_rate': successes / len(test_inputs)
        }
    
    return results
```

### 2. Coverage Analysis

```python
def analyze_grammar_coverage(grammar_tree, generated_samples):
    """Analyze how well generated samples cover grammar constructs"""
    
    coverage = {
        'rules_used': set(),
        'alternatives_used': set(),
        'quantifier_patterns': set(),
        'terminal_patterns': set()
    }
    
    # Analyze each sample (would need parser to build AST)
    for sample in generated_samples:
        ast = parse_sample_with_reference_parser(sample)
        analyze_ast_for_coverage(ast, coverage)
    
    # Calculate coverage percentages
    total_rules = len(grammar_tree)
    total_alternatives = count_alternatives_in_grammar(grammar_tree)
    
    results = {
        'rule_coverage': len(coverage['rules_used']) / total_rules,
        'alternative_coverage': len(coverage['alternatives_used']) / total_alternatives,
        'constructs_exercised': len(coverage['quantifier_patterns']),
        'terminal_variety': len(coverage['terminal_patterns'])
    }
    
    return results
```

### 3. Edge Case Generation

```python
def generate_edge_cases(generator):
    """Generate specific edge cases for thorough testing"""
    
    edge_cases = []
    
    # Minimal cases - shortest possible valid inputs
    generator.config.prefer_shorter_alternatives = True
    generator.config.quantifier_max = {'*': 0, '+': 1, '?': 0}
    minimal_cases = generator.generate(count=5)
    edge_cases.extend(('minimal', case) for case in minimal_cases)
    
    # Maximal cases - deeply nested/complex inputs
    generator.config.prefer_shorter_alternatives = False
    generator.config.quantifier_max = {'*': 10, '+': 10, '?': 1}
    generator.config.max_depth = 15
    maximal_cases = generator.generate(count=5)
    edge_cases.extend(('maximal', case) for case in maximal_cases)
    
    # Repetition cases - high quantifier counts
    generator.config.quantifier_min = {'*': 5, '+': 5, '?': 1}
    generator.config.quantifier_max = {'*': 20, '+': 20, '?': 1}
    repetition_cases = generator.generate(count=5)
    edge_cases.extend(('repetition', case) for case in repetition_cases)
    
    return edge_cases
```

---

## Usage Examples

### 1. Generate Test Data for JSON Grammar

```bash
# Generate JSON test data
python syntactic_generator.py json_grammar.json \
    --output json_test_data.txt \
    --count 50 \
    --seed 12345

# Generate with custom configuration
python syntactic_generator.py json_grammar.json \
    --config json_config.json \
    --output json_complex.txt \
    --count 20
```

### 2. Test All Parsers with Generated Data

```bash
#!/bin/bash
# Test all parser generators with same input data

GRAMMAR="arithmetic.ebnf"
TEST_DATA="arithmetic_test_data.txt"

# Generate test data
python syntactic_generator.py $GRAMMAR.json -o $TEST_DATA -n 100

# Test Perl parser
perl_parser_gen $GRAMMAR.json -o ArithmeticParser.pm
perl test_parser.pl ArithmeticParser.pm $TEST_DATA

# Test Rust parser  
rust_parser_gen $GRAMMAR.json -o arithmetic_parser.rs
rustc arithmetic_parser.rs && ./arithmetic_parser $TEST_DATA

# Test Julia parser
julia_parser_gen $GRAMMAR.json -o arithmetic_parser.jl  
julia arithmetic_parser.jl $TEST_DATA

# Compare results
echo "Parser testing complete. Check results for consistency."
```

### 3. Grammar Coverage Analysis

```python
# Analyze how well our generated data covers the grammar
from syntactic_generator import SyntacticDataGenerator, analyze_coverage

# Load grammar and generate data
with open('json_grammar.json') as f:
    grammar = json.load(f)

generator = SyntacticDataGenerator(grammar['grammar_tree'], grammar['rule_order'])
samples = generator.generate(count=500)

# Analyze coverage
coverage = analyze_coverage(grammar['grammar_tree'], samples)
print(f"Rule coverage: {coverage['rule_coverage']:.1%}")
print(f"Alternative coverage: {coverage['alternative_coverage']:.1%}")
print(f"Edge cases found: {coverage['edge_cases_count']}")
```

---

## Integration with Parser Testing

### Automated Test Pipeline

```yaml
# .github/workflows/parser-testing.yml
name: Multi-Language Parser Testing

on: [push, pull_request]

jobs:
  generate-test-data:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Generate test data for all grammars
        run: |
          for grammar in test_grammars/*.ebnf; do
            python tools/syntactic_generator.py "$grammar.json" \
              --output "test_data/$(basename $grammar .ebnf)_tests.txt" \
              --count 100
          done
      - name: Upload test data
        uses: actions/upload-artifact@v2
        with:
          name: generated-test-data
          path: test_data/

  test-perl-parser:
    needs: generate-test-data
    runs-on: ubuntu-latest
    steps:
      - name: Download test data
        uses: actions/download-artifact@v2
      - name: Test Perl parsers
        run: |
          for test_file in test_data/*_tests.txt; do
            # Test corresponding parser
            echo "Testing $(basename $test_file)"
            # ... parser testing logic
          done

  test-rust-parser:
    needs: generate-test-data  
    runs-on: ubuntu-latest
    steps:
      - name: Test Rust parsers with generated data
        run: |
          # Similar testing for Rust parsers
          echo "Testing Rust parsers..."
          
  # Similar jobs for Julia, Go, Zig, TypeScript...
```

---

## Next Steps

### Immediate Implementation

1. **Create Base Framework** 
   - Implement core `SyntacticDataGenerator` class
   - Add basic quantifier and alternative handling
   - Simple terminal generation

2. **Add Configuration System**
   - JSON configuration file support
   - Command-line option overrides
   - Reasonable defaults

3. **Test with Simple Grammars**
   - Start with arithmetic expressions
   - Test JSON grammar
   - Validate against existing parsers

### Future Enhancements

1. **Advanced Regex Generation**
   - Use regex-to-string generation library
   - Support complex character classes
   - Handle lookaheads/lookbehinds

2. **Intelligent Alternative Selection** 
   - Machine learning-based weighting
   - Coverage-guided generation
   - Adaptive complexity

3. **Performance Optimization**
   - Parallel generation for large counts
   - Streaming output for big datasets
   - Memory-efficient generation

4. **Integration Tools**
   - IDE plugins for grammar testing
   - Continuous integration hooks
   - Parser performance benchmarking

## Conclusion

This **Syntactic DataGenerator** provides the foundation needed for comprehensive parser testing across all target languages. It focuses on what we can do **right now** - generating syntactically valid inputs - while laying the groundwork for future semantic enhancements.

**Ready for implementation!** 🚀
