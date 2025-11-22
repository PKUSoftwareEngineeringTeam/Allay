use std::process::exit;

/// Abort the program with an error message without panicking.
pub fn show_error(msg: &str) -> ! {
    eprintln!("Error: {}", msg);
    exit(1)
}

pub trait NoPanicUnwrap<T, E> {
    fn expect_(self, msg: &str) -> T
    where
        Self: Sized,
    {
        self.expect_on(|_| msg.to_string())
    }

    fn expect_on(self, f: impl FnOnce(E) -> String) -> T;
}

impl<T> NoPanicUnwrap<T, ()> for Option<T> {
    fn expect_on(self, f: impl FnOnce(()) -> String) -> T {
        self.unwrap_or_else(|| show_error(&f(())))
    }
}

impl<T, E> NoPanicUnwrap<T, E> for Result<T, E> {
    fn expect_on(self, f: impl FnOnce(E) -> String) -> T {
        self.unwrap_or_else(|e| show_error(&f(e)))
    }
}
