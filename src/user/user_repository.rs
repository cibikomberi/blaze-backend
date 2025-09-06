// use diesel::{QueryDsl};
// use diesel_async::{AsyncConnection, RunQueryDsl};
// use uuid::Uuid;
// use crate::schema::users::dsl::users;
// use crate::user::user_model::User;
// pub struct UserRepository {}

// impl<C> UserRepository
// where
//     C: AsyncConnection
// {
//     pub async fn find_by_id(id: Uuid, conn: &mut C) -> Result<User, diesel::result::Error> {
//         users.find(id).first(conn).await
//     }

// }

