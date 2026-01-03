const std = @import("std");
const allocator = std.heap.c_allocator;

pub fn fetch_readmes(parsed_json: std.json.Value) !std.StringHashMap([]const u8) {
    var iter = parsed_json.object.get("programs").?.object.iterator();
    // I had discussed this with someone on discord
    // and found out that zig does maintain keep alive
    // for a client only for a single domain
    // hence I am creating 2 clients, one for github
    // other for codeberg.
    var github_client = std.http.Client{ .allocator = allocator };
    defer github_client.deinit();
    var codeberg_client = std.http.Client{ .allocator = allocator };
    defer codeberg_client.deinit();
    var output = std.StringHashMap([]const u8).init(allocator);
    while (iter.next()) |it| {
        const repo_name_id = it.key_ptr.*;
        const value = it.value_ptr.*.object;
        var repo_name_id_iter = std.mem.splitScalar(u8, repo_name_id, '/');
        const provider_id = repo_name_id_iter.next().?;
        const owner_name = repo_name_id_iter.next().?;
        const repo_name = repo_name_id_iter.next().?;

        var response_writer = std.io.Writer.Allocating.init(allocator);
        defer response_writer.deinit();

        // WOOH! This if else statement was dangerous
        // the database:
        // contains a repo
        // a repo may or may not have dbi i.e default branch information
        // the default branch information may or may not contain r i.e readme url
        // the readme url may or may not be a string
        // if it is a string it may or maynot by empty.
        if (value.contains("dbi") and value.get("dbi").?.object.contains("r") and value.get("dbi").?.object.get("r").? == .string and !std.mem.eql(u8, value.get("dbi").?.object.get("r").?.string, "")) {
            const url = value.get("dbi").?.object.get("r").?.string;
            if (std.mem.eql(u8, provider_id, "gh")) {
                const responce = try github_client.fetch(.{
                    .location = .{ .url = url },
                    .response_writer = &response_writer.writer,
                });
                if (responce.status != .ok) {
                    continue; // I am doing this because I can't crash the whole process for 1 readme.
                }
                var readme_content = response_writer.writer.buffered();
                const lower = std.ascii.lowerString(readme_content, readme_content[0..@min(readme_content.len, 2000)]);
                const result = try std.fmt.allocPrint(allocator, "{s} {s} {s}", .{ lower, owner_name, repo_name });
                try output.put(repo_name_id, result);
            } else if (std.mem.eql(u8, provider_id, "cb")) {
                const responce = try codeberg_client.fetch(.{
                    .location = .{ .url = url },
                    .response_writer = &response_writer.writer,
                });
                if (responce.status != .ok) {
                    continue; // I am doing this because I can't crash the whole process for 1 readme.
                }
                var readme_content = response_writer.writer.buffered();
                const lower = std.ascii.lowerString(readme_content, readme_content[0..2000]);
                const result = try std.fmt.allocPrint(allocator, "{s} {s} {s}", .{ lower, owner_name, repo_name });
                try output.put(repo_name_id, result);
            } else {
                try output.put(repo_name_id, repo_name_id);
            }
        } else {
            try output.put(repo_name_id, repo_name_id);
        }
    }
    return output;
}
pub fn main() !u8 {
    var client = std.http.Client{ .allocator = allocator };
    defer client.deinit();
    var response_writer = std.io.Writer.Allocating.init(allocator);
    defer response_writer.deinit();

    const responce = try client.fetch(.{
        .location = .{ .url = "https://" },
        .response_writer = &response_writer.writer,
    });
    if (responce.status != .ok) {
        std.debug.print("Error: {d}\n", .{responce.status});
        return 1;
    }

    const raw_json = response_writer.writer.buffered();
    defer allocator.free(raw_json);

    const parsed = std.json.parseFromSlice(std.json.Value, allocator, raw_json, .{}) catch @panic("Failed to fetch readmes.");
    defer parsed.deinit();

    const new_data = fetch_readmes(parsed.value) catch @panic("Failed to fetch readmes.");
    const MapWrapper = struct {
        map: std.StringHashMap([]const u8), // ideally, switch to a StringArrayHashMapUnmanaged for faster iteration!
        pub fn jsonStringify(wrapper: @This(), s: *std.json.Stringify) !void {
            try s.beginObject();
            var it = wrapper.map.iterator();
            while (it.next()) |entry| {
                try s.objectField(entry.key_ptr.*);
                try s.write(entry.value_ptr.*);
            }
            try s.endObject();
        }
    };
    var writer2 = std.io.Writer.Allocating.init(allocator);
    defer writer2.deinit();
    var thing = MapWrapper{ .map = new_data };
    var stringify_obj = std.json.Stringify{
        .writer = &writer2.writer,
    };
    try thing.jsonStringify(&stringify_obj);
    const mybuf = writer2.writer.buffer;
    try std.fs.cwd().writeFile(.{
        .data = mybuf,
        .sub_path = "search_data.json",
        .flags = .{ .truncate = true },
    });
    return 0;
}
