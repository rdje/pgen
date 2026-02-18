use anyhow::{Context, Result, anyhow, bail};
use chrono::Utc;
use clap::{Parser, ValueEnum};
use pgen::NoOpLogger;
use pgen::ast_pipeline::{UnifiedReturnAST, UnifiedSemanticAST};
use pgen::generated_parsers::return_annotation::Return_annotationParser;
use pgen::generated_parsers::semantic_annotation::Semantic_annotationParser;
use pgen::test_runner::{TestSuite, UniversalTestRunner};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

const DEFAULT_THRESHOLDS_PATH: &str = "perf/thresholds.json";

#[derive(Debug, Clone, ValueEnum)]
enum ParserFilter {
    All,
    Return,
    Semantic,
}

#[derive(Debug, Clone, Copy)]
enum ParserKind {
    Return,
    Semantic,
}

impl ParserKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Return => "return",
            Self::Semantic => "semantic",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum BackendKind {
    Bootstrap,
    Generated,
}

impl BackendKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Bootstrap => "bootstrap",
            Self::Generated => "generated",
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "perf_bench")]
#[command(
    about = "Benchmark bootstrap vs generated parser throughput/latency and enforce thresholds"
)]
#[command(version = "1.0.0")]
struct Args {
    /// Parser family to benchmark
    #[arg(long, value_enum, default_value = "all")]
    parser: ParserFilter,

    /// Number of measured benchmark loops over the selected corpus
    #[arg(long, default_value_t = 80)]
    iterations: usize,

    /// Number of warmup loops over the selected corpus
    #[arg(long, default_value_t = 10)]
    warmup_iterations: usize,

    /// Optional suite name/substring filter
    #[arg(long)]
    suite: Option<String>,

    /// Maximum number of corpus cases per parser family
    #[arg(long, default_value_t = 64)]
    max_cases: usize,

    /// Threshold policy JSON path
    #[arg(long, default_value = DEFAULT_THRESHOLDS_PATH)]
    thresholds_json: PathBuf,

    /// Optional JSON report output path
    #[arg(long)]
    output_json: Option<PathBuf>,

    /// Enforce thresholds with non-zero exit on violation
    #[arg(long)]
    enforce_thresholds: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct ThresholdConfig {
    version: u32,
    min_cases_per_parser: usize,
    parsers: HashMap<String, ParserThresholds>,
}

#[derive(Debug, Clone, Deserialize)]
struct ParserThresholds {
    bootstrap: BackendThresholds,
    generated: BackendThresholds,
    generated_vs_bootstrap_min_throughput_ratio: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct BackendThresholds {
    min_throughput_ops_per_sec: f64,
    max_avg_latency_us: f64,
}

#[derive(Debug, Clone, Serialize)]
struct BenchmarkCase {
    suite: String,
    test: String,
    input: String,
}

#[derive(Debug, Clone, Serialize)]
struct BackendMetrics {
    backend: String,
    attempts: usize,
    successes: usize,
    parse_failures: usize,
    elapsed_ms: f64,
    throughput_ops_per_sec: f64,
    avg_latency_us: f64,
    sample_failures: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ParserBenchmarkResult {
    parser: String,
    cases: usize,
    iterations: usize,
    warmup_iterations: usize,
    bootstrap: BackendMetrics,
    generated: BackendMetrics,
    generated_vs_bootstrap_throughput_ratio: f64,
}

#[derive(Debug, Clone, Serialize)]
struct PerfReportArgs {
    parser: String,
    iterations: usize,
    warmup_iterations: usize,
    suite: Option<String>,
    max_cases: usize,
    thresholds_json: String,
    enforce_thresholds: bool,
}

#[derive(Debug, Clone, Serialize)]
struct PerfReport {
    generated_at: String,
    thresholds_version: u32,
    args: PerfReportArgs,
    results: Vec<ParserBenchmarkResult>,
    violations: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.iterations == 0 {
        bail!("--iterations must be >= 1");
    }
    if args.max_cases == 0 {
        bail!("--max-cases must be >= 1");
    }

    let thresholds_content = std::fs::read_to_string(&args.thresholds_json).with_context(|| {
        format!(
            "Failed to read thresholds file '{}'",
            args.thresholds_json.display()
        )
    })?;
    let thresholds: ThresholdConfig =
        serde_json::from_str(&thresholds_content).context("Invalid thresholds JSON format")?;

    let parser_kinds: Vec<ParserKind> = match args.parser {
        ParserFilter::All => vec![ParserKind::Return, ParserKind::Semantic],
        ParserFilter::Return => vec![ParserKind::Return],
        ParserFilter::Semantic => vec![ParserKind::Semantic],
    };

    let mut results = Vec::new();
    let mut violations = Vec::new();
    for parser_kind in parser_kinds {
        let parser_key = parser_kind.as_str().to_string();
        let parser_thresholds = thresholds
            .parsers
            .get(&parser_key)
            .with_context(|| format!("Missing thresholds for parser '{}'", parser_key))?;

        let cases = collect_benchmark_cases(parser_kind, args.suite.as_deref(), args.max_cases)?;
        if cases.len() < thresholds.min_cases_per_parser {
            violations.push(format!(
                "[{}] benchmark corpus too small: {} case(s) < min_cases_per_parser {}",
                parser_key,
                cases.len(),
                thresholds.min_cases_per_parser
            ));
        }

        let bootstrap = benchmark_backend(
            &cases,
            parser_kind,
            BackendKind::Bootstrap,
            args.warmup_iterations,
            args.iterations,
        );
        let generated = benchmark_backend(
            &cases,
            parser_kind,
            BackendKind::Generated,
            args.warmup_iterations,
            args.iterations,
        );

        let throughput_ratio = if bootstrap.throughput_ops_per_sec > 0.0 {
            generated.throughput_ops_per_sec / bootstrap.throughput_ops_per_sec
        } else {
            0.0
        };

        let result = ParserBenchmarkResult {
            parser: parser_key.clone(),
            cases: cases.len(),
            iterations: args.iterations,
            warmup_iterations: args.warmup_iterations,
            bootstrap,
            generated,
            generated_vs_bootstrap_throughput_ratio: throughput_ratio,
        };
        validate_backend_thresholds(
            &result.parser,
            &result.bootstrap,
            &parser_thresholds.bootstrap,
            &mut violations,
        );
        validate_backend_thresholds(
            &result.parser,
            &result.generated,
            &parser_thresholds.generated,
            &mut violations,
        );
        if throughput_ratio < parser_thresholds.generated_vs_bootstrap_min_throughput_ratio {
            violations.push(format!(
                "[{}] generated/bootstrap throughput ratio {:.4} < minimum {:.4}",
                result.parser,
                throughput_ratio,
                parser_thresholds.generated_vs_bootstrap_min_throughput_ratio
            ));
        }

        print_result_summary(&result);
        results.push(result);
    }

    let report = PerfReport {
        generated_at: Utc::now().to_rfc3339(),
        thresholds_version: thresholds.version,
        args: PerfReportArgs {
            parser: match args.parser {
                ParserFilter::All => "all".to_string(),
                ParserFilter::Return => "return".to_string(),
                ParserFilter::Semantic => "semantic".to_string(),
            },
            iterations: args.iterations,
            warmup_iterations: args.warmup_iterations,
            suite: args.suite.clone(),
            max_cases: args.max_cases,
            thresholds_json: args.thresholds_json.display().to_string(),
            enforce_thresholds: args.enforce_thresholds,
        },
        results,
        violations: violations.clone(),
    };

    if let Some(output_json_path) = args.output_json.as_ref() {
        if let Some(parent) = output_json_path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create output directory '{}'", parent.display())
            })?;
        }
        let report_json = serde_json::to_string_pretty(&report)?;
        std::fs::write(output_json_path, report_json).with_context(|| {
            format!(
                "Failed to write benchmark report '{}'",
                output_json_path.display()
            )
        })?;
        println!(
            "Wrote benchmark report JSON: {}",
            output_json_path.display()
        );
    }

    if violations.is_empty() {
        println!("✅ Benchmark thresholds satisfied.");
        return Ok(());
    }

    println!(
        "⚠️ Benchmark threshold violations detected ({}):",
        violations.len()
    );
    for violation in &violations {
        println!("  - {}", violation);
    }

    if args.enforce_thresholds {
        bail!("Performance threshold gate failed");
    }

    Ok(())
}

fn collect_benchmark_cases(
    parser_kind: ParserKind,
    suite_filter: Option<&str>,
    max_cases: usize,
) -> Result<Vec<BenchmarkCase>> {
    let runner = UniversalTestRunner::new();
    let suites = runner.discover_test_suites()?;
    let normalized_suite_filter = suite_filter.map(|s| s.trim().to_lowercase());
    let mut cases = Vec::new();

    for suite in suites {
        if let Some(ref filter) = normalized_suite_filter {
            if !suite_matches_filter(&suite, filter) {
                continue;
            }
        }

        for test in suite.tests {
            if test.skip {
                continue;
            }
            if canonical_parser_type(&test.parser_type) != parser_kind.as_str() {
                continue;
            }
            if !expectation_is_pass(&test.expectations.bootstrap_parser)
                || !expectation_is_pass(&test.expectations.generated_parser)
            {
                continue;
            }
            cases.push(BenchmarkCase {
                suite: suite.name.clone(),
                test: test.name,
                input: test.input,
            });
        }
    }

    cases.sort_by(|a, b| (&a.suite, &a.test).cmp(&(&b.suite, &b.test)));
    if cases.len() > max_cases {
        cases.truncate(max_cases);
    }
    if cases.is_empty() {
        bail!(
            "No benchmark cases selected for parser '{}' (suite filter: {:?})",
            parser_kind.as_str(),
            suite_filter
        );
    }
    Ok(cases)
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

fn canonical_parser_type(parser_type: &str) -> String {
    match parser_type.trim().to_lowercase().as_str() {
        "return" | "return_annotation" | "return_annotations" => "return".to_string(),
        "semantic" | "semantic_annotation" | "semantic_annotations" => "semantic".to_string(),
        "regex" => "regex".to_string(),
        other => other.to_string(),
    }
}

fn expectation_is_pass(expectation: &str) -> bool {
    let normalized = expectation.trim().to_lowercase();
    normalized.is_empty() || normalized == "pass"
}

fn benchmark_backend(
    cases: &[BenchmarkCase],
    parser_kind: ParserKind,
    backend_kind: BackendKind,
    warmup_iterations: usize,
    iterations: usize,
) -> BackendMetrics {
    for _ in 0..warmup_iterations {
        for case in cases {
            let _ = parse_once(parser_kind, backend_kind, &case.input);
        }
    }

    let mut parse_failures = 0usize;
    let mut successes = 0usize;
    let mut sample_failures = Vec::new();

    let start = Instant::now();
    for _ in 0..iterations {
        for case in cases {
            match parse_once(parser_kind, backend_kind, &case.input) {
                Ok(()) => successes = successes.saturating_add(1),
                Err(err) => {
                    parse_failures = parse_failures.saturating_add(1);
                    if sample_failures.len() < 5 {
                        sample_failures.push(format!("{} / {}: {}", case.suite, case.test, err));
                    }
                }
            }
        }
    }
    let elapsed = start.elapsed();
    let attempts = cases.len().saturating_mul(iterations);
    let elapsed_secs = elapsed.as_secs_f64();
    let throughput_ops_per_sec = if elapsed_secs > 0.0 {
        (attempts as f64) / elapsed_secs
    } else {
        0.0
    };
    let avg_latency_us = if attempts > 0 {
        (elapsed_secs * 1_000_000.0) / (attempts as f64)
    } else {
        0.0
    };

    BackendMetrics {
        backend: backend_kind.as_str().to_string(),
        attempts,
        successes,
        parse_failures,
        elapsed_ms: elapsed_secs * 1000.0,
        throughput_ops_per_sec,
        avg_latency_us,
        sample_failures,
    }
}

fn parse_once(parser_kind: ParserKind, backend_kind: BackendKind, input: &str) -> Result<()> {
    match (parser_kind, backend_kind) {
        (ParserKind::Return, BackendKind::Bootstrap) => {
            let logger = NoOpLogger;
            UnifiedReturnAST::parse_bootstrap(input, &logger)
                .map(|_| ())
                .map_err(|e| anyhow!("bootstrap return parse error: {}", e))
        }
        (ParserKind::Return, BackendKind::Generated) => {
            let mut parser = Return_annotationParser::new(input, Box::new(NoOpLogger));
            parser
                .parse_full_return_annotation()
                .map(|_| ())
                .map_err(|e| anyhow!("generated return parse error: {}", e))
        }
        (ParserKind::Semantic, BackendKind::Bootstrap) => {
            let logger = NoOpLogger;
            UnifiedSemanticAST::parse_bootstrap(input, &logger)
                .map(|_| ())
                .map_err(|e| anyhow!("bootstrap semantic parse error: {}", e))
        }
        (ParserKind::Semantic, BackendKind::Generated) => {
            let mut parser = Semantic_annotationParser::new(input, Box::new(NoOpLogger));
            parser
                .parse_full_semantic_annotation()
                .map(|_| ())
                .map_err(|e| anyhow!("generated semantic parse error: {}", e))
        }
    }
}

fn validate_backend_thresholds(
    parser: &str,
    metrics: &BackendMetrics,
    thresholds: &BackendThresholds,
    violations: &mut Vec<String>,
) {
    if metrics.parse_failures > 0 {
        violations.push(format!(
            "[{}:{}] parse failures: {} (attempts {})",
            parser, metrics.backend, metrics.parse_failures, metrics.attempts
        ));
        for sample in &metrics.sample_failures {
            violations.push(format!(
                "[{}:{}] sample failure: {}",
                parser, metrics.backend, sample
            ));
        }
    }
    if metrics.throughput_ops_per_sec < thresholds.min_throughput_ops_per_sec {
        violations.push(format!(
            "[{}:{}] throughput {:.2} ops/s < min {:.2}",
            parser,
            metrics.backend,
            metrics.throughput_ops_per_sec,
            thresholds.min_throughput_ops_per_sec
        ));
    }
    if metrics.avg_latency_us > thresholds.max_avg_latency_us {
        violations.push(format!(
            "[{}:{}] avg latency {:.2} us > max {:.2}",
            parser, metrics.backend, metrics.avg_latency_us, thresholds.max_avg_latency_us
        ));
    }
}

fn print_result_summary(result: &ParserBenchmarkResult) {
    println!("⚡ Parser benchmark: {}", result.parser);
    println!(
        "  Cases={} warmup={} iterations={}",
        result.cases, result.warmup_iterations, result.iterations
    );
    println!(
        "  bootstrap: throughput={:.2} ops/s avg_latency={:.2} us failures={}",
        result.bootstrap.throughput_ops_per_sec,
        result.bootstrap.avg_latency_us,
        result.bootstrap.parse_failures
    );
    println!(
        "  generated: throughput={:.2} ops/s avg_latency={:.2} us failures={}",
        result.generated.throughput_ops_per_sec,
        result.generated.avg_latency_us,
        result.generated.parse_failures
    );
    println!(
        "  generated/bootstrap throughput ratio={:.4}",
        result.generated_vs_bootstrap_throughput_ratio
    );
}
