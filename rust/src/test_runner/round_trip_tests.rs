use crate::test_runner::Parser;
use anyhow::Result;
/// Round-trip testing framework for mathematical parser validation
/// Provides complete input → parse → AST → unparse → output validation
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, Deserialize, Serialize)]
pub struct TestExpectations {
    #[serde(default)]
    pub bootstrap_parser: String, // "pass", "fail", "expected_fail", "skip"
    #[serde(default)]
    pub generated_parser: String, // "pass", "fail", "expected_fail", "skip"
}

impl Default for TestExpectations {
    fn default() -> Self {
        Self {
            bootstrap_parser: "pass".to_string(),
            generated_parser: "pass".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RoundTripTest {
    pub name: String,
    pub description: String,
    pub input: String,
    pub expected_round_trip: String,
    #[serde(default)]
    pub parser_type: String,
    #[serde(default)]
    pub normalizer: String,
    #[serde(default)]
    pub float_precision: Option<usize>,
    #[serde(default)]
    pub skip: bool,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub tier: Option<String>, // "A", "B", "C"
    #[serde(default)]
    pub expectations: TestExpectations,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SuiteFile {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub parser_type: String,
    pub tests: Vec<RoundTripTest>,
}

#[derive(Debug)]
pub struct TestSuite {
    pub name: String,
    pub suite_name: String,
    pub parser_type: String,
    pub description: String,
    pub tests: Vec<RoundTripTest>,
}

#[derive(Debug, Default)]
pub struct Report {
    pub passed: usize,
    pub failed: usize,
    pub total: usize,
    pub results: Vec<TestResult>,
}

impl Report {
    pub fn add_result(&mut self, result: TestResult) {
        self.results.push(result.clone());
        self.total += 1;
        if result.passed {
            self.passed += 1;
        } else {
            self.failed += 1;
        }
    }

    pub fn print_summary(&self) {
        println!("Summary: {}/{} passed", self.passed, self.total);
        if self.total == 0 {
            println!("No tests were executed (check your filters).");
            return;
        }
        if self.failed > 0 {
            println!("Failures:");
            for r in &self.results {
                if !r.passed {
                    println!("  - {}: {}", r.suite, r.test);
                    println!("    ↳ {}", r.message);
                }
            }
        }
    }

    pub fn print_dashboard(&self, parser_name: &str) {
        println!("Dashboard for {}:", parser_name);
        let rate = if self.total == 0 {
            0.0
        } else {
            (self.passed as f64 / self.total as f64) * 100.0
        };
        println!(
            "Passed: {}, Failed: {}, Rate: {:.1}%",
            self.passed, self.failed, rate
        );
        println!(
            "| {:<20} | {:<15} | {:<6} | {}",
            "Suite", "Test", "Passed", "Message"
        );
        println!("{}", "-".repeat(70));
        for r in &self.results {
            println!(
                "| {:<20} | {:<15} | {:<6} | {}",
                r.suite, r.test, r.passed, r.message
            );
        }
    }
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub suite: String,
    pub test: String,
    pub passed: bool,
    pub message: String,
}

pub struct RoundTripTestRunner {
    test_data_dir: PathBuf,
    pub results: Vec<TestResult>,
    parser: Option<Box<dyn Parser>>,
    verbose: bool,
    parser_filter: Option<String>,
    suite_filter: Option<String>,
    tag_filter: Vec<String>,
    fail_fast: bool,
}

impl RoundTripTestRunner {
    pub fn new() -> Self {
        let test_data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data");
        Self {
            test_data_dir,
            results: Vec::new(),
            parser: None,
            verbose: false,
            parser_filter: None,
            suite_filter: None,
            tag_filter: Vec::new(),
            fail_fast: false,
        }
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn with_parser(mut self, parser: Box<dyn Parser>) -> Self {
        self.parser = Some(parser);
        self
    }

    pub fn with_parser_filter(mut self, filter: String) -> Self {
        self.parser_filter = Some(filter);
        self
    }

    pub fn with_suite_filter(mut self, filter: String) -> Self {
        self.suite_filter = Some(filter);
        self
    }

    pub fn with_tag_filter(mut self, tags: Vec<String>) -> Self {
        self.tag_filter = tags;
        self
    }

    pub fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }

    pub fn discover_test_suites(&self) -> Result<Vec<TestSuite>> {
        let mut suites = Vec::new();
        if !self.test_data_dir.exists() {
            fs::create_dir_all(&self.test_data_dir)?;
        }

        // Find all .json files in subdirectories
        for entry in fs::read_dir(&self.test_data_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let subdir_name = path.file_name().unwrap().to_string_lossy().into_owned();
                let inferred_parser_type = Self::infer_parser_type_from_dir(&subdir_name);

                // Look for all .json files in this subdirectory
                for json_entry in fs::read_dir(&path)? {
                    let json_entry = json_entry?;
                    let json_path = json_entry.path();

                    if json_path.is_file()
                        && json_path.extension() == Some(std::ffi::OsStr::new("json"))
                    {
                        let json_name = json_path.file_stem().unwrap().to_string_lossy();
                        let suite_name = format!("{}_{}", subdir_name, json_name);

                        match fs::read_to_string(&json_path) {
                            Ok(json_str) => {
                                if let Ok(mut tests) =
                                    serde_json::from_str::<Vec<RoundTripTest>>(&json_str)
                                {
                                    Self::normalize_tests_parser_types(
                                        &mut tests,
                                        &inferred_parser_type,
                                    );
                                    let parser_type = tests
                                        .first()
                                        .map(|t| t.parser_type.clone())
                                        .unwrap_or_else(|| inferred_parser_type.clone());

                                    suites.push(TestSuite {
                                        name: suite_name.clone(),
                                        suite_name,
                                        parser_type,
                                        description: format!(
                                            "Tests from {}/{}",
                                            subdir_name,
                                            json_path.file_name().unwrap().to_string_lossy()
                                        ),
                                        tests,
                                    });
                                    continue;
                                }

                                if let Ok(mut suite_file) =
                                    serde_json::from_str::<SuiteFile>(&json_str)
                                {
                                    let suite_parser_type =
                                        if suite_file.parser_type.trim().is_empty() {
                                            inferred_parser_type.clone()
                                        } else {
                                            Self::canonical_parser_type(&suite_file.parser_type)
                                        };
                                    Self::normalize_tests_parser_types(
                                        &mut suite_file.tests,
                                        &suite_parser_type,
                                    );

                                    let parser_type = suite_file
                                        .tests
                                        .first()
                                        .map(|t| t.parser_type.clone())
                                        .unwrap_or_else(|| suite_parser_type.clone());
                                    let description = if suite_file.description.trim().is_empty() {
                                        format!(
                                            "Tests from {}/{}",
                                            subdir_name,
                                            json_path.file_name().unwrap().to_string_lossy()
                                        )
                                    } else {
                                        suite_file.description.clone()
                                    };
                                    let name = if suite_file.name.trim().is_empty() {
                                        suite_name.clone()
                                    } else {
                                        suite_file.name.clone()
                                    };

                                    suites.push(TestSuite {
                                        name,
                                        suite_name,
                                        parser_type,
                                        description,
                                        tests: suite_file.tests,
                                    });
                                }
                            }
                            Err(_) => continue,
                        }
                    }
                }
            }
        }
        Ok(suites)
    }

    pub fn run_all_tests(&mut self) -> Result<Report> {
        self.run_all_tests_with_filters(
            self.parser_filter.clone(),
            self.suite_filter.clone(),
            self.tag_filter.clone(),
            self.fail_fast,
        )
    }

    pub fn run_all_tests_with_filters(
        &mut self,
        parser_filter: Option<String>,
        suite_filter: Option<String>,
        tag_filter: Vec<String>,
        fail_fast: bool,
    ) -> Result<Report> {
        let suites = self.discover_test_suites()?;
        let mut report = Report::default();
        let normalized_parser_filter = parser_filter
            .as_ref()
            .map(|parser| Self::canonical_parser_type(parser));
        let normalized_suite_filter = suite_filter.as_ref().map(|s| s.trim().to_lowercase());

        for suite in suites {
            if let Some(ref suite_filter) = normalized_suite_filter {
                if !Self::suite_matches_filter(&suite, suite_filter) {
                    continue;
                }
            }
            for test in &suite.tests {
                if test.skip {
                    continue;
                }

                // Enforce per-target expectations.
                // Only explicit "skip" is skipped; expected failures are executed and validated.
                if Self::should_skip_for_active_target(test) {
                    if self.verbose {
                        println!(
                            "⏭️  Skipping '{}' for target '{}' (expectation: '{}')",
                            test.name,
                            Self::active_parser_target_name(),
                            Self::active_expectation(test)
                        );
                    }
                    continue;
                }

                // Filter by parser
                if let Some(ref parser) = normalized_parser_filter {
                    if Self::canonical_parser_type(&test.parser_type) != *parser {
                        continue;
                    }
                }

                // Filter by tags
                if !tag_filter.is_empty() {
                    let has_matching_tag = test.tags.iter().any(|tag| tag_filter.contains(tag));
                    if !has_matching_tag {
                        continue;
                    }
                }

                // Run test with timeout
                let start_time = Instant::now();
                let test_result = self.run_single_test_with_timeout(&suite, &test);
                let duration = start_time.elapsed();

                // Check for timeout
                let timeout_ms = test.timeout_ms.unwrap_or(5000);
                if duration > Duration::from_millis(timeout_ms) {
                    report.add_result(TestResult {
                        suite: suite.name.clone(),
                        test: test.name.clone(),
                        passed: false,
                        message: format!("TIMEOUT: {}ms > {}ms", duration.as_millis(), timeout_ms),
                    });
                    if fail_fast {
                        self.results = report.results.clone();
                        return Ok(report);
                    }
                    continue;
                }

                let failed_now = !test_result.passed;
                report.add_result(test_result);
                if fail_fast && failed_now {
                    self.results = report.results.clone();
                    return Ok(report);
                }
            }
        }

        self.results = report.results.clone();
        Ok(report)
    }

    fn run_single_test_with_timeout(&self, suite: &TestSuite, test: &RoundTripTest) -> TestResult {
        let expectation = Self::active_expectation(test);
        let expect_failure = Self::expect_failure_for_active_target(test);

        // Log the start of this test case
        println!("\n🧪 Testing: {}", test.input);
        println!(
            "🎯 Expectation ({}): {}",
            Self::active_parser_target_name(),
            if expectation.is_empty() {
                "pass".to_string()
            } else {
                expectation.clone()
            }
        );
        println!("{}", "─".repeat(60));

        // Use real parser if available, otherwise fall back to mock
        let parse_result = if let Some(ref parser) = self.parser {
            parser.round_trip(&test.input)
        } else {
            // Mock round-trip for testing the framework
            let parsed = self.mock_parse(&test.input);
            Ok(self.mock_unparse(&parsed))
        };

        // Outcome validation against expectation:
        // - Expect pass: parse must succeed + round-trip must match expected.
        // - Expect fail/expected_fail: parse must fail.
        match parse_result {
            Err(e) => {
                if expect_failure {
                    let msg = format!("✅ PASSED - Expected parser failure observed: {}", e);
                    println!("Result: {}", msg);
                    println!("");
                    TestResult {
                        suite: suite.name.clone(),
                        test: test.name.clone(),
                        passed: true,
                        message: msg,
                    }
                } else {
                    println!("❌ Parser error: {}", e);
                    println!("");
                    TestResult {
                        suite: suite.name.clone(),
                        test: test.name.clone(),
                        passed: false,
                        message: format!("PARSER ERROR: {}", e),
                    }
                }
            }
            Ok(unparsed) => {
                if expect_failure {
                    let msg =
                        "❌ FAILED - Expected parser failure, but parsing succeeded".to_string();
                    println!("Result: {}", msg);
                    println!("");
                    return TestResult {
                        suite: suite.name.clone(),
                        test: test.name.clone(),
                        passed: false,
                        message: msg,
                    };
                }

                // Expected pass path: validate normalized round-trip output
                let mut normalizer =
                    crate::test_runner::normalization::Normalizer::from_str(&test.normalizer);
                if matches!(normalizer, crate::test_runner::normalization::Normalizer::Text)
                    && Self::canonical_parser_type(&test.parser_type) == "return"
                {
                    normalizer = crate::test_runner::normalization::Normalizer::ReturnAst;
                }
                let actual = crate::test_runner::normalization::apply_normalizer(
                    normalizer,
                    &unparsed,
                    test.float_precision,
                );
                let expected = crate::test_runner::normalization::apply_normalizer(
                    normalizer,
                    &test.expected_round_trip,
                    test.float_precision,
                );

                let passed = actual == expected;
                let msg = if passed {
                    "✅ PASSED".to_string()
                } else {
                    format!("❌ FAILED - Expected: '{}', Got: '{}'", expected, actual)
                };

                println!("Result: {}", msg);
                println!("");

                TestResult {
                    suite: suite.name.clone(),
                    test: test.name.clone(),
                    passed,
                    message: msg,
                }
            }
        }
    }

    // Mock implementations for testing the framework
    fn mock_parse(&self, input: &str) -> String {
        // Identity parse for infrastructure validation
        input.to_string()
    }

    fn mock_unparse(&self, ast: &str) -> String {
        // For now, just return the AST as a string
        ast.to_string()
    }

    fn canonical_parser_type(parser_type: &str) -> String {
        match parser_type.trim().to_lowercase().as_str() {
            "return" | "return_annotation" | "return_annotations" => "return".to_string(),
            "semantic" | "semantic_annotation" | "semantic_annotations" => "semantic".to_string(),
            "regex" => "regex".to_string(),
            "mock" => "mock".to_string(),
            "all" => "all".to_string(),
            _ => parser_type.trim().to_string(),
        }
    }

    fn infer_parser_type_from_dir(dir_name: &str) -> String {
        let normalized = dir_name.trim().to_lowercase();
        if normalized.contains("return") {
            "return".to_string()
        } else if normalized.contains("semantic") {
            "semantic".to_string()
        } else if normalized.contains("regex") {
            "regex".to_string()
        } else {
            "unknown".to_string()
        }
    }

    fn normalize_tests_parser_types(tests: &mut [RoundTripTest], default_parser_type: &str) {
        let fallback = Self::canonical_parser_type(default_parser_type);
        for test in tests.iter_mut() {
            if test.parser_type.trim().is_empty() {
                test.parser_type = fallback.clone();
            } else {
                test.parser_type = Self::canonical_parser_type(&test.parser_type);
            }
        }
    }

    fn suite_matches_filter(suite: &TestSuite, suite_filter: &str) -> bool {
        let name = suite.name.to_lowercase();
        let suite_name = suite.suite_name.to_lowercase();
        let description = suite.description.to_lowercase();

        name == suite_filter
            || suite_name == suite_filter
            || name.contains(suite_filter)
            || suite_name.contains(suite_filter)
            || description.contains(suite_filter)
    }

    fn active_parser_target_name() -> &'static str {
        #[cfg(feature = "generated_parsers")]
        {
            "generated_parser"
        }
        #[cfg(not(feature = "generated_parsers"))]
        {
            "bootstrap_parser"
        }
    }

    fn active_expectation(test: &RoundTripTest) -> String {
        #[cfg(feature = "generated_parsers")]
        {
            test.expectations.generated_parser.trim().to_lowercase()
        }
        #[cfg(not(feature = "generated_parsers"))]
        {
            test.expectations.bootstrap_parser.trim().to_lowercase()
        }
    }

    fn should_skip_for_active_target(test: &RoundTripTest) -> bool {
        let expectation = Self::active_expectation(test);
        expectation == "skip"
    }

    fn expect_failure_for_active_target(test: &RoundTripTest) -> bool {
        let expectation = Self::active_expectation(test);
        expectation == "fail" || expectation == "expected_fail"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip_runner() {
        let mut runner = RoundTripTestRunner::new();
        let report = runner.run_all_tests().unwrap();
        assert!(report.total > 0);
        assert_eq!(report.total, report.passed + report.failed);
    }
}
