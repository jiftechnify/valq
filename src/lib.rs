//! # valq
//! `valq` provides a macro for querying and extracting an inner value from a structured data **with the JavaScript-ish syntax**.
//!
//! look & feel:
//!
//! ```
//! # use serde_json::json;
//! use serde_json::Value;
//! use valq::query_value;
//!
//! // let obj: Value = ...;
//! # let obj = json!({});
//! let deep_val: Option<&Value> = query_value!(obj.path.to.value.at.deep);
//! ```
//!
//! For now, there is only single macro exported: `query_value`. Refer to [the `query_value` doc] for detailed usage.
//!
//! [the `query_value` doc]: crate::query_value

#[doc(hidden)]
pub use paste::paste;

/// A macro for querying an inner value of a structured ("JSON-ish") data.
///
/// # Usage
/// ## Basic Queries
///
/// With basic queries, `query_value!` extracts a shared reference (`&`) to the inner value by default. Think of it as a function that has following signature:
///
/// ```txt
/// query_value!(query...) -> Option(&Value)
/// ```
///
/// Example:
///
/// ```
/// use valq::query_value;
/// # use serde_json::{json, Value};
/// #
/// # let obj = json!({"foo":{"bar":"bar!"},"arr":[1,2,3],"path":{"to":{"matrix":[[{},{"abyss":"I gaze into you."}],[{},{}]]}}});
/// # let arr = json!([1,2,3]);
///
/// // let obj = json!({ ... });
/// // let arr = json!([ ... ]);
///
/// // get the field `foo` from the JSON-ish object `obj`
/// let foo: Option<&Value> = query_value!(obj.foo);
///
/// // get the nested field `bar` inside `foo` inside `obj`
/// let bar = query_value!(obj.foo.bar);
///
/// // get the first item of the JSON array 'arr'
/// let head = query_value!(arr[0]);
///
/// // get the first item of the nested JSON array `arr` in `obj`
/// let head = query_value!(obj.arr[0]);
///
/// // more complex example!
/// let abyss = query_value!(obj.path.to.matrix[0][1].abyss);
/// ```
///
/// ## `mut`: Extracting Mutable Reference to Inner Value
///
/// Queries start with `mut` extract the mutable reference (`&mut`) to the inner value instead:
///
/// ```txt
/// query_value!(mut query...) -> Option(&mut Value)
/// ```
///
/// Example:
///
/// ```
/// use serde_json::{json, Value};
/// use valq::query_value;
///
/// let mut obj = json!({"foo": { "bar": { "x": 1, "y": 2 }}});
/// {
///     let bar: &mut Value = query_value!(mut obj.foo.bar).unwrap();
///     *bar = json!({"x": 100, "y": 200});
/// }
/// // see below for `->` syntax
/// assert_eq!(query_value!(obj.foo.bar.x -> u64), Some(100));
/// assert_eq!(query_value!(obj.foo.bar.y -> u64), Some(200));
/// ```
///
/// ## `->`: Converting Value with `as_***()`
///
/// Queries end with `-> ***` try to convert the extracted value with `as_***()` method.
/// In the `mut` context, `as_***_mut()` method is used instead.
///
/// ```txt
/// // assuming your value has the method `as_str(&self) -> Option(&str)`
/// query_value!(query... -> str) -> Option(&str)
///
/// // assuming your value has the method `as_array_mut(&mut self) -> Option(&mut Vec<Value>)`
/// query_value!(mut query... -> array) -> Option(&mut Vec<Value>)
/// ```
///
/// ```
/// use serde_json::{json, Value};
/// use valq::query_value;
///
/// let mut obj = json!({"foo": "hello", "arr": [1, 2]});
///
/// // try to convert extracted value with `as_u64` method on that value
/// // results in `None` in case of type mismatch
/// let foo_str: Option<&str> = query_value!(obj.foo -> str);
/// assert_eq!(foo_str, Some("hello"));
///
/// // `mut` example
/// let arr_vec: Option<&mut Vec<Value>> = query_value!(mut obj.arr -> array);
/// assert_eq!(arr_vec, Some(&mut vec![json!(1), json!(2)]));
/// ```
///
/// ## `>>`: Deserializing Value into Any Types Implement `serde::Deserialize` trait
///
/// Queries end with `>> Type` try to deserialize the extracted value using `deserialize()` method on the `Type`.
/// i.e. you can get a value of your `Type` out of the queried value, assuming your `Type` implements `serde::Deserialize`.
///
/// ```txt
/// // assuming `Type` has a method `deserialize()` that is compatible with the extracted value
/// query_value!(query... >> Type) -> Option(Type)
/// ```
///
/// ```
/// use serde::Deserialize;
/// use serde_json::json;
/// use valq::query_value;
///
/// #[derive(Debug, PartialEq, Deserialize)]
/// struct Person {
///     name: String,
///     age: u8,
/// }
///
/// let j = json!({"author": {"name": "jiftechnify", "age": 31}});
/// assert_eq!(
///     query_value!(j.author >> Person),
///     Some(Person {
///         name: "jiftechnify".into(),
///         age: 31u8,
///     }),
/// );
/// ```
///
/// Note that deserialization with `>>` involves cloning of the queried value. You may want to use `->` conversion if possible.
///
/// # Query Syntax
///
/// ```txt
/// query_value!(("mut")? <value> ("." <key> | "[" <idx> "]")* ("->" <as_dest> | ">>" <deser_dest>)?)
/// ```
///
/// where:
///
/// - `<value>`: An expression evaluates to a structured data to be queried
/// - `<key>`: A property/field key to extract value from a key-value structure
/// - `<idx>`: An index to extract value from structure
///     + For an array-like structure, any expressions evaluates to an integer can be used
///     + For a key-value structure, any expressions evaluates to a string can be used
///         * You may want to use this syntax to get a value paired with a non-identifier key (e.g. starts with digits, like `"1st"`)
/// - `<as_dest>`: A destination type of conversion with `as_***()` / `as_***_mut()` methods
/// - `<deser_dest>`: A type name into which the queried value is deserialized
///     + The specified type *MUST* implement the `serde::Deserialize` trait.
///
/// # Compatibility
/// `query_value!` can be used with arbitrary data structure(to call, `Value`) that supports `get(&self, idx) -> Option<&Value>` method that retrieves a value at `idx`(can be string (retrieving "property"/"field"), or integer (indexing "array"/"sequence")).
///
/// Extracting mutable reference is also supported when `Value` supports `get_mut(&mut self, idx) -> Option<&Value>`.
///
/// Instances of compatible data structures:
///
/// - [`serde_json::Value`](https://docs.rs/serde_json/latest/serde_json/enum.Value.html)
/// - [`serde_yaml::Value`](https://docs.rs/serde_yaml/latest/serde_yaml/enum.Value.html)
/// - [`toml::Value`](https://docs.rs/toml/latest/toml/value/enum.Value.html)
/// - and more...
#[macro_export]
macro_rules! query_value {
    /* non-mut traversal */
    (@trv { $vopt:expr }) => {
        $vopt
    };
    (@trv { $vopt:expr } -> $dest:ident) => {
        $crate::paste! {
            $vopt.and_then(|v| v.[<as_ $dest>]())
        }
    };
    (@trv { $vopt:expr } >> $dest:ty) => {
        $vopt.and_then(|v| <$dest>::deserialize(v.clone()).ok())
    };
    (@trv { $vopt:expr } . $key:ident $($rest:tt)*) => {
        query_value!(@trv { $vopt.and_then(|v| v.get(stringify!($key))) } $($rest)*)
    };
    (@trv { $vopt:expr } [ $idx:expr ] $($rest:tt)*) => {
        query_value!(@trv { $vopt.and_then(|v| v.get($idx)) } $($rest)*)
    };
    (@trv $($_:tt)*) => {
        compile_error!("invalid query syntax for query_value!()")
    };

    /* mut traversal */
    (@trv_mut { $vopt:expr }) => {
        $vopt
    };
    (@trv_mut { $vopt:expr } -> $dest:ident) => {
        $crate::paste! {
            $vopt.and_then(|v| v.[<as_ $dest _mut>]())
        }
    };
    (@trv_mut { $vopt:expr } >> $dest:ty) => {
        $vopt.and_then(|v| <$dest>::deserialize(v.clone()).ok())
    };
    (@trv_mut { $vopt:expr } . $key:ident $($rest:tt)*) => {
        query_value!(@trv_mut { $vopt.and_then(|v| v.get_mut(stringify!($key))) } $($rest)*)
    };
    (@trv_mut { $vopt:expr } [ $idx:expr ] $($rest:tt)*) => {
        query_value!(@trv_mut { $vopt.and_then(|v| v.get_mut($idx)) } $($rest)*)
    };
    (@trv_mut $($_:tt)*) => {
        compile_error!("invalid query syntax for query_value!()")
    };

    /* entry points */
    (mut $v:tt $($rest:tt)*) => {
      query_value!(@trv_mut { Some(&mut $v) } $($rest)*)
    };
    ($v:tt $($rest:tt)*) => {
      query_value!(@trv { Some(&$v) } $($rest)*)
    };
}

#[cfg(test)]
mod tests {
    use super::query_value;
    macro_rules! test_is_some_of_expected_val {
        ($tests:expr) => {
            for (res, exp) in $tests {
                if let Some(act) = res {
                    assert_eq!(act, &exp)
                } else {
                    panic!("expect Some(...) but actually None")
                }
            }
        };
    }

    macro_rules! test_all_true_or_failed_idx {
        ($test_res:expr) => {
            if let Some(failed_idx) = $test_res.iter().position(|&r| !r) {
                panic!("test idx: {} failed", failed_idx)
            }
        };
    }

    #[cfg(test)]
    mod json {
        use serde_json::{json, Value};

        fn make_sample_json() -> Value {
            json!({
                "str": "s",
                "nums": {
                    "u64": 123,
                    "i64": -123,
                    "f64": 1.23,
                },
                "bool": true,
                "null": null,
                "obj": {
                    "inner": "zzz"
                },
                "arr": [
                    "first",
                    42,
                    { "hidden": "tale" },
                    [0]
                ],
                "1st": "prop starts with digit!"
            })
        }

        #[test]
        fn test_query_with_dot_syntax() {
            let j = make_sample_json();

            let tests = vec![
                (query_value!(j.str), json!("s")),
                (query_value!(j.nums.u64), json!(123)),
                (query_value!(j.nums.i64), json!(-123)),
                (query_value!(j.nums.f64), json!(1.23)),
                (query_value!(j.bool), json!(true)),
                (query_value!(j.null), json!(null)),
                (query_value!(j.obj), json!({"inner": "zzz"})),
                (
                    query_value!(j.arr),
                    json!(["first", 42, {"hidden": "tale"}, [0]]),
                ),
                (query_value!(j["1st"]), json!("prop starts with digit!")),
            ];

            test_is_some_of_expected_val!(tests);
        }

        #[test]
        fn test_query_with_bracket_syntax() {
            let j = make_sample_json();

            let tests = vec![
                (query_value!(j["str"]), json!("s")),
                (query_value!(j["nums"]["u64"]), json!(123)),
                (query_value!(j["nums"].i64), json!(-123)), // mixed query
                (query_value!(j["1st"]), json!("prop starts with digit!")),
            ];

            test_is_some_of_expected_val!(tests);
        }

        #[test]
        fn test_indexing_array() {
            let j = make_sample_json();
            let tests = vec![
                (query_value!(j.arr[0]), json!("first")),
                (query_value!(j.arr[1]), json!(42)),
                (query_value!(j.arr[2].hidden), json!("tale")), // more complex query!
                (query_value!(j.arr[3][0]), json!(0)),          // successive indexing
            ];

            test_is_some_of_expected_val!(tests);
        }

        #[test]
        fn test_query_and_convert() {
            let j = make_sample_json();

            let tests = [
                query_value!(j.str -> str) == Some("s"),
                query_value!(j.nums.u64 -> u64) == Some(123),
                query_value!(j.nums.i64 -> i64) == Some(-123),
                query_value!(j.nums.f64 -> f64) == Some(1.23),
                query_value!(j.bool -> bool) == Some(true),
                query_value!(j.null -> null) == Some(()),
                query_value!(j.obj -> object).unwrap().get("inner").unwrap() == "zzz",
                query_value!(j.arr -> array).unwrap()
                    == &vec![
                        json!("first"),
                        json!(42),
                        json!({"hidden": "tale"}),
                        json!([0]),
                    ],
            ];

            test_all_true_or_failed_idx!(tests);
        }

        #[test]
        fn test_query_and_deserialize() {
            use serde::Deserialize;

            #[derive(Debug, PartialEq, Deserialize)]
            struct Person {
                name: String,
                age: u8,
            }

            let j = json!({ "author": {"name": "jiftechnify", "age": 31 } });
            assert_eq!(
                query_value!(j.author >> Person),
                Some(Person {
                    name: "jiftechnify".into(),
                    age: 31u8,
                }),
            );
        }

        #[test]
        fn test_query_mut() {
            let mut j = make_sample_json();

            // rewriting value of prop
            {
                let obj_inner = query_value!(mut j.obj.inner).unwrap();
                *obj_inner = json!("just woke up!");
            }
            assert_eq!(
                query_value!(j.obj),
                Some(&json!({"inner": "just woke up!"}))
            );

            // get inner object as Map, then add new prop via insert()
            {
                let obj = query_value!(mut j.obj -> object).unwrap();
                obj.insert("new_prop".to_string(), json!("yeah"));
            }
            assert_eq!(query_value!(j.obj.new_prop -> str), Some("yeah"));

            // get inner array as Vec, then append new value via push()
            {
                let arr = query_value!(mut j.arr -> array).unwrap();
                arr.push(json!("appended!"));
            }
            assert_eq!(query_value!(j.arr[4] -> str), Some("appended!"));
        }

        #[test]
        fn test_query_fail() {
            let j = make_sample_json();

            let tests = [
                query_value!(j.unknown),   // non existent property
                query_value!(j.nums.i128), // non existent property of nested object
                query_value!(j.obj[0]),    // indexing against non-array value
                query_value!(j.arr[100]),  // indexing out of bound
            ]
            .iter()
            .map(|res| res.is_none())
            .collect::<Vec<_>>();

            test_all_true_or_failed_idx!(tests);
        }

        #[test]
        fn test_query_fail_mut() {
            let mut j = make_sample_json();

            let tests = [
                { query_value!(mut j.unknown).is_none() },
                { query_value!(mut j.nums.i128).is_none() },
                { query_value!(mut j.obj[0]).is_none() },
                { query_value!(mut j.arr[100]).is_none() },
            ];

            test_all_true_or_failed_idx!(tests);
        }
    }

    #[cfg(test)]
    mod yaml {
        use super::query_value;
        use serde_yaml::{from_str, Mapping, Sequence, Value};

        fn make_sample_yaml() -> Value {
            let yaml_str = include_str!("../res/sample.yaml");
            from_str(yaml_str).unwrap()
        }

        fn sample_mapping() -> Mapping {
            Mapping::from_iter([
                (
                    Value::String("first".to_string()),
                    Value::String("zzz".to_string()),
                ),
                (
                    Value::String("second".to_string()),
                    Value::String("yyy".to_string()),
                ),
            ])
        }
        fn sample_map_in_seq() -> Mapping {
            Mapping::from_iter([(
                Value::String("hidden".to_string()),
                Value::String("tale".to_string()),
            )])
        }
        fn sample_sequence() -> Sequence {
            Sequence::from_iter(vec![
                Value::String("first".to_string()),
                Value::Number(42.into()),
                Value::Mapping(sample_map_in_seq()),
            ])
        }

        #[test]
        fn test_query() {
            let y = make_sample_yaml();

            let tests = vec![
                (query_value!(y.str), Value::String("s".to_string())),
                (query_value!(y.num), Value::Number(123.into())),
                (query_value!(y.map), Value::Mapping(sample_mapping())),
                (query_value!(y.map.second), Value::String("yyy".to_string())),
                (query_value!(y.seq), Value::Sequence(sample_sequence())),
                (query_value!(y.seq[0]), Value::String("first".to_string())),
                (query_value!(y.seq[2]), Value::Mapping(sample_map_in_seq())),
            ];
            test_is_some_of_expected_val!(tests);
        }

        #[test]
        fn test_query_and_convert() {
            let y = make_sample_yaml();

            let tests = [
                query_value!(y.str -> str) == Some("s"),
                query_value!(y.num -> u64) == Some(123),
                query_value!(y.map -> mapping).unwrap().len() == 2,
                query_value!(y.seq -> sequence).unwrap()
                    == &vec![
                        Value::String("first".to_string()),
                        Value::Number(42.into()),
                        Value::Mapping(sample_map_in_seq()),
                    ],
            ];

            test_all_true_or_failed_idx!(tests);
        }

        #[test]
        fn test_query_and_deserialize() {
            use serde::Deserialize;

            #[derive(Debug, PartialEq, Deserialize)]
            struct Person {
                name: String,
                age: u8,
            }

            let y = make_sample_yaml();
            assert_eq!(
                query_value!(y.author >> Person),
                Some(Person {
                    name: "jiftechnify".into(),
                    age: 31u8,
                }),
            );
        }
    }

    #[cfg(test)]
    mod toml {
        use super::query_value;
        use toml::{
            from_str,
            value::{Array, Table},
            Value,
        };

        fn make_sample_toml() -> Value {
            let toml_str = include_str!("../res/sample.toml");
            from_str(toml_str).unwrap()
        }
        fn sample_table() -> Table {
            Table::from_iter([
                ("first".to_string(), Value::String("zzz".to_string())),
                ("second".to_string(), Value::String("yyy".to_string())),
            ])
        }
        fn sample_array() -> Array {
            vec!["first", "second", "third"]
                .into_iter()
                .map(|e| Value::String(e.to_string()))
                .collect()
        }
        fn sample_arr_of_tables() -> Array {
            let t1 = Table::from_iter([("hidden".to_string(), Value::String("tale".to_string()))]);
            let t2 = Table::from_iter([
                ("hoge".to_string(), Value::Integer(1)),
                ("fuga".to_string(), Value::Integer(2)),
            ]);
            let t3 = Table::from_iter([(
                "inner_arr".to_string(),
                Value::Array(vec![
                    Value::Integer(1),
                    Value::Integer(2),
                    Value::Integer(3),
                ]),
            )]);

            vec![t1, t2, t3].into_iter().map(Value::Table).collect()
        }

        #[test]
        fn test_query() {
            let t = make_sample_toml();

            let tests = vec![
                (query_value!(t.str), Value::String("s".to_string())),
                (query_value!(t.int), Value::Integer(123)),
                (query_value!(t.float), Value::Float(1.23)),
                (query_value!(t.table), Value::Table(sample_table())),
                (
                    query_value!(t.table.second),
                    Value::String("yyy".to_string()),
                ),
                (query_value!(t.arr), Value::Array(sample_array())),
                (query_value!(t.arr[2]), Value::String("third".to_string())),
                (
                    query_value!(t.arr_of_tables),
                    Value::Array(sample_arr_of_tables()),
                ),
                (
                    query_value!(t.arr_of_tables[0].hidden),
                    Value::String("tale".to_string()),
                ),
                (
                    query_value!(t.arr_of_tables[2].inner_arr[0]),
                    Value::Integer(1),
                ),
            ];
            test_is_some_of_expected_val!(tests);
        }

        #[test]
        fn test_query_and_convert() {
            let t = make_sample_toml();

            let tests = [
                query_value!(t.str -> str) == Some("s"),
                query_value!(t.int -> integer) == Some(123),
                query_value!(t.float -> float) == Some(1.23),
                query_value!(t.date -> datetime).unwrap().to_string()
                    == "2021-12-18T12:15:12+09:00",
                query_value!(t.table -> table).unwrap().len() == 2,
                query_value!(t.arr -> array).unwrap()
                    == &vec!["first", "second", "third"]
                        .into_iter()
                        .map(|v| Value::String(v.to_string()))
                        .collect::<Vec<_>>(),
                query_value!(t.arr_of_tables -> array).unwrap().len() == 3,
            ];

            test_all_true_or_failed_idx!(tests);
        }

        #[test]
        fn test_query_and_deserialize() {
            use serde::Deserialize;

            #[derive(Debug, PartialEq, Deserialize)]
            struct Person {
                name: String,
                age: u8,
            }

            let t = make_sample_toml();
            assert_eq!(
                query_value!(t.author >> Person),
                Some(Person {
                    name: "jiftechnify".into(),
                    age: 31u8,
                }),
            );
        }
    }
}
