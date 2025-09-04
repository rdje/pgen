//! Annotation Parser Test Suite
//! 
//! Utilities and common functionality for testing annotation parsers

pub mod return_annotation_tests;
pub mod round_trip_tests;
pub mod semantic_annotation_tests;
pub mod regex_parser_tests;

/// Common test utilities
pub mod test_utils {
    use std::fs;
    use std::path::Path;

    /// Load test data from a file
    pub fn load_test_data(relative_path: &str) -> String {
        let test_data_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/annotation_parsers/test_data");
        let file_path = test_data_dir.join(relative_path);
        
        fs::read_to_string(&file_path)
            .unwrap_or_else(|e| panic!("Failed to load test data from {:?}: {}", file_path, e))
    }

    /// Load multiple test cases from a directory
    pub fn load_test_cases(dir_name: &str) -> Vec<(String, String)> {
        let test_data_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/annotation_parsers/test_data")
            .join(dir_name);

        let mut cases = Vec::new();
        
        if let Ok(entries) = fs::read_dir(&test_data_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        cases.push((name.to_string(), content));
                    }
                }
            }
        }
        
        cases.sort_by(|a, b| a.0.cmp(&b.0)); // Sort by filename
        cases
    }
}

/// Test result aggregation
pub struct TestResults {
    pub passed: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

impl TestResults {
    pub fn new() -> Self {
        Self {
            passed: 0,
            failed: 0,
            errors: Vec::new(),
        }
    }

    pub fn add_pass(&mut self) {
        self.passed += 1;
    }

    pub fn add_fail(&mut self, error: String) {
        self.failed += 1;
        self.errors.push(error);
    }

    pub fn total(&self) -> usize {
        self.passed + self.failed
    }

    pub fn success_rate(&self) -> f64 {
        if self.total() == 0 {
            0.0
        } else {
            self.passed as f64 / self.total() as f64
        }
    }
}
