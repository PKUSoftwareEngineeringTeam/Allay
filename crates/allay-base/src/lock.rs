#[macro_export]
macro_rules! read {
    ($lock: expr) => {{
        use std::sync::PoisonError;
        $lock.read().unwrap_or_else(PoisonError::into_inner)
    }};
}

#[macro_export]
macro_rules! write {
    ($lock: expr) => {{
        use std::sync::PoisonError;
        $lock.write().unwrap_or_else(PoisonError::into_inner)
    }};
}

#[macro_export]
macro_rules! lock {
    ($lock: expr) => {{
        use std::sync::PoisonError;
        $lock.lock().unwrap_or_else(PoisonError::into_inner)
    }};
}
