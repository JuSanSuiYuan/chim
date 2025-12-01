const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});
    
    // 构建动态库
    const lib = b.addSharedLibrary(.{
        .name = "chim_backend",
        .root_source_file = b.path("src/lib.zig"),
        .target = target,
        .optimize = optimize,
    });
    
    lib.linkLibC();
    b.installArtifact(lib);
}
