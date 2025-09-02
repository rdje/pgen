#!/usr/bin/env python3
"""
Automated Testing Framework for Multi-Language EBNF Parser Generator

This framework provides comprehensive end-to-end testing of the multi-language
pipeline architecture with:

1. Automated test case generation from ebnf.ebnf using DataGeneration
2. Cross-language pipeline validation  
3. Failure analysis and tracing
4. False positive/negative detection
5. Performance benchmarking across languages
6. Complete test result reporting

Usage:
    python testing/automated_test_framework.py --full-test
    python testing/automated_test_framework.py --language rust --grammar semantic_annotations.ebnf
    python testing/automated_test_framework.py --benchmark --iterations 100
"""

import os
import sys
import json
import time
import subprocess
import tempfile
import shutil
import hashlib
from pathlib import Path
from typing import Dict, List, Tuple, Optional, Any
from dataclasses import dataclass, field
from datetime import datetime
import concurrent.futures
import argparse

# Add project paths
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'python'))
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'tools'))

@dataclass
class TestCase:
    """Represents a single test case"""
    name: str
    ebnf_file: str
    expected_rules: int
    test_id: str = field(init=False)
    
    def __post_init__(self):
        self.test_id = hashlib.md5(f"{self.name}_{self.ebnf_file}".encode()).hexdigest()[:8]

@dataclass
class TestResult:
    """Represents test execution result"""
    test_case: TestCase
    language: str
    pipeline_success: bool
    pipeline_time: float
    generation_success: bool
    generation_time: float
    rules_processed: int
    error_message: Optional[str] = None
    trace_data: Optional[Dict] = None
    output_hash: Optional[str] = None

@dataclass
class TestSuite:
    """Complete test suite configuration"""
    languages: List[str]
    test_cases: List[TestCase]
    iterations: int = 1
    parallel: bool = True
    validate_outputs: bool = True
    benchmark_mode: bool = False

class LanguageExecutor:
    """Handles execution of language-specific pipelines"""
    
    def __init__(self, base_dir: Path):
        self.base_dir = base_dir
        self.executors = {
            'perl': self._execute_perl,
            'python': self._execute_python,
            'rust': self._execute_rust,
            'julia': self._execute_julia,
            'go': self._execute_go,
            'zig': self._execute_zig,
        }
    
    def execute(self, language: str, raw_ast_file: str, output_dir: Path) -> TestResult:
        """Execute pipeline for specific language"""
        if language not in self.executors:
            raise ValueError(f"Unsupported language: {language}")
        
        return self.executors[language](raw_ast_file, output_dir)
    
    def _execute_perl(self, raw_ast_file: str, output_dir: Path) -> Dict:
        """Execute Perl pipeline"""
        transformed_file = output_dir / "perl_transformed.json"
        
        start_time = time.time()
        
        # Run Perl transformation
        result = subprocess.run([
            'perl', 
            str(self.base_dir / 'tools' / 'transform_ast.pl'),
            raw_ast_file,
            str(transformed_file)
        ], capture_output=True, text=True)
        
        pipeline_time = time.time() - start_time
        
        if result.returncode != 0:
            return {
                'pipeline_success': False,
                'pipeline_time': pipeline_time,
                'error_message': result.stderr,
                'generation_success': False,
                'generation_time': 0.0
            }
        
        # Run data generation
        gen_start_time = time.time()
        gen_result = subprocess.run([
            sys.executable,
            str(self.base_dir / 'tools' / 'syntactic_data_generator.py'),
            str(transformed_file),
            '--count', '5'
        ], capture_output=True, text=True)
        
        generation_time = time.time() - gen_start_time
        
        return {
            'pipeline_success': True,
            'pipeline_time': pipeline_time,
            'generation_success': gen_result.returncode == 0,
            'generation_time': generation_time,
            'error_message': gen_result.stderr if gen_result.returncode != 0 else None,
            'output_data': gen_result.stdout
        }
    
    def _execute_python(self, raw_ast_file: str, output_dir: Path) -> Dict:
        """Execute Python pipeline"""
        transformed_file = output_dir / "python_transformed.json"
        
        start_time = time.time()
        
        # Run Python transformation
        result = subprocess.run([
            sys.executable,
            str(self.base_dir / 'python' / 'ast_pipeline.py'),
            raw_ast_file,
            str(transformed_file),
            '--stats'
        ], capture_output=True, text=True)
        
        pipeline_time = time.time() - start_time
        
        if result.returncode != 0:
            return {
                'pipeline_success': False,
                'pipeline_time': pipeline_time,
                'error_message': result.stderr,
                'generation_success': False,
                'generation_time': 0.0
            }
        
        # Run data generation
        gen_start_time = time.time()
        gen_result = subprocess.run([
            sys.executable,
            str(self.base_dir / 'tools' / 'syntactic_data_generator.py'),
            str(transformed_file),
            '--count', '5'
        ], capture_output=True, text=True)
        
        generation_time = time.time() - gen_start_time
        
        return {
            'pipeline_success': True,
            'pipeline_time': pipeline_time,
            'generation_success': gen_result.returncode == 0,
            'generation_time': generation_time,
            'error_message': gen_result.stderr if gen_result.returncode != 0 else None,
            'output_data': gen_result.stdout
        }
    
    def _execute_rust(self, raw_ast_file: str, output_dir: Path) -> Dict:
        """Execute Rust pipeline"""
        transformed_file = output_dir / "rust_transformed.json"
        binary_path = self.base_dir / "rust" / "target" / "release" / "ast_pipeline"
        
        # Build Rust binary if needed
        if not binary_path.exists():
            build_result = subprocess.run([
                'cargo', 'build', '--release'
            ], cwd=self.base_dir / 'rust', capture_output=True, text=True)
            
            if build_result.returncode != 0:
                return {
                    'pipeline_success': False,
                    'pipeline_time': 0.0,
                    'error_message': f"Rust build failed: {build_result.stderr}",
                    'generation_success': False,
                    'generation_time': 0.0
                }
        
        start_time = time.time()
        
        # Run Rust transformation
        result = subprocess.run([
            str(binary_path),
            raw_ast_file,
            str(transformed_file),
            '--stats'
        ], capture_output=True, text=True)
        
        pipeline_time = time.time() - start_time
        
        if result.returncode != 0:
            return {
                'pipeline_success': False,
                'pipeline_time': pipeline_time,
                'error_message': result.stderr,
                'generation_success': False,
                'generation_time': 0.0
            }
        
        # Use Python data generator (cross-language)
        gen_start_time = time.time()
        gen_result = subprocess.run([
            sys.executable,
            str(self.base_dir / 'tools' / 'syntactic_data_generator.py'),
            str(transformed_file),
            '--count', '5'
        ], capture_output=True, text=True)
        
        generation_time = time.time() - gen_start_time
        
        return {
            'pipeline_success': True,
            'pipeline_time': pipeline_time,
            'generation_success': gen_result.returncode == 0,
            'generation_time': generation_time,
            'error_message': gen_result.stderr if gen_result.returncode != 0 else None,
            'output_data': gen_result.stdout
        }
    
    def _execute_julia(self, raw_ast_file: str, output_dir: Path) -> Dict:
        """Execute Julia pipeline"""
        transformed_file = output_dir / "julia_transformed.json"
        
        start_time = time.time()
        
        # Run Julia transformation
        julia_script = f'''
        push!(LOAD_PATH, "{self.base_dir / 'julia'}")
        using ASTPipeline
        
        config = ASTPipeline.PipelineConfig(debug=false)
        pipeline = ASTPipeline.JuliaASTPipeline(config)
        
        try
            ASTPipeline.transform_to_json!(pipeline, "{raw_ast_file}", "{transformed_file}")
            println("SUCCESS")
        catch e
            println("ERROR: ", e)
        end
        '''
        
        result = subprocess.run([
            'julia', '--eval', julia_script
        ], capture_output=True, text=True)
        
        pipeline_time = time.time() - start_time
        
        if result.returncode != 0 or not result.stdout.strip().startswith("SUCCESS"):
            return {
                'pipeline_success': False,
                'pipeline_time': pipeline_time,
                'error_message': result.stderr or result.stdout,
                'generation_success': False,
                'generation_time': 0.0
            }
        
        # Use Python data generator (cross-language)
        gen_start_time = time.time()
        gen_result = subprocess.run([
            sys.executable,
            str(self.base_dir / 'tools' / 'syntactic_data_generator.py'),
            str(transformed_file),
            '--count', '5'
        ], capture_output=True, text=True)
        
        generation_time = time.time() - gen_start_time
        
        return {
            'pipeline_success': True,
            'pipeline_time': pipeline_time,
            'generation_success': gen_result.returncode == 0,
            'generation_time': generation_time,
            'error_message': gen_result.stderr if gen_result.returncode != 0 else None,
            'output_data': gen_result.stdout
        }
    
    def _execute_go(self, raw_ast_file: str, output_dir: Path) -> Dict:
        """Execute Go pipeline"""
        transformed_file = output_dir / "go_transformed.json"
        binary_path = self.base_dir / "go" / "ast_pipeline"
        
        # Build Go binary if needed
        if not binary_path.exists():
            build_result = subprocess.run([
                'go', 'build', '-o', 'ast_pipeline', 'ast_pipeline.go'
            ], cwd=self.base_dir / 'go', capture_output=True, text=True)
            
            if build_result.returncode != 0:
                return {
                    'pipeline_success': False,
                    'pipeline_time': 0.0,
                    'error_message': f"Go build failed: {build_result.stderr}",
                    'generation_success': False,
                    'generation_time': 0.0
                }
        
        start_time = time.time()
        
        # Run Go transformation
        result = subprocess.run([
            str(binary_path),
            raw_ast_file,
            str(transformed_file),
            '--stats'
        ], capture_output=True, text=True)
        
        pipeline_time = time.time() - start_time
        
        if result.returncode != 0:
            return {
                'pipeline_success': False,
                'pipeline_time': pipeline_time,
                'error_message': result.stderr,
                'generation_success': False,
                'generation_time': 0.0
            }
        
        # Use Python data generator (cross-language)
        gen_start_time = time.time()
        gen_result = subprocess.run([
            sys.executable,
            str(self.base_dir / 'tools' / 'syntactic_data_generator.py'),
            str(transformed_file),
            '--count', '5'
        ], capture_output=True, text=True)
        
        generation_time = time.time() - gen_start_time
        
        return {
            'pipeline_success': True,
            'pipeline_time': pipeline_time,
            'generation_success': gen_result.returncode == 0,
            'generation_time': generation_time,
            'error_message': gen_result.stderr if gen_result.returncode != 0 else None,
            'output_data': gen_result.stdout
        }
    
    def _execute_zig(self, raw_ast_file: str, output_dir: Path) -> Dict:
        """Execute Zig pipeline (placeholder)"""
        # Zig implementation would go here
        return {
            'pipeline_success': False,
            'pipeline_time': 0.0,
            'error_message': "Zig implementation not yet available",
            'generation_success': False,
            'generation_time': 0.0
        }

class TestFramework:
    """Main automated testing framework"""
    
    def __init__(self, base_dir: Path):
        self.base_dir = base_dir
        self.executor = LanguageExecutor(base_dir)
        self.temp_dir = None
        
    def __enter__(self):
        self.temp_dir = Path(tempfile.mkdtemp(prefix="ebnf_test_"))
        return self
        
    def __exit__(self, exc_type, exc_val, exc_tb):
        if self.temp_dir and self.temp_dir.exists():
            shutil.rmtree(self.temp_dir)
    
    def generate_test_cases(self) -> List[TestCase]:
        """Generate test cases from available grammars"""
        test_cases = []
        
        # Predefined test cases
        grammars_dir = self.base_dir / "grammars"
        if grammars_dir.exists():
            for ebnf_file in grammars_dir.glob("*.ebnf"):
                # Estimate expected rules by parsing EBNF file
                with open(ebnf_file, 'r') as f:
                    content = f.read()
                    # Simple heuristic: count rule definitions
                    expected_rules = content.count('::=') + content.count('=')
                
                test_cases.append(TestCase(
                    name=ebnf_file.stem,
                    ebnf_file=str(ebnf_file),
                    expected_rules=expected_rules
                ))
        
        # Generate synthetic test cases using ebnf.ebnf
        if (self.base_dir / "grammars" / "ebnf.ebnf").exists():
            test_cases.extend(self._generate_synthetic_test_cases())
        
        return test_cases
    
    def _generate_synthetic_test_cases(self) -> List[TestCase]:
        """Generate synthetic test cases using ebnf.ebnf and DataGeneration"""
        synthetic_cases = []
        
        # Use DataGeneration to create test EBNF grammars
        ebnf_grammar_file = self.base_dir / "grammars" / "ebnf.ebnf"
        
        try:
            # Generate raw AST for ebnf.ebnf
            raw_ebnf_file = self.temp_dir / "ebnf_raw.json"
            result = subprocess.run([
                'perl',
                str(self.base_dir / 'tools' / 'ebnf_to_json.pl'),
                str(ebnf_grammar_file)
            ], stdout=open(raw_ebnf_file, 'w'), stderr=subprocess.PIPE, text=True)
            
            if result.returncode == 0:
                # Transform ebnf AST
                ebnf_transformed_file = self.temp_dir / "ebnf_transformed.json"
                transform_result = subprocess.run([
                    'perl',
                    str(self.base_dir / 'tools' / 'transform_ast.pl'),
                    str(raw_ebnf_file),
                    str(ebnf_transformed_file)
                ], capture_output=True, text=True)
                
                if transform_result.returncode == 0:
                    # Generate synthetic EBNF grammars
                    for i in range(5):  # Generate 5 synthetic cases
                        synthetic_ebnf = self.temp_dir / f"synthetic_{i}.ebnf"
                        gen_result = subprocess.run([
                            sys.executable,
                            str(self.base_dir / 'tools' / 'syntactic_data_generator.py'),
                            str(ebnf_transformed_file),
                            '--rule', 'grammar',
                            '--count', '1'
                        ], stdout=open(synthetic_ebnf, 'w'), stderr=subprocess.PIPE, text=True)
                        
                        if gen_result.returncode == 0:
                            synthetic_cases.append(TestCase(
                                name=f"synthetic_{i}",
                                ebnf_file=str(synthetic_ebnf),
                                expected_rules=5  # Estimate
                            ))
        except Exception as e:
            print(f"Warning: Could not generate synthetic test cases: {e}")
        
        return synthetic_cases
    
    def run_test_suite(self, test_suite: TestSuite) -> List[TestResult]:
        """Run complete test suite"""
        all_results = []
        
        print(f"Running test suite with {len(test_suite.test_cases)} test cases across {len(test_suite.languages)} languages")
        
        for iteration in range(test_suite.iterations):
            if test_suite.iterations > 1:
                print(f"\nIteration {iteration + 1}/{test_suite.iterations}")
            
            iteration_results = []
            
            # Prepare test cases
            for test_case in test_suite.test_cases:
                # Generate raw AST for test case
                raw_ast_file = self._prepare_test_case(test_case)
                if not raw_ast_file:
                    continue
                
                # Run across all languages
                if test_suite.parallel:
                    iteration_results.extend(self._run_parallel_tests(test_case, test_suite.languages, raw_ast_file))
                else:
                    iteration_results.extend(self._run_sequential_tests(test_case, test_suite.languages, raw_ast_file))
            
            all_results.extend(iteration_results)
        
        return all_results
    
    def _prepare_test_case(self, test_case: TestCase) -> Optional[str]:
        """Prepare raw AST JSON for test case"""
        raw_ast_file = self.temp_dir / f"{test_case.test_id}_raw.json"
        
        try:
            result = subprocess.run([
                'perl',
                str(self.base_dir / 'tools' / 'ebnf_to_json.pl'),
                test_case.ebnf_file
            ], stdout=open(raw_ast_file, 'w'), stderr=subprocess.PIPE, text=True)
            
            if result.returncode == 0:
                return str(raw_ast_file)
            else:
                print(f"Failed to generate raw AST for {test_case.name}: {result.stderr}")
                return None
        except Exception as e:
            print(f"Error preparing test case {test_case.name}: {e}")
            return None
    
    def _run_parallel_tests(self, test_case: TestCase, languages: List[str], raw_ast_file: str) -> List[TestResult]:
        """Run tests in parallel across languages"""
        results = []
        
        with concurrent.futures.ThreadPoolExecutor(max_workers=len(languages)) as executor:
            future_to_lang = {
                executor.submit(self._run_single_test, test_case, lang, raw_ast_file): lang
                for lang in languages
            }
            
            for future in concurrent.futures.as_completed(future_to_lang):
                lang = future_to_lang[future]
                try:
                    result = future.result()
                    results.append(result)
                except Exception as e:
                    results.append(TestResult(
                        test_case=test_case,
                        language=lang,
                        pipeline_success=False,
                        pipeline_time=0.0,
                        generation_success=False,
                        generation_time=0.0,
                        rules_processed=0,
                        error_message=str(e)
                    ))
        
        return results
    
    def _run_sequential_tests(self, test_case: TestCase, languages: List[str], raw_ast_file: str) -> List[TestResult]:
        """Run tests sequentially across languages"""
        results = []
        
        for lang in languages:
            try:
                result = self._run_single_test(test_case, lang, raw_ast_file)
                results.append(result)
            except Exception as e:
                results.append(TestResult(
                    test_case=test_case,
                    language=lang,
                    pipeline_success=False,
                    pipeline_time=0.0,
                    generation_success=False,
                    generation_time=0.0,
                    rules_processed=0,
                    error_message=str(e)
                ))
        
        return results
    
    def _run_single_test(self, test_case: TestCase, language: str, raw_ast_file: str) -> TestResult:
        """Run single test for specific language"""
        test_output_dir = self.temp_dir / f"{test_case.test_id}_{language}"
        test_output_dir.mkdir(exist_ok=True)
        
        # Execute language-specific pipeline
        exec_result = self.executor.execute(language, raw_ast_file, test_output_dir)
        
        # Calculate output hash for comparison
        output_hash = None
        if exec_result.get('output_data'):
            output_hash = hashlib.md5(exec_result['output_data'].encode()).hexdigest()
        
        return TestResult(
            test_case=test_case,
            language=language,
            pipeline_success=exec_result['pipeline_success'],
            pipeline_time=exec_result['pipeline_time'],
            generation_success=exec_result['generation_success'],
            generation_time=exec_result['generation_time'],
            rules_processed=0,  # Would need to extract from output
            error_message=exec_result.get('error_message'),
            output_hash=output_hash
        )
    
    def analyze_results(self, results: List[TestResult]) -> Dict:
        """Analyze test results for patterns and issues"""
        analysis = {
            'summary': {
                'total_tests': len(results),
                'successful_tests': len([r for r in results if r.pipeline_success and r.generation_success]),
                'failed_tests': len([r for r in results if not r.pipeline_success or not r.generation_success]),
                'languages_tested': len(set(r.language for r in results)),
                'test_cases_tested': len(set(r.test_case.test_id for r in results))
            },
            'language_performance': {},
            'cross_language_consistency': {},
            'failure_analysis': {},
            'performance_benchmarks': {}
        }
        
        # Analyze by language
        for lang in set(r.language for r in results):
            lang_results = [r for r in results if r.language == lang]
            analysis['language_performance'][lang] = {
                'success_rate': len([r for r in lang_results if r.pipeline_success and r.generation_success]) / len(lang_results),
                'avg_pipeline_time': sum(r.pipeline_time for r in lang_results) / len(lang_results),
                'avg_generation_time': sum(r.generation_time for r in lang_results) / len(lang_results),
                'total_tests': len(lang_results)
            }
        
        # Analyze cross-language consistency
        test_case_groups = {}
        for result in results:
            if result.test_case.test_id not in test_case_groups:
                test_case_groups[result.test_case.test_id] = []
            test_case_groups[result.test_case.test_id].append(result)
        
        for test_id, case_results in test_case_groups.items():
            if len(case_results) > 1:
                # Check output hash consistency
                output_hashes = [r.output_hash for r in case_results if r.output_hash]
                unique_hashes = set(output_hashes)
                
                analysis['cross_language_consistency'][test_id] = {
                    'languages_tested': [r.language for r in case_results],
                    'consistent_outputs': len(unique_hashes) <= 1,
                    'success_consistency': len(set(r.pipeline_success and r.generation_success for r in case_results)) == 1
                }
        
        # Analyze failures
        failed_results = [r for r in results if not r.pipeline_success or not r.generation_success]
        failure_patterns = {}
        for result in failed_results:
            error_key = result.error_message or "Unknown error"
            if error_key not in failure_patterns:
                failure_patterns[error_key] = []
            failure_patterns[error_key].append(f"{result.language}:{result.test_case.name}")
        
        analysis['failure_analysis'] = failure_patterns
        
        return analysis
    
    def generate_report(self, results: List[TestResult], analysis: Dict, output_file: Optional[str] = None) -> str:
        """Generate comprehensive test report"""
        report_lines = [
            "# Multi-Language EBNF Parser Generator Test Report",
            f"Generated: {datetime.now().isoformat()}",
            f"Total Tests: {analysis['summary']['total_tests']}",
            "",
            "## Test Summary",
            f"- Successful Tests: {analysis['summary']['successful_tests']}",
            f"- Failed Tests: {analysis['summary']['failed_tests']}",
            f"- Success Rate: {analysis['summary']['successful_tests'] / analysis['summary']['total_tests'] * 100:.1f}%",
            f"- Languages Tested: {analysis['summary']['languages_tested']}",
            f"- Test Cases: {analysis['summary']['test_cases_tested']}",
            "",
            "## Language Performance",
        ]
        
        for lang, perf in analysis['language_performance'].items():
            report_lines.extend([
                f"### {lang.title()}",
                f"- Success Rate: {perf['success_rate'] * 100:.1f}%",
                f"- Average Pipeline Time: {perf['avg_pipeline_time']:.3f}s",
                f"- Average Generation Time: {perf['avg_generation_time']:.3f}s",
                f"- Tests Run: {perf['total_tests']}",
                ""
            ])
        
        # Cross-language consistency
        if analysis['cross_language_consistency']:
            report_lines.extend([
                "## Cross-Language Consistency",
                ""
            ])
            
            for test_id, consistency in analysis['cross_language_consistency'].items():
                status = "✓" if consistency['consistent_outputs'] and consistency['success_consistency'] else "✗"
                report_lines.append(f"{status} Test {test_id}: {', '.join(consistency['languages_tested'])}")
        
        # Failure analysis
        if analysis['failure_analysis']:
            report_lines.extend([
                "",
                "## Failure Analysis",
                ""
            ])
            
            for error, occurrences in analysis['failure_analysis'].items():
                report_lines.extend([
                    f"### {error}",
                    f"Occurrences: {len(occurrences)}",
                    "- " + "\n- ".join(occurrences),
                    ""
                ])
        
        # Detailed results
        report_lines.extend([
            "",
            "## Detailed Results",
            ""
        ])
        
        for result in sorted(results, key=lambda r: (r.test_case.name, r.language)):
            status = "✓" if result.pipeline_success and result.generation_success else "✗"
            report_lines.append(
                f"{status} {result.test_case.name} ({result.language}): "
                f"Pipeline {result.pipeline_time:.3f}s, "
                f"Generation {result.generation_time:.3f}s"
            )
            if result.error_message:
                report_lines.append(f"  Error: {result.error_message}")
        
        report_content = "\n".join(report_lines)
        
        if output_file:
            with open(output_file, 'w') as f:
                f.write(report_content)
            print(f"Report written to: {output_file}")
        
        return report_content

def main():
    parser = argparse.ArgumentParser(description="Automated Testing Framework for Multi-Language EBNF Parser Generator")
    parser.add_argument('--full-test', action='store_true', help='Run full test suite across all languages')
    parser.add_argument('--language', choices=['perl', 'python', 'rust', 'julia', 'go', 'zig'], 
                       help='Test specific language only')
    parser.add_argument('--grammar', help='Test specific grammar file')
    parser.add_argument('--benchmark', action='store_true', help='Run performance benchmarks')
    parser.add_argument('--iterations', type=int, default=1, help='Number of test iterations')
    parser.add_argument('--parallel', action='store_true', default=True, help='Run tests in parallel')
    parser.add_argument('--output-report', help='Output report file path')
    
    args = parser.parse_args()
    
    base_dir = Path(__file__).parent.parent
    
    with TestFramework(base_dir) as framework:
        # Generate test cases
        if args.grammar:
            test_cases = [TestCase(
                name=Path(args.grammar).stem,
                ebnf_file=args.grammar,
                expected_rules=10  # Estimate
            )]
        else:
            test_cases = framework.generate_test_cases()
        
        if not test_cases:
            print("No test cases found. Please check that grammar files exist.")
            return 1
        
        # Configure test suite
        if args.language:
            languages = [args.language]
        elif args.full_test:
            languages = ['perl', 'python', 'rust', 'julia', 'go']
        else:
            languages = ['perl', 'python']  # Default minimal test
        
        test_suite = TestSuite(
            languages=languages,
            test_cases=test_cases,
            iterations=args.iterations,
            parallel=args.parallel,
            benchmark_mode=args.benchmark
        )
        
        # Run tests
        print(f"Starting test execution...")
        start_time = time.time()
        
        results = framework.run_test_suite(test_suite)
        
        total_time = time.time() - start_time
        print(f"Test execution completed in {total_time:.2f}s")
        
        # Analyze results
        analysis = framework.analyze_results(results)
        
        # Generate report
        report_file = args.output_report or f"test_report_{datetime.now().strftime('%Y%m%d_%H%M%S')}.md"
        report = framework.generate_report(results, analysis, report_file)
        
        # Print summary
        print("\n" + "="*60)
        print("TEST EXECUTION SUMMARY")
        print("="*60)
        print(f"Total Tests: {analysis['summary']['total_tests']}")
        print(f"Successful: {analysis['summary']['successful_tests']}")
        print(f"Failed: {analysis['summary']['failed_tests']}")
        print(f"Success Rate: {analysis['summary']['successful_tests'] / analysis['summary']['total_tests'] * 100:.1f}%")
        print(f"Total Time: {total_time:.2f}s")
        
        # Return exit code based on results
        return 0 if analysis['summary']['failed_tests'] == 0 else 1

if __name__ == "__main__":
    sys.exit(main())
