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
        $query_value
    };
}

// fake implementation illustrates the macro syntax for docs
#[cfg(doc)]
doc! {macro_rules! query_value {
    ($(mut)? $value:tt $(query:tt)*) => {};
    ($(mut)? $value:tt $(query:tt)* -> $as:ident) => {};
    ($(mut)? $value:tt $(query:tt)* >> $deser_to:ty) => {};
}}

// actual implementation
#[cfg(not(doc))]
doc! {macro_rules! query_value {
    /* non-mut traversal */
    (@trv { $vopt:expr }) => {
        $vopt
    };
    (@trv { $vopt:expr } -> $dest:ident) => {
        $crate::__paste! {
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
        $crate::__paste! {
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
}}
