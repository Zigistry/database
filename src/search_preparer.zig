const std = @import("std");

pub fn main() u8 {
    var arena = std.heap.ArenaAllocator.init(std.heap.c_allocator);
    defer arena.deinit();
    const allocator = arena.allocator();
    var client = std.http.Client{ .allocator = allocator };
    defer client.deinit();
    var responce_body = std.ArrayList(u8).init(allocator);
    defer responce_body.deinit();
    const responce = try client.fetch(.{
        .location = .{ .url = "https://" },
        .response_storage = .{ .dynamic = &responce_body },
    });
    if (responce.status == .ok) {
        const raw_json = responce_body.toOwnedSlice();
        defer allocator.free(raw_json);
        std.json.parseFromSlice(std.json.Value, allocator, , .{});
    } else {
        std.debug.print("Error: {d}\n", .{responce_body.status});
    }
    return 0;
}
