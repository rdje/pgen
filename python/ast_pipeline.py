#!/usr/bin/env python3
"""
Python AST Pipeline Implementation

Demonstrates the multi-language architecture with dual-mode API:
1. Same-language optimization: In-memory data structures
2. Cross-language interface: JSON input/output

This implementation replicates the Perl AST::Transform pipeline stages:
- Step 2: Group by OR operators
- Step 2.5: Handle parentheses  
- Step 3: Parse sequences
- Step 4: Handle quantifiers
- Step 5: Build tree structure
"""

import json
import sys
import copy
from typing import Dict, List, Any, Optional, Union, Tuple
from dataclasses import dataclass, field
from pathlib import Path


@dataclass
class PipelineConfig:
    """Configuration for AST transformation pipeline"""
    debug: bool = False
    preserve_annotations: bool = True
    validate_input: bool = True
    validate_output: bool = True


class ASTNode:
    """Represents a node in the transformed AST"""
    
    def __init__(self, node_type: str, **kwargs):
        self.type = node_type
        for key, value in kwargs.items():
            setattr(self, key, value)
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for JSON serialization"""
        result = {"type": self.type}
        for key, value in self.__dict__.items():
            if key != "type":
                if isinstance(value, ASTNode):
                    result[key] = value.to_dict()
                elif isinstance(value, list):
                    result[key] = [
                        item.to_dict() if isinstance(item, ASTNode) else item 
                        for item in value
                    ]
                else:
                    result[key] = value
        return result


class PythonASTPipeline:
    """Python implementation of the AST transformation pipeline"""
    
    def __init__(self, config: PipelineConfig = None):
        self.config = config or PipelineConfig()
        self.debug = self.config.debug
        
        # Statistics and state
        self.stats = {
            'rules_processed': 0,
            'annotations_preserved': 0,
            'transformations_applied': 0
        }
        
        # Preserved annotations
        self.annotations = {
            'semantic_annotations': {},
            'logging_annotations': {},
            'return_annotations': {}
        }
    
    def load_raw_ast(self, json_file: str) -> Dict[str, Any]:
        """Load raw AST JSON from file"""
        if self.debug:
            print(f"Loading raw AST from: {json_file}")
        
        with open(json_file, 'r') as f:
            data = json.load(f)
        
        if self.config.validate_input:
            self._validate_raw_ast_format(data)
        
        return data
    
    def _validate_raw_ast_format(self, data: Dict[str, Any]):
        """Validate raw AST JSON format"""
        required_fields = ['grammar_name', 'raw_ast', 'metadata']
        for field in required_fields:
            if field not in data:
                raise ValueError(f"Raw AST JSON missing required field: {field}")
        
        if not isinstance(data['raw_ast'], list):
            raise ValueError("raw_ast field must be an array")
        
        if data.get('metadata', {}).get('format') != 'raw_ast':
            raise ValueError("metadata.format must be 'raw_ast'")
    
    def transform_raw_ast(self, raw_ast: List[List[List[str]]]) -> Tuple[Dict[str, ASTNode], List[str]]:
        """Transform raw AST to semantic AST (main pipeline)"""
        if self.debug:
            print("=== Python AST Transformation Pipeline ===")
        
        # Step 1: Extract annotations (preprocessing)
        cleaned_ast = self._extract_annotations(raw_ast)
        
        # Step 2: Group by OR operators
        grouped_rules = self._group_by_or_operators(cleaned_ast)
        
        # Step 2.5: Handle parentheses
        processed_rules = self._handle_parentheses(grouped_rules)
        
        # Step 3: Parse sequences  
        sequenced_rules = self._parse_sequences(processed_rules)
        
        # Step 4: Handle quantifiers
        quantified_rules = self._handle_quantifiers(sequenced_rules)
        
        # Step 5: Build tree structure
        grammar_tree, rule_order = self._build_tree_structure(quantified_rules)
        
        self.stats['rules_processed'] = len(grammar_tree)
        self.stats['transformations_applied'] = 5  # Number of pipeline stages
        
        return grammar_tree, rule_order
    
    def _extract_annotations(self, raw_ast: List[List[List[str]]]) -> List[List[List[str]]]:
        """Extract and preserve annotations from raw AST"""
        if self.debug:
            print("Step 1: Extracting annotations...")
        
        cleaned_ast = []
        
        for rule_def in raw_ast:
            if not rule_def:
                continue
            
            rule_name = None
            cleaned_rule = []
            
            for token in rule_def:
                if len(token) != 2:
                    continue
                
                token_type, token_value = token
                
                if token_type == "rule":
                    rule_name = token_value
                    cleaned_rule.append(token)
                elif token_type in ["semantic_annotation", "logging_annotation"]:
                    # Preserve annotations - parse format: ["annotation_type", [name, value]] for semantic
                    # or ["annotation_type", [name, [args...]]] for logging
                    if rule_name and self.config['preserve_annotations']:
                        try:
                            parsed_value = json.loads(token_value)
                            if isinstance(parsed_value, list) and len(parsed_value) >= 2:
                                annotation_name = str(parsed_value[0])
                                
                                if token_type == "semantic_annotation":
                                    if rule_name not in self.annotations['semantic_annotations']:
                                        self.annotations['semantic_annotations'][rule_name] = []
                                    annotation_value = str(parsed_value[1])
                                    formatted_annotation = f"{annotation_name}:{annotation_value}"
                                    self.annotations['semantic_annotations'][rule_name].append(formatted_annotation)
                                    
                                elif token_type == "logging_annotation":
                                    if rule_name not in self.annotations['logging_annotations']:
                                        self.annotations['logging_annotations'][rule_name] = []
                                    if isinstance(parsed_value[1], list):
                                        args = ",".join(str(arg) for arg in parsed_value[1])
                                    else:
                                        args = str(parsed_value[1])
                                    formatted_annotation = f"{annotation_name}({args})"
                                    self.annotations['logging_annotations'][rule_name].append(formatted_annotation)
                            else:
                                # Fallback for malformed annotation data
                                if token_type == "semantic_annotation":
                                    if rule_name not in self.annotations['semantic_annotations']:
                                        self.annotations['semantic_annotations'][rule_name] = []
                                    self.annotations['semantic_annotations'][rule_name].append(f"raw:{token_value}")
                                elif token_type == "logging_annotation":
                                    if rule_name not in self.annotations['logging_annotations']:
                                        self.annotations['logging_annotations'][rule_name] = []
                                    self.annotations['logging_annotations'][rule_name].append(f"raw:{token_value}")
                                    
                        except (json.JSONDecodeError, ValueError):
                            # Fallback for JSON parsing errors
                            if token_type == "semantic_annotation":
                                if rule_name not in self.annotations['semantic_annotations']:
                                    self.annotations['semantic_annotations'][rule_name] = []
                                self.annotations['semantic_annotations'][rule_name].append(f"raw:{token_value}")
                            elif token_type == "logging_annotation":
                                if rule_name not in self.annotations['logging_annotations']:
                                    self.annotations['logging_annotations'][rule_name] = []
                                self.annotations['logging_annotations'][rule_name].append(f"raw:{token_value}")
                        
                        self.stats['annotations_preserved'] += 1
                    # Don't add to cleaned rule
                elif token_type in ["return_scalar", "return_array", "return_object"]:
                    # Preserve return annotations
                    if rule_name:
                        self.annotations['return_annotations'][rule_name] = token_type
                    # Don't add to cleaned rule
                else:
                    cleaned_rule.append(token)
            
            if cleaned_rule:
                cleaned_ast.append(cleaned_rule)
        
        if self.debug:
            print(f"Preserved {self.stats['annotations_preserved']} annotations")
        
        return cleaned_ast
    
    def _group_by_or_operators(self, ast: List[List[List[str]]]) -> Dict[str, List[List[List[str]]]]:
        """Group rule definitions by OR operators (Step 2)"""
        if self.debug:
            print("Step 2: Grouping by OR operators...")
        
        grouped = {}
        
        for rule_def in ast:
            if not rule_def or len(rule_def) < 1:
                continue
                
            rule_name = None
            for token in rule_def:
                if len(token) == 2 and token[0] == "rule":
                    rule_name = token[1]
                    break
            
            if not rule_name:
                continue
            
            # Split by OR operators
            alternatives = []
            current_alt = []
            
            for token in rule_def[1:]:  # Skip rule definition token
                if len(token) == 2 and token[0] == "operator" and token[1] == "|":
                    if current_alt:
                        alternatives.append(current_alt)
                        current_alt = []
                else:
                    current_alt.append(token)
            
            if current_alt:
                alternatives.append(current_alt)
            
            if rule_name not in grouped:
                grouped[rule_name] = []
            
            grouped[rule_name].extend(alternatives)
        
        return grouped
    
    def _handle_parentheses(self, grouped_rules: Dict[str, List[List[List[str]]]]) -> Dict[str, List[List[List[str]]]]:
        """Handle parentheses and grouping (Step 2.5)"""
        if self.debug:
            print("Step 2.5: Handling parentheses...")
        
        processed = {}
        
        for rule_name, alternatives in grouped_rules.items():
            processed_alts = []
            
            for alt in alternatives:
                processed_alt = self._process_parentheses_in_sequence(alt)
                processed_alts.append(processed_alt)
            
            processed[rule_name] = processed_alts
        
        return processed
    
    def _process_parentheses_in_sequence(self, sequence: List[List[str]]) -> List[List[str]]:
        """Process parentheses within a sequence"""
        result = []
        i = 0
        
        while i < len(sequence):
            token = sequence[i]
            
            if len(token) == 2 and token[0] == "group_open":
                # Find matching close
                paren_count = 1
                j = i + 1
                group_content = []
                
                while j < len(sequence) and paren_count > 0:
                    if len(sequence[j]) == 2:
                        if sequence[j][0] == "group_open":
                            paren_count += 1
                        elif sequence[j][0] == "group_close":
                            paren_count -= 1
                    
                    if paren_count > 0:
                        group_content.append(sequence[j])
                    j += 1
                
                # Add grouped content as nested structure
                if group_content:
                    result.append(["group", group_content])
                
                i = j
            else:
                result.append(token)
                i += 1
        
        return result
    
    def _parse_sequences(self, processed_rules: Dict[str, List[List[List[str]]]]) -> Dict[str, List[ASTNode]]:
        """Parse sequences of grammar elements (Step 3)"""
        if self.debug:
            print("Step 3: Parsing sequences...")
        
        sequenced = {}
        
        for rule_name, alternatives in processed_rules.items():
            parsed_alts = []
            
            for alt in alternatives:
                if len(alt) == 1:
                    # Single element
                    parsed_alts.append(self._parse_single_element(alt[0]))
                else:
                    # Sequence of elements
                    elements = [self._parse_single_element(elem) for elem in alt]
                    parsed_alts.append(ASTNode("sequence", elements=elements))
            
            sequenced[rule_name] = parsed_alts
        
        return sequenced
    
    def _parse_single_element(self, element: List[str]) -> ASTNode:
        """Parse a single grammar element"""
        if len(element) != 2:
            return ASTNode("atom", value=element)
        
        token_type, token_value = element
        
        if token_type == "group":
            # Handle grouped elements (from parentheses processing)
            if isinstance(token_value, list):
                if len(token_value) == 1:
                    return self._parse_single_element(token_value[0])
                else:
                    elements = [self._parse_single_element(elem) for elem in token_value]
                    return ASTNode("sequence", elements=elements)
        
        return ASTNode("atom", value=[token_type, token_value])
    
    def _handle_quantifiers(self, sequenced_rules: Dict[str, List[ASTNode]]) -> Dict[str, List[ASTNode]]:
        """Handle quantifiers (*, +, ?) (Step 4)"""
        if self.debug:
            print("Step 4: Handling quantifiers...")
        
        quantified = {}
        
        for rule_name, alternatives in sequenced_rules.items():
            processed_alts = []
            
            for alt in alternatives:
                processed_alt = self._apply_quantifiers_to_node(alt)
                processed_alts.append(processed_alt)
            
            quantified[rule_name] = processed_alts
        
        return quantified
    
    def _apply_quantifiers_to_node(self, node: ASTNode) -> ASTNode:
        """Apply quantifiers to AST node"""
        if node.type == "sequence":
            # Process elements in sequence and look for quantifiers
            new_elements = []
            i = 0
            
            while i < len(node.elements):
                element = node.elements[i]
                
                # Check if next token is a quantifier
                if (i + 1 < len(node.elements) and 
                    hasattr(node.elements[i + 1], 'value') and
                    isinstance(node.elements[i + 1].value, list) and
                    len(node.elements[i + 1].value) == 2 and
                    node.elements[i + 1].value[0] == "operator" and
                    node.elements[i + 1].value[1] in ["*", "+", "?"]):
                    
                    quantifier = node.elements[i + 1].value[1]
                    quantified_node = ASTNode("quantified", 
                                            element=element, 
                                            quantifier=quantifier)
                    new_elements.append(quantified_node)
                    i += 2  # Skip quantifier token
                else:
                    new_elements.append(element)
                    i += 1
            
            return ASTNode("sequence", elements=new_elements)
        
        return node
    
    def _build_tree_structure(self, quantified_rules: Dict[str, List[ASTNode]]) -> Tuple[Dict[str, ASTNode], List[str]]:
        """Build final tree structure (Step 5)"""
        if self.debug:
            print("Step 5: Building tree structure...")
        
        grammar_tree = {}
        rule_order = list(quantified_rules.keys())
        
        for rule_name, alternatives in quantified_rules.items():
            if len(alternatives) == 1:
                # Single alternative
                grammar_tree[rule_name] = alternatives[0]
            else:
                # Multiple alternatives - create OR node
                grammar_tree[rule_name] = ASTNode("or", alternatives=alternatives)
        
        return grammar_tree, rule_order
    
    def save_transformed_ast(self, 
                           grammar_tree: Dict[str, ASTNode], 
                           rule_order: List[str],
                           grammar_name: str,
                           output_file: str):
        """Save transformed AST to JSON file"""
        if self.debug:
            print(f"Saving transformed AST to: {output_file}")
        
        # Convert AST nodes to dictionaries
        tree_dict = {name: node.to_dict() for name, node in grammar_tree.items()}
        
        transformed_data = {
            "grammar_name": grammar_name,
            "grammar_tree": tree_dict,
            "rule_order": rule_order,
            "metadata": {
                "format": "transformed_ast",
                "source_format": "raw_ast",
                "transformed_at": self._current_timestamp(),
                "transformer": "Python AST Pipeline v1.0",
                "pipeline_stage": "transformation",
                "annotations": self.annotations,
                "stats": self.stats
            }
        }
        
        with open(output_file, 'w') as f:
            json.dump(transformed_data, f, indent=2, sort_keys=True)
        
        if self.debug:
            print(f"Transformed AST saved successfully")
    
    def _current_timestamp(self) -> str:
        """Get current timestamp in ISO format"""
        from datetime import datetime
        return datetime.now().isoformat()
    
    # --- Dual-Mode API ---
    
    def transform_from_file(self, raw_ast_json_file: str, 
                          output_json_file: Optional[str] = None) -> Tuple[Dict[str, ASTNode], List[str]]:
        """
        Same-Language API: Transform raw AST JSON file to in-memory AST
        Optionally save transformed JSON for cross-language usage
        """
        # Load raw AST
        raw_data = self.load_raw_ast(raw_ast_json_file)
        
        # Transform in-memory
        grammar_tree, rule_order = self.transform_raw_ast(raw_data['raw_ast'])
        
        # Optionally save JSON for cross-language usage
        if output_json_file:
            self.save_transformed_ast(grammar_tree, rule_order, 
                                    raw_data['grammar_name'], output_json_file)
        
        return grammar_tree, rule_order
    
    def transform_to_json(self, raw_ast_json_file: str, output_json_file: str):
        """
        Cross-Language API: Transform raw AST JSON → transformed AST JSON
        """
        grammar_tree, rule_order = self.transform_from_file(raw_ast_json_file)
        
        # Load raw data for grammar name
        raw_data = self.load_raw_ast(raw_ast_json_file)
        
        self.save_transformed_ast(grammar_tree, rule_order, 
                                raw_data['grammar_name'], output_json_file)


def main():
    """Command-line interface for Python AST Pipeline"""
    import argparse
    
    parser = argparse.ArgumentParser(
        description="Python AST Transformation Pipeline"
    )
    parser.add_argument("input_json", help="Raw AST JSON file")
    parser.add_argument("output_json", nargs='?', help="Output transformed AST JSON file")
    parser.add_argument("--debug", "-d", action="store_true", help="Enable debug output")
    parser.add_argument("--stats", "-s", action="store_true", help="Show transformation statistics")
    
    args = parser.parse_args()
    
    # Create pipeline
    config = PipelineConfig(debug=args.debug)
    pipeline = PythonASTPipeline(config)
    
    try:
        if args.output_json:
            # Cross-language mode: JSON → JSON
            pipeline.transform_to_json(args.input_json, args.output_json)
            print(f"Transformed AST saved to: {args.output_json}")
        else:
            # Same-language mode: JSON → In-memory
            grammar_tree, rule_order = pipeline.transform_from_file(args.input_json)
            print(f"Transformed AST loaded in-memory: {len(grammar_tree)} rules")
            print(f"Rule order: {', '.join(rule_order)}")
        
        # Show statistics
        if args.stats:
            print("\nTransformation Statistics:")
            for key, value in pipeline.stats.items():
                print(f"  {key.replace('_', ' ').title()}: {value}")
            
            if pipeline.annotations['semantic_annotations']:
                print(f"  Semantic annotations: {len(pipeline.annotations['semantic_annotations'])} rules")
            if pipeline.annotations['logging_annotations']:
                print(f"  Logging annotations: {len(pipeline.annotations['logging_annotations'])} rules")
    
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
