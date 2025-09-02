const std = @import("std");
const testing = std.testing;
const ast_pipeline = @import("ast_pipeline.zig");

const ZigASTPipeline = ast_pipeline.ZigASTPipeline;
const PipelineConfig = ast_pipeline.PipelineConfig;

test "ZigASTPipeline initialization" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const config = PipelineConfig{
        .debug = false,
        .preserve_annotations = true,
        .validate_input = true,
        .validate_output = true,
        .max_recursion_depth = 100,
    };

    var pipeline = ZigASTPipeline.init(allocator, config);
    defer pipeline.deinit();

    try testing.expect(pipeline.stats.rules_processed == 0);
    try testing.expect(pipeline.stats.annotations_preserved == 0);
    try testing.expect(pipeline.stats.transformations_applied == 0);
    try testing.expect(pipeline.config.debug == false);
    try testing.expect(pipeline.config.preserve_annotations == true);
}

test "Pipeline configuration options" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    // Test default configuration
    const default_config = PipelineConfig{};
    var pipeline1 = ZigASTPipeline.init(allocator, default_config);
    defer pipeline1.deinit();

    try testing.expect(pipeline1.config.debug == false);
    try testing.expect(pipeline1.config.preserve_annotations == true);
    try testing.expect(pipeline1.config.validate_input == true);
    try testing.expect(pipeline1.config.validate_output == true);
    try testing.expect(pipeline1.config.max_recursion_depth == 100);

    // Test custom configuration
    const custom_config = PipelineConfig{
        .debug = true,
        .preserve_annotations = false,
        .validate_input = false,
        .validate_output = false,
        .max_recursion_depth = 50,
    };
    var pipeline2 = ZigASTPipeline.init(allocator, custom_config);
    defer pipeline2.deinit();

    try testing.expect(pipeline2.config.debug == true);
    try testing.expect(pipeline2.config.preserve_annotations == false);
    try testing.expect(pipeline2.config.validate_input == false);
    try testing.expect(pipeline2.config.validate_output == false);
    try testing.expect(pipeline2.config.max_recursion_depth == 50);
}

test "Raw AST validation" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const config = PipelineConfig{ .validate_input = true };
    var pipeline = ZigASTPipeline.init(allocator, config);
    defer pipeline.deinit();

    // Valid metadata
    var valid_metadata = std.json.ObjectMap.init(allocator);
    defer valid_metadata.deinit();

    const format_value = std.json.Value{ .String = "raw_ast" };
    try valid_metadata.put("format", format_value);

    const metadata = std.json.Value{ .Object = valid_metadata };

    // Valid raw AST
    var empty_array = std.json.Array.init(allocator);
    defer empty_array.deinit();
    const raw_ast = std.json.Value{ .Array = empty_array };

    // Test with valid grammar name
    const grammar_name = "test_grammar";
    try pipeline.validateRawAST(grammar_name, &raw_ast, &metadata);

    // Test with empty grammar name - should fail
    const empty_name = "";
    const result = pipeline.validateRawAST(empty_name, &raw_ast, &metadata);
    try testing.expectError(error.EmptyGrammarName, result);
}

test "AST node creation and cleanup" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    // Test atom node creation
    var token = std.ArrayList([]const u8).init(allocator);
    defer token.deinit();
    try token.append("identifier");
    try token.append("example");

    var atom_node = ast_pipeline.ASTNode{
        .atom = .{
            .value = ast_pipeline.ASTValue{ .token = token },
        },
    };

    // Test that the node is properly constructed
    switch (atom_node.atom.value) {
        .token => |tok| {
            try testing.expectEqualStrings("identifier", tok.items[0]);
            try testing.expectEqualStrings("example", tok.items[1]);
        },
        else => return error.UnexpectedNodeType,
    }

    // No cleanup needed here since we're not actually calling deinit
}

test "Annotations storage" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    var annotations = ast_pipeline.Annotations.init(allocator);
    defer annotations.deinit();

    // Test adding semantic annotation
    const rule_name = "test_rule";
    const annotation = "test_annotation";

    const result = try annotations.semantic_annotations.getOrPut(rule_name);
    if (!result.found_existing) {
        result.value_ptr.* = std.ArrayList([]const u8).init(allocator);
    }
    try result.value_ptr.append(annotation);

    // Verify annotation was stored
    const stored_annotations = annotations.semantic_annotations.get(rule_name);
    try testing.expect(stored_annotations != null);
    if (stored_annotations) |annotations_list| {
        try testing.expect(annotations_list.items.len == 1);
        try testing.expectEqualStrings(annotation, annotations_list.items[0]);
    }
}

test "Mock transformation pipeline" {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const config = PipelineConfig{ .debug = false };
    var pipeline = ZigASTPipeline.init(allocator, config);
    defer pipeline.deinit();

    // Create a minimal mock raw AST JSON structure
    var raw_ast_array = std.json.Array.init(allocator);
    defer raw_ast_array.deinit();

    // Add a simple rule definition: ["rule", "test_rule"], ["identifier", "example"]
    var rule_def = std.json.Array.init(allocator);
    defer rule_def.deinit();

    var rule_token = std.json.Array.init(allocator);
    defer rule_token.deinit();
    try rule_token.append(std.json.Value{ .String = "rule" });
    try rule_token.append(std.json.Value{ .String = "test_rule" });
    try rule_def.append(std.json.Value{ .Array = rule_token });

    var id_token = std.json.Array.init(allocator);
    defer id_token.deinit();
    try id_token.append(std.json.Value{ .String = "identifier" });
    try id_token.append(std.json.Value{ .String = "example" });
    try rule_def.append(std.json.Value{ .Array = id_token });

    try raw_ast_array.append(std.json.Value{ .Array = rule_def });

    const raw_ast = std.json.Value{ .Array = raw_ast_array };

    // Test the transformation pipeline
    const result = pipeline.transformRawAST(&raw_ast) catch |err| {
        std.debug.print("Transformation error: {}\n", .{err});
        return err;
    };
    
    defer {
        var iter = result.grammar_tree.iterator();
        while (iter.next()) |entry| {
            var node = entry.value_ptr;
            node.deinit(allocator);
        }
        result.grammar_tree.deinit();
        result.rule_order.deinit();
    }

    // Verify that the transformation completed
    try testing.expect(pipeline.stats.transformations_applied == 5);
    try testing.expect(result.grammar_tree.count() > 0);
    try testing.expect(result.rule_order.items.len > 0);
}

// Test runner
test {
    std.testing.refAllDecls(@This());
}
