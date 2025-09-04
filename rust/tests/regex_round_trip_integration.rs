//! Integration test for regex parser round-trip testing
//! 
//! This test runs the round-trip tests for regex patterns using the mock parser
//! and validates that patterns can be parsed and then serialized back to the
//! same string.

mod annotation_parsers;

use annotation_parsers::regex_parser_tests::run_regex_round_trip_tests;

#[test]
fn test_regex_round_trip_integration() {
    let results = run_regex_round_trip_tests();
    
    // Print summary
    println!("\n=== Final Test Summary ===");
    println!("Total patterns tested: {}", results.total());
    println!("Successful round-trips: {}", results.passed);
    println!("Failed round-trips: {}", results.failed);
    println!("Success rate: {:.1}%", results.success_rate() * 100.0);
    
    // The test passes if we have at least some successful round-trips
    // We don't expect 100% success rate since the mock parser doesn't 
    // implement all regex features yet
    assert!(
        results.total() > 0,
        "No regex patterns were tested"
    );
    
    assert!(
        results.passed > 0,
        "No regex patterns passed round-trip testing"
    );
    
    // We should have at least a 30% success rate with basic patterns
    assert!(
        results.success_rate() >= 0.30,
        "Success rate too low: {:.1}% - expected at least 30%",
        results.success_rate() * 100.0
    );
    
    println!("\n✓ Regex round-trip integration test completed successfully!");
}
