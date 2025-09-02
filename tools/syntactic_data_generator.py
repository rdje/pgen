#!/usr/bin/env python3
"""
Syntactic DataGenerator - Universal EBNF Input Generator

Generates syntactically valid input files from transformed AST grammars.
Works with any EBNF grammar without requiring domain-specific knowledge.

Usage:
    python syntactic_data_generator.py grammar.json --output test_data.txt --count 50
    python syntactic_data_generator.py grammar.json --config config.json --count 100
"""

import json
import random
import argparse
import re
from typing import Dict, List, Any, Optional, Tuple, Union
from dataclasses import dataclass, field
from pathlib import Path

@dataclass
class GeneratorConfig:
    """Configuration for syntactic data generation"""
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
    prefer_shorter_alternatives: bool = True
    
    # Terminal generation
    string_length_range: Tuple[int, int] = (3, 15)
    regex_generation_attempts: int = 10
    
    # Output formatting
    pretty_print: bool = True
    indent_size: int = 2

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
            
            try:
                sample = self.generate_rule(start_rule, 0)
                samples.append(sample)
            except RecursionError as e:
                print(f"Warning: Recursion error generating sample {i+1}, skipping: {e}")
                continue
            except Exception as e:
                print(f"Warning: Error generating sample {i+1}, skipping: {e}")
                continue
        
        return samples
    
    def generate_rule(self, rule_name: str, depth: int) -> str:
        """Generate content for a specific grammar rule"""
        
        # Check depth limits to prevent infinite recursion
        if depth > self.config.max_depth:
            return self._generate_minimal_for_rule(rule_name)
        
        # Track recursion to detect direct/indirect left recursion
        if self.call_stack.count(rule_name) > self.config.recursion_limit:
            return self._generate_minimal_for_rule(rule_name)
        
        self.call_stack.append(rule_name)
        self.stats['rules_generated'] += 1
        self.stats['max_depth_reached'] = max(self.stats['max_depth_reached'], depth)
        
        try:
            if rule_name not in self.grammar_tree:
                return f"<{rule_name}>"  # Fallback for missing rules
                
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
            return f"<unknown:{rule_type}>"

    def _generate_atom(self, atom_value: Union[List, Dict], depth: int) -> str:
        """Generate content for atomic elements"""
        
        # Handle different atom value formats
        if isinstance(atom_value, list) and len(atom_value) >= 2:
            token_type, token_value = atom_value[0], atom_value[1]
        elif isinstance(atom_value, dict):
            # Handle nested structures within atoms
            if "type" in atom_value:
                return self._generate_from_rule_definition(atom_value, depth)
            else:
                return f"<atom_error>"
        else:
            return f"<atom_error>"
        
        if token_type == "quoted_string":
            return token_value.strip('"\'')  # Return literal string
        elif token_type == "regex":
            return self._generate_from_regex(token_value)
        elif token_type == "rule_reference":
            return self.generate_rule(token_value, depth + 1)
        else:
            return f"<{token_type}:{token_value}>"

    def _generate_sequence(self, elements: List, depth: int) -> str:
        """Generate content for sequence elements"""
        results = []
        
        for element in elements:
            if isinstance(element, list) and len(element) >= 2:
                # Handle token format
                if element[0] == "quoted_string":
                    results.append(element[1].strip('"\''))
                elif element[0] == "rule_reference":
                    results.append(self.generate_rule(element[1], depth + 1))
                elif element[0] == "regex":
                    results.append(self._generate_from_regex(element[1]))
                else:
                    results.append(f"<{element[0]}:{element[1]}>")
            elif isinstance(element, dict):
                # Handle structured elements
                results.append(self._generate_from_rule_definition(element, depth + 1))
            else:
                results.append(str(element))
        
        # Add spaces between elements if pretty printing
        if self.config.pretty_print and len(results) > 1:
            return " ".join(results)
        else:
            return "".join(results)

    def _generate_alternative(self, alternatives: List[Dict], depth: int) -> str:
        """Choose one alternative from OR choices"""
        
        if not alternatives:
            return ""
        
        # Apply weighting strategies
        weights = self._calculate_alternative_weights(alternatives, depth)
        
        # Choose alternative based on weights
        chosen_alt = self.random.choices(alternatives, weights=weights)[0]
        self.stats['alternatives_chosen'] += 1
        
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
            
            weights.append(base_weight)
        
        return weights
    
    def _estimate_complexity(self, rule_def: Dict) -> float:
        """Estimate complexity of a rule definition"""
        rule_type = rule_def.get("type", "")
        
        if rule_type == "atom":
            return 1.0
        elif rule_type == "sequence":
            return len(rule_def.get("elements", []))
        elif rule_type == "or":
            return len(rule_def.get("alternatives", []))
        elif rule_type == "quantified":
            return 2.0  # Quantified elements add complexity
        else:
            return 1.0

    def _generate_quantified(self, element: Dict, quantifier: str, depth: int) -> str:
        """Generate repeated content for quantified elements"""
        
        # Determine quantity based on quantifier and config
        min_count = self.config.quantifier_min.get(quantifier, 0)
        max_count = self.config.quantifier_max.get(quantifier, 1)
        
        # Adjust for depth to prevent explosion
        if depth > 7:
            max_count = min(max_count, 2)
        
        count = self.random.randint(min_count, max_count)
        self.stats['quantifiers_expanded'] += 1
        
        results = []
        for i in range(count):
            # Generate each repetition
            result = self._generate_from_rule_definition(element, depth + 1)
            if result:  # Only add non-empty results
                results.append(result)
                
                # Add separators for readability if configured
                if i < count - 1 and self.config.pretty_print:
                    if self._needs_separator(element):
                        results.append(" ")
        
        return "".join(results)
    
    def _needs_separator(self, element: Dict) -> bool:
        """Determine if element needs separator when repeated"""
        # Add separator for rule references and complex elements
        return element.get("type") in ["atom", "sequence", "or"]

    def _generate_from_regex(self, regex_pattern: str) -> str:
        """Generate string matching regex pattern"""
        
        # Basic regex generation - can be extended
        if regex_pattern in [r"(\d+)", r"\d+", "\\d+"]:
            return str(self.random.randint(1, 999))
        elif regex_pattern in [r"([a-zA-Z_]\w*)", r"[a-zA-Z_]\w*"]:
            return self._generate_identifier()
        elif regex_pattern in [r'"([^"\\]|\\.)*"', r'"[^"]*"']:
            return f'"{self._generate_string_content()}"'
        elif regex_pattern in [r"'([^'\\]|\\.)*'", r"'[^']*'"]:
            return f"'{self._generate_string_content()}'"
        elif regex_pattern in [r"\s+", "\\s+"]:
            return " "
        else:
            # Fallback: generate based on pattern analysis
            return self._generate_from_regex_analysis(regex_pattern)

    def _generate_identifier(self) -> str:
        """Generate valid identifier"""
        first_chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_"
        rest_chars = first_chars + "0123456789"
        
        length = self.random.randint(3, 12)
        result = [self.random.choice(first_chars)]
        
        for _ in range(length - 1):
            result.append(self.random.choice(rest_chars))
        
        return "".join(result)
    
    def _generate_string_content(self) -> str:
        """Generate string content for quoted strings"""
        chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 ._-"
        length = self.random.randint(*self.config.string_length_range)
        return "".join(self.random.choice(chars) for _ in range(length))
    
    def _generate_from_regex_analysis(self, regex_pattern: str) -> str:
        """Fallback regex generation using simple pattern analysis"""
        
        # Remove common regex delimiters
        pattern = regex_pattern.strip("/()^$")
        
        # Handle simple character classes
        if "[0-9]" in pattern or "\\d" in pattern:
            return str(self.random.randint(0, 9))
        elif "[a-z]" in pattern:
            return self.random.choice("abcdefghijklmnopqrstuvwxyz")
        elif "[A-Z]" in pattern:
            return self.random.choice("ABCDEFGHIJKLMNOPQRSTUVWXYZ")
        elif "[a-zA-Z]" in pattern:
            return self.random.choice("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ")
        else:
            # Simple fallback
            return "value"

    def _generate_minimal_for_rule(self, rule_name: str) -> str:
        """Generate minimal valid content when depth/recursion limits hit"""
        
        if rule_name not in self.grammar_tree:
            return ""
            
        rule_def = self.grammar_tree[rule_name]
        return self._generate_minimal_from_rule_def(rule_def)

    def _generate_minimal_from_rule_def(self, rule_def: Dict) -> str:
        """Generate shortest possible valid content"""
        
        rule_type = rule_def.get("type")
        
        if rule_type == "atom":
            atom_value = rule_def["value"]
            if isinstance(atom_value, list) and len(atom_value) >= 2:
                if atom_value[0] == "quoted_string":
                    return atom_value[1].strip("\"'")
                elif atom_value[0] == "regex":
                    return self._minimal_regex_match(atom_value[1])
                else:
                    return ""  # Minimal for rule reference
            return ""
        
        elif rule_type == "sequence":
            # Generate minimal for each element
            results = []
            for element in rule_def.get("elements", []):
                if isinstance(element, list) and element[0] == "quoted_string":
                    results.append(element[1].strip("\"'"))
                # Skip other elements for minimal generation
            return "".join(results)
        
        elif rule_type == "or":
            # Choose shortest alternative
            alternatives = rule_def.get("alternatives", [])
            if alternatives:
                shortest = min(alternatives, key=self._estimate_minimal_length)
                return self._generate_minimal_from_rule_def(shortest)
            return ""
        
        elif rule_type == "quantified":
            quantifier = rule_def.get("quantifier", "")
            if quantifier in ["*", "?"]:
                return ""  # Zero repetitions
            else:  # "+"
                return self._generate_minimal_from_rule_def(rule_def["element"])
        
        return ""
    
    def _estimate_minimal_length(self, rule_def: Dict) -> int:
        """Estimate minimal length of rule definition output"""
        rule_type = rule_def.get("type", "")
        
        if rule_type == "atom":
            return 1
        elif rule_type == "sequence":
            return len(rule_def.get("elements", []))
        elif rule_type == "quantified":
            quantifier = rule_def.get("quantifier", "")
            return 0 if quantifier in ["*", "?"] else 1
        else:
            return 1

    def _minimal_regex_match(self, regex_pattern: str) -> str:
        """Generate minimal string matching regex"""
        if "\\d" in regex_pattern or "[0-9]" in regex_pattern:
            return "0"
        elif "\\w" in regex_pattern or "[a-zA-Z]" in regex_pattern:
            return "a"
        else:
            return "x"
    
    def generate_to_file(self, output_file: str, start_rule: str = None, count: int = 10):
        """Generate samples and write to file"""
        samples = self.generate(start_rule, count)
        
        with open(output_file, 'w') as f:
            f.write(f"# Generated test data from grammar\n")
            f.write(f"# Generated {len(samples)} samples\n")
            f.write(f"# Configuration: seed={self.config.seed}, max_depth={self.config.max_depth}\n\n")
            
            for i, sample in enumerate(samples):
                f.write(f"# Sample {i + 1}\n")
                f.write(sample)
                f.write("\n\n")
        
        return len(samples)

def load_grammar_from_json(json_file: str) -> Tuple[Dict, List[str]]:
    """Load grammar from transformed JSON file"""
    with open(json_file) as f:
        data = json.load(f)
    
    # Handle different JSON formats
    if "grammar_tree" in data and "rule_order" in data:
        return data["grammar_tree"], data["rule_order"]
    elif "raw_ast" in data:
        # If we have raw AST, we need to transform it first
        # For now, return empty - this would need the transformation pipeline
        print("Warning: Raw AST format detected, transformation needed")
        return {}, []
    else:
        print("Warning: Unknown JSON format")
        return {}, []

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
    parser.add_argument("--stats", action="store_true", help="Show generation statistics")
    
    args = parser.parse_args()
    
    # Load grammar
    try:
        grammar_tree, rule_order = load_grammar_from_json(args.grammar_json)
        if not grammar_tree:
            print("Error: Could not load grammar from JSON file")
            return 1
    except Exception as e:
        print(f"Error loading grammar: {e}")
        return 1
    
    # Load configuration
    config = GeneratorConfig()
    if args.config:
        try:
            with open(args.config) as f:
                config_dict = json.load(f)
                # Update config with JSON values
                for key, value in config_dict.items():
                    if hasattr(config, key):
                        setattr(config, key, value)
        except Exception as e:
            print(f"Warning: Could not load config file: {e}")
    
    # Apply command line overrides
    config.seed = args.seed
    config.max_depth = args.max_depth
    
    # Initialize generator
    generator = SyntacticDataGenerator(grammar_tree, rule_order, config)
    
    # Generate samples
    try:
        if args.output:
            count = generator.generate_to_file(args.output, args.rule, args.count)
            print(f"Generated {count} samples to {args.output}")
        else:
            samples = generator.generate(args.rule, args.count)
            for i, sample in enumerate(samples):
                print(f"# Sample {i + 1}")
                print(sample)
                print()
        
        # Show statistics if requested
        if args.stats:
            print(f"\nGeneration Statistics:")
            print(f"  Rules generated: {generator.stats['rules_generated']}")
            print(f"  Alternatives chosen: {generator.stats['alternatives_chosen']}")
            print(f"  Quantifiers expanded: {generator.stats['quantifiers_expanded']}")
            print(f"  Max depth reached: {generator.stats['max_depth_reached']}")
    
    except Exception as e:
        print(f"Error during generation: {e}")
        return 1
    
    return 0

if __name__ == "__main__":
    exit(main())
