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
pub use paste::paste as __paste;

macro_rules! doc {
    ($query_value:item) => {
        /// A macro for querying an inner value of a structured ("JSON-ish") data.
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
        /// ## Query Notations
        ///
        /// You can traverse nested data using JavaScript-ish accessor notaions:
        ///
        /// - **Dot notation** (`.field`): Access a property of an "object" (key-value structure) by name
        /// - **Bracket notation** (`["field"]`): Access a property of an "object", or an element of an "array"-like value by index
        ///     - With string index, you get access to object properties, similar to dot notation.
        ///       This is especially useful for keys that are not valid Rust identifiers (e.g. `"1st"`, `"foo-bar"`).
        ///     - With integer index, you get access to array elements.
        ///     - Dynamic query: you can place a Rust expression that evaluate to string or integer in the brackets.
        ///
        /// Of course, you can chain these accessors arbitrarily to navigate through complex structures.
        ///
        /// ## Query Result
        ///
        /// `query_value!` returns an `Option` as the result of the query (except when unwrapping operator `??` is used; see below for details).
        ///
        /// Queries can fail for the following reasons. In that case, `query_value!` returns `None`:
        ///
        /// - The specified key or index does not exist in the target value
        /// - Indexing an object (key-value structure) with an integer
        /// - Indexing an array with a string key
        ///
        /// Otherwise, i.e. if your query succeeds, `query_value!` returns the queried value wrapped in `Some`.
        ///
        /// With basic queries, `query_value!` extracts a shared reference (`&`) to the inner value by default. Think of it as a function that has following signature:
        ///
        /// ```txt
        /// query_value!(query...) -> Option(&Value)
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
        /// ## `??`: Unwarp Query Result with Default Value
        ///
        /// You put `?? ...` at the end of the query to unwrap the query result with providing a default value in case of query failure.
        ///
        /// - `?? <expr>`: Use the value of`<expr>` as the default.
        /// - `?? default`: Use `Default::default()` as the default.
        ///
        /// This is especilly useful together with `->` or `>>` conversions:
        ///
        /// ```
        /// use serde_json::{json, Value};
        /// use valq::query_value;
        ///
        /// let obj = json!({"foo": {"bar": "not a number"}});
        /// assert_eq!(query_value!(obj.foo.bar -> str ?? "failed!"), "not a number");
        /// assert_eq!(query_value!(obj.foo.bar -> u64 ?? 42), 42);
        /// assert_eq!(query_value!(obj.foo.bar -> u64 ?? default), 0u64); // u64::default()
        /// ```
        ///
        /// ## Query Syntax Specification
        ///
        /// ```txt
        /// query_value!(
        ///     ("mut")?
        ///     <value> ("." <key> | "[" <idx> "]")*
        ///     ("->" <as_dest> | ">>" <deser_dest>)?
        ///     ("??" ("default" | <default_expr>))?
        /// )
        /// ```
        ///
        /// where:
        ///
        /// - `<value>`: An expression evaluates to a structured data to be queried
        /// - `<key>`: A property/field key to extract value from a key-value structure
        /// - `<idx>`: An index to extract value from structure
        ///     + For an array-like structure, any expressions evaluates to an integer can be used
        ///     + For a key-value structure, any expressions evaluates to a string can be used
        /// - `<as_dest>`: A destination type of conversion with `as_***()` / `as_***_mut()` methods
        /// - `<deser_dest>`: A type name into which the queried value is deserialized
        ///     + The specified type *MUST* implement the `serde::Deserialize` trait
        /// - `<default_expr>`: An expression for a default value in case of query failure
        ///     + Instead, you can put `default` keyword in this place to use `Default::default()` as the default value
        /// ## Compatibility
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
        $query_value
    };
}

// fake implementation illustrates the macro syntax for docs
#[cfg(doc)]
doc! {macro_rules! query_value {
    ($(mut)? $value:tt $(query:tt)* $(?? $default:expr)?) => {};
    ($(mut)? $value:tt $(query:tt)* -> $as:ident $(?? $default:expr)?) => {};
    ($(mut)? $value:tt $(query:tt)* >> $deser_to:ident $(?? $default:expr)?) => {};
}}

// actual implementation
#[cfg(not(doc))]
doc! {macro_rules! query_value {
    /* non-mut traversal */
    // traversal step
    (@trv { $vopt:expr } . $key:ident $($rest:tt)*) => {
        query_value!(@trv { $vopt.and_then(|v| v.get(stringify!($key))) } $($rest)*)
    };
    (@trv { $vopt:expr } [ $idx:expr ] $($rest:tt)*) => {
        query_value!(@trv { $vopt.and_then(|v| v.get($idx)) } $($rest)*)
    };
    // conversion step -> convert then jump to finalization step
    (@trv { $vopt:expr } -> $dest:ident $($rest:tt)*) => {
        $crate::__paste! {
            query_value!(@fin { $vopt.and_then(|v| v.[<as_ $dest>]()) } $($rest)*)
        }
    };
    (@trv { $vopt:expr } >> $dest:ident $($rest:tt)*) => {
        query_value!(@fin { $vopt.and_then(|v| <$dest>::deserialize(v.clone()).ok()) } $($rest)*)
    };
    // no conversion -> just jump to finalization step
    (@trv { $vopt:expr } $($rest:tt)*) => {
        query_value!(@fin { $vopt } $($rest)*)
    };

    /* mut traversal */
    // traversal step
    (@trv_mut { $vopt:expr } . $key:ident $($rest:tt)*) => {
        query_value!(@trv_mut { $vopt.and_then(|v| v.get_mut(stringify!($key))) } $($rest)*)
    };
    (@trv_mut { $vopt:expr } [ $idx:expr ] $($rest:tt)*) => {
        query_value!(@trv_mut { $vopt.and_then(|v| v.get_mut($idx)) } $($rest)*)
    };
    // conversion step -> convert then jump to finalization step
    (@trv_mut { $vopt:expr } -> $dest:ident $($rest:tt)*) => {
        $crate::__paste! {
            query_value!(@fin { $vopt.and_then(|v| v.[<as_ $dest _mut>]()) } $($rest)*)
        }
    };
    (@trv_mut { $vopt:expr } >> $dest:ident $($rest:tt)*) => {
        query_value!(@fin { $vopt.and_then(|v| <$dest>::deserialize(v.clone()).ok()) } $($rest)*)
    };
    // no conversion -> just jump to finalization step
    (@trv_mut { $vopt:expr } $($rest:tt)*) => {
        query_value!(@fin { $vopt } $($rest)*)
    };

    /* finalize: handle terminal operators */
    // ??: "null coalescing"
    (@fin { $vopt:expr } ?? default) => {
        $vopt.unwrap_or_default()
    };
    (@fin { $vopt:expr } ?? $default:expr) => {
        $vopt.unwrap_or_else(|| $default)
    };
    // no terminal operator
    (@fin { $vopt:expr }) => {
        $vopt
    };
    // unreachable branch -> report syntax error
    (@fin $($_:tt)*) => {
        compile_error!("invalid query syntax for query_value!()")
    };

    /* entry points */
    (mut $v:tt $($rest:tt)*) => {
        query_value!(@trv_mut { Some(&mut $v) } $($rest)*)
    };
    ($v:tt $($rest:tt)*) => {
        query_value!(@trv { Some(&$v) } $($rest)*)
    };
}}
