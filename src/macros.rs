/// Logging with indentation and verbosity filtering.
macro_rules! log {
    ($params:expr) => {
        log!($params, 0);
    };
    ($params:expr, $indent:expr $(, $($msg:tt)*)?) => {
        if $params.verbosity > $indent {
            #[allow(clippy::reversed_empty_ranges)]
            for _ in 0..$indent {
                print!("  ");
            }
            println!($($($msg)*)?);
        }
    };
}
