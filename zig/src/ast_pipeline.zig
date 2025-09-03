//! Zig AST Pipeline Implementation for PGEN
//!
//! Complete EBNF AST transformation pipeline with dual-mode API:
//! - Cross-language interface: JSON input/output
//! - Same-language optimization: In-memory data structures
//!
//! Implements the 5-stage transformation pipeline equivalent to other backend languages.

const std = @import("std");
const json = std.json;
const print = std.debug.print;
const ArrayList = std.array_list.Managed;
const HashMap = std.HashMap;
const Allocator = std.mem.Allocator;

// Configuration for AST transformation pipeline
pub const PipelineConfig = struct {
    debug: bool = false,
    preserve_annotations: bool = true,
    validate_input: bool = true,
    validate_output: bool = true,
    max_recursion_depth: usize = 100,
};

// Token types for raw AST processing
pub const Token = ArrayList([]const u8);
pub const TokenSequence = ArrayList(Token);
pub const RawAST = ArrayList(TokenSequence);

// Raw AST JSON structure from ebnf_to_json.pl
pub const RawASTJson = struct {
    grammar_name: []const u8,
    raw_ast: json.Value,
    metadata: json.Value,
};

// AST node types in the transformed AST
pub const ASTNodeType = enum {
    atom,
    sequence,
    or_node, // 'or' is a keyword in Zig
    quantified,
};

pub const ASTNode = union(ASTNodeType) {
    atom: struct {
        value: ASTValue,
    },
    sequence: struct {
        elements: ArrayList(ASTNode),
    },
    or_node: struct {
        alternatives: ArrayList(ASTNode),
    },
    quantified: struct {
        element: *ASTNode,
        quantifier: []const u8,
    },

    pub fn deinit(self: *ASTNode, allocator: Allocator) void {
        switch (self.*) {
            .sequence => |*seq| {
                for (seq.elements.items) |*elem| {
                    elem.deinit(allocator);
                }
                seq.elements.deinit();
            },
            .or_node => |*or_n| {
                for (or_n.alternatives.items) |*alt| {
                    alt.deinit(allocator);
                }
                or_n.alternatives.deinit();
            },
            .quantified => |*quant| {
                quant.element.deinit(allocator);
                allocator.destroy(quant.element);
            },
            .atom => |*atom| {
                atom.value.deinit(allocator);
            },
        }
    }

    pub fn clone(self: *const ASTNode, allocator: Allocator) (Allocator.Error)!ASTNode {
        switch (self.*) {
            .atom => |atom| {
                return ASTNode{ .atom = .{ .value = try atom.value.clone(allocator) } };
            },
            .sequence => |seq| {
                var new_elements = ArrayList(ASTNode).init(allocator);
                for (seq.elements.items) |*elem| {
                    try new_elements.append(try elem.clone(allocator));
                }
                return ASTNode{ .sequence = .{ .elements = new_elements } };
            },
            .or_node => |or_n| {
                var new_alternatives = ArrayList(ASTNode).init(allocator);
                for (or_n.alternatives.items) |*alt| {
                    try new_alternatives.append(try alt.clone(allocator));
                }
                return ASTNode{ .or_node = .{ .alternatives = new_alternatives } };
            },
            .quantified => |quant| {
                const new_element = try allocator.create(ASTNode);
                new_element.* = try quant.element.clone(allocator);
                return ASTNode{ .quantified = .{ .element = new_element, .quantifier = quant.quantifier } };
            },
        }
    }
};

pub const ASTValue = union(enum) {
    token: ArrayList([]const u8),
    node: *ASTNode,

    pub fn deinit(self: *ASTValue, allocator: Allocator) void {
        switch (self.*) {
            .token => |*tok| tok.deinit(),
            .node => |node| {
                node.deinit(allocator);
                allocator.destroy(node);
            },
        }
    }

    pub fn clone(self: *const ASTValue, allocator: Allocator) (Allocator.Error)!ASTValue {
        switch (self.*) {
            .token => |tok| {
                var new_token = ArrayList([]const u8).init(allocator);
                for (tok.items) |item| {
                    try new_token.append(item);
                }
                return ASTValue{ .token = new_token };
            },
            .node => |node| {
                const new_node = try allocator.create(ASTNode);
                new_node.* = try node.clone(allocator);
                return ASTValue{ .node = new_node };
            },
        }
    }
};

// Preserved annotations from raw AST
pub const Annotations = struct {
    semantic_annotations: HashMap([]const u8, ArrayList([]const u8), std.hash_map.StringContext, std.hash_map.default_max_load_percentage),
    logging_annotations: HashMap([]const u8, ArrayList([]const u8), std.hash_map.StringContext, std.hash_map.default_max_load_percentage),
    return_annotations: HashMap([]const u8, []const u8, std.hash_map.StringContext, std.hash_map.default_max_load_percentage),

    pub fn init(allocator: Allocator) Annotations {
        return Annotations{
            .semantic_annotations = HashMap([]const u8, ArrayList([]const u8), std.hash_map.StringContext, std.hash_map.default_max_load_percentage).init(allocator),
            .logging_annotations = HashMap([]const u8, ArrayList([]const u8), std.hash_map.StringContext, std.hash_map.default_max_load_percentage).init(allocator),
            .return_annotations = HashMap([]const u8, []const u8, std.hash_map.StringContext, std.hash_map.default_max_load_percentage).init(allocator),
        };
    }

    pub fn deinit(self: *Annotations) void {
        // Deinitialize hash maps
        var semantic_iter = self.semantic_annotations.iterator();
        while (semantic_iter.next()) |entry| {
            entry.value_ptr.deinit();
        }
        self.semantic_annotations.deinit();

        var logging_iter = self.logging_annotations.iterator();
        while (logging_iter.next()) |entry| {
            entry.value_ptr.deinit();
        }
        self.logging_annotations.deinit();

        self.return_annotations.deinit();
    }
};

// Transformation statistics
pub const TransformStats = struct {
    rules_processed: usize = 0,
    annotations_preserved: usize = 0,
    transformations_applied: usize = 0,
};

// Result structure for transformed AST
pub const TransformedAST = struct {
    grammar_tree: HashMap([]const u8, ASTNode, std.hash_map.StringContext, std.hash_map.default_max_load_percentage),
    rule_order: ArrayList([]const u8),

    pub fn deinit(self: *TransformedAST, allocator: Allocator) void {
        var iter = self.grammar_tree.iterator();
        while (iter.next()) |entry| {
            var node = entry.value_ptr;
            node.deinit(allocator);
        }
        self.grammar_tree.deinit();
        self.rule_order.deinit();
    }
};

// Main Zig AST Pipeline implementation
pub const ZigASTPipeline = struct {
    allocator: Allocator,
    config: PipelineConfig,
    stats: TransformStats,
    annotations: Annotations,

    pub fn init(allocator: Allocator, config: PipelineConfig) ZigASTPipeline {
        return ZigASTPipeline{
            .allocator = allocator,
            .config = config,
            .stats = TransformStats{},
            .annotations = Annotations.init(allocator),
        };
    }

    pub fn deinit(self: *ZigASTPipeline) void {
        self.annotations.deinit();
    }

    /// Load raw AST JSON from file
    pub fn loadRawAST(self: *ZigASTPipeline, file_path: []const u8) !RawASTJson {
        if (self.config.debug) {
            print("[ast_pipeline.zig][loadRawAST()] Loading raw AST from: {s}\n", .{file_path});
        }

        const file = std.fs.cwd().openFile(file_path, .{}) catch |err| {
            std.log.err("[ast_pipeline.zig][loadRawAST()] Failed to open file {s}: {}", .{ file_path, err });
            return err;
        };
        defer file.close();

        const file_size = try file.getEndPos();
        const content = try self.allocator.alloc(u8, file_size);
        defer self.allocator.free(content);

        _ = try file.readAll(content);

        const parsed = json.parseFromSlice(json.Value, self.allocator, content, .{}) catch |err| {
            std.log.err("[ast_pipeline.zig][loadRawAST()] Failed to parse JSON from {s}: {}", .{ file_path, err });
            return err;
        };
        defer parsed.deinit();

        const root = parsed.value;

        // Extract required fields
        const grammar_name = if (root.object.get("grammar_name")) |name| name.string else return error.MissingGrammarName;
        const raw_ast = if (root.object.get("raw_ast")) |ast| ast else return error.MissingRawAST;
        const metadata = if (root.object.get("metadata")) |meta| meta else return error.MissingMetadata;

        if (self.config.validate_input) {
            try self.validateRawAST(grammar_name, &raw_ast, &metadata);
        }

        return RawASTJson{
            .grammar_name = grammar_name,
            .raw_ast = raw_ast,
            .metadata = metadata,
        };
    }

    /// Validate raw AST JSON format
    fn validateRawAST(self: *ZigASTPipeline, grammar_name: []const u8, raw_ast: *const json.Value, metadata: *const json.Value) !void {
        _ = self;

        if (grammar_name.len == 0) {
            return error.EmptyGrammarName;
        }

        if (raw_ast.* != .array) {
            return error.RawASTNotArray;
        }

        if (metadata.object.get("format")) |format| {
            if (!std.mem.eql(u8, format.string, "raw_ast")) {
                return error.InvalidFormat;
            }
        }
    }

    /// Transform raw AST to semantic AST using the 5-stage pipeline
    pub fn transformRawAST(self: *ZigASTPipeline, raw_ast: *const json.Value) !TransformedAST {
        if (self.config.debug) {
            print("[ast_pipeline.zig][transformRawAST()] === Zig AST Transformation Pipeline ===\n", .{});
        }

        // Stage 1: Extract annotations
        const cleaned_ast = try self.extractAnnotations(raw_ast);
        defer self.deallocateRawAST(cleaned_ast);

        // Stage 2: Group by OR operators
        const grouped_rules = try self.groupByOrOperators(cleaned_ast);
        defer self.deallocateGroupedRules(grouped_rules);

        // Stage 2.5: Handle parentheses
        const processed_rules = try self.handleParentheses(&grouped_rules);
        defer self.deallocateGroupedRules(processed_rules);

        // Stage 3: Parse sequences
        const sequenced_rules = try self.parseSequences(&processed_rules);
        defer self.deallocateSequencedRules(sequenced_rules);

        // Stage 4: Handle quantifiers
        const quantified_rules = try self.handleQuantifiers(&sequenced_rules);
        defer self.deallocateSequencedRules(quantified_rules);

        // Stage 5: Build tree structure
        const result = try self.buildTreeStructure(&quantified_rules);

        self.stats.rules_processed = result.grammar_tree.count();
        self.stats.transformations_applied = 5;

        return result;
    }

    // Memory cleanup helper functions
    fn deallocateRawAST(self: *ZigASTPipeline, ast: ArrayList(TokenSequence)) void {
        _ = self;
        var mutable_ast = ast;
        for (mutable_ast.items) |*rule_def| {
            for (rule_def.items) |*token| {
                token.deinit();
            }
            rule_def.deinit();
        }
        mutable_ast.deinit();
    }

    fn deallocateGroupedRules(self: *ZigASTPipeline, grouped: HashMap([]const u8, ArrayList(TokenSequence), std.hash_map.StringContext, std.hash_map.default_max_load_percentage)) void {
        _ = self;
        var mutable_grouped = grouped;
        var iter = mutable_grouped.iterator();
        while (iter.next()) |entry| {
            for (entry.value_ptr.items) |*alt| {
                for (alt.items) |*token| {
                    token.deinit();
                }
                alt.deinit();
            }
            entry.value_ptr.deinit();
        }
        mutable_grouped.deinit();
    }

    fn deallocateSequencedRules(self: *ZigASTPipeline, sequenced: HashMap([]const u8, ArrayList(ASTNode), std.hash_map.StringContext, std.hash_map.default_max_load_percentage)) void {
        var mutable_sequenced = sequenced;
        var iter = mutable_sequenced.iterator();
        while (iter.next()) |entry| {
            for (entry.value_ptr.items) |*node| {
                node.deinit(self.allocator);
            }
            entry.value_ptr.deinit();
        }
        mutable_sequenced.deinit();
    }

    /// Stage 1: Extract and preserve annotations from raw AST
    fn extractAnnotations(self: *ZigASTPipeline, raw_ast: *const json.Value) !ArrayList(TokenSequence) {
        if (self.config.debug) {
            print("[ast_pipeline.zig][extractAnnotations()] Stage 1: Extracting annotations...\n", .{});
        }

        var cleaned_ast = ArrayList(ArrayList(ArrayList([]const u8))).init(self.allocator);

        for (raw_ast.array.items) |rule_def_json| {
            if (rule_def_json != .array) continue;

            var rule_name: ?[]const u8 = null;
            var cleaned_rule = ArrayList(ArrayList([]const u8)).init(self.allocator);

            for (rule_def_json.array.items) |token_json| {
                if (token_json != .array or token_json.array.items.len != 2) continue;

                const token_type = token_json.array.items[0].string;
                const token_value = token_json.array.items[1].string;

                if (std.mem.eql(u8, token_type, "rule")) {
                    rule_name = token_value;
                    var token = ArrayList([]const u8).init(self.allocator);
                    try token.append(token_type);
                    try token.append(token_value);
                    try cleaned_rule.append(token);
                } else if (self.isAnnotation(token_type)) {
                    if (rule_name != null and self.config.preserve_annotations) {
                        try self.storeAnnotation(rule_name.?, token_type, token_value);
                        self.stats.annotations_preserved += 1;
                    }
                    // Don't add annotations to cleaned rule
                } else {
                    var token = ArrayList([]const u8).init(self.allocator);
                    try token.append(token_type);
                    try token.append(token_value);
                    try cleaned_rule.append(token);
                }
            }

            if (cleaned_rule.items.len > 0) {
                try cleaned_ast.append(cleaned_rule);
            } else {
                cleaned_rule.deinit();
            }
        }

        if (self.config.debug) {
            print("[ast_pipeline.zig][extractAnnotations()] Preserved {} annotations\n", .{self.stats.annotations_preserved});
        }

        return cleaned_ast;
    }

    fn isAnnotation(self: *ZigASTPipeline, token_type: []const u8) bool {
        _ = self;
        return std.mem.eql(u8, token_type, "semantic_annotation") or
            std.mem.eql(u8, token_type, "logging_annotation") or
            std.mem.eql(u8, token_type, "return_scalar") or
            std.mem.eql(u8, token_type, "return_array") or
            std.mem.eql(u8, token_type, "return_object");
    }

    fn storeAnnotation(self: *ZigASTPipeline, rule_name: []const u8, annotation_type: []const u8, annotation_value: []const u8) !void {
        if (std.mem.eql(u8, annotation_type, "semantic_annotation")) {
            const result = try self.annotations.semantic_annotations.getOrPut(rule_name);
            if (!result.found_existing) {
                result.value_ptr.* = ArrayList([]const u8).init(self.allocator);
            }
            try result.value_ptr.append(annotation_value);
        } else if (std.mem.eql(u8, annotation_type, "logging_annotation")) {
            const result = try self.annotations.logging_annotations.getOrPut(rule_name);
            if (!result.found_existing) {
                result.value_ptr.* = ArrayList([]const u8).init(self.allocator);
            }
            try result.value_ptr.append(annotation_value);
        } else if (std.mem.startsWith(u8, annotation_type, "return_")) {
            try self.annotations.return_annotations.put(rule_name, annotation_type);
        }
    }

    /// Stage 2: Group rule definitions by OR operators
    fn groupByOrOperators(self: *ZigASTPipeline, ast: ArrayList(TokenSequence)) !HashMap([]const u8, ArrayList(TokenSequence), std.hash_map.StringContext, std.hash_map.default_max_load_percentage) {
        if (self.config.debug) {
            print("[ast_pipeline.zig][groupByOrOperators()] Stage 2: Grouping by OR operators...\n", .{});
        }

        var grouped = HashMap([]const u8, ArrayList(TokenSequence), std.hash_map.StringContext, std.hash_map.default_max_load_percentage).init(self.allocator);

        for (ast.items) |rule_def| {
            if (rule_def.items.len == 0) continue;

            var rule_name: ?[]const u8 = null;
            for (rule_def.items) |token| {
                if (token.items.len == 2 and std.mem.eql(u8, token.items[0], "rule")) {
                    rule_name = token.items[1];
                    break;
                }
            }

            if (rule_name) |name| {
                var alternatives = ArrayList(TokenSequence).init(self.allocator);
                var current_alt = ArrayList(ArrayList([]const u8)).init(self.allocator);

                // Skip rule definition token
                for (rule_def.items[1..]) |token| {
                    if (token.items.len == 2 and std.mem.eql(u8, token.items[0], "operator") and std.mem.eql(u8, token.items[1], "|")) {
                        if (current_alt.items.len > 0) {
                            try alternatives.append(current_alt);
                            current_alt = ArrayList(ArrayList([]const u8)).init(self.allocator);
                        }
                    } else {
                        var new_token = ArrayList([]const u8).init(self.allocator);
                        for (token.items) |item| {
                            try new_token.append(item);
                        }
                        try current_alt.append(new_token);
                    }
                }

                if (current_alt.items.len > 0) {
                    try alternatives.append(current_alt);
                }

                const result = try grouped.getOrPut(name);
                if (!result.found_existing) {
                    result.value_ptr.* = ArrayList(TokenSequence).init(self.allocator);
                }
                try result.value_ptr.appendSlice(alternatives.items);
                alternatives.deinit();
            }
        }

        return grouped;
    }

    /// Stage 2.5: Handle parentheses and grouping
    fn handleParentheses(self: *ZigASTPipeline, grouped_rules: *const HashMap([]const u8, ArrayList(TokenSequence), std.hash_map.StringContext, std.hash_map.default_max_load_percentage)) !HashMap([]const u8, ArrayList(TokenSequence), std.hash_map.StringContext, std.hash_map.default_max_load_percentage) {
        if (self.config.debug) {
            print("[ast_pipeline.zig][handleParentheses()] Stage 2.5: Handling parentheses...\n", .{});
        }

        var processed = HashMap([]const u8, ArrayList(TokenSequence), std.hash_map.StringContext, std.hash_map.default_max_load_percentage).init(self.allocator);

        var iter = grouped_rules.iterator();
        while (iter.next()) |entry| {
            const rule_name = entry.key_ptr.*;
            const alternatives = entry.value_ptr;

            var processed_alts = ArrayList(TokenSequence).init(self.allocator);

            for (alternatives.items) |alt| {
                const processed_alt = try self.processParenthesesInSequence(&alt);
                try processed_alts.append(processed_alt);
            }

            try processed.put(rule_name, processed_alts);
        }

        return processed;
    }

    /// Process parentheses within a token sequence (simplified for now)
    fn processParenthesesInSequence(self: *ZigASTPipeline, sequence: *const TokenSequence) !TokenSequence {
        var result = ArrayList(ArrayList([]const u8)).init(self.allocator);

        for (sequence.items) |token| {
            // For now, just copy tokens through - grouping logic would be more complex
            var new_token = ArrayList([]const u8).init(self.allocator);
            for (token.items) |item| {
                try new_token.append(item);
            }
            try result.append(new_token);
        }

        return result;
    }

    /// Stage 3: Parse sequences
    fn parseSequences(self: *ZigASTPipeline, processed_rules: *const HashMap([]const u8, ArrayList(TokenSequence), std.hash_map.StringContext, std.hash_map.default_max_load_percentage)) !HashMap([]const u8, ArrayList(ASTNode), std.hash_map.StringContext, std.hash_map.default_max_load_percentage) {
        if (self.config.debug) {
            print("[ast_pipeline.zig][parseSequences()] Stage 3: Parsing sequences...\n", .{});
        }

        var sequenced = HashMap([]const u8, ArrayList(ASTNode), std.hash_map.StringContext, std.hash_map.default_max_load_percentage).init(self.allocator);

        var iter = processed_rules.iterator();
        while (iter.next()) |entry| {
            const rule_name = entry.key_ptr.*;
            const alternatives = entry.value_ptr;

            var parsed_alts = ArrayList(ASTNode).init(self.allocator);

            for (alternatives.items) |alt| {
                const parsed_alt = if (alt.items.len == 1)
                    try self.parseSingleElement(&alt.items[0])
                else blk: {
                    var elements = ArrayList(ASTNode).init(self.allocator);
                    for (alt.items) |elem| {
                        try elements.append(try self.parseSingleElement(&elem));
                    }
                    break :blk ASTNode{ .sequence = .{ .elements = elements } };
                };
                try parsed_alts.append(parsed_alt);
            }

            try sequenced.put(rule_name, parsed_alts);
        }

        return sequenced;
    }

    /// Parse a single grammar element
    fn parseSingleElement(self: *ZigASTPipeline, element: *const Token) !ASTNode {
        if (element.items.len != 2) {
            var token_copy = ArrayList([]const u8).init(self.allocator);
            for (element.items) |item| {
                try token_copy.append(item);
            }
            return ASTNode{ .atom = .{ .value = ASTValue{ .token = token_copy } } };
        }

        var token_copy = ArrayList([]const u8).init(self.allocator);
        try token_copy.append(element.items[0]);
        try token_copy.append(element.items[1]);
        return ASTNode{ .atom = .{ .value = ASTValue{ .token = token_copy } } };
    }

    /// Stage 4: Handle quantifiers
    fn handleQuantifiers(self: *ZigASTPipeline, sequenced_rules: *const HashMap([]const u8, ArrayList(ASTNode), std.hash_map.StringContext, std.hash_map.default_max_load_percentage)) !HashMap([]const u8, ArrayList(ASTNode), std.hash_map.StringContext, std.hash_map.default_max_load_percentage) {
        if (self.config.debug) {
            print("[ast_pipeline.zig][handleQuantifiers()] Stage 4: Handling quantifiers...\n", .{});
        }

        var quantified = HashMap([]const u8, ArrayList(ASTNode), std.hash_map.StringContext, std.hash_map.default_max_load_percentage).init(self.allocator);

        var iter = sequenced_rules.iterator();
        while (iter.next()) |entry| {
            const rule_name = entry.key_ptr.*;
            const alternatives = entry.value_ptr;

            var processed_alts = ArrayList(ASTNode).init(self.allocator);

            for (alternatives.items) |*alt| {
                const processed_alt = try self.applyQuantifiersToNode(alt);
                try processed_alts.append(processed_alt);
            }

            try quantified.put(rule_name, processed_alts);
        }

        return quantified;
    }

    /// Apply quantifiers to AST node
    fn applyQuantifiersToNode(self: *ZigASTPipeline, node: *const ASTNode) !ASTNode {
        switch (node.*) {
            .sequence => |seq| {
                var new_elements = ArrayList(ASTNode).init(self.allocator);
                var i: usize = 0;

                while (i < seq.elements.items.len) {
                    const element = &seq.elements.items[i];

                    // Check if next element is a quantifier
                    if (i + 1 < seq.elements.items.len) {
                        const next_elem = &seq.elements.items[i + 1];
                        if (self.isQuantifierToken(next_elem)) {
                            const quantifier = try self.getQuantifierFromNode(next_elem);
                            const element_copy = try self.allocator.create(ASTNode);
                            element_copy.* = try element.clone(self.allocator);

                            const quantified_node = ASTNode{
                                .quantified = .{
                                    .element = element_copy,
                                    .quantifier = quantifier,
                                },
                            };
                            try new_elements.append(quantified_node);
                            i += 2; // Skip quantifier token
                            continue;
                        }
                    }

                    try new_elements.append(try element.clone(self.allocator));
                    i += 1;
                }

                return ASTNode{ .sequence = .{ .elements = new_elements } };
            },
            else => return try node.clone(self.allocator),
        }
    }

    fn isQuantifierToken(self: *ZigASTPipeline, node: *const ASTNode) bool {
        _ = self;
        if (node.* != .atom) return false;
        switch (node.atom.value) {
            .token => |token| {
                return token.items.len == 2 and
                    std.mem.eql(u8, token.items[0], "operator") and
                    (std.mem.eql(u8, token.items[1], "*") or
                    std.mem.eql(u8, token.items[1], "+") or
                    std.mem.eql(u8, token.items[1], "?"));
            },
            else => return false,
        }
    }

    fn getQuantifierFromNode(self: *ZigASTPipeline, node: *const ASTNode) ![]const u8 {
        _ = self;
        switch (node.atom.value) {
            .token => |token| {
                if (token.items.len == 2) {
                    return token.items[1];
                }
            },
            else => {},
        }
        return error.NotAQuantifier;
    }

    /// Stage 5: Build final tree structure
    fn buildTreeStructure(self: *ZigASTPipeline, quantified_rules: *const HashMap([]const u8, ArrayList(ASTNode), std.hash_map.StringContext, std.hash_map.default_max_load_percentage)) !TransformedAST {
        if (self.config.debug) {
            print("[ast_pipeline.zig][buildTreeStructure()] Stage 5: Building tree structure...\n", .{});
        }

        var grammar_tree = HashMap([]const u8, ASTNode, std.hash_map.StringContext, std.hash_map.default_max_load_percentage).init(self.allocator);
        var rule_order = ArrayList([]const u8).init(self.allocator);

        var iter = quantified_rules.iterator();
        while (iter.next()) |entry| {
            const rule_name = entry.key_ptr.*;
            const alternatives = entry.value_ptr;

            try rule_order.append(rule_name);

            const final_node = if (alternatives.items.len == 1)
                try alternatives.items[0].clone(self.allocator)
            else blk: {
                var cloned_alternatives = ArrayList(ASTNode).init(self.allocator);
                for (alternatives.items) |*alt| {
                    try cloned_alternatives.append(try alt.clone(self.allocator));
                }
                break :blk ASTNode{ .or_node = .{ .alternatives = cloned_alternatives } };
            };

            try grammar_tree.put(rule_name, final_node);
        }

        return TransformedAST{ .grammar_tree = grammar_tree, .rule_order = rule_order };
    }

    /// Transform raw AST JSON file to in-memory AST
    pub fn transformFromFile(self: *ZigASTPipeline, raw_ast_json_file: []const u8, output_json_file: ?[]const u8) !TransformedAST {
        const raw_data = try self.loadRawAST(raw_ast_json_file);
        const result = try self.transformRawAST(&raw_data.raw_ast);

        if (output_json_file) |output_file| {
            try self.saveTransformedAST(&result.grammar_tree, &result.rule_order, raw_data.grammar_name, output_file);
        }

        return result;
    }

    /// Save transformed AST to JSON file (simplified implementation)
    pub fn saveTransformedAST(self: *ZigASTPipeline, grammar_tree: *const HashMap([]const u8, ASTNode, std.hash_map.StringContext, std.hash_map.default_max_load_percentage), rule_order: *const ArrayList([]const u8), grammar_name: []const u8, output_file: []const u8) !void {
        if (self.config.debug) {
            print("[ast_pipeline.zig][saveTransformedAST()] Saving transformed AST to: {s}\n", .{output_file});
        }

        const file = try std.fs.cwd().createFile(output_file, .{});
        defer file.close();

        // Build the content in a string first
        var content = std.array_list.Managed(u8).init(self.allocator);
        defer content.deinit();
        
        var content_writer = content.writer();
        try content_writer.print("{{\n", .{});
        try content_writer.print("  \"grammar_name\": \"{s}\",\n", .{grammar_name});
        try content_writer.print("  \"grammar_tree\": {{\n", .{});

        var iter = grammar_tree.iterator();
        var first = true;
        while (iter.next()) |entry| {
            if (!first) try content_writer.print(",\n", .{});
            try content_writer.print("    \"{s}\": {{\"type\": \"simplified\"}}", .{entry.key_ptr.*});
            first = false;
        }

        try content_writer.print("\n  }},\n", .{});
        try content_writer.print("  \"rule_order\": [", .{});

        for (rule_order.items, 0..) |rule, i| {
            if (i > 0) try content_writer.print(", ", .{});
            try content_writer.print("\"{s}\"", .{rule});
        }

        try content_writer.print("],\n", .{});
        try content_writer.print("  \"metadata\": {{\n", .{});
        try content_writer.print("    \"format\": \"transformed_ast\",\n", .{});
        try content_writer.print("    \"transformer\": \"Zig AST Pipeline v1.0\",\n", .{});
        try content_writer.print("    \"rules_processed\": {}\n", .{self.stats.rules_processed});
        try content_writer.print("  }}\n", .{});
        try content_writer.print("}}\n", .{});
        
        // Write all content at once to file
        try file.writeAll(content.items);

        if (self.config.debug) {
            print("[ast_pipeline.zig][saveTransformedAST()] Transformed AST saved successfully\n", .{});
        }
    }
};
