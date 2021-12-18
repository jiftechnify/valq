#[macro_export]
macro_rules! query_value {
    /* non-mut traversal */
    (@trv ($to:ident) $v:tt . $prop:ident $($rest:tt)*) => {
        $v.get(stringify!($prop)).and_then(|v| query_value!(@trv ($to) v $($rest)*))
    };
    (@trv ($to:ident) $v:tt . $prop:literal $($rest:tt)*) => {
        $v.get($prop as &str).and_then(|v| query_value!(@trv ($to) v $($rest)*))
    };
    (@trv ($to:ident) $v:tt [ $idx:expr ] $($rest:tt)*) => {
        $v.get($idx as usize).and_then(|v| query_value!(@trv ($to) v $($rest)*))
    };
    (@trv ($to:ident) $v:tt) => {
        query_value!(@conv ($to) $v)
    };
    (@trv $($_:tt)*) => {
        compile_error!("invalid query syntax for query_value!()")
    };
    /* non-mut conversion */
    (@conv (val) $v:tt) => {
        Some($v)
    };
    (@conv (str) $v:tt) => {
        $v.as_str()
    };
    (@conv (u64) $v:tt) => {
        $v.as_u64()
    };
    (@conv (i64) $v:tt) => {
        $v.as_i64()
    };
    (@conv (f64) $v:tt) => {
        $v.as_f64()
    };
    (@conv (bool) $v:tt) => {
        $v.as_bool()
    };
    (@conv (null) $v:tt) => {
        $v.as_null()
    };
    (@conv (object) $v:tt) => {
        $v.as_object()
    };
    (@conv (array) $v:tt) => {
        $v.as_array()
    };
    // for serde_yaml::Value
    (@conv (mapping) $v:tt) => {
        $v.as_mapping()
    };
    (@conv (sequence) $v:tt) => {
        $v.as_sequence()
    };
    // for toml::Value
    (@conv (datetime) $v:tt) => {
        $v.as_datetime()
    };
    (@conv (table) $v:tt) => {
        $v.as_table()
    };
    (@conv ($to:ident) $v:tt) => {
        compile_error!(concat!("unsupported target type `", stringify!($to), "` is specified in query_value!()"))
    };

    /* mut traversal */
    (@trv_mut ($to:ident) $v:tt . $prop:ident $($rest:tt)*) => {
        $v.get_mut(stringify!($prop)).and_then(|v| query_value!(@trv_mut ($to) v $($rest)*))
    };
    (@trv_mut ($to:ident) $v:tt . $prop:literal $($rest:tt)*) => {
        $v.get_mut($prop as &str).and_then(|v| query_value!(@trv_mut ($to) v $($rest)*))
    };
    (@trv_mut ($to:ident) $v:tt [ $idx:expr ] $($rest:tt)*) => {
        $v.get_mut($idx as usize).and_then(|v| query_value!(@trv_mut ($to) v $($rest)*))
    };
    (@trv_mut ($to:ident) $v:tt) => {
        query_value!(@conv_mut ($to) $v)
    };
    (@trv_mut $($_:tt)*) => {
        compile_error!("invalid query syntax for query_value!()")
    };
    /* mut conversion */
    (@conv_mut (val) $v:tt) => {
        Some($v)
    };
    (@conv_mut (object) $v:tt) => {
        $v.as_object_mut()
    };
    (@conv_mut (array) $v:tt) => {
        $v.as_array_mut()
    };
    // for serde_yaml::Value
    (@conv_mut (mapping) $v:tt) => {
        $v.as_mapping_mut()
    };
    (@conv_mut (sequence) $v:tt) => {
        $v.as_sequence_mut()
    };
    // for toml::Value
    (@conv_mut (table) $v:tt) => {
        $v.as_table_mut()
    };
    (@conv_mut ($to:ident) $v:tt) => {
        compile_error!(concat!("unsupported target type `", stringify!($to), "` is specified in query_value!()"))
    };

    /* starting point */
    (mut ($v:tt $($path:tt)+) -> $to:ident) => {
        query_value!(@trv_mut ($to) $v $($path)+)
    };
    (mut $v:tt $($path:tt)+) => {
        query_value!(mut ($v $($path)+) -> val)
    };
    (($v:tt $($path:tt)+) -> $to:ident) => {
        query_value!(@trv ($to) $v $($path)+)
    };
    ($v:tt $($path:tt)+) => {
        query_value!(($v $($path)+) -> val)
    };
}

#[cfg(test)]
mod tests_json {
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
                { "hidden": "tale" }
            ],
            "1st": "prop starts with digit!"
        })
    }

    #[test]
    fn test_query_json() {
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
                json!(["first", 42, {"hidden": "tale"}]),
            ),
            (query_value!(j."1st"), json!("prop starts with digit!")),
        ];
        for (res, exp) in tests {
            if let Some(act) = res {
                assert_eq!(act, &exp)
            } else {
                panic!("result must be Some(...)")
            }
        }
    }

    #[test]
    fn test_indexing_json_array() {
        let j = make_sample_json();

        let tests = vec![
            (query_value!(j.arr[0]), json!("first")),
            (query_value!(j.arr[1]), json!(42)),
            (query_value!(j.arr[2].hidden), json!("tale")), // more complex query!
        ];
        for (res, exp) in tests {
            if let Some(act) = res {
                assert_eq!(act, &exp)
            } else {
                panic!("result must be Some(...)")
            }
        }
    }

    #[test]
    fn test_query_and_convert_json() {
        let j = make_sample_json();

        let tests = vec![
            query_value!((j.str) -> str) == Some("s"),
            query_value!((j.nums.u64) -> u64) == Some(123),
            query_value!((j.nums.i64) -> i64) == Some(-123),
            query_value!((j.nums.f64) -> f64) == Some(1.23),
            query_value!((j.bool) -> bool) == Some(true),
            query_value!((j.null) -> null) == Some(()),
            query_value!((j.obj) -> object)
                .unwrap()
                .get("inner")
                .unwrap()
                == "zzz",
            query_value!((j.arr) -> array).unwrap()
                == &vec![json!("first"), json!(42), json!({"hidden": "tale"})],
        ];

        if tests.iter().any(|&r| !r) {
            panic!("some test failed")
        }
    }

    #[test]
    fn test_query_json_mut() {
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
            let obj = query_value!(mut (j.obj) -> object).unwrap();
            obj.insert("new_prop".to_string(), json!("yeah"));
        }
        assert_eq!(query_value!((j.obj.new_prop) -> str), Some("yeah"));

        // get inner array as Vec, then append new value via push()
        {
            let arr = query_value!(mut (j.arr) -> array).unwrap();
            arr.push(json!("appended!"));
        }
        assert_eq!(query_value!((j.arr[3]) -> str), Some("appended!"));
    }

    #[test]
    fn test_query_json_fail() {
        let j = make_sample_json();

        let tests = vec![
            query_value!(j.unknown),   // non existent property
            query_value!(j.nums.i128), // non existent property of nested object
            query_value!(j.obj[0]),    // indexing against non-array value
            query_value!(j.arr[100]),  // indexing out of bound
        ];

        for res in tests {
            if res.is_some() {
                panic!("result is Some(...) unexpectedly")
            }
        }
    }

    #[test]
    fn test_query_json_fail_mut() {
        let mut j = make_sample_json();

        let tests_mut = vec![
            { query_value!(mut j.unknown).is_none() },
            { query_value!(mut j.nums.i128).is_none() },
            { query_value!(mut j.obj[0]).is_none() },
            { query_value!(mut j.arr[100]).is_none() },
        ];
        if tests_mut.iter().any(|&r| !r) {
            panic!("result is Some(...) unexpectedly")
        }
    }
}
