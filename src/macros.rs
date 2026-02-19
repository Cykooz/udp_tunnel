#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! debug {
    ($x: literal) => {{
        let _ = format!($x);
    }};
}

#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! info {
    ($x: literal) => {
        println!($x)
    };
}

#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! error {
    ($x: literal) => {
        println!($x)
    };
}
