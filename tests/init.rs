#[ctor::ctor]
fn init_test() { color_eyre::install().unwrap(); }
