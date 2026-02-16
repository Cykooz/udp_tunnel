#[cfg(not(feature = "tracing"))]
macro_rules! debug {
    ($x: literal) => {
        println!($x)
    };
}

#[cfg(not(feature = "tracing"))]
macro_rules! info {
    ($x: literal) => {
        println!($x)
    };
}

#[cfg(not(feature = "tracing"))]
macro_rules! error {
    ($x: literal) => {
        println!($x)
    };
}
