const std = @import("std");

// ==================== IR结构体定义 ====================

pub const ChimIRArg = extern struct {
    name: [*c]u8,
    type: [*c]u8,
};

pub const ChimIRFunction = extern struct {
    name: [*c]u8,
    args: [*c]ChimIRArg,
    arg_count: u32,
    return_type: [*c]u8,
};

pub const ChimIRModule = extern struct {
    funcs: [*c]ChimIRFunction,
    func_count: u32,
};

// ==================== Channel支持 ====================

pub const Channel = struct {
    value: i64,
    ready: bool,
    closed: bool,
    
    pub fn init() Channel {
        return Channel{
            .value = 0,
            .ready = false,
            .closed = false,
        };
    }
};

pub export fn chim_channel_create() ?*anyopaque {
    const allocator = std.heap.page_allocator;
    const ch = allocator.create(Channel) catch return null;
    ch.* = Channel.init();
    return @ptrCast(ch);
}

pub export fn chim_channel_close(ch: ?*anyopaque) void {
    if (ch) |ptr| {
        const channel = @as(*Channel, @ptrCast(@alignCast(ptr)));
        channel.closed = true;
    }
}

pub export fn chim_channel_destroy(ch: ?*anyopaque) void {
    if (ch) |ptr| {
        const channel = @as(*Channel, @ptrCast(@alignCast(ptr)));
        std.heap.page_allocator.destroy(channel);
    }
}

pub export fn chim_send(ch: ?*anyopaque, value: i64) void {
    if (ch) |ptr| {
        const channel = @as(*Channel, @ptrCast(@alignCast(ptr)));
        if (!channel.closed) {
            channel.value = value;
            channel.ready = true;
        }
    }
}

pub export fn chim_recv(ch: ?*anyopaque) i64 {
    if (ch) |ptr| {
        const channel = @as(*Channel, @ptrCast(@alignCast(ptr)));
        if (channel.ready and !channel.closed) {
            channel.ready = false;
            return channel.value;
        }
    }
    return 0;
}

pub export fn chim_channel_is_ready(ch: ?*anyopaque) bool {
    if (ch) |ptr| {
        const channel = @as(*Channel, @ptrCast(@alignCast(ptr)));
        return channel.ready;
    }
    return false;
}

// ==================== Arena分配器 ====================

pub const Arena = struct {
    allocator: std.heap.ArenaAllocator,
    
    pub fn init() Arena {
        return Arena{
            .allocator = std.heap.ArenaAllocator.init(std.heap.page_allocator),
        };
    }
    
    pub fn deinit(self: *Arena) void {
        self.allocator.deinit();
    }
};

pub export fn chim_arena_create() ?*anyopaque {
    const allocator = std.heap.page_allocator;
    const arena = allocator.create(Arena) catch return null;
    arena.* = Arena.init();
    return @ptrCast(arena);
}

pub export fn chim_arena_destroy(arena: ?*anyopaque) void {
    if (arena) |ptr| {
        const a = @as(*Arena, @ptrCast(@alignCast(ptr)));
        a.deinit();
        std.heap.page_allocator.destroy(a);
    }
}

pub export fn chim_arena_alloc(arena: ?*anyopaque, size: usize) ?*anyopaque {
    if (arena) |ptr| {
        const a = @as(*Arena, @ptrCast(@alignCast(ptr)));
        const mem = a.allocator.allocator().alloc(u8, size) catch return null;
        return @ptrCast(mem.ptr);
    }
    return null;
}

pub export fn chim_arena_reset(arena: ?*anyopaque) void {
    if (arena) |ptr| {
        const a = @as(*Arena, @ptrCast(@alignCast(ptr)));
        _ = a.allocator.reset(.retain_capacity);
    }
}

// ==================== 内存操作 ====================

pub export fn chim_alloc(size: usize) ?*anyopaque {
    const allocator = std.heap.page_allocator;
    const mem = allocator.alloc(u8, size) catch return null;
    return @ptrCast(mem.ptr);
}

pub export fn chim_free(ptr: ?*anyopaque, size: usize) void {
    if (ptr) |p| {
        const allocator = std.heap.page_allocator;
        const slice = @as([*]u8, @ptrCast(@alignCast(p)))[0..size];
        allocator.free(slice);
    }
}

pub export fn chim_realloc(ptr: ?*anyopaque, old_size: usize, new_size: usize) ?*anyopaque {
    const allocator = std.heap.page_allocator;
    if (ptr) |p| {
        const old_slice = @as([*]u8, @ptrCast(@alignCast(p)))[0..old_size];
        const new_mem = allocator.realloc(old_slice, new_size) catch return null;
        return @ptrCast(new_mem.ptr);
    }
    return chim_alloc(new_size);
}

pub export fn chim_memcpy(dest: ?*anyopaque, src: ?*anyopaque, size: usize) void {
    if (dest != null and src != null) {
        const d = @as([*]u8, @ptrCast(@alignCast(dest)));
        const s = @as([*]u8, @ptrCast(@alignCast(src)));
        @memcpy(d[0..size], s[0..size]);
    }
}

pub export fn chim_memset(ptr: ?*anyopaque, value: u8, size: usize) void {
    if (ptr) |p| {
        const mem = @as([*]u8, @ptrCast(@alignCast(p)));
        @memset(mem[0..size], value);
    }
}

// ==================== 数组操作 ====================

pub const Array = struct {
    data: []i64,
    length: usize,
    capacity: usize,
    allocator: std.mem.Allocator,
    
    pub fn init(allocator: std.mem.Allocator, capacity: usize) !Array {
        const data = try allocator.alloc(i64, capacity);
        return Array{
            .data = data,
            .length = 0,
            .capacity = capacity,
            .allocator = allocator,
        };
    }
    
    pub fn deinit(self: *Array) void {
        self.allocator.free(self.data);
    }
};

pub export fn chim_array_create(size: usize) ?*anyopaque {
    const allocator = std.heap.page_allocator;
    const array = allocator.create(Array) catch return null;
    array.* = Array.init(allocator, size) catch {
        allocator.destroy(array);
        return null;
    };
    array.length = size;
    // 初始化为0
    @memset(array.data, 0);
    return @ptrCast(array);
}

pub export fn chim_array_destroy(arr: ?*anyopaque) void {
    if (arr) |ptr| {
        const array = @as(*Array, @ptrCast(@alignCast(ptr)));
        array.deinit();
        std.heap.page_allocator.destroy(array);
    }
}

pub export fn chim_array_get(arr: ?*anyopaque, index: usize) i64 {
    if (arr) |ptr| {
        const array = @as(*Array, @ptrCast(@alignCast(ptr)));
        if (index < array.length) {
            return array.data[index];
        }
    }
    return 0;
}

pub export fn chim_array_set(arr: ?*anyopaque, index: usize, value: i64) void {
    if (arr) |ptr| {
        const array = @as(*Array, @ptrCast(@alignCast(ptr)));
        if (index < array.length) {
            array.data[index] = value;
        }
    }
}

pub export fn chim_array_length(arr: ?*anyopaque) usize {
    if (arr) |ptr| {
        const array = @as(*Array, @ptrCast(@alignCast(ptr)));
        return array.length;
    }
    return 0;
}

pub export fn chim_array_push(arr: ?*anyopaque, value: i64) bool {
    if (arr) |ptr| {
        const array = @as(*Array, @ptrCast(@alignCast(ptr)));
        if (array.length < array.capacity) {
            array.data[array.length] = value;
            array.length += 1;
            return true;
        }
    }
    return false;
}

pub export fn chim_array_pop(arr: ?*anyopaque) i64 {
    if (arr) |ptr| {
        const array = @as(*Array, @ptrCast(@alignCast(ptr)));
        if (array.length > 0) {
            array.length -= 1;
            return array.data[array.length];
        }
    }
    return 0;
}

// ==================== 字符串操作 ====================

pub export fn chim_string_concat(s1: [*c]const u8, s2: [*c]const u8) [*c]u8 {
    const str1 = std.mem.span(s1);
    const str2 = std.mem.span(s2);
    
    const allocator = std.heap.page_allocator;
    const result = allocator.alloc(u8, str1.len + str2.len + 1) catch return null;
    
    @memcpy(result[0..str1.len], str1);
    @memcpy(result[str1.len..str1.len + str2.len], str2);
    result[str1.len + str2.len] = 0;
    
    return @ptrCast(result.ptr);
}

pub export fn chim_string_length(s: [*c]const u8) usize {
    return std.mem.len(s);
}

pub export fn chim_string_equals(s1: [*c]const u8, s2: [*c]const u8) bool {
    const str1 = std.mem.span(s1);
    const str2 = std.mem.span(s2);
    return std.mem.eql(u8, str1, str2);
}

pub export fn chim_string_substring(s: [*c]const u8, start: usize, end: usize) [*c]u8 {
    const str = std.mem.span(s);
    if (start >= str.len or end > str.len or start >= end) {
        return null;
    }
    
    const allocator = std.heap.page_allocator;
    const len = end - start;
    const result = allocator.alloc(u8, len + 1) catch return null;
    
    @memcpy(result[0..len], str[start..end]);
    result[len] = 0;
    
    return @ptrCast(result.ptr);
}

pub export fn chim_string_free(s: [*c]u8) void {
    if (s != null) {
        const str = std.mem.span(s);
        const allocator = std.heap.page_allocator;
        allocator.free(str);
    }
}

// 打印函数（用于生成的代码）
pub export fn chim_print_i64(value: i64) void {
    std.debug.print("{}", .{value});
}

pub export fn chim_print_f64(value: f64) void {
    std.debug.print("{d}", .{value});
}

pub export fn chim_print_str(str: [*c]const u8) void {
    std.debug.print("{s}", .{str});
}

pub export fn chim_print_newline() void {
    std.debug.print("\n", .{});
}

// ==================== 数学函数 ====================

pub export fn chim_abs_i64(value: i64) i64 {
    return if (value < 0) -value else value;
}

pub export fn chim_abs_f64(value: f64) f64 {
    return @abs(value);
}

pub export fn chim_pow_f64(base: f64, exp: f64) f64 {
    return std.math.pow(f64, base, exp);
}

pub export fn chim_sqrt_f64(value: f64) f64 {
    return @sqrt(value);
}

pub export fn chim_sin_f64(value: f64) f64 {
    return @sin(value);
}

pub export fn chim_cos_f64(value: f64) f64 {
    return @cos(value);
}

pub export fn chim_tan_f64(value: f64) f64 {
    return @tan(value);
}

pub export fn chim_floor_f64(value: f64) f64 {
    return @floor(value);
}

pub export fn chim_ceil_f64(value: f64) f64 {
    return @ceil(value);
}

pub export fn chim_round_f64(value: f64) f64 {
    return @round(value);
}

pub export fn chim_min_i64(a: i64, b: i64) i64 {
    return if (a < b) a else b;
}

pub export fn chim_max_i64(a: i64, b: i64) i64 {
    return if (a > b) a else b;
}

pub export fn chim_min_f64(a: f64, b: f64) f64 {
    return @min(a, b);
}

pub export fn chim_max_f64(a: f64, b: f64) f64 {
    return @max(a, b);
}

// ==================== 后端代码生成 ====================

/// 后端代码生成入口
pub export fn chim_backend_codegen(mod: *const ChimIRModule) u32 {
    if (mod.func_count == 0) return 0;
    
    std.debug.print("[Zig Backend] Generating code for {} functions\n", .{mod.func_count});
    
    var i: usize = 0;
    while (i < mod.func_count) : (i += 1) {
        const func = mod.funcs[i];
        const name = std.mem.span(func.name);
        std.debug.print("  Function: {s}\n", .{name});
    }
    
    return 1;
}
