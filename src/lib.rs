#[macro_export]
macro_rules! query_value {
    /* non-mut traversal */
    (@trv ($to:ident) $v:tt . $prop:ident $($rest:tt)*) => {
        $v.get(stringify!($prop)).and_then(|v| query_value!(@trv ($to) v $($rest)*))
    };
    (@trv ($to:ident) $v:tt . $prop:literal $($rest:tt)*) => {
        $v.get($prop as &str).and_then(|v| query_value!(v $($rest)*))
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
    (@conv (obj) $v:tt) => {
        $v.as_object()
    };
    (@conv (arr) $v:tt) => {
        $v.as_arr()
    };
    // for serde_yaml::Value
    (@conv (map) $v:tt) => {
        $v.as_mapping()
    };
    (@conv (seq) $v:tt) => {
        $v.as_sequence()
    };
    (@conv ($to:ident) $v:tt) => {
        compile_error!(concat!("unsupported target type `", stringify!($to), "` is specified in query_value!()"))
    };

    /* mut traversal */
    (@trv_mut ($to:ident) $v:tt . $prop:ident $($rest:tt)*) => {
        $v.get_mut(stringify!($prop)).and_then(|v| query_value!(@trv_mut ($to) v $($rest)*))
    };
    (@trv_mut ($to:ident) $v:tt . $prop:literal $($rest:tt)*) => {
        $v.get_mut($prop as &str).and_then(|v| query_value!(v $($rest)*))
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
    (@conv_mut (obj) $v:tt) => {
        $v.as_object_mut()
    };
    (@conv_mut (arr) $v:tt) => {
        $v.as_array_mut()
    };
    // for serde_yaml::Value
    (@conv_mut (map) $v:tt) => {
        $v.as_mapping_mut()
    };
    (@conv_mut (seq) $v:tt) => {
        $v.as_sequence_mut()
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
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
