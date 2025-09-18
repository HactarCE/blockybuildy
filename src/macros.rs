/// Logging with indentation and verbosity filtering.
macro_rules! log {
    ($params:expr) => {
        log!($params, 0);
    };
    ($params:expr, $indent:expr $(, $($msg:tt)*)?) => {
        if $params.verbosity >= $indent + 1 {
            for _ in 0..$indent {
                print!("  ");
            }
            println!($($($msg)*)?);
        }
    };
}
