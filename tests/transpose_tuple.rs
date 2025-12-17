use serde_json::json;
use valq::{query_value, query_value_result, transpose_tuple, Error};

#[test]
fn test_option_variant_all_some() {
    let data = json!({
        "name": "valq",
        "version": "0.2.0",
        "stars": 100
    });

    let result = transpose_tuple!(
        query_value!(data.name -> str),
        query_value!(data.version -> str),
        query_value!(data.stars -> u64),
    );

    assert_eq!(result, Some(("valq", "0.2.0", 100u64)));
}

#[test]
fn test_option_variant_with_none() {
    let data = json!({
        "name": "valq",
        "version": "0.2.0",
    });

    let result = transpose_tuple!(
        query_value!(data.name -> str),
        query_value!(data.nonexistent -> str),
        query_value!(data.version -> str),
    );

    assert_eq!(result, None);
}

#[test]
fn test_option_variant_explicit_option_prefix() {
    let data = json!({
        "a": 1,
        "b": 2,
        "c": 3
    });

    let result = transpose_tuple!(
        Option;
        query_value!(data.a -> u64),
        query_value!(data.b -> u64),
        query_value!(data.c -> u64),
    );

    assert_eq!(result, Some((1u64, 2u64, 3u64)));
}

#[test]
fn test_option_variant_many_elements() {
    let data = json!({
        "v1": 1,
        "v2": 2,
        "v3": 3,
        "v4": 4,
        "v5": 5,
        "v6": 6
    });

    let result = transpose_tuple!(
        query_value!(data.v1 -> u64),
        query_value!(data.v2 -> u64),
        query_value!(data.v3 -> u64),
        query_value!(data.v4 -> u64),
        query_value!(data.v5 -> u64),
        query_value!(data.v6 -> u64),
    );

    assert_eq!(result, Some((1u64, 2u64, 3u64, 4u64, 5u64, 6u64)));
}

#[test]
fn test_option_variant_trailing_comma() {
    let data = json!({
        "a": "foo",
        "b": "bar",
    });

    let result = transpose_tuple!(query_value!(data.a -> str), query_value!(data.b -> str),);

    assert_eq!(result, Some(("foo", "bar")));
}

#[test]
fn test_result_variant_all_ok() {
    let data = json!({
        "name": "valq",
        "keywords": ["macro", "query", "serde"],
        "author": {
            "name": "jiftechnify",
            "age": 31
        }
    });

    let result = transpose_tuple!(
        Result;
        query_value_result!(data.name -> str),
        query_value_result!(data.keywords[1] -> str),
        query_value_result!(data.author.age -> u64),
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ("valq", "query", 31u64));
}

#[test]
fn test_result_variant_with_error() {
    let data = json!({
        "name": "valq",
        "version": "0.2.0"
    });

    let result = transpose_tuple!(
        Result;
        query_value_result!(data.name -> str),
        query_value_result!(data.nonexistent),
    );

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::ValueNotFoundAtPath(_)));
}

#[test]
fn test_result_variant_type_mismatch_error() {
    let data = json!({
        "value": "not a number",
        "other": 123
    });

    let result = transpose_tuple!(
        Result;
        query_value_result!(data.value -> u64),
        query_value_result!(data.other -> u64),
    );

    assert!(result.is_err());
}

#[test]
fn test_result_variant_array_out_of_bounds() {
    let data = json!({
        "items": [1, 2, 3]
    });

    let result = transpose_tuple!(
        Result;
        query_value_result!(data.items[0] -> u64),
        query_value_result!(data.items[10] -> u64),
    );

    assert!(result.is_err());
}

#[test]
fn test_result_variant_trailing_comma() {
    let data = json!({
        "a": 1.5,
        "b": 2.5,
    });

    let result = transpose_tuple!(
        Result;
        query_value_result!(data.a -> f64),
        query_value_result!(data.b -> f64),
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), (1.5f64, 2.5f64));
}

#[test]
fn test_mixed_types() {
    let data = json!({
        "string": "hello",
        "number": 42,
        "boolean": true,
        "float": 3.14
    });

    let result = transpose_tuple!(
        query_value!(data.string -> str),
        query_value!(data.number -> u64),
        query_value!(data.boolean -> bool),
        query_value!(data.float -> f64),
    );

    assert_eq!(result, Some(("hello", 42u64, true, 3.14f64)));
}

#[test]
fn test_nested_object_access() {
    let data = json!({
        "level1": {
            "level2": {
                "level3": {
                    "value": "deep",
                    "number": 99
                }
            }
        }
    });

    let result = transpose_tuple!(
        Result;
        query_value_result!(data.level1.level2.level3.value -> str),
        query_value_result!(data.level1.level2.level3.number -> u64),
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ("deep", 99u64));
}
