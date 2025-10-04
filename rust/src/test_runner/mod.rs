// Test runner module for pgen
// Provides infrastructure for running JSON-based tests

pub mod round_trip_tests;
pub mod normalization;

pub use round_trip_tests::RoundTripTestRunner;