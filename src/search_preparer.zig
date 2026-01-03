const std = @import("std");

pub fn fetch_readmes(allocator: std.mem.Allocator, parsed_json: std.json.Value) !std.StringHashMap([]const u8) {
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
        const repo_name_id_iter = std.mem.splitScalar('/', repo_name_id, '/');
        const provider_id = repo_name_id_iter.next().?;
        const owner_name = repo_name_id_iter.next().?;
        const repo_name = repo_name_id_iter.next().?;

        var responce_body: std.array_list.Managed(u8) = .init(allocator);

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
                    .response_storage = .{ .dynamic = &responce_body },
                });
                if (responce.status != .ok) {
                    continue; // I am doing this because I can't crash the whole process for 1 readme.
                }
                var readme_content = try responce_body.toOwnedSlice();
                const lower = std.ascii.lowerString(&readme_content, &readme_content[0..2000]);
                const result = try std.fmt.allocPrint(allocator, "{s} {s} {s}", .{ lower, owner_name, repo_name });
                defer allocator.free(result);
                output.put(repo_name_id, result);
            } else if (std.mem.eql(u8, provider_id, "cb")) {
                const responce = try codeberg_client.fetch(.{
                    .location = .{ .url = url },
                    .response_storage = .{ .dynamic = &responce_body },
                });
                if (responce.status != .ok) {
                    continue; // I am doing this because I can't crash the whole process for 1 readme.
                }
                const readme_content = try responce_body.toOwnedSlice();
                const lower = std.ascii.lowerString(&readme_content, &readme_content[0..2000]);
                const result = try std.fmt.allocPrint(allocator, "{s} {s} {s}", .{ lower, owner_name, repo_name });
                defer allocator.free(result);
                output.put(repo_name_id, result);
            } else {
                output.put(repo_name_id, repo_name_id);
            }
        } else {
            output.put(repo_name_id, repo_name_id);
        }
    }
    return output;
}
pub fn main() u8 {
    var arena = std.heap.ArenaAllocator.init(std.heap.c_allocator);
    defer arena.deinit();
    const allocator = arena.allocator();
    var client = std.http.Client{ .allocator = allocator };
    defer client.deinit();
    var responce_body: std.array_list.Managed(u8) = .init(allocator);
    defer responce_body.deinit();
    const responce = try client.fetch(.{
        .location = .{ .url = "https://" },
        .response_storage = .{ .dynamic = &responce_body },
    });
    if (responce.status == .ok) {
        const raw_json = responce_body.toOwnedSlice() catch return 1;
        defer allocator.free(raw_json);
        const parsed = std.json.parseFromSlice(std.json.Value, allocator, raw_json, .{});
        defer parsed.deinit();
        const new_data = fetch_readmes(parsed) catch @panic("Failed to fetch readmes.");
        const iter = new_data.iterator();
        const stdout = std.fs.File.stdout().deprecatedWriter();

        stdout.print("{{", .{});
        while (iter.next()) |thing| {
            th
        }
    } else {
        std.debug.print("Error: {d}\n", .{responce_body.status});
        return 1;
    }
    return 0;
}
