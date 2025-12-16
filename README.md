# valq &emsp; [![Docs.rs shield]][Docs.rs link] [![crates.io shield]][crates.io link]

[Docs.rs shield]: https://img.shields.io/docsrs/valq/latest
[Docs.rs link]: https://docs.rs/valq/latest
[crates.io shield]: https://img.shields.io/crates/v/valq
[crates.io link]: https://crates.io/crates/valq

`valq` provides a macro for querying semi-structured ("JSON-ish") data **with the JavaScript-like syntax**.

Look & Feel:

```rust
let obj: serde_json::Value = ...;

// without valq: tedious and_then() chain...
let deep = obj
    .get("path")
    .and_then(|v| v.get("to"))
    .and_then(|v| v.get("value"))
    .and_then(|v| v.get("at"))
    .and_then(|v| v.get("deep"));

// with valq: very concise and readable!
use valq::query_value;
let deep = query_value!(obj.path.to.value.at.deep);
```

## Installation

Add this to the `Cargo.toml` in your project:

```toml
[dependencies]
valq = "*"
```

The principal macro provided by this crate is `query_value!`.
Also, there is a `Result`-returning variant of `query_value!`, called `query_value_result!`.

## `query_value!` macro
A macro for querying, extracting and converting inner value of semi-structured data.

### Basic Queries
```rust
// get field `foo` from JSON object `obj`
let foo = query_value!(obj.foo);

// get the first item of the nested JSON array `arr` in `obj`
let head = query_value!(obj.arr[0]);

// more complex query, just works!
let abyss = query_value!(obj.path.to.matrix[0][1].abyss);
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
// `->` syntax converts `Value` to typed value (see below)
assert_eq!(query_value!(obj.foo.bar.x -> u64), Some(100));
assert_eq!(query_value!(obj.foo.bar.y -> u64), Some(200));
```

### Converting & Deserializing to Specified Type
```rust
// try to convert the queried value into `u64` using `as_u64()` method on that value.
// results in `None` in case of type mismatch
let foo_u64: Option<u64> = query_value!(obj.foo -> u64);

// in the context of mutable reference extraction (see below), `as_xxx_mut()` method is used instead.
let arr_vec: Option<&mut Vec<Value>> = query_value!(mut obj.arr -> array);
```

```rust
use serde::Deserialize;
use serde_json::json;
use valq::query_value;

#[derive(Deserialize)]
struct Person {
    name: String,
    age: u8,
}

let j = json!({"author": {"name": "jiftechnify", "age": 31}});

// try to deserialize the queried value into a value of type `Person`.
let author: Option<Person> = query_value!(j.author >> (Person));
```

### Unwrapping Query Results with Default Values
```rust
use serde_json::json;
use valq::query_value;

let obj = json!({"foo": {"bar": "not a number"}});
assert_eq!(query_value!(obj.foo.bar -> str ?? "failed!"), "not a number");
assert_eq!(query_value!(obj.foo.bar -> u64 ?? 42), 42); // explicitly provided default
assert_eq!(query_value!(obj.foo.bar -> u64 ?? default), 0u64); // using u64::default()
```

## `query_value_result!` macro
A variant of `query_value!` that returns `Result<T, valq::Error>` instead of `Option<T>`.

```rust
use serde::Deserialize;
use serde_json::json;
use valq::{query_value_result, Error};

let obj = json!({"foo": {"bar": 42}});

// Error::ValueNotFoundAtPath: querying non-existent path
let result = query_value_result!(obj.foo.baz);
assert!(matches!(result, Err(Error::ValueNotFoundAtPath(_))));

// Error::AsCastFailed: type conversion failure
let result = query_value_result!(obj.foo.bar -> str);
assert!(matches!(result, Err(Error::AsCastFailed(_))));

// Error::DeserializationFailed: deserialization failure
let result = query_value_result!(obj.foo >> (Vec<u8>));
assert!(matches!(result, Err(Error::DeserializationFailed(_))));
```

## Compatibility
The `query_value!` macro can be used with arbitrary data structure(to call, `Value`) that supports `get(&self, idx) -> Option<&Value>` method that retrieves a value at `idx`. 

Extracting mutable reference is also supported if your `Value` supports `get_mut(&mut self, idx) -> Option<&Value>`.

Instances of compatible data structures:
- [`serde_json::Value`](https://docs.rs/serde_json/latest/serde_json/enum.Value.html)
- [`serde_yaml::Value`](https://docs.rs/serde_yaml/latest/serde_yaml/enum.Value.html)
- [`toml::Value`](https://docs.rs/toml/latest/toml/value/enum.Value.html)
- and more...
