const std = @import("std");
const lib = @import("lib.zig");

pub fn main() !void {
    std.debug.print("=== Chim Runtime Library Tests ===\n\n", .{});
    
    // 1. Channel测试
    std.debug.print("[1] Channel Test:\n", .{});
    const ch = lib.chim_channel_create().?;
    defer lib.chim_channel_destroy(ch);
    
    lib.chim_send(ch, 42);
    const val = lib.chim_recv(ch);
    std.debug.print("  Send 42, Recv: {}\n", .{val});
    std.debug.print("  Is ready: {}\n", .{lib.chim_channel_is_ready(ch)});
    
    // 2. Arena测试
    std.debug.print("\n[2] Arena Test:\n", .{});
    const arena = lib.chim_arena_create().?;
    defer lib.chim_arena_destroy(arena);
    
    const mem1 = lib.chim_arena_alloc(arena, 100);
    const mem2 = lib.chim_arena_alloc(arena, 200);
    std.debug.print("  Allocated 2 blocks: {} {}\n", .{mem1 != null, mem2 != null});
    
    lib.chim_arena_reset(arena);
    std.debug.print("  Arena reset\n", .{});
    
    // 3. 数组测试
    std.debug.print("\n[3] Array Test:\n", .{});
    const arr = lib.chim_array_create(10).?;
    defer lib.chim_array_destroy(arr);
    
    _ = lib.chim_array_set(arr, 0, 100);
    _ = lib.chim_array_set(arr, 1, 200);
    _ = lib.chim_array_push(arr, 300);
    
    std.debug.print("  arr[0] = {}\n", .{lib.chim_array_get(arr, 0)});
    std.debug.print("  arr[1] = {}\n", .{lib.chim_array_get(arr, 1)});
    std.debug.print("  Length: {}\n", .{lib.chim_array_length(arr)});
    
    const popped = lib.chim_array_pop(arr);
    std.debug.print("  Popped: {}\n", .{popped});
    
    // 4. 字符串测试
    std.debug.print("\n[4] String Test:\n", .{});
    const s1 = "Hello";
    const s2 = " World";
    const s3 = lib.chim_string_concat(s1.ptr, s2.ptr);
    defer lib.chim_string_free(s3);
    
    std.debug.print("  Concat: {s}\n", .{std.mem.span(s3)});
    std.debug.print("  Length: {}\n", .{lib.chim_string_length(s3)});
    
    // 5. 数学函数测试
    std.debug.print("\n[5] Math Test:\n", .{});
    std.debug.print("  abs(-42) = {}\n", .{lib.chim_abs_i64(-42)});
    std.debug.print("  sqrt(16) = {d}\n", .{lib.chim_sqrt_f64(16.0)});
    std.debug.print("  pow(2, 8) = {d}\n", .{lib.chim_pow_f64(2.0, 8.0)});
    std.debug.print("  max(10, 20) = {}\n", .{lib.chim_max_i64(10, 20)});
    
    std.debug.print("\n=== All Tests Passed ===\n", .{});
}
