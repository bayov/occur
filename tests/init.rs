#[ctor::ctor]
fn init_test() { color_backtrace::install(); }
