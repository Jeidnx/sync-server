use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::models::User;
use crate::{DbConnection, models};

type DbError = diesel::result::Error;
use crate::schema::user::dsl::*;

pub async fn find_user_by_id(
    conn: &mut DbConnection,
    id_: &str,
) -> Result<Option<models::User>, DbError> {
    let item = user
        .filter(id.eq(id_.to_string()))
        .select(models::User::as_select())
        .first::<models::User>(conn)
        .await
        .optional()?;

    Ok(item)
}

pub async fn find_user_by_name_hash(
    conn: &mut DbConnection,
    name_hash_: &str,
) -> Result<Option<models::User>, DbError> {
    let item = user
        .filter(name_hash.eq(name_hash_.to_string()))
        .select(models::User::as_select())
        .first::<models::User>(conn)
        .await
        .optional()?;

    Ok(item)
}

pub async fn insert_new_user(
    conn: &mut DbConnection,
    new_user: &User,
) -> Result<models::User, DbError> {
    let created_user = diesel::insert_into(user)
        .values(new_user)
        .returning(models::User::as_returning())
        .get_result(conn)
        .await?;

    Ok(created_user)
}

pub async fn delete_existing_user(
    conn: &mut DbConnection,
    user_id: &str, // prevent collision with db column imported inside the function
) -> Result<(), DbError> {
    diesel::delete(user.filter(id.eq(user_id.to_string())))
        .execute(conn)
        .await?;

    Ok(())
}
