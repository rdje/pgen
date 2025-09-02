const std = @import("std");
const testing = std.testing;
const ast_pipeline = @import("ast_pipeline.zig");

const ZigASTPipeline = ast_pipeline.ZigASTPipeline;
const PipelineConfig = ast_pipeline.PipelineConfig;

// Create a test raw AST JSON file with correct annotation format
fn createTestRawASTFile(allocator: std.mem.Allocator, file_path: []const u8) !void {
    const file = try std.fs.cwd().createFile(file_path, .{});
    defer file.close();

    const writer = file.writer();
    try writer.writeAll(
        \\{
        \\  "grammar_name": "test_grammar",
        \\  "raw_ast": [
        \\    [
        \\      ["rule", "expression"],
        \\      ["semantic_annotation", "[\"description\", \"This represents an expression\"]"],
        \\      ["identifier", "term"],
        \\      ["operator", "+"],
        \\      ["identifier", "factor"]
        \\    ],
        \\    [
        \\      ["rule", "term"],
        \\      ["logging_annotation", "[\"debug\", [\"processing\", \"term\"]]"],
        \\      ["identifier", "factor"],
        \\      ["operator", "*"],
        \\      ["identifier", "atom"]
        \\    ],
        \\    [
        \\      ["rule", "factor"],
        \\      ["return_scalar", "number"],
        \\      ["number", "42"]
        \\    ]
        \\  ],
        \\  "metadata": {
        \\    "format": "raw_ast",
        \\    "source": "integration_test"
        \\  }
        \\}
    );
}

// Integration test: Full pipeline from file to file
test "Full pipeline: file to file transformation" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const input_file = "test_input.json";
    const output_file = "test_output.json";

    // Create test input file
    try createTestRawASTFile(allocator, input_file);
    defer std.fs.cwd().deleteFile(input_file) catch {};
    defer std.fs.cwd().deleteFile(output_file) catch {};

    const config = PipelineConfig{
        .debug = false,
        .preserve_annotations = true,
        .validate_input = true,
        .validate_output = true,
    };

    var pipeline = ZigASTPipeline.init(allocator, config);
    defer pipeline.deinit();

    // Transform file to file
    const result = try pipeline.transformFromFile(input_file, output_file);
    defer {
        var iter = result.grammar_tree.iterator();
        while (iter.next()) |entry| {
            var node = entry.value_ptr;
            node.deinit(allocator);
        }
        result.grammar_tree.deinit();
        result.rule_order.deinit();
    }

    // Verify output file exists
    const output_exists = std.fs.cwd().access(output_file, .{}) catch false;
    try testing.expect(output_exists);

    // Verify transformation statistics
    try testing.expect(pipeline.stats.transformations_applied == 5);
    try testing.expect(pipeline.stats.rules_processed == result.grammar_tree.count());
    try testing.expect(pipeline.stats.annotations_preserved > 0);

    // Verify structure
    try testing.expect(result.grammar_tree.count() >= 3); // expression, term, factor
    try testing.expect(result.rule_order.items.len >= 3);
}

// Integration test: Same-language API (file to memory)
test "Same-language API: file to memory transformation" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const input_file = "test_memory_input.json";

    // Create test input file
    try createTestRawASTFile(allocator, input_file);
    defer std.fs.cwd().deleteFile(input_file) catch {};

    const config = PipelineConfig{
        .debug = true,
        .preserve_annotations = true,
    };

    var pipeline = ZigASTPipeline.init(allocator, config);
    defer pipeline.deinit();

    // Transform file to memory
    const result = try pipeline.transformFromFile(input_file, null);
    defer {
        var iter = result.grammar_tree.iterator();
        while (iter.next()) |entry| {
            var node = entry.value_ptr;
            node.deinit(allocator);
        }
        result.grammar_tree.deinit();
        result.rule_order.deinit();
    }

    // Verify in-memory structure
    try testing.expect(result.grammar_tree.count() >= 3);
    try testing.expect(result.rule_order.items.len >= 3);

    // Check that expected rules are present
    const expression_rule = result.grammar_tree.get("expression");
    const term_rule = result.grammar_tree.get("term");
    const factor_rule = result.grammar_tree.get("factor");

    try testing.expect(expression_rule != null);
    try testing.expect(term_rule != null);
    try testing.expect(factor_rule != null);

    // Verify annotations were preserved
    try testing.expect(pipeline.stats.annotations_preserved >= 2);
}

// Integration test: Error handling
test "Error handling: invalid input file" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const config = PipelineConfig{};
    var pipeline = ZigASTPipeline.init(allocator, config);
    defer pipeline.deinit();

    // Test with non-existent file
    const result = pipeline.transformFromFile("non_existent_file.json", null);
    try testing.expectError(error.FileNotFound, result);
}

// Integration test: Malformed JSON handling
test "Error handling: malformed JSON" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const malformed_file = "malformed_test.json";

    // Create malformed JSON file
    const file = try std.fs.cwd().createFile(malformed_file, .{});
    defer file.close();
    defer std.fs.cwd().deleteFile(malformed_file) catch {};

    try file.writeAll("{ invalid json content");

    const config = PipelineConfig{};
    var pipeline = ZigASTPipeline.init(allocator, config);
    defer pipeline.deinit();

    // Should fail with JSON parsing error
    const result = pipeline.transformFromFile(malformed_file, null);
    try testing.expectError(error.InvalidJson, result);
}

// Integration test: Empty grammar handling
test "Edge case: empty grammar" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const empty_file = "empty_grammar_test.json";

    // Create empty grammar file
    const file = try std.fs.cwd().createFile(empty_file, .{});
    defer file.close();
    defer std.fs.cwd().deleteFile(empty_file) catch {};

    try file.writeAll(
        \\{
        \\  "grammar_name": "empty_grammar",
        \\  "raw_ast": [],
        \\  "metadata": {
        \\    "format": "raw_ast"
        \\  }
        \\}
    );

    const config = PipelineConfig{};
    var pipeline = ZigASTPipeline.init(allocator, config);
    defer pipeline.deinit();

    const result = try pipeline.transformFromFile(empty_file, null);
    defer {
        result.grammar_tree.deinit();
        result.rule_order.deinit();
    }

    // Should handle empty grammar gracefully
    try testing.expect(result.grammar_tree.count() == 0);
    try testing.expect(result.rule_order.items.len == 0);
    try testing.expect(pipeline.stats.rules_processed == 0);
}

// Integration test: Complex grammar with multiple features
test "Complex grammar transformation" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const complex_file = "complex_test.json";

    // Create complex grammar file with OR operators, quantifiers, and groups
    const file = try std.fs.cwd().createFile(complex_file, .{});
    defer file.close();
    defer std.fs.cwd().deleteFile(complex_file) catch {};

    try file.writeAll(
        \\{
        \\  "grammar_name": "complex_grammar",
        \\  "raw_ast": [
        \\    [
        \\      ["rule", "statement"],
        \\      ["semantic_annotation", "[\"type\", \"A statement in the language\"]"],
        \\      ["identifier", "if_stmt"],
        \\      ["operator", "|"],
        \\      ["identifier", "while_stmt"],
        \\      ["operator", "|"],
        \\      ["identifier", "assignment"]
        \\    ],
        \\    [
        \\      ["rule", "if_stmt"],
        \\      ["logging_annotation", "[\"trace\", [\"processing\", \"if\", \"statement\"]]"],
        \\      ["keyword", "if"],
        \\      ["group_open", "("],
        \\      ["identifier", "condition"],
        \\      ["group_close", ")"],
        \\      ["identifier", "block"]
        \\    ],
        \\    [
        \\      ["rule", "list"],
        \\      ["return_array", "items"],
        \\      ["identifier", "item"],
        \\      ["operator", "*"]
        \\    ]
        \\  ],
        \\  "metadata": {
        \\    "format": "raw_ast",
        \\    "features": ["or_operators", "groups", "quantifiers", "annotations"]
        \\  }
        \\}
    );

    const config = PipelineConfig{
        .debug = true,
        .preserve_annotations = true,
    };

    var pipeline = ZigASTPipeline.init(allocator, config);
    defer pipeline.deinit();

    const result = try pipeline.transformFromFile(complex_file, null);
    defer {
        var iter = result.grammar_tree.iterator();
        while (iter.next()) |entry| {
            var node = entry.value_ptr;
            node.deinit(allocator);
        }
        result.grammar_tree.deinit();
        result.rule_order.deinit();
    }

    // Verify complex transformation
    try testing.expect(result.grammar_tree.count() >= 3);
    try testing.expect(pipeline.stats.annotations_preserved >= 2);
    try testing.expect(pipeline.stats.transformations_applied == 5);

    // Check specific rules
    const statement_rule = result.grammar_tree.get("statement");
    const if_stmt_rule = result.grammar_tree.get("if_stmt");
    const list_rule = result.grammar_tree.get("list");

    try testing.expect(statement_rule != null);
    try testing.expect(if_stmt_rule != null);
    try testing.expect(list_rule != null);
}

// Test runner
test {
    std.testing.refAllDecls(@This());
}
