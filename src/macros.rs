//! Place to dump any macros that might come in handy.

#[macro_export]
#[cfg(target_family = "wasm")]
macro_rules! tweak {
    ($e:expr) => {
        $e
    };
}

#[macro_export]
#[cfg(not(target_family = "wasm"))]
macro_rules! tweak {
    ($e:expr) => {
        ::inline_tweak::tweak!($e)
    };
}
