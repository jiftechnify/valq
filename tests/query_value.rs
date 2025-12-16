use valq::query_value;

macro_rules! test_is_some_of_expected_val {
    ($tests:expr) => {
        for (res, exp) in $tests {
            if let Some(act) = res {
                assert_eq!(act, &exp)
            }
            else {
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

mod json {
    use super::query_value;
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
            (query_value!(j.str), json!("s")),
            (query_value!(j.nums.u64), json!(123)),
            (query_value!(j.nums.i64), json!(-123)),
            (query_value!(j.nums.f64), json!(1.23)),
            (query_value!(j.bool), json!(true)),
            (query_value!(j.null), json!(null)),
            (
                query_value!(j.obj),
                json!({"inner": "value", "more": "item"}),
            ),
            (query_value!(j.obj.inner), json!("value")),
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

        let tests = [
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
        let tests = [
            (query_value!(j.arr[0]), json!("first")),
            (query_value!(j.arr[1]), json!(42)),
            (query_value!(j.arr[2].hidden), json!("tale")), // more complex query!
            (query_value!(j.arr[3][0]), json!(0)),          // successive indexing
        ];

        test_is_some_of_expected_val!(tests);
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
            Some(&json!({"inner": "just woke up!", "more": "item"}))
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
    fn test_query_and_convert() {
        let j = make_sample_json();

        let tests = [
            query_value!(j.str -> str) == Some("s"),
            query_value!(j.nums.u64 -> u64) == Some(123),
            query_value!(j.nums.i64 -> i64) == Some(-123),
            query_value!(j.nums.f64 -> f64) == Some(1.23),
            query_value!(j.bool -> bool) == Some(true),
            query_value!(j.null -> null) == Some(()),
            query_value!(j.obj -> object).unwrap().get("inner").unwrap() == "value",
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

        let j = make_sample_json();

        let tests = [
            query_value!(j.str >> (String)) == Some("s".into()),
            query_value!(j.str >> (std::string::String)) == Some("s".into()),
            query_value!(j.str >> String) == Some("s".into()), // parens around type name can be omitted if single identifier
            query_value!(j.nums.u64 >> u8) == Some(123u8),
            query_value!(j.nums.i64 >> i8) == Some(-123i8),
            query_value!(j.nums.i64 >> u8) == None, // fails since can't deserialize negative value into u8
            query_value!(j.nums.f64 >> f32) == Some(1.23f32),
            query_value!(j.null >> (())) == Some(()),
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
            query_value!(j.author >> Person),
            Some(Person {
                name: "jiftechnify".into(),
                age: 31u8,
            }),
        );
    }

    #[test]
    fn test_query_with_unwrapping() {
        let j = make_sample_json();

        let default_str = &json!("default");

        // basic query with ??
        assert_eq!(query_value!(j.str ?? default_str), &json!("s"));
        assert_eq!(query_value!(j.unknown ?? default_str), &json!("default"));

        // `?? default`
        assert_eq!(query_value!(j.nums.u64 -> u64 ?? default), 123u64);
        assert_eq!(query_value!(j.unknown -> u64 ?? default), 0u64); // u64::default()
        assert_eq!(query_value!(j.unknown -> str ?? default), ""); // &str::default()

        // with conversion (->)
        assert_eq!(query_value!(j.str -> str ?? "default"), "s");
        assert_eq!(query_value!(j.nums.u64 -> u64 ?? 999), 123);
        assert_eq!(
            query_value!(j.nums.u64 -> str ?? "not a string"),
            "not a string"
        ); // type mismatch
        assert_eq!(query_value!(j.unknown -> str ?? "default"), "default");
        assert_eq!(query_value!(j.unknown -> str ?? default), ""); // &str::default()

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

        assert_eq!(query_value!(j.nums >> Nums ?? default), expected_nums);
        assert_eq!(
            query_value!(j.unknown >> Nums ?? Nums { u64: 999, i64: -999, f64: 9.99 }),
            Nums {
                u64: 999,
                i64: -999,
                f64: 9.99
            }
        );
        assert_eq!(
            query_value!(j.unknown >> Nums ?? default),
            Nums {
                u64: 0,
                i64: 0,
                f64: 0.0,
            }
        );

        assert_eq!(
            query_value!(j.num_arr >> (Vec<u8>) ?? default),
            vec![0, 1, 2]
        );
        assert_eq!(
            query_value!(j.arr >> (Vec<u8>) ?? default),
            Vec::<u8>::new()
        );
        assert_eq!(query_value!(j.arr >> (Vec<u8>) ?? vec![42]), vec![42]);
    }

    #[test]
    fn test_deserialize_into_vec() {
        use serde::Deserialize;

        let j = make_sample_json();

        let tests = [
            query_value!(j.num_arr >> (Vec<u8>)) == Some(vec![0, 1, 2]),
            query_value!(j.num_arr >> (Vec<u8>) ?? default) == vec![0, 1, 2],
            query_value!(j.arr >> (Vec<u8>)) == None,
            query_value!(j.arr >> (Vec<u8>) ?? default) == Vec::<u8>::new(),
            query_value!(j.arr >> (Vec<u8>) ?? vec![42]) == vec![42],
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
            query_value!(j.obj >> (HashMap<String, Value>)) == Some(exp_json),
            query_value!(j.obj >> (HashMap<String, String>)) == Some(exp_string),
            query_value!(j.obj >> (HashMap<String, u8>)) == None,
        ];
        test_all_true_or_failed_idx!(tests);
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

    #[test]
    fn test_query_with_dynamic_indices() {
        let j = make_sample_json();

        // Dynamic string key
        let key = "str";
        assert_eq!(query_value!(j[key]), Some(&json!("s")));

        let obj_key = "obj";
        let inner_key = "inner";
        assert_eq!(query_value!(j[obj_key][inner_key]), Some(&json!("value")));

        // Dynamic integer index
        let index = 0;
        assert_eq!(query_value!(j.arr[index]), Some(&json!("first")));

        let arr_index = 1;
        assert_eq!(query_value!(j.arr[arr_index]), Some(&json!(42)));

        // Mix of static and dynamic
        let key2 = "nums";
        assert_eq!(query_value!(j[key2].u64), Some(&json!(123)));

        // Dynamic expression
        let base_index = 1;
        assert_eq!(
            query_value!(j.arr[base_index + 1].hidden),
            Some(&json!("tale"))
        );

        // With conversion
        assert_eq!(query_value!(j[key] -> str), Some("s"));
        assert_eq!(query_value!(j.arr[index] -> str), Some("first"));

        // With unwrapping operator
        let missing_key = "missing";
        let fallback = json!("fallback");
        assert_eq!(
            query_value!(j[missing_key]?? & fallback),
            &json!("fallback")
        );

        let out_of_bounds = 999;
        let oob_val = json!("oob");
        assert_eq!(
            query_value!(j.arr[out_of_bounds]?? & oob_val),
            &json!("oob")
        );
    }

    #[test]
    fn test_query_with_dynamic_indices_mut() {
        let mut j = make_sample_json();

        // Dynamic string key (mut)
        let key = "str";
        {
            let val = query_value!(mut j[key]).unwrap();
            *val = json!("modified");
        }
        assert_eq!(query_value!(j.str), Some(&json!("modified")));

        // Dynamic integer index (mut)
        let index = 1;
        {
            let val = query_value!(mut j.arr[index]).unwrap();
            *val = json!(100);
        }
        assert_eq!(query_value!(j.arr[1]), Some(&json!(100)));

        // With conversion (mut)
        let obj_key = "obj";
        let dynamic_key = "dynamic_key";
        {
            let obj = query_value!(mut j[obj_key] -> object).unwrap();
            obj.insert("dynamic_key".to_string(), json!("added"));
        }
        assert_eq!(query_value!(j.obj[dynamic_key]), Some(&json!("added")));
    }

    #[test]
    fn test_query_complex_expressions() {
        use serde::Deserialize;

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
            query_value!((json!({ "x": 1 })).x) == Some(&json!(1)),
            query_value!((json!({ "x": 1 })).y ?? default) == &json!(null),
            query_value!((json!({ "x": 1 })).x -> u64) == Some(1u64),
            query_value!((json!({ "x": 1 })).x -> str ?? default) == "",
            query_value!((json!({ "x": 1 })).x -> str ?? "not str") == "not str",
            query_value!((json!({ "x": 1 })).x >> u8) == Some(1u8),
            query_value!((json!({ "x": 1 })).x >> String ?? default) == "".to_string(),
            query_value!((json!({ "x": 1 })).x >> String ?? "deser failed".to_string())
                == "deser failed".to_string(),
            // querying immediate value (mut)
            query_value!(mut (json!({ "x": 1 })).x) == Some(&mut json!(1)),
            query_value!(mut (json!({ "x": 1 })).x >> u8) == Some(1u8),
            query_value!(mut (json!({ "x": 1 })).x >> String ?? "deser failed".to_string())
                == "deser failed".to_string(),
            // querying return value of function
            query_value!((gen_value()).x) == Some(&json!(1)),
            query_value!((gen_value()).x -> u64) == Some(1u64),
            query_value!((gen_value()).x -> str ?? "not str") == "not str",
            // querying element of tuple
            query_value!((tuple.0).x) == Some(&json!(1)),
            query_value!((tuple.0).x -> u64) == Some(1u64),
            query_value!((tuple.0).x -> str ?? "not str") == "not str",
            // querying field of struct
            query_value!((s.value).x) == Some(&json!(1)),
            query_value!((s.value).x -> u64) == Some(1u64),
            query_value!((s.value).x -> str ?? "not str") == "not str",
            // querying return value of method
            query_value!((s.gen_value()).x) == Some(&json!(1)),
            query_value!((s.gen_value()).x -> u64) == Some(1u64),
            query_value!((s.gen_value()).x -> str ?? "not str") == "not str",
            // querying indexed value
            query_value!((v[0]).x) == Some(&json!(1)),
            query_value!((v[0]).x -> u64) == Some(1u64),
            query_value!((v[0]).x -> str ?? "not str") == "not str",
        ];
        test_all_true_or_failed_idx!(tests);
    }
}

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

        let tests = [
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

        let tests = [
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
            query_value!(t.date -> datetime).unwrap().to_string() == "2021-12-18T12:15:12+09:00",
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
