use diesel_async::RunQueryDsl;

use crate::{DbConnection, database::DbError, models::Channel, schema::channel::dsl::*};

pub async fn create_or_update_channel(
    conn: &mut DbConnection,
    channel_: &Channel,
) -> Result<(), DbError> {
    // TODO: update existing entries if the channel already exists
    diesel::insert_or_ignore_into(channel)
        .values(channel_)
        .execute(conn)
        .await?;

    Ok(())
}
