const std = @import("std");
const debug = false;

const MapWrapper = struct {
    // https://discord.com/channels/605571803288698900/1457072282718437618
    map: std.StringArrayHashMapUnmanaged([]const u8), // ideally, switch to a StringArrayHashMapUnmanaged for faster iteration!
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

pub fn fetch_readmes(allocator: std.mem.Allocator, output: *std.StringArrayHashMapUnmanaged([]const u8), parsed_json: std.json.Value, thing_to_parse: []const u8) !void {
    var iter = parsed_json.object.get(thing_to_parse).?.object.iterator();
    // I had discussed this with someone on discord
    // and found out that zig does maintain keep alive
    // for a client only for a single domain
    // hence I am creating 2 clients, one for github
    // other for codeberg.

    var github_client = std.http.Client{ .allocator = allocator };
    defer github_client.deinit();
    var codeberg_client = std.http.Client{ .allocator = allocator };
    defer codeberg_client.deinit();

    var response_writer: std.io.Writer.Allocating = .init(allocator);
    defer response_writer.deinit();

    while (iter.next()) |it| {
        const repo_name_id = it.key_ptr.*;
        if (output.contains(repo_name_id)) {
            continue;
        }
        const value = it.value_ptr.*.object;
        var repo_name_id_iter = std.mem.splitScalar(u8, repo_name_id, '/');
        const provider_id = repo_name_id_iter.next().?;
        const owner_name = repo_name_id_iter.next().?;
        const repo_name = repo_name_id_iter.next().?;

        response_writer.clearRetainingCapacity();

        // WOOH! This if else statement was dangerous
        // the database:
        // contains a repo
        // a repo may or may not have dbi i.e default branch information
        // the default branch information may or may not contain r i.e readme url
        // the readme url may or may not be a string
        // if it is a string it may or maynot by empty.
        // if the string is non empty, it can be either "404 unable to find readme." or finally a valid url.
        // A.K.A ( -_-;)
        if (value.contains("dbi") and value.get("dbi").?.object.contains("r") and value.get("dbi").?.object.get("r").? == .string and !std.mem.eql(u8, value.get("dbi").?.object.get("r").?.string, "") and value.get("dbi").?.object.get("r").?.string[0] != '4') {
            const url = value.get("dbi").?.object.get("r").?.string;
            std.debug.print("FETCHING: {s}", .{url});
            var client = if (std.mem.eql(u8, provider_id, "gh")) &github_client else &codeberg_client;

            const responce = try client.fetch(.{
                .location = .{ .url = url },
                .response_writer = &response_writer.writer,
            });
            if (responce.status != .ok) {
                continue; // I am doing this because I can't crash the whole process for 1 readme.
            }
            const readme_content = response_writer.written();
            var buf: [2000]u8 = undefined;
            var slice: []u8 = &buf;
            var count: usize = 0;
            if (!std.unicode.utf8ValidateSlice(readme_content)) {
                const description = if (value.get("description") != null and value.get("description").? == .string)
                    value.get("description").?.string
                else
                    "";
                const result = try std.fmt.allocPrint(allocator, "{s} {s}", .{ repo_name_id, description });
                try output.put(allocator, repo_name_id, result);
                continue;
            }
            var iterr: std.unicode.Utf8Iterator = .{ .bytes = readme_content, .i = 0 };
            while (iterr.nextCodepoint()) |c| {
                if (count == slice.len) {
                    break;
                }
                if (c <= 0x7f) {
                    slice[count] = @as(u8, @intCast(c));
                    count += 1;
                }
            }
            const lower = slice[0..count];
            const description = if (value.get("description") != null and value.get("description").? == .string)
                value.get("description").?.string
            else
                "";
            const result = try std.fmt.allocPrint(allocator, "{s} {s} {s} {s}", .{ lower, owner_name, repo_name, description });
            try output.put(allocator, repo_name_id, result);
            if (debug) {
                std.debug.print("\n\nOUTPUT :: {s}\n\n", .{result});
            }
        } else {
            if (debug) {
                break;
            }
            const description = if (value.get("description") != null and value.get("description").? == .string)
                value.get("description").?.string
            else
                "";
            const result = try std.fmt.allocPrint(allocator, "{s} {s}", .{ repo_name_id, description });
            try output.put(allocator, repo_name_id, result);
        }
    }
}

pub fn main() !u8 {
    var gpa: std.heap.GeneralPurposeAllocator(.{}) = .init;
    defer {
        const deinit_status = gpa.deinit();
        //fail test; can't try in defer as defer is executed after we return
        if (deinit_status == .leak) std.testing.expect(false) catch @panic("TEST FAIL");
    }
    const allocator = gpa.allocator();

    var output: std.StringArrayHashMapUnmanaged([]const u8) = .empty;

    var main_database_fetcher_client: std.http.Client = .{ .allocator = allocator };
    defer main_database_fetcher_client.deinit();

    var response_writer: std.io.Writer.Allocating = .init(allocator);
    defer response_writer.deinit();

    const main_database_fetch = try main_database_fetcher_client.fetch(.{
        .location = .{ .url = "https://github.com/Zigistry/database/releases/download/database/database.json" },
        .response_writer = &response_writer.writer,
        .redirect_behavior = .init(2),
    });

    if (main_database_fetch.status != .ok) {
        std.debug.print("Error: {d}\n", .{main_database_fetch.status});
        return 1;
    }

    const main_database_raw = response_writer.written();
    const main_database_parsed = std.json.parseFromSlice(std.json.Value, allocator, main_database_raw, .{}) catch @panic("Failed to parse json.");
    defer main_database_parsed.deinit();

    fetch_readmes(allocator, &output, main_database_parsed.value, "packages") catch @panic("Failed to fetch readmes.");
    fetch_readmes(allocator, &output, main_database_parsed.value, "programs") catch @panic("Failed to fetch readmes.");
    var stringifier: std.io.Writer.Allocating = .init(allocator);
    defer stringifier.deinit();
    var stringify_obj: std.json.Stringify = .{
        .writer = &stringifier.writer,
        .options = .{ .escape_unicode = true, .emit_nonportable_numbers_as_strings = true },
    };

    var processed_readmes_wrapped: MapWrapper = .{ .map = output };
    try processed_readmes_wrapped.jsonStringify(&stringify_obj);

    const final_stringified_json = stringifier.written();

    var file = try std.fs.cwd().createFile("search_data.json", .{});
    defer file.close();

    defer {
        var it = output.iterator();
        while (it.next()) |entry| {
            allocator.free(entry.value_ptr.*);
        }
        output.deinit(allocator);
    }

    _ = try file.writeAll(final_stringified_json);
    return 0;
}
