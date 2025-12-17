/// Transposes an arbitrary length of tuple of `Option`s/`Result`s into an `Option`/`Result` of a tuple.
///
/// Syntax:
///
/// - `Option`-variant: `transpose_tuple!(Option; e1, e2, ..., eN)` ... transposes a tuple of `Option`s.
///     - You can omit `Option;` part, like: `transpose_tuple!(e1, e2, ..., eN)`.
/// - `Result`-variant: `transpose_tuple!(Result; e1, e2, ..., eN)`... transposes a tuple of [`valq::Result`]s.
///
/// Note: `Result`-variant is meant to be used with [`valq::Result`]  (return type of `query_value_result!` macro).
/// It doesn't support other error types for now.
///
/// [`valq::Result`]: crate::Result
///
/// ## Examples
///
/// ```
/// // Options
/// use serde_json::json;
/// use valq::{query_value, transpose_tuple};
///
/// let valq = json!({
///     "name": "valq",
///     "keywords": ["macro", "query", "serde"],
///     "author": {
///         "name": "jiftechnify",
///         "age": 31
///     }
/// });
///
/// let picks = transpose_tuple!(
///     query_value!(valq.name -> str),
///     query_value!(valq.keywords[1] -> str),
///     query_value!(valq.author.age -> u64),
/// );
/// assert_eq!(picks, Some(("valq", "query", 31u64)));
///
/// let wrong_picks = transpose_tuple!(
///     query_value!(valq.name -> str),
///     query_value!(valq.keywords[10] -> str), // out of bounds
/// );
/// assert_eq!(wrong_picks, None);
/// ```
///
/// ```
/// // Results
/// # use serde_json::json;
/// use valq::{query_value_result, transpose_tuple};
/// # let valq = json!({
/// #     "name": "valq",
/// #     "keywords": ["macro", "query", "serde"],
/// #     "author": {
/// #         "name": "jiftechnify",
/// #         "age": 31
/// #     }
/// # });
///
/// let picks = transpose_tuple!(
///     Result; // don't forget this!
///     query_value_result!(valq.name -> str),
///     query_value_result!(valq.keywords[1] -> str),
///     query_value_result!(valq.author.age -> u64),
/// );
/// assert!(matches!(picks, Ok(("valq", "query", 31u64))));
///
/// let wrong_picks = transpose_tuple!(
///     Result;
///     query_value_result!(valq.name -> str),
///     query_value_result!(valq.secrets), // non-existent path
/// );
/// assert!(matches!(wrong_picks, Err(valq::Error::ValueNotFoundAtPath(_))));
/// ```
#[macro_export]
macro_rules! transpose_tuple {
    (Option; $first:expr, $($rest:expr),+ $(,)?) => {
        (|| {
            Some(( $first?, $($rest?),+ ))
        })()
    };
    (Result; $first:expr, $($rest:expr),+ $(,)?) => {
        (|| {
            Ok(( $first?, $($rest?),+ )) as $crate::Result<_>
        })()
    };
    ($first:expr, $($rest:expr),+ $(,)?) => {
        transpose_tuple!(Option; $first, $($rest),+)
    };
}
