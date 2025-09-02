const std = @import("std");

pub fn build(b: *std.Build) void {
    // Create executable
    const exe = b.addExecutable(.{
        .name = "zig_ast_pipeline",
    });
    exe.root_module.root_source_file = b.path("ast_pipeline.zig");
    
    b.installArtifact(exe);
    
    // Run command
    const run_cmd = b.addRunArtifact(exe);
    const run_step = b.step("run", "Run the app");
    run_step.dependOn(&run_cmd.step);
}
