use std::time::SystemTime;

pub trait Time {
    fn now() -> Self;
}

impl Time for SystemTime {
    fn now() -> Self { Self::now() }
}
