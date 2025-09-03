//! Zig AST Pipeline CLI - Main Entry Point
//!
//! Command-line interface for the Zig backend of PGEN AST transformation pipeline.
//! Transforms raw EBNF AST JSON into structured AST JSON.

const std = @import("std");
const ast_pipeline = @import("ast_pipeline.zig");
const print = std.debug.print;

const ZigASTPipeline = ast_pipeline.ZigASTPipeline;
const PipelineConfig = ast_pipeline.PipelineConfig;
const TransformedAST = ast_pipeline.TransformedAST;

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const args = try std.process.argsAlloc(allocator);
    defer std.process.argsFree(allocator, args);

    if (args.len < 2) {
        print("Zig AST Pipeline - PGEN Backend Language Implementation\n", .{});
        print("\n", .{});
        print("Usage: {s} input_raw.json [output_transformed.json] [options]\n", .{args[0]});
        print("\n", .{});
        print("Arguments:\n", .{});
        print("  input_raw.json              Raw AST JSON file from ebnf_to_json.pl\n", .{});
        print("  output_transformed.json     Output file for transformed AST (optional)\n", .{});
        print("\n", .{});
        print("Options:\n", .{});
        print("  --debug, -d                 Enable debug output\n", .{});
        print("  --stats, -s                 Show transformation statistics\n", .{});
        print("  --quiet, -q                 Suppress debug output (opposite of --debug)\n", .{});
        print("  --no-annotations            Don't preserve annotations\n", .{});
        print("  --no-validation             Skip input/output validation\n", .{});
        print("  --help, -h                  Show this help message\n", .{});
        print("\n", .{});
        print("Examples:\n", .{});
        print("  # Cross-language mode: JSON -> JSON\n", .{});
        print("  {s} grammar.json transformed.json --debug\n", .{args[0]});
        print("\n", .{});
        print("  # Same-language mode: JSON -> In-memory (no output file)\n", .{});
        print("  {s} grammar.json --stats\n", .{args[0]});
        print("\n", .{});
        print("This is the Zig backend language implementation for PGEN.\n", .{});
        print("Part of the 5-stage AST transformation pipeline.\n", .{});
        return;
    }

    const input_file = args[1];
    var output_file: ?[]const u8 = null;
    var debug = false;
    var quiet = false;
    var stats = false;
    var preserve_annotations = true;
    var validate = true;

    // Parse command line arguments
    var i: usize = 2;
    while (i < args.len) {
        const arg = args[i];
        if (std.mem.eql(u8, arg, "--debug") or std.mem.eql(u8, arg, "-d")) {
            debug = true;
        } else if (std.mem.eql(u8, arg, "--quiet") or std.mem.eql(u8, arg, "-q")) {
            quiet = true;
        } else if (std.mem.eql(u8, arg, "--stats") or std.mem.eql(u8, arg, "-s")) {
            stats = true;
        } else if (std.mem.eql(u8, arg, "--no-annotations")) {
            preserve_annotations = false;
        } else if (std.mem.eql(u8, arg, "--no-validation")) {
            validate = false;
        } else if (std.mem.eql(u8, arg, "--help") or std.mem.eql(u8, arg, "-h")) {
            // Print help and exit (already printed above)
            return;
        } else if (!std.mem.startsWith(u8, arg, "--") and output_file == null) {
            output_file = arg;
        } else {
            print("Unknown option: {s}\n", .{arg});
            return;
        }
        i += 1;
    }

    // Quiet overrides debug
    if (quiet) debug = false;

    const config = PipelineConfig{
        .debug = debug,
        .preserve_annotations = preserve_annotations,
        .validate_input = validate,
        .validate_output = validate,
        .max_recursion_depth = 100,
    };

    if (debug) {
            print("[main.zig][main()] === Zig AST Pipeline Starting ===\n", .{});
        print("[main.zig][main()] Input file: {s}\n", .{input_file});
        if (output_file) |out_file| {
            print("[main.zig][main()] Output file: {s}\n", .{out_file});
        } else {
            print("[main.zig][main()] Mode: Same-language (in-memory)\n", .{});
        }
        print("[main.zig][main()] Configuration: debug={}, annotations={}, validation={}\n", .{ debug, preserve_annotations, validate });
    }

    var pipeline = ZigASTPipeline.init(allocator, config);
    defer pipeline.deinit();

    var result = pipeline.transformFromFile(input_file, output_file) catch |err| {
        std.log.err("[main.zig][main()] Transformation error: {}", .{err});
        switch (err) {
            error.FileNotFound => print("Error: Input file '{s}' not found\n", .{input_file}),
            error.MissingGrammarName => print("Error: Missing 'grammar_name' field in JSON\n", .{}),
            error.MissingRawAST => print("Error: Missing 'raw_ast' field in JSON\n", .{}),
            error.MissingMetadata => print("Error: Missing 'metadata' field in JSON\n", .{}),
            error.EmptyGrammarName => print("Error: Grammar name cannot be empty\n", .{}),
            error.RawASTNotArray => print("Error: 'raw_ast' field must be an array\n", .{}),
            error.InvalidFormat => print("Error: Invalid metadata format (expected 'raw_ast')\n", .{}),
            else => print("Error: Transformation failed with error: {}\n", .{err}),
        }
        std.process.exit(1);
    };
    defer result.deinit(allocator);

    // Success output
    if (output_file) |out_file| {
        print("Zig AST Pipeline: Transformed AST saved to: {s}\n", .{out_file});
    } else {
        print("Zig AST Pipeline: Transformed AST loaded in-memory: {} rules\n", .{result.grammar_tree.count()});
        if (debug) {
            print("[main.zig][main()] Rule order: ", .{});
            for (result.rule_order.items, 0..) |rule, idx| {
                if (idx > 0) print(", ", .{});
                print("{s}", .{rule});
            }
            print("\n", .{});
        }
    }

    // Show statistics if requested
    if (stats) {
        print("\n=== Zig AST Pipeline Statistics ===\n", .{});
        print("  Backend Language: Zig\n", .{});
        print("  Pipeline Version: v1.0\n", .{});
        print("  Rules processed: {}\n", .{pipeline.stats.rules_processed});
        print("  Annotations preserved: {}\n", .{pipeline.stats.annotations_preserved});
        print("  Transformations applied: {}\n", .{pipeline.stats.transformations_applied});
        print("  5-Stage Pipeline: Extract -> Group -> Parentheses -> Sequences -> Quantifiers -> Tree\n", .{});
        print("====================================\n", .{});
    }

    if (debug) {
        print("[main.zig][main()] === Zig AST Pipeline Completed Successfully ===\n", .{});
    }
}
