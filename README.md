# valq &emsp; [![Docs.rs shield]][Docs.rs link] [![crates.io shield]][crates.io link]

[Docs.rs shield]: https://img.shields.io/docsrs/valq/latest
[Docs.rs link]: https://docs.rs/valq/latest
[crates.io shield]: https://img.shields.io/crates/v/valq
[crates.io link]: https://crates.io/crates/valq

`valq` provides a macro for querying and extracting value from structured data **in very concise manner, like the JavaScript syntax**.

Look & Feel:

```rust
use serde_json::Value;
use valq::query_value;

let j: Value = ...;
let deep_val: Option<&Value> = query_value!(j.path.to.value.at.deep);
```

For now, there is only single macro exported: `query_value`.

## `query_value` macro
A macro for querying inner value of structured data.
### Basic Usage
```rust
// get field `foo` from JSON object `obj`
let foo = query_value!(obj.foo);

// get nested field `bar` inside object `foo` in JSON object `obj`
let bar = query_value!(obj.foo.bar);

// get head of JSON array 'arr'
let head = query_value!(arr[0]);

// get head of nested JSON array `arr` in JSON object `obj`
let head = query_value!(obj.arr[0]);

// more complex example!
let abyss = query_value!(obj.path.to.matrix[0][1].abyss);
```

### Converting to Specified Type
```rust
// try to convert extracted value to `u64` by `as_u64()` method  on that value.
// results in `None` in case of type mismatch
let foo_u64: Option<u64> = query_value!(obj.foo -> u64)

// in case of mutable reference extraction (see below), `as_xxx_mut()` method will be used.
let arr_vec: Option<&mut Vec<Value>> = query_value!(mut obj.arr -> array)
```

### Extracting Mutable Reference to Inner Value
```rust
use serde_json::{json, Value}

let mut obj = json!({"foo": { "bar": { "x": 1, "y": 2 }}});
{
    // prefixed `mut` means extracting mutable reference
    let bar: &mut Value = query_value!(mut obj.foo.bar).unwrap();
    *bar = json!({"x": 100, "y": 200});
}
assert_eq!(query_value!(obj.foo.bar.x -> u64), Some(100));
assert_eq!(query_value!(obj.foo.bar.y -> u64), Some(200));
```
