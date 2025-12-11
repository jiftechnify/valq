use valq::query_value;

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
