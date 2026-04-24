use diesel::{
    ExpressionMethods, OptionalExtension, SelectableHelper,
    query_dsl::methods::{FilterDsl, SelectDsl},
};
use diesel_async::RunQueryDsl;

use crate::{DbConnection, database::DbError, models::Channel, schema::channel::dsl::*};

pub async fn create_or_update_channel(
    conn: &mut DbConnection,
    channel_: &Channel,
) -> Result<(), DbError> {
    diesel::insert_into(channel)
        .values(channel_)
        .on_conflict(id)
        .do_update()
        .set(channel_)
        .execute(conn)
        .await?;

    Ok(())
}

pub async fn get_channel_by_id(
    conn: &mut DbConnection,
    channel_id: &str,
) -> Result<Option<Channel>, DbError> {
    channel
        .filter(id.eq(channel_id.to_string()))
        .select(Channel::as_select())
        .first(conn)
        .await
        .optional()
}
