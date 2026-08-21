// The literal rule must stay first. An ident matcher would otherwise capture a
// quoted key and stringify it with its quotes intact.
#[macro_export]
#[doc(hidden)]
macro_rules! field_key {
    ($key:literal) => {
        $key
    };
    ($key:ident) => {
        stringify!($key)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __emit {
    ($level:expr, $msg:expr, $fields:expr) => {
        $crate::Logger::emit($crate::LogEvent {
            level: $level,
            target: ::core::module_path!().to_string(),
            message: ::std::string::ToString::to_string(&$msg),
            fields: $fields,
            timestamp: $crate::__private::chrono::Utc::now(),
            file: ::core::option::Option::Some(::core::file!().to_string()),
            line: ::core::option::Option::Some(::core::line!()),
        })
    };
}

// The block rule must precede the expression rule, or `{}` and `{ k: v }` are
// both swallowed as block expressions.
#[macro_export]
#[doc(hidden)]
macro_rules! __log {
    ($level:expr, $msg:expr) => {
        $crate::__emit!($level, $msg, $crate::Fields::new())
    };
    ($level:expr, $msg:expr, { $($key:tt : $value:expr),* $(,)? }) => {{
        let mut fields = $crate::Fields::new();
        $(
            fields.insert($crate::field_key!($key), $value);
        )*
        $crate::__emit!($level, $msg, fields)
    }};
    ($level:expr, $msg:expr, $fields:expr) => {
        $crate::__emit!($level, $msg, $crate::Fields::from_serializable($fields))
    };
}

#[macro_export]
macro_rules! error {
    ($($args:tt)*) => { $crate::__log!($crate::Level::Error, $($args)*) };
}

#[macro_export]
macro_rules! warn {
    ($($args:tt)*) => { $crate::__log!($crate::Level::Warn, $($args)*) };
}

#[macro_export]
macro_rules! info {
    ($($args:tt)*) => { $crate::__log!($crate::Level::Info, $($args)*) };
}

#[macro_export]
macro_rules! debug {
    ($($args:tt)*) => { $crate::__log!($crate::Level::Debug, $($args)*) };
}

#[macro_export]
macro_rules! trace {
    ($($args:tt)*) => { $crate::__log!($crate::Level::Trace, $($args)*) };
}
