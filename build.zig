const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{
        .preferred_optimize_mode = std.builtin.OptimizeMode.ReleaseFast,
    });
    // Helper Functions Library
    const helperFunctions = b.addModule("helperFunctions", .{
        .root_source_file = b.path("scripts/libs/functionsProvider.zig"),
        .target = target,
        .optimize = optimize,
    });
    // Database Compiler
    {
        const databaseCompiler = b.addExecutable(.{
            .name = "databaseCompiler",
            .root_source_file = b.path("scripts/databaseCompiler.zig"),
            .target = target,
            .optimize = optimize,
        });
        databaseCompiler.linkSystemLibrary("c");
        databaseCompiler.root_module.addImport("helperFunctions", helperFunctions);
        b.installArtifact(databaseCompiler);

        const databaseCompilerRunCmd = b.addRunArtifact(databaseCompiler);
        databaseCompilerRunCmd.step.dependOn(b.getInstallStep());
        if (b.args) |args| {
            databaseCompilerRunCmd.addArgs(args);
        }
        const databaseCompilerRunStep = b.step("run_databaseCompiler", "Run database compiler");
        databaseCompilerRunStep.dependOn(&databaseCompilerRunCmd.step);
    }
    {
        const codeberg = b.addExecutable(.{
            .name = "codeberg",
            .root_source_file = b.path("scripts/codeberg.zig"),
            .target = target,
            .optimize = optimize,
        });
        codeberg.linkSystemLibrary("c");
        codeberg.root_module.addImport("helperFunctions", helperFunctions);
        b.installArtifact(codeberg);

        const codebergRunCmd = b.addRunArtifact(codeberg);
        codebergRunCmd.step.dependOn(b.getInstallStep());
        if (b.args) |args| {
            codebergRunCmd.addArgs(args);
        }
        const codebergRunStep = b.step("run_codeberg", "Run codeberg");
        codebergRunStep.dependOn(&codebergRunCmd.step);
    }
    {
        const gitlab = b.addExecutable(.{
            .name = "gitlab",
            .root_source_file = b.path("scripts/gitlab.zig"),
            .target = target,
            .optimize = optimize,
        });
        gitlab.linkSystemLibrary("c");
        gitlab.root_module.addImport("helperFunctions", helperFunctions);
        b.installArtifact(gitlab);

        const gitlabRunCmd = b.addRunArtifact(gitlab);
        gitlabRunCmd.step.dependOn(b.getInstallStep());
        if (b.args) |args| {
            gitlabRunCmd.addArgs(args);
        }
        const gitlabRunStep = b.step("run_gitlab", "Run gitlab");
        gitlabRunStep.dependOn(&gitlabRunCmd.step);
    }
    {
        const databaseCompiler2 = b.addExecutable(.{
            .name = "databaseCompiler2",
            .root_source_file = b.path("scripts/databaseCompiler2.zig"),
            .target = target,
            .optimize = optimize,
        });
        databaseCompiler2.linkSystemLibrary("c");
        databaseCompiler2.root_module.addImport("helperFunctions", helperFunctions);
        b.installArtifact(databaseCompiler2);

        const databaseCompilerRunCmd2 = b.addRunArtifact(databaseCompiler2);
        databaseCompilerRunCmd2.step.dependOn(b.getInstallStep());
        if (b.args) |args| {
            databaseCompilerRunCmd2.addArgs(args);
        }
        const databaseCompilerRunStep2 = b.step("run_databaseCompiler2", "Run database compiler");
        databaseCompilerRunStep2.dependOn(&databaseCompilerRunCmd2.step);
    }
    // Repo List Compressor
    {
        const repoListCompressor = b.addExecutable(.{
            .name = "repoListCompressor",
            .root_source_file = b.path("scripts/repoListCompressor.zig"),
            .target = target,
            .optimize = optimize,
        });
        repoListCompressor.linkSystemLibrary("c");
        repoListCompressor.root_module.addImport("helperFunctions", helperFunctions);
        b.installArtifact(repoListCompressor);

        const repoListCompressorRunCmd = b.addRunArtifact(repoListCompressor);
        repoListCompressorRunCmd.step.dependOn(b.getInstallStep());
        if (b.args) |args| {
            repoListCompressorRunCmd.addArgs(args);
        }
        const repoListCompressorRunStep = b.step("run_repoListCompressor", "Run repoListCompressor");
        repoListCompressorRunStep.dependOn(&repoListCompressorRunCmd.step);
    }
    // Create Deep Search Database
    {
        const createDeepSearchDB = b.addExecutable(.{
            .name = "createDeepSearchDB",
            .root_source_file = b.path("scripts/createDeepSearchDB.zig"),
            .target = target,
            .optimize = optimize,
        });
        createDeepSearchDB.linkSystemLibrary("c");
        createDeepSearchDB.root_module.addImport("helperFunctions", helperFunctions);
        b.installArtifact(createDeepSearchDB);

        const createDeepSearchDBRunCmd = b.addRunArtifact(createDeepSearchDB);
        createDeepSearchDBRunCmd.step.dependOn(b.getInstallStep());
        if (b.args) |args| {
            createDeepSearchDBRunCmd.addArgs(args);
        }
        const repoListCompressorRunStep = b.step("run_createDeepSearchDB", "Run createDeepSearchDB");
        repoListCompressorRunStep.dependOn(&createDeepSearchDBRunCmd.step);
    }
    // Helper Functions Unit Tests
    {
        const functionsProviderUnitTests = b.addTest(.{
            .root_source_file = b.path("scripts/libs/functionsProvider.zig"),
            .target = target,
            .optimize = optimize,
        });
        functionsProviderUnitTests.linkSystemLibrary("c");
        const runFunctionsProviderUnitTests = b.addRunArtifact(functionsProviderUnitTests);
        const functionsProviderTestStep = b.step("run_testlib", "Run helperFunctions lib tests");
        functionsProviderTestStep.dependOn(&runFunctionsProviderUnitTests.step);
    }
}
