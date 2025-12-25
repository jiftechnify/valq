use serde::Deserialize;
use serde_json::json;
use valq::{query_value, query_value_result};

fn main() {
    let data = json!({
        "package": {
            "name": "valq",
            "version": "0.3.0",
            "authors": ["jiftechnify"],
            "description": "macros for querying semi-structured data with the JavaScript-like syntax",
            "keywords": ["macro", "query", "json"]
        },
        "dependencies": {
            "paste": { "version": "1.0.15" }
        },
        "dev-dependencies": {
            "serde": {
                "version": "1.0.228",
                "features": ["derive"]
            }
        }
    });

    assert_eq!(query_value!(data.package.name -> str).unwrap(), "valq");

    assert_eq!(
        query_value!(data.package.keywords >> (Vec<String>)).unwrap(),
        ["macro", "query", "json"],
    );

    let res: valq::Result<&str> = query_value_result!(data.package.readme -> str);
    if let Err(valq::Error::ValueNotFoundAtPath(path)) = res {
        assert_eq!(path, ".package.readme")
    }
    else {
        unreachable!()
    }

    assert_eq!(
        query_value!(data.package.readme -> str ?? "README.md"),
        "README.md",
    );

    let dep_name = "paste";
    assert_eq!(
        query_value!(data.dependencies[dep_name].version -> str).unwrap(),
        "1.0.15",
    );

    assert_eq!(
        query_value!(data["dev-dependencies"]["serde"].features[0] >> String ?? "none".into()),
        "derive".to_string(),
    );
}
