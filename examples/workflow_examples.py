#!/usr/bin/env python3
"""
Multi-Language Workflow Examples

Demonstrates the flexible architecture with concrete examples:
1. Same-language workflow (Python pipeline + Python generator)
2. Cross-language workflow (Python pipeline → JSON → other generators)
3. Performance comparison between approaches
4. Integration with existing Perl pipeline

This serves as both documentation and integration testing.
"""

import os
import sys
import time
import json
from pathlib import Path

# Add python directory to path for imports
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'python'))

from ast_pipeline import PythonASTPipeline, PipelineConfig

# Add tools directory for existing data generator
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'tools'))

class WorkflowExamples:
    """Demonstrates various workflow patterns"""
    
    def __init__(self, base_dir: str):
        self.base_dir = Path(base_dir)
        self.examples_dir = self.base_dir / "examples" / "temp"
        self.examples_dir.mkdir(exist_ok=True, parents=True)
        
        # Performance tracking
        self.timings = {}
    
    def setup_test_data(self):
        """Create test raw AST JSON for examples"""
        print("=== Setting Up Test Data ===")
        
        # Simple arithmetic grammar
        simple_raw_ast = {
            "grammar_name": "simple_arithmetic",
            "raw_ast": [
                [
                    ["rule", "expression"],
                    ["rule_reference", "term"],
                    ["group_open", "("],
                    ["quoted_string", "+"],
                    ["rule_reference", "term"],
                    ["group_close", ")"],
                    ["quantifier", "*"]
                ],
                [
                    ["rule", "term"],
                    ["regex", "(\\d+)"]
                ]
            ],
            "metadata": {
                "source_file": "simple_arithmetic.ebnf",
                "format": "raw_ast",
                "generated_at": "2025-09-02T02:00:00Z",
                "parser": "ebnf_to_json.pl",
                "rule_count": 2
            }
        }
        
        # Complex grammar with annotations
        complex_raw_ast = {
            "grammar_name": "annotated_calculator",
            "raw_ast": [
                [
                    ["rule", "calculator"],
                    ["semantic_annotation", "@type:calculator"],
                    ["semantic_annotation", "@description:Simple calculator"],
                    ["rule_reference", "expression"]
                ],
                [
                    ["rule", "expression"],
                    ["logging_annotation", "@log:expression_parsing"],
                    ["rule_reference", "term"],
                    ["group_open", "("],
                    ["quoted_string", "+"],
                    ["operator", "|"],
                    ["quoted_string", "-"],
                    ["group_close", ")"],
                    ["rule_reference", "term"],
                    ["quantifier", "*"],
                    ["return_object", "return_object"]
                ],
                [
                    ["rule", "term"],
                    ["rule_reference", "factor"],
                    ["group_open", "("],
                    ["quoted_string", "*"],
                    ["operator", "|"],
                    ["quoted_string", "/"],
                    ["group_close", ")"],
                    ["rule_reference", "factor"],
                    ["quantifier", "*"]
                ],
                [
                    ["rule", "factor"],
                    ["group_open", "("],
                    ["quoted_string", "("],
                    ["rule_reference", "expression"],
                    ["quoted_string", ")"],
                    ["group_close", ")"],
                    ["operator", "|"],
                    ["rule_reference", "number"]
                ],
                [
                    ["rule", "number"],
                    ["regex", "\\d+(\\.\\d+)?"],
                    ["return_scalar", "return_scalar"]
                ]
            ],
            "metadata": {
                "source_file": "calculator.ebnf",
                "format": "raw_ast",
                "generated_at": "2025-09-02T02:00:00Z",
                "parser": "ebnf_to_json.pl",
                "rule_count": 5
            }
        }
        
        # Save test files
        simple_file = self.examples_dir / "simple_raw.json"
        complex_file = self.examples_dir / "complex_raw.json"
        
        with open(simple_file, 'w') as f:
            json.dump(simple_raw_ast, f, indent=2)
        
        with open(complex_file, 'w') as f:
            json.dump(complex_raw_ast, f, indent=2)
        
        print(f"Created test files:")
        print(f"  - {simple_file}")
        print(f"  - {complex_file}")
        
        return simple_file, complex_file
    
    def example_1_same_language_workflow(self, raw_file: Path):
        """Example 1: Same-language optimization (Python throughout)"""
        print("\n=== Example 1: Same-Language Workflow (Python) ===")
        print("Demonstrates: In-memory optimization when using same language")
        
        start_time = time.time()
        
        # Step 1: Initialize Python pipeline
        config = PipelineConfig(debug=False, preserve_annotations=True)
        pipeline = PythonASTPipeline(config)
        
        # Step 2: Transform raw AST (JSON → in-memory)
        print(f"Loading and transforming: {raw_file}")
        grammar_tree, rule_order = pipeline.transform_from_file(str(raw_file))
        
        transform_time = time.time()
        
        # Step 3: Generate data directly from in-memory AST
        print("Generating test data from in-memory AST...")
        
        # Import existing data generator
        sys.path.append(str(self.base_dir / "tools"))
        from syntactic_data_generator import SyntacticDataGenerator, GeneratorConfig
        
        # Create compatible data structures for existing generator
        # Convert ASTNode objects to dict format expected by generator
        grammar_dict = {}\n        for name, node in grammar_tree.items():\n            grammar_dict[name] = node.to_dict()\n        \n        gen_config = GeneratorConfig(seed=42, max_depth=8)\n        generator = SyntacticDataGenerator(grammar_dict, rule_order, gen_config)\n        \n        # Generate samples\n        samples = generator.generate(start_rule=rule_order[0], count=3)\n        \n        generation_time = time.time()\n        \n        # Results\n        print(f\"\\nResults:\")\n        print(f\"  Grammar: {len(grammar_tree)} rules\")\n        print(f\"  Samples generated: {len(samples)}\")\n        print(f\"  Annotations preserved: {len(pipeline.annotations['semantic_annotations'])}\")\n        \n        for i, sample in enumerate(samples, 1):\n            print(f\"    Sample {i}: {sample[:50]}{'...' if len(sample) > 50 else ''}\")\n        \n        # Performance metrics\n        total_time = generation_time - start_time\n        transform_duration = transform_time - start_time\n        generation_duration = generation_time - transform_time\n        \n        self.timings['same_language'] = {\n            'total': total_time,\n            'transform': transform_duration, \n            'generate': generation_duration,\n            'samples': len(samples)\n        }\n        \n        print(f\"\\nPerformance (Same-Language):\")\n        print(f\"  Transform time: {transform_duration:.4f}s\")\n        print(f\"  Generation time: {generation_duration:.4f}s\") \n        print(f\"  Total time: {total_time:.4f}s\")\n        print(f\"  Benefit: No JSON serialization overhead\")\n        \n        return samples, grammar_tree, rule_order\n    \n    def example_2_cross_language_workflow(self, raw_file: Path):\n        \"\"\"Example 2: Cross-language workflow (Python → JSON → other tools)\"\"\"\n        print(\"\\n=== Example 2: Cross-Language Workflow ===)\") \n        print(\"Demonstrates: JSON interface for language interoperability\")\n        \n        start_time = time.time()\n        \n        # Step 1: Python pipeline transforms raw → transformed JSON\n        config = PipelineConfig(debug=False, preserve_annotations=True)\n        pipeline = PythonASTPipeline(config)\n        \n        transformed_file = self.examples_dir / \"transformed_cross_lang.json\"\n        \n        print(f\"Transforming: {raw_file} → {transformed_file}\")\n        pipeline.transform_to_json(str(raw_file), str(transformed_file))\n        \n        json_save_time = time.time()\n        \n        # Step 2: Use existing tools via JSON interface\n        print(f\"Generating data via JSON interface...\")\n        \n        # Use command-line interface of existing data generator\n        import subprocess\n        \n        output_file = self.examples_dir / \"cross_lang_output.txt\"\n        cmd = [\n            sys.executable, \n            str(self.base_dir / \"tools\" / \"syntactic_data_generator.py\"),\n            str(transformed_file),\n            \"--rule\", \"expression\", \n            \"--count\", \"3\",\n            \"--output\", str(output_file)\n        ]\n        \n        result = subprocess.run(cmd, capture_output=True, text=True)\n        \n        generation_time = time.time()\n        \n        # Step 3: Read generated output\n        if output_file.exists():\n            with open(output_file, 'r') as f:\n                output_content = f.read()\n            \n            samples = []\n            for line in output_content.split('\\n'):\n                if line.strip() and not line.startswith('#'):\n                    samples.append(line.strip())\n        else:\n            samples = result.stdout.strip().split('\\n') if result.stdout else []\n        \n        # Performance metrics  \n        total_time = generation_time - start_time\n        transform_duration = json_save_time - start_time\n        generation_duration = generation_time - json_save_time\n        \n        self.timings['cross_language'] = {\n            'total': total_time,\n            'transform': transform_duration,\n            'generate': generation_duration, \n            'samples': len([s for s in samples if s])\n        }\n        \n        # Results\n        print(f\"\\nResults:\")\n        print(f\"  Transformed JSON saved: {transformed_file}\")\n        print(f\"  Samples generated: {len([s for s in samples if s])}\")\n        \n        valid_samples = [s for s in samples if s and not s.startswith('#')]\n        for i, sample in enumerate(valid_samples[:3], 1):\n            print(f\"    Sample {i}: {sample[:50]}{'...' if len(sample) > 50 else ''}\")\n        \n        print(f\"\\nPerformance (Cross-Language):\")\n        print(f\"  Transform + JSON save: {transform_duration:.4f}s\")\n        print(f\"  JSON-based generation: {generation_duration:.4f}s\") \n        print(f\"  Total time: {total_time:.4f}s\")\n        print(f\"  Cost: JSON serialization + subprocess overhead\")\n        \n        return samples, transformed_file\n    \n    def example_3_perl_integration(self):\n        \"\"\"Example 3: Integration with existing Perl pipeline\"\"\"\n        print(\"\\n=== Example 3: Perl Integration Workflow ===\")\n        print(\"Demonstrates: Using existing Perl tools in multi-language pipeline\")\n        \n        # Use semantic annotations grammar if available\n        semantic_file = self.base_dir / \"semantic_annotations_raw.json\"\n        \n        if not semantic_file.exists():\n            print(f\"Skipping Perl integration - {semantic_file} not found\")\n            print(\"Run: perl tools/ebnf_to_json.pl grammars/semantic_annotations.ebnf > semantic_annotations_raw.json\")\n            return\n        \n        start_time = time.time()\n        \n        # Step 1: Use existing Perl transformation\n        perl_transformed = self.examples_dir / \"perl_transformed.json\"\n        \n        print(f\"Using Perl pipeline: {semantic_file} → {perl_transformed}\")\n        \n        import subprocess\n        perl_cmd = [\n            \"perl\", \n            str(self.base_dir / \"tools\" / \"transform_ast.pl\"),\n            str(semantic_file),\n            str(perl_transformed)\n        ]\n        \n        perl_result = subprocess.run(perl_cmd, capture_output=True, text=True)\n        \n        if perl_result.returncode != 0:\n            print(f\"Perl transformation failed: {perl_result.stderr}\")\n            return\n        \n        perl_time = time.time()\n        \n        # Step 2: Python generator consumes Perl-transformed JSON\n        print(f\"Python generator consuming Perl-transformed AST...\")\n        \n        python_cmd = [\n            sys.executable,\n            str(self.base_dir / \"tools\" / \"syntactic_data_generator.py\"),\n            str(perl_transformed),\n            \"--rule\", \"semantic_annotation\",\n            \"--count\", \"2\"\n        ]\n        \n        python_result = subprocess.run(python_cmd, capture_output=True, text=True)\n        \n        generation_time = time.time()\n        \n        # Results\n        total_time = generation_time - start_time\n        perl_duration = perl_time - start_time\n        python_duration = generation_time - perl_time\n        \n        print(f\"\\nResults:\")\n        print(f\"  Perl transformation: SUCCESS\") \n        print(f\"  Python generation: {'SUCCESS' if python_result.returncode == 0 else 'FAILED'}\")\n        \n        if python_result.stdout:\n            print(f\"  Generated output:\")\n            for line in python_result.stdout.strip().split('\\n')[:5]:\n                if line.strip() and not line.startswith('#'):\n                    print(f\"    {line}\")\n        \n        print(f\"\\nPerformance (Perl→Python):\")\n        print(f\"  Perl transform: {perl_duration:.4f}s\")\n        print(f\"  Python generate: {python_duration:.4f}s\")\n        print(f\"  Total: {total_time:.4f}s\")\n        print(f\"  Benefit: Best-of-breed tools per language\")\n    \n    def performance_comparison(self):\n        \"\"\"Compare performance of different workflow approaches\"\"\"\n        print(\"\\n=== Performance Comparison ===\")\n        \n        if 'same_language' in self.timings and 'cross_language' in self.timings:\n            same = self.timings['same_language']\n            cross = self.timings['cross_language']\n            \n            print(\"\\nTiming Comparison:\")\n            print(f\"  {'Metric':<20} {'Same-Lang':<12} {'Cross-Lang':<12} {'Difference':<15}\")\n            print(f\"  {'-'*20} {'-'*12} {'-'*12} {'-'*15}\")\n            \n            total_diff = ((cross['total'] - same['total']) / same['total']) * 100\n            transform_diff = ((cross['transform'] - same['transform']) / same['transform']) * 100\n            \n            print(f\"  {'Total Time':<20} {same['total']:<12.4f} {cross['total']:<12.4f} {total_diff:>+7.1f}%\")\n            print(f\"  {'Transform Time':<20} {same['transform']:<12.4f} {cross['transform']:<12.4f} {transform_diff:>+7.1f}%\")\n            print(f\"  {'Generate Time':<20} {same['generate']:<12.4f} {cross['generate']:<12.4f} {'N/A':<15}\")\n            \n            print(\"\\nRecommendations:\")\n            if total_diff < 20:\n                print(\"  ✓ Cross-language overhead is minimal - use best tools for each task\")\n            else:\n                print(\"  ⚠ Significant cross-language overhead - consider same-language for performance-critical tasks\")\n            \n            print(f\"  ✓ Same-language: Optimal for {same['samples']} samples in {same['total']:.4f}s\")\n            print(f\"  ✓ Cross-language: Flexibility for {cross['samples']} samples in {cross['total']:.4f}s\")\n    \n    def cleanup(self):\n        \"\"\"Clean up temporary files\"\"\"\n        print(\"\\n=== Cleaning Up ===\")\n        \n        import shutil\n        if self.examples_dir.exists():\n            shutil.rmtree(self.examples_dir)\n            print(f\"Removed temporary directory: {self.examples_dir}\")\n\n\ndef main():\n    \"\"\"Run all workflow examples\"\"\"\n    print(\"Multi-Language EBNF Parser Generator Workflow Examples\")\n    print(\"=\" * 60)\n    \n    # Get base directory\n    base_dir = Path(__file__).parent.parent\n    \n    # Initialize examples\n    examples = WorkflowExamples(str(base_dir))\n    \n    try:\n        # Setup test data\n        simple_file, complex_file = examples.setup_test_data()\n        \n        # Run examples\n        examples.example_1_same_language_workflow(simple_file)\n        examples.example_2_cross_language_workflow(complex_file) \n        examples.example_3_perl_integration()\n        \n        # Performance analysis\n        examples.performance_comparison()\n        \n        print(\"\\n=== Summary ===\")\n        print(\"\")\n        print(\"This demonstration shows three workflow patterns:\")\n        print(\"\")\n        print(\"1. **Same-Language Optimization**:\")\n        print(\"   - Use when performance is critical\")\n        print(\"   - In-memory data structures avoid serialization overhead\")\n        print(\"   - Best for production pipelines in single language\")\n        print(\"\")\n        print(\"2. **Cross-Language Flexibility**:\")\n        print(\"   - Use when leveraging best-of-breed tools\")\n        print(\"   - JSON interface enables language interoperability\")\n        print(\"   - Best for mixed development teams or specialized tools\")\n        print(\"\")\n        print(\"3. **Legacy Integration**:\")\n        print(\"   - Use existing Perl tools with new language implementations\")\n        print(\"   - Gradual migration path from existing systems\")\n        print(\"   - Best for organizations with existing EBNF toolchains\")\n        print(\"\")\n        print(\"Choose the pattern that best fits your requirements!\")\n        \n    finally:\n        # Cleanup\n        examples.cleanup()\n\n\nif __name__ == \"__main__\":\n    main()
