use rstest::fixture;
use uuid::Uuid;

use crate::example::user;

#[fixture]
pub fn admin_id() -> user::Id { user::Id(Uuid::now_v7()) }

#[fixture]
pub fn admin_created() -> user::Event {
    user::Event::Created { name: "admin".to_owned(), is_admin: true }
}
