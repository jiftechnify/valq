use valq::{query_value_result, QueryValueError};

macro_rules! test_all_true_or_failed_idx {
    ($test_res:expr) => {
        if let Some(failed_idx) = $test_res.iter().position(|&r| !r) {
            panic!("test idx: {} failed", failed_idx)
        }
    };
}

mod json {
    use super::{query_value_result, QueryValueError};
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
                "inner": "value",
                "more": "item"
            },
            "arr": [
                "first",
                42,
                { "hidden": "tale" },
                [0]
            ],
            "num_arr": [0, 1, 2],
            "1st": "prop starts with digit!"
        })
    }

    #[test]
    fn test_query_with_dot_syntax() {
        let j = make_sample_json();

        let tests = [
            (query_value_result!(j.str), json!("s")),
            (query_value_result!(j.nums.u64), json!(123)),
            (query_value_result!(j.nums.i64), json!(-123)),
            (query_value_result!(j.nums.f64), json!(1.23)),
            (query_value_result!(j.bool), json!(true)),
            (query_value_result!(j.null), json!(null)),
            (
                query_value_result!(j.obj),
                json!({"inner": "value", "more": "item"}),
            ),
            (query_value_result!(j.obj.inner), json!("value")),
            (
                query_value_result!(j.arr),
                json!(["first", 42, {"hidden": "tale"}, [0]]),
            ),
            (
                query_value_result!(j["1st"]),
                json!("prop starts with digit!"),
            ),
        ];

        for (res, exp) in tests {
            assert_eq!(res.unwrap(), &exp);
        }
    }

    #[test]
    fn test_query_with_bracket_syntax() {
        let j = make_sample_json();

        let tests = [
            (query_value_result!(j["str"]), json!("s")),
            (query_value_result!(j["nums"]["u64"]), json!(123)),
            (query_value_result!(j["nums"].i64), json!(-123)), // mixed query
            (
                query_value_result!(j["1st"]),
                json!("prop starts with digit!"),
            ),
        ];

        for (res, exp) in tests {
            assert_eq!(res.unwrap(), &exp);
        }
    }

    #[test]
    fn test_indexing_array() {
        let j = make_sample_json();
        let tests = [
            (query_value_result!(j.arr[0]), json!("first")),
            (query_value_result!(j.arr[1]), json!(42)),
            (query_value_result!(j.arr[2].hidden), json!("tale")), // more complex query!
            (query_value_result!(j.arr[3][0]), json!(0)),          // successive indexing
        ];

        for (res, exp) in tests {
            assert_eq!(res.unwrap(), &exp);
        }
    }

    #[test]
    fn test_query_mut() {
        let mut j = make_sample_json();

        // rewriting value of prop
        {
            let obj_inner = query_value_result!(mut j.obj.inner).unwrap();
            *obj_inner = json!("just woke up!");
        }
        assert_eq!(
            query_value_result!(j.obj).unwrap(),
            &json!({"inner": "just woke up!", "more": "item"})
        );

        // get inner object as Map, then add new prop via insert()
        {
            let obj = query_value_result!(mut j.obj -> object).unwrap();
            obj.insert("new_prop".to_string(), json!("yeah"));
        }
        assert_eq!(query_value_result!(j.obj.new_prop -> str).unwrap(), "yeah");

        // get inner array as Vec, then append new value via push()
        {
            let arr = query_value_result!(mut j.arr -> array).unwrap();
            arr.push(json!("appended!"));
        }
        assert_eq!(query_value_result!(j.arr[4] -> str).unwrap(), "appended!");
    }

    #[test]
    fn test_query_and_convert() {
        let j = make_sample_json();

        let tests = [
            query_value_result!(j.str -> str).unwrap() == "s",
            query_value_result!(j.nums.u64 -> u64).unwrap() == 123,
            query_value_result!(j.nums.i64 -> i64).unwrap() == -123,
            query_value_result!(j.nums.f64 -> f64).unwrap() == 1.23,
            query_value_result!(j.bool -> bool).unwrap() == true,
            query_value_result!(j.null -> null).unwrap() == (),
            query_value_result!(j.obj -> object)
                .unwrap()
                .get("inner")
                .unwrap()
                == "value",
            query_value_result!(j.arr -> array).unwrap()
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

        let j = make_sample_json();

        let tests = [
            query_value_result!(j.str >> (String)).unwrap() == "s",
            query_value_result!(j.str >> (std::string::String)).unwrap() == "s",
            query_value_result!(j.str >> String).unwrap() == "s", // parens around type name can be omitted if single identifier
            query_value_result!(j.nums.u64 >> u8).unwrap() == 123u8,
            query_value_result!(j.nums.i64 >> i8).unwrap() == -123i8,
            query_value_result!(j.nums.f64 >> f32).unwrap() == 1.23f32,
            query_value_result!(j.null >> (())).unwrap() == (),
        ];

        test_all_true_or_failed_idx!(tests);
    }

    #[test]
    fn test_deserialize_into_custom_struct() {
        use serde::Deserialize;

        #[derive(Debug, PartialEq, Deserialize)]
        struct Person {
            name: String,
            age: u8,
        }

        let j = json!({ "author": {"name": "jiftechnify", "age": 31 } });
        assert_eq!(
            query_value_result!(j.author >> Person).unwrap(),
            Person {
                name: "jiftechnify".into(),
                age: 31u8,
            },
        );
    }

    #[test]
    fn test_query_with_unwrapping() {
        let j = make_sample_json();

        let default_str = &json!("default");

        // basic query with ??
        assert_eq!(query_value_result!(j.str ?? default_str), &json!("s"));
        assert_eq!(
            query_value_result!(j.unknown ?? default_str),
            &json!("default")
        );

        // `?? default`
        assert_eq!(query_value_result!(j.nums.u64 -> u64 ?? default), 123u64);
        assert_eq!(query_value_result!(j.unknown -> u64 ?? default), 0u64); // u64::default()
        assert_eq!(query_value_result!(j.unknown -> str ?? default), ""); // &str::default()

        // with conversion (->)
        assert_eq!(query_value_result!(j.str -> str ?? "default"), "s");
        assert_eq!(query_value_result!(j.nums.u64 -> u64 ?? 999), 123);
        assert_eq!(
            query_value_result!(j.nums.u64 -> str ?? "not a string"),
            "not a string"
        ); // type mismatch
        assert_eq!(
            query_value_result!(j.unknown -> str ?? "default"),
            "default"
        );
        assert_eq!(query_value_result!(j.unknown -> str ?? default), ""); // &str::default()

        // with deserialization (>>)
        use serde::Deserialize;

        #[derive(Debug, PartialEq, Deserialize, Default)]
        struct Nums {
            u64: u64,
            i64: i64,
            f64: f64,
        }

        let expected_nums = Nums {
            u64: 123,
            i64: -123,
            f64: 1.23,
        };

        assert_eq!(
            query_value_result!(j.nums >> Nums ?? default),
            expected_nums
        );
        assert_eq!(
            query_value_result!(j.unknown >> Nums ?? Nums { u64: 999, i64: -999, f64: 9.99 }),
            Nums {
                u64: 999,
                i64: -999,
                f64: 9.99
            }
        );
        assert_eq!(
            query_value_result!(j.unknown >> Nums ?? default),
            Nums {
                u64: 0,
                i64: 0,
                f64: 0.0,
            }
        );

        assert_eq!(
            query_value_result!(j.num_arr >> (Vec<u8>) ?? default),
            vec![0, 1, 2]
        );
        assert_eq!(
            query_value_result!(j.arr >> (Vec<u8>) ?? default),
            Vec::<u8>::new()
        );
        assert_eq!(
            query_value_result!(j.arr >> (Vec<u8>) ?? vec![42]),
            vec![42]
        );
    }

    #[test]
    fn test_deserialize_into_vec() {
        use serde::Deserialize;

        let j = make_sample_json();

        let tests = [
            query_value_result!(j.num_arr >> (Vec<u8>)).unwrap() == vec![0, 1, 2],
            query_value_result!(j.num_arr >> (Vec<u8>) ?? default) == vec![0, 1, 2],
            query_value_result!(j.arr >> (Vec<u8>) ?? default) == Vec::<u8>::new(),
            query_value_result!(j.arr >> (Vec<u8>) ?? vec![42]) == vec![42],
        ];
        test_all_true_or_failed_idx!(tests);
    }

    #[test]
    fn test_deserialize_into_hash_map() {
        use serde::Deserialize;
        use serde_json::{json, Value};
        use std::collections::HashMap;

        let j = make_sample_json();

        let exp_json: HashMap<String, Value> = HashMap::from([
            ("inner".into(), json!("value")),
            ("more".into(), json!("item")),
        ]);

        let exp_string: HashMap<String, String> = HashMap::from([
            ("inner".into(), "value".into()),
            ("more".into(), "item".into()),
        ]);

        let tests = [
            query_value_result!(j.obj >> (HashMap<String, Value>)).unwrap() == exp_json,
            query_value_result!(j.obj >> (HashMap<String, String>)).unwrap() == exp_string,
        ];
        test_all_true_or_failed_idx!(tests);
    }

    #[test]
    fn test_query_complex_expressions() {
        fn gen_value() -> serde_json::Value {
            json!({ "x": 1 })
        }

        let tuple = (json!({ "x": 1 }),);

        struct S {
            value: serde_json::Value,
        }
        impl S {
            fn new() -> Self {
                Self {
                    value: json!({ "x": 1 }),
                }
            }
            fn gen_value(&self) -> serde_json::Value {
                json!({ "x": 1 })
            }
        }
        let s = S::new();

        let v = vec![json!({ "x": 1 })];

        let tests = [
            // querying immediate value
            query_value_result!((json!({ "x": 1 })).x).unwrap() == &json!(1),
            query_value_result!((json!({ "x": 1 })).y ?? default) == &json!(null),
            query_value_result!((json!({ "x": 1 })).x -> u64).unwrap() == 1u64,
            query_value_result!((json!({ "x": 1 })).x -> str ?? default) == "",
            query_value_result!((json!({ "x": 1 })).x -> str ?? "not str") == "not str",
            // querying immediate value (mut)
            query_value_result!(mut (json!({ "x": 1 })).x).unwrap() == &mut json!(1),
            // querying return value of function
            query_value_result!((gen_value()).x).unwrap() == &json!(1),
            query_value_result!((gen_value()).x -> u64).unwrap() == 1u64,
            query_value_result!((gen_value()).x -> str ?? "not str") == "not str",
            // querying element of tuple
            query_value_result!((tuple.0).x).unwrap() == &json!(1),
            query_value_result!((tuple.0).x -> u64).unwrap() == 1u64,
            query_value_result!((tuple.0).x -> str ?? "not str") == "not str",
            // querying field of struct
            query_value_result!((s.value).x).unwrap() == &json!(1),
            query_value_result!((s.value).x -> u64).unwrap() == 1u64,
            query_value_result!((s.value).x -> str ?? "not str") == "not str",
            // querying return value of method
            query_value_result!((s.gen_value()).x).unwrap() == &json!(1),
            query_value_result!((s.gen_value()).x -> u64).unwrap() == 1u64,
            query_value_result!((s.gen_value()).x -> str ?? "not str") == "not str",
            // querying indexed value
            query_value_result!((v[0]).x).unwrap() == &json!(1),
            query_value_result!((v[0]).x -> u64).unwrap() == 1u64,
            query_value_result!((v[0]).x -> str ?? "not str") == "not str",
        ];
        test_all_true_or_failed_idx!(tests);
    }

    #[test]
    fn test_query_with_dynamic_indices() {
        let j = make_sample_json();

        // Dynamic string key
        let key = "str";
        assert_eq!(query_value_result!(j[key]).unwrap(), &json!("s"));

        let obj_key = "obj";
        let inner_key = "inner";
        assert_eq!(
            query_value_result!(j[obj_key][inner_key]).unwrap(),
            &json!("value")
        );

        // Dynamic integer index
        let index = 0;
        assert_eq!(query_value_result!(j.arr[index]).unwrap(), &json!("first"));

        let arr_index = 1;
        assert_eq!(query_value_result!(j.arr[arr_index]).unwrap(), &json!(42));

        // Mix of static and dynamic
        let key2 = "nums";
        assert_eq!(query_value_result!(j[key2].u64).unwrap(), &json!(123));

        // Dynamic expression
        let base_index = 1;
        assert_eq!(
            query_value_result!(j.arr[base_index + 1].hidden).unwrap(),
            &json!("tale")
        );

        // With conversion
        assert_eq!(query_value_result!(j[key] -> str).unwrap(), "s");
        assert_eq!(query_value_result!(j.arr[index] -> str).unwrap(), "first");

        // With unwrapping operator
        let missing_key = "missing";
        let fallback = json!("fallback");
        assert_eq!(
            query_value_result!(j[missing_key]?? & fallback),
            &json!("fallback")
        );

        let out_of_bounds = 999;
        let oob_val = json!("oob");
        assert_eq!(
            query_value_result!(j.arr[out_of_bounds]?? & oob_val),
            &json!("oob")
        );
    }

    #[test]
    fn test_query_with_dynamic_indices_mut() {
        let mut j = make_sample_json();

        // Dynamic string key (mut)
        let key = "str";
        {
            let val = query_value_result!(mut j[key]).unwrap();
            *val = json!("modified");
        }
        assert_eq!(query_value_result!(j.str).unwrap(), &json!("modified"));

        // Dynamic integer index (mut)
        let index = 1;
        {
            let val = query_value_result!(mut j.arr[index]).unwrap();
            *val = json!(100);
        }
        assert_eq!(query_value_result!(j.arr[1]).unwrap(), &json!(100));

        // With conversion (mut)
        let obj_key = "obj";
        let dynamic_key = "dynamic_key";
        {
            let obj = query_value_result!(mut j[obj_key] -> object).unwrap();
            obj.insert("dynamic_key".to_string(), json!("added"));
        }
        assert_eq!(
            query_value_result!(j.obj[dynamic_key]).unwrap(),
            &json!("added")
        );
    }

    // Error case tests - ValueNotFoundAtPath
    #[test]
    fn test_error_value_not_found() {
        let j = make_sample_json();

        let tests = [
            (query_value_result!(j.unknown), ".unknown"),
            (query_value_result!(j.nums.i128), ".nums.i128"),
            (query_value_result!(j.obj[0]), ".obj[0]"), // indexing against non-array value
            (query_value_result!(j.arr[100]), ".arr[100]"), // indexing out of bound
            (
                query_value_result!(j.obj.inner.not_here.oh.nothing.but.pain),
                ".obj.inner.not_here", // make sure that it reports the shallowest non-existing path
            ),
        ];

        for (result, expected_path) in tests {
            if let Err(QueryValueError::ValueNotFoundAtPath(path)) = result {
                assert_eq!(path, expected_path);
            }
            else {
                panic!("expected ValueNotFoundAtPath error, but got: {:?}", result);
            }
        }
    }

    // Error case tests - AsCastFailed
    #[test]
    fn test_error_as_cast_failed() {
        let j = make_sample_json();

        let tests = [
            (query_value_result!(j.str -> u64).unwrap_err(), "as_u64"),
            (
                query_value_result!(j.nums.u64 -> str).unwrap_err(),
                "as_str",
            ),
            (query_value_result!(j.obj -> array).unwrap_err(), "as_array"),
            (
                query_value_result!(j.arr -> object).unwrap_err(),
                "as_object",
            ),
        ];

        for (result, expected_conv_name) in tests {
            if let QueryValueError::AsCastFailed(conv_name) = result {
                assert_eq!(conv_name, expected_conv_name);
            }
            else {
                panic!("expected AsCastFailed error, but got: {:?}", result);
            }
        }
    }

    // Error case tests - AsCastFailed (mut)
    #[test]
    fn test_error_as_cast_failed_mut() {
        let mut j = make_sample_json();

        let tests = [
            (
                query_value_result!(mut j.obj -> array).unwrap_err(),
                "as_array_mut",
            ),
            (
                query_value_result!(mut j.arr -> object).unwrap_err(),
                "as_object_mut",
            ),
        ];

        for (result, expected_conv_name) in tests {
            if let QueryValueError::AsCastFailed(conv_name) = result {
                assert_eq!(conv_name, expected_conv_name);
            }
            else {
                panic!("expected AsCastFailed error, but got: {:?}", result);
            }
        }
    }

    // Error case tests - DeserializationFailed
    #[test]
    fn test_error_deserialization_failed() {
        use serde::Deserialize;
        use std::collections::HashMap;

        let j = make_sample_json();

        #[derive(Debug, PartialEq, Deserialize)]
        struct Person {
            name: String,
            age: u8,
        }

        let tests = [
            query_value_result!(j.nums.i64 >> u8).unwrap_err(),
            query_value_result!(j.str >> u8).unwrap_err(),
            query_value_result!(j.obj >> Person).unwrap_err(),
            query_value_result!(j.arr >> (Vec<u8>)).unwrap_err(),
            query_value_result!(j.obj >> (HashMap<String, u8>)).unwrap_err(),
        ];

        for result in tests {
            assert!(matches!(result, QueryValueError::DeserializationFailed(_)));
        }
    }

    // Test error display
    #[test]
    fn test_error_display() {
        use serde::Deserialize;

        let j = make_sample_json();

        // ValueNotFoundAtPath
        let err = query_value_result!(j.unknown).unwrap_err();
        assert_eq!(err.to_string(), "value not found at the path: .unknown");

        // AsCastFailed
        let err = query_value_result!(j.str -> u64).unwrap_err();
        assert_eq!(err.to_string(), "conversion with as_u64() failed");

        // DeserializationFailed
        let err = query_value_result!(j.nums.i64 >> u8).unwrap_err();
        assert!(err
            .to_string()
            .starts_with("failed to deserialize the queried value:"));
    }
}

mod yaml {
    use super::{query_value_result, QueryValueError};
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

        let tests = [
            (query_value_result!(y.str), Value::String("s".to_string())),
            (query_value_result!(y.num), Value::Number(123.into())),
            (query_value_result!(y.map), Value::Mapping(sample_mapping())),
            (
                query_value_result!(y.map.second),
                Value::String("yyy".to_string()),
            ),
            (
                query_value_result!(y.seq),
                Value::Sequence(sample_sequence()),
            ),
            (
                query_value_result!(y.seq[0]),
                Value::String("first".to_string()),
            ),
            (
                query_value_result!(y.seq[2]),
                Value::Mapping(sample_map_in_seq()),
            ),
        ];

        for (res, exp) in tests {
            assert_eq!(res.unwrap(), &exp);
        }
    }

    #[test]
    fn test_query_and_convert() {
        let y = make_sample_yaml();

        let tests = [
            query_value_result!(y.str -> str).unwrap() == "s",
            query_value_result!(y.num -> u64).unwrap() == 123,
            query_value_result!(y.map -> mapping).unwrap().len() == 2,
            query_value_result!(y.seq -> sequence).unwrap()
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
            query_value_result!(y.author >> Person).unwrap(),
            Person {
                name: "jiftechnify".into(),
                age: 31u8,
            },
        );
    }

    // Error case tests for YAML
    #[test]
    fn test_error_value_not_found() {
        let y = make_sample_yaml();

        let result = query_value_result!(y.unknown);
        assert!(matches!(
            result,
            Err(QueryValueError::ValueNotFoundAtPath(_))
        ));
        if let Err(QueryValueError::ValueNotFoundAtPath(path)) = result {
            assert_eq!(path, ".unknown");
        }
    }

    #[test]
    fn test_error_as_cast_failed() {
        let y = make_sample_yaml();

        // string cannot be converted to u64
        let result = query_value_result!(y.str -> u64);
        assert!(matches!(result, Err(QueryValueError::AsCastFailed(_))));
    }

    #[test]
    fn test_error_deserialization_failed() {
        use serde::Deserialize;

        let y = make_sample_yaml();

        // string into u8
        let result = query_value_result!(y.str >> u8);
        assert!(matches!(
            result,
            Err(QueryValueError::DeserializationFailed(_))
        ));
    }
}

mod toml {
    use super::{query_value_result, QueryValueError};
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

        let tests = [
            (query_value_result!(t.str), Value::String("s".to_string())),
            (query_value_result!(t.int), Value::Integer(123)),
            (query_value_result!(t.float), Value::Float(1.23)),
            (query_value_result!(t.table), Value::Table(sample_table())),
            (
                query_value_result!(t.table.second),
                Value::String("yyy".to_string()),
            ),
            (query_value_result!(t.arr), Value::Array(sample_array())),
            (
                query_value_result!(t.arr[2]),
                Value::String("third".to_string()),
            ),
            (
                query_value_result!(t.arr_of_tables),
                Value::Array(sample_arr_of_tables()),
            ),
            (
                query_value_result!(t.arr_of_tables[0].hidden),
                Value::String("tale".to_string()),
            ),
            (
                query_value_result!(t.arr_of_tables[2].inner_arr[0]),
                Value::Integer(1),
            ),
        ];

        for (res, exp) in tests {
            assert_eq!(res.unwrap(), &exp);
        }
    }

    #[test]
    fn test_query_and_convert() {
        let t = make_sample_toml();

        let tests = [
            query_value_result!(t.str -> str).unwrap() == "s",
            query_value_result!(t.int -> integer).unwrap() == 123,
            query_value_result!(t.float -> float).unwrap() == 1.23,
            query_value_result!(t.date -> datetime).unwrap().to_string()
                == "2021-12-18T12:15:12+09:00",
            query_value_result!(t.table -> table).unwrap().len() == 2,
            query_value_result!(t.arr -> array).unwrap()
                == &vec!["first", "second", "third"]
                    .into_iter()
                    .map(|v| Value::String(v.to_string()))
                    .collect::<Vec<_>>(),
            query_value_result!(t.arr_of_tables -> array).unwrap().len() == 3,
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
            query_value_result!(t.author >> Person).unwrap(),
            Person {
                name: "jiftechnify".into(),
                age: 31u8,
            },
        );
    }

    // Error case tests for TOML
    #[test]
    fn test_error_value_not_found() {
        let t = make_sample_toml();

        let result = query_value_result!(t.unknown);
        assert!(matches!(
            result,
            Err(QueryValueError::ValueNotFoundAtPath(_))
        ));
        if let Err(QueryValueError::ValueNotFoundAtPath(path)) = result {
            assert_eq!(path, ".unknown");
        }
    }

    #[test]
    fn test_error_as_cast_failed() {
        let t = make_sample_toml();

        // string cannot be converted to integer
        let result = query_value_result!(t.str -> integer);
        assert!(matches!(result, Err(QueryValueError::AsCastFailed(_))));
    }

    #[test]
    fn test_error_deserialization_failed() {
        use serde::Deserialize;

        let t = make_sample_toml();

        // string into u8
        let result = query_value_result!(t.str >> u8);
        assert!(matches!(
            result,
            Err(QueryValueError::DeserializationFailed(_))
        ));
    }
}
