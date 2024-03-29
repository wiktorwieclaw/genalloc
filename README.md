# Generational Allocations
Heavily inspired by [generational-box](https://crates.io/crates/generational-box).

## Examples
```rust
let mut span = genalloc::Span::new();
let ptr: genalloc::Ptr<u32> = span.alloc(5); // No lifetimes! `Ptr` is `Copy`!
assert_eq!(*ptr.read(), 5);
```

`Span` is the owner of the memory so the `Ptr` is valid as long as it's `Span` is alive.
```rust
let ptr = {
    let mut span = genalloc::Span::new();
    span.alloc(5)
};
*ptr.read(); // Panics!
```