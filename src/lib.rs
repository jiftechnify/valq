//! # valq
//! `valq` provides a macro for querying and extracting value from structured data **in very concise manner, like the JavaScript syntax**.
//!
//! look & feel:
//!
//! ```ignore
//! use serde_json::Value;
//! use valq::query_value;
//!
//! let j: Value = ...;
//! let deep_val: Option<&Value> = query_value!(j.path.to.value.at.deep);
//! ```
//!
//! For now, there is only single macro exported: `query_value`. See document of `query_value` for detailed usage.

/// A macro for querying inner value of structured data.
///
/// # Examples
/// ## Basic Usage
/// ```ignore
/// // get field `foo` from JSON object `obj`
/// let foo = query_value!(obj.foo);
///
/// // get nested field `bar` inside object `foo` in JSON object `obj`
/// let bar = query_value!(obj.foo.bar);
///
/// // get head of JSON array 'arr'
/// let head = query_value!(arr[0]);
///
/// // get head of nested JSON array `arr` in JSON object `obj`
/// let head = query_value!(obj.arr[0]);
///
/// // more complex example!
/// let abyss = query_value!(obj.path.to.matrix[0][1].abyss);
/// ```
///
/// ## Converting to Specified Type
/// ```ignore
/// // try to convert extracted value to `u64` by `as_u64()` method  on that value.
/// // results in `None` in case of type mismatch
/// let foo_u64: Option<u64> = query_value!(obj.foo -> u64)
///
/// // in case of mutable reference extraction (see below), `as_xxx_mut()` method will be used.
/// let arr_vec: Option<&mut Vec<Value>> = query_value!(mut obj.arr -> array)
/// ```
///
/// ## Extracting Mutable Reference to Inner Value
/// ```
/// use serde_json::{json, Value};
/// use valq::query_value;
///
/// let mut obj = json!({"foo": { "bar": { "x": 1, "y": 2 }}});
/// {
///     // prefixed `mut` means extracting mutable reference
///     let bar: &mut Value = query_value!(mut obj.foo.bar).unwrap();
///     *bar = json!({"x": 100, "y": 200});
/// }
/// assert_eq!(query_value!(obj.foo.bar.x -> u64), Some(100));
/// assert_eq!(query_value!(obj.foo.bar.y -> u64), Some(200));
/// ```
///
/// # Query Syntax
///
/// ```txt
/// query_value!(("mut")? <value> ("." <key> | "[" <idx> "]")+ ("->" <to_type>)?)
/// ```
///
/// where:
///
/// - `<value>`: An expression of structured data to query
/// - `<key>`: A key of "property"/"field to extract
///     + Any identifiers or `str` literals can be used. You may want to use `str` literals to get property keyed by a string that is invalid identifier in Rust (e.g. starts with digits).
/// - `<idx>`: An index of array-like stracture to extract
///     + Any expressions evaluates to integer value can be used.
/// - `<to_type>`: A name of "type" queried value should be converted to
///
/// # Compatibility
/// This macro can be used with arbitrary data structure(to call, `Value`) that supports `get(&self, idx) -> Option<&Value>` method that retrieves a value at `idx`(can be string (retrieving "property"/"field"), or integer (indexing "array"/"sequence")).
///
/// Type conversion query `-> xxx` is available if `Value` has conversion method `as_xxx(&self) -> Option<X>`/`as_xxx_mut(&mut self) -> Option<X>`.
///
/// Extracting mutable reference is also supported when `Value` supports `get_mut(&mut self, idx) -> Option<&Value>`.
///
/// Instances of compatible data structures:
///
/// - [`serde_json::Value`](https://docs.rs/serde_json/latest/serde_json/enum.Value.html)
/// - [`serde_yaml::Value`](https://docs.rs/serde_yaml/latest/serde_yaml/enum.Value.html)
/// - [`toml::Value`](https://docs.rs/toml/latest/toml/value/enum.Value.html)
/// - and more...
///
#[macro_export]
macro_rules! query_value {
    /* non-mut traversal */
    (@trv { $vopt:expr }) => {
        $vopt
    };
    (@trv { $vopt:expr } -> $to:ident) => {
        $vopt.and_then(|v| query_value!(@conv v, $to))
    };
    (@trv { $vopt:expr } . $key:ident $($rest:tt)*) => {
        query_value!(@trv { $vopt.and_then(|v| v.get(stringify!($key))) } $($rest)*)
    };
    (@trv { $vopt:expr } . $key:literal $($rest:tt)*) => {
        query_value!(@trv { $vopt.and_then(|v| v.get($key as &str)) } $($rest)*)
    };
    (@trv { $vopt:expr } [ $idx:expr ] $($rest:tt)*) => {
        query_value!(@trv { $vopt.and_then(|v| v.get($idx as usize)) } $($rest)*)
    };
    (@trv $($_:tt)*) => {
        compile_error!("invalid query syntax for query_value!()")
    };

    /* non-mut conversion */
    (@conv $v:expr, str) => {
        $v.as_str()
    };
    (@conv $v:expr, u64) => {
        $v.as_u64()
    };
    (@conv $v:expr, i64) => {
        $v.as_i64()
    };
    (@conv $v:expr, f64) => {
        $v.as_f64()
    };
    (@conv $v:expr, bool) => {
        $v.as_bool()
    };
    (@conv $v:expr, null) => {
        $v.as_null()
    };
    (@conv $v:expr, object) => {
        $v.as_object()
    };
    (@conv $v:expr, array) => {
        $v.as_array()
    };
    // for serde_yaml::Value
    (@conv $v:expr, mapping) => {
        $v.as_mapping()
    };
    (@conv $v:expr, sequence) => {
        $v.as_sequence()
    };
    // for toml::Value
    (@conv $v:expr, integer) => {
        $v.as_integer()
    };
    (@conv $v:expr, float) => {
        $v.as_float()
    };
    (@conv $v:expr, datetime) => {
        $v.as_datetime()
    };
    (@conv $v:expr, table) => {
        $v.as_table()
    };
    (@conv $v:expr, $to:ident) => {
        compile_error!(concat!("unsupported target type `", stringify!($to), "` is specified in query_value!()"))
    };

    /* mut traversal */
    (@trv_mut { $vopt:expr }) => {
        $vopt
    };
    (@trv_mut { $vopt:expr } -> $to:ident) => {
        $vopt.and_then(|v| query_value!(@conv_mut v, $to))
    };
    (@trv_mut { $vopt:expr } . $key:ident $($rest:tt)*) => {
        query_value!(@trv_mut { $vopt.and_then(|v| v.get_mut(stringify!($key))) } $($rest)*)
    };
    (@trv_mut { $vopt:expr } . $key:literal $($rest:tt)*) => {
        query_value!(@trv_mut { $vopt.and_then(|v| v.get_mut($key as &str)) } $($rest)*)
    };
    (@trv_mut { $vopt:expr } [ $idx:expr ] $($rest:tt)*) => {
        query_value!(@trv_mut { $vopt.and_then(|v| v.get_mut($idx as usize)) } $($rest)*)
    };
    (@trv_mut $($_:tt)*) => {
        compile_error!("invalid query syntax for query_value!()")
    };

    /* mut conversion */
    (@conv_mut $v:expr, val) => {
        Some($v)
    };
    (@conv_mut $v:expr, object) => {
        $v.as_object_mut()
    };
    (@conv_mut $v:expr, array) => {
        $v.as_array_mut()
    };
    // for serde_yaml::Value
    (@conv_mut $v:expr, mapping) => {
        $v.as_mapping_mut()
    };
    (@conv_mut $v:expr, sequence) => {
        $v.as_sequence_mut()
    };
    // for toml::Value
    (@conv_mut $v:expr, table) => {
        $v.as_table_mut()
    };
    (@conv_mut $v:expr, $to:ident) => {
        compile_error!(concat!("unsupported target type `", stringify!($to), "` is specified in query_value!()"))
    };

    /* entry point */
    ($v:tt . $key:ident $($rest:tt)*) => {
        query_value!(@trv { $v.get(stringify!($key)) } $($rest)*)
    };
    ($v:tt . $key:literal $($rest:tt)*) => {
        query_value!(@trv { $v.get($key as &str) } $($rest)*)
    };
    ($v:tt [ $idx:expr ] $($rest:tt)*) => {
        query_value!(@trv { $v.get($idx as usize) } $($rest)*)
    };
    (mut $v:tt . $key:ident $($rest:tt)*) => {
        query_value!(@trv_mut { $v.get_mut(stringify!($key)) } $($rest)*)
    };
    (mut $v:tt . $key:literal $($rest:tt)*) => {
        query_value!(@trv_mut { $v.get_mut($key as &str) } $($rest)*)
    };
    (mut $v:tt [ $idx:expr ] $($rest:tt)*) => {
        query_value!(@trv_mut { $v.get_mut($idx as usize) } $($rest)*)
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
            // json!({"foo": { "bar": { "x": 1, "y": 2 }}})
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
        fn test_query() {
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
                (query_value!(j."1st"), json!("prop starts with digit!")),
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
        fn test_query_mut() {
            let mut j = make_sample_json();

            // rewriting value of prop
            {
                let obj_innner = query_value!(mut j.obj.inner).unwrap();
                *obj_innner = json!("just woke up!");
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
    }
}
