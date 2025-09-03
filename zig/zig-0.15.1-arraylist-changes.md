# Zig 0.15.1 ArrayList API Breaking Changes

In Zig 0.15.1, a breaking change was introduced to make the unmanaged variant of `std.ArrayList` the default, as detailed in the release notes. This means the previous `std.ArrayList` (which stored an allocator internally) has been renamed to `std.array_list.Managed`, while `std.ArrayList` now refers to what was previously `std.ArrayListUnmanaged` (which does not store an allocator and requires passing it explicitly to mutating methods like `append` or `deinit`). Note that `std.ArrayList` and `std.ArrayListAligned` are marked for eventual removal in future versions, likely to be fully replaced by explicitly named unmanaged types.

## 1. How to Initialize an ArrayList

The `.init(allocator)` method no longer works directly on `std.ArrayList` because it is now unmanaged by default and does not take an allocator during initialization (since it doesn't store one). Instead:

### For the default unmanaged `std.ArrayList` (recommended for most cases to avoid the overhead of storing an allocator):
```zig
var list: std.ArrayList(i32) = .{};  // Initializes an empty ArrayList
```
- This creates an empty list with no capacity allocated yet.
- To pre-allocate capacity, use `ensureTotalCapacity` after initialization:
  ```zig
  var list: std.ArrayList(i32) = .{};
  try list.ensureTotalCapacity(allocator, 10);  // Pre-allocates space for 10 items
  ```
- When done, free memory explicitly:
  ```zig
  defer list.deinit(allocator);
  ```

### If you want the old managed behavior (which stores the allocator internally, avoiding the need to pass it to every method):
```zig
var list = std.array_list.Managed(i32).init(allocator);
defer list.deinit();  // No allocator needed here
```

## 2. The Correct Constructor Method

There is no single "constructor" like before; it depends on whether you want managed or unmanaged behavior:

### For unmanaged (`std.ArrayList(T)`): 
Use struct literal initialization `std.ArrayList(T){}` (no `.init(allocator)`). All mutating operations (e.g., `append`, `insert`, `ensureUnusedCapacity`) now require passing the allocator explicitly:
```zig
try list.append(allocator, 42);
```

### For managed (`std.array_list.Managed(T)`): 
Use `std.array_list.Managed(T).init(allocator)`. Mutating operations do not require passing the allocator again:
```zig
try list.append(42);
```

The release notes recommend transitioning to the unmanaged default where possible, as it simplifies the API and reduces memory overhead, especially for nested data structures. If your code relied on the old `.init(allocator)` on the ArrayList type, update it to use `std.array_list.Managed` to minimize changes, or refactor to pass the allocator explicitly for the unmanaged version.
