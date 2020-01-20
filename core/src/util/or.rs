#[macro_export]
macro_rules! or_break {
    ($e:expr) => {
        match $e {
            Some(x) => x,
            None => break,
        }
    };
}

#[macro_export]
macro_rules! or_return {
    ($e:expr, $r:expr) => {
        match $e {
            Some(x) => x,
            None => return $r,
        }
    };
}
