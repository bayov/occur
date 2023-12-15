use rstest::fixture;

use crate::example::user;

#[fixture]
pub fn admin_id() -> user::Id { user::Id(42) }

#[fixture]
pub fn admin_stream(admin_id: user::Id) -> user::Stream {
    user::Stream::new(admin_id)
}

#[fixture]
pub fn admin_created() -> user::Event {
    user::Event::Created { name: "admin".to_owned(), is_admin: true }
}
